//! Le chronomètre (ADR-089) : un bloc que la page dessine elle-même, au rythme de l'écran, quand
//! une règle le démarre. Le moteur ne reçoit que le temps final, en millisecondes, dans la valeur
//! que le chronomètre nomme ; puis `Chrono.stopped`. Remettre à zéro efface le cadran, pas la
//! valeur : elle garde le dernier temps final.
//!
//! ```holo
//! Page(
//!   state: State(time: 0),
//!   children: [
//!     Stopwatch(name: Chrono, value: time, label: "Ma course"),
//!     Button(name: Go, text: "Démarrer"),
//!     Button(name: Halt, text: "Arrêter"),
//!     P("Ton temps : {time:stopwatch}"),
//!   ],
//!   rules: [ On(Go.tap, effect: Chrono.start), On(Halt.tap, effect: Chrono.stop) ],
//! )
//! ```

use crate::holo::{Block, Error, Program, Value};

/// La valeur où un chronomètre range son temps final, s'il en nomme une.
pub fn value_of<'a>(program: &'a Program, name: &str) -> Option<&'a str> {
    let mut found = None;
    let _ = crate::rules::for_each_block(&program.root, &mut |block| {
        if block.name == "Stopwatch" && block.argument("name").is_some_and(|a| matches!(&a.value, Value::Name(n) if n == name)) {
            if let Some(Value::Name(value)) = block.argument("value").map(|a| &a.value) {
                found = Some(value.as_str());
            }
        }
        Ok(())
    });
    found
}

/// Vérifie les chronomètres : un nom, et une valeur qui est un nombre entier de la page.
pub fn check(program: &Program) -> Result<(), Error> {
    let numbers = crate::state::initial(program).unwrap_or_default();
    crate::rules::for_each_block(&program.root, &mut |block: &Block| {
        if block.name != "Stopwatch" {
            return Ok(());
        }
        let example = "Stopwatch(name: Chrono, value: time, label: \"Ma course\")";
        if !matches!(block.argument("name").map(|a| &a.value), Some(Value::Name(_))) {
            return Err(Error { message: format!("« Stopwatch » a un nom, pour que les règles le démarrent et l'arrêtent : {example}"), pos: block.pos });
        }
        for argument in &block.arguments {
            match (argument.name.as_deref(), &argument.value) {
                (Some("name"), _) | (Some("label"), Value::Text(_)) => {}
                (Some("value"), Value::Name(value)) => {
                    if crate::state::CLOCK.contains(&value.as_str()) || !numbers.iter().any(|(known, _)| known == value) {
                        return Err(Error { message: format!("« Stopwatch(value: {value}) » : le temps final va dans un nombre de la page, en millisecondes ; déclare-le, state: State({value}: 0)"), pos: argument.pos });
                    }
                    if crate::state::places(program, value) > 0 {
                        return Err(Error { message: format!("« Stopwatch(value: {value}) » : le temps se compte en millisecondes, un nombre entier"), pos: argument.pos });
                    }
                }
                (Some("label"), _) => return Err(Error { message: "« Stopwatch(label: …) » attend un texte entre guillemets : ce que le lecteur d'écran annonce".into(), pos: argument.pos }),
                (Some("value"), _) => return Err(Error { message: format!("« Stopwatch(value: …) » attend le nom d'un nombre de la page : {example}"), pos: argument.pos }),
                (Some(other), _) => return Err(Error { message: format!("« Stopwatch » n'a pas de paramètre « {other} » ; paramètres possibles : name, value, label"), pos: argument.pos }),
                (None, _) => return Err(Error { message: format!("chaque paramètre de « Stopwatch » est nommé : {example}"), pos: argument.pos }),
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    const RACE: &str = "Page(state: State(time: 0, laps: 0), children: [ Stopwatch(name: Chrono, value: time, label: \"Ma course\"), Button(name: Go, text: \"Go\"), Button(name: Halt, text: \"Stop\"), P(\"Ton temps : {time:stopwatch}\") ], rules: [ On(Go.tap, effect: Chrono.start), On(Halt.tap, effect: Chrono.stop), On(Chrono.stopped, effect: laps.add(1)) ])";

    #[test]
    fn a_stopwatch_is_drawn_and_receives_its_final_time() {
        let html = crate::flat_view(RACE, "").unwrap();
        assert!(html.contains("<span class=\"holo-Stopwatch\" data-name=\"Chrono\" role=\"timer\" aria-label=\"Ma course\" data-stopwatch=\"time\">00:00,00</span>"), "{html}");
        // Le temps final arrive, puis Chrono.stopped.
        let after = crate::stopwatch_stopped(RACE, &crate::initial_state(RACE), "Chrono", 83_456);
        assert!(after.starts_with("time=83456;laps=1"), "{after}");
        // Les règles qui guettent le temps répondent : le meilleur temps est gardé.
        let best = "Page(state: State(time: 0, best: 0, tries: 0), children: [ Stopwatch(name: C, value: time) ], rules: [ On(C.stopped, effect: tries.add(1)), When(tries, is: 1, effect: best.set(time)), When(time, under: best, effect: best.set(time)) ])";
        let first = crate::stopwatch_stopped(best, &crate::initial_state(best), "C", 9_000);
        let slower = crate::stopwatch_stopped(best, &first, "C", 12_000);
        let faster = crate::stopwatch_stopped(best, &slower, "C", 7_500);
        assert!(first.starts_with("time=9000;best=9000;tries=1") && slower.starts_with("time=12000;best=9000;tries=2") && faster.starts_with("time=7500;best=7500;tries=3"), "{first}\n{slower}\n{faster}");
        // La page servie montre le temps gardé, écrit comme un chronomètre.
        let shown = crate::flat_view_with_data(&RACE.replace("Page(", "Page(data: Data(from: \"d.json\"), "), "", r#"{"time": 61500}"#).unwrap();
        assert!(shown.contains(">01:01,50</span>") && shown.contains("Ton temps : <span data-state=\"time\" data-format=\"stopwatch\">01:01,50</span>"), "{shown}");
        for (source, message) in [
            ("Page(children: [ Stopwatch(value: t) ])", "a un nom"),
            ("Page(children: [ Stopwatch(name: C, value: t) ])", "le temps final va dans un nombre de la page"),
            ("Page(state: State(t: 1.5), children: [ Stopwatch(name: C, value: t) ])", "un nombre entier"),
            ("Page(children: [ Stopwatch(name: C, label: 3) ])", "attend un texte"),
            ("Page(children: [ Stopwatch(name: C, speed: 3) ])", "n'a pas de paramètre « speed »"),
            ("Page(state: State(t: 0), children: [ Stopwatch(name: C, value: t), Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: C.pause) ])", "start"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn the_second_is_given_only_to_pages_that_read_it() {
        crate::set_now([2026, 10, 8, 4, 9, 41]);
        crate::state::set_second(37);
        let clock = "Page(children: [ P(\"Il est {hour} h {minute:00} min {second:00} s.\") ])";
        assert!(crate::initial_state(clock).contains("second=37"), "{}", crate::initial_state(clock));
        assert!(crate::flat_view(clock, "").unwrap().contains(">37</span> s."));
        assert!(crate::reads_seconds(clock) && !crate::reads_seconds("Page(children: [ P(\"{minute}\") ])"));
        assert!(crate::check_page("Page(state: State(second: 0), children: [])").unwrap_err().message.contains("l'heure du visiteur"));
    }
}
