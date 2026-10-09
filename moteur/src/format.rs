//! Les formats d'affichage d'un nombre (ADR-043) : `{minute:00}`, `{total:number}`,
//! `{price:cents}`, `{weekday:name}`. Le nombre reste un nombre entier dans l'état ; seul ce
//! qu'on lit change. La langue est celle de la page (`Page(lang:)`), le français sinon.

thread_local! {
    /// La langue de la page en cours de lecture, pour les formats écrits à la lecture
    /// (`{item.price:cents}` dans une répétition).
    static LANGUAGE: std::cell::RefCell<String> = std::cell::RefCell::new("fr".into());
}

pub fn set_language(language: &str) {
    LANGUAGE.with(|l| *l.borrow_mut() = language.to_string());
}

pub fn language() -> String {
    LANGUAGE.with(|l| l.borrow().clone())
}

thread_local! {
    /// Les valeurs à virgule de la page en cours de fabrication, et leurs chiffres (ADR-066).
    static DECIMALS: std::cell::RefCell<Vec<(String, u32)>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn set_decimals(decimals: Vec<(String, u32)>) {
    DECIMALS.with(|d| *d.borrow_mut() = decimals);
}

/// Les chiffres après la virgule de cette valeur, dans la page en cours ; 0 pour un nombre entier.
pub fn decimal_places(name: &str) -> u32 {
    DECIMALS.with(|d| d.borrow().iter().find(|(known, _)| known == name).map_or(0, |(_, p)| *p))
}

/// Les formats connus, pour les messages.
pub const FORMATS: &[&str] = &["00", "number", "cents", "name", "stopwatch"];

/// Un format est-il connu ? `00` à `000000` : autant de chiffres au moins.
pub fn is_format(format: &str) -> bool {
    (2..=6).contains(&format.len()) && format.chars().all(|c| c == '0') || matches!(format, "number" | "cents" | "name" | "stopwatch")
}

const DAYS_FR: [&str; 7] = ["lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche"];
const DAYS_EN: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const MONTHS_FR: [&str; 12] = ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"];
const MONTHS_EN: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// Le séparateur des milliers et celui des décimales, selon la langue.
fn separators(language: &str) -> (&'static str, &'static str) {
    match language.split('-').next().unwrap_or("") {
        "en" => (",", "."),
        "de" | "es" | "it" | "pt" | "nl" => (".", ","),
        // Le français, et par défaut : une espace fine insécable, et la virgule.
        _ => ("\u{202F}", ","),
    }
}

/// 1234567 → « 1 234 567 ».
fn grouper(value: u64, thousands: &str) -> String {
    let digits = value.to_string();
    let mut output = String::new();
    for (rank, c) in digits.chars().enumerate() {
        if rank > 0 && (digits.len() - rank) % 3 == 0 {
            output.push_str(thousands);
        }
        output.push(c);
    }
    output
}

/// Écrit `valeur` (la valeur `nom`) selon `format`, dans la langue de la page.
pub fn format_value(name: &str, value: u64, format: &str, language: &str) -> String {
    let (thousands, decimals) = separators(language);
    let english = language.starts_with("en");
    match format {
        "number" => grouper(value, thousands),
        "cents" => format!("{}{decimals}{:02}", grouper(value / 100, thousands), value % 100),
        // Un nombre à virgule (ADR-066), gardé à l'échelle : `d2` montre 1250 en « 12,50 » ;
        // `nd2`, groupé par milliers, « 1 234,50 ». Le moteur les écrit, pas l'auteur.
        f if f.len() == 2 && f.starts_with('d') || f.len() == 3 && f.starts_with("nd") => {
            let places = f[f.len() - 1..].parse::<u32>().unwrap_or(0).min(6);
            let scale = 10u64.pow(places);
            let whole = if f.starts_with('n') { grouper(value / scale, thousands) } else { (value / scale).to_string() };
            if places == 0 { whole } else { format!("{whole}{decimals}{:0width$}", value % scale, width = places as usize) }
        }
        // Un temps de chronomètre, gardé en millisecondes (ADR-089) : « 01:23,45 », et les heures
        // devant quand il y en a, « 1:02:03,45 ».
        "stopwatch" => {
            let hundredths = value / 10;
            let (hours, minutes, seconds, rest) = (hundredths / 360_000, hundredths / 6_000 % 60, hundredths / 100 % 60, hundredths % 100);
            if hours > 0 {
                format!("{hours}:{minutes:02}:{seconds:02}{decimals}{rest:02}")
            } else {
                format!("{minutes:02}:{seconds:02}{decimals}{rest:02}")
            }
        }
        "name" => {
            let (days, month) = if english { (DAYS_EN, MONTHS_EN) } else { (DAYS_FR, MONTHS_FR) };
            let list: &[&str] = if name == "weekday" { &days } else { &month };
            value.checked_sub(1).and_then(|i| list.get(i as usize)).map_or_else(|| value.to_string(), |n| (*n).to_string())
        }
        zeros => format!("{value:0width$}", width = zeros.len()),
    }
}

/// Les valeurs à format d'un texte : `{minute:00}` → (`minute`, `00`).
pub fn formats_in(text: &str) -> Vec<(&str, &str)> {
    let mut found_list = Vec::new();
    let mut remainder = text;
    while let Some(start) = remainder.find('{') {
        remainder = &remainder[start + 1..];
        let Some(end) = remainder.find('}') else { break };
        if let Some((name, format)) = remainder[..end].split_once(':') {
            if name.starts_with(|c: char| c.is_ascii_lowercase()) && name.chars().all(|c| c.is_ascii_alphanumeric()) && !format.is_empty() && format.chars().all(|c| c.is_ascii_alphanumeric()) {
                found_list.push((name, format));
            }
        }
        remainder = &remainder[end..];
    }
    found_list
}

/// Remplit, dans une page fabriquée, chaque valeur à format par son départ :
/// `<span data-state="minute" data-format="00"></span>` → `…>05</span>`.
pub fn fill(html: &str, values: &[(String, u64)], language: &str) -> String {
    const START: &str = "<span data-state=\"";
    let mut output = String::with_capacity(html.len());
    let mut remainder = html;
    while let Some(place) = remainder.find(START) {
        output.push_str(&remainder[..place]);
        let after = &remainder[place + START.len()..];
        let opening = after.find('>').map(|f| &after[..f]);
        if let Some((name, format)) = opening.and_then(|o| o.split_once("\" data-format=\"")).map(|(n, f)| (n, f.trim_end_matches('"'))) {
            if let (Some((_, value)), true) = (values.iter().find(|(known, _)| known == name), after[name.len() + format.len() + 16..].starts_with("></span>")) {
                let header = &remainder[place..place + START.len() + name.len() + 15 + format.len() + 2];
                output.push_str(header);
                output.push_str(&format_value(name, *value, format, language));
                output.push_str("</span>");
                remainder = &after[name.len() + format.len() + 16 + "></span>".len()..];
                continue;
            }
        }
        output.push_str(START);
        remainder = after;
    }
    output.push_str(remainder);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_formats() {
        assert_eq!(format_value("minute", 5, "00", "fr"), "05");
        assert_eq!(format_value("n", 1234567, "number", "fr"), "1\u{202F}234\u{202F}567");
        assert_eq!(format_value("n", 1234567, "number", "en"), "1,234,567");
        assert_eq!(format_value("total", 123450, "cents", "fr"), "1\u{202F}234,50");
        assert_eq!(format_value("total", 7, "cents", "en"), "0.07");
        // Un chronomètre (ADR-089) : des millisecondes, écrites en minutes, secondes et centièmes.
        assert_eq!(format_value("time", 83_456, "stopwatch", "fr"), "01:23,45");
        assert_eq!(format_value("time", 3_723_450, "stopwatch", "en"), "1:02:03.45");
        assert_eq!(format_value("time", 0, "stopwatch", "fr"), "00:00,00");
        assert_eq!(format_value("weekday", 2, "name", "fr"), "mardi");
        assert_eq!(format_value("month", 10, "name", "en"), "October");
        assert_eq!(format_value("month", 0, "name", "fr"), "0");
        assert_eq!(formats_in("il est {hour} h {minute:00}, {total:cents} €"), [("minute", "00"), ("total", "cents")]);
        let html = "<p><span data-state=\"minute\" data-format=\"00\"></span> et <span data-state=\"n\"></span></p>";
        assert_eq!(fill(html, &[("minute".into(), 7)], "fr"), "<p><span data-state=\"minute\" data-format=\"00\">07</span> et <span data-state=\"n\"></span></p>");
    }
}
