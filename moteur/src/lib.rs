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

pub mod blocks;
pub mod components;
pub mod state;
pub mod files;
pub mod format;
pub mod seed;
pub mod holo;
pub mod lists;
pub mod modules;
pub mod mosaic;
pub mod movement;
pub mod navigation;
pub mod tools;
pub mod flat;
pub mod rules;
pub mod repeat;
pub mod styles;
pub mod universe;
pub mod view;

#[cfg(all(target_arch = "wasm32", feature = "drawing"))]
mod renderer;
#[cfg(all(target_arch = "wasm32", feature = "drawing"))]
mod web;
#[cfg(target_arch = "wasm32")]
mod web_page;

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
    state::check_state(&program)?;
    rules::check_rules(&program)?;
    view::settings(&program)?;
    // Les modules enfermés, et leur annonce en haut du fichier (ADR-045).
    modules::modules(&program, &state::initial(&program)?)?;
    // Les fichiers qu'un formulaire envoie (ADR-059).
    files::files(&program)?;
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

/// L'éditeur (ADR-046) : le fichier est-il juste ? `ok`, ou la première faute, telle que le
/// moteur la refuse : `ligne 7, colonne 5 : « h1 » : … écris « H1 »`. Un point seul (`Point(…)`)
/// se vérifie comme un monde ; un morceau importé (`Component(…)`), pour ses blocs et ses styles :
/// le reste se vérifie dans la page qui l'importe.
pub fn check_text(source: &str) -> String {
    // Un fichier de styles seuls (ADR-052) : ses styles se vérifient comme ceux d'un morceau.
    if holo::read(source).is_err() {
        let styles = format!("Component(name: HoloStyles, children: []) {source}");
        if let Ok(p) = holo::read(&styles) {
            if !p.styles.is_empty() {
                return match styles::check_styles(&p) {
                    Ok(()) => "ok : un fichier de styles, à importer dans une page".into(),
                    Err(e) => e.to_string(),
                };
            }
        }
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
        ("Repeat", &["items", "over", "children", "rules"]),
        ("Item", &["key"]),
        ("Data", &["from", "every"]),
        ("Enter", MOVEMENT),
        ("Loop", &cycle),
        ("Zoom", &["active", "max", "shrink", "levels", "speed"]),
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
        list(&["enter", "leave", "play", "stop", "portals", "open", "close", "send", "run"]),
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

/// Les effets que les règles du fichier demandent pour un signal, comme `Open.tap`.
pub fn effects(source: &str, signal: &str) -> Vec<String> {
    check_page(source).map(|program| rules::effects(&program, signal)).unwrap_or_default()
}

/// L'état entier, tel qu'il voyage entre le moteur et la page : les nombres, ce que le moteur
/// calcule, puis les textes. `cart=2;count=2;total=240;buyer='Ada`.
fn write_all(program: &Program, numbers: &state::State, texts: &state::Texts, lists: &lists::Lists) -> String {
    // Les sons demandés par une règle de temps ou une règle qui guette suivent l'état, sous le
    // nom « ! » : ce n'est pas une valeur, la page le lit et le retire.
    let capabilities = state::requested_capabilities();
    let sounds = if capabilities.is_empty() { String::new() } else { format!("!={}", capabilities.join(",")) };
    [state::write(&state::to_show(program, numbers)), state::write_texts(texts), lists::write(lists), sounds].into_iter().filter(|chunk| !chunk.is_empty()).collect::<Vec<_>>().join(";")
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
    let numbers = state::arbitrate(&program, &state::reread(&program, state), base);
    let (texts, lists) = lists::arbitrate(&program, &numbers, &state::reread_texts(&program, state), &lists::reread(&program, state), signal);
    write_all(&program, &numbers, &texts, &lists)
}

/// Les horloges d'une page, une par règle `Every` : son rythme en millisecondes et la valeur
/// qu'elle fait changer. `1000:time;2000:starX`.
pub fn clocks(source: &str) -> String {
    check_page(source).map(|program| state::clocks(&program).iter().map(|(ms, value)| format!("{ms}:{value}")).collect::<Vec<_>>().join(";")).unwrap_or_default()
}

/// Les attentes d'une page (`After`), et si chacune court pour cet état : `3000:1;5000:0` (ADR-039).
pub fn delays(source: &str, state: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::delays(&program, &state::reread(&program, state)).iter().map(|(ms, short)| format!("{ms}:{}", u8::from(*short))).collect::<Vec<_>>().join(";")
}

/// Ce qu'un formulaire de la page envoie au serveur, en JSON (ADR-042). Vide s'il n'existe pas.
pub fn submission(source: &str, state: &str, form_name: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::submission(&program, &state::reread(&program, state), &state::reread_texts(&program, state), form_name).unwrap_or_default()
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
    let Some(module) = modules::modules(&program, &numbers).ok().and_then(|m| m.into_iter().find(|m| m.name == name)) else { return String::new() };
    let entry = module.entry.and_then(|e| numbers.iter().find(|(c, _)| c == e)).map_or(0, |(_, v)| *v);
    format!("{}|{entry}|{}|{}", module.source, module.time, module.pages)
}

/// Le module a rendu son nombre : le nouvel état, après `Nom.done`.
pub fn module_finished(source: &str, state: &str, name: &str, value: u64) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    write_all(&program, &modules::finished(&program, &state::reread(&program, state), name, value), &state::reread_texts(&program, state), &lists::reread(&program, state))
}

/// Les lignes d'une liste pour cet état (ADR-044) : la page les pose à la place des anciennes.
pub fn list_html(source: &str, base: &str, state: &str, name: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    flat::list_lines(&program, base, &state::reread(&program, state), &state::reread_texts(&program, state), &lists::reread(&program, state), name)
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
        write_all(&program, &numbers, &state::input_text(&program, &texts, name, written), &lists::reread(&program, state))
    } else {
        write_all(&program, &state::input(&program, &numbers, name, written), &texts, &lists::reread(&program, state))
    }
}

/// D'où viennent les données de la page, et à quel rythme : `stock.json|30000` (0 : une seule
/// fois). Vide si la page n'en demande pas.
pub fn data(source: &str) -> String {
    check_page(source).ok().and_then(|program| state::data_source(&program).ok().flatten()).map(|(file, rhythm)| format!("{file}|{rhythm}")).unwrap_or_default()
}

/// Les données viennent d'arriver du serveur : l'arbitre les range et rend le nouvel état.
pub fn receive(source: &str, state: &str, json: &str) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    let (numbers, texts) = state::receive(&program, &state::reread(&program, state), &state::reread_texts(&program, state), json);
    // Les listes aussi : un tableau de textes, ou d'objets (ADR-051).
    let lists = lists::receive(&program, &lists::reread(&program, state), json);
    write_all(&program, &numbers, &texts, &lists)
}

/// Le visiteur fait glisser un bloc d'un plateau : l'arbitre rend le nouvel état.
pub fn drag(source: &str, state: &str, name: &str, x: u32, y: u32) -> String {
    let Ok(program) = check_page(source) else { return String::new() };
    state::requested_capabilities();
    write_all(&program, &state::drag(&program, &state::reread(&program, state), name, u64::from(x), u64::from(y)), &state::reread_texts(&program, state), &lists::reread(&program, state))
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
            let mut shown = state::with_texts(&state::to_show(&program, &state::reread(&program, state)), &state::reread_texts(&program, state));
            shown.extend(lists::counts(&lists::reread(&program, state)));
            state::conditions(&program, &shown).iter().map(|(key, real_one)| format!("{key}:{}", u8::from(*real_one))).collect::<Vec<_>>().join(";")
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
        Value::Number { value, unit: None } => Some(value.to_string()),
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
    fn the_guide_examples_are_accepted_by_the_engine() {
        // Un dépôt extrait sous Windows peut avoir des fins de ligne « \r\n » (relevé par Codex).
        let guide = include_str!("../../docs/01-holocode/GUIDE.md").replace("\r\n", "\n");
        let examples: Vec<&str> = guide.split("```holo
").skip(1).map(|suite| suite.split("```").next().unwrap()).collect();
        assert!(examples.len() >= 11, "le guide a perdu ses exemples : {}", examples.len());
        for example in examples {
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
