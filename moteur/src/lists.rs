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

use crate::state::{State, Texts};
use crate::holo::{Argument, Block, Error, Program, Value};
use crate::rules::for_each_block;

pub type Lists = Vec<(String, Vec<String>)>;

thread_local! {
    /// Les listes de la page en cours de fabrication.
    static RUNNING: std::cell::RefCell<Lists> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn set_running(lists: Lists) {
    RUNNING.with(|l| *l.borrow_mut() = lists);
}

pub fn running() -> Lists {
    RUNNING.with(|l| l.borrow().clone())
}

/// Le nombre d'éléments d'une liste, au plus, et la longueur d'un élément (ou d'un champ).
pub const ELEMENTS_MAX: usize = 100;
pub const ELEMENT_MAX: usize = 200;
/// Le nombre de champs d'un élément, au plus (ADR-051).
pub const FIELDS_MAX: usize = 16;

/// Un élément à champs commence par ce signe, que le visiteur ne peut pas écrire : les signes
/// invisibles sont retirés de tout ce qu'il saisit. Suivent les champs, `title=Sunrise&price=12000`,
/// chaque valeur codée par `coder`.
pub const RECORD: char = '\u{1d}';

/// Les champs d'un élément, dans l'ordre écrit ; aucun pour un élément de texte.
pub fn fields(element: &str) -> Vec<(String, String)> {
    let Some(remainder) = element.strip_prefix(RECORD) else { return Vec::new() };
    remainder
        .split('&')
        .filter_map(|c| c.split_once('='))
        .filter(|(name, _)| is_field_name(name))
        .filter_map(|(name, code)| crate::state::decode(code).map(|v| (name.to_string(), clean(&v))))
        .take(FIELDS_MAX)
        .collect()
}

/// Écrit un élément à champs.
pub fn record(fields: &[(String, String)]) -> String {
    let mut output = String::from(RECORD);
    output.push_str(&fields.iter().take(FIELDS_MAX).map(|(name, value)| format!("{name}={}", crate::state::encode(&clean(value)))).collect::<Vec<_>>().join("&"));
    output
}

/// Ce qu'on montre pour `{item}` : le texte d'un élément, ou le premier champ d'un élément à champs.
pub fn text_of(element: &str) -> String {
    if element.starts_with(RECORD) {
        return fields(element).into_iter().next().map(|(_, v)| v).unwrap_or_default();
    }
    element.to_string()
}

fn is_field_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase()) && name.chars().all(|c| c.is_ascii_alphanumeric()) && name.len() <= 40 && name != "key"
}

/// Dans une ligne, `If(item.done, is: 1, children: [ … ], else: [ … ])` choisit ce qu'il montre
/// d'après le champ de l'élément ; `If(item, is: "…")` d'après le texte d'un élément de texte
/// (ADR-057). Les `If` qui regardent l'élément sont remplacés par la branche choisie : le reste du
/// moteur ne voit que des blocs ordinaires. Un champ se compare à un nombre (`is`, `not`, `over`,
/// `under`) ou à un texte (`is`, `not`).
pub fn choose_by_element(value: &mut Value, fields: &[(String, String)], text: &str) {
    match value {
        Value::List(elements) => {
            let mut placed_list = Vec::with_capacity(elements.len());
            for mut element in std::mem::take(elements) {
                if let Value::Block(block) = &element {
                    if let Some(chosen_ones) = element_branch(block, fields, text) {
                        for mut chosen in chosen_ones {
                            choose_by_element(&mut chosen, fields, text);
                            placed_list.push(chosen);
                        }
                        continue;
                    }
                }
                choose_by_element(&mut element, fields, text);
                placed_list.push(element);
            }
            *elements = placed_list;
        }
        Value::Block(block) => block.arguments.iter_mut().for_each(|a| choose_by_element(&mut a.value, fields, text)),
        _ => {}
    }
}

/// Le sujet d'un `If` qui regarde l'élément : `item` ou `item.done`.
pub fn element_subject(block: &Block) -> Option<&str> {
    if block.name != "If" {
        return None;
    }
    match block.arguments.first() {
        Some(Argument { name: None, value: Value::Name(subject), .. }) if subject == "item" || subject.starts_with("item.") => Some(subject),
        _ => None,
    }
}

fn element_branch(block: &Block, fields: &[(String, String)], text: &str) -> Option<Vec<Value>> {
    let subject = element_subject(block)?;
    let raw = match subject.strip_prefix("item.") {
        Some(field) => fields.iter().find(|(c, _)| c == field).map(|(_, v)| v.clone()).unwrap_or_default(),
        None => text_of(text),
    };
    let number = raw.parse::<u64>().ok();
    let real = block.arguments[1..].iter().all(|a| match (a.name.as_deref(), &a.value) {
        (Some("is"), Value::Integer(n)) => number == Some(*n),
        (Some("not"), Value::Integer(n)) => number != Some(*n),
        (Some("over"), Value::Integer(n)) => number.is_some_and(|v| v > *n),
        (Some("under"), Value::Integer(n)) => number.is_some_and(|v| v < *n),
        (Some("is"), Value::Text(t)) => raw == *t,
        (Some("not"), Value::Text(t)) => raw != *t,
        _ => true,
    });
    let list = |name: &str| match block.argument(name).map(|a| &a.value) {
        Some(Value::List(l)) => l.clone(),
        _ => Vec::new(),
    };
    Some(if real {
        if block.argument("rules").is_some() { list("rules") } else { list("children") }
    } else {
        list("else")
    })
}

/// Vérifie un `If` qui regarde l'élément d'une ligne : le champ existe, les comparaisons sont
/// bien écrites.
pub fn check_element_if(block: &Block, program: &Program, list: Option<&str>) -> Result<(), Error> {
    let Some(subject) = element_subject(block) else { return Ok(()) };
    let error = |message: String| Err(Error { message, pos: block.pos });
    let Some(list) = list else {
        return error(format!("« If({subject}, …) » regarde l'élément d'une ligne : il s'écrit dans Repeat(…, children: [ … ])"));
    };
    if let Some(field) = subject.strip_prefix("item.") {
        match kind(program, list) {
            Some(Kind::Texts) => return error(format!("les éléments de « {list} » sont des textes, sans champs : If(item, is: \"…\")")),
            Some(Kind::Records(fields)) if !fields.iter().any(|c| c == field) => return error(format!("les éléments de « {list} » n'ont pas de champ « {field} » ; champs : {}", fields.join(", "))),
            _ => {}
        }
    }
    let mut comparisons = 0;
    for a in &block.arguments[1..] {
        match (a.name.as_deref(), &a.value) {
            (Some("children" | "else" | "rules" | "name"), _) => {}
            (Some("is" | "not"), Value::Integer(_) | Value::Text(_)) | (Some("over" | "under"), Value::Integer(_)) => comparisons += 1,
            (Some(word @ ("is" | "not" | "over" | "under")), _) => return error(format!("« If({subject}, {word}: …) » attend un nombre entier{}", if matches!(word, "is" | "not") { ", ou un texte entre guillemets" } else { "" })),
            (Some(word), _) => return error(format!("« If » n'a pas de paramètre « {word} » ; paramètres possibles : is, not, over, under, children, else")),
            (None, _) => return error(format!("« If({subject}, …) » : chaque comparaison est nommée, is: 1")),
        }
    }
    if comparisons == 0 {
        return error(format!("« If({subject}, …) » attend une comparaison : If({subject}, is: 1, children: [ … ])"));
    }
    Ok(())
}

/// La sorte d'une liste, d'après sa déclaration.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// `State(tasks: ["Pain"])` : des textes.
    Texts,
    /// `State(articles: [ Item(title: "…", price: 0) ])` : des éléments à champs, ceux-là.
    Records(Vec<String>),
    /// `State(articles: [])` : vide au départ, des textes ou des éléments à champs.
    Free,
}

/// La sorte d'une liste déclarée.
pub fn kind(program: &Program, name: &str) -> Option<Kind> {
    // Une liste calculée a les éléments de la liste dont elle part.
    if crate::computed::is_computed(program, name) {
        return crate::computed::source_of(program, name).and_then(|source| kind(program, &source));
    }
    let Some(Value::Block(state)) = program.root.argument("state").map(|a| &a.value) else { return None };
    let Some(Value::List(elements)) = state.argument(name).map(|a| &a.value) else { return None };
    Some(match elements.first() {
        None => Kind::Free,
        Some(Value::Block(item)) => Kind::Records(item.arguments.iter().filter_map(|a| a.name.clone()).collect()),
        Some(_) => Kind::Texts,
    })
}

/// Ce qu'on peut demander à une liste.
pub const REQUESTS: &[&str] = &["push", "remove", "clear"];

/// Les listes déclarées par la page, à leur départ : `State(tasks: [])`.
pub fn initial(program: &Program) -> Lists {
    let Some(Value::Block(state)) = program.root.argument("state").map(|a| &a.value) else { return Vec::new() };
    state.arguments
        .iter()
        .filter_map(|a| match (&a.name, &a.value) {
            (Some(name), Value::List(elements)) => Some((name.clone(), elements.iter().filter_map(written_element).collect())),
            _ => None,
        })
        .collect()
}

/// Un élément écrit dans le fichier : un texte, ou un `Item(…)` dont les champs sont des textes
/// ou des nombres entiers.
fn written_element(value: &Value) -> Option<String> {
    match value {
        Value::Text(t) => Some(t.clone()),
        Value::Block(item) if item.name == "Item" => Some(record(
            &item
                .arguments
                .iter()
                .filter_map(|a| match (&a.name, &a.value) {
                    (Some(n), Value::Text(t)) => Some((n.clone(), t.clone())),
                    (Some(n), Value::Integer(e)) => Some((n.clone(), e.to_string())),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        )),
        _ => None,
    }
}

/// Vérifie une liste déclarée : des textes, ou des `Item(…)` qui ont tous les mêmes champs ;
/// cent éléments au plus, de deux cents caractères au plus.
pub fn check_declaration(argument: &Argument) -> Result<(), Error> {
    let Value::List(elements) = &argument.value else { return Ok(()) };
    let refusal = |message: String| Err(Error { message, pos: argument.pos });
    if elements.len() > ELEMENTS_MAX {
        return refusal(format!("une liste a au plus {ELEMENTS_MAX} éléments"));
    }
    let mut first_ones: Option<Vec<String>> = None;
    for element in elements {
        match element {
            Value::Text(t) if t.chars().count() <= ELEMENT_MAX && first_ones.is_none() && !elements.iter().any(|e| matches!(e, Value::Block(_))) => {}
            Value::Text(_) if elements.iter().any(|e| matches!(e, Value::Block(_))) => return refusal("une liste contient des textes ou des Item(…), pas les deux".into()),
            Value::Text(_) => return refusal(format!("un élément de liste fait au plus {ELEMENT_MAX} caractères")),
            Value::Block(item) if item.name == "Item" => {
                let mut names = Vec::new();
                for a in &item.arguments {
                    let Some(name) = a.name.as_deref() else { return refusal("chaque champ d'un « Item » est nommé : Item(title: \"Sunrise\", price: 120)".into()) };
                    if !is_field_name(name) {
                        return refusal(format!("« {name} » : un champ s'écrit comme une valeur, en minuscules, les mots joints par une majuscule (ADR-037)"));
                    }
                    match &a.value {
                        Value::Text(t) if t.chars().count() <= ELEMENT_MAX => {}
                        Value::Integer(_) => {}
                        _ => return refusal(format!("le champ « {name} » attend un texte (deux cents caractères au plus) ou un nombre entier")),
                    }
                    if names.contains(&name.to_string()) {
                        return refusal(format!("le champ « {name} » est donné deux fois"));
                    }
                    names.push(name.to_string());
                }
                if names.is_empty() || names.len() > FIELDS_MAX {
                    return refusal(format!("un « Item » a de 1 à {FIELDS_MAX} champs"));
                }
                match &first_ones {
                    Some(expected) if *expected != names => return refusal(format!("les éléments d'une liste ont les mêmes champs, dans le même ordre : {}", expected.join(", "))),
                    Some(_) => {}
                    None => first_ones = Some(names),
                }
            }
            _ => return refusal("une liste contient des textes entre guillemets, State(tasks: [\"Acheter du pain\"]), ou des Item(…), State(articles: [ Item(title: \"Sunrise\") ])".into()),
        }
    }
    Ok(())
}

pub fn is_list(program: &Program, name: &str) -> bool {
    initial(program).iter().any(|(known, _)| known == name) || crate::computed::is_computed(program, name)
}

/// `tasks=[Pain,Lait]` : chaque élément codé comme un texte, pour ne pas se mêler aux séparateurs.
pub fn write(lists: &Lists) -> String {
    lists.iter().map(|(name, elements)| format!("{name}=[{}]", elements.iter().map(|e| crate::state::encode(e)).collect::<Vec<_>>().join(","))).collect::<Vec<_>>().join(";")
}

/// Relit les listes d'un état. Seules celles que la page déclare, bornées.
pub fn reread(program: &Program, written: &str) -> Lists {
    let mut lists = initial(program);
    for chunk in written.split(';') {
        if let Some((name, remainder)) = chunk.split_once("=[") {
            let Some(inside) = remainder.strip_suffix(']') else { continue };
            if let Some((_, place)) = lists.iter_mut().find(|(known, _)| known == name) {
                *place = inside.split(',').filter(|e| !e.is_empty()).filter_map(crate::state::decode).map(|e| clean_element(&e)).filter(|e| !e.is_empty()).take(ELEMENTS_MAX).collect();
            }
        }
    }
    lists
}

fn clean(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).take(ELEMENT_MAX).collect()
}

/// Un élément relu : un texte nettoyé, ou un élément à champs réécrit champ par champ.
fn clean_element(element: &str) -> String {
    if element.starts_with(RECORD) {
        let read_ones = fields(element);
        return if read_ones.is_empty() { String::new() } else { record(&read_ones) };
    }
    clean(element)
}

/// Les données venues du serveur (`Data`, ADR-051) : un tableau de textes remplit une liste de
/// textes ; un tableau d'objets remplit une liste à champs (seuls les champs déclarés sont repris,
/// ceux qui manquent valent "" ou 0). Une liste que la page ne déclare pas est laissée de côté.
pub fn receive(program: &Program, lists: &Lists, json: &str) -> Lists {
    let mut lists = lists.clone();
    if crate::state::data_source(program).ok().flatten().is_none() || json.len() > crate::state::DATA_BYTES {
        return lists;
    }
    let Some(Json::Object(keys)) = Json::read(json) else { return lists };
    for (key, value) in keys {
        let (Json::Table(elements), Some(kind)) = (value, kind(program, &key)) else { continue };
        let Some((_, place)) = lists.iter_mut().find(|(n, _)| *n == key) else { continue };
        let mut new_ones = Vec::new();
        for element in elements.into_iter().take(ELEMENTS_MAX) {
            match (element, &kind) {
                (Json::Text(t), Kind::Texts | Kind::Free) => new_ones.push(clean(&t)),
                (Json::Object(read_fields), Kind::Records(_) | Kind::Free) => {
                    let taken: Vec<(String, String)> = match &kind {
                        Kind::Records(expected) => expected
                            .iter()
                            .map(|n| (n.clone(), read_fields.iter().find(|(c, _)| c == n).map(|(_, v)| v.as_text()).unwrap_or_default()))
                            .collect(),
                        _ => read_fields.iter().filter(|(n, _)| is_field_name(n)).map(|(n, v)| (n.clone(), v.as_text())).collect(),
                    };
                    if !taken.is_empty() {
                        new_ones.push(record(&taken));
                    }
                }
                _ => {}
            }
        }
        *place = new_ones.into_iter().filter(|e| !e.is_empty()).collect();
    }
    lists
}

/// Ce texte est-il un objet JSON, `{ "stock": 4 }`, que la page sait lire ?
pub fn is_json_object(json: &str) -> bool {
    json.len() <= crate::state::DATA_BYTES && matches!(Json::read(json), Some(Json::Object(_)))
}

/// Une valeur JSON, juste ce qu'il faut pour des données de page : textes, nombres entiers,
/// tableaux et objets, trois niveaux au plus. Le reste ne donne rien.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Json {
    Text(String),
    Number(u64),
    /// Un nombre à virgule, tel qu'écrit : « 12.50 » (ADR-066).
    Decimal(String),
    Table(Vec<Json>),
    Object(Vec<(String, Json)>),
    Other,
}

impl Json {
    pub(crate) fn read(text: &str) -> Option<Json> {
        let t: Vec<char> = text.chars().collect();
        let mut i = 0;
        let value = Self::value(&t, &mut i, 0)?;
        Self::blanks(&t, &mut i);
        (i == t.len()).then_some(value)
    }

    fn as_text(&self) -> String {
        match self {
            Json::Text(t) => t.clone(),
            Json::Number(n) => n.to_string(),
            Json::Decimal(d) => d.clone(),
            _ => String::new(),
        }
    }

    fn blanks(t: &[char], i: &mut usize) {
        while t.get(*i).is_some_and(|c| c.is_whitespace()) {
            *i += 1;
        }
    }

    fn value(t: &[char], i: &mut usize, depth: usize) -> Option<Json> {
        Self::blanks(t, i);
        match *t.get(*i)? {
            '"' => Self::text(t, i).map(Json::Text),
            '[' if depth < 3 => {
                *i += 1;
                let mut elements = Vec::new();
                loop {
                    Self::blanks(t, i);
                    if t.get(*i) == Some(&']') {
                        *i += 1;
                        return Some(Json::Table(elements));
                    }
                    elements.push(Self::value(t, i, depth + 1)?);
                    Self::blanks(t, i);
                    match *t.get(*i)? {
                        ',' => *i += 1,
                        ']' => {}
                        _ => return None,
                    }
                }
            }
            '{' if depth < 3 => {
                *i += 1;
                let mut keys = Vec::new();
                loop {
                    Self::blanks(t, i);
                    if t.get(*i) == Some(&'}') {
                        *i += 1;
                        return Some(Json::Object(keys));
                    }
                    let key = Self::text(t, i)?;
                    Self::blanks(t, i);
                    if t.get(*i) != Some(&':') {
                        return None;
                    }
                    *i += 1;
                    let value = Self::value(t, i, depth + 1)?;
                    keys.push((key, value));
                    Self::blanks(t, i);
                    match *t.get(*i)? {
                        ',' => *i += 1,
                        '}' => {}
                        _ => return None,
                    }
                }
            }
            c if c.is_ascii_digit() || c == '-' => {
                let start = *i;
                while t.get(*i).is_some_and(|c| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')) {
                    *i += 1;
                }
                let written: String = t[start..*i].iter().collect();
                let decimal = written.split_once('.').is_some_and(|(a, b)| !a.is_empty() && !b.is_empty() && a.chars().chain(b.chars()).all(|c| c.is_ascii_digit()));
                Some(if decimal { Json::Decimal(written) } else { written.parse::<u64>().map_or(Json::Other, Json::Number) })
            }
            _ => {
                for word in ["true", "false", "null"] {
                    if t[*i..].iter().take(word.len()).copied().eq(word.chars()) {
                        *i += word.len();
                        return Some(match word {
                            "true" => Json::Number(1),
                            "false" => Json::Number(0),
                            _ => Json::Other,
                        });
                    }
                }
                None
            }
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
}

/// Pour les conditions et les textes, une liste vaut son nombre d'éléments : `If(tasks, is: 0)`.
pub fn counts(lists: &Lists) -> State {
    lists.iter().map(|(name, elements)| (name.clone(), elements.len() as u64)).collect()
}

/// Les répétitions dynamiques du fichier, `Repeat(over: tasks, …)`, avec le nom de leur liste.
pub fn repeats(program: &Program) -> Vec<(&Block, String)> {
    let mut found_ones = Vec::new();
    let _ = for_each_block(&program.root, &mut |block| {
        if let (true, Some(Value::Name(list))) = (block.name == "Repeat", block.argument("over").map(|a| &a.value)) {
            found_ones.push((block, list.clone()));
        }
        Ok(())
    });
    found_ones
}

/// Les règles écrites dans une répétition dynamique : elles répondent au geste d'une ligne.
pub fn line_rules(program: &Program) -> Vec<(&Block, String)> {
    let mut rules = Vec::new();
    for (repeat, list) in repeats(program) {
        if let Some(Value::List(inside)) = repeat.argument("rules").map(|a| &a.value) {
            for rule in inside {
                if let Value::Block(b) = rule {
                    rules.push((b, list.clone()));
                }
            }
        }
    }
    rules
}

/// Les blocs du modèle d'une répétition dynamique (où `{item}` a un sens).
pub fn in_model(program: &Program) -> Vec<*const Block> {
    models_and_lists(program).into_iter().map(|(b, _)| b).collect()
}

/// Les blocs du modèle d'une répétition dynamique, avec le nom de la liste qu'elle montre.
pub fn models_and_lists(program: &Program) -> Vec<(*const Block, String)> {
    let mut blocks = Vec::new();
    for (repeat, list) in repeats(program) {
        for argument in &repeat.arguments {
            if argument.name.as_deref() == Some("children") || argument.name.as_deref() == Some("rules") {
                if let Value::List(inside) = &argument.value {
                    for v in inside {
                        if let Value::Block(b) = v {
                            let _ = for_each_block(b, &mut |x| {
                                blocks.push((x as *const Block, list.clone()));
                                Ok(())
                            });
                        }
                    }
                }
            }
        }
    }
    blocks
}

/// Vérifie une demande faite à une liste ou à un texte. `dans_une_ligne` : la règle est écrite
/// dans une répétition dynamique, où `item` désigne l'élément de la ligne.
pub fn check_request(request: &Block, program: &Program, rule: &Block, in_line: Option<&str>) -> Result<(), Error> {
    let error = |message: String| Error { message, pos: request.pos };
    let Some((value, verb)) = request.name.split_once('.') else { return Err(error("une demande s'écrit « tasks.push(task) »".into())) };
    if rule.name != "On" {
        return Err(error(format!("« {value}.{verb} » : une liste ou un texte ne change que par un geste du visiteur, dans une règle « On »")));
    }
    let texts = crate::state::initial_texts(program);
    let is_text = |n: &str| texts.iter().any(|(known, _)| known == n);
    // `item.done.set(1)` : changer un champ de l'élément de la ligne touchée (ADR-057).
    if value == "item" {
        let Some(list) = in_line else {
            return Err(error(format!("« item.{verb} » change l'élément d'une ligne : il s'écrit dans les règles de Repeat(over: …)")));
        };
        let Some((field, operation)) = verb.split_once('.') else {
            return Err(error("pour changer un champ de l'élément : item.done.set(1), item.likes.add(1)".into()));
        };
        match kind(program, list) {
            Some(Kind::Texts) => return Err(error(format!("les éléments de « {list} » sont des textes, sans champs"))),
            Some(Kind::Records(fields)) if !fields.iter().any(|c| c == field) => return Err(error(format!("les éléments de « {list} » n'ont pas de champ « {field} » ; champs : {}", fields.join(", ")))),
            _ => {}
        }
        let numbers = crate::state::initial(program).unwrap_or_default();
        return match (operation, request.arguments.as_slice()) {
            ("set", [Argument { name: None, value: Value::Integer(_) | Value::Text(_), .. }]) => Ok(()),
            ("set", [Argument { name: None, value: Value::Name(n), .. }]) if is_text(n) || numbers.iter().any(|(c, _)| c == n) => Ok(()),
            ("add" | "sub", [Argument { name: None, value: Value::Integer(_), .. }]) => Ok(()),
            _ => Err(error(format!("« item.{field}.{operation} » : on demande set (un nombre, un texte, ou une valeur de la page), add ou sub (un nombre entier)"))),
        };
    }
    if crate::computed::is_computed(program, value) {
        return Err(error(format!("« {value} » est une liste calculée : elle se refait d'après sa source, on ne la change pas par une demande ; change plutôt « {} »", crate::computed::source_of(program, value).unwrap_or_default())));
    }
    if is_list(program, value) {
        let the_kind = kind(program, value).unwrap_or(Kind::Free);
        let numbers = crate::state::initial(program).unwrap_or_default();
        let is_number = |n: &str| numbers.iter().any(|(known, _)| known == n);
        match (verb, request.arguments.as_slice()) {
            ("push", [Argument { name: None, value: Value::Name(t), .. }]) if is_text(t) && !matches!(the_kind, Kind::Records(_)) => Ok(()),
            ("push", [Argument { name: None, value: Value::Text(_), .. }]) if !matches!(the_kind, Kind::Records(_)) => Ok(()),
            ("push", [Argument { name: None, value: Value::Block(item), .. }]) if item.name == "Item" && the_kind != Kind::Texts => {
                let mut names = Vec::new();
                for a in &item.arguments {
                    let Some(name) = a.name.as_deref() else { return Err(error("chaque champ d'un « Item » est nommé : Item(title: task, price: 0)".into())) };
                    match &a.value {
                        Value::Text(t) if t.chars().count() <= ELEMENT_MAX => {}
                        Value::Integer(_) => {}
                        Value::Name(n) if is_text(n) || is_number(n) => {}
                        _ => return Err(error(format!("le champ « {name} » prend un texte, un nombre, ou le nom d'une valeur de la page"))),
                    }
                    names.push(name.to_string());
                }
                match &the_kind {
                    Kind::Records(expected) if *expected != names => Err(error(format!("« {value}.push(Item(…)) » : les éléments de « {value} » ont les champs {}, dans cet ordre", expected.join(", ")))),
                    _ if names.is_empty() || names.len() > FIELDS_MAX => Err(error(format!("un « Item » a de 1 à {FIELDS_MAX} champs"))),
                    _ => Ok(()),
                }
            }
            ("push", _) if matches!(the_kind, Kind::Records(_)) => Err(error(format!("« {value} » a des éléments à champs : {value}.push(Item(…))"))),
            ("push", _) => Err(error(format!("« {value}.push » attend un texte de la page, ou un texte entre guillemets : {value}.push(task)"))),
            ("remove", [Argument { name: None, value: Value::Name(item), .. }]) if item == "item" => match in_line {
                Some(list) if list == value => Ok(()),
                // Dans la répétition d'une liste calculée : on retire l'élément de sa source.
                Some(list) if crate::computed::is_computed(program, list) && crate::computed::source_of(program, list).as_deref() == Some(value) => Ok(()),
                _ => Err(error(format!("« {value}.remove(item) » s'écrit dans les règles de Repeat(over: {value}, …) : il retire l'élément de la ligne touchée"))),
            },
            ("remove", _) => Err(error(format!("« {value}.remove » retire l'élément d'une ligne : {value}.remove(item), dans Repeat(over: {value}, rules: [ … ])"))),
            ("clear", []) => Ok(()),
            ("clear", _) => Err(error(format!("« {value}.clear() » vide la liste : rien entre les parenthèses"))),
            _ => Err(error(format!("demande inconnue « {verb} » : pour une liste, on peut demander {}", REQUESTS.join(", ")))),
        }
    } else if is_text(value) {
        match (verb, request.arguments.as_slice()) {
            ("set", [Argument { name: None, value: Value::Text(t), .. }]) if t.chars().count() <= crate::state::TEXT_MAX => Ok(()),
            ("set", [Argument { name: None, value: Value::Name(other), .. }]) if is_text(other) || (other == "item" && in_line.is_some()) => Ok(()),
            // Une date avance ou recule de jours entiers : due.add(7), due.sub(1) (ADR-067).
            ("add" | "sub", [Argument { name: None, value: Value::Integer(n), .. }]) if crate::dates::is_date(program, value) && *n <= 3_660_000 => Ok(()),
            ("add" | "sub", _) if crate::dates::is_date(program, value) => Err(error(format!("« {value}.{verb} » ajoute ou retire des jours à une date : {value}.{verb}(7)"))),
            _ if crate::dates::is_date(program, value) => Err(error(format!("« {value} » est une date : on demande {value}.set(today), {value}.set(\"2026-12-24\"), {value}.add(7) ou {value}.sub(1)"))),
            _ => Err(error(format!("« {value} » est un texte : on demande seulement « {value}.set(\"\") », ou « {value}.set(autreTexte) »"))),
        }
    } else {
        Err(error(format!("aucune liste ni aucun texte ne s'appelle « {value} »")))
    }
}

/// Est-ce une demande faite à une liste ou à un texte ?
pub fn concerns(request: &Block, program: &Program) -> bool {
    request.name.split_once('.').is_some_and(|(value, _)| value == "item" || is_list(program, value) || crate::state::initial_texts(program).iter().any(|(t, _)| t == value))
}

/// Le signal d'une ligne : `Done.tap@2` → (`Done.tap`, Some(2)).
pub fn signal_and_line(signal: &str) -> (&str, Option<usize>) {
    match signal.split_once('@') {
        Some((base, rank)) => (base, rank.parse().ok()),
        None => (signal, None),
    }
}

/// Fait les demandes aux listes et aux textes que ce signal déclenche, dans l'ordre écrit.
pub fn arbitrate(program: &Program, numbers: &State, texts: &Texts, lists: &Lists, signal: &str) -> (Texts, Lists) {
    let (base, line) = signal_and_line(signal);
    let (mut texts, mut lists) = (texts.clone(), lists.clone());
    let of_line = line_rules(program);
    // Une règle écrite dans la répétition d'une liste calculée : la ligne touchée est un élément de
    // la liste calculée, une copie d'un élément de sa source (lot 2 du web).
    let computed = crate::computed::apply(program, numbers, &texts, &lists);
    let _ = for_each_block(&program.root, &mut |rule| {
        if rule.name != "On" || !matches!(rule.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value), Some(Value::Name(s)) if s == base) {
            return Ok(());
        }
        // Une règle de ligne ne répond qu'au geste d'une ligne, et connaît son élément.
        let line_list = of_line.iter().find(|(b, _)| std::ptr::eq(*b, rule)).map(|(_, l)| l.clone());
        if line_list.is_some() != line.is_some() {
            return Ok(());
        }
        let element = match (&line_list, line) {
            (Some(list), Some(rank)) => lists.iter().chain(computed.iter()).find(|(n, _)| n == list).and_then(|(_, e)| e.get(rank).cloned()),
            _ => None,
        };
        // Là où vit l'élément touché : sa liste déclarée, et son rang. Pour une liste calculée, sa
        // source, au rang de l'élément identique (autant d'identiques avant lui) : `item.done.set(1)`
        // et `tasks.remove(item)` changent l'élément d'origine (avant : rien ne changeait).
        let home: Option<(String, usize)> = match (&line_list, line, &element) {
            (Some(list), Some(rank), _) if !computed.iter().any(|(n, _)| n == list) => Some((list.clone(), rank)),
            (Some(list), Some(rank), Some(touched)) => {
                let before = computed.iter().find(|(n, _)| n == list).map_or(0, |(_, shown)| shown[..rank.min(shown.len())].iter().filter(|e| *e == touched).count());
                crate::computed::source_of(program, list).and_then(|source| {
                    let index = lists.iter().find(|(n, _)| *n == source)?.1.iter().enumerate().filter(|(_, e)| *e == touched).nth(before)?.0;
                    Some((source, index))
                })
            }
            _ => None,
        };
        for request in crate::state::requests_of(rule) {
            let Some((value, verb)) = request.name.split_once('.') else { continue };
            let argument = request.arguments.first().map(|a| &a.value);
            let read_text = |name: &str, texts: &Texts| -> Option<String> {
                if name == "item" {
                    return element.as_deref().map(text_of);
                }
                texts.iter().find(|(n, _)| n == name).map(|(_, t)| t.clone())
            };
            // `item.done.set(1)` : le champ de l'élément de la ligne touchée (ADR-057).
            if value == "item" {
                if let (Some((list, rank)), Some((field, operation))) = (&home, verb.split_once('.')) {
                    let rank = *rank;
                    let new_text = match argument {
                        Some(Value::Name(n)) => read_text(n, &texts).or_else(|| numbers.iter().find(|(c, _)| c == n).map(|(_, v)| v.to_string())),
                        _ => None,
                    };
                    if let Some((_, elements)) = lists.iter_mut().find(|(n, _)| n == list) {
                        if let Some(element) = elements.get_mut(rank) {
                            let mut fields_of = fields(element);
                            if !fields_of.iter().any(|(c, _)| c == field) {
                                fields_of.push((field.to_string(), String::new()));
                            }
                            if let Some((_, v)) = fields_of.iter_mut().find(|(c, _)| c == field) {
                                let present = v.parse::<u64>().unwrap_or(0);
                                *v = match (operation, argument) {
                                    ("set", Some(Value::Integer(n))) => n.to_string(),
                                    ("set", Some(Value::Text(t))) => t.clone(),
                                    ("set", Some(Value::Name(_))) => new_text.unwrap_or_default(),
                                    ("add", Some(Value::Integer(n))) => present.saturating_add(*n).min(crate::state::VALUE_MAX).to_string(),
                                    ("sub", Some(Value::Integer(n))) => present.saturating_sub(*n).to_string(),
                                    _ => v.clone(),
                                };
                            }
                            *element = record(&fields_of);
                        }
                    }
                }
                continue;
            }
            if let Some((_, elements)) = lists.iter_mut().find(|(n, _)| n == value) {
                match (verb, argument) {
                    ("push", Some(Value::Name(t))) => {
                        if let Some(text) = read_text(t, &texts).map(|t| clean(t.trim())).filter(|t| !t.is_empty()) {
                            if elements.len() < ELEMENTS_MAX {
                                elements.push(text);
                            }
                        }
                    }
                    ("push", Some(Value::Text(t))) if elements.len() < ELEMENTS_MAX && !t.is_empty() => elements.push(clean(t)),
                    ("push", Some(Value::Block(item))) if item.name == "Item" && elements.len() < ELEMENTS_MAX => {
                        let placed_fields: Vec<(String, String)> = item
                            .arguments
                            .iter()
                            .filter_map(|a| {
                                let value = match &a.value {
                                    Value::Text(t) => t.clone(),
                                    Value::Integer(e) => e.to_string(),
                                    Value::Name(n) => read_text(n, &texts).or_else(|| numbers.iter().find(|(c, _)| c == n).map(|(_, v)| v.to_string())).unwrap_or_default(),
                                    _ => return None,
                                };
                                Some((a.name.clone()?, value.trim().to_string()))
                            })
                            .collect();
                        // Un élément dont tous les textes sont vides n'est pas ajouté, comme un texte vide.
                        if placed_fields.iter().any(|(_, v)| !v.is_empty() && v != "0") {
                            elements.push(record(&placed_fields));
                        }
                    }
                    ("remove", _) => {
                        if let Some((list, rank)) = &home {
                            if list == value && *rank < elements.len() {
                                elements.remove(*rank);
                            }
                        }
                    }
                    ("clear", _) => elements.clear(),
                    _ => {}
                }
            } else if verb == "set" {
                let new_one = match argument {
                    Some(Value::Text(t)) => Some(t.clone()),
                    Some(Value::Name(other)) => read_text(other, &texts),
                    _ => None,
                };
                if let (Some(new_one), Some((_, place))) = (new_one, texts.iter_mut().find(|(n, _)| n == value)) {
                    *place = new_one.chars().filter(|c| *c == '\n' || !c.is_control()).take(crate::state::TEXT_MAX).collect();
                }
            } else if let (true, Some(Value::Integer(n))) = (verb == "add" || verb == "sub", argument) {
                // Une date avance ou recule de jours entiers (ADR-067) ; une date vide ne bouge pas.
                let shift = if verb == "add" { *n as i64 } else { -(*n as i64) };
                if let Some((_, place)) = texts.iter_mut().find(|(n, _)| n == value) {
                    if let Some(moved) = crate::dates::shifted(place, shift) {
                        *place = moved;
                    }
                }
            }
        }
        Ok(())
    });
    (texts, lists)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_list_changes_through_gestures() {
        let source = "Page(state: State(task: \"\", tasks: [\"Pain\"]), children: [ Input(value: task, label: \"T\"), Button(name: Add, text: \"+\"), Button(name: Empty, text: \"0\"), P(\"{tasks} tâche(s)\"), If(tasks, is: 0, children: [ P(\"Rien\") ]), Repeat(over: tasks, children: [ Row(children: [ Text(\"{item}\"), Button(name: Done, text: \"x\") ]) ], rules: [ On(Done.tap, effect: tasks.remove(item)) ]) ], rules: [ On(Add.tap, effect: [tasks.push(task), task.set(\"\")]), On(Empty.tap, effect: tasks.clear()) ])";
        let start_value = crate::initial_state(source);
        assert_eq!(start_value, "task=';tasks=[Pain]");
        let written = crate::input(source, &start_value, "task", "Lait, frais");
        let after = crate::arbitrate(source, &written, "Add.tap");
        assert_eq!(after, "task=';tasks=[Pain,Lait%2C%20frais]");
        // Le geste d'une ligne retire son élément ; sans ligne, la règle de ligne ne répond pas.
        assert_eq!(crate::arbitrate(source, &after, "Done.tap@0"), "task=';tasks=[Lait%2C%20frais]");
        assert_eq!(crate::arbitrate(source, &after, "Done.tap"), after);
        assert_eq!(crate::arbitrate(source, &after, "Empty.tap"), "task=';tasks=[]");
        // La page fabriquée montre une ligne par élément ; le compte, et la condition.
        let html = crate::flat_view(source, "").unwrap();
        assert!(html.contains("<div class=\"holo-Lines\" data-list=\"tasks\" data-repeat=\"") && html.contains("\"><div class=\"holo-line\" data-rank=\"0\" data-key=\"") && html.contains("-0\"><div class=\"holo-Row\" style=\"\"><div class=\"holo-Text\">Pain</div><button type=\"button\" class=\"holo-Button\" data-name=\"Done\">x</button></div></div></div>"), "{html}");
        // La clé d'un élément ne change pas quand un autre élément est retiré avant lui.
        let key_of = |state: &str, text: &str| crate::list_html(source, "", state, "tasks").split("data-key=\"").skip(1).find(|l| l.contains(text)).map(|l| l[..l.find('"').unwrap()].to_string());
        assert_eq!(key_of(&after, "Lait"), key_of(&crate::arbitrate(source, &after, "Done.tap@0"), "Lait"));
        assert!(html.contains("<span data-state=\"tasks\">1</span> tâche(s)"), "{html}");
        assert_eq!(crate::list_html(source, "", &after, "tasks").matches("data-rank").count(), 2);
        // Un texte saisi par le visiteur ne devient jamais une balise, ni une valeur montrée.
        let trap = crate::arbitrate(source, &crate::input(source, &start_value, "task", "<b>{task}</b>"), "Add.tap");
        let lines = crate::list_html(source, "", &trap, "tasks");
        assert!(lines.contains("&lt;b&gt;{task}&lt;/b&gt;") && !lines.contains("data-state"), "{lines}");
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
            let error = crate::check_page(source).err().or_else(|| crate::flat_view(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn two_repeats_of_one_list_keep_their_own_lines() {
        // Avant : après un changement, la seconde répétition recevait les lignes de la première.
        let source = "Page(
  state: State(tasks: [\"Pain\", \"Lait\"]),
  children: [
    Repeat(over: tasks, children: [ Text(\"A: {item}\") ]),
    Repeat(over: tasks, children: [ Text(\"B: {item}\") ]),
  ],
)";
        let html = crate::flat_view(source, "").unwrap();
        assert!(html.contains(r#"data-list="tasks" data-repeat="4:5">"#) && html.contains(r#"data-list="tasks" data-repeat="5:5">"#), "{html}");
        let state = crate::initial_state(source);
        let first = crate::list_html(source, "", &state, "tasks@4:5");
        let second = crate::list_html(source, "", &state, "tasks@5:5");
        assert!(first.contains("A: Pain") && !first.contains("B:"), "{first}");
        assert!(second.contains("B: Pain") && second.contains("B: Lait") && !second.contains("A:"), "{second}");
        // Sans place, la première, comme avant.
        assert_eq!(crate::list_html(source, "", &state, "tasks"), first);
    }

    #[test]
    fn a_chosen_key_stays_when_the_element_changes() {
        // `key: id` : la clé de la ligne est ce champ ; elle reste quand un autre champ change.
        let source = r#"Page(
  state: State(tasks: [ Item(id: "t1", title: "Pain", done: 0), Item(id: "t2", title: "Lait", done: 0) ]),
  children: [
    Repeat(over: tasks, key: id, children: [ Text("{item.title}"), Button(name: Done, text: "Fait") ],
           rules: [ On(Done.tap, effect: item.done.set(1)) ]),
  ],
)"#;
        let start = crate::initial_state(source);
        let before = crate::list_html(source, "", &start, "tasks");
        assert!(before.contains(r#"data-rank="0" data-key="k:t1-0""#) && before.contains(r#"data-rank="1" data-key="k:t2-0""#), "{before}");
        let after = crate::list_html(source, "", &crate::arbitrate(source, &start, "Done.tap@0"), "tasks");
        assert!(after.contains(r#"data-key="k:t1-0""#), "{after}");
        // Sans clé choisie, la clé suit le contenu : elle change avec lui.
        let plain = source.replace("key: id, ", "");
        let (a, b) = (crate::list_html(&plain, "", &start, "tasks"), crate::list_html(&plain, "", &crate::arbitrate(&plain, &start, "Done.tap@0"), "tasks"));
        let key = |html: &str| html.split("data-key=\"").nth(1).unwrap().split('"').next().unwrap().to_string();
        assert_ne!(key(&a), key(&b));
        // Les refus : un champ qui n'existe pas, une liste de textes, autre chose qu'un nom.
        for (wrong, message) in [
            ("key: ref,", "n'ont pas de champ « ref »"),
            ("key: \"id\",", "attend le nom d'un champ"),
        ] {
            let error = crate::check_page(&source.replace("key: id,", wrong)).unwrap_err();
            assert!(error.message.contains(message), "{wrong}\n→ {error}");
        }
        let texts = "Page(state: State(tasks: [\"Pain\"]), children: [ Repeat(over: tasks, key: title, children: [ Text(\"{item}\") ]) ])";
        assert!(crate::check_page(texts).unwrap_err().message.contains("sont des textes"));
    }

    #[test]
    fn a_list_with_fields_is_shown_filled_and_comes_from_the_server() {
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
        crate::check_page(source).unwrap();
        let start_value = crate::initial_state(source);
        let html = crate::flat_view(source, "").unwrap();
        assert!(html.contains("<img class=\"holo-Image\" src=\"sunrise.png\" alt=\"Sunrise\">"), "{html}");
        assert!(html.contains("Sunrise : 120,00 euros") && html.contains("River : 9,00 euros"), "{html}");
        assert!(html.contains("<span data-state=\"articles\">2</span> article(s)"), "{html}");
        // Ajouter un élément à champs, depuis ce que le visiteur a saisi ; puis en retirer un.
        let written = crate::input(source, &crate::input(source, &start_value, "name", "<Night>"), "price", "60");
        let after = crate::arbitrate(source, &written, "Add.tap");
        let lines = crate::list_html(source, "", &after, "articles");
        assert_eq!(lines.matches("data-rank").count(), 3, "{lines}");
        assert!(lines.contains("&lt;Night&gt; : 0,60 euros"), "{lines}");
        let less = crate::arbitrate(source, &after, "Remove.tap@0");
        assert_eq!(crate::list_html(source, "", &less, "articles").matches("data-rank").count(), 2);
        // Les données du serveur remplacent la liste : seuls les champs déclarés sont repris.
        let received = crate::receive(source, &start_value, r#"{"articles": [ {"title": "Forest", "price": 4500, "image": "f.png", "secret": "x"} ]}"#);
        let lines = crate::list_html(source, "", &received, "articles");
        assert!(lines.contains("Forest : 45,00 euros") && lines.contains("src=\"f.png\"") && !lines.contains("secret"), "{lines}");
        assert_eq!(lines.matches("data-rank").count(), 1);
        // Une image venue du serveur reste dans le dossier de la page. Une image refusée, ou
        // absente, ne retire que l'image : la ligne garde son texte (avant : toute la liste
        // disparaissait, défaut D10).
        let trap = crate::receive(source, &start_value, r#"{"articles": [ {"title": "Trap", "price": 1, "image": "javascript:alert(1)"}, {"title": "Up", "price": 2, "image": "../x.svg"}, {"title": "None", "price": 3}, {"title": "Fine", "price": 4, "image": "fine.png"} ]}"#);
        let lines = crate::list_html(source, "", &trap, "articles");
        assert!(!lines.contains("javascript") && !lines.contains("x.svg"), "{lines}");
        assert_eq!(lines.matches("data-rank").count(), 4, "{lines}");
        assert!(lines.contains("class=\"holo-Text\">Trap</div>") && lines.contains("Trap : 0,01 euros"), "{lines}");
        assert!(lines.contains("src=\"fine.png\" alt=\"Fine\""), "{lines}");
    }

    #[test]
    fn a_list_of_texts_comes_from_the_server() {
        let source = "Page(state: State(news: []), data: Data(from: \"news.json\"), children: [ Repeat(over: news, children: [ P(\"{item}\") ]) ])";
        let received = crate::receive(source, &crate::initial_state(source), r#"{"news": ["Open today", "New paintings"]}"#);
        let lines = crate::list_html(source, "", &received, "news");
        assert!(lines.contains("Open today") && lines.contains("New paintings"), "{lines}");
    }

    #[test]
    fn what_is_refused_for_lists_with_fields() {
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
            let error = crate::check_page(source).err().or_else(|| crate::flat_view(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn a_condition_and_a_change_on_a_line_field() {
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
        crate::check_page(source).unwrap();
        let start_value = crate::initial_state(source);
        let lines = crate::list_html(source, "", &start_value, "tasks");
        assert!(lines.contains(">Pain</div>") && lines.contains("holo-s-fait\">Lait ✓</div>"), "{lines}");
        let after = crate::arbitrate(source, &start_value, "Done.tap@0");
        let lines = crate::list_html(source, "", &after, "tasks");
        assert!(lines.contains("holo-s-fait\">Pain ✓</div>"), "{lines}");
        // Dans une répétition fixe aussi.
        let fixed = "Page(children: [ H1(\"x\"), Repeat(items: [ Item(t: \"a\", n: 3), Item(t: \"b\", n: 0) ], children: [ If(item.n, over: 0, children: [ P(\"{item.t} en stock\") ], else: [ P(\"{item.t} épuisé\") ]) ]) ])";
        let html = crate::flat_view(fixed, "").unwrap();
        assert!(html.contains("a en stock") && html.contains("b épuisé"), "{html}");
        for (source, message) in [
            ("Page(state: State(l: [ Item(a: 1) ]), children: [ Repeat(over: l, children: [ If(item.b, is: 1, children: [ P(\"x\") ]) ]) ])", "pas de champ « b »"),
            ("Page(children: [ If(item.a, is: 1, children: [ P(\"x\") ]) ])", "dans Repeat"),
            ("Page(state: State(l: [ Item(a: 1) ]), children: [ Repeat(over: l, children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: item.z.set(1)) ]) ])", "pas de champ « z »"),
            ("Page(state: State(l: [ Item(a: 1) ]), children: [ Repeat(over: l, children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: item.a.mul(2)) ]) ])", "on demande set"),
        ] {
            let error = crate::check_page(source).err().or_else(|| crate::flat_view(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }
}
