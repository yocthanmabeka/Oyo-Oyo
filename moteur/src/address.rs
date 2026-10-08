//! Les adresses qui portent des valeurs (ADR-078). Un fichier `profil/{id}.holo` sert toutes les
//! adresses `/profil/123`, `/profil/ada` ; la page lit `{id}` comme ses autres valeurs. Décision
//! de Yocthan du 2026-10-07 : l'adresse se dit par le nom du fichier.
//!
//! Les valeurs arrivent au moteur comme un petit fichier joint après la page et ses imports,
//! nommé `@adresse`, qui contient `id=123` (les morceaux d'adresse tels qu'ils sont dans l'URL).
//! Le serveur, la page et `holo check` passent tous par là : ils ne peuvent pas diverger.

use crate::holo::{Argument, Block, Error, Program, Value, NAME_SEPARATOR, NEXT_FILE};

/// Le nom du fichier joint qui porte les valeurs de l'adresse.
pub const ADDRESS_FILE: &str = "@adresse";

/// Une valeur d'adresse : 200 caractères au plus.
const LENGTH_MAX: usize = 200;

/// Les noms d'un modèle d'adresse, dans l'ordre : `/profil/{id}.holo` → [id] ;
/// `/{author}/notes/{note}.holo` → [author, note].
pub fn names(file: &str) -> Vec<String> {
    file.split(['/', '\\'])
        .filter_map(|part| part.strip_suffix(".holo").unwrap_or(part).strip_prefix('{')?.strip_suffix('}').map(str::to_string))
        .collect()
}

/// Un nom de valeur d'adresse s'écrit comme un nom de valeur (ADR-037) : `id`, `userName`.
pub fn valid_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase()) && name.len() <= 40 && name.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Les morceaux d'une adresse qui correspondent aux valeurs d'un modèle, tels qu'ils sont dans
/// l'URL : (`/profil/{id}.holo`, `/profil/123`) → [(id, 123)]. `None` si l'adresse ne
/// correspond pas au modèle, ou si un morceau est vide ou mal écrit.
pub fn values(pattern: &str, address: &str) -> Option<Vec<(String, String)>> {
    let pattern = pattern.strip_suffix(".holo")?;
    let wanted: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
    let got: Vec<&str> = address.trim_start_matches('/').trim_end_matches('/').split('/').collect();
    if wanted.len() != got.len() {
        return None;
    }
    let mut values = Vec::new();
    for (w, g) in wanted.iter().zip(&got) {
        match w.strip_prefix('{').and_then(|p| p.strip_suffix('}')) {
            Some(name) if valid_name(name) => {
                let decoded = decode(g)?;
                if decoded.is_empty() || decoded.chars().count() > LENGTH_MAX || decoded.chars().any(char::is_control) {
                    return None;
                }
                values.push((name.to_string(), (*g).to_string()));
            }
            Some(_) => return None,
            // Un morceau fixe se compare décodé : un dossier accentué (`écrits`) arrive encodé.
            None if decode(g).as_deref() == Some(*w) => {}
            None => return None,
        }
    }
    Some(values)
}

/// Le texte complet pour le moteur : la page (et ses imports), puis les valeurs de l'adresse.
pub fn joined(source: &str, values: &[(String, String)]) -> String {
    // Un « & » ou un « = » dans une valeur ne doit pas ouvrir une autre valeur.
    let pairs: Vec<String> = values.iter().map(|(name, raw)| format!("{name}={}", raw.replace('&', "%26").replace('=', "%3D"))).collect();
    format!("{source}{NEXT_FILE}{ADDRESS_FILE}{NAME_SEPARATOR}{}", pairs.join("&"))
}

/// Pour vérifier un modèle sans adresse : chaque nom, avec une valeur vide (`holo check`).
pub fn empty_values(file: &str) -> Vec<(String, String)> {
    names(file).into_iter().map(|name| (name, String::new())).collect()
}

/// `%C3%A9` → « é ». `None` pour un `%` mal écrit ou des octets qui ne font pas un texte.
pub fn decode(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Les valeurs de l'adresse deviennent des textes de la page, comme `State(id: "123")` : la page
/// les lit (`{id}`, `If(id, is: "ada")`), sans pouvoir les changer.
pub fn inject(program: &mut Program, text: &str) -> Result<(), Error> {
    let pos = program.root.pos;
    let refusal = |message: String| Error { message, pos };
    if program.root.name != "Page" {
        return Err(refusal("une adresse qui porte une valeur (un fichier nommé comme « {id}.holo ») demande une page : Page(…)".into()));
    }
    let mut pairs: Vec<(String, String)> = Vec::new();
    for pair in text.split('&').filter(|p| !p.is_empty()) {
        let (name, raw) = pair.split_once('=').unwrap_or((pair, ""));
        if !valid_name(name) {
            return Err(refusal(format!("« {{{name}}} » : le nom d'une valeur d'adresse s'écrit comme une valeur, une minuscule d'abord, sans « _ » ni « - » : « {{id}} », « {{userName}} » (ADR-037)")));
        }
        let value = decode(raw).filter(|v| v.chars().count() <= LENGTH_MAX && !v.chars().any(char::is_control)).ok_or_else(|| refusal(format!("la valeur de « {name} » dans l'adresse est mal écrite ou trop longue ({LENGTH_MAX} caractères au plus)")))?;
        if pairs.iter().any(|(known, _)| known == name) {
            return Err(refusal(format!("« {{{name}}} » est deux fois dans l'adresse : chaque valeur a son nom")));
        }
        pairs.push((name.to_string(), value));
    }
    let given = |name: &str, value: &str, pos| Argument { name: Some(name.to_string()), value: Value::Text(value.to_string()), pos };
    match program.root.arguments.iter_mut().find(|a| a.name.as_deref() == Some("state")) {
        Some(Argument { value: Value::Block(state), .. }) if state.name == "State" => {
            for (name, value) in &pairs {
                if state.arguments.iter().any(|a| a.name.as_deref() == Some(name.as_str())) {
                    return Err(Error { message: format!("« {name} » vient de l'adresse (le nom du fichier) : ne la déclare pas aussi dans State"), pos: state.pos });
                }
                state.arguments.push(given(name, value, state.pos));
            }
        }
        Some(_) => return Err(refusal("« state » attend un bloc « State(...) », sur la page : state: State(cart: 0)".into())),
        None => {
            let arguments = pairs.iter().map(|(name, value)| given(name, value, pos)).collect();
            program.root.arguments.push(Argument { name: Some("state".into()), value: Value::Block(Block { name: "State".into(), styles: Vec::new(), arguments, pos }), pos });
        }
    }
    program.address = pairs.into_iter().map(|(name, _)| name).collect();
    // Une valeur d'adresse se lit ; elle ne se change pas, ni par une règle, ni par un champ.
    let names = program.address.clone();
    crate::rules::for_each_block(&program.root, &mut |block| {
        let changed = match block.name.split_once('.') {
            Some((target, _)) if crate::state::is_requested(block) => Some(target.to_string()),
            _ if ["Input", "Checkbox", "Choice", "Slider"].contains(&block.name.as_str()) => match block.argument("value").map(|a| &a.value) {
                Some(Value::Name(bound)) => Some(bound.clone()),
                _ => None,
            },
            _ => None,
        };
        match changed {
            Some(name) if names.contains(&name) => Err(Error { message: format!("« {name} » vient de l'adresse de la page (le nom du fichier) : on la lit, on ne la change pas"), pos: block.pos }),
            _ => Ok(()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_carries_values() {
        assert_eq!(names("/exemples/profil/{id}.holo"), ["id"]);
        assert_eq!(names("/{author}/notes/{note}.holo"), ["author", "note"]);
        assert!(names("/exemples/lecons/01-page.holo").is_empty());
        let raw = |pairs: Option<Vec<(String, String)>>| pairs.map(|p| p.into_iter().map(|(n, v)| format!("{n}={v}")).collect::<Vec<_>>().join("&"));
        assert_eq!(raw(values("/profil/{id}.holo", "/profil/123")).as_deref(), Some("id=123"));
        assert_eq!(raw(values("/{author}/notes/{note}.holo", "/ada/notes/7/")).as_deref(), Some("author=ada&note=7"));
        assert_eq!(raw(values("/profil/{id}.holo", "/profil/Ad%C3%A9")).as_deref(), Some("id=Ad%C3%A9"));
        // Ce qui ne correspond pas au modèle, un morceau vide ou mal écrit : rien.
        assert!(values("/profil/{id}.holo", "/profil").is_none());
        assert!(values("/profil/{id}.holo", "/profil/1/2").is_none());
        assert!(values("/profil/{id}.holo", "/autre/1").is_none());
        assert!(values("/profil/{id}.holo", "/profil/%ZZ").is_none());
        assert!(values("/profil/{user_id}.holo", "/profil/1").is_none());
        assert_eq!(decode("Ad%C3%A9").as_deref(), Some("Adé"));
        // Un dossier accentué ; un « & » dans une valeur n'en ouvre pas une autre.
        assert_eq!(raw(values("/écrits/{id}.holo", "/%C3%A9crits/7")).as_deref(), Some("id=7"));
        let program = crate::check_page(&joined("Page(children: [ H1(\"{id}\") ])", &[("id".into(), "a&b=c".into())])).unwrap();
        assert_eq!(program.address, ["id"]);
        assert!(crate::flat_view(&joined("Page(children: [ H1(\"{id}\") ])", &[("id".into(), "a&b=c".into())]), "").unwrap().contains(">a&amp;b=c<"));
    }

    #[test]
    fn the_page_reads_its_address_and_never_changes_it() {
        let page = "Page(children: [ H1(\"Profil n° {id}\"), If(id, is: \"ada\", children: [ P(\"Bonjour, Ada\") ]) ])";
        // Lue, comme un texte de la page.
        let html = crate::flat_view(&joined(page, &[("id".into(), "ada".into())]), "").unwrap();
        assert!(html.contains("Profil n° <span data-state=\"id\">ada</span>"), "{html}");
        assert!(html.contains("Bonjour, Ada"), "{html}");
        let html = crate::flat_view(&joined(page, &[("id".into(), "Ad%C3%A9".into())]), "").unwrap();
        assert!(html.contains(">Adé<"), "{html}");
        // Vérifiée sans adresse : chaque nom vaut un texte vide.
        crate::check_page(&joined(page, &empty_values("/profil/{id}.holo"))).unwrap();
        // Sans adresse, `{id}` n'est pas une valeur.
        assert!(crate::check_page(page).is_err());
        // Refusé : la changer, la déclarer deux fois, un mauvais nom, une page qui n'en est pas une.
        let refused = |source: &str, text: &str| crate::check_page(&format!("{source}{NEXT_FILE}{ADDRESS_FILE}{NAME_SEPARATOR}{text}")).unwrap_err().message;
        assert!(refused("Page(children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: id.set(\"x\")) ])", "id=1").contains("on la lit, on ne la change pas"));
        assert!(refused("Page(children: [ Input(value: id, label: \"x\") ])", "id=1").contains("on la lit, on ne la change pas"));
        assert!(refused("Page(state: State(id: \"\"), children: [ \"x\" ])", "id=1").contains("ne la déclare pas aussi dans State"));
        assert!(refused("Page(children: [ \"x\" ])", "user_id=1").contains("une minuscule d'abord"));
        assert!(refused("Point(name: A, seed: 1)", "id=1").contains("demande une page"));
    }
}
