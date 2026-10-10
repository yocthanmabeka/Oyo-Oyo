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
pub const PARAMS: &[&str] = &["name", "from", "contains", "in", "field", "is", "sortBy", "reverse", "offset", "limit", "total"];

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
    /// Combien d’éléments sauter après le tri ; la position est bornée à la longueur réelle.
    offset: Option<Value>,
    /// Le nombre d'éléments trouvés avant de couper, sous ce nom : « 12 sur 40 ».
    pub total: Option<String>,
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
        offset: block.argument("offset").map(|a| a.value.clone()),
        total: name_of("total"),
    }
}

/// Les listes calculées de la page.
pub fn filters(program: &Program) -> Vec<Filter> {
    blocks(program).into_iter().filter(|b| b.name == "Filter").map(read).collect()
}

/// Les réglages de `Days` (ADR-067).
pub const DAYS_PARAMS: &[&str] = &["name", "from", "to"];

/// Les `Days` de la page : (nom, date de départ, date d'arrivée).
fn days_blocks(program: &Program) -> Vec<(String, String, String)> {
    let name = |block: &Block, param: &str| match block.argument(param).map(|a| &a.value) {
        Some(Value::Name(n)) => n.clone(),
        _ => String::new(),
    };
    blocks(program).into_iter().filter(|b| b.name == "Days").map(|b| (name(b, "name"), name(b, "from"), name(b, "to"))).collect()
}

/// Le nom des nombres de jours, `Days(name: nights, …)` ; et celui des nombres de minutes,
/// `Minutes(name: left, …)` (ADR-109), des nombres que le moteur calcule de la même façon.
pub fn days_names(program: &Program) -> Vec<String> {
    days_blocks(program).into_iter().map(|(name, _, _)| name).chain(crate::hours::minutes_names(program)).collect()
}

/// Les nombres de jours, d'après les dates de la page : de `from` à `to`, 0 si l'une manque, ou si
/// `to` vient avant `from` (pas encore de nombre négatif).
pub fn days_values(program: &Program, texts: &Texts) -> State {
    let date = |name: &str| texts.iter().find(|(n, _)| n == name).and_then(|(_, t)| crate::dates::days(t));
    days_blocks(program)
        .into_iter()
        .map(|(name, from, to)| {
            let count = match (date(&from), date(&to)) {
                (Some(a), Some(b)) if b > a => (b - a) as u64,
                _ => 0,
            };
            (name, count)
        })
        // Les nombres de minutes (ADR-109), d'après les heures et les moments de la page.
        .chain(crate::hours::minutes_values(program, texts))
        .collect()
}

/// Le nom des listes calculées.
pub fn names(program: &Program) -> Vec<String> {
    filters(program).into_iter().map(|f| f.name).collect()
}

/// Le nom des totaux des listes calculées, `total: matching` : des nombres que la page montre.
pub fn total_names(program: &Program) -> Vec<String> {
    filters(program).into_iter().filter_map(|f| f.total).collect()
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
            return Err(Error { message: "« computed » ne contient que des « Filter(…) », des « Days(…) » et des « Minutes(…) »".into(), pos: argument.pos });
        };
        let error = |message: String| Err(Error { message, pos: block.pos });
        // Les jours entre deux dates (ADR-067) : Days(name: nights, from: arrival, to: departure).
        if block.name == "Days" {
            for a in &block.arguments {
                match a.name.as_deref() {
                    Some(n) if DAYS_PARAMS.contains(&n) => {}
                    Some(n) => return Err(Error { message: format!("« Days » n'a pas de paramètre « {n} » ; paramètres possibles : {}", DAYS_PARAMS.join(", ")), pos: a.pos }),
                    None => return Err(Error { message: "chaque paramètre de « Days » est nommé : Days(name: nights, from: arrival, to: departure)".into(), pos: a.pos }),
                }
            }
            let name = match block.argument("name").map(|a| &a.value) {
                Some(Value::Name(n)) if n.starts_with(|c: char| c.is_ascii_lowercase()) => n.clone(),
                _ => return error("« Days » attend « name », le nom du nombre de jours, en minuscules : Days(name: nights, from: arrival, to: departure)".into()),
            };
            let taken = numbers.iter().any(|(n, _)| *n == name) || texts.iter().any(|(n, _)| *n == name) || crate::lists::initial(program).iter().any(|(n, _)| *n == name) || known.contains(&name);
            if taken {
                return error(format!("« {name} » est déjà le nom d'une valeur : un nombre de jours a son propre nom"));
            }
            for param in ["from", "to"] {
                match block.argument(param).map(|a| &a.value) {
                    Some(Value::Name(date)) if crate::dates::is_date(program, date) => {}
                    Some(Value::Name(other)) => return error(format!("« Days({param}: {other}) » : « {other} » n'est pas une date ; une date est today, un champ Input(type: date), ou un texte déclaré « AAAA-MM-JJ »")),
                    _ => return error(format!("« Days » attend « {param} », une date : Days(name: {name}, from: arrival, to: departure)")),
                }
            }
            known.push(name);
            continue;
        }
        // Les minutes entre deux heures ou deux moments (ADR-109) : Minutes(name: left, from: now, to: train).
        if block.name == "Minutes" {
            let name = crate::hours::check_minutes(program, block)?;
            let taken = numbers.iter().any(|(n, _)| *n == name) || texts.iter().any(|(n, _)| *n == name) || crate::lists::initial(program).iter().any(|(n, _)| *n == name) || known.contains(&name);
            if taken {
                return error(format!("« {name} » est déjà le nom d'une valeur : un nombre de minutes a son propre nom"));
            }
            known.push(name);
            continue;
        }
        if block.name != "Filter" {
            return error(format!("« computed » ne contient que des « Filter(…) », des « Days(…) » et des « Minutes(…) », pas « {} »", block.name));
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
        match &filter.offset {
            None | Some(Value::Integer(_)) => {}
            Some(Value::Name(n)) if crate::state::places(program, n) > 0 => return error(format!("« Filter(offset: {n}) » attend une position entière, sans chiffres après la virgule")),
            Some(Value::Name(n)) if numbers.iter().any(|(m, _)| m == n) => {}
            Some(Value::Name(n)) => return error(format!("« Filter(offset: {n}) » : aucun nombre ne s'appelle « {n} »")),
            Some(_) => return error("« Filter(offset: …) » attend un entier positif ou nul, ou le nom d'un nombre entier de la page".into()),
        }
        match &filter.limit {
            None => {}
            Some(Value::Integer(n)) if (1..=ELEMENTS_MAX as u64).contains(n) => {}
            Some(Value::Name(n)) if crate::state::places(program, n) > 0 => return error(format!("« Filter(limit: {n}) » : « {n} » a des chiffres après la virgule ; on montre un nombre entier d'éléments")),
            Some(Value::Name(n)) if numbers.iter().any(|(m, _)| m == n) => {}
            Some(Value::Name(n)) => return error(format!("« Filter(limit: {n}) » : aucun nombre ne s'appelle « {n} » ; déclare-le, state: State({n}: 12)")),
            Some(_) => return error(format!("« Filter(limit: …) » attend un nombre entier de 1 à {ELEMENTS_MAX}, ou le nom d'un nombre de la page")),
        }
        if let Some(a) = block.argument("total") {
            let taken = |n: &str| numbers.iter().any(|(m, _)| m == n) || texts.iter().any(|(t, _)| t == n) || crate::lists::initial(program).iter().any(|(l, _)| l == n) || known.iter().any(|k| k == n) || n == filter.name;
            match &a.value {
                Value::Name(n) if !n.starts_with(|c: char| c.is_ascii_lowercase()) => return Err(Error { message: format!("« Filter(total: {n}) » : le nom d'un nombre s'écrit en minuscules au début, comme total: matching"), pos: a.pos }),
                Value::Name(n) if taken(n) => return Err(Error { message: format!("« {n} » est déjà le nom d'une valeur : le total d'une liste calculée a son propre nom"), pos: a.pos }),
                Value::Name(n) => known.push(n.clone()),
                _ => return Err(Error { message: "« Filter(total: …) » attend un nom, celui du nombre d'éléments trouvés avant de couper : total: matching".into(), pos: a.pos }),
            }
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
    apply_with_totals(program, numbers, texts, lists).0
}

/// Le total de chaque liste calculée qui le demande (`total: matching`) : le nombre d'éléments
/// trouvés avant de couper.
pub fn totals(program: &Program, numbers: &State, texts: &Texts, lists: &Lists) -> State {
    apply_with_totals(program, numbers, texts, lists).1
}

/// Les listes calculées, et leurs totaux.
pub fn apply_with_totals(program: &Program, numbers: &State, texts: &Texts, lists: &Lists) -> (Lists, State) {
    let mut computed: Lists = Vec::new();
    let mut totals: State = Vec::new();
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
        if let Some(total) = filter.total {
            totals.push((total, elements.len() as u64));
        }
        // Le total précède les deux coupes. Un grand entier reste grand sur wasm32 :
        // on le borne avant de le convertir en usize, sans allocation supplémentaire.
        let offset = match &filter.offset {
            Some(Value::Integer(n)) => *n,
            Some(Value::Name(n)) => number_value(n).unwrap_or(0),
            _ => 0,
        };
        let skipped = offset.min(elements.len() as u64) as usize;
        elements.drain(..skipped);
        if let Some(limit) = limit {
            elements.truncate(limit.min(ELEMENTS_MAX));
        }
        computed.push((filter.name, elements));
    }
    (computed, totals)
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
            (PAGE.replace("limit: shown", "limit: 500"), "de 1 à 200"),
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
    fn a_line_of_a_computed_list_changes_its_source() {
        // Une liste triée par « done » : toucher « Fait » sur une ligne change la tâche d'origine,
        // qui descend ; « x » la retire de « tasks » (avant : le geste ne changeait rien).
        let source = r#"Page(
  state: State(tasks: [ Item(id: "t1", title: "Pain", done: 0), Item(id: "t2", title: "Lait", done: 0), Item(id: "t3", title: "Thé", done: 0) ]),
  computed: [ Filter(name: ordered, from: tasks, sortBy: done) ],
  children: [
    Repeat(over: ordered, key: id, children: [ Text("{item.title}"), Button(name: Done, text: "Fait"), Button(name: Drop, text: "x") ],
           rules: [ On(Done.tap, effect: item.done.set(1)), On(Drop.tap, effect: tasks.remove(item)) ]),
  ],
)"#;
        let program = crate::check_page(source).unwrap();
        let titles = |state: &str, list: &str| -> Vec<String> {
            let numbers = crate::state::reread(&program, state);
            let texts = crate::state::reread_texts(&program, state);
            let lists = crate::lists::reread(&program, state);
            let all: Vec<(String, Vec<String>)> = lists.iter().cloned().chain(apply(&program, &numbers, &texts, &lists)).collect();
            all.into_iter().find(|(n, _)| n == list).unwrap().1.iter().map(|e| {
                let f = fields(e);
                let get = |k: &str| f.iter().find(|(c, _)| c == k).map(|(_, v)| v.clone()).unwrap_or_default();
                format!("{}{}", get("title"), if get("done") == "1" { "✓" } else { "" })
            }).collect()
        };
        let start = crate::initial_state(source);
        // « Fait » sur la deuxième ligne de la liste triée : « Lait » est fait, et passe en bas.
        let done = crate::arbitrate(source, &start, "Done.tap@1");
        assert_eq!(titles(&done, "tasks"), ["Pain", "Lait✓", "Thé"]);
        assert_eq!(titles(&done, "ordered"), ["Pain", "Thé", "Lait✓"]);
        // La ligne du bas de la liste triée est « Lait » : « x » le retire de « tasks ».
        let dropped = crate::arbitrate(source, &done, "Drop.tap@2");
        assert_eq!(titles(&dropped, "tasks"), ["Pain", "Thé"]);
        // Sa clé est restée « t2 » quand il a changé : la page a gardé la ligne.
        assert!(crate::list_html(source, "", &done, "ordered").contains(r#"data-rank="2" data-key="k:t2-0""#));
    }

    #[test]
    fn the_total_before_the_limit_is_shown_and_compared() {
        // `total: matching` : combien d'éléments sont trouvés avant de couper (« 3 sur 4 »).
        let source = PAGE.replace("limit: shown),", "limit: shown, total: matching),").replace(
            "children: [ Repeat",
            "children: [ Input(value: search, label: \"Chercher\"), P(\"{found} sur {matching}\"), If(shown, under: matching, children: [ Button(name: More, text: \"Plus\") ]), Repeat",
        );
        let start = crate::initial_state(&source);
        assert!(start.contains("found=[") && start.contains("matching=4"), "{start}");
        let html = crate::flat_view(&source, "").unwrap();
        assert!(html.contains(r#"<span data-state="found">3</span> sur <span data-state="matching">4</span>"#), "{html}");
        assert!(html.contains(r#"data-if="shown|under=matching"><button"#), "{html}");
        // Chercher « ri » : une seule trouvée (« La rivière »), moins que les trois montrées ; le bouton se cache.
        let typed = crate::input(&source, &start, "search", "ri");
        assert!(typed.contains("matching=1"), "{typed}");
        assert_eq!(crate::conditions(&source, &typed), "shown|under=matching:0");
        assert_eq!(crate::conditions(&source, &start), "shown|under=matching:1");
        // Les refus : un nom déjà pris, une majuscule, autre chose qu'un nom.
        for (wrong, message) in [
            ("total: search", "déjà le nom d'une valeur"),
            ("total: found", "déjà le nom d'une valeur"),
            ("total: Matching", "en minuscules"),
            ("total: 3", "attend un nom"),
        ] {
            let error = crate::check_page(&source.replace("total: matching", wrong)).unwrap_err();
            assert!(error.message.contains(message), "{wrong}\n→ {error}");
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

    #[test]
    fn two_hundred_products_are_searched_sorted_and_paged() {
        let items = (1..=200).map(|n| format!("Item(id: \"p{n:03}\", title: \"Produit {n:03}\", price: {})", 201 - n)).collect::<Vec<_>>().join(",");
        let source = format!(r#"Page(state: State(search: "", offset: 0, products: [{items}]),
            computed: [Filter(name: found, from: products, contains: search, in: [title], sortBy: price, offset: offset, limit: 20, total: matching)],
            children: [H1("Catalogue"), Repeat(over: found, key: id, children: [P("{{item.title}}")])])"#);
        let program = crate::check_page(&source).unwrap();
        let list = |saved: &str| {
            let nums = crate::state::reread(&program, saved);
            let texts = crate::state::reread_texts(&program, saved);
            let lists = crate::lists::reread(&program, saved);
            apply_with_totals(&program, &nums, &texts, &lists)
        };
        let mut seen = Vec::new();
        for page in 0..10 {
            let (found, totals) = list(&format!("offset={}", page * 20));
            assert_eq!(totals, [("matching".into(), 200)]);
            assert_eq!(found[0].1.len(), 20);
            seen.extend(found[0].1.iter().map(|e| text_of(e)));
        }
        let expected = (1..=200).rev().map(|n| format!("p{n:03}")).collect::<Vec<_>>();
        assert_eq!(seen, expected, "aucun doublon, aucun article perdu après le tri");
        let (found, total) = list("search='Produit%20001;offset=0");
        assert_eq!(found[0].1.len(), 1);
        assert_eq!(text_of(&found[0].1[0]), "p001");
        assert_eq!(total, [("matching".into(), 1)]);
        for offset in [200, u64::MAX] {
            let (found, totals) = list(&format!("offset={offset}"));
            assert!(found[0].1.is_empty());
            assert_eq!(totals, [("matching".into(), 200)]);
        }
        let too_many = source.replace("products: [", "products: [Item(id: \"extra\", title: \"Extra\", price: 0),");
        assert!(crate::check_page(&too_many).unwrap_err().message.contains("200 éléments"));
        let literal = source.replace("offset: offset", "offset: 18446744073709551615");
        assert!(crate::list_html(&literal, "", "", "found").contains("holo-empty") || {
            let p = crate::check_page(&literal).unwrap();
            apply(&p, &crate::state::initial(&p).unwrap(), &crate::state::initial_texts(&p), &crate::lists::initial(&p))[0].1.is_empty()
        });
    }

    #[test]
    fn a_page_offset_refuses_text_decimals_units_and_unknown_names() {
        let source = r#"Page(state: State(position: 1.5, products: ["A", "B"]), computed: [Filter(name: found, from: products, offset: 0, limit: 1)], children: [P("Liste")])"#;
        crate::check_page(source).unwrap();
        for wrong in ["\"1\"", "1.5", "1px", "true", "missing", "position"] {
            let bad = source.replace("offset: 0", &format!("offset: {wrong}"));
            assert!(crate::check_page(&bad).unwrap_err().message.contains("offset"), "{wrong}");
        }
        let source = source.replace("offset: 0", "offset: 1");
        let program = crate::check_page(&source).unwrap();
        let found = apply(&program, &crate::state::initial(&program).unwrap(), &crate::state::initial_texts(&program), &crate::lists::initial(&program));
        assert_eq!(found[0].1, ["B"]);
    }

}
