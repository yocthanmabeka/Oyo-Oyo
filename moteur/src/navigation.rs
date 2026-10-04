//! Navigation : le point se morcelle quand on zoome, on s'approche d'un point, on entre
//! dedans, et son monde naît de sa graine. En dézoomant, on ressort.
//!
//! Tout est ici en Rust pur : aucune carte graphique, aucun navigateur. Le rendu ne fait
//! que dessiner la liste de `Sprite` que ce module produit.
//!
//! Mémoire : la pile des niveaux traversés ne garde que deux nombres par niveau (la graine
//! du monde quitté et l'index du point dans lequel on est entré). Le monde parent est
//! recalculé depuis sa graine quand on ressort. Un point pèse une graine.

use crate::univers::{Monde, PointDecl};

/// Un disque lumineux à dessiner. Repère : `y` dans [-1, 1], `x` dans [-aspect, aspect].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub rayon: f32,
    pub couleur: [f32; 4],
}

/// La taille que le point visé doit atteindre à l'écran pour qu'on y entre : il déborde alors
/// de tous les côtés, il nous a absorbés. (La demi-hauteur de l'écran vaut 1.)
pub const RAYON_ENTREE: f32 = 3.0;
/// Zoom juste après l'entrée : les points du nouveau monde sont écartés, exactement là où on
/// les voyait grandir à l'intérieur du point. Rien ne saute.
pub const ZOOM_APRES_ENTREE: f32 = 1.0;
/// La taille d'un point entier, pas encore morcelé. En ressortant d'un monde, on retrouve le
/// point quitté à cette taille : celle qu'avait son monde, refermé, juste avant.
const RAYON_DU_POINT: f32 = 0.42;
/// De combien l'agrandissement double par unité de zoom.
const CROISSANCE: f32 = 1.35;

const DISTANCE_CAMERA: f32 = 3.2;
const FOCALE: f32 = 1.7;

pub struct Navigation {
    racine: PointDecl,
    /// Niveaux au-dessus du niveau courant : (graine du monde, index du point où l'on est entré).
    chemin: Vec<(u64, usize)>,
    courant: Monde,
    focal: usize,
    /// Vrai quand l'utilisateur a touché un point : on ne change plus de cible à la rotation.
    focal_choisi: bool,
    /// Monde du point visé, calculé d'avance pour l'apercevoir avant d'entrer.
    apercu: Option<Monde>,
    pub zoom: f32,
    pub lacet: f32,
    pub tangage: f32,
    /// 1 juste après une entrée ou une sortie, puis décroît : adoucit le changement de monde.
    pub transition: f32,
    /// La couleur du point dans lequel on vient d'entrer : elle remplissait l'écran, elle se
    /// dissipe autour de nous pendant la transition.
    enveloppe: Option<[f32; 3]>,
    /// Secondes écoulées, pour la pulsation du point visé.
    temps: f32,
}

/// Un point enfant tel qu'il apparaît à l'écran.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointEcran {
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub rayon: f32,
    /// Facteur de perspective : plus grand quand le point est tourné vers nous.
    pub k: f32,
}

impl Navigation {
    pub fn new(racine: PointDecl) -> Self {
        let courant = Monde::racine(&racine);
        let mut nav = Navigation {
            racine,
            chemin: Vec::new(),
            courant,
            focal: 0,
            focal_choisi: false,
            apercu: None,
            zoom: 0.0,
            lacet: 0.0,
            tangage: 0.0,
            transition: 0.0,
            enveloppe: None,
            temps: 0.0,
        };
        nav.choisir_focal();
        nav
    }

    pub fn focal(&self) -> usize {
        self.focal
    }

    /// L'utilisateur touche l'écran en (x, y), dans le repère des sprites. Si un point est
    /// sous le doigt, il devient la cible et le reste jusqu'à un autre toucher. Rend vrai
    /// si un point a été touché.
    pub fn viser_ecran(&mut self, x: f32, y: f32, aspect: f32) -> bool {
        let touche = self
            .points_ecran(aspect)
            .into_iter()
            .map(|p| ((p.x - x).powi(2) + (p.y - y).powi(2)).sqrt() - p.rayon.max(0.05) * 1.5)
            .enumerate()
            .filter(|(_, d)| *d <= 0.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i);
        match touche {
            Some(i) => {
                self.focal = self.points_ecran(aspect)[i].index;
                self.focal_choisi = true;
                self.rafraichir_apercu();
                true
            }
            None => false,
        }
    }

    pub fn profondeur(&self) -> usize {
        self.chemin.len() + 1
    }

    pub fn graine_courante(&self) -> u64 {
        self.courant.graine
    }

    pub fn nombre_de_points(&self) -> usize {
        self.courant.enfants.len()
    }

    /// `Origine › 7 › 2` : la racine, puis l'index de chaque point traversé.
    pub fn chemin(&self) -> String {
        let mut s = self.racine.nom.clone();
        for (_, index) in &self.chemin {
            s.push_str(&format!(" › {index}"));
        }
        s
    }

    /// Fait tourner le monde autour de son centre (un doigt qui glisse).
    pub fn tourner(&mut self, d_lacet: f32, d_tangage: f32) {
        self.lacet += d_lacet;
        self.tangage = (self.tangage + d_tangage).clamp(-1.2, 1.2);
        if self.zoom < 1.2 {
            self.choisir_focal();
        }
    }

    /// Avance ou recule (pincement, molette). Entre dans le point visé ou ressort du monde
    /// courant quand le zoom franchit les seuils.
    pub fn zoomer(&mut self, delta: f32) {
        self.zoom += delta;
        if self.zoom < 1.2 {
            self.choisir_focal();
        }
        // On entre quand le point visé a grandi jusqu'à déborder de l'écran : il nous absorbe.
        let rayon = self.rayon_du_point_vise();
        if rayon >= RAYON_ENTREE {
            self.entrer(rayon);
        } else if self.zoom < 0.0 {
            if self.chemin.is_empty() {
                self.zoom = 0.0;
            } else {
                self.sortir();
            }
        }
    }

    /// Fait avancer les effets qui dépendent du temps (en secondes).
    pub fn avancer_temps(&mut self, dt: f32) {
        self.transition = (self.transition - dt * 2.5).max(0.0);
        self.temps = (self.temps + dt) % 1000.0;
    }

    /// La taille, à l'écran, du point visé.
    fn rayon_du_point_vise(&self) -> f32 {
        let (m, s, _) = self.parametres();
        let e = &self.courant.enfants[self.focal];
        let (_, k) = self.projeter(echelle(e.position, m));
        e.rayon * k * (0.35 + 0.65 * m) * s
    }

    fn entrer(&mut self, rayon: f32) {
        let enfant = &self.courant.enfants[self.focal];
        self.enveloppe = Some(enfant.couleur);
        let nouveau = self.apercu.take().filter(|m| m.graine == enfant.graine).unwrap_or_else(|| Monde::depuis_graine(enfant.graine));
        self.chemin.push((self.courant.graine, self.focal));
        self.courant = nouveau;
        // Si le dernier geste a dépassé le seuil, on garde l'élan : la vue continue d'où elle était.
        self.zoom = ZOOM_APRES_ENTREE + (rayon / RAYON_ENTREE).log2() / CROISSANCE;
        self.transition = 1.0;
        self.focal_choisi = false;
        self.choisir_focal();
    }

    fn sortir(&mut self) {
        let (graine, focal) = self.chemin.pop().expect("sortir() n'est appelé qu'en profondeur");
        self.apercu = Some(std::mem::replace(&mut self.courant, Monde::depuis_graine(graine)));
        if self.chemin.is_empty() {
            self.courant = Monde::racine(&self.racine);
        }
        self.focal = focal.min(self.courant.enfants.len() - 1);
        self.focal_choisi = false;
        self.enveloppe = None;
        // On retrouve le point quitté à la taille qu'avait son monde, refermé en un seul point.
        let e = &self.courant.enfants[self.focal];
        let (_, k) = self.projeter(e.position);
        self.zoom = 1.0 + (RAYON_DU_POINT / (e.rayon * k)).max(1.0).log2() / CROISSANCE;
        self.transition = 1.0;
    }

    fn rafraichir_apercu(&mut self) {
        let graine = self.courant.enfants[self.focal].graine;
        if self.apercu.as_ref().is_none_or(|m| m.graine != graine) {
            self.apercu = Some(Monde::depuis_graine(graine));
        }
    }

    /// Sans toucher de l'utilisateur, le point visé est celui qui, après rotation, est le
    /// plus proche du centre de l'écran parmi ceux tournés vers nous.
    fn choisir_focal(&mut self) {
        if self.focal_choisi {
            self.rafraichir_apercu();
            return;
        }
        let mut meilleur = (f32::MAX, 0usize);
        for (i, e) in self.courant.enfants.iter().enumerate() {
            let p = self.tourner_point(e.position);
            let penalite = if p[2] < -0.2 { 10.0 } else { 0.0 };
            let d = p[0] * p[0] + p[1] * p[1] + penalite;
            if d < meilleur.0 {
                meilleur = (d, i);
            }
        }
        self.focal = meilleur.1;
        self.rafraichir_apercu();
    }

    fn tourner_point(&self, p: [f32; 3]) -> [f32; 3] {
        let (sl, cl) = self.lacet.sin_cos();
        let (st, ct) = self.tangage.sin_cos();
        let x = p[0] * cl + p[2] * sl;
        let z = -p[0] * sl + p[2] * cl;
        let y = p[1] * ct - z * st;
        let z = p[1] * st + z * ct;
        [x, y, z]
    }

    /// Projette un point du monde (sphère unité) : (position écran, facteur d'échelle).
    fn projeter(&self, p: [f32; 3]) -> ([f32; 2], f32) {
        let r = self.tourner_point(p);
        let k = FOCALE / (DISTANCE_CAMERA + r[2]);
        ([r[0] * k, r[1] * k], k)
    }

    /// Morcellement (0 : un seul point ; 1 : les enfants sont écartés), agrandissement et
    /// recentrage sur le point visé, tous dérivés du zoom.
    fn parametres(&self) -> (f32, f32, f32) {
        let m = doux((self.zoom / 1.0).clamp(0.0, 1.0));
        let s = 2f32.powf((self.zoom - 1.0).max(0.0) * CROISSANCE);
        let recentrage = doux(((self.zoom - 1.0) / 0.8).clamp(0.0, 1.0));
        (m, s, recentrage)
    }

    /// Décalage de l'écran : on recentre progressivement sur le point visé pendant la plongée.
    fn centre_ecran(&self, m: f32, recentrage: f32) -> [f32; 2] {
        let (c, _) = self.projeter(echelle(self.courant.enfants[self.focal].position, m));
        [c[0] * recentrage, c[1] * recentrage]
    }

    /// Où se trouve, à l'écran, une surface plate posée au centre du monde et tournant avec
    /// lui : la page que voit un personnage, comme une feuille tenue dans le lieu.
    /// Rend : x, y (mêmes unités que les points), agrandissement d'une unité du monde,
    /// lacet, tangage, profondeur dans les mondes, distance de la caméra.
    pub fn surface(&self) -> [f32; 7] {
        let (m, s, recentrage) = self.parametres();
        let centre = self.centre_ecran(m, recentrage);
        [-centre[0] * s, -centre[1] * s, FOCALE / DISTANCE_CAMERA * s, self.lacet, self.tangage, self.chemin.len() as f32, DISTANCE_CAMERA]
    }

    /// Où est chaque point enfant à l'écran. Sert au dessin et au toucher.
    pub fn points_ecran(&self, _aspect: f32) -> Vec<PointEcran> {
        let (m, s, recentrage) = self.parametres();
        let centre = self.centre_ecran(m, recentrage);
        self.courant
            .enfants
            .iter()
            .enumerate()
            .map(|(index, e)| {
                let (p, k) = self.projeter(echelle(e.position, m));
                PointEcran {
                    index,
                    x: (p[0] - centre[0]) * s,
                    y: (p[1] - centre[1]) * s,
                    rayon: e.rayon * k * (0.35 + 0.65 * m) * s,
                    k,
                }
            })
            .collect()
    }

    /// Tout ce qu'il faut dessiner, pour un écran de rapport largeur/hauteur `aspect`.
    pub fn sprites(&self, aspect: f32) -> Vec<Sprite> {
        let (m, s, recentrage) = self.parametres();
        let lumiere = self.courant.lumiere;
        let mut sprites = Vec::with_capacity(self.courant.enfants.len() + 24);
        let centre = self.centre_ecran(m, recentrage);

        // On vient d'entrer : la couleur du point, qui remplissait l'écran, se dissipe autour de nous.
        if let (Some(c), true) = (self.enveloppe, self.transition > 0.0) {
            sprites.push(Sprite { x: 0.0, y: 0.0, rayon: RAYON_ENTREE * (1.0 + 0.8 * (1.0 - self.transition)), couleur: [c[0], c[1], c[2], self.transition] });
        }

        // Le point lui-même : entier au départ, il s'efface à mesure qu'il se morcelle.
        let c = self.courant.couleur;
        let eclat = lumiere * (1.0 - 0.8 * m) * (1.0 - 0.85 * recentrage);
        sprites.push(Sprite { x: -centre[0] * s, y: -centre[1] * s, rayon: RAYON_DU_POINT * (1.0 - 0.75 * m) * s, couleur: [c[0], c[1], c[2], eclat] });

        for p in self.points_ecran(aspect) {
            let e = &self.courant.enfants[p.index];
            let vise = p.index == self.focal;
            let attenuation = if vise { 1.0 } else { 1.0 - 0.5 * recentrage };
            let alpha = (0.35 + 0.65 * m) * lumiere * attenuation;

            // Le point visé est signalé par un halo blanc qui respire, dès que les points
            // sont écartés et jusqu'à ce que l'on soit dedans.
            if vise && m > 0.3 {
                let pulsation = 0.5 + 0.5 * (self.temps * 3.0).sin();
                // Le halo s'efface à mesure que le point nous entoure.
                let force = (m - 0.3) / 0.7 * (1.0 - 0.6 * recentrage) * (1.0 - (p.rayon / RAYON_ENTREE).clamp(0.0, 1.0));
                sprites.push(Sprite { x: p.x, y: p.y, rayon: p.rayon * (1.55 + 0.15 * pulsation), couleur: [1.0, 1.0, 1.0, 0.22 * force] });
            }
            sprites.push(Sprite { x: p.x, y: p.y, rayon: p.rayon, couleur: [e.couleur[0], e.couleur[1], e.couleur[2], alpha] });

            // Dans le point visé, on voit déjà le monde qu'il contient, et il grandit avec lui.
            // Ses points sont placés comme ils le seront une fois dedans : quand le point visé
            // atteint `RAYON_ENTREE`, ils sont exactement là où le nouveau monde les dessine.
            // On ne change pas d'image en entrant : on est absorbé.
            if vise && s > 1.8 {
                if let Some(interieur) = &self.apercu {
                    let visibilite = ((s - 1.8) / 2.5).clamp(0.0, 1.0);
                    let taille = p.rayon / RAYON_ENTREE;
                    let ci = interieur.couleur;
                    sprites.push(Sprite { x: p.x, y: p.y, rayon: RAYON_DU_POINT * 0.25 * taille, couleur: [ci[0], ci[1], ci[2], interieur.lumiere * 0.2 * visibilite] });
                    for g in &interieur.enfants {
                        let (q, k) = self.projeter(g.position);
                        sprites.push(Sprite {
                            x: p.x + q[0] * taille,
                            y: p.y + q[1] * taille,
                            rayon: g.rayon * k * taille,
                            couleur: [g.couleur[0], g.couleur[1], g.couleur[2], interieur.lumiere * visibilite],
                        });
                    }
                }
            }
        }

        sprites.retain(|sp| sp.couleur[3] > 0.002 && sp.rayon > 0.0005 && sp.x.abs() <= aspect + sp.rayon && sp.y.abs() <= 1.0 + sp.rayon);
        sprites
    }
}

fn echelle(p: [f32; 3], f: f32) -> [f32; 3] {
    [p[0] * f, p[1] * f, p[2] * f]
}

fn doux(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn depart() -> Navigation {
        Navigation::new(PointDecl { nom: "Origin".into(), graine: 1, lumiere: 1.0, morceler: 12, couleur: None, palette: Vec::new() })
    }

    #[test]
    fn au_depart_un_seul_point_et_rien_a_l_exterieur_de_l_ecran() {
        let nav = depart();
        let sprites = nav.sprites(0.5);
        assert_eq!(nav.profondeur(), 1);
        assert_eq!(nav.chemin(), "Origin");
        assert!(sprites.len() >= 1 && sprites.len() <= 13);
        assert!(sprites.iter().all(|s| s.x.is_finite() && s.y.is_finite() && s.rayon.is_finite()));
    }

    #[test]
    fn la_surface_suit_le_monde() {
        let mut nav = depart();
        let depart = nav.surface();
        assert_eq!(&depart[..2], &[0.0, 0.0], "au départ, la surface est au centre de l'écran");
        assert!((depart[2] - FOCALE / DISTANCE_CAMERA).abs() < 1e-6);
        assert_eq!(depart[5], 0.0);
        nav.tourner(0.3, -0.2);
        let tournee = nav.surface();
        assert_eq!((tournee[3], tournee[4]), (nav.lacet, nav.tangage), "elle tourne avec le monde");
        for _ in 0..8 {
            nav.zoomer(0.25);
        }
        assert!(nav.surface()[2] > depart[2], "elle grandit quand on s'approche");
        for _ in 0..24 {
            nav.zoomer(0.25);
        }
        assert!(nav.surface()[5] >= 1.0, "entré dans un point, on a quitté le lieu de la surface");
    }

    #[test]
    fn le_zoom_fait_entrer_puis_ressortir() {
        let mut nav = depart();
        for _ in 0..200 {
            if nav.profondeur() == 2 {
                break;
            }
            nav.zoomer(0.1);
        }
        assert_eq!(nav.profondeur(), 2, "zoom {}", nav.zoom);
        assert!(nav.chemin().starts_with("Origin › "));
        assert!(nav.zoom >= ZOOM_APRES_ENTREE && nav.zoom < ZOOM_APRES_ENTREE + 0.2, "zoom {}", nav.zoom);
        let graine_interieure = nav.graine_courante();
        assert_ne!(graine_interieure, 1);

        for _ in 0..200 {
            nav.zoomer(-0.1);
        }
        assert_eq!(nav.profondeur(), 1);
        assert_eq!(nav.graine_courante(), 1);
        assert_eq!(nav.zoom, 0.0, "à la racine, le zoom s'arrête à zéro");
    }

    #[test]
    fn on_est_absorbe_par_le_point_sans_que_rien_ne_saute() {
        // Yocthan, le 2026-10-04 : « le point devrait s'agrandir et nous faire immerger à
        // l'intérieur ; ici, en grossissant, il disparaît. »
        let mut nav = depart();
        let mut avant = Vec::new();
        let mut rayon_avant = 0.0;
        for _ in 0..20000 {
            if nav.profondeur() == 2 {
                break;
            }
            avant = nav.sprites(2.0);
            rayon_avant = nav.rayon_du_point_vise();
            nav.zoomer(0.002);
        }
        assert_eq!(nav.profondeur(), 2);
        // Juste avant d'entrer, le point visé couvre l'écran, même large : il nous entoure.
        assert!(rayon_avant > RAYON_ENTREE * 0.99 && rayon_avant > (2.0f32 * 2.0 + 1.0).sqrt(), "rayon {rayon_avant}");
        // Juste après, chaque point du nouveau monde est là où on le voyait déjà, à la même taille.
        for p in nav.points_ecran(2.0) {
            let deja_la = avant.iter().any(|s| (s.x - p.x).abs() < 0.01 && (s.y - p.y).abs() < 0.01 && (s.rayon / p.rayon - 1.0).abs() < 0.02);
            assert!(deja_la, "le point {} apparaît d'un coup en ({}, {})", p.index, p.x, p.y);
        }
        // Et la couleur du point, qui remplissait l'écran, est encore là : elle se dissipe ensuite.
        let apres = nav.sprites(2.0);
        assert!(apres.iter().any(|s| s.rayon >= RAYON_ENTREE && s.couleur[3] > 0.9), "l'enveloppe manque");
        nav.avancer_temps(1.0);
        assert!(nav.sprites(2.0).iter().all(|s| s.rayon < RAYON_ENTREE), "l'enveloppe ne s'est pas dissipée");
    }

    #[test]
    fn en_ressortant_on_retrouve_le_point_quitte_a_la_taille_de_son_monde() {
        let mut nav = depart();
        for _ in 0..2000 {
            if nav.profondeur() == 2 {
                break;
            }
            nav.zoomer(0.05);
        }
        for _ in 0..2000 {
            if nav.profondeur() == 1 {
                break;
            }
            nav.zoomer(-0.05);
        }
        assert_eq!(nav.profondeur(), 1);
        assert!((nav.rayon_du_point_vise() - RAYON_DU_POINT).abs() < 0.01, "rayon {}", nav.rayon_du_point_vise());
    }

    #[test]
    fn deux_parcours_identiques_donnent_les_memes_images() {
        let parcours = |nav: &mut Navigation| {
            let mut images = Vec::new();
            for i in 0..120 {
                nav.tourner(0.03, if i % 7 == 0 { 0.01 } else { 0.0 });
                nav.zoomer(0.08);
                nav.avancer_temps(1.0 / 60.0);
                images.push(nav.sprites(0.5));
            }
            images
        };
        let (mut a, mut b) = (depart(), depart());
        assert_eq!(parcours(&mut a), parcours(&mut b));
        assert_eq!(a.chemin(), b.chemin());
        assert!(a.profondeur() >= 2);
    }

    #[test]
    fn toucher_un_point_le_rend_cible_et_la_rotation_ne_change_plus_la_cible() {
        let mut nav = depart();
        nav.zoomer(1.0); // les points sont écartés
        let points = nav.points_ecran(0.5);
        let autre = points.iter().find(|p| p.index != nav.focal() && p.k > 0.4).expect("un point tourné vers nous");
        assert!(nav.viser_ecran(autre.x, autre.y, 0.5));
        assert_eq!(nav.focal(), autre.index);
        let cible = nav.focal();
        nav.tourner(0.6, 0.2);
        assert_eq!(nav.focal(), cible, "une cible touchée tient malgré la rotation");
        assert!(!nav.viser_ecran(5.0, 5.0, 0.5), "toucher dans le vide ne change rien");
        assert_eq!(nav.focal(), cible);
        for _ in 0..200 {
            if nav.profondeur() == 2 {
                break;
            }
            nav.zoomer(0.1);
        }
        assert_eq!(nav.profondeur(), 2);
        assert!(nav.chemin().ends_with(&format!(" › {cible}")), "on entre bien dans le point touché : {}", nav.chemin());
    }

    #[test]
    fn la_pile_ne_garde_que_deux_nombres_par_niveau() {
        let mut nav = depart();
        for _ in 0..800 {
            nav.zoomer(0.1);
        }
        assert!(nav.profondeur() >= 9, "profondeur {}", nav.profondeur());
        assert_eq!(std::mem::size_of::<(u64, usize)>() * nav.chemin.len(), 16 * nav.chemin.len());
        // Ressortir jusqu'à la racine redonne exactement le monde de départ.
        for _ in 0..4000 {
            nav.zoomer(-0.1);
        }
        nav.avancer_temps(10.0);
        assert_eq!(nav.profondeur(), 1);
        assert_eq!(nav.sprites(0.5), depart().sprites(0.5));
    }
}
