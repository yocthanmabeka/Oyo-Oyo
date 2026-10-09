//! Le moteur en ligne de commande, compilé pour le PC ou le serveur (pas pour le navigateur).
//!
//! ```text
//! holo check boutique.holo            vérifie le fichier ; « ok », ou l'erreur avec sa ligne
//! holo check - [dossier]              vérifie le texte reçu sur l'entrée standard (un éditeur
//!                                     qui n'a pas encore enregistré) ; « ok », ou « ligne L,
//!                                     colonne C : message » ; les imports sont lus dans `dossier`
//! holo html boutique.holo [dossier]   écrit la page web ordinaire du fichier (HTML et CSS)
//! holo vocabulary                     tous les mots du langage, en JSON, pour un éditeur
//! holo fmt page.holo                  remet le fichier en forme, et l'écrit (ADR-054)
//! holo test page.holo page.test       joue les gestes d'un essai écrit, vérifie les valeurs (ADR-054)
//! holo serve [dossier] [port]         sert un site sur ce PC, avec sa base SQLite ; les boutons
//!                                     marchent même sans JavaScript (ADR-074)
//! holo messages [dossier]             les messages reçus par ses formulaires, en JSON (ADR-075)
//! holo backup [dossier]               sauvegarde sa base dans holo-data/backups/ (ADR-076)
//! holo share page.holo < geste.json   un geste sur une page qui partage des valeurs (ADR-079) :
//!                                     reçoit {"shared":…,"state":…,"signal":…}, rend
//!                                     {"accepted":…,"changed":…,"state":…,"shared":…} ; le serveur d'essai
//!                                     s'en sert pour arbitrer avec le même moteur
//! ```
//!
//! C'est le même code Rust que dans le navigateur. Le serveur s'en sert pour envoyer la page
//! déjà fabriquée : un robot de recherche, ou un navigateur qui ne lance pas le moteur, lit
//! quand même le site. `dossier` est l'adresse du dossier du fichier, pour retrouver ses images.

use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    // Les mots du langage, pour l'extension VS Code (ADR-046).
    if arguments.first().map(String::as_str) == Some("vocabulary") {
        println!("{}", holo_engine::vocabulary());
        return ExitCode::SUCCESS;
    }
    // Servir un site (ADR-074) : le dossier donné, ou celui où l'on est ; le port 8080, ou un autre.
    #[cfg(not(target_arch = "wasm32"))]
    if arguments.first().map(String::as_str) == Some("serve") {
        let folder = arguments.get(1).map_or(".", String::as_str);
        let Some(port) = arguments.get(2).map_or(Some(8080), |p| p.parse::<u16>().ok()) else {
            eprintln!("usage : holo serve [folder] [port]");
            return ExitCode::from(2);
        };
        // Le moteur pour le navigateur : moteur/web, ou le dossier de HOLO_WEB.
        let web = std::env::var("HOLO_WEB").map_or_else(|_| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web"), std::path::PathBuf::from);
        return match holo_engine::server::serve(std::path::Path::new(folder), &web, port) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        };
    }
    // Sauvegarder la base d'un site servi (ADR-076).
    #[cfg(not(target_arch = "wasm32"))]
    if arguments.first().map(String::as_str) == Some("backup") {
        let folder = arguments.get(1).map_or(".", String::as_str);
        return match holo_engine::server::backup(std::path::Path::new(folder)) {
            Ok(path) => {
                println!("{}", path.display());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{folder} : {error}");
                ExitCode::FAILURE
            }
        };
    }
    // Les messages reçus par les formulaires d'un site servi (ADR-075), une ligne JSON chacun.
    #[cfg(not(target_arch = "wasm32"))]
    if arguments.first().map(String::as_str) == Some("messages") {
        let folder = arguments.get(1).map_or(".", String::as_str);
        return match holo_engine::server::messages(std::path::Path::new(folder)) {
            Ok(lines) => {
                for line in &lines {
                    println!("{line}");
                }
                eprintln!("{} message(s)", lines.len());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{folder} : {error}");
                ExitCode::FAILURE
            }
        };
    }
    // Remettre un fichier en forme : seuls les blancs changent (ADR-054).
    if let [command, file] = arguments.as_slice() {
        if command == "fmt" {
            let Ok(text) = std::fs::read_to_string(file) else {
                eprintln!("{file} : illisible");
                return ExitCode::from(2);
            };
            let shape = holo_engine::tools::format_source(&text);
            if shape == text {
                println!("{file} : déjà en forme");
            } else if std::fs::write(file, &shape).is_ok() {
                println!("{file} : remis en forme");
            } else {
                eprintln!("{file} : impossible d'écrire");
                return ExitCode::from(2);
            }
            return ExitCode::SUCCESS;
        }
    }
    // Jouer un essai écrit (ADR-054).
    if let [command, page, test] = arguments.as_slice() {
        if command == "test" {
            let (Ok(mut source), Ok(text)) = (std::fs::read_to_string(page), std::fs::read_to_string(test)) else {
                eprintln!("{page} ou {test} : illisible");
                return ExitCode::from(2);
            };
            let here = std::path::Path::new(page).parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
            for name in holo_engine::imports(&source).split(';').filter(|name| !name.is_empty()).map(str::to_string).collect::<Vec<_>>() {
                if let Ok(imported) = std::fs::read_to_string(here.join(&name)) {
                    source.push(holo_engine::holo::NEXT_FILE);
                    source.push_str(&name);
                    source.push(holo_engine::holo::NAME_SEPARATOR);
                    source.push_str(&imported);
                }
            }
            return match holo_engine::tools::play(&source, &text) {
                Ok(holo_engine::tools::Test { succeeded, failure: None }) => {
                    println!("{test} : ok, {succeeded} ligne(s) jouée(s)");
                    ExitCode::SUCCESS
                }
                Ok(holo_engine::tools::Test { failure: Some((line, message)), .. }) => {
                    eprintln!("{test}, ligne {line} : {message}");
                    ExitCode::FAILURE
                }
                Err(error) => {
                    eprintln!("{page} : {error}");
                    ExitCode::FAILURE
                }
            };
        }
    }
    let (command, file, folder) = match arguments.as_slice() {
        [command, file] => (command.as_str(), file, ""),
        [command, file, folder] => (command.as_str(), file, folder.as_str()),
        _ => ("", &String::new(), ""),
    };
    if command != "check" && command != "html" && command != "files" && command != "form" && command != "share" {
        eprintln!("usage : holo check file.holo | holo check - [folder] | holo html file.holo [folder] | holo files page.holo | holo form page.holo < message.json | holo share page.holo < gesture.json | holo fmt file.holo | holo test page.holo page.test | holo serve [folder] [port] | holo messages [folder] | holo backup [folder] | holo vocabulary");
        return ExitCode::from(2);
    }
    // `-` : le texte arrive par l'entrée standard, tel qu'il est dans l'éditeur (ADR-046).
    let from_editor = file == "-";
    let mut source = if from_editor {
        let mut text = String::new();
        if std::io::stdin().read_to_string(&mut text).is_err() {
            eprintln!("le texte reçu n'est pas lisible");
            return ExitCode::from(2);
        }
        text
    } else {
        match std::fs::read_to_string(file) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("{file} : {error}");
                return ExitCode::from(2);
            }
        }
    };
    // Les fichiers importés sont lus à côté, et joints au texte : le moteur ne lit rien seul.
    let here = if from_editor {
        std::path::PathBuf::from(if folder.is_empty() { "." } else { folder })
    } else {
        std::path::Path::new(file).parent().unwrap_or(std::path::Path::new(".")).to_path_buf()
    };
    for name in holo_engine::imports(&source).split(';').filter(|name| !name.is_empty()).map(str::to_string).collect::<Vec<_>>() {
        if let Ok(text) = std::fs::read_to_string(here.join(&name)) {
            source.push(holo_engine::holo::NEXT_FILE);
            source.push_str(&name);
            source.push(holo_engine::holo::NAME_SEPARATOR);
            source.push_str(&text);
        }
    }
    // Les valeurs de l'adresse (ADR-078) : celles que donne le serveur (HOLO_ADDRESS=id=123, les
    // morceaux tels qu'ils sont dans l'URL) ; ou, pour vérifier un modèle `profil/{id}.holo`, un
    // texte vide par nom.
    let address = std::env::var("HOLO_ADDRESS").ok();
    if address.is_some() || !holo_engine::address::names(file).is_empty() {
        let values: Vec<(String, String)> = match address {
            Some(pairs) => pairs
                .split('&')
                .filter(|pair| !pair.is_empty())
                .map(|pair| {
                    let (name, raw) = pair.split_once('=').unwrap_or((pair, ""));
                    (name.to_string(), raw.to_string())
                })
                .collect(),
            None => holo_engine::address::empty_values(file),
        };
        source = holo_engine::address::joined(&source, &values);
    }
    // L'heure du lieu, si le serveur la donne (HOLO_NOW=2026,10,6,2,14,5 : année, mois,
    // jour, jour de la semaine, heure, minute) ; sinon l'heure universelle (ADR-039).
    let datum: Option<Vec<u64>> = std::env::var("HOLO_NOW").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect());
    match datum.as_deref() {
        Some(&[year, month, day, week, hour, minute]) => holo_engine::set_now([year, month, day, week, hour, minute]),
        _ => {
            let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
            holo_engine::set_now(holo_engine::state::from_unix_seconds(seconds));
        }
    }
    // Pour un éditeur : la même réponse que celle de l'éditeur du navigateur (ADR-046).
    if from_editor && command == "check" {
        let response = holo_engine::check_text(&source);
        println!("{response}");
        return if response.starts_with("ok") { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }
    // Un fichier de styles seuls (ADR-052) se vérifie comme dans l'éditeur : c'est un thème, que
    // des pages importent, pas une page.
    if command == "check" && holo_engine::is_styles_file(&source) {
        let response = holo_engine::check_text(&source);
        if response.starts_with("ok") {
            println!("{response}");
            return ExitCode::SUCCESS;
        }
        eprintln!("{file} : {response}");
        return ExitCode::FAILURE;
    }
    // Pour le serveur : ce qu'un formulaire a envoyé, lu sur l'entrée standard, est-il bon ?
    // « ok », ou une ligne par erreur (ADR-068).
    if command == "form" {
        let mut message = String::new();
        if std::io::stdin().read_to_string(&mut message).is_err() {
            eprintln!("le message reçu n'est pas lisible");
            return ExitCode::from(2);
        }
        return match holo_engine::check_submission(&source, &message) {
            Ok(errors) if errors.is_empty() => {
                println!("ok");
                ExitCode::SUCCESS
            }
            Ok(errors) => {
                println!("{errors}");
                ExitCode::FAILURE
            }
            Err(error) => {
                eprintln!("{file} : {error}");
                ExitCode::from(2)
            }
        };
    }
    // Pour le serveur d'essai (ADR-079) : un geste sur une page qui partage des valeurs, arbitré
    // par le même moteur que la page et que holo serve.
    if command == "share" {
        let mut gesture = String::new();
        if std::io::stdin().read_to_string(&mut gesture).is_err() {
            eprintln!("le geste reçu n'est pas lisible");
            return ExitCode::from(2);
        }
        return match holo_engine::share_command(&source, &gesture) {
            Ok(answer) => {
                println!("{answer}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{file} : {error}");
                ExitCode::FAILURE
            }
        };
    }
    // Les valeurs partagées du moment (HOLO_SHARED=seats=19;likes=3), que le serveur d'essai
    // garde : la page est fabriquée avec elles (ADR-079).
    let shared = std::env::var("HOLO_SHARED").ok().filter(|_| command == "html" && !holo_engine::shared_names(&source).is_empty());
    let result = match command {
        "check" => holo_engine::check_page(&source).map(|_| "ok".to_string()),
        _ if shared.is_some() => holo_engine::shared_page(&source, folder, shared.as_deref().unwrap_or("")),
        // Pour le serveur : les fichiers qu'un formulaire de la page peut envoyer (ADR-059).
        "files" => holo_engine::files_for_server(&source),
        // La page fabriquée avec ses données (ADR-064) : le fichier de `Data(from:)`, rangé à
        // côté du .holo, s'il existe et pèse 64 Ko au plus. Sinon, la page de départ.
        _ => {
            let data_file = holo_engine::data(&source).split('|').next().map(str::to_string).filter(|f| !f.is_empty());
            let json = data_file
                .map(|f| here.join(f))
                .filter(|path| std::fs::metadata(path).is_ok_and(|m| m.len() <= holo_engine::state::DATA_BYTES as u64))
                .and_then(|path| std::fs::read_to_string(path).ok());
            // Les valeurs que l'adresse porte après le `?` (ADR-091), que donne le serveur
            // (HOLO_QUERY=tab=photos&page=2) : la page est fabriquée avec elles.
            match json {
                Some(json) => holo_engine::flat_view_with_data(&source, folder, &json),
                None => holo_engine::flat_view_at(&source, folder, &std::env::var("HOLO_QUERY").unwrap_or_default()),
            }
        }
    };
    match result {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{file} : {error}");
            ExitCode::FAILURE
        }
    }
}
