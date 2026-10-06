//! Les blocs que le format connaît, et les règles qui portent sur leur nom et sur l'ordre
//! des titres (ADR-016, ADR-020). Rien n'est affiché ici : on vérifie seulement.

use crate::holo::{Bloc, Erreur, Programme, Valeur};

/// `Text` est du texte sans rôle ; `P`, `H1`, `H2` et `H3` sont un `Text` avec un rôle (ADR-020).
pub const BLOCS: &[&str] = &["Page", "Text", "P", "H1", "H2", "H3", "A", "Button", "Image", "List", "Point", "World", "On", "Zoom", "Points", "Relief", "Portals", "State", "Prices", "Row", "Column", "Grid", "If", "Hr", "Quote", "Code", "Every", "Board", "Input", "Checkbox", "When", "Part", "Use", "Data", "Sound", "Shape", "Scenes", "Scene", "Enter", "Loop", "H4", "H5", "H6", "Main", "Nav", "Header", "Footer", "Stack", "Video", "Table", "Choice", "After", "Repeat", "Item", "Font", "Slider", "Progress", "Details", "Dialog", "Form", "Module"];

/// Le titre le plus profond : `H6`, comme en HTML (correction d'ADR-020 du 2026-10-06 ; les
/// longs documents en ont besoin). Le numéro dit toujours la place dans le plan, jamais la taille.
pub const TITRE_MAX: u32 = 6;

/// Vérifie tous les blocs d'un fichier : chacun existe, et les titres ne sautent pas de niveau.
pub fn verifier_blocs(programme: &Programme) -> Result<(), Erreur> {
    parcourir(&programme.racine, &mut 0, "")
}

/// Les réglages que chaque bloc accepte. Un réglage inconnu est refusé, jamais avalé en silence
/// (correction du 2026-10-06 : `Page(Title: …)` passait, et le titre était perdu). Les blocs
/// absents de cette liste vérifient leurs réglages eux-mêmes (`State`, `Prices`, `Data`,
/// `Zoom`, `Points`, `Relief`, `Portals`, `Enter`, `Loop`, `Use`).
const REGLAGES_DES_BLOCS: &[(&str, &[&str])] = &[
    ("Page", &["name", "title", "children", "pixels", "rules", "state", "prices", "keep", "data", "zoom", "points", "relief", "portals", "lang", "description", "image", "fonts", "icon", "modules", "parts"]),
    ("World", &["name", "children", "pixels", "rules"]),
    ("Part", &["name", "params", "children", "rules"]),
    ("Text", &["name"]),
    ("P", &["name"]),
    ("H1", &["name"]),
    ("H2", &["name"]),
    ("H3", &["name"]),
    ("H4", &["name"]),
    ("H5", &["name"]),
    ("H6", &["name"]),
    ("A", &["name", "to"]),
    ("Button", &["name", "text"]),
    ("Image", &["name", "source", "weight", "alt", "phone", "caption"]),
    ("Sound", &["name", "source", "weight", "label"]),
    ("Shape", &["name", "form", "color", "size"]),
    ("List", &["name", "children", "ordered"]),
    ("Hr", &["name"]),
    ("Quote", &["name", "by"]),
    ("Code", &["name"]),
    ("Header", &["name", "children"]),
    ("Nav", &["name", "children"]),
    ("Main", &["name", "children"]),
    ("Footer", &["name", "children"]),
    ("Row", &["name", "children", "gap", "align"]),
    ("Column", &["name", "children", "gap", "align"]),
    ("Grid", &["name", "children", "gap", "columns"]),
    ("Stack", &["name", "children"]),
    ("Board", &["name", "children", "height"]),
    ("Point", &["name", "seed", "brightness", "fragments", "color", "palette", "budget", "inside", "above"]),
    ("Input", &["name", "value", "label", "max", "lines", "type"]),
    ("Slider", &["name", "value", "label", "min", "max"]),
    ("Progress", &["name", "value", "max", "label"]),
    ("Details", &["name", "summary", "children", "open"]),
    ("Dialog", &["name", "children"]),
    ("Form", &["name", "children"]),
    ("Choice", &["name", "value", "label", "options", "menu"]),
    ("Video", &["name", "source", "label", "weight"]),
    ("Table", &["name", "caption", "head", "rows"]),
    ("Checkbox", &["name", "value", "label"]),
    ("If", &["name", "is", "not", "over", "under", "children", "rules", "else"]),
    ("On", &["effect"]),
    ("Every", &["effect"]),
    ("After", &["effect"]),
    ("When", &["is", "not", "over", "under", "meets", "within", "effect"]),
    ("Scenes", &["name", "children", "height", "repeat"]),
    ("Scene", &["name", "children", "for"]),
    ("Font", &["family", "source"]),
    ("Module", &["name", "source", "input", "output", "time", "memory"]),
];

/// Les réglages de chaque bloc, pour l'éditeur (ADR-046) : il propose ceux du bloc où l'on écrit.
pub fn parametres_des_blocs() -> &'static [(&'static str, &'static [&'static str])] {
    REGLAGES_DES_BLOCS
}

/// Les blocs qui ne se voient pas : ils ne bougent pas (`enter`, `loop`).
const SANS_MOUVEMENT: &[&str] = &["Page", "World", "Part", "On", "Every", "When", "After", "Sound", "Scene"];

/// Vérifie les réglages d'un bloc, selon le bloc qui le contient (`parent`).
fn verifier_reglages(bloc: &Bloc, parent: &str) -> Result<(), Erreur> {
    let Some((_, permis)) = REGLAGES_DES_BLOCS.iter().find(|(nom, _)| *nom == bloc.nom) else { return Ok(()) };
    for argument in &bloc.arguments {
        let Some(nom) = argument.nom.as_deref() else { continue };
        let mouvement = (nom == "enter" || nom == "loop") && !SANS_MOUVEMENT.contains(&bloc.nom.as_str());
        let sur_un_plateau = matches!(nom, "x" | "y" | "drag") && parent == "Board";
        let dans_une_pile = nom == "align" && parent == "Stack";
        if permis.contains(&nom) || mouvement || sur_un_plateau || dans_une_pile {
            // Un nom de bloc commence par une majuscule, comme un bloc : ce qu'on touche a une
            // majuscule, ce qui change (une valeur) n'en a pas.
            if nom == "name" {
                if let crate::holo::Valeur::Nom(donne) = &argument.valeur {
                    if donne.starts_with(|c: char| c.is_ascii_lowercase()) || donne.contains('_') {
                        let flutter = crate::etat::en_flutter(donne);
                        let majuscule: String = flutter.chars().take(1).map(|c| c.to_ascii_uppercase()).chain(flutter.chars().skip(1)).collect();
                        return Err(Erreur { message: format!("« name: {donne} » : un nom de bloc s'écrit comme en Flutter, une majuscule au début et à chaque mot ; écris « name: {majuscule} » (ADR-037)"), pos: argument.pos });
                    }
                }
            }
            continue;
        }
        let message = if matches!(nom, "x" | "y" | "drag") {
            format!("« {nom}: » place un bloc sur un plateau : mets « {} » dans Board(children: [ … ])", bloc.nom)
        } else if nom == "align" && bloc.nom != "Row" && bloc.nom != "Column" {
            format!("« align: » place un bloc posé sur un autre : mets « {} » dans Stack(children: [ … ])", bloc.nom)
        } else if let Some(bon) = permis.iter().find(|connu| connu.eq_ignore_ascii_case(nom)) {
            format!("« {nom} » : un paramètre s'écrit en minuscules, écris « {bon} »")
        } else if nom == "styles" || nom == "style" {
            "un style s'écrit comme en CSS, après le bloc racine : « P { color: gray; } » (ADR-017)".to_string()
        } else if bloc.nom == "World" && matches!(nom, "state" | "prices" | "keep" | "data" | "zoom" | "points" | "relief" | "portals" | "title") {
            format!("« {nom}: » se règle sur la page, pas dans un monde : un monde partage les valeurs et la vue de sa page")
        } else {
            format!("« {} » n'a pas de paramètre « {nom} » ; paramètres possibles : {}", bloc.nom, permis.join(", "))
        };
        return Err(Erreur { message, pos: argument.pos });
    }
    Ok(())
}

fn parcourir(bloc: &Bloc, dernier_titre: &mut u32, parent: &str) -> Result<(), Erreur> {
    if crate::etat::est_demande(bloc) {
        // `p.card(...)` : un bloc écrit en minuscules, plutôt qu'une demande.
        let (avant, apres) = bloc.nom.split_once('.').unwrap_or((&bloc.nom, ""));
        let majuscule: String = avant.chars().take(1).map(|c| c.to_ascii_uppercase()).chain(avant.chars().skip(1)).collect();
        let message = if BLOCS.contains(&majuscule.as_str()) {
            format!("« {} » : un nom de bloc commence par une majuscule, écris « {majuscule}.{apres} »", bloc.nom)
        } else {
            format!("« {}(...) » est une demande : elle s'écrit dans l'effet d'une règle, On(Add.tap, effect: {}(1))", bloc.nom, bloc.nom)
        };
        return Err(Erreur { message, pos: bloc.pos });
    }
    if !BLOCS.contains(&bloc.nom.as_str()) {
        return Err(Erreur { message: bloc_inconnu(&bloc.nom), pos: bloc.pos });
    }
    // Une répétition est dépliée à la lecture (ADR-040) : un « Item » qui reste est mal placé.
    // Depuis ADR-051, un « Item » est aussi un élément d'une liste de la page, dans State ou
    // dans une demande « push ».
    if bloc.nom == "Item" && parent != "State" && !parent.ends_with(".push") {
        return Err(Erreur { message: "« Item » est un élément d'une répétition : Repeat(items: [ Item(…) ], children: [ … ])".into(), pos: bloc.pos });
    }
    verifier_reglages(bloc, parent)?;
    if let Some(niveau) = niveau_de_titre(&bloc.nom) {
        // Le numéro dit la place dans le plan, jamais la taille : `H3` ne suit pas `H1`.
        if niveau > *dernier_titre + 1 {
            let message = if *dernier_titre == 0 {
                format!("« {} » : le premier titre est « H1 » ; le numéro dit la place dans le plan, la taille se règle par le style (ADR-020)", bloc.nom)
            } else {
                format!(
                    "« {} » arrive après « H{} » : un titre ne saute pas de niveau, écris « H{} » ; la taille se règle par le style (ADR-020)",
                    bloc.nom,
                    *dernier_titre,
                    *dernier_titre + 1
                )
            };
            return Err(Erreur { message, pos: bloc.pos });
        }
        *dernier_titre = niveau;
    }
    // Une page et un monde ont chacun leur propre plan.
    let mut plan_propre = 0;
    let plan = if bloc.nom == "Page" || bloc.nom == "World" { &mut plan_propre } else { dernier_titre };
    for argument in &bloc.arguments {
        // L'effet d'une règle peut être une demande, `cart.add(1)` : `regles.rs` la vérifie.
        // (une seule, ou plusieurs entre crochets : dans les deux cas, on ne descend pas dedans)
        let demande = matches!(bloc.nom.as_str(), "On" | "Every" | "When" | "After") && argument.nom.as_deref() == Some("effect");
        if !demande {
            visiter(&argument.valeur, plan, &bloc.nom)?;
        }
    }
    Ok(())
}

fn visiter(valeur: &Valeur, dernier_titre: &mut u32, parent: &str) -> Result<(), Erreur> {
    match valeur {
        Valeur::Bloc(bloc) => parcourir(bloc, dernier_titre, parent),
        Valeur::Liste(elements) => elements.iter().try_for_each(|e| visiter(e, dernier_titre, parent)),
        _ => Ok(()),
    }
}

/// `H1` → 1. Rend `None` pour tout ce qui n'est pas un titre connu.
fn niveau_de_titre(nom: &str) -> Option<u32> {
    let niveau: u32 = nom.strip_prefix('H')?.parse().ok()?;
    (1..=TITRE_MAX).contains(&niveau).then_some(niveau)
}

/// Le message pour un bloc qui n'existe pas, avec le mot à écrire quand on le devine.
pub fn bloc_inconnu(nom: &str) -> String {
    let chiffres = nom.strip_prefix('H').unwrap_or("");
    if !chiffres.is_empty() && chiffres.bytes().all(|c| c.is_ascii_digit()) {
        return format!("bloc inconnu « {nom} » : les titres vont de « H1 » à « H{TITRE_MAX} » (ADR-020)");
    }
    if nom == "Style" || nom == "Theme" {
        return format!("« {nom} » n'est pas un bloc : un style s'écrit comme en CSS, après le bloc racine, « .card {{ color: gray; }} » ; le thème est le style de « Page » ou de « World » (ADR-017)");
    }
    match ancien_mot(nom) {
        Some(nouveau) => format!("bloc inconnu « {nom} » : le vocabulaire est en anglais, écris « {nouveau} » (ADR-016)"),
        None => format!("bloc inconnu « {nom} »"),
    }
}

/// Les mots français d'avant ADR-016, pour guider vers le mot anglais plutôt que de dire
/// seulement « inconnu ».
pub fn ancien_mot(mot: &str) -> Option<&'static str> {
    Some(match mot {
        "nom" => "name",
        "graine" => "seed",
        "lumiere" => "brightness",
        "morceler" => "fragments",
        "contenu" => "children",
        "interieur" => "inside",
        "phenomenes" => "rules",
        "titre" => "title",
        "texte" => "text",
        "couleur" => "color",
        "Texte" => "Text",
        "Bouton" => "Button",
        "Monde" => "World",
        "Liste" => "List",
        "Quand" => "On",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::lire;

    fn verifier(src: &str) -> Result<(), Erreur> {
        verifier_blocs(&lire(src)?)
    }

    #[test]
    fn accepte_les_cas_valides_de_la_suite() {
        for cas in [
            include_str!("../../experiments/conformite-v0.1/cas/valides/01-page-simple.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/02-big-bang.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/03-entrer-dans-un-point.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/04-budget-respecte.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/05-texte-sans-role.holo"),
        ] {
            verifier(cas).unwrap();
        }
    }

    #[test]
    fn refuse_les_blocs_inconnus_avec_la_bonne_ligne() {
        let div = verifier(include_str!("../../experiments/conformite-v0.1/cas/refuses/E05-bloc-inconnu.holo")).unwrap_err();
        assert_eq!(div.pos.ligne, 4);
        assert!(div.message.contains("bloc inconnu « Div »"));
        let h4 = verifier(include_str!("../../experiments/conformite-v0.1/cas/refuses/E11-titre-trop-profond.holo")).unwrap_err();
        assert_eq!(h4.pos.ligne, 7);
        assert!(h4.message.contains("de « H1 » à « H6 »"));
        assert!(verifier("Page(children: [ Texte(\"Bonjour\") ])").unwrap_err().message.contains("écris « Text »"));
        assert!(verifier("Page(styles: [ Style(color: gray) ])").unwrap_err().message.contains("comme en CSS"));
    }

    #[test]
    fn un_bloc_en_minuscules_est_refuse_avec_le_mot_a_ecrire() {
        let e = verifier(include_str!("../../experiments/conformite-v0.1/cas/refuses/E09-bloc-en-minuscules.holo")).unwrap_err();
        assert_eq!(e.pos.ligne, 5);
        assert!(e.message.contains("écris « H1 »"));
        assert!(verifier("page(title: \"x\")").unwrap_err().message.contains("écris « Page »"));
        // Un réglage reste en minuscules : seul ce qui ouvre une parenthèse est un bloc.
        verifier("Page(children: [ Button(name: Open, text: \"Enter\") ])").unwrap();
    }

    #[test]
    fn les_titres_ne_sautent_pas_de_niveau() {
        let saut = verifier(include_str!("../../experiments/conformite-v0.1/cas/refuses/E10-titre-saute-un-niveau.holo")).unwrap_err();
        assert_eq!(saut.pos.ligne, 7);
        assert!(saut.message.contains("écris « H2 »"));
        assert!(verifier("Page(children: [ H2(\"x\") ])").unwrap_err().message.contains("le premier titre est « H1 »"));
        // Remonter d'un ou de plusieurs niveaux est permis ; descendre se fait un par un.
        verifier("Page(children: [ H1(\"a\"), H2(\"b\"), H3(\"c\"), H1(\"d\"), H2(\"e\") ])").unwrap();
        // Chaque page et chaque monde a son propre plan.
        verifier("Page(children: [ H1(\"a\"), H2(\"b\"), Point(name: A, seed: 1, inside: World(children: [ H1(\"c\") ])), H3(\"d\") ])").unwrap();
        assert!(verifier("Page(children: [ H1(\"a\"), Point(name: A, seed: 1, inside: World(children: [ H2(\"c\") ])) ])").is_err());
    }

    #[test]
    fn rien_n_est_avale_en_silence() {
        for (source, message) in [
            ("Page(Title: \"a\", children: [ H1(\"a\") ])", "écris « title »"),
            ("Page(colour: \"a\", children: [])", "n'a pas de paramètre « colour »"),
            ("Page(children: [ Button(name: buy, text: \"x\") ])", "écris « name: Buy »"),
            ("Page(children: [ P(\"a\", x: 10, y: 10) ])", "mets « P » dans Board"),
            ("Page(children: [ P(\"a\", align: top) ])", "mets « P » dans Stack"),
            ("Page(children: [ Image(source: \"a.png\", Alt: \"x\") ])", "écris « alt »"),
            ("Page(children: [ H1(\"a\", size: 3) ])", "« H1 » n'a pas de paramètre « size »"),
        ] {
            let erreur = verifier(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
        // Ce qui est permis selon la place : sur un plateau, dans une pile, en mouvement.
        verifier("Page(children: [ Board(children: [ Shape(name: S, form: circle, x: 1, y: 2, drag: true) ]), Stack(children: [ P(\"a\"), P(\"b\", align: top) ]), H1(\"c\", enter: Enter(y: 4px)) ])").unwrap();
    }

}
