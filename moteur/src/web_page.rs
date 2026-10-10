//! La partie du moteur qui parle au navigateur pour la page à plat : lire le fichier, fabriquer
//! la page, arbitrer les valeurs (ADR-053). Elle ne dessine rien : le moteur léger
//! (`--no-default-features`) n'a qu'elle, et pèse bien moins que le moteur entier, qui y ajoute
//! le dessin (`web.rs`, `rendu.rs`).

use wasm_bindgen::prelude::*;

/// Les réglages de vue écrits dans un fichier `.holo` ; sans fichier, les réglages par défaut.
pub(crate) fn settings_of(source: Option<String>) -> Result<crate::view::Settings, JsValue> {
    match source {
        Some(source) => crate::check_page(&source).and_then(|p| crate::view::settings(&p)).map_err(|e| JsValue::from_str(&e.to_string())),
        None => Ok(crate::view::Settings::default()),
    }
}

/// Cette page a-t-elle besoin du dessin (des points, un monde, la vue points, le relief) ? Sinon,
/// le moteur léger suffit, et le dessin n'est jamais téléchargé (ADR-053).
#[wasm_bindgen]
pub fn needs_drawing(source: &str) -> bool {
    crate::needs_drawing(source)
}

/// Le moteur léger ne dessine rien : la page d'entrée lui demande quand même de se mettre en
/// pause, ou combien d'images il a dessinées. Le moteur entier remplace ces trois-là.
#[cfg(not(feature = "drawing"))]
#[wasm_bindgen]
pub fn pause(_active: bool) {}

#[cfg(not(feature = "drawing"))]
#[wasm_bindgen]
pub fn wake() {}

#[cfg(not(feature = "drawing"))]
#[wasm_bindgen]
pub fn frames_drawn() -> u32 {
    0
}

/// Ce que la page d'entrée doit savoir des réglages d'un fichier, dans cet ordre : la densité
/// des points ; si dézoomer réduit la page (1) ou non (0) ; jusqu'à quel grossissement la page
/// reste un site ordinaire ; si le zoom est permis (1) ou non (0) ; la disposition des portails
/// (0 grille, 1 ligne, 2 colonne, 3 diagonale), leur nombre, leur taille, la lumière du fond ;
/// la vitesse du zoom à la molette ; la durée d'ouverture d'un portail, en millisecondes ;
/// jusqu'où la page tourne, en degrés (0 : elle ne tourne pas) ; si les pixels deviennent des
/// points au zoom (1) ou si la page reste un site ordinaire (0).
#[wasm_bindgen]
pub fn view_settings(source: &str) -> Result<Vec<f64>, JsValue> {
    let r = settings_of(Some(source.to_string()))?;
    let layout = match r.portals_layout {
        crate::view::Layout::Grid => 0.0,
        crate::view::Layout::Line => 1.0,
        crate::view::Layout::Column => 2.0,
        crate::view::Layout::Diagonal => 3.0,
    };
    Ok(vec![r.density, f64::from(u8::from(r.reduce)), r.after, f64::from(u8::from(r.zoom_active)), layout, f64::from(r.portals_count), r.portals_size, r.portals_light, r.zoom_speed, r.portals_duration, r.angle_max.to_degrees(), f64::from(u8::from(r.active_points)), f64::from(u8::from(r.detach))])
}

/// Les mondes voisins d'un site, calculés à partir d'une graine, pour remplir le carrefour.
/// Chacun s'écrit « graine:rouge,vert,bleu » ; ils sont séparés par des points-virgules. La
/// graine reste un texte : un nombre de 64 bits ne tient pas dans un nombre de JavaScript.
#[wasm_bindgen]
pub fn neighbour_worlds(source: &str, path: &str, number: u32) -> String {
    let text = |(seed, c): (u64, [f32; 3])| format!("{seed}:{},{},{}", (c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8);
    crate::neighbour_worlds(source, path, number).into_iter().map(text).collect::<Vec<_>>().join(";")
}

/// Le monde où la page est posée quand on la regarde en personnage.
#[cfg(feature = "drawing")]
#[wasm_bindgen]
pub fn home_world(source: &str) -> Option<String> {
    crate::home_world(source)
}

/// La vue à plat : la page web ordinaire d'un fichier `.holo`, fabriquée par le moteur.
#[wasm_bindgen]
pub fn flat_view(source: &str, base: &str, path: Option<String>) -> Result<String, JsValue> {
    crate::flat_view_of(source, base, path.as_deref().unwrap_or("")).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Les effets demandés par un signal (`Open.tap`), séparés par des virgules.
#[wasm_bindgen]
pub fn effects(source: &str, signal: &str) -> String {
    crate::effects(source, signal).join(",")
}

/// Les valeurs d'une page à leur départ (`cart=0;likes=3`).
#[wasm_bindgen]
pub fn initial_state(source: &str) -> String {
    crate::initial_state(source)
}

/// Les horloges d'une page : `1000:time;2000:starX`.
#[wasm_bindgen]
pub fn clocks(source: &str) -> String {
    crate::clocks(source)
}

/// Les attentes d'une page et si chacune court : `3000:1;5000:0`.
#[wasm_bindgen]
pub fn delays(source: &str, state: &str) -> String {
    crate::delays(source, state)
}

/// La seconde de l'appareil du visiteur (ADR-089).
#[wasm_bindgen]
pub fn set_second(second: u32) {
    crate::set_second(u64::from(second));
}

/// Le fichier lit-il la seconde ? (ADR-089)
#[wasm_bindgen]
pub fn reads_seconds(source: &str) -> bool {
    crate::reads_seconds(source)
}

/// Un chronomètre s'est arrêté : son temps final, en millisecondes (ADR-089).
#[wasm_bindgen]
pub fn stopwatch_stopped(source: &str, state: &str, name: &str, milliseconds: f64) -> String {
    crate::stopwatch_stopped(source, state, name, milliseconds.max(0.0) as u64)
}

/// La page lit-elle `scroll`, la place du visiteur dans la page ? (ADR-106)
#[wasm_bindgen]
pub fn reads_scroll(source: &str) -> bool {
    crate::reads_scroll(source)
}

/// Le visiteur a défilé : sa place, de 0 à 100, et le nouvel état (ADR-106).
#[wasm_bindgen]
pub fn scrolled(source: &str, state: &str, value: u32) -> String {
    crate::scrolled(source, state, u64::from(value))
}

/// Le titre de la page pour cet état (ADR-090).
#[wasm_bindgen]
pub fn page_title(source: &str, state: &str) -> String {
    crate::page_title(source, state)
}

/// Les valeurs que l'adresse porte après le `?`, posées sur l'état (ADR-091).
#[wasm_bindgen]
pub fn from_query(source: &str, state: &str, query: &str) -> String {
    crate::from_query(source, state, query)
}

/// L'adresse que demandent les valeurs de la page, après le `?` (ADR-091).
#[wasm_bindgen]
pub fn address_query(source: &str, state: &str) -> String {
    crate::address_query(source, state)
}

/// Les noms des valeurs que la page écrit dans son adresse : `tab,page` (ADR-091).
#[wasm_bindgen]
pub fn address_names(source: &str) -> String {
    crate::address_names(source)
}

/// Ce qu'il faut pour faire tourner un module : `somme.wasm|10|100|16|1` (le dernier chiffre : le
/// premier contrat suffit, ADR-077).
#[wasm_bindgen]
pub fn module_info(source: &str, state: &str, name: &str) -> String {
    crate::module_info(source, state, name)
}

/// Le module a rendu son nombre : le nouvel état.
#[wasm_bindgen]
pub fn module_finished(source: &str, state: &str, name: &str, value: f64) -> String {
    crate::module_finished(source, state, name, value.max(0.0) as u64)
}

/// Ce que reçoit un module du second contrat, en JSON (ADR-077).
#[wasm_bindgen]
pub fn module_input(source: &str, state: &str, name: &str) -> String {
    crate::module_input(source, state, name)
}

/// La réponse d'un module du second contrat : le nouvel état, ou la raison du refus.
#[wasm_bindgen]
pub fn module_received(source: &str, state: &str, name: &str, json: &str) -> Result<String, JsValue> {
    crate::module_received(source, state, name, json).map_err(|reason| JsValue::from_str(&reason))
}

/// Ce que la boîte refuse dans un fichier de module, avant de le lancer : vide, ou la raison (ADR-118).
#[wasm_bindgen]
pub fn module_check(bytes: &[u8], pages: u32) -> String {
    crate::module_check(bytes, pages)
}

/// Les formes d'un dessin venues d'une liste, pour cet état (ADR-088).
#[wasm_bindgen]
pub fn shapes_html(source: &str, state: &str, list: &str) -> String {
    crate::shapes_html(source, state, list)
}

/// Le dessin d'un graphique pour cet état (ADR-087).
#[wasm_bindgen]
pub fn chart_html(source: &str, state: &str, spec: &str) -> String {
    crate::chart_html(source, state, spec)
}

/// Les suggestions d'un champ venues d'une liste, pour cet état (ADR-100).
#[wasm_bindgen]
pub fn suggestions_html(source: &str, state: &str, list: &str) -> String {
    crate::suggestions_html(source, state, list)
}

/// Les lignes d'une liste pour cet état.
#[wasm_bindgen]
pub fn list_html(source: &str, base: &str, state: &str, name: &str) -> String {
    crate::list_html(source, base, state, name)
}

/// Ce qu'un formulaire envoie au serveur, en JSON.
#[wasm_bindgen]
pub fn submission(source: &str, state: &str, form_name: &str) -> String {
    crate::submission(source, state, form_name)
}

/// Une valeur écrite avec son format, dans la langue de la page.
#[wasm_bindgen]
pub fn format_value(name: &str, value: f64, format: &str, language: &str) -> String {
    // Un nombre négatif (ADR-102) garde son signe : la page lit « -5 » dans l'état, le moteur
    // l'écrit avec le signe moins de la langue. Un nombre positif ne change pas.
    crate::format_value(name, crate::negative::stored(value as i64), format, language)
}

/// Ce qui ne va pas dans un formulaire avant de l'envoyer (ADR-068) : `name|message`, une ligne
/// par champ.
#[wasm_bindgen]
pub fn form_errors(source: &str, state: &str, form_name: &str) -> String {
    crate::form_errors(source, state, form_name)
}

/// Ce qui ne va pas dans le mot de passe d'un formulaire (ADR-114), d'après sa seule longueur, en
/// caractères : `holo-password|message`. Le mot de passe lui-même n'entre jamais dans le moteur.
#[wasm_bindgen]
pub fn password_errors(source: &str, form_name: &str, length: usize, secure: bool) -> String {
    crate::password_errors(source, form_name, length, secure)
}

/// Une date « 2026-10-07 » dans la langue de la page : « 7 octobre 2026 », « mercredi » (ADR-067).
#[wasm_bindgen]
pub fn format_date(text: &str, format: &str, language: &str) -> String {
    crate::dates::format(text, format, language)
}

/// La page lit-elle l'heure du visiteur ?
#[wasm_bindgen]
pub fn reads_time(source: &str) -> bool {
    crate::reads_time(source)
}

/// L'heure de l'appareil du visiteur, donnée au moteur.
#[wasm_bindgen]
pub fn set_now(year: u32, month: u32, day: u32, week: u32, hour: u32, minute: u32) {
    crate::set_now([year, month, day, week, hour, minute].map(u64::from));
}

/// Une minute a passé : le nouvel état.
#[wasm_bindgen]
pub fn advance_clock(source: &str, state: &str) -> String {
    crate::advance_clock(source, state)
}

/// Les fichiers qu'une page importe.
#[wasm_bindgen]
pub fn imports(source: &str) -> String {
    crate::imports(source)
}

/// Les touches du clavier que la page écoute.
#[wasm_bindgen]
pub fn keypresses(source: &str) -> String {
    crate::keypresses(source)
}

/// Les valeurs qu'un signal fait changer : leurs horloges repartent de zéro.
#[wasm_bindgen]
pub fn touched_ones(source: &str, signal: &str) -> String {
    crate::touched_ones(source, signal)
}

/// Le visiteur a écrit dans un champ ou coché une case.
#[wasm_bindgen]
pub fn input(source: &str, state: &str, name: &str, written: &str) -> String {
    crate::input(source, state, name, written)
}

/// D'où viennent les données de la page, et à quel rythme.
#[wasm_bindgen]
pub fn data(source: &str) -> String {
    crate::data(source)
}

/// Les données viennent d'arriver du serveur.
#[wasm_bindgen]
pub fn receive(source: &str, state: &str, json: &str) -> String {
    crate::receive(source, state, json)
}

/// Le visiteur fait glisser un bloc d'un plateau.
#[wasm_bindgen]
pub fn drag(source: &str, state: &str, name: &str, x: u32, y: u32) -> String {
    crate::drag(source, state, name, x, y)
}

/// Ce que la page garde d'une visite à l'autre.
#[wasm_bindgen]
pub fn to_keep(source: &str, state: &str) -> String {
    crate::to_keep(source, state)
}

/// L'état de départ, avec ce que la page avait gardé.
#[wasm_bindgen]
pub fn resume(source: &str, kept: &str) -> String {
    crate::resume(source, kept)
}

/// Les valeurs que la page retient le temps de la visite (ADR-113) : `prenom,personnes`.
#[wasm_bindgen]
pub fn visit_names(source: &str) -> String {
    crate::visit_names(source)
}

/// Ce que la page écrit dans sa mémoire de visite : une ligne par valeur, son nom et son JSON.
#[wasm_bindgen]
pub fn to_visit(source: &str, state: &str) -> String {
    crate::to_visit(source, state)
}

/// L'état, avec ce que la mémoire de visite rend, relu avec méfiance.
#[wasm_bindgen]
pub fn from_visit(source: &str, state: &str, stored: &str) -> String {
    crate::from_visit(source, state, stored)
}

/// Les conditions d'une page et leur réponse pour cet état (`count|is=0:1;…`).
#[wasm_bindgen]
pub fn conditions(source: &str, state: &str) -> String {
    crate::conditions(source, state)
}

/// L'arbitre : les valeurs d'une page après ce signal (`Add.tap`).
#[wasm_bindgen]
pub fn arbitrate(source: &str, state: &str, signal: &str) -> String {
    crate::arbitrate(source, state, signal)
}

/// Les valeurs que la page partage (ADR-079) : `seats;likes`.
#[wasm_bindgen]
pub fn shared_names(source: &str) -> String {
    crate::shared_names(source)
}

/// Les valeurs partagées d'un état : `seats=19;likes=3`.
#[wasm_bindgen]
pub fn shared_of(source: &str, state: &str) -> String {
    crate::shared_of(source, state)
}

/// L'état de la page avec les valeurs partagées reçues du serveur.
#[wasm_bindgen]
pub fn with_shared(source: &str, state: &str, shared: &str) -> String {
    crate::with_shared(source, state, shared)
}

/// Ce toucher change-t-il une valeur partagée ? Le serveur l'arbitre alors.
#[wasm_bindgen]
pub fn touches_shared(source: &str, signal: &str) -> bool {
    crate::touches_shared(source, signal)
}

/// Le fichier `.holo` d'un seul point de la page, pour ouvrir sa vue en profondeur.
#[cfg(feature = "drawing")]
#[wasm_bindgen]
pub fn point_source(source: &str, name: &str) -> Option<String> {
    crate::point_source(source, name)
}

/// L'éditeur : `ok`, ou la première faute avec sa ligne et sa colonne (ADR-046).
#[wasm_bindgen]
pub fn check_text(source: &str) -> String {
    crate::check_text(source)
}

/// L'éditeur : tous les mots du langage, en JSON.
#[wasm_bindgen]
pub fn vocabulary() -> String {
    crate::vocabulary()
}

/// Vérifie un fichier `.holo` sans rien lancer. Rend `ok` ou le message d'erreur.
#[cfg(feature = "drawing")]
#[wasm_bindgen]
pub fn check_holo(source: &str) -> String {
    match crate::check(source) {
        Ok(d) => format!("ok : Point « {} », seed {}, {} fragments", d.name, d.seed, d.shatter),
        Err(e) => e.to_string(),
    }
}

/// Export local de valeurs explicitement déclarées.
#[wasm_bindgen]
pub fn capability_export(source: &str, state: &str, name: &str) -> Result<String, JsValue> {
    crate::capability_export(source, state, name).map_err(|e| JsValue::from_str(&e))
}
/// Import vérifié avant mutation.
#[wasm_bindgen]
pub fn capability_received(source: &str, state: &str, name: &str, json: &str) -> Result<String, JsValue> {
    crate::capability_received(source, state, name, json).map_err(|e| JsValue::from_str(&e))
}
