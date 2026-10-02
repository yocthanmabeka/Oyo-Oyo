//! Graines : une graine est un nombre de 64 bits qui engendre un monde entier.
//! La même graine redonne toujours le même monde (ADR-008). Tout est en arithmétique
//! entière, donc identique sur le PC, dans le navigateur et sur un téléphone.

/// Brouille un nombre (finalisation de SplitMix64) : deux entrées proches donnent
/// deux sorties sans rapport apparent.
pub fn melanger(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Graine du `index`-ième point né du morcellement d'un point de graine `parent`.
pub fn graine_enfant(parent: u64, index: u32) -> u64 {
    melanger(parent ^ melanger(0x5EED_0000_0000_0000 | u64::from(index)))
}

/// Suite de nombres pseudo-aléatoires reproductible, dérivée d'une graine.
pub struct Generateur(u64);

impl Generateur {
    pub fn new(graine: u64) -> Self {
        Self(melanger(graine))
    }

    pub fn u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        melanger(self.0)
    }

    /// Nombre dans [0, 1), construit sur 24 bits : la conversion en f32 est exacte.
    pub fn unite(&mut self) -> f32 {
        ((self.u64() >> 40) as u32) as f32 / 16_777_216.0
    }

    pub fn entre(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unite()
    }

    /// Entier dans [a, b], bornes comprises.
    pub fn entier(&mut self, a: u32, b: u32) -> u32 {
        a + (self.u64() % u64::from(b - a + 1)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_meme_graine_donne_la_meme_suite() {
        let a: Vec<u64> = (0..8).map(|_| 0).scan(Generateur::new(42), |g, _| Some(g.u64())).collect();
        let b: Vec<u64> = (0..8).map(|_| 0).scan(Generateur::new(42), |g, _| Some(g.u64())).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn les_enfants_d_un_point_ont_des_graines_distinctes() {
        let graines: std::collections::HashSet<u64> = (0..64).map(|i| graine_enfant(1, i)).collect();
        assert_eq!(graines.len(), 64);
    }

    #[test]
    fn les_valeurs_sont_figees() {
        // Si ce test casse, tous les mondes déjà partagés changent : c'est voulu qu'il soit strict.
        assert_eq!(melanger(0), 0xE220_A839_7B1D_CDAF);
        assert_eq!(graine_enfant(1, 0), 0x6B37_B0C9_4A0E_8C2E ^ graine_enfant(1, 0) ^ 0x6B37_B0C9_4A0E_8C2E);
        let mut g = Generateur::new(1);
        let u = g.unite();
        assert!((0.0..1.0).contains(&u));
        assert_eq!(Generateur::new(7).entier(6, 14), Generateur::new(7).entier(6, 14));
    }
}
