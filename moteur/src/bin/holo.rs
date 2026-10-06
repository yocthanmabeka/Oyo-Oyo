//! Le moteur en ligne de commande, compilé pour le PC ou le serveur (pas pour le navigateur).
//!
//! ```text
//! holo check boutique.holo            vérifie le fichier ; « ok », ou l'erreur avec sa ligne
//! holo check - [dossier]              vérifie le texte reçu sur l'entrée standard (un éditeur
//!                                     qui n'a pas encore enregistré) ; « ok », ou « ligne L,
//!                                     colonne C : message » ; les imports sont lus dans `dossier`
//! holo html boutique.holo [dossier]   écrit la page web ordinaire du fichier (HTML et CSS)
//! holo vocabulaire                    tous les mots du langage, en JSON, pour un éditeur
//! holo fmt page.holo                  remet le fichier en forme, et l'écrit (ADR-054)
//! holo essai page.holo page.essai     joue les gestes d'un essai écrit, vérifie les valeurs (ADR-054)
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
    if arguments.first().map(String::as_str) == Some("vocabulaire") {
        println!("{}", holo_moteur::vocabulaire());
        return ExitCode::SUCCESS;
    }
    // Remettre un fichier en forme : seuls les blancs changent (ADR-054).
    if let [commande, fichier] = arguments.as_slice() {
        if commande == "fmt" {
            let Ok(texte) = std::fs::read_to_string(fichier) else {
                eprintln!("{fichier} : illisible");
                return ExitCode::from(2);
            };
            let forme = holo_moteur::outils::mettre_en_forme(&texte);
            if forme == texte {
                println!("{fichier} : déjà en forme");
            } else if std::fs::write(fichier, &forme).is_ok() {
                println!("{fichier} : remis en forme");
            } else {
                eprintln!("{fichier} : impossible d'écrire");
                return ExitCode::from(2);
            }
            return ExitCode::SUCCESS;
        }
    }
    // Jouer un essai écrit (ADR-054).
    if let [commande, page, essai] = arguments.as_slice() {
        if commande == "essai" {
            let (Ok(mut source), Ok(texte)) = (std::fs::read_to_string(page), std::fs::read_to_string(essai)) else {
                eprintln!("{page} ou {essai} : illisible");
                return ExitCode::from(2);
            };
            let ici = std::path::Path::new(page).parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
            for nom in holo_moteur::imports(&source).split(';').filter(|nom| !nom.is_empty()).map(str::to_string).collect::<Vec<_>>() {
                if let Ok(importe) = std::fs::read_to_string(ici.join(&nom)) {
                    source.push(holo_moteur::holo::FICHIER_SUIVANT);
                    source.push_str(&nom);
                    source.push(holo_moteur::holo::SEPARE_LE_NOM);
                    source.push_str(&importe);
                }
            }
            return match holo_moteur::outils::jouer(&source, &texte) {
                Ok(holo_moteur::outils::Essai { reussies, echec: None }) => {
                    println!("{essai} : ok, {reussies} ligne(s) jouée(s)");
                    ExitCode::SUCCESS
                }
                Ok(holo_moteur::outils::Essai { echec: Some((ligne, message)), .. }) => {
                    eprintln!("{essai}, ligne {ligne} : {message}");
                    ExitCode::FAILURE
                }
                Err(erreur) => {
                    eprintln!("{page} : {erreur}");
                    ExitCode::FAILURE
                }
            };
        }
    }
    let (commande, fichier, dossier) = match arguments.as_slice() {
        [commande, fichier] => (commande.as_str(), fichier, ""),
        [commande, fichier, dossier] => (commande.as_str(), fichier, dossier.as_str()),
        _ => ("", &String::new(), ""),
    };
    if commande != "check" && commande != "html" {
        eprintln!("usage : holo check fichier.holo | holo check - [dossier] | holo html fichier.holo [dossier] | holo fmt fichier.holo | holo essai page.holo page.essai | holo vocabulaire");
        return ExitCode::from(2);
    }
    // `-` : le texte arrive par l'entrée standard, tel qu'il est dans l'éditeur (ADR-046).
    let depuis_l_editeur = fichier == "-";
    let mut source = if depuis_l_editeur {
        let mut texte = String::new();
        if std::io::stdin().read_to_string(&mut texte).is_err() {
            eprintln!("le texte reçu n'est pas lisible");
            return ExitCode::from(2);
        }
        texte
    } else {
        match std::fs::read_to_string(fichier) {
            Ok(source) => source,
            Err(erreur) => {
                eprintln!("{fichier} : {erreur}");
                return ExitCode::from(2);
            }
        }
    };
    // Les fichiers importés sont lus à côté, et joints au texte : le moteur ne lit rien seul.
    let ici = if depuis_l_editeur {
        std::path::PathBuf::from(if dossier.is_empty() { "." } else { dossier })
    } else {
        std::path::Path::new(fichier).parent().unwrap_or(std::path::Path::new(".")).to_path_buf()
    };
    for nom in holo_moteur::imports(&source).split(';').filter(|nom| !nom.is_empty()).map(str::to_string).collect::<Vec<_>>() {
        if let Ok(texte) = std::fs::read_to_string(ici.join(&nom)) {
            source.push(holo_moteur::holo::FICHIER_SUIVANT);
            source.push_str(&nom);
            source.push(holo_moteur::holo::SEPARE_LE_NOM);
            source.push_str(&texte);
        }
    }
    // L'heure du lieu, si le serveur la donne (HOLO_MAINTENANT=2026,10,6,2,14,5 : année, mois,
    // jour, jour de la semaine, heure, minute) ; sinon l'heure universelle (ADR-039).
    let donnee: Option<Vec<u64>> = std::env::var("HOLO_MAINTENANT").ok().map(|v| v.split(',').filter_map(|n| n.trim().parse().ok()).collect());
    match donnee.as_deref() {
        Some(&[annee, mois, jour, semaine, heure, minute]) => holo_moteur::regler_maintenant([annee, mois, jour, semaine, heure, minute]),
        _ => {
            let secondes = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
            holo_moteur::regler_maintenant(holo_moteur::etat::depuis_secondes_unix(secondes));
        }
    }
    // Pour un éditeur : la même réponse que celle de l'éditeur du navigateur (ADR-046).
    if depuis_l_editeur && commande == "check" {
        let reponse = holo_moteur::verifier_texte(&source);
        println!("{reponse}");
        return if reponse.starts_with("ok") { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }
    let resultat = match commande {
        "check" => holo_moteur::verifier_page(&source).map(|_| "ok".to_string()),
        _ => holo_moteur::vue_a_plat(&source, dossier),
    };
    match resultat {
        Ok(sortie) => {
            println!("{sortie}");
            ExitCode::SUCCESS
        }
        Err(erreur) => {
            eprintln!("{fichier} : {erreur}");
            ExitCode::FAILURE
        }
    }
}
