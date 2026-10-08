//! Les comptes de `holo serve` (ADR-081, lot 7 du web) : créer un compte, se connecter avec un mot
//! de passe et, si on l'a activé, un code à 6 chiffres, se déconnecter. Tout est gardé par le
//! serveur de l'auteur, dans la base du site (`holo-data/site.sqlite`), comme avec Django : aucun
//! prestataire (Google, Apple, Microsoft, un service d'e-mails) n'est jamais nécessaire.
//!
//! - **La cryptographie ne s'écrit jamais à la main.** Les mots de passe ne sont gardés que par
//!   leur empreinte Argon2id (bibliothèque `argon2` de RustCrypto) : salée, lente à calculer exprès,
//!   impossible à défaire. Le code à 6 chiffres suit la RFC 6238 (TOTP) : HMAC-SHA-1 (bibliothèques
//!   `hmac` et `sha1` de RustCrypto) d'une clé secrète et de l'heure, par pas de 30 secondes.
//!   L'application d'authentification du visiteur (Aegis, FreeOTP, Google Authenticator…) fait le
//!   même calcul, sans Internet ni SMS. Un code ne sert qu'une fois.
//! - Une session est un numéro de 128 bits tiré au hasard par le système, dans un cookie
//!   `HttpOnly; SameSite=Lax`. La base n'en garde que l'empreinte : une copie de la base (une
//!   sauvegarde) ne permet pas d'entrer. Elle est oubliée après 14 jours sans visite, 30 au plus.
//! - Un frein contre les essais répétés : cinq essais, puis une minute d'attente, qui double à
//!   chaque nouvel échec (une heure au plus). Le message ne dit jamais si c'est le nom ou le mot de
//!   passe qui est faux, et le serveur prend le même temps dans les deux cas.
//! - Les pages (créer un compte, se connecter, le code, le compte) sont fabriquées ici, en HTML
//!   ordinaire, sans JavaScript : elles marchent de la même façon sur un téléphone et sur un
//!   ordinateur, avec ou sans lui. Elles sont accessibles : chaque champ a son étiquette, ses
//!   explications et son erreur reliées (`aria-describedby`), l'erreur est lue par un lecteur
//!   d'écran (`role="alert"`), et le navigateur sait quel mot de passe proposer (`autocomplete`).

use std::sync::OnceLock;

use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::Argon2;
use hmac::{Hmac, KeyInit, Mac};
use rusqlite::{params, Connection, OptionalExtension};
use sha1::{Digest, Sha1};

use crate::flat::escape;
use crate::server::{Ask, Reply, Site};

/// Le nom du cookie qui porte le numéro de session.
pub const SESSION_COOKIE: &str = "holo_session";
/// Un mot de passe : 12 caractères au moins (une phrase est un bon mot de passe), 128 au plus.
pub const PASSWORD_MIN: usize = 12;
const PASSWORD_MAX: usize = 128;
/// Une session est oubliée après 14 jours sans visite, et 30 jours après la connexion au plus.
pub const SESSION_IDLE: u64 = 14 * 24 * 3600;
pub const SESSION_MAX: u64 = 30 * 24 * 3600;
/// Entre le mot de passe et le code, cinq minutes au plus.
const PENDING_MAX: u64 = 5 * 60;
/// Cinq essais ratés, puis une minute d'attente, qui double à chaque nouvel échec, une heure au plus.
pub const FREE_TRIES: i64 = 5;
const WAIT_FIRST: u64 = 60;
const WAIT_MAX: u64 = 3600;
/// Le nombre de comptes d'un site, au plus.
const ACCOUNTS_MAX: i64 = 10_000;
/// Le code change toutes les 30 secondes ; il a 6 chiffres ; sa clé fait 20 octets (160 bits,
/// ce que conseille la RFC 4226).
pub const STEP: u64 = 30;
const DIGITS: u32 = 6;
const SECRET_BYTES: usize = 20;
/// Ce qu'un formulaire de compte peut peser.
const FORM_MAX: usize = 4096;

/// Un membre connecté : son numéro dans la base, son nom.
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub id: i64,
    pub name: String,
}

impl Member {
    /// La clé sous laquelle son état est gardé, page par page : la même sur tous ses appareils.
    pub fn visit_key(&self) -> String {
        format!("account:{}", self.id)
    }
}

/// Crée les tables des comptes s'il le faut, et oublie les sessions et les essais périmés.
pub fn prepare(base: &Connection, now: u64) -> Result<(), String> {
    base.execute_batch(
        "CREATE TABLE IF NOT EXISTS accounts (
             id INTEGER PRIMARY KEY,
             name TEXT NOT NULL,
             key TEXT NOT NULL UNIQUE,
             password TEXT NOT NULL,
             code_secret BLOB,
             code_pending BLOB,
             code_step INTEGER NOT NULL DEFAULT 0,
             created INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS sessions (
             fingerprint TEXT PRIMARY KEY,
             account INTEGER NOT NULL,
             pending INTEGER NOT NULL DEFAULT 0,
             created INTEGER NOT NULL,
             seen INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS attempts (
             key TEXT PRIMARY KEY,
             failures INTEGER NOT NULL,
             until INTEGER NOT NULL,
             updated INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS recoveries(account INTEGER NOT NULL, fingerprint TEXT NOT NULL, PRIMARY KEY(account,fingerprint));
         CREATE TABLE IF NOT EXISTS account_ips(ip TEXT PRIMARY KEY, started INTEGER NOT NULL, requests INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS erased_accounts(id INTEGER PRIMARY KEY, at INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS account_erased_files(path TEXT PRIMARY KEY);",
    )
    .map_err(|e| e.to_string())?;
    let ago = |seconds: u64| now.saturating_sub(seconds) as i64;
    base.execute("DELETE FROM sessions WHERE seen < ?1 OR created < ?2 OR (pending = 1 AND created < ?3)", params![ago(SESSION_IDLE), ago(SESSION_MAX), ago(PENDING_MAX)]).map_err(|e| e.to_string())?;
    base.execute("DELETE FROM attempts WHERE updated < ?1", params![ago(24 * 3600)]).map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------- la cryptographie, empruntée

/// L'empreinte Argon2id d'un mot de passe, au format PHC (`$argon2id$v=19$m=19456,t=2,p=1$…`) : le
/// sel, tiré au hasard par le système, et les réglages y sont écrits. Les réglages par défaut de la
/// bibliothèque sont ceux que conseille l'OWASP : 19 Mo de mémoire, deux passes.
pub fn password_print(password: &str) -> Result<String, String> {
    let mut salt = [0u8; 16];
    getrandom::getrandom(&mut salt).map_err(|e| e.to_string())?;
    Argon2::default().hash_password_with_salt(password.as_bytes(), &salt).map(|print| print.to_string()).map_err(|e| e.to_string())
}

/// Ce mot de passe donne-t-il cette empreinte ? (La bibliothèque compare en temps constant.)
pub fn password_matches(password: &str, print: &str) -> bool {
    Argon2::default().verify_password(password.as_bytes(), print).is_ok()
}

/// Une empreinte pour rien : quand le nom n'existe pas, le serveur vérifie le mot de passe contre
/// elle, pour prendre le même temps que pour un vrai compte. Le temps ne dit donc pas si le nom existe.
fn print_for_nobody() -> &'static str {
    static NOBODY: OnceLock<String> = OnceLock::new();
    NOBODY.get_or_init(|| password_print(&random_hex(16)).unwrap_or_default())
}

/// Le code d'un pas de 30 secondes (RFC 4226, § 5.3 ; RFC 6238) : l'empreinte HMAC-SHA-1 de la clé
/// et du numéro du pas ; le dernier octet dit où prendre 31 bits ; on en garde les derniers chiffres.
pub fn code_at(secret: &[u8], step: u64, digits: u32) -> u32 {
    let mut mac = <Hmac<Sha1> as KeyInit>::new_from_slice(secret).expect("HMAC accepte une clé de toute longueur");
    mac.update(&step.to_be_bytes());
    let print = mac.finalize().into_bytes();
    let at = usize::from(print[19] & 0x0f);
    u32::from_be_bytes([print[at] & 0x7f, print[at + 1], print[at + 2], print[at + 3]]) % 10u32.pow(digits)
}

/// Le pas de 30 secondes dont ce code est le code : celui de maintenant, ou l'un de ses deux voisins
/// (l'horloge d'un téléphone peut avancer ou retarder un peu). `None` : ce n'est pas le code.
fn step_of(secret: &[u8], code: u32, now: u64) -> Option<u64> {
    let step = now / STEP;
    [step, step.saturating_sub(1), step + 1].into_iter().find(|candidate| code_at(secret, *candidate, DIGITS) == code)
}

/// Une clé écrite pour être recopiée dans l'application (RFC 4648, base 32, sans « = ») : 32 lettres
/// et chiffres pour 20 octets. Ce n'est pas de la cryptographie : une façon d'écrire des octets.
pub fn base32(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut written = String::new();
    let (mut buffer, mut bits) = (0u32, 0u32);
    for byte in bytes {
        buffer = ((buffer << 8) | u32::from(*byte)) & 0xFFFF;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            written.push(char::from(ALPHABET[((buffer >> bits) & 31) as usize]));
        }
    }
    if bits > 0 {
        written.push(char::from(ALPHABET[((buffer << (5 - bits)) & 31) as usize]));
    }
    written
}

/// Des octets tirés au hasard par le système, écrits en hexadécimal.
fn random_hex(bytes: usize) -> String {
    let mut drawn = vec![0u8; bytes];
    getrandom::getrandom(&mut drawn).expect("le système ne donne pas de hasard");
    drawn.iter().map(|b| format!("{b:02x}")).collect()
}

/// L'empreinte d'un numéro de session, seule gardée dans la base. Le numéro a 128 bits tirés au
/// hasard : on ne peut pas le retrouver depuis son empreinte (la faiblesse connue de SHA-1, deux
/// textes qui donnent la même empreinte, ne sert à rien ici).
fn fingerprint(token: &str) -> String {
    Sha1::digest(token.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// La valeur d'un cookie, si elle a la forme d'un numéro : 32 chiffres hexadécimaux.
fn cookie_value<'a>(cookie: &'a str, name: &str) -> Option<&'a str> {
    cookie
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(known, _)| *known == name)
        .map(|(_, value)| value)
        .filter(|value| value.len() == 32 && value.chars().all(|c| c.is_ascii_hexdigit()))
}

// ---------------------------------------------------------------- les comptes et les sessions

/// Ce que la base garde d'un compte, pour le vérifier : l'empreinte de son mot de passe, la clé de
/// son code (activée, ou qui attend le premier code), le dernier pas de 30 secondes déjà servi.
struct Account {
    id: i64,
    password: String,
    secret: Option<Vec<u8>>,
    pending: Option<Vec<u8>>,
    step: i64,
}

/// Un compte, par son numéro (`id`) ou par son nom en minuscules (`key`).
fn account_where(base: &Connection, column: &str, value: &dyn rusqlite::ToSql) -> Option<Account> {
    base.query_row(&format!("SELECT id, password, code_secret, code_pending, code_step FROM accounts WHERE {column} = ?1"), [value], |row| {
        Ok(Account { id: row.get(0)?, password: row.get(1)?, secret: row.get(2)?, pending: row.get(3)?, step: row.get(4)? })
    })
    .optional()
    .ok()
    .flatten()
}

/// Le membre connecté, d'après son cookie : une session entière (mot de passe, et code s'il l'a
/// activé), ni trop vieille, ni oubliée.
pub fn member_of(site: &Site, cookie: &str) -> Option<Member> {
    let token = cookie_value(cookie, SESSION_COOKIE)?;
    let base = site.base.lock().ok()?;
    member_in(&base, token, crate::server::now())
}

fn member_in(base: &Connection, token: &str, now: u64) -> Option<Member> {
    let print = fingerprint(token);
    let (id, name, created, seen): (i64, String, i64, i64) = base
        .query_row("SELECT a.id, a.name, s.created, s.seen FROM sessions s JOIN accounts a ON a.id = s.account WHERE s.fingerprint = ?1 AND s.pending = 0", params![print], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .optional()
        .ok()??;
    // Une session volée ne sert pas éternellement.
    if now.saturating_sub(seen as u64) > SESSION_IDLE || now.saturating_sub(created as u64) > SESSION_MAX {
        let _ = base.execute("DELETE FROM sessions WHERE fingerprint = ?1", params![print]);
        return None;
    }
    if now.saturating_sub(seen as u64) >= 60 {
        let _ = base.execute("UPDATE sessions SET seen = ?1 WHERE fingerprint = ?2", params![now as i64, print]);
    }
    Some(Member { id, name })
}

/// Le compte qui a donné le bon mot de passe et doit encore donner son code : cinq minutes au plus.
fn halfway(base: &Connection, cookie: &str, now: u64) -> Option<i64> {
    let print = fingerprint(cookie_value(cookie, SESSION_COOKIE)?);
    let (account, created): (i64, i64) = base.query_row("SELECT account, created FROM sessions WHERE fingerprint = ?1 AND pending = 1", params![print], |row| Ok((row.get(0)?, row.get(1)?))).optional().ok()??;
    (now.saturating_sub(created as u64) <= PENDING_MAX).then_some(account)
}

/// Ouvre une session pour ce compte (`pending` : le code reste à donner), et rend le cookie à poser.
/// La session que portait le navigateur, s'il y en avait une, est oubliée : jamais deux à la fois.
fn open_session(base: &Connection, cookie: &str, account: i64, pending: bool, now: u64) -> Option<String> {
    forget_session(base, cookie);
    let token = random_hex(16);
    base.execute("INSERT INTO sessions (fingerprint, account, pending, created, seen) VALUES (?1, ?2, ?3, ?4, ?4)", params![fingerprint(&token), account, i64::from(pending), now as i64]).ok()?;
    let max_age = if pending { PENDING_MAX } else { SESSION_MAX };
    Some(format!("{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}"))
}

fn forget_session(base: &Connection, cookie: &str) {
    if let Some(token) = cookie_value(cookie, SESSION_COOKIE) {
        let _ = base.execute("DELETE FROM sessions WHERE fingerprint = ?1", params![fingerprint(token)]);
    }
}

/// Le cookie qui efface la session dans le navigateur.
fn cleared_cookie() -> String {
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

/// Ce qu'un visiteur a fait avant de se connecter (son panier, sans JavaScript) suit son compte,
/// pour chaque page où le compte n'avait encore rien ; il quitte son cookie de visiteur. Après la
/// déconnexion, cet appareil repart donc d'une visite neuve.
fn bring_visits(base: &Connection, cookie: &str, member: i64) {
    if let Some(visitor) = crate::server::visitor_of(cookie) {
        let key = format!("account:{member}");
        let _ = base.execute("INSERT OR IGNORE INTO visits (visitor, page, state, tried, updated) SELECT ?1, page, state, tried, updated FROM visits WHERE visitor = ?2", params![key, visitor]);
        let _ = base.execute("DELETE FROM visits WHERE visitor = ?1", params![visitor]);
    }
}

// ---------------------------------------------------------------- le frein

/// Combien de secondes attendre encore avant un nouvel essai, pour ce nom ou ce compte.
fn waiting(base: &Connection, key: &str, now: u64) -> Option<u64> {
    let until: i64 = base.query_row("SELECT until FROM attempts WHERE key = ?1", params![key], |row| row.get(0)).optional().ok()??;
    (until as u64 > now).then(|| until as u64 - now)
}

/// Un essai raté. Au cinquième, une minute d'attente ; chaque nouvel échec la double.
fn failed(base: &Connection, key: &str, now: u64) {
    let failures = base.query_row("SELECT failures FROM attempts WHERE key = ?1", params![key], |row| row.get::<_, i64>(0)).optional().ok().flatten().unwrap_or(0) + 1;
    let until = if failures >= FREE_TRIES { now + (WAIT_FIRST << (failures - FREE_TRIES).clamp(0, 10) as u32).min(WAIT_MAX) } else { 0 };
    let _ = base.execute(
        "INSERT INTO attempts (key, failures, until, updated) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (key) DO UPDATE SET failures = excluded.failures, until = excluded.until, updated = excluded.updated",
        params![key, failures, until as i64, now as i64],
    );
}

fn succeeded(base: &Connection, key: &str) {
    let _ = base.execute("DELETE FROM attempts WHERE key = ?1", params![key]);
}

/// « Trop d'essais : attends 2 minutes avant de réessayer. »
fn wait_message(seconds: u64) -> String {
    let minutes = seconds.div_ceil(60);
    format!("Trop d'essais : attends {minutes} minute{} avant de réessayer.", if minutes > 1 { "s" } else { "" })
}

/// Le frein d'un nom tapé, qu'il existe ou non : son attente ne dit donc pas si le nom existe.
fn name_key(name: &str) -> String {
    format!("name:{}", name.to_ascii_lowercase().chars().take(64).collect::<String>())
}

/// Un code tapé : six chiffres, les espaces ne comptent pas (« 123 456 »).
fn read_code(text: &str) -> Option<u32> {
    let digits: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if digits.len() == DIGITS as usize && digits.chars().all(|c| c.is_ascii_digit()) {
        digits.parse().ok()
    } else {
        None
    }
}

// ---------------------------------------------------------------- les adresses

/// Où revenir après s'être connecté : une adresse de ce site, jamais d'un autre (`//ailleurs.example`).
fn safe_next(raw: &str) -> Option<String> {
    let local = raw.starts_with('/') && !raw.starts_with("//") && raw.len() <= 512 && raw.chars().all(|c| c.is_ascii_graphic() && !matches!(c, '\\' | '"' | '<' | '>'));
    local.then(|| raw.to_string())
}

/// La page d'où vient le visiteur (l'en-tête `Referer`), si elle est de ce site et n'est pas une
/// page de compte : un lien « Se connecter » y ramène.
fn came_from(ask: &Ask) -> Option<String> {
    let rest = ask.referer.split_once("://")?.1;
    let (host, path) = rest.split_at(rest.find('/')?);
    (host == ask.host && path != "/account" && !path.starts_with("/account/") && !path.starts_with("/account?")).then(|| safe_next(path)).flatten()
}

/// Une valeur écrite dans une adresse (`?next=`) : tout ce qui n'est ni lettre, ni chiffre, ni « / »,
/// ni « - . _ ~ » s'écrit `%XX`.
fn in_query(text: &str) -> String {
    text.bytes().map(|b| if b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'.' | b'_' | b'~') { char::from(b).to_string() } else { format!("%{b:02X}") }).collect()
}

/// La page « Se connecter », qui dit pourquoi on y arrive et ramène ensuite à cette adresse : là où
/// mène une page réservée aux membres quand on ne l'est pas.
pub fn sign_in_address(back: &str) -> String {
    let next = with_next(safe_next(back).as_deref().unwrap_or(""));
    format!("/account/signin{next}{}for=members", if next.is_empty() { "?" } else { "&" })
}

/// `?next=…`, ou rien.
fn with_next(next: &str) -> String {
    if next.is_empty() {
        String::new()
    } else {
        format!("?next={}", in_query(next))
    }
}

// ---------------------------------------------------------------- les réponses

/// Les en-têtes d'une page de compte : jamais gardée en cache (un ordinateur partagé), jamais
/// posée dans le cadre d'un autre site (on ne fait pas toucher ses boutons à son insu), sans aucun
/// script ni aucune ressource d'ailleurs.
fn private_headers() -> Vec<(String, String)> {
    vec![
        ("Cache-Control".into(), "no-store".into()),
        ("X-Content-Type-Options".into(), "nosniff".into()),
        ("Referrer-Policy".into(), "same-origin".into()),
        ("X-Frame-Options".into(), "DENY".into()),
        ("Content-Security-Policy".into(), "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'".into()),
    ]
}

fn page(status: u16, title: &str, main: &str, cookies: &[String]) -> Reply {
    let mut headers = vec![("Content-Type".to_string(), "text/html; charset=utf-8".to_string())];
    headers.extend(private_headers());
    headers.extend(cookies.iter().map(|cookie| ("Set-Cookie".to_string(), cookie.clone())));
    let body = format!(
        "<!doctype html>\n<html lang=\"fr\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{} · compte</title><style>{STYLE}</style></head><body><main>{main}</main></body></html>\n",
        escape(title)
    );
    Reply { status, headers, body: body.into_bytes() }
}

fn redirect(location: &str, cookies: &[String]) -> Reply {
    let mut headers = vec![("Location".to_string(), location.to_string())];
    headers.extend(private_headers());
    headers.extend(cookies.iter().map(|cookie| ("Set-Cookie".to_string(), cookie.clone())));
    Reply { status: 303, headers, body: Vec::new() }
}

fn refused(status: u16, text: &str) -> Reply {
    let mut headers = vec![("Content-Type".to_string(), "text/plain; charset=utf-8".to_string())];
    headers.extend(private_headers());
    Reply { status, headers, body: text.as_bytes().to_vec() }
}

/// Lisible en clair comme en sombre, de grandes cibles pour le doigt (48 pixels), le focus visible.
const STYLE: &str = ":root{color-scheme:light dark;--back:#fff;--text:#1b1b1f;--soft:#55565c;--accent:#7a4e00;--wrong:#b00020;--edge:#76767c}\
@media (prefers-color-scheme:dark){:root{--back:#101020;--text:#f2f2f5;--soft:#b9b9c2;--accent:#E9B44C;--wrong:#ff8a8a;--edge:#8a8a94}}\
body{margin:0;background:var(--back);color:var(--text);font:1.0625rem/1.5 system-ui,sans-serif}\
main{max-width:30rem;margin:0 auto;padding:2rem 1rem}h1{font-size:1.75rem;margin:0 0 1rem}h2{font-size:1.25rem;margin:2rem 0 .5rem}\
label{display:block;font-weight:600;margin:1.25rem 0 .25rem}\
input{box-sizing:border-box;width:100%;min-height:3rem;padding:.5rem .75rem;font:inherit;color:inherit;background:transparent;border:2px solid var(--edge);border-radius:.5rem}\
input[aria-invalid=true]{border-color:var(--wrong)}\
input:focus-visible,button:focus-visible,a:focus-visible{outline:3px solid var(--accent);outline-offset:2px}\
.hint{color:var(--soft);margin:0 0 .35rem;font-size:.95rem}.error{color:var(--wrong);font-weight:600;margin:.35rem 0 0}\
.alert{border:2px solid var(--wrong);border-radius:.5rem;padding:.75rem 1rem;color:var(--wrong);font-weight:600}\
.done{border:2px solid var(--accent);border-radius:.5rem;padding:.75rem 1rem}\
button{min-height:3rem;margin-top:1.25rem;padding:.5rem 1.25rem;font:inherit;font-weight:600;color:var(--back);background:var(--text);border:0;border-radius:.5rem;cursor:pointer}\
.key{font:600 1.2rem/1.7 ui-monospace,Consolas,monospace;letter-spacing:.06em;word-spacing:.35em;user-select:all;overflow-wrap:anywhere}\
a{color:var(--accent)}ol{padding-left:1.25rem}li{margin:.5rem 0}";

/// Un champ d'un formulaire de compte : son étiquette, ses explications et son erreur, reliées à lui.
struct Field<'a> {
    id: &'a str,
    label: &'a str,
    kind: &'a str,
    autocomplete: &'a str,
    value: &'a str,
    hint: &'a str,
    error: &'a str,
    /// Des attributs en plus : `inputmode="numeric"`…
    more: &'a str,
}

fn field(f: &Field, focus: bool) -> String {
    let mut described = Vec::new();
    let mut html = format!("<label for=\"{}\">{}</label>", f.id, escape(f.label));
    if !f.hint.is_empty() {
        html.push_str(&format!("<p class=\"hint\" id=\"{}-hint\">{}</p>", f.id, escape(f.hint)));
        described.push(format!("{}-hint", f.id));
    }
    if !f.error.is_empty() {
        described.push(format!("{}-error", f.id));
    }
    html.push_str(&format!(
        "<input id=\"{id}\" name=\"{id}\" type=\"{kind}\" autocomplete=\"{autocomplete}\" required{value}{more}{described}{invalid}{focus}>",
        id = f.id,
        kind = f.kind,
        autocomplete = f.autocomplete,
        value = if f.value.is_empty() { String::new() } else { format!(" value=\"{}\"", escape(f.value)) },
        more = if f.more.is_empty() { String::new() } else { format!(" {}", f.more) },
        described = if described.is_empty() { String::new() } else { format!(" aria-describedby=\"{}\"", described.join(" ")) },
        invalid = if f.error.is_empty() { "" } else { " aria-invalid=\"true\"" },
        focus = if focus { " autofocus" } else { "" },
    ));
    if !f.error.is_empty() {
        html.push_str(&format!("<p class=\"error\" id=\"{}-error\">{}</p>", f.id, escape(f.error)));
    }
    html
}

/// Un message en tête de page : une erreur, lue tout de suite par un lecteur d'écran, ou une nouvelle.
fn notice(alert: &str, done: &str) -> String {
    let mut html = String::new();
    if !alert.is_empty() {
        html.push_str(&format!("<p class=\"alert\" role=\"alert\">{}</p>", escape(alert)));
    }
    if !done.is_empty() {
        html.push_str(&format!("<p class=\"done\" role=\"status\">{}</p>", escape(done)));
    }
    html
}

fn hidden_next(next: &str) -> String {
    if next.is_empty() {
        String::new()
    } else {
        format!("<input type=\"hidden\" name=\"next\" value=\"{}\">", escape(next))
    }
}

/// Les nouvelles que les pages annoncent après un geste (`?done=…`).
fn news(done: &str) -> &'static str {
    match done {
        "signedout" => "Tu t'es déconnecté.",
        "created" => "Ton compte est créé, et tu es connecté.",
        "code" => "Le code à 6 chiffres est activé : il te sera demandé à chaque connexion.",
        "removed" => "Le code à 6 chiffres est retiré.",
        _ => "",
    }
}

fn signin_page(next: &str, name: &str, alert: &str, done: &str) -> Reply {
    let focus_password = !name.is_empty();
    let main = format!(
        "<h1>Se connecter</h1>{}<form method=\"post\" action=\"/account/signin\" novalidate>{}{}{}<button type=\"submit\">Se connecter</button></form><p>Pas encore de compte ? <a href=\"/account/signup{}\">Créer un compte</a></p>",
        notice(alert, done),
        hidden_next(next),
        field(&Field { id: "name", label: "Nom", kind: "text", autocomplete: "username", value: name, hint: "", error: "", more: "autocapitalize=\"none\" spellcheck=\"false\"" }, !focus_password),
        field(&Field { id: "password", label: "Mot de passe", kind: "password", autocomplete: "current-password", value: "", hint: "", error: "", more: "" }, focus_password),
        with_next(next),
    );
    page(if alert.is_empty() { 200 } else { 401 }, "Se connecter", &main, &[])
}

fn signup_page(next: &str, name: &str, errors: &[(&str, String)]) -> Reply {
    let error = |id: &str| errors.iter().find(|(field, _)| *field == id).map_or("", |(_, message)| message.as_str());
    let first = errors.first().map_or("", |(field, _)| *field);
    let alert = if errors.is_empty() { String::new() } else { format!("Le compte n'est pas créé. {}", errors.iter().map(|(_, message)| message.as_str()).collect::<Vec<_>>().join(" ")) };
    let main = format!(
        "<h1>Créer un compte</h1><p>Ton compte est gardé par ce site, sur l'ordinateur de son auteur : ni Google, ni Apple, ni adresse e-mail.</p>{}<form method=\"post\" action=\"/account/signup\" novalidate>{}{}{}{}<button type=\"submit\">Créer le compte</button></form><p>Déjà un compte ? <a href=\"/account/signin{}\">Se connecter</a></p>",
        notice(&alert, ""),
        hidden_next(next),
        field(&Field { id: "name", label: "Nom", kind: "text", autocomplete: "username", value: name, hint: "De 3 à 30 lettres sans accent, chiffres, « . », « - » ou « _ ».", error: error("name"), more: "autocapitalize=\"none\" spellcheck=\"false\"" }, first == "name" || first.is_empty()),
        field(&Field { id: "password", label: "Mot de passe", kind: "password", autocomplete: "new-password", value: "", hint: "12 caractères au moins. Une phrase que toi seul connais est un bon mot de passe.", error: error("password"), more: "" }, first == "password"),
        field(&Field { id: "again", label: "Le même mot de passe, encore une fois", kind: "password", autocomplete: "new-password", value: "", hint: "", error: error("again"), more: "" }, first == "again"),
        with_next(next),
    );
    page(if errors.is_empty() { 200 } else { 422 }, "Créer un compte", &main, &[])
}

fn code_page(next: &str, alert: &str) -> Reply {
    let main = format!(
        "<h1>Le code à 6 chiffres</h1><p>Ton mot de passe est le bon. Ouvre ton application d'authentification, et écris le code qu'elle montre pour ce site : il change toutes les 30 secondes. Si ton téléphone est perdu, utilise un de tes codes de secours encore inutilisés.</p>{}<form method=\"post\" action=\"/account/code\" novalidate>{}{}<button type=\"submit\">Continuer</button></form><p><a href=\"/account/signin{}\">Recommencer</a></p>",
        notice(alert, ""),
        hidden_next(next),
        field(&Field { id: "code", label: "Code à 6 chiffres ou code de secours", kind: "text", autocomplete: "one-time-code", value: "", hint: "", error: "", more: "maxlength=\"40\" spellcheck=\"false\"" }, true),
        with_next(next),
    );
    page(if alert.is_empty() { 200 } else { 401 }, "Le code à 6 chiffres", &main, &[])
}

/// Activer le code : la clé à recopier dans l'application, puis le premier code qu'elle montre.
fn setup_page(issuer: &str, member: &Member, secret: &[u8], alert: &str) -> Reply {
    let key = base32(secret);
    let grouped = key.as_bytes().chunks(4).map(|chunk| String::from_utf8_lossy(chunk).into_owned()).collect::<Vec<_>>().join(" ");
    let label = format!("{}:{}", in_query(issuer), in_query(&member.name));
    let link = format!("otpauth://totp/{label}?secret={key}&issuer={}&algorithm=SHA1&digits=6&period=30", in_query(issuer));
    let main = format!(
        "<h1>Activer le code à 6 chiffres</h1><p>Avec lui, ton mot de passe seul ne suffit plus pour entrer : il faut aussi ton téléphone. Le code est calculé par une application, sans Internet ni SMS.</p>{}\
<ol><li>Ouvre une application d'authentification : Aegis, FreeOTP, 2FAS, Google Authenticator, Microsoft Authenticator…</li>\
<li>Ajoute un compte en écrivant une clé (« saisir une clé de configuration »). Nom du compte : <strong>{}</strong>. La clé :<p class=\"key\" id=\"key\">{}</p>\
Sur ce téléphone, tu peux aussi <a href=\"{}\">l'ouvrir directement dans l'application</a>.</li>\
<li>Écris le code à 6 chiffres que l'application montre maintenant.</li></ol>\
<form method=\"post\" action=\"/account/code/setup\" novalidate>{}<button type=\"submit\">Activer le code</button></form><p><a href=\"/account\">Plus tard</a></p>",
        notice(alert, ""),
        escape(&member.name),
        grouped,
        escape(&link),
        field(&Field { id: "code", label: "Code", kind: "text", autocomplete: "one-time-code", value: "", hint: "", error: "", more: "inputmode=\"numeric\" maxlength=\"7\" spellcheck=\"false\"" }, !alert.is_empty()),
    );
    let main=main.replace("<li>Ajoute un compte", &format!("<li>Scanne ce QR ou recopie la clé : {} Ajoute un compte",qr_svg(&link)));
    page(if alert.is_empty() { 200 } else { 422 }, "Activer le code à 6 chiffres", &main, &[])
}

fn account_page(member: &Member, active: bool, done: &str, alert: &str, back: Option<&str>) -> Reply {
    let code = if active {
        format!(
            "<p>Il est activé : à chaque connexion, après ton mot de passe, ton application d'authentification te le donne.</p><form method=\"post\" action=\"/account/code/remove\" novalidate>{}<button type=\"submit\">Retirer le code</button></form>",
            field(&Field { id: "code", label: "Pour le retirer, le code que montre l'application", kind: "text", autocomplete: "one-time-code", value: "", hint: "", error: "", more: "inputmode=\"numeric\" maxlength=\"7\" spellcheck=\"false\"" }, !alert.is_empty())
        )
    } else {
        "<p>Il protège ton compte même si quelqu'un apprend ton mot de passe : il faudrait aussi ton téléphone.</p><form method=\"post\" action=\"/account/code/setup\"><button type=\"submit\">Activer le code à 6 chiffres</button></form>".to_string()
    };
    let back = back.map(|address| format!("<p><a href=\"{}\">← Revenir à la page d'avant</a></p>", escape(address))).unwrap_or_default();
    let main = format!(
        "<h1>Ton compte</h1>{}<p>Tu es connecté sous le nom <strong>{}</strong>.</p>{back}<h2>Le code à 6 chiffres</h2>{code}<h2>Se déconnecter</h2><p>Sur un ordinateur partagé, pense à te déconnecter.</p><form method=\"post\" action=\"/account/signout\"><button type=\"submit\">Se déconnecter</button></form>",
        notice(alert, done),
        escape(&member.name),
    );
    let main=format!("{main}<h2>Effacer mon compte</h2><p><a href=\"/account/delete\">Effacer mon compte et ses données</a></p>");
    page(if alert.is_empty() { 200 } else { 422 }, "Ton compte", &main, &[])
}

// ---------------------------------------------------------------- les demandes

/// Les pages de compte, à leurs adresses : `/account` (le compte), `/account/signup` (créer un
/// compte), `/account/signin` (se connecter), `/account/code` (le code, après le mot de passe),
/// `/account/code/setup` et `/account/code/remove` (activer et retirer le code),
/// `/account/signout` (se déconnecter). `None` : l'adresse n'est pas une page de compte.
pub fn answer(site: &Site, ask: &Ask, path: &str) -> Option<Reply> {
    if path != "/account" && !path.starts_with("/account/") {
        return None;
    }
    let query = ask.url.split_once('?').map_or("", |(_, query)| query.split('#').next().unwrap_or(""));
    let asked = crate::gestures::read_form(query);
    let posted = if ask.method == "POST" { crate::gestures::read_form(&String::from_utf8_lossy(ask.body)) } else { Vec::new() };
    let value = |fields: &[(String, String)], name: &str| fields.iter().find(|(known, _)| known == name).map_or(String::new(), |(_, value)| value.clone());
    let next = safe_next(&value(&posted, "next")).or_else(|| safe_next(&value(&asked, "next"))).unwrap_or_default();
    // Arrivé d'une page réservée (`for=members`) : la page le dit. Sinon, la nouvelle d'un geste.
    let done = if value(&asked, "for") == "members" { "Cette page est réservée aux membres : connecte-toi pour la voir." } else { news(&value(&asked, "done")) };
    if ask.method == "POST" {
        // Un formulaire d'un autre site ne crée pas de compte, ne connecte personne.
        if !ask.origin.is_empty() && ask.origin.split("://").nth(1) != Some(ask.host) {
            return Some(refused(403, "cet envoi vient d'un autre site"));
        }
        if !ask.content_type.starts_with("application/x-www-form-urlencoded") {
            return Some(refused(415, "un formulaire de compte"));
        }
        if ask.body.len() > FORM_MAX {
            return Some(refused(413, "formulaire trop lourd"));
        }
    }
    let now = crate::server::now();
    if ask.method == "POST" {
        let Ok(base)=site.base.lock() else {return Some(refused(500,"base indisponible"))};
        if !ip_allowed(&base,ask.peer,now) {let mut reply=refused(429,"Trop de demandes depuis cette adresse : attends une minute.");reply.headers.push(("Retry-After".into(),"60".into()));return Some(reply);}
    }
    let member = member_of(site, ask.cookie);
    let base = || site.base.lock().ok();
    let reply = match (ask.method, path) {
        ("GET" | "HEAD", "/account") => match &member {
            Some(member) => {
                let active = base().and_then(|base| account_where(&base, "id", &member.id)).is_some_and(|account| account.secret.is_some());
                account_page(member, active, done, "", came_from(ask).as_deref())
            }
            None => redirect(&format!("/account/signin{}", with_next(&next)), &[]),
        },
        // Sans `?next=`, la page d'où l'on vient : un lien « Se connecter » y ramène.
        ("GET" | "HEAD", "/account/signin") => match &member {
            Some(_) => redirect(if next.is_empty() { "/account" } else { &next }, &[]),
            None => signin_page(&if next.is_empty() { came_from(ask).unwrap_or_default() } else { next.clone() }, "", "", done),
        },
        ("GET" | "HEAD", "/account/signup") => match &member {
            Some(_) => redirect("/account", &[]),
            None => signup_page(&if next.is_empty() { came_from(ask).unwrap_or_default() } else { next.clone() }, "", &[]),
        },
        ("GET" | "HEAD", "/account/code") => match base().and_then(|base| halfway(&base, ask.cookie, now)) {
            Some(_) => code_page(&next, ""),
            None => redirect(&format!("/account/signin{}", with_next(&next)), &[]),
        },
        ("GET" | "HEAD", "/account/code/setup") => match &member {
            Some(member) => match base().and_then(|base| account_where(&base, "id", &member.id)).and_then(|account| account.pending) {
                Some(secret) => setup_page(&issuer(site), member, &secret, ""),
                None => redirect("/account", &[]),
            },
            None => redirect("/account/signin?next=/account", &[]),
        },
        ("POST", "/account/signup") => sign_up(site, ask, &posted, &next, now),
        ("POST", "/account/signin") => sign_in(site, ask, &posted, &next, now),
        ("POST", "/account/code") => second_step(site, ask, &posted, &next, now),
        ("POST", "/account/code/setup") => match &member {
            Some(member) => set_up_code(site, member, &posted, now),
            None => redirect("/account/signin?next=/account", &[]),
        },
        ("POST", "/account/code/remove") => match &member {
            Some(member) => remove_code(site, ask, member, &posted, now),
            None => redirect("/account/signin?next=/account", &[]),
        },
        ("GET" | "HEAD", "/account/delete") => match &member {
            Some(member) => {
                let active=base().and_then(|base|account_where(&base,"id",&member.id)).is_some_and(|a|a.secret.is_some());
                deletion_page(member,active,"")
            },
            None => redirect("/account/signin?next=/account",&[]),
        },
        ("POST", "/account/delete") => match &member {Some(member)=>delete_account(site,ask,member,&posted,now),None=>redirect("/account/signin?next=/account",&[])},
        ("POST", "/account/signout") => {
            if let Some(base) = base() {
                forget_session(&base, ask.cookie);
            }
            redirect("/account/signin?done=signedout", &[cleared_cookie()])
        }
        ("GET" | "HEAD", "/account/signout") => refused(405, "on se déconnecte par le bouton « Se déconnecter » de la page du compte, /account"),
        ("GET" | "HEAD" | "POST", _) => refused(404, "aucune page de compte à cette adresse"),
        _ => refused(405, "seuls GET et POST sont reçus"),
    };
    Some(reply)
}

/// Le nom que l'application d'authentification montre pour ce site : celui de son dossier.
fn issuer(site: &Site) -> String {
    site.folder.file_name().and_then(|name| name.to_str()).filter(|name| !name.is_empty()).unwrap_or("HoloCode").to_string()
}

fn sign_up(site: &Site, ask: &Ask, posted: &[(String, String)], next: &str, now: u64) -> Reply {
    let value = |name: &str| posted.iter().find(|(known, _)| known == name).map_or("", |(_, value)| value.as_str());
    let (name, password, again) = (value("name").trim(), value("password"), value("again"));
    let mut errors: Vec<(&str, String)> = Vec::new();
    if name.is_empty() {
        errors.push(("name", "Écris un nom.".into()));
    } else if !crate::account::valid_name(name) {
        errors.push(("name", "Ce nom ne va pas : de 3 à 30 lettres sans accent, chiffres, « . », « - » ou « _ », une lettre ou un chiffre d'abord.".into()));
    }
    let length = password.chars().count();
    if length < PASSWORD_MIN {
        errors.push(("password", format!("Le mot de passe est trop court : {PASSWORD_MIN} caractères au moins.")));
    } else if length > PASSWORD_MAX {
        errors.push(("password", format!("Le mot de passe est trop long : {PASSWORD_MAX} caractères au plus.")));
    } else if again != password {
        errors.push(("again", "Les deux mots de passe ne sont pas les mêmes.".into()));
    }
    if !errors.is_empty() {
        return signup_page(next, name, &errors);
    }
    let key = name.to_ascii_lowercase();
    {
        let Ok(base) = site.base.lock() else { return refused(500, "base indisponible") };
        if account_where(&base, "key", &key).is_some() {
            return signup_page(next, name, &[("name", "Ce nom est déjà pris : choisis-en un autre.".into())]);
        }
        let count: i64 = base.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0)).unwrap_or(ACCOUNTS_MAX);
        if count >= ACCOUNTS_MAX {
            return signup_page(next, name, &[("name", "Ce site a déjà trop de comptes.".into())]);
        }
    }
    // L'empreinte se calcule sans bloquer la base : elle prend un instant, exprès.
    let Ok(print) = password_print(password) else { return refused(500, "empreinte impossible") };
    let Ok(base) = site.base.lock() else { return refused(500, "base indisponible") };
    let id: i64=base.query_row("SELECT MAX(id)+1 FROM (SELECT COALESCE(MAX(id),0) AS id FROM accounts UNION ALL SELECT COALESCE(MAX(id),0) AS id FROM erased_accounts)",[],|r|r.get(0)).unwrap_or(i64::MAX);
    if base.execute("INSERT INTO accounts (id, name, key, password, created) VALUES (?1, ?2, ?3, ?4, ?5)", params![id, name, key, print, now as i64]).is_err() {
        // Deux demandes pour le même nom au même instant : la seconde trouve le nom pris.
        return signup_page(next, name, &[("name", "Ce nom est déjà pris : choisis-en un autre.".into())]);
    }
    let id = base.last_insert_rowid();
    println!("Compte créé : {name}");
    let Some(cookie) = open_session(&base, ask.cookie, id, false, now) else { return refused(500, "session impossible") };
    bring_visits(&base, ask.cookie, id);
    redirect(if next.is_empty() { "/account?done=created" } else { next }, &[cookie])
}

fn sign_in(site: &Site, ask: &Ask, posted: &[(String, String)], next: &str, now: u64) -> Reply {
    let value = |name: &str| posted.iter().find(|(known, _)| known == name).map_or("", |(_, value)| value.as_str());
    let (name, password) = (value("name").trim(), value("password"));
    let throttle = name_key(name);
    let account = {
        let Ok(base) = site.base.lock() else { return refused(500, "base indisponible") };
        if let Some(seconds) = waiting(&base, &throttle, now) {
            return signin_page(next, name, &wait_message(seconds), "");
        }
        account_where(&base, "key", &name.to_ascii_lowercase())
    };
    // Le mot de passe est vérifié même pour un nom inconnu, contre une empreinte pour rien : le temps
    // de la réponse ne dit pas si le nom existe. La réponse non plus.
    let print = match &account {
        Some(account) => account.password.as_str(),
        None => print_for_nobody(),
    };
    let matches = password_matches(password, print);
    let Ok(base) = site.base.lock() else { return refused(500, "base indisponible") };
    let Some(account) = account.filter(|a| matches && !password.is_empty() && account_where(&base,"id",&a.id).is_some()) else {
        failed(&base, &throttle, now);
        return signin_page(next, name, "Ce nom et ce mot de passe ne vont pas ensemble.", "");
    };
    succeeded(&base, &throttle);
    // Le code à 6 chiffres est activé : le mot de passe ne suffit pas, une demi-session attend le code.
    if account.secret.is_some() {
        let Some(cookie) = open_session(&base, ask.cookie, account.id, true, now) else { return refused(500, "session impossible") };
        return redirect(&format!("/account/code{}", with_next(next)), &[cookie]);
    }
    let Some(cookie) = open_session(&base, ask.cookie, account.id, false, now) else { return refused(500, "session impossible") };
    bring_visits(&base, ask.cookie, account.id);
    redirect(if next.is_empty() { "/account" } else { next }, &[cookie])
}

/// Le code, après le bon mot de passe : la demi-session devient une session entière.
fn second_step(site: &Site, ask: &Ask, posted: &[(String, String)], next: &str, now: u64) -> Reply {
    let Ok(base) = site.base.lock() else { return refused(500, "base indisponible") };
    let Some(account) = halfway(&base, ask.cookie, now).and_then(|id| account_where(&base, "id", &id)) else {
        return redirect(&format!("/account/signin{}", with_next(next)), &[cleared_cookie()]);
    };
    let Some(secret) = account.secret.as_deref() else { return redirect("/account/signin", &[cleared_cookie()]) };
    let throttle = format!("code:{}", account.id);
    if let Some(seconds) = waiting(&base, &throttle, now) {
        return code_page(next, &wait_message(seconds));
    }
    let typed = posted.iter().find(|(known, _)| known == "code").map_or("", |(_, value)| value.as_str());
    let step = match read_code(typed).and_then(|code| step_of(secret, code, now)) {
        Some(step) if step as i64 <= account.step => return code_page(next, "Ce code a déjà servi : attends que l'application en montre un nouveau."),
        Some(step) => Some(step),
        None if use_recovery(&base, account.id, secret, typed) => None,
        None => {failed(&base,&throttle,now);return code_page(next,"Ce code ne va pas : utilise un code actuel ou un code de secours encore inutilisé.");}
    };
    succeeded(&base,&throttle);
    if let Some(step)=step {
        if base.execute("UPDATE accounts SET code_step=?1 WHERE id=?2",params![step as i64,account.id]).is_err(){return refused(500,"base indisponible");}
    }
    let Some(cookie) = open_session(&base, ask.cookie, account.id, false, now) else { return refused(500, "session impossible") };
    bring_visits(&base, ask.cookie, account.id);
    redirect(if next.is_empty() { "/account" } else { next }, &[cookie])
}

/// Activer le code : sans code envoyé, une nouvelle clé attend (montrée par `/account/code/setup`) ;
/// avec le premier code que montre l'application, elle devient la clé du compte.
fn set_up_code(site: &Site, member: &Member, posted: &[(String, String)], now: u64) -> Reply {
    let Ok(mut base) = site.base.lock() else { return refused(500, "base indisponible") };
    let Some(account) = account_where(&base, "id", &member.id) else { return redirect("/account/signin", &[cleared_cookie()]) };
    if account.secret.is_some() {
        return redirect("/account", &[]);
    }
    let Some(typed) = posted.iter().find(|(known, _)| known == "code").map(|(_, value)| value.as_str()) else {
        if account.pending.is_none() {
            let mut secret = [0u8; SECRET_BYTES];
            if getrandom::getrandom(&mut secret).is_err() || base.execute("UPDATE accounts SET code_pending = ?1 WHERE id = ?2", params![secret.to_vec(), account.id]).is_err() {
                return refused(500, "clé impossible");
            }
        }
        return redirect("/account/code/setup", &[]);
    };
    let Some(secret) = account.pending.as_deref() else { return redirect("/account", &[]) };
    let throttle = format!("code:{}", account.id);
    if let Some(seconds) = waiting(&base, &throttle, now) {
        return setup_page(&issuer(site), member, secret, &wait_message(seconds));
    }
    let Some(step) = read_code(typed).and_then(|code| step_of(secret, code, now)) else {
        failed(&base, &throttle, now);
        return setup_page(&issuer(site), member, secret, "Ce code ne va pas : vérifie la clé recopiée, puis écris les 6 chiffres que l'application montre maintenant.");
    };
    succeeded(&base, &throttle);
    let Ok(tx)=base.transaction() else{return refused(500,"base indisponible")};
    let Ok(clear)=new_recoveries(&tx,account.id,secret) else{return refused(500,"codes de secours impossibles")};
    if tx.execute("UPDATE accounts SET code_secret=code_pending,code_pending=NULL,code_step=?1 WHERE id=?2",params![step as i64,account.id]).is_err() || tx.commit().is_err() {return refused(500,"base indisponible");}
    recovery_page(member,&clear)
}

/// Retirer le code : il faut le code du moment (un téléphone perdu : l'auteur du site le retire
/// pour toi, voir les dettes de l'ADR-081).
fn remove_code(site: &Site, ask: &Ask, member: &Member, posted: &[(String, String)], now: u64) -> Reply {
    let Ok(base) = site.base.lock() else { return refused(500, "base indisponible") };
    let Some(account) = account_where(&base, "id", &member.id) else { return redirect("/account/signin", &[cleared_cookie()]) };
    let Some(secret) = account.secret.as_deref() else { return redirect("/account", &[]) };
    let throttle = format!("code:{}", account.id);
    let back = came_from(ask);
    if let Some(seconds) = waiting(&base, &throttle, now) {
        return account_page(member, true, "", &wait_message(seconds), back.as_deref());
    }
    let typed = posted.iter().find(|(known, _)| known == "code").map_or("", |(_, value)| value.as_str());
    match read_code(typed).and_then(|code| step_of(secret, code, now)) {
        Some(step) if step as i64 > account.step => {
            succeeded(&base, &throttle);
            let _ = base.execute("UPDATE accounts SET code_secret = NULL, code_step = ?1 WHERE id = ?2", params![step as i64, account.id]);
            let _ = base.execute("DELETE FROM recoveries WHERE account=?1",params![account.id]);
            redirect("/account?done=removed", &[])
        }
        Some(_) => account_page(member, true, "", "Ce code a déjà servi : attends que l'application en montre un nouveau.", back.as_deref()),
        None => {
            failed(&base, &throttle, now);
            account_page(member, true, "", "Ce code ne va pas : le code n'est pas retiré.", back.as_deref())
        }
    }
}


// Une seconde protection : tous les POST de compte d'une même IP, même sous des noms différents.
const IP_REQUESTS_MAX: i64 = 30;
const IP_WINDOW: u64 = 60;
fn ip_allowed(base: &Connection, peer: &str, now: u64) -> bool {
    let Ok(ip) = peer.parse::<std::net::IpAddr>() else { return false };
    let key = ip.to_string();
    let _ = base.execute("DELETE FROM account_ips WHERE started < ?1", params![now.saturating_sub(IP_WINDOW) as i64]);
    let old: Option<(i64,i64)> = base.query_row("SELECT started, requests FROM account_ips WHERE ip = ?1", params![key], |r| Ok((r.get(0)?,r.get(1)?))).optional().ok().flatten();
    let (start,count) = match old { Some((s,n)) if now.saturating_sub(s as u64) < IP_WINDOW => (s,n), _ => (now as i64,0) };
    if count >= IP_REQUESTS_MAX { return false; }
    if old.is_none() && base.query_row("SELECT COUNT(*) FROM account_ips", [], |r| r.get::<_,i64>(0)).unwrap_or(10_000) >= 10_000 { return false; }
    base.execute("INSERT INTO account_ips(ip,started,requests) VALUES(?1,?2,?3) ON CONFLICT(ip) DO UPDATE SET started=excluded.started, requests=excluded.requests", params![key,start,count+1]).is_ok()
}

fn recovery_print(secret: &[u8], token: &str) -> Option<String> {
    let clear: String = token.chars().filter(|c| *c != '-').collect();
    if clear.len() != 32 || !clear.bytes().all(|c| c.is_ascii_hexdigit()) { return None; }
    let mut mac = <Hmac<Sha1> as KeyInit>::new_from_slice(secret).ok()?;
    mac.update(b"holocode-recovery-v1:"); mac.update(clear.to_ascii_lowercase().as_bytes());
    Some(mac.finalize().into_bytes().iter().map(|b| format!("{b:02x}")).collect())
}
fn new_recoveries(base: &Connection, account: i64, secret: &[u8]) -> Result<Vec<String>, rusqlite::Error> {
    base.execute("DELETE FROM recoveries WHERE account=?1",params![account])?;
    let mut clear = Vec::new();
    for _ in 0..10 {
        let token=random_hex(16);
        let print=recovery_print(secret,&token).expect("un code tiré par le système est valide");
        base.execute("INSERT INTO recoveries(account,fingerprint) VALUES(?1,?2)",params![account,print])?;
        clear.push(token.as_bytes().chunks(8).map(|c| String::from_utf8_lossy(c).into_owned()).collect::<Vec<_>>().join("-"));
    }
    Ok(clear)
}
fn use_recovery(base: &Connection, account: i64, secret: &[u8], code: &str) -> bool {
    recovery_print(secret,code).is_some_and(|p| base.execute("DELETE FROM recoveries WHERE account=?1 AND fingerprint=?2",params![account,p]).is_ok_and(|n|n==1))
}
fn recovery_page(member: &Member, clear: &[String]) -> Reply {
    let codes=clear.iter().map(|c| format!("<li><code data-recovery>{}</code></li>",escape(c))).collect::<Vec<_>>().join("");
    page(200,"Garder tes codes de secours",&format!("<h1>Garde tes dix codes de secours</h1><p>Le code à 6 chiffres est activé. Cette liste est montrée une seule fois. Copie-la dans un endroit sûr, séparé de ton téléphone. Chaque code sert une seule fois, après ton mot de passe.</p><p>Compte : {}</p><ol>{codes}</ol><p>Un rechargement ne les montre plus. Aucun code en clair n'est gardé dans la base.</p><p><a href=\"/account\">Continuer vers mon compte</a></p>",escape(&member.name)),&[])
}
fn qr_svg(link: &str) -> String {
    let Ok(code)=qrcode::QrCode::new(link.as_bytes()) else { return "<p>QR indisponible ; recopie la clé.</p>".into() };
    let width=code.width();let size=width+8;let mut path=String::new();
    for y in 0..width { for x in 0..width { if code[(x,y)]==qrcode::Color::Dark { path.push_str(&format!("M{} {}h1v1h-1z",x+4,y+4)); }}}
    format!("<svg id=\"setup-qr\" role=\"img\" aria-label=\"Scanner cette clé dans ton application d'authentification\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {size} {size}\" width=\"280\" height=\"280\" style=\"max-width:100%;height:auto\" shape-rendering=\"crispEdges\"><rect width=\"{size}\" height=\"{size}\" fill=\"white\"/><path fill=\"black\" d=\"{path}\"/></svg>")
}
fn deletion_page(member: &Member, active: bool, alert: &str) -> Reply {
    let factor=if active {field(&Field{id:"code",label:"Code à 6 chiffres ou code de secours",kind:"text",autocomplete:"one-time-code",value:"",hint:"",error:"",more:"maxlength=\"40\" spellcheck=\"false\""},false)}else{String::new()};
    let main=format!("<h1>Effacer mon compte</h1>{}<p>Cette action efface ton compte, ses sessions, ses paniers et les messages et fichiers qui lui sont associés dans la base active et les sauvegardes locales du moteur. Les copies sorties de ce dossier restent à l'auteur. Le contenu public partagé appartient à la page.</p><form method=\"post\" action=\"/account/delete\" novalidate>{}{}{factor}<label for=\"confirm\">Pour confirmer, écris ton nom : {}</label><input id=\"confirm\" name=\"confirm\" autocomplete=\"off\" maxlength=\"30\" required><button type=\"submit\">Effacer définitivement mon compte</button></form><p><a href=\"/account\">Garder mon compte</a></p>",notice(alert,""),field(&Field{id:"password",label:"Ton mot de passe",kind:"password",autocomplete:"current-password",value:"",hint:"",error:"",more:"maxlength=\"128\""},true), "", escape(&member.name));
    page(if alert.is_empty(){200}else{422},"Effacer mon compte",&main,&[])
}
// Les chemins du moteur, jamais un chemin fourni librement par le navigateur.
pub(crate) fn retry_erased_files(folder: &std::path::Path, base: &Connection) -> Result<(), String> { erase_files(&folder.join(crate::server::DATA_FOLDER),base) }
fn erase_files(data: &std::path::Path, base: &Connection) -> Result<(), String> {
    let mut stmt=base.prepare("SELECT path FROM account_erased_files").map_err(|e| e.to_string())?;
    let paths=stmt.query_map([],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;drop(stmt);
    for relative in paths {
        let parts:Vec<&str>=relative.split('/').collect();
        if parts.len()!=3||parts[0]!="files"||parts[1..].iter().any(|p|p.is_empty()||*p=="."||*p==".."||!p.bytes().all(|c|c.is_ascii_alphanumeric()||b"-_.".contains(&c))) {return Err("chemin de fichier privé refusé".into());}
        let path=data.join(&relative);
        if path.exists() {
            let root=data.join("files").canonicalize().map_err(|e|e.to_string())?;
            let target=path.canonicalize().map_err(|e|e.to_string())?;
            if !target.starts_with(root) || std::fs::symlink_metadata(&path).map_err(|e|e.to_string())?.file_type().is_symlink() {return Err("fichier privé hors du dossier".into());}
            std::fs::remove_file(&path).map_err(|e|e.to_string())?;
        }
        base.execute("DELETE FROM account_erased_files WHERE path=?1",params![relative]).map_err(|e|e.to_string())?;
    }
    Ok(())
}
fn erase_in(base: &mut Connection, member: &Member, at: u64, extra_files: &[String]) -> Result<(), String> {
    let tx=base.transaction().map_err(|e|e.to_string())?;
    let mut query=tx.prepare("SELECT files FROM messages WHERE account=?1").map_err(|e|e.to_string())?;
    let messages=query.query_map(params![member.id],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;drop(query);
    for path in extra_files {tx.execute("INSERT OR IGNORE INTO account_erased_files(path) VALUES(?1)",params![path]).map_err(|e|e.to_string())?;}
    for encoded in messages {
        if let Some(crate::lists::Json::Table(files))=crate::lists::Json::read(&encoded) {
            for file in files {if let crate::lists::Json::Object(fields)=file {if let Some(crate::lists::Json::Text(path))=fields.iter().find(|(k,_)|k=="file").map(|(_,v)|v) {tx.execute("INSERT OR IGNORE INTO account_erased_files(path) VALUES(?1)",params![path]).map_err(|e|e.to_string())?;}}}
        }
    }
    tx.execute("INSERT OR IGNORE INTO erased_accounts(id,at) VALUES(?1,?2)",params![member.id,at as i64]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM sessions WHERE account=?1",params![member.id]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM recoveries WHERE account=?1",params![member.id]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM visits WHERE visitor=?1",params![member.visit_key()]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM messages WHERE account=?1",params![member.id]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM attempts WHERE key=?1 OR key=?2 OR key=?3",params![name_key(&member.name),format!("code:{}",member.id),format!("delete:{}",member.id)]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM accounts WHERE id=?1",params![member.id]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    Ok(())
}
fn delete_account(site: &Site, ask: &Ask, member: &Member, fields: &[(String,String)], now: u64) -> Reply {
    let value=|key:&str|fields.iter().find(|(k,_)|k==key).map_or("",|(_,v)|v.as_str());
    let Ok(_gesture)=site.gestures.lock() else{return refused(500,"arbitre indisponible")};
    let Ok(mut base)=site.base.lock() else{return refused(500,"base indisponible")};
    let Some(account)=account_where(&base,"id",&member.id) else{return redirect("/account/signin",&[cleared_cookie()])};
    let throttle=format!("delete:{}",member.id);
    if waiting(&base,&throttle,now).is_some() {return deletion_page(member,account.secret.is_some(),"Trop d'essais : attends une minute.");}
    if value("confirm")!=member.name||!password_matches(value("password"),&account.password) {failed(&base,&throttle,now);return deletion_page(member,account.secret.is_some(),"Le nom de confirmation ou le mot de passe ne va pas.");}
    if let Some(secret)=account.secret.as_deref() {
        let valid=read_code(value("code")).and_then(|code|step_of(secret,code,now)).is_some_and(|step|step as i64>account.step)||use_recovery(&base,account.id,secret,value("code"));
        if !valid {failed(&base,&throttle,now);return deletion_page(member,true,"Le code ne va pas, ou il a déjà servi.");}
    }
    let data=site.folder.join(crate::server::DATA_FOLDER);let backups=data.join("backups");
    // Préparer et vérifier les copies gérées par ce moteur avant de retirer le compte actif.
    let paths=match std::fs::read_dir(&backups){Ok(entries)=>entries.filter_map(Result::ok).map(|e|e.path()).filter(|p|p.file_name().and_then(|n|n.to_str()).is_some_and(|n|n.starts_with("site-")&&n.ends_with(".sqlite"))).collect::<Vec<_>>(),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Vec::new(),Err(_)=>return refused(500,"sauvegardes illisibles : compte conservé")};
    let mut backup_files=Vec::new();
    for path in paths {
        let Ok(meta)=std::fs::symlink_metadata(&path) else{return refused(500,"sauvegarde illisible : compte conservé")};
        if !meta.is_file()||meta.file_type().is_symlink() {return refused(500,"sauvegarde refusée : compte conservé");}
        let Ok(mut copy)=Connection::open(&path) else{return refused(500,"sauvegarde indisponible : compte conservé")};
        if prepare(&copy,now).is_err(){return refused(500,"sauvegarde indisponible : compte conservé");}
        let _=copy.execute("ALTER TABLE messages ADD COLUMN account INTEGER",[]);
        if erase_in(&mut copy,member,now,&[]).is_err(){return refused(500,"effacement d'une sauvegarde impossible : compte conservé");}
        let Ok(mut query)=copy.prepare("SELECT path FROM account_erased_files") else{return refused(500,"fichiers de sauvegarde illisibles : compte conservé")};
        let Ok(rows)=query.query_map([],|r|r.get::<_,String>(0)) else{return refused(500,"fichiers de sauvegarde illisibles : compte conservé")};
        for row in rows {if let Ok(path)=row {backup_files.push(path);}else{return refused(500,"fichier de sauvegarde illisible : compte conservé");}}
    }
    if erase_in(&mut base,member,now,&backup_files).is_err(){return refused(500,"effacement impossible : compte conservé");}
    let files_done=erase_files(&data,&base).is_ok();
    forget_session(&base,ask.cookie);
    let note=if files_done{"Ton compte et ses données ont été effacés."}else{"Le compte est effacé. Un fichier privé attend encore l'effacement par le serveur."};
    let _=base.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    let mut reply=page(200,"Compte effacé",&format!("<h1>Compte effacé</h1><p>{note}</p><p><a href=\"/account/signup\">Créer un nouveau compte</a></p>"),&[]);
    reply.headers.push(("Set-Cookie".into(),cleared_cookie()));reply
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_codes_follow_rfc_6238() {
        // RFC 6238, annexe B : la clé « 12345678901234567890 », SHA-1, des pas de 30 secondes, 8 chiffres.
        let secret = b"12345678901234567890";
        for (time, expected) in [(59u64, 94_287_082u32), (1_111_111_109, 7_081_804), (1_111_111_111, 14_050_471), (1_234_567_890, 89_005_924), (2_000_000_000, 69_279_037), (20_000_000_000, 65_353_130)] {
            assert_eq!(code_at(secret, time / STEP, 8), expected, "à {time} s");
            // Les 6 derniers chiffres : ce que montre l'application.
            assert_eq!(code_at(secret, time / STEP, 6), expected % 1_000_000, "à {time} s, 6 chiffres");
        }
        // RFC 4226, annexe D : les dix premiers compteurs, 6 chiffres.
        let hotp = [755_224, 287_082, 359_152, 969_429, 338_314, 254_676, 287_922, 162_583, 399_871, 520_489];
        for (counter, expected) in hotp.into_iter().enumerate() {
            assert_eq!(code_at(secret, counter as u64, 6), expected, "compteur {counter}");
        }
        // Le pas de maintenant, et ses deux voisins seulement.
        let now = 1_111_111_111;
        assert_eq!(step_of(secret, 50_471, now), Some(now / STEP));
        assert_eq!(step_of(secret, code_at(secret, now / STEP + 1, 6), now), Some(now / STEP + 1));
        assert_eq!(step_of(secret, code_at(secret, now / STEP - 1, 6), now), Some(now / STEP - 1));
        assert_eq!(step_of(secret, code_at(secret, now / STEP + 2, 6), now), None);
        assert_eq!((read_code("050 471"), read_code("50471"), read_code("12345a")), (Some(50_471), None, None));
    }

    #[test]
    fn the_key_is_written_in_base_32() {
        // RFC 4648, § 10, sans les « = ».
        for (bytes, written) in [("", ""), ("f", "MY"), ("fo", "MZXQ"), ("foo", "MZXW6"), ("foob", "MZXW6YQ"), ("fooba", "MZXW6YTB"), ("foobar", "MZXW6YTBOI")] {
            assert_eq!(base32(bytes.as_bytes()), written);
        }
        assert_eq!(base32(b"12345678901234567890"), "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
    }

    #[test]
    fn a_password_is_never_kept_in_clear() {
        let print = password_print("une phrase que je connais").unwrap();
        assert!(print.starts_with("$argon2id$v=19$m=19456,t=2,p=1$") && !print.contains("une phrase"), "{print}");
        assert!(password_matches("une phrase que je connais", &print));
        assert!(!password_matches("une phrase que je connai", &print));
        // Salée : le même mot de passe ne donne jamais deux fois la même empreinte.
        assert_ne!(password_print("une phrase que je connais").unwrap(), print);
        assert!(!password_matches("", print_for_nobody()));
    }

    // ------------------------------------------------ le serveur entier, par ses demandes

    const MEMBERS: &str = "Page(title: \"Membres\", access: members, children: [ H1(\"Bonjour, {account}\"), P(\"Le coin des membres.\") ])";
    const CART: &str = "Page(title: \"Panier\", state: State(cart: 0, note: \"\"), children: [ P(\"Panier : {cart}\"), Input(label: \"Note\", value: note), Button(name: Add, text: \"Ajouter\"), Form(name: Contact, children: [ Input(label: \"Nom\", value: note), Button(name: Send, text: \"Envoyer\") ]) ], rules: [ On(Add.tap, effect: cart.add(1)), On(Send.tap, effect: Contact.send) ])";

    fn site() -> (Site, std::path::PathBuf) {
        let folder = std::env::temp_dir().join(format!("holo-accounts-{}", random_hex(8)));
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("members.holo"), MEMBERS).unwrap();
        std::fs::write(folder.join("cart.holo"), CART).unwrap();
        let web = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
        (Site::open(&folder, &web).unwrap(), folder)
    }

    fn ask<'a>(method: &'a str, url: &'a str, cookie: &'a str, body: &'a [u8]) -> Ask<'a> {
        Ask { method, url, accept: "text/html", cookie, content_type: "application/x-www-form-urlencoded", origin: "", host: "localhost:8080", referer: "", peer: "127.0.0.1", body }
    }

    fn header<'a>(reply: &'a Reply, name: &str) -> &'a str {
        reply.headers.iter().find(|(known, _)| known == name).map_or("", |(_, value)| value.as_str())
    }

    /// Le cookie de session posé par une réponse, prêt à être renvoyé : `holo_session=…`.
    fn session(reply: &Reply) -> String {
        reply.headers.iter().filter(|(name, _)| name == "Set-Cookie").map(|(_, value)| value.split(';').next().unwrap().to_string()).find(|c| c.starts_with("holo_session=")).unwrap_or_default()
    }

    fn text(reply: Reply) -> String {
        String::from_utf8(reply.body).unwrap()
    }

    fn sign_up(site: &Site, name: &str, password: &str) -> String {
        let body = format!("name={name}&password={password}&again={password}");
        let reply = site.answer(&ask("POST", "/account/signup", "", body.as_bytes()));
        assert_eq!((reply.status, header(&reply, "Location")), (303, "/account?done=created"));
        session(&reply)
    }

    fn sign_in(site: &Site, cookie: &str, name: &str, password: &str) -> Reply {
        site.answer(&ask("POST", "/account/signin", cookie, format!("name={name}&password={password}").as_bytes()))
    }

    fn secret_of(site: &Site, name: &str, column: &str) -> Vec<u8> {
        site.base.lock().unwrap().query_row(&format!("SELECT {column} FROM accounts WHERE key = ?1"), params![name.to_ascii_lowercase()], |row| row.get(0)).unwrap()
    }

    /// Un code qui n'est le code d'aucun des pas qu'on pourrait accepter.
    fn wrong_code(secret: &[u8], now: u64) -> String {
        let step = now / STEP;
        let near: Vec<u32> = (step.saturating_sub(2)..=step + 2).map(|s| code_at(secret, s, 6)).collect();
        let wrong = (0..).map(|n: u32| (near[0] + 1 + n * 7919) % 1_000_000).find(|code| !near.contains(code)).unwrap();
        format!("{wrong:06}")
    }

    #[test]
    fn an_account_with_its_code_and_its_brake() {
        let (site, folder) = site();
        // Créer un compte : ce qui ne va pas est dit sous le champ, rien n'est créé.
        let short = text(site.answer(&ask("POST", "/account/signup", "", b"name=Ada&password=court&again=court")));
        assert!(short.contains("Le mot de passe est trop court : 12 caractères au moins.") && short.contains("aria-describedby=\"password-hint password-error\" aria-invalid=\"true\""), "{short}");
        assert!(short.contains("role=\"alert\"") && short.contains("autocomplete=\"new-password\"") && short.contains("value=\"Ada\""), "{short}");
        assert!(text(site.answer(&ask("POST", "/account/signup", "", b"name=A%C3%A9&password=une+phrase+assez+longue&again=une+phrase+assez+longue"))).contains("Ce nom ne va pas"));
        assert!(text(site.answer(&ask("POST", "/account/signup", "", b"name=Ada&password=une+phrase+assez+longue&again=une+autre+phrase+longue"))).contains("Les deux mots de passe ne sont pas les mêmes."));
        let count = || site.base.lock().unwrap().query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get::<_, i64>(0)).unwrap();
        assert_eq!(count(), 0);
        let first = sign_up(&site, "Ada", "une+phrase+assez+longue");
        assert!(first.len() == "holo_session=".len() + 32, "{first}");
        // Le même nom, même écrit autrement, est déjà pris.
        assert!(text(site.answer(&ask("POST", "/account/signup", "", b"name=ADA&password=une+phrase+assez+longue&again=une+phrase+assez+longue"))).contains("Ce nom est déjà pris"));
        // Le mot de passe n'est jamais gardé en clair : seulement son empreinte Argon2id.
        let print: String = site.base.lock().unwrap().query_row("SELECT password FROM accounts", [], |row| row.get(0)).unwrap();
        assert!(print.starts_with("$argon2id$") && !print.contains("phrase"), "{print}");
        let page = text(site.answer(&ask("GET", "/account", &first, b"")));
        assert!(page.contains("Tu es connecté sous le nom <strong>Ada</strong>") && page.contains("Activer le code à 6 chiffres"), "{page}");

        // Activer le code : une clé attend, montrée à recopier ; le premier code l'active.
        let start = site.answer(&ask("POST", "/account/code/setup", &first, b""));
        assert_eq!((start.status, header(&start, "Location")), (303, "/account/code/setup"));
        let pending = secret_of(&site, "Ada", "code_pending");
        assert_eq!(pending.len(), SECRET_BYTES);
        let shown = text(site.answer(&ask("GET", "/account/code/setup", &first, b"")));
        let key = base32(&pending);
        assert!(shown.contains(&key[..4]) && shown.contains(&format!("secret={key}&amp;issuer=")) && shown.contains("autocomplete=\"one-time-code\""), "{shown}");
        let now = crate::server::now();
        let refused = site.answer(&ask("POST", "/account/code/setup", &first, format!("code={}", wrong_code(&pending, now)).as_bytes()));
        assert_eq!(refused.status, 422);
        let used = now / STEP;
        let activated = site.answer(&ask("POST", "/account/code/setup", &first, format!("code={:06}", code_at(&pending, used, 6)).as_bytes()));
        assert_eq!(activated.status,200);
        assert_eq!(text(activated).matches("data-recovery").count(),10);
        assert_eq!(secret_of(&site, "Ada", "code_secret"), pending);
        assert!(text(site.answer(&ask("GET", "/account?done=code", &first, b""))).contains("Le code à 6 chiffres est activé"));

        // Se déconnecter : la session est oubliée, le cookie effacé.
        let out = site.answer(&ask("POST", "/account/signout", &first, b""));
        assert_eq!((out.status, header(&out, "Location")), (303, "/account/signin?done=signedout"));
        assert!(out.headers.iter().any(|(name, value)| name == "Set-Cookie" && value.starts_with("holo_session=;") && value.contains("Max-Age=0")));
        assert_eq!(header(&site.answer(&ask("GET", "/account", &first, b"")), "Location"), "/account/signin");

        // Un mauvais mot de passe, un nom inconnu : le même message, qui ne dit pas lequel est faux.
        let alert = |reply: Reply| {
            let page = text(reply);
            page.split("role=\"alert\">").nth(1).and_then(|rest| rest.split('<').next()).unwrap_or("").to_string()
        };
        let wrong = alert(sign_in(&site, "", "Ada", "pas+le+bon+mot+de+passe"));
        let nobody = alert(sign_in(&site, "", "Personne", "pas+le+bon+mot+de+passe"));
        assert_eq!(wrong, "Ce nom et ce mot de passe ne vont pas ensemble.");
        assert_eq!(wrong, nobody);

        // Le bon mot de passe : il reste le code. La demi-session n'ouvre aucune page.
        let half = sign_in(&site, "", "Ada", "une+phrase+assez+longue");
        assert_eq!((half.status, header(&half, "Location")), (303, "/account/code"));
        let half = session(&half);
        assert_eq!(header(&site.answer(&ask("GET", "/account", &half, b"")), "Location"), "/account/signin");
        assert_eq!(header(&site.answer(&ask("GET", "/members.holo", &half, b"")), "Location"), "/account/signin?next=/members.holo&for=members");
        assert!(text(site.answer(&ask("GET", "/account/code", &half, b""))).contains("maxlength=\"40\""));
        // Un mauvais code ; le code déjà servi pour activer ; puis le suivant, qui ouvre la session.
        assert!(text(site.answer(&ask("POST", "/account/code", &half, format!("code={}", wrong_code(&pending, now)).as_bytes()))).contains("Ce code ne va pas"));
        assert!(text(site.answer(&ask("POST", "/account/code", &half, format!("code={:06}", code_at(&pending, used, 6)).as_bytes()))).contains("Ce code a déjà servi"));
        let whole = site.answer(&ask("POST", "/account/code", &half, format!("code={:06}", code_at(&pending, used + 1, 6)).as_bytes()));
        assert_eq!((whole.status, header(&whole, "Location")), (303, "/account"));
        let second = session(&whole);
        assert!(text(site.answer(&ask("GET", "/members.holo", &second, b""))).contains("Bonjour, <span data-state=\"account\">Ada</span>"));
        // La demi-session a été remplacée : elle ne sert plus.
        assert_eq!(header(&site.answer(&ask("POST", "/account/code", &half, format!("code={:06}", code_at(&pending, used + 1, 6)).as_bytes())), "Location"), "/account/signin");

        // Le frein : cinq mauvais mots de passe, puis une attente, même avec le bon.
        for _ in 0..FREE_TRIES {
            assert!(alert(sign_in(&site, "", "ada", "toujours+pas+le+bon")).contains("ne vont pas ensemble"));
        }
        let braked = alert(sign_in(&site, "", "Ada", "une+phrase+assez+longue"));
        assert!(braked.starts_with("Trop d'essais : attends 1 minute"), "{braked}");
        // Un nom inconnu a le même frein : l'attente ne dit pas si le nom existe.
        for _ in 0..FREE_TRIES {
            sign_in(&site, "", "Personne", "x");
        }
        assert!(alert(sign_in(&site, "", "Personne", "x")).starts_with("Trop d'essais"));
        // Le frein du code : cinq mauvais codes, puis une attente.
        site.base.lock().unwrap().execute("DELETE FROM attempts", []).unwrap();
        let half = session(&sign_in(&site, "", "Ada", "une+phrase+assez+longue"));
        for _ in 0..FREE_TRIES {
            site.answer(&ask("POST", "/account/code", &half, format!("code={}", wrong_code(&pending, now)).as_bytes()));
        }
        assert!(alert(site.answer(&ask("POST", "/account/code", &half, format!("code={:06}", code_at(&pending, used + 2, 6)).as_bytes()))).starts_with("Trop d'essais"));

        // Un formulaire de compte venu d'un autre site est refusé.
        let mut foreign = ask("POST", "/account/signin", "", b"name=Ada&password=une+phrase+assez+longue");
        foreign.origin = "https://ailleurs.example";
        assert_eq!(site.answer(&foreign).status, 403);
        // La base n'est jamais servie.
        assert_eq!(site.answer(&ask("GET", "/holo-data/site.sqlite", &second, b"")).status, 404);
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn a_page_for_members_and_a_cart_that_follows_the_account() {
        let (site, folder) = site();
        // Sans être connecté : la page réservée mène à « Se connecter », et rien ne passe.
        let away = site.answer(&ask("GET", "/members.holo", "", b""));
        assert_eq!((away.status, header(&away, "Location")), (303, "/account/signin?next=/members.holo&for=members"));
        let mut raw = ask("GET", "/members.holo", "", b"");
        raw.accept = "text/plain";
        assert_eq!(site.answer(&raw).status, 401);
        assert_eq!(site.answer(&ask("POST", "/members.holo", "", b"signal=Add.tap")).status, 401);
        let mut json = ask("POST", "/members.holo", "", br#"{"form":"Contact","values":{}}"#);
        json.content_type = "application/json";
        assert_eq!(site.answer(&json).status, 401);
        // La page « Se connecter » garde où revenir.
        let signin = text(site.answer(&ask("GET", "/account/signin?next=/members.holo&for=members", "", b"")));
        assert!(signin.contains("<input type=\"hidden\" name=\"next\" value=\"/members.holo\">") && signin.contains("autocomplete=\"current-password\""), "{signin}");
        assert!(signin.contains("<p class=\"done\" role=\"status\">Cette page est réservée aux membres : connecte-toi pour la voir.</p>"), "{signin}");

        // Avant de se connecter, sans JavaScript : deux tableaux dans le panier, sous un cookie de visiteur.
        let anonymous = site.answer(&ask("POST", "/cart.holo", "", b"signal=Add.tap"));
        let visitor = anonymous.headers.iter().find(|(name, _)| name == "Set-Cookie").map(|(_, value)| value.split(';').next().unwrap().to_string()).unwrap();
        site.answer(&ask("POST", "/cart.holo", &visitor, b"signal=Add.tap"));
        // Le compte est créé depuis cet appareil : le panier du visiteur suit le compte.
        let body = b"name=Grace&password=une+phrase+assez+longue&again=une+phrase+assez+longue&next=%2Fmembers.holo";
        let created = site.answer(&ask("POST", "/account/signup", &visitor, body));
        assert_eq!(header(&created, "Location"), "/members.holo");
        let phone = session(&created);
        let both = format!("{visitor}; {phone}");
        let members = site.answer(&ask("GET", "/members.holo", &both, b""));
        assert_eq!(header(&members, "Cache-Control"), "private, no-store");
        let members = text(members);
        assert!(members.contains("Bonjour, <span data-state=\"account\">Grace</span>") && members.contains("<meta name=\"holo-account\" content=\"Grace\">"), "{members}");
        assert!(text(site.answer(&ask("GET", "/cart.holo", &both, b""))).contains("Panier : <span data-state=\"cart\">2</span>"));
        // Un troisième toucher, depuis le téléphone, sans JavaScript.
        site.answer(&ask("POST", "/cart.holo", &both, b"signal=Add.tap"));
        // Sur l'ordinateur (une autre session, d'autres cookies) : le même panier.
        let computer = session(&sign_in(&site, "", "grace", "une+phrase+assez+longue"));
        assert_ne!(computer, phone);
        assert!(text(site.answer(&ask("GET", "/cart.holo", &computer, b""))).contains("Panier : <span data-state=\"cart\">3</span>"));
        // Avec JavaScript, le moteur a déjà joué le toucher : il le renvoie (`?mirror`), le serveur le
        // rejoue sur le panier du compte, avec le champ écrit, sans renvoyer la page.
        let mirrored = site.answer(&ask("POST", "/cart.holo?mirror", &computer, b"note=pour+Ada&signal=Add.tap"));
        assert_eq!((mirrored.status, header(&mirrored, "Location")), (204, ""));
        let cart = text(site.answer(&ask("GET", "/cart.holo", &phone, b"")));
        assert!(cart.contains("Panier : <span data-state=\"cart\">4</span>") && cart.contains("value=\"pour Ada\""), "{cart}");
        // Un toucher qui envoie un formulaire, renvoyé par le moteur : le serveur n'envoie rien (le
        // moteur l'a fait) ; sans JavaScript, il l'envoie lui-même.
        site.answer(&ask("POST", "/cart.holo?mirror", &computer, b"note=Grace&signal=Send.tap"));
        assert!(crate::server::messages(&folder).unwrap().is_empty());
        site.answer(&ask("POST", "/cart.holo", &computer, b"note=Grace&signal=Send.tap"));
        assert_eq!(crate::server::messages(&folder).unwrap().len(), 1);
        // Le visiteur d'avant la connexion n'a plus rien sous son cookie ; un autre visiteur part de zéro.
        assert!(text(site.answer(&ask("GET", "/cart.holo", &visitor, b""))).contains("Panier : <span data-state=\"cart\">0</span>"));
        assert!(text(site.answer(&ask("GET", "/cart.holo", "", b""))).contains("Panier : <span data-state=\"cart\">0</span>"));
        // Gardé dans la base : un nouveau serveur sur le même dossier retrouve le panier du compte.
        let web = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
        let reopened = Site::open(&folder, &web).unwrap();
        assert!(text(reopened.answer(&ask("GET", "/cart.holo", &phone, b""))).contains("Panier : <span data-state=\"cart\">4</span>"));
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn a_shared_value_that_only_members_change() {
        // Avec le lot 6 (ADR-079) : un bouton montré aux seuls membres ne se touche pas sans l'être,
        // car le serveur arbitre avec ce qu'il sait du visiteur, jamais avec ce que dit la page.
        let (site, folder) = site();
        std::fs::write(
            folder.join("book.holo"),
            "Page(title: \"Livre d'or\", state: State(signed: 0), shared: Shared(likes: 0), children: [ P(\"{likes}\"), If(signedIn, is: 1, children: [ Button(name: Like, text: \"J'aime\") ], else: [ P(\"Connecte-toi pour aimer.\") ]), Button(name: Sign, text: \"Signer\") ], rules: [ On(Like.tap, effect: likes.add(1)), On(Sign.tap, effect: signed.set(1)) ])",
        )
        .unwrap();
        std::fs::write(folder.join("vip.holo"), "Page(title: \"Membres\", access: members, shared: Shared(likes: 0), children: [ P(\"{likes}\"), Button(name: Like, text: \"+\") ], rules: [ On(Like.tap, effect: likes.add(1)) ])").unwrap();
        let json = |url: &'static str, cookie: &str, body: &'static [u8]| {
            let mut asked = ask("POST", url, cookie, body);
            asked.content_type = "application/json";
            site.answer(&asked)
        };
        // Un visiteur qui écrit `signedIn=1` dans l'état qu'il envoie : refusé, le bouton est caché pour lui.
        assert_eq!(json("/book.holo", "", br#"{"signal":"Like.tap","state":"signedIn=1;account='Admin"}"#).status, 409);
        // Sans JavaScript, de même : rien ne change.
        site.answer(&ask("POST", "/book.holo", "", b"signal=Like.tap"));
        let likes = |cookie: &str| text(site.answer(&ask("GET", "/book.holo", cookie, b""))).split("<span data-state=\"likes\">").nth(1).and_then(|rest| rest.split('<').next()).unwrap_or("?").to_string();
        assert_eq!(likes(""), "0");
        // Un membre : accepté, et son état est gardé sous son compte.
        let member = sign_up(&site, "Ada", "une+phrase+assez+longue");
        let accepted = json("/book.holo", &member, br#"{"signal":"Like.tap","state":""}"#);
        assert_eq!(accepted.status, 200, "{}", String::from_utf8_lossy(&accepted.body));
        site.answer(&ask("POST", "/book.holo", &member, b"signal=Like.tap"));
        assert_eq!(likes(""), "2");
        let kept: i64 = site.base.lock().unwrap().query_row("SELECT COUNT(*) FROM visits WHERE visitor LIKE 'account:%' AND page = '/book.holo'", [], |row| row.get(0)).unwrap();
        assert_eq!(kept, 1);
        // Une page réservée : ni son geste partagé ni son direct sans être membre.
        assert_eq!(json("/vip.holo", "", br#"{"signal":"Like.tap","state":""}"#).status, 401);
        let mut listen = ask("GET", "/vip.holo", "", b"");
        listen.accept = "text/event-stream";
        assert_eq!(site.live_page(&listen).err().map(|reply| reply.status), Some(401));
        let mut listen = ask("GET", "/vip.holo", &member, b"");
        listen.accept = "text/event-stream";
        assert!(site.live_page(&listen).is_ok());
        assert_eq!(json("/vip.holo", &member, br#"{"signal":"Like.tap","state":""}"#).status, 200);
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn a_members_page_with_its_title_and_its_address() {
        // Avec le lot 9 : sur une page réservée, le titre lit les valeurs (ADR-090) et l'adresse
        // porte des valeurs (ADR-091).
        let (site, folder) = site();
        std::fs::write(
            folder.join("carnet.holo"),
            "Page(title: \"Le carnet de {account} : page {page}\", access: members, state: State(page: 1, seen: 0), address: [page], children: [ P(\"Page {page}, vue {seen}\"), Button(name: Next, text: \"Suivante\") ], rules: [ On(Next.tap, effect: [page.add(1), seen.set(page)]) ])",
        )
        .unwrap();
        // Sans être membre : « Se connecter », qui ramènera à l'adresse avec ses valeurs.
        let away = site.answer(&ask("GET", "/carnet.holo?page=3", "", b""));
        assert_eq!(header(&away, "Location"), "/account/signin?next=/carnet.holo%3Fpage%3D3&for=members");
        let created = site.answer(&ask("POST", "/account/signup", "", b"name=Ada&password=une+phrase+assez+longue&again=une+phrase+assez+longue&next=%2Fcarnet.holo%3Fpage%3D3"));
        assert_eq!(header(&created, "Location"), "/carnet.holo?page=3");
        let ada = session(&created);
        let page = text(site.answer(&ask("GET", "/carnet.holo?page=3", &ada, b"")));
        assert!(page.contains("<title>Le carnet de Ada : page 3</title>") && page.contains("Page <span data-state=\"page\">3</span>"), "{page}");
        // Le toucher renvoyé par le moteur porte l'adresse d'avant : le serveur part de la page 3.
        assert_eq!(site.answer(&ask("POST", "/carnet.holo?page=3&mirror", &ada, b"signal=Next.tap")).status, 204);
        let kept: String = site.base.lock().unwrap().query_row("SELECT state FROM visits WHERE visitor LIKE 'account:%' AND page = '/carnet.holo'", [], |row| row.get(0)).unwrap();
        assert!(kept.split(';').any(|part| part == "seen=4"), "{kept}");
        // Sans JavaScript, le toucher mène à l'adresse des nouvelles valeurs.
        let touched = site.answer(&ask("POST", "/carnet.holo?page=4", &ada, b"signal=Next.tap"));
        assert_eq!(header(&touched, "Location"), "/carnet.holo?page=5");
        // Le titre suit le visiteur : l'adresse nue, la page 1 ; la vue gardée par le compte.
        let start = text(site.answer(&ask("GET", "/carnet.holo", &ada, b"")));
        assert!(start.contains("<title>Le carnet de Ada : page 1</title>") && start.contains("vue <span data-state=\"seen\">5</span>"), "{start}");
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn a_stolen_session_expires() {
        let (site, folder) = site();
        let cookie = sign_up(&site, "Ada", "une+phrase+assez+longue");
        let token = cookie.trim_start_matches("holo_session=").to_string();
        let member = || member_of(&site, &cookie);
        assert_eq!(member().map(|m| m.name), Some("Ada".to_string()));
        // La base ne garde que l'empreinte du numéro : une copie de la base ne permet pas d'entrer.
        let kept: Vec<String> = site.base.lock().unwrap().prepare("SELECT fingerprint FROM sessions").unwrap().query_map([], |row| row.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(kept, [fingerprint(&token)]);
        assert!(member_of(&site, &format!("holo_session={}", kept[0].get(..32).unwrap())).is_none());
        // Un numéro inventé n'est personne.
        assert!(member_of(&site, &format!("holo_session={}", random_hex(16))).is_none());
        // Quatorze jours sans visite : oubliée.
        let now = crate::server::now() as i64;
        site.base.lock().unwrap().execute("UPDATE sessions SET seen = ?1", params![now - SESSION_IDLE as i64 - 10]).unwrap();
        assert!(member().is_none());
        assert_eq!(site.base.lock().unwrap().query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        // Trente jours après la connexion, même si on revient chaque jour : oubliée.
        let again = session(&sign_in(&site, "", "Ada", "une+phrase+assez+longue"));
        site.base.lock().unwrap().execute("UPDATE sessions SET created = ?1", params![now - SESSION_MAX as i64 - 10]).unwrap();
        assert!(member_of(&site, &again).is_none());
        // Se déconnecter efface la session de la base.
        let third = session(&sign_in(&site, "", "Ada", "une+phrase+assez+longue"));
        assert!(member_of(&site, &third).is_some());
        site.answer(&ask("POST", "/account/signout", &third, b""));
        assert!(member_of(&site, &third).is_none());
        assert_eq!(site.base.lock().unwrap().query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        // Ce qu'il faut pour revenir : une adresse de ce site, jamais d'un autre.
        let reply = site.answer(&ask("POST", "/account/signin", "", b"name=Ada&password=une+phrase+assez+longue&next=%2F%2Failleurs.example"));
        assert_eq!(header(&reply, "Location"), "/account");
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn where_to_go_back() {
        assert_eq!(safe_next("/105-une-page.holo").as_deref(), Some("/105-une-page.holo"));
        for refused in ["//ailleurs.example/x", "https://ailleurs.example/", "/a\\b", "x.holo", "/a\"b", "/a b"] {
            assert!(safe_next(refused).is_none(), "{refused}");
        }
        assert_eq!(with_next("/profils/Ad%C3%A9"), "?next=/profils/Ad%25C3%25A9");
        assert_eq!(wait_message(61), "Trop d'essais : attends 2 minutes avant de réessayer.");
        assert_eq!(wait_message(30), "Trop d'essais : attends 1 minute avant de réessayer.");
    }
    #[test]
    fn recoveries_are_shown_once_hashed_and_single_use_after_password() {
        let (site,folder)=site();let password="une phrase locale de secours";
        let cookie=sign_up(&site,"Rescue",password);
        assert_eq!(site.answer(&ask("POST","/account/code/setup",&cookie,b"")).status,303);
        let secret=secret_of(&site,"Rescue","code_pending");
        let now=crate::server::now();
        let reply=site.answer(&ask("POST","/account/code/setup",&cookie,format!("code={:06}",code_at(&secret,now/STEP,6)).as_bytes()));
        assert_eq!(reply.status,200);
        let html=text(reply);
        let clear:Vec<String>=html.split("<code data-recovery>").skip(1).map(|c|c.split("</code>").next().unwrap().to_string()).collect();
        assert_eq!(clear.len(),10);assert_eq!(clear.iter().collect::<std::collections::HashSet<_>>().len(),10);
        assert!(!text(site.answer(&ask("GET","/account/code/setup",&cookie,b""))).contains("data-recovery"));
        {
            let base=site.base.lock().unwrap();
            let mut q=base.prepare("SELECT fingerprint FROM recoveries").unwrap();
            let prints=q.query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
            assert_eq!(prints.len(),10);assert!(!prints.iter().any(|p|clear.iter().any(|c|c.replace('-',"")==*p)));
        }
        site.answer(&ask("POST","/account/signout",&cookie,b""));
        let first=site.answer(&ask("POST","/account/signin","",format!("name=Rescue&password={password}").as_bytes()));let waiting=session(&first);
        assert_eq!(header(&first,"Location"),"/account/code");
        let accepted=site.answer(&ask("POST","/account/code",&waiting,format!("code={}",clear[0]).as_bytes()));
        assert_eq!(accepted.status,303);assert!(!session(&accepted).is_empty());
        let next=site.answer(&ask("POST","/account/signin","",format!("name=Rescue&password={password}").as_bytes()));let waiting=session(&next);
        assert_eq!(site.answer(&ask("POST","/account/code",&waiting,format!("code={}",clear[0]).as_bytes())).status,401);
        assert_eq!(site.base.lock().unwrap().query_row("SELECT COUNT(*) FROM recoveries",[],|r|r.get::<_,i64>(0)).unwrap(),9);
        drop(site);std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn ip_limits_ignore_changed_account_names_and_expire() {
        let base=Connection::open_in_memory().unwrap();prepare(&base,1000).unwrap();
        for _ in 0..IP_REQUESTS_MAX {assert!(ip_allowed(&base,"192.0.2.1",1000));}
        assert!(!ip_allowed(&base,"192.0.2.1",1001));assert!(ip_allowed(&base,"192.0.2.2",1001));
        assert!(ip_allowed(&base,"192.0.2.1",1060));assert!(!ip_allowed(&base,"not-an-ip",1060));
    }
    #[test]
    fn deleting_requires_confirmation_and_cleans_live_data_backups_and_files() {
        let (site,folder)=site();let password="une phrase locale pour effacer";
        let cookie=sign_up(&site,"Erase",password);let member=member_of(&site,&cookie).unwrap();
        let other=sign_up(&site,"Remain",password);let bob=member_of(&site,&other).unwrap();
        let data=folder.join(crate::server::DATA_FOLDER);
        std::fs::create_dir_all(data.join("files").join("cart")).unwrap();
        std::fs::write(data.join("files/cart/photo.png"),b"personal").unwrap();
        {
            let base=site.base.lock().unwrap();
            base.execute("INSERT INTO visits(visitor,page,state,updated) VALUES(?1,'/cart.holo','cart=2',1)",params![member.visit_key()]).unwrap();
            base.execute("INSERT INTO messages(received,page,form,submission,files,account) VALUES(1,'/cart.holo','Order','{}',?1,?2)",params![r#"[{"file":"files/cart/photo.png"}]"#,member.id]).unwrap();
            new_recoveries(&base,member.id,b"secret").unwrap();
        }
        let backup=crate::server::backup(&folder).unwrap();
        assert_eq!(site.answer(&ask("GET","/account/delete",&cookie,b"")).status,200);
        assert_eq!(site.answer(&ask("POST","/account/delete",&cookie,b"confirm=Erase&password=wrong")).status,422);
        assert!(member_of(&site,&cookie).is_some());
        let body=format!("confirm=Erase&password={password}");
        let erased=site.answer(&ask("POST","/account/delete",&cookie,body.as_bytes()));assert_eq!(erased.status,200);
        assert!(member_of(&site,&cookie).is_none());assert!(member_of(&site,&other).is_some());
        assert!(!data.join("files/cart/photo.png").exists());
        for db in [Connection::open(data.join("site.sqlite")).unwrap(),Connection::open(&backup).unwrap()] {
            for sql in ["SELECT COUNT(*) FROM accounts WHERE id=?1","SELECT COUNT(*) FROM sessions WHERE account=?1","SELECT COUNT(*) FROM recoveries WHERE account=?1","SELECT COUNT(*) FROM messages WHERE account=?1"]{
                assert_eq!(db.query_row(sql,params![member.id],|r|r.get::<_,i64>(0)).unwrap(),0, "{sql}");
            }
            assert_eq!(db.query_row("SELECT COUNT(*) FROM visits WHERE visitor=?1",params![member.visit_key()],|r|r.get::<_,i64>(0)).unwrap(),0);
            assert_eq!(db.query_row("SELECT COUNT(*) FROM accounts WHERE id=?1",params![bob.id],|r|r.get::<_,i64>(0)).unwrap(),1);
        }
        let new=sign_up(&site,"Erase",password);assert!(member_of(&site,&new).unwrap().id>member.id);
        drop(site);std::fs::remove_dir_all(folder).unwrap();
    }

}
