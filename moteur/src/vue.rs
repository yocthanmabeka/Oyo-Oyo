//! Ce que le fichier `.holo` dit de la façon dont sa page se regarde : jusqu'où l'on zoome,
//! quand ses pixels deviennent des points, comment ils se morcellent, quel relief ils prennent.
//!
//! ```holo
//! Page(
//!   zoom: Zoom(max: 1000000, shrink: false),
//!   points: Points(size: 6px, fragment: 40px, grid: 4, depth: 20, density: 2),
//!   relief: Relief(height: 10px, tilt: 52deg),
//! )
//! ```
//!
//! Chaque réglage a des bornes : ce sont les garde-fous. Un auteur ne peut pas écrire une
//! page qui demanderait à la machine plus qu'elle ne peut donner (ADR-005).

use crate::holo::{Bloc, Erreur, Programme, Valeur};

/// Les réglages de la vue d'une page. Sans rien écrire, on obtient ceux-ci.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reglages {
    /// `Zoom(max:)` : combien de fois on peut grossir la page, au plus.
    pub zoom_max: f64,
    /// `Zoom(shrink:)` : vrai, dézoomer réduit la page jusqu'à un seul point ; faux, on ne
    /// dézoome pas en deçà de la page entière.
    pub reduire: bool,
    /// `Points(size:)` : la taille à l'écran (en pixels) où un pixel de la page devient un point.
    pub taille_point: f64,
    /// `Points(fragment:)` : la taille où un point se morcelle.
    pub taille_morceler: f64,
    /// `Points(grid:)` : un point se morcelle en une grille de `grid × grid`.
    pub cote: u64,
    /// `Points(depth:)` : combien de fois de suite un point peut se morceler.
    pub niveaux: u32,
    /// `Points(density:)` : points par pixel d'écran, dans chaque sens, au repos.
    pub densite: f64,
    /// `Relief(height:)` : de combien se soulève ce qui est lumineux, quand la page est de biais.
    pub relief: f64,
    /// `Relief(tilt:)` : jusqu'où l'on peut tourner la page, en radians.
    pub angle_max: f64,
}

impl Default for Reglages {
    fn default() -> Self {
        Reglages { zoom_max: 1e12, reduire: false, taille_point: 6.0, taille_morceler: 40.0, cote: 4, niveaux: 20, densite: 2.0, relief: 10.0, angle_max: 52f64.to_radians() }
    }
}

fn refus<T>(bloc: &Bloc, param: &str, attendu: &str) -> Result<T, Erreur> {
    let pos = bloc.argument(param).map_or(bloc.pos, |a| a.pos);
    Err(Erreur { message: format!("« {}({param}: …) » attend {attendu}", bloc.nom), pos })
}

/// Un nombre, avec l'unité demandée (ou sans unité), compris entre deux bornes.
fn nombre(bloc: &Bloc, param: &str, unite: Option<&str>, min: f64, max: f64, defaut: f64) -> Result<f64, Erreur> {
    let Some(argument) = bloc.argument(param) else { return Ok(defaut) };
    let valeur = match (&argument.valeur, unite) {
        (Valeur::Entier(e), None) => Some(*e as f64),
        (Valeur::Nombre { valeur, unite: None }, None) => Some(*valeur),
        (Valeur::Nombre { valeur, unite: Some(u) }, Some(attendue)) if u == attendue => Some(*valeur),
        _ => None,
    };
    match valeur {
        Some(v) if (min..=max).contains(&v) => Ok(v),
        _ => refus(bloc, param, &format!("un nombre entre {min}{u} et {max}{u}", u = unite.unwrap_or(""))),
    }
}

fn entier(bloc: &Bloc, param: &str, min: u64, max: u64, defaut: u64) -> Result<u64, Erreur> {
    match bloc.argument(param).map(|a| &a.valeur) {
        None => Ok(defaut),
        Some(Valeur::Entier(e)) if (min..=max).contains(e) => Ok(*e),
        Some(_) => refus(bloc, param, &format!("un nombre entier entre {min} et {max}")),
    }
}

/// Refuse un paramètre que le bloc ne connaît pas, en donnant la liste des possibles.
fn seulement(bloc: &Bloc, connus: &[&str]) -> Result<(), Erreur> {
    for argument in &bloc.arguments {
        match argument.nom.as_deref() {
            Some(nom) if connus.contains(&nom) => {}
            Some(nom) => return Err(Erreur { message: format!("« {} » n'a pas de paramètre « {nom} » ; paramètres possibles : {}", bloc.nom, connus.join(", ")), pos: argument.pos }),
            None => return Err(Erreur { message: format!("chaque paramètre de « {} » est nommé : {}", bloc.nom, connus.join(", ")), pos: argument.pos }),
        }
    }
    Ok(())
}

/// Le bloc donné à un paramètre de la page : `zoom: Zoom(...)`.
fn bloc_de<'a>(page: &'a Bloc, param: &str, nom: &str) -> Result<Option<&'a Bloc>, Erreur> {
    match page.argument(param).map(|a| &a.valeur) {
        None => Ok(None),
        Some(Valeur::Bloc(bloc)) if bloc.nom == nom => Ok(Some(bloc)),
        Some(_) => refus(page, param, &format!("un bloc « {nom}(...) »")),
    }
}

/// Lit et vérifie les réglages de vue écrits dans la page. Sans eux, les réglages par défaut.
pub fn reglages(programme: &Programme) -> Result<Reglages, Erreur> {
    let mut r = Reglages::default();
    let page = &programme.racine;
    if page.nom != "Page" {
        return Ok(r);
    }
    if let Some(zoom) = bloc_de(page, "zoom", "Zoom")? {
        seulement(zoom, &["max", "shrink"])?;
        r.zoom_max = nombre(zoom, "max", None, 1.0, 1e12, r.zoom_max)?;
        r.reduire = match zoom.argument("shrink").map(|a| &a.valeur) {
            None => r.reduire,
            Some(Valeur::Bool(b)) => *b,
            Some(_) => return refus(zoom, "shrink", "true ou false"),
        };
    }
    if let Some(points) = bloc_de(page, "points", "Points")? {
        seulement(points, &["size", "fragment", "grid", "depth", "density"])?;
        r.taille_point = nombre(points, "size", Some("px"), 2.0, 32.0, r.taille_point)?;
        r.cote = entier(points, "grid", 2, 8, r.cote)?;
        r.niveaux = entier(points, "depth", 0, 20, u64::from(r.niveaux))? as u32;
        r.densite = nombre(points, "density", None, 1.0, 3.0, r.densite)?;
        r.taille_morceler = nombre(points, "fragment", Some("px"), 8.0, 400.0, r.taille_morceler)?;
        // Garde-fou : un point qui vient de se morceler doit rester visible, et il ne doit
        // jamais y avoir plus de points à l'écran que quand ils apparaissent.
        if r.taille_morceler / (r.cote as f64) < r.taille_point {
            return refus(points, "fragment", &format!("au moins {}px : « fragment » divisé par « grid » ne doit pas être plus petit que « size »", r.taille_point * r.cote as f64));
        }
    }
    if let Some(relief) = bloc_de(page, "relief", "Relief")? {
        seulement(relief, &["height", "tilt"])?;
        r.relief = nombre(relief, "height", Some("px"), 0.0, 40.0, r.relief)?;
        r.angle_max = nombre(relief, "tilt", Some("deg"), 0.0, 80.0, r.angle_max.to_degrees())?.to_radians();
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::lire;

    fn lus(src: &str) -> Result<Reglages, Erreur> {
        reglages(&lire(src)?)
    }

    #[test]
    fn sans_rien_ecrire_on_a_les_reglages_par_defaut() {
        assert_eq!(lus("Page(title: \"x\")").unwrap(), Reglages::default());
        assert_eq!(lus("Point(name: A, seed: 1)").unwrap(), Reglages::default());
    }

    #[test]
    fn la_boutique_ecrit_ses_reglages() {
        let r = lus(include_str!("../../exemples/boutique-comparee/boutique.holo")).unwrap();
        assert_eq!((r.zoom_max, r.reduire), (1_000_000.0, false));
        assert_eq!((r.taille_point, r.taille_morceler, r.cote, r.niveaux, r.densite), (6.0, 40.0, 4, 20, 2.0));
        assert_eq!(r.relief, 10.0);
        assert!((r.angle_max - 52f64.to_radians()).abs() < 1e-12);
    }

    #[test]
    fn chaque_reglage_a_ses_garde_fous() {
        let page = |reglage: &str| format!("Page({reglage})");
        for (reglage, message) in [
            ("zoom: Zoom(max: 0)", "un nombre entre 1 et"),
            ("zoom: Zoom(shrink: 1)", "true ou false"),
            ("zoom: Zoom(speed: 3)", "n'a pas de paramètre « speed »"),
            ("zoom: 4", "un bloc « Zoom(...) »"),
            ("zoom: Points(size: 6px)", "un bloc « Zoom(...) »"),
            ("points: Points(size: 6)", "entre 2px et 32px"),
            ("points: Points(size: 1px)", "entre 2px et 32px"),
            ("points: Points(grid: 20)", "entier entre 2 et 8"),
            ("points: Points(depth: 99)", "entier entre 0 et 20"),
            ("points: Points(fragment: 12px)", "au moins 24px"),
            ("points: Points(density: 9)", "entre 1 et 3"),
            ("relief: Relief(height: 500px)", "entre 0px et 40px"),
            ("relief: Relief(tilt: 90deg)", "entre 0deg et 80deg"),
            ("relief: Relief(tilt: 30px)", "entre 0deg et 80deg"),
        ] {
            let erreur = lus(&page(reglage)).unwrap_err();
            assert!(erreur.message.contains(message), "{reglage} → {erreur}");
        }
        let r = lus(&page("zoom: Zoom(max: 50, shrink: true), points: Points(size: 8px, fragment: 64px, grid: 2, depth: 3), relief: Relief(height: 0px, tilt: 0deg)")).unwrap();
        assert_eq!((r.zoom_max, r.reduire, r.taille_point, r.taille_morceler, r.cote, r.niveaux, r.relief, r.angle_max), (50.0, true, 8.0, 64.0, 2, 3, 0.0, 0.0));
    }
}
