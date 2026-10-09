//! Moteur HoloCode, sprint Big Bang.
//!
//! Le cœur est en Rust pur et se teste sur le PC (`cargo test`) :
//! - `holo` lit un fichier `.holo` ;
//! - `blocs` vérifie que chaque bloc existe et que les titres ne sautent pas de niveau ;
//! - `styles` vérifie les styles, écrits comme en CSS ;
//! - `regles` vérifie les noms, les règles et les budgets, et dit ce qu'un signal demande ;
//! - `etat` tient les valeurs d'une page et arbitre les demandes qui les changent ;
//! - `plat` fabrique la page web ordinaire d'un fichier (la vue à plat) ;
//! - `univers` en fait un monde, entièrement calculé à partir d'une graine ;
//! - `navigation` gère le morcellement, le zoom, l'entrée et la sortie.
//!
//! La partie qui parle au navigateur et à la carte graphique (`web`, `rendu`) n'est
//! compilée que pour WebAssembly.

pub mod address;
pub mod blocks;
pub mod chart;
pub mod components;
pub mod computed;
pub mod dates;
pub mod drawing;
pub mod state;
pub mod stopwatch;
pub mod files;
pub mod fonts;
pub mod format;
pub mod gestures;
pub mod seed;
pub mod holo;
pub mod history;
pub mod lists;
pub mod modules;
pub mod mosaic;
pub mod movement;
pub mod navigation;
pub mod tools;
pub mod flat;
pub mod rules;
pub mod repeat;
pub mod shared;
pub mod styles;
pub mod universe;
pub mod view;

#[cfg(all(target_arch = "wasm32", feature = "drawing"))]
mod renderer;
#[cfg(all(target_arch = "wasm32", feature = "drawing"))]
mod web;
#[cfg(target_arch = "wasm32")]
mod web_page;
// Le serveur (`holo serve`, ADR-074) : seulement sur le PC.
#[cfg(not(target_arch = "wasm32"))]
pub mod server;

use holo::{Error, Program, Value};
use universe::PointDecl;

/// Lit et vérifie un fichier `.holo`, sans rien exécuter. Toute erreur est rendue avec sa
/// ligne et sa colonne.
pub fn check(source: &str) -> Result<PointDecl, Error> {
    let program = holo::read(source)?;
    blocks::check_blocks(&program)?;
    styles::check_styles(&program)?;
    universe::point_from(&program)
}

/// Lit et vérifie un fichier `.holo` entier : blocs, titres, styles, noms, règles, budgets.
pub fn check_page(source: &str) -> Result<Program, Error> {
    let program = holo::read(source)?;
    blocks::check_blocks(&program)?;
    styles::check_styles(&program)?;
    // Les valeurs partagées (ADR-079) : rien ne les change sans passer par le serveur.
    shared::check(&program)?;
    // Les listes calculées (lot 2 du web) : d'abord, car les lignes et les règles les nomment.
    computed::check(&program)?;
    state::check_state(&program)?;
    rules::check_rules(&program)?;
    view::settings(&program)?;
    // Les modules enfermés, et leur annonce en haut du fichier (ADR-045).
    modules::modules(&program)?;
    // Les mesures d'un dessin liées à des valeurs de la page (ADR-086).
    drawing::check(&program)?;
    // Les fichiers qu'un formulaire envoie (ADR-059).
    files::files(&program)?;
    // Les chronomètres et la valeur de leur temps (ADR-089).
    stopwatch::check(&program)?;
    // Ce que l'affichage refuserait (une adresse en `javascript:`, une image hors du dossier)
    // est refusé dès la vérification : on fabrique la page à blanc (revue Codex, B-11).
    if program.root.name == "Page" {
        flat::page_html(&program, "")?;
    }
    Ok(program)
}

/// La page a-t-elle besoin du dessin ? Oui si elle montre des points (`Point`, un monde), si ses
/// pixels deviennent des points au zoom (`points:`, `pixels:`), si elle tourne (`Relief(tilt:)`),
/// ou si c'est un monde seul. Sinon, le moteur léger suffit (ADR-053).
pub fn needs_drawing(source: &str) -> bool {
    let Ok(program) = holo::read(source) else { return true };
    if program.root.name != "Page" {
        return true;
    }
    if let Ok(r) = view::settings(&program) {
        if r.active_points || r.angle_max > 0.0 {
            return true;
        }
    }
    let mut drawing = false;
    let _ = rules::for_each_block(&program.root, &mut |block| {
        if matches!(block.name.as_str(), "Point" | "World") {
            drawing = true;
        }
        Ok(())
    });
    drawing
}

/// Un fichier de styles seuls (ADR-052) ne se lit pas comme une page : ses styles se lisent
/// derrière un morceau vide, écrit sur une ligne à part, pour que les fautes gardent leur ligne
/// (moins une) et leur colonne.
fn styles_only(source: &str) -> Option<holo::Program> {
    if holo::read(source).is_ok() {
        return None;
    }
    let program = holo::read(&format!("Component(name: HoloStyles, children: [])
{source}")).ok()?;
    (!program.styles.is_empty()).then_some(program)
}

/// Pour la ligne de commande : ce fichier est-il un fichier de styles seuls (un thème) ?
pub fn is_styles_file(source: &str) -> bool {
    styles_only(source).is_some()
}

/// L'éditeur (ADR-046) : le fichier est-il juste ? `ok`, ou la première faute, telle que le
/// moteur la refuse : `ligne 7, colonne 5 : « h1 » : … écris « H1 »`. Un point seul (`Point(…)`)
/// se vérifie comme un monde ; un morceau importé (`Component(…)`), pour ses blocs et ses styles :
/// le reste se vérifie dans la page qui l'importe.
pub fn check_text(source: &str) -> String {
    // Un fichier de styles seuls (ADR-052) : ses styles se vérifient comme ceux d'un morceau.
    if let Some(program) = styles_only(source) {
        return match styles::check_styles(&program) {
            Ok(()) => "ok : un fichier de styles, à importer dans une page".into(),
            // La ligne ajoutée devant ne compte pas : la faute garde sa place dans le fichier.
            Err(mut error) => {
                error.pos.line = error.pos.line.saturating_sub(1);
                error.to_string()
            }
        };
    }
    let root = holo::read(source).map(|p| p.root.name);
    let result = match root.as_deref() {
        Ok("Point") => check(source).map(|_| ()),
        Ok("Component") => holo::read(source).and_then(|p| blocks::check_blocks(&p).and_then(|()| styles::check_styles(&p))),
        _ => check_page(source).map(|_| ()),
    };
    match result {
        Ok(()) if root.as_deref() == Ok("Component") => "ok : un morceau, à vérifier aussi dans la page qui l'importe".into(),
        Ok(()) => "ok".into(),
        Err(error) => error.to_string(),
    }
}

/// Tous les mots du langage, pour l'éditeur (ADR-046) : il les propose pendant qu'on écrit, pour
/// qu'on les touche au lieu de les taper (une majuscule au milieu d'un mot coûte cher sur un
/// téléphone, ADR-037). En JSON.
pub fn vocabulary() -> String {
    fn list(words: &[&str]) -> String {
        format!("[{}]", words.iter().map(|m| format!("\"{m}\"")).collect::<Vec<_>>().join(","))
    }
    const MOVEMENT: &[&str] = &["opacity", "x", "y", "scale", "rotate", "flip", "tilt", "blur", "hue", "round", "at", "for", "ease", "letters", "each"];
    let mut cycle: Vec<&str> = MOVEMENT.to_vec();
    cycle.push("back");
    let others: [(&str, &[&str]); 10] = [
        ("Repeat", &["items", "over", "key", "empty", "children", "rules"]),
        ("Item", &["key"]),
        ("Data", &["name", "from", "every"]),
        ("Enter", MOVEMENT),
        ("Loop", &cycle),
        ("Zoom", &["active", "max", "shrink", "levels", "speed", "detach"]),
        ("Points", &["after", "size", "fragment", "divisions", "levels", "density"]),
        ("Relief", &["height", "tilt"]),
        ("Portals", &["layout", "count", "size", "brightness", "duration"]),
        ("Font", &["family", "source"]),
    ];
    let mut params: Vec<String> = blocks::block_params().iter().map(|(block, p)| format!("\"{block}\":{}", list(p))).collect();
    for (block, p) in others {
        if !blocks::block_params().iter().any(|(b, _)| *b == block) {
            params.push(format!("\"{block}\":{}", list(p)));
        }
    }
    let mut settings = styles::setting_names();
    settings.push("display");
    format!(
        "{{\"blocks\":{},\"params\":{{{}}},\"settings\":{},\"states\":{},\"requests\":{},\"signals\":{},\"capabilities\":{},\"keypresses\":{},\"words\":{},\"computed\":{},\"formats\":{}}}",
        list(blocks::BLOCKS),
        params.join(","),
        list(&settings),
        list(holo::STATES),
        list(&["add", "sub", "set", "random", "mul", "div", "push", "remove", "clear"]),
        list(&["tap", "hover", "hoverEnd", "sent", "failed", "done"]),
        list(&["enter", "leave", "play", "stop", "portals", "open", "close", "send", "run", "refresh"]),
        list(state::KEYPRESSES),
        list(&[
            "true", "false", "item", "circle", "square", "triangle", "diamond", "start", "center", "end", "between", "topLeft", "top", "topRight", "left", "right", "bottomLeft", "bottom",
            "bottomRight", "linear", "smooth", "out", "in", "back", "spring", "bounce", "forever", "grid", "row", "column", "diagonal", "date", "time", "color", "none", "uppercase",
            "lowercase", "capitalize", "underline", "line-through", "bold", "italic", "normal", "solid", "dashed", "dotted",
        ]),
        list(&["count", "total", "year", "month", "day", "weekday", "hour", "minute"]),
        list(format::FORMATS),
    )
}

/// La vue à plat d'un fichier `.holo` : une page web ordinaire, fabriquée par le moteur.
pub fn flat_view(source: &str, base: &str) -> Result<String, Error> {
    flat_view_of(source, base, "")
}

/// La vue à plat d'un site du fichier : sa page (chemin vide), ou le monde d'un de ses points
/// (`Shop/Secret`), ouvert en grand comme une page.
pub fn flat_view_of(source: &str, base: &str, path: &str) -> Result<String, Error> {
    let program = check_page(source)?;
    let site = rules::site_of(&program, path)?;
    flat::site_html(&program, site, base, path.rsplit('/').next().unwrap_or(""))
}

/// La page fabriquée par le serveur avec ses données (ADR-064). Le serveur a lu le fichier de
/// `Data(from:)` ; l'arbitre le range, comme dans le navigateur, puis `Shop.done` si les données
/// ont un nom. La page garde ce qu'elle a reçu dans `data-received` : le navigateur rejoue la même
/// réception et part du même état. Des données illisibles : la page de départ, sans rien.
pub fn flat_view_with_data(source: &str, base: &str, json: &str) -> Result<String, Error> {
    flat_view_with_data_at(source, base, json, "")
}

/// La même, pour une adresse qui porte des valeurs après le `?` (ADR-091) : les données d'abord,
/// puis l'adresse, que le visiteur a choisie ; le navigateur les reprend dans le même ordre.
pub fn flat_view_with_data_at(source: &str, base: &str, json: &str, query: &str) -> Result<String, Error> {
    let program = check_page(source)?;
    if state::data_source(&program).ok().flatten().is_none() || !lists::is_json_object(json) {
        return flat_view_at(source, base, query);
    }
    let mut written = receive(source, &initial_state(source), json);
    if let Some(name) = state::data_name(&program) {
        written = arbitrate(source, &written, &format!("{name}.done"));
    }
    if !query.is_empty() {
        written = from_query(source, &written, query);
    }
    let start = (state::reread(&program, &written), state::reread_texts(&program, &written), lists::reread(&program, &written));
    let html = flat::site_html_from(&program, &program.root, base, "", Some(&start))?;
    let received = json.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
    Ok(html.replacen(" data-title=\"", &format!(" data-received=\"{received}\" data-title=\""), 1))
}

/// La page d'un visiteur que `holo serve` connaît (ADR-074) : fabriquée avec ses valeurs, et
/// prête à renvoyer ses gestes au serveur si le navigateur ne lance pas le moteur. La page garde
/// cet état dans `data-visit` : avec JavaScript, le moteur repart de là.
///
/// `tried` : les formulaires qu'il a essayé d'envoyer sans y arriver ; leurs messages d'erreur
/// sont écrits sous les champs, d'après ses valeurs d'aujourd'hui (ADR-075).
pub fn visitor_page(source: &str, base: &str, state: &str, tried: &[String]) -> Result<String, Error> {
    let program = check_page(source)?;
    let start = (state::reread(&program, state), state::reread_texts(&program, state), lists::reread(&program, state));
    let html = with_shared_mark(&program, flat::site_html_from(&program, &program.root, base, "", Some(&start))?, state);
    let written = state.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
    let mut page = gestures::without_script(&html.replacen(" data-title=\"", &format!(" data-visit=\"{written}\" data-title=\""), 1));
    for form in tried {
        let errors: Vec<(String, String)> = form_errors(source, state, form).lines().filter_map(|line| line.split_once('|')).map(|(bind, message)| (bind.to_string(), message.to_string())).collect();
        page = gestures::with_errors(&page, form, &errors);
    }
    Ok(page)
}

/// Ce que devient l'état d'un visiteur quand il envoie un formulaire des gestes, sans
/// JavaScript (ADR-074) : ses champs d'abord, comme s'il venait de les écrire, puis son toucher.
/// Le même arbitre que dans le navigateur ; un geste qui n'est pas un toucher est ignoré.
pub fn visitor_gesture(source: &str, state: &str, fields: &[(String, String)]) -> String {
    let mut written = state.to_string();
    for (name, value) in fields.iter().filter(|(name, _)| name != gestures::SIGNAL) {
        // Un bouton rond envoie son groupe (`choix-size`) : la valeur s'appelle `size`.
        let name = name.strip_prefix("choix-").unwrap_or(name);
        let after = input(source, &written, name, value);
        if !after.is_empty() {
            written = after;
        }
    }
    if let Some((_, signal)) = fields.iter().find(|(name, signal)| name == gestures::SIGNAL && gestures::is_tap(signal)) {
        let after = arbitrate(source, &written, signal);
        if !after.is_empty() {
            written = after;
        }
    }
    // Les sons demandés (« ! ») ne se jouent pas sans le moteur : ils ne sont pas gardés.
    written.split(';').filter(|chunk| !chunk.starts_with("!=")).collect::<Vec<_>>().join(";")
}

/// Les effets que les règles du fichier demandent pour un signal, comme `Open.tap`.
pub fn effects(source: &str, signal: &str) -> Vec<String> {
    check_page(source).map(|program| rules::effects(&program, signal)).unwrap_or_default()
}

/// Les valeurs que la page partage (ADR-079), `seats;likes` ; vide si elle n'en partage pas.
pub fn shared_names(source: &str) -> String {
    check_page(source).map(|program| program.shared.join(";")).unwrap_or_default()
}

/// Les valeurs partagées d'un état, écrites comme l'état : `seats=19;likes=3;last='Ada` (ADR-079).
pub fn shared_of(source: &str, state: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    shared::written(&program, &state::reread(&program, state), &state::reread_texts(&program, state))
}

/// L'état d'un visiteur avec les valeurs partagées que le serveur garde (ADR-079) : celles de
/// `shared` remplacent les siennes, et une valeur absente vaut son départ. Puis les règles qui
/// guettent ont leur mot à dire, comme après des données reçues : `When(seats, is: 0, …)`.
pub fn with_shared(source: &str, state: &str, shared: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    let (numbers, texts, lists) = (state::reread(&program, state), state::reread_texts(&program, state), lists::reread(&program, state));
    let (merged_numbers, merged_texts) = shared::merged(&program, &numbers, &texts, shared);
    let after = state::after_change(&program, numbers, &texts, merged_numbers, &merged_texts);
    write_all(&program, &after, &merged_texts, &lists)
}

/// Ce geste change-t-il une valeur partagée ? Un toucher dont une règle demande de la changer
/// (ADR-079) : la page l'envoie alors au serveur, qui l'arbitre, au lieu de l'arbitrer seule.
pub fn touches_shared(source: &str, signal: &str) -> bool {
    let Ok(program) = check_page(source) else { return false };
    gestures::is_tap(signal) && state::touched_ones(&program, lists::signal_and_line(signal).0).iter().any(|value| program.shared.contains(value))
}

/// Le serveur arbitre un geste sur une page qui partage des valeurs (ADR-079). `state` est l'état
/// du visiteur : il a pu le forger, il est donc relu avec méfiance, et ses valeurs partagées sont
/// remplacées par `shared`, celles que le serveur garde. Seul le toucher d'un bouton que la page
/// montre, pour ces valeurs, est arbitré ; sinon rien ne change. Rend l'état d'après, et si le
/// geste a été accepté. Les sons demandés (« ! ») restent dans l'état : la page du visiteur les joue.
pub fn share(source: &str, state: &str, shared: &str, signal: &str) -> (String, bool) {
    let merged = with_shared(source, state, shared);
    let Ok(program) = check_page(source) else { return (merged, false) };
    if !gestures::is_tap(signal) || !shared::shown(&program, &merged, signal) {
        return (merged, false);
    }
    match arbitrate(source, &merged, signal) {
        after if after.is_empty() => (merged, false),
        after => (cut_shared(&program, &after), true),
    }
}

/// Un texte partagé ne dépasse pas sa longueur permise, même dans l'état du visiteur qui vient
/// de l'écrire (`last.set(name)`) : il voit ce que les autres pages reçoivent.
fn cut_shared(program: &Program, written: &str) -> String {
    written
        .split(';')
        .map(|chunk| match chunk.split_once("='") {
            Some((name, code)) if program.shared.iter().any(|known| known == name) => {
                let text = state::decode(code).unwrap_or_default();
                format!("{name}='{}", state::encode(&text.chars().take(shared::SHARED_TEXT_MAX).collect::<String>()))
            }
            _ => chunk.to_string(),
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// Un geste partagé envoyé par le moteur de la page, en JSON (ADR-079) :
/// `{"signal": "Book.tap", "state": "…"}`. Rend le signal et l'état, ou rien s'il est mal formé.
pub fn read_gesture(json: &str) -> Option<(String, String)> {
    let lists::Json::Object(top) = lists::Json::read(json)? else { return None };
    let text = |key: &str| match top.iter().find(|(known, _)| known == key).map(|(_, value)| value) {
        Some(lists::Json::Text(text)) => Some(text.clone()),
        None => Some(String::new()),
        Some(_) => None,
    };
    let signal = text("signal").filter(|signal| !signal.is_empty())?;
    Some((signal, text("state")?))
}

/// `holo share` (ADR-079), pour un serveur qui ne parle au moteur que par la ligne de commande
/// (le serveur d'essai) : reçoit `{"shared": "seats=20", "state": "…", "signal": "Book.tap"}`,
/// rend `{"accepted": true, "changed": true, "state": "…", "shared": "…"}`. Sans signal, l'état est seulement
/// complété des valeurs partagées. Sans `shared`, elles valent leur départ.
pub fn share_command(source: &str, json: &str) -> Result<String, String> {
    let program = check_page(source).map_err(|e| e.to_string())?;
    if program.shared.is_empty() {
        return Err("cette page ne partage aucune valeur : shared: Shared(seats: 20)".into());
    }
    let Some(lists::Json::Object(top)) = lists::Json::read(json) else { return Err("demande illisible : un objet JSON, {\"shared\": …, \"state\": …, \"signal\": …}".into()) };
    let text = |key: &str| match top.iter().find(|(known, _)| known == key).map(|(_, value)| value) {
        Some(lists::Json::Text(text)) => Ok(text.clone()),
        None => Ok(String::new()),
        Some(_) => Err(format!("« {key} » attend un texte")),
    };
    let (shared, visitor, signal) = (text("shared")?, text("state")?, text("signal")?);
    if !signal.is_empty() && !touches_shared(source, &signal) {
        return Err(format!("« {signal} » ne change aucune valeur partagée : la page le fait seule"));
    }
    let visitor = if visitor.is_empty() { initial_state(source) } else { visitor };
    let (after, accepted) = if signal.is_empty() { (with_shared(source, &visitor, &shared), false) } else { share(source, &visitor, &shared, &signal) };
    // Les valeurs partagées ont-elles changé ? Le serveur les range et les envoie alors en direct.
    let now_shared = shared_of(source, &after);
    let changed = accepted && now_shared != shared_of(source, &with_shared(source, "", &shared));
    Ok(format!("{{\"accepted\":{accepted},\"changed\":{changed},\"state\":{},\"shared\":{}}}", json_text(&after), json_text(&now_shared)))
}

/// Un texte écrit en JSON, entre guillemets.
pub fn json_text(text: &str) -> String {
    let mut written = String::with_capacity(text.len() + 2);
    written.push('"');
    for c in text.chars() {
        match c {
            '"' => written.push_str("\\\""),
            '\\' => written.push_str("\\\\"),
            c if (c as u32) < 0x20 => written.push_str(&format!("\\u{:04x}", c as u32)),
            c => written.push(c),
        }
    }
    written.push('"');
    written
}

/// La page fabriquée avec les valeurs partagées du moment (ADR-079), pour un serveur qui ne
/// connaît pas le visiteur (le serveur d'essai) : l'état de départ, avec ces valeurs.
pub fn shared_page(source: &str, base: &str, shared: &str) -> Result<String, Error> {
    let program = check_page(source)?;
    let written = with_shared(source, &initial_state(source), shared);
    let start = (state::reread(&program, &written), state::reread_texts(&program, &written), lists::reread(&program, &written));
    let html = flat::site_html_from(&program, &program.root, base, "", Some(&start))?;
    Ok(with_shared_mark(&program, html, &written))
}

/// Une page qui partage des valeurs garde celles du moment dans `data-shared` : le moteur du
/// navigateur part d'elles, puis les reçoit en direct (ADR-079).
fn with_shared_mark(program: &Program, html: String, written: &str) -> String {
    if program.shared.is_empty() {
        return html;
    }
    let values = shared::written(program, &state::reread(program, written), &state::reread_texts(program, written));
    let values = values.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
    html.replacen(" data-title=\"", &format!(" data-shared=\"{values}\" data-title=\""), 1)
}

/// L'état entier, tel qu'il voyage entre le moteur et la page : les nombres, ce que le moteur
/// calcule, puis les textes. `cart=2;count=2;total=240;buyer='Ada`.
fn write_all(program: &Program, numbers: &state::State, texts: &state::Texts, lists: &lists::Lists) -> String {
    // Les sons demandés par une règle de temps ou une règle qui guette suivent l'état, sous le
    // nom « ! » : ce n'est pas une valeur, la page le lit et le retire.
    let capabilities = state::requested_capabilities();
    let sounds = if capabilities.is_empty() { String::new() } else { format!("!={}", capabilities.join(",")) };
    // Les listes calculées suivent l'état : la page les montre comme les autres, l'arbitre ne
    // les relit jamais (il les refait).
    let (computed_lists, totals) = computed::apply_with_totals(program, numbers, texts, lists);
    let computed = lists::write(&computed_lists);
    // Leurs totaux aussi (`total: matching`), et les nombres de jours (`Days`, ADR-067) : des
    // nombres que la page montre, jamais relus.
    let mut totals = totals;
    totals.extend(computed::days_values(program, texts));
    let totals = state::write(&totals);
    [state::write(&state::to_show(program, numbers)), state::write_texts(texts), lists::write(lists), computed, totals, sounds].into_iter().filter(|chunk| !chunk.is_empty()).collect::<Vec<_>>().join(";")
}

/// Les valeurs d'une page à leur départ, écrites `cart=0;likes=3`, suivies de celles que le
/// moteur calcule quand la page donne des prix (`count`, `total`), puis des textes.
pub fn initial_state(source: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    write_all(&program, &state::initial(&program).unwrap_or_default(), &state::initial_texts(&program), &lists::initial(&program))
}

/// L'arbitre : ce que deviennent les valeurs d'une page quand un signal est émis. L'état
/// reçu est relu avec méfiance : rien n'y passe que la page ne déclare.
pub fn arbitrate(source: &str, state: &str, signal: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    // Un geste d'une ligne (`Done.tap@2`) : les nombres changent comme pour `Done.tap` ; les
    // listes et les textes savent de quelle ligne il vient (ADR-044).
    let base = lists::signal_and_line(signal).0;
    let texts = state::reread_texts(&program, state);
    let numbers = state::arbitrate(&program, &state::reread(&program, state), &texts, base);
    let (after, lists) = lists::arbitrate(&program, &numbers, &texts, &lists::reread(&program, state), signal);
    // Un texte changé par le geste : les règles qui le guettent ont leur mot à dire (ADR-063).
    write_all(&program, &state::after_texts(&program, numbers, &texts, &after), &after, &lists)
}

/// Les horloges d'une page, une par règle `Every` : son rythme en millisecondes et la valeur
/// qu'elle fait changer. `1000:time;2000:starX`.
pub fn clocks(source: &str) -> String {
    check_page(source).map(|program| state::clocks(&program).iter().map(|(ms, value)| format!("{ms}:{value}")).collect::<Vec<_>>().join(";")).unwrap_or_default()
}

/// Les attentes d'une page (`After`), et si chacune court pour cet état : `3000:1;5000:0` (ADR-039).
pub fn delays(source: &str, state: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::delays(&program, &state::reread(&program, state), &state::reread_texts(&program, state)).iter().map(|(ms, short)| format!("{ms}:{}", u8::from(*short))).collect::<Vec<_>>().join(";")
}

/// Ce qu'un formulaire de la page envoie au serveur, en JSON (ADR-042). Vide s'il n'existe pas.
pub fn submission(source: &str, state: &str, form_name: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::submission(&program, &state::reread(&program, state), &state::reread_texts(&program, state), form_name).unwrap_or_default()
}

/// Ce qui ne va pas dans un formulaire avant de l'envoyer, une ligne par champ : `name|Ce champ est
/// obligatoire.` (ADR-068). Vide : il peut partir.
pub fn form_errors(source: &str, state: &str, form_name: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::form_errors(&program, &state::reread(&program, state), &state::reread_texts(&program, state), form_name)
        .into_iter()
        .map(|(field, message)| format!("{field}|{message}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Pour le serveur : ce qu'un formulaire a envoyé est-il bon ? Une ligne par erreur ; vide s'il
/// l'est (ADR-068).
pub fn check_submission(source: &str, json: &str) -> Result<String, Error> {
    let program = check_page(source)?;
    Ok(state::check_submission(&program, json).into_iter().map(|(field, message)| format!("{field}|{message}")).collect::<Vec<_>>().join("\n"))
}

/// Pour le serveur : les champs de fichier de la page, une ligne chacun (ADR-059).
pub fn files_for_server(source: &str) -> Result<String, Error> {
    files::for_server(&check_page(source)?)
}

/// Une valeur écrite avec son format (ADR-043) : `formater("minute", 5, "00", "fr")` → `05`.
pub fn format_value(name: &str, value: u64, format: &str, language: &str) -> String {
    format::format_value(name, value, format, language)
}

/// La page lit-elle l'heure du visiteur ? Elle la tient alors à jour, minute après minute.
pub fn reads_time(source: &str) -> bool {
    check_page(source).is_ok_and(|program| state::reads_time(&program))
}

/// Donne au moteur l'heure du visiteur : année, mois, jour, jour de la semaine (1 lundi), heure, minute.
pub fn set_now(values: [u64; 6]) {
    state::set_now(values);
}

/// Une minute a passé : l'état avec la nouvelle heure, après les règles qui la guettent.
pub fn advance_clock(source: &str, state: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    write_all(&program, &state::advance_clock(&program, state), &state::reread_texts(&program, state), &lists::reread(&program, state))
}

/// Les fichiers qu'une page importe (`commun.holo;pied.holo`), pour qu'on aille les chercher et
/// qu'on les joigne à son texte avant de le donner au moteur.
pub fn imports(source: &str) -> String {
    holo::imports_of(source).map(|names| names.join(";")).unwrap_or_default()
}

/// Les touches du clavier que la page écoute (`left;right`).
pub fn keypresses(source: &str) -> String {
    check_page(source).map(|program| state::keypresses(&program).join(";")).unwrap_or_default()
}

/// Ce que la page doit savoir pour faire tourner un module (ADR-045) : son fichier, le nombre
/// qu'il reçoit (pour cet état), son temps en millisecondes et sa mémoire en pages de 64 Ko.
/// `somme.wasm|10|100|16`. Vide si aucun module ne porte ce nom.
pub fn module_info(source: &str, state: &str, name: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    let numbers = state::reread(&program, state);
    let Some(module) = modules::modules(&program).ok().and_then(|m| m.into_iter().find(|m| m.name == name)) else { return String::new() };
    let entry = module.inputs.first().and_then(|e| numbers.iter().find(|(c, _)| c == e)).map_or(0, |(_, v)| *v);
    format!("{}|{entry}|{}|{}|{}", module.source, module.time, module.pages, u8::from(module.simple(&program)))
}

/// Le module du premier contrat a rendu son nombre : le nouvel état, après `Nom.done`.
pub fn module_finished(source: &str, state: &str, name: &str, value: u64) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    let texts = state::reread_texts(&program, state);
    write_all(&program, &modules::finished(&program, &state::reread(&program, state), &texts, name, value), &texts, &lists::reread(&program, state))
}

/// Ce que reçoit un module du second contrat (ADR-077) : ses valeurs `input`, en un texte JSON.
pub fn module_input(source: &str, state: &str, name: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    let Some(module) = modules::modules(&program).ok().and_then(|m| m.into_iter().find(|m| m.name == name)) else { return String::new() };
    modules::input_json(&program, &state::reread(&program, state), &state::reread_texts(&program, state), &lists::reread(&program, state), &module)
}

/// La réponse d'un module du second contrat (ADR-077) : le nouvel état, après `Nom.done` ; ou la
/// raison du refus (le module a alors échoué, `Nom.failed`).
pub fn module_received(source: &str, state: &str, name: &str, json: &str) -> Result<String, String> {
    let program = check_page(source).map_err(|e| e.message)?;
    state::requested_capabilities();
    let (numbers, texts, lists) = modules::received(&program, &state::reread(&program, state), &state::reread_texts(&program, state), &lists::reread(&program, state), name, json)?;
    Ok(arbitrate(source, &write_all(&program, &numbers, &texts, &lists), &format!("{name}.done")))
}

/// Les lignes d'une liste pour cet état (ADR-044) : la page les pose à la place des anciennes.
pub fn list_html(source: &str, base: &str, state: &str, name: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    let (numbers, texts) = (state::reread(&program, state), state::reread_texts(&program, state));
    let mut lists = lists::reread(&program, state);
    let computed = computed::apply(&program, &numbers, &texts, &lists);
    lists.extend(computed);
    flat::list_lines(&program, base, &numbers, &texts, &lists, name)
}

/// Le dessin d'un graphique pour cet état (ADR-087) : la page le pose à la place de l'ancien
/// quand sa liste change. `spec` est celui que la page a reçu (`bars|sales|amount|day|`) ; vide
/// s'il ne désigne pas une liste de la page.
pub fn chart_html(source: &str, state: &str, spec: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    let Some(spec) = chart::Spec::read(spec) else { return String::new() };
    let (numbers, texts) = (state::reread(&program, state), state::reread_texts(&program, state));
    let mut lists = lists::reread(&program, state);
    let computed = computed::apply(&program, &numbers, &texts, &lists);
    lists.extend(computed);
    let Some((_, elements)) = lists.iter().find(|(name, _)| name == spec.over) else { return String::new() };
    chart::drawing(&spec, elements)
}

/// Les formes d'un dessin venues d'une liste, pour cet état (ADR-088) : la page les pose à la
/// place des anciennes quand la liste change. Vide si la liste n'existe pas.
pub fn shapes_html(source: &str, state: &str, list: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    let (numbers, texts) = (state::reread(&program, state), state::reread_texts(&program, state));
    let mut lists = lists::reread(&program, state);
    let computed = computed::apply(&program, &numbers, &texts, &lists);
    lists.extend(computed);
    lists.iter().find(|(name, _)| name == list).map(|(_, elements)| drawing::listed_shapes(elements)).unwrap_or_default()
}

/// La seconde de l'appareil du visiteur, de 0 à 59 (ADR-089).
pub fn set_second(second: u64) {
    state::set_second(second);
}

/// Le fichier lit-il la seconde ? La page donne alors l'heure chaque seconde (ADR-089).
pub fn reads_seconds(source: &str) -> bool {
    check_page(source).is_ok_and(|program| state::reads_seconds(&program))
}

/// Un chronomètre s'est arrêté (ADR-089) : son temps final, en millisecondes, va dans sa valeur,
/// bornée, et les règles qui la guettent répondent ; puis `Chrono.stopped`.
pub fn stopwatch_stopped(source: &str, state: &str, name: &str, milliseconds: u64) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    let (mut numbers, texts, lists) = (state::reread(&program, state), state::reread_texts(&program, state), lists::reread(&program, state));
    if let Some(value) = stopwatch::value_of(&program, name) {
        numbers = state::received_number(&program, &numbers, &texts, value, milliseconds);
    }
    arbitrate(source, &write_all(&program, &numbers, &texts, &lists), &format!("{name}.stopped"))
}

/// Le titre de la page pour cet état, quand il lit des valeurs (ADR-090) : « Mon panier (3) ».
/// La page le donne à l'onglet quand ses valeurs changent. Vide sans titre.
pub fn page_title(source: &str, state: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    let Some(holo::Value::Text(model)) = program.root.argument("title").map(|a| &a.value) else { return String::new() };
    format::set_decimals(state::decimals(&program));
    let (numbers, texts, lists) = (state::reread(&program, state), state::reread_texts(&program, state), lists::reread(&program, state));
    // Ce qu'un texte peut montrer, comme au premier affichage : les nombres, le nombre d'éléments
    // d'une liste (« {tasks} tâches »), ceux des listes calculées, leurs totaux, les jours.
    let mut shown = state::to_show(&program, &numbers);
    let (computed, totals) = computed::apply_with_totals(&program, &numbers, &texts, &lists);
    shown.extend(lists::counts(&lists));
    shown.extend(lists::counts(&computed));
    shown.extend(totals);
    shown.extend(computed::days_values(&program, &texts));
    flat::plain_text(model, &shown, &texts)
}

/// Les valeurs qu'un signal fait changer (`time;score`) : leurs horloges repartent de zéro.
pub fn touched_ones(source: &str, signal: &str) -> String {
    let signal = lists::signal_and_line(signal).0;
    check_page(source).map(|program| state::touched_ones(&program, signal).join(";")).unwrap_or_default()
}

/// Le visiteur a écrit dans un champ ou coché une case : l'arbitre rend le nouvel état.
pub fn input(source: &str, state: &str, name: &str, written: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    let (numbers, texts) = (state::reread(&program, state), state::reread_texts(&program, state));
    if texts.iter().any(|(known, _)| known == name) {
        // Un texte écrit : les règles qui le guettent ont leur mot à dire (ADR-063).
        let after = state::input_text(&program, &texts, name, written);
        write_all(&program, &state::after_texts(&program, numbers, &texts, &after), &after, &lists::reread(&program, state))
    } else {
        write_all(&program, &state::input(&program, &numbers, &texts, name, written), &texts, &lists::reread(&program, state))
    }
}

/// D'où viennent les données de la page, à quel rythme, et leur nom : `stock.json|30000|Stock`
/// (0 : une seule fois ; sans nom, rien après la seconde barre). Vide si la page n'en demande pas.
pub fn data(source: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    let name = state::data_name(&program).unwrap_or_default();
    state::data_source(&program).ok().flatten().map(|(file, rhythm)| format!("{file}|{rhythm}|{name}")).unwrap_or_default()
}

/// Les données viennent d'arriver du serveur : l'arbitre les range et rend le nouvel état.
pub fn receive(source: &str, state: &str, json: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    let (before_numbers, before_texts) = (state::reread(&program, state), state::reread_texts(&program, state));
    let (numbers, texts) = state::receive(&program, &before_numbers, &before_texts, json);
    // Des données reçues ne changent pas une valeur partagée : seul le serveur la change (ADR-079).
    let (numbers, texts) = if program.shared.is_empty() { (numbers, texts) } else { shared::merged(&program, &numbers, &texts, &shared::written(&program, &before_numbers, &before_texts)) };
    // Les listes aussi : un tableau de textes, ou d'objets (ADR-051).
    let lists = lists::receive(&program, &lists::reread(&program, state), json);
    write_all(&program, &numbers, &texts, &lists)
}

/// Le visiteur fait glisser un bloc d'un plateau : l'arbitre rend le nouvel état.
pub fn drag(source: &str, state: &str, name: &str, x: u32, y: u32) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    let texts = state::reread_texts(&program, state);
    write_all(&program, &state::drag(&program, &state::reread(&program, state), &texts, name, u64::from(x), u64::from(y)), &texts, &lists::reread(&program, state))
}

/// Ce que la page garde d'une visite à l'autre (`keep:`), tiré de cet état : `cart=2;buyer='Ada`.
pub fn to_keep(source: &str, state: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    let kept_values = state::kept_values(&program).unwrap_or_default();
    let numbers: state::State = state::reread(&program, state).into_iter().filter(|(name, _)| kept_values.contains(name)).collect();
    let texts: state::Texts = state::reread_texts(&program, state).into_iter().filter(|(name, _)| kept_values.contains(name)).collect();
    let lists: lists::Lists = lists::reread(&program, state).into_iter().filter(|(name, _)| kept_values.contains(name)).collect();
    [state::write(&numbers), state::write_texts(&texts), lists::write(&lists)].into_iter().filter(|chunk| !chunk.is_empty()).collect::<Vec<_>>().join(";")
}

/// L'historique dans une page (ADR-091) : les valeurs que l'adresse porte (`tab=photos&page=2`,
/// telle qu'après le `?`), posées sur l'état. Seules celles que la page nomme (`address: [tab]`) ;
/// une valeur absente ou mal écrite reprend son départ. L'état tel quel pour une page qui n'en
/// nomme pas.
pub fn from_query(source: &str, state: &str, query: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    if history::names(&program).unwrap_or_default().is_empty() {
        return state.to_string();
    }
    let (numbers, texts) = history::from_query(&program, &state::reread(&program, state), &state::reread_texts(&program, state), query);
    write_all(&program, &numbers, &texts, &lists::reread(&program, state))
}

/// L'adresse que demandent les valeurs de la page, après le `?` : `tab=photos&page=2` ; vide
/// quand elles sont toutes à leur départ (ADR-091).
pub fn address_query(source: &str, state: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    history::query(&program, &state::reread(&program, state), &state::reread_texts(&program, state))
}

/// Les noms des valeurs que la page écrit dans son adresse : `tab,page` (ADR-091).
pub fn address_names(source: &str) -> String {
    check_page(source).ok().and_then(|program| history::names(&program).ok()).unwrap_or_default().join(",")
}

/// La page fabriquée par un serveur pour une adresse qui porte des valeurs après le `?`
/// (ADR-091) : celles que la page nomme, sinon la page de départ.
pub fn flat_view_at(source: &str, base: &str, query: &str) -> Result<String, Error> {
    let program = check_page(source)?;
    if query.is_empty() || history::names(&program)?.is_empty() {
        return flat_view(source, base);
    }
    let written = from_query(source, &initial_state(source), query);
    let start = (state::reread(&program, &written), state::reread_texts(&program, &written), lists::reread(&program, &written));
    flat::site_html_from(&program, &program.root, base, "", Some(&start))
}

/// L'état de départ d'une page, avec ce qu'elle avait gardé d'une visite précédente.
pub fn resume(source: &str, kept: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    let kept_values = state::kept_values(&program).unwrap_or_default();
    let reread_ones = state::reread_texts(&program, kept);
    // Seuls les textes que la page dit garder sont repris ; les autres partent de leur départ.
    let texts: state::Texts = state::initial_texts(&program).into_iter().map(|(name, start_value)| {
        let resumed = reread_ones.iter().find(|(known, _)| *known == name && kept_values.contains(&name)).map(|(_, text)| text.clone());
        (name, resumed.unwrap_or(start_value))
    }).collect();
    // Seules les listes que la page dit garder sont reprises (ADR-044).
    let lists: lists::Lists = lists::reread(&program, kept).into_iter().map(|(name, elements)| if kept_values.contains(&name) { (name, elements) } else { (name.clone(), lists::initial(&program).into_iter().find(|(n, _)| *n == name).map(|(_, e)| e).unwrap_or_default()) }).collect();
    write_all(&program, &state::resume(&program, kept), &texts, &lists)
}

/// Les conditions d'une page (`If`), avec leur réponse pour cet état : `count|is=0:1;total|over=299:0`.
pub fn conditions(source: &str, state: &str) -> String {
    match check_page(source) {
        Ok(program) => {
            let (numbers, texts, lists) = (state::reread(&program, state), state::reread_texts(&program, state), lists::reread(&program, state));
            let mut shown = state::to_show(&program, &numbers);
            shown.extend(lists::counts(&lists));
            // Une liste calculée n'est pas relue de l'état : elle se refait d'après lui (ADR-062).
            let (computed_lists, totals) = computed::apply_with_totals(&program, &numbers, &texts, &lists);
            shown.extend(lists::counts(&computed_lists));
            shown.extend(totals);
            shown.extend(computed::days_values(&program, &texts));
            state::conditions(&program, &shown, &texts).iter().map(|(key, real_one)| format!("{key}:{}", u8::from(*real_one))).collect::<Vec<_>>().join(";")
        }
        Err(_) => String::new(),
    }
}

/// Le fichier `.holo` d'un seul point de la page, pour ouvrir sa vue en profondeur.
pub fn point_source(source: &str, name: &str) -> Option<String> {
    let program = check_page(source).ok()?;
    let point = rules::named_block(&program, name).filter(|block| block.name == "Point")?;
    let write = |value: &Value| match value {
        Value::Name(n) => Some(n.clone()),
        Value::Integer(e) => Some(e.to_string()),
        Value::Number { value, unit: None, .. } => Some(value.to_string()),
        Value::Text(t) => Some(format!("\"{t}\"")),
        Value::List(elements) => {
            let texts: Option<Vec<String>> = elements
                .iter()
                .map(|e| match e {
                    Value::Text(t) => Some(format!("\"{t}\"")),
                    _ => None,
                })
                .collect();
            texts.map(|t| format!("[{}]", t.join(", ")))
        }
        _ => None,
    };
    let settings: Vec<String> = ["name", "seed", "brightness", "fragments", "color", "palette"]
        .iter()
        .filter_map(|param| Some(format!("{param}: {}", write(&point.argument(param)?.value)?)))
        .collect();
    Some(format!("Point({})", settings.join(", ")))
}

/// Les mondes voisins d'un site, calculés à partir d'une graine : ils remplissent le carrefour
/// autour des sites écrits dans le fichier. La graine vient du nom du site, pour que le même
/// fichier montre toujours les mêmes mondes (ADR-008). Rend, pour chacun, sa graine et sa couleur.
pub fn neighbour_worlds(source: &str, path: &str, number: u32) -> Vec<(u64, [f32; 3])> {
    let Ok(program) = check_page(source) else { return Vec::new() };
    let name = path.rsplit('/').next().filter(|n| !n.is_empty()).or_else(|| rules::name_of(&program.root)).unwrap_or("Home");
    let seed = name.bytes().fold(0u64, |g, byte| seed::mix_bits(g ^ u64::from(byte)));
    (0..number)
        .map(|i| {
            let neighbour = seed::child_seed(seed, i);
            (neighbour, universe::World::from_seed(neighbour).children[0].color)
        })
        .collect()
}

/// Le monde où la page est posée quand on la regarde en personnage. Provisoire : tant que
/// le langage ne sait pas écrire « un monde qui contient une page », la graine de ce monde
/// se tire du nom de la page, pour que le même fichier redonne le même lieu (ADR-008).
pub fn home_world(source: &str) -> Option<String> {
    let program = check_page(source).ok()?;
    let name = rules::name_of(&program.root).unwrap_or("Home");
    let seed = name.bytes().fold(0u64, |g, byte| seed::mix_bits(g ^ u64::from(byte)));
    // Yocthan : pas de points décoratifs autour de la page. Le lieu est éteint ; ce sont les
    // éléments de la page eux-mêmes qui deviendront des points (voir `mosaique.rs`).
    Some(format!("Point(name: {name}, seed: {seed}, brightness: 0, fragments: 12)"))
}

#[cfg(test)]
mod editor {
    #[test]
    fn the_editor_receives_the_fault_and_the_vocabulary() {
        assert_eq!(crate::check_text("Page(children: [ H1(\"a\") ])"), "ok");
        assert_eq!(crate::check_text("Page(children: [ h1(\"a\") ])"), "ligne 1, colonne 18 : « h1 » : un nom de bloc commence par une majuscule, écris « H1 »");
        assert!(crate::check_text("Point(name: A, seed: 1)").starts_with("ok"));
        assert!(crate::check_text("Component(name: Menu, children: [ P(\"x\") ])").starts_with("ok : un morceau"));
        let words = crate::vocabulary();
        for expected in ["\"blocks\":[\"Page\",", "\"Page\":[\"name\",", "\"Repeat\":[\"items\"", "\"hoverEnd\"", "\"topRight\"", "\"letter-spacing\"", "\"dark\"", "\"cents\""] {
            assert!(words.contains(expected), "manque {expected} dans {words}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHOP: &str = include_str!("../../exemples/boutique-comparee/boutique.holo");

    #[test]
    fn the_shop_goes_from_page_to_point() {
        assert!(flat_view(SHOP, "").unwrap().contains("<h1 class=\"holo-H1\">My shop</h1>"));
        assert_eq!(effects(SHOP, "Open.tap"), ["Workshop.enter"]);
        let point = point_source(SHOP, "Workshop").unwrap();
        assert_eq!(point, "Point(name: Workshop, seed: 42, brightness: 0.8, fragments: 6, color: \"#E9B44C\", palette: [\"#E9B44C\", \"#245C45\"])");
        // Ce point seul est un fichier que la vue en profondeur accepte.
        let decl = check(&point).unwrap();
        assert_eq!((decl.seed, decl.shatter, decl.palette.len()), (42, 6, 2));
        assert_eq!(point_source(SHOP, "Open"), None);
    }

    #[test]
    fn the_home_world_is_always_the_same() {
        let home = home_world(SHOP).unwrap();
        assert_eq!(Some(home.clone()), home_world(SHOP));
        let decl = check(&home).unwrap();
        assert_eq!((decl.name.as_str(), decl.shatter), ("Shop", 12));
        // Une autre page a un autre lieu.
        assert_ne!(home_world("Page(name: Blog)").unwrap(), home);
        assert!(home_world("Page(children: [ Div() ])").is_none());
    }

    #[test]
    fn the_server_page_starts_with_its_data() {
        // ADR-064 : la page fabriquée par le serveur contient déjà ses données, rangées par
        // l'arbitre, puis `Shop.done` ; une liste calculée suit les données reçues.
        let source = r#"Page(
  state: State(loading: 1, news: "", visitors: 0, works: [ Item(title: "Au départ", image: "a.png") ]),
  data: Data(name: Shop, from: "shop.json"),
  computed: [ Filter(name: sorted, from: works, sortBy: title) ],
  children: [
    If(loading, is: 1, children: [ "Loading…" ]),
    Text("{news} · {visitors}"),
    Repeat(over: sorted, children: [ Text("{item.title}") ]),
  ],
  rules: [ On(Shop.done, effect: loading.set(0)) ],
)"#;
        let json = r#"{ "news": "Open <today>", "visitors": 42, "works": [ {"title": "Zèbre", "image": "z.png"}, {"title": "Arbre", "image": "b.png"} ] }"#;
        let html = flat_view_with_data(source, "", json).unwrap();
        assert!(html.contains(r#"data-if="loading|is=1" hidden>"#), "{html}");
        assert!(html.contains(r#"<span data-state="news">Open &lt;today&gt;</span>"#) && html.contains(r#"<span data-state="visitors">42</span>"#), "{html}");
        let main = &html[html.find("<main>").unwrap()..];
        let (first, last) = (main.find("Arbre").unwrap(), main.find("Zèbre").unwrap());
        assert!(first < last && !html.contains("Au départ"), "{html}");
        assert!(html.contains(r#" data-received="{ &quot;news&quot;: &quot;Open &lt;today&gt;&quot;"#), "{html}");
        // Des données illisibles, ou une page sans `Data` : la page de départ, telle quelle.
        let plain = flat_view(source, "").unwrap();
        assert_eq!(flat_view_with_data(source, "", "{ pas du json").unwrap(), plain);
        assert_eq!(flat_view_with_data(source, "", "[1, 2]").unwrap(), plain);
        assert!(plain.contains(r#"data-if="loading|is=1">"#) && !plain.contains("data-received") && plain.contains("Au départ"));
        let without = source.replace("data: Data(name: Shop, from: \"shop.json\"),", "").replace("rules: [ On(Shop.done, effect: loading.set(0)) ],", "");
        assert_eq!(flat_view_with_data(&without, "", json).unwrap(), flat_view(&without, "").unwrap());
    }

    #[test]
    fn the_guide_examples_are_accepted_by_the_engine() {
        // Un dépôt extrait sous Windows peut avoir des fins de ligne « \r\n » (relevé par Codex).
        let guide = include_str!("../../docs/01-holocode/GUIDE.md").replace("\r\n", "\n");
        let examples: Vec<&str> = guide.split("```holo
").skip(1).map(|suite| suite.split("```").next().unwrap()).collect();
        assert!(examples.len() >= 11, "le guide a perdu ses exemples : {}", examples.len());
        for example in examples {
            // Un modèle d'adresse se nomme sur sa première ligne, `// profil/{id}.holo` (ADR-078) :
            // il se vérifie comme `holo check`, chaque nom valant un texte vide.
            let file = example.trim_start().strip_prefix("// ").and_then(|l| l.lines().next()).unwrap_or("");
            let joined = if file.ends_with(".holo") && !address::names(file).is_empty() { address::joined(example, &address::empty_values(file)) } else { example.to_string() };
            let example = joined.as_str();
            // Une page passe toutes les vérifications et se fabrique ; un point seul s'ouvre en
            // profondeur ; un morceau (un fichier fait pour être importé) est vérifié sans être affiché.
            let start = example.trim_start();
            let result = if start.starts_with("Point(") {
                check(example).map(|_| ())
            } else if start.starts_with("Component(") {
                check_page(example).map(|_| ())
            } else {
                flat_view(example, "").map(|_| ())
            };
            if let Err(error) = result {
                panic!("un exemple du guide est refusé : {error}
{example}");
            }
        }
    }

    #[test]
    fn each_lesson_is_accepted_by_the_engine() {
        // Une leçon par notion, dans exemples/lecons/ : chacune doit marcher telle qu'elle est écrite.
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../exemples/lecons");
        let mut lessons = 0;
        for entry in std::fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("holo") {
                continue;
            }
            let mut source = std::fs::read_to_string(&path).unwrap();
            // Comme le fait la page d'entrée : les fichiers importés sont joints au texte.
            for name in imports(&source).split(';').filter(|n| !n.is_empty()).map(str::to_string).collect::<Vec<_>>() {
                source.push(holo::NEXT_FILE);
                source.push_str(&name);
                source.push(holo::NAME_SEPARATOR);
                source.push_str(&std::fs::read_to_string(folder.join(&name)).unwrap());
            }
            let start = source.lines().find(|l| !l.trim().is_empty() && !l.trim_start().starts_with("//") && !l.starts_with("import")).unwrap_or("");
            // Un fichier de styles seuls (ADR-052) : un thème, importé par une autre leçon.
            let verdict = check_text(&source);
            let result = if verdict.starts_with("ok : un fichier de styles") {
                Ok(())
            } else if start.starts_with("Point(") {
                check(&source).map(|_| ())
            } else if start.starts_with("Component(") {
                check_page(&source).map(|_| ())
            } else {
                flat_view(&source, "").map(|_| ())
            };
            if let Err(error) = result {
                panic!("la leçon {} est refusée : {error}", path.display());
            }
            lessons += 1;
        }
        assert!(lessons >= 27, "des leçons ont disparu : {lessons}");
    }

    #[test]
    fn neighbour_worlds_are_always_the_same() {
        let neighbours = neighbour_worlds(SHOP, "", 9);
        assert_eq!(neighbours.len(), 9);
        assert_eq!(neighbours, neighbour_worlds(SHOP, "", 9));
        let seeds: std::collections::HashSet<u64> = neighbours.iter().map(|(g, _)| *g).collect();
        assert_eq!(seeds.len(), 9, "neuf mondes différents");
        // Un autre site a d'autres voisins ; un fichier refusé n'en a pas.
        assert_ne!(neighbour_worlds(SHOP, "Workshop", 9), neighbours);
        assert!(neighbour_worlds("Page(children: [ Div() ])", "", 9).is_empty());
        // Chacun est un monde que la vue en profondeur sait ouvrir.
        assert!(check(&format!("Point(name: World, seed: {}, fragments: 12)", neighbours[0].0)).is_ok());
    }

    #[test]
    fn the_house_and_its_garden_are_two_valid_files() {
        for file in [include_str!("../../exemples/maison/salon.holo"), include_str!("../../exemples/maison/jardin.holo")] {
            flat_view(file, "/exemples/maison/").unwrap();
        }
        // Le salon a une porte vers le jardin : toucher le point demande d'y entrer.
        assert_eq!(effects(include_str!("../../exemples/maison/salon.holo"), "Jardin.tap"), ["Jardin.enter"]);
    }

    #[test]
    fn checking_refuses_what_rendering_would_refuse() {
        assert!(check_page("Page(children: [ A(\"x\", to: \"javascript:alert(1)\") ])").unwrap_err().message.contains("« to »"));
        assert!(check_page("Page(children: [ Image(source: \"../secret.png\") ])").is_err());
        assert!(check_page("Page(children: [ Point(name: G, seed: 1, inside: \"http://192.168.1.1/x.holo\") ])").is_err());
        assert!(check_page("Page(children: [ A(\"x\", to: \"garden.holo\") ])").is_ok());
    }

    #[test]
    fn a_refused_file_gives_neither_page_nor_effect() {
        let letter_case = SHOP.replace("Workshop.enter", "Workshop.fly");
        assert!(flat_view(&letter_case, "").unwrap_err().message.contains("capacité inconnue"));
        assert!(effects(&letter_case, "Open.tap").is_empty());
    }
}
