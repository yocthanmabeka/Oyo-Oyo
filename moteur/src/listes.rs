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
//!
//! Depuis ADR-051, un élément peut avoir des champs, comme un `Item` de `Repeat(items:)` :
//! `State(articles: [ Item(title: "Sunrise", price: 12000) ])`, montrés par `{item.title}` et
//! `{item.price:cents}`. Une telle liste se remplit aussi par `Data` (un tableau d'objets JSON) et
//! par `articles.push(Item(title: task, price: 0))`.

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

/// Le nombre d'éléments d'une liste, au plus, et la longueur d'un élément (ou d'un champ).
pub const ELEMENTS_MAX: usize = 100;
pub const ELEMENT_MAX: usize = 200;
/// Le nombre de champs d'un élément, au plus (ADR-051).
pub const CHAMPS_MAX: usize = 16;

/// Un élément à champs commence par ce signe, que le visiteur ne peut pas écrire : les signes
/// invisibles sont retirés de tout ce qu'il saisit. Suivent les champs, `title=Sunrise&price=12000`,
/// chaque valeur codée par `coder`.
pub const FICHE: char = '\u{1d}';

/// Les champs d'un élément, dans l'ordre écrit ; aucun pour un élément de texte.
pub fn champs(element: &str) -> Vec<(String, String)> {
    let Some(reste) = element.strip_prefix(FICHE) else { return Vec::new() };
    reste
        .split('&')
        .filter_map(|c| c.split_once('='))
        .filter(|(nom, _)| est_nom_de_champ(nom))
        .filter_map(|(nom, code)| crate::etat::decoder(code).map(|v| (nom.to_string(), propre(&v))))
        .take(CHAMPS_MAX)
        .collect()
}

/// Écrit un élément à champs.
pub fn fiche(champs: &[(String, String)]) -> String {
    let mut sortie = String::from(FICHE);
    sortie.push_str(&champs.iter().take(CHAMPS_MAX).map(|(nom, valeur)| format!("{nom}={}", crate::etat::coder(&propre(valeur)))).collect::<Vec<_>>().join("&"));
    sortie
}

/// Ce qu'on montre pour `{item}` : le texte d'un élément, ou le premier champ d'un élément à champs.
pub fn texte_de(element: &str) -> String {
    if element.starts_with(FICHE) {
        return champs(element).into_iter().next().map(|(_, v)| v).unwrap_or_default();
    }
    element.to_string()
}

fn est_nom_de_champ(nom: &str) -> bool {
    nom.starts_with(|c: char| c.is_ascii_lowercase()) && nom.chars().all(|c| c.is_ascii_alphanumeric()) && nom.len() <= 40 && nom != "key"
}

/// Dans une ligne, `If(item.done, is: 1, children: [ … ], else: [ … ])` choisit ce qu'il montre
/// d'après le champ de l'élément ; `If(item, is: "…")` d'après le texte d'un élément de texte
/// (ADR-057). Les `If` qui regardent l'élément sont remplacés par la branche choisie : le reste du
/// moteur ne voit que des blocs ordinaires. Un champ se compare à un nombre (`is`, `not`, `over`,
/// `under`) ou à un texte (`is`, `not`).
pub fn choisir_selon_l_element(valeur: &mut Valeur, champs: &[(String, String)], texte: &str) {
    match valeur {
        Valeur::Liste(elements) => {
            let mut poses = Vec::with_capacity(elements.len());
            for mut element in std::mem::take(elements) {
                if let Valeur::Bloc(bloc) = &element {
                    if let Some(choisis) = branche_de_l_element(bloc, champs, texte) {
                        for mut choisi in choisis {
                            choisir_selon_l_element(&mut choisi, champs, texte);
                            poses.push(choisi);
                        }
                        continue;
                    }
                }
                choisir_selon_l_element(&mut element, champs, texte);
                poses.push(element);
            }
            *elements = poses;
        }
        Valeur::Bloc(bloc) => bloc.arguments.iter_mut().for_each(|a| choisir_selon_l_element(&mut a.valeur, champs, texte)),
        _ => {}
    }
}

/// Le sujet d'un `If` qui regarde l'élément : `item` ou `item.done`.
pub fn sujet_de_l_element(bloc: &Bloc) -> Option<&str> {
    if bloc.nom != "If" {
        return None;
    }
    match bloc.arguments.first() {
        Some(Argument { nom: None, valeur: Valeur::Nom(sujet), .. }) if sujet == "item" || sujet.starts_with("item.") => Some(sujet),
        _ => None,
    }
}

fn branche_de_l_element(bloc: &Bloc, champs: &[(String, String)], texte: &str) -> Option<Vec<Valeur>> {
    let sujet = sujet_de_l_element(bloc)?;
    let brut = match sujet.strip_prefix("item.") {
        Some(champ) => champs.iter().find(|(c, _)| c == champ).map(|(_, v)| v.clone()).unwrap_or_default(),
        None => texte_de(texte),
    };
    let nombre = brut.parse::<u64>().ok();
    let vrai = bloc.arguments[1..].iter().all(|a| match (a.nom.as_deref(), &a.valeur) {
        (Some("is"), Valeur::Entier(n)) => nombre == Some(*n),
        (Some("not"), Valeur::Entier(n)) => nombre != Some(*n),
        (Some("over"), Valeur::Entier(n)) => nombre.is_some_and(|v| v > *n),
        (Some("under"), Valeur::Entier(n)) => nombre.is_some_and(|v| v < *n),
        (Some("is"), Valeur::Texte(t)) => brut == *t,
        (Some("not"), Valeur::Texte(t)) => brut != *t,
        _ => true,
    });
    let liste = |nom: &str| match bloc.argument(nom).map(|a| &a.valeur) {
        Some(Valeur::Liste(l)) => l.clone(),
        _ => Vec::new(),
    };
    Some(if vrai {
        if bloc.argument("rules").is_some() { liste("rules") } else { liste("children") }
    } else {
        liste("else")
    })
}

/// Vérifie un `If` qui regarde l'élément d'une ligne : le champ existe, les comparaisons sont
/// bien écrites.
pub fn verifier_si_de_l_element(bloc: &Bloc, programme: &Programme, liste: Option<&str>) -> Result<(), Erreur> {
    let Some(sujet) = sujet_de_l_element(bloc) else { return Ok(()) };
    let erreur = |message: String| Err(Erreur { message, pos: bloc.pos });
    let Some(liste) = liste else {
        return erreur(format!("« If({sujet}, …) » regarde l'élément d'une ligne : il s'écrit dans Repeat(…, children: [ … ])"));
    };
    if let Some(champ) = sujet.strip_prefix("item.") {
        match sorte(programme, liste) {
            Some(Sorte::Textes) => return erreur(format!("les éléments de « {liste} » sont des textes, sans champs : If(item, is: \"…\")")),
            Some(Sorte::Fiches(champs)) if !champs.iter().any(|c| c == champ) => return erreur(format!("les éléments de « {liste} » n'ont pas de champ « {champ} » ; champs : {}", champs.join(", "))),
            _ => {}
        }
    }
    let mut comparaisons = 0;
    for a in &bloc.arguments[1..] {
        match (a.nom.as_deref(), &a.valeur) {
            (Some("children" | "else" | "rules" | "name"), _) => {}
            (Some("is" | "not"), Valeur::Entier(_) | Valeur::Texte(_)) | (Some("over" | "under"), Valeur::Entier(_)) => comparaisons += 1,
            (Some(mot @ ("is" | "not" | "over" | "under")), _) => return erreur(format!("« If({sujet}, {mot}: …) » attend un nombre entier{}", if matches!(mot, "is" | "not") { ", ou un texte entre guillemets" } else { "" })),
            (Some(mot), _) => return erreur(format!("« If » n'a pas de paramètre « {mot} » ; paramètres possibles : is, not, over, under, children, else")),
            (None, _) => return erreur(format!("« If({sujet}, …) » : chaque comparaison est nommée, is: 1")),
        }
    }
    if comparaisons == 0 {
        return erreur(format!("« If({sujet}, …) » attend une comparaison : If({sujet}, is: 1, children: [ … ])"));
    }
    Ok(())
}

/// La sorte d'une liste, d'après sa déclaration.
#[derive(Debug, Clone, PartialEq)]
pub enum Sorte {
    /// `State(tasks: ["Pain"])` : des textes.
    Textes,
    /// `State(articles: [ Item(title: "…", price: 0) ])` : des éléments à champs, ceux-là.
    Fiches(Vec<String>),
    /// `State(articles: [])` : vide au départ, des textes ou des éléments à champs.
    Libre,
}

/// La sorte d'une liste déclarée.
pub fn sorte(programme: &Programme, nom: &str) -> Option<Sorte> {
    let Some(Valeur::Bloc(etat)) = programme.racine.argument("state").map(|a| &a.valeur) else { return None };
    let Some(Valeur::Liste(elements)) = etat.argument(nom).map(|a| &a.valeur) else { return None };
    Some(match elements.first() {
        None => Sorte::Libre,
        Some(Valeur::Bloc(item)) => Sorte::Fiches(item.arguments.iter().filter_map(|a| a.nom.clone()).collect()),
        Some(_) => Sorte::Textes,
    })
}

/// Ce qu'on peut demander à une liste.
pub const DEMANDES: &[&str] = &["push", "remove", "clear"];

/// Les listes déclarées par la page, à leur départ : `State(tasks: [])`.
pub fn initiales(programme: &Programme) -> Listes {
    let Some(Valeur::Bloc(etat)) = programme.racine.argument("state").map(|a| &a.valeur) else { return Vec::new() };
    etat.arguments
        .iter()
        .filter_map(|a| match (&a.nom, &a.valeur) {
            (Some(nom), Valeur::Liste(elements)) => Some((nom.clone(), elements.iter().filter_map(element_ecrit).collect())),
            _ => None,
        })
        .collect()
}

/// Un élément écrit dans le fichier : un texte, ou un `Item(…)` dont les champs sont des textes
/// ou des nombres entiers.
fn element_ecrit(valeur: &Valeur) -> Option<String> {
    match valeur {
        Valeur::Texte(t) => Some(t.clone()),
        Valeur::Bloc(item) if item.nom == "Item" => Some(fiche(
            &item
                .arguments
                .iter()
                .filter_map(|a| match (&a.nom, &a.valeur) {
                    (Some(n), Valeur::Texte(t)) => Some((n.clone(), t.clone())),
                    (Some(n), Valeur::Entier(e)) => Some((n.clone(), e.to_string())),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        )),
        _ => None,
    }
}

/// Vérifie une liste déclarée : des textes, ou des `Item(…)` qui ont tous les mêmes champs ;
/// cent éléments au plus, de deux cents caractères au plus.
pub fn verifier_declaration(argument: &Argument) -> Result<(), Erreur> {
    let Valeur::Liste(elements) = &argument.valeur else { return Ok(()) };
    let refus = |message: String| Err(Erreur { message, pos: argument.pos });
    if elements.len() > ELEMENTS_MAX {
        return refus(format!("une liste a au plus {ELEMENTS_MAX} éléments"));
    }
    let mut premiers: Option<Vec<String>> = None;
    for element in elements {
        match element {
            Valeur::Texte(t) if t.chars().count() <= ELEMENT_MAX && premiers.is_none() && !elements.iter().any(|e| matches!(e, Valeur::Bloc(_))) => {}
            Valeur::Texte(_) if elements.iter().any(|e| matches!(e, Valeur::Bloc(_))) => return refus("une liste contient des textes ou des Item(…), pas les deux".into()),
            Valeur::Texte(_) => return refus(format!("un élément de liste fait au plus {ELEMENT_MAX} caractères")),
            Valeur::Bloc(item) if item.nom == "Item" => {
                let mut noms = Vec::new();
                for a in &item.arguments {
                    let Some(nom) = a.nom.as_deref() else { return refus("chaque champ d'un « Item » est nommé : Item(title: \"Sunrise\", price: 120)".into()) };
                    if !est_nom_de_champ(nom) {
                        return refus(format!("« {nom} » : un champ s'écrit comme une valeur, en minuscules, les mots joints par une majuscule (ADR-037)"));
                    }
                    match &a.valeur {
                        Valeur::Texte(t) if t.chars().count() <= ELEMENT_MAX => {}
                        Valeur::Entier(_) => {}
                        _ => return refus(format!("le champ « {nom} » attend un texte (deux cents caractères au plus) ou un nombre entier")),
                    }
                    if noms.contains(&nom.to_string()) {
                        return refus(format!("le champ « {nom} » est donné deux fois"));
                    }
                    noms.push(nom.to_string());
                }
                if noms.is_empty() || noms.len() > CHAMPS_MAX {
                    return refus(format!("un « Item » a de 1 à {CHAMPS_MAX} champs"));
                }
                match &premiers {
                    Some(attendus) if *attendus != noms => return refus(format!("les éléments d'une liste ont les mêmes champs, dans le même ordre : {}", attendus.join(", "))),
                    Some(_) => {}
                    None => premiers = Some(noms),
                }
            }
            _ => return refus("une liste contient des textes entre guillemets, State(tasks: [\"Acheter du pain\"]), ou des Item(…), State(articles: [ Item(title: \"Sunrise\") ])".into()),
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
                *place = dedans.split(',').filter(|e| !e.is_empty()).filter_map(crate::etat::decoder).map(|e| propre_element(&e)).filter(|e| !e.is_empty()).take(ELEMENTS_MAX).collect();
            }
        }
    }
    listes
}

fn propre(texte: &str) -> String {
    texte.chars().filter(|c| !c.is_control()).take(ELEMENT_MAX).collect()
}

/// Un élément relu : un texte nettoyé, ou un élément à champs réécrit champ par champ.
fn propre_element(element: &str) -> String {
    if element.starts_with(FICHE) {
        let lus = champs(element);
        return if lus.is_empty() { String::new() } else { fiche(&lus) };
    }
    propre(element)
}

/// Les données venues du serveur (`Data`, ADR-051) : un tableau de textes remplit une liste de
/// textes ; un tableau d'objets remplit une liste à champs (seuls les champs déclarés sont repris,
/// ceux qui manquent valent "" ou 0). Une liste que la page ne déclare pas est laissée de côté.
pub fn recevoir(programme: &Programme, listes: &Listes, json: &str) -> Listes {
    let mut listes = listes.clone();
    if crate::etat::source_de_donnees(programme).ok().flatten().is_none() || json.len() > crate::etat::DONNEES_OCTETS {
        return listes;
    }
    let Some(Json::Objet(cles)) = Json::lire(json) else { return listes };
    for (cle, valeur) in cles {
        let (Json::Tableau(elements), Some(sorte)) = (valeur, sorte(programme, &cle)) else { continue };
        let Some((_, place)) = listes.iter_mut().find(|(n, _)| *n == cle) else { continue };
        let mut nouveaux = Vec::new();
        for element in elements.into_iter().take(ELEMENTS_MAX) {
            match (element, &sorte) {
                (Json::Texte(t), Sorte::Textes | Sorte::Libre) => nouveaux.push(propre(&t)),
                (Json::Objet(champs_lus), Sorte::Fiches(_) | Sorte::Libre) => {
                    let pris: Vec<(String, String)> = match &sorte {
                        Sorte::Fiches(attendus) => attendus
                            .iter()
                            .map(|n| (n.clone(), champs_lus.iter().find(|(c, _)| c == n).map(|(_, v)| v.en_texte()).unwrap_or_default()))
                            .collect(),
                        _ => champs_lus.iter().filter(|(n, _)| est_nom_de_champ(n)).map(|(n, v)| (n.clone(), v.en_texte())).collect(),
                    };
                    if !pris.is_empty() {
                        nouveaux.push(fiche(&pris));
                    }
                }
                _ => {}
            }
        }
        *place = nouveaux.into_iter().filter(|e| !e.is_empty()).collect();
    }
    listes
}

/// Une valeur JSON, juste ce qu'il faut pour des données de page : textes, nombres entiers,
/// tableaux et objets, trois niveaux au plus. Le reste ne donne rien.
#[derive(Debug, Clone, PartialEq)]
enum Json {
    Texte(String),
    Nombre(u64),
    Tableau(Vec<Json>),
    Objet(Vec<(String, Json)>),
    Autre,
}

impl Json {
    fn lire(texte: &str) -> Option<Json> {
        let t: Vec<char> = texte.chars().collect();
        let mut i = 0;
        let valeur = Self::valeur(&t, &mut i, 0)?;
        Self::blancs(&t, &mut i);
        (i == t.len()).then_some(valeur)
    }

    fn en_texte(&self) -> String {
        match self {
            Json::Texte(t) => t.clone(),
            Json::Nombre(n) => n.to_string(),
            _ => String::new(),
        }
    }

    fn blancs(t: &[char], i: &mut usize) {
        while t.get(*i).is_some_and(|c| c.is_whitespace()) {
            *i += 1;
        }
    }

    fn valeur(t: &[char], i: &mut usize, profondeur: usize) -> Option<Json> {
        Self::blancs(t, i);
        match *t.get(*i)? {
            '"' => Self::texte(t, i).map(Json::Texte),
            '[' if profondeur < 3 => {
                *i += 1;
                let mut elements = Vec::new();
                loop {
                    Self::blancs(t, i);
                    if t.get(*i) == Some(&']') {
                        *i += 1;
                        return Some(Json::Tableau(elements));
                    }
                    elements.push(Self::valeur(t, i, profondeur + 1)?);
                    Self::blancs(t, i);
                    match *t.get(*i)? {
                        ',' => *i += 1,
                        ']' => {}
                        _ => return None,
                    }
                }
            }
            '{' if profondeur < 3 => {
                *i += 1;
                let mut cles = Vec::new();
                loop {
                    Self::blancs(t, i);
                    if t.get(*i) == Some(&'}') {
                        *i += 1;
                        return Some(Json::Objet(cles));
                    }
                    let cle = Self::texte(t, i)?;
                    Self::blancs(t, i);
                    if t.get(*i) != Some(&':') {
                        return None;
                    }
                    *i += 1;
                    let valeur = Self::valeur(t, i, profondeur + 1)?;
                    cles.push((cle, valeur));
                    Self::blancs(t, i);
                    match *t.get(*i)? {
                        ',' => *i += 1,
                        '}' => {}
                        _ => return None,
                    }
                }
            }
            c if c.is_ascii_digit() || c == '-' => {
                let debut = *i;
                while t.get(*i).is_some_and(|c| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')) {
                    *i += 1;
                }
                let ecrit: String = t[debut..*i].iter().collect();
                Some(ecrit.parse::<u64>().map_or(Json::Autre, Json::Nombre))
            }
            _ => {
                for mot in ["true", "false", "null"] {
                    if t[*i..].iter().take(mot.len()).copied().eq(mot.chars()) {
                        *i += mot.len();
                        return Some(match mot {
                            "true" => Json::Nombre(1),
                            "false" => Json::Nombre(0),
                            _ => Json::Autre,
                        });
                    }
                }
                None
            }
        }
    }

    fn texte(t: &[char], i: &mut usize) -> Option<String> {
        if t.get(*i) != Some(&'"') {
            return None;
        }
        *i += 1;
        let mut sortie = String::new();
        loop {
            let c = *t.get(*i)?;
            *i += 1;
            match c {
                '"' => return Some(sortie),
                '\\' => {
                    let e = *t.get(*i)?;
                    *i += 1;
                    match e {
                        'n' => sortie.push('\n'),
                        't' => sortie.push(' '),
                        'u' => {
                            let hexa: String = t.get(*i..*i + 4)?.iter().collect();
                            *i += 4;
                            sortie.push(char::from_u32(u32::from_str_radix(&hexa, 16).ok()?).unwrap_or('?'));
                        }
                        autre => sortie.push(autre),
                    }
                }
                autre => sortie.push(autre),
            }
        }
    }
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
    modeles_et_listes(programme).into_iter().map(|(b, _)| b).collect()
}

/// Les blocs du modèle d'une répétition dynamique, avec le nom de la liste qu'elle montre.
pub fn modeles_et_listes(programme: &Programme) -> Vec<(*const Bloc, String)> {
    let mut blocs = Vec::new();
    for (repetition, liste) in repetitions(programme) {
        for argument in &repetition.arguments {
            if argument.nom.as_deref() == Some("children") || argument.nom.as_deref() == Some("rules") {
                if let Valeur::Liste(dedans) = &argument.valeur {
                    for v in dedans {
                        if let Valeur::Bloc(b) = v {
                            let _ = pour_chaque_bloc(b, &mut |x| {
                                blocs.push((x as *const Bloc, liste.clone()));
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
    // `item.done.set(1)` : changer un champ de l'élément de la ligne touchée (ADR-057).
    if valeur == "item" {
        let Some(liste) = dans_une_ligne else {
            return Err(erreur(format!("« item.{verbe} » change l'élément d'une ligne : il s'écrit dans les règles de Repeat(over: …)")));
        };
        let Some((champ, operation)) = verbe.split_once('.') else {
            return Err(erreur("pour changer un champ de l'élément : item.done.set(1), item.likes.add(1)".into()));
        };
        match sorte(programme, liste) {
            Some(Sorte::Textes) => return Err(erreur(format!("les éléments de « {liste} » sont des textes, sans champs"))),
            Some(Sorte::Fiches(champs)) if !champs.iter().any(|c| c == champ) => return Err(erreur(format!("les éléments de « {liste} » n'ont pas de champ « {champ} » ; champs : {}", champs.join(", ")))),
            _ => {}
        }
        let nombres = crate::etat::initial(programme).unwrap_or_default();
        return match (operation, demande.arguments.as_slice()) {
            ("set", [Argument { nom: None, valeur: Valeur::Entier(_) | Valeur::Texte(_), .. }]) => Ok(()),
            ("set", [Argument { nom: None, valeur: Valeur::Nom(n), .. }]) if est_texte(n) || nombres.iter().any(|(c, _)| c == n) => Ok(()),
            ("add" | "sub", [Argument { nom: None, valeur: Valeur::Entier(_), .. }]) => Ok(()),
            _ => Err(erreur(format!("« item.{champ}.{operation} » : on demande set (un nombre, un texte, ou une valeur de la page), add ou sub (un nombre entier)"))),
        };
    }
    if est_liste(programme, valeur) {
        let la_sorte = sorte(programme, valeur).unwrap_or(Sorte::Libre);
        let nombres = crate::etat::initial(programme).unwrap_or_default();
        let est_nombre = |n: &str| nombres.iter().any(|(connu, _)| connu == n);
        match (verbe, demande.arguments.as_slice()) {
            ("push", [Argument { nom: None, valeur: Valeur::Nom(t), .. }]) if est_texte(t) && !matches!(la_sorte, Sorte::Fiches(_)) => Ok(()),
            ("push", [Argument { nom: None, valeur: Valeur::Texte(_), .. }]) if !matches!(la_sorte, Sorte::Fiches(_)) => Ok(()),
            ("push", [Argument { nom: None, valeur: Valeur::Bloc(item), .. }]) if item.nom == "Item" && la_sorte != Sorte::Textes => {
                let mut noms = Vec::new();
                for a in &item.arguments {
                    let Some(nom) = a.nom.as_deref() else { return Err(erreur("chaque champ d'un « Item » est nommé : Item(title: task, price: 0)".into())) };
                    match &a.valeur {
                        Valeur::Texte(t) if t.chars().count() <= ELEMENT_MAX => {}
                        Valeur::Entier(_) => {}
                        Valeur::Nom(n) if est_texte(n) || est_nombre(n) => {}
                        _ => return Err(erreur(format!("le champ « {nom} » prend un texte, un nombre, ou le nom d'une valeur de la page"))),
                    }
                    noms.push(nom.to_string());
                }
                match &la_sorte {
                    Sorte::Fiches(attendus) if *attendus != noms => Err(erreur(format!("« {valeur}.push(Item(…)) » : les éléments de « {valeur} » ont les champs {}, dans cet ordre", attendus.join(", ")))),
                    _ if noms.is_empty() || noms.len() > CHAMPS_MAX => Err(erreur(format!("un « Item » a de 1 à {CHAMPS_MAX} champs"))),
                    _ => Ok(()),
                }
            }
            ("push", _) if matches!(la_sorte, Sorte::Fiches(_)) => Err(erreur(format!("« {valeur} » a des éléments à champs : {valeur}.push(Item(…))"))),
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
    demande.nom.split_once('.').is_some_and(|(valeur, _)| valeur == "item" || est_liste(programme, valeur) || crate::etat::textes_initiaux(programme).iter().any(|(t, _)| t == valeur))
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
                    return element.as_deref().map(texte_de);
                }
                textes.iter().find(|(n, _)| n == nom).map(|(_, t)| t.clone())
            };
            // `item.done.set(1)` : le champ de l'élément de la ligne touchée (ADR-057).
            if valeur == "item" {
                if let (Some(liste), Some(rang), Some((champ, operation))) = (&liste_de_ligne, ligne, verbe.split_once('.')) {
                    let nouveau_texte = match argument {
                        Some(Valeur::Nom(n)) => lire_texte(n, &textes).or_else(|| nombres.iter().find(|(c, _)| c == n).map(|(_, v)| v.to_string())),
                        _ => None,
                    };
                    if let Some((_, elements)) = listes.iter_mut().find(|(n, _)| n == liste) {
                        if let Some(element) = elements.get_mut(rang) {
                            let mut champs_de = champs(element);
                            if !champs_de.iter().any(|(c, _)| c == champ) {
                                champs_de.push((champ.to_string(), String::new()));
                            }
                            if let Some((_, v)) = champs_de.iter_mut().find(|(c, _)| c == champ) {
                                let present = v.parse::<u64>().unwrap_or(0);
                                *v = match (operation, argument) {
                                    ("set", Some(Valeur::Entier(n))) => n.to_string(),
                                    ("set", Some(Valeur::Texte(t))) => t.clone(),
                                    ("set", Some(Valeur::Nom(_))) => nouveau_texte.unwrap_or_default(),
                                    ("add", Some(Valeur::Entier(n))) => present.saturating_add(*n).min(crate::etat::VALEUR_MAX).to_string(),
                                    ("sub", Some(Valeur::Entier(n))) => present.saturating_sub(*n).to_string(),
                                    _ => v.clone(),
                                };
                            }
                            *element = fiche(&champs_de);
                        }
                    }
                }
                continue;
            }
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
                    ("push", Some(Valeur::Bloc(item))) if item.nom == "Item" && elements.len() < ELEMENTS_MAX => {
                        let champs_poses: Vec<(String, String)> = item
                            .arguments
                            .iter()
                            .filter_map(|a| {
                                let valeur = match &a.valeur {
                                    Valeur::Texte(t) => t.clone(),
                                    Valeur::Entier(e) => e.to_string(),
                                    Valeur::Nom(n) => lire_texte(n, &textes).or_else(|| nombres.iter().find(|(c, _)| c == n).map(|(_, v)| v.to_string())).unwrap_or_default(),
                                    _ => return None,
                                };
                                Some((a.nom.clone()?, valeur.trim().to_string()))
                            })
                            .collect();
                        // Un élément dont tous les textes sont vides n'est pas ajouté, comme un texte vide.
                        if champs_poses.iter().any(|(_, v)| !v.is_empty() && v != "0") {
                            elements.push(fiche(&champs_poses));
                        }
                    }
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
        assert!(html.contains("<div class=\"holo-Liste\" data-liste=\"tasks\"><div class=\"holo-ligne\" data-rang=\"0\" data-cle=\"") && html.contains("-0\"><div class=\"holo-Row\" style=\"\"><div class=\"holo-Text\">Pain</div><button type=\"button\" class=\"holo-Button\" data-name=\"Done\">x</button></div></div></div>"), "{html}");
        // La clé d'un élément ne change pas quand un autre élément est retiré avant lui.
        let cle_de = |etat: &str, texte: &str| crate::liste_html(source, "", etat, "tasks").split("data-cle=\"").skip(1).find(|l| l.contains(texte)).map(|l| l[..l.find('"').unwrap()].to_string());
        assert_eq!(cle_de(&apres, "Lait"), cle_de(&crate::arbitrer(source, &apres, "Done.tap@0"), "Lait"));
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

    #[test]
    fn une_liste_a_champs_se_montre_se_remplit_et_vient_du_serveur() {
        let source = r#"Page(
  state: State(name: "", price: 0, articles: [ Item(title: "Sunrise", price: 12000, image: "sunrise.png"), Item(title: "River", price: 900, image: "river.png") ]),
  data: Data(from: "stock.json"),
  children: [
    Input(value: name, label: "Name"), Input(value: price, label: "Price"), Button(name: Add, text: "Add"),
    P("{articles} article(s)"),
    Repeat(over: articles, children: [ Column(children: [ Image(source: item.image, alt: "{item.title}"), Text("{item.title} : {item.price:cents} euros"), Button(name: Remove, text: "x") ]) ],
           rules: [ On(Remove.tap, effect: articles.remove(item)) ]),
  ],
  rules: [ On(Add.tap, effect: [articles.push(Item(title: name, price: price, image: "new.png")), name.set("")]) ],
)"#;
        crate::verifier_page(source).unwrap();
        let depart = crate::etat_initial(source);
        let html = crate::vue_a_plat(source, "").unwrap();
        assert!(html.contains("<img class=\"holo-Image\" src=\"sunrise.png\" alt=\"Sunrise\">"), "{html}");
        assert!(html.contains("Sunrise : 120,00 euros") && html.contains("River : 9,00 euros"), "{html}");
        assert!(html.contains("<span data-state=\"articles\">2</span> article(s)"), "{html}");
        // Ajouter un élément à champs, depuis ce que le visiteur a saisi ; puis en retirer un.
        let ecrit = crate::saisir(source, &crate::saisir(source, &depart, "name", "<Night>"), "price", "60");
        let apres = crate::arbitrer(source, &ecrit, "Add.tap");
        let lignes = crate::liste_html(source, "", &apres, "articles");
        assert_eq!(lignes.matches("data-rang").count(), 3, "{lignes}");
        assert!(lignes.contains("&lt;Night&gt; : 0,60 euros"), "{lignes}");
        let moins = crate::arbitrer(source, &apres, "Remove.tap@0");
        assert_eq!(crate::liste_html(source, "", &moins, "articles").matches("data-rang").count(), 2);
        // Les données du serveur remplacent la liste : seuls les champs déclarés sont repris.
        let recu = crate::recevoir(source, &depart, r#"{"articles": [ {"title": "Forest", "price": 4500, "image": "f.png", "secret": "x"} ]}"#);
        let lignes = crate::liste_html(source, "", &recu, "articles");
        assert!(lignes.contains("Forest : 45,00 euros") && lignes.contains("src=\"f.png\"") && !lignes.contains("secret"), "{lignes}");
        assert_eq!(lignes.matches("data-rang").count(), 1);
        // Une image venue du serveur reste dans le dossier de la page.
        let piege = crate::recevoir(source, &depart, r#"{"articles": [ {"title": "x", "price": 1, "image": "javascript:alert(1)"} ]}"#);
        assert!(!crate::liste_html(source, "", &piege, "articles").contains("javascript"));
    }

    #[test]
    fn une_liste_de_textes_vient_du_serveur() {
        let source = "Page(state: State(news: []), data: Data(from: \"news.json\"), children: [ Repeat(over: news, children: [ P(\"{item}\") ]) ])";
        let recu = crate::recevoir(source, &crate::etat_initial(source), r#"{"news": ["Open today", "New paintings"]}"#);
        let lignes = crate::liste_html(source, "", &recu, "news");
        assert!(lignes.contains("Open today") && lignes.contains("New paintings"), "{lignes}");
    }

    #[test]
    fn ce_qui_est_refuse_pour_les_listes_a_champs() {
        for (source, message) in [
            ("Page(state: State(l: [ Item(a: 1), Item(b: 2) ]), children: [])", "les mêmes champs"),
            ("Page(state: State(l: [ Item(a: 1), \"x\" ]), children: [])", "pas les deux"),
            ("Page(state: State(l: [ Item(a: [1]) ]), children: [])", "attend un texte"),
            ("Page(state: State(l: [ \"x\" ]), children: [ Repeat(over: l, children: [ P(\"{item.a}\") ]) ])", "il n'a pas de champs"),
            ("Page(state: State(l: [ Item(a: 1) ]), children: [ Repeat(over: l, children: [ P(\"{item.b}\") ]) ])", "pas de champ « b »"),
            ("Page(state: State(l: [ Item(a: 1) ]), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: l.push(\"x\")) ])", "l.push(Item"),
            ("Page(state: State(l: [ Item(a: 1) ]), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: l.push(Item(b: 2))) ])", "ont les champs a"),
            ("Page(state: State(l: [ \"x\" ]), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: l.push(Item(a: 2))) ])", "attend un texte"),
        ] {
            let erreur = crate::verifier_page(source).err().or_else(|| crate::vue_a_plat(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn une_condition_et_un_changement_sur_un_champ_de_la_ligne() {
        let source = r#"Page(
  state: State(tasks: [ Item(title: "Pain", done: 0), Item(title: "Lait", done: 1) ]),
  children: [
    H1("Tasks"),
    Repeat(over: tasks, children: [
      Row(children: [
        If(item.done, is: 1, children: [ Text.fait("{item.title} ✓") ], else: [ Text("{item.title}") ]),
        Button(name: Done, text: "Done"),
      ]),
    ], rules: [ On(Done.tap, effect: item.done.set(1)) ]),
  ],
)
.fait { opacity: 0.6; }"#;
        crate::verifier_page(source).unwrap();
        let depart = crate::etat_initial(source);
        let lignes = crate::liste_html(source, "", &depart, "tasks");
        assert!(lignes.contains(">Pain</div>") && lignes.contains("holo-s-fait\">Lait ✓</div>"), "{lignes}");
        let apres = crate::arbitrer(source, &depart, "Done.tap@0");
        let lignes = crate::liste_html(source, "", &apres, "tasks");
        assert!(lignes.contains("holo-s-fait\">Pain ✓</div>"), "{lignes}");
        // Dans une répétition fixe aussi.
        let fixe = "Page(children: [ H1(\"x\"), Repeat(items: [ Item(t: \"a\", n: 3), Item(t: \"b\", n: 0) ], children: [ If(item.n, over: 0, children: [ P(\"{item.t} en stock\") ], else: [ P(\"{item.t} épuisé\") ]) ]) ])";
        let html = crate::vue_a_plat(fixe, "").unwrap();
        assert!(html.contains("a en stock") && html.contains("b épuisé"), "{html}");
        for (source, message) in [
            ("Page(state: State(l: [ Item(a: 1) ]), children: [ Repeat(over: l, children: [ If(item.b, is: 1, children: [ P(\"x\") ]) ]) ])", "pas de champ « b »"),
            ("Page(children: [ If(item.a, is: 1, children: [ P(\"x\") ]) ])", "dans Repeat"),
            ("Page(state: State(l: [ Item(a: 1) ]), children: [ Repeat(over: l, children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: item.z.set(1)) ]) ])", "pas de champ « z »"),
            ("Page(state: State(l: [ Item(a: 1) ]), children: [ Repeat(over: l, children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: item.a.mul(2)) ]) ])", "on demande set"),
        ] {
            let erreur = crate::verifier_page(source).err().or_else(|| crate::vue_a_plat(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }
}
