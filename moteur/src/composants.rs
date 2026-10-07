//! Les composants (ADR-050) : un bloc écrit une fois, avec des paramètres, posé comme un bloc
//! ordinaire, à la manière d'un widget Flutter, et restylé par le CSS, à la manière du web.
//!
//! ```holo
//! Page(
//!   state: State(cart: 0),
//!   components: [
//!     Component(
//!       name: ArticleCard,
//!       params: [title, price, image],
//!       children: [ Column.card(children: [ Image(source: image, alt: title), H3("{title}"), Text("{price} euros"), Button(name: Add, text: "Add") ]) ],
//!       rules: [ On(Add.tap, effect: cart.add(price)) ],
//!     ),
//!   ],
//!   children: [
//!     ArticleCard(name: Sunrise, title: "Sunrise", price: 120, image: "sunrise.png"),
//!     ArticleCard.promo(name: Night, title: "Night", price: 60, image: "night.png"),
//!   ],
//! )
//!
//! ArticleCard { --accent: #E9B44C; }
//! .promo { --accent: crimson; }
//! ```
//!
//! Comme `Use` et `Repeat`, un composant est déplié à la lecture : le reste du moteur ne voit que
//! les blocs qu'on aurait écrits à la main.
//! - Un paramètre s'emploie par son nom : `title` à la place d'une valeur, `{title}` dans un texte.
//!   Donné par le nom d'une valeur de la page (`count: sunrise`), il la suit : `{count}` montre
//!   la valeur, `count.add(1)` la change.
//! - Un bloc nommé dans le composant reçoit le nom de la copie : `Add` devient `AddSunrise`.
//! - Les règles du composant rejoignent celles de la page (ou du monde), une fois par copie.
//! - Le bloc racine de la copie porte la marque du composant (`ArticleCard { … }` le vise) et
//!   les noms de style écrits à l'appel (`ArticleCard.promo` : `.promo { … }` le vise).

use crate::holo::{Argument, Bloc, Erreur, Pos, Valeur};

/// `Part` s'appelle `Component` depuis ADR-056 : le mot `Part` est gardé pour la 3D (une pièce
/// d'un objet, comme dans Roblox). L'ancienne écriture est refusée avec le bon mot.
pub const ANCIEN_PART: &str = "« Part » s'appelle maintenant « Component » (ADR-056) : écris « Component » ; le mot « Part » est gardé pour la 3D";

/// Le nombre de paramètres d'un composant, au plus.
pub const PARAMETRES_MAX: usize = 16;
/// La profondeur des composants posés les uns dans les autres, au plus.
pub const PROFONDEUR_MAX: usize = 8;
/// Le nombre de copies de composants qu'une page peut poser, au plus.
pub const COPIES_MAX: usize = 2_000;

/// Les mots qu'un paramètre ne peut pas porter : ils ont déjà un sens dans le langage.
const MOTS_RESERVES: &[&str] = &[
    "name", "children", "rules", "params", "item", "key", "add", "sub", "set", "mul", "div", "push", "remove", "clear", "random", "enter", "leave",
    "play", "portals", "tap", "hover", "hoverEnd", "count", "total", "year", "month", "day", "weekday", "hour", "minute", "true", "false",
];

/// Un composant lu : son nom, ses paramètres, son contenu, ses règles.
#[derive(Debug, Clone)]
pub struct Composant {
    pub nom: String,
    pub parametres: Vec<String>,
    /// Les valeurs par défaut : `params: [title, price: 0]` (ADR-056).
    pub defauts: Vec<(String, Valeur)>,
    /// Les signaux que le composant émet : `emits: [add]` ; la page les branche à l'appel,
    /// `onAdd: cart.add(1)` (ADR-056).
    pub emis: Vec<String>,
    pub enfants: Vec<Valeur>,
    pub regles: Vec<Valeur>,
    pub pos: Pos,
}

/// Lit un `Component(name:, params:, children:, rules:)`.
pub fn lire_part(part: &Bloc) -> Result<Composant, Erreur> {
    let exemple = "Component(name: ArticleCard, params: [title, price], children: [ Column(children: [ H3(\"{title}\"), Text(\"{price} euros\") ]) ])";
    let refus = |message: String, pos: Pos| Err(Erreur { message, pos });
    let (mut nom, mut parametres, mut enfants, mut regles) = (None, Vec::new(), None, Vec::new());
    let mut defauts: Vec<(String, Valeur)> = Vec::new();
    let mut emis: Vec<String> = Vec::new();
    for argument in &part.arguments {
        match (argument.nom.as_deref(), &argument.valeur) {
            (Some("name"), Valeur::Nom(n)) => nom = Some(n.clone()),
            (Some("name"), _) => return refus(format!("le nom d'un composant s'écrit sans guillemets, avec une majuscule : {exemple}"), argument.pos),
            (Some("params"), Valeur::Liste(liste)) => {
                for valeur in liste {
                    match valeur {
                        Valeur::Nom(p) => parametres.push(p.clone()),
                        // `price: 0` : un paramètre facultatif, et sa valeur par défaut.
                        Valeur::Bloc(b) if b.nom == crate::holo::VALEUR_NOMMEE => {
                            let Some(Argument { nom: Some(p), valeur: defaut, .. }) = b.arguments.first() else { continue };
                            if !matches!(defaut, Valeur::Texte(_) | Valeur::Entier(_) | Valeur::Nombre { .. } | Valeur::Nom(_) | Valeur::Bool(_)) {
                                return refus(format!("la valeur par défaut de « {p} » est un texte, un nombre ou un nom, pas un bloc ni une liste"), b.pos);
                            }
                            parametres.push(p.clone());
                            defauts.push((p.clone(), defaut.clone()));
                        }
                        _ => return refus("« params » est une liste de noms, avec ou sans valeur par défaut : params: [title, price: 0]".into(), argument.pos),
                    }
                }
            }
            (Some("emits"), Valeur::Liste(liste)) => {
                for valeur in liste {
                    match valeur {
                        Valeur::Nom(e) if e.starts_with(|c: char| c.is_ascii_lowercase()) && e.chars().all(|c| c.is_ascii_alphanumeric()) && !emis.contains(e) => emis.push(e.clone()),
                        _ => return refus("« emits » est une liste de signaux, en minuscules, chacun une fois : emits: [add, remove]".into(), argument.pos),
                    }
                }
            }
            (Some("children"), Valeur::Liste(liste)) => enfants = Some(liste.clone()),
            (Some("rules"), Valeur::Liste(liste)) => regles = liste.clone(),
            (Some(mot @ ("params" | "emits" | "children" | "rules")), _) => return refus(format!("« Component({mot}: …) » attend une liste entre crochets : {exemple}"), argument.pos),
            (Some(autre), _) => return refus(format!("« Component » n'a pas de réglage « {autre} » ; réglages possibles : name, params, emits, children, rules"), argument.pos),
            (None, _) => return refus(format!("chaque réglage de « Component » est nommé : {exemple}"), argument.pos),
        }
    }
    let (Some(nom), Some(enfants)) = (nom, enfants) else {
        return refus(format!("un morceau a un nom et un contenu : {exemple}"), part.pos);
    };
    if !nom.starts_with(|c: char| c.is_ascii_uppercase()) || !nom.chars().all(|c| c.is_ascii_alphanumeric()) {
        return refus(format!("« {nom} » : le nom d'un composant s'écrit comme un bloc, avec une majuscule et sans « _ », comme « ArticleCard » (ADR-037)"), part.pos);
    }
    if crate::blocs::BLOCS.contains(&nom.as_str()) {
        return refus(format!("« {nom} » est déjà un bloc du langage ; choisis un autre nom de composant"), part.pos);
    }
    if parametres.len() > PARAMETRES_MAX {
        return refus(format!("« {nom} » a trop de paramètres : {PARAMETRES_MAX} au plus"), part.pos);
    }
    for (i, p) in parametres.iter().enumerate() {
        if !p.starts_with(|c: char| c.is_ascii_lowercase()) || !p.chars().all(|c| c.is_ascii_alphanumeric()) {
            return refus(format!("« {p} » : un paramètre s'écrit comme une valeur, en minuscules, les mots joints par une majuscule, comme « title » ou « oldPrice » (ADR-037)"), part.pos);
        }
        if MOTS_RESERVES.contains(&p.as_str()) {
            return refus(format!("« {p} » est un mot du langage ; choisis un autre nom de paramètre"), part.pos);
        }
        if parametres[..i].contains(p) {
            return refus(format!("le paramètre « {p} » est écrit deux fois dans « {nom} »"), part.pos);
        }
    }
    for regle in &regles {
        let Valeur::Bloc(b) = regle else { return refus("« Component(rules: …) » range des règles : rules: [ On(Add.tap, effect: cart.add(1)) ]".into(), part.pos) };
        if !matches!(b.nom.as_str(), "On" | "Every" | "When" | "After" | "If") {
            return refus("« Component(rules: …) » range des règles : rules: [ On(Add.tap, effect: cart.add(1)) ]".into(), part.pos);
        }
        for signal in signaux_emis(b) {
            if !emis.contains(&signal) {
                return refus(format!("« emit: {signal} » : déclare ce signal dans le composant, emits: [{signal}]"), b.pos);
            }
        }
    }
    Ok(Composant { nom, parametres, defauts, emis, enfants, regles, pos: part.pos })
}

/// Les signaux qu'une règle émet : `emit: add`, ou `emit: [add, remove]`.
fn signaux_emis(regle: &Bloc) -> Vec<String> {
    match regle.argument("emit").map(|a| &a.valeur) {
        Some(Valeur::Nom(n)) => vec![n.clone()],
        Some(Valeur::Liste(l)) => l.iter().filter_map(|v| if let Valeur::Nom(n) = v { Some(n.clone()) } else { None }).collect(),
        _ => Vec::new(),
    }
}

/// `add` → `onAdd` : le nom du branchement d'un signal, à l'appel.
fn branchement(signal: &str) -> String {
    let mut lettres = signal.chars();
    format!("on{}", lettres.next().map(|c| c.to_ascii_uppercase().to_string() + lettres.as_str()).unwrap_or_default())
}

/// Remplace `emit: add` dans une règle de la copie par ce que la page a branché (`onAdd:`).
/// Une règle qui n'émet que des signaux non branchés, et ne fait rien d'autre, disparaît.
fn brancher(mut regle: Valeur, branches: &[(String, Valeur)]) -> Option<Valeur> {
    let Valeur::Bloc(bloc) = &mut regle else { return Some(regle) };
    let signaux = signaux_emis(bloc);
    if signaux.is_empty() {
        return Some(regle);
    }
    bloc.arguments.retain(|a| a.nom.as_deref() != Some("emit"));
    let mut effets: Vec<Valeur> = match bloc.argument("effect").map(|a| a.valeur.clone()) {
        Some(Valeur::Liste(l)) => l,
        Some(v) => vec![v],
        None => Vec::new(),
    };
    for signal in &signaux {
        if let Some((_, valeur)) = branches.iter().find(|(s, _)| s == signal) {
            match valeur {
                Valeur::Liste(l) => effets.extend(l.iter().cloned()),
                v => effets.push(v.clone()),
            }
        }
    }
    if effets.is_empty() {
        return None;
    }
    bloc.arguments.retain(|a| a.nom.as_deref() != Some("effect"));
    let pos = bloc.pos;
    bloc.arguments.push(Argument { nom: Some("effect".into()), valeur: if effets.len() == 1 { effets.remove(0) } else { Valeur::Liste(effets) }, pos });
    Some(regle)
}

/// Retire les composants écrits dans la page (`components: [ Component(…) ]`) et les rend.
pub fn retirer_les_parts(page: &mut Bloc) -> Result<Vec<Composant>, Erreur> {
    if let Some(ancien) = page.arguments.iter().find(|a| a.nom.as_deref() == Some("parts")) {
        return Err(Erreur { message: "« parts » s'appelle maintenant « components » (ADR-056) : écris « components »".into(), pos: ancien.pos });
    }
    let Some(i) = page.arguments.iter().position(|a| a.nom.as_deref() == Some("components")) else { return Ok(Vec::new()) };
    let argument = page.arguments.remove(i);
    if page.nom != "Page" {
        return Err(Erreur { message: "« components » s'écrit dans la page : Page(components: [ Component(…) ])".into(), pos: argument.pos });
    }
    let Valeur::Liste(liste) = argument.valeur else {
        return Err(Erreur { message: "« components » est une liste de composants : components: [ Component(name: ArticleCard, …) ]".into(), pos: argument.pos });
    };
    let mut composants = Vec::new();
    for valeur in &liste {
        match valeur {
            Valeur::Bloc(b) if b.nom == "Component" => composants.push(lire_part(b)?),
            Valeur::Bloc(b) if b.nom == "Part" => return Err(Erreur { message: ANCIEN_PART.into(), pos: b.pos }),
            _ => return Err(Erreur { message: "« components » ne contient que des « Component(…) »".into(), pos: argument.pos }),
        }
    }
    Ok(composants)
}

/// Les noms des valeurs de la page (`State(cart: 0)`) : un paramètre ne peut pas en porter un.
fn valeurs_de_la_page(page: &Bloc) -> Vec<String> {
    match page.argument("state").map(|a| &a.valeur) {
        Some(Valeur::Bloc(etat)) => etat.arguments.iter().filter_map(|a| a.nom.clone()).collect(),
        _ => Vec::new(),
    }
}

/// Pose les composants d'un site (une page, ou un monde) et de tout ce qu'il contient.
pub fn poser_site(site: &mut Bloc, composants: &[Composant]) -> Result<(), Erreur> {
    if composants.is_empty() {
        return Ok(());
    }
    for composant in composants {
        for p in &composant.parametres {
            if valeurs_de_la_page(site).contains(p) {
                return Err(Erreur { message: format!("le paramètre « {p} » de « {} » porte le nom d'une valeur de la page ; choisis un autre nom", composant.nom), pos: composant.pos });
            }
        }
    }
    let mut regles = Vec::new();
    let mut copies = 0;
    let mut sans_nom = Vec::new();
    for argument in &mut site.arguments {
        poser_valeur(&mut argument.valeur, composants, &mut regles, &mut Vec::new(), &mut copies, &mut sans_nom)?;
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

fn poser_valeur(valeur: &mut Valeur, composants: &[Composant], regles: &mut Vec<Valeur>, chemin: &mut Vec<String>, copies: &mut usize, sans_nom: &mut Vec<String>) -> Result<(), Erreur> {
    match valeur {
        // Le monde d'un point est un autre site : ses règles restent chez lui.
        Valeur::Bloc(bloc) if bloc.nom == "World" && chemin.is_empty() => poser_site(bloc, composants),
        Valeur::Bloc(bloc) if composants.iter().any(|c| c.nom == bloc.nom) => {
            Err(Erreur { message: format!("« {} » se pose parmi des blocs : children: [ {}(…) ]", bloc.nom, bloc.nom), pos: bloc.pos })
        }
        // Dans le modèle d'une répétition, les règles d'un composant rejoignent celles de la
        // répétition : elles y sont écrites une fois par élément, avec « item ».
        Valeur::Bloc(bloc) if bloc.nom == "Repeat" => {
            let mut locales = Vec::new();
            for argument in &mut bloc.arguments {
                poser_valeur(&mut argument.valeur, composants, &mut locales, chemin, copies, sans_nom)?;
            }
            if !locales.is_empty() {
                match bloc.arguments.iter_mut().find(|a| a.nom.as_deref() == Some("rules")) {
                    Some(Argument { valeur: Valeur::Liste(liste), .. }) => liste.extend(locales),
                    Some(argument) => return Err(Erreur { message: "« Repeat(rules: …) » attend une liste entre crochets".into(), pos: argument.pos }),
                    None => bloc.arguments.push(Argument { nom: Some("rules".into()), valeur: Valeur::Liste(locales), pos: bloc.pos }),
                }
            }
            Ok(())
        }
        Valeur::Bloc(bloc) => bloc.arguments.iter_mut().try_for_each(|a| poser_valeur(&mut a.valeur, composants, regles, chemin, copies, sans_nom)),
        Valeur::Liste(elements) => {
            let mut poses = Vec::with_capacity(elements.len());
            for element in std::mem::take(elements) {
                match element {
                    Valeur::Bloc(appel) if composants.iter().any(|c| c.nom == appel.nom) => {
                        let composant = composants.iter().find(|c| c.nom == appel.nom).expect("trouvé juste au-dessus");
                        if chemin.contains(&composant.nom) {
                            return Err(Erreur { message: format!("« {} » se pose lui-même, directement ou par un autre composant : il ne finirait jamais", composant.nom), pos: appel.pos });
                        }
                        if chemin.len() >= PROFONDEUR_MAX {
                            return Err(Erreur { message: format!("trop de composants les uns dans les autres : {PROFONDEUR_MAX} au plus"), pos: appel.pos });
                        }
                        *copies += 1;
                        if *copies > COPIES_MAX {
                            return Err(Erreur { message: format!("la page pose plus de {COPIES_MAX} composants : c'est trop pour une page"), pos: appel.pos });
                        }
                        let (racine, regles_de_la_copie) = poser_copie(composant, &appel, sans_nom)?;
                        chemin.push(composant.nom.clone());
                        // La racine peut être elle-même un composant : on la pose comme un élément de liste.
                        let mut seule = Valeur::Liste(vec![racine]);
                        poser_valeur(&mut seule, composants, regles, chemin, copies, sans_nom)?;
                        for mut regle in regles_de_la_copie {
                            poser_valeur(&mut regle, composants, regles, chemin, copies, sans_nom)?;
                            regles.push(regle);
                        }
                        chemin.pop();
                        if let Valeur::Liste(posees) = seule {
                            poses.extend(posees);
                        }
                    }
                    mut autre => {
                        poser_valeur(&mut autre, composants, regles, chemin, copies, sans_nom)?;
                        poses.push(autre);
                    }
                }
            }
            *elements = poses;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Écrit une copie d'un composant, d'après son appel : `ArticleCard.promo(name: Night, title: "Night", …)`.
fn poser_copie(composant: &Composant, appel: &Bloc, sans_nom: &mut Vec<String>) -> Result<(Valeur, Vec<Valeur>), Erreur> {
    let nom = &composant.nom;
    let attendus = composant.parametres.iter().map(|p| format!("{p}: …")).collect::<Vec<_>>().join(", ");
    let exemple = if attendus.is_empty() { format!("{nom}()") } else { format!("{nom}({attendus})") };
    let mut nom_de_copie = None;
    let mut donnes: Vec<(&str, &Valeur)> = Vec::new();
    let mut branches: Vec<(String, Valeur)> = Vec::new();
    for argument in &appel.arguments {
        let Some(cle) = argument.nom.as_deref() else {
            return Err(Erreur { message: format!("chaque paramètre de « {nom} » est nommé : {exemple}"), pos: argument.pos });
        };
        match (cle, &argument.valeur) {
            ("name", Valeur::Nom(n)) if n.starts_with(|c: char| c.is_ascii_uppercase()) && n.chars().all(|c| c.is_ascii_alphanumeric()) => nom_de_copie = Some(n.clone()),
            ("name", _) => return Err(Erreur { message: format!("le nom d'une copie s'écrit comme un bloc, avec une majuscule : {nom}(name: Sunrise, …)"), pos: argument.pos }),
            // `onAdd: cart.add(1)` : la page branche un signal émis par le composant.
            (cle, v) if composant.emis.iter().any(|e| branchement(e) == cle) => {
                let bon = match v {
                    Valeur::Bloc(_) | Valeur::Nom(_) => true,
                    Valeur::Liste(l) => !l.is_empty() && l.iter().all(|e| matches!(e, Valeur::Bloc(_) | Valeur::Nom(_))),
                    _ => false,
                };
                if !bon {
                    return Err(Erreur { message: format!("« {cle} » attend une demande ou une liste de demandes : {cle}: cart.add(1)"), pos: argument.pos });
                }
                let signal = composant.emis.iter().find(|e| branchement(e) == cle).cloned().unwrap_or_default();
                branches.push((signal, v.clone()));
            }
            (p, v) if composant.parametres.iter().any(|connu| connu == p) => {
                if donnes.iter().any(|(connu, _)| *connu == p) {
                    return Err(Erreur { message: format!("le paramètre « {p} » est donné deux fois"), pos: argument.pos });
                }
                if !matches!(v, Valeur::Texte(_) | Valeur::Entier(_) | Valeur::Nombre { .. } | Valeur::Nom(_) | Valeur::Bool(_)) {
                    return Err(Erreur { message: format!("le paramètre « {p} » attend un texte, un nombre ou un nom, pas un bloc ni une liste"), pos: argument.pos });
                }
                donnes.push((p, v));
            }
            (autre, _) => {
                let mut connus = composant.parametres.clone();
                connus.extend(composant.emis.iter().map(|e| branchement(e)));
                let suggestion = proche(autre, &connus);
                let message = match suggestion {
                    Some(bon) => format!("« {nom} » n'a pas de paramètre « {autre} » : écris « {bon} »"),
                    None if composant.parametres.is_empty() => format!("« {nom} » n'a pas de paramètres : {nom}()"),
                    None => format!("« {nom} » n'a pas de paramètre « {autre} » ; paramètres : {}", composant.parametres.join(", ")),
                };
                return Err(Erreur { message, pos: argument.pos });
            }
        }
    }
    // Un paramètre oublié prend sa valeur par défaut, s'il en a une.
    for (p, defaut) in &composant.defauts {
        if !donnes.iter().any(|(d, _)| d == p) {
            donnes.push((p.as_str(), defaut));
        }
    }
    if let Some(manque) = composant.parametres.iter().find(|p| !donnes.iter().any(|(d, _)| d == p)) {
        return Err(Erreur { message: format!("« {nom} » attend le paramètre « {manque} » : {exemple}"), pos: appel.pos });
    }
    // Le contenu : un seul bloc racine, comme le widget que rend Flutter.
    let [Valeur::Bloc(_)] = composant.enfants.as_slice() else {
        return Err(Erreur { message: format!("« {nom} » se pose comme un bloc : son contenu a un seul bloc racine ; range ses blocs dans Column(children: [ … ])"), pos: composant.pos });
    };
    let mut noms = Vec::new();
    for valeur in composant.enfants.iter().chain(composant.regles.iter()) {
        noms_donnes(valeur, &mut noms);
    }
    if !noms.is_empty() && nom_de_copie.is_none() {
        // Sans nom, une copie garde les noms de ses blocs : une seule copie peut le faire.
        if sans_nom.contains(nom) {
            return Err(Erreur { message: format!("« {nom} » nomme ses blocs ({}) : donne un nom à chaque copie, {nom}(name: Sunrise, …)", noms.join(", ")), pos: appel.pos });
        }
        sans_nom.push(nom.clone());
    }
    let copie = Copie { donnes: &donnes, noms: &noms, suffixe: nom_de_copie.as_deref() };
    let mut racine = composant.enfants[0].clone();
    remplacer(&mut racine, &copie)?;
    if let Valeur::Bloc(bloc) = &mut racine {
        // La marque du composant d'abord, puis les noms de style écrits à l'appel.
        let mut styles = vec![nom.clone()];
        styles.extend(appel.styles.iter().cloned());
        styles.extend(std::mem::take(&mut bloc.styles));
        bloc.styles = styles;
    }
    let mut regles = Vec::new();
    for regle in &composant.regles {
        let mut r = regle.clone();
        remplacer(&mut r, &copie)?;
        if let Some(r) = brancher(r, &branches) {
            regles.push(r);
        }
    }
    Ok((racine, regles))
}

/// Le paramètre le plus proche d'un nom mal écrit : une ou deux lettres de différence.
fn proche<'a>(nom: &str, connus: &'a [String]) -> Option<&'a str> {
    fn distance(a: &str, b: &str) -> usize {
        let (a, b): (Vec<char>, Vec<char>) = (a.to_lowercase().chars().collect(), b.to_lowercase().chars().collect());
        let mut ligne: Vec<usize> = (0..=b.len()).collect();
        for i in 1..=a.len() {
            let mut precedent = ligne[0];
            ligne[0] = i;
            for j in 1..=b.len() {
                let ancien = ligne[j];
                ligne[j] = (ligne[j] + 1).min(ligne[j - 1] + 1).min(precedent + usize::from(a[i - 1] != b[j - 1]));
                precedent = ancien;
            }
        }
        ligne[b.len()]
    }
    connus.iter().map(|c| (distance(nom, c), c)).filter(|(d, _)| *d <= 2).min_by_key(|(d, _)| *d).map(|(_, c)| c.as_str())
}

struct Copie<'a> {
    donnes: &'a [(&'a str, &'a Valeur)],
    noms: &'a [String],
    suffixe: Option<&'a str>,
}

impl Copie<'_> {
    fn parametre(&self, nom: &str) -> Option<&Valeur> {
        self.donnes.iter().find(|(p, _)| *p == nom).map(|(_, v)| *v)
    }
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

/// Remplace, dans la copie, les paramètres par ce qui a été donné, et renomme les blocs nommés.
fn remplacer(valeur: &mut Valeur, copie: &Copie) -> Result<(), Erreur> {
    match valeur {
        Valeur::Texte(texte) => *texte = remplacer_dans_le_texte(texte, copie),
        Valeur::Nom(nom) => {
            let (debut, suite) = nom.split_once('.').map_or((nom.as_str(), None), |(a, b)| (a, Some(b)));
            if let Some(donne) = copie.parametre(debut) {
                match (donne, suite) {
                    (_, None) => *valeur = donne.clone(),
                    // `count.enter` : une capacité demandée à la valeur donnée.
                    (Valeur::Nom(n), Some(s)) => *nom = format!("{n}.{s}"),
                    _ => {}
                }
            } else if copie.noms.iter().any(|n| n == debut) {
                if let Some(suffixe) = copie.suffixe {
                    let nouveau = format!("{debut}{suffixe}");
                    *nom = match suite {
                        Some(s) => format!("{nouveau}.{s}"),
                        None => nouveau,
                    };
                }
            }
        }
        Valeur::Liste(elements) => {
            for e in elements {
                remplacer(e, copie)?;
            }
        }
        Valeur::Bloc(bloc) => {
            // Une demande faite à une valeur donnée en paramètre : `count.add(1)`.
            if let Some((debut, verbe)) = bloc.nom.split_once('.') {
                if debut.starts_with(|c: char| c.is_ascii_lowercase()) {
                    if let Some(donne) = copie.parametre(debut) {
                        match donne {
                            Valeur::Nom(n) => bloc.nom = format!("{n}.{verbe}"),
                            _ => return Err(Erreur { message: format!("« {debut}.{verbe}(…) » : le paramètre « {debut} » doit recevoir le nom d'une valeur de la page pour qu'on puisse la changer, comme {debut}: cart"), pos: bloc.pos }),
                        }
                    }
                }
            }
            for argument in &mut bloc.arguments {
                remplacer(&mut argument.valeur, copie)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// `{title}` devient ce qui a été donné ; donné par un nom (`count: sunrise`, `title: item.title`),
/// il reste à montrer : `{sunrise}`, `{item.title}`.
fn remplacer_dans_le_texte(texte: &str, copie: &Copie) -> String {
    if !texte.contains('{') || copie.donnes.is_empty() {
        return texte.to_string();
    }
    let mut sortie = String::with_capacity(texte.len());
    let mut reste = texte;
    while let Some(debut) = reste.find('{') {
        sortie.push_str(&reste[..debut]);
        let apres = &reste[debut + 1..];
        let Some(fin) = apres.find('}') else {
            sortie.push_str(&reste[debut..]);
            reste = "";
            break;
        };
        let dedans = &apres[..fin];
        let (nom, format) = dedans.split_once(':').map_or((dedans, None), |(n, f)| (n, Some(f)));
        match (copie.parametre(nom), format) {
            (Some(Valeur::Nom(n)), Some(f)) => sortie.push_str(&format!("{{{n}:{f}}}")),
            (Some(Valeur::Nom(n)), None) => sortie.push_str(&format!("{{{n}}}")),
            (Some(Valeur::Entier(n)), Some(f)) if crate::format::est_format(f) && f != "name" => sortie.push_str(&crate::format::formater(nom, *n, f, &crate::format::langue())),
            (Some(Valeur::Texte(t)), None) => sortie.push_str(t),
            (Some(Valeur::Entier(n)), None) => sortie.push_str(&n.to_string()),
            (Some(Valeur::Nombre { valeur, unite }), None) => sortie.push_str(&format!("{valeur}{}", unite.as_deref().unwrap_or(""))),
            (Some(Valeur::Bool(b)), None) => sortie.push_str(if *b { "true" } else { "false" }),
            _ => sortie.push_str(&reste[debut..debut + 1 + fin + 1]),
        }
        reste = &apres[fin + 1..];
    }
    sortie.push_str(reste);
    sortie
}

#[cfg(test)]
mod tests {
    use crate::holo::lire;

    const BOUTIQUE: &str = r#"Page(
  state: State(cart: 0, sunrise: 0),
  components: [
    Component(
      name: ArticleCard,
      params: [title, price, qty],
      children: [ Column.card(children: [ H2("{title}"), Text("{price:cents} euros, {qty} in the cart"), Button(name: Add, text: "Add") ]) ],
      rules: [ On(Add.tap, effect: [cart.add(price), qty.add(1)]) ],
    ),
  ],
  children: [
    H1("Shop"),
    ArticleCard(name: Sunrise, title: "Sunrise", price: 12000, qty: sunrise),
    ArticleCard.promo(name: Night, title: "Night", price: 6000, qty: cart),
  ],
)
ArticleCard { --accent: #E9B44C; border: 1px solid --accent; }
.card { padding: 8px; }
.promo { --accent: crimson; }
"#;

    #[test]
    fn un_composant_se_pose_comme_un_bloc_et_se_restyle_par_le_css() {
        let programme = lire(BOUTIQUE).unwrap();
        let html = crate::plat::page_html(&programme, "").unwrap();
        assert!(html.contains("<div class=\"holo-Column holo-c-ArticleCard holo-s-card\""), "{html}");
        assert!(html.contains("<div class=\"holo-Column holo-c-ArticleCard holo-s-promo holo-s-card\""), "{html}");
        assert!(html.contains("<h2 class=\"holo-H2\">Sunrise</h2>"), "{html}");
        assert!(html.contains("data-name=\"AddSunrise\"") && html.contains("data-name=\"AddNight\""), "{html}");
        assert!(html.contains(".holo-c-ArticleCard{--accent:#E9B44C;border:1px solid var(--accent);}"), "{html}");
        assert!(html.contains(".holo-s-promo{--accent:crimson;}"), "{html}");
        crate::verifier_page(BOUTIQUE).unwrap();
        // Les règles du composant, une fois par copie, avec les paramètres remplacés.
        assert_eq!(crate::arbitrer(BOUTIQUE, "cart=0;sunrise=0", "AddSunrise.tap"), "cart=12000;sunrise=1");
        assert_eq!(crate::arbitrer(BOUTIQUE, "cart=0;sunrise=0", "AddNight.tap"), "cart=6001;sunrise=0");
    }

    #[test]
    fn un_composant_dans_une_repetition_et_dans_un_fichier_importe() {
        let commun = "Component(name: Card, params: [title], children: [ Column(children: [ H2(\"{title}\"), Button(name: Add, text: \"Add\") ]) ], rules: [ On(Add.tap, effect: item.add(1)) ])\nCard { color: navy; }";
        let page = "import \"commun.holo\"\nPage(state: State(a: 0, b: 0), children: [ H1(\"x\"), Repeat(items: [ Item(key: a, title: \"A\"), Item(key: b, title: \"B\") ], children: [ Card(title: item.title) ]) ])";
        let source = format!("{page}{}commun.holo{}{commun}", crate::holo::FICHIER_SUIVANT, crate::holo::SEPARE_LE_NOM);
        let programme = lire(&source).unwrap();
        let html = crate::plat::page_html(&programme, "").unwrap();
        assert!(html.contains("<h2 class=\"holo-H2\">A</h2>") && html.contains("data-name=\"AddB\""), "{html}");
        crate::verifier_page(&source).unwrap();
    }

    #[test]
    fn l_exemple_du_guide_dans_une_repetition() {
        let source = r#"Page(
  state: State(cart: 0, a: 0, b: 0),
  components: [ Component(name: ArticleCard, params: [title, price, qty], children: [ Column(children: [ H2("{title}"), Text("{price:cents} euros, {qty}"), Button(name: Add, text: "Add") ]) ], rules: [ On(Add.tap, effect: [qty.add(1), cart.add(price)]) ]) ],
  children: [ H1("Shop"), Repeat(items: [ Item(key: a, title: "A", price: 1250), Item(key: b, title: "B", price: 300) ], children: [ ArticleCard(title: item.title, price: item.price, qty: item) ]) ],
)"#;
        crate::verifier_page(source).unwrap();
        assert_eq!(crate::arbitrer(source, "cart=0;a=0;b=0", "AddB.tap"), "cart=300;a=0;b=1");
        let html = crate::plat::page_html(&lire(source).unwrap(), "").unwrap();
        assert!(html.contains("<h2 class=\"holo-H2\">A</h2>") && html.contains("12,50 euros"), "{html}");
    }

    #[test]
    fn valeurs_par_defaut_et_signaux_emis() {
        let source = r#"Page(
  state: State(cart: 0, likes: 0),
  components: [
    Component(
      name: ArticleCard,
      params: [title, price: 0, image: "placeholder.svg"],
      emits: [add, like],
      children: [ Column(children: [ Image(source: image, alt: title), H2("{title}"), Text("{price} euros"), Button(name: Add, text: "Add"), Button(name: Like, text: "♥") ]) ],
      rules: [ On(Add.tap, emit: add), On(Like.tap, emit: like) ],
    ),
  ],
  children: [
    H1("Shop"),
    ArticleCard(name: Sunrise, title: "Sunrise", price: 120, image: "sunrise.png", onAdd: cart.add(120), onLike: likes.add(1)),
    ArticleCard(name: Gift, title: "Gift card", onAdd: [cart.add(10), likes.add(1)]),
  ],
)"#;
        crate::verifier_page(source).unwrap();
        let html = crate::plat::page_html(&lire(source).unwrap(), "").unwrap();
        // Les valeurs par défaut : l'image et le prix oubliés par la seconde copie.
        assert!(html.contains("src=\"placeholder.svg\" alt=\"Gift card\"") && html.contains("0 euros"), "{html}");
        // Les signaux émis, branchés par la page.
        assert_eq!(crate::arbitrer(source, "cart=0;likes=0", "AddSunrise.tap"), "cart=120;likes=0");
        assert_eq!(crate::arbitrer(source, "cart=0;likes=0", "LikeSunrise.tap"), "cart=0;likes=1");
        assert_eq!(crate::arbitrer(source, "cart=0;likes=0", "AddGift.tap"), "cart=10;likes=1");
        // Un signal que la page ne branche pas ne fait rien.
        assert_eq!(crate::arbitrer(source, "cart=0;likes=0", "LikeGift.tap"), "cart=0;likes=0");
        for (source, message) in [
            ("Page(components: [ Component(name: Card, children: [ Button(name: Go, text: \"go\") ], rules: [ On(Go.tap, emit: go) ]) ], children: [ Card() ])", "déclare ce signal"),
            ("Page(components: [ Component(name: Card, emits: [go], children: [ Button(name: Go, text: \"go\") ], rules: [ On(Go.tap, emit: go) ]) ], children: [ Card(onGo: 3) ])", "attend une demande"),
            ("Page(components: [ Component(name: Card, emits: [go], children: [ P(\"x\") ]) ], children: [ Card(onGoo: x.add(1)) ])", "écris « onGo »"),
            ("Page(state: State(l: [ x: 1 ]), children: [])", "seuls les paramètres d'un composant"),
            ("Page(components: [ Part(name: Card, children: [ P(\"x\") ]) ], children: [])", "écris « Component »"),
            ("Page(parts: [ Component(name: Card, children: [ P(\"x\") ]) ], children: [])", "écris « components »"),
        ] {
            let erreur = crate::verifier_page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn ce_qui_est_refuse() {
        let part = |reste: &str| format!("Page(components: [ Component(name: Card, params: [title], children: [ H3(\"{{title}}\") ]) ], children: [ {reste} ])");
        for (source, message) in [
            (part("Card()"), "attend le paramètre « title »"),
            (part("Card(titel: \"a\")"), "écris « title »"),
            (part("Card(title: \"a\", title: \"b\")"), "donné deux fois"),
            (part("Card(\"a\")"), "est nommé"),
            ("Page(components: [ Component(name: Card, children: [ Card() ]) ], children: [ Card() ])".into(), "se pose lui-même"),
            ("Page(components: [ Component(name: Text, children: [ P(\"x\") ]) ], children: [ Text() ])".into(), "déjà un bloc du langage"),
            ("Page(components: [ Component(name: Card, params: [add], children: [ P(\"x\") ]) ], children: [ ])".into(), "mot du langage"),
            ("Page(state: State(title: \"\"), components: [ Component(name: Card, params: [title], children: [ P(\"x\") ]) ], children: [ Card(title: \"a\") ])".into(), "nom d'une valeur de la page"),
            ("Page(components: [ Component(name: Card, children: [ P(\"a\"), P(\"b\") ]) ], children: [ Card() ])".into(), "un seul bloc racine"),
            ("Page(components: [ Component(name: Card, children: [ Button(name: Go, text: \"go\") ]) ], children: [ Card(), Card() ])".into(), "donne un nom à chaque copie"),
            ("Page(components: [ Component(name: Card, params: [n], children: [ Button(name: Go, text: \"go\") ], rules: [ On(Go.tap, effect: n.add(1)) ]) ], children: [ Card(n: 3) ])".into(), "doit recevoir le nom d'une valeur"),
            ("Page(children: [ P.Big(\"x\") ])".into(), "en minuscules"),
        ] {
            let erreur = crate::verifier_page(&source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }
}
