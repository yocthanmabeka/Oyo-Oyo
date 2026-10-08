//! Les graphiques d'un tableau de bord (ADR-087) : une liste à champs, dessinée en barres, en
//! courbe ou en parts, par le moteur, en SVG. Un tableau caché donne les mêmes chiffres au lecteur
//! d'écran. Le graphique suit la liste : des données reçues, une liste calculée, un élément ajouté.
//!
//! ```holo
//! Chart(kind: bars, over: sales, value: amount, label: day, title: "Sales of the week")
//! ```
//!
//! `value` et `label` sont des champs des éléments de la liste : le nombre à dessiner, et son nom.

use crate::holo::{Block, Error, Value};

/// Les sortes de graphiques.
pub const KINDS: &[&str] = &["bars", "line", "pie"];
/// Les réglages d'un graphique.
pub const PARAMS: &[&str] = &["name", "kind", "over", "value", "label", "title", "color"];
/// Les couleurs des parts, dans l'ordre : assez différentes pour se distinguer côte à côte.
const PALETTE: &[&str] = &["#E9B44C", "#8fd3ff", "#9be7a1", "#ff8fa3", "#c3a6ff", "#ffd27f", "#7fdbda", "#f4a261"];
/// La taille du dessin, dans ses unités : il rétrécit avec l'écran.
const WIDTH: f64 = 480.0;
const HEIGHT: f64 = 260.0;

/// Ce qu'il faut pour dessiner un graphique, et le redessiner quand la liste change.
#[derive(Debug, PartialEq)]
pub struct Spec<'a> {
    pub kind: &'a str,
    pub over: &'a str,
    pub value: &'a str,
    pub label: &'a str,
    pub color: &'a str,
}

impl Spec<'_> {
    /// Écrit dans la page, `data-chart="bars|sales|amount|day|"` : la page le rend au moteur pour
    /// redessiner le graphique.
    fn written(&self) -> String {
        format!("{}|{}|{}|{}|{}", self.kind, self.over, self.value, self.label, self.color)
    }

    /// Relu depuis la page ; les noms sont vérifiés à nouveau par `drawing`.
    pub fn read(written: &str) -> Option<Spec<'_>> {
        let mut parts = written.split('|');
        let spec = Spec { kind: parts.next()?, over: parts.next()?, value: parts.next()?, label: parts.next()?, color: parts.next().unwrap_or("") };
        let named = |n: &str| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric());
        (KINDS.contains(&spec.kind) && named(spec.over) && named(spec.value) && named(spec.label) && (spec.color.is_empty() || crate::styles::is_color(spec.color)) && parts.next().is_none()).then_some(spec)
    }
}

/// Le graphique entier : son titre, son dessin et son tableau caché.
pub fn html(block: &Block, classes: &str, name: &str, lists: &crate::lists::Lists) -> Result<String, Error> {
    let error = |message: String, pos| Error { message, pos };
    let example = "Chart(kind: bars, over: sales, value: amount, label: day, title: \"Sales of the week\")";
    let (mut kind, mut over, mut value, mut label, mut title, mut color) = (None, None, None, None, None, "");
    for argument in &block.arguments {
        match (argument.name.as_deref(), &argument.value) {
            (Some("name"), _) => {}
            (Some("kind"), Value::Name(k)) if KINDS.contains(&k.as_str()) => kind = Some(k.as_str()),
            (Some("kind"), _) => return Err(error("« Chart(kind: …) » attend bars (des barres), line (une courbe) ou pie (des parts)".into(), argument.pos)),
            (Some("over"), Value::Name(list)) => over = Some(list.as_str()),
            (Some("value"), Value::Name(field)) => value = Some(field.as_str()),
            (Some("label"), Value::Name(field)) => label = Some(field.as_str()),
            (Some(p @ ("over" | "value" | "label")), _) => return Err(error(format!("« Chart({p}: …) » attend un nom : over la liste, value le champ du nombre, label le champ du nom ; {example}"), argument.pos)),
            (Some("title"), Value::Text(t)) if !t.trim().is_empty() => title = Some(t.as_str()),
            (Some("title"), _) => return Err(error("« Chart(title: …) » dit ce que montre le graphique, pour tous : title: \"Sales of the week\"".into(), argument.pos)),
            (Some("color"), Value::Text(c)) if crate::styles::is_color(c) => color = c.as_str(),
            (Some("color"), _) => return Err(error("« Chart(color: …) » attend une couleur entre guillemets, comme \"#E9B44C\"".into(), argument.pos)),
            (Some(other), _) => return Err(error(format!("« Chart » n'a pas de paramètre « {other} » ; paramètres possibles : {}", PARAMS.join(", ")), argument.pos)),
            (None, _) => return Err(error(format!("chaque paramètre de « Chart » est nommé : {example}"), argument.pos)),
        }
    }
    let (Some(kind), Some(over), Some(value), Some(label), Some(title)) = (kind, over, value, label, title) else {
        return Err(error(format!("« Chart » attend kind, over, value, label et title : {example}"), block.pos));
    };
    let Some((_, elements)) = lists.iter().find(|(known, _)| known == over) else {
        return Err(error(format!("« Chart(over: {over}) » : aucune liste ne s'appelle « {over} » ; déclare-la, state: State({over}: [ Item({label}: \"…\", {value}: 0) ])"), block.pos));
    };
    // Des éléments à champs, et ces deux champs-là, quand la liste en a déjà.
    if let Some(first) = elements.first() {
        let fields = crate::lists::fields(first);
        if fields.is_empty() {
            return Err(error(format!("« Chart(over: {over}) » attend une liste à champs : State({over}: [ Item({label}: \"…\", {value}: 0) ])"), block.pos));
        }
        for field in [value, label] {
            if !fields.iter().any(|(known, _)| known == field) {
                return Err(error(format!("« Chart » : les éléments de « {over} » n'ont pas de champ « {field} »"), block.pos));
            }
        }
    }
    let spec = Spec { kind, over, value, label, color };
    Ok(format!(
        "<figure class=\"{classes}\"{name} data-chart=\"{}\"><figcaption>{}</figcaption>{}</figure>",
        spec.written(),
        escape(title),
        drawing(&spec, elements)
    ))
}

/// Le dessin et le tableau caché, pour ces éléments : ce que la page remplace quand la liste change.
pub fn drawing(spec: &Spec, elements: &[String]) -> String {
    let rows: Vec<(String, String, f64)> = elements
        .iter()
        .map(|element| {
            let fields = crate::lists::fields(element);
            let field = |name: &str| fields.iter().find(|(known, _)| known == name).map(|(_, v)| v.clone()).unwrap_or_default();
            let written = field(spec.value);
            let number = number(&written);
            (field(spec.label), written, number)
        })
        .collect();
    let mut svg = format!("<svg aria-hidden=\"true\" viewBox=\"0 0 {WIDTH} {HEIGHT}\" width=\"{WIDTH}\" height=\"{HEIGHT}\">");
    if rows.is_empty() || rows.iter().all(|(_, _, n)| *n == 0.0) && spec.kind == "pie" {
        svg.push_str(&format!("<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" fill=\"currentColor\" font-size=\"16\">—</text>", WIDTH / 2.0, HEIGHT / 2.0));
    } else {
        match spec.kind {
            "pie" => pie(&rows, &mut svg),
            kind => bars_or_line(&rows, kind, if spec.color.is_empty() { PALETTE[0] } else { spec.color }, &mut svg),
        }
    }
    svg.push_str("</svg>");
    // Un tableau ne se réduit pas à un pixel : c'est le bloc qui l'entoure qui est caché.
    let mut table = format!("<div class=\"holo-hidden\"><table><tr><th scope=\"col\">{}</th><th scope=\"col\">{}</th></tr>", escape(spec.label), escape(spec.value));
    for (label, written, _) in &rows {
        table.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", escape(label), escape(written)));
    }
    table.push_str("</table></div>");
    format!("<div class=\"holo-chart-drawing\">{svg}{table}</div>")
}

/// Des barres, ou une courbe : les noms dessous, les nombres au-dessus.
fn bars_or_line(rows: &[(String, String, f64)], kind: &str, color: &str, svg: &mut String) {
    let (top, bottom, side) = (24.0, 36.0, 12.0);
    let highest = rows.iter().map(|(_, _, n)| *n).fold(0.0_f64, f64::max).max(f64::MIN_POSITIVE);
    let step = (WIDTH - 2.0 * side) / rows.len() as f64;
    let height_of = |n: f64| (HEIGHT - top - bottom) * n / highest;
    svg.push_str(&format!("<line x1=\"{side}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"currentColor\" stroke-opacity=\"0.4\"/>", HEIGHT - bottom, WIDTH - side, HEIGHT - bottom));
    let mut points = Vec::new();
    for (rank, (label, written, n)) in rows.iter().enumerate() {
        let middle = side + step * (rank as f64 + 0.5);
        let y = HEIGHT - bottom - height_of(*n);
        if kind == "bars" {
            let width = step * 0.7;
            svg.push_str(&format!("<rect x=\"{:.1}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{:.1}\" rx=\"3\" fill=\"{color}\"/>", middle - width / 2.0, height_of(*n)));
        } else {
            svg.push_str(&format!("<circle cx=\"{middle:.1}\" cy=\"{y:.1}\" r=\"4\" fill=\"{color}\"/>"));
            points.push(format!("{middle:.1},{y:.1}"));
        }
        svg.push_str(&format!("<text x=\"{middle:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"currentColor\" font-size=\"13\">{}</text>", y - 6.0, escape(&short(written, 8))));
        svg.push_str(&format!("<text x=\"{middle:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"currentColor\" font-size=\"13\">{}</text>", HEIGHT - bottom + 20.0, escape(&short(label, 10))));
    }
    if kind == "line" && points.len() > 1 {
        svg.push_str(&format!("<polyline points=\"{}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2.5\"/>", points.join(" ")));
    }
}

/// Des parts, et leur légende à droite.
fn pie(rows: &[(String, String, f64)], svg: &mut String) {
    let (cx, cy, r) = (130.0, HEIGHT / 2.0, 110.0);
    let total: f64 = rows.iter().map(|(_, _, n)| *n).sum();
    let mut angle = -std::f64::consts::FRAC_PI_2;
    for (rank, (label, written, n)) in rows.iter().enumerate() {
        let color = PALETTE[rank % PALETTE.len()];
        let share = n / total;
        if share >= 0.999_999 {
            svg.push_str(&format!("<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"{color}\"/>"));
        } else if share > 0.0 {
            let end = angle + share * std::f64::consts::TAU;
            let (x1, y1, x2, y2) = (cx + r * angle.cos(), cy + r * angle.sin(), cx + r * end.cos(), cy + r * end.sin());
            let large = u8::from(share > 0.5);
            svg.push_str(&format!("<path d=\"M{cx} {cy} L{x1:.1} {y1:.1} A{r} {r} 0 {large} 1 {x2:.1} {y2:.1} Z\" fill=\"{color}\"/>"));
            angle = end;
        }
        // La légende : au plus douze lignes ; le tableau caché a toutes les parts.
        if rank < 12 {
            let y = 24.0 + rank as f64 * 19.0;
            svg.push_str(&format!("<rect x=\"270\" y=\"{:.1}\" width=\"12\" height=\"12\" rx=\"2\" fill=\"{color}\"/>", y - 10.0));
            svg.push_str(&format!("<text x=\"290\" y=\"{y:.1}\" fill=\"currentColor\" font-size=\"13\">{} : {}</text>", escape(&short(label, 16)), escape(&short(written, 8))));
        }
    }
}

/// Le nombre écrit dans un champ : « 12 », « 12.5 » ou « 12,5 » ; 0 s'il n'y en a pas.
fn number(written: &str) -> f64 {
    written.trim().replace(',', ".").parse::<f64>().ok().filter(|n| n.is_finite() && *n >= 0.0).unwrap_or(0.0)
}

/// Un texte raccourci pour tenir sous une barre : le tableau caché, lui, le garde entier.
fn short(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(max - 1).collect::<String>())
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SALES: &str = "Page(state: State(sales: [ Item(day: \"Lundi\", amount: \"120\"), Item(day: \"Mardi <2>\", amount: \"80,5\"), Item(day: \"Mercredi\", amount: \"200\") ]), children: [ Chart(kind: bars, over: sales, value: amount, label: day, title: \"Les ventes\") ])";

    #[test]
    fn a_chart_is_drawn_and_written_for_screen_readers() {
        let html = crate::flat_view(SALES, "").unwrap();
        for expected in [
            "<figure class=\"holo-Chart\" data-chart=\"bars|sales|amount|day|\"><figcaption>Les ventes</figcaption><div class=\"holo-chart-drawing\"><svg aria-hidden=\"true\" viewBox=\"0 0 480 260\"",
            // La plus haute barre prend toute la hauteur : 260 - 24 - 36 = 200.
            "height=\"200.0\" rx=\"3\" fill=\"#E9B44C\"/>",
            ">Mardi &lt;2&gt;</text>",
            "<div class=\"holo-hidden\"><table><tr><th scope=\"col\">day</th><th scope=\"col\">amount</th></tr><tr><td>Lundi</td><td>120</td></tr><tr><td>Mardi &lt;2&gt;</td><td>80,5</td></tr><tr><td>Mercredi</td><td>200</td></tr></table></div>",
        ] {
            assert!(html.contains(expected), "manque : {expected}\n{html}");
        }
        // Une courbe, et des parts.
        let line = crate::flat_view(&SALES.replace("kind: bars", "kind: line"), "").unwrap();
        assert!(line.contains("<polyline points=\"") && line.contains("<circle cx="), "{line}");
        let pie = crate::flat_view(&SALES.replace("kind: bars", "kind: pie"), "").unwrap();
        assert_eq!(pie.matches("<path d=\"M130").count(), 3, "{pie}");
        assert!(pie.contains(">Lundi : 120</text>"), "{pie}");
        // Redessiné quand la liste change : la page rend le réglage au moteur.
        let after = crate::chart_html(SALES, "sales=[%1Dday%3DJeudi%26amount%3D50]", "bars|sales|amount|day|");
        assert!(after.starts_with("<div class=\"holo-chart-drawing\">") && after.contains("<td>Jeudi</td><td>50</td>"), "{after}");
        assert_eq!(crate::chart_html(SALES, "", "bars|sales|amount|day|<script>"), "");
        assert_eq!(crate::chart_html(SALES, "", "bars|absent|amount|day|"), "");
        // Une liste vide : un tiret, et un tableau sans ligne.
        let empty = crate::flat_view("Page(state: State(sales: []), children: [ Chart(kind: pie, over: sales, value: amount, label: day, title: \"Rien\") ])", "").unwrap();
        assert!(empty.contains(">—</text>"), "{empty}");
        for (source, message) in [
            ("Page(state: State(s: []), children: [ Chart(over: s, value: a, label: b, title: \"t\") ])", "attend kind, over, value, label et title"),
            ("Page(state: State(s: []), children: [ Chart(kind: donut, over: s, value: a, label: b, title: \"t\") ])", "bars (des barres), line (une courbe) ou pie"),
            ("Page(children: [ Chart(kind: bars, over: rien, value: a, label: b, title: \"t\") ])", "aucune liste ne s'appelle « rien »"),
            ("Page(state: State(s: [\"a\"]), children: [ Chart(kind: bars, over: s, value: a, label: b, title: \"t\") ])", "attend une liste à champs"),
            ("Page(state: State(s: [ Item(b: \"x\", n: 1) ]), children: [ Chart(kind: bars, over: s, value: a, label: b, title: \"t\") ])", "n'ont pas de champ « a »"),
            ("Page(state: State(s: []), children: [ Chart(kind: bars, over: s, value: a, label: b, title: \"\") ])", "dit ce que montre le graphique"),
            ("Page(state: State(s: []), children: [ Chart(kind: bars, over: s, value: a, label: b, title: \"t\", color: \"rouge vif\") ])", "attend une couleur"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
        assert_eq!(Spec::read("pie|s|a|b|#fff"), Some(Spec { kind: "pie", over: "s", value: "a", label: "b", color: "#fff" }));
        assert_eq!(Spec::read("pie|s|a|b|#fff|x"), None);
    }
}
