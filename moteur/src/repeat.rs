//! La répétition (ADR-040) : un bloc écrit une fois, posé pour chaque élément d'une liste.
//!
//! ```holo
//! Repeat(
//!   items: [
//!     Item(key: sunrise, title: "Sunrise", image: "sunrise.png"),
//!     Item(key: river, title: "The river", image: "river.png"),
//!   ],
//!   children: [ Column(children: [ Image(source: item.image, alt: "{item.title}"), H2("{item.title}"), Button(name: Add, text: "Add") ]) ],
//!   rules: [ On(Add.tap, effect: item.add(1)) ],
//! )
//! ```
//!
//! Comme `Use`, la répétition est dépliée à la lecture : le reste du moteur ne voit que les blocs
//! qu'on aurait écrits à la main. Dans le modèle, `item` désigne l'élément :
//! - `{item.title}` dans un texte devient le champ ; `{item}` montre la valeur de la page qui
//!   porte le nom de la clé (`{sunrise}`) ;
//! - `item.image` à la place d'une valeur devient le champ lui-même ;
//! - `item`, et `item.add(1)`, désignent la valeur de la page qui porte le nom de la clé ;
//! - un bloc nommé reçoit le nom de son élément : `Add` devient `AddSunrise`, partout dans le
//!   modèle et dans ses règles. Ses règles rejoignent celles de la page (ou du monde).

use crate::holo::{Argument, Block, Error, Value};

/// Le nombre d'éléments d'une répétition, au plus.
pub const ELEMENTS_MAX: usize = 200;
/// Le nombre de blocs qu'une page peut obtenir en dépliant ses répétitions, au plus.
pub const BLOCKS_MAX: usize = 20_000;

/// Les mots du langage qu'un champ ne peut pas porter : `item.add` doit rester une demande.
const RESERVED_WORDS: &[&str] = &["key", "add", "sub", "set", "random", "enter", "leave", "play", "portals", "tap", "hover", "hoverEnd"];

/// Déplie les répétitions d'un site (une page, ou un monde) et de tout ce qu'il contient.
pub fn unfold_site(site: &mut Block, count: &mut usize) -> Result<(), Error> {
    let mut rules = Vec::new();
    for argument in &mut site.arguments {
        unfold_value(&mut argument.value, &mut rules, count)?;
    }
    if rules.is_empty() {
        return Ok(());
    }
    match site.arguments.iter_mut().find(|a| a.name.as_deref() == Some("rules")) {
        Some(Argument { value: Value::List(list), .. }) => list.extend(rules),
        Some(argument) => return Err(Error { message: "« rules » est une liste de règles : rules: [ … ]".into(), pos: argument.pos }),
        None => site.arguments.push(Argument { name: Some("rules".into()), value: Value::List(rules), pos: site.pos }),
    }
    Ok(())
}

fn unfold_value(value: &mut Value, rules: &mut Vec<Value>, count: &mut usize) -> Result<(), Error> {
    match value {
        // Le monde d'un point est un autre site : ses règles restent chez lui.
        Value::Block(block) if block.name == "World" => unfold_site(block, count),
        Value::Block(block) if block.name == "Repeat" && block.argument("over").is_none() => {
            Err(Error { message: "une répétition se pose parmi des blocs : children: [ Repeat(items: [ … ], children: [ … ]) ]".into(), pos: block.pos })
        }
        Value::Block(block) => block.arguments.iter_mut().try_for_each(|a| unfold_value(&mut a.value, rules, count)),
        Value::List(elements) => {
            let mut placed_list = Vec::with_capacity(elements.len());
            for mut element in std::mem::take(elements) {
                match element {
                    // Une répétition dynamique (`over:`) reste telle quelle : la page la redessine (ADR-044).
                    Value::Block(repeat) if repeat.name == "Repeat" && repeat.argument("over").is_none() => {
                        let (children, repeat_rules) = unfold_repeat(&repeat)?;
                        for mut child in children {
                            unfold_value(&mut child, rules, count)?;
                            *count += size(&child);
                            placed_list.push(child);
                        }
                        rules.extend(repeat_rules);
                        if *count > BLOCKS_MAX {
                            return Err(Error { message: format!("les répétitions de la page donnent plus de {BLOCKS_MAX} blocs : c'est trop pour une page"), pos: repeat.pos });
                        }
                    }
                    _ => {
                        unfold_value(&mut element, rules, count)?;
                        placed_list.push(element);
                    }
                }
            }
            *elements = placed_list;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Le nombre de blocs d'une valeur.
fn size(value: &Value) -> usize {
    match value {
        Value::Block(block) => 1 + block.arguments.iter().map(|a| size(&a.value)).sum::<usize>(),
        Value::List(elements) => elements.iter().map(size).sum(),
        _ => 0,
    }
}

/// Un élément de la liste : sa clé (le nom d'une valeur de la page) et ses champs.
struct Element<'a> {
    key: Option<&'a str>,
    fields: Vec<(&'a str, &'a Value)>,
    pos: crate::holo::Pos,
}

/// Les enfants et les règles d'une répétition, écrits une fois par élément.
fn unfold_repeat(repeat: &Block) -> Result<(Vec<Value>, Vec<Value>), Error> {
    let example = "Repeat(items: [ Item(title: \"Sunrise\"), Item(title: \"River\") ], children: [ H2(\"{item.title}\") ])";
    let (mut elements, mut model, mut rules) = (None, None, Vec::new());
    for argument in &repeat.arguments {
        match (argument.name.as_deref(), &argument.value) {
            (Some("items"), Value::List(list)) => elements = Some((list, argument.pos)),
            (Some("children"), Value::List(list)) => model = Some(list),
            (Some("rules"), Value::List(list)) => rules = list.clone(),
            (Some(word @ ("items" | "children" | "rules")), _) => return Err(Error { message: format!("« Repeat({word}: …) » attend une liste entre crochets : {example}"), pos: argument.pos }),
            (Some(other), _) => return Err(Error { message: format!("« Repeat » n'a pas de paramètre « {other} » ; paramètres possibles : items, children, rules"), pos: argument.pos }),
            (None, _) => return Err(Error { message: format!("chaque paramètre de « Repeat » est nommé : {example}"), pos: argument.pos }),
        }
    }
    let (Some((list, list_pos)), Some(model)) = (elements, model) else {
        return Err(Error { message: format!("« Repeat » attend « items » et « children » : {example}"), pos: repeat.pos });
    };
    if list.is_empty() || list.len() > ELEMENTS_MAX {
        return Err(Error { message: format!("« Repeat(items: …) » attend de 1 à {ELEMENTS_MAX} éléments"), pos: list_pos });
    }
    if contains_repeat(model) || rules.iter().any(|r| contains_repeat(std::slice::from_ref(r))) {
        return Err(Error { message: "une répétition dans une répétition n'est pas permise : déplie la liste du dedans à part".into(), pos: repeat.pos });
    }
    for rule in &rules {
        if !matches!(rule, Value::Block(b) if matches!(b.name.as_str(), "On" | "Every" | "When" | "After" | "If")) {
            return Err(Error { message: "« Repeat(rules: …) » range des règles : rules: [ On(Add.tap, effect: item.add(1)) ]".into(), pos: repeat.pos });
        }
    }
    // Les éléments.
    let mut read_ones: Vec<Element> = Vec::new();
    for value in list {
        let Value::Block(item) = value else {
            return Err(Error { message: "« items » contient des « Item(…) » : items: [ Item(title: \"Sunrise\") ]".into(), pos: list_pos });
        };
        if item.name != "Item" {
            return Err(Error { message: format!("« items » contient des « Item(…) », pas des « {} »", item.name), pos: item.pos });
        }
        let mut element = Element { key: None, fields: Vec::new(), pos: item.pos };
        for argument in &item.arguments {
            let Some(name) = argument.name.as_deref() else {
                return Err(Error { message: "chaque champ d'un « Item » est nommé : Item(title: \"Sunrise\", price: 120)".into(), pos: argument.pos });
            };
            match (name, &argument.value) {
                ("key", Value::Name(key)) if key.starts_with(|c: char| c.is_ascii_lowercase()) && key.chars().all(|c| c.is_ascii_alphanumeric()) => element.key = Some(key),
                ("key", _) => return Err(Error { message: "« key » est le nom d'une valeur de la page, comme sunrise : Item(key: sunrise, …)".into(), pos: argument.pos }),
                (word, _) if RESERVED_WORDS.contains(&word) => return Err(Error { message: format!("« {word} » est un mot du langage ; choisis un autre nom de champ"), pos: argument.pos }),
                (_, Value::Text(_) | Value::Integer(_) | Value::Number { .. } | Value::Name(_) | Value::Bool(_)) => {
                    if !name.starts_with(|c: char| c.is_ascii_lowercase()) || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
                        return Err(Error { message: format!("« {name} » : un champ s'écrit comme une valeur, en minuscules, les mots joints par une majuscule (ADR-037)"), pos: argument.pos });
                    }
                    if element.fields.iter().any(|(known, _)| *known == name) {
                        return Err(Error { message: format!("le champ « {name} » est donné deux fois"), pos: argument.pos });
                    }
                    element.fields.push((name, &argument.value));
                }
                _ => return Err(Error { message: format!("le champ « {name} » attend un texte, un nombre ou un mot, pas un bloc ni une liste"), pos: argument.pos }),
            }
        }
        if let Some(key) = element.key {
            if read_ones.iter().any(|other| other.key == Some(key)) {
                return Err(Error { message: format!("deux éléments ont la clé « {key} »"), pos: item.pos });
            }
        }
        read_ones.push(element);
    }
    // Les noms donnés dans le modèle : chacun reçoit le nom de son élément.
    let mut names = Vec::new();
    for value in model.iter().chain(rules.iter()) {
        given_names(value, &mut names);
    }
    let mut children = Vec::new();
    let mut unfolded_rules = Vec::new();
    for element in &read_ones {
        // Les `If(item.champ, …)` choisissent leur branche d'après l'élément (ADR-057).
        let fields: Vec<(String, String)> = element
            .fields
            .iter()
            .map(|(n, v)| {
                let text = match v {
                    Value::Text(t) => t.clone(),
                    Value::Integer(e) => e.to_string(),
                    Value::Name(m) => m.clone(),
                    Value::Bool(b) => u8::from(*b).to_string(),
                    Value::Number { value, .. } => value.to_string(),
                    _ => String::new(),
                };
                (n.to_string(), text)
            })
            .collect();
        let mut chosen_model = Value::List(model.clone());
        crate::lists::choose_by_element(&mut chosen_model, &fields, "");
        let mut chosen_rules = Value::List(rules.clone());
        crate::lists::choose_by_element(&mut chosen_rules, &fields, "");
        let (Value::List(chosen_model), Value::List(chosen_rules)) = (chosen_model, chosen_rules) else { unreachable!() };
        for value in &chosen_model {
            let mut copy = value.clone();
            replace(&mut copy, element, &names, repeat.pos)?;
            children.push(copy);
        }
        for value in &chosen_rules {
            let mut copy = value.clone();
            replace(&mut copy, element, &names, repeat.pos)?;
            unfolded_rules.push(copy);
        }
    }
    Ok((children, unfolded_rules))
}

fn contains_repeat(values: &[Value]) -> bool {
    values.iter().any(|value| match value {
        Value::Block(block) => block.name == "Repeat" || block.arguments.iter().any(|a| contains_repeat(std::slice::from_ref(&a.value))),
        Value::List(elements) => contains_repeat(elements),
        _ => false,
    })
}

fn given_names(value: &Value, names: &mut Vec<String>) {
    match value {
        Value::Block(block) => {
            if let Some(Value::Name(name)) = block.argument("name").map(|a| &a.value) {
                if !names.contains(name) {
                    names.push(name.clone());
                }
            }
            block.arguments.iter().for_each(|a| given_names(&a.value, names));
        }
        Value::List(elements) => elements.iter().for_each(|e| given_names(e, names)),
        _ => {}
    }
}

/// `sunrise` → `Sunrise` ; `blueDoor` → `BlueDoor`.
fn uppercased(key: &str) -> String {
    let mut letters = key.chars();
    letters.next().map(|c| c.to_ascii_uppercase().to_string() + letters.as_str()).unwrap_or_default()
}

/// Écrit le modèle pour un élément.
fn replace(value: &mut Value, element: &Element, names: &[String], pos: crate::holo::Pos) -> Result<(), Error> {
    let without_key = |what: &str| Error {
        message: format!("{what} : il faut une clé à chaque élément, le nom d'une valeur de la page, Item(key: sunrise, …)"),
        pos: element.pos,
    };
    let field = |name: &str| element.fields.iter().find(|(known, _)| *known == name).map(|(_, v)| *v);
    match value {
        Value::Text(text) => *text = replace_in_text(text, element)?,
        Value::Name(name) => {
            let key = || element.key.ok_or_else(|| without_key("« item » désigne la valeur de l'élément"));
            if name == "item" || name == "item.key" {
                *name = key()?.to_string();
            } else if let Some(field_name) = name.strip_prefix("item.").map(str::to_string) {
                match field(&field_name) {
                    Some(replacement_) => *value = replacement_.clone(),
                    // `item.enter` : une capacité demandée à la valeur de l'élément.
                    None if RESERVED_WORDS.contains(&field_name.as_str()) => *name = format!("{}.{field_name}", key()?),
                    None => return Err(Error { message: format!("l'élément n'a pas de champ « {field_name} » : Item({field_name}: …)"), pos: element.pos }),
                }
            } else {
                // Un nom donné dans le modèle, seul (`name: Add`) ou suivi d'un signal (`Add.tap`).
                let (start, suite) = name.split_once('.').map_or((name.as_str(), None), |(a, b)| (a, Some(b)));
                if names.iter().any(|n| n == start) {
                    let key = element.key.ok_or_else(|| without_key(&format!("« {start} » est nommé dans la répétition, et chaque copie doit avoir son propre nom")))?;
                    let new_one = format!("{start}{}", uppercased(key));
                    *name = match suite {
                        Some(suite) => format!("{new_one}.{suite}"),
                        None => new_one,
                    };
                }
            }
        }
        Value::List(elements) => {
            for e in elements {
                replace(e, element, names, pos)?;
            }
        }
        Value::Block(block) => {
            // Une demande faite à la valeur de l'élément : `item.add(1)`.
            if let Some(verb) = block.name.strip_prefix("item.") {
                block.name = format!("{}.{verb}", element.key.ok_or_else(|| without_key("« item » désigne la valeur de l'élément"))?);
            }
            if block.name == "Item" {
                return Err(Error { message: "« Item » se place dans « items », pas dans le modèle".into(), pos: block.pos });
            }
            for argument in &mut block.arguments {
                replace(&mut argument.value, element, names, pos)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// `{item.title}` devient le champ ; `{item}`, la valeur de la page qui porte la clé.
fn replace_in_text(text: &str, element: &Element) -> Result<String, Error> {
    if !text.contains("{item") {
        return Ok(text.to_string());
    }
    let mut output = String::with_capacity(text.len());
    let mut remainder = text;
    while let Some(start) = remainder.find("{item") {
        output.push_str(&remainder[..start]);
        let after = &remainder[start + 1..];
        let Some(end) = after.find('}') else {
            output.push_str(&remainder[start..]);
            remainder = "";
            break;
        };
        let inside = &after[..end];
        if inside == "item" {
            let key = element.key.ok_or_else(|| Error { message: "« {item} » montre la valeur de l'élément : il faut une clé, Item(key: sunrise, …)".into(), pos: element.pos })?;
            output.push_str(&format!("{{{key}}}"));
        } else if let Some((name, format)) = inside.strip_prefix("item.").and_then(|n| n.split_once(':')) {
            // `{item.price:cents}` : un champ nombre, écrit avec son format (ADR-043).
            let Some(Value::Integer(n)) = element.fields.iter().find(|(known, _)| *known == name).map(|(_, v)| *v) else {
                return Err(Error { message: format!("« {{item.{name}:{format}}} » : le champ « {name} » doit être un nombre entier"), pos: element.pos });
            };
            if !crate::format::is_format(format) || format == "name" {
                return Err(Error { message: format!("« {{item.{name}:{format}}} » : format inconnu ; pour un champ : 00, number, cents"), pos: element.pos });
            }
            output.push_str(&crate::format::format_value(name, *n, format, &crate::format::language()));
        } else if let Some(name) = inside.strip_prefix("item.") {
            match element.fields.iter().find(|(known, _)| *known == name).map(|(_, v)| *v) {
                Some(Value::Text(t)) => output.push_str(t),
                Some(Value::Integer(n)) => output.push_str(&n.to_string()),
                Some(Value::Number { value, unit }) => output.push_str(&format!("{value}{}", unit.as_deref().unwrap_or(""))),
                Some(Value::Name(n)) => output.push_str(n),
                Some(Value::Bool(b)) => output.push_str(if *b { "true" } else { "false" }),
                Some(_) | None if name == "key" && element.key.is_some() => output.push_str(element.key.unwrap_or_default()),
                _ => return Err(Error { message: format!("l'élément n'a pas de champ « {name} » : Item({name}: …)"), pos: element.pos }),
            }
        } else {
            // `{itemCount}` : une valeur de la page dont le nom commence par « item ».
            output.push_str(&remainder[start..start + 1 + end + 1]);
        }
        remainder = &after[end + 1..];
    }
    output.push_str(remainder);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::holo::read;

    #[test]
    fn a_repeat_is_unfolded_when_read() {
        let source = "Page(state: State(sunrise: 0, river: 0), children: [ H1(\"Shop\"), Grid(children: [ Repeat(items: [ Item(key: sunrise, title: \"Sunrise\", image: \"s.png\"), Item(key: river, title: \"River\", image: \"r.png\") ], children: [ Column(children: [ Image(source: item.image, alt: \"{item.title}\"), H2(\"{item.title} ({item})\"), Button(name: Add, text: \"Add\") ]) ], rules: [ On(Add.tap, effect: item.add(1)) ]) ]) ])";
        let program = read(source).unwrap();
        let html = crate::flat::page_html(&program, "").unwrap();
        assert!(html.contains("<img class=\"holo-Image\" src=\"s.png\" alt=\"Sunrise\"><h2 class=\"holo-H2\">Sunrise (<span data-state=\"sunrise\">0</span>)</h2><button type=\"button\" class=\"holo-Button\" data-name=\"AddSunrise\">Add</button>"), "{html}");
        assert!(html.contains("data-name=\"AddRiver\""), "{html}");
        crate::check_page(source).unwrap();
        assert_eq!(crate::arbitrate(source, "sunrise=0;river=0", "AddRiver.tap"), "sunrise=0;river=1");
    }

    #[test]
    fn what_is_refused() {
        for (source, message) in [
            ("Page(children: [ Repeat(items: [], children: [ P(\"x\") ]) ])", "de 1 à 200"),
            ("Page(children: [ Repeat(children: [ P(\"x\") ]) ])", "attend « items » et « children »"),
            ("Page(children: [ Repeat(items: [ Item(title: \"a\") ], children: [ P(\"{item.price}\") ]) ])", "pas de champ « price »"),
            ("Page(children: [ Repeat(items: [ Item(title: \"a\") ], children: [ Button(name: B, text: \"b\") ]) ])", "il faut une clé"),
            ("Page(children: [ Repeat(items: [ Item(key: a), Item(key: a) ], children: [ P(\"x\") ]) ])", "deux éléments ont la clé"),
            ("Page(children: [ Repeat(items: [ Item(add: 1) ], children: [ P(\"x\") ]) ])", "est un mot du langage"),
            ("Page(children: [ Repeat(items: [ Item(t: 1) ], children: [ Repeat(items: [ Item(u: 1) ], children: [ P(\"x\") ]) ]) ])", "dans une répétition n'est pas permise"),
            ("Page(children: [ Repeat(items: [ Item(t: 1) ], children: [ P(\"x\") ], colour: 3) ])", "n'a pas de paramètre « colour »"),
            ("Page(children: [ Item(t: 1) ])", "élément d'une répétition"),
            ("Page(children: [ P(\"{item.title}\") ])", "dans une répétition"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }
}
