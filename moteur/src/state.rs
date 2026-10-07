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

use crate::holo::{Argument, Block, Error, Program, Value};
use crate::rules::for_each_block;

/// Garde-fous du langage : le nombre de valeurs d'une page, et jusqu'où va chacune.
pub const VALUES_MAX: usize = 32;
pub const VALUE_MAX: u64 = 1_000_000_000;

/// Les deux valeurs que le moteur calcule quand la page donne des prix (`prices:`) : le nombre
/// d'articles, et ce qu'ils coûtent ensemble. L'auteur n'écrit aucun calcul.
pub const COMPUTED: &[&str] = &["count", "total"];

/// Les comparaisons d'une condition : `If(count, is: 0)`. Des mots, pas des signes.
pub const COMPARISONS: &[&str] = &["is", "not", "over", "under"];

/// Une condition lue dans un bloc `If` : la valeur regardée, et ce à quoi on la compare.
/// Plusieurs comparaisons valent ensemble : `If(count, over: 0, under: 10)`.
/// Ce à quoi une valeur est comparée, ou ce qu'une demande ajoute : un nombre écrit dans le
/// fichier, ou le nom d'une autre valeur, lue au moment où l'on en a besoin.
/// `If(score, over: 10)`, `When(score, over: best, effect: best.set(score))`.
/// Un texte se compare à un texte écrit entre guillemets : `If(size, is: "M")` (ADR-063).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Term<'a> {
    Number(u64),
    Value(&'a str),
    Text(&'a str),
}

impl Term<'_> {
    /// Ce que vaut ce terme, pour cet état. Une valeur inconnue vaut 0.
    pub fn equals(&self, state: &State) -> u64 {
        match self {
            Term::Number(number) => *number,
            Term::Value(name) => state.iter().find(|(known, _)| known == name).map_or(0, |(_, v)| *v),
            // Un texte ne vaut pas un nombre : il se compare à un texte (`holds`).
            Term::Text(_) => 0,
        }
    }
}

impl std::fmt::Display for Term<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Number(number) => write!(f, "{number}"),
            Term::Value(name) => write!(f, "{name}"),
            // Dans le nom d'une condition, `size|is="M"` : les signes qui séparent ces noms
            // (`;` entre deux réponses, `|` et `=` entre les comparaisons, `"` autour du texte)
            // et les caractères de contrôle s'écrivent `%XX`.
            Term::Text(text) => {
                f.write_str("\"")?;
                for c in text.chars() {
                    if matches!(c, '%' | '"' | ';' | '|' | '=') || c.is_control() {
                        write!(f, "%{:02X}", c as u32)?;
                    } else {
                        write!(f, "{c}")?;
                    }
                }
                f.write_str("\"")
            }
        }
    }
}

pub fn condition(block: &Block) -> Result<(&str, Vec<(&str, Term<'_>)>), Error> {
    // `If` montre ses enfants ; `When` est une règle, elle a un effet.
    let rule = block.name == "When";
    let error = |message: &str| Error { message: message.into(), pos: block.pos };
    let writing = if rule {
        "une règle qui guette s'écrit « When(lives, is: 0, effect: score.set(0)) » ; comparaisons : is (égal), not (différent), over (plus grand), under (plus petit) ; un texte : When(answer, is: \"Paris\", …)"
    } else {
        "une condition s'écrit « If(count, is: 0, children: [ … ]) » ; comparaisons : is (égal), not (différent), over (plus grand), under (plus petit) ; un texte : If(size, is: \"M\", …)"
    };
    let value = match block.arguments.first() {
        Some(Argument { name: None, value: Value::Name(value), .. }) => value.as_str(),
        _ => return Err(error(writing)),
    };
    let mut comparisons = Vec::new();
    for argument in &block.arguments[1..] {
        match (argument.name.as_deref(), &argument.value) {
            (Some("children" | "rules" | "name" | "else"), _) if !rule => {}
            (Some("effect"), _) if rule => {}
            // Un texte se compare à un texte (ADR-063) : If(size, is: "M"), If(buyer, not: "")
            // (rempli). Plus grand, plus petit : seulement des nombres.
            (Some(word @ ("is" | "not")), Value::Text(text)) => comparisons.push((if word == "is" { "is" } else { "not" }, Term::Text(text.as_str()))),
            (Some(word @ ("over" | "under")), Value::Text(_)) => {
                return Err(Error { message: format!("« {word} » compare des nombres ; un texte se compare par is (égal) ou not (différent) : {}({value}, is: \"…\")", block.name), pos: argument.pos })
            }
            (Some(word), Value::Integer(number)) if COMPARISONS.contains(&word) => comparisons.push((COMPARISONS[COMPARISONS.iter().position(|c| *c == word).unwrap_or(0)], Term::Number(*number))),
            // Comparer à une autre valeur : If(score, over: best).
            (Some(word), Value::Name(other)) if COMPARISONS.contains(&word) && is_value_name(other) => {
                comparisons.push((COMPARISONS[COMPARISONS.iter().position(|c| *c == word).unwrap_or(0)], Term::Value(other.as_str())))
            }
            (Some(word), _) if COMPARISONS.contains(&word) => {
                return Err(Error { message: format!("« {}({value}, {word}: …) » attend un nombre entier, un texte entre guillemets, ou le nom d'une autre valeur", block.name), pos: argument.pos })
            }
            (Some(word), _) => {
                return Err(Error {
                    message: format!("« {} » n'a pas de paramètre « {word} » ; paramètres possibles : is, not, over, under, {}", block.name, if rule { "effect" } else { "children" }),
                    pos: argument.pos,
                })
            }
            (None, _) => return Err(error(writing)),
        }
    }
    if comparisons.is_empty() {
        return Err(error(writing));
    }
    // Un `If` montre des blocs (children), ou met des règles sous condition (rules) : l'un ou l'autre.
    let list = |param: &str| matches!(block.argument(param).map(|a| &a.value), Some(Value::List(_)));
    if !rule && (list("children") == list("rules")) {
        return Err(error("« If » attend ce qu'il montre, If(count, is: 0, children: [ … ]), ou les règles qu'il met sous condition, If(lives, over: 0, rules: [ … ])"));
    }
    // Le « sinon » (ADR-039) : ce qu'on montre quand la condition est fausse. Il va avec children.
    if let Some(argument) = block.argument("else").filter(|_| !rule) {
        if !matches!(argument.value, Value::List(_)) || !list("children") {
            return Err(Error { message: "« else » va avec children : il montre des blocs quand la condition est fausse, If(cart, is: 0, children: [ … ], else: [ … ])".into(), pos: argument.pos });
        }
    }
    Ok((value, comparisons))
}

/// Le nom sous lequel une condition est connue de la page : `count|is=0`, `total|over=0|under=300`.
/// Deux conditions écrites pareil portent le même nom, et ont toujours la même réponse.
pub fn key(value: &str, comparisons: &[(&str, Term<'_>)]) -> String {
    comparisons.iter().fold(value.to_string(), |key, (word, number)| format!("{key}|{word}={number}"))
}

/// La condition est-elle vraie pour ces valeurs ? Un texte se compare à des textes, écrits
/// dans le fichier ou d'autres valeurs de texte, à la lettre près (ADR-063) ; un nombre, à des
/// nombres. Une valeur que la page ne connaît pas ne rend rien vrai.
pub fn holds(value: &str, comparisons: &[(&str, Term<'_>)], numbers: &State, texts: &Texts) -> bool {
    let text_of = |name: &str| texts.iter().find(|(known, _)| known == name).map(|(_, text)| text.as_str());
    if let Some(text) = text_of(value) {
        return comparisons.iter().all(|(word, term)| {
            let other = match term {
                Term::Text(other) => Some(*other),
                Term::Value(name) => text_of(name),
                Term::Number(_) => None,
            };
            match (*word, other) {
                ("is", Some(other)) => text == other,
                ("not", Some(other)) => text != other,
                _ => false,
            }
        });
    }
    numbers.iter().find(|(known, _)| known == value).is_some_and(|(_, number)| real_one(comparisons, *number, numbers))
}

/// Toutes les conditions du fichier, avec leur réponse pour ces valeurs. C'est le seul endroit
/// où une condition est décidée : au premier affichage comme après chaque changement.
pub fn conditions(program: &Program, shown: &State, texts: &Texts) -> Vec<(String, bool)> {
    let mut responses: Vec<(String, bool)> = Vec::new();
    let _ = for_each_block(&program.root, &mut |block| {
        if block.name == "If" {
            if let Ok((value, comparisons)) = condition(block) {
                let key = key(value, &comparisons);
                if !responses.iter().any(|(known_one, _)| *known_one == key) {
                    responses.push((key, holds(value, &comparisons, shown, texts)));
                }
            }
        }
        Ok(())
    });
    responses
}

/// La condition est-elle vraie pour cette valeur ?
pub fn real_one(comparisons: &[(&str, Term<'_>)], value: u64, state: &State) -> bool {
    comparisons.iter().all(|(word, term)| {
        let number = term.equals(state);
        match *word {
            "is" => value == number,
            "not" => value != number,
            "over" => value > number,
            _ => value < number,
        }
    })
}

/// Les valeurs que la page garde d'une visite à l'autre : `keep: [cart, best]`.
pub fn kept_values(program: &Program) -> Result<Vec<String>, Error> {
    let Some(argument) = program.root.argument("keep") else { return Ok(Vec::new()) };
    let error = |message: String| Error { message, pos: argument.pos };
    let Value::List(names) = &argument.value else {
        return Err(error("« keep » attend la liste des valeurs à garder : keep: [cart]".into()));
    };
    let declared = initial(program)?;
    let texts = initial_texts(program);
    let mut kept_values = Vec::new();
    for name in names {
        match name {
            Value::Name(name) if CLOCK.contains(&name.as_str()) => return Err(error(format!("« keep » : « {name} » est l'heure du visiteur, elle ne se garde pas"))),
            Value::Name(name) if crate::computed::is_computed(program, name) => return Err(error(format!("« keep » : « {name} » est une liste calculée ; elle se refait d'après sa source, garde plutôt la source"))),
            Value::Name(name) if declared.iter().any(|(known, _)| known == name) || texts.iter().any(|(known, _)| known == name) || crate::lists::is_list(program, name) => kept_values.push(name.clone()),
            Value::Name(name) => return Err(error(format!("« keep » : aucune valeur ne s'appelle « {name} » ; on ne garde que des valeurs déclarées dans « State »"))),
            _ => return Err(error("« keep » attend des noms de valeurs : keep: [cart]".into())),
        }
    }
    Ok(kept_values)
}

/// Jusqu'où une valeur peut monter par la saisie : 1 pour une case à cocher, le `max` d'un champ
/// s'il en a un, sinon la borne du langage.
fn ceiling(program: &Program, name: &str) -> u64 {
    let mut ceiling = VALUE_MAX;
    let _ = for_each_block(&program.root, &mut |block| {
        if matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(value)) if value == name) {
            match (block.name.as_str(), block.argument("max").map(|a| &a.value)) {
                ("Checkbox", _) => ceiling = ceiling.min(1),
                ("Input" | "Slider", Some(Value::Integer(max))) => ceiling = ceiling.min(*max),
                ("Slider", None) => ceiling = ceiling.min(100),
                _ => {}
            }
        }
        // Une valeur qui sert de place sur un plateau reste sur le plateau : de 0 à 100.
        for axis in ["x", "y"] {
            if matches!(block.argument(axis).map(|a| &a.value), Some(Value::Name(value)) if value == name) {
                ceiling = ceiling.min(100);
            }
        }
        Ok(())
    });
    ceiling
}

/// Le plus petit nombre qu'une glissière laisse choisir (ADR-042) ; 0 sinon.
fn floor(program: &Program, name: &str) -> u64 {
    let mut floor = 0;
    let _ = for_each_block(&program.root, &mut |block| {
        if block.name == "Slider" && matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(value)) if value == name) {
            if let Some(Value::Integer(min)) = block.argument("min").map(|a| &a.value) {
                floor = floor.max(*min);
            }
        }
        Ok(())
    });
    floor
}

/// Le visiteur a écrit dans un champ, ou coché une case. C'est encore l'arbitre qui change la
/// valeur : seulement une valeur que la page déclare et qu'un champ présente, et jamais
/// au-delà de son plafond. Un texte qui n'est pas un nombre ne change rien.
pub fn input(program: &Program, state: &State, texts: &Texts, name: &str, written: &str) -> State {
    let before = state.clone();
    let mut state = state.clone();
    let presented = {
        let mut found_one = false;
        let _ = for_each_block(&program.root, &mut |block| {
            found_one |= matches!(block.name.as_str(), "Input" | "Checkbox" | "Slider") && matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(value)) if value == name);
            Ok(())
        });
        found_one
    };
    let written = written.trim();
    let number = if written.is_empty() { Some(0) } else { written.parse::<u64>().ok() };
    let floor = floor(program, name);
    if let (true, Some(number), Some((_, place))) = (presented, number, state.iter_mut().find(|(known, _)| known == name)) {
        *place = number.min(ceiling(program, name)).max(floor);
    }
    suites(program, before, texts, state, texts)
}

/// Reprend les valeurs gardées lors d'une visite précédente. Ce qui est relu vient du
/// navigateur du visiteur, donc on s'en méfie : seules les valeurs que la page dit garder sont
/// reprises, et dans leurs bornes. Tout le reste part de son départ.
pub fn resume(program: &Program, kept: &str) -> State {
    let mut state = initial(program).unwrap_or_default();
    let kept_values = kept_values(program).unwrap_or_default();
    for chunk in kept.split(';') {
        if let Some((name, value)) = chunk.split_once('=') {
            if let (true, Some((_, place)), Ok(value)) = (kept_values.iter().any(|g| g == name), state.iter_mut().find(|(known, _)| known == name), value.parse::<u64>()) {
                *place = value.min(VALUE_MAX);
            }
        }
    }
    state
}

/// Le rythme le plus rapide et le plus lent auquel une page redemande ses données.
pub const DATA_MIN: u64 = 1_000;
pub const DATA_MAX: u64 = 3_600_000;
/// La taille d'un fichier de données, au plus.
pub const DATA_BYTES: usize = 65_536;

/// D'où viennent les données de la page : `data: Data(from: "stock.json", every: 30s)`.
/// Rend le fichier, et le rythme en millisecondes (0 : une seule fois, à l'ouverture).
pub fn data_source(program: &Program) -> Result<Option<(String, u64)>, Error> {
    let Some(argument) = program.root.argument("data") else { return Ok(None) };
    let block = match &argument.value {
        Value::Block(block) if block.name == "Data" && program.root.name == "Page" => block,
        _ => return Err(Error { message: "« data » attend un bloc « Data(...) », sur la page : data: Data(from: \"stock.json\")".into(), pos: argument.pos }),
    };
    let (mut file, mut rhythm) = (None, 0);
    for argument in &block.arguments {
        match (argument.name.as_deref(), &argument.value) {
            // Un fichier rangé à côté de la page : ni adresse complète, ni remontée de dossier.
            // La page ne parle qu'au serveur d'où elle vient.
            (Some("from"), Value::Text(name)) if name.ends_with(".json") && !name.starts_with('/') && !name.contains("..") && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/')) => {
                file = Some(name.clone());
            }
            (Some("from"), _) => return Err(Error { message: "« Data(from: …) » attend un fichier .json rangé à côté de la page, comme \"stock.json\"".into(), pos: argument.pos }),
            (Some("every"), Value::Number { value, unit: Some(unit) }) if unit == "s" || unit == "ms" => {
                let ms = if unit == "s" { value * 1000.0 } else { *value };
                if !(DATA_MIN as f64..=DATA_MAX as f64).contains(&ms) {
                    return Err(Error { message: "« Data(every: …) » va de 1s à 3600s".into(), pos: argument.pos });
                }
                rhythm = ms.round() as u64;
            }
            (Some("every"), _) => return Err(Error { message: "« Data(every: …) » attend une durée, de 1s à 3600s".into(), pos: argument.pos }),
            (Some(word), _) => return Err(Error { message: format!("« Data » n'a pas de paramètre « {word} » ; paramètres possibles : from, every"), pos: argument.pos }),
            (None, _) => return Err(Error { message: "chaque paramètre de « Data » est nommé : Data(from: \"stock.json\")".into(), pos: argument.pos }),
        }
    }
    match file {
        Some(file) => Ok(Some((file, rhythm))),
        None => Err(Error { message: "« Data » attend « from » : Data(from: \"stock.json\")".into(), pos: block.pos }),
    }
}

/// Ce qu'on lit dans un fichier de données : un nombre entier, ou un texte.
#[derive(Debug, PartialEq)]
pub enum Datum {
    Number(u64),
    Text(String),
}

/// Lit un fichier de données : un objet JSON à plat, `{"stock": 4, "message": "Ouvert"}`.
/// Sont repris : les nombres entiers positifs, les textes, et `true`/`false` (1 et 0). Tout le
/// reste (nombres à virgule ou négatifs, listes, objets emboîtés, `null`) est laissé de côté.
/// Un fichier mal formé ne donne rien du tout.
pub fn read_data(json: &str) -> Vec<(String, Datum)> {
    fn blanks(t: &[char], i: &mut usize) {
        while *i < t.len() && t[*i].is_whitespace() {
            *i += 1;
        }
    }
    fn text(t: &[char], i: &mut usize) -> Option<String> {
        if t.get(*i) != Some(&'"') {
            return None;
        }
        *i += 1;
        let mut output = String::new();
        loop {
            let c = *t.get(*i)?;
            *i += 1;
            match c {
                '"' => return Some(output),
                '\\' => {
                    let e = *t.get(*i)?;
                    *i += 1;
                    match e {
                        'n' => output.push('\n'),
                        't' => output.push(' '),
                        'u' => {
                            let hex: String = t.get(*i..*i + 4)?.iter().collect();
                            *i += 4;
                            output.push(char::from_u32(u32::from_str_radix(&hex, 16).ok()?).unwrap_or('?'));
                        }
                        other => output.push(other),
                    }
                }
                other => output.push(other),
            }
        }
    }
    // Saute une valeur qu'on ne reprend pas : une liste, un objet, un nombre à virgule, null.
    fn skip(t: &[char], i: &mut usize) -> Option<()> {
        let mut depth = 0usize;
        loop {
            match *t.get(*i)? {
                '"' => {
                    text(t, i)?;
                    continue;
                }
                '[' | '{' => depth += 1,
                ']' | '}' if depth > 0 => depth -= 1,
                ',' | '}' if depth == 0 => return Some(()),
                _ => {}
            }
            *i += 1;
        }
    }
    let read = || -> Option<Vec<(String, Datum)>> {
        if json.len() > DATA_BYTES {
            return None;
        }
        let t: Vec<char> = json.chars().collect();
        let mut i = 0;
        let mut data = Vec::new();
        blanks(&t, &mut i);
        if t.get(i) != Some(&'{') {
            return None;
        }
        i += 1;
        loop {
            blanks(&t, &mut i);
            if t.get(i) == Some(&'}') {
                return Some(data);
            }
            let key = text(&t, &mut i)?;
            blanks(&t, &mut i);
            if t.get(i) != Some(&':') {
                return None;
            }
            i += 1;
            blanks(&t, &mut i);
            let start = i;
            match *t.get(i)? {
                '"' => data.push((key, Datum::Text(text(&t, &mut i)?))),
                c if c.is_ascii_digit() => {
                    while t.get(i).is_some_and(|c| c.is_ascii_digit()) {
                        i += 1;
                    }
                    // Un nombre à virgule ou avec exposant n'est pas repris.
                    if t.get(i).is_some_and(|c| matches!(c, '.' | 'e' | 'E')) {
                        skip(&t, &mut i)?;
                    } else if let Ok(number) = t[start..i].iter().collect::<String>().parse::<u64>() {
                        data.push((key, Datum::Number(number)));
                    }
                }
                _ => {
                    let word: String = t[i..].iter().take(5).collect();
                    if word.starts_with("true") {
                        data.push((key, Datum::Number(1)));
                    } else if word.starts_with("false") {
                        data.push((key, Datum::Number(0)));
                    }
                    skip(&t, &mut i)?;
                }
            }
            blanks(&t, &mut i);
            match *t.get(i)? {
                ',' => i += 1,
                '}' => return Some(data),
                _ => return None,
            }
        }
    };
    read().unwrap_or_default()
}

/// Les données viennent d'arriver. C'est l'arbitre qui les range : seulement dans des valeurs
/// que la page déclare, de la bonne sorte (un nombre dans un nombre, un texte dans un texte),
/// et dans leurs bornes. Puis les règles qui guettent ont leur mot à dire.
pub fn receive(program: &Program, state: &State, texts: &Texts, json: &str) -> (State, Texts) {
    let (before, texts_before) = (state.clone(), texts.clone());
    let (mut state, mut texts) = (state.clone(), texts.clone());
    if data_source(program).ok().flatten().is_none() {
        return (state, texts);
    }
    for (key, datum) in read_data(json) {
        match datum {
            Datum::Number(number) => {
                let ceiling = ceiling(program, &key);
                if let Some((_, place)) = state.iter_mut().find(|(known, _)| *known == key && known != DRAWS && !CLOCK.contains(&known.as_str())) {
                    *place = number.min(ceiling);
                }
            }
            Datum::Text(text) => {
                if let Some((_, place)) = texts.iter_mut().find(|(known, _)| *known == key) {
                    *place = clean(&text, TEXT_MAX);
                }
            }
        }
    }
    (suites(program, before, &texts_before, state, &texts), texts)
}

thread_local! {
    /// Les capacités demandées par les règles de temps et les règles qui guettent pendant un
    /// appel à l'arbitre (`Ding.play`). La page les lit après l'appel, pour faire entendre le son.
    static CAPABILITIES: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Vide la liste des capacités demandées, et la rend.
pub fn requested_capabilities() -> Vec<String> {
    CAPABILITIES.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Note les capacités qu'une règle demande, à côté de ses demandes : `effect: [score.add(1), Ding.play]`.
fn record_capabilities(rule: &Block) {
    let names: Vec<&String> = match rule.argument("effect").map(|a| &a.value) {
        Some(Value::Name(name)) => vec![name],
        Some(Value::List(elements)) => elements.iter().filter_map(|e| match e {
            Value::Name(name) => Some(name),
            _ => None,
        }).collect(),
        _ => Vec::new(),
    };
    CAPABILITIES.with(|c| c.borrow_mut().extend(names.into_iter().cloned()));
}

/// Parcourt tous les blocs en disant, pour chacun, s'il est en vigueur pour cet état. Des règles
/// rangées dans `If(lives, over: 0, rules: [ … ])` ne valent que tant que la condition est vraie.
fn with_their_force<'a>(program: &'a Program, state: &State, texts: &Texts, f: &mut dyn FnMut(&'a Block, bool)) {
    fn visit<'a>(value: &'a Value, in_force: bool, shown: &State, texts: &Texts, f: &mut dyn FnMut(&'a Block, bool)) {
        match value {
            Value::List(elements) => elements.iter().for_each(|e| visit(e, in_force, shown, texts, f)),
            Value::Block(block) => {
                f(block, in_force);
                let under_condition = block.name == "If" && block.argument("rules").is_some();
                let inside = in_force
                    && (!under_condition
                        || condition(block).is_ok_and(|(value, comparisons)| holds(value, &comparisons, shown, texts)));
                for argument in &block.arguments {
                    let here = if under_condition && argument.name.as_deref() == Some("rules") { inside } else { in_force };
                    visit(&argument.value, here, shown, texts, f);
                }
            }
            _ => {}
        }
    }
    let shown = to_show(program, state);
    f(&program.root, true);
    for argument in &program.root.arguments {
        visit(&argument.value, true, &shown, texts, f);
    }
}

/// Cette règle est-elle en vigueur, pour cet état ?
fn in_force(program: &Program, rule: &Block, state: &State, texts: &Texts) -> bool {
    let mut response = true;
    with_their_force(program, state, texts, &mut |block, force| {
        if std::ptr::eq(block, rule) {
            response = force;
        }
    });
    response
}

/// Les demandes d'une règle : une seule (`effect: cart.add(1)`), ou plusieurs entre crochets
/// (`effect: [score.set(0), lives.set(3)]`), faites dans l'ordre où elles sont écrites.
pub fn requests_of(rule: &Block) -> Vec<&Block> {
    match rule.argument("effect").map(|a| &a.value) {
        Some(Value::Block(request)) if is_requested(request) => vec![request],
        Some(Value::List(elements)) => elements.iter().filter_map(|e| match e {
            Value::Block(request) if is_requested(request) => Some(request),
            _ => None,
        }).collect(),
        _ => Vec::new(),
    }
}

/// Ce qu'on peut demander pour une valeur.
pub const REQUESTS: &[&str] = &["add", "sub", "set", "random", "mul", "div"];

/// Sous ce nom, l'état garde le nombre de tirages au hasard déjà faits. Ce n'est pas une valeur
/// de l'auteur (le nom n'est pas un nom permis) : il sert à ce que le hasard soit rejouable.
const DRAWS: &str = "~";

/// Le rythme le plus rapide et le plus lent d'une règle `Every`, en millisecondes.
pub const RHYTHM_MIN: u64 = 100;
pub const RHYTHM_MAX: u64 = 3_600_000;

/// Le rythme d'une règle `Every(1s, effect: …)`, en millisecondes.
pub fn rhythm(rule: &Block) -> Result<u64, Error> {
    let error = || Error {
        message: if rule.name == "After" {
            "une attente s'écrit « After(3s, effect: shown.set(1)) » : une durée en s ou en ms, de 100ms à 3600s".into()
        } else {
            "une règle de temps s'écrit « Every(1s, effect: time.sub(1)) » : une durée en s ou en ms, de 100ms à 3600s".into()
        },
        pos: rule.pos,
    };
    let milliseconds = match rule.arguments.first() {
        Some(Argument { name: None, value: Value::Number { value, unit: Some(unit) }, .. }) if unit == "s" => value * 1000.0,
        Some(Argument { name: None, value: Value::Number { value, unit: Some(unit) }, .. }) if unit == "ms" => *value,
        _ => return Err(error()),
    };
    if !(RHYTHM_MIN as f64..=RHYTHM_MAX as f64).contains(&milliseconds) {
        return Err(error());
    }
    Ok(milliseconds.round() as u64)
}

/// Les horloges du fichier : une par règle `Every`, dans l'ordre où elles sont écrites, avec
/// son rythme et la valeur qu'elle fait changer. Chaque règle a son horloge, pour qu'on puisse
/// en relancer une sans toucher aux autres.
pub fn clocks(program: &Program) -> Vec<(u64, String)> {
    let mut clocks = Vec::new();
    let _ = for_each_block(&program.root, &mut |block| {
        if block.name == "Every" {
            let values: Vec<&str> = requests_of(block).iter().filter_map(|d| d.name.split('.').next()).collect();
            clocks.push((rhythm(block).unwrap_or(RHYTHM_MAX), values.join(",")));
        }
        Ok(())
    });
    clocks
}

/// Les attentes du fichier, une par règle `After`, dans l'ordre où elles sont écrites : sa durée,
/// et si elle court pour cet état (ADR-039). Une attente posée dans la page court dès
/// l'ouverture ; une attente rangée sous une condition, `If(toast, is: 1, rules: [ After(3s, …) ])`,
/// court à partir du moment où la condition devient vraie, et s'arrête si elle redevient fausse.
/// Chacune ne sonne qu'une fois par période où elle court : c'est la page qui tient le compte.
pub fn delays(program: &Program, state: &State, texts: &Texts) -> Vec<(u64, bool)> {
    let mut delays = Vec::new();
    with_their_force(program, state, texts, &mut |block, force| {
        if block.name == "After" {
            delays.push((rhythm(block).unwrap_or(RHYTHM_MAX), force));
        }
    });
    delays
}

/// Les valeurs qu'un signal fait changer, d'après les règles `On`. Quand un geste change une
/// valeur, l'horloge qui s'occupe de cette valeur repart de zéro : « Play » remet le temps à
/// trente secondes, et la première seconde dure une vraie seconde.
pub fn touched_ones(program: &Program, signal: &str) -> Vec<String> {
    let mut values = Vec::new();
    let _ = for_each_block(&program.root, &mut |block| {
        let trigger = block.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value);
        if let ("On", Some(Value::Name(s))) = (block.name.as_str(), trigger) {
            if s == signal {
                for effect in requests_of(block) {
                    if let Some((value, _)) = effect.name.split_once('.') {
                        if !values.iter().any(|v| v == value) {
                            values.push(value.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    });
    values
}

/// La graine du hasard d'un fichier : tirée du nom de sa page. Le même fichier, avec les mêmes
/// gestes aux mêmes moments, redonne la même partie (ADR-008).
fn random_seed(program: &Program) -> u64 {
    let name = crate::rules::name_of(&program.root).unwrap_or("Home");
    name.bytes().fold(0x4A5E_u64, |g, byte| crate::seed::mix_bits(g ^ u64::from(byte)))
}

/// Les valeurs d'une page, dans l'ordre où elles sont déclarées.
pub type State = Vec<(String, u64)>;

/// Les valeurs de texte d'une page : `State(buyer: "")`. Elles ne changent que par un champ
/// (`Input`) ; on les montre (`{buyer}`), et l'on peut demander si elles sont vides.
pub type Texts = Vec<(String, String)>;

/// La longueur d'un texte, au plus. Un champ peut l'abaisser par `max:`.
pub const TEXT_MAX: usize = 2000;
/// La longueur d'un texte saisi quand le champ ne dit rien.
pub const TEXT_SHORT: usize = 80;
/// La longueur d'un texte long (`Input(lines:)`) quand l'auteur n'a pas dit `max`.
pub const TEXT_LONG: usize = 1000;

/// Les textes déclarés par la page, à leur départ.
pub fn initial_texts(program: &Program) -> Texts {
    let Ok(Some(block)) = state_block(program) else { return Vec::new() };
    block.arguments
        .iter()
        .filter_map(|a| match (&a.name, &a.value) {
            (Some(name), Value::Text(text)) => Some((name.clone(), text.clone())),
            _ => None,
        })
        .collect()
}

/// Les champs montrés dans un texte : `{item.title}`, `{item.price:cents}` → (title, None), (price, Some(cents)).
pub fn shown_fields(text: &str) -> Vec<(String, Option<String>)> {
    let mut fields = Vec::new();
    let mut remainder = text;
    while let Some(start) = remainder.find("{item.") {
        let after = &remainder[start + 6..];
        let Some(end) = after.find('}') else { break };
        let inside = &after[..end];
        let (name, format) = inside.split_once(':').map_or((inside, None), |(n, f)| (n, Some(f.to_string())));
        fields.push((name.to_string(), format));
        remainder = &after[end + 1..];
    }
    fields
}

/// Le texte sans ses `{item}` ni ses `{item.…}`, pour vérifier le reste.
fn without_fields(text: &str) -> String {
    let mut output = String::new();
    let mut remainder = text;
    while let Some(start) = remainder.find("{item") {
        output.push_str(&remainder[..start]);
        let after = &remainder[start..];
        let Some(end) = after.find('}') else {
            output.push_str(after);
            return output;
        };
        let inside = &after[1..end];
        if inside != "item" && !inside.starts_with("item.") {
            output.push_str(&after[..=end]);
        }
        remainder = &after[end + 1..];
    }
    output.push_str(remainder);
    output
}

/// Un texte, écrit pour voyager dans l'état sans se mêler à ses séparateurs : tout ce qui
/// n'est pas une lettre ou un chiffre ordinaire devient « %XX ».
pub fn encode(text: &str) -> String {
    text.bytes().map(|byte| if byte.is_ascii_alphanumeric() { char::from(byte).to_string() } else { format!("%{byte:02X}") }).collect()
}

/// L'inverse. Rend `None` si ce n'est pas un texte codé par `coder`.
pub fn decode(code: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(code.len());
    let mut remainder = code.as_bytes();
    while let Some((first, suite)) = remainder.split_first() {
        if *first == b'%' {
            let hex = std::str::from_utf8(suite.get(..2)?).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            remainder = &suite[2..];
        } else {
            bytes.push(*first);
            remainder = suite;
        }
    }
    String::from_utf8(bytes).ok()
}

/// `buyer='Ada;city='` : les textes, à la suite des nombres. L'apostrophe dit « c'est un texte ».
pub fn write_texts(texts: &Texts) -> String {
    texts.iter().map(|(name, text)| format!("{name}='{}", encode(text))).collect::<Vec<_>>().join(";")
}

/// Relit les textes d'un état. Seuls ceux que la page déclare sont repris, et bornés.
pub fn reread_texts(program: &Program, written: &str) -> Texts {
    let mut texts = initial_texts(program);
    for chunk in written.split(';') {
        if let Some((name, code)) = chunk.split_once("='") {
            if let (Some((_, place)), Some(text)) = (texts.iter_mut().find(|(known, _)| known == name), decode(code)) {
                // Les retours à la ligne d'un texte long (Input(lines:)) sont gardés ; les autres
                // caractères invisibles, non.
                *place = clean_multiline(&text, TEXT_MAX);
            }
        }
    }
    texts
}

/// Un texte saisi, nettoyé : sans caractère invisible, et pas plus long que permis.
fn clean(text: &str, length: usize) -> String {
    text.chars().filter(|c| !c.is_control()).take(length).collect()
}

/// Un texte long (`Input(lines:)`) garde ses retours à la ligne.
fn clean_multiline(text: &str, length: usize) -> String {
    text.replace("\r\n", "\n").chars().filter(|c| *c == '\n' || !c.is_control()).take(length).collect()
}

/// Les options d'un `Choice`, telles qu'écrites.
pub fn choice_options(block: &Block) -> Vec<&str> {
    match block.argument("options").map(|a| &a.value) {
        Some(Value::List(options)) => options.iter().filter_map(|o| match o {
            Value::Text(t) => Some(t.as_str()),
            _ => None,
        }).collect(),
        _ => Vec::new(),
    }
}

/// Le visiteur a écrit dans un champ de texte. Comme pour un nombre : seulement une valeur
/// qu'un champ présente, et pas plus longue que ce champ ne le permet.
pub fn input_text(program: &Program, texts: &Texts, name: &str, written: &str) -> Texts {
    let mut texts = texts.clone();
    let mut length = None;
    let mut lines = false;
    let mut choice: Option<Vec<String>> = None;
    let mut kind: Option<String> = None;
    let _ = for_each_block(&program.root, &mut |block| {
        let presents = matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(value)) if value == name);
        if let (true, true, Some(Value::Name(t))) = (block.name == "Input", presents, block.argument("type").map(|a| &a.value)) {
            kind = Some(t.clone());
        }
        if block.name == "Input" && presents {
            lines = block.argument("lines").is_some();
            length = Some(match block.argument("max").map(|a| &a.value) {
                Some(Value::Integer(max)) => (*max as usize).min(TEXT_MAX),
                _ if lines => TEXT_LONG,
                _ => TEXT_SHORT,
            });
        }
        // Un choix n'accepte que l'une de ses options (ou rien).
        if block.name == "Choice" && presents {
            choice = Some(choice_options(block).into_iter().map(str::to_string).collect());
        }
        Ok(())
    });
    // Une date, une heure, une couleur (ADR-042) : seulement ce que le navigateur sait écrire.
    // Un fichier (ADR-059) : on garde son nom seul, sans le dossier, et pas trop long.
    if kind.as_deref() == Some("file") {
        let single = written.rsplit(['/', '\\']).next().unwrap_or_default();
        if let Some((_, place)) = texts.iter_mut().find(|(known, _)| known == name) {
            *place = clean(single, 120);
        }
        return texts;
    }
    if let Some(kind) = kind {
        let digits = |t: &str, template: &str| t.len() == template.len() && t.chars().zip(template.chars()).all(|(c, g)| if g == '9' { c.is_ascii_digit() } else { c == g });
        let correct = written.is_empty()
            || match kind.as_str() {
                "date" => digits(written, "9999-99-99"),
                "time" => digits(written, "99:99"),
                _ => written.len() == 7 && written.starts_with('#') && written[1..].chars().all(|c| c.is_ascii_hexdigit()),
            };
        if let (true, Some((_, place))) = (correct, texts.iter_mut().find(|(known, _)| known == name)) {
            *place = written.to_ascii_lowercase();
        }
        return texts;
    }
    if let Some(options) = choice {
        if let Some((_, place)) = texts.iter_mut().find(|(known, _)| known == name) {
            if written.is_empty() || options.iter().any(|o| o == written) {
                *place = written.to_string();
            }
        }
        return texts;
    }
    if let (Some(length), Some((_, place))) = (length, texts.iter_mut().find(|(known, _)| known == name)) {
        *place = if lines { clean_multiline(written, length) } else { clean(written, length) };
    }
    texts
}

/// Ce qu'un formulaire envoie (ADR-042) : les valeurs que présentent ses champs, en JSON,
/// `{"form":"Contact","values":{"name":"Ada","size":"M","quantity":2}}`. `None` si aucun
/// formulaire ne porte ce nom.
pub fn submission(program: &Program, state: &State, texts: &Texts, form_name: &str) -> Option<String> {
    let form = crate::rules::named_block(program, form_name).filter(|b| b.name == "Form")?;
    let mut names: Vec<&str> = Vec::new();
    let _ = for_each_block(form, &mut |block| {
        if matches!(block.name.as_str(), "Input" | "Checkbox" | "Choice" | "Slider") {
            if let Some(Value::Name(value)) = block.argument("value").map(|a| &a.value) {
                if !names.contains(&value.as_str()) {
                    names.push(value);
                }
            }
        }
        Ok(())
    });
    let json = |text: &str| {
        let mut output = String::from("\"");
        for c in text.chars() {
            match c {
                '"' => output.push_str("\\\""),
                '\\' => output.push_str("\\\\"),
                '\n' => output.push_str("\\n"),
                c if (c as u32) < 0x20 => output.push_str(&format!("\\u{:04x}", c as u32)),
                c => output.push(c),
            }
        }
        output.push('"');
        output
    };
    let values: Vec<String> = names
        .iter()
        .filter_map(|name| match (state.iter().find(|(c, _)| c == name), texts.iter().find(|(c, _)| c == name)) {
            (Some((_, n)), _) => Some(format!("{}:{n}", json(name))),
            (_, Some((_, t))) => Some(format!("{}:{}", json(name), json(t))),
            _ => None,
        })
        .collect();
    Some(format!("{{\"form\":{},\"values\":{{{}}}}}", json(form_name), values.join(",")))
}

/// Pour les conditions, un texte vaut 0 quand il est vide, 1 sinon : `If(buyer, not: "")`.
pub fn with_texts(shown: &State, texts: &Texts) -> State {
    let mut all = shown.clone();
    all.extend(texts.iter().map(|(name, text)| (name.clone(), u64::from(!text.is_empty()))));
    all
}

/// `cart`, `items_seen` : une minuscule, puis des minuscules, des chiffres ou `_`.
/// Un nom de valeur s'écrit comme en Flutter : une minuscule au début, puis lettres et chiffres,
/// les mots joints par une majuscule (`appleX`, `blueDoor`), sans `_` (ADR-037).
fn is_value_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase()) && name.chars().all(|c| c.is_ascii_alphanumeric())
}

/// `appleX` → `appleX` ; `topRight` → `topRight`. Pour dire le bon mot à qui a écrit l'autre.
pub fn in_flutter(name: &str) -> String {
    let mut output = String::with_capacity(name.len());
    let mut uppercase = false;
    for c in name.chars() {
        if c == '_' {
            uppercase = !output.is_empty();
        } else if uppercase {
            output.push(c.to_ascii_uppercase());
            uppercase = false;
        } else {
            output.push(c);
        }
    }
    output
}

/// Le message pour un nom de valeur mal écrit.
fn badly_written_value_name(name: &str) -> String {
    if name.contains('_') {
        format!("« {name} » : deux mots se joignent comme en Flutter, par une majuscule ; écris « {} » (ADR-037)", in_flutter(name))
    } else {
        format!("« {name} » : le nom d'une valeur commence par une minuscule, comme « cart » ou « appleX » (ADR-037)")
    }
}

/// Le bloc `State(...)` donné à la page par `state:`.
fn state_block(program: &Program) -> Result<Option<&Block>, Error> {
    match program.root.argument("state") {
        None => Ok(None),
        Some(argument) => match &argument.value {
            Value::Block(block) if block.name == "State" && program.root.name == "Page" => Ok(Some(block)),
            _ => Err(Error { message: "« state » attend un bloc « State(...) », sur la page : state: State(cart: 0)".into(), pos: argument.pos }),
        },
    }
}

/// Les prix donnés par la page : `prices: Prices(sunrise: 120)`. Chaque prix porte le nom
/// d'une valeur déclarée, qui est la quantité de cet article.
pub fn price(program: &Program) -> Result<State, Error> {
    let Some(argument) = program.root.argument("prices") else { return Ok(Vec::new()) };
    let block = match &argument.value {
        Value::Block(block) if block.name == "Prices" && program.root.name == "Page" => block,
        _ => return Err(Error { message: "« prices » attend un bloc « Prices(...) », sur la page : prices: Prices(sunrise: 120)".into(), pos: argument.pos }),
    };
    let quantities = initial_without_prices(program)?;
    let mut price = State::new();
    for argument in &block.arguments {
        let (Some(name), Value::Integer(amount)) = (&argument.name, &argument.value) else {
            return Err(Error { message: "un prix se donne par le nom de l'article et un nombre entier : Prices(sunrise: 120)".into(), pos: argument.pos });
        };
        if !quantities.iter().any(|(known, _)| known == name) {
            return Err(Error { message: format!("le prix « {name} » ne correspond à aucune valeur : déclare sa quantité, state: State({name}: 0)"), pos: argument.pos });
        }
        if price.iter().any(|(known, _)| known == name) {
            return Err(Error { message: format!("le prix « {name} » est donné deux fois"), pos: argument.pos });
        }
        if *amount > VALUE_MAX {
            return Err(Error { message: format!("« {name} » : un prix va de 0 à {VALUE_MAX}"), pos: argument.pos });
        }
        price.push((name.clone(), *amount));
    }
    Ok(price)
}

/// Ce que le moteur calcule à partir des quantités et des prix : `count` (le nombre
/// d'articles) et `total` (ce qu'ils coûtent). Rien si la page ne donne pas de prix.
pub fn computed(program: &Program, state: &State) -> State {
    let price = price(program).unwrap_or_default();
    if program.root.argument("prices").is_none() {
        return Vec::new();
    }
    let (mut number, mut total) = (0u128, 0u128);
    for (name, amount) in &price {
        let quantity = state.iter().find(|(known, _)| known == name).map_or(0, |(_, q)| u128::from(*q));
        number += quantity;
        total += quantity * u128::from(*amount);
    }
    // Un total ne déborde jamais : au pire, il s'arrête au plus grand nombre que l'on sait écrire.
    let clamp = |v: u128| u64::try_from(v).unwrap_or(u64::MAX);
    vec![("count".to_string(), clamp(number)), ("total".to_string(), clamp(total))]
}

/// Tout ce qu'un texte peut montrer : les valeurs de la page, puis celles que le moteur calcule.
pub fn to_show(program: &Program, state: &State) -> State {
    let mut all = state.clone();
    all.extend(computed(program, state));
    all
}

/// Les valeurs déclarées par la page, avec leur départ.
pub fn initial(program: &Program) -> Result<State, Error> {
    let state = initial_without_prices(program)?;
    // Avec des prix, « count » et « total » sont calculés par le moteur : on ne les déclare pas.
    if program.root.argument("prices").is_some() {
        if let Some((name, _)) = state.iter().find(|(name, _)| COMPUTED.contains(&name.as_str())) {
            let pos = program.root.argument("state").map_or(program.root.pos, |a| a.pos);
            return Err(Error { message: format!("« {name} » est calculé par le moteur quand la page donne des prix : ne le déclare pas dans « State »"), pos });
        }
    }
    // L'heure du visiteur, quand le fichier la lit (ADR-039).
    let mut state = state;
    let now = now();
    for name in clock_read(program) {
        let rank = CLOCK.iter().position(|h| *h == name).unwrap_or(0);
        state.push((name.to_string(), now[rank]));
    }
    Ok(state)
}

/// L'heure du visiteur, que le moteur donne comme il donne `count` et `total` (ADR-039) :
/// l'année, le mois (1 à 12), le jour (1 à 31), le jour de la semaine (1 lundi, 7 dimanche),
/// l'heure (0 à 23) et la minute. On la lit, on ne la change pas.
pub const CLOCK: &[&str] = &["year", "month", "day", "weekday", "hour", "minute"];

thread_local! {
    /// L'heure donnée par celui qui appelle le moteur : la page (l'heure de l'appareil du
    /// visiteur), ou le serveur qui fabrique la page d'avance.
    static NOW: std::cell::Cell<[u64; 6]> = const { std::cell::Cell::new([2026, 1, 1, 4, 0, 0]) };
}

/// Donne l'heure au moteur : année, mois, jour, jour de la semaine, heure, minute.
pub fn set_now(values: [u64; 6]) {
    NOW.with(|m| m.set(values));
}

pub fn now() -> [u64; 6] {
    NOW.with(std::cell::Cell::get)
}

/// L'heure en temps universel, d'après les secondes écoulées depuis le 1er janvier 1970.
/// Sert quand personne n'a donné l'heure du lieu.
pub fn from_unix_seconds(seconds: u64) -> [u64; 6] {
    let days = (seconds / 86_400) as i64;
    let remainder = seconds % 86_400;
    let z = days + 719_468;
    let ere = z.div_euclid(146_097);
    let day_of_era = z - ere * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let m = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * m + 2) / 5 + 1;
    let month = if m < 10 { m + 3 } else { m - 9 };
    let year = year_of_era + ere * 400 + i64::from(month <= 2);
    // Le 1er janvier 1970 était un jeudi : le quatrième jour de la semaine.
    let week = (days + 3).rem_euclid(7) + 1;
    [year as u64, month as u64, day as u64, week as u64, remainder / 3600, remainder % 3600 / 60]
}

/// Les noms de l'heure que le fichier lit : dans un texte (`{hour}`), une condition, une
/// comparaison, une demande (`best.set(hour)`) ou une place.
fn clock_read(program: &Program) -> Vec<&'static str> {
    fn record(name: &str, read_ones: &mut Vec<&'static str>) {
        if let Some(h) = CLOCK.iter().find(|h| **h == name) {
            if !read_ones.contains(h) {
                read_ones.push(h);
            }
        }
    }
    fn visit(value: &Value, read_ones: &mut Vec<&'static str>) {
        match value {
            Value::Text(text) => names_in(text).into_iter().for_each(|name| record(name, read_ones)),
            Value::Name(name) => record(name, read_ones),
            Value::List(elements) => elements.iter().for_each(|e| visit(e, read_ones)),
            Value::Block(block) => block.arguments.iter().for_each(|a| visit(&a.value, read_ones)),
            _ => {}
        }
    }
    let mut read_ones = Vec::new();
    program.root.arguments.iter().for_each(|a| visit(&a.value, &mut read_ones));
    read_ones.sort_by_key(|h| CLOCK.iter().position(|x| x == h));
    read_ones
}

/// Le fichier lit-il l'heure ? La page la tient alors à jour, minute après minute.
pub fn reads_time(program: &Program) -> bool {
    !clock_read(program).is_empty()
}

/// Une minute a passé : l'heure écrite dans l'état devient l'heure donnée au moteur, et les
/// règles qui guettent l'heure (`When(hour, is: 12, …)`) ont leur mot à dire.
pub fn advance_clock(program: &Program, written: &str) -> State {
    let now = reread(program, written);
    let texts = reread_texts(program, written);
    let mut before = now.clone();
    for chunk in written.split(';') {
        if let Some((name, value)) = chunk.split_once('=') {
            if let (true, Some((_, place)), Ok(value)) = (CLOCK.contains(&name), before.iter_mut().find(|(known, _)| known == name), value.parse::<u64>()) {
                *place = value;
            }
        }
    }
    suites(program, before, &texts, now, &texts)
}

fn initial_without_prices(program: &Program) -> Result<State, Error> {
    let Some(block) = state_block(program)? else { return Ok(Vec::new()) };
    let mut state = State::new();
    for argument in &block.arguments {
        // Une valeur est un nombre entier (cart: 0) ou un texte (buyer: "").
        let (name, start_value) = match (&argument.name, &argument.value) {
            (Some(name), Value::Integer(start_value)) => (name, Some(*start_value)),
            (Some(name), Value::Text(text)) if text.chars().count() <= TEXT_MAX => (name, None),
            (Some(name), Value::Text(_)) => return Err(Error { message: format!("« {name} » : un texte fait au plus {TEXT_MAX} caractères"), pos: argument.pos }),
            // Une liste de textes (ADR-044) : State(tasks: []).
            (Some(name), Value::List(_)) => {
                crate::lists::check_declaration(argument)?;
                (name, None)
            }
            _ => {
                return Err(Error {
                    message: "une valeur se déclare par son nom et son départ, un nombre entier, un texte ou une liste : State(cart: 0, buyer: \"\", tasks: [])".into(),
                    pos: argument.pos,
                })
            }
        };
        if !is_value_name(name) {
            return Err(Error { message: badly_written_value_name(name), pos: argument.pos });
        }
        if CLOCK.contains(&name.as_str()) {
            return Err(Error { message: format!("« {name} » est l'heure du visiteur, donnée par le moteur ; choisis un autre nom pour ta valeur (ADR-039)"), pos: argument.pos });
        }
        if block.arguments.iter().filter(|a| a.name.as_deref() == Some(name.as_str())).count() > 1 {
            return Err(Error { message: format!("la valeur « {name} » est déclarée deux fois"), pos: argument.pos });
        }
        match start_value {
            Some(start_value) if start_value > VALUE_MAX => return Err(Error { message: format!("« {name} » : une valeur va de 0 à {VALUE_MAX}"), pos: argument.pos }),
            Some(start_value) => state.push((name.clone(), start_value)),
            None => {}
        }
    }
    if block.arguments.len() > VALUES_MAX {
        return Err(Error { message: format!("trop de valeurs : une page en déclare au plus {VALUES_MAX}"), pos: block.pos });
    }
    Ok(state)
}

/// Une demande faite à l'arbitre : `cart.add(1)`.
#[derive(Debug, PartialEq)]
pub struct Request<'a> {
    pub value: &'a str,
    pub verb: &'a str,
    pub quantity: u64,
    /// Quand la quantité est une autre valeur : `best.set(score)`. Elle est lue au moment où
    /// la demande est faite.
    pub since: Option<&'a str>,
}

/// Un bloc est-il une demande ? Une demande commence par une minuscule : `cart.add(1)`.
pub fn is_requested(block: &Block) -> bool {
    block.name.starts_with(|c: char| c.is_ascii_lowercase())
}

/// Lit et vérifie une demande, d'après les valeurs déclarées.
pub fn request<'a>(block: &'a Block, state: &State) -> Result<Request<'a>, Error> {
    let error = |message: String| Error { message, pos: block.pos };
    let Some((value, verb)) = block.name.split_once('.') else {
        return Err(error(format!("« {} » : une demande s'écrit « cart.add(1) »", block.name)));
    };
    if CLOCK.contains(&value) {
        return Err(error(format!("« {value} » est l'heure du visiteur : on la lit, on ne la change pas")));
    }
    if !state.iter().any(|(known, _)| known == value) {
        return Err(error(format!("aucune valeur ne s'appelle « {value} » : déclare-la sur la page, state: State({value}: 0)")));
    }
    if !REQUESTS.contains(&verb) {
        return Err(error(format!("demande inconnue « {verb} » : pour une valeur, on peut demander {}", REQUESTS.join(", "))));
    }
    match block.arguments.as_slice() {
        // La quantité peut être une autre valeur de la page : best.set(score).
        [Argument { name: None, value: Value::Name(other), .. }] if state.iter().any(|(known, _)| known == other) => Ok(Request { value, verb, quantity: 0, since: Some(other.as_str()) }),
        [Argument { name: None, value: Value::Name(other), .. }] => Err(error(format!("« {value}.{verb}({other}) » : aucun nombre ne s'appelle « {other} » ; déclare-le sur la page, state: State({other}: 0)"))),
        [argument] if argument.name.is_none() => match argument.value {
            // « random(0) » ne tirerait jamais que 0 : c'est sûrement une erreur.
            Value::Integer(0) if verb == "div" => Err(error(format!("« {value}.div(0) » : on ne divise pas par 0"))),
            Value::Integer(0) if verb == "random" => Err(error(format!("« {value}.random » attend le plus grand nombre possible, au moins 1 : {value}.random(100) tire de 0 à 100"))),
            Value::Integer(quantity) if quantity <= VALUE_MAX => Ok(Request { value, verb, quantity, since: None }),
            _ => Err(error(format!("« {value}.{verb} » attend un nombre entier de 0 à {VALUE_MAX} : {value}.{verb}(1)"))),
        },
        _ => Err(error(format!("« {value}.{verb} » attend un seul nombre : {value}.{verb}(1)"))),
    }
}

/// Les noms entre accolades d'un texte : `{cart}` dans « {cart} paintings ».
pub fn names_in(text: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut remainder = text;
    while let Some(start) = remainder.find('{') {
        remainder = &remainder[start + 1..];
        if let Some(end) = remainder.find('}') {
            // `{minute:00}` : la valeur `minute`, montrée avec un format (ADR-043).
            let name = remainder[..end].split_once(':').map_or(&remainder[..end], |(name, _)| name);
            if name.starts_with(|c: char| c.is_ascii_lowercase()) && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                names.push(name);
            }
        }
    }
    names
}

/// Vérifie l'état : la déclaration, sa place, et chaque `{nom}` écrit dans un texte.
/// Les demandes des règles sont vérifiées avec les règles (`regles.rs`).
pub fn check_state(program: &Program) -> Result<State, Error> {
    let state = initial(program)?;
    price(program)?;
    kept_values(program)?;
    data_source(program)?;
    // Ce qu'un texte peut montrer : les valeurs déclarées, nombres et textes, et celles que le
    // moteur calcule.
    let texts = initial_texts(program);
    let mut showable = with_texts(&to_show(program, &state), &texts);
    showable.extend(crate::lists::counts(&crate::lists::initial(program)));
    // Le nombre d'éléments d'une liste calculée se montre aussi : « {found} résultat(s) ».
    showable.extend(crate::computed::names(program).into_iter().map(|name| (name, 0)));
    let models = crate::lists::models_and_lists(program);
    let is_text = |name: &str| texts.iter().any(|(known, _)| known == name);
    let declare = state_block(program)?;
    let declared_prices = match program.root.argument("prices").map(|a| &a.value) {
        Some(Value::Block(block)) => Some(block),
        _ => None,
    };
    for_each_block(&program.root, &mut |block| {
        if block.name == "State" && !declare.is_some_and(|d| std::ptr::eq(d, block)) {
            return Err(Error { message: "« State » se déclare une seule fois, sur la page : state: State(cart: 0)".into(), pos: block.pos });
        }
        if (block.name == "Prices" && !declared_prices.is_some_and(|d| std::ptr::eq(d, block))) || (block.argument("prices").is_some() && !std::ptr::eq(block, &program.root)) {
            return Err(Error { message: "« Prices » se donne une seule fois, sur la page : prices: Prices(sunrise: 120)".into(), pos: block.pos });
        }
        if block.argument("state").is_some() && !std::ptr::eq(block, &program.root) {
            return Err(Error { message: "les valeurs se déclarent sur la page, pas dans un monde : elles valent pour tout le fichier".into(), pos: block.pos });
        }
        // Un champ ou une case présente une valeur déclarée par la page, et dit ce qu'il attend.
        // Un choix présente un texte, et ses options sont des textes (ADR-038).
        if block.name == "Choice" {
            let example = "Choice(value: size, label: \"Size\", options: [\"S\", \"M\", \"L\"])";
            for argument in &block.arguments {
                match (argument.name.as_deref(), &argument.value) {
                    (Some("name"), _) | (Some("label"), Value::Text(_)) | (Some("menu"), Value::Bool(_)) => {}
                    (Some("value"), Value::Name(value)) if is_text(value) => {}
                    (Some("value"), Value::Name(value)) => {
                        return Err(Error { message: format!("« Choice(value: {value}) » : un choix présente un texte ; déclare-le ainsi : state: State({value}: \"\")"), pos: argument.pos })
                    }
                    (Some("options"), Value::List(options)) if (2..=20).contains(&options.len()) && options.iter().all(|o| matches!(o, Value::Text(t) if !t.is_empty() && t.chars().count() <= 80)) => {}
                    (Some("options"), _) => return Err(Error { message: "« Choice(options: …) » attend de 2 à 20 textes entre guillemets : options: [\"S\", \"M\", \"L\"]".into(), pos: argument.pos }),
                    (Some(word), _) => return Err(Error { message: format!("« Choice({word}: …) » est mal écrit : {example}"), pos: argument.pos }),
                    (None, _) => return Err(Error { message: format!("chaque paramètre de « Choice » est nommé : {example}"), pos: argument.pos }),
                }
            }
            for required in ["value", "label", "options"] {
                if block.argument(required).is_none() {
                    return Err(Error { message: format!("« Choice » attend « {required} » : {example}"), pos: block.pos });
                }
            }
            let options = choice_options(block);
            if options.iter().enumerate().any(|(i, o)| options[..i].contains(o)) {
                return Err(Error { message: "« Choice » : deux options ont le même texte".into(), pos: block.pos });
            }
        }
        if block.name == "Input" || block.name == "Checkbox" {
            let allowed: &[&str] = if block.name == "Input" { &["name", "value", "label", "max", "lines", "type", "accept"] } else { &["name", "value", "label"] };
            let example = if block.name == "Input" { "Input(value: quantity, label: \"How many?\")" } else { "Checkbox(value: gift, label: \"Gift wrap\")" };
            for argument in &block.arguments {
                match (argument.name.as_deref(), &argument.value) {
                    (Some("name"), _) | (Some("label"), Value::Text(_)) => {}
                    (Some("value"), Value::Name(value)) if CLOCK.contains(&value.as_str()) => {
                        return Err(Error { message: format!("« {}(value: {value}) » : « {value} » est l'heure du visiteur ; on la lit, on ne l'écrit pas", block.name), pos: argument.pos })
                    }
                    (Some("value"), Value::Name(value)) if crate::lists::is_list(program, value) => {
                        return Err(Error { message: format!("« {}(value: {value}) » : « {value} » est une liste ; un champ présente un texte qu'on ajoute ensuite, {value}.push(task)", block.name), pos: argument.pos })
                    }
                    (Some("value"), Value::Name(value)) if state.iter().any(|(known, _)| known == value) => {}
                    // Un champ peut présenter un texte ; une case, non.
                    (Some("value"), Value::Name(value)) if is_text(value) && block.name == "Input" => {}
                    (Some("value"), Value::Name(value)) if is_text(value) => {
                        return Err(Error { message: format!("« Checkbox(value: {value}) » : « {value} » est un texte ; une case attend un nombre, state: State(gift: 0)"), pos: argument.pos })
                    }
                    (Some("value"), Value::Name(value)) => {
                        return Err(Error { message: format!("« {}(value: {value}) » : aucune valeur ne s'appelle « {value} » ; déclare-la sur la page, state: State({value}: 0)", block.name), pos: argument.pos })
                    }
                    // Un fichier (ADR-059) : sa taille et ses sortes sont vérifiées dans fichiers.rs.
                    (Some("max" | "accept"), _) if crate::files::is_file(block) => {}
                    (Some("accept"), _) => return Err(Error { message: "« accept: » ne sert qu'à un champ de fichier, Input(type: file, …)".into(), pos: argument.pos }),
                    (Some("max"), Value::Integer(max)) if block.name == "Input" && *max <= VALUE_MAX => {}
                    // Un texte long : de 2 à 20 lignes visibles, pour une valeur qui est un texte.
                    (Some("lines"), Value::Integer(n)) if block.name == "Input" && (2..=20).contains(n) => {
                        if !matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(v)) if is_text(v)) {
                            return Err(Error { message: "« Input(lines: …) » écrit un texte long : sa valeur est un texte, state: State(message: \"\")".into(), pos: argument.pos });
                        }
                    }
                    (Some("lines"), _) => return Err(Error { message: "« Input(lines: …) » attend un nombre de lignes, de 2 à 20".into(), pos: argument.pos }),
                    // Une date, une heure, une couleur (ADR-042) : la valeur est un texte.
                    (Some("type"), Value::Name(t)) if block.name == "Input" && ["date", "time", "color", "file"].contains(&t.as_str()) => {
                        if !matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(v)) if is_text(v)) {
                            return Err(Error { message: format!("« Input(type: {t}) » écrit un texte : sa valeur se déclare ainsi, state: State(arrivee: \"\")"), pos: argument.pos });
                        }
                    }
                    (Some("type"), _) => return Err(Error { message: "« Input(type: …) » attend date, time, color ou file ; un nombre ou un texte se devinent tout seuls".into(), pos: argument.pos }),
                    // `grow:` range le bloc dans Row ou Column (ADR-052) ; sa place est vérifiée ailleurs.
                    (Some("grow"), _) => {}
                    (Some(word), _) if allowed.contains(&word) => return Err(Error { message: format!("« {}({word}: …) » est mal écrit : {example}", block.name), pos: argument.pos }),
                    (Some(word), _) => return Err(Error { message: format!("« {} » n'a pas de paramètre « {word} » ; paramètres possibles : {}", block.name, allowed.join(", ")), pos: argument.pos }),
                    (None, _) => return Err(Error { message: format!("chaque paramètre de « {} » est nommé : {example}", block.name), pos: argument.pos }),
                }
            }
            // Un champ sans étiquette est un champ qu'un lecteur d'écran ne sait pas nommer.
            for required in ["value", "label"] {
                if block.argument(required).is_none() {
                    return Err(Error { message: format!("« {} » attend « {required} » : {example}", block.name), pos: block.pos });
                }
            }
        }
        // Une glissière présente un nombre de la page ; une barre de progression le montre (ADR-042).
        if block.name == "Slider" || block.name == "Progress" {
            let example = if block.name == "Slider" { "Slider(value: volume, label: \"Volume\", min: 0, max: 100)" } else { "Progress(value: lives, max: 3, label: \"Lives\")" };
            for argument in &block.arguments {
                match (argument.name.as_deref(), &argument.value) {
                    (Some("name"), _) | (Some("label"), Value::Text(_)) => {}
                    (Some("value"), Value::Name(value)) if CLOCK.contains(&value.as_str()) && block.name == "Slider" => {
                        return Err(Error { message: format!("« Slider(value: {value}) » : « {value} » est l'heure du visiteur ; on la lit, on ne l'écrit pas"), pos: argument.pos })
                    }
                    (Some("value"), Value::Name(value)) if to_show(program, &state).iter().any(|(known, _)| known == value) => {}
                    (Some("value"), Value::Integer(_)) if block.name == "Progress" => {}
                    (Some("value"), _) => return Err(Error { message: format!("« {}(value: …) » présente un nombre de la page : déclare-le, state: State(volume: 50) ; {example}", block.name), pos: argument.pos }),
                    (Some("min"), Value::Integer(_)) if block.name == "Slider" => {}
                    (Some("max"), Value::Integer(max)) if (1..=VALUE_MAX).contains(max) => {}
                    (Some(word @ ("min" | "max")), _) => return Err(Error { message: format!("« {}({word}: …) » attend un nombre entier, de 1 à {VALUE_MAX} pour max", block.name), pos: argument.pos }),
                    (Some(word), _) => return Err(Error { message: format!("« {}({word}: …) » est mal écrit : {example}", block.name), pos: argument.pos }),
                    (None, _) => return Err(Error { message: format!("chaque paramètre de « {} » est nommé : {example}", block.name), pos: argument.pos }),
                }
            }
            for required in ["value", "label"] {
                if block.argument(required).is_none() {
                    return Err(Error { message: format!("« {} » attend « {required} » : {example}", block.name), pos: block.pos });
                }
            }
            let integer = |p: &str| match block.argument(p).map(|a| &a.value) {
                Some(Value::Integer(n)) => Some(*n),
                _ => None,
            };
            if block.name == "Slider" && integer("min").unwrap_or(0) >= integer("max").unwrap_or(100) {
                return Err(Error { message: "« Slider » : min doit être plus petit que max".into(), pos: block.pos });
            }
        }
        // Une règle qui guette regarde une valeur : un nombre déclaré ou calculé, ou un texte
        // (ADR-063).
        if block.name == "When" && block.argument("meets").is_none() {
            let (value, _) = condition(block)?;
            if !to_show(program, &state).iter().any(|(known, _)| known == value) && !is_text(value) {
                return Err(Error { message: format!("« When({value}, …) » : aucune valeur ne s'appelle « {value} » ; déclare-la sur la page, state: State({value}: 0)"), pos: block.pos });
            }
        }
        // Une condition qui regarde l'élément d'une ligne (ADR-057) : vérifiée à part.
        if crate::lists::element_subject(block).is_some() {
            let list = models.iter().find(|(b, _)| std::ptr::eq(*b, block)).map(|(_, l)| l.as_str());
            crate::lists::check_element_if(block, program, list)?;
        } else if block.name == "If" {
            let (value, _) = condition(block)?;
            if !showable.iter().any(|(known, _)| known == value) {
                return Err(Error { message: format!("« If({value}, …) » : aucune valeur ne s'appelle « {value} » ; déclare-la sur la page, state: State({value}: 0)"), pos: block.pos });
            }
        }
        // Comparer sans mélange (ADR-063) : un texte à un texte (entre guillemets, ou une autre
        // valeur de texte), un nombre à un nombre (écrit, ou une autre valeur de nombre).
        let compares = (block.name == "If" && crate::lists::element_subject(block).is_none()) || (block.name == "When" && block.argument("meets").is_none());
        if let (true, Some(Argument { value: Value::Name(value), .. })) = (compares, block.arguments.first()) {
            let numbers = to_show(program, &state);
            let is_number = |name: &str| numbers.iter().any(|(known, _)| known == name);
            let sort = |name: &str| if is_text(name) { "un texte" } else { "un nombre" };
            for argument in &block.arguments[1..] {
                let Some(word) = argument.name.as_deref().filter(|word| COMPARISONS.contains(word)) else { continue };
                let error = |message: String| Err(Error { message, pos: argument.pos });
                match &argument.value {
                    Value::Text(_) if !is_text(value) => return error(format!("« {value} » est un nombre : on le compare à un nombre, {}({value}, {word}: 0)", block.name)),
                    Value::Integer(_) if is_text(value) => {
                        return error(format!("« {value} » est un texte : on le compare à un texte entre guillemets, {}({value}, is: \"…\"), ou à une autre valeur de texte", block.name))
                    }
                    Value::Name(other) if !is_text(other) && !is_number(other) => {
                        return error(if is_text(value) {
                            format!("« {word}: {other} » : aucun texte ne s'appelle « {other} » ; déclare-le sur la page, state: State({other}: \"\")")
                        } else {
                            format!("« {word}: {other} » : aucun nombre ne s'appelle « {other} » ; déclare-le sur la page, state: State({other}: 0)")
                        })
                    }
                    Value::Name(other) if is_text(other) != is_text(value) => {
                        return error(format!("« {value} » est {} et « {other} » {} : on compare deux nombres, ou deux textes", sort(value), sort(other)))
                    }
                    Value::Name(_) if is_text(value) && matches!(word, "over" | "under") => {
                        return error(format!("« {word} » compare des nombres ; un texte se compare par is (égal) ou not (différent) : {}({value}, is: \"…\")", block.name))
                    }
                    _ => {}
                }
            }
        }
        // Sur un plateau, la place d'un bloc est une valeur de la page : Point(x: starX, y: starY).
        for axis in ["x", "y"] {
            if let Some(Argument { value: Value::Name(value), pos, .. }) = block.argument(axis) {
                if !showable.iter().any(|(known, _)| known == value) {
                    return Err(Error { message: format!("« {axis}: {value} » : aucune valeur ne s'appelle « {value} » ; déclare-la sur la page, state: State({value}: 50)"), pos: *pos });
                }
            }
        }
        let model_list = models.iter().find(|(b, _)| std::ptr::eq(*b, block)).map(|(_, l)| l.as_str());
        let in_model = model_list.is_some();
        let check_text = |text: &str, pos| {
            for (name, format) in crate::format::formats_in(text) {
                if !crate::format::is_format(format) {
                    return Err(Error { message: format!("« {{{name}:{format}}} » : format inconnu ; formats possibles : 00 (zéros devant), number (1 234), cents (12,50), name (le nom du jour ou du mois)"), pos });
                }
                if format == "name" && name != "weekday" && name != "month" {
                    return Err(Error { message: format!("« {{{name}:name}} » : seuls weekday et month ont un nom (mardi, octobre)"), pos });
                }
                if is_text(name) {
                    return Err(Error { message: format!("« {{{name}:{format}}} » : « {name} » est un texte ; un format s'applique à un nombre"), pos });
                }
            }
            // Dans les lignes d'une liste à champs, `{item.title}` montre un champ (ADR-051).
            if let Some(list) = model_list {
                for (field, format) in shown_fields(text) {
                    match crate::lists::kind(program, list) {
                        Some(crate::lists::Kind::Texts) | None => {
                            return Err(Error { message: format!("dans les lignes de « {list} », « {{item}} » est le texte de l'élément ; il n'a pas de champs : pour des champs, déclare la liste avec des Item(…), State({list}: [ Item(title: \"…\") ])"), pos });
                        }
                        Some(crate::lists::Kind::Records(fields)) if !fields.contains(&field) => {
                            return Err(Error { message: format!("les éléments de « {list} » n'ont pas de champ « {field} » ; champs : {}", fields.join(", ")), pos });
                        }
                        _ => {}
                    }
                    if let Some(f) = format {
                        if !crate::format::is_format(&f) || f == "name" {
                            return Err(Error { message: format!("« {{item.{field}:{f}}} » : format inconnu ; pour un champ : 00, number, cents"), pos });
                        }
                    }
                }
            }
            let text = if in_model { without_fields(text) } else { text.to_string() };
            let text = text.as_str();
            if text.contains("{item.") || text.contains("{item}") {
                return Err(Error { message: "« {item…} » montre un champ de l'élément : il n'a de sens que dans une répétition, Repeat(items: [ … ], children: [ … ])".into(), pos });
            }
            names_in(text).into_iter().find(|name| !showable.iter().any(|(known, _)| known == name)).map_or(Ok(()), |name| {
                if name.contains('_') {
                    return Err(Error { message: format!("« {{{name}}} » : deux mots se joignent comme en Flutter ; écris « {{{}}} » (ADR-037)", in_flutter(name)), pos });
                }
                Err(Error { message: format!("« {{{name}}} » : aucune valeur ne s'appelle « {name} » ; déclare-la sur la page, state: State({name}: 0)"), pos })
            })
        };
        for argument in &block.arguments {
            match (&argument.name.as_deref(), &argument.value) {
                (None | Some("text"), Value::Text(text)) => check_text(text, argument.pos)?,
                // Les phrases seules et les lignes d'une liste.
                (Some("children" | "else"), Value::List(elements)) => {
                    for element in elements {
                        if let Value::Text(text) = element {
                            check_text(text, argument.pos)?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    })?;
    Ok(state)
}

/// L'arbitre : ce que deviennent les valeurs quand ce signal est émis. Les demandes sont
/// appliquées dans l'ordre où les règles sont écrites. Une valeur ne descend pas sous 0 et ne
/// dépasse pas `VALEUR_MAX` : elle s'arrête à la borne.
pub fn arbitrate(program: &Program, state: &State, texts: &Texts, signal: &str) -> State {
    let before_signal = state.clone();
    let mut state = state.clone();
    let seed = random_seed(program);
    let mut draws = state.iter().find(|(name, _)| name == DRAWS).map_or(0, |(_, n)| *n);
    let start_value = draws;
    let mut clock = 0usize;
    let mut waiting = 0usize;
    // Les règles rangées sous une condition ne répondent que si elle est vraie au moment du signal.
    with_their_force(program, &before_signal, texts, &mut |block, force| {
        // Une règle répond à un signal : celui d'un bloc (`On(Add.tap, …)`), ou celui de sa
        // propre horloge (`Every(1s, …)` : « every:0 » pour la première règle de temps du
        // fichier, « every:1 » pour la deuxième).
        let concerned = match block.name.as_str() {
            "On" => matches!(block.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value), Some(Value::Name(s)) if s == signal),
            "Every" => {
                clock += 1;
                signal == format!("every:{}", clock - 1)
            }
            // Une attente qui vient de finir : « after:0 » pour la première du fichier.
            "After" => {
                waiting += 1;
                signal == format!("after:{}", waiting - 1)
            }
            _ => false,
        };
        if concerned && force {
            for effect in requests_of(block) {
                apply(program, &mut state, effect, seed, &mut draws);
            }
            // Les capacités d'une règle « On » sont appliquées par la page (elle connaît le
            // geste) ; celles d'une règle de temps sont notées ici.
            if block.name == "Every" || block.name == "After" {
                record_capabilities(block);
            }
        }
    });
    watch(program, before_signal, texts, &mut state, texts, seed, &mut draws);
    record_draws(&mut state, start_value, draws);
    state
}

/// Ce qui suit un changement fait sans signal (une saisie, un glissement, des données) : les
/// règles qui guettent ont leur mot à dire, comme après un geste. Les textes d'avant et
/// d'après comptent aussi : `When(answer, is: "Paris", …)` (ADR-063).
fn suites(program: &Program, before: State, texts_before: &Texts, mut state: State, texts: &Texts) -> State {
    let seed = random_seed(program);
    let start_value = state.iter().find(|(name, _)| name == DRAWS).map_or(0, |(_, n)| *n);
    let mut draws = start_value;
    watch(program, before, texts_before, &mut state, texts, seed, &mut draws);
    record_draws(&mut state, start_value, draws);
    state
}

/// Des textes ont changé, par un geste ou une saisie : les règles qui guettent un texte, ou
/// qui sont rangées sous une condition sur un texte, ont leur mot à dire (ADR-063).
pub fn after_texts(program: &Program, state: State, before: &Texts, texts: &Texts) -> State {
    if before == texts {
        return state;
    }
    suites(program, state.clone(), before, state, texts)
}

fn record_draws(state: &mut State, start_value: u64, draws: u64) {
    if draws != start_value {
        match state.iter_mut().find(|(name, _)| name == DRAWS) {
            Some((_, n)) => *n = draws,
            None => state.push((DRAWS.to_string(), draws)),
        }
    }
}

/// Le visiteur fait glisser un bloc posé sur un plateau (`drag: true`). Ses places, si ce sont
/// des valeurs de la page, suivent le doigt. C'est encore l'arbitre qui change les valeurs :
/// seulement pour un bloc qui se laisse glisser, et sans sortir du plateau.
pub fn drag(program: &Program, state: &State, texts: &Texts, name: &str, x: u64, y: u64) -> State {
    let before = state.clone();
    let mut state = state.clone();
    let draggable = crate::rules::named_block(program, name).filter(|block| matches!(block.argument("drag").map(|a| &a.value), Some(Value::Bool(true))));
    if let Some(block) = draggable {
        for (axis, place) in [("x", x), ("y", y)] {
            if let Some(Value::Name(value)) = block.argument(axis).map(|a| &a.value).filter(|v| !matches!(v, Value::Name(n) if CLOCK.contains(&n.as_str()))) {
                if let Some((_, v)) = state.iter_mut().find(|(known, _)| known == value) {
                    *v = place.min(100);
                }
            }
        }
    }
    suites(program, before, texts, state, texts)
}

/// Les règles qui guettent (`When`) : chacune se déclenche au moment où ce qu'elle guette
/// devient vrai, pas tant qu'il le reste.
fn watch(program: &Program, before_change: State, texts_before: &Texts, state: &mut State, texts: &Texts, seed: u64, draws: &mut u64) {
    let start_state = before_change;
    // Les effets d'une règle qui guette ne changent que des nombres : après la première photo,
    // les textes d'avant sont ceux d'après.
    let mut texts_before = texts_before;
    {
    // Toutes celles qui se déclenchent en même temps jugent sur la même photo de l'état, puis
    // leurs effets s'appliquent dans l'ordre. Un effet peut en déclencher d'autres : on
    // recommence, huit fois au plus, pour qu'un fichier mal écrit ne tourne pas sans fin.
    let mut before = start_state;
    for _ in 0..8 {
        let photo = state.clone();
        let mut triggered = false;
        let _ = for_each_block(&program.root, &mut |block| {
            let seen = |state: &State, texts: &Texts| in_force(program, block, state, texts) && watches(program, block, state, texts);
            if block.name == "When" && !seen(&before, texts_before) && seen(&photo, texts) {
                for effect in requests_of(block) {
                    apply(program, state, effect, seed, draws);
                    triggered = true;
                }
                record_capabilities(block);
            }
            Ok(())
        });
        if !triggered {
            break;
        }
        before = photo;
        texts_before = texts;
    }
    }
}

/// Fait ce qu'une demande demande : `cart.add(1)`. Une valeur ne descend pas sous 0 et ne
/// dépasse pas son plafond.
fn apply(program: &Program, state: &mut State, effect: &Block, seed: u64, draws: &mut u64) {
    let Ok(d) = request(effect, state) else { return };
    let quantity = d.since.map_or(d.quantity, |other| state.iter().find(|(known, _)| known == other).map_or(0, |(_, v)| *v));
    let ceiling = ceiling(program, d.value);
    if let Some((_, value)) = state.iter_mut().find(|(name, _)| name == d.value) {
        *value = match d.verb {
            "add" => value.saturating_add(quantity),
            "sub" => value.saturating_sub(quantity),
            // Multiplier, diviser (ADR-043) : des nombres entiers ; la division arrondit vers le bas,
            // et une division par une valeur qui vaut 0 ne change rien.
            "mul" => value.saturating_mul(quantity),
            "div" if quantity == 0 => *value,
            "div" => *value / quantity,
            // Le hasard n'en est pas un : c'est le énième tirage d'une suite fixée par la graine
            // du fichier. Rejouer les mêmes gestes redonne les mêmes nombres.
            "random" => {
                *draws = draws.wrapping_add(1);
                crate::seed::mix_bits(seed ^ crate::seed::mix_bits(*draws)) % (quantity + 1)
            }
            _ => quantity,
        }
        .min(ceiling);
    }
}

/// La distance, sur un plateau, en deçà de laquelle deux blocs se rencontrent, quand la règle
/// ne le dit pas. Les places vont de 0 à 100.
pub const MEETING: u64 = 10;

/// Les deux blocs d'une rencontre, `When(Basket, meets: Apple, within: 9, …)`, et la distance.
pub fn meets(rule: &Block) -> Result<(&str, &str, u64), Error> {
    let writing = "une rencontre s'écrit « When(Basket, meets: Apple, effect: score.add(1)) » : les noms de deux blocs posés sur un plateau";
    let (Some(Argument { name: None, value: Value::Name(a), .. }), Some(Value::Name(b))) = (rule.arguments.first(), rule.argument("meets").map(|a| &a.value)) else {
        return Err(Error { message: writing.into(), pos: rule.pos });
    };
    let mut distance = MEETING;
    for argument in &rule.arguments[1..] {
        match (argument.name.as_deref(), &argument.value) {
            (Some("meets" | "effect"), _) => {}
            (Some("within"), Value::Integer(n)) if (1..=100).contains(n) => distance = *n,
            (Some("within"), _) => return Err(Error { message: "« When(…, within: …) » attend un nombre entier de 1 à 100 : la distance de rencontre, sur un plateau qui va de 0 à 100".into(), pos: argument.pos }),
            (Some(word), _) => return Err(Error { message: format!("une rencontre n'a pas de paramètre « {word} » ; paramètres possibles : meets, within, effect"), pos: argument.pos }),
            (None, _) => return Err(Error { message: writing.into(), pos: argument.pos }),
        }
    }
    Ok((a, b, distance))
}

/// Où est un bloc sur son plateau, d'après ses réglages `x` et `y` : un nombre écrit, ou une
/// valeur de la page.
fn place_of(program: &Program, state: &State, name: &str) -> Option<(u64, u64)> {
    let block = crate::rules::named_block(program, name)?;
    let read = |axis: &str| match &block.argument(axis)?.value {
        Value::Integer(n) => Some((*n).min(100)),
        Value::Name(value) => state.iter().find(|(known, _)| known == value).map(|(_, v)| (*v).min(100)),
        _ => None,
    };
    Some((read("x")?, read("y")?))
}

/// La largeur d'un plateau, dans ses propres unités ; sa hauteur est `Board(height:)`. Le plateau
/// garde ses proportions à l'écran : les rencontres sont les mêmes sur tous les écrans.
pub const BOARD_WIDTH: f64 = 640.0;

/// Un objet posé sur un plateau : son centre et son encombrement, dans les unités du plateau.
struct Body {
    cx: f64,
    cy: f64,
    half: f64,
    circle: bool,
}

/// Où est un bloc, et quelle place il prend, sur son plateau.
fn body(program: &Program, state: &State, name: &str) -> Option<Body> {
    let block = crate::rules::named_block(program, name)?;
    let (x, y) = place_of(program, state, name)?;
    let pixels = |block: &Block, param: &str| match block.argument(param).map(|a| &a.value) {
        Some(Value::Number { value, unit: Some(unit) }) if unit == "px" => Some(*value),
        _ => None,
    };
    // La taille du bloc, et la part de cette taille qu'on voit vraiment.
    let (size, half, circle) = match (block.name.as_str(), block.argument("form").map(|a| &a.value)) {
        ("Shape", Some(Value::Name(shape))) => {
            let size = pixels(block, "size").unwrap_or(48.0);
            match shape.as_str() {
                "square" => (size, size / 2.0, false),
                "circle" => (size, size / 2.0, true),
                // Un triangle et un losange ne remplissent pas leur carré : un rond un peu plus petit.
                _ => (size, size * 0.4, true),
            }
        }
        // Un point est une lumière qui s'éteint vers le bord : son cœur fait un tiers de sa taille.
        ("Point", _) => (64.0, 64.0 * 0.35, true),
        _ => (48.0, 24.0, false),
    };
    // La hauteur du plateau où il est posé.
    let mut height = 320.0;
    let _ = for_each_block(&program.root, &mut |board| {
        if board.name == "Board" {
            if let Some(Value::List(children)) = board.argument("children").map(|a| &a.value) {
                if children.iter().any(|e| matches!(e, Value::Block(b) if crate::rules::name_of(b) == Some(name))) {
                    height = pixels(board, "height").unwrap_or(320.0);
                }
            }
        }
        Ok(())
    });
    let width = BOARD_WIDTH;
    // À 0 le bloc touche un bord, à 100 l'autre : son centre parcourt le plateau moins sa taille.
    Some(Body { cx: x as f64 / 100.0 * (width - size) + size / 2.0, cy: y as f64 / 100.0 * (height - size) + size / 2.0, half, circle })
}

/// Deux objets se touchent-ils ? Le bord de l'un atteint le bord de l'autre.
fn touch_each_other(a: &Body, b: &Body) -> bool {
    let (dx, dy) = ((a.cx - b.cx).abs(), (a.cy - b.cy).abs());
    match (a.circle, b.circle) {
        (true, true) => dx.hypot(dy) <= a.half + b.half,
        (false, false) => dx <= a.half + b.half && dy <= a.half + b.half,
        // Un rond et un carré : le point du carré le plus proche du centre du rond.
        _ => {
            let (circle, square) = if a.circle { (a, b) } else { (b, a) };
            (dx - square.half).max(0.0).hypot((dy - square.half).max(0.0)) <= circle.half
        }
    }
}

/// Ce qu'une règle `When` guette est-il vrai, pour cet état ? Une valeur (`When(lives, is: 0)`),
/// ou la rencontre de deux blocs (`When(Basket, meets: Apple)`).
fn watches(program: &Program, rule: &Block, state: &State, texts: &Texts) -> bool {
    if rule.argument("meets").is_some() {
        return meets(rule).is_ok_and(|(a, b, distance)| {
            // Avec « within », on juge sur l'écart entre les places, de 0 à 100. Sans lui, sur le
            // contact : le bord de l'un touche le bord de l'autre.
            if rule.argument("within").is_some() {
                return match (place_of(program, state, a), place_of(program, state, b)) {
                    (Some((ax, ay)), Some((bx, by))) => ax.abs_diff(bx) <= distance && ay.abs_diff(by) <= distance,
                    _ => false,
                };
            }
            match (body(program, state, a), body(program, state, b)) {
                (Some(a), Some(b)) => touch_each_other(&a, &b),
                _ => false,
            }
        });
    }
    condition(rule).is_ok_and(|(value, comparisons)| holds(value, &comparisons, &to_show(program, state), texts))
}

/// Les touches du clavier que les règles du fichier écoutent : `On(Key.left, …)`. Les flèches,
/// l'espace, Entrée et Échap ; les lettres, celles écrites sur la touche ; les chiffres, de la
/// rangée du haut ou du pavé numérique, avec ou sans Maj (ADR-061). Jamais Tab : elle sert à
/// passer d'un bouton à l'autre.
pub const KEYPRESSES: &[&str] = &[
    "left", "right", "up", "down", "space", "enter", "escape",
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
    "digit0", "digit1", "digit2", "digit3", "digit4", "digit5", "digit6", "digit7", "digit8", "digit9",
];

pub fn keypresses(program: &Program) -> Vec<String> {
    let mut keypresses = Vec::new();
    let _ = for_each_block(&program.root, &mut |block| {
        if let ("On", Some(Value::Name(signal))) = (block.name.as_str(), block.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value)) {
            if let Some(keypress) = signal.strip_prefix("Key.") {
                if !keypresses.iter().any(|t| t == keypress) {
                    keypresses.push(keypress.to_string());
                }
            }
        }
        Ok(())
    });
    keypresses
}

/// `cart=2;likes=0` : l'état, pour le garder d'un geste à l'autre du côté de la page.
pub fn write(state: &State) -> String {
    state.iter().map(|(name, value)| format!("{name}={value}")).collect::<Vec<_>>().join(";")
}

/// Relit un état écrit par `ecrire`. Seules les valeurs que la page déclare sont reprises, et
/// jamais au-delà des bornes : un état abîmé ou falsifié ne fait rien de plus que le départ.
pub fn reread(program: &Program, written: &str) -> State {
    let mut state = initial(program).unwrap_or_default();
    for chunk in written.split(';') {
        if let Some((name, value)) = chunk.split_once('=') {
            // Le compte des tirages au hasard suit l'état, pour que la suite continue.
            if let (true, Ok(n)) = (name == DRAWS, value.parse::<u64>()) {
                state.push((DRAWS.to_string(), n));
                continue;
            }
            // L'heure n'est jamais reprise de l'état écrit : c'est celle donnée au moteur.
            if CLOCK.contains(&name) {
                continue;
            }
            if let (Some((_, place)), Ok(value)) = (state.iter_mut().find(|(known, _)| known == name), value.parse::<u64>()) {
                *place = value.min(VALUE_MAX);
            }
        }
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    const CART: &str = "Page(
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

    fn page(source: &str) -> Result<Program, Error> {
        crate::check_page(source)
    }

    #[test]
    fn the_cart_fills_and_empties_through_requests() {
        let program = page(CART).unwrap();
        let start_value = initial(&program).unwrap();
        assert_eq!(write(&start_value), "cart=0;likes=3");
        let two = arbitrate(&program, &arbitrate(&program, &start_value, &Texts::new(), "Add.tap"), &Texts::new(), "Add.tap");
        assert_eq!(write(&two), "cart=2;likes=3");
        assert_eq!(write(&arbitrate(&program, &two, &Texts::new(), "Remove.tap")), "cart=1;likes=3");
        // Un signal, deux règles : les deux demandes sont faites, dans l'ordre.
        assert_eq!(write(&arbitrate(&program, &two, &Texts::new(), "Empty.tap")), "cart=0;likes=4");
        // Un signal sans règle ne change rien.
        assert_eq!(arbitrate(&program, &two, &Texts::new(), "Nobody.tap"), two);
    }

    #[test]
    fn a_value_stays_within_its_bounds() {
        let program = page(CART).unwrap();
        let start_value = initial(&program).unwrap();
        // On ne descend pas sous zéro.
        assert_eq!(write(&arbitrate(&program, &start_value, &Texts::new(), "Remove.tap")), "cart=0;likes=3");
        // On ne dépasse pas le plafond, même en partant d'un état falsifié.
        let full = reread(&program, "cart=99999999999999;likes=7;intrus=4;cart");
        assert_eq!(write(&full), format!("cart={VALUE_MAX};likes=7"));
        assert_eq!(write(&arbitrate(&program, &full, &Texts::new(), "Add.tap")), format!("cart={VALUE_MAX};likes=7"));
    }

    #[test]
    fn the_page_shows_values_in_its_texts() {
        let html = crate::flat_view(CART, "").unwrap();
        assert!(html.contains("<span data-state=\"cart\">0</span> paintings in the cart"), "{html}");
        assert!(html.contains("Add (<span data-state=\"cart\">0</span>)"), "{html}");
        assert!(html.contains("<span data-state=\"likes\">3</span> people"), "{html}");
        // Des accolades qui n'entourent pas un nom restent du texte.
        let html = crate::flat_view("Page(children: [ \"{ } and {Not A Name} and {}\" ])", "").unwrap();
        assert!(html.contains("{ } and {Not A Name} and {}"), "{html}");
    }

    const SHOP: &str = "Page(
  state: State(sunrise: 0, blueDoor: 2, likes: 5),
  prices: Prices(sunrise: 120, blueDoor: 90),
  children: [
    Text(\"{count} paintings, {total} euros\"),
    Button(name: Add, text: \"Add\"),
  ],
  rules: [ On(Add.tap, effect: sunrise.add(1)) ],
)";

    #[test]
    fn with_prices_the_engine_counts_and_adds() {
        let program = page(SHOP).unwrap();
        let start_value = initial(&program).unwrap();
        // « likes » n'a pas de prix : ce n'est pas un article, il ne compte pas.
        assert_eq!(write(&to_show(&program, &start_value)), "sunrise=0;blueDoor=2;likes=5;count=2;total=180");
        let after = arbitrate(&program, &start_value, &Texts::new(), "Add.tap");
        assert_eq!(write(&to_show(&program, &after)), "sunrise=1;blueDoor=2;likes=5;count=3;total=300");
        assert!(crate::flat_view(SHOP, "").unwrap().contains("<span data-state=\"count\">2</span> paintings, <span data-state=\"total\">180</span> euros"));
        // Ce que la page renvoie contient les valeurs calculées ; elles ne sont pas reprises telles
        // quelles : le moteur les recalcule toujours.
        assert_eq!(crate::arbitrate(SHOP, "sunrise=1;blueDoor=2;likes=5;count=999;total=1", "Add.tap"), "sunrise=2;blueDoor=2;likes=5;count=4;total=420");
        // Sans prix, pas de valeurs calculées : « count » est un nom libre.
        assert_eq!(crate::initial_state("Page(state: State(count: 7))"), "count=7");
        // Un total ne déborde pas.
        let huge = page("Page(state: State(a: 1000000000), prices: Prices(a: 1000000000))").unwrap();
        assert_eq!(write(&computed(&huge, &initial(&huge).unwrap())), "count=1000000000;total=1000000000000000000");
    }

    #[test]
    fn a_condition_shows_or_hides_by_a_value() {
        let source = "Page(
  state: State(cart: 0),
  children: [
    If(cart, is: 0, children: [ \"Your cart is empty.\" ]),
    If(cart, over: 0, under: 3, children: [ Button(name: Pay, text: \"Pay\") ]),
    If(cart, not: 0, children: [ \"{cart} in your cart\" ]),
  ],
)";
        let html = crate::flat_view(source, "").unwrap();
        // Au départ, le panier est vide : la première condition est vraie, les deux autres non.
        assert!(html.contains("<div class=\"holo-If\" data-if=\"cart|is=0\"><p class=\"holo-P\">Your cart is empty.</p></div>"), "{html}");
        assert!(html.contains("<div class=\"holo-If\" data-if=\"cart|over=0|under=3\" hidden><button"), "{html}");
        assert!(html.contains("data-if=\"cart|not=0\" hidden>"), "{html}");
        // Après un changement, c'est encore le moteur qui répond, par le même calcul.
        assert_eq!(crate::conditions(source, "cart=0"), "cart|is=0:1;cart|over=0|under=3:0;cart|not=0:0");
        assert_eq!(crate::conditions(source, "cart=2"), "cart|is=0:0;cart|over=0|under=3:1;cart|not=0:1");
        assert_eq!(crate::conditions(source, "cart=3"), "cart|is=0:0;cart|over=0|under=3:0;cart|not=0:1");
        let (between, nothing) = ([("over", Term::Number(0)), ("under", Term::Number(3))], State::new());
        assert!(real_one(&between, 2, &nothing) && !real_one(&between, 3, &nothing) && !real_one(&between, 0, &nothing));
        assert!(real_one(&[("is", Term::Number(5))], 5, &nothing) && real_one(&[("not", Term::Number(5))], 4, &nothing) && !real_one(&[("not", Term::Number(5))], 5, &nothing));
        // Avec des prix, une condition peut regarder ce que le moteur calcule.
        page("Page(state: State(a: 0), prices: Prices(a: 10), children: [ If(total, over: 100, children: [ \"Free delivery\" ]) ])").unwrap();
        for (source, message) in [
            ("Page(children: [ If(cart, is: 0, children: []) ])", "aucune valeur ne s'appelle « cart »"),
            ("Page(state: State(cart: 0), children: [ If(cart, children: []) ])", "une condition s'écrit"),
            ("Page(state: State(cart: 0), children: [ If(cart, is: \"zero\", children: []) ])", "« cart » est un nombre : on le compare à un nombre"),
            ("Page(state: State(cart: 0), children: [ If(cart, above: 0, children: []) ])", "n'a pas de paramètre « above »"),
            ("Page(state: State(cart: 0), children: [ If(cart, is: 0) ])", "attend ce qu'il montre"),
            ("Page(state: State(cart: 0), children: [ If(is: 0, children: []) ])", "une condition s'écrit"),
        ] {
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    const GAME: &str = include_str!("../../exemples/jeu/attraper.holo");

    #[test]
    fn the_game_is_played_with_rules_time_and_randomness() {
        let program = page(GAME).unwrap();
        // Trois règles de temps, trois horloges : le temps chaque seconde, l'étoile toutes les deux.
        assert_eq!(clocks(&program), [(1000, "time".to_string()), (2000, "starX,starY".to_string())]);
        let start_value = initial(&program).unwrap();
        assert_eq!(write(&start_value), "time=0;score=0;starX=50;starY=50;best=0");
        // Tant que la partie n'a pas commencé, le temps reste à zéro : il ne descend pas dessous.
        assert_eq!(arbitrate(&program, &start_value, &Texts::new(), "every:0")[0], ("time".to_string(), 0));
        // « Play » : trente secondes. Ce geste change le temps : son horloge repartira de zéro.
        let launched = arbitrate(&program, &start_value, &Texts::new(), "Play.tap");
        assert_eq!((launched[0].1, launched[1].1), (30, 0));
        assert_eq!(touched_ones(&program, "Play.tap"), ["score", "time"]);
        // Une seconde passe : seul le temps change. L'étoile a sa propre horloge.
        let one_second = arbitrate(&program, &launched, &Texts::new(), "every:0");
        assert_eq!((one_second[0].1, one_second[2].1, one_second[3].1), (29, 50, 50));
        let moved = arbitrate(&program, &one_second, &Texts::new(), "every:1");
        assert!(moved[2].1 <= 100 && moved[3].1 <= 100);
        assert_ne!((moved[2].1, moved[3].1), (50, 50), "l'étoile n'a pas bougé");
        // Toucher l'étoile : un point, elle part ailleurs, et son horloge repart : elle reste là
        // deux vraies secondes.
        let touched = arbitrate(&program, &moved, &Texts::new(), "Star.tap");
        assert_eq!(touched[1].1, 1);
        assert_ne!((touched[2].1, touched[3].1), (moved[2].1, moved[3].1));
        assert_eq!(touched_ones(&program, "Star.tap"), ["score", "starX", "starY"]);
        // Trente secondes plus tard, la partie est finie, et le score est gardé.
        let end = (0..40).fold(touched, |state, _| arbitrate(&program, &state, &Texts::new(), "every:0"));
        assert_eq!((end[0].1, end[1].1), (0, 1));
    }

    const APPLE_CART: &str = include_str!("../../exemples/jeu/panier.holo");

    /// La règle de rencontre du jeu de la pomme.
    fn rule_program(program: &Program) -> &Block {
        let mut rule = None;
        let _ = for_each_block(&program.root, &mut |block| {
            if block.name == "When" && block.argument("meets").is_some() {
                rule = Some(block);
            }
            Ok(())
        });
        rule.unwrap()
    }

    #[test]
    fn the_keyboard_what_falls_and_the_meetings() {
        let program = page(APPLE_CART).unwrap();
        assert_eq!(keypresses(&program), ["left", "right"]);
        let value = |state: &State, name: &str| state.iter().find(|(known, _)| known == name).unwrap().1;
        let start_value = initial(&program).unwrap();
        let plays = arbitrate(&program, &start_value, &Texts::new(), "Play.tap");
        assert_eq!((value(&plays, "lives"), value(&plays, "basket"), value(&plays, "appleY")), (3, 50, 0));
        // Le clavier déplace le panier, qui ne sort jamais du plateau.
        let left = (0..20).fold(plays.clone(), |state, _| arbitrate(&program, &state, &Texts::new(), "Key.left"));
        assert_eq!(value(&left, "basket"), 0);
        let right = (0..40).fold(left, |state, _| arbitrate(&program, &state, &Texts::new(), "Key.right"));
        assert_eq!(value(&right, "basket"), 100);
        // La pomme tombe. Le panier est dessous (50 et 50) : à la rencontre, un point, et une
        // nouvelle pomme repart d'en haut. Une seule fois, pas à chaque battement.
        let mut state = plays.clone();
        let mut beats = 0;
        while value(&state, "score") == 0 && beats < 60 {
            state = arbitrate(&program, &state, &Texts::new(), "every:0");
            beats += 1;
        }
        assert_eq!((value(&state, "score"), value(&state, "appleY"), value(&state, "lives")), (1, 0, 3), "après {beats} battements");
        assert!(beats > 20, "la pomme a été prise trop tôt : {beats}");
        // Le panier parti loin, la pomme arrive en bas : une vie de moins, une seule, et une
        // nouvelle pomme.
        let mut state = (0..20).fold(state, |e, _| arbitrate(&program, &e, &Texts::new(), "Key.left"));
        let apple_on_right = APPLE_CART.replace("appleX.random(100)", "appleX.set(90)");
        let program = page(&apple_on_right).unwrap();
        state.iter_mut().find(|(name, _)| name == "appleX").unwrap().1 = 90;
        let mut beats = 0;
        while value(&state, "lives") == 3 && beats < 60 {
            state = arbitrate(&program, &state, &Texts::new(), "every:0");
            beats += 1;
        }
        assert_eq!((value(&state, "lives"), value(&state, "appleY"), value(&state, "score")), (2, 0, 1));
        // Trois pommes perdues : la partie est finie.
        let end = (0..200).fold(state, |e, _| arbitrate(&program, &e, &Texts::new(), "every:0"));
        assert_eq!((value(&end, "lives"), value(&end, "score")), (0, 1));
        for (source, message) in [
            ("Page(state: State(a: 0), rules: [ When(a, effect: a.set(0)) ])", "une règle qui guette s'écrit"),
            ("Page(state: State(a: 0), rules: [ When(b, is: 1, effect: a.set(0)) ])", "aucune valeur ne s'appelle « b »"),
            ("Page(state: State(a: 0), rules: [ When(a, is: 1) ])", "attend une demande"),
            ("Page(state: State(a: 0), rules: [ When(a, is: 1, children: []) ])", "n'a pas de paramètre « children »"),
            ("Page(state: State(a: 0), children: [ Board(children: [ Point(name: A, seed: 1, x: 1, y: 1) ]) ], rules: [ When(A, meets: 3, effect: a.add(1)) ])", "une rencontre s'écrit"),
            ("Page(state: State(a: 0), children: [ Board(children: [ Point(name: A, seed: 1, x: 1, y: 1) ]), P(name: B, \"x\") ], rules: [ When(A, meets: B, effect: a.add(1)) ])", "« B » n'est pas posé sur un plateau"),
            ("Page(state: State(a: 0), children: [ Board(children: [ Point(name: A, seed: 1, x: 1, y: 1), Point(name: B, seed: 2, x: 2, y: 2) ]) ], rules: [ When(A, meets: B, within: 500, effect: a.add(1)) ])", "de 1 à 100"),
            ("Page(state: State(a: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(Key.home, effect: a.add(1)) ])", "le clavier donne"),
            ("Page(state: State(a: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(Key.A, effect: a.add(1)) ])", "écris « a »"),
            ("Page(state: State(a: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(Key.tab, effect: a.add(1)) ])", "Tab sert à passer d'un bouton à l'autre"),
        ] {
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
        // Les règles du jeu sont rangées sous une condition : tant que la partie n'a pas commencé
        // (aucune vie), la pomme ne tombe pas, et rien n'est rattrapé ni perdu en cachette.
        requested_capabilities();
        let before_playing = (0..60).fold(start_value.clone(), |e, _| arbitrate(&program, &e, &Texts::new(), "every:0"));
        assert_eq!(before_playing, start_value, "le jeu a joué tout seul avant « Play »");
        assert!(requested_capabilities().is_empty(), "un son a été demandé avant « Play »");
        // Le contact, pas la pénétration : la pomme (un rond de 44) est prise au moment où son
        // bord touche le dessus du panier (un carré de 64), pas quand elle est déjà dedans.
        let program = page(APPLE_CART).unwrap();
        let with = |y: u64| { let mut e = plays.clone(); e.iter_mut().find(|(n, _)| n == "appleY").unwrap().1 = y; e };
        // Plateau de 360 : le dessus du panier est à 284 ; le bas de la pomme est à y/100 × 316 + 44.
        assert!(!watches(&program, rule_program(&program), &with(75), &Texts::new()), "à 75, le bas de la pomme est à 281 : elle ne touche pas encore");
        assert!(watches(&program, rule_program(&program), &with(76), &Texts::new()), "à 76, le bas de la pomme est à 284 : elle touche");
        // Sur le côté : le panier décalé d'un peu plus que la moitié des deux largeurs ne touche plus.
        let mut beside = with(96);
        beside.iter_mut().find(|(n, _)| n == "basket").unwrap().1 = 59;
        assert!(watches(&program, rule_program(&program), &beside, &Texts::new()));
        beside.iter_mut().find(|(n, _)| n == "basket").unwrap().1 = 60;
        assert!(!watches(&program, rule_program(&program), &beside, &Texts::new()));
        // Le plateau garde ses proportions : une page qui annoncerait la largeur de son écran
        // (comme avant) ne change rien à la rencontre.
        assert_eq!(write(&reread(&program, &format!("{};<=320", write(&beside)))), write(&beside));
        // Comme dans le navigateur : l'état voyage en texte. La pomme tombe, et elle est prise.
        let mut text = crate::arbitrate(APPLE_CART, &crate::initial_state(APPLE_CART), "Play.tap");
        let mut beats = 0;
        while !text.contains("score=1") && beats < 60 {
            text = crate::arbitrate(APPLE_CART, &text, "every:0");
            beats += 1;
        }
        assert!(text.contains("score=1") && text.contains("lives=3"), "après {beats} battements : {text}");
        // Faire glisser le panier : sa valeur suit le doigt, sans sortir du plateau ; un bloc qui
        // ne se laisse pas glisser ne bouge pas ; et une rencontre faite en glissant compte.
        let program = page(APPLE_CART).unwrap();
        let dragging = drag(&program, &plays, &Texts::new(), "Basket", 250, 10);
        assert_eq!((value(&dragging, "basket"), value(&dragging, "appleY")), (100, 0));
        assert_eq!(drag(&program, &plays, &Texts::new(), "Apple", 10, 90), plays);
        let mut near = plays.clone();
        near.iter_mut().find(|(name, _)| name == "appleY").unwrap().1 = 90;
        near.iter_mut().find(|(name, _)| name == "basket").unwrap().1 = 10;
        let caught = drag(&program, &near, &Texts::new(), "Basket", 50, 0);
        assert_eq!((value(&caught, "score"), value(&caught, "appleY")), (1, 0));
        // Une règle peut faire plusieurs demandes, dans l'ordre ; une seule s'écrit sans crochets.
        let several = page("Page(state: State(a: 0, b: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: [a.add(2), b.set(7), a.add(1)]) ])").unwrap();
        assert_eq!(write(&arbitrate(&several, &initial(&several).unwrap(), &Texts::new(), "B.tap")), "a=3;b=7");
        // Un fichier où deux règles se relancent l'une l'autre ne tourne pas sans fin.
        let cycle = page("Page(state: State(a: 0, b: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: a.set(1)), When(a, is: 1, effect: a.set(0)), When(a, is: 0, effect: a.set(1)) ])").unwrap();
        let _ = arbitrate(&cycle, &initial(&cycle).unwrap(), &Texts::new(), "B.tap");
    }

    #[test]
    fn server_data_goes_through_the_arbiter() {
        let source = "Page(
  state: State(stock: 0, message: \"\", cart: 2, ouvert: 0, alerte: 0),
  data: Data(from: \"stock.json\", every: 30s),
  children: [ Text(\"{stock} en stock. {message}\"), Input(value: cart, label: \"x\", max: 5) ],
  rules: [ When(stock, is: 0, effect: alerte.set(1)) ],
)";
        let program = page(source).unwrap();
        assert_eq!(data_source(&program).unwrap(), Some(("stock.json".to_string(), 30_000)));
        assert_eq!(crate::data(source), "stock.json|30000");
        // Un nombre va dans un nombre, un texte dans un texte ; le reste est laissé de côté.
        let start_value = crate::initial_state(source);
        let received = crate::receive(source, &start_value, r#"{ "stock": 4, "message": "Ouvert \"aujourd'hui\"", "ouvert": true, "cart": 99, "inconnu": 7, "prix": 3.5, "liste": [1, {"a": "}"}], "rien": null, "stock2": -1 }"#);
        assert_eq!(received, format!("stock=4;cart=5;ouvert=1;alerte=0;message='{}", encode("Ouvert \"aujourd'hui\"")));
        // Un texte offert à un nombre, ou l'inverse, ne change rien.
        assert_eq!(crate::receive(source, &start_value, r#"{"stock": "beaucoup", "message": 12}"#), start_value);
        // Un fichier mal formé, ou trop gros, ne change rien.
        for bad in ["", "[1, 2]", "{\"stock\": 4", "{stock: 4}", "<html>", &format!("{{\"message\": \"{}\"}}", "x".repeat(DATA_BYTES))] {
            assert_eq!(crate::receive(source, &start_value, bad), start_value, "{}", &bad[..bad.len().min(30)]);
        }
        // Les règles qui guettent voient arriver les données : le stock tombe à zéro.
        let full = crate::receive(source, &start_value, r#"{"stock": 3}"#);
        assert!(crate::receive(source, &full, r#"{"stock": 0}"#).contains("alerte=1"));
        // Sans « data: », rien n'est reçu.
        let without = source.replace("  data: Data(from: \"stock.json\", every: 30s),\n", "");
        assert_eq!(crate::receive(&without, &crate::initial_state(&without), r#"{"stock": 4}"#), crate::initial_state(&without));
        assert_eq!(crate::data(&without), "");
        for (source, message) in [
            ("Page(data: Data(from: \"https://ailleurs.example/x.json\"))", "rangé à côté"),
            ("Page(data: Data(from: \"../secret.json\"))", "rangé à côté"),
            ("Page(data: Data(from: \"stock.txt\"))", "rangé à côté"),
            ("Page(data: Data(every: 30s))", "attend « from »"),
            ("Page(data: Data(from: \"a.json\", every: 10ms))", "de 1s à 3600s"),
            ("Page(data: Data(from: \"a.json\", fill: 3))", "n'a pas de paramètre « fill »"),
            ("Page(data: 3)", "un bloc « Data(...) »"),
        ] {
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn a_sound_is_played_by_a_rule() {
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
        let html = crate::flat_view(source, "/x/").unwrap();
        assert!(html.contains("<audio class=\"holo-Sound\" data-name=\"Ding\" preload=\"auto\" src=\"/x/ding.wav\"></audio>"), "{html}");
        // Le son demandé par un geste est donné à la page avec les autres effets du geste.
        assert_eq!(crate::effects(source, "B.tap"), ["Ding.play"]);
        // Celui d'une règle qui guette, ou d'une règle de temps, suit l'état, sous le nom « ! ».
        let one = crate::arbitrate(source, &crate::initial_state(source), "B.tap");
        assert_eq!(one, "n=1");
        assert_eq!(crate::arbitrate(source, &one, "B.tap"), "n=0;!=Ding.play");
        assert_eq!(crate::arbitrate(source, &one, "every:0"), "n=1;!=Ding.play");
        // Ce « ! » n'est pas une valeur : relu, il est laissé de côté.
        assert_eq!(crate::arbitrate(source, "n=1;!=Ding.play", "Nobody.tap"), "n=1");
        for (source, message) in [
            ("Page(children: [ Sound(name: D, source: \"https://x.example/a.wav\") ])", "un fichier de son rangé à côté"),
            ("Page(children: [ Sound(name: D, source: \"a.exe\") ])", "un fichier de son rangé à côté"),
            ("Page(children: [ Sound(source: \"a.wav\") ])", "un son a un nom"),
            ("Page(children: [ Sound(name: D, source: \"a.wav\", loop: yes) ])", "true ou false"),
            ("Page(children: [ Sound(name: D, source: \"a.wav\", volume: 1.5) ])", "de 0 (muet) à 1"),
            ("Page(children: [ Sound(name: D, source: \"a.wav\", volume: 40px) ])", "de 0 (muet) à 1"),
            ("Page(children: [ Sound(name: D, source: \"a.wav\"), Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: D.pause) ])", "capacité inconnue « pause »"),
            ("Page(state: State(n: 0), children: [ Point(name: P, seed: 1, inside: World(children: [])) ], rules: [ Every(1s, effect: P.enter) ])", "demande un geste du visiteur"),
            ("Page(state: State(n: 0), children: [ Point(name: P, seed: 1, inside: World(children: [])) ], rules: [ When(n, is: 1, effect: [n.set(0), P.enter]) ])", "demande un geste du visiteur"),
        ] {
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn compare_and_set_from_another_value() {
        // Le meilleur score : au moment où le score dépasse le meilleur, le meilleur le rattrape.
        let source = "Page(
  state: State(score: 0, best: 2),
  children: [ Button(name: B, text: \"x\"), If(score, over: best, children: [ \"jamais vu : la règle rattrape\" ]), If(score, is: best, children: [ \"record égalé\" ]) ],
  rules: [ On(B.tap, effect: score.add(1)), When(score, over: best, effect: best.set(score)) ],
)";
        let suite: Vec<String> = (0..4).scan(crate::initial_state(source), |state, _| { *state = crate::arbitrate(source, state, "B.tap"); Some(state.clone()) }).collect();
        assert_eq!(suite, ["score=1;best=2", "score=2;best=2", "score=3;best=3", "score=4;best=4"]);
        assert_eq!(crate::conditions(source, "score=2;best=2"), "score|over=best:0;score|is=best:1");
        assert_eq!(crate::conditions(source, "score=1;best=2"), "score|over=best:0;score|is=best:0");
        // Ajouter une autre valeur : a.add(b).
        let sum = "Page(state: State(a: 1, b: 5), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: a.add(b)) ])";
        assert_eq!(crate::arbitrate(sum, &crate::initial_state(sum), "B.tap"), "a=6;b=5");
        for (source, message) in [
            ("Page(state: State(a: 0), children: [ If(a, over: b, children: []) ])", "aucun nombre ne s'appelle « b »"),
            ("Page(state: State(a: 0, t: \"\"), children: [ If(a, over: t, children: []) ])", "« a » est un nombre et « t » un texte : on compare deux nombres, ou deux textes"),
            ("Page(state: State(a: 0), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: a.set(b)) ])", "aucun nombre ne s'appelle « b »"),
            ("Page(state: State(a: 0), rules: [ When(a, over: b, effect: a.set(0)) ])", "aucun nombre ne s'appelle « b »"),
        ] {
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn a_text_is_compared_to_a_text() {
        // Une taille choisie, une réponse écrite, deux mots de passe (ADR-063).
        let source = r#"Page(
  state: State(size: "M", answer: "", password: "", again: "", score: 0, label: """a;b|c=d"e%"""),
  children: [
    Choice(value: size, label: "Size", options: [ "S", "M", "L" ]),
    If(size, is: "L", children: [ "Large." ], else: [ "Not large." ]),
    If(size, not: "M", children: [ "Not medium." ]),
    Input(value: answer, label: "Capital of France"),
    Input(value: password, label: "Password"),
    Input(value: again, label: "Again"),
    If(again, not: password, children: [ "The two differ." ]),
    If(label, is: """a;b|c=d"e%""", children: [ "Odd label." ]),
    Button(name: Pick, text: "Paris"),
    Button(name: Clear, text: "Clear"),
  ],
  rules: [
    When(answer, is: "Paris", effect: score.add(1)),
    On(Pick.tap, effect: answer.set("Paris")),
    On(Clear.tap, effect: answer.set("")),
  ],
)"#;
        // Au départ : « M », donc ni « L » ni « pas M » ; le « sinon » de « L » se montre.
        let html = crate::flat_view(source, "").unwrap();
        assert!(html.contains(r#"data-if="size|is=&quot;L&quot;" hidden><p class="holo-P">Large.</p></div><div class="holo-If" data-else="size|is=&quot;L&quot;"><p"#), "{html}");
        assert!(html.contains(r#"data-if="size|not=&quot;M&quot;" hidden>"#), "{html}");
        // Les signes qui séparent les réponses sont écrits %XX dans le nom de la condition.
        assert!(html.contains(r#"data-if="label|is=&quot;a%3Bb%7Cc%3Dd%22e%25&quot;"><p"#), "{html}");
        let start = crate::initial_state(source);
        let large = crate::input(source, &start, "size", "L");
        assert_eq!(
            crate::conditions(source, &large),
            r#"size|is="L":1;size|not="M":1;again|not=password:0;label|is="a%3Bb%7Cc%3Dd%22e%25":1"#
        );
        // À la lettre près : « l » n'est pas « L ».
        assert!(crate::conditions(source, &crate::input(source, &start, "size", "l")).starts_with(r#"size|is="L":0;"#));
        // Deux valeurs de texte comparées entre elles.
        let typed = crate::input(source, &start, "password", "secret");
        assert!(crate::conditions(source, &typed).contains("again|not=password:1"));
        assert!(crate::conditions(source, &crate::input(source, &typed, "again", "secret")).contains("again|not=password:0"));
        // Une règle qui guette un texte : au moment où il devient « Paris », pas tant qu'il le reste.
        let score = |state: &str| state.split(';').find_map(|chunk| chunk.strip_prefix("score=")).unwrap_or("?").to_string();
        let mut state = start.clone();
        let mut seen = Vec::new();
        for written in ["P", "Pari", "Paris", "Paris", "Lyon", "Paris"] {
            state = crate::input(source, &state, "answer", written);
            seen.push(score(&state));
        }
        assert_eq!(seen, ["0", "0", "1", "1", "1", "2"]);
        // Par un geste aussi : le texte changé par la règle « On » est guetté.
        let picked = crate::arbitrate(source, &crate::arbitrate(source, &state, "Clear.tap"), "Pick.tap");
        assert_eq!(score(&picked), "3");
        assert_eq!(score(&crate::arbitrate(source, &picked, "Pick.tap")), "3", "déjà « Paris » : rien ne devient vrai");
        // Les refus : pas de mélange, et plus grand ou plus petit pour les nombres seulement.
        for (wrong, message) in [
            ("If(size, over: \"M\", children: [])", "« over » compare des nombres ; un texte se compare par is (égal) ou not (différent)"),
            ("If(size, is: 3, children: [])", "« size » est un texte : on le compare à un texte entre guillemets"),
            ("If(size, is: nobody, children: [])", "aucun texte ne s'appelle « nobody »"),
            ("If(size, under: answer, children: [])", "« under » compare des nombres"),
            ("If(score, is: size, children: [])", "« score » est un nombre et « size » un texte"),
            ("If(size, is: score, children: [])", "« size » est un texte et « score » un nombre"),
        ] {
            let page_source = source.replace("Button(name: Pick", &format!("{wrong}, Button(name: Pick"));
            let error = page(&page_source).unwrap_err();
            assert!(error.message.contains(message), "{wrong}\n→ {error}");
        }
        let error = page(&source.replace("When(answer, is: \"Paris\"", "When(answer, over: 2")).unwrap_err();
        assert!(error.message.contains("« answer » est un texte"), "{error}");
    }

    #[test]
    fn rules_under_a_text_condition() {
        // Avant ADR-063, une règle rangée sous une condition sur un texte ne valait jamais.
        let source = r#"Page(
  state: State(mode: "", time: 0, done: 0),
  children: [ Input(value: mode, label: "Mode"), P("{time}") ],
  rules: [ If(mode, is: "play", rules: [ Every(1s, effect: time.add(1)), After(2s, effect: done.set(1)) ]) ],
)"#;
        let start = crate::initial_state(source);
        assert_eq!(crate::arbitrate(source, &start, "every:0"), start, "« mode » est vide : l'horloge ne compte pas");
        assert_eq!(crate::delays(source, &start), "2000:0");
        let playing = crate::input(source, &start, "mode", "play");
        assert!(crate::arbitrate(source, &playing, "every:0").starts_with("time=1;"));
        assert_eq!(crate::delays(source, &playing), "2000:1");
    }

    #[test]
    fn a_value_can_be_a_text() {
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
        let html = crate::flat_view(source, "").unwrap();
        assert!(html.contains("<input type=\"text\" maxlength=\"12\" value=\"\" data-bind=\"buyer\">"), "{html}");
        assert!(html.contains("<input type=\"text\" maxlength=\"80\" value=\"Paris\" data-bind=\"city\">"), "{html}");
        // Au départ le prénom est vide : la salutation est cachée, la question montrée.
        assert!(html.contains("data-if=\"buyer|not=&quot;&quot;\" hidden><p class=\"holo-P\">Hello <span data-state=\"buyer\"></span>, from <span data-state=\"city\">Paris</span>.</p>"), "{html}");
        assert!(html.contains("data-if=\"buyer|is=&quot;&quot;\"><p"), "{html}");
        let start_value = crate::initial_state(source);
        assert_eq!(start_value, "cart=0;buyer=';city='Paris");
        // Écrire un prénom : il est nettoyé et coupé à la longueur permise ; rien ne se mêle
        // aux séparateurs de l'état, même un texte hostile.
        let written = crate::input(source, &start_value, "buyer", "Zoé;cart=99<b>&\u{7} et la suite est trop longue");
        assert_eq!(written, format!("cart=0;buyer='{};city='Paris", encode("Zoé;cart=99<")));
        assert_eq!(crate::conditions(source, &written), "buyer|not=\"\":1;buyer|is=\"\":0");
        // Un geste ailleurs ne touche pas aux textes.
        let after = crate::arbitrate(source, &written, "Add.tap");
        assert!(after.starts_with("cart=1;buyer='Zo") && after.ends_with(";city='Paris"), "{after}");
        // Garder et reprendre : seul le prénom est gardé.
        assert_eq!(crate::to_keep(source, &after), format!("buyer='{}", encode("Zoé;cart=99<")));
        let resumed = crate::resume(source, &format!("buyer='{};city='{};cart=5", encode("Ada"), encode("Lyon")));
        assert_eq!(resumed, "cart=0;buyer='Ada;city='Paris");
        // Ce que montre la page ne devient jamais du code.
        let hostile = crate::flat_view(&source.replace("city: \"Paris\"", "city: \"<script>x</script>\""), "").unwrap();
        assert!(!hostile.contains("<script>") && hostile.contains("&lt;script&gt;"), "{hostile}");
        assert_eq!(decode(&encode("é à 🙂 ; = '")).as_deref(), Some("é à 🙂 ; = '"));
        assert_eq!(decode("%ZZ"), None);
        for (source, message) in [
            ("Page(state: State(a: \"\"), children: [ If(a, over: 2, children: []) ])", "est un texte"),
            ("Page(state: State(a: 0), children: [ If(a, is: \"\", children: []) ])", "est un nombre"),
            ("Page(state: State(a: \"\"), children: [ If(a, under: \"oui\", children: []) ])", "« under » compare des nombres"),
            ("Page(state: State(a: \"\"), children: [ Checkbox(value: a, label: \"x\") ])", "une case attend un nombre"),
            ("Page(state: State(a: \"\", a: 0))", "déclarée deux fois"),
            ("Page(state: State(a: \"\"), children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: a.add(1)) ])", "« a » est un texte"),
        ] {
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn a_field_a_checkbox_and_kept_values() {
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
        let program = page(source).unwrap();
        let html = crate::flat_view(source, "").unwrap();
        assert!(html.contains("<label class=\"holo-Input\"><span>Tip, in euros</span><input type=\"number\" inputmode=\"numeric\" min=\"0\" max=\"50\" value=\"0\" data-bind=\"tip\"></label>"), "{html}");
        assert!(html.contains("<label class=\"holo-Checkbox\"><input type=\"checkbox\" data-bind=\"gift\"><span>Gift <strong>wrap</strong></span></label>"), "{html}");
        // La saisie passe par l'arbitre : bornée, et sourde à ce qui n'est pas un nombre.
        let start_value = initial(&program).unwrap();
        assert_eq!(write(&input(&program, &start_value, &Texts::new(), "tip", " 12 ")), "tip=12;gift=0;visits=0");
        assert_eq!(write(&input(&program, &start_value, &Texts::new(), "tip", "9999")), "tip=50;gift=0;visits=0");
        assert_eq!(write(&input(&program, &start_value, &Texts::new(), "tip", "douze")), "tip=0;gift=0;visits=0");
        assert_eq!(write(&input(&program, &input(&program, &start_value, &Texts::new(), "tip", "7"), &Texts::new(), "tip", "")), "tip=0;gift=0;visits=0");
        assert_eq!(write(&input(&program, &start_value, &Texts::new(), "gift", "5")), "tip=0;gift=1;visits=0");
        // Une valeur qu'aucun champ ne présente ne se saisit pas, même si on le demande.
        assert_eq!(write(&input(&program, &start_value, &Texts::new(), "visits", "40")), "tip=0;gift=0;visits=0");
        // Garder : seules les valeurs nommées par « keep » sont écrites, et seules elles sont reprises.
        assert_eq!(crate::to_keep(source, "tip=12;gift=1;visits=9"), "tip=12;gift=1");
        assert_eq!(crate::resume(source, "tip=12;gift=1;visits=9;intrus=3"), "tip=12;gift=1;visits=0");
        assert_eq!(crate::resume(source, "n'importe quoi"), "tip=0;gift=0;visits=0");
        // Une case déjà cochée et un champ déjà rempli le sont dès le premier affichage.
        let filled_in = crate::flat_view(&source.replace("State(tip: 0, gift: 0", "State(tip: 8, gift: 1"), "").unwrap();
        assert!(filled_in.contains("value=\"8\" data-bind=\"tip\"") && filled_in.contains("<input type=\"checkbox\" data-bind=\"gift\" checked>"), "{filled_in}");
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
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn randomness_is_replayable_and_stays_within_bounds() {
        let source = "Page(name: Dice, state: State(die: 0), children: [ Button(name: Roll, text: \"Roll\") ], rules: [ On(Roll.tap, effect: die.random(5)) ])";
        let throws = |n: usize| {
            let mut state = crate::initial_state(source);
            (0..n).map(|_| { state = crate::arbitrate(source, &state, "Roll.tap"); state.clone() }).collect::<Vec<_>>()
        };
        // Les mêmes gestes redonnent les mêmes nombres (ADR-008).
        assert_eq!(throws(50), throws(50));
        // De 0 à 5, bornes comprises, et toutes les faces sortent.
        let faces: Vec<u64> = throws(200).iter().map(|e| e.split(';').next().unwrap().trim_start_matches("die=").parse().unwrap()).collect();
        assert!(faces.iter().all(|f| *f <= 5));
        for face in 0..=5 {
            assert!(faces.contains(&face), "la face {face} ne sort jamais");
        }
        // Deux fichiers de noms différents n'ont pas la même suite.
        let other = source.replace("name: Dice", "name: Other");
        let suite = |src: &str| (0..20).fold((crate::initial_state(src), Vec::new()), |(state, mut seen_ones), _| { let e = crate::arbitrate(src, &state, "Roll.tap"); seen_ones.push(e.clone()); (e, seen_ones) }).1;
        assert_ne!(suite(source), suite(&other));
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
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn badly_written_prices_are_refused() {
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
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn what_is_badly_written_is_refused_with_a_clear_message() {
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
            let error = page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }
}
