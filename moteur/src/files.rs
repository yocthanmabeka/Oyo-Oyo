//! L'envoi d'un fichier par un formulaire (ADR-059).
//!
//! ```holo
//! Form(name: Candidature, children: [
//!   Input(value: nom, label: "Ton nom"),
//!   Input(type: file, value: photo, label: "Une photo de ton tableau", accept: image, max: 2MB),
//!   Button(name: Envoyer, text: "Envoyer"),
//! ])
//! ```
//!
//! - La valeur (`photo`) est un texte : le nom du fichier choisi, `""` sinon. On peut le montrer,
//!   `{photo}`, et le tester, `If(photo, not: "")`.
//! - `accept:` est obligatoire : `image` (PNG, JPEG, WebP, GIF), `pdf`, ou les deux, `[image, pdf]`.
//! - `max:` de 1KB à 10MB ; 2MB si on ne l'écrit pas.
//! - Un fichier ne part que par un formulaire, quatre au plus par formulaire.
//! - La page vérifie la taille et la sorte avant l'envoi ; le serveur vérifie tout à nouveau,
//!   la sorte d'après les premiers octets du fichier, jamais d'après son nom.

use crate::holo::{Block, Error, Program, Value};

/// Le nombre de fichiers qu'un formulaire peut envoyer, au plus.
pub const FILES_PER_FORM: usize = 4;
/// La taille d'un fichier, au plus, en octets.
pub const SIZE_MAX: u64 = 10_000_000;
/// La taille d'un fichier, sans `max:`.
pub const DEFAULT_SIZE: u64 = 2_000_000;
/// Les sortes de fichiers permises, et ce que le navigateur en propose.
pub const KINDS: &[(&str, &str)] = &[("image", "image/png,image/jpeg,image/webp,image/gif"), ("pdf", "application/pdf")];

const EXAMPLE: &str = "Input(type: file, value: photo, label: \"Ta photo\", accept: image, max: 2MB)";

/// Un champ qui envoie un fichier : `Input(type: file, …)`.
pub fn is_file(block: &Block) -> bool {
    block.name == "Input" && matches!(block.argument("type").map(|a| &a.value), Some(Value::Name(t)) if t == "file")
}

/// Les sortes permises : `accept: image`, `accept: [image, pdf]`.
pub fn kinds(block: &Block) -> Result<Vec<String>, Error> {
    let refusal = || Error { message: format!("« accept: » dit quels fichiers sont permis : image, pdf, ou [image, pdf] ; {EXAMPLE}"), pos: block.pos };
    let names: Vec<&Value> = match block.argument("accept").map(|a| &a.value) {
        None => return Err(Error { message: format!("un champ de fichier dit quels fichiers il accepte : {EXAMPLE}"), pos: block.pos }),
        Some(Value::List(l)) if !l.is_empty() => l.iter().collect(),
        Some(v) => vec![v],
    };
    let mut kinds = Vec::new();
    for name in names {
        match name {
            Value::Name(n) if KINDS.iter().any(|(s, _)| s == n) && !kinds.contains(n) => kinds.push(n.clone()),
            _ => return Err(refusal()),
        }
    }
    Ok(kinds)
}

/// La taille permise, en octets : `max: 2MB`, `max: 500KB`.
pub fn max_size(block: &Block) -> Result<u64, Error> {
    match block.argument("max").map(|a| &a.value) {
        None => Ok(DEFAULT_SIZE),
        Some(Value::Number { value, unit: Some(u), .. }) if u == "KB" || u == "MB" => {
            let bytes = (if u == "MB" { value * 1e6 } else { value * 1e3 }).round();
            if (1e3..=SIZE_MAX as f64).contains(&bytes) {
                Ok(bytes as u64)
            } else {
                Err(Error { message: "« max: » d'un fichier va de 1KB à 10MB".into(), pos: block.pos })
            }
        }
        Some(_) => Err(Error { message: format!("« max: » d'un fichier est une taille, en KB ou en MB : max: 2MB ; {EXAMPLE}"), pos: block.pos }),
    }
}

/// Ce que le navigateur propose de choisir : `accept="image/png,…"`.
pub fn accept_html(kinds: &[String]) -> String {
    kinds.iter().filter_map(|s| KINDS.iter().find(|(n, _)| n == s).map(|(_, a)| *a)).collect::<Vec<_>>().join(",")
}

/// Un champ de fichier, tel que le serveur doit le vérifier.
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub form_name: String,
    pub value: String,
    pub kinds: Vec<String>,
    pub max: u64,
}

/// Vérifie les champs de fichier de la page, et les rend : leur formulaire, leur valeur, ce qu'ils acceptent.
pub fn files(program: &Program) -> Result<Vec<File>, Error> {
    let mut found_list = Vec::new();
    search(&program.root, None, &mut found_list)?;
    Ok(found_list)
}

fn search(block: &Block, form_name: Option<&str>, found_list: &mut Vec<File>) -> Result<(), Error> {
    let inside = if block.name == "Form" { crate::rules::name_of(block).or(Some("")) } else { form_name };
    if is_file(block) {
        let Some(form_name) = inside else {
            return Err(Error { message: "un fichier ne part que par un formulaire : range ce champ dans Form(name: Contact, children: [ … ])".into(), pos: block.pos });
        };
        if block.argument("lines").is_some() {
            return Err(Error { message: format!("« lines: » ne sert pas à un fichier : {EXAMPLE}"), pos: block.pos });
        }
        let Some(Value::Name(value)) = block.argument("value").map(|a| &a.value) else {
            return Err(Error { message: format!("un champ de fichier présente un texte, le nom du fichier choisi : {EXAMPLE}"), pos: block.pos });
        };
        if found_list.iter().filter(|f: &&File| f.form_name == form_name).count() >= FILES_PER_FORM {
            return Err(Error { message: format!("un formulaire envoie {FILES_PER_FORM} fichiers au plus"), pos: block.pos });
        }
        if found_list.iter().any(|f| f.value == *value) {
            return Err(Error { message: format!("« {value} » reçoit déjà un fichier : chaque champ de fichier a sa propre valeur"), pos: block.pos });
        }
        found_list.push(File { form_name: form_name.to_string(), value: value.clone(), kinds: kinds(block)?, max: max_size(block)? });
    }
    for argument in &block.arguments {
        visit(&argument.value, inside, found_list)?;
    }
    Ok(())
}

fn visit(value: &Value, form_name: Option<&str>, found_list: &mut Vec<File>) -> Result<(), Error> {
    match value {
        Value::Block(b) => search(b, form_name, found_list),
        Value::List(l) => l.iter().try_for_each(|v| visit(v, form_name, found_list)),
        _ => Ok(()),
    }
}

/// Pour le serveur : une ligne par champ de fichier, `Formulaire|valeur|image,pdf|octets`.
pub fn for_server(program: &Program) -> Result<String, Error> {
    Ok(files(program)?.iter().map(|f| format!("{}|{}|{}|{}", f.form_name, f.value, f.kinds.join(","), f.max)).collect::<Vec<_>>().join("\n"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_file_in_a_form() {
        let source = r#"Page(
  state: State(nom: "", photo: "", devis: ""),
  children: [
    H1("Apply"),
    Form(name: Candidature, children: [
      Input(value: nom, label: "Name"),
      Input(type: file, value: photo, label: "A photo", accept: image, max: 500KB),
      Input(type: file, value: devis, label: "A quote", accept: [image, pdf]),
      Button(name: Envoyer, text: "Send"),
    ]),
    If(photo, not: "", children: [ P("Chosen: {photo}") ]),
  ],
  rules: [ On(Envoyer.tap, effect: Candidature.send) ],
)"#;
        let program = crate::check_page(source).unwrap();
        assert_eq!(super::for_server(&program).unwrap(), "Candidature|photo|image|500000\nCandidature|devis|image,pdf|2000000");
        let html = crate::flat::page_html(&program, "").unwrap();
        assert!(html.contains("<input type=\"file\" accept=\"image/png,image/jpeg,image/webp,image/gif\" data-max=\"500000\" data-bind=\"photo\">"), "{html}");
        assert!(html.contains("accept=\"image/png,image/jpeg,image/webp,image/gif,application/pdf\" data-max=\"2000000\""), "{html}");
        // Le nom du fichier choisi est un texte de la page, comme ce qu'on écrit.
        let state = crate::input(source, "", "photo", "C:\\fakepath\\sunrise.png");
        let submission = crate::submission(source, &state, "Candidature");
        assert!(submission.contains("\"photo\":\"sunrise.png\""), "{submission}");
        let page = |field: &str| format!("Page(state: State(photo: \"\", n: 0), children: [ H1(\"x\"), Form(name: F, children: [ {field} ]) ])");
        for (source, message) in [
            ("Page(state: State(photo: \"\"), children: [ H1(\"x\"), Input(type: file, value: photo, label: \"P\", accept: image) ])".to_string(), "que par un formulaire"),
            (page("Input(type: file, value: photo, label: \"P\")"), "quels fichiers il accepte"),
            (page("Input(type: file, value: photo, label: \"P\", accept: video)"), "image, pdf"),
            (page("Input(type: file, value: photo, label: \"P\", accept: image, max: 20MB)"), "de 1KB à 10MB"),
            (page("Input(type: file, value: photo, label: \"P\", accept: image, max: 3)"), "en KB ou en MB"),
            (page("Input(type: file, value: n, label: \"P\", accept: image)"), "écrit un texte"),
            (page("Input(value: photo, label: \"P\", accept: image)"), "Input(type: file, …)"),
            (page("Input(type: file, value: photo, label: \"P\", accept: image), Input(type: file, value: photo, label: \"Q\", accept: image)"), "déjà un fichier"),
        ] {
            let error = crate::check_page(&source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }
}
