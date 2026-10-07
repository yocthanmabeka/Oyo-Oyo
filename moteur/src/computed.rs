//! Les listes calculées (lot 2 du web ; l'écriture A, choisie par Yocthan le 2026-10-07) : une
//! liste qui se calcule d'une autre à chaque changement, pour chercher, filtrer, trier et couper.
//!
//! ```holo
//! Page(
//!   state: State(search: "", chosen: "", shown: 6, articles: [ Item(title: "Sunrise", kind: "huile", price: 120) ]),
//!   computed: [
//!     Filter(name: found, from: articles, contains: search, in: [title],
//!            field: kind, is: chosen, sortBy: price, reverse: true, limit: shown),
//!   ],
//!   children: [ Repeat(over: found, empty: "Aucun résultat", children: [ Text("{item.title}") ]) ],
//! )
//! ```
//!
//! La liste calculée n'est jamais écrite par une règle ni gardée : elle est refaite d'après sa
//! source et les valeurs qu'elle lit, chaque fois que l'état change. Elle se montre comme une
//! liste ordinaire (`Repeat(over: found)`, `{found}` pour son nombre d'éléments). Une valeur vide
//! ne filtre pas : un champ de recherche vide montre toute la liste.

use crate::holo::{Block, Error, Program, Value};
use crate::lists::{fields, text_of, Lists, ELEMENTS_MAX};
use crate::state::{State, Texts};

/// Les réglages d'un `Filter`.
pub const PARAMS: &[&str] = &["name", "from", "contains", "in", "field", "is", "sortBy", "reverse", "limit"];

/// Une liste calculée, lue dans le fichier.
#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub name: String,
    pub from: String,
    contains: Option<String>,
    within: Vec<String>,
    field: Option<String>,
    is: Option<Value>,
    sort_by: Option<String>,
    reverse: bool,
    limit: Option<Value>,
}

/// Les `Filter` de la page, dans l'ordre écrit (un filtre peut partir d'un filtre écrit avant lui).
fn blocks(program: &Program) -> Vec<&Block> {
    match program.root.argument("computed").map(|a| &a.value) {
        Some(Value::List(items)) => items.iter().filter_map(|v| if let Value::Block(b) = v { Some(b) } else { None }).collect(),
        _ => Vec::new(),
    }
}

fn read(block: &Block) -> Filter {
    let name_of = |param: &str| match block.argument(param).map(|a| &a.value) {
        Some(Value::Name(n)) => Some(n.clone()),
        _ => None,
    };
    let within = match block.argument("in").map(|a| &a.value) {
        Some(Value::List(names)) => names.iter().filter_map(|v| if let Value::Name(n) = v { Some(n.clone()) } else { None }).collect(),
        Some(Value::Name(n)) => vec![n.clone()],
        _ => Vec::new(),
    };
    Filter {
        name: name_of("name").unwrap_or_default(),
        from: name_of("from").unwrap_or_default(),
        contains: name_of("contains"),
        within,
        field: name_of("field"),
        is: block.argument("is").map(|a| a.value.clone()),
        sort_by: name_of("sortBy"),
        reverse: matches!(block.argument("reverse").map(|a| &a.value), Some(Value::Bool(true))),
        limit: block.argument("limit").map(|a| a.value.clone()),
    }
}

/// Les listes calculées de la page.
pub fn filters(program: &Program) -> Vec<Filter> {
    blocks(program).into_iter().map(read).collect()
}

/// Le nom des listes calculées.
pub fn names(program: &Program) -> Vec<String> {
    filters(program).into_iter().map(|f| f.name).collect()
}

pub fn is_computed(program: &Program, name: &str) -> bool {
    filters(program).iter().any(|f| f.name == name)
}

/// La liste déclarée dont part une liste calculée (en remontant les filtres posés l'un sur l'autre).
pub fn source_of(program: &Program, name: &str) -> Option<String> {
    let all = filters(program);
    let mut current = name.to_string();
    for _ in 0..=all.len() {
        match all.iter().find(|f| f.name == current) {
            Some(f) => current = f.from.clone(),
            None => return Some(current),
        }
    }
    None
}

/// Vérifie les listes calculées : chaque erreur dit ce qu'il faut écrire.
pub fn check(program: &Program) -> Result<(), Error> {
    let Some(argument) = program.root.argument("computed") else { return Ok(()) };
    let Value::List(items) = &argument.value else {
        return Err(Error { message: "« computed » attend une liste : computed: [ Filter(name: found, from: articles, contains: search) ]".into(), pos: argument.pos });
    };
    let texts = crate::state::initial_texts(program);
    let numbers = crate::state::initial(program)?;
    let mut known: Vec<String> = Vec::new();
    for item in items {
        let Value::Block(block) = item else {
            return Err(Error { message: "« computed » ne contient que des « Filter(…) »".into(), pos: argument.pos });
        };
        let error = |message: String| Err(Error { message, pos: block.pos });
        if block.name != "Filter" {
            return error(format!("« computed » ne contient que des « Filter(…) », pas « {} »", block.name));
        }
        for a in &block.arguments {
            match a.name.as_deref() {
                Some(n) if PARAMS.contains(&n) => {}
                Some(n) => return Err(Error { message: format!("« Filter » n'a pas de paramètre « {n} » ; paramètres possibles : {}", PARAMS.join(", ")), pos: a.pos }),
                None => return Err(Error { message: "chaque paramètre de « Filter » est nommé : Filter(name: found, from: articles)".into(), pos: a.pos }),
            }
        }
        let filter = read(block);
        if filter.name.is_empty() || !filter.name.starts_with(|c: char| c.is_ascii_lowercase()) {
            return error("« Filter » attend « name », le nom de la liste calculée, en minuscules : Filter(name: found, …)".into());
        }
        let taken = numbers.iter().any(|(n, _)| *n == filter.name) || texts.iter().any(|(n, _)| *n == filter.name) || crate::lists::initial(program).iter().any(|(n, _)| *n == filter.name) || known.contains(&filter.name);
        if taken {
            return error(format!("« {} » est déjà le nom d'une valeur : une liste calculée a son propre nom", filter.name));
        }
        if filter.from.is_empty() {
            return error(format!("« Filter(name: {}) » attend « from » : la liste qu'il filtre, from: articles", filter.name));
        }
        let from_declared = crate::lists::initial(program).iter().any(|(n, _)| *n == filter.from);
        if !from_declared && !known.contains(&filter.from) {
            return error(format!("« Filter(from: {}) » : aucune liste ne s'appelle « {} » ; déclare-la, state: State({}: [])", filter.from, filter.from, filter.from));
        }
        let record_fields = match crate::lists::kind(program, &filter.from) {
            Some(crate::lists::Kind::Records(f)) => Some(f),
            _ => None,
        };
        let check_field = |field: &str, word: &str| -> Result<(), Error> {
            match &record_fields {
                Some(f) if f.iter().any(|known| known == field) => Ok(()),
                Some(f) => Err(Error { message: format!("« Filter({word}: {field}) » : les éléments de « {} » n'ont pas de champ « {field} » ; champs : {}", filter.from, f.join(", ")), pos: block.pos }),
                None if field == "item" => Ok(()),
                None => Err(Error { message: format!("« Filter({word}: …) » : les éléments de « {} » sont des textes, sans champs ; écris « item »", filter.from), pos: block.pos }),
            }
        };
        if let Some(search) = &filter.contains {
            if !texts.iter().any(|(n, _)| n == search) {
                return error(format!("« Filter(contains: {search}) » attend une valeur de texte, comme le champ de recherche : state: State({search}: \"\")"));
            }
        } else if block.argument("contains").is_some() {
            return error("« Filter(contains: …) » attend le nom d'une valeur de texte : contains: search".into());
        }
        if block.argument("in").is_some() && filter.within.is_empty() {
            return error("« Filter(in: …) » attend les champs où chercher : in: [title, note]".into());
        }
        if !filter.within.is_empty() && filter.contains.is_none() {
            return error("« Filter(in: …) » dit où chercher : il va avec « contains »".into());
        }
        for field in &filter.within {
            check_field(field, "in")?;
        }
        match (&filter.field, &filter.is) {
            (Some(field), Some(value)) => {
                check_field(field, "field")?;
                match value {
                    Value::Text(_) | Value::Integer(_) => {}
                    Value::Name(n) if texts.iter().any(|(t, _)| t == n) || numbers.iter().any(|(m, _)| m == n) => {}
                    Value::Name(n) => return error(format!("« Filter(is: {n}) » : aucune valeur ne s'appelle « {n} »")),
                    _ => return error("« Filter(is: …) » attend un texte, un nombre entier ou le nom d'une valeur".into()),
                }
            }
            (Some(_), None) => return error("« Filter(field: …) » va avec « is » : field: kind, is: chosen".into()),
            (None, Some(_)) => return error("« Filter(is: …) » va avec « field » : field: kind, is: chosen".into()),
            (None, None) => {}
        }
        if let Some(field) = &filter.sort_by {
            check_field(field, "sortBy")?;
        } else if block.argument("sortBy").is_some() {
            return error("« Filter(sortBy: …) » attend le nom d'un champ : sortBy: price".into());
        }
        if let Some(a) = block.argument("reverse") {
            if !matches!(a.value, Value::Bool(_)) {
                return Err(Error { message: "« Filter(reverse: …) » attend true ou false".into(), pos: a.pos });
            }
            if filter.sort_by.is_none() {
                return Err(Error { message: "« Filter(reverse: …) » renverse un tri : il va avec « sortBy »".into(), pos: a.pos });
            }
        }
        match &filter.limit {
            None => {}
            Some(Value::Integer(n)) if (1..=ELEMENTS_MAX as u64).contains(n) => {}
            Some(Value::Name(n)) if numbers.iter().any(|(m, _)| m == n) => {}
            Some(Value::Name(n)) => return error(format!("« Filter(limit: {n}) » : aucun nombre ne s'appelle « {n} » ; déclare-le, state: State({n}: 12)")),
            Some(_) => return error(format!("« Filter(limit: …) » attend un nombre entier de 1 à {ELEMENTS_MAX}, ou le nom d'un nombre de la page")),
        }
        known.push(filter.name.clone());
    }
    Ok(())
}

/// Sans majuscules ni accents, pour chercher : « Élan » trouve « elan », et l'inverse.
pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ý' | 'ÿ' => 'y',
            'œ' => 'o',
            'æ' => 'a',
            other => other,
        })
        .collect()
}

/// Calcule les listes de la page, dans l'ordre écrit. Rien n'est lu que la page ne déclare.
pub fn apply(program: &Program, numbers: &State, texts: &Texts, lists: &Lists) -> Lists {
    let mut computed: Lists = Vec::new();
    for filter in filters(program) {
        let source = lists.iter().chain(computed.iter()).find(|(n, _)| *n == filter.from).map(|(_, e)| e.clone()).unwrap_or_default();
        let text_value = |name: &str| texts.iter().find(|(n, _)| n == name).map(|(_, v)| v.clone());
        let number_value = |name: &str| numbers.iter().find(|(n, _)| n == name).map(|(_, v)| *v);
        let value_of = |element: &str, field: &str| -> String {
            if field == "item" || !element.starts_with(crate::lists::RECORD) {
                text_of(element)
            } else {
                fields(element).into_iter().find(|(n, _)| n == field).map(|(_, v)| v).unwrap_or_default()
            }
        };
        let mut elements: Vec<String> = source;
        // Chercher : une valeur vide ne filtre pas.
        if let Some(search) = filter.contains.as_deref().and_then(text_value).map(|s| fold(s.trim())).filter(|s| !s.is_empty()) {
            elements.retain(|element| {
                let haystack: Vec<String> = if filter.within.is_empty() {
                    if element.starts_with(crate::lists::RECORD) { fields(element).into_iter().map(|(_, v)| v).collect() } else { vec![element.clone()] }
                } else {
                    filter.within.iter().map(|f| value_of(element, f)).collect()
                };
                haystack.iter().any(|h| fold(h).contains(&search))
            });
        }
        // Filtrer par un champ : là aussi, une valeur vide ne filtre pas.
        if let (Some(field), Some(is)) = (&filter.field, &filter.is) {
            let wanted = match is {
                Value::Text(t) => Some(t.clone()),
                Value::Integer(i) => Some(i.to_string()),
                Value::Name(n) => text_value(n).or_else(|| number_value(n).map(|v| v.to_string())),
                _ => None,
            };
            if let Some(wanted) = wanted.filter(|w| !w.is_empty()) {
                let wanted = fold(&wanted);
                elements.retain(|element| fold(&value_of(element, field)) == wanted);
            }
        }
        // Trier : des nombres comme des nombres, des textes sans majuscules ni accents ; à égalité,
        // l'ordre de départ est gardé.
        if let Some(field) = &filter.sort_by {
            elements.sort_by(|a, b| {
                let (x, y) = (value_of(a, field), value_of(b, field));
                match (x.parse::<i64>(), y.parse::<i64>()) {
                    (Ok(m), Ok(n)) => m.cmp(&n),
                    _ => fold(&x).cmp(&fold(&y)),
                }
            });
            if filter.reverse {
                elements.reverse();
            }
        }
        // Couper.
        let limit = match &filter.limit {
            Some(Value::Integer(n)) => Some(*n as usize),
            Some(Value::Name(n)) => number_value(n).map(|v| v as usize),
            _ => None,
        };
        if let Some(limit) = limit {
            elements.truncate(limit.min(ELEMENTS_MAX));
        }
        computed.push((filter.name, elements));
    }
    computed
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = r#"Page(
  title: "Catalogue",
  state: State(search: "", chosen: "", shown: 3, articles: [
    Item(title: "Élan du matin", kind: "huile", price: 120),
    Item(title: "La rivière", kind: "aquarelle", price: 90),
    Item(title: "Nuit", kind: "encre", price: 60),
    Item(title: "Le phare", kind: "huile", price: 200),
  ]),
  computed: [
    Filter(name: found, from: articles, contains: search, in: [title], field: kind, is: chosen, sortBy: price, reverse: true, limit: shown),
  ],
  children: [ Repeat(over: found, empty: "Aucun résultat", children: [ Text("{item.title}") ]) ],
)"#;

    fn titles(state: &str) -> Vec<String> {
        let program = crate::check_page(PAGE).unwrap();
        let numbers = crate::state::reread(&program, state);
        let texts = crate::state::reread_texts(&program, state);
        let lists = crate::lists::reread(&program, state);
        apply(&program, &numbers, &texts, &lists).remove(0).1.iter().map(|e| text_of(e)).collect()
    }

    #[test]
    fn a_computed_list_searches_filters_sorts_and_cuts() {
        // Rien d'écrit : toute la liste, triée par prix décroissant, coupée à 3.
        assert_eq!(titles(""), ["Le phare", "Élan du matin", "La rivière"]);
        // Chercher sans majuscules ni accents : « ELAN » trouve « Élan ».
        assert_eq!(titles("search='ELAN"), ["Élan du matin"]);
        // Filtrer par un champ, et montrer plus.
        assert_eq!(titles("chosen='huile;shown=10"), ["Le phare", "Élan du matin"]);
        assert_eq!(titles("search='zzz"), Vec::<String>::new());
    }

    #[test]
    fn a_computed_list_is_shown_and_never_written() {
        let state = crate::initial_state(PAGE);
        assert!(state.contains("found=["), "{state}");
        let html = crate::flat_view(PAGE, "").unwrap();
        assert!(html.contains("Le phare") && !html.contains("Nuit"), "{html}");
        // Après une recherche qui ne trouve rien : « Aucun résultat ».
        let html = crate::list_html(PAGE, "", "search='zzz", "found");
        assert!(html.contains("Aucun résultat"), "{html}");
        // Une liste calculée ne se change pas par une demande, et ne se garde pas.
        for (source, message) in [
            (PAGE.replace("children: [ Repeat", "rules: [ On(B.tap, effect: found.clear()) ], children: [ Button(name: B, text: \"x\"), Repeat"), "liste calculée"),
            (PAGE.replace("title: \"Catalogue\",", "title: \"Catalogue\", keep: [found],"), "liste calculée"),
            (PAGE.replace("from: articles", "from: nothing"), "aucune liste ne s'appelle « nothing »"),
            (PAGE.replace("in: [title]", "in: [colour]"), "n'ont pas de champ « colour »"),
            (PAGE.replace("contains: search", "contains: shown"), "attend une valeur de texte"),
            (PAGE.replace("limit: shown", "limit: 500"), "de 1 à 100"),
            (PAGE.replace("limit: shown),", "limit: shown), Filter(name: found, from: articles),"), "déjà le nom d'une valeur"),
            (PAGE.replace("limit: shown),", "limit: shown), Filter(name: search, from: articles),"), "déjà le nom d'une valeur"),
            (PAGE.replace("reverse: true", "reverse: yes"), "true ou false"),
            (PAGE.replace("sortBy: price,", "colour: red,"), "n'a pas de paramètre « colour »"),
        ] {
            let error = crate::check_page(&source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn the_count_of_a_computed_list_is_compared_after_a_change() {
        // L'état écrit contient la liste calculée, mais elle n'est pas relue : la réponse d'un
        // « If(found, …) » doit venir de la liste refaite (avant : toujours « 0 » après un changement).
        let source = PAGE.replace("children: [ Repeat", "children: [ Input(value: search, label: \"Chercher\"), If(found, is: 0, children: [ \"Rien\" ]), Repeat");
        let start = crate::initial_state(&source);
        assert_eq!(crate::conditions(&source, &start), "found|is=0:0");
        let typed = crate::input(&source, &start, "search", "ri");
        assert_eq!(crate::conditions(&source, &typed), "found|is=0:0");
        assert_eq!(crate::conditions(&source, &crate::input(&source, &typed, "search", "zzz")), "found|is=0:1");
    }

    #[test]
    fn fold_removes_case_and_accents() {
        assert_eq!(fold("Élan À LA Crème Brûlée"), "elan a la creme brulee");
    }
}
