//! Les blocs que le format connaît, et les règles qui portent sur leur nom et sur l'ordre
//! des titres (ADR-016, ADR-020). Rien n'est affiché ici : on vérifie seulement.

use crate::holo::{Block, Error, Program, Value};

/// `Text` est du texte sans rôle ; `P`, `H1`, `H2` et `H3` sont un `Text` avec un rôle (ADR-020).
pub const BLOCKS: &[&str] = &["Page", "Text", "P", "H1", "H2", "H3", "A", "Button", "Image", "List", "Point", "World", "On", "Zoom", "Points", "Relief", "Portals", "State", "Prices", "Row", "Column", "Grid", "If", "Hr", "Quote", "Code", "Every", "Board", "Input", "Checkbox", "When", "Component", "Use", "Data", "Sound", "Shape", "Scenes", "Scene", "Enter", "Loop", "H4", "H5", "H6", "Main", "Nav", "Header", "Footer", "Aside", "Stack", "Video", "Table", "Choice", "After", "Repeat", "Item", "Font", "Slider", "Progress", "Details", "Dialog", "Form", "Module", "Filter", "Days", "Drawing", "Rect", "Circle", "Line", "Path", "Chart", "Shared"];

/// Le titre le plus profond : `H6`, comme en HTML (correction d'ADR-020 du 2026-10-06 ; les
/// longs documents en ont besoin). Le numéro dit toujours la place dans le plan, jamais la taille.
pub const TITLE_MAX: u32 = 6;

/// Vérifie tous les blocs d'un fichier : chacun existe, et les titres ne sautent pas de niveau.
pub fn check_blocks(program: &Program) -> Result<(), Error> {
    walk(&program.root, &mut 0, "")
}

/// Les réglages que chaque bloc accepte. Un réglage inconnu est refusé, jamais avalé en silence
/// (correction du 2026-10-06 : `Page(Title: …)` passait, et le titre était perdu). Les blocs
/// absents de cette liste vérifient leurs réglages eux-mêmes (`State`, `Shared`, `Prices`, `Data`,
/// `Zoom`, `Points`, `Relief`, `Portals`, `Enter`, `Loop`, `Use`).
const BLOCK_SETTINGS: &[(&str, &[&str])] = &[
    ("Page", &["name", "title", "children", "pixels", "rules", "state", "shared", "prices", "keep", "data", "zoom", "points", "relief", "portals", "lang", "description", "image", "fonts", "icon", "modules", "components", "computed"]),
    ("World", &["name", "children", "pixels", "rules"]),
    ("Component", &["name", "params", "emits", "children", "rules"]),
    ("Text", &["name"]),
    ("P", &["name"]),
    ("H1", &["name"]),
    ("H2", &["name"]),
    ("H3", &["name"]),
    ("H4", &["name"]),
    ("H5", &["name"]),
    ("H6", &["name"]),
    ("A", &["name", "to", "newTab", "download"]),
    ("Button", &["name", "text"]),
    ("Image", &["name", "source", "weight", "alt", "phone", "caption"]),
    ("Sound", &["name", "source", "weight", "label", "volume", "loop"]),
    ("Filter", crate::computed::PARAMS),
    ("Days", crate::computed::DAYS_PARAMS),
    ("Shape", &["name", "form", "color", "size"]),
    ("Drawing", &["name", "label", "width", "height", "children", "shapes"]),
    ("Chart", crate::chart::PARAMS),
    ("Rect", crate::drawing::RECT),
    ("Circle", crate::drawing::CIRCLE),
    ("Line", crate::drawing::LINE),
    ("Path", crate::drawing::PATH),
    ("List", &["name", "children", "ordered"]),
    ("Hr", &["name"]),
    ("Quote", &["name", "by"]),
    ("Code", &["name"]),
    ("Header", &["name", "children"]),
    ("Nav", &["name", "children"]),
    ("Main", &["name", "children"]),
    ("Footer", &["name", "children"]),
    ("Aside", &["name", "children"]),
    ("Row", &["name", "children", "gap", "align"]),
    ("Column", &["name", "children", "gap", "align"]),
    ("Grid", &["name", "children", "gap", "columns"]),
    ("Stack", &["name", "children"]),
    ("Board", &["name", "children", "height"]),
    ("Point", &["name", "seed", "brightness", "fragments", "color", "palette", "budget", "inside", "above"]),
    ("Input", &["name", "value", "label", "min", "max", "lines", "type", "accept", "required"]),
    ("Slider", &["name", "value", "label", "min", "max"]),
    ("Progress", &["name", "value", "max", "label"]),
    ("Details", &["name", "summary", "children", "open"]),
    ("Dialog", &["name", "children"]),
    ("Form", &["name", "children"]),
    ("Choice", &["name", "value", "label", "options", "menu", "required"]),
    ("Video", &["name", "source", "label", "weight", "captions"]),
    ("Table", &["name", "caption", "head", "rows"]),
    ("Checkbox", &["name", "value", "label", "required"]),
    ("If", &["name", "is", "not", "over", "under", "children", "rules", "else"]),
    ("On", &["effect"]),
    ("Every", &["effect"]),
    ("After", &["effect"]),
    ("When", &["is", "not", "over", "under", "meets", "within", "effect"]),
    ("Scenes", &["name", "children", "height", "repeat"]),
    ("Scene", &["name", "children", "for"]),
    ("Font", &["family", "source"]),
    ("Module", &["name", "source", "input", "output", "time", "memory"]),
    // Les données de la page (ADR-030) ; leur nom, que les règles écoutent (ADR-064).
    ("Data", &["name", "from", "every"]),
];

/// Les réglages de chaque bloc, pour l'éditeur (ADR-046) : il propose ceux du bloc où l'on écrit.
pub fn block_params() -> &'static [(&'static str, &'static [&'static str])] {
    BLOCK_SETTINGS
}

/// Les blocs qui ne se voient pas : ils ne bougent pas (`enter`, `loop`).
const NO_MOVEMENT: &[&str] = &["Page", "World", "Component", "On", "Every", "When", "After", "Sound", "Scene"];

/// Vérifie les réglages d'un bloc, selon le bloc qui le contient (`parent`).
fn check_settings(block: &Block, parent: &str) -> Result<(), Error> {
    let Some((_, allowed)) = BLOCK_SETTINGS.iter().find(|(name, _)| *name == block.name) else { return Ok(()) };
    for argument in &block.arguments {
        let Some(name) = argument.name.as_deref() else { continue };
        let movement = (name == "enter" || name == "loop") && !NO_MOVEMENT.contains(&block.name.as_str());
        let on_board = matches!(name, "x" | "y" | "drag") && parent == "Board";
        let in_stack = name == "align" && parent == "Stack";
        let in_row = name == "grow" && (parent == "Row" || parent == "Column");
        if allowed.contains(&name) || movement || on_board || in_stack || in_row {
            // Un nom de bloc commence par une majuscule, comme un bloc : ce qu'on touche a une
            // majuscule, ce qui change (une valeur) n'en a pas. `Filter(name: found)` nomme une
            // liste, donc une valeur : en minuscules (lot 2 du web).
            if name == "name" && block.name != "Filter" && block.name != "Days" {
                if let crate::holo::Value::Name(given) = &argument.value {
                    if given.starts_with(|c: char| c.is_ascii_lowercase()) || given.contains('_') {
                        let flutter = crate::state::in_flutter(given);
                        let uppercase: String = flutter.chars().take(1).map(|c| c.to_ascii_uppercase()).chain(flutter.chars().skip(1)).collect();
                        return Err(Error { message: format!("« name: {given} » : un nom de bloc s'écrit comme en Flutter, une majuscule au début et à chaque mot ; écris « name: {uppercase} » (ADR-037)"), pos: argument.pos });
                    }
                }
            }
            continue;
        }
        let message = if name == "grow" {
            format!("« grow: » fait grandir un bloc rangé dans Row ou Column : mets « {} » dans Row(children: [ … ])", block.name)
        } else if matches!(name, "x" | "y" | "drag") {
            format!("« {name}: » place un bloc sur un plateau : mets « {} » dans Board(children: [ … ])", block.name)
        } else if name == "align" && block.name != "Row" && block.name != "Column" {
            format!("« align: » place un bloc posé sur un autre : mets « {} » dans Stack(children: [ … ])", block.name)
        } else if let Some(good) = allowed.iter().find(|known| known.eq_ignore_ascii_case(name)) {
            format!("« {name} » : un paramètre s'écrit en minuscules, écris « {good} »")
        } else if name == "styles" || name == "style" {
            "un style s'écrit comme en CSS, après le bloc racine : « P { color: gray; } » (ADR-017)".to_string()
        } else if block.name == "World" && matches!(name, "state" | "shared" | "prices" | "keep" | "data" | "zoom" | "points" | "relief" | "portals" | "title") {
            format!("« {name}: » se règle sur la page, pas dans un monde : un monde partage les valeurs et la vue de sa page")
        } else {
            format!("« {} » n'a pas de paramètre « {name} » ; paramètres possibles : {}", block.name, allowed.join(", "))
        };
        return Err(Error { message, pos: argument.pos });
    }
    Ok(())
}

fn walk(block: &Block, last_title: &mut u32, parent: &str) -> Result<(), Error> {
    if crate::state::is_requested(block) {
        // `p.card(...)` : un bloc écrit en minuscules, plutôt qu'une demande.
        let (before, after) = block.name.split_once('.').unwrap_or((&block.name, ""));
        let uppercase: String = before.chars().take(1).map(|c| c.to_ascii_uppercase()).chain(before.chars().skip(1)).collect();
        let message = if BLOCKS.contains(&uppercase.as_str()) {
            format!("« {} » : un nom de bloc commence par une majuscule, écris « {uppercase}.{after} »", block.name)
        } else {
            format!("« {}(...) » est une demande : elle s'écrit dans l'effet d'une règle, On(Add.tap, effect: {}(1))", block.name, block.name)
        };
        return Err(Error { message, pos: block.pos });
    }
    if block.name == crate::holo::NAMED_VALUE {
        let name = block.arguments.first().and_then(|a| a.name.clone()).unwrap_or_default();
        return Err(Error { message: format!("« {name}: … » dans une liste : seuls les paramètres d'un composant ont une valeur par défaut, params: [title, {name}: …] ; ailleurs, une liste contient des valeurs sans nom"), pos: block.pos });
    }
    if block.name == "Part" {
        return Err(Error { message: crate::components::OLD_PART.into(), pos: block.pos });
    }
    if !BLOCKS.contains(&block.name.as_str()) {
        return Err(Error { message: unknown_block(&block.name), pos: block.pos });
    }
    // Une répétition est dépliée à la lecture (ADR-040) : un « Item » qui reste est mal placé.
    // Depuis ADR-051, un « Item » est aussi un élément d'une liste de la page, dans State ou
    // dans une demande « push ».
    if block.name == "Item" && parent != "State" && !parent.ends_with(".push") {
        return Err(Error { message: "« Item » est un élément d'une répétition : Repeat(items: [ Item(…) ], children: [ … ])".into(), pos: block.pos });
    }
    check_settings(block, parent)?;
    if let Some(level) = heading_level(&block.name) {
        // Le numéro dit la place dans le plan, jamais la taille : `H3` ne suit pas `H1`.
        if level > *last_title + 1 {
            let message = if *last_title == 0 {
                format!("« {} » : le premier titre est « H1 » ; le numéro dit la place dans le plan, la taille se règle par le style (ADR-020)", block.name)
            } else {
                format!(
                    "« {} » arrive après « H{} » : un titre ne saute pas de niveau, écris « H{} » ; la taille se règle par le style (ADR-020)",
                    block.name,
                    *last_title,
                    *last_title + 1
                )
            };
            return Err(Error { message, pos: block.pos });
        }
        *last_title = level;
    }
    // Une page et un monde ont chacun leur propre plan.
    let mut clean_plane = 0;
    let plane = if block.name == "Page" || block.name == "World" { &mut clean_plane } else { last_title };
    for argument in &block.arguments {
        // L'effet d'une règle peut être une demande, `cart.add(1)` : `regles.rs` la vérifie.
        // (une seule, ou plusieurs entre crochets : dans les deux cas, on ne descend pas dedans)
        let request = matches!(block.name.as_str(), "On" | "Every" | "When" | "After") && argument.name.as_deref() == Some("effect");
        if !request {
            visit(&argument.value, plane, &block.name)?;
        }
    }
    Ok(())
}

fn visit(value: &Value, last_title: &mut u32, parent: &str) -> Result<(), Error> {
    match value {
        Value::Block(block) => walk(block, last_title, parent),
        Value::List(elements) => elements.iter().try_for_each(|e| visit(e, last_title, parent)),
        _ => Ok(()),
    }
}

/// `H1` → 1. Rend `None` pour tout ce qui n'est pas un titre connu.
fn heading_level(name: &str) -> Option<u32> {
    let level: u32 = name.strip_prefix('H')?.parse().ok()?;
    (1..=TITLE_MAX).contains(&level).then_some(level)
}

/// Le message pour un bloc qui n'existe pas, avec le mot à écrire quand on le devine.
pub fn unknown_block(name: &str) -> String {
    let digits = name.strip_prefix('H').unwrap_or("");
    if !digits.is_empty() && digits.bytes().all(|c| c.is_ascii_digit()) {
        return format!("bloc inconnu « {name} » : les titres vont de « H1 » à « H{TITLE_MAX} » (ADR-020)");
    }
    if name == "Style" || name == "Theme" {
        return format!("« {name} » n'est pas un bloc : un style s'écrit comme en CSS, après le bloc racine, « .card {{ color: gray; }} » ; le thème est le style de « Page » ou de « World » (ADR-017)");
    }
    match old_word(name) {
        Some(new_one) => format!("bloc inconnu « {name} » : le vocabulaire est en anglais, écris « {new_one} » (ADR-016)"),
        None => format!("bloc inconnu « {name} »"),
    }
}

/// Les mots français d'avant ADR-016, pour guider vers le mot anglais plutôt que de dire
/// seulement « inconnu ».
pub fn old_word(word: &str) -> Option<&'static str> {
    Some(match word {
        "nom" => "name",
        "graine" => "seed",
        "lumiere" => "brightness",
        "morceler" => "fragments",
        "contenu" => "children",
        "interieur" => "inside",
        "phenomenes" => "rules",
        "titre" => "title",
        "texte" => "text",
        "couleur" => "color",
        "Texte" => "Text",
        "Bouton" => "Button",
        "Monde" => "World",
        "Liste" => "List",
        "Quand" => "On",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::read;

    fn check(src: &str) -> Result<(), Error> {
        check_blocks(&read(src)?)
    }

    #[test]
    fn accepts_valid_cases_of_the_suite() {
        for sample in [
            include_str!("../../experiments/conformite-v0.1/cas/valides/01-page-simple.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/02-big-bang.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/03-entrer-dans-un-point.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/04-budget-respecte.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/05-texte-sans-role.holo"),
        ] {
            check(sample).unwrap();
        }
    }

    #[test]
    fn refuses_unknown_blocks_at_the_right_line() {
        let div = check(include_str!("../../experiments/conformite-v0.1/cas/refuses/E05-bloc-inconnu.holo")).unwrap_err();
        assert_eq!(div.pos.line, 4);
        assert!(div.message.contains("bloc inconnu « Div »"));
        let h4 = check(include_str!("../../experiments/conformite-v0.1/cas/refuses/E11-titre-trop-profond.holo")).unwrap_err();
        assert_eq!(h4.pos.line, 7);
        assert!(h4.message.contains("de « H1 » à « H6 »"));
        assert!(check("Page(children: [ Texte(\"Bonjour\") ])").unwrap_err().message.contains("écris « Text »"));
        assert!(check("Page(styles: [ Style(color: gray) ])").unwrap_err().message.contains("comme en CSS"));
    }

    #[test]
    fn a_lowercase_block_is_refused_with_the_word_to_write() {
        let e = check(include_str!("../../experiments/conformite-v0.1/cas/refuses/E09-bloc-en-minuscules.holo")).unwrap_err();
        assert_eq!(e.pos.line, 5);
        assert!(e.message.contains("écris « H1 »"));
        assert!(check("page(title: \"x\")").unwrap_err().message.contains("écris « Page »"));
        // Un réglage reste en minuscules : seul ce qui ouvre une parenthèse est un bloc.
        check("Page(children: [ Button(name: Open, text: \"Enter\") ])").unwrap();
    }

    #[test]
    fn headings_do_not_skip_a_level() {
        let jump = check(include_str!("../../experiments/conformite-v0.1/cas/refuses/E10-titre-saute-un-niveau.holo")).unwrap_err();
        assert_eq!(jump.pos.line, 7);
        assert!(jump.message.contains("écris « H2 »"));
        assert!(check("Page(children: [ H2(\"x\") ])").unwrap_err().message.contains("le premier titre est « H1 »"));
        // Remonter d'un ou de plusieurs niveaux est permis ; descendre se fait un par un.
        check("Page(children: [ H1(\"a\"), H2(\"b\"), H3(\"c\"), H1(\"d\"), H2(\"e\") ])").unwrap();
        // Chaque page et chaque monde a son propre plan.
        check("Page(children: [ H1(\"a\"), H2(\"b\"), Point(name: A, seed: 1, inside: World(children: [ H1(\"c\") ])), H3(\"d\") ])").unwrap();
        assert!(check("Page(children: [ H1(\"a\"), Point(name: A, seed: 1, inside: World(children: [ H2(\"c\") ])) ])").is_err());
    }

    #[test]
    fn nothing_is_swallowed_silently() {
        for (source, message) in [
            ("Page(Title: \"a\", children: [ H1(\"a\") ])", "écris « title »"),
            ("Page(colour: \"a\", children: [])", "n'a pas de paramètre « colour »"),
            ("Page(children: [ Button(name: buy, text: \"x\") ])", "écris « name: Buy »"),
            ("Page(children: [ P(\"a\", x: 10, y: 10) ])", "mets « P » dans Board"),
            ("Page(children: [ P(\"a\", align: top) ])", "mets « P » dans Stack"),
            ("Page(children: [ Image(source: \"a.png\", Alt: \"x\") ])", "écris « alt »"),
            ("Page(children: [ H1(\"a\", size: 3) ])", "« H1 » n'a pas de paramètre « size »"),
        ] {
            let error = check(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
        // Ce qui est permis selon la place : sur un plateau, dans une pile, en mouvement.
        check("Page(children: [ Board(children: [ Shape(name: S, form: circle, x: 1, y: 2, drag: true) ]), Stack(children: [ P(\"a\"), P(\"b\", align: top) ]), H1(\"c\", enter: Enter(y: 4px)) ])").unwrap();
    }

}
