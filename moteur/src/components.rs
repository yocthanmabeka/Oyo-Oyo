//! Les composants (ADR-050) : un bloc écrit une fois, avec des paramètres, posé comme un bloc
//! ordinaire, à la manière d'un widget Flutter, et restylé par le CSS, à la manière du web.
//!
//! ```holo
//! Page(
//!   state: State(cart: 0),
//!   components: [
//!     Component(
//!       name: ArticleCard,
//!       params: [title, price, image],
//!       children: [ Column.card(children: [ Image(source: image, alt: title), H3("{title}"), Text("{price} euros"), Button(name: Add, text: "Add") ]) ],
//!       rules: [ On(Add.tap, effect: cart.add(price)) ],
//!     ),
//!   ],
//!   children: [
//!     ArticleCard(name: Sunrise, title: "Sunrise", price: 120, image: "sunrise.png"),
//!     ArticleCard.promo(name: Night, title: "Night", price: 60, image: "night.png"),
//!   ],
//! )
//!
//! ArticleCard { --accent: #E9B44C; }
//! .promo { --accent: crimson; }
//! ```
//!
//! Comme `Use` et `Repeat`, un composant est déplié à la lecture : le reste du moteur ne voit que
//! les blocs qu'on aurait écrits à la main.
//! - Un paramètre s'emploie par son nom : `title` à la place d'une valeur, `{title}` dans un texte.
//!   Donné par le nom d'une valeur de la page (`count: sunrise`), il la suit : `{count}` montre
//!   la valeur, `count.add(1)` la change.
//! - Un bloc nommé dans le composant reçoit le nom de la copie : `Add` devient `AddSunrise`.
//! - Les règles du composant rejoignent celles de la page (ou du monde), une fois par copie.
//! - Le bloc racine de la copie porte la marque du composant (`ArticleCard { … }` le vise) et
//!   les noms de style écrits à l'appel (`ArticleCard.promo` : `.promo { … }` le vise).
//! - Un emplacement pour du contenu (ADR-058) : `params: [title, children]` ; la page donne
//!   `Card(title: "…", children: [ P("…") ])`, et le mot `children` posé seul dans une liste du
//!   composant devient ces blocs, comme le `children` d'un widget Flutter.

use crate::holo::{Argument, Block, Error, Pos, Value};

/// `Part` s'appelle `Component` depuis ADR-056 : le mot `Part` est gardé pour la 3D (une pièce
/// d'un objet, comme dans Roblox). L'ancienne écriture est refusée avec le bon mot.
pub const OLD_PART: &str = "« Part » s'appelle maintenant « Component » (ADR-056) : écris « Component » ; le mot « Part » est gardé pour la 3D";

/// Le nombre de paramètres d'un composant, au plus.
pub const PARAMS_MAX: usize = 16;
/// La profondeur des composants posés les uns dans les autres, au plus.
pub const DEPTH_MAX: usize = 8;
/// Le nombre de copies de composants qu'une page peut poser, au plus.
pub const COPIES_MAX: usize = 2_000;

/// Les mots qu'un paramètre ne peut pas porter : ils ont déjà un sens dans le langage.
const RESERVED_WORDS: &[&str] = &[
    "name", "children", "rules", "params", "item", "key", "add", "sub", "set", "mul", "div", "push", "remove", "clear", "random", "enter", "leave",
    "play", "portals", "tap", "hover", "hoverEnd", "count", "total", "year", "month", "day", "weekday", "hour", "minute", "true", "false",
];

/// Un composant lu : son nom, ses paramètres, son contenu, ses règles.
#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub params: Vec<String>,
    /// Les valeurs par défaut : `params: [title, price: 0]` (ADR-056).
    pub defaults: Vec<(String, Value)>,
    /// Les signaux que le composant émet : `emits: [add]` ; la page les branche à l'appel,
    /// `onAdd: cart.add(1)` (ADR-056).
    pub emitted: Vec<String>,
    /// Le composant a un emplacement pour du contenu : `params: [title, children]` (ADR-058).
    pub slot: bool,
    pub children: Vec<Value>,
    pub rules: Vec<Value>,
    pub pos: Pos,
}

/// Lit un `Component(name:, params:, children:, rules:)`.
pub fn read_component(part: &Block) -> Result<Component, Error> {
    let example = "Component(name: ArticleCard, params: [title, price], children: [ Column(children: [ H3(\"{title}\"), Text(\"{price} euros\") ]) ])";
    let refusal = |message: String, pos: Pos| Err(Error { message, pos });
    let (mut name, mut params, mut children, mut rules) = (None, Vec::new(), None, Vec::new());
    let mut defaults: Vec<(String, Value)> = Vec::new();
    let mut emitted: Vec<String> = Vec::new();
    let mut slot = false;
    for argument in &part.arguments {
        match (argument.name.as_deref(), &argument.value) {
            (Some("name"), Value::Name(n)) => name = Some(n.clone()),
            (Some("name"), _) => return refusal(format!("le nom d'un composant s'écrit sans guillemets, avec une majuscule : {example}"), argument.pos),
            (Some("params"), Value::List(list)) => {
                for value in list {
                    match value {
                        Value::Name(p) if p == "children" => slot = true,
                        Value::Name(p) if p == "child" => return refusal("un seul mot pour le contenu, au pluriel même pour un bloc : params: [title, children]".into(), argument.pos),
                        Value::Name(p) => params.push(p.clone()),
                        // `price: 0` : un paramètre facultatif, et sa valeur par défaut.
                        Value::Block(b) if b.name == crate::holo::NAMED_VALUE => {
                            let Some(Argument { name: Some(p), value: default_value, .. }) = b.arguments.first() else { continue };
                            if p == "children" {
                                return refusal("« children » n'a pas de valeur par défaut : sans contenu donné, l'emplacement reste vide ; écris params: [title, children]".into(), b.pos);
                            }
                            if !matches!(default_value, Value::Text(_) | Value::Integer(_) | Value::Number { .. } | Value::Name(_) | Value::Bool(_)) {
                                return refusal(format!("la valeur par défaut de « {p} » est un texte, un nombre ou un nom, pas un bloc ni une liste"), b.pos);
                            }
                            params.push(p.clone());
                            defaults.push((p.clone(), default_value.clone()));
                        }
                        _ => return refusal("« params » est une liste de noms, avec ou sans valeur par défaut : params: [title, price: 0]".into(), argument.pos),
                    }
                }
            }
            (Some("emits"), Value::List(list)) => {
                for value in list {
                    match value {
                        Value::Name(e) if e.starts_with(|c: char| c.is_ascii_lowercase()) && e.chars().all(|c| c.is_ascii_alphanumeric()) && !emitted.contains(e) => emitted.push(e.clone()),
                        _ => return refusal("« emits » est une liste de signaux, en minuscules, chacun une fois : emits: [add, remove]".into(), argument.pos),
                    }
                }
            }
            (Some("children"), Value::List(list)) => children = Some(list.clone()),
            (Some("rules"), Value::List(list)) => rules = list.clone(),
            (Some(word @ ("params" | "emits" | "children" | "rules")), _) => return refusal(format!("« Component({word}: …) » attend une liste entre crochets : {example}"), argument.pos),
            (Some(other), _) => return refusal(format!("« Component » n'a pas de réglage « {other} » ; réglages possibles : name, params, emits, children, rules"), argument.pos),
            (None, _) => return refusal(format!("chaque réglage de « Component » est nommé : {example}"), argument.pos),
        }
    }
    let (Some(name), Some(children)) = (name, children) else {
        return refusal(format!("un morceau a un nom et un contenu : {example}"), part.pos);
    };
    if !name.starts_with(|c: char| c.is_ascii_uppercase()) || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return refusal(format!("« {name} » : le nom d'un composant s'écrit comme un bloc, avec une majuscule et sans « _ », comme « ArticleCard » (ADR-037)"), part.pos);
    }
    if crate::blocks::BLOCKS.contains(&name.as_str()) {
        return refusal(format!("« {name} » est déjà un bloc du langage ; choisis un autre nom de composant"), part.pos);
    }
    if params.len() > PARAMS_MAX {
        return refusal(format!("« {name} » a trop de paramètres : {PARAMS_MAX} au plus"), part.pos);
    }
    for (i, p) in params.iter().enumerate() {
        if !p.starts_with(|c: char| c.is_ascii_lowercase()) || !p.chars().all(|c| c.is_ascii_alphanumeric()) {
            return refusal(format!("« {p} » : un paramètre s'écrit comme une valeur, en minuscules, les mots joints par une majuscule, comme « title » ou « oldPrice » (ADR-037)"), part.pos);
        }
        if RESERVED_WORDS.contains(&p.as_str()) {
            return refusal(format!("« {p} » est un mot du langage ; choisis un autre nom de paramètre"), part.pos);
        }
        if params[..i].contains(p) {
            return refusal(format!("le paramètre « {p} » est écrit deux fois dans « {name} »"), part.pos);
        }
    }
    for rule in &rules {
        let Value::Block(b) = rule else { return refusal("« Component(rules: …) » range des règles : rules: [ On(Add.tap, effect: cart.add(1)) ]".into(), part.pos) };
        if !matches!(b.name.as_str(), "On" | "Every" | "When" | "After" | "If") {
            return refusal("« Component(rules: …) » range des règles : rules: [ On(Add.tap, effect: cart.add(1)) ]".into(), part.pos);
        }
        for signal in emitted_signals(b) {
            if !emitted.contains(&signal) {
                return refusal(format!("« emit: {signal} » : déclare ce signal dans le composant, emits: [{signal}]"), b.pos);
            }
        }
    }
    let mut standalone = 0;
    count_slots(&Value::List(children.clone()), true, &mut standalone);
    if slot && standalone == 0 {
        return refusal(format!("« {name} » déclare « children » sans le poser : écris le mot seul là où va le contenu, Column(children: [ H2(\"{{title}}\"), children ])"), part.pos);
    }
    if !slot && standalone > 0 {
        return refusal(format!("« {name} » pose « children » sans le déclarer : écris params: [{}children]", params.iter().map(|p| format!("{p}, ")).collect::<String>()), part.pos);
    }
    Ok(Component { name, params, defaults, emitted, slot, children, rules, pos: part.pos })
}

/// Compte les `children` posés seuls dans les listes du contenu : là où ira le contenu donné.
fn count_slots(value: &Value, in_list: bool, standalone: &mut usize) {
    match value {
        Value::Name(n) if n == "children" && in_list => *standalone += 1,
        Value::List(l) => l.iter().for_each(|v| count_slots(v, true, standalone)),
        Value::Block(b) => b.arguments.iter().for_each(|a| count_slots(&a.value, false, standalone)),
        _ => {}
    }
}

/// Met le contenu donné à l'appel à la place du mot `children` posé seul dans une liste.
fn fill_slot(value: &mut Value, content: &[Value]) {
    match value {
        Value::List(l) => {
            let mut filled = Vec::with_capacity(l.len() + content.len());
            for mut v in std::mem::take(l) {
                if matches!(&v, Value::Name(n) if n == "children") {
                    filled.extend(content.iter().cloned());
                } else {
                    fill_slot(&mut v, content);
                    filled.push(v);
                }
            }
            *l = filled;
        }
        Value::Block(b) => b.arguments.iter_mut().for_each(|a| fill_slot(&mut a.value, content)),
        _ => {}
    }
}

/// Les signaux qu'une règle émet : `emit: add`, ou `emit: [add, remove]`.
fn emitted_signals(rule: &Block) -> Vec<String> {
    match rule.argument("emit").map(|a| &a.value) {
        Some(Value::Name(n)) => vec![n.clone()],
        Some(Value::List(l)) => l.iter().filter_map(|v| if let Value::Name(n) = v { Some(n.clone()) } else { None }).collect(),
        _ => Vec::new(),
    }
}

/// `add` → `onAdd` : le nom du branchement d'un signal, à l'appel.
fn wiring(signal: &str) -> String {
    let mut letters = signal.chars();
    format!("on{}", letters.next().map(|c| c.to_ascii_uppercase().to_string() + letters.as_str()).unwrap_or_default())
}

/// Remplace `emit: add` dans une règle de la copie par ce que la page a branché (`onAdd:`).
/// Une règle qui n'émet que des signaux non branchés, et ne fait rien d'autre, disparaît.
fn wire(mut rule: Value, branches: &[(String, Value)]) -> Option<Value> {
    let Value::Block(block) = &mut rule else { return Some(rule) };
    let signals = emitted_signals(block);
    if signals.is_empty() {
        return Some(rule);
    }
    block.arguments.retain(|a| a.name.as_deref() != Some("emit"));
    let mut effects: Vec<Value> = match block.argument("effect").map(|a| a.value.clone()) {
        Some(Value::List(l)) => l,
        Some(v) => vec![v],
        None => Vec::new(),
    };
    for signal in &signals {
        if let Some((_, value)) = branches.iter().find(|(s, _)| s == signal) {
            match value {
                Value::List(l) => effects.extend(l.iter().cloned()),
                v => effects.push(v.clone()),
            }
        }
    }
    if effects.is_empty() {
        return None;
    }
    block.arguments.retain(|a| a.name.as_deref() != Some("effect"));
    let pos = block.pos;
    block.arguments.push(Argument { name: Some("effect".into()), value: if effects.len() == 1 { effects.remove(0) } else { Value::List(effects) }, pos });
    Some(rule)
}

/// Retire les composants écrits dans la page (`components: [ Component(…) ]`) et les rend.
pub fn take_components(page: &mut Block) -> Result<Vec<Component>, Error> {
    if let Some(old) = page.arguments.iter().find(|a| a.name.as_deref() == Some("parts")) {
        return Err(Error { message: "« parts » s'appelle maintenant « components » (ADR-056) : écris « components »".into(), pos: old.pos });
    }
    let Some(i) = page.arguments.iter().position(|a| a.name.as_deref() == Some("components")) else { return Ok(Vec::new()) };
    let argument = page.arguments.remove(i);
    if page.name != "Page" {
        return Err(Error { message: "« components » s'écrit dans la page : Page(components: [ Component(…) ])".into(), pos: argument.pos });
    }
    let Value::List(list) = argument.value else {
        return Err(Error { message: "« components » est une liste de composants : components: [ Component(name: ArticleCard, …) ]".into(), pos: argument.pos });
    };
    let mut components = Vec::new();
    for value in &list {
        match value {
            Value::Block(b) if b.name == "Component" => components.push(read_component(b)?),
            Value::Block(b) if b.name == "Part" => return Err(Error { message: OLD_PART.into(), pos: b.pos }),
            _ => return Err(Error { message: "« components » ne contient que des « Component(…) »".into(), pos: argument.pos }),
        }
    }
    Ok(components)
}

/// Les noms des valeurs de la page (`State(cart: 0)`) : un paramètre ne peut pas en porter un.
fn page_values(page: &Block) -> Vec<String> {
    match page.argument("state").map(|a| &a.value) {
        Some(Value::Block(state)) => state.arguments.iter().filter_map(|a| a.name.clone()).collect(),
        _ => Vec::new(),
    }
}

/// Pose les composants d'un site (une page, ou un monde) et de tout ce qu'il contient.
pub fn place_site(site: &mut Block, components: &[Component]) -> Result<(), Error> {
    if components.is_empty() {
        return Ok(());
    }
    for component in components {
        for p in &component.params {
            if page_values(site).contains(p) {
                return Err(Error { message: format!("le paramètre « {p} » de « {} » porte le nom d'une valeur de la page ; choisis un autre nom", component.name), pos: component.pos });
            }
        }
    }
    let mut rules = Vec::new();
    let mut copies = 0;
    let mut unnamed = Vec::new();
    for argument in &mut site.arguments {
        place_value(&mut argument.value, components, &mut rules, &mut Vec::new(), &mut copies, &mut unnamed)?;
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

fn place_value(value: &mut Value, components: &[Component], rules: &mut Vec<Value>, path: &mut Vec<String>, copies: &mut usize, unnamed: &mut Vec<String>) -> Result<(), Error> {
    match value {
        // Le monde d'un point est un autre site : ses règles restent chez lui.
        Value::Block(block) if block.name == "World" && path.is_empty() => place_site(block, components),
        Value::Block(block) if components.iter().any(|c| c.name == block.name) => {
            Err(Error { message: format!("« {} » se pose parmi des blocs : children: [ {}(…) ]", block.name, block.name), pos: block.pos })
        }
        // Dans le modèle d'une répétition, les règles d'un composant rejoignent celles de la
        // répétition : elles y sont écrites une fois par élément, avec « item ».
        Value::Block(block) if block.name == "Repeat" => {
            let mut local_rules = Vec::new();
            for argument in &mut block.arguments {
                place_value(&mut argument.value, components, &mut local_rules, path, copies, unnamed)?;
            }
            if !local_rules.is_empty() {
                match block.arguments.iter_mut().find(|a| a.name.as_deref() == Some("rules")) {
                    Some(Argument { value: Value::List(list), .. }) => list.extend(local_rules),
                    Some(argument) => return Err(Error { message: "« Repeat(rules: …) » attend une liste entre crochets".into(), pos: argument.pos }),
                    None => block.arguments.push(Argument { name: Some("rules".into()), value: Value::List(local_rules), pos: block.pos }),
                }
            }
            Ok(())
        }
        Value::Block(block) => block.arguments.iter_mut().try_for_each(|a| place_value(&mut a.value, components, rules, path, copies, unnamed)),
        Value::List(elements) => {
            let mut placed_list = Vec::with_capacity(elements.len());
            for element in std::mem::take(elements) {
                match element {
                    Value::Block(call) if components.iter().any(|c| c.name == call.name) => {
                        let component = components.iter().find(|c| c.name == call.name).expect("trouvé juste au-dessus");
                        if path.contains(&component.name) {
                            return Err(Error { message: format!("« {} » se pose lui-même, directement ou par un autre composant : il ne finirait jamais", component.name), pos: call.pos });
                        }
                        if path.len() >= DEPTH_MAX {
                            return Err(Error { message: format!("trop de composants les uns dans les autres : {DEPTH_MAX} au plus"), pos: call.pos });
                        }
                        *copies += 1;
                        if *copies > COPIES_MAX {
                            return Err(Error { message: format!("la page pose plus de {COPIES_MAX} composants : c'est trop pour une page"), pos: call.pos });
                        }
                        // Le contenu donné appartient à la page : on le déplie là où il est écrit,
                        // avant de le poser dans la copie (une carte peut contenir une carte).
                        let mut call = call;
                        if let Some(argument) = call.arguments.iter_mut().find(|a| a.name.as_deref() == Some("children")) {
                            place_value(&mut argument.value, components, rules, path, copies, unnamed)?;
                        }
                        let (root, copy_rules) = place_copy(component, &call, unnamed)?;
                        path.push(component.name.clone());
                        // La racine peut être elle-même un composant : on la pose comme un élément de liste.
                        let mut single_one = Value::List(vec![root]);
                        place_value(&mut single_one, components, rules, path, copies, unnamed)?;
                        for mut rule in copy_rules {
                            place_value(&mut rule, components, rules, path, copies, unnamed)?;
                            rules.push(rule);
                        }
                        path.pop();
                        if let Value::List(placed_ones) = single_one {
                            placed_list.extend(placed_ones);
                        }
                    }
                    mut other => {
                        place_value(&mut other, components, rules, path, copies, unnamed)?;
                        placed_list.push(other);
                    }
                }
            }
            *elements = placed_list;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Écrit une copie d'un composant, d'après son appel : `ArticleCard.promo(name: Night, title: "Night", …)`.
fn place_copy(component: &Component, call: &Block, unnamed: &mut Vec<String>) -> Result<(Value, Vec<Value>), Error> {
    let name = &component.name;
    let expected = component.params.iter().map(|p| format!("{p}: …")).collect::<Vec<_>>().join(", ");
    let example = if expected.is_empty() { format!("{name}()") } else { format!("{name}({expected})") };
    let mut copy_name = None;
    let mut given_values: Vec<(&str, &Value)> = Vec::new();
    let mut branches: Vec<(String, Value)> = Vec::new();
    let mut content: Option<&Vec<Value>> = None;
    for argument in &call.arguments {
        let Some(key) = argument.name.as_deref() else {
            return Err(Error { message: format!("chaque paramètre de « {name} » est nommé : {example}"), pos: argument.pos });
        };
        match (key, &argument.value) {
            ("name", Value::Name(n)) if n.starts_with(|c: char| c.is_ascii_uppercase()) && n.chars().all(|c| c.is_ascii_alphanumeric()) => copy_name = Some(n.clone()),
            ("name", _) => return Err(Error { message: format!("le nom d'une copie s'écrit comme un bloc, avec une majuscule : {name}(name: Sunrise, …)"), pos: argument.pos }),
            // `children: [ … ]` : le contenu que la page met dans l'emplacement (ADR-058).
            ("children", Value::List(l)) if component.slot => {
                if content.is_some() {
                    return Err(Error { message: "« children » est donné deux fois".into(), pos: argument.pos });
                }
                content = Some(l);
            }
            ("children", _) if component.slot => return Err(Error { message: format!("« children » attend une liste de blocs entre crochets : {name}(children: [ P(\"…\") ])"), pos: argument.pos }),
            ("child", _) if component.slot => return Err(Error { message: format!("écris « children » : un seul mot pour le contenu, au pluriel même pour un bloc, {name}(children: [ P(\"…\") ])"), pos: argument.pos }),
            ("children" | "child", _) => return Err(Error { message: format!("« {name} » n'a pas d'emplacement pour du contenu : pour en avoir un, déclare params: [{}children] et pose le mot children dans son contenu", component.params.iter().map(|p| format!("{p}, ")).collect::<String>()), pos: argument.pos }),
            // `onAdd: cart.add(1)` : la page branche un signal émis par le composant.
            (key, v) if component.emitted.iter().any(|e| wiring(e) == key) => {
                let good = match v {
                    Value::Block(_) | Value::Name(_) => true,
                    Value::List(l) => !l.is_empty() && l.iter().all(|e| matches!(e, Value::Block(_) | Value::Name(_))),
                    _ => false,
                };
                if !good {
                    return Err(Error { message: format!("« {key} » attend une demande ou une liste de demandes : {key}: cart.add(1)"), pos: argument.pos });
                }
                let signal = component.emitted.iter().find(|e| wiring(e) == key).cloned().unwrap_or_default();
                branches.push((signal, v.clone()));
            }
            (p, v) if component.params.iter().any(|known| known == p) => {
                if given_values.iter().any(|(known, _)| *known == p) {
                    return Err(Error { message: format!("le paramètre « {p} » est donné deux fois"), pos: argument.pos });
                }
                if !matches!(v, Value::Text(_) | Value::Integer(_) | Value::Number { .. } | Value::Name(_) | Value::Bool(_)) {
                    return Err(Error { message: format!("le paramètre « {p} » attend un texte, un nombre ou un nom, pas un bloc ni une liste"), pos: argument.pos });
                }
                given_values.push((p, v));
            }
            (other, _) => {
                let mut known_ones = component.params.clone();
                known_ones.extend(component.emitted.iter().map(|e| wiring(e)));
                let suggestion = closest(other, &known_ones);
                let message = match suggestion {
                    Some(good) => format!("« {name} » n'a pas de paramètre « {other} » : écris « {good} »"),
                    None if component.params.is_empty() => format!("« {name} » n'a pas de paramètres : {name}()"),
                    None => format!("« {name} » n'a pas de paramètre « {other} » ; paramètres : {}", component.params.join(", ")),
                };
                return Err(Error { message, pos: argument.pos });
            }
        }
    }
    // Un paramètre oublié prend sa valeur par défaut, s'il en a une.
    for (p, default_value) in &component.defaults {
        if !given_values.iter().any(|(d, _)| d == p) {
            given_values.push((p.as_str(), default_value));
        }
    }
    if let Some(missing) = component.params.iter().find(|p| !given_values.iter().any(|(d, _)| d == p)) {
        return Err(Error { message: format!("« {name} » attend le paramètre « {missing} » : {example}"), pos: call.pos });
    }
    // Le contenu : un seul bloc racine, comme le widget que rend Flutter.
    let [Value::Block(_)] = component.children.as_slice() else {
        return Err(Error { message: format!("« {name} » se pose comme un bloc : son contenu a un seul bloc racine ; range ses blocs dans Column(children: [ … ])"), pos: component.pos });
    };
    let mut names = Vec::new();
    for value in component.children.iter().chain(component.rules.iter()) {
        given_names(value, &mut names);
    }
    if !names.is_empty() && copy_name.is_none() {
        // Sans nom, une copie garde les noms de ses blocs : une seule copie peut le faire.
        if unnamed.contains(name) {
            return Err(Error { message: format!("« {name} » nomme ses blocs ({}) : donne un nom à chaque copie, {name}(name: Sunrise, …)", names.join(", ")), pos: call.pos });
        }
        unnamed.push(name.clone());
    }
    let copy = Copy { given_values: &given_values, names: &names, suffix: copy_name.as_deref() };
    let mut root = component.children[0].clone();
    replace(&mut root, &copy)?;
    // Après le remplacement : le contenu donné n'est ni renommé ni touché par les paramètres.
    if component.slot {
        fill_slot(&mut root, content.map_or(&[][..], |c| c.as_slice()));
    }
    if let Value::Block(block) = &mut root {
        // La marque du composant d'abord, puis les noms de style écrits à l'appel.
        let mut styles = vec![name.clone()];
        styles.extend(call.styles.iter().cloned());
        styles.extend(std::mem::take(&mut block.styles));
        block.styles = styles;
    }
    let mut rules = Vec::new();
    for rule in &component.rules {
        let mut r = rule.clone();
        replace(&mut r, &copy)?;
        if let Some(r) = wire(r, &branches) {
            rules.push(r);
        }
    }
    Ok((root, rules))
}

/// Le paramètre le plus proche d'un nom mal écrit : une ou deux lettres de différence.
fn closest<'a>(name: &str, known_ones: &'a [String]) -> Option<&'a str> {
    fn distance(a: &str, b: &str) -> usize {
        let (a, b): (Vec<char>, Vec<char>) = (a.to_lowercase().chars().collect(), b.to_lowercase().chars().collect());
        let mut line: Vec<usize> = (0..=b.len()).collect();
        for i in 1..=a.len() {
            let mut previous = line[0];
            line[0] = i;
            for j in 1..=b.len() {
                let old = line[j];
                line[j] = (line[j] + 1).min(line[j - 1] + 1).min(previous + usize::from(a[i - 1] != b[j - 1]));
                previous = old;
            }
        }
        line[b.len()]
    }
    known_ones.iter().map(|c| (distance(name, c), c)).filter(|(d, _)| *d <= 2).min_by_key(|(d, _)| *d).map(|(_, c)| c.as_str())
}

struct Copy<'a> {
    given_values: &'a [(&'a str, &'a Value)],
    names: &'a [String],
    suffix: Option<&'a str>,
}

impl Copy<'_> {
    fn param(&self, name: &str) -> Option<&Value> {
        self.given_values.iter().find(|(p, _)| *p == name).map(|(_, v)| *v)
    }
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

/// Remplace, dans la copie, les paramètres par ce qui a été donné, et renomme les blocs nommés.
fn replace(value: &mut Value, copy: &Copy) -> Result<(), Error> {
    match value {
        Value::Text(text) => *text = replace_in_text(text, copy),
        Value::Name(name) => {
            let (start, suite) = name.split_once('.').map_or((name.as_str(), None), |(a, b)| (a, Some(b)));
            if let Some(given) = copy.param(start) {
                match (given, suite) {
                    (_, None) => *value = given.clone(),
                    // `count.enter` : une capacité demandée à la valeur donnée.
                    (Value::Name(n), Some(s)) => *name = format!("{n}.{s}"),
                    _ => {}
                }
            } else if copy.names.iter().any(|n| n == start) {
                if let Some(suffix) = copy.suffix {
                    let new_one = format!("{start}{suffix}");
                    *name = match suite {
                        Some(s) => format!("{new_one}.{s}"),
                        None => new_one,
                    };
                }
            }
        }
        Value::List(elements) => {
            for e in elements {
                replace(e, copy)?;
            }
        }
        Value::Block(block) => {
            // Une demande faite à une valeur donnée en paramètre : `count.add(1)`.
            if let Some((start, verb)) = block.name.split_once('.') {
                if start.starts_with(|c: char| c.is_ascii_lowercase()) {
                    if let Some(given) = copy.param(start) {
                        match given {
                            Value::Name(n) => block.name = format!("{n}.{verb}"),
                            _ => return Err(Error { message: format!("« {start}.{verb}(…) » : le paramètre « {start} » doit recevoir le nom d'une valeur de la page pour qu'on puisse la changer, comme {start}: cart"), pos: block.pos }),
                        }
                    }
                }
            }
            for argument in &mut block.arguments {
                replace(&mut argument.value, copy)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// `{title}` devient ce qui a été donné ; donné par un nom (`count: sunrise`, `title: item.title`),
/// il reste à montrer : `{sunrise}`, `{item.title}`.
fn replace_in_text(text: &str, copy: &Copy) -> String {
    if !text.contains('{') || copy.given_values.is_empty() {
        return text.to_string();
    }
    let mut output = String::with_capacity(text.len());
    let mut remainder = text;
    while let Some(start) = remainder.find('{') {
        output.push_str(&remainder[..start]);
        let after = &remainder[start + 1..];
        let Some(end) = after.find('}') else {
            output.push_str(&remainder[start..]);
            remainder = "";
            break;
        };
        let inside = &after[..end];
        let (name, format) = inside.split_once(':').map_or((inside, None), |(n, f)| (n, Some(f)));
        match (copy.param(name), format) {
            (Some(Value::Name(n)), Some(f)) => output.push_str(&format!("{{{n}:{f}}}")),
            (Some(Value::Name(n)), None) => output.push_str(&format!("{{{n}}}")),
            (Some(Value::Integer(n)), Some(f)) if crate::format::is_format(f) && f != "name" => output.push_str(&crate::format::format_value(name, *n, f, &crate::format::language())),
            (Some(Value::Text(t)), None) => output.push_str(t),
            (Some(Value::Integer(n)), None) => output.push_str(&n.to_string()),
            (Some(Value::Number { value, unit, .. }), None) => output.push_str(&format!("{value}{}", unit.as_deref().unwrap_or(""))),
            (Some(Value::Bool(b)), None) => output.push_str(if *b { "true" } else { "false" }),
            _ => output.push_str(&remainder[start..start + 1 + end + 1]),
        }
        remainder = &after[end + 1..];
    }
    output.push_str(remainder);
    output
}

#[cfg(test)]
mod tests {
    use crate::holo::read;

    const SHOP: &str = r#"Page(
  state: State(cart: 0, sunrise: 0),
  components: [
    Component(
      name: ArticleCard,
      params: [title, price, qty],
      children: [ Column.card(children: [ H2("{title}"), Text("{price:cents} euros, {qty} in the cart"), Button(name: Add, text: "Add") ]) ],
      rules: [ On(Add.tap, effect: [cart.add(price), qty.add(1)]) ],
    ),
  ],
  children: [
    H1("Shop"),
    ArticleCard(name: Sunrise, title: "Sunrise", price: 12000, qty: sunrise),
    ArticleCard.promo(name: Night, title: "Night", price: 6000, qty: cart),
  ],
)
ArticleCard { --accent: #E9B44C; border: 1px solid --accent; }
.card { padding: 8px; }
.promo { --accent: crimson; }
"#;

    #[test]
    fn a_component_is_placed_like_a_block_and_restyled_by_css() {
        let program = read(SHOP).unwrap();
        let html = crate::flat::page_html(&program, "").unwrap();
        assert!(html.contains("<div class=\"holo-Column holo-c-ArticleCard holo-s-card\""), "{html}");
        assert!(html.contains("<div class=\"holo-Column holo-c-ArticleCard holo-s-promo holo-s-card\""), "{html}");
        assert!(html.contains("<h2 class=\"holo-H2\">Sunrise</h2>"), "{html}");
        assert!(html.contains("data-name=\"AddSunrise\"") && html.contains("data-name=\"AddNight\""), "{html}");
        assert!(html.contains(".holo-c-ArticleCard{--accent:#E9B44C;border:1px solid var(--accent);}"), "{html}");
        assert!(html.contains(".holo-s-promo{--accent:crimson;}"), "{html}");
        crate::check_page(SHOP).unwrap();
        // Les règles du composant, une fois par copie, avec les paramètres remplacés.
        assert_eq!(crate::arbitrate(SHOP, "cart=0;sunrise=0", "AddSunrise.tap"), "cart=12000;sunrise=1");
        assert_eq!(crate::arbitrate(SHOP, "cart=0;sunrise=0", "AddNight.tap"), "cart=6001;sunrise=0");
    }

    #[test]
    fn a_component_in_a_repeat_and_in_an_imported_file() {
        let common = "Component(name: Card, params: [title], children: [ Column(children: [ H2(\"{title}\"), Button(name: Add, text: \"Add\") ]) ], rules: [ On(Add.tap, effect: item.add(1)) ])\nCard { color: navy; }";
        let page = "import \"commun.holo\"\nPage(state: State(a: 0, b: 0), children: [ H1(\"x\"), Repeat(items: [ Item(key: a, title: \"A\"), Item(key: b, title: \"B\") ], children: [ Card(title: item.title) ]) ])";
        let source = format!("{page}{}commun.holo{}{common}", crate::holo::NEXT_FILE, crate::holo::NAME_SEPARATOR);
        let program = read(&source).unwrap();
        let html = crate::flat::page_html(&program, "").unwrap();
        assert!(html.contains("<h2 class=\"holo-H2\">A</h2>") && html.contains("data-name=\"AddB\""), "{html}");
        crate::check_page(&source).unwrap();
    }

    #[test]
    fn the_guide_example_in_a_repeat() {
        let source = r#"Page(
  state: State(cart: 0, a: 0, b: 0),
  components: [ Component(name: ArticleCard, params: [title, price, qty], children: [ Column(children: [ H2("{title}"), Text("{price:cents} euros, {qty}"), Button(name: Add, text: "Add") ]) ], rules: [ On(Add.tap, effect: [qty.add(1), cart.add(price)]) ]) ],
  children: [ H1("Shop"), Repeat(items: [ Item(key: a, title: "A", price: 1250), Item(key: b, title: "B", price: 300) ], children: [ ArticleCard(title: item.title, price: item.price, qty: item) ]) ],
)"#;
        crate::check_page(source).unwrap();
        assert_eq!(crate::arbitrate(source, "cart=0;a=0;b=0", "AddB.tap"), "cart=300;a=0;b=1");
        let html = crate::flat::page_html(&read(source).unwrap(), "").unwrap();
        assert!(html.contains("<h2 class=\"holo-H2\">A</h2>") && html.contains("12,50 euros"), "{html}");
    }

    #[test]
    fn default_values_and_emitted_signals() {
        let source = r#"Page(
  state: State(cart: 0, likes: 0),
  components: [
    Component(
      name: ArticleCard,
      params: [title, price: 0, image: "placeholder.svg"],
      emits: [add, like],
      children: [ Column(children: [ Image(source: image, alt: title), H2("{title}"), Text("{price} euros"), Button(name: Add, text: "Add"), Button(name: Like, text: "♥") ]) ],
      rules: [ On(Add.tap, emit: add), On(Like.tap, emit: like) ],
    ),
  ],
  children: [
    H1("Shop"),
    ArticleCard(name: Sunrise, title: "Sunrise", price: 120, image: "sunrise.png", onAdd: cart.add(120), onLike: likes.add(1)),
    ArticleCard(name: Gift, title: "Gift card", onAdd: [cart.add(10), likes.add(1)]),
  ],
)"#;
        crate::check_page(source).unwrap();
        let html = crate::flat::page_html(&read(source).unwrap(), "").unwrap();
        // Les valeurs par défaut : l'image et le prix oubliés par la seconde copie.
        assert!(html.contains("src=\"placeholder.svg\" alt=\"Gift card\"") && html.contains("0 euros"), "{html}");
        // Les signaux émis, branchés par la page.
        assert_eq!(crate::arbitrate(source, "cart=0;likes=0", "AddSunrise.tap"), "cart=120;likes=0");
        assert_eq!(crate::arbitrate(source, "cart=0;likes=0", "LikeSunrise.tap"), "cart=0;likes=1");
        assert_eq!(crate::arbitrate(source, "cart=0;likes=0", "AddGift.tap"), "cart=10;likes=1");
        // Un signal que la page ne branche pas ne fait rien.
        assert_eq!(crate::arbitrate(source, "cart=0;likes=0", "LikeGift.tap"), "cart=0;likes=0");
        for (source, message) in [
            ("Page(components: [ Component(name: Card, children: [ Button(name: Go, text: \"go\") ], rules: [ On(Go.tap, emit: go) ]) ], children: [ Card() ])", "déclare ce signal"),
            ("Page(components: [ Component(name: Card, emits: [go], children: [ Button(name: Go, text: \"go\") ], rules: [ On(Go.tap, emit: go) ]) ], children: [ Card(onGo: 3) ])", "attend une demande"),
            ("Page(components: [ Component(name: Card, emits: [go], children: [ P(\"x\") ]) ], children: [ Card(onGoo: x.add(1)) ])", "écris « onGo »"),
            ("Page(state: State(l: [ x: 1 ]), children: [])", "seuls les paramètres d'un composant"),
            ("Page(components: [ Part(name: Card, children: [ P(\"x\") ]) ], children: [])", "écris « Component »"),
            ("Page(parts: [ Component(name: Card, children: [ P(\"x\") ]) ], children: [])", "écris « components »"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn a_slot_for_content() {
        let source = r#"Page(
  state: State(open: 0),
  components: [
    Component(
      name: Panel,
      params: [title, children],
      children: [ Column(children: [ H2("{title}"), children, Button(name: Close, text: "Close") ]) ],
      rules: [ On(Close.tap, effect: open.set(0)) ],
    ),
  ],
  children: [
    H1("Panels"),
    Panel(name: Info, title: "Info", children: [ P("First"), Button(name: More, text: "More") ]),
    Panel(name: Outer, title: "Outer", children: [ Panel(name: Inner, title: "Inner", children: [ P("Deep") ]) ]),
    Panel(name: Empty, title: "Empty"),
  ],
  rules: [ On(More.tap, effect: open.set(1)) ],
)"#;
        crate::check_page(source).unwrap();
        let html = crate::flat::page_html(&read(source).unwrap(), "").unwrap();
        // Le contenu est posé entre le titre et le bouton ; ses noms ne sont pas renommés.
        let (title, first, button) = (html.find(">Info<").unwrap(), html.find(">First<").unwrap(), html.find("data-name=\"CloseInfo\"").unwrap());
        assert!(title < first && first < button, "{html}");
        assert!(html.contains("data-name=\"More\""), "{html}");
        // Une carte dans une carte, et un emplacement laissé vide.
        assert!(html.contains(">Deep<") && html.contains("data-name=\"CloseInner\"") && html.contains(">Empty<"), "{html}");
        assert_eq!(crate::arbitrate(source, "open=0", "More.tap"), "open=1");
        assert_eq!(crate::arbitrate(source, "open=1", "CloseInner.tap"), "open=0");
        let card = |params: &str, content: &str, call: &str| format!("Page(components: [ Component(name: Card, params: [{params}], children: [ Column(children: [ {content} ]) ]) ], children: [ {call} ])");
        for (source, message) in [
            (card("children", "P(\"x\")", "Card()"), "sans le poser"),
            (card("", "children", "Card()"), "sans le déclarer"),
            (card("", "P(\"x\")", "Card(children: [ P(\"y\") ])"), "pas d'emplacement"),
            (card("children", "children", "Card(child: P(\"y\"))"), "écris « children »"),
            (card("child", "children", "Card()"), "au pluriel"),
            (card("children", "children", "Card(children: P(\"y\"))"), "liste de blocs"),
            (card("children: []", "children", "Card()"), "pas de valeur par défaut"),
        ] {
            let error = crate::check_page(&source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn what_is_refused() {
        let part = |remainder: &str| format!("Page(components: [ Component(name: Card, params: [title], children: [ H3(\"{{title}}\") ]) ], children: [ {remainder} ])");
        for (source, message) in [
            (part("Card()"), "attend le paramètre « title »"),
            (part("Card(titel: \"a\")"), "écris « title »"),
            (part("Card(title: \"a\", title: \"b\")"), "donné deux fois"),
            (part("Card(\"a\")"), "est nommé"),
            ("Page(components: [ Component(name: Card, children: [ Card() ]) ], children: [ Card() ])".into(), "se pose lui-même"),
            ("Page(components: [ Component(name: Text, children: [ P(\"x\") ]) ], children: [ Text() ])".into(), "déjà un bloc du langage"),
            ("Page(components: [ Component(name: Card, params: [add], children: [ P(\"x\") ]) ], children: [ ])".into(), "mot du langage"),
            ("Page(state: State(title: \"\"), components: [ Component(name: Card, params: [title], children: [ P(\"x\") ]) ], children: [ Card(title: \"a\") ])".into(), "nom d'une valeur de la page"),
            ("Page(components: [ Component(name: Card, children: [ P(\"a\"), P(\"b\") ]) ], children: [ Card() ])".into(), "un seul bloc racine"),
            ("Page(components: [ Component(name: Card, children: [ Button(name: Go, text: \"go\") ]) ], children: [ Card(), Card() ])".into(), "donne un nom à chaque copie"),
            ("Page(components: [ Component(name: Card, params: [n], children: [ Button(name: Go, text: \"go\") ], rules: [ On(Go.tap, effect: n.add(1)) ]) ], children: [ Card(n: 3) ])".into(), "doit recevoir le nom d'une valeur"),
            ("Page(children: [ P.Big(\"x\") ])".into(), "en minuscules"),
        ] {
            let error = crate::check_page(&source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }
}
