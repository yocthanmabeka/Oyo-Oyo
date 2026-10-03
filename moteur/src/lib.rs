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
pub mod navigation;
pub mod plat;
pub mod regles;
pub mod styles;
pub mod univers;

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
    Ok(programme)
}

/// La vue à plat d'un fichier `.holo` : une page web ordinaire, fabriquée par le moteur.
pub fn vue_a_plat(source: &str, base: &str) -> Result<String, Erreur> {
    plat::page_html(&verifier_page(source)?, base)
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
    fn un_fichier_refuse_ne_donne_ni_page_ni_effet() {
        let casse = BOUTIQUE.replace("Workshop.enter", "Workshop.fly");
        assert!(vue_a_plat(&casse, "").unwrap_err().message.contains("capacité inconnue"));
        assert!(effets(&casse, "Open.tap").is_empty());
    }
}
