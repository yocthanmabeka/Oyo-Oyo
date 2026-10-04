//! Le moteur en ligne de commande, compilé pour le PC ou le serveur (pas pour le navigateur).
//!
//! ```text
//! holo check boutique.holo            vérifie le fichier ; « ok », ou l'erreur avec sa ligne
//! holo html boutique.holo [dossier]   écrit la page web ordinaire du fichier (HTML et CSS)
//! ```
//!
//! C'est le même code Rust que dans le navigateur. Le serveur s'en sert pour envoyer la page
//! déjà fabriquée : un robot de recherche, ou un navigateur qui ne lance pas le moteur, lit
//! quand même le site. `dossier` est l'adresse du dossier du fichier, pour retrouver ses images.

use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (commande, fichier, dossier) = match arguments.as_slice() {
        [commande, fichier] => (commande.as_str(), fichier, ""),
        [commande, fichier, dossier] => (commande.as_str(), fichier, dossier.as_str()),
        _ => ("", &String::new(), ""),
    };
    if commande != "check" && commande != "html" {
        eprintln!("usage : holo check fichier.holo | holo html fichier.holo [dossier]");
        return ExitCode::from(2);
    }
    let mut source = match std::fs::read_to_string(fichier) {
        Ok(source) => source,
        Err(erreur) => {
            eprintln!("{fichier} : {erreur}");
            return ExitCode::from(2);
        }
    };
    // Les fichiers importés sont lus à côté, et joints au texte : le moteur ne lit rien seul.
    let ici = std::path::Path::new(fichier).parent().unwrap_or(std::path::Path::new("."));
    for nom in holo_moteur::imports(&source).split(';').filter(|nom| !nom.is_empty()).map(str::to_string).collect::<Vec<_>>() {
        if let Ok(texte) = std::fs::read_to_string(ici.join(&nom)) {
            source.push(holo_moteur::holo::FICHIER_SUIVANT);
            source.push_str(&nom);
            source.push(holo_moteur::holo::SEPARE_LE_NOM);
            source.push_str(&texte);
        }
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
