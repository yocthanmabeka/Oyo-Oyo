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

use crate::seed::{child_seed, mix_bits};
use crate::navigation::Sprite;
use crate::view::Settings;

// Les tailles, la grille, la profondeur, le relief et les limites du zoom viennent du fichier
// `.holo` (voir `vue.rs`) : `self.r`.
/// Le relief ne dépasse jamais cette hauteur à l'écran (en pixels) : en zoomant très profond,
/// il emporterait sinon les points hors de vue.
pub const RELIEF_MAX: f64 = 80.0;
/// On ne dessine jamais plus de points que cela, même vu de biais.
pub const POINTS_MAX: usize = 200_000;

pub struct Mosaic {
    width: u32,
    height: u32,
    /// Rouge, vert, bleu, opacité : quatre octets par pixel.
    colors: Vec<u8>,
    seed: u64,
    /// Le pixel de l'image qui se trouve au centre de l'écran.
    pub cx: f64,
    pub cy: f64,
    /// Pixels d'écran pour un pixel de l'image.
    pub scale: f64,
    min_scale: f64,
    max_scale: f64,
    /// L'échelle où l'on voit la page entière, telle qu'elle est : un point par pixel.
    pub rest_scale: f64,
    /// La couleur moyenne de la page : celle du point qu'elle devient quand on la réduit.
    average: [f32; 3],
    r: Settings,
    /// La page tournée autour de son axe vertical (lacet) et horizontal (tangage), en radians.
    pub yaw: f64,
    pub pitch: f64,
    /// Vrai : glisser fait tourner la page ; faux : glisser la déplace.
    pub rotate: bool,
}

/// Distance de l'œil à la page, en pixels d'écran. La même valeur sert à la page ordinaire,
/// pour qu'elle se superpose exactement aux points.
pub fn distance(view_h: f64) -> f64 {
    view_h * 1.6
}

impl Mosaic {
    /// `couleurs` contient `largeur × hauteur × 4` octets. L'image est montrée entière dans
    /// une vue de `vue_l × vue_h` pixels.
    pub fn new(width: u32, height: u32, colors: Vec<u8>, seed: u64, view_w: f64, view_h: f64, r: Settings) -> Option<Mosaic> {
        if width == 0 || height == 0 || colors.len() != width as usize * height as usize * 4 {
            return None;
        }
        let scale = (view_w / f64::from(width)).min(view_h / f64::from(height)).max(1e-6);
        // Les garde-fous du zoom. Vers le petit : la page entière, ou, si l'auteur le permet
        // (`Zoom(shrink: true)`), la page réduite à un seul pixel. Vers le grand : ce que
        // l'auteur a fixé, et jamais au-delà du dernier morcellement.
        let min_scale = if r.reduce { 1.0 / f64::from(width.max(height)) } else { scale };
        let max_scale = (scale * r.zoom_max).min(r.shatter_size * (r.side as f64).powi(r.levels as i32)).max(scale);
        let mut sum = [0f64; 3];
        for pixel in colors.chunks_exact(4) {
            for (s, c) in sum.iter_mut().zip(pixel) {
                *s += f64::from(*c);
            }
        }
        let average = sum.map(|s| (s / (f64::from(width) * f64::from(height)) / 255.0) as f32);
        // Une page sombre réduite à un point resterait invisible : on garde sa teinte, plus claire.
        let brightness = average[0].max(average[1]).max(average[2]);
        let average = if brightness > 0.0 && brightness < 0.6 { average.map(|c| c * 0.6 / brightness) } else { average };
        Some(Mosaic {
            width,
            height,
            colors,
            seed,
            cx: f64::from(width) / 2.0,
            cy: f64::from(height) / 2.0,
            scale,
            min_scale,
            max_scale,
            rest_scale: scale,
            average,
            r,
            yaw: 0.0,
            pitch: 0.0,
            rotate: false,
        })
    }

    /// Combien de fois chaque point s'est morcelé à cette échelle. 0 : un point par pixel.
    pub fn level(&self) -> u32 {
        let mut level = 0;
        let mut size = self.scale;
        while size >= self.r.shatter_size && level < self.r.levels {
            size /= self.r.side as f64;
            level += 1;
        }
        level
    }

    /// De 0 (on voit l'image ordinaire) à 1 (on voit les points).
    pub fn points_opacity(&self) -> f64 {
        ((self.scale - self.r.point_size) / self.r.point_size).clamp(0.0, 1.0)
    }

    /// De 0 (la page est de face, plate) à 1 (elle est de biais, le relief est entier).
    pub fn tilt(&self) -> f64 {
        ((self.yaw.abs() + self.pitch.abs()) / 0.35).clamp(0.0, 1.0)
    }

    /// Zoome en gardant sous le doigt le point de l'image qui s'y trouvait.
    pub fn zoom_in(&mut self, factor: f64, x: f64, y: f64, view_w: f64, view_h: f64) {
        let mut new_one = (self.scale * factor).clamp(self.min_scale, self.max_scale);
        // En passant par la page entière, on s'y arrête : c'est là qu'on retrouve le site normal.
        if (self.scale - self.rest_scale) * (new_one - self.rest_scale) < 0.0 {
            new_one = self.rest_scale;
        }
        let (dx, dy) = (x - view_w / 2.0, y - view_h / 2.0);
        // Où le doigt touche la page, en pixels d'écran comptés sur la page elle-même.
        let [ux, uy] = self.on_page(dx, dy, distance(view_h)).unwrap_or([dx, dy]);
        self.cx += ux / self.scale - ux / new_one;
        self.cy += uy / self.scale - uy / new_one;
        self.scale = new_one;
        self.clamp();
    }

    /// Déplace la vue de `dx`, `dy` pixels d'écran.
    pub fn translate(&mut self, dx: f64, dy: f64) {
        // Vue par derrière, la page est retournée : elle suit quand même le doigt.
        self.cx -= dx / self.scale * self.yaw.cos().signum();
        self.cy -= dy / self.scale * self.pitch.cos().signum();
        self.clamp();
    }

    /// Fait tourner la page : on la voit de biais.
    pub fn pivot(&mut self, yaw: f64, pitch: f64) {
        let limit = self.r.angle_max;
        // Un demi-tour permis ou davantage : la rotation est libre, on fait le tour de la page et
        // on la voit par derrière. Sinon, on s'arrête à l'angle fixé par l'auteur.
        let clamp = |angle: f64| {
            if limit >= std::f64::consts::PI {
                (angle + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
            } else {
                angle.clamp(-limit, limit)
            }
        };
        self.yaw = clamp(self.yaw + yaw);
        self.pitch = clamp(self.pitch + pitch);
    }

    /// La vitesse du zoom à la molette, fixée par le fichier (`Zoom(speed:)`).
    pub fn speed(&self) -> f64 {
        self.r.zoom_speed
    }

    /// Remet la page de face.
    pub fn front_facing(&mut self) {
        self.yaw = 0.0;
        self.pitch = 0.0;
    }

    fn clamp(&mut self) {
        self.cx = self.cx.clamp(0.0, f64::from(self.width));
        self.cy = self.cy.clamp(0.0, f64::from(self.height));
    }

    /// Place à l'écran un point de l'espace. `u` est compté en pixels d'écran depuis le centre
    /// de la vue : x vers la droite, y vers le bas, z vers l'œil. Rend la position à l'écran
    /// (depuis le centre) et l'agrandissement dû à la perspective ; rien si le point est
    /// derrière l'œil. Ce sont les calculs que fait le navigateur pour
    /// `perspective(d) rotateX(tangage) rotateY(lacet)`.
    pub fn project(&self, u: [f64; 3], d: f64) -> Option<([f64; 2], f64)> {
        let (sl, cl) = self.yaw.sin_cos();
        let (st, ct) = self.pitch.sin_cos();
        let x = u[0] * cl + u[2] * sl;
        let z = -u[0] * sl + u[2] * cl;
        let y = u[1] * ct - z * st;
        let z = u[1] * st + z * ct;
        let remainder = d - z;
        (remainder > d * 0.05).then(|| ([x * d / remainder, y * d / remainder], d / remainder))
    }

    /// L'inverse : quel endroit de la page (à plat, z = 0) se trouve sous ce point de l'écran.
    pub fn on_page(&self, sx: f64, sy: f64, d: f64) -> Option<[f64; 2]> {
        let (sl, cl) = self.yaw.sin_cos();
        let (st, ct) = self.pitch.sin_cos();
        let divert = |v: [f64; 3]| {
            let y = v[1] * ct + v[2] * st;
            let z = -v[1] * st + v[2] * ct;
            [v[0] * cl - z * sl, y, v[0] * sl + z * cl]
        };
        let eye = divert([0.0, 0.0, d]);
        let radius = divert([sx, sy, -d]);
        // De face, le regard descend vers la page ; par derrière, il y remonte. Dans les deux
        // cas, elle est devant l'œil. Vue par la tranche, on ne la touche pas.
        let t = -eye[2] / radius[2];
        (radius[2].abs() > 1e-9 && t > 0.0).then(|| [eye[0] + t * radius[0], eye[1] + t * radius[1]])
    }

    fn pixel(&self, x: usize, y: usize) -> [f32; 3] {
        let i = (y * self.width as usize + x) * 4;
        [f32::from(self.colors[i]) / 255.0, f32::from(self.colors[i + 1]) / 255.0, f32::from(self.colors[i + 2]) / 255.0]
    }

    /// La couleur de l'image à un endroit quelconque, fondue entre les quatre pixels voisins.
    /// C'est ce qui garde les lettres lisses quand leurs points se morcellent : sans cela,
    /// chaque pixel deviendrait un carré.
    fn faded_color(&self, x: f64, y: f64) -> [f32; 3] {
        let gx = (x - 0.5).clamp(0.0, f64::from(self.width - 1));
        let gy = (y - 0.5).clamp(0.0, f64::from(self.height - 1));
        let (x0, y0) = (gx.floor() as usize, gy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.width as usize - 1), (y0 + 1).min(self.height as usize - 1));
        let (tx, ty) = ((gx - x0 as f64) as f32, (gy - y0 as f64) as f32);
        let top = mix(self.pixel(x0, y0), self.pixel(x1, y0), tx);
        let bottom = mix(self.pixel(x0, y1), self.pixel(x1, y1), tx);
        mix(top, bottom, ty)
    }

    /// La graine du point posé sur le pixel (x, y) : il ne pèse rien d'autre.
    pub fn pixel_seed(&self, x: u64, y: u64) -> u64 {
        child_seed(mix_bits(self.seed ^ y), x as u32)
    }

    /// La couleur d'un point né du morcellement : celle de l'image à cet endroit, déplacée
    /// par sa graine. Les premiers morcellements la déplacent peu, pour que les lettres
    /// restent nettes ; plus on descend, plus chaque point prend sa couleur propre.
    /// Même un point noir contient des points qui luisent faiblement.
    fn vary(color: [f32; 3], seed: u64, force: f32) -> [f32; 3] {
        let draw = |offset: u32| ((seed >> offset) & 0xFFFF) as f32 / 65535.0 - 0.5;
        let glow = 1.0 + 2.0 * force * draw(0);
        [
            (color[0] * glow + draw(16) * 0.5 * force).clamp(0.0, 1.0),
            (color[1] * glow + draw(32) * 0.5 * force).clamp(0.0, 1.0),
            (color[2] * glow + draw(48) * 0.5 * force).clamp(0.0, 1.0),
        ]
    }

    /// Les points à dessiner pour une vue de `vue_l × vue_h` pixels. Seuls ceux qui sont à
    /// l'écran sont calculés : leur nombre ne dépend pas de la profondeur du zoom.
    pub fn sprites(&self, view_w: f64, view_h: f64) -> Vec<Sprite> {
        let d = distance(view_h);
        let half_screen = view_h / 2.0;
        // La page réduite à presque rien : elle n'est plus qu'un point, de sa couleur moyenne.
        let screen_side = f64::from(self.width.max(self.height)) * self.scale;
        if screen_side < 12.0 {
            let center = [(f64::from(self.width) / 2.0 - self.cx) * self.scale, (f64::from(self.height) / 2.0 - self.cy) * self.scale, 0.0];
            return match self.project(center, d) {
                Some(([x, y], _)) => vec![Sprite { x: (x / half_screen) as f32, y: (-y / half_screen) as f32, radius: (screen_side.max(5.0) / half_screen) as f32, color: [self.average[0], self.average[1], self.average[2], 1.0] }],
                None => Vec::new(),
            };
        }
        let opacity = self.points_opacity() as f32;
        if opacity <= 0.0 {
            return Vec::new();
        }
        let (side, shatter_threshold) = (self.r.side, self.r.shatter_size);
        let level = self.level();
        let per_pixel = side.pow(level); // cellules par côté dans un pixel de l'image
        let size = self.scale / per_pixel as f64; // taille d'une cellule à l'écran
        // Juste après un morcellement, les enfants ont encore la couleur de leur parent ;
        // elle se déplace vers la leur à mesure qu'ils grossissent.
        let progress = if level == 0 { 1.0 } else { ((size - shatter_threshold / side as f64) / (shatter_threshold / 2.0)).clamp(0.0, 1.0) as f32 };
        let tilt = self.tilt();

        // La partie de l'image qui est à l'écran : ce que couvrent les quatre coins de la vue.
        let corners = [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].map(|(sx, sy)| self.on_page(sx * view_w / 2.0, sy * view_h / 2.0, d));
        let (mut left, mut right, mut top, mut bottom) = (0.0, f64::from(self.width), 0.0, f64::from(self.height));
        if let [Some(a), Some(b), Some(c), Some(e)] = corners {
            // Une marge pour les points soulevés par le relief, qui débordent de leur place.
            let margin = (2.0 * RELIEF_MAX * tilt + size) / self.scale;
            let xs = [a[0], b[0], c[0], e[0]].map(|v| self.cx + v / self.scale);
            let ys = [a[1], b[1], c[1], e[1]].map(|v| self.cy + v / self.scale);
            left = xs.iter().copied().fold(f64::MAX, f64::min) - margin;
            right = xs.iter().copied().fold(f64::MIN, f64::max) + margin;
            top = ys.iter().copied().fold(f64::MAX, f64::min) - margin;
            bottom = ys.iter().copied().fold(f64::MIN, f64::max) + margin;
        }
        // Vue presque par la tranche, la page s'étend jusqu'à l'horizon : on ne calcule que les
        // points proches de l'endroit regardé. Les autres seraient minuscules.
        let (reach_w, reach_h) = (2.0 * view_w / self.scale, 2.0 * view_h / self.scale);
        left = left.max(self.cx - reach_w);
        right = right.min(self.cx + reach_w);
        top = top.max(self.cy - reach_h);
        bottom = bottom.min(self.cy + reach_h);
        let bound = |v: f64, max: u32| (v.clamp(0.0, f64::from(max)) * per_pixel as f64) as u64;
        let (x0, x1) = (bound(left, self.width), bound(right, self.width));
        let (y0, y1) = (bound(top, self.height), bound(bottom, self.height));
        let end_x = (u64::from(self.width) * per_pixel).saturating_sub(1);
        let end_y = (u64::from(self.height) * per_pixel).saturating_sub(1);

        let mut sprites = Vec::new();
        for cy in y0..=y1.min(end_y) {
            for cx in x0..=x1.min(end_x) {
                if sprites.len() >= POINTS_MAX {
                    return sprites;
                }
                let (px, py) = (cx / per_pixel, cy / per_pixel);
                // Où est la cellule dans l'image, en pixels de l'image.
                let (ix, iy) = ((cx as f64 + 0.5) / per_pixel as f64, (cy as f64 + 0.5) / per_pixel as f64);
                let background = if level == 0 { self.pixel(px as usize, py as usize) } else { self.faded_color(ix, iy) };
                let mut color = background;
                let mut seed = self.pixel_seed(px, py);
                let mut gap = 0.0; // de combien ce point s'écarte de la page, vers l'œil
                // On descend du pixel jusqu'à la cellule, un morcellement à la fois.
                for stage in (0..level).rev() {
                    let divisor = side.pow(stage);
                    let index = (cx / divisor % side) + side * (cy / divisor % side);
                    seed = child_seed(seed, index as u32);
                    let force = (0.05 * (level - stage) as f32).min(0.4);
                    let child = Self::vary(color, seed, force);
                    color = if stage == 0 { mix(color, child, progress) } else { child };
                    // Les enfants d'un point ne sont pas tous à la même hauteur.
                    gap += (((seed >> 20) & 0xFFFF) as f64 / 65535.0 - 0.5) * self.scale / (divisor * side) as f64 * 1.5;
                }
                if color[0].max(color[1]).max(color[2]) * opacity < 0.02 {
                    continue; // un point éteint n'est pas dessiné
                }
                // Le relief : ce qui est lumineux dans l'image se soulève, quand la page est de biais.
                let light = f64::from(background[0].max(background[1]).max(background[2]));
                let z = ((self.r.relief * self.scale).min(RELIEF_MAX) * light + gap.clamp(-RELIEF_MAX, RELIEF_MAX)) * tilt;
                let Some(([x, y], k)) = self.project([(ix - self.cx) * self.scale, (iy - self.cy) * self.scale, z], d) else { continue };
                let radius = size * 0.46 * k;
                if x.abs() > view_w / 2.0 + radius * 2.0 || y.abs() > view_h / 2.0 + radius * 2.0 {
                    continue;
                }
                sprites.push(Sprite {
                    x: (x / half_screen) as f32,
                    y: (-y / half_screen) as f32,
                    radius: (radius / half_screen) as f32,
                    color: [color[0], color[1], color[2], opacity],
                });
            }
        }
        sprites
    }
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: (f64, f64) = (800.0, 600.0);
    const POINT_THRESHOLD: f64 = 6.0;

    fn mosaic(width: u32, height: u32, colors: Vec<u8>) -> Option<Mosaic> {
        // Les essais tournent la page : la rotation est activée, comme par « Relief(tilt: 360deg) ».
        Mosaic::new(width, height, colors, 1, VIEW.0, VIEW.1, Settings { angle_max: std::f64::consts::TAU, ..Settings::default() })
    }

    /// Une image de 80 × 60 pixels, toute dorée sauf un pixel noir en haut à gauche.
    fn image() -> Mosaic {
        let mut colors = [233u8, 180, 76, 255].repeat(80 * 60);
        colors[..3].copy_from_slice(&[0, 0, 0]);
        mosaic(80, 60, colors).unwrap()
    }

    #[test]
    fn at_rest_the_image_suffices_then_each_pixel_becomes_a_point() {
        let m = mosaic(800, 600, vec![255; 800 * 600 * 4]).unwrap();
        assert_eq!(m.scale, 1.0, "un point fait exactement un pixel");
        assert!(m.sprites(VIEW.0, VIEW.1).is_empty(), "des points de la taille d'un pixel : l'image ordinaire suffit");

        // L'image de 80 × 60 remplit la vue à l'échelle 10 : un point par pixel, bien rangés.
        let m = image();
        assert_eq!((m.scale, m.level()), (10.0, 0));
        let points = m.sprites(VIEW.0, VIEW.1);
        assert_eq!(points.len(), 80 * 60 - 1, "un point par pixel ; le pixel noir est éteint");
        let step = points[1].x - points[0].x;
        assert!((step - 10.0 / 300.0).abs() < 1e-5, "les points sont espacés d'un pixel de l'image");
        assert!(points.iter().all(|p| (p.radius - points[0].radius).abs() < 1e-9), "de face, la page est plate : tous de la même taille");
    }

    #[test]
    fn a_growing_point_shatters_into_a_grid() {
        let mut m = image();
        m.zoom_in(4.0, 400.0, 300.0, VIEW.0, VIEW.1);
        assert_eq!((m.scale, m.level()), (40.0, 1), "à 40 pixels, chaque point se morcelle en 4 × 4");
        let points = m.sprites(VIEW.0, VIEW.1);
        // 800 / 10 = 80 cellules par ligne, 60 par colonne, plus celles qui dépassent au bord.
        assert!((80 * 60..=82 * 62).contains(&points.len()), "{}", points.len());
        m.zoom_in(4.0, 400.0, 300.0, VIEW.0, VIEW.1);
        assert_eq!(m.level(), 2);
    }

    #[test]
    fn the_number_of_points_does_not_grow_with_depth() {
        let mut m = image();
        let ceiling = ((VIEW.0 / POINT_THRESHOLD + 2.0) * (VIEW.1 / POINT_THRESHOLD + 2.0)) as usize;
        for _ in 0..60 {
            m.zoom_in(1.7, 311.0, 207.0, VIEW.0, VIEW.1);
            assert!(m.sprites(VIEW.0, VIEW.1).len() <= ceiling, "niveau {}", m.level());
        }
        assert!(m.level() >= 15, "on est descendu très profond : niveau {}", m.level());
        assert!(!m.sprites(VIEW.0, VIEW.1).is_empty(), "et il y a toujours des points à voir");
    }

    #[test]
    fn zoom_keeps_under_the_finger_what_was_there() {
        // De face, puis la page de biais : dans les deux cas le doigt, à (200, 150), ne lâche pas son pixel.
        for (yaw, pitch) in [(0.0, 0.0), (0.5, -0.3)] {
            let mut m = image();
            m.pivot(yaw, pitch);
            let aimed = |m: &Mosaic| {
                let [ux, uy] = m.on_page(200.0 - 400.0, 150.0 - 300.0, distance(VIEW.1)).unwrap();
                (m.cx + ux / m.scale, m.cy + uy / m.scale)
            };
            let before = aimed(&m);
            m.zoom_in(3.0, 200.0, 150.0, VIEW.0, VIEW.1);
            let after = aimed(&m);
            assert!((before.0 - after.0).abs() < 1e-9 && (before.1 - after.1).abs() < 1e-9);
        }
        // On ne dézoome pas en deçà de l'image entière.
        let mut m = image();
        m.zoom_in(0.001, 0.0, 0.0, VIEW.0, VIEW.1);
        assert_eq!(m.scale, 10.0);
    }

    #[test]
    fn the_same_image_gives_the_same_points() {
        let (mut a, mut b) = (image(), image());
        for m in [&mut a, &mut b] {
            m.zoom_in(37.0, 123.0, 456.0, VIEW.0, VIEW.1);
            m.translate(-14.0, 9.0);
            m.pivot(0.4, 0.2);
        }
        assert_eq!(a.sprites(VIEW.0, VIEW.1), b.sprites(VIEW.0, VIEW.1));
        assert_eq!(a.pixel_seed(3, 4), b.pixel_seed(3, 4));
        assert_ne!(a.pixel_seed(3, 4), a.pixel_seed(4, 3), "chaque pixel a sa graine");
    }

    #[test]
    fn even_black_contains_points() {
        let black = mosaic(80, 60, [0u8, 0, 0, 255].repeat(80 * 60));
        let mut m = black.unwrap();
        assert!(m.sprites(VIEW.0, VIEW.1).is_empty(), "une page noire : aucun point allumé");
        for _ in 0..14 {
            m.zoom_in(2.0, 400.0, 300.0, VIEW.0, VIEW.1);
        }
        assert!(m.level() >= 4);
        assert!(!m.sprites(VIEW.0, VIEW.1).is_empty(), "le noir, vu de près, contient des points qui luisent");
    }

    #[test]
    fn tilted_the_page_has_depth() {
        let mut m = image();
        let d = distance(VIEW.1);
        // Aller et retour : un endroit de la page, vu à l'écran, se retrouve sur la page.
        m.pivot(0.6, -0.4);
        let (screen, _) = m.project([120.0, -80.0, 0.0], d).unwrap();
        let back = m.on_page(screen[0], screen[1], d).unwrap();
        assert!((back[0] - 120.0).abs() < 1e-6 && (back[1] + 80.0).abs() < 1e-6);

        // Le côté qui s'éloigne paraît plus petit que celui qui s'approche.
        let points = m.sprites(VIEW.0, VIEW.1);
        let (small, big) = points.iter().fold((f32::MAX, 0f32), |(p, g), s| (p.min(s.radius), g.max(s.radius)));
        assert!(big > small * 1.2, "{small} {big}");

        m.front_facing();
        assert_eq!((m.tilt(), m.yaw), (0.0, 0.0));
    }

    #[test]
    fn wraps_around_the_page_unless_the_author_sets_a_limit() {
        // Sans rien écrire, la page ne tourne pas : un site reste un site.
        let mut m = Mosaic::new(80, 60, [233u8, 180, 76, 255].repeat(80 * 60), 1, VIEW.0, VIEW.1, Settings::default()).unwrap();
        m.pivot(1.0, 1.0);
        assert_eq!((m.yaw, m.pitch), (0.0, 0.0));
        // « Relief(tilt: 360deg) » : la rotation est libre ; un demi-tour, et l'on voit la page par derrière.
        let mut m = image();
        m.pivot(std::f64::consts::PI * 0.75, 0.0);
        assert!((m.yaw - std::f64::consts::PI * 0.75).abs() < 1e-12, "on a dépassé le quart de tour");
        m.pivot(std::f64::consts::PI * 0.5, 0.0);
        assert!((m.yaw + std::f64::consts::PI * 0.75).abs() < 1e-9, "après un tour, l'angle revient : {}", m.yaw);
        // Par derrière, on pointe toujours la page, et elle suit le doigt.
        m.front_facing();
        m.pivot(std::f64::consts::PI, 0.0);
        let [ux, uy] = m.on_page(100.0, 50.0, distance(VIEW.1)).expect("la page est devant l'œil");
        let ([x, y], _) = m.project([ux, uy, 0.0], distance(VIEW.1)).unwrap();
        assert!((x - 100.0).abs() < 1e-6 && (y - 50.0).abs() < 1e-6, "{x} {y}");
        assert!(ux < 0.0, "par derrière, la droite de l'écran est la gauche de la page");
        let before = m.cx;
        m.translate(-10.0, 0.0);
        assert!(m.cx < before, "glisser vers la gauche amène ce qui était à gauche de l'écran");
        // Par la tranche, le moteur ne s'affole pas : il dessine peu, et ne dépasse jamais sa limite.
        for quarter in [0.5, 0.49, 0.51, 1.0] {
            m.front_facing();
            m.pivot(std::f64::consts::PI * quarter, 0.2);
            assert!(m.sprites(VIEW.0, VIEW.1).len() <= POINTS_MAX, "{quarter}");
        }
        // « Relief(tilt: 52deg) » : l'auteur arrête la rotation à 52 degrés.
        let limit = 52f64.to_radians();
        let mut m = Mosaic::new(80, 60, [233u8, 180, 76, 255].repeat(80 * 60), 1, VIEW.0, VIEW.1, Settings { angle_max: limit, ..Settings::default() }).unwrap();
        m.pivot(10.0, -10.0);
        assert_eq!((m.yaw, m.pitch), (limit, -limit));
    }

    #[test]
    fn tilted_what_is_bright_rises() {
        // Moitié gauche noire, moitié droite blanche.
        let colors: Vec<u8> = (0..80 * 60).flat_map(|i| if i % 80 < 40 { [20, 20, 20, 255] } else { [255, 255, 255, 255] }).collect();
        let mut m = mosaic(80, 60, colors).unwrap();
        let d = distance(VIEW.1);
        m.pivot(0.0, -0.5);
        // Un point de la page sans relief, à la même place, serait dessiné ici :
        let (a_plat, _) = m.project([105.0, 5.0, 0.0], d).unwrap();
        let white = m.sprites(VIEW.0, VIEW.1).into_iter().filter(|s| s.color[0] > 0.9).min_by(|a, b| {
            let gap = |s: &Sprite| (f64::from(s.x) * 300.0 - a_plat[0]).abs() + (f64::from(-s.y) * 300.0 - a_plat[1]).abs();
            gap(a).total_cmp(&gap(b))
        });
        let white = white.unwrap();
        // Le point blanc le plus proche n'est pas exactement là : il est soulevé.
        let offset = (f64::from(white.x) * 300.0 - a_plat[0]).abs() + (f64::from(-white.y) * 300.0 - a_plat[1]).abs();
        assert!(offset > 3.0, "{offset}");
    }

    #[test]
    fn letters_stay_smooth_while_shattering() {
        // Une frontière nette entre noir et blanc : les points nés du morcellement font le
        // dégradé entre les deux, au lieu de dessiner des carrés.
        let colors: Vec<u8> = (0..80 * 60).flat_map(|i| if i % 80 < 40 { [0, 0, 0, 255] } else { [255, 255, 255, 255] }).collect();
        let m = mosaic(80, 60, colors).unwrap();
        let middle = m.faded_color(40.0, 30.0)[0];
        assert!((middle - 0.5).abs() < 0.01, "à la frontière, la couleur est entre les deux : {middle}");
        assert_eq!(m.faded_color(39.5, 30.0)[0], 0.0, "au centre d'un pixel, c'est sa couleur exacte");
        assert_eq!(m.faded_color(40.5, 30.0)[0], 1.0);
    }

    #[test]
    fn a_badly_described_image_is_refused() {
        assert!(mosaic(2, 2, vec![0; 15]).is_none());
        assert!(mosaic(0, 2, vec![]).is_none());
    }

    #[test]
    fn zoom_guardrails_come_from_the_file() {
        let image = |r: Settings| Mosaic::new(80, 60, [233u8, 180, 76, 255].repeat(80 * 60), 1, VIEW.0, VIEW.1, r).unwrap();
        // « Zoom(max: 50) » : on ne grossit pas la page plus de cinquante fois.
        let mut m = image(Settings { zoom_max: 50.0, ..Settings::default() });
        for _ in 0..40 {
            m.zoom_in(3.0, 400.0, 300.0, VIEW.0, VIEW.1);
        }
        assert_eq!(m.scale, 10.0 * 50.0);
        // Sans « shrink », on ne dézoome pas en deçà de la page entière.
        for _ in 0..40 {
            m.zoom_in(0.3, 400.0, 300.0, VIEW.0, VIEW.1);
        }
        assert_eq!(m.scale, m.rest_scale);

        // « Zoom(shrink: true) » : la page se réduit jusqu'à un pixel, et n'est plus qu'un point.
        let mut m = image(Settings { reduce: true, ..Settings::default() });
        for _ in 0..40 {
            m.zoom_in(0.3, 400.0, 300.0, VIEW.0, VIEW.1);
        }
        assert_eq!(m.scale, 1.0 / 80.0, "la page fait un pixel de large");
        let points = m.sprites(VIEW.0, VIEW.1);
        assert_eq!(points.len(), 1, "la page entière est devenue un point");
        assert!(points[0].color[0] > 0.9 && points[0].color[2] < 0.35, "de sa couleur moyenne");
        // En remontant, on s'arrête sur la page entière : c'est là qu'on retrouve le site.
        for _ in 0..40 {
            m.zoom_in(1.9, 400.0, 300.0, VIEW.0, VIEW.1);
            if m.scale == m.rest_scale {
                break;
            }
        }
        assert_eq!(m.scale, m.rest_scale);

        // « Points(divisions: 2, levels: 3) » : trois morcellements, en grilles de 2 × 2, pas un de plus.
        let mut m = image(Settings { side: 2, levels: 3, ..Settings::default() });
        for _ in 0..40 {
            m.zoom_in(3.0, 400.0, 300.0, VIEW.0, VIEW.1);
        }
        assert_eq!((m.level(), m.scale), (3, 40.0 * 8.0));
    }
}
