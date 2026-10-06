//! Les formats d'affichage d'un nombre (ADR-043) : `{minute:00}`, `{total:number}`,
//! `{price:cents}`, `{weekday:name}`. Le nombre reste un nombre entier dans l'état ; seul ce
//! qu'on lit change. La langue est celle de la page (`Page(lang:)`), le français sinon.

thread_local! {
    /// La langue de la page en cours de lecture, pour les formats écrits à la lecture
    /// (`{item.price:cents}` dans une répétition).
    static LANGUE: std::cell::RefCell<String> = std::cell::RefCell::new("fr".into());
}

pub fn regler_langue(langue: &str) {
    LANGUE.with(|l| *l.borrow_mut() = langue.to_string());
}

pub fn langue() -> String {
    LANGUE.with(|l| l.borrow().clone())
}

/// Les formats connus, pour les messages.
pub const FORMATS: &[&str] = &["00", "number", "cents", "name"];

/// Un format est-il connu ? `00` à `000000` : autant de chiffres au moins.
pub fn est_format(format: &str) -> bool {
    (2..=6).contains(&format.len()) && format.chars().all(|c| c == '0') || matches!(format, "number" | "cents" | "name")
}

const JOURS_FR: [&str; 7] = ["lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche"];
const JOURS_EN: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const MOIS_FR: [&str; 12] = ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"];
const MOIS_EN: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// Le séparateur des milliers et celui des décimales, selon la langue.
fn separateurs(langue: &str) -> (&'static str, &'static str) {
    match langue.split('-').next().unwrap_or("") {
        "en" => (",", "."),
        "de" | "es" | "it" | "pt" | "nl" => (".", ","),
        // Le français, et par défaut : une espace fine insécable, et la virgule.
        _ => ("\u{202F}", ","),
    }
}

/// 1234567 → « 1 234 567 ».
fn grouper(valeur: u64, milliers: &str) -> String {
    let chiffres = valeur.to_string();
    let mut sortie = String::new();
    for (rang, c) in chiffres.chars().enumerate() {
        if rang > 0 && (chiffres.len() - rang) % 3 == 0 {
            sortie.push_str(milliers);
        }
        sortie.push(c);
    }
    sortie
}

/// Écrit `valeur` (la valeur `nom`) selon `format`, dans la langue de la page.
pub fn formater(nom: &str, valeur: u64, format: &str, langue: &str) -> String {
    let (milliers, decimales) = separateurs(langue);
    let anglais = langue.starts_with("en");
    match format {
        "number" => grouper(valeur, milliers),
        "cents" => format!("{}{decimales}{:02}", grouper(valeur / 100, milliers), valeur % 100),
        "name" => {
            let (jours, mois) = if anglais { (JOURS_EN, MOIS_EN) } else { (JOURS_FR, MOIS_FR) };
            let liste: &[&str] = if nom == "weekday" { &jours } else { &mois };
            valeur.checked_sub(1).and_then(|i| liste.get(i as usize)).map_or_else(|| valeur.to_string(), |n| (*n).to_string())
        }
        zeros => format!("{valeur:0largeur$}", largeur = zeros.len()),
    }
}

/// Les valeurs à format d'un texte : `{minute:00}` → (`minute`, `00`).
pub fn formats_dans(texte: &str) -> Vec<(&str, &str)> {
    let mut trouves = Vec::new();
    let mut reste = texte;
    while let Some(debut) = reste.find('{') {
        reste = &reste[debut + 1..];
        let Some(fin) = reste.find('}') else { break };
        if let Some((nom, format)) = reste[..fin].split_once(':') {
            if nom.starts_with(|c: char| c.is_ascii_lowercase()) && nom.chars().all(|c| c.is_ascii_alphanumeric()) && !format.is_empty() && format.chars().all(|c| c.is_ascii_alphanumeric()) {
                trouves.push((nom, format));
            }
        }
        reste = &reste[fin..];
    }
    trouves
}

/// Remplit, dans une page fabriquée, chaque valeur à format par son départ :
/// `<span data-state="minute" data-format="00"></span>` → `…>05</span>`.
pub fn remplir(html: &str, valeurs: &[(String, u64)], langue: &str) -> String {
    const DEBUT: &str = "<span data-state=\"";
    let mut sortie = String::with_capacity(html.len());
    let mut reste = html;
    while let Some(place) = reste.find(DEBUT) {
        sortie.push_str(&reste[..place]);
        let apres = &reste[place + DEBUT.len()..];
        let ouverture = apres.find('>').map(|f| &apres[..f]);
        if let Some((nom, format)) = ouverture.and_then(|o| o.split_once("\" data-format=\"")).map(|(n, f)| (n, f.trim_end_matches('"'))) {
            if let (Some((_, valeur)), true) = (valeurs.iter().find(|(connu, _)| connu == nom), apres[nom.len() + format.len() + 16..].starts_with("></span>")) {
                let entete = &reste[place..place + DEBUT.len() + nom.len() + 15 + format.len() + 2];
                sortie.push_str(entete);
                sortie.push_str(&formater(nom, *valeur, format, langue));
                sortie.push_str("</span>");
                reste = &apres[nom.len() + format.len() + 16 + "></span>".len()..];
                continue;
            }
        }
        sortie.push_str(DEBUT);
        reste = apres;
    }
    sortie.push_str(reste);
    sortie
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_formats() {
        assert_eq!(formater("minute", 5, "00", "fr"), "05");
        assert_eq!(formater("n", 1234567, "number", "fr"), "1\u{202F}234\u{202F}567");
        assert_eq!(formater("n", 1234567, "number", "en"), "1,234,567");
        assert_eq!(formater("total", 123450, "cents", "fr"), "1\u{202F}234,50");
        assert_eq!(formater("total", 7, "cents", "en"), "0.07");
        assert_eq!(formater("weekday", 2, "name", "fr"), "mardi");
        assert_eq!(formater("month", 10, "name", "en"), "October");
        assert_eq!(formater("month", 0, "name", "fr"), "0");
        assert_eq!(formats_dans("il est {hour} h {minute:00}, {total:cents} €"), [("minute", "00"), ("total", "cents")]);
        let html = "<p><span data-state=\"minute\" data-format=\"00\"></span> et <span data-state=\"n\"></span></p>";
        assert_eq!(remplir(html, &[("minute".into(), 7)], "fr"), "<p><span data-state=\"minute\" data-format=\"00\">07</span> et <span data-state=\"n\"></span></p>");
    }
}
