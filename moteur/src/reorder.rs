//! Réordonner les lignes d'une liste (ADR-105) : `Repeat(over: tasks, reorder: true, …)`.
//!
//! ```holo
//! Page(
//!   state: State(tasks: ["Pain", "Lait", "Œufs"]),
//!   children: [ Repeat(over: tasks, reorder: true, children: [ Text("{item}") ]) ],
//! )
//! ```
//!
//! Le moteur pose sur chaque ligne une poignée (⠿), qu'on fait glisser à la souris ou au doigt,
//! et deux boutons, « Monter » et « Descendre », qu'on touche au doigt, à la souris, au clavier
//! ou au lecteur d'écran, et qui marchent aussi sans JavaScript (`holo serve`). Aucune règle à
//! écrire : la page dit à l'arbitre « l'élément du rang 2 va au rang 0 » (`move:tasks@2:0`), et
//! c'est lui qui change la liste, comme pour un bloc qu'on fait glisser sur un plateau
//! (`drag: true`, ADR-028). Une liste calculée a l'ordre qu'elle calcule, et une liste partagée
//! ne se réordonne pas encore : les deux sont refusées.

use crate::holo::{Block, Error, Program, Value};
use crate::lists::Lists;

/// Le début du signal d'une ligne déplacée : `move:tasks@2:0`.
pub const MOVE: &str = "move:";

/// Lit le signal d'une ligne déplacée : `move:tasks@2:0` → (`tasks`, 2, 0). Rien d'autre ne
/// passe : un nom de liste (une minuscule, puis des lettres et des chiffres), deux rangs de six
/// chiffres au plus.
pub fn read(signal: &str) -> Option<(&str, usize, usize)> {
    let (list, ranks) = signal.strip_prefix(MOVE)?.split_once('@')?;
    let (from, to) = ranks.split_once(':')?;
    let rank = |r: &str| if !r.is_empty() && r.len() <= 6 && r.bytes().all(|b| b.is_ascii_digit()) { r.parse::<usize>().ok() } else { None };
    let named = list.len() <= 64 && list.starts_with(|c: char| c.is_ascii_lowercase()) && list.chars().all(|c| c.is_ascii_alphanumeric());
    named.then_some((list, rank(from)?, rank(to)?))
}

/// Est-ce le geste d'une ligne déplacée ?
pub fn is_move(signal: &str) -> bool {
    read(signal).is_some()
}

/// La répétition laisse-t-elle réordonner ses lignes (`reorder: true`) ?
pub fn reorders(repeat: &Block) -> bool {
    matches!(repeat.argument("reorder").map(|a| &a.value), Some(Value::Bool(true)))
}

/// Les listes que la page laisse réordonner : celles d'un `Repeat(over: …, reorder: true)`.
pub fn reorderable(program: &Program) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for (repeat, list) in crate::lists::repeats(program) {
        if reorders(repeat) && !names.contains(&list) {
            names.push(list);
        }
    }
    names
}

/// Vérifie `reorder:` : `true` ou `false`, et seulement sur une liste à soi, déclarée dans
/// `State`. L'ordre d'une liste calculée vient d'elle ; une liste partagée attend que le serveur
/// sache arbitrer un déplacement.
pub fn check(program: &Program) -> Result<(), Error> {
    for (repeat, list) in crate::lists::repeats(program) {
        let Some(argument) = repeat.argument("reorder") else { continue };
        let refusal = |message: String| Err(Error { message, pos: argument.pos });
        match &argument.value {
            Value::Bool(false) => {}
            // Un texte découpé (ADR-103) garde l'ordre de son texte.
            Value::Bool(true) if crate::computed::split_source(program, &list).is_some() => {
                let from = crate::computed::split_source(program, &list).unwrap_or_default();
                return refusal(format!("« {list} » est découpée dans le texte « {from} » : ses morceaux gardent l'ordre du texte ; pour une liste qu'on réordonne, déclare-la dans State"));
            }
            Value::Bool(true) if crate::computed::is_computed(program, &list) => {
                let source = crate::computed::source_of(program, &list).unwrap_or_default();
                return refusal(format!("« {list} » est une liste calculée : son ordre vient d'elle (sortBy, reverse) ; réordonne sa source, Repeat(over: {source}, reorder: true, …)"));
            }
            Value::Bool(true) if program.shared.iter().any(|known| *known == list) => {
                return refusal(format!("« {list} » est partagée : ses lignes ne se réordonnent pas encore, il faudrait que le serveur arbitre chaque déplacement ; réordonne une liste à toi, déclarée dans State"));
            }
            Value::Bool(true) => {}
            _ => return refusal("« Repeat(reorder: …) » attend true ou false : Repeat(over: tasks, reorder: true, children: [ … ])".into()),
        }
    }
    Ok(())
}

/// L'arbitre d'une ligne déplacée : l'élément du rang `from` va au rang `to`, les autres gardent
/// leur ordre. Rien ne change si la liste ne se laisse pas réordonner, ou si un rang est hors de la
/// liste (une page en retard, un geste forgé).
pub fn moved(program: &Program, lists: &Lists, signal: &str) -> Lists {
    let mut lists = lists.clone();
    let Some((list, from, to)) = read(signal) else { return lists };
    if !reorderable(program).iter().any(|known| known == list) {
        return lists;
    }
    if let Some((_, elements)) = lists.iter_mut().find(|(name, _)| name == list) {
        if from < elements.len() && to < elements.len() && from != to {
            let element = elements.remove(from);
            elements.insert(to, element);
        }
    }
    lists
}

/// Le nom d'une ligne, que disent les boutons et l'annonce : le texte de l'élément ; pour une
/// fiche, son premier champ de texte qui n'est ni sa clé (`key: id`) ni un nombre, sinon son
/// premier champ.
pub fn label(repeat: &Block, element: &str) -> String {
    let fields = crate::lists::fields(element);
    if fields.is_empty() {
        return crate::lists::text_of(element);
    }
    let key = match repeat.argument("key").map(|a| &a.value) {
        Some(Value::Name(key)) => Some(key.as_str()),
        _ => None,
    };
    fields
        .iter()
        .find(|(name, value)| Some(name.as_str()) != key && !value.trim().is_empty() && value.parse::<u64>().is_err())
        .or_else(|| fields.first())
        .map(|(_, value)| value.clone())
        .unwrap_or_default()
}

/// Une ligne qu'on réordonne : la poignée ⠿ (pour la souris et le doigt ; cachée au lecteur
/// d'écran, et sans JavaScript), le contenu de la ligne, puis « Monter » et « Descendre », de
/// vrais boutons, nommés avec la ligne, dans la langue de la page.
pub fn controls(repeat: &Block, element: &str, line: &str) -> String {
    let name = crate::flat::escape(&label(repeat, element));
    let language = crate::format::language();
    let (grip, up, down, quoted) = if language.is_empty() || language.starts_with("fr") {
        ("Glisser pour déplacer", "Monter", "Descendre", format!("« {name} »"))
    } else {
        ("Drag to move", "Move up", "Move down", format!("“{name}”"))
    };
    format!(
        "<div class=\"holo-movable\" data-label=\"{name}\"><span class=\"holo-move holo-grip\" aria-hidden=\"true\" title=\"{grip}\">⠿</span><div class=\"holo-line-content\">{line}</div>\
<button type=\"button\" class=\"holo-move holo-up\" data-move=\"up\" aria-label=\"{up} {quoted}\">↑</button><button type=\"button\" class=\"holo-move holo-down\" data-move=\"down\" aria-label=\"{down} {quoted}\">↓</button></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAINTINGS: &str = r#"Page(
  state: State(paintings: ["Sunrise", "The river", "The blue door"], other: ["A", "B"]),
  computed: [ Filter(name: first, from: paintings, limit: 1) ],
  children: [
    Column(gap: 8px, children: [
      Repeat(over: paintings, reorder: true, children: [ Text("{item}"), Button(name: Remove, text: "Remove") ],
             rules: [ On(Remove.tap, effect: paintings.remove(item)) ]),
    ]),
    Repeat(over: other, children: [ Text("{item}") ]),
    Repeat(over: first, children: [ P("First: {item}") ]),
  ],
)"#;

    #[test]
    fn a_line_moves_through_the_arbiter() {
        let start = crate::initial_state(PAINTINGS);
        assert!(start.contains("paintings=[Sunrise,The%20river,The%20blue%20door]"), "{start}");
        // La troisième ligne va en tête : la liste calculée d'après elle suit.
        let after = crate::arbitrate(PAINTINGS, &start, "move:paintings@2:0");
        assert!(after.contains("paintings=[The%20blue%20door,Sunrise,The%20river]"), "{after}");
        assert!(after.contains("first=[The%20blue%20door]"), "{after}");
        // Puis elle descend d'un rang.
        let down = crate::arbitrate(PAINTINGS, &after, "move:paintings@0:1");
        assert!(down.contains("paintings=[Sunrise,The%20blue%20door,The%20river]"), "{down}");
        // Rien ne change : une liste qui ne se réordonne pas, un rang hors de la liste, le même
        // rang, une liste inconnue, un signal mal écrit.
        for signal in ["move:other@1:0", "move:paintings@3:0", "move:paintings@0:3", "move:paintings@1:1", "move:nothing@1:0", "move:paintings@-1:0", "move:paintings@1", "move:Paintings@1:0", "move:paintings@1234567:0"] {
            assert_eq!(crate::arbitrate(PAINTINGS, &start, signal), start, "{signal}");
        }
        // Le geste d'une ligne retire toujours le bon élément après un déplacement.
        let removed = crate::arbitrate(PAINTINGS, &after, "Remove.tap@0");
        assert!(removed.contains("paintings=[Sunrise,The%20river]"), "{removed}");
        assert!(is_move("move:paintings@2:0") && !is_move("Remove.tap@0") && !is_move("move:paintings@2>0"));
    }

    #[test]
    fn the_lines_carry_a_grip_and_two_named_buttons() {
        let html = crate::flat_view(PAINTINGS, "").unwrap();
        // La poignée, cachée au lecteur d'écran ; le contenu ; « Monter » et « Descendre », nommés avec la ligne.
        assert!(html.contains(r#"<div class="holo-line" data-rank="2" data-key="#), "{html}");
        assert!(html.contains(r#"<div class="holo-movable" data-label="The blue door"><span class="holo-move holo-grip" aria-hidden="true" title="Glisser pour déplacer">⠿</span><div class="holo-line-content"><div class="holo-Text">The blue door</div><button type="button" class="holo-Button" data-name="Remove">Remove</button></div><button type="button" class="holo-move holo-up" data-move="up" aria-label="Monter « The blue door »">↑</button><button type="button" class="holo-move holo-down" data-move="down" aria-label="Descendre « The blue door »">↓</button></div>"#), "{html}");
        // Une répétition sans reorder ne change pas.
        assert!(html.contains(r#"-0"><div class="holo-Text">A</div></div>"#), "{html}");
        assert_eq!(html.matches("class=\"holo-movable\"").count(), 3);
        // La page est vivante : le moteur arrive tout de suite, un glissement ne se rejoue pas.
        assert!(html.contains(" data-live"), "{html}");
        // En anglais ; et le nom d'une fiche : son premier texte qui n'est pas sa clé.
        let english = r#"Page(lang: "en", state: State(tasks: [ Item(id: "t1", title: "Buy <bread>", done: 0) ]), children: [ Repeat(over: tasks, key: id, reorder: true, children: [ Text("{item.title}") ]) ])"#;
        let html = crate::flat_view(english, "").unwrap();
        assert!(html.contains(r#"data-label="Buy &lt;bread&gt;""#) && html.contains(r#"aria-label="Move up “Buy &lt;bread&gt;”""#) && html.contains(r#"title="Drag to move""#), "{html}");
        // Les lignes refaites par la page (list_html) ont les mêmes boutons.
        let lines = crate::list_html(PAINTINGS, "", &crate::arbitrate(PAINTINGS, &crate::initial_state(PAINTINGS), "move:paintings@2:0"), "paintings");
        assert!(lines.starts_with(r#"<div class="holo-line" data-rank="0" data-key=""#) && lines.contains(r#"data-label="The blue door"><span"#), "{lines}");
    }

    #[test]
    fn without_javascript_the_buttons_send_the_move() {
        let state = crate::initial_state(PAINTINGS);
        let page = crate::visitor_page(PAINTINGS, "", &state, &[]).unwrap();
        // « Monter » de la ligne 1 part au rang 0, « Descendre » au rang 2 ; un bouton qui ne mène
        // nulle part (Monter en tête) ne change rien.
        assert!(page.contains(r#"<button type="submit" form="holo-gestures" name="signal" value="move:paintings@1:0" class="holo-move holo-up""#), "{page}");
        assert!(page.contains(r#"value="move:paintings@1:2" class="holo-move holo-down""#), "{page}");
        assert!(page.contains(r#"value="move:paintings@0:0" class="holo-move holo-up""#), "{page}");
        // La poignée reste ce qu'elle est : sans JavaScript, elle est cachée par le style.
        assert!(page.contains(r#"<span class="holo-move holo-grip" aria-hidden="true""#), "{page}");
        // Le serveur passe le geste à l'arbitre, avec les champs, comme un toucher.
        let after = crate::visitor_gesture(PAINTINGS, &state, &[("signal".to_string(), "move:paintings@1:0".to_string())]);
        assert!(after.contains("paintings=[The%20river,Sunrise,The%20blue%20door]"), "{after}");
        let forged = crate::visitor_gesture(PAINTINGS, &state, &[("signal".to_string(), "move:other@1:0".to_string())]);
        assert_eq!(forged, state);
    }

    #[test]
    fn what_is_refused() {
        for (source, message) in [
            (PAINTINGS.replace("reorder: true", "reorder: 1"), "attend true ou false"),
            (PAINTINGS.replace("Repeat(over: first, children:", "Repeat(over: first, reorder: true, children:"), "« first » est une liste calculée"),
            ("Page(shared: Shared(names: [\"Ada\"]), children: [ Repeat(over: names, reorder: true, children: [ Text(\"{item}\") ]) ])".to_string(), "« names » est partagée"),
            ("Page(children: [ Repeat(items: [ Item(t: \"a\") ], reorder: true, children: [ P(\"{item.t}\") ]) ])".to_string(), "réordonne les lignes d'une liste qui change"),
        ] {
            let error = crate::check_page(&source).err().unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
        // reorder: false, c'est comme ne rien écrire.
        let off = crate::flat_view(&PAINTINGS.replace("reorder: true", "reorder: false"), "").unwrap();
        assert!(!off.contains("class=\"holo-movable\"") && !off.contains(" data-live"), "{off}");
    }
}
