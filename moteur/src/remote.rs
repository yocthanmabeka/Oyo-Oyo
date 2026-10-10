//! Les données d'un autre site (ADR-116) : `Data(from: "https://…")`. C'est le serveur de
//! l'auteur (`holo serve`) qui va les chercher, jamais le navigateur du visiteur : l'autre site ne
//! voit ni l'adresse IP du visiteur, ni ses cookies, ni rien de lui ; il voit ce serveur. Les clés
//! restent ici.
//!
//! Le moteur tient les règles, jamais l'auteur :
//! 1. HTTPS seulement, vers un site écrit dans `holo-data/sites.txt`, comparé exactement (aucun
//!    sous-domaine deviné) ; une adresse IP écrite à la place d'un nom est refusée.
//! 2. Aucune demande vers ce PC ni vers le réseau privé : le nom est résolu, chaque adresse est
//!    vérifiée (IPv4 et IPv6, une adresse IPv4 portée par une IPv6 comprise), et la connexion se
//!    fait à l'adresse vérifiée, sans seconde résolution (`SafeResolver`).
//! 3. Aucune redirection suivie.
//! 4. Des bornes : 4 s pour se connecter, 8 s en tout ; 64 Ko lus au plus, coupés au-delà ; un
//!    objet JSON vérifié comme le fichier de `Data` ; 32 sites, 64 adresses gardées, 8 par site.
//! 5. Ce qui arrive est gardé : chaque adresse est demandée au plus une fois par `every` (une
//!    minute au moins, dix minutes sans `every`), quel que soit le nombre de visiteurs, et un
//!    échec est gardé une minute. Un visiteur ne force jamais une demande.
//! 6. Une clé ne sort jamais du serveur : ni dans la page, ni dans un message, ni au journal.
//! 7. Un `User-Agent` honnête ; ni cookie, ni rien du visiteur.

use std::collections::HashMap;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

use ureq::unversioned::resolver::{ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::NextTimeout;

use crate::state::{remote_address, RemoteAddress};

/// Le fichier des sites permis, dans `holo-data/` : jamais servi, jamais versionné.
pub const SITES_FILE: &str = "sites.txt";
/// Les sites permis, au plus.
pub const SITES_MAX: usize = 32;
/// Les adresses gardées, au plus, pour tout le site, et pour un même autre site.
pub const ENTRIES_MAX: usize = 64;
pub const ENTRIES_PER_SITE_MAX: usize = 8;
/// Le temps qu'une réponse est gardée quand la page n'écrit pas `every` : dix minutes (en ms).
pub const EVERY_DEFAULT: u64 = 600_000;
/// Le temps le plus court entre deux demandes à une même adresse, réussies ou non : une minute.
pub const EVERY_MIN: u64 = crate::state::REMOTE_EVERY_MIN;
/// Se connecter (le nom résolu, TLS compris), puis tout, jusqu'au dernier octet lu.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(4);
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(8);
/// Ce qu'on lit d'une réponse, au plus : autant que le fichier de données d'une page (64 Ko).
pub const BYTES_MAX: usize = crate::state::DATA_BYTES;
/// Une clé, au plus.
const KEY_MAX: usize = 512;
/// `holo-data/sites.txt`, au plus.
const SITES_FILE_MAX: u64 = 65_536;
/// Les pages regardées au démarrage, au plus, et la profondeur des dossiers.
const PAGES_SEEN_MAX: usize = 2_000;
const FOLDERS_DEPTH_MAX: usize = 8;
/// L'interrupteur des essais, lu seulement dans l'environnement de holo serve (`FakeSite`).
pub const SWITCH: &str = "HOLO_TEST_ONLY_INSECURE_SITE";
/// Ce que ce serveur dit de lui-même à l'autre site : le logiciel, sa version, d'où il vient.
pub const USER_AGENT: &str = concat!("HoloCode/", env!("CARGO_PKG_VERSION"), " (holo serve; +https://github.com/yocthanmabeka/Oyo-Oyo)");

/// La clé d'un site, rangée sur sa ligne de `holo-data/sites.txt`. Sa valeur n'est jamais écrite,
/// ni dans la page, ni dans un message, ni au journal ; même `Debug` la cache.
#[derive(Clone, PartialEq)]
pub enum Key {
    /// Le site n'en demande pas.
    None,
    /// Un paramètre ajouté à l'adresse : `?appid=…`.
    Parameter(String, String),
    /// Un en-tête : `X-Api-Key: …`.
    Header(String, String),
    /// Annoncée, mais vide ou mal écrite : le site n'est pas lu (`failed`), et le serveur le dit.
    Missing,
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Key::None => "sans clé",
            Key::Parameter(..) => "une clé, en paramètre",
            Key::Header(..) => "une clé, en en-tête",
            Key::Missing => "une clé manquante",
        })
    }
}

impl Key {
    /// La valeur de la clé : pour l'ajouter à la demande, et la chercher là où elle ne doit pas être.
    fn secret(&self) -> Option<&str> {
        match self {
            Key::Parameter(_, value) | Key::Header(_, value) => Some(value),
            _ => None,
        }
    }
}

/// Un site permis : son nom exact, et sa clé.
#[derive(Debug, Clone, PartialEq)]
pub struct Permit {
    pub host: String,
    pub key: Key,
}

/// Les en-têtes qu'une clé ne prend jamais : ceux du protocole, ceux qui diraient quelque chose du
/// visiteur ou d'un proxy, et ceux par lesquels ce serveur dit qui il est et ce qu'il attend.
const RESERVED_HEADERS: &[&str] = &[
    "host", "cookie", "cookie2", "set-cookie", "user-agent", "accept", "accept-encoding", "connection", "content-length", "content-type", "transfer-encoding", "te",
    "trailer", "upgrade", "expect", "keep-alive", "forwarded", "via", "referer", "origin", "x-real-ip",
];

fn reserved_header(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    RESERVED_HEADERS.contains(&name.as_str()) || name.starts_with("proxy-") || name.starts_with("sec-") || name.starts_with("x-forwarded-")
}

/// Lit `holo-data/sites.txt` : un site par ligne, son nom exact, puis sa clé s'il en demande une,
/// comme sa documentation la montre : `?appid=ta-clé` (un paramètre de l'adresse) ou
/// `X-Api-Key: ta-clé` (un en-tête). Les lignes vides et celles qui commencent par `#` ne comptent
/// pas. Rend les sites permis, et ce qui ne va pas, en français ; rien de ce qui suit le nom d'un
/// site n'est jamais recopié dans un message (une clé mal placée y serait).
pub fn read_sites(text: &str) -> (Vec<Permit>, Vec<String>) {
    let mut permits: Vec<Permit> = Vec::new();
    let mut problems = Vec::new();
    for (rank, line) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let say = |problem: String| format!("holo-data/{SITES_FILE}, ligne {} : {problem}", rank + 1);
        let (name, rest) = line.split_once(char::is_whitespace).map_or((line, ""), |(name, rest)| (name, rest.trim()));
        let host = name.to_ascii_lowercase();
        if host.contains('*') {
            problems.push(say("pas de « * » : chaque site s'écrit en entier, et seul ce nom exact est lu, jamais ses sous-domaines".into()));
            continue;
        }
        if host.contains("://") || host.contains('/') {
            let bare = host.split("://").last().unwrap_or("").split(['/', '?']).next().unwrap_or("").to_string();
            problems.push(say(if crate::state::check_site_name(&bare).is_ok() {
                format!("écris seulement le nom du site, sans « https:// » ni chemin : « {bare} »")
            } else {
                "écris seulement le nom du site, sans « https:// » ni chemin, comme api.exemple.org".into()
            }));
            continue;
        }
        if let Err(reason) = crate::state::check_site_name(&host) {
            problems.push(say(reason));
            continue;
        }
        if permits.iter().any(|permit| permit.host == host) {
            problems.push(say(format!("« {host} » est déjà écrit plus haut : seule la première ligne compte")));
            continue;
        }
        if permits.len() >= SITES_MAX {
            problems.push(say(format!("plus de {SITES_MAX} sites : « {host} » et les suivants sont laissés de côté")));
            break;
        }
        let key = read_key(rest).unwrap_or_else(|problem| {
            problems.push(say(format!("{host} : {problem} ; la page qui le lit recevra « failed »")));
            Key::Missing
        });
        permits.push(Permit { host, key });
    }
    (permits, problems)
}

/// La clé écrite après le nom d'un site, ou ce qui ne va pas (sans jamais la recopier).
fn read_key(written: &str) -> Result<Key, String> {
    if written.is_empty() {
        return Ok(Key::None);
    }
    if let Some(parameter) = written.strip_prefix('?') {
        let (name, value) = parameter.split_once('=').ok_or("une clé en paramètre s'écrit « ?nom=clé », comme ?appid=ta-clé")?;
        if name.is_empty() || name.len() > 64 || !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')) {
            return Err("le nom du paramètre de la clé s'écrit avec des lettres, des chiffres, « _ », « - » ou « . »".into());
        }
        if value.is_empty() {
            return Err("la clé manque : écris-la après le « = »".into());
        }
        if value.len() > KEY_MAX || !value.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(format!("une clé en paramètre s'écrit d'un seul tenant, sans espace ni accent, {KEY_MAX} caractères au plus"));
        }
        return Ok(Key::Parameter(name.to_string(), value.to_string()));
    }
    let (name, value) = written.split_once(':').ok_or("après le nom du site, une clé s'écrit « ?nom=clé » (un paramètre de l'adresse) ou « Nom: clé » (un en-tête)")?;
    let (name, value) = (name.trim(), value.trim());
    if name.is_empty() || name.len() > 64 || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("le nom de l'en-tête de la clé s'écrit avec des lettres, des chiffres et des tirets, comme X-Api-Key".into());
    }
    if reserved_header(name) {
        return Err("cet en-tête n'est pas permis pour une clé : il appartient au protocole, au visiteur ou à ce serveur".into());
    }
    if value.is_empty() {
        return Err("la clé manque : écris-la après le « : »".into());
    }
    if value.len() > KEY_MAX || !value.bytes().all(|b| b == b' ' || b.is_ascii_graphic()) {
        return Err(format!("une clé en en-tête s'écrit sans accent, {KEY_MAX} caractères au plus"));
    }
    Ok(Key::Header(name.to_string(), value.to_string()))
}

/// Une adresse joignable sur Internet, et rien d'autre (règle 2) : ni ce PC (bouclage), ni le
/// réseau privé, ni le lien local, ni la diffusion de groupe (multicast), ni les adresses
/// réservées, de documentation ou non routables. Une adresse IPv4 portée par une IPv6
/// (`::ffff:127.0.0.1`, ou `64:ff9b::7f00:1` du NAT64) est jugée comme l'IPv4 qu'elle porte.
pub fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => public_v4(ip),
        IpAddr::V6(ip) => public_v6(ip),
    }
}

fn public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(a == 0 // « ce réseau », 0.0.0.0/8
        || a == 10 // privé
        || (a == 100 && (64..=127).contains(&b)) // partagé par les opérateurs (CGNAT), 100.64.0.0/10
        || a == 127 // ce PC (bouclage)
        || (a == 169 && b == 254) // lien local, dont 169.254.169.254 des nuages
        || (a == 172 && (16..=31).contains(&b)) // privé
        || (a == 192 && b == 0 && (c == 0 || c == 2)) // protocoles de l'IETF ; documentation
        || (a == 192 && b == 88 && c == 99) // relais 6to4, abandonné
        || (a == 192 && b == 168) // privé
        || (a == 198 && (b == 18 || b == 19)) // mesures de performance, 198.18.0.0/15
        || (a == 198 && b == 51 && c == 100) // documentation
        || (a == 203 && b == 0 && c == 113) // documentation
        || a >= 224) // multicast (224/4), réservé (240/4), diffusion
}

fn public_v6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    let carried = || Ipv4Addr::new((s[6] >> 8) as u8, s[6] as u8, (s[7] >> 8) as u8, s[7] as u8);
    // ::ffff:a.b.c.d (une IPv4 portée), 64:ff9b::a.b.c.d (le NAT64 d'un réseau seulement IPv6).
    if s[..6] == [0, 0, 0, 0, 0, 0xffff] || s[..6] == [0x64, 0xff9b, 0, 0, 0, 0] {
        return public_v4(carried());
    }
    // Sinon, seulement l'unicast mondial (2000::/3) : ni ::, ni ::1, ni fc00::/7 (privé), ni
    // fe80::/10 (lien local), ni ff00::/8 (multicast)… ; et, dedans, ni les protocoles de l'IETF
    // (2001::/23, dont Teredo), ni la documentation (2001:db8::/32, 3fff::/20), ni 6to4 (2002::/16).
    (s[0] & 0xe000) == 0x2000 && !(s[0] == 0x2001 && (s[1] < 0x0200 || s[1] == 0x0db8)) && s[0] != 0x2002 && !(s[0] == 0x3fff && s[1] < 0x1000)
}

/// Pourquoi une demande n'a pas abouti, sans rien de l'adresse ni de la clé.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Failure {
    /// Le nom mène à ce PC ou au réseau privé (règle 2).
    Private,
    /// Le nom est inconnu, ou le réseau est coupé.
    Unresolved,
    /// Trop lent (règle 4).
    Timeout,
    /// Injoignable : la connexion, ou le certificat, refusés.
    Unreachable,
    /// Plus de 64 Ko (règle 4).
    TooBig,
}

/// Les adresses d'un nom, vérifiées (règle 2) : toutes permises, sinon aucune. Une réponse du DNS
/// qui mêle une adresse publique et une adresse privée est refusée entière : c'est un piège connu.
pub fn checked(addresses: Vec<SocketAddr>, allowed: fn(IpAddr) -> bool) -> Result<Vec<SocketAddr>, Failure> {
    if addresses.is_empty() {
        return Err(Failure::Unresolved);
    }
    if addresses.iter().any(|address| !allowed(address.ip())) {
        return Err(Failure::Private);
    }
    Ok(addresses)
}

/// Ce qui donne les adresses d'un nom : le système (DNS), ou un faux dans les essais.
pub type Lookup = Arc<dyn Fn(&str, u16) -> std::io::Result<Vec<SocketAddr>> + Send + Sync>;

/// La résolution du système.
pub fn system_lookup() -> Lookup {
    Arc::new(|host, port| (host, port).to_socket_addrs().map(Iterator::collect))
}

/// Le refus d'une adresse de ce PC ou du réseau privé, porté par l'erreur que rend le résolveur.
#[derive(Debug)]
struct PrivateAddress;

impl std::fmt::Display for PrivateAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("adresse de ce PC ou du réseau privé : refusée")
    }
}

impl std::error::Error for PrivateAddress {}

/// Le résolveur que ureq emploie (règle 2) : il résout le nom une fois, vérifie chaque adresse,
/// et ne rend que des adresses vérifiées ; ureq se connecte à celles-là, sans résoudre une seconde
/// fois. Un DNS qui changerait de réponse entre la vérification et la connexion ne passe donc pas.
#[derive(Clone)]
struct SafeResolver {
    lookup: Lookup,
    allowed: fn(IpAddr) -> bool,
}

impl std::fmt::Debug for SafeResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SafeResolver")
    }
}

impl Resolver for SafeResolver {
    fn resolve(&self, uri: &ureq::http::Uri, _: &ureq::config::Config, timeout: NextTimeout) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let host = uri.host().ok_or(ureq::Error::HostNotFound)?.to_string();
        let port = uri.port_u16().unwrap_or(if uri.scheme_str() == Some("http") { 80 } else { 443 });
        // La résolution du système n'a pas de délai : elle se fait à côté, et on ne l'attend pas
        // au-delà du temps qui reste.
        let wait = if timeout.after.is_not_happening() { CONNECT_TIMEOUT } else { *timeout.after };
        let (lookup, (sender, receiver)) = (Arc::clone(&self.lookup), std::sync::mpsc::sync_channel(1));
        std::thread::spawn(move || {
            let _ = sender.send(lookup(&host, port));
        });
        let found = match receiver.recv_timeout(wait) {
            Ok(Ok(found)) => found,
            Ok(Err(_)) => return Err(ureq::Error::HostNotFound),
            Err(_) => return Err(ureq::Error::Timeout(timeout.reason)),
        };
        let found = checked(found, self.allowed).map_err(|failure| match failure {
            Failure::Private => ureq::Error::Io(std::io::Error::new(std::io::ErrorKind::PermissionDenied, PrivateAddress)),
            _ => ureq::Error::HostNotFound,
        })?;
        let mut addresses = self.empty();
        for address in found {
            if addresses.try_push(address).is_err() {
                break;
            }
        }
        Ok(addresses)
    }
}

/// Le résolveur de l'interrupteur des essais : toujours 127.0.0.1, au port du faux site.
#[derive(Debug, Clone)]
struct LoopbackResolver(u16);

impl Resolver for LoopbackResolver {
    fn resolve(&self, _: &ureq::http::Uri, _: &ureq::config::Config, _: NextTimeout) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let mut addresses = self.empty();
        addresses.push(SocketAddr::from(([127, 0, 0, 1], self.0)));
        Ok(addresses)
    }
}

/// Ce que le serveur demande à un autre site : une demande GET, déjà vérifiée.
#[derive(Clone, PartialEq)]
pub struct Outgoing {
    /// L'adresse complète, avec la clé si elle va dans un paramètre : jamais écrite au journal.
    pub address: String,
    /// Le nom du site.
    pub host: String,
    /// Tous les en-têtes ajoutés : `User-Agent`, `Accept`, et la clé si elle va dans un en-tête.
    pub headers: Vec<(String, String)>,
}

impl std::fmt::Debug for Outgoing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = self.headers.iter().map(|(name, _)| name.as_str()).collect();
        write!(f, "GET https://{}/… (en-têtes : {})", self.host, names.join(", "))
    }
}

/// Ce qu'un autre site a répondu : son code, et son contenu (pour un code 200 seulement).
#[derive(Debug, Clone, PartialEq)]
pub struct Incoming {
    pub status: u16,
    pub body: Vec<u8>,
}

/// Le transport : le vrai client HTTPS, ou un faux dans les essais, pour éprouver ce qui est
/// gardé, les bornes, le JSON et les refus sans réseau.
pub trait Transport: Send + Sync {
    fn get(&self, request: &Outgoing) -> Result<Incoming, Failure>;
}

/// Lit une réponse sans jamais garder plus de `max` octets : au-delà, la lecture s'arrête et rien
/// n'est pris. Une réponse sans fin n'occupe pas la mémoire du serveur.
pub fn read_capped(reader: impl Read, max: usize) -> Result<Vec<u8>, Failure> {
    let mut body = Vec::new();
    match reader.take(max as u64 + 1).read_to_end(&mut body) {
        Ok(_) if body.len() > max => Err(Failure::TooBig),
        Ok(_) => Ok(body),
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => Err(Failure::Timeout),
        Err(error) if error.get_ref().is_some_and(|inner| inner.to_string().contains("timeout") || inner.to_string().contains("timed out")) => Err(Failure::Timeout),
        Err(_) => Err(Failure::Unreachable),
    }
}

/// Une erreur de ureq, réduite à sa raison : son texte n'est jamais repris (il pourrait contenir
/// l'adresse, donc une clé en paramètre).
fn failure_of(error: &ureq::Error) -> Failure {
    match error {
        ureq::Error::Io(io) if io.get_ref().is_some_and(|inner| inner.is::<PrivateAddress>()) => Failure::Private,
        ureq::Error::Io(io) if io.kind() == std::io::ErrorKind::TimedOut => Failure::Timeout,
        ureq::Error::Timeout(_) => Failure::Timeout,
        ureq::Error::HostNotFound => Failure::Unresolved,
        ureq::Error::BodyExceedsLimit(_) => Failure::TooBig,
        _ => Failure::Unreachable,
    }
}

/// Les réglages de ureq (règles 2, 3, 4 et 7) : aucun proxy, même s'il y en a un dans
/// l'environnement (il résoudrait le nom à la place de `SafeResolver`) ; aucune redirection ; les
/// délais ; aucune connexion gardée ouverte ; notre `User-Agent`. ureq est compilé sans cookies ni
/// compression. `https_only` : faux seulement pour l'interrupteur des essais.
fn agent(https_only: bool, connect: Duration, total: Duration, resolver: impl Resolver) -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .https_only(https_only)
        .proxy(None)
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_connect(Some(connect))
        .timeout_global(Some(total))
        .max_idle_connections(0)
        .max_idle_connections_per_host(0)
        .user_agent(USER_AGENT)
        .accept("application/json")
        .accept_encoding("")
        .build();
    ureq::Agent::with_parts(config, ureq::unversioned::transport::DefaultConnector::default(), resolver)
}

/// Le vrai client : HTTPS avec rustls et les certificats racines de Mozilla (webpki-roots).
pub struct Https {
    agent: ureq::Agent,
    fake: Option<(FakeSite, ureq::Agent)>,
}

impl Https {
    pub fn new(lookup: Lookup, fake: Option<FakeSite>, connect: Duration, total: Duration) -> Https {
        let fake = fake.map(|site| {
            let port = site.port;
            (site, agent(false, connect, total, LoopbackResolver(port)))
        });
        Https { agent: agent(true, connect, total, SafeResolver { lookup, allowed: is_public }), fake }
    }
}

impl Transport for Https {
    fn get(&self, request: &Outgoing) -> Result<Incoming, Failure> {
        let (agent, address) = match &self.fake {
            Some((site, agent)) if site.host == request.host => {
                (agent, request.address.replacen(&format!("https://{}", site.host), &format!("http://{}:{}", site.host, site.port), 1))
            }
            _ => (&self.agent, request.address.clone()),
        };
        let mut call = agent.get(&address);
        for (name, value) in &request.headers {
            call = call.header(name, value);
        }
        let mut response = call.call().map_err(|error| failure_of(&error))?;
        let status = response.status().as_u16();
        if status != 200 {
            return Ok(Incoming { status, body: Vec::new() });
        }
        // Une taille annoncée trop grande : refusée avant de rien lire.
        if response.body().content_length().is_some_and(|length| length > BYTES_MAX as u64) {
            return Err(Failure::TooBig);
        }
        let body = read_capped(response.body_mut().as_reader(), BYTES_MAX)?;
        Ok(Incoming { status, body })
    }
}

/// Le client HTTPS, fabriqué à la première demande : un site qui ne lit aucun autre site ne
/// prépare jamais TLS.
struct LazyHttps {
    fake: Option<FakeSite>,
    client: OnceLock<Https>,
}

impl Transport for LazyHttps {
    fn get(&self, request: &Outgoing) -> Result<Incoming, Failure> {
        self.client.get_or_init(|| Https::new(system_lookup(), self.fake.clone(), CONNECT_TIMEOUT, TOTAL_TIMEOUT)).get(request)
    }
}

/// L'interrupteur des essais : `HOLO_TEST_ONLY_INSECURE_SITE=meteo.test:43210`. Le site
/// `meteo.test` (déclaré comme les autres dans `holo-data/sites.txt`) est alors lu en HTTP clair
/// sur 127.0.0.1:43210, pour qu'un essai dans Chrome ait un faux « autre site » sur ce PC. Toutes
/// les autres règles restent. Éteint par défaut ; lu seulement dans l'environnement, au démarrage
/// de holo serve (aucune page, aucune demande ne l'allume) ; seulement pour un nom en `.test`, qui
/// n'existe jamais sur Internet ; annoncé au démarrage.
#[derive(Debug, Clone, PartialEq)]
pub struct FakeSite {
    pub host: String,
    pub port: u16,
}

/// Lit l'interrupteur des essais : `None` (absent) l'éteint ; mal écrit, holo serve refuse de démarrer.
pub fn fake_site(value: Option<&str>) -> Result<Option<FakeSite>, String> {
    let Some(value) = value else { return Ok(None) };
    let refusal = || format!("{SWITCH} : « nom.test:port » attendu, comme meteo.test:43210 (un nom en .test, jamais un vrai site)");
    let (host, port) = value.rsplit_once(':').ok_or_else(refusal)?;
    let port = port.parse::<u16>().ok().filter(|port| *port > 0).ok_or_else(refusal)?;
    if !host.ends_with(".test") || crate::state::check_site_name(host).is_err() {
        return Err(refusal());
    }
    Ok(Some(FakeSite { host: host.to_string(), port }))
}

/// Pourquoi les données d'un autre site ne sont pas arrivées : la page reçoit `failed` ; l'auteur
/// lit la raison au journal. Aucune ne contient jamais une clé.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    /// L'adresse écrite dans la page n'est pas celle d'un autre site (règle 1).
    Address(String),
    /// Le site n'est pas dans `holo-data/sites.txt` (règle 1).
    NotDeclared(String),
    /// Sa clé est vide ou mal écrite (règle 6).
    MissingKey(String),
    /// L'adresse écrite dans la page contient la clé (règle 6).
    KeyInPage(String),
    /// L'adresse écrite dans la page donne déjà le paramètre de la clé (règle 6).
    KeyNameInPage(String),
    /// Trop d'adresses gardées à la fois (règle 4).
    TooMany(String),
    /// Le site répond encore à une autre demande, trop longtemps.
    Busy(String),
    /// Le site n'a pas répondu il y a moins d'une minute : pas de nouvelle demande avant (règle 5).
    Recent(String),
    Private(String),
    Unresolved(String),
    Timeout(String),
    Unreachable(String),
    /// Une redirection, jamais suivie (règle 3).
    Redirect(String, u16),
    Status(String, u16),
    TooBig(String),
    NotJson(String),
    /// La réponse contient la clé (règle 6).
    KeyEchoed(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file = format!("holo-data/{SITES_FILE}");
        match self {
            Refusal::Address(reason) => write!(f, "l'adresse de l'autre site est refusée : {reason}"),
            Refusal::NotDeclared(host) => write!(f, "« {host} » n'est pas dans {file} : ajoute la ligne « {host} » pour que tes pages le lisent"),
            Refusal::MissingKey(host) => write!(f, "la clé de {host} manque ou est mal écrite dans {file} : rien n'est demandé"),
            Refusal::KeyInPage(host) => write!(f, "l'adresse écrite dans la page contient la clé de {host} : retire-la de la page (elle est publique) et demande une nouvelle clé au site"),
            Refusal::KeyNameInPage(host) => write!(f, "l'adresse écrite dans la page donne déjà le paramètre de la clé de {host} : la clé s'écrit seulement dans {file}"),
            Refusal::TooMany(host) => write!(f, "trop d'adresses d'autres sites gardées à la fois ({ENTRIES_MAX} au plus, {ENTRIES_PER_SITE_MAX} par site) : {host} attendra qu'une place se libère"),
            Refusal::Busy(host) => write!(f, "{host} n'a pas fini de répondre à une autre demande"),
            Refusal::Recent(host) => write!(f, "{host} n'a pas répondu il y a moins d'une minute : pas de nouvelle demande avant"),
            Refusal::Private(host) => write!(f, "{host} mène à une adresse de ce PC ou du réseau privé : refusé"),
            Refusal::Unresolved(host) => write!(f, "{host} est introuvable (nom inconnu, ou pas de réseau)"),
            Refusal::Timeout(host) => write!(f, "{host} n'a pas répondu à temps ({} s pour se connecter, {} s en tout)", CONNECT_TIMEOUT.as_secs(), TOTAL_TIMEOUT.as_secs()),
            Refusal::Unreachable(host) => write!(f, "{host} est injoignable (connexion ou certificat refusés)"),
            Refusal::Redirect(host, status) => write!(f, "{host} répond par une redirection ({status}), qui n'est jamais suivie : écris dans la page l'adresse où elle mène"),
            Refusal::Status(host, status @ (401 | 403)) => write!(f, "{host} refuse la demande ({status}) : demande-t-il une clé ? Elle s'écrit sur sa ligne de {file}"),
            Refusal::Status(host, status) => write!(f, "{host} a répondu {status} au lieu de 200"),
            Refusal::TooBig(host) => write!(f, "la réponse de {host} dépasse {} Ko : coupée, rien n'est pris", BYTES_MAX / 1024),
            Refusal::NotJson(host) => write!(f, "la réponse de {host} n'est pas un objet JSON que la page sait lire"),
            Refusal::KeyEchoed(host) => write!(f, "la réponse de {host} contient sa clé : refusée, pour que la clé n'arrive jamais dans une page"),
        }
    }
}

impl Refusal {
    fn of(failure: Failure, host: &str) -> Refusal {
        let host = host.to_string();
        match failure {
            Failure::Private => Refusal::Private(host),
            Failure::Unresolved => Refusal::Unresolved(host),
            Failure::Timeout => Refusal::Timeout(host),
            Failure::Unreachable => Refusal::Unreachable(host),
            Failure::TooBig => Refusal::TooBig(host),
        }
    }
}

/// Ce qui est gardé pour une adresse : une demande en cours, une réponse arrivée, ou un échec.
enum Kept {
    Asking,
    Arrived { json: String, at: u64 },
    Failed { at: u64 },
}

struct Entry {
    host: String,
    kept: Kept,
}

/// Les sites permis, et de quoi savoir si `holo-data/sites.txt` a changé.
#[derive(Default)]
struct Sites {
    stamp: Option<(SystemTime, u64)>,
    read: bool,
    permits: Vec<Permit>,
    problems: Vec<String>,
}

/// Les autres sites d'un site servi : ceux qui sont permis, ce qui est gardé, le transport.
pub struct Remote {
    file: PathBuf,
    sites: Mutex<Sites>,
    cache: Mutex<HashMap<String, Entry>>,
    arrived: Condvar,
    /// Les refus déjà écrits au journal : une fois par minute et par adresse, pas à chaque visiteur.
    said: Mutex<HashMap<String, u64>>,
    transport: Box<dyn Transport>,
    /// Le temps, en millisecondes ; une horloge qu'on avance, dans les essais.
    clock: Box<dyn Fn() -> u64 + Send + Sync>,
    /// Le journal de holo serve ; une liste, dans les essais.
    log: Box<dyn Fn(&str) + Send + Sync>,
    /// L'interrupteur des essais, s'il est allumé (jamais par défaut).
    pub fake: Option<FakeSite>,
}

/// Sous quel nom une adresse est gardée : `https://`, son nom de site en minuscules, son chemin.
fn kept_as(address: &RemoteAddress) -> String {
    format!("https://{}{}", address.host, address.target)
}

/// Le temps qu'une réponse est gardée pour une page : son `every`, une minute au moins ; dix
/// minutes sans `every` (en millisecondes).
fn keep_for(every: u64) -> u64 {
    if every == 0 { EVERY_DEFAULT } else { every.max(EVERY_MIN) }
}

/// Un caractère d'une clé, écrit pour une adresse : tel quel s'il ne demande rien, sinon en %XX.
fn encoded(text: &str) -> String {
    text.bytes().map(|b| if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

/// Les noms des paramètres d'une adresse (`?q=Kinshasa&appid=…` → `q`, `appid`), tels qu'écrits et décodés.
fn parameter_names(target: &str) -> Vec<String> {
    let Some((_, query)) = target.split_once('?') else { return Vec::new() };
    query.split('&').filter_map(|pair| pair.split('=').next()).flat_map(|name| [name.to_string(), crate::address::decode(name).unwrap_or_default()]).collect()
}

impl Remote {
    /// Les autres sites du dossier servi : `holo-data/sites.txt`, le vrai client HTTPS, et
    /// l'interrupteur des essais lu dans l'environnement (éteint s'il n'y est pas).
    pub fn open(folder: &Path) -> Result<Remote, String> {
        let switch = std::env::var_os(SWITCH).map(|value| value.to_string_lossy().into_owned());
        let fake = fake_site(switch.as_deref())?;
        let start = Instant::now();
        Ok(Remote::with_parts(
            folder,
            Box::new(LazyHttps { fake: fake.clone(), client: OnceLock::new() }),
            fake,
            Box::new(move || start.elapsed().as_millis() as u64),
            Box::new(|line| println!("{line}")),
        ))
    }

    /// Les mêmes, avec un transport, une horloge et un journal à soi (les essais).
    pub fn with_parts(folder: &Path, transport: Box<dyn Transport>, fake: Option<FakeSite>, clock: Box<dyn Fn() -> u64 + Send + Sync>, log: Box<dyn Fn(&str) + Send + Sync>) -> Remote {
        Remote {
            file: folder.join(crate::server::DATA_FOLDER).join(SITES_FILE),
            sites: Mutex::default(),
            cache: Mutex::default(),
            arrived: Condvar::new(),
            said: Mutex::default(),
            transport,
            clock,
            log,
            fake,
        }
    }

    /// Relit `holo-data/sites.txt` s'il a changé (sa date ou sa taille) : l'auteur l'écrit pendant
    /// que le serveur tourne, la demande suivante le voit, et le journal dit ce qui ne va pas.
    /// Rend vrai si le fichier vient d'être relu après un changement.
    fn refresh(&self, sites: &mut Sites) -> bool {
        // Un fichier ordinaire seulement : ni un dossier, ni un tube qui bloquerait la lecture.
        let stamp = std::fs::metadata(&self.file).ok().filter(std::fs::Metadata::is_file).and_then(|meta| Some((meta.modified().ok()?, meta.len())));
        if sites.read && stamp == sites.stamp {
            return false;
        }
        let text = match stamp {
            Some((_, length)) if length <= SITES_FILE_MAX => std::fs::read(&self.file).map(|bytes| String::from_utf8_lossy(&bytes).into_owned()).unwrap_or_default(),
            Some(_) => {
                (self.log)(&format!("Autres sites    : holo-data/{SITES_FILE} dépasse {} Ko : il n'est pas lu", SITES_FILE_MAX / 1024));
                String::new()
            }
            None => String::new(),
        };
        let (permits, problems) = read_sites(&text);
        let again = sites.read;
        *sites = Sites { stamp, read: true, permits, problems };
        if again {
            for line in summary(sites, stamp.is_some()) {
                (self.log)(&line);
            }
            // Un échec gardé peut venir d'une clé qui manquait : le changement s'essaie tout de suite.
            if let Ok(mut cache) = self.cache.lock() {
                cache.retain(|_, entry| !matches!(entry.kept, Kept::Failed { .. }));
            }
        }
        again
    }

    fn permit(&self, host: &str) -> Option<Permit> {
        let mut sites = self.sites.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.refresh(&mut sites);
        sites.permits.iter().find(|permit| permit.host == host).cloned()
    }

    /// Ce que holo serve dit au démarrage : l'interrupteur des essais s'il est allumé, les sites
    /// permis (jamais leur clé), ce qui ne va pas dans `holo-data/sites.txt`, et les pages qui lisent
    /// un site qui n'y est pas, ou dont la clé manque. Rien, si aucune page ne lit d'autre site et
    /// que le fichier n'existe pas.
    pub fn announce(&self, folder: &Path) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(fake) = &self.fake {
            lines.push(format!(
                "ESSAIS SEULEMENT : {SWITCH}={}:{} : https://{}/ est lu en HTTP clair sur 127.0.0.1:{}. Jamais pour un vrai site.",
                fake.host, fake.port, fake.host, fake.port
            ));
        }
        let mut sites = self.sites.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.refresh(&mut sites);
        let pages = pages_reading_other_sites(folder);
        if sites.stamp.is_none() && pages.is_empty() {
            return lines;
        }
        lines.extend(summary(&sites, sites.stamp.is_some()));
        for (page, host) in pages {
            match sites.permits.iter().find(|permit| permit.host == host) {
                None => lines.push(format!("                  {page} lit {host}, qui n'est pas dans holo-data/{SITES_FILE} : la page recevra « failed » ; pour le permettre, ajoute la ligne « {host} »")),
                Some(permit) if permit.key == Key::Missing => lines.push(format!("                  {page} lit {host}, dont la clé manque dans holo-data/{SITES_FILE} : la page recevra « failed »")),
                Some(_) => {}
            }
        }
        lines
    }

    /// Ce qui est gardé pour une adresse, sans jamais demander ni attendre : sous le verrou des
    /// gestes, un site lent retiendrait tous les visiteurs. `None` : rien de frais.
    pub fn kept(&self, written: &str, every: u64) -> Option<Result<String, ()>> {
        let key = remote_address(written).ok().map(|address| kept_as(&address))?;
        let now = (self.clock)();
        let cache = self.cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        match cache.get(&key).map(|entry| &entry.kept) {
            Some(Kept::Arrived { json, at }) if now < at + keep_for(every) => Some(Ok(json.clone())),
            Some(Kept::Failed { at }) if now < at + EVERY_MIN => Some(Err(())),
            _ => None,
        }
    }

    /// Les données d'un autre site pour une page (ADR-116) : depuis ce qui est gardé, si c'est
    /// encore frais ; sinon une demande, une seule à la fois pour une même adresse, que les autres
    /// visiteurs attendent. `page` : la page, pour le journal ; `written` : l'adresse écrite dans
    /// la page ; `every` : son rythme en millisecondes (0 : une seule fois, à l'ouverture). Une
    /// réponse est gardée ce temps-là, une minute au moins, dix minutes sans `every`. Rend le JSON
    /// vérifié, ou la raison du refus (écrite au journal).
    pub fn read(&self, page: &str, written: &str, every: u64) -> Result<String, Refusal> {
        let address = remote_address(written).map_err(|reason| self.refused(page, written, Refusal::Address(reason)))?;
        let host = address.host.clone();
        let Some(permit) = self.permit(&host) else { return Err(self.refused(page, written, Refusal::NotDeclared(host))) };
        if permit.key == Key::Missing {
            return Err(self.refused(page, written, Refusal::MissingKey(host)));
        }
        // La clé ne s'écrit jamais dans la page : ni elle, ni le paramètre qui la porte.
        if permit.key.secret().is_some_and(|secret| written.contains(secret) || written.contains(&encoded(secret))) {
            return Err(self.refused(page, written, Refusal::KeyInPage(host)));
        }
        if let Key::Parameter(name, _) = &permit.key {
            if parameter_names(&address.target).iter().any(|written_name| written_name == name) {
                return Err(self.refused(page, written, Refusal::KeyNameInPage(host)));
            }
        }
        let keep = keep_for(every);
        // Une même adresse, écrite en majuscules ou non, n'est gardée (et demandée) qu'une fois.
        let key = kept_as(&address);
        let deadline = Instant::now() + TOTAL_TIMEOUT + Duration::from_secs(2);
        let mut cache = self.cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        loop {
            let now = (self.clock)();
            match cache.get(&key).map(|entry| &entry.kept) {
                Some(Kept::Arrived { json, at }) if now < at + keep => return Ok(json.clone()),
                Some(Kept::Failed { at }) if now < at + EVERY_MIN => return Err(Refusal::Recent(host)),
                Some(Kept::Asking) => {
                    let left = deadline.saturating_duration_since(Instant::now());
                    if left.is_zero() {
                        return Err(Refusal::Busy(host));
                    }
                    cache = self.arrived.wait_timeout(cache, left).map(|(cache, _)| cache).unwrap_or_else(|poisoned| poisoned.into_inner().0);
                }
                _ => break,
            }
        }
        if !cache.contains_key(&key) && !room(&mut cache, &host, (self.clock)()) {
            drop(cache);
            return Err(self.refused(page, written, Refusal::TooMany(host)));
        }
        cache.insert(key.clone(), Entry { host: host.clone(), kept: Kept::Asking });
        drop(cache);
        // Si la demande s'interrompait (une panique), l'adresse ne resterait pas « en cours ».
        let mut pending = Pending { remote: self, key: &key, done: false };
        let outcome = self.ask(page, &address, &permit, keep);
        pending.finish(outcome.as_ref().ok().cloned());
        outcome
    }

    /// La seule voie vers un autre site : la demande, puis ce qu'on vérifie de la réponse.
    fn ask(&self, page: &str, address: &RemoteAddress, permit: &Permit, keep: u64) -> Result<String, Refusal> {
        let host = &address.host;
        let mut target = address.target.clone();
        let mut headers = vec![("User-Agent".to_string(), USER_AGENT.to_string()), ("Accept".to_string(), "application/json".to_string())];
        match &permit.key {
            Key::Parameter(name, value) => target.push_str(&format!("{}{name}={}", if target.contains('?') { '&' } else { '?' }, encoded(value))),
            Key::Header(name, value) => headers.push((name.clone(), value.clone())),
            Key::None | Key::Missing => {}
        }
        let request = Outgoing { address: format!("https://{host}{target}"), host: host.clone(), headers };
        let outcome = match self.transport.get(&request) {
            Err(failure) => Err(Refusal::of(failure, host)),
            Ok(Incoming { status: 200, body }) => checked_json(body, permit, host),
            Ok(Incoming { status, .. }) if (300..400).contains(&status) => Err(Refusal::Redirect(host.clone(), status)),
            Ok(Incoming { status, .. }) => Err(Refusal::Status(host.clone(), status)),
        };
        (self.log)(&match &outcome {
            Ok(json) => format!("Autre site      : {page} : {host} a répondu, {} octets, gardés {} s", json.len(), keep / 1000),
            Err(refusal) => format!("Autre site      : {page} : {refusal}"),
        });
        outcome
    }

    /// Un refus avant toute demande : au journal une fois par minute et par adresse, pas à chaque visiteur.
    fn refused(&self, page: &str, written: &str, refusal: Refusal) -> Refusal {
        let now = (self.clock)();
        let mut said = self.said.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let key = format!("{written}\n{:?}", std::mem::discriminant(&refusal));
        if said.get(&key).is_none_or(|at| now >= at + EVERY_MIN) {
            if said.len() >= 256 {
                said.clear();
            }
            said.insert(key, now);
            (self.log)(&format!("Autre site      : {page} : {refusal}"));
        }
        refusal
    }
}

/// Une demande en cours : quand elle finit, sa réponse (ou son échec) est gardée, et les visiteurs
/// qui l'attendaient sont réveillés ; si elle s'interrompt, elle compte comme un échec.
struct Pending<'a> {
    remote: &'a Remote,
    key: &'a str,
    done: bool,
}

impl Pending<'_> {
    fn finish(&mut self, json: Option<String>) {
        let at = (self.remote.clock)();
        let mut cache = self.remote.cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = cache.get_mut(self.key) {
            entry.kept = match json {
                Some(json) => Kept::Arrived { json, at },
                None => Kept::Failed { at },
            };
        }
        self.done = true;
        self.remote.arrived.notify_all();
    }
}

impl Drop for Pending<'_> {
    fn drop(&mut self) {
        if !self.done {
            self.finish(None);
        }
    }
}

/// Une place pour une nouvelle adresse (règle 4) : 64 adresses au plus, 8 par site. Une place se
/// libère seulement d'une adresse demandée il y a une minute au moins : chaque place sert donc au
/// plus une demande par minute, et un site reçoit au plus 8 demandes par minute de ce serveur.
fn room(cache: &mut HashMap<String, Entry>, host: &str, now: u64) -> bool {
    let free = |cache: &mut HashMap<String, Entry>, of_host: bool| {
        let oldest = cache
            .iter()
            .filter(|(_, entry)| !of_host || entry.host == host)
            .filter_map(|(written, entry)| match entry.kept {
                Kept::Arrived { at, .. } | Kept::Failed { at } if now >= at + EVERY_MIN => Some((at, written.clone())),
                _ => None,
            })
            .min();
        oldest.map(|(_, written)| cache.remove(&written)).is_some()
    };
    if cache.values().filter(|entry| entry.host == host).count() >= ENTRIES_PER_SITE_MAX && !free(cache, true) {
        return false;
    }
    cache.len() < ENTRIES_MAX || free(cache, false)
}

/// Ce qu'on prend d'une réponse (règles 4 et 6) : du texte UTF-8, un objet JSON que la page sait
/// lire (comme le fichier de `Data` : 64 Ko, trois niveaux), et jamais la clé du site.
fn checked_json(body: Vec<u8>, permit: &Permit, host: &str) -> Result<String, Refusal> {
    if body.len() > BYTES_MAX {
        return Err(Refusal::TooBig(host.to_string()));
    }
    let Ok(json) = String::from_utf8(body) else { return Err(Refusal::NotJson(host.to_string())) };
    if let Some(secret) = permit.key.secret() {
        if json.contains(secret) || json.contains(&encoded(secret)) || json.contains(&secret.replace('/', "\\/")) {
            return Err(Refusal::KeyEchoed(host.to_string()));
        }
    }
    if !crate::lists::is_json_object(&json) {
        return Err(Refusal::NotJson(host.to_string()));
    }
    Ok(json)
}

/// Les sites permis, en une ligne (jamais une clé), puis ce qui ne va pas dans le fichier.
fn summary(sites: &Sites, file_there: bool) -> Vec<String> {
    let mut lines = vec![if !file_there {
        format!("Autres sites    : aucun permis (holo-data/{SITES_FILE} absent) ; une page ne lit un autre site que s'il y est écrit")
    } else if sites.permits.is_empty() {
        format!("Autres sites    : aucun permis (holo-data/{SITES_FILE})")
    } else {
        let named: Vec<String> = sites
            .permits
            .iter()
            .map(|permit| match permit.key {
                Key::None => permit.host.clone(),
                _ => format!("{} ({:?})", permit.host, permit.key),
            })
            .collect();
        format!("Autres sites    : {} permis (holo-data/{SITES_FILE}) : {}", sites.permits.len(), named.join(" ; "))
    }];
    lines.extend(sites.problems.iter().map(|problem| format!("                  {problem}")));
    lines
}

/// Les pages du dossier qui lisent un autre site, et ce site : pour les dire au démarrage.
fn pages_reading_other_sites(folder: &Path) -> Vec<(String, String)> {
    fn walk(folder: &Path, prefix: &str, depth: usize, seen: &mut usize, found: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(folder) else { return };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || (depth == 0 && name == crate::server::DATA_FOLDER) || *seen >= PAGES_SEEN_MAX {
                continue;
            }
            let path = entry.path();
            if path.is_dir() && depth < FOLDERS_DEPTH_MAX {
                walk(&path, &format!("{prefix}{name}/"), depth + 1, seen, found);
            } else if name.ends_with(".holo") && entry.metadata().is_ok_and(|meta| meta.len() <= 262_144) {
                *seen += 1;
                let Ok(text) = std::fs::read_to_string(&path) else { continue };
                let Ok(program) = crate::holo::read(&text) else { continue };
                if let Some((from, _)) = crate::state::data_source(&program).ok().flatten().filter(|(from, _)| crate::state::is_remote(from)) {
                    if let Ok(address) = remote_address(&from) {
                        found.push((format!("{prefix}{name}"), address.host));
                    }
                }
            }
        }
    }
    let mut found = Vec::new();
    walk(folder, "/", 0, &mut 0, &mut found);
    found
}

/// Un module venu d'ailleurs (ADR-118) : le temps de son téléchargement, en tout (4 Mo sur une
/// connexion lente), et les redirections suivies, au plus.
pub const MODULE_TIMEOUT: Duration = Duration::from_secs(60);
pub const MODULE_REDIRECTS_MAX: u32 = 5;

/// Télécharge un module venu d'ailleurs, une fois, pour holo serve (ADR-118) : le même chemin sûr
/// que les données d'un autre site. L'adresse est lue strictement (règle 1) ; le nom est résolu
/// et chaque adresse vérifiée, la connexion se fait à l'adresse vérifiée (règle 2, `SafeResolver`) ;
/// ni proxy, ni cookie, ni compression, notre `User-Agent` (règle 7) ; `max` octets au plus,
/// coupés au-delà (règle 4). Aucune clé : un module n'en demande pas, et `holo-data/sites.txt`
/// ne le concerne pas, puisque son empreinte décide.
///
/// Deux différences avec les données, parce que le fichier est épinglé par son empreinte, que
/// holo serve vérifie ensuite (`copies`) : les redirections sont suivies (cinq au plus, chacune
/// en HTTPS et vérifiée comme la première : les fichiers d'une version sur GitHub passent par une
/// redirection) ; et le délai est d'une minute en tout.
pub fn download(address: &str, max: usize, fake: Option<&FakeSite>) -> Result<Vec<u8>, String> {
    let checked = remote_address(address)?;
    let host = checked.host.clone();
    let config = |https_only: bool| {
        ureq::Agent::config_builder()
            .https_only(https_only)
            .proxy(None)
            .max_redirects(MODULE_REDIRECTS_MAX)
            .http_status_as_error(false)
            .timeout_connect(Some(CONNECT_TIMEOUT))
            .timeout_global(Some(MODULE_TIMEOUT))
            .max_idle_connections(0)
            .max_idle_connections_per_host(0)
            .user_agent(USER_AGENT)
            .accept("application/wasm")
            .accept_encoding("")
            .build()
    };
    // L'interrupteur des essais : un nom en `.test`, lu en HTTP clair sur ce PC (`FakeSite`).
    let (agent, url) = match fake {
        Some(site) if site.host == host => (
            ureq::Agent::with_parts(config(false), ureq::unversioned::transport::DefaultConnector::default(), LoopbackResolver(site.port)),
            address.replacen(&format!("https://{}", site.host), &format!("http://{}:{}", site.host, site.port), 1),
        ),
        _ => (ureq::Agent::with_parts(config(true), ureq::unversioned::transport::DefaultConnector::default(), SafeResolver { lookup: system_lookup(), allowed: is_public }), address.to_string()),
    };
    let said = |failure: Failure| match failure {
        Failure::TooBig => format!("{host} envoie plus de {} Mo : coupé, rien n'est pris", max / 1_000_000),
        Failure::Timeout => format!("{host} n'a pas tout envoyé à temps ({} s pour se connecter, {} s en tout)", CONNECT_TIMEOUT.as_secs(), MODULE_TIMEOUT.as_secs()),
        other => Refusal::of(other, &host).to_string(),
    };
    let mut response = agent.get(&url).call().map_err(|error| match error {
        ureq::Error::TooManyRedirects => format!("{host} redirige plus de {MODULE_REDIRECTS_MAX} fois"),
        ureq::Error::RequireHttpsOnly(_) => format!("{host} redirige vers une adresse qui n'est pas en HTTPS"),
        error => said(failure_of(&error)),
    })?;
    let status = response.status().as_u16();
    if status != 200 {
        return Err(format!("{host} a répondu {status} au lieu de 200"));
    }
    // Une taille annoncée trop grande : refusée avant de rien lire.
    if response.body().content_length().is_some_and(|length| length > max as u64) {
        return Err(said(Failure::TooBig));
    }
    read_capped(response.body_mut().as_reader(), max).map_err(said)
}

/// Pour les essais de holo serve : un `Remote` dont le transport est un faux, qui note chaque
/// demande, et dont le journal est gardé.
#[cfg(test)]
pub(crate) fn fake(folder: &Path, answer: impl Fn(&Outgoing) -> Result<Incoming, Failure> + Send + Sync + 'static) -> (Remote, Arc<Mutex<Vec<Outgoing>>>, Arc<Mutex<Vec<String>>>) {
    struct Recorded(Arc<Mutex<Vec<Outgoing>>>, Box<dyn Fn(&Outgoing) -> Result<Incoming, Failure> + Send + Sync>);
    impl Transport for Recorded {
        fn get(&self, request: &Outgoing) -> Result<Incoming, Failure> {
            self.0.lock().unwrap().push(request.clone());
            (self.1)(request)
        }
    }
    let (asked, journal) = (Arc::new(Mutex::new(Vec::new())), Arc::new(Mutex::new(Vec::new())));
    let lines = Arc::clone(&journal);
    let start = Instant::now();
    let remote = Remote::with_parts(
        folder,
        Box::new(Recorded(Arc::clone(&asked), Box::new(answer))),
        None,
        Box::new(move || start.elapsed().as_millis() as u64),
        Box::new(move |line: &str| lines.lock().unwrap().push(line.to_string())),
    );
    (remote, asked, journal)
}

#[cfg(test)]
mod remote_tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const KEY: &str = "s3cr3t-0123456789";
    const HEADER_KEY: &str = "s3cr3t-abcdef-9876";

    /// Un faux transport : il note chaque demande, et répond ce qu'on lui dit.
    type Answer = Arc<dyn Fn(&Outgoing) -> Result<Incoming, Failure> + Send + Sync>;
    struct Fake {
        asked: Arc<Mutex<Vec<Outgoing>>>,
        answer: Answer,
    }

    impl Transport for Fake {
        fn get(&self, request: &Outgoing) -> Result<Incoming, Failure> {
            self.asked.lock().unwrap().push(request.clone());
            (self.answer)(request)
        }
    }

    /// Un site d'essai : un dossier et son holo-data/sites.txt, une horloge qu'on avance, le journal gardé.
    struct Bench {
        remote: Remote,
        asked: Arc<Mutex<Vec<Outgoing>>>,
        clock: Arc<Mutex<u64>>,
        journal: Arc<Mutex<Vec<String>>>,
        folder: PathBuf,
    }

    impl Bench {
        fn asked(&self) -> Vec<Outgoing> {
            self.asked.lock().unwrap().clone()
        }
        fn advance(&self, milliseconds: u64) {
            *self.clock.lock().unwrap() += milliseconds;
        }
        fn journal(&self) -> Vec<String> {
            self.journal.lock().unwrap().clone()
        }
    }

    impl Drop for Bench {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.folder);
        }
    }

    fn folder() -> PathBuf {
        let mut bytes = [0u8; 8];
        getrandom::getrandom(&mut bytes).unwrap();
        let folder = std::env::temp_dir().join(format!("holo-remote-{}", bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()));
        std::fs::create_dir_all(folder.join(crate::server::DATA_FOLDER)).unwrap();
        folder
    }

    fn new_bench(sites: &str, answer: impl Fn(&Outgoing) -> Result<Incoming, Failure> + Send + Sync + 'static) -> Bench {
        let folder = folder();
        std::fs::write(folder.join(crate::server::DATA_FOLDER).join(SITES_FILE), sites).unwrap();
        let (asked, clock, journal) = (Arc::new(Mutex::new(Vec::new())), Arc::new(Mutex::new(1_000_000u64)), Arc::new(Mutex::new(Vec::new())));
        let (time, lines) = (Arc::clone(&clock), Arc::clone(&journal));
        let remote = Remote::with_parts(
            &folder,
            Box::new(Fake { asked: Arc::clone(&asked), answer: Arc::new(answer) }),
            None,
            Box::new(move || *time.lock().unwrap()),
            Box::new(move |line: &str| lines.lock().unwrap().push(line.to_string())),
        );
        Bench { remote, asked, clock, journal, folder }
    }

    fn json(text: &str) -> Result<Incoming, Failure> {
        Ok(Incoming { status: 200, body: text.as_bytes().to_vec() })
    }

    fn header<'a>(request: &'a Outgoing, name: &str) -> Option<&'a str> {
        request.headers.iter().find(|(known, _)| known.eq_ignore_ascii_case(name)).map(|(_, value)| value.as_str())
    }

    #[test]
    fn only_a_declared_site_is_read_compared_exactly() {
        // Règle 1, côté serveur : seul un nom écrit dans holo-data/sites.txt, exactement ; ni un
        // sous-domaine, ni le domaine au-dessus, ni un nom qui le contient.
        let bench = new_bench("api.exemple.org\n", |_| json(r#"{"temperature": 21}"#));
        for written in ["https://exemple.org/m", "https://autre.api.exemple.org/m", "https://api.exemple.org.piege.test/m", "https://xapi.exemple.org/m", "https://api-exemple.org/m"] {
            assert_eq!(bench.remote.read("/p.holo", written, 0), Err(Refusal::NotDeclared(remote_address(written).unwrap().host)), "{written}");
        }
        assert!(bench.asked().is_empty());
        // HTTP, une adresse IP (IPv4, IPv6, en hexadécimal, en un seul nombre), un port, un nom et un
        // mot de passe : refusés avant toute demande (et déjà par le moteur, dans holo check).
        for written in ["http://api.exemple.org/m", "https://93.184.216.34/m", "https://[2606:4700::1111]/m", "https://0x7f.0.0.1/m", "https://2130706433/m", "https://api.exemple.org:8443/m", "https://moi:mdp@api.exemple.org/m", "https://localhost/m"] {
            assert!(matches!(bench.remote.read("/p.holo", written, 0), Err(Refusal::Address(_))), "{written}");
        }
        assert!(bench.asked().is_empty());
        // Le nom exact, en majuscules ou non : lu.
        assert_eq!(bench.remote.read("/p.holo", "https://API.exemple.org/m", 0).unwrap(), r#"{"temperature": 21}"#);
        assert_eq!(bench.asked().len(), 1);
        assert_eq!(bench.asked()[0].address, "https://api.exemple.org/m");
        // La même adresse, écrite autrement : déjà gardée, aucune demande de plus.
        assert!(bench.remote.read("/autre.holo", "https://api.exemple.org/m", 0).is_ok());
        assert_eq!(bench.remote.kept("https://Api.Exemple.org/m", 0), Some(Ok(r#"{"temperature": 21}"#.to_string())));
        assert_eq!(bench.asked().len(), 1);
    }

    #[test]
    fn the_sites_file_is_read_strictly_and_never_repeats_a_key() {
        let text = "\u{feff}# Les autres sites\n\nfr.wikipedia.org\napi.exemple.org ?appid=s3cr3t-0123456789\nweather.exemple.net X-Api-Key: s3cr3t-abcdef-9876\n*.exemple.org\nhttps://site.exemple.org/x\n127.0.0.1\nlocalhost\nFR.wikipedia.org\nvide.exemple.org ?appid=\nmal.exemple.org Cookie: s3cr3t-cookie\nseule.exemple.org s3cr3t-seule\nnom.exemple.org ?s3cr3t_nom avec espace=x\nautre.exemple.org s3cr3t.point: x\n";
        let (permits, problems) = read_sites(text);
        let hosts: Vec<&str> = permits.iter().map(|permit| permit.host.as_str()).collect();
        assert_eq!(hosts, ["fr.wikipedia.org", "api.exemple.org", "weather.exemple.net", "vide.exemple.org", "mal.exemple.org", "seule.exemple.org", "nom.exemple.org", "autre.exemple.org"]);
        assert_eq!(permits[0].key, Key::None);
        assert_eq!(permits[1].key, Key::Parameter("appid".into(), "s3cr3t-0123456789".into()));
        assert_eq!(permits[2].key, Key::Header("X-Api-Key".into(), "s3cr3t-abcdef-9876".into()));
        assert!(permits[3..].iter().all(|permit| permit.key == Key::Missing));
        // Une ligne par erreur, avec son numéro ; jamais rien de ce qui suit le nom d'un site.
        assert_eq!(problems.len(), 10, "{problems:#?}");
        for (problem, line) in problems.iter().zip([6, 7, 8, 9, 10, 11, 12, 13, 14, 15]) {
            assert!(problem.starts_with(&format!("holo-data/sites.txt, ligne {line} : ")), "{problem}");
        }
        assert!(problems.iter().all(|problem| !problem.contains("s3cr3t")), "{problems:#?}");
        assert!(problems[0].contains("pas de « * »") && problems[1].contains("« site.exemple.org »") && problems[2].contains("adresse IP") && problems[3].contains("localhost"));
        assert!(problems[4].contains("déjà écrit plus haut") && problems[5].contains("vide.exemple.org : la clé manque"));
        // Le résumé du démarrage ne montre jamais une clé, même par Debug.
        let sites = Sites { stamp: None, read: true, permits: permits.clone(), problems: problems.clone() };
        let said = summary(&sites, true).join("\n") + &format!("{permits:?}");
        assert!(said.contains("8 permis") && said.contains("api.exemple.org (une clé, en paramètre)") && said.contains("weather.exemple.net (une clé, en en-tête)") && !said.contains("s3cr3t"), "{said}");
        // Au plus 32 sites.
        let many: String = (0..40).map(|n| format!("site{n}.exemple.org\n")).collect();
        let (permits, problems) = read_sites(&many);
        assert_eq!(permits.len(), SITES_MAX);
        assert_eq!(problems, ["holo-data/sites.txt, ligne 33 : plus de 32 sites : « site32.exemple.org » et les suivants sont laissés de côté"]);
    }

    #[test]
    fn this_pc_and_private_networks_are_refused_in_ipv4_and_ipv6() {
        // Règle 2 : bouclage, privé, lien local, multicast, réservé, documentation ; une IPv4 portée
        // par une IPv6 jugée comme l'IPv4 qu'elle porte.
        for refused in [
            "0.0.0.0", "0.1.2.3", "10.0.0.1", "100.64.0.1", "100.127.255.254", "127.0.0.1", "127.255.255.255", "169.254.169.254", "172.16.0.1", "172.31.255.255", "192.0.0.8", "192.0.2.1",
            "192.88.99.1", "192.168.1.1", "198.18.0.1", "198.19.255.255", "198.51.100.7", "203.0.113.9", "224.0.0.1", "239.255.255.250", "240.0.0.1", "255.255.255.255", "::", "::1",
            "::ffff:127.0.0.1", "::ffff:10.0.0.1", "::ffff:169.254.169.254", "::127.0.0.1", "::ffff:0:7f00:1", "64:ff9b::7f00:1", "64:ff9b::a00:1", "64:ff9b:1::1", "fe80::1", "fc00::1",
            "fd12:3456::1", "fec0::1", "ff02::1", "ff05::2", "100::1", "2001::1", "2001:1ff::1", "2001:db8::1", "2002:7f00:1::1", "3fff::1",
        ] {
            assert!(!is_public(refused.parse().unwrap()), "{refused}");
        }
        for allowed in ["1.1.1.1", "8.8.8.8", "93.184.216.34", "100.63.255.255", "100.128.0.1", "172.15.255.255", "172.32.0.1", "192.0.1.1", "198.17.255.255", "223.255.255.255", "2606:4700::1111", "2a00:1450:4007::64", "::ffff:8.8.8.8", "64:ff9b::808:808", "2001:4860:4860::8888"] {
            assert!(is_public(allowed.parse().unwrap()), "{allowed}");
        }
        // Les adresses d'un nom : toutes vérifiées ; une seule privée suffit à tout refuser.
        let public: SocketAddr = "93.184.216.34:443".parse().unwrap();
        assert_eq!(checked(vec![public], is_public), Ok(vec![public]));
        assert_eq!(checked(vec![public, "127.0.0.1:443".parse().unwrap()], is_public), Err(Failure::Private));
        assert_eq!(checked(vec!["[::ffff:192.168.0.1]:443".parse().unwrap()], is_public), Err(Failure::Private));
        assert_eq!(checked(Vec::new(), is_public), Err(Failure::Unresolved));
    }

    /// Un petit serveur HTTP local, pour éprouver le vrai client : il note chaque demande reçue
    /// (ses en-têtes), et répond ce qu'on lui dit ; `None` : il ne répond jamais.
    struct Local {
        port: u16,
        seen: Arc<Mutex<Vec<String>>>,
    }

    fn local(reply: Option<Vec<u8>>) -> Local {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let noted = Arc::clone(&seen);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let (noted, reply) = (Arc::clone(&noted), reply.clone());
                std::thread::spawn(move || {
                    let mut head = Vec::new();
                    let mut byte = [0u8; 1];
                    while !head.ends_with(b"\r\n\r\n") && head.len() < 65_536 && stream.read(&mut byte).is_ok_and(|n| n == 1) {
                        head.push(byte[0]);
                    }
                    noted.lock().unwrap().push(String::from_utf8_lossy(&head).into_owned());
                    match reply {
                        Some(reply) => {
                            let _ = stream.write_all(&reply);
                        }
                        None => std::thread::sleep(Duration::from_secs(5)),
                    }
                });
            }
        });
        Local { port, seen }
    }

    fn ms(milliseconds: u64) -> Duration {
        Duration::from_millis(milliseconds)
    }

    fn outgoing(address: &str, host: &str) -> Outgoing {
        Outgoing { address: address.into(), host: host.into(), headers: vec![("User-Agent".into(), USER_AGENT.into()), ("Accept".into(), "application/json".into())] }
    }

    #[test]
    fn a_name_that_leads_here_is_refused_before_any_connection() {
        // Règle 2, avec le vrai client : le nom mène à 127.0.0.1 ; le résolveur refuse, et le
        // serveur qui écoute à cette adresse ne reçoit rien.
        let here = local(Some(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}".to_vec()));
        let port = here.port;
        let https = Https::new(Arc::new(move |_, _| Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])), None, ms(500), ms(1000));
        assert_eq!(https.get(&outgoing("https://ici.exemple.org/donnees", "ici.exemple.org")), Err(Failure::Private));
        let https = Https::new(Arc::new(move |_, _| Ok(vec![SocketAddr::from(([0xfe80, 0, 0, 0, 0, 0, 0, 1], port))])), None, ms(500), ms(1000));
        assert_eq!(https.get(&outgoing("https://ici.exemple.org/donnees", "ici.exemple.org")), Err(Failure::Private));
        std::thread::sleep(ms(200));
        assert!(here.seen.lock().unwrap().is_empty());
        // Et en HTTP clair, le vrai client refuse : HTTPS seulement.
        assert_eq!(Https::new(system_lookup(), None, ms(500), ms(1000)).get(&outgoing("http://ici.exemple.org/donnees", "ici.exemple.org")), Err(Failure::Unreachable));
    }

    #[test]
    fn the_connection_goes_to_the_checked_address_even_if_the_name_changes_afterwards() {
        // Règle 2 : une seule résolution par demande, vérifiée, et la connexion se fait à l'adresse
        // vérifiée. Ici, la règle d'essai ne permet que 127.0.0.1 ; le nom répond 127.0.0.1, puis
        // 127.0.0.2 (un DNS qui change, comme dans l'attaque par « rebinding »).
        let server = local(Some(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".to_vec()));
        let (port, lookups) = (server.port, Arc::new(AtomicUsize::new(0)));
        let counted = Arc::clone(&lookups);
        let lookup: Lookup = Arc::new(move |_, _| {
            let rank = counted.fetch_add(1, Ordering::SeqCst);
            Ok(vec![SocketAddr::from(([127, 0, 0, if rank == 0 { 1 } else { 2 }], port))])
        });
        let only_this_one: fn(IpAddr) -> bool = |ip| ip == IpAddr::from([127, 0, 0, 1]);
        let agent = agent(false, ms(500), ms(1000), SafeResolver { lookup, allowed: only_this_one });
        let address = format!("http://changeant.exemple.org:{port}/donnees");
        assert_eq!(agent.get(&address).call().unwrap().status().as_u16(), 200);
        assert_eq!((lookups.load(Ordering::SeqCst), server.seen.lock().unwrap().len()), (1, 1));
        // Le nom répond autre chose : vérifié de nouveau, refusé, aucune connexion.
        assert_eq!(failure_of(&agent.get(&address).call().unwrap_err()), Failure::Private);
        assert_eq!((lookups.load(Ordering::SeqCst), server.seen.lock().unwrap().len()), (2, 1));
    }

    #[test]
    fn redirects_are_never_followed() {
        // Règle 3 : 301, 302, 303, 307, 308 : refusés, une seule demande, la cible jamais demandée.
        for status in [301, 302, 303, 307, 308] {
            let bench = new_bench("api.exemple.org\n", move |_| Ok(Incoming { status, body: Vec::new() }));
            assert_eq!(bench.remote.read("/p.holo", "https://api.exemple.org/vieux", 0), Err(Refusal::Redirect("api.exemple.org".into(), status)));
            assert_eq!(bench.asked().len(), 1);
        }
        // Le vrai client : réglé sans redirection, et un serveur qui redirige ne voit qu'une demande.
        let server = local(Some(b"HTTP/1.1 302 Found\r\nLocation: /ailleurs\r\nContent-Length: 0\r\n\r\n".to_vec()));
        let https = Https::new(system_lookup(), Some(FakeSite { host: "faux.test".into(), port: server.port }), ms(500), ms(1000));
        assert_eq!(https.agent.config().max_redirects(), 0);
        assert_eq!(https.get(&outgoing("https://faux.test/vieux", "faux.test")).map(|incoming| incoming.status), Ok(302));
        std::thread::sleep(ms(200));
        assert_eq!(server.seen.lock().unwrap().len(), 1);
    }

    #[test]
    fn an_answer_is_bounded_in_time_size_and_json() {
        // Règle 4. Une réponse sans fin : la lecture s'arrête à 64 Ko et un octet, rien n'est pris.
        assert_eq!(read_capped(std::io::repeat(b'{'), BYTES_MAX), Err(Failure::TooBig));
        assert_eq!(read_capped(&b"{\"a\": 1}"[..], BYTES_MAX), Ok(b"{\"a\": 1}".to_vec()));
        // Le faux transport : trop gros, pas du JSON, un tableau, trop profond, pas de l'UTF-8.
        let big = format!("{{\"note\": \"{}\"}}", "x".repeat(BYTES_MAX));
        for (body, refusal) in [
            (big.into_bytes(), Refusal::TooBig("api.exemple.org".into())),
            (b"<html>panne</html>".to_vec(), Refusal::NotJson("api.exemple.org".into())),
            (b"[1, 2]".to_vec(), Refusal::NotJson("api.exemple.org".into())),
            (b"{\"a\": {\"b\": {\"c\": {\"d\": 1}}}}".to_vec(), Refusal::NotJson("api.exemple.org".into())),
            (vec![b'{', 0xff, b'}'], Refusal::NotJson("api.exemple.org".into())),
        ] {
            let bench = new_bench("api.exemple.org\n", move |_| Ok(Incoming { status: 200, body: body.clone() }));
            assert_eq!(bench.remote.read("/p.holo", "https://api.exemple.org/m", 0), Err(refusal));
        }
        // Le vrai client, par l'interrupteur des essais (les mêmes réglages, sans TLS) : un serveur
        // muet est abandonné au délai ; une réponse trop grosse, annoncée ou non, est coupée.
        let mute = local(None);
        let https = Https::new(system_lookup(), Some(FakeSite { host: "faux.test".into(), port: mute.port }), ms(300), ms(600));
        let started = Instant::now();
        assert_eq!(https.get(&outgoing("https://faux.test/lent", "faux.test")), Err(Failure::Timeout));
        assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
        let mut endless = b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n".to_vec();
        endless.extend(std::iter::repeat_n(b' ', 200_000));
        let announced = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{{}}", 100_000).into_bytes();
        for reply in [endless, announced] {
            let server = local(Some(reply));
            let https = Https::new(system_lookup(), Some(FakeSite { host: "faux.test".into(), port: server.port }), ms(500), ms(2000));
            assert_eq!(https.get(&outgoing("https://faux.test/gros", "faux.test")), Err(Failure::TooBig));
        }
        let config = Https::new(system_lookup(), None, CONNECT_TIMEOUT, TOTAL_TIMEOUT).agent.config().timeouts();
        assert_eq!((config.connect, config.global), (Some(CONNECT_TIMEOUT), Some(TOTAL_TIMEOUT)));
    }

    #[test]
    fn each_address_is_asked_at_most_once_per_interval_whatever_the_visitors() {
        // Règle 5 : cent visiteurs dans la minute, une seule demande ; la suivante quand le temps
        // de la page est passé.
        let bench = new_bench("api.exemple.org\n", |_| json(r#"{"temperature": 21}"#));
        let address = "https://api.exemple.org/meteo";
        for _ in 0..100 {
            assert_eq!(bench.remote.read("/p.holo", address, 600_000).unwrap(), r#"{"temperature": 21}"#);
        }
        assert_eq!(bench.asked().len(), 1);
        bench.advance(599_999);
        bench.remote.read("/p.holo", address, 600_000).unwrap();
        assert_eq!(bench.asked().len(), 1);
        bench.advance(1);
        bench.remote.read("/p.holo", address, 600_000).unwrap();
        assert_eq!(bench.asked().len(), 2);
        // Moins d'une minute compte pour une minute ; sans `every`, dix minutes.
        bench.advance(30_000);
        bench.remote.read("/p.holo", address, 1_000).unwrap();
        assert_eq!(bench.asked().len(), 2);
        let other = "https://api.exemple.org/autre";
        bench.remote.read("/p.holo", other, 0).unwrap();
        bench.advance(EVERY_DEFAULT - 1);
        bench.remote.read("/p.holo", other, 0).unwrap();
        assert_eq!(bench.asked().len(), 3);
        // Un échec est gardé une minute : aucune nouvelle demande avant, même si l'on insiste.
        let failing = new_bench("api.exemple.org\n", |_| Err(Failure::Timeout));
        assert_eq!(failing.remote.read("/p.holo", address, 600_000), Err(Refusal::Timeout("api.exemple.org".into())));
        for _ in 0..50 {
            assert_eq!(failing.remote.read("/p.holo", address, 600_000), Err(Refusal::Recent("api.exemple.org".into())));
        }
        assert_eq!(failing.asked().len(), 1);
        failing.advance(EVERY_MIN);
        assert!(failing.remote.read("/p.holo", address, 600_000).is_err());
        assert_eq!(failing.asked().len(), 2);
        // Huit visiteurs en même temps, un site lent : une seule demande, la même réponse pour tous.
        let slow = new_bench("api.exemple.org\n", |_| {
            std::thread::sleep(ms(300));
            json(r#"{"temperature": 22}"#)
        });
        let answers: Vec<Result<String, Refusal>> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8).map(|_| scope.spawn(|| slow.remote.read("/p.holo", address, 0))).collect();
            handles.into_iter().map(|handle| handle.join().unwrap()).collect()
        });
        assert!(answers.iter().all(|answer| answer.as_deref() == Ok(r#"{"temperature": 22}"#)), "{answers:?}");
        assert_eq!(slow.asked().len(), 1);
    }

    #[test]
    fn the_kept_addresses_are_bounded() {
        // Règle 4 : 8 adresses par site, 64 en tout. Une place se libère d'une adresse demandée il y
        // a une minute au moins : un site reçoit au plus 8 demandes par minute de ce serveur.
        let sites: String = (0..9).map(|n| format!("site{n}.exemple.org\n")).collect();
        let bench = new_bench(&sites, |_| json("{}"));
        for n in 0..8 {
            bench.remote.read("/p.holo", &format!("https://site0.exemple.org/{n}"), 0).unwrap();
        }
        assert_eq!(bench.remote.read("/p.holo", "https://site0.exemple.org/8", 0), Err(Refusal::TooMany("site0.exemple.org".into())));
        bench.advance(EVERY_MIN);
        assert!(bench.remote.read("/p.holo", "https://site0.exemple.org/8", 0).is_ok());
        assert_eq!(bench.asked().len(), 9);
        // 64 en tout : huit sites pleins, le neuvième attend.
        for site in 1..8 {
            for n in 0..8 {
                bench.remote.read("/p.holo", &format!("https://site{site}.exemple.org/{n}"), 0).unwrap();
            }
        }
        bench.advance(1);
        assert_eq!(bench.remote.cache.lock().unwrap().len(), ENTRIES_MAX);
        let refused = bench.remote.read("/p.holo", "https://site8.exemple.org/0", 0);
        assert!(refused.is_ok() || refused == Err(Refusal::TooMany("site8.exemple.org".into())), "{refused:?}");
        assert!(bench.remote.cache.lock().unwrap().len() <= ENTRIES_MAX);
    }

    #[test]
    fn a_key_goes_only_to_its_site_and_never_into_a_page_a_message_or_the_journal() {
        // Règle 6.
        let sites = format!("api.exemple.org ?appid={KEY}\nweather.exemple.net X-Api-Key: {HEADER_KEY}\nfr.wikipedia.org\nvide.exemple.org ?appid=\n");
        let bench = new_bench(&sites, |request| match request.host.as_str() {
            "api.exemple.org" if request.address.contains("echo") => json(&format!("{{\"message\": \"clé {KEY} inconnue\"}}")),
            "api.exemple.org" => json(r#"{"temperature": 21}"#),
            "weather.exemple.net" => Ok(Incoming { status: 401, body: Vec::new() }),
            _ => json(r#"{"title": "Kinshasa"}"#),
        });
        // En paramètre : ajoutée à l'adresse, seulement pour son site.
        assert!(bench.remote.read("/p.holo", "https://api.exemple.org/meteo?q=Kinshasa", 0).is_ok());
        assert_eq!(bench.asked()[0].address, format!("https://api.exemple.org/meteo?q=Kinshasa&appid={KEY}"));
        // En en-tête : seulement pour son site ; un refus 401 dit où écrire la clé, sans elle.
        assert_eq!(bench.remote.read("/p.holo", "https://weather.exemple.net/now", 0), Err(Refusal::Status("weather.exemple.net".into(), 401)));
        assert_eq!(header(&bench.asked()[1], "X-Api-Key"), Some(HEADER_KEY));
        // Un site sans clé n'en reçoit aucune.
        assert!(bench.remote.read("/p.holo", "https://fr.wikipedia.org/api/rest_v1/page/summary/Kinshasa", 0).is_ok());
        let wiki = &bench.asked()[2];
        assert!(!wiki.address.contains("s3cr3t") && wiki.headers.iter().all(|(_, value)| !value.contains("s3cr3t")));
        // Une clé vide : aucune demande, et la page recevra « failed ».
        assert_eq!(bench.remote.read("/p.holo", "https://vide.exemple.org/x", 0), Err(Refusal::MissingKey("vide.exemple.org".into())));
        // La clé écrite dans la page, ou le paramètre qui la porte : refusés avant toute demande.
        assert_eq!(bench.remote.read("/p.holo", &format!("https://api.exemple.org/meteo?key={KEY}"), 0), Err(Refusal::KeyInPage("api.exemple.org".into())));
        assert_eq!(bench.remote.read("/p.holo", "https://api.exemple.org/meteo?appid=autre", 0), Err(Refusal::KeyNameInPage("api.exemple.org".into())));
        // Une réponse qui contient la clé (un message d'erreur qui la recopie) : refusée, jamais gardée.
        assert_eq!(bench.remote.read("/p.holo", "https://api.exemple.org/echo", 0), Err(Refusal::KeyEchoed("api.exemple.org".into())));
        assert_eq!(bench.asked().len(), 4);
        // Ni dans un message, ni au journal, ni dans ce que dit le démarrage.
        let every_refusal = [Refusal::NotDeclared("a".into()), Refusal::MissingKey("a".into()), Refusal::KeyInPage("a".into()), Refusal::KeyNameInPage("a".into()), Refusal::KeyEchoed("a".into()), Refusal::Status("a".into(), 401)];
        assert!(every_refusal.iter().all(|refusal| !refusal.to_string().contains("s3cr3t")));
        let journal = bench.journal().join("\n") + "\n" + &bench.remote.announce(&bench.folder).join("\n");
        assert!(!journal.is_empty() && !journal.contains("s3cr3t"), "{journal}");
        assert!(journal.contains("weather.exemple.net refuse la demande (401) : demande-t-il une clé ?"), "{journal}");
    }

    #[test]
    fn the_request_says_who_asks_and_carries_nothing_of_the_visitor() {
        // Règle 7 : exactement ces en-têtes ; ni cookie, ni Referer, ni adresse du visiteur.
        let bench = new_bench("api.exemple.org\n", |_| json("{}"));
        bench.remote.read("/p.holo", "https://api.exemple.org/meteo", 0).unwrap();
        assert_eq!(bench.asked()[0].headers, [("User-Agent".to_string(), USER_AGENT.to_string()), ("Accept".to_string(), "application/json".to_string())]);
        assert!(USER_AGENT.starts_with("HoloCode/") && USER_AGENT.is_ascii());
        // Le vrai client : aucun proxy, même si l'environnement en donne un (HTTPS_PROXY), et ce qu'un
        // serveur reçoit vraiment : notre User-Agent, Accept, et rien d'autre de ce qui compte.
        let https = Https::new(system_lookup(), None, CONNECT_TIMEOUT, TOTAL_TIMEOUT);
        assert!(https.agent.config().proxy().is_none() && https.agent.config().https_only());
        let server = local(Some(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}".to_vec()));
        let https = Https::new(system_lookup(), Some(FakeSite { host: "faux.test".into(), port: server.port }), ms(500), ms(1000));
        assert_eq!(https.get(&outgoing("https://faux.test/meteo", "faux.test")).map(|incoming| incoming.body), Ok(b"{}".to_vec()));
        std::thread::sleep(ms(100));
        let seen = server.seen.lock().unwrap()[0].to_ascii_lowercase();
        assert!(seen.starts_with("get /meteo http/1.1\r\n") && seen.contains(&format!("user-agent: {}\r\n", USER_AGENT.to_ascii_lowercase())) && seen.contains("accept: application/json\r\n"), "{seen}");
        for absent in ["cookie", "referer", "x-forwarded-for", "forwarded", "origin", "accept-encoding", "authorization"] {
            assert!(!seen.contains(&format!("\r\n{absent}:")), "{absent} : {seen}");
        }
    }

    #[test]
    fn the_test_switch_is_off_by_default_and_takes_only_a_test_name() {
        // Éteint par défaut : sans la variable, aucun faux site.
        assert_eq!(fake_site(None), Ok(None));
        if std::env::var_os(SWITCH).is_none() {
            let folder = folder();
            assert!(Remote::open(&folder).unwrap().fake.is_none());
            let _ = std::fs::remove_dir_all(folder);
        }
        assert_eq!(fake_site(Some("meteo.test:43210")), Ok(Some(FakeSite { host: "meteo.test".into(), port: 43210 })));
        // Seulement un nom en .test (qui n'existe jamais sur Internet), avec un port ; sinon holo serve refuse de démarrer.
        for bad in ["", "meteo.test", "meteo.test:", "meteo.test:0", "meteo.test:70000", "meteo.test:x", "api.exemple.org:443", "127.0.0.1:80", "localhost:8080", "test:80", "METEO.test:80"] {
            assert!(fake_site(Some(bad)).is_err(), "{bad}");
        }
        // Sans l'interrupteur, un nom en .test suit le chemin de tous : il doit être déclaré.
        let bench = new_bench("", |_| json("{}"));
        assert_eq!(bench.remote.read("/p.holo", "https://meteo.test/donnees.json", 0), Err(Refusal::NotDeclared("meteo.test".into())));
        assert!(bench.asked().is_empty());
    }

    #[test]
    fn the_server_is_quiet_at_startup_when_no_page_reads_another_site() {
        let folder = folder();
        std::fs::write(folder.join("local.holo"), "Page(title: \"x\", state: State(t: \"\"), data: Data(from: \"stock.json\"), children: [ P(\"{t}\") ])").unwrap();
        let remote = Remote::with_parts(&folder, Box::new(Fake { asked: Arc::default(), answer: Arc::new(|_| Err(Failure::Unreachable)) }), None, Box::new(|| 0), Box::new(|_: &str| {}));
        assert!(remote.announce(&folder).is_empty());
        // Un dossier à la place du fichier ne bloque rien, et ne permet rien.
        std::fs::create_dir_all(folder.join(crate::server::DATA_FOLDER).join(SITES_FILE)).unwrap();
        assert_eq!(remote.read("/p.holo", "https://api.exemple.org/m", 0), Err(Refusal::NotDeclared("api.exemple.org".into())));
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn the_server_says_at_startup_what_is_permitted_and_what_is_missing() {
        let bench = new_bench(&format!("fr.wikipedia.org\nvide.exemple.org X-Api-Key:\napi.exemple.org ?appid={KEY}\n"), |_| json("{}"));
        let page = |from: &str| format!("Page(title: \"x\", state: State(t: \"\"), data: Data(from: \"{from}\"), children: [ P(\"{{t}}\") ])");
        std::fs::write(bench.folder.join("wiki.holo"), page("https://fr.wikipedia.org/api/rest_v1/page/summary/Kinshasa")).unwrap();
        std::fs::create_dir_all(bench.folder.join("blog")).unwrap();
        std::fs::write(bench.folder.join("blog").join("meteo.holo"), page("https://api.meteo.exemple.org/now")).unwrap();
        std::fs::write(bench.folder.join("vide.holo"), page("https://vide.exemple.org/x")).unwrap();
        std::fs::write(bench.folder.join("local.holo"), page("stock.json")).unwrap();
        let said = bench.remote.announce(&bench.folder);
        assert_eq!(said[0], "Autres sites    : 3 permis (holo-data/sites.txt) : fr.wikipedia.org ; vide.exemple.org (une clé manquante) ; api.exemple.org (une clé, en paramètre)");
        assert!(said[1].contains("ligne 2 : vide.exemple.org : la clé manque"), "{said:#?}");
        assert!(said.iter().any(|line| line.contains("/blog/meteo.holo lit api.meteo.exemple.org, qui n'est pas dans holo-data/sites.txt")), "{said:#?}");
        assert!(said.iter().any(|line| line.contains("/vide.holo lit vide.exemple.org, dont la clé manque")), "{said:#?}");
        assert!(!said.iter().any(|line| line.contains("wiki.holo") || line.contains("local.holo") || line.contains("s3cr3t")), "{said:#?}");
        // Le fichier change pendant que le serveur tourne : relu, et le journal le dit.
        std::thread::sleep(ms(20));
        std::fs::write(bench.folder.join(crate::server::DATA_FOLDER).join(SITES_FILE), "fr.wikipedia.org\napi.meteo.exemple.org\n").unwrap();
        assert!(bench.remote.read("/blog/meteo.holo", "https://api.meteo.exemple.org/now", 0).is_ok());
        assert!(bench.journal().iter().any(|line| line.starts_with("Autres sites    : 2 permis")), "{:#?}", bench.journal());
    }
}
