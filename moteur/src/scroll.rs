//! Où en est le visiteur dans la page, et un bloc qui reste à l'écran (ADR-106).
//!
//! ```holo
//! Page(
//!   children: [
//!     Row(sticky: top, children: [ Progress(value: scroll, max: 100, label: "Lecture") ]),
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
//!   pour se lire. Des données reçues ne la changent pas (`state::take_values`).
//! - `sticky: top | bottom` : un bloc posé directement dans la page (ou `Header`, `Footer`)
//!   reste à l'écran, en haut ou en bas, pendant qu'on défile ; c'est le `position: sticky` du
//!   CSS, en pur CSS, donc aussi sans JavaScript. Une exception étroite au refus de `position`
//!   (ADR-017) : ni décalage, ni ordre de superposition, rien d'autre ; un bloc par bord ; jamais
//!   plus du cinquième de la hauteur de l'écran ; rien ne reste sur un écran de moins de 480px de
//!   haut, ni pendant qu'on écrit avec le clavier de l'écran, ni sur papier ; il ne cache jamais
//!   ce qui a le focus (`scroll-padding`).

use crate::holo::{Argument, Block, Error, Program, Target, Value};
use crate::rules::for_each_block;

/// La place du visiteur dans la page, une valeur que le moteur donne : on la lit, on ne la change pas.
pub const NAME: &str = "scroll";

/// Les deux bords où un bloc reste à l'écran.
pub const EDGES: &[&str] = &["top", "bottom"];

/// La part de la hauteur de l'écran qu'un bloc qui reste peut prendre, au plus, en pour cent : le
/// cinquième. Avec un bloc en haut et un en bas, il reste toujours plus de la moitié de l'écran.
pub const SHARE: u32 = 20;

/// Sous cette hauteur d'écran, en pixels, aucun bloc ne reste : un téléphone couché, une page
/// grossie à 200 % ou plus (WCAG 1.4.10), où il prendrait la place de la lecture.
pub const LOWEST: u32 = 480;

/// Les blocs qui ne restent pas à l'écran : ils ne se voient pas eux-mêmes, ou ont déjà leur place.
const NOT_STICKY: &[&str] = &["If", "Repeat", "Dialog", "Main", "Item", "Point", "Scenes", "Scene", "Module", "Data", "Filter", "Days", "Font", "Abbreviation", "Term"];

/// Le style d'un bloc qui reste à l'écran, écrit seulement quand la page en a un, et seulement sur
/// un écran assez haut (sur papier, sur un écran bas, le bloc garde sa place dans la page) :
/// - sa place, en haut ou en bas ; en bas, au-dessus des touches à l'écran (`--holo-keys`, ADR-069) ;
/// - sa hauteur, le cinquième de l'écran au plus (ce qui dépasse défile dans le bloc) ;
/// - son fond, en `:where()` pour qu'un style de l'auteur l'emporte : sans fond, le texte qui
///   passe dessous se lirait à travers ;
/// - la marge laissée au focus (WCAG 2.4.11) : la hauteur du bloc, mesurée par la page
///   (`--holo-sticky-top`, `--holo-sticky-bottom`), sinon le cinquième de l'écran ; un bloc caché
///   par un `If` faux n'en demande pas ;
/// - le bouton rond du moteur monte au-dessus d'un bloc resté en bas (sauf quand les touches à
///   l'écran sont là : le bloc est alors au-dessus d'elles) ;
/// - le clavier de l'écran ouvert sur un champ (`holo-keyboard`, posé par la page) : le bloc
///   reprend sa place, pour ne pas cacher ce qu'on écrit.
const CSS: &str = "@media screen and (min-height:481px){\
.holo-Page [data-sticky]{position:sticky;z-index:3;max-height:20vh;max-height:20svh;overflow-y:auto}\
.holo-Page [data-sticky=top]{top:0}.holo-Page [data-sticky=bottom]{bottom:var(--holo-keys,0px)}\
:where(.holo-Page [data-sticky]){background:var(--holo-sticky-background,Canvas)}\
html:not(.holo-keyboard):has(.holo-Page [data-sticky=top]:not([hidden] *)){scroll-padding-top:calc(var(--holo-sticky-top,20svh) + 8px)}\
html:not(.holo-keyboard):has(.holo-Page [data-sticky=bottom]:not([hidden] *)){scroll-padding-bottom:calc(var(--holo-sticky-bottom,20svh) + var(--holo-keys,0px) + 8px)}\
html:not(:has(#keys:not([hidden]))) #menu{bottom:calc(12px + var(--holo-sticky-bottom,0px))}\
html.holo-keyboard .holo-Page [data-sticky]{position:static;max-height:none;overflow-y:visible}";

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
        if block.name == "Sound" && block.argument("label").is_none() {
            return refusal("« Sound(sticky: …) » : un son sans lecteur ne se voit pas ; donne-lui un lecteur, label: \"…\", pour qu'il reste à l'écran".into());
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
/// page (une couleur, ou une variable), aussi dans le thème sombre. Un fond de page qui n'est pas
/// une couleur (un dégradé, une image) : celui du navigateur, avec son texte (`Canvas`,
/// `CanvasText`), toujours lisible ensemble. Vide pour une page sans `sticky`.
pub fn css(program: &Program) -> String {
    let mut sticky = false;
    let _ = for_each_block(&program.root, &mut |block| {
        sticky |= sticky_of(block).is_some();
        Ok(())
    });
    if !sticky {
        return String::new();
    }
    let color = |settings: &[crate::holo::Setting]| {
        let value = settings.iter().find(|s| s.name == "background")?.value.trim().to_string();
        if crate::styles::is_color(&value) {
            Some(Some(value))
        } else if value.starts_with("--") && crate::styles::variables_of(&value) == [value.as_str()] {
            Some(Some(format!("var({value})")))
        } else {
            // Un dégradé, une image : pas une couleur.
            Some(None)
        }
    };
    let mut css = String::from(CSS);
    for rule in program.styles.iter().filter(|rule| matches!(&rule.target, Target::Type(t) if t == "Page")) {
        match color(&rule.settings) {
            Some(Some(value)) => css.push_str(&format!(".holo-Page{{--holo-sticky-background:{value}}}")),
            Some(None) => css.push_str(":where(.holo-Page [data-sticky]){color:CanvasText}"),
            None => {}
        }
        for (_, settings, _) in rule.states.iter().filter(|(state, ..)| state == "dark") {
            if let Some(Some(value)) = color(settings) {
                css.push_str(&format!("@media (prefers-color-scheme:dark){{.holo-Page{{--holo-sticky-background:{value}}}}}"));
            }
        }
    }
    css.push('}');
    css
}

#[cfg(test)]
mod tests {
    const ARTICLE: &str = r##"Page(
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
.up { background: #E9B44C; color: #101020; }"##;

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
        // Un essai écrit fait défiler la page : `scroll 95` (holo test).
        let played = crate::tools::play(ARTICLE, "scroll 95\nexpect scroll = 95\nexpect done = 1\nscroll 0\nexpect done = 1").unwrap();
        assert_eq!(played.failure, None);
        let refused = crate::tools::play("Page(children: [ P(\"x\") ])", "scroll 50").unwrap();
        assert!(refused.failure.is_some_and(|(line, message)| line == 1 && message.contains("ne lit pas « scroll »")));
        // Une page sans scroll ni sticky n'a rien de plus : ni valeur, ni style, ni moteur tout de suite.
        let plain = crate::flat_view("Page(children: [ P(\"x\") ])", "").unwrap();
        assert!(!plain.contains("scroll") && !plain.contains("data-sticky") && !plain.contains(" data-live"), "{plain}");
    }

    #[test]
    fn a_sticky_block_stays_on_screen_and_never_hides_the_focus() {
        let html = crate::flat_view(ARTICLE, "").unwrap();
        // Le bloc garde sa balise et sa place ; il est marqué, en haut ou en bas.
        assert!(html.contains(r#"<div data-sticky="top" class="holo-Row holo-s-bar""#), "{html}");
        assert!(html.contains(r##"<a data-sticky="bottom" class="holo-A holo-s-up" href="#Top">↑ Back to top</a>"##), "{html}");
        // Le style : seulement sur un écran assez haut ; la place, le cinquième de l'écran au plus,
        // le focus jamais caché, le bouton du moteur au-dessus du bloc du bas, le clavier ouvert
        // qui rend sa place au bloc.
        for rule in [
            "@media screen and (min-height:481px){",
            ".holo-Page [data-sticky]{position:sticky;z-index:3;max-height:20vh;max-height:20svh;overflow-y:auto}",
            ".holo-Page [data-sticky=bottom]{bottom:var(--holo-keys,0px)}",
            "{scroll-padding-top:calc(var(--holo-sticky-top,20svh) + 8px)}",
            "{scroll-padding-bottom:calc(var(--holo-sticky-bottom,20svh) + var(--holo-keys,0px) + 8px)}",
            "#menu{bottom:calc(12px + var(--holo-sticky-bottom,0px))}",
            "html.holo-keyboard .holo-Page [data-sticky]{position:static;",
        ] {
            assert!(html.contains(rule), "{rule}\n{html}");
        }
        // Le fond de la page, aussi dans le thème sombre, quand le bloc n'a pas le sien ; et la
        // règle du média se referme après lui.
        assert!(html.contains(".holo-Page{--holo-sticky-background:#101020}@media (prefers-color-scheme:dark){.holo-Page{--holo-sticky-background:var(--night)}}}"), "{html}");
        // Un fond de page en dégradé : le fond et le texte du navigateur, toujours lisibles ensemble.
        let gradient = crate::flat_view("Page(children: [ Header(sticky: top, children: [ Text(\"x\") ]) ])\nPage { background: linear-gradient(to bottom, #101020, #303060); color: white; }", "").unwrap();
        assert!(gradient.contains(":where(.holo-Page [data-sticky]){color:CanvasText}") && !gradient.contains("--holo-sticky-background:linear"), "{gradient}");
        // Header et Footer posés dans la page ; un bloc sous un If ou dans Main.
        let landmarks = "Page(children: [ Header(sticky: top, children: [ Text(\"Studio\") ]), Main(children: [ P(\"x\") ]), Footer(sticky: bottom, children: [ Text(\"©\") ]) ])";
        let html = crate::flat_view(landmarks, "").unwrap();
        assert!(html.contains(r#"<header data-sticky="top" class="holo-Header">"#) && html.contains(r#"<footer data-sticky="bottom" class="holo-Footer">"#), "{html}");
        assert!(!html.contains(".holo-Page{--holo-sticky-background"), "{html}");
        // Un bloc qui reste n'est pas une page vivante : le CSS suffit, le moteur attend un geste.
        assert!(!html.contains(" data-live"), "{html}");
        crate::check_page("Page(children: [ Main(children: [ If(n, is: 0, children: [ P(\"x\", sticky: top) ], else: [ P(\"y\") ]) ]) ], state: State(n: 0))").unwrap();
        // Un lecteur de son reste en bas, comme un lecteur de musique.
        crate::check_page("Page(children: [ Sound(source: \"a.mp3\", label: \"Le podcast\", sticky: bottom) ])").unwrap();
        // Un bloc qui bouge et qui reste : marqué sur son enveloppe de mouvement, qui tient sa place.
        let moving = crate::flat_view("Page(children: [ H1(\"x\", sticky: top, enter: Enter(y: 4px)) ])", "").unwrap();
        assert!(moving.contains(" data-sticky=\"top\""), "{moving}");
        // Un composant : le réglage est sur son bloc racine, et il se pose dans la page.
        let component = "Page(components: [ Component(name: Bar, children: [ Header(sticky: top, children: [ Text(\"x\") ]) ]) ], children: [ Bar() ])";
        assert!(crate::flat_view(component, "").unwrap().contains(r#"<header data-sticky="top""#));
        // Un monde seul n'a pas de bloc qui reste.
        assert!(crate::check("Point(name: W, seed: 1, inside: World(children: [ P(\"x\", sticky: top) ]))").unwrap_err().message.contains("posé directement dans la page"));
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
            ("Page(children: [ P(\"x\", sticky: true) ])", "attend top ou bottom"),
            ("Page(children: [ Row(children: [ P(\"x\", sticky: top) ]) ])", "posé directement dans la page"),
            ("Page(children: [ Header(children: [ Nav(sticky: top, children: [ A(\"x\", to: \"#X\") ]) ]), H1(\"X\", name: X) ])", "posé directement dans la page"),
            ("Page(state: State(n: 0), children: [ If(n, is: 0, sticky: top, children: [ P(\"x\") ]) ])", "un If ne reste pas à l'écran"),
            ("Page(children: [ Main(sticky: top, children: [ P(\"x\") ]) ])", "un Main ne reste pas à l'écran"),
            ("Page(children: [ Sound(name: Ding, source: \"a.mp3\", sticky: bottom) ])", "un son sans lecteur ne se voit pas"),
            ("Page(children: [ P(\"a\", sticky: top), P(\"b\", sticky: top) ])", "un seul bloc qui reste en haut"),
            ("Page(children: [ Point(name: P1, seed: 1, inside: World(children: [ P(\"x\", sticky: top) ])) ])", "posé directement dans la page"),
            ("Page(children: [ P(\"x\") ], rules: [ On(P.tap, sticky: top, effect: x.add(1)) ])", "n'a pas de paramètre « sticky »"),
            ("Page(components: [ Component(name: Bar, children: [ Text(\"x\") ]) ], children: [ Bar(sticky: top) ])", "écris « sticky » dans le composant"),
        ] {
            let error = crate::check_page(source).err().unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }
}
