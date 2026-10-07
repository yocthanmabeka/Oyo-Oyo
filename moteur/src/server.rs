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
//! Les valeurs de chaque visiteur sont à lui seul : rien n'est encore partagé entre visiteurs
//! (ce sera le lot 6), et personne n'a de compte (le lot 7).

use std::io::Read;
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

/// Ce que le serveur garde d'un visiteur sur une page : ses valeurs, et les formulaires qu'il a
/// essayé d'envoyer sans y arriver (leurs messages d'erreur suivent ce qu'il corrige).
#[derive(Default)]
struct Visit {
    state: String,
    tried: Vec<String>,
}

/// Ce que le serveur sait de son site.
pub struct Site {
    /// Le dossier servi : les pages, les images, les fichiers.
    folder: PathBuf,
    /// Le dossier du moteur pour le navigateur (`moteur/web`) : la page d'entrée, le moteur.
    web: PathBuf,
    /// La base : un seul fichier, `holo-data/site.sqlite`.
    base: Mutex<Connection>,
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
             );",
        )
        .map_err(|e| e.to_string())?;
        // Une base faite avant les formulaires (ADR-075) reçoit leur colonne.
        let _ = base.execute("ALTER TABLE visits ADD COLUMN tried TEXT NOT NULL DEFAULT ''", []);
        base.execute("DELETE FROM visits WHERE updated < ?1", params![now().saturating_sub(FORGET_AFTER) as i64]).map_err(|e| e.to_string())?;
        Ok(Site { folder, web: web.to_path_buf(), base: Mutex::new(base) })
    }

    /// Répond à une demande. Tout passe par ici : c'est ce que les essais éprouvent.
    pub fn answer(&self, ask: &Ask) -> Reply {
        let Some(path) = url_path(ask.url) else { return Reply::text(400, "adresse illisible") };
        let path = if path == "/" { "/index.holo".to_string() } else { path };
        match ask.method {
            "GET" | "HEAD" => self.get(ask, &path),
            "POST" => self.gesture(ask, &path),
            _ => Reply::text(405, "seuls GET et POST sont reçus"),
        }
    }

    fn get(&self, ask: &Ask, path: &str) -> Reply {
        let Some(file) = self.find(path) else { return Reply::text(404, "introuvable") };
        // Un .holo demandé pour être affiché : sa page, fabriquée pour ce visiteur. Demandé par le
        // moteur (`text/plain`), le fichier lui-même.
        if path.ends_with(".holo") && ask.accept.contains("text/html") {
            let visit = visitor_of(ask.cookie).and_then(|visitor| self.stored(&visitor, path));
            return match self.page(&file, path, visit) {
                Ok(html) => {
                    let mut reply = Reply { status: 200, headers: vec![("Content-Type".into(), "text/html; charset=utf-8".into())], body: html.into_bytes() };
                    reply.headers.extend(common_headers());
                    reply
                }
                Err(message) => Reply::text(500, &message),
            };
        }
        match std::fs::read(&file) {
            Ok(bytes) => {
                let mut headers = vec![("Content-Type".to_string(), content_type(&file).to_string())];
                headers.extend(common_headers());
                Reply { status: 200, headers, body: bytes }
            }
            Err(_) => Reply::text(404, "introuvable"),
        }
    }

    /// Un toucher envoyé sans JavaScript (ADR-074) : le même arbitre, puis la page à jour par
    /// une nouvelle demande (`303`), pour qu'un rechargement ne rejoue pas le geste.
    fn gesture(&self, ask: &Ask, path: &str) -> Reply {
        if !path.ends_with(".holo") {
            return Reply::text(405, "seule une page .holo reçoit des gestes");
        }
        let Some(file) = self.find(path) else { return Reply::text(404, "page introuvable") };
        // Une page d'un autre site ne fait pas toucher les boutons de celle-ci.
        if !ask.origin.is_empty() && ask.origin.split("://").nth(1) != Some(ask.host) {
            return Reply::text(403, "ce geste vient d'un autre site");
        }
        // Un formulaire envoyé par le moteur, en JSON, avec ou sans fichiers (ADR-075).
        if ask.content_type.starts_with("application/json") || ask.content_type.starts_with("multipart/form-data") {
            return self.message(ask, path, &file);
        }
        if !ask.content_type.starts_with("application/x-www-form-urlencoded") {
            return Reply::text(415, "un geste, ou un formulaire en JSON");
        }
        if ask.body.len() as u64 > BODY_MAX {
            return Reply::text(413, "formulaire trop lourd");
        }
        let Ok(source) = read_with_imports(&file) else { return Reply::text(404, "page introuvable") };
        set_clock();
        let (visitor, new_visitor) = match visitor_of(ask.cookie) {
            Some(visitor) => (visitor, false),
            None => (new_visitor(), true),
        };
        let mut visit = self.stored(&visitor, path).unwrap_or_else(|| Visit { state: starting_state(&source, &file), tried: Vec::new() });
        let fields = crate::gestures::read_form(&String::from_utf8_lossy(ask.body));
        visit.state = crate::visitor_gesture(&source, &visit.state, &fields);
        // Un toucher qui envoie un formulaire (`On(Send.tap, effect: Contact.send)`) : le serveur
        // vérifie, range le message, puis `Contact.sent` ; ou garde les messages d'erreur.
        let signal = fields.iter().find(|(name, signal)| name == crate::gestures::SIGNAL && crate::gestures::is_tap(signal)).map(|(_, signal)| signal.as_str()).unwrap_or("");
        for form in crate::effects(&source, signal).iter().filter_map(|effect| effect.strip_suffix(".send")) {
            visit.tried.retain(|tried| tried != form);
            if !crate::form_errors(&source, &visit.state, form).is_empty() {
                visit.tried.push(form.to_string());
                continue;
            }
            let submission = crate::submission(&source, &visit.state, form);
            let outcome = if self.keep_message(path, form, &submission, "[]").is_ok() { "sent" } else { "failed" };
            visit.state = without_sounds(&crate::arbitrate(&source, &visit.state, &format!("{form}.{outcome}")));
        }
        if visit.state.len() <= STATE_MAX {
            self.store(&visitor, path, &visit);
        }
        let mut headers = vec![("Location".to_string(), path.to_string())];
        if new_visitor {
            headers.push(("Set-Cookie".into(), format!("{COOKIE}={visitor}; Path=/; HttpOnly; SameSite=Lax; Max-Age={FORGET_AFTER}")));
        }
        headers.extend(common_headers());
        Reply { status: 303, headers, body: Vec::new() }
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
    fn page(&self, file: &Path, path: &str, visit: Option<Visit>) -> Result<String, String> {
        let source = read_with_imports(file).map_err(|e| e.to_string())?;
        if source.lines().map(|line| line.split("//").next().unwrap_or("").trim()).find(|line| !line.is_empty()).is_some_and(|line| line.starts_with("Point")) {
            return std::fs::read_to_string(self.web.join("index.html")).map_err(|e| e.to_string());
        }
        let template = std::fs::read_to_string(self.web.join("page.html")).map_err(|e| format!("page d'entrée du moteur introuvable : {e}"))?;
        set_clock();
        let visit = visit.unwrap_or_else(|| Visit { state: starting_state(&source, file), tried: Vec::new() });
        let base = &path[..=path.rfind('/').unwrap_or(0)];
        // Un fichier refusé : la page d'entrée seule, qui affichera l'erreur du moteur.
        let Ok(html) = crate::visitor_page(&source, base, &visit.state, &visit.tried) else { return Ok(template) };
        Ok(filled_template(&template, &html))
    }

    fn stored(&self, visitor: &str, page: &str) -> Option<Visit> {
        let base = self.base.lock().ok()?;
        base.query_row("SELECT state, tried FROM visits WHERE visitor = ?1 AND page = ?2", params![visitor, page], |row| {
            let tried: String = row.get(1)?;
            Ok(Visit { state: row.get(0)?, tried: tried.split(',').filter(|t| !t.is_empty()).map(str::to_string).collect() })
        })
        .optional()
        .ok()
        .flatten()
    }

    fn store(&self, visitor: &str, page: &str, visit: &Visit) {
        if let Ok(base) = self.base.lock() {
            let _ = base.execute(
                "INSERT INTO visits (visitor, page, state, tried, updated) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (visitor, page) DO UPDATE SET state = excluded.state, tried = excluded.tried, updated = excluded.updated",
                params![visitor, page, visit.state, visit.tried.join(","), now() as i64],
            );
        }
    }

    /// Un formulaire envoyé par le moteur du navigateur (ADR-042, ADR-059), reçu ici (ADR-075) :
    /// vérifié à nouveau par le moteur (la page peut être contournée, le serveur non), ses
    /// fichiers reconnus à leurs premiers octets, puis rangé dans la base. `204`, ou le refus.
    fn message(&self, ask: &Ask, path: &str, file: &Path) -> Reply {
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
        let Ok(source) = read_with_imports(file) else { return Reply::text(404, "page introuvable") };
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
        match self.keep_message(path, form, &json, &format!("[{}]", kept.join(","))) {
            Ok(()) => Reply { status: 204, headers: common_headers(), body: Vec::new() },
            Err(reply) => reply,
        }
    }

    /// Range un message dans la base, sauf si la page en garde déjà trop.
    fn keep_message(&self, path: &str, form: &str, submission: &str, files: &str) -> Result<(), Reply> {
        let base = self.base.lock().map_err(|_| Reply::text(500, "base indisponible"))?;
        let already: i64 = base.query_row("SELECT COALESCE(SUM(LENGTH(submission)), 0) FROM messages WHERE page = ?1", params![path], |row| row.get(0)).map_err(|_| Reply::text(500, "base illisible"))?;
        if already > MESSAGES_PER_PAGE_MAX {
            return Err(Reply::text(507, "trop de messages gardés pour cette page"));
        }
        base.execute("INSERT INTO messages (received, page, form, submission, files) VALUES (?1, ?2, ?3, ?4, ?5)", params![now() as i64, path, form, submission, files])
            .map_err(|_| Reply::text(500, "message impossible à ranger"))?;
        println!("Message reçu : {path} ({form})");
        Ok(())
    }
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
    let (accept, cookie, content_type, origin, host) = (header("Accept"), header("Cookie"), header("Content-Type"), header("Origin"), header("Host"));
    let method = request.method().as_str().to_string();
    let url = request.url().to_string();
    let mut body = Vec::new();
    if method == "POST" {
        let limit = if content_type.starts_with("multipart/form-data") { WITH_FILES_MAX } else { BODY_MAX };
        let _ = request.as_reader().take(limit + 1).read_to_end(&mut body);
    }
    let reply = site.answer(&Ask { method: &method, url: &url, accept: &accept, cookie: &cookie, content_type: &content_type, origin: &origin, host: &host, body: &body });
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
fn visitor_of(cookie: &str) -> Option<String> {
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

fn now() -> u64 {
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
        Ask { method, url, accept: "text/html", cookie, content_type: "application/x-www-form-urlencoded", origin: "", host: "localhost:8080", body }
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
}
