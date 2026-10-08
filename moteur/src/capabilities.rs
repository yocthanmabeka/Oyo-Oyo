//! Capacités du navigateur, explicites et bornées (propositions ADR-093 à ADR-096).
use crate::holo::{Block, Error, Program, Value};
use crate::{lists, modules, rules, state};
pub const BLOCKS: &[&str] = &["Transfer", "Device", "Notification", "Offline"];
pub const BYTES_MAX: usize = 65_536;

fn names(block: &Block) -> Result<Vec<&str>, Error> {
    let Some(Value::List(values)) = block.argument("values").map(|a| &a.value) else {
        return Err(Error { message: "« Transfer(values: …) » attend une liste de valeurs : values: [notes, title]".into(), pos: block.pos });
    };
    let mut names = Vec::new();
    for value in values {
        let Value::Name(name) = value else { return Err(Error { message: "« values » contient des noms de valeurs".into(), pos: block.pos }); };
        if names.contains(&name.as_str()) { return Err(Error { message: "une valeur n'est annoncée qu'une fois".into(), pos: block.pos }); }
        names.push(name.as_str());
    }
    if names.is_empty() || names.len() > 16 { return Err(Error { message: "de 1 à 16 valeurs à transférer".into(), pos: block.pos }); }
    Ok(names)
}
pub fn text<'a>(block: &'a Block, key: &str) -> Option<&'a str> {
    match block.argument(key).map(|a| &a.value) { Some(Value::Text(t)) => Some(t), _ => None }
}
pub fn word<'a>(block: &'a Block, key: &str) -> Option<&'a str> {
    match block.argument(key).map(|a| &a.value) { Some(Value::Name(t)) => Some(t), _ => None }
}
pub fn check(program: &Program) -> Result<(), Error> {
    rules::for_each_block(&program.root, &mut |block| {
        if !BLOCKS.contains(&block.name.as_str()) { return Ok(()); }
        let error = |message: &str| Error { message: message.into(), pos: block.pos };
        if rules::name_of(block).is_none() { return Err(error("une capacité attend son nom, name: Save")); }
        if !text(block, "label").is_some_and(|t| !t.trim().is_empty() && t.chars().count() <= 200) {
            return Err(error("une capacité attend une étiquette, label: \"Sauvegarder\" (200 caractères au plus)"));
        }
        // Les capacités d'un monde entré ne doivent pas continuer après le retour à la page.
        let mut direct = false;
        if let Some(Value::List(children)) = program.root.argument("children").map(|a| &a.value) {
            direct = children.iter().any(|v| matches!(v, Value::Block(b) if std::ptr::eq(b, block)));
        }
        if !direct { return Err(error("Transfer, Device, Notification et Offline se rangent directement dans les enfants de Page")); }
        match block.name.as_str() {
            "Transfer" => {
                for name in names(block)? {
                    let declared = program.root.argument("state").is_some_and(|a| matches!(&a.value, Value::Block(s) if s.argument(name).is_some()));
                    if !declared || program.shared.iter().any(|n| n == name) {
                        return Err(error("un transfert ne reçoit que des valeurs personnelles déclarées dans State, jamais l'heure, une valeur partagée ou une valeur calculée"));
                    }
                }
                if !text(block, "file").is_some_and(|f| f.ends_with(".json") && f.len() <= 80 && f.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))) {
                    return Err(error("« Transfer(file: …) » attend un simple nom de fichier .json"));
                }
            }
            "Device" => match word(block, "kind") {
                Some("position" | "clipboard") => {
                    if !word(block, "value").is_some_and(|n| state::initial_texts(program).iter().any(|(k, _)| k == n)) {
                        return Err(error("position et clipboard rendent un texte déclaré dans State, value: result"));
                    }
                }
                Some("camera" | "microphone") if block.argument("value").is_none() => {}
                _ => return Err(error("« Device(kind: …) » attend position, clipboard, camera ou microphone ; caméra et microphone donnent un aperçu local, sans value")),
            },
            "Notification" => {
                if !text(block, "title").is_some_and(|s| !s.trim().is_empty() && s.chars().count() <= 100)
                    || block.argument("body").is_some() && !text(block, "body").is_some_and(|s| s.chars().count() <= 200) {
                    return Err(error("une notification attend un titre de 100 caractères au plus et un texte de 200 caractères au plus"));
                }
                if let Some(a) = block.argument("after") {
                    if !matches!(&a.value, Value::Number{value, unit: Some(u), ..} if u == "s" && *value >= 0.0 && *value <= 3600.0) {
                        return Err(error("« Notification(after: …) » va de 0s à 3600s ; le rappel ne dure que tant que cette page reste ouverte"));
                    }
                }
            }
            "Offline" => {
                let Some(Value::List(files)) = block.argument("files").map(|a| &a.value) else { return Err(error("« Offline(files: …) » attend des fichiers locaux : files: [\"image.svg\"]")); };
                if files.len() > 16 || files.iter().any(|v| !matches!(v, Value::Text(f) if crate::flat::path_on(f) && !f.contains('?') && !f.contains('#') && !f.ends_with(".holo") && !f.ends_with(".html") && !f.ends_with(".js") && !f.ends_with(".wasm"))) {
                    return Err(error("16 ressources locales au plus ; pas d'autres pages, scripts ou modules : ceux du moteur sont ajoutés par le moteur"));
                }
                let mut private = program.root.argument("access").is_some() || !program.shared.is_empty() || program.root.argument("modules").is_some() || program.root.argument("keep").is_some();
                rules::for_each_block(&program.root, &mut |b| { private |= matches!(b.name.as_str(), "Form" | "Point" | "World" | "Sound" | "Video" | "Device" | "Notification" | "Transfer"); Ok(()) })?;
                if private { return Err(error("le premier hors-ligne garde une page publique locale : pas de compte, partage, formulaire, module, capture, son, vidéo ou autre capacité dans cette page")); }
                let source = format!("{:?}", program.root);
                if source.contains("signedIn") || source.contains("account") { return Err(error("une page hors-ligne ne lit pas un compte")); }
            }
            _ => {}
        }
        Ok(())
    })
}
pub fn is_offline(program: &Program) -> bool {
    let mut found = false;
    let _ = rules::for_each_block(&program.root, &mut |b| { found |= b.name == "Offline"; Ok(()) });
    found
}
pub fn html(block: &Block) -> String {
    let mut parts = Vec::new();
    for a in &block.arguments {
        let Some(key) = a.name.as_deref() else { continue };
        let val = match &a.value {
            Value::Text(t) | Value::Name(t) => crate::json_text(t),
            Value::List(vs) => format!("[{}]", vs.iter().filter_map(|v| match v { Value::Name(t) | Value::Text(t) => Some(crate::json_text(t)), _ => None }).collect::<Vec<_>>().join(",")),
            Value::Number { value, .. } => value.to_string(),
            _ => continue,
        };
        parts.push(format!("{}:{val}", crate::json_text(key)));
    }
    parts.push(format!("\"type\":{}", crate::json_text(&block.name)));
    format!("<section class=\"holo-{}\" data-name=\"{}\" data-browser-capability=\"{}\"><p>{}</p><p role=\"status\" aria-live=\"polite\" data-capability-status>Prêt.</p><span data-capability-preview></span><noscript>Cette capacité demande JavaScript ; le reste de la page reste lisible.</noscript></section>",
        block.name, crate::flat::escape(rules::name_of(block).unwrap_or("")), crate::flat::escape(&format!("{{{}}}", parts.join(","))), crate::flat::escape(text(block, "label").unwrap_or("")))
}
pub fn export(program: &Program, written: &str, name: &str) -> Result<String, String> {
    let b = rules::named_block(program, name).filter(|b| b.name == "Transfer" || b.name == "Device").ok_or("aucun transfert de ce nom")?;
    let values = if b.name == "Device" { vec![word(b, "value").ok_or("aucun texte à écrire")?] } else { names(b).map_err(|e| e.message)? };
    let module = modules::Module { name, source: "", inputs: values, outputs: vec![], time: 100, pages: 1 };
    let json = modules::input_json(program, &state::reread(program, written), &state::reread_texts(program, written), &lists::reread(program, written), &module);
    if json.len() > BYTES_MAX { return Err("transfert de plus de 64 Ko".into()); }
    Ok(json)
}
pub fn received(program: &Program, written: &str, name: &str, json: &str) -> Result<(state::State, state::Texts, lists::Lists), String> {
    use lists::Json;
    let b = rules::named_block(program, name).ok_or("capacité inconnue")?;
    let outputs = match b.name.as_str() { "Transfer" => names(b).map_err(|e| e.message)?, "Device" => vec![word(b, "value").ok_or("cet appareil ne rend pas de valeur")?], _ => return Err("cette capacité ne reçoit pas de valeurs".into()) };
    if json.len() > BYTES_MAX { return Err("transfert de plus de 64 Ko".into()); }
    let Some(Json::Object(keys)) = Json::read(json) else { return Err("un objet JSON est attendu".into()); };
    if keys.len() != outputs.len() { return Err("toutes les valeurs annoncées, une fois chacune".into()); }
    let mut seen = Vec::new();
    for (key, value) in &keys {
        if !outputs.contains(&key.as_str()) || seen.contains(&key.as_str()) { return Err("valeur inconnue ou répétée".into()); }
        seen.push(key.as_str());
        let valid = match (modules::sort_of(program, key), value) {
            (Some(modules::Sort::Number), Json::Number(n)) => *n <= state::VALUE_MAX,
            (Some(modules::Sort::Number), Json::Decimal(d)) => state::places(program, key) > 0 && state::parse_decimal(d, state::places(program, key)).is_some_and(|n| n <= state::VALUE_MAX),
            (Some(modules::Sort::Text), Json::Text(t)) => t.chars().count() <= 200 && !t.contains('\0'),
            (Some(modules::Sort::List), Json::Table(vs)) => vs.len() <= lists::ELEMENTS_MAX && vs.iter().all(|v| match v {
                Json::Text(t) => t.chars().count() <= lists::ELEMENT_MAX,
                Json::Object(fields) => fields.len() <= lists::FIELDS_MAX && fields.iter().all(|(_,v)| matches!(v, Json::Text(t) if t.chars().count() <= lists::ELEMENT_MAX)),
                _ => false
            }),
            _ => false
        };
        if !valid { return Err(format!("« {key} » : mauvaise sorte ou limite dépassée")); }
    }
    let numbers = state::reread(program, written);
    let texts = state::reread_texts(program, written);
    let old_lists = lists::reread(program, written);
    let (numbers, texts) = state::take_values(program, &numbers, &texts, json);
    let new_lists = lists::take_lists(program, &old_lists, json);
    Ok((numbers, texts, new_lists))
}
#[cfg(test)]
mod tests {
    #[test]
    fn imports_are_atomic_declared_and_bounded() {
        let source = r#"Page(state: State(note: "", n: 0, rows: []), children: [H1("Essai"), Transfer(name: File, label: "Mes valeurs", file: "notes.json", values: [note,n,rows]), Button(name: Load, text: "Importer")], rules: [On(Load.tap, effect: File.import)])"#;
        let p = crate::check_page(source).unwrap();
        let good = r#"{"note":"Bonjour","n":3,"rows":["A","B"]}"#;
        let (ns, ts, ls) = super::received(&p, "", "File", good).unwrap();
        assert!(ns.iter().any(|(k,n)| k == "n" && *n == 3));
        assert!(ts.iter().any(|(k,t)| k == "note" && t == "Bonjour"));
        assert_eq!(ls.iter().find(|(k,_)| k == "rows").unwrap().1.len(), 2);
        for bad in [r#"{"note":"A","n":1,"rows":[],"admin":1}"#, r#"{"note":"A","note":"B","rows":[]}"#, r#"{"note":"A","n":"1","rows":[]}"#, r#"{"note":"A","n":1}"#] {
            assert!(super::received(&p, "", "File", bad).is_err(), "{bad}");
        }
        let huge = format!(r#"{{"note":"{}","n":1,"rows":[]}}"#, "x".repeat(201));
        assert!(super::received(&p, "", "File", &huge).is_err());
        let forged = source.replace("On(Load.tap, effect: File.import)", "Every(1s, effect: File.import)");
        assert!(crate::check_page(&forged).is_err());
    }
    #[test]
    fn a_permission_never_comes_from_a_timer_or_completion() {
        let good = r#"Page(state: State(result: ""), children: [H1("Appareil"), Device(name: Location, kind: position, value: result, label: "Ma position"), Button(name: Ask, text: "Demander")], rules: [On(Ask.tap, effect: Location.request)])"#;
        assert!(crate::check_page(good).is_ok());
        for bad in [good.replace("Ask.tap", "Location.done"), good.replace("kind: position", "kind: arbitrary"), good.replace("value: result", "value: missing")] { assert!(crate::check_page(&bad).is_err()); }
    }
}