//! Les gestes sans JavaScript (ADR-074). La page fabriquée par `holo serve` pour un visiteur
//! garde exactement son HTML ; seuls les boutons et les champs reçoivent de quoi partir vers le
//! serveur par un formulaire ordinaire, quand le moteur ne tourne pas dans le navigateur.
//!
//! Un seul formulaire caché, `holo-gestures`, en tête de la page : les boutons et les champs s'y
//! rattachent par `form="holo-gestures"`, sans bouger de leur place. La mise en page ne change
//! donc pas. Avec JavaScript, la page légère empêche ce formulaire de partir : le moteur fait le
//! travail dans le navigateur, comme avant.

/// L'identifiant du formulaire auquel se rattachent les gestes.
pub const FORM: &str = "holo-gestures";

/// Le nom du champ qui porte le geste : `signal=Add.tap`, ou `signal=Done.tap@2` dans une liste.
pub const SIGNAL: &str = "signal";

/// Rattache les boutons et les champs de la page au formulaire des gestes.
///
/// - un bouton nommé (`Button`, `Shape`) devient un bouton d'envoi qui porte son geste ; dans la
///   ligne d'une liste, il porte aussi sa ligne (`Done.tap@2`, ADR-044) ;
/// - un champ lié à une valeur (`data-bind`) envoie sa valeur sous ce nom ; une case à cocher
///   décochée envoie `0`, grâce à un champ caché placé juste avant elle ;
/// - un bouton de point (qui ouvre un monde), un pixel de la vue points et un fichier à envoyer
///   restent comme ils sont : ils demandent le moteur.
///
/// Les champs et le bouton d'un `Form` s'y rattachent aussi : « Envoyer » part comme un autre
/// toucher, et le serveur fait l'envoi (ADR-075).
///
/// `keyed` : les listes dont une ligne se désigne par sa clé (une liste partagée, ou calculée
/// d'après elle, ADR-080) ; le bouton d'une de leurs lignes porte aussi la clé, `Done.tap@2#…`.
pub fn without_script(html: &str, keyed: &[String]) -> String {
    let mut output = String::with_capacity(html.len() + 256);
    // Pour chaque `div` ouvert : son rang et sa clé de ligne, la liste qu'il montre, pour savoir
    // de quelle ligne, et de quelle liste, vient un bouton.
    let mut divs: Vec<(Option<String>, Option<String>, Option<String>)> = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        output.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some(end) = rest.find('>') else { break };
        let tag = &rest[..=end];
        rest = &rest[end + 1..];
        if tag.starts_with("<div") {
            let read = |name: &str| attribute(tag, name).map(str::to_string);
            divs.push((read("data-rank"), read("data-key"), read("data-list")));
        } else if tag.starts_with("</div") {
            divs.pop();
        }
        if tag.starts_with("<button type=\"button\"") {
            let class = attribute(tag, "class").unwrap_or("");
            let tappable = class.split(' ').any(|c| c == "holo-Button" || c.starts_with("holo-Shape"));
            match (tappable, attribute(tag, "data-name")) {
                (true, Some(name)) => {
                    let at = divs.iter().rposition(|(rank, _, _)| rank.is_some());
                    let line = at.and_then(|at| divs[at].0.as_ref()).map(|rank| format!("@{rank}")).unwrap_or_default();
                    // La clé, déjà écrite pour le HTML dans `data-key` : elle passe telle quelle.
                    let list = at.and_then(|at| divs[..at].iter().rev().find_map(|(_, _, list)| list.as_ref()));
                    let key = match (at, list) {
                        (Some(at), Some(list)) if keyed.contains(list) => divs[at].1.as_ref().map(|key| format!("#{key}")).unwrap_or_default(),
                        _ => String::new(),
                    };
                    let sending = format!("<button type=\"submit\" form=\"{FORM}\" name=\"{SIGNAL}\" value=\"{name}.tap{line}{key}\"");
                    output.push_str(&tag.replacen("<button type=\"button\"", &sending, 1));
                }
                _ => output.push_str(tag),
            }
            continue;
        }
        let field = tag.starts_with("<input") || tag.starts_with("<textarea") || tag.starts_with("<select");
        match (field, attribute(tag, "data-bind")) {
            (true, Some(bind)) if !tag.contains("type=\"file\"") && bind.chars().all(|c| c.is_ascii_alphanumeric()) => {
                let opening = &tag[..tag.find(' ').unwrap_or(tag.len() - 1)];
                // Un bouton rond garde son nom de groupe (`choix-taille`) : le serveur le relit.
                let named = if attribute(tag, "name").is_some() { String::new() } else { format!(" name=\"{bind}\"") };
                if tag.contains("type=\"checkbox\"") {
                    output.push_str(&format!("<input type=\"hidden\" form=\"{FORM}\" name=\"{bind}\" value=\"0\">"));
                    output.push_str(&tag.replacen(opening, &format!("{opening} form=\"{FORM}\"{named} value=\"1\""), 1));
                } else {
                    output.push_str(&tag.replacen(opening, &format!("{opening} form=\"{FORM}\"{named}"), 1));
                }
            }
            _ => output.push_str(tag),
        }
    }
    output.push_str(rest);
    // Le formulaire vient en tête : son bouton caché est donc le premier. La touche Entrée dans un
    // champ « appuie » sur lui, et non sur le premier bouton de la page : elle envoie les champs,
    // sans toucher à rien d'autre.
    format!("<form id=\"{FORM}\" method=\"post\" hidden><button type=\"submit\" name=\"{SIGNAL}\" value=\"\" tabindex=\"-1\"></button></form>{output}")
}

/// Les messages d'un formulaire envoyé sans JavaScript qui ne va pas (ADR-075), comme le moteur
/// les montre dans le navigateur (ADR-068) : sous chaque champ, relié à lui
/// (`aria-describedby`), le champ marqué (`aria-invalid`). `errors` : `(valeur, message)`.
pub fn with_errors(html: &str, form: &str, errors: &[(String, String)]) -> String {
    let opening = format!(" data-name=\"{form}\" novalidate");
    let Some(start) = html.find(&opening) else { return html.to_string() };
    let end = html[start..].find("</form>").map_or(html.len(), |e| start + e);
    let mut inside = html[start..end].replacen(&opening, &format!(" data-name=\"{form}\" data-tried=\"1\" novalidate"), 1);
    for (bind, message) in errors {
        let id = format!("holo-error-{form}-{bind}");
        let note = format!("<p class=\"holo-error\" id=\"{id}\">{}</p>", message.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;"));
        let marks = format!(" aria-invalid=\"true\" aria-describedby=\"{id}\"");
        // Un groupe de boutons ronds (`fieldset`), ou un champ dans son `label`.
        let (marker, closing) = match inside.find(&format!(" data-group=\"{bind}\"")) {
            Some(_) => (format!(" data-group=\"{bind}\""), "</fieldset>"),
            None => (format!(" data-bind=\"{bind}\""), "</label>"),
        };
        let Some(at) = inside.find(&marker) else { continue };
        inside.insert_str(at, &marks);
        if let Some(after) = inside[at..].find(closing).map(|c| at + c + closing.len()) {
            inside.insert_str(after, &note);
        }
    }
    format!("{}{inside}{}", &html[..start], &html[end..])
}

/// La valeur d'un attribut dans une balise : `data-name="Add"` donne `Add`.
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!(" {name}=\"");
    let start = tag.find(&marker)? + marker.len();
    let length = tag[start..].find('"')?;
    Some(&tag[start..start + length])
}

/// Ce qu'un formulaire ordinaire envoie (`application/x-www-form-urlencoded`), en paires
/// nom et valeur. Un nom répété garde sa dernière valeur : celle de la case cochée, après
/// le champ caché qui dit `0`.
pub fn read_form(body: &str) -> Vec<(String, String)> {
    let mut fields: Vec<(String, String)> = Vec::new();
    for pair in body.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        let (name, value) = (percent_decode(name, true), percent_decode(value, true));
        match fields.iter_mut().find(|(known, _)| *known == name) {
            Some(field) => field.1 = value,
            None => fields.push((name, value)),
        }
    }
    fields
}

/// Défait l'écriture d'un formulaire ou d'une adresse : `%C3%A9` est un « é », et `+` une
/// espace dans un formulaire. Une suite d'octets qui n'est pas de l'UTF-8 est remplacée, jamais
/// acceptée telle quelle.
pub fn percent_decode(text: &str, plus_is_space: bool) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' if plus_is_space => out.push(b' '),
            b'%' if i + 2 < bytes.len() => match (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                (Some(high), Some(low)) => {
                    out.push(high * 16 + low);
                    i += 2;
                }
                _ => out.push(b'%'),
            },
            byte => out.push(byte),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(byte: u8) -> Option<u8> {
    (byte as char).to_digit(16).map(|digit| digit as u8)
}

/// La clé d'une ligne, au plus : celle d'un champ de 200 caractères, avec sa marque et son rang.
pub const LINE_KEY_MAX: usize = 1024;

/// Un geste envoyé sans JavaScript est-il de ceux qu'une page reçoit ? Seulement un toucher
/// (`Add.tap`), avec sa ligne s'il en a une (`Done.tap@2`), et la clé de cette ligne dans une
/// liste partagée (`Done.tap@2#k:Ada-0`, ADR-080). Rien d'autre : ni une horloge, ni un survol,
/// ni un signal inventé.
pub fn is_tap(signal: &str) -> bool {
    let (gesture, line) = signal.split_once('@').unwrap_or((signal, "0"));
    let (line, key) = match line.split_once('#') {
        Some((rank, key)) => (rank, Some(key)),
        None => (line, None),
    };
    let Some(name) = gesture.strip_suffix(".tap") else { return false };
    !name.is_empty()
        && name.len() <= 64
        && name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && name.chars().all(|c| c.is_ascii_alphanumeric())
        && !line.is_empty()
        && line.len() <= 6
        && line.chars().all(|c| c.is_ascii_digit())
        && key.is_none_or(|key| !key.is_empty() && key.len() <= LINE_KEY_MAX && !key.chars().any(char::is_control))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_and_fields_join_the_gesture_form() {
        let html = "<div class=\"holo-Page\"><button type=\"button\" class=\"holo-Button\" data-name=\"Add\">Ajouter</button>\
            <div class=\"holo-Lines\"><div class=\"holo-line\" data-rank=\"2\" data-key=\"k\"><div class=\"holo-Row\"><button type=\"button\" class=\"holo-Button\" data-name=\"Done\">x</button></div></div></div>\
            <label><span>Nom</span><input type=\"text\" value=\"Ada\" data-bind=\"buyer\"></label>\
            <label><input type=\"checkbox\" data-bind=\"gift\" checked><span>Cadeau</span></label>\
            <label><input type=\"radio\" name=\"choix-size\" value=\"S\" data-bind=\"size\"><span>S</span></label>\
            <button type=\"button\" class=\"holo-Point\" data-name=\"Shop\"></button>\
            <form class=\"holo-Form\" data-name=\"Contact\" novalidate><input type=\"text\" value=\"\" data-bind=\"mail\"><button type=\"button\" class=\"holo-Button\" data-name=\"Send\">Envoyer</button></form></div>";
        let out = without_script(html, &[]);
        assert!(out.contains("<button type=\"submit\" form=\"holo-gestures\" name=\"signal\" value=\"Add.tap\" class=\"holo-Button\" data-name=\"Add\">Ajouter</button>"), "{out}");
        assert!(out.contains("value=\"Done.tap@2\""), "{out}");
        // La ligne d'une liste partagée porte aussi sa clé (ADR-080), telle qu'écrite pour le HTML.
        let shared = "<div class=\"holo-Lines\" data-list=\"names\" data-repeat=\"3:5\"><div class=\"holo-line\" data-rank=\"1\" data-key=\"k:Tom &amp; Jerry-0\"><button type=\"button\" class=\"holo-Button\" data-name=\"Remove\">x</button></div></div>";
        assert!(without_script(shared, &["names".to_string()]).contains("value=\"Remove.tap@1#k:Tom &amp; Jerry-0\""), "{}", without_script(shared, &["names".to_string()]));
        assert!(without_script(shared, &["others".to_string()]).contains("value=\"Remove.tap@1\""));
        assert!(out.contains("<input form=\"holo-gestures\" name=\"buyer\" type=\"text\" value=\"Ada\" data-bind=\"buyer\">"), "{out}");
        assert!(out.contains("<input type=\"hidden\" form=\"holo-gestures\" name=\"gift\" value=\"0\"><input form=\"holo-gestures\" name=\"gift\" value=\"1\" type=\"checkbox\" data-bind=\"gift\" checked>"), "{out}");
        assert!(out.contains("<input form=\"holo-gestures\" type=\"radio\" name=\"choix-size\""), "{out}");
        // Un point demande le moteur : rien ne change. Un `Form` part comme le reste (ADR-075).
        assert!(out.contains("<button type=\"button\" class=\"holo-Point\" data-name=\"Shop\">"), "{out}");
        assert!(out.contains("<input form=\"holo-gestures\" name=\"mail\" type=\"text\" value=\"\" data-bind=\"mail\"><button type=\"submit\" form=\"holo-gestures\" name=\"signal\" value=\"Send.tap\""), "{out}");
        assert!(out.starts_with("<form id=\"holo-gestures\" method=\"post\" hidden><button type=\"submit\" name=\"signal\" value=\"\" tabindex=\"-1\"></button></form><div class=\"holo-Page\">"), "{out}");
    }

    #[test]
    fn errors_are_written_under_their_fields() {
        let html = "<form class=\"holo-Form\" data-name=\"Contact\" novalidate><label class=\"holo-Input\"><span>Nom</span><input type=\"text\" value=\"\" data-bind=\"name\"></label><fieldset class=\"holo-Choice\" data-group=\"size\"><legend>T</legend></fieldset></form><p>après</p>";
        let out = with_errors(html, "Contact", &[("name".into(), "Ce champ est obligatoire.".into()), ("size".into(), "Choisis <une> taille.".into())]);
        assert_eq!(out, "<form class=\"holo-Form\" data-name=\"Contact\" data-tried=\"1\" novalidate><label class=\"holo-Input\"><span>Nom</span><input type=\"text\" value=\"\" aria-invalid=\"true\" aria-describedby=\"holo-error-Contact-name\" data-bind=\"name\"></label><p class=\"holo-error\" id=\"holo-error-Contact-name\">Ce champ est obligatoire.</p><fieldset class=\"holo-Choice\" aria-invalid=\"true\" aria-describedby=\"holo-error-Contact-size\" data-group=\"size\"><legend>T</legend></fieldset><p class=\"holo-error\" id=\"holo-error-Contact-size\">Choisis &lt;une&gt; taille.</p></form><p>après</p>");
        assert_eq!(with_errors(html, "Absent", &[("name".into(), "x".into())]), html);
    }

    #[test]
    fn a_form_body_is_read_and_only_taps_pass() {
        assert_eq!(read_form("signal=Add.tap&buyer=Ad%C3%A9le+B&gift=0&gift=1"), vec![
            ("signal".to_string(), "Add.tap".to_string()),
            ("buyer".to_string(), "Adéle B".to_string()),
            ("gift".to_string(), "1".to_string()),
        ]);
        assert_eq!(read_form("a=%ZZ&b=%4"), vec![("a".to_string(), "%ZZ".to_string()), ("b".to_string(), "%4".to_string())]);
        assert!(is_tap("Add.tap") && is_tap("Done.tap@12") && is_tap("Done.tap@2#k:Ada-0") && is_tap("Done.tap@0#9f3a-1"));
        let long = format!("Done.tap@2#{}", "a".repeat(LINE_KEY_MAX + 1));
        for refused in ["Add.hover", "add.tap", ".tap", "Add.tap@", "Add.tap@x", "Shop.portals", "A-b.tap", "Add.tap@1234567", "Add.tap#k", "Done.tap@2#", "Done.tap@#k", "Done.tap@2#a\nb", long.as_str()] {
            assert!(!is_tap(refused), "{refused}");
        }
    }
}
