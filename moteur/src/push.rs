//! Prévenir un visiteur quand sa page est fermée (ADR-119) : `Notification(push: true)`.
//!
//! Le visiteur touche « Me prévenir » : son navigateur lui demande la permission, puis s'abonne
//! auprès de son propre service de notification (Google pour Chrome, Mozilla pour Firefox, Apple
//! pour Safari, Microsoft pour Edge), avec la clé publique de ce site. La page donne cet abonnement
//! (une adresse chez le service, et deux clés du navigateur) à holo serve, qui le garde. Quand
//! holo serve accepte un toucher qui demande `News.send`, il chiffre le message pour chaque abonné
//! et le remet au service, qui le porte au navigateur, même page fermée.
//!
//! Le moteur tient les règles, jamais l'auteur :
//! 1. Les clés du site (VAPID, RFC 8292) sont fabriquées ici, une fois, sans compte ni contrat chez
//!    personne : une paire P-256, rangée dans la base (`holo-data/site.sqlite`), jamais servie ;
//!    seule la clé publique sort. Chaque demande au service est signée (ES256).
//! 2. Chaque message est chiffré de bout en bout pour le navigateur de l'abonné (RFC 8291 : ECDH
//!    P-256, HKDF-SHA-256, AES-128-GCM ; une clé d'un seul usage et un sel neufs à chaque message) :
//!    le service ne fait que le porter. 4096 octets au plus.
//! 3. L'adresse d'un abonnement vient du navigateur du visiteur : c'est une donnée externe. Seul un
//!    service de notification connu est accepté (une liste fermée : Google, Mozilla, Apple,
//!    Microsoft), en HTTPS, lu strictement ; l'envoi passe par le chemin sûr d'`ADR-116` (adresses
//!    publiques seulement, connexion à l'adresse vérifiée, ni proxy ni redirection, 4 s, 8 s, 4 Ko).
//! 4. Les clés d'un abonnement sont vérifiées : un point de la courbe P-256 (65 octets), un secret
//!    de 16 octets.
//! 5. Des bornes : 16 abonnements par visiteur, 1 000 par notification, 10 000 en tout ; un envoi au
//!    plus par notification et par minute (les touchers d'entre-temps n'en font qu'un, à la fin de
//!    la minute) ; un abonnement que le service dit disparu (404, 410) est oublié.
//! 6. Rien sans un geste du visiteur : la permission et l'abonnement ne se demandent que sur un
//!    toucher, jamais au chargement ; se désabonner marche toujours, même si holo serve ne répond pas.
//! 7. Le journal ne dit jamais l'adresse d'un abonné ni une clé : le service, le nombre, la raison.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use ring::rand::{SecureRandom, SystemRandom};
use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};
use ring::{aead, agreement, digest, hkdf};
use rusqlite::{params, Connection, OptionalExtension};

use crate::passkeys::{b64, un64};
use crate::remote::{Failure, FakeSite};
use crate::server::Site;
use crate::state::RemoteAddress;

/// La question qui, à l'adresse d'une page, abonne ou désabonne (`/club.holo?push`).
pub const QUERY: &str = "push";
/// La question qui rend le manifeste d'une page qui prévient (`/club.holo?manifest`) : sur
/// l'iPhone, seule une page installée sur l'écran d'accueil reçoit des notifications.
pub const MANIFEST_QUERY: &str = "manifest";
/// Un message chiffré, au plus : 4096 octets (RFC 8291, section 4 ; RFC 8030 : ce que tout
/// service doit accepter).
pub const MESSAGE_MAX: usize = 4096;
/// Un seul enregistrement (RFC 8188), qui tient dans 4096 octets.
const RECORD_SIZE: u32 = 4096;
/// L'en-tête du contenu chiffré : le sel (16), la taille d'enregistrement (4), la longueur de la
/// clé (1), la clé d'un seul usage du serveur (65).
const HEADER: usize = 16 + 4 + 1 + 65;
/// Les abonnements, au plus : pour un même visiteur (son cookie ou son compte), pour une même
/// notification d'une page, et pour tout le site.
pub const PER_VISITOR_MAX: i64 = 16;
pub const PER_NOTIFICATION_MAX: i64 = 1_000;
pub const FOLLOWS_MAX: i64 = 10_000;
/// Le temps le plus court entre deux envois d'une même notification, en secondes.
pub const ROUND_EVERY: u64 = 60;
/// Le temps qu'un service garde un message pour un appareil éteint : un jour (en-tête `TTL`).
pub const TTL: u64 = 24 * 3600;
/// La signature d'une demande (VAPID) vaut douze heures (24 h au plus, RFC 8292), et se refait au
/// plus une fois par heure (Apple le demande).
const SIGNATURE_LIFETIME: u64 = 12 * 3600;
const SIGNATURE_RENEW: u64 = 3600;
/// Ce qu'une demande d'abonnement peut peser, en JSON.
pub const REQUEST_MAX: usize = 4096;
/// Les notifications qui attendent leur envoi, au plus (une par page et par nom).
const WANTED_MAX: usize = 1_000;
/// Un envoi à tous les abonnés d'une notification ne dure pas plus de deux minutes.
const ROUND_DEADLINE: u64 = 120;

/// Les services de notification connus : une liste fermée, vérifiée dans leur documentation
/// (2026-10-10). Le nom du domaine, si ses sous-domaines servent aussi, et qui le tient.
/// - Google, `fcm.googleapis.com` : Chrome et les navigateurs faits sur Chromium (Opera, Samsung
///   Internet, Brave) ; ce nom exactement, jamais un autre de `googleapis.com`.
/// - Mozilla, `push.services.mozilla.com` (`updates.push.services.mozilla.com/wpush/v2/…`) :
///   Firefox ; Mozilla prévient que l'adresse peut changer.
/// - Apple, `*.push.apple.com` (`web.push.apple.com`) : Safari ; Apple dit d'autoriser
///   `https://*.push.apple.com`.
/// - Microsoft, `*.notify.windows.com` : Edge sous Windows ; Microsoft dit de n'envoyer qu'à ce
///   domaine, dont le sous-domaine change.
pub const SERVICES: &[(&str, bool, &str)] = &[
    ("fcm.googleapis.com", false, "Google"),
    ("push.services.mozilla.com", true, "Mozilla"),
    ("push.apple.com", true, "Apple"),
    ("notify.windows.com", true, "Microsoft"),
];

/// Crée les tables des notifications push s'il le faut : la paire de clés du site (une seule
/// ligne, jamais servie) et les abonnements, chacun à une notification d'une page.
pub fn prepare(base: &Connection) -> Result<(), String> {
    base.execute_batch(
        "CREATE TABLE IF NOT EXISTS push_keys (
             id INTEGER PRIMARY KEY CHECK (id = 1),
             private BLOB NOT NULL,
             public BLOB NOT NULL,
             created INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS push_follows (
             endpoint TEXT NOT NULL,
             page TEXT NOT NULL,
             name TEXT NOT NULL,
             visitor TEXT NOT NULL,
             p256dh BLOB NOT NULL,
             auth BLOB NOT NULL,
             origin TEXT NOT NULL DEFAULT '',
             created INTEGER NOT NULL,
             PRIMARY KEY (endpoint, page, name)
         );",
    )
    .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------- les clés du site (VAPID)

/// La paire de clés du site : fabriquée à la première demande, puis toujours la même. La clé
/// privée (PKCS#8) ne sort jamais de la base ; si l'auteur efface `holo-data/`, une nouvelle
/// paire est fabriquée, et les anciens abonnements meurent d'eux-mêmes (le service les refuse).
fn key_pair(base: &Connection) -> Result<EcdsaKeyPair, String> {
    let rng = SystemRandom::new();
    let read = |base: &Connection| base.query_row("SELECT private FROM push_keys WHERE id = 1", [], |row| row.get::<_, Vec<u8>>(0)).optional().map_err(|e| e.to_string());
    let pkcs8 = match read(base)? {
        Some(kept) => kept,
        None => {
            let made = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).map_err(|_| "le hasard du système manque : aucune clé fabriquée".to_string())?;
            let pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, made.as_ref(), &rng).map_err(|_| "la clé fabriquée est refusée".to_string())?;
            base.execute(
                "INSERT OR IGNORE INTO push_keys (id, private, public, created) VALUES (1, ?1, ?2, ?3)",
                params![made.as_ref(), pair.public_key().as_ref(), crate::server::now() as i64],
            )
            .map_err(|e| e.to_string())?;
            read(base)?.ok_or("la clé du site n'a pas été rangée")?
        }
    };
    EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &pkcs8, &rng).map_err(|_| "la clé du site, dans holo-data/site.sqlite, est abîmée".to_string())
}

/// La clé publique du site, en base64url (65 octets) : celle que le navigateur donne à son service
/// en s'abonnant (`applicationServerKey`). La seule qui sorte jamais.
pub fn public_key(base: &Connection) -> Result<String, String> {
    Ok(b64(key_pair(base)?.public_key().as_ref()))
}

/// La signature d'une demande au service (RFC 8292) : un jeton JWT signé ES256 qui dit à qui il
/// s'adresse (`aud`, l'origine du service), jusqu'à quand (`exp`), et qui joindre (`sub`).
pub fn vapid(pair: &EcdsaKeyPair, audience: &str, subject: Option<&str>, now: u64) -> Result<String, String> {
    let header = b64(br#"{"typ":"JWT","alg":"ES256"}"#);
    let mut claims = format!("{{\"aud\":{},\"exp\":{}", crate::json_text(audience), now + SIGNATURE_LIFETIME);
    if let Some(subject) = subject {
        claims.push_str(&format!(",\"sub\":{}", crate::json_text(subject)));
    }
    claims.push('}');
    let signed = format!("{header}.{}", b64(claims.as_bytes()));
    let signature = pair.sign(&SystemRandom::new(), signed.as_bytes()).map_err(|_| "la signature a échoué".to_string())?;
    Ok(format!("{signed}.{}", b64(signature.as_ref())))
}

/// Le contact donné aux services avec chaque demande (`sub`) : l'adresse publique du site,
/// fixée par l'auteur (`HOLO_ORIGIN`), sinon celle d'où le visiteur s'est abonné, si c'est une
/// adresse HTTPS avec un vrai nom ; sinon aucun. Mozilla et Apple refusent une demande sans
/// contact, ou avec `https://localhost`.
pub fn contact(origin: &str) -> String {
    let Some(host) = origin.strip_prefix("https://") else { return String::new() };
    if crate::state::check_site_name(host).is_err() {
        return String::new();
    }
    origin.to_string()
}

// ---------------------------------------------------------------- le chiffrement (RFC 8291)

/// Une longueur pour HKDF-Expand.
struct Len(usize);

impl hkdf::KeyType for Len {
    fn len(&self) -> usize {
        self.0
    }
}

fn expand(prk: &hkdf::Prk, info: &[&[u8]], out: &mut [u8]) -> Result<(), ring::error::Unspecified> {
    prk.expand(info, Len(out.len()))?.fill(out)
}

/// La clé et le nonce d'AES-128-GCM d'un message (RFC 8291, section 3.4) : le secret partagé
/// (ECDH), mêlé au secret `auth` de l'abonnement et aux deux clés publiques, puis au sel.
fn content_keys(secret: &[u8], auth: &[u8], ua_public: &[u8], as_public: &[u8], salt: &[u8]) -> Result<([u8; 16], [u8; 12]), ring::error::Unspecified> {
    let mut ikm = [0u8; 32];
    expand(&hkdf::Salt::new(hkdf::HKDF_SHA256, auth).extract(secret), &[b"WebPush: info\0", ua_public, as_public], &mut ikm)?;
    let prk = hkdf::Salt::new(hkdf::HKDF_SHA256, salt).extract(&ikm);
    let (mut cek, mut nonce) = ([0u8; 16], [0u8; 12]);
    expand(&prk, &[b"Content-Encoding: aes128gcm\0"], &mut cek)?;
    expand(&prk, &[b"Content-Encoding: nonce\0"], &mut nonce)?;
    Ok((cek, nonce))
}

/// Chiffre un message pour le navigateur d'un abonné (RFC 8291) : une clé d'un seul usage et un
/// sel, tirés au hasard par le système à chaque message. Seul ce navigateur, qui a la clé privée
/// de son abonnement, peut le lire ; le service de notification ne voit que des octets.
pub fn encrypt(plaintext: &[u8], ua_public: &[u8; 65], auth: &[u8; 16]) -> Result<Vec<u8>, String> {
    let rng = SystemRandom::new();
    let ephemeral = agreement::EphemeralPrivateKey::generate(&agreement::ECDH_P256, &rng).map_err(|_| "le hasard du système manque".to_string())?;
    let mut salt = [0u8; 16];
    rng.fill(&mut salt).map_err(|_| "le hasard du système manque".to_string())?;
    seal(plaintext, ua_public, auth, ephemeral, salt)
}

/// Le chiffrement lui-même, avec la clé d'un seul usage et le sel donnés (les essais rejouent
/// l'exemple de la RFC 8291, annexe A) : un seul enregistrement, sans remplissage, puis l'en-tête
/// d'`aes128gcm` (RFC 8188) qui porte le sel et la clé publique d'un seul usage.
fn seal(plaintext: &[u8], ua_public: &[u8; 65], auth: &[u8; 16], ephemeral: agreement::EphemeralPrivateKey, salt: [u8; 16]) -> Result<Vec<u8>, String> {
    if HEADER + plaintext.len() + 1 + aead::AES_128_GCM.tag_len() > MESSAGE_MAX {
        return Err(format!("le message dépasse {MESSAGE_MAX} octets une fois chiffré"));
    }
    let as_public = ephemeral.compute_public_key().map_err(|_| "la clé d'un seul usage est refusée".to_string())?;
    let peer = agreement::UnparsedPublicKey::new(&agreement::ECDH_P256, ua_public);
    let (cek, nonce) = agreement::agree_ephemeral(ephemeral, &peer, |secret| content_keys(secret, auth, ua_public, as_public.as_ref(), &salt))
        .map_err(|_| "la clé du navigateur n'est pas un point de la courbe P-256".to_string())?
        .map_err(|_| "la dérivation des clés a échoué".to_string())?;
    let key = aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_128_GCM, &cek).map_err(|_| "clé AES refusée".to_string())?);
    // Le texte, puis 2 : le dernier enregistrement (RFC 8188), sans remplissage.
    let mut record = Vec::with_capacity(plaintext.len() + 1 + aead::AES_128_GCM.tag_len());
    record.extend_from_slice(plaintext);
    record.push(2);
    key.seal_in_place_append_tag(aead::Nonce::assume_unique_for_key(nonce), aead::Aad::empty(), &mut record).map_err(|_| "le chiffrement a échoué".to_string())?;
    let mut body = Vec::with_capacity(HEADER + record.len());
    body.extend_from_slice(&salt);
    body.extend_from_slice(&RECORD_SIZE.to_be_bytes());
    body.push(65);
    body.extend_from_slice(as_public.as_ref());
    body.extend_from_slice(&record);
    Ok(body)
}

// ---------------------------------------------------------------- les abonnements

/// Un abonnement, tel que le navigateur du visiteur l'a donné, vérifié : l'adresse chez son
/// service (jamais écrite au journal), le nom du service, et les deux clés du navigateur.
#[derive(Clone, PartialEq)]
pub struct Subscription {
    pub endpoint: String,
    pub host: String,
    pub p256dh: [u8; 65],
    pub auth: [u8; 16],
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "un abonnement chez {}", self.host)
    }
}

/// L'adresse d'un abonnement, lue strictement comme celle d'un autre site (HTTPS, un nom et jamais
/// une adresse IP, ni `localhost`, ni port, ni `nom@`), puis comparée à la liste fermée des services
/// de notification : un domaine exact, ou l'un de ses sous-domaines quand le service s'en sert.
/// L'interrupteur des essais (`FakeSite`, un nom en `.test`) y ajoute son faux service.
pub fn service(endpoint: &str, fake: Option<&FakeSite>) -> Result<RemoteAddress, String> {
    let address = crate::state::remote_address(endpoint).map_err(|reason| format!("adresse d'abonnement refusée : {reason}"))?;
    let known = SERVICES.iter().any(|(domain, below, _)| address.host == *domain || (*below && address.host.strip_suffix(domain).is_some_and(|start| start.ends_with('.'))));
    if known || fake.is_some_and(|site| site.host == address.host) {
        Ok(address)
    } else {
        Err(format!("« {} » n'est pas un service de notification connu (Google, Mozilla, Apple, Microsoft) : rien n'y est envoyé", address.host))
    }
}

/// Vérifie un abonnement : son adresse (`service`), sa clé `p256dh` (un point de la courbe P-256,
/// non compressé : 65 octets, le premier vaut 4 ; ring le vérifie en faisant un accord de clés pour
/// rien) et son secret `auth` (16 octets). Le base64url des navigateurs, sans remplissage.
pub fn subscription(endpoint: &str, p256dh: &str, auth: &str, fake: Option<&FakeSite>) -> Result<Subscription, String> {
    let address = service(endpoint, fake)?;
    let key = un64(p256dh.trim_end_matches('='), 65).filter(|k| k.len() == 65 && k[0] == 4).ok_or("la clé « p256dh » de l'abonnement n'est pas une clé P-256 : 65 octets en base64url, le premier vaut 4")?;
    let probe = agreement::EphemeralPrivateKey::generate(&agreement::ECDH_P256, &SystemRandom::new()).map_err(|_| "le hasard du système manque".to_string())?;
    agreement::agree_ephemeral(probe, &agreement::UnparsedPublicKey::new(&agreement::ECDH_P256, &key), |_| ()).map_err(|_| "la clé « p256dh » de l'abonnement n'est pas un point de la courbe P-256".to_string())?;
    let secret = un64(auth.trim_end_matches('='), 16).filter(|a| a.len() == 16).ok_or("le secret « auth » de l'abonnement fait 16 octets, en base64url")?;
    Ok(Subscription { endpoint: endpoint.to_string(), host: address.host, p256dh: key.try_into().map_err(|_| "clé p256dh")?, auth: secret.try_into().map_err(|_| "secret auth")? })
}

/// Ce que la page demande à `?push`, en JSON : s'abonner (`{"follow": "News", "endpoint": …,
/// "p256dh": …, "auth": …}`) ou se désabonner (`{"unfollow": "News", "endpoint": …}`).
#[derive(Debug, PartialEq)]
pub enum Asked {
    Follow { name: String, endpoint: String, p256dh: String, auth: String },
    Unfollow { name: String, endpoint: String },
}

impl Asked {
    pub fn name(&self) -> &str {
        match self {
            Asked::Follow { name, .. } | Asked::Unfollow { name, .. } => name,
        }
    }
}

/// Lit une demande d'abonnement : un objet JSON de 4 Ko au plus, chaque champ une fois, des textes
/// seulement, et rien d'autre que les champs attendus.
pub fn read_request(body: &[u8]) -> Result<Asked, String> {
    if body.len() > REQUEST_MAX {
        return Err(format!("une demande d'abonnement pèse {} Ko au plus", REQUEST_MAX / 1024));
    }
    let text = std::str::from_utf8(body).map_err(|_| "une demande d'abonnement s'écrit en UTF-8".to_string())?;
    let Some(crate::lists::Json::Object(fields)) = crate::lists::Json::read(text) else { return Err("un objet JSON est attendu".into()) };
    let mut seen: Vec<&str> = Vec::new();
    for (key, value) in &fields {
        if seen.contains(&key.as_str()) || !matches!(key.as_str(), "follow" | "unfollow" | "endpoint" | "p256dh" | "auth") || !matches!(value, crate::lists::Json::Text(_)) {
            return Err("champ inconnu, répété, ou qui n'est pas un texte".into());
        }
        seen.push(key);
    }
    let field = |name: &str| fields.iter().find(|(key, _)| key == name).and_then(|(_, value)| match value { crate::lists::Json::Text(t) => Some(t.clone()), _ => None });
    match (field("follow"), field("unfollow"), field("endpoint"), field("p256dh"), field("auth")) {
        (Some(name), None, Some(endpoint), Some(p256dh), Some(auth)) => Ok(Asked::Follow { name, endpoint, p256dh, auth }),
        (None, Some(name), Some(endpoint), None, None) => Ok(Asked::Unfollow { name, endpoint }),
        _ => Err("« follow » avec endpoint, p256dh et auth, ou « unfollow » avec endpoint".into()),
    }
}

/// Range un abonnement à une notification d'une page (`page` : l'adresse de ses valeurs
/// partagées). Un abonnement déjà rangé est mis à jour (le visiteur, ses clés) ; un nouveau doit
/// tenir dans les bornes. Rend le code et la raison d'un refus.
pub fn follow(base: &Connection, page: &str, name: &str, visitor: &str, origin: &str, subscription: &Subscription, now: u64) -> Result<(), (u16, String)> {
    let count = |sql: &str, values: &[&dyn rusqlite::ToSql]| base.query_row(sql, values, |row| row.get::<_, i64>(0)).unwrap_or(i64::MAX);
    let known = count("SELECT COUNT(*) FROM push_follows WHERE endpoint = ?1 AND page = ?2 AND name = ?3", &[&subscription.endpoint, &page, &name]) > 0;
    if !known {
        if count("SELECT COUNT(*) FROM push_follows WHERE visitor = ?1", &[&visitor]) >= PER_VISITOR_MAX {
            return Err((429, format!("{PER_VISITOR_MAX} abonnements au plus pour un même visiteur : retire-en un (« Ne plus me prévenir ») avant d'en prendre un autre")));
        }
        if count("SELECT COUNT(*) FROM push_follows WHERE page = ?1 AND name = ?2", &[&page, &name]) >= PER_NOTIFICATION_MAX {
            return Err((503, format!("cette notification a déjà {PER_NOTIFICATION_MAX} abonnés : holo serve n'en prend pas plus")));
        }
        if count("SELECT COUNT(*) FROM push_follows", &[]) >= FOLLOWS_MAX {
            return Err((503, format!("ce site a déjà {FOLLOWS_MAX} abonnements : holo serve n'en prend pas plus")));
        }
    }
    base.execute(
        "INSERT INTO push_follows (endpoint, page, name, visitor, p256dh, auth, origin, created) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT (endpoint, page, name) DO UPDATE SET visitor = excluded.visitor, p256dh = excluded.p256dh, auth = excluded.auth, origin = excluded.origin",
        params![subscription.endpoint, page, name, visitor, &subscription.p256dh[..], &subscription.auth[..], origin, now as i64],
    )
    .map(drop)
    .map_err(|_| (500, "abonnement non rangé".to_string()))
}

/// Oublie l'abonnement de ce navigateur à une notification d'une page. Rend le nombre
/// d'abonnements qui lui restent sur ce site : à zéro, le navigateur se désabonne aussi chez son
/// service. Toujours permis, sans cookie : l'adresse de l'abonnement suffit.
pub fn unfollow(base: &Connection, page: &str, name: &str, endpoint: &str) -> i64 {
    let _ = base.execute("DELETE FROM push_follows WHERE endpoint = ?1 AND page = ?2 AND name = ?3", params![endpoint, page, name]);
    base.query_row("SELECT COUNT(*) FROM push_follows WHERE endpoint = ?1", params![endpoint], |row| row.get(0)).unwrap_or(0)
}

/// Le manifeste d'une page qui prévient (`?manifest`) : sur l'iPhone et l'iPad (iOS 16.4 et plus),
/// seule une page ajoutée à l'écran d'accueil, qui s'ouvre comme une application, reçoit des
/// notifications ; ce manifeste le lui permet. Le nom de la page, son adresse, rien d'autre.
pub fn manifest(title: &str, start: &str) -> String {
    let title = if title.trim().is_empty() { "HoloCode" } else { title.trim() };
    let short: String = title.chars().take(30).collect();
    format!(
        "{{\"name\":{},\"short_name\":{},\"start_url\":{},\"scope\":\"/\",\"display\":\"standalone\"}}",
        crate::json_text(title),
        crate::json_text(&short),
        crate::json_text(start)
    )
}

// ---------------------------------------------------------------- l'envoi

/// Un message prêt à partir : l'adresse de l'abonnement (jamais écrite), le nom du service, les
/// en-têtes, le contenu chiffré.
#[derive(Clone)]
pub struct Outgoing {
    pub endpoint: String,
    pub host: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl std::fmt::Debug for Outgoing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "POST https://{}/… ({} octets chiffrés)", self.host, self.body.len())
    }
}

/// Le transport : le vrai client HTTPS (le chemin sûr de `remote::post`), ou un faux service de
/// notification dans les essais.
pub trait Transport: Send + Sync {
    fn post(&self, message: &Outgoing) -> Result<u16, Failure>;
}

struct Https {
    fake: Option<FakeSite>,
}

impl Transport for Https {
    fn post(&self, message: &Outgoing) -> Result<u16, Failure> {
        crate::remote::post(&message.endpoint, &message.host, &message.headers, &message.body, self.fake.as_ref())
    }
}

/// Une notification demandée par un toucher accepté, qui attend son envoi : son titre et son
/// texte, et le visiteur qui l'a demandée, s'ils sont tous du même (il voit déjà le changement).
struct Wanted {
    title: String,
    body: String,
    sender: Option<String>,
}

/// Ce qui attend, et le moment du dernier envoi de chaque notification (une page, un nom).
#[derive(Default)]
struct Rounds {
    wanted: HashMap<(String, String), Wanted>,
    last: HashMap<(String, String), u64>,
    told_contact: bool,
}

impl Rounds {
    /// Dans combien de secondes la prochaine notification peut partir ; `None` : rien n'attend.
    fn next_due(&self, now: u64) -> Option<u64> {
        self.wanted.keys().map(|key| self.last.get(key).map_or(0, |last| (last + ROUND_EVERY).saturating_sub(now))).min()
    }
}

/// Les notifications push du site : celles qui attendent, ce qui les porte, l'horloge et le journal.
pub struct Push {
    rounds: Mutex<Rounds>,
    wake: Condvar,
    transport: Box<dyn Transport>,
    /// Les signatures faites, par service et par contact : au plus une nouvelle par heure.
    signatures: Mutex<HashMap<(String, String), (String, u64)>>,
    /// Le temps, en secondes ; une horloge qu'on avance, dans les essais.
    clock: Box<dyn Fn() -> u64 + Send + Sync>,
    /// Le journal de holo serve ; une liste, dans les essais.
    log: Box<dyn Fn(&str) + Send + Sync>,
    /// L'interrupteur des essais, s'il est allumé (jamais par défaut) : son faux service est accepté.
    pub fake: Option<FakeSite>,
}

impl Push {
    /// Les notifications d'un site servi : le vrai client HTTPS, l'heure du système, la console.
    pub fn new(fake: Option<FakeSite>) -> Push {
        Push::with_parts(Box::new(Https { fake: fake.clone() }), fake, Box::new(crate::server::now), Box::new(|line: &str| println!("{line}")))
    }

    pub fn with_parts(transport: Box<dyn Transport>, fake: Option<FakeSite>, clock: Box<dyn Fn() -> u64 + Send + Sync>, log: Box<dyn Fn(&str) + Send + Sync>) -> Push {
        Push { rounds: Mutex::default(), wake: Condvar::new(), transport, signatures: Mutex::default(), clock, log, fake }
    }

    fn rounds(&self) -> std::sync::MutexGuard<'_, Rounds> {
        self.rounds.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Un toucher accepté demande cette notification : elle attend son envoi. Rien ne part d'ici
    /// (aucun réseau sous le verrou de la base) ; plusieurs touchers dans la même minute n'en font
    /// qu'un.
    pub fn ask(&self, page: &str, name: &str, title: &str, body: &str, sender: &str) {
        let mut rounds = self.rounds();
        let key = (page.to_string(), name.to_string());
        if !rounds.wanted.contains_key(&key) && rounds.wanted.len() >= WANTED_MAX {
            (self.log)(&format!("Notification    : {page}, {name} : pas envoyée, {WANTED_MAX} notifications attendent déjà"));
            return;
        }
        let wanted = rounds.wanted.entry(key).or_insert_with(|| Wanted { title: title.to_string(), body: body.to_string(), sender: Some(sender.to_string()) });
        if wanted.sender.as_deref() != Some(sender) {
            wanted.sender = None;
        }
        self.wake.notify_all();
    }

    /// Envoie les notifications dont le tour est venu, et rend dans combien de secondes la suivante
    /// le sera (`None` : rien n'attend). Le réseau ne passe jamais sous un verrou.
    pub fn round(&self, site: &Site) -> Option<u64> {
        let now = (self.clock)();
        let due: Vec<((String, String), Wanted)> = {
            let mut rounds = self.rounds();
            rounds.last.retain(|_, last| now < *last + ROUND_EVERY);
            let ready: Vec<(String, String)> = rounds.wanted.keys().filter(|key| !rounds.last.contains_key(*key)).cloned().collect();
            ready
                .into_iter()
                .filter_map(|key| {
                    rounds.last.insert(key.clone(), now);
                    rounds.wanted.remove(&key).map(|wanted| (key, wanted))
                })
                .collect()
        };
        for ((page, name), wanted) in due {
            self.send(site, &page, &name, &wanted, now);
        }
        self.rounds().next_due((self.clock)())
    }

    /// Attend qu'une notification soit due, ou qu'un toucher en demande une.
    fn sleep(&self) {
        let rounds = self.rounds();
        let wait = match rounds.next_due((self.clock)()) {
            Some(0) => return,
            Some(seconds) => seconds,
            None => 3600,
        };
        let _ = self.wake.wait_timeout(rounds, Duration::from_secs(wait));
    }

    /// La signature pour un service et un contact : la même pendant une heure, puis une nouvelle.
    fn signature(&self, pair: &EcdsaKeyPair, audience: &str, subject: Option<&str>, now: u64) -> Result<String, String> {
        let key = (audience.to_string(), subject.unwrap_or("").to_string());
        let mut signatures = self.signatures.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((token, made)) = signatures.get(&key) {
            if now < made + SIGNATURE_RENEW {
                return Ok(token.clone());
            }
        }
        let token = vapid(pair, audience, subject, now)?;
        if signatures.len() > 64 {
            signatures.clear();
        }
        signatures.insert(key, (token.clone(), now));
        Ok(token)
    }

    /// Envoie une notification à ses abonnés, sauf au visiteur qui l'a demandée, puis oublie ceux
    /// que le service dit disparus. Le journal dit combien, et pourquoi, jamais à qui.
    fn send(&self, site: &Site, page: &str, name: &str, wanted: &Wanted, now: u64) {
        struct Row {
            endpoint: String,
            visitor: String,
            p256dh: Vec<u8>,
            auth: Vec<u8>,
            origin: String,
        }
        let (pair, rows) = {
            let Ok(base) = site.base.lock() else { return };
            let pair = key_pair(&base);
            let rows: Vec<Row> = base
                .prepare("SELECT endpoint, visitor, p256dh, auth, origin FROM push_follows WHERE page = ?1 AND name = ?2 ORDER BY created, rowid LIMIT ?3")
                .and_then(|mut query| query.query_map(params![page, name, PER_NOTIFICATION_MAX], |r| Ok(Row { endpoint: r.get(0)?, visitor: r.get(1)?, p256dh: r.get(2)?, auth: r.get(3)?, origin: r.get(4)? })).map(|rows| rows.flatten().collect::<Vec<Row>>()))
                .unwrap_or_default();
            (pair, rows)
        };
        if rows.is_empty() {
            return;
        }
        let pair = match pair {
            Ok(pair) => pair,
            Err(reason) => return (self.log)(&format!("Notification    : {page}, {name} : pas envoyée, {reason}")),
        };
        let payload = format!(
            "{{\"title\":{},\"body\":{},\"url\":{},\"tag\":{}}}",
            crate::json_text(&wanted.title),
            crate::json_text(&wanted.body),
            crate::json_text(page),
            crate::json_text(&topic(page, name))
        );
        let public = b64(pair.public_key().as_ref());
        let (mut sent, mut gone, mut failed, mut broken) = (0, Vec::new(), Vec::new(), HashSet::new());
        for row in rows {
            if wanted.sender.as_deref() == Some(row.visitor.as_str()) {
                continue;
            }
            // Vérifiée à l'abonnement, l'adresse l'est encore ici : la liste a pu changer depuis.
            let Ok(address) = service(&row.endpoint, self.fake.as_ref()) else {
                gone.push(row.endpoint);
                continue;
            };
            // Un service qui n'a pas répondu ne retient pas l'envoi : ses autres abonnés attendent
            // le prochain, et deux minutes au plus pour tous.
            if broken.contains(&address.host) || (self.clock)() > now + ROUND_DEADLINE {
                failed.push(format!("{} : pas essayé", address.host));
                continue;
            }
            let (Ok(p256dh), Ok(auth)) = (<[u8; 65]>::try_from(row.p256dh.as_slice()), <[u8; 16]>::try_from(row.auth.as_slice())) else {
                gone.push(row.endpoint);
                continue;
            };
            let subject = site.passkeys_origin.clone().or_else(|| (!row.origin.is_empty()).then(|| row.origin.clone()));
            if subject.is_none() && address.host != "fcm.googleapis.com" {
                let mut rounds = self.rounds();
                if !rounds.told_contact {
                    rounds.told_contact = true;
                    (self.log)("Notification    : sans HOLO_ORIGIN (l'adresse publique HTTPS du site), les services de Mozilla et d'Apple refusent un message sans contact");
                }
            }
            let message = encrypt(payload.as_bytes(), &p256dh, &auth).and_then(|body| {
                let token = self.signature(&pair, &format!("https://{}", address.host), subject.as_deref(), now)?;
                Ok(Outgoing {
                    endpoint: row.endpoint.clone(),
                    host: address.host.clone(),
                    headers: vec![
                        ("TTL".into(), TTL.to_string()),
                        ("Urgency".into(), "normal".into()),
                        ("Topic".into(), topic(page, name)),
                        ("Content-Encoding".into(), "aes128gcm".into()),
                        ("Content-Type".into(), "application/octet-stream".into()),
                        ("Authorization".into(), format!("vapid t={token}, k={public}")),
                    ],
                    body,
                })
            });
            let message = match message {
                Ok(message) => message,
                Err(reason) => {
                    failed.push(reason);
                    continue;
                }
            };
            match self.transport.post(&message) {
                Ok(200..=299) => sent += 1,
                // L'abonnement n'existe plus chez le service : oublié.
                Ok(404 | 410) => gone.push(row.endpoint),
                Ok(status) => failed.push(format!("{} a répondu {status}", address.host)),
                Err(failure) => {
                    broken.insert(address.host.clone());
                    failed.push(format!("{} : {}", address.host, said(failure)));
                }
            }
        }
        if !gone.is_empty() {
            if let Ok(base) = site.base.lock() {
                for endpoint in &gone {
                    let _ = base.execute("DELETE FROM push_follows WHERE endpoint = ?1", params![endpoint]);
                }
            }
        }
        let mut line = format!("Notification    : {page}, {name} : {sent} envoyée{}", if sent > 1 { "s" } else { "" });
        if !gone.is_empty() {
            line.push_str(&format!(", {} abonnement{} oublié{} (le service ne le connaît plus)", gone.len(), if gone.len() > 1 { "s" } else { "" }, if gone.len() > 1 { "s" } else { "" }));
        }
        if !failed.is_empty() {
            failed.sort();
            failed.dedup();
            line.push_str(&format!(", en échec : {}", failed.join(" ; ")));
        }
        (self.log)(&line);
    }
}

/// Pourquoi un service n'a pas reçu le message, sans rien de l'adresse.
fn said(failure: Failure) -> &'static str {
    match failure {
        Failure::Private => "mène à ce PC ou au réseau privé, refusé",
        Failure::Unresolved => "introuvable",
        Failure::Timeout => "trop lent",
        Failure::Unreachable => "injoignable",
        Failure::TooBig => "réponse trop longue",
    }
}

/// Le sujet d'une notification pour les services (`Topic`, RFC 8030) : un message qui attend
/// encore un appareil éteint est remplacé par le suivant du même sujet. 32 caractères du base64url,
/// tirés de la page et du nom, qui ne disent ni l'une ni l'autre.
fn topic(page: &str, name: &str) -> String {
    b64(digest::digest(&digest::SHA256, format!("{page}\n{name}").as_bytes()).as_ref())[..32].to_string()
}

/// Le fil qui envoie les notifications de holo serve, à côté des fils qui répondent aux visiteurs.
pub fn start(site: Arc<Site>) {
    std::thread::spawn(move || loop {
        let _ = site.push.round(&site);
        site.push.sleep();
    });
}

#[cfg(test)]
mod push_tests {
    use super::*;
    use crate::server::{Ask, Reply};
    use std::path::{Path, PathBuf};

    /// Des octets écrits en hexadécimal, comme dans l'exemple de la RFC.
    #[allow(non_snake_case)]
    fn H(hex: &str) -> Vec<u8> {
        (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap()).collect()
    }

    /// Une clé « d'un seul usage » tirée d'octets fixes : celle du navigateur d'essai, ou celle de
    /// l'exemple de la RFC 8291. ring n'en importe pas autrement ; c'est son aléa d'essai.
    #[allow(deprecated)]
    fn fixed_key(private: &[u8]) -> agreement::EphemeralPrivateKey {
        agreement::EphemeralPrivateKey::generate(&agreement::ECDH_P256, &ring::test::rand::FixedSliceRandom { bytes: private }).unwrap()
    }

    /// Le faux navigateur : sa clé privée d'abonnement, sa clé publique, son secret.
    struct Browser {
        private: [u8; 32],
        public: [u8; 65],
        auth: [u8; 16],
    }

    impl Browser {
        fn new(seed: u8) -> Browser {
            let private = [seed; 32];
            let public: [u8; 65] = fixed_key(&private).compute_public_key().unwrap().as_ref().try_into().unwrap();
            Browser { private, public, auth: [seed.wrapping_mul(3); 16] }
        }
        /// Déchiffre un message comme le ferait le navigateur (RFC 8291, côté navigateur).
        fn open(&self, message: &[u8]) -> Result<Vec<u8>, &'static str> {
            if message.len() < HEADER + 17 || message[16..20] != RECORD_SIZE.to_be_bytes() || message[20] != 65 {
                return Err("en-tête");
            }
            let (salt, as_public, record) = (&message[..16], &message[21..86], &message[86..]);
            let peer = agreement::UnparsedPublicKey::new(&agreement::ECDH_P256, as_public);
            let (cek, nonce) = agreement::agree_ephemeral(fixed_key(&self.private), &peer, |secret| content_keys(secret, &self.auth, &self.public, as_public, salt)).map_err(|_| "accord")?.map_err(|_| "clés")?;
            let key = aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_128_GCM, &cek).unwrap());
            let mut record = record.to_vec();
            let clear = key.open_in_place(aead::Nonce::assume_unique_for_key(nonce), aead::Aad::empty(), &mut record).map_err(|_| "déchiffrement")?;
            match clear.last() {
                Some(2) => Ok(clear[..clear.len() - 1].to_vec()),
                _ => Err("délimiteur"),
            }
        }
        fn json(&self, endpoint: &str) -> String {
            format!("{{\"follow\":\"News\",\"endpoint\":\"{endpoint}\",\"p256dh\":\"{}\",\"auth\":\"{}\"}}", b64(&self.public), b64(&self.auth))
        }
    }

    #[test]
    fn the_encryption_is_the_example_of_rfc_8291() {
        // RFC 8291, annexe A (reprise telle quelle par la bibliothèque ece de Mozilla) : les clés, le
        // secret, le sel et le texte de l'exemple donnent exactement le message de l'exemple.
        let as_private = H("c9f58f89813e9f8e872e71f42aa64e1757c9254dcc62b72ddc010bb4043ea11c");
        let ua_public: [u8; 65] = H("042571b2becdfde360551aaf1ed0f4cd366c11cebe555f89bcb7b186a53339173168ece2ebe018597bd30479b86e3c8f8eced577ca59187e9246990db682008b0e").try_into().unwrap();
        let auth: [u8; 16] = H("05305932a1c7eabe13b6cec9fda48882").try_into().unwrap();
        let salt: [u8; 16] = H("0c6bfaadad67958803092d454676f397").try_into().unwrap();
        let message = seal(b"When I grow up, I want to be a watermelon", &ua_public, &auth, fixed_key(&as_private), salt).unwrap();
        let expected = H("0c6bfaadad67958803092d454676f397000010004104fe33f4ab0dea71914db55823f73b54948f41306d920732dbb9a59a53286482200e597a7b7bc260ba1c227998580992e93973002f3012a28ae8f06bbb78e5ec0ff297de5b429bba7153d3a4ae0caa091fd425f3b4b5414add8ab37a19c1bbb05cf5cb5b2a2e0562d558635641ec52812c6c8ff42e95ccb86be7cd");
        assert_eq!(message, expected);
        // Le navigateur de l'exemple, avec sa clé privée, le relit.
        let browser = Browser { private: H("ab5757a70dd4a53e553a6bbf71ffefea2874ec07a6b379e3c48f895a02dc33de").try_into().unwrap(), public: ua_public, auth };
        assert_eq!(browser.open(&message).unwrap(), b"When I grow up, I want to be a watermelon");
        // Chaque message tire une clé et un sel neufs : deux messages pareils ne se ressemblent pas.
        let (a, b) = (encrypt(b"pareil", &ua_public, &auth).unwrap(), encrypt(b"pareil", &ua_public, &auth).unwrap());
        assert_ne!(a[..16], b[..16]);
        assert_ne!(a[21..86], b[21..86]);
        assert_eq!(browser.open(&a).unwrap(), b"pareil");
        // Un autre navigateur ne lit rien ; un octet changé en route non plus.
        assert!(Browser::new(7).open(&a).is_err());
        let mut touched = a.clone();
        *touched.last_mut().unwrap() ^= 1;
        assert!(browser.open(&touched).is_err());
        // 4096 octets au plus une fois chiffré : 3993 octets de texte passent, pas un de plus.
        assert_eq!(encrypt(&[b'a'; 3993], &ua_public, &auth).unwrap().len(), MESSAGE_MAX);
        assert!(encrypt(&[b'a'; 3994], &ua_public, &auth).unwrap_err().contains("4096"));
    }

    #[test]
    fn a_request_is_signed_by_the_key_of_the_site() {
        let base = Connection::open_in_memory().unwrap();
        prepare(&base).unwrap();
        let pair = key_pair(&base).unwrap();
        let token = vapid(&pair, "https://fcm.googleapis.com", Some("https://club.example.org"), 1_000_000).unwrap();
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(un64(parts[0], 64).unwrap(), br#"{"typ":"JWT","alg":"ES256"}"#);
        let claims = String::from_utf8(un64(parts[1], 512).unwrap()).unwrap();
        assert_eq!(claims, format!(r#"{{"aud":"https://fcm.googleapis.com","exp":{},"sub":"https://club.example.org"}}"#, 1_000_000 + SIGNATURE_LIFETIME));
        assert!(SIGNATURE_LIFETIME <= 24 * 3600);
        // La signature se vérifie avec la clé publique que le site donne aux navigateurs, et
        // seulement avec elle ; un jeton changé ne se vérifie plus.
        let public = un64(&public_key(&base).unwrap(), 65).unwrap();
        let check = |signed: &str, signature: &[u8]| ring::signature::UnparsedPublicKey::new(&ring::signature::ECDSA_P256_SHA256_FIXED, &public).verify(signed.as_bytes(), signature).is_ok();
        let signature = un64(parts[2], 64).unwrap();
        assert!(check(&format!("{}.{}", parts[0], parts[1]), &signature));
        assert!(!check(&format!("{}.{}x", parts[0], parts[1]), &signature));
        let other = Connection::open_in_memory().unwrap();
        prepare(&other).unwrap();
        assert_ne!(public_key(&other).unwrap(), public_key(&base).unwrap());
        // La même paire à chaque demande ; sans contact, pas de « sub ».
        assert_eq!(public_key(&base).unwrap(), b64(&public));
        let bare = vapid(&pair, "https://web.push.apple.com", None, 5).unwrap();
        assert!(!String::from_utf8(un64(bare.split('.').nth(1).unwrap(), 512).unwrap()).unwrap().contains("sub"));
        // Le contact : une adresse HTTPS avec un vrai nom, jamais localhost ni une adresse IP.
        assert_eq!(contact("https://club.example.org"), "https://club.example.org");
        for refused in ["http://club.example.org", "https://localhost:8080", "https://127.0.0.1", "https://[::1]", "", "https://club.example.org/page"] {
            assert_eq!(contact(refused), "", "{refused}");
        }
    }

    #[test]
    fn only_a_known_notification_service_in_https_is_accepted() {
        for good in [
            "https://fcm.googleapis.com/fcm/send/d61c5u920dw:APA91bEmnw8utjDYCqSRplFMVCzQMg9e5XxpYajvh37mv2QIlISdasBFLbFca9ZZ4Uqcya0ck-SP84YJUEnWsVr3mwYfaDB7vGtsDQuEpfDdcIqOX_wrCRkBW2NDWRZ9qUz9hSgtI3sY",
            "https://updates.push.services.mozilla.com/wpush/v2/gAAAAABk",
            "https://push.services.mozilla.com/wpush/abc123",
            "https://web.push.apple.com/QGuQyavXutnMH",
            "https://wns2-par02p.notify.windows.com/w/?token=BQYAAAB%2bK7pS",
            "https://cloud.notify.windows.com/?token=AQE%2bU%2fSjZOCvRjjpILow%3d%3d",
        ] {
            assert!(service(good, None).is_ok(), "{good}");
        }
        for (bad, why) in [
            ("http://fcm.googleapis.com/fcm/send/x", "HTTPS"),
            ("https://fcm.googleapis.com:8443/fcm/send/x", "port"),
            ("https://user@fcm.googleapis.com/fcm/send/x", "mot de passe"),
            ("https://127.0.0.1/wpush/x", "adresse IP"),
            ("https://[::1]/wpush/x", "adresse IP"),
            ("https://2130706433/x", "adresse IP"),
            ("https://localhost/wpush/x", "localhost"),
            ("https://evil.example/fcm/send/x", "n'est pas un service de notification connu"),
            ("https://fcm.googleapis.com.evil.example/x", "n'est pas un service de notification connu"),
            ("https://storage.googleapis.com/x", "n'est pas un service de notification connu"),
            ("https://evilfcm.googleapis.com/x", "n'est pas un service de notification connu"),
            ("https://notify.windows.com.evil.example/w/", "n'est pas un service de notification connu"),
            ("https://xnotify.windows.com/w/", "n'est pas un service de notification connu"),
            ("https://evilpush.apple.com/x", "n'est pas un service de notification connu"),
            ("https://push.test/wpush/x", "n'est pas un service de notification connu"),
            ("https://fcm.googleapis.com/fcm/send/x#y", "#"),
            ("https://fcm.googleapis.com/fcm send", "espace"),
            ("javascript:alert(1)", "HTTPS"),
            ("", "HTTPS"),
        ] {
            let error = service(bad, None).unwrap_err();
            assert!(error.contains(why), "{bad} : {error}");
        }
        assert!(service(&format!("https://fcm.googleapis.com/{}", "a".repeat(2100)), None).is_err());
        // L'interrupteur des essais ajoute son faux service, en .test, et rien d'autre.
        let fake = FakeSite { host: "push.test".into(), port: 4321 };
        assert!(service("https://push.test/wpush/x", Some(&fake)).is_ok());
        assert!(service("https://evil.example/wpush/x", Some(&fake)).is_err());
    }

    #[test]
    fn the_keys_of_a_subscription_are_checked() {
        let browser = Browser::new(5);
        let (p256dh, auth) = (b64(&browser.public), b64(&browser.auth));
        let endpoint = "https://fcm.googleapis.com/fcm/send/abc";
        let good = subscription(endpoint, &p256dh, &auth, None).unwrap();
        assert_eq!((good.host.as_str(), good.p256dh, good.auth), ("fcm.googleapis.com", browser.public, browser.auth));
        // Le remplissage « = » de certains navigateurs est accepté.
        assert!(subscription(endpoint, &format!("{p256dh}="), &format!("{auth}=="), None).is_ok());
        let mut off_curve = browser.public;
        off_curve[64] ^= 1;
        let mut compressed = vec![2u8];
        compressed.extend_from_slice(&browser.public[1..33]);
        for (key, secret, why) in [
            (b64(&off_curve), auth.clone(), "n'est pas un point de la courbe P-256"),
            (b64(&[4u8; 65]), auth.clone(), "n'est pas un point de la courbe P-256"),
            ({ let mut zero = [0u8; 65]; zero[0] = 4; b64(&zero) }, auth.clone(), "n'est pas un point de la courbe P-256"),
            (b64(&compressed), auth.clone(), "65 octets"),
            (b64(&browser.public[..64]), auth.clone(), "65 octets"),
            ({ let mut five = browser.public; five[0] = 5; b64(&five) }, auth.clone(), "65 octets"),
            ("pas du base64 !".into(), auth.clone(), "65 octets"),
            (p256dh.clone(), b64(&[1u8; 15]), "16 octets"),
            (p256dh.clone(), b64(&[1u8; 17]), "16 octets"),
            (p256dh.clone(), "????".into(), "16 octets"),
        ] {
            let error = subscription(endpoint, &key, &secret, None).unwrap_err();
            assert!(error.contains(why), "{key} {secret} : {error}");
        }
        assert!(subscription("https://evil.example/x", &p256dh, &auth, None).unwrap_err().contains("connu"));
        // Le Debug d'un abonnement ne dit que son service.
        assert_eq!(format!("{good:?}"), "un abonnement chez fcm.googleapis.com");
    }

    #[test]
    fn a_subscription_request_is_read_strictly() {
        assert_eq!(read_request(br#"{"follow":"News","endpoint":"e","p256dh":"p","auth":"a"}"#).unwrap(), Asked::Follow { name: "News".into(), endpoint: "e".into(), p256dh: "p".into(), auth: "a".into() });
        assert_eq!(read_request(br#"{"unfollow":"News","endpoint":"e"}"#).unwrap(), Asked::Unfollow { name: "News".into(), endpoint: "e".into() });
        for bad in [
            &br#"{"follow":"News","endpoint":"e","p256dh":"p"}"#[..],
            br#"{"follow":"News","follow":"News","endpoint":"e","p256dh":"p","auth":"a"}"#,
            br#"{"follow":"News","endpoint":"e","p256dh":"p","auth":"a","admin":"1"}"#,
            br#"{"follow":"News","endpoint":3,"p256dh":"p","auth":"a"}"#,
            br#"{"unfollow":"News","endpoint":"e","auth":"a"}"#,
            br#"{"follow":"A","unfollow":"B","endpoint":"e"}"#,
            br#"["follow"]"#,
            b"\xff\xfe",
        ] {
            assert!(read_request(bad).is_err(), "{}", String::from_utf8_lossy(bad));
        }
        let huge = format!(r#"{{"unfollow":"News","endpoint":"{}"}}"#, "e".repeat(REQUEST_MAX));
        assert!(read_request(huge.as_bytes()).unwrap_err().contains("4 Ko"));
    }

    // ------------------------------------------------------------ holo serve, avec un faux service

    /// Le faux service de notification : il note chaque message, et répond ce qu'on lui dit.
    type Answer = Box<dyn Fn(&Outgoing) -> Result<u16, Failure> + Send + Sync>;
    struct FakeService(Arc<Mutex<Vec<Outgoing>>>, Answer);

    impl Transport for FakeService {
        fn post(&self, message: &Outgoing) -> Result<u16, Failure> {
            self.0.lock().unwrap().push(message.clone());
            (self.1)(message)
        }
    }

    struct Bench {
        site: Site,
        folder: PathBuf,
        posted: Arc<Mutex<Vec<Outgoing>>>,
        clock: Arc<Mutex<u64>>,
        journal: Arc<Mutex<Vec<String>>>,
    }

    impl Bench {
        fn posted(&self) -> Vec<Outgoing> {
            self.posted.lock().unwrap().clone()
        }
        fn advance(&self, seconds: u64) {
            *self.clock.lock().unwrap() += seconds;
        }
        fn journal(&self) -> String {
            self.journal.lock().unwrap().join("\n")
        }
        fn round(&self) -> Option<u64> {
            self.site.push.round(&self.site)
        }
        fn count(&self, sql: &str) -> i64 {
            self.site.base.lock().unwrap().query_row(sql, [], |row| row.get(0)).unwrap()
        }
    }

    impl Drop for Bench {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.folder);
        }
    }

    const CLUB: &str = r#"Page(title: "Le club", state: State(mine: 0), shared: Shared(posts: 0), children: [
      H1("Le club"), P("{posts} messages"), Button(name: Post, text: "Épingler"), Button(name: Mine, text: "À moi"),
      Notification(name: News, label: "Être prévenu", title: "Le club", body: "Un nouveau message.", push: true),
      Button(name: Follow, text: "Me prévenir"), Button(name: Unfollow, text: "Ne plus me prévenir"),
    ], rules: [On(Post.tap, effect: [posts.add(1), News.send]), On(Mine.tap, effect: mine.add(1)), On(Follow.tap, effect: News.request), On(Unfollow.tap, effect: News.stop)])"#;

    fn bench(answer: impl Fn(&Outgoing) -> Result<u16, Failure> + Send + Sync + 'static) -> Bench {
        let mut bytes = [0u8; 8];
        getrandom::getrandom(&mut bytes).unwrap();
        let folder = std::env::temp_dir().join(format!("holo-push-{}", bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()));
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("club.holo"), CLUB).unwrap();
        std::fs::write(folder.join("sans.holo"), r#"Page(title: "Sans", children: [H1("Sans notification")])"#).unwrap();
        let mut site = Site::open(&folder, &Path::new(env!("CARGO_MANIFEST_DIR")).join("web")).unwrap();
        let (posted, clock, journal) = (Arc::new(Mutex::new(Vec::new())), Arc::new(Mutex::new(1_000_000u64)), Arc::new(Mutex::new(Vec::new())));
        let (time, lines) = (Arc::clone(&clock), Arc::clone(&journal));
        site.push = Push::with_parts(
            Box::new(FakeService(Arc::clone(&posted), Box::new(answer))),
            Some(FakeSite { host: "push.test".into(), port: 1 }),
            Box::new(move || *time.lock().unwrap()),
            Box::new(move |line: &str| lines.lock().unwrap().push(line.to_string())),
        );
        Bench { site, folder, posted, clock, journal }
    }

    fn ask<'a>(method: &'a str, url: &'a str, cookie: &'a str, origin: &'a str, body: &'a [u8]) -> Ask<'a> {
        Ask { method, url, accept: "application/json", cookie, content_type: "application/json", origin, host: "localhost:8080", referer: "", peer: "127.0.0.1", forwarded: "", body }
    }

    fn follow(site: &Site, cookie: &str, json: &str) -> Reply {
        site.answer(&ask("POST", "/club.holo?push", cookie, "http://localhost:8080", json.as_bytes()))
    }

    fn cookie_of(reply: &Reply) -> String {
        reply.headers.iter().find(|(name, _)| name == "Set-Cookie").map(|(_, value)| value.split(';').next().unwrap().to_string()).unwrap_or_default()
    }

    /// Un toucher partagé, envoyé par le moteur de la page en JSON (ADR-079).
    fn tap(site: &Site, cookie: &str, signal: &str) -> Reply {
        let body = format!(r#"{{"signal":"{signal}","state":""}}"#);
        site.answer(&ask("POST", "/club.holo", cookie, "http://localhost:8080", body.as_bytes()))
    }

    fn text(reply: &Reply) -> String {
        String::from_utf8_lossy(&reply.body).into_owned()
    }

    /// Carol : une visiteuse qui ne s'abonne pas.
    const CAROL: &str = "holo_visitor=cccccccccccccccccccccccccccccccc";

    /// Un toucher sans JavaScript, par le formulaire des gestes (ADR-074).
    fn form_tap(site: &Site, cookie: &str, signal: &str) -> Reply {
        let body = format!("signal={signal}");
        site.answer(&Ask { content_type: "application/x-www-form-urlencoded", accept: "text/html", ..ask("POST", "/club.holo", cookie, "http://localhost:8080", body.as_bytes()) })
    }

    #[test]
    fn a_notification_goes_encrypted_and_signed_to_each_follower_except_the_sender() {
        let bench = bench(|_| Ok(201));
        // La clé publique du site : 65 octets, la même à chaque demande.
        let key = bench.site.answer(&ask("GET", "/club.holo?push", "", "", b""));
        assert_eq!(key.status, 200, "{}", text(&key));
        let public = text(&key).trim_start_matches("{\"key\":\"").trim_end_matches("\"}").to_string();
        assert_eq!(un64(&public, 65).unwrap().len(), 65);
        assert_eq!(text(&bench.site.answer(&ask("GET", "/club.holo?push", "", "", b""))), text(&key));
        // Deux navigateurs s'abonnent, chez Google et chez le faux service des essais.
        let (ada, bob) = (Browser::new(5), Browser::new(9));
        let first = follow(&bench.site, "", &ada.json("https://fcm.googleapis.com/fcm/send/ada"));
        assert_eq!(first.status, 204, "{}", text(&first));
        let ada_cookie = cookie_of(&first);
        assert!(ada_cookie.starts_with("holo_visitor="), "{ada_cookie}");
        assert_eq!(follow(&bench.site, "", &bob.json("https://push.test/wpush/bob")).status, 204);
        assert_eq!(bench.count("SELECT COUNT(*) FROM push_follows"), 2);
        // Un toucher qui change seulement une valeur à soi ne prévient personne.
        assert_eq!(form_tap(&bench.site, CAROL, "Mine.tap").status, 303);
        assert_eq!(bench.round(), None);
        assert!(bench.posted().is_empty());
        // Ada épingle un message : Bob est prévenu, pas Ada, qui le voit déjà.
        let posted = tap(&bench.site, &ada_cookie, "Post.tap");
        assert_eq!(posted.status, 200, "{}", text(&posted));
        assert!(bench.posted().is_empty(), "rien ne part sous le verrou du geste");
        assert_eq!(bench.round(), None);
        let sent = bench.posted();
        assert_eq!(sent.len(), 1, "{sent:?}");
        let message = &sent[0];
        assert_eq!((message.endpoint.as_str(), message.host.as_str()), ("https://push.test/wpush/bob", "push.test"));
        let header = |name: &str| message.headers.iter().find(|(known, _)| known == name).map(|(_, value)| value.clone()).unwrap_or_default();
        assert_eq!((header("TTL"), header("Urgency"), header("Content-Encoding"), header("Content-Type")), ("86400".into(), "normal".into(), "aes128gcm".into(), "application/octet-stream".into()));
        assert_eq!(header("Topic").len(), 32);
        assert!(header("Topic").bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'));
        // Chiffré de bout en bout : Bob le lit avec sa clé ; Ada, non ; le message ne contient pas le texte en clair.
        let clear = String::from_utf8(bob.open(&message.body).unwrap()).unwrap();
        assert_eq!(clear, format!(r#"{{"title":"Le club","body":"Un nouveau message.","url":"/club.holo","tag":"{}"}}"#, header("Topic")));
        assert!(ada.open(&message.body).is_err());
        assert!(!String::from_utf8_lossy(&message.body).contains("Le club"));
        // Signé par la clé du site : vapid t=…, k=<la clé publique donnée aux navigateurs>.
        let authorization = header("Authorization");
        let (token, k) = authorization.strip_prefix("vapid t=").unwrap().split_once(", k=").unwrap();
        assert_eq!(k, public);
        let parts: Vec<&str> = token.split('.').collect();
        let claims = String::from_utf8(un64(parts[1], 512).unwrap()).unwrap();
        assert!(claims.starts_with(r#"{"aud":"https://push.test","exp":"#), "{claims}");
        let verified = ring::signature::UnparsedPublicKey::new(&ring::signature::ECDSA_P256_SHA256_FIXED, un64(&public, 65).unwrap()).verify(format!("{}.{}", parts[0], parts[1]).as_bytes(), &un64(parts[2], 64).unwrap());
        assert!(verified.is_ok());
        // Carol épingle : Ada et Bob sont prévenus, au tour suivant (une minute après).
        tap(&bench.site, CAROL, "Post.tap");
        assert_eq!(bench.round(), Some(ROUND_EVERY));
        assert_eq!(bench.posted().len(), 1);
        bench.advance(ROUND_EVERY);
        bench.round();
        let hosts: Vec<String> = bench.posted()[1..].iter().map(|m| m.host.clone()).collect();
        assert_eq!(hosts, ["fcm.googleapis.com", "push.test"]);
        // Le journal dit combien, jamais à qui ni une clé.
        let journal = bench.journal();
        assert!(journal.contains("Notification    : /club.holo, News : 1 envoyée") && journal.contains("Notification    : /club.holo, News : 2 envoyées"), "{journal}");
        for secret in ["/fcm/send/ada", "/wpush/bob", &b64(&ada.auth), &b64(&ada.public)] {
            assert!(!journal.contains(secret), "{secret} au journal : {journal}");
        }
    }

    #[test]
    fn the_private_key_never_leaves_the_server() {
        let bench = bench(|_| Ok(201));
        let key = text(&bench.site.answer(&ask("GET", "/club.holo?push", "", "", b"")));
        let private: Vec<u8> = bench.site.base.lock().unwrap().query_row("SELECT private FROM push_keys", [], |row| row.get(0)).unwrap();
        // La clé privée est la seule chose secrète : ses 32 octets, sous toutes leurs formes.
        let pair = key_pair(&bench.site.base.lock().unwrap()).unwrap();
        assert!(private.len() > 32 && !key.contains(&b64(&private)));
        let page = bench.site.answer(&Ask { accept: "text/html", ..ask("GET", "/club.holo", "", "", b"") });
        let source = bench.site.answer(&Ask { accept: "text/plain", ..ask("GET", "/club.holo", "", "", b"") });
        let manifest = bench.site.answer(&ask("GET", "/club.holo?manifest", "", "", b""));
        let seen: Vec<String> = [&page, &source, &manifest].iter().map(|reply| text(reply)).chain([bench.journal(), format!("{pair:?}")]).collect();
        // Le PKCS#8 de ring porte la clé privée en clair, après « 04 20 », à l'octet 36 : elle n'est nulle part.
        assert_eq!(private[34..36], [0x04, 0x20]);
        let scalar = &private[36..68];
        for shown in &seen {
            assert!(!shown.contains(&b64(scalar)) && !shown.contains(&b64(&private)) && !shown.contains(&scalar.iter().map(|b| format!("{b:02x}")).collect::<String>()), "{shown}");
        }
        // La page dit seulement qu'elle prévient : ni clé, ni demande au chargement.
        let html = text(&page);
        assert!(html.contains("&quot;push&quot;:true") && html.contains(r#"<link rel="manifest" href="/club.holo?manifest">"#), "{html}");
        assert!(!html.contains(&key[8..40]));
        // Le dossier holo-data n'est jamais servi.
        assert_eq!(bench.site.answer(&Ask { accept: "*/*", ..ask("GET", "/holo-data/site.sqlite", "", "", b"") }).status, 404);
        // Le manifeste : le nom de la page, son adresse, rien d'autre ; installée, elle s'ouvre seule.
        assert_eq!(text(&manifest), r#"{"name":"Le club","short_name":"Le club","start_url":"/club.holo","scope":"/","display":"standalone"}"#);
        assert!(manifest.headers.iter().any(|(n, v)| n == "Content-Type" && v.starts_with("application/manifest+json")));
        // Une page sans notification push n'a ni clé ni manifeste.
        assert_eq!(bench.site.answer(&ask("GET", "/sans.holo?push", "", "", b"")).status, 404);
        assert_eq!(bench.site.answer(&ask("GET", "/sans.holo?manifest", "", "", b"")).status, 404);
        assert!(!text(&bench.site.answer(&Ask { accept: "text/html", ..ask("GET", "/sans.holo", "", "", b"") })).contains("manifest"));
    }

    #[test]
    fn a_subscription_comes_from_the_page_of_this_site_and_is_checked() {
        let bench = bench(|_| Ok(201));
        let ada = Browser::new(5);
        let json = ada.json("https://fcm.googleapis.com/fcm/send/ada");
        let post = |url: &str, origin: &str, content_type: &str, body: &str| bench.site.answer(&Ask { content_type, ..ask("POST", url, "", origin, body.as_bytes()) });
        for (reply, status, why) in [
            (post("/club.holo?push", "", "application/json", &json), 403, "depuis la page de ce site"),
            (post("/club.holo?push", "https://evil.example", "application/json", &json), 403, "depuis la page de ce site"),
            (post("/club.holo?push", "http://localhost:8080", "text/plain", &json), 415, "JSON"),
            (post("/club.holo?push", "http://localhost:8080", "application/json", &json.replace("\"News\"", "\"Other\"")), 400, "pas de notification push de ce nom"),
            (post("/sans.holo?push", "http://localhost:8080", "application/json", &json), 404, "pas de notification push"),
            (post("/club.holo?push", "http://localhost:8080", "application/json", &ada.json("https://127.0.0.1/wpush/x")), 400, "adresse IP"),
            (post("/club.holo?push", "http://localhost:8080", "application/json", &ada.json("https://169.254.169.254/latest")), 400, "adresse IP"),
            (post("/club.holo?push", "http://localhost:8080", "application/json", &ada.json("https://intranet.example/push")), 400, "connu"),
            (post("/club.holo?push", "http://localhost:8080", "application/json", &ada.json("http://fcm.googleapis.com/fcm/send/x")), 400, "HTTPS"),
            (post("/club.holo?push", "http://localhost:8080", "application/json", &json.replace(&b64(&ada.auth), "AAAA")), 400, "16 octets"),
            (post("/club.holo?push", "http://localhost:8080", "application/json", "{"), 400, "objet JSON"),
        ] {
            assert_eq!(reply.status, status, "{why} : {}", text(&reply));
            assert!(text(&reply).contains(why), "{why} : {}", text(&reply));
        }
        assert_eq!(bench.count("SELECT COUNT(*) FROM push_follows"), 0);
        assert!(bench.posted().is_empty(), "rien n'est envoyé pour vérifier un abonnement");
        // Trente demandes par minute et par adresse IP, puis 429.
        for _ in 0..40 {
            post("/club.holo?push", "http://localhost:8080", "application/json", r#"{"unfollow":"News","endpoint":"https://fcm.googleapis.com/fcm/send/x"}"#);
        }
        let limited = post("/club.holo?push", "http://localhost:8080", "application/json", &json);
        assert_eq!(limited.status, 429, "{}", text(&limited));
        assert!(limited.headers.iter().any(|(n, _)| n == "Retry-After"));
    }

    #[test]
    fn a_page_for_members_is_followed_only_by_members() {
        let bench = bench(|_| Ok(201));
        std::fs::write(bench.folder.join("membres.holo"), CLUB.replace("Page(title: \"Le club\",", "Page(title: \"Le club\", access: members,")).unwrap();
        let ada = Browser::new(5);
        let reply = bench.site.answer(&ask("POST", "/membres.holo?push", "", "http://localhost:8080", ada.json("https://fcm.googleapis.com/fcm/send/ada").as_bytes()));
        assert_eq!(reply.status, 401, "{}", text(&reply));
        assert_eq!(bench.site.answer(&ask("GET", "/membres.holo?push", "", "", b"")).status, 401);
    }

    #[test]
    fn following_is_bounded_per_visitor_per_notification_and_in_all() {
        let bench = bench(|_| Ok(201));
        let ada = Browser::new(5);
        let cookie = cookie_of(&follow(&bench.site, "", &ada.json("https://fcm.googleapis.com/fcm/send/0")));
        for n in 1..PER_VISITOR_MAX {
            assert_eq!(follow(&bench.site, &cookie, &ada.json(&format!("https://fcm.googleapis.com/fcm/send/{n}"))).status, 204);
            // Le frein des adresses IP (30 par minute) : on le remet à zéro entre deux demandes.
            bench.site.base.lock().unwrap().execute("DELETE FROM account_ips", []).unwrap();
        }
        let refused = follow(&bench.site, &cookie, &ada.json("https://fcm.googleapis.com/fcm/send/more"));
        assert_eq!(refused.status, 429, "{}", text(&refused));
        assert!(text(&refused).contains("16 abonnements au plus"));
        // Le même abonnement, redonné, ne compte pas deux fois.
        assert_eq!(follow(&bench.site, &cookie, &ada.json("https://fcm.googleapis.com/fcm/send/3")).status, 204);
        // Pour une notification, et pour tout le site : les places prises restent, les nouvelles sont refusées.
        {
            let base = bench.site.base.lock().unwrap();
            base.execute("DELETE FROM account_ips", []).unwrap();
            base.execute_batch("BEGIN").unwrap();
            for n in 0..PER_NOTIFICATION_MAX {
                base.execute("INSERT INTO push_follows (endpoint, page, name, visitor, p256dh, auth, created) VALUES (?1, '/club.holo', 'News', ?2, x'00', x'00', 0)", params![format!("https://fcm.googleapis.com/fcm/send/x{n}"), format!("v{n}")]).unwrap();
            }
            base.execute_batch("COMMIT").unwrap();
        }
        let full = follow(&bench.site, "", &Browser::new(9).json("https://fcm.googleapis.com/fcm/send/new"));
        assert_eq!(full.status, 503, "{}", text(&full));
        assert!(text(&full).contains("1000 abonnés"), "{}", text(&full));
        {
            let base = bench.site.base.lock().unwrap();
            base.execute("DELETE FROM account_ips", []).unwrap();
            base.execute_batch("BEGIN").unwrap();
            for n in 0..FOLLOWS_MAX {
                base.execute("INSERT INTO push_follows (endpoint, page, name, visitor, p256dh, auth, created) VALUES (?1, '/autre.holo', 'News', ?2, x'00', x'00', 0)", params![format!("https://fcm.googleapis.com/fcm/send/y{n}"), format!("w{n}")]).unwrap();
            }
            base.execute("DELETE FROM push_follows WHERE page = '/club.holo' AND visitor LIKE 'v%'", []).unwrap();
            base.execute_batch("COMMIT").unwrap();
        }
        let all = follow(&bench.site, "", &Browser::new(9).json("https://fcm.googleapis.com/fcm/send/new"));
        assert_eq!(all.status, 503, "{}", text(&all));
        assert!(text(&all).contains("10000 abonnements"), "{}", text(&all));
    }

    #[test]
    fn a_burst_of_taps_makes_one_sending_per_minute() {
        let bench = bench(|_| Ok(201));
        let bob = Browser::new(9);
        follow(&bench.site, "", &bob.json("https://push.test/wpush/bob"));
        tap(&bench.site, CAROL, "Post.tap");
        bench.round();
        assert_eq!(bench.posted().len(), 1);
        // Cinq touchers dans la minute : rien de plus avant la fin de la minute, puis un seul envoi.
        for _ in 0..5 {
            bench.advance(5);
            tap(&bench.site, CAROL, "Post.tap");
            bench.round();
        }
        assert_eq!(bench.posted().len(), 1);
        bench.advance(ROUND_EVERY);
        bench.round();
        assert_eq!(bench.posted().len(), 2);
        bench.advance(ROUND_EVERY * 3);
        assert_eq!(bench.round(), None);
        assert_eq!(bench.posted().len(), 2, "rien n'attend plus : plus rien ne part");
    }

    #[test]
    fn a_subscription_the_service_no_longer_knows_is_forgotten() {
        // Le faux service : 410 pour un abonnement, 404 pour un autre, 429 pour un troisième, 500.
        let bench = bench(|message| Ok(match message.endpoint.rsplit('/').next().unwrap() {
            "gone" => 410,
            "invalid" => 404,
            "busy" => 429,
            "broken" => 500,
            _ => 201,
        }));
        for (seed, name) in [(3, "gone"), (4, "invalid"), (5, "busy"), (6, "broken"), (7, "fine")] {
            assert_eq!(follow(&bench.site, "", &Browser::new(seed).json(&format!("https://push.test/wpush/{name}"))).status, 204);
            bench.site.base.lock().unwrap().execute("DELETE FROM account_ips", []).unwrap();
        }
        tap(&bench.site, CAROL, "Post.tap");
        bench.round();
        assert_eq!(bench.posted().len(), 5);
        let left: Vec<String> = bench.site.base.lock().unwrap().prepare("SELECT endpoint FROM push_follows ORDER BY endpoint").unwrap().query_map([], |r| r.get(0)).unwrap().flatten().collect();
        assert_eq!(left, ["https://push.test/wpush/broken", "https://push.test/wpush/busy", "https://push.test/wpush/fine"]);
        let journal = bench.journal();
        assert!(journal.contains("1 envoyée, 2 abonnements oubliés (le service ne le connaît plus), en échec : push.test a répondu 429 ; push.test a répondu 500"), "{journal}");
    }

    #[test]
    fn a_silent_service_does_not_hold_back_the_others() {
        let bench = bench(|message| if message.host == "push.test" { Err(Failure::Timeout) } else { Ok(201) });
        for (seed, endpoint) in [(3, "https://push.test/wpush/a"), (4, "https://push.test/wpush/b"), (5, "https://push.test/wpush/c"), (6, "https://web.push.apple.com/d")] {
            follow(&bench.site, "", &Browser::new(seed).json(endpoint));
            bench.site.base.lock().unwrap().execute("DELETE FROM account_ips", []).unwrap();
        }
        tap(&bench.site, CAROL, "Post.tap");
        bench.round();
        // Le faux service ne répond pas : essayé une fois, pas trois ; Apple reçoit le sien.
        let hosts: Vec<String> = bench.posted().iter().map(|m| m.host.clone()).collect();
        assert_eq!(hosts.iter().filter(|h| *h == "push.test").count(), 1, "{hosts:?}");
        assert!(hosts.contains(&"web.push.apple.com".to_string()));
        assert_eq!(bench.count("SELECT COUNT(*) FROM push_follows"), 4, "un service lent n'efface personne");
        let journal = bench.journal();
        assert!(journal.contains("push.test : trop lent") && journal.contains("push.test : pas essayé"), "{journal}");
        // Sans HOLO_ORIGIN ni adresse HTTPS d'abonnement : le journal dit qu'Apple et Mozilla veulent un contact.
        assert!(journal.contains("sans HOLO_ORIGIN"), "{journal}");
    }

    #[test]
    fn unsubscribing_always_works() {
        let bench = bench(|_| Ok(201));
        let ada = Browser::new(5);
        let cookie = cookie_of(&follow(&bench.site, "", &ada.json("https://fcm.googleapis.com/fcm/send/ada")));
        std::fs::write(bench.folder.join("autre.holo"), CLUB).unwrap();
        bench.site.answer(&ask("POST", "/autre.holo?push", &cookie, "http://localhost:8080", ada.json("https://fcm.googleapis.com/fcm/send/ada").as_bytes()));
        assert_eq!(bench.count("SELECT COUNT(*) FROM push_follows"), 2);
        // Sans cookie (effacé depuis), l'adresse de l'abonnement suffit ; il en reste un sur ce site.
        let left = follow(&bench.site, "", r#"{"unfollow":"News","endpoint":"https://fcm.googleapis.com/fcm/send/ada"}"#);
        assert_eq!((left.status, text(&left).as_str()), (200, r#"{"left":1}"#));
        let last = bench.site.answer(&ask("POST", "/autre.holo?push", "", "http://localhost:8080", br#"{"unfollow":"News","endpoint":"https://fcm.googleapis.com/fcm/send/ada"}"#));
        assert_eq!(text(&last), r#"{"left":0}"#);
        // Un abonnement inconnu : rien à oublier, zéro restant (le navigateur se désabonne lui-même).
        assert_eq!(text(&follow(&bench.site, "", r#"{"unfollow":"News","endpoint":"https://fcm.googleapis.com/fcm/send/nobody"}"#)), r#"{"left":0}"#);
        // Désabonné, on n'est plus prévenu.
        tap(&bench.site, CAROL, "Post.tap");
        bench.round();
        assert!(bench.posted().is_empty());
    }

    #[test]
    fn without_javascript_a_shared_tap_also_notifies() {
        let bench = bench(|_| Ok(201));
        follow(&bench.site, "", &Browser::new(9).json("https://push.test/wpush/bob"));
        // Le formulaire des gestes, sans JavaScript : holo serve arbitre, puis prévient. « News.send »
        // n'est pas l'envoi d'un formulaire : aucun message n'est rangé.
        let reply = form_tap(&bench.site, CAROL, "Post.tap");
        assert_eq!(reply.status, 303, "{}", text(&reply));
        bench.round();
        assert_eq!(bench.posted().len(), 1);
        assert_eq!(bench.count("SELECT COUNT(*) FROM messages"), 0);
        // Le toucher de « Me prévenir », sans JavaScript, ne range ni message ni abonnement.
        form_tap(&bench.site, CAROL, "Follow.tap");
        assert_eq!(bench.count("SELECT COUNT(*) FROM messages"), 0);
        assert_eq!(bench.count("SELECT COUNT(*) FROM push_follows"), 1);
    }
}
