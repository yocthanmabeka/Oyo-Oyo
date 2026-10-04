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
pub fn condition(bloc: &Bloc) -> Result<(&str, Vec<(&str, u64)>), Erreur> {
    let erreur = |message: &str| Erreur { message: message.into(), pos: bloc.pos };
    let ecriture = "une condition s'écrit « If(count, is: 0, children: [ … ]) » ; comparaisons : is (égal), not (différent), over (plus grand), under (plus petit)";
    let valeur = match bloc.arguments.first() {
        Some(Argument { nom: None, valeur: Valeur::Nom(valeur), .. }) => valeur.as_str(),
        _ => return Err(erreur(ecriture)),
    };
    let mut comparaisons = Vec::new();
    for argument in &bloc.arguments[1..] {
        match (argument.nom.as_deref(), &argument.valeur) {
            (Some("children" | "name"), _) => {}
            (Some(mot), Valeur::Entier(nombre)) if COMPARAISONS.contains(&mot) => comparaisons.push((COMPARAISONS[COMPARAISONS.iter().position(|c| *c == mot).unwrap_or(0)], *nombre)),
            (Some(mot), _) if COMPARAISONS.contains(&mot) => return Err(Erreur { message: format!("« If({valeur}, {mot}: …) » attend un nombre entier"), pos: argument.pos }),
            (Some(mot), _) => return Err(Erreur { message: format!("« If » n'a pas de paramètre « {mot} » ; paramètres possibles : is, not, over, under, children"), pos: argument.pos }),
            (None, _) => return Err(erreur(ecriture)),
        }
    }
    if comparaisons.is_empty() {
        return Err(erreur(ecriture));
    }
    if !matches!(bloc.argument("children").map(|a| &a.valeur), Some(Valeur::Liste(_))) {
        return Err(erreur("« If » attend ce qu'il montre : If(count, is: 0, children: [ … ])"));
    }
    Ok((valeur, comparaisons))
}

/// Le nom sous lequel une condition est connue de la page : `count|is=0`, `total|over=0|under=300`.
/// Deux conditions écrites pareil portent le même nom, et ont toujours la même réponse.
pub fn cle(valeur: &str, comparaisons: &[(&str, u64)]) -> String {
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
                    reponses.push((cle, vraie(&comparaisons, nombre)));
                }
            }
        }
        Ok(())
    });
    reponses
}

/// La condition est-elle vraie pour cette valeur ?
pub fn vraie(comparaisons: &[(&str, u64)], valeur: u64) -> bool {
    comparaisons.iter().all(|(mot, nombre)| match *mot {
        "is" => valeur == *nombre,
        "not" => valeur != *nombre,
        "over" => valeur > *nombre,
        _ => valeur < *nombre,
    })
}

/// Ce qu'on peut demander pour une valeur.
pub const DEMANDES: &[&str] = &["add", "sub", "set", "random"];

/// Sous ce nom, l'état garde le nombre de tirages au hasard déjà faits. Ce n'est pas une valeur
/// de l'auteur (le nom n'est pas un nom permis) : il sert à ce que le hasard soit rejouable.
const TIRAGES: &str = "~";

/// Le rythme le plus rapide et le plus lent d'une règle `Every`, en millisecondes.
pub const RYTHME_MIN: u64 = 100;
pub const RYTHME_MAX: u64 = 3_600_000;

/// Le rythme d'une règle `Every(1s, effect: …)`, en millisecondes.
pub fn rythme(regle: &Bloc) -> Result<u64, Erreur> {
    let erreur = || Erreur {
        message: "une règle de temps s'écrit « Every(1s, effect: time.sub(1)) » : une durée en s ou en ms, de 100ms à 3600s".into(),
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

/// Les rythmes des règles de temps du fichier, sans doublon : la page tient une horloge par rythme.
pub fn rythmes(programme: &Programme) -> Vec<u64> {
    let mut rythmes = Vec::new();
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if bloc.nom == "Every" {
            if let Ok(ms) = rythme(bloc) {
                if !rythmes.contains(&ms) {
                    rythmes.push(ms);
                }
            }
        }
        Ok(())
    });
    rythmes
}

/// La graine du hasard d'un fichier : tirée du nom de sa page. Le même fichier, avec les mêmes
/// gestes aux mêmes moments, redonne la même partie (ADR-008).
fn graine_du_hasard(programme: &Programme) -> u64 {
    let nom = crate::regles::nom_de(&programme.racine).unwrap_or("Home");
    nom.bytes().fold(0x4A5E_u64, |g, octet| crate::graine::melanger(g ^ u64::from(octet)))
}

/// Les valeurs d'une page, dans l'ordre où elles sont déclarées.
pub type Etat = Vec<(String, u64)>;

/// `cart`, `items_seen` : une minuscule, puis des minuscules, des chiffres ou `_`.
fn est_nom_de_valeur(nom: &str) -> bool {
    nom.starts_with(|c: char| c.is_ascii_lowercase()) && nom.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
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
    Ok(etat)
}

fn initial_sans_prix(programme: &Programme) -> Result<Etat, Erreur> {
    let Some(bloc) = bloc_d_etat(programme)? else { return Ok(Vec::new()) };
    let mut etat = Etat::new();
    for argument in &bloc.arguments {
        let (Some(nom), Valeur::Entier(depart)) = (&argument.nom, &argument.valeur) else {
            return Err(Erreur { message: "une valeur se déclare par son nom et son départ, un nombre entier : State(cart: 0)".into(), pos: argument.pos });
        };
        if !est_nom_de_valeur(nom) {
            return Err(Erreur { message: format!("« {nom} » : le nom d'une valeur s'écrit en minuscules, comme « cart »"), pos: argument.pos });
        }
        if etat.iter().any(|(connu, _)| connu == nom) {
            return Err(Erreur { message: format!("la valeur « {nom} » est déclarée deux fois"), pos: argument.pos });
        }
        if *depart > VALEUR_MAX {
            return Err(Erreur { message: format!("« {nom} » : une valeur va de 0 à {VALEUR_MAX}"), pos: argument.pos });
        }
        etat.push((nom.clone(), *depart));
    }
    if etat.len() > VALEURS_MAX {
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
    if !etat.iter().any(|(connu, _)| connu == valeur) {
        return Err(erreur(format!("aucune valeur ne s'appelle « {valeur} » : déclare-la sur la page, state: State({valeur}: 0)")));
    }
    if !DEMANDES.contains(&verbe) {
        return Err(erreur(format!("demande inconnue « {verbe} » : pour une valeur, on peut demander {}", DEMANDES.join(", "))));
    }
    match bloc.arguments.as_slice() {
        [argument] if argument.nom.is_none() => match argument.valeur {
            // « random(0) » ne tirerait jamais que 0 : c'est sûrement une erreur.
            Valeur::Entier(0) if verbe == "random" => Err(erreur(format!("« {valeur}.random » attend le plus grand nombre possible, au moins 1 : {valeur}.random(100) tire de 0 à 100"))),
            Valeur::Entier(quantite) if quantite <= VALEUR_MAX => Ok(Demande { valeur, verbe, quantite }),
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
            if est_nom_de_valeur(&reste[..fin]) {
                noms.push(&reste[..fin]);
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
    // Ce qu'un texte peut montrer : les valeurs déclarées, et celles que le moteur calcule.
    let montrables = a_montrer(programme, &etat);
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
        // Une condition regarde une valeur que la page déclare, ou que le moteur calcule.
        if bloc.nom == "If" {
            let (valeur, _) = condition(bloc)?;
            if !montrables.iter().any(|(connu, _)| connu == valeur) {
                return Err(Erreur { message: format!("« If({valeur}, …) » : aucune valeur ne s'appelle « {valeur} » ; déclare-la sur la page, state: State({valeur}: 0)"), pos: bloc.pos });
            }
        }
        // Sur un plateau, la place d'un bloc est une valeur de la page : Point(x: star_x, y: star_y).
        for axe in ["x", "y"] {
            if let Some(Argument { valeur: Valeur::Nom(valeur), pos, .. }) = bloc.argument(axe) {
                if !montrables.iter().any(|(connu, _)| connu == valeur) {
                    return Err(Erreur { message: format!("« {axe}: {valeur} » : aucune valeur ne s'appelle « {valeur} » ; déclare-la sur la page, state: State({valeur}: 50)"), pos: *pos });
                }
            }
        }
        let verifier_texte = |texte: &str, pos| {
            noms_dans(texte).into_iter().find(|nom| !montrables.iter().any(|(connu, _)| connu == nom)).map_or(Ok(()), |nom| {
                Err(Erreur { message: format!("« {{{nom}}} » : aucune valeur ne s'appelle « {nom} » ; déclare-la sur la page, state: State({nom}: 0)"), pos })
            })
        };
        for argument in &bloc.arguments {
            match (&argument.nom.as_deref(), &argument.valeur) {
                (None | Some("text"), Valeur::Texte(texte)) => verifier_texte(texte, argument.pos)?,
                // Les phrases seules et les lignes d'une liste.
                (Some("children"), Valeur::Liste(elements)) => {
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
    let mut etat = etat.clone();
    let graine = graine_du_hasard(programme);
    let mut tirages = etat.iter().find(|(nom, _)| nom == TIRAGES).map_or(0, |(_, n)| *n);
    let depart = tirages;
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        // Une règle répond à un signal : celui d'un bloc (`On(Add.tap, …)`), ou celui de
        // l'horloge (`Every(1s, …)`, signal « every:1000 »).
        let concernee = match bloc.nom.as_str() {
            "On" => matches!(bloc.arguments.iter().find(|a| a.nom.is_none()).map(|a| &a.valeur), Some(Valeur::Nom(s)) if s == signal),
            "Every" => rythme(bloc).is_ok_and(|ms| signal == format!("every:{ms}")),
            _ => false,
        };
        if let (true, Some(Valeur::Bloc(effet))) = (concernee, bloc.argument("effect").map(|a| &a.valeur)) {
            if let Ok(d) = demande(effet, &etat) {
                if let Some((_, valeur)) = etat.iter_mut().find(|(nom, _)| nom == d.valeur) {
                    *valeur = match d.verbe {
                        "add" => valeur.saturating_add(d.quantite).min(VALEUR_MAX),
                        "sub" => valeur.saturating_sub(d.quantite),
                        // Le hasard n'en est pas un : c'est le énième tirage d'une suite fixée par
                        // la graine du fichier. Rejouer les mêmes gestes redonne les mêmes nombres.
                        "random" => {
                            tirages += 1;
                            crate::graine::melanger(graine ^ crate::graine::melanger(tirages)) % (d.quantite + 1)
                        }
                        _ => d.quantite,
                    };
                }
            }
        }
        Ok(())
    });
    if tirages != depart {
        match etat.iter_mut().find(|(nom, _)| nom == TIRAGES) {
            Some((_, n)) => *n = tirages,
            None => etat.push((TIRAGES.to_string(), tirages)),
        }
    }
    etat
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
  state: State(sunrise: 0, blue_door: 2, likes: 5),
  prices: Prices(sunrise: 120, blue_door: 90),
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
        assert_eq!(ecrire(&a_montrer(&programme, &depart)), "sunrise=0;blue_door=2;likes=5;count=2;total=180");
        let apres = arbitrer(&programme, &depart, "Add.tap");
        assert_eq!(ecrire(&a_montrer(&programme, &apres)), "sunrise=1;blue_door=2;likes=5;count=3;total=300");
        assert!(crate::vue_a_plat(BOUTIQUE, "").unwrap().contains("<span data-state=\"count\">2</span> paintings, <span data-state=\"total\">180</span> euros"));
        // Ce que la page renvoie contient les valeurs calculées ; elles ne sont pas reprises telles
        // quelles : le moteur les recalcule toujours.
        assert_eq!(crate::arbitrer(BOUTIQUE, "sunrise=1;blue_door=2;likes=5;count=999;total=1", "Add.tap"), "sunrise=2;blue_door=2;likes=5;count=4;total=420");
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
        assert!(vraie(&[("over", 0), ("under", 3)], 2) && !vraie(&[("over", 0), ("under", 3)], 3) && !vraie(&[("over", 0), ("under", 3)], 0));
        assert!(vraie(&[("is", 5)], 5) && vraie(&[("not", 5)], 4) && !vraie(&[("not", 5)], 5));
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
        assert_eq!(rythmes(&programme), [1000]);
        let depart = initial(&programme).unwrap();
        assert_eq!(ecrire(&depart), "time=0;score=0;star_x=50;star_y=50");
        // Tant que la partie n'a pas commencé, le temps reste à zéro : il ne descend pas dessous.
        let attente = arbitrer(&programme, &depart, "every:1000");
        assert_eq!(attente[0], ("time".to_string(), 0));
        // « Play » : trente secondes. Puis une seconde passe, l'étoile a bougé.
        let lancee = arbitrer(&programme, &depart, "Play.tap");
        assert_eq!((lancee[0].1, lancee[1].1), (30, 0));
        let une_seconde = arbitrer(&programme, &lancee, "every:1000");
        assert_eq!(une_seconde[0].1, 29);
        assert!(une_seconde[2].1 <= 100 && une_seconde[3].1 <= 100);
        assert_ne!((une_seconde[2].1, une_seconde[3].1), (50, 50), "l'étoile n'a pas bougé");
        // Toucher l'étoile : un point, et elle part ailleurs.
        let touchee = arbitrer(&programme, &une_seconde, "Star.tap");
        assert_eq!(touchee[1].1, 1);
        assert_ne!((touchee[2].1, touchee[3].1), (une_seconde[2].1, une_seconde[3].1));
        // Trente secondes plus tard, la partie est finie, et le score est gardé.
        let fin = (0..40).fold(touchee, |etat, _| arbitrer(&programme, &etat, "every:1000"));
        assert_eq!((fin[0].1, fin[1].1), (0, 1));
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
            ("Page(state: State(a: 0), children: [ Point(name: P, seed: 1, inside: World(children: [])) ], rules: [ Every(1s, effect: P.enter) ])", "attend une demande"),
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
            ("Page(state: State(Cart: 0))", "en minuscules"),
            ("Page(state: State(cart: \"zero\"))", "un nombre entier"),
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
