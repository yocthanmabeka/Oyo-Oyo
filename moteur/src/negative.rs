//! Des nombres négatifs (ADR-102) : une température, un solde, une position, un écart. Une valeur
//! ne descend sous zéro que si la page le dit, `negative: [balance, temperature]` ; les autres
//! s'arrêtent à 0, comme avant : un panier ne compte jamais −1 tableau.
//!
//! ```holo
//! Page(
//!   state: State(temperature: -2),
//!   negative: [temperature],
//!   children: [ P("{temperature} °C"), Button(name: Colder, text: "Colder") ],
//!   rules: [ On(Colder.tap, effect: temperature.sub(5)) ],
//! )
//! ```
//!
//! Dans l'état, une valeur reste un nombre entier sans signe (`u64`) : un nombre négatif y est
//! gardé en complément à deux (−5 est gardé 2⁶⁴ − 5) et relu avec son signe (`as i64`). Une valeur
//! positive ne change pas : aucune ne dépasse 10¹⁵, bien loin de 2⁶³. L'état écrit porte le signe,
//! `temperature=-5`, et la page le montre avec le signe moins de sa langue.

use crate::holo::{Block, Error, Program, Value};
use crate::rules::for_each_block;
use crate::state::{self, VALUE_MAX};

/// Le réglage de la page : `negative: [balance]`.
pub const SETTING: &str = "negative";

/// Une valeur de l'état, lue avec son signe.
pub fn signed(units: u64) -> i64 {
    units as i64
}

/// Un nombre signé, gardé dans l'état.
pub fn stored(units: i64) -> u64 {
    units as u64
}

/// Les valeurs que la page laisse descendre sous zéro, dans l'ordre où elle les nomme.
pub fn names(program: &Program) -> Vec<String> {
    match program.root.argument(SETTING).map(|a| &a.value) {
        Some(Value::List(names)) => names
            .iter()
            .filter_map(|name| match name {
                Value::Name(name) => Some(name.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Cette valeur peut-elle descendre sous zéro ?
pub fn allowed(program: &Program, name: &str) -> bool {
    matches!(program.root.argument(SETTING).map(|a| &a.value), Some(Value::List(names)) if names.iter().any(|n| matches!(n, Value::Name(n) if n == name)))
}

/// La borne du langage, à l'échelle de la valeur : un milliard, d'un côté comme de l'autre de zéro.
pub fn limit(places: u32) -> i64 {
    (VALUE_MAX * state::scale(places)) as i64
}

/// Un nombre écrit dans le fichier, avec son signe, à l'échelle de `places` chiffres après la
/// virgule : `-2.5` → −250 pour deux chiffres. Rien s'il a plus de chiffres, une unité, ou s'il
/// dépasse un milliard.
pub fn literal(value: &Value, places: u32) -> Option<i64> {
    let units = match value {
        Value::Integer(n) => i64::try_from(*n).ok()?.checked_mul(state::scale(places) as i64)?,
        Value::Number { value, unit: None, places: written } if u32::from(*written) <= places && value.is_finite() => (value * state::scale(places) as f64).round() as i64,
        _ => return None,
    };
    (units.abs() <= limit(places)).then_some(units)
}

/// Un nombre écrit par le visiteur, reçu ou relu (« -12,5 », « −12.5 », « 7 ») : la valeur à
/// l'échelle de `places` chiffres, arrondie au plus proche, la moitié en s'éloignant de zéro, comme
/// sans le signe. Le signe moins s'écrit « - », ou « − » (celui qu'on copie d'une page suédoise).
/// Un nombre entier n'a pas de virgule. Rien si ce n'est pas un nombre.
pub fn parse(text: &str, places: u32) -> Option<i64> {
    let text = text.trim();
    let (negative, digits) = match text.strip_prefix('-').or_else(|| text.strip_prefix('\u{2212}')) {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    if digits.is_empty() || !digits.starts_with(|c: char| c.is_ascii_digit() || c == '.' || c == ',') || (places == 0 && !digits.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    let magnitude = i64::try_from(state::parse_decimal(digits, places)?).ok()?;
    Some(if negative { -magnitude } else { magnitude })
}

/// Range un nombre entre deux bornes, puis le garde dans l'état.
pub fn clamp(units: i128, (low, high): (i64, i64)) -> u64 {
    stored(units.max(i128::from(low)).min(i128::from(high.max(low))) as i64)
}

/// Les bornes d'une valeur qui peut être négative, pour les règles et les données : un milliard de
/// chaque côté de zéro ; le `max:` d'un champ qui la présente l'abaisse, comme pour toute valeur.
pub fn bounds(program: &Program, name: &str) -> (i64, i64) {
    let places = state::places(program, name);
    let mut high = limit(places);
    let _ = for_each_block(&program.root, &mut |block| {
        if presents(block, name) {
            if let Some(max) = block.argument("max").and_then(|a| literal(&a.value, places)) {
                high = high.min(max);
            }
        }
        Ok(())
    });
    (-limit(places), high)
}

/// Les bornes de ce qu'on écrit dans le champ : son `min:` relève le plancher, comme pour une
/// valeur qui ne descend pas sous zéro.
pub fn typed_bounds(program: &Program, name: &str) -> (i64, i64) {
    let places = state::places(program, name);
    let (mut low, high) = bounds(program, name);
    let _ = for_each_block(&program.root, &mut |block| {
        if presents(block, name) {
            if let Some(min) = block.argument("min").and_then(|a| literal(&a.value, places)) {
                low = low.max(min);
            }
        }
        Ok(())
    });
    (low, high)
}

/// Un champ de nombre qui présente cette valeur : `Input(value: temperature, …)`.
fn presents(block: &Block, name: &str) -> bool {
    block.name == "Input" && matches!(block.argument("value").map(|a| &a.value), Some(Value::Name(value)) if value == name)
}

/// Ce que le visiteur a écrit dans le champ d'une valeur qui peut être négative : la valeur
/// gardée, dans les bornes du champ ; rien si ce n'est pas un nombre. Un champ vide vaut 0.
pub fn typed(program: &Program, name: &str, written: &str) -> Option<u64> {
    let units = if written.trim().is_empty() { 0 } else { parse(written, state::places(program, name))? };
    Some(clamp(i128::from(units), typed_bounds(program, name)))
}

/// Un nombre de l'état écrit (`temperature=-5`), relu dans les bornes du langage.
pub fn reread(program: &Program, name: &str, written: &str) -> Option<u64> {
    let limit = limit(state::places(program, name));
    Some(clamp(i128::from(written.parse::<i64>().ok()?), (-limit, limit)))
}

/// Un nombre de l'état écrit, avec ou sans signe : « -250 », « 12 ». Rien sinon.
pub fn read_units(written: &str) -> Option<u64> {
    written.parse::<u64>().ok().or_else(|| written.parse::<i64>().ok().filter(|n| *n < 0).map(stored))
}

/// Une demande écrite avec un nombre négatif : `temperature.set(-5)`, `speed.mul(-1)`. Rend la
/// quantité gardée et les chiffres après la virgule du facteur, ou la raison du refus.
pub fn request(program: &Program, value: &str, verb: &str, number: f64, written: u32, places: u32) -> Result<(u64, u32), String> {
    let magnitude = format!("{:.prec$}", -number, prec = written as usize);
    match verb {
        "add" => return Err(format!("« {value}.add(-{magnitude}) » : pour retirer, écris {value}.sub({magnitude})")),
        "sub" => return Err(format!("« {value}.sub(-{magnitude}) » : pour ajouter, écris {value}.add({magnitude})")),
        "random" => return Err(format!("« {value}.random » attend le plus grand nombre possible, au moins 1 : {value}.random(100) tire de 0 à 100")),
        _ => {}
    }
    if !allowed(program, value) {
        return Err(format!("« {value}.{verb}(-{magnitude}) » : « {value} » ne descend pas sous zéro ; pour une valeur qui peut être négative, écris negative: [{value}] sur la page (ADR-102)"));
    }
    if verb == "set" {
        if written > places {
            let sort = if places == 0 { "est un nombre entier".to_string() } else { format!("a {places} chiffre(s) après la virgule") };
            return Err(format!("« {value} » {sort} : « {value}.set(…) » ne prend pas plus de chiffres après la virgule"));
        }
        let number = Value::Number { value: number, unit: None, places: written as u8 };
        return literal(&number, places).map(|units| (stored(units), 0)).ok_or_else(|| format!("« {value}.set(…) » attend un nombre de -{VALUE_MAX} à {VALUE_MAX}"));
    }
    // Multiplier ou diviser par un nombre négatif change le signe : speed.mul(-1).
    if written > state::PLACES_MAX || -number > VALUE_MAX as f64 {
        return Err(format!("« {value}.{verb}(…) » attend un facteur de -{VALUE_MAX} à {VALUE_MAX}, avec au plus {} chiffres après la virgule", state::PLACES_MAX));
    }
    Ok((stored((number * state::scale(written) as f64).round() as i64), written))
}

/// a ÷ b, arrondi au plus proche : la moitié s'éloigne de zéro (2,5 → 3 et −2,5 → −3), pour qu'un
/// nombre négatif s'arrondisse comme sans son signe. Pour deux nombres positifs, c'est l'arrondi
/// d'ADR-066 (la moitié vers le haut).
pub fn round_div(a: i128, b: i128) -> i128 {
    if b == 0 {
        return 0;
    }
    let magnitude = (a.unsigned_abs() + b.unsigned_abs() / 2) / b.unsigned_abs();
    let magnitude = i128::try_from(magnitude).unwrap_or(i128::MAX);
    if (a < 0) != (b < 0) { -magnitude } else { magnitude }
}

/// Les arguments d'un bloc qui nomment des valeurs : `value: temperature`, `values: [a, b]`.
fn named<'a>(value: &'a Value) -> Vec<&'a str> {
    match value {
        Value::Name(name) => vec![name.as_str()],
        Value::List(elements) => elements.iter().filter_map(|e| if let Value::Name(n) = e { Some(n.as_str()) } else { None }).collect(),
        _ => Vec::new(),
    }
}

/// Vérifie `negative: [ … ]`, puis que ces valeurs ne servent que là où un nombre négatif a un
/// sens : un texte (`{temperature}`), une condition, une règle qui guette, une demande, un champ de
/// nombre (`Input`), `keep`. Une glissière, une barre, une case, une place sur un plateau, un
/// dessin, un module, un fichier exporté, `limit:` ou l'adresse attendent un nombre qui ne descend
/// pas sous zéro : la page est refusée, avec la raison.
pub fn check(program: &Program) -> Result<(), Error> {
    let Some(argument) = program.root.argument(SETTING) else { return Ok(()) };
    let error = |message: String| Err(Error { message, pos: argument.pos });
    let Value::List(list) = &argument.value else {
        return error("« negative » attend la liste des valeurs qui peuvent descendre sous zéro : negative: [balance]".into());
    };
    let numbers = state::initial(program)?;
    let texts = state::initial_texts(program);
    let prices = state::price(program)?;
    let mut seen: Vec<&str> = Vec::new();
    for element in list {
        let Value::Name(name) = element else {
            return error("« negative » attend des noms de valeurs, sans guillemets : negative: [balance]".into());
        };
        let name = name.as_str();
        if seen.contains(&name) {
            return error(format!("« negative » : « {name} » est nommé deux fois"));
        }
        if state::CLOCK.contains(&name) || name == crate::dates::TODAY {
            return error(format!("« negative » : « {name} » est l'heure du visiteur, donnée par le moteur ; elle ne descend jamais sous zéro"));
        }
        if program.shared.iter().any(|shared| shared == name) {
            return error(format!("« negative » : « {name} » est une valeur partagée ; une valeur partagée ne descend pas encore sous zéro (ADR-102)"));
        }
        if texts.iter().any(|(known, _)| known == name) {
            return error(format!("« negative » : « {name} » est un texte ; seul un nombre peut être négatif"));
        }
        if crate::lists::is_list(program, name) {
            return error(format!("« negative » : « {name} » est une liste ; seul un nombre peut être négatif"));
        }
        if !numbers.iter().any(|(known, _)| known == name) {
            return error(format!("« negative » : aucune valeur ne s'appelle « {name} » ; déclare-la sur la page, state: State({name}: 0)"));
        }
        if prices.iter().any(|(known, _)| known == name) {
            return error(format!("« negative » : « {name} » a un prix ; une quantité d'articles ne descend pas sous zéro"));
        }
        seen.push(name);
    }
    for_each_block(&program.root, &mut |block| {
        // Une liste, un graphique, des dates ou un élément nomment des champs ou des listes, jamais
        // un nombre de la page ; un filtre, seulement par sa limite et son décalage.
        if matches!(block.name.as_str(), "Repeat" | "Chart" | "Days" | "Item") {
            return Ok(());
        }
        for (rank, argument) in block.arguments.iter().enumerate() {
            let param = argument.name.as_deref();
            if block.name == "Filter" && !matches!(param, Some("limit" | "offset")) {
                continue;
            }
            let Some(name) = named(&argument.value).into_iter().find(|name| seen.contains(name)) else { continue };
            let fine = match block.name.as_str() {
                "If" | "When" => rank == 0 || param.is_some_and(|p| state::COMPARISONS.contains(&p)),
                "Input" => param == Some("value"),
                "Page" => matches!(param, Some("keep" | SETTING)),
                _ => state::is_requested(block) && param.is_none(),
            };
            if !fine {
                let place = param.map_or_else(|| format!("« {}({name}) »", block.name), |p| format!("« {}({p}: {name}) »", block.name));
                return Err(Error {
                    message: format!("{place} : « {name} » peut descendre sous zéro (negative:) ; ici, il faut un nombre qui ne descend pas sous zéro. Un nombre négatif se montre dans un texte, se compare, se change par une demande et s'écrit dans un champ, Input(value: {name}) (ADR-102)"),
                    pos: argument.pos,
                });
            }
        }
        Ok(())
    })
}

/// Vérifie la borne d'un champ qui présente une valeur négative : `Input(value: temperature,
/// min: -50, max: 50)`.
pub fn check_bound(program: &Program, block: &Block, word: &str, bound: &Value, name: &str) -> Result<(), String> {
    let places = state::places(program, name);
    let Some(units) = literal(bound, places) else {
        return Err(format!("« Input({word}: …) » attend un nombre de -{VALUE_MAX} à {VALUE_MAX}, avec au plus {places} chiffre(s) après la virgule"));
    };
    if word == "min" {
        if let Some(max) = block.argument("max").and_then(|a| literal(&a.value, places)) {
            if units >= max {
                return Err("« Input » : min doit être plus petit que max".into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::state::{arbitrate, initial, initial_texts, input, reread, resume, write, Texts};

    const WEATHER: &str = "Page(
  title: \"Weather\",
  state: State(temperature: -2, gap: 0, cart: 0, balance: -12.50),
  negative: [temperature, gap, balance],
  keep: [temperature],
  children: [
    P(\"Up there: {temperature} °C, {gap:number}, {gap:00}, {balance} €, {cart} in the cart\"),
    Input(value: temperature, label: \"Temperature\", min: -50, max: 50),
    Input(value: balance, label: \"Balance\"),
    If(temperature, under: 0, children: [ P(\"It freezes.\") ], else: [ P(\"No frost.\") ]),
    If(temperature, under: -20, children: [ P(\"Big frost.\") ]),
    If(temperature, is: -7, children: [ P(\"Minus seven.\") ]),
    If(balance, over: -12.49, children: [ P(\"Almost.\") ]),
    Button(name: Colder, text: \"Colder\"), Button(name: Warmer, text: \"Warmer\"),
    Button(name: Freeze, text: \"Freeze\"), Button(name: Flip, text: \"Flip\"),
    Button(name: Half, text: \"Half\"), Button(name: Gap, text: \"Gap\"),
    Button(name: Spend, text: \"Spend\"), Button(name: Tip, text: \"Tip\"),
    Button(name: Cart, text: \"Cart\"), Button(name: Abyss, text: \"Abyss\"),
  ],
  rules: [
    On(Colder.tap, effect: temperature.sub(5)),
    On(Warmer.tap, effect: temperature.add(5)),
    On(Freeze.tap, effect: temperature.set(-10)),
    On(Flip.tap, effect: temperature.mul(-1)),
    On(Half.tap, effect: temperature.div(2)),
    On(Gap.tap, effect: [gap.set(cart), gap.sub(temperature)]),
    On(Spend.tap, effect: balance.sub(0.25)),
    On(Tip.tap, effect: balance.mul(1.1)),
    On(Cart.tap, effect: [cart.set(5), cart.add(temperature)]),
    On(Abyss.tap, effect: [temperature.sub(1000000000), temperature.sub(1000000000), gap.set(temperature)]),
  ],
)";

    fn page(source: &str) -> crate::holo::Program {
        crate::check_page(source).unwrap()
    }

    fn value(state: &crate::state::State, name: &str) -> i64 {
        super::signed(state.iter().find(|(known, _)| known == name).map(|(_, v)| *v).unwrap())
    }

    #[test]
    fn a_value_goes_below_zero_only_when_the_page_says_so() {
        let program = page(WEATHER);
        let texts: Texts = initial_texts(&program);
        let start = initial(&program).unwrap();
        assert_eq!(value(&start, "temperature"), -2);
        assert_eq!(value(&start, "balance"), -1250);
        let tap = |state: &crate::state::State, button: &str| arbitrate(&program, state, &texts, &format!("{button}.tap"));
        // −2 − 5 = −7 ; + 5 = −2 ; une valeur fixée sous zéro ; le signe changé ; la moitié.
        let colder = tap(&start, "Colder");
        assert_eq!(value(&colder, "temperature"), -7);
        assert_eq!(value(&tap(&colder, "Warmer"), "temperature"), -2);
        let frozen = tap(&start, "Freeze");
        assert_eq!(value(&frozen, "temperature"), -10);
        assert_eq!(value(&tap(&frozen, "Flip"), "temperature"), 10);
        // −7 ÷ 2 = −3, comme 7 ÷ 2 = 3 : la partie entière, sans regarder le signe.
        assert_eq!(value(&tap(&colder, "Half"), "temperature"), -3);
        // Un écart : 0 − (−7) = 7.
        assert_eq!(value(&tap(&colder, "Gap"), "gap"), 7);
        // Un nombre à virgule négatif reste exact : −12,50 − 0,25 = −12,75 ; × 1,1 = −14,025,
        // arrondi en s'éloignant de zéro : −14,03 (JavaScript donnerait −14,02 avec Math.round).
        let spent = tap(&start, "Spend");
        assert_eq!(value(&spent, "balance"), -1275);
        assert_eq!(value(&tap(&spent, "Tip"), "balance"), -1403);
        // Une valeur qui ne peut pas être négative s'arrête à 0, même quand on lui ajoute un nombre
        // négatif : 5 + (−7) = 0, jamais −2.
        assert_eq!(value(&tap(&colder, "Cart"), "cart"), 0);
        assert_eq!(value(&tap(&start, "Cart"), "cart"), 3);
        // La borne du langage : un milliard sous zéro.
        let abyss = tap(&start, "Abyss");
        assert_eq!(value(&abyss, "temperature"), -1_000_000_000);
        assert_eq!(value(&abyss, "gap"), -1_000_000_000);
        // L'état écrit porte le signe, et se relit.
        assert!(write(&colder).contains("temperature=-7"), "{}", write(&colder));
        assert_eq!(value(&reread(&program, &write(&colder)), "temperature"), -7);
        assert_eq!(value(&reread(&program, "temperature=-99999999999"), "temperature"), -1_000_000_000);
        // Une valeur gardée (keep) revient avec son signe.
        assert_eq!(value(&resume(&program, "temperature=-7"), "temperature"), -7);
    }

    #[test]
    fn a_negative_number_is_compared_and_shown_in_the_page_language() {
        let program = page(WEATHER);
        let texts = initial_texts(&program);
        let start = initial(&program).unwrap();
        let colder = arbitrate(&program, &start, &texts, "Colder.tap");
        let html = crate::flat_view(WEATHER, "").unwrap();
        // −2 : le signe moins du français (celui du CLDR, « - ») ; les formats suivent.
        assert!(html.contains("Up there: <span data-state=\"temperature\" data-format=\"d0\">-2</span> °C, <span data-state=\"gap\" data-format=\"number\">0</span>, <span data-state=\"gap\" data-format=\"00\">00</span>, <span data-state=\"balance\" data-format=\"d2\">-12,50</span> €, <span data-state=\"cart\">0</span>"), "{html}");
        let shown = |state: &crate::state::State, key: &str| crate::state::conditions(&program, &crate::state::to_show(&program, state), &texts).into_iter().find(|(k, _)| k == key).map(|(_, v)| v);
        assert_eq!(shown(&start, "temperature|under=0"), Some(true));
        assert_eq!(shown(&start, "temperature|under=-20"), Some(false));
        assert_eq!(shown(&colder, "temperature|is=-7"), Some(true));
        assert_eq!(shown(&start, "balance|over=-12.49"), Some(false));
        let spent = arbitrate(&program, &start, &texts, "Spend.tap");
        assert_eq!(shown(&spent, "balance|over=-12.49"), Some(false));
        let warmer = arbitrate(&program, &arbitrate(&program, &start, &texts, "Warmer.tap"), &texts, "Warmer.tap");
        assert_eq!(shown(&warmer, "temperature|under=0"), Some(false));
        // Les formats, dans plusieurs langues : le signe moins du suédois est « − » (U+2212) ; en
        // arabe, une marque de gauche à droite garde le signe devant le nombre.
        let minus_seven = super::stored(-7);
        assert_eq!(crate::format::format_value("t", minus_seven, "d0", "fr"), "-7");
        assert_eq!(crate::format::format_value("t", minus_seven, "d0", "sv"), "\u{2212}7");
        assert_eq!(crate::format::format_value("t", minus_seven, "d0", "ar"), "\u{200E}-7");
        assert_eq!(crate::format::format_value("t", minus_seven, "00", "fr"), "-07");
        assert_eq!(crate::format::format_value("n", super::stored(-1_234_567), "number", "fr"), "-1\u{202F}234\u{202F}567");
        assert_eq!(crate::format::format_value("n", super::stored(-123_450), "cents", "en"), "-1,234.50");
        assert_eq!(crate::format::format_value("n", super::stored(-5), "cents", "fr"), "-0,05");
        assert_eq!(crate::format::format_value("n", super::stored(-1250), "nd2", "de"), "-12,50");
        assert_eq!(crate::format::format_value("n", 0, "d2", "fr"), "0,00");
        // Dans une page suédoise, le moteur l'écrit aussi ; le titre de l'onglet suit.
        let swedish = WEATHER.replace("title: \"Weather\"", "title: \"{temperature} grader\", lang: \"sv\"");
        let html = crate::flat_view(&swedish, "").unwrap();
        assert!(html.contains("<span data-state=\"temperature\" data-format=\"d0\">\u{2212}2</span>") && html.contains("data-title=\"\u{2212}2 grader\""), "{html}");
    }

    #[test]
    fn a_field_takes_a_negative_number_with_a_keyboard_that_has_the_minus_sign() {
        let program = page(WEATHER);
        let texts = initial_texts(&program);
        let start = initial(&program).unwrap();
        let typed = |written: &str| value(&input(&program, &start, &texts, "temperature", written), "temperature");
        assert_eq!(typed("-12"), -12);
        // Le signe moins copié d'une page suédoise est compris ; les bornes du champ tiennent.
        assert_eq!(typed("\u{2212}3"), -3);
        assert_eq!(typed("-60"), -50);
        assert_eq!(typed("70"), 50);
        assert_eq!(typed(""), 0);
        // Ce qui n'est pas un nombre ne change rien ; un nombre entier n'a pas de virgule.
        assert_eq!(typed("froid"), -2);
        assert_eq!(typed("-2.5"), -2);
        assert_eq!(typed("--4"), -2);
        let balance = |written: &str| value(&input(&program, &start, &texts, "balance", written), "balance");
        assert_eq!(balance("-3,456"), -346);
        assert_eq!(balance("-0,004"), 0);
        // Le champ : un nombre, sans `inputmode` (le clavier du téléphone garde le signe moins),
        // et le plus petit nombre permis écrit pour le navigateur.
        let html = crate::flat_view(WEATHER, "").unwrap();
        assert!(html.contains("<input type=\"number\" min=\"-50\" max=\"50\" value=\"-2\" data-bind=\"temperature\">"), "{html}");
        assert!(html.contains("<input type=\"number\" min=\"-1000000000.00\" step=\"0.01\" data-places=\"2\" value=\"-12.50\" data-bind=\"balance\">"), "{html}");
    }

    #[test]
    fn data_forms_and_the_server_keep_the_sign() {
        let source = "Page(
  state: State(temperature: 0, count: 0, note: 0.0),
  negative: [temperature, note],
  data: Data(from: \"weather.json\"),
  children: [
    P(\"{temperature} {count} {note}\"),
    Form(name: Report, children: [ Input(value: temperature, label: \"Temperature\", min: -60), Button(name: Send, text: \"Send\") ]),
  ],
  rules: [ On(Send.tap, effect: Report.send) ],
)";
        let program = page(source);
        let texts = initial_texts(&program);
        let start = initial(&program).unwrap();
        // Des données reçues : un nombre négatif va dans une valeur qui peut l'être, jamais dans
        // une autre ; un nombre à virgule ne va pas dans un nombre entier.
        let (received, _) = crate::state::receive(&program, &start, &texts, "{\"temperature\": -3, \"count\": -4, \"note\": -2.25}");
        assert_eq!((value(&received, "temperature"), value(&received, "count"), value(&received, "note")), (-3, 0, -23));
        let (decimal, _) = crate::state::receive(&program, &start, &texts, "{\"temperature\": -3.5}");
        assert_eq!(value(&decimal, "temperature"), 0);
        // Un formulaire envoie le nombre avec son signe ; le serveur le vérifie à nouveau.
        let sent = crate::state::submission(&program, &received, &texts, "Report").unwrap();
        assert_eq!(sent, "{\"form\":\"Report\",\"values\":{\"temperature\":-3}}");
        assert!(crate::state::check_submission(&program, &sent).is_empty());
        let too_cold = crate::state::check_submission(&program, "{\"form\":\"Report\",\"values\":{\"temperature\":-61}}");
        assert_eq!(too_cold, vec![("temperature".to_string(), "Un nombre de -60 à 1000000000.".to_string())]);
        assert!(!crate::state::check_submission(&program, "{\"form\":\"Report\",\"values\":{\"temperature\":-1.5}}").is_empty());
        // Le serveur relit un nombre négatif de l'état d'une page.
        assert_eq!(super::read_units("-250"), Some(super::stored(-250)));
        assert_eq!(super::read_units("12"), Some(12));
        assert_eq!(super::read_units("-"), None);
        // Sans JavaScript, un toucher passe par le serveur : le même arbitre.
        let after = crate::visitor_gesture(source, "temperature=-3", &[("temperature".into(), "-8".into())]);
        assert!(after.contains("temperature=-8"), "{after}");
    }

    #[test]
    fn what_cannot_be_negative_is_refused_with_the_reason() {
        let refused = |source: &str, expected: &str| {
            let message = crate::check_page(source).map(|_| "accepté".to_string()).unwrap_or_else(|e| e.message);
            assert!(message.contains(expected), "{source}\n→ {message}");
        };
        refused("Page(state: State(t: -5), children: [])", "« t » commence sous zéro");
        refused("Page(state: State(t: -2.5), children: [])", "écris negative: [t]");
        refused("Page(state: State(t: 0), negative: t, children: [])", "« negative » attend la liste");
        refused("Page(state: State(t: 0), negative: [u], children: [])", "aucune valeur ne s'appelle « u »");
        refused("Page(state: State(t: 0), negative: [t, t], children: [])", "nommé deux fois");
        refused("Page(state: State(t: \"\"), negative: [t], children: [])", "est un texte");
        refused("Page(state: State(t: []), negative: [t], children: [])", "est une liste");
        refused("Page(state: State(t: 0), negative: [hour], children: [ P(\"{hour}\") ])", "l'heure du visiteur");
        refused("Page(state: State(t: 0), prices: Prices(t: 3), negative: [t], children: [])", "a un prix");
        refused("Page(shared: Shared(t: 0), negative: [t], children: [])", "valeur partagée");
        refused("Page(state: State(t: 0), negative: [t], children: [ Progress(value: t, label: \"x\") ])", "« Progress(value: t) » : « t » peut descendre sous zéro");
        refused("Page(state: State(t: 0), negative: [t], children: [ Slider(value: t, label: \"x\") ])", "« Slider(value: t) »");
        refused("Page(state: State(t: 0), negative: [t], children: [ Board(children: [ Shape(name: S, form: circle, x: t, y: 1) ]) ])", "« Shape(x: t) »");
        refused("Page(state: State(t: 0), negative: [t], address: [t], children: [])", "« Page(address: t) »");
        refused("Page(state: State(t: 0, cart: 0), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: cart.set(-1)) ])", "« cart » ne descend pas sous zéro");
        refused("Page(state: State(t: 0), negative: [t], children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: t.add(-5)) ])", "écris t.sub(5)");
        refused("Page(state: State(t: 0), negative: [t], children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: t.sub(-5)) ])", "écris t.add(5)");
        refused("Page(state: State(t: 0.0), negative: [t], children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: t.set(-2.55)) ])", "ne prend pas plus de chiffres");
        refused("Page(state: State(cart: 0), children: [ If(cart, under: -1, children: [ \"x\" ]) ])", "« cart » ne descend pas sous zéro");
        refused("Page(state: State(cart: 0), children: [ Input(value: cart, label: \"x\", min: -5) ])", "« cart » ne descend pas sous zéro");
        refused("Page(state: State(t: 0), negative: [t], children: [ Input(value: t, label: \"x\", min: 5, max: -5) ])", "min doit être plus petit que max");
        refused("Page(state: State(t: 0), negative: [t], children: [ P(\"{t:stopwatch}\") ])", "un temps de chronomètre ne descend pas sous zéro");
        // Ce qui est permis : un texte, une condition, une règle qui guette, des demandes, un champ, keep.
        crate::check_page("Page(state: State(t: -3, best: 0), negative: [t, best], keep: [t], children: [ P(\"{t}\"), Input(value: t, label: \"x\", min: -9, max: -1), If(t, over: best, children: [ \"x\" ]) ], rules: [ When(t, under: -5, effect: best.set(t)) ])").unwrap();
    }

    #[test]
    fn a_component_shows_and_changes_a_negative_value_it_is_given() {
        let source = "Page(
  state: State(t: -3),
  negative: [t],
  components: [ Component(name: Gauge, params: [level], children: [ Column(children: [ Text(\"{level} °C\"), Button(name: Down, text: \"Down\") ]) ], rules: [ On(Down.tap, effect: level.sub(2)) ]) ],
  children: [ Gauge(name: Peak, level: t) ],
)";
        let html = crate::flat_view(source, "").unwrap();
        assert!(html.contains("<span data-state=\"t\" data-format=\"d0\">-3</span> °C"), "{html}");
        let after = crate::arbitrate(source, &crate::initial_state(source), "DownPeak.tap");
        assert!(after.contains("t=-5"), "{after}");
    }
}
