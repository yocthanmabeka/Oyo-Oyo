//! Les filtres d'image dans les styles (ADR-108) : `grayscale`, `saturate`, `brightness`,
//! `contrast`, `hue` et `blur`, un réglage par effet, que le moteur compose en un seul `filter`
//! CSS, toujours dans le même ordre ; dans un état, un réglage change sans effacer les autres,
//! là où la liste `filter` du CSS s'efface entière. Et `backdrop-blur` sur une fenêtre : la page,
//! derrière elle, devient floue.
//!
//! Un filtre ne touche jamais un texte : il se pose sur une image, une forme ou un dessin, qui
//! n'en portent pas ; ni sur une vidéo, dont les commandes et les sous-titres seraient filtrés
//! avec elle. Au clavier, un bloc filtré qui a le focus se montre sans filtre, pour que son cadre
//! de focus reste net : vu dans Chrome, un flou le brouille et une luminosité basse l'efface presque.

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

/// Le flou de la page derrière une fenêtre.
pub const BACKDROP: &str = "backdrop-blur";

/// Les blocs sur lesquels un filtre se pose : ils ne portent ni texte ni commande.
pub const PICTURES: &[&str] = &["Image", "Shape", "Drawing"];

/// Un réglage de filtre ?
pub fn is_filter(name: &str) -> bool {
    FILTERS.iter().any(|(known, _)| *known == name)
}

/// Le style filtre-t-il son bloc, dans ses réglages ou dans l'un de ses états ?
pub fn filtered(rule: &StyleRule) -> bool {
    rule.settings.iter().chain(rule.states.iter().flat_map(|(_, settings, _)| settings.iter())).any(|s| is_filter(&s.name))
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

/// Les filtres se posent sur une image, une forme ou un dessin, jamais au focus ; le flou de
/// derrière, sur une fenêtre, dans son style lui-même.
pub fn check(rule: &StyleRule, program: &Program) -> Result<(), Error> {
    for (state, settings, _) in &rule.states {
        if let Some(setting) = settings.iter().find(|s| is_filter(&s.name) || s.name == BACKDROP) {
            if setting.name == BACKDROP {
                return Err(Error {
                    message: format!("« backdrop-blur » dans l'état « {state} » de « {} » : il s'écrit dans le style de la fenêtre lui-même, « Dialog {{ backdrop-blur: 6px; }} » (ADR-108)", rule.target),
                    pos: setting.pos,
                });
            }
            if state == "focus" {
                return Err(Error {
                    message: format!(
                        "« {} » dans l'état « focus » de « {} » : au clavier, le moteur montre déjà le bloc sans filtre quand il a le focus, pour que son cadre de focus se voie net (ADR-108)",
                        setting.name, rule.target
                    ),
                    pos: setting.pos,
                });
            }
        }
    }
    let filter = rule.settings.iter().chain(rule.states.iter().flat_map(|(_, settings, _)| settings.iter())).find(|s| is_filter(&s.name));
    if let (Some(filter), Some(what)) = (filter, filter.and_then(|_| outside(rule, program, PICTURES))) {
        let touched = if what == "Video" { "ses commandes et ses sous-titres, qui se verraient mal" } else { "les textes et les boutons qu'il porte, qui se liraient mal" };
        return Err(Error {
            message: format!(
                "« {} » dans « {} » : un filtre se pose sur une image, une forme ou un dessin (« Image.photo {{ grayscale: 1; }} ») ; sur « {what} », il toucherait aussi {touched} ; pour les couleurs d'un texte, « color » et « background » (ADR-108)",
                filter.name, rule.target
            ),
            pos: filter.pos,
        });
    }
    if let Some(behind) = rule.settings.iter().find(|s| s.name == BACKDROP) {
        if let Some(what) = outside(rule, program, &["Dialog"]) {
            return Err(Error {
                message: format!(
                    "« backdrop-blur » dans « {} » : il floute la page derrière une fenêtre ouverte, et se pose sur une fenêtre, « Dialog {{ backdrop-blur: 6px; }} », pas sur « {what} » ; pour un fond qui laisse voir derrière lui, « background: #10102080 » (ADR-108)",
                    rule.target
                ),
                pos: behind.pos,
            });
        }
    }
    Ok(())
}

/// Le premier type de bloc visé par le style qui n'est pas permis (`allowed`) : le type du style
/// lui-même, ou celui d'un bloc qui porte son nom ; un composant n'est jamais permis.
fn outside(rule: &StyleRule, program: &Program, allowed: &[&str]) -> Option<String> {
    match &rule.target {
        Target::Type(t) => (!allowed.contains(&t.as_str())).then(|| t.clone()),
        Target::Name(n) => {
            let mut found = None;
            let _ = crate::rules::for_each_block(&program.root, &mut |block| {
                if found.is_none() && block.styles.iter().any(|s| s == n) && !allowed.contains(&block.name.as_str()) {
                    found = Some(block.name.clone());
                }
                Ok(())
            });
            found
        }
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
    }

    #[test]
    fn a_filter_only_on_an_image_a_shape_a_drawing() {
        let check = |source: &str| crate::styles::check_styles(&crate::holo::read(source).unwrap());
        assert!(check("Page(children: [ Image.photo(source: \"a.png\", alt: \"\"), Shape.rond(form: circle), Shape.cible(name: Cible, form: square) ])\n.photo { blur: 4px; }\n.rond { opacity: 0.9; hover: { blur: 2px; } }\n.cible { grayscale: 1; active: { grayscale: 0; } }\nImage { brightness: 0.8; }\nDrawing { blur: 8px; hue: 90deg; }").is_ok());
        let refused = |source: &str| check(source).unwrap_err().message;
        // Un texte, un bloc qui en contient, un composant, une vidéo : jamais.
        assert!(refused("Page(children: [ P(\"x\") ])\nP { blur: 4px; }").contains("sur « P », il toucherait aussi les textes et les boutons qu'il porte"));
        assert!(refused("Page(children: [ Text.tag(\"x\") ])\n.tag { color: white; background: #2d6a4f; brightness: 0.8; }").contains("pour les couleurs d'un texte, « color » et « background »"));
        assert!(refused("Page(children: [ Column.carte(children: [ Image(source: \"a.png\", alt: \"\") ]) ])\n.carte { grayscale: 1; }").contains("sur « Column »"));
        assert!(refused("Page(children: [ Image.photo(source: \"a.png\", alt: \"\"), Button.photo(name: B, text: \"x\") ])\n.photo { opacity: 0.9; hover: { saturate: 2; } }").contains("« saturate » dans « .photo »"));
        assert!(refused("Page(children: [ Video(source: \"f.mp4\", label: \"x\") ])\nVideo { grayscale: 1; }").contains("ses commandes et ses sous-titres"));
        // Au focus, le moteur montre le bloc sans filtre : un filtre écrit là est refusé.
        assert!(refused("Page(children: [ Shape.cible(name: Cible, form: circle) ])\n.cible { focus: { brightness: 1.4; } }").contains("au clavier, le moteur montre déjà le bloc sans filtre"));
        // Le flou de derrière : sur une fenêtre, dans son style lui-même.
        assert!(check("Page(children: [ Dialog(name: D, children: [ P(\"y\") ]), Dialog.verre(name: E, children: [ P(\"z\") ]) ])\nDialog { backdrop-blur: 6px; background: navy; color: white; }\n.verre { backdrop-blur: 12px; }").is_ok());
        assert!(refused("Page(children: [ Column.verre(children: [ P(\"x\") ]) ])\n.verre { backdrop-blur: 8px; background: #10102080; }").contains("se pose sur une fenêtre, « Dialog { backdrop-blur: 6px; } », pas sur « Column »"));
        assert!(refused("Page(children: [ Dialog(name: D, children: [ P(\"y\") ]) ])\nDialog { phone: { backdrop-blur: 2px; } }").contains("dans l'état « phone »"));
    }
}
