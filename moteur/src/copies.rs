//! Les copies des modules venus d'ailleurs (ADR-118), sur le PC seulement.
//!
//! - `holo check` vérifie la copie rangée à côté de la page : son poids (4 Mo au plus), son
//!   empreinte si la page en écrit une, et ce qu'elle demande à la boîte (`modules::check_wasm`).
//!   Il dit son poids, son contrat, sa licence et d'où elle vient.
//! - `holo serve` télécharge une fois la copie qui manque, si une page du même dossier la déclare
//!   avec son adresse (`from`) : par le chemin sûr des autres sites (`remote::download`), puis il
//!   la vérifie comme `holo check`, la range à côté de la page, et la sert. Une copie qui ne
//!   correspond pas n'est jamais rangée. Le navigateur du visiteur ne va jamais chez l'autre site :
//!   il ne reçoit que la copie de l'auteur, et la vérifie encore avec son empreinte.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Mutex;

use sha2::{Digest, Sha256};

use crate::modules::{check_wasm, Module, Offer, BYTES_MAX};

/// Les pages regardées dans un dossier, au plus, pour trouver celle qui déclare une copie.
const PAGES_SEEN_MAX: usize = 500;
/// Après un téléchargement raté, une adresse n'est pas redemandée avant une minute (en ms).
pub const RETRY_AFTER: u64 = 60_000;

/// L'empreinte d'un fichier, en 64 chiffres hexadécimaux (minuscules).
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Un poids, dit en français : « 205 octets », « 7,1 Ko », « 3,4 Mo ».
pub fn weight(bytes: usize) -> String {
    let decimal = |value: f64| {
        let written = format!("{value:.1}").replace('.', ",");
        written.strip_suffix(",0").map_or(written.clone(), str::to_string)
    };
    match bytes {
        0..=999 => format!("{bytes} octets"),
        1_000..=999_999 => format!("{} Ko", decimal(bytes as f64 / 1e3)),
        _ => format!("{} Mo", decimal(bytes as f64 / 1e6)),
    }
}

/// Vérifie une copie (ADR-118) : 4 Mo au plus, son empreinte si la page en écrit une, et ce
/// qu'elle demande à la boîte. Rend son contrat, ou la raison du refus.
pub fn check_copy(module: &Module, bytes: &[u8]) -> Result<Offer, String> {
    if bytes.len() > BYTES_MAX {
        return Err(format!("« {} » pèse plus de 4 Mo : un module pèse 4 Mo au plus", module.source));
    }
    if let Some(expected) = module.sha256 {
        let actual = sha256_hex(bytes);
        if !actual.eq_ignore_ascii_case(expected) {
            let what_to_do = if module.from.is_some() { "efface-le : holo serve retéléchargera le bon depuis son adresse" } else { "remets le fichier d'origine" };
            return Err(format!(
                "l'empreinte de « {} » ne correspond pas : ce fichier n'est pas celui dont la page écrit l'empreinte, et le moteur ne le lancera pas. Si tu ne l'as pas changé toi-même, {what_to_do}. Si tu l'as remplacé exprès par une version que tu as vérifiée, son empreinte est : {actual}",
                module.source
            ));
        }
    }
    check_wasm(bytes, module.pages).map_err(|reason| format!("« {} » : {reason}", module.source))
}

/// Lit un fichier de module, 4 Mo au plus : au-delà, il n'est pas lu plus loin.
fn read_module(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?.take(BYTES_MAX as u64 + 1).read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Ce que `holo check` dit d'un module de la page (ADR-118) : une ligne (son poids, son contrat,
/// sa licence, son empreinte vérifiée, d'où il vient), ou la raison qui le refuse. `folder` : le
/// dossier de la page.
pub fn report(module: &Module, folder: &Path) -> Result<String, String> {
    let Ok(bytes) = read_module(&folder.join(module.source)) else {
        return match module.from {
            Some(from) => Ok(format!("module {} : pas encore là ; holo serve le téléchargera une fois depuis {from}, vérifiera son empreinte et le rangera à côté de la page", module.source)),
            None => Err(format!("le fichier du module « {} » manque à côté de la page", module.source)),
        };
    };
    let offer = check_copy(module, &bytes)?;
    let mut said = vec![weight(bytes.len()), (if offer == Offer::First { "premier contrat, run(nombre)" } else { "second contrat, alloc et run(adresse, taille)" }).to_string()];
    match (module.license, module.sha256) {
        (Some(license), Some(_)) => said.extend([format!("licence : {license}"), "empreinte vérifiée".to_string()]),
        (Some(license), None) => said.push(format!("licence : {license}")),
        (None, _) => said.push(format!("sans empreinte ni licence (un module à toi) ; son empreinte, s'il venait d'ailleurs : {}", sha256_hex(&bytes))),
    }
    if let Some(from) = module.from {
        said.push(format!("venu de {from}"));
    }
    Ok(format!("module {} : {}", module.source, said.join(" ; ")))
}

/// Ce qu'une page déclare d'une copie qui manque : de quoi la télécharger et la vérifier.
struct Wanted {
    page: String,
    from: String,
    sha256: String,
    license: String,
    pages: u64,
}

/// La page du dossier qui déclare `file` avec une adresse (`from`), s'il y en a une.
fn declared(folder: &Path, file: &str) -> Option<Wanted> {
    let mut entries: Vec<_> = std::fs::read_dir(folder).ok()?.flatten().collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries.into_iter().take(PAGES_SEEN_MAX) {
        let page = entry.file_name().to_string_lossy().into_owned();
        if page.starts_with('.') || !page.ends_with(".holo") || !entry.metadata().is_ok_and(|meta| meta.is_file() && meta.len() <= 262_144) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(entry.path()) else { continue };
        let Ok(program) = crate::holo::read(&text) else { continue };
        let Ok(modules) = crate::modules::modules(&program) else { continue };
        if let Some(module) = modules.iter().find(|module| module.source == file && module.from.is_some()) {
            return Some(Wanted {
                page,
                from: module.from?.to_string(),
                sha256: module.sha256?.to_ascii_lowercase(),
                license: module.license.unwrap_or_default().to_string(),
                pages: module.pages,
            });
        }
    }
    None
}

/// Range une copie vérifiée à côté de la page : d'abord dans un fichier caché (jamais servi), puis
/// renommée d'un coup, pour qu'une copie à moitié écrite ne soit jamais servie. Un fichier qui
/// existe déjà n'est jamais remplacé.
fn keep(folder: &Path, file: &str, bytes: &[u8]) -> Result<(), String> {
    let target = folder.join(file);
    if target.exists() {
        return Err(format!("« {file} » existe déjà : il n'est jamais remplacé"));
    }
    let part = folder.join(format!(".{file}.part"));
    let _ = std::fs::remove_file(&part);
    let written = std::fs::OpenOptions::new().write(true).create_new(true).open(&part).and_then(|mut out| {
        out.write_all(bytes)?;
        out.sync_all()
    });
    if let Err(error) = written.and_then(|()| std::fs::rename(&part, &target)) {
        let _ = std::fs::remove_file(&part);
        return Err(format!("impossible de ranger « {file} » à côté de la page ({error}) : télécharge-le toi-même"));
    }
    Ok(())
}

/// Les copies que holo serve télécharge (ADR-118). Une seule à la fois : une autre demande, pendant
/// ce temps, reçoit 404 tout de suite, sans retenir un fil du serveur. Un échec est gardé une
/// minute par adresse : aucun visiteur ne fait marteler l'autre site.
pub struct Copies {
    download: Box<dyn Fn(&str) -> Result<Vec<u8>, String> + Send + Sync>,
    clock: Box<dyn Fn() -> u64 + Send + Sync>,
    log: Box<dyn Fn(&str) + Send + Sync>,
    busy: Mutex<()>,
    failed: Mutex<HashMap<String, u64>>,
}

impl Copies {
    /// Le vrai téléchargement, par `remote::download` ; l'interrupteur des essais s'il est allumé.
    pub fn open(fake: Option<crate::remote::FakeSite>) -> Copies {
        let start = std::time::Instant::now();
        Copies::with_parts(
            Box::new(move |address| crate::remote::download(address, BYTES_MAX, fake.as_ref())),
            Box::new(move || start.elapsed().as_millis() as u64),
            Box::new(|line| println!("{line}")),
        )
    }

    /// Les mêmes, avec un téléchargement, une horloge et un journal à soi (les essais).
    pub fn with_parts(download: Box<dyn Fn(&str) -> Result<Vec<u8>, String> + Send + Sync>, clock: Box<dyn Fn() -> u64 + Send + Sync>, log: Box<dyn Fn(&str) + Send + Sync>) -> Copies {
        Copies { download, clock, log, busy: Mutex::new(()), failed: Mutex::default() }
    }

    /// La copie d'un module qui manque, demandée à `path` (comme `/lecons/141-premiers.wasm`) dans
    /// le dossier servi `folder`. Si une page du même dossier déclare ce fichier avec son adresse et
    /// son empreinte, il est téléchargé une fois, vérifié comme par holo check, rangé à côté
    /// d'elle, puis rendu. Rien sinon : le serveur répond 404, et la page reçoit `failed`.
    pub fn missing(&self, folder: &Path, path: &str) -> Option<Vec<u8>> {
        let (dir, file) = path.rsplit_once('/')?;
        if !file.ends_with(".wasm") || file.starts_with('.') || !crate::flat::path_on(file) || file.contains('/') {
            return None;
        }
        // Les mêmes refus que pour servir un fichier : ni dossier caché, ni la base.
        let parts: Vec<&str> = dir.split('/').filter(|part| !part.is_empty()).collect();
        if parts.iter().any(|part| part.starts_with('.') || part.contains(['\\', ':'])) || parts.first() == Some(&crate::server::DATA_FOLDER) {
            return None;
        }
        let here = parts.iter().fold(folder.to_path_buf(), |path, part| path.join(part));
        let wanted = declared(&here, file)?;
        let Ok(_one) = self.busy.try_lock() else { return None };
        // Une autre demande vient peut-être de la ranger.
        if let Ok(bytes) = read_module(&here.join(file)) {
            return Some(bytes);
        }
        let now = (self.clock)();
        if self.failed.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).get(&wanted.from).is_some_and(|at| now < at + RETRY_AFTER) {
            return None;
        }
        let module = Module { name: "", source: file, inputs: Vec::new(), outputs: Vec::new(), time: 0, pages: wanted.pages, sha256: Some(&wanted.sha256), from: Some(&wanted.from), license: Some(&wanted.license) };
        let outcome = (self.download)(&wanted.from).and_then(|bytes| {
            check_copy(&module, &bytes).map_err(|reason| if reason.starts_with("l'empreinte") { format!("l'empreinte du fichier téléchargé ne correspond pas à celle que la page écrit (la sienne : {})", sha256_hex(&bytes)) } else { reason })?;
            keep(&here, file, &bytes)?;
            Ok(bytes)
        });
        let shown = format!("{dir}/{file}");
        match outcome {
            Ok(bytes) => {
                (self.log)(&format!(
                    "Module          : {shown} : téléchargé une fois depuis {}, empreinte vérifiée, {}, licence : {} ; la copie est rangée à côté de {} : garde-la avec ton projet",
                    wanted.from,
                    weight(bytes.len()),
                    wanted.license,
                    wanted.page
                ));
                Some(bytes)
            }
            Err(reason) => {
                self.failed.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).insert(wanted.from.clone(), now);
                (self.log)(&format!("Module          : {shown} : {reason} ; rien n'est rangé, la page recevra « failed »"));
                None
            }
        }
    }
}

#[cfg(test)]
mod copies_tests {
    use super::{sha256_hex, weight, Copies, RETRY_AFTER};
    use crate::modules::foreign_tests::asking_for_the_network;
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};

    const PRIMES: &[u8] = include_bytes!("../../exemples/lecons/141-premiers.wasm");
    const FROM: &str = "https://modules.exemple.org/premiers/1.0/premiers.wasm";

    fn folder() -> PathBuf {
        let mut bytes = [0u8; 8];
        getrandom::getrandom(&mut bytes).unwrap();
        let folder = std::env::temp_dir().join(format!("holo-copies-{}", bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()));
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    /// Une page qui emploie `premiers.wasm`, ses paramètres en plus de name, source, input et output.
    fn page(extra: &str) -> String {
        format!("module \"premiers.wasm\"\nPage(state: State(n: 100, total: 0), modules: [ Module(name: Primes, source: \"premiers.wasm\", input: n, output: total{extra}) ], children: [ P(\"{{total}}\") ])")
    }

    fn report(folder: &Path, extra: &str) -> Result<Vec<String>, String> {
        let program = crate::check_page(&page(extra)).unwrap();
        crate::check_module_files(&program, folder).map_err(|error| error.message)
    }

    #[test]
    fn holo_check_says_the_weight_the_license_and_refuses_a_copy_that_changed() {
        let here = folder();
        let pinned = format!(", from: \"{FROM}\", sha256: \"{}\", license: \"MIT\"", sha256_hex(PRIMES));
        // Pas encore là : holo serve ira la chercher ; sans adresse, il manque un fichier.
        assert_eq!(report(&here, &pinned).unwrap(), vec![format!("module premiers.wasm : pas encore là ; holo serve le téléchargera une fois depuis {FROM}, vérifiera son empreinte et le rangera à côté de la page")]);
        assert!(report(&here, "").unwrap_err().contains("le fichier du module « premiers.wasm » manque à côté de la page"));
        // La bonne copie : son poids, son contrat, sa licence, son empreinte vérifiée, d'où elle vient.
        std::fs::write(here.join("premiers.wasm"), PRIMES).unwrap();
        assert_eq!(report(&here, &pinned).unwrap(), vec![format!("module premiers.wasm : 205 octets ; premier contrat, run(nombre) ; licence : MIT ; empreinte vérifiée ; venu de {FROM}")]);
        assert!(report(&here, "").unwrap()[0].contains(&format!("sans empreinte ni licence (un module à toi) ; son empreinte, s'il venait d'ailleurs : {}", sha256_hex(PRIMES))));
        // Un octet de changé : refusée, avec ce qu'il faut faire, et l'empreinte du fichier.
        let mut changed = PRIMES.to_vec();
        *changed.last_mut().unwrap() ^= 1;
        std::fs::write(here.join("premiers.wasm"), &changed).unwrap();
        let refusal = report(&here, &pinned).unwrap_err();
        assert!(refusal.starts_with("l'empreinte de « premiers.wasm » ne correspond pas") && refusal.contains("efface-le : holo serve retéléchargera le bon") && refusal.ends_with(&sha256_hex(&changed)), "{refusal}");
        assert!(report(&here, &format!(", sha256: \"{}\", license: \"MIT\"", sha256_hex(PRIMES))).unwrap_err().contains("remets le fichier d'origine"));
        // Un module qui demande le réseau : refusé, même avec la bonne empreinte.
        let network = asking_for_the_network();
        std::fs::write(here.join("premiers.wasm"), &network).unwrap();
        let refusal = report(&here, &format!(", sha256: \"{}\", license: \"MIT\"", sha256_hex(&network))).unwrap_err();
        assert!(refusal.contains("« premiers.wasm » : le module demande « env.fetch » (une fonction)"), "{refusal}");
        // Plus de 4 Mo : refusé sans être lu plus loin.
        std::fs::write(here.join("premiers.wasm"), vec![0u8; 4_000_001]).unwrap();
        assert!(report(&here, "").unwrap_err().contains("pèse plus de 4 Mo"));
        // L'erreur porte la ligne du Module(…), pour l'éditeur.
        let program = crate::check_page(&page("")).unwrap();
        assert_eq!(crate::check_module_files(&program, &here).unwrap_err().pos.line, 2);
        assert_eq!((weight(205), weight(7128), weight(7000), weight(3_400_000)), ("205 octets".into(), "7,1 Ko".into(), "7 Ko".into(), "3,4 Mo".into()));
        let _ = std::fs::remove_dir_all(here);
    }

    /// Un holo serve d'essai : ses téléchargements sont un faux, qui note chaque adresse demandée ;
    /// son horloge s'avance à la main ; son journal est gardé.
    struct Bench {
        copies: Copies,
        asked: Arc<Mutex<Vec<String>>>,
        journal: Arc<Mutex<Vec<String>>>,
        now: Arc<AtomicU64>,
    }

    fn bench(answer: impl Fn(&str) -> Result<Vec<u8>, String> + Send + Sync + 'static) -> Bench {
        let (asked, journal, now) = (Arc::new(Mutex::new(Vec::new())), Arc::new(Mutex::new(Vec::new())), Arc::new(AtomicU64::new(0)));
        let (noted, said, clock) = (Arc::clone(&asked), Arc::clone(&journal), Arc::clone(&now));
        let copies = Copies::with_parts(
            Box::new(move |address| {
                noted.lock().unwrap().push(address.to_string());
                answer(address)
            }),
            Box::new(move || clock.load(Ordering::SeqCst)),
            Box::new(move |line| said.lock().unwrap().push(line.to_string())),
        );
        Bench { copies, asked, journal, now }
    }

    #[test]
    fn holo_serve_downloads_a_missing_copy_once_checks_it_and_keeps_it_next_to_the_page() {
        let here = folder();
        std::fs::create_dir_all(here.join("lecons")).unwrap();
        std::fs::write(here.join("lecons").join("primes.holo"), page(&format!(", from: \"{FROM}\", sha256: \"{}\", license: \"MIT\"", sha256_hex(PRIMES)))).unwrap();
        let b = bench(|_| Ok(PRIMES.to_vec()));
        // Demandée par le navigateur : téléchargée une fois, rangée à côté de la page, servie.
        assert_eq!(b.copies.missing(&here, "/lecons/premiers.wasm").as_deref(), Some(PRIMES));
        assert_eq!(std::fs::read(here.join("lecons").join("premiers.wasm")).unwrap(), PRIMES);
        assert_eq!(*b.asked.lock().unwrap(), vec![FROM.to_string()]);
        assert_eq!(
            b.journal.lock().unwrap()[0],
            format!("Module          : /lecons/premiers.wasm : téléchargé une fois depuis {FROM}, empreinte vérifiée, 205 octets, licence : MIT ; la copie est rangée à côté de primes.holo : garde-la avec ton projet")
        );
        // Aucun fichier caché ne reste ; une seconde demande lit la copie, sans rien télécharger.
        assert!(!here.join("lecons").join(".premiers.wasm.part").exists());
        assert_eq!(b.copies.missing(&here, "/lecons/premiers.wasm").as_deref(), Some(PRIMES));
        assert_eq!(b.asked.lock().unwrap().len(), 1);
        // Rien pour un fichier qu'aucune page ne déclare avec une adresse, ni hors du site servi.
        for path in ["/lecons/autre.wasm", "/premiers.wasm", "/lecons/premiers.js", "/.cache/premiers.wasm", "/holo-data/premiers.wasm", "/lecons/.premiers.wasm"] {
            assert_eq!(b.copies.missing(&here, path), None, "{path}");
        }
        assert_eq!(b.asked.lock().unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(here);
    }

    #[test]
    fn a_downloaded_copy_that_does_not_match_is_never_kept_and_the_other_site_is_not_hammered() {
        let here = folder();
        std::fs::write(here.join("primes.holo"), page(&format!(", from: \"{FROM}\", sha256: \"{}\", license: \"MIT\"", sha256_hex(PRIMES)))).unwrap();
        // L'autre site sert un autre fichier (sa version a changé en silence) : rien n'est rangé.
        let b = bench(|_| Ok(asking_for_the_network()));
        assert_eq!(b.copies.missing(&here, "/premiers.wasm"), None);
        assert!(!here.join("premiers.wasm").exists() && !here.join(".premiers.wasm.part").exists());
        let said = b.journal.lock().unwrap()[0].clone();
        assert!(said.starts_with("Module          : /premiers.wasm : l'empreinte du fichier téléchargé ne correspond pas à celle que la page écrit (la sienne : ") && said.ends_with("rien n'est rangé, la page recevra « failed »"), "{said}");
        // Pendant une minute, plus aucune demande à l'autre site, quel que soit le nombre de visiteurs.
        for _ in 0..5 {
            assert_eq!(b.copies.missing(&here, "/premiers.wasm"), None);
        }
        assert_eq!(b.asked.lock().unwrap().len(), 1);
        b.now.store(RETRY_AFTER, Ordering::SeqCst);
        assert_eq!(b.copies.missing(&here, "/premiers.wasm"), None);
        assert_eq!(b.asked.lock().unwrap().len(), 2);
        // Un fichier qui a la bonne empreinte mais demande le réseau : refusé par la boîte, jamais rangé.
        let network = asking_for_the_network();
        std::fs::write(here.join("primes.holo"), page(&format!(", from: \"{FROM}\", sha256: \"{}\", license: \"MIT\"", sha256_hex(&network)))).unwrap();
        b.now.store(2 * RETRY_AFTER, Ordering::SeqCst);
        assert_eq!(b.copies.missing(&here, "/premiers.wasm"), None);
        assert!(b.journal.lock().unwrap().last().unwrap().contains("le module demande « env.fetch » (une fonction)"));
        assert!(!here.join("premiers.wasm").exists());
        // L'autre site ne répond pas : dit au journal, rien de rangé.
        let silent = bench(|_| Err("modules.exemple.org est introuvable (nom inconnu, ou pas de réseau)".to_string()));
        assert_eq!(silent.copies.missing(&here, "/premiers.wasm"), None);
        assert!(silent.journal.lock().unwrap()[0].contains("introuvable"));
        // Un téléchargement en cours : une autre demande reçoit 404 tout de suite, sans attendre.
        let busy = bench(|_| Ok(PRIMES.to_vec()));
        let held = busy.copies.busy.lock().unwrap();
        assert_eq!(busy.copies.missing(&here, "/premiers.wasm"), None);
        drop(held);
        assert!(busy.asked.lock().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(here);
    }

    /// Un faux « autre site » sur ce PC, en HTTP clair (atteint par l'interrupteur des essais) : il
    /// répond selon le chemin demandé, et garde chaque demande reçue.
    fn other_site(answer: impl Fn(&str) -> Vec<u8> + Send + 'static) -> (u16, Arc<Mutex<Vec<String>>>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let noted = Arc::clone(&seen);
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let (mut request, mut buffer) = (Vec::new(), [0u8; 1024]);
                while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                    match stream.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => request.extend(&buffer[..n]),
                    }
                }
                let request = String::from_utf8_lossy(&request).into_owned();
                let path = request.split_whitespace().nth(1).unwrap_or("").to_string();
                noted.lock().unwrap().push(request);
                let _ = stream.write_all(&answer(&path));
            }
        });
        (port, seen)
    }

    fn reply(head: &str, body: &[u8]) -> Vec<u8> {
        [format!("HTTP/1.1 {head}\r\nConnection: close\r\n\r\n").into_bytes(), body.to_vec()].concat()
    }

    #[test]
    fn the_download_takes_the_safe_road_of_the_other_sites() {
        let (port, seen) = other_site(|path| match path {
            "/premiers.wasm" => reply(&format!("200 OK\r\nContent-Type: application/wasm\r\nContent-Length: {}", PRIMES.len()), PRIMES),
            "/redirige.wasm" => reply("302 Found\r\nLocation: /premiers.wasm\r\nContent-Length: 0", b""),
            "/boucle.wasm" => reply("302 Found\r\nLocation: /boucle.wasm\r\nContent-Length: 0", b""),
            "/gros.wasm" => reply("200 OK\r\nContent-Length: 4000001", b""),
            "/sans-taille.wasm" => reply("200 OK", &vec![0u8; 4_000_100]),
            _ => reply("404 Not Found\r\nContent-Length: 0", b""),
        });
        let fake = crate::remote::FakeSite { host: "modules.test".into(), port };
        let download = |path: &str| crate::remote::download(&format!("https://modules.test{path}"), crate::modules::BYTES_MAX, Some(&fake));
        assert_eq!(download("/premiers.wasm").unwrap(), PRIMES);
        // Ce que l'autre site reçoit : notre User-Agent, ce qu'on attend, et rien du visiteur.
        let first = seen.lock().unwrap()[0].to_ascii_lowercase();
        assert!(first.starts_with("get /premiers.wasm http/1.1") && first.contains("user-agent: holocode/") && first.contains("accept: application/wasm") && !first.contains("cookie") && !first.contains("referer"), "{first}");
        // Une redirection est suivie (le fichier est épinglé par son empreinte), cinq au plus.
        assert_eq!(download("/redirige.wasm").unwrap(), PRIMES);
        assert_eq!(download("/boucle.wasm").unwrap_err(), "modules.test redirige plus de 5 fois");
        // Plus de 4 Mo : annoncé, refusé avant de lire ; sans taille annoncée, coupé.
        assert!(download("/gros.wasm").unwrap_err().contains("envoie plus de 4 Mo : coupé, rien n'est pris"));
        assert!(download("/sans-taille.wasm").unwrap_err().contains("envoie plus de 4 Mo : coupé, rien n'est pris"));
        assert_eq!(download("/absent.wasm").unwrap_err(), "modules.test a répondu 404 au lieu de 200");
        // L'adresse est lue strictement, comme celle des données : http://, une adresse IP…
        assert!(crate::remote::download("http://modules.test/premiers.wasm", 100, Some(&fake)).unwrap_err().contains("HTTPS seulement"));
        assert!(crate::remote::download("https://127.0.0.1/premiers.wasm", 100, None).unwrap_err().contains("adresse IP"));
    }

    #[test]
    fn holo_serve_answers_with_the_downloaded_copy_and_the_page_tells_its_license() {
        let here = folder();
        std::fs::write(here.join("primes.holo"), page(&format!(", from: \"{FROM}\", sha256: \"{}\", license: \"MIT\"", sha256_hex(PRIMES)))).unwrap();
        let web = Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
        let mut site = crate::server::Site::open(&here, &web).unwrap();
        let b = bench(|_| Ok(PRIMES.to_vec()));
        site.copies = b.copies;
        let ask = |url| crate::server::Ask { method: "GET", url, accept: "*/*", cookie: "", content_type: "", origin: "", host: "localhost:8080", referer: "", peer: "127.0.0.1", forwarded: "", body: b"" };
        let answer = site.answer(&ask("/premiers.wasm"));
        assert_eq!((answer.status, answer.body.as_slice()), (200, PRIMES));
        assert!(answer.headers.iter().any(|(name, value)| name == "Content-Type" && value == "application/wasm"));
        assert_eq!(b.asked.lock().unwrap().len(), 1);
        // La copie rangée, la demande suivante est un fichier comme un autre ; rien de plus vers l'autre site.
        assert_eq!(site.answer(&ask("/premiers.wasm")).status, 200);
        assert_eq!(b.asked.lock().unwrap().len(), 1);
        assert_eq!(site.answer(&ask("/autre.wasm")).status, 404);
        // La page, sans JavaScript, dit la licence et le site d'où vient le module.
        let html = String::from_utf8(site.answer(&crate::server::Ask { accept: "text/html", ..ask("/primes.holo") }).body).unwrap();
        assert!(html.contains("<aside class=\"holo-modules\" aria-label=\"Modules de cette page\"><p data-module=\"Primes\" role=\"status\">Module «\u{202F}premiers.wasm\u{202F}», venu de modules.exemple.org — licence\u{a0}: MIT"), "{html}");
        let _ = std::fs::remove_dir_all(here);
    }
}
