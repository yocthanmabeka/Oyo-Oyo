//! La vue à plat : d'un fichier `.holo` vérifié, le moteur fabrique une page web ordinaire,
//! du HTML et du CSS (ADR-007, ADR-011). L'auteur n'en écrit jamais lui-même.
//!
//! Le monde à l'intérieur d'un point est rendu comme un panneau lisible, caché tant qu'on
//! n'y est pas entré (ADR-018, option A) : la page qui accueille le moteur le pose devant
//! la vue en profondeur.

use crate::holo::{Argument, Block, Target, Error, Program, Value};
use crate::rules::name_of;
use crate::styles::is_color;

/// La disposition et l'allure de base, que l'auteur n'a pas à écrire. `:where` laisse
/// toujours le dernier mot aux styles du fichier.
const BASE: &str = "\
:where(.holo-Page){min-height:100vh;box-sizing:border-box;margin:0;overflow-wrap:break-word}\
:where(.holo-Page>main,.holo-Page>header,.holo-Page>footer){display:block;max-width:var(--holo-width,640px);margin:0 auto;position:relative}\
:where(img.holo-Image){object-fit:cover}\
:where(.holo-Page>main,.holo-Page>header,.holo-Page>footer,.holo-panel,.holo-Header,.holo-Footer,.holo-Main)>*{display:block;box-sizing:border-box;margin:0 0 16px 0}\
:where(.holo-Nav)>*{margin:0}\
:where(.holo-Stack){display:inline-grid;position:relative;max-width:100%;vertical-align:top}:where(.holo-Stack>:first-child .holo-Image){width:100%;display:block}:where(.holo-Stack)>*{grid-area:1/1;min-width:0;margin:0}\
:where(.holo-stacked){z-index:1;margin:6px}\
:where(.holo-Button){font:inherit;color:inherit;cursor:pointer;background:transparent;border:1px solid currentColor;border-radius:6px;padding:6px 12px}\
:where(.holo-Point){width:64px;height:64px;padding:0;border:0;border-radius:50%;cursor:pointer;\
background:radial-gradient(circle,white 0%,var(--holo-color,white) 35%,transparent 70%);opacity:var(--holo-brightness,1)}\
:where(.holo-pixel){position:absolute;width:1px;height:1px;margin:0;padding:0;border:0;background:var(--holo-color,white);cursor:pointer}:where(.holo-World){position:fixed;inset:0;margin:0;pointer-events:none}\
:where(.holo-World[hidden]){display:none}\
:where(.holo-panel){position:absolute;z-index:1;left:0;right:0;bottom:0;max-height:46vh;overflow:auto;padding:16px max(16px,calc(50% - 320px));\
box-sizing:border-box;background:rgba(0,0,0,0.6);pointer-events:auto}\
:where(.holo-If){display:contents}:where(.holo-If)>*{display:block;box-sizing:border-box;margin:0 0 16px 0}\
:where(.holo-Row){display:flex;flex-wrap:wrap;align-items:center;gap:var(--holo-gap,1rem);justify-content:var(--holo-align,flex-start)}\
:where(.holo-Column){display:flex;flex-direction:column;gap:var(--holo-gap,1rem);align-items:var(--holo-align,stretch)}\
:where(.holo-Grid){display:grid;gap:var(--holo-gap,1rem);\
grid-template-columns:repeat(auto-fill,minmax(min(100%,max(7.5rem,calc((100% - (var(--holo-columns,2) - 1)*var(--holo-gap,1rem))/var(--holo-columns,2)))),1fr))}\
:where(.holo-Row,.holo-Column,.holo-Grid)>*{margin:0;box-sizing:border-box;min-width:0}\
:where(.holo-grow){display:flex;flex-direction:column;min-width:0}:where(.holo-grow)>*{flex:1 1 auto;margin:0}.holo-grow :is(input,textarea,select){width:100%;box-sizing:border-box}:where(.holo-grow .holo-Input){align-items:stretch}\
:where(.holo-Row,.holo-Column,.holo-Grid)>.holo-If>*{margin:0}:where(.holo-If[hidden]){display:none}\
:where(.holo-Input){display:flex;flex-direction:column;gap:4px;align-items:flex-start}\
:where(.holo-Input input){font:inherit;color:inherit;background:transparent;border:1px solid currentColor;border-radius:6px;padding:6px 10px;width:120px}\
:where(.holo-Input input[type=text]){width:min(100%,280px);box-sizing:border-box}:where(.holo-Input input[type=file]){width:min(100%,360px);box-sizing:border-box}\
:where(.holo-Checkbox){display:flex;align-items:center;gap:8px;cursor:pointer}\
:where(.holo-Checkbox input){width:18px;height:18px;margin:0;accent-color:currentColor}\
:where(.holo-Input textarea){font:inherit;color:inherit;background:transparent;border:1px solid currentColor;border-radius:6px;padding:6px 10px;width:min(100%,480px);box-sizing:border-box;resize:vertical}\
:where(.holo-Choice){border:0;padding:0;margin:0 0 16px 0;display:flex;flex-wrap:wrap;gap:8px 18px;align-items:center}\
:where(.holo-Choice legend){padding:0;margin:0 0 6px 0;width:100%}:where(.holo-Choice label){display:flex;gap:6px;align-items:center;cursor:pointer}\
:where(.holo-Choice input){accent-color:currentColor;width:18px;height:18px;margin:0}\
:where(label.holo-Choice){flex-direction:column;align-items:flex-start;gap:4px}:where(.holo-Choice select){font:inherit;color:inherit;background:transparent;border:1px solid currentColor;border-radius:6px;padding:6px 10px}\
:where(.holo-Video){display:block;width:100%;max-width:640px;border-radius:12px;background:black}\
:where(.holo-table-wrap){overflow-x:auto;max-width:100%}:where(.holo-Table){border-collapse:collapse;min-width:100%}\
:where(.holo-Table caption){text-align:left;font-weight:bold;padding:0 0 8px 0}:where(.holo-Table th,.holo-Table td){text-align:left;padding:8px 12px;border-bottom:1px solid color-mix(in srgb,currentColor 25%,transparent)}\
:where(.holo-Table th){font-weight:bold}\
:where(.holo-Board){position:relative;overflow:hidden;border-radius:12px;container-type:inline-size;margin-inline:auto}\
:where(.holo-positioned){position:absolute;left:calc(var(--x)*1%);top:calc(var(--y)*1%);transform:translate(calc(var(--x)*-1%),calc(var(--y)*-1%));\
transition:left .12s linear,top .12s linear,transform .12s linear}\
:where(.holo-positioned[data-drag]){touch-action:none;cursor:grab}.holo-positioned.holo-dragging{transition:none;cursor:grabbing}\
@media (prefers-reduced-motion:reduce){.holo-positioned{transition:none}.holo-Page,.holo-Page *{transition:none!important}}\
:where(.holo-Sound){display:none}:where(audio.holo-Sound[controls]){display:block;width:100%;max-width:480px}\
:where(.holo-figure){margin:0 0 16px 0}:where(.holo-figure figcaption){font-size:0.9em;opacity:0.8;margin-top:6px}:where(picture){display:contents}\
:where(.holo-Slider,.holo-Progress){display:flex;flex-direction:column;gap:4px;align-items:flex-start}:where(.holo-Slider input){width:min(100%,320px);accent-color:currentColor}\
:where(.holo-Progress progress){width:min(100%,320px);accent-color:currentColor}\
:where(.holo-Details summary){cursor:pointer;font-weight:bold}:where(.holo-Details[open] summary){margin-bottom:8px}\
:where(.holo-Dialog){max-width:min(90vw,480px);border:1px solid currentColor;border-radius:12px;padding:16px 20px;color:inherit;background:var(--fond,Canvas)}.holo-Dialog:not([open]){display:none}\
:where(.holo-Dialog)::backdrop{background:rgba(0,0,0,0.5)}:where(.holo-Dialog>*){margin:0 0 12px 0}:where(.holo-close){display:flex;justify-content:flex-end;margin:0}\
:where(.holo-close button){font:inherit;color:inherit;background:transparent;border:0;cursor:pointer;font-size:1.2em;line-height:1}\
:where(.holo-Lines,.holo-line){display:contents}:where(.holo-Form){display:block}:where(.holo-error){font-weight:bold;margin:4px 0 0 0}:where(.holo-error)::before{content:\"⚠ \"}:where([aria-invalid=true]){outline:2px solid currentColor;outline-offset:2px}:where(.holo-Form>*){box-sizing:border-box;margin:0 0 16px 0}:where(.holo-Form>:not(.holo-Input)){display:block}\
:where(.holo-Shape){display:block;width:var(--holo-size,48px);height:var(--holo-size,48px);padding:0;border:0;background:var(--holo-color,currentColor)}\
:where(button.holo-Shape){cursor:pointer}\
:where(.holo-forme-circle){border-radius:50%}\
:where(.holo-Board .holo-Shape){width:calc(var(--holo-n,48)*100cqw/640);height:calc(var(--holo-n,48)*100cqw/640)}\
:where(.holo-Board .holo-Point){width:10cqw;height:10cqw}\
:where(.holo-forme-triangle){clip-path:polygon(50% 0,100% 100%,0 100%)}\
:where(.holo-forme-diamond){clip-path:polygon(50% 0,100% 50%,50% 100%,0 50%)}\
:where(.holo-Hr){border:0;border-top:1px solid currentColor;opacity:0.4;height:0}\
:where(.holo-Quote){border-left:3px solid currentColor;padding:0 0 0 12px;font-style:italic}\
:where(.holo-Quote>p){margin:0 0 4px 0}:where(.holo-Quote>footer){font-style:normal;font-size:0.9em;opacity:0.7}\
:where(.holo-Code){font-family:ui-monospace,Consolas,monospace;background:rgba(127,127,127,0.18);padding:8px 12px;border-radius:6px;overflow:auto;white-space:pre-wrap}\
:where(.holo-Page code){font-family:ui-monospace,Consolas,monospace;background:rgba(127,127,127,0.18);padding:0 4px;border-radius:4px}:where(.holo-Code code){background:none;padding:0}\
:where(.holo-Aside){display:block;box-sizing:border-box;border-left:3px solid currentColor;padding:0 0 0 16px;margin:0 0 16px 0}:where(.holo-Aside)>*{display:block;margin:0 0 12px 0}:where(.holo-Drawing){display:block;max-width:100%;height:auto}:where(.holo-Chart){display:block;margin:0 0 16px 0}:where(.holo-Chart figcaption){font-weight:bold;margin:0 0 8px 0}:where(.holo-Chart svg){display:block;max-width:100%;height:auto}\
:where(.holo-hidden){position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap}:where(.holo-Stopwatch){display:block;font-variant-numeric:tabular-nums}\
@media print{:where(.holo-Dialog:not([open]),.holo-Video,audio){display:none!important}:where(.holo-Page){min-height:0}:where(.holo-Page a[href^=\"http\"])::after{content:\" (\" attr(href) \")\";font-size:.85em}}";

/// Entoure, dans le HTML en cours de fabrication, la condition d'un bloc `If` : `site_html`
/// la remplace par « hidden » quand elle est fausse au départ. Ce caractère ne peut pas venir
/// d'un texte de l'auteur : `echapper` le retire.
const MARK: char = '\u{1}';

/// Fabrique la page. `base` est le dossier du fichier `.holo`, pour retrouver ses images.
/// Le fichier doit avoir passé les vérifications (`crate::verifier_page`).
pub fn page_html(program: &Program, base: &str) -> Result<String, Error> {
    site_html(program, &program.root, base, "")
}

/// Fabrique la page d'un site. Un site est la `Page` du fichier, ou le monde contenu dans
/// l'un de ses points : ouvert en grand, ce monde se regarde exactement comme une page, avec
/// ses propres points, dans lesquels on peut entrer à leur tour. `titre` sert au monde, qui
/// n'en a pas.
pub fn site_html(program: &Program, page: &Block, base: &str, title: &str) -> Result<String, Error> {
    site_html_from(program, page, base, title, None)
}

/// Les valeurs d'où part une page : ses nombres, ses textes et ses listes.
pub type Start = (crate::state::State, crate::state::Texts, crate::lists::Lists);

/// Fabrique la page d'un site à partir de ces valeurs, ou de son départ (`None`). Le serveur
/// part des données qu'il a lues (ADR-064).
pub fn site_html_from(program: &Program, page: &Block, base: &str, title: &str, start: Option<&Start>) -> Result<String, Error> {
    // Les mouvements de la page deviennent du CSS, ajouté à son style (ADR-034).
    crate::movement::begin();
    let html = raw_site_html(program, page, base, title, start);
    let movements = crate::movement::finish();
    let html = html?;
    if movements.is_empty() && !html.contains("holo-Scene") {
        return Ok(html);
    }
    Ok(html.replacen("</style>", &format!("{}{movements}</style>", crate::movement::BASE), 1))
}

thread_local! {
    /// La langue de la page en cours : le texte caché d'un lien vers un nouvel onglet la suit (ADR-073).
    static LANGUAGE: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn set_language(program: &Program) {
    let language = match program.root.argument("lang").map(|a| &a.value) {
        Some(Value::Text(l)) => l.clone(),
        _ => "fr".to_string(),
    };
    LANGUAGE.with(|l| *l.borrow_mut() = language);
}

/// Ce que lit un lecteur d'écran après un lien qui s'ouvre dans un nouvel onglet.
fn new_tab_text() -> &'static str {
    LANGUAGE.with(|l| if l.borrow().starts_with("fr") || l.borrow().is_empty() { " (s'ouvre dans un nouvel onglet)" } else { " (opens in a new tab)" })
}

fn raw_site_html(program: &Program, page: &Block, base: &str, title: &str, start: Option<&Start>) -> Result<String, Error> {
    set_language(program);
    // Les valeurs à virgule (ADR-066) se montrent avec leurs chiffres.
    crate::format::set_decimals(crate::state::decimals(program));
    // Les valeurs d'où part la page : son départ, ou celles que le serveur a données.
    let (start_value, texts, mut lists) = match start {
        Some((numbers, texts, lists)) => (numbers.clone(), texts.clone(), lists.clone()),
        None => (crate::state::initial(program).unwrap_or_default(), crate::state::initial_texts(program), crate::lists::initial(program)),
    };
    // Les lignes de `Repeat(over:)` sont fabriquées d'après les listes, et les listes calculées.
    let (computed, totals) = crate::computed::apply_with_totals(program, &start_value, &texts, &lists);
    lists.extend(computed);
    crate::lists::set_running(lists);
    if page.name != "Page" && page.name != "World" {
        return Err(Error { message: format!("la vue à plat affiche une « Page » ; ce fichier commence par « {} »", page.name), pos: page.pos });
    }
    let mut body = String::new();
    let mut worlds = String::new();
    // Les repères (ADR-036) : un `Header` et un `Footer` posés directement dans la page en
    // sont l'en-tête et le pied, hors du contenu principal ; un `Main` dit où est ce contenu.
    let mut header = String::new();
    let mut footer = String::new();
    if let Some(Value::List(elements)) = page.argument("children").map(|a| &a.value) {
        for element in elements {
            match element {
                Value::Block(b) if b.name == "Header" => render(element, &mut header, &mut worlds, base, page)?,
                Value::Block(b) if b.name == "Footer" => render(element, &mut footer, &mut worlds, base, page)?,
                Value::Block(b) if b.name == "Main" => {
                    allowed_landmarks(b)?;
                    children(b, &mut body, &mut worlds, base)?;
                }
                _ => render(element, &mut body, &mut worlds, base, page)?,
            }
        }
    }
    if let Some(Value::List(planted_ones)) = page.argument("pixels").map(|a| &a.value) {
        for planted in planted_ones {
            planted_pixel(planted, &mut body, &mut worlds, base, page)?;
        }
    }
    let title = match page.argument("title").map(|a| &a.value) {
        Some(Value::Text(t)) => escape(t),
        _ => escape(title),
    };
    // Un monde ouvert en grand prend le thème de la page, puis son propre style.
    let classes = match page.name.as_str() {
        "World" => format!("holo-Page holo-world-open{}", style_names(page)),
        _ => classes(page),
    };
    // Les valeurs de la page, à leur départ, là où un texte les montre : « {cart} » (ADR-023).
    let mut shown = crate::state::to_show(program, &start_value);
    shown.extend(totals);
    shown.extend(crate::computed::days_values(program, &texts));
    // Une liste montre son nombre d'éléments, et une condition le compare (ADR-044).
    shown.extend(crate::lists::counts(&crate::lists::running()));
    // Les conditions, à leur départ : ce qui est faux est caché dès le premier affichage (ADR-025).
    // La réponse vient de `etat::conditions`, comme après chaque changement : une condition
    // n'est décidée qu'à un seul endroit.
    let responses = crate::state::conditions(program, &shown, &texts);
    let conditions = |html: String| fill_marks(html, &shown, &texts, &responses);
    body = conditions(body);
    worlds = conditions(worlds);
    header = conditions(header);
    footer = conditions(footer);
    // Les textes, à leur départ, là où un texte les montre ; une date aussi avec son format
    // (ADR-067) : `{arrival:date}` → « 7 octobre 2026 », dans la langue de la page.
    let page_language = match program.root.argument("lang").map(|a| &a.value) {
        Some(Value::Text(l)) => l.as_str(),
        _ => "fr",
    };
    for (name, text) in &texts {
        let mut places = vec![(format!("<span data-state=\"{name}\"></span>"), format!("<span data-state=\"{name}\">{}</span>", escape(text)))];
        for format in crate::dates::FORMATS {
            let opening = format!("<span data-state=\"{name}\" data-format=\"{format}\">");
            places.push((format!("{opening}</span>"), format!("{opening}{}</span>", escape(&crate::dates::format(text, format, page_language)))));
        }
        for (empty, full_one) in &places {
            body = body.replace(empty, full_one);
            worlds = worlds.replace(empty, full_one);
            header = header.replace(empty, full_one);
            footer = footer.replace(empty, full_one);
        }
    }
    // Les valeurs à format, à leur départ, dans la langue de la page.
    let language = match program.root.argument("lang").map(|a| &a.value) {
        Some(Value::Text(l)) => l.as_str(),
        _ => "fr",
    };
    body = crate::format::fill(&body, &shown, language);
    worlds = crate::format::fill(&worlds, &shown, language);
    header = crate::format::fill(&header, &shown, language);
    footer = crate::format::fill(&footer, &shown, language);
    for (name, value) in shown.clone() {
        let (empty, full_one) = (format!("<span data-state=\"{name}\"></span>"), format!("<span data-state=\"{name}\">{value}</span>"));
        body = body.replace(&empty, &full_one);
        worlds = worlds.replace(&empty, &full_one);
        header = header.replace(&empty, &full_one);
        footer = footer.replace(&empty, &full_one);
    }
    // Une page vivante bouge ou écoute sans qu'on la touche : une horloge, le clavier, des
    // données à recevoir, un bloc à faire glisser, des valeurs partagées. (Des valeurs gardées, `keep`, ne la rendent
    // pas vivante : la page légère regarde s'il y a vraiment quelque chose de gardé.) Le moteur doit
    // alors arriver tout de suite. Les autres pages s'affichent seules : le moteur n'est
    // téléchargé qu'au premier geste qui en a besoin (ADR-033).
    let live = !crate::state::clocks(program).is_empty()
        || !crate::state::delays(program, &start_value, &texts).is_empty()
        || crate::state::reads_time(program)
        || !crate::state::keypresses(program).is_empty()
        || crate::state::data_source(program).ok().flatten().is_some()
        || body.contains("data-drag=")
        // Une page qui partage des valeurs écoute le serveur, pour les voir changer en direct (ADR-079).
        || !program.shared.is_empty();
    let live = if live { " data-live" } else { "" };
    // Qui grossit la page quand on zoome (ADR-069) ? Par défaut, le navigateur, comme pour
    // n'importe quel site : la page reste à sa place. Le moteur, seulement si l'auteur l'a
    // écrit : des points, une page qui se réduit en un point, ou un zoom coupé. La page légère
    // le sait avant l'arrivée du moteur, pour laisser faire le navigateur ou non.
    let by_engine = crate::view::settings(program).is_ok_and(|r| r.active_points || r.reduce || !r.zoom_active);
    let live = format!("{live}{}", if by_engine { " data-zoom" } else { "" });
    // Un bloc qu'une règle écoute au survol le dit à la page (ADR-039) : la page légère fait
    // venir le moteur quand la souris arrive dessus.
    // Un bloc vers lequel un lien de la page mène (ADR-042) reçoit son nom comme `id` : le
    // navigateur y descend tout seul, sans le moteur.
    for name in crate::rules::anchors(program) {
        let single = format!(" data-name=\"{}\"", escape(&name));
        for html in [&mut body, &mut header, &mut footer, &mut worlds] {
            if let Some(place) = html.find(&single) {
                html.insert_str(place + single.len(), &format!(" id=\"{}\"", escape(&name)));
                break;
            }
        }
    }
    // Un bloc qui ne reçoit pas le focus de lui-même (une carte, un texte) le reçoit alors, pour
    // qu'on le survole aussi au clavier, avec Tab.
    for name in crate::rules::hovered_ones(program) {
        let single = format!(" data-name=\"{}\"", escape(&name));
        for html in [&mut body, &mut header, &mut footer, &mut worlds] {
            let Some(place) = html.find(&single) else { continue };
            let tag: String = html[..place].rsplit('<').next().unwrap_or("").chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
            let focus = if ["button", "a", "label", "fieldset", "video"].contains(&tag.as_str()) { "" } else { " tabindex=\"0\"" };
            html.replace_range(place..place + single.len(), &format!("{single} data-hover{focus}"));
        }
    }
    // La langue, la description et l'image de partage (ADR-038) : le serveur et le moteur les
    // reprennent dans l'en-tête de la page, pour les lecteurs d'écran, Google et les réseaux.
    let mut share = String::new();
    for (param, attribute) in [("lang", "data-lang"), ("description", "data-description"), ("image", "data-image"), ("icon", "data-icon")] {
        match page.argument(param).map(|a| &a.value) {
            None => {}
            Some(Value::Text(text)) if param == "lang" && is_language(text) => share.push_str(&format!(" {attribute}=\"{}\"", escape(text))),
            Some(Value::Text(text)) if param == "description" && text.chars().count() <= 300 => share.push_str(&format!(" {attribute}=\"{}\"", escape(text))),
            Some(Value::Text(text)) if param == "image" && path_on(text) => share.push_str(&format!(" {attribute}=\"{}{}\"", escape(base), escape(text))),
            // La petite image de l'onglet (ADR-042).
            Some(Value::Text(text)) if param == "icon" && path_on(text) && [".png", ".svg", ".ico"].iter().any(|end| text.ends_with(end)) => {
                share.push_str(&format!(" {attribute}=\"{}{}\"", escape(base), escape(text)))
            }
            Some(_) => {
                let pos = page.argument(param).map_or(page.pos, |a| a.pos);
                return Err(Error {
                    message: match param {
                        "lang" => "« Page(lang: …) » attend une langue, comme \"fr\", \"en\" ou \"fr-CA\"".into(),
                        "description" => "« Page(description: …) » attend un texte de 300 caractères au plus : ce que Google montre sous le titre".into(),
                        "icon" => "« Page(icon: …) » attend une petite image rangée à côté du fichier, en .png, .svg ou .ico : celle de l'onglet".into(),
                        _ => "« Page(image: …) » attend une image rangée à côté du fichier, comme \"partage.png\" : celle qu'on voit quand on partage le lien".into(),
                    },
                    pos,
                });
            }
        }
    }
    Ok(format!(
        "<style>{}{BASE}{}</style><div class=\"{classes}\" data-title=\"{title}\"{live}{share}>{header}<main>{body}</main>{footer}{worlds}</div>",
        fonts(&program.root, base)?,
        css(program, base)
    ))
}

/// Remplit les marques laissées dans le HTML en cours de fabrication : la condition d'un `If`,
/// le genre et la valeur d'un champ, une case cochée, une place sur un plateau.
fn fill_marks(html: String, shown: &crate::state::State, texts: &crate::state::Texts, responses: &[(String, bool)]) -> String {
    let start_text = |name: &str| texts.iter().find(|(known, _)| known == name).map(|(_, text)| text.as_str());
    {
        let mut output = String::with_capacity(html.len());
        for (rank, chunk) in html.split(MARK).enumerate() {
            if rank % 2 == 0 {
                output.push_str(chunk);
            } else if let Some(field) = chunk.strip_prefix('!') {
                // Un champ : de texte ou de nombre, selon la valeur qu'il présente.
                let mut parts = field.splitn(3, '|');
                let (name, max, min) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""), parts.next().unwrap_or(""));
                // Le plus petit nombre permis : 0, ou `min:` (lot 2 du web).
                let min = if min.is_empty() { "0" } else { min };
                if start_text(name).is_some() {
                    let length = max.parse::<usize>().map_or(crate::state::TEXT_SHORT, |m| m.min(crate::state::TEXT_MAX));
                    output.push_str(&format!(" type=\"text\" maxlength=\"{length}\""));
                    // La longueur la plus courte (ADR-068) : vérifiée à l'envoi.
                    if min != "0" {
                        output.push_str(&format!(" minlength=\"{min}\""));
                    }
                } else if crate::format::decimal_places(name) > 0 {
                    // Un nombre à virgule (ADR-066) : le clavier décimal, et un pas de 0,01.
                    let places = crate::format::decimal_places(name);
                    output.push_str(&format!(" type=\"number\" inputmode=\"decimal\" min=\"{min}\" step=\"{}\" data-places=\"{places}\"", crate::state::format_decimal(1, places)));
                    if !max.is_empty() {
                        output.push_str(&format!(" max=\"{max}\""));
                    }
                } else {
                    output.push_str(&format!(" type=\"number\" inputmode=\"numeric\" min=\"{min}\""));
                    if !max.is_empty() {
                        output.push_str(&format!(" max=\"{max}\""));
                    }
                }
            } else if let Some(choice) = chunk.strip_prefix('=') {
                // Un choix : l'option cochée (ou choisie) au départ est celle de la valeur.
                let mut parts = choice.splitn(3, '|');
                let (attribute, name, option) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""), parts.next().unwrap_or(""));
                if start_text(name).is_some_and(|t| escape(t) == option && !t.is_empty()) {
                    output.push_str(&format!(" {attribute}"));
                }
            } else if let Some(name) = chunk.strip_prefix('#') {
                // Un champ : la valeur de départ, telle quelle.
                match start_text(name) {
                    Some(text) => output.push_str(&escape(text)),
                    None => {
                        let units = shown.iter().find(|(known, _)| known == name).map_or(0, |(_, v)| *v);
                        output.push_str(&crate::state::format_decimal(units, crate::format::decimal_places(name)));
                    }
                }
            } else if let Some(name) = chunk.strip_prefix('?') {
                // Une case : cochée au départ si la valeur n'est pas zéro.
                if shown.iter().any(|(known, v)| known == name && *v > 0) {
                    output.push_str(" checked");
                }
            } else if let Some(name) = chunk.strip_prefix('%') {
                // Une mesure d'un dessin (ADR-086) : la valeur de départ, un nombre entier.
                let value = shown.iter().find(|(known, _)| known == name).map_or(0, |(_, v)| *v);
                output.push_str(&value.min(u64::from(crate::drawing::SIZE_MAX)).to_string());
            } else if let Some(name) = chunk.strip_prefix('&') {
                // Le temps d'un chronomètre (ADR-089), écrit dans la langue de la page.
                let value = shown.iter().find(|(known, _)| known == name).map_or(0, |(_, v)| *v);
                output.push_str(&crate::format::format_value(name, value, "stopwatch", &crate::format::language()));
            } else if let Some(name) = chunk.strip_prefix('@') {
                // La place d'un bloc sur un plateau : la valeur de départ, de 0 à 100.
                let value = shown.iter().find(|(known, _)| known == name).map_or(0, |(_, v)| *v);
                output.push_str(&value.min(100).to_string());
            } else if let Some(key) = chunk.strip_prefix('^') {
                // Le « sinon » d'une condition : caché quand elle est vraie (ADR-039).
                if responses.iter().any(|(known_one, real_one)| known_one == key && *real_one) {
                    output.push_str(" hidden");
                }
            } else if !responses.iter().any(|(key, real_one)| key == chunk && *real_one) {
                output.push_str(" hidden");
            }
        }
        output
    }
}

/// Le CSS des styles du fichier. Le thème d'abord, puis les types, puis les styles nommés :
/// à précision égale, le dernier écrit l'emporte, ce qui donne la priorité voulue (ADR-017).
/// Les polices de la page (ADR-041) : `fonts: [ Font(family: "Carlito", source: "carlito.woff2") ]`.
/// Le texte s'affiche tout de suite avec la police de secours, puis prend la sienne quand elle
/// arrive (`font-display: swap`) : jamais de texte invisible en attendant.
fn fonts(page: &Block, base: &str) -> Result<String, Error> {
    let Some(argument) = page.argument("fonts") else { return Ok(String::new()) };
    let example = "fonts: [ Font(family: \"Carlito\", source: \"carlito.woff2\") ]";
    let Value::List(list) = &argument.value else {
        return Err(Error { message: format!("« fonts » est une liste de polices : {example}"), pos: argument.pos });
    };
    if list.len() > 8 {
        return Err(Error { message: "une page charge au plus 8 polices".into(), pos: argument.pos });
    }
    let mut css = String::new();
    let mut families: Vec<&str> = Vec::new();
    for element in list {
        let Value::Block(font) = element else {
            return Err(Error { message: format!("« fonts » contient des « Font(…) » : {example}"), pos: argument.pos });
        };
        if font.name != "Font" {
            return Err(Error { message: format!("« fonts » contient des « Font(…) », pas des « {} »", font.name), pos: font.pos });
        }
        let (mut family, mut source) = (None, None);
        for a in &font.arguments {
            match (a.name.as_deref(), &a.value) {
                (Some("family"), Value::Text(t)) if !t.is_empty() && t.len() <= 40 && t.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-') => family = Some(t.as_str()),
                (Some("family"), _) => return Err(Error { message: "« Font(family: …) » attend le nom de la police entre guillemets : lettres, chiffres, espaces".into(), pos: a.pos }),
                (Some("source"), Value::Text(s)) if path_on(s) && [".woff2", ".woff", ".ttf", ".otf"].iter().any(|end| s.ends_with(end)) => source = Some(s.as_str()),
                (Some("source"), _) => return Err(Error { message: "« Font(source: …) » attend un fichier de police rangé à côté : .woff2, .woff, .ttf ou .otf".into(), pos: a.pos }),
                (Some(other), _) => return Err(Error { message: format!("« Font » n'a pas de paramètre « {other} » ; paramètres possibles : family, source"), pos: a.pos }),
                (None, _) => return Err(Error { message: format!("chaque paramètre de « Font » est nommé : {example}"), pos: a.pos }),
            }
        }
        let (Some(family), Some(source)) = (family, source) else {
            return Err(Error { message: format!("« Font » attend « family » et « source » : {example}"), pos: font.pos });
        };
        if families.contains(&family) {
            return Err(Error { message: format!("la police « {family} » est chargée deux fois"), pos: font.pos });
        }
        families.push(family);
        css.push_str(&format!("@font-face{{font-family:\"{family}\";src:url(\"{}{}\");font-display:swap}}", escape(base), escape(source)));
    }
    Ok(css)
}

fn css(program: &Program, base: &str) -> String {
    let rank = |target: &Target| match target {
        Target::Type(t) if t == "Page" || t == "World" => 0,
        Target::Type(_) => 1,
        Target::Name(_) => 2,
    };
    let mut rules: Vec<_> = program.styles.iter().collect();
    rules.sort_by_key(|r| rank(&r.target));
    let mut output = String::new();
    for rule in rules {
        let selector = match &rule.target {
            // Le style d'un monde vaut aussi quand ce monde est ouvert en grand.
            Target::Type(t) if t == "World" => ".holo-World,.holo-world-open".to_string(),
            Target::Type(t) if program.components.contains(t) => format!(".holo-c-{t}"),
            Target::Type(t) => format!(".holo-{t}"),
            Target::Name(n) => format!(".holo-s-{n}"),
        };
        output.push_str(&selector);
        output.push('{');
        // La largeur de la page (ADR-069) : `Page { max-width: 960px; }` élargit la colonne où
        // tout se range (640px sans rien écrire), par exemple sur un ordinateur.
        let page = matches!(&rule.target, Target::Type(t) if t == "Page");
        let declaration = |setting: &crate::holo::Setting| match setting.name.as_str() {
            "max-width" if page => format!("--holo-width:{};", css_value(setting, base)),
            name => format!("{name}:{};", css_value(setting, base)),
        };
        for setting in &rule.settings {
            output.push_str(&declaration(setting));
        }
        // Un style qui change au survol ou à l'appui passe d'un aspect à l'autre en douceur,
        // à moins que l'auteur n'ait dit sa propre durée (`transition:`).
        if rule.states.iter().any(|(state, ..)| state == "hover" || state == "active") && !rule.settings.iter().any(|r| r.name == "transition") {
            output.push_str("transition:background .15s,color .15s,border-color .15s,opacity .15s,box-shadow .15s,scale .15s,rotate .15s;");
        }
        output.push('}');
        // Les états (ADR-036). Le survol n'existe qu'avec une souris : sur un écran tactile, il
        // resterait collé après un toucher. Le focus est celui du clavier. Le thème sombre suit
        // le choix du visiteur ; « phone » vaut pour un écran plus étroit que la page (ADR-041) ;
        // « computer » pour un écran large, 1024px ou plus ; « narrow » pour une case de grille
        // de moins de 320px, quel que soit l'écran : la place que le bloc reçoit (ADR-069). La
        // page marque elle-même ces cases (`holo-narrow`) : en CSS, une case ne peut pas se
        // mesurer elle-même, seulement ce qu'elle contient, et l'auteur s'y tromperait.
        for (state, settings, _) in &rule.states {
            let body: String = settings.iter().map(&declaration).collect();
            let rule_state = match state.as_str() {
                "hover" => format!("@media (hover:hover){{{}:hover{{{body}}}}}", selector.split(',').map(str::to_string).collect::<Vec<_>>().join(":hover,")),
                "focus" => format!("{}:focus-visible{{{body}}}", selector.split(',').collect::<Vec<_>>().join(":focus-visible,")),
                "dark" => format!("@media (prefers-color-scheme:dark){{{selector}{{{body}}}}}"),
                "phone" => format!("@media (max-width:{AUTHOR_WIDTH}px){{{selector}{{{body}}}}}"),
                "computer" => format!("@media (min-width:{COMPUTER_WIDTH}px){{{selector}{{{body}}}}}"),
                "narrow" => format!("{}{{{body}}}", selector.split(',').map(|s| format!("{s}.holo-narrow,.holo-narrow {s}")).collect::<Vec<_>>().join(",")),
                // Sur papier (ADR-073) : `print: { display: none; }` cache un bloc à l'impression.
                "print" => format!("@media print{{{selector}{{{body}}}}}"),
                _ => format!("{}:active{{{body}}}", selector.split(',').collect::<Vec<_>>().join(":active,")),
            };
            output.push_str(&rule_state);
        }
    }
    output
}

/// La largeur de page pour laquelle un auteur écrit ses tailles (`main` fait 640px au plus).
const AUTHOR_WIDTH: f64 = 640.0;
/// Un écran d'ordinateur, ou de tablette couchée : 1024px de large ou plus (ADR-069).
const COMPUTER_WIDTH: f64 = 1024.0;

/// La valeur d'un réglage, telle que le navigateur la reçoit. Une taille de texte écrite en
/// pixels suit le réglage « texte plus grand » du visiteur (en rem : 16px = 1rem) ; un grand
/// titre rétrécit sur un écran plus étroit que la page, sans jamais passer sous 24px (ADR-036).
fn css_value(setting: &crate::holo::Setting, base: &str) -> String {
    let mut value = setting.value.replace('<', "");
    // Une variable (ADR-041) : `--or` devient `var(--or)` ; sa définition reste telle quelle.
    for variable in crate::styles::variables_of(&setting.value) {
        value = value.replacen(variable, &format!("var({variable})"), 1);
    }
    // Une image de fond couvre toujours le bloc, centrée, sans se répéter en mosaïque.
    if setting.name == "background" {
        if let Some(image) = crate::styles::background_image(&setting.value) {
            return format!("url(\"{}{}\") center/cover no-repeat", escape(base), escape(image));
        }
    }
    // Une durée de passage : elle vaut pour tout ce qui change d'allure.
    if setting.name == "transition" && value != "none" {
        return ["background", "color", "border-color", "opacity", "box-shadow", "scale", "rotate", "letter-spacing"].iter().map(|p| format!("{p} {value}")).collect::<Vec<_>>().join(",");
    }
    // Une taille écrite en pixels suit le réglage « texte plus grand » du visiteur, comme le texte
    // (ADR-061) : 16px = 1rem. Les traits, les ombres et l'écart entre les lettres restent en pixels.
    if setting.name == "height" && value == "screen" {
        // Tout l'écran, au moins : la hauteur visible sur un téléphone (sans la barre d'adresse,
        // le défaut de 100vh), et le bloc grandit si son contenu est plus long.
        return "auto;min-height:100vh;min-height:100dvh".into();
    }
    // Le texte justifié coupe aussi les mots en fin de ligne, dans la langue de la page : sans
    // cela, des trous s'ouvrent entre les mots sur un écran étroit, le défaut du CSS (ADR-069).
    if setting.name == "text-align" && value == "justify" {
        return "justify;hyphens:auto;-webkit-hyphens:auto".into();
    }
    // Couper un texte après quelques lignes, avec « … » : un seul réglage, quand le CSS en
    // demande quatre ensemble (ADR-069).
    if setting.name == "line-clamp" {
        return format!("{value};-webkit-line-clamp:{value};display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden");
    }
    // Un curseur dessiné : le navigateur exige une forme de secours, sans quoi il ignore tout ;
    // le moteur l'ajoute (ADR-069).
    if setting.name == "cursor" {
        if let Some(image) = crate::styles::cursor_image(&setting.value) {
            return format!("url(\"{}{}\"),auto", escape(base), escape(image));
        }
    }
    if ["padding", "margin", "width", "max-width", "height", "border-radius", "min-width", "min-height", "max-height"].contains(&setting.name.as_str()) {
        return value.split(' ').map(to_rem).collect::<Vec<_>>().join(" ");
    }
    if setting.name != "font-size" {
        return value;
    }
    let Some(px) = value.strip_suffix("px").and_then(|n| n.trim().parse::<f64>().ok()) else { return value };
    let rem = px / 16.0;
    if px <= 24.0 {
        return format!("{}rem", rounded(rem));
    }
    format!("clamp(1.5rem,{}vw,{}rem)", rounded(px * 100.0 / AUTHOR_WIDTH), rounded(rem))
}

/// `24px` → `1.5rem` ; le reste ne change pas.
fn to_rem(word: &str) -> String {
    match word.strip_suffix("px").and_then(|n| n.parse::<f64>().ok()) {
        Some(px) if px == 0.0 => "0".to_string(),
        Some(px) => format!("{}rem", rounded(px / 16.0)),
        None => word.to_string(),
    }
}

fn rounded(n: f64) -> String {
    let n = (n * 1000.0).round() / 1000.0;
    if n.fract() == 0.0 { format!("{n:.0}") } else { format!("{n}") }
}

/// Une langue, comme « fr », « en » ou « fr-CA ».
fn is_language(text: &str) -> bool {
    let (language, region) = text.split_once('-').unwrap_or((text, ""));
    (2..=3).contains(&language.len()) && language.chars().all(|c| c.is_ascii_lowercase()) && (region.is_empty() || (region.len() == 2 && region.chars().all(|c| c.is_ascii_uppercase())))
}

/// Un repère ne prend que des enfants et un nom.
fn allowed_landmarks(block: &Block) -> Result<(), Error> {
    for argument in &block.arguments {
        match argument.name.as_deref() {
            Some("name" | "children" | "enter" | "loop") => {}
            Some(other) => return Err(Error { message: format!("« {} » n'a pas de paramètre « {other} » ; paramètres possibles : children, name", block.name), pos: argument.pos }),
            None => return Err(Error { message: format!("« {0} » range des blocs : {0}(children: [ … ])", block.name), pos: argument.pos }),
        }
    }
    Ok(())
}

/// Les places d'un bloc posé dans un `Stack`.
const CORNERS: &[&str] = &["topLeft", "top", "topRight", "left", "center", "right", "bottomLeft", "bottom", "bottomRight"];

/// `topRight` → (haut, côté) en CSS.
fn corner(word: &str) -> Option<(&'static str, &'static str)> {
    Some(match word {
        "topLeft" => ("start", "start"),
        "top" => ("start", "center"),
        "topRight" => ("start", "end"),
        "left" => ("center", "start"),
        "center" => ("center", "center"),
        "right" => ("center", "end"),
        "bottomLeft" => ("end", "start"),
        "bottom" => ("end", "center"),
        "bottomRight" => ("end", "end"),
        _ => return None,
    })
}

fn classes(block: &Block) -> String {
    format!("holo-{}{}", block.name, style_names(block))
}

/// Les classes des noms de style d'un bloc : ` holo-s-card` ; la marque d'un composant (un nom
/// qui commence par une majuscule, posé par le moteur) : ` holo-c-ArticleCard` (ADR-050).
fn style_names(block: &Block) -> String {
    block.styles.iter().map(|s| if s.starts_with(|c: char| c.is_ascii_uppercase()) { format!(" holo-c-{s}") } else { format!(" holo-s-{s}") }).collect()
}

fn children(block: &Block, output: &mut String, worlds: &mut String, base: &str) -> Result<(), Error> {
    if let Some(Value::List(elements)) = block.argument("children").map(|a| &a.value) {
        for element in elements {
            render(element, output, worlds, base, block)?;
        }
    }
    Ok(())
}

fn render(value: &Value, output: &mut String, worlds: &mut String, base: &str, parent: &Block) -> Result<(), Error> {
    let block = match value {
        // Une phrase seule est un paragraphe (ADR-019, ADR-020).
        Value::Text(text) => {
            output.push_str(&format!("<p class=\"holo-P\">{}</p>", markdown(text)));
            return Ok(());
        }
        Value::Block(block) => block,
        _ => return Err(Error { message: "« children » contient des blocs ou des phrases entre guillemets".into(), pos: parent.pos }),
    };
    // Un bloc qui bouge (enter:, loop:) : on le fabrique sans ses mouvements, puis on
    // l'enveloppe dans eux (ADR-034).
    if let Some((movements, remainder)) = crate::movement::of_block(block)? {
        let mut inside = String::new();
        let children = match remainder.argument("children").map(|a| &a.value) {
            Some(Value::List(list)) => list.len(),
            _ => 0,
        };
        render(&Value::Block(remainder), &mut inside, worlds, base, parent)?;
        crate::movement::wrap(&movements, inside, children, output);
        return Ok(());
    }
    // Un bloc rangé dans Row ou Column qui prend la place qui reste, comme Expanded en Flutter
    // (ADR-052) : `grow: 1`, ou plus pour en prendre une plus grande part.
    if let Some(argument) = block.argument("grow") {
        if parent.name != "Row" && parent.name != "Column" {
            return Err(Error { message: format!("« grow: » fait grandir un bloc rangé dans Row ou Column : mets « {} » dans Row(children: [ … ])", block.name), pos: argument.pos });
        }
        let Value::Integer(part @ 1..=12) = argument.value else {
            return Err(Error { message: "« grow: » attend un nombre entier de 1 à 12 : la part de la place qui reste".into(), pos: argument.pos });
        };
        let mut remainder = block.clone();
        remainder.arguments.retain(|a| a.name.as_deref() != Some("grow"));
        output.push_str(&format!("<div class=\"holo-grow\" style=\"flex:{part} 1 0\">"));
        render(&Value::Block(remainder), output, worlds, base, parent)?;
        output.push_str("</div>");
        return Ok(());
    }
    let classes = classes(block);
    let name = name_of(block).map(|n| format!(" data-name=\"{}\"", escape(n))).unwrap_or_default();
    match block.name.as_str() {
        // Des scènes qui s'enchaînent, l'une après l'autre, au même endroit (ADR-034).
        "Scenes" => {
            let mut height = 480.0;
            let mut always = false;
            for argument in &block.arguments {
                match (argument.name.as_deref(), &argument.value) {
                    (Some("name" | "children"), _) => {}
                    (Some("height"), Value::Number { value, unit: Some(unit), .. }) if unit == "px" && (80.0..=2000.0).contains(value) => height = *value,
                    (Some("height"), _) => return Err(Error { message: "« Scenes(height: …) » attend une hauteur entre 80px et 2000px".into(), pos: argument.pos }),
                    (Some("repeat"), Value::Name(word)) if word == "forever" => always = true,
                    (Some("repeat"), _) => return Err(Error { message: "« Scenes(repeat: …) » attend « forever » : les scènes recommencent sans fin".into(), pos: argument.pos }),
                    (Some(other), _) => return Err(Error { message: format!("« Scenes » n'a pas de paramètre « {other} » ; paramètres possibles : children, height, repeat, name"), pos: argument.pos }),
                    (None, _) => return Err(Error { message: "« Scenes » range des scènes : Scenes(children: [ Scene(for: 3s, children: [ … ]) ])".into(), pos: argument.pos }),
                }
            }
            let mut scenes = Vec::new();
            if let Some(Value::List(elements)) = block.argument("children").map(|a| &a.value) {
                for element in elements {
                    let scene = match element {
                        Value::Block(scene) if scene.name == "Scene" => scene,
                        _ => return Err(Error { message: "« Scenes » ne range que des « Scene(for: …, children: [ … ]) »".into(), pos: block.pos }),
                    };
                    let mut duration = None;
                    for argument in &scene.arguments {
                        match (argument.name.as_deref(), &argument.value) {
                            (Some("name" | "children"), _) => {}
                            (Some("for"), value) => {
                                duration = Some(crate::movement::scene_duration(value).ok_or_else(|| Error { message: "« Scene(for: …) » attend une durée de 200ms à 600s, comme for: 4s".into(), pos: argument.pos })?)
                            }
                            (Some(other), _) => return Err(Error { message: format!("« Scene » n'a pas de paramètre « {other} » ; paramètres possibles : for, children, name"), pos: argument.pos }),
                            (None, _) => return Err(Error { message: "« Scene » range des blocs : Scene(for: 3s, children: [ … ])".into(), pos: argument.pos }),
                        }
                    }
                    let duration = duration.ok_or_else(|| Error { message: "« Scene » dit combien de temps elle dure : Scene(for: 3s, children: [ … ])".into(), pos: scene.pos })?;
                    scenes.push((scene, duration));
                }
            }
            let turn = always.then(|| scenes.iter().map(|(_, d)| d).sum::<f64>());
            output.push_str(&format!("<div class=\"{classes}\"{name} style=\"height:{height}px\">"));
            let mut start = 0.0;
            let last = scenes.len().saturating_sub(1);
            for (rank, (scene, duration)) in scenes.into_iter().enumerate() {
                let scene_name = crate::movement::new_scene(start, duration, turn, rank == last);
                let name = name_of(scene).map(|n| format!(" data-name=\"{}\"", escape(n))).unwrap_or_default();
                output.push_str(&format!("<div class=\"{} {scene_name}\"{name}>", self::classes(scene)));
                crate::movement::in_scene(start, turn, || children(scene, output, worlds, base))?;
                output.push_str("</div>");
                start += duration;
            }
            output.push_str("</div>");
        }
        // Les repères, pour qui navigue avec un lecteur d'écran (ADR-036).
        "Nav" | "Header" | "Footer" | "Aside" => {
            allowed_landmarks(block)?;
            let tag = block.name.to_ascii_lowercase();
            output.push_str(&format!("<{tag} class=\"{classes}\"{name}>"));
            children(block, output, worlds, base)?;
            output.push_str(&format!("</{tag}>"));
        }
        "Main" => return Err(Error { message: "« Main » se place directement dans la page : Page(children: [ Header(…), Main(children: [ … ]), Footer(…) ])".into(), pos: block.pos }),
        // La superposition (ADR-036) : le premier enfant donne la taille ; les autres se posent
        // dessus, chacun à sa place (align:), comme un badge sur une image.
        "Stack" => {
            for argument in &block.arguments {
                match argument.name.as_deref() {
                    Some("name" | "children") => {}
                    Some(other) => return Err(Error { message: format!("« Stack » n'a pas de paramètre « {other} » ; paramètres possibles : children, name"), pos: argument.pos }),
                    None => return Err(Error { message: "« Stack » superpose des blocs : Stack(children: [ Image(…), Text(\"Promo\", align: topRight) ])".into(), pos: argument.pos }),
                }
            }
            output.push_str(&format!("<div class=\"{classes}\"{name}>"));
            if let Some(Value::List(elements)) = block.argument("children").map(|a| &a.value) {
                for (rank, element) in elements.iter().enumerate() {
                    let (placed, style) = match element {
                        Value::Block(child) => match child.argument("align") {
                            Some(argument) if child.name != "Row" && child.name != "Column" => {
                                let (top, side) = match &argument.value {
                                    Value::Name(word) if word.contains('_') && corner(&crate::state::in_flutter(word)).is_some() => {
                                        return Err(Error { message: format!("« {word} » : deux mots se joignent comme en Flutter ; écris « {} » (ADR-037)", crate::state::in_flutter(word)), pos: argument.pos })
                                    }
                                    Value::Name(word) => corner(word).ok_or_else(|| Error { message: format!("« align: » dans « Stack » attend l'une de ces places : {}", CORNERS.join(", ")), pos: argument.pos })?,
                                    _ => return Err(Error { message: format!("« align: » dans « Stack » attend l'une de ces places : {}", CORNERS.join(", ")), pos: argument.pos }),
                                };
                                let mut remainder = child.clone();
                                remainder.arguments.retain(|a| a.name.as_deref() != Some("align"));
                                (Value::Block(remainder), format!(" class=\"holo-stacked\" style=\"align-self:{top};justify-self:{side}\""))
                            }
                            _ => (element.clone(), if rank == 0 { String::new() } else { " class=\"holo-stacked\" style=\"align-self:center;justify-self:center\"".to_string() }),
                        },
                        _ => (element.clone(), String::new()),
                    };
                    output.push_str(&format!("<div{style}>"));
                    render(&placed, output, worlds, base, block)?;
                    output.push_str("</div>");
                }
            }
            output.push_str("</div>");
        }
        "Scene" => return Err(Error { message: "« Scene » se range dans des scènes : Scenes(children: [ Scene(for: 3s, children: [ … ]) ])".into(), pos: block.pos }),
        "H1" | "H2" | "H3" | "H4" | "H5" | "H6" | "P" | "Text" => {
            let tag = match block.name.as_str() {
                "P" => "p".to_string(),
                "Text" => "div".to_string(),
                title => title.to_ascii_lowercase(),
            };
            output.push_str(&format!("<{tag} class=\"{classes}\"{name}>{}</{tag}>", markdown(text_of(block)?)));
        }
        "Button" => {
            let text = match block.argument("text").map(|a| &a.value) {
                Some(Value::Text(t)) => t,
                _ => return Err(Error { message: "« Button » attend un paramètre « text » entre guillemets".into(), pos: block.pos }),
            };
            output.push_str(&format!("<button type=\"button\" class=\"{classes}\"{name}>{}</button>", markdown(text)));
        }
        "Image" => {
            let source = match block.argument("source").map(|a| &a.value) {
                Some(Value::Text(s)) if path_on(s) => s,
                _ => {
                    return Err(Error {
                        message: "« Image » attend un paramètre « source » : un fichier rangé à côté du .holo, comme \"painting.png\"".into(),
                        pos: block.pos,
                    })
                }
            };
            // `alt` : le texte qui remplace l'image pour qui ne la voit pas. Il est obligatoire
            // (ADR-038) : pour un simple décor, on l'écrit vide, alt: "", et un lecteur d'écran
            // la passe. On ne l'oublie plus sans le savoir.
            let alt = match block.argument("alt").map(|a| &a.value) {
                None => return Err(Error { message: "« Image » attend « alt » : ce que montre l'image, pour qui ne la voit pas ; pour un simple décor, alt: \"\"".into(), pos: block.pos }),
                Some(Value::Text(text)) => text.as_str(),
                Some(_) => return Err(Error { message: "« Image(alt: …) » attend un texte entre guillemets : ce que montre l'image".into(), pos: block.pos }),
            };
            // Les images plus bas dans la page ne viennent qu'en approchant de l'écran (ADR-073) ;
            // la première de chaque partie vient tout de suite, elle est souvent visible d'emblée.
            let later = if output.contains("<img") { " loading=\"lazy\" decoding=\"async\"" } else { "" };
            let mut image = format!("<img class=\"{classes}\"{name} src=\"{}{}\" alt=\"{}\"{later}>", escape(base), escape(source), escape(alt));
            // Une image plus légère pour un téléphone (ADR-042) : le navigateur ne télécharge que
            // celle qu'il montre.
            match block.argument("phone").map(|a| &a.value) {
                None => {}
                Some(Value::Text(small_one)) if path_on(small_one) => {
                    image = format!("<picture><source media=\"(max-width:{AUTHOR_WIDTH}px)\" srcset=\"{}{}\">{image}</picture>", escape(base), escape(small_one))
                }
                Some(_) => return Err(Error { message: "« Image(phone: …) » attend une image plus légère, rangée à côté : phone: \"photo-petite.jpg\"".into(), pos: block.pos }),
            }
            // Une légende, sous l'image.
            match block.argument("caption").map(|a| &a.value) {
                None => output.push_str(&image),
                Some(Value::Text(caption)) => output.push_str(&format!("<figure class=\"holo-figure\">{image}<figcaption>{}</figcaption></figure>", markdown(caption))),
                Some(_) => return Err(Error { message: "« Image(caption: …) » attend un texte entre guillemets : la légende".into(), pos: block.pos }),
            }
        }
        "List" => {
            // `ordered: true` : une liste numérotée.
            let tag = if matches!(block.argument("ordered").map(|a| &a.value), Some(Value::Bool(true))) { "ol" } else { "ul" };
            output.push_str(&format!("<{tag} class=\"{classes}\"{name}>"));
            if let Some(Value::List(elements)) = block.argument("children").map(|a| &a.value) {
                for element in elements {
                    output.push_str("<li>");
                    match element {
                        Value::Text(text) => output.push_str(&markdown(text)),
                        other => render(other, output, worlds, base, block)?,
                    }
                    output.push_str("</li>");
                }
            }
            output.push_str(&format!("</{tag}>"));
        }
        // Une condition : ce qu'elle contient ne se montre que si elle est vraie (ADR-025).
        "If" => {
            if block.argument("rules").is_some() {
                return Err(Error { message: "un « If » qui range des règles se place dans « rules », pas dans « children »".into(), pos: block.pos });
            }
            let (value, comparisons) = crate::state::condition(block)?;
            let key = crate::state::key(value, &comparisons);
            output.push_str(&format!("<div class=\"{classes}\"{name} data-if=\"{}\"{MARK}{key}{MARK}>", escape(&key)));
            children(block, output, worlds, base)?;
            output.push_str("</div>");
            // Le « sinon » : ce qui se montre quand la condition est fausse (ADR-039).
            if let Some(Value::List(otherwise)) = block.argument("else").map(|a| &a.value) {
                output.push_str(&format!("<div class=\"holo-If\" data-else=\"{}\"{MARK}^{key}{MARK}>", escape(&key)));
                for element in otherwise {
                    render(element, output, worlds, base, block)?;
                }
                output.push_str("</div>");
            }
        }
        // Un champ où le visiteur écrit un nombre, et une case qu'il coche. Chacun présente une
        // valeur de la page ; l'étiquette est obligatoire (ADR-027). `etat.rs` les a vérifiés.
        "Input" | "Checkbox" => {
            let (Some(Value::Name(value)), Some(Value::Text(label))) = (block.argument("value").map(|a| &a.value), block.argument("label").map(|a| &a.value)) else {
                return Err(Error { message: format!("« {} » attend « value » et « label »", block.name), pos: block.pos });
            };
            let value = escape(value);
            if block.name == "Input" && block.argument("lines").is_some() {
                // Un texte long : plusieurs lignes, ses retours à la ligne gardés (ADR-038).
                let lines = match block.argument("lines").map(|a| &a.value) {
                    Some(Value::Integer(n)) => *n,
                    _ => 4,
                };
                let max = match block.argument("max").map(|a| &a.value) {
                    Some(Value::Integer(max)) => (*max as usize).min(crate::state::TEXT_MAX),
                    _ => crate::state::TEXT_LONG,
                };
                output.push_str(&format!(
                    "<label class=\"{classes}\"{name}><span>{}</span><textarea rows=\"{lines}\" maxlength=\"{max}\"{} data-bind=\"{value}\">{MARK}#{value}{MARK}</textarea></label>",
                    markdown(label),
                    checked_by_form(block)
                ));
            } else if crate::files::is_file(block) {
                // Un fichier (ADR-059) : la page vérifie sa sorte et sa taille avant l'envoi.
                let kinds = crate::files::kinds(block)?;
                let max = crate::files::max_size(block)?;
                output.push_str(&format!(
                    "<label class=\"{classes}\"{name}><span>{}</span><input type=\"file\" accept=\"{}\" data-max=\"{max}\" data-bind=\"{value}\"></label>",
                    markdown(label),
                    crate::files::accept_html(&kinds)
                ));
            } else if let (true, Some(Value::Name(kind))) = (block.name == "Input", block.argument("type").map(|a| &a.value)) {
                // Une date, une heure, une couleur : le navigateur montre son propre choisisseur (ADR-042).
                // Une date a ses bornes : `min: today`, `max: "2026-12-31"` (ADR-067).
                let mut bounds = String::new();
                if kind == "date" {
                    for bound in ["min", "max"] {
                        if let Some(day) = crate::dates::bound(block.argument(bound).map(|a| &a.value)) {
                            bounds.push_str(&format!(" {bound}=\"{day}\""));
                        }
                    }
                }
                // Un e-mail (ADR-068) : le clavier des adresses, et le navigateur propose la sienne.
                if kind == "email" {
                    let max = match block.argument("max").map(|a| &a.value) {
                        Some(Value::Integer(max)) => (*max as usize).min(254),
                        _ => 254,
                    };
                    bounds.push_str(&format!(" inputmode=\"email\" autocomplete=\"email\" maxlength=\"{max}\""));
                }
                output.push_str(&format!(
                    "<label class=\"{classes}\"{name}><span>{}</span><input type=\"{}\"{bounds}{} value=\"{MARK}#{value}{MARK}\" data-bind=\"{value}\"></label>",
                    markdown(label),
                    escape(kind),
                    checked_by_form(block)
                ));
            } else if block.name == "Input" {
                // Les bornes d'un champ de nombre, écrites comme dans le fichier : 99.99, 1.
                let written = |param: &str| match block.argument(param).map(|a| &a.value) {
                    Some(Value::Integer(n)) => n.to_string(),
                    Some(Value::Number { value, unit: None, places }) => format!("{value:.prec$}", prec = *places as usize),
                    _ => String::new(),
                };
                let (max, min) = (written("max"), written("min"));
                output.push_str(&format!(
                    "<label class=\"{classes}\"{name}><span>{}</span><input{MARK}!{value}|{max}|{min}{MARK}{} value=\"{MARK}#{value}{MARK}\" data-bind=\"{value}\"></label>",
                    markdown(label),
                    checked_by_form(block)
                ));
            } else {
                output.push_str(&format!(
                    "<label class=\"{classes}\"{name}><input type=\"checkbox\"{} data-bind=\"{value}\"{MARK}?{value}{MARK}><span>{}</span></label>",
                    checked_by_form(block),
                    markdown(label)
                ));
            }
        }
        // Un choix : des boutons ronds (radio), ou une liste déroulante avec menu: true (ADR-038).
        "Choice" => {
            let (Some(Value::Name(value)), Some(Value::Text(label))) = (block.argument("value").map(|a| &a.value), block.argument("label").map(|a| &a.value)) else {
                return Err(Error { message: "« Choice » attend « value », « label » et « options »".into(), pos: block.pos });
            };
            let value = escape(value);
            let options = crate::state::choice_options(block);
            if matches!(block.argument("menu").map(|a| &a.value), Some(Value::Bool(true))) {
                output.push_str(&format!("<label class=\"{classes}\"{name}><span>{}</span><select{} data-bind=\"{value}\"><option value=\"\">—</option>", markdown(label), checked_by_form(block)));
                for option in options {
                    let o = escape(option);
                    output.push_str(&format!("<option value=\"{o}\"{MARK}=selected|{value}|{o}{MARK}>{o}</option>"));
                }
                output.push_str("</select></label>");
            } else {
                // Des boutons ronds obligatoires : le groupe le dit à un lecteur d'écran (ADR-068).
                let group = if checked_by_form(block).is_empty() { "" } else { " role=\"radiogroup\" aria-required=\"true\"" };
                output.push_str(&format!("<fieldset class=\"{classes}\"{name}{group} data-group=\"{value}\"><legend>{}</legend>", markdown(label)));
                for option in options {
                    let o = escape(option);
                    output.push_str(&format!("<label><input type=\"radio\" name=\"choix-{value}\" value=\"{o}\" data-bind=\"{value}\"{MARK}=checked|{value}|{o}{MARK}><span>{o}</span></label>"));
                }
                output.push_str("</fieldset>");
            }
        }
        // Une vidéo : avec ses commandes, jamais lancée toute seule (ADR-038).
        "Video" => {
            let source = match block.argument("source").map(|a| &a.value) {
                Some(Value::Text(s)) if path_on(s) && (s.ends_with(".mp4") || s.ends_with(".webm")) => s,
                _ => return Err(Error { message: "« Video » attend « source » : une vidéo rangée à côté du fichier, en .mp4 ou .webm, comme \"film.mp4\"".into(), pos: block.pos }),
            };
            let Some(Value::Text(label)) = block.argument("label").map(|a| &a.value) else {
                return Err(Error { message: "« Video » attend « label » : ce que montre la vidéo, pour qui ne la voit pas".into(), pos: block.pos });
            };
            // Des sous-titres (ADR-073) : un fichier WebVTT rangé à côté, montrés d'emblée.
            let track = match block.argument("captions").map(|a| &a.value) {
                None => String::new(),
                Some(Value::Text(file)) if path_on(file) && file.ends_with(".vtt") => {
                    let language = LANGUAGE.with(|l| l.borrow().clone());
                    let words = if language.starts_with("fr") || language.is_empty() { "Sous-titres" } else { "Captions" };
                    format!("<track kind=\"captions\" src=\"{}{}\" srclang=\"{}\" label=\"{words}\" default>", escape(base), escape(file), escape(if language.is_empty() { "fr" } else { &language }))
                }
                Some(_) => return Err(Error { message: "« Video(captions: …) » attend un fichier de sous-titres rangé à côté, en .vtt : captions: \"film.vtt\"".into(), pos: block.pos }),
            };
            output.push_str(&format!(
                "<video class=\"{classes}\"{name} src=\"{}{}\" controls preload=\"metadata\" playsinline aria-label=\"{}\">{track}</video>",
                escape(base),
                escape(source),
                escape(label)
            ));
        }
        // Un tableau de données : une légende, une ligne de titres, des lignes (ADR-038).
        "Table" => {
            let line = |value: &Value, pos| -> Result<Vec<String>, Error> {
                match value {
                    Value::List(cells) => cells.iter().map(|c| match c {
                        Value::Text(t) => Ok(markdown(t)),
                        _ => Err(Error { message: "une case de « Table » est un texte entre guillemets".into(), pos }),
                    }).collect(),
                    _ => Err(Error { message: "une ligne de « Table » s'écrit entre crochets : [\"Lundi\", \"9 h – 18 h\"]".into(), pos }),
                }
            };
            let head = match block.argument("head") {
                Some(argument) => Some(line(&argument.value, argument.pos)?),
                None => None,
            };
            let Some(lines_argument) = block.argument("rows") else {
                return Err(Error { message: "« Table » attend « rows » : rows: [ [\"Lundi\", \"9 h\"], [\"Mardi\", \"9 h\"] ]".into(), pos: block.pos });
            };
            let Value::List(rows) = &lines_argument.value else {
                return Err(Error { message: "« Table(rows: …) » est une liste de lignes : rows: [ [\"Lundi\", \"9 h\"] ]".into(), pos: lines_argument.pos });
            };
            let width = head.as_ref().map(Vec::len);
            output.push_str(&format!("<div class=\"holo-table-wrap\"><table class=\"{classes}\"{name}>"));
            if let Some(Value::Text(caption)) = block.argument("caption").map(|a| &a.value) {
                output.push_str(&format!("<caption>{}</caption>", markdown(caption)));
            }
            if let Some(head) = &head {
                output.push_str("<thead><tr>");
                for cell in head {
                    output.push_str(&format!("<th scope=\"col\">{cell}</th>"));
                }
                output.push_str("</tr></thead>");
            }
            output.push_str("<tbody>");
            for row in rows {
                let cells = line(row, lines_argument.pos)?;
                if width.is_some_and(|l| l != cells.len()) {
                    return Err(Error { message: format!("chaque ligne de « Table » a autant de cases que « head » ({}) ; celle-ci en a {}", width.unwrap_or(0), cells.len()), pos: lines_argument.pos });
                }
                output.push_str("<tr>");
                for cell in cells {
                    output.push_str(&format!("<td>{cell}</td>"));
                }
                output.push_str("</tr>");
            }
            output.push_str("</tbody></table></div>");
        }
        // Un plateau : ce qu'il contient se place où l'on veut, par x et y, de 0 à 100 (ADR-026).
        "Board" => {
            let mut height = 320.0;
            for argument in &block.arguments {
                match (argument.name.as_deref(), &argument.value) {
                    (Some("name" | "children"), _) => {}
                    (Some("height"), Value::Number { value, unit: Some(unit), .. }) if unit == "px" && (80.0..=800.0).contains(value) => height = *value,
                    (Some("height"), _) => return Err(Error { message: "« Board(height: …) » attend une taille entre 80px et 800px".into(), pos: argument.pos }),
                    (Some(other), _) => return Err(Error { message: format!("« Board » n'a pas de paramètre « {other} » ; paramètres possibles : name, children, height"), pos: argument.pos }),
                    (None, _) => return Err(Error { message: "« Board » range des blocs : Board(children: [ … ])".into(), pos: argument.pos }),
                }
            }
            // Un plateau garde ses proportions : 640 de large, `height` de haut. Il s'agrandit ou
            // rétrécit avec l'écran, et ce qu'il contient avec lui : une partie est la même sur
            // un téléphone et sur un grand écran. Il ne dépasse pas les quatre cinquièmes de la
            // hauteur de l'écran.
            let width = crate::state::BOARD_WIDTH;
            let wider = 80.0 * width / height;
            output.push_str(&format!("<div class=\"{classes}\"{name} style=\"aspect-ratio:{width}/{height};width:min(100%,{wider:.1}vh)\">"));
            if let Some(Value::List(elements)) = block.argument("children").map(|a| &a.value) {
                for element in elements {
                    match element {
                        Value::Block(placed) if placed.argument("x").is_some() || placed.argument("y").is_some() => {
                            let (attribute_x, x) = place(placed, "x")?;
                            let (attribute_y, y) = place(placed, "y")?;
                            // `drag: true` : le visiteur peut faire glisser ce bloc, et ses valeurs suivent.
                            let dragging = match (placed.argument("drag").map(|a| &a.value), name_of(placed)) {
                                (None | Some(Value::Bool(false)), _) => String::new(),
                                (Some(Value::Bool(true)), Some(name)) if !attribute_x.is_empty() || !attribute_y.is_empty() => format!(" data-drag=\"{}\"", escape(name)),
                                _ => {
                                    return Err(Error {
                                        message: "« drag: true » demande un bloc qui a un nom, et dont « x » ou « y » est une valeur de la page : Point(name: Basket, x: basket, y: 96, drag: true)".into(),
                                        pos: placed.pos,
                                    })
                                }
                            };
                            output.push_str(&format!("<div class=\"holo-positioned\"{attribute_x}{attribute_y}{dragging} style=\"--x:{x};--y:{y}\">"));
                            render(element, output, worlds, base, block)?;
                            output.push_str("</div>");
                        }
                        other => render(other, output, worlds, base, block)?,
                    }
                }
            }
            output.push_str("</div>");
        }
        // Un graphique (ADR-087) : une liste à champs, en barres, en courbe ou en parts.
        "Chart" => output.push_str(&crate::chart::html(block, &classes, &name, &crate::lists::running())?),
        // Un dessin vectoriel (ADR-086) : des formes, fabriquées en SVG.
        "Drawing" => output.push_str(&crate::drawing::html(block, &classes, &name, &crate::lists::running())?),
        "Rect" | "Circle" | "Line" | "Path" => {
            return Err(Error { message: format!("« {} » se dessine dans un Drawing : Drawing(label: \"…\", width: 200, height: 100, children: [ {}(…) ])", block.name, block.name), pos: block.pos })
        }
        // Un chronomètre (ADR-089) : la page le dessine au rythme de l'écran quand une règle le
        // démarre ; ici, le temps gardé, ou zéro. Le lecteur d'écran ne l'annonce pas à chaque
        // centième (`role="timer"`), seulement son temps final.
        "Stopwatch" => {
            let label = match block.argument("label").map(|a| &a.value) {
                Some(Value::Text(t)) => t.clone(),
                _ if crate::format::language().starts_with("fr") || crate::format::language().is_empty() => "Chronomètre".to_string(),
                _ => "Stopwatch".to_string(),
            };
            let (value, shown) = match block.argument("value").map(|a| &a.value) {
                Some(Value::Name(v)) => (v.as_str(), format!("{MARK}&{v}{MARK}")),
                _ => ("", crate::format::format_value("", 0, "stopwatch", &crate::format::language())),
            };
            output.push_str(&format!("<span class=\"{classes}\"{name} role=\"timer\" aria-label=\"{}\" data-stopwatch=\"{}\">{shown}</span>", escape(&label), escape(value)));
        }
        // Une forme simple, d'une seule couleur : un rond, un carré, un triangle, un losange.
        "Shape" => {
            let (mut shape, mut pace) = (None, String::new());
            for argument in &block.arguments {
                match (argument.name.as_deref(), &argument.value) {
                    (Some("name" | "x" | "y" | "drag"), _) => {}
                    (Some("form"), Value::Name(word)) if ["circle", "square", "triangle", "diamond"].contains(&word.as_str()) => shape = Some(word.as_str()),
                    (Some("form"), _) => return Err(Error { message: "« Shape(form: …) » attend l'un de ces mots : circle, square, triangle, diamond".into(), pos: argument.pos }),
                    (Some("color"), Value::Text(color)) if is_color(color) => pace.push_str(&format!("--holo-color:{color};")),
                    (Some("color"), _) => return Err(Error { message: "« Shape(color: …) » attend une couleur entre guillemets, comme \"#E9B44C\"".into(), pos: argument.pos }),
                    (Some("size"), Value::Number { value, unit: Some(unit), .. }) if unit == "px" && (8.0..=400.0).contains(value) => pace.push_str(&format!("--holo-size:{value}px;--holo-n:{value};")),
                    (Some("size"), _) => return Err(Error { message: "« Shape(size: …) » attend une taille entre 8px et 400px".into(), pos: argument.pos }),
                    (Some(other), _) => return Err(Error { message: format!("« Shape » n'a pas de paramètre « {other} » ; paramètres possibles : form, color, size, name"), pos: argument.pos }),
                    (None, _) => return Err(Error { message: "chaque paramètre de « Shape » est nommé : Shape(form: circle, color: \"#E9B44C\", size: 48px)".into(), pos: argument.pos }),
                }
            }
            let Some(shape) = shape else {
                return Err(Error { message: "« Shape » attend « form » : Shape(form: circle)".into(), pos: block.pos });
            };
            // Une forme qui a un nom peut être touchée : c'est un vrai bouton, qu'on atteint au
            // clavier et qu'un lecteur d'écran nomme. Sans nom, c'est un dessin.
            match name_of(block) {
                Some(n) => output.push_str(&format!("<button type=\"button\" class=\"{classes} holo-forme-{shape}\"{name} aria-label=\"{}\" style=\"{pace}\"></button>", escape(n))),
                None => output.push_str(&format!("<div class=\"{classes} holo-forme-{shape}\" style=\"{pace}\"></div>")),
            }
        }
        // Un son, qu'une règle fait entendre : Ding.play. Il ne se voit pas.
        "Sound" => {
            let mut source = None;
            // Le volume et la boucle (ADR-061) : `volume: 0.4`, `loop: true`.
            let mut settings = String::new();
            for argument in &block.arguments {
                match (argument.name.as_deref(), &argument.value) {
                    (Some("name" | "weight"), _) | (Some("label"), Value::Text(_)) => {}
                    (Some("volume"), Value::Number { value, unit: None, .. }) if (0.0..=1.0).contains(value) => settings.push_str(&format!(" data-volume=\"{value}\"")),
                    (Some("volume"), Value::Integer(i)) if *i <= 1 => settings.push_str(&format!(" data-volume=\"{i}\"")),
                    (Some("volume"), _) => return Err(Error { message: "« Sound(volume: …) » attend un nombre de 0 (muet) à 1 (le plus fort), comme l'opacité : volume: 0.4".into(), pos: argument.pos }),
                    (Some("loop"), Value::Bool(true)) => settings.push_str(" loop"),
                    (Some("loop"), Value::Bool(false)) => {}
                    (Some("loop"), _) => return Err(Error { message: "« Sound(loop: …) » attend true ou false ; un son qui boucle s'arrête par « Rain.stop »".into(), pos: argument.pos }),
                    (Some("source"), Value::Text(s)) if path_on(s) && [".wav", ".mp3", ".ogg"].iter().any(|end| s.ends_with(end)) => source = Some(s),
                    (Some("source"), _) => {
                        return Err(Error { message: "« Sound(source: …) » attend un fichier de son rangé à côté du .holo : \"ding.wav\" (.wav, .mp3 ou .ogg)".into(), pos: argument.pos })
                    }
                    (Some(other), _) => return Err(Error { message: format!("« Sound » n'a pas de paramètre « {other} » ; paramètres possibles : name, source, label, volume, loop, weight"), pos: argument.pos }),
                    (None, _) => return Err(Error { message: "chaque paramètre de « Sound » est nommé : Sound(name: Ding, source: \"ding.wav\")".into(), pos: argument.pos }),
                }
            }
            // Avec une étiquette, le son est un lecteur, avec ses boutons, jamais lancé seul (ADR-042).
            if let (Some(source), Some(Value::Text(label))) = (source, block.argument("label").map(|a| &a.value)) {
                output.push_str(&format!("<audio class=\"{classes}\"{name} controls preload=\"metadata\" src=\"{}{}\" aria-label=\"{}\"{settings}></audio>", escape(base), escape(source), escape(label)));
                return Ok(());
            }
            let (Some(source), false) = (source, name.is_empty()) else {
                return Err(Error { message: "un son a un nom, pour qu'une règle puisse le jouer, et un fichier : Sound(name: Ding, source: \"ding.wav\") ; avec label:, c'est un lecteur".into(), pos: block.pos });
            };
            output.push_str(&format!("<audio class=\"{classes}\"{name} preload=\"auto\" src=\"{}{}\"{settings}></audio>", escape(base), escape(source)));
        }
        // Un trait de séparation.
        "Hr" => {
            if let Some(argument) = block.arguments.iter().find(|a| a.name.as_deref() != Some("name")) {
                return Err(Error { message: "« Hr » est un trait de séparation : il s'écrit « Hr() »".into(), pos: argument.pos });
            }
            output.push_str(&format!("<hr class=\"{classes}\"{name}>"));
        }
        // Une citation, avec son auteur si on le donne : Quote("…", by: "…").
        "Quote" => {
            let author = match block.argument("by").map(|a| &a.value) {
                None => String::new(),
                Some(Value::Text(author)) => format!("<footer>— {}</footer>", markdown(author)),
                Some(_) => return Err(Error { message: "« Quote(by: …) » attend un texte entre guillemets : qui l'a dit".into(), pos: block.pos }),
            };
            output.push_str(&format!("<blockquote class=\"{classes}\"{name}><p>{}</p>{author}</blockquote>", markdown(text_of(block)?)));
        }
        // Du texte montré tel quel, lettre pour lettre : un code, une commande, une adresse.
        "Code" => output.push_str(&format!("<pre class=\"{classes}\"{name}><code>{}</code></pre>", escape(text_of(block)?))),
        // La disposition : côte à côte, l'un sous l'autre, en grille (ADR-024).
        "Row" | "Column" | "Grid" => {
            output.push_str(&format!("<div class=\"{classes}\"{name} style=\"{}\">", layout(block)?));
            children(block, output, worlds, base)?;
            output.push_str("</div>");
        }
        // Le lien classique : on quitte la page pour une autre adresse, comme <a href> en HTML.
        "A" => {
            let address = match block.argument("to").map(|a| &a.value) {
                Some(Value::Text(address)) => safe_address(address, base),
                _ => None,
            };
            let Some(address) = address else {
                return Err(Error {
                    message: "« A » attend un paramètre « to » : un fichier rangé à côté (\"garden.holo\") ou une adresse du web (\"https://…\")".into(),
                    pos: block.pos,
                });
            };
            let flag = |p: &str| -> Result<bool, Error> {
                match block.argument(p) {
                    None => Ok(false),
                    Some(Argument { value: Value::Bool(b), .. }) => Ok(*b),
                    Some(argument) => Err(Error { message: format!("« A({p}: …) » attend true ou false : A(\"…\", to: \"…\", {p}: true)"), pos: argument.pos }),
                }
            };
            let (new_tab, download) = (flag("newTab")?, flag("download")?);
            let mut extra = String::new();
            let mut after = String::new();
            // Un nouvel onglet (ADR-073) : le lecteur d'écran l'annonce, et la page ouverte ne peut
            // pas toucher à celle-ci (noopener).
            if new_tab {
                extra.push_str(" target=\"_blank\" rel=\"noopener\"");
                after = format!("<span class=\"holo-hidden\">{}</span>", new_tab_text());
            }
            // Un téléchargement : seulement un fichier rangé à côté, pas une page ni une adresse du web.
            if download {
                let Some(Value::Text(file)) = block.argument("to").map(|a| &a.value) else { unreachable!("« to » vérifié plus haut") };
                if !path_on(file) || file.ends_with(".holo") || file.contains('#') {
                    return Err(Error { message: "« A(download: true) » télécharge un fichier rangé à côté, comme \"catalogue.pdf\" ; pas une page .holo ni une adresse du web".into(), pos: block.pos });
                }
                if new_tab {
                    return Err(Error { message: "« A » télécharge ou ouvre un nouvel onglet, pas les deux : garde download: true ou newTab: true".into(), pos: block.pos });
                }
                extra.push_str(" download");
            }
            output.push_str(&format!("<a class=\"{classes}\"{name} href=\"{}\"{extra}>{}{after}</a>", escape(&address), markdown(text_of(block)?)));
        }
        // Une liste qui change pendant la visite (ADR-044) : une ligne par élément ; la page les
        // redessine quand la liste change.
        "Repeat" => {
            let Some(Value::Name(list)) = block.argument("over").map(|a| &a.value) else {
                return Err(Error { message: "« Repeat » a été déplié à la lecture ; ici, il attend « over: » : Repeat(over: tasks, children: [ … ])".into(), pos: block.pos });
            };
            // Où elle est écrite dans le fichier : deux répétitions d'une même liste se redessinent
            // chacune avec son propre modèle (avant : la seconde recevait les lignes de la première).
            output.push_str(&format!("<div class=\"holo-Lines\" data-list=\"{}\" data-repeat=\"{}:{}\">", escape(list), block.pos.line, block.pos.column));
            output.push_str(&lines(block, list, base)?);
            output.push_str("</div>");
        }
        // Une glissière : choisir un nombre entre deux bornes (ADR-042).
        "Slider" => {
            let (Some(Value::Name(value)), Some(Value::Text(label))) = (block.argument("value").map(|a| &a.value), block.argument("label").map(|a| &a.value)) else {
                return Err(Error { message: "« Slider » attend « value » et « label »".into(), pos: block.pos });
            };
            let bound = |p: &str, default_value: u64| match block.argument(p).map(|a| &a.value) {
                Some(Value::Integer(n)) => *n,
                _ => default_value,
            };
            let value = escape(value);
            output.push_str(&format!(
                "<label class=\"{classes}\"{name}><span>{}</span><input type=\"range\" min=\"{}\" max=\"{}\" value=\"{MARK}#{value}{MARK}\" data-bind=\"{value}\"></label>",
                markdown(label),
                bound("min", 0),
                bound("max", 100)
            ));
        }
        // Une barre de progression : une jauge de vie, un téléchargement (ADR-042).
        "Progress" => {
            let Some(Value::Text(label)) = block.argument("label").map(|a| &a.value) else {
                return Err(Error { message: "« Progress » attend « label » : ce que mesure la barre".into(), pos: block.pos });
            };
            let max = match block.argument("max").map(|a| &a.value) {
                Some(Value::Integer(n)) => *n,
                _ => 100,
            };
            let (link, start_value) = match block.argument("value").map(|a| &a.value) {
                Some(Value::Name(v)) => (format!(" data-progress=\"{}\"", escape(v)), format!("{MARK}#{}{MARK}", escape(v))),
                Some(Value::Integer(n)) => (String::new(), n.min(&max).to_string()),
                _ => return Err(Error { message: "« Progress » attend « value » : un nombre de la page, ou un nombre".into(), pos: block.pos }),
            };
            output.push_str(&format!("<label class=\"{classes}\"{name}><span>{}</span><progress max=\"{max}\" value=\"{start_value}\"{link}></progress></label>", markdown(label)));
        }
        // Un pli qui s'ouvre : une question, sa réponse (ADR-042). Il marche sans le moteur.
        "Details" => {
            let Some(Value::Text(summary)) = block.argument("summary").map(|a| &a.value) else {
                return Err(Error { message: "« Details » attend « summary » : ce qu'on voit fermé, Details(summary: \"Livrez-vous ?\", children: [ … ])".into(), pos: block.pos });
            };
            let opened = match block.argument("open").map(|a| &a.value) {
                None | Some(Value::Bool(false)) => "",
                Some(Value::Bool(true)) => " open",
                Some(_) => return Err(Error { message: "« Details(open: …) » attend true ou false".into(), pos: block.pos }),
            };
            output.push_str(&format!("<details class=\"{classes}\"{name}{opened}><summary>{}</summary>", markdown(summary)));
            children(block, output, worlds, base)?;
            output.push_str("</details>");
        }
        // Une fenêtre par-dessus la page (ADR-042) : une règle l'ouvre (Confirm.open) ; la croix,
        // la touche Échap ou une règle (Confirm.close) la ferment.
        "Dialog" => {
            if name.is_empty() {
                return Err(Error { message: "« Dialog » a un nom, pour qu'une règle l'ouvre : Dialog(name: Confirm, children: [ … ]), puis On(Ask.tap, effect: Confirm.open)".into(), pos: block.pos });
            }
            output.push_str(&format!("<dialog class=\"{classes}\"{name}><form method=\"dialog\" class=\"holo-close\"><button aria-label=\"Fermer\">✕</button></form>"));
            children(block, output, worlds, base)?;
            output.push_str("</dialog>");
        }
        // Un formulaire qu'on envoie (ADR-042) : une règle l'envoie (Contact.send) ; il dit ensuite
        // si l'envoi est arrivé (Contact.sent) ou non (Contact.failed).
        "Form" => {
            if name.is_empty() {
                return Err(Error { message: "« Form » a un nom, pour qu'une règle l'envoie : Form(name: Contact, children: [ … ]), puis On(Send.tap, effect: Contact.send)".into(), pos: block.pos });
            }
            if let Some(Value::List(inside)) = block.argument("children").map(|a| &a.value) {
                let mut nested = false;
                for value in inside {
                    if let Value::Block(child) = value {
                        let _ = crate::rules::for_each_block(child, &mut |b| {
                            nested |= b.name == "Form";
                            Ok(())
                        });
                    }
                }
                if nested {
                    return Err(Error { message: "un formulaire dans un formulaire n'est pas permis".into(), pos: block.pos });
                }
            }
            output.push_str(&format!("<form class=\"{classes}\"{name} novalidate>"));
            children(block, output, worlds, base)?;
            output.push_str("</form>");
        }
        "Point" => {
            let pace = point_pace(block)?;
            let label = name_of(block).map(|n| format!(" aria-label=\"{}\"", escape(n))).unwrap_or_default();
            // `inside: "garden.holo"` : le monde de ce point est un autre fichier. On y passe
            // sans changer de page ; la page d'entrée va le chercher.
            let file = match block.argument("inside").map(|a| &a.value) {
                Some(Value::Text(file)) if path_on(file) && file.ends_with(".holo") => format!(" data-file=\"{}{}\"", escape(base), escape(file)),
                // Le fichier d'un autre auteur, sur un autre serveur : son adresse complète.
                Some(Value::Text(address)) if passage_address(address) && address.ends_with(".holo") => format!(" data-file=\"{}\"", escape(address)),
                Some(Value::Text(_)) => {
                    return Err(Error {
                        message: "« inside » attend un monde, un fichier .holo rangé à côté (\"garden.holo\"), ou l'adresse complète d'un fichier .holo en https (\"https://…/garden.holo\")".into(),
                        pos: block.pos,
                    })
                }
                _ => String::new(),
            };
            output.push_str(&format!("<button type=\"button\" class=\"{classes}\"{name}{label}{file} style=\"{pace}\"></button>"));
            inner_world(block, worlds, base)?;
        }
        other => return Err(Error { message: format!("« {other} » ne se place pas dans « children »"), pos: block.pos }),
    }
    Ok(())
}

/// Le caractère qui tient la place de l'élément d'une ligne pendant la fabrication. Il ne peut
/// pas venir d'un texte du visiteur : un texte saisi est nettoyé de ses caractères invisibles.
const ELEMENT: char = '\u{2}';
/// La marque d'un champ d'élément dans une ligne : `\u{3}0\u{3}` pour le premier champ montré.
const FIELD: char = '\u{3}';

/// Les lignes d'une répétition dynamique, pour les éléments en cours de sa liste. Le texte de
/// l'élément est posé après la fabrication, échappé : ce qu'un visiteur a écrit ne devient jamais
/// une balise, ni du gras, ni une valeur montrée.
fn lines(repeat: &Block, list: &str, base: &str) -> Result<String, Error> {
    for argument in &repeat.arguments {
        match argument.name.as_deref() {
            Some("over" | "children" | "rules" | "name") => {}
            // La clé choisie par l'auteur : un champ de l'élément, key: id (lot 2 du web).
            Some("key") if matches!(argument.value, Value::Name(_)) => {}
            Some("key") => return Err(Error { message: "« Repeat(key: …) » attend le nom d'un champ des éléments, comme key: id".into(), pos: argument.pos }),
            // Ce qui s'écrit quand la liste est vide (lot 2 du web) : « Aucun résultat ».
            Some("empty") if matches!(argument.value, Value::Text(_)) => {}
            Some("empty") => return Err(Error { message: "« Repeat(empty: …) » attend un texte entre guillemets : empty: \"Aucun résultat\"".into(), pos: argument.pos }),
            Some(other) => return Err(Error { message: format!("« Repeat(over: …) » n'a pas de paramètre « {other} » ; paramètres possibles : over, key, empty, children, rules"), pos: argument.pos }),
            None => return Err(Error { message: "chaque paramètre de « Repeat » est nommé : Repeat(over: tasks, children: [ … ])".into(), pos: argument.pos }),
        }
    }
    let Some(Value::List(model)) = repeat.argument("children").map(|a| &a.value) else {
        return Err(Error { message: "« Repeat(over: …) » attend « children » : le modèle d'une ligne".into(), pos: repeat.pos });
    };
    let lists = crate::lists::running();
    let Some((_, elements)) = lists.iter().find(|(name, _)| name == list) else {
        return Err(Error { message: format!("« Repeat(over: {list}) » : aucune liste ne s'appelle « {list} » ; déclare-la, state: State({list}: [])"), pos: repeat.pos });
    };
    // Chaque élément est posé dans le modèle après la fabrication, échappé : `{item}` et
    // `{item.title}` deviennent des marques, remplacées par le texte de l'élément ou du champ ;
    // `item.image` à la place d'une valeur devient le champ (ADR-051).
    fn mark_rows(value: &mut Value, fields: &[(String, String)], shown_ones: &mut Vec<(String, Option<String>)>) {
        match value {
            Value::Text(t) => {
                let mut output = String::new();
                let mut remainder = t.as_str();
                while let Some(start) = remainder.find("{item") {
                    output.push_str(&remainder[..start]);
                    let after = &remainder[start..];
                    let Some(end) = after.find('}') else {
                        output.push_str(after);
                        remainder = "";
                        break;
                    };
                    let inside = &after[1..end];
                    if inside == "item" {
                        output.push(ELEMENT);
                    } else if let Some(field) = inside.strip_prefix("item.") {
                        let (name, format) = field.split_once(':').map_or((field, None), |(n, f)| (n, Some(f.to_string())));
                        output.push_str(&format!("{FIELD}{}{FIELD}", shown_ones.len()));
                        shown_ones.push((name.to_string(), format));
                    } else {
                        output.push_str(&after[..=end]);
                    }
                    remainder = &after[end + 1..];
                }
                output.push_str(remainder);
                *t = output;
            }
            Value::Name(n) if n.starts_with("item.") => {
                let field = &n["item.".len()..];
                if let Some((_, v)) = fields.iter().find(|(c, _)| c == field) {
                    *value = match v.parse::<u64>() {
                        Ok(e) if !v.starts_with('0') || v == "0" => Value::Integer(e),
                        _ => Value::Text(v.clone()),
                    };
                }
            }
            Value::List(l) => l.iter_mut().for_each(|v| mark_rows(v, fields, shown_ones)),
            // Une image dont le fichier vient de l'élément, mais qu'on ne peut pas montrer (hors du
            // dossier de la page, ou absent) : la ligne garde le texte de remplacement de l'image,
            // au lieu de disparaître avec toute la liste (défaut D10 de l'exploration #82).
            Value::Block(b) if b.name == "Image" && refused_by_element(b, "source", fields) => {
                let alt = match b.argument("alt").map(|a| &a.value) {
                    Some(Value::Text(t)) => t.clone(),
                    _ => String::new(),
                };
                let pos = b.pos;
                *value = Value::Block(Block { name: "Text".into(), styles: Vec::new(), arguments: vec![crate::holo::Argument { name: None, value: Value::Text(alt), pos }], pos });
                mark_rows(value, fields, shown_ones);
            }
            Value::Block(b) => {
                // Une image légère pour le téléphone, ou un texte de remplacement, qui manquent à
                // l'élément : l'image se montre sans eux.
                if b.name == "Image" {
                    if refused_by_element(b, "phone", fields) {
                        b.arguments.retain(|a| a.name.as_deref() != Some("phone"));
                    }
                    if let Some(a) = b.arguments.iter_mut().find(|a| a.name.as_deref() == Some("alt") && matches!(&a.value, Value::Name(n) if n.starts_with("item.") && !fields.iter().any(|(c, _)| Some(c.as_str()) == n.strip_prefix("item.")))) {
                        a.value = Value::Text(String::new());
                    }
                }
                b.arguments.iter_mut().for_each(|a| mark_rows(&mut a.value, fields, shown_ones))
            }
            _ => {}
        }
    }
    /// Le fichier d'une image, pris dans un champ de l'élément (`source: item.image`), est-il
    /// absent ou hors du dossier de la page ?
    fn refused_by_element(image: &Block, parameter: &str, fields: &[(String, String)]) -> bool {
        match image.argument(parameter).map(|a| &a.value) {
            Some(Value::Name(n)) => n.strip_prefix("item.").is_some_and(|field| fields.iter().find(|(c, _)| c == field).is_none_or(|(_, v)| !path_on(v))),
            _ => false,
        }
    }
    let mut output = String::new();
    let mut worlds = String::new();
    for (rank, element) in elements.iter().enumerate() {
        let fields = crate::lists::fields(element);
        let mut shown_ones = Vec::new();
        let mut line = String::new();
        for value in model {
            let mut copy = Value::List(vec![value.clone()]);
            crate::lists::choose_by_element(&mut copy, &fields, element);
            mark_rows(&mut copy, &fields, &mut shown_ones);
            if let Value::List(chosen_ones) = &copy {
                for chosen in chosen_ones {
                    render(chosen, &mut line, &mut worlds, base, repeat)?;
                }
            }
        }
        let mut line = line.replace(ELEMENT, &escape(&crate::lists::text_of(element)));
        for (i, (name, format)) in shown_ones.iter().enumerate() {
            let raw = fields.iter().find(|(c, _)| c == name).map(|(_, v)| v.clone()).unwrap_or_default();
            let shows = match (format, raw.parse::<u64>()) {
                (Some(f), Ok(n)) => crate::format::format_value(name, n, f, &crate::format::language()),
                _ => raw,
            };
            line = line.replace(&format!("{FIELD}{i}{FIELD}"), &escape(&shows));
        }
        // La clé de la ligne (ADR-057) : la même pour le même élément, d'un état à l'autre. La page
        // garde telle quelle une ligne dont la clé et le contenu n'ont pas changé : le champ où
        // l'on écrit, un pli ouvert, le focus restent où ils sont.
        // Avec `key: id`, la clé est ce champ : elle reste la même quand le reste de l'élément change,
        // et la page garde la ligne (le focus avec) ; elle commence par « k: ».
        let key = match repeat.argument("key").map(|a| &a.value) {
            Some(Value::Name(field)) => {
                let of = |e: &String| crate::lists::fields(e).into_iter().find(|(c, _)| c == field).map(|(_, v)| v).unwrap_or_default();
                let value = of(element);
                let already = elements[..rank].iter().filter(|e| of(e) == value).count();
                format!("k:{}-{already}", escape(&value))
            }
            _ => {
                let already = elements[..rank].iter().filter(|e| *e == element).count();
                let hash = element.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, o| (h ^ u64::from(o)).wrapping_mul(0x0100_0000_01b3));
                format!("{hash:x}-{already}")
            }
        };
        output.push_str(&format!("<div class=\"holo-line\" data-rank=\"{rank}\" data-key=\"{key}\">{line}</div>"));
    }
    // Une liste vide dit ce qu'on a écrit dans `empty:` (lot 2 du web) ; un lecteur d'écran
    // l'annonce quand il apparaît, après une recherche qui ne trouve rien.
    if elements.is_empty() {
        if let Some(Value::Text(empty)) = repeat.argument("empty").map(|a| &a.value) {
            output.push_str(&format!("<p class=\"holo-empty\" role=\"status\">{}</p>", escape(empty)));
        }
    }
    Ok(output)
}

/// Les lignes d'une liste pour cet état : la page les pose à la place des anciennes (ADR-044).
pub fn list_lines(program: &Program, base: &str, numbers: &crate::state::State, texts: &crate::state::Texts, lists: &crate::lists::Lists, name: &str) -> String {
    crate::lists::set_running(lists.clone());
    crate::format::set_decimals(crate::state::decimals(program));
    set_language(program);
    // `tasks@12:5` : la répétition de « tasks » écrite ligne 12, colonne 5 ; `tasks` seul : la première.
    let (name, place) = name.split_once('@').map_or((name, None), |(n, p)| (n, Some(p)));
    let written_at = |repeat: &Block| place.is_none_or(|p| p == format!("{}:{}", repeat.pos.line, repeat.pos.column));
    let Some((repeat, _)) = crate::lists::repeats(program).into_iter().find(|(r, l)| l == name && written_at(r)) else { return String::new() };
    let mut shown = crate::state::to_show(program, numbers);
    shown.extend(crate::computed::totals(program, numbers, texts, lists));
    shown.extend(crate::computed::days_values(program, texts));
    shown.extend(crate::lists::counts(lists));
    let responses = crate::state::conditions(program, &shown, texts);
    lines(repeat, name, base).map(|html| fill_marks(html, &shown, texts, &responses)).unwrap_or_default()
}

/// La place d'un bloc sur un plateau, le long d'un axe : un nombre de 0 à 100 écrit dans le
/// fichier, ou le nom d'une valeur de la page, que le bloc suit alors quand elle change.
/// Rend l'attribut qui dit à la page quelle valeur suivre, et la place de départ.
fn place(block: &Block, axis: &str) -> Result<(String, String), Error> {
    match block.argument(axis).map(|a| &a.value) {
        Some(Value::Integer(number)) if *number <= 100 => Ok((String::new(), number.to_string())),
        Some(Value::Name(value)) => Ok((format!(" data-{axis}=\"{}\"", escape(value)), format!("{MARK}@{value}{MARK}"))),
        _ => Err(Error {
            message: format!("sur un plateau, « {} » se place par x et y : un nombre de 0 à 100, ou le nom d'une valeur de la page (il manque « {axis} », ou il est mal écrit)", block.name),
            pos: block.pos,
        }),
    }
}

/// Les réglages d'un bloc de disposition : l'écart entre ses éléments, leur placement, et le
/// nombre de colonnes d'une grille. Tout est borné, et un réglage inconnu est refusé.
fn layout(block: &Block) -> Result<String, Error> {
    let known_ones: &[&str] = if block.name == "Grid" { &["name", "children", "gap", "columns"] } else { &["name", "children", "gap", "align"] };
    let mut style = String::new();
    for argument in &block.arguments {
        let error = |expected: &str| Error { message: format!("« {}({}: …) » attend {expected}", block.name, argument.name.as_deref().unwrap_or("")), pos: argument.pos };
        match (argument.name.as_deref(), &argument.value) {
            (Some("name" | "children"), _) => {}
            (Some("gap"), value) => match value {
                Value::Number { value, unit: Some(unit), .. } if unit == "px" && (0.0..=64.0).contains(value) => style.push_str(&format!("--holo-gap:{};", to_rem(&format!("{value}px")))),
                _ => return Err(error("une taille entre 0px et 64px")),
            },
            (Some("columns"), value) if block.name == "Grid" => match value {
                Value::Integer(columns) if (1..=12).contains(columns) => style.push_str(&format!("--holo-columns:{columns};")),
                _ => return Err(error("un nombre entier entre 1 et 12")),
            },
            (Some("align"), value) if block.name != "Grid" => {
                let place = match (block.name.as_str(), value) {
                    (_, Value::Name(word)) if word == "start" => "flex-start",
                    (_, Value::Name(word)) if word == "center" => "center",
                    (_, Value::Name(word)) if word == "end" => "flex-end",
                    ("Row", Value::Name(word)) if word == "between" => "space-between",
                    ("Row", _) => return Err(error("l'un de ces mots : start, center, end, between")),
                    _ => return Err(error("l'un de ces mots : start, center, end")),
                };
                style.push_str(&format!("--holo-align:{place};"));
            }
            (Some(name), _) => {
                return Err(Error { message: format!("« {} » n'a pas de paramètre « {name} » ; paramètres possibles : {}", block.name, known_ones.join(", ")), pos: argument.pos })
            }
            (None, _) => return Err(Error { message: format!("« {} » range des blocs : {}(children: [ … ])", block.name, block.name), pos: argument.pos }),
        }
    }
    Ok(style)
}

/// La couleur et la lumière d'un point, pour la page : celles que l'auteur impose, sinon
/// celles que donne la graine, la même que dans la vue en profondeur.
fn point_pace(block: &Block) -> Result<String, Error> {
    let mut pace = String::new();
    if let Some(Value::Text(color)) = block.argument("color").map(|a| &a.value) {
        if !is_color(color) {
            return Err(Error { message: format!("« color: \"{color}\" » : une couleur est attendue, comme \"#E9B44C\""), pos: block.pos });
        }
        pace.push_str(&format!("--holo-color:{color};"));
    } else if let Some(Value::Integer(seed)) = block.argument("seed").map(|a| &a.value) {
        let [r, v, b] = crate::universe::World::from_seed(*seed).color.map(|c| (c * 255.0).round() as u8);
        pace.push_str(&format!("--holo-color:rgb({r},{v},{b});"));
    }
    match block.argument("brightness").map(|a| &a.value) {
        Some(Value::Number { value, unit: None, .. }) => pace.push_str(&format!("--holo-brightness:{value};")),
        Some(Value::Integer(integer)) => pace.push_str(&format!("--holo-brightness:{integer};")),
        _ => {}
    }
    Ok(pace)
}

/// Le monde à l'intérieur d'un point : son contenu, caché tant qu'on n'y est pas entré.
fn inner_world(point: &Block, worlds: &mut String, base: &str) -> Result<(), Error> {
    if let (Some(Value::Block(world)), Some(point_name)) = (point.argument("inside").map(|a| &a.value), name_of(point)) {
        let mut panel = String::new();
        let mut deeper = String::new();
        children(world, &mut panel, &mut deeper, base)?;
        worlds.push_str(&format!(
            "<section class=\"{}\" data-world=\"{}\" hidden><div class=\"holo-panel\">{panel}</div></section>{deeper}",
            classes(world),
            escape(point_name)
        ));
    }
    Ok(())
}

/// Un point planté dans un pixel de la page (`pixels:` d'une `Page`). Au repos il occupe un
/// seul pixel : on ne le remarque qu'en s'approchant. La page d'entrée le place juste
/// au-dessus du bloc nommé par `above`, à l'extrémité droite.
fn planted_pixel(value: &Value, output: &mut String, worlds: &mut String, base: &str, page: &Block) -> Result<(), Error> {
    let point = match value {
        Value::Block(block) if block.name == "Point" => block,
        _ => return Err(Error { message: "« pixels » contient des blocs « Point(...) »".into(), pos: page.argument("pixels").map_or(page.pos, |a| a.pos) }),
    };
    let (Some(name), Some(Value::Name(landmark))) = (name_of(point), point.argument("above").map(|a| &a.value)) else {
        return Err(Error { message: "un point planté dans un pixel a un nom et un repère : Point(name: Secret, above: Open, ...)".into(), pos: point.pos });
    };
    // Un vrai bouton : on l'atteint au clavier, et un lecteur d'écran dit son nom.
    output.push_str(&format!(
        "<button type=\"button\" class=\"holo-pixel\" data-name=\"{name}\" aria-label=\"{name}\" data-above=\"{}\" style=\"{}\"></button>",
        escape(landmark),
        point_pace(point)?,
        name = escape(name)
    ));
    inner_world(point, worlds, base)
}

/// Le texte d'un bloc : `P("Bonjour")`.
fn text_of(block: &Block) -> Result<&str, Error> {
    block.arguments
        .iter()
        .find_map(|a| match (&a.name, &a.value) {
            (None, Value::Text(t)) => Some(t.as_str()),
            _ => None,
        })
        .ok_or_else(|| Error { message: format!("« {} » attend un texte entre guillemets : {}(\"…\")", block.name, block.name), pos: block.pos })
}

/// L'adresse d'un lien : un fichier rangé à côté (rendu relatif au dossier du `.holo`), un
/// site de la page (`#Workshop`), ou une adresse du web en http ou https. Rien d'autre : pas
/// de `javascript:`, pas de caractères qui sortiraient de l'attribut.
fn safe_address(address: &str, base: &str) -> Option<String> {
    if address.starts_with("https://") || address.starts_with("http://") {
        return web_address(address).then(|| address.to_string());
    }
    let (file, anchor) = address.split_once('#').unwrap_or((address, ""));
    let safe_anchor = anchor.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/'));
    match (file.is_empty(), anchor.is_empty()) {
        (true, false) if safe_anchor => Some(format!("#{anchor}")),
        (false, _) if link_on(file) && safe_anchor => Some(if anchor.is_empty() { format!("{base}{file}") } else { format!("{base}{file}#{anchor}") }),
        _ => None,
    }
}

/// Le chemin d'un lien vers une page du site (`A(to:)`) : comme un fichier rangé à côté, mais il
/// peut aussi remonter d'un dossier (`../accueil.holo`), comme sur le web, et porter des lettres
/// accentuées (`profils/Adé`), que le navigateur encode (ADR-078). Jamais `/` en tête, ni `..`
/// ailleurs qu'au début.
fn link_on(file: &str) -> bool {
    let mut rest = file;
    while let Some(after) = rest.strip_prefix("../") {
        rest = after;
    }
    !rest.is_empty() && !rest.starts_with('/') && !rest.contains("..") && rest.chars().all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
}

/// Une adresse du web, en http ou https, sans caractère qui sortirait d'un attribut.
fn web_address(address: &str) -> bool {
    (address.starts_with("https://") || address.starts_with("http://"))
        && address.len() > 8
        && address.chars().all(|c| c.is_ascii_graphic() && !matches!(c, '"' | '<' | '>' | '\\' | '`'))
}

/// L'adresse d'un passage vers un autre serveur : en https. Le http n'est accepté que vers
/// sa propre machine (`localhost`, `127.0.0.1`), pour les essais : sinon, une page publique
/// pourrait faire partir des requêtes vers le réseau privé du visiteur (revue Codex).
fn passage_address(address: &str) -> bool {
    let at_home = ["http://localhost", "http://127.0.0.1"].iter().any(|start| {
        address.strip_prefix(start).is_some_and(|suite| suite.starts_with(':') || suite.starts_with('/'))
    });
    web_address(address) && (address.starts_with("https://") || at_home)
}

/// Une image se range à côté du fichier : ni adresse complète, ni remontée de dossier.
pub(crate) fn path_on(source: &str) -> bool {
    !source.is_empty()
        && !source.starts_with('/')
        && !source.contains("..")
        && source.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
}

/// Un champ obligatoire le dit à un lecteur d'écran (ADR-068) ; c'est l'envoi du formulaire qui
/// le vérifie, et le moteur qui écrit le message.
fn checked_by_form(block: &Block) -> &'static str {
    if matches!(block.argument("required").map(|a| &a.value), Some(Value::Bool(true))) {
        " aria-required=\"true\""
    } else {
        ""
    }
}

pub(crate) fn escape(text: &str) -> String {
    text.replace(MARK, "").replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Le Markdown d'une ligne : `**gras**` et `*italique*`. Le reste du Markdown viendra.
fn markdown(text: &str) -> String {
    fn alternate(text: &str, mark: &str, tag: &str) -> String {
        let chunks: Vec<&str> = text.split(mark).collect();
        if chunks.len() % 2 == 0 {
            // Une marque jamais refermée reste du texte ordinaire.
            return text.to_string();
        }
        chunks
            .iter()
            .enumerate()
            .map(|(i, m)| if i % 2 == 1 { format!("<{tag}>{m}</{tag}>") } else { m.to_string() })
            .collect()
    }
    // `code` entre accents graves, puis le gras et l'italique. Dans un texte écrit sur
    // plusieurs lignes (entre trois guillemets), chaque retour à la ligne est gardé.
    let mut html = alternate(&alternate(&alternate(&escape(text), "`", "code"), "**", "strong"), "*", "em");
    // `~~barré~~`, `==surligné==`, `m^2^` et `H~2~O` (ADR-042).
    html = alternate(&alternate(&alternate(&alternate(&html, "~~", "s"), "==", "mark"), "^", "sup"), "~", "sub").replace('\n', "<br>");
    // `{cart}` : l'endroit où s'affiche une valeur de la page. `site_html` y écrit son départ,
    // la page d'entrée la tient à jour.
    // Une valeur à virgule (ADR-066) a son format : `d2`, ou `nd2` groupée par milliers.
    for name in crate::state::names_in(text) {
        let span = match crate::format::decimal_places(name) {
            0 => format!("<span data-state=\"{name}\"></span>"),
            places => format!("<span data-state=\"{name}\" data-format=\"d{places}\"></span>"),
        };
        html = html.replace(&format!("{{{name}}}"), &span);
    }
    // `{minute:00}` : la valeur, avec son format (ADR-043).
    for (name, format) in crate::format::formats_in(text) {
        let shown = match (crate::format::decimal_places(name), format) {
            (places, "number") if places > 0 => format!("nd{places}"),
            _ => format.to_string(),
        };
        html = html.replace(&format!("{{{name}:{format}}}"), &format!("<span data-state=\"{name}\" data-format=\"{shown}\"></span>"));
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::read;

    fn page(src: &str) -> Result<String, Error> {
        page_html(&read(src)?, "")
    }

    #[test]
    fn the_shop_becomes_a_web_page() {
        let html = page_html(&read(include_str!("../../exemples/boutique-comparee/boutique.holo")).unwrap(), "/ex/").unwrap();
        for expected in [
            "<div class=\"holo-Page\" data-title=\"My shop\" data-zoom><header class=\"holo-Header\">",
            "<nav class=\"holo-Nav\">",
            "</header><main><h1 class=\"holo-H1\">My shop</h1>",
            "<div class=\"holo-Stack\"><div><img class=\"holo-Image\"",
            "<div class=\"holo-Text holo-s-badge\">New</div>",
            "<h4 class=\"holo-H4\">Weekdays</h4>",
            "<footer class=\"holo-Footer\"><hr class=\"holo-Hr\">",
            "@media (hover:hover){.holo-Button:hover{background:#2a2a4e;}}",
            "<p class=\"holo-P\">Paintings made by hand, one at a time.</p>",
            "<p class=\"holo-P holo-s-card\">Free delivery from <strong>30 euros</strong>.</p>",
            "<img class=\"holo-Image\" src=\"/ex/painting.svg\" alt=\"A painting: a yellow sun over green hills\">",
            "<ul class=\"holo-List\"><li>Sunrise over the river</li>",
            "<div class=\"holo-Text holo-s-note\">Open until 6 pm</div>",
            "<button type=\"button\" class=\"holo-Button holo-s-card\" data-name=\"Open\">Enter the workshop</button>",
            "data-name=\"Workshop\" aria-label=\"Workshop\" style=\"--holo-color:#E9B44C;--holo-brightness:0.8;\"",
            "<section class=\"holo-World\" data-world=\"Workshop\" hidden><div class=\"holo-panel\"><h1 class=\"holo-H1\">The workshop</h1>",
            "data-name=\"Back\">Back to the shop</button>",
            // Les tailles suivent le texte du visiteur (ADR-061) ; le trait reste en pixels.
            ".holo-s-card{background:#1a1a2e;border:1px solid #E9B44C;border-radius:0.75rem;padding:0.5rem 1rem;}",
        ] {
            assert!(html.contains(expected), "manque : {expected}\n{html}");
        }
        // Le thème avant les types, les types avant les styles nommés.
        let place = |chunk: &str| html.find(chunk).unwrap();
        assert!(place(".holo-Page{") < place(".holo-H1{") && place(".holo-World,") < place(".holo-H1{") && place(".holo-P{") < place(".holo-s-card{"));
    }

    #[test]
    fn the_missing_text_rule_quote_code_line_break_and_alt() {
        let html = page(
            "Page(children: [ Hr(), Quote(\"A **fine** shop.\", by: \"A visitor\"), Code(\"<b> WELCOME10 & co\"), P(\"\"\"\n  First line\n  Second `line`\n\"\"\"), Image(source: \"a.png\", alt: \"A red \\\"door\\\"\") ])"
                .replace("\\\"", "'")
                .as_str(),
        )
        .unwrap();
        assert!(html.contains("<hr class=\"holo-Hr\">"), "{html}");
        assert!(html.contains("<blockquote class=\"holo-Quote\"><p>A <strong>fine</strong> shop.</p><footer>— A visitor</footer></blockquote>"), "{html}");
        // Dans Code, rien n'est interprété : ni balise, ni gras.
        assert!(html.contains("<pre class=\"holo-Code\"><code>&lt;b&gt; WELCOME10 &amp; co</code></pre>"), "{html}");
        assert!(html.contains("<p class=\"holo-P\">First line<br>Second <code>line</code></p>"), "{html}");
        assert!(html.contains("<img class=\"holo-Image\" src=\"a.png\" alt=\"A red 'door'\">"), "{html}");
        // Un décor s'écrit alt: "" ; sans alt, l'image est refusée (ADR-038).
        assert!(page("Page(children: [ Image(source: \"a.png\", alt: \"\") ])").unwrap().contains("alt=\"\""));
        assert!(page("Page(children: [ Image(source: \"a.png\") ])").unwrap_err().message.contains("attend « alt »"));
        assert!(page("Page(children: [ Image(source: \"a.png\", alt: 3) ])").unwrap_err().message.contains("attend un texte"));
        assert!(page("Page(children: [ Hr(color: red) ])").unwrap_err().message.contains("s'écrit « Hr() »"));
        assert!(page("Page(children: [ Quote(\"x\", by: 3) ])").unwrap_err().message.contains("qui l'a dit"));
        // Le caractère qui sert de marque aux conditions ne passe pas par un texte.
        assert!(!page("Page(children: [ \"a\u{1}b\" ])").unwrap().contains('\u{1}'));
    }

    #[test]
    fn a_shape_is_a_drawing_or_a_button() {
        let html = page("Page(children: [ Shape(form: circle, color: \"#E9B44C\", size: 40px), Shape(name: Cible, form: triangle) ])").unwrap();
        assert!(html.contains("<div class=\"holo-Shape holo-forme-circle\" style=\"--holo-color:#E9B44C;--holo-size:40px;--holo-n:40;\"></div>"), "{html}");
        assert!(html.contains("<button type=\"button\" class=\"holo-Shape holo-forme-triangle\" data-name=\"Cible\" aria-label=\"Cible\" style=\"\"></button>"), "{html}");
        // Une forme nommée se touche, comme un bouton, et se place sur un plateau.
        page("Page(state: State(n: 0, sx: 5), children: [ Board(children: [ Shape(name: S, form: square, x: sx, y: 50, drag: true) ]) ], rules: [ On(S.tap, effect: n.add(1)) ])").unwrap();
        for (source, message) in [
            ("Page(children: [ Shape(color: \"red\") ])", "attend « form »"),
            ("Page(children: [ Shape(form: hexagon) ])", "circle, square, triangle, diamond"),
            ("Page(children: [ Shape(form: circle, color: \"url(x)\") ])", "attend une couleur"),
            ("Page(children: [ Shape(form: circle, size: 5000px) ])", "entre 8px et 400px"),
            ("Page(children: [ Shape(form: circle, border: 2) ])", "n'a pas de paramètre « border »"),
        ] {
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn a_board_places_its_blocks_anywhere() {
        let html = page(
            "Page(state: State(sx: 70, sy: 250), children: [ Board(height: 200px, children: [ Point(name: Star, seed: 7, x: sx, y: sy), Button(name: B, text: \"b\", x: 10, y: 90), P(\"libre\") ]) ])",
        )
        .unwrap();
        // L'étoile suit deux valeurs ; au départ elle est à leur place, sans dépasser le plateau.
        assert!(html.contains("<div class=\"holo-Board\" style=\"aspect-ratio:640/200;width:min(100%,256.0vh)\"><div class=\"holo-positioned\" data-x=\"sx\" data-y=\"sy\" style=\"--x:70;--y:100\"><button type=\"button\" class=\"holo-Point\" data-name=\"Star\""), "{html}");
        // Un bloc posé à une place fixe, et un bloc sans place.
        assert!(html.contains("<div class=\"holo-positioned\" style=\"--x:10;--y:90\"><button type=\"button\" class=\"holo-Button\" data-name=\"B\">b</button></div><p class=\"holo-P\">libre</p></div>"), "{html}");
        for (source, message) in [
            ("Page(children: [ Board(height: 5000px, children: []) ])", "entre 80px et 800px"),
            ("Page(children: [ Board(width: 10px, children: []) ])", "n'a pas de paramètre « width »"),
            ("Page(children: [ Board(children: [ P(\"a\", x: 10) ]) ])", "il manque « y »"),
            ("Page(children: [ Board(children: [ P(\"a\", x: 10, y: 500) ]) ])", "un nombre de 0 à 100"),
        ] {
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn a_growing_block_and_a_styles_file() {
        let source = "Page(children: [ Row(children: [ Input(value: q, label: \"Search\", grow: 1), Button(name: Go, text: \"Go\") ]) ], state: State(q: \"\"))";
        let html = page(source).unwrap();
        assert!(html.contains("<div class=\"holo-grow\" style=\"flex:1 1 0\"><label class=\"holo-Input\""), "{html}");
        crate::check_page(source).unwrap();
        for (source, message) in [
            ("Page(children: [ P(\"x\", grow: 1) ])", "rangé dans Row ou Column"),
            ("Page(children: [ Row(children: [ P(\"x\", grow: 20) ]) ])", "de 1 à 12"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
        // Un fichier qui ne contient que des styles s'importe comme un thème.
        let theme = "Page { --gold: #E9B44C; background: #101020; }\nH1 { color: --gold; }";
        let page_source = format!("import \"theme.holo\"\nPage(children: [ H1(\"a\") ]){}theme.holo{}{theme}", crate::holo::NEXT_FILE, crate::holo::NAME_SEPARATOR);
        let html = page(&page_source).unwrap();
        assert!(html.contains(".holo-H1{color:var(--gold);}"), "{html}");
        crate::check_page(&page_source).unwrap();
        assert_eq!(crate::check_text(theme), "ok : un fichier de styles, à importer dans une page");
        assert!(crate::check_text("H1 { colour: red; }").contains("colour"));
        // La faute d'un thème garde sa place dans le fichier : ligne 2, colonne 6.
        assert_eq!(crate::check_text("// Un thème.
H1 { colour: red; }").split(" : ").next(), Some("ligne 2, colonne 6"));
        assert!(crate::is_styles_file(theme));
        assert!(!crate::is_styles_file("Page(children: [ H1(\"a\") ])"));
    }

    #[test]
    fn lot_9_sizes_sound_and_entrance_in_view() {
        // Les tailles en pixels deviennent des rem ; « screen » remplit l'écran, au moins.
        let html = page("Page(children: [ P.bloc(\"x\") ])\n\n.bloc { padding: 0 24px; max-width: 480px; height: screen; border: 2px solid red; }").unwrap();
        assert!(html.contains(".holo-s-bloc{padding:0 1.5rem;max-width:30rem;height:auto;min-height:100vh;min-height:100dvh;border:2px solid red;}"), "{html}");
        // Le son : son volume, sa boucle.
        let html = page("Page(children: [ Sound(name: Rain, source: \"pluie.mp3\", volume: 0.4, loop: true) ])").unwrap();
        assert!(html.contains("src=\"pluie.mp3\" data-volume=\"0.4\" loop></audio>"), "{html}");
        // Une entrée qui attend d'être vue.
        let html = page("Page(children: [ P(\"x\", enter: Enter(y: 40px, opacity: 0, inView: true)) ])").unwrap();
        assert!(html.contains("<div class=\"holo-animated holo-in-view hm"), "{html}");
        assert!(html.contains(".holo-js .holo-in-view:not(.holo-seen)"), "{html}");
        let error = page("Page(children: [ P(\"x\", loop: Loop(scale: 1.1, inView: true)) ])").unwrap_err();
        assert!(error.message.contains("inView"), "{error}");
    }

    #[test]
    fn layout_arranges_side_by_side_in_column_and_in_grid() {
        let html = page(
            "Page(children: [ Row(gap: 8px, align: between, children: [ H1(\"Shop\"), Button(name: Menu, text: \"Menu\") ]), Grid(columns: 3, children: [ \"a\", Column(align: center, children: [ P(\"b\"), P(\"c\") ]) ]) ])",
        )
        .unwrap();
        assert!(html.contains("<div class=\"holo-Row\" style=\"--holo-gap:0.5rem;--holo-align:space-between;\"><h1 class=\"holo-H1\">Shop</h1><button"), "{html}");
        assert!(html.contains("<div class=\"holo-Grid\" style=\"--holo-columns:3;\"><p class=\"holo-P\">a</p><div class=\"holo-Column\" style=\"--holo-align:center;\">"), "{html}");
        // Un bouton rangé dans une ligne reste un bouton : sa règle le trouve.
        assert!(html.contains("data-name=\"Menu\""));
        for (source, message) in [
            ("Page(children: [ Row(gap: 8, children: []) ])", "entre 0px et 64px"),
            ("Page(children: [ Row(gap: 500px, children: []) ])", "entre 0px et 64px"),
            ("Page(children: [ Row(align: middle, children: []) ])", "start, center, end, between"),
            ("Page(children: [ Column(align: between, children: []) ])", "start, center, end"),
            ("Page(children: [ Grid(columns: 40, children: []) ])", "entre 1 et 12"),
            ("Page(children: [ Row(columns: 2, children: []) ])", "n'a pas de paramètre « columns »"),
            ("Page(children: [ Grid(align: center, children: []) ])", "n'a pas de paramètre « align »"),
            ("Page(children: [ Row(wrap: false, children: []) ])", "n'a pas de paramètre « wrap »"),
            ("Page(children: [ Row(\"a\", \"b\") ])", "range des blocs"),
        ] {
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn a_site_is_planted_in_a_page_pixel() {
        let html = page(
            "Page(children: [ Button(name: Open, text: \"x\") ], pixels: [ Point(name: Secret, above: Open, seed: 7, color: \"#FF4D6D\", inside: World(children: [ H1(\"Hidden\") ])) ])",
        )
        .unwrap();
        assert!(html.contains("<button type=\"button\" class=\"holo-pixel\" data-name=\"Secret\" aria-label=\"Secret\" data-above=\"Open\" style=\"--holo-color:#FF4D6D;\"></button>"), "{html}");
        assert!(html.contains("data-world=\"Secret\" hidden><div class=\"holo-panel\"><h1 class=\"holo-H1\">Hidden</h1>"));
        assert!(page("Page(pixels: [ P(\"x\") ])").unwrap_err().message.contains("des blocs « Point(...) »"));
        assert!(page("Page(pixels: [ Point(name: A, seed: 1) ])").unwrap_err().message.contains("un nom et un repère"));
    }

    #[test]
    fn a_point_world_opens_like_a_page() {
        let program = read(
            "Page(title: \"Top\", children: [ Point(name: A, seed: 1, inside: World.rose(children: [ H1(\"Inside\"), Button(name: Out, text: \"x\") ], pixels: [ Point(name: B, above: Out, seed: 2, inside: World(children: [ P(\"Deeper\") ])) ])) ])\n.rose { background: #3a0d1a; }\nWorld { color: white; }",
        )
        .unwrap();
        let world = crate::rules::site_of(&program, "A").unwrap();
        let html = site_html(&program, world, "", "A").unwrap();
        assert!(html.contains("<div class=\"holo-Page holo-world-open holo-s-rose\" data-title=\"A\" data-zoom><main><h1 class=\"holo-H1\">Inside</h1>"), "{html}");
        assert!(html.contains(".holo-World,.holo-world-open{color:white;}"));
        // Il a ses propres points plantés, et leurs mondes : la boucle continue.
        assert!(html.contains("class=\"holo-pixel\" data-name=\"B\" aria-label=\"B\" data-above=\"Out\""));
        assert!(html.contains("data-world=\"B\" hidden>"));
        let deeper = crate::rules::site_of(&program, "A/B").unwrap();
        assert!(site_html(&program, deeper, "", "B").unwrap().contains("Deeper"));
        assert!(crate::rules::site_of(&program, "A/Nobody").unwrap_err().message.contains("aucun point"));
        assert_eq!(crate::rules::site_of(&program, "").unwrap().name, "Page");
    }

    #[test]
    fn links_and_doors_to_another_file() {
        let html = page_html(
            &read("Page(children: [ A(\"The garden\", to: \"garden.holo\"), A(\"Elsewhere\", to: \"https://example.com/a?b=1\"), A(\"Inside\", to: \"#Workshop\"), List(ordered: true, children: [ A(\"x\", to: \"a/b.holo#S\") ]), Point(name: Garden, seed: 3, inside: \"garden.holo\") ])").unwrap(),
            "/ex/",
        )
        .unwrap();
        for expected in [
            "<a class=\"holo-A\" href=\"/ex/garden.holo\">The garden</a>",
            "<a class=\"holo-A\" href=\"https://example.com/a?b=1\">Elsewhere</a>",
            "<a class=\"holo-A\" href=\"#Workshop\">Inside</a>",
            "<ol class=\"holo-List\"><li><a class=\"holo-A\" href=\"/ex/a/b.holo#S\">x</a></li></ol>",
            "data-name=\"Garden\" aria-label=\"Garden\" data-file=\"/ex/garden.holo\"",
        ] {
            assert!(html.contains(expected), "manque : {expected}\n{html}");
        }
        // Un lien peut remonter d'un dossier, comme sur le web, et porter des lettres accentuées
        // (ADR-078) ; le serveur, lui, ne sort jamais du site.
        let up = page("Page(children: [ A(\"Up\", to: \"../home.holo\"), A(\"Adé\", to: \"profils/Adé\") ])").unwrap();
        assert!(up.contains("href=\"../home.holo\"") && up.contains("href=\"profils/Adé\""), "{up}");
        // Un lien ne peut pas cacher de code, ni partir de la racine, ni remonter au milieu.
        for bad in ["javascript:alert(1)", "/secret.holo", "a/../../secret.holo", "https://a.example/\\\"onclick=", "data:text/html,x", "a%2e%2e", ""] {
            assert!(page(&format!("Page(children: [ A(\"x\", to: \"{bad}\") ])")).is_err(), "{bad}");
        }
        assert!(page("Page(children: [ A(\"x\") ])").unwrap_err().message.contains("« to »"));
        assert!(page("Page(children: [ Point(name: G, seed: 1, inside: \"garden.txt\") ])").unwrap_err().message.contains("fichier .holo"));
        // Le fichier d'un autre auteur, sur un autre serveur.
        let elsewhere = page("Page(children: [ Point(name: G, seed: 1, inside: \"https://friend.example/home/garden.holo\") ])").unwrap();
        assert!(elsewhere.contains("data-file=\"https://friend.example/home/garden.holo\""), "{elsewhere}");
        assert!(page("Page(children: [ Point(name: G, seed: 1, inside: \"https://friend.example/x.html\") ])").is_err());
        assert!(page("Page(children: [ Point(name: G, seed: 1, inside: \"javascript:x.holo\") ])").is_err());
        // En http, seulement vers sa propre machine ; jamais vers le réseau privé de quelqu'un.
        assert!(page("Page(children: [ Point(name: G, seed: 1, inside: \"http://127.0.0.1:8081/x.holo\") ])").is_ok());
        assert!(page("Page(children: [ Point(name: G, seed: 1, inside: \"http://localhost/x.holo\") ])").is_ok());
        for refuses in ["http://friend.example/x.holo", "http://192.168.1.1/x.holo", "http://localhost.evil.example/x.holo", "http://127.0.0.1.evil.example/x.holo"] {
            assert!(page(&format!("Page(children: [ Point(name: G, seed: 1, inside: \"{refuses}\") ])")).is_err(), "{refuses}");
        }
    }

    #[test]
    fn the_same_file_gives_the_same_page() {
        let source = include_str!("../../exemples/boutique-comparee/boutique.holo");
        assert_eq!(page_html(&read(source).unwrap(), ""), page_html(&read(source).unwrap(), ""));
    }

    #[test]
    fn nothing_the_author_writes_becomes_code() {
        let html = page("Page(title: \"<script>\", children: [ \"<script>alert(1)</script> & *ok*\" ])").unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt; &amp; <em>ok</em>"));
        assert!(page("Page(children: [ Image(source: \"https://ailleurs.example/x.png\") ])").unwrap_err().message.contains("à côté du .holo"));
        assert!(page("Page(children: [ Image(source: \"../secret.png\") ])").is_err());
        assert!(page("Page(children: [ Point(name: A, seed: 1, color: \"red;background:url(x)\") ])").is_err());
    }

    #[test]
    fn errors_are_reported() {
        assert!(page("Point(name: A, seed: 1)").unwrap_err().message.contains("affiche une « Page »"));
        assert!(page("Page(children: [ H1() ])").unwrap_err().message.contains("attend un texte"));
        assert!(page("Page(children: [ Button(name: A) ])").unwrap_err().message.contains("« text »"));
        assert!(page("Page(children: [ Point(name: A, seed: 7) ])").unwrap().contains("--holo-color:rgb("));
        assert!(page("Page(children: [ World(children: []) ])").unwrap_err().message.contains("ne se place pas"));
        assert_eq!(markdown("2 * 3"), "2 * 3");
    }

    #[test]
    fn a_page_that_moves_by_itself_requests_the_engine_at_once() {
        // La boutique attend qu'on la touche : elle s'affiche sans le moteur.
        let shop = crate::flat_view(include_str!("../../exemples/boutique-comparee/boutique.holo"), "").unwrap();
        assert!(!shop.contains("data-live"));
        // Un jeu a des horloges et le clavier : le moteur doit arriver tout de suite.
        for game in [include_str!("../../exemples/jeu/attraper.holo"), include_str!("../../exemples/jeu/panier.holo")] {
            assert!(crate::flat_view(game, "").unwrap().contains("data-live>"));
        }
    }


    #[test]
    fn movement_becomes_css_and_only_moves_when_written() {
        // Sans mouvement écrit, rien de plus dans la page.
        let calm = crate::flat_view("Page(children: [ H1(\"Hi\") ])", "").unwrap();
        assert!(!calm.contains("holo-animated") && !calm.contains("@keyframes"));
        // Une entrée : le bloc est enveloppé, ses images clés vont de la pose écrite au repos.
        let html = crate::flat_view("Page(children: [ H1(\"Hi\", enter: Enter(y: 40px, opacity: 0, at: 1s, for: 0.5s, ease: linear)) ])", "").unwrap();
        assert!(html.contains("<div class=\"holo-animated hm1\"><h1 class=\"holo-H1\">Hi</h1></div>"), "{html}");
        assert!(html.contains("@keyframes hm1{from{opacity:0;translate:0px 40px;}to{opacity:1;translate:0px 0px;}}"), "{html}");
        assert!(html.contains(".hm1{animation:hm1 0.5s linear 1s both}"), "{html}");
        // Lettre à lettre : chaque lettre a son rang, et son délai.
        let html = crate::flat_view("Page(children: [ P(\"Oh\", enter: Enter(scale: 0, letters: 0.1s)) ])", "").unwrap();
        assert!(html.contains("<span class=\"holo-letter\" style=\"--i:1\">h</span>"), "{html}");
        assert!(html.contains("animation-delay:calc(0s + var(--i) * 0.1s)"), "{html}");
        // Des scènes qui recommencent : la deuxième apparaît à la moitié du tour.
        let html = crate::flat_view("Page(children: [ Scenes(repeat: forever, children: [ Scene(for: 2s, children: [ \"a\" ]), Scene(for: 2s, children: [ P(\"b\", enter: Enter(opacity: 0, at: 1s, for: 1s)) ]) ]) ])", "").unwrap();
        assert!(html.contains("@keyframes hs2{0%{opacity:0;visibility:hidden}50.000%{opacity:0;visibility:visible}"), "{html}");
        // L'entrée de la deuxième scène part à 3 s sur un tour de 4 s : 75 %.
        assert!(html.contains("75.000%{opacity:0;animation-timing-function:"), "{html}");
        assert!(html.contains("prefers-reduced-motion:reduce"), "{html}");
        // Ce qui est refusé, avec une phrase qui dit quoi faire.
        for (source, message) in [
            ("Page(children: [ H1(\"a\", enter: Enter()) ])", "dit ce qui bouge"),
            ("Page(children: [ H1(\"a\", enter: Enter(y: 40)) ])", "un nombre en px"),
            ("Page(children: [ H1(\"a\", enter: Enter(opacity: 3)) ])", "de 0 à 1"),
            ("Page(children: [ H1(\"a\", enter: Enter(y: 4px, ease: wobble)) ])", "l'une de ces courbes"),
            ("Page(children: [ H1(\"a\", enter: Enter(y: 4px, back: false)) ])", "n'a pas de paramètre « back »"),
            ("Page(children: [ H1(\"a\", enter: Loop(y: 4px)) ])", "attend Enter"),
            ("Page(children: [ Shape(form: circle, enter: Enter(y: 4px, letters: 0.1s)) ])", "coupe un texte en lettres"),
            ("Page(children: [ H1(\"a\", enter: Enter(y: 4px, each: 0.1s)) ])", "a des « children »"),
            ("Page(children: [ Scenes(children: [ P(\"a\") ]) ])", "ne range que des"),
            ("Page(children: [ Scenes(children: [ Scene(children: []) ]) ])", "combien de temps elle dure"),
            ("Page(children: [ Scene(for: 1s, children: []) ])", "se range dans des scènes"),
        ] {
            let error = crate::flat_view(source, "").unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }


    #[test]
    fn landmarks_states_and_stacking() {
        // L'en-tête et le pied sortent du contenu principal ; Main dit où il est.
        let html = crate::flat_view("Page(children: [ Header(children: [ Nav(children: [ A(\"Home\", to: \"a.holo\") ]) ]), Main(children: [ H1(\"Hi\") ]), Footer(children: [ \"End\" ]) ])", "").unwrap();
        assert!(html.contains("<header class=\"holo-Header\"><nav class=\"holo-Nav\"><a class=\"holo-A\" href=\"a.holo\">Home</a></nav></header><main><h1 class=\"holo-H1\">Hi</h1></main><footer class=\"holo-Footer\"><p class=\"holo-P\">End</p></footer>"), "{html}");
        // Les titres jusqu'à H6.
        assert!(crate::flat_view("Page(children: [ H1(\"a\"), H2(\"b\"), H3(\"c\"), H4(\"d\"), H5(\"e\"), H6(\"f\") ])", "").unwrap().contains("<h6 class=\"holo-H6\">f</h6>"));
        // Le texte suit le réglage du visiteur ; un grand titre rétrécit sur un petit écran.
        let html = crate::flat_view("Page(children: [ H1(\"a\"), P(\"b\") ])\nP { font-size: 18px; }\nH1 { font-size: 64px; }", "").unwrap();
        assert!(html.contains(".holo-P{font-size:1.125rem;}"), "{html}");
        assert!(html.contains(".holo-H1{font-size:clamp(1.5rem,10vw,4rem);}"), "{html}");
        // Les états d'un style.
        let html = crate::flat_view("Page(children: [ Button.go(name: G, text: \"Go\") ])\n.go { background: blue; hover: { background: navy; } focus: { border: 2px solid white; } active: { opacity: 0.5; } }", "").unwrap();
        assert!(html.contains(".holo-s-go{background:blue;transition:"), "{html}");
        assert!(html.contains("@media (hover:hover){.holo-s-go:hover{background:navy;}}"), "{html}");
        assert!(html.contains(".holo-s-go:focus-visible{border:2px solid white;}"), "{html}");
        assert!(html.contains(".holo-s-go:active{opacity:0.5;}"), "{html}");
        // La superposition.
        let html = crate::flat_view("Page(children: [ Stack(children: [ Image(source: \"a.png\", alt: \"x\"), Text(\"New\", align: topRight) ]) ])", "").unwrap();
        assert!(html.contains("<div class=\"holo-Stack\"><div><img"), "{html}");
        assert!(html.contains("<div class=\"holo-stacked\" style=\"align-self:start;justify-self:end\"><div class=\"holo-Text\">New</div></div></div>"), "{html}");
        // Ce qui est refusé.
        for (source, message) in [
            ("Page(children: [ Row(children: [ Main(children: []) ]) ])", "se place directement dans la page"),
            ("Page(children: [ Nav(color: red, children: []) ])", "n'a pas de paramètre « color »"),
            ("Page(children: [ Stack(children: [ P(\"a\"), P(\"b\", align: middle) ]) ])", "l'une de ces places"),
            ("Page(children: [ P(\"a\") ])\nP { hover: { color: red; } hover: { color: blue; } }", "donné deux fois"),
            ("Page(children: [ P(\"a\") ])\nP { visited: { color: red; } }", "n'est pas un état"),
            ("Page(children: [ P(\"a\") ])\nP { hover: { position: absolute; } }", "la disposition vient des blocs"),
            ("Page(children: [ H1(\"a\"), H7(\"b\") ])", "de « H1 » à « H6 »"),
        ] {
            let error = crate::check_page(source).err().or_else(|| crate::flat_view(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }


    #[test]
    fn batch_1_language_video_table_text_choice() {
        // La langue, la description et l'image de partage.
        let html = crate::flat_view("Page(lang: \"fr\", description: \"Une boutique\", image: \"p.png\", children: [ H1(\"a\") ])", "/ex/").unwrap();
        assert!(html.contains(" data-lang=\"fr\" data-description=\"Une boutique\" data-image=\"/ex/p.png\">"), "{html}");
        // La vidéo : avec ses boutons, jamais lancée seule.
        let html = crate::flat_view("Page(children: [ Video(source: \"f.mp4\", label: \"Un tour\") ])", "").unwrap();
        assert!(html.contains("<video class=\"holo-Video\" src=\"f.mp4\" controls preload=\"metadata\" playsinline aria-label=\"Un tour\"></video>"), "{html}");
        assert!(!html.contains("autoplay"));
        // Le tableau.
        let html = crate::flat_view("Page(children: [ Table(caption: \"Horaires\", head: [\"Jour\", \"Heures\"], rows: [ [\"Lundi\", \"**9 h**\"] ]) ])", "").unwrap();
        assert!(html.contains("<div class=\"holo-table-wrap\"><table class=\"holo-Table\"><caption>Horaires</caption><thead><tr><th scope=\"col\">Jour</th><th scope=\"col\">Heures</th></tr></thead><tbody><tr><td>Lundi</td><td><strong>9 h</strong></td></tr></tbody></table></div>"), "{html}");
        // Le texte long et le choix, avec leur valeur de départ.
        let html = crate::flat_view("Page(state: State(message: \"Bonjour\", taille: \"M\"), children: [ Input(value: message, label: \"Message\", lines: 4), Choice(value: taille, label: \"Taille\", options: [\"S\", \"M\"]), Choice(value: taille, label: \"Encore\", options: [\"S\", \"M\"], menu: true) ])", "").unwrap();
        assert!(html.contains("<textarea rows=\"4\" maxlength=\"1000\" data-bind=\"message\">Bonjour</textarea>"), "{html}");
        assert!(html.contains("<input type=\"radio\" name=\"choix-taille\" value=\"S\" data-bind=\"taille\"><span>S</span>"), "{html}");
        assert!(html.contains("<input type=\"radio\" name=\"choix-taille\" value=\"M\" data-bind=\"taille\" checked><span>M</span>"), "{html}");
        assert!(html.contains("<option value=\"M\" selected>M</option>"), "{html}");
        // Le choix n'accepte que ses options ; le texte long garde ses retours à la ligne.
        let source = "Page(state: State(taille: \"\", message: \"\"), children: [ Choice(value: taille, label: \"T\", options: [\"S\", \"M\"]), Input(value: message, label: \"M\", lines: 3) ])";
        let start_value = crate::initial_state(source);
        assert!(crate::input(source, &start_value, "taille", "M").contains("taille='M"));
        assert!(!crate::input(source, &start_value, "taille", "XXL").contains("XXL"));
        let two_lines = crate::input(source, &start_value, "message", "a\nb");
        assert!(two_lines.contains("message='a%0Ab"));
        // Relu pour le geste suivant, le texte long garde ses deux lignes.
        assert!(crate::input(source, &two_lines, "taille", "S").contains("message='a%0Ab"));
        // Ce qui est refusé.
        for (source, message) in [
            ("Page(lang: \"français\", children: [])", "attend une langue"),
            ("Page(children: [ Video(source: \"f.avi\", label: \"x\") ])", "en .mp4 ou .webm"),
            ("Page(children: [ Video(source: \"f.mp4\") ])", "attend « label »"),
            ("Page(children: [ Table(head: [\"a\", \"b\"], rows: [ [\"1\"] ]) ])", "autant de cases"),
            ("Page(state: State(n: 0), children: [ Input(value: n, label: \"x\", lines: 3) ])", "sa valeur est un texte"),
            ("Page(state: State(n: 0), children: [ Choice(value: n, label: \"x\", options: [\"a\", \"b\"]) ])", "un choix présente un texte"),
            ("Page(state: State(t: \"\"), children: [ Choice(value: t, label: \"x\", options: [\"a\"]) ])", "de 2 à 20 textes"),
            ("Page(state: State(t: \"\"), children: [ Choice(value: t, label: \"x\", options: [\"a\", \"a\"]) ])", "le même texte"),
        ] {
            let error = crate::check_page(source).err().or_else(|| crate::flat_view(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn batch_6_computing_and_formats() {
        crate::set_now([2026, 10, 6, 2, 9, 5]);
        let source = "Page(state: State(n: 10, total: 123450), children: [ P(\"{weekday:name} {day} {month:name}, {hour} h {minute:00} ; {total:cents} ; {n:number}\"), Repeat(items: [ Item(price: 1999) ], children: [ P(\"{item.price:cents}\") ]) ], rules: [])";
        let html = crate::flat_view(source, "").unwrap();
        assert!(html.contains("<span data-state=\"weekday\" data-format=\"name\">mardi</span> <span data-state=\"day\">6</span> <span data-state=\"month\" data-format=\"name\">octobre</span>, <span data-state=\"hour\">9</span> h <span data-state=\"minute\" data-format=\"00\">05</span> ; <span data-state=\"total\" data-format=\"cents\">1\u{202F}234,50</span> ; <span data-state=\"n\" data-format=\"number\">10</span>"), "{html}");
        assert!(html.contains("<p class=\"holo-P\">19,99</p>"), "{html}");
        // En anglais, les séparateurs et les noms changent.
        let english = crate::flat_view(&source.replace("Page(state", "Page(lang: \"en\", state"), "").unwrap();
        assert!(english.contains(">Tuesday<") && english.contains(">1,234.50<") && english.contains("<p class=\"holo-P\">19.99</p>"), "{english}");
        // Multiplier, diviser.
        let source = "Page(state: State(a: 7, b: 0), children: [ Button(name: M, text: \"m\"), Button(name: D, text: \"d\"), Button(name: Z, text: \"z\") ], rules: [ On(M.tap, effect: a.mul(3)), On(D.tap, effect: a.div(2)), On(Z.tap, effect: a.div(b)) ])";
        assert_eq!(crate::arbitrate(source, "a=7;b=0", "M.tap"), "a=21;b=0");
        assert_eq!(crate::arbitrate(source, "a=7;b=0", "D.tap"), "a=3;b=0");
        assert_eq!(crate::arbitrate(source, "a=7;b=0", "Z.tap"), "a=7;b=0");
        for (source, message) in [
            ("Page(state: State(a: 1), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: a.div(0)) ])", "on ne divise pas par 0"),
            ("Page(state: State(a: 1), children: [ P(\"{a:euros}\") ])", "format inconnu"),
            ("Page(state: State(a: 1), children: [ P(\"{a:name}\") ])", "seuls weekday et month"),
            ("Page(state: State(t: \"\"), children: [ P(\"{t:00}\") ])", "un format s'applique à un nombre"),
            ("Page(children: [ Repeat(items: [ Item(t: \"x\") ], children: [ P(\"{item.t:cents}\") ]) ])", "doit être un nombre entier"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn batch_8_aside_links_captions_lazy_images_and_print() {
        let source = "Page(lang: \"en\", children: [ H1(\"a\"), Image(source: \"a.png\", alt: \"A\"), Aside(children: [ P(\"x\") ]), A(\"Wiki\", to: \"https://example.com/w\", newTab: true), A(\"Plan\", to: \"plan.pdf\", download: true), Image(source: \"b.png\", alt: \"B\"), Video(source: \"f.mp4\", label: \"F\", captions: \"f.vtt\") ])\nNav { print: { display: none; } }";
        crate::check_page(source).unwrap();
        let html = page_html(&read(source).unwrap(), "/ex/").unwrap();
        for expected in [
            "<aside class=\"holo-Aside\"><p class=\"holo-P\">x</p></aside>",
            "href=\"https://example.com/w\" target=\"_blank\" rel=\"noopener\">Wiki<span class=\"holo-hidden\"> (opens in a new tab)</span></a>",
            "href=\"/ex/plan.pdf\" download>Plan</a>",
            "<img class=\"holo-Image\" src=\"/ex/a.png\" alt=\"A\">",
            "<img class=\"holo-Image\" src=\"/ex/b.png\" alt=\"B\" loading=\"lazy\" decoding=\"async\">",
            "<track kind=\"captions\" src=\"/ex/f.vtt\" srclang=\"en\" label=\"Captions\" default>",
            "@media print{.holo-Nav{display:none;}}",
        ] {
            assert!(html.contains(expected), "{expected}\n{html}");
        }
        // En français, le texte caché est en français.
        let html = page_html(&read("Page(children: [ A(\"W\", to: \"https://example.com\", newTab: true) ])").unwrap(), "").unwrap();
        assert!(html.contains("(s'ouvre dans un nouvel onglet)"), "{html}");
        for (source, message) in [
            ("Page(children: [ A(\"x\", to: \"https://example.com/a.pdf\", download: true) ])", "fichier rangé à côté"),
            ("Page(children: [ A(\"x\", to: \"b.holo\", download: true) ])", "pas une page .holo"),
            ("Page(children: [ A(\"x\", to: \"a.pdf\", download: true, newTab: true) ])", "pas les deux"),
            ("Page(children: [ A(\"x\", to: \"a.holo\", newTab: yes) ])", "true ou false"),
            ("Page(children: [ Video(source: \"f.mp4\", label: \"F\", captions: \"f.srt\") ])", "en .vtt"),
            ("Page(children: [ Aside(\"x\") ])", "range des blocs"),
            ("Page(children: [ H1(\"x\") ])\nP { print: { display: block; } }", "display"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn batch_5_useful_html_and_the_form() {
        let source = "Page(icon: \"i.svg\", state: State(taille: 60, vies: 2, jour: \"\", nom: \"Ada\", message: \"\"), children: [ H1(\"a\"), A(\"bas\", to: \"#Bas\"), P(\"~~120~~ ==oui== m^2^ H~2~O\"), Image(source: \"g.jpg\", alt: \"x\", phone: \"p.jpg\", caption: \"Une *légende*\"), Sound(source: \"s.wav\", label: \"Le son\"), Slider(value: taille, label: \"T\", min: 20, max: 120), Input(value: jour, label: \"J\", type: date), Progress(value: vies, max: 3, label: \"Vies\"), Details(summary: \"Q ?\", open: true, children: [ P(\"R\") ]), Dialog(name: Fenetre, children: [ P(\"D\") ]), Form(name: Contact, children: [ Input(value: nom, label: \"N\"), Input(value: message, label: \"M\", lines: 3), Slider(value: taille, label: \"T2\", min: 20, max: 120) ]), H2(\"b\", name: Bas) ])";
        let html = crate::flat_view(source, "/ex/").unwrap();
        for expected in [
            " data-icon=\"/ex/i.svg\"",
            "<a class=\"holo-A\" href=\"#Bas\">bas</a>",
            "<h2 class=\"holo-H2\" data-name=\"Bas\" id=\"Bas\">b</h2>",
            "<s>120</s> <mark>oui</mark> m<sup>2</sup> H<sub>2</sub>O",
            "<figure class=\"holo-figure\"><picture><source media=\"(max-width:640px)\" srcset=\"/ex/p.jpg\"><img class=\"holo-Image\" src=\"/ex/g.jpg\" alt=\"x\"></picture><figcaption>Une <em>légende</em></figcaption></figure>",
            "<audio class=\"holo-Sound\" controls preload=\"metadata\" src=\"/ex/s.wav\" aria-label=\"Le son\"></audio>",
            "<input type=\"range\" min=\"20\" max=\"120\" value=\"60\" data-bind=\"taille\">",
            "<input type=\"date\" value=\"\" data-bind=\"jour\">",
            "<progress max=\"3\" value=\"2\" data-progress=\"vies\"></progress>",
            "<details class=\"holo-Details\" open><summary>Q ?</summary><p class=\"holo-P\">R</p></details>",
            "<dialog class=\"holo-Dialog\" data-name=\"Fenetre\"><form method=\"dialog\" class=\"holo-close\">",
            "<form class=\"holo-Form\" data-name=\"Contact\" novalidate>",
        ] {
            assert!(html.contains(expected), "manque : {expected}\n{html}");
        }
        // La glissière reste dans ses bornes ; la date n'accepte qu'une date.
        let start_value = crate::initial_state(source);
        assert!(crate::input(source, &start_value, "taille", "500").starts_with("taille=120;"));
        assert!(crate::input(source, &start_value, "taille", "3").starts_with("taille=20;"));
        assert!(crate::input(source, &start_value, "jour", "2026-10-06").contains("jour='2026%2D10%2D06"));
        assert!(!crate::input(source, &start_value, "jour", "demain").contains("demain"));
        // Le formulaire envoie les valeurs de ses champs, et seulement elles.
        let message = crate::input(source, &start_value, "message", "Bonjour \"toi\"\nà bientôt");
        assert_eq!(crate::submission(source, &message, "Contact"), "{\"form\":\"Contact\",\"values\":{\"nom\":\"Ada\",\"message\":\"Bonjour \\\"toi\\\"\\nà bientôt\",\"taille\":60}}");
        assert_eq!(crate::submission(source, &message, "Personne"), "");
        // Les règles : ouvrir, fermer, envoyer ; l'envoi arrivé ou non.
        crate::check_page("Page(state: State(ok: 0), children: [ Button(name: B, text: \"b\"), Dialog(name: D, children: [ P(\"x\") ]), Form(name: F, children: [ P(\"y\") ]) ], rules: [ On(B.tap, effect: [D.open, F.send]), On(F.sent, effect: [ok.set(1), D.close]), On(F.failed, effect: ok.set(2)) ])").unwrap();
        for (source, message) in [
            ("Page(children: [ A(\"x\", to: \"#Nulle\") ])", "aucun bloc ne s'appelle « Nulle »"),
            ("Page(icon: \"i.gif\", children: [])", "en .png, .svg ou .ico"),
            ("Page(children: [ Image(source: \"a.png\", alt: \"\", caption: 3) ])", "la légende"),
            ("Page(state: State(t: \"\"), children: [ Slider(value: t, label: \"x\") ])", "présente un nombre de la page"),
            ("Page(state: State(n: 5), children: [ Slider(value: n, label: \"x\", min: 9, max: 3) ])", "min doit être plus petit que max"),
            ("Page(state: State(n: 0), children: [ Input(value: n, label: \"x\", type: date) ])", "écrit un texte"),
            ("Page(state: State(t: \"\"), children: [ Input(value: t, label: \"x\", type: week) ])", "date, time, color ou file"),
            ("Page(children: [ Dialog(children: [ P(\"x\") ]) ])", "« Dialog » a un nom"),
            ("Page(children: [ Form(name: A, children: [ Form(name: B, children: []) ]) ])", "un formulaire dans un formulaire"),
            ("Page(state: State(n: 0), children: [ Button(name: B, text: \"b\"), Form(name: F, children: []) ], rules: [ On(B.tap, effect: F.open) ])", "un « Form » offre send"),
        ] {
            let error = crate::check_page(source).err().or_else(|| crate::flat_view(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn batch_4_useful_css() {
        let source = "Page(fonts: [ Font(family: \"Carlito\", source: \"carlito.woff2\") ], children: [ P.carte(\"x\") ])\n\
            Page { --or: #E9B44C; --marge: 16px; background: --or; dark: { --or: #806020; } }\n\
            P { line-height: 1.6; letter-spacing: -0.5px; text-transform: uppercase; text-decoration: line-through; text-shadow: 0 2px 6px #000000AA; phone: { display: none; } }\n\
            .carte { background: url(\"fond.jpg\"); box-shadow: 0 8px 24px #00000080, 0 1px 2px black; padding: --marge; rotate: -2deg; scale: 1.05; transition: 0.3s; hover: { rotate: 0deg; } }";
        let html = crate::flat_view(source, "/ex/").unwrap();
        for expected in [
            "@font-face{font-family:\"Carlito\";src:url(\"/ex/carlito.woff2\");font-display:swap}",
            ".holo-Page{--or:#E9B44C;--marge:16px;background:var(--or);}",
            "@media (prefers-color-scheme:dark){.holo-Page{--or:#806020;}}",
            "@media (max-width:640px){.holo-P{display:none;}}",
            "line-height:1.6;letter-spacing:-0.5px;text-transform:uppercase;text-decoration:line-through;text-shadow:0 2px 6px #000000AA;",
            "background:url(\"/ex/fond.jpg\") center/cover no-repeat;",
            "padding:var(--marge);rotate:-2deg;scale:1.05;transition:background 0.3s,color 0.3s,",
            "@media (hover:hover){.holo-s-carte:hover{rotate:0deg;}}",
        ] {
            assert!(html.contains(expected), "manque : {expected}\n{html}");
        }
        // Avec sa propre durée, la carte ne reçoit pas la durée automatique du survol.
        assert!(!html.contains("scale .15s"), "{html}");
        for (styles, message) in [
            ("P { color: --rouge; }", "« --rouge » n'est définie nulle part"),
            ("Page { --rouge: url(x); }", "une couleur ou une taille"),
            ("P { line-height: 24px; }", "un nombre sans unité"),
            ("P { display: none; }", "phone: { display: none; }"),
            ("P { phone: { display: flex; } }", "ne prend que « none »"),
            ("P { background: linear-gradient(red); }", "un dégradé"),
            ("P { background: url(\"../secret.png\"); }", "une image rangée à côté"),
            ("P { box-shadow: 0 4px; }", "une ombre"),
            ("P { rotate: 3turn; }", "un angle"),
            ("P { transition: 9s; }", "de 0 à 2s"),
            ("P { night: { color: red; } }", "n'est pas un état"),
        ] {
            let source = format!("Page(children: [ P(\"x\") ])\n{styles}");
            let error = crate::check_page(&source).unwrap_err();
            assert!(error.message.contains(message), "{styles}\n→ {error}");
        }
        for (source, message) in [
            ("Page(fonts: [ Font(family: \"A\", source: \"a.exe\") ], children: [])", ".woff2, .woff"),
            ("Page(fonts: [ Font(source: \"a.woff2\") ], children: [])", "attend « family » et « source »"),
            ("Page(fonts: Font(family: \"A\", source: \"a.woff2\"), children: [])", "une liste de polices"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn batch_2_hover_else_wait_time() {
        // Le survol : la page sait quels blocs l'écoutent ; il change des valeurs.
        let source = "Page(state: State(vu: 0), children: [ Column(name: Carte, children: [ P(\"a\") ]), If(vu, is: 1, children: [ P(\"oui\") ], else: [ P(\"non\") ]) ], rules: [ On(Carte.hover, effect: vu.set(1)), On(Carte.hoverEnd, effect: vu.set(0)) ])";
        let html = crate::flat_view(source, "").unwrap();
        assert!(html.contains("<div class=\"holo-Column\" data-name=\"Carte\" data-hover tabindex=\"0\""), "{html}");
        // Le sinon : montré au départ, puisque la condition est fausse ; le « oui » est caché.
        assert!(html.contains("data-if=\"vu|is=1\" hidden><p class=\"holo-P\">oui</p></div><div class=\"holo-If\" data-else=\"vu|is=1\"><p class=\"holo-P\">non</p></div>"), "{html}");
        let hovered = crate::arbitrate(source, &crate::initial_state(source), "Carte.hover");
        assert_eq!(hovered, "vu=1");
        assert_eq!(crate::arbitrate(source, &hovered, "Carte.hoverEnd"), "vu=0");
        // Une attente : une seule fois ; sous une condition, elle ne court que si la condition est vraie.
        let source = "Page(state: State(bonjour: 0, message: 0), children: [ Text(\"{bonjour}\") ], rules: [ After(2s, effect: bonjour.set(1)), If(message, is: 1, rules: [ After(3s, effect: message.set(0)) ]) ])";
        let start_value = crate::initial_state(source);
        assert_eq!(crate::delays(source, &start_value), "2000:1;3000:0");
        assert_eq!(crate::delays(source, "bonjour=0;message=1"), "2000:1;3000:1");
        assert_eq!(crate::arbitrate(source, &start_value, "after:0"), "bonjour=1;message=0");
        assert_eq!(crate::arbitrate(source, "bonjour=1;message=1", "after:1"), "bonjour=1;message=0");
        assert!(crate::flat_view(source, "").unwrap().contains(" data-live"));
        // L'heure du visiteur : donnée au moteur, lue par la page, jamais changée par elle.
        crate::set_now([2026, 10, 6, 2, 14, 5]);
        let source = "Page(children: [ Sound(name: Ding, source: \"d.wav\"), P(\"{hour} h {minute}\"), If(hour, over: 8, under: 18, children: [ P(\"ouvert\") ], else: [ P(\"fermé\") ]) ], rules: [ When(hour, is: 15, effect: Ding.play) ])\n".to_string();
        let html = crate::flat_view(&source, "").unwrap();
        assert!(html.contains("<span data-state=\"hour\">14</span> h <span data-state=\"minute\">5</span>"), "{html}");
        assert!(html.contains("data-else=\"hour|over=8|under=18\" hidden>"), "{html}");
        assert_eq!(crate::initial_state(&source), "hour=14;minute=5");
        // Une minute plus tard, l'heure avance ; à 15 h, la règle qui guette l'heure sonne.
        crate::set_now([2026, 10, 6, 2, 15, 0]);
        assert_eq!(crate::advance_clock(&source, "hour=14;minute=59"), "hour=15;minute=0;!=Ding.play");
        // L'état écrit ne change pas l'heure : c'est celle donnée au moteur.
        assert_eq!(crate::arbitrate(&source, "hour=3;minute=3", "rien"), "hour=15;minute=0");
        assert_eq!(crate::state::from_unix_seconds(0), [1970, 1, 1, 4, 0, 0]);
        assert_eq!(crate::state::from_unix_seconds(1_791_244_800 + 14 * 3600 + 5 * 60), [2026, 10, 6, 2, 14, 5]);
        // Ce qui est refusé.
        for (source, message) in [
            ("Page(state: State(hour: 0), children: [])", "est l'heure du visiteur"),
            ("Page(state: State(n: 0), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: hour.add(1)) ])", "on ne la change pas"),
            ("Page(state: State(n: 0), children: [ Input(value: hour, label: \"h\") ])", "on ne l'écrit pas"),
            ("Page(state: State(n: 0), keep: [hour], children: [ P(\"{hour}\") ])", "ne se garde pas"),
            ("Page(state: State(n: 0), children: [ If(n, is: 0, rules: [ Every(1s, effect: n.add(1)) ], else: [ P(\"x\") ]) ])", "va avec children"),
            ("Page(state: State(n: 0), children: [ If(n, is: 0, children: [ P(\"a\") ], else: P(\"x\")) ])", "va avec children"),
            ("Page(state: State(n: 0), children: [], rules: [ After(effect: n.add(1)) ])", "une attente s'écrit"),
            ("Page(state: State(n: 0), children: [], rules: [ After(2h, effect: n.add(1)) ])", "une attente s'écrit"),
            ("Page(state: State(n: 0), children: [ Point(name: A, seed: 1), P(\"x\") ], rules: [ On(A.hover, effect: A.enter) ])", "demande que le visiteur touche"),
            ("Page(state: State(n: 0), children: [ Main(name: M, children: [ P(\"x\") ]) ], rules: [ On(M.hover, effect: n.add(1)) ])", "signal inconnu « hover »"),
        ] {
            let error = crate::check_page(source).err().or_else(|| crate::flat_view(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn the_layout_of_lot_4_reaches_the_browser() {
        // ADR-069 : ce que l'auteur écrit, et ce que le navigateur reçoit.
        let html = crate::flat_view(
            "Page(children: [ Grid(columns: 3, children: [ P.card(\"a\") ]) ])\nPage { max-width: 960px; computer: { max-width: 1200px; } }\n.card { text-align: justify; line-clamp: 3; cursor: url(\"viseur.svg\"); min-height: 32px; narrow: { padding: 8px; } }",
            "/ex/",
        )
        .unwrap();
        // La largeur de la page, aussi sur un ordinateur.
        assert!(html.contains(".holo-Page{--holo-width:60rem;}"), "{html}");
        assert!(html.contains("@media (min-width:1024px){.holo-Page{--holo-width:75rem;}}"), "{html}");
        // Justifié : les mots se coupent ; trois lignes : les quatre réglages du CSS ; le curseur dessiné : sa forme de secours.
        assert!(html.contains("text-align:justify;hyphens:auto;-webkit-hyphens:auto;"), "{html}");
        assert!(html.contains("line-clamp:3;-webkit-line-clamp:3;display:-webkit-box;-webkit-box-orient:vertical;overflow:hidden;"), "{html}");
        assert!(html.contains("cursor:url(\"/ex/viseur.svg\"),auto;"), "{html}");
        assert!(html.contains("min-height:2rem;"), "{html}");
        // La place : la case marquée par la page, et ce qu'elle contient.
        assert!(html.contains(".holo-s-card.holo-narrow,.holo-narrow .holo-s-card{padding:0.5rem;}"), "{html}");
        // Une page ordinaire laisse le zoom au navigateur ; des points le confient au moteur.
        assert!(!html.contains("data-zoom"), "{html}");
        assert!(crate::flat_view("Page(points: Points(), children: [ \"a\" ])", "").unwrap().contains(" data-zoom"));
        assert!(!crate::flat_view("Page(zoom: Zoom(detach: true), children: [ \"a\" ])", "").unwrap().contains("data-zoom"));
    }

}
