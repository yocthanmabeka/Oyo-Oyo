//! Les noms, les règles et les budgets d'un fichier (ADR-005, ADR-015).
//!
//! Un bloc ne contient aucun code : une règle `On(Open.tap, effect: Workshop.enter)` relie
//! un signal à une capacité demandée. Ce fichier vérifie ces règles avant toute exécution,
//! puis répond à la seule question que pose l'affichage : « ce signal, que demande-t-il ? ».

use crate::holo::{Bloc, Erreur, Programme, Valeur};

/// Ce qu'un bloc sait émettre (signaux) et ce qu'on peut lui demander (capacités).
fn signaux(bloc: &str) -> &'static [&'static str] {
    match bloc {
        "Button" | "Point" => &["tap"],
        _ => &[],
    }
}

fn capacites(bloc: &str) -> &'static [&'static str] {
    match bloc {
        "Point" => &["enter", "leave"],
        // Ouvrir le carrefour : les portails vers les mondes voisins.
        "Page" => &["portals"],
        _ => &[],
    }
}

/// Parcourt tous les blocs du fichier, dans l'ordre où ils sont écrits.
pub fn pour_chaque_bloc<'a>(bloc: &'a Bloc, f: &mut dyn FnMut(&'a Bloc) -> Result<(), Erreur>) -> Result<(), Erreur> {
    fn visiter<'a>(valeur: &'a Valeur, f: &mut dyn FnMut(&'a Bloc) -> Result<(), Erreur>) -> Result<(), Erreur> {
        match valeur {
            Valeur::Bloc(bloc) => pour_chaque_bloc(bloc, f),
            Valeur::Liste(elements) => elements.iter().try_for_each(|e| visiter(e, f)),
            _ => Ok(()),
        }
    }
    f(bloc)?;
    bloc.arguments.iter().try_for_each(|a| visiter(&a.valeur, f))
}

/// Le nom donné à un bloc par `name: Open`.
pub fn nom_de(bloc: &Bloc) -> Option<&str> {
    match &bloc.argument("name")?.valeur {
        Valeur::Nom(nom) => Some(nom),
        _ => None,
    }
}

/// Le bloc qui porte ce nom.
pub fn bloc_nomme<'a>(programme: &'a Programme, nom: &str) -> Option<&'a Bloc> {
    let mut trouve = None;
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if trouve.is_none() && nom_de(bloc) == Some(nom) {
            trouve = Some(bloc);
        }
        Ok(())
    });
    trouve
}

/// Le site que désigne un chemin de points : vide, c'est la page du fichier ; `Shop/Secret`,
/// c'est le monde du point `Secret`, lui-même dans le monde du point `Shop`.
pub fn site_de<'a>(programme: &'a Programme, chemin: &str) -> Result<&'a Bloc, Erreur> {
    let mut site = &programme.racine;
    for nom in chemin.split('/').filter(|n| !n.is_empty()) {
        let mut trouve = None;
        let _ = pour_chaque_bloc(site, &mut |bloc| {
            if trouve.is_none() && bloc.nom == "Point" && nom_de(bloc) == Some(nom) {
                trouve = Some(bloc);
            }
            Ok(())
        });
        site = match trouve.and_then(|point| point.argument("inside")).map(|a| &a.valeur) {
            Some(Valeur::Bloc(monde)) if monde.nom == "World" => monde,
            _ => return Err(Erreur { message: format!("aucun point nommé « {nom} » ne contient un monde, à cet endroit du fichier"), pos: site.pos }),
        };
    }
    Ok(site)
}

/// Vérifie les noms (aucun en double), les règles (signaux et capacités connus) et les
/// budgets (le contenu d'un point ne pèse pas plus que ce qu'il déclare).
pub fn verifier_regles(programme: &Programme) -> Result<(), Erreur> {
    let mut noms: Vec<(&str, &str)> = Vec::new();
    pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if let Some(nom) = nom_de(bloc) {
            if noms.iter().any(|(connu, _)| *connu == nom) {
                let pos = bloc.argument("name").map_or(bloc.pos, |a| a.pos);
                return Err(Erreur { message: format!("le nom « {nom} » est déjà porté par un autre bloc : deux blocs ne partagent pas un nom"), pos });
            }
            noms.push((nom, &bloc.nom));
        }
        Ok(())
    })?;
    verifier_reperes(&programme.racine)?;
    let etat = crate::etat::initial(programme)?;
    pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if bloc.nom == "On" {
            verifier_regle(bloc, &noms, &etat)?;
        }
        // Une règle qui guette une valeur, ou la rencontre de deux blocs (ADR-028).
        if bloc.nom == "When" {
            if bloc.argument("meets").is_some() {
                let (a, b, _) = crate::etat::rencontre(bloc)?;
                for nom in [a, b] {
                    let pose = bloc_nomme(programme, nom).is_some_and(|pose| pose.argument("x").is_some() && pose.argument("y").is_some());
                    if !pose {
                        return Err(Erreur { message: format!("une rencontre : « {nom} » n'est pas posé sur un plateau ; il lui faut un nom, et « x » et « y » dans un « Board »"), pos: bloc.pos });
                    }
                }
            } else {
                crate::etat::condition(bloc)?;
            }
            verifier_effets(bloc, &noms, &etat, false)?;
        }
        // Une règle de temps : un rythme, et une ou plusieurs demandes faites à l'arbitre (ADR-026).
        if bloc.nom == "Every" {
            crate::etat::rythme(bloc)?;
            verifier_effets(bloc, &noms, &etat, false)?;
        }
        if bloc.nom == "Point" {
            verifier_budget(bloc)?;
        }
        Ok(())
    })
}

/// Un point planté dans un pixel se repère par rapport à un bloc du même site : la page, ou
/// le monde, où il est planté. Un bloc rangé dans le monde d'un autre point n'est pas à
/// l'écran au même moment : l'affichage ne le trouverait pas (revue Codex, B-06).
fn verifier_reperes(site: &Bloc) -> Result<(), Erreur> {
    fn visiter<'a>(valeur: &'a Valeur, noms: &mut Vec<&'a str>, mondes: &mut Vec<&'a Bloc>) {
        match valeur {
            Valeur::Liste(elements) => elements.iter().for_each(|e| visiter(e, noms, mondes)),
            Valeur::Bloc(bloc) => {
                if let Some(nom) = nom_de(bloc) {
                    noms.push(nom);
                }
                for argument in &bloc.arguments {
                    match &argument.valeur {
                        // Le monde d'un point est un autre site : on n'y descend pas.
                        Valeur::Bloc(monde) if bloc.nom == "Point" && argument.nom.as_deref() == Some("inside") => mondes.push(monde),
                        autre => visiter(autre, noms, mondes),
                    }
                }
            }
            _ => {}
        }
    }
    let (mut noms, mut mondes) = (Vec::new(), Vec::new());
    site.arguments.iter().for_each(|a| visiter(&a.valeur, &mut noms, &mut mondes));
    if let Some(Valeur::Liste(plantes)) = site.argument("pixels").map(|a| &a.valeur) {
        for plante in plantes {
            let Valeur::Bloc(point) = plante else { continue };
            if let Some(argument) = point.argument("above") {
                let connu = matches!(&argument.valeur, Valeur::Nom(repere) if noms.contains(&repere.as_str()));
                if !connu {
                    return Err(Erreur { message: "« above » attend le nom d'un bloc de la page où le point est planté : above: Open".into(), pos: argument.pos });
                }
            }
        }
    }
    mondes.into_iter().try_for_each(verifier_reperes)
}

/// `Open.tap` → (`Open`, `tap`).
fn nom_et_mot<'a>(bloc: &'a Bloc, valeur: Option<&'a Valeur>, quoi: &str) -> Result<(&'a str, &'a str), Erreur> {
    match valeur {
        Some(Valeur::Nom(n)) => n.split_once('.').filter(|(a, b)| !a.is_empty() && !b.is_empty() && !b.contains('.')),
        _ => None,
    }
    .ok_or_else(|| Erreur { message: format!("une règle s'écrit « On(Open.tap, effect: Workshop.enter) » : {quoi} manque ou est mal écrit"), pos: bloc.pos })
}

fn verifier_regle(regle: &Bloc, noms: &[(&str, &str)], etat: &crate::etat::Etat) -> Result<(), Erreur> {
    let signal = regle.arguments.iter().find(|a| a.nom.is_none()).map(|a| &a.valeur);
    let (source, mot) = nom_et_mot(regle, signal, "le signal")?;
    let type_de = |nom: &str| noms.iter().find(|(connu, _)| *connu == nom).map(|(_, bloc)| *bloc);
    let inconnu = |nom: &str| Erreur { message: format!("aucun bloc ne s'appelle « {nom} »"), pos: regle.pos };
    // « Key » est le clavier du visiteur : On(Key.left, effect: basket.sub(8)).
    if source == "Key" && type_de(source).is_none() {
        if !crate::etat::TOUCHES.contains(&mot) {
            return Err(Erreur { message: format!("touche inconnue « {mot} » : le clavier donne {}", crate::etat::TOUCHES.join(", ")), pos: regle.pos });
        }
    } else {
        let type_source = type_de(source).ok_or_else(|| inconnu(source))?;
        if !signaux(type_source).contains(&mot) {
            return Err(Erreur { message: format!("signal inconnu « {mot} » : un « {type_source} » émet {}", lister(signaux(type_source))), pos: regle.pos });
        }
    }
    verifier_effets(regle, noms, etat, true)
}

/// Les effets d'une règle : un seul, ou plusieurs entre crochets.
fn effets_de(regle: &Bloc) -> Vec<&Valeur> {
    match regle.argument("effect").map(|a| &a.valeur) {
        Some(Valeur::Liste(elements)) => elements.iter().collect(),
        Some(effet) => vec![effet],
        None => Vec::new(),
    }
}

/// Vérifie les effets d'une règle. Un effet est une demande faite à l'arbitre (`cart.add(1)`,
/// ADR-023) ou, pour une règle `On`, une capacité demandée à un bloc (`Workshop.enter`).
fn verifier_effets(regle: &Bloc, noms: &[(&str, &str)], etat: &crate::etat::Etat, capacites_permises: bool) -> Result<(), Erreur> {
    let type_de = |nom: &str| noms.iter().find(|(connu, _)| *connu == nom).map(|(_, bloc)| *bloc);
    let attend_une_demande = || Erreur { message: format!("« {} » attend une demande : {}(…, effect: score.add(1))", regle.nom, regle.nom), pos: regle.pos };
    let effets = effets_de(regle);
    if effets.is_empty() {
        return if capacites_permises { nom_et_mot(regle, None, "l'effet").map(|_| ()) } else { Err(attend_une_demande()) };
    }
    for effet in effets {
        match effet {
            Valeur::Bloc(demande) if crate::etat::est_demande(demande) => {
                crate::etat::demande(demande, etat)?;
            }
            Valeur::Nom(_) if capacites_permises => {
                let (cible, capacite) = nom_et_mot(regle, Some(effet), "l'effet")?;
                if etat.iter().any(|(valeur, _)| valeur == cible) {
                    return Err(Erreur { message: format!("« {cible}.{capacite} » s'écrit avec sa quantité, entre parenthèses : {cible}.{capacite}(1)"), pos: regle.pos });
                }
                let type_cible = type_de(cible).ok_or_else(|| Erreur { message: format!("aucun bloc ne s'appelle « {cible} »"), pos: regle.pos })?;
                if !capacites(type_cible).contains(&capacite) {
                    return Err(Erreur { message: format!("capacité inconnue « {capacite} » : un « {type_cible} » offre {}", lister(capacites(type_cible))), pos: regle.pos });
                }
            }
            _ if capacites_permises => {
                nom_et_mot(regle, Some(effet), "l'effet")?;
            }
            _ => return Err(attend_une_demande()),
        }
    }
    Ok(())
}

fn lister(mots: &[&str]) -> String {
    if mots.is_empty() {
        "rien".into()
    } else {
        mots.join(", ")
    }
}

/// Un poids en octets. Les tailles sont décimales : 1 KB = 1 000 octets.
fn octets(bloc: &Bloc, param: &str) -> Result<Option<f64>, Erreur> {
    let Some(argument) = bloc.argument(param) else { return Ok(None) };
    let facteur = |unite: &str| match unite {
        "B" => Some(1.0),
        "KB" => Some(1e3),
        "MB" => Some(1e6),
        "GB" => Some(1e9),
        _ => None,
    };
    match &argument.valeur {
        Valeur::Nombre { valeur, unite: Some(unite) } if *valeur >= 0.0 && facteur(unite).is_some() => Ok(facteur(unite).map(|f| valeur * f)),
        _ => Err(Erreur { message: format!("le paramètre « {param} » attend une taille, comme « 500KB » (unités : B, KB, MB, GB)"), pos: argument.pos }),
    }
}

fn verifier_budget(point: &Bloc) -> Result<(), Erreur> {
    let budget = octets(point, "budget")?;
    let mut poids = 0.0;
    if let Some(Valeur::Bloc(interieur)) = point.argument("inside").map(|a| &a.valeur) {
        pour_chaque_bloc(interieur, &mut |bloc| {
            poids += octets(bloc, "weight")?.unwrap_or(0.0);
            Ok(())
        })?;
    }
    match budget {
        Some(budget) if poids > budget => Err(Erreur {
            message: format!("budget dépassé : le contenu pèse {poids} octets, le budget est de {budget} octets (ADR-005)"),
            pos: point.argument("budget").map_or(point.pos, |a| a.pos),
        }),
        _ => Ok(()),
    }
}

/// Les effets demandés par un signal, dans l'ordre où les règles sont écrites.
/// `Open.tap` → [`Workshop.enter`]. Toucher un point demande d'y entrer, sans règle à écrire.
pub fn effets(programme: &Programme, signal: &str) -> Vec<String> {
    let mut effets = Vec::new();
    let _ = pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if bloc.nom == "On" {
            let declencheur = bloc.arguments.iter().find(|a| a.nom.is_none()).map(|a| &a.valeur);
            if matches!(declencheur, Some(Valeur::Nom(s)) if s == signal) {
                for effet in effets_de(bloc) {
                    if let Valeur::Nom(effet) = effet {
                        effets.push(effet.clone());
                    }
                }
            }
        }
        Ok(())
    });
    if let Some(nom) = signal.strip_suffix(".tap") {
        if bloc_nomme(programme, nom).is_some_and(|b| b.nom == "Point") && effets.is_empty() {
            effets.push(format!("{nom}.enter"));
        }
    }
    effets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::lire;

    fn verifier(src: &str) -> Result<(), Erreur> {
        verifier_regles(&lire(src)?)
    }

    #[test]
    fn refuse_ce_que_la_suite_refuse_a_la_bonne_ligne() {
        let cas = [
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E02-unite-incompatible.holo"), 5, "attend une taille"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E03-budget-depasse.holo"), 5, "budget dépassé"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E06-capacite-inconnue.holo"), 8, "capacité inconnue « fly »"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E07-nom-en-double.holo"), 5, "déjà porté"),
        ];
        for (source, ligne, message) in cas {
            let erreur = verifier(source).unwrap_err();
            assert_eq!(erreur.pos.ligne, ligne, "{erreur}");
            assert!(erreur.message.contains(message), "{erreur}");
        }
    }

    #[test]
    fn accepte_les_cas_valides_et_la_boutique() {
        for source in [
            include_str!("../../experiments/conformite-v0.1/cas/valides/03-entrer-dans-un-point.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/04-budget-respecte.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/06-boutique-avec-styles.holo"),
            include_str!("../../exemples/boutique-comparee/boutique.holo"),
        ] {
            verifier(source).unwrap();
        }
    }

    #[test]
    fn une_regle_mal_ecrite_est_refusee() {
        let page = |regle: &str| format!("Page(children: [ Button(name: Open, text: \"x\"), Point(name: A, seed: 1) ], rules: [ {regle} ])");
        assert!(verifier(&page("On(Open.tap, effect: A.enter)")).is_ok());
        assert!(verifier(&page("On(Open.hover, effect: A.enter)")).unwrap_err().message.contains("signal inconnu « hover »"));
        assert!(verifier(&page("On(Nobody.tap, effect: A.enter)")).unwrap_err().message.contains("aucun bloc ne s'appelle « Nobody »"));
        assert!(verifier(&page("On(Open.tap, effect: Open.enter)")).unwrap_err().message.contains("un « Button » offre rien"));
        assert!(verifier(&page("On(Open.tap)")).unwrap_err().message.contains("l'effet manque"));
        // Une page offre une capacité : ouvrir son carrefour.
        assert!(verifier("Page(name: Shop, children: [ Button(name: Map, text: \"x\") ], rules: [ On(Map.tap, effect: Shop.portals) ])").is_ok());
        assert!(verifier("Page(name: Shop, children: [ Button(name: Map, text: \"x\") ], rules: [ On(Map.tap, effect: Shop.enter) ])").unwrap_err().message.contains("un « Page » offre portals"));
        assert!(verifier("Page(children: [ Button(name: Open, text: \"x\") ], pixels: [ Point(name: S, seed: 1, above: Open) ])").is_ok());
        assert!(verifier("Page(pixels: [ Point(name: S, seed: 1, above: Nobody) ])").unwrap_err().message.contains("le nom d'un bloc de la page"));
        // Le repère doit être dans le même site : pas dans le monde d'un autre point.
        let ailleurs = "Page(children: [ Point(name: C, seed: 1, inside: World(children: [ Button(name: Inner, text: \"x\") ])) ], pixels: [ Point(name: S, seed: 2, above: Inner) ])";
        assert!(verifier(ailleurs).unwrap_err().message.contains("où le point est planté"));
        // Dans un monde, le repère se cherche dans ce monde.
        let dedans = "Page(children: [ Point(name: C, seed: 1, inside: World(children: [ Button(name: Inner, text: \"x\") ], pixels: [ Point(name: S, seed: 2, above: Inner) ])) ])";
        assert!(verifier(dedans).is_ok());
    }

    #[test]
    fn un_signal_donne_ses_effets() {
        let p = lire(include_str!("../../exemples/boutique-comparee/boutique.holo")).unwrap();
        assert_eq!(effets(&p, "Open.tap"), ["Workshop.enter"]);
        assert_eq!(effets(&p, "Back.tap"), ["Workshop.leave"]);
        // Toucher un point demande d'y entrer, sans règle à écrire.
        assert_eq!(effets(&p, "Workshop.tap"), ["Workshop.enter"]);
        assert!(effets(&p, "Open.hover").is_empty());
    }
}
