//! Graines : une graine est un nombre de 64 bits qui engendre un monde entier.
//! La même graine redonne toujours le même monde (ADR-008). Tout est en arithmétique
//! entière, donc identique sur le PC, dans le navigateur et sur un téléphone.

/// Brouille un nombre (finalisation de SplitMix64) : deux entrées proches donnent
/// deux sorties sans rapport apparent.
pub fn mix_bits(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Graine du `index`-ième point né du morcellement d'un point de graine `parent`.
pub fn child_seed(parent: u64, index: u32) -> u64 {
    mix_bits(parent ^ mix_bits(0x5EED_0000_0000_0000 | u64::from(index)))
}

/// Suite de nombres pseudo-aléatoires reproductible, dérivée d'une graine.
pub struct Generator(u64);

impl Generator {
    pub fn new(seed: u64) -> Self {
        Self(mix_bits(seed))
    }

    pub fn u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix_bits(self.0)
    }

    /// Nombre dans [0, 1), construit sur 24 bits : la conversion en f32 est exacte.
    pub fn unit(&mut self) -> f32 {
        ((self.u64() >> 40) as u32) as f32 / 16_777_216.0
    }

    pub fn between(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unit()
    }

    /// Entier dans [a, b], bornes comprises.
    pub fn integer(&mut self, a: u32, b: u32) -> u32 {
        a + (self.u64() % u64::from(b - a + 1)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_sequence() {
        let a: Vec<u64> = (0..8).map(|_| 0).scan(Generator::new(42), |g, _| Some(g.u64())).collect();
        let b: Vec<u64> = (0..8).map(|_| 0).scan(Generator::new(42), |g, _| Some(g.u64())).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn the_children_of_a_point_have_distinct_seeds() {
        let seeds: std::collections::HashSet<u64> = (0..64).map(|i| child_seed(1, i)).collect();
        assert_eq!(seeds.len(), 64);
    }

    #[test]
    fn values_are_frozen() {
        // Valeurs calculées une fois et figées. Si ce test casse, tous les mondes déjà
        // partagés changent : c'est voulu qu'il soit strict. (Revue Codex : la première
        // version de ce test était une tautologie.)
        assert_eq!(mix_bits(0), 0xE220_A839_7B1D_CDAF);
        assert_eq!(child_seed(1, 0), 0x0033_4C53_F388_50D4);
        assert_eq!(child_seed(1, 1), 0x4A7C_B667_230D_5971);
        assert_eq!(child_seed(42, 5), 0x127A_24A8_33D5_1395);
        let mut g = Generator::new(7);
        assert_eq!(g.u64(), 0xA653_05FD_338E_C8FE);
        assert_eq!(g.u64(), 0x8CA3_CBB6_CA63_129B);
        let u = Generator::new(1).unit();
        assert!((0.0..1.0).contains(&u));
    }
}
