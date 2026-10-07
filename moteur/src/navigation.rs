//! Navigation : le point se morcelle quand on zoome, on s'approche d'un point, on entre
//! dedans, et son monde naît de sa graine. En dézoomant, on ressort.
//!
//! Tout est ici en Rust pur : aucune carte graphique, aucun navigateur. Le rendu ne fait
//! que dessiner la liste de `Sprite` que ce module produit.
//!
//! Mémoire : la pile des niveaux traversés ne garde que deux nombres par niveau (la graine
//! du monde quitté et l'index du point dans lequel on est entré). Le monde parent est
//! recalculé depuis sa graine quand on ressort. Un point pèse une graine.

use crate::universe::{World, PointDecl};

/// Un disque lumineux à dessiner. Repère : `y` dans [-1, 1], `x` dans [-aspect, aspect].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: [f32; 4],
}

/// La taille que le point visé doit atteindre à l'écran pour qu'on y entre : il déborde alors
/// de tous les côtés, il nous a absorbés. (La demi-hauteur de l'écran vaut 1.)
pub const ENTRY_RADIUS: f32 = 3.0;
/// Zoom juste après l'entrée : les points du nouveau monde sont écartés, exactement là où on
/// les voyait grandir à l'intérieur du point. Rien ne saute.
pub const ZOOM_AFTER_ENTRY: f32 = 1.0;
/// La taille d'un point entier, pas encore morcelé. En ressortant d'un monde, on retrouve le
/// point quitté à cette taille : celle qu'avait son monde, refermé, juste avant.
const POINT_RADIUS: f32 = 0.42;
/// De combien l'agrandissement double par unité de zoom.
const GROWTH: f32 = 1.35;

const DISTANCE_CAMERA: f32 = 3.2;
const FOCAL: f32 = 1.7;

pub struct Navigation {
    root: PointDecl,
    /// Niveaux au-dessus du niveau courant : (graine du monde, index du point où l'on est entré).
    path: Vec<(u64, usize)>,
    current: World,
    focal: usize,
    /// Vrai quand l'utilisateur a touché un point : on ne change plus de cible à la rotation.
    chosen_focal: bool,
    /// Monde du point visé, calculé d'avance pour l'apercevoir avant d'entrer.
    preview: Option<World>,
    pub zoom: f32,
    pub yaw: f32,
    pub pitch: f32,
    /// 1 juste après une entrée ou une sortie, puis décroît : adoucit le changement de monde.
    pub transition: f32,
    /// La couleur du point dans lequel on vient d'entrer : elle remplissait l'écran, elle se
    /// dissipe autour de nous pendant la transition.
    envelope: Option<[f32; 3]>,
    /// Secondes écoulées, pour la pulsation du point visé.
    time: f32,
}

/// Un point enfant tel qu'il apparaît à l'écran.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint {
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    /// Facteur de perspective : plus grand quand le point est tourné vers nous.
    pub k: f32,
}

impl Navigation {
    pub fn new(root: PointDecl) -> Self {
        let current = World::root(&root);
        let mut nav = Navigation {
            root,
            path: Vec::new(),
            current,
            focal: 0,
            chosen_focal: false,
            preview: None,
            zoom: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            transition: 0.0,
            envelope: None,
            time: 0.0,
        };
        nav.choose_focal();
        nav
    }

    pub fn focal(&self) -> usize {
        self.focal
    }

    /// L'utilisateur touche l'écran en (x, y), dans le repère des sprites. Si un point est
    /// sous le doigt, il devient la cible et le reste jusqu'à un autre toucher. Rend vrai
    /// si un point a été touché.
    pub fn aim_screen(&mut self, x: f32, y: f32, aspect: f32) -> bool {
        let keypress = self
            .screen_points(aspect)
            .into_iter()
            .map(|p| ((p.x - x).powi(2) + (p.y - y).powi(2)).sqrt() - p.radius.max(0.05) * 1.5)
            .enumerate()
            .filter(|(_, d)| *d <= 0.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i);
        match keypress {
            Some(i) => {
                self.focal = self.screen_points(aspect)[i].index;
                self.chosen_focal = true;
                self.refresh_preview();
                true
            }
            None => false,
        }
    }

    pub fn depth(&self) -> usize {
        self.path.len() + 1
    }

    pub fn current_seed(&self) -> u64 {
        self.current.seed
    }

    pub fn point_count(&self) -> usize {
        self.current.children.len()
    }

    /// `Origine › 7 › 2` : la racine, puis l'index de chaque point traversé.
    pub fn path(&self) -> String {
        let mut s = self.root.name.clone();
        for (_, index) in &self.path {
            s.push_str(&format!(" › {index}"));
        }
        s
    }

    /// Fait tourner le monde autour de son centre (un doigt qui glisse).
    pub fn rotate(&mut self, d_yaw: f32, d_pitch: f32) {
        self.yaw += d_yaw;
        self.pitch = (self.pitch + d_pitch).clamp(-1.2, 1.2);
        if self.zoom < 1.2 {
            self.choose_focal();
        }
    }

    /// Avance ou recule (pincement, molette). Entre dans le point visé ou ressort du monde
    /// courant quand le zoom franchit les seuils.
    pub fn zoom_in(&mut self, delta: f32) {
        self.zoom += delta;
        if self.zoom < 1.2 {
            self.choose_focal();
        }
        // On entre quand le point visé a grandi jusqu'à déborder de l'écran : il nous absorbe.
        let radius = self.aimed_point_radius();
        if radius >= ENTRY_RADIUS {
            self.enter(radius);
        } else if self.zoom < 0.0 {
            if self.path.is_empty() {
                self.zoom = 0.0;
            } else {
                self.exit();
            }
        }
    }

    /// Fait avancer les effets qui dépendent du temps (en secondes).
    pub fn advance_time(&mut self, dt: f32) {
        self.transition = (self.transition - dt * 2.5).max(0.0);
        self.time = (self.time + dt) % 1000.0;
    }

    /// La taille, à l'écran, du point visé.
    fn aimed_point_radius(&self) -> f32 {
        let (m, s, _) = self.params();
        let e = &self.current.children[self.focal];
        let (_, k) = self.project(scale(e.position, m));
        e.radius * k * (0.35 + 0.65 * m) * s
    }

    fn enter(&mut self, radius: f32) {
        let child = &self.current.children[self.focal];
        self.envelope = Some(child.color);
        let new_one = self.preview.take().filter(|m| m.seed == child.seed).unwrap_or_else(|| World::from_seed(child.seed));
        self.path.push((self.current.seed, self.focal));
        self.current = new_one;
        // Si le dernier geste a dépassé le seuil, on garde l'élan : la vue continue d'où elle était.
        self.zoom = ZOOM_AFTER_ENTRY + (radius / ENTRY_RADIUS).log2() / GROWTH;
        self.transition = 1.0;
        self.chosen_focal = false;
        self.choose_focal();
    }

    fn exit(&mut self) {
        let (seed, focal) = self.path.pop().expect("sortir() n'est appelé qu'en profondeur");
        self.preview = Some(std::mem::replace(&mut self.current, World::from_seed(seed)));
        if self.path.is_empty() {
            self.current = World::root(&self.root);
        }
        self.focal = focal.min(self.current.children.len() - 1);
        self.chosen_focal = false;
        self.envelope = None;
        // On retrouve le point quitté à la taille qu'avait son monde, refermé en un seul point.
        let e = &self.current.children[self.focal];
        let (_, k) = self.project(e.position);
        self.zoom = 1.0 + (POINT_RADIUS / (e.radius * k)).max(1.0).log2() / GROWTH;
        self.transition = 1.0;
    }

    fn refresh_preview(&mut self) {
        let seed = self.current.children[self.focal].seed;
        if self.preview.as_ref().is_none_or(|m| m.seed != seed) {
            self.preview = Some(World::from_seed(seed));
        }
    }

    /// Sans toucher de l'utilisateur, le point visé est celui qui, après rotation, est le
    /// plus proche du centre de l'écran parmi ceux tournés vers nous.
    fn choose_focal(&mut self) {
        if self.chosen_focal {
            self.refresh_preview();
            return;
        }
        let mut best = (f32::MAX, 0usize);
        for (i, e) in self.current.children.iter().enumerate() {
            let p = self.rotate_point(e.position);
            let penalty = if p[2] < -0.2 { 10.0 } else { 0.0 };
            let d = p[0] * p[0] + p[1] * p[1] + penalty;
            if d < best.0 {
                best = (d, i);
            }
        }
        self.focal = best.1;
        self.refresh_preview();
    }

    fn rotate_point(&self, p: [f32; 3]) -> [f32; 3] {
        let (sl, cl) = self.yaw.sin_cos();
        let (st, ct) = self.pitch.sin_cos();
        let x = p[0] * cl + p[2] * sl;
        let z = -p[0] * sl + p[2] * cl;
        let y = p[1] * ct - z * st;
        let z = p[1] * st + z * ct;
        [x, y, z]
    }

    /// Projette un point du monde (sphère unité) : (position écran, facteur d'échelle).
    fn project(&self, p: [f32; 3]) -> ([f32; 2], f32) {
        let r = self.rotate_point(p);
        let k = FOCAL / (DISTANCE_CAMERA + r[2]);
        ([r[0] * k, r[1] * k], k)
    }

    /// Morcellement (0 : un seul point ; 1 : les enfants sont écartés), agrandissement et
    /// recentrage sur le point visé, tous dérivés du zoom.
    fn params(&self) -> (f32, f32, f32) {
        let m = soft((self.zoom / 1.0).clamp(0.0, 1.0));
        let s = 2f32.powf((self.zoom - 1.0).max(0.0) * GROWTH);
        let recentering = soft(((self.zoom - 1.0) / 0.8).clamp(0.0, 1.0));
        (m, s, recentering)
    }

    /// Décalage de l'écran : on recentre progressivement sur le point visé pendant la plongée.
    fn screen_center(&self, m: f32, recentering: f32) -> [f32; 2] {
        let (c, _) = self.project(scale(self.current.children[self.focal].position, m));
        [c[0] * recentering, c[1] * recentering]
    }

    /// Où se trouve, à l'écran, une surface plate posée au centre du monde et tournant avec
    /// lui : la page que voit un personnage, comme une feuille tenue dans le lieu.
    /// Rend : x, y (mêmes unités que les points), agrandissement d'une unité du monde,
    /// lacet, tangage, profondeur dans les mondes, distance de la caméra.
    pub fn surface(&self) -> [f32; 7] {
        let (m, s, recentering) = self.params();
        let center = self.screen_center(m, recentering);
        [-center[0] * s, -center[1] * s, FOCAL / DISTANCE_CAMERA * s, self.yaw, self.pitch, self.path.len() as f32, DISTANCE_CAMERA]
    }

    /// Où est chaque point enfant à l'écran. Sert au dessin et au toucher.
    pub fn screen_points(&self, _aspect: f32) -> Vec<ScreenPoint> {
        let (m, s, recentering) = self.params();
        let center = self.screen_center(m, recentering);
        self.current
            .children
            .iter()
            .enumerate()
            .map(|(index, e)| {
                let (p, k) = self.project(scale(e.position, m));
                ScreenPoint {
                    index,
                    x: (p[0] - center[0]) * s,
                    y: (p[1] - center[1]) * s,
                    radius: e.radius * k * (0.35 + 0.65 * m) * s,
                    k,
                }
            })
            .collect()
    }

    /// Tout ce qu'il faut dessiner, pour un écran de rapport largeur/hauteur `aspect`.
    pub fn sprites(&self, aspect: f32) -> Vec<Sprite> {
        let (m, s, recentering) = self.params();
        let light = self.current.light;
        let mut sprites = Vec::with_capacity(self.current.children.len() + 24);
        let center = self.screen_center(m, recentering);

        // On vient d'entrer : la couleur du point, qui remplissait l'écran, se dissipe autour de nous.
        if let (Some(c), true) = (self.envelope, self.transition > 0.0) {
            sprites.push(Sprite { x: 0.0, y: 0.0, radius: ENTRY_RADIUS * (1.0 + 0.8 * (1.0 - self.transition)), color: [c[0], c[1], c[2], self.transition] });
        }

        // Le point lui-même : entier au départ, il s'efface à mesure qu'il se morcelle.
        let c = self.current.color;
        let glow = light * (1.0 - 0.8 * m) * (1.0 - 0.85 * recentering);
        sprites.push(Sprite { x: -center[0] * s, y: -center[1] * s, radius: POINT_RADIUS * (1.0 - 0.75 * m) * s, color: [c[0], c[1], c[2], glow] });

        for p in self.screen_points(aspect) {
            let e = &self.current.children[p.index];
            let aimed = p.index == self.focal;
            let attenuation = if aimed { 1.0 } else { 1.0 - 0.5 * recentering };
            let alpha = (0.35 + 0.65 * m) * light * attenuation;

            // Le point visé est signalé par un halo blanc qui respire, dès que les points
            // sont écartés et jusqu'à ce que l'on soit dedans.
            if aimed && m > 0.3 {
                let pulse = 0.5 + 0.5 * (self.time * 3.0).sin();
                // Le halo s'efface à mesure que le point nous entoure.
                let force = (m - 0.3) / 0.7 * (1.0 - 0.6 * recentering) * (1.0 - (p.radius / ENTRY_RADIUS).clamp(0.0, 1.0));
                sprites.push(Sprite { x: p.x, y: p.y, radius: p.radius * (1.55 + 0.15 * pulse), color: [1.0, 1.0, 1.0, 0.22 * force] });
            }
            sprites.push(Sprite { x: p.x, y: p.y, radius: p.radius, color: [e.color[0], e.color[1], e.color[2], alpha] });

            // Dans le point visé, on voit déjà le monde qu'il contient, et il grandit avec lui.
            // Ses points sont placés comme ils le seront une fois dedans : quand le point visé
            // atteint `RAYON_ENTREE`, ils sont exactement là où le nouveau monde les dessine.
            // On ne change pas d'image en entrant : on est absorbé.
            if aimed && s > 1.8 {
                if let Some(inner) = &self.preview {
                    let visibility = ((s - 1.8) / 2.5).clamp(0.0, 1.0);
                    let size = p.radius / ENTRY_RADIUS;
                    let ci = inner.color;
                    sprites.push(Sprite { x: p.x, y: p.y, radius: POINT_RADIUS * 0.25 * size, color: [ci[0], ci[1], ci[2], inner.light * 0.2 * visibility] });
                    for g in &inner.children {
                        let (q, k) = self.project(g.position);
                        sprites.push(Sprite {
                            x: p.x + q[0] * size,
                            y: p.y + q[1] * size,
                            radius: g.radius * k * size,
                            color: [g.color[0], g.color[1], g.color[2], inner.light * visibility],
                        });
                    }
                }
            }
        }

        sprites.retain(|sp| sp.color[3] > 0.002 && sp.radius > 0.0005 && sp.x.abs() <= aspect + sp.radius && sp.y.abs() <= 1.0 + sp.radius);
        sprites
    }
}

fn scale(p: [f32; 3], f: f32) -> [f32; 3] {
    [p[0] * f, p[1] * f, p[2] * f]
}

fn soft(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_value() -> Navigation {
        Navigation::new(PointDecl { name: "Origin".into(), seed: 1, light: 1.0, shatter: 12, color: None, palette: Vec::new() })
    }

    #[test]
    fn at_start_a_single_point_and_nothing_outside_the_screen() {
        let nav = start_value();
        let sprites = nav.sprites(0.5);
        assert_eq!(nav.depth(), 1);
        assert_eq!(nav.path(), "Origin");
        assert!(sprites.len() >= 1 && sprites.len() <= 13);
        assert!(sprites.iter().all(|s| s.x.is_finite() && s.y.is_finite() && s.radius.is_finite()));
    }

    #[test]
    fn the_surface_follows_the_world() {
        let mut nav = start_value();
        let start_value = nav.surface();
        assert_eq!(&start_value[..2], &[0.0, 0.0], "au départ, la surface est au centre de l'écran");
        assert!((start_value[2] - FOCAL / DISTANCE_CAMERA).abs() < 1e-6);
        assert_eq!(start_value[5], 0.0);
        nav.rotate(0.3, -0.2);
        let turned = nav.surface();
        assert_eq!((turned[3], turned[4]), (nav.yaw, nav.pitch), "elle tourne avec le monde");
        for _ in 0..8 {
            nav.zoom_in(0.25);
        }
        assert!(nav.surface()[2] > start_value[2], "elle grandit quand on s'approche");
        for _ in 0..24 {
            nav.zoom_in(0.25);
        }
        assert!(nav.surface()[5] >= 1.0, "entré dans un point, on a quitté le lieu de la surface");
    }

    #[test]
    fn zoom_enters_then_exits() {
        let mut nav = start_value();
        for _ in 0..200 {
            if nav.depth() == 2 {
                break;
            }
            nav.zoom_in(0.1);
        }
        assert_eq!(nav.depth(), 2, "zoom {}", nav.zoom);
        assert!(nav.path().starts_with("Origin › "));
        assert!(nav.zoom >= ZOOM_AFTER_ENTRY && nav.zoom < ZOOM_AFTER_ENTRY + 0.2, "zoom {}", nav.zoom);
        let inner_seed = nav.current_seed();
        assert_ne!(inner_seed, 1);

        for _ in 0..200 {
            nav.zoom_in(-0.1);
        }
        assert_eq!(nav.depth(), 1);
        assert_eq!(nav.current_seed(), 1);
        assert_eq!(nav.zoom, 0.0, "à la racine, le zoom s'arrête à zéro");
    }

    #[test]
    fn absorbed_by_the_point_without_any_jump() {
        // Yocthan, le 2026-10-04 : « le point devrait s'agrandir et nous faire immerger à
        // l'intérieur ; ici, en grossissant, il disparaît. »
        let mut nav = start_value();
        let mut before = Vec::new();
        let mut radius_before = 0.0;
        for _ in 0..20000 {
            if nav.depth() == 2 {
                break;
            }
            before = nav.sprites(2.0);
            radius_before = nav.aimed_point_radius();
            nav.zoom_in(0.002);
        }
        assert_eq!(nav.depth(), 2);
        // Juste avant d'entrer, le point visé couvre l'écran, même large : il nous entoure.
        assert!(radius_before > ENTRY_RADIUS * 0.99 && radius_before > (2.0f32 * 2.0 + 1.0).sqrt(), "rayon {radius_before}");
        // Juste après, chaque point du nouveau monde est là où on le voyait déjà, à la même taille.
        for p in nav.screen_points(2.0) {
            let already_there = before.iter().any(|s| (s.x - p.x).abs() < 0.01 && (s.y - p.y).abs() < 0.01 && (s.radius / p.radius - 1.0).abs() < 0.02);
            assert!(already_there, "le point {} apparaît d'un coup en ({}, {})", p.index, p.x, p.y);
        }
        // Et la couleur du point, qui remplissait l'écran, est encore là : elle se dissipe ensuite.
        let after = nav.sprites(2.0);
        assert!(after.iter().any(|s| s.radius >= ENTRY_RADIUS && s.color[3] > 0.9), "l'enveloppe manque");
        nav.advance_time(1.0);
        assert!(nav.sprites(2.0).iter().all(|s| s.radius < ENTRY_RADIUS), "l'enveloppe ne s'est pas dissipée");
    }

    #[test]
    fn on_exit_the_left_point_is_found_again_at_the_size_of_its_world() {
        let mut nav = start_value();
        for _ in 0..2000 {
            if nav.depth() == 2 {
                break;
            }
            nav.zoom_in(0.05);
        }
        for _ in 0..2000 {
            if nav.depth() == 1 {
                break;
            }
            nav.zoom_in(-0.05);
        }
        assert_eq!(nav.depth(), 1);
        assert!((nav.aimed_point_radius() - POINT_RADIUS).abs() < 0.01, "rayon {}", nav.aimed_point_radius());
    }

    #[test]
    fn two_identical_journeys_give_the_same_frames() {
        let journey = |nav: &mut Navigation| {
            let mut images = Vec::new();
            for i in 0..120 {
                nav.rotate(0.03, if i % 7 == 0 { 0.01 } else { 0.0 });
                nav.zoom_in(0.08);
                nav.advance_time(1.0 / 60.0);
                images.push(nav.sprites(0.5));
            }
            images
        };
        let (mut a, mut b) = (start_value(), start_value());
        assert_eq!(journey(&mut a), journey(&mut b));
        assert_eq!(a.path(), b.path());
        assert!(a.depth() >= 2);
    }

    #[test]
    fn touching_a_point_targets_it_and_rotation_no_longer_changes_the_target() {
        let mut nav = start_value();
        nav.zoom_in(1.0); // les points sont écartés
        let points = nav.screen_points(0.5);
        let other = points.iter().find(|p| p.index != nav.focal() && p.k > 0.4).expect("un point tourné vers nous");
        assert!(nav.aim_screen(other.x, other.y, 0.5));
        assert_eq!(nav.focal(), other.index);
        let target = nav.focal();
        nav.rotate(0.6, 0.2);
        assert_eq!(nav.focal(), target, "une cible touchée tient malgré la rotation");
        assert!(!nav.aim_screen(5.0, 5.0, 0.5), "toucher dans le vide ne change rien");
        assert_eq!(nav.focal(), target);
        for _ in 0..200 {
            if nav.depth() == 2 {
                break;
            }
            nav.zoom_in(0.1);
        }
        assert_eq!(nav.depth(), 2);
        assert!(nav.path().ends_with(&format!(" › {target}")), "on entre bien dans le point touché : {}", nav.path());
    }

    #[test]
    fn the_stack_keeps_only_two_numbers_per_level() {
        let mut nav = start_value();
        for _ in 0..800 {
            nav.zoom_in(0.1);
        }
        assert!(nav.depth() >= 9, "profondeur {}", nav.depth());
        assert_eq!(std::mem::size_of::<(u64, usize)>() * nav.path.len(), 16 * nav.path.len());
        // Ressortir jusqu'à la racine redonne exactement le monde de départ.
        for _ in 0..4000 {
            nav.zoom_in(-0.1);
        }
        nav.advance_time(10.0);
        assert_eq!(nav.depth(), 1);
        assert_eq!(nav.sprites(0.5), start_value().sprites(0.5));
    }
}
