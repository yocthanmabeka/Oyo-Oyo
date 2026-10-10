//! Capacités du navigateur, explicites et bornées (propositions ADR-093 à ADR-096 ; la notification
//! push, ADR-119).
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
/// Ce que les règles du fichier demandent à un bloc : `On(Go.tap, effect: Share.request)` → `request`.
fn asked_of<'a>(program: &'a Program, name: &str) -> Vec<&'a str> {
    let mut asked = Vec::new();
    let _ = rules::for_each_block(&program.root, &mut |rule| {
        let effects: Vec<&Value> = match rule.argument("effect").map(|a| &a.value) { Some(Value::List(vs)) => vs.iter().collect(), Some(v) => vec![v], None => Vec::new() };
        for effect in effects {
            if let Some((target, action)) = match effect { Value::Name(n) => n.split_once('.'), _ => None } {
                if target == name { asked.push(action); }
            }
        }
        Ok(())
    });
    asked
}
/// Ce que les règles du fichier écoutent d'un bloc : `On(Buzz.done, …)` → `done`.
fn heard_of<'a>(program: &'a Program, name: &str) -> Vec<&'a str> {
    let mut heard = Vec::new();
    let _ = rules::for_each_block(&program.root, &mut |rule| {
        if rule.name == "On" {
            if let Some(Value::Name(signal)) = rule.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value) {
                if let Some((source, word)) = signal.split_once('.') {
                    if source == name { heard.push(word); }
                }
            }
        }
        Ok(())
    });
    heard
}
/// Les règles `On` qui demandent `action` à ce bloc, et le signal de chacune : `On(Post.tap,
/// effect: [posts.add(1), News.send])` → `Post.tap` pour `News` et `send`.
fn triggers_of<'a>(program: &'a Program, name: &str, action: &str) -> Vec<&'a str> {
    let mut found = Vec::new();
    let wanted = format!("{name}.{action}");
    let _ = rules::for_each_block(&program.root, &mut |rule| {
        let asks = match rule.argument("effect").map(|a| &a.value) {
            Some(Value::List(vs)) => vs.iter().any(|v| matches!(v, Value::Name(n) if *n == wanted)),
            Some(Value::Name(n)) => *n == wanted,
            _ => false,
        };
        if let (true, "On", Some(Value::Name(signal))) = (asks, rule.name.as_str(), rule.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value)) {
            found.push(signal.as_str());
        }
        Ok(())
    });
    found
}
/// Une notification qui prévient aussi quand la page est fermée (ADR-119) : `Notification(push: true)`.
pub fn is_push(block: &Block) -> bool {
    block.name == "Notification" && matches!(block.argument("push").map(|a| &a.value), Some(Value::Bool(true)))
}
/// Les notifications push d'une page (ADR-119) : leur nom, leur titre et leur texte, tels que
/// l'auteur les a écrits. Rien d'autre ne part jamais dans un message.
pub fn push_blocks(program: &Program) -> Vec<(String, String, String)> {
    let mut found = Vec::new();
    let _ = rules::for_each_block(&program.root, &mut |block| {
        if is_push(block) {
            found.push((rules::name_of(block).unwrap_or("").to_string(), text(block, "title").unwrap_or("").to_string(), text(block, "body").unwrap_or("").to_string()));
        }
        Ok(())
    });
    found
}
/// Les notifications push qu'un toucher envoie (ADR-119) : `On(Post.tap, effect: [posts.add(1),
/// News.send])` → `News`. C'est holo serve qui les envoie, quand il accepte le toucher.
pub fn push_sent(program: &Program, signal: &str) -> Vec<String> {
    rules::effects(program, signal)
        .into_iter()
        .filter_map(|effect| effect.strip_suffix(".send").map(str::to_string))
        .filter(|name| rules::named_block(program, name).is_some_and(is_push))
        .collect()
}
/// Ce nom est-il celui d'une vibration (ADR-110) ? Elle se joue alors comme un son, là où un son se joue.
pub fn vibrates(program: &Program, name: &str) -> bool {
    rules::named_block(program, name).is_some_and(|b| b.name == "Device" && word(b, "kind") == Some("vibration"))
}
/// Le motif d'une vibration, en millisecondes (ADR-110) : `for: 200ms` donne [200] ; une liste
/// alterne vibration et silence. 200 ms si rien n'est écrit. Rien si le motif sort des bornes :
/// chaque durée avec son unité, dix durées au plus, une seconde en tout au plus.
pub fn pattern(block: &Block) -> Option<Vec<u64>> {
    let ms = |v: &Value| match v {
        Value::Number { value, unit: Some(u), .. } if u == "ms" => Some(value.round()),
        Value::Number { value, unit: Some(u), .. } if u == "s" => Some((value * 1000.0).round()),
        _ => None,
    }
    .filter(|m| (1.0..=1000.0).contains(m))
    .map(|m| m as u64);
    let durations = match block.argument("for").map(|a| &a.value) {
        None => vec![200],
        Some(Value::List(vs)) => vs.iter().map(ms).collect::<Option<Vec<u64>>>()?,
        Some(v) => vec![ms(v)?],
    };
    (!durations.is_empty() && durations.len() <= 10 && durations.iter().sum::<u64>() <= 1000).then_some(durations)
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
        // Ce qu'on demande à chaque sorte d'appareil : un partage s'ouvre (request, ADR-107), une
        // vibration se joue (play, ADR-110) ; stop, permis partout, arrête ou oublie ce qui est en cours.
        if block.name == "Device" {
            let (name, kind) = (rules::name_of(block).unwrap_or(""), word(block, "kind").unwrap_or(""));
            if kind != "vibration" && block.argument("for").is_some() {
                return Err(error("« for: » dit la durée d'une vibration : Device(kind: vibration, for: 200ms)"));
            }
            for action in asked_of(program, name) {
                let refused = match kind {
                    "share" => (!matches!(action, "request" | "stop")).then(|| format!("« {name}.{action} » : un partage s'ouvre par {name}.request, sur le toucher d'un bouton")),
                    "vibration" => (!matches!(action, "play" | "stop")).then(|| format!("« {name}.{action} » : une vibration se joue par {name}.play, comme un son, et s'arrête par {name}.stop")),
                    _ => (action == "play").then(|| format!("« {name}.play » : seule une vibration se joue ; cet appareil se demande par {name}.request")),
                };
                if let Some(message) = refused { return Err(error(&message)); }
            }
            // Une vibration ne dit rien en retour (ADR-110) : ce qui compte se montre à l'écran.
            if kind == "vibration" {
                if let Some(signal) = heard_of(program, name).into_iter().next() {
                    return Err(error(&format!("« {name}.{signal} » : une vibration ne dit rien en retour ; montre à l'écran ce qui s'est passé, dans la règle qui la joue")));
                }
            }
        }
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
                // Partager la page (ADR-107) : son titre et son adresse ; rien n'est rendu à la page.
                Some("share") if block.argument("value").is_none() => {}
                // Faire vibrer le téléphone (ADR-110) : une durée, ou un motif, bornés.
                Some("vibration") if block.argument("value").is_none() => {
                    if pattern(block).is_none() {
                        return Err(error("« for: » attend une durée, for: 200ms, ou une liste qui alterne vibration et silence, for: [100ms, 50ms, 100ms] : dix durées au plus, une seconde en tout au plus"));
                    }
                }
                _ => return Err(error("« Device(kind: …) » attend position, clipboard, camera, microphone, share ou vibration ; caméra et microphone donnent un aperçu local, le partage envoie le titre et l'adresse de la page, une vibration se sent : sans value")),
            },
            "Notification" => {
                // Prévenir aussi quand la page est fermée (ADR-119) : le visiteur s'abonne sur un toucher
                // (request), se désabonne (stop) ; holo serve prévient les abonnés (send) quand il
                // accepte un toucher qui change une valeur partagée, le seul qu'il voit toujours passer.
                let name = rules::name_of(block).unwrap_or("");
                match block.argument("push").map(|a| &a.value) {
                    None => {
                        if let Some(action) = asked_of(program, name).into_iter().find(|a| matches!(*a, "request" | "send")) {
                            return Err(error(&format!("« {name}.{action} » : une notification locale se montre par {name}.show, page ouverte ; pour prévenir aussi quand la page est fermée, ajoute push: true")));
                        }
                    }
                    Some(Value::Bool(true)) => {
                        if block.argument("after").is_some() {
                            return Err(error("une notification push part quand holo serve l'envoie, après un toucher : sans « after: » ; un rappel à heure fixe, page ouverte, reste une notification sans push"));
                        }
                        if asked_of(program, name).contains(&"show") {
                            return Err(error(&format!("« {name}.show » : une notification push ne se montre pas d'ici ; {name}.request abonne le visiteur, {name}.send prévient les abonnés, {name}.stop désabonne")));
                        }
                        for signal in triggers_of(program, name, "send") {
                            let shared = crate::state::touched_ones(program, crate::lists::signal_and_line(signal).0).iter().any(|v| program.shared.contains(v)) || crate::shared::changes_shared_line(program, signal);
                            if !shared {
                                return Err(error(&format!("« {name}.send » part de holo serve, avec un changement partagé : écris-le dans la règle d'un bouton qui change une valeur de Shared, comme On(Post.tap, effect: [posts.add(1), {name}.send])")));
                            }
                        }
                    }
                    Some(_) => return Err(error("« push: » attend true : Notification(…, push: true) prévient aussi quand la page est fermée ; sans push, la notification reste locale")),
                }
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
                let mut private = program.root.argument("access").is_some() || !program.shared.is_empty() || program.root.argument("modules").is_some() || program.root.argument("keep").is_some() || program.root.argument("data").is_some() || !program.imports.is_empty();
                rules::for_each_block(&program.root, &mut |b| { private |= matches!(b.name.as_str(), "Form" | "Point" | "World" | "Sound" | "Video" | "Device" | "Notification" | "Transfer"); Ok(()) })?;
                if private { return Err(error("le premier hors-ligne garde une page publique locale : pas de compte, partage, données reçues, import, formulaire, module, capture, son, vidéo ou autre capacité dans cette page")); }
                let source = format!("{:?}", program.root);
                if source.contains("signedIn") || source.contains("account") { return Err(error("une page hors-ligne ne lit pas un compte")); }
            }
            _ => {}
        }
        Ok(())
    })
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
    // Le motif d'une vibration, en millisecondes (ADR-110).
    if word(block, "kind") == Some("vibration") {
        let durations = pattern(block).unwrap_or_default();
        parts.push(format!("\"pattern\":[{}]", durations.iter().map(u64::to_string).collect::<Vec<_>>().join(",")));
    }
    // Une notification push (ADR-119) : la page s'abonne chez holo serve, qui seul envoie.
    if is_push(block) {
        parts.push("\"push\":true".into());
    }
    parts.push(format!("\"type\":{}", crate::json_text(&block.name)));
    // Sans JavaScript, le partage dit comment partager quand même (ADR-107) ; rien ne vibre (ADR-110).
    let without_script = match word(block, "kind") {
        Some("share") => "Sans JavaScript, ce bouton ne partage pas : copie l'adresse de la page dans la barre du navigateur, ou prends « Partager » dans son menu.",
        Some("vibration") => "Sans JavaScript, le téléphone ne vibre pas ; le reste de la page reste lisible.",
        _ if is_push(block) => "Sans JavaScript, cette page ne peut pas te prévenir quand elle est fermée ; tout le reste marche, et elle montre les nouveautés quand tu la rouvres.",
        _ => "Cette capacité demande JavaScript ; le reste de la page reste lisible.",
    };
    format!("<section class=\"holo-{}\" data-name=\"{}\" data-browser-capability=\"{}\"><p>{}</p><p role=\"status\" aria-live=\"polite\" data-capability-status>Prêt.</p><span data-capability-preview></span><noscript>{without_script}</noscript></section>",
        block.name, crate::flat::escape(rules::name_of(block).unwrap_or("")), crate::flat::escape(&format!("{{{}}}", parts.join(","))), crate::flat::escape(text(block, "label").unwrap_or("")))
}
pub fn export(program: &Program, written: &str, name: &str) -> Result<String, String> {
    let b = rules::named_block(program, name).filter(|b| b.name == "Transfer" || b.name == "Device").ok_or("aucun transfert de ce nom")?;
    let values = if b.name == "Device" { vec![word(b, "value").ok_or("aucun texte à écrire")?] } else { names(b).map_err(|e| e.message)? };
    let module = modules::Module { name, source: "", inputs: values, outputs: vec![], time: 100, pages: 1, sha256: None, from: None, license: None };
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
            (Some(modules::Sort::List), Json::Table(vs)) => {
                let kind = lists::kind(program, key).ok_or("liste inconnue")?;
                let clean = |t: &str| t.chars().count() <= lists::ELEMENT_MAX && !t.chars().any(char::is_control);
                vs.len() <= lists::ELEMENTS_MAX && vs.iter().all(|v| match (v, &kind) {
                    (Json::Text(t), lists::Kind::Texts | lists::Kind::Free) => !t.is_empty() && clean(t),
                    (Json::Object(fields), lists::Kind::Records(_) | lists::Kind::Free) => {
                        let mut seen = Vec::new();
                        !fields.is_empty() && fields.len() <= lists::FIELDS_MAX
                            && fields.iter().all(|(k,v)| {
                                let unique = !seen.contains(&k.as_str()); seen.push(k.as_str());
                                unique && k.len() <= 40 && k != "key"
                                    && k.starts_with(|c: char| c.is_ascii_lowercase())
                                    && k.chars().all(|c| c.is_ascii_alphanumeric())
                                    && matches!(v, Json::Text(t) if clean(t))
                            })
                            && match &kind { lists::Kind::Records(expected) => fields.len() == expected.len() && expected.iter().all(|n| fields.iter().any(|(k,_)| k == n)), _ => true }
                    }
                    _ => false
                })
            },
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
    fn imports_do_not_fire_completion_twice_or_drop_record_fields() {
        let source = r#"Page(state: State(count: 0, rows: [Item(title: "A", price: 1)]), children: [H1("Essai"), Transfer(name: File, label: "Mes valeurs", file: "rows.json", values: [rows]), Button(name: Load, text: "Importer")], rules: [On(Load.tap, effect: File.import), On(File.done, effect: count.add(1))])"#;
        let state = crate::capability_received(source, "", "File", r#"{"rows":[{"title":"B","price":"2"}]}"#).unwrap();
        let p = crate::check_page(source).unwrap();
        assert_eq!(crate::state::reread(&p, &state).iter().find(|(k,_)| k == "count").unwrap().1, 0);
        let after = crate::arbitrate(source, &state, "File.done");
        assert_eq!(crate::state::reread(&p, &after).iter().find(|(k,_)| k == "count").unwrap().1, 1);
        for bad in [r#"{"rows":["wrong kind"]}"#, r#"{"rows":[{"title":"B"}]}"#, r#"{"rows":[{"title":"B","price":"2","extra":"x"}]}"#, r#"{"rows":[{"title":"B","price":"2","price":"3"}]}"#] {
            assert!(crate::capability_received(source, &after, "File", bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn a_permission_never_comes_from_a_timer_or_completion() {
        let good = r#"Page(state: State(result: ""), children: [H1("Appareil"), Device(name: Location, kind: position, value: result, label: "Ma position"), Button(name: Ask, text: "Demander")], rules: [On(Ask.tap, effect: Location.request)])"#;
        assert!(crate::check_page(good).is_ok());
        for bad in [good.replace("Ask.tap", "Location.done"), good.replace("kind: position", "kind: arbitrary"), good.replace("value: result", "value: missing")] { assert!(crate::check_page(&bad).is_err()); }
    }
    #[test]
    fn a_page_is_shared_from_a_button_and_nothing_else() {
        // Partager la page (ADR-107) : sur le toucher d'un bouton ; la page sait que c'est fait, ou raté.
        let good = r#"Page(state: State(sent: 0, missed: 0), children: [H1("Partage"), Device(name: Share, kind: share, label: "Partager cette page"), Button(name: Go, text: "Partager")], rules: [On(Go.tap, effect: Share.request), On(Share.done, effect: sent.add(1)), On(Share.failed, effect: missed.add(1))])"#;
        assert!(crate::check_page(good).is_ok());
        let html = crate::flat_view(good, "").unwrap();
        assert!(html.contains("&quot;kind&quot;:&quot;share&quot;") && html.contains("<p role=\"status\" aria-live=\"polite\" data-capability-status>"), "{html}");
        // Sans JavaScript, la page dit comment partager quand même.
        assert!(html.contains("<noscript>Sans JavaScript, ce bouton ne partage pas : copie l'adresse de la page dans la barre du navigateur"), "{html}");
        // Jamais d'une minuterie, d'une fin ou d'une règle qui guette ; rien d'autre que s'ouvrir ; aucune valeur rendue.
        for (bad, why) in [
            (good.replace("On(Go.tap, effect: Share.request)", "Every(1s, effect: Share.request)"), "un geste du visiteur"),
            (good.replace("On(Go.tap, effect: Share.request)", "On(Share.done, effect: Share.request)"), "le toucher d'un bouton"),
            (good.replace("On(Go.tap, effect: Share.request)", "When(sent, is: 1, effect: Share.request)"), "un geste du visiteur"),
            (good.replace("On(Go.tap, effect: Share.request)", "On(Go.tap, effect: Share.write)"), "un partage s'ouvre par Share.request"),
            (good.replace("kind: share", "kind: share, value: sent"), "sans value"),
        ] {
            let error = crate::check_page(&bad).unwrap_err();
            assert!(error.message.contains(why), "{bad} : {error}");
        }
    }
    #[test]
    fn a_vibration_plays_like_a_sound_and_says_nothing_back() {
        // Faire vibrer (ADR-110) : d'un bouton, d'une touche, d'une règle de temps, d'une rencontre,
        // comme un son, sans permission ; elle ne dit rien en retour.
        let good = r#"Page(state: State(n: 0, x: 10, y: 50), children: [H1("Vibrer"), Device(name: Buzz, kind: vibration, for: [100ms, 80ms, 100ms], label: "Deux vibrations"), Button(name: Go, text: "Vibrer"), Board(height: 200px, children: [Shape(name: Me, form: square, size: 40px, x: x, y: y), Shape(name: Goal, form: diamond, size: 40px, x: 80, y: 50)])], rules: [On(Go.tap, effect: [n.add(1), Buzz.play]), On(Key.space, effect: Buzz.play), On(Key.right, effect: x.add(5)), Every(2s, effect: Buzz.play), When(Me, meets: Goal, effect: [n.add(1), Buzz.play]), When(n, is: 1, effect: Buzz.play), When(n, is: 3, effect: Buzz.stop)])"#;
        crate::check_page(good).unwrap();
        let html = crate::flat_view(good, "").unwrap();
        assert!(html.contains("&quot;pattern&quot;:[100,80,100]") && html.contains("<noscript>Sans JavaScript, le téléphone ne vibre pas"), "{html}");
        // Une règle qui guette joue la vibration comme un son : l'arbitre la rend sous « ! ».
        let after = crate::arbitrate(good, &crate::initial_state(good), "Go.tap");
        assert!(after.split(';').any(|chunk| chunk == "!=Buzz.play"), "{after}");
        for (bad, why) in [
            (good.replace("On(Go.tap, effect: [n.add(1), Buzz.play])", "On(Go.tap, effect: Buzz.request)"), "une vibration se joue par Buzz.play"),
            (good.replace("Every(2s, effect: Buzz.play)", "On(Buzz.done, effect: n.add(1))"), "ne dit rien en retour"),
            (good.replace("Every(2s, effect: Buzz.play)", "On(Buzz.failed, effect: n.add(1))"), "ne dit rien en retour"),
            (good.replace("kind: vibration, for: [100ms, 80ms, 100ms]", "kind: vibration, value: n"), "sans value"),
        ] {
            let error = crate::check_page(&bad).unwrap_err();
            assert!(error.message.contains(why), "{bad} : {error}");
        }
        // Seule une vibration se joue : un autre appareil se demande.
        let position = r#"Page(state: State(at: ""), children: [H1("Où"), Device(name: Where, kind: position, value: at, label: "Ma position"), Button(name: Go, text: "Où suis-je ?")], rules: [On(Go.tap, effect: Where.play)])"#;
        assert!(crate::check_page(position).unwrap_err().message.contains("seule une vibration se joue"));
        // Une règle de temps ne demande toujours pas une permission.
        assert!(crate::check_page(&position.replace("On(Go.tap, effect: Where.play)", "Every(1s, effect: Where.request)")).unwrap_err().message.contains("un geste du visiteur"));
    }
    #[test]
    fn a_vibration_is_short() {
        let page = |device: &str| format!(r#"Page(children: [H1("Vibrer"), {device}, Button(name: Go, text: "Vibrer")], rules: [On(Go.tap, effect: Buzz.play)])"#);
        // 200 ms si rien n'est écrit ; une durée en secondes devient des millisecondes.
        assert!(crate::flat_view(&page(r#"Device(name: Buzz, kind: vibration, label: "Vibrer")"#), "").unwrap().contains("&quot;pattern&quot;:[200]"));
        assert!(crate::flat_view(&page(r#"Device(name: Buzz, kind: vibration, for: 0.5s, label: "Vibrer")"#), "").unwrap().contains("&quot;pattern&quot;:[500]"));
        for (device, why) in [
            (r#"Device(name: Buzz, kind: vibration, for: 200, label: "Vibrer")"#, "attend une durée"),
            (r#"Device(name: Buzz, kind: vibration, for: 0ms, label: "Vibrer")"#, "attend une durée"),
            (r#"Device(name: Buzz, kind: vibration, for: 2s, label: "Vibrer")"#, "une seconde en tout au plus"),
            (r#"Device(name: Buzz, kind: vibration, for: [600ms, 100ms, 600ms], label: "Vibrer")"#, "une seconde en tout au plus"),
            (r#"Device(name: Buzz, kind: vibration, for: [10ms, 10ms, 10ms, 10ms, 10ms, 10ms, 10ms, 10ms, 10ms, 10ms, 10ms], label: "Vibrer")"#, "dix durées au plus"),
            (r#"Device(name: Buzz, kind: camera, for: 200ms, label: "Vibrer")"#, "la durée d'une vibration"),
        ] {
            let error = crate::check_page(&page(device)).unwrap_err();
            assert!(error.message.contains(why), "{device} : {error}");
        }
    }
    #[test]
    fn a_push_notification_is_asked_by_a_tap_and_sent_with_a_shared_change() {
        // Prévenir quand la page est fermée (ADR-119) : s'abonner et se désabonner sur un toucher ;
        // prévenir les abonnés depuis la règle d'un bouton qui change une valeur partagée.
        let good = r#"Page(state: State(mine: 0), shared: Shared(posts: 0), children: [H1("Le club"), P("{posts}"), Button(name: Post, text: "Épingler"), Button(name: Mine, text: "À moi"), Notification(name: News, label: "Être prévenu", title: "Le club", body: "Un nouveau message.", push: true), Button(name: Follow, text: "Me prévenir"), Button(name: Unfollow, text: "Ne plus me prévenir")], rules: [On(Post.tap, effect: [posts.add(1), News.send]), On(Mine.tap, effect: mine.add(1)), On(Follow.tap, effect: News.request), On(Unfollow.tap, effect: News.stop)])"#;
        let program = crate::check_page(good).unwrap();
        assert_eq!(super::push_blocks(&program), [("News".to_string(), "Le club".to_string(), "Un nouveau message.".to_string())]);
        assert_eq!(super::push_sent(&program, "Post.tap"), ["News"]);
        assert!(super::push_sent(&program, "Follow.tap").is_empty() && super::push_sent(&program, "Mine.tap").is_empty());
        // La page dit seulement qu'elle prévient ; sans JavaScript, elle dit ce qui manque.
        let html = crate::flat_view(good, "").unwrap();
        assert!(html.contains("&quot;push&quot;:true") && html.contains("<noscript>Sans JavaScript, cette page ne peut pas te prévenir quand elle est fermée"), "{html}");
        for (bad, why) in [
            // Jamais d'une minuterie, d'une règle qui guette, ni de la fin d'autre chose.
            (good.replace("On(Follow.tap, effect: News.request)", "Every(10s, effect: News.request)"), "un geste du visiteur"),
            (good.replace("On(Follow.tap, effect: News.request)", "When(posts, over: 3, effect: News.request)"), "un geste du visiteur"),
            (good.replace("On(Follow.tap, effect: News.request)", "On(News.done, effect: News.request)"), "le toucher d'un bouton"),
            (good.replace("On(Post.tap, effect: [posts.add(1), News.send])", "On(Post.tap, effect: posts.add(1)), Every(1s, effect: News.send)"), "un geste du visiteur"),
            (good.replace("On(Post.tap, effect: [posts.add(1), News.send])", "On(Post.tap, effect: posts.add(1)), When(posts, over: 3, effect: News.send)"), "un geste du visiteur"),
            // Prévenir part de holo serve avec un changement partagé, le seul toucher qu'il voit toujours.
            (good.replace("On(Mine.tap, effect: mine.add(1))", "On(Mine.tap, effect: [mine.add(1), News.send])"), "avec un changement partagé"),
            // Une notification push ne se montre pas d'ici, n'attend pas ; push vaut true ou rien.
            (good.replace("On(Unfollow.tap, effect: News.stop)", "On(Unfollow.tap, effect: News.show)"), "ne se montre pas d'ici"),
            (good.replace("push: true", "push: true, after: 3s"), "sans « after: »"),
            (good.replace("push: true", "push: false"), "« push: » attend true"),
            (good.replace("push: true", "push: yes"), "« push: » attend true"),
            // Une notification locale ne s'abonne pas et ne prévient personne.
            (good.replace(", push: true", ""), "ajoute push: true"),
            (good.replace("On(Post.tap, effect: [posts.add(1), News.send])", "On(Post.tap, effect: posts.add(1))").replace(", push: true", ""), "ajoute push: true"),
            (good.replace("News.request", "News.play"), "capacité inconnue"),
        ] {
            let error = crate::check_page(&bad).unwrap_err();
            assert!(error.message.contains(why), "{bad} : {error}");
        }
        // Le premier hors-ligne ne garde pas une page qui prévient.
        let offline = good.replace("Button(name: Follow", "Offline(name: Copy, label: \"Copie\", files: []), Button(name: Follow");
        assert!(crate::check_page(&offline).is_err());
    }
}