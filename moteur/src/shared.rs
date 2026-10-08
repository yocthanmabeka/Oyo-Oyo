//! Les valeurs partagées (ADR-079) : `shared: Shared(seats: 20, likes: 0)`, à côté de
//! `state: State(…)`. Le serveur de l'auteur les garde pour tout le monde, une fois pour chaque
//! adresse (`/concert/12` et `/concert/13` ont chacune leurs places) ; chaque visiteur les voit
//! changer en direct ; et c'est toujours le serveur qui arbitre un geste qui les change, avec le
//! même moteur que la page. Décision de Yocthan du 2026-10-08.
//!
//! À la lecture, les valeurs partagées rejoignent celles de la page (`State`), comme les valeurs
//! de l'adresse (ADR-078) : les textes et les conditions les lisent (`{seats}`,
//! `If(seats, over: 0, …)`), les règles les changent comme les autres
//! (`On(Book.tap, effect: seats.sub(1))`). Ce module refuse ce qui les changerait sans passer par
//! le serveur, et sépare ou fusionne les valeurs partagées et l'état d'un visiteur.

use crate::holo::{Argument, Block, Error, Program, Value};
use crate::rules::for_each_block;
use crate::state::{State, Texts};

/// Les valeurs qu'une page partage, au plus.
pub const SHARED_MAX: usize = 16;
/// Un texte partagé, au plus : un nom, une phrase courte. Le serveur le garde pour tous, et
/// chaque page ouverte le reçoit : il reste court.
pub const SHARED_TEXT_MAX: usize = 200;

const EXAMPLE: &str = "shared: Shared(seats: 20, likes: 0)";

/// Lit `shared: Shared(…)` sur la page, le vérifie, et range ses valeurs avec celles de `State` :
/// le reste du moteur les lit et les change comme les autres. Leurs noms sont gardés dans
/// `program.shared`.
pub fn inject(program: &mut Program) -> Result<(), Error> {
    // Ailleurs que sur une page (un monde, un composant), le réglage est refusé avec les autres.
    if program.root.name != "Page" {
        return Ok(());
    }
    let Some(index) = program.root.arguments.iter().position(|a| a.name.as_deref() == Some("shared")) else { return Ok(()) };
    let argument = program.root.arguments.remove(index);
    let block = match argument.value {
        Value::Block(block) if block.name == "Shared" => block,
        _ => return Err(Error { message: format!("« shared » attend un bloc « Shared(…) », sur la page : {EXAMPLE}"), pos: argument.pos }),
    };
    if block.arguments.len() > SHARED_MAX {
        return Err(Error { message: format!("trop de valeurs partagées : une page en partage au plus {SHARED_MAX}"), pos: block.pos });
    }
    let declared: Vec<String> = match program.root.argument("state").map(|a| &a.value) {
        Some(Value::Block(state)) if state.name == "State" => state.arguments.iter().filter_map(|a| a.name.clone()).collect(),
        Some(_) => return Err(Error { message: "« state » attend un bloc « State(...) », sur la page : state: State(cart: 0)".into(), pos: argument.pos }),
        None => Vec::new(),
    };
    for value in &block.arguments {
        let Some(name) = value.name.as_deref() else {
            return Err(Error { message: format!("une valeur partagée se déclare par son nom et son départ : {EXAMPLE}"), pos: value.pos });
        };
        let refusal = |message: String| Err(Error { message, pos: value.pos });
        match &value.value {
            Value::Integer(_) | Value::Number { unit: None, .. } => {}
            Value::Text(text) if text.chars().count() <= SHARED_TEXT_MAX => {}
            Value::Text(_) => return refusal(format!("« {name} » : un texte partagé fait au plus {SHARED_TEXT_MAX} caractères ; chaque page ouverte le reçoit")),
            Value::List(_) => return refusal(format!("« {name} » : une liste partagée n'existe pas encore (une dette de l'ADR-079) ; on partage un nombre ou un texte, Shared({name}: 0)")),
            _ => return refusal(format!("« {name} » : une valeur partagée est un nombre entier, un nombre à virgule ou un texte : Shared(seats: 20, price: 12.50, last: \"\")")),
        }
        // L'adresse d'abord : sa valeur est aussi dans State, mais c'est d'elle qu'elle vient.
        if program.address.iter().any(|known| known == name) {
            return refusal(format!("« {name} » vient de l'adresse de la page (le nom du fichier) : on la lit, elle ne peut pas être aussi partagée"));
        }
        if declared.iter().any(|known| known == name) {
            return refusal(format!("« {name} » est déjà dans State : une valeur est à chaque visiteur (State) ou à tous (Shared), pas aux deux"));
        }
    }
    let names: Vec<String> = block.arguments.iter().filter_map(|a| a.name.clone()).collect();
    match program.root.arguments.iter_mut().find(|a| a.name.as_deref() == Some("state")) {
        Some(Argument { value: Value::Block(state), .. }) => state.arguments.extend(block.arguments),
        _ => {
            let pos = block.pos;
            program.root.arguments.insert(index, Argument { name: Some("state".into()), value: Value::Block(Block { name: "State".into(), styles: Vec::new(), arguments: block.arguments, pos }), pos: argument.pos });
        }
    }
    program.shared = names;
    Ok(())
}

/// Une valeur partagée ne change que par un toucher, et le serveur l'arbitre : tout ce qui la
/// changerait sans lui est refusé, avec la raison.
pub fn check(program: &Program) -> Result<(), Error> {
    // `Shared` a été rangé avec `State` : un autre est mal placé.
    for_each_block(&program.root, &mut |block| {
        if block.name == "Shared" {
            return Err(Error { message: format!("« Shared » se déclare une seule fois, sur la page : {EXAMPLE}"), pos: block.pos });
        }
        Ok(())
    })?;
    if program.shared.is_empty() {
        return Ok(());
    }
    let is_shared = |name: &str| program.shared.iter().any(|known| known == name);
    let changed = |rule: &Block| crate::state::requests_of(rule).into_iter().find_map(|request| request.name.split_once('.').map(|(value, _)| value.to_string()).filter(|value| is_shared(value)));
    for_each_block(&program.root, &mut |block| {
        let refusal = |message: String| Err(Error { message, pos: block.pos });
        match block.name.as_str() {
            "On" => {
                let trigger = match block.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value) {
                    Some(Value::Name(signal)) => signal.as_str(),
                    _ => "",
                };
                if let Some(name) = changed(block).filter(|_| !trigger.ends_with(".tap")) {
                    return refusal(format!("« {name} » est partagée : seul un toucher la change, et le serveur l'arbitre, On(Book.tap, effect: {name}.sub(1)) ; « {trigger} » ne passe pas par lui"));
                }
            }
            "Every" | "After" => {
                if let Some(name) = changed(block) {
                    return refusal(format!("« {name} » est partagée : une horloge ({}) ne la change pas, chaque page ouverte la ferait changer ; seul un toucher la change, On(Book.tap, effect: {name}.sub(1))", block.name));
                }
            }
            "When" => {
                if let Some(name) = changed(block) {
                    return refusal(format!("« {name} » est partagée : une règle qui guette ne la change pas ; écris le changement dans la règle du toucher, On(Book.tap, effect: [{name}.sub(1), …])"));
                }
            }
            "Input" | "Checkbox" | "Choice" | "Slider" => {
                if let Some(Argument { value: Value::Name(name), pos, .. }) = block.argument("value").filter(|a| matches!(&a.value, Value::Name(n) if is_shared(n))) {
                    return Err(Error { message: format!("« {}(value: {name}) » : « {name} » est partagée par tous les visiteurs ; un champ ne la change pas encore (une dette de l'ADR-079) : un bouton et sa règle, On(Book.tap, effect: {name}.sub(1))", block.name), pos: *pos });
                }
            }
            "Module" => {
                if let Some(Argument { value: Value::Name(name), pos, .. }) = block.argument("output").filter(|a| matches!(&a.value, Value::Name(n) if is_shared(n))) {
                    return Err(Error { message: format!("« Module(output: {name}) » : « {name} » est partagée ; un module tourne dans la page d'un seul visiteur, il ne la change pas"), pos: *pos });
                }
            }
            _ => {}
        }
        // Un bloc qu'on fait glisser change sa place : elle ne peut pas être partagée.
        if matches!(block.argument("drag").map(|a| &a.value), Some(Value::Bool(true))) {
            for axis in ["x", "y"] {
                if let Some(Argument { value: Value::Name(name), pos, .. }) = block.argument(axis).filter(|a| matches!(&a.value, Value::Name(n) if is_shared(n))) {
                    return Err(Error { message: format!("« {axis}: {name} » : « {name} » est partagée ; un bloc qu'on fait glisser ne la change pas, seul un toucher la change"), pos: *pos });
                }
            }
        }
        Ok(())
    })?;
    // Garder dans le navigateur ce que le serveur garde pour tous n'aurait pas de sens.
    if let Some(Argument { value: Value::List(kept), pos, .. }) = program.root.argument("keep") {
        if let Some(name) = kept.iter().find_map(|value| match value {
            Value::Name(name) if is_shared(name) => Some(name),
            _ => None,
        }) {
            return Err(Error { message: format!("« keep » : « {name} » est partagée, le serveur la garde pour tous ; keep garde ce qui est à un seul visiteur, dans son navigateur"), pos: *pos });
        }
    }
    Ok(())
}

/// Les valeurs partagées d'un état, dans l'ordre de la page, écrites comme l'état :
/// `seats=19;likes=3;last='Ada`. Un texte partagé est coupé à sa longueur permise.
pub fn written(program: &Program, numbers: &State, texts: &Texts) -> String {
    let mut chunks = Vec::new();
    for name in &program.shared {
        if let Some((_, value)) = numbers.iter().find(|(known, _)| known == name) {
            chunks.push(format!("{name}={value}"));
        } else if let Some((_, text)) = texts.iter().find(|(known, _)| known == name) {
            chunks.push(format!("{name}='{}", crate::state::encode(&cut(text))));
        }
    }
    chunks.join(";")
}

/// L'état d'un visiteur, avec les valeurs partagées écrites dans `given` (celles que le serveur
/// garde) à la place des siennes. `given` est relu avec méfiance, comme tout état : seules les
/// valeurs partagées de la page en sont prises, dans leurs bornes ; une valeur absente vaut son
/// départ, celui du fichier.
pub fn merged(program: &Program, numbers: &State, texts: &Texts, given: &str) -> (State, Texts) {
    let (given_numbers, given_texts) = (crate::state::reread(program, given), crate::state::reread_texts(program, given));
    let is_shared = |name: &str| program.shared.iter().any(|known| known == name);
    let numbers = numbers
        .iter()
        .map(|(name, value)| match given_numbers.iter().find(|(known, _)| known == name).filter(|_| is_shared(name)) {
            Some((_, given)) => (name.clone(), *given),
            None => (name.clone(), *value),
        })
        .collect();
    let texts = texts
        .iter()
        .map(|(name, text)| match given_texts.iter().find(|(known, _)| known == name).filter(|_| is_shared(name)) {
            Some((_, given)) => (name.clone(), cut(given)),
            None => (name.clone(), text.clone()),
        })
        .collect();
    (numbers, texts)
}

/// Un texte partagé, coupé à sa longueur permise.
fn cut(text: &str) -> String {
    text.chars().take(SHARED_TEXT_MAX).collect()
}

/// Le bloc touché (`Book.tap`, `Done.tap@2`) se voit-il, pour cet état ? Un bouton rangé dans un
/// `If` dont la condition est fausse (ou dans son `else` quand elle est vraie) est caché : le
/// serveur refuse de le toucher, même si la demande arrive (une page en retard, ou forgée). C'est
/// ainsi qu'une condition garde une valeur partagée : `If(seats, over: 0, children: [ Button(…) ])`.
/// Les conditions sur l'élément d'une ligne (`If(item.done, …)`) ne sont pas regardées ici.
pub fn shown(program: &Program, written: &str, signal: &str) -> bool {
    let name = crate::lists::signal_and_line(signal).0.strip_suffix(".tap").unwrap_or("");
    if name.is_empty() {
        return false;
    }
    // Les conditions qui entourent le bloc touché : chacune, et si le bloc est dans son « else ».
    fn around<'a>(block: &'a Block, name: &str, path: &mut Vec<(&'a Block, bool)>) -> Option<Vec<(&'a Block, bool)>> {
        if crate::rules::name_of(block) == Some(name) {
            return Some(path.clone());
        }
        for argument in &block.arguments {
            let guarded = block.name == "If" && matches!(argument.name.as_deref(), Some("children" | "else"));
            if guarded {
                path.push((block, argument.name.as_deref() == Some("else")));
            }
            let found = inside(&argument.value, name, path);
            if guarded {
                path.pop();
            }
            if found.is_some() {
                return found;
            }
        }
        None
    }
    fn inside<'a>(value: &'a Value, name: &str, path: &mut Vec<(&'a Block, bool)>) -> Option<Vec<(&'a Block, bool)>> {
        match value {
            Value::Block(block) => around(block, name, path),
            Value::List(elements) => elements.iter().find_map(|e| inside(e, name, path)),
            _ => None,
        }
    }
    let Some(conditions) = around(&program.root, name, &mut Vec::new()) else { return false };
    let numbers = crate::state::reread(program, written);
    let texts = crate::state::reread_texts(program, written);
    let lists = crate::lists::reread(program, written);
    // Ce qu'une condition peut regarder, comme pour la page (`conditions` dans lib.rs).
    let mut values = crate::state::to_show(program, &numbers);
    values.extend(crate::lists::counts(&lists));
    let (computed, totals) = crate::computed::apply_with_totals(program, &numbers, &texts, &lists);
    values.extend(crate::lists::counts(&computed));
    values.extend(totals);
    values.extend(crate::computed::days_values(program, &texts));
    conditions.into_iter().filter(|(block, _)| crate::lists::element_subject(block).is_none()).all(|(block, in_else)| {
        let holds = crate::state::condition(block).is_ok_and(|(value, comparisons)| crate::state::holds(program, value, &comparisons, &values, &texts));
        holds != in_else
    })
}

#[cfg(test)]
mod tests {
    use crate::holo::{NAME_SEPARATOR, NEXT_FILE};

    const SEATS: &str = "Page(
  title: \"Concert\",
  state: State(booked: 0, name: \"\"),
  shared: Shared(seats: 2, likes: 0, last: \"\"),
  children: [
    H1(\"{seats} places\"),
    Input(value: name, label: \"Ton nom\"),
    If(seats, over: 0, children: [ Button(name: Book, text: \"Réserver\") ], else: [ P(\"Complet\") ]),
    If(booked, is: 1, children: [ P(\"Ta place est gardée\") ]),
    Button(name: Like, text: \"J'aime ({likes})\"),
  ],
  rules: [
    On(Book.tap, effect: [seats.sub(1), booked.set(1), last.set(name)]),
    On(Like.tap, effect: likes.add(1)),
  ],
)";

    #[test]
    fn shared_values_join_the_page_values() {
        let program = crate::check_page(SEATS).unwrap();
        assert_eq!(program.shared, ["seats", "likes", "last"]);
        assert_eq!(crate::initial_state(SEATS), "booked=0;seats=2;likes=0;name=';last='");
        let html = crate::flat_view(SEATS, "").unwrap();
        assert!(html.contains("<span data-state=\"seats\">2</span> places"), "{html}");
        // Une page qui partage est vivante : le moteur arrive tout de suite pour écouter.
        assert!(html.contains(" data-live"), "{html}");
        // Sans State : Shared en fait un.
        let alone = crate::check_page("Page(shared: Shared(likes: 0), children: [ P(\"{likes}\"), Button(name: L, text: \"+\") ], rules: [ On(L.tap, effect: likes.add(1)) ])").unwrap();
        assert_eq!(alone.shared, ["likes"]);
        assert_eq!(crate::initial_state("Page(shared: Shared(likes: 0, price: 12.50), children: [ P(\"{likes} {price}\") ])"), "likes=0;price=1250");
    }

    #[test]
    fn what_would_change_a_shared_value_without_the_server_is_refused() {
        let refused = |source: &str| crate::check_page(source).unwrap_err().message;
        let page = |state: &str, children: &str, rules: &str| format!("Page({state} shared: Shared(seats: 20), children: [ P(\"{{seats}}\"), Button(name: Book, text: \"x\"), Button(name: Other, text: \"y\"){children} ], rules: [ On(Book.tap, effect: seats.sub(1)){rules} ])");
        // Le même nom deux fois : dans State, dans l'adresse.
        assert!(refused(&page("state: State(seats: 1),", "", "")).contains("est déjà dans State"));
        let address = format!("{}{NEXT_FILE}@adresse{NAME_SEPARATOR}seats=1", page("", "", ""));
        assert!(refused(&address).contains("vient de l'adresse de la page"));
        // Ce qui la changerait sans le serveur.
        assert!(refused(&page("", "", ", Every(1s, effect: seats.add(1))")).contains("une horloge (Every)"));
        assert!(refused(&page("", "", ", After(2s, effect: seats.set(0))")).contains("une horloge (After)"));
        assert!(refused(&page("", "", ", When(seats, is: 0, effect: seats.set(20))")).contains("une règle qui guette"));
        assert!(refused(&page("", "", ", On(Book.hover, effect: seats.add(1))")).contains("« Book.hover » ne passe pas par lui"));
        assert!(refused(&page("", "", ", On(Key.space, effect: seats.add(1))")).contains("seul un toucher la change"));
        assert!(refused(&page("", ", Input(value: seats, label: \"Places\")", "")).contains("un champ ne la change pas encore"));
        assert!(refused(&page("", ", Board(children: [ Shape(name: S, form: circle, x: seats, y: seats, drag: true) ])", "")).contains("un bloc qu'on fait glisser"));
        assert!(refused(&format!("Page(shared: Shared(seats: 1), keep: [seats], children: [ P(\"{{seats}}\") ])")).contains("keep garde ce qui est à un seul visiteur"));
        // Les sortes et les limites.
        assert!(refused("Page(shared: Shared(seats: [\"a\"]), children: [ \"x\" ])").contains("une liste partagée n'existe pas encore"));
        assert!(refused(&format!("Page(shared: Shared(last: \"{}\"), children: [ \"x\" ])", "a".repeat(201))).contains("au plus 200 caractères"));
        let many: Vec<String> = (0..17).map(|i| format!("v{i}: 0")).collect();
        assert!(refused(&format!("Page(shared: Shared({}), children: [ \"x\" ])", many.join(", "))).contains("au plus 16"));
        assert!(refused("Page(shared: State(seats: 1), children: [ \"x\" ])").contains("attend un bloc « Shared(…) »"));
        assert!(refused("Page(children: [ Shared(seats: 1) ])").contains("se déclare une seule fois, sur la page"));
        assert!(refused("Page(shared: Shared(seats_left: 1), children: [ \"x\" ])").contains("seatsLeft"));
        assert!(refused("Page(shared: Shared(hour: 1), children: [ \"x\" ])").contains("l'heure du visiteur"));
        assert!(refused("Page(shared: Shared(seats: 1, seats: 2), children: [ \"x\" ])").contains("déclarée deux fois"));
        assert!(refused("Page(children: [ Point(name: P, seed: 1, inside: World(shared: Shared(a: 1), children: [ \"x\" ])) ])").contains("se règle sur la page, pas dans un monde"));
        // Lire une valeur partagée, la donner à une autre : permis.
        crate::check_page(&page("state: State(best: 0),", ", If(seats, is: 0, children: [ \"Complet\" ])", ", On(Other.tap, effect: best.set(seats)), When(seats, is: 0, effect: best.set(1))")).unwrap();
    }

    #[test]
    fn the_server_values_replace_the_visitor_ones() {
        // Le visiteur envoie son état ; ses valeurs partagées ne comptent pas : celles du serveur, si.
        let visitor = "booked=0;seats=99;likes=7;name='Ada;last='Moi";
        assert_eq!(crate::shared_of(SEATS, visitor), "seats=99;likes=7;last='Moi");
        let merged = crate::with_shared(SEATS, visitor, "seats=1;likes=3;last='Grace");
        assert_eq!(merged, "booked=0;seats=1;likes=3;name='Ada;last='Grace");
        // Une valeur absente vaut son départ ; une valeur qui n'est pas partagée ne passe pas.
        assert_eq!(crate::with_shared(SEATS, visitor, "booked=1;likes=4"), "booked=0;seats=2;likes=4;name='Ada;last='");
        // Un texte trop long est coupé ; un nombre, borné.
        let long = crate::state::encode(&"é".repeat(300));
        let cut = crate::with_shared(SEATS, "", &format!("seats=99999999999;last='{long}"));
        assert!(cut.contains("seats=1000000000;") && cut.ends_with(&format!("last='{}", crate::state::encode(&"é".repeat(200)))), "{cut}");
        assert_eq!(crate::shared_names(SEATS), "seats;likes;last");
        assert!(crate::touches_shared(SEATS, "Book.tap") && crate::touches_shared(SEATS, "Like.tap"));
        assert!(!crate::touches_shared(SEATS, "Like.hover") && !crate::touches_shared(SEATS, "Nobody.tap"));
    }

    #[test]
    fn the_server_arbitrates_and_a_hidden_button_is_not_touched() {
        // Deux places, trois visiteurs : chacun son tour ; le troisième trouve le bouton caché.
        let mut shared = String::new();
        let mut people = Vec::new();
        for name in ["Ada", "Grace", "Hedy"] {
            let visitor = crate::input(SEATS, &crate::initial_state(SEATS), "name", name);
            let (after, accepted) = crate::share(SEATS, &visitor, &shared, "Book.tap");
            people.push((name, accepted, after.clone()));
            shared = crate::shared_of(SEATS, &after);
        }
        assert_eq!(shared, "seats=0;likes=0;last='Grace");
        assert!(people[0].1 && people[1].1 && !people[2].1, "{people:?}");
        assert!(people[1].2.starts_with("booked=1;seats=0;") && people[2].2.starts_with("booked=0;seats=0;"), "{people:?}");
        // Un geste qui n'est pas un toucher, ou d'aucun bouton : rien ne change.
        for signal in ["Like.hover", "every:0", "Nobody.tap", "Book.tap@x"] {
            let (after, accepted) = crate::share(SEATS, "", "seats=2", signal);
            assert!(!accepted && after.starts_with("booked=0;seats=2;"), "{signal} : {after}");
        }
        // « J'aime » n'est gardé par aucune condition : il se touche toujours.
        let (after, accepted) = crate::share(SEATS, "", "seats=0;likes=41", "Like.tap");
        assert!(accepted && crate::shared_of(SEATS, &after).starts_with("seats=0;likes=42;"), "{after}");
        // Un texte partagé écrit d'après un texte trop long (un état forgé) : coupé, aussi pour celui
        // qui l'a écrit.
        let forged = format!("name='{}", crate::state::encode(&"é".repeat(300)));
        let (after, accepted) = crate::share(SEATS, &forged, "", "Book.tap");
        assert!(accepted && after.ends_with(&format!("last='{}", crate::state::encode(&"é".repeat(200)))), "{after}");
    }
}
