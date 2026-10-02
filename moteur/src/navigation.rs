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

/// Zoom à partir duquel on entre dans le point visé.
pub const ZOOM_ENTREE: f32 = 3.6;
/// Zoom juste après l'entrée : le nouveau point est entier, prêt à se morceler.
pub const ZOOM_APRES_ENTREE: f32 = 0.55;
/// Zoom juste après la sortie : on voit de nouveau le point que l'on vient de quitter, gros.
pub const ZOOM_APRES_SORTIE: f32 = 3.0;

const DISTANCE_CAMERA: f32 = 3.2;
const FOCALE: f32 = 1.7;

pub struct Navigation {
    racine: PointDecl,
    /// Niveaux au-dessus du niveau courant : (graine du monde, index du point où l'on est entré).
    chemin: Vec<(u64, usize)>,
    courant: Monde,
    focal: usize,
    /// Monde du point visé, calculé d'avance pour l'apercevoir avant d'entrer.
    apercu: Option<Monde>,
    pub zoom: f32,
    pub lacet: f32,
    pub tangage: f32,
    /// 1 juste après une entrée ou une sortie, puis décroît : adoucit le changement de monde.
    pub transition: f32,
}

impl Navigation {
    pub fn new(racine: PointDecl) -> Self {
        let courant = Monde::racine(&racine);
        let mut nav = Navigation { racine, chemin: Vec::new(), courant, focal: 0, apercu: None, zoom: 0.0, lacet: 0.0, tangage: 0.0, transition: 0.0 };
        nav.choisir_focal();
        nav
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
        if self.zoom >= ZOOM_ENTREE {
            self.entrer();
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
    }

    fn entrer(&mut self) {
        let enfant = &self.courant.enfants[self.focal];
        let nouveau = self.apercu.take().filter(|m| m.graine == enfant.graine).unwrap_or_else(|| Monde::depuis_graine(enfant.graine));
        self.chemin.push((self.courant.graine, self.focal));
        self.courant = nouveau;
        self.zoom = ZOOM_APRES_ENTREE;
        self.transition = 1.0;
        self.choisir_focal();
    }

    fn sortir(&mut self) {
        let (graine, focal) = self.chemin.pop().expect("sortir() n'est appelé qu'en profondeur");
        self.apercu = Some(std::mem::replace(&mut self.courant, Monde::depuis_graine(graine)));
        if self.chemin.is_empty() {
            self.courant = Monde::racine(&self.racine);
        }
        self.focal = focal.min(self.courant.enfants.len() - 1);
        self.zoom = ZOOM_APRES_SORTIE;
        self.transition = 1.0;
    }

    /// Le point visé est celui qui, après rotation, est le plus proche du centre de l'écran
    /// parmi ceux tournés vers nous.
    fn choisir_focal(&mut self) {
        let mut meilleur = (f32::MAX, 0usize);
        for (i, e) in self.courant.enfants.iter().enumerate() {
            let p = self.tourner_point(e.position);
            let penalite = if p[2] < -0.2 { 10.0 } else { 0.0 };
            let d = p[0] * p[0] + p[1] * p[1] + penalite;
            if d < meilleur.0 {
                meilleur = (d, i);
            }
        }
        if meilleur.1 != self.focal || self.apercu.is_none() {
            self.focal = meilleur.1;
            self.apercu = None;
        }
        let graine = self.courant.enfants[self.focal].graine;
        if self.apercu.as_ref().is_none_or(|m| m.graine != graine) {
            self.apercu = Some(Monde::depuis_graine(graine));
        }
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
        let s = 2f32.powf((self.zoom - 1.0).max(0.0) * 1.35);
        let recentrage = doux(((self.zoom - 1.0) / 0.8).clamp(0.0, 1.0));
        (m, s, recentrage)
    }

    /// Tout ce qu'il faut dessiner, pour un écran de rapport largeur/hauteur `aspect`.
    pub fn sprites(&self, aspect: f32) -> Vec<Sprite> {
        let (m, s, recentrage) = self.parametres();
        let voile = 1.0 - 0.6 * self.transition;
        let lumiere = self.courant.lumiere;
        let mut sprites = Vec::with_capacity(self.courant.enfants.len() + 24);

        let focal = &self.courant.enfants[self.focal];
        let (centre_focal, _) = self.projeter(echelle(focal.position, m));
        let centre = [centre_focal[0] * recentrage, centre_focal[1] * recentrage];
        let ecran = |p: [f32; 2]| [(p[0] - centre[0]) * s, (p[1] - centre[1]) * s];

        // Le point lui-même : entier au départ, il s'efface à mesure qu'il se morcelle.
        let c = self.courant.couleur;
        let eclat = lumiere * (1.0 - 0.8 * m) * (1.0 - 0.85 * recentrage) * voile;
        let [x, y] = ecran([0.0, 0.0]);
        sprites.push(Sprite { x, y, rayon: 0.42 * (1.0 - 0.75 * m) * s, couleur: [c[0], c[1], c[2], eclat] });

        for (i, e) in self.courant.enfants.iter().enumerate() {
            let (p, k) = self.projeter(echelle(e.position, m));
            let [x, y] = ecran(p);
            let rayon = e.rayon * k * (0.35 + 0.65 * m) * s;
            let attenuation = if i == self.focal { 1.0 } else { 1.0 - 0.5 * recentrage };
            let alpha = (0.35 + 0.65 * m) * lumiere * voile * attenuation;
            sprites.push(Sprite { x, y, rayon, couleur: [e.couleur[0], e.couleur[1], e.couleur[2], alpha] });

            // Dans le point visé, on aperçoit déjà le monde qu'il contient.
            if i == self.focal && s > 1.8 {
                if let Some(interieur) = &self.apercu {
                    let visibilite = ((s - 1.8) / 2.5).clamp(0.0, 1.0) * voile;
                    for g in &interieur.enfants {
                        let q = self.tourner_point(g.position);
                        sprites.push(Sprite {
                            x: x + q[0] * rayon * 0.72,
                            y: y + q[1] * rayon * 0.72,
                            rayon: (0.1 + 0.03 * q[2]) * rayon,
                            couleur: [g.couleur[0], g.couleur[1], g.couleur[2], visibilite * (0.6 + 0.4 * q[2].max(0.0))],
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
        Navigation::new(PointDecl { nom: "Origine".into(), graine: 1, lumiere: 1.0, morceler: 12 })
    }

    #[test]
    fn au_depart_un_seul_point_et_rien_a_l_exterieur_de_l_ecran() {
        let nav = depart();
        let sprites = nav.sprites(0.5);
        assert_eq!(nav.profondeur(), 1);
        assert_eq!(nav.chemin(), "Origine");
        assert!(sprites.len() >= 1 && sprites.len() <= 13);
        assert!(sprites.iter().all(|s| s.x.is_finite() && s.y.is_finite() && s.rayon.is_finite()));
    }

    #[test]
    fn le_zoom_fait_entrer_puis_ressortir() {
        let mut nav = depart();
        for _ in 0..40 {
            nav.zoomer(0.1);
        }
        assert_eq!(nav.profondeur(), 2, "zoom {}", nav.zoom);
        assert!(nav.chemin().starts_with("Origine › "));
        assert!((nav.zoom - ZOOM_APRES_ENTREE).abs() < 1e-5 || nav.zoom > ZOOM_APRES_ENTREE);
        let graine_interieure = nav.graine_courante();
        assert_ne!(graine_interieure, 1);

        for _ in 0..60 {
            nav.zoomer(-0.1);
        }
        assert_eq!(nav.profondeur(), 1);
        assert_eq!(nav.graine_courante(), 1);
        assert_eq!(nav.zoom, 0.0, "à la racine, le zoom s'arrête à zéro");
    }

    #[test]
    fn deux_parcours_identiques_donnent_les_memes_images() {
        let parcours = |nav: &mut Navigation| {
            let mut images = Vec::new();
            for i in 0..60 {
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
    fn la_pile_ne_garde_que_deux_nombres_par_niveau() {
        let mut nav = depart();
        for _ in 0..400 {
            nav.zoomer(0.1);
        }
        assert!(nav.profondeur() >= 9, "profondeur {}", nav.profondeur());
        assert_eq!(std::mem::size_of::<(u64, usize)>() * nav.chemin.len(), 16 * nav.chemin.len());
        // Ressortir jusqu'à la racine redonne exactement le monde de départ.
        for _ in 0..2000 {
            nav.zoomer(-0.1);
        }
        nav.avancer_temps(10.0);
        assert_eq!(nav.profondeur(), 1);
        assert_eq!(nav.sprites(0.5), depart().sprites(0.5));
    }
}
