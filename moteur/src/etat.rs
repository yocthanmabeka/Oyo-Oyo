//! L'état d'une page : des valeurs nommées, qui changent quand une règle le demande
//! (ADR-015, ADR-023).
//!
//! ```holo
//! Page(
//!   state: State(cart: 0),
//!   children: [ Text("{cart} paintings in the cart"), Button(name: Add, text: "Add") ],
//!   rules: [ On(Add.tap, effect: cart.add(1)) ],
//! )
//! ```
//!
//! Un bouton ne change rien lui-même. Il émet un signal ; une règle demande un changement ;
//! c'est l'arbitre, ici, qui le fait. Il n'y a pas de code à écrire, et la liste des demandes
//! est courte : `add`, `sub`, `set`.

use crate::holo::{Argument, Bloc, Erreur, Programme, Valeur};
use crate::regles::pour_chaque_bloc;

/// Garde-fous du langage : le nombre de valeurs d'une page, et jusqu'où va chacune.
pub const VALEURS_MAX: usize = 32;
pub const VALEUR_MAX: u64 = 1_000_000_000;

/// Les deux valeurs que le moteur calcule quand la page donne des prix (`prices:`) : le nombre
/// d'articles, et ce qu'ils coûtent ensemble. L'auteur n'écrit aucun calcul.
pub const CALCULEES: &[&str] = &["count", "total"];

/// Les comparaisons d'une condition : `If(count, is: 0)`. Des mots, pas des signes.
pub const COMPARAISONS: &[&str] = &["is", "not", "over", "under"];

/// Une condition lue dans un bloc `If` : la valeur regardée, et ce à quoi on la compare.
/// Plusieurs comparaisons valent ensemble : `If(count, over: 0, under: 10)`.
/// Ce à quoi une valeur est comparée, ou ce qu'une demande ajoute : un nombre écrit dans le
/// fichier, ou le nom d'une autre valeur, lue au moment où l'on en a besoin.
/// `If(score, over: 10)`, `When(score, over: best, effect: best.set(score))`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Terme<'a> {
    Nombre(u64),
    Valeur(&'a str),
}

impl Terme<'_> {
    /// Ce que vaut ce terme, pour cet état. Une valeur inconnue vaut 0.
    pub fn vaut(&self, etat: &Etat) -> u64 {
        match self {
            Terme::Nombre(nombre) => *nombre,
            Terme::Valeur(nom) => etat.iter().find(|(connu, _)| connu == nom).map_or(0, |(_, v)| *v),
        }
    }
}

impl std::fmt::Display for Terme<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Terme::Nombre(nombre) => write!(f, "{nombre}"),
            Terme::Valeur(nom) => write!(f, "{nom}"),
        }
    }
}

pub fn condition(bloc: &Bloc) -> Result<(&str, Vec<(&str, Terme<'_>)>), Erreur> {
    // `If` montre ses enfants ; `When` est une règle, elle a un effet.
    let regle = bloc.nom == "When";
    let erreur = |message: &str| Erreur { message: message.into(), pos: bloc.pos };
    let ecriture = if regle {
        "une règle qui guette s'écrit « When(lives, is: 0, effect: score.set(0)) » ; comparaisons : is (égal), not (différent), over (plus grand), under (plus petit)"
    } else {
        "une condition s'écrit « If(count, is: 0, children: [ … ]) » ; comparaisons : is (égal), not (différent), over (plus grand), under (plus petit)"
    };
    let valeur = match bloc.arguments.first() {
        Some(Argument { nom: None, valeur: Valeur::Nom(valeur), .. }) => valeur.as_str(),
        _ => return Err(erreur(ecriture)),
    };
    let mut comparaisons = Vec::new();
    for argument in &bloc.arguments[1..] {
        match (argument.nom.as_deref(), &argument.valeur) {
            (Some("children" | "rules" | "name" | "else"), _) if !regle => {}
            (Some("effect"), _) if regle => {}
            // Pour un texte : « is: "" » (vide) et « not: "" » (rempli). Vide vaut 0.
            (Some(mot @ ("is" | "not")), Valeur::Texte(texte)) if texte.is_empty() => comparaisons.push((if mot == "is" { "is" } else { "not" }, Terme::Nombre(0))),
            (Some(mot), Valeur::Entier(nombre)) if COMPARAISONS.contains(&mot) => comparaisons.push((COMPARAISONS[COMPARAISONS.iter().position(|c| *c == mot).unwrap_or(0)], Terme::Nombre(*nombre))),
            // Comparer à une autre valeur : If(score, over: best).
            (Some(mot), Valeur::Nom(autre)) if COMPARAISONS.contains(&mot) && est_nom_de_valeur(autre) => {
                comparaisons.push((COMPARAISONS[COMPARAISONS.iter().position(|c| *c == mot).unwrap_or(0)], Terme::Valeur(autre.as_str())))
            }
            (Some(mot), _) if COMPARAISONS.contains(&mot) => return Err(Erreur { message: format!("« If({valeur}, {mot}: …) » attend un nombre entier, ou le nom d'une autre valeur"), pos: argument.pos }),
            (Some(mot), _) => {
                return Err(Erreur {
                    message: format!("« {} » n'a pas de paramètre « {mot} » ; paramètres possibles : is, not, over, under, {}", bloc.nom, if regle { "effect" } else { "children" }),
                    pos: argument.pos,
                })
            }
            (None, _) => return Err(erreur(ecriture)),
        }
    }
    if comparaisons.is_empty() {
        return Err(erreur(ecriture));
    }
    // Un `If` montre des blocs (children), ou met des règles sous condition (rules) : l'un ou l'autre.
    let liste = |param: &str| matches!(bloc.argument(param).map(|a| &a.valeur), Some(Valeur::Liste(_)));
    if !regle && (liste("children") == liste("rules")) {
        return Err(erreur("« If » attend ce qu'il montre, If(count, is: 0, children: [ … ]), ou les règles qu'il met sous condition, If(lives, over: 0, rules: [ … ])"));
    }
    // Le « sinon » (ADR-039) : ce qu'on montre quand la condition est fausse. Il va avec children.
    if let Some(argument) = bloc.argument("else").filter(|_| !regle) {
        if !matches!(argument.valeur, Valeur::Liste(_)) || !liste("children") {
            return Err(Erreur { message: "« else » va avec children : il montre des blocs quand la condition est fausse, If(cart, is: 0, children: [ … ], else: [ … ])".into(), pos: argument.pos });
        }
    }
    Ok((valeur, comparaisons))
}

/// Le nom sous lequel une condition est connue de la page : `count|is=0`, `total|over=0|under=300`.
/// Deux conditions écrites pareil portent le même nom, et ont toujours la même réponse.
pub fn cle(valeur: &str, comparaisons: &[(&str, Terme<'_>)]) -> String {
    comparaisons.iter().fold(valeur.to_string(), |cle, (mot, nombre)| format!("{cle}|{mot}={nombre}"))
}

/// Toutes les conditions du fichier, avec leur réponse pour ces valeurs. C'est le seul endroit
/// où une condition est décidée : au premier affichage comme après chaque changement.
pub fn conditions(programme: &Programme, montrees: &Etat) -> Vec<(String, bool)> {
    let mut reponses: Vec<(String, bool)> = Vec::new();
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if bloc.nom == "If" {
            if let Ok((valeur, comparaisons)) = condition(bloc) {
                let cle = cle(valeur, &comparaisons);
                if !reponses.iter().any(|(connue, _)| *connue == cle) {
                    let nombre = montrees.iter().find(|(connu, _)| connu == valeur).map_or(0, |(_, v)| *v);
                    reponses.push((cle, vraie(&comparaisons, nombre, montrees)));
                }
            }
        }
        Ok(())
    });
    reponses
}

/// La condition est-elle vraie pour cette valeur ?
pub fn vraie(comparaisons: &[(&str, Terme<'_>)], valeur: u64, etat: &Etat) -> bool {
    comparaisons.iter().all(|(mot, terme)| {
        let nombre = terme.vaut(etat);
        match *mot {
            "is" => valeur == nombre,
            "not" => valeur != nombre,
            "over" => valeur > nombre,
            _ => valeur < nombre,
        }
    })
}

/// Les valeurs que la page garde d'une visite à l'autre : `keep: [cart, best]`.
pub fn gardees(programme: &Programme) -> Result<Vec<String>, Erreur> {
    let Some(argument) = programme.racine.argument("keep") else { return Ok(Vec::new()) };
    let erreur = |message: String| Erreur { message, pos: argument.pos };
    let Valeur::Liste(noms) = &argument.valeur else {
        return Err(erreur("« keep » attend la liste des valeurs à garder : keep: [cart]".into()));
    };
    let declarees = initial(programme)?;
    let textes = textes_initiaux(programme);
    let mut gardees = Vec::new();
    for nom in noms {
        match nom {
            Valeur::Nom(nom) if HORLOGE.contains(&nom.as_str()) => return Err(erreur(format!("« keep » : « {nom} » est l'heure du visiteur, elle ne se garde pas"))),
            Valeur::Nom(nom) if declarees.iter().any(|(connu, _)| connu == nom) || textes.iter().any(|(connu, _)| connu == nom) || crate::listes::est_liste(programme, nom) => gardees.push(nom.clone()),
            Valeur::Nom(nom) => return Err(erreur(format!("« keep » : aucune valeur ne s'appelle « {nom} » ; on ne garde que des valeurs déclarées dans « State »"))),
            _ => return Err(erreur("« keep » attend des noms de valeurs : keep: [cart]".into())),
        }
    }
    Ok(gardees)
}

/// Jusqu'où une valeur peut monter par la saisie : 1 pour une case à cocher, le `max` d'un champ
/// s'il en a un, sinon la borne du langage.
fn plafond(programme: &Programme, nom: &str) -> u64 {
    let mut plafond = VALEUR_MAX;
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if matches!(bloc.argument("value").map(|a| &a.valeur), Some(Valeur::Nom(valeur)) if valeur == nom) {
            match (bloc.nom.as_str(), bloc.argument("max").map(|a| &a.valeur)) {
                ("Checkbox", _) => plafond = plafond.min(1),
                ("Input" | "Slider", Some(Valeur::Entier(max))) => plafond = plafond.min(*max),
                ("Slider", None) => plafond = plafond.min(100),
                _ => {}
            }
        }
        // Une valeur qui sert de place sur un plateau reste sur le plateau : de 0 à 100.
        for axe in ["x", "y"] {
            if matches!(bloc.argument(axe).map(|a| &a.valeur), Some(Valeur::Nom(valeur)) if valeur == nom) {
                plafond = plafond.min(100);
            }
        }
        Ok(())
    });
    plafond
}

/// Le plus petit nombre qu'une glissière laisse choisir (ADR-042) ; 0 sinon.
fn plancher(programme: &Programme, nom: &str) -> u64 {
    let mut plancher = 0;
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if bloc.nom == "Slider" && matches!(bloc.argument("value").map(|a| &a.valeur), Some(Valeur::Nom(valeur)) if valeur == nom) {
            if let Some(Valeur::Entier(min)) = bloc.argument("min").map(|a| &a.valeur) {
                plancher = plancher.max(*min);
            }
        }
        Ok(())
    });
    plancher
}

/// Le visiteur a écrit dans un champ, ou coché une case. C'est encore l'arbitre qui change la
/// valeur : seulement une valeur que la page déclare et qu'un champ présente, et jamais
/// au-delà de son plafond. Un texte qui n'est pas un nombre ne change rien.
pub fn saisir(programme: &Programme, etat: &Etat, nom: &str, ecrit: &str) -> Etat {
    let avant = etat.clone();
    let mut etat = etat.clone();
    let presentee = {
        let mut trouvee = false;
        let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
            trouvee |= matches!(bloc.nom.as_str(), "Input" | "Checkbox" | "Slider") && matches!(bloc.argument("value").map(|a| &a.valeur), Some(Valeur::Nom(valeur)) if valeur == nom);
            Ok(())
        });
        trouvee
    };
    let ecrit = ecrit.trim();
    let nombre = if ecrit.is_empty() { Some(0) } else { ecrit.parse::<u64>().ok() };
    let plancher = plancher(programme, nom);
    if let (true, Some(nombre), Some((_, place))) = (presentee, nombre, etat.iter_mut().find(|(connu, _)| connu == nom)) {
        *place = nombre.min(plafond(programme, nom)).max(plancher);
    }
    suites(programme, avant, etat)
}

/// Reprend les valeurs gardées lors d'une visite précédente. Ce qui est relu vient du
/// navigateur du visiteur, donc on s'en méfie : seules les valeurs que la page dit garder sont
/// reprises, et dans leurs bornes. Tout le reste part de son départ.
pub fn reprendre(programme: &Programme, garde: &str) -> Etat {
    let mut etat = initial(programme).unwrap_or_default();
    let gardees = gardees(programme).unwrap_or_default();
    for morceau in garde.split(';') {
        if let Some((nom, valeur)) = morceau.split_once('=') {
            if let (true, Some((_, place)), Ok(valeur)) = (gardees.iter().any(|g| g == nom), etat.iter_mut().find(|(connu, _)| connu == nom), valeur.parse::<u64>()) {
                *place = valeur.min(VALEUR_MAX);
            }
        }
    }
    etat
}

/// Le rythme le plus rapide et le plus lent auquel une page redemande ses données.
pub const DONNEES_MIN: u64 = 1_000;
pub const DONNEES_MAX: u64 = 3_600_000;
/// La taille d'un fichier de données, au plus.
pub const DONNEES_OCTETS: usize = 65_536;

/// D'où viennent les données de la page : `data: Data(from: "stock.json", every: 30s)`.
/// Rend le fichier, et le rythme en millisecondes (0 : une seule fois, à l'ouverture).
pub fn source_de_donnees(programme: &Programme) -> Result<Option<(String, u64)>, Erreur> {
    let Some(argument) = programme.racine.argument("data") else { return Ok(None) };
    let bloc = match &argument.valeur {
        Valeur::Bloc(bloc) if bloc.nom == "Data" && programme.racine.nom == "Page" => bloc,
        _ => return Err(Erreur { message: "« data » attend un bloc « Data(...) », sur la page : data: Data(from: \"stock.json\")".into(), pos: argument.pos }),
    };
    let (mut fichier, mut rythme) = (None, 0);
    for argument in &bloc.arguments {
        match (argument.nom.as_deref(), &argument.valeur) {
            // Un fichier rangé à côté de la page : ni adresse complète, ni remontée de dossier.
            // La page ne parle qu'au serveur d'où elle vient.
            (Some("from"), Valeur::Texte(nom)) if nom.ends_with(".json") && !nom.starts_with('/') && !nom.contains("..") && nom.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/')) => {
                fichier = Some(nom.clone());
            }
            (Some("from"), _) => return Err(Erreur { message: "« Data(from: …) » attend un fichier .json rangé à côté de la page, comme \"stock.json\"".into(), pos: argument.pos }),
            (Some("every"), Valeur::Nombre { valeur, unite: Some(unite) }) if unite == "s" || unite == "ms" => {
                let ms = if unite == "s" { valeur * 1000.0 } else { *valeur };
                if !(DONNEES_MIN as f64..=DONNEES_MAX as f64).contains(&ms) {
                    return Err(Erreur { message: "« Data(every: …) » va de 1s à 3600s".into(), pos: argument.pos });
                }
                rythme = ms.round() as u64;
            }
            (Some("every"), _) => return Err(Erreur { message: "« Data(every: …) » attend une durée, de 1s à 3600s".into(), pos: argument.pos }),
            (Some(mot), _) => return Err(Erreur { message: format!("« Data » n'a pas de paramètre « {mot} » ; paramètres possibles : from, every"), pos: argument.pos }),
            (None, _) => return Err(Erreur { message: "chaque paramètre de « Data » est nommé : Data(from: \"stock.json\")".into(), pos: argument.pos }),
        }
    }
    match fichier {
        Some(fichier) => Ok(Some((fichier, rythme))),
        None => Err(Erreur { message: "« Data » attend « from » : Data(from: \"stock.json\")".into(), pos: bloc.pos }),
    }
}

/// Ce qu'on lit dans un fichier de données : un nombre entier, ou un texte.
#[derive(Debug, PartialEq)]
pub enum Donnee {
    Nombre(u64),
    Texte(String),
}

/// Lit un fichier de données : un objet JSON à plat, `{"stock": 4, "message": "Ouvert"}`.
/// Sont repris : les nombres entiers positifs, les textes, et `true`/`false` (1 et 0). Tout le
/// reste (nombres à virgule ou négatifs, listes, objets emboîtés, `null`) est laissé de côté.
/// Un fichier mal formé ne donne rien du tout.
pub fn lire_donnees(json: &str) -> Vec<(String, Donnee)> {
    fn blancs(t: &[char], i: &mut usize) {
        while *i < t.len() && t[*i].is_whitespace() {
            *i += 1;
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
    // Saute une valeur qu'on ne reprend pas : une liste, un objet, un nombre à virgule, null.
    fn sauter(t: &[char], i: &mut usize) -> Option<()> {
        let mut profondeur = 0usize;
        loop {
            match *t.get(*i)? {
                '"' => {
                    texte(t, i)?;
                    continue;
                }
                '[' | '{' => profondeur += 1,
                ']' | '}' if profondeur > 0 => profondeur -= 1,
                ',' | '}' if profondeur == 0 => return Some(()),
                _ => {}
            }
            *i += 1;
        }
    }
    let lire = || -> Option<Vec<(String, Donnee)>> {
        if json.len() > DONNEES_OCTETS {
            return None;
        }
        let t: Vec<char> = json.chars().collect();
        let mut i = 0;
        let mut donnees = Vec::new();
        blancs(&t, &mut i);
        if t.get(i) != Some(&'{') {
            return None;
        }
        i += 1;
        loop {
            blancs(&t, &mut i);
            if t.get(i) == Some(&'}') {
                return Some(donnees);
            }
            let cle = texte(&t, &mut i)?;
            blancs(&t, &mut i);
            if t.get(i) != Some(&':') {
                return None;
            }
            i += 1;
            blancs(&t, &mut i);
            let debut = i;
            match *t.get(i)? {
                '"' => donnees.push((cle, Donnee::Texte(texte(&t, &mut i)?))),
                c if c.is_ascii_digit() => {
                    while t.get(i).is_some_and(|c| c.is_ascii_digit()) {
                        i += 1;
                    }
                    // Un nombre à virgule ou avec exposant n'est pas repris.
                    if t.get(i).is_some_and(|c| matches!(c, '.' | 'e' | 'E')) {
                        sauter(&t, &mut i)?;
                    } else if let Ok(nombre) = t[debut..i].iter().collect::<String>().parse::<u64>() {
                        donnees.push((cle, Donnee::Nombre(nombre)));
                    }
                }
                _ => {
                    let mot: String = t[i..].iter().take(5).collect();
                    if mot.starts_with("true") {
                        donnees.push((cle, Donnee::Nombre(1)));
                    } else if mot.starts_with("false") {
                        donnees.push((cle, Donnee::Nombre(0)));
                    }
                    sauter(&t, &mut i)?;
                }
            }
            blancs(&t, &mut i);
            match *t.get(i)? {
                ',' => i += 1,
                '}' => return Some(donnees),
                _ => return None,
            }
        }
    };
    lire().unwrap_or_default()
}

/// Les données viennent d'arriver. C'est l'arbitre qui les range : seulement dans des valeurs
/// que la page déclare, de la bonne sorte (un nombre dans un nombre, un texte dans un texte),
/// et dans leurs bornes. Puis les règles qui guettent ont leur mot à dire.
pub fn recevoir(programme: &Programme, etat: &Etat, textes: &Textes, json: &str) -> (Etat, Textes) {
    let avant = etat.clone();
    let (mut etat, mut textes) = (etat.clone(), textes.clone());
    if source_de_donnees(programme).ok().flatten().is_none() {
        return (etat, textes);
    }
    for (cle, donnee) in lire_donnees(json) {
        match donnee {
            Donnee::Nombre(nombre) => {
                let plafond = plafond(programme, &cle);
                if let Some((_, place)) = etat.iter_mut().find(|(connu, _)| *connu == cle && connu != TIRAGES && !HORLOGE.contains(&connu.as_str())) {
                    *place = nombre.min(plafond);
                }
            }
            Donnee::Texte(texte) => {
                if let Some((_, place)) = textes.iter_mut().find(|(connu, _)| *connu == cle) {
                    *place = propre(&texte, TEXTE_MAX);
                }
            }
        }
    }
    (suites(programme, avant, etat), textes)
}

thread_local! {
    /// Les capacités demandées par les règles de temps et les règles qui guettent pendant un
    /// appel à l'arbitre (`Ding.play`). La page les lit après l'appel, pour faire entendre le son.
    static CAPACITES: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Vide la liste des capacités demandées, et la rend.
pub fn capacites_demandees() -> Vec<String> {
    CAPACITES.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Note les capacités qu'une règle demande, à côté de ses demandes : `effect: [score.add(1), Ding.play]`.
fn noter_les_capacites(regle: &Bloc) {
    let noms: Vec<&String> = match regle.argument("effect").map(|a| &a.valeur) {
        Some(Valeur::Nom(nom)) => vec![nom],
        Some(Valeur::Liste(elements)) => elements.iter().filter_map(|e| match e {
            Valeur::Nom(nom) => Some(nom),
            _ => None,
        }).collect(),
        _ => Vec::new(),
    };
    CAPACITES.with(|c| c.borrow_mut().extend(noms.into_iter().cloned()));
}

/// Parcourt tous les blocs en disant, pour chacun, s'il est en vigueur pour cet état. Des règles
/// rangées dans `If(lives, over: 0, rules: [ … ])` ne valent que tant que la condition est vraie.
fn avec_leur_vigueur<'a>(programme: &'a Programme, etat: &Etat, f: &mut dyn FnMut(&'a Bloc, bool)) {
    fn visiter<'a>(valeur: &'a Valeur, en_vigueur: bool, montrees: &Etat, f: &mut dyn FnMut(&'a Bloc, bool)) {
        match valeur {
            Valeur::Liste(elements) => elements.iter().for_each(|e| visiter(e, en_vigueur, montrees, f)),
            Valeur::Bloc(bloc) => {
                f(bloc, en_vigueur);
                let sous_condition = bloc.nom == "If" && bloc.argument("rules").is_some();
                let dedans = en_vigueur
                    && (!sous_condition
                        || condition(bloc).is_ok_and(|(valeur, comparaisons)| montrees.iter().find(|(connu, _)| connu == valeur).is_some_and(|(_, n)| vraie(&comparaisons, *n, montrees))));
                for argument in &bloc.arguments {
                    let ici = if sous_condition && argument.nom.as_deref() == Some("rules") { dedans } else { en_vigueur };
                    visiter(&argument.valeur, ici, montrees, f);
                }
            }
            _ => {}
        }
    }
    let montrees = a_montrer(programme, etat);
    f(&programme.racine, true);
    for argument in &programme.racine.arguments {
        visiter(&argument.valeur, true, &montrees, f);
    }
}

/// Cette règle est-elle en vigueur, pour cet état ?
fn en_vigueur(programme: &Programme, regle: &Bloc, etat: &Etat) -> bool {
    let mut reponse = true;
    avec_leur_vigueur(programme, etat, &mut |bloc, vigueur| {
        if std::ptr::eq(bloc, regle) {
            reponse = vigueur;
        }
    });
    reponse
}

/// Les demandes d'une règle : une seule (`effect: cart.add(1)`), ou plusieurs entre crochets
/// (`effect: [score.set(0), lives.set(3)]`), faites dans l'ordre où elles sont écrites.
pub fn demandes_de(regle: &Bloc) -> Vec<&Bloc> {
    match regle.argument("effect").map(|a| &a.valeur) {
        Some(Valeur::Bloc(demande)) if est_demande(demande) => vec![demande],
        Some(Valeur::Liste(elements)) => elements.iter().filter_map(|e| match e {
            Valeur::Bloc(demande) if est_demande(demande) => Some(demande),
            _ => None,
        }).collect(),
        _ => Vec::new(),
    }
}

/// Ce qu'on peut demander pour une valeur.
pub const DEMANDES: &[&str] = &["add", "sub", "set", "random", "mul", "div"];

/// Sous ce nom, l'état garde le nombre de tirages au hasard déjà faits. Ce n'est pas une valeur
/// de l'auteur (le nom n'est pas un nom permis) : il sert à ce que le hasard soit rejouable.
const TIRAGES: &str = "~";

/// Le rythme le plus rapide et le plus lent d'une règle `Every`, en millisecondes.
pub const RYTHME_MIN: u64 = 100;
pub const RYTHME_MAX: u64 = 3_600_000;

/// Le rythme d'une règle `Every(1s, effect: …)`, en millisecondes.
pub fn rythme(regle: &Bloc) -> Result<u64, Erreur> {
    let erreur = || Erreur {
        message: if regle.nom == "After" {
            "une attente s'écrit « After(3s, effect: shown.set(1)) » : une durée en s ou en ms, de 100ms à 3600s".into()
        } else {
            "une règle de temps s'écrit « Every(1s, effect: time.sub(1)) » : une durée en s ou en ms, de 100ms à 3600s".into()
        },
        pos: regle.pos,
    };
    let millisecondes = match regle.arguments.first() {
        Some(Argument { nom: None, valeur: Valeur::Nombre { valeur, unite: Some(unite) }, .. }) if unite == "s" => valeur * 1000.0,
        Some(Argument { nom: None, valeur: Valeur::Nombre { valeur, unite: Some(unite) }, .. }) if unite == "ms" => *valeur,
        _ => return Err(erreur()),
    };
    if !(RYTHME_MIN as f64..=RYTHME_MAX as f64).contains(&millisecondes) {
        return Err(erreur());
    }
    Ok(millisecondes.round() as u64)
}

/// Les horloges du fichier : une par règle `Every`, dans l'ordre où elles sont écrites, avec
/// son rythme et la valeur qu'elle fait changer. Chaque règle a son horloge, pour qu'on puisse
/// en relancer une sans toucher aux autres.
pub fn horloges(programme: &Programme) -> Vec<(u64, String)> {
    let mut horloges = Vec::new();
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if bloc.nom == "Every" {
            let valeurs: Vec<&str> = demandes_de(bloc).iter().filter_map(|d| d.nom.split('.').next()).collect();
            horloges.push((rythme(bloc).unwrap_or(RYTHME_MAX), valeurs.join(",")));
        }
        Ok(())
    });
    horloges
}

/// Les attentes du fichier, une par règle `After`, dans l'ordre où elles sont écrites : sa durée,
/// et si elle court pour cet état (ADR-039). Une attente posée dans la page court dès
/// l'ouverture ; une attente rangée sous une condition, `If(toast, is: 1, rules: [ After(3s, …) ])`,
/// court à partir du moment où la condition devient vraie, et s'arrête si elle redevient fausse.
/// Chacune ne sonne qu'une fois par période où elle court : c'est la page qui tient le compte.
pub fn delais(programme: &Programme, etat: &Etat) -> Vec<(u64, bool)> {
    let mut delais = Vec::new();
    avec_leur_vigueur(programme, etat, &mut |bloc, vigueur| {
        if bloc.nom == "After" {
            delais.push((rythme(bloc).unwrap_or(RYTHME_MAX), vigueur));
        }
    });
    delais
}

/// Les valeurs qu'un signal fait changer, d'après les règles `On`. Quand un geste change une
/// valeur, l'horloge qui s'occupe de cette valeur repart de zéro : « Play » remet le temps à
/// trente secondes, et la première seconde dure une vraie seconde.
pub fn touchees(programme: &Programme, signal: &str) -> Vec<String> {
    let mut valeurs = Vec::new();
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        let declencheur = bloc.arguments.iter().find(|a| a.nom.is_none()).map(|a| &a.valeur);
        if let ("On", Some(Valeur::Nom(s))) = (bloc.nom.as_str(), declencheur) {
            if s == signal {
                for effet in demandes_de(bloc) {
                    if let Some((valeur, _)) = effet.nom.split_once('.') {
                        if !valeurs.iter().any(|v| v == valeur) {
                            valeurs.push(valeur.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    });
    valeurs
}

/// La graine du hasard d'un fichier : tirée du nom de sa page. Le même fichier, avec les mêmes
/// gestes aux mêmes moments, redonne la même partie (ADR-008).
fn graine_du_hasard(programme: &Programme) -> u64 {
    let nom = crate::regles::nom_de(&programme.racine).unwrap_or("Home");
    nom.bytes().fold(0x4A5E_u64, |g, octet| crate::graine::melanger(g ^ u64::from(octet)))
}

/// Les valeurs d'une page, dans l'ordre où elles sont déclarées.
pub type Etat = Vec<(String, u64)>;

/// Les valeurs de texte d'une page : `State(buyer: "")`. Elles ne changent que par un champ
/// (`Input`) ; on les montre (`{buyer}`), et l'on peut demander si elles sont vides.
pub type Textes = Vec<(String, String)>;

/// La longueur d'un texte, au plus. Un champ peut l'abaisser par `max:`.
pub const TEXTE_MAX: usize = 2000;
/// La longueur d'un texte saisi quand le champ ne dit rien.
pub const TEXTE_COURANT: usize = 80;
/// La longueur d'un texte long (`Input(lines:)`) quand l'auteur n'a pas dit `max`.
pub const TEXTE_LONG: usize = 1000;

/// Les textes déclarés par la page, à leur départ.
pub fn textes_initiaux(programme: &Programme) -> Textes {
    let Ok(Some(bloc)) = bloc_d_etat(programme) else { return Vec::new() };
    bloc.arguments
        .iter()
        .filter_map(|a| match (&a.nom, &a.valeur) {
            (Some(nom), Valeur::Texte(texte)) => Some((nom.clone(), texte.clone())),
            _ => None,
        })
        .collect()
}

/// Un texte, écrit pour voyager dans l'état sans se mêler à ses séparateurs : tout ce qui
/// n'est pas une lettre ou un chiffre ordinaire devient « %XX ».
pub fn coder(texte: &str) -> String {
    texte.bytes().map(|octet| if octet.is_ascii_alphanumeric() { char::from(octet).to_string() } else { format!("%{octet:02X}") }).collect()
}

/// L'inverse. Rend `None` si ce n'est pas un texte codé par `coder`.
pub fn decoder(code: &str) -> Option<String> {
    let mut octets = Vec::with_capacity(code.len());
    let mut reste = code.as_bytes();
    while let Some((premier, suite)) = reste.split_first() {
        if *premier == b'%' {
            let hexa = std::str::from_utf8(suite.get(..2)?).ok()?;
            octets.push(u8::from_str_radix(hexa, 16).ok()?);
            reste = &suite[2..];
        } else {
            octets.push(*premier);
            reste = suite;
        }
    }
    String::from_utf8(octets).ok()
}

/// `buyer='Ada;city='` : les textes, à la suite des nombres. L'apostrophe dit « c'est un texte ».
pub fn ecrire_textes(textes: &Textes) -> String {
    textes.iter().map(|(nom, texte)| format!("{nom}='{}", coder(texte))).collect::<Vec<_>>().join(";")
}

/// Relit les textes d'un état. Seuls ceux que la page déclare sont repris, et bornés.
pub fn relire_textes(programme: &Programme, ecrit: &str) -> Textes {
    let mut textes = textes_initiaux(programme);
    for morceau in ecrit.split(';') {
        if let Some((nom, code)) = morceau.split_once("='") {
            if let (Some((_, place)), Some(texte)) = (textes.iter_mut().find(|(connu, _)| connu == nom), decoder(code)) {
                // Les retours à la ligne d'un texte long (Input(lines:)) sont gardés ; les autres
                // caractères invisibles, non.
                *place = propre_sur_plusieurs_lignes(&texte, TEXTE_MAX);
            }
        }
    }
    textes
}

/// Un texte saisi, nettoyé : sans caractère invisible, et pas plus long que permis.
fn propre(texte: &str, longueur: usize) -> String {
    texte.chars().filter(|c| !c.is_control()).take(longueur).collect()
}

/// Un texte long (`Input(lines:)`) garde ses retours à la ligne.
fn propre_sur_plusieurs_lignes(texte: &str, longueur: usize) -> String {
    texte.replace("\r\n", "\n").chars().filter(|c| *c == '\n' || !c.is_control()).take(longueur).collect()
}

/// Les options d'un `Choice`, telles qu'écrites.
pub fn options_du_choix(bloc: &Bloc) -> Vec<&str> {
    match bloc.argument("options").map(|a| &a.valeur) {
        Some(Valeur::Liste(options)) => options.iter().filter_map(|o| match o {
            Valeur::Texte(t) => Some(t.as_str()),
            _ => None,
        }).collect(),
        _ => Vec::new(),
    }
}

/// Le visiteur a écrit dans un champ de texte. Comme pour un nombre : seulement une valeur
/// qu'un champ présente, et pas plus longue que ce champ ne le permet.
pub fn saisir_texte(programme: &Programme, textes: &Textes, nom: &str, ecrit: &str) -> Textes {
    let mut textes = textes.clone();
    let mut longueur = None;
    let mut lignes = false;
    let mut choix: Option<Vec<String>> = None;
    let mut sorte: Option<String> = None;
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        let presente = matches!(bloc.argument("value").map(|a| &a.valeur), Some(Valeur::Nom(valeur)) if valeur == nom);
        if let (true, true, Some(Valeur::Nom(t))) = (bloc.nom == "Input", presente, bloc.argument("type").map(|a| &a.valeur)) {
            sorte = Some(t.clone());
        }
        if bloc.nom == "Input" && presente {
            lignes = bloc.argument("lines").is_some();
            longueur = Some(match bloc.argument("max").map(|a| &a.valeur) {
                Some(Valeur::Entier(max)) => (*max as usize).min(TEXTE_MAX),
                _ if lignes => TEXTE_LONG,
                _ => TEXTE_COURANT,
            });
        }
        // Un choix n'accepte que l'une de ses options (ou rien).
        if bloc.nom == "Choice" && presente {
            choix = Some(options_du_choix(bloc).into_iter().map(str::to_string).collect());
        }
        Ok(())
    });
    // Une date, une heure, une couleur (ADR-042) : seulement ce que le navigateur sait écrire.
    if let Some(sorte) = sorte {
        let chiffres = |t: &str, gabarit: &str| t.len() == gabarit.len() && t.chars().zip(gabarit.chars()).all(|(c, g)| if g == '9' { c.is_ascii_digit() } else { c == g });
        let correct = ecrit.is_empty()
            || match sorte.as_str() {
                "date" => chiffres(ecrit, "9999-99-99"),
                "time" => chiffres(ecrit, "99:99"),
                _ => ecrit.len() == 7 && ecrit.starts_with('#') && ecrit[1..].chars().all(|c| c.is_ascii_hexdigit()),
            };
        if let (true, Some((_, place))) = (correct, textes.iter_mut().find(|(connu, _)| connu == nom)) {
            *place = ecrit.to_ascii_lowercase();
        }
        return textes;
    }
    if let Some(options) = choix {
        if let Some((_, place)) = textes.iter_mut().find(|(connu, _)| connu == nom) {
            if ecrit.is_empty() || options.iter().any(|o| o == ecrit) {
                *place = ecrit.to_string();
            }
        }
        return textes;
    }
    if let (Some(longueur), Some((_, place))) = (longueur, textes.iter_mut().find(|(connu, _)| connu == nom)) {
        *place = if lignes { propre_sur_plusieurs_lignes(ecrit, longueur) } else { propre(ecrit, longueur) };
    }
    textes
}

/// Ce qu'un formulaire envoie (ADR-042) : les valeurs que présentent ses champs, en JSON,
/// `{"form":"Contact","values":{"name":"Ada","size":"M","quantity":2}}`. `None` si aucun
/// formulaire ne porte ce nom.
pub fn envoi(programme: &Programme, etat: &Etat, textes: &Textes, formulaire: &str) -> Option<String> {
    let form = crate::regles::bloc_nomme(programme, formulaire).filter(|b| b.nom == "Form")?;
    let mut noms: Vec<&str> = Vec::new();
    let _ = pour_chaque_bloc(form, &mut |bloc| {
        if matches!(bloc.nom.as_str(), "Input" | "Checkbox" | "Choice" | "Slider") {
            if let Some(Valeur::Nom(valeur)) = bloc.argument("value").map(|a| &a.valeur) {
                if !noms.contains(&valeur.as_str()) {
                    noms.push(valeur);
                }
            }
        }
        Ok(())
    });
    let json = |texte: &str| {
        let mut sortie = String::from("\"");
        for c in texte.chars() {
            match c {
                '"' => sortie.push_str("\\\""),
                '\\' => sortie.push_str("\\\\"),
                '\n' => sortie.push_str("\\n"),
                c if (c as u32) < 0x20 => sortie.push_str(&format!("\\u{:04x}", c as u32)),
                c => sortie.push(c),
            }
        }
        sortie.push('"');
        sortie
    };
    let valeurs: Vec<String> = noms
        .iter()
        .filter_map(|nom| match (etat.iter().find(|(c, _)| c == nom), textes.iter().find(|(c, _)| c == nom)) {
            (Some((_, n)), _) => Some(format!("{}:{n}", json(nom))),
            (_, Some((_, t))) => Some(format!("{}:{}", json(nom), json(t))),
            _ => None,
        })
        .collect();
    Some(format!("{{\"form\":{},\"values\":{{{}}}}}", json(formulaire), valeurs.join(",")))
}

/// Pour les conditions, un texte vaut 0 quand il est vide, 1 sinon : `If(buyer, not: "")`.
pub fn avec_textes(montrees: &Etat, textes: &Textes) -> Etat {
    let mut tout = montrees.clone();
    tout.extend(textes.iter().map(|(nom, texte)| (nom.clone(), u64::from(!texte.is_empty()))));
    tout
}

/// `cart`, `items_seen` : une minuscule, puis des minuscules, des chiffres ou `_`.
/// Un nom de valeur s'écrit comme en Flutter : une minuscule au début, puis lettres et chiffres,
/// les mots joints par une majuscule (`appleX`, `blueDoor`), sans `_` (ADR-037).
fn est_nom_de_valeur(nom: &str) -> bool {
    nom.starts_with(|c: char| c.is_ascii_lowercase()) && nom.chars().all(|c| c.is_ascii_alphanumeric())
}

/// `appleX` → `appleX` ; `topRight` → `topRight`. Pour dire le bon mot à qui a écrit l'autre.
pub fn en_flutter(nom: &str) -> String {
    let mut sortie = String::with_capacity(nom.len());
    let mut majuscule = false;
    for c in nom.chars() {
        if c == '_' {
            majuscule = !sortie.is_empty();
        } else if majuscule {
            sortie.push(c.to_ascii_uppercase());
            majuscule = false;
        } else {
            sortie.push(c);
        }
    }
    sortie
}

/// Le message pour un nom de valeur mal écrit.
fn nom_de_valeur_mal_ecrit(nom: &str) -> String {
    if nom.contains('_') {
        format!("« {nom} » : deux mots se joignent comme en Flutter, par une majuscule ; écris « {} » (ADR-037)", en_flutter(nom))
    } else {
        format!("« {nom} » : le nom d'une valeur commence par une minuscule, comme « cart » ou « appleX » (ADR-037)")
    }
}

/// Le bloc `State(...)` donné à la page par `state:`.
fn bloc_d_etat(programme: &Programme) -> Result<Option<&Bloc>, Erreur> {
    match programme.racine.argument("state") {
        None => Ok(None),
        Some(argument) => match &argument.valeur {
            Valeur::Bloc(bloc) if bloc.nom == "State" && programme.racine.nom == "Page" => Ok(Some(bloc)),
            _ => Err(Erreur { message: "« state » attend un bloc « State(...) », sur la page : state: State(cart: 0)".into(), pos: argument.pos }),
        },
    }
}

/// Les prix donnés par la page : `prices: Prices(sunrise: 120)`. Chaque prix porte le nom
/// d'une valeur déclarée, qui est la quantité de cet article.
pub fn prix(programme: &Programme) -> Result<Etat, Erreur> {
    let Some(argument) = programme.racine.argument("prices") else { return Ok(Vec::new()) };
    let bloc = match &argument.valeur {
        Valeur::Bloc(bloc) if bloc.nom == "Prices" && programme.racine.nom == "Page" => bloc,
        _ => return Err(Erreur { message: "« prices » attend un bloc « Prices(...) », sur la page : prices: Prices(sunrise: 120)".into(), pos: argument.pos }),
    };
    let quantites = initial_sans_prix(programme)?;
    let mut prix = Etat::new();
    for argument in &bloc.arguments {
        let (Some(nom), Valeur::Entier(montant)) = (&argument.nom, &argument.valeur) else {
            return Err(Erreur { message: "un prix se donne par le nom de l'article et un nombre entier : Prices(sunrise: 120)".into(), pos: argument.pos });
        };
        if !quantites.iter().any(|(connu, _)| connu == nom) {
            return Err(Erreur { message: format!("le prix « {nom} » ne correspond à aucune valeur : déclare sa quantité, state: State({nom}: 0)"), pos: argument.pos });
        }
        if prix.iter().any(|(connu, _)| connu == nom) {
            return Err(Erreur { message: format!("le prix « {nom} » est donné deux fois"), pos: argument.pos });
        }
        if *montant > VALEUR_MAX {
            return Err(Erreur { message: format!("« {nom} » : un prix va de 0 à {VALEUR_MAX}"), pos: argument.pos });
        }
        prix.push((nom.clone(), *montant));
    }
    Ok(prix)
}

/// Ce que le moteur calcule à partir des quantités et des prix : `count` (le nombre
/// d'articles) et `total` (ce qu'ils coûtent). Rien si la page ne donne pas de prix.
pub fn calculees(programme: &Programme, etat: &Etat) -> Etat {
    let prix = prix(programme).unwrap_or_default();
    if programme.racine.argument("prices").is_none() {
        return Vec::new();
    }
    let (mut nombre, mut total) = (0u128, 0u128);
    for (nom, montant) in &prix {
        let quantite = etat.iter().find(|(connu, _)| connu == nom).map_or(0, |(_, q)| u128::from(*q));
        nombre += quantite;
        total += quantite * u128::from(*montant);
    }
    // Un total ne déborde jamais : au pire, il s'arrête au plus grand nombre que l'on sait écrire.
    let borner = |v: u128| u64::try_from(v).unwrap_or(u64::MAX);
    vec![("count".to_string(), borner(nombre)), ("total".to_string(), borner(total))]
}

/// Tout ce qu'un texte peut montrer : les valeurs de la page, puis celles que le moteur calcule.
pub fn a_montrer(programme: &Programme, etat: &Etat) -> Etat {
    let mut tout = etat.clone();
    tout.extend(calculees(programme, etat));
    tout
}

/// Les valeurs déclarées par la page, avec leur départ.
pub fn initial(programme: &Programme) -> Result<Etat, Erreur> {
    let etat = initial_sans_prix(programme)?;
    // Avec des prix, « count » et « total » sont calculés par le moteur : on ne les déclare pas.
    if programme.racine.argument("prices").is_some() {
        if let Some((nom, _)) = etat.iter().find(|(nom, _)| CALCULEES.contains(&nom.as_str())) {
            let pos = programme.racine.argument("state").map_or(programme.racine.pos, |a| a.pos);
            return Err(Erreur { message: format!("« {nom} » est calculé par le moteur quand la page donne des prix : ne le déclare pas dans « State »"), pos });
        }
    }
    // L'heure du visiteur, quand le fichier la lit (ADR-039).
    let mut etat = etat;
    let maintenant = maintenant();
    for nom in horloge_lue(programme) {
        let rang = HORLOGE.iter().position(|h| *h == nom).unwrap_or(0);
        etat.push((nom.to_string(), maintenant[rang]));
    }
    Ok(etat)
}

/// L'heure du visiteur, que le moteur donne comme il donne `count` et `total` (ADR-039) :
/// l'année, le mois (1 à 12), le jour (1 à 31), le jour de la semaine (1 lundi, 7 dimanche),
/// l'heure (0 à 23) et la minute. On la lit, on ne la change pas.
pub const HORLOGE: &[&str] = &["year", "month", "day", "weekday", "hour", "minute"];

thread_local! {
    /// L'heure donnée par celui qui appelle le moteur : la page (l'heure de l'appareil du
    /// visiteur), ou le serveur qui fabrique la page d'avance.
    static MAINTENANT: std::cell::Cell<[u64; 6]> = const { std::cell::Cell::new([2026, 1, 1, 4, 0, 0]) };
}

/// Donne l'heure au moteur : année, mois, jour, jour de la semaine, heure, minute.
pub fn regler_maintenant(valeurs: [u64; 6]) {
    MAINTENANT.with(|m| m.set(valeurs));
}

pub fn maintenant() -> [u64; 6] {
    MAINTENANT.with(std::cell::Cell::get)
}

/// L'heure en temps universel, d'après les secondes écoulées depuis le 1er janvier 1970.
/// Sert quand personne n'a donné l'heure du lieu.
pub fn depuis_secondes_unix(secondes: u64) -> [u64; 6] {
    let jours = (secondes / 86_400) as i64;
    let reste = secondes % 86_400;
    let z = jours + 719_468;
    let ere = z.div_euclid(146_097);
    let jour_de_l_ere = z - ere * 146_097;
    let annee_de_l_ere = (jour_de_l_ere - jour_de_l_ere / 1460 + jour_de_l_ere / 36_524 - jour_de_l_ere / 146_096) / 365;
    let jour_de_l_annee = jour_de_l_ere - (365 * annee_de_l_ere + annee_de_l_ere / 4 - annee_de_l_ere / 100);
    let m = (5 * jour_de_l_annee + 2) / 153;
    let jour = jour_de_l_annee - (153 * m + 2) / 5 + 1;
    let mois = if m < 10 { m + 3 } else { m - 9 };
    let annee = annee_de_l_ere + ere * 400 + i64::from(mois <= 2);
    // Le 1er janvier 1970 était un jeudi : le quatrième jour de la semaine.
    let semaine = (jours + 3).rem_euclid(7) + 1;
    [annee as u64, mois as u64, jour as u64, semaine as u64, reste / 3600, reste % 3600 / 60]
}

/// Les noms de l'heure que le fichier lit : dans un texte (`{hour}`), une condition, une
/// comparaison, une demande (`best.set(hour)`) ou une place.
fn horloge_lue(programme: &Programme) -> Vec<&'static str> {
    fn noter(nom: &str, lus: &mut Vec<&'static str>) {
        if let Some(h) = HORLOGE.iter().find(|h| **h == nom) {
            if !lus.contains(h) {
                lus.push(h);
            }
        }
    }
    fn visiter(valeur: &Valeur, lus: &mut Vec<&'static str>) {
        match valeur {
            Valeur::Texte(texte) => noms_dans(texte).into_iter().for_each(|nom| noter(nom, lus)),
            Valeur::Nom(nom) => noter(nom, lus),
            Valeur::Liste(elements) => elements.iter().for_each(|e| visiter(e, lus)),
            Valeur::Bloc(bloc) => bloc.arguments.iter().for_each(|a| visiter(&a.valeur, lus)),
            _ => {}
        }
    }
    let mut lus = Vec::new();
    programme.racine.arguments.iter().for_each(|a| visiter(&a.valeur, &mut lus));
    lus.sort_by_key(|h| HORLOGE.iter().position(|x| x == h));
    lus
}

/// Le fichier lit-il l'heure ? La page la tient alors à jour, minute après minute.
pub fn lit_l_heure(programme: &Programme) -> bool {
    !horloge_lue(programme).is_empty()
}

/// Une minute a passé : l'heure écrite dans l'état devient l'heure donnée au moteur, et les
/// règles qui guettent l'heure (`When(hour, is: 12, …)`) ont leur mot à dire.
pub fn avancer_l_horloge(programme: &Programme, ecrit: &str) -> Etat {
    let maintenant = relire(programme, ecrit);
    let mut avant = maintenant.clone();
    for morceau in ecrit.split(';') {
        if let Some((nom, valeur)) = morceau.split_once('=') {
            if let (true, Some((_, place)), Ok(valeur)) = (HORLOGE.contains(&nom), avant.iter_mut().find(|(connu, _)| connu == nom), valeur.parse::<u64>()) {
                *place = valeur;
            }
        }
    }
    suites(programme, avant, maintenant)
}

fn initial_sans_prix(programme: &Programme) -> Result<Etat, Erreur> {
    let Some(bloc) = bloc_d_etat(programme)? else { return Ok(Vec::new()) };
    let mut etat = Etat::new();
    for argument in &bloc.arguments {
        // Une valeur est un nombre entier (cart: 0) ou un texte (buyer: "").
        let (nom, depart) = match (&argument.nom, &argument.valeur) {
            (Some(nom), Valeur::Entier(depart)) => (nom, Some(*depart)),
            (Some(nom), Valeur::Texte(texte)) if texte.chars().count() <= TEXTE_MAX => (nom, None),
            (Some(nom), Valeur::Texte(_)) => return Err(Erreur { message: format!("« {nom} » : un texte fait au plus {TEXTE_MAX} caractères"), pos: argument.pos }),
            // Une liste de textes (ADR-044) : State(tasks: []).
            (Some(nom), Valeur::Liste(_)) => {
                crate::listes::verifier_declaration(argument)?;
                (nom, None)
            }
            _ => {
                return Err(Erreur {
                    message: "une valeur se déclare par son nom et son départ, un nombre entier, un texte ou une liste : State(cart: 0, buyer: \"\", tasks: [])".into(),
                    pos: argument.pos,
                })
            }
        };
        if !est_nom_de_valeur(nom) {
            return Err(Erreur { message: nom_de_valeur_mal_ecrit(nom), pos: argument.pos });
        }
        if HORLOGE.contains(&nom.as_str()) {
            return Err(Erreur { message: format!("« {nom} » est l'heure du visiteur, donnée par le moteur ; choisis un autre nom pour ta valeur (ADR-039)"), pos: argument.pos });
        }
        if bloc.arguments.iter().filter(|a| a.nom.as_deref() == Some(nom.as_str())).count() > 1 {
            return Err(Erreur { message: format!("la valeur « {nom} » est déclarée deux fois"), pos: argument.pos });
        }
        match depart {
            Some(depart) if depart > VALEUR_MAX => return Err(Erreur { message: format!("« {nom} » : une valeur va de 0 à {VALEUR_MAX}"), pos: argument.pos }),
            Some(depart) => etat.push((nom.clone(), depart)),
            None => {}
        }
    }
    if bloc.arguments.len() > VALEURS_MAX {
        return Err(Erreur { message: format!("trop de valeurs : une page en déclare au plus {VALEURS_MAX}"), pos: bloc.pos });
    }
    Ok(etat)
}

/// Une demande faite à l'arbitre : `cart.add(1)`.
#[derive(Debug, PartialEq)]
pub struct Demande<'a> {
    pub valeur: &'a str,
    pub verbe: &'a str,
    pub quantite: u64,
    /// Quand la quantité est une autre valeur : `best.set(score)`. Elle est lue au moment où
    /// la demande est faite.
    pub depuis: Option<&'a str>,
}

/// Un bloc est-il une demande ? Une demande commence par une minuscule : `cart.add(1)`.
pub fn est_demande(bloc: &Bloc) -> bool {
    bloc.nom.starts_with(|c: char| c.is_ascii_lowercase())
}

/// Lit et vérifie une demande, d'après les valeurs déclarées.
pub fn demande<'a>(bloc: &'a Bloc, etat: &Etat) -> Result<Demande<'a>, Erreur> {
    let erreur = |message: String| Erreur { message, pos: bloc.pos };
    let Some((valeur, verbe)) = bloc.nom.split_once('.') else {
        return Err(erreur(format!("« {} » : une demande s'écrit « cart.add(1) »", bloc.nom)));
    };
    if HORLOGE.contains(&valeur) {
        return Err(erreur(format!("« {valeur} » est l'heure du visiteur : on la lit, on ne la change pas")));
    }
    if !etat.iter().any(|(connu, _)| connu == valeur) {
        return Err(erreur(format!("aucune valeur ne s'appelle « {valeur} » : déclare-la sur la page, state: State({valeur}: 0)")));
    }
    if !DEMANDES.contains(&verbe) {
        return Err(erreur(format!("demande inconnue « {verbe} » : pour une valeur, on peut demander {}", DEMANDES.join(", "))));
    }
    match bloc.arguments.as_slice() {
        // La quantité peut être une autre valeur de la page : best.set(score).
        [Argument { nom: None, valeur: Valeur::Nom(autre), .. }] if etat.iter().any(|(connu, _)| connu == autre) => Ok(Demande { valeur, verbe, quantite: 0, depuis: Some(autre.as_str()) }),
        [Argument { nom: None, valeur: Valeur::Nom(autre), .. }] => Err(erreur(format!("« {valeur}.{verbe}({autre}) » : aucun nombre ne s'appelle « {autre} » ; déclare-le sur la page, state: State({autre}: 0)"))),
        [argument] if argument.nom.is_none() => match argument.valeur {
            // « random(0) » ne tirerait jamais que 0 : c'est sûrement une erreur.
            Valeur::Entier(0) if verbe == "div" => Err(erreur(format!("« {valeur}.div(0) » : on ne divise pas par 0"))),
            Valeur::Entier(0) if verbe == "random" => Err(erreur(format!("« {valeur}.random » attend le plus grand nombre possible, au moins 1 : {valeur}.random(100) tire de 0 à 100"))),
            Valeur::Entier(quantite) if quantite <= VALEUR_MAX => Ok(Demande { valeur, verbe, quantite, depuis: None }),
            _ => Err(erreur(format!("« {valeur}.{verbe} » attend un nombre entier de 0 à {VALEUR_MAX} : {valeur}.{verbe}(1)"))),
        },
        _ => Err(erreur(format!("« {valeur}.{verbe} » attend un seul nombre : {valeur}.{verbe}(1)"))),
    }
}

/// Les noms entre accolades d'un texte : `{cart}` dans « {cart} paintings ».
pub fn noms_dans(texte: &str) -> Vec<&str> {
    let mut noms = Vec::new();
    let mut reste = texte;
    while let Some(debut) = reste.find('{') {
        reste = &reste[debut + 1..];
        if let Some(fin) = reste.find('}') {
            // `{minute:00}` : la valeur `minute`, montrée avec un format (ADR-043).
            let nom = reste[..fin].split_once(':').map_or(&reste[..fin], |(nom, _)| nom);
            if nom.starts_with(|c: char| c.is_ascii_lowercase()) && nom.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                noms.push(nom);
            }
        }
    }
    noms
}

/// Vérifie l'état : la déclaration, sa place, et chaque `{nom}` écrit dans un texte.
/// Les demandes des règles sont vérifiées avec les règles (`regles.rs`).
pub fn verifier_etat(programme: &Programme) -> Result<Etat, Erreur> {
    let etat = initial(programme)?;
    prix(programme)?;
    gardees(programme)?;
    source_de_donnees(programme)?;
    // Ce qu'un texte peut montrer : les valeurs déclarées, nombres et textes, et celles que le
    // moteur calcule.
    let textes = textes_initiaux(programme);
    let mut montrables = avec_textes(&a_montrer(programme, &etat), &textes);
    montrables.extend(crate::listes::comptes(&crate::listes::initiales(programme)));
    let modeles = crate::listes::dans_un_modele(programme);
    let est_texte = |nom: &str| textes.iter().any(|(connu, _)| connu == nom);
    let declare = bloc_d_etat(programme)?;
    let prix_declares = match programme.racine.argument("prices").map(|a| &a.valeur) {
        Some(Valeur::Bloc(bloc)) => Some(bloc),
        _ => None,
    };
    pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if bloc.nom == "State" && !declare.is_some_and(|d| std::ptr::eq(d, bloc)) {
            return Err(Erreur { message: "« State » se déclare une seule fois, sur la page : state: State(cart: 0)".into(), pos: bloc.pos });
        }
        if (bloc.nom == "Prices" && !prix_declares.is_some_and(|d| std::ptr::eq(d, bloc))) || (bloc.argument("prices").is_some() && !std::ptr::eq(bloc, &programme.racine)) {
            return Err(Erreur { message: "« Prices » se donne une seule fois, sur la page : prices: Prices(sunrise: 120)".into(), pos: bloc.pos });
        }
        if bloc.argument("state").is_some() && !std::ptr::eq(bloc, &programme.racine) {
            return Err(Erreur { message: "les valeurs se déclarent sur la page, pas dans un monde : elles valent pour tout le fichier".into(), pos: bloc.pos });
        }
        // Un champ ou une case présente une valeur déclarée par la page, et dit ce qu'il attend.
        // Un choix présente un texte, et ses options sont des textes (ADR-038).
        if bloc.nom == "Choice" {
            let exemple = "Choice(value: size, label: \"Size\", options: [\"S\", \"M\", \"L\"])";
            for argument in &bloc.arguments {
                match (argument.nom.as_deref(), &argument.valeur) {
                    (Some("name"), _) | (Some("label"), Valeur::Texte(_)) | (Some("menu"), Valeur::Bool(_)) => {}
                    (Some("value"), Valeur::Nom(valeur)) if est_texte(valeur) => {}
                    (Some("value"), Valeur::Nom(valeur)) => {
                        return Err(Erreur { message: format!("« Choice(value: {valeur}) » : un choix présente un texte ; déclare-le ainsi : state: State({valeur}: \"\")"), pos: argument.pos })
                    }
                    (Some("options"), Valeur::Liste(options)) if (2..=20).contains(&options.len()) && options.iter().all(|o| matches!(o, Valeur::Texte(t) if !t.is_empty() && t.chars().count() <= 80)) => {}
                    (Some("options"), _) => return Err(Erreur { message: "« Choice(options: …) » attend de 2 à 20 textes entre guillemets : options: [\"S\", \"M\", \"L\"]".into(), pos: argument.pos }),
                    (Some(mot), _) => return Err(Erreur { message: format!("« Choice({mot}: …) » est mal écrit : {exemple}"), pos: argument.pos }),
                    (None, _) => return Err(Erreur { message: format!("chaque paramètre de « Choice » est nommé : {exemple}"), pos: argument.pos }),
                }
            }
            for requis in ["value", "label", "options"] {
                if bloc.argument(requis).is_none() {
                    return Err(Erreur { message: format!("« Choice » attend « {requis} » : {exemple}"), pos: bloc.pos });
                }
            }
            let options = options_du_choix(bloc);
            if options.iter().enumerate().any(|(i, o)| options[..i].contains(o)) {
                return Err(Erreur { message: "« Choice » : deux options ont le même texte".into(), pos: bloc.pos });
            }
        }
        if bloc.nom == "Input" || bloc.nom == "Checkbox" {
            let permis: &[&str] = if bloc.nom == "Input" { &["name", "value", "label", "max", "lines", "type"] } else { &["name", "value", "label"] };
            let exemple = if bloc.nom == "Input" { "Input(value: quantity, label: \"How many?\")" } else { "Checkbox(value: gift, label: \"Gift wrap\")" };
            for argument in &bloc.arguments {
                match (argument.nom.as_deref(), &argument.valeur) {
                    (Some("name"), _) | (Some("label"), Valeur::Texte(_)) => {}
                    (Some("value"), Valeur::Nom(valeur)) if HORLOGE.contains(&valeur.as_str()) => {
                        return Err(Erreur { message: format!("« {}(value: {valeur}) » : « {valeur} » est l'heure du visiteur ; on la lit, on ne l'écrit pas", bloc.nom), pos: argument.pos })
                    }
                    (Some("value"), Valeur::Nom(valeur)) if crate::listes::est_liste(programme, valeur) => {
                        return Err(Erreur { message: format!("« {}(value: {valeur}) » : « {valeur} » est une liste ; un champ présente un texte qu'on ajoute ensuite, {valeur}.push(task)", bloc.nom), pos: argument.pos })
                    }
                    (Some("value"), Valeur::Nom(valeur)) if etat.iter().any(|(connu, _)| connu == valeur) => {}
                    // Un champ peut présenter un texte ; une case, non.
                    (Some("value"), Valeur::Nom(valeur)) if est_texte(valeur) && bloc.nom == "Input" => {}
                    (Some("value"), Valeur::Nom(valeur)) if est_texte(valeur) => {
                        return Err(Erreur { message: format!("« Checkbox(value: {valeur}) » : « {valeur} » est un texte ; une case attend un nombre, state: State(gift: 0)"), pos: argument.pos })
                    }
                    (Some("value"), Valeur::Nom(valeur)) => {
                        return Err(Erreur { message: format!("« {}(value: {valeur}) » : aucune valeur ne s'appelle « {valeur} » ; déclare-la sur la page, state: State({valeur}: 0)", bloc.nom), pos: argument.pos })
                    }
                    (Some("max"), Valeur::Entier(max)) if bloc.nom == "Input" && *max <= VALEUR_MAX => {}
                    // Un texte long : de 2 à 20 lignes visibles, pour une valeur qui est un texte.
                    (Some("lines"), Valeur::Entier(n)) if bloc.nom == "Input" && (2..=20).contains(n) => {
                        if !matches!(bloc.argument("value").map(|a| &a.valeur), Some(Valeur::Nom(v)) if est_texte(v)) {
                            return Err(Erreur { message: "« Input(lines: …) » écrit un texte long : sa valeur est un texte, state: State(message: \"\")".into(), pos: argument.pos });
                        }
                    }
                    (Some("lines"), _) => return Err(Erreur { message: "« Input(lines: …) » attend un nombre de lignes, de 2 à 20".into(), pos: argument.pos }),
                    // Une date, une heure, une couleur (ADR-042) : la valeur est un texte.
                    (Some("type"), Valeur::Nom(t)) if bloc.nom == "Input" && ["date", "time", "color"].contains(&t.as_str()) => {
                        if !matches!(bloc.argument("value").map(|a| &a.valeur), Some(Valeur::Nom(v)) if est_texte(v)) {
                            return Err(Erreur { message: format!("« Input(type: {t}) » écrit un texte : sa valeur se déclare ainsi, state: State(arrivee: \"\")"), pos: argument.pos });
                        }
                    }
                    (Some("type"), _) => return Err(Erreur { message: "« Input(type: …) » attend date, time ou color ; un nombre ou un texte se devinent tout seuls".into(), pos: argument.pos }),
                    (Some(mot), _) if permis.contains(&mot) => return Err(Erreur { message: format!("« {}({mot}: …) » est mal écrit : {exemple}", bloc.nom), pos: argument.pos }),
                    (Some(mot), _) => return Err(Erreur { message: format!("« {} » n'a pas de paramètre « {mot} » ; paramètres possibles : {}", bloc.nom, permis.join(", ")), pos: argument.pos }),
                    (None, _) => return Err(Erreur { message: format!("chaque paramètre de « {} » est nommé : {exemple}", bloc.nom), pos: argument.pos }),
                }
            }
            // Un champ sans étiquette est un champ qu'un lecteur d'écran ne sait pas nommer.
            for requis in ["value", "label"] {
                if bloc.argument(requis).is_none() {
                    return Err(Erreur { message: format!("« {} » attend « {requis} » : {exemple}", bloc.nom), pos: bloc.pos });
                }
            }
        }
        // Une glissière présente un nombre de la page ; une barre de progression le montre (ADR-042).
        if bloc.nom == "Slider" || bloc.nom == "Progress" {
            let exemple = if bloc.nom == "Slider" { "Slider(value: volume, label: \"Volume\", min: 0, max: 100)" } else { "Progress(value: lives, max: 3, label: \"Lives\")" };
            for argument in &bloc.arguments {
                match (argument.nom.as_deref(), &argument.valeur) {
                    (Some("name"), _) | (Some("label"), Valeur::Texte(_)) => {}
                    (Some("value"), Valeur::Nom(valeur)) if HORLOGE.contains(&valeur.as_str()) && bloc.nom == "Slider" => {
                        return Err(Erreur { message: format!("« Slider(value: {valeur}) » : « {valeur} » est l'heure du visiteur ; on la lit, on ne l'écrit pas"), pos: argument.pos })
                    }
                    (Some("value"), Valeur::Nom(valeur)) if a_montrer(programme, &etat).iter().any(|(connu, _)| connu == valeur) => {}
                    (Some("value"), Valeur::Entier(_)) if bloc.nom == "Progress" => {}
                    (Some("value"), _) => return Err(Erreur { message: format!("« {}(value: …) » présente un nombre de la page : déclare-le, state: State(volume: 50) ; {exemple}", bloc.nom), pos: argument.pos }),
                    (Some("min"), Valeur::Entier(_)) if bloc.nom == "Slider" => {}
                    (Some("max"), Valeur::Entier(max)) if (1..=VALEUR_MAX).contains(max) => {}
                    (Some(mot @ ("min" | "max")), _) => return Err(Erreur { message: format!("« {}({mot}: …) » attend un nombre entier, de 1 à {VALEUR_MAX} pour max", bloc.nom), pos: argument.pos }),
                    (Some(mot), _) => return Err(Erreur { message: format!("« {}({mot}: …) » est mal écrit : {exemple}", bloc.nom), pos: argument.pos }),
                    (None, _) => return Err(Erreur { message: format!("chaque paramètre de « {} » est nommé : {exemple}", bloc.nom), pos: argument.pos }),
                }
            }
            for requis in ["value", "label"] {
                if bloc.argument(requis).is_none() {
                    return Err(Erreur { message: format!("« {} » attend « {requis} » : {exemple}", bloc.nom), pos: bloc.pos });
                }
            }
            let entier = |p: &str| match bloc.argument(p).map(|a| &a.valeur) {
                Some(Valeur::Entier(n)) => Some(*n),
                _ => None,
            };
            if bloc.nom == "Slider" && entier("min").unwrap_or(0) >= entier("max").unwrap_or(100) {
                return Err(Erreur { message: "« Slider » : min doit être plus petit que max".into(), pos: bloc.pos });
            }
        }
        // Une règle qui guette regarde un nombre : une valeur déclarée ou calculée.
        if bloc.nom == "When" && bloc.argument("meets").is_none() {
            let (valeur, _) = condition(bloc)?;
            if !a_montrer(programme, &etat).iter().any(|(connu, _)| connu == valeur) {
                return Err(Erreur { message: format!("« When({valeur}, …) » : aucun nombre ne s'appelle « {valeur} » ; déclare-le sur la page, state: State({valeur}: 0)"), pos: bloc.pos });
            }
        }
        // Une condition regarde une valeur que la page déclare, ou que le moteur calcule.
        if bloc.nom == "If" {
            let (valeur, _) = condition(bloc)?;
            if !montrables.iter().any(|(connu, _)| connu == valeur) {
                return Err(Erreur { message: format!("« If({valeur}, …) » : aucune valeur ne s'appelle « {valeur} » ; déclare-la sur la page, state: State({valeur}: 0)"), pos: bloc.pos });
            }
            // Un texte se compare au vide, un nombre à un nombre : pas de mélange.
            let au_vide = bloc.arguments.iter().any(|a| matches!((a.nom.as_deref(), &a.valeur), (Some("is" | "not"), Valeur::Texte(_))));
            let a_un_nombre = bloc.arguments.iter().any(|a| a.nom.as_deref().is_some_and(|mot| COMPARAISONS.contains(&mot)) && matches!(a.valeur, Valeur::Entier(_)));
            if est_texte(valeur) && a_un_nombre {
                return Err(Erreur { message: format!("« {valeur} » est un texte : on demande seulement s'il est vide, If({valeur}, is: \"\") ou If({valeur}, not: \"\")"), pos: bloc.pos });
            }
            if !est_texte(valeur) && au_vide {
                return Err(Erreur { message: format!("« {valeur} » est un nombre : on le compare à un nombre, If({valeur}, is: 0)"), pos: bloc.pos });
            }
        }
        // Une comparaison à une autre valeur : cette valeur doit être un nombre de la page.
        if bloc.nom == "If" || bloc.nom == "When" {
            for argument in &bloc.arguments {
                if let (Some(mot), Valeur::Nom(autre)) = (argument.nom.as_deref(), &argument.valeur) {
                    if COMPARAISONS.contains(&mot) && !a_montrer(programme, &etat).iter().any(|(connu, _)| connu == autre) {
                        return Err(Erreur { message: format!("« {mot}: {autre} » : aucun nombre ne s'appelle « {autre} » ; déclare-le sur la page, state: State({autre}: 0)"), pos: argument.pos });
                    }
                }
            }
        }
        // Sur un plateau, la place d'un bloc est une valeur de la page : Point(x: starX, y: starY).
        for axe in ["x", "y"] {
            if let Some(Argument { valeur: Valeur::Nom(valeur), pos, .. }) = bloc.argument(axe) {
                if !montrables.iter().any(|(connu, _)| connu == valeur) {
                    return Err(Erreur { message: format!("« {axe}: {valeur} » : aucune valeur ne s'appelle « {valeur} » ; déclare-la sur la page, state: State({valeur}: 50)"), pos: *pos });
                }
            }
        }
        let dans_un_modele = modeles.contains(&(bloc as *const Bloc));
        let verifier_texte = |texte: &str, pos| {
            for (nom, format) in crate::format::formats_dans(texte) {
                if !crate::format::est_format(format) {
                    return Err(Erreur { message: format!("« {{{nom}:{format}}} » : format inconnu ; formats possibles : 00 (zéros devant), number (1 234), cents (12,50), name (le nom du jour ou du mois)"), pos });
                }
                if format == "name" && nom != "weekday" && nom != "month" {
                    return Err(Erreur { message: format!("« {{{nom}:name}} » : seuls weekday et month ont un nom (mardi, octobre)"), pos });
                }
                if est_texte(nom) {
                    return Err(Erreur { message: format!("« {{{nom}:{format}}} » : « {nom} » est un texte ; un format s'applique à un nombre"), pos });
                }
            }
            if dans_un_modele && texte.contains("{item.") {
                return Err(Erreur { message: "dans les lignes d'une liste, « {item} » est le texte de l'élément ; il n'a pas de champs".into(), pos });
            }
            let texte = if dans_un_modele { texte.replace("{item}", "") } else { texte.to_string() };
            let texte = texte.as_str();
            if texte.contains("{item.") || texte.contains("{item}") {
                return Err(Erreur { message: "« {item…} » montre un champ de l'élément : il n'a de sens que dans une répétition, Repeat(items: [ … ], children: [ … ])".into(), pos });
            }
            noms_dans(texte).into_iter().find(|nom| !montrables.iter().any(|(connu, _)| connu == nom)).map_or(Ok(()), |nom| {
                if nom.contains('_') {
                    return Err(Erreur { message: format!("« {{{nom}}} » : deux mots se joignent comme en Flutter ; écris « {{{}}} » (ADR-037)", en_flutter(nom)), pos });
                }
                Err(Erreur { message: format!("« {{{nom}}} » : aucune valeur ne s'appelle « {nom} » ; déclare-la sur la page, state: State({nom}: 0)"), pos })
            })
        };
        for argument in &bloc.arguments {
            match (&argument.nom.as_deref(), &argument.valeur) {
                (None | Some("text"), Valeur::Texte(texte)) => verifier_texte(texte, argument.pos)?,
                // Les phrases seules et les lignes d'une liste.
                (Some("children" | "else"), Valeur::Liste(elements)) => {
                    for element in elements {
                        if let Valeur::Texte(texte) = element {
                            verifier_texte(texte, argument.pos)?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    })?;
    Ok(etat)
}

/// L'arbitre : ce que deviennent les valeurs quand ce signal est émis. Les demandes sont
/// appliquées dans l'ordre où les règles sont écrites. Une valeur ne descend pas sous 0 et ne
/// dépasse pas `VALEUR_MAX` : elle s'arrête à la borne.
pub fn arbitrer(programme: &Programme, etat: &Etat, signal: &str) -> Etat {
    let avant_le_signal = etat.clone();
    let mut etat = etat.clone();
    let graine = graine_du_hasard(programme);
    let mut tirages = etat.iter().find(|(nom, _)| nom == TIRAGES).map_or(0, |(_, n)| *n);
    let depart = tirages;
    let mut horloge = 0usize;
    let mut attente = 0usize;
    // Les règles rangées sous une condition ne répondent que si elle est vraie au moment du signal.
    avec_leur_vigueur(programme, &avant_le_signal, &mut |bloc, vigueur| {
        // Une règle répond à un signal : celui d'un bloc (`On(Add.tap, …)`), ou celui de sa
        // propre horloge (`Every(1s, …)` : « every:0 » pour la première règle de temps du
        // fichier, « every:1 » pour la deuxième).
        let concernee = match bloc.nom.as_str() {
            "On" => matches!(bloc.arguments.iter().find(|a| a.nom.is_none()).map(|a| &a.valeur), Some(Valeur::Nom(s)) if s == signal),
            "Every" => {
                horloge += 1;
                signal == format!("every:{}", horloge - 1)
            }
            // Une attente qui vient de finir : « after:0 » pour la première du fichier.
            "After" => {
                attente += 1;
                signal == format!("after:{}", attente - 1)
            }
            _ => false,
        };
        if concernee && vigueur {
            for effet in demandes_de(bloc) {
                appliquer(programme, &mut etat, effet, graine, &mut tirages);
            }
            // Les capacités d'une règle « On » sont appliquées par la page (elle connaît le
            // geste) ; celles d'une règle de temps sont notées ici.
            if bloc.nom == "Every" || bloc.nom == "After" {
                noter_les_capacites(bloc);
            }
        }
    });
    guetter(programme, avant_le_signal, &mut etat, graine, &mut tirages);
    noter_les_tirages(&mut etat, depart, tirages);
    etat
}

/// Ce qui suit un changement fait sans signal (une saisie, un glissement) : les règles qui
/// guettent ont leur mot à dire, comme après un geste.
fn suites(programme: &Programme, avant: Etat, mut etat: Etat) -> Etat {
    let graine = graine_du_hasard(programme);
    let depart = etat.iter().find(|(nom, _)| nom == TIRAGES).map_or(0, |(_, n)| *n);
    let mut tirages = depart;
    guetter(programme, avant, &mut etat, graine, &mut tirages);
    noter_les_tirages(&mut etat, depart, tirages);
    etat
}

fn noter_les_tirages(etat: &mut Etat, depart: u64, tirages: u64) {
    if tirages != depart {
        match etat.iter_mut().find(|(nom, _)| nom == TIRAGES) {
            Some((_, n)) => *n = tirages,
            None => etat.push((TIRAGES.to_string(), tirages)),
        }
    }
}

/// Le visiteur fait glisser un bloc posé sur un plateau (`drag: true`). Ses places, si ce sont
/// des valeurs de la page, suivent le doigt. C'est encore l'arbitre qui change les valeurs :
/// seulement pour un bloc qui se laisse glisser, et sans sortir du plateau.
pub fn glisser(programme: &Programme, etat: &Etat, nom: &str, x: u64, y: u64) -> Etat {
    let avant = etat.clone();
    let mut etat = etat.clone();
    let glissable = crate::regles::bloc_nomme(programme, nom).filter(|bloc| matches!(bloc.argument("drag").map(|a| &a.valeur), Some(Valeur::Bool(true))));
    if let Some(bloc) = glissable {
        for (axe, place) in [("x", x), ("y", y)] {
            if let Some(Valeur::Nom(valeur)) = bloc.argument(axe).map(|a| &a.valeur).filter(|v| !matches!(v, Valeur::Nom(n) if HORLOGE.contains(&n.as_str()))) {
                if let Some((_, v)) = etat.iter_mut().find(|(connu, _)| connu == valeur) {
                    *v = place.min(100);
                }
            }
        }
    }
    suites(programme, avant, etat)
}

/// Les règles qui guettent (`When`) : chacune se déclenche au moment où ce qu'elle guette
/// devient vrai, pas tant qu'il le reste.
fn guetter(programme: &Programme, avant_le_changement: Etat, etat: &mut Etat, graine: u64, tirages: &mut u64) {
    let etat_de_depart = avant_le_changement;
    {
    // Toutes celles qui se déclenchent en même temps jugent sur la même photo de l'état, puis
    // leurs effets s'appliquent dans l'ordre. Un effet peut en déclencher d'autres : on
    // recommence, huit fois au plus, pour qu'un fichier mal écrit ne tourne pas sans fin.
    let mut avant = etat_de_depart;
    for _ in 0..8 {
        let photo = etat.clone();
        let mut declenchee = false;
        let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
            let vu = |etat: &Etat| en_vigueur(programme, bloc, etat) && guette(programme, bloc, etat);
            if bloc.nom == "When" && !vu(&avant) && vu(&photo) {
                for effet in demandes_de(bloc) {
                    appliquer(programme, etat, effet, graine, tirages);
                    declenchee = true;
                }
                noter_les_capacites(bloc);
            }
            Ok(())
        });
        if !declenchee {
            break;
        }
        avant = photo;
    }
    }
}

/// Fait ce qu'une demande demande : `cart.add(1)`. Une valeur ne descend pas sous 0 et ne
/// dépasse pas son plafond.
fn appliquer(programme: &Programme, etat: &mut Etat, effet: &Bloc, graine: u64, tirages: &mut u64) {
    let Ok(d) = demande(effet, etat) else { return };
    let quantite = d.depuis.map_or(d.quantite, |autre| etat.iter().find(|(connu, _)| connu == autre).map_or(0, |(_, v)| *v));
    let plafond = plafond(programme, d.valeur);
    if let Some((_, valeur)) = etat.iter_mut().find(|(nom, _)| nom == d.valeur) {
        *valeur = match d.verbe {
            "add" => valeur.saturating_add(quantite),
            "sub" => valeur.saturating_sub(quantite),
            // Multiplier, diviser (ADR-043) : des nombres entiers ; la division arrondit vers le bas,
            // et une division par une valeur qui vaut 0 ne change rien.
            "mul" => valeur.saturating_mul(quantite),
            "div" if quantite == 0 => *valeur,
            "div" => *valeur / quantite,
            // Le hasard n'en est pas un : c'est le énième tirage d'une suite fixée par la graine
            // du fichier. Rejouer les mêmes gestes redonne les mêmes nombres.
            "random" => {
                *tirages = tirages.wrapping_add(1);
                crate::graine::melanger(graine ^ crate::graine::melanger(*tirages)) % (quantite + 1)
            }
            _ => quantite,
        }
        .min(plafond);
    }
}

/// La distance, sur un plateau, en deçà de laquelle deux blocs se rencontrent, quand la règle
/// ne le dit pas. Les places vont de 0 à 100.
pub const RENCONTRE: u64 = 10;

/// Les deux blocs d'une rencontre, `When(Basket, meets: Apple, within: 9, …)`, et la distance.
pub fn rencontre(regle: &Bloc) -> Result<(&str, &str, u64), Erreur> {
    let ecriture = "une rencontre s'écrit « When(Basket, meets: Apple, effect: score.add(1)) » : les noms de deux blocs posés sur un plateau";
    let (Some(Argument { nom: None, valeur: Valeur::Nom(a), .. }), Some(Valeur::Nom(b))) = (regle.arguments.first(), regle.argument("meets").map(|a| &a.valeur)) else {
        return Err(Erreur { message: ecriture.into(), pos: regle.pos });
    };
    let mut distance = RENCONTRE;
    for argument in &regle.arguments[1..] {
        match (argument.nom.as_deref(), &argument.valeur) {
            (Some("meets" | "effect"), _) => {}
            (Some("within"), Valeur::Entier(n)) if (1..=100).contains(n) => distance = *n,
            (Some("within"), _) => return Err(Erreur { message: "« When(…, within: …) » attend un nombre entier de 1 à 100 : la distance de rencontre, sur un plateau qui va de 0 à 100".into(), pos: argument.pos }),
            (Some(mot), _) => return Err(Erreur { message: format!("une rencontre n'a pas de paramètre « {mot} » ; paramètres possibles : meets, within, effect"), pos: argument.pos }),
            (None, _) => return Err(Erreur { message: ecriture.into(), pos: argument.pos }),
        }
    }
    Ok((a, b, distance))
}

/// Où est un bloc sur son plateau, d'après ses réglages `x` et `y` : un nombre écrit, ou une
/// valeur de la page.
fn place_de(programme: &Programme, etat: &Etat, nom: &str) -> Option<(u64, u64)> {
    let bloc = crate::regles::bloc_nomme(programme, nom)?;
    let lire = |axe: &str| match &bloc.argument(axe)?.valeur {
        Valeur::Entier(n) => Some((*n).min(100)),
        Valeur::Nom(valeur) => etat.iter().find(|(connu, _)| connu == valeur).map(|(_, v)| (*v).min(100)),
        _ => None,
    };
    Some((lire("x")?, lire("y")?))
}

/// La largeur d'un plateau, dans ses propres unités ; sa hauteur est `Board(height:)`. Le plateau
/// garde ses proportions à l'écran : les rencontres sont les mêmes sur tous les écrans.
pub const LARGEUR_DU_PLATEAU: f64 = 640.0;

/// Un objet posé sur un plateau : son centre et son encombrement, dans les unités du plateau.
struct Corps {
    cx: f64,
    cy: f64,
    demi: f64,
    rond: bool,
}

/// Où est un bloc, et quelle place il prend, sur son plateau.
fn corps(programme: &Programme, etat: &Etat, nom: &str) -> Option<Corps> {
    let bloc = crate::regles::bloc_nomme(programme, nom)?;
    let (x, y) = place_de(programme, etat, nom)?;
    let pixels = |bloc: &Bloc, param: &str| match bloc.argument(param).map(|a| &a.valeur) {
        Some(Valeur::Nombre { valeur, unite: Some(unite) }) if unite == "px" => Some(*valeur),
        _ => None,
    };
    // La taille du bloc, et la part de cette taille qu'on voit vraiment.
    let (taille, demi, rond) = match (bloc.nom.as_str(), bloc.argument("form").map(|a| &a.valeur)) {
        ("Shape", Some(Valeur::Nom(forme))) => {
            let taille = pixels(bloc, "size").unwrap_or(48.0);
            match forme.as_str() {
                "square" => (taille, taille / 2.0, false),
                "circle" => (taille, taille / 2.0, true),
                // Un triangle et un losange ne remplissent pas leur carré : un rond un peu plus petit.
                _ => (taille, taille * 0.4, true),
            }
        }
        // Un point est une lumière qui s'éteint vers le bord : son cœur fait un tiers de sa taille.
        ("Point", _) => (64.0, 64.0 * 0.35, true),
        _ => (48.0, 24.0, false),
    };
    // La hauteur du plateau où il est posé.
    let mut hauteur = 320.0;
    let _ = pour_chaque_bloc(&programme.racine, &mut |plateau| {
        if plateau.nom == "Board" {
            if let Some(Valeur::Liste(enfants)) = plateau.argument("children").map(|a| &a.valeur) {
                if enfants.iter().any(|e| matches!(e, Valeur::Bloc(b) if crate::regles::nom_de(b) == Some(nom))) {
                    hauteur = pixels(plateau, "height").unwrap_or(320.0);
                }
            }
        }
        Ok(())
    });
    let largeur = LARGEUR_DU_PLATEAU;
    // À 0 le bloc touche un bord, à 100 l'autre : son centre parcourt le plateau moins sa taille.
    Some(Corps { cx: x as f64 / 100.0 * (largeur - taille) + taille / 2.0, cy: y as f64 / 100.0 * (hauteur - taille) + taille / 2.0, demi, rond })
}

/// Deux objets se touchent-ils ? Le bord de l'un atteint le bord de l'autre.
fn se_touchent(a: &Corps, b: &Corps) -> bool {
    let (dx, dy) = ((a.cx - b.cx).abs(), (a.cy - b.cy).abs());
    match (a.rond, b.rond) {
        (true, true) => dx.hypot(dy) <= a.demi + b.demi,
        (false, false) => dx <= a.demi + b.demi && dy <= a.demi + b.demi,
        // Un rond et un carré : le point du carré le plus proche du centre du rond.
        _ => {
            let (rond, carre) = if a.rond { (a, b) } else { (b, a) };
            (dx - carre.demi).max(0.0).hypot((dy - carre.demi).max(0.0)) <= rond.demi
        }
    }
}

/// Ce qu'une règle `When` guette est-il vrai, pour cet état ? Une valeur (`When(lives, is: 0)`),
/// ou la rencontre de deux blocs (`When(Basket, meets: Apple)`).
fn guette(programme: &Programme, regle: &Bloc, etat: &Etat) -> bool {
    if regle.argument("meets").is_some() {
        return rencontre(regle).is_ok_and(|(a, b, distance)| {
            // Avec « within », on juge sur l'écart entre les places, de 0 à 100. Sans lui, sur le
            // contact : le bord de l'un touche le bord de l'autre.
            if regle.argument("within").is_some() {
                return match (place_de(programme, etat, a), place_de(programme, etat, b)) {
                    (Some((ax, ay)), Some((bx, by))) => ax.abs_diff(bx) <= distance && ay.abs_diff(by) <= distance,
                    _ => false,
                };
            }
            match (corps(programme, etat, a), corps(programme, etat, b)) {
                (Some(a), Some(b)) => se_touchent(&a, &b),
                _ => false,
            }
        });
    }
    condition(regle).is_ok_and(|(valeur, comparaisons)| {
        let montrees = a_montrer(programme, etat);
        montrees.iter().find(|(connu, _)| connu == valeur).is_some_and(|(_, nombre)| vraie(&comparaisons, *nombre, &montrees))
    })
}

/// Les touches du clavier que les règles du fichier écoutent : `On(Key.left, …)`.
pub const TOUCHES: &[&str] = &["left", "right", "up", "down", "space"];

pub fn touches(programme: &Programme) -> Vec<String> {
    let mut touches = Vec::new();
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if let ("On", Some(Valeur::Nom(signal))) = (bloc.nom.as_str(), bloc.arguments.iter().find(|a| a.nom.is_none()).map(|a| &a.valeur)) {
            if let Some(touche) = signal.strip_prefix("Key.") {
                if !touches.iter().any(|t| t == touche) {
                    touches.push(touche.to_string());
                }
            }
        }
        Ok(())
    });
    touches
}

/// `cart=2;likes=0` : l'état, pour le garder d'un geste à l'autre du côté de la page.
pub fn ecrire(etat: &Etat) -> String {
    etat.iter().map(|(nom, valeur)| format!("{nom}={valeur}")).collect::<Vec<_>>().join(";")
}

/// Relit un état écrit par `ecrire`. Seules les valeurs que la page déclare sont reprises, et
/// jamais au-delà des bornes : un état abîmé ou falsifié ne fait rien de plus que le départ.
pub fn relire(programme: &Programme, ecrit: &str) -> Etat {
    let mut etat = initial(programme).unwrap_or_default();
    for morceau in ecrit.split(';') {
        if let Some((nom, valeur)) = morceau.split_once('=') {
            // Le compte des tirages au hasard suit l'état, pour que la suite continue.
            if let (true, Ok(n)) = (nom == TIRAGES, valeur.parse::<u64>()) {
                etat.push((TIRAGES.to_string(), n));
                continue;
            }
            // L'heure n'est jamais reprise de l'état écrit : c'est celle donnée au moteur.
            if HORLOGE.contains(&nom) {
                continue;
            }
            if let (Some((_, place)), Ok(valeur)) = (etat.iter_mut().find(|(connu, _)| connu == nom), valeur.parse::<u64>()) {
                *place = valeur.min(VALEUR_MAX);
            }
        }
    }
    etat
}

#[cfg(test)]
mod tests {
    use super::*;

    const PANIER: &str = "Page(
  state: State(cart: 0, likes: 3),
  children: [
    Text(\"{cart} paintings in the cart\"),
    Button(name: Add, text: \"Add ({cart})\"),
    Button(name: Remove, text: \"Remove\"),
    Button(name: Empty, text: \"Empty\"),
    \"{likes} people like this shop.\",
  ],
  rules: [
    On(Add.tap, effect: cart.add(1)),
    On(Remove.tap, effect: cart.sub(1)),
    On(Empty.tap, effect: cart.set(0)),
    On(Empty.tap, effect: likes.add(1)),
  ],
)";

    fn page(source: &str) -> Result<Programme, Erreur> {
        crate::verifier_page(source)
    }

    #[test]
    fn le_panier_se_remplit_et_se_vide_par_des_demandes() {
        let programme = page(PANIER).unwrap();
        let depart = initial(&programme).unwrap();
        assert_eq!(ecrire(&depart), "cart=0;likes=3");
        let deux = arbitrer(&programme, &arbitrer(&programme, &depart, "Add.tap"), "Add.tap");
        assert_eq!(ecrire(&deux), "cart=2;likes=3");
        assert_eq!(ecrire(&arbitrer(&programme, &deux, "Remove.tap")), "cart=1;likes=3");
        // Un signal, deux règles : les deux demandes sont faites, dans l'ordre.
        assert_eq!(ecrire(&arbitrer(&programme, &deux, "Empty.tap")), "cart=0;likes=4");
        // Un signal sans règle ne change rien.
        assert_eq!(arbitrer(&programme, &deux, "Nobody.tap"), deux);
    }

    #[test]
    fn une_valeur_reste_entre_ses_bornes() {
        let programme = page(PANIER).unwrap();
        let depart = initial(&programme).unwrap();
        // On ne descend pas sous zéro.
        assert_eq!(ecrire(&arbitrer(&programme, &depart, "Remove.tap")), "cart=0;likes=3");
        // On ne dépasse pas le plafond, même en partant d'un état falsifié.
        let plein = relire(&programme, "cart=99999999999999;likes=7;intrus=4;cart");
        assert_eq!(ecrire(&plein), format!("cart={VALEUR_MAX};likes=7"));
        assert_eq!(ecrire(&arbitrer(&programme, &plein, "Add.tap")), format!("cart={VALEUR_MAX};likes=7"));
    }

    #[test]
    fn la_page_affiche_les_valeurs_dans_ses_textes() {
        let html = crate::vue_a_plat(PANIER, "").unwrap();
        assert!(html.contains("<span data-state=\"cart\">0</span> paintings in the cart"), "{html}");
        assert!(html.contains("Add (<span data-state=\"cart\">0</span>)"), "{html}");
        assert!(html.contains("<span data-state=\"likes\">3</span> people"), "{html}");
        // Des accolades qui n'entourent pas un nom restent du texte.
        let html = crate::vue_a_plat("Page(children: [ \"{ } and {Not A Name} and {}\" ])", "").unwrap();
        assert!(html.contains("{ } and {Not A Name} and {}"), "{html}");
    }

    const BOUTIQUE: &str = "Page(
  state: State(sunrise: 0, blueDoor: 2, likes: 5),
  prices: Prices(sunrise: 120, blueDoor: 90),
  children: [
    Text(\"{count} paintings, {total} euros\"),
    Button(name: Add, text: \"Add\"),
  ],
  rules: [ On(Add.tap, effect: sunrise.add(1)) ],
)";

    #[test]
    fn avec_des_prix_le_moteur_compte_et_additionne() {
        let programme = page(BOUTIQUE).unwrap();
        let depart = initial(&programme).unwrap();
        // « likes » n'a pas de prix : ce n'est pas un article, il ne compte pas.
        assert_eq!(ecrire(&a_montrer(&programme, &depart)), "sunrise=0;blueDoor=2;likes=5;count=2;total=180");
        let apres = arbitrer(&programme, &depart, "Add.tap");
        assert_eq!(ecrire(&a_montrer(&programme, &apres)), "sunrise=1;blueDoor=2;likes=5;count=3;total=300");
        assert!(crate::vue_a_plat(BOUTIQUE, "").unwrap().contains("<span data-state=\"count\">2</span> paintings, <span data-state=\"total\">180</span> euros"));
        // Ce que la page renvoie contient les valeurs calculées ; elles ne sont pas reprises telles
        // quelles : le moteur les recalcule toujours.
        assert_eq!(crate::arbitrer(BOUTIQUE, "sunrise=1;blueDoor=2;likes=5;count=999;total=1", "Add.tap"), "sunrise=2;blueDoor=2;likes=5;count=4;total=420");
        // Sans prix, pas de valeurs calculées : « count » est un nom libre.
        assert_eq!(crate::etat_initial("Page(state: State(count: 7))"), "count=7");
        // Un total ne déborde pas.
        let enorme = page("Page(state: State(a: 1000000000), prices: Prices(a: 1000000000))").unwrap();
        assert_eq!(ecrire(&calculees(&enorme, &initial(&enorme).unwrap())), "count=1000000000;total=1000000000000000000");
    }

    #[test]
    fn une_condition_montre_ou_cache_selon_une_valeur() {
        let source = "Page(
  state: State(cart: 0),
  children: [
    If(cart, is: 0, children: [ \"Your cart is empty.\" ]),
    If(cart, over: 0, under: 3, children: [ Button(name: Pay, text: \"Pay\") ]),
    If(cart, not: 0, children: [ \"{cart} in your cart\" ]),
  ],
)";
        let html = crate::vue_a_plat(source, "").unwrap();
        // Au départ, le panier est vide : la première condition est vraie, les deux autres non.
        assert!(html.contains("<div class=\"holo-If\" data-if=\"cart|is=0\"><p class=\"holo-P\">Your cart is empty.</p></div>"), "{html}");
        assert!(html.contains("<div class=\"holo-If\" data-if=\"cart|over=0|under=3\" hidden><button"), "{html}");
        assert!(html.contains("data-if=\"cart|not=0\" hidden>"), "{html}");
        // Après un changement, c'est encore le moteur qui répond, par le même calcul.
        assert_eq!(crate::conditions(source, "cart=0"), "cart|is=0:1;cart|over=0|under=3:0;cart|not=0:0");
        assert_eq!(crate::conditions(source, "cart=2"), "cart|is=0:0;cart|over=0|under=3:1;cart|not=0:1");
        assert_eq!(crate::conditions(source, "cart=3"), "cart|is=0:0;cart|over=0|under=3:0;cart|not=0:1");
        let (entre, rien) = ([("over", Terme::Nombre(0)), ("under", Terme::Nombre(3))], Etat::new());
        assert!(vraie(&entre, 2, &rien) && !vraie(&entre, 3, &rien) && !vraie(&entre, 0, &rien));
        assert!(vraie(&[("is", Terme::Nombre(5))], 5, &rien) && vraie(&[("not", Terme::Nombre(5))], 4, &rien) && !vraie(&[("not", Terme::Nombre(5))], 5, &rien));
        // Avec des prix, une condition peut regarder ce que le moteur calcule.
        page("Page(state: State(a: 0), prices: Prices(a: 10), children: [ If(total, over: 100, children: [ \"Free delivery\" ]) ])").unwrap();
        for (source, message) in [
            ("Page(children: [ If(cart, is: 0, children: []) ])", "aucune valeur ne s'appelle « cart »"),
            ("Page(state: State(cart: 0), children: [ If(cart, children: []) ])", "une condition s'écrit"),
            ("Page(state: State(cart: 0), children: [ If(cart, is: \"zero\", children: []) ])", "attend un nombre entier"),
            ("Page(state: State(cart: 0), children: [ If(cart, above: 0, children: []) ])", "n'a pas de paramètre « above »"),
            ("Page(state: State(cart: 0), children: [ If(cart, is: 0) ])", "attend ce qu'il montre"),
            ("Page(state: State(cart: 0), children: [ If(is: 0, children: []) ])", "une condition s'écrit"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    const JEU: &str = include_str!("../../exemples/jeu/attraper.holo");

    #[test]
    fn le_jeu_se_joue_par_des_regles_le_temps_et_le_hasard() {
        let programme = page(JEU).unwrap();
        // Trois règles de temps, trois horloges : le temps chaque seconde, l'étoile toutes les deux.
        assert_eq!(horloges(&programme), [(1000, "time".to_string()), (2000, "starX,starY".to_string())]);
        let depart = initial(&programme).unwrap();
        assert_eq!(ecrire(&depart), "time=0;score=0;starX=50;starY=50;best=0");
        // Tant que la partie n'a pas commencé, le temps reste à zéro : il ne descend pas dessous.
        assert_eq!(arbitrer(&programme, &depart, "every:0")[0], ("time".to_string(), 0));
        // « Play » : trente secondes. Ce geste change le temps : son horloge repartira de zéro.
        let lancee = arbitrer(&programme, &depart, "Play.tap");
        assert_eq!((lancee[0].1, lancee[1].1), (30, 0));
        assert_eq!(touchees(&programme, "Play.tap"), ["score", "time"]);
        // Une seconde passe : seul le temps change. L'étoile a sa propre horloge.
        let une_seconde = arbitrer(&programme, &lancee, "every:0");
        assert_eq!((une_seconde[0].1, une_seconde[2].1, une_seconde[3].1), (29, 50, 50));
        let bougee = arbitrer(&programme, &une_seconde, "every:1");
        assert!(bougee[2].1 <= 100 && bougee[3].1 <= 100);
        assert_ne!((bougee[2].1, bougee[3].1), (50, 50), "l'étoile n'a pas bougé");
        // Toucher l'étoile : un point, elle part ailleurs, et son horloge repart : elle reste là
        // deux vraies secondes.
        let touchee = arbitrer(&programme, &bougee, "Star.tap");
        assert_eq!(touchee[1].1, 1);
        assert_ne!((touchee[2].1, touchee[3].1), (bougee[2].1, bougee[3].1));
        assert_eq!(touchees(&programme, "Star.tap"), ["score", "starX", "starY"]);
        // Trente secondes plus tard, la partie est finie, et le score est gardé.
        let fin = (0..40).fold(touchee, |etat, _| arbitrer(&programme, &etat, "every:0"));
        assert_eq!((fin[0].1, fin[1].1), (0, 1));
    }

    const PANIER_DE_POMMES: &str = include_str!("../../exemples/jeu/panier.holo");

    /// La règle de rencontre du jeu de la pomme.
    fn programme_regle(programme: &Programme) -> &Bloc {
        let mut regle = None;
        let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
            if bloc.nom == "When" && bloc.argument("meets").is_some() {
                regle = Some(bloc);
            }
            Ok(())
        });
        regle.unwrap()
    }

    #[test]
    fn le_clavier_ce_qui_tombe_et_les_rencontres() {
        let programme = page(PANIER_DE_POMMES).unwrap();
        assert_eq!(touches(&programme), ["left", "right"]);
        let valeur = |etat: &Etat, nom: &str| etat.iter().find(|(connu, _)| connu == nom).unwrap().1;
        let depart = initial(&programme).unwrap();
        let joue = arbitrer(&programme, &depart, "Play.tap");
        assert_eq!((valeur(&joue, "lives"), valeur(&joue, "basket"), valeur(&joue, "appleY")), (3, 50, 0));
        // Le clavier déplace le panier, qui ne sort jamais du plateau.
        let gauche = (0..20).fold(joue.clone(), |etat, _| arbitrer(&programme, &etat, "Key.left"));
        assert_eq!(valeur(&gauche, "basket"), 0);
        let droite = (0..40).fold(gauche, |etat, _| arbitrer(&programme, &etat, "Key.right"));
        assert_eq!(valeur(&droite, "basket"), 100);
        // La pomme tombe. Le panier est dessous (50 et 50) : à la rencontre, un point, et une
        // nouvelle pomme repart d'en haut. Une seule fois, pas à chaque battement.
        let mut etat = joue.clone();
        let mut battements = 0;
        while valeur(&etat, "score") == 0 && battements < 60 {
            etat = arbitrer(&programme, &etat, "every:0");
            battements += 1;
        }
        assert_eq!((valeur(&etat, "score"), valeur(&etat, "appleY"), valeur(&etat, "lives")), (1, 0, 3), "après {battements} battements");
        assert!(battements > 20, "la pomme a été prise trop tôt : {battements}");
        // Le panier parti loin, la pomme arrive en bas : une vie de moins, une seule, et une
        // nouvelle pomme.
        let mut etat = (0..20).fold(etat, |e, _| arbitrer(&programme, &e, "Key.left"));
        let pomme_a_droite = PANIER_DE_POMMES.replace("appleX.random(100)", "appleX.set(90)");
        let programme = page(&pomme_a_droite).unwrap();
        etat.iter_mut().find(|(nom, _)| nom == "appleX").unwrap().1 = 90;
        let mut battements = 0;
        while valeur(&etat, "lives") == 3 && battements < 60 {
            etat = arbitrer(&programme, &etat, "every:0");
            battements += 1;
        }
        assert_eq!((valeur(&etat, "lives"), valeur(&etat, "appleY"), valeur(&etat, "score")), (2, 0, 1));
        // Trois pommes perdues : la partie est finie.
        let fin = (0..200).fold(etat, |e, _| arbitrer(&programme, &e, "every:0"));
        assert_eq!((valeur(&fin, "lives"), valeur(&fin, "score")), (0, 1));
        for (source, message) in [
            ("Page(state: State(a: 0), rules: [ When(a, effect: a.set(0)) ])", "une règle qui guette s'écrit"),
            ("Page(state: State(a: 0), rules: [ When(b, is: 1, effect: a.set(0)) ])", "aucun nombre ne s'appelle « b »"),
            ("Page(state: State(a: 0), rules: [ When(a, is: 1) ])", "attend une demande"),
            ("Page(state: State(a: 0), rules: [ When(a, is: 1, children: []) ])", "n'a pas de paramètre « children »"),
            ("Page(state: State(a: \"\", b: 0), rules: [ When(a, is: \"\", effect: b.set(1)) ])", "aucun nombre ne s'appelle « a »"),
            ("Page(state: State(a: 0), children: [ Board(children: [ Point(name: A, seed: 1, x: 1, y: 1) ]) ], rules: [ When(A, meets: 3, effect: a.add(1)) ])", "une rencontre s'écrit"),
            ("Page(state: State(a: 0), children: [ Board(children: [ Point(name: A, seed: 1, x: 1, y: 1) ]), P(name: B, \"x\") ], rules: [ When(A, meets: B, effect: a.add(1)) ])", "« B » n'est pas posé sur un plateau"),
            ("Page(state: State(a: 0), children: [ Board(children: [ Point(name: A, seed: 1, x: 1, y: 1), Point(name: B, seed: 2, x: 2, y: 2) ]) ], rules: [ When(A, meets: B, within: 500, effect: a.add(1)) ])", "de 1 à 100"),
            ("Page(state: State(a: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(Key.enter, effect: a.add(1)) ])", "le clavier donne"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
        // Les règles du jeu sont rangées sous une condition : tant que la partie n'a pas commencé
        // (aucune vie), la pomme ne tombe pas, et rien n'est rattrapé ni perdu en cachette.
        capacites_demandees();
        let avant_de_jouer = (0..60).fold(depart.clone(), |e, _| arbitrer(&programme, &e, "every:0"));
        assert_eq!(avant_de_jouer, depart, "le jeu a joué tout seul avant « Play »");
        assert!(capacites_demandees().is_empty(), "un son a été demandé avant « Play »");
        // Le contact, pas la pénétration : la pomme (un rond de 44) est prise au moment où son
        // bord touche le dessus du panier (un carré de 64), pas quand elle est déjà dedans.
        let programme = page(PANIER_DE_POMMES).unwrap();
        let avec = |y: u64| { let mut e = joue.clone(); e.iter_mut().find(|(n, _)| n == "appleY").unwrap().1 = y; e };
        // Plateau de 360 : le dessus du panier est à 284 ; le bas de la pomme est à y/100 × 316 + 44.
        assert!(!guette(&programme, programme_regle(&programme), &avec(75)), "à 75, le bas de la pomme est à 281 : elle ne touche pas encore");
        assert!(guette(&programme, programme_regle(&programme), &avec(76)), "à 76, le bas de la pomme est à 284 : elle touche");
        // Sur le côté : le panier décalé d'un peu plus que la moitié des deux largeurs ne touche plus.
        let mut a_cote = avec(96);
        a_cote.iter_mut().find(|(n, _)| n == "basket").unwrap().1 = 59;
        assert!(guette(&programme, programme_regle(&programme), &a_cote));
        a_cote.iter_mut().find(|(n, _)| n == "basket").unwrap().1 = 60;
        assert!(!guette(&programme, programme_regle(&programme), &a_cote));
        // Le plateau garde ses proportions : une page qui annoncerait la largeur de son écran
        // (comme avant) ne change rien à la rencontre.
        assert_eq!(ecrire(&relire(&programme, &format!("{};<=320", ecrire(&a_cote)))), ecrire(&a_cote));
        // Comme dans le navigateur : l'état voyage en texte. La pomme tombe, et elle est prise.
        let mut texte = crate::arbitrer(PANIER_DE_POMMES, &crate::etat_initial(PANIER_DE_POMMES), "Play.tap");
        let mut battements = 0;
        while !texte.contains("score=1") && battements < 60 {
            texte = crate::arbitrer(PANIER_DE_POMMES, &texte, "every:0");
            battements += 1;
        }
        assert!(texte.contains("score=1") && texte.contains("lives=3"), "après {battements} battements : {texte}");
        // Faire glisser le panier : sa valeur suit le doigt, sans sortir du plateau ; un bloc qui
        // ne se laisse pas glisser ne bouge pas ; et une rencontre faite en glissant compte.
        let programme = page(PANIER_DE_POMMES).unwrap();
        let glisse = glisser(&programme, &joue, "Basket", 250, 10);
        assert_eq!((valeur(&glisse, "basket"), valeur(&glisse, "appleY")), (100, 0));
        assert_eq!(glisser(&programme, &joue, "Apple", 10, 90), joue);
        let mut pres = joue.clone();
        pres.iter_mut().find(|(nom, _)| nom == "appleY").unwrap().1 = 90;
        pres.iter_mut().find(|(nom, _)| nom == "basket").unwrap().1 = 10;
        let rattrapee = glisser(&programme, &pres, "Basket", 50, 0);
        assert_eq!((valeur(&rattrapee, "score"), valeur(&rattrapee, "appleY")), (1, 0));
        // Une règle peut faire plusieurs demandes, dans l'ordre ; une seule s'écrit sans crochets.
        let plusieurs = page("Page(state: State(a: 0, b: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: [a.add(2), b.set(7), a.add(1)]) ])").unwrap();
        assert_eq!(ecrire(&arbitrer(&plusieurs, &initial(&plusieurs).unwrap(), "B.tap")), "a=3;b=7");
        // Un fichier où deux règles se relancent l'une l'autre ne tourne pas sans fin.
        let boucle = page("Page(state: State(a: 0, b: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: a.set(1)), When(a, is: 1, effect: a.set(0)), When(a, is: 0, effect: a.set(1)) ])").unwrap();
        let _ = arbitrer(&boucle, &initial(&boucle).unwrap(), "B.tap");
    }

    #[test]
    fn les_donnees_d_un_serveur_passent_par_l_arbitre() {
        let source = "Page(
  state: State(stock: 0, message: \"\", cart: 2, ouvert: 0, alerte: 0),
  data: Data(from: \"stock.json\", every: 30s),
  children: [ Text(\"{stock} en stock. {message}\"), Input(value: cart, label: \"x\", max: 5) ],
  rules: [ When(stock, is: 0, effect: alerte.set(1)) ],
)";
        let programme = page(source).unwrap();
        assert_eq!(source_de_donnees(&programme).unwrap(), Some(("stock.json".to_string(), 30_000)));
        assert_eq!(crate::donnees(source), "stock.json|30000");
        // Un nombre va dans un nombre, un texte dans un texte ; le reste est laissé de côté.
        let depart = crate::etat_initial(source);
        let recu = crate::recevoir(source, &depart, r#"{ "stock": 4, "message": "Ouvert \"aujourd'hui\"", "ouvert": true, "cart": 99, "inconnu": 7, "prix": 3.5, "liste": [1, {"a": "}"}], "rien": null, "stock2": -1 }"#);
        assert_eq!(recu, format!("stock=4;cart=5;ouvert=1;alerte=0;message='{}", coder("Ouvert \"aujourd'hui\"")));
        // Un texte offert à un nombre, ou l'inverse, ne change rien.
        assert_eq!(crate::recevoir(source, &depart, r#"{"stock": "beaucoup", "message": 12}"#), depart);
        // Un fichier mal formé, ou trop gros, ne change rien.
        for mauvais in ["", "[1, 2]", "{\"stock\": 4", "{stock: 4}", "<html>", &format!("{{\"message\": \"{}\"}}", "x".repeat(DONNEES_OCTETS))] {
            assert_eq!(crate::recevoir(source, &depart, mauvais), depart, "{}", &mauvais[..mauvais.len().min(30)]);
        }
        // Les règles qui guettent voient arriver les données : le stock tombe à zéro.
        let plein = crate::recevoir(source, &depart, r#"{"stock": 3}"#);
        assert!(crate::recevoir(source, &plein, r#"{"stock": 0}"#).contains("alerte=1"));
        // Sans « data: », rien n'est reçu.
        let sans = source.replace("  data: Data(from: \"stock.json\", every: 30s),\n", "");
        assert_eq!(crate::recevoir(&sans, &crate::etat_initial(&sans), r#"{"stock": 4}"#), crate::etat_initial(&sans));
        assert_eq!(crate::donnees(&sans), "");
        for (source, message) in [
            ("Page(data: Data(from: \"https://ailleurs.example/x.json\"))", "rangé à côté"),
            ("Page(data: Data(from: \"../secret.json\"))", "rangé à côté"),
            ("Page(data: Data(from: \"stock.txt\"))", "rangé à côté"),
            ("Page(data: Data(every: 30s))", "attend « from »"),
            ("Page(data: Data(from: \"a.json\", every: 10ms))", "de 1s à 3600s"),
            ("Page(data: Data(from: \"a.json\", fill: 3))", "n'a pas de paramètre « fill »"),
            ("Page(data: 3)", "un bloc « Data(...) »"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn un_son_se_joue_par_une_regle() {
        let source = "Page(
  state: State(n: 0),
  children: [ Sound(name: Ding, source: \"ding.wav\"), Button(name: B, text: \"x\"), Point(name: P, seed: 1, inside: World(children: [])) ],
  rules: [
    On(B.tap, effect: [n.add(1), Ding.play]),
    When(n, is: 2, effect: [n.set(0), Ding.play]),
    Every(1s, effect: Ding.play),
  ],
)";
        page(source).unwrap();
        let html = crate::vue_a_plat(source, "/x/").unwrap();
        assert!(html.contains("<audio class=\"holo-Sound\" data-name=\"Ding\" preload=\"auto\" src=\"/x/ding.wav\"></audio>"), "{html}");
        // Le son demandé par un geste est donné à la page avec les autres effets du geste.
        assert_eq!(crate::effets(source, "B.tap"), ["Ding.play"]);
        // Celui d'une règle qui guette, ou d'une règle de temps, suit l'état, sous le nom « ! ».
        let un = crate::arbitrer(source, &crate::etat_initial(source), "B.tap");
        assert_eq!(un, "n=1");
        assert_eq!(crate::arbitrer(source, &un, "B.tap"), "n=0;!=Ding.play");
        assert_eq!(crate::arbitrer(source, &un, "every:0"), "n=1;!=Ding.play");
        // Ce « ! » n'est pas une valeur : relu, il est laissé de côté.
        assert_eq!(crate::arbitrer(source, "n=1;!=Ding.play", "Nobody.tap"), "n=1");
        for (source, message) in [
            ("Page(children: [ Sound(name: D, source: \"https://x.example/a.wav\") ])", "un fichier de son rangé à côté"),
            ("Page(children: [ Sound(name: D, source: \"a.exe\") ])", "un fichier de son rangé à côté"),
            ("Page(children: [ Sound(source: \"a.wav\") ])", "un son a un nom"),
            ("Page(children: [ Sound(name: D, source: \"a.wav\", loop: true) ])", "n'a pas de paramètre « loop »"),
            ("Page(children: [ Sound(name: D, source: \"a.wav\"), Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: D.stop) ])", "capacité inconnue « stop »"),
            ("Page(state: State(n: 0), children: [ Point(name: P, seed: 1, inside: World(children: [])) ], rules: [ Every(1s, effect: P.enter) ])", "demande un geste du visiteur"),
            ("Page(state: State(n: 0), children: [ Point(name: P, seed: 1, inside: World(children: [])) ], rules: [ When(n, is: 1, effect: [n.set(0), P.enter]) ])", "demande un geste du visiteur"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn on_compare_et_on_fixe_d_apres_une_autre_valeur() {
        // Le meilleur score : au moment où le score dépasse le meilleur, le meilleur le rattrape.
        let source = "Page(
  state: State(score: 0, best: 2),
  children: [ Button(name: B, text: \"x\"), If(score, over: best, children: [ \"jamais vu : la règle rattrape\" ]), If(score, is: best, children: [ \"record égalé\" ]) ],
  rules: [ On(B.tap, effect: score.add(1)), When(score, over: best, effect: best.set(score)) ],
)";
        let suite: Vec<String> = (0..4).scan(crate::etat_initial(source), |etat, _| { *etat = crate::arbitrer(source, etat, "B.tap"); Some(etat.clone()) }).collect();
        assert_eq!(suite, ["score=1;best=2", "score=2;best=2", "score=3;best=3", "score=4;best=4"]);
        assert_eq!(crate::conditions(source, "score=2;best=2"), "score|over=best:0;score|is=best:1");
        assert_eq!(crate::conditions(source, "score=1;best=2"), "score|over=best:0;score|is=best:0");
        // Ajouter une autre valeur : a.add(b).
        let somme = "Page(state: State(a: 1, b: 5), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: a.add(b)) ])";
        assert_eq!(crate::arbitrer(somme, &crate::etat_initial(somme), "B.tap"), "a=6;b=5");
        for (source, message) in [
            ("Page(state: State(a: 0), children: [ If(a, over: b, children: []) ])", "aucun nombre ne s'appelle « b »"),
            ("Page(state: State(a: 0, t: \"\"), children: [ If(a, over: t, children: []) ])", "aucun nombre ne s'appelle « t »"),
            ("Page(state: State(a: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: a.set(b)) ])", "aucun nombre ne s'appelle « b »"),
            ("Page(state: State(a: 0), rules: [ When(a, over: b, effect: a.set(0)) ])", "aucun nombre ne s'appelle « b »"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn une_valeur_peut_etre_un_texte() {
        let source = "Page(
  state: State(buyer: \"\", city: \"Paris\", cart: 0),
  keep: [buyer],
  children: [
    Input(value: buyer, label: \"Your first name\", max: 12),
    Input(value: city, label: \"Your city\"),
    If(buyer, not: \"\", children: [ \"Hello {buyer}, from {city}.\" ]),
    If(buyer, is: \"\", children: [ \"Who are you?\" ]),
    Button(name: Add, text: \"Add\"),
  ],
  rules: [ On(Add.tap, effect: cart.add(1)) ],
)";
        let html = crate::vue_a_plat(source, "").unwrap();
        assert!(html.contains("<input type=\"text\" maxlength=\"12\" value=\"\" data-bind=\"buyer\">"), "{html}");
        assert!(html.contains("<input type=\"text\" maxlength=\"80\" value=\"Paris\" data-bind=\"city\">"), "{html}");
        // Au départ le prénom est vide : la salutation est cachée, la question montrée.
        assert!(html.contains("data-if=\"buyer|not=0\" hidden><p class=\"holo-P\">Hello <span data-state=\"buyer\"></span>, from <span data-state=\"city\">Paris</span>.</p>"), "{html}");
        assert!(html.contains("data-if=\"buyer|is=0\"><p"), "{html}");
        let depart = crate::etat_initial(source);
        assert_eq!(depart, "cart=0;buyer=';city='Paris");
        // Écrire un prénom : il est nettoyé et coupé à la longueur permise ; rien ne se mêle
        // aux séparateurs de l'état, même un texte hostile.
        let ecrit = crate::saisir(source, &depart, "buyer", "Zoé;cart=99<b>&\u{7} et la suite est trop longue");
        assert_eq!(ecrit, format!("cart=0;buyer='{};city='Paris", coder("Zoé;cart=99<")));
        assert_eq!(crate::conditions(source, &ecrit), "buyer|not=0:1;buyer|is=0:0");
        // Un geste ailleurs ne touche pas aux textes.
        let apres = crate::arbitrer(source, &ecrit, "Add.tap");
        assert!(apres.starts_with("cart=1;buyer='Zo") && apres.ends_with(";city='Paris"), "{apres}");
        // Garder et reprendre : seul le prénom est gardé.
        assert_eq!(crate::a_garder(source, &apres), format!("buyer='{}", coder("Zoé;cart=99<")));
        let repris = crate::reprendre(source, &format!("buyer='{};city='{};cart=5", coder("Ada"), coder("Lyon")));
        assert_eq!(repris, "cart=0;buyer='Ada;city='Paris");
        // Ce que montre la page ne devient jamais du code.
        let hostile = crate::vue_a_plat(&source.replace("city: \"Paris\"", "city: \"<script>x</script>\""), "").unwrap();
        assert!(!hostile.contains("<script>") && hostile.contains("&lt;script&gt;"), "{hostile}");
        assert_eq!(decoder(&coder("é à 🙂 ; = '")).as_deref(), Some("é à 🙂 ; = '"));
        assert_eq!(decoder("%ZZ"), None);
        for (source, message) in [
            ("Page(state: State(a: \"\"), children: [ If(a, over: 2, children: []) ])", "est un texte"),
            ("Page(state: State(a: 0), children: [ If(a, is: \"\", children: []) ])", "est un nombre"),
            ("Page(state: State(a: \"\"), children: [ If(a, is: \"oui\", children: []) ])", "attend un nombre entier"),
            ("Page(state: State(a: \"\"), children: [ Checkbox(value: a, label: \"x\") ])", "une case attend un nombre"),
            ("Page(state: State(a: \"\", a: 0))", "déclarée deux fois"),
            ("Page(state: State(a: \"\"), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: a.add(1)) ])", "« a » est un texte"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn un_champ_une_case_et_des_valeurs_gardees() {
        let source = "Page(
  state: State(tip: 0, gift: 0, visits: 0),
  keep: [tip, gift],
  children: [
    Input(value: tip, label: \"Tip, in euros\", max: 50),
    Checkbox(value: gift, label: \"Gift **wrap**\"),
    Button(name: B, text: \"x\"),
  ],
  rules: [ On(B.tap, effect: visits.add(1)) ],
)";
        let programme = page(source).unwrap();
        let html = crate::vue_a_plat(source, "").unwrap();
        assert!(html.contains("<label class=\"holo-Input\"><span>Tip, in euros</span><input type=\"number\" inputmode=\"numeric\" min=\"0\" max=\"50\" value=\"0\" data-bind=\"tip\"></label>"), "{html}");
        assert!(html.contains("<label class=\"holo-Checkbox\"><input type=\"checkbox\" data-bind=\"gift\"><span>Gift <strong>wrap</strong></span></label>"), "{html}");
        // La saisie passe par l'arbitre : bornée, et sourde à ce qui n'est pas un nombre.
        let depart = initial(&programme).unwrap();
        assert_eq!(ecrire(&saisir(&programme, &depart, "tip", " 12 ")), "tip=12;gift=0;visits=0");
        assert_eq!(ecrire(&saisir(&programme, &depart, "tip", "9999")), "tip=50;gift=0;visits=0");
        assert_eq!(ecrire(&saisir(&programme, &depart, "tip", "douze")), "tip=0;gift=0;visits=0");
        assert_eq!(ecrire(&saisir(&programme, &saisir(&programme, &depart, "tip", "7"), "tip", "")), "tip=0;gift=0;visits=0");
        assert_eq!(ecrire(&saisir(&programme, &depart, "gift", "5")), "tip=0;gift=1;visits=0");
        // Une valeur qu'aucun champ ne présente ne se saisit pas, même si on le demande.
        assert_eq!(ecrire(&saisir(&programme, &depart, "visits", "40")), "tip=0;gift=0;visits=0");
        // Garder : seules les valeurs nommées par « keep » sont écrites, et seules elles sont reprises.
        assert_eq!(crate::a_garder(source, "tip=12;gift=1;visits=9"), "tip=12;gift=1");
        assert_eq!(crate::reprendre(source, "tip=12;gift=1;visits=9;intrus=3"), "tip=12;gift=1;visits=0");
        assert_eq!(crate::reprendre(source, "n'importe quoi"), "tip=0;gift=0;visits=0");
        // Une case déjà cochée et un champ déjà rempli le sont dès le premier affichage.
        let rempli = crate::vue_a_plat(&source.replace("State(tip: 0, gift: 0", "State(tip: 8, gift: 1"), "").unwrap();
        assert!(rempli.contains("value=\"8\" data-bind=\"tip\"") && rempli.contains("<input type=\"checkbox\" data-bind=\"gift\" checked>"), "{rempli}");
        for (source, message) in [
            ("Page(state: State(a: 0), keep: [b])", "aucune valeur ne s'appelle « b »"),
            ("Page(state: State(a: 0), keep: a)", "attend la liste"),
            ("Page(state: State(a: 0), children: [ Input(value: a) ])", "attend « label »"),
            ("Page(state: State(a: 0), children: [ Input(label: \"x\") ])", "attend « value »"),
            ("Page(state: State(a: 0), children: [ Input(value: b, label: \"x\") ])", "aucune valeur ne s'appelle « b »"),
            ("Page(state: State(a: 0), children: [ Input(value: a, label: \"x\", min: 2) ])", "n'a pas de paramètre « min »"),
            ("Page(state: State(a: 0), children: [ Checkbox(value: a, label: \"x\", max: 2) ])", "n'a pas de paramètre « max »"),
            ("Page(state: State(a: 0), children: [ Input(value: a, label: 3) ])", "est mal écrit"),
            ("Page(state: State(a: 0), prices: Prices(a: 1), children: [ Input(value: total, label: \"x\") ])", "aucune valeur ne s'appelle « total »"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn le_hasard_est_rejouable_et_reste_dans_ses_bornes() {
        let source = "Page(name: Dice, state: State(die: 0), children: [ Button(name: Roll, text: \"Roll\") ], rules: [ On(Roll.tap, effect: die.random(5)) ])";
        let lancers = |n: usize| {
            let mut etat = crate::etat_initial(source);
            (0..n).map(|_| { etat = crate::arbitrer(source, &etat, "Roll.tap"); etat.clone() }).collect::<Vec<_>>()
        };
        // Les mêmes gestes redonnent les mêmes nombres (ADR-008).
        assert_eq!(lancers(50), lancers(50));
        // De 0 à 5, bornes comprises, et toutes les faces sortent.
        let faces: Vec<u64> = lancers(200).iter().map(|e| e.split(';').next().unwrap().trim_start_matches("die=").parse().unwrap()).collect();
        assert!(faces.iter().all(|f| *f <= 5));
        for face in 0..=5 {
            assert!(faces.contains(&face), "la face {face} ne sort jamais");
        }
        // Deux fichiers de noms différents n'ont pas la même suite.
        let autre = source.replace("name: Dice", "name: Other");
        let suite = |src: &str| (0..20).fold((crate::etat_initial(src), Vec::new()), |(etat, mut vus), _| { let e = crate::arbitrer(src, &etat, "Roll.tap"); vus.push(e.clone()); (e, vus) }).1;
        assert_ne!(suite(source), suite(&autre));
        for (source, message) in [
            ("Page(state: State(a: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: a.random(0)) ])", "au moins 1"),
            ("Page(state: State(a: 0), rules: [ Every(1, effect: a.add(1)) ])", "une durée en s ou en ms"),
            ("Page(state: State(a: 0), rules: [ Every(10ms, effect: a.add(1)) ])", "de 100ms à 3600s"),
            ("Page(state: State(a: 0), rules: [ Every(2h, effect: a.add(1)) ])", "une durée en s ou en ms"),
            ("Page(state: State(a: 0), rules: [ Every(1s) ])", "attend une demande"),
            ("Page(state: State(a: 0), children: [ Point(name: P, seed: 1, inside: World(children: [])) ], rules: [ Every(1s, effect: P.enter) ])", "demande un geste du visiteur"),
            ("Page(state: State(a: 0), rules: [ Every(1s, effect: b.add(1)) ])", "aucune valeur ne s'appelle « b »"),
            ("Page(children: [ Board(children: [ Point(name: S, seed: 1, x: nowhere, y: 10) ]) ])", "aucune valeur ne s'appelle « nowhere »"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn les_prix_mal_ecrits_sont_refuses() {
        for (source, message) in [
            ("Page(state: State(a: 0), prices: Prices(b: 10))", "ne correspond à aucune valeur"),
            ("Page(prices: Prices(a: 10))", "ne correspond à aucune valeur"),
            ("Page(state: State(a: 0), prices: Prices(a: 10, a: 20))", "donné deux fois"),
            ("Page(state: State(a: 0), prices: Prices(a: \"dix\"))", "un nombre entier"),
            ("Page(state: State(a: 0), prices: 10)", "un bloc « Prices(...) »"),
            ("Page(state: State(a: 0, total: 0), prices: Prices(a: 10))", "« total » est calculé par le moteur"),
            ("Page(state: State(a: 0), children: [ Prices(a: 10) ])", "une seule fois, sur la page"),
            ("Page(state: State(a: 0), children: [ \"{total}\" ])", "aucune valeur ne s'appelle « total »"),
            ("Page(state: State(a: 0), prices: Prices(a: 10), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: total.set(0)) ])", "aucune valeur ne s'appelle « total »"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn ce_qui_est_mal_ecrit_est_refuse_avec_un_message_clair() {
        for (source, message) in [
            ("Page(children: [ \"{cart} items\" ])", "aucune valeur ne s'appelle « cart »"),
            ("Page(state: State(cart: 0), children: [ Button(name: B, text: \"{car}\") ])", "aucune valeur ne s'appelle « car »"),
            ("Page(state: State(cart: 0), children: [ List(children: [ \"{total}\" ]) ])", "aucune valeur ne s'appelle « total »"),
            ("Page(state: State(Cart: 0))", "commence par une minuscule"),
            // L'écriture de Flutter (ADR-037) : l'autre est refusée avec le bon mot.
            ("Page(state: State(apple_x: 0))", "écris « appleX »"),
            ("Page(state: State(appleX: 0), children: [ Text(\"{apple_x}\") ])", "écris « {appleX} »"),
            ("Page(children: [ Button(name: Less_sunrise, text: \"-\") ])", "écris « name: LessSunrise »"),
            ("Page(children: [ Stack(children: [ P(\"a\"), P(\"b\", align: top_right) ]) ])", "écris « topRight »"),
            ("Page(state: State(cart: 1.5))", "un nombre entier, un texte ou une liste"),
            ("Page(state: State(cart: 0, cart: 1))", "déclarée deux fois"),
            ("Page(state: State(cart: 5000000000))", "de 0 à 1000000000"),
            ("Page(state: 4)", "un bloc « State(...) »"),
            ("Page(children: [ State(cart: 0) ])", "une seule fois, sur la page"),
            ("Page(children: [ Point(name: A, seed: 1, inside: World(state: State(cart: 0))) ])", "sur la page"),
            ("Page(state: State(cart: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: cart.double(1)) ])", "demande inconnue « double »"),
            ("Page(state: State(cart: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: total.add(1)) ])", "aucune valeur ne s'appelle « total »"),
            ("Page(state: State(cart: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: cart.add()) ])", "attend un seul nombre"),
            ("Page(state: State(cart: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: cart.add(1, 2)) ])", "attend un seul nombre"),
            ("Page(state: State(cart: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: cart.add(\"1\")) ])", "attend un nombre entier"),
            ("Page(state: State(cart: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: cart.add) ])", "s'écrit avec sa quantité"),
            ("Page(state: State(cart: 0), children: [ cart.add(1) ])", "dans l'effet d'une règle"),
            ("Page(state: State(cart: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: cart(1)) ])", "commence par une majuscule"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }
}
