//! Vérification des styles (ADR-017). Un style s'écrit comme en CSS, `.card { color: gray; }`,
//! mais rien n'est toléré en silence : un réglage inconnu, une valeur mal écrite, un style
//! défini deux fois ou jamais défini sont refusés avec leur ligne.

use crate::blocs::{bloc_inconnu, BLOCS};
use crate::holo::{Bloc, Cible, Erreur, Programme, Reglage, Valeur};

/// Ce qu'un réglage accepte comme valeur.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Forme {
    Couleur,
    /// Une taille : `0`, `16px` ou `50%`.
    Taille,
    /// D'une à quatre tailles, comme `padding: 8px 16px`.
    Tailles,
    /// Un mot parmi une liste.
    Mot(&'static [&'static str]),
    /// Un nombre entre 0 et 1.
    Fraction,
    /// `1px solid gray`.
    Bordure,
    /// Un ou plusieurs noms de police, séparés par des virgules.
    Police,
}

/// Les réglages connus : l'apparence, avec les noms du CSS de base.
const REGLAGES: &[(&str, Forme)] = &[
    ("color", Forme::Couleur),
    ("background", Forme::Couleur),
    ("font-size", Forme::Taille),
    ("font-weight", Forme::Mot(&["normal", "bold"])),
    ("font-style", Forme::Mot(&["normal", "italic"])),
    ("font-family", Forme::Police),
    ("text-align", Forme::Mot(&["left", "center", "right"])),
    ("border", Forme::Bordure),
    ("border-radius", Forme::Taille),
    ("padding", Forme::Tailles),
    ("margin", Forme::Tailles),
    ("width", Forme::Taille),
    ("height", Forme::Taille),
    ("max-width", Forme::Taille),
    ("opacity", Forme::Fraction),
];

/// La disposition ne se règle pas dans un style : elle vient des blocs (ADR-017, règle 3).
const DISPOSITION: &[&str] = &[
    "display", "position", "float", "clear", "top", "left", "right", "bottom", "z-index", "flex", "flex-direction", "flex-wrap",
    "justify-content", "align-items", "align-self", "gap", "grid", "grid-template-columns", "grid-template-rows", "order",
];

const COULEURS: &[&str] = &[
    "transparent", "black", "white", "gray", "silver", "red", "maroon", "orange", "gold", "yellow", "olive", "green", "lime",
    "teal", "aqua", "cyan", "blue", "navy", "purple", "magenta", "fuchsia", "pink", "brown", "beige", "ivory", "indigo", "violet",
    "turquoise", "salmon", "coral", "crimson", "khaki", "lavender", "tan",
];

/// Vérifie les règles de style d'un fichier et les noms de style posés sur les blocs.
pub fn verifier_styles(programme: &Programme) -> Result<(), Erreur> {
    for (i, regle) in programme.styles.iter().enumerate() {
        if let Cible::Type(nom) = &regle.cible {
            if !BLOCS.contains(&nom.as_str()) {
                let majuscule = majuscule(nom);
                let message = if BLOCS.contains(&majuscule.as_str()) {
                    format!("« {nom} » : un type de bloc commence par une majuscule, écris « {majuscule} {{ … }} » (ADR-020)")
                } else {
                    bloc_inconnu(nom)
                };
                return Err(Erreur { message, pos: regle.pos });
            }
        }
        if programme.styles[..i].iter().any(|autre| autre.cible == regle.cible) {
            return Err(Erreur {
                message: format!("le style « {} » est défini deux fois : rassemble ses réglages au même endroit", regle.cible),
                pos: regle.pos,
            });
        }
        for (j, reglage) in regle.reglages.iter().enumerate() {
            if regle.reglages[..j].iter().any(|autre| autre.nom == reglage.nom) {
                return Err(Erreur { message: format!("le réglage « {} » est donné deux fois dans « {} »", reglage.nom, regle.cible), pos: reglage.pos });
            }
            verifier_reglage(reglage)?;
        }
    }
    noms_poses(&programme.racine, programme)
}

/// Chaque nom de style posé sur un bloc (`P.card(...)`) doit être défini.
fn noms_poses(bloc: &Bloc, programme: &Programme) -> Result<(), Erreur> {
    if let Some(nom) = &bloc.style {
        if !programme.styles.iter().any(|r| r.cible == Cible::Nom(nom.clone())) {
            return Err(Erreur {
                message: format!("le style « .{nom} » n'est défini nulle part : écris « .{nom} {{ … }} » après le bloc racine"),
                pos: bloc.pos,
            });
        }
    }
    fn visiter(valeur: &Valeur, programme: &Programme) -> Result<(), Erreur> {
        match valeur {
            Valeur::Bloc(bloc) => noms_poses(bloc, programme),
            Valeur::Liste(elements) => elements.iter().try_for_each(|e| visiter(e, programme)),
            _ => Ok(()),
        }
    }
    bloc.arguments.iter().try_for_each(|a| visiter(&a.valeur, programme))
}

fn verifier_reglage(reglage: &Reglage) -> Result<(), Erreur> {
    let refus = |message: String| Err(Erreur { message, pos: reglage.pos });
    let nom = reglage.nom.as_str();
    if DISPOSITION.contains(&nom) {
        return refus(format!(
            "« {nom} » règle la disposition, pas l'apparence : un style ne dit que l'apparence, la disposition vient des blocs (ADR-017)"
        ));
    }
    if nom == "background-color" {
        return refus("« background-color » s'écrit « background » : une seule écriture par réglage".into());
    }
    let Some((_, forme)) = REGLAGES.iter().find(|(connu, _)| *connu == nom) else {
        let connus: Vec<&str> = REGLAGES.iter().map(|(n, _)| *n).collect();
        return refus(format!("réglage inconnu « {nom} » ; réglages possibles : {}", connus.join(", ")));
    };
    let valeur = reglage.valeur.as_str();
    let mots: Vec<&str> = valeur.split_whitespace().collect();
    let correct = match forme {
        Forme::Couleur => mots.len() == 1 && est_couleur(valeur),
        Forme::Taille => mots.len() == 1 && est_taille(valeur),
        Forme::Tailles => (1..=4).contains(&mots.len()) && mots.iter().all(|m| est_taille(m)),
        Forme::Mot(possibles) => possibles.contains(&valeur),
        Forme::Fraction => valeur.parse::<f64>().is_ok_and(|v| (0.0..=1.0).contains(&v)),
        Forme::Bordure => mots.len() == 3 && est_taille(mots[0]) && ["solid", "dashed", "dotted"].contains(&mots[1]) && est_couleur(mots[2]),
        Forme::Police => valeur.split(',').all(|police| {
            let police = police.trim().trim_matches('"');
            !police.is_empty() && police.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-')
        }),
    };
    if correct {
        return Ok(());
    }
    let attendu = match forme {
        Forme::Couleur => "une couleur, comme « gray » ou « #E9B44C »".to_string(),
        Forme::Taille => "une taille, comme « 16px » ou « 50% »".to_string(),
        Forme::Tailles => "une à quatre tailles, comme « 8px 16px »".to_string(),
        Forme::Mot(possibles) => format!("l'un de ces mots : {}", possibles.join(", ")),
        Forme::Fraction => "un nombre entre 0 et 1".to_string(),
        Forme::Bordure => "une épaisseur, un trait et une couleur, comme « 1px solid gray »".to_string(),
        Forme::Police => "un ou plusieurs noms de police, séparés par des virgules".to_string(),
    };
    refus(format!("« {nom}: {valeur} » : ce réglage attend {attendu}"))
}

pub(crate) fn est_couleur(valeur: &str) -> bool {
    match valeur.strip_prefix('#') {
        Some(hexa) => [3, 6, 8].contains(&hexa.len()) && hexa.bytes().all(|c| c.is_ascii_hexdigit()),
        None => COULEURS.contains(&valeur),
    }
}

fn est_taille(valeur: &str) -> bool {
    if valeur == "0" {
        return true;
    }
    let nombre = valeur.strip_suffix("px").or_else(|| valeur.strip_suffix('%'));
    nombre.is_some_and(|n| !n.is_empty() && !n.starts_with('-') && n.parse::<f64>().is_ok_and(f64::is_finite))
}

fn majuscule(nom: &str) -> String {
    let mut lettres = nom.chars();
    lettres.next().map(|c| format!("{}{}", c.to_ascii_uppercase(), lettres.as_str())).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::lire;

    fn verifier(src: &str) -> Result<(), Erreur> {
        verifier_styles(&lire(src)?)
    }

    fn page(styles: &str) -> String {
        format!("Page(children: [ P.card(\"x\") ])\n.card {{ color: gray; }}\n{styles}")
    }

    #[test]
    fn la_boutique_de_la_suite_est_acceptee() {
        let source = include_str!("../../experiments/conformite-v0.1/cas/valides/06-boutique-avec-styles.holo");
        let programme = lire(source).unwrap();
        verifier_styles(&programme).unwrap();
        crate::blocs::verifier_blocs(&programme).unwrap();
        assert_eq!(programme.styles.len(), 5);
        assert_eq!(programme.styles[0].cible, Cible::Type("Page".into()));
        assert_eq!(programme.styles[3].cible, Cible::Nom("card".into()));
        assert_eq!(programme.styles[3].reglages[0].nom, "background");
        assert_eq!(programme.styles[3].reglages[0].valeur, "#1a1a2e");
    }

    #[test]
    fn la_boutique_comparee_emploie_tout_le_vocabulaire() {
        let source = include_str!("../../exemples/boutique-comparee/boutique.holo");
        let programme = lire(source).unwrap();
        crate::blocs::verifier_blocs(&programme).unwrap();
        verifier_styles(&programme).unwrap();
        // Le jeu complète la boutique : à eux deux, ils emploient tous les mots du langage.
        let jeu = include_str!("../../exemples/jeu/attraper.holo");
        crate::verifier_page(jeu).unwrap();
        let second = include_str!("../../exemples/jeu/panier.holo");
        crate::verifier_page(second).unwrap();
        // Un site de deux pages, avec un morceau importé.
        let (accueil, commun) = (include_str!("../../exemples/site/accueil.holo"), include_str!("../../exemples/site/commun.holo"));
        crate::verifier_page(&format!("{accueil}{}commun.holo{}{commun}", crate::holo::FICHIER_SUIVANT, crate::holo::SEPARE_LE_NOM)).unwrap();
        let contact = include_str!("../../exemples/site/contact.holo");
        crate::verifier_page(&format!("{contact}{}commun.holo{}{commun}", crate::holo::FICHIER_SUIVANT, crate::holo::SEPARE_LE_NOM)).unwrap();
        let donnees = include_str!("../../exemples/lecons/27-donnees.holo");
        crate::verifier_page(donnees).unwrap();
        let source = &format!("{source}\n{jeu}\n{second}\n{accueil}\n{commun}\n{donnees}");
        for bloc in crate::blocs::BLOCS {
            assert!(source.contains(&format!("{bloc}(")) || source.contains(&format!("{bloc}.")), "le bloc « {bloc} » manque dans l'exemple");
        }
        for (reglage, _) in REGLAGES {
            assert!(source.contains(&format!("{reglage}:")), "le réglage « {reglage} » manque dans l'exemple");
        }
        for mot in ["name:", "title:", "seed:", "brightness:", "fragments:", "children:", "inside:", "rules:", "effect:", "budget:", "weight:", "source:", "text:", "color:", "palette:", ".tap", ".enter", ".leave", "state:", "prices:", "{count}", "{total}", ".add(", ".sub(", ".set(", "gap:", "align:", "columns:", "alt:", "is:", "over:", "by:", ".random(", "x:", "y:", "keep:", "value:", "label:", "max:", "Key.left", "meets:", "drag:", "data:", "from:", ".play", "form:"] {
            assert!(source.contains(mot), "« {mot} » manque dans l'exemple");
        }
    }

    #[test]
    fn refuse_ce_que_la_suite_refuse_a_la_bonne_ligne() {
        let cas = [
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E12-reglage-inconnu.holo"), 6, "réglage inconnu « colour »"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E13-disposition-dans-un-style.holo"), 7, "la disposition vient des blocs"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E14-style-non-defini.holo"), 5, "n'est défini nulle part"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E15-point-virgule-manquant.holo"), 6, "« ; » manquant"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E16-selecteur-compose.holo"), 6, "rien d'autre"),
        ];
        for (source, ligne, message) in cas {
            let erreur = verifier(source).unwrap_err();
            assert_eq!(erreur.pos.ligne, ligne, "{erreur}");
            assert!(erreur.message.contains(message), "{erreur}");
        }
    }

    #[test]
    fn rien_n_est_tolere_en_silence() {
        assert!(verifier(&page("P { color: grey; }")).unwrap_err().message.contains("une couleur"));
        assert!(verifier(&page("P { font-size: 16; }")).unwrap_err().message.contains("une taille"));
        assert!(verifier(&page("P { font-size: 16em; }")).unwrap_err().message.contains("une taille"));
        assert!(verifier(&page("P { color: gray; color: red; }")).unwrap_err().message.contains("deux fois"));
        assert!(verifier(&page(".card { color: red; }")).unwrap_err().message.contains("défini deux fois"));
        assert!(verifier(&page("p { color: gray; }")).unwrap_err().message.contains("écris « P { … } »"));
        assert!(verifier(&page("Div { color: gray; }")).unwrap_err().message.contains("bloc inconnu"));
        assert!(verifier(&page("P { background-color: red; }")).unwrap_err().message.contains("s'écrit « background »"));
        assert!(verifier(&page("P { opacity: 2; }")).unwrap_err().message.contains("entre 0 et 1"));
        assert!(verifier(&page("P { border: 1px gray; }")).unwrap_err().message.contains("1px solid gray"));
    }

    #[test]
    fn accepte_le_css_de_base() {
        verifier(&page(
            "P { font-size: 16px; font-weight: bold; font-family: Georgia, \"Times New Roman\"; text-align: center }\n\
             H1 { color: #E9B44C; margin: 0 0 8px 0; }\n\
             Button { border: 1px solid gold; border-radius: 8px; padding: 8px 16px; opacity: 0.9; background: transparent; }\n\
             Image { width: 100%; max-width: 320px; }",
        ))
        .unwrap();
        // Un fichier sans style reste valable, et un style qui ne sert pas n'est pas une erreur.
        verifier("Point(name: A, seed: 1)").unwrap();
        verifier("Point(name: A, seed: 1)\n.card { color: gray; }").unwrap();
    }

    #[test]
    fn un_bloc_ne_porte_qu_un_nom_de_style() {
        assert!(verifier("Page(children: [ P.card.big(\"x\") ])").unwrap_err().message.contains("un seul nom de style"));
        assert_eq!(lire("Page(children: [ P.card(\"x\") ])").unwrap().racine.arguments.len(), 1);
    }
}
