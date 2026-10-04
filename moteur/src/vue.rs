//! Ce que le fichier `.holo` dit de la façon dont sa page se regarde : jusqu'où l'on zoome,
//! quand ses pixels deviennent des points, comment ils se morcellent, quel relief ils prennent.
//!
//! ```holo
//! Page(
//!   zoom: Zoom(max: 1000000, shrink: false, levels: 8, speed: 1),
//!   points: Points(after: 4, size: 6px, fragment: 40px, grid: 4, depth: 20, density: 2),
//!   relief: Relief(height: 10px, tilt: 360deg),
//!   portals: Portals(layout: grid, count: 12, size: 170px, brightness: 0.15, duration: 450ms),
//! )
//! ```
//!
//! Chaque réglage a des bornes : ce sont les garde-fous. Un auteur ne peut pas écrire une
//! page qui demanderait à la machine plus qu'elle ne peut donner (ADR-005).

use crate::holo::{Bloc, Erreur, Programme, Valeur};

/// Comment les portails du carrefour se rangent, et dans quel sens on les fait défiler.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Disposition {
    /// En grille, sur toute la fenêtre.
    Grille,
    /// Sur une ligne : on défile de gauche à droite.
    Ligne,
    /// Sur une colonne : on défile de haut en bas.
    Colonne,
    /// En diagonale.
    Diagonale,
}

/// Les réglages de la vue d'une page. Sans rien écrire, on obtient ceux-ci.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reglages {
    /// `Zoom(active:)` : faux, le visiteur ne peut pas zoomer dans la page.
    pub zoom_actif: bool,
    /// `Portals(layout:)` : comment les portails du carrefour se rangent.
    pub portails_disposition: Disposition,
    /// `Portals(count:)` : combien de mondes le carrefour montre. Les sites écrits dans le
    /// fichier passent d'abord ; le reste est rempli par des mondes calculés à partir d'une graine.
    pub portails_nombre: u32,
    /// `Portals(size:)` : la taille d'un portail, en pixels.
    pub portails_taille: f64,
    /// `Portals(brightness:)` : la lumière du fond du carrefour, de 0 (aucune) à 1.
    pub portails_lumiere: f64,
    /// `Zoom(speed:)` : la vitesse du zoom à la molette. 1 : la vitesse ordinaire.
    pub zoom_vitesse: f64,
    /// `Portals(duration:)` : la durée, en millisecondes, de l'ouverture d'un portail.
    pub portails_duree: f64,
    /// `Zoom(max:)` : combien de fois on peut grossir la page, au plus.
    pub zoom_max: f64,
    /// `Zoom(shrink:)` : vrai, dézoomer réduit la page jusqu'à un seul point ; faux, on ne
    /// dézoome pas en deçà de la page entière.
    pub reduire: bool,
    /// `Zoom(levels:)` : combien de sites peuvent s'emboîter les uns dans les autres, au plus.
    pub niveaux_de_sites: u32,
    /// `Points(after:)` : jusqu'à ce grossissement, la page reste un site ordinaire, qu'on lit,
    /// qu'on sélectionne et qu'on copie ; au-delà, ses pixels deviennent des points.
    pub apres: f64,
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
    /// `Relief(tilt:)` : jusqu'où l'on peut tourner la page, en radians. Un demi-tour ou plus :
    /// la rotation est libre, on fait le tour de la page.
    pub angle_max: f64,
}

impl Default for Reglages {
    fn default() -> Self {
        Reglages {
            zoom_actif: true,
            portails_disposition: Disposition::Grille,
            portails_nombre: 12,
            portails_taille: 170.0,
            portails_lumiere: 0.15,
            zoom_vitesse: 1.0,
            portails_duree: 450.0,
            zoom_max: 1e12, reduire: false, niveaux_de_sites: 8, apres: 4.0, taille_point: 6.0, taille_morceler: 40.0, cote: 4, niveaux: 20, densite: 2.0, relief: 10.0, angle_max: 360f64.to_radians() }
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
        seulement(zoom, &["active", "max", "shrink", "levels", "speed"])?;
        r.zoom_vitesse = nombre(zoom, "speed", None, 0.25, 4.0, r.zoom_vitesse)?;
        r.zoom_actif = match zoom.argument("active").map(|a| &a.valeur) {
            None => r.zoom_actif,
            Some(Valeur::Bool(b)) => *b,
            Some(_) => return refus(zoom, "active", "true ou false"),
        };
        r.niveaux_de_sites = entier(zoom, "levels", 1, 16, u64::from(r.niveaux_de_sites))? as u32;
        r.zoom_max = nombre(zoom, "max", None, 1.0, 1e12, r.zoom_max)?;
        r.reduire = match zoom.argument("shrink").map(|a| &a.valeur) {
            None => r.reduire,
            Some(Valeur::Bool(b)) => *b,
            Some(_) => return refus(zoom, "shrink", "true ou false"),
        };
    }
    if let Some(points) = bloc_de(page, "points", "Points")? {
        seulement(points, &["after", "size", "fragment", "grid", "depth", "density"])?;
        r.apres = nombre(points, "after", None, 1.0, 16.0, r.apres)?;
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
        r.angle_max = nombre(relief, "tilt", Some("deg"), 0.0, 360.0, r.angle_max.to_degrees())?.to_radians();
    }
    if let Some(portails) = bloc_de(page, "portals", "Portals")? {
        seulement(portails, &["layout", "count", "size", "brightness", "duration"])?;
        r.portails_duree = nombre(portails, "duration", Some("ms"), 0.0, 2000.0, r.portails_duree)?;
        r.portails_disposition = match portails.argument("layout").map(|a| &a.valeur) {
            None => r.portails_disposition,
            Some(Valeur::Nom(nom)) if nom == "grid" => Disposition::Grille,
            Some(Valeur::Nom(nom)) if nom == "row" => Disposition::Ligne,
            Some(Valeur::Nom(nom)) if nom == "column" => Disposition::Colonne,
            Some(Valeur::Nom(nom)) if nom == "diagonal" => Disposition::Diagonale,
            Some(_) => return refus(portails, "layout", "l'un de ces mots : grid, row, column, diagonal"),
        };
        r.portails_nombre = entier(portails, "count", 1, 64, u64::from(r.portails_nombre))? as u32;
        r.portails_taille = nombre(portails, "size", Some("px"), 80.0, 400.0, r.portails_taille)?;
        r.portails_lumiere = nombre(portails, "brightness", None, 0.0, 1.0, r.portails_lumiere)?;
    }
    // Garde-fou : le zoom ordinaire fait partie du zoom. On ne peut pas grossir la page vivante
    // au-delà de `Zoom(max:)` (revue Codex du 2026-10-03, B-01).
    if r.apres > r.zoom_max {
        let ou = if page.argument("points").is_some() { "points" } else { "zoom" };
        return Err(Erreur {
            message: format!("« Points(after: {}) » dépasse « Zoom(max: {}) » : la page ne peut pas grossir plus que le zoom ne le permet", r.apres, r.zoom_max),
            pos: page.argument(ou).map_or(page.pos, |a| a.pos),
        });
    }
    // Garde-fou : les sites ne s'emboîtent pas plus profond que `Zoom(levels:)`.
    sites_emboites(page, 1, r.niveaux_de_sites)?;
    Ok(r)
}

/// Parcourt les sites contenus dans un site (les points qui ont un `inside`), et refuse celui
/// qui dépasse le nombre de niveaux permis.
fn sites_emboites(site: &Bloc, niveau: u32, max: u32) -> Result<(), Erreur> {
    fn visiter(valeur: &Valeur, niveau: u32, max: u32) -> Result<(), Erreur> {
        match valeur {
            Valeur::Liste(elements) => elements.iter().try_for_each(|e| visiter(e, niveau, max)),
            Valeur::Bloc(bloc) => match (bloc.nom.as_str(), bloc.argument("inside").map(|a| &a.valeur)) {
                ("Point", Some(Valeur::Bloc(monde))) => {
                    if niveau + 1 > max {
                        return Err(Erreur {
                            message: format!("trop de sites emboîtés : ce point ouvrirait un niveau {} alors que la limite est {max} (Zoom(levels: {max}))", niveau + 1),
                            pos: bloc.pos,
                        });
                    }
                    sites_emboites(monde, niveau + 1, max)
                }
                _ => bloc.arguments.iter().try_for_each(|a| visiter(&a.valeur, niveau, max)),
            },
            _ => Ok(()),
        }
    }
    site.arguments.iter().try_for_each(|a| visiter(&a.valeur, niveau, max))
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
        assert_eq!((r.zoom_max, r.reduire, r.niveaux_de_sites, r.apres), (1_000_000.0, false, 8, 4.0));
        assert_eq!((r.taille_point, r.taille_morceler, r.cote, r.niveaux, r.densite), (6.0, 40.0, 4, 20, 2.0));
        assert_eq!(r.relief, 10.0);
        assert!((r.angle_max - 360f64.to_radians()).abs() < 1e-12);
        assert_eq!((r.zoom_vitesse, r.portails_duree), (1.0, 450.0));
    }

    #[test]
    fn chaque_reglage_a_ses_garde_fous() {
        let page = |reglage: &str| format!("Page({reglage})");
        for (reglage, message) in [
            ("zoom: Zoom(max: 0)", "un nombre entre 1 et"),
            ("zoom: Zoom(shrink: 1)", "true ou false"),
            ("zoom: Zoom(turbo: 3)", "n'a pas de paramètre « turbo »"),
            ("zoom: 4", "un bloc « Zoom(...) »"),
            ("zoom: Points(size: 6px)", "un bloc « Zoom(...) »"),
            ("zoom: Zoom(levels: 0)", "entier entre 1 et 16"),
            ("zoom: Zoom(max: 2)", "dépasse « Zoom(max: 2) »"),
            ("zoom: Zoom(max: 1), points: Points(after: 16)", "dépasse « Zoom(max: 1) »"),
            ("zoom: Zoom(active: yes)", "true ou false"),
            ("portals: Portals(layout: circle)", "grid, row, column, diagonal"),
            ("portals: Portals(count: 0)", "entier entre 1 et 64"),
            ("portals: Portals(size: 20px)", "entre 80px et 400px"),
            ("portals: Portals(brightness: 3)", "entre 0 et 1"),
            ("points: Points(after: 40)", "entre 1 et 16"),
            ("points: Points(size: 6)", "entre 2px et 32px"),
            ("points: Points(size: 1px)", "entre 2px et 32px"),
            ("points: Points(grid: 20)", "entier entre 2 et 8"),
            ("points: Points(depth: 99)", "entier entre 0 et 20"),
            ("points: Points(fragment: 12px)", "au moins 24px"),
            ("points: Points(density: 9)", "entre 1 et 3"),
            ("relief: Relief(height: 500px)", "entre 0px et 40px"),
            ("relief: Relief(tilt: 400deg)", "entre 0deg et 360deg"),
            ("relief: Relief(tilt: 30px)", "entre 0deg et 360deg"),
            ("zoom: Zoom(speed: 10)", "entre 0.25 et 4"),
            ("portals: Portals(duration: 5s)", "entre 0ms et 2000ms"),
            ("portals: Portals(duration: 450)", "entre 0ms et 2000ms"),
        ] {
            let erreur = lus(&page(reglage)).unwrap_err();
            assert!(erreur.message.contains(message), "{reglage} → {erreur}");
        }
        assert!(lus(&page("zoom: Zoom(max: 2), points: Points(after: 2)")).is_ok());
        let r = lus(&page("zoom: Zoom(max: 50, shrink: true), points: Points(size: 8px, fragment: 64px, grid: 2, depth: 3), relief: Relief(height: 0px, tilt: 0deg)")).unwrap();
        assert_eq!((r.zoom_max, r.reduire, r.taille_point, r.taille_morceler, r.cote, r.niveaux, r.relief, r.angle_max), (50.0, true, 8.0, 64.0, 2, 3, 0.0, 0.0));
    }

    #[test]
    fn le_carrefour_et_le_zoom_se_reglent() {
        let r = lus("Page(zoom: Zoom(active: false, speed: 2), portals: Portals(layout: diagonal, count: 30, size: 120px, brightness: 0.4, duration: 0ms))").unwrap();
        assert!(!r.zoom_actif);
        assert_eq!((r.zoom_vitesse, r.portails_duree), (2.0, 0.0));
        assert_eq!((r.portails_disposition, r.portails_nombre, r.portails_taille, r.portails_lumiere), (Disposition::Diagonale, 30, 120.0, 0.4));
        let defaut = Reglages::default();
        assert!(defaut.zoom_actif);
        assert_eq!((defaut.portails_disposition, defaut.portails_nombre), (Disposition::Grille, 12));
    }

    #[test]
    fn les_sites_ne_s_emboitent_pas_sans_limite() {
        // Trois sites : la page, A dans la page, B dans A.
        let fichier = |limite: &str| {
            format!("Page({limite}children: [\n Point(name: A, seed: 1, inside: World(children: [\n  Point(name: B, seed: 2, inside: World(children: [ P(\"x\") ])),\n ])),\n])")
        };
        assert!(lus(&fichier("")).is_ok(), "huit niveaux sont permis sans rien écrire");
        assert!(lus(&fichier("zoom: Zoom(levels: 3), ")).is_ok());
        let erreur = lus(&fichier("zoom: Zoom(levels: 2), ")).unwrap_err();
        assert!(erreur.message.contains("la limite est 2"), "{erreur}");
        assert_eq!(erreur.pos.ligne, 3, "l'erreur désigne le point de trop");
        // Les points plantés dans un pixel comptent aussi.
        assert!(lus("Page(zoom: Zoom(levels: 1), pixels: [ Point(name: A, above: A, seed: 1, inside: World(children: [])) ])").is_err());
    }
}
