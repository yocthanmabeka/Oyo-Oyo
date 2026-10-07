//! Ce que le fichier `.holo` dit de la façon dont sa page se regarde : jusqu'où l'on zoome,
//! quand ses pixels deviennent des points, comment ils se morcellent, quel relief ils prennent.
//!
//! ```holo
//! Page(
//!   zoom: Zoom(max: 1000000, shrink: false, levels: 8, speed: 1),
//!   points: Points(after: 4, size: 6px, fragment: 40px, divisions: 4, levels: 20, density: 2),
//!   relief: Relief(height: 10px, tilt: 360deg),   // sans « tilt », la page ne tourne pas
//!   portals: Portals(layout: grid, count: 12, size: 170px, brightness: 0.15, duration: 450ms),
//! )
//! ```
//!
//! Chaque réglage a des bornes : ce sont les garde-fous. Un auteur ne peut pas écrire une
//! page qui demanderait à la machine plus qu'elle ne peut donner (ADR-005).

use crate::holo::{Block, Error, Program, Value};

/// Comment les portails du carrefour se rangent, et dans quel sens on les fait défiler.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layout {
    /// En grille, sur toute la fenêtre.
    Grid,
    /// Sur une ligne : on défile de gauche à droite.
    Line,
    /// Sur une colonne : on défile de haut en bas.
    Column,
    /// En diagonale.
    Diagonal,
}

/// Les réglages de la vue d'une page. Sans rien écrire, on obtient ceux-ci.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// `Zoom(active:)` : faux, le visiteur ne peut pas zoomer dans la page.
    pub zoom_active: bool,
    /// `Portals(layout:)` : comment les portails du carrefour se rangent.
    pub portals_layout: Layout,
    /// `Portals(count:)` : combien de mondes le carrefour montre. Les sites écrits dans le
    /// fichier passent d'abord ; le reste est rempli par des mondes calculés à partir d'une graine.
    pub portals_count: u32,
    /// `Portals(size:)` : la taille d'un portail, en pixels.
    pub portals_size: f64,
    /// `Portals(brightness:)` : la lumière du fond du carrefour, de 0 (aucune) à 1.
    pub portals_light: f64,
    /// Les pixels de la page deviennent-ils des points quand on zoome ? Seulement si l'auteur
    /// l'a demandé, en écrivant `points:` ou en plantant un site dans un pixel (`pixels:`).
    /// Sinon la page reste un site ordinaire, quel que soit le zoom.
    pub active_points: bool,
    /// `Zoom(speed:)` : la vitesse du zoom à la molette. 1 : la vitesse ordinaire.
    pub zoom_speed: f64,
    /// `Portals(duration:)` : la durée, en millisecondes, de l'ouverture d'un portail.
    pub portals_duration: f64,
    /// `Zoom(max:)` : combien de fois on peut grossir la page, au plus.
    pub zoom_max: f64,
    /// `Zoom(shrink:)` : vrai, dézoomer réduit la page jusqu'à un seul point ; faux, on ne
    /// dézoome pas en deçà de la page entière.
    pub reduce: bool,
    /// `Zoom(levels:)` : combien de sites peuvent s'emboîter les uns dans les autres, au plus.
    pub site_levels: u32,
    /// `Points(after:)` : jusqu'à ce grossissement, la page reste un site ordinaire, qu'on lit,
    /// qu'on sélectionne et qu'on copie ; au-delà, ses pixels deviennent des points.
    pub after: f64,
    /// `Points(size:)` : la taille à l'écran (en pixels) où un pixel de la page devient un point.
    pub point_size: f64,
    /// `Points(fragment:)` : la taille où un point se morcelle.
    pub shatter_size: f64,
    /// `Points(divisions:)` : un point se morcelle en une grille de `divisions × divisions`.
    pub side: u64,
    /// `Points(levels:)` : combien de fois de suite un point peut se morceler.
    pub levels: u32,
    /// `Points(density:)` : points par pixel d'écran, dans chaque sens, au repos.
    pub density: f64,
    /// `Relief(height:)` : de combien se soulève ce qui est lumineux, quand la page est de biais.
    pub relief: f64,
    /// `Relief(tilt:)` : jusqu'où l'on peut tourner la page, en radians. Zéro, le départ : la
    /// page ne tourne pas, c'est un site ordinaire ; l'auteur active la rotation en l'écrivant.
    /// Un demi-tour ou plus : la rotation est libre, on fait le tour de la page.
    pub angle_max: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            zoom_active: true,
            portals_layout: Layout::Grid,
            portals_count: 12,
            portals_size: 170.0,
            portals_light: 0.15,
            active_points: false,
            zoom_speed: 1.0,
            portals_duration: 450.0,
            zoom_max: 1e12, reduce: false, site_levels: 8, after: 4.0, point_size: 6.0, shatter_size: 40.0, side: 4, levels: 20, density: 2.0, relief: 10.0, angle_max: 0.0 }
    }
}

fn refusal<T>(block: &Block, param: &str, expected: &str) -> Result<T, Error> {
    let pos = block.argument(param).map_or(block.pos, |a| a.pos);
    Err(Error { message: format!("« {}({param}: …) » attend {expected}", block.name), pos })
}

/// Un nombre, avec l'unité demandée (ou sans unité), compris entre deux bornes.
fn number(block: &Block, param: &str, unit: Option<&str>, min: f64, max: f64, default_value: f64) -> Result<f64, Error> {
    let Some(argument) = block.argument(param) else { return Ok(default_value) };
    let value = match (&argument.value, unit) {
        (Value::Integer(e), None) => Some(*e as f64),
        (Value::Number { value, unit: None, .. }, None) => Some(*value),
        (Value::Number { value, unit: Some(u), .. }, Some(expected_unit)) if u == expected_unit => Some(*value),
        _ => None,
    };
    match value {
        Some(v) if (min..=max).contains(&v) => Ok(v),
        _ => refusal(block, param, &format!("un nombre entre {min}{u} et {max}{u}", u = unit.unwrap_or(""))),
    }
}

fn integer(block: &Block, param: &str, min: u64, max: u64, default_value: u64) -> Result<u64, Error> {
    match block.argument(param).map(|a| &a.value) {
        None => Ok(default_value),
        Some(Value::Integer(e)) if (min..=max).contains(e) => Ok(*e),
        Some(_) => refusal(block, param, &format!("un nombre entier entre {min} et {max}")),
    }
}

/// Un ancien nom, changé le 2026-10-06 (`ADR-047`) : refusé, avec le bon mot, que l'éditeur
/// remplace d'un clic.
fn old_name(block: &Block, old: &str, new_one: &str) -> Result<(), Error> {
    match block.argument(old) {
        Some(a) => Err(Error { message: format!("« {}({old}:) » s'appelle maintenant « {new_one} » : écris « {new_one} »", block.name), pos: a.pos }),
        None => Ok(()),
    }
}

/// Refuse un paramètre que le bloc ne connaît pas, en donnant la liste des possibles.
fn only(block: &Block, known_ones: &[&str]) -> Result<(), Error> {
    for argument in &block.arguments {
        match argument.name.as_deref() {
            Some(name) if known_ones.contains(&name) => {}
            Some(name) => return Err(Error { message: format!("« {} » n'a pas de paramètre « {name} » ; paramètres possibles : {}", block.name, known_ones.join(", ")), pos: argument.pos }),
            None => return Err(Error { message: format!("chaque paramètre de « {} » est nommé : {}", block.name, known_ones.join(", ")), pos: argument.pos }),
        }
    }
    Ok(())
}

/// Le bloc donné à un paramètre de la page : `zoom: Zoom(...)`.
fn block_of<'a>(page: &'a Block, param: &str, name: &str) -> Result<Option<&'a Block>, Error> {
    match page.argument(param).map(|a| &a.value) {
        None => Ok(None),
        Some(Value::Block(block)) if block.name == name => Ok(Some(block)),
        Some(_) => refusal(page, param, &format!("un bloc « {name}(...) »")),
    }
}

/// Lit et vérifie les réglages de vue écrits dans la page. Sans eux, les réglages par défaut.
pub fn settings(program: &Program) -> Result<Settings, Error> {
    let mut r = Settings::default();
    let page = &program.root;
    if page.name != "Page" {
        return Ok(r);
    }
    if let Some(zoom) = block_of(page, "zoom", "Zoom")? {
        only(zoom, &["active", "max", "shrink", "levels", "speed"])?;
        r.zoom_speed = number(zoom, "speed", None, 0.25, 4.0, r.zoom_speed)?;
        r.zoom_active = match zoom.argument("active").map(|a| &a.value) {
            None => r.zoom_active,
            Some(Value::Bool(b)) => *b,
            Some(_) => return refusal(zoom, "active", "true ou false"),
        };
        r.site_levels = integer(zoom, "levels", 1, 16, u64::from(r.site_levels))? as u32;
        r.zoom_max = number(zoom, "max", None, 1.0, 1e12, r.zoom_max)?;
        r.reduce = match zoom.argument("shrink").map(|a| &a.value) {
            None => r.reduce,
            Some(Value::Bool(b)) => *b,
            Some(_) => return refusal(zoom, "shrink", "true ou false"),
        };
    }
    if let Some(points) = block_of(page, "points", "Points")? {
        old_name(points, "grid", "divisions")?;
        old_name(points, "depth", "levels")?;
        only(points, &["after", "size", "fragment", "divisions", "levels", "density"])?;
        // Jamais moins de 2 : tout visiteur peut au moins doubler la taille du texte avant que
        // la page ne change de nature (accessibilité, WCAG 1.4.4).
        r.after = number(points, "after", None, 2.0, 16.0, r.after)?;
        r.point_size = number(points, "size", Some("px"), 2.0, 32.0, r.point_size)?;
        r.side = integer(points, "divisions", 2, 8, r.side)?;
        r.levels = integer(points, "levels", 0, 20, u64::from(r.levels))? as u32;
        r.density = number(points, "density", None, 1.0, 3.0, r.density)?;
        r.shatter_size = number(points, "fragment", Some("px"), 8.0, 400.0, r.shatter_size)?;
        // Garde-fou : un point qui vient de se morceler doit rester visible, et il ne doit
        // jamais y avoir plus de points à l'écran que quand ils apparaissent.
        if r.shatter_size / (r.side as f64) < r.point_size {
            return refusal(points, "fragment", &format!("au moins {}px : « fragment » divisé par « divisions » ne doit pas être plus petit que « size »", r.point_size * r.side as f64));
        }
    }
    if let Some(relief) = block_of(page, "relief", "Relief")? {
        only(relief, &["height", "tilt"])?;
        r.relief = number(relief, "height", Some("px"), 0.0, 40.0, r.relief)?;
        r.angle_max = number(relief, "tilt", Some("deg"), 0.0, 360.0, r.angle_max.to_degrees())?.to_radians();
    }
    if let Some(portals) = block_of(page, "portals", "Portals")? {
        only(portals, &["layout", "count", "size", "brightness", "duration"])?;
        r.portals_duration = number(portals, "duration", Some("ms"), 0.0, 2000.0, r.portals_duration)?;
        r.portals_layout = match portals.argument("layout").map(|a| &a.value) {
            None => r.portals_layout,
            Some(Value::Name(name)) if name == "grid" => Layout::Grid,
            Some(Value::Name(name)) if name == "row" => Layout::Line,
            Some(Value::Name(name)) if name == "column" => Layout::Column,
            Some(Value::Name(name)) if name == "diagonal" => Layout::Diagonal,
            Some(_) => return refusal(portals, "layout", "l'un de ces mots : grid, row, column, diagonal"),
        };
        r.portals_count = integer(portals, "count", 1, 64, u64::from(r.portals_count))? as u32;
        r.portals_size = number(portals, "size", Some("px"), 80.0, 400.0, r.portals_size)?;
        r.portals_light = number(portals, "brightness", None, 0.0, 1.0, r.portals_light)?;
    }
    // Les points s'activent : en écrivant `points:`, ou en plantant un site dans un pixel, qu'on
    // ne trouve qu'en vue points. Sans cela, la page reste un site ordinaire.
    let mut planted_pixels = false;
    let _ = crate::rules::for_each_block(page, &mut |block| {
        planted_pixels |= block.argument("pixels").is_some();
        Ok(())
    });
    r.active_points = page.argument("points").is_some() || planted_pixels;
    // Le relief est celui des points : sans eux, il n'a rien à soulever ni à faire tourner.
    if let (false, Some(relief)) = (r.active_points, page.argument("relief")) {
        return Err(Error {
            message: "« relief » demande les points : ajoute « points: Points() » à la page (le relief est celui des points, et c'est en points que la page tourne)".into(),
            pos: relief.pos,
        });
    }
    // Sans les points, le zoom ordinaire s'arrête de lui-même à ce que `Zoom(max:)` permet.
    if !r.active_points {
        r.after = r.after.min(r.zoom_max);
    }
    // Garde-fou : le zoom ordinaire fait partie du zoom. On ne peut pas grossir la page vivante
    // au-delà de `Zoom(max:)` (revue Codex du 2026-10-03, B-01).
    if r.after > r.zoom_max {
        let or_ = if page.argument("points").is_some() { "points" } else { "zoom" };
        return Err(Error {
            message: format!("« Points(after: {}) » dépasse « Zoom(max: {}) » : la page ne peut pas grossir plus que le zoom ne le permet", r.after, r.zoom_max),
            pos: page.argument(or_).map_or(page.pos, |a| a.pos),
        });
    }
    // Garde-fou : les sites ne s'emboîtent pas plus profond que `Zoom(levels:)`.
    nested_sites(page, 1, r.site_levels)?;
    Ok(r)
}

/// Parcourt les sites contenus dans un site (les points qui ont un `inside`), et refuse celui
/// qui dépasse le nombre de niveaux permis.
fn nested_sites(site: &Block, level: u32, max: u32) -> Result<(), Error> {
    fn visit(value: &Value, level: u32, max: u32) -> Result<(), Error> {
        match value {
            Value::List(elements) => elements.iter().try_for_each(|e| visit(e, level, max)),
            Value::Block(block) => match (block.name.as_str(), block.argument("inside").map(|a| &a.value)) {
                ("Point", Some(Value::Block(world))) => {
                    if level + 1 > max {
                        return Err(Error {
                            message: format!("trop de sites emboîtés : ce point ouvrirait un niveau {} alors que la limite est {max} (Zoom(levels: {max}))", level + 1),
                            pos: block.pos,
                        });
                    }
                    nested_sites(world, level + 1, max)
                }
                _ => block.arguments.iter().try_for_each(|a| visit(&a.value, level, max)),
            },
            _ => Ok(()),
        }
    }
    site.arguments.iter().try_for_each(|a| visit(&a.value, level, max))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::read;

    fn read_ones(src: &str) -> Result<Settings, Error> {
        settings(&read(src)?)
    }

    #[test]
    fn without_writing_anything_we_get_default_settings() {
        assert_eq!(read_ones("Page(title: \"x\")").unwrap(), Settings::default());
        assert_eq!(read_ones("Point(name: A, seed: 1)").unwrap(), Settings::default());
    }

    #[test]
    fn the_shop_writes_its_settings() {
        let r = read_ones(include_str!("../../exemples/boutique-comparee/boutique.holo")).unwrap();
        assert_eq!((r.zoom_max, r.reduce, r.site_levels, r.after), (1_000_000.0, false, 8, 4.0));
        assert_eq!((r.point_size, r.shatter_size, r.side, r.levels, r.density), (6.0, 40.0, 4, 20, 2.0));
        assert_eq!(r.relief, 10.0);
        assert!((r.angle_max - 360f64.to_radians()).abs() < 1e-12);
        assert_eq!((r.zoom_speed, r.portals_duration), (1.0, 450.0));
    }

    #[test]
    fn each_setting_has_its_guardrails() {
        let page = |setting: &str| format!("Page({setting})");
        for (setting, message) in [
            ("zoom: Zoom(max: 0)", "un nombre entre 1 et"),
            ("zoom: Zoom(shrink: 1)", "true ou false"),
            ("zoom: Zoom(turbo: 3)", "n'a pas de paramètre « turbo »"),
            ("zoom: 4", "un bloc « Zoom(...) »"),
            ("zoom: Points(size: 6px)", "un bloc « Zoom(...) »"),
            ("zoom: Zoom(levels: 0)", "entier entre 1 et 16"),
            ("zoom: Zoom(max: 2), points: Points()", "dépasse « Zoom(max: 2) »"),
            ("zoom: Zoom(max: 2), pixels: []", "dépasse « Zoom(max: 2) »"),
            ("relief: Relief(tilt: 360deg)", "« relief » demande les points"),
            ("points: Points(after: 1)", "entre 2 et 16"),
            ("zoom: Zoom(max: 1), points: Points(after: 16)", "dépasse « Zoom(max: 1) »"),
            ("zoom: Zoom(active: yes)", "true ou false"),
            ("portals: Portals(layout: circle)", "grid, row, column, diagonal"),
            ("portals: Portals(count: 0)", "entier entre 1 et 64"),
            ("portals: Portals(size: 20px)", "entre 80px et 400px"),
            ("portals: Portals(brightness: 3)", "entre 0 et 1"),
            ("points: Points(after: 40)", "entre 2 et 16"),
            ("points: Points(size: 6)", "entre 2px et 32px"),
            ("points: Points(size: 1px)", "entre 2px et 32px"),
            ("points: Points(divisions: 20)", "entier entre 2 et 8"),
            ("points: Points(levels: 99)", "entier entre 0 et 20"),
            ("points: Points(grid: 4)", "s'appelle maintenant « divisions » : écris « divisions »"),
            ("points: Points(depth: 6)", "s'appelle maintenant « levels » : écris « levels »"),
            ("points: Points(fragment: 12px)", "au moins 24px"),
            ("points: Points(density: 9)", "entre 1 et 3"),
            ("points: Points(), relief: Relief(height: 500px)", "entre 0px et 40px"),
            ("points: Points(), relief: Relief(tilt: 400deg)", "entre 0deg et 360deg"),
            ("points: Points(), relief: Relief(tilt: 30px)", "entre 0deg et 360deg"),
            ("zoom: Zoom(speed: 10)", "entre 0.25 et 4"),
            ("portals: Portals(duration: 5s)", "entre 0ms et 2000ms"),
            ("portals: Portals(duration: 450)", "entre 0ms et 2000ms"),
        ] {
            let error = read_ones(&page(setting)).unwrap_err();
            assert!(error.message.contains(message), "{setting} → {error}");
        }
        assert!(read_ones(&page("zoom: Zoom(max: 2), points: Points(after: 2)")).is_ok());
        // Sans rien écrire, une page ne devient pas des points : c'est un site ordinaire, et son
        // zoom s'arrête à ce que « Zoom(max:) » permet.
        let simple = read_ones(&page("zoom: Zoom(max: 2)")).unwrap();
        assert_eq!((simple.active_points, simple.after), (false, 2.0));
        assert!(!read_ones(&page("title: \"x\"")).unwrap().active_points);
        // Les points s'activent en les écrivant, ou en plantant un site dans un pixel.
        assert!(read_ones(&page("points: Points()")).unwrap().active_points);
        assert!(read_ones(&page("children: [ Button(name: B, text: \"x\") ], pixels: [ Point(name: S, above: B, seed: 1) ]")).unwrap().active_points);
        assert!(read_ones(&page("children: [ Point(name: A, seed: 1, inside: World(children: [ Button(name: B, text: \"x\") ], pixels: [ Point(name: S, above: B, seed: 2) ])) ]")).unwrap().active_points);
        let r = read_ones(&page("zoom: Zoom(max: 50, shrink: true), points: Points(size: 8px, fragment: 64px, divisions: 2, levels: 3), relief: Relief(height: 0px, tilt: 0deg)")).unwrap();
        assert_eq!((r.zoom_max, r.reduce, r.point_size, r.shatter_size, r.side, r.levels, r.relief, r.angle_max), (50.0, true, 8.0, 64.0, 2, 3, 0.0, 0.0));
    }

    #[test]
    fn crossroads_and_zoom_are_configurable() {
        let r = read_ones("Page(zoom: Zoom(active: false, speed: 2), portals: Portals(layout: diagonal, count: 30, size: 120px, brightness: 0.4, duration: 0ms))").unwrap();
        assert!(!r.zoom_active);
        assert_eq!((r.zoom_speed, r.portals_duration), (2.0, 0.0));
        assert_eq!((r.portals_layout, r.portals_count, r.portals_size, r.portals_light), (Layout::Diagonal, 30, 120.0, 0.4));
        let default_value = Settings::default();
        assert!(default_value.zoom_active);
        assert_eq!((default_value.portals_layout, default_value.portals_count), (Layout::Grid, 12));
    }

    #[test]
    fn sites_do_not_nest_without_limit() {
        // Trois sites : la page, A dans la page, B dans A.
        let file = |limit: &str| {
            format!("Page({limit}children: [\n Point(name: A, seed: 1, inside: World(children: [\n  Point(name: B, seed: 2, inside: World(children: [ P(\"x\") ])),\n ])),\n])")
        };
        assert!(read_ones(&file("")).is_ok(), "huit niveaux sont permis sans rien écrire");
        assert!(read_ones(&file("zoom: Zoom(levels: 3), ")).is_ok());
        let error = read_ones(&file("zoom: Zoom(levels: 2), ")).unwrap_err();
        assert!(error.message.contains("la limite est 2"), "{error}");
        assert_eq!(error.pos.line, 3, "l'erreur désigne le point de trop");
        // Les points plantés dans un pixel comptent aussi.
        assert!(read_ones("Page(zoom: Zoom(levels: 1), pixels: [ Point(name: A, above: A, seed: 1, inside: World(children: [])) ])").is_err());
    }
}
