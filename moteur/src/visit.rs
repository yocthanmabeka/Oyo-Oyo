//! La mémoire de visite (ADR-113) : des valeurs que la page retient le temps d'une visite, d'une
//! page à l'autre du même site, dans le même onglet, et que le navigateur efface quand l'onglet
//! se ferme (`sessionStorage`).
//!
//! ```holo
//! Page(
//!   state: State(firstName: "", people: 1),
//!   visit: [firstName, people],
//!   children: [
//!     Input(value: firstName, label: "Your first name"),
//!     A("Step 2", to: "step-2.holo"),
//!   ],
//! )
//! ```
//!
//! `keep` garde des valeurs sous l'adresse de la page, d'une visite à l'autre (ADR-027, ADR-090).
//! `visit` les range sous leur nom (`holo-visit:firstName`) : deux pages du site qui retiennent le
//! même nom parlent de la même valeur, et elle suit le visiteur de l'une à l'autre.
//!
//! Ce qui revient vient d'une autre page, peut-être écrite autrement : un texte ici, un nombre là,
//! d'autres bornes. Chaque valeur est donc relue seule, comme un import (ADR-093) : de sa sorte,
//! et dans les bornes de cette page-ci. Sinon elle est ignorée, sans erreur, et la page garde la
//! sienne : jamais une valeur fausse (un texte coupé, un nombre arrondi ou ramené à sa borne).
//! Rien ne part au serveur ; aucun cookie.

use crate::holo::{Error, Program, Value};
use crate::lists::{Json, Kind, Lists};
use crate::state::{State, Texts};

/// Une valeur retenue, au plus : 64 Ko de JSON, comme un fichier importé (ADR-093).
pub const BYTES_MAX: usize = crate::capabilities::BYTES_MAX;

/// Les valeurs que la page retient le temps de la visite (`visit: [firstName, people]`),
/// vérifiées : des nombres, des textes ou des listes déclarés dans `State` ; ni une valeur gardée
/// (`keep`), ni une valeur partagée (`shared`), ni une valeur de l'adresse (`address:`, ou le nom
/// du fichier), ni l'heure, ni une liste calculée.
pub fn names(program: &Program) -> Result<Vec<String>, Error> {
    let Some(argument) = program.root.argument("visit") else { return Ok(Vec::new()) };
    let error = |message: String| Error { message, pos: argument.pos };
    let Value::List(written) = &argument.value else {
        return Err(error("« visit » attend la liste des valeurs à retenir le temps de la visite : visit: [firstName]".into()));
    };
    let numbers = crate::state::initial(program)?;
    let texts = crate::state::initial_texts(program);
    let kept = crate::state::kept_values(program)?;
    let address = crate::history::names(program)?;
    let mut names = Vec::new();
    for name in written {
        let Value::Name(name) = name else {
            return Err(error("« visit » attend des noms de valeurs : visit: [firstName]".into()));
        };
        // Le moment présent (ADR-109) : le moteur le redonne à chaque minute.
        if name == crate::hours::NOW {
            return Err(error("« visit » : « now » est le moment présent ; il ne se retient pas, le moteur le redonne".into()));
        }
        let message = if crate::state::CLOCK.contains(&name.as_str()) || name == crate::dates::TODAY {
            format!("« visit » : « {name} » est l'heure du visiteur ; elle ne se retient pas, le moteur la redonne")
        } else if crate::account::GIVEN.contains(&name.as_str()) {
            format!("« visit » : « {name} » est donné par le serveur à chaque visite ; il ne se retient pas")
        } else if program.shared.contains(name) {
            format!("« visit » : « {name} » est partagée, le serveur la garde pour tous (ADR-079) ; visit retient ce qui est à un seul visiteur, dans son onglet")
        } else if program.address.contains(name) {
            format!("« visit » : « {name} » vient de l'adresse, par le nom du fichier (ADR-078) ; elle ne se retient pas")
        } else if kept.contains(name) {
            format!("« visit » : « {name} » est déjà gardée (keep), d'une visite à l'autre ; une valeur est gardée, ou retenue le temps de la visite, pas les deux")
        } else if address.contains(name) {
            format!("« visit » : « {name} » est dans l'adresse (address) ; une valeur vient de l'adresse ou de la visite, pas des deux")
        } else if crate::computed::is_computed(program, name) {
            format!("« visit » : « {name} » est une liste calculée ; elle se refait d'après sa source, retiens plutôt la source")
        } else if names.contains(name) {
            format!("« visit » : « {name} » est écrit deux fois")
        } else if numbers.iter().any(|(known, _)| known == name) || texts.iter().any(|(known, _)| known == name) || crate::lists::is_list(program, name) {
            names.push(name.clone());
            continue;
        } else {
            format!("« visit » : aucune valeur ne s'appelle « {name} » ; la visite ne retient que des valeurs déclarées dans « State »")
        };
        return Err(error(message));
    }
    Ok(names)
}

/// Ce que la page écrit dans sa mémoire de visite, pour ces valeurs : une ligne par valeur
/// retenue, son nom, une tabulation, sa valeur en JSON (`firstName\t"Ada"`, `people\t2`,
/// `price\t12.50`, `tasks\t["Pain"]`). Le JSON n'a ni tabulation ni retour à la ligne : ils sont
/// écrits `\u0009`, `\u000a`. Une valeur de plus de 64 Ko est écrite vide : la page l'oublie
/// plutôt que de laisser une valeur d'avant.
pub fn to_store(program: &Program, numbers: &State, texts: &Texts, lists: &Lists) -> String {
    let mut lines = Vec::new();
    for name in names(program).unwrap_or_default() {
        let json = if let Some((_, value)) = numbers.iter().find(|(known, _)| *known == name) {
            crate::state::format_decimal(*value, crate::state::places(program, &name))
        } else if let Some((_, text)) = texts.iter().find(|(known, _)| *known == name) {
            crate::json_text(text)
        } else if let Some((_, elements)) = lists.iter().find(|(known, _)| *known == name) {
            let elements: Vec<String> = elements
                .iter()
                .map(|element| {
                    if element.starts_with(crate::lists::RECORD) {
                        let fields: Vec<String> = crate::lists::fields(element).iter().map(|(field, value)| format!("{}:{}", crate::json_text(field), crate::json_text(value))).collect();
                        format!("{{{}}}", fields.join(","))
                    } else {
                        crate::json_text(element)
                    }
                })
                .collect();
            format!("[{}]", elements.join(","))
        } else {
            continue;
        };
        lines.push(format!("{name}\t{}", if json.len() > BYTES_MAX { "" } else { json.as_str() }));
    }
    lines.join("\n")
}

/// Pose sur ces valeurs ce que la mémoire de visite rend, une ligne par valeur, comme l'écrit
/// `to_store`. Chaque valeur est relue seule, comme un import (ADR-093) : une valeur que cette
/// page retient, de sa sorte, dans ses bornes. Sinon elle est ignorée, sans erreur, et la valeur
/// de la page reste.
pub fn recall(program: &Program, numbers: &State, texts: &Texts, lists: &Lists, stored: &str) -> (State, Texts, Lists) {
    let (mut numbers, mut texts, mut lists) = (numbers.clone(), texts.clone(), lists.clone());
    let names = names(program).unwrap_or_default();
    for line in stored.lines() {
        let Some((name, json)) = line.split_once('\t') else { continue };
        if !names.iter().any(|known| known == name) || json.len() > BYTES_MAX {
            continue;
        }
        let Some(value) = Json::read(json) else { continue };
        if let Some((_, place)) = numbers.iter_mut().find(|(known, _)| known == name) {
            if let Some(units) = number(program, name, &value) {
                *place = units;
            }
        } else if let Some((_, place)) = texts.iter_mut().find(|(known, _)| known == name) {
            if let Json::Text(text) = value {
                if text_fits(program, name, &text) {
                    *place = text;
                }
            }
        } else if let Some((_, place)) = lists.iter_mut().find(|(known, _)| known == name) {
            if let Some(elements) = list(program, name, &value) {
                *place = elements;
            }
        }
    }
    (numbers, texts, lists)
}

/// Un nombre retenu, à l'échelle de cette page, sans rien arrondir : `12.50` et `12.5` valent
/// 1250 pour deux chiffres après la virgule ; `12.55` ne va pas dans une valeur à un chiffre, ni
/// `3.5` dans un nombre entier. Puis dans les bornes de la page (le `min` et le `max` de ses
/// champs, 0 ou 1 pour une case) : hors d'elles, rien, un nombre n'est jamais ramené à sa borne.
fn number(program: &Program, name: &str, value: &Json) -> Option<u64> {
    let places = crate::state::places(program, name);
    let units = match value {
        Json::Number(whole) => whole.checked_mul(crate::state::scale(places))?,
        Json::Decimal(written) => {
            let (_, fraction) = written.split_once('.')?;
            if fraction.bytes().skip(places as usize).any(|digit| digit != b'0') {
                return None;
            }
            crate::state::parse_decimal(written, places)?
        }
        _ => return None,
    };
    (crate::state::floor(program, name)..=crate::state::ceiling(program, name)).contains(&units).then_some(units)
}

/// Un texte retenu, tel que cette page l'accepte : 2 000 caractères au plus, sans caractère
/// invisible (les retours à la ligne d'un texte long gardés). Une page qui le présente dans un
/// champ le fait passer par son arbitre, comme une saisie (sa longueur, les options d'un choix,
/// une vraie date) : s'il en ressort changé, il n'est pas pris. Une page qui ne fait que le
/// montrer (un récapitulatif) le prend tel quel.
fn text_fits(program: &Program, name: &str, text: &str) -> bool {
    // Un dessin (ADR-115) : seulement un dessin juste pour sa feuille, relu strictement.
    if let Some(sheet) = crate::sketch::sheet_of(program, name) {
        return crate::sketch::clean(&sheet, text).is_some();
    }
    if text.chars().count() > crate::state::TEXT_MAX || text.chars().any(|c| c.is_control() && c != '\n') {
        return false;
    }
    let mut presented = false;
    let _ = crate::rules::for_each_block(&program.root, &mut |block| {
        presented |= matches!(block.name.as_str(), "Input" | "Choice") && matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(value)) if value == name);
        Ok(())
    });
    if !presented {
        return true;
    }
    crate::state::input_text(program, &crate::state::initial_texts(program), name, text).iter().any(|(known, after)| known == name && after == text)
}

/// Une liste retenue, comme un import (ADR-093) : 200 éléments au plus ; des textes pour une liste
/// de textes, des éléments à champs avec exactement leurs champs pour une liste à champs ; chaque
/// texte de 200 caractères au plus, sans caractère invisible. Un seul élément faux : rien.
fn list(program: &Program, name: &str, value: &Json) -> Option<Vec<String>> {
    let Json::Table(elements) = value else { return None };
    let kind = crate::lists::kind(program, name)?;
    if elements.len() > crate::lists::ELEMENTS_MAX {
        return None;
    }
    let clean = |text: &str| text.chars().count() <= crate::lists::ELEMENT_MAX && !text.chars().any(char::is_control);
    let field_name = |field: &str| field.len() <= 40 && field != "key" && field.starts_with(|c: char| c.is_ascii_lowercase()) && field.chars().all(|c| c.is_ascii_alphanumeric());
    let mut taken = Vec::new();
    for element in elements {
        match (element, &kind) {
            (Json::Text(text), Kind::Texts | Kind::Free) if !text.is_empty() && clean(text) => taken.push(text.clone()),
            (Json::Object(fields), Kind::Records(_) | Kind::Free) => {
                let mut read: Vec<(String, String)> = Vec::new();
                for (field, value) in fields {
                    let Json::Text(text) = value else { return None };
                    if !field_name(field) || !clean(text) || read.iter().any(|(known, _)| known == field) {
                        return None;
                    }
                    read.push((field.clone(), text.clone()));
                }
                if read.is_empty() || read.len() > crate::lists::FIELDS_MAX {
                    return None;
                }
                // Une liste à champs : exactement les siens, rangés dans l'ordre de sa déclaration.
                if let Kind::Records(expected) = &kind {
                    if read.len() != expected.len() || !expected.iter().all(|field| read.iter().any(|(known, _)| known == field)) {
                        return None;
                    }
                    read = expected.iter().filter_map(|field| read.iter().find(|(known, _)| known == field).cloned()).collect();
                }
                taken.push(crate::lists::record(&read));
            }
            _ => return None,
        }
    }
    Some(taken)
}

#[cfg(test)]
mod tests {
    /// Une inscription en deux pages : la première demande le prénom et le nombre de personnes.
    const FIRST: &str = "Page(
  title: \"Étape 1\",
  state: State(firstName: \"\", people: 1, price: 12.50, note: \"\", workshop: \"\", tasks: [], seen: 0),
  visit: [firstName, people, price, note, workshop, tasks],
  children: [
    H1(\"Étape 1\"),
    Input(value: firstName, label: \"Prénom\", max: 40),
    Input(value: people, label: \"Personnes\", min: 1, max: 6),
    Input(value: price, label: \"Prix\"),
    Input(value: note, label: \"Un mot\", lines: 3),
    Choice(value: workshop, label: \"Atelier\", options: [\"Aquarelle\", \"Poterie\"]),
    P(\"{seen}\"),
  ],
)";

    /// La seconde ne fait que les montrer, et les retient aussi.
    const SECOND: &str = "Page(
  title: \"Étape 2\",
  state: State(firstName: \"Inconnu\", people: 0, price: 0.0, note: \"\", tasks: []),
  visit: [firstName, people, price, note, tasks],
  children: [ H1(\"Étape 2\"), P(\"{firstName} {people} {price} {note} {tasks}\") ],
)";

    fn values(source: &str, state: &str) -> String {
        crate::to_visit(source, state)
    }

    #[test]
    fn a_value_follows_the_visitor_from_one_page_to_the_other() {
        // La page 1 : le visiteur écrit ; la page écrit chaque valeur sous son nom, en JSON.
        let mut state = crate::initial_state(FIRST);
        for (name, written) in [("firstName", "Zoé \"Ada\"\\"), ("people", "3"), ("price", "9,5"), ("note", "deux\nlignes"), ("workshop", "Poterie")] {
            state = crate::input(FIRST, &state, name, written);
        }
        let stored = values(FIRST, &state);
        assert_eq!(stored, "firstName\t\"Zoé \\\"Ada\\\"\\\\\"\npeople\t3\nprice\t9.50\nnote\t\"deux\\u000alignes\"\nworkshop\t\"Poterie\"\ntasks\t[]");
        // La valeur que la page ne retient pas (seen) n'y est pas.
        assert!(!stored.contains("seen"));
        // La page 2 les reprend, à son échelle : 9.50 dans une valeur à un chiffre, 9.5.
        let second = crate::from_visit(SECOND, &crate::initial_state(SECOND), &stored);
        let program = crate::check_page(SECOND).unwrap();
        assert_eq!(crate::state::reread_texts(&program, &second), [("firstName".to_string(), "Zoé \"Ada\"\\".to_string()), ("note".to_string(), "deux\nlignes".to_string())]);
        assert_eq!(crate::state::reread(&program, &second), [("people".to_string(), 3), ("price".to_string(), 95)]);
        // « workshop », que la page 2 ne déclare pas, est laissé de côté sans erreur.
        assert_eq!(values(SECOND, &second), "firstName\t\"Zoé \\\"Ada\\\"\\\\\"\npeople\t3\nprice\t9.5\nnote\t\"deux\\u000alignes\"\ntasks\t[]");
        // Et revient à la page 1 : la même valeur, à l'échelle de la page 1.
        let back = crate::from_visit(FIRST, &crate::initial_state(FIRST), &values(SECOND, &second));
        assert_eq!(crate::state::reread(&crate::check_page(FIRST).unwrap(), &back).iter().find(|(name, _)| name == "price").unwrap().1, 950);
        // Une page sans visit n'écrit rien et ne reprend rien.
        let plain = "Page(state: State(firstName: \"\"), children: [ P(\"{firstName}\") ])";
        assert_eq!(values(plain, &crate::initial_state(plain)), "");
        assert_eq!(crate::from_visit(plain, &crate::initial_state(plain), &stored), crate::initial_state(plain));
        assert_eq!(crate::visit_names(FIRST), "firstName,people,price,note,workshop,tasks");
        assert_eq!(crate::visit_names(plain), "");
    }

    #[test]
    fn a_value_of_another_sort_or_out_of_bounds_is_ignored() {
        let start = crate::initial_state(FIRST);
        let program = crate::check_page(FIRST).unwrap();
        let taken = |stored: &str| crate::from_visit(FIRST, &start, stored);
        // Jamais une valeur fausse, jamais une erreur : chaque valeur relue est ignorée seule.
        for stored in [
            "people\t\"3\"",          // un texte pour un nombre
            "firstName\t3",           // un nombre pour un texte
            "people\t9",              // au-delà du max de son champ (6), jamais ramené à 6
            "people\t0",              // en deçà de son min (1)
            "people\t-2",             // un nombre négatif
            "people\t2.5",            // un nombre à virgule dans un nombre entier
            "price\t9.555",           // trois chiffres dans une valeur à deux : jamais arrondi
            "price\t[1]",             // une liste pour un nombre
            "people\t3e2",            // une écriture que le moteur ne relit pas
            "firstName\t\"Ada",       // un JSON abîmé
            "firstName\tAda",         // pas du JSON
            "firstName\t",            // vide
            "workshop\t\"Gravure\"",  // pas une option du choix
            "tasks\t[\"\"]",          // un élément vide
            "tasks\t[\"a\", 1]",      // un élément d'une autre sorte
            "admin\t1",               // un nom que la page ne retient pas
            "seen\t5",                // une valeur de la page qu'elle ne retient pas
        ] {
            assert_eq!(taken(stored), start, "{stored}");
        }
        // Un texte trop long pour son champ (40) : ignoré, jamais coupé.
        assert_eq!(taken(&format!("firstName\t\"{}\"", "a".repeat(41))), start);
        assert_ne!(taken(&format!("firstName\t\"{}\"", "a".repeat(40))), start);
        // Un retour à la ligne dans un champ d'une ligne : ignoré ; dans un texte long, gardé.
        assert_eq!(taken("firstName\t\"a\\nb\""), start);
        assert!(crate::state::reread_texts(&program, &taken("note\t\"a\\nb\"")).contains(&("note".to_string(), "a\nb".to_string())));
        // Un caractère invisible : ignoré.
        assert_eq!(taken("note\t\"a\\u0007b\""), start);
        // Plus de 64 Ko : ignoré, avant même d'être lu.
        assert_eq!(taken(&format!("note\t\"{}\"", "a".repeat(70_000))), start);
        // Les autres valeurs, justes, sont prises : une ligne fausse n'empêche pas les autres.
        let mixed = taken("people\t\"3\"\nfirstName\t\"Ada\"\nprice\t7\nworkshop\t\"Aquarelle\"");
        assert_eq!(values(FIRST, &mixed), "firstName\t\"Ada\"\npeople\t1\nprice\t7.00\nnote\t\"\"\nworkshop\t\"Aquarelle\"\ntasks\t[]");
        // Une case vaut 0 ou 1 ; une date, un vrai jour du calendrier.
        let form = "Page(state: State(gift: 0, arrival: \"\"), visit: [gift, arrival], children: [ Checkbox(value: gift, label: \"Cadeau\"), Input(value: arrival, label: \"Arrivée\", type: date) ])";
        let form_start = crate::initial_state(form);
        assert_eq!(crate::from_visit(form, &form_start, "gift\t2\narrival\t\"2026-02-30\""), form_start);
        assert_eq!(values(form, &crate::from_visit(form, &form_start, "gift\t1\narrival\t\"2026-10-09\"")), "gift\t1\narrival\t\"2026-10-09\"");
    }

    #[test]
    fn a_list_is_read_back_like_an_import() {
        let page = "Page(state: State(tasks: [\"Pain\"], cart: [ Item(title: \"Thé\", price: 3) ], free: []), visit: [tasks, cart, free], children: [ P(\"{tasks} {cart} {free}\") ])";
        let program = crate::check_page(page).unwrap();
        let start = crate::initial_state(page);
        assert_eq!(values(page, &start), "tasks\t[\"Pain\"]\ncart\t[{\"title\":\"Thé\",\"price\":\"3\"}]\nfree\t[]");
        // Des fiches avec exactement leurs champs, dans n'importe quel ordre ; une liste libre prend
        // des textes ou des fiches.
        let after = crate::from_visit(page, &start, "tasks\t[\"Lait\",\"Œufs\"]\ncart\t[{\"price\":\"5\",\"title\":\"Café\"}]\nfree\t[{\"a\":\"1\"}]");
        assert_eq!(values(page, &after), "tasks\t[\"Lait\",\"Œufs\"]\ncart\t[{\"title\":\"Café\",\"price\":\"5\"}]\nfree\t[{\"a\":\"1\"}]");
        assert_eq!(crate::lists::reread(&program, &after)[0].1, ["Lait", "Œufs"]);
        // Une fiche à qui manque un champ, qui en a un de trop, ou un nombre au lieu d'un texte ;
        // un texte dans une liste de fiches ; 201 éléments : rien n'est pris.
        for stored in [
            "cart\t[{\"title\":\"Café\"}]",
            "cart\t[{\"title\":\"Café\",\"price\":\"5\",\"admin\":\"1\"}]",
            "cart\t[{\"title\":\"Café\",\"price\":5}]",
            "cart\t[\"Café\"]",
            "free\t[{\"key\":\"1\"}]",
            "tasks\t[{\"title\":\"Café\"}]",
        ] {
            assert_eq!(crate::from_visit(page, &start, stored), start, "{stored}");
        }
        let many: Vec<String> = (0..201).map(|i| format!("\"t{i}\"")).collect();
        assert_eq!(crate::from_visit(page, &start, &format!("tasks\t[{}]", many.join(","))), start);
        // Deux cents fiches de deux longs champs passent 64 Ko : la liste s'écrit vide, et la page
        // la retire de sa mémoire de visite plutôt que d'y laisser celle d'avant.
        let long = crate::lists::record(&[("title".to_string(), "x".repeat(200)), ("price".to_string(), "9".repeat(200))]);
        let big = crate::lists::write(&vec![("cart".to_string(), vec![long; 200])]);
        assert_eq!(values(page, &big), "tasks\t[\"Pain\"]\ncart\t\nfree\t[]");
    }

    #[test]
    fn what_the_visit_cannot_remember_is_refused_with_its_reason() {
        let refused = |visit: &str| {
            let page = format!("Page(state: State(firstName: \"\", count: 0, tab: \"a\", tasks: [\"pain\"]), keep: [count], address: [tab], computed: [ Filter(name: found, from: tasks, contains: firstName) ], visit: {visit}, children: [ P(\"{{firstName}} {{count}} {{tab}} {{tasks}} {{found}}\") ])");
            crate::check_page(&page).unwrap_err().message
        };
        assert!(refused("[count]").contains("est déjà gardée (keep)"), "{}", refused("[count]"));
        assert!(refused("[tab]").contains("est dans l'adresse (address)"), "{}", refused("[tab]"));
        assert!(refused("[found]").contains("liste calculée"), "{}", refused("[found]"));
        assert!(refused("[rien]").contains("aucune valeur ne s'appelle « rien »"), "{}", refused("[rien]"));
        assert!(refused("[firstName, firstName]").contains("écrit deux fois"), "{}", refused("[firstName, firstName]"));
        assert!(refused("[minute]").contains("l'heure du visiteur"), "{}", refused("[minute]"));
        assert!(refused("firstName").contains("attend la liste"), "{}", refused("firstName"));
        assert!(refused("[\"firstName\"]").contains("attend des noms"), "{}", refused("[\"firstName\"]"));
        // Une valeur partagée vient du serveur ; une valeur du nom du fichier, de l'adresse.
        let shared = crate::check_page("Page(shared: Shared(seats: 20), visit: [seats], children: [ P(\"{seats}\") ])").unwrap_err().message;
        assert!(shared.contains("est partagée, le serveur la garde pour tous"), "{shared}");
        let template = crate::address::joined("Page(visit: [id], children: [ P(\"{id}\") ])", &[("id".to_string(), "ada".to_string())]);
        assert!(crate::check_page(&template).unwrap_err().message.contains("par le nom du fichier"));
        // Un nombre qui peut descendre sous zéro (ADR-102) : la vérification des nombres négatifs le
        // refuse ici, car la visite relit ses nombres sans signe.
        let negative = crate::check_page("Page(state: State(temperature: -2), negative: [temperature], visit: [temperature], children: [ P(\"{temperature}\") ])").unwrap_err().message;
        assert!(negative.contains("peut descendre sous zéro"), "{negative}");
        // Ce que le serveur dit du membre connecté.
        let member = crate::check_page("Page(visit: [signedIn], children: [ P(\"{signedIn}\") ])").unwrap_err().message;
        assert!(member.contains("donné par le serveur"), "{member}");
        // Dans un monde : la mémoire de visite est celle de la page.
        let world = crate::check_page("Page(state: State(n: 0), children: [ Point(name: W, seed: 1, inside: World(visit: [n], children: [ P(\"{n}\") ])) ])").unwrap_err().message;
        assert!(world.contains("se règle sur la page, pas dans un monde"), "{world}");
        // Le message tombe à sa ligne.
        let error = crate::check_page("Page(\n  state: State(a: 0),\n  visit: [b],\n  children: [ P(\"{a}\") ],\n)").unwrap_err();
        assert_eq!(error.pos.line, 3);
    }

    #[test]
    fn the_light_page_knows_what_the_page_remembers() {
        // La page fabriquée par le serveur dit les noms : la page légère fait venir le moteur
        // seulement si l'onglet en retient déjà pour elle. Elle ne la rend pas vivante.
        let html = crate::flat_view(FIRST, "").unwrap();
        assert!(html.contains(" data-visit-names=\"firstName people price note workshop tasks\""), "{html}");
        assert!(!html.contains("data-live"), "{html}");
        assert!(!crate::flat_view("Page(state: State(a: 0), children: [ P(\"{a}\") ])", "").unwrap().contains("data-visit-names"));
        // Les deux pages de la leçon 136 sont acceptées, et retiennent les mêmes valeurs.
        let first = include_str!("../../exemples/lecons/136-se-souvenir-le-temps-d-une-visite.holo");
        let second = include_str!("../../exemples/lecons/136-inscription/etape-2.holo");
        crate::check_page(first).unwrap();
        crate::check_page(second).unwrap();
        assert_eq!(crate::visit_names(first), "prenom,personnes,atelier");
        assert_eq!(crate::visit_names(second), "prenom,personnes,atelier");
    }
}
