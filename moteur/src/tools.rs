//! Les outils de l'auteur (ADR-054) : remettre un fichier en forme, et jouer des essais écrits.
//!
//! ```text
//! holo fmt page.holo                  écrit le fichier remis en forme (deux espaces par niveau)
//! holo test page.holo page.test       joue les gestes écrits, vérifie les valeurs attendues
//! ```
//!
//! Un fichier d'essai, une ligne par geste ou par vérification :
//!
//! ```text
//! // Ajouter deux fois le tableau « Night » remplit le panier.
//! tap AddNight
//! tap AddNight
//! expect cart = 12000
//! type name "Forest"
//! expect name = "Forest"
//! expect articles = 4
//! receive {"articles": []}
//! ```
//!
//! `tap Nom` touche un bouton (le signal `Nom.tap`) ; `signal Nom.hover` envoie un autre signal ;
//! `type valeur "texte"` écrit dans un champ ; `receive {…}` fait comme si le serveur envoyait ces
//! données ; `expect valeur = …` vérifie un nombre, un texte entre guillemets, ou le nombre
//! d'éléments d'une liste. Les essais utilisent l'arbitre de la page : le même code que dans le
//! navigateur.

/// Remet un fichier `.holo` en forme, comme les leçons du dépôt : deux espaces de plus après
/// une ligne qui ouvre (une parenthèse, un crochet, une accolade, ou plusieurs à la fois), deux
/// de moins quand ce qu'elle a ouvert se referme ; les espaces de fin de ligne retirés ; jamais
/// plus d'une ligne vide de suite. Rien d'autre ne change : ni les mots, ni l'ordre, ni les
/// commentaires, ni le contenu d'un texte, même sur plusieurs lignes.
pub fn format_source(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    // Chaque niveau ouvert : combien de signes ouvrants il attend encore qu'on referme.
    let mut levels: Vec<usize> = Vec::new();
    let mut in_long_text = false;
    let mut empty_lines = 0;
    for line in source.lines() {
        let line = line.trim_end_matches('\r');
        if in_long_text {
            // Le contenu d'un texte long est gardé tel quel, à la lettre.
            output.push_str(line);
            output.push('\n');
            let mut signs = Vec::new();
            in_long_text = walk(line, true, &mut |c| signs.push(c));
            close_and_open(&mut levels, &signs);
            continue;
        }
        let clean = line.trim();
        if clean.is_empty() {
            empty_lines += 1;
            if empty_lines <= 1 && !output.is_empty() {
                output.push('\n');
            }
            continue;
        }
        empty_lines = 0;
        let mut signs = Vec::new();
        in_long_text = walk(clean, false, &mut |c| signs.push(c));
        let signs: Vec<char> = signs.into_iter().filter(|c| "()[]{}".contains(*c)).collect();
        // Une ligne qui commence par des fermetures se range au niveau de ce qu'elles referment.
        let header = clean.chars().take_while(|c| matches!(c, ')' | ']' | '}')).count();
        // Un niveau entamé par ces fermetures, même sans être refermé tout à fait, se range aussi.
        let mut remaining_ones = levels.clone();
        let mut keypresses = 0;
        for _ in 0..header {
            let rank = remaining_ones.len();
            let Some(last) = remaining_ones.last_mut() else { break };
            if *last == levels.get(rank - 1).copied().unwrap_or(0) {
                keypresses += 1;
            }
            *last -= 1;
            if *last == 0 {
                remaining_ones.pop();
            }
        }
        let indent = levels.len().saturating_sub(keypresses) * 2;
        // Un commentaire seul, aligné plus loin par l'auteur (la suite d'un commentaire de fin de
        // ligne), garde sa place.
        let already = line.len() - line.trim_start().len();
        if clean.starts_with("//") && already > indent {
            output.push_str(&line[..already]);
        } else {
            output.push_str(&" ".repeat(indent));
        }
        output.push_str(clean);
        output.push('\n');
        close_and_open(&mut levels, &signs);
    }
    while output.ends_with("\n\n") {
        output.pop();
    }
    output
}

/// Ferme puis ouvre les niveaux d'une ligne : ce qu'une ligne ouvre et referme elle-même ne
/// compte pas ; ce qu'elle laisse ouvert fait un seul niveau, quel qu'en soit le nombre.
fn close_and_open(levels: &mut Vec<usize>, signs: &[char]) {
    let mut opened_here = 0usize;
    for c in signs {
        match c {
            '(' | '[' | '{' => opened_here += 1,
            ')' | ']' | '}' if opened_here > 0 => opened_here -= 1,
            ')' | ']' | '}' => {
                if let Some(last) = levels.last_mut() {
                    *last -= 1;
                    if *last == 0 {
                        levels.pop();
                    }
                }
            }
            _ => {}
        }
    }
    if opened_here > 0 {
        levels.push(opened_here);
    }
}

/// Parcourt les signes d'une ligne qui sont du code (ni texte, ni commentaire) ; rend vrai si la
/// ligne se termine à l'intérieur d'un texte long (`"""`).
fn walk(line: &str, already_in_long_text: bool, f: &mut dyn FnMut(char)) -> bool {
    let signs: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut long = already_in_long_text;
    let mut short = false;
    while i < signs.len() {
        let triple = signs.get(i..i + 3).is_some_and(|t| t.iter().all(|c| *c == '"'));
        if long {
            if triple {
                long = false;
                i += 3;
            } else {
                i += 1;
            }
            continue;
        }
        if short {
            match signs[i] {
                '\\' => i += 2,
                '"' => {
                    short = false;
                    i += 1;
                }
                _ => i += 1,
            }
            continue;
        }
        if triple {
            long = true;
            i += 3;
            continue;
        }
        match signs[i] {
            '"' => short = true,
            '/' if signs.get(i + 1) == Some(&'/') => break,
            c => f(c),
        }
        i += 1;
    }
    long
}

/// Le résultat d'un essai : chaque ligne jouée, ou la première qui échoue.
#[derive(Debug, PartialEq)]
pub struct Test {
    pub succeeded: usize,
    pub failure: Option<(usize, String)>,
}

/// Joue un fichier d'essai sur une page (voir le haut de ce fichier).
pub fn play(source: &str, test: &str) -> Result<Test, crate::holo::Error> {
    let program = crate::check_page(source)?;
    let mut state = crate::initial_state(source);
    let mut succeeded = 0;
    for (rank, line) in test.lines().enumerate() {
        let number_ = rank + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let (verb, remainder) = line.split_once(' ').map_or((line, ""), |(v, r)| (v, r.trim()));
        let failure = |message: String| Ok(Test { succeeded, failure: Some((number_, message)) });
        match verb {
            "tap" if !remainder.is_empty() => state = crate::arbitrate(source, &state, &format!("{remainder}.tap")),
            "signal" if remainder.contains('.') => state = crate::arbitrate(source, &state, remainder),
            "type" => {
                let Some((name, text)) = remainder.split_once(' ') else { return failure("« type » s'écrit : type name \"Forest\"".into()) };
                let text = text.trim();
                let Some(text) = text.strip_prefix('"').and_then(|t| t.strip_suffix('"')) else { return failure("le texte de « type » s'écrit entre guillemets : type name \"Forest\"".into()) };
                state = crate::input(source, &state, name, text);
            }
            "receive" => state = crate::receive(source, &state, remainder),
            // Le visiteur défile (ADR-106) : `scroll 50`, de 0 (en haut de la page) à 100 (tout en bas).
            "scroll" => {
                let Ok(place @ 0..=100) = remainder.parse::<u64>() else { return failure("« scroll » s'écrit : scroll 50, de 0 (en haut de la page) à 100 (tout en bas)".into()) };
                if !crate::scroll::reads(&program) {
                    return failure("cette page ne lit pas « scroll » : rien à faire défiler pour elle".into());
                }
                state = crate::scrolled(source, &state, place);
            }
            "expect" => {
                let Some((name, expected)) = remainder.split_once('=') else { return failure("« expect » s'écrit : expect cart = 12000".into()) };
                let (name, expected) = (name.trim(), expected.trim());
                let Some(seen) = value_in(&program, &state, name) else { return failure(format!("la page n'a pas de valeur « {name} »")) };
                let expected_readable = expected.strip_prefix('"').and_then(|t| t.strip_suffix('"')).unwrap_or(expected);
                if seen != expected_readable {
                    return failure(format!("« {name} » vaut {}, et l'essai attendait {expected}", if expected.starts_with('"') { format!("\"{seen}\"") } else { seen }));
                }
            }
            _ => return failure(format!("ligne inconnue « {line} » ; on écrit : tap, signal, type, receive, scroll, expect")),
        }
        succeeded += 1;
    }
    Ok(Test { succeeded, failure: None })
}

/// Ce que vaut une valeur dans un état écrit : un nombre, un texte décodé, ou le nombre
/// d'éléments d'une liste.
fn value_in(program: &crate::holo::Program, state: &str, name: &str) -> Option<String> {
    let _ = program;
    state.split(';').find_map(|chunk| {
        let (n, v) = chunk.split_once('=')?;
        if n != name {
            return None;
        }
        Some(if let Some(text) = v.strip_prefix('\'') {
            crate::state::decode(text).unwrap_or_default()
        } else if let Some(list) = v.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            list.split(',').filter(|e| !e.is_empty()).count().to_string()
        } else {
            v.to_string()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_only_changes_whitespace() {
        let source = "Page(\ntitle: \"A (b\",   \n  children: [\n        H1(\"x\"), // un commentaire (\n P(\"\"\"\n   garde\n mes  espaces\n\"\"\"),\n\n\n\n Row(children: [\nText(\"]\")\n])\n],\n)\n\nP { color: red; }\n";
        let expected = "Page(\n  title: \"A (b\",\n  children: [\n    H1(\"x\"), // un commentaire (\n    P(\"\"\"\n   garde\n mes  espaces\n\"\"\"),\n\n    Row(children: [\n      Text(\"]\")\n    ])\n  ],\n)\n\nP { color: red; }\n";
        assert_eq!(format_source(source), expected);
        // Deux fois de suite, le même résultat ; et le moteur lit la même page.
        assert_eq!(format_source(expected), expected);
        for lesson in ["68-liste-qui-change.holo", "70-composants.holo", "43-texte-long.holo"] {
            let text = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../exemples/lecons").join(lesson)).unwrap();
            let shape = format_source(&text);
            assert_eq!(crate::flat_view(&shape, "").unwrap(), crate::flat_view(&text, "").unwrap(), "{lesson}");
            assert_eq!(format_source(&shape), shape, "{lesson}");
        }
    }

    #[test]
    fn a_written_test_plays_gestures_and_checks_values() {
        let page = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../exemples/lecons/70-composants.holo")).unwrap();
        let test = "// deux tableaux Night\ntap AddNight\ntap AddNight\nexpect night = 2\nexpect cart = 12000\n";
        assert_eq!(play(&page, test).unwrap(), Test { succeeded: 4, failure: None });
        let rate = play(&page, "tap AddNight\nexpect cart = 1\n").unwrap();
        assert_eq!(rate.failure, Some((2, "« cart » vaut 6000, et l'essai attendait 1".into())));
        assert!(play(&page, "dance\n").unwrap().failure.unwrap().1.contains("ligne inconnue"));
        let list = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../exemples/lecons/68-liste-qui-change.holo")).unwrap();
        let test = "type tache \"Pain\"\ntap Ajouter\nexpect taches = 2\nexpect tache = \"\"\ntap Vider\nexpect taches = 0\n";
        assert_eq!(play(&list, test).unwrap().failure, None);
    }

    #[test]
    fn each_example_test_passes() {
        // Un fichier `x.test` à côté de `x.holo` : il doit passer, comme les leçons.
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../exemples/lecons");
        let mut played = 0;
        for entry in std::fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("test") {
                continue;
            }
            let page = std::fs::read_to_string(path.with_extension("holo")).unwrap();
            let result = play(&page, &std::fs::read_to_string(&path).unwrap()).unwrap();
            assert_eq!(result.failure, None, "{}", path.display());
            played += 1;
        }
        assert!(played >= 2, "des essais ont disparu : {played}");
    }
}
