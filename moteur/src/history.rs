//! L'historique dans une page (ADR-091). Une page écrit des valeurs dans son adresse, après le
//! `?` : `Page(address: [tab, page])` donne `galerie.holo?tab=photos&page=2`. Chaque toucher qui
//! les change fait un pas dans l'historique du navigateur, que « Précédent » défait ; l'adresse
//! se partage, et qui l'ouvre arrive sur les mêmes valeurs.
//!
//! Ce qui arrive par l'adresse vient de n'importe qui : comme les valeurs gardées (`keep`), seules
//! celles que la page nomme sont reprises, dans leurs bornes ; une valeur mal écrite part de son
//! départ, sans erreur. Le serveur, la page et `holo html` passent tous par ici.

use crate::holo::{Error, Program, Value};
use crate::state::{State, Texts};

/// Une valeur de texte dans l'adresse : 200 caractères au plus, comme celles d'un modèle
/// d'adresse (ADR-078). Plus longue, elle reste dans la page, pas dans l'adresse.
const LENGTH_MAX: usize = 200;

/// Les réglages que le moteur lit déjà dans une adresse (`?values`, `?view=points`…) : une page
/// ne peut pas y ranger ses valeurs.
const RESERVED: &[&str] = &["values", "bare", "webgl", "world", "view", "enter", "zoom", "x", "y", "yaw", "pitch", "measures"];

/// Les valeurs que la page écrit dans son adresse (`address: [tab, page]`), vérifiées : des
/// nombres ou des textes déclarés dans `State`, ni une liste, ni l'heure, ni une valeur gardée
/// (`keep`), ni une valeur qui vient déjà du nom du fichier (ADR-078).
pub fn names(program: &Program) -> Result<Vec<String>, Error> {
    let Some(argument) = program.root.argument("address") else { return Ok(Vec::new()) };
    let error = |message: String| Error { message, pos: argument.pos };
    let Value::List(written) = &argument.value else {
        return Err(error("« address » attend la liste des valeurs que l'adresse porte : address: [tab]".into()));
    };
    let numbers = crate::state::initial(program)?;
    let texts = crate::state::initial_texts(program);
    let kept = crate::state::kept_values(program)?;
    let mut names = Vec::new();
    for name in written {
        let Value::Name(name) = name else {
            return Err(error("« address » attend des noms de valeurs : address: [tab]".into()));
        };
        let message = if crate::state::CLOCK.contains(&name.as_str()) || name == crate::dates::TODAY {
            format!("« address » : « {name} » est l'heure du visiteur, elle ne va pas dans l'adresse")
        } else if program.address.contains(name) {
            format!("« address » : « {name} » vient déjà de l'adresse, par le nom du fichier (ADR-078)")
        } else if RESERVED.contains(&name.as_str()) {
            format!("« address » : « {name} » sert déjà au moteur dans une adresse (?{name}) ; choisis un autre nom")
        } else if crate::lists::is_list(program, name) || crate::computed::is_computed(program, name) {
            format!("« address » : « {name} » est une liste ; une adresse ne porte que des nombres et des textes, garde plutôt la liste (keep)")
        } else if kept.contains(name) {
            format!("« address » : « {name} » est déjà gardé (keep) ; une valeur vient de l'adresse ou du navigateur, pas des deux")
        } else if names.contains(name) {
            format!("« address » : « {name} » est écrit deux fois")
        } else if numbers.iter().any(|(known, _)| known == name) || texts.iter().any(|(known, _)| known == name) {
            names.push(name.clone());
            continue;
        } else {
            format!("« address » : aucune valeur ne s'appelle « {name} » ; l'adresse ne porte que des valeurs déclarées dans « State »")
        };
        return Err(error(message));
    }
    Ok(names)
}

/// Les valeurs de l'adresse (`tab=photos&page=2`, telle qu'après le `?`), posées sur l'état :
/// celles que la page nomme, et seulement elles ; une valeur absente ou mal écrite reprend sa
/// valeur de départ (« Précédent » revient aussi à l'onglet du début, que l'adresse ne dit pas).
pub fn from_query(program: &Program, numbers: &State, texts: &Texts, query: &str) -> (State, Texts) {
    let (mut numbers, mut texts) = (numbers.clone(), texts.clone());
    let names = names(program).unwrap_or_default();
    if names.is_empty() {
        return (numbers, texts);
    }
    let given = pairs(query);
    let written = |name: &str| given.iter().rev().find(|(known, _)| known == name).map(|(_, value)| value.as_str());
    let (start, start_texts) = (crate::state::initial(program).unwrap_or_default(), crate::state::initial_texts(program));
    for name in &names {
        if let Some((_, place)) = numbers.iter_mut().find(|(known, _)| known == name) {
            let departure = start.iter().find(|(known, _)| known == name).map_or(0, |(_, v)| *v);
            *place = written(name).and_then(|w| number(program, name, w)).unwrap_or(departure);
        } else if let Some((_, place)) = texts.iter_mut().find(|(known, _)| known == name) {
            let departure = start_texts.iter().find(|(known, _)| known == name).map(|(_, v)| v.clone()).unwrap_or_default();
            *place = written(name).filter(|w| text_ok(program, name, w)).map_or(departure, str::to_string);
        }
    }
    (numbers, texts)
}

/// L'adresse nomme-t-elle une valeur de la page (`?page=2`) ? Une adresse nue, ou qui ne porte
/// que des réglages du moteur (`?values`), ne dit rien des valeurs : une page qui arrive avec
/// ses données (ADR-064) les garde.
pub fn names_a_value(program: &Program, query: &str) -> bool {
    let names = names(program).unwrap_or_default();
    pairs(query).iter().any(|(name, _)| names.contains(name))
}

/// L'adresse que demandent les valeurs de la page, après le `?` : `tab=photos&page=2`. Une
/// valeur à son départ n'y est pas : la page du début garde son adresse nue.
pub fn query(program: &Program, numbers: &State, texts: &Texts) -> String {
    let (start, start_texts) = (crate::state::initial(program).unwrap_or_default(), crate::state::initial_texts(program));
    let mut pieces = Vec::new();
    for name in names(program).unwrap_or_default() {
        if let Some((_, value)) = numbers.iter().find(|(known, _)| *known == name) {
            if start.iter().any(|(known, v)| *known == name && v != value) {
                pieces.push(format!("{name}={}", written_number(*value, crate::state::places(program, &name))));
            }
        } else if let Some((_, value)) = texts.iter().find(|(known, _)| *known == name) {
            if start_texts.iter().any(|(known, v)| *known == name && v != value) && value.chars().count() <= LENGTH_MAX {
                pieces.push(format!("{name}={}", encode(value)));
            }
        }
    }
    pieces.join("&")
}

/// Un nombre de l'adresse, à l'échelle de la valeur (`12.50` → 1250 pour deux chiffres après la
/// virgule), dans les bornes que la page lui donne.
fn number(program: &Program, name: &str, written: &str) -> Option<u64> {
    let places = crate::state::places(program, name);
    let units = if places > 0 { crate::state::parse_decimal(written, places)? } else { written.parse::<u64>().ok()? };
    Some(units.min(crate::state::ceiling(program, name)).max(crate::state::floor(program, name)))
}

/// Un texte de l'adresse : 200 caractères au plus, sans caractère invisible ; et, pour une
/// valeur que la page n'écrit qu'avec des mots fixes, l'un de ces mots.
fn text_ok(program: &Program, name: &str, written: &str) -> bool {
    if written.chars().count() > LENGTH_MAX || written.chars().any(char::is_control) {
        return false;
    }
    known_texts(program, name).is_none_or(|known| known.iter().any(|text| text == written))
}

/// Les textes que la page peut donner à une valeur, quand elle n'en écrit que d'avance : son
/// départ, ses `set("…")`, les options d'un `Choice`. Une adresse forgée (`?tab=pirate`) ne met
/// pas la page dans un état qu'elle ne peut pas atteindre. `None` : un texte libre, qu'un champ,
/// une autre valeur ou des données reçues peuvent écrire.
fn known_texts(program: &Program, name: &str) -> Option<Vec<String>> {
    if crate::state::data_source(program).ok().flatten().is_some() {
        return None;
    }
    let mut known: Vec<String> = crate::state::initial_texts(program).into_iter().filter(|(known, _)| known == name).map(|(_, text)| text).collect();
    let mut free = false;
    let bound = |block: &crate::holo::Block| matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(value)) if value == name);
    let _ = crate::rules::for_each_block(&program.root, &mut |block| {
        match block.name.split_once('.') {
            Some((target, verb)) if target == name && crate::state::is_requested(block) => match (verb, block.arguments.first().map(|a| &a.value)) {
                ("set", Some(Value::Text(text))) if !text.contains('{') => known.push(text.clone()),
                _ => free = true,
            },
            _ if block.name == "Choice" && bound(block) => known.extend(crate::state::choice_options(block).into_iter().map(str::to_string)),
            _ if bound(block) => free = true,
            _ => {}
        }
        Ok(())
    });
    (!free).then_some(known)
}

/// 1250 à deux chiffres après la virgule → « 12.50 » ; 3 → « 3 ».
fn written_number(units: u64, places: u32) -> String {
    if places == 0 {
        return units.to_string();
    }
    let scale = crate::state::scale(places);
    format!("{}.{:0width$}", units / scale, units % scale, width = places as usize)
}

/// Les paires d'une adresse après le `?`, décodées comme le fait un navigateur (`+` est une
/// espace). Une paire mal encodée est laissée de côté.
fn pairs(query: &str) -> Vec<(String, String)> {
    query
        .trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.is_empty())
        .filter_map(|pair| {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            Some((crate::address::decode(&name.replace('+', " "))?, crate::address::decode(&value.replace('+', " "))?))
        })
        .collect()
}

/// Encode un texte pour l'adresse : lettres, chiffres et `-_.~` tels quels, le reste en `%XX`
/// (« Adé » → `Ad%C3%A9`), comme `encodeURIComponent`.
fn encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    const GALLERY: &str = "Page(
  title: \"Galerie\",
  state: State(tab: \"toiles\", page: 1, price: 12.50, artist: \"\", note: \"\"),
  address: [tab, page, price, artist],
  children: [
    H1(\"Galerie\"),
    P(\"{tab}, page {page}, {price} {artist}\"),
    Button(name: Next, text: \"Suivante\"),
    Choice(value: tab, label: \"Onglet\", options: [\"toiles\", \"aquarelles\", \"dessins\"]),
    Input(value: artist, label: \"Artiste\"),
  ],
  rules: [ On(Next.tap, effect: page.add(1)) ],
)";

    #[test]
    fn the_address_carries_the_values_the_page_names() {
        // Au départ, l'adresse est nue ; après deux pages, elle dit la page.
        let start = crate::initial_state(GALLERY);
        assert_eq!(crate::address_query(GALLERY, &start), "");
        let later = crate::arbitrate(GALLERY, &crate::arbitrate(GALLERY, &start, "Next.tap"), "Next.tap");
        assert_eq!(crate::address_query(GALLERY, &later), "page=3");
        // Un texte accentué, un nombre à virgule.
        let state = crate::from_query(GALLERY, &start, "tab=aquarelles&page=4&price=9.5");
        assert_eq!(crate::address_query(GALLERY, &state), "tab=aquarelles&page=4&price=9.50");
        // Ce que l'adresse ne nomme pas reprend son départ : « Précédent » revient au début.
        let back = crate::from_query(GALLERY, &state, "");
        assert_eq!(back, start);
        // Ce que la page ne nomme pas n'est jamais repris ; une valeur mal écrite part de son départ.
        let forged = crate::from_query(GALLERY, &start, "note=pirate&page=abc&tab=inconnu&values");
        assert_eq!(forged, start);
        // Un texte que la page n'écrit qu'avec des mots fixes n'en prend pas d'autre.
        let tabs = "Page(state: State(tab: \"toiles\", query: \"\"), address: [tab, query], children: [ P(\"{tab}\"), Input(value: query, label: \"Chercher\"), Button(name: Ink, text: \"Encres\") ], rules: [ On(Ink.tap, effect: tab.set(\"encres\")) ])";
        let tabs_start = crate::initial_state(tabs);
        assert_eq!(crate::address_query(tabs, &crate::from_query(tabs, &tabs_start, "tab=pirate")), "");
        assert_eq!(crate::address_query(tabs, &crate::from_query(tabs, &tabs_start, "tab=encres&query=pirate")), "tab=encres&query=pirate");
        // `+` est une espace, `%C3%A9` un « é », et l'adresse les écrit comme un navigateur.
        let named = crate::from_query(GALLERY, &start, "artist=Ad%C3%A9+L.");
        assert_eq!(crate::address_query(GALLERY, &named), "artist=Ad%C3%A9%20L.");
        // Le serveur fabrique la page avec ces valeurs.
        let html = crate::flat_view_at(GALLERY, "", "page=7&tab=dessins&artist=Zo%C3%A9").unwrap();
        assert!(html.contains("<span data-state=\"page\">7</span>") && html.contains("<span data-state=\"artist\">Zoé</span>"), "{html}");
        assert!(html.contains("value=\"dessins\" data-bind=\"tab\" checked"), "{html}");
        // Les noms, pour le navigateur.
        assert_eq!(crate::address_names(GALLERY), "tab,page,price,artist");
    }

    #[test]
    fn the_address_refuses_what_it_cannot_carry() {
        let refused = |address: &str| {
            let page = format!("Page(state: State(tab: \"a\", count: 0, cart: [\"pain\"]), keep: [count], address: {address}, children: [ P(\"{{tab}} {{count}}\") ])");
            crate::check_page(&page).unwrap_err().message
        };
        assert!(refused("[cart]").contains("est une liste"), "{}", refused("[cart]"));
        assert!(refused("[count]").contains("déjà gardé (keep)"), "{}", refused("[count]"));
        assert!(refused("[minute]").contains("l'heure du visiteur"), "{}", refused("[minute]"));
        assert!(refused("[rien]").contains("aucune valeur ne s'appelle « rien »"), "{}", refused("[rien]"));
        assert!(refused("[tab, tab]").contains("écrit deux fois"), "{}", refused("[tab, tab]"));
        assert!(refused("tab").contains("attend la liste"), "{}", refused("tab"));
        let reserved = crate::check_page("Page(state: State(view: \"a\"), address: [view], children: [ P(\"{view}\") ])").unwrap_err().message;
        assert!(reserved.contains("sert déjà au moteur"), "{reserved}");
        // Une valeur qui vient du nom du fichier (ADR-078) est déjà dans l'adresse.
        let template = crate::address::joined("Page(address: [id], children: [ P(\"{id}\") ])", &[("id".to_string(), "ada".to_string())]);
        assert!(crate::check_page(&template).unwrap_err().message.contains("par le nom du fichier"));
    }

    #[test]
    fn the_address_wins_over_the_data_served_with_the_page() {
        // Les données servies d'abord (ADR-064), puis l'adresse, que le visiteur a choisie.
        let page = "Page(state: State(tab: \"a\", page: 1), address: [page], data: Data(from: \"d.json\"), children: [ P(\"{tab} {page}\"), Button(name: Next, text: \"+\") ], rules: [ On(Next.tap, effect: page.add(1)) ])";
        let json = r#"{"page": 5, "tab": "b"}"#;
        let html = crate::flat_view_with_data_at(page, "", json, "page=9").unwrap();
        assert!(html.contains("<span data-state=\"page\">9</span>") && html.contains("<span data-state=\"tab\">b</span>"), "{html}");
        // Sans adresse : les données ; sans données : l'adresse.
        let html = crate::flat_view_with_data_at(page, "", json, "").unwrap();
        assert!(html.contains("<span data-state=\"page\">5</span>"), "{html}");
        // Une adresse qui ne nomme aucune valeur de la page (un réglage du moteur) : les données aussi.
        let html = crate::flat_view_with_data_at(page, "", json, "values").unwrap();
        assert!(html.contains("<span data-state=\"page\">5</span>"), "{html}");
        let html = crate::flat_view_with_data_at(page, "", "pas du JSON", "page=9").unwrap();
        assert!(html.contains("<span data-state=\"page\">9</span>") && html.contains("<span data-state=\"tab\">a</span>"), "{html}");
    }
}
