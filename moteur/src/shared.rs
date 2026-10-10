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
/// Première liste partagée : cinquante textes et 16 Kio pour toutes les valeurs codées.
pub const SHARED_LIST_MAX: usize = 50;
pub const SHARED_BYTES_MAX: usize = 16_384;

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
            // Une liste partagée (ADR-080) : cinquante textes, ou cinquante fiches `Item(…)`, de deux
            // cents caractères par texte ; la forme des fiches est vérifiée avec les autres listes.
            Value::List(elements)
                if elements.len() <= SHARED_LIST_MAX
                    && elements.iter().all(|element| match element {
                        Value::Text(text) => text.chars().count() <= SHARED_TEXT_MAX,
                        Value::Block(item) if item.name == "Item" => item.arguments.iter().all(|a| !matches!(&a.value, Value::Text(text) if text.chars().count() > SHARED_TEXT_MAX)),
                        _ => false,
                    }) => {}
            Value::List(_) => return refusal(format!("« {name} » : une liste partagée attend au plus {SHARED_LIST_MAX} textes, ou fiches Item(…), de {SHARED_TEXT_MAX} caractères par texte")),
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
                    if block.name != "Input" || !crate::state::initial_texts(program).iter().any(|(n,_)|n==name) || !has_confirmation(program,name) {
                        return Err(Error { message: format!("« {}(value: {name}) » : un champ ne la change pas encore sans confirmation ; un texte partagé se prépare par Input(value: {name}), puis On(Save.tap, effect: {name}.set({name}))", block.name), pos: *pos });
                    }
                }
            }
            "Module" => {
                // Une valeur rendue seule (`output: likes`) ou dans une liste (`output: [likes, total]`, ADR-077).
                if let Some(argument) = block.argument("output") {
                    let names: Vec<&String> = match &argument.value {
                        Value::Name(name) => vec![name],
                        Value::List(values) => values.iter().filter_map(|v| if let Value::Name(n) = v { Some(n) } else { None }).collect(),
                        _ => Vec::new(),
                    };
                    if let Some(name) = names.into_iter().find(|n| is_shared(n)) {
                        return Err(Error { message: format!("« Module(output: …) » : « {name} » est partagée ; un module tourne dans la page d'un seul visiteur, il ne la change pas"), pos: argument.pos });
                    }
                }
            }
            _ => {}
        }
        // Une liste partagée se change comme une autre (push, remove(item), clear(), item.done.set(1)),
        // vérifiée par les listes : une ligne touchée se désigne par sa clé (ADR-080, line_by_key).
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
    let start=written(program,&crate::state::initial(program).unwrap_or_default(),&crate::state::initial_texts(program),&crate::lists::initial(program));
    if start.len()>SHARED_BYTES_MAX{return Err(Error{message:format!("valeurs partagées : {SHARED_BYTES_MAX} octets codés au plus"),pos:program.root.pos});}
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
pub fn written(program: &Program, numbers: &State, texts: &Texts, lists: &crate::lists::Lists) -> String {
    let mut chunks = Vec::new();
    for name in &program.shared {
        if let Some((_, value)) = numbers.iter().find(|(known, _)| known == name) {
            chunks.push(format!("{name}={value}"));
        } else if let Some((_, text)) = texts.iter().find(|(known, _)| known == name) {
            // Un dessin partagé (ADR-115) a ses propres bornes : il n'est pas coupé.
            let text = if crate::sketch::is_drawing(program, name) { text.clone() } else { cut(text) };
            chunks.push(format!("{name}='{}", crate::state::encode(&text)));
        } else if let Some((_,values))=lists.iter().find(|(n,_)|n==name){
            chunks.push(crate::lists::write(&vec![(name.clone(),values.clone())]));
        }
    }
    chunks.join(";")
}

/// L'état d'un visiteur, avec les valeurs partagées écrites dans `given` (celles que le serveur
/// garde) à la place des siennes. `given` est relu avec méfiance, comme tout état : seules les
/// valeurs partagées de la page en sont prises, dans leurs bornes ; une valeur absente vaut son
/// départ, celui du fichier.
pub fn merged(program: &Program, numbers: &State, texts: &Texts, lists: &crate::lists::Lists, given: &str) -> (State, Texts, crate::lists::Lists) {
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
            Some((_, given)) if crate::sketch::is_drawing(program, name) => (name.clone(), given.clone()),
            Some((_, given)) => (name.clone(), cut(given)),
            None => (name.clone(), text.clone()),
        })
        .collect();
    let authoritative=crate::lists::reread(program,given);
    let lists=lists.iter().map(|(name,items)|{let items=authoritative.iter().find(|(n,_)|n==name).filter(|_|is_shared(name)).map_or(items,|(_,v)|v);(name.clone(),items.clone())}).collect();
    (numbers, texts, lists)
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
        // Un module rend ses valeurs, une seule ou une liste (ADR-077) : jamais une valeur partagée.
        for output in ["seats", "[best, seats]"] {
            let module = format!("module \"s.wasm\"\nPage(state: State(best: 0), shared: Shared(seats: 20), modules: [ Module(name: S, source: \"s.wasm\", input: best, output: {output}) ], children: [ P(\"{{seats}} {{best}}\"), Button(name: Go, text: \"g\") ], rules: [ On(Go.tap, effect: S.run) ])");
            assert!(refused(&module).contains("« seats » est partagée ; un module tourne"), "{output} : {}", refused(&module));
        }
        // Les sortes et les limites.
        assert!(refused("Page(shared: Shared(seats: [1, 2]), children: [ \"x\" ])").contains("une liste partagée attend"));
        let fifty_one: Vec<String> = (0..51).map(|i| format!("\"n{i}\"")).collect();
        assert!(refused(&format!("Page(shared: Shared(names: [{}]), children: [ \"x\" ])", fifty_one.join(", "))).contains("une liste partagée attend au plus 50"));
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

/// Le champ prépare un texte ; seul son toucher de confirmation le publie.
fn has_confirmation(program:&Program,name:&str)->bool{
 let mut yes=false;let _=for_each_block(&program.root,&mut|b|{
  let tap=b.name=="On"&&b.arguments.iter().find(|a|a.name.is_none()).is_some_and(|a|matches!(&a.value,Value::Name(n) if n.ends_with(".tap")));
  if tap&&crate::state::requests_of(b).iter().any(|r|r.name==format!("{name}.set")&&matches!(r.arguments.as_slice(),[Argument{name:None,value:Value::Name(n),..}]if n==name)){yes=true;}Ok(())
 });yes
}
/// Copier la proposition d'un champ dans la seule demande explicite qui la confirme.
/// La condition et l'état partagé restent ceux du serveur, jamais ceux du brouillon.
pub fn with_drafts(program:&Program,candidate:&str,signal:&str)->Program{
 let mut result=program.clone();let names=crate::state::initial_texts(program).into_iter().filter(|(n,_)|program.shared.contains(n)&&has_confirmation(program,n)).map(|(n,_)|n).collect::<Vec<_>>();
 let parsed=crate::state::reread_texts(program,candidate);
 let values=names.into_iter().filter_map(|n|{
  // Un champ absent ne publie pas sa valeur de départ.
  if !candidate.split(';').any(|c|c.starts_with(&format!("{n}='"))){return None;}
  let value=parsed.iter().find(|(known,_)|known==&n)?.1.clone();
  let checked=crate::state::input_text(program,&parsed,&n,&value);
  Some((n.clone(),cut(&checked.iter().find(|(known,_)|known==&n)?.1)))
 }).collect::<Vec<_>>();
 fn visit(b:&mut Block,values:&[(String,String)],signal:&str){
  let is_tap=b.name=="On"&&b.arguments.iter().find(|a|a.name.is_none()).is_some_and(|a|matches!(&a.value,Value::Name(n)if n==signal));
  if is_tap{if let Some(a)=b.arguments.iter_mut().find(|a|a.name.as_deref()==Some("effect")){
   fn change(v:&mut Value,values:&[(String,String)]){match v{Value::Block(r)=>{
    if let Some((n,_))=r.name.split_once('.').filter(|(_,verb)|*verb=="set"){
     if let Some((_,text))=values.iter().find(|(known,_)|known==n){if let [Argument{name:None,value:Value::Name(own),..}]=r.arguments.as_slice(){if own==n{r.arguments[0].value=Value::Text(text.clone());}}}
    }
   },Value::List(a)=>a.iter_mut().for_each(|v|change(v,values)),_=>{}}}
   change(&mut a.value,values);
  }}
  fn child(v:&mut Value,values:&[(String,String)],signal:&str){match v{Value::Block(b)=>visit(b,values,signal),Value::List(a)=>a.iter_mut().for_each(|v|child(v,values,signal)),_=>{}}}
  for a in &mut b.arguments{child(&mut a.value,values,signal);}
 }visit(&mut result.root,&values,crate::lists::signal_and_line(signal).0);result
}
/// Une liste partagée reste dans ses bornes : cinquante éléments, des textes (ou des champs de
/// fiche) de deux cents caractères, 16 Kio pour toutes les valeurs partagées codées.
pub fn within_budget(program:&Program,state:&str)->bool{
 let lists=crate::lists::reread(program,state);
 let short=|text:&str|text.chars().count()<=SHARED_TEXT_MAX;
 lists.iter().filter(|(n,_)|program.shared.contains(n)).all(|(_,v)|v.len()<=SHARED_LIST_MAX&&v.iter().all(|t|if t.starts_with(crate::lists::RECORD){crate::lists::fields(t).iter().all(|(_,field)|short(field))}else{short(t)}))
 &&written(program,&crate::state::reread(program,state),&crate::state::reread_texts(program,state),&lists).len()<=SHARED_BYTES_MAX
}

/// Les listes dont une ligne se désigne par sa clé (ADR-080) : les listes partagées, et les
/// listes calculées d'après elles (`Filter(from: names, …)`).
pub fn keyed_lists(program: &Program) -> Vec<String> {
    let mut keyed: Vec<String> = program.shared.iter().filter(|name| crate::lists::kind(program, name).is_some()).cloned().collect();
    for (_, list) in crate::lists::repeats(program) {
        let from_shared = crate::computed::is_computed(program, &list) && crate::computed::source_of(program, &list).is_some_and(|source| program.shared.contains(&source));
        if from_shared && !keyed.contains(&list) {
            keyed.push(list);
        }
    }
    keyed
}

/// Les répétitions dont une règle de ligne répond à ce geste (`Remove.tap`), avec leur liste.
fn answering<'a>(program: &'a Program, base: &str) -> Vec<(&'a Block, String)> {
    crate::lists::repeats(program)
        .into_iter()
        .filter(|(repeat, _)| match repeat.argument("rules").map(|a| &a.value) {
            Some(Value::List(rules)) => rules.iter().any(|rule| matches!(rule, Value::Block(b) if b.name == "On" && matches!(b.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value), Some(Value::Name(s)) if s == base))),
            _ => false,
        })
        .collect()
}

/// Ce geste de ligne change-t-il une fiche d'une liste partagée (`item.done.set(1)`) ? Il part
/// alors au serveur, comme un geste qui demande la liste elle-même.
pub fn changes_shared_line(program: &Program, signal: &str) -> bool {
    let base = crate::lists::signal_and_line(signal).0;
    let keyed = keyed_lists(program);
    answering(program, base).into_iter().filter(|(_, list)| keyed.contains(list)).any(|(repeat, _)| match repeat.argument("rules").map(|a| &a.value) {
        Some(Value::List(rules)) => rules.iter().any(|rule| match rule {
            Value::Block(b) if b.name == "On" && matches!(b.arguments.iter().find(|a| a.name.is_none()).map(|a| &a.value), Some(Value::Name(s)) if s == base) => crate::state::requests_of(b).iter().any(|r| r.name.starts_with("item.")),
            _ => false,
        }),
        _ => false,
    })
}

/// Le geste d'une ligne d'une liste partagée porte la clé de la ligne touchée
/// (`Remove.tap@2#k:Ada-0`, ADR-080) : entre la page vue et le toucher, d'autres ont pu ajouter ou
/// retirer des lignes, et le rang ne dit plus laquelle. Le serveur la retrouve par sa clé dans la
/// liste qu'il garde (`written`), et rend le geste au rang qu'elle y a maintenant
/// (`Remove.tap@0`). Sans clé, si la ligne n'y est plus, ou si deux répétitions répondent au même
/// geste : `None`, le geste est refusé. La ligne d'une liste à chaque visiteur garde son rang.
pub fn line_by_key(program: &Program, written: &str, signal: &str) -> Option<String> {
    let (base, Some(_)) = crate::lists::signal_and_line(signal) else { return Some(signal.to_string()) };
    let plain = signal.split_once('#').map_or(signal, |(plain, _)| plain).to_string();
    let keyed = keyed_lists(program);
    let answering = answering(program, base);
    if !answering.iter().any(|(_, list)| keyed.contains(list)) {
        return Some(plain);
    }
    let [(repeat, list)] = answering.as_slice() else { return None };
    let key = crate::lists::line_key_of(signal)?;
    let numbers = crate::state::reread(program, written);
    let texts = crate::state::reread_texts(program, written);
    let lists = crate::lists::reread(program, written);
    let computed = crate::computed::apply(program, &numbers, &texts, &lists);
    let elements = &lists.iter().chain(computed.iter()).find(|(name, _)| name == list)?.1;
    let field = match repeat.argument("key").map(|a| &a.value) {
        Some(Value::Name(field)) => Some(field.as_str()),
        _ => None,
    };
    let rank = (0..elements.len()).find(|rank| crate::lists::line_key(elements, *rank, field) == key)?;
    Some(format!("{base}@{rank}"))
}

#[cfg(test)]
mod completion_tests {
    use crate::state::encode;

    const NAMES: &str = "Page(state: State(note: \"\", sent: 0), shared: Shared(names: []), children: [Input(value: note, label: \"Nom\"), Button(name: Add, text: \"Ajouter\"), Button(name: Clear, text: \"Effacer\"), Repeat(over: names, children: [P(\"{item}\"), Button(name: Remove, text: \"Retirer\")], rules: [On(Remove.tap, effect: names.remove(item))])], rules: [On(Add.tap, effect: [names.push(note), note.set(\"\"), sent.add(1)]), On(Clear.tap, effect: names.clear())])";

    /// Les clés des lignes d'une liste, telles que la page les écrit (`data-key`).
    fn keys(source: &str, state: &str, list: &str) -> Vec<String> {
        crate::list_html(source, "", state, list).split(" data-key=\"").skip(1).map(|rest| rest[..rest.find('"').unwrap()].replace("&amp;", "&")).collect()
    }

    #[test]
    fn concurrent_list_appends_start_from_server_and_stay_bounded() {
        let mut shared = String::new();
        for i in 0..50 {
            let forged = format!("sent=0;note='Nom{i};names=[Forged]");
            let (after, accepted) = crate::share(NAMES, &forged, &shared, "Add.tap");
            assert!(accepted, "{i} : {after}");
            shared = crate::shared_of(NAMES, &after);
            assert!(!shared.contains("Forged"));
        }
        let (after, accepted) = crate::share(NAMES, "sent=0;note='EnTrop", &shared, "Add.tap");
        assert!(!accepted);
        assert_eq!(crate::shared_of(NAMES, &after), shared);
        assert!(after.contains("sent=0"));
        let (after, accepted) = crate::share(NAMES, "", &shared, "Clear.tap");
        assert!(accepted);
        assert_eq!(crate::shared_of(NAMES, &after), "names=[]");
    }

    #[test]
    fn a_shared_field_is_a_draft_until_its_explicit_confirmation() {
        let source = "Page(shared: Shared(title: \"Initial\"), children: [Input(value: title, label: \"Titre\"), If(title, is: \"Initial\", children: [Button(name: Save, text: \"Publier\")])], rules: [On(Save.tap, effect: title.set(title))])";
        let (after, accepted) = crate::share(source, "title='Brouillon", "title='Initial", "Save.tap");
        assert!(accepted);
        assert_eq!(after, "title='Brouillon");
        // Une valeur forgée ne révèle pas au serveur un bouton maintenant caché.
        let published = format!("title='{}", encode("Publié"));
        let (after, accepted) = crate::share(source, "title='Initial", &published, "Save.tap");
        assert!(!accepted);
        assert_eq!(after, published);
        let absent = "Page(shared: Shared(title: \"\"), children: [Input(value: title, label: \"Titre\")])";
        assert!(crate::check_page(absent).unwrap_err().message.contains("sans confirmation"));
    }

    #[test]
    fn a_line_of_a_shared_list_is_found_by_its_key() {
        // Deux pages voient Ada, Bob et Cy ; la première retire Ada.
        let start = crate::input(NAMES, &crate::initial_state(NAMES), "note", "Ada");
        let mut shared = String::new();
        for name in ["Ada", "Bob", "Cy"] {
            let (after, _) = crate::share(NAMES, &crate::input(NAMES, &start, "note", name), &shared, "Add.tap");
            shared = crate::shared_of(NAMES, &after);
        }
        let seen = crate::with_shared(NAMES, "", &shared);
        let shown = keys(NAMES, &seen, "names");
        assert_eq!(shown.len(), 3);
        let (after, accepted) = crate::share(NAMES, &seen, &shared, &format!("Remove.tap@0#{}", shown[0]));
        assert!(accepted, "{after}");
        shared = crate::shared_of(NAMES, &after);
        assert_eq!(shared, "names=[Bob,Cy]");
        // La seconde, en retard, touche « Retirer » sur la ligne de Bob, au rang 1 qu'elle voit
        // encore : c'est Bob qui part, et non Cy, qui est maintenant au rang 1.
        let (after, accepted) = crate::share(NAMES, &seen, &shared, &format!("Remove.tap@1#{}", shown[1]));
        assert!(accepted, "{after}");
        shared = crate::shared_of(NAMES, &after);
        assert_eq!(shared, "names=[Cy]");
        // Ada n'y est plus : son geste est refusé. De même sans clé, ou avec une clé inventée.
        for signal in [format!("Remove.tap@0#{}", shown[0]), "Remove.tap@0".to_string(), "Remove.tap@0#abc-0".to_string()] {
            let (after, accepted) = crate::share(NAMES, &seen, &shared, &signal);
            assert!(!accepted, "{signal}");
            assert_eq!(crate::shared_of(NAMES, &after), "names=[Cy]", "{signal}");
        }
        // Le geste part au serveur, avec ou sans clé : la page ne le joue pas seule.
        assert!(crate::touches_shared(NAMES, "Remove.tap@0") && crate::touches_shared(NAMES, &format!("Remove.tap@0#{}", shown[2])));
        // Sans JavaScript, chaque bouton de ligne porte la clé de sa ligne.
        let page = crate::visitor_page(NAMES, "", &crate::with_shared(NAMES, "", &shared), &[]).unwrap();
        let cy = keys(NAMES, &crate::with_shared(NAMES, "", &shared), "names").remove(0);
        assert!(page.contains(&format!("value=\"Remove.tap@0#{cy}\"")), "{page}");
        // Deux éléments pareils : chacun a sa clé, et le premier part d'abord.
        let twins = "names=[Ana,Ana]";
        let both = keys(NAMES, &crate::with_shared(NAMES, "", twins), "names");
        assert_ne!(both[0], both[1]);
        let (after, accepted) = crate::share(NAMES, "", twins, &format!("Remove.tap@1#{}", both[1]));
        assert!(accepted);
        assert_eq!(crate::shared_of(NAMES, &after), "names=[Ana]");
    }

    const CHORES: &str = "Page(state: State(what: \"\"), shared: Shared(chores: [Item(what: \"Pain\", done: 0)]), children: [Input(value: what, label: \"Quoi\"), Button(name: Add, text: \"Ajouter\"), Repeat(over: chores, children: [P(\"{item.what}\"), Button(name: Done, text: \"Fait\"), Button(name: Remove, text: \"Retirer\")], rules: [On(Done.tap, effect: item.done.set(1)), On(Remove.tap, effect: chores.remove(item))])], rules: [On(Add.tap, effect: [chores.push(Item(what: what, done: 0)), what.set(\"\")])])";

    #[test]
    fn shared_records_are_added_ticked_and_removed_by_their_key() {
        let program = crate::check_page(CHORES).unwrap();
        let fields = |state: &str| -> Vec<Vec<(String, String)>> { crate::lists::reread(&program, state).into_iter().find(|(name, _)| name == "chores").unwrap().1.iter().map(|e| crate::lists::fields(e)).collect() };
        let pair = |what: &str, done: &str| vec![("what".to_string(), what.to_string()), ("done".to_string(), done.to_string())];
        // Une fiche ajoutée, avec le texte du champ.
        let (after, accepted) = crate::share(CHORES, &crate::input(CHORES, &crate::initial_state(CHORES), "what", "Lait"), "", "Add.tap");
        assert!(accepted);
        let shared = crate::shared_of(CHORES, &after);
        assert_eq!(fields(&shared), [pair("Pain", "0"), pair("Lait", "0")]);
        // Cocher une fiche change la liste de tous : le geste part au serveur.
        assert!(crate::touches_shared(CHORES, "Done.tap@1"));
        let seen = crate::with_shared(CHORES, "", &shared);
        let shown = keys(CHORES, &seen, "chores");
        let (after, accepted) = crate::share(CHORES, &seen, &shared, &format!("Done.tap@1#{}", shown[1]));
        assert!(accepted);
        let ticked = crate::shared_of(CHORES, &after);
        assert_eq!(fields(&ticked), [pair("Pain", "0"), pair("Lait", "1")]);
        // La fiche a changé, sa clé aussi : un geste d'une page en retard sur elle est refusé.
        let (after, accepted) = crate::share(CHORES, &seen, &ticked, &format!("Remove.tap@1#{}", shown[1]));
        assert!(!accepted);
        assert_eq!(crate::shared_of(CHORES, &after), ticked);
        // Retirer le pain, par sa clé.
        let (after, accepted) = crate::share(CHORES, &seen, &ticked, &format!("Remove.tap@0#{}", shown[0]));
        assert!(accepted);
        assert_eq!(fields(&crate::shared_of(CHORES, &after)), [pair("Lait", "1")]);
        // Des fiches de deux cents caractères par champ, cinquante au plus.
        let long = format!("Page(shared: Shared(chores: [Item(what: \"{}\")]), children: [ \"x\" ])", "a".repeat(201));
        assert!(crate::check_page(&long).unwrap_err().message.contains("une liste partagée attend"));
    }

    #[test]
    fn a_line_of_a_list_computed_from_a_shared_list_is_found_in_its_source() {
        let source = "Page(state: State(search: \"\"), shared: Shared(names: [\"Ada\", \"Bob\", \"Abe\"]), computed: [Filter(name: found, from: names, contains: search)], children: [Input(value: search, label: \"Chercher\"), Repeat(over: found, children: [P(\"{item}\"), Button(name: Remove, text: \"x\")], rules: [On(Remove.tap, effect: names.remove(item))])])";
        assert_eq!(super::keyed_lists(&crate::check_page(source).unwrap()), ["names", "found"]);
        let seen = crate::input(source, &crate::initial_state(source), "search", "A");
        let found = keys(source, &seen, "found");
        assert!(!found.is_empty());
        let before = crate::lists::reread(&crate::check_page(source).unwrap(), &seen).into_iter().find(|(n, _)| n == "names").unwrap().1;
        let (after, accepted) = crate::share(source, &seen, "", &format!("Remove.tap@0#{}", found[0]));
        assert!(accepted, "{after}");
        let left = crate::lists::reread(&crate::check_page(source).unwrap(), &after).into_iter().find(|(n, _)| n == "names").unwrap().1;
        assert_eq!(left.len(), before.len() - 1);
    }
}
