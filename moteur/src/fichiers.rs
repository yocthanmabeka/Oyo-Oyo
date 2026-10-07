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

use crate::holo::{Bloc, Erreur, Programme, Valeur};

/// Le nombre de fichiers qu'un formulaire peut envoyer, au plus.
pub const FICHIERS_PAR_FORMULAIRE: usize = 4;
/// La taille d'un fichier, au plus, en octets.
pub const TAILLE_MAX: u64 = 10_000_000;
/// La taille d'un fichier, sans `max:`.
pub const TAILLE_PAR_DEFAUT: u64 = 2_000_000;
/// Les sortes de fichiers permises, et ce que le navigateur en propose.
pub const SORTES: &[(&str, &str)] = &[("image", "image/png,image/jpeg,image/webp,image/gif"), ("pdf", "application/pdf")];

const EXEMPLE: &str = "Input(type: file, value: photo, label: \"Ta photo\", accept: image, max: 2MB)";

/// Un champ qui envoie un fichier : `Input(type: file, …)`.
pub fn est_un_fichier(bloc: &Bloc) -> bool {
    bloc.nom == "Input" && matches!(bloc.argument("type").map(|a| &a.valeur), Some(Valeur::Nom(t)) if t == "file")
}

/// Les sortes permises : `accept: image`, `accept: [image, pdf]`.
pub fn sortes(bloc: &Bloc) -> Result<Vec<String>, Erreur> {
    let refus = || Erreur { message: format!("« accept: » dit quels fichiers sont permis : image, pdf, ou [image, pdf] ; {EXEMPLE}"), pos: bloc.pos };
    let noms: Vec<&Valeur> = match bloc.argument("accept").map(|a| &a.valeur) {
        None => return Err(Erreur { message: format!("un champ de fichier dit quels fichiers il accepte : {EXEMPLE}"), pos: bloc.pos }),
        Some(Valeur::Liste(l)) if !l.is_empty() => l.iter().collect(),
        Some(v) => vec![v],
    };
    let mut sortes = Vec::new();
    for nom in noms {
        match nom {
            Valeur::Nom(n) if SORTES.iter().any(|(s, _)| s == n) && !sortes.contains(n) => sortes.push(n.clone()),
            _ => return Err(refus()),
        }
    }
    Ok(sortes)
}

/// La taille permise, en octets : `max: 2MB`, `max: 500KB`.
pub fn taille_max(bloc: &Bloc) -> Result<u64, Erreur> {
    match bloc.argument("max").map(|a| &a.valeur) {
        None => Ok(TAILLE_PAR_DEFAUT),
        Some(Valeur::Nombre { valeur, unite: Some(u) }) if u == "KB" || u == "MB" => {
            let octets = (if u == "MB" { valeur * 1e6 } else { valeur * 1e3 }).round();
            if (1e3..=TAILLE_MAX as f64).contains(&octets) {
                Ok(octets as u64)
            } else {
                Err(Erreur { message: "« max: » d'un fichier va de 1KB à 10MB".into(), pos: bloc.pos })
            }
        }
        Some(_) => Err(Erreur { message: format!("« max: » d'un fichier est une taille, en KB ou en MB : max: 2MB ; {EXEMPLE}"), pos: bloc.pos }),
    }
}

/// Ce que le navigateur propose de choisir : `accept="image/png,…"`.
pub fn accept_html(sortes: &[String]) -> String {
    sortes.iter().filter_map(|s| SORTES.iter().find(|(n, _)| n == s).map(|(_, a)| *a)).collect::<Vec<_>>().join(",")
}

/// Un champ de fichier, tel que le serveur doit le vérifier.
#[derive(Debug, Clone, PartialEq)]
pub struct Fichier {
    pub formulaire: String,
    pub valeur: String,
    pub sortes: Vec<String>,
    pub max: u64,
}

/// Vérifie les champs de fichier de la page, et les rend : leur formulaire, leur valeur, ce qu'ils acceptent.
pub fn fichiers(programme: &Programme) -> Result<Vec<Fichier>, Erreur> {
    let mut trouves = Vec::new();
    chercher(&programme.racine, None, &mut trouves)?;
    Ok(trouves)
}

fn chercher(bloc: &Bloc, formulaire: Option<&str>, trouves: &mut Vec<Fichier>) -> Result<(), Erreur> {
    let dans = if bloc.nom == "Form" { crate::regles::nom_de(bloc).or(Some("")) } else { formulaire };
    if est_un_fichier(bloc) {
        let Some(formulaire) = dans else {
            return Err(Erreur { message: "un fichier ne part que par un formulaire : range ce champ dans Form(name: Contact, children: [ … ])".into(), pos: bloc.pos });
        };
        if bloc.argument("lines").is_some() {
            return Err(Erreur { message: format!("« lines: » ne sert pas à un fichier : {EXEMPLE}"), pos: bloc.pos });
        }
        let Some(Valeur::Nom(valeur)) = bloc.argument("value").map(|a| &a.valeur) else {
            return Err(Erreur { message: format!("un champ de fichier présente un texte, le nom du fichier choisi : {EXEMPLE}"), pos: bloc.pos });
        };
        if trouves.iter().filter(|f: &&Fichier| f.formulaire == formulaire).count() >= FICHIERS_PAR_FORMULAIRE {
            return Err(Erreur { message: format!("un formulaire envoie {FICHIERS_PAR_FORMULAIRE} fichiers au plus"), pos: bloc.pos });
        }
        if trouves.iter().any(|f| f.valeur == *valeur) {
            return Err(Erreur { message: format!("« {valeur} » reçoit déjà un fichier : chaque champ de fichier a sa propre valeur"), pos: bloc.pos });
        }
        trouves.push(Fichier { formulaire: formulaire.to_string(), valeur: valeur.clone(), sortes: sortes(bloc)?, max: taille_max(bloc)? });
    }
    for argument in &bloc.arguments {
        visiter(&argument.valeur, dans, trouves)?;
    }
    Ok(())
}

fn visiter(valeur: &Valeur, formulaire: Option<&str>, trouves: &mut Vec<Fichier>) -> Result<(), Erreur> {
    match valeur {
        Valeur::Bloc(b) => chercher(b, formulaire, trouves),
        Valeur::Liste(l) => l.iter().try_for_each(|v| visiter(v, formulaire, trouves)),
        _ => Ok(()),
    }
}

/// Pour le serveur : une ligne par champ de fichier, `Formulaire|valeur|image,pdf|octets`.
pub fn pour_le_serveur(programme: &Programme) -> Result<String, Erreur> {
    Ok(fichiers(programme)?.iter().map(|f| format!("{}|{}|{}|{}", f.formulaire, f.valeur, f.sortes.join(","), f.max)).collect::<Vec<_>>().join("\n"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn un_fichier_dans_un_formulaire() {
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
        let programme = crate::verifier_page(source).unwrap();
        assert_eq!(super::pour_le_serveur(&programme).unwrap(), "Candidature|photo|image|500000\nCandidature|devis|image,pdf|2000000");
        let html = crate::plat::page_html(&programme, "").unwrap();
        assert!(html.contains("<input type=\"file\" accept=\"image/png,image/jpeg,image/webp,image/gif\" data-max=\"500000\" data-bind=\"photo\">"), "{html}");
        assert!(html.contains("accept=\"image/png,image/jpeg,image/webp,image/gif,application/pdf\" data-max=\"2000000\""), "{html}");
        // Le nom du fichier choisi est un texte de la page, comme ce qu'on écrit.
        let etat = crate::saisir(source, "", "photo", "C:\\fakepath\\sunrise.png");
        let envoi = crate::envoi(source, &etat, "Candidature");
        assert!(envoi.contains("\"photo\":\"sunrise.png\""), "{envoi}");
        let page = |champ: &str| format!("Page(state: State(photo: \"\", n: 0), children: [ H1(\"x\"), Form(name: F, children: [ {champ} ]) ])");
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
            let erreur = crate::verifier_page(&source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }
}
