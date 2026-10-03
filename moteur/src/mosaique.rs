//! La mosaïque : une image vue comme un ensemble de points, un point par pixel.
//!
//! C'est l'idée de Yocthan : le site que l'on regarde est une image découpée en pixels, et
//! chaque pixel est un point de l'Holoverse. Les points sont rangés exactement comme les
//! pixels, pas éparpillés. En zoomant, un point grossit, puis se morcelle en une grille de
//! points plus petits, qui se morcellent à leur tour, sans fin : c'est le Big Bang, rangé.
//!
//! Rien n'est stocké au-delà de l'image : la graine d'un point se calcule à partir de sa
//! place, et sa couleur à partir de sa graine (ADR-005, ADR-008). On ne dessine jamais plus
//! de points que l'écran n'a de place pour en montrer : c'est la limite de perception.

use crate::graine::{graine_enfant, melanger};
use crate::navigation::Sprite;

/// Un point se morcelle en une grille de 4 × 4.
pub const COTE: u64 = 4;
/// En dessous de cette taille à l'écran (en pixels), un point se confond avec un pixel :
/// l'image ordinaire suffit, aucun point n'est dessiné.
pub const SEUIL_POINT: f64 = 6.0;
/// Au-delà de cette taille, un point se morcelle.
pub const SEUIL_MORCELER: f64 = 40.0;
/// Profondeur maximale : 4^20 points par côté dans un seul pixel.
pub const NIVEAU_MAX: u32 = 20;

pub struct Mosaique {
    largeur: u32,
    hauteur: u32,
    /// Rouge, vert, bleu, opacité : quatre octets par pixel.
    couleurs: Vec<u8>,
    graine: u64,
    /// Le pixel de l'image qui se trouve au centre de l'écran.
    pub cx: f64,
    pub cy: f64,
    /// Pixels d'écran pour un pixel de l'image.
    pub echelle: f64,
    echelle_min: f64,
}

impl Mosaique {
    /// `couleurs` contient `largeur × hauteur × 4` octets. L'image est montrée entière dans
    /// une vue de `vue_l × vue_h` pixels.
    pub fn new(largeur: u32, hauteur: u32, couleurs: Vec<u8>, graine: u64, vue_l: f64, vue_h: f64) -> Option<Mosaique> {
        if largeur == 0 || hauteur == 0 || couleurs.len() != largeur as usize * hauteur as usize * 4 {
            return None;
        }
        let echelle = (vue_l / f64::from(largeur)).min(vue_h / f64::from(hauteur)).max(1e-6);
        Some(Mosaique { largeur, hauteur, couleurs, graine, cx: f64::from(largeur) / 2.0, cy: f64::from(hauteur) / 2.0, echelle, echelle_min: echelle })
    }

    /// Combien de fois chaque point s'est morcelé à cette échelle. 0 : un point par pixel.
    pub fn niveau(&self) -> u32 {
        let mut niveau = 0;
        let mut taille = self.echelle;
        while taille >= SEUIL_MORCELER && niveau < NIVEAU_MAX {
            taille /= COTE as f64;
            niveau += 1;
        }
        niveau
    }

    /// De 0 (on voit l'image ordinaire) à 1 (on voit les points).
    pub fn opacite_des_points(&self) -> f64 {
        ((self.echelle - SEUIL_POINT) / SEUIL_POINT).clamp(0.0, 1.0)
    }

    /// Zoome en gardant sous le doigt le point de l'image qui s'y trouvait.
    pub fn zoomer(&mut self, facteur: f64, x: f64, y: f64, vue_l: f64, vue_h: f64) {
        let max = SEUIL_MORCELER * (COTE as f64).powi(NIVEAU_MAX as i32);
        let nouvelle = (self.echelle * facteur).clamp(self.echelle_min, max);
        let (dx, dy) = (x - vue_l / 2.0, y - vue_h / 2.0);
        self.cx += dx / self.echelle - dx / nouvelle;
        self.cy += dy / self.echelle - dy / nouvelle;
        self.echelle = nouvelle;
        self.borner();
    }

    /// Déplace la vue de `dx`, `dy` pixels d'écran.
    pub fn deplacer(&mut self, dx: f64, dy: f64) {
        self.cx -= dx / self.echelle;
        self.cy -= dy / self.echelle;
        self.borner();
    }

    fn borner(&mut self) {
        self.cx = self.cx.clamp(0.0, f64::from(self.largeur));
        self.cy = self.cy.clamp(0.0, f64::from(self.hauteur));
    }

    fn pixel(&self, x: u64, y: u64) -> [f32; 3] {
        let i = (y as usize * self.largeur as usize + x as usize) * 4;
        [f32::from(self.couleurs[i]) / 255.0, f32::from(self.couleurs[i + 1]) / 255.0, f32::from(self.couleurs[i + 2]) / 255.0]
    }

    /// La graine du point posé sur le pixel (x, y) : il ne pèse rien d'autre.
    pub fn graine_du_pixel(&self, x: u64, y: u64) -> u64 {
        graine_enfant(melanger(self.graine ^ y), x as u32)
    }

    /// La couleur d'un point né du morcellement : celle de son parent, déplacée par sa graine.
    /// Même un point noir contient des points qui luisent faiblement.
    fn varier(couleur: [f32; 3], graine: u64) -> [f32; 3] {
        let tirage = |decalage: u32| ((graine >> decalage) & 0xFFFF) as f32 / 65535.0;
        let eclat = 0.55 + 0.9 * tirage(0);
        [
            (couleur[0] * eclat + (tirage(16) - 0.5) * 0.16).clamp(0.0, 1.0),
            (couleur[1] * eclat + (tirage(32) - 0.5) * 0.16).clamp(0.0, 1.0),
            (couleur[2] * eclat + (tirage(48) - 0.5) * 0.16).clamp(0.0, 1.0),
        ]
    }

    /// Les points à dessiner pour une vue de `vue_l × vue_h` pixels. Seuls ceux qui sont à
    /// l'écran sont calculés : leur nombre ne dépend pas de la profondeur du zoom.
    pub fn sprites(&self, vue_l: f64, vue_h: f64) -> Vec<Sprite> {
        let opacite = self.opacite_des_points() as f32;
        if opacite <= 0.0 {
            return Vec::new();
        }
        let niveau = self.niveau();
        let par_pixel = COTE.pow(niveau); // cellules par côté dans un pixel de l'image
        let taille = self.echelle / par_pixel as f64; // taille d'une cellule à l'écran
        // Juste après un morcellement, les enfants ont encore la couleur de leur parent ;
        // elle se déplace vers la leur à mesure qu'ils grossissent.
        let avancement = if niveau == 0 { 1.0 } else { ((taille - SEUIL_MORCELER / COTE as f64) / (SEUIL_MORCELER / 2.0)).clamp(0.0, 1.0) as f32 };

        let demi_l = vue_l / 2.0 / self.echelle;
        let demi_h = vue_h / 2.0 / self.echelle;
        let borne = |v: f64, max: u32| (v.clamp(0.0, f64::from(max)) * par_pixel as f64) as u64;
        let (x0, x1) = (borne(self.cx - demi_l, self.largeur), borne(self.cx + demi_l, self.largeur));
        let (y0, y1) = (borne(self.cy - demi_h, self.hauteur), borne(self.cy + demi_h, self.hauteur));
        let fin_x = (u64::from(self.largeur) * par_pixel).saturating_sub(1);
        let fin_y = (u64::from(self.hauteur) * par_pixel).saturating_sub(1);

        let demi_ecran = vue_h / 2.0;
        let rayon = (taille * 0.46 / demi_ecran) as f32;
        let mut sprites = Vec::new();
        for cy in y0..=y1.min(fin_y) {
            for cx in x0..=x1.min(fin_x) {
                let (px, py) = (cx / par_pixel, cy / par_pixel);
                let mut couleur = self.pixel(px, py);
                let mut graine = self.graine_du_pixel(px, py);
                // On descend du pixel jusqu'à la cellule, un morcellement à la fois.
                for etage in (0..niveau).rev() {
                    let diviseur = COTE.pow(etage);
                    let index = (cx / diviseur % COTE) + COTE * (cy / diviseur % COTE);
                    graine = graine_enfant(graine, index as u32);
                    let enfant = Self::varier(couleur, graine);
                    couleur = if etage == 0 { melange(couleur, enfant, avancement) } else { enfant };
                }
                if couleur[0].max(couleur[1]).max(couleur[2]) * opacite < 0.02 {
                    continue; // un point éteint n'est pas dessiné
                }
                let x = ((cx as f64 + 0.5) / par_pixel as f64 - self.cx) * self.echelle / demi_ecran;
                let y = -((cy as f64 + 0.5) / par_pixel as f64 - self.cy) * self.echelle / demi_ecran;
                sprites.push(Sprite { x: x as f32, y: y as f32, rayon, couleur: [couleur[0], couleur[1], couleur[2], opacite] });
            }
        }
        sprites
    }
}

fn melange(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

#[cfg(test)]
mod tests {
    use super::*;

    const VUE: (f64, f64) = (800.0, 600.0);

    /// Une image de 80 × 60 pixels, toute dorée sauf un pixel noir en haut à gauche.
    fn image() -> Mosaique {
        let mut couleurs = [233u8, 180, 76, 255].repeat(80 * 60);
        couleurs[..3].copy_from_slice(&[0, 0, 0]);
        Mosaique::new(80, 60, couleurs, 1, VUE.0, VUE.1).unwrap()
    }

    #[test]
    fn au_repos_l_image_suffit_puis_chaque_pixel_devient_un_point() {
        let m = Mosaique::new(800, 600, vec![255; 800 * 600 * 4], 1, VUE.0, VUE.1).unwrap();
        assert_eq!(m.echelle, 1.0, "un point fait exactement un pixel");
        assert!(m.sprites(VUE.0, VUE.1).is_empty(), "des points de la taille d'un pixel : l'image ordinaire suffit");

        // L'image de 80 × 60 remplit la vue à l'échelle 10 : un point par pixel, bien rangés.
        let m = image();
        assert_eq!((m.echelle, m.niveau()), (10.0, 0));
        let points = m.sprites(VUE.0, VUE.1);
        assert_eq!(points.len(), 80 * 60 - 1, "un point par pixel ; le pixel noir est éteint");
        let pas = points[1].x - points[0].x;
        assert!((pas - 10.0 / 300.0).abs() < 1e-5, "les points sont espacés d'un pixel de l'image");
        assert!(points.iter().all(|p| (p.rayon - points[0].rayon).abs() < 1e-9), "tous de la même taille");
    }

    #[test]
    fn un_point_qui_grossit_se_morcelle_en_grille() {
        let mut m = image();
        m.zoomer(4.0, 400.0, 300.0, VUE.0, VUE.1);
        assert_eq!((m.echelle, m.niveau()), (40.0, 1), "à 40 pixels, chaque point se morcelle en 4 × 4");
        let points = m.sprites(VUE.0, VUE.1);
        // 800 / 10 = 80 cellules par ligne, 60 par colonne, plus celle qui dépasse au bord.
        assert!((80 * 60..=81 * 61).contains(&points.len()), "{}", points.len());
        m.zoomer(4.0, 400.0, 300.0, VUE.0, VUE.1);
        assert_eq!(m.niveau(), 2);
    }

    #[test]
    fn le_nombre_de_points_ne_grandit_pas_avec_la_profondeur() {
        let mut m = image();
        let plafond = ((VUE.0 / SEUIL_POINT + 2.0) * (VUE.1 / SEUIL_POINT + 2.0)) as usize;
        for _ in 0..60 {
            m.zoomer(1.7, 311.0, 207.0, VUE.0, VUE.1);
            assert!(m.sprites(VUE.0, VUE.1).len() <= plafond, "niveau {}", m.niveau());
        }
        assert!(m.niveau() >= 15, "on est descendu très profond : niveau {}", m.niveau());
        assert!(!m.sprites(VUE.0, VUE.1).is_empty(), "et il y a toujours des points à voir");
    }

    #[test]
    fn le_zoom_garde_sous_le_doigt_ce_qui_s_y_trouvait() {
        let mut m = image();
        // Le doigt est à (200, 150) : il vise le pixel (20, 15).
        let vise = |m: &Mosaique| (m.cx + (200.0 - 400.0) / m.echelle, m.cy + (150.0 - 300.0) / m.echelle);
        let avant = vise(&m);
        m.zoomer(3.0, 200.0, 150.0, VUE.0, VUE.1);
        let apres = vise(&m);
        assert!((avant.0 - apres.0).abs() < 1e-9 && (avant.1 - apres.1).abs() < 1e-9);
        // On ne dézoome pas en deçà de l'image entière.
        m.zoomer(0.001, 0.0, 0.0, VUE.0, VUE.1);
        assert_eq!(m.echelle, 10.0);
    }

    #[test]
    fn la_meme_image_donne_les_memes_points() {
        let (mut a, mut b) = (image(), image());
        for m in [&mut a, &mut b] {
            m.zoomer(37.0, 123.0, 456.0, VUE.0, VUE.1);
            m.deplacer(-14.0, 9.0);
        }
        assert_eq!(a.sprites(VUE.0, VUE.1), b.sprites(VUE.0, VUE.1));
        assert_eq!(a.graine_du_pixel(3, 4), b.graine_du_pixel(3, 4));
        assert_ne!(a.graine_du_pixel(3, 4), a.graine_du_pixel(4, 3), "chaque pixel a sa graine");
    }

    #[test]
    fn meme_le_noir_contient_des_points() {
        let mut m = image();
        // On plonge dans le pixel noir, en haut à gauche.
        for _ in 0..12 {
            m.zoomer(2.0, 5.0, 5.0, VUE.0, VUE.1);
        }
        assert!(m.niveau() >= 3);
        assert!(m.cx < 1.0 && m.cy < 1.0, "on est bien dans le pixel noir");
        let points = m.sprites(VUE.0, VUE.1);
        assert!(!points.is_empty(), "le noir, vu de près, contient des points qui luisent");
    }

    #[test]
    fn une_image_mal_decrite_est_refusee() {
        assert!(Mosaique::new(2, 2, vec![0; 15], 1, 10.0, 10.0).is_none());
        assert!(Mosaique::new(0, 2, vec![], 1, 10.0, 10.0).is_none());
    }
}
