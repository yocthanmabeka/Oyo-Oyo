//! Une grille qui place ses cases (ADR-104) : une case sur plusieurs colonnes ou plusieurs
//! lignes (`columnSpan: 2`, `rowSpan: 2`), et des zones nommées, dessinées avec des mots
//! (`Grid(areas: ["top top", "menu main"])`, puis `area: menu` sur chaque bloc).
//!
//! La place se dit dans les blocs, jamais dans un style (ADR-017). Trois défauts du CSS ne
//! reviennent pas :
//! - une case plus large que la grille crée des colonnes de plus, et la page déborde sur un
//!   téléphone : ici, quand la grille n'a pas assez de colonnes, la case prend toute la ligne ;
//! - des zones qui changent l'ordre : l'œil lit une chose, le clavier et le lecteur d'écran en
//!   suivent une autre (WCAG 1.3.2, 2.4.3) : ici, les blocs s'écrivent dans l'ordre des zones ;
//! - une faute dans un nom de zone, un dessin qui n'est pas un rectangle, des lignes de longueurs
//!   différentes : le CSS les ignore en silence ; ici, chacune est refusée avec sa raison.
//!
//! Sur un téléphone, ou dans une case étroite, les zones passent l'une sous l'autre, dans cet
//! ordre. La grille se mesure elle-même (`container-type`) : c'est la place qu'elle reçoit qui
//! compte, pas l'écran ; et tout est du CSS fabriqué par le moteur, sans JavaScript.

use crate::holo::{Block, Error, Program, Value};

/// La largeur la plus petite d'une colonne de `Grid`, en pixels à 16px le rem (7,5rem), la
/// même que dans la règle de base de `flat.rs`.
const COLUMN_MIN: f64 = 120.0;
/// La largeur d'une grille à zones, au moins, pour que ses zones se rangent côte à côte (30rem).
/// Un téléphone tenu droit est plus étroit : ses zones passent l'une sous l'autre.
const AREAS_MIN: f64 = 480.0;
/// Le plus de colonnes ou de lignes qu'une case prend, et que des zones dessinent.
const MAX: usize = 12;
/// Le plus de lettres d'un nom de zone.
const NAME_MAX: usize = 24;

/// Les réglages qu'un bloc prend dans une grille.
pub const CELL_PARAMS: &[&str] = &["columnSpan", "rowSpan", "area"];

/// Une zone : son nom, et les lignes de la grille entre lesquelles elle se tient (de 1 à …, la
/// fin exclue), comme `grid-row: 2 / 3`.
#[derive(Debug, Clone, PartialEq)]
pub struct Zone {
    pub name: String,
    pub rows: (usize, usize),
    pub columns: (usize, usize),
}

/// Les zones d'une grille, dans l'ordre où on les lit : de gauche à droite, puis de haut en bas.
#[derive(Debug, Clone, PartialEq)]
pub struct Areas {
    pub columns: usize,
    pub zones: Vec<Zone>,
}

/// La case qui enveloppe un enfant de grille : sa classe et son style.
pub struct Cell {
    pub class: String,
    pub style: String,
}

fn error(message: String, pos: crate::holo::Pos) -> Error {
    Error { message, pos }
}

/// L'écart de la grille, en pixels : celui de `gap:`, 16px sans rien écrire.
fn gap(grid: &Block) -> f64 {
    match grid.argument("gap").map(|a| &a.value) {
        Some(Value::Number { value, unit: Some(unit), .. }) if unit == "px" => *value,
        _ => 16.0,
    }
}

/// Les colonnes d'une grille ordinaire, au plus : celles de `columns:`, 2 sans rien écrire.
fn columns(grid: &Block) -> u64 {
    match grid.argument("columns").map(|a| &a.value) {
        Some(Value::Integer(n)) => *n,
        _ => 2,
    }
}

/// Un nom de zone : comme le nom d'une valeur, une minuscule au début, des lettres et des
/// chiffres, les mots joints par une majuscule (ADR-037).
fn is_zone_name(word: &str) -> bool {
    word.len() <= NAME_MAX && word.starts_with(|c: char| c.is_ascii_lowercase()) && word.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Lit et vérifie le dessin des zones : `None` pour une grille sans zones.
pub fn areas(grid: &Block) -> Result<Option<Areas>, Error> {
    let Some(argument) = grid.argument("areas") else { return Ok(None) };
    let pos = argument.pos;
    let example = "Grid(areas: [\"top top\", \"menu main\"], children: [ … ])";
    let Value::List(lines) = &argument.value else {
        return Err(error(format!("« Grid(areas: …) » attend une liste de lignes, une par rangée, les noms des zones séparés par des espaces : {example}"), pos));
    };
    if grid.argument("columns").is_some() {
        return Err(error("« Grid » prend « columns: » ou « areas: », pas les deux : avec des zones, les colonnes sont celles du dessin".into(), pos));
    }
    if lines.is_empty() || lines.len() > MAX {
        return Err(error(format!("« Grid(areas: …) » attend de 1 à {MAX} lignes : {example}"), pos));
    }
    let mut cells: Vec<Vec<Option<String>>> = Vec::new();
    for line in lines {
        let Value::Text(text) = line else {
            return Err(error(format!("« Grid(areas: …) » : chaque ligne s'écrit entre guillemets, les noms des zones séparés par des espaces : {example}"), pos));
        };
        let mut row = Vec::new();
        for word in text.split_whitespace() {
            if word.chars().all(|c| c == '.') {
                row.push(None);
            } else if is_zone_name(word) {
                row.push(Some(word.to_string()));
            } else {
                let advice = if word.contains('_') || word.contains('-') {
                    format!(" ; écris « {} »", crate::state::in_flutter(&word.replace('-', "_")))
                } else {
                    String::new()
                };
                return Err(error(format!("« {word} » : une zone se nomme comme une valeur, une minuscule au début, des lettres et des chiffres sans accent, les mots joints par une majuscule (sideMenu), {NAME_MAX} lettres au plus ; un point « . » laisse une case vide{advice}"), pos));
            }
        }
        if row.is_empty() || row.len() > MAX {
            return Err(error(format!("« Grid(areas: …) » : une ligne a de 1 à {MAX} cases, « {text} » en a {}", row.len()), pos));
        }
        if let Some(first) = cells.first() {
            if first.len() != row.len() {
                return Err(error(format!("« Grid(areas: …) » : chaque ligne a le même nombre de cases ; la première en a {}, « {text} » en a {}", first.len(), row.len()), pos));
            }
        }
        if row.iter().all(Option::is_none) {
            return Err(error(format!("« Grid(areas: …) » : la ligne « {text} » n'a que des cases vides ; retire-la"), pos));
        }
        cells.push(row);
    }
    // Les zones, dans l'ordre où on les lit : la première fois que chacune vient.
    let mut zones: Vec<Zone> = Vec::new();
    for (r, row) in cells.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let Some(name) = cell else { continue };
            if zones.iter().any(|z| &z.name == name) {
                continue;
            }
            // Son rectangle : le plus petit qui contient toutes ses cases ; chacune de ses cases
            // doit porter son nom, sinon ce n'est pas un rectangle.
            let mut rows = (r, r);
            let mut columns = (c, c);
            for (r2, row2) in cells.iter().enumerate() {
                for (c2, cell2) in row2.iter().enumerate() {
                    if cell2.as_ref() == Some(name) {
                        rows = (rows.0.min(r2), rows.1.max(r2));
                        columns = (columns.0.min(c2), columns.1.max(c2));
                    }
                }
            }
            let whole = (rows.0..=rows.1).all(|r2| (columns.0..=columns.1).all(|c2| cells[r2][c2].as_ref() == Some(name)));
            if !whole {
                return Err(error(format!("« Grid(areas: …) » : la zone « {name} » n'est pas un rectangle ; ses cases se touchent et forment un seul bloc de lignes et de colonnes"), pos));
            }
            zones.push(Zone { name: name.clone(), rows: (rows.0 + 1, rows.1 + 2), columns: (columns.0 + 1, columns.1 + 2) });
        }
    }
    if zones.len() < 2 {
        return Err(error(format!("« Grid(areas: …) » dessine au moins deux zones ; pour un seul bloc, pas besoin de zones : {example}"), pos));
    }
    Ok(Some(Areas { columns: cells[0].len(), zones }))
}

/// La largeur où des zones se rangent comme on les a dessinées, en pixels à 16px le rem :
/// chacune garde au moins `COLUMN_MIN`, et la grille au moins `AREAS_MIN`. La plus étroite des
/// zones décide : une zone sur k colonnes de C fait k × (largeur − (C − 1) × écart) / C + (k − 1) × écart.
fn areas_threshold(areas: &Areas, gap: f64) -> f64 {
    let c = areas.columns as f64;
    let narrowest = areas.zones.iter().map(|z| z.columns.1 - z.columns.0).min().unwrap_or(1) as f64;
    let each = c * (COLUMN_MIN - (narrowest - 1.0) * gap) / narrowest + (c - 1.0) * gap;
    each.max(AREAS_MIN)
}

/// La largeur où une case prend `span` colonnes : la grille en a alors au moins autant, chacune
/// d'au moins `COLUMN_MIN`, avec leurs écarts. En dessous, la case prend toute la ligne.
fn span_threshold(span: u64, gap: f64) -> f64 {
    span as f64 * COLUMN_MIN + (span as f64 - 1.0) * gap
}

/// Le seuil écrit dans le nom de la classe : en pixels entiers, un de plus que le compte, pour
/// que la grille ait toujours assez de place quand la règle vaut (les arrondis du navigateur).
fn threshold_class(width: f64) -> u32 {
    width.ceil() as u32 + 1
}

/// La place d'un enfant de grille, s'il en dit une : `None` pour une case ordinaire. Vérifie
/// ses réglages selon la grille qui le contient.
pub fn cell(block: &Block, grid: &Block) -> Result<Option<Cell>, Error> {
    let given: Vec<&crate::holo::Argument> = block.arguments.iter().filter(|a| a.name.as_deref().is_some_and(|n| CELL_PARAMS.contains(&n))).collect();
    let Some(first) = given.first() else { return Ok(None) };
    let word = first.name.as_deref().unwrap_or("");
    if grid.name != "Grid" {
        return Err(error(format!("« {word}: » place une case dans une grille : mets « {} » dans Grid(children: [ … ])", block.name), first.pos));
    }
    // Une fenêtre s'ouvre par-dessus la page, un son sans lecteur ne se voit pas : leur case serait vide.
    if block.name == "Dialog" || (block.name == "Sound" && block.argument("label").is_none()) {
        return Err(error(format!("« {word}: » donne sa place à une case qu'on voit ; « {} » ne prend pas de place dans la page", block.name), first.pos));
    }
    let areas = areas(grid)?;
    let gap = gap(grid);
    if let Some(areas) = areas {
        if let Some(span) = given.iter().find(|a| a.name.as_deref() != Some("area")) {
            return Err(error(format!("« {}: » ne sert pas dans une grille à zones : la zone dit déjà la place de la case (area: …)", span.name.as_deref().unwrap_or("")), span.pos));
        }
        let names = areas.zones.iter().map(|z| z.name.as_str()).collect::<Vec<_>>().join(", ");
        let argument = given[0];
        let Value::Name(name) = &argument.value else {
            return Err(error(format!("« area: » attend le nom d'une zone de la grille, sans guillemets : area: menu ; zones : {names}"), argument.pos));
        };
        let Some(zone) = areas.zones.iter().find(|z| &z.name == name) else {
            return Err(error(format!("« area: {name} » : la grille n'a pas de zone « {name} » ; zones : {names}"), argument.pos));
        };
        let class = format!("holo-cell holo-roomy holo-from-{}", threshold_class(areas_threshold(&areas, gap)));
        let style = format!("--holo-wide-column:{} / {};--holo-wide-row:{} / {};", zone.columns.0, zone.columns.1, zone.rows.0, zone.rows.1);
        return Ok(Some(Cell { class, style }));
    }
    let mut class = "holo-cell".to_string();
    let mut style = String::new();
    let mut wide_row = String::new();
    let mut span_columns = None;
    for argument in &given {
        match (argument.name.as_deref(), &argument.value) {
            (Some("area"), _) => {
                return Err(error("« area: » place un bloc dans une zone : la grille dessine d'abord ses zones, Grid(areas: [\"top top\", \"menu main\"], children: [ P(\"…\", area: menu) ])".into(), argument.pos));
            }
            (Some("columnSpan"), Value::Integer(n)) if (2..=columns(grid)).contains(n) => span_columns = Some(*n),
            (Some("columnSpan"), _) => {
                let most = columns(grid);
                let writing = if grid.argument("columns").is_some() { format!("columns: {most}") } else { format!("columns: {most} sans rien écrire") };
                return Err(error(format!("« columnSpan: » attend un nombre entier de 2 à {most} : les colonnes que la case prend, au plus celles de la grille ({writing})"), argument.pos));
            }
            (Some("rowSpan"), Value::Integer(n)) if (2..=MAX as u64).contains(n) => wide_row = format!("span {n}"),
            (Some("rowSpan"), _) => return Err(error(format!("« rowSpan: » attend un nombre entier de 2 à {MAX} : les lignes que la case prend"), argument.pos)),
            _ => {}
        }
    }
    match span_columns {
        // Trop étroite pour ses colonnes, la case prend toute la ligne, et une seule ligne.
        Some(span) => {
            class.push_str(&format!(" holo-roomy holo-from-{}", threshold_class(span_threshold(span, gap))));
            style.push_str(&format!("--holo-wide-column:span {span};"));
            if !wide_row.is_empty() {
                style.push_str(&format!("--holo-wide-row:{wide_row};"));
            }
        }
        // Plusieurs lignes seulement : rien ne peut déborder, la case les prend toujours.
        None => style.push_str(&format!("--holo-row:{wide_row};")),
    }
    Ok(Some(Cell { class, style }))
}

/// La grille elle-même : `None` si elle ne place aucune case. Sinon, ce qu'elle ajoute à sa
/// classe et à son style. Avec des zones, chaque zone a son bloc, et les blocs viennent dans
/// l'ordre des zones.
pub fn grid(block: &Block) -> Result<Option<(String, String)>, Error> {
    let elements: &[Value] = match block.argument("children").map(|a| &a.value) {
        Some(Value::List(elements)) => elements,
        _ => &[],
    };
    let Some(areas) = areas(block)? else {
        let roomy = elements.iter().any(|e| matches!(e, Value::Block(b) if b.argument("columnSpan").is_some()));
        return Ok(roomy.then(|| (" holo-placed".to_string(), String::new())));
    };
    let names = areas.zones.iter().map(|z| z.name.as_str()).collect::<Vec<_>>().join(", ");
    let mut seen: Vec<(&str, &Block)> = Vec::new();
    for element in elements {
        let Value::Block(child) = element else {
            return Err(error(format!("dans une grille à zones, chaque bloc dit sa zone : une phrase seule s'écrit P(\"…\", area: …) ; zones : {names}"), block.pos));
        };
        let Some(Value::Name(zone)) = child.argument("area").map(|a| &a.value) else {
            return Err(error(format!("dans une grille à zones, chaque bloc dit sa zone : {}(…, area: …) ; zones : {names}", child.name), child.pos));
        };
        if !areas.zones.iter().any(|z| &z.name == zone) {
            let pos = child.argument("area").map_or(child.pos, |a| a.pos);
            return Err(error(format!("« area: {zone} » : la grille n'a pas de zone « {zone} » ; zones : {names}"), pos));
        }
        if seen.iter().any(|(z, _)| z == zone) {
            return Err(error(format!("deux blocs dans la zone « {zone} » : une zone reçoit un seul bloc ; pour en ranger plusieurs, Column(area: {zone}, children: [ … ])"), child.pos));
        }
        seen.push((zone.as_str(), child));
    }
    if let Some(missing) = areas.zones.iter().find(|z| !seen.iter().any(|(s, _)| *s == z.name)) {
        return Err(error(format!("la zone « {} » n'a pas de bloc : pose un bloc avec area: {}, ou retire-la du dessin", missing.name, missing.name), block.pos));
    }
    // L'ordre : celui des zones, de gauche à droite puis de haut en bas. L'œil, le clavier et le
    // lecteur d'écran suivent alors le même chemin, et sur un téléphone les zones s'empilent ainsi.
    for (rank, (zone, child)) in seen.iter().enumerate() {
        let expected = &areas.zones[rank].name;
        if zone != expected {
            return Err(error(
                format!("« {} » (area: {zone}) vient avant la zone « {expected} » : range les blocs dans l'ordre des zones, de gauche à droite puis de haut en bas ({names}) ; l'œil, le clavier et le lecteur d'écran suivent alors le même chemin", child.name),
                child.pos,
            ));
        }
    }
    // Des colonnes de même largeur, et un écart entre colonnes qui ne déborde jamais : sur une grille
    // très étroite, il rétrécit (au plus la largeur divisée par le nombre de colonnes).
    let c = areas.columns;
    let style = format!("grid-template-columns:repeat({c},minmax(0,1fr));column-gap:min(var(--holo-gap,1rem),calc(100% / {c}));");
    Ok(Some((" holo-placed".to_string(), style)))
}

/// La disposition de base des cases, et une règle par seuil employé dans le fichier. Rien si le
/// fichier ne place aucune case.
pub fn css(program: &Program) -> String {
    let mut thresholds: Vec<u32> = Vec::new();
    let mut cells = false;
    let _ = crate::rules::for_each_block(&program.root, &mut |block| {
        if block.name != "Grid" {
            return Ok(());
        }
        let gap = gap(block);
        if let Ok(Some(areas)) = areas(block) {
            cells = true;
            thresholds.push(threshold_class(areas_threshold(&areas, gap)));
        }
        if let Some(Value::List(elements)) = block.argument("children").map(|a| &a.value) {
            for element in elements {
                let Value::Block(child) = element else { continue };
                if let Some(Value::Integer(span)) = child.argument("columnSpan").map(|a| &a.value) {
                    thresholds.push(threshold_class(span_threshold(*span, gap)));
                }
                cells |= child.argument("columnSpan").is_some() || child.argument("rowSpan").is_some();
            }
        }
        Ok(())
    });
    if !cells {
        return String::new();
    }
    thresholds.sort_unstable();
    thresholds.dedup();
    // La case se mesure par sa grille (`container-type`) ; elle remplit sa place, en largeur et en
    // hauteur ; trop étroite, elle prend toute la ligne. Une case dont le contenu est caché (un
    // `If` faux) ne laisse pas de trou.
    let mut css = String::from(
        ":where(.holo-placed){container-type:inline-size;width:100%;box-sizing:border-box}\
:where(.holo-cell){display:flex;flex-direction:column;min-width:0;box-sizing:border-box;grid-row:var(--holo-row,auto)}\
:where(.holo-cell)>*{flex:1 1 auto;margin:0;min-width:0;box-sizing:border-box}:where(.holo-cell)>.holo-If>*{flex:1 1 auto;margin:0}\
:where(.holo-roomy){grid-column:1/-1;grid-row:auto}:where(.holo-cell:has(>.holo-If[hidden]:only-child)){display:none}",
    );
    for n in thresholds {
        css.push_str(&format!("@container (min-width:{}rem){{.holo-from-{n}{{grid-column:var(--holo-wide-column);grid-row:var(--holo-wide-row,auto)}}}}", rem(n)));
    }
    css
}

/// Des pixels écrits en rem (16px = 1rem), qui suivent la taille du texte choisie par le visiteur.
fn rem(px: u32) -> String {
    let value = f64::from(px) / 16.0;
    if value.fract() == 0.0 { format!("{value:.0}") } else { format!("{value}") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::read;

    fn grid_of(source: &str) -> Block {
        let program = read(source).unwrap();
        let mut found = None;
        let _ = crate::rules::for_each_block(&program.root, &mut |b| {
            if b.name == "Grid" && found.is_none() {
                found = Some(b.clone());
            }
            Ok(())
        });
        found.unwrap()
    }

    #[test]
    fn the_areas_are_read_in_reading_order_with_their_rectangles() {
        let grid = grid_of("Page(children: [ Grid(areas: [\"menu top top\", \"menu main .\", \"foot foot foot\"], children: []) ])");
        let areas = areas(&grid).unwrap().unwrap();
        assert_eq!(areas.columns, 3);
        let found: Vec<(&str, (usize, usize), (usize, usize))> = areas.zones.iter().map(|z| (z.name.as_str(), z.rows, z.columns)).collect();
        assert_eq!(found, vec![("menu", (1, 3), (1, 2)), ("top", (1, 2), (2, 4)), ("main", (2, 3), (2, 3)), ("foot", (3, 4), (1, 4))]);
    }

    #[test]
    fn the_thresholds_keep_every_column_and_every_area_wide_enough() {
        // Deux colonnes et un écart de 12px : 120 + 12 + 120 = 252, et un pixel de marge.
        assert_eq!(threshold_class(span_threshold(2, 12.0)), 253);
        assert_eq!(threshold_class(span_threshold(3, 16.0)), 393);
        // Des zones : au moins 480px, et au moins 120px pour la plus étroite.
        let two = areas(&grid_of("Page(children: [ Grid(areas: [\"a b\"], children: []) ])")).unwrap().unwrap();
        assert_eq!(threshold_class(areas_threshold(&two, 16.0)), 481);
        let many = areas(&grid_of("Page(children: [ Grid(areas: [\"a b c d e f\"], children: []) ])")).unwrap().unwrap();
        assert_eq!(threshold_class(areas_threshold(&many, 16.0)), 6 * 120 + 5 * 16 + 1);
        // Une zone sur trois colonnes de douze garde 120px dès 12 × (120 − 2 × 16) / 3 + 11 × 16 = 528.
        let wide = areas(&grid_of("Page(children: [ Grid(areas: [\"a a a b b b b b b b b b\"], children: []) ])")).unwrap().unwrap();
        assert_eq!(threshold_class(areas_threshold(&wide, 16.0)), 529);
        assert_eq!(rem(253), "15.8125");
        assert_eq!(rem(480), "30");
    }

    #[test]
    fn a_cell_takes_several_columns_and_rows() {
        let html = crate::flat_view("Page(children: [ Grid(columns: 3, gap: 12px, children: [ P.big(\"a\", columnSpan: 2, rowSpan: 2), P(\"b\"), P(\"c\", rowSpan: 2), \"d\" ]) ])\n.big { color: gold; }", "").unwrap();
        // La grille se mesure elle-même ; la grande case prend deux colonnes et deux lignes quand
        // la grille a la place (253px), toute la ligne sinon ; la case haute prend toujours ses lignes.
        assert!(html.contains("<div class=\"holo-Grid holo-placed\" style=\"--holo-columns:3;--holo-gap:0.75rem;\"><div class=\"holo-cell holo-roomy holo-from-253\" style=\"--holo-wide-column:span 2;--holo-wide-row:span 2;\"><p class=\"holo-P holo-s-big\">a</p></div><p class=\"holo-P\">b</p><div class=\"holo-cell\" style=\"--holo-row:span 2;\"><p class=\"holo-P\">c</p></div><p class=\"holo-P\">d</p></div>"), "{html}");
        assert!(html.contains(":where(.holo-placed){container-type:inline-size;width:100%;box-sizing:border-box}"), "{html}");
        assert!(html.contains(":where(.holo-roomy){grid-column:1/-1;grid-row:auto}"), "{html}");
        assert!(html.contains("@container (min-width:15.8125rem){.holo-from-253{grid-column:var(--holo-wide-column);grid-row:var(--holo-wide-row,auto)}}</style>"), "{html}");
        // Un bloc qui bouge : la case l'enveloppe, c'est elle que la grille range.
        let moving = crate::flat_view("Page(children: [ Grid(children: [ P(\"a\", columnSpan: 2, enter: Enter(opacity: 0)) ]) ])", "").unwrap();
        assert!(moving.contains("<div class=\"holo-cell holo-roomy holo-from-257\" style=\"--holo-wide-column:span 2;\"><div class=\"holo-animated hm1\"><p class=\"holo-P\">a</p></div></div>"), "{moving}");
        // Une grille qui ne place rien ne change pas, et la page n'a pas une règle de plus.
        let plain = crate::flat_view("Page(children: [ Grid(columns: 3, children: [ P(\"a\") ]) ])", "").unwrap();
        assert!(plain.contains("<div class=\"holo-Grid\" style=\"--holo-columns:3;\"><p class=\"holo-P\">a</p></div>") && !plain.contains("holo-cell") && !plain.contains("@container"), "{plain}");
    }

    #[test]
    fn named_areas_follow_the_reading_order() {
        let page = "Page(state: State(shown: 0), children: [ Grid(areas: [\"top top\", \"menu main\"], children: [ Header(area: top, children: [ Text(\"Le jardin\") ]), Nav(area: menu, children: [ A(\"Accueil\", to: \"a.holo\") ]), If(shown, is: 0, area: main, children: [ P(\"Bonjour\") ]) ]) ])";
        let html = crate::flat_view(page, "").unwrap();
        assert!(html.contains("<div class=\"holo-Grid holo-placed\" style=\"grid-template-columns:repeat(2,minmax(0,1fr));column-gap:min(var(--holo-gap,1rem),calc(100% / 2));\">"), "{html}");
        assert!(html.contains("<div class=\"holo-cell holo-roomy holo-from-481\" style=\"--holo-wide-column:1 / 3;--holo-wide-row:1 / 2;\"><header class=\"holo-Header\">"), "{html}");
        assert!(html.contains("<div class=\"holo-cell holo-roomy holo-from-481\" style=\"--holo-wide-column:1 / 2;--holo-wide-row:2 / 3;\"><nav class=\"holo-Nav\">"), "{html}");
        assert!(html.contains("<div class=\"holo-cell holo-roomy holo-from-481\" style=\"--holo-wide-column:2 / 3;--holo-wide-row:2 / 3;\"><div class=\"holo-If\""), "{html}");
        assert!(html.contains("@container (min-width:30.0625rem){.holo-from-481{"), "{html}");
        // Une case dont le `If` est faux ne laisse pas de trou.
        assert!(html.contains(":where(.holo-cell:has(>.holo-If[hidden]:only-child)){display:none}"), "{html}");
        // Les champs aussi prennent leur place : une case, un choix, une glissière.
        crate::check_page("Page(state: State(a: \"\", b: 0, c: \"\", d: 5), children: [ Grid(columns: 4, children: [ Input(value: a, label: \"A\", columnSpan: 2), Checkbox(value: b, label: \"B\", rowSpan: 2), Choice(value: c, label: \"C\", options: [\"x\", \"y\"], columnSpan: 2), Slider(value: d, label: \"D\", columnSpan: 4) ]) ])").unwrap();
    }

    #[test]
    fn a_grid_refuses_what_would_overflow_or_mislead() {
        let refused = |page: &str| crate::check_page(page).unwrap_err().message;
        let grid = |inside: &str| format!("Page(children: [ {inside} ])");
        assert!(refused(&grid("P(\"a\", columnSpan: 2)")).contains("« columnSpan: » place une case dans une grille : mets « P » dans Grid(children: [ … ])"));
        assert!(refused(&grid("Row(children: [ P(\"a\", area: top) ])")).contains("« area: » place une case dans une grille"));
        assert!(refused(&grid("Grid(children: [ P(\"a\", columnSpan: 3) ])")).contains("de 2 à 2 : les colonnes que la case prend, au plus celles de la grille (columns: 2 sans rien écrire)"));
        assert!(refused(&grid("Grid(columns: 4, children: [ P(\"a\", columnSpan: 1) ])")).contains("de 2 à 4"));
        assert!(refused(&grid("Grid(children: [ P(\"a\", rowSpan: 13) ])")).contains("« rowSpan: » attend un nombre entier de 2 à 12"));
        assert!(refused(&grid("Grid(children: [ P(\"a\", area: top) ])")).contains("la grille dessine d'abord ses zones"));
        // Le dessin des zones.
        let areas = |drawing: &str, children: &str| format!("Page(children: [ Grid(areas: {drawing}, children: [ {children} ]) ])");
        let two = "P(\"t\", area: top), P(\"m\", area: main)";
        assert!(refused(&areas("\"top main\"", two)).contains("attend une liste de lignes"));
        assert!(refused(&grid(&format!("Grid(columns: 2, areas: [\"top main\"], children: [ {two} ])"))).contains("« columns: » ou « areas: », pas les deux"));
        assert!(refused(&areas("[\"top top\", \"main main side\"]", two)).contains("chaque ligne a le même nombre de cases ; la première en a 2, « main main side » en a 3"));
        assert!(refused(&areas("[\"Top main\"]", two)).contains("« Top » : une zone se nomme comme une valeur"));
        assert!(refused(&areas("[\"top_bar main\"]", two)).contains("écris « topBar »"));
        assert!(refused(&areas("[\"top main\", \"main top\"]", two)).contains("la zone « top » n'est pas un rectangle"));
        assert!(refused(&areas("[\"top top\"]", "P(\"t\", area: top)")).contains("au moins deux zones"));
        assert!(refused(&areas("[\". .\", \"top main\"]", two)).contains("la ligne « . . » n'a que des cases vides"));
        assert!(refused(&areas("[\"top main\", 3]", two)).contains("chaque ligne s'écrit entre guillemets"));
        // Les blocs et leurs zones.
        assert!(refused(&areas("[\"top main\"]", "P(\"t\", area: top), P(\"m\")")).contains("chaque bloc dit sa zone : P(…, area: …) ; zones : top, main"));
        assert!(refused(&areas("[\"top main\"]", "P(\"t\", area: top), \"m\"")).contains("une phrase seule s'écrit P(\"…\", area: …)"));
        assert!(refused(&areas("[\"top main\"]", "P(\"t\", area: top), P(\"m\", area: side)")).contains("« area: side » : la grille n'a pas de zone « side » ; zones : top, main"));
        assert!(refused(&areas("[\"top main\"]", "P(\"t\", area: top), P(\"m\", area: top)")).contains("deux blocs dans la zone « top »"));
        assert!(refused(&areas("[\"top main side\"]", two)).contains("la zone « side » n'a pas de bloc"));
        assert!(refused(&areas("[\"top main\"]", "P(\"m\", area: main), P(\"t\", area: top)")).contains("« P » (area: main) vient avant la zone « top » : range les blocs dans l'ordre des zones, de gauche à droite puis de haut en bas (top, main)"));
        assert!(refused(&areas("[\"top main\"]", "P(\"t\", area: top, columnSpan: 2), P(\"m\", area: main)")).contains("« columnSpan: » ne sert pas dans une grille à zones"));
        assert!(refused(&areas("[\"top main\"]", "P(\"t\", area: \"top\"), P(\"m\", area: main)")).contains("dans une grille à zones, chaque bloc dit sa zone"));
        // Ce qui ne prend pas de place dans la page n'a pas de case ; un style ne place rien.
        assert!(refused(&grid("Grid(children: [ Dialog(name: D, columnSpan: 2, children: [ P(\"a\") ]) ])")).contains("« Dialog » ne prend pas de place dans la page"));
        assert!(refused(&grid("Grid(children: [ Sound(name: S, source: \"a.mp3\", rowSpan: 2) ])")).contains("« Sound » ne prend pas de place dans la page"));
        assert!(refused("Page(children: [ P.wide(\"a\") ])\n.wide { grid-column: span 2; }").contains("« grid-column » place une case de grille : la place se dit sur le bloc"));
        assert!(refused("Page(children: [ P.wide(\"a\") ])\n.wide { grid-template-areas: none; }").contains("la place se dit sur le bloc"));
    }
}
