//! Branchement sur le navigateur : la zone de dessin, les doigts (glisser, pincer), la
//! molette, la boucle d'affichage et les mesures publiées dans `window.__holo`.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, PointerEvent, WheelEvent};

use crate::mosaic::Mosaic;
use crate::navigation::Navigation;
use crate::renderer::Renderer;

/// Au-delà, les pixels coûtent cher sur un téléphone pour un gain invisible.
const DENSITY_MAX: f64 = 2.0;

struct State {
    nav: Navigation,
    /// Quand elle est là, c'est elle qu'on affiche et qu'on manipule, à la place du monde.
    mosaic: Option<Mosaic>,
    /// La vue de la mosaïque à la dernière image, et depuis combien d'images elle n'a pas
    /// bougé : une vue immobile n'est pas redessinée.
    last_view: Option<[f64; 7]>,
    still_ones: u8,
    renderer: Renderer,
    canvas: HtmlCanvasElement,
    pointers: HashMap<i32, (f32, f32)>,
    /// Où chaque doigt s'est posé : un doigt qui se relève sans avoir bougé est un toucher.
    starts: HashMap<i32, (f32, f32)>,
    last_t: f64,
    first_frame_ms: Option<f64>,
    images: u32,
    worst_ms: f64,
    last_report: f64,
    nb_points: usize,
}

/// Point d'entrée appelé par la page : lit le fichier `.holo`, prépare la carte graphique
/// et lance la boucle d'affichage.
#[wasm_bindgen]
pub async fn start_engine(canvas: HtmlCanvasElement, source: &str, zoom_initial: f32) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let decl = crate::check(source).map_err(|e| JsValue::from_str(&format!("fichier .holo refusé : {e}")))?;
    launch(canvas, navigation(decl, zoom_initial), None).await
}

/// Affiche une image comme une mosaïque de points, un point par pixel (voir `mosaique.rs`).
/// `couleurs` contient quatre octets par pixel : rouge, vert, bleu, opacité.
#[wasm_bindgen]
pub async fn start_mosaic(canvas: HtmlCanvasElement, colors: Vec<u8>, width: u32, height: u32, source: Option<String>, already_enlarged: Option<f64>) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let (view_w, view_h) = view_size(&canvas);
    let mosaic = Mosaic::new(width, height, colors, 1, view_w, view_h, remaining_settings(source, already_enlarged)?).ok_or_else(|| JsValue::from_str("image mal décrite : il faut largeur × hauteur × 4 octets"))?;
    // Le monde n'est pas affiché tant que la mosaïque est là ; il faut pourtant un point.
    let decl = crate::universe::PointDecl { name: "Mosaic".into(), seed: 1, light: 0.0, shatter: 1, color: None, palette: Vec::new() };
    launch(canvas, Navigation::new(decl), Some(mosaic)).await
}

/// Où est la mosaïque : centre (x, y, en pixels de l'image), pixels d'écran par pixel
/// d'image, opacité des points (0 : on voit l'image, 1 : on voit les points), niveau de
/// morcellement, nombre de points dessinés, lacet, tangage, distance de l'œil en pixels,
/// échelle où l'on voit la page entière.
/// Vide s'il n'y a pas de mosaïque.
#[wasm_bindgen]
pub fn mosaic_camera() -> Vec<f64> {
    STATE.with(|e| {
        e.borrow()
            .as_ref()
            .and_then(|state| {
                let state = state.borrow();
                let (_, view_h) = view_size(&state.canvas);
                state.mosaic.as_ref().map(|m| {
                    vec![m.cx, m.cy, m.scale, m.points_opacity(), f64::from(m.level()), state.nb_points as f64, m.yaw, m.pitch, crate::mosaic::distance(view_h), m.rest_scale]
                })
            })
            .unwrap_or_default()
    })
}


/// Les réglages de vue, une fois décompté le grossissement déjà fait par le zoom ordinaire :
/// `Zoom(max:)` borne le zoom entier, pas seulement la vue points.
fn remaining_settings(source: Option<String>, already_enlarged: Option<f64>) -> Result<crate::view::Settings, JsValue> {
    let mut r = crate::web_page::settings_of(source)?;
    r.zoom_max = (r.zoom_max / already_enlarged.unwrap_or(1.0).max(1.0)).max(1.0);
    Ok(r)
}


fn with_mosaic(f: impl FnOnce(&mut Mosaic)) {
    STATE.with(|e| {
        if let Some(state) = e.borrow().as_ref() {
            if let Some(m) = state.borrow_mut().mosaic.as_mut() {
                f(m);
            }
        }
    });
    let _ = wake();
}

/// Affiche une mosaïque alors que le moteur tourne déjà : la page devient des points.
#[wasm_bindgen]
pub fn place_mosaic(colors: Vec<u8>, width: u32, height: u32, source: Option<String>, already_enlarged: Option<f64>) -> Result<(), JsValue> {
    let settings = remaining_settings(source, already_enlarged)?;
    STATE.with(|e| match e.borrow().as_ref() {
        Some(state) => {
            let mut state = state.borrow_mut();
            let (view_w, view_h) = view_size(&state.canvas);
            let mosaic = Mosaic::new(width, height, colors, 1, view_w, view_h, settings);
            state.mosaic = Some(mosaic.ok_or_else(|| JsValue::from_str("image mal décrite : il faut largeur × hauteur × 4 octets"))?);
            drop(state);
            wake()
        }
        None => Err(JsValue::from_str("le moteur n'est pas encore démarré")),
    })
}

/// Retire la mosaïque : le moteur affiche de nouveau son monde.
#[wasm_bindgen]
pub fn remove_mosaic() {
    STATE.with(|e| {
        if let Some(state) = e.borrow().as_ref() {
            state.borrow_mut().mosaic = None;
        }
    });
    let _ = wake();
}

/// Quel endroit de la page se trouve sous ce point de l'écran : x et y, en pixels de l'image.
/// Vide s'il n'y a pas de mosaïque, ou si l'on vise à côté de la page.
#[wasm_bindgen]
pub fn mosaic_under(x: f64, y: f64) -> Vec<f64> {
    STATE.with(|e| {
        e.borrow()
            .as_ref()
            .and_then(|state| {
                let state = state.borrow();
                let (view_w, view_h) = view_size(&state.canvas);
                let m = state.mosaic.as_ref()?;
                let [ux, uy] = m.on_page(x - view_w / 2.0, y - view_h / 2.0, crate::mosaic::distance(view_h))?;
                Some(vec![m.cx + ux / m.scale, m.cy + uy / m.scale])
            })
            .unwrap_or_default()
    })
}

/// Choisit ce que fait un glissement sur la mosaïque : tourner la page, ou la déplacer.
#[wasm_bindgen]
pub fn mosaic_turn(active: bool) {
    with_mosaic(|m| m.rotate = active);
}

/// Fait tourner la page de la mosaïque (en radians) : on la voit de biais.
#[wasm_bindgen]
pub fn mosaic_pivot(yaw: f64, pitch: f64) {
    with_mosaic(|m| m.pivot(yaw, pitch));
}

/// Remet la page de la mosaïque de face.
#[wasm_bindgen]
pub fn mosaic_front() {
    with_mosaic(Mosaic::front_facing);
}

/// La taille de la zone de dessin, en pixels de la page.
fn view_size(canvas: &HtmlCanvasElement) -> (f64, f64) {
    (f64::from(canvas.client_width()).max(1.0), f64::from(canvas.client_height()).max(1.0))
}

async fn launch(canvas: HtmlCanvasElement, nav: Navigation, mosaic: Option<Mosaic>) -> Result<(), JsValue> {
    adjust_size(&canvas);
    let (renderer, canvas) = Renderer::new_one(canvas).await?;
    let state = Rc::new(RefCell::new(State {
        nav,
        mosaic,
        last_view: None,
        still_ones: 0,
        renderer,
        canvas: canvas.clone(),
        pointers: HashMap::new(),
        starts: HashMap::new(),
        last_t: 0.0,
        first_frame_ms: None,
        images: 0,
        worst_ms: 0.0,
        last_report: 0.0,
        nb_points: 0,
    }));
    wire(&canvas, &state)?;
    cycle(state)
}

/// Prépare la visite d'un point, en partant d'un zoom donné : par petits pas, pour franchir
/// les seuils d'entrée exactement comme le ferait un pincement.
fn navigation(decl: crate::universe::PointDecl, zoom_initial: f32) -> Navigation {
    let mut nav = Navigation::new(decl);
    let mut remaining = zoom_initial.max(0.0);
    while remaining > 0.0 {
        let step = remaining.min(0.25);
        nav.zoom_in(step);
        remaining -= step;
    }
    nav.advance_time(10.0);
    nav
}

/// Remplace le monde affiché par celui d'un autre point, sans relancer la carte graphique.
#[wasm_bindgen]
pub fn change_world(source: &str, zoom_initial: f32) -> Result<(), JsValue> {
    let decl = crate::check(source).map_err(|e| JsValue::from_str(&format!("fichier .holo refusé : {e}")))?;
    STATE.with(|e| match e.borrow().as_ref() {
        Some(state) => {
            state.borrow_mut().nav = navigation(decl, zoom_initial);
            wake()
        }
        None => Err(JsValue::from_str("le moteur n'est pas encore démarré")),
    })
}

/// Où poser la page dans le monde, pour la vue personnage : voir `Navigation::surface`.
/// Vide tant que le moteur n'est pas démarré.
#[wasm_bindgen]
pub fn surface() -> Vec<f32> {
    STATE.with(|e| e.borrow().as_ref().map(|state| state.borrow().nav.surface().to_vec()).unwrap_or_default())
}


fn adjust_size(canvas: &HtmlCanvasElement) -> bool {
    let density = web_sys::window().map(|w| w.device_pixel_ratio()).unwrap_or(1.0).min(DENSITY_MAX);
    let width = (f64::from(canvas.client_width()) * density).round().max(1.0) as u32;
    let height = (f64::from(canvas.client_height()) * density).round().max(1.0) as u32;
    if canvas.width() != width || canvas.height() != height {
        canvas.set_width(width);
        canvas.set_height(height);
        return true;
    }
    false
}

fn wire(canvas: &HtmlCanvasElement, state: &Rc<RefCell<State>>) -> Result<(), JsValue> {
    let listen = |name: &str, f: Box<dyn FnMut(PointerEvent)>| -> Result<(), JsValue> {
        let c = Closure::wrap(f);
        canvas.add_event_listener_with_callback(name, c.as_ref().unchecked_ref())?;
        c.forget();
        Ok(())
    };

    let e = state.clone();
    listen(
        "pointerdown",
        Box::new(move |ev: PointerEvent| {
            ev.prevent_default();
            let mut state = e.borrow_mut();
            let pos = (ev.client_x() as f32, ev.client_y() as f32);
            state.pointers.insert(ev.pointer_id(), pos);
            state.starts.insert(ev.pointer_id(), pos);
        }),
    )?;

    let e = state.clone();
    listen(
        "pointermove",
        Box::new(move |ev: PointerEvent| {
            let mut state = e.borrow_mut();
            let id = ev.pointer_id();
            if !state.pointers.contains_key(&id) {
                return;
            }
            let new_one = (ev.client_x() as f32, ev.client_y() as f32);
            let old = state.pointers[&id];
            let height = f64::from(state.canvas.client_height()).max(1.0) as f32;
            if state.pointers.len() >= 2 {
                // Pincer : le rapport des écartements donne le zoom.
                let other = *state.pointers.iter().find(|(k, _)| **k != id).map(|(_, v)| v).unwrap();
                let d_before = ((old.0 - other.0).powi(2) + (old.1 - other.1).powi(2)).sqrt().max(1.0);
                let d_after = ((new_one.0 - other.0).powi(2) + (new_one.1 - other.1).powi(2)).sqrt().max(1.0);
                let (view_w, view_h) = view_size(&state.canvas);
                let state = &mut *state;
                match state.mosaic.as_mut() {
                    // Le milieu des deux doigts reste sur le même point de l'image.
                    Some(m) => m.zoom_in(f64::from(d_after / d_before), f64::from(new_one.0 + other.0) / 2.0, f64::from(new_one.1 + other.1) / 2.0, view_w, view_h),
                    None => state.nav.zoom_in((d_after / d_before).log2() * 1.1),
                }
            } else {
                let state = &mut *state;
                match state.mosaic.as_mut() {
                    // Glisser avec le bouton droit, avec Maj, ou en mode « tourner » : la page
                    // tourne, on la voit de biais. Sinon, glisser la déplace.
                    Some(m) if m.rotate || ev.shift_key() || ev.buttons() & 2 != 0 => {
                        m.pivot(f64::from(new_one.0 - old.0) * 0.005, -f64::from(new_one.1 - old.1) * 0.005)
                    }
                    Some(m) => m.translate(f64::from(new_one.0 - old.0), f64::from(new_one.1 - old.1)),
                    // Glisser : on tourne le monde.
                    None => {
                        let dx = (new_one.0 - old.0) / height;
                        let dy = (new_one.1 - old.1) / height;
                        state.nav.rotate(-dx * 2.4, -dy * 2.4);
                    }
                }
            }
            state.pointers.insert(id, new_one);
        }),
    )?;

    for name in ["pointerup", "pointercancel", "pointerleave"] {
        let e = state.clone();
        listen(
            name,
            Box::new(move |ev: PointerEvent| {
                let mut state = e.borrow_mut();
                let id = ev.pointer_id();
                let start_value = state.starts.remove(&id);
                let single = state.pointers.len() == 1;
                state.pointers.remove(&id);
                // Un toucher : un seul doigt, relevé à moins de 10 pixels de là où il s'est posé.
                if let Some((x0, y0)) = start_value {
                    let (x, y) = (ev.client_x() as f32, ev.client_y() as f32);
                    if state.mosaic.is_none() && single && name == "pointerup" && (x - x0).hypot(y - y0) < 10.0 {
                        let width = f64::from(state.canvas.client_width()).max(1.0) as f32;
                        let height = f64::from(state.canvas.client_height()).max(1.0) as f32;
                        let aspect = state.renderer.aspect();
                        let vx = (x / width * 2.0 - 1.0) * aspect;
                        let vy = -(y / height * 2.0 - 1.0);
                        state.nav.aim_screen(vx, vy, aspect);
                    }
                }
            }),
        )?;
    }

    let e = state.clone();
    let wheel = Closure::<dyn FnMut(WheelEvent)>::new(move |ev: WheelEvent| {
        ev.prevent_default();
        let mut state = e.borrow_mut();
        let (view_w, view_h) = view_size(&state.canvas);
        let state = &mut *state;
        match state.mosaic.as_mut() {
            Some(m) => {
                // Un pincement arrive comme une molette avec Ctrl, par petits pas : on les grossit.
                let step = if ev.ctrl_key() && ev.delta_y().abs() < 50.0 { ev.delta_y() * 6.0 } else { ev.delta_y() };
                // Zoom(speed:) règle la molette de la main ; un zoom envoyé par la page (deux
                // doigts, approche d'un point) suit exactement ce qu'elle demande.
                let speed = if ev.is_trusted() { m.speed() } else { 1.0 };
                m.zoom_in(2f64.powf(-step * 0.003 * speed), f64::from(ev.client_x()), f64::from(ev.client_y()), view_w, view_h)
            }
            None => state.nav.zoom_in(-(ev.delta_y() as f32) * 0.0018),
        }
    });
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(false);
    canvas.add_event_listener_with_callback_and_add_event_listener_options("wheel", wheel.as_ref().unchecked_ref(), &options)?;
    wheel.forget();

    // Tout geste sur la zone de dessin relance la boucle si elle s'était arrêtée.
    for name in ["pointerdown", "pointermove", "pointerup", "wheel"] {
        let wakeup = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
            // Une souris qui passe sans bouton enfoncé ne change rien à la vue.
            if name != "pointermove" || ev.buttons() != 0 {
                let _ = wake();
            }
        });
        canvas.add_event_listener_with_callback(name, wakeup.as_ref().unchecked_ref())?;
        wakeup.forget();
    }
    Ok(())
}

thread_local! {
    /// La boucle d'affichage, gardée ici pour pouvoir la relancer après une pause.
    static LOOP: RefCell<Option<Closure<dyn FnMut(f64)>>> = const { RefCell::new(None) };
    static PAUSED: Cell<bool> = const { Cell::new(false) };
    /// Vrai quand la boucle s'est arrêtée d'elle-même parce que rien ne bougeait.
    static AT_REST: Cell<bool> = const { Cell::new(false) };
    /// Combien d'images ont été dessinées depuis le démarrage : sert à vérifier la sobriété.
    static IMAGES: Cell<u32> = const { Cell::new(0) };
    static STATE: RefCell<Option<Rc<RefCell<State>>>> = const { RefCell::new(None) };
}

/// Met le monde en pause, ou le relance. En pause, plus rien n'est calculé ni dessiné :
/// le processeur, la carte graphique et la batterie se reposent.
#[wasm_bindgen]
pub fn pause(active: bool) -> Result<(), JsValue> {
    let before = PAUSED.with(|p| p.replace(active));
    if before && !active {
        AT_REST.with(|r| r.set(false));
        // À la reprise, on repart du temps présent : pas de saut d'animation.
        STATE.with(|e| {
            if let Some(state) = e.borrow().as_ref() {
                let mut state = state.borrow_mut();
                state.last_t = 0.0;
                state.last_report = 0.0;
                state.images = 0;
            }
        });
        LOOP.with(|b| match b.borrow().as_ref() {
            Some(c) => request_frame(c),
            None => Ok(()),
        })?;
    }
    Ok(())
}

fn cycle(state: Rc<RefCell<State>>) -> Result<(), JsValue> {
    STATE.with(|e| *e.borrow_mut() = Some(state.clone()));
    let closing = Closure::new(move |t: f64| {
        if PAUSED.with(|p| p.get()) {
            return;
        }
        match image(&state, t) {
            Err(e) => {
                web_sys::console::error_1(&e);
                return;
            }
            // Rien n'a bougé : la boucle s'arrête, jusqu'au prochain geste. Plus rien n'est
            // calculé ni dessiné, la machine se repose.
            Ok(false) => {
                AT_REST.with(|r| r.set(true));
                return;
            }
            Ok(true) => {}
        }
        let suite = LOOP.with(|b| match b.borrow().as_ref() {
            Some(c) => request_frame(c),
            None => Ok(()),
        });
        if let Err(e) = suite {
            web_sys::console::error_1(&e);
        }
    });
    LOOP.with(|b| *b.borrow_mut() = Some(closing));
    LOOP.with(|b| match b.borrow().as_ref() {
        Some(c) => request_frame(c),
        None => Ok(()),
    })
}

/// Relance la boucle d'affichage si elle s'était arrêtée faute de mouvement.
#[wasm_bindgen]
pub fn wake() -> Result<(), JsValue> {
    if AT_REST.with(|r| r.replace(false)) && !PAUSED.with(|p| p.get()) {
        LOOP.with(|b| match b.borrow().as_ref() {
            Some(c) => request_frame(c),
            None => Ok(()),
        })?;
    }
    Ok(())
}

/// Combien d'images le moteur a dessinées depuis son démarrage.
#[wasm_bindgen]
pub fn frames_drawn() -> u32 {
    IMAGES.with(|i| i.get())
}

fn request_frame(c: &Closure<dyn FnMut(f64)>) -> Result<(), JsValue> {
    web_sys::window().ok_or("pas de fenêtre")?.request_animation_frame(c.as_ref().unchecked_ref())?;
    Ok(())
}

/// Dessine une image. Rend faux quand il n'y a plus rien à redessiner.
fn image(state: &Rc<RefCell<State>>, t: f64) -> Result<bool, JsValue> {
    let mut e = state.borrow_mut();
    let dt = if e.last_t > 0.0 { ((t - e.last_t) / 1000.0).min(0.1) as f32 } else { 0.0 };
    let frame_duration = if e.last_t > 0.0 { t - e.last_t } else { 0.0 };
    e.last_t = t;

    let resized = adjust_size(&e.canvas);
    if resized {
        let (l, h) = (e.canvas.width(), e.canvas.height());
        e.renderer.resize(l, h);
    }
    // Une mosaïque ne bouge que si on la touche : immobile depuis quelques images, on ne la
    // redessine plus. (Un monde, lui, vit : ses points pulsent.)
    let view = e.mosaic.as_ref().map(|m| {
        let (view_w, view_h) = view_size(&e.canvas);
        [m.cx, m.cy, m.scale, m.yaw, m.pitch, view_w, view_h]
    });
    if view.is_some() && view == e.last_view && !resized {
        e.still_ones = e.still_ones.saturating_add(1);
        if e.still_ones > 3 {
            return Ok(false);
        }
    } else {
        e.still_ones = 0;
    }
    e.last_view = view;
    IMAGES.with(|i| i.set(i.get().wrapping_add(1)));
    e.nav.advance_time(dt);
    let aspect = e.renderer.aspect();
    let sprites = match &e.mosaic {
        Some(m) => {
            let (view_w, view_h) = view_size(&e.canvas);
            m.sprites(view_w, view_h)
        }
        None => e.nav.sprites(aspect),
    };
    e.nb_points = e.renderer.draw(&sprites, (t / 1000.0) as f32)?;

    if e.first_frame_ms.is_none() {
        e.first_frame_ms = Some(t);
    }
    e.images += 1;
    e.worst_ms = e.worst_ms.max(frame_duration);
    if t - e.last_report >= 500.0 {
        let seconds = ((t - e.last_report) / 1000.0).max(0.001);
        let fps = f64::from(e.images) / seconds;
        publish(&e, fps)?;
        e.images = 0;
        e.worst_ms = 0.0;
        e.last_report = t;
    }
    Ok(true)
}

/// Écrit les mesures dans `window.__holo`, que la page affiche et copie.
fn publish(e: &State, fps: f64) -> Result<(), JsValue> {
    let o = js_sys::Object::new();
    let place = |key: &str, v: JsValue| js_sys::Reflect::set(&o, &JsValue::from_str(key), &v).map(|_| ());
    place("backend", JsValue::from_str(e.renderer.backend))?;
    place("ips", JsValue::from_f64((fps * 10.0).round() / 10.0))?;
    place("pire_ms", JsValue::from_f64((e.worst_ms * 10.0).round() / 10.0))?;
    place("profondeur", JsValue::from_f64(e.nav.depth() as f64))?;
    place("chemin", JsValue::from_str(&e.nav.path()))?;
    place("graine", JsValue::from_str(&e.nav.current_seed().to_string()))?;
    place("points_dessines", JsValue::from_f64(e.nb_points as f64))?;
    place("zoom", JsValue::from_f64((f64::from(e.nav.zoom) * 100.0).round() / 100.0))?;
    place("largeur", JsValue::from_f64(f64::from(e.canvas.width())))?;
    place("hauteur", JsValue::from_f64(f64::from(e.canvas.height())))?;
    place("premiere_image_ms", JsValue::from_f64(e.first_frame_ms.unwrap_or(0.0).round()))?;
    let window = web_sys::window().ok_or("pas de fenêtre")?;
    js_sys::Reflect::set(&window, &JsValue::from_str("__holo"), &o)?;
    Ok(())
}
