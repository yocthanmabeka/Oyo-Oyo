//! Le premier vrai serveur de HoloCode : `holo serve` (ADR-074). Un programme en Rust, sur le
//! PC de l'auteur, sans rien d'autre à installer : ni Node.js, ni base de données à part, ni
//! service extérieur (chez soi d'abord).
//!
//! Il sert un dossier : ses pages `.holo` fabriquées d'avance, ses images et ses fichiers, et le
//! moteur pour le navigateur. Il fait tourner le même arbitre que le navigateur : un visiteur
//! sans JavaScript touche un bouton, le formulaire des gestes part, le serveur calcule le nouvel
//! état, le range dans SQLite sous le numéro du visiteur, et renvoie la page à jour.
//!
//! Il reçoit aussi les formulaires `Form` (ADR-075), envoyés par le moteur du navigateur ou sans
//! JavaScript, et les range dans la même base ; `holo messages` les montre à l'auteur.
//!
//! Les valeurs de chaque visiteur sont à lui seul, sauf celles que la page partage
//! (`Shared`, ADR-079) : le serveur les garde pour tout le monde, une fois par adresse, arbitre
//! chaque geste qui les change, chacun son tour, et les envoie en direct à toutes les pages
//! ouvertes à cette adresse (`text/event-stream`). Un visiteur qui a un compte (ADR-081,
//! `accounts.rs`) et s'est connecté retrouve les siennes sur tous ses appareils : elles sont
//! gardées sous son compte.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};

/// Le dossier, dans le site servi, où le serveur range sa base. Il n'est jamais servi.
pub const DATA_FOLDER: &str = "holo-data";
/// Ce qu'un formulaire des gestes peut peser : bien plus que ses champs n'en demandent.
const BODY_MAX: u64 = 65_536;
/// L'état d'un visiteur, au plus : au-delà, il n'est pas gardé.
const STATE_MAX: usize = 262_144;
/// Un visiteur qui ne revient pas pendant trente jours est oublié.
const FORGET_AFTER: u64 = 30 * 24 * 3600;
/// Le nom du cookie qui porte le numéro du visiteur.
const COOKIE: &str = "holo_visitor";
/// Un message d'un formulaire, en JSON : 16 Ko au plus (ADR-042).
const MESSAGE_MAX: usize = 16_384;
/// Un envoi avec des fichiers : quatre fichiers de 10 Mo, et le message (ADR-059).
const WITH_FILES_MAX: u64 = 4 * 10_000_000 + 65_536;
/// Ce qu'une page garde de messages, et de fichiers, au plus.
const MESSAGES_PER_PAGE_MAX: i64 = 5_000_000;
const FILES_PER_PAGE_MAX: u64 = 500_000_000;
/// Les pages ouvertes en direct (ADR-079), au plus, pour tout le site : chacune tient une
/// connexion et un fil du serveur.
const LIVE_MAX: usize = 128;
/// Un battement toutes les dix secondes vers chaque page en direct : une page fermée est oubliée
/// au plus tard au second battement qui ne passe pas.
const HEARTBEAT: u64 = 10;
/// Les changements qu'une page en direct peut avoir en retard ; au-delà, elle est coupée (elle se
/// reconnecte d'elle-même et reçoit les valeurs du moment).
const LIVE_QUEUE: usize = 64;

/// Ce que le serveur garde d'un visiteur sur une page : ses valeurs, et les formulaires qu'il a
/// essayé d'envoyer sans y arriver (leurs messages d'erreur suivent ce qu'il corrige).
#[derive(Default)]
struct Visit {
    state: String,
    tried: Vec<String>,
}

/// Ce que le serveur sait de son site.
pub struct Site {
    /// Origine publique fixée par l'auteur ; HTTPS arrive par son proxy local.
    pub(crate) passkeys_origin: Option<String>,
    /// Le dossier servi : les pages, les images, les fichiers.
    pub(crate) folder: PathBuf,
    /// Le dossier du moteur pour le navigateur (`moteur/web`) : la page d'entrée, le moteur.
    web: PathBuf,
    /// La base : un seul fichier, `holo-data/site.sqlite`.
    pub(crate) base: Mutex<Connection>,
    /// Un geste de page à la fois, de la lecture de la visite à son enregistrement : les envois
    /// JSON, les formulaires et les miroirs d'un même compte ne relisent pas un état périmé.
    /// On prend ce verrou avant la base ; les lecteurs et les comptes restent indépendants.
    pub(crate) gestures: Mutex<()>,
    /// Les pages ouvertes en direct (ADR-079) : chacune reçoit les valeurs partagées de son
    /// adresse quand elles changent. On prend toujours la base avant cette liste, jamais l'inverse.
    lives: Arc<Mutex<Lives>>,
}

/// Les pages ouvertes en direct, et le numéro de la prochaine.
#[derive(Default)]
struct Lives {
    pages: Vec<Live>,
    next: u64,
}

/// Une page ouverte en direct : son adresse, et ce qui porte les changements jusqu'à son fil.
struct Live {
    id: u64,
    key: String,
    sender: std::sync::mpsc::SyncSender<String>,
}

/// Une demande, réduite à ce que le serveur lit.
pub struct Ask<'a> {
    pub method: &'a str,
    pub url: &'a str,
    pub accept: &'a str,
    pub cookie: &'a str,
    pub content_type: &'a str,
    pub origin: &'a str,
    pub host: &'a str,
    /// La page d'où vient le visiteur : un lien « Se connecter » y ramène (ADR-081).
    pub referer: &'a str,
    /// Adresse du pair TCP, jamais un en-tête fourni par le visiteur.
    pub peer: &'a str,
    /// Les en-têtes `X-Forwarded-For`, mis bout à bout : lus seulement derrière le proxy HTTPS de
    /// l'auteur (`client_address`), jamais pour un visiteur qui parle directement au serveur.
    pub forwarded: &'a str,
    pub body: &'a [u8],
}

/// Une réponse : son code, ses en-têtes, son contenu.
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Reply {
    fn text(status: u16, text: &str) -> Reply {
        Reply { status, headers: vec![("Content-Type".into(), "text/plain; charset=utf-8".into())], body: text.as_bytes().to_vec() }
    }
}

impl Site {
    /// Ouvre le site d'un dossier : crée `holo-data/` et sa base s'il le faut, et oublie les
    /// visiteurs partis depuis trente jours.
    pub fn open(folder: &Path, web: &Path) -> Result<Site, String> {
        let folder = folder.canonicalize().map_err(|e| format!("{} : {e}", folder.display()))?;
        let data = folder.join(DATA_FOLDER);
        std::fs::create_dir_all(&data).map_err(|e| format!("{} : {e}", data.display()))?;
        let base = Connection::open(data.join("site.sqlite")).map_err(|e| e.to_string())?;
        base.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS visits (
                 visitor TEXT NOT NULL,
                 page TEXT NOT NULL,
                 state TEXT NOT NULL,
                 updated INTEGER NOT NULL,
                 PRIMARY KEY (visitor, page)
             );
             CREATE TABLE IF NOT EXISTS messages (
                 id INTEGER PRIMARY KEY,
                 received INTEGER NOT NULL,
                 page TEXT NOT NULL,
                 form TEXT NOT NULL,
                 submission TEXT NOT NULL,
                 files TEXT NOT NULL DEFAULT '[]'
             );
             CREATE TABLE IF NOT EXISTS shared_limits (key TEXT PRIMARY KEY,started INTEGER NOT NULL,touches INTEGER NOT NULL);
             CREATE TABLE IF NOT EXISTS shared (
                 page TEXT NOT NULL,
                 name TEXT NOT NULL,
                 value TEXT NOT NULL,
                 version INTEGER NOT NULL,
                 updated INTEGER NOT NULL,
                 PRIMARY KEY (page, name)
             );",
        )
        .map_err(|e| e.to_string())?;
        // Une base faite avant les formulaires (ADR-075) reçoit leur colonne.
        let _ = base.execute("ALTER TABLE visits ADD COLUMN tried TEXT NOT NULL DEFAULT ''", []);
        // Le modèle de la page qui reçoit un message (ADR-078) : le quota se compte par modèle, pour
        // qu'on ne remplisse pas la base en inventant des adresses ; chaque message garde la sienne.
        let _ = base.execute("ALTER TABLE messages ADD COLUMN model TEXT NOT NULL DEFAULT ''", []);
        // Un visiteur absent est oublié ; ce que garde un compte (`account:7`), non : il reste à lui.
        base.execute("DELETE FROM visits WHERE updated < ?1 AND visitor NOT LIKE 'account:%'", params![now().saturating_sub(FORGET_AFTER) as i64]).map_err(|e| e.to_string())?;
        // Les comptes, les sessions, le frein contre les essais répétés (ADR-081).
        crate::accounts::prepare(&base, now())?;
        let _=base.execute("ALTER TABLE messages ADD COLUMN account INTEGER",[]);
        // Les fichiers privés d'un compte effacé qui attendent encore (un fichier pris par un autre
        // programme, sous Windows) : un nouvel essai. Un échec n'empêche pas le site de démarrer ;
        // le fichier reste en attente, et le serveur le redit au prochain démarrage.
        if let Err(error) = crate::accounts::retry_erased_files(&folder, &base) {
            eprintln!("Effacement en attente : {error}");
        }
        Ok(Site { passkeys_origin: crate::passkeys::configured_origin()?, folder, web: web.to_path_buf(), base: Mutex::new(base), gestures: Mutex::new(()), lives: Arc::default() })
    }

    /// Répond à une demande. Tout passe par ici : c'est ce que les essais éprouvent.
    pub fn answer(&self, ask: &Ask) -> Reply {
        let Some(path) = url_path(ask.url) else { return Reply::text(400, "adresse illisible") };
        // Les pages de compte (ADR-081) : créer un compte, se connecter, le code, se déconnecter.
        if let Some(reply) = crate::accounts::answer(self, ask, &path) {
            return reply;
        }
        let path = if path == "/" { "/index.holo".to_string() } else { path };
        // L'adresse telle qu'elle est dans l'URL, encodée : les valeurs d'un modèle en viennent (ADR-078).
        let raw = ask.url.split(['?', '#']).next().unwrap_or("/");
        match ask.method {
            "GET" | "HEAD" => self.get(ask, &path, raw),
            "POST" => self.gesture(ask, &path, raw),
            _ => Reply::text(405, "seuls GET et POST sont reçus"),
        }
    }

    fn get(&self, ask: &Ask, path: &str, raw: &str) -> Reply {
        let Some((file, holo, values)) = self.locate(path, raw) else { return Reply::text(404, "introuvable") };
        // Une copie hors-ligne est toujours une page publique initiale, sans valeurs de visiteur.
        if holo.ends_with(".holo") && ask.accept.contains("text/html") {
            if let Ok(source) = source_at(&file, &values) {
                if crate::check_page(&source).is_ok_and(|p| crate::capabilities::is_offline(&p)) {
                    let base = &holo[..=holo.rfind('/').unwrap_or(0)];
                    let rendered = crate::flat_view(&source, base).map_err(|e| e.message);
                    return match rendered.and_then(|html| std::fs::read_to_string(self.web.join("page.html")).map(|template| filled_template(&template, &html)).map_err(|e| e.to_string())) {
                        Ok(html) => Reply { status: 200, headers: vec![("Content-Type".into(), "text/html; charset=utf-8".into()), ("Cache-Control".into(), "no-cache".into())], body: html.into_bytes() },
                        Err(_) => Reply::text(500, "copie publique impossible"),
                    };
                }
            }
        }
        // Un .holo demandé pour être affiché : sa page, fabriquée pour ce visiteur. Demandé par le
        // moteur (`text/plain`), le fichier lui-même. Une adresse sans `.holo` (`/contact`, un
        // modèle `profil/{id}.holo`) est toujours une page (ADR-078).
        let shown = holo.ends_with(".holo") && (ask.accept.contains("text/html") || holo != path);
        // Le membre connecté (ADR-081). Une page réservée aux membres n'est ni fabriquée, ni donnée
        // au moteur, pour qui ne l'est pas : il est mené à « Se connecter », puis ramené ici.
        let member = crate::accounts::member_of(self, ask.cookie);
        if holo.ends_with(".holo") && member.is_none() && members_only(&file, &values) {
            if !shown {
                return Reply::text(401, "page réservée aux membres : connecte-toi d'abord");
            }
            let mut headers = vec![("Location".to_string(), crate::accounts::sign_in_address(ask.url.split('#').next().unwrap_or(raw)))];
            headers.extend(member_headers());
            return Reply { status: 303, headers, body: Vec::new() };
        }
        // Une page faite pour un membre, ou son texte, n'est jamais gardée en cache ; le moteur et les
        // images, si : ils sont les mêmes pour tous.
        let headers = if member.is_some() && holo.ends_with(".holo") { member_headers() } else { common_headers() };
        if shown {
            let visit = visit_key(member.as_ref(), ask.cookie).and_then(|key| self.stored(&key, path));
            return match self.page(&file, path, &holo, &values, visit, member.as_ref(), query_of(ask.url)) {
                Ok(html) => {
                    let mut reply = Reply { status: 200, headers: vec![("Content-Type".into(), "text/html; charset=utf-8".into())], body: html.into_bytes() };
                    reply.headers.extend(headers);
                    reply
                }
                Err(message) => Reply::text(500, &message),
            };
        }
        match std::fs::read(&file) {
            Ok(bytes) => {
                let mut all = vec![("Content-Type".to_string(), content_type(&file).to_string())];
                all.extend(headers);
                Reply { status: 200, headers: all, body: bytes }
            }
            Err(_) => Reply::text(404, "introuvable"),
        }
    }

    /// Un toucher envoyé sans JavaScript (ADR-074) : le même arbitre, puis la page à jour par
    /// une nouvelle demande (`303`), pour qu'un rechargement ne rejoue pas le geste.
    fn gesture(&self, ask: &Ask, path: &str, raw: &str) -> Reply {
        let Ok(_gesture)=self.gestures.lock() else{return Reply::text(500,"arbitre indisponible")};
        let Some((file, holo, values)) = self.locate(path, raw) else { return Reply::text(404, "page introuvable") };
        if !holo.ends_with(".holo") {
            return Reply::text(405, "seule une page .holo reçoit des gestes");
        }
        // Une page d'un autre site ne fait pas toucher les boutons de celle-ci.
        if !ask.origin.is_empty() && ask.origin.split("://").nth(1) != Some(ask.host) {
            return Reply::text(403, "ce geste vient d'un autre site");
        }
        // Une page réservée aux membres ne reçoit ni geste ni message de qui ne l'est pas (ADR-081).
        let member = crate::accounts::member_of(self, ask.cookie);
        if member.is_none() && members_only(&file, &values) {
            return Reply::text(401, "page réservée aux membres : connecte-toi d'abord");
        }
        // Un geste partagé envoyé par le moteur de la page, en JSON (ADR-079) : il porte un signal.
        if ask.content_type.starts_with("application/json") && ask.body.len() as u64 <= BODY_MAX {
            if let Some((signal, state)) = crate::read_gesture(&String::from_utf8_lossy(ask.body)) {
                return self.shared_gesture(ask, path, &file, &holo, &values, &signal, &state, member.as_ref());
            }
        }
        // Un formulaire envoyé par le moteur, en JSON, avec ou sans fichiers (ADR-075).
        if ask.content_type.starts_with("application/json") || ask.content_type.starts_with("multipart/form-data") {
            return self.message(ask, path, &holo, &file, &values, member.as_ref());
        }
        if !ask.content_type.starts_with("application/x-www-form-urlencoded") {
            return Reply::text(415, "un geste, ou un formulaire en JSON");
        }
        if ask.body.len() as u64 > BODY_MAX {
            return Reply::text(413, "formulaire trop lourd");
        }
        let Ok(source) = source_for(&file, &values, member.as_ref()) else { return Reply::text(404, "page introuvable") };
        set_clock();
        // L'état d'un membre est gardé sous son compte ; celui d'un visiteur, sous son cookie.
        let (visitor, new_visitor) = match visit_key(member.as_ref(), ask.cookie) {
            Some(key) => (key, false),
            None => (new_visitor(), true),
        };
        // Un toucher que le moteur du navigateur a déjà joué (`?mirror`, ADR-081) : le serveur le
        // rejoue sur l'état qu'il garde, pour que le compte le retrouve ailleurs. Il n'envoie rien
        // (le moteur s'en charge), et répond sans renvoyer la page.
        let mirror = ask.url.split_once('?').is_some_and(|(_, query)| query.split('&').any(|part| part == "mirror"));
        let mut visit = self.stored(&visitor, path).unwrap_or_else(|| Visit { state: starting_state(&source, &file), tried: Vec::new() });
        // Le geste part de la page que montre le navigateur, avec les valeurs de son adresse
        // (ADR-091), même si le visiteur est revenu en arrière depuis.
        visit.state = with_query(&source, &visit.state, query_of(ask.url));
        let fields = crate::gestures::read_form(&String::from_utf8_lossy(ask.body));
        let tap = fields.iter().find(|(name, signal)| name == crate::gestures::SIGNAL && crate::gestures::is_tap(signal)).map(|(_, signal)| signal.as_str()).unwrap_or("");
        // Une page qui partage des valeurs (ADR-079) : les champs, puis le toucher, arbitré avec les
        // valeurs que le serveur garde, chacun son tour ; un bouton caché ne se touche pas.
        let signal = if crate::shared_names(&source).is_empty() {
            visit.state = crate::visitor_gesture(&source, &visit.state, &fields);
            tap
        } else {
            let inputs: Vec<(String, String)> = fields.iter().filter(|(name, _)| name != crate::gestures::SIGNAL).cloned().collect();
            let Ok(base) = self.base.lock() else { return Reply::text(500, "base indisponible") };
            let key = shared_key(&holo, &values);
            if !tap.is_empty()&&!shared_allowed(&base,&visitor,&client_address(self, ask),&key,now()){return shared_limited(new_visitor.then_some(visitor.as_str()));}
            let (current, version) = shared_in(&base, &key);
            visit.state = crate::visitor_gesture(&source, &crate::with_shared(&source, &visit.state, &current), &inputs);
            let (after, accepted) = if tap.is_empty() { (visit.state.clone(), false) } else { crate::share(&source, &visit.state, &current, tap) };
            if accepted {
                visit.state = without_sounds(&after);
                self.changed(&base, &key, &source, &current, &visit.state, version);
            }
            if accepted { tap } else { "" }
        };
        // Un toucher qui envoie un formulaire (`On(Send.tap, effect: Contact.send)`) : le serveur
        // vérifie, range le message, puis `Contact.sent` ; ou garde les messages d'erreur.
        // Un toucher renvoyé par le moteur (`?mirror`, ADR-081) : le moteur a déjà fait l'envoi.
        for form in crate::effects(&source, signal).iter().filter(|_| !mirror).filter_map(|effect| effect.strip_suffix(".send")) {
            visit.tried.retain(|tried| tried != form);
            if !crate::form_errors(&source, &visit.state, form).is_empty() {
                visit.tried.push(form.to_string());
                continue;
            }
            let submission = crate::submission(&source, &visit.state, form);
            let outcome = if self.keep_message(path, &holo, form, &submission, "[]", member.as_ref().map(|m|m.id)).is_ok() { "sent" } else { "failed" };
            visit.state = without_sounds(&crate::arbitrate(&source, &visit.state, &format!("{form}.{outcome}")));
        }
        if visit.state.len() <= STATE_MAX {
            self.store(&visitor, path, &visit);
        }
        // La page à jour, à l'adresse de ses nouvelles valeurs (ADR-091) : sans JavaScript aussi,
        // « Précédent » revient à l'onglet d'avant.
        let query = crate::address_query(&source, &visit.state);
        let location = if query.is_empty() { path.to_string() } else { format!("{path}?{query}") };
        let mut headers = if mirror { Vec::new() } else { vec![("Location".to_string(), location)] };
        if new_visitor {
            headers.push(("Set-Cookie".into(), format!("{COOKIE}={visitor}; Path=/; HttpOnly; SameSite=Lax; Max-Age={FORGET_AFTER}")));
        }
        headers.extend(common_headers());
        Reply { status: if mirror { 204 } else { 303 }, headers, body: Vec::new() }
    }

    /// Le fichier d'une page, et ce qui la décrit (ADR-078) : le fichier même de l'adresse ; sinon
    /// la même adresse avec `.holo` (`/contact` → `contact.holo`) ; sinon un modèle
    /// (`profil/{id}.holo` pour `/profil/123`). Rend le fichier, son adresse `.holo` (celle que le
    /// moteur du navigateur lit) et les valeurs de l'adresse.
    fn locate(&self, path: &str, raw: &str) -> Option<(PathBuf, String, Vec<(String, String)>)> {
        if let Some(file) = self.find(path) {
            // Un modèle ouvert lui-même : un texte vide par nom, comme `holo check`.
            return Some((file, path.to_string(), crate::address::empty_values(path)));
        }
        if !path.ends_with(".holo") {
            let plain = format!("{path}.holo");
            if let Some(file) = self.find(&plain) {
                return Some((file, plain, Vec::new()));
            }
        }
        self.template(raw)
    }

    /// Le modèle d'une adresse : morceau par morceau, le dossier du même nom, sinon un dossier
    /// `{x}` ; pour le dernier, un fichier `{x}.holo`. Jamais la base, ni un dossier caché.
    fn template(&self, raw: &str) -> Option<(PathBuf, String, Vec<(String, String)>)> {
        let parts: Vec<&str> = raw.trim_start_matches('/').trim_end_matches('/').split('/').collect();
        let mut folder = self.folder.clone();
        let mut pattern = String::new();
        for (i, part) in parts.iter().enumerate() {
            let last = i + 1 == parts.len();
            let decoded = crate::address::decode(part)?;
            if decoded.is_empty() || decoded.starts_with('.') || decoded.contains(['\\', ':']) || (i == 0 && decoded == DATA_FOLDER) {
                return None;
            }
            let exact = folder.join(&decoded);
            let name = if !last && exact.is_dir() { decoded } else { holder(&folder, last)? };
            folder = folder.join(&name);
            pattern.push('/');
            pattern.push_str(&name);
        }
        let values = crate::address::values(&pattern, raw)?;
        Some((folder, pattern, values))
    }

    /// Le fichier d'une adresse : dans le site d'abord, puis dans le moteur. Jamais la base, un
    /// fichier caché, ni rien hors de ces deux dossiers.
    fn find(&self, path: &str) -> Option<PathBuf> {
        let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
        if parts.iter().any(|part| part.starts_with('.') || part.contains('\\') || part.contains(':')) || parts.first() == Some(&DATA_FOLDER) {
            return None;
        }
        [&self.folder, &self.web].into_iter().map(|root| parts.iter().fold(root.to_path_buf(), |path, part| path.join(part))).find(|file| file.is_file())
    }

    /// La page d'entrée, avec la page du fichier déjà fabriquée dedans, comme le fait
    /// outils/server.mjs. Un fichier de points (`Point`) ouvre la porte des mondes.
    fn page(&self, file: &Path, path: &str, holo: &str, values: &[(String, String)], visit: Option<Visit>, member: Option<&crate::accounts::Member>, query: &str) -> Result<String, String> {
        let source = source_for(file, values, member).map_err(|e| e.to_string())?;
        if source.lines().map(|line| line.split("//").next().unwrap_or("").trim()).find(|line| !line.is_empty()).is_some_and(|line| line.starts_with("Point")) {
            return std::fs::read_to_string(self.web.join("index.html")).map_err(|e| e.to_string());
        }
        let template = std::fs::read_to_string(self.web.join("page.html")).map_err(|e| format!("page d'entrée du moteur introuvable : {e}"))?;
        set_clock();
        let mut visit = visit.unwrap_or_else(|| Visit { state: starting_state(&source, file), tried: Vec::new() });
        // Les valeurs que l'adresse porte après le `?` (ADR-091) : l'adresse les dit, même
        // revenue en arrière ; celles qu'elle ne dit pas reprennent leur départ.
        visit.state = with_query(&source, &visit.state, query);
        let _ = path;
        let base = &holo[..=holo.rfind('/').unwrap_or(0)];
        // Le fichier de la page (ADR-078) : le moteur du navigateur y lit son texte, même quand
        // l'adresse ne le dit pas (`/contact`, un modèle `profil/{id}.holo`).
        // Le membre connecté (ADR-081) : le moteur du navigateur lit son nom, et renvoie ses touchers
        // au serveur, qui les garde sous son compte.
        let signed = member.map(|member| format!("<meta name=\"holo-account\" content=\"{}\">", crate::flat::escape(&member.name))).unwrap_or_default();
        let template = template.replacen("<title>HoloCode</title>", &format!("<title>HoloCode</title><meta name=\"holo-file\" content=\"{}\">{signed}", crate::flat::escape(holo)), 1);
        // Les valeurs partagées du moment, gardées pour cette adresse (ADR-079).
        let mut visit = visit;
        if !crate::shared_names(&source).is_empty() {
            let (current, _) = self.base.lock().map(|base| shared_in(&base, &shared_key(holo, values))).unwrap_or_default();
            visit.state = crate::with_shared(&source, &visit.state, &current);
        }
        // Un fichier refusé : la page d'entrée seule, qui affichera l'erreur du moteur.
        let Ok(html) = crate::visitor_page(&source, base, &visit.state, &visit.tried) else { return Ok(template) };
        Ok(filled_template(&template, &html))
    }

    fn stored(&self, visitor: &str, page: &str) -> Option<Visit> {
        stored_in(&*self.base.lock().ok()?, visitor, page)
    }

    fn store(&self, visitor: &str, page: &str, visit: &Visit) {
        if let Ok(base) = self.base.lock() {
            store_in(&base, visitor, page, visit);
        }
    }

    /// Un geste partagé envoyé par le moteur de la page (ADR-079) : `{"signal": "Book.tap",
    /// "state": "…"}`. Le serveur prend la base, arbitre avec les valeurs qu'il garde (l'état du
    /// visiteur, qu'il a pu forger, n'en change aucune), range les nouvelles, les envoie en direct
    /// à toutes les pages ouvertes à cette adresse, garde l'état du visiteur comme sans JavaScript,
    /// et répond l'état d'après : `200`, ou `409` si le geste est refusé (un bouton caché).
    ///
    /// Un membre connecté (ADR-081) : son état vient de la base, jamais de la copie envoyée par
    /// la page. Seules les valeurs liées à un champ passent par la validation des saisies.
    /// Un geste refusé ne garde rien. Pour un visiteur sans compte, l'état personnel reste fourni
    /// par sa page (limite d'ADR-079) ; ses valeurs partagées viennent toujours de la base.
    #[allow(clippy::too_many_arguments)]
    fn shared_gesture(&self, ask: &Ask, path: &str, file: &Path, holo: &str, values: &[(String, String)], signal: &str, state: &str, member: Option<&crate::accounts::Member>) -> Reply {
        let Ok(source) = source_for(file, values, member) else { return Reply::text(404, "page introuvable") };
        if crate::shared_names(&source).is_empty() {
            return Reply::text(400, "cette page ne partage aucune valeur");
        }
        if !crate::touches_shared(&source, signal) {
            return Reply::text(400, "ce geste ne change aucune valeur partagée : la page le fait seule");
        }
        set_clock();
        let (visitor, new_visitor) = match visit_key(member, ask.cookie) {
            Some(key) => (key, false),
            None => (new_visitor(), true),
        };
        let Ok(base) = self.base.lock() else { return Reply::text(500, "base indisponible") };
        let key = shared_key(holo, values);
        if !shared_allowed(&base,&visitor,&client_address(self, ask),&key,now()){return shared_limited(new_visitor.then_some(visitor.as_str()));}
        let (current, mut version) = shared_in(&base, &key);
        let mut visit = stored_in(&base, &visitor, path).unwrap_or_else(|| Visit { state: starting_state(&source, file), tried: Vec::new() });
        let before = crate::with_shared(&source, &visit.state, &current);
        let candidate = if member.is_some() {
            // La copie de la page n'est qu'un transport pour ses champs : elle ne peut ni remettre
            // booked à zéro, ni changer une liste, un score, le compte ou le compteur de hasard.
            crate::visitor_gesture(&source, &before, &inputs_from_state(&source, state))
        } else {
            state.to_string()
        };
        let (mut after, accepted) = crate::share(&source, &candidate, &current, signal);
        if accepted {
            let saved = without_sounds(&after);
            if saved.len() > STATE_MAX {
                return Reply::text(413, "état trop lourd");
            }
            version = self.changed(&base, &key, &source, &current, &after, version);
            visit.state = saved;
            store_in(&base, &visitor, path, &visit);
        } else if member.is_some() {
            // Le refus ne valide pas non plus les saisies jointes ; la réponse rend l'état gardé.
            after = before;
        }
        drop(base);
        let body = format!("{{\"accepted\":{accepted},\"state\":{},\"shared\":{},\"version\":{version}}}", crate::json_text(&after), crate::json_text(&crate::shared_of(&source, &after)));
        let mut headers = vec![("Content-Type".to_string(), "application/json; charset=utf-8".to_string())];
        if new_visitor {
            headers.push(("Set-Cookie".into(), format!("{COOKIE}={visitor}; Path=/; HttpOnly; SameSite=Lax; Max-Age={FORGET_AFTER}")));
        }
        headers.extend(common_headers());
        Reply { status: if accepted { 200 } else { 409 }, headers, body: body.into_bytes() }
    }

    /// Les valeurs partagées après un geste accepté : si elles ont changé, elles sont rangées sous
    /// un nouveau numéro et envoyées en direct, pendant que la base est encore prise (ADR-079) :
    /// deux gestes ne se croisent jamais, et chaque page reçoit les changements dans l'ordre. Rend
    /// le numéro des valeurs du moment.
    fn changed(&self, base: &Connection, key: &str, source: &str, current: &str, after: &str, version: i64) -> i64 {
        let before = crate::shared_of(source, &crate::with_shared(source, "", current));
        let now_shared = crate::shared_of(source, after);
        if now_shared == before {
            return version;
        }
        let version = version + 1;
        let stamp = now() as i64;
        for chunk in now_shared.split(';').filter(|chunk| !chunk.is_empty()) {
            let (name, value) = chunk.split_once('=').unwrap_or((chunk, ""));
            let _ = base.execute(
                "INSERT INTO shared (page, name, value, version, updated) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (page, name) DO UPDATE SET value = excluded.value, version = excluded.version, updated = excluded.updated",
                params![key, name, value, version, stamp],
            );
        }
        self.broadcast(key, &now_shared, version);
        version
    }

    /// Envoie les valeurs partagées d'une adresse à toutes les pages qui l'écoutent. Une page qui
    /// ne suit plus (fermée, ou trop en retard) est retirée.
    fn broadcast(&self, key: &str, written: &str, version: i64) {
        let event = live_event(written, version);
        if let Ok(mut lives) = self.lives.lock() {
            let before = lives.pages.len();
            lives.pages.retain(|live| live.key != key || live.sender.try_send(event.clone()).is_ok());
            if lives.pages.len() != before {
                println!("Direct          : {key}, {} page(s) à l'écoute", lives.pages.iter().filter(|live| live.key == key).count());
            }
        }
    }

    /// Une adresse que le navigateur veut écouter en direct (`Accept: text/event-stream`,
    /// ADR-079) : une page qui partage des valeurs. Rend l'adresse sous laquelle elles sont gardées
    /// et le texte de la page, ou le refus.
    pub fn live_page(&self, ask: &Ask) -> Result<(String, String), Reply> {
        let Some(path) = url_path(ask.url) else { return Err(Reply::text(400, "adresse illisible")) };
        let path = if path == "/" { "/index.holo".to_string() } else { path };
        let raw = ask.url.split(['?', '#']).next().unwrap_or("/");
        if !ask.origin.is_empty() && ask.origin.split("://").nth(1) != Some(ask.host) {
            return Err(Reply::text(403, "cette page est écoutée depuis un autre site"));
        }
        let Some((file, holo, values)) = self.locate(&path, raw) else { return Err(Reply::text(404, "introuvable")) };
        // Les valeurs partagées d'une page réservée ne s'écoutent qu'en membre (ADR-081).
        if crate::accounts::member_of(self, ask.cookie).is_none() && members_only(&file, &values) {
            return Err(Reply::text(401, "page réservée aux membres : connecte-toi d'abord"));
        }
        let source = source_at(&file, &values).map_err(|_| Reply::text(404, "introuvable"))?;
        if !holo.ends_with(".holo") || crate::shared_names(&source).is_empty() {
            return Err(Reply::text(404, "cette page ne partage aucune valeur à écouter"));
        }
        Ok((shared_key(&holo, &values), source))
    }

    /// Une page écoute son adresse en direct : elle reçoit d'abord les valeurs du moment, puis
    /// chaque changement, et un battement de temps en temps. Son propre fil écrit pour elle : les
    /// quatre fils du serveur restent libres, et une page lente ne retient personne.
    pub fn listen(&self, key: &str, source: &str, mut writer: Box<dyn Write + Send>) {
        let Ok(base) = self.base.lock() else { return };
        let Ok(mut lives) = self.lives.lock() else { return };
        // Une page fermée s'est déjà retirée elle-même : son fil le fait en s'arrêtant.
        if lives.pages.len() >= LIVE_MAX {
            let refusal = "trop de pages ouvertes en direct";
            let _ = write!(writer, "HTTP/1.1 503 Service Unavailable\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{refusal}", refusal.len());
            let _ = writer.flush();
            return;
        }
        // Les valeurs du moment : rien ne peut changer entre elles et l'inscription (la base est prise).
        let (current, version) = shared_in(&base, key);
        let now_shared = crate::shared_of(source, &crate::with_shared(source, "", &current));
        let (sender, receiver) = std::sync::mpsc::sync_channel::<String>(LIVE_QUEUE);
        let _ = sender.try_send(live_event(&now_shared, version));
        let id = lives.next;
        lives.next += 1;
        lives.pages.push(Live { id, key: key.to_string(), sender });
        println!("Direct          : {key}, {} page(s) à l'écoute", lives.pages.iter().filter(|live| live.key == key).count());
        drop(lives);
        drop(base);
        let (lives, key) = (Arc::clone(&self.lives), key.to_string());
        std::thread::spawn(move || {
            let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream; charset=utf-8\r\nCache-Control: no-cache\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\nretry: 3000\n\n";
            let mut alive = writer.write_all(head.as_bytes()).and_then(|()| writer.flush()).is_ok();
            while alive {
                let chunk = match receiver.recv_timeout(std::time::Duration::from_secs(HEARTBEAT)) {
                    Ok(event) => event,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => ":\n\n".to_string(),
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                };
                alive = writer.write_all(chunk.as_bytes()).and_then(|()| writer.flush()).is_ok();
            }
            // La page est fermée, ou ne suit plus : elle est oubliée.
            if let Ok(mut lives) = lives.lock() {
                let before = lives.pages.len();
                lives.pages.retain(|live| live.id != id);
                if lives.pages.len() != before {
                    println!("Direct          : {key}, {} page(s) à l'écoute", lives.pages.iter().filter(|live| live.key == key).count());
                }
            }
        });
    }

    /// Un formulaire envoyé par le moteur du navigateur (ADR-042, ADR-059), reçu ici (ADR-075) :
    /// vérifié à nouveau par le moteur (la page peut être contournée, le serveur non), ses
    /// fichiers reconnus à leurs premiers octets, puis rangé dans la base. `204`, ou le refus.
    fn message(&self, ask: &Ask, path: &str, holo: &str, file: &Path, values: &[(String, String)], member: Option<&crate::accounts::Member>) -> Reply {
        let multipart = ask.content_type.starts_with("multipart/form-data");
        if ask.body.len() as u64 > if multipart { WITH_FILES_MAX } else { MESSAGE_MAX as u64 } {
            return Reply::text(413, "message trop long");
        }
        let (json, sent_files) = if multipart {
            let Some(parts) = read_multipart(ask.body, ask.content_type) else { return Reply::text(400, "message mal formé") };
            let Some(values) = parts.iter().find(|part| part.name == "submission").filter(|part| part.bytes.len() <= MESSAGE_MAX) else { return Reply::text(400, "message mal formé") };
            let json = String::from_utf8_lossy(&values.bytes).into_owned();
            (json, parts.into_iter().filter(|part| part.name != "submission" && !part.bytes.is_empty()).collect::<Vec<_>>())
        } else {
            (String::from_utf8_lossy(ask.body).into_owned(), Vec::new())
        };
        let Ok(source) = source_for(file, values, member) else { return Reply::text(404, "page introuvable") };
        match crate::check_submission(&source, &json) {
            Ok(errors) if errors.is_empty() => {}
            Ok(errors) => return Reply::text(422, &errors),
            Err(_) => return Reply::text(400, "page refusée par le moteur"),
        }
        let Some(crate::lists::Json::Object(top)) = crate::lists::Json::read(&json) else { return Reply::text(400, "message illisible") };
        let Some(crate::lists::Json::Text(form)) = top.iter().find(|(key, _)| key == "form").map(|(_, value)| value) else { return Reply::text(400, "message mal formé") };
        // Les fichiers : ce que la page permet, demandé au moteur, jamais à ce que dit le navigateur.
        let allowed = crate::files_for_server(&source).unwrap_or_default();
        let folder = self.folder.join(DATA_FOLDER).join("files").join(page_name(path));
        let mut place = FILES_PER_PAGE_MAX.saturating_sub(folder_size(&folder));
        let mut ready = Vec::new();
        for part in &sent_files {
            let rule = allowed.lines().map(|line| line.split('|').collect::<Vec<_>>()).find(|rule| rule.len() == 4 && rule[0] == form && rule[1] == part.name);
            let Some(rule) = rule.filter(|_| !ready.iter().any(|(field, _, _): &(String, &[u8], &str)| *field == part.name)) else {
                return Reply::text(400, &format!("aucun champ de fichier « {} » dans ce formulaire", part.name));
            };
            if part.bytes.len() as u64 > rule[3].parse::<u64>().unwrap_or(0) {
                return Reply::text(413, "fichier trop lourd");
            }
            let Some(extension) = file_kind(&part.bytes).filter(|(kind, _)| rule[2].split(',').any(|k| k == *kind)).map(|(_, extension)| extension) else {
                return Reply::text(415, "sorte de fichier refusée");
            };
            place = match place.checked_sub(part.bytes.len() as u64) {
                Some(rest) => rest,
                None => return Reply::text(507, "trop de fichiers gardés pour cette page"),
            };
            ready.push((part.name.clone(), part.bytes.as_slice(), extension));
        }
        // Tout est vérifié : on range. Le nom donné par le visiteur n'est gardé que dans le message.
        let mut kept = Vec::new();
        for (field, bytes, extension) in ready {
            let name = format!("{}-{}.{extension}", now(), &new_visitor()[..12]);
            if std::fs::create_dir_all(&folder).and_then(|_| std::fs::write(folder.join(&name), bytes)).is_err() {
                return Reply::text(500, "fichier impossible à ranger");
            }
            kept.push(format!("{{\"field\":\"{field}\",\"file\":\"files/{}/{name}\",\"size\":{}}}", page_name(path), bytes.len()));
        }
        match self.keep_message(path, holo, form, &json, &format!("[{}]", kept.join(",")), member.map(|m|m.id)) {
            Ok(()) => Reply { status: 204, headers: common_headers(), body: Vec::new() },
            Err(reply) => reply,
        }
    }

    /// Range un message dans la base, sauf si la page en garde déjà trop.
    fn keep_message(&self, path: &str, model: &str, form: &str, submission: &str, files: &str, account: Option<i64>) -> Result<(), Reply> {
        let base = self.base.lock().map_err(|_| Reply::text(500, "base indisponible"))?;
        let already: i64 = base.query_row("SELECT COALESCE(SUM(LENGTH(submission)), 0) FROM messages WHERE model = ?1", params![model], |row| row.get(0)).map_err(|_| Reply::text(500, "base illisible"))?;
        if already > MESSAGES_PER_PAGE_MAX {
            return Err(Reply::text(507, "trop de messages gardés pour cette page"));
        }
        base.execute("INSERT INTO messages (received, page, model, form, submission, files, account) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![now() as i64, path, model, form, submission, files, account])
            .map_err(|_| Reply::text(500, "message impossible à ranger"))?;
        println!("Message reçu : {path} ({form})");
        Ok(())
    }
}

fn stored_in(base: &Connection, visitor: &str, page: &str) -> Option<Visit> {
    base.query_row("SELECT state, tried FROM visits WHERE visitor = ?1 AND page = ?2", params![visitor, page], |row| {
        let tried: String = row.get(1)?;
        Ok(Visit { state: row.get(0)?, tried: tried.split(',').filter(|t| !t.is_empty()).map(str::to_string).collect() })
    })
    .optional()
    .ok()
    .flatten()
}

fn store_in(base: &Connection, visitor: &str, page: &str, visit: &Visit) {
    let _ = base.execute(
        "INSERT INTO visits (visitor, page, state, tried, updated) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (visitor, page) DO UPDATE SET state = excluded.state, tried = excluded.tried, updated = excluded.updated",
        params![visitor, page, visit.state, visit.tried.join(","), now() as i64],
    );
}

/// Les valeurs partagées gardées pour une adresse, écrites comme l'état (`seats=19;likes=3`), et
/// le numéro de leur dernier changement (0 : jamais changées, elles valent leur départ).
fn shared_in(base: &Connection, key: &str) -> (String, i64) {
    let Ok(mut query) = base.prepare("SELECT name, value, version FROM shared WHERE page = ?1 ORDER BY rowid") else { return (String::new(), 0) };
    let rows: Vec<(String, String, i64)> = query.query_map(params![key], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).map(|rows| rows.flatten().collect()).unwrap_or_default();
    let version = rows.iter().map(|(_, _, version)| *version).max().unwrap_or(0);
    (rows.iter().map(|(name, value, _)| format!("{name}={value}")).collect::<Vec<_>>().join(";"), version)
}

/// L'adresse sous laquelle une page garde ses valeurs partagées (ADR-079) : celle de son fichier
/// (`/salle.holo`, même ouvert par `/salle`), ou, pour un modèle, l'adresse avec ses valeurs
/// décodées (`/concert/12` pour `/concert/{id}.holo`) : `/concert/12` et `/concert/13` ont
/// chacune leurs places. Une barre ou un « % » dans une valeur reste écrit `%2F`, `%25`.
fn shared_key(holo: &str, values: &[(String, String)]) -> String {
    if values.iter().all(|(_, raw)| raw.is_empty()) {
        return holo.to_string();
    }
    let pattern = holo.strip_suffix(".holo").unwrap_or(holo);
    pattern
        .split('/')
        .map(|part| match part.strip_prefix('{').and_then(|p| p.strip_suffix('}')) {
            Some(name) => values.iter().find(|(known, _)| known == name).and_then(|(_, raw)| crate::address::decode(raw)).unwrap_or_default().replace('%', "%25").replace('/', "%2F"),
            None => part.to_string(),
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Un changement envoyé en direct : son numéro, puis les valeurs (ADR-079).
fn live_event(written: &str, version: i64) -> String {
    format!("event: shared\nid: {version}\ndata: {written}\n\n")
}

/// Combien de sauvegardes garder : les plus récentes ; les plus anciennes sont effacées.
const BACKUPS_KEPT: usize = 14;
/// Une sauvegarde par jour tant que le serveur tourne.
const BACKUP_EVERY: u64 = 24 * 3600;

/// Sauvegarde la base d'un site (ADR-076) : une copie entière et cohérente, même pendant que le
/// serveur écrit (`VACUUM INTO`), dans `holo-data/backups/site-2026-10-07-2215-09.sqlite`
/// (l'heure universelle). Garde les quatorze plus récentes. Rend le chemin de la copie.
pub fn backup(folder: &Path) -> Result<PathBuf, String> {
    let data = folder.join(DATA_FOLDER);
    if !data.join("site.sqlite").is_file() {
        return Err("aucune base : ce dossier n'a jamais été servi par holo serve".into());
    }
    let base = Connection::open(data.join("site.sqlite")).map_err(|e| e.to_string())?;
    backup_into(&base, &data)
}

fn backup_into(base: &Connection, data: &Path) -> Result<PathBuf, String> {
    let backups = data.join("backups");
    std::fs::create_dir_all(&backups).map_err(|e| format!("{} : {e}", backups.display()))?;
    let seconds = now();
    let [year, month, day, _, hour, minute] = crate::state::from_unix_seconds(seconds);
    let target = backups.join(format!("site-{year}-{month:02}-{day:02}-{hour:02}{minute:02}-{:02}.sqlite", seconds % 60));
    if target.exists() {
        return Ok(target);
    }
    base.execute("VACUUM INTO ?1", params![target.to_string_lossy()]).map_err(|e| e.to_string())?;
    let mut kept: Vec<PathBuf> = std::fs::read_dir(&backups)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("site-") && n.ends_with(".sqlite")))
        .collect();
    kept.sort();
    let surplus = kept.len().saturating_sub(BACKUPS_KEPT);
    for old in &kept[..surplus] {
        let _ = std::fs::remove_file(old);
    }
    Ok(target)
}

/// La date de la dernière sauvegarde, en secondes, d'après le fichier le plus récent.
fn last_backup(data: &Path) -> Option<u64> {
    std::fs::read_dir(data.join("backups"))
        .ok()?
        .flatten()
        .filter_map(|entry| entry.metadata().ok()?.modified().ok()?.duration_since(UNIX_EPOCH).ok())
        .map(|age| age.as_secs())
        .max()
}

/// Les messages reçus par un site, du plus ancien au plus récent, une ligne JSON chacun : ce que
/// `holo messages` affiche à l'auteur (ADR-075).
pub fn messages(folder: &Path) -> Result<Vec<String>, String> {
    let base = Connection::open(folder.join(DATA_FOLDER).join("site.sqlite")).map_err(|e| e.to_string())?;
    let mut query = base.prepare("SELECT received, page, form, submission, files FROM messages ORDER BY id").map_err(|_| "aucun message reçu".to_string())?;
    let rows = query
        .query_map([], |row| {
            let (received, page, form, submission, files): (i64, String, String, String, String) = (row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?);
            let quoted = |text: &str| text.replace('\\', "\\\\").replace('"', "\\\"");
            Ok(format!("{{\"received\":{received},\"page\":\"{}\",\"form\":\"{}\",\"submission\":{submission},\"files\":{files}}}", quoted(&page), quoted(&form)))
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Lance le serveur et ne rend plus la main. `0.0.0.0` : un téléphone sur le même Wi-Fi le voit.
pub fn serve(folder: &Path, web: &Path, port: u16) -> Result<(), String> {
    let site = Arc::new(Site::open(folder, web)?);
    let server = Arc::new(tiny_http::Server::http(("0.0.0.0", port)).map_err(|e| format!("port {port} : {e}"))?);
    println!("Site servi      : {}", site.folder.display());
    println!("Sa base         : {}", site.folder.join(DATA_FOLDER).join("site.sqlite").display());
    println!("Sur ce PC       : http://localhost:{port}");
    println!("Sur le téléphone (même Wi-Fi) : http://<adresse de ce PC>:{port}");
    // Une sauvegarde au départ si la dernière a plus d'un jour, puis une par jour (ADR-076).
    {
        let site = Arc::clone(&site);
        std::thread::spawn(move || loop {
            let data = site.folder.join(DATA_FOLDER);
            let wait = match last_backup(&data) {
                Some(last) if now().saturating_sub(last) < BACKUP_EVERY => BACKUP_EVERY - now().saturating_sub(last),
                _ => {
                    match site.base.lock().map_err(|_| "base indisponible".to_string()).and_then(|base| backup_into(&base, &data)) {
                        Ok(path) => println!("Sauvegarde      : {}", path.display()),
                        Err(error) => eprintln!("Sauvegarde impossible : {error}"),
                    }
                    BACKUP_EVERY
                }
            };
            std::thread::sleep(std::time::Duration::from_secs(wait));
        });
    }
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let (site, server) = (Arc::clone(&site), Arc::clone(&server));
            std::thread::spawn(move || {
                while let Ok(request) = server.recv() {
                    reply_to(&site, request);
                }
            })
        })
        .collect();
    for worker in workers {
        let _ = worker.join();
    }
    Ok(())
}

fn reply_to(site: &Site, mut request: tiny_http::Request) {
    let header = |name: &str| request.headers().iter().find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name)).map(|h| h.value.as_str().to_string()).unwrap_or_default();
    let (accept, cookie, content_type, origin, host, referer) = (header("Accept"), header("Cookie"), header("Content-Type"), header("Origin"), header("Host"), header("Referer"));
    let peer=request.remote_addr().map(|p|p.ip().to_string()).unwrap_or_default();
    // Toutes les lignes `X-Forwarded-For`, dans l'ordre : un proxy en ajoute une à la fin, ou
    // complète la dernière ; seule la dernière adresse vient de lui (client_address).
    let forwarded = request.headers().iter().filter(|h| h.field.as_str().as_str().eq_ignore_ascii_case("X-Forwarded-For")).map(|h| h.value.as_str()).collect::<Vec<_>>().join(", ");
    let method = request.method().as_str().to_string();
    let url = request.url().to_string();
    // Une page qui écoute ses valeurs partagées en direct (ADR-079) : la connexion reste ouverte,
    // tenue par un fil à elle ; ce fil-ci retourne aussitôt servir les autres.
    if method == "GET" && accept.contains("text/event-stream") {
        let ask = Ask { method: &method, url: &url, accept: &accept, cookie: &cookie, content_type: &content_type, origin: &origin, host: &host, referer: &referer, peer: &peer, forwarded: &forwarded, body: &[] };
        match site.live_page(&ask) {
            Ok((key, source)) => site.listen(&key, &source, request.into_writer()),
            Err(reply) => {
                let mut response = tiny_http::Response::from_data(reply.body).with_status_code(reply.status);
                for (name, value) in reply.headers {
                    if let Ok(header) = tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()) {
                        response.add_header(header);
                    }
                }
                let _ = request.respond(response);
            }
        }
        return;
    }
    let mut body = Vec::new();
    if method == "POST" {
        let limit = if content_type.starts_with("multipart/form-data") { WITH_FILES_MAX } else { BODY_MAX };
        let _ = request.as_reader().take(limit + 1).read_to_end(&mut body);
    }
    let reply = site.answer(&Ask { method: &method, url: &url, accept: &accept, cookie: &cookie, content_type: &content_type, origin: &origin, host: &host, referer: &referer, peer: &peer, forwarded: &forwarded, body: &body });
    let mut response = tiny_http::Response::from_data(if method == "HEAD" { Vec::new() } else { reply.body }).with_status_code(reply.status);
    for (name, value) in reply.headers {
        if let Ok(header) = tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()) {
            response.add_header(header);
        }
    }
    let _ = request.respond(response);
}

/// Un morceau d'un envoi en plusieurs morceaux : son nom de champ, ses octets.
struct Part {
    name: String,
    bytes: Vec<u8>,
}

/// Lit un envoi en plusieurs morceaux (`multipart/form-data`), ou rien s'il est mal formé.
fn read_multipart(body: &[u8], content_type: &str) -> Option<Vec<Part>> {
    let boundary = content_type.split(';').map(str::trim).find_map(|part| part.strip_prefix("boundary="))?.trim_matches('"');
    let separator = format!("--{boundary}").into_bytes();
    let find = |from: usize| body.get(from..).and_then(|rest| rest.windows(separator.len()).position(|w| w == separator.as_slice())).map(|at| from + at);
    let mut parts = Vec::new();
    let mut start = find(0)?;
    loop {
        start += separator.len();
        if body.get(start..start + 2) == Some(b"--") {
            return Some(parts);
        }
        let end = find(start)?;
        // Sans le retour à la ligne de part et d'autre.
        let chunk = body.get(start + 2..end.checked_sub(2)?)?;
        let cut = chunk.windows(4).position(|w| w == b"\r\n\r\n")?;
        let headers = String::from_utf8_lossy(&chunk[..cut]);
        let name = headers.split(';').map(str::trim).find_map(|h| h.strip_prefix("name=\""))?.split('"').next()?.to_string();
        parts.push(Part { name, bytes: chunk[cut + 4..].to_vec() });
        if parts.len() > 64 {
            return None;
        }
        start = end;
    }
}

/// La sorte d'un fichier, lue dans ses premiers octets, jamais dans son nom (ADR-059).
fn file_kind(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        Some(("image", "png"))
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some(("image", "jpg"))
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some(("image", "gif"))
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        Some(("image", "webp"))
    } else if bytes.starts_with(b"%PDF-") {
        Some(("pdf", "pdf"))
    } else {
        None
    }
}

/// Le nom d'une page pour ranger ses fichiers : `/lecons/76-envoyer.holo` → `lecons_76-envoyer`.
fn page_name(path: &str) -> String {
    path.trim_start_matches('/').trim_end_matches(".holo").chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}

fn folder_size(folder: &Path) -> u64 {
    std::fs::read_dir(folder).map(|entries| entries.flatten().filter_map(|e| e.metadata().ok()).filter(|m| m.is_file()).map(|m| m.len()).sum()).unwrap_or(0)
}

/// Les sons demandés (« ! ») ne se jouent pas sans le moteur : ils ne sont pas gardés.
fn without_sounds(state: &str) -> String {
    state.split(';').filter(|chunk| !chunk.starts_with("!=")).collect::<Vec<_>>().join(";")
}

/// Les en-têtes de toute réponse : pas de devinette sur le type, pas de cache périmé.
fn common_headers() -> Vec<(String, String)> {
    vec![("Cache-Control".into(), "no-cache".into()), ("X-Content-Type-Options".into(), "nosniff".into()), ("Referrer-Policy".into(), "same-origin".into())]
}

/// Les en-têtes d'une réponse faite pour un membre connecté (ADR-081) : elle porte son nom et ses
/// valeurs, elle n'est donc jamais gardée (un ordinateur partagé, le bouton « retour » après s'être
/// déconnecté).
fn member_headers() -> Vec<(String, String)> {
    common_headers().into_iter().map(|(name, value)| if name == "Cache-Control" { (name, "private, no-store".to_string()) } else { (name, value) }).collect()
}

/// La clé sous laquelle l'état d'une visite est gardé : le compte d'un membre connecté (ses valeurs
/// le suivent sur tous ses appareils, ADR-081), sinon le numéro du cookie du visiteur.
fn visit_key(member: Option<&crate::accounts::Member>, cookie: &str) -> Option<String> {
    member.map(crate::accounts::Member::visit_key).or_else(|| visitor_of(cookie))
}

/// Une page réservée aux membres (`access: members`, ADR-081) ? Lue avec ses imports et son
/// adresse ; une page que le moteur refuse reste réservée si son texte le dit.
fn members_only(file: &Path, values: &[(String, String)]) -> bool {
    let Ok(source) = source_at(file, values) else { return false };
    match crate::holo::read(&source) {
        Ok(program) => crate::account::members_only(&program),
        Err(_) => crate::account::members_in_text(&source),
    }
}

/// Le chemin d'une adresse, décodé (`/le%20site.holo` → `/le site.holo`), sans sa question.
fn url_path(url: &str) -> Option<String> {
    let path = url.split(['?', '#']).next().unwrap_or("");
    if !path.starts_with('/') {
        return None;
    }
    let decoded = crate::gestures::percent_decode(path, false);
    (!decoded.contains('\u{FFFD}') && !decoded.contains('\0')).then_some(decoded)
}

/// Le numéro du visiteur, dans son cookie : 32 chiffres hexadécimaux, sinon aucun.
pub(crate) fn visitor_of(cookie: &str) -> Option<String> {
    cookie
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE)
        .map(|(_, value)| value.to_string())
        .filter(|value| value.len() == 32 && value.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Un numéro de visiteur neuf, tiré au hasard par le système : impossible à deviner.
fn new_visitor() -> String {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).expect("le système ne donne pas de hasard");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// L'état de départ d'une page : ses valeurs de départ, et ses données (`Data(from:)`) si le
/// fichier est rangé à côté et pèse 64 Ko au plus (ADR-064).
fn starting_state(source: &str, file: &Path) -> String {
    let mut state = crate::initial_state(source);
    let data = crate::data(source);
    let mut parts = data.split('|');
    let (from, name) = (parts.next().unwrap_or(""), parts.nth(1).unwrap_or(""));
    if from.is_empty() || from.contains("..") || from.contains("://") {
        return state;
    }
    let path = file.parent().unwrap_or(Path::new(".")).join(from);
    if std::fs::metadata(&path).is_ok_and(|m| m.len() <= crate::state::DATA_BYTES as u64) {
        if let Ok(json) = std::fs::read_to_string(path) {
            let received = crate::receive(source, &state, &json);
            if !received.is_empty() {
                state = received;
            }
            if !name.is_empty() {
                let done = crate::arbitrate(source, &state, &format!("{name}.done"));
                if !done.is_empty() {
                    state = done;
                }
            }
        }
    }
    without_sounds(&state)
}

/// Extrait d'une copie d'état seulement les saisies que la page déclare. Leur validation
/// (bornes, longueur, options, dates) reste celle de `visitor_gesture`. Les nombres de l'état
/// sont à l'échelle : 1250 pour un champ déclaré à 12.50 devient « 12.50 », pas « 1250 ».
/// Une valeur absente ou mal codée ne remet pas le champ à sa valeur de départ.
fn inputs_from_state(source: &str, state: &str) -> Vec<(String, String)> {
    let Ok(program) = crate::check_page(source) else { return Vec::new() };
    let numbers = crate::state::initial(&program).unwrap_or_default();
    let texts = crate::state::initial_texts(&program);
    let mut names: Vec<String> = Vec::new();
    let _ = crate::rules::for_each_block(&program.root, &mut |block| {
        if matches!(block.name.as_str(), "Input" | "Checkbox" | "Slider" | "Choice") {
            if let Some(crate::holo::Value::Name(name)) = block.argument("value").map(|argument| &argument.value) {
                if name != crate::gestures::SIGNAL && (!program.shared.contains(name)||texts.iter().any(|(n,_)|n==name)) && !names.contains(name) {
                    names.push(name.clone());
                }
            }
        }
        Ok(())
    });
    names.into_iter().filter_map(|name| {
        let written = state.split(';').rev().find_map(|chunk| {
            let (known, written) = chunk.split_once('=')?;
            (known == name).then_some(written)
        })?;
        let value = if texts.iter().any(|(known, _)| *known == name) {
            crate::state::decode(written.strip_prefix('\'')?)?
        } else if numbers.iter().any(|(known, _)| *known == name) {
            crate::state::format_decimal(written.parse().ok()?, crate::state::places(&program, &name))
        } else {
            return None;
        };
        Some((name, value))
    }).collect()
}

/// Le texte d'une page, avec ses imports, puis les valeurs de son adresse (ADR-078).
fn source_at(file: &Path, values: &[(String, String)]) -> std::io::Result<String> {
    let source = read_with_imports(file)?;
    Ok(if values.is_empty() { source } else { crate::address::joined(&source, values) })
}

/// Le texte d'une page pour ce visiteur : `source_at`, puis le nom du membre connecté (ADR-081),
/// que la page lit (`{account}`, `signedIn`) sans pouvoir le changer ; vide pour un visiteur.
fn source_for(file: &Path, values: &[(String, String)], member: Option<&crate::accounts::Member>) -> std::io::Result<String> {
    Ok(crate::account::joined(&source_at(file, values)?, member.map_or("", |member| member.name.as_str())))
}

/// Le dossier `{x}`, ou pour le dernier morceau le fichier `{x}.holo`, d'un dossier ; le premier
/// par ordre alphabétique s'il y en a plusieurs.
fn holder(folder: &Path, last: bool) -> Option<String> {
    let mut found: Vec<String> = std::fs::read_dir(folder)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| if last { entry.path().is_file() } else { entry.path().is_dir() })
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| {
            let stem = if last { name.strip_suffix(".holo") } else { Some(name.as_str()) };
            stem.and_then(|s| s.strip_prefix('{')?.strip_suffix('}')).is_some_and(crate::address::valid_name)
        })
        .collect();
    found.sort();
    found.into_iter().next()
}

/// Le texte d'un fichier, avec les fichiers qu'il importe joints à la suite, lus à côté de lui.
fn read_with_imports(file: &Path) -> std::io::Result<String> {
    let mut source = std::fs::read_to_string(file)?;
    let here = file.parent().unwrap_or(Path::new("."));
    for name in crate::imports(&source).split(';').filter(|name| !name.is_empty() && !name.contains("..")).map(str::to_string).collect::<Vec<_>>() {
        if let Ok(text) = std::fs::read_to_string(here.join(&name)) {
            source.push(crate::holo::NEXT_FILE);
            source.push_str(&name);
            source.push(crate::holo::NAME_SEPARATOR);
            source.push_str(&text);
        }
    }
    Ok(source)
}

/// La page du fichier posée dans la page d'entrée, avec son titre, sa langue, sa description,
/// son image de partage et sa petite image d'onglet (ADR-038, ADR-042).
/// Ce que l'adresse porte après le `?`, sans le `#…` : `tab=photos&page=2`.
fn query_of(url: &str) -> &str {
    url.split('#').next().unwrap_or("").split_once('?').map_or("", |(_, query)| query)
}

/// L'état d'un visiteur avec les valeurs de l'adresse (ADR-091) ; tel quel si le moteur refuse.
fn with_query(source: &str, state: &str, query: &str) -> String {
    let after = crate::from_query(source, state, query);
    if after.is_empty() { state.to_string() } else { after }
}

fn filled_template(template: &str, html: &str) -> String {
    let head = &html[..html.len().min(20_000)];
    let read = |name: &str| {
        let marker = format!("{name}=\"");
        head.find(&marker).map(|start| {
            let rest = &head[start + marker.len()..];
            rest[..rest.find('"').unwrap_or(0)].to_string()
        })
    };
    let title = read("data-title").filter(|t| !t.is_empty()).unwrap_or_else(|| "HoloCode".into());
    let mut header = format!("<title>{title}</title><meta property=\"og:title\" content=\"{title}\">");
    if let Some(icon) = read("data-icon") {
        header.push_str(&format!("<link rel=\"icon\" href=\"{icon}\">"));
    }
    if let Some(description) = read("data-description") {
        header.push_str(&format!("<meta name=\"description\" content=\"{description}\"><meta property=\"og:description\" content=\"{description}\">"));
    }
    if let Some(image) = read("data-image") {
        header.push_str(&format!("<meta property=\"og:image\" content=\"{image}\">"));
    }
    let mut page = template.replacen("<div id=\"page\"></div>", &format!("<div id=\"page\">{html}</div>"), 1).replacen("<title>HoloCode</title>", &header, 1);
    if let Some(language) = read("data-lang") {
        page = page.replacen("<html lang=\"fr\">", &format!("<html lang=\"{language}\">"), 1);
    }
    page
}

/// L'heure universelle, pour une page qui lit l'heure (ADR-039).
fn set_clock() {
    crate::set_now(crate::state::from_unix_seconds(now()));
}

pub(crate) fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn content_type(file: &Path) -> &'static str {
    match file.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "holo" | "txt" | "vtt" => "text/plain; charset=utf-8",
        "json" => "application/json",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHOP: &str = "Page(\n  title: \"Boutique\",\n  state: State(cart: 0, buyer: \"\"),\n  children: [\n    H1(\"Boutique\"),\n    P(\"Panier : {cart}\"),\n    Input(label: \"Nom\", value: buyer),\n    P(\"Bonjour {buyer}\"),\n    Button(name: Add, text: \"Ajouter\"),\n  ],\n  rules: [\n    On(Add.tap, effect: cart.add(1)),\n  ],\n)\n";

    fn site() -> (Site, PathBuf) {
        let folder = std::env::temp_dir().join(format!("holo-serve-{}", new_visitor()));
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("shop.holo"), SHOP).unwrap();
        std::fs::write(folder.join("secret.txt"), "non").unwrap();
        let web = Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
        (Site::open(&folder, &web).unwrap(), folder)
    }

    fn ask<'a>(method: &'a str, url: &'a str, cookie: &'a str, body: &'a [u8]) -> Ask<'a> {
        Ask { method, url, accept: "text/html", cookie, content_type: "application/x-www-form-urlencoded", origin: "", host: "localhost:8080", referer: "", peer: "127.0.0.1", forwarded: "", body }
    }

    #[test]
    fn an_address_carries_a_value_with_and_without_javascript() {
        // ADR-078 : `profils/{nom}.holo` sert `/profils/ada` ; la page lit `{nom}`, sans le changer.
        let (site, folder) = site();
        std::fs::create_dir_all(folder.join("profils")).unwrap();
        std::fs::write(
            folder.join("profils").join("{nom}.holo"),
            "Page(title: \"Profil\", state: State(likes: 0, email: \"\"), children: [ H1(\"Bonjour, {nom}\"), If(nom, is: \"ada\", children: [ P(\"Ada a écrit le premier programme.\") ]), P(\"J'aime : {likes}\"), Button(name: Like, text: \"J'aime\"), Form(name: Contact, children: [ Input(value: email, type: email, label: \"E-mail\", required: true), Button(name: Send, text: \"Envoyer\") ]) ], rules: [ On(Like.tap, effect: likes.add(1)), On(Send.tap, effect: Contact.send) ])",
        )
        .unwrap();
        std::fs::write(folder.join("contact.holo"), "Page(title: \"Contact\", children: [ H1(\"Écris-nous\") ])").unwrap();
        // La page fabriquée par le serveur, avec la valeur, et le fichier nommé pour le moteur.
        let page = String::from_utf8(site.answer(&ask("GET", "/profils/ada", "", b"")).body).unwrap();
        assert!(page.contains("Bonjour, <span data-state=\"nom\">ada</span>") && page.contains("Ada a écrit le premier programme."), "{page}");
        assert!(page.contains("<meta name=\"holo-file\" content=\"/profils/{nom}.holo\">"), "{page}");
        let accented = String::from_utf8(site.answer(&ask("GET", "/profils/Ad%C3%A9", "", b"")).body).unwrap();
        assert!(accented.contains(">Adé<") && accented.contains("data-if=\"nom|is=&quot;ada&quot;\" hidden"), "{accented}");
        // Le texte du modèle, pour le moteur du navigateur ; le modèle ouvert lui-même, un nom vide.
        let mut plain = ask("GET", "/profils/%7Bnom%7D.holo", "", b"");
        plain.accept = "text/plain";
        assert!(String::from_utf8(site.answer(&plain).body).unwrap().starts_with("Page(title: \"Profil\""));
        assert!(String::from_utf8(site.answer(&ask("GET", "/profils/%7Bnom%7D.holo", "", b"")).body).unwrap().contains("Bonjour, <span data-state=\"nom\"></span>"));
        // Une adresse sans `.holo`, et une adresse sans modèle.
        let contact = String::from_utf8(site.answer(&ask("GET", "/contact", "", b"")).body).unwrap();
        assert!(contact.contains("Écris-nous") && contact.contains("<meta name=\"holo-file\" content=\"/contact.holo\">"), "{contact}");
        assert_eq!(site.answer(&ask("GET", "/profils/ada/plus", "", b"")).status, 404);
        assert_eq!(site.answer(&ask("GET", "/holo-data/x", "", b"")).status, 404);
        // Sans JavaScript : un toucher à l'adresse ; l'état est gardé pour cette adresse-là.
        let touched = site.answer(&ask("POST", "/profils/ada", "", b"signal=Like.tap"));
        assert_eq!(touched.status, 303);
        assert!(touched.headers.iter().any(|(name, value)| name == "Location" && value == "/profils/ada"));
        let cookie = touched.headers.iter().find(|(name, _)| name == "Set-Cookie").map(|(_, value)| value.split(';').next().unwrap().to_string()).unwrap();
        let again = String::from_utf8(site.answer(&ask("GET", "/profils/ada", &cookie, b"")).body).unwrap();
        assert!(again.contains("J'aime : <span data-state=\"likes\">1</span>") && again.contains(">ada<"), "{again}");
        let other = String::from_utf8(site.answer(&ask("GET", "/profils/grace", &cookie, b"")).body).unwrap();
        assert!(other.contains("J'aime : <span data-state=\"likes\">0</span>"), "{other}");
        // Avec JavaScript : un formulaire envoyé à l'adresse, vérifié avec la valeur, rangé avec son modèle.
        let mut json = ask("POST", "/profils/ada", "", br#"{"form":"Contact","values":{"email":"ada@example.org"}}"#);
        json.content_type = "application/json";
        assert_eq!(site.answer(&json).status, 204);
        let base = site.base.lock().unwrap();
        let (page, model): (String, String) = base.query_row("SELECT page, model FROM messages", [], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
        assert_eq!((page.as_str(), model.as_str()), ("/profils/ada", "/profils/{nom}.holo"));
        drop(base);
        let _ = std::fs::remove_dir_all(&folder);
    }

    #[test]
    fn the_address_carries_the_history_of_a_page_without_javascript() {
        // ADR-091 : `address: [tab, page]` ; l'adresse dit l'onglet, même sans JavaScript.
        let (site, folder) = site();
        std::fs::write(
            folder.join("galerie.holo"),
            "Page(title: \"Galerie\", state: State(tab: \"toiles\", page: 1), address: [tab, page], children: [ P(\"Onglet {tab}, page {page}\"), Button(name: Dessins, text: \"Dessins\"), Button(name: Next, text: \"Suivante\") ], rules: [ On(Dessins.tap, effect: [tab.set(\"dessins\"), page.set(1)]), On(Next.tap, effect: page.add(1)) ])",
        )
        .unwrap();
        // Une adresse partagée : la page fabriquée a ses valeurs.
        let shared = String::from_utf8(site.answer(&ask("GET", "/galerie.holo?tab=dessins&page=3", "", b"")).body).unwrap();
        assert!(shared.contains("Onglet <span data-state=\"tab\">dessins</span>, page <span data-state=\"page\">3</span>"), "{shared}");
        // Un toucher sans JavaScript mène à l'adresse des nouvelles valeurs.
        let touched = site.answer(&ask("POST", "/galerie.holo?tab=dessins&page=3", "", b"signal=Next.tap"));
        assert!(touched.headers.iter().any(|(name, value)| name == "Location" && value == "/galerie.holo?tab=dessins&page=4"), "{:?}", touched.headers);
        let cookie = touched.headers.iter().find(|(name, _)| name == "Set-Cookie").map(|(_, value)| value.split(';').next().unwrap().to_string()).unwrap();
        // « Précédent » : le navigateur redemande l'adresse d'avant ; elle dit encore la page 3.
        let back = String::from_utf8(site.answer(&ask("GET", "/galerie.holo?tab=dessins&page=3", &cookie, b"")).body).unwrap();
        assert!(back.contains("page <span data-state=\"page\">3</span>"), "{back}");
        // Et l'adresse nue, la page du début.
        let start = String::from_utf8(site.answer(&ask("GET", "/galerie.holo", &cookie, b"")).body).unwrap();
        assert!(start.contains("Onglet <span data-state=\"tab\">toiles</span>, page <span data-state=\"page\">1</span>"), "{start}");
        let first = site.answer(&ask("POST", "/galerie.holo", &cookie, b"signal=Dessins.tap"));
        assert!(first.headers.iter().any(|(name, value)| name == "Location" && value == "/galerie.holo?tab=dessins"), "{:?}", first.headers);
        let _ = std::fs::remove_dir_all(&folder);
    }

    #[test]
    fn a_visitor_without_javascript_fills_the_cart() {
        let (site, folder) = site();
        let first = site.answer(&ask("GET", "/shop.holo", "", b""));
        let html = String::from_utf8(first.body).unwrap();
        assert_eq!(first.status, 200);
        assert!(html.contains("Panier : <span data-state=\"cart\">0</span>") && html.contains("value=\"Add.tap\"") && html.contains("<form id=\"holo-gestures\" method=\"post\" hidden>"), "{html}");
        assert!(html.contains("<title>Boutique</title>"), "{html}");
        // Deux touchers, le nom écrit avec le premier.
        let reply = site.answer(&ask("POST", "/shop.holo", "", b"buyer=Ada&signal=Add.tap"));
        assert_eq!(reply.status, 303);
        let cookie = reply.headers.iter().find(|(n, _)| n == "Set-Cookie").map(|(_, v)| v.split(';').next().unwrap().to_string()).unwrap();
        assert!(cookie.starts_with("holo_visitor=") && cookie.len() == "holo_visitor=".len() + 32, "{cookie}");
        let again = site.answer(&ask("POST", "/shop.holo", &cookie, b"buyer=Ada&signal=Add.tap"));
        assert!(again.headers.iter().all(|(n, _)| n != "Set-Cookie"));
        let html = String::from_utf8(site.answer(&ask("GET", "/shop.holo", &cookie, b"")).body).unwrap();
        assert!(html.contains("Panier : <span data-state=\"cart\">2</span>") && html.contains("Bonjour <span data-state=\"buyer\">Ada</span>") && html.contains("data-visit=\"cart=2;"), "{html}");
        // Un autre visiteur part de zéro : rien n'est partagé.
        let html = String::from_utf8(site.answer(&ask("GET", "/shop.holo", "", b"")).body).unwrap();
        assert!(html.contains("Panier : <span data-state=\"cart\">0</span>"), "{html}");
        // Gardé dans la base : un nouveau serveur sur le même dossier s'en souvient.
        let web = Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
        let reopened = Site::open(&folder, &web).unwrap();
        let html = String::from_utf8(reopened.answer(&ask("GET", "/shop.holo", &cookie, b"")).body).unwrap();
        assert!(html.contains("Panier : <span data-state=\"cart\">2</span>"), "{html}");
        let _ = std::fs::remove_dir_all(folder);
    }

    const CONTACT: &str = "Page(\n  title: \"Contact\",\n  state: State(name: \"\", photo: \"\", sent: 0),\n  children: [\n    Form(name: Contact, children: [\n      Input(value: name, label: \"Nom\", required: true, min: 2),\n      Input(type: file, value: photo, label: \"Photo\", accept: image, max: 500KB),\n      Button(name: Send, text: \"Envoyer\"),\n    ]),\n    If(sent, is: 1, children: [ P(\"Merci\") ]),\n  ],\n  rules: [\n    On(Send.tap, effect: Contact.send),\n    On(Contact.sent, effect: sent.set(1)),\n  ],\n)\n";

    #[test]
    fn a_form_sent_without_javascript() {
        let (site, folder) = site();
        std::fs::write(folder.join("contact.holo"), CONTACT).unwrap();
        // Envoyer vide : rien n'est rangé, le message d'erreur est écrit sous le champ.
        let reply = site.answer(&ask("POST", "/contact.holo", "", b"name=&signal=Send.tap"));
        let cookie = reply.headers.iter().find(|(n, _)| n == "Set-Cookie").map(|(_, v)| v.split(';').next().unwrap().to_string()).unwrap();
        let html = String::from_utf8(site.answer(&ask("GET", "/contact.holo", &cookie, b"")).body).unwrap();
        assert!(html.contains("data-tried=\"1\"") && html.contains("aria-invalid=\"true\" aria-describedby=\"holo-error-Contact-name\"") && html.contains("<p class=\"holo-error\" id=\"holo-error-Contact-name\">"), "{html}");
        assert!(html.contains(" data-visit=\"sent=0;"), "{html}");
        assert!(messages(&folder).unwrap().is_empty());
        // Corrigé : rangé, puis `Contact.sent`.
        site.answer(&ask("POST", "/contact.holo", &cookie, b"name=Ada&signal=Send.tap"));
        let html = String::from_utf8(site.answer(&ask("GET", "/contact.holo", &cookie, b"")).body).unwrap();
        assert!(html.contains(" data-visit=\"sent=1;") && !html.contains("class=\"holo-error\"") && !html.contains("data-tried"), "{html}");
        let kept = messages(&folder).unwrap();
        assert_eq!(kept.len(), 1);
        assert!(kept[0].contains("\"page\":\"/contact.holo\",\"form\":\"Contact\",\"submission\":{\"form\":\"Contact\",\"values\":{\"name\":\"Ada\""), "{}", kept[0]);
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn a_form_sent_by_the_engine() {
        let (site, folder) = site();
        std::fs::write(folder.join("contact.holo"), CONTACT).unwrap();
        let json = |body: &'static [u8]| {
            let mut asked = ask("POST", "/contact.holo", "", body);
            asked.content_type = "application/json";
            site.answer(&asked)
        };
        assert_eq!(json(br#"{"form":"Contact","values":{"name":"Ada"}}"#).status, 204);
        let refused = json(br#"{"form":"Contact","values":{"name":"A"}}"#);
        assert_eq!(refused.status, 422, "{}", String::from_utf8_lossy(&refused.body));
        assert_eq!(json(br#"{"form":"Contact","values":{"admin":1}}"#).status, 422);
        assert_eq!(json(b"pas du json").status, 422);
        // Avec un fichier : une image reconnue à ses octets ; un faux PNG refusé.
        let multipart = |file: &[u8]| {
            let mut body = b"--B\r\nContent-Disposition: form-data; name=\"submission\"\r\n\r\n{\"form\":\"Contact\",\"values\":{\"name\":\"Bob\",\"photo\":\"moi.png\"}}\r\n--B\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"moi.png\"\r\nContent-Type: image/png\r\n\r\n".to_vec();
            body.extend_from_slice(file);
            body.extend_from_slice(b"\r\n--B--\r\n");
            body
        };
        let png = multipart(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 1, 2, 3]);
        let mut asked = ask("POST", "/contact.holo", "", &png);
        asked.content_type = "multipart/form-data; boundary=B";
        assert_eq!(site.answer(&asked).status, 204);
        let fake = multipart(b"<script>alert(1)</script>");
        let mut asked = ask("POST", "/contact.holo", "", &fake);
        asked.content_type = "multipart/form-data; boundary=B";
        assert_eq!(site.answer(&asked).status, 415);
        let kept = messages(&folder).unwrap();
        assert_eq!(kept.len(), 2);
        let file = kept[1].split("\"file\":\"").nth(1).unwrap().split('"').next().unwrap();
        assert!(file.starts_with("files/contact/") && file.ends_with(".png"), "{file}");
        assert_eq!(std::fs::read(folder.join(DATA_FOLDER).join(file)).unwrap().len(), 11);
        // Le fichier rangé n'est pas servi.
        assert_eq!(site.answer(&ask("GET", &format!("/{DATA_FOLDER}/{file}"), "", b"")).status, 404);
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn a_backup_is_a_whole_copy_and_old_ones_go() {
        let (site, folder) = site();
        let reply = site.answer(&ask("POST", "/shop.holo", "", b"signal=Add.tap"));
        let cookie = reply.headers.iter().find(|(n, _)| n == "Set-Cookie").map(|(_, v)| v.split(';').next().unwrap().to_string()).unwrap();
        let copy = backup(&folder).unwrap();
        assert!(copy.starts_with(folder.join(DATA_FOLDER).join("backups")), "{}", copy.display());
        // La copie se relit seule : la valeur du visiteur y est.
        let saved = Connection::open(&copy).unwrap();
        let state: String = saved.query_row("SELECT state FROM visits WHERE visitor = ?1", params![cookie.trim_start_matches("holo_visitor=")], |row| row.get(0)).unwrap();
        assert!(state.starts_with("cart=1;"), "{state}");
        // Seules les quatorze plus récentes restent.
        let backups = folder.join(DATA_FOLDER).join("backups");
        for day in 1..=20 {
            std::fs::write(backups.join(format!("site-2020-01-{day:02}-0000-00.sqlite")), b"").unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(1100));
        backup(&folder).unwrap();
        let count = std::fs::read_dir(&backups).unwrap().count();
        assert_eq!(count, BACKUPS_KEPT, "{count}");
        assert!(!backups.join("site-2020-01-01-0000-00.sqlite").exists());
        // Jamais servies.
        assert_eq!(site.answer(&ask("GET", "/holo-data/backups/site-2020-01-20-0000-00.sqlite", "", b"")).status, 404);
        // Un dossier jamais servi n'a rien à sauvegarder.
        assert!(backup(&std::env::temp_dir().join("holo-jamais-servi")).is_err());
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn what_is_refused() {
        let (site, folder) = site();
        // Un signal qui n'est pas un toucher ne change rien.
        let reply = site.answer(&ask("POST", "/shop.holo", "", b"signal=Add.hover"));
        let cookie = reply.headers.iter().find(|(n, _)| n == "Set-Cookie").map(|(_, v)| v.split(';').next().unwrap().to_string()).unwrap();
        let html = String::from_utf8(site.answer(&ask("GET", "/shop.holo", &cookie, b"")).body).unwrap();
        assert!(html.contains("Panier : <span data-state=\"cart\">0</span>"), "{html}");
        // Un geste venu d'un autre site.
        let mut foreign = ask("POST", "/shop.holo", "", b"signal=Add.tap");
        foreign.origin = "https://ailleurs.example";
        assert_eq!(site.answer(&foreign).status, 403);
        // Un corps d'une autre sorte, un corps trop lourd, une page absente, une autre méthode.
        let mut other = ask("POST", "/shop.holo", "", b"x");
        other.content_type = "text/plain";
        assert_eq!(site.answer(&other).status, 415);
        let heavy = vec![b'a'; BODY_MAX as usize + 1];
        assert_eq!(site.answer(&ask("POST", "/shop.holo", "", &heavy)).status, 413);
        assert_eq!(site.answer(&ask("POST", "/absent.holo", "", b"signal=Add.tap")).status, 404);
        assert_eq!(site.answer(&ask("DELETE", "/shop.holo", "", b"")).status, 405);
        // La base, un fichier caché, une sortie du dossier : jamais servis.
        for url in ["/holo-data/site.sqlite", "/.env", "/../shop.holo", "/%2e%2e/shop.holo", "/a%5c..%5cshop.holo"] {
            assert_eq!(site.answer(&ask("GET", url, "", b"")).status, 404, "{url}");
        }
        // Un fichier ordinaire du site, et le moteur pour le navigateur.
        assert_eq!(site.answer(&ask("GET", "/secret.txt", "", b"")).body, b"non");
        assert_eq!(site.answer(&ask("GET", "/page-engine.js", "", b"")).status, 200);
        // Le moteur demande le fichier lui-même.
        let mut raw = ask("GET", "/shop.holo", "", b"");
        raw.accept = "text/plain";
        assert_eq!(site.answer(&raw).body, SHOP.as_bytes());
        // Un cookie inventé n'est pas un visiteur.
        assert_eq!(visitor_of("holo_visitor=../../etc"), None);
        assert_eq!(visitor_of("a=1; holo_visitor=0123456789abcdef0123456789abcdef").as_deref(), Some("0123456789abcdef0123456789abcdef"));
        let _ = std::fs::remove_dir_all(folder);
    }

    const CONCERT: &str = "Page(\n  title: \"Concert\",\n  state: State(booked: 0),\n  shared: Shared(seats: 2, likes: 0),\n  children: [\n    H1(\"Concert\"),\n    P(\"{seats} place(s)\"),\n    If(seats, over: 0, children: [ If(booked, is: 0, children: [ Button(name: Book, text: \"Réserver\") ]) ], else: [ P(\"Complet\") ]),\n    If(booked, is: 1, children: [ P(\"Ta place est gardée\") ]),\n    Button(name: Like, text: \"J'aime ({likes})\"),\n  ],\n  rules: [ On(Book.tap, effect: [seats.sub(1), booked.set(1)]), On(Like.tap, effect: likes.add(1)) ],\n)\n";

    fn json<'a>(url: &'a str, cookie: &'a str, body: &'a [u8]) -> Ask<'a> {
        let mut asked = ask("POST", url, cookie, body);
        asked.content_type = "application/json";
        asked
    }

    fn cookie_of(reply: &Reply) -> String {
        reply.headers.iter().find(|(n, _)| n == "Set-Cookie").map(|(_, v)| v.split(';').next().unwrap().to_string()).unwrap_or_default()
    }

    #[test]
    fn shared_values_are_kept_for_everyone_with_and_without_javascript() {
        // ADR-079 : le serveur garde les places pour tout le monde, arbitre chaque geste, chacun son
        // tour ; un bouton caché ne se touche pas ; chaque adresse a ses valeurs.
        let (site, folder) = site();
        std::fs::write(folder.join("concert.holo"), CONCERT).unwrap();
        let first = String::from_utf8(site.answer(&ask("GET", "/concert.holo", "", b"")).body).unwrap();
        assert!(first.contains("<span data-state=\"seats\">2</span> place(s)") && first.contains(" data-shared=\"seats=2;likes=0\"") && first.contains(" data-live"), "{first}");
        // Sans JavaScript, Ada réserve : la page revient avec sa place, et tout le monde voit une place de moins.
        let ada = site.answer(&ask("POST", "/concert.holo", "", b"signal=Book.tap"));
        assert_eq!(ada.status, 303);
        let ada = cookie_of(&ada);
        let seen = String::from_utf8(site.answer(&ask("GET", "/concert.holo", &ada, b"")).body).unwrap();
        assert!(seen.contains("<span data-state=\"seats\">1</span> place(s)") && seen.contains("data-if=\"booked|is=1\">"), "{seen}");
        let other = String::from_utf8(site.answer(&ask("GET", "/concert.holo", "", b"")).body).unwrap();
        assert!(other.contains("<span data-state=\"seats\">1</span> place(s)") && other.contains("data-if=\"booked|is=1\" hidden>"), "{other}");
        // Avec JavaScript, Grace envoie son geste et un état forgé (99 places) : le serveur garde les siennes.
        let grace = site.answer(&json("/concert.holo", "", br#"{"signal":"Book.tap","state":"booked=0;seats=99;likes=0"}"#));
        assert_eq!(grace.status, 200);
        assert_eq!(String::from_utf8(grace.body).unwrap(), r#"{"accepted":true,"state":"booked=1;seats=0;likes=0","shared":"seats=0;likes=0","version":2}"#);
        // Hedy arrive trop tard : le bouton est caché pour le serveur, le geste est refusé (409).
        let hedy = site.answer(&json("/concert.holo", "", br#"{"signal":"Book.tap","state":""}"#));
        assert_eq!(hedy.status, 409);
        assert!(String::from_utf8(hedy.body).unwrap().starts_with(r#"{"accepted":false,"state":"booked=0;seats=0;likes=0""#));
        // Sans JavaScript non plus : la page revient telle quelle.
        let late = site.answer(&ask("POST", "/concert.holo", "", b"signal=Book.tap"));
        let late = String::from_utf8(site.answer(&ask("GET", "/concert.holo", &cookie_of(&late), b"")).body).unwrap();
        assert!(late.contains("<span data-state=\"seats\">0</span> place(s)") && late.contains("data-if=\"booked|is=1\" hidden>"), "{late}");
        // Un geste qui n'est pas un toucher, ou qui ne change rien de partagé : refusé.
        assert_eq!(site.answer(&json("/concert.holo", "", br#"{"signal":"Like.hover","state":""}"#)).status, 400);
        assert_eq!(site.answer(&json("/concert.holo", "", br#"{"signal":"Nobody.tap","state":""}"#)).status, 400);
        assert_eq!(site.answer(&json("/shop.holo", "", br#"{"signal":"Add.tap","state":""}"#)).status, 400);
        // Une demande trop lourde.
        let heavy = format!(r#"{{"signal":"Like.tap","state":"{}"}}"#, "a".repeat(BODY_MAX as usize));
        assert_eq!(site.answer(&json("/concert.holo", "", heavy.as_bytes())).status, 413);
        // Chaque adresse a ses valeurs : /salle/12 et /salle/13.
        std::fs::create_dir_all(folder.join("salle")).unwrap();
        std::fs::write(folder.join("salle").join("{id}.holo"), "Page(title: \"Salle\", shared: Shared(likes: 0), children: [ H1(\"Salle {id}\"), P(\"{likes} j'aime\"), Button(name: Like, text: \"J'aime\") ], rules: [ On(Like.tap, effect: likes.add(1)) ])").unwrap();
        for url in ["/salle/12", "/salle/12", "/salle/13"] {
            assert_eq!(site.answer(&json(url, "", br#"{"signal":"Like.tap","state":""}"#)).status, 200);
        }
        let twelve = String::from_utf8(site.answer(&ask("GET", "/salle/12", "", b"")).body).unwrap();
        let thirteen = String::from_utf8(site.answer(&ask("GET", "/salle/13", "", b"")).body).unwrap();
        assert!(twelve.contains("<span data-state=\"likes\">2</span> j'aime") && thirteen.contains("<span data-state=\"likes\">1</span> j'aime"), "{twelve}\n{thirteen}");
        // Dans la base : l'adresse, le nom, la valeur ; et un nouveau serveur s'en souvient.
        let rows: Vec<String> = {
            let base = site.base.lock().unwrap();
            let mut query = base.prepare("SELECT page, name, value, version FROM shared ORDER BY page, name").unwrap();
            query.query_map([], |row| Ok(format!("{} {}={} ({})", row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?))).unwrap().flatten().collect()
        };
        assert_eq!(rows, ["/concert.holo likes=0 (2)", "/concert.holo seats=0 (2)", "/salle/12 likes=2 (2)", "/salle/13 likes=1 (1)"]);
        let web = Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
        let reopened = Site::open(&folder, &web).unwrap();
        assert!(String::from_utf8(reopened.answer(&ask("GET", "/concert.holo", "", b"")).body).unwrap().contains("<span data-state=\"seats\">0</span> place(s)"));
        drop(reopened);
        let _ = std::fs::remove_dir_all(folder);
    }


    const MEMBER_CONCERT: &str = include_str!("../../proposals/GPT5.6/reprise-pc-comptes-partage-2026-10-08/concert.holo");

    fn concert_member(site: &Site, name: &str) -> String {
        let body = format!("name={name}&password=une+phrase+assez+longue&again=une+phrase+assez+longue");
        let reply = site.answer(&ask("POST", "/account/signup", "", body.as_bytes()));
        assert_eq!(reply.status, 303, "{}", String::from_utf8_lossy(&reply.body));
        let cookie = cookie_of(&reply);
        assert!(cookie.starts_with("holo_session="), "{cookie}");
        cookie
    }

    fn member_state(site: &Site, cookie: &str) -> String {
        let member = crate::accounts::member_of(site, cookie).unwrap();
        site.stored(&member.visit_key(), "/member-concert.holo").unwrap().state
    }

    #[test]
    fn a_member_cannot_reserve_twice_by_forging_the_json_state() {
        let (site, folder) = site();
        crate::check_page(MEMBER_CONCERT).unwrap();
        std::fs::write(folder.join("member-concert.holo"), MEMBER_CONCERT).unwrap();
        let ada = concert_member(&site, "Ada");
        assert_eq!(site.answer(&ask("POST", "/member-concert.holo", &ada, b"signal=Book.tap")).status, 303);
        let saved = member_state(&site, &ada);
        let forged = br#"{"signal":"Book.tap","state":"booked=0;cart=777;seats=999;likes=0"}"#;
        let refused = site.answer(&json("/member-concert.holo", &ada, forged));
        assert_eq!(refused.status, 409, "{}", String::from_utf8_lossy(&refused.body));
        assert_eq!(member_state(&site, &ada), saved);
        assert!(String::from_utf8(refused.body).unwrap().contains("\"shared\":\"seats=2;likes=0;last='\""));
        // Un geste accepté ne reprend pas davantage cart=777.
        assert_eq!(site.answer(&json("/member-concert.holo", &ada, br#"{"signal":"Like.tap","state":"booked=0;cart=777;likes=999"}"#)).status, 200);
        assert!(member_state(&site, &ada).contains("booked=1;cart=0"), "{}", member_state(&site, &ada));
        let grace = concert_member(&site, "Grace");
        assert_eq!(site.answer(&json("/member-concert.holo", &grace, forged)).status, 200);
        assert!(member_state(&site, &grace).contains("booked=1;cart=0"), "{}", member_state(&site, &grace));
        drop(site);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn a_refused_member_gesture_saves_neither_forged_state_nor_inputs() {
        let (site, folder) = site();
        std::fs::write(folder.join("member-concert.holo"), MEMBER_CONCERT).unwrap();
        let ada = concert_member(&site, "Ada");
        site.answer(&ask("POST", "/member-concert.holo", &ada, b"note=Ada&signal=Book.tap"));
        let saved = member_state(&site, &ada);
        let body = br#"{"signal":"Book.tap","state":"booked=1;cart=777;note='Eve;price=99999"}"#;
        let refused = site.answer(&json("/member-concert.holo", &ada, body));
        assert_eq!(refused.status, 409);
        assert_eq!(member_state(&site, &ada), saved);
        assert!(String::from_utf8(refused.body).unwrap().contains(&crate::json_text(&saved)));
        drop(site);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn member_json_inputs_use_the_normal_validation_and_decimal_scale() {
        let (site, folder) = site();
        std::fs::write(folder.join("member-concert.holo"), MEMBER_CONCERT).unwrap();
        let ada = concert_member(&site, "Ada");
        // Les champs restent utilisables sans attendre leur miroir. Le texte est nettoyé et
        // borné, les décimales gardent leur échelle, la glissière et le choix gardent leurs bornes.
        let body = br#"{"signal":"Book.tap","state":"booked=0;cart=777;note='Ad%C3%A8leOK;price=1350;agreed=1;level=999;size='XL;seats=999;last='Eve"}"#;
        let reply = site.answer(&json("/member-concert.holo", &ada, body));
        assert_eq!(reply.status, 200, "{}", String::from_utf8_lossy(&reply.body));
        let saved = member_state(&site, &ada);
        for chunk in ["booked=1", "cart=0", "price=1350", "agreed=1", "level=10", "size='S", "note='Ad%C3%A8le", "last='Ad%C3%A8le", "seats=2"] {
            assert!(saved.split(';').any(|part| part == chunk), "{chunk}: {saved}");
        }
        assert!(inputs_from_state(MEMBER_CONCERT, "cart=777;booked=0;price=oops;note='%ZZ;last='Eve").is_empty());
        assert!(inputs_from_state(MEMBER_CONCERT, "").is_empty());
        drop(site);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn a_refused_guest_json_gesture_does_not_create_a_saved_visit() {
        let (site, folder) = site();
        std::fs::write(folder.join("concert.holo"), CONCERT).unwrap();
        let reply = site.answer(&json("/concert.holo", "", br#"{"signal":"Book.tap","state":"booked=1;seats=999"}"#));
        assert_eq!(reply.status, 409);
        let visitor = visitor_of(&cookie_of(&reply)).unwrap();
        assert!(site.stored(&visitor, "/concert.holo").is_none());
        drop(site);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn concurrent_form_and_json_taps_of_one_account_reserve_one_seat() {
        let (site, folder) = site();
        std::fs::write(folder.join("member-concert.holo"), MEMBER_CONCERT).unwrap();
        let ada = concert_member(&site, "Ada");
        let barrier = std::sync::Barrier::new(8);
        std::thread::scope(|scope| {
            for rank in 0..8 {
                let barrier = &barrier;
                let site = &site;
                let ada = &ada;
                scope.spawn(move || {
                    barrier.wait();
                    if rank % 2 == 0 {
                        assert_eq!(site.answer(&ask("POST", "/member-concert.holo", ada, b"signal=Book.tap")).status, 303);
                    } else {
                        let status = site.answer(&json("/member-concert.holo", ada, br#"{"signal":"Book.tap","state":"booked=0;seats=999"}"#)).status;
                        assert!(status == 200 || status == 409, "{status}");
                    }
                });
            }
        });
        let base = site.base.lock().unwrap();
        assert_eq!(shared_in(&base, "/member-concert.holo"), ("seats=2;likes=0;last='".into(), 1));
        drop(base);
        assert!(member_state(&site, &ada).contains("booked=1;cart=0"));
        drop(site);
        std::fs::remove_dir_all(folder).unwrap();
    }

    /// Ce qu'une page en direct reçoit, pour les essais.
    #[derive(Clone, Default)]
    struct Recorder(Arc<Mutex<Vec<u8>>>);

    impl Write for Recorder {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Recorder {
        fn text(&self) -> String {
            String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
        }
        fn waits_for(&self, expected: &str) -> bool {
            (0..300).any(|_| {
                std::thread::sleep(std::time::Duration::from_millis(10));
                self.text().contains(expected)
            })
        }
    }

    /// Une page fermée : rien ne passe plus.
    struct Closed;

    impl Write for Closed {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::ConnectionReset, "page fermée"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_page_listens_live_and_a_closed_one_is_forgotten() {
        let (site, folder) = site();
        std::fs::write(folder.join("concert.holo"), CONCERT).unwrap();
        // Seule une page qui partage des valeurs s'écoute ; pas depuis un autre site.
        let listen_ask = |url: &'static str| Ask { method: "GET", url, accept: "text/event-stream", cookie: "", content_type: "", origin: "", host: "localhost:8080", referer: "", peer: "127.0.0.1", forwarded: "", body: b"" };
        assert_eq!(site.live_page(&listen_ask("/shop.holo")).err().map(|r| r.status), Some(404));
        let mut foreign = listen_ask("/concert.holo");
        foreign.origin = "https://ailleurs.example";
        assert_eq!(site.live_page(&foreign).err().map(|r| r.status), Some(403));
        let (key, source) = site.live_page(&listen_ask("/concert.holo")).ok().unwrap();
        assert_eq!(key, "/concert.holo");
        // La page reçoit d'abord les valeurs du moment, puis chaque changement.
        let page = Recorder::default();
        site.listen(&key, &source, Box::new(page.clone()));
        assert!(page.waits_for("event: shared\nid: 0\ndata: seats=2;likes=0\n\n"), "{}", page.text());
        assert!(page.text().starts_with("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream; charset=utf-8\r\n"), "{}", page.text());
        assert_eq!(site.answer(&json("/concert.holo", "", br#"{"signal":"Like.tap","state":""}"#)).status, 200);
        assert!(page.waits_for("id: 1\ndata: seats=2;likes=1\n\n"), "{}", page.text());
        // Huit visiteurs touchent « J'aime » en même temps : chacun son tour, aucun n'est perdu, et
        // la page reçoit les changements dans l'ordre.
        std::thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| assert_eq!(site.answer(&json("/concert.holo", "", br#"{"signal":"Like.tap","state":""}"#)).status, 200));
            }
        });
        assert!(page.waits_for("id: 9\ndata: seats=2;likes=9\n\n"), "{}", page.text());
        let seen = page.text();
        let places: Vec<usize> = (1..=9).map(|n| seen.find(&format!("id: {n}\ndata: seats=2;likes={n}\n")).unwrap_or(usize::MAX)).collect();
        assert!(places.windows(2).all(|w| w[0] < w[1]), "{seen}");
        // Sans JavaScript aussi, le changement part en direct.
        site.answer(&ask("POST", "/concert.holo", "", b"signal=Book.tap"));
        assert!(page.waits_for("id: 10\ndata: seats=1;likes=9\n\n"), "{}", page.text());
        // Une page fermée est oubliée ; celle qui reste écoute encore.
        site.listen(&key, &source, Box::new(Closed));
        let forgotten = (0..300).any(|_| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            site.lives.lock().unwrap().pages.len() == 1
        });
        assert!(forgotten, "{} page(s)", site.lives.lock().unwrap().pages.len());
        // Au-delà de 128 pages en direct, la suivante est refusée.
        let mut kept = Vec::new();
        {
            let mut lives = site.lives.lock().unwrap();
            while lives.pages.len() < LIVE_MAX {
                let (sender, receiver) = std::sync::mpsc::sync_channel(1);
                kept.push(receiver);
                let id = lives.next;
                lives.next += 1;
                lives.pages.push(Live { id, key: "/ailleurs.holo".into(), sender });
            }
        }
        let refused = Recorder::default();
        site.listen(&key, &source, Box::new(refused.clone()));
        assert!(refused.text().starts_with("HTTP/1.1 503 "), "{}", refused.text());
        drop(kept);
        let _ = std::fs::remove_dir_all(folder);
    }
}

/// Le visiteur réel, pour les freins (ADR-080, ADR-083) : l'adresse du pair TCP. Derrière le
/// proxy HTTPS de l'auteur (`HOLO_ORIGIN` fixé, et la demande vient de ce PC : c'est le proxy),
/// tous les visiteurs arriveraient de 127.0.0.1 et partageraient un seul frein ; c'est alors la
/// dernière adresse de `X-Forwarded-For`, celle qu'a écrite ce proxy. Une adresse écrite par un
/// visiteur qui parle directement au serveur n'est jamais crue ; une adresse illisible non plus.
pub(crate) fn client_address(site: &Site, ask: &Ask) -> String {
    forwarded_client(ask.peer, ask.forwarded, site.passkeys_origin.is_some())
}

fn forwarded_client(peer: &str, forwarded: &str, behind_proxy: bool) -> String {
    let Ok(peer) = peer.parse::<std::net::IpAddr>() else { return peer.to_string() };
    if !behind_proxy || !peer.is_loopback() {
        return peer.to_string();
    }
    forwarded.rsplit(',').next().and_then(|last| last.trim().parse::<std::net::IpAddr>().ok()).unwrap_or(peer).to_string()
}

/// Un visiteur : soixante touchers par adresse et minute. Une IP : cent quatre-vingts,
// afin qu'effacer le cookie ne suffise pas. Aucune adresse fournie dans un en-tête n'est crue,
// sauf celle qu'écrit le proxy de l'auteur (client_address).
fn shared_allowed(base:&Connection,visitor:&str,peer:&str,page:&str,now:u64)->bool{
 if peer.parse::<std::net::IpAddr>().is_err(){return false;}
 if base.execute("DELETE FROM shared_limits WHERE started<=?1",params![now.saturating_sub(60)as i64]).is_err(){return false;}
 if base.query_row("SELECT COUNT(*) FROM shared_limits",[],|r|r.get::<_,i64>(0)).unwrap_or(50_000)>=50_000{return false;}
 let keys=[(format!("visitor:{visitor}:{page}"),60),(format!("ip:{peer}"),180)];
 for (key,max) in &keys{
  let count:i64=base.query_row("SELECT touches FROM shared_limits WHERE key=?1",params![key],|r|r.get(0)).optional().ok().flatten().unwrap_or(0);
  if count>=*max{return false;}
 }
 for(key,_)in keys{if base.execute("INSERT INTO shared_limits(key,started,touches)VALUES(?1,?2,1) ON CONFLICT(key)DO UPDATE SET touches=touches+1",params![key,now as i64]).is_err(){return false;}}
 true
}
fn shared_limited(visitor:Option<&str>)->Reply{
 let mut h=common_headers();h.push(("Content-Type".into(),"text/plain; charset=utf-8".into()));h.push(("Retry-After".into(),"60".into()));
 if let Some(v)=visitor{h.push(("Set-Cookie".into(),format!("{COOKIE}={v}; Path=/; HttpOnly; SameSite=Lax; Max-Age={FORGET_AFTER}")));}
 Reply{status:429,headers:h,body:"Trop de touchers : attends une minute. Rien n’a changé.".as_bytes().to_vec()}
}

#[cfg(test)]
mod sharing_completion_tests{
 use super::*;
 #[test]fn a_visitors_brake_is_also_bound_to_the_real_ip(){
  let base=Connection::open_in_memory().unwrap();base.execute_batch("CREATE TABLE shared_limits(key TEXT PRIMARY KEY,started INTEGER,touches INTEGER)").unwrap();
  for _ in 0..60{assert!(shared_allowed(&base,"visitor","127.0.0.1","/book",1000));}
  assert!(!shared_allowed(&base,"visitor","127.0.0.1","/book",1001));
  assert!(shared_allowed(&base,"second","127.0.0.1","/book",1001));
  assert!(shared_allowed(&base,"visitor","127.0.0.1","/other",1001));
  assert!(shared_allowed(&base,"visitor","127.0.0.1","/book",1060));
  assert!(!shared_allowed(&base,"third","forged","/book",1060));
 }
 #[test]
 fn the_real_client_is_read_only_behind_the_authors_proxy() {
  // Sans proxy déclaré (HOLO_ORIGIN absent), X-Forwarded-For n'est jamais cru.
  assert_eq!(forwarded_client("127.0.0.1", "203.0.113.7", false), "127.0.0.1");
  // Derrière le proxy de l'auteur, la demande vient de ce PC : la dernière adresse est celle
  // que le proxy a écrite ; celles d'avant viennent du visiteur.
  assert_eq!(forwarded_client("127.0.0.1", "198.51.100.1, 203.0.113.7", true), "203.0.113.7");
  assert_eq!(forwarded_client("::1", "2001:db8::5", true), "2001:db8::5");
  // Un visiteur du Wi-Fi qui parle directement au serveur : son adresse, jamais celle qu'il écrit.
  assert_eq!(forwarded_client("192.168.1.20", "203.0.113.7", true), "192.168.1.20");
  // Rien de lisible : le pair, donc un frein commun, jamais aucun frein.
  for unreadable in ["", "unknown", "203.0.113.7:4000", "203.0.113.7, x"] {
   assert_eq!(forwarded_client("127.0.0.1", unreadable, true), "127.0.0.1", "{unreadable}");
  }
 }
}
