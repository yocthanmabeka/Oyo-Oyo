//! Le dessin vectoriel déclaré (ADR-086) : des formes posées dans un `Drawing`, fabriquées en SVG
//! dans la page. Net à toute taille, lu par le lecteur d'écran par son `label`, et sans dessin
//! trait par trait en JavaScript : l'auteur dit ce qu'il y a, pas comment le tracer.
//!
//! ```holo
//! Drawing(label: "Un paysage", width: 200, height: 100, children: [
//!   Rect(x: 0, y: 0, width: 200, height: 100, fill: "#101020"),
//!   Circle(x: 150, y: sun, r: 12, fill: "#E9B44C"),
//!   Line(from: [0, 80], to: [200, 80], stroke: "#8fd3ff", thickness: 2),
//!   Path(d: "M20 80 L40 60 L60 80 Z", fill: "#9be7a1"),
//! ])
//! ```
//!
//! Les mesures sont dans les unités du dessin : `width` sur `height`, comme une feuille ; le dessin
//! garde ses proportions et rétrécit avec l'écran. Une mesure peut être le nom d'un nombre entier
//! de la page (`y: sun`) : la forme bouge quand il change.

use crate::holo::{Block, Error, Program, Value};

/// Les formes qu'on pose dans un `Drawing`.
pub const SHAPES: &[&str] = &["Rect", "Circle", "Line", "Path"];
/// Les réglages de chaque forme.
pub const RECT: &[&str] = &["name", "x", "y", "width", "height", "radius", "fill", "stroke", "thickness", "opacity"];
pub const CIRCLE: &[&str] = &["name", "x", "y", "r", "fill", "stroke", "thickness", "opacity"];
pub const LINE: &[&str] = &["name", "from", "to", "stroke", "thickness", "opacity"];
pub const PATH: &[&str] = &["name", "d", "fill", "stroke", "thickness", "opacity"];
/// La plus grande mesure d'un dessin, et le nombre de formes qu'il contient, au plus.
pub const SIZE_MAX: u32 = 4000;
pub const SHAPES_MAX: usize = 500;
/// Le tracé d'un `Path`, au plus.
pub const PATH_MAX: usize = 4000;

/// Le signe qui entoure, dans le HTML en fabrication, le nom d'un nombre de la page à écrire :
/// celui des autres marques de la page plate.
const MARK: char = '\u{1}';

/// Le dessin entier, en SVG.
pub fn html(block: &Block, classes: &str, name: &str) -> Result<String, Error> {
    let error = |message: String, pos| Error { message, pos };
    let example = "Drawing(label: \"Un paysage\", width: 200, height: 100, children: [ Circle(x: 50, y: 50, r: 20) ])";
    let (mut label, mut width, mut height) = (None, None, None);
    for argument in &block.arguments {
        match (argument.name.as_deref(), &argument.value) {
            (Some("name" | "children"), _) => {}
            (Some("label"), Value::Text(text)) if !text.trim().is_empty() => label = Some(text.as_str()),
            (Some("label"), _) => return Err(error("« Drawing(label: …) » dit ce que montre le dessin, pour qui ne le voit pas : label: \"Un paysage\"".into(), argument.pos)),
            (Some(side @ ("width" | "height")), Value::Integer(n)) if (1..=u64::from(SIZE_MAX)).contains(n) => {
                if side == "width" {
                    width = Some(*n);
                } else {
                    height = Some(*n);
                }
            }
            (Some(side @ ("width" | "height")), _) => return Err(error(format!("« Drawing({side}: …) » attend un nombre entier de 1 à {SIZE_MAX} : les unités du dessin"), argument.pos)),
            (Some(other), _) => return Err(error(format!("« Drawing » n'a pas de paramètre « {other} » ; paramètres possibles : label, width, height, children, name"), argument.pos)),
            (None, _) => return Err(error(format!("chaque paramètre de « Drawing » est nommé : {example}"), argument.pos)),
        }
    }
    let (Some(label), Some(width), Some(height)) = (label, width, height) else {
        return Err(error(format!("« Drawing » attend label, width et height : {example}"), block.pos));
    };
    let shapes = match block.argument("children").map(|a| &a.value) {
        Some(Value::List(elements)) => elements.as_slice(),
        None => &[],
        Some(_) => return Err(error(format!("« Drawing(children: …) » est une liste de formes : {example}"), block.pos)),
    };
    if shapes.len() > SHAPES_MAX {
        return Err(error(format!("un dessin contient au plus {SHAPES_MAX} formes"), block.pos));
    }
    let mut output = format!("<svg class=\"{classes}\"{name} role=\"img\" aria-label=\"{}\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\">", escape(label));
    for element in shapes {
        let Value::Block(shape) = element else {
            return Err(error(format!("« Drawing » contient des formes : {}", SHAPES.join(", ")), block.pos));
        };
        output.push_str(&shape_html(shape)?);
    }
    output.push_str("</svg>");
    Ok(output)
}

/// Une forme, en SVG. Ses mesures liées à des valeurs de la page sont notées dans `data-svg`
/// (`cy:sun`) : la page les récrit quand elles changent.
fn shape_html(shape: &Block) -> Result<String, Error> {
    let error = |message: String, pos| Error { message, pos };
    let (tag, known): (&str, &[&str]) = match shape.name.as_str() {
        "Rect" => ("rect", RECT),
        "Circle" => ("circle", CIRCLE),
        "Line" => ("line", LINE),
        "Path" => ("path", PATH),
        other => return Err(error(format!("« {other} » ne se dessine pas ; un « Drawing » contient des formes : {}", SHAPES.join(", ")), shape.pos)),
    };
    let mut attributes = String::new();
    let mut followed = Vec::new();
    let (mut fill, mut stroke, mut thickness) = (None, None, None);
    for argument in &shape.arguments {
        let parameter = argument.name.as_deref().unwrap_or("");
        if !known.contains(&parameter) {
            return Err(error(format!("« {} » n'a pas de paramètre « {parameter} » ; paramètres possibles : {}", shape.name, known.join(", ")), argument.pos));
        }
        match (parameter, &argument.value) {
            ("name", _) => {}
            ("fill", Value::Text(color)) if is_paint(color) => fill = Some(color.as_str()),
            ("stroke", Value::Text(color)) if is_paint(color) => stroke = Some(color.as_str()),
            ("fill" | "stroke", _) => return Err(error(format!("« {}({parameter}: …) » attend une couleur entre guillemets, comme \"#E9B44C\", ou \"none\"", shape.name), argument.pos)),
            ("thickness", value) => thickness = Some(measure(shape, parameter, value)?),
            ("opacity", Value::Number { value, unit: None, .. }) if (0.0..=1.0).contains(value) => attributes.push_str(&format!(" opacity=\"{value}\"")),
            ("opacity", Value::Integer(n @ 0..=1)) => attributes.push_str(&format!(" opacity=\"{n}\"")),
            ("opacity", _) => return Err(error(format!("« {}(opacity: …) » attend un nombre de 0 (invisible) à 1", shape.name), argument.pos)),
            ("d", Value::Text(d)) if is_path(d) => attributes.push_str(&format!(" d=\"{d}\"")),
            ("d", _) => return Err(error(format!("« Path(d: …) » attend un tracé SVG qui commence par M, comme \"M10 80 L50 20 Z\" : des lettres de tracé, des nombres, des espaces ; {PATH_MAX} signes au plus"), argument.pos)),
            (end @ ("from" | "to"), Value::List(pair)) if pair.len() == 2 => {
                let (x, y) = if end == "from" { ("x1", "y1") } else { ("x2", "y2") };
                for (attribute, value) in [(x, &pair[0]), (y, &pair[1])] {
                    attributes.push_str(&written(shape, end, attribute, value, &mut followed)?);
                }
            }
            ("from" | "to", _) => return Err(error(format!("« Line({parameter}: …) » attend deux mesures, x et y : {parameter}: [0, 80]"), argument.pos)),
            (measured, value) => {
                let attribute = match (tag, measured) {
                    ("circle", "x") => "cx",
                    ("circle", "y") => "cy",
                    ("rect", "radius") => "rx",
                    (_, other) => other,
                };
                attributes.push_str(&written(shape, measured, attribute, value, &mut followed)?);
            }
        }
    }
    // Ce qu'il faut pour que la forme existe.
    let required: &[&str] = match tag {
        "rect" => &["x", "y", "width", "height"],
        "circle" => &["x", "y", "r"],
        "line" => &["from", "to"],
        _ => &["d"],
    };
    for parameter in required {
        if shape.argument(parameter).is_none() {
            return Err(error(format!("« {} » attend {} : {}", shape.name, required.join(", "), example(tag)), shape.pos));
        }
    }
    // Une forme fermée se remplit de la couleur du texte ; un trait se trace de cette couleur.
    if tag == "line" {
        attributes.push_str(&format!(" stroke=\"{}\" stroke-width=\"{}\"", stroke.unwrap_or("currentColor"), thickness.as_deref().unwrap_or("1")));
    } else {
        attributes.push_str(&format!(" fill=\"{}\"", fill.unwrap_or("currentColor")));
        if let Some(stroke) = stroke {
            attributes.push_str(&format!(" stroke=\"{stroke}\" stroke-width=\"{}\"", thickness.as_deref().unwrap_or("1")));
        }
    }
    if !followed.is_empty() {
        attributes.push_str(&format!(" data-svg=\"{}\"", followed.join(" ")));
    }
    Ok(format!("<{tag}{attributes}/>"))
}

fn example(tag: &str) -> &'static str {
    match tag {
        "rect" => "Rect(x: 0, y: 0, width: 200, height: 100)",
        "circle" => "Circle(x: 50, y: 50, r: 20)",
        "line" => "Line(from: [0, 80], to: [200, 80])",
        _ => "Path(d: \"M10 80 L50 20 L90 80 Z\")",
    }
}

/// Une mesure écrite : un nombre de 0 à 4 000 (à virgule aussi), ou le nom d'un nombre entier de
/// la page, que la forme suit.
fn written(shape: &Block, parameter: &str, attribute: &str, value: &Value, followed: &mut Vec<String>) -> Result<String, Error> {
    if let Value::Name(name) = value {
        followed.push(format!("{attribute}:{name}"));
        return Ok(format!(" {attribute}=\"{MARK}%{name}{MARK}\""));
    }
    Ok(format!(" {attribute}=\"{}\"", measure(shape, parameter, value)?))
}

/// Un nombre écrit dans le fichier, borné.
fn measure(shape: &Block, parameter: &str, value: &Value) -> Result<String, Error> {
    match value {
        Value::Integer(n) if (0..=u64::from(SIZE_MAX)).contains(n) => Ok(n.to_string()),
        Value::Number { value, unit: None, .. } if (0.0..=f64::from(SIZE_MAX)).contains(value) => Ok(value.to_string()),
        _ => Err(Error { message: format!("« {}({parameter}: …) » attend un nombre de 0 à {SIZE_MAX}, ou le nom d'un nombre entier de la page", shape.name), pos: shape.pos }),
    }
}

/// Une couleur de remplissage ou de trait : celles du style, ou « none ».
fn is_paint(color: &str) -> bool {
    color == "none" || crate::styles::is_color(color)
}

/// Un tracé SVG : des lettres de tracé, des nombres, des espaces et des virgules, rien d'autre ;
/// il commence par M (aller à un point).
fn is_path(d: &str) -> bool {
    d.len() <= PATH_MAX && d.trim_start().starts_with(['M', 'm']) && d.chars().all(|c| "MmLlHhVvCcSsQqTtAaZz0123456789 ,.-".contains(c))
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Vérifie les mesures liées à des valeurs : chacune est un nombre entier de la page.
pub fn check(program: &Program) -> Result<(), Error> {
    let numbers = crate::state::initial(program).unwrap_or_default();
    crate::rules::for_each_block(&program.root, &mut |block| {
        if !SHAPES.contains(&block.name.as_str()) {
            return Ok(());
        }
        for argument in block.arguments.iter().filter(|a| a.name.as_deref() != Some("name")) {
            let names: Vec<&String> = match &argument.value {
                Value::Name(name) => vec![name],
                Value::List(elements) => elements.iter().filter_map(|e| if let Value::Name(name) = e { Some(name) } else { None }).collect(),
                _ => Vec::new(),
            };
            for name in names {
                if !numbers.iter().any(|(known, _)| known == name) {
                    return Err(Error { message: format!("« {}({}: {name}) » : « {name} » n'est pas un nombre de la page ; une mesure est un nombre, ou le nom d'un nombre déclaré dans State", block.name, argument.name.as_deref().unwrap_or("")), pos: argument.pos });
                }
                if crate::state::places(program, name) > 0 {
                    return Err(Error { message: format!("« {}({}: {name}) » : « {name} » a des chiffres après la virgule ; une mesure liée prend un nombre entier", block.name, argument.name.as_deref().unwrap_or("")), pos: argument.pos });
                }
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_drawing_becomes_svg() {
        let source = "Page(state: State(sun: 40, big: 12), children: [ Drawing(label: \"Un paysage, <le soir>\", width: 200, height: 100, children: [ Rect(x: 0, y: 0, width: 200, height: 100, radius: 8, fill: \"#101020\"), Circle(x: 150, y: sun, r: big, fill: \"#E9B44C\", opacity: 0.8), Line(from: [0, 80], to: [200, sun], stroke: \"#8fd3ff\", thickness: 2), Path(d: \"M20 80 L40 60 L60 80 Z\", fill: \"none\", stroke: \"white\") ]) ])";
        let html = crate::flat_view(source, "").unwrap();
        for expected in [
            "<svg class=\"holo-Drawing\" role=\"img\" aria-label=\"Un paysage, &lt;le soir&gt;\" viewBox=\"0 0 200 100\" width=\"200\" height=\"100\">",
            "<rect x=\"0\" y=\"0\" width=\"200\" height=\"100\" rx=\"8\" fill=\"#101020\"/>",
            "<circle cx=\"150\" cy=\"40\" r=\"12\" opacity=\"0.8\" fill=\"#E9B44C\" data-svg=\"cy:sun r:big\"/>",
            "<line x1=\"0\" y1=\"80\" x2=\"200\" y2=\"40\" stroke=\"#8fd3ff\" stroke-width=\"2\" data-svg=\"y2:sun\"/>",
            "<path d=\"M20 80 L40 60 L60 80 Z\" fill=\"none\" stroke=\"white\" stroke-width=\"1\"/>",
            "</svg>",
        ] {
            assert!(html.contains(expected), "manque : {expected}\n{html}");
        }
        // La page part des valeurs du serveur, comme le reste.
        let shown = crate::flat_view_with_data(&source.replace("Page(", "Page(data: Data(from: \"d.json\"), "), "", r#"{"sun": 25}"#).unwrap();
        assert!(shown.contains("cy=\"25\""), "{shown}");
        for (source, message) in [
            ("Page(children: [ Drawing(width: 10, height: 10, children: []) ])", "attend label, width et height"),
            ("Page(children: [ Drawing(label: \"x\", width: 0, height: 10, children: []) ])", "un nombre entier de 1 à 4000"),
            ("Page(children: [ Circle(x: 1, y: 1, r: 1) ])", "se dessine dans un Drawing"),
            ("Page(children: [ Drawing(label: \"x\", width: 10, height: 10, children: [ Circle(x: 1, y: 1) ]) ])", "« Circle » attend x, y, r"),
            ("Page(children: [ Drawing(label: \"x\", width: 10, height: 10, children: [ Circle(x: 1, y: 1, r: 1, fill: \"url(#a)\") ]) ])", "une couleur entre guillemets"),
            ("Page(children: [ Drawing(label: \"x\", width: 10, height: 10, children: [ Path(d: \"<script>\") ]) ])", "un tracé SVG qui commence par M"),
            ("Page(children: [ Drawing(label: \"x\", width: 10, height: 10, children: [ Circle(x: 1, y: 1, r: 1, angle: 3) ]) ])", "n'a pas de paramètre « angle »"),
            ("Page(children: [ Drawing(label: \"x\", width: 10, height: 10, children: [ Circle(x: rien, y: 1, r: 1) ]) ])", "aucune valeur ne s'appelle « rien »"),
            ("Page(state: State(p: 1.5), children: [ Drawing(label: \"x\", width: 10, height: 10, children: [ Circle(x: p, y: 1, r: 1) ]) ])", "une mesure liée prend un nombre entier"),
            ("Page(children: [ Drawing(label: \"x\", width: 10, height: 10, children: [ Line(from: [0], to: [1, 1]) ]) ])", "deux mesures, x et y"),
            ("Page(children: [ Drawing(label: \"x\", width: 10, height: 10, children: [ P(\"t\") ]) ])", "ne se dessine pas"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }
}
