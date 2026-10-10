//! La zone de dessin du visiteur (ADR-115) : `Sketch(value: drawing, label: "Ton dessin")`.
//!
//! ```holo
//! Page(
//!   state: State(drawing: ""),
//!   keep: [drawing],
//!   children: [
//!     Sketch(value: drawing, label: "Draw a cat", export: [png, svg]),
//!   ],
//! )
//! ```
//!
//! Le visiteur dessine au doigt, à la souris, au stylet, ou au clavier ; l'auteur n'écrit jamais de
//! code de dessin. Chaque trait devient une donnée, rangée dans un texte de la page :
//! `#c62828 5 120,30 140,35 160,42|#1a1a1a 2 10,10` : sa couleur, son épaisseur, puis ses points,
//! en unités de la feuille (`width` sur `height`, 400 sur 300 par défaut), en nombres entiers.
//!
//! Le moteur tient les règles, jamais l'auteur ni la page :
//! - un trait arrive brut de la page ; le moteur le range dans la feuille, le lisse, le simplifie
//!   (Ramer-Douglas-Peucker), et ne le garde que s'il tient dans les bornes du dessin ;
//! - un dessin qui entre d'ailleurs (un état relu, `keep`, `visit`, un `Form` reçu par le serveur,
//!   une valeur partagée, des données) est relu strictement : une couleur ou une épaisseur que la
//!   feuille ne propose pas, un point hors de la feuille, un nombre mal écrit, trop de traits ou de
//!   points, un dessin trop lourd, et il est refusé tout entier ;
//! - l'image enregistrée (SVG) est fabriquée par le moteur d'après ces nombres seuls : aucun texte
//!   du visiteur n'y entre, rien ne s'y exécute. Le PNG est cette image, dessinée par le navigateur.

use crate::flat::escape;
use crate::holo::{Argument, Block, Error, Program, Value};
use crate::rules::for_each_block;

/// Les paramètres de `Sketch`.
pub const PARAMS: &[&str] = &["name", "value", "label", "width", "height", "colors", "thickness", "export", "required"];
/// La feuille quand l'auteur n'en dit rien : 400 sur 300 unités.
pub const WIDTH: u32 = 400;
pub const HEIGHT: u32 = 300;
/// Le plus petit et le plus grand côté d'une feuille, en unités.
pub const SIDE_MIN: u32 = 100;
pub const SIDE_MAX: u32 = 2000;
/// Les couleurs proposées quand l'auteur n'en dit rien : noir, rouge, orange, vert, bleu, violet,
/// chacune lisible sur la feuille blanche (un contraste de 3 pour 1 au moins, WCAG 1.4.11).
pub const COLORS: &[&str] = &["#1a1a1a", "#c62828", "#e65100", "#2e7d32", "#1565c0", "#6a1b9a"];
/// Les couleurs qu'une feuille propose, au plus.
pub const COLORS_MAX: usize = 12;
/// Le contraste d'une couleur sur la feuille blanche, au moins (WCAG 1.4.11, les éléments graphiques).
pub const CONTRAST_MIN: f64 = 3.0;
/// Les épaisseurs qu'une feuille propose, au plus, et l'épaisseur la plus grande, en unités.
pub const THICKNESSES_MAX: usize = 5;
pub const THICKNESS_MAX: u32 = 64;
/// Le nom d'une zone de dessin, au plus.
pub const LABEL_MAX: usize = 200;
/// Les zones de dessin d'une page, au plus : chacune pèse jusqu'à `WEIGHT_MAX` dans l'état.
pub const SKETCHES_MAX: usize = 4;
/// Un dessin, au plus : ses traits, et les points d'un trait une fois simplifié.
pub const STROKES_MAX: usize = 200;
pub const STROKE_POINTS_MAX: usize = 500;
/// Le poids d'un dessin, au plus : ses octets tels qu'il voyage dans l'état de la page (un signe
/// qui n'est ni une lettre ni un chiffre y compte pour trois). Un dessin entier tient ainsi dans un
/// message de formulaire (16 Kio) et dans les valeurs partagées d'une page (16 Kio). Il borne aussi
/// ses points : 1 500 au plus (un point coûte au moins huit octets, « ␣1,2 »).
pub const WEIGHT_MAX: usize = 12_000;
/// Les points bruts d'un trait que la page envoie, au plus, avant qu'il soit simplifié.
pub const RAW_MAX: usize = 4000;
/// Les formats d'image qu'on peut enregistrer.
pub const FORMATS: &[&str] = &["png", "svg"];

/// Ce qu'une zone de dessin dit de sa feuille : la valeur où ses traits sont rangés, sa taille,
/// les couleurs et les épaisseurs qu'elle propose. Deux feuilles égales acceptent les mêmes traits.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    pub value: String,
    pub width: u32,
    pub height: u32,
    /// Les couleurs, en `#rrggbb` minuscules, dans l'ordre de l'auteur : la première est prise au départ.
    pub colors: Vec<String>,
    /// Les épaisseurs, de la plus fine à la plus épaisse : celle du milieu est prise au départ.
    pub thickness: Vec<u32>,
    pub png: bool,
    pub svg: bool,
}

/// Un trait : sa couleur, son épaisseur, ses points, dans la feuille.
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub color: String,
    pub thickness: u32,
    pub points: Vec<(u32, u32)>,
}

/// Pourquoi un trait n'est pas gardé.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Refusal {
    /// Aucune zone de dessin ne range ses traits dans cette valeur.
    Unknown,
    /// La feuille montre une valeur partagée : on ne dessine pas dessus, un toucher la change.
    Shared,
    /// Une couleur que la feuille ne propose pas.
    Color,
    /// Une épaisseur que la feuille ne propose pas.
    Thickness,
    /// Des points illisibles : pas des nombres, ou aucun point.
    Unreadable,
    /// Plus de points bruts, ou plus de points une fois simplifié, qu'un trait n'en a.
    TooLong,
    /// Le dessin est plein : trop de traits, de points, ou trop lourd.
    Full,
}

impl Refusal {
    /// Ce que la page dit au visiteur, dans sa langue.
    pub fn said(self, french: bool) -> String {
        let (fr, en) = match self {
            Refusal::Unknown => ("Cette zone de dessin n'existe pas.", "This drawing area does not exist."),
            Refusal::Shared => ("On ne dessine pas sur un dessin partagé.", "A shared drawing cannot be drawn on."),
            Refusal::Color => ("Cette couleur n'est pas proposée ici.", "This colour is not offered here."),
            Refusal::Thickness => ("Cette épaisseur n'est pas proposée ici.", "This thickness is not offered here."),
            Refusal::Unreadable => ("Ce trait est illisible : il n'est pas gardé.", "This stroke cannot be read: it is not kept."),
            Refusal::TooLong => ("Ce trait est trop long : il n'est pas gardé.", "This stroke is too long: it is not kept."),
            Refusal::Full => ("Le dessin est plein : ce trait n'est pas gardé. Annule ou efface pour continuer.", "The drawing is full: this stroke is not kept. Undo or clear to go on."),
        };
        if french { fr.to_string() } else { en.to_string() }
    }
}

/// La page est-elle en français ? (Sans `lang:`, oui.)
pub fn french(program: &Program) -> bool {
    match program.root.argument("lang").map(|a| &a.value) {
        Some(Value::Text(language)) => language.starts_with("fr"),
        _ => true,
    }
}

/// Le poids d'un texte tel qu'il voyage dans l'état : une lettre ou un chiffre pour un octet, tout
/// autre octet pour trois (`%XX`, voir `state::encode`).
pub fn weight(text: &str) -> usize {
    text.bytes().map(|byte| if byte.is_ascii_alphanumeric() { 1 } else { 3 }).sum()
}

/// Les épaisseurs proposées quand l'auteur n'en dit rien : fine, moyenne, épaisse, à l'échelle de
/// la feuille (2, 5 et 12 unités sur une feuille de 400).
fn default_thickness(width: u32, height: u32) -> Vec<u32> {
    let side = f64::from(width.max(height));
    let mut found: Vec<u32> = [200.0, 80.0, 33.0].iter().map(|part| ((side / part).round() as u32).clamp(1, THICKNESS_MAX)).collect();
    found.dedup();
    found
}

/// Une couleur écrite par l'auteur, en `#rrggbb` minuscules : un nom de couleur du style
/// (`"navy"`), `#abc` ou `#aabbcc`. `None` pour le reste (transparente, un dégradé, une variable).
fn canonical_color(written: &str) -> Option<String> {
    if written.starts_with("--") || written == "transparent" || !crate::styles::is_color(written) {
        return None;
    }
    let [r, g, b] = crate::styles::rgb(written, &[])?;
    Some(format!("#{:02x}{:02x}{:02x}", r as u8, g as u8, b as u8))
}

/// Le rouge, le vert et le bleu d'une couleur `#rrggbb`.
fn channels(color: &str) -> [f64; 3] {
    let byte = |at: usize| color.get(at..at + 2).and_then(|hex| u8::from_str_radix(hex, 16).ok()).map_or(0.0, f64::from);
    [byte(1), byte(3), byte(5)]
}

/// Les zones de dessin de la page, chacune avec sa feuille. Une zone mal écrite est laissée de côté :
/// `check` la refuse, avec sa raison.
pub fn sheets(program: &Program) -> Vec<Sheet> {
    let mut found = Vec::new();
    let _ = for_each_block(&program.root, &mut |block| {
        if block.name == "Sketch" {
            if let Ok(sheet) = sheet(block) {
                found.push(sheet);
            }
        }
        Ok(())
    });
    found
}

/// La feuille dont les traits vont dans cette valeur.
pub fn sheet_of(program: &Program, value: &str) -> Option<Sheet> {
    sheets(program).into_iter().find(|sheet| sheet.value == value)
}

/// Cette valeur est-elle un dessin ?
pub fn is_drawing(program: &Program, value: &str) -> bool {
    sheet_of(program, value).is_some()
}

/// Lit la feuille d'une zone de dessin, ou la raison de son refus.
pub fn sheet(block: &Block) -> Result<Sheet, Error> {
    let refused = |message: String, pos| Error { message, pos };
    let example = "Sketch(value: drawing, label: \"Ton dessin\")";
    let mut sheet = Sheet { value: String::new(), width: WIDTH, height: HEIGHT, colors: Vec::new(), thickness: Vec::new(), png: false, svg: false };
    let (mut width, mut height, mut thickness) = (None, None, None);
    for argument in &block.arguments {
        let pos = argument.pos;
        match (argument.name.as_deref(), &argument.value) {
            (None, _) => return Err(refused(format!("chaque paramètre de « Sketch » est nommé : {example}"), pos)),
            (Some("name" | "label" | "required"), _) => {}
            (Some(word), _) if crate::grid::CELL_PARAMS.contains(&word) || word == "sticky" || word == "grow" => {}
            (Some("value"), Value::Name(value)) => sheet.value = value.clone(),
            (Some("value"), _) => return Err(refused(format!("« Sketch(value: …) » attend le nom d'un texte de la page, où vont les traits : {example}, avec state: State(drawing: \"\")"), pos)),
            (Some(side @ ("width" | "height")), Value::Integer(n)) if (u64::from(SIDE_MIN)..=u64::from(SIDE_MAX)).contains(n) => {
                if side == "width" {
                    width = Some(*n as u32);
                } else {
                    height = Some(*n as u32);
                }
            }
            (Some(side @ ("width" | "height")), _) => return Err(refused(format!("« Sketch({side}: …) » attend un nombre entier de {SIDE_MIN} à {SIDE_MAX} : les unités de la feuille, comme pour un Drawing"), pos)),
            (Some("colors"), Value::List(colors)) if (1..=COLORS_MAX).contains(&colors.len()) => {
                for color in colors {
                    let Value::Text(written) = color else {
                        return Err(refused(format!("« Sketch(colors: …) » contient des couleurs entre guillemets : colors: [\"black\", \"#c62828\", \"navy\"]"), pos));
                    };
                    let Some(canonical) = canonical_color(written) else {
                        return Err(refused(format!("« {written} » n'est pas une couleur pleine : un nom de couleur du style (\"navy\"), ou \"#rrggbb\""), pos));
                    };
                    let seen = crate::styles::contrast(channels(&canonical), [255.0, 255.0, 255.0]);
                    if seen < CONTRAST_MIN {
                        return Err(refused(format!("« {written} » se verrait mal sur la feuille blanche : un contraste de {} pour 1, il en faut 3 au moins (WCAG 1.4.11) ; prends une couleur plus foncée", format!("{seen:.1}").replace('.', ",")), pos));
                    }
                    if sheet.colors.contains(&canonical) {
                        return Err(refused(format!("la couleur « {written} » est proposée deux fois"), pos));
                    }
                    sheet.colors.push(canonical);
                }
            }
            (Some("colors"), _) => return Err(refused(format!("« Sketch(colors: …) » attend de 1 à {COLORS_MAX} couleurs : colors: [\"black\", \"#c62828\"]"), pos)),
            (Some("thickness"), Value::List(sizes)) if (1..=THICKNESSES_MAX).contains(&sizes.len()) => {
                let mut taken = Vec::new();
                for size in sizes {
                    match size {
                        Value::Integer(n) if (1..=u64::from(THICKNESS_MAX)).contains(n) && !taken.contains(&(*n as u32)) => taken.push(*n as u32),
                        Value::Integer(n) if (1..=u64::from(THICKNESS_MAX)).contains(n) => return Err(refused(format!("l'épaisseur {n} est proposée deux fois"), pos)),
                        _ => return Err(refused(format!("« Sketch(thickness: …) » attend des nombres entiers de 1 à {THICKNESS_MAX}, en unités de la feuille : thickness: [2, 6, 12]"), pos)),
                    }
                }
                taken.sort_unstable();
                thickness = Some(taken);
            }
            (Some("thickness"), _) => return Err(refused(format!("« Sketch(thickness: …) » attend de 1 à {THICKNESSES_MAX} épaisseurs : thickness: [2, 6, 12]"), pos)),
            (Some("export"), Value::List(formats)) if (1..=FORMATS.len()).contains(&formats.len()) => {
                for format in formats {
                    match format {
                        Value::Name(word) if word == "png" && !sheet.png => sheet.png = true,
                        Value::Name(word) if word == "svg" && !sheet.svg => sheet.svg = true,
                        _ => return Err(refused("« Sketch(export: …) » attend png, svg, ou les deux, chacun une fois : export: [png, svg]".into(), pos)),
                    }
                }
            }
            (Some("export"), _) => return Err(refused("« Sketch(export: …) » attend la liste des images qu'on peut enregistrer : export: [png, svg]".into(), pos)),
            (Some(other), _) => return Err(refused(format!("« Sketch » n'a pas de paramètre « {other} » ; paramètres possibles : {}", PARAMS.join(", ")), pos)),
        }
    }
    match (width, height) {
        (Some(w), Some(h)) => (sheet.width, sheet.height) = (w, h),
        (None, None) => {}
        _ => return Err(refused("« Sketch » attend width et height ensemble : la feuille, comme width: 600, height: 200 ; sans eux, elle fait 400 sur 300".into(), block.pos)),
    }
    if sheet.value.is_empty() {
        return Err(refused(format!("« Sketch » attend « value » : le texte de la page où vont les traits, {example}"), block.pos));
    }
    if sheet.colors.is_empty() {
        sheet.colors = COLORS.iter().map(|c| c.to_string()).collect();
    }
    sheet.thickness = thickness.unwrap_or_else(|| default_thickness(sheet.width, sheet.height));
    Ok(sheet)
}

/// L'épaisseur prise au départ : celle du milieu (la moyenne des trois de la feuille par défaut).
pub fn starting_thickness(sheet: &Sheet) -> u32 {
    sheet.thickness[(sheet.thickness.len() - 1) / 2]
}

/// Un nombre entier écrit sans signe ni zéro devant (`0`, `12`, jamais `012` ni `+3`), borné.
fn integer(written: &str, max: u32) -> Option<u32> {
    if written.is_empty() || written.len() > 4 || !written.bytes().all(|b| b.is_ascii_digit()) || (written.len() > 1 && written.starts_with('0')) {
        return None;
    }
    written.parse::<u32>().ok().filter(|n| *n <= max)
}

/// Lit un dessin strictement, pour cette feuille. `None` s'il ne s'écrit pas exactement comme le
/// moteur l'écrit, ou s'il sort d'une seule de ses bornes : il est alors refusé tout entier.
pub fn strokes(sheet: &Sheet, text: &str) -> Option<Vec<Stroke>> {
    if text.is_empty() {
        return Some(Vec::new());
    }
    if text.len() > WEIGHT_MAX || weight(text) > WEIGHT_MAX {
        return None;
    }
    let mut found = Vec::new();
    for written in text.split('|') {
        if found.len() == STROKES_MAX {
            return None;
        }
        let mut parts = written.split(' ');
        let color = parts.next()?;
        if !sheet.colors.iter().any(|known| known == color) {
            return None;
        }
        let thickness = integer(parts.next()?, THICKNESS_MAX)?;
        if !sheet.thickness.contains(&thickness) {
            return None;
        }
        let mut points = Vec::new();
        for point in parts {
            let (x, y) = point.split_once(',')?;
            points.push((integer(x, sheet.width)?, integer(y, sheet.height)?));
            if points.len() > STROKE_POINTS_MAX {
                return None;
            }
        }
        if points.is_empty() {
            return None;
        }
        found.push(Stroke { color: color.to_string(), thickness, points });
    }
    Some(found)
}

/// Écrit un dessin comme le moteur l'écrit : `#rrggbb épaisseur x,y x,y…`, les traits séparés par `|`.
pub fn write(strokes: &[Stroke]) -> String {
    strokes
        .iter()
        .map(|stroke| {
            let points: Vec<String> = stroke.points.iter().map(|(x, y)| format!("{x},{y}")).collect();
            format!("{} {} {}", stroke.color, stroke.thickness, points.join(" "))
        })
        .collect::<Vec<_>>()
        .join("|")
}

/// Le dessin, s'il est juste pour cette feuille : le même texte ; sinon `None`.
pub fn clean(sheet: &Sheet, text: &str) -> Option<String> {
    let read = strokes(sheet, text)?;
    let written = write(&read);
    (written == text).then_some(written)
}

/// Les textes d'un état, chaque dessin relu strictement : un dessin faux reprend son départ. Le
/// moteur n'écrit donc jamais un dessin faux, d'où qu'il vienne (une règle, des données, un module).
pub fn sanitized(program: &Program, texts: &crate::state::Texts) -> crate::state::Texts {
    let found = sheets(program);
    if found.is_empty() {
        return texts.clone();
    }
    let initial = crate::state::initial_texts(program);
    texts
        .iter()
        .map(|(name, text)| match found.iter().find(|sheet| sheet.value == *name) {
            Some(sheet) if clean(sheet, text).is_none() => {
                let start = initial.iter().find(|(known, _)| known == name).map(|(_, t)| t.clone()).unwrap_or_default();
                (name.clone(), if clean(sheet, &start).is_some() { start } else { String::new() })
            }
            _ => (name.clone(), text.clone()),
        })
        .collect()
}

/// Range un point brut dans la feuille.
fn inside(sheet: &Sheet, (x, y): (f64, f64)) -> (f64, f64) {
    (x.clamp(0.0, f64::from(sheet.width)), y.clamp(0.0, f64::from(sheet.height)))
}

/// Lisse un trait : chaque point intérieur se rapproche de ses deux voisins (le tremblement d'un
/// doigt s'efface) ; le premier et le dernier restent où ils sont.
fn smoothed(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut out = Vec::with_capacity(points.len());
    out.push(points[0]);
    for window in points.windows(3) {
        let [a, b, c] = [window[0], window[1], window[2]];
        out.push(((a.0 + 2.0 * b.0 + c.0) / 4.0, (a.1 + 2.0 * b.1 + c.1) / 4.0));
    }
    out.push(points[points.len() - 1]);
    out
}

/// La distance d'un point au segment de deux autres.
fn distance(point: (f64, f64), from: (f64, f64), to: (f64, f64)) -> f64 {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = dx * dx + dy * dy;
    if length == 0.0 {
        return ((point.0 - from.0).powi(2) + (point.1 - from.1).powi(2)).sqrt();
    }
    let t = (((point.0 - from.0) * dx + (point.1 - from.1) * dy) / length).clamp(0.0, 1.0);
    ((point.0 - from.0 - t * dx).powi(2) + (point.1 - from.1 - t * dy).powi(2)).sqrt()
}

/// Simplifie un trait (Ramer-Douglas-Peucker) : on ne garde que les points qui s'écartent de plus de
/// `tolerance` de la ligne droite entre ceux qu'on garde. Sans récursion : une pile, bornée par le
/// nombre de points.
fn simplified(points: &[(f64, f64)], tolerance: f64) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut kept = vec![false; points.len()];
    kept[0] = true;
    kept[points.len() - 1] = true;
    let mut pending = vec![(0, points.len() - 1)];
    while let Some((first, last)) = pending.pop() {
        let mut farthest = (0.0, first);
        for (index, point) in points.iter().enumerate().take(last).skip(first + 1) {
            let away = distance(*point, points[first], points[last]);
            if away > farthest.0 {
                farthest = (away, index);
            }
        }
        if farthest.0 > tolerance {
            kept[farthest.1] = true;
            pending.push((first, farthest.1));
            pending.push((farthest.1, last));
        }
    }
    points.iter().zip(kept).filter(|(_, keep)| *keep).map(|(point, _)| *point).collect()
}

/// L'écart permis par la simplification : une unité sur une feuille de 400, davantage sur une plus grande.
fn tolerance(sheet: &Sheet) -> f64 {
    (f64::from(sheet.width.max(sheet.height)) / 400.0).max(1.0)
}

/// Un trait arrivé de la page : ses points bruts, en unités de la feuille (`"12.5 30 14 31.25 …"`,
/// x puis y). Le moteur les range dans la feuille, les lisse, les simplifie, les arrondit, et rend le
/// trait tel qu'il le gardera ; ou la raison de son refus.
pub fn stroke_from(sheet: &Sheet, color: &str, thickness: u32, raw: &str) -> Result<Stroke, Refusal> {
    if !sheet.colors.iter().any(|known| known == color) {
        return Err(Refusal::Color);
    }
    if !sheet.thickness.contains(&thickness) {
        return Err(Refusal::Thickness);
    }
    let mut numbers = Vec::new();
    for written in raw.split_ascii_whitespace() {
        if numbers.len() == 2 * RAW_MAX {
            return Err(Refusal::TooLong);
        }
        let number: f64 = written.parse().map_err(|_| Refusal::Unreadable)?;
        if !number.is_finite() {
            return Err(Refusal::Unreadable);
        }
        numbers.push(number);
    }
    if numbers.is_empty() || numbers.len() % 2 != 0 {
        return Err(Refusal::Unreadable);
    }
    let placed: Vec<(f64, f64)> = numbers.chunks(2).map(|pair| inside(sheet, (pair[0], pair[1]))).collect();
    let simple = simplified(&smoothed(&placed), tolerance(sheet));
    let mut points: Vec<(u32, u32)> = Vec::with_capacity(simple.len());
    for (x, y) in simple {
        let point = (x.round() as u32, y.round() as u32);
        if points.last() != Some(&point) {
            points.push(point);
        }
    }
    if points.len() > STROKE_POINTS_MAX {
        return Err(Refusal::TooLong);
    }
    Ok(Stroke { color: color.to_string(), thickness, points })
}

/// Ajoute un trait arrivé de la page au dessin `text` : le nouveau dessin, ou la raison du refus.
pub fn add_stroke(sheet: &Sheet, text: &str, color: &str, thickness: u32, raw: &str) -> Result<String, Refusal> {
    let stroke = stroke_from(sheet, color, thickness, raw)?;
    let mut all = strokes(sheet, text).unwrap_or_default();
    all.push(stroke);
    let written = write(&all);
    // Le dessin entier, relu comme s'il venait d'ailleurs : ses bornes sont celles de tout dessin.
    strokes(sheet, &written).map(|_| written).ok_or(Refusal::Full)
}

/// Un nombre de la feuille, écrit sans zéro inutile : `12`, `12.5`.
fn number(n: f64) -> String {
    let rounded = (n * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 { format!("{}", rounded as i64) } else { format!("{rounded:.1}") }
}

/// Le tracé d'un trait de deux points ou plus : une courbe qui passe par le milieu de chaque segment
/// (des quadratiques), pour qu'un trait simplifié reste rond.
fn path(points: &[(u32, u32)]) -> String {
    let p = |i: usize| (f64::from(points[i].0), f64::from(points[i].1));
    let mut d = format!("M{} {}", points[0].0, points[0].1);
    if points.len() == 2 {
        d.push_str(&format!("L{} {}", points[1].0, points[1].1));
        return d;
    }
    for i in 1..points.len() - 1 {
        let (control, next) = (p(i), p(i + 1));
        let middle = ((control.0 + next.0) / 2.0, (control.1 + next.1) / 2.0);
        d.push_str(&format!("Q{} {} {} {}", number(control.0), number(control.1), number(middle.0), number(middle.1)));
    }
    let end = points[points.len() - 1];
    d.push_str(&format!("L{} {}", end.0, end.1));
    d
}

/// Les traits d'un dessin juste, en SVG : un point seul est un rond, les autres des tracés aux bouts
/// arrondis. Chaque nombre et chaque couleur viennent du dessin relu : rien d'autre n'y entre.
pub fn strokes_svg(strokes: &[Stroke]) -> String {
    let mut output = String::new();
    for stroke in strokes {
        if stroke.points.len() == 1 {
            let (x, y) = stroke.points[0];
            output.push_str(&format!("<circle cx=\"{x}\" cy=\"{y}\" r=\"{}\" fill=\"{}\"/>", number(f64::from(stroke.thickness) / 2.0), stroke.color));
        } else {
            output.push_str(&format!(
                "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                path(&stroke.points),
                stroke.color,
                stroke.thickness
            ));
        }
    }
    output
}

/// La teinte, la saturation et la lumière d'une couleur (de 0 à 360, de 0 à 1, de 0 à 1).
fn hsl([r, g, b]: [f64; 3]) -> (f64, f64, f64) {
    let (r, g, b) = (r / 255.0, g / 255.0, b / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let light = (max + min) / 2.0;
    if max == min {
        return (0.0, 0.0, light);
    }
    let delta = max - min;
    let saturation = if light > 0.5 { delta / (2.0 - max - min) } else { delta / (max + min) };
    let hue = if max == r {
        ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    } * 60.0;
    (hue, saturation, light)
}

/// Le nom d'une couleur, tel que le lecteur d'écran le dit : sa teinte, foncée ou claire. Calculé
/// d'après la couleur seule, dans la langue de la page.
fn color_name(color: &str, french: bool) -> String {
    let (hue, saturation, light) = hsl(channels(color));
    let (fr, en) = if light < 0.12 {
        ("noir", "black")
    } else if light > 0.92 {
        ("blanc", "white")
    } else if saturation < 0.15 {
        ("gris", "grey")
    } else if (10.0..45.0).contains(&hue) && light < 0.3 {
        ("marron", "brown")
    } else {
        match hue {
            h if !(15.0..345.0).contains(&h) => ("rouge", "red"),
            h if h < 40.0 => ("orange", "orange"),
            h if h < 65.0 => ("jaune", "yellow"),
            h if h < 165.0 => ("vert", "green"),
            h if h < 195.0 => ("turquoise", "turquoise"),
            h if h < 255.0 => ("bleu", "blue"),
            h if h < 290.0 => ("violet", "purple"),
            _ => ("rose", "pink"),
        }
    };
    let (dark, pale) = (light < 0.3 && !matches!(fr, "noir" | "blanc" | "marron"), light > 0.7 && !matches!(fr, "noir" | "blanc"));
    match (french, dark, pale) {
        (true, true, _) => format!("{fr} foncé"),
        (true, _, true) => format!("{fr} clair"),
        (true, _, _) => fr.to_string(),
        (false, true, _) => format!("dark {en}"),
        (false, _, true) => format!("light {en}"),
        (false, _, _) => en.to_string(),
    }
}

/// Les noms des couleurs d'une feuille, chacun une fois : deux bleus deviennent « bleu » et « bleu 2 ».
pub fn color_names(sheet: &Sheet, french: bool) -> Vec<String> {
    let bases: Vec<String> = sheet.colors.iter().map(|color| color_name(color, french)).collect();
    bases
        .iter()
        .enumerate()
        .map(|(rank, base)| match bases[..rank].iter().filter(|before| *before == base).count() {
            0 => base.clone(),
            same => format!("{base} {}", same + 1),
        })
        .collect()
}

/// Les noms des épaisseurs d'une feuille, de la plus fine à la plus épaisse.
pub fn thickness_names(sheet: &Sheet, french: bool) -> Vec<&'static str> {
    let (fr, en): (&[&str], &[&str]) = match sheet.thickness.len() {
        1 => (&["moyen"], &["medium"]),
        2 => (&["fin", "épais"], &["thin", "thick"]),
        3 => (&["fin", "moyen", "épais"], &["thin", "medium", "thick"]),
        4 => (&["très fin", "fin", "épais", "très épais"], &["very thin", "thin", "thick", "very thick"]),
        _ => (&["très fin", "fin", "moyen", "épais", "très épais"], &["very thin", "thin", "medium", "thick", "very thick"]),
    };
    if french { fr.to_vec() } else { en.to_vec() }
}

/// Où sont les traits sur la feuille : en haut à gauche, au centre… ; ou sur toute la feuille.
fn place(sheet: &Sheet, strokes: &[Stroke], french: bool) -> &'static str {
    let points = strokes.iter().flat_map(|s| s.points.iter());
    let (mut left, mut top, mut right, mut bottom) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y) in points {
        (left, top, right, bottom) = (left.min(*x), top.min(*y), right.max(*x), bottom.max(*y));
    }
    let (width, height) = (f64::from(sheet.width), f64::from(sheet.height));
    if f64::from(right - left) >= width * 2.0 / 3.0 && f64::from(bottom - top) >= height * 2.0 / 3.0 {
        return if french { "sur toute la feuille" } else { "across the whole sheet" };
    }
    let third = |middle: f64, side: f64| if middle < side / 3.0 { 0 } else if middle > side * 2.0 / 3.0 { 2 } else { 1 };
    let column = third(f64::from(left + right) / 2.0, width);
    let row = third(f64::from(top + bottom) / 2.0, height);
    let fr = [["en haut à gauche", "en haut", "en haut à droite"], ["à gauche", "au centre", "à droite"], ["en bas à gauche", "en bas", "en bas à droite"]];
    let en = [["at the top left", "at the top", "at the top right"], ["on the left", "in the middle", "on the right"], ["at the bottom left", "at the bottom", "at the bottom right"]];
    if french { fr[row][column] } else { en[row][column] }
}

/// Ce qui a été dessiné, dit avec des mots, d'après les traits seuls : leur nombre, leurs couleurs,
/// leur place. « 3 traits (2 en noir, 1 en rouge), en haut à gauche. » Le lecteur d'écran le dit, et
/// l'image enregistrée le garde (`desc`).
pub fn described(sheet: &Sheet, strokes: &[Stroke], french: bool) -> String {
    if strokes.is_empty() {
        return if french { "Rien n'est encore dessiné.".into() } else { "Nothing drawn yet.".into() };
    }
    let names = color_names(sheet, french);
    let mut counts: Vec<(String, usize)> = Vec::new();
    for stroke in strokes {
        let name = sheet.colors.iter().position(|c| *c == stroke.color).map_or_else(String::new, |at| names[at].clone());
        match counts.iter_mut().find(|(known, _)| *known == name) {
            Some((_, count)) => *count += 1,
            None => counts.push((name, 1)),
        }
    }
    let n = strokes.len();
    let (word, preposition) = match (french, n) {
        (true, 1) => ("trait", "en"),
        (true, _) => ("traits", "en"),
        (false, 1) => ("stroke", "in"),
        (false, _) => ("strokes", "in"),
    };
    let colors = if counts.len() == 1 {
        format!(" {preposition} {}", counts[0].0)
    } else {
        format!(" ({})", counts.iter().map(|(name, count)| format!("{count} {preposition} {name}")).collect::<Vec<_>>().join(", "))
    };
    format!("{n} {word}{colors}, {}.", place(sheet, strokes, french))
}

/// L'image enregistrée, en SVG : la feuille blanche, puis les traits. Son titre est le nom que
/// l'auteur a donné à la zone ; sa description, celle que le moteur dit. Aucun texte du visiteur,
/// aucun script, aucun lien : seulement des formes, des nombres et des couleurs de la feuille.
pub fn image_svg(sheet: &Sheet, strokes: &[Stroke], label: &str, french: bool) -> String {
    let (w, h) = (sheet.width, sheet.height);
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\"><title>{}</title><desc>{}</desc><rect width=\"{w}\" height=\"{h}\" fill=\"#ffffff\"/>{}</svg>",
        escape(label),
        escape(&described(sheet, strokes, french)),
        strokes_svg(strokes)
    )
}

thread_local! {
    /// La page en cours de fabrication : ses valeurs partagées (ADR-079), et les feuilles de ses zones
    /// de dessin. Une zone qui montre une valeur partagée n'a pas d'outils de dessin : un toucher la
    /// change. Ses traits de départ sont relus avec sa feuille (`flat::fill_marks`).
    static PAGE: std::cell::RefCell<(Vec<String>, Vec<Sheet>)> = const { std::cell::RefCell::new((Vec::new(), Vec::new())) };
}

/// Retient ce qu'il faut de la page qu'on fabrique (`flat::raw_site_html`).
pub fn set_page(program: &Program) {
    PAGE.with(|page| *page.borrow_mut() = (program.shared.clone(), sheets(program)));
}

/// Le nom d'une zone de dessin, tel que l'auteur l'a écrit.
fn label_of(block: &Block) -> &str {
    match block.argument("label").map(|a| &a.value) {
        Some(Value::Text(label)) => label,
        _ => "",
    }
}

/// La zone de dessin dans la page. Sans JavaScript, la feuille montre le dessin (une image, avec son
/// nom et sa description), et les outils restent cachés. Avec JavaScript, `web/sketch.js` en fait
/// une zone où l'on dessine, au doigt, à la souris, au stylet ou au clavier, et montre les outils.
/// Les traits et la description de départ sont posés par `flat::fill_marks` (`~` et `+`).
pub fn html(block: &Block, classes: &str, name: &str, french: bool) -> Result<String, Error> {
    let sheet = sheet(block)?;
    let shared = PAGE.with(|page| page.borrow().0.contains(&sheet.value));
    let label = escape(label_of(block));
    let value = escape(&sheet.value);
    let (w, h) = (sheet.width, sheet.height);
    let id = format!("holo-sketch-{value}");
    let say = |fr: &str, en: &str| if french { fr.to_string() } else { en.to_string() };
    let mut tools = String::new();
    if !shared {
        // Les couleurs et les épaisseurs : de vrais boutons ronds, qu'on choisit au doigt, à la souris,
        // ou au clavier (Tab entre dans le groupe, les flèches choisissent).
        if sheet.colors.len() > 1 {
            tools.push_str(&format!("<div class=\"holo-sketch-choices\" role=\"radiogroup\" aria-label=\"{}\">", say("Couleur", "Colour")));
            for (rank, (color, name)) in sheet.colors.iter().zip(color_names(&sheet, french)).enumerate() {
                let checked = if rank == 0 { " checked" } else { "" };
                tools.push_str(&format!(
                    "<label class=\"holo-sketch-choice\"><input type=\"radio\" name=\"{id}-color\" value=\"{color}\" data-sketch-color{checked}><span class=\"holo-sketch-swatch\" style=\"background:{color}\"></span><span class=\"holo-hidden\">{}</span></label>",
                    escape(&name)
                ));
            }
            tools.push_str("</div>");
        }
        if sheet.thickness.len() > 1 {
            let start = starting_thickness(&sheet);
            let biggest = f64::from(*sheet.thickness.last().unwrap_or(&1));
            tools.push_str(&format!("<div class=\"holo-sketch-choices\" role=\"radiogroup\" aria-label=\"{}\">", say("Épaisseur", "Thickness")));
            for (size, name) in sheet.thickness.iter().zip(thickness_names(&sheet, french)) {
                let checked = if *size == start { " checked" } else { "" };
                // L'aperçu : un trait de 2 à 16 pixels, à l'échelle des épaisseurs de la feuille.
                let shown = (2.0 + 14.0 * f64::from(*size) / biggest).round();
                tools.push_str(&format!(
                    "<label class=\"holo-sketch-choice\"><input type=\"radio\" name=\"{id}-size\" value=\"{size}\" data-sketch-size{checked}><span class=\"holo-sketch-line\" style=\"height:{shown}px\"></span><span class=\"holo-hidden\">{name}</span></label>"
                ));
            }
            tools.push_str("</div>");
        }
        tools.push_str(&format!("<button type=\"button\" class=\"holo-sketch-do\" data-sketch-do=\"undo\">{}</button>", say("Annuler", "Undo")));
        tools.push_str(&format!("<button type=\"button\" class=\"holo-sketch-do\" data-sketch-do=\"clear\">{}</button>", say("Effacer", "Clear")));
    }
    for (wanted, format) in [(sheet.svg, "svg"), (sheet.png, "png")] {
        if wanted {
            tools.push_str(&format!("<button type=\"button\" class=\"holo-sketch-do\" data-sketch-do=\"{format}\">{}</button>", say(&format!("Enregistrer en {}", format.to_uppercase()), &format!("Save as {}", format.to_uppercase()))));
        }
    }
    let how = if shared {
        say("Ce dessin est partagé : chacun le voit changer en direct.", "This drawing is shared: everyone sees it change live.")
    } else {
        say(
            "Dessine au doigt, à la souris ou au stylet. Au clavier : les flèches déplacent la pointe, Entrée ou Espace pose puis lève le crayon, Échap oublie le trait en cours, Ctrl+Z annule.",
            "Draw with a finger, a mouse or a pen. With the keyboard: the arrows move the tip, Enter or Space puts the pen down then lifts it, Escape forgets the current stroke, Ctrl+Z undoes.",
        )
    };
    let without_script = if shared {
        say("Ce dessin se voit sans JavaScript ; il change en direct avec JavaScript.", "This drawing shows without JavaScript; it changes live with JavaScript.")
    } else {
        say("Dessiner demande JavaScript ; le dessin se voit quand même.", "Drawing needs JavaScript; the drawing still shows.")
    };
    // Dans un formulaire, la zone est un groupe de champs : ses erreurs se posent après elle (ADR-068).
    let group = format!(" data-group=\"{value}\"");
    let mark = crate::flat::MARK;
    Ok(format!(
        "<fieldset class=\"{classes}\"{name} data-sketch=\"{value}\"{}{group} data-start-color=\"{}\" data-start-size=\"{}\"><legend>{label}</legend>\
<svg class=\"holo-sketch-sheet\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" style=\"width:min(100%,calc(70vh * {w} / {h}))\" role=\"img\" aria-label=\"{label}\" aria-describedby=\"{id}-said\" data-sketch-sheet>\
<g class=\"holo-sketch-strokes\">{mark}~{value}{mark}</g></svg>\
<p class=\"holo-sketch-said\" id=\"{id}-said\" role=\"status\">{mark}+{value}{mark}</p>\
<p class=\"holo-sketch-how\" id=\"{id}-how\">{how}</p>\
<div class=\"holo-sketch-tools\" hidden>{tools}</div>\
<noscript><p class=\"holo-sketch-note\">{without_script}</p></noscript></fieldset>",
        if shared { " data-sketch-shared" } else { "" },
        sheet.colors[0],
        starting_thickness(&sheet),
    ))
}

/// Les traits (`~`) ou la description (`+`) de départ de la zone de dessin `value`, pour
/// `flat::fill_marks` : d'après son texte de départ, relu avec sa feuille. Une feuille vide s'il
/// n'est pas juste.
pub fn shown(value: &str, text: &str, mark: char, french: bool) -> String {
    let Some(sheet) = PAGE.with(|page| page.borrow().1.iter().find(|sheet| sheet.value == value).cloned()) else { return String::new() };
    let read = strokes(&sheet, text).unwrap_or_default();
    if mark == '~' { strokes_svg(&read) } else { described(&sheet, &read, french) }
}

/// Les textes d'une page où une valeur est montrée : `{drawing}`, `{drawing:upper}`.
fn shows(text: &str, value: &str) -> bool {
    text.contains(&format!("{{{value}}}")) || text.contains(&format!("{{{value}:"))
}

/// Vérifie les zones de dessin d'une page. Chaque refus dit pourquoi.
pub fn check(program: &Program) -> Result<(), Error> {
    let mut found: Vec<(Sheet, crate::holo::Pos)> = Vec::new();
    let texts = crate::state::initial_texts(program);
    let in_lines = crate::lists::in_model(program);
    let forms = forms_of(program);
    for_each_block(&program.root, &mut |block| {
        if block.name != "Sketch" {
            return Ok(());
        }
        let refused = |message: String| Err(Error { message, pos: block.pos });
        if program.root.name != "Page" {
            return refused("une zone de dessin se pose dans une page : Page(state: State(drawing: \"\"), children: [ Sketch(value: drawing, label: \"Ton dessin\") ])".into());
        }
        if in_lines.contains(&(block as *const Block)) {
            return refused("une zone de dessin ne se répète pas dans les lignes d'une liste : chaque zone range ses traits dans un texte de la page".into());
        }
        let sheet = sheet(block)?;
        let label = label_of(block);
        if label.trim().is_empty() {
            return refused("« Sketch » attend « label » : le nom de la zone, que le lecteur d'écran dit et que l'image enregistrée garde, label: \"Ton dessin\"".into());
        }
        if label.chars().count() > LABEL_MAX || label.contains('\n') || label.contains('{') {
            return refused(format!("« Sketch(label: …) » est un texte fixe d'une ligne, de {LABEL_MAX} caractères au plus, sans valeur « {{…}} »"));
        }
        if let Some(argument) = block.argument("required") {
            if !matches!(argument.value, Value::Bool(_)) {
                return Err(Error { message: "« Sketch(required: …) » attend true ou false".into(), pos: argument.pos });
            }
            if !forms.contains(&(block as *const Block)) {
                return Err(Error { message: "« required: true » se vérifie à l'envoi d'un formulaire : mets la zone de dessin dans un Form(children: [ … ])".into(), pos: argument.pos });
            }
        }
        let value = &sheet.value;
        let Some((_, start)) = texts.iter().find(|(known, _)| known == value) else {
            return refused(format!("« Sketch(value: {value}) » : les traits vont dans un texte de la page ; déclare-le vide, state: State({value}: \"\")"));
        };
        if program.address.contains(value) || crate::account::GIVEN.contains(&value.as_str()) || value == crate::dates::TODAY || value == crate::hours::NOW {
            return refused(format!("« {value} » est donné à la page (son adresse, le compte, la date) : les traits vont dans un texte à elle, state: State(drawing: \"\")"));
        }
        if clean(&sheet, start).is_none() {
            return refused(format!("« {value} » commence par un texte qui n'est pas un dessin de cette feuille : déclare-le vide, state: State({value}: \"\")"));
        }
        if found.iter().any(|(known, _)| known.value == *value) {
            return refused(format!("« {value} » est déjà la valeur d'une autre zone de dessin : chaque zone range ses traits dans son propre texte"));
        }
        found.push((sheet, block.pos));
        Ok(())
    })?;
    if found.is_empty() {
        return Ok(());
    }
    if let Some((_, pos)) = found.get(SKETCHES_MAX) {
        return Err(Error { message: format!("une page a {SKETCHES_MAX} zones de dessin au plus : chacune peut peser jusqu'à {WEIGHT_MAX} octets"), pos: *pos });
    }
    let drawing = |name: &str| found.iter().find(|(sheet, _)| sheet.value == name).map(|(sheet, _)| sheet);
    let shared: Vec<&(Sheet, crate::holo::Pos)> = found.iter().filter(|(sheet, _)| program.shared.contains(&sheet.value)).collect();
    if let Some((_, pos)) = shared.get(1) {
        return Err(Error { message: "une page partage un seul dessin : chacun pèse jusqu'à 12 000 octets, et toutes les valeurs partagées tiennent en 16 Kio".into(), pos: *pos });
    }
    // Ce qui changerait un dessin autrement qu'en dessinant, ou qui le montrerait comme un texte.
    for_each_block(&program.root, &mut |block| {
        let refused = |message: String, pos| Err(Error { message, pos });
        match block.name.as_str() {
            "Input" | "Checkbox" | "Choice" | "Slider" | "Stopwatch" => {
                if let Some(Argument { value: Value::Name(name), pos, .. }) = block.argument("value") {
                    if drawing(name).is_some() {
                        return refused(format!("« {}(value: {name}) » : « {name} » est un dessin ; il change dans sa zone de dessin, Sketch(value: {name})", block.name), *pos);
                    }
                }
            }
            "Device" => {
                if let Some(Argument { value: Value::Name(name), pos, .. }) = block.argument("value") {
                    if drawing(name).is_some() {
                        return refused(format!("« Device(value: {name}) » : « {name} » est un dessin ; un appareil n'y écrit pas"), *pos);
                    }
                }
            }
            "Transfer" => {
                if let Some(Argument { value: Value::List(values), pos, .. }) = block.argument("values") {
                    if let Some(name) = values.iter().find_map(|v| if let Value::Name(n) = v { drawing(n).map(|_| n) } else { None }) {
                        return refused(format!("« Transfer(values: …) » : « {name} » est un dessin ; il s'enregistre en image, Sketch(value: {name}, export: [svg, png])"), *pos);
                    }
                }
            }
            "Module" => {
                if let Some(Argument { value, pos, .. }) = block.argument("output") {
                    let names: Vec<&String> = match value {
                        Value::Name(n) => vec![n],
                        Value::List(vs) => vs.iter().filter_map(|v| if let Value::Name(n) = v { Some(n) } else { None }).collect(),
                        _ => Vec::new(),
                    };
                    if let Some(name) = names.into_iter().find(|n| drawing(n).is_some()) {
                        return refused(format!("« Module(output: …) » : « {name} » est un dessin ; seul le visiteur dessine, un module n'y écrit pas"), *pos);
                    }
                }
            }
            "Split" => {
                if let Some(Argument { value: Value::Name(name), pos, .. }) = block.argument("from") {
                    if drawing(name).is_some() {
                        return refused(format!("« Split(from: {name}) » : « {name} » est un dessin, pas un texte à découper"), *pos);
                    }
                }
            }
            _ => {}
        }
        // Une règle : un dessin se vide (`drawing.set("")`), ou prend un autre dessin de la même feuille.
        for request in crate::state::requests_of(block) {
            let Some((target, verb)) = request.name.split_once('.') else { continue };
            let given = match request.arguments.as_slice() {
                [Argument { name: None, value, .. }] => Some(value),
                _ => None,
            };
            match (drawing(target), verb, given) {
                (Some(_), "set", Some(Value::Text(text))) if text.is_empty() => {}
                (Some(sheet), "set", Some(Value::Name(other))) => match drawing(other) {
                    Some(source) if source.width == sheet.width && source.height == sheet.height && source.colors == sheet.colors && source.thickness == sheet.thickness => {}
                    Some(_) => return refused(format!("« {target}.set({other}) » : les deux zones de dessin n'ont pas la même feuille (taille, couleurs, épaisseurs) ; donne-leur la même, pour que chaque trait y soit permis"), request.pos),
                    None => return refused(format!("« {target}.set({other}) » : « {target} » est un dessin ; il prend un autre dessin, ou se vide, {target}.set(\"\")"), request.pos),
                },
                (Some(_), _, _) => return refused(format!("« {target}.{verb} » : « {target} » est un dessin ; une règle le vide, {target}.set(\"\"), ou lui donne un autre dessin de la même feuille, {target}.set(autreDessin)"), request.pos),
                (None, "set", Some(Value::Name(other))) if drawing(other).is_some() => {
                    return refused(format!("« {target}.set({other}) » : « {other} » est un dessin ; il ne va que dans un autre dessin, jamais dans un texte"), request.pos)
                }
                _ => {}
            }
        }
        Ok(())
    })?;
    // Un dessin ne se montre pas comme un texte : il se voit dans sa zone de dessin.
    fn texts_in<'a>(value: &'a Value, found: &mut Vec<&'a String>) {
        match value {
            Value::Text(text) => found.push(text),
            Value::List(elements) => elements.iter().for_each(|e| texts_in(e, found)),
            Value::Block(block) => block.arguments.iter().for_each(|a| texts_in(&a.value, found)),
            _ => {}
        }
    }
    let mut written = Vec::new();
    program.root.arguments.iter().filter(|a| a.name.as_deref() != Some("state")).for_each(|a| texts_in(&a.value, &mut written));
    for (sheet, pos) in &found {
        if written.iter().any(|text| shows(text, &sheet.value)) {
            return Err(Error { message: format!("« {{{0}}} » : « {0} » est un dessin, il se voit dans sa zone ; pour dire s'il est vide, If({0}, is: \"\", children: [ … ])", sheet.value), pos: *pos });
        }
        if crate::history::names(program).unwrap_or_default().contains(&sheet.value) {
            return Err(Error { message: format!("« address: » : « {} » est un dessin ; il ne s'écrit pas dans l'adresse de la page (trop lourd, et lisible par tous)", sheet.value), pos: *pos });
        }
    }
    Ok(())
}

/// Les zones de dessin rangées dans un formulaire.
fn forms_of(program: &Program) -> Vec<*const Block> {
    let mut inside = Vec::new();
    let _ = for_each_block(&program.root, &mut |block| {
        if block.name == "Form" {
            let _ = for_each_block(block, &mut |child| {
                if child.name == "Sketch" {
                    inside.push(child as *const Block);
                }
                Ok(())
            });
        }
        Ok(())
    });
    inside
}


#[cfg(test)]
mod sketch_tests {
    use super::*;
    use crate::state::encode;

    const PAGE: &str = "Page(title: \"Essai\", state: State(drawing: \"\", name: \"\"), keep: [drawing], children: [ H1(\"Essai\"), Sketch(name: Cat, value: drawing, label: \"Ton dessin\", export: [svg, png]), If(drawing, not: \"\", children: [ P(\"Merci\") ]) ])";

    fn sheet_of_page(source: &str, value: &str) -> Sheet {
        sheet_of(&crate::check_page(source).unwrap(), value).unwrap()
    }

    /// Un trait droit, en `n` points bruts, comme les envoie la page.
    fn line(from: (f64, f64), to: (f64, f64), n: usize) -> String {
        (0..n).map(|i| {
            let t = i as f64 / (n - 1) as f64;
            format!("{} {}", from.0 + t * (to.0 - from.0), from.1 + t * (to.1 - from.1))
        }).collect::<Vec<_>>().join(" ")
    }

    /// Le dessin d'un état, décodé.
    fn drawing_in(state: &str, name: &str) -> String {
        state.split(';').find_map(|chunk| chunk.strip_prefix(&format!("{name}='"))).map(|code| crate::state::decode(code).unwrap()).unwrap_or_default()
    }

    fn refused(source: &str) -> String {
        match crate::check_page(source) {
            Ok(_) => panic!("accepté, alors qu'il devait être refusé :\n{source}"),
            Err(error) => error.message,
        }
    }

    /// Des dessins faux, chacun refusé tout entier, d'où qu'il vienne.
    fn forged() -> Vec<(String, &'static str)> {
        let many = vec!["#1a1a1a 2 1,1"; STROKES_MAX + 1].join("|");
        let long = format!("#1a1a1a 2 {}", (0..=STROKE_POINTS_MAX).map(|i| format!("{},{}", i % 400, i % 300)).collect::<Vec<_>>().join(" "));
        let heavy = (0..40).map(|_| format!("#1a1a1a 2 {}", (0..40).map(|i| format!("{},{}", 100 + i, 200 + i)).collect::<Vec<_>>().join(" "))).collect::<Vec<_>>().join("|");
        assert!(weight(&heavy) > WEIGHT_MAX && heavy.split('|').count() <= STROKES_MAX, "{}", weight(&heavy));
        vec![
            ("#123456 5 10,10".into(), "une couleur que la feuille ne propose pas"),
            ("#1A1A1A 5 10,10".into(), "une couleur écrite autrement"),
            ("red 5 10,10".into(), "une couleur par son nom"),
            ("#1a1a1a 7 10,10".into(), "une épaisseur que la feuille ne propose pas"),
            ("#1a1a1a 05 10,10".into(), "un zéro devant"),
            ("#1a1a1a 5 -1,10".into(), "un nombre négatif"),
            ("#1a1a1a 5 1.5,10".into(), "un nombre à virgule"),
            ("#1a1a1a 5 401,10".into(), "hors de la feuille, à droite"),
            ("#1a1a1a 5 10,301".into(), "hors de la feuille, en bas"),
            ("#1a1a1a 5 99999,10".into(), "un nombre trop long"),
            ("#1a1a1a 5  10,10".into(), "deux espaces"),
            ("#1a1a1a 5 10,10|".into(), "un trait vide à la fin"),
            ("#1a1a1a 5".into(), "un trait sans point"),
            ("#1a1a1a 5 10;10".into(), "un point mal écrit"),
            ("#1a1a1a 5 10,10\n".into(), "un retour à la ligne"),
            ("#1a1a1a 5 10,10\"/><script>alert(1)</script>".into(), "du code"),
            (many, "plus de 200 traits"),
            (long, "un trait de plus de 500 points"),
            (heavy, "plus de 12 000 octets"),
        ]
    }

    #[test]
    fn a_sketch_is_a_drawing_area_with_its_tools() {
        let html = crate::flat_view(PAGE, "").unwrap();
        // Un groupe de champs nommé ; la feuille est une image avec son nom et sa description.
        assert!(html.contains("<fieldset class=\"holo-Sketch\" data-name=\"Cat\" data-sketch=\"drawing\" data-group=\"drawing\" data-start-color=\"#1a1a1a\" data-start-size=\"5\"><legend>Ton dessin</legend><svg class=\"holo-sketch-sheet\" viewBox=\"0 0 400 300\" width=\"400\" height=\"300\" style=\"width:min(100%,calc(70vh * 400 / 300))\" role=\"img\" aria-label=\"Ton dessin\" aria-describedby=\"holo-sketch-drawing-said\" data-sketch-sheet><g class=\"holo-sketch-strokes\"></g></svg><p class=\"holo-sketch-said\" id=\"holo-sketch-drawing-said\" role=\"status\">Rien n'est encore dessiné.</p>"), "{html}");
        // Les outils, cachés sans JavaScript : six couleurs nommées, trois épaisseurs (la moyenne au
        // départ), annuler, effacer, enregistrer.
        assert!(html.contains("<div class=\"holo-sketch-tools\" hidden><div class=\"holo-sketch-choices\" role=\"radiogroup\" aria-label=\"Couleur\"><label class=\"holo-sketch-choice\"><input type=\"radio\" name=\"holo-sketch-drawing-color\" value=\"#1a1a1a\" data-sketch-color checked><span class=\"holo-sketch-swatch\" style=\"background:#1a1a1a\"></span><span class=\"holo-hidden\">noir</span></label>"), "{html}");
        for name in ["rouge", "orange", "vert", "bleu", "violet", "fin", "moyen", "épais"] {
            assert!(html.contains(&format!("<span class=\"holo-hidden\">{name}</span>")), "{name}\n{html}");
        }
        assert!(html.contains("value=\"5\" data-sketch-size checked>") && html.contains("value=\"2\" data-sketch-size>") && html.contains("value=\"12\" data-sketch-size>"), "{html}");
        assert!(html.contains("<button type=\"button\" class=\"holo-sketch-do\" data-sketch-do=\"undo\">Annuler</button><button type=\"button\" class=\"holo-sketch-do\" data-sketch-do=\"clear\">Effacer</button><button type=\"button\" class=\"holo-sketch-do\" data-sketch-do=\"svg\">Enregistrer en SVG</button><button type=\"button\" class=\"holo-sketch-do\" data-sketch-do=\"png\">Enregistrer en PNG</button></div><noscript><p class=\"holo-sketch-note\">Dessiner demande JavaScript ; le dessin se voit quand même.</p></noscript></fieldset>"), "{html}");
        // Une page avec une zone de dessin est vivante : le moteur arrive tout de suite.
        assert!(html.contains(" data-live"), "{html}");
        // Le défilement d'un téléphone n'est pris que sur la feuille : ni sur le groupe, ni sur les outils.
        let rule = |selector: &str| html.split(&format!(":where({selector}){{")).nth(1).and_then(|rest| rest.split('}').next()).unwrap_or("").to_string();
        assert!(rule(".holo-sketch-sheet").contains("touch-action:none"), "{html}");
        for other in [".holo-Sketch", ".holo-sketch-tools", ".holo-sketch-choice", ".holo-sketch-do"] {
            assert!(!rule(other).is_empty() && !rule(other).contains("touch-action"), "{other} : {}", rule(other));
        }
        // En anglais, les mots suivent la langue de la page ; une seule couleur, deux épaisseurs, une feuille large.
        let english = crate::flat_view("Page(lang: \"en\", title: \"Sign\", state: State(signature: \"\"), children: [ H1(\"Sign\"), Sketch(value: signature, label: \"Your signature\", width: 600, height: 200, colors: [\"navy\"], thickness: [3, 6]) ])", "").unwrap();
        assert!(english.contains("viewBox=\"0 0 600 200\"") && english.contains("Nothing drawn yet.") && english.contains(">Undo</button>") && english.contains(">Clear</button>"), "{english}");
        assert!(!english.contains("aria-label=\"Colour\"") && english.contains("aria-label=\"Thickness\"") && english.contains(">thin</span>") && english.contains(">thick</span>"), "{english}");
        assert!(!english.contains("data-sketch-do=\"png\"") && english.contains("Drawing needs JavaScript; the drawing still shows."), "{english}");
        let sheet = sheet_of_page(PAGE, "drawing");
        assert_eq!((sheet.width, sheet.height, sheet.thickness.clone(), starting_thickness(&sheet), sheet.png, sheet.svg), (400, 300, vec![2, 5, 12], 5, true, true));
    }

    #[test]
    fn a_stroke_is_smoothed_simplified_and_kept_as_data() {
        let start = crate::initial_state(PAGE);
        assert_eq!(start, "drawing=';name='");
        // Une droite de 51 points bruts : deux points gardés, en nombres entiers.
        let after = crate::sketch_stroke(PAGE, &start, "drawing", "#c62828", 5, &line((10.0, 10.0), (110.0, 60.0), 51)).unwrap();
        assert_eq!(drawing_in(&after, "drawing"), "#c62828 5 10,10 110,60");
        // Un doigt qui tremble (±0,4 unité) sur une ligne : lissé, il redevient une droite.
        let shaky: Vec<String> = (0..80).map(|i| format!("{} {}", 20.0 + i as f64 * 4.0, 200.0 + if i % 2 == 0 { 0.4 } else { -0.4 })).collect();
        let after = crate::sketch_stroke(PAGE, &after, "drawing", "#1a1a1a", 2, &shaky.join(" ")).unwrap();
        assert_eq!(drawing_in(&after, "drawing"), "#c62828 5 10,10 110,60|#1a1a1a 2 20,200 336,200");
        // Un demi-cercle de 61 points : une vingtaine au plus, du premier au dernier.
        let curve: Vec<String> = (0..=60).map(|i| { let t = i as f64 / 60.0 * std::f64::consts::PI; format!("{:.2} {:.2}", 200.0 + 80.0 * t.cos(), 150.0 + 80.0 * t.sin()) }).collect();
        let after = crate::sketch_stroke(PAGE, &after, "drawing", "#1565c0", 12, &curve.join(" ")).unwrap();
        let sheet = sheet_of_page(PAGE, "drawing");
        let read = strokes(&sheet, &drawing_in(&after, "drawing")).unwrap();
        assert!(read[2].points.len() <= 20 && read[2].points.first() == Some(&(280, 150)) && read[2].points.last() == Some(&(120, 150)), "{:?}", read[2]);
        // Ce qui sort de la feuille (le doigt a glissé dehors) est ramené au bord ; un point seul est un rond.
        let after = crate::sketch_stroke(PAGE, &after, "drawing", "#2e7d32", 5, "-50 -10 500 400").unwrap();
        let after = crate::sketch_stroke(PAGE, &after, "drawing", "#6a1b9a", 12, "33.4 44.6").unwrap();
        assert!(drawing_in(&after, "drawing").ends_with("|#2e7d32 5 0,0 400,300|#6a1b9a 12 33,45"), "{after}");
        // La page montre les traits, puis ce qui a été dessiné, dit avec des mots.
        let view = crate::sketch_view(PAGE, &after, "drawing");
        let (shown, said) = view.split_once('\n').unwrap();
        assert!(shown.starts_with("<path d=\"M10 10L110 60\" fill=\"none\" stroke=\"#c62828\" stroke-width=\"5\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>") && shown.ends_with("<circle cx=\"33\" cy=\"45\" r=\"6\" fill=\"#6a1b9a\"/>"), "{shown}");
        assert!(shown.contains("<path d=\"M280 150Q"), "une courbe qui passe par le milieu des segments : {shown}");
        assert_eq!(said, "5 traits (1 en rouge, 1 en noir, 1 en bleu, 1 en vert, 1 en violet), sur toute la feuille.");
        // La page fabriquée par un serveur part du même dessin ; une condition le voit.
        let program = crate::check_page(PAGE).unwrap();
        let start = (crate::state::reread(&program, &after), crate::state::reread_texts(&program, &after), crate::lists::reread(&program, &after));
        let html = crate::flat::site_html_from(&program, &program.root, "", "", Some(&start)).unwrap();
        assert!(html.contains("<g class=\"holo-sketch-strokes\"><path d=\"M10 10L110 60\"") && html.contains(">5 traits (1 en rouge, "), "{html}");
        assert!(crate::conditions(PAGE, &after).contains(":1") && crate::conditions(PAGE, &start_state()).contains(":0"));
        // Gardé d'une visite à l'autre (keep), puis repris tel quel.
        let kept = crate::to_keep(PAGE, &after);
        assert_eq!(drawing_in(&crate::resume(PAGE, &kept), "drawing"), drawing_in(&after, "drawing"));
        // Annuler et effacer : la page redonne un dessin d'avant, ou vide ; le moteur le relit.
        let undone = crate::input(PAGE, &after, "drawing", "#c62828 5 10,10 110,60");
        assert_eq!(drawing_in(&undone, "drawing"), "#c62828 5 10,10 110,60");
        assert_eq!(drawing_in(&crate::input(PAGE, &after, "drawing", ""), "drawing"), "");
    }

    fn start_state() -> String {
        crate::initial_state(PAGE)
    }

    #[test]
    fn a_stroke_out_of_bounds_is_refused_from_the_page() {
        // Règle : une couleur, une épaisseur, des points hors des bornes sont refusés ; l'état ne change pas.
        let start = start_state();
        let refusal = |color: &str, thickness: u32, points: &str| crate::sketch_stroke(PAGE, &start, "drawing", color, thickness, points).unwrap_err();
        assert_eq!(refusal("#123456", 5, "1 1"), "Cette couleur n'est pas proposée ici.");
        assert_eq!(refusal("#C62828", 5, "1 1"), "Cette couleur n'est pas proposée ici.");
        assert_eq!(refusal("#c62828", 7, "1 1"), "Cette épaisseur n'est pas proposée ici.");
        for unreadable in ["", "1", "1 2 3", "1 NaN", "inf 2", "1 x", "1,2 3,4", "0x10 1"] {
            assert_eq!(refusal("#c62828", 5, unreadable), "Ce trait est illisible : il n'est pas gardé.", "{unreadable:?}");
        }
        // Plus de 4 000 points bruts : trop long, même s'il se simplifierait.
        assert_eq!(refusal("#c62828", 5, &line((0.0, 0.0), (400.0, 300.0), RAW_MAX + 1)), "Ce trait est trop long : il n'est pas gardé.");
        crate::sketch_stroke(PAGE, &start, "drawing", "#c62828", 5, &line((0.0, 0.0), (400.0, 300.0), RAW_MAX)).unwrap();
        // Un gribouillis qui saute partout : plus de 500 points une fois simplifié.
        let mut seed: u64 = 7;
        let scribble: Vec<String> = (0..1500).map(|_| {
            seed = (seed * 1_103_515_245 + 12_345) % (1 << 31);
            format!("{} {}", 20 + seed % 360, 20 + (seed / 360) % 260)
        }).collect();
        assert_eq!(refusal("#c62828", 5, &scribble.join(" ")), "Ce trait est trop long : il n'est pas gardé.");
        // Le dessin plein : le 201e trait n'est pas gardé.
        let mut state = start.clone();
        for i in 0..STROKES_MAX {
            state = crate::sketch_stroke(PAGE, &state, "drawing", "#1a1a1a", 2, &format!("{} {}", i % 400, i % 300)).unwrap();
        }
        assert_eq!(drawing_in(&state, "drawing").split('|').count(), STROKES_MAX);
        assert_eq!(crate::sketch_stroke(PAGE, &state, "drawing", "#1a1a1a", 2, "5 5").unwrap_err(), "Le dessin est plein : ce trait n'est pas gardé. Annule ou efface pour continuer.");
        // Trop lourd : de longs traits, jusqu'à 12 000 octets.
        let mut state = start.clone();
        let mut refused_at = None;
        for i in 0..STROKES_MAX {
            let wavy: Vec<String> = (0..60).map(|k| format!("{} {}", 10.0 + 6.0 * k as f64, 20.0 + (i * 7 % 260) as f64 + 15.0 * (k as f64 / 3.0).sin())).collect();
            match crate::sketch_stroke(PAGE, &state, "drawing", "#1a1a1a", 2, &wavy.join(" ")) {
                Ok(after) => state = after,
                Err(reason) => {
                    refused_at = Some((i, reason));
                    break;
                }
            }
        }
        let (at, reason) = refused_at.expect("le dessin aurait dû être plein");
        assert!(at > 5 && reason.starts_with("Le dessin est plein") && weight(&drawing_in(&state, "drawing")) <= WEIGHT_MAX, "{at} {reason}");
        // Une valeur qui n'est pas celle d'une zone de dessin ; en anglais.
        assert_eq!(crate::sketch_stroke(PAGE, &start, "name", "#1a1a1a", 2, "1 1").unwrap_err(), "Cette zone de dessin n'existe pas.");
        let english = PAGE.replace("Page(title:", "Page(lang: \"en\", title:");
        assert_eq!(crate::sketch_stroke(&english, &start, "drawing", "#123456", 5, "1 1").unwrap_err(), "This colour is not offered here.");
    }

    #[test]
    fn a_drawing_from_elsewhere_is_read_strictly_wherever_it_enters() {
        // Règle : un dessin faux est refusé tout entier, partout où il entre : un état relu (une page,
        // holo serve, un état forgé), la page qui le donne (annuler, un champ forgé sans JavaScript),
        // keep, visit, des données, un formulaire reçu par le serveur, une valeur partagée.
        let sheet = sheet_of_page(PAGE, "drawing");
        let good = "#c62828 5 10,10 110,60|#1a1a1a 2 0,0";
        let with_good = crate::input(PAGE, &start_state(), "drawing", good);
        assert_eq!(drawing_in(&with_good, "drawing"), good);
        let visit = "Page(title: \"Essai\", state: State(drawing: \"\"), visit: [drawing], children: [ H1(\"Essai\"), Sketch(value: drawing, label: \"Ton dessin\") ])";
        let data = "Page(title: \"Essai\", state: State(drawing: \"\"), data: Data(from: \"d.json\"), children: [ H1(\"Essai\"), Sketch(value: drawing, label: \"Ton dessin\") ])";
        let form = "Page(title: \"Essai\", state: State(drawing: \"\"), children: [ H1(\"Essai\"), Form(name: Order, children: [ Sketch(value: drawing, label: \"Ton dessin\"), Button(name: Send, text: \"Envoyer\") ]) ], rules: [ On(Send.tap, effect: Order.send) ])";
        let shared = "Page(title: \"Essai\", state: State(mine: \"\"), shared: Shared(drawing: \"\"), children: [ H1(\"Essai\"), Sketch(value: mine, label: \"Le tien\"), Sketch(value: drawing, label: \"Le mur\"), Button(name: Wall, text: \"Au mur\") ], rules: [ On(Wall.tap, effect: drawing.set(mine)) ])";
        // Le bon dessin passe partout.
        assert_eq!(drawing_in(&crate::resume(PAGE, &format!("drawing='{}", encode(good))), "drawing"), good);
        assert_eq!(drawing_in(&crate::from_visit(visit, &crate::initial_state(visit), &format!("drawing\t{}", crate::json_text(good))), "drawing"), good);
        assert_eq!(drawing_in(&crate::receive(data, &crate::initial_state(data), &format!("{{\"drawing\": {}}}", crate::json_text(good))), "drawing"), good);
        assert_eq!(crate::check_submission(form, &format!("{{\"form\":\"Order\",\"values\":{{\"drawing\":{}}}}}", crate::json_text(good))).unwrap(), "");
        assert_eq!(drawing_in(&crate::with_shared(shared, &crate::initial_state(shared), &format!("drawing='{}", encode(good))), "drawing"), good);
        for (text, why) in forged() {
            assert!(strokes(&sheet, &text).is_none() && clean(&sheet, &text).is_none(), "{why} : accepté");
            let state = format!("drawing='{}", encode(&text));
            // Un état relu (la page, holo serve, un état forgé) : le dessin reprend son départ.
            assert_eq!(drawing_in(&crate::arbitrate(PAGE, &state, "Nothing.tap"), "drawing"), "", "{why}");
            assert_eq!(crate::sketch_view(PAGE, &state, "drawing"), "\nRien n'est encore dessiné.", "{why}");
            // La page, ou un champ forgé envoyé sans JavaScript : le dessin ne change pas.
            assert_eq!(drawing_in(&crate::input(PAGE, &with_good, "drawing", &text), "drawing"), good, "{why}");
            // keep, visit, des données.
            assert_eq!(drawing_in(&crate::resume(PAGE, &state), "drawing"), "", "{why}");
            assert_eq!(drawing_in(&crate::from_visit(visit, &crate::initial_state(visit), &format!("drawing\t{}", crate::json_text(&text))), "drawing"), "", "{why}");
            assert_eq!(drawing_in(&crate::receive(data, &crate::initial_state(data), &format!("{{\"drawing\": {}}}", crate::json_text(&text))), "drawing"), "", "{why}");
            // Un formulaire reçu par le serveur.
            assert_eq!(crate::check_submission(form, &format!("{{\"form\":\"Order\",\"values\":{{\"drawing\":{}}}}}", crate::json_text(&text))).unwrap(), "drawing|un dessin de cette feuille attendu", "{why}");
            // Une valeur partagée, telle que le serveur la garde ou que le direct l'apporte.
            assert_eq!(drawing_in(&crate::with_shared(shared, &crate::initial_state(shared), &state), "drawing"), "", "{why}");
        }
        assert_eq!(crate::check_submission(form, "{\"form\":\"Order\",\"values\":{\"drawing\":12}}").unwrap(), "drawing|un dessin attendu");
    }

    #[test]
    fn the_saved_image_is_made_by_the_engine_only() {
        // Règle : rien d'exécutable dans l'image enregistrée. Le moteur la fabrique d'après les
        // nombres et les couleurs relus ; aucun texte du visiteur n'y entre.
        let source = PAGE.replace("label: \"Ton dessin\"", "label: \"Mon <b>chat</b> & moi\"");
        let after = crate::sketch_stroke(&source, &start_state(), "drawing", "#c62828", 5, &line((10.0, 10.0), (110.0, 60.0), 20)).unwrap();
        let after = crate::sketch_stroke(&source, &after, "drawing", "#1a1a1a", 12, "200 100").unwrap();
        let svg = crate::sketch_svg(&source, &after, "drawing");
        assert_eq!(svg, "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"400\" height=\"300\" viewBox=\"0 0 400 300\"><title>Mon &lt;b&gt;chat&lt;/b&gt; &amp; moi</title><desc>2 traits (1 en rouge, 1 en noir), en haut à gauche.</desc><rect width=\"400\" height=\"300\" fill=\"#ffffff\"/><path d=\"M10 10L110 60\" fill=\"none\" stroke=\"#c62828\" stroke-width=\"5\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/><circle cx=\"200\" cy=\"100\" r=\"6\" fill=\"#1a1a1a\"/></svg>");
        // Seulement ces balises et ces attributs : ni script, ni lien, ni style, ni objet étranger.
        let tags: Vec<&str> = svg.split('<').skip(1).map(|t| t.trim_start_matches('/').split([' ', '>', '/']).next().unwrap()).collect();
        assert!(tags.iter().all(|t| ["svg", "title", "desc", "rect", "path", "circle"].contains(t)), "{tags:?}");
        for word in ["script", "href", "style", "foreignObject", "<a ", "javascript", " on"] {
            assert!(!svg.contains(word), "{word} : {svg}");
        }
        // Un dessin forgé avec du code est refusé avant : l'image est celle d'une feuille vide.
        let forged = format!("drawing='{}", encode("#c62828 5 10,10\"/><script>alert(1)</script>"));
        assert!(crate::sketch_svg(&source, &forged, "drawing").contains("<desc>Rien n'est encore dessiné.</desc><rect width=\"400\" height=\"300\" fill=\"#ffffff\"/></svg>"));
        // Une zone qui ne propose pas d'enregistrer ne rend pas d'image.
        assert_eq!(crate::sketch_svg(&PAGE.replace(", export: [svg, png]", ""), &after, "drawing"), "");
    }

    #[test]
    fn the_checks_refuse_what_would_misuse_a_drawing() {
        let page = |state: &str, children: &str| format!("Page(title: \"Essai\", state: State({state}), children: [ H1(\"Essai\"), {children} ])");
        let sketch = |params: &str| page("drawing: \"\"", &format!("Sketch(value: drawing, label: \"Ton dessin\"{params})"));
        crate::check_page(&sketch("")).unwrap();
        crate::check_page(&sketch(", width: 600, height: 200, colors: [\"black\", \"#c62828\", \"navy\"], thickness: [3, 6], export: [png]")).unwrap();
        for (source, message) in [
            (page("drawing: \"\"", "Sketch(label: \"Ton dessin\")"), "attend « value »"),
            (page("drawing: \"\"", "Sketch(value: drawing)"), "attend « label »"),
            (sketch(", label: \"\"").replace("label: \"Ton dessin\", ", ""), "attend « label »"),
            (page("drawing: \"\"", "Sketch(value: \"drawing\", label: \"x\")"), "le nom d'un texte de la page"),
            (page("other: \"\"", "Sketch(value: drawing, label: \"x\")"), "déclare-le vide, state: State(drawing: \"\")"),
            (page("drawing: 0", "Sketch(value: drawing, label: \"x\")"), "déclare-le vide"),
            (page("drawing: \"abc\"", "Sketch(value: drawing, label: \"x\")"), "n'est pas un dessin de cette feuille"),
            (page("drawing: \"\"", "Sketch(value: drawing, label: \"a\"), Sketch(value: drawing, label: \"b\")"), "déjà la valeur d'une autre zone"),
            (page("a: \"\", b: \"\", c: \"\", d: \"\", e: \"\"", "Sketch(value: a, label: \"a\"), Sketch(value: b, label: \"b\"), Sketch(value: c, label: \"c\"), Sketch(value: d, label: \"d\"), Sketch(value: e, label: \"e\")"), "4 zones de dessin au plus"),
            (sketch(", width: 600"), "width et height ensemble"),
            (sketch(", width: 50, height: 200"), "de 100 à 2000"),
            (sketch(", width: 600, height: 3000"), "de 100 à 2000"),
            (sketch(", colors: []"), "de 1 à 12 couleurs"),
            (sketch(&format!(", colors: [{}]", vec!["\"black\""; 13].join(", "))), "de 1 à 12 couleurs"),
            (sketch(", colors: [\"transparent\"]"), "n'est pas une couleur pleine"),
            (sketch(", colors: [\"url(#a)\"]"), "n'est pas une couleur pleine"),
            (sketch(", colors: [\"#00000080\"]"), "n'est pas une couleur pleine"),
            (sketch(", colors: [black]"), "des couleurs entre guillemets"),
            (sketch(", colors: [\"yellow\"]"), "se verrait mal sur la feuille blanche : un contraste de 1,1 pour 1"),
            (sketch(", colors: [\"#000\", \"black\"]"), "proposée deux fois"),
            (sketch(", thickness: [0]"), "de 1 à 64"),
            (sketch(", thickness: [65]"), "de 1 à 64"),
            (sketch(", thickness: [2, 2]"), "proposée deux fois"),
            (sketch(", thickness: [1, 2, 3, 4, 5, 6]"), "de 1 à 5 épaisseurs"),
            (sketch(", thickness: [2.5]"), "de 1 à 64"),
            (sketch(", export: [gif]"), "png, svg, ou les deux"),
            (sketch(", export: [png, png]"), "png, svg, ou les deux"),
            (sketch(", export: png"), "la liste des images"),
            (sketch(", required: true"), "se vérifie à l'envoi d'un formulaire"),
            (sketch(", pressure: true"), "n'a pas de paramètre « pressure »"),
            (sketch(", label: \"{name}\"").replace("label: \"Ton dessin\", ", ""), "sans valeur « {…} »"),
            (sketch(&format!(", label: \"{}\"", "a".repeat(201))).replace("label: \"Ton dessin\", ", ""), "200 caractères au plus"),
            (page("drawing: \"\", tasks: [\"a\"]", "Repeat(over: tasks, children: [ Sketch(value: drawing, label: \"x\") ])"), "ne se répète pas dans les lignes d'une liste"),
            (page("drawing: \"\"", "Sketch(value: drawing, label: \"x\"), Input(value: drawing, label: \"y\")"), "« Input(value: drawing) » : « drawing » est un dessin"),
            (page("drawing: \"\"", "Sketch(value: drawing, label: \"x\"), Choice(value: drawing, label: \"y\", options: [\"a\", \"b\"])"), "« Choice(value: drawing) » : « drawing » est un dessin"),
            (page("drawing: \"\"", "Sketch(value: drawing, label: \"x\"), Device(name: Clip, label: \"Coller\", kind: clipboard, value: drawing)"), "un appareil n'y écrit pas"),
            (page("drawing: \"\"", "Sketch(value: drawing, label: \"x\"), Transfer(name: File, label: \"Fichier\", file: \"d.json\", values: [drawing])"), "il s'enregistre en image"),
            (page("drawing: \"\"", "Sketch(value: drawing, label: \"x\"), Split(name: parts, from: drawing, by: \"|\")"), "pas un texte à découper"),
            (page("drawing: \"\"", "Sketch(value: drawing, label: \"x\"), P(\"{drawing}\")"), "« {drawing} » : « drawing » est un dessin, il se voit dans sa zone"),
            (format!("Page(title: \"{{drawing}}\", state: State(drawing: \"\"), children: [ H1(\"x\"), Sketch(value: drawing, label: \"x\") ])"), "il se voit dans sa zone"),
            (format!("Page(title: \"x\", state: State(drawing: \"\"), address: [drawing], children: [ H1(\"x\"), Sketch(value: drawing, label: \"x\") ])"), "ne s'écrit pas dans l'adresse"),
        ] {
            let error = refused(&source);
            assert!(error.contains(message), "{source}\n→ {error}");
        }
        // Les règles : un dessin se vide, ou prend un autre dessin de la même feuille ; rien d'autre.
        let ruled = |rules: &str, more: &str| format!("Page(title: \"x\", state: State(drawing: \"\", other: \"\", name: \"\"), children: [ H1(\"x\"), Sketch(value: drawing, label: \"x\"), Sketch(value: other, label: \"y\"{more}), Button(name: Go, text: \"Go\") ], rules: [ {rules} ])");
        crate::check_page(&ruled("On(Go.tap, effect: [drawing.set(\"\"), other.set(drawing)])", "")).unwrap();
        for (source, message) in [
            (ruled("On(Go.tap, effect: drawing.set(\"abc\"))", ""), "une règle le vide, drawing.set(\"\")"),
            (ruled("On(Go.tap, effect: drawing.set(name))", ""), "il prend un autre dessin, ou se vide"),
            (ruled("On(Go.tap, effect: name.set(drawing))", ""), "il ne va que dans un autre dessin, jamais dans un texte"),
            (ruled("On(Go.tap, effect: other.set(drawing))", ", colors: [\"black\"]"), "n'ont pas la même feuille"),
            (ruled("On(Go.tap, effect: other.set(drawing))", ", width: 400, height: 400"), "n'ont pas la même feuille"),
        ] {
            let error = refused(&source);
            assert!(error.contains(message), "{source}\n→ {error}");
        }
        // Une zone de dessin hors d'une page, et deux dessins partagés.
        assert!(refused("Point(name: P, seed: 1, inside: World(children: [ Sketch(value: drawing, label: \"x\") ]))").contains("dans une page") || crate::check("Point(name: P, seed: 1, inside: World(children: [ Sketch(value: drawing, label: \"x\") ]))").is_err());
        let two = "Page(title: \"x\", shared: Shared(a: \"\", b: \"\"), children: [ H1(\"x\"), Sketch(value: a, label: \"a\"), Sketch(value: b, label: \"b\") ])";
        assert!(refused(two).contains("une page partage un seul dessin"), "{}", refused(two));
    }

    #[test]
    fn a_shared_drawing_changes_only_by_a_tap() {
        // Règle : un dessin partagé ne change que par un toucher, arbitré par le serveur, qui relit le
        // dessin donné ; il garde ses propres bornes (il n'est pas coupé à 200 caractères).
        let source = "Page(title: \"Le mur\", state: State(mine: \"\"), shared: Shared(wall: \"\"), children: [ H1(\"Le mur\"), Sketch(value: mine, label: \"Ton dessin\"), If(mine, not: \"\", children: [ Button(name: Wall, text: \"Le mettre au mur\") ]), Sketch(value: wall, label: \"Le mur\", export: [png]) ], rules: [ On(Wall.tap, effect: wall.set(mine)) ])";
        let html = crate::flat_view(source, "").unwrap();
        let wall = html.split("data-sketch=\"wall\"").nth(1).unwrap().split("</fieldset>").next().unwrap();
        assert!(html.contains("data-sketch=\"wall\" data-sketch-shared") && !wall.contains("data-sketch-color") && !wall.contains("data-sketch-do=\"undo\"") && !wall.contains("data-sketch-do=\"clear\"") && wall.contains("data-sketch-do=\"png\""), "{wall}");
        assert!(wall.contains("Ce dessin est partagé : chacun le voit changer en direct."), "{wall}");
        // On ne dessine pas sur le mur.
        assert_eq!(crate::sketch_stroke(source, &crate::initial_state(source), "wall", "#1a1a1a", 5, "1 1").unwrap_err(), "On ne dessine pas sur un dessin partagé.");
        // Un visiteur dessine, puis touche « Le mettre au mur » : le serveur arbitre.
        let mut visitor = crate::initial_state(source);
        for i in 0..4 {
            let wave: Vec<String> = (0..60).map(|k| format!("{} {}", 20.0 + 6.0 * k as f64, 60.0 + 60.0 * i as f64 + 20.0 * (k as f64 / 4.0).sin())).collect();
            visitor = crate::sketch_stroke(source, &visitor, "mine", "#1565c0", 5, &wave.join(" ")).unwrap();
        }
        let drawn = drawing_in(&visitor, "mine");
        let (after, accepted) = crate::share(source, &visitor, "", "Wall.tap");
        assert!(accepted, "{after}");
        assert!(drawn.len() > crate::shared::SHARED_TEXT_MAX && drawing_in(&crate::shared_of(source, &after), "wall") == drawn, "{after}");
        // Un état forgé : son dessin faux est relu vide ; le bouton est alors caché, le serveur refuse.
        let forged = format!("mine='{}", encode("#1565c0 5 1,1\"><script>"));
        let (_, accepted) = crate::share(source, &forged, &crate::shared_of(source, &after), "Wall.tap");
        assert!(!accepted);
        // Ce que le serveur garde, ou ce que le direct apporte, est relu : un mur forgé reste vide.
        assert_eq!(drawing_in(&crate::with_shared(source, &visitor, &format!("wall='{}", encode("#000000 5 1,1"))), "wall"), "");
        // Un mur plein : 16 Kio de valeurs partagées au plus, que le dessin tient.
        assert!(crate::shared_of(source, &after).len() <= crate::shared::SHARED_BYTES_MAX);
    }

    #[test]
    fn a_drawing_travels_in_a_form() {
        let source = "Page(title: \"Signer\", state: State(signature: \"\", sent: 0), children: [ H1(\"Signer\"), Form(name: Order, children: [ Sketch(value: signature, label: \"Ta signature\", width: 600, height: 200, colors: [\"navy\"], thickness: [3, 6], required: true), Button(name: Send, text: \"Envoyer\") ]) ], rules: [ On(Send.tap, effect: Order.send), On(Order.sent, effect: [sent.set(1), signature.set(\"\")]) ])";
        let start = crate::initial_state(source);
        // Obligatoire : un trait au moins avant d'envoyer.
        assert_eq!(crate::form_errors(source, &start, "Order"), "signature|Dessine avant d'envoyer.");
        assert_eq!(crate::form_errors(&source.replace("Page(title:", "Page(lang: \"en\", title:"), &start, "Order"), "signature|Draw before sending.");
        let signed = crate::sketch_stroke(source, &start, "signature", "#000080", 3, &line((50.0, 150.0), (550.0, 50.0), 40)).unwrap();
        assert_eq!(crate::form_errors(source, &signed, "Order"), "");
        // Le formulaire envoie le dessin, tel que le moteur l'écrit ; le serveur le relit.
        let sent = crate::submission(source, &signed, "Order");
        assert_eq!(sent, "{\"form\":\"Order\",\"values\":{\"signature\":\"#000080 3 50,150 550,50\"}}");
        assert_eq!(crate::check_submission(source, &sent).unwrap(), "");
        // Un dessin forgé : refusé ; et la signature, restée vide, manque.
        assert_eq!(crate::check_submission(source, "{\"form\":\"Order\",\"values\":{\"signature\":\"#000080 3 50,150 601,50\"}}").unwrap(), "signature|un dessin de cette feuille attendu\nsignature|Dessine avant d'envoyer.");
        assert_eq!(crate::check_submission(source, "{\"form\":\"Order\",\"values\":{\"signature\":\"\"}}").unwrap(), "signature|Dessine avant d'envoyer.");
        // Envoyé : la règle vide la signature.
        assert_eq!(drawing_in(&crate::arbitrate(source, &signed, "Order.sent"), "signature"), "");
        // Dans un formulaire, la zone est un groupe : ses erreurs se posent après elle.
        assert!(crate::flat_view(source, "").unwrap().contains("<fieldset class=\"holo-Sketch\" data-sketch=\"signature\" data-group=\"signature\" data-start-color=\"#000080\" data-start-size=\"3\"><legend>Ta signature</legend>"));
    }

    #[test]
    fn the_colours_are_named_and_readable() {
        let sheet = sheet_of_page(PAGE, "drawing");
        // Les couleurs par défaut se lisent sur la feuille blanche : 3 pour 1 au moins (WCAG 1.4.11).
        for color in COLORS {
            assert!(crate::styles::contrast(channels(color), [255.0; 3]) >= CONTRAST_MIN, "{color}");
        }
        assert_eq!(color_names(&sheet, true), ["noir", "rouge", "orange", "vert", "bleu", "violet"]);
        assert_eq!(color_names(&sheet, false), ["black", "red", "orange", "green", "blue", "purple"]);
        let mine = sheet_of_page(&PAGE.replace("export: [svg, png]", "colors: [\"navy\", \"#1565c0\", \"maroon\", \"#5d4037\", \"#0d47a1\", \"gray\"]"), "drawing");
        assert_eq!(color_names(&mine, true), ["bleu foncé", "bleu", "rouge foncé", "marron", "bleu 2", "gris"]);
        assert_eq!(color_names(&mine, false), ["dark blue", "blue", "dark red", "brown", "blue 2", "grey"]);
        assert_eq!(thickness_names(&sheet, true), ["fin", "moyen", "épais"]);
        // Ce qui a été dessiné, avec une seule couleur ; un trait seul.
        let one = |text: &str| described(&sheet, &strokes(&sheet, text).unwrap(), true);
        assert_eq!(one("#1a1a1a 5 10,10 20,20|#1a1a1a 2 30,30"), "2 traits en noir, en haut à gauche.");
        assert_eq!(one("#c62828 12 380,280"), "1 trait en rouge, en bas à droite.");
        assert_eq!(described(&sheet, &strokes(&sheet, "#c62828 12 200,150").unwrap(), false), "1 stroke in red, in the middle.");
    }
}
