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
/// - un bouton de point (qui ouvre un monde), un pixel de la vue points, un fichier à envoyer,
///   et tout ce qui est dans un `Form` restent comme ils sont : ils demandent le moteur.
pub fn without_script(html: &str) -> String {
    let mut output = String::with_capacity(html.len() + 256);
    // Le rang de ligne de chaque `div` ouvert, pour savoir de quelle ligne vient un bouton.
    let mut ranks: Vec<Option<String>> = Vec::new();
    let mut in_form = 0usize;
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        output.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some(end) = rest.find('>') else { break };
        let tag = &rest[..=end];
        rest = &rest[end + 1..];
        if tag.starts_with("<div") {
            ranks.push(attribute(tag, "data-rank").map(str::to_string));
        } else if tag.starts_with("</div") {
            ranks.pop();
        } else if tag.starts_with("<form") && tag.contains("holo-Form") {
            in_form += 1;
        } else if tag.starts_with("</form") {
            in_form = in_form.saturating_sub(1);
        }
        if in_form > 0 {
            output.push_str(tag);
            continue;
        }
        if tag.starts_with("<button type=\"button\"") {
            let class = attribute(tag, "class").unwrap_or("");
            let tappable = class.split(' ').any(|c| c == "holo-Button" || c.starts_with("holo-Shape"));
            match (tappable, attribute(tag, "data-name")) {
                (true, Some(name)) => {
                    let line = ranks.iter().rev().find_map(Option::as_ref).map(|rank| format!("@{rank}")).unwrap_or_default();
                    let sending = format!("<button type=\"submit\" form=\"{FORM}\" name=\"{SIGNAL}\" value=\"{name}.tap{line}\"");
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

/// Un geste envoyé sans JavaScript est-il de ceux qu'une page reçoit ? Seulement un toucher
/// (`Add.tap`), avec sa ligne s'il en a une (`Done.tap@2`). Rien d'autre : ni une horloge, ni
/// un survol, ni un signal inventé.
pub fn is_tap(signal: &str) -> bool {
    let (gesture, line) = signal.split_once('@').unwrap_or((signal, "0"));
    let Some(name) = gesture.strip_suffix(".tap") else { return false };
    !name.is_empty()
        && name.len() <= 64
        && name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && name.chars().all(|c| c.is_ascii_alphanumeric())
        && !line.is_empty()
        && line.len() <= 6
        && line.chars().all(|c| c.is_ascii_digit())
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
        let out = without_script(html);
        assert!(out.contains("<button type=\"submit\" form=\"holo-gestures\" name=\"signal\" value=\"Add.tap\" class=\"holo-Button\" data-name=\"Add\">Ajouter</button>"), "{out}");
        assert!(out.contains("value=\"Done.tap@2\""), "{out}");
        assert!(out.contains("<input form=\"holo-gestures\" name=\"buyer\" type=\"text\" value=\"Ada\" data-bind=\"buyer\">"), "{out}");
        assert!(out.contains("<input type=\"hidden\" form=\"holo-gestures\" name=\"gift\" value=\"0\"><input form=\"holo-gestures\" name=\"gift\" value=\"1\" type=\"checkbox\" data-bind=\"gift\" checked>"), "{out}");
        assert!(out.contains("<input form=\"holo-gestures\" type=\"radio\" name=\"choix-size\""), "{out}");
        // Un point, et ce qui est dans un `Form`, demandent le moteur : rien ne change.
        assert!(out.contains("<button type=\"button\" class=\"holo-Point\" data-name=\"Shop\">"), "{out}");
        assert!(out.contains("<input type=\"text\" value=\"\" data-bind=\"mail\"><button type=\"button\" class=\"holo-Button\" data-name=\"Send\">"), "{out}");
        assert!(out.starts_with("<form id=\"holo-gestures\" method=\"post\" hidden><button type=\"submit\" name=\"signal\" value=\"\" tabindex=\"-1\"></button></form><div class=\"holo-Page\">"), "{out}");
    }

    #[test]
    fn a_form_body_is_read_and_only_taps_pass() {
        assert_eq!(read_form("signal=Add.tap&buyer=Ad%C3%A9le+B&gift=0&gift=1"), vec![
            ("signal".to_string(), "Add.tap".to_string()),
            ("buyer".to_string(), "Adéle B".to_string()),
            ("gift".to_string(), "1".to_string()),
        ]);
        assert_eq!(read_form("a=%ZZ&b=%4"), vec![("a".to_string(), "%ZZ".to_string()), ("b".to_string(), "%4".to_string())]);
        assert!(is_tap("Add.tap") && is_tap("Done.tap@12"));
        for refused in ["Add.hover", "add.tap", ".tap", "Add.tap@", "Add.tap@x", "Shop.portals", "A-b.tap", "Add.tap@1234567"] {
            assert!(!is_tap(refused), "{refused}");
        }
    }
}
