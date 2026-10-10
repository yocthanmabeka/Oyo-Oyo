//! Le champ mot de passe (ADR-114) : `Input(type: password, label: "…")`, dans un formulaire.
//!
//! ```holo
//! Form(name: Protect, children: [
//!   Input(value: title, label: "Le nom du carnet"),
//!   Input(type: password, new: true, label: "Un mot de passe pour ce carnet"),
//!   Button(name: Send, text: "Protéger"),
//! ])
//! ```
//!
//! Le moteur tient les règles, jamais l'auteur :
//! - **la page ne lit jamais ce qui est tapé** : le champ n'a pas de valeur (`value:` est refusé).
//!   Ni une règle, ni un texte `{…}`, ni l'état, ni `keep`, `visit`, `Shared`, l'adresse ne
//!   l'atteignent. Le moteur du navigateur le lit une seule fois, au moment d'envoyer le
//!   formulaire, et le met dans l'envoi, à côté des valeurs, jamais parmi elles ;
//! - **seulement en HTTPS, ou sur ce PC** : ailleurs, le champ se ferme et dit pourquoi ;
//! - sans `new`, c'est **le mot de passe du compte** du visiteur (`ADR-081`), que `holo serve`
//!   vérifie (Argon2id, avec le frein des comptes) : la page est réservée aux membres. Avec
//!   `new: true`, **un mot de passe que le visiteur choisit** : l'auteur n'en reçoit que l'empreinte
//!   Argon2id, celle des comptes. Créer un compte et se connecter restent les pages du moteur ;
//! - les longueurs de NIST SP 800-63B : aucune règle de composition ; de 12 à 128 caractères pour
//!   un nouveau, comme les comptes, comptés lettre à lettre (un point de code Unicode chacun) ;
//!   rien n'est coupé en silence : un mot de passe trop long est refusé, avec la raison ;
//! - le collage n'est jamais empêché ; un bouton « Montrer », au doigt, à la souris, au clavier et
//!   au lecteur d'écran (son nom, et son état : enfoncé ou non) ; `autocomplete` posé par le moteur
//!   (`current-password` ou `new-password`), pour que les gestionnaires de mots de passe marchent.

use crate::flat::escape;
use crate::holo::{Block, Error, Program, Value};
use crate::rules::{for_each_block, name_of};

/// Un nouveau mot de passe : 12 caractères au moins, comme ceux des comptes (`ADR-081`). Une seule
/// règle pour tout le moteur : les pages de compte la lisent ici.
pub const MIN: usize = 12;
/// Un mot de passe, au plus : bien plus que les 64 caractères que NIST demande d'accepter, et une
/// borne contre un envoi géant (le serveur calcule une empreinte Argon2id, lente exprès).
pub const MAX: usize = 128;
/// Le nom du champ dans les messages d'erreur (`holo-password|…`), et le début de son nom dans
/// le formulaire des gestes, sans JavaScript (`holo-password-Login`). Le tiret le distingue de
/// toute valeur de la page : un nom de valeur n'en a jamais (ADR-037).
pub const FIELD: &str = "holo-password";
/// La clé du mot de passe dans un envoi en JSON : à côté des valeurs, jamais parmi elles,
/// `{"form":"Login","values":{…},"password":"…"}`.
pub const KEY: &str = "password";

/// Les paramètres d'un champ mot de passe. Ni `value` (la page ne le lit pas), ni `min`, `max`
/// (les longueurs sont celles du moteur), ni `required` (il l'est toujours), ni `suggestions`.
const PARAMS: &[&str] = &["name", "type", "label", "new"];

/// Ce que le visiteur donne : le mot de passe de son compte, ou un mot de passe qu'il choisit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Le mot de passe du compte du visiteur, vérifié par `holo serve` (`autocomplete="current-password"`).
    Current,
    /// Un nouveau mot de passe : l'auteur n'en reçoit que l'empreinte (`autocomplete="new-password"`).
    New,
}

impl Kind {
    /// Ce que le navigateur et le gestionnaire de mots de passe doivent proposer.
    pub fn autocomplete(self) -> &'static str {
        match self {
            Kind::Current => "current-password",
            Kind::New => "new-password",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Kind::Current => "current",
            Kind::New => "new",
        }
    }
}

/// Le bloc est-il un champ mot de passe, et lequel ?
pub fn kind(block: &Block) -> Option<Kind> {
    if block.name != "Input" || !matches!(block.argument("type").map(|a| &a.value), Some(Value::Name(t)) if t == "password") {
        return None;
    }
    Some(if matches!(block.argument("new").map(|a| &a.value), Some(Value::Bool(true))) { Kind::New } else { Kind::Current })
}

pub fn is_password(block: &Block) -> bool {
    kind(block).is_some()
}

/// Le mot de passe d'un formulaire, s'il en a un : le champ, et ce qu'il demande.
pub fn of_form(form: &Block) -> Option<(&Block, Kind)> {
    let mut found = None;
    let _ = for_each_block(form, &mut |block| {
        if found.is_none() {
            found = kind(block).map(|kind| (block, kind));
        }
        Ok(())
    });
    found
}

/// Le mot de passe du formulaire qui porte ce nom.
pub fn of_named_form<'a>(program: &'a Program, form_name: &str) -> Option<(&'a Block, Kind)> {
    crate::rules::named_block(program, form_name).filter(|b| b.name == "Form").and_then(of_form)
}

/// L'identifiant du champ d'un formulaire : un seul mot de passe par formulaire.
fn field_id(form_name: &str) -> String {
    format!("{FIELD}-{form_name}")
}

/// Un champ ordinaire qui demande un mot de passe : son étiquette ou le nom de sa valeur le
/// disent. Il ferait entrer le mot de passe dans les valeurs de la page (l'écran, l'adresse, la
/// base, en clair) : le moteur le refuse et montre la bonne écriture. Seulement des mots sans
/// ambiguïté : « Passeport » ou « Code promo » restent des champs ordinaires.
fn asks_for_a_password(block: &Block) -> bool {
    if block.name != "Input" || block.argument("type").is_some() {
        return false;
    }
    let label = match block.argument("label").map(|a| &a.value) {
        Some(Value::Text(label)) => label.to_lowercase(),
        _ => String::new(),
    };
    let words: Vec<&str> = label.split(|c: char| !c.is_alphanumeric()).collect();
    let in_label = ["mot de passe", "mots de passe", "password", "passphrase", "phrase de passe", "passcode"].iter().any(|said| label.contains(said)) || words.contains(&"mdp");
    let value = match block.argument("value").map(|a| &a.value) {
        Some(Value::Name(value)) => value.to_lowercase(),
        _ => String::new(),
    };
    let in_value = ["password", "motdepasse", "passphrase", "passcode"].iter().any(|said| value.contains(said)) || ["mdp", "pwd", "passwd"].contains(&value.as_str());
    in_label || in_value
}

/// Vérifie les champs mot de passe d'un fichier. Chaque refus dit pourquoi, et la bonne écriture.
pub fn check(program: &Program) -> Result<(), Error> {
    let example = "Input(type: password, label: \"Ton mot de passe\")";
    // Les formulaires, et la liste répétée (`Repeat`) où se trouve chaque champ.
    let mut forms: Vec<&Block> = Vec::new();
    let mut repeated: Vec<&Block> = Vec::new();
    for_each_block(&program.root, &mut |block| {
        match block.name.as_str() {
            "Form" => forms.push(block),
            "Repeat" => repeated.push(block),
            _ => {}
        }
        Ok(())
    })?;
    let inside = |outer: &Block, block: &Block| {
        let mut found = false;
        let _ = for_each_block(outer, &mut |b| {
            found |= std::ptr::eq(b, block);
            Ok(())
        });
        found
    };
    for_each_block(&program.root, &mut |block| {
        if asks_for_a_password(block) {
            let value = match block.argument("value").map(|a| &a.value) {
                Some(Value::Name(value)) => format!("value: {value}"),
                _ => "…".into(),
            };
            return Err(Error { message: format!("« Input({value}) » demande un mot de passe dans un champ ordinaire : il entrerait dans les valeurs de la page, se verrait à l'écran, et partirait en clair. Écris {example} dans un formulaire : la page ne le lit jamais, et le serveur n'en garde rien en clair (ADR-114)"), pos: block.pos });
        }
        if let Some(argument) = block.argument("new").filter(|_| !is_password(block)) {
            return Err(Error { message: "« new: true » ne sert qu'à un mot de passe que le visiteur choisit : Input(type: password, new: true, label: \"Choisis un mot de passe\")".into(), pos: argument.pos });
        }
        let Some(kind) = kind(block) else { return Ok(()) };
        for argument in &block.arguments {
            match (argument.name.as_deref(), &argument.value) {
                (Some("name" | "type"), _) | (Some("label"), Value::Text(_)) | (Some("new"), Value::Bool(_)) => {}
                (Some("new"), _) => return Err(Error { message: "« Input(new: …) » attend true ou false : new: true pour un mot de passe que le visiteur choisit".into(), pos: argument.pos }),
                (Some("value"), _) => {
                    return Err(Error { message: "« Input(type: password, value: …) » : un mot de passe n'entre jamais dans les valeurs de la page ; ni une règle, ni un texte, ni l'état ne le lisent. Il part seulement avec son formulaire, vers le serveur : retire value:".into(), pos: argument.pos })
                }
                (Some(word @ ("min" | "max")), _) => {
                    return Err(Error { message: format!("« Input(type: password, {word}: …) » : les longueurs d'un mot de passe sont celles du moteur, sans règle de composition (NIST SP 800-63B) : de {MIN} à {MAX} caractères pour un nouveau ; retire {word}:"), pos: argument.pos })
                }
                (Some("required"), _) => return Err(Error { message: "« Input(type: password, required: …) » : un mot de passe est toujours demandé ; retire required:".into(), pos: argument.pos }),
                (Some("suggestions"), _) => return Err(Error { message: "« Input(type: password, suggestions: …) » : aucune suggestion pour un mot de passe ; le gestionnaire de mots de passe du visiteur propose les siennes".into(), pos: argument.pos }),
                // Sa place dans une ligne, une grille, ou en haut de l'écran : vérifiée ailleurs.
                (Some("grow" | "sticky"), _) => {}
                (Some(word), _) if crate::grid::CELL_PARAMS.contains(&word) => {}
                (Some(word), _) => return Err(Error { message: format!("« Input(type: password) » n'a pas de paramètre « {word} » ; paramètres possibles : {}", PARAMS.join(", ")), pos: argument.pos }),
                (None, _) => return Err(Error { message: format!("chaque paramètre de « Input » est nommé : {example}"), pos: argument.pos }),
            }
        }
        if !matches!(block.argument("label").map(|a| &a.value), Some(Value::Text(label)) if !label.trim().is_empty()) {
            return Err(Error { message: format!("« Input(type: password) » attend « label » : ce que le lecteur d'écran dit, et ce que le gestionnaire de mots de passe lit ; {example}"), pos: block.pos });
        }
        let Some(form) = forms.iter().copied().find(|&form| inside(form, block)) else {
            return Err(Error { message: "« Input(type: password) » part avec un formulaire, jamais seul : mets-le dans Form(name: Login, children: [ … ]), envoyé par une règle, On(Send.tap, effect: Login.send)".into(), pos: block.pos });
        };
        if repeated.iter().copied().any(|list| inside(list, block)) {
            return Err(Error { message: "un mot de passe ne se répète pas dans une liste (Repeat) : un formulaire, un mot de passe".into(), pos: block.pos });
        }
        let mut count = 0;
        let _ = for_each_block(form, &mut |b| {
            count += usize::from(is_password(b));
            Ok(())
        });
        if count > 1 {
            return Err(Error { message: "un formulaire a un seul mot de passe : pour relire ce qu'il a tapé, le visiteur touche « Montrer » ; il ne le tape pas deux fois".into(), pos: block.pos });
        }
        // Le mot de passe du compte : seul un membre connecté en a un, que le serveur vérifie. Un
        // morceau importé (`Component`) n'a pas de page : celle qui l'importe le dit.
        if kind == Kind::Current && program.root.name == "Page" && !crate::account::members_only(program) {
            return Err(Error { message: "« Input(type: password) » demande le mot de passe du compte du visiteur, que le serveur vérifie : la page est réservée aux membres, Page(access: members). Pour un mot de passe que le visiteur choisit, écris new: true".into(), pos: block.pos });
        }
        Ok(())
    })
}

/// La langue des messages : celle de la page, le français sinon.
fn english(program: &Program) -> bool {
    matches!(program.root.argument("lang").map(|a| &a.value), Some(Value::Text(l)) if l.starts_with("en"))
}

/// Ce qui ne va pas dans un mot de passe, d'après sa longueur seulement (le moteur ne voit jamais
/// le mot de passe lui-même) : vide, trop court pour un nouveau, trop long. Les messages ne disent
/// jamais ce qui a été tapé. `secure` : la page est-elle en HTTPS, ou sur ce PC ?
pub fn errors(program: &Program, form_name: &str, length: usize, secure: bool) -> Vec<(String, String)> {
    refusal(program, form_name, length, secure).map(|code| (FIELD.to_string(), message(program, form_name, code))).into_iter().collect()
}

/// Le code du refus d'un mot de passe, d'après sa longueur ; `None` : il peut partir, ou le
/// formulaire n'en a pas.
pub fn refusal(program: &Program, form_name: &str, length: usize, secure: bool) -> Option<&'static str> {
    let (_, kind) = of_named_form(program, form_name)?;
    if !secure {
        Some("insecure")
    } else if length == 0 {
        Some("empty")
    } else if kind == Kind::New && length < MIN {
        Some("short")
    } else if length > MAX {
        Some("long")
    } else {
        None
    }
}

/// Le message d'un refus, dans la langue de la page, d'après son code : `empty`, `short`, `long`,
/// `insecure`, `wrong` (ce n'est pas le mot de passe du compte), `wait-3` (le frein : attendre
/// trois minutes), `member` (le visiteur n'est plus connecté), `busy` (le frein par adresse),
/// `unavailable` (le serveur ne peut pas calculer l'empreinte). Le serveur garde le code d'un
/// envoi refusé sans JavaScript, jamais le mot de passe, pour l'écrire sous le champ.
pub fn message(program: &Program, form_name: &str, code: &str) -> String {
    let kind = of_named_form(program, form_name).map_or(Kind::Current, |(_, kind)| kind);
    let en = english(program);
    let say = |fr: String, en_text: String| if en { en_text } else { fr };
    if let Some(minutes) = code.strip_prefix("wait-").and_then(|m| m.parse::<u64>().ok()) {
        let plural = if minutes > 1 { "s" } else { "" };
        return say(format!("Trop d'essais : attends {minutes} minute{plural} avant de réessayer."), format!("Too many tries: wait {minutes} minute{plural} before trying again."));
    }
    match (code, kind) {
        ("empty", Kind::Current) => say("Écris ton mot de passe.".into(), "Enter your password.".into()),
        ("empty", Kind::New) => say("Choisis un mot de passe.".into(), "Choose a password.".into()),
        ("short", _) => say(format!("Au moins {MIN} caractères."), format!("At least {MIN} characters.")),
        ("long", _) => say(format!("Au plus {MAX} caractères."), format!("At most {MAX} characters.")),
        ("insecure", _) => say("Ce mot de passe ne part pas : la page n'est pas en HTTPS.".into(), "This password is not sent: the page is not served over HTTPS.".into()),
        ("wrong", _) => say("Ce n'est pas le mot de passe de ton compte.".into(), "This is not the password of your account.".into()),
        ("member", _) => say("Connecte-toi d'abord : ce mot de passe est celui de ton compte.".into(), "Sign in first: this is the password of your account.".into()),
        ("busy", _) => say("Trop de demandes depuis cette adresse : attends une minute.".into(), "Too many requests from this address: wait a minute.".into()),
        ("unavailable", _) => say("Le serveur ne peut pas recevoir ce mot de passe pour l'instant : réessaie plus tard.".into(), "The server cannot take this password right now: try again later.".into()),
        _ => say("Ce mot de passe ne va pas.".into(), "This password does not work.".into()),
    }
}

/// Les codes de refus que le serveur peut garder (`Login:wrong`) : rien d'autre ne s'écrit.
pub fn is_code(code: &str) -> bool {
    matches!(code, "empty" | "short" | "long" | "insecure" | "wrong" | "member" | "busy" | "unavailable") || code.strip_prefix("wait-").is_some_and(|m| !m.is_empty() && m.len() <= 3 && m.bytes().all(|b| b.is_ascii_digit()))
}

/// Le champ, tel que la page le montre. Sans `name` ni `data-bind` : aucun formulaire ordinaire ne
/// l'envoie, et le moteur du navigateur ne le range dans aucune valeur. Sans `maxlength` : un mot
/// de passe collé n'est jamais coupé en silence (le moteur dit qu'il est trop long). Le bouton
/// « Montrer » attend la page légère, qui le montre (sans JavaScript, il ne pourrait rien faire) ;
/// la note dit pourquoi le champ est fermé, hors HTTPS.
pub fn html(block: &Block, classes: &str, form_name: &str, french: bool) -> String {
    let kind = kind(block).unwrap_or(Kind::Current);
    let label = match block.argument("label").map(|a| &a.value) {
        Some(Value::Text(label)) => label.as_str(),
        _ => "",
    };
    let name = name_of(block).map(|n| format!(" data-name=\"{}\"", escape(n))).unwrap_or_default();
    let id = field_id(form_name);
    let (show, show_said, note) = if french {
        ("Montrer", "Montrer le mot de passe", "Ce champ ne s'ouvre qu'en HTTPS : un mot de passe ne part jamais en clair.")
    } else {
        ("Show", "Show password", "This field only opens over HTTPS: a password never travels in clear.")
    };
    let (hint, described) = match kind {
        Kind::New => {
            let said = if french { format!("{MIN} caractères au moins : une phrase que toi seul connais est un bon mot de passe.") } else { format!("At least {MIN} characters: a sentence only you know makes a good password.") };
            (format!("<p class=\"holo-password-hint\" id=\"{id}-hint\">{}</p>", escape(&said)), format!(" aria-describedby=\"{id}-hint\" data-hint=\"{id}-hint\""))
        }
        Kind::Current => (String::new(), String::new()),
    };
    format!(
        "<div class=\"{classes} holo-password\"{name}><label><span>{}</span><input type=\"password\" id=\"{id}\" class=\"holo-secret\" data-secret=\"{}\" data-form=\"{}\" autocomplete=\"{}\" aria-required=\"true\"{described}></label>\
<button type=\"button\" class=\"holo-reveal\" aria-pressed=\"false\" aria-controls=\"{id}\" aria-label=\"{show_said}\" hidden>{show}</button>{hint}\
<p class=\"holo-password-note\" id=\"{id}-note\" hidden>{}</p></div>",
        escape(label),
        kind.word(),
        escape(form_name),
        kind.autocomplete(),
        escape(note)
    )
}

/// Ferme les champs mot de passe d'une page qui n'est pas en HTTPS (ni sur ce PC) : `disabled`
/// (le navigateur ne les envoie plus, on n'y écrit plus), retirés du formulaire des gestes, et la
/// note dit pourquoi. C'est ce que fait la page légère quand le navigateur dit que la page n'est
/// pas sûre ; `holo serve` le fait pour un visiteur sans JavaScript.
pub fn closed(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find("<input ") {
        output.push_str(&rest[..start]);
        let end = rest[start..].find('>').map_or(rest.len(), |e| start + e + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        if !tag.contains(" data-secret=\"") {
            output.push_str(tag);
            continue;
        }
        let id = attribute(tag, "id").unwrap_or_default();
        // Ni dans le formulaire des gestes, ni nommé : le navigateur ne l'envoie plus. Seule la note
        // le décrit.
        let mut kept = tag.replace(&format!(" form=\"{}\"", crate::gestures::FORM), "");
        for name in ["name", "aria-describedby"] {
            if let Some(value) = attribute(&kept, name) {
                kept = kept.replace(&format!(" {name}=\"{value}\""), "");
            }
        }
        output.push_str(&kept.replacen("<input ", &format!("<input disabled aria-describedby=\"{id}-note\" "), 1));
        // Son bouton « Montrer » ne sert plus ; sa note se montre.
        let note = format!("<p class=\"holo-password-note\" id=\"{id}-note\" hidden>");
        if let Some(at) = rest.find(&note) {
            let reveal = format!(" aria-controls=\"{id}\"");
            output.push_str(&rest[..at].replacen(&reveal, &format!("{reveal} disabled"), 1));
            output.push_str(&note.replace(" hidden>", ">"));
            rest = &rest[at + note.len()..];
        }
    }
    output.push_str(rest);
    output
}

/// La valeur d'un attribut dans une balise.
fn attribute(tag: &str, name: &str) -> Option<String> {
    let marker = format!(" {name}=\"");
    let start = tag.find(&marker)? + marker.len();
    let length = tag[start..].find('"')?;
    Some(tag[start..start + length].to_string())
}

#[cfg(test)]
mod password_tests {
    use super::*;

    /// Une page réservée aux membres, avec le mot de passe du compte ; et une page publique avec un
    /// nouveau mot de passe.
    fn members(children: &str) -> String {
        format!("Page(title: \"Essai\", access: members, state: State(note: \"\", done: 0), children: [ {children} ], rules: [ On(Send.tap, effect: Login.send) ])")
    }
    fn public(children: &str) -> String {
        format!("Page(title: \"Essai\", state: State(note: \"\", done: 0), children: [ {children} ], rules: [ On(Send.tap, effect: Login.send) ])")
    }
    const FORM_CURRENT: &str = "Form(name: Login, children: [ Input(type: password, label: \"Ton mot de passe\"), Button(name: Send, text: \"Confirmer\") ])";
    const FORM_NEW: &str = "Form(name: Login, children: [ Input(value: note, label: \"Le nom du carnet\"), Input(type: password, new: true, label: \"Un mot de passe pour ce carnet\"), Button(name: Send, text: \"Protéger\") ])";

    fn refused(source: &str) -> String {
        match crate::check_page(source) {
            Ok(_) => panic!("accepté, alors qu'il devait être refusé :\n{source}"),
            Err(error) => error.message,
        }
    }

    #[test]
    fn a_password_field_is_never_a_value_of_the_page() {
        // Règle 1 : la page ne lit jamais ce qui est tapé. Le champ n'a pas de valeur : aucun nom
        // par lequel une règle, un texte, `keep`, `visit`, l'adresse ou `Shared` l'atteindraient.
        let source = public(FORM_NEW);
        crate::check_page(&source).unwrap();
        let with_value = source.replace("Input(type: password, new: true,", "Input(type: password, new: true, value: note,");
        assert!(refused(&with_value).contains("un mot de passe n'entre jamais dans les valeurs de la page"), "{}", refused(&with_value));
        // Aucune valeur ne s'appelle comme lui : chaque façon de le lire est refusée, faute de nom.
        for (reader, said) in [
            ("P(\"Tu as tapé {password}\"),", "password"),
            ("P(\"Tu as tapé {password:length} caractères\"),", "password"),
            ("If(password, is: \"x\", children: [ P(\"x\") ]),", "password"),
        ] {
            let read = source.replace("children: [ Form(", &format!("children: [ {reader} Form("));
            let message = refused(&read);
            assert!(message.contains(said), "{reader} : {message}");
        }
        for setting in ["keep: [password]", "visit: [password]", "address: [password]"] {
            let kept = source.replace("state: State(", &format!("{setting}, state: State("));
            assert!(crate::check_page(&kept).is_err(), "{setting} accepté");
        }
        let rule = source.replace("On(Send.tap, effect: Login.send)", "On(Send.tap, effect: [Login.send, password.set(\"x\")])");
        assert!(crate::check_page(&rule).is_err(), "une règle qui écrit « password » est acceptée");
        let shared = source.replace("Input(type: password, new: true,", "Input(type: password, new: true, value: likes,").replace("state: State(", "shared: Shared(likes: \"\"), state: State(");
        assert!(refused(&shared).contains("un mot de passe n'entre jamais dans les valeurs de la page"));
        // Fabriqué : ni `data-bind`, ni `name`, ni `value`, ni `maxlength` (rien n'est coupé en silence),
        // aucun empêchement du collage ; `autocomplete` posé par le moteur.
        let html = crate::flat_view(&source, "").unwrap();
        let field = html.split("<input type=\"password\"").nth(1).unwrap().split('>').next().unwrap();
        assert!(field.contains("data-secret=\"new\"") && field.contains("autocomplete=\"new-password\"") && field.contains("aria-required=\"true\""), "{field}");
        for absent in ["data-bind", " name=", " value=", "maxlength", "minlength", "onpaste", "oncopy", "readonly", "disabled"] {
            assert!(!field.contains(absent), "{absent} : {field}");
        }
        assert!(!html.contains("onpaste") && !html.contains("paste"), "{html}");
        // L'état de départ, l'envoi fabriqué par le moteur, l'adresse : rien du mot de passe.
        let start = crate::initial_state(&source);
        assert!(!start.contains("password") && !start.contains("holo-"), "{start}");
        let typed = crate::input(&source, &start, "note", "Mon carnet");
        for name in [FIELD.to_string(), format!("{FIELD}-Login"), "password".to_string()] {
            let after = crate::input(&source, &typed, &name, "secret phrase très longue");
            assert!(after == typed && !after.contains("secret"), "« {name} » : {after}");
        }
        assert_eq!(crate::submission(&source, &typed, "Login"), r#"{"form":"Login","values":{"note":"Mon carnet"}}"#);
        // Sans JavaScript, le champ envoyé avec un toucher ne change rien à l'état.
        let fields = vec![(format!("{FIELD}-Login"), "secret phrase très longue".to_string()), ("note".to_string(), "Mon carnet".to_string()), ("signal".to_string(), "Send.tap".to_string())];
        let after = crate::visitor_gesture(&source, &start, &fields);
        assert!(!after.contains("secret") && !after.contains("tr%C3%A8s"), "{after}");
        assert_eq!(crate::from_query(&source, &start, "password=secret&holo-password-Login=secret"), start);
    }

    #[test]
    fn a_password_field_is_read_strictly() {
        crate::check_page(&members(FORM_CURRENT)).unwrap();
        crate::check_page(&public(FORM_NEW)).unwrap();
        // Le mot de passe du compte demande une page réservée aux membres.
        assert!(refused(&public(FORM_CURRENT)).contains("Page(access: members)"));
        // Hors d'un formulaire ; deux dans un formulaire ; dans une liste répétée.
        assert!(refused(&members("Input(type: password, label: \"Mot\"), Form(name: Login, children: [ Button(name: Send, text: \"Ok\") ])")).contains("part avec un formulaire"));
        let twice = FORM_NEW.replace("Button(", "Input(type: password, new: true, label: \"Encore\"), Button(");
        assert!(refused(&public(&twice)).contains("un seul mot de passe"));
        let listed = "Form(name: Login, children: [ Repeat(over: notes, children: [ Input(type: password, new: true, label: \"Mot\") ]), Button(name: Send, text: \"Ok\") ])";
        let message = refused(&public(listed).replace("state: State(", "state: State(notes: [\"a\"], "));
        assert!(message.contains("ne se répète pas dans une liste") || message.contains("Repeat"), "{message}");
        // Les paramètres : chacun refusé avec sa raison.
        for (parameter, said) in [
            ("min: 4", "les longueurs d'un mot de passe sont celles du moteur"),
            ("max: 20", "les longueurs d'un mot de passe sont celles du moteur"),
            ("required: true", "toujours demandé"),
            ("suggestions: [\"a\", \"b\"]", "aucune suggestion"),
            ("lines: 3", "n'a pas de paramètre « lines »"),
            ("accept: [\"image\"]", "n'a pas de paramètre « accept »"),
            ("new: 1", "attend true ou false"),
        ] {
            let source = public(&FORM_NEW.replace("new: true,", &format!("new: true, {parameter},")).replace("new: true, new: 1,", "new: 1,"));
            let message = refused(&source);
            assert!(message.contains(said), "{parameter} : {message}");
        }
        assert!(refused(&public(&FORM_NEW.replace(", label: \"Un mot de passe pour ce carnet\"", ""))).contains("attend « label »"));
        assert!(refused(&public(&FORM_NEW.replace("label: \"Un mot de passe pour ce carnet\"", "label: \"  \""))).contains("attend « label »"));
        // `new:` ne sert qu'à un mot de passe.
        assert!(refused(&public(&FORM_NEW.replace("Input(value: note, label: \"Le nom du carnet\")", "Input(value: note, new: true, label: \"Le nom du carnet\")"))).contains("ne sert qu'à un mot de passe"));
        // Un morceau importé : la page qui l'importe dit si elle est réservée.
        crate::check_page(&format!("Component(name: Confirm, children: [ {FORM_CURRENT} ])")).unwrap();
    }

    #[test]
    fn a_plain_field_that_asks_for_a_password_is_refused() {
        // Le défaut du web : `<input type="text" name="password">`. Un champ ordinaire qui demande un
        // mot de passe mettrait ce mot de passe dans les valeurs de la page, à l'écran, en clair.
        for field in [
            "Input(value: note, label: \"Mot de passe\")",
            "Input(value: note, label: \"Ton MOT DE PASSE, s'il te plaît\")",
            "Input(value: note, label: \"Password\")",
            "Input(value: note, label: \"Ton mdp\")",
            "Input(value: motDePasse, label: \"Secret\")",
            "Input(value: password, label: \"Secret\")",
            "Input(value: pwd, label: \"Secret\")",
        ] {
            let source = public(&format!("Form(name: Login, children: [ {field}, Button(name: Send, text: \"Ok\") ])")).replace("State(note: \"\"", "State(note: \"\", motDePasse: \"\", password: \"\", pwd: \"\"");
            let message = refused(&source);
            assert!(message.contains("demande un mot de passe dans un champ ordinaire") && message.contains("Input(type: password"), "{field} : {message}");
        }
        // Des mots qui ressemblent, sans demander un mot de passe : acceptés.
        for field in ["Input(value: note, label: \"Numéro de passeport\")", "Input(value: note, label: \"Code promo\")", "Input(value: passport, label: \"Passe-temps préféré\")", "Input(value: note, label: \"Le mot de la fin\")"] {
            let source = public(&format!("Form(name: Login, children: [ {field}, Button(name: Send, text: \"Ok\") ])")).replace("State(note: \"\"", "State(note: \"\", passport: \"\"");
            crate::check_page(&source).unwrap_or_else(|e| panic!("{field} : {}", e.message));
        }
    }

    #[test]
    fn the_lengths_follow_nist_without_composition_rules() {
        // Règle 3 : aucune règle de composition ; au moins 64 caractères permis ; comptés lettre à
        // lettre (un point de code Unicode chacun, comme le demande NIST SP 800-63B).
        let new = crate::check_page(&public(FORM_NEW)).unwrap();
        let said = |length: usize| errors(&new, "Login", length, true);
        assert_eq!(said(0), vec![(FIELD.to_string(), "Choisis un mot de passe.".to_string())]);
        assert_eq!(said(MIN - 1), vec![(FIELD.to_string(), "Au moins 12 caractères.".to_string())]);
        for allowed in [MIN, 64, MAX] {
            assert!(said(allowed).is_empty(), "{allowed} caractères refusés");
        }
        assert_eq!(said(MAX + 1), vec![(FIELD.to_string(), "Au plus 128 caractères.".to_string())]);
        assert_eq!(errors(&new, "Login", 20, false), vec![(FIELD.to_string(), "Ce mot de passe ne part pas : la page n'est pas en HTTPS.".to_string())]);
        assert!(errors(&new, "Other", 0, true).is_empty());
        // Le mot de passe du compte : aucun minimum à la vérification (c'est le compte qui l'a fixé).
        let current = crate::check_page(&members(FORM_CURRENT)).unwrap();
        assert!(errors(&current, "Login", 1, true).is_empty());
        assert_eq!(errors(&current, "Login", 0, true)[0].1, "Écris ton mot de passe.");
        assert_eq!(errors(&current, "Login", MAX + 1, true)[0].1, "Au plus 128 caractères.");
        // Douze lettres accentuées, ou douze fois la même : un mot de passe permis.
        assert_eq!("éèêëàâäôöûüç".chars().count(), 12);
        assert!(said("éèêëàâäôöûüç".chars().count()).is_empty() && said("aaaaaaaaaaaa".chars().count()).is_empty());
        // En anglais, avec la langue de la page ; et les codes que le serveur garde.
        let english = crate::check_page(&public(FORM_NEW).replace("title: \"Essai\",", "title: \"Essai\", lang: \"en\",")).unwrap();
        assert_eq!(errors(&english, "Login", 3, true)[0].1, "At least 12 characters.");
        assert_eq!(message(&current, "Login", "wrong"), "Ce n'est pas le mot de passe de ton compte.");
        assert_eq!(message(&current, "Login", "wait-2"), "Trop d'essais : attends 2 minutes avant de réessayer.");
        assert_eq!(message(&english, "Login", "wait-1"), "Too many tries: wait 1 minute before trying again.");
        for code in ["empty", "short", "long", "insecure", "wrong", "member", "busy", "unavailable", "wait-60"] {
            assert!(is_code(code), "{code}");
        }
        for code in ["", "wait-", "wait-x", "wait-1234", "secret", "wrong,"] {
            assert!(!is_code(code), "{code}");
        }
    }

    #[test]
    fn the_field_helps_password_managers_keyboards_and_screen_readers() {
        // Règles 4 et 5 : `autocomplete` juste ; le bouton « Montrer » nommé, avec son état, relié au
        // champ, montré par la page légère ; l'explication d'un nouveau mot de passe reliée au champ.
        let current = crate::flat_view(&members(FORM_CURRENT), "").unwrap();
        assert!(current.contains("<div class=\"holo-Input holo-password\"><label><span>Ton mot de passe</span><input type=\"password\" id=\"holo-password-Login\" class=\"holo-secret\" data-secret=\"current\" data-form=\"Login\" autocomplete=\"current-password\" aria-required=\"true\"></label><button type=\"button\" class=\"holo-reveal\" aria-pressed=\"false\" aria-controls=\"holo-password-Login\" aria-label=\"Montrer le mot de passe\" hidden>Montrer</button><p class=\"holo-password-note\" id=\"holo-password-Login-note\" hidden>"), "{current}");
        let new = crate::flat_view(&public(FORM_NEW), "").unwrap();
        assert!(new.contains("autocomplete=\"new-password\" aria-required=\"true\" aria-describedby=\"holo-password-Login-hint\" data-hint=\"holo-password-Login-hint\">"), "{new}");
        assert!(new.contains("<p class=\"holo-password-hint\" id=\"holo-password-Login-hint\">12 caractères au moins : une phrase que toi seul connais est un bon mot de passe.</p>"), "{new}");
        let english = crate::flat_view(&public(FORM_NEW).replace("title: \"Essai\",", "title: \"Essai\", lang: \"en\","), "").unwrap();
        assert!(english.contains("aria-label=\"Show password\" hidden>Show</button>") && english.contains("At least 12 characters"), "{english}");
        // Un nom de bloc : pour les styles.
        let named = crate::flat_view(&members(&FORM_CURRENT.replace("Input(type: password,", "Input(name: Secret, type: password,")), "").unwrap();
        assert!(named.contains("<div class=\"holo-Input holo-password\" data-name=\"Secret\">"), "{named}");
        // Hors HTTPS : fermé, retiré du formulaire des gestes, et la note dit pourquoi.
        let served = crate::gestures::without_script(&new, &[]);
        assert!(served.contains("<input form=\"holo-gestures\" name=\"holo-password-Login\" type=\"password\" id=\"holo-password-Login\""), "{served}");
        let shut = closed(&served);
        assert!(shut.contains("<input disabled aria-describedby=\"holo-password-Login-note\" type=\"password\" id=\"holo-password-Login\""), "{shut}");
        assert!(!shut.contains("name=\"holo-password-Login\"") && !shut.contains("aria-describedby=\"holo-password-Login-hint\"") && shut.contains("aria-controls=\"holo-password-Login\" disabled"), "{shut}");
        assert!(shut.contains("<p class=\"holo-password-note\" id=\"holo-password-Login-note\">Ce champ ne s'ouvre qu'en HTTPS"), "{shut}");
        assert_eq!(closed(&crate::flat_view(&public("P(\"Rien\"), Form(name: Login, children: [ Button(name: Send, text: \"Ok\") ])"), "").unwrap()), crate::flat_view(&public("P(\"Rien\"), Form(name: Login, children: [ Button(name: Send, text: \"Ok\") ])"), "").unwrap());
    }
}
