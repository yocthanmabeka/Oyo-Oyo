//! Le premier vrai serveur de HoloCode : `holo serve` (ADR-074). Un programme en Rust, sur le
//! PC de l'auteur, sans rien d'autre à installer : ni Node.js, ni base de données à part, ni
//! service extérieur (chez soi d'abord).
//!
//! Il sert un dossier : ses pages `.holo` fabriquées d'avance, ses images et ses fichiers, et le
//! moteur pour le navigateur. Il fait tourner le même arbitre que le navigateur : un visiteur
//! sans JavaScript touche un bouton, le formulaire des gestes part, le serveur calcule le nouvel
//! état, le range dans SQLite sous le numéro du visiteur, et renvoie la page à jour.
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
             );",
        )
        .map_err(|e| e.to_string())?;
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
            let state = visitor_of(ask.cookie).and_then(|visitor| self.stored(&visitor, path));
            return match self.page(&file, path, state) {
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
        if !ask.content_type.starts_with("application/x-www-form-urlencoded") {
            return Reply::text(415, "un formulaire `Form` part encore par outils/server.mjs ; holo serve ne reçoit que les gestes");
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
        let before = self.stored(&visitor, path).unwrap_or_else(|| starting_state(&source, &file));
        let fields = crate::gestures::read_form(&String::from_utf8_lossy(ask.body));
        let after = crate::visitor_gesture(&source, &before, &fields);
        if after.len() <= STATE_MAX {
            self.store(&visitor, path, &after);
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
    fn page(&self, file: &Path, path: &str, state: Option<String>) -> Result<String, String> {
        let source = read_with_imports(file).map_err(|e| e.to_string())?;
        if source.lines().map(|line| line.split("//").next().unwrap_or("").trim()).find(|line| !line.is_empty()).is_some_and(|line| line.starts_with("Point")) {
            return std::fs::read_to_string(self.web.join("index.html")).map_err(|e| e.to_string());
        }
        let template = std::fs::read_to_string(self.web.join("page.html")).map_err(|e| format!("page d'entrée du moteur introuvable : {e}"))?;
        set_clock();
        let start = state.unwrap_or_else(|| starting_state(&source, file));
        let base = &path[..=path.rfind('/').unwrap_or(0)];
        // Un fichier refusé : la page d'entrée seule, qui affichera l'erreur du moteur.
        let Ok(html) = crate::visitor_page(&source, base, &start) else { return Ok(template) };
        Ok(filled_template(&template, &html))
    }

    fn stored(&self, visitor: &str, page: &str) -> Option<String> {
        let base = self.base.lock().ok()?;
        base.query_row("SELECT state FROM visits WHERE visitor = ?1 AND page = ?2", params![visitor, page], |row| row.get(0)).optional().ok().flatten()
    }

    fn store(&self, visitor: &str, page: &str, state: &str) {
        if let Ok(base) = self.base.lock() {
            let _ = base.execute(
                "INSERT INTO visits (visitor, page, state, updated) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (visitor, page) DO UPDATE SET state = excluded.state, updated = excluded.updated",
                params![visitor, page, state, now() as i64],
            );
        }
    }
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
        let _ = request.as_reader().take(BODY_MAX + 1).read_to_end(&mut body);
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
    state.split(';').filter(|chunk| !chunk.starts_with("!=")).collect::<Vec<_>>().join(";")
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
        // Un formulaire `Form` en JSON, un corps trop lourd, une page absente, une autre méthode.
        let mut json = ask("POST", "/shop.holo", "", b"{}");
        json.content_type = "application/json";
        assert_eq!(site.answer(&json).status, 415);
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
