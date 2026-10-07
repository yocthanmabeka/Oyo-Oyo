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

use crate::holo::{Block, Error, Program, Value};

/// Les bornes d'un module : son temps, et sa mémoire (en pages de 64 Ko, celles de WebAssembly).
pub const TIME_MIN: u64 = 10;
pub const TIME_MAX: u64 = 5_000;
pub const CURRENT_TIME: u64 = 100;
pub const PAGE: u64 = 65_536;
pub const PAGES_MAX: u64 = 256;
pub const CURRENT_PAGES: u64 = 16;
pub const MODULES_MAX: usize = 8;

/// Un module déclaré par la page.
#[derive(Debug, PartialEq)]
pub struct Module<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub entry: Option<&'a str>,
    pub output: &'a str,
    pub time: u64,
    pub pages: u64,
}

fn example() -> &'static str {
    "modules: [ Module(name: Sum, source: \"somme.wasm\", input: n, output: total) ]"
}

/// Les modules de la page, vérifiés. `nombres` : les valeurs de la page qui sont des nombres.
pub fn modules<'a>(program: &'a Program, numbers: &[(String, u64)]) -> Result<Vec<Module<'a>>, Error> {
    let announcements: Vec<&crate::holo::Import> = program.imports.iter().filter(|i| i.kind == "module").collect();
    let Some(argument) = program.root.argument("modules") else {
        if let Some(announcement) = announcements.first() {
            return Err(Error { message: format!("« module \"{}\" » est annoncé en haut du fichier, mais la page ne le déclare pas : {}", announcement.target, example()), pos: announcement.pos });
        }
        return Ok(Vec::new());
    };
    let Value::List(list) = &argument.value else {
        return Err(Error { message: format!("« modules » est une liste : {}", example()), pos: argument.pos });
    };
    if list.len() > MODULES_MAX {
        return Err(Error { message: format!("une page fait tourner au plus {MODULES_MAX} modules"), pos: argument.pos });
    }
    let is_number = |name: &str| numbers.iter().any(|(known, _)| known == name);
    let mut read_ones = Vec::new();
    for value in list {
        let Value::Block(block) = value else {
            return Err(Error { message: format!("« modules » contient des « Module(…) » : {}", example()), pos: argument.pos });
        };
        if block.name != "Module" {
            return Err(Error { message: format!("« modules » contient des « Module(…) », pas des « {} »", block.name), pos: block.pos });
        }
        read_ones.push(a_module(block, &is_number)?);
    }
    // Chaque module se lit en haut du fichier ; chaque annonce a son module.
    for module in &read_ones {
        if !announcements.iter().any(|a| a.target == module.source) {
            return Err(Error { message: format!("le module « {} » doit être annoncé en haut du fichier, pour que le risque se lise d'un coup d'œil : module \"{}\"", module.name, module.source), pos: argument.pos });
        }
    }
    for announcement in &announcements {
        if !read_ones.iter().any(|m| m.source == announcement.target) {
            return Err(Error { message: format!("« module \"{}\" » est annoncé, mais aucun Module ne l'emploie", announcement.target), pos: announcement.pos });
        }
    }
    Ok(read_ones)
}

fn a_module<'a>(block: &'a Block, is_number: &dyn Fn(&str) -> bool) -> Result<Module<'a>, Error> {
    let error = |message: String, pos| Error { message, pos };
    let (mut name, mut source, mut entry, mut output, mut time, mut pages) = (None, None, None, None, CURRENT_TIME, CURRENT_PAGES);
    for a in &block.arguments {
        match (a.name.as_deref(), &a.value) {
            (Some("name"), Value::Name(n)) => name = Some(n.as_str()),
            (Some("source"), Value::Text(s)) if s.ends_with(".wasm") && crate::flat::path_on(s) => source = Some(s.as_str()),
            (Some("source"), _) => return Err(error("« Module(source: …) » attend un fichier .wasm rangé à côté de la page, comme \"somme.wasm\"".into(), a.pos)),
            (Some("input"), Value::Name(v)) if is_number(v) => entry = Some(v.as_str()),
            (Some("output"), Value::Name(v)) if is_number(v) && !crate::state::CLOCK.contains(&v.as_str()) => output = Some(v.as_str()),
            (Some(p @ ("input" | "output")), _) => return Err(error(format!("« Module({p}: …) » attend le nom d'un nombre de la page : un module reçoit un nombre et rend un nombre"), a.pos)),
            (Some("time"), Value::Number { value, unit: Some(u), .. }) if u == "ms" || u == "s" => {
                let ms = if u == "s" { value * 1000.0 } else { *value };
                if !(TIME_MIN as f64..=TIME_MAX as f64).contains(&ms) {
                    return Err(error(format!("« Module(time: …) » va de {TIME_MIN}ms à 5s : au-delà, le module est arrêté"), a.pos));
                }
                time = ms.round() as u64;
            }
            (Some("time"), _) => return Err(error("« Module(time: …) » attend une durée, comme 100ms".into(), a.pos)),
            (Some("memory"), Value::Number { value, unit: Some(u), .. }) if u == "KB" || u == "MB" => {
                let bytes = if u == "MB" { value * 1e6 } else { value * 1e3 };
                let wanted_ones = (bytes / PAGE as f64).ceil() as u64;
                if !(1..=PAGES_MAX).contains(&wanted_ones) {
                    return Err(error("« Module(memory: …) » va de 64KB à 16MB".into(), a.pos));
                }
                pages = wanted_ones;
            }
            (Some("memory"), _) => return Err(error("« Module(memory: …) » attend une taille, comme 1MB".into(), a.pos)),
            (Some(other), _) => return Err(error(format!("« Module » n'a pas de paramètre « {other} » ; paramètres possibles : name, source, input, output, time, memory"), a.pos)),
            (None, _) => return Err(error(format!("chaque paramètre de « Module » est nommé : {}", example()), a.pos)),
        }
    }
    match (name, source, output) {
        (Some(name), Some(source), Some(output)) => Ok(Module { name, source, entry, output, time, pages }),
        _ => Err(error(format!("« Module » attend name, source et output : {}", example()), block.pos)),
    }
}

/// Le module a rendu son nombre : il va dans sa valeur de sortie, bornée ; puis `Nom.done`.
pub fn finished(program: &Program, state: &crate::state::State, texts: &crate::state::Texts, name: &str, value: u64) -> crate::state::State {
    let numbers = crate::state::initial(program).unwrap_or_default();
    let Some(module) = modules(program, &numbers).ok().and_then(|m| m.into_iter().find(|m| m.name == name)) else { return state.clone() };
    let mut state = state.clone();
    if let Some((_, place)) = state.iter_mut().find(|(known, _)| known == module.output) {
        *place = value.min(crate::state::VALUE_MAX);
    }
    crate::state::arbitrate(program, &state, texts, &format!("{name}.done"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_module_is_declared_and_announced() {
        let source = "module \"somme.wasm\"\nPage(state: State(n: 10, total: 0, ok: 0), modules: [ Module(name: Sum, source: \"somme.wasm\", input: n, output: total, time: 50ms, memory: 1MB) ], children: [ Button(name: Go, text: \"g\") ], rules: [ On(Go.tap, effect: Sum.run), On(Sum.done, effect: ok.set(1)), On(Sum.failed, effect: ok.set(2)) ])";
        crate::check_page(source).unwrap();
        assert_eq!(crate::module_info(source, "n=10;total=0;ok=0", "Sum"), "somme.wasm|10|50|16");
        assert_eq!(crate::module_finished(source, "n=10;total=0;ok=0", "Sum", 55), "n=10;total=55;ok=1");
        for (source, message) in [
            ("Page(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t) ], children: [])", "doit être annoncé en haut du fichier"),
            ("module \"s.wasm\"\nPage(children: [])", "la page ne le déclare pas"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t, time: 9s) ], children: [])", "de 10ms à 5s"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t, memory: 1GB) ], children: [])", "attend une taille"),
            ("module \"s.wasm\"\nPage(state: State(t: \"\"), modules: [ Module(name: S, source: \"s.wasm\", output: t) ], children: [])", "un module reçoit un nombre et rend un nombre"),
            ("module \"../s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"../s.wasm\", output: t) ], children: [])", "rangé à côté de la page"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t) ], children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: S.fly) ])", "un « Module » offre run"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }
}
