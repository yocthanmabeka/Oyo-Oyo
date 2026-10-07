//! Le mouvement (ADR-034) : faire entrer un bloc, le faire vivre en boucle, enchaîner des scènes.
//!
//! ```holo
//! Scenes(height: 480px, repeat: forever, children: [
//!   Scene(for: 4s, children: [
//!     H1("Hello", enter: Enter(y: 40px, opacity: 0, letters: 0.04s, ease: spring)),
//!     Shape(form: circle, loop: Loop(scale: 1.2, for: 1s)),
//!   ]),
//! ])
//! ```
//!
//! Rien ne bouge si l'auteur ne l'écrit pas. Tout devient du CSS fabriqué par le moteur : une
//! page qui bouge n'a pas besoin du moteur dans le navigateur pour bouger, et le visiteur qui
//! demande moins de mouvement (prefers-reduced-motion) voit la page arrêtée.

use crate::holo::{Block, Error, Value};
use std::cell::RefCell;
use std::fmt::Write;

/// Les paramètres de mouvement qu'accepte tout bloc qui se voit.
pub const PARAMS: &[&str] = &["enter", "loop"];

/// Les courbes, du plus simple au plus vivant.
const CURVES: &[(&str, &str)] = &[
    ("linear", "linear"),
    ("smooth", "cubic-bezier(.65,0,.35,1)"),
    ("out", "cubic-bezier(.16,1,.3,1)"),
    ("in", "cubic-bezier(.7,0,.84,0)"),
    ("back", "cubic-bezier(.34,1.56,.64,1)"),
    ("spring", "linear(0,.009,.035 2.1%,.141,.281 6.7%,.723 12.9%,.938 16.7%,1.017,1.077,1.121,1.149 24.3%,1.159,1.163,1.161,1.154 29.9%,1.129 32.8%,1.051 39.6%,1.017 43.1%,.991,.977 51%,.974 53.8%,.975 57.1%,.997 69.8%,1.003 76.9%,1.004 83.8%,1)"),
    ("bounce", "linear(0,.004,.016,.035,.063,.098,.141 13.6%,.25,.391,.563,.765,1,.891 40.9%,.848,.813,.785,.766,.754,.75,.754,.766,.785,.813,.848,.891 68.2%,1 72.7%,.973,.953,.941,.938,.941,.953,.973,1,.988,.984,.988,1)"),
];

/// Ce qu'on peut faire bouger, avec son unité et ses bornes.
const PROPERTIES: &[(&str, Option<&str>, f64, f64)] = &[
    ("opacity", None, 0.0, 1.0),
    ("x", Some("px"), -4000.0, 4000.0),
    ("y", Some("px"), -4000.0, 4000.0),
    ("scale", None, 0.0, 20.0),
    ("rotate", Some("deg"), -36000.0, 36000.0),
    ("flip", Some("deg"), -36000.0, 36000.0),
    ("tilt", Some("deg"), -36000.0, 36000.0),
    ("blur", Some("px"), 0.0, 200.0),
    ("hue", Some("deg"), -36000.0, 36000.0),
    ("round", None, 0.0, 50.0),
];

/// Une pose : les valeurs de départ d'une entrée, ou celles qu'une boucle va chercher.
#[derive(Debug, Default, Clone, PartialEq)]
struct Placed(Vec<(&'static str, f64)>);

impl Placed {
    fn a(&self, name: &str) -> Option<f64> {
        self.0.iter().find(|(known, _)| *known == name).map(|(_, v)| *v)
    }

    /// Les déclarations CSS de cette pose ; `naturel` : celles du bloc au repos.
    fn css(&self, natural: bool) -> String {
        let v = |name: &str, rest: f64| if natural { rest } else { self.a(name).unwrap_or(rest) };
        let a = |name: &str| self.a(name).is_some();
        let mut css = String::new();
        if a("opacity") {
            let _ = write!(css, "opacity:{};", v("opacity", 1.0));
        }
        if a("x") || a("y") {
            let _ = write!(css, "translate:{}px {}px;", v("x", 0.0), v("y", 0.0));
        }
        if a("scale") {
            let _ = write!(css, "scale:{};", v("scale", 1.0));
        }
        if a("rotate") {
            let _ = write!(css, "rotate:{}deg;", v("rotate", 0.0));
        }
        if a("flip") || a("tilt") {
            let _ = write!(css, "transform:perspective(800px) rotateX({}deg) rotateY({}deg);", v("tilt", 0.0), v("flip", 0.0));
        }
        if a("blur") || a("hue") {
            let _ = write!(css, "filter:blur({}px) hue-rotate({}deg);", v("blur", 0.0), v("hue", 0.0));
        }
        if a("round") {
            let _ = write!(css, "border-radius:{}%;", v("round", 0.0));
        }
        css
    }
}

/// Un mouvement lu dans le fichier : `Enter(…)` ou `Loop(…)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Movement {
    cycle: bool,
    placed: Placed,
    /// Quand il commence, en secondes, depuis le début de la scène (ou de la page).
    a: f64,
    /// Combien de temps il dure, en secondes (pour une boucle : un aller).
    duration: f64,
    curve: &'static str,
    /// Lettre après lettre : l'écart entre deux lettres.
    letters: Option<f64>,
    /// Enfant après enfant : l'écart entre deux enfants.
    each: Option<f64>,
    /// Une boucle revient-elle à son point de départ (aller-retour) ? Sinon elle recommence.
    back: bool,
    /// `Enter(inView: true)` : l'entrée attend que le bloc arrive à l'écran (ADR-061).
    in_view: bool,
}

fn error(message: String, block: &Block) -> Error {
    Error { message, pos: block.pos }
}

fn number(value: &Value) -> Option<(f64, Option<&str>)> {
    match value {
        Value::Integer(n) => Some((*n as f64, None)),
        Value::Number { value, unit } => Some((*value, unit.as_deref())),
        _ => None,
    }
}

/// Une durée, `0.4s` ou `400ms`, en secondes.
fn duration(value: &Value) -> Option<f64> {
    match number(value)? {
        (n, Some("s")) => Some(n),
        (n, Some("ms")) => Some(n / 1000.0),
        _ => None,
    }
}

/// La durée d'une scène, de 200ms à 600s.
pub fn scene_duration(value: &Value) -> Option<f64> {
    duration(value).filter(|d| (0.2..=600.0).contains(d))
}

/// Lit `Enter(…)` ou `Loop(…)`.
pub fn read(value: &Value, param: &str) -> Result<Movement, Error> {
    let expected = if param == "enter" { "Enter" } else { "Loop" };
    let block = match value {
        Value::Block(block) if block.name == expected => block,
        _ => {
            return Err(Error {
                message: format!("« {param}: » attend {expected}(…), comme {param}: {expected}(y: 40px, opacity: 0)"),
                pos: crate::holo::Pos { line: 0, column: 0 },
            })
        }
    };
    let cycle = expected == "Loop";
    let mut movement = Movement { cycle, placed: Placed::default(), a: 0.0, duration: if cycle { 1.0 } else { 0.8 }, curve: if cycle { CURVES[1].1 } else { CURVES[2].1 }, letters: None, each: None, back: true, in_view: false };
    let possible = || {
        let mut names: Vec<&str> = PROPERTIES.iter().map(|(n, ..)| *n).collect();
        names.extend(["at", "for", "ease", "letters", "each"]);
        if cycle {
            names.push("back");
        } else {
            names.push("inView");
        }
        names.join(", ")
    };
    for argument in &block.arguments {
        let Some(name) = argument.name.as_deref() else {
            return Err(error(format!("chaque paramètre de « {expected} » est nommé : {expected}(y: 40px, opacity: 0)"), block));
        };
        let v = &argument.value;
        let outside = |what: String| Error { message: what, pos: argument.pos };
        match name {
            "at" => movement.a = duration(v).filter(|d| (0.0..=600.0).contains(d)).ok_or_else(|| outside(format!("« {expected}(at: …) » attend un moment de 0s à 600s, comme at: 1.5s")))?,
            "for" => movement.duration = duration(v).filter(|d| (0.05..=600.0).contains(d)).ok_or_else(|| outside(format!("« {expected}(for: …) » attend une durée de 50ms à 600s, comme for: 0.8s")))?,
            "letters" => movement.letters = Some(duration(v).filter(|d| (0.005..=2.0).contains(d)).ok_or_else(|| outside(format!("« {expected}(letters: …) » attend l'écart entre deux lettres, de 5ms à 2s, comme letters: 0.04s")))?),
            "each" => movement.each = Some(duration(v).filter(|d| (0.01..=10.0).contains(d)).ok_or_else(|| outside(format!("« {expected}(each: …) » attend l'écart entre deux enfants, de 10ms à 10s, comme each: 0.1s")))?),
            "ease" => {
                movement.curve = match v {
                    Value::Name(word) => CURVES.iter().find(|(known, _)| known == word).map(|(_, css)| *css),
                    _ => None,
                }
                .ok_or_else(|| outside(format!("« {expected}(ease: …) » attend l'une de ces courbes : {}", CURVES.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", "))))?
            }
            "back" if cycle => match v {
                Value::Bool(b) => movement.back = *b,
                _ => return Err(outside("« Loop(back: …) » attend true ou false".into())),
            },
            "inView" if !cycle => match v {
                Value::Bool(b) => movement.in_view = *b,
                _ => return Err(outside("« Enter(inView: …) » attend true ou false : le bloc entre quand il arrive à l'écran".into())),
            },
            _ => match PROPERTIES.iter().find(|(known, ..)| *known == name) {
                Some((property, unit, min, max)) => {
                    let read_value = number(v).filter(|(n, u)| *u == *unit && (*min..=*max).contains(n)).map(|(n, _)| n);
                    let example = match unit {
                        Some(u) => format!("un nombre en {u}, de {min} à {max}"),
                        None => format!("un nombre de {min} à {max}"),
                    };
                    let n = read_value.ok_or_else(|| outside(format!("« {expected}({property}: …) » attend {example}")))?;
                    movement.placed.0.push((property, n));
                }
                None => return Err(outside(format!("« {expected} » n'a pas de paramètre « {name} » ; paramètres possibles : {}", possible()))),
            },
        }
    }
    if movement.placed.0.is_empty() {
        return Err(error(format!("« {expected} » dit ce qui bouge : {expected}(y: 40px), {expected}(scale: 1.2), {expected}(opacity: 0)…"), block));
    }
    Ok(movement)
}

/// La scène où l'on est en train de fabriquer la page : son début, et la durée d'un tour
/// quand les scènes recommencent sans fin.
#[derive(Debug, Clone, Copy, Default)]
struct Scene {
    start: f64,
    turn: Option<f64>,
}

#[derive(Default)]
struct Built {
    depth: u32,
    css: String,
    number_: u32,
    scene: Scene,
}

thread_local! {
    static BUILT: RefCell<Built> = RefCell::new(Built::default());
}

/// On commence à fabriquer une page.
pub fn begin() {
    BUILT.with(|f| {
        let mut f = f.borrow_mut();
        if f.depth == 0 {
            *f = Built::default();
        }
        f.depth += 1;
    });
}

/// La page est fabriquée : le CSS des mouvements qu'elle contient.
pub fn finish() -> String {
    BUILT.with(|f| {
        let mut f = f.borrow_mut();
        f.depth = f.depth.saturating_sub(1);
        if f.depth == 0 {
            std::mem::take(&mut f.css)
        } else {
            String::new()
        }
    })
}

fn number_() -> u32 {
    BUILT.with(|f| {
        let mut f = f.borrow_mut();
        f.number_ += 1;
        f.number_
    })
}

fn add(css: &str) {
    BUILT.with(|f| f.borrow_mut().css.push_str(css));
}

fn scene() -> Scene {
    BUILT.with(|f| f.borrow().scene)
}

/// Fabrique ce qui est dans une scène : ses mouvements partent de son début.
pub fn in_scene<T>(start: f64, turn: Option<f64>, f: impl FnOnce() -> T) -> T {
    let before = BUILT.with(|fab| std::mem::replace(&mut fab.borrow_mut().scene, Scene { start, turn }));
    let result = f();
    BUILT.with(|fab| fab.borrow_mut().scene = before);
    result
}

fn pourcent(t: f64, turn: f64) -> String {
    format!("{:.3}%", (t / turn * 100.0).clamp(0.0, 100.0))
}

/// L'animation d'un mouvement : la règle `animation:` et ses images clés, sous ce nom.
fn animation(m: &Movement, name: &str) -> (String, String) {
    let Scene { start, turn } = scene();
    let start_value = start + m.a;
    let (since, vers) = if m.cycle { (m.placed.css(true), m.placed.css(false)) } else { (m.placed.css(false), m.placed.css(true)) };
    match (m.cycle, turn) {
        // Une boucle tourne à son rythme, une fois sa scène commencée.
        (true, _) => {
            let direction = if m.back { "alternate" } else { "normal" };
            (format!("{name} {}s {} {}s infinite {direction} both", m.duration, m.curve, start_value), format!("@keyframes {name}{{from{{{since}}}to{{{vers}}}}}"))
        }
        // Une entrée, une seule fois.
        (false, None) => (format!("{name} {}s {} {}s both", m.duration, m.curve, start_value), format!("@keyframes {name}{{from{{{since}}}to{{{vers}}}}}")),
        // Une entrée dans des scènes qui recommencent : elle se rejoue à chaque tour.
        (false, Some(turn)) => (
            format!("{name} {turn}s linear infinite both"),
            format!(
                "@keyframes {name}{{0%{{{since}}}{}{{{since}animation-timing-function:{}}}{}{{{vers}}}100%{{{vers}}}}}",
                pourcent(start_value, turn),
                m.curve,
                pourcent(start_value + m.duration, turn)
            ),
        ),
    }
}

/// Le décalage de chaque lettre ou de chaque enfant, ajouté au départ.
fn shifted(rule: &str, gap: f64, rank: &str) -> String {
    format!("{rule};animation-delay:calc({} + {rank} * {gap}s)", delay_of(rule))
}

/// Le départ écrit dans une règle `animation:` (le dernier temps, ou 0s).
fn delay_of(rule: &str) -> String {
    let time: Vec<&str> = rule.split(' ').filter(|m| m.ends_with('s') && m.trim_end_matches('s').parse::<f64>().is_ok()).collect();
    if time.len() >= 2 { time[1].to_string() } else { "0s".to_string() }
}

/// Coupe un HTML en lettres, chacune dans sa boîte, en gardant les mots entiers.
fn in_letters(html: &str) -> String {
    let mut output = String::with_capacity(html.len() * 4);
    let mut rank = 0;
    let mut in_word = false;
    let mut chars = html.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '<' => {
                if in_word {
                    output.push_str("</span>");
                    in_word = false;
                }
                let end = html[i..].find('>').map_or(html.len(), |f| i + f + 1);
                output.push_str(&html[i..end]);
                while chars.peek().is_some_and(|(j, _)| *j < end) {
                    chars.next();
                }
            }
            c if c.is_whitespace() => {
                if in_word {
                    output.push_str("</span>");
                    in_word = false;
                }
                output.push(' ');
            }
            _ => {
                if !in_word {
                    output.push_str("<span class=\"holo-word\">");
                    in_word = true;
                }
                // Une entité (&amp;) reste une seule lettre.
                let letter = if c == '&' {
                    let end = html[i..].find(';').map_or(i + 1, |f| i + f + 1);
                    while chars.peek().is_some_and(|(j, _)| *j < end) {
                        chars.next();
                    }
                    &html[i..end]
                } else {
                    &html[i..i + c.len_utf8()]
                };
                let _ = write!(output, "<span class=\"holo-letter\" style=\"--i:{rank}\">{letter}</span>");
                rank += 1;
            }
        }
    }
    if in_word {
        output.push_str("</span>");
    }
    output
}

/// Les mouvements d'un bloc, et le bloc sans eux (pour le fabriquer comme d'habitude).
pub fn of_block(block: &Block) -> Result<Option<(Vec<Movement>, Block)>, Error> {
    // Un son ne se voit pas : rien à faire bouger (son « loop » serait autre chose).
    if block.name == "Sound" || !block.arguments.iter().any(|a| a.name.as_deref().is_some_and(|n| PARAMS.contains(&n))) {
        return Ok(None);
    }
    let mut movements = Vec::new();
    let mut remainder = block.clone();
    remainder.arguments.retain(|a| !a.name.as_deref().is_some_and(|n| PARAMS.contains(&n)));
    // La boucle d'abord : elle enveloppe l'entrée, pour que les deux ne se gênent pas.
    for param in ["loop", "enter"] {
        if let Some(argument) = block.argument(param) {
            let m = read(&argument.value, param).map_err(|e| Error { pos: argument.pos, ..e })?;
            if m.letters.is_some() && !matches!(block.name.as_str(), "H1" | "H2" | "H3" | "H4" | "H5" | "H6" | "P" | "Text" | "Button" | "Quote") {
                return Err(Error { message: "« letters: » coupe un texte en lettres : il va sur un titre (H1 à H6), P, Text, Button ou Quote".into(), pos: argument.pos });
            }
            if m.each.is_some() && !matches!(block.argument("children").map(|a| &a.value), Some(Value::List(_))) {
                return Err(Error { message: "« each: » fait bouger les enfants l'un après l'autre : il va sur un bloc qui a des « children »".into(), pos: argument.pos });
            }
            movements.push(m);
        }
    }
    Ok(Some((movements, remainder)))
}

/// Enveloppe le HTML d'un bloc dans ses mouvements.
pub fn wrap(movements: &[Movement], inside: String, children: usize, output: &mut String) {
    let mut html = inside;
    for m in movements.iter().rev() {
        let n = number_();
        let name = format!("hm{n}");
        let (rule, images) = animation(m, &name);
        let mut css = images;
        let rounded = if m.placed.a("round").is_some() { "overflow:hidden;" } else { "" };
        if let Some(gap) = m.letters {
            let _ = write!(css, ".{name}{{{rounded}}}.{name} .holo-letter{{animation:{}}}", shifted(&rule, gap, "var(--i)"));
            html = in_letters(&html);
        } else if let Some(gap) = m.each {
            let _ = write!(css, ".{name}{{{rounded}}}");
            for rank in 0..children {
                let _ = write!(css, ".{name}>*>:nth-child({}){{animation:{}}}", rank + 1, shifted(&rule, gap, &rank.to_string()));
            }
        } else {
            let _ = write!(css, ".{name}{{{rounded}animation:{rule}}}");
        }
        add(&css);
        // Une entrée qui attend d'être vue : la page la met en route quand le bloc arrive à l'écran.
        let in_view = if m.in_view { " holo-in-view" } else { "" };
        html = format!("<div class=\"holo-animated{in_view} {name}\">{html}</div>");
    }
    output.push_str(&html);
}

/// Le CSS d'une scène : elle apparaît à son début et s'efface à sa fin ; la dernière reste,
/// sauf quand les scènes recommencent.
fn scene_css(name: &str, start: f64, duration: f64, turn: Option<f64>, last: bool) -> String {
    let fade = (duration / 4.0).min(0.4);
    match turn {
        None => {
            let f = pourcent(fade, duration);
            let end = if last { "100%{opacity:1;visibility:visible}".to_string() } else { format!("{}{{opacity:1}}100%{{opacity:0;visibility:hidden}}", pourcent(duration - fade, duration)) };
            format!("@keyframes {name}{{0%{{opacity:0;visibility:visible}}{f}{{opacity:1}}{end}}}.{name}{{animation:{name} {duration}s linear {start}s both}}")
        }
        Some(turn) => format!(
            "@keyframes {name}{{0%{{opacity:0;visibility:hidden}}{}{{opacity:0;visibility:visible}}{}{{opacity:1}}{}{{opacity:1}}{}{{opacity:0;visibility:hidden}}100%{{opacity:0;visibility:hidden}}}}.{name}{{animation:{name} {turn}s linear infinite both}}",
            pourcent(start, turn),
            pourcent(start + fade, turn),
            pourcent(start + duration - fade, turn),
            pourcent(start + duration, turn)
        ),
    }
}

/// Une nouvelle scène : son CSS est ajouté à la page, et son nom rendu.
pub fn new_scene(start: f64, duration: f64, turn: Option<f64>, last: bool) -> String {
    let name = format!("hs{}", number_());
    add(&scene_css(&name, start, duration, turn, last));
    name
}

/// Le CSS de base du mouvement, ajouté à celui de la page.
pub const BASE: &str = ":where(.holo-animated){display:block}\
:where(.holo-word){display:inline-block;white-space:nowrap}:where(.holo-letter){display:inline-block}\
:where(.holo-Scenes){position:relative;overflow:hidden;width:100%}\
:where(.holo-Scene){position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:16px;text-align:center;opacity:0;padding:16px;box-sizing:border-box}\
:where(.holo-Scene)>*{margin:0}\
.holo-js .holo-in-view:not(.holo-seen),.holo-js .holo-in-view:not(.holo-seen) *{animation-play-state:paused}\
@media (prefers-reduced-motion:reduce){.holo-animated,.holo-animated *,.holo-Scene{animation:none!important}.holo-Scene{opacity:0;visibility:hidden}.holo-Scene:last-child{opacity:1;visibility:visible}}";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_letter_per_box_and_whole_words() {
        assert_eq!(
            in_letters("Hi <strong>you</strong> &amp; me"),
            "<span class=\"holo-word\"><span class=\"holo-letter\" style=\"--i:0\">H</span><span class=\"holo-letter\" style=\"--i:1\">i</span></span> <strong><span class=\"holo-word\"><span class=\"holo-letter\" style=\"--i:2\">y</span><span class=\"holo-letter\" style=\"--i:3\">o</span><span class=\"holo-letter\" style=\"--i:4\">u</span></span></strong> <span class=\"holo-word\"><span class=\"holo-letter\" style=\"--i:5\">&amp;</span></span> <span class=\"holo-word\"><span class=\"holo-letter\" style=\"--i:6\">m</span><span class=\"holo-letter\" style=\"--i:7\">e</span></span>"
        );
    }
}
