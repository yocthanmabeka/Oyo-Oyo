//! Vérification des styles (ADR-017). Un style s'écrit comme en CSS, `.card { color: gray; }`,
//! mais rien n'est toléré en silence : un réglage inconnu, une valeur mal écrite, un style
//! défini deux fois ou jamais défini sont refusés avec leur ligne.

use crate::blocks::{unknown_block, BLOCKS};
use crate::holo::{Block, Target, Error, Program, Setting, Value};

/// Ce qu'un réglage accepte comme valeur.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Shape {
    Color,
    /// Une taille : `0`, `16px` ou `50%`.
    Size,
    /// D'une à quatre tailles, comme `padding: 8px 16px`.
    Sizes,
    /// Un mot parmi une liste.
    Word(&'static [&'static str]),
    /// Un nombre entre 0 et 1.
    Fraction,
    /// `1px solid gray`.
    Border,
    /// Un ou plusieurs noms de police, séparés par des virgules.
    Font,
    /// Le fond : une couleur, un dégradé (`linear-gradient(…)`, `radial-gradient(…)`), ou une
    /// image rangée à côté du fichier (`url("fond.jpg")`), qui couvre toujours le bloc (ADR-041).
    Background,
    /// Un nombre sans unité, entre deux bornes : `line-height: 1.5`, `scale: 1.1`.
    Number(f64, f64),
    /// Un écart en pixels, qui peut être négatif : `letter-spacing: -0.5px`.
    Gap,
    /// Une à trois ombres, séparées par des virgules : `0 4px 12px #00000066` ; ou `none`.
    Shadow,
    /// Un angle, de -360deg à 360deg : `rotate: -3deg`.
    Angle,
    /// La durée d'un passage d'une allure à l'autre : `0.3s`, `200ms`, ou `none`.
    Duration,
    /// Des proportions : `16/9`, `4 / 3`, ou un seul nombre, `1` (ADR-069).
    Ratio,
    /// Un nombre entier entre deux bornes : `line-clamp: 3`.
    Count(u32, u32),
    /// La forme du curseur : un mot, ou une image rangée à côté, `url("viseur.png")` (ADR-069).
    Cursor,
}

/// Les réglages connus : l'apparence, avec les noms du CSS de base.
const SETTINGS: &[(&str, Shape)] = &[
    ("color", Shape::Color),
    ("background", Shape::Background),
    ("font-size", Shape::Size),
    ("font-weight", Shape::Word(&["normal", "bold"])),
    ("font-style", Shape::Word(&["normal", "italic"])),
    ("font-family", Shape::Font),
    // `justify` coupe aussi les mots en fin de ligne, dans la langue de la page (ADR-069).
    ("text-align", Shape::Word(&["left", "center", "right", "justify"])),
    ("border", Shape::Border),
    ("border-radius", Shape::Size),
    ("padding", Shape::Sizes),
    ("margin", Shape::Sizes),
    ("width", Shape::Size),
    ("height", Shape::Size),
    ("max-width", Shape::Size),
    ("opacity", Shape::Fraction),
    // Le lot 4 (ADR-041) : le texte, les ombres, la pose, le passage d'une allure à l'autre.
    ("line-height", Shape::Number(0.8, 3.0)),
    ("letter-spacing", Shape::Gap),
    ("text-transform", Shape::Word(&["none", "uppercase", "lowercase", "capitalize"])),
    ("text-decoration", Shape::Word(&["none", "underline", "line-through"])),
    ("box-shadow", Shape::Shadow),
    ("text-shadow", Shape::Shadow),
    ("rotate", Shape::Angle),
    ("scale", Shape::Number(0.1, 5.0)),
    ("transition", Shape::Duration),
    // Le lot 4 du web (ADR-069) : les tailles au plus et au moins, les proportions, ce qui
    // dépasse, le curseur.
    ("min-width", Shape::Size),
    ("min-height", Shape::Size),
    ("max-height", Shape::Size),
    ("aspect-ratio", Shape::Ratio),
    ("object-fit", Shape::Word(&["cover", "contain", "fill", "none", "scale-down"])),
    ("object-position", Shape::Word(&["center", "top", "bottom", "left", "right", "top left", "top right", "bottom left", "bottom right"])),
    ("overflow", Shape::Word(OVERFLOW)),
    ("overflow-x", Shape::Word(OVERFLOW)),
    ("overflow-y", Shape::Word(OVERFLOW)),
    ("white-space", Shape::Word(&["normal", "nowrap", "pre-line", "pre-wrap"])),
    ("line-clamp", Shape::Count(1, 20)),
    ("cursor", Shape::Cursor),
];

const OVERFLOW: &[&str] = &["visible", "hidden", "auto", "scroll"];

/// Les formes de curseur du web, sauf celles des bords qu'on étire (`ew-resize`…), qui
/// n'ont rien à étirer sans disposition à la main (ADR-017).
const CURSORS: &[&str] = &[
    "auto", "default", "pointer", "text", "move", "grab", "grabbing", "not-allowed", "help", "wait", "progress", "crosshair", "zoom-in", "zoom-out", "none",
    "copy", "alias", "no-drop", "cell", "context-menu", "vertical-text", "all-scroll",
];

/// Les noms des réglages d'un style, pour l'éditeur (ADR-046).
pub fn setting_names() -> Vec<&'static str> {
    SETTINGS.iter().map(|(name, _)| *name).collect()
}

/// Les variables (ADR-041) : `--or: #E9B44C;` dans le style de `Page`, puis `color: --or;`
/// partout. Depuis ADR-050, un composant ou un nom de style peut aussi en définir ou en redéfinir
/// une (`.promo { --accent: crimson; }`) : elle vaut pour le bloc et ce qu'il contient. Rend
/// chaque variable et sa première valeur, celle du thème d'abord.
pub fn variables(program: &Program) -> Vec<(String, String)> {
    let mut variables: Vec<(String, String)> = Vec::new();
    let mut rules: Vec<_> = program.styles.iter().collect();
    rules.sort_by_key(|r| !matches!(&r.target, Target::Type(t) if t == "Page" || t == "World"));
    for rule in rules {
        {
            for setting in rule.settings.iter().chain(rule.states.iter().flat_map(|(_, r, _)| r.iter())) {
                if setting.name.starts_with("--") && !variables.iter().any(|(n, _)| *n == setting.name) {
                    variables.push((setting.name.clone(), setting.value.clone()));
                }
            }
        }
    }
    variables
}

/// Les variables d'une valeur, `--or` dans `0 4px 8px --ombre`.
pub fn variables_of(value: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut remainder = value;
    while let Some(start) = remainder.find("--") {
        let before_ok = start == 0 || remainder[..start].ends_with([' ', ',', '(']);
        let end = remainder[start + 2..].find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')).map_or(remainder.len(), |f| start + 2 + f);
        if before_ok && end > start + 2 {
            names.push(&remainder[start..end]);
        }
        remainder = &remainder[end.max(start + 2)..];
    }
    names
}

/// La disposition ne se règle pas dans un style : elle vient des blocs (ADR-017, règle 3).
const LAYOUT: &[&str] = &[
    "display", "position", "float", "clear", "top", "left", "right", "bottom", "z-index", "flex", "flex-direction", "flex-wrap",
    "justify-content", "align-items", "align-self", "gap", "grid", "grid-template-columns", "grid-template-rows", "order",
];

const COLORS: &[&str] = &[
    "transparent", "black", "white", "gray", "silver", "red", "maroon", "orange", "gold", "yellow", "olive", "green", "lime",
    "teal", "aqua", "cyan", "blue", "navy", "purple", "magenta", "fuchsia", "pink", "brown", "beige", "ivory", "indigo", "violet",
    "turquoise", "salmon", "coral", "crimson", "khaki", "lavender", "tan",
];

/// La couleur d'une valeur, en rouge, vert, bleu (0 à 255) : une couleur nommée, `#abc`,
/// `#aabbcc`, ou une variable qui en porte une. `None` pour ce qui n'est pas une couleur pleine
/// (un dégradé, une image, `transparent`, une couleur à demi transparente).
fn rgb(value: &str, variables: &[(String, String)]) -> Option<[f64; 3]> {
    let value = value.trim();
    if value.starts_with("--") {
        let (_, v) = variables.iter().find(|(n, _)| n == value)?;
        return if v.starts_with("--") { None } else { rgb(v, variables) };
    }
    if let Some(hex) = value.strip_prefix('#') {
        let byte = |a: &str| u8::from_str_radix(a, 16).ok().map(f64::from);
        return match hex.len() {
            3 => Some([byte(&hex[0..1].repeat(2))?, byte(&hex[1..2].repeat(2))?, byte(&hex[2..3].repeat(2))?]),
            6 => Some([byte(&hex[0..2])?, byte(&hex[2..4])?, byte(&hex[4..6])?]),
            8 if hex[6..8].eq_ignore_ascii_case("ff") => Some([byte(&hex[0..2])?, byte(&hex[2..4])?, byte(&hex[4..6])?]),
            _ => None,
        };
    }
    const NAMED: &[(&str, u32)] = &[
        ("black", 0x000000), ("white", 0xffffff), ("gray", 0x808080), ("silver", 0xc0c0c0), ("red", 0xff0000), ("maroon", 0x800000),
        ("orange", 0xffa500), ("gold", 0xffd700), ("yellow", 0xffff00), ("olive", 0x808000), ("green", 0x008000), ("lime", 0x00ff00),
        ("teal", 0x008080), ("aqua", 0x00ffff), ("cyan", 0x00ffff), ("blue", 0x0000ff), ("navy", 0x000080), ("purple", 0x800080),
        ("magenta", 0xff00ff), ("fuchsia", 0xff00ff), ("pink", 0xffc0cb), ("brown", 0xa52a2a), ("beige", 0xf5f5dc), ("ivory", 0xfffff0),
        ("indigo", 0x4b0082), ("violet", 0xee82ee), ("turquoise", 0x40e0d0), ("salmon", 0xfa8072), ("coral", 0xff7f50),
        ("crimson", 0xdc143c), ("khaki", 0xf0e68c), ("lavender", 0xe6e6fa), ("tan", 0xd2b48c),
    ];
    let (_, code) = NAMED.iter().find(|(n, _)| *n == value)?;
    Some([f64::from((code >> 16) & 0xff), f64::from((code >> 8) & 0xff), f64::from(code & 0xff)])
}

/// Le contraste de deux couleurs, de 1 à 21, comme le calcule le WCAG.
fn contrast(a: [f64; 3], b: [f64; 3]) -> f64 {
    let luminance = |c: [f64; 3]| {
        let l = |v: f64| {
            let v = v / 255.0;
            if v <= 0.040_45 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * l(c[0]) + 0.7152 * l(c[1]) + 0.0722 * l(c[2])
    };
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Un style qui donne à la fois la couleur du texte et celle du fond doit pouvoir être lu par
/// tous : 4,5 pour 1 au moins, 3 pour 1 pour un grand texte (24px, ou 19px en gras), comme le
/// demande le WCAG (ADR-055). Vérifié pour le style, et pour chacun de ses états.
fn check_contrast(rule: &crate::holo::StyleRule, variables: &[(String, String)]) -> Result<(), Error> {
    let value = |settings: &[Setting], name: &str| settings.iter().find(|r| r.name == name).map(|r| r.value.clone());
    let mut sample: Vec<(Option<&str>, Vec<Setting>, crate::holo::Pos)> = vec![(None, rule.settings.clone(), rule.pos)];
    for (state, settings, pos) in &rule.states {
        let mut mix = rule.settings.clone();
        mix.retain(|r| !settings.iter().any(|e| e.name == r.name));
        mix.extend(settings.iter().cloned());
        sample.push((Some(state.as_str()), mix, *pos));
    }
    for (state, settings, pos) in sample {
        let (Some(text), Some(background)) = (value(&settings, "color"), value(&settings, "background")) else { continue };
        // Une variable redéfinie dans ce style ou cet état (`dark: { --ink: #F5F5F5; }`) y vaut d'abord.
        let mut local_rules: Vec<(String, String)> = settings.iter().filter(|r| r.name.starts_with("--")).map(|r| (r.name.clone(), r.value.clone())).collect();
        local_rules.extend(variables.iter().cloned());
        let (Some(t), Some(f)) = (rgb(&text, &local_rules), rgb(&background, &local_rules)) else { continue };
        let size = value(&settings, "font-size").and_then(|v| v.strip_suffix("px").and_then(|n| n.trim().parse::<f64>().ok())).unwrap_or(16.0);
        let bold = value(&settings, "font-weight").is_some_and(|v| v == "bold");
        let big = size >= 24.0 || (bold && size >= 19.0);
        let threshold = if big { 3.0 } else { 4.5 };
        let seen = contrast(t, f);
        if seen + 1e-9 < threshold {
            let or_ = state.map_or(String::new(), |e| format!(", dans l'état « {e} »"));
            let written = |x: f64| format!("{:.1}", (x * 10.0).floor() / 10.0).replace('.', ",");
            return Err(Error {
                message: format!(
                    "« {} »{or_} : le texte « {text} » sur le fond « {background} » a un contraste de {} pour 1 ; il faut {} pour 1 au moins pour qu'il soit lu par tous (WCAG) : fonce le fond ou éclaircis le texte, ou l'inverse",
                    rule.target,
                    written(seen),
                    if big { "3" } else { "4,5" }
                ),
                pos,
            });
        }
    }
    Ok(())
}

/// Vérifie les règles de style d'un fichier et les noms de style posés sur les blocs.
pub fn check_styles(program: &Program) -> Result<(), Error> {
    let variables = variables(program);
    for (i, rule) in program.styles.iter().enumerate() {
        if let Target::Type(name) = &rule.target {
            if !BLOCKS.contains(&name.as_str()) && !program.components.contains(name) {
                let uppercase = uppercase(name);
                let message = if BLOCKS.contains(&uppercase.as_str()) {
                    format!("« {name} » : un type de bloc commence par une majuscule, écris « {uppercase} {{ … }} » (ADR-020)")
                } else {
                    unknown_block(name)
                };
                return Err(Error { message, pos: rule.pos });
            }
        }
        if program.styles[..i].iter().any(|other| other.target == rule.target) {
            return Err(Error {
                message: format!("le style « {} » est défini deux fois : rassemble ses réglages au même endroit", rule.target),
                pos: rule.pos,
            });
        }
        for (j, setting) in rule.settings.iter().enumerate() {
            if rule.settings[..j].iter().any(|other| other.name == setting.name) {
                return Err(Error { message: format!("le réglage « {} » est donné deux fois dans « {} »", setting.name, rule.target), pos: setting.pos });
            }
            check_setting(setting, None, &variables)?;
        }
        check_contrast(rule, &variables)?;
        check_parity(rule, program)?;
        // Les états (hover, focus, active, dark, phone) : chacun une fois, avec des réglages connus.
        for (k, (state, settings, pos)) in rule.states.iter().enumerate() {
            if rule.states[..k].iter().any(|(other, ..)| other == state) {
                return Err(Error { message: format!("l'état « {state} » est donné deux fois dans « {} »", rule.target), pos: *pos });
            }
            if settings.is_empty() {
                return Err(Error { message: format!("l'état « {state} » de « {} » est vide : écris ce qui change, comme « {state}: {{ background: navy; }} »", rule.target), pos: *pos });
            }
            for (j, setting) in settings.iter().enumerate() {
                if settings[..j].iter().any(|other| other.name == setting.name) {
                    return Err(Error { message: format!("le réglage « {} » est donné deux fois dans l'état « {state} »", setting.name), pos: setting.pos });
                }
                check_setting(setting, Some(state), &variables)?;
            }
        }
    }
    placed_names(&program.root, program)
}

/// Les blocs qui agissent : on les touche, on y écrit, ils mènent ailleurs ou se jouent.
const ACTING: &[&str] = &["Button", "A", "Input", "Checkbox", "Choice", "Slider", "Form", "Point", "Details", "Video", "Board"];

/// La parité (règle de Yocthan du 2026-10-07, ADR-069) : ce qui existe sur un appareil existe sur
/// l'autre. `display: none` dans `phone:`, `computer:` ou `narrow:` ne cache donc que ce qui se
/// lit (une phrase, une image) ; jamais ce qui agit : un bloc d'`ACTING`, ou un bloc qu'une règle
/// écoute (`On(Card.tap, …)`), ni ce qui en contient un.
fn check_parity(rule: &crate::holo::StyleRule, program: &Program) -> Result<(), Error> {
    let mut listened: Vec<&str> = Vec::new();
    let _ = crate::rules::for_each_block(&program.root, &mut |block| {
        if block.name == "On" {
            if let Some(Value::Name(signal)) = block.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value) {
                if let Some((name, _)) = signal.split_once('.').filter(|(name, _)| *name != "Key") {
                    listened.push(name);
                }
            }
        }
        Ok(())
    });
    let acting = |block: &Block| ACTING.contains(&block.name.as_str()) || crate::rules::name_of(block).is_some_and(|n| listened.contains(&n));
    for (state, settings, pos) in &rule.states {
        if !["phone", "computer", "narrow"].contains(&state.as_str()) || !settings.iter().any(|s| s.name == "display" && s.value == "none") {
            continue;
        }
        let mut found: Option<String> = None;
        let _ = crate::rules::for_each_block(&program.root, &mut |block| {
            let carries = match &rule.target {
                Target::Type(t) => &block.name == t,
                Target::Name(n) => block.styles.iter().any(|s| s == n),
            };
            if carries && found.is_none() {
                let _ = crate::rules::for_each_block(block, &mut |inner| {
                    if found.is_none() && acting(inner) {
                        found = Some(inner.name.clone());
                    }
                    Ok(())
                });
            }
            Ok(())
        });
        if let Some(what) = found {
            let place = match state.as_str() {
                "phone" => "sur un téléphone",
                "computer" => "sur un ordinateur",
                _ => "dans une case étroite",
            };
            return Err(Error {
                message: format!(
                    "« {} » : « display: none » dans « {state}: » cacherait « {what} » {place} seulement ; ce qui agit (un bouton, un lien, un champ, un formulaire, un bloc qu'une règle écoute) existe sur tous les appareils (règle de parité, ADR-069) : cache plutôt ce qui ne fait que se lire, ou change son allure",
                    rule.target
                ),
                pos: *pos,
            });
        }
    }
    Ok(())
}

/// Chaque nom de style posé sur un bloc (`P.card(...)`) doit être défini.
fn placed_names(block: &Block, program: &Program) -> Result<(), Error> {
    for name in block.styles.iter().filter(|s| s.starts_with(|c: char| c.is_ascii_lowercase())) {
        if !program.styles.iter().any(|r| r.target == Target::Name(name.clone())) {
            return Err(Error {
                message: format!("le style « .{name} » n'est défini nulle part : écris « .{name} {{ … }} » après le bloc racine"),
                pos: block.pos,
            });
        }
    }
    fn visit(value: &Value, program: &Program) -> Result<(), Error> {
        match value {
            Value::Block(block) => placed_names(block, program),
            Value::List(elements) => elements.iter().try_for_each(|e| visit(e, program)),
            _ => Ok(()),
        }
    }
    block.arguments.iter().try_for_each(|a| visit(&a.value, program))
}

fn check_setting(setting: &Setting, state: Option<&str>, variables: &[(String, String)]) -> Result<(), Error> {
    let refusal = |message: String| Err(Error { message, pos: setting.pos });
    let name = setting.name.as_str();
    // Une variable : une couleur ou une taille. Définie dans le thème (le style de Page), elle
    // vaut partout ; dans un composant ou un nom de style, pour ce bloc et son contenu (ADR-050).
    if let Some(remainder) = name.strip_prefix("--") {
        if remainder.is_empty() || !remainder.starts_with(|c: char| c.is_ascii_lowercase()) || !remainder.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return refusal(format!("« {name} » : une variable s'écrit comme en CSS, en minuscules, les mots joints par « - », comme « --or-clair »"));
        }
        let value = setting.value.as_str();
        if !(is_color(value) || is_size(value)) {
            return refusal(format!("« {name}: {value} » : une variable porte une couleur ou une taille, comme « #E9B44C » ou « 16px »"));
        }
        return Ok(());
    }
    // Cacher un bloc sur un téléphone, sur un ordinateur, ou quand la place manque : seulement
    // dans « phone: { … } », « computer: { … } » ou « narrow: { … } » (ADR-041, ADR-069).
    if name == "display" {
        return match (state, setting.value.as_str()) {
            (Some("phone" | "computer" | "narrow" | "print"), "none") => Ok(()),
            (Some(screen @ ("phone" | "computer" | "narrow" | "print")), _) => refusal(format!("dans « {screen}: {{ … }} », « display » ne prend que « none » : cacher le bloc")),
            _ => refusal("« display » règle la disposition, pas l'apparence : la disposition vient des blocs ; pour cacher un bloc sur un téléphone : « phone: { display: none; } » ; selon une valeur : If (ADR-017, ADR-041)".into()),
        };
    }
    // Les « … » d'un texte trop long : en CSS, il faut trois réglages ensemble, et `text-overflow`
    // seul ne fait rien. Ici, un seul (ADR-069).
    if name == "text-overflow" {
        return refusal("« text-overflow » ne fait rien seul ; écris « line-clamp: 1 » : une seule ligne, et « … » à la fin (ou « line-clamp: 3 » : trois lignes)".into());
    }
    // Un texte qui ne passe jamais à la ligne déborde de l'écran d'un téléphone (ADR-069).
    if name == "white-space" && setting.value == "pre" {
        return refusal("« white-space: pre » ne passe jamais à la ligne : sur un téléphone, le texte sort de l'écran ; écris « pre-wrap » : les espaces et les retours à la ligne sont gardés, et la ligne passe quand il le faut".into());
    }
    // La place d'une case de grille se dit sur le bloc, jamais dans un style (ADR-104).
    if matches!(name, "grid-column" | "grid-row" | "grid-area" | "grid-template-areas" | "grid-column-start" | "grid-column-end" | "grid-row-start" | "grid-row-end") {
        return refusal(format!("« {name} » place une case de grille : la place se dit sur le bloc, columnSpan: 2, rowSpan: 2, ou une zone, Grid(areas: [\"top top\", \"menu main\"]) puis area: menu (ADR-104)"));
    }
    if LAYOUT.contains(&name) {
        return refusal(format!(
            "« {name} » règle la disposition, pas l'apparence : un style ne dit que l'apparence, la disposition vient des blocs (ADR-017)"
        ));
    }
    if name == "background-color" {
        return refusal("« background-color » s'écrit « background » : une seule écriture par réglage".into());
    }
    let Some((_, shape)) = SETTINGS.iter().find(|(known, _)| *known == name) else {
        let known_ones: Vec<&str> = SETTINGS.iter().map(|(n, _)| *n).collect();
        return refusal(format!("réglage inconnu « {name} » ; réglages possibles : {}", known_ones.join(", ")));
    };
    // Les variables sont remplacées par leur valeur, pour vérifier ce qu'elles donnent.
    let mut value = setting.value.clone();
    for variable in variables_of(&setting.value) {
        match variables.iter().find(|(n, _)| n == variable) {
            Some((_, replacement)) => value = value.replacen(variable, replacement, 1),
            None => return refusal(format!("« {variable} » n'est définie nulle part : écris « Page {{ {variable}: … }} »")),
        }
    }
    let value = value.as_str();
    // `height: screen` : tout l'écran, au moins (ADR-061).
    if name == "height" && value == "screen" {
        return Ok(());
    }
    let words: Vec<&str> = value.split_whitespace().collect();
    let correct = match shape {
        Shape::Color => words.len() == 1 && is_color(value),
        Shape::Size => words.len() == 1 && is_size(value),
        Shape::Sizes => (1..=4).contains(&words.len()) && words.iter().all(|m| is_size(m)),
        Shape::Word(possible) => possible.contains(&value),
        Shape::Fraction => value.parse::<f64>().is_ok_and(|v| (0.0..=1.0).contains(&v)),
        Shape::Border => words.len() == 3 && is_size(words[0]) && ["solid", "dashed", "dotted"].contains(&words[1]) && is_color(words[2]),
        Shape::Font => value.split(',').all(|font| {
            let font = font.trim().trim_matches('"');
            !font.is_empty() && font.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-')
        }),
        Shape::Background => is_color(value) || is_gradient(value) || background_image(value).is_some(),
        Shape::Number(min, max) => value.parse::<f64>().is_ok_and(|v| (*min..=*max).contains(&v)),
        Shape::Gap => signed_pixels(value).is_some_and(|v| (-10.0..=40.0).contains(&v)),
        Shape::Shadow => value == "none" || {
            let shadows: Vec<&str> = value.split(',').map(str::trim).collect();
            shadows.len() <= 3 && shadows.iter().all(|shadow| is_shadow(shadow))
        },
        Shape::Angle => value.strip_suffix("deg").and_then(|n| n.parse::<f64>().ok()).is_some_and(|v| (-360.0..=360.0).contains(&v)),
        Shape::Duration => value == "none" || duration_in_ms(value).is_some_and(|ms| (0.0..=2000.0).contains(&ms)),
        Shape::Ratio => {
            let positive = |n: &str| n.trim().parse::<f64>().is_ok_and(|v| v.is_finite() && v > 0.0 && v <= 1000.0);
            match value.split_once('/') {
                Some((width, height)) => positive(width) && positive(height),
                None => positive(value),
            }
        }
        Shape::Count(min, max) => value.parse::<u32>().is_ok_and(|v| (*min..=*max).contains(&v)),
        Shape::Cursor => CURSORS.contains(&value) || cursor_image(value).is_some(),
    };
    if correct {
        return Ok(());
    }
    let expected = match shape {
        Shape::Color => "une couleur, comme « gray » ou « #E9B44C »".to_string(),
        Shape::Size if name == "height" => "une taille, comme « 16px » ou « 50% », ou « screen » : tout l'écran".to_string(),
        Shape::Size => "une taille, comme « 16px » ou « 50% »".to_string(),
        Shape::Sizes => "une à quatre tailles, comme « 8px 16px »".to_string(),
        Shape::Word(possible) => format!("l'un de ces mots : {}", possible.join(", ")),
        Shape::Fraction => "un nombre entre 0 et 1".to_string(),
        Shape::Border => "une épaisseur, un trait et une couleur, comme « 1px solid gray »".to_string(),
        Shape::Font => "un ou plusieurs noms de police, séparés par des virgules".to_string(),
        Shape::Background => "une couleur, un dégradé comme « linear-gradient(#E9B44C, #1a1a2e) », ou une image rangée à côté, « url(\"fond.jpg\") »".to_string(),
        Shape::Number(min, max) if name == "line-height" => format!("un nombre sans unité, de {min} à {max}, comme « 1.5 » : la hauteur de ligne suit alors la taille du texte"),
        Shape::Number(min, max) => format!("un nombre de {min} à {max}, comme « 1.1 »"),
        Shape::Gap => "un écart en pixels, de -10px à 40px, comme « 1px »".to_string(),
        Shape::Shadow => "une ombre : décalage, flou et couleur, comme « 0 4px 12px #00000066 » (trois au plus, séparées par des virgules), ou « none »".to_string(),
        Shape::Angle => "un angle de -360deg à 360deg, comme « -3deg »".to_string(),
        Shape::Duration => "une durée de 0 à 2s, comme « 0.3s » ou « 200ms », ou « none »".to_string(),
        Shape::Ratio => "des proportions, la largeur puis la hauteur, comme « 16/9 », ou « 1 » pour un carré".to_string(),
        Shape::Count(min, max) if name == "line-clamp" => format!("un nombre de lignes, de {min} à {max}, comme « 3 »"),
        Shape::Count(min, max) => format!("un nombre entier de {min} à {max}"),
        Shape::Cursor => format!("l'une de ces formes : {}, ou une image rangée à côté, « url(\"viseur.png\") » (.png, .svg ou .cur)", CURSORS.join(", ")),
    };
    refusal(format!("« {name}: {value} » : ce réglage attend {expected}"))
}

pub(crate) fn is_color(value: &str) -> bool {
    match value.strip_prefix('#') {
        Some(hex) => [3, 6, 8].contains(&hex.len()) && hex.bytes().all(|c| c.is_ascii_hexdigit()),
        None => COLORS.contains(&value),
    }
}

/// `linear-gradient(to right, #E9B44C, #1a1a2e)` ou `radial-gradient(white, navy)` : une
/// direction ou un angle facultatifs, puis de deux à cinq couleurs.
fn is_gradient(value: &str) -> bool {
    let Some(inside) = value.strip_prefix("linear-gradient(").or_else(|| value.strip_prefix("radial-gradient(")).and_then(|v| v.strip_suffix(')')) else { return false };
    let mut parts: Vec<&str> = inside.split(',').map(str::trim).collect();
    let linear = value.starts_with("linear");
    if linear && parts.first().is_some_and(|p| p.starts_with("to ") || p.ends_with("deg")) {
        let direction = parts.remove(0);
        let correct = match direction.strip_prefix("to ") {
            Some(sides) => sides.split_whitespace().all(|c| ["top", "bottom", "left", "right"].contains(&c)),
            None => direction.strip_suffix("deg").and_then(|n| n.parse::<f64>().ok()).is_some_and(|v| (-360.0..=360.0).contains(&v)),
        };
        if !correct {
            return false;
        }
    }
    (2..=5).contains(&parts.len()) && parts.iter().all(|c| is_color(c))
}

/// `url("fond.jpg")` : le nom d'une image rangée à côté du fichier.
pub(crate) fn background_image(value: &str) -> Option<&str> {
    let inside = value.strip_prefix("url(")?.strip_suffix(')')?.trim().trim_matches('"');
    let image = [".png", ".jpg", ".jpeg", ".webp", ".svg", ".gif", ".avif"].iter().any(|end| inside.ends_with(end));
    (image && crate::flat::path_on(inside)).then_some(inside)
}

/// `url("viseur.png")` : le nom d'une image de curseur rangée à côté du fichier (ADR-069).
pub(crate) fn cursor_image(value: &str) -> Option<&str> {
    let inside = value.strip_prefix("url(")?.strip_suffix(')')?.trim().trim_matches('"');
    let image = [".png", ".svg", ".cur"].iter().any(|end| inside.ends_with(end));
    (image && crate::flat::path_on(inside)).then_some(inside)
}

/// `-0.5px` → -0.5.
fn signed_pixels(value: &str) -> Option<f64> {
    if value == "0" {
        return Some(0.0);
    }
    value.strip_suffix("px")?.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Une ombre : deux décalages (qui peuvent être négatifs), un flou facultatif, une couleur.
fn is_shadow(shadow: &str) -> bool {
    let words: Vec<&str> = shadow.split_whitespace().collect();
    let Some((color, sizes)) = words.split_last() else { return false };
    is_color(color)
        && (2..=3).contains(&sizes.len())
        && sizes[..2].iter().all(|t| signed_pixels(t).is_some_and(|v| v.abs() <= 100.0))
        && sizes.get(2).is_none_or(|blur| is_size(blur) && signed_pixels(blur).is_some_and(|v| v <= 200.0))
}

/// `0.3s` → 300 ; `200ms` → 200.
pub(crate) fn duration_in_ms(value: &str) -> Option<f64> {
    if let Some(ms) = value.strip_suffix("ms") {
        return ms.parse().ok();
    }
    value.strip_suffix('s')?.parse::<f64>().ok().map(|s| s * 1000.0)
}

fn is_size(value: &str) -> bool {
    if value == "0" {
        return true;
    }
    let number = value.strip_suffix("px").or_else(|| value.strip_suffix('%'));
    number.is_some_and(|n| !n.is_empty() && !n.starts_with('-') && n.parse::<f64>().is_ok_and(f64::is_finite))
}

fn uppercase(name: &str) -> String {
    let mut letters = name.chars();
    letters.next().map(|c| format!("{}{}", c.to_ascii_uppercase(), letters.as_str())).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::read;

    fn check(src: &str) -> Result<(), Error> {
        check_styles(&read(src)?)
    }

    fn page(styles: &str) -> String {
        format!("Page(children: [ P.card(\"x\") ])\n.card {{ color: gray; }}\n{styles}")
    }

    #[test]
    fn the_suite_shop_is_accepted() {
        let source = include_str!("../../experiments/conformite-v0.1/cas/valides/06-boutique-avec-styles.holo");
        let program = read(source).unwrap();
        check_styles(&program).unwrap();
        crate::blocks::check_blocks(&program).unwrap();
        assert_eq!(program.styles.len(), 5);
        assert_eq!(program.styles[0].target, Target::Type("Page".into()));
        assert_eq!(program.styles[3].target, Target::Name("card".into()));
        assert_eq!(program.styles[3].settings[0].name, "background");
        assert_eq!(program.styles[3].settings[0].value, "#1a1a2e");
    }

    #[test]
    fn the_compared_shop_uses_the_whole_vocabulary() {
        let source = include_str!("../../exemples/boutique-comparee/boutique.holo");
        let program = read(source).unwrap();
        crate::blocks::check_blocks(&program).unwrap();
        check_styles(&program).unwrap();
        // Le jeu complète la boutique : à eux deux, ils emploient tous les mots du langage.
        let game = include_str!("../../exemples/jeu/attraper.holo");
        crate::check_page(game).unwrap();
        let second = include_str!("../../exemples/jeu/panier.holo");
        crate::check_page(second).unwrap();
        // Un site de deux pages, avec un morceau importé.
        let (home, common) = (include_str!("../../exemples/site/accueil.holo"), include_str!("../../exemples/site/commun.holo"));
        crate::check_page(&format!("{home}{}commun.holo{}{common}", crate::holo::NEXT_FILE, crate::holo::NAME_SEPARATOR)).unwrap();
        let contact = include_str!("../../exemples/site/contact.holo");
        crate::check_page(&format!("{contact}{}commun.holo{}{common}", crate::holo::NEXT_FILE, crate::holo::NAME_SEPARATOR)).unwrap();
        let data = include_str!("../../exemples/lecons/27-donnees.holo");
        crate::check_page(data).unwrap();
        // Le film en mouvement (ADR-034).
        let film = include_str!("../../exemples/motion/holocode/showreel.holo");
        crate::check_page(film).unwrap();
        // Les repères, les titres profonds, les états et la superposition (ADR-036).
        let lessons = [
            include_str!("../../exemples/lecons/35-reperes.holo"),
            include_str!("../../exemples/lecons/36-titres-profonds.holo"),
            include_str!("../../exemples/lecons/37-survol.holo"),
            include_str!("../../exemples/lecons/38-superposition.holo"),
            include_str!("../../exemples/lecons/41-video.holo"),
            include_str!("../../exemples/lecons/42-tableau.holo"),
            include_str!("../../exemples/lecons/44-choix.holo"),
            include_str!("../../exemples/lecons/45-survol-qui-agit.holo"),
            include_str!("../../exemples/lecons/46-sinon.holo"),
            include_str!("../../exemples/lecons/47-plus-tard.holo"),
            include_str!("../../exemples/lecons/48-heure.holo"),
            include_str!("../../exemples/lecons/49-repeter.holo"),
            include_str!("../../exemples/lecons/50-texte-soigne.holo"),
            include_str!("../../exemples/lecons/51-ombres-et-fonds.holo"),
            include_str!("../../exemples/lecons/52-variables-et-theme-sombre.holo"),
            include_str!("../../exemples/lecons/53-telephone.holo"),
            include_str!("../../exemples/lecons/54-police.holo"),
            include_str!("../../exemples/lecons/55-petits-textes.holo"),
            include_str!("../../exemples/lecons/56-aller-plus-bas.holo"),
            include_str!("../../exemples/lecons/57-image-et-legende.holo"),
            include_str!("../../exemples/lecons/58-lecteur-de-son.holo"),
            include_str!("../../exemples/lecons/59-glissiere.holo"),
            include_str!("../../exemples/lecons/60-date-heure-couleur.holo"),
            include_str!("../../exemples/lecons/61-progression.holo"),
            include_str!("../../exemples/lecons/62-plis.holo"),
            include_str!("../../exemples/lecons/63-fenetre.holo"),
            include_str!("../../exemples/lecons/64-formulaire.holo"),
            include_str!("../../exemples/lecons/65-icone-de-l-onglet.holo"),
            include_str!("../../exemples/lecons/66-calculer.holo"),
            include_str!("../../exemples/lecons/67-formats.holo"),
            include_str!("../../exemples/lecons/68-liste-qui-change.holo"),
            include_str!("../../exemples/lecons/69-module-enferme.holo"),
            include_str!("../../exemples/lecons/82-chercher-filtrer-trier.holo"),
            include_str!("../../exemples/lecons/84-donnees-arrivees-ou-pas.holo"),
            include_str!("../../exemples/lecons/87-des-dates.holo"),
            // La mise en page (ADR-069).
            include_str!("../../exemples/lecons/89-telephone-et-ordinateur.holo"),
            include_str!("../../exemples/lecons/90-ce-qui-depasse.holo"),
            include_str!("../../exemples/lecons/91-garder-des-proportions.holo"),
            include_str!("../../exemples/lecons/92-le-curseur.holo"),
            include_str!("../../exemples/lecons/93-texte-justifie.holo"),
            include_str!("../../exemples/lecons/94-decrocher-la-page.holo"),
            // Le HTML et les médias qui manquent (ADR-073).
            include_str!("../../exemples/lecons/95-un-article-long.holo"),
            include_str!("../../exemples/lecons/96-une-video-sous-titree.holo"),
            // Les capacités larges (ADR-077, ADR-086, ADR-087, ADR-088).
            include_str!("../../exemples/lecons/97-un-module-qui-recoit-une-liste.holo"),
            include_str!("../../exemples/lecons/98-un-dessin.holo"),
            include_str!("../../exemples/lecons/99-un-tableau-de-bord.holo"),
            include_str!("../../exemples/lecons/110-un-module-qui-dessine.holo"),
            // Les valeurs partagées (ADR-079).
            include_str!("../../exemples/lecons/101-une-valeur-partagee.holo"),
            // Les secondes et le chronomètre (ADR-089).
            include_str!("../../exemples/lecons/111-un-chronometre.holo"),
            // Trois dettes des lots 4 et 5 (ADR-090).
            include_str!("../../exemples/lecons/112-une-adresse-qui-se-souvient.holo"),
            include_str!("../../exemples/lecons/113-une-rangee-qui-se-serre.holo"),
            // L'historique dans une page (ADR-091).
            include_str!("../../exemples/lecons/114-l-historique-dans-une-page.holo"),
            // Des polices libres pour toutes les écritures (ADR-092).
            include_str!("../../exemples/lecons/115-des-polices-pour-toutes-les-ecritures.holo"),
            include_str!("../../exemples/lecons/116-importer-et-exporter.holo"),
            include_str!("../../exemples/lecons/117-appareil-sur-permission.holo"),
            include_str!("../../exemples/lecons/118-notifications-locales.holo"),
            include_str!("../../exemples/lecons/119-une-page-hors-ligne.holo"),
            include_str!("../../exemples/lecons/120-une-liste-de-definitions.holo"),
            // Une abréviation, une date pour les machines, une adresse (ADR-098).
            include_str!("../../exemples/lecons/121-une-abreviation-une-date-une-adresse.holo"),
            // Un groupe de champs et son nom (ADR-099).
            include_str!("../../exemples/lecons/122-un-groupe-de-champs.holo"),
            // Des suggestions dans un champ (ADR-100).
            include_str!("../../exemples/lecons/123-des-suggestions-dans-un-champ.holo"),
            // Une citation courte, le titre d'une œuvre (ADR-101).
            include_str!("../../exemples/lecons/124-une-citation-courte.holo"),
            // Travailler un texte : majuscules, longueur, couper, découper (ADR-103).
            include_str!("../../exemples/lecons/126-travailler-un-texte.holo"),
            // Une grille qui place ses cases : plusieurs colonnes ou lignes, des zones (ADR-104).
            include_str!("../../exemples/lecons/127-une-grille-et-ses-zones.holo"),
            // Partager la page : la feuille du téléphone, sinon l'adresse copiée (ADR-107).
            include_str!("../../exemples/lecons/130-partager-la-page.holo"),
            // Réordonner les lignes d'une liste (ADR-105).
            include_str!("../../exemples/lecons/128-reordonner-une-liste.holo"),
            // Faire vibrer le téléphone, d'un toucher ou d'une règle de jeu (ADR-110).
            include_str!("../../exemples/lecons/133-faire-vibrer-le-telephone.holo"),
            // Mélanger des sons : un fondu, un volume qui suit une valeur (ADR-112).
            include_str!("../../exemples/lecons/135-melanger-des-sons.holo"),
            // Se souvenir le temps d'une visite, un formulaire en deux pages (ADR-113).
            include_str!("../../exemples/lecons/136-se-souvenir-le-temps-d-une-visite.holo"),
            include_str!("../../exemples/lecons/136-inscription/etape-2.holo"),
            // Les données d'un autre site, lues par le serveur de l'auteur (ADR-116).
            include_str!("../../exemples/lecons/139-les-donnees-d-un-autre-site.holo"),
            // Une page dans la page : une carte et une vidéo d'autres sites, derrière leur façade (ADR-117).
            include_str!("../../exemples/lecons/140-une-page-dans-la-page.holo"),
        ];
        for lesson in lessons {
            crate::check_page(lesson).unwrap();
        }
        let lessons = lessons.join("\n");
        let source = &format!("{source}\n{game}\n{second}\n{home}\n{common}\n{data}\n{film}\n{lessons}");
        for block in crate::blocks::BLOCKS {
            assert!(source.contains(&format!("{block}(")) || source.contains(&format!("{block}.")), "le bloc « {block} » manque dans l'exemple");
        }
        for (setting, _) in SETTINGS {
            assert!(source.contains(&format!("{setting}:")), "le réglage « {setting} » manque dans l'exemple");
        }
        for word in ["name:", "title:", "seed:", "brightness:", "fragments:", "children:", "inside:", "rules:", "effect:", "budget:", "weight:", "source:", "text:", "color:", "palette:", ".tap", ".enter", ".leave", "state:", "prices:", "{count}", "{total}", ".add(", ".sub(", ".set(", "gap:", "align:", "columns:", "alt:", "is:", "over:", "by:", ".random(", "x:", "y:", "keep:", "value:", "label:", "max:", "Key.left", "meets:", "drag:", "data:", "from:", ".play", "form:", "enter:", "loop:", "letters:", "each:", "repeat:", "ease:", "rotate:", "flip:", "tilt:", "blur:", "hue:", "round:", "scale:", "opacity:", "hover:", "focus:", "active:", "topRight", ".hover", ".hoverEnd", "else:", "{year}", "{month}", "{day}", "weekday", "{hour}", "{minute}", "{second}", ":stopwatch}", "items:", "key:", "{item.", "item.add(", "dark:", "phone:", "display: none", "linear-gradient(", "url(", "fonts:", "family:", ": --", "~~", "==", "^2^", "~2~", "to: \"#", "caption:", "phone:", "type: date", "type: time", "type: color", "summary:", "open: true", ".open", ".close", ".send", ".sent", ".failed", "icon:", ".mul(", ".div(", ":00}", ":number}", ":cents}", ":name}", "over:", ".push(", ".remove(item)", ".clear()", ".set(\"\")", "module \"", "modules:", ".run", ".done", "time:", "memory:", ".refresh", "computer:", "narrow:", "detach:", "justify"] {
            assert!(source.contains(word), "« {word} » manque dans l'exemple");
        }
        // Une grille qui place ses cases (ADR-104).
        for word in ["columnSpan:", "rowSpan:", "areas:", "area:"] {
            assert!(source.contains(word), "« {word} » manque dans l'exemple");
        }
        // Une page dans la page : les sites permis, l'image de la façade (ADR-117).
        for word in ["embeds:", "Embed(", "image: \"140-carte.svg\""] {
            assert!(source.contains(word), "« {word} » manque dans l'exemple");
        }
        // Travailler un texte (ADR-103).
        for word in [":upper}", ":lower}", ":length}", ":max40}", "Split(", "by: \",\"", "by: \" \""] {
            assert!(source.contains(word), "« {word} » manque dans l'exemple");
        }
    }

    #[test]
    fn refuses_what_the_suite_refuses_at_the_right_line() {
        let sample = [
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E12-reglage-inconnu.holo"), 6, "réglage inconnu « colour »"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E13-disposition-dans-un-style.holo"), 7, "la disposition vient des blocs"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E14-style-non-defini.holo"), 5, "n'est défini nulle part"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E15-point-virgule-manquant.holo"), 6, "« ; » manquant"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E16-selecteur-compose.holo"), 6, "rien d'autre"),
        ];
        for (source, line, message) in sample {
            let error = check(source).unwrap_err();
            assert_eq!(error.pos.line, line, "{error}");
            assert!(error.message.contains(message), "{error}");
        }
    }

    #[test]
    fn nothing_is_tolerated_silently() {
        assert!(check(&page("P { color: grey; }")).unwrap_err().message.contains("une couleur"));
        assert!(check(&page("P { font-size: 16; }")).unwrap_err().message.contains("une taille"));
        assert!(check(&page("P { font-size: 16em; }")).unwrap_err().message.contains("une taille"));
        assert!(check(&page("P { color: gray; color: red; }")).unwrap_err().message.contains("deux fois"));
        assert!(check(&page(".card { color: red; }")).unwrap_err().message.contains("défini deux fois"));
        assert!(check(&page("p { color: gray; }")).unwrap_err().message.contains("écris « P { … } »"));
        assert!(check(&page("Div { color: gray; }")).unwrap_err().message.contains("bloc inconnu"));
        assert!(check(&page("P { background-color: red; }")).unwrap_err().message.contains("s'écrit « background »"));
        assert!(check(&page("P { opacity: 2; }")).unwrap_err().message.contains("entre 0 et 1"));
        assert!(check(&page("P { border: 1px gray; }")).unwrap_err().message.contains("1px solid gray"));
    }

    #[test]
    fn accepts_base_css() {
        check(&page(
            "P { font-size: 16px; font-weight: bold; font-family: Georgia, \"Times New Roman\"; text-align: center }\n\
             H1 { color: #E9B44C; margin: 0 0 8px 0; }\n\
             Button { border: 1px solid gold; border-radius: 8px; padding: 8px 16px; opacity: 0.9; background: transparent; }\n\
             Image { width: 100%; max-width: 320px; }",
        ))
        .unwrap();
        // Un fichier sans style reste valable, et un style qui ne sert pas n'est pas une erreur.
        check("Point(name: A, seed: 1)").unwrap();
        check("Point(name: A, seed: 1)\n.card { color: gray; }").unwrap();
    }

    #[test]
    fn a_low_contrast_text_is_refused() {
        let page = |styles: &str| format!("Page(children: [ H1(\"x\"), P.card(\"y\") ])\n{styles}");
        // Refusé : blanc sur orange vif, en petit.
        let error = check(&page(".card { color: white; background: #E4572E; }")).unwrap_err();
        assert!(error.message.contains("contraste de 3,6 pour 1 ; il faut 4,5"), "{error}");
        // Accepté : le même en grand texte (3 pour 1 suffit), ou un fond plus sombre.
        assert!(check(&page(".card { color: white; background: #E4572E; font-size: 24px; }")).is_ok());
        assert!(check(&page(".card { color: white; background: #B83A1F; }")).is_ok());
        // Une variable, et un état sombre qui redéfinit la sienne.
        assert!(check(&page("Page { --ink: #777777; }\n.card { color: --ink; background: #888888; }")).unwrap_err().message.contains(".card"));
        assert!(check(&page("Page { --ink: #1a1a2e; background: white; color: --ink; dark: { --ink: #F5F5F5; background: #101020; } }\n.card { padding: 4px; }")).is_ok());
        let error = check(&page(".card { color: navy; background: white; hover: { background: blue; } }")).unwrap_err();
        assert!(error.message.contains("dans l'état « hover »"), "{error}");
        // Un dégradé, une image, une couleur à demi transparente : non mesurés.
        assert!(check(&page(".card { color: white; background: linear-gradient(white, #eeeeee); }")).is_ok());
    }

    #[test]
    fn the_layout_settings_of_lot_4() {
        // Accepté : les tailles au plus et au moins, les proportions, ce qui dépasse, le curseur,
        // le texte justifié, et les états d'écran (ADR-069).
        assert!(check(&page(".box { min-width: 120px; min-height: 60px; max-height: 50%; aspect-ratio: 16/9; overflow: auto; overflow-x: hidden; white-space: nowrap; line-clamp: 3; cursor: help; text-align: justify; }")).is_ok());
        assert!(check(&page("Image { aspect-ratio: 4 / 3; object-fit: contain; object-position: top left; cursor: url(\"viseur.svg\"); }")).is_ok());
        assert!(check(&page(".box { computer: { font-size: 22px; display: none; } narrow: { padding: 8px; display: none; } }")).is_ok());
        // Refusé, avec le bon mot.
        assert!(check(&page(".box { text-overflow: ellipsis; }")).unwrap_err().message.contains("line-clamp: 1"));
        assert!(check(&page(".box { white-space: pre; }")).unwrap_err().message.contains("pre-wrap"));
        assert!(check(&page(".box { white-space: pre-wrap; cursor: copy; }")).is_ok());
        assert!(check(&page(".box { aspect-ratio: 16:9; }")).unwrap_err().message.contains("16/9"));
        assert!(check(&page(".box { line-clamp: 0; }")).unwrap_err().message.contains("nombre de lignes"));
        assert!(check(&page(".box { cursor: hand; }")).unwrap_err().message.contains("pointer"));
        assert!(check(&page(".box { cursor: url(\"viseur.jpg\"); }")).unwrap_err().message.contains(".cur"));
        assert!(check(&page(".box { computer: { display: flex; } }")).unwrap_err().message.contains("ne prend que « none »"));
        assert!(check(&page(".box { wide: { color: red; } }")).unwrap_err().message.contains("computer"));
    }

    #[test]
    fn what_acts_exists_on_every_device() {
        // La parité (ADR-069) : une phrase peut se cacher sur un seul appareil, pas ce qui agit.
        let src = |styles: &str| format!("Page(children: [ P.note(\"x\"), Button(name: Buy, text: \"Buy\"), Column.menu(children: [ A(\"Home\", to: \"a.holo\") ]), Text.card(name: Card, text: \"y\") ], rules: [ On(Card.tap, effect: Card.hover) ])\n{styles}");
        assert!(check(&src(".note { phone: { display: none; } }\n.menu { color: red; }\n.card { color: red; }")).is_ok());
        let refused = |styles: &str| check(&src(styles)).unwrap_err().message;
        assert!(refused("Button { phone: { display: none; } }\n.note { color: red; }\n.menu { color: red; }\n.card { color: red; }").contains("cacherait « Button » sur un téléphone"));
        assert!(refused(".menu { computer: { display: none; } }\n.note { color: red; }\n.card { color: red; }").contains("cacherait « A » sur un ordinateur"));
        assert!(refused(".card { narrow: { display: none; } }\n.note { color: red; }\n.menu { color: red; }").contains("dans une case étroite"));
    }

    #[test]
    fn a_block_carries_several_style_names() {
        assert!(check("Page(children: [ P.card.big(\"x\") ])\n.card { color: red; }\n.big { font-size: 24px; }").is_ok());
        assert!(check("Page(children: [ P.card.big(\"x\") ])\n.card { color: red; }").unwrap_err().message.contains("« .big » n'est défini nulle part"));
        assert!(check("Page(children: [ P.a.b.c.d.e(\"x\") ])").unwrap_err().message.contains("trop de noms de style"));
        assert_eq!(read("Page(children: [ P.card(\"x\") ])").unwrap().root.arguments.len(), 1);
    }
}
