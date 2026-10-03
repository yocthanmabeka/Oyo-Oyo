//! Moteur HoloCode, sprint Big Bang.
//!
//! Le cœur est en Rust pur et se teste sur le PC (`cargo test`) :
//! - `holo` lit un fichier `.holo` ;
//! - `blocs` vérifie que chaque bloc existe et que les titres ne sautent pas de niveau ;
//! - `styles` vérifie les styles, écrits comme en CSS ;
//! - `regles` vérifie les noms, les règles et les budgets, et dit ce qu'un signal demande ;
//! - `plat` fabrique la page web ordinaire d'un fichier (la vue à plat) ;
//! - `univers` en fait un monde, entièrement calculé à partir d'une graine ;
//! - `navigation` gère le morcellement, le zoom, l'entrée et la sortie.
//!
//! La partie qui parle au navigateur et à la carte graphique (`web`, `rendu`) n'est
//! compilée que pour WebAssembly.

pub mod blocs;
pub mod graine;
pub mod holo;
pub mod mosaique;
pub mod navigation;
pub mod plat;
pub mod regles;
pub mod styles;
pub mod univers;
pub mod vue;

#[cfg(target_arch = "wasm32")]
mod rendu;
#[cfg(target_arch = "wasm32")]
mod web;

use holo::{Erreur, Programme, Valeur};
use univers::PointDecl;

/// Lit et vérifie un fichier `.holo`, sans rien exécuter. Toute erreur est rendue avec sa
/// ligne et sa colonne.
pub fn verifier(source: &str) -> Result<PointDecl, Erreur> {
    let programme = holo::lire(source)?;
    blocs::verifier_blocs(&programme)?;
    styles::verifier_styles(&programme)?;
    univers::point_depuis(&programme)
}

/// Lit et vérifie un fichier `.holo` entier : blocs, titres, styles, noms, règles, budgets.
pub fn verifier_page(source: &str) -> Result<Programme, Erreur> {
    let programme = holo::lire(source)?;
    blocs::verifier_blocs(&programme)?;
    styles::verifier_styles(&programme)?;
    regles::verifier_regles(&programme)?;
    vue::reglages(&programme)?;
    Ok(programme)
}

/// La vue à plat d'un fichier `.holo` : une page web ordinaire, fabriquée par le moteur.
pub fn vue_a_plat(source: &str, base: &str) -> Result<String, Erreur> {
    vue_a_plat_de(source, base, "")
}

/// La vue à plat d'un site du fichier : sa page (chemin vide), ou le monde d'un de ses points
/// (`Shop/Secret`), ouvert en grand comme une page.
pub fn vue_a_plat_de(source: &str, base: &str, chemin: &str) -> Result<String, Erreur> {
    let programme = verifier_page(source)?;
    let site = regles::site_de(&programme, chemin)?;
    plat::site_html(&programme, site, base, chemin.rsplit('/').next().unwrap_or(""))
}

/// Les effets que les règles du fichier demandent pour un signal, comme `Open.tap`.
pub fn effets(source: &str, signal: &str) -> Vec<String> {
    verifier_page(source).map(|programme| regles::effets(&programme, signal)).unwrap_or_default()
}

/// Le fichier `.holo` d'un seul point de la page, pour ouvrir sa vue en profondeur.
pub fn source_du_point(source: &str, nom: &str) -> Option<String> {
    let programme = verifier_page(source).ok()?;
    let point = regles::bloc_nomme(&programme, nom).filter(|bloc| bloc.nom == "Point")?;
    let ecrire = |valeur: &Valeur| match valeur {
        Valeur::Nom(n) => Some(n.clone()),
        Valeur::Entier(e) => Some(e.to_string()),
        Valeur::Nombre { valeur, unite: None } => Some(valeur.to_string()),
        Valeur::Texte(t) => Some(format!("\"{t}\"")),
        Valeur::Liste(elements) => {
            let textes: Option<Vec<String>> = elements
                .iter()
                .map(|e| match e {
                    Valeur::Texte(t) => Some(format!("\"{t}\"")),
                    _ => None,
                })
                .collect();
            textes.map(|t| format!("[{}]", t.join(", ")))
        }
        _ => None,
    };
    let reglages: Vec<String> = ["name", "seed", "brightness", "fragments", "color", "palette"]
        .iter()
        .filter_map(|param| Some(format!("{param}: {}", ecrire(&point.argument(param)?.valeur)?)))
        .collect();
    Some(format!("Point({})", reglages.join(", ")))
}

/// Le monde où la page est posée quand on la regarde en personnage. Provisoire : tant que
/// le langage ne sait pas écrire « un monde qui contient une page », la graine de ce monde
/// se tire du nom de la page, pour que le même fichier redonne le même lieu (ADR-008).
pub fn monde_d_accueil(source: &str) -> Option<String> {
    let programme = verifier_page(source).ok()?;
    let nom = regles::nom_de(&programme.racine).unwrap_or("Home");
    let graine = nom.bytes().fold(0u64, |g, octet| graine::melanger(g ^ u64::from(octet)));
    // Yocthan : pas de points décoratifs autour de la page. Le lieu est éteint ; ce sont les
    // éléments de la page eux-mêmes qui deviendront des points (voir `mosaique.rs`).
    Some(format!("Point(name: {nom}, seed: {graine}, brightness: 0, fragments: 12)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOUTIQUE: &str = include_str!("../../exemples/boutique-comparee/boutique.holo");

    #[test]
    fn la_boutique_passe_de_la_page_au_point() {
        assert!(vue_a_plat(BOUTIQUE, "").unwrap().contains("<h1 class=\"holo-H1\">My shop</h1>"));
        assert_eq!(effets(BOUTIQUE, "Open.tap"), ["Workshop.enter"]);
        let point = source_du_point(BOUTIQUE, "Workshop").unwrap();
        assert_eq!(point, "Point(name: Workshop, seed: 42, brightness: 0.8, fragments: 6, color: \"#E9B44C\", palette: [\"#E9B44C\", \"#245C45\"])");
        // Ce point seul est un fichier que la vue en profondeur accepte.
        let decl = verifier(&point).unwrap();
        assert_eq!((decl.graine, decl.morceler, decl.palette.len()), (42, 6, 2));
        assert_eq!(source_du_point(BOUTIQUE, "Open"), None);
    }

    #[test]
    fn le_monde_d_accueil_est_toujours_le_meme() {
        let accueil = monde_d_accueil(BOUTIQUE).unwrap();
        assert_eq!(Some(accueil.clone()), monde_d_accueil(BOUTIQUE));
        let decl = verifier(&accueil).unwrap();
        assert_eq!((decl.nom.as_str(), decl.morceler), ("Shop", 12));
        // Une autre page a un autre lieu.
        assert_ne!(monde_d_accueil("Page(name: Blog)").unwrap(), accueil);
        assert!(monde_d_accueil("Page(children: [ Div() ])").is_none());
    }

    #[test]
    fn les_exemples_du_guide_sont_acceptes_par_le_moteur() {
        let guide = include_str!("../../docs/01-holocode/GUIDE.md");
        let exemples: Vec<&str> = guide.split("```holo
").skip(1).map(|suite| suite.split("```").next().unwrap()).collect();
        assert!(exemples.len() >= 8, "le guide a perdu ses exemples : {}", exemples.len());
        for exemple in exemples {
            // Une page passe toutes les vérifications et se fabrique ; un point seul s'ouvre en profondeur.
            let resultat = if exemple.trim_start().starts_with("Point(") { verifier(exemple).map(|_| ()) } else { vue_a_plat(exemple, "").map(|_| ()) };
            if let Err(erreur) = resultat {
                panic!("un exemple du guide est refusé : {erreur}
{exemple}");
            }
        }
    }

    #[test]
    fn un_fichier_refuse_ne_donne_ni_page_ni_effet() {
        let casse = BOUTIQUE.replace("Workshop.enter", "Workshop.fly");
        assert!(vue_a_plat(&casse, "").unwrap_err().message.contains("capacité inconnue"));
        assert!(effets(&casse, "Open.tap").is_empty());
    }
}
