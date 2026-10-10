//! Les filtres d'image dans les styles (ADR-108) : `grayscale`, `saturate`, `brightness`,
//! `contrast`, `hue` et `blur`, un réglage par effet, que le moteur compose en un seul `filter`
//! CSS, toujours dans le même ordre ; dans un état, un réglage change sans effacer les autres,
//! là où la liste `filter` du CSS s'efface entière. Et `backdrop-blur` : ce qui est derrière le
//! bloc devient flou, toute la page derrière une fenêtre.
//!
//! Le contraste d'un texte sur son fond reste mesuré, filtre compris (ADR-055) : les mêmes
//! matrices que le navigateur, appliquées aux deux couleurs. Et le flou ne se pose jamais sur un
//! texte, qui ne se lirait plus.

use crate::holo::{Error, Program, Setting, StyleRule, Target};

/// Les six réglages, dans l'ordre où le moteur les applique (gris, saturation, luminosité,
/// contraste, teinte, flou), et la fonction CSS de chacun.
pub const FILTERS: &[(&str, &str)] = &[
    ("grayscale", "grayscale"),
    ("saturate", "saturate"),
    ("brightness", "brightness"),
    ("contrast", "contrast"),
    ("hue", "hue-rotate"),
    ("blur", "blur"),
];

/// Le flou de ce qui est derrière le bloc.
pub const BACKDROP: &str = "backdrop-blur";

/// Les blocs sur lesquels un flou se pose : ils ne portent pas de texte.
const BLURRABLE: &[&str] = &["Image", "Shape", "Video", "Drawing"];

/// Un réglage de filtre ?
pub fn is_filter(name: &str) -> bool {
    FILTERS.iter().any(|(known, _)| *known == name)
}

/// La déclaration `filter:…;` d'un style, ou de l'un de ses états (`state`), dont les réglages
/// passent par-dessus ceux du style. `value` donne chaque valeur telle que le navigateur la
/// reçoit. Vide quand rien n'est filtré, ou quand l'état ne touche à aucun filtre (il garde
/// alors ceux du style).
pub fn declaration(base: &[Setting], state: Option<&[Setting]>, value: &dyn Fn(&Setting) -> String) -> String {
    if state.is_some_and(|settings| !settings.iter().any(|s| is_filter(&s.name))) {
        return String::new();
    }
    let functions: Vec<String> = FILTERS
        .iter()
        .filter_map(|(name, function)| {
            let setting = state.and_then(|settings| settings.iter().find(|s| s.name == *name)).or_else(|| base.iter().find(|s| s.name == *name))?;
            Some(format!("{function}({})", value(setting)))
        })
        .collect();
    if functions.is_empty() {
        String::new()
    } else {
        format!("filter:{};", functions.join(" "))
    }
}

/// `backdrop-blur: 8px` → les deux déclarations, avec le préfixe que Safari a longtemps exigé.
pub fn backdrop_declaration(value: &str) -> String {
    format!("-webkit-backdrop-filter:blur({value});backdrop-filter:blur({value});")
}

/// Les filtres d'un style ou d'un état qui changent les couleurs, écrits pour un message :
/// « , une fois « brightness: 0.4 » appliqué » ; vide s'il n'y en a pas.
pub fn applied(settings: &[Setting]) -> String {
    let written: Vec<String> = FILTERS.iter().filter(|(name, _)| *name != "blur").filter_map(|(name, _)| settings.iter().find(|s| s.name == *name)).map(|s| format!("« {}: {} »", s.name, s.value)).collect();
    if written.is_empty() {
        String::new()
    } else {
        format!(", une fois {} appliqué", written.join(" et "))
    }
}

/// Les couleurs d'un texte et de son fond (rouge, vert, bleu, de 0 à 255), telles que le
/// navigateur les montre une fois les filtres du style appliqués : les matrices des fonctions
/// de `filter`, dans l'espace sRGB, chacune bornée à sa sortie. Le flou ne change pas une couleur.
pub fn seen(settings: &[Setting], colors: [[f64; 3]; 2]) -> [[f64; 3]; 2] {
    let number = |name: &str| settings.iter().find(|s| s.name == name).and_then(|s| s.value.trim_end_matches("deg").parse::<f64>().ok());
    let clamp = |c: [f64; 3]| c.map(|v| v.clamp(0.0, 1.0));
    colors.map(|color| {
        let mut c = color.map(|v| v / 255.0);
        if let Some(amount) = number("grayscale") {
            c = clamp(multiply(grayscale(amount), c));
        }
        if let Some(s) = number("saturate") {
            c = clamp(multiply(saturate(s), c));
        }
        if let Some(b) = number("brightness") {
            c = clamp(c.map(|v| v * b));
        }
        if let Some(k) = number("contrast") {
            c = clamp(c.map(|v| (v - 0.5) * k + 0.5));
        }
        if let Some(angle) = number("hue") {
            c = clamp(multiply(hue_rotate(angle), c));
        }
        c.map(|v| v * 255.0)
    })
}

fn multiply(m: [[f64; 3]; 3], c: [f64; 3]) -> [f64; 3] {
    m.map(|row| row[0] * c[0] + row[1] * c[1] + row[2] * c[2])
}

/// La matrice de `grayscale(a)` du standard Filter Effects.
fn grayscale(a: f64) -> [[f64; 3]; 3] {
    let r = 1.0 - a;
    [
        [0.2126 + 0.7874 * r, 0.7152 - 0.7152 * r, 0.0722 - 0.0722 * r],
        [0.2126 - 0.2126 * r, 0.7152 + 0.2848 * r, 0.0722 - 0.0722 * r],
        [0.2126 - 0.2126 * r, 0.7152 - 0.7152 * r, 0.0722 + 0.9278 * r],
    ]
}

/// La matrice de `saturate(s)`.
fn saturate(s: f64) -> [[f64; 3]; 3] {
    [
        [0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s],
        [0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s],
        [0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s],
    ]
}

/// La matrice de `hue-rotate(angle)`, l'angle en degrés.
fn hue_rotate(angle: f64) -> [[f64; 3]; 3] {
    let (sin, cos) = angle.to_radians().sin_cos();
    [
        [0.213 + cos * 0.787 - sin * 0.213, 0.715 - cos * 0.715 - sin * 0.715, 0.072 - cos * 0.072 + sin * 0.928],
        [0.213 - cos * 0.213 + sin * 0.143, 0.715 + cos * 0.285 + sin * 0.140, 0.072 - cos * 0.072 - sin * 0.283],
        [0.213 - cos * 0.213 - sin * 0.787, 0.715 - cos * 0.715 + sin * 0.715, 0.072 + cos * 0.928 + sin * 0.072],
    ]
}

/// Le flou se pose sur une image, une forme, une vidéo ou un dessin : jamais sur un bloc qui
/// porte un texte, qui ne se lirait plus, ni sur un composant.
pub fn check_blur(rule: &StyleRule, program: &Program) -> Result<(), Error> {
    let Some(blur) = rule.settings.iter().chain(rule.states.iter().flat_map(|(_, settings, _)| settings.iter())).find(|s| s.name == "blur") else { return Ok(()) };
    let offender: Option<String> = match &rule.target {
        Target::Type(t) if BLURRABLE.contains(&t.as_str()) => None,
        Target::Type(t) => Some(t.clone()),
        Target::Name(n) => {
            let mut found = None;
            let _ = crate::rules::for_each_block(&program.root, &mut |block| {
                if found.is_none() && block.styles.iter().any(|s| s == n) && !BLURRABLE.contains(&block.name.as_str()) {
                    found = Some(block.name.clone());
                }
                Ok(())
            });
            found
        }
    };
    match offender {
        None => Ok(()),
        Some(what) => Err(Error {
            message: format!(
                "« blur » dans « {} » : un texte flou ne se lit pas ; le flou se pose sur une image, une forme, une vidéo ou un dessin (« Image.photo {{ blur: 4px; }} »), pas sur « {what} » ; pour estomper un bloc, « opacity » ; pour ce qui est derrière une fenêtre, « backdrop-blur » (ADR-108)",
                rule.target
            ),
            pos: blur.pos,
        }),
    }
}

/// Un flou de derrière ne se voit qu'à travers un fond à demi transparent : avec un fond opaque
/// dans le même style, il est refusé. Une fenêtre (`Dialog`) fait exception : son flou vaut pour
/// toute la page derrière elle.
pub fn check_backdrop(rule: &StyleRule) -> Result<(), Error> {
    if matches!(&rule.target, Target::Type(t) if t == "Dialog") {
        return Ok(());
    }
    let mut mixes: Vec<Vec<&Setting>> = vec![rule.settings.iter().collect()];
    for (_, settings, _) in &rule.states {
        let mut mix: Vec<&Setting> = rule.settings.iter().filter(|r| !settings.iter().any(|s| s.name == r.name)).collect();
        mix.extend(settings.iter());
        mixes.push(mix);
    }
    for mix in mixes {
        let (Some(behind), Some(background)) = (mix.iter().find(|s| s.name == BACKDROP), mix.iter().find(|s| s.name == "background")) else { continue };
        if opaque(&background.value) {
            return Err(Error {
                message: format!(
                    "« backdrop-blur » dans « {} » : avec le fond opaque « {} », rien ne se verrait derrière ; écris un fond à demi transparent, comme « #10102080 », ou pose le flou de derrière sur une fenêtre, « Dialog {{ backdrop-blur: 6px; }} » (ADR-108)",
                    rule.target, background.value
                ),
                pos: behind.pos,
            });
        }
    }
    Ok(())
}

/// Une couleur pleine : nommée (hors `transparent`), `#abc`, `#aabbcc`, ou `#aabbccff`.
fn opaque(value: &str) -> bool {
    let value = value.trim();
    if value == "transparent" || !crate::styles::is_color(value) {
        return false;
    }
    match value.strip_prefix('#') {
        Some(hex) if hex.len() == 8 => hex[6..].eq_ignore_ascii_case("ff"),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::Pos;

    fn setting(name: &str, value: &str) -> Setting {
        Setting { name: name.into(), value: value.into(), pos: Pos { line: 1, column: 1 } }
    }

    #[test]
    fn the_filters_compose_in_one_declaration_in_a_fixed_order() {
        let base = [setting("blur", "3px"), setting("color", "red"), setting("grayscale", "1"), setting("hue", "30deg")];
        let plain = |s: &Setting| s.value.clone();
        assert_eq!(declaration(&base, None, &plain), "filter:grayscale(1) hue-rotate(30deg) blur(3px);");
        // Un état qui change un réglage garde les autres ; un état qui n'en touche aucun n'écrit rien.
        assert_eq!(declaration(&base, Some(&[setting("grayscale", "0")]), &plain), "filter:grayscale(0) hue-rotate(30deg) blur(3px);");
        assert_eq!(declaration(&base, Some(&[setting("color", "blue")]), &plain), "");
        assert_eq!(declaration(&[setting("color", "red")], None, &plain), "");
        assert_eq!(declaration(&[setting("saturate", "1.8"), setting("brightness", "0.6"), setting("contrast", "1.2")], None, &plain), "filter:saturate(1.8) brightness(0.6) contrast(1.2);");
        assert_eq!(backdrop_declaration("6px"), "-webkit-backdrop-filter:blur(6px);backdrop-filter:blur(6px);");
        assert_eq!(applied(&base), ", une fois « grayscale: 1 » et « hue: 30deg » appliqué");
        assert_eq!(applied(&[setting("blur", "3px")]), "");
    }

    #[test]
    fn the_colors_once_filtered() {
        let white = [255.0, 255.0, 255.0];
        let green = [45.0, 106.0, 79.0];
        // Rien d'écrit : rien ne change.
        assert_eq!(seen(&[], [white, green]), [white, green]);
        // La luminosité assombrit les deux couleurs du même facteur.
        let [w, g] = seen(&[setting("brightness", "0.4")], [white, green]);
        assert_eq!(w.map(f64::round), [102.0, 102.0, 102.0]);
        assert_eq!(g.map(f64::round), [18.0, 42.0, 32.0]);
        // Tout gris : une seule valeur par couleur, et le blanc reste blanc.
        let [w, g] = seen(&[setting("grayscale", "1")], [white, green]);
        assert_eq!(w.map(f64::round), [255.0, 255.0, 255.0]);
        assert!((g[0] - g[1]).abs() < 0.5 && (g[1] - g[2]).abs() < 0.5, "{g:?}");
        // Le contraste écarte du gris moyen ; la teinte tourne les couleurs sans toucher au blanc.
        let [w, _] = seen(&[setting("contrast", "1.5"), setting("hue", "90deg"), setting("saturate", "2")], [white, green]);
        assert_eq!(w.map(f64::round), [255.0, 255.0, 255.0]);
        let [_, g] = seen(&[setting("hue", "180deg")], [white, green]);
        assert!(g[0] > g[1], "les couleurs ont tourné : {g:?}");
    }

    #[test]
    fn a_blur_only_on_an_image_a_shape_a_video_a_drawing() {
        let check = |source: &str| crate::styles::check_styles(&crate::holo::read(source).unwrap());
        assert!(check("Page(children: [ Image.photo(source: \"a.png\", alt: \"\"), Shape.rond(form: circle) ])\n.photo { blur: 4px; }\n.rond { opacity: 0.9; hover: { blur: 2px; } }\nImage { blur: 1px; }\nVideo { blur: 0; }\nDrawing { blur: 8px; }").is_ok());
        let refused = |source: &str| check(source).unwrap_err().message;
        assert!(refused("Page(children: [ P(\"x\") ])\nP { blur: 4px; }").contains("pas sur « P »"));
        assert!(refused("Page(children: [ Column.flou(children: [ P(\"x\") ]) ])\n.flou { blur: 4px; }").contains("pas sur « Column »"));
        assert!(refused("Page(children: [ Button.flou(name: B, text: \"x\") ])\n.flou { opacity: 0.9; hover: { blur: 4px; } }").contains("un texte flou ne se lit pas"));
        // Le flou de derrière, lui, se pose partout : à travers un fond à demi transparent.
        assert!(check("Page(children: [ Column.verre(children: [ P(\"x\") ]), Dialog(name: D, children: [ P(\"y\") ]) ])\n.verre { backdrop-blur: 8px; background: #10102080; }\nDialog { backdrop-blur: 6px; background: navy; color: white; }").is_ok());
        assert!(refused("Page(children: [ Column.verre(children: [ P(\"x\") ]) ])\n.verre { backdrop-blur: 8px; background: #101020; }").contains("rien ne se verrait derrière"));
        assert!(refused("Page(children: [ Column.verre(children: [ P(\"x\") ]) ])\n.verre { backdrop-blur: 8px; hover: { background: navy; } }").contains("fond opaque « navy »"));
        assert!(!opaque("transparent") && !opaque("#10102080") && opaque("#101020ff") && opaque("gold") && !opaque("url(\"a.png\")"));
    }
}
