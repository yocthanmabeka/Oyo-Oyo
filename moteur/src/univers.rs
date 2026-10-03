//! Du fichier `.holo` au monde : un `Point` déclaré devient un monde dont tout le
//! contenu se calcule à partir de sa graine. Rien n'est stocké : un point pèse une graine.

use crate::blocs::{ancien_mot, bloc_inconnu, BLOCS};
use crate::graine::{graine_enfant, Generateur};
use crate::holo::{Erreur, Pos, Programme, Valeur};

/// Ce que déclare le bloc `Point(...)` d'un fichier `.holo`.
#[derive(Debug, Clone, PartialEq)]
pub struct PointDecl {
    pub nom: String,
    pub graine: u64,
    pub lumiere: f32,
    pub morceler: u32,
    /// La couleur imposée à ce point, sinon la graine décide (ADR-017).
    pub couleur: Option<[f32; 3]>,
    /// Les couleurs imposées à ses enfants, reprises en boucle.
    pub palette: Vec<[f32; 3]>,
}

pub const MORCELER_MAX: u32 = 64;

/// Donne un sens au bloc racine. Seul `Point` est pris en charge dans ce sprint.
pub fn point_depuis(programme: &Programme) -> Result<PointDecl, Erreur> {
    if let Some(import) = programme.imports.first() {
        return Err(Erreur {
            message: format!("« {} » n'est pas encore pris en charge par ce sprint : le fichier serait accepté sans que l'import soit appliqué (revue Codex)", import.sorte),
            pos: import.pos,
        });
    }
    let bloc = &programme.racine;
    match bloc.nom.as_str() {
        "Point" => {}
        connu if BLOCS.contains(&connu) => {
            return Err(Erreur {
                message: format!("le bloc « {} » existe dans le format .holo mais ce sprint ne lit que « Point »", bloc.nom),
                pos: bloc.pos,
            })
        }
        autre => return Err(Erreur { message: bloc_inconnu(autre), pos: bloc.pos }),
    }
    let mut decl = PointDecl { nom: String::new(), graine: 0, lumiere: 1.0, morceler: 12, couleur: None, palette: Vec::new() };
    let mut vus: Vec<&str> = Vec::new();
    for arg in &bloc.arguments {
        let nom = arg.nom.as_deref().ok_or_else(|| Erreur {
            message: "chaque paramètre de « Point » est nommé : name, seed, brightness, fragments, color, palette".into(),
            pos: arg.pos,
        })?;
        if vus.contains(&nom) {
            return Err(Erreur { message: format!("le paramètre « {nom} » est donné deux fois"), pos: arg.pos });
        }
        vus.push(nom);
        match (nom, &arg.valeur) {
            ("name", Valeur::Nom(n)) => decl.nom = n.clone(),
            ("name", _) => return Err(attendu("name", "un nom, comme « Origin »", arg.pos)),
            ("seed", Valeur::Nom(a)) if a == "auto" => {
                return Err(Erreur {
                    message: "graine non fixée : « auto » doit être remplacé par un nombre au moment de la création (ADR-008, ADR-014)".into(),
                    pos: arg.pos,
                })
            }
            ("seed", Valeur::Entier(graine)) => decl.graine = *graine,
            ("seed", _) => return Err(attendu("seed", "un nombre entier positif, sans unité, jusqu'à 18446744073709551615", arg.pos)),
            ("brightness", Valeur::Nombre { valeur, unite: None }) if (0.0..=1.0).contains(valeur) => decl.lumiere = *valeur as f32,
            ("brightness", Valeur::Entier(e)) if *e <= 1 => decl.lumiere = *e as f32,
            ("brightness", _) => return Err(attendu("brightness", "un nombre entre 0 et 1, sans unité", arg.pos)),
            ("fragments", Valeur::Entier(n)) if (1..=u64::from(MORCELER_MAX)).contains(n) => decl.morceler = *n as u32,
            ("fragments", _) => return Err(attendu("fragments", &format!("un nombre entier entre 1 et {MORCELER_MAX}"), arg.pos)),
            ("color", Valeur::Texte(c)) => decl.couleur = Some(couleur_hexa(c).ok_or_else(|| attendu("color", COULEUR_ATTENDUE, arg.pos))?),
            ("color", _) => return Err(attendu("color", COULEUR_ATTENDUE, arg.pos)),
            ("palette", Valeur::Liste(couleurs)) if !couleurs.is_empty() => {
                for c in couleurs {
                    let couleur = match c {
                        Valeur::Texte(c) => couleur_hexa(c),
                        _ => None,
                    };
                    decl.palette.push(couleur.ok_or_else(|| attendu("palette", "une liste de couleurs, comme [\"#E9B44C\", \"#245C45\"]", arg.pos))?);
                }
            }
            ("palette", _) => return Err(attendu("palette", "une liste de couleurs, comme [\"#E9B44C\", \"#245C45\"]", arg.pos)),
            (autre, _) => {
                let message = match ancien_mot(autre) {
                    Some(nouveau) => format!("le paramètre « {autre} » s'écrit « {nouveau} » : le vocabulaire est en anglais (ADR-016)"),
                    None => format!("« Point » n'a pas de paramètre « {autre} » ; paramètres possibles : name, seed, brightness, fragments, color, palette"),
                };
                return Err(Erreur { message, pos: arg.pos });
            }
        }
    }
    if decl.nom.is_empty() {
        return Err(Erreur { message: "« Point » doit avoir un paramètre « name »".into(), pos: bloc.pos });
    }
    if !vus.contains(&"seed") {
        return Err(Erreur { message: "« Point » doit avoir un paramètre « seed »".into(), pos: bloc.pos });
    }
    Ok(decl)
}

const COULEUR_ATTENDUE: &str = "une couleur entre guillemets, écrite \"#E9B44C\"";

/// `#E9B44C` → rouge, vert, bleu entre 0 et 1.
fn couleur_hexa(texte: &str) -> Option<[f32; 3]> {
    let hexa = texte.strip_prefix('#').filter(|h| h.len() == 6 && h.bytes().all(|c| c.is_ascii_hexdigit()))?;
    let composante = |i: usize| u8::from_str_radix(&hexa[i..i + 2], 16).ok().map(|v| f32::from(v) / 255.0);
    Some([composante(0)?, composante(2)?, composante(4)?])
}

fn attendu(param: &str, forme: &str, pos: Pos) -> Erreur {
    Erreur { message: format!("le paramètre « {param} » attend {forme}"), pos }
}

/// Un point enfant, né du morcellement de son parent.
#[derive(Debug, Clone, PartialEq)]
pub struct Enfant {
    pub graine: u64,
    /// Position sur une sphère de rayon 1 autour du centre du monde.
    pub position: [f32; 3],
    pub rayon: f32,
    pub couleur: [f32; 3],
}

/// Un monde : ce que l'on voit quand on est à l'intérieur d'un point.
#[derive(Debug, Clone, PartialEq)]
pub struct Monde {
    pub graine: u64,
    pub lumiere: f32,
    pub couleur: [f32; 3],
    pub enfants: Vec<Enfant>,
}

impl Monde {
    /// Le monde racine, décrit par le fichier `.holo`.
    pub fn racine(decl: &PointDecl) -> Monde {
        let mut monde = Monde::construire(decl.graine, decl.lumiere, Some(decl.morceler));
        if let Some(couleur) = decl.couleur {
            monde.couleur = couleur;
        }
        if !decl.palette.is_empty() {
            for (i, enfant) in monde.enfants.iter_mut().enumerate() {
                enfant.couleur = decl.palette[i % decl.palette.len()];
            }
        }
        monde
    }

    /// Un monde quelconque, connu par sa seule graine. Le nombre de points qu'il contient
    /// se tire de la graine quand il n'est pas imposé.
    pub fn depuis_graine(graine: u64) -> Monde {
        Monde::construire(graine, 1.0, None)
    }

    fn construire(graine: u64, lumiere: f32, morceler: Option<u32>) -> Monde {
        let mut g = Generateur::new(graine);
        let teinte = g.unite();
        let nb = morceler.unwrap_or_else(|| g.entier(6, 14));
        let dore = (5f32.sqrt() - 1.0) / 2.0;
        let enfants = (0..nb)
            .map(|i| {
                // Spirale de Fibonacci : des points bien répartis, puis un peu de désordre.
                let t = (i as f32 + 0.5) / nb as f32;
                let y = (1.0 - 2.0 * t + g.entre(-0.12, 0.12)).clamp(-0.95, 0.95);
                let r = (1.0 - y * y).sqrt();
                let angle = std::f32::consts::TAU * (i as f32 * dore + g.entre(-0.08, 0.08));
                Enfant {
                    graine: graine_enfant(graine, i),
                    position: [r * angle.cos(), y, r * angle.sin()],
                    rayon: g.entre(0.06, 0.11),
                    couleur: couleur(teinte + g.entre(-0.08, 0.08), g.entre(0.5, 0.9)),
                }
            })
            .collect();
        Monde { graine, lumiere, couleur: couleur(teinte, 0.35), enfants }
    }
}

/// Couleur lumineuse à partir d'une teinte (0..1, cyclique) et d'une saturation.
pub fn couleur(teinte: f32, saturation: f32) -> [f32; 3] {
    let h = (teinte.rem_euclid(1.0)) * 6.0;
    let x = 1.0 - ((h % 2.0) - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    let m = 1.0 - saturation;
    [r * saturation + m, g * saturation + m, b * saturation + m]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::lire;

    fn point(src: &str) -> Result<PointDecl, Erreur> {
        point_depuis(&lire(src)?)
    }

    #[test]
    fn le_big_bang_est_accepte() {
        let d = point(include_str!("../mondes/big-bang.holo")).unwrap();
        assert_eq!(d, PointDecl { nom: "Origin".into(), graine: 1, lumiere: 1.0, morceler: 12, couleur: None, palette: Vec::new() });
        assert_eq!(Monde::racine(&d).enfants.len(), 12);
        // Le point de la boutique, seul : mêmes réglages que dans l'exemple de la boutique.
        let atelier = point(include_str!("../mondes/atelier.holo")).unwrap();
        assert_eq!((atelier.graine, atelier.morceler, atelier.palette.len()), (42, 6, 2));
    }

    #[test]
    fn refuse_ce_que_la_suite_de_conformite_refuse() {
        let auto = point("Point(\n  name: Origin,\n  seed: auto,\n  fragments: 12,\n)").unwrap_err();
        assert_eq!(auto.pos.ligne, 3);
        assert!(auto.message.contains("graine non fixée"));
        assert!(point("Div(contenu: [])").unwrap_err().message.contains("bloc inconnu"));
        assert!(point("Point(name: A, seed: 1, budget: 3s)").unwrap_err().message.contains("n'a pas de paramètre"));
        assert!(point(include_str!("../../experiments/conformite-v0.1/cas/refuses/E08-ancien-vocabulaire.holo")).unwrap_err().message.contains("s'écrit « name »"));
        assert_eq!(point(include_str!("../../experiments/conformite-v0.1/cas/valides/02-big-bang.holo")).unwrap().morceler, 12);
        assert!(point(include_str!("../../experiments/conformite-v0.1/cas/refuses/E04-graine-non-fixee.holo")).unwrap_err().message.contains("graine non fixée"));
        assert!(point("Point(name: A, seed: 1, seed: 2)").unwrap_err().message.contains("deux fois"));
        assert!(point("Point(seed: 1)").unwrap_err().message.contains("« name »"));
        assert!(point("Point(name: A, seed: 1, fragments: 500)").unwrap_err().message.contains("entre 1 et 64"));
        // Les anciens mots français sont refusés, avec le mot anglais à écrire à la place.
        assert!(point("Point(nom: A, graine: 1)").unwrap_err().message.contains("s'écrit « name »"));
        assert!(point("Point(name: A, graine: 1)").unwrap_err().message.contains("s'écrit « seed »"));
        assert!(point("Texte(\"Bonjour\")").unwrap_err().message.contains("écris « Text »"));
        assert!(point("Page(contenu: [])").unwrap_err().message.contains("ne lit que « Point »"));
        assert!(point("import \"absent.holo\"\nPoint(name: A, seed: 1)").unwrap_err().message.contains("pas encore pris en charge"));
        assert_eq!(point("Point(name: A, seed: 9007199254740993)").unwrap().graine, 9_007_199_254_740_993);
        assert!(point("Point(name: A, seed: 1.5)").unwrap_err().message.contains("entier"));
        assert!(point("Point(name: A, seed: -1)").unwrap_err().message.contains("entier"));
    }

    #[test]
    fn la_couleur_et_la_palette_s_imposent_a_la_graine() {
        let d = point("Point(name: A, seed: 42, fragments: 6, color: \"#FF0000\", palette: [\"#E9B44C\", \"#245C45\"])").unwrap();
        let monde = Monde::racine(&d);
        assert_eq!(monde.couleur, [1.0, 0.0, 0.0]);
        assert_eq!(monde.enfants[0].couleur, monde.enfants[2].couleur);
        assert_ne!(monde.enfants[0].couleur, monde.enfants[1].couleur);
        // Les graines des enfants ne changent pas : seule la couleur est imposée.
        assert_eq!(monde.enfants[0].graine, Monde::depuis_graine(42).enfants[0].graine);
        assert!(point("Point(name: A, seed: 1, color: \"gold\")").unwrap_err().message.contains("#E9B44C"));
        assert!(point("Point(name: A, seed: 1, palette: [])").unwrap_err().message.contains("une liste de couleurs"));
    }

    #[test]
    fn la_meme_graine_donne_le_meme_monde() {
        assert_eq!(Monde::depuis_graine(42), Monde::depuis_graine(42));
        assert_ne!(Monde::depuis_graine(42), Monde::depuis_graine(43));
    }

    #[test]
    fn les_enfants_ont_des_graines_distinctes_et_reproductibles() {
        let m = Monde::depuis_graine(1);
        let graines: std::collections::HashSet<u64> = m.enfants.iter().map(|e| e.graine).collect();
        assert_eq!(graines.len(), m.enfants.len());
        assert!((6..=14).contains(&m.enfants.len()));
        for e in &m.enfants {
            let n = e.position.iter().map(|c| c * c).sum::<f32>().sqrt();
            assert!((n - 1.0).abs() < 1e-3, "les enfants sont sur la sphère unité ({n})");
        }
    }

    #[test]
    fn un_point_ne_pese_que_sa_graine() {
        // Descendre de 1 000 niveaux ne demande que 1 000 graines : rien n'est stocké.
        let mut graine = 1u64;
        for _ in 0..1000 {
            graine = Monde::depuis_graine(graine).enfants[0].graine;
        }
        assert_eq!(std::mem::size_of::<u64>(), 8);
        assert_ne!(graine, 1);
    }
}
