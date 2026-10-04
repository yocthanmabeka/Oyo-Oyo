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

use crate::holo::{Bloc, Erreur, Programme, Valeur};
use crate::regles::pour_chaque_bloc;

/// Garde-fous du langage : le nombre de valeurs d'une page, et jusqu'où va chacune.
pub const VALEURS_MAX: usize = 32;
pub const VALEUR_MAX: u64 = 1_000_000_000;

/// Les deux valeurs que le moteur calcule quand la page donne des prix (`prices:`) : le nombre
/// d'articles, et ce qu'ils coûtent ensemble. L'auteur n'écrit aucun calcul.
pub const CALCULEES: &[&str] = &["count", "total"];

/// Ce qu'on peut demander pour une valeur.
pub const DEMANDES: &[&str] = &["add", "sub", "set"];

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
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if bloc.nom != "On" {
            return Ok(());
        }
        let declencheur = bloc.arguments.iter().find(|a| a.nom.is_none()).map(|a| &a.valeur);
        if let (Some(Valeur::Nom(s)), Some(Valeur::Bloc(effet))) = (declencheur, bloc.argument("effect").map(|a| &a.valeur)) {
            if s == signal {
                if let Ok(d) = demande(effet, &etat) {
                    if let Some((_, valeur)) = etat.iter_mut().find(|(nom, _)| nom == d.valeur) {
                        *valeur = match d.verbe {
                            "add" => valeur.saturating_add(d.quantite).min(VALEUR_MAX),
                            "sub" => valeur.saturating_sub(d.quantite),
                            _ => d.quantite,
                        };
                    }
                }
            }
        }
        Ok(())
    });
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
