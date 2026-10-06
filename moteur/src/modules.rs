//! Les modules enfermés (ADR-011, partie C ; ADR-045) : du code compilé en WebAssembly, que le
//! moteur fait tourner dans une boîte fermée.
//!
//! ```holo
//! module "somme.wasm"
//! Page(
//!   state: State(n: 10, total: 0),
//!   modules: [ Module(name: Sum, source: "somme.wasm", input: n, output: total, time: 100ms, memory: 1MB) ],
//!   children: [ Button(name: Go, text: "Compute"), P("{total}") ],
//!   rules: [ On(Go.tap, effect: Sum.run), On(Sum.failed, effect: …) ],
//! )
//! ```
//!
//! La boîte : le module ne reçoit qu'un nombre (`input`) et ne rend qu'un nombre (`output`) ; il
//! n'a accès à rien d'autre (ni réseau, ni page, ni heure, ni stockage). Il tourne à part, sans
//! jamais bloquer la page ; s'il dépasse son temps (`time`), il est arrêté ; sa mémoire ne peut
//! pas grandir au-delà de son plafond (`memory`). Il se présente en holoscénique : une capacité
//! (`run`) et deux signaux (`done`, `failed`), jamais du Rust. Chaque module est annoncé en haut
//! du fichier, `module "somme.wasm"`, pour que le risque se lise d'un coup d'œil.

use crate::holo::{Bloc, Erreur, Programme, Valeur};

/// Les bornes d'un module : son temps, et sa mémoire (en pages de 64 Ko, celles de WebAssembly).
pub const TEMPS_MIN: u64 = 10;
pub const TEMPS_MAX: u64 = 5_000;
pub const TEMPS_COURANT: u64 = 100;
pub const PAGE: u64 = 65_536;
pub const PAGES_MAX: u64 = 256;
pub const PAGES_COURANTES: u64 = 16;
pub const MODULES_MAX: usize = 8;

/// Un module déclaré par la page.
#[derive(Debug, PartialEq)]
pub struct Module<'a> {
    pub nom: &'a str,
    pub source: &'a str,
    pub entree: Option<&'a str>,
    pub sortie: &'a str,
    pub temps: u64,
    pub pages: u64,
}

fn exemple() -> &'static str {
    "modules: [ Module(name: Sum, source: \"somme.wasm\", input: n, output: total) ]"
}

/// Les modules de la page, vérifiés. `nombres` : les valeurs de la page qui sont des nombres.
pub fn modules<'a>(programme: &'a Programme, nombres: &[(String, u64)]) -> Result<Vec<Module<'a>>, Erreur> {
    let annonces: Vec<&crate::holo::Import> = programme.imports.iter().filter(|i| i.sorte == "module").collect();
    let Some(argument) = programme.racine.argument("modules") else {
        if let Some(annonce) = annonces.first() {
            return Err(Erreur { message: format!("« module \"{}\" » est annoncé en haut du fichier, mais la page ne le déclare pas : {}", annonce.cible, exemple()), pos: annonce.pos });
        }
        return Ok(Vec::new());
    };
    let Valeur::Liste(liste) = &argument.valeur else {
        return Err(Erreur { message: format!("« modules » est une liste : {}", exemple()), pos: argument.pos });
    };
    if liste.len() > MODULES_MAX {
        return Err(Erreur { message: format!("une page fait tourner au plus {MODULES_MAX} modules"), pos: argument.pos });
    }
    let est_nombre = |nom: &str| nombres.iter().any(|(connu, _)| connu == nom);
    let mut lus = Vec::new();
    for valeur in liste {
        let Valeur::Bloc(bloc) = valeur else {
            return Err(Erreur { message: format!("« modules » contient des « Module(…) » : {}", exemple()), pos: argument.pos });
        };
        if bloc.nom != "Module" {
            return Err(Erreur { message: format!("« modules » contient des « Module(…) », pas des « {} »", bloc.nom), pos: bloc.pos });
        }
        lus.push(un_module(bloc, &est_nombre)?);
    }
    // Chaque module se lit en haut du fichier ; chaque annonce a son module.
    for module in &lus {
        if !annonces.iter().any(|a| a.cible == module.source) {
            return Err(Erreur { message: format!("le module « {} » doit être annoncé en haut du fichier, pour que le risque se lise d'un coup d'œil : module \"{}\"", module.nom, module.source), pos: argument.pos });
        }
    }
    for annonce in &annonces {
        if !lus.iter().any(|m| m.source == annonce.cible) {
            return Err(Erreur { message: format!("« module \"{}\" » est annoncé, mais aucun Module ne l'emploie", annonce.cible), pos: annonce.pos });
        }
    }
    Ok(lus)
}

fn un_module<'a>(bloc: &'a Bloc, est_nombre: &dyn Fn(&str) -> bool) -> Result<Module<'a>, Erreur> {
    let erreur = |message: String, pos| Erreur { message, pos };
    let (mut nom, mut source, mut entree, mut sortie, mut temps, mut pages) = (None, None, None, None, TEMPS_COURANT, PAGES_COURANTES);
    for a in &bloc.arguments {
        match (a.nom.as_deref(), &a.valeur) {
            (Some("name"), Valeur::Nom(n)) => nom = Some(n.as_str()),
            (Some("source"), Valeur::Texte(s)) if s.ends_with(".wasm") && crate::plat::chemin_sur(s) => source = Some(s.as_str()),
            (Some("source"), _) => return Err(erreur("« Module(source: …) » attend un fichier .wasm rangé à côté de la page, comme \"somme.wasm\"".into(), a.pos)),
            (Some("input"), Valeur::Nom(v)) if est_nombre(v) => entree = Some(v.as_str()),
            (Some("output"), Valeur::Nom(v)) if est_nombre(v) && !crate::etat::HORLOGE.contains(&v.as_str()) => sortie = Some(v.as_str()),
            (Some(p @ ("input" | "output")), _) => return Err(erreur(format!("« Module({p}: …) » attend le nom d'un nombre de la page : un module reçoit un nombre et rend un nombre"), a.pos)),
            (Some("time"), Valeur::Nombre { valeur, unite: Some(u) }) if u == "ms" || u == "s" => {
                let ms = if u == "s" { valeur * 1000.0 } else { *valeur };
                if !(TEMPS_MIN as f64..=TEMPS_MAX as f64).contains(&ms) {
                    return Err(erreur(format!("« Module(time: …) » va de {TEMPS_MIN}ms à 5s : au-delà, le module est arrêté"), a.pos));
                }
                temps = ms.round() as u64;
            }
            (Some("time"), _) => return Err(erreur("« Module(time: …) » attend une durée, comme 100ms".into(), a.pos)),
            (Some("memory"), Valeur::Nombre { valeur, unite: Some(u) }) if u == "KB" || u == "MB" => {
                let octets = if u == "MB" { valeur * 1e6 } else { valeur * 1e3 };
                let voulues = (octets / PAGE as f64).ceil() as u64;
                if !(1..=PAGES_MAX).contains(&voulues) {
                    return Err(erreur("« Module(memory: …) » va de 64KB à 16MB".into(), a.pos));
                }
                pages = voulues;
            }
            (Some("memory"), _) => return Err(erreur("« Module(memory: …) » attend une taille, comme 1MB".into(), a.pos)),
            (Some(autre), _) => return Err(erreur(format!("« Module » n'a pas de paramètre « {autre} » ; paramètres possibles : name, source, input, output, time, memory"), a.pos)),
            (None, _) => return Err(erreur(format!("chaque paramètre de « Module » est nommé : {}", exemple()), a.pos)),
        }
    }
    match (nom, source, sortie) {
        (Some(nom), Some(source), Some(sortie)) => Ok(Module { nom, source, entree, sortie, temps, pages }),
        _ => Err(erreur(format!("« Module » attend name, source et output : {}", exemple()), bloc.pos)),
    }
}

/// Le module a rendu son nombre : il va dans sa valeur de sortie, bornée ; puis `Nom.done`.
pub fn fini(programme: &Programme, etat: &crate::etat::Etat, nom: &str, valeur: u64) -> crate::etat::Etat {
    let nombres = crate::etat::initial(programme).unwrap_or_default();
    let Some(module) = modules(programme, &nombres).ok().and_then(|m| m.into_iter().find(|m| m.nom == nom)) else { return etat.clone() };
    let mut etat = etat.clone();
    if let Some((_, place)) = etat.iter_mut().find(|(connu, _)| connu == module.sortie) {
        *place = valeur.min(crate::etat::VALEUR_MAX);
    }
    crate::etat::arbitrer(programme, &etat, &format!("{nom}.done"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn un_module_se_declare_et_s_annonce() {
        let source = "module \"somme.wasm\"\nPage(state: State(n: 10, total: 0, ok: 0), modules: [ Module(name: Sum, source: \"somme.wasm\", input: n, output: total, time: 50ms, memory: 1MB) ], children: [ Button(name: Go, text: \"g\") ], rules: [ On(Go.tap, effect: Sum.run), On(Sum.done, effect: ok.set(1)), On(Sum.failed, effect: ok.set(2)) ])";
        crate::verifier_page(source).unwrap();
        assert_eq!(crate::module_info(source, "n=10;total=0;ok=0", "Sum"), "somme.wasm|10|50|16");
        assert_eq!(crate::module_fini(source, "n=10;total=0;ok=0", "Sum", 55), "n=10;total=55;ok=1");
        for (source, message) in [
            ("Page(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t) ], children: [])", "doit être annoncé en haut du fichier"),
            ("module \"s.wasm\"\nPage(children: [])", "la page ne le déclare pas"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t, time: 9s) ], children: [])", "de 10ms à 5s"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t, memory: 1GB) ], children: [])", "attend une taille"),
            ("module \"s.wasm\"\nPage(state: State(t: \"\"), modules: [ Module(name: S, source: \"s.wasm\", output: t) ], children: [])", "un module reçoit un nombre et rend un nombre"),
            ("module \"../s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"../s.wasm\", output: t) ], children: [])", "rangé à côté de la page"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t) ], children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: S.fly) ])", "un « Module » offre run"),
        ] {
            let erreur = crate::verifier_page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }
}
