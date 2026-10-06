//! La répétition (ADR-040) : un bloc écrit une fois, posé pour chaque élément d'une liste.
//!
//! ```holo
//! Repeat(
//!   items: [
//!     Item(key: sunrise, title: "Sunrise", image: "sunrise.png"),
//!     Item(key: river, title: "The river", image: "river.png"),
//!   ],
//!   children: [ Column(children: [ Image(source: item.image, alt: "{item.title}"), H2("{item.title}"), Button(name: Add, text: "Add") ]) ],
//!   rules: [ On(Add.tap, effect: item.add(1)) ],
//! )
//! ```
//!
//! Comme `Use`, la répétition est dépliée à la lecture : le reste du moteur ne voit que les blocs
//! qu'on aurait écrits à la main. Dans le modèle, `item` désigne l'élément :
//! - `{item.title}` dans un texte devient le champ ; `{item}` montre la valeur de la page qui
//!   porte le nom de la clé (`{sunrise}`) ;
//! - `item.image` à la place d'une valeur devient le champ lui-même ;
//! - `item`, et `item.add(1)`, désignent la valeur de la page qui porte le nom de la clé ;
//! - un bloc nommé reçoit le nom de son élément : `Add` devient `AddSunrise`, partout dans le
//!   modèle et dans ses règles. Ses règles rejoignent celles de la page (ou du monde).

use crate::holo::{Argument, Bloc, Erreur, Valeur};

/// Le nombre d'éléments d'une répétition, au plus.
pub const ELEMENTS_MAX: usize = 200;
/// Le nombre de blocs qu'une page peut obtenir en dépliant ses répétitions, au plus.
pub const BLOCS_MAX: usize = 20_000;

/// Les mots du langage qu'un champ ne peut pas porter : `item.add` doit rester une demande.
const MOTS_RESERVES: &[&str] = &["key", "add", "sub", "set", "random", "enter", "leave", "play", "portals", "tap", "hover", "hoverEnd"];

/// Déplie les répétitions d'un site (une page, ou un monde) et de tout ce qu'il contient.
pub fn deplier_site(site: &mut Bloc, compte: &mut usize) -> Result<(), Erreur> {
    let mut regles = Vec::new();
    for argument in &mut site.arguments {
        deplier_valeur(&mut argument.valeur, &mut regles, compte)?;
    }
    if regles.is_empty() {
        return Ok(());
    }
    match site.arguments.iter_mut().find(|a| a.nom.as_deref() == Some("rules")) {
        Some(Argument { valeur: Valeur::Liste(liste), .. }) => liste.extend(regles),
        Some(argument) => return Err(Erreur { message: "« rules » est une liste de règles : rules: [ … ]".into(), pos: argument.pos }),
        None => site.arguments.push(Argument { nom: Some("rules".into()), valeur: Valeur::Liste(regles), pos: site.pos }),
    }
    Ok(())
}

fn deplier_valeur(valeur: &mut Valeur, regles: &mut Vec<Valeur>, compte: &mut usize) -> Result<(), Erreur> {
    match valeur {
        // Le monde d'un point est un autre site : ses règles restent chez lui.
        Valeur::Bloc(bloc) if bloc.nom == "World" => deplier_site(bloc, compte),
        Valeur::Bloc(bloc) if bloc.nom == "Repeat" => {
            Err(Erreur { message: "une répétition se pose parmi des blocs : children: [ Repeat(items: [ … ], children: [ … ]) ]".into(), pos: bloc.pos })
        }
        Valeur::Bloc(bloc) => bloc.arguments.iter_mut().try_for_each(|a| deplier_valeur(&mut a.valeur, regles, compte)),
        Valeur::Liste(elements) => {
            let mut poses = Vec::with_capacity(elements.len());
            for mut element in std::mem::take(elements) {
                match element {
                    Valeur::Bloc(repetition) if repetition.nom == "Repeat" => {
                        let (enfants, regles_de_la_repetition) = deplier_repetition(&repetition)?;
                        for mut enfant in enfants {
                            deplier_valeur(&mut enfant, regles, compte)?;
                            *compte += taille(&enfant);
                            poses.push(enfant);
                        }
                        regles.extend(regles_de_la_repetition);
                        if *compte > BLOCS_MAX {
                            return Err(Erreur { message: format!("les répétitions de la page donnent plus de {BLOCS_MAX} blocs : c'est trop pour une page"), pos: repetition.pos });
                        }
                    }
                    _ => {
                        deplier_valeur(&mut element, regles, compte)?;
                        poses.push(element);
                    }
                }
            }
            *elements = poses;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Le nombre de blocs d'une valeur.
fn taille(valeur: &Valeur) -> usize {
    match valeur {
        Valeur::Bloc(bloc) => 1 + bloc.arguments.iter().map(|a| taille(&a.valeur)).sum::<usize>(),
        Valeur::Liste(elements) => elements.iter().map(taille).sum(),
        _ => 0,
    }
}

/// Un élément de la liste : sa clé (le nom d'une valeur de la page) et ses champs.
struct Element<'a> {
    cle: Option<&'a str>,
    champs: Vec<(&'a str, &'a Valeur)>,
    pos: crate::holo::Pos,
}

/// Les enfants et les règles d'une répétition, écrits une fois par élément.
fn deplier_repetition(repetition: &Bloc) -> Result<(Vec<Valeur>, Vec<Valeur>), Erreur> {
    let exemple = "Repeat(items: [ Item(title: \"Sunrise\"), Item(title: \"River\") ], children: [ H2(\"{item.title}\") ])";
    let (mut elements, mut modele, mut regles) = (None, None, Vec::new());
    for argument in &repetition.arguments {
        match (argument.nom.as_deref(), &argument.valeur) {
            (Some("items"), Valeur::Liste(liste)) => elements = Some((liste, argument.pos)),
            (Some("children"), Valeur::Liste(liste)) => modele = Some(liste),
            (Some("rules"), Valeur::Liste(liste)) => regles = liste.clone(),
            (Some(mot @ ("items" | "children" | "rules")), _) => return Err(Erreur { message: format!("« Repeat({mot}: …) » attend une liste entre crochets : {exemple}"), pos: argument.pos }),
            (Some(autre), _) => return Err(Erreur { message: format!("« Repeat » n'a pas de paramètre « {autre} » ; paramètres possibles : items, children, rules"), pos: argument.pos }),
            (None, _) => return Err(Erreur { message: format!("chaque paramètre de « Repeat » est nommé : {exemple}"), pos: argument.pos }),
        }
    }
    let (Some((liste, pos_liste)), Some(modele)) = (elements, modele) else {
        return Err(Erreur { message: format!("« Repeat » attend « items » et « children » : {exemple}"), pos: repetition.pos });
    };
    if liste.is_empty() || liste.len() > ELEMENTS_MAX {
        return Err(Erreur { message: format!("« Repeat(items: …) » attend de 1 à {ELEMENTS_MAX} éléments"), pos: pos_liste });
    }
    if contient_une_repetition(modele) || regles.iter().any(|r| contient_une_repetition(std::slice::from_ref(r))) {
        return Err(Erreur { message: "une répétition dans une répétition n'est pas permise : déplie la liste du dedans à part".into(), pos: repetition.pos });
    }
    for regle in &regles {
        if !matches!(regle, Valeur::Bloc(b) if matches!(b.nom.as_str(), "On" | "Every" | "When" | "After" | "If")) {
            return Err(Erreur { message: "« Repeat(rules: …) » range des règles : rules: [ On(Add.tap, effect: item.add(1)) ]".into(), pos: repetition.pos });
        }
    }
    // Les éléments.
    let mut lus: Vec<Element> = Vec::new();
    for valeur in liste {
        let Valeur::Bloc(item) = valeur else {
            return Err(Erreur { message: "« items » contient des « Item(…) » : items: [ Item(title: \"Sunrise\") ]".into(), pos: pos_liste });
        };
        if item.nom != "Item" {
            return Err(Erreur { message: format!("« items » contient des « Item(…) », pas des « {} »", item.nom), pos: item.pos });
        }
        let mut element = Element { cle: None, champs: Vec::new(), pos: item.pos };
        for argument in &item.arguments {
            let Some(nom) = argument.nom.as_deref() else {
                return Err(Erreur { message: "chaque champ d'un « Item » est nommé : Item(title: \"Sunrise\", price: 120)".into(), pos: argument.pos });
            };
            match (nom, &argument.valeur) {
                ("key", Valeur::Nom(cle)) if cle.starts_with(|c: char| c.is_ascii_lowercase()) && cle.chars().all(|c| c.is_ascii_alphanumeric()) => element.cle = Some(cle),
                ("key", _) => return Err(Erreur { message: "« key » est le nom d'une valeur de la page, comme sunrise : Item(key: sunrise, …)".into(), pos: argument.pos }),
                (mot, _) if MOTS_RESERVES.contains(&mot) => return Err(Erreur { message: format!("« {mot} » est un mot du langage ; choisis un autre nom de champ"), pos: argument.pos }),
                (_, Valeur::Texte(_) | Valeur::Entier(_) | Valeur::Nombre { .. } | Valeur::Nom(_) | Valeur::Bool(_)) => {
                    if !nom.starts_with(|c: char| c.is_ascii_lowercase()) || !nom.chars().all(|c| c.is_ascii_alphanumeric()) {
                        return Err(Erreur { message: format!("« {nom} » : un champ s'écrit comme une valeur, en minuscules, les mots joints par une majuscule (ADR-037)"), pos: argument.pos });
                    }
                    if element.champs.iter().any(|(connu, _)| *connu == nom) {
                        return Err(Erreur { message: format!("le champ « {nom} » est donné deux fois"), pos: argument.pos });
                    }
                    element.champs.push((nom, &argument.valeur));
                }
                _ => return Err(Erreur { message: format!("le champ « {nom} » attend un texte, un nombre ou un mot, pas un bloc ni une liste"), pos: argument.pos }),
            }
        }
        if let Some(cle) = element.cle {
            if lus.iter().any(|autre| autre.cle == Some(cle)) {
                return Err(Erreur { message: format!("deux éléments ont la clé « {cle} »"), pos: item.pos });
            }
        }
        lus.push(element);
    }
    // Les noms donnés dans le modèle : chacun reçoit le nom de son élément.
    let mut noms = Vec::new();
    for valeur in modele.iter().chain(regles.iter()) {
        noms_donnes(valeur, &mut noms);
    }
    let mut enfants = Vec::new();
    let mut regles_depliees = Vec::new();
    for element in &lus {
        for valeur in modele {
            let mut copie = valeur.clone();
            remplacer(&mut copie, element, &noms, repetition.pos)?;
            enfants.push(copie);
        }
        for valeur in &regles {
            let mut copie = valeur.clone();
            remplacer(&mut copie, element, &noms, repetition.pos)?;
            regles_depliees.push(copie);
        }
    }
    Ok((enfants, regles_depliees))
}

fn contient_une_repetition(valeurs: &[Valeur]) -> bool {
    valeurs.iter().any(|valeur| match valeur {
        Valeur::Bloc(bloc) => bloc.nom == "Repeat" || bloc.arguments.iter().any(|a| contient_une_repetition(std::slice::from_ref(&a.valeur))),
        Valeur::Liste(elements) => contient_une_repetition(elements),
        _ => false,
    })
}

fn noms_donnes(valeur: &Valeur, noms: &mut Vec<String>) {
    match valeur {
        Valeur::Bloc(bloc) => {
            if let Some(Valeur::Nom(nom)) = bloc.argument("name").map(|a| &a.valeur) {
                if !noms.contains(nom) {
                    noms.push(nom.clone());
                }
            }
            bloc.arguments.iter().for_each(|a| noms_donnes(&a.valeur, noms));
        }
        Valeur::Liste(elements) => elements.iter().for_each(|e| noms_donnes(e, noms)),
        _ => {}
    }
}

/// `sunrise` → `Sunrise` ; `blueDoor` → `BlueDoor`.
fn en_majuscule(cle: &str) -> String {
    let mut lettres = cle.chars();
    lettres.next().map(|c| c.to_ascii_uppercase().to_string() + lettres.as_str()).unwrap_or_default()
}

/// Écrit le modèle pour un élément.
fn remplacer(valeur: &mut Valeur, element: &Element, noms: &[String], pos: crate::holo::Pos) -> Result<(), Erreur> {
    let sans_cle = |quoi: &str| Erreur {
        message: format!("{quoi} : il faut une clé à chaque élément, le nom d'une valeur de la page, Item(key: sunrise, …)"),
        pos: element.pos,
    };
    let champ = |nom: &str| element.champs.iter().find(|(connu, _)| *connu == nom).map(|(_, v)| *v);
    match valeur {
        Valeur::Texte(texte) => *texte = remplacer_dans_le_texte(texte, element)?,
        Valeur::Nom(nom) => {
            let cle = || element.cle.ok_or_else(|| sans_cle("« item » désigne la valeur de l'élément"));
            if nom == "item" || nom == "item.key" {
                *nom = cle()?.to_string();
            } else if let Some(nom_du_champ) = nom.strip_prefix("item.").map(str::to_string) {
                match champ(&nom_du_champ) {
                    Some(remplacant) => *valeur = remplacant.clone(),
                    // `item.enter` : une capacité demandée à la valeur de l'élément.
                    None if MOTS_RESERVES.contains(&nom_du_champ.as_str()) => *nom = format!("{}.{nom_du_champ}", cle()?),
                    None => return Err(Erreur { message: format!("l'élément n'a pas de champ « {nom_du_champ} » : Item({nom_du_champ}: …)"), pos: element.pos }),
                }
            } else {
                // Un nom donné dans le modèle, seul (`name: Add`) ou suivi d'un signal (`Add.tap`).
                let (debut, suite) = nom.split_once('.').map_or((nom.as_str(), None), |(a, b)| (a, Some(b)));
                if noms.iter().any(|n| n == debut) {
                    let cle = element.cle.ok_or_else(|| sans_cle(&format!("« {debut} » est nommé dans la répétition, et chaque copie doit avoir son propre nom")))?;
                    let nouveau = format!("{debut}{}", en_majuscule(cle));
                    *nom = match suite {
                        Some(suite) => format!("{nouveau}.{suite}"),
                        None => nouveau,
                    };
                }
            }
        }
        Valeur::Liste(elements) => {
            for e in elements {
                remplacer(e, element, noms, pos)?;
            }
        }
        Valeur::Bloc(bloc) => {
            // Une demande faite à la valeur de l'élément : `item.add(1)`.
            if let Some(verbe) = bloc.nom.strip_prefix("item.") {
                bloc.nom = format!("{}.{verbe}", element.cle.ok_or_else(|| sans_cle("« item » désigne la valeur de l'élément"))?);
            }
            if bloc.nom == "Item" {
                return Err(Erreur { message: "« Item » se place dans « items », pas dans le modèle".into(), pos: bloc.pos });
            }
            for argument in &mut bloc.arguments {
                remplacer(&mut argument.valeur, element, noms, pos)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// `{item.title}` devient le champ ; `{item}`, la valeur de la page qui porte la clé.
fn remplacer_dans_le_texte(texte: &str, element: &Element) -> Result<String, Erreur> {
    if !texte.contains("{item") {
        return Ok(texte.to_string());
    }
    let mut sortie = String::with_capacity(texte.len());
    let mut reste = texte;
    while let Some(debut) = reste.find("{item") {
        sortie.push_str(&reste[..debut]);
        let apres = &reste[debut + 1..];
        let Some(fin) = apres.find('}') else {
            sortie.push_str(&reste[debut..]);
            reste = "";
            break;
        };
        let dedans = &apres[..fin];
        if dedans == "item" {
            let cle = element.cle.ok_or_else(|| Erreur { message: "« {item} » montre la valeur de l'élément : il faut une clé, Item(key: sunrise, …)".into(), pos: element.pos })?;
            sortie.push_str(&format!("{{{cle}}}"));
        } else if let Some(nom) = dedans.strip_prefix("item.") {
            match element.champs.iter().find(|(connu, _)| *connu == nom).map(|(_, v)| *v) {
                Some(Valeur::Texte(t)) => sortie.push_str(t),
                Some(Valeur::Entier(n)) => sortie.push_str(&n.to_string()),
                Some(Valeur::Nombre { valeur, unite }) => sortie.push_str(&format!("{valeur}{}", unite.as_deref().unwrap_or(""))),
                Some(Valeur::Nom(n)) => sortie.push_str(n),
                Some(Valeur::Bool(b)) => sortie.push_str(if *b { "true" } else { "false" }),
                Some(_) | None if nom == "key" && element.cle.is_some() => sortie.push_str(element.cle.unwrap_or_default()),
                _ => return Err(Erreur { message: format!("l'élément n'a pas de champ « {nom} » : Item({nom}: …)"), pos: element.pos }),
            }
        } else {
            // `{itemCount}` : une valeur de la page dont le nom commence par « item ».
            sortie.push_str(&reste[debut..debut + 1 + fin + 1]);
        }
        reste = &apres[fin + 1..];
    }
    sortie.push_str(reste);
    Ok(sortie)
}

#[cfg(test)]
mod tests {
    use crate::holo::lire;

    #[test]
    fn une_repetition_est_depliee_a_la_lecture() {
        let source = "Page(state: State(sunrise: 0, river: 0), children: [ H1(\"Shop\"), Grid(children: [ Repeat(items: [ Item(key: sunrise, title: \"Sunrise\", image: \"s.png\"), Item(key: river, title: \"River\", image: \"r.png\") ], children: [ Column(children: [ Image(source: item.image, alt: \"{item.title}\"), H2(\"{item.title} ({item})\"), Button(name: Add, text: \"Add\") ]) ], rules: [ On(Add.tap, effect: item.add(1)) ]) ]) ])";
        let programme = lire(source).unwrap();
        let html = crate::plat::page_html(&programme, "").unwrap();
        assert!(html.contains("<img class=\"holo-Image\" src=\"s.png\" alt=\"Sunrise\"><h2 class=\"holo-H2\">Sunrise (<span data-state=\"sunrise\">0</span>)</h2><button type=\"button\" class=\"holo-Button\" data-name=\"AddSunrise\">Add</button>"), "{html}");
        assert!(html.contains("data-name=\"AddRiver\""), "{html}");
        crate::verifier_page(source).unwrap();
        assert_eq!(crate::arbitrer(source, "sunrise=0;river=0", "AddRiver.tap"), "sunrise=0;river=1");
    }

    #[test]
    fn ce_qui_est_refuse() {
        for (source, message) in [
            ("Page(children: [ Repeat(items: [], children: [ P(\"x\") ]) ])", "de 1 à 200"),
            ("Page(children: [ Repeat(children: [ P(\"x\") ]) ])", "attend « items » et « children »"),
            ("Page(children: [ Repeat(items: [ Item(title: \"a\") ], children: [ P(\"{item.price}\") ]) ])", "pas de champ « price »"),
            ("Page(children: [ Repeat(items: [ Item(title: \"a\") ], children: [ Button(name: B, text: \"b\") ]) ])", "il faut une clé"),
            ("Page(children: [ Repeat(items: [ Item(key: a), Item(key: a) ], children: [ P(\"x\") ]) ])", "deux éléments ont la clé"),
            ("Page(children: [ Repeat(items: [ Item(add: 1) ], children: [ P(\"x\") ]) ])", "est un mot du langage"),
            ("Page(children: [ Repeat(items: [ Item(t: 1) ], children: [ Repeat(items: [ Item(u: 1) ], children: [ P(\"x\") ]) ]) ])", "dans une répétition n'est pas permise"),
            ("Page(children: [ Repeat(items: [ Item(t: 1) ], children: [ P(\"x\") ], colour: 3) ])", "n'a pas de paramètre « colour »"),
            ("Page(children: [ Item(t: 1) ])", "élément d'une répétition"),
            ("Page(children: [ P(\"{item.title}\") ])", "dans une répétition"),
        ] {
            let erreur = crate::verifier_page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }
}
