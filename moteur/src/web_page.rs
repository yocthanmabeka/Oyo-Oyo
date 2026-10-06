//! La partie du moteur qui parle au navigateur pour la page à plat : lire le fichier, fabriquer
//! la page, arbitrer les valeurs (ADR-053). Elle ne dessine rien : le moteur léger
//! (`--no-default-features`) n'a qu'elle, et pèse bien moins que le moteur entier, qui y ajoute
//! le dessin (`web.rs`, `rendu.rs`).

use wasm_bindgen::prelude::*;

/// Les réglages de vue écrits dans un fichier `.holo` ; sans fichier, les réglages par défaut.
pub(crate) fn reglages_de(source: Option<String>) -> Result<crate::vue::Reglages, JsValue> {
    match source {
        Some(source) => crate::verifier_page(&source).and_then(|p| crate::vue::reglages(&p)).map_err(|e| JsValue::from_str(&e.to_string())),
        None => Ok(crate::vue::Reglages::default()),
    }
}

/// Cette page a-t-elle besoin du dessin (des points, un monde, la vue points, le relief) ? Sinon,
/// le moteur léger suffit, et le dessin n'est jamais téléchargé (ADR-053).
#[wasm_bindgen]
pub fn a_besoin_du_dessin(source: &str) -> bool {
    crate::a_besoin_du_dessin(source)
}

/// Le moteur léger ne dessine rien : la page d'entrée lui demande quand même de se mettre en
/// pause, ou combien d'images il a dessinées. Le moteur entier remplace ces trois-là.
#[cfg(not(feature = "dessin"))]
#[wasm_bindgen]
pub fn pause(_active: bool) {}

#[cfg(not(feature = "dessin"))]
#[wasm_bindgen]
pub fn reveiller() {}

#[cfg(not(feature = "dessin"))]
#[wasm_bindgen]
pub fn images_dessinees() -> u32 {
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
pub fn reglages_de_vue(source: &str) -> Result<Vec<f64>, JsValue> {
    let r = reglages_de(Some(source.to_string()))?;
    let disposition = match r.portails_disposition {
        crate::vue::Disposition::Grille => 0.0,
        crate::vue::Disposition::Ligne => 1.0,
        crate::vue::Disposition::Colonne => 2.0,
        crate::vue::Disposition::Diagonale => 3.0,
    };
    Ok(vec![r.densite, f64::from(u8::from(r.reduire)), r.apres, f64::from(u8::from(r.zoom_actif)), disposition, f64::from(r.portails_nombre), r.portails_taille, r.portails_lumiere, r.zoom_vitesse, r.portails_duree, r.angle_max.to_degrees(), f64::from(u8::from(r.points_actifs))])
}

/// Les mondes voisins d'un site, calculés à partir d'une graine, pour remplir le carrefour.
/// Chacun s'écrit « graine:rouge,vert,bleu » ; ils sont séparés par des points-virgules. La
/// graine reste un texte : un nombre de 64 bits ne tient pas dans un nombre de JavaScript.
#[wasm_bindgen]
pub fn mondes_voisins(source: &str, chemin: &str, nombre: u32) -> String {
    let texte = |(graine, c): (u64, [f32; 3])| format!("{graine}:{},{},{}", (c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8);
    crate::mondes_voisins(source, chemin, nombre).into_iter().map(texte).collect::<Vec<_>>().join(";")
}

/// Le monde où la page est posée quand on la regarde en personnage.
#[cfg(feature = "dessin")]
#[wasm_bindgen]
pub fn monde_d_accueil(source: &str) -> Option<String> {
    crate::monde_d_accueil(source)
}

/// La vue à plat : la page web ordinaire d'un fichier `.holo`, fabriquée par le moteur.
#[wasm_bindgen]
pub fn vue_a_plat(source: &str, base: &str, chemin: Option<String>) -> Result<String, JsValue> {
    crate::vue_a_plat_de(source, base, chemin.as_deref().unwrap_or("")).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Les effets demandés par un signal (`Open.tap`), séparés par des virgules.
#[wasm_bindgen]
pub fn effets(source: &str, signal: &str) -> String {
    crate::effets(source, signal).join(",")
}

/// Les valeurs d'une page à leur départ (`cart=0;likes=3`).
#[wasm_bindgen]
pub fn etat_initial(source: &str) -> String {
    crate::etat_initial(source)
}

/// Les horloges d'une page : `1000:time;2000:starX`.
#[wasm_bindgen]
pub fn horloges(source: &str) -> String {
    crate::horloges(source)
}

/// Les attentes d'une page et si chacune court : `3000:1;5000:0`.
#[wasm_bindgen]
pub fn delais(source: &str, etat: &str) -> String {
    crate::delais(source, etat)
}

/// Ce qu'il faut pour faire tourner un module : `somme.wasm|10|100|16`.
#[wasm_bindgen]
pub fn module_info(source: &str, etat: &str, nom: &str) -> String {
    crate::module_info(source, etat, nom)
}

/// Le module a rendu son nombre : le nouvel état.
#[wasm_bindgen]
pub fn module_fini(source: &str, etat: &str, nom: &str, valeur: f64) -> String {
    crate::module_fini(source, etat, nom, valeur.max(0.0) as u64)
}

/// Les lignes d'une liste pour cet état.
#[wasm_bindgen]
pub fn liste_html(source: &str, base: &str, etat: &str, nom: &str) -> String {
    crate::liste_html(source, base, etat, nom)
}

/// Ce qu'un formulaire envoie au serveur, en JSON.
#[wasm_bindgen]
pub fn envoi(source: &str, etat: &str, formulaire: &str) -> String {
    crate::envoi(source, etat, formulaire)
}

/// Une valeur écrite avec son format, dans la langue de la page.
#[wasm_bindgen]
pub fn formater(nom: &str, valeur: f64, format: &str, langue: &str) -> String {
    crate::formater(nom, valeur.max(0.0) as u64, format, langue)
}

/// La page lit-elle l'heure du visiteur ?
#[wasm_bindgen]
pub fn lit_l_heure(source: &str) -> bool {
    crate::lit_l_heure(source)
}

/// L'heure de l'appareil du visiteur, donnée au moteur.
#[wasm_bindgen]
pub fn regler_maintenant(annee: u32, mois: u32, jour: u32, semaine: u32, heure: u32, minute: u32) {
    crate::regler_maintenant([annee, mois, jour, semaine, heure, minute].map(u64::from));
}

/// Une minute a passé : le nouvel état.
#[wasm_bindgen]
pub fn avancer_l_horloge(source: &str, etat: &str) -> String {
    crate::avancer_l_horloge(source, etat)
}

/// Les fichiers qu'une page importe.
#[wasm_bindgen]
pub fn imports(source: &str) -> String {
    crate::imports(source)
}

/// Les touches du clavier que la page écoute.
#[wasm_bindgen]
pub fn touches(source: &str) -> String {
    crate::touches(source)
}

/// Les valeurs qu'un signal fait changer : leurs horloges repartent de zéro.
#[wasm_bindgen]
pub fn touchees(source: &str, signal: &str) -> String {
    crate::touchees(source, signal)
}

/// Le visiteur a écrit dans un champ ou coché une case.
#[wasm_bindgen]
pub fn saisir(source: &str, etat: &str, nom: &str, ecrit: &str) -> String {
    crate::saisir(source, etat, nom, ecrit)
}

/// D'où viennent les données de la page, et à quel rythme.
#[wasm_bindgen]
pub fn donnees(source: &str) -> String {
    crate::donnees(source)
}

/// Les données viennent d'arriver du serveur.
#[wasm_bindgen]
pub fn recevoir(source: &str, etat: &str, json: &str) -> String {
    crate::recevoir(source, etat, json)
}

/// Le visiteur fait glisser un bloc d'un plateau.
#[wasm_bindgen]
pub fn glisser(source: &str, etat: &str, nom: &str, x: u32, y: u32) -> String {
    crate::glisser(source, etat, nom, x, y)
}

/// Ce que la page garde d'une visite à l'autre.
#[wasm_bindgen]
pub fn a_garder(source: &str, etat: &str) -> String {
    crate::a_garder(source, etat)
}

/// L'état de départ, avec ce que la page avait gardé.
#[wasm_bindgen]
pub fn reprendre(source: &str, garde: &str) -> String {
    crate::reprendre(source, garde)
}

/// Les conditions d'une page et leur réponse pour cet état (`count|is=0:1;…`).
#[wasm_bindgen]
pub fn conditions(source: &str, etat: &str) -> String {
    crate::conditions(source, etat)
}

/// L'arbitre : les valeurs d'une page après ce signal (`Add.tap`).
#[wasm_bindgen]
pub fn arbitrer(source: &str, etat: &str, signal: &str) -> String {
    crate::arbitrer(source, etat, signal)
}

/// Le fichier `.holo` d'un seul point de la page, pour ouvrir sa vue en profondeur.
#[cfg(feature = "dessin")]
#[wasm_bindgen]
pub fn source_du_point(source: &str, nom: &str) -> Option<String> {
    crate::source_du_point(source, nom)
}

/// L'éditeur : `ok`, ou la première faute avec sa ligne et sa colonne (ADR-046).
#[wasm_bindgen]
pub fn verifier_texte(source: &str) -> String {
    crate::verifier_texte(source)
}

/// L'éditeur : tous les mots du langage, en JSON.
#[wasm_bindgen]
pub fn vocabulaire() -> String {
    crate::vocabulaire()
}

/// Vérifie un fichier `.holo` sans rien lancer. Rend `ok` ou le message d'erreur.
#[cfg(feature = "dessin")]
#[wasm_bindgen]
pub fn verifier_holo(source: &str) -> String {
    match crate::verifier(source) {
        Ok(d) => format!("ok : Point « {} », seed {}, {} fragments", d.nom, d.graine, d.morceler),
        Err(e) => e.to_string(),
    }
}
