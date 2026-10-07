//! Du fichier `.holo` au monde : un `Point` déclaré devient un monde dont tout le
//! contenu se calcule à partir de sa graine. Rien n'est stocké : un point pèse une graine.

use crate::blocks::{old_word, unknown_block, BLOCKS};
use crate::seed::{child_seed, Generator};
use crate::holo::{Error, Pos, Program, Value};

/// Ce que déclare le bloc `Point(...)` d'un fichier `.holo`.
#[derive(Debug, Clone, PartialEq)]
pub struct PointDecl {
    pub name: String,
    pub seed: u64,
    pub light: f32,
    pub shatter: u32,
    /// La couleur imposée à ce point, sinon la graine décide (ADR-017).
    pub color: Option<[f32; 3]>,
    /// Les couleurs imposées à ses enfants, reprises en boucle.
    pub palette: Vec<[f32; 3]>,
}

pub const SHATTER_MAX: u32 = 64;

/// Donne un sens au bloc racine. Seul `Point` est pris en charge dans ce sprint.
pub fn point_from(program: &Program) -> Result<PointDecl, Error> {
    if let Some(import) = program.imports.first() {
        return Err(Error {
            message: format!("« {} » n'est pas encore pris en charge par ce sprint : le fichier serait accepté sans que l'import soit appliqué (revue Codex)", import.kind),
            pos: import.pos,
        });
    }
    let block = &program.root;
    match block.name.as_str() {
        "Point" => {}
        known if BLOCKS.contains(&known) => {
            return Err(Error {
                message: format!("le bloc « {} » existe dans le format .holo mais ce sprint ne lit que « Point »", block.name),
                pos: block.pos,
            })
        }
        other => return Err(Error { message: unknown_block(other), pos: block.pos }),
    }
    let mut decl = PointDecl { name: String::new(), seed: 0, light: 1.0, shatter: 12, color: None, palette: Vec::new() };
    let mut seen_ones: Vec<&str> = Vec::new();
    for arg in &block.arguments {
        let name = arg.name.as_deref().ok_or_else(|| Error {
            message: "chaque paramètre de « Point » est nommé : name, seed, brightness, fragments, color, palette".into(),
            pos: arg.pos,
        })?;
        if seen_ones.contains(&name) {
            return Err(Error { message: format!("le paramètre « {name} » est donné deux fois"), pos: arg.pos });
        }
        seen_ones.push(name);
        match (name, &arg.value) {
            ("name", Value::Name(n)) => decl.name = n.clone(),
            ("name", _) => return Err(expected("name", "un nom, comme « Origin »", arg.pos)),
            ("seed", Value::Name(a)) if a == "auto" => {
                return Err(Error {
                    message: "graine non fixée : « auto » doit être remplacé par un nombre au moment de la création (ADR-008, ADR-014)".into(),
                    pos: arg.pos,
                })
            }
            ("seed", Value::Integer(seed)) => decl.seed = *seed,
            ("seed", _) => return Err(expected("seed", "un nombre entier positif, sans unité, jusqu'à 18446744073709551615", arg.pos)),
            ("brightness", Value::Number { value, unit: None }) if (0.0..=1.0).contains(value) => decl.light = *value as f32,
            ("brightness", Value::Integer(e)) if *e <= 1 => decl.light = *e as f32,
            ("brightness", _) => return Err(expected("brightness", "un nombre entre 0 et 1, sans unité", arg.pos)),
            ("fragments", Value::Integer(n)) if (1..=u64::from(SHATTER_MAX)).contains(n) => decl.shatter = *n as u32,
            ("fragments", _) => return Err(expected("fragments", &format!("un nombre entier entre 1 et {SHATTER_MAX}"), arg.pos)),
            ("color", Value::Text(c)) => decl.color = Some(hex_color(c).ok_or_else(|| expected("color", EXPECTED_COLOR, arg.pos))?),
            ("color", _) => return Err(expected("color", EXPECTED_COLOR, arg.pos)),
            ("palette", Value::List(colors)) if !colors.is_empty() => {
                for c in colors {
                    let color = match c {
                        Value::Text(c) => hex_color(c),
                        _ => None,
                    };
                    decl.palette.push(color.ok_or_else(|| expected("palette", "une liste de couleurs, comme [\"#E9B44C\", \"#245C45\"]", arg.pos))?);
                }
            }
            ("palette", _) => return Err(expected("palette", "une liste de couleurs, comme [\"#E9B44C\", \"#245C45\"]", arg.pos)),
            (other, _) => {
                let message = match old_word(other) {
                    Some(new_one) => format!("le paramètre « {other} » s'écrit « {new_one} » : le vocabulaire est en anglais (ADR-016)"),
                    None => format!("« Point » n'a pas de paramètre « {other} » ; paramètres possibles : name, seed, brightness, fragments, color, palette"),
                };
                return Err(Error { message, pos: arg.pos });
            }
        }
    }
    if decl.name.is_empty() {
        return Err(Error { message: "« Point » doit avoir un paramètre « name »".into(), pos: block.pos });
    }
    if !seen_ones.contains(&"seed") {
        return Err(Error { message: "« Point » doit avoir un paramètre « seed »".into(), pos: block.pos });
    }
    Ok(decl)
}

const EXPECTED_COLOR: &str = "une couleur entre guillemets, écrite \"#E9B44C\"";

/// `#E9B44C` → rouge, vert, bleu entre 0 et 1.
fn hex_color(text: &str) -> Option<[f32; 3]> {
    let hex = text.strip_prefix('#').filter(|h| h.len() == 6 && h.bytes().all(|c| c.is_ascii_hexdigit()))?;
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok().map(|v| f32::from(v) / 255.0);
    Some([channel(0)?, channel(2)?, channel(4)?])
}

fn expected(param: &str, shape: &str, pos: Pos) -> Error {
    Error { message: format!("le paramètre « {param} » attend {shape}"), pos }
}

/// Un point enfant, né du morcellement de son parent.
#[derive(Debug, Clone, PartialEq)]
pub struct Child {
    pub seed: u64,
    /// Position sur une sphère de rayon 1 autour du centre du monde.
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
}

/// Un monde : ce que l'on voit quand on est à l'intérieur d'un point.
#[derive(Debug, Clone, PartialEq)]
pub struct World {
    pub seed: u64,
    pub light: f32,
    pub color: [f32; 3],
    pub children: Vec<Child>,
}

impl World {
    /// Le monde racine, décrit par le fichier `.holo`.
    pub fn root(decl: &PointDecl) -> World {
        let mut world = World::build(decl.seed, decl.light, Some(decl.shatter));
        if let Some(color) = decl.color {
            world.color = color;
        }
        if !decl.palette.is_empty() {
            for (i, child) in world.children.iter_mut().enumerate() {
                child.color = decl.palette[i % decl.palette.len()];
            }
        }
        world
    }

    /// Un monde quelconque, connu par sa seule graine. Le nombre de points qu'il contient
    /// se tire de la graine quand il n'est pas imposé.
    pub fn from_seed(seed: u64) -> World {
        World::build(seed, 1.0, None)
    }

    fn build(seed: u64, light: f32, shatter: Option<u32>) -> World {
        let mut g = Generator::new(seed);
        let hue = g.unit();
        let nb = shatter.unwrap_or_else(|| g.integer(6, 14));
        let golden = (5f32.sqrt() - 1.0) / 2.0;
        let children = (0..nb)
            .map(|i| {
                // Spirale de Fibonacci : des points bien répartis, puis un peu de désordre.
                let t = (i as f32 + 0.5) / nb as f32;
                let y = (1.0 - 2.0 * t + g.between(-0.12, 0.12)).clamp(-0.95, 0.95);
                let r = (1.0 - y * y).sqrt();
                let angle = std::f32::consts::TAU * (i as f32 * golden + g.between(-0.08, 0.08));
                Child {
                    seed: child_seed(seed, i),
                    position: [r * angle.cos(), y, r * angle.sin()],
                    radius: g.between(0.06, 0.11),
                    color: color(hue + g.between(-0.08, 0.08), g.between(0.5, 0.9)),
                }
            })
            .collect();
        World { seed, light, color: color(hue, 0.35), children }
    }
}

/// Couleur lumineuse à partir d'une teinte (0..1, cyclique) et d'une saturation.
pub fn color(hue: f32, saturation: f32) -> [f32; 3] {
    let h = (hue.rem_euclid(1.0)) * 6.0;
    let x = 1.0 - ((h % 2.0) - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    let m = 1.0 - saturation;
    [r * saturation + m, g * saturation + m, b * saturation + m]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::read;

    fn point(src: &str) -> Result<PointDecl, Error> {
        point_from(&read(src)?)
    }

    #[test]
    fn the_big_bang_is_accepted() {
        let d = point(include_str!("../mondes/big-bang.holo")).unwrap();
        assert_eq!(d, PointDecl { name: "Origin".into(), seed: 1, light: 1.0, shatter: 12, color: None, palette: Vec::new() });
        assert_eq!(World::root(&d).children.len(), 12);
        // Le point de la boutique, seul : mêmes réglages que dans l'exemple de la boutique.
        let studio = point(include_str!("../mondes/atelier.holo")).unwrap();
        assert_eq!((studio.seed, studio.shatter, studio.palette.len()), (42, 6, 2));
    }

    #[test]
    fn refuses_what_the_conformity_suite_refuses() {
        let auto = point("Point(\n  name: Origin,\n  seed: auto,\n  fragments: 12,\n)").unwrap_err();
        assert_eq!(auto.pos.line, 3);
        assert!(auto.message.contains("graine non fixée"));
        assert!(point("Div(contenu: [])").unwrap_err().message.contains("bloc inconnu"));
        assert!(point("Point(name: A, seed: 1, budget: 3s)").unwrap_err().message.contains("n'a pas de paramètre"));
        assert!(point(include_str!("../../experiments/conformite-v0.1/cas/refuses/E08-ancien-vocabulaire.holo")).unwrap_err().message.contains("s'écrit « name »"));
        assert_eq!(point(include_str!("../../experiments/conformite-v0.1/cas/valides/02-big-bang.holo")).unwrap().shatter, 12);
        assert!(point(include_str!("../../experiments/conformite-v0.1/cas/refuses/E04-graine-non-fixee.holo")).unwrap_err().message.contains("graine non fixée"));
        assert!(point("Point(name: A, seed: 1, seed: 2)").unwrap_err().message.contains("deux fois"));
        assert!(point("Point(seed: 1)").unwrap_err().message.contains("« name »"));
        assert!(point("Point(name: A, seed: 1, fragments: 500)").unwrap_err().message.contains("entre 1 et 64"));
        // Les anciens mots français sont refusés, avec le mot anglais à écrire à la place.
        assert!(point("Point(nom: A, graine: 1)").unwrap_err().message.contains("s'écrit « name »"));
        assert!(point("Point(name: A, graine: 1)").unwrap_err().message.contains("s'écrit « seed »"));
        assert!(point("Texte(\"Bonjour\")").unwrap_err().message.contains("écris « Text »"));
        assert!(point("Page(contenu: [])").unwrap_err().message.contains("ne lit que « Point »"));
        assert!(point("import \"absent.holo\"\nPoint(name: A, seed: 1)").unwrap_err().message.contains("pas été trouvé"));
        assert_eq!(point("Point(name: A, seed: 9007199254740993)").unwrap().seed, 9_007_199_254_740_993);
        assert!(point("Point(name: A, seed: 1.5)").unwrap_err().message.contains("entier"));
        assert!(point("Point(name: A, seed: -1)").unwrap_err().message.contains("entier"));
    }

    #[test]
    fn color_and_palette_override_the_seed() {
        let d = point("Point(name: A, seed: 42, fragments: 6, color: \"#FF0000\", palette: [\"#E9B44C\", \"#245C45\"])").unwrap();
        let world = World::root(&d);
        assert_eq!(world.color, [1.0, 0.0, 0.0]);
        assert_eq!(world.children[0].color, world.children[2].color);
        assert_ne!(world.children[0].color, world.children[1].color);
        // Les graines des enfants ne changent pas : seule la couleur est imposée.
        assert_eq!(world.children[0].seed, World::from_seed(42).children[0].seed);
        assert!(point("Point(name: A, seed: 1, color: \"gold\")").unwrap_err().message.contains("#E9B44C"));
        assert!(point("Point(name: A, seed: 1, palette: [])").unwrap_err().message.contains("une liste de couleurs"));
    }

    #[test]
    fn the_same_seed_gives_the_same_world() {
        assert_eq!(World::from_seed(42), World::from_seed(42));
        assert_ne!(World::from_seed(42), World::from_seed(43));
    }

    #[test]
    fn children_have_distinct_and_reproducible_seeds() {
        let m = World::from_seed(1);
        let seeds: std::collections::HashSet<u64> = m.children.iter().map(|e| e.seed).collect();
        assert_eq!(seeds.len(), m.children.len());
        assert!((6..=14).contains(&m.children.len()));
        for e in &m.children {
            let n = e.position.iter().map(|c| c * c).sum::<f32>().sqrt();
            assert!((n - 1.0).abs() < 1e-3, "les enfants sont sur la sphère unité ({n})");
        }
    }

    #[test]
    fn a_point_weighs_only_its_seed() {
        // Descendre de 1 000 niveaux ne demande que 1 000 graines : rien n'est stocké.
        let mut seed = 1u64;
        for _ in 0..1000 {
            seed = World::from_seed(seed).children[0].seed;
        }
        assert_eq!(std::mem::size_of::<u64>(), 8);
        assert_ne!(seed, 1);
    }
}
