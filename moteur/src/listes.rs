//! Les listes qui changent pendant la visite (ADR-044).
//!
//! ```holo
//! Page(
//!   state: State(task: "", tasks: []),
//!   children: [
//!     Input(value: task, label: "A task"),
//!     Button(name: Add, text: "Add"),
//!     Repeat(over: tasks, children: [ Row(children: [ Text("{item}"), Button(name: Done, text: "Done") ]) ],
//!            rules: [ On(Done.tap, effect: tasks.remove(item)) ]),
//!   ],
//!   rules: [ On(Add.tap, effect: [tasks.push(task), task.set("")]) ],
//! )
//! ```
//!
//! Une liste est une valeur de la page : des textes, cent au plus. Elle ne change que par un geste
//! du visiteur (une règle `On`), par trois demandes : `push` ajoute un texte, `remove` retire
//! l'élément d'une ligne, `clear` vide la liste. Une valeur de texte se vide ou se remplit par
//! `set` : `task.set("")`. `Repeat(over: tasks, …)` montre la liste, une ligne par élément ; dans
//! la ligne, `{item}` est le texte de l'élément, et `item` désigne cet élément dans les règles.

use crate::etat::{Etat, Textes};
use crate::holo::{Argument, Bloc, Erreur, Programme, Valeur};
use crate::regles::pour_chaque_bloc;

pub type Listes = Vec<(String, Vec<String>)>;

thread_local! {
    /// Les listes de la page en cours de fabrication.
    static EN_COURS: std::cell::RefCell<Listes> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn mettre_en_cours(listes: Listes) {
    EN_COURS.with(|l| *l.borrow_mut() = listes);
}

pub fn en_cours() -> Listes {
    EN_COURS.with(|l| l.borrow().clone())
}

/// Le nombre d'éléments d'une liste, au plus, et la longueur d'un élément.
pub const ELEMENTS_MAX: usize = 100;
pub const ELEMENT_MAX: usize = 200;

/// Ce qu'on peut demander à une liste.
pub const DEMANDES: &[&str] = &["push", "remove", "clear"];

/// Les listes déclarées par la page, à leur départ : `State(tasks: [])`.
pub fn initiales(programme: &Programme) -> Listes {
    let Some(Valeur::Bloc(etat)) = programme.racine.argument("state").map(|a| &a.valeur) else { return Vec::new() };
    etat.arguments
        .iter()
        .filter_map(|a| match (&a.nom, &a.valeur) {
            (Some(nom), Valeur::Liste(elements)) => Some((nom.clone(), elements.iter().filter_map(|e| if let Valeur::Texte(t) = e { Some(t.clone()) } else { None }).collect())),
            _ => None,
        })
        .collect()
}

/// Vérifie une liste déclarée : des textes, cent au plus, de deux cents caractères au plus.
pub fn verifier_declaration(argument: &Argument) -> Result<(), Erreur> {
    let Valeur::Liste(elements) = &argument.valeur else { return Ok(()) };
    if elements.len() > ELEMENTS_MAX {
        return Err(Erreur { message: format!("une liste a au plus {ELEMENTS_MAX} éléments"), pos: argument.pos });
    }
    for element in elements {
        match element {
            Valeur::Texte(t) if t.chars().count() <= ELEMENT_MAX => {}
            Valeur::Texte(_) => return Err(Erreur { message: format!("un élément de liste fait au plus {ELEMENT_MAX} caractères"), pos: argument.pos }),
            _ => return Err(Erreur { message: "une liste contient des textes entre guillemets : State(tasks: [\"Acheter du pain\"])".into(), pos: argument.pos }),
        }
    }
    Ok(())
}

pub fn est_liste(programme: &Programme, nom: &str) -> bool {
    initiales(programme).iter().any(|(connu, _)| connu == nom)
}

/// `tasks=[Pain,Lait]` : chaque élément codé comme un texte, pour ne pas se mêler aux séparateurs.
pub fn ecrire(listes: &Listes) -> String {
    listes.iter().map(|(nom, elements)| format!("{nom}=[{}]", elements.iter().map(|e| crate::etat::coder(e)).collect::<Vec<_>>().join(","))).collect::<Vec<_>>().join(";")
}

/// Relit les listes d'un état. Seules celles que la page déclare, bornées.
pub fn relire(programme: &Programme, ecrit: &str) -> Listes {
    let mut listes = initiales(programme);
    for morceau in ecrit.split(';') {
        if let Some((nom, reste)) = morceau.split_once("=[") {
            let Some(dedans) = reste.strip_suffix(']') else { continue };
            if let Some((_, place)) = listes.iter_mut().find(|(connu, _)| connu == nom) {
                *place = dedans.split(',').filter(|e| !e.is_empty()).filter_map(crate::etat::decoder).map(|e| propre(&e)).filter(|e| !e.is_empty()).take(ELEMENTS_MAX).collect();
            }
        }
    }
    listes
}

fn propre(texte: &str) -> String {
    texte.chars().filter(|c| !c.is_control()).take(ELEMENT_MAX).collect()
}

/// Pour les conditions et les textes, une liste vaut son nombre d'éléments : `If(tasks, is: 0)`.
pub fn comptes(listes: &Listes) -> Etat {
    listes.iter().map(|(nom, elements)| (nom.clone(), elements.len() as u64)).collect()
}

/// Les répétitions dynamiques du fichier, `Repeat(over: tasks, …)`, avec le nom de leur liste.
pub fn repetitions(programme: &Programme) -> Vec<(&Bloc, String)> {
    let mut trouvees = Vec::new();
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if let (true, Some(Valeur::Nom(liste))) = (bloc.nom == "Repeat", bloc.argument("over").map(|a| &a.valeur)) {
            trouvees.push((bloc, liste.clone()));
        }
        Ok(())
    });
    trouvees
}

/// Les règles écrites dans une répétition dynamique : elles répondent au geste d'une ligne.
pub fn regles_de_ligne(programme: &Programme) -> Vec<(&Bloc, String)> {
    let mut regles = Vec::new();
    for (repetition, liste) in repetitions(programme) {
        if let Some(Valeur::Liste(dedans)) = repetition.argument("rules").map(|a| &a.valeur) {
            for regle in dedans {
                if let Valeur::Bloc(b) = regle {
                    regles.push((b, liste.clone()));
                }
            }
        }
    }
    regles
}

/// Les blocs du modèle d'une répétition dynamique (où `{item}` a un sens).
pub fn dans_un_modele(programme: &Programme) -> Vec<*const Bloc> {
    let mut blocs = Vec::new();
    for (repetition, _) in repetitions(programme) {
        for argument in &repetition.arguments {
            if argument.nom.as_deref() == Some("children") || argument.nom.as_deref() == Some("rules") {
                if let Valeur::Liste(dedans) = &argument.valeur {
                    for v in dedans {
                        if let Valeur::Bloc(b) = v {
                            let _ = pour_chaque_bloc(b, &mut |x| {
                                blocs.push(x as *const Bloc);
                                Ok(())
                            });
                        }
                    }
                }
            }
        }
    }
    blocs
}

/// Vérifie une demande faite à une liste ou à un texte. `dans_une_ligne` : la règle est écrite
/// dans une répétition dynamique, où `item` désigne l'élément de la ligne.
pub fn verifier_demande(demande: &Bloc, programme: &Programme, regle: &Bloc, dans_une_ligne: Option<&str>) -> Result<(), Erreur> {
    let erreur = |message: String| Erreur { message, pos: demande.pos };
    let Some((valeur, verbe)) = demande.nom.split_once('.') else { return Err(erreur("une demande s'écrit « tasks.push(task) »".into())) };
    if regle.nom != "On" {
        return Err(erreur(format!("« {valeur}.{verbe} » : une liste ou un texte ne change que par un geste du visiteur, dans une règle « On »")));
    }
    let textes = crate::etat::textes_initiaux(programme);
    let est_texte = |n: &str| textes.iter().any(|(connu, _)| connu == n);
    if est_liste(programme, valeur) {
        match (verbe, demande.arguments.as_slice()) {
            ("push", [Argument { nom: None, valeur: Valeur::Nom(t), .. }]) if est_texte(t) => Ok(()),
            ("push", [Argument { nom: None, valeur: Valeur::Texte(_), .. }]) => Ok(()),
            ("push", _) => Err(erreur(format!("« {valeur}.push » attend un texte de la page, ou un texte entre guillemets : {valeur}.push(task)"))),
            ("remove", [Argument { nom: None, valeur: Valeur::Nom(item), .. }]) if item == "item" => match dans_une_ligne {
                Some(liste) if liste == valeur => Ok(()),
                _ => Err(erreur(format!("« {valeur}.remove(item) » s'écrit dans les règles de Repeat(over: {valeur}, …) : il retire l'élément de la ligne touchée"))),
            },
            ("remove", _) => Err(erreur(format!("« {valeur}.remove » retire l'élément d'une ligne : {valeur}.remove(item), dans Repeat(over: {valeur}, rules: [ … ])"))),
            ("clear", []) => Ok(()),
            ("clear", _) => Err(erreur(format!("« {valeur}.clear() » vide la liste : rien entre les parenthèses"))),
            _ => Err(erreur(format!("demande inconnue « {verbe} » : pour une liste, on peut demander {}", DEMANDES.join(", ")))),
        }
    } else if est_texte(valeur) {
        match (verbe, demande.arguments.as_slice()) {
            ("set", [Argument { nom: None, valeur: Valeur::Texte(t), .. }]) if t.chars().count() <= crate::etat::TEXTE_MAX => Ok(()),
            ("set", [Argument { nom: None, valeur: Valeur::Nom(autre), .. }]) if est_texte(autre) || (autre == "item" && dans_une_ligne.is_some()) => Ok(()),
            _ => Err(erreur(format!("« {valeur} » est un texte : on demande seulement « {valeur}.set(\"\") », ou « {valeur}.set(autreTexte) »"))),
        }
    } else {
        Err(erreur(format!("aucune liste ni aucun texte ne s'appelle « {valeur} »")))
    }
}

/// Est-ce une demande faite à une liste ou à un texte ?
pub fn concerne(demande: &Bloc, programme: &Programme) -> bool {
    demande.nom.split_once('.').is_some_and(|(valeur, _)| est_liste(programme, valeur) || crate::etat::textes_initiaux(programme).iter().any(|(t, _)| t == valeur))
}

/// Le signal d'une ligne : `Done.tap@2` → (`Done.tap`, Some(2)).
pub fn signal_et_ligne(signal: &str) -> (&str, Option<usize>) {
    match signal.split_once('@') {
        Some((base, rang)) => (base, rang.parse().ok()),
        None => (signal, None),
    }
}

/// Fait les demandes aux listes et aux textes que ce signal déclenche, dans l'ordre écrit.
pub fn arbitrer(programme: &Programme, nombres: &Etat, textes: &Textes, listes: &Listes, signal: &str) -> (Textes, Listes) {
    let (base, ligne) = signal_et_ligne(signal);
    let (mut textes, mut listes) = (textes.clone(), listes.clone());
    let de_ligne = regles_de_ligne(programme);
    let _ = nombres;
    let _ = pour_chaque_bloc(&programme.racine, &mut |regle| {
        if regle.nom != "On" || !matches!(regle.arguments.iter().find(|a| a.nom.is_none()).map(|a| &a.valeur), Some(Valeur::Nom(s)) if s == base) {
            return Ok(());
        }
        // Une règle de ligne ne répond qu'au geste d'une ligne, et connaît son élément.
        let liste_de_ligne = de_ligne.iter().find(|(b, _)| std::ptr::eq(*b, regle)).map(|(_, l)| l.clone());
        if liste_de_ligne.is_some() != ligne.is_some() {
            return Ok(());
        }
        let element = match (&liste_de_ligne, ligne) {
            (Some(liste), Some(rang)) => listes.iter().find(|(n, _)| n == liste).and_then(|(_, e)| e.get(rang).cloned()),
            _ => None,
        };
        for demande in crate::etat::demandes_de(regle) {
            let Some((valeur, verbe)) = demande.nom.split_once('.') else { continue };
            let argument = demande.arguments.first().map(|a| &a.valeur);
            let lire_texte = |nom: &str, textes: &Textes| -> Option<String> {
                if nom == "item" {
                    return element.clone();
                }
                textes.iter().find(|(n, _)| n == nom).map(|(_, t)| t.clone())
            };
            if let Some((_, elements)) = listes.iter_mut().find(|(n, _)| n == valeur) {
                match (verbe, argument) {
                    ("push", Some(Valeur::Nom(t))) => {
                        if let Some(texte) = lire_texte(t, &textes).map(|t| propre(t.trim())).filter(|t| !t.is_empty()) {
                            if elements.len() < ELEMENTS_MAX {
                                elements.push(texte);
                            }
                        }
                    }
                    ("push", Some(Valeur::Texte(t))) if elements.len() < ELEMENTS_MAX && !t.is_empty() => elements.push(propre(t)),
                    ("remove", _) => {
                        if let (Some(rang), true) = (ligne, liste_de_ligne.as_deref() == Some(valeur)) {
                            if rang < elements.len() {
                                elements.remove(rang);
                            }
                        }
                    }
                    ("clear", _) => elements.clear(),
                    _ => {}
                }
            } else if verbe == "set" {
                let nouveau = match argument {
                    Some(Valeur::Texte(t)) => Some(t.clone()),
                    Some(Valeur::Nom(autre)) => lire_texte(autre, &textes),
                    _ => None,
                };
                if let (Some(nouveau), Some((_, place))) = (nouveau, textes.iter_mut().find(|(n, _)| n == valeur)) {
                    *place = nouveau.chars().filter(|c| *c == '\n' || !c.is_control()).take(crate::etat::TEXTE_MAX).collect();
                }
            }
        }
        Ok(())
    });
    (textes, listes)
}

#[cfg(test)]
mod tests {
    #[test]
    fn une_liste_change_par_des_gestes() {
        let source = "Page(state: State(task: \"\", tasks: [\"Pain\"]), children: [ Input(value: task, label: \"T\"), Button(name: Add, text: \"+\"), Button(name: Empty, text: \"0\"), P(\"{tasks} tâche(s)\"), If(tasks, is: 0, children: [ P(\"Rien\") ]), Repeat(over: tasks, children: [ Row(children: [ Text(\"{item}\"), Button(name: Done, text: \"x\") ]) ], rules: [ On(Done.tap, effect: tasks.remove(item)) ]) ], rules: [ On(Add.tap, effect: [tasks.push(task), task.set(\"\")]), On(Empty.tap, effect: tasks.clear()) ])";
        let depart = crate::etat_initial(source);
        assert_eq!(depart, "task=';tasks=[Pain]");
        let ecrit = crate::saisir(source, &depart, "task", "Lait, frais");
        let apres = crate::arbitrer(source, &ecrit, "Add.tap");
        assert_eq!(apres, "task=';tasks=[Pain,Lait%2C%20frais]");
        // Le geste d'une ligne retire son élément ; sans ligne, la règle de ligne ne répond pas.
        assert_eq!(crate::arbitrer(source, &apres, "Done.tap@0"), "task=';tasks=[Lait%2C%20frais]");
        assert_eq!(crate::arbitrer(source, &apres, "Done.tap"), apres);
        assert_eq!(crate::arbitrer(source, &apres, "Empty.tap"), "task=';tasks=[]");
        // La page fabriquée montre une ligne par élément ; le compte, et la condition.
        let html = crate::vue_a_plat(source, "").unwrap();
        assert!(html.contains("<div class=\"holo-Liste\" data-liste=\"tasks\"><div class=\"holo-ligne\" data-rang=\"0\"><div class=\"holo-Row\" style=\"\"><div class=\"holo-Text\">Pain</div><button type=\"button\" class=\"holo-Button\" data-name=\"Done\">x</button></div></div></div>"), "{html}");
        assert!(html.contains("<span data-state=\"tasks\">1</span> tâche(s)"), "{html}");
        assert_eq!(crate::liste_html(source, "", &apres, "tasks").matches("data-rang").count(), 2);
        // Un texte saisi par le visiteur ne devient jamais une balise, ni une valeur montrée.
        let piege = crate::arbitrer(source, &crate::saisir(source, &depart, "task", "<b>{task}</b>"), "Add.tap");
        let lignes = crate::liste_html(source, "", &piege, "tasks");
        assert!(lignes.contains("&lt;b&gt;{task}&lt;/b&gt;") && !lignes.contains("data-state"), "{lignes}");
        // Ce qui est refusé.
        for (source, message) in [
            ("Page(state: State(l: [1]), children: [])", "une liste contient des textes"),
            ("Page(state: State(l: [], n: 0), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: l.remove(item)) ])", "s'écrit dans les règles de Repeat"),
            ("Page(state: State(l: [], n: 0), children: [], rules: [ Every(1s, effect: l.clear()) ])", "par un geste du visiteur"),
            ("Page(state: State(l: []), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: l.push(3)) ])", "attend un texte"),
            ("Page(state: State(l: []), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: l.add(1)) ])", "pour une liste, on peut demander"),
            ("Page(state: State(t: \"\"), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: t.add(1)) ])", "est un texte"),
            ("Page(children: [ Repeat(over: rien, children: [ P(\"x\") ]) ])", "aucune liste ne s'appelle « rien »"),
        ] {
            let erreur = crate::verifier_page(source).err().or_else(|| crate::vue_a_plat(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }
}
