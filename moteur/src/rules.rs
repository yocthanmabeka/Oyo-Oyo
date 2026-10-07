//! Les noms, les règles et les budgets d'un fichier (ADR-005, ADR-015).
//!
//! Un bloc ne contient aucun code : une règle `On(Open.tap, effect: Workshop.enter)` relie
//! un signal à une capacité demandée. Ce fichier vérifie ces règles avant toute exécution,
//! puis répond à la seule question que pose l'affichage : « ce signal, que demande-t-il ? ».

use crate::holo::{Block, Error, Program, Value};

/// Ce qu'un bloc sait émettre (signaux) et ce qu'on peut lui demander (capacités).
fn signals(block: &str) -> &'static [&'static str] {
    match block {
        "Button" | "Point" | "Shape" => &["tap", "hover", "hoverEnd"],
        // Un formulaire dit si son envoi est arrivé, ou non (ADR-042).
        "Form" => &["sent", "failed", "hover", "hoverEnd"],
        // Un module enfermé dit s'il a rendu son nombre, ou s'il a été arrêté (ADR-045).
        "Module" => &["done", "failed"],
        // Les données de la page disent si elles sont arrivées, ou non (ADR-064).
        "Data" => &["done", "failed"],
        // Tout bloc qui se voit peut être survolé (ADR-039) : la souris arrive dessus, le
        // clavier s'y pose, ou le doigt le touche sur un téléphone.
        other if crate::blocks::BLOCKS.contains(&other) && !INVISIBLE.contains(&other) => &["hover", "hoverEnd"],
        _ => &[],
    }
}

/// Les blocs qui ne se voient pas, ou qui ne sont pas une boîte à l'écran : on ne les survole pas.
const INVISIBLE: &[&str] = &["Page", "World", "Main", "On", "Every", "When", "After", "State", "Prices", "Data", "Zoom", "Points", "Relief", "Portals", "Component", "Use", "Sound", "Scene", "Enter", "Loop", "If", "Repeat", "Item", "Font", "Module"];

/// Le survol : la souris arrive sur le bloc (`hover`), puis le quitte (`hoverEnd`).
pub const HOVER: &[&str] = &["hover", "hoverEnd"];

fn capabilities(block: &str) -> &'static [&'static str] {
    match block {
        "Point" => &["enter", "leave"],
        // Jouer un son : On(Star.tap, effect: Ding.play).
        "Sound" => &["play", "stop"],
        // Ouvrir le carrefour : les portails vers les mondes voisins.
        "Page" => &["portals"],
        // Une fenêtre par-dessus la page, et un formulaire qu'on envoie (ADR-042).
        "Dialog" => &["open", "close"],
        "Form" => &["send"],
        "Module" => &["run"],
        // Relire les données : On(Retry.tap, effect: Stock.refresh) (ADR-064).
        "Data" => &["refresh"],
        _ => &[],
    }
}

/// Parcourt tous les blocs du fichier, dans l'ordre où ils sont écrits.
pub fn for_each_block<'a>(block: &'a Block, f: &mut dyn FnMut(&'a Block) -> Result<(), Error>) -> Result<(), Error> {
    fn visit<'a>(value: &'a Value, f: &mut dyn FnMut(&'a Block) -> Result<(), Error>) -> Result<(), Error> {
        match value {
            Value::Block(block) => for_each_block(block, f),
            Value::List(elements) => elements.iter().try_for_each(|e| visit(e, f)),
            _ => Ok(()),
        }
    }
    f(block)?;
    block.arguments.iter().try_for_each(|a| visit(&a.value, f))
}

/// Le nom donné à un bloc par `name: Open`.
pub fn name_of(block: &Block) -> Option<&str> {
    match &block.argument("name")?.value {
        Value::Name(name) => Some(name),
        _ => None,
    }
}

/// Le bloc qui porte ce nom.
pub fn named_block<'a>(program: &'a Program, name: &str) -> Option<&'a Block> {
    let mut found = None;
    let _ = for_each_block(&program.root, &mut |block| {
        if found.is_none() && name_of(block) == Some(name) {
            found = Some(block);
        }
        Ok(())
    });
    found
}

/// Le site que désigne un chemin de points : vide, c'est la page du fichier ; `Shop/Secret`,
/// c'est le monde du point `Secret`, lui-même dans le monde du point `Shop`.
pub fn site_of<'a>(program: &'a Program, path: &str) -> Result<&'a Block, Error> {
    let mut site = &program.root;
    for name in path.split('/').filter(|n| !n.is_empty()) {
        let mut found = None;
        let _ = for_each_block(site, &mut |block| {
            if found.is_none() && block.name == "Point" && name_of(block) == Some(name) {
                found = Some(block);
            }
            Ok(())
        });
        site = match found.and_then(|point| point.argument("inside")).map(|a| &a.value) {
            Some(Value::Block(world)) if world.name == "World" => world,
            _ => return Err(Error { message: format!("aucun point nommé « {name} » ne contient un monde, à cet endroit du fichier"), pos: site.pos }),
        };
    }
    Ok(site)
}

/// Vérifie les noms (aucun en double), les règles (signaux et capacités connus) et les
/// budgets (le contenu d'un point ne pèse pas plus que ce qu'il déclare).
pub fn check_rules(program: &Program) -> Result<(), Error> {
    let mut names: Vec<(&str, &str)> = Vec::new();
    for_each_block(&program.root, &mut |block| {
        if let Some(name) = name_of(block) {
            if names.iter().any(|(known, _)| *known == name) {
                let pos = block.argument("name").map_or(block.pos, |a| a.pos);
                return Err(Error { message: format!("le nom « {name} » est déjà porté par un autre bloc : deux blocs ne partagent pas un nom"), pos });
            }
            names.push((name, &block.name));
        }
        Ok(())
    })?;
    check_landmarks(&program.root)?;
    // Un lien vers un endroit de la page (ADR-042) vise un bloc qui existe : sur le web, une
    // ancre mal écrite ne mène nulle part, sans rien dire.
    for_each_block(&program.root, &mut |block| {
        if let (true, Some(crate::holo::Argument { value: Value::Text(address), pos, .. })) = (block.name == "A", block.argument("to")) {
            if let Some(target) = address.strip_prefix('#').filter(|c| !c.is_empty() && !c.contains('/') && !c.starts_with('@')) {
                if !names.iter().any(|(known, _)| *known == target) {
                    return Err(Error { message: format!("« to: \"#{target}\" » : aucun bloc ne s'appelle « {target} » ; un lien vers un endroit de la page vise le nom d'un bloc, comme H2(\"Horaires\", name: {target})"), pos: *pos });
                }
            }
        }
        Ok(())
    })?;
    let state = crate::state::initial(program)?;
    // Une répétition dynamique montre une liste qui existe (ADR-044).
    for (repeat, list) in crate::lists::repeats(program) {
        if !crate::lists::is_list(program, &list) {
            return Err(Error { message: format!("« Repeat(over: {list}) » : aucune liste ne s'appelle « {list} » ; déclare-la, state: State({list}: [])"), pos: repeat.pos });
        }
    }
    for_each_block(&program.root, &mut |block| {
        if block.name == "On" {
            check_rule(block, &names, &state, program)?;
        }
        // Des règles sous condition : If(lives, over: 0, rules: [ … ]). Seules les règles de temps
        // et les règles qui guettent s'y rangent : une règle « On » répond à un geste, et c'est
        // le bouton qu'on cache, par un If dans la page.
        if block.name == "If" {
            if let Some(Value::List(rules)) = block.argument("rules").map(|a| &a.value) {
                for rule in rules {
                    if !matches!(rule, Value::Block(b) if b.name == "Every" || b.name == "When" || b.name == "After") {
                        return Err(Error { message: "sous une condition, on range des règles de temps et des règles qui guettent : If(lives, over: 0, rules: [ Every(…), After(…), When(…) ])".into(), pos: block.pos });
                    }
                }
            }
        }
        // Une règle qui guette une valeur, ou la rencontre de deux blocs (ADR-028).
        if block.name == "When" {
            if block.argument("meets").is_some() {
                let (a, b, _) = crate::state::meets(block)?;
                for name in [a, b] {
                    let placed = named_block(program, name).is_some_and(|placed| placed.argument("x").is_some() && placed.argument("y").is_some());
                    if !placed {
                        return Err(Error { message: format!("une rencontre : « {name} » n'est pas posé sur un plateau ; il lui faut un nom, et « x » et « y » dans un « Board »"), pos: block.pos });
                    }
                }
            } else {
                crate::state::condition(block)?;
            }
            check_effects(block, &names, &state, false, program)?;
        }
        // Une règle de temps : un rythme, et une ou plusieurs demandes faites à l'arbitre (ADR-026).
        // Une seule fois, plus tard : After(3s, effect: …) (ADR-039).
        if block.name == "Every" || block.name == "After" {
            crate::state::rhythm(block)?;
            check_effects(block, &names, &state, false, program)?;
        }
        if block.name == "Point" {
            check_budget(block)?;
        }
        Ok(())
    })
}

/// Un point planté dans un pixel se repère par rapport à un bloc du même site : la page, ou
/// le monde, où il est planté. Un bloc rangé dans le monde d'un autre point n'est pas à
/// l'écran au même moment : l'affichage ne le trouverait pas (revue Codex, B-06).
fn check_landmarks(site: &Block) -> Result<(), Error> {
    fn visit<'a>(value: &'a Value, names: &mut Vec<&'a str>, worlds: &mut Vec<&'a Block>) {
        match value {
            Value::List(elements) => elements.iter().for_each(|e| visit(e, names, worlds)),
            Value::Block(block) => {
                if let Some(name) = name_of(block) {
                    names.push(name);
                }
                for argument in &block.arguments {
                    match &argument.value {
                        // Le monde d'un point est un autre site : on n'y descend pas.
                        Value::Block(world) if block.name == "Point" && argument.name.as_deref() == Some("inside") => worlds.push(world),
                        other => visit(other, names, worlds),
                    }
                }
            }
            _ => {}
        }
    }
    let (mut names, mut worlds) = (Vec::new(), Vec::new());
    site.arguments.iter().for_each(|a| visit(&a.value, &mut names, &mut worlds));
    if let Some(Value::List(planted_ones)) = site.argument("pixels").map(|a| &a.value) {
        for planted in planted_ones {
            let Value::Block(point) = planted else { continue };
            if let Some(argument) = point.argument("above") {
                let known = matches!(&argument.value, Value::Name(landmark) if names.contains(&landmark.as_str()));
                if !known {
                    return Err(Error { message: "« above » attend le nom d'un bloc de la page où le point est planté : above: Open".into(), pos: argument.pos });
                }
            }
        }
    }
    worlds.into_iter().try_for_each(check_landmarks)
}

/// `Open.tap` → (`Open`, `tap`).
fn name_and_word<'a>(block: &'a Block, value: Option<&'a Value>, what: &str) -> Result<(&'a str, &'a str), Error> {
    match value {
        Some(Value::Name(n)) => n.split_once('.').filter(|(a, b)| !a.is_empty() && !b.is_empty() && !b.contains('.')),
        _ => None,
    }
    .ok_or_else(|| Error { message: format!("une règle s'écrit « On(Open.tap, effect: Workshop.enter) » : {what} manque ou est mal écrit"), pos: block.pos })
}

fn check_rule(rule: &Block, names: &[(&str, &str)], state: &crate::state::State, program: &Program) -> Result<(), Error> {
    let signal = rule.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value);
    let (source, word) = name_and_word(rule, signal, "le signal")?;
    let type_of = |name: &str| names.iter().find(|(known, _)| *known == name).map(|(_, block)| *block);
    let unknown = |name: &str| Error { message: format!("aucun bloc ne s'appelle « {name} »"), pos: rule.pos };
    // « Key » est le clavier du visiteur : On(Key.left, effect: basket.sub(8)).
    if source == "Key" && type_of(source).is_none() {
        if !crate::state::KEYPRESSES.contains(&word) {
            let lower = word.to_lowercase();
            let message = if crate::state::KEYPRESSES.contains(&lower.as_str()) {
                format!("une touche s'écrit en minuscules : écris « {lower} »")
            } else if lower == "tab" {
                "Tab sert à passer d'un bouton à l'autre : une page ne la prend jamais".to_string()
            } else {
                format!("touche inconnue « {word} » : le clavier donne left, right, up, down, space, enter, escape, les lettres de a à z et les chiffres de digit0 à digit9")
            };
            return Err(Error { message, pos: rule.pos });
        }
    } else {
        let type_source = type_of(source).ok_or_else(|| unknown(source))?;
        if !signals(type_source).contains(&word) {
            return Err(Error { message: format!("signal inconnu « {word} » : un « {type_source} » émet {}", list_all(signals(type_source))), pos: rule.pos });
        }
    }
    // Un survol ne fait que changer des valeurs ou jouer un son : on n'emmène pas le visiteur
    // ailleurs parce que sa souris est passée par là.
    check_effects(rule, names, state, !HOVER.contains(&word), program)
}

/// Les effets d'une règle : un seul, ou plusieurs entre crochets.
fn effects_of(rule: &Block) -> Vec<&Value> {
    match rule.argument("effect").map(|a| &a.value) {
        Some(Value::List(elements)) => elements.iter().collect(),
        Some(effect) => vec![effect],
        None => Vec::new(),
    }
}

/// Vérifie les effets d'une règle. Un effet est une demande faite à l'arbitre (`cart.add(1)`,
/// ADR-023) ou, pour une règle `On`, une capacité demandée à un bloc (`Workshop.enter`).
fn check_effects(rule: &Block, names: &[(&str, &str)], state: &crate::state::State, allowed_capabilities: bool, program: &Program) -> Result<(), Error> {
    let type_of = |name: &str| names.iter().find(|(known, _)| *known == name).map(|(_, block)| *block);
    let expects_request = || Error { message: format!("« {} » attend une demande : {}(…, effect: score.add(1))", rule.name, rule.name), pos: rule.pos };
    let effects = effects_of(rule);
    if effects.is_empty() {
        return if allowed_capabilities { name_and_word(rule, None, "l'effet").map(|_| ()) } else { Err(expects_request()) };
    }
    for effect in effects {
        match effect {
            // Une demande faite à une liste ou à un texte (ADR-044).
            Value::Block(request) if crate::state::is_requested(request) && crate::lists::concerns(request, program) => {
                let line = crate::lists::line_rules(program).into_iter().find(|(b, _)| std::ptr::eq(*b, rule)).map(|(_, l)| l);
                crate::lists::check_request(request, program, rule, line.as_deref())?;
            }
            Value::Block(request) if crate::state::is_requested(request) => {
                crate::state::request(request, state)?;
            }
            Value::Name(_) if allowed_capabilities => {
                let (target, capability) = name_and_word(rule, Some(effect), "l'effet")?;
                if state.iter().any(|(value, _)| value == target) {
                    return Err(Error { message: format!("« {target}.{capability} » s'écrit avec sa quantité, entre parenthèses : {target}.{capability}(1)"), pos: rule.pos });
                }
                let target_type = type_of(target).ok_or_else(|| Error { message: format!("aucun bloc ne s'appelle « {target} »"), pos: rule.pos })?;
                if !capabilities(target_type).contains(&capability) {
                    return Err(Error { message: format!("capacité inconnue « {capability} » : un « {target_type} » offre {}", list_all(capabilities(target_type))), pos: rule.pos });
                }
            }
            _ if allowed_capabilities => {
                name_and_word(rule, Some(effect), "l'effet")?;
            }
            // Une règle de temps ou une règle qui guette peut faire entendre un son ; elle ne
            // peut pas emmener le visiteur ailleurs sans qu'il ait rien touché.
            Value::Name(_) => {
                let (target, capability) = name_and_word(rule, Some(effect), "l'effet")?;
                if (type_of(target) != Some("Sound") || !matches!(capability, "play" | "stop")) && rule.name == "On" {
                    return Err(Error {
                        message: format!("un survol change des valeurs ou joue un son ; « {target}.{capability} » demande que le visiteur touche : On(…tap, effect: {target}.{capability})"),
                        pos: rule.pos,
                    });
                }
                if type_of(target) != Some("Sound") || !matches!(capability, "play" | "stop") {
                    return Err(Error {
                        message: format!("« {} » : en dehors d'une demande, seule la lecture d'un son est permise ici (Ding.play) ; « {target}.{capability} » demande un geste du visiteur, dans une règle « On »", rule.name),
                        pos: rule.pos,
                    });
                }
            }
            _ => return Err(expects_request()),
        }
    }
    Ok(())
}

fn list_all(words: &[&str]) -> String {
    if words.is_empty() {
        "rien".into()
    } else {
        words.join(", ")
    }
}

/// Un poids en octets. Les tailles sont décimales : 1 KB = 1 000 octets.
fn bytes(block: &Block, param: &str) -> Result<Option<f64>, Error> {
    let Some(argument) = block.argument(param) else { return Ok(None) };
    let factor = |unit: &str| match unit {
        "B" => Some(1.0),
        "KB" => Some(1e3),
        "MB" => Some(1e6),
        "GB" => Some(1e9),
        _ => None,
    };
    match &argument.value {
        Value::Number { value, unit: Some(unit) } if *value >= 0.0 && factor(unit).is_some() => Ok(factor(unit).map(|f| value * f)),
        _ => Err(Error { message: format!("le paramètre « {param} » attend une taille, comme « 500KB » (unités : B, KB, MB, GB)"), pos: argument.pos }),
    }
}

fn check_budget(point: &Block) -> Result<(), Error> {
    let budget = bytes(point, "budget")?;
    let mut weight = 0.0;
    if let Some(Value::Block(inner)) = point.argument("inside").map(|a| &a.value) {
        for_each_block(inner, &mut |block| {
            weight += bytes(block, "weight")?.unwrap_or(0.0);
            Ok(())
        })?;
    }
    match budget {
        Some(budget) if weight > budget => Err(Error {
            message: format!("budget dépassé : le contenu pèse {weight} octets, le budget est de {budget} octets (ADR-005)"),
            pos: point.argument("budget").map_or(point.pos, |a| a.pos),
        }),
        _ => Ok(()),
    }
}

/// Les blocs vers lesquels un lien de la page mène (`A(to: "#Hours")`), sauf les points, dont le
/// lien ouvre le monde. Ils reçoivent un `id` : le navigateur y descend tout seul.
pub fn anchors(program: &Program) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let _ = for_each_block(&program.root, &mut |block| {
        if let (true, Some(Value::Text(address))) = (block.name == "A", block.argument("to").map(|a| &a.value)) {
            if let Some(target) = address.strip_prefix('#').filter(|c| !c.is_empty() && !c.contains('/') && !c.starts_with('@')) {
                let point = named_block(program, target).is_some_and(|b| b.name == "Point");
                if !point && !names.iter().any(|n| n == target) {
                    names.push(target.to_string());
                }
            }
        }
        Ok(())
    });
    names
}

/// Les blocs qu'une règle écoute au survol : `On(Card.hover, …)` ou `On(Card.hoverEnd, …)`.
pub fn hovered_ones(program: &Program) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let _ = for_each_block(&program.root, &mut |block| {
        if block.name == "On" {
            if let Some(Value::Name(signal)) = block.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value) {
                if let Some((name, word)) = signal.split_once('.') {
                    if HOVER.contains(&word) && name != "Key" && !names.iter().any(|n| n == name) {
                        names.push(name.to_string());
                    }
                }
            }
        }
        Ok(())
    });
    names
}

/// Les effets demandés par un signal, dans l'ordre où les règles sont écrites.
/// `Open.tap` → [`Workshop.enter`]. Toucher un point demande d'y entrer, sans règle à écrire.
pub fn effects(program: &Program, signal: &str) -> Vec<String> {
    let signal = crate::lists::signal_and_line(signal).0;
    let mut effects = Vec::new();
    let _ = for_each_block(&program.root, &mut |block| {
        if block.name == "On" {
            let trigger = block.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value);
            if matches!(trigger, Some(Value::Name(s)) if s == signal) {
                for effect in effects_of(block) {
                    if let Value::Name(effect) = effect {
                        effects.push(effect.clone());
                    }
                }
            }
        }
        Ok(())
    });
    if let Some(name) = signal.strip_suffix(".tap") {
        if named_block(program, name).is_some_and(|b| b.name == "Point") && effects.is_empty() {
            effects.push(format!("{name}.enter"));
        }
    }
    effects
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::read;

    fn check(src: &str) -> Result<(), Error> {
        check_rules(&read(src)?)
    }

    #[test]
    fn refuses_what_the_suite_refuses_at_the_right_line() {
        let sample = [
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E02-unite-incompatible.holo"), 5, "attend une taille"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E03-budget-depasse.holo"), 5, "budget dépassé"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E06-capacite-inconnue.holo"), 8, "capacité inconnue « fly »"),
            (include_str!("../../experiments/conformite-v0.1/cas/refuses/E07-nom-en-double.holo"), 5, "déjà porté"),
        ];
        for (source, line, message) in sample {
            let error = check(source).unwrap_err();
            assert_eq!(error.pos.line, line, "{error}");
            assert!(error.message.contains(message), "{error}");
        }
    }

    #[test]
    fn accepts_valid_cases_and_the_shop() {
        for source in [
            include_str!("../../experiments/conformite-v0.1/cas/valides/03-entrer-dans-un-point.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/04-budget-respecte.holo"),
            include_str!("../../experiments/conformite-v0.1/cas/valides/06-boutique-avec-styles.holo"),
            include_str!("../../exemples/boutique-comparee/boutique.holo"),
        ] {
            check(source).unwrap();
        }
    }

    #[test]
    fn a_badly_written_rule_is_refused() {
        let page = |rule: &str| format!("Page(children: [ Button(name: Open, text: \"x\"), Point(name: A, seed: 1) ], rules: [ {rule} ])");
        assert!(check(&page("On(Open.tap, effect: A.enter)")).is_ok());
        assert!(check(&page("On(Open.swipe, effect: A.enter)")).unwrap_err().message.contains("signal inconnu « swipe »"));
        // Un survol change des valeurs ; il n'emmène pas ailleurs (ADR-039).
        assert!(check(&page("On(Open.hover, effect: A.enter)")).unwrap_err().message.contains("demande que le visiteur touche"));
        assert!(check(&page("On(Nobody.tap, effect: A.enter)")).unwrap_err().message.contains("aucun bloc ne s'appelle « Nobody »"));
        assert!(check(&page("On(Open.tap, effect: Open.enter)")).unwrap_err().message.contains("un « Button » offre rien"));
        assert!(check(&page("On(Open.tap)")).unwrap_err().message.contains("l'effet manque"));
        // Une page offre une capacité : ouvrir son carrefour.
        assert!(check("Page(name: Shop, children: [ Button(name: Map, text: \"x\") ], rules: [ On(Map.tap, effect: Shop.portals) ])").is_ok());
        assert!(check("Page(name: Shop, children: [ Button(name: Map, text: \"x\") ], rules: [ On(Map.tap, effect: Shop.enter) ])").unwrap_err().message.contains("un « Page » offre portals"));
        assert!(check("Page(children: [ Button(name: Open, text: \"x\") ], pixels: [ Point(name: S, seed: 1, above: Open) ])").is_ok());
        assert!(check("Page(pixels: [ Point(name: S, seed: 1, above: Nobody) ])").unwrap_err().message.contains("le nom d'un bloc de la page"));
        // Le repère doit être dans le même site : pas dans le monde d'un autre point.
        let elsewhere = "Page(children: [ Point(name: C, seed: 1, inside: World(children: [ Button(name: Inner, text: \"x\") ])) ], pixels: [ Point(name: S, seed: 2, above: Inner) ])";
        assert!(check(elsewhere).unwrap_err().message.contains("où le point est planté"));
        // Dans un monde, le repère se cherche dans ce monde.
        let inside = "Page(children: [ Point(name: C, seed: 1, inside: World(children: [ Button(name: Inner, text: \"x\") ], pixels: [ Point(name: S, seed: 2, above: Inner) ])) ])";
        assert!(check(inside).is_ok());
    }

    #[test]
    fn a_signal_gives_its_effects() {
        let p = read(include_str!("../../exemples/boutique-comparee/boutique.holo")).unwrap();
        assert_eq!(effects(&p, "Open.tap"), ["Workshop.enter"]);
        assert_eq!(effects(&p, "Back.tap"), ["Workshop.leave"]);
        // Toucher un point demande d'y entrer, sans règle à écrire.
        assert_eq!(effects(&p, "Workshop.tap"), ["Workshop.enter"]);
        assert!(effects(&p, "Open.hover").is_empty());
    }
}
