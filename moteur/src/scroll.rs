//! Où en est le visiteur dans la page, et un bloc qui reste à l'écran (ADR-106).
//!
//! ```holo
//! Page(
//!   children: [
//!     Progress(value: scroll, max: 100, label: "Lecture", sticky: top),
//!     H1("Un long article", name: Top),
//!     If(scroll, over: 10, children: [ A("↑ Retour en haut", to: "#Top", sticky: bottom) ]),
//!   ],
//! )
//! ```
//!
//! - `scroll` : de 0 (en haut de la page) à 100 (tout en bas), en pour cent entiers ; 0 pour une
//!   page qui tient dans l'écran. La page la lit comme ses autres valeurs (`{scroll}`,
//!   `If(scroll, over: 10, …)`, `Progress(value: scroll)`, `When(scroll, over: 90, …)`), sans
//!   jamais la changer : c'est le navigateur qui la donne au moteur quand le visiteur défile, au
//!   plus dix fois par seconde, jamais à chaque pixel (`page-engine.js`), et seulement à une page
//!   qui la lit. Sans JavaScript, elle vaut 0 : la page arrive en haut, et ne dépend pas d'elle
//!   pour se lire.
//! - `sticky: top | bottom` : un bloc posé directement dans la page (ou `Header`, `Footer`)
//!   reste à l'écran, en haut ou en bas, pendant qu'on défile ; c'est le `position: sticky` du
//!   CSS, en pur CSS, donc aussi sans JavaScript. Une exception étroite au refus de `position`
//!   (ADR-017) : ni décalage, ni ordre de superposition, rien d'autre ; un bloc par bord ; jamais
//!   plus du quart de l'écran ; il ne cache jamais ce qui a le focus (`scroll-padding`).

use crate::holo::{Argument, Block, Error, Program, Target, Value};
use crate::rules::for_each_block;
use crate::state::State;

/// La place du visiteur dans la page, une valeur que le moteur donne : on la lit, on ne la change pas.
pub const NAME: &str = "scroll";

/// Les deux bords où un bloc reste à l'écran.
pub const EDGES: &[&str] = &["top", "bottom"];

/// La part de l'écran qu'un bloc qui reste peut prendre, au plus : le quart.
pub const SHARE: &str = "25vh";

/// Les blocs qui ne restent pas à l'écran : ils ne se voient pas eux-mêmes, ou ont déjà leur place.
const NOT_STICKY: &[&str] = &["If", "Repeat", "Dialog", "Sound", "Main", "Item", "Point", "Scenes", "Scene", "Module", "Data", "Filter", "Days", "Font", "Abbreviation", "Term"];

/// Le style d'un bloc qui reste à l'écran, écrit seulement quand la page en a un. En `:where()`,
/// pour qu'un style de l'auteur (son fond, sa marge) l'emporte toujours ; la part de l'écran et la
/// marge laissée au focus, elles, ne se changent pas. Sur un écran bas (un téléphone couché, le
/// clavier qui réduit la page), sous le clavier de l'écran (`holo-keyboard`, posé par la page) et à
/// l'impression, le bloc rend sa place. En bas, il laisse la place du bouton rond du moteur.
const CSS: &str = ":where([data-sticky]){position:sticky;z-index:1;background:var(--holo-sticky-background,Canvas)}\
:where([data-sticky=top]){top:0}:where([data-sticky=bottom]){bottom:0;padding-right:72px}\
.holo-Page [data-sticky]{max-height:25vh;overflow-y:auto}\
html:has([data-sticky=top]){scroll-padding-top:var(--holo-sticky-top,25vh)}\
html:has([data-sticky=bottom]){scroll-padding-bottom:var(--holo-sticky-bottom,25vh)}\
@media (max-height:480px){.holo-Page [data-sticky]{position:static;max-height:none}html:has([data-sticky]){scroll-padding:0}}\
.holo-keyboard [data-sticky]{position:static}\
@media print{.holo-Page [data-sticky]{position:static;max-height:none}}";

/// La page lit-elle `scroll` : dans un texte (`{scroll}`), une condition, une comparaison, une
/// barre (`Progress(value: scroll)`), une demande (`best.set(scroll)`) ?
fn read_by(program: &Program) -> bool {
    fn visit(value: &Value) -> bool {
        match value {
            Value::Text(text) => crate::state::names_in(text).contains(&NAME),
            Value::Name(name) => name == NAME,
            Value::List(elements) => elements.iter().any(visit),
            Value::Block(block) => block.arguments.iter().any(|a| visit(&a.value)),
            _ => false,
        }
    }
    program.root.arguments.iter().any(|a| visit(&a.value))
}

/// La page a-t-elle reçu `scroll` (elle la lit) ? Le navigateur la lui donne alors quand le
/// visiteur défile.
pub fn reads(program: &Program) -> bool {
    matches!(program.root.argument("state").map(|a| &a.value), Some(Value::Block(state)) if state.argument(NAME).is_some())
}

/// Pose `scroll` dans les valeurs de la page, à 0, si elle la lit, comme si elle l'avait déclarée
/// dans `State`. Refusé, avec la raison : la déclarer (dans `State` ou dans `Shared`), la changer
/// (une demande, un champ, un module, l'appareil, un glissement, un chronomètre, un fichier
/// importé), la garder (`keep`) ou la mettre dans l'adresse (`address:`).
pub fn inject(program: &mut Program) -> Result<(), Error> {
    for holder in ["state", "shared"] {
        if let Some(Value::Block(block)) = program.root.argument(holder).map(|a| &a.value) {
            if let Some(declared) = block.argument(NAME) {
                return Err(Error { message: "« scroll » est la place du visiteur dans la page, donnée par le navigateur : ne la déclare ni dans State ni dans Shared, choisis un autre nom pour ta valeur (ADR-106)".into(), pos: declared.pos });
            }
        }
    }
    for (setting, why) in [("keep", "elle ne se garde pas d'une visite à l'autre"), ("address", "l'adresse ne la porte pas")] {
        if let Some(Argument { value: Value::List(names), pos, .. }) = program.root.argument(setting) {
            if names.iter().any(|n| matches!(n, Value::Name(n) if n == NAME)) {
                return Err(Error { message: format!("« {setting} » : « scroll » est la place du visiteur dans la page, donnée par le navigateur ; {why}"), pos: *pos });
            }
        }
    }
    for_each_block(&program.root, &mut |block| {
        let named = |key: &str| matches!(block.argument(key).map(|a| &a.value), Some(Value::Name(n)) if n == NAME);
        let changed = match block.name.split_once('.') {
            Some((target, _)) if crate::state::is_requested(block) => target == NAME,
            _ if ["Input", "Checkbox", "Choice", "Slider", "Stopwatch", "Device"].contains(&block.name.as_str()) => named("value"),
            _ if block.name == "Module" => match block.argument("output").map(|a| &a.value) {
                Some(Value::Name(output)) => output == NAME,
                Some(Value::List(outputs)) => outputs.iter().any(|o| matches!(o, Value::Name(n) if n == NAME)),
                _ => false,
            },
            _ if block.name == "Transfer" => matches!(block.argument("values").map(|a| &a.value), Some(Value::List(values)) if values.iter().any(|v| matches!(v, Value::Name(n) if n == NAME))),
            _ if matches!(block.argument("drag").map(|a| &a.value), Some(Value::Bool(true))) => named("x") || named("y"),
            _ => false,
        };
        if changed {
            return Err(Error { message: "« scroll » est la place du visiteur dans la page, donnée par le navigateur quand il défile : on la lit, on ne la change pas".into(), pos: block.pos });
        }
        Ok(())
    })?;
    if !read_by(program) {
        return Ok(());
    }
    match program.root.name.as_str() {
        "Page" => {}
        // Un monde ne défile pas ; un morceau (`Component`) se vérifie avec la page qui le pose.
        "World" | "Point" => return Err(Error { message: "« scroll » est la place du visiteur dans une page : elle se lit dans Page(…)".into(), pos: program.root.pos }),
        _ => return Ok(()),
    }
    let pos = program.root.pos;
    let given = |pos| Argument { name: Some(NAME.to_string()), value: Value::Integer(0), pos };
    match program.root.arguments.iter_mut().find(|a| a.name.as_deref() == Some("state")) {
        Some(Argument { value: Value::Block(state), .. }) if state.name == "State" => {
            let at = state.pos;
            state.arguments.push(given(at));
        }
        Some(_) => return Err(Error { message: "« state » attend un bloc « State(...) », sur la page : state: State(cart: 0)".into(), pos }),
        None => program.root.arguments.push(Argument { name: Some("state".into()), value: Value::Block(Block { name: "State".into(), styles: Vec::new(), arguments: vec![given(pos)], pos }), pos }),
    }
    Ok(())
}

/// Des données reçues (`Data`) ne changent pas la place du visiteur : elle reste celle d'avant.
/// Seul le navigateur la donne.
pub fn kept(program: &Program, before: &State, mut after: State) -> State {
    if !reads(program) {
        return after;
    }
    let place = before.iter().find(|(name, _)| name == NAME).map_or(0, |(_, value)| *value);
    if let Some((_, slot)) = after.iter_mut().find(|(name, _)| name == NAME) {
        *slot = place;
    }
    after
}

/// Le bord où un bloc reste à l'écran : `sticky: top` ou `sticky: bottom`.
pub fn sticky_of(block: &Block) -> Option<&str> {
    match &block.argument("sticky")?.value {
        Value::Name(edge) if EDGES.contains(&edge.as_str()) => Some(edge),
        _ => None,
    }
}

/// Vérifie `sticky:` : `top` ou `bottom`, sur un bloc qui se voit, posé directement dans la page
/// (dans `Page(children:)` ou `Main(children:)`, au besoin sous un `If`), ou sur `Header` et
/// `Footer` posés dans la page ; un bloc par bord.
pub fn check(program: &Program) -> Result<(), Error> {
    // Les places permises : les blocs de la page, et ceux d'un `If` ou d'un `Main` posé dans la page.
    fn allowed_in(values: &[Value], allowed: &mut Vec<*const Block>) {
        for value in values {
            let Value::Block(block) = value else { continue };
            allowed.push(block as *const Block);
            if matches!(block.name.as_str(), "If" | "Main") {
                for key in ["children", "else"] {
                    if let Some(Value::List(inside)) = block.argument(key).map(|a| &a.value) {
                        allowed_in(inside, allowed);
                    }
                }
            }
        }
    }
    let mut allowed = Vec::new();
    if program.root.name == "Page" {
        if let Some(Value::List(children)) = program.root.argument("children").map(|a| &a.value) {
            allowed_in(children, &mut allowed);
        }
    }
    let mut edges: Vec<String> = Vec::new();
    for_each_block(&program.root, &mut |block| {
        let Some(argument) = block.argument("sticky") else { return Ok(()) };
        let refusal = |message: String| Err(Error { message, pos: argument.pos });
        let Some(edge) = sticky_of(block) else {
            return refusal("« sticky » attend top ou bottom : le bord de l'écran où le bloc reste, sticky: top".into());
        };
        if NOT_STICKY.contains(&block.name.as_str()) {
            return refusal(format!("« {}(sticky: …) » : un {} ne reste pas à l'écran ; mets « sticky » sur un bloc qui se voit, posé dans la page", block.name, block.name));
        }
        if !allowed.contains(&(block as *const Block)) {
            return refusal(format!("« sticky » garde à l'écran un bloc posé directement dans la page, ou Header et Footer : « {} » est rangé dans un autre bloc ; mets « sticky » sur le bloc de la page qui le contient", block.name));
        }
        if edges.iter().any(|known| known == edge) {
            return refusal(format!("une page a un seul bloc qui reste en {} (sticky: {edge}) : range ensemble ce qui doit rester, Row(sticky: {edge}, children: [ … ])", if edge == "top" { "haut" } else { "bas" }));
        }
        edges.push(edge.to_string());
        Ok(())
    })
}

/// Marque le HTML d'un bloc qui reste à l'écran, sur sa propre balise (une enveloppe changerait
/// sa place dans la page) : `<header class=…>` → `<header data-sticky="top" class=…>`.
pub fn with_sticky(html: &str, edge: &str) -> String {
    let Some(start) = html.find('<') else { return html.to_string() };
    let name_end = html[start + 1..].find(|c: char| c == ' ' || c == '>').map_or(html.len(), |end| start + 1 + end);
    format!("{} data-sticky=\"{edge}\"{}", &html[..name_end], &html[name_end..])
}

/// Le style des blocs qui restent à l'écran, quand la page en a : leur place, leur part de
/// l'écran, la marge laissée au focus ; et leur fond, quand l'auteur n'en donne pas : celui de la
/// page (une couleur, ou une variable), aussi dans le thème sombre ; sinon celui du navigateur.
/// Sans fond, le texte qui passe dessous se lirait à travers. Vide pour une page sans `sticky`.
pub fn css(program: &Program) -> String {
    let mut sticky = false;
    let _ = for_each_block(&program.root, &mut |block| {
        sticky |= block.argument("sticky").is_some();
        Ok(())
    });
    if !sticky {
        return String::new();
    }
    let color = |settings: &[crate::holo::Setting]| {
        let value = settings.iter().find(|s| s.name == "background")?.value.trim().to_string();
        if crate::styles::is_color(&value) {
            Some(value)
        } else if value.starts_with("--") && crate::styles::variables_of(&value) == [value.as_str()] {
            Some(format!("var({value})"))
        } else {
            None
        }
    };
    let mut css = String::from(CSS);
    for rule in program.styles.iter().filter(|rule| matches!(&rule.target, Target::Type(t) if t == "Page")) {
        if let Some(value) = color(&rule.settings) {
            css.push_str(&format!(".holo-Page{{--holo-sticky-background:{value}}}"));
        }
        for (_, settings, _) in rule.states.iter().filter(|(state, ..)| state == "dark") {
            if let Some(value) = color(settings) {
                css.push_str(&format!("@media (prefers-color-scheme:dark){{.holo-Page{{--holo-sticky-background:{value}}}}}"));
            }
        }
    }
    css
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTICLE: &str = r#"Page(
  title: "Read {scroll} %",
  state: State(done: 0),
  children: [
    Row.bar(sticky: top, children: [ Progress(value: scroll, max: 100, label: "Reading"), Text("{scroll} %") ]),
    H1("A long article", name: Top),
    P("Some text."),
    If(scroll, over: 10, children: [ A.up("↑ Back to top", to: "#Top", sticky: bottom) ]),
    If(done, is: 1, children: [ P("Thank you for reading.") ]),
  ],
  rules: [ When(scroll, over: 89, effect: done.set(1)) ],
)
Page { background: #101020; color: white; dark: { background: --night; } --night: #000010; }
.bar { padding: 8px 0; }
.up { background: #E9B44C; color: #101020; }"#;

    #[test]
    fn scroll_is_given_by_the_browser_and_watched_by_the_rules() {
        // La page lit scroll : elle la reçoit, à 0 ; elle est vivante (le moteur arrive tout de suite).
        let start = crate::initial_state(ARTICLE);
        assert!(start.contains("scroll=0") && start.contains("done=0"), "{start}");
        assert!(crate::reads_scroll(ARTICLE) && !crate::reads_scroll("Page(children: [ P(\"x\") ])"));
        let html = crate::flat_view(ARTICLE, "").unwrap();
        assert!(html.contains(" data-live") && html.contains(r#"data-progress="scroll""#) && html.contains(r#"data-if="scroll|over=10" hidden"#), "{html}");
        // Le visiteur descend : la valeur suit, bornée à 100 ; la règle qui guette se déclenche une
        // fois, au moment où la valeur passe 89 ; un titre qui lit scroll suit aussi.
        let half = crate::scrolled(ARTICLE, &start, 50);
        assert!(half.contains("scroll=50") && half.contains("done=0"), "{half}");
        assert_eq!(crate::conditions(ARTICLE, &half), "scroll|over=10:1;done|is=1:0");
        let end = crate::scrolled(ARTICLE, &half, 250);
        assert!(end.contains("scroll=100") && end.contains("done=1"), "{end}");
        assert_eq!(crate::page_title(ARTICLE, &end), "Read 100 %");
        // Remonter ne défait pas ce que la règle a fait ; un toucher ne change pas scroll.
        let up = crate::scrolled(ARTICLE, &end, 0);
        assert!(up.contains("scroll=0") && up.contains("done=1"), "{up}");
        assert!(crate::arbitrate(ARTICLE, &half, "Nobody.tap").contains("scroll=50"));
        // Une page qui ne lit pas scroll ne la reçoit pas.
        assert_eq!(crate::scrolled("Page(state: State(n: 1), children: [ P(\"{n}\") ])", "n=1", 40), "n=1");
        // Des données reçues ne la changent pas : seul le navigateur la donne.
        let data = "Page(data: Data(from: \"d.json\"), state: State(n: 0), children: [ P(\"{scroll} {n}\") ])";
        let received = crate::receive(data, &crate::scrolled(data, &crate::initial_state(data), 30), "{\"scroll\": 77, \"n\": 2}");
        assert!(received.contains("scroll=30") && received.contains("n=2"), "{received}");
        // Une page sans scroll ni sticky n'a rien de plus : ni valeur, ni style, ni moteur tout de suite.
        let plain = crate::flat_view("Page(children: [ P(\"x\") ])", "").unwrap();
        assert!(!plain.contains("scroll") && !plain.contains("data-sticky") && !plain.contains(" data-live"), "{plain}");
    }

    #[test]
    fn a_sticky_block_stays_on_screen_and_never_hides_the_focus() {
        let html = crate::flat_view(ARTICLE, "").unwrap();
        // Le bloc garde sa balise et sa place ; il est marqué, en haut ou en bas.
        assert!(html.contains(r#"<div data-sticky="top" class="holo-Row holo-s-bar""#), "{html}");
        assert!(html.contains(r#"<a data-sticky="bottom" class="holo-A holo-s-up" href="#Top">↑ Back to top</a>"#), "{html}");
        // Le style : la place, le quart de l'écran au plus, le focus jamais caché, un écran trop bas
        // ou le clavier ouvert qui rendent sa place au bloc ; à l'impression aussi.
        for rule in [
            ":where([data-sticky]){position:sticky;",
            ".holo-Page [data-sticky]{max-height:25vh;overflow-y:auto}",
            "html:has([data-sticky=top]){scroll-padding-top:var(--holo-sticky-top,25vh)}",
            "html:has([data-sticky=bottom]){scroll-padding-bottom:var(--holo-sticky-bottom,25vh)}",
            "@media (max-height:480px){",
            ".holo-keyboard [data-sticky]{position:static}",
            "@media print{",
        ] {
            assert!(html.contains(rule), "{rule}\n{html}");
        }
        // Le fond de la page, aussi dans le thème sombre, quand le bloc n'a pas le sien.
        assert!(html.contains(".holo-Page{--holo-sticky-background:#101020}@media (prefers-color-scheme:dark){.holo-Page{--holo-sticky-background:var(--night)}}"), "{html}");
        // Header et Footer posés dans la page ; un bloc sous un If ou dans Main.
        let landmarks = "Page(children: [ Header(sticky: top, children: [ Text(\"Studio\") ]), Main(children: [ P(\"x\") ]), Footer(sticky: bottom, children: [ Text(\"©\") ]) ])";
        let html = crate::flat_view(landmarks, "").unwrap();
        assert!(html.contains(r#"<header data-sticky="top" class="holo-Header">"#) && html.contains(r#"<footer data-sticky="bottom" class="holo-Footer">"#), "{html}");
        assert!(!html.contains("--holo-sticky-background"), "{html}");
        // Un bloc qui reste n'est pas une page vivante : le CSS suffit, le moteur attend un geste.
        assert!(!html.contains(" data-live"), "{html}");
        crate::check_page("Page(children: [ Main(children: [ If(n, is: 0, children: [ P(\"x\", sticky: top) ], else: [ P(\"y\") ]) ]) ], state: State(n: 0))").unwrap();
        // Un bloc qui bouge et qui reste : marqué sur son enveloppe de mouvement, qui tient sa place.
        let moving = crate::flat_view("Page(children: [ H1(\"x\", sticky: top, enter: Enter(y: 4px)) ])", "").unwrap();
        assert!(moving.contains(" data-sticky=\"top\""), "{moving}");
        // Un style de l'auteur ne dit pas « position » : le réglage est sur le bloc.
        let error = crate::check_page("Page(children: [ P.bar(\"x\") ])\n.bar { position: sticky; }").unwrap_err();
        assert!(error.message.contains("écris « sticky: top »"), "{error}");
    }

    #[test]
    fn what_is_refused() {
        for (source, message) in [
            ("Page(state: State(scroll: 0), children: [ P(\"{scroll}\") ])", "ne la déclare ni dans State ni dans Shared"),
            ("Page(state: State(scroll: 0), children: [ P(\"x\") ])", "ne la déclare ni dans State ni dans Shared"),
            ("Page(shared: Shared(scroll: 0), children: [ P(\"{scroll}\") ])", "ne la déclare ni dans State ni dans Shared"),
            ("Page(children: [ Button(name: B, text: \"x\"), P(\"{scroll}\") ], rules: [ On(B.tap, effect: scroll.set(0)) ])", "on la lit, on ne la change pas"),
            ("Page(children: [ Slider(value: scroll, label: \"x\") ])", "on la lit, on ne la change pas"),
            ("Page(keep: [scroll], children: [ P(\"{scroll}\") ])", "elle ne se garde pas"),
            ("Page(address: [scroll], children: [ P(\"{scroll}\") ])", "l'adresse ne la porte pas"),
            ("Point(name: W, seed: 1, inside: World(children: [ P(\"{scroll}\") ]))", "se lit dans Page"),
            ("Page(children: [ P(\"x\", sticky: middle) ])", "attend top ou bottom"),
            ("Page(children: [ P(\"x\", sticky: \"top\") ])", "attend top ou bottom"),
            ("Page(children: [ Row(children: [ P(\"x\", sticky: top) ]) ])", "posé directement dans la page"),
            ("Page(children: [ Header(children: [ Nav(sticky: top, children: [ A(\"x\", to: \"#X\") ]) ]), H2(\"X\", name: X) ])", "posé directement dans la page"),
            ("Page(state: State(n: 0), children: [ If(n, is: 0, sticky: top, children: [ P(\"x\") ]) ])", "un If ne reste pas à l'écran"),
            ("Page(children: [ P(\"a\", sticky: top), P(\"b\", sticky: top) ])", "un seul bloc qui reste en haut"),
            ("Page(children: [ Point(name: P1, seed: 1, inside: World(children: [ P(\"x\", sticky: top) ])) ])", "posé directement dans la page"),
            ("Page(children: [ P(\"x\") ], rules: [ On(P.tap, sticky: top, effect: x.add(1)) ])", "n'a pas de paramètre « sticky »"),
        ] {
            let error = crate::check_page(source).err().unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }
}
