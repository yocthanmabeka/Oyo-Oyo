//! Les heures (ADR-109). Une heure est un texte « HH:MM », la forme que donnent le champ
//! `Input(type: time)` et les données JSON ; un moment, un texte « AAAA-MM-JJTHH:MM », une date et
//! une heure (ISO 8601, la forme de `<input type="datetime-local">`). Sans fuseau : c'est l'heure
//! de l'horloge du visiteur, jamais convertie en silence. `now` est le moment présent, donné par
//! l'appareil du visiteur (ou par le serveur qui fabrique la page), comme `today`, `hour` et
//! `minute`.
//!
//! ```holo
//! Page(
//!   state: State(train: "18:45", arrival: "09:00", departure: "17:30", meeting: "14:00"),
//!   computed: [
//!     Minutes(name: left, from: now, to: train),
//!     Minutes(name: worked, from: arrival, to: departure),
//!   ],
//!   children: [
//!     P("The {train:time} train leaves in {left:duration}."),
//!     Input(value: arrival, label: "Arrival", type: time),
//!     Input(value: departure, label: "Departure", type: time),
//!     P("Worked: {worked:duration}. Meeting at {meeting:time}."),
//!     Button(name: Later, text: "+15 min"),
//!   ],
//!   rules: [ On(Later.tap, effect: meeting.add(15min)) ],
//! )
//! ```
//!
//! Entre deux moments, `Minutes` compte les vraies minutes : la page donne au moteur les
//! changements d'heure du fuseau du visiteur (`set_zone`), et la nuit du passage à l'heure d'hiver
//! compte une heure de plus, comme la montre au poignet. Avec une heure seule, sans date, le compte
//! avance jusqu'à la prochaine fois que l'horloge la montre : de 22:00 à 06:00, 8 h.

use crate::holo::{Block, Error, Program, Value};
use crate::state::{State, Texts};
use std::cell::RefCell;
use std::cmp::Ordering;

/// Le nom du moment présent, que le moteur donne comme il donne `today`.
pub const NOW: &str = "now";

/// Les formats d'une heure et d'une durée : `{train:time}` (« 18:45 »), `{left:duration}`
/// (« 2 h et 15 min »).
pub const FORMATS: &[&str] = &["time", "duration"];

/// Les réglages de `Minutes`.
pub const MINUTES_PARAMS: &[&str] = &["name", "from", "to"];

/// Une durée ajoutée ou retirée d'un coup, au plus : une année de 366 jours, en minutes.
pub const SHIFT_MAX: i64 = 527_040;

/// Les minutes d'un jour.
const DAY: i64 = 1440;

/// Ce qu'un texte dit du temps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Time {
    /// Une heure seule, « 18:45 » : les minutes depuis minuit, de 0 à 1439.
    Hour(i64),
    /// Un moment, « 2026-12-31T20:30 » : les minutes depuis le 1er janvier 1970 à minuit, lues à
    /// l'horloge du visiteur.
    Moment(i64),
    /// Une date (ADR-067), « 2026-12-31 » : les jours depuis le 1er janvier 1970.
    Date(i64),
}

/// Les minutes depuis minuit d'une heure « 18:45 », si elle existe à l'horloge (de 00:00 à
/// 23:59). « 18:45:30 » aussi, celle des données : les secondes ne comptent pas.
pub fn minutes_of_day(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if !(bytes.len() == 5 || bytes.len() == 8 && bytes[5] == b':') || bytes[2] != b':' {
        return None;
    }
    let number = |range: std::ops::Range<usize>| text.get(range).filter(|t| t.bytes().all(|c| c.is_ascii_digit())).and_then(|t| t.parse::<i64>().ok());
    let (hours, minutes) = (number(0..2)?, number(3..5)?);
    if bytes.len() == 8 && number(6..8)? > 59 {
        return None;
    }
    (hours < 24 && minutes < 60).then_some(hours * 60 + minutes)
}

/// Lit un texte : une heure, un moment (la date, « T » ou une espace, puis l'heure), une date ;
/// rien sinon.
pub fn read(text: &str) -> Option<Time> {
    if let Some(minutes) = minutes_of_day(text) {
        return Some(Time::Hour(minutes));
    }
    if let Some(days) = crate::dates::days(text) {
        return Some(Time::Date(days));
    }
    if !matches!(text.as_bytes().get(10), Some(b'T' | b' ')) {
        return None;
    }
    Some(Time::Moment(crate::dates::days(text.get(..10)?)? * DAY + minutes_of_day(text.get(11..)?)?))
}

/// L'heure de ces minutes depuis minuit : 1125 → « 18:45 ». Elle fait le tour du cadran.
pub fn hour_text(minutes: i64) -> String {
    let minutes = minutes.rem_euclid(DAY);
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// Le moment qui tombe à cette minute : « 2026-12-31T20:30 ». Rien hors des années 0001 à 9999.
pub fn moment_text(minutes: i64) -> Option<String> {
    Some(format!("{}T{}", crate::dates::shifted("1970-01-01", minutes.div_euclid(DAY))?, hour_text(minutes)))
}

/// Le moment présent, d'après l'heure donnée au moteur (`set_now`) : « 2026-10-10T14:30 ».
pub fn now() -> String {
    let [year, month, day, _, hour, minute] = crate::state::now();
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}")
}

/// La page lit-elle le moment présent ? `{now:time}`, `Minutes(from: now, …)`, `start.set(now)`,
/// `If(now, over: "18:45")`.
pub fn uses_now(program: &Program) -> bool {
    fn visit(value: &Value) -> bool {
        match value {
            Value::Name(name) => name == NOW,
            Value::Text(text) => crate::state::names_in(text).contains(&NOW),
            Value::List(elements) => elements.iter().any(visit),
            Value::Block(block) => block.arguments.iter().any(|a| visit(&a.value)),
            _ => false,
        }
    }
    program.root.arguments.iter().any(|a| visit(&a.value))
}

/// Une valeur de la page est-elle une heure ? Un texte déclaré avec une heure, `State(train:
/// "18:45")`, ou un texte qu'un champ `Input(type: time)` présente.
pub fn is_hour(program: &Program, name: &str) -> bool {
    if crate::state::initial_texts(program).iter().any(|(known, text)| known == name && minutes_of_day(text).is_some()) {
        return true;
    }
    let mut presented = false;
    let _ = crate::rules::for_each_block(&program.root, &mut |block| {
        presented |= block.name == "Input"
            && matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(v)) if v == name)
            && matches!(block.argument("type").map(|a| &a.value), Some(Value::Name(t)) if t == "time");
        Ok(())
    });
    presented
}

/// Une valeur de la page est-elle un moment ? `now`, ou un texte déclaré avec un moment,
/// `State(concert: "2026-12-31T20:30")`.
pub fn is_moment(program: &Program, name: &str) -> bool {
    name == NOW || crate::state::initial_texts(program).iter().any(|(known, text)| known == name && matches!(read(text), Some(Time::Moment(_))))
}

/// Une heure, ou un moment.
pub fn is_time(program: &Program, name: &str) -> bool {
    is_moment(program, name) || is_hour(program, name)
}

// ---------------------------------------------------------------- le fuseau du visiteur

/// Le fuseau du visiteur, donné par la page : la minute présente en temps universel, le décalage
/// de son horloge (en minutes, à l'est du temps universel) avant le premier changement d'heure
/// connu, puis chaque changement de l'année passée et de l'année qui vient (la minute universelle
/// où il se fait, et le nouveau décalage). Sans fuseau donné (le serveur, un essai du moteur),
/// l'horloge est celle du temps universel, sans changement d'heure.
#[derive(Debug, Clone, Default)]
struct Zone {
    now: Option<i64>,
    first: i64,
    changes: Vec<(i64, i64)>,
}

thread_local! {
    static ZONE: RefCell<Zone> = RefCell::new(Zone::default());
}

/// Donne au moteur le fuseau du visiteur : « 29857830;120;29862540:60;30361260:120 » (la minute
/// présente en temps universel ; le décalage de départ ; chaque changement). Un texte abîmé est
/// lu jusqu'où il va ; vide, il redonne le temps universel.
pub fn set_zone(text: &str) {
    let mut parts = text.split(';');
    let mut zone = Zone { now: parts.next().and_then(|n| n.trim().parse().ok()), ..Zone::default() };
    // Un décalage va de −18 h à +18 h, comme pour `Date`.
    let offset = |text: &str| text.trim().parse::<i64>().ok().filter(|o| o.abs() <= 1080);
    zone.first = parts.next().and_then(offset).unwrap_or(0);
    for change in parts.take(64) {
        match change.split_once(':').map(|(at, o)| (at.trim().parse::<i64>().ok(), offset(o))) {
            Some((Some(at), Some(o))) if zone.changes.last().is_none_or(|(before, _)| at > *before) => zone.changes.push((at, o)),
            _ => break,
        }
    }
    ZONE.with(|z| *z.borrow_mut() = zone);
}

/// Les minutes universelles où l'horloge du visiteur montre ce moment, dans l'ordre : une seule
/// d'habitude ; deux pour l'heure répétée, la nuit du passage à l'heure d'hiver ; pour l'heure
/// sautée au printemps, qui n'existe pas, celle d'une heure plus tard, comme le fait `Date`.
fn occurrences(local: i64) -> Vec<i64> {
    ZONE.with(|z| {
        let zone = z.borrow();
        let slices: Vec<(i64, i64)> = std::iter::once((i64::MIN, zone.first)).chain(zone.changes.iter().copied()).collect();
        let mut found = Vec::new();
        let mut skipped = local - zone.first;
        for (rank, (start, offset)) in slices.iter().enumerate() {
            let end = slices.get(rank + 1).map_or(i64::MAX, |(next, _)| *next);
            let candidate = local - offset;
            if (*start..end).contains(&candidate) {
                found.push(candidate);
            } else if candidate >= *start {
                skipped = candidate;
            }
        }
        if found.is_empty() {
            found.push(skipped);
        }
        found
    })
}

/// Les minutes universelles d'un moment lu à l'horloge du visiteur ; l'heure répétée est prise
/// la première fois.
fn universal(local: i64) -> i64 {
    occurrences(local)[0]
}

/// Le moment, à l'horloge du visiteur, d'une minute universelle.
fn local(universal: i64) -> i64 {
    ZONE.with(|z| {
        let zone = z.borrow();
        universal + zone.changes.iter().take_while(|(at, _)| *at <= universal).last().map_or(zone.first, |(_, offset)| *offset)
    })
}

/// La minute universelle du moment présent : celle que la page a donnée avec l'heure, quand elle
/// tombe sur la même minute de l'horloge ; sinon, celle qu'on lit à l'horloge.
fn universal_now(local_now: i64) -> i64 {
    ZONE.with(|z| z.borrow().now).filter(|given| local(*given) == local_now).unwrap_or_else(|| universal(local_now))
}

// ---------------------------------------------------------------- compter

/// Les minutes universelles d'un moment ; `now` compte depuis la minute que la page a donnée.
fn real(local_minutes: i64, is_now: bool) -> i64 {
    if is_now {
        universal_now(local_minutes)
    } else {
        universal(local_minutes)
    }
}

/// Les minutes de `from` à `to` (`from_now`, `to_now` : c'est `now`).
/// - Entre deux moments : les vraies minutes, d'après le fuseau du visiteur ; négatives si `to`
///   vient avant `from`.
/// - Entre deux heures seules : le compte avance jusqu'à la prochaine fois que l'horloge montre
///   `to` (de 22:00 à 06:00 : 480 ; d'une heure à elle-même : 0).
/// - D'un moment à une heure : jusqu'à la prochaine fois que l'horloge la montre ; d'une heure à un
///   moment, depuis la dernière fois qu'elle l'a montrée. De 0 à moins d'un jour.
///
/// Rien si l'un des deux n'est ni une heure ni un moment.
pub fn between(from: &str, to: &str, from_now: bool, to_now: bool) -> Option<i64> {
    match (read(from)?, read(to)?) {
        (Time::Moment(a), Time::Moment(b)) => Some(real(b, to_now) - real(a, from_now)),
        (Time::Hour(a), Time::Hour(b)) => Some((b - a).rem_euclid(DAY)),
        (Time::Moment(a), Time::Hour(b)) => {
            let start = real(a, from_now);
            let next = a + (b - a.rem_euclid(DAY)).rem_euclid(DAY);
            // La prochaine fois : aujourd'hui (la seconde, pour l'heure répétée), sinon demain.
            let found = occurrences(next).into_iter().find(|u| *u >= start).unwrap_or_else(|| universal(next + DAY));
            Some(found - start)
        }
        (Time::Hour(a), Time::Moment(b)) => {
            let end = real(b, to_now);
            let last = b - (b.rem_euclid(DAY) - a).rem_euclid(DAY);
            let found = occurrences(last).into_iter().rev().find(|u| *u <= end).unwrap_or_else(|| *occurrences(last - DAY).last().unwrap_or(&end));
            Some(end - found)
        }
        _ => None,
    }
}

/// Les `Minutes` de la page, dans l'ordre écrit : (nom, de, à).
fn minutes_blocks(program: &Program) -> Vec<(String, String, String)> {
    let Some(Value::List(items)) = program.root.argument("computed").map(|a| &a.value) else { return Vec::new() };
    let name = |block: &Block, param: &str| match block.argument(param).map(|a| &a.value) {
        Some(Value::Name(n)) => n.clone(),
        _ => String::new(),
    };
    items.iter().filter_map(|v| if let Value::Block(b) = v { Some(b) } else { None }).filter(|b| b.name == "Minutes").map(|b| (name(b, "name"), name(b, "from"), name(b, "to"))).collect()
}

/// Le nom des nombres de minutes, `Minutes(name: left, …)`.
pub fn minutes_names(program: &Program) -> Vec<String> {
    minutes_blocks(program).into_iter().map(|(name, _, _)| name).collect()
}

/// Les nombres de minutes, d'après les heures et les moments de la page : 0 si l'un manque, ou si
/// le moment `to` est passé, sauf pour une valeur que la page laisse descendre sous zéro
/// (`negative: [late]`, ADR-102). Bornés à un milliard, comme toute valeur.
pub fn minutes_values(program: &Program, texts: &Texts) -> State {
    let text = |name: &str| texts.iter().find(|(n, _)| n == name).map(|(_, t)| t.as_str());
    minutes_blocks(program)
        .into_iter()
        .map(|(name, from, to)| {
            let count = match (text(&from), text(&to)) {
                (Some(a), Some(b)) => between(a, b, from == NOW, to == NOW).unwrap_or(0),
                _ => 0,
            };
            let floor = if crate::negative::allowed(program, &name) { -(crate::state::VALUE_MAX as i64) } else { 0 };
            (name, crate::negative::stored(count.clamp(floor, crate::state::VALUE_MAX as i64)))
        })
        .collect()
}

/// Vérifie un `Minutes(name: left, from: now, to: train)` : ses réglages, un nom en minuscules,
/// et deux heures ou moments. Rend son nom ; celui-ci est comparé aux autres valeurs dans
/// `computed.rs`, avec les listes calculées et les `Days`.
pub fn check_minutes(program: &Program, block: &Block) -> Result<String, Error> {
    let example = "Minutes(name: left, from: now, to: train)";
    let error = |message: String| Err(Error { message, pos: block.pos });
    for argument in &block.arguments {
        match argument.name.as_deref() {
            Some(n) if MINUTES_PARAMS.contains(&n) => {}
            Some(n) => return Err(Error { message: format!("« Minutes » n'a pas de paramètre « {n} » ; paramètres possibles : {}", MINUTES_PARAMS.join(", ")), pos: argument.pos }),
            None => return Err(Error { message: format!("chaque paramètre de « Minutes » est nommé : {example}"), pos: argument.pos }),
        }
    }
    let name = match block.argument("name").map(|a| &a.value) {
        Some(Value::Name(n)) if n.starts_with(|c: char| c.is_ascii_lowercase()) && !n.contains('_') => n.clone(),
        _ => return error(format!("« Minutes » attend « name », le nom du nombre de minutes, en minuscules et sans « _ » : {example}")),
    };
    for param in ["from", "to"] {
        match block.argument(param).map(|a| &a.value) {
            Some(Value::Name(time)) if is_time(program, time) => {}
            Some(Value::Name(date)) if crate::dates::is_date(program, date) => {
                return error(format!("« Minutes({param}: {date}) » : « {date} » est une date ; les jours entre deux dates se comptent par Days(name: …, from: …, to: …) ; des minutes, entre deux heures « HH:MM », deux moments « AAAA-MM-JJTHH:MM », ou depuis now"));
            }
            Some(Value::Name(other)) => {
                return error(format!("« Minutes({param}: {other}) » : « {other} » n'est ni une heure ni un moment ; une heure est un texte déclaré « HH:MM » ou un champ Input(type: time), un moment un texte « AAAA-MM-JJTHH:MM », et now le moment présent"));
            }
            _ => return error(format!("« Minutes » attend « {param} », une heure ou un moment : {example}")),
        }
    }
    Ok(name)
}

// ---------------------------------------------------------------- décaler

/// Une durée écrite avec son unité, en minutes : `15min`, `2h`, `1.5h`. Rien sans unité de temps
/// (`15`), à la seconde (`30s`), si elle ne tombe pas sur une minute entière (`1.01h`), ou si elle
/// dépasse une année.
pub fn written_minutes(value: &Value) -> Option<i64> {
    let Value::Number { value, unit: Some(unit), .. } = value else { return None };
    let minutes = match unit.as_str() {
        "min" => *value,
        "h" => value * 60.0,
        _ => return None,
    };
    ((minutes - minutes.round()).abs() < 1e-6 && minutes.abs() <= SHIFT_MAX as f64).then(|| minutes.round() as i64)
}

/// Une heure ou un moment décalé de `minutes` (en arrière si c'est négatif). Une heure seule fait
/// le tour du cadran (23:50 + 15 min = 00:05) ; un moment change de jour, en vraies minutes, d'après
/// le fuseau du visiteur. Rien si ce n'est ni une heure ni un moment, ou si l'on sort des années
/// 0001 à 9999.
pub fn shifted(text: &str, minutes: i64) -> Option<String> {
    match read(text)? {
        Time::Hour(hour) => Some(hour_text(hour + minutes)),
        Time::Moment(moment) => moment_text(local(universal(moment) + minutes)),
        Time::Date(_) => None,
    }
}

/// Vérifie qu'une heure ou un moment avance d'une durée : `meeting.add(15min)`, `meeting.sub(2h)`,
/// ou d'un nombre de minutes de la page, `end.add(length)`.
pub fn check_shift(program: &Program, value: &str, verb: &str, argument: &Value) -> Result<(), String> {
    let numbers = crate::state::initial(program).unwrap_or_default();
    let is_number = |name: &str| numbers.iter().any(|(known, _)| known == name) || crate::computed::days_names(program).iter().any(|known| known == name);
    match argument {
        _ if written_minutes(argument).is_some() => Ok(()),
        Value::Name(other) if is_number(other) && crate::state::places(program, other) == 0 => Ok(()),
        Value::Name(other) if is_number(other) => Err(format!("« {value}.{verb}({other}) » : « {other} » est un nombre à virgule ; une heure avance d'un nombre entier de minutes")),
        Value::Number { unit: Some(unit), .. } if unit == "s" || unit == "ms" => Err(format!("« {value}.{verb} » : une heure se compte à la minute ; écris des minutes ou des heures, {value}.{verb}(15min), {value}.{verb}(2h)")),
        Value::Number { unit: Some(unit), .. } if unit == "min" || unit == "h" => Err(format!("« {value}.{verb} » : une durée tombe sur une minute entière, et dépasse au plus une année (527040min)")),
        _ => Err(format!("« {value}.{verb} » ajoute ou retire une durée, écrite avec son unité : {value}.{verb}(15min), {value}.{verb}(2h) ; ou un nombre de minutes de la page, {value}.{verb}(length)")),
    }
}

/// Ce que devient le texte d'une heure ou d'un moment après une demande qui le décale, `add` ou
/// `sub` ; ou après `start.set(now)`, où une heure prend l'heure du moment présent (« 14:30 ») et
/// un moment le moment entier. Rien pour une autre demande, ou une autre valeur.
pub fn after_request(program: &Program, value: &str, verb: &str, argument: Option<&Value>, texts: &Texts, numbers: &State) -> Option<String> {
    let current = texts.iter().find(|(n, _)| n == value).map(|(_, t)| t.as_str())?;
    match (verb, argument?) {
        ("set", Value::Name(other)) if other == NOW && is_hour(program, value) => {
            let now = texts.iter().find(|(n, _)| n == NOW).map_or_else(now, |(_, t)| t.clone());
            match read(&now)? {
                Time::Moment(moment) => Some(hour_text(moment)),
                _ => None,
            }
        }
        ("add" | "sub", amount) if is_time(program, value) => {
            let minutes = match amount {
                Value::Name(other) => {
                    let units = numbers.iter().find(|(known, _)| known == other).map(|(_, v)| *v).or_else(|| crate::computed::days_values(program, texts).into_iter().find(|(known, _)| known == other).map(|(_, v)| v))?;
                    crate::negative::signed(units)
                }
                written => written_minutes(written)?,
            };
            // Une heure vide, ou qui n'en est pas une, ne bouge pas.
            shifted(current, if verb == "add" { minutes } else { -minutes })
        }
        _ => None,
    }
}

// ---------------------------------------------------------------- comparer

/// L'ordre de deux temps : deux heures, deux moments, deux dates ; un moment face à une heure seule
/// se compare par son heure, face à une date par sa date. Rien entre une heure et une date, ni
/// pour un texte qui n'est pas un temps.
pub fn order(a: &str, b: &str) -> Option<Ordering> {
    let day = |m: i64| m.div_euclid(DAY);
    Some(match (read(a)?, read(b)?) {
        (Time::Hour(x), Time::Hour(y)) | (Time::Moment(x), Time::Moment(y)) | (Time::Date(x), Time::Date(y)) => x.cmp(&y),
        (Time::Moment(m), Time::Hour(h)) => m.rem_euclid(DAY).cmp(&h),
        (Time::Hour(h), Time::Moment(m)) => h.cmp(&m.rem_euclid(DAY)),
        (Time::Moment(m), Time::Date(d)) => day(m).cmp(&d),
        (Time::Date(d), Time::Moment(m)) => d.cmp(&day(m)),
        _ => return None,
    })
}

/// Deux textes se comparent-ils comme des heures ? Il faut qu'au moins l'un soit une heure ou un
/// moment : deux dates restent comparées par `dates.rs` (ADR-067).
pub fn comparable(a: &str, b: &str) -> bool {
    let timed = |t: &str| matches!(read(t), Some(Time::Hour(_) | Time::Moment(_)));
    (timed(a) || timed(b)) && order(a, b).is_some()
}

/// Une comparaison entre deux heures est-elle vraie ? `is` (la même minute), `not`, `over` (plus
/// tard), `under` (plus tôt).
pub fn holds(word: &str, a: &str, b: &str) -> bool {
    match (word, order(a, b)) {
        ("is", Some(o)) => o == Ordering::Equal,
        ("not", Some(o)) => o != Ordering::Equal,
        ("over", Some(o)) => o == Ordering::Greater,
        ("under", Some(o)) => o == Ordering::Less,
        _ => false,
    }
}

/// Vérifie une condition sur une heure ou un moment, comparé à une autre valeur (`other`) ou à un
/// texte écrit (`written`) : `If(now, over: "18:45")`, `If(now, under: closing)`. Deux temps qui
/// se comparent (une heure à une heure ou à un moment, un moment aussi à une date) ; et, par `is`
/// ou `not`, n'importe quel texte (`If(arrival, is: "")`).
pub fn check_comparison(program: &Program, value: &str, word: &str, other: Option<&str>, written: Option<&str>) -> Result<(), String> {
    let kind = |name: &str| {
        if is_moment(program, name) {
            "un moment"
        } else if is_hour(program, name) {
            "une heure"
        } else if crate::dates::is_date(program, name) {
            "une date"
        } else {
            "un texte"
        }
    };
    let sample = |kind: &str| match kind {
        "une heure" => "18:45",
        "un moment" => "2026-12-31T20:30",
        "une date" => "2026-12-31",
        _ => "",
    };
    let (mine, equality) = (kind(value), matches!(word, "is" | "not"));
    match (other, written) {
        (Some(other), _) if order(sample(mine), sample(kind(other))).is_some() => Ok(()),
        (Some(other), _) if equality && (mine == "un texte" || kind(other) == "un texte") => Ok(()),
        (Some(other), _) => Err(format!("« {word}: {other} » : « {value} » est {mine} et « {other} » {} ; une heure se compare à une heure ou à un moment, un moment aussi à une date", kind(other))),
        (None, Some(text)) if order(sample(mine), text).is_some() || equality && read(text).is_none() => Ok(()),
        _ => Err(format!(
            "« {word}: … » : « {value} » est {mine} ; on {} compare à une heure « 18:45 », à un moment « 2026-12-31T20:30 »{}, ou à une autre valeur de la page",
            if mine == "un moment" { "le" } else { "la" },
            if mine == "un moment" { ", à une date « 2026-12-31 »" } else { "" }
        )),
    }
}

// ---------------------------------------------------------------- montrer

/// Le premier mot d'une langue : `fr-CA` → `fr`. Les variantes d'une langue suivent pour
/// l'instant leur langue de base, comme les dates (ADR-067).
fn base(language: &str) -> String {
    language.split('-').next().unwrap_or("").to_ascii_lowercase()
}

/// Une heure dans la langue de la page, comme `Intl.DateTimeFormat` (`timeStyle: "short"`, d'après
/// le CLDR) : « 18:45 » en français, en allemand, en italien, en portugais, en néerlandais,
/// en suédois ; « 6:45 PM » en anglais ; « 9:05 » en espagnol (sans zéro devant l'heure).
pub fn time_text(minutes: i64, language: &str) -> String {
    let (hours, rest) = (minutes.rem_euclid(DAY) / 60, minutes.rem_euclid(60));
    match base(language).as_str() {
        "en" => format!("{}:{rest:02} {}", if hours % 12 == 0 { 12 } else { hours % 12 }, if hours < 12 { "AM" } else { "PM" }),
        "es" => format!("{hours}:{rest:02}"),
        _ => format!("{hours:02}:{rest:02}"),
    }
}

/// Les mots d'une durée dans une langue (le style « short » du CLDR, celui d'`Intl.DurationFormat`) :
/// le jour (au singulier, au pluriel), l'heure, la minute, l'espace qui les suit, et comment se
/// joignent deux, puis trois morceaux.
struct Words {
    day: (&'static str, &'static str),
    hour: &'static str,
    minute: &'static str,
    /// L'espace entre le nombre et le jour ou l'heure, puis celle entre le nombre et la minute.
    spaces: (&'static str, &'static str),
    /// Entre deux morceaux ; entre les deux derniers de trois (les premiers sont séparés par une
    /// virgule).
    and: (&'static str, &'static str),
}

fn words(language: &str) -> Words {
    match base(language).as_str() {
        // L'espace fine insécable devant « j » et « h », l'espace insécable devant « min » : celles du CLDR.
        "fr" => Words { day: ("j", "j"), hour: "h", minute: "min", spaces: ("\u{202F}", "\u{A0}"), and: (" et ", " et ") },
        "de" => Words { day: ("Tg.", "Tg."), hour: "Std.", minute: "Min.", spaces: (" ", " "), and: (", ", " und ") },
        "es" => Words { day: ("d", "d"), hour: "h", minute: "min", spaces: (" ", " "), and: (" y ", ", ") },
        "it" => Words { day: ("giorno", "giorni"), hour: "h", minute: "min", spaces: (" ", " "), and: (" e ", " e ") },
        "pt" => Words { day: ("dia", "dias"), hour: "h", minute: "min", spaces: (" ", " "), and: (" e ", " e ") },
        "nl" => Words { day: ("dag", "dagen"), hour: "uur", minute: "min", spaces: (" ", " "), and: (", ", ", ") },
        "sv" => Words { day: ("d", "d"), hour: "tim", minute: "min", spaces: (" ", " "), and: (", ", ", ") },
        // L'anglais, et les autres langues pour l'instant, comme les noms des jours (ADR-043).
        _ => Words { day: ("day", "days"), hour: "hr", minute: "min", spaces: (" ", " "), and: (", ", ", ") },
    }
}

/// Une durée en minutes, dans la langue de la page, comme `Intl.DurationFormat` (style « short »,
/// d'après le CLDR) : « 2 h et 15 min », « 2 hr, 15 min », « 3 j, 4 h et 5 min ». Les jours, puis
/// les heures, puis les minutes ; ce qui vaut zéro ne s'écrit pas, mais une durée nulle s'écrit
/// « 0 min » (là où `Intl.DurationFormat` n'écrit rien). Un compte négatif prend le signe moins de
/// la langue (ADR-102), devant le premier nombre.
pub fn duration(minutes: i64, language: &str) -> String {
    if minutes < 0 {
        return format!("{}{}", crate::format::minus(language), duration(minutes.checked_neg().unwrap_or(i64::MAX), language));
    }
    let words = words(language);
    let (days, hours, rest) = (minutes / DAY, minutes % DAY / 60, minutes % 60);
    let mut parts = Vec::new();
    if days > 0 {
        let number = crate::format::format_value("days", days as u64, "number", language);
        parts.push(format!("{number}{}{}", words.spaces.0, if days == 1 { words.day.0 } else { words.day.1 }));
    }
    if hours > 0 {
        parts.push(format!("{hours}{}{}", words.spaces.0, words.hour));
    }
    if rest > 0 || parts.is_empty() {
        parts.push(format!("{rest}{}{}", words.spaces.1, words.minute));
    }
    match parts.as_slice() {
        [one] => one.clone(),
        [first, second] => format!("{first}{}{second}", words.and.0),
        [first, second, third] => format!("{first}, {second}{}{third}", words.and.1),
        _ => String::new(),
    }
}

/// Une durée pour les machines, `<time datetime="PT2H15M">` (la forme ISO 8601 que lit HTML) :
/// « PT2H15M », « P3DT4H5M », « P1D », « PT0M ». Rien pour un compte négatif, que HTML n'écrit pas.
pub fn machine_duration(minutes: i64) -> Option<String> {
    if minutes < 0 {
        return None;
    }
    let (days, hours, rest) = (minutes / DAY, minutes % DAY / 60, minutes % 60);
    let mut time = String::new();
    if hours > 0 {
        time.push_str(&format!("{hours}H"));
    }
    if rest > 0 || (days == 0 && hours == 0) {
        time.push_str(&format!("{rest}M"));
    }
    let days = if days > 0 { format!("{days}D") } else { String::new() };
    Some(if time.is_empty() { format!("P{days}") } else { format!("P{days}T{time}") })
}

/// Une heure ou un moment montré dans la langue de la page : `time` → « 18:45 » ; un moment aussi
/// par sa date, `date` (« 31 décembre 2026 ») et `weekday` (« jeudi »). Un texte qui n'en est pas
/// un est rendu tel quel.
pub fn format_text(text: &str, format: &str, language: &str) -> String {
    match (read(text), format) {
        (Some(Time::Hour(minutes) | Time::Moment(minutes)), "time") => time_text(minutes, language),
        (Some(Time::Moment(minutes)), "date" | "weekday") => crate::dates::format(&crate::dates::text(minutes.div_euclid(DAY)), format, language),
        _ => text.to_string(),
    }
}

/// Une heure ou un moment pour les machines (`<time datetime="…">`) : « 18:45 »,
/// « 2026-12-31T20:30 ». Rien pour un autre texte (une date a déjà le sien, ADR-098).
pub fn machine_text(text: &str) -> Option<String> {
    match read(text)? {
        Time::Hour(minutes) => Some(hour_text(minutes)),
        Time::Moment(minutes) => moment_text(minutes),
        Time::Date(_) => None,
    }
}

/// Ce que la page montre d'une heure, d'un moment ou d'une durée, puis ce qu'en lisent les
/// machines, sur deux lignes : « 18:45\n18:45 », « 2 h et 15 min\nPT2H15M ». La page s'en sert
/// quand la valeur change.
pub fn shown_and_machine(value: &str, format: &str, language: &str) -> String {
    if format == "duration" {
        let minutes = value.trim().replace('\u{2212}', "-").parse::<i64>().unwrap_or(0);
        return format!("{}\n{}", duration(minutes, language), machine_duration(minutes).unwrap_or_default());
    }
    format!("{}\n{}", format_text(value, format, language), machine_text(value).unwrap_or_default())
}

/// Les places d'une heure ou d'un moment dans une page fabriquée, à remplir par son départ :
/// `<time data-state="train" data-format="time"></time>` → `…datetime="18:45">18:45</time>` ; un
/// moment aussi par sa date (`date`, `weekday`), avec le moment pour les machines.
pub fn places(name: &str, text: &str, language: &str) -> Vec<(String, String)> {
    let machine = machine_text(text).map(|m| format!(" datetime=\"{}\"", crate::flat::escape(&m))).unwrap_or_default();
    let formats: &[&str] = if matches!(read(text), Some(Time::Moment(_))) { &["time", "date", "weekday"] } else { &["time"] };
    formats
        .iter()
        .map(|format| {
            let opening = format!("<time data-state=\"{name}\" data-format=\"{format}\"");
            (format!("{opening}></time>"), format!("{opening}{machine}>{}</time>", crate::flat::escape(&format_text(text, format, language))))
        })
        .collect()
}

/// Remplit, dans une page fabriquée, chaque durée montrée par son départ :
/// `<time data-state="left" data-format="duration"></time>` → `…datetime="PT2H15M">2 h et 15 min</time>`.
pub fn fill(html: &str, values: &State, language: &str) -> String {
    let mut html = html.to_string();
    for (name, value) in values {
        let empty = format!("<time data-state=\"{name}\" data-format=\"duration\"></time>");
        if !html.contains(&empty) {
            continue;
        }
        let minutes = crate::negative::signed(*value);
        let machine = machine_duration(minutes).map(|m| format!(" datetime=\"{m}\"")).unwrap_or_default();
        html = html.replace(&empty, &format!("<time data-state=\"{name}\" data-format=\"duration\"{machine}>{}</time>", crate::flat::escape(&duration(minutes, language))));
    }
    html
}

/// Vérifie le format d'une valeur qui est une heure, un moment, ou montrée comme une heure ou une
/// durée : `{train:time}`, `{concert:date}`, `{left:duration}`.
pub fn check_format(program: &Program, name: &str, format: &str) -> Result<(), String> {
    let (hour, moment) = (is_hour(program, name), is_moment(program, name));
    let text = crate::state::initial_texts(program).iter().any(|(known, _)| known == name) || name == NOW;
    match format {
        "time" if hour || moment => Ok(()),
        "date" | "weekday" if moment => Ok(()),
        "duration" if !text && crate::state::places(program, name) == 0 => Ok(()),
        "time" => Err(format!("« {{{name}:time}} » montre une heure : « {name} » est {} ; une heure est un texte « HH:MM » (State({name}: \"18:45\") ou un champ Input(type: time)), un moment un texte « AAAA-MM-JJTHH:MM »", if text { "un texte qui n'est ni une heure ni un moment" } else { "un nombre" })),
        "duration" if text => Err(format!("« {{{name}:duration}} » écrit un nombre de minutes ; « {name} » est un texte : pour l'heure, {{{name}:time}}")),
        "duration" => Err(format!("« {{{name}:duration}} » écrit un nombre entier de minutes ; « {name} » est un nombre à virgule")),
        _ if moment => Err(format!("« {{{name}:{format}}} » : « {name} » est un moment ; formats possibles : time (20:30), date (31 décembre 2026), weekday (jeudi)")),
        _ => Err(format!("« {{{name}:{format}}} » : « {name} » est une heure ; format possible : time (18:45)")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'heure d'hiver de Paris en 2026, comme la donne la page : le décalage de départ (+60, l'heure
    /// d'hiver), puis le passage à l'heure d'été (le 29 mars à 1 h, temps universel) et le retour à
    /// l'heure d'hiver (le 25 octobre à 1 h, temps universel). `now` : la minute universelle donnée.
    fn paris(now: Option<i64>) -> String {
        let minute = |date: &str, hour: i64| crate::dates::days(date).unwrap() * DAY + hour * 60;
        format!("{};60;{}:120;{}:60", now.map_or(String::new(), |n| n.to_string()), minute("2026-03-29", 1), minute("2026-10-25", 1))
    }

    #[test]
    fn hours_and_moments_are_read_and_written() {
        assert_eq!(read("18:45"), Some(Time::Hour(1125)));
        assert_eq!(read("18:45:30"), Some(Time::Hour(1125)));
        assert_eq!(read("2026-10-10"), Some(Time::Date(20_736)));
        assert_eq!(read("2026-10-10T14:30"), Some(Time::Moment(20_736 * DAY + 870)));
        assert_eq!(read("2026-10-10 14:30:00"), read("2026-10-10T14:30"));
        // Une heure qui n'existe pas à l'horloge, ni au calendrier.
        for wrong in ["24:00", "25:99", "9:30", "18h45", "18:60", "2026-02-30T10:00", "2026-10-10T25:00", "2026-10-10X14:30", "", "«18:45»"] {
            assert_eq!(read(wrong).filter(|t| !matches!(t, Time::Date(_))), None, "{wrong}");
        }
        assert_eq!(hour_text(1125), "18:45");
        assert_eq!(hour_text(-15), "23:45");
        assert_eq!(moment_text(20_736 * DAY + 870).as_deref(), Some("2026-10-10T14:30"));
        // Décaler : une heure fait le tour du cadran, un moment change de jour.
        assert_eq!(shifted("23:50", 15).as_deref(), Some("00:05"));
        assert_eq!(shifted("00:10", -30).as_deref(), Some("23:40"));
        assert_eq!(shifted("2026-12-31T23:50", 15).as_deref(), Some("2027-01-01T00:05"));
        assert_eq!(shifted("2026-10-10", 15), None);
        assert_eq!(shifted("9999-12-31T23:50", 15), None);
        assert_eq!(written_minutes(&Value::Number { value: 15.0, unit: Some("min".into()), places: 0 }), Some(15));
        assert_eq!(written_minutes(&Value::Number { value: 1.5, unit: Some("h".into()), places: 1 }), Some(90));
        assert_eq!(written_minutes(&Value::Number { value: 1.01, unit: Some("h".into()), places: 2 }), None);
        assert_eq!(written_minutes(&Value::Number { value: 30.0, unit: Some("s".into()), places: 0 }), None);
        assert_eq!(written_minutes(&Value::Integer(15)), None);
        // Comparer : un moment face à une heure seule, par son heure ; face à une date, par sa date.
        assert!(holds("over", "2026-10-10T19:00", "18:45") && holds("under", "2026-10-10T18:00", "18:45") && holds("is", "2026-10-10T18:45", "18:45:00"));
        assert!(holds("over", "2026-12-25T00:00", "2026-12-24") && !holds("over", "2026-12-24T23:59", "2026-12-24"));
        assert!(!comparable("18:45", "2026-12-24") && !comparable("2026-12-24", "2026-12-25") && !comparable("18:45", "") && comparable("2026-12-24", "2026-12-24T10:00"));
    }

    #[test]
    fn hours_and_durations_are_written_like_intl() {
        // Ce qu'écrivent Intl.DateTimeFormat (timeStyle: "short") et Intl.DurationFormat (style:
        // "short") dans Chrome, langue par langue (relevé le 2026-10-10) ; « 0 min » là où
        // Intl.DurationFormat n'écrit rien.
        assert_eq!(time_text(545, "fr"), "09:05");
        assert_eq!(time_text(845, "en"), "2:05 PM");
        assert_eq!(time_text(5, "en"), "12:05 AM");
        assert_eq!(time_text(720, "en-US"), "12:00 PM");
        assert_eq!(time_text(5, "es"), "0:05");
        assert_eq!(time_text(1439, "de"), "23:59");
        let cases: &[(&str, i64, &str)] = &[
            ("fr", 135, "2\u{202F}h et 15\u{A0}min"),
            ("fr", 45, "45\u{A0}min"),
            ("fr", 120, "2\u{202F}h"),
            ("fr", 3 * DAY + 245, "3\u{202F}j, 4\u{202F}h et 5\u{A0}min"),
            ("fr", DAY + 1, "1\u{202F}j et 1\u{A0}min"),
            ("fr", 0, "0\u{A0}min"),
            ("fr", -135, "-2\u{202F}h et 15\u{A0}min"),
            ("en", 135, "2 hr, 15 min"),
            ("en", DAY, "1 day"),
            ("en", 400 * DAY + 1439, "400 days, 23 hr, 59 min"),
            ("de", 135, "2 Std., 15 Min."),
            ("de", 400 * DAY + 1439, "400 Tg., 23 Std. und 59 Min."),
            ("es", 135, "2 h y 15 min"),
            ("es", 400 * DAY + 1439, "400 d, 23 h, 59 min"),
            ("it", 3 * DAY + 245, "3 giorni, 4 h e 5 min"),
            ("it", DAY + 1, "1 giorno e 1 min"),
            ("pt", 3 * DAY + 245, "3 dias, 4 h e 5 min"),
            ("nl", 3 * DAY + 245, "3 dagen, 4 uur, 5 min"),
            ("sv", -135, "\u{2212}2 tim, 15 min"),
        ];
        for (language, minutes, expected) in cases {
            assert_eq!(duration(*minutes, language), *expected, "{language} {minutes}");
        }
        // Pour les machines : la forme ISO 8601 que lit HTML.
        assert_eq!(machine_duration(135).as_deref(), Some("PT2H15M"));
        assert_eq!(machine_duration(DAY).as_deref(), Some("P1D"));
        assert_eq!(machine_duration(3 * DAY + 245).as_deref(), Some("P3DT4H5M"));
        assert_eq!(machine_duration(0).as_deref(), Some("PT0M"));
        assert_eq!(machine_duration(-5), None);
        assert_eq!(format_text("2026-12-31T20:30", "date", "fr"), "31 décembre 2026");
        assert_eq!(format_text("2026-12-31T20:30", "weekday", "fr"), "jeudi");
        assert_eq!(format_text("2026-12-31T20:30", "time", "en"), "8:30 PM");
        assert_eq!(format_text("pas une heure", "time", "fr"), "pas une heure");
        assert_eq!(shown_and_machine("-15", "duration", "fr"), "-15\u{A0}min\n");
        assert_eq!(shown_and_machine("18:45", "time", "en"), "6:45 PM\n18:45");
    }

    #[test]
    fn the_visitors_zone_counts_the_real_minutes() {
        // Paris, la nuit du 24 au 25 octobre 2026 : à 3 h (heure d'été), l'horloge revient à 2 h.
        set_zone(&paris(None));
        let moment = |text: &str| match read(text) {
            Some(Time::Moment(m)) => m,
            _ => panic!("{text}"),
        };
        // De 22 h à 4 h : 7 vraies heures, pas 6 ; et 23 au printemps, de midi à midi.
        assert_eq!(between("2026-10-24T22:00", "2026-10-25T04:00", false, false), Some(7 * 60));
        assert_eq!(between("2026-03-28T12:00", "2026-03-29T12:00", false, false), Some(23 * 60));
        assert_eq!(between("2026-10-24T16:30", "2026-12-31T20:30", false, false), Some(68 * DAY + 5 * 60));
        // L'heure répétée est prise la première fois ; l'heure sautée avance d'autant, comme Date.
        assert_eq!(universal(moment("2026-10-25T02:30")), universal(moment("2026-10-25T01:30")) + 60);
        assert_eq!(universal(moment("2026-03-29T02:30")), universal(moment("2026-03-29T03:30")));
        assert_eq!(local(universal(moment("2026-07-14T12:00"))), moment("2026-07-14T12:00"));
        // Ajouter 2 h à 1 h 30, cette nuit-là : 2 h 30 à l'horloge, qui a reculé d'une heure en chemin.
        assert_eq!(shifted("2026-10-25T01:30", 120).as_deref(), Some("2026-10-25T02:30"));
        // Du moment présent à une heure seule : jusqu'à la prochaine fois que l'horloge la montre.
        // `now`, à 1 h 30 la seconde fois (après le retour à l'heure d'hiver) : la minute universelle
        // donnée par la page dit laquelle des deux.
        let second_time = universal(moment("2026-10-25T01:30")) + 120;
        set_zone(&paris(Some(second_time)));
        assert_eq!(local(second_time), moment("2026-10-25T02:30"));
        assert_eq!(between("2026-10-25T02:30", "04:00", true, false), Some(90));
        assert_eq!(between("2026-10-25T02:30", "04:00", false, false), Some(150));
        // La prochaine fois que l'horloge montre 2 h 45 : dans un quart d'heure, l'heure répétée.
        assert_eq!(between("2026-10-25T02:30", "02:45", true, false), Some(15));
        assert_eq!(between("2026-10-25T02:30", "02:15", true, false), Some(DAY - 15));
        // Sans fuseau donné (le serveur, un essai) : l'horloge du temps universel, sans changement.
        set_zone("");
        assert_eq!(between("2026-10-24T22:00", "2026-10-25T04:00", false, false), Some(6 * 60));
        assert_eq!(between("22:00", "06:00", false, false), Some(480));
        assert_eq!(between("09:00", "2026-10-10T14:30", false, false), Some(330));
        assert_eq!(between("2026-10-10T14:30", "2026-10-10T14:00", false, false), Some(-30));
        assert_eq!(between("2026-10-10T14:30", "2026-10-10", false, false), None);
    }

    /// Une page de l'heure : un train, un concert, des heures de travail, une réunion.
    const HOURS: &str = "Page(
  title: \"Train in {left:duration}\",
  state: State(train: \"18:45\", concert: \"2026-12-31T20:30\", arrival: \"09:00\", departure: \"17:30\", meeting: \"14:00\", end: \"14:00\", length: 90, bell: 0),
  negative: [late],
  computed: [
    Minutes(name: left, from: now, to: train),
    Minutes(name: wait, from: now, to: concert),
    Minutes(name: worked, from: arrival, to: departure),
    Minutes(name: late, from: concert, to: now),
  ],
  children: [
    P(\"Il est {now:time}.\"),
    If(now, under: train, children: [ P(\"Le train de {train:time} part dans {left:duration}.\") ], else: [ P(\"Le train est parti.\") ]),
    P(\"Concert le {concert:weekday} {concert:date} à {concert:time}, dans {wait:duration} ({late}).\"),
    Input(value: arrival, label: \"Arrivée\", type: time),
    Input(value: departure, label: \"Départ\", type: time),
    P(\"Travail : {worked:duration} ({worked} min).\"),
    P(\"Réunion à {meeting:time}, fin à {end:time}.\"),
    Button(name: In, text: \"Pointer\"), Button(name: Later, text: \"+15 min\"), Button(name: Earlier, text: \"-1 h\"), Button(name: End, text: \"Fin\"),
  ],
  rules: [
    On(In.tap, effect: arrival.set(now)),
    On(Later.tap, effect: meeting.add(15min)),
    On(Earlier.tap, effect: meeting.sub(1h)),
    On(End.tap, effect: [end.set(meeting), end.add(length)]),
    When(left, is: 0, effect: bell.add(1)),
    When(now, is: \"07:00\", effect: bell.add(10)),
  ],
)";

    fn value<'a>(state: &'a str, name: &str) -> &'a str {
        state.split(';').find_map(|chunk| chunk.strip_prefix(&format!("{name}="))).unwrap_or("?")
    }

    #[test]
    fn minutes_count_between_two_times_and_follow_the_clock() {
        set_zone("");
        crate::set_now([2026, 10, 10, 6, 16, 30]);
        let start = crate::initial_state(HOURS);
        // De 16 h 30 au train de 18 h 45 ; au concert, 82 jours et 4 heures ; de 9 h à 17 h 30 ; et le
        // retard, sous zéro (negative:) tant que le concert n'a pas commencé.
        assert_eq!([value(&start, "left"), value(&start, "wait"), value(&start, "worked"), value(&start, "late")], ["135", "118320", "510", "-118320"], "{start}");
        // Les gestes : une réunion qui avance d'un quart d'heure, recule d'une heure ; une fin calculée ;
        // l'arrivée pointée à l'heure du moment présent.
        let later = crate::arbitrate(HOURS, &start, "Later.tap");
        let earlier = crate::arbitrate(HOURS, &later, "Earlier.tap");
        let end = crate::arbitrate(HOURS, &earlier, "End.tap");
        assert_eq!([value(&later, "meeting"), value(&earlier, "meeting"), value(&end, "end")], ["'14%3A15", "'13%3A15", "'14%3A45"]);
        let clocked = crate::arbitrate(HOURS, &start, "In.tap");
        assert_eq!([value(&clocked, "arrival"), value(&clocked, "worked")], ["'16%3A30", "60"]);
        // Un champ heure : la nuit, de 22 h à 6 h, 8 h ; « 25:99 » n'est pas une heure, refusé.
        let night = crate::input(HOURS, &crate::input(HOURS, &start, "arrival", "22:00"), "departure", "06:00");
        assert_eq!(value(&night, "worked"), "480");
        assert_eq!(value(&crate::input(HOURS, &night, "arrival", "25:99"), "arrival"), "'22%3A00");
        // La minute qui passe : le compte suit l'horloge ; à 18 h 45, il est à zéro, la règle qui le
        // guette sonne une fois, et le train est parti ; à 18 h 46, il compte jusqu'au train de demain.
        crate::set_now([2026, 10, 10, 6, 18, 44]);
        let before = crate::advance_clock(HOURS, &start);
        let leaving = crate::conditions(HOURS, &before);
        crate::set_now([2026, 10, 10, 6, 18, 45]);
        let due = crate::advance_clock(HOURS, &before);
        let gone = crate::conditions(HOURS, &due);
        crate::set_now([2026, 10, 10, 6, 18, 46]);
        let after = crate::advance_clock(HOURS, &due);
        assert_eq!([value(&before, "left"), value(&due, "left"), value(&after, "left")], ["1", "0", "1439"]);
        assert_eq!([value(&before, "bell"), value(&due, "bell"), value(&after, "bell")], ["0", "1", "1"]);
        assert!(leaving.contains("now|under=train:1") && gone.contains("now|under=train:0"), "{leaving}\n{gone}");
        // Une règle qui guette une heure : When(now, is: "07:00") sonne à 7 h.
        crate::set_now([2026, 10, 11, 7, 6, 59]);
        let dawn = crate::advance_clock(HOURS, &after);
        crate::set_now([2026, 10, 11, 7, 7, 0]);
        let seven = crate::advance_clock(HOURS, &dawn);
        assert_eq!([value(&dawn, "bell"), value(&seven, "bell")], ["1", "11"]);
        // Le titre de l'onglet lit la durée (ADR-090).
        crate::set_now([2026, 10, 10, 6, 16, 30]);
        assert_eq!(crate::page_title(HOURS, &start), "Train in 2\u{202F}h et 15\u{A0}min");
    }

    #[test]
    fn the_page_shows_hours_and_durations_for_people_and_machines() {
        set_zone("");
        crate::set_now([2026, 10, 10, 6, 16, 30]);
        let html = crate::flat_view(HOURS, "").unwrap();
        for expected in [
            "Il est <time data-state=\"now\" data-format=\"time\" datetime=\"2026-10-10T16:30\">16:30</time>.",
            "Le train de <time data-state=\"train\" data-format=\"time\" datetime=\"18:45\">18:45</time> part dans <time data-state=\"left\" data-format=\"duration\" datetime=\"PT2H15M\">2\u{202F}h et 15\u{A0}min</time>.",
            "Concert le <time data-state=\"concert\" data-format=\"weekday\" datetime=\"2026-12-31T20:30\">jeudi</time> <time data-state=\"concert\" data-format=\"date\" datetime=\"2026-12-31T20:30\">31 décembre 2026</time> à <time data-state=\"concert\" data-format=\"time\" datetime=\"2026-12-31T20:30\">20:30</time>, dans <time data-state=\"wait\" data-format=\"duration\" datetime=\"P82DT4H\">82\u{202F}j et 4\u{202F}h</time> (<span data-state=\"late\" data-format=\"d0\">-118320</span>).",
            "Travail : <time data-state=\"worked\" data-format=\"duration\" datetime=\"PT8H30M\">8\u{202F}h et 30\u{A0}min</time> (<span data-state=\"worked\">510</span> min).",
            "<input type=\"time\" value=\"09:00\" data-bind=\"arrival\">",
        ] {
            assert!(html.contains(expected), "{expected}\n---\n{html}");
        }
        // Le compte n'est dans aucune région que le lecteur d'écran annonce à chaque changement.
        assert!(!html.contains("aria-live=\"polite\" data-state") && !html.contains("role=\"timer\""), "{html}");
        // En anglais : « 6:45 PM », « 2 hr, 15 min ».
        let english = crate::flat_view(&HOURS.replace("title:", "lang: \"en\", title:"), "").unwrap();
        assert!(english.contains(">6:45 PM</time>") && english.contains(">2 hr, 15 min</time>") && english.contains(">Thursday</time>"), "{english}");
        // Ce que la page récrit quand une valeur change.
        assert_eq!(shown_and_machine("2026-12-31T20:30", "date", "fr"), "31 décembre 2026\n2026-12-31T20:30");
        assert_eq!(shown_and_machine("510", "duration", "en"), "8 hr, 30 min\nPT8H30M");
    }

    #[test]
    fn a_form_counts_hours_without_javascript() {
        // holo serve, sans JavaScript (ADR-074) : les champs envoyés, puis la page fabriquée à nouveau.
        set_zone("");
        crate::set_now([2026, 10, 10, 6, 16, 30]);
        let fields = [("arrival".to_string(), "22:00".to_string()), ("departure".to_string(), "06:00".to_string())];
        let state = crate::visitor_gesture(HOURS, &crate::initial_state(HOURS), &fields);
        assert_eq!(value(&state, "worked"), "480");
        let page = crate::visitor_page(HOURS, "", &state, &[]).unwrap();
        assert!(page.contains(">8\u{202F}h</time> (<span data-state=\"worked\">480</span> min)"), "{page}");
        // Un toucher sans JavaScript : la réunion avance d'un quart d'heure.
        let tapped = crate::visitor_gesture(HOURS, &state, &[(crate::gestures::SIGNAL.to_string(), "Later.tap".to_string())]);
        assert_eq!(value(&tapped, "meeting"), "'14%3A15");
    }

    #[test]
    fn what_is_not_an_hour_is_refused_with_the_reason() {
        let page = |state: &str, more: &str, children: &str, rules: &str| format!("Page(state: State(train: \"18:45\", concert: \"2026-12-31T20:30\", buyer: \"\", length: 90, price: 2.50{state}){more}, children: [ P(\"{{train}}\"){children} ], rules: [ {rules} ])");
        for (source, message) in [
            (page(", now: \"\"", "", "", ""), "« now » est le moment présent"),
            (page("", ", keep: [now]", ", P(\"{now}\")", ""), "« keep » : « now » est le moment présent"),
            (page("", ", address: [now]", ", P(\"{now}\")", ""), "« now » est le moment présent, il ne va pas dans l'adresse"),
            (page("", ", visit: [now]", ", P(\"{now}\")", ""), "« visit » : « now » est le moment présent"),
            (page("", ", negative: [now]", ", P(\"{now}\")", ""), "seul un nombre peut être négatif"),
            (page("", "", ", Input(value: now, label: \"Now\")", ""), "on le lit, on ne l'écrit pas"),
            (page("", ", computed: [ Minutes(name: left, from: today, to: train) ]", "", ""), "est une date ; les jours entre deux dates se comptent par Days"),
            (page("", ", computed: [ Minutes(name: left, from: buyer, to: train) ]", "", ""), "n'est ni une heure ni un moment"),
            (page("", ", computed: [ Minutes(name: Left, from: now, to: train) ]", "", ""), "en minuscules"),
            (page("", ", computed: [ Minutes(name: left, from: now) ]", "", ""), "attend « to »"),
            (page("", ", computed: [ Minutes(name: left, from: now, to: train, every: 1) ]", "", ""), "n'a pas de paramètre « every »"),
            (page("", ", computed: [ Minutes(name: train, from: now, to: concert) ]", "", ""), "est déjà le nom d'une valeur"),
            (page("", "", ", Button(name: B, text: \"b\")", "On(B.tap, effect: train.add(15))"), "écrite avec son unité"),
            (page("", "", ", Button(name: B, text: \"b\")", "On(B.tap, effect: train.add(30s))"), "une heure se compte à la minute"),
            (page("", "", ", Button(name: B, text: \"b\")", "On(B.tap, effect: train.add(1.01h))"), "minute entière"),
            (page("", "", ", Button(name: B, text: \"b\")", "On(B.tap, effect: train.add(price))"), "nombre à virgule"),
            (page("", "", ", Button(name: B, text: \"b\")", "On(B.tap, effect: train.set(\"25:00\"))"), "elle reçoit une heure « HH:MM »"),
            (page("", "", ", Button(name: B, text: \"b\")", "On(B.tap, effect: train.mul(2))"), "« train » est une heure : on demande"),
            (page("", "", ", P(\"{train:duration}\")", ""), "« train » est un texte : pour l'heure, {train:time}"),
            (page("", "", ", P(\"{length:time}\")", ""), "montre une heure : « length » est un nombre"),
            (page("", "", ", P(\"{price:duration}\")", ""), "nombre à virgule"),
            (page("", "", ", P(\"{train:date}\")", ""), "« train » est une heure ; format possible : time"),
            (page("", "", ", P(\"{concert:number}\")", ""), "« concert » est un moment ; formats possibles"),
            (page("", "", ", If(train, over: \"2026-12-24\", children: [ P(\"x\") ])", ""), "« train » est une heure ; on la compare"),
            (page("", "", ", If(train, over: buyer, children: [ P(\"x\") ])", ""), "« train » est une heure et « buyer » un texte"),
        ] {
            let error = crate::check_page(&source).expect_err(&source);
            assert!(error.message.contains(message), "{source}\n→ {}", error.message);
        }
        // Permis : comparer à un texte vide, un moment à une date, une durée de la page.
        for source in [
            page("", "", ", If(train, is: \"\", children: [ P(\"x\") ]), If(now, over: \"2026-12-24\", children: [ P(\"y\") ]), P(\"{length:duration}\")", ""),
            page("", "", ", Button(name: B, text: \"b\")", "On(B.tap, effect: [train.add(length), train.add(1.5h), concert.sub(90min), train.set(now)])"),
        ] {
            assert!(crate::check_page(&source).is_ok(), "{source}\n→ {:?}", crate::check_page(&source).err());
        }
    }
}
