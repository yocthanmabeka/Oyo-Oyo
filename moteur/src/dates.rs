//! Les dates (ADR-067). Une date est un texte « AAAA-MM-JJ » : la forme que donnent le champ date
//! du navigateur et les données JSON. Le moteur la compare dans le temps, lui ajoute des jours,
//! compte les jours entre deux dates, et la montre dans la langue de la page. `today` est la date
//! du jour, donnée par l'appareil du visiteur (ou par le serveur qui fabrique la page).

/// Le nom de la date du jour, que le moteur donne comme il donne `hour` et `minute`.
pub const TODAY: &str = "today";

/// Les formats d'une date : « 7 octobre 2026 », « mercredi ».
pub const FORMATS: &[&str] = &["date", "weekday"];

use crate::holo::{Program, Value};

/// La page lit-elle la date du jour ? `{today}`, `If(arrival, under: today)`, `Days(from: today, …)`.
pub fn uses_today(program: &Program) -> bool {
    fn visit(value: &Value) -> bool {
        match value {
            Value::Name(name) => name == TODAY,
            Value::Text(text) => crate::state::names_in(text).contains(&TODAY),
            Value::List(elements) => elements.iter().any(visit),
            Value::Block(block) => block.arguments.iter().any(|a| visit(&a.value)),
            _ => false,
        }
    }
    program.root.arguments.iter().any(|a| visit(&a.value))
}

/// Une valeur de la page est-elle une date ? `today` ; un texte déclaré avec une date,
/// `State(due: "2026-12-24")` ; ou un texte qu'un champ date présente, `Input(type: date)`.
pub fn is_date(program: &Program, name: &str) -> bool {
    if name == TODAY {
        return true;
    }
    if crate::state::initial_texts(program).iter().any(|(known, text)| known == name && days(text).is_some()) {
        return true;
    }
    let mut presented = false;
    let _ = crate::rules::for_each_block(&program.root, &mut |block| {
        presented |= block.name == "Input"
            && matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(v)) if v == name)
            && matches!(block.argument("type").map(|a| &a.value), Some(Value::Name(t)) if t == "date");
        Ok(())
    });
    presented
}

/// Les bornes d'un champ date, `Input(type: date, min: today, max: "2026-12-31")` : `today`, ou
/// une date écrite.
pub fn bound(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::Name(name) if name == TODAY => Some(today()),
        Value::Text(text) if days(text).is_some() => Some(text.clone()),
        _ => None,
    }
}

/// Les jours depuis le 1er janvier 1970 d'une date « 2026-10-07 », si elle existe au calendrier
/// (pas de 31 avril, un 29 février seulement les années bissextiles).
pub fn days(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' || !text.chars().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit()) {
        return None;
    }
    let (year, month, day): (i64, u32, u32) = (text[0..4].parse().ok()?, text[5..7].parse().ok()?, text[8..10].parse().ok()?);
    if !(1..=12).contains(&month) || day == 0 || day > month_length(year, month) {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

/// La date qui tombe ce jour-là : 20733 → « 2026-10-07 ».
pub fn text(days: i64) -> String {
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Une date décalée de `shift` jours (négatif : en arrière). Rien si ce n'est pas une date, ou si
/// l'on sort des années 0001 à 9999.
pub fn shifted(date: &str, shift: i64) -> Option<String> {
    let moved = days(date)?.checked_add(shift)?;
    let year = civil_from_days(moved).0;
    (1..=9999).contains(&year).then(|| text(moved))
}

/// La date du jour, d'après l'heure donnée au moteur (`set_now`).
pub fn today() -> String {
    let [year, month, day, ..] = crate::state::now();
    format!("{year:04}-{month:02}-{day:02}")
}

/// Le jour de la semaine : 1 lundi … 7 dimanche (le 1er janvier 1970 était un jeudi).
pub fn weekday(days: i64) -> u64 {
    (days + 3).rem_euclid(7) as u64 + 1
}

/// Une date dans la langue de la page : `date` → « 7 octobre 2026 » (« 1er octobre » en
/// français), « October 7, 2026 » en anglais ; `weekday` → « mercredi ». Un texte qui n'est pas
/// une date est rendu tel quel.
pub fn format(text: &str, format: &str, language: &str) -> String {
    let Some(days) = days(text) else { return text.to_string() };
    let (year, month, day) = civil_from_days(days);
    if format == "weekday" {
        return crate::format::format_value("weekday", weekday(days), "name", language);
    }
    let month_name = crate::format::format_value("month", u64::from(month), "name", language);
    if language.starts_with("en") {
        format!("{month_name} {day}, {year}")
    } else {
        format!("{day}{} {month_name} {year}", if day == 1 { "er" } else { "" })
    }
}

fn leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn month_length(year: i64, month: u32) -> u32 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// Les deux calculs du calendrier, d'après Howard Hinnant (« chrono-compatible low-level date
// algorithms ») : exacts pour toutes les dates du calendrier grégorien.
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_from_march = i64::from((month + 9) % 12);
    let day_of_year = (153 * month_from_march + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_from_march + 2) / 5 + 1) as u32;
    let month = if month_from_march < 10 { month_from_march + 3 } else { month_from_march - 9 } as u32;
    (if month <= 2 { year_of_era + era * 400 + 1 } else { year_of_era + era * 400 }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_calendar_is_exact() {
        assert_eq!(days("1970-01-01"), Some(0));
        assert_eq!(days("2026-10-07"), Some(20_733));
        assert_eq!(text(20_733), "2026-10-07");
        // Les années bissextiles : 2024 oui, 2100 non, 2000 oui.
        assert!(days("2024-02-29").is_some() && days("2100-02-29").is_none() && days("2000-02-29").is_some());
        assert!(days("2026-04-31").is_none() && days("2026-13-01").is_none() && days("26-10-07").is_none() && days("2026-1a-07").is_none());
        // Chaque jour, de 1900 à 2100, revient à lui-même.
        for d in days("1900-01-01").unwrap()..days("2100-12-31").unwrap() {
            assert_eq!(days(&text(d)), Some(d));
        }
        assert_eq!(shifted("2026-12-30", 3).as_deref(), Some("2027-01-02"));
        assert_eq!(shifted("2024-03-01", -1).as_deref(), Some("2024-02-29"));
        assert_eq!(weekday(days("2026-10-07").unwrap()), 3, "le 7 octobre 2026 est un mercredi");
        assert_eq!(format("2026-10-07", "date", "fr"), "7 octobre 2026");
        assert_eq!(format("2026-10-01", "date", "fr"), "1er octobre 2026");
        assert_eq!(format("2026-10-07", "date", "en"), "October 7, 2026");
        assert_eq!(format("2026-10-07", "weekday", "fr"), "mercredi");
        assert_eq!(format("pas une date", "date", "fr"), "pas une date");
    }
}
