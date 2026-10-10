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
//! La boîte : le module n'a accès à rien d'autre que ce que la page lui donne (ni réseau, ni
//! page, ni heure, ni stockage). Il tourne à part, sans jamais bloquer la page ; s'il dépasse son
//! temps (`time`), il est arrêté ; sa mémoire ne peut pas grandir au-delà de son plafond
//! (`memory`). Il se présente en holoscénique : une capacité (`run`) et deux signaux (`done`,
//! `failed`), jamais du Rust. Chaque module est annoncé en haut du fichier, `module "somme.wasm"`,
//! pour que le risque se lise d'un coup d'œil.
//!
//! Deux contrats (ADR-077) :
//!
//! - le premier (ADR-045) : `run(nombre) -> nombre`. Le module reçoit au plus un nombre de la
//!   page (`input: n`) et en rend un (`output: total`) ;
//! - le second : le module reçoit des valeurs de la page (`input: [notes, titre]`) et en rend
//!   (`output: [moyenne, avis]`) : des nombres, à virgule aussi, des textes, des listes. Elles
//!   voyagent en un texte JSON, `{"notes":[…],"titre":"…"}`. Le module offre `alloc(taille)`, qui
//!   dit où écrire ce texte dans sa mémoire, et `run(adresse, taille)`, qui rend l'adresse et la
//!   taille de sa réponse. Le moteur relit la réponse avec méfiance, comme des données venues
//!   d'un serveur : seulement les valeurs annoncées dans `output`, de la bonne sorte, dans leurs
//!   bornes, 64 Ko au plus ; sinon, `failed`.

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
    /// Les valeurs de la page qu'il reçoit (aucune, une ou plusieurs).
    pub inputs: Vec<&'a str>,
    /// Les valeurs de la page qu'il rend (une ou plusieurs).
    pub outputs: Vec<&'a str>,
    pub time: u64,
    pub pages: u64,
}

/// Une réponse de module, au plus (ADR-077) : celle des données d'un serveur.
pub const ANSWER_MAX: usize = crate::state::DATA_BYTES;
/// Les valeurs qu'un module reçoit, ou rend, au plus.
pub const VALUES_MAX: usize = 16;

/// La sorte d'une valeur de la page, pour un module.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sort {
    Number,
    Text,
    List,
}

/// La sorte d'une valeur que la page déclare ; rien pour un nom inconnu. Une liste calculée
/// (ADR-062) est une liste.
pub fn sort_of(program: &Program, name: &str) -> Option<Sort> {
    if crate::state::initial(program).unwrap_or_default().iter().any(|(known, _)| known == name) {
        Some(Sort::Number)
    } else if crate::state::initial_texts(program).iter().any(|(known, _)| known == name) {
        Some(Sort::Text)
    } else if crate::lists::is_list(program, name) {
        Some(Sort::List)
    } else {
        None
    }
}

impl Module<'_> {
    /// Le premier contrat suffit-il : au plus un nombre reçu, et un seul nombre rendu ?
    pub fn simple(&self, program: &Program) -> bool {
        let number = |name: &&str| sort_of(program, name) == Some(Sort::Number);
        self.inputs.len() <= 1 && self.inputs.iter().all(number) && self.outputs.len() == 1 && self.outputs.iter().all(number)
    }
}

fn example() -> &'static str {
    "modules: [ Module(name: Sum, source: \"somme.wasm\", input: n, output: total) ]"
}

/// Les modules de la page, vérifiés.
pub fn modules(program: &Program) -> Result<Vec<Module<'_>>, Error> {
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
    let mut read_ones = Vec::new();
    for value in list {
        let Value::Block(block) = value else {
            return Err(Error { message: format!("« modules » contient des « Module(…) » : {}", example()), pos: argument.pos });
        };
        if block.name != "Module" {
            return Err(Error { message: format!("« modules » contient des « Module(…) », pas des « {} »", block.name), pos: block.pos });
        }
        read_ones.push(a_module(program, block)?);
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

fn a_module<'a>(program: &Program, block: &'a Block) -> Result<Module<'a>, Error> {
    let error = |message: String, pos| Error { message, pos };
    let (mut name, mut source, mut inputs, mut outputs, mut time, mut pages) = (None, None, Vec::new(), None, CURRENT_TIME, CURRENT_PAGES);
    for a in &block.arguments {
        match (a.name.as_deref(), &a.value) {
            (Some("name"), Value::Name(n)) => name = Some(n.as_str()),
            (Some("source"), Value::Text(s)) if s.ends_with(".wasm") && crate::flat::path_on(s) => source = Some(s.as_str()),
            (Some("source"), _) => return Err(error("« Module(source: …) » attend un fichier .wasm rangé à côté de la page, comme \"somme.wasm\"".into(), a.pos)),
            (Some(p @ ("input" | "output")), value) => {
                let names = value_names(program, p, value).map_err(|message| error(message, a.pos))?;
                if p == "input" {
                    inputs = names;
                } else {
                    outputs = Some(names);
                }
            }
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
    match (name, source, outputs) {
        (Some(name), Some(source), Some(outputs)) => Ok(Module { name, source, inputs, outputs, time, pages }),
        _ => Err(error(format!("« Module » attend name, source et output : {}", example()), block.pos)),
    }
}

/// Les valeurs d'`input` ou d'`output` : un nom, ou une liste de noms (ADR-077). Chacune est une
/// valeur que la page déclare ; un module n'écrit ni l'heure ni la date du visiteur, ni une liste
/// calculée.
fn value_names<'a>(program: &Program, parameter: &str, value: &'a Value) -> Result<Vec<&'a str>, String> {
    let names: Vec<&str> = match value {
        Value::Name(name) => vec![name.as_str()],
        Value::List(elements) if !elements.is_empty() => {
            let mut names = Vec::new();
            for element in elements {
                let Value::Name(name) = element else {
                    return Err(format!("« Module({parameter}: […]) » attend des noms de valeurs de la page, comme [notes, titre]"));
                };
                names.push(name.as_str());
            }
            names
        }
        _ => return Err(format!("« Module({parameter}: …) » attend le nom d'une valeur de la page, ou une liste de noms : {parameter}: total, ou {parameter}: [moyenne, avis]")),
    };
    if names.len() > VALUES_MAX {
        return Err(format!("« Module({parameter}: …) » : {VALUES_MAX} valeurs au plus"));
    }
    for (i, name) in names.iter().enumerate() {
        if names[..i].contains(name) {
            return Err(format!("« Module({parameter}: …) » nomme « {name} » deux fois"));
        }
        if sort_of(program, name).is_none() {
            return Err(format!("« Module({parameter}: …) » : « {name} » n'est pas une valeur de la page ; un module reçoit et rend des nombres, des textes ou des listes déclarés dans State"));
        }
        // Le moment présent (ADR-109), donné par le moteur comme la date du jour.
        if parameter == "output" && *name == crate::hours::NOW {
            return Err("« now » est le moment présent, donné par le moteur : un module ne l'écrit pas".into());
        }
        if parameter == "output" && (crate::state::CLOCK.contains(name) || *name == crate::dates::TODAY) {
            return Err(format!("« {name} » est l'heure ou la date du visiteur, donnée par le moteur : un module ne l'écrit pas"));
        }
        if parameter == "output" && crate::computed::is_computed(program, name) {
            return Err(format!("« {name} » est une liste calculée : le moteur la refait, un module ne l'écrit pas"));
        }
    }
    Ok(names)
}

/// Le module du premier contrat a rendu son nombre : il va dans sa valeur de sortie, bornée ;
/// puis `Nom.done`.
pub fn finished(program: &Program, state: &crate::state::State, texts: &crate::state::Texts, name: &str, value: u64) -> crate::state::State {
    let Some(module) = modules(program).ok().and_then(|m| m.into_iter().find(|m| m.name == name)) else { return state.clone() };
    if !module.simple(program) {
        return state.clone();
    }
    let mut state = state.clone();
    if let Some((_, place)) = state.iter_mut().find(|(known, _)| known == module.outputs[0]) {
        *place = value.min(crate::state::VALUE_MAX);
    }
    crate::state::arbitrate(program, &state, texts, &format!("{name}.done"))
}

/// Ce que le module du second contrat reçoit, en un texte JSON (ADR-077) : un nombre à virgule
/// écrit avec ses chiffres (`12.50`), un texte entre guillemets, une liste en tableau ; les champs
/// d'un élément arrivent en texte, comme dans la page.
pub fn input_json(program: &Program, numbers: &crate::state::State, texts: &crate::state::Texts, lists: &crate::lists::Lists, module: &Module) -> String {
    let computed = crate::computed::apply_with_totals(program, numbers, texts, lists).0;
    let mut written = Vec::new();
    for name in &module.inputs {
        let value = if let Some((_, n)) = numbers.iter().find(|(known, _)| known == name) {
            number_json(*n, crate::state::places(program, name))
        } else if let Some((_, t)) = texts.iter().find(|(known, _)| known == name) {
            text_json(t)
        } else if let Some((_, elements)) = lists.iter().chain(computed.iter()).find(|(known, _)| known == name) {
            let elements: Vec<String> = elements
                .iter()
                .map(|element| match crate::lists::fields(element) {
                    fields if element.starts_with(crate::lists::RECORD) => format!("{{{}}}", fields.iter().map(|(k, v)| format!("{}:{}", text_json(k), text_json(v))).collect::<Vec<_>>().join(",")),
                    _ => text_json(element),
                })
                .collect();
            format!("[{}]", elements.join(","))
        } else {
            continue;
        };
        written.push(format!("{}:{value}", text_json(name)));
    }
    format!("{{{}}}", written.join(","))
}

/// Un nombre de la page en JSON : 1250 à deux chiffres après la virgule s'écrit 12.50.
fn number_json(value: u64, places: u32) -> String {
    if places == 0 {
        return value.to_string();
    }
    let scale = crate::state::scale(places);
    format!("{}.{:0width$}", value / scale, value % scale, width = places as usize)
}

/// Un texte en JSON, entre guillemets, ses signes spéciaux échappés.
fn text_json(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// La réponse d'un module du second contrat (ADR-077), relue avec méfiance : un objet JSON de
/// 64 Ko au plus, dont chaque clé est une valeur annoncée dans `output`, de la bonne sorte. Puis
/// elle est rangée comme des données reçues : dans les bornes de chaque valeur.
pub fn received(program: &Program, numbers: &crate::state::State, texts: &crate::state::Texts, lists: &crate::lists::Lists, name: &str, json: &str) -> Result<(crate::state::State, crate::state::Texts, crate::lists::Lists), String> {
    use crate::lists::Json;
    let Some(module) = modules(program).ok().and_then(|m| m.into_iter().find(|m| m.name == name)) else { return Err(format!("aucun module « {name} »")) };
    if json.len() > ANSWER_MAX {
        return Err(format!("une réponse de plus de {} Ko", ANSWER_MAX / 1024));
    }
    let Some(Json::Object(keys)) = Json::read(json) else { return Err("la réponse n'est pas un objet JSON".into()) };
    for (key, value) in &keys {
        if !module.outputs.contains(&key.as_str()) {
            return Err(format!("le module a rendu « {key} », qu'il n'annonce pas dans output"));
        }
        let (fits, expected) = match sort_of(program, key) {
            Some(Sort::Number) => (matches!(value, Json::Number(_)) || (matches!(value, Json::Decimal(_)) && crate::state::places(program, key) > 0), "un nombre positif"),
            Some(Sort::Text) => (matches!(value, Json::Text(_)), "un texte"),
            Some(Sort::List) => (matches!(value, Json::Table(_)), "une liste"),
            None => (false, "une valeur de la page"),
        };
        if !fits {
            return Err(format!("« {key} » attend {expected}"));
        }
    }
    let (numbers, texts) = crate::state::take_values(program, numbers, texts, json);
    let lists = crate::lists::take_lists(program, lists, json);
    Ok((numbers, texts, lists))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_module_is_declared_and_announced() {
        let source = "module \"somme.wasm\"\nPage(state: State(n: 10, total: 0, ok: 0), modules: [ Module(name: Sum, source: \"somme.wasm\", input: n, output: total, time: 50ms, memory: 1MB) ], children: [ Button(name: Go, text: \"g\") ], rules: [ On(Go.tap, effect: Sum.run), On(Sum.done, effect: ok.set(1)), On(Sum.failed, effect: ok.set(2)) ])";
        crate::check_page(source).unwrap();
        assert_eq!(crate::module_info(source, "n=10;total=0;ok=0", "Sum"), "somme.wasm|10|50|16|1");
        assert_eq!(crate::module_finished(source, "n=10;total=0;ok=0", "Sum", 55), "n=10;total=55;ok=1");
        for (source, message) in [
            ("Page(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t) ], children: [])", "doit être annoncé en haut du fichier"),
            ("module \"s.wasm\"\nPage(children: [])", "la page ne le déclare pas"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t, time: 9s) ], children: [])", "de 10ms à 5s"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t, memory: 1GB) ], children: [])", "attend une taille"),
            ("module \"s.wasm\"\nPage(state: State(t: \"\"), modules: [ Module(name: S, source: \"s.wasm\", output: rien) ], children: [])", "« rien » n'est pas une valeur de la page"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: [t, t]) ], children: [])", "nomme « t » deux fois"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: [hour]) ], children: [ P(\"{hour}\") ])", "donnée par le moteur"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: [\"t\"]) ], children: [])", "attend des noms de valeurs"),
            ("module \"../s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"../s.wasm\", output: t) ], children: [])", "rangé à côté de la page"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t) ], children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: S.fly) ])", "un « Module » offre run"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn a_module_receives_and_returns_several_values() {
        let source = "module \"notes.wasm\"\nPage(state: State(notes: [ Item(subject: \"Maths\", mark: \"15.5\") ], title: \"\", average: 0.0, best: \"\", count: 0, ok: 0), modules: [ Module(name: Marks, source: \"notes.wasm\", input: [notes, title, average], output: [average, best, count]) ], children: [ Input(value: title, label: \"Titre\"), Button(name: Go, text: \"g\"), P(\"{average} {best} {count}\") ], rules: [ On(Go.tap, effect: Marks.run), On(Marks.done, effect: ok.set(1)) ])";
        let program = crate::check_page(source).unwrap();
        let start = crate::input(source, &crate::initial_state(source), "title", r#"Trimestre "1" \ fin"#);
        // Le second contrat : pas le premier.
        assert!(crate::module_info(source, &start, "Marks").ends_with("|0"), "{}", crate::module_info(source, &start, "Marks"));
        assert!(!super::modules(&program).unwrap()[0].simple(&program));
        // Ce que le module reçoit : la liste à champs, le texte échappé, le nombre à virgule.
        assert_eq!(crate::module_input(source, &start, "Marks"), r#"{"notes":[{"subject":"Maths","mark":"15.5"}],"title":"Trimestre \"1\" \\ fin","average":0.0}"#);
        // Une bonne réponse : rangée, puis Marks.done.
        let after = crate::module_received(source, &start, "Marks", r#"{"average": 13.25, "best": "Maths", "count": 3}"#).unwrap();
        assert!(after.contains("average=133;") && after.contains("count=3;") && after.contains("ok=1") && after.contains("best='Maths"), "{after}");
        // Ce qui est refusé : une clé non annoncée, une mauvaise sorte, un nombre négatif, pas un objet, trop long.
        for (answer, message) in [
            (r#"{"admin": 1}"#.to_string(), "« admin », qu'il n'annonce pas"),
            (r#"{"count": "trois"}"#.to_string(), "« count » attend un nombre positif"),
            (r#"{"count": -3}"#.to_string(), "« count » attend un nombre positif"),
            (r#"{"best": ["Maths"]}"#.to_string(), "« best » attend un texte"),
            (r#"[1, 2]"#.to_string(), "pas un objet JSON"),
            (format!(r#"{{"best": "{}"}}"#, "x".repeat(super::ANSWER_MAX)), "plus de 64 Ko"),
        ] {
            let error = crate::module_received(source, &start, "Marks", &answer).unwrap_err();
            assert!(error.contains(message), "{answer:.80} → {error}");
        }
        // Une liste rendue : bornée comme des données reçues.
        let source = "module \"tri.wasm\"\nPage(state: State(words: [\"b\", \"a\"]), modules: [ Module(name: Sort, source: \"tri.wasm\", input: words, output: words) ], children: [ Button(name: Go, text: \"g\") ], rules: [ On(Go.tap, effect: Sort.run) ])";
        let after = crate::module_received(source, &crate::initial_state(source), "Sort", r#"{"words": ["a", "b", "c"]}"#).unwrap();
        assert!(after.starts_with("words=[a,b,c]"), "{after}");
    }
}
