//! Branchement sur le navigateur : la zone de dessin, les doigts (glisser, pincer), la
//! molette, la boucle d'affichage et les mesures publiées dans `window.__holo`.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, PointerEvent, WheelEvent};

use crate::navigation::Navigation;
use crate::rendu::Rendu;

/// Au-delà, les pixels coûtent cher sur un téléphone pour un gain invisible.
const DENSITE_MAX: f64 = 2.0;

struct Etat {
    nav: Navigation,
    rendu: Rendu,
    canvas: HtmlCanvasElement,
    pointeurs: HashMap<i32, (f32, f32)>,
    /// Où chaque doigt s'est posé : un doigt qui se relève sans avoir bougé est un toucher.
    departs: HashMap<i32, (f32, f32)>,
    dernier_t: f64,
    premiere_image_ms: Option<f64>,
    images: u32,
    pire_ms: f64,
    dernier_rapport: f64,
    nb_points: usize,
}

/// Point d'entrée appelé par la page : lit le fichier `.holo`, prépare la carte graphique
/// et lance la boucle d'affichage.
#[wasm_bindgen]
pub async fn demarrer(canvas: HtmlCanvasElement, source: &str, zoom_initial: f32) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let decl = crate::verifier(source).map_err(|e| JsValue::from_str(&format!("fichier .holo refusé : {e}")))?;
    ajuster_taille(&canvas);
    let (rendu, canvas) = Rendu::nouveau(canvas).await?;
    let mut nav = Navigation::new(decl);
    // Pour les tests et les captures d'écran : partir d'un zoom donné, par petits pas pour
    // franchir les seuils d'entrée exactement comme le ferait un pincement.
    let mut restant = zoom_initial.max(0.0);
    while restant > 0.0 {
        let pas = restant.min(0.25);
        nav.zoomer(pas);
        restant -= pas;
    }
    nav.avancer_temps(10.0);
    let etat = Rc::new(RefCell::new(Etat {
        nav,
        rendu,
        canvas: canvas.clone(),
        pointeurs: HashMap::new(),
        departs: HashMap::new(),
        dernier_t: 0.0,
        premiere_image_ms: None,
        images: 0,
        pire_ms: 0.0,
        dernier_rapport: 0.0,
        nb_points: 0,
    }));
    brancher(&canvas, &etat)?;
    boucle(etat)
}

/// La vue à plat : la page web ordinaire d'un fichier `.holo`, fabriquée par le moteur.
#[wasm_bindgen]
pub fn vue_a_plat(source: &str, base: &str) -> Result<String, JsValue> {
    crate::vue_a_plat(source, base).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Les effets demandés par un signal (`Open.tap`), séparés par des virgules.
#[wasm_bindgen]
pub fn effets(source: &str, signal: &str) -> String {
    crate::effets(source, signal).join(",")
}

/// Le fichier `.holo` d'un seul point de la page, pour ouvrir sa vue en profondeur.
#[wasm_bindgen]
pub fn source_du_point(source: &str, nom: &str) -> Option<String> {
    crate::source_du_point(source, nom)
}

/// Vérifie un fichier `.holo` sans rien lancer. Rend `ok` ou le message d'erreur.
#[wasm_bindgen]
pub fn verifier_holo(source: &str) -> String {
    match crate::verifier(source) {
        Ok(d) => format!("ok : Point « {} », seed {}, {} fragments", d.nom, d.graine, d.morceler),
        Err(e) => e.to_string(),
    }
}

fn ajuster_taille(canvas: &HtmlCanvasElement) -> bool {
    let densite = web_sys::window().map(|w| w.device_pixel_ratio()).unwrap_or(1.0).min(DENSITE_MAX);
    let largeur = (f64::from(canvas.client_width()) * densite).round().max(1.0) as u32;
    let hauteur = (f64::from(canvas.client_height()) * densite).round().max(1.0) as u32;
    if canvas.width() != largeur || canvas.height() != hauteur {
        canvas.set_width(largeur);
        canvas.set_height(hauteur);
        return true;
    }
    false
}

fn brancher(canvas: &HtmlCanvasElement, etat: &Rc<RefCell<Etat>>) -> Result<(), JsValue> {
    let ecouter = |nom: &str, f: Box<dyn FnMut(PointerEvent)>| -> Result<(), JsValue> {
        let c = Closure::wrap(f);
        canvas.add_event_listener_with_callback(nom, c.as_ref().unchecked_ref())?;
        c.forget();
        Ok(())
    };

    let e = etat.clone();
    ecouter(
        "pointerdown",
        Box::new(move |ev: PointerEvent| {
            ev.prevent_default();
            let mut etat = e.borrow_mut();
            let pos = (ev.client_x() as f32, ev.client_y() as f32);
            etat.pointeurs.insert(ev.pointer_id(), pos);
            etat.departs.insert(ev.pointer_id(), pos);
        }),
    )?;

    let e = etat.clone();
    ecouter(
        "pointermove",
        Box::new(move |ev: PointerEvent| {
            let mut etat = e.borrow_mut();
            let id = ev.pointer_id();
            if !etat.pointeurs.contains_key(&id) {
                return;
            }
            let nouveau = (ev.client_x() as f32, ev.client_y() as f32);
            let ancien = etat.pointeurs[&id];
            let hauteur = f64::from(etat.canvas.client_height()).max(1.0) as f32;
            if etat.pointeurs.len() >= 2 {
                // Pincer : le rapport des écartements donne le zoom.
                let autre = *etat.pointeurs.iter().find(|(k, _)| **k != id).map(|(_, v)| v).unwrap();
                let d_avant = ((ancien.0 - autre.0).powi(2) + (ancien.1 - autre.1).powi(2)).sqrt().max(1.0);
                let d_apres = ((nouveau.0 - autre.0).powi(2) + (nouveau.1 - autre.1).powi(2)).sqrt().max(1.0);
                etat.nav.zoomer((d_apres / d_avant).log2() * 1.1);
            } else {
                // Glisser : on tourne le monde.
                let dx = (nouveau.0 - ancien.0) / hauteur;
                let dy = (nouveau.1 - ancien.1) / hauteur;
                etat.nav.tourner(-dx * 2.4, -dy * 2.4);
            }
            etat.pointeurs.insert(id, nouveau);
        }),
    )?;

    for nom in ["pointerup", "pointercancel", "pointerleave"] {
        let e = etat.clone();
        ecouter(
            nom,
            Box::new(move |ev: PointerEvent| {
                let mut etat = e.borrow_mut();
                let id = ev.pointer_id();
                let depart = etat.departs.remove(&id);
                let seul = etat.pointeurs.len() == 1;
                etat.pointeurs.remove(&id);
                // Un toucher : un seul doigt, relevé à moins de 10 pixels de là où il s'est posé.
                if let Some((x0, y0)) = depart {
                    let (x, y) = (ev.client_x() as f32, ev.client_y() as f32);
                    if seul && nom == "pointerup" && (x - x0).hypot(y - y0) < 10.0 {
                        let largeur = f64::from(etat.canvas.client_width()).max(1.0) as f32;
                        let hauteur = f64::from(etat.canvas.client_height()).max(1.0) as f32;
                        let aspect = etat.rendu.aspect();
                        let vx = (x / largeur * 2.0 - 1.0) * aspect;
                        let vy = -(y / hauteur * 2.0 - 1.0);
                        etat.nav.viser_ecran(vx, vy, aspect);
                    }
                }
            }),
        )?;
    }

    let e = etat.clone();
    let molette = Closure::<dyn FnMut(WheelEvent)>::new(move |ev: WheelEvent| {
        ev.prevent_default();
        e.borrow_mut().nav.zoomer(-(ev.delta_y() as f32) * 0.0018);
    });
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(false);
    canvas.add_event_listener_with_callback_and_add_event_listener_options("wheel", molette.as_ref().unchecked_ref(), &options)?;
    molette.forget();
    Ok(())
}

thread_local! {
    /// La boucle d'affichage, gardée ici pour pouvoir la relancer après une pause.
    static BOUCLE: RefCell<Option<Closure<dyn FnMut(f64)>>> = const { RefCell::new(None) };
    static EN_PAUSE: Cell<bool> = const { Cell::new(false) };
    static ETAT: RefCell<Option<Rc<RefCell<Etat>>>> = const { RefCell::new(None) };
}

/// Met le monde en pause, ou le relance. En pause, plus rien n'est calculé ni dessiné :
/// le processeur, la carte graphique et la batterie se reposent.
#[wasm_bindgen]
pub fn pause(active: bool) -> Result<(), JsValue> {
    let avant = EN_PAUSE.with(|p| p.replace(active));
    if avant && !active {
        // À la reprise, on repart du temps présent : pas de saut d'animation.
        ETAT.with(|e| {
            if let Some(etat) = e.borrow().as_ref() {
                let mut etat = etat.borrow_mut();
                etat.dernier_t = 0.0;
                etat.dernier_rapport = 0.0;
                etat.images = 0;
            }
        });
        BOUCLE.with(|b| match b.borrow().as_ref() {
            Some(c) => demander_image(c),
            None => Ok(()),
        })?;
    }
    Ok(())
}

fn boucle(etat: Rc<RefCell<Etat>>) -> Result<(), JsValue> {
    ETAT.with(|e| *e.borrow_mut() = Some(etat.clone()));
    let fermeture = Closure::new(move |t: f64| {
        if EN_PAUSE.with(|p| p.get()) {
            return;
        }
        if let Err(e) = image(&etat, t) {
            web_sys::console::error_1(&e);
            return;
        }
        let suite = BOUCLE.with(|b| match b.borrow().as_ref() {
            Some(c) => demander_image(c),
            None => Ok(()),
        });
        if let Err(e) = suite {
            web_sys::console::error_1(&e);
        }
    });
    BOUCLE.with(|b| *b.borrow_mut() = Some(fermeture));
    BOUCLE.with(|b| match b.borrow().as_ref() {
        Some(c) => demander_image(c),
        None => Ok(()),
    })
}

fn demander_image(c: &Closure<dyn FnMut(f64)>) -> Result<(), JsValue> {
    web_sys::window().ok_or("pas de fenêtre")?.request_animation_frame(c.as_ref().unchecked_ref())?;
    Ok(())
}

fn image(etat: &Rc<RefCell<Etat>>, t: f64) -> Result<(), JsValue> {
    let mut e = etat.borrow_mut();
    let dt = if e.dernier_t > 0.0 { ((t - e.dernier_t) / 1000.0).min(0.1) as f32 } else { 0.0 };
    let duree_image = if e.dernier_t > 0.0 { t - e.dernier_t } else { 0.0 };
    e.dernier_t = t;

    if ajuster_taille(&e.canvas) {
        let (l, h) = (e.canvas.width(), e.canvas.height());
        e.rendu.redimensionner(l, h);
    }
    e.nav.avancer_temps(dt);
    let aspect = e.rendu.aspect();
    let sprites = e.nav.sprites(aspect);
    e.nb_points = e.rendu.dessiner(&sprites, (t / 1000.0) as f32)?;

    if e.premiere_image_ms.is_none() {
        e.premiere_image_ms = Some(t);
    }
    e.images += 1;
    e.pire_ms = e.pire_ms.max(duree_image);
    if t - e.dernier_rapport >= 500.0 {
        let secondes = ((t - e.dernier_rapport) / 1000.0).max(0.001);
        let ips = f64::from(e.images) / secondes;
        publier(&e, ips)?;
        e.images = 0;
        e.pire_ms = 0.0;
        e.dernier_rapport = t;
    }
    Ok(())
}

/// Écrit les mesures dans `window.__holo`, que la page affiche et copie.
fn publier(e: &Etat, ips: f64) -> Result<(), JsValue> {
    let o = js_sys::Object::new();
    let poser = |cle: &str, v: JsValue| js_sys::Reflect::set(&o, &JsValue::from_str(cle), &v).map(|_| ());
    poser("backend", JsValue::from_str(e.rendu.backend))?;
    poser("ips", JsValue::from_f64((ips * 10.0).round() / 10.0))?;
    poser("pire_ms", JsValue::from_f64((e.pire_ms * 10.0).round() / 10.0))?;
    poser("profondeur", JsValue::from_f64(e.nav.profondeur() as f64))?;
    poser("chemin", JsValue::from_str(&e.nav.chemin()))?;
    poser("graine", JsValue::from_str(&e.nav.graine_courante().to_string()))?;
    poser("points_dessines", JsValue::from_f64(e.nb_points as f64))?;
    poser("zoom", JsValue::from_f64((f64::from(e.nav.zoom) * 100.0).round() / 100.0))?;
    poser("largeur", JsValue::from_f64(f64::from(e.canvas.width())))?;
    poser("hauteur", JsValue::from_f64(f64::from(e.canvas.height())))?;
    poser("premiere_image_ms", JsValue::from_f64(e.premiere_image_ms.unwrap_or(0.0).round()))?;
    let fenetre = web_sys::window().ok_or("pas de fenêtre")?;
    js_sys::Reflect::set(&fenetre, &JsValue::from_str("__holo"), &o)?;
    Ok(())
}
