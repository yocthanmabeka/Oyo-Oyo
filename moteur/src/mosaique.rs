//! La mosaïque : une image vue comme un ensemble de points, un point par pixel.
//!
//! C'est l'idée de Yocthan : le site que l'on regarde est une image découpée en pixels, et
//! chaque pixel est un point de l'Holoverse. Les points sont rangés exactement comme les
//! pixels, pas éparpillés. En zoomant, un point grossit, puis se morcelle en une grille de
//! points plus petits, qui se morcellent à leur tour, sans fin : c'est le Big Bang, rangé.
//!
//! Les points ont trois coordonnées. De face, la page est plate comme une feuille. Quand on
//! la fait tourner, on la voit de biais, et ce qui est lumineux (les lettres) se soulève
//! au-dessus du fond : le relief.
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
/// On ne tourne pas la page au-delà de cet angle (en radians, environ 52°) : plus loin, on
/// la verrait par la tranche et il faudrait dessiner des points jusqu'à l'horizon.
pub const ANGLE_MAX: f64 = 0.9;
/// Hauteur, en pixels de l'image, à laquelle un point blanc se soulève quand la page est de biais.
pub const RELIEF: f64 = 10.0;
/// Le relief ne dépasse jamais cette hauteur à l'écran (en pixels) : en zoomant très profond,
/// il emporterait sinon les points hors de vue.
pub const RELIEF_MAX: f64 = 80.0;
/// On ne dessine jamais plus de points que cela, même vu de biais.
pub const POINTS_MAX: usize = 200_000;

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
    /// La page tournée autour de son axe vertical (lacet) et horizontal (tangage), en radians.
    pub lacet: f64,
    pub tangage: f64,
    /// Vrai : glisser fait tourner la page ; faux : glisser la déplace.
    pub tourner: bool,
}

/// Distance de l'œil à la page, en pixels d'écran. La même valeur sert à la page ordinaire,
/// pour qu'elle se superpose exactement aux points.
pub fn distance(vue_h: f64) -> f64 {
    vue_h * 1.6
}

impl Mosaique {
    /// `couleurs` contient `largeur × hauteur × 4` octets. L'image est montrée entière dans
    /// une vue de `vue_l × vue_h` pixels.
    pub fn new(largeur: u32, hauteur: u32, couleurs: Vec<u8>, graine: u64, vue_l: f64, vue_h: f64) -> Option<Mosaique> {
        if largeur == 0 || hauteur == 0 || couleurs.len() != largeur as usize * hauteur as usize * 4 {
            return None;
        }
        let echelle = (vue_l / f64::from(largeur)).min(vue_h / f64::from(hauteur)).max(1e-6);
        Some(Mosaique {
            largeur,
            hauteur,
            couleurs,
            graine,
            cx: f64::from(largeur) / 2.0,
            cy: f64::from(hauteur) / 2.0,
            echelle,
            echelle_min: echelle,
            lacet: 0.0,
            tangage: 0.0,
            tourner: false,
        })
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

    /// De 0 (la page est de face, plate) à 1 (elle est de biais, le relief est entier).
    pub fn inclinaison(&self) -> f64 {
        ((self.lacet.abs() + self.tangage.abs()) / 0.35).clamp(0.0, 1.0)
    }

    /// Zoome en gardant sous le doigt le point de l'image qui s'y trouvait.
    pub fn zoomer(&mut self, facteur: f64, x: f64, y: f64, vue_l: f64, vue_h: f64) {
        let max = SEUIL_MORCELER * (COTE as f64).powi(NIVEAU_MAX as i32);
        let nouvelle = (self.echelle * facteur).clamp(self.echelle_min, max);
        let (dx, dy) = (x - vue_l / 2.0, y - vue_h / 2.0);
        // Où le doigt touche la page, en pixels d'écran comptés sur la page elle-même.
        let [ux, uy] = self.sur_la_page(dx, dy, distance(vue_h)).unwrap_or([dx, dy]);
        self.cx += ux / self.echelle - ux / nouvelle;
        self.cy += uy / self.echelle - uy / nouvelle;
        self.echelle = nouvelle;
        self.borner();
    }

    /// Déplace la vue de `dx`, `dy` pixels d'écran.
    pub fn deplacer(&mut self, dx: f64, dy: f64) {
        self.cx -= dx / self.echelle;
        self.cy -= dy / self.echelle;
        self.borner();
    }

    /// Fait tourner la page : on la voit de biais.
    pub fn pivoter(&mut self, lacet: f64, tangage: f64) {
        self.lacet = (self.lacet + lacet).clamp(-ANGLE_MAX, ANGLE_MAX);
        self.tangage = (self.tangage + tangage).clamp(-ANGLE_MAX, ANGLE_MAX);
    }

    /// Remet la page de face.
    pub fn de_face(&mut self) {
        self.lacet = 0.0;
        self.tangage = 0.0;
    }

    fn borner(&mut self) {
        self.cx = self.cx.clamp(0.0, f64::from(self.largeur));
        self.cy = self.cy.clamp(0.0, f64::from(self.hauteur));
    }

    /// Place à l'écran un point de l'espace. `u` est compté en pixels d'écran depuis le centre
    /// de la vue : x vers la droite, y vers le bas, z vers l'œil. Rend la position à l'écran
    /// (depuis le centre) et l'agrandissement dû à la perspective ; rien si le point est
    /// derrière l'œil. Ce sont les calculs que fait le navigateur pour
    /// `perspective(d) rotateX(tangage) rotateY(lacet)`.
    pub fn projeter(&self, u: [f64; 3], d: f64) -> Option<([f64; 2], f64)> {
        let (sl, cl) = self.lacet.sin_cos();
        let (st, ct) = self.tangage.sin_cos();
        let x = u[0] * cl + u[2] * sl;
        let z = -u[0] * sl + u[2] * cl;
        let y = u[1] * ct - z * st;
        let z = u[1] * st + z * ct;
        let reste = d - z;
        (reste > d * 0.05).then(|| ([x * d / reste, y * d / reste], d / reste))
    }

    /// L'inverse : quel endroit de la page (à plat, z = 0) se trouve sous ce point de l'écran.
    pub fn sur_la_page(&self, sx: f64, sy: f64, d: f64) -> Option<[f64; 2]> {
        let (sl, cl) = self.lacet.sin_cos();
        let (st, ct) = self.tangage.sin_cos();
        let detourner = |v: [f64; 3]| {
            let y = v[1] * ct + v[2] * st;
            let z = -v[1] * st + v[2] * ct;
            [v[0] * cl - z * sl, y, v[0] * sl + z * cl]
        };
        let oeil = detourner([0.0, 0.0, d]);
        let rayon = detourner([sx, sy, -d]);
        (rayon[2] < -1e-9).then(|| {
            let t = -oeil[2] / rayon[2];
            [oeil[0] + t * rayon[0], oeil[1] + t * rayon[1]]
        })
    }

    fn pixel(&self, x: usize, y: usize) -> [f32; 3] {
        let i = (y * self.largeur as usize + x) * 4;
        [f32::from(self.couleurs[i]) / 255.0, f32::from(self.couleurs[i + 1]) / 255.0, f32::from(self.couleurs[i + 2]) / 255.0]
    }

    /// La couleur de l'image à un endroit quelconque, fondue entre les quatre pixels voisins.
    /// C'est ce qui garde les lettres lisses quand leurs points se morcellent : sans cela,
    /// chaque pixel deviendrait un carré.
    fn couleur_fondue(&self, x: f64, y: f64) -> [f32; 3] {
        let gx = (x - 0.5).clamp(0.0, f64::from(self.largeur - 1));
        let gy = (y - 0.5).clamp(0.0, f64::from(self.hauteur - 1));
        let (x0, y0) = (gx.floor() as usize, gy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.largeur as usize - 1), (y0 + 1).min(self.hauteur as usize - 1));
        let (tx, ty) = ((gx - x0 as f64) as f32, (gy - y0 as f64) as f32);
        let haut = melange(self.pixel(x0, y0), self.pixel(x1, y0), tx);
        let bas = melange(self.pixel(x0, y1), self.pixel(x1, y1), tx);
        melange(haut, bas, ty)
    }

    /// La graine du point posé sur le pixel (x, y) : il ne pèse rien d'autre.
    pub fn graine_du_pixel(&self, x: u64, y: u64) -> u64 {
        graine_enfant(melanger(self.graine ^ y), x as u32)
    }

    /// La couleur d'un point né du morcellement : celle de l'image à cet endroit, déplacée
    /// par sa graine. Les premiers morcellements la déplacent peu, pour que les lettres
    /// restent nettes ; plus on descend, plus chaque point prend sa couleur propre.
    /// Même un point noir contient des points qui luisent faiblement.
    fn varier(couleur: [f32; 3], graine: u64, force: f32) -> [f32; 3] {
        let tirage = |decalage: u32| ((graine >> decalage) & 0xFFFF) as f32 / 65535.0 - 0.5;
        let eclat = 1.0 + 2.0 * force * tirage(0);
        [
            (couleur[0] * eclat + tirage(16) * 0.5 * force).clamp(0.0, 1.0),
            (couleur[1] * eclat + tirage(32) * 0.5 * force).clamp(0.0, 1.0),
            (couleur[2] * eclat + tirage(48) * 0.5 * force).clamp(0.0, 1.0),
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
        let d = distance(vue_h);
        let inclinaison = self.inclinaison();

        // La partie de l'image qui est à l'écran : ce que couvrent les quatre coins de la vue.
        let coins = [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].map(|(sx, sy)| self.sur_la_page(sx * vue_l / 2.0, sy * vue_h / 2.0, d));
        let (mut gauche, mut droite, mut haut, mut bas) = (0.0, f64::from(self.largeur), 0.0, f64::from(self.hauteur));
        if let [Some(a), Some(b), Some(c), Some(e)] = coins {
            // Une marge pour les points soulevés par le relief, qui débordent de leur place.
            let marge = (2.0 * RELIEF_MAX * inclinaison + taille) / self.echelle;
            let xs = [a[0], b[0], c[0], e[0]].map(|v| self.cx + v / self.echelle);
            let ys = [a[1], b[1], c[1], e[1]].map(|v| self.cy + v / self.echelle);
            gauche = xs.iter().copied().fold(f64::MAX, f64::min) - marge;
            droite = xs.iter().copied().fold(f64::MIN, f64::max) + marge;
            haut = ys.iter().copied().fold(f64::MAX, f64::min) - marge;
            bas = ys.iter().copied().fold(f64::MIN, f64::max) + marge;
        }
        let borne = |v: f64, max: u32| (v.clamp(0.0, f64::from(max)) * par_pixel as f64) as u64;
        let (x0, x1) = (borne(gauche, self.largeur), borne(droite, self.largeur));
        let (y0, y1) = (borne(haut, self.hauteur), borne(bas, self.hauteur));
        let fin_x = (u64::from(self.largeur) * par_pixel).saturating_sub(1);
        let fin_y = (u64::from(self.hauteur) * par_pixel).saturating_sub(1);

        let demi_ecran = vue_h / 2.0;
        let mut sprites = Vec::new();
        for cy in y0..=y1.min(fin_y) {
            for cx in x0..=x1.min(fin_x) {
                if sprites.len() >= POINTS_MAX {
                    return sprites;
                }
                let (px, py) = (cx / par_pixel, cy / par_pixel);
                // Où est la cellule dans l'image, en pixels de l'image.
                let (ix, iy) = ((cx as f64 + 0.5) / par_pixel as f64, (cy as f64 + 0.5) / par_pixel as f64);
                let fond = if niveau == 0 { self.pixel(px as usize, py as usize) } else { self.couleur_fondue(ix, iy) };
                let mut couleur = fond;
                let mut graine = self.graine_du_pixel(px, py);
                let mut ecart = 0.0; // de combien ce point s'écarte de la page, vers l'œil
                // On descend du pixel jusqu'à la cellule, un morcellement à la fois.
                for etage in (0..niveau).rev() {
                    let diviseur = COTE.pow(etage);
                    let index = (cx / diviseur % COTE) + COTE * (cy / diviseur % COTE);
                    graine = graine_enfant(graine, index as u32);
                    let force = (0.05 * (niveau - etage) as f32).min(0.4);
                    let enfant = Self::varier(couleur, graine, force);
                    couleur = if etage == 0 { melange(couleur, enfant, avancement) } else { enfant };
                    // Les enfants d'un point ne sont pas tous à la même hauteur.
                    ecart += (((graine >> 20) & 0xFFFF) as f64 / 65535.0 - 0.5) * self.echelle / (diviseur * COTE) as f64 * 1.5;
                }
                if couleur[0].max(couleur[1]).max(couleur[2]) * opacite < 0.02 {
                    continue; // un point éteint n'est pas dessiné
                }
                // Le relief : ce qui est lumineux dans l'image se soulève, quand la page est de biais.
                let lumiere = f64::from(fond[0].max(fond[1]).max(fond[2]));
                let z = ((RELIEF * self.echelle).min(RELIEF_MAX) * lumiere + ecart.clamp(-RELIEF_MAX, RELIEF_MAX)) * inclinaison;
                let Some(([x, y], k)) = self.projeter([(ix - self.cx) * self.echelle, (iy - self.cy) * self.echelle, z], d) else { continue };
                let rayon = taille * 0.46 * k;
                if x.abs() > vue_l / 2.0 + rayon * 2.0 || y.abs() > vue_h / 2.0 + rayon * 2.0 {
                    continue;
                }
                sprites.push(Sprite {
                    x: (x / demi_ecran) as f32,
                    y: (-y / demi_ecran) as f32,
                    rayon: (rayon / demi_ecran) as f32,
                    couleur: [couleur[0], couleur[1], couleur[2], opacite],
                });
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
        assert!(points.iter().all(|p| (p.rayon - points[0].rayon).abs() < 1e-9), "de face, la page est plate : tous de la même taille");
    }

    #[test]
    fn un_point_qui_grossit_se_morcelle_en_grille() {
        let mut m = image();
        m.zoomer(4.0, 400.0, 300.0, VUE.0, VUE.1);
        assert_eq!((m.echelle, m.niveau()), (40.0, 1), "à 40 pixels, chaque point se morcelle en 4 × 4");
        let points = m.sprites(VUE.0, VUE.1);
        // 800 / 10 = 80 cellules par ligne, 60 par colonne, plus celles qui dépassent au bord.
        assert!((80 * 60..=82 * 62).contains(&points.len()), "{}", points.len());
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
        // De face, puis la page de biais : dans les deux cas le doigt, à (200, 150), ne lâche pas son pixel.
        for (lacet, tangage) in [(0.0, 0.0), (0.5, -0.3)] {
            let mut m = image();
            m.pivoter(lacet, tangage);
            let vise = |m: &Mosaique| {
                let [ux, uy] = m.sur_la_page(200.0 - 400.0, 150.0 - 300.0, distance(VUE.1)).unwrap();
                (m.cx + ux / m.echelle, m.cy + uy / m.echelle)
            };
            let avant = vise(&m);
            m.zoomer(3.0, 200.0, 150.0, VUE.0, VUE.1);
            let apres = vise(&m);
            assert!((avant.0 - apres.0).abs() < 1e-9 && (avant.1 - apres.1).abs() < 1e-9);
        }
        // On ne dézoome pas en deçà de l'image entière.
        let mut m = image();
        m.zoomer(0.001, 0.0, 0.0, VUE.0, VUE.1);
        assert_eq!(m.echelle, 10.0);
    }

    #[test]
    fn la_meme_image_donne_les_memes_points() {
        let (mut a, mut b) = (image(), image());
        for m in [&mut a, &mut b] {
            m.zoomer(37.0, 123.0, 456.0, VUE.0, VUE.1);
            m.deplacer(-14.0, 9.0);
            m.pivoter(0.4, 0.2);
        }
        assert_eq!(a.sprites(VUE.0, VUE.1), b.sprites(VUE.0, VUE.1));
        assert_eq!(a.graine_du_pixel(3, 4), b.graine_du_pixel(3, 4));
        assert_ne!(a.graine_du_pixel(3, 4), a.graine_du_pixel(4, 3), "chaque pixel a sa graine");
    }

    #[test]
    fn meme_le_noir_contient_des_points() {
        let noir = Mosaique::new(80, 60, [0u8, 0, 0, 255].repeat(80 * 60), 1, VUE.0, VUE.1);
        let mut m = noir.unwrap();
        assert!(m.sprites(VUE.0, VUE.1).is_empty(), "une page noire : aucun point allumé");
        for _ in 0..14 {
            m.zoomer(2.0, 400.0, 300.0, VUE.0, VUE.1);
        }
        assert!(m.niveau() >= 4);
        assert!(!m.sprites(VUE.0, VUE.1).is_empty(), "le noir, vu de près, contient des points qui luisent");
    }

    #[test]
    fn de_biais_la_page_a_de_la_profondeur() {
        let mut m = image();
        let d = distance(VUE.1);
        // Aller et retour : un endroit de la page, vu à l'écran, se retrouve sur la page.
        m.pivoter(0.6, -0.4);
        let (ecran, _) = m.projeter([120.0, -80.0, 0.0], d).unwrap();
        let retour = m.sur_la_page(ecran[0], ecran[1], d).unwrap();
        assert!((retour[0] - 120.0).abs() < 1e-6 && (retour[1] + 80.0).abs() < 1e-6);

        // Le côté qui s'éloigne paraît plus petit que celui qui s'approche.
        let points = m.sprites(VUE.0, VUE.1);
        let (petit, grand) = points.iter().fold((f32::MAX, 0f32), |(p, g), s| (p.min(s.rayon), g.max(s.rayon)));
        assert!(grand > petit * 1.2, "{petit} {grand}");

        // On ne passe pas derrière la page, et « de face » la remet à plat.
        m.pivoter(10.0, 10.0);
        assert_eq!((m.lacet, m.tangage), (ANGLE_MAX, ANGLE_MAX));
        assert!(m.sprites(VUE.0, VUE.1).len() <= POINTS_MAX);
        m.de_face();
        assert_eq!((m.inclinaison(), m.lacet), (0.0, 0.0));
    }

    #[test]
    fn de_biais_ce_qui_est_lumineux_se_souleve() {
        // Moitié gauche noire, moitié droite blanche.
        let couleurs: Vec<u8> = (0..80 * 60).flat_map(|i| if i % 80 < 40 { [20, 20, 20, 255] } else { [255, 255, 255, 255] }).collect();
        let mut m = Mosaique::new(80, 60, couleurs, 1, VUE.0, VUE.1).unwrap();
        let d = distance(VUE.1);
        m.pivoter(0.0, -0.5);
        // Un point de la page sans relief, à la même place, serait dessiné ici :
        let (a_plat, _) = m.projeter([105.0, 5.0, 0.0], d).unwrap();
        let blanc = m.sprites(VUE.0, VUE.1).into_iter().filter(|s| s.couleur[0] > 0.9).min_by(|a, b| {
            let ecart = |s: &Sprite| (f64::from(s.x) * 300.0 - a_plat[0]).abs() + (f64::from(-s.y) * 300.0 - a_plat[1]).abs();
            ecart(a).total_cmp(&ecart(b))
        });
        let blanc = blanc.unwrap();
        // Le point blanc le plus proche n'est pas exactement là : il est soulevé.
        let decalage = (f64::from(blanc.x) * 300.0 - a_plat[0]).abs() + (f64::from(-blanc.y) * 300.0 - a_plat[1]).abs();
        assert!(decalage > 3.0, "{decalage}");
    }

    #[test]
    fn les_lettres_restent_lisses_en_se_morcelant() {
        // Une frontière nette entre noir et blanc : les points nés du morcellement font le
        // dégradé entre les deux, au lieu de dessiner des carrés.
        let couleurs: Vec<u8> = (0..80 * 60).flat_map(|i| if i % 80 < 40 { [0, 0, 0, 255] } else { [255, 255, 255, 255] }).collect();
        let m = Mosaique::new(80, 60, couleurs, 1, VUE.0, VUE.1).unwrap();
        let milieu = m.couleur_fondue(40.0, 30.0)[0];
        assert!((milieu - 0.5).abs() < 0.01, "à la frontière, la couleur est entre les deux : {milieu}");
        assert_eq!(m.couleur_fondue(39.5, 30.0)[0], 0.0, "au centre d'un pixel, c'est sa couleur exacte");
        assert_eq!(m.couleur_fondue(40.5, 30.0)[0], 1.0);
    }

    #[test]
    fn une_image_mal_decrite_est_refusee() {
        assert!(Mosaique::new(2, 2, vec![0; 15], 1, 10.0, 10.0).is_none());
        assert!(Mosaique::new(0, 2, vec![], 1, 10.0, 10.0).is_none());
    }
}
