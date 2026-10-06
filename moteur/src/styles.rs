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
    /// Le fond : une couleur, un dégradé (`linear-gradient(…)`, `radial-gradient(…)`), ou une
    /// image rangée à côté du fichier (`url("fond.jpg")`), qui couvre toujours le bloc (ADR-041).
    Fond,
    /// Un nombre sans unité, entre deux bornes : `line-height: 1.5`, `scale: 1.1`.
    Nombre(f64, f64),
    /// Un écart en pixels, qui peut être négatif : `letter-spacing: -0.5px`.
    Ecart,
    /// Une à trois ombres, séparées par des virgules : `0 4px 12px #00000066` ; ou `none`.
    Ombre,
    /// Un angle, de -360deg à 360deg : `rotate: -3deg`.
    Angle,
    /// La durée d'un passage d'une allure à l'autre : `0.3s`, `200ms`, ou `none`.
    Duree,
}

/// Les réglages connus : l'apparence, avec les noms du CSS de base.
const REGLAGES: &[(&str, Forme)] = &[
    ("color", Forme::Couleur),
    ("background", Forme::Fond),
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
    // Le lot 4 (ADR-041) : le texte, les ombres, la pose, le passage d'une allure à l'autre.
    ("line-height", Forme::Nombre(0.8, 3.0)),
    ("letter-spacing", Forme::Ecart),
    ("text-transform", Forme::Mot(&["none", "uppercase", "lowercase", "capitalize"])),
    ("text-decoration", Forme::Mot(&["none", "underline", "line-through"])),
    ("box-shadow", Forme::Ombre),
    ("text-shadow", Forme::Ombre),
    ("rotate", Forme::Angle),
    ("scale", Forme::Nombre(0.1, 5.0)),
    ("transition", Forme::Duree),
];

/// Les variables (ADR-041) : `--or: #E9B44C;` dans le style de `Page`, puis `color: --or;`
/// partout. Rend chaque variable et sa valeur (celle du thème clair).
pub fn variables(programme: &Programme) -> Vec<(String, String)> {
    let mut variables: Vec<(String, String)> = Vec::new();
    for regle in &programme.styles {
        if matches!(&regle.cible, Cible::Type(t) if t == "Page" || t == "World") {
            for reglage in regle.reglages.iter().chain(regle.etats.iter().flat_map(|(_, r, _)| r.iter())) {
                if reglage.nom.starts_with("--") && !variables.iter().any(|(n, _)| *n == reglage.nom) {
                    variables.push((reglage.nom.clone(), reglage.valeur.clone()));
                }
            }
        }
    }
    variables
}

/// Les variables d'une valeur, `--or` dans `0 4px 8px --ombre`.
pub fn variables_de(valeur: &str) -> Vec<&str> {
    let mut noms = Vec::new();
    let mut reste = valeur;
    while let Some(debut) = reste.find("--") {
        let avant_ok = debut == 0 || reste[..debut].ends_with([' ', ',', '(']);
        let fin = reste[debut + 2..].find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')).map_or(reste.len(), |f| debut + 2 + f);
        if avant_ok && fin > debut + 2 {
            noms.push(&reste[debut..fin]);
        }
        reste = &reste[fin.max(debut + 2)..];
    }
    noms
}

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
    let variables = variables(programme);
    for (i, regle) in programme.styles.iter().enumerate() {
        let theme = matches!(&regle.cible, Cible::Type(t) if t == "Page" || t == "World");
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
            verifier_reglage(reglage, None, theme, &variables)?;
        }
        // Les états (hover, focus, active, dark, phone) : chacun une fois, avec des réglages connus.
        for (k, (etat, reglages, pos)) in regle.etats.iter().enumerate() {
            if regle.etats[..k].iter().any(|(autre, ..)| autre == etat) {
                return Err(Erreur { message: format!("l'état « {etat} » est donné deux fois dans « {} »", regle.cible), pos: *pos });
            }
            if reglages.is_empty() {
                return Err(Erreur { message: format!("l'état « {etat} » de « {} » est vide : écris ce qui change, comme « {etat}: {{ background: navy; }} »", regle.cible), pos: *pos });
            }
            for (j, reglage) in reglages.iter().enumerate() {
                if reglages[..j].iter().any(|autre| autre.nom == reglage.nom) {
                    return Err(Erreur { message: format!("le réglage « {} » est donné deux fois dans l'état « {etat} »", reglage.nom), pos: reglage.pos });
                }
                verifier_reglage(reglage, Some(etat), theme, &variables)?;
            }
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

fn verifier_reglage(reglage: &Reglage, etat: Option<&str>, theme: bool, variables: &[(String, String)]) -> Result<(), Erreur> {
    let refus = |message: String| Err(Erreur { message, pos: reglage.pos });
    let nom = reglage.nom.as_str();
    // Une variable se définit dans le thème (le style de Page) : une couleur ou une taille.
    if let Some(reste) = nom.strip_prefix("--") {
        if !theme {
            return refus(format!("« {nom} » : une variable se définit dans le style de la page, « Page {{ {nom}: … }} », pour valoir partout"));
        }
        if reste.is_empty() || !reste.starts_with(|c: char| c.is_ascii_lowercase()) || !reste.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return refus(format!("« {nom} » : une variable s'écrit comme en CSS, en minuscules, les mots joints par « - », comme « --or-clair »"));
        }
        let valeur = reglage.valeur.as_str();
        if !(est_couleur(valeur) || est_taille(valeur)) {
            return refus(format!("« {nom}: {valeur} » : une variable porte une couleur ou une taille, comme « #E9B44C » ou « 16px »"));
        }
        return Ok(());
    }
    // Cacher un bloc sur un téléphone : seulement dans « phone: { … } » (ADR-041).
    if nom == "display" {
        return match (etat, reglage.valeur.as_str()) {
            (Some("phone"), "none") => Ok(()),
            (Some("phone"), _) => refus("dans « phone: { … } », « display » ne prend que « none » : cacher le bloc sur un téléphone".into()),
            _ => refus("« display » règle la disposition, pas l'apparence : la disposition vient des blocs ; pour cacher un bloc sur un téléphone : « phone: { display: none; } » ; selon une valeur : If (ADR-017, ADR-041)".into()),
        };
    }
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
    // Les variables sont remplacées par leur valeur, pour vérifier ce qu'elles donnent.
    let mut valeur = reglage.valeur.clone();
    for variable in variables_de(&reglage.valeur) {
        match variables.iter().find(|(n, _)| n == variable) {
            Some((_, remplacement)) => valeur = valeur.replacen(variable, remplacement, 1),
            None => return refus(format!("« {variable} » n'est définie nulle part : écris « Page {{ {variable}: … }} »")),
        }
    }
    let valeur = valeur.as_str();
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
        Forme::Fond => est_couleur(valeur) || est_degrade(valeur) || image_de_fond(valeur).is_some(),
        Forme::Nombre(min, max) => valeur.parse::<f64>().is_ok_and(|v| (*min..=*max).contains(&v)),
        Forme::Ecart => pixels_signes(valeur).is_some_and(|v| (-10.0..=40.0).contains(&v)),
        Forme::Ombre => valeur == "none" || {
            let ombres: Vec<&str> = valeur.split(',').map(str::trim).collect();
            ombres.len() <= 3 && ombres.iter().all(|ombre| est_ombre(ombre))
        },
        Forme::Angle => valeur.strip_suffix("deg").and_then(|n| n.parse::<f64>().ok()).is_some_and(|v| (-360.0..=360.0).contains(&v)),
        Forme::Duree => valeur == "none" || duree_en_ms(valeur).is_some_and(|ms| (0.0..=2000.0).contains(&ms)),
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
        Forme::Fond => "une couleur, un dégradé comme « linear-gradient(#E9B44C, #1a1a2e) », ou une image rangée à côté, « url(\"fond.jpg\") »".to_string(),
        Forme::Nombre(min, max) if nom == "line-height" => format!("un nombre sans unité, de {min} à {max}, comme « 1.5 » : la hauteur de ligne suit alors la taille du texte"),
        Forme::Nombre(min, max) => format!("un nombre de {min} à {max}, comme « 1.1 »"),
        Forme::Ecart => "un écart en pixels, de -10px à 40px, comme « 1px »".to_string(),
        Forme::Ombre => "une ombre : décalage, flou et couleur, comme « 0 4px 12px #00000066 » (trois au plus, séparées par des virgules), ou « none »".to_string(),
        Forme::Angle => "un angle de -360deg à 360deg, comme « -3deg »".to_string(),
        Forme::Duree => "une durée de 0 à 2s, comme « 0.3s » ou « 200ms », ou « none »".to_string(),
    };
    refus(format!("« {nom}: {valeur} » : ce réglage attend {attendu}"))
}

pub(crate) fn est_couleur(valeur: &str) -> bool {
    match valeur.strip_prefix('#') {
        Some(hexa) => [3, 6, 8].contains(&hexa.len()) && hexa.bytes().all(|c| c.is_ascii_hexdigit()),
        None => COULEURS.contains(&valeur),
    }
}

/// `linear-gradient(to right, #E9B44C, #1a1a2e)` ou `radial-gradient(white, navy)` : une
/// direction ou un angle facultatifs, puis de deux à cinq couleurs.
fn est_degrade(valeur: &str) -> bool {
    let Some(dedans) = valeur.strip_prefix("linear-gradient(").or_else(|| valeur.strip_prefix("radial-gradient(")).and_then(|v| v.strip_suffix(')')) else { return false };
    let mut parts: Vec<&str> = dedans.split(',').map(str::trim).collect();
    let lineaire = valeur.starts_with("linear");
    if lineaire && parts.first().is_some_and(|p| p.starts_with("to ") || p.ends_with("deg")) {
        let sens = parts.remove(0);
        let correct = match sens.strip_prefix("to ") {
            Some(cotes) => cotes.split_whitespace().all(|c| ["top", "bottom", "left", "right"].contains(&c)),
            None => sens.strip_suffix("deg").and_then(|n| n.parse::<f64>().ok()).is_some_and(|v| (-360.0..=360.0).contains(&v)),
        };
        if !correct {
            return false;
        }
    }
    (2..=5).contains(&parts.len()) && parts.iter().all(|c| est_couleur(c))
}

/// `url("fond.jpg")` : le nom d'une image rangée à côté du fichier.
pub(crate) fn image_de_fond(valeur: &str) -> Option<&str> {
    let dedans = valeur.strip_prefix("url(")?.strip_suffix(')')?.trim().trim_matches('"');
    let image = [".png", ".jpg", ".jpeg", ".webp", ".svg", ".gif", ".avif"].iter().any(|fin| dedans.ends_with(fin));
    (image && crate::plat::chemin_sur(dedans)).then_some(dedans)
}

/// `-0.5px` → -0.5.
fn pixels_signes(valeur: &str) -> Option<f64> {
    if valeur == "0" {
        return Some(0.0);
    }
    valeur.strip_suffix("px")?.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Une ombre : deux décalages (qui peuvent être négatifs), un flou facultatif, une couleur.
fn est_ombre(ombre: &str) -> bool {
    let mots: Vec<&str> = ombre.split_whitespace().collect();
    let Some((couleur, tailles)) = mots.split_last() else { return false };
    est_couleur(couleur)
        && (2..=3).contains(&tailles.len())
        && tailles[..2].iter().all(|t| pixels_signes(t).is_some_and(|v| v.abs() <= 100.0))
        && tailles.get(2).is_none_or(|flou| est_taille(flou) && pixels_signes(flou).is_some_and(|v| v <= 200.0))
}

/// `0.3s` → 300 ; `200ms` → 200.
pub(crate) fn duree_en_ms(valeur: &str) -> Option<f64> {
    if let Some(ms) = valeur.strip_suffix("ms") {
        return ms.parse().ok();
    }
    valeur.strip_suffix('s')?.parse::<f64>().ok().map(|s| s * 1000.0)
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
        // Le film en mouvement (ADR-034).
        let film = include_str!("../../exemples/motion/holocode/showreel.holo");
        crate::verifier_page(film).unwrap();
        // Les repères, les titres profonds, les états et la superposition (ADR-036).
        let lecons = [
            include_str!("../../exemples/lecons/35-reperes.holo"),
            include_str!("../../exemples/lecons/36-titres-profonds.holo"),
            include_str!("../../exemples/lecons/37-survol.holo"),
            include_str!("../../exemples/lecons/38-superposition.holo"),
            include_str!("../../exemples/lecons/41-video.holo"),
            include_str!("../../exemples/lecons/42-tableau.holo"),
            include_str!("../../exemples/lecons/44-choix.holo"),
            include_str!("../../exemples/lecons/45-survol-qui-agit.holo"),
            include_str!("../../exemples/lecons/46-sinon.holo"),
            include_str!("../../exemples/lecons/47-plus-tard.holo"),
            include_str!("../../exemples/lecons/48-heure.holo"),
            include_str!("../../exemples/lecons/49-repeter.holo"),
            include_str!("../../exemples/lecons/50-texte-soigne.holo"),
            include_str!("../../exemples/lecons/51-ombres-et-fonds.holo"),
            include_str!("../../exemples/lecons/52-variables-et-theme-sombre.holo"),
            include_str!("../../exemples/lecons/53-telephone.holo"),
            include_str!("../../exemples/lecons/54-police.holo"),
            include_str!("../../exemples/lecons/55-petits-textes.holo"),
            include_str!("../../exemples/lecons/56-aller-plus-bas.holo"),
            include_str!("../../exemples/lecons/57-image-et-legende.holo"),
            include_str!("../../exemples/lecons/58-lecteur-de-son.holo"),
            include_str!("../../exemples/lecons/59-glissiere.holo"),
            include_str!("../../exemples/lecons/60-date-heure-couleur.holo"),
            include_str!("../../exemples/lecons/61-progression.holo"),
            include_str!("../../exemples/lecons/62-plis.holo"),
            include_str!("../../exemples/lecons/63-fenetre.holo"),
            include_str!("../../exemples/lecons/64-formulaire.holo"),
            include_str!("../../exemples/lecons/65-icone-de-l-onglet.holo"),
            include_str!("../../exemples/lecons/66-calculer.holo"),
            include_str!("../../exemples/lecons/67-formats.holo"),
            include_str!("../../exemples/lecons/68-liste-qui-change.holo"),
        ];
        for lecon in lecons {
            crate::verifier_page(lecon).unwrap();
        }
        let lecons = lecons.join("\n");
        let source = &format!("{source}\n{jeu}\n{second}\n{accueil}\n{commun}\n{donnees}\n{film}\n{lecons}");
        for bloc in crate::blocs::BLOCS {
            assert!(source.contains(&format!("{bloc}(")) || source.contains(&format!("{bloc}.")), "le bloc « {bloc} » manque dans l'exemple");
        }
        for (reglage, _) in REGLAGES {
            assert!(source.contains(&format!("{reglage}:")), "le réglage « {reglage} » manque dans l'exemple");
        }
        for mot in ["name:", "title:", "seed:", "brightness:", "fragments:", "children:", "inside:", "rules:", "effect:", "budget:", "weight:", "source:", "text:", "color:", "palette:", ".tap", ".enter", ".leave", "state:", "prices:", "{count}", "{total}", ".add(", ".sub(", ".set(", "gap:", "align:", "columns:", "alt:", "is:", "over:", "by:", ".random(", "x:", "y:", "keep:", "value:", "label:", "max:", "Key.left", "meets:", "drag:", "data:", "from:", ".play", "form:", "enter:", "loop:", "letters:", "each:", "repeat:", "ease:", "rotate:", "flip:", "tilt:", "blur:", "hue:", "round:", "scale:", "opacity:", "hover:", "focus:", "active:", "topRight", ".hover", ".hoverEnd", "else:", "{year}", "{month}", "{day}", "weekday", "{hour}", "{minute}", "items:", "key:", "{item.", "item.add(", "dark:", "phone:", "display: none", "linear-gradient(", "url(", "fonts:", "family:", ": --", "~~", "==", "^2^", "~2~", "to: \"#", "caption:", "phone:", "type: date", "type: time", "type: color", "summary:", "open: true", ".open", ".close", ".send", ".sent", ".failed", "icon:", ".mul(", ".div(", ":00}", ":number}", ":cents}", ":name}", "over:", ".push(", ".remove(item)", ".clear()", ".set(\"\")"] {
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
