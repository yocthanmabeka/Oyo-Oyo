//! Travailler un texte (ADR-103) : le montrer en majuscules ou en minuscules, compter ses
//! caractères, le couper, le découper en liste.
//!
//! ```holo
//! Page(
//!   state: State(code: "", message: "", keywords: ""),
//!   computed: [ Split(name: tags, from: keywords, by: ",") ],
//!   children: [
//!     Input(value: code, label: "Code"),
//!     P("Code : {code:upper}"),
//!     Input(value: message, label: "Message", lines: 3, max: 140),
//!     P("{message:length} caractères sur 140. Aperçu : {message:max40}"),
//!     Input(value: keywords, label: "Mots-clés, séparés par des virgules"),
//!     Repeat(over: tags, children: [ Text("{item}") ]),
//!   ],
//! )
//! ```
//!
//! Un format ne change pas ce que le visiteur a écrit : seulement ce qu'on montre. Sa valeur
//! montrée voyage dans l'état sous son propre nom, `code:upper='ADA`, `message:length=12`, comme
//! les nombres de jours : la page l'écrit telle quelle, et l'arbitre ne la relit jamais.
//!
//! Un caractère est ce qu'une personne compte comme une lettre : une grappe de graphèmes
//! (UAX #29), comme `Intl.Segmenter` dans Chrome. « é » écrit en deux morceaux, 👍🏽, 🇫🇷 et
//! « क्षि » comptent chacun pour un. JavaScript compte autrement (`"👍".length` vaut 2), et un
//! texte coupé au hasard casse un émoji en deux : le moteur ne coupe jamais au milieu d'une lettre.

use crate::holo::{Block, Error, Program, Value};
use crate::state::Texts;

/// Les formats qui travaillent un texte, pour les messages : `{code:upper}`, `{code:lower}`,
/// `{message:length}`, et `{bio:max40}` (au plus 40 caractères).
pub const FORMATS: &[&str] = &["upper", "lower", "length", "max40"];

/// La coupe la plus courte et la plus longue : de `max2` à `max2000`.
pub const CUT_MIN: usize = 2;
pub const CUT_MAX: usize = crate::state::TEXT_MAX;

/// `max40` → 40 : au plus 40 caractères, « … » compris.
pub fn cut_length(format: &str) -> Option<usize> {
    let digits = format.strip_prefix("max")?;
    if digits.is_empty() || digits.starts_with('0') || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse::<usize>().ok().filter(|n| (CUT_MIN..=CUT_MAX).contains(n))
}

/// Ce format travaille-t-il un texte ?
pub fn is_format(format: &str) -> bool {
    matches!(format, "upper" | "lower" | "length") || cut_length(format).is_some()
}

/// Un format qui ressemble à ceux d'un texte, mal écrit : la raison, et le bon mot.
pub fn misspelled(name: &str, format: &str) -> Option<String> {
    let lowered = format.to_ascii_lowercase();
    let right = match lowered.as_str() {
        "uppercase" | "touppercase" | "upcase" | "caps" | "majuscules" => "upper",
        "lowercase" | "tolowercase" | "downcase" | "minuscules" => "lower",
        "len" | "size" | "count" | "chars" | "characters" | "longueur" => "length",
        _ => "",
    };
    if !right.is_empty() {
        return Some(format!("« {{{name}:{format}}} » : écris {{{name}:{right}}}"));
    }
    if lowered == "capitalize" || lowered == "title" {
        return Some(format!("« {{{name}:{format}}} » : une majuscule à chaque mot n'est pas un format ; pour l'allure, le style text-transform: capitalize"));
    }
    if let Some(digits) = lowered.strip_prefix("max").or_else(|| lowered.strip_prefix("truncate")).or_else(|| lowered.strip_prefix("cut")) {
        if digits.is_empty() || digits.bytes().all(|b| b.is_ascii_digit()) {
            return Some(format!("« {{{name}:{format}}} » : « max » suivi du nombre de caractères gardés, de {CUT_MIN} à {CUT_MAX}, « … » compris : {{{name}:max40}}"));
        }
    }
    None
}

/// Écrit `text` selon un format de texte, dans la langue de la page.
pub fn apply(text: &str, format: &str, language: &str) -> String {
    match format {
        "upper" => upper(text, language),
        "lower" => lower(text, language),
        "length" => count(text).to_string(),
        other => cut_length(other).map_or_else(|| text.to_string(), |max| cut(text, max)),
    }
}

// ---------------------------------------------------------------- les lettres d'un texte

/// La classe d'un caractère pour les règles d'UAX #29.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Other,
    Extend,
    SpacingMark,
    Control,
    Prepend,
    Pictographic,
    Regional,
    Linker,
    Consonant,
    Zwj,
    L,
    V,
    T,
    Lv,
    Lvt,
    Cr,
    Lf,
}

fn class(c: char) -> Class {
    let code = c as u32;
    match code {
        0x0D => return Class::Cr,
        0x0A => return Class::Lf,
        // Les syllabes coréennes toutes faites : LV quand elles n'ont pas de consonne finale.
        0xAC00..=0xD7A3 => return if (code - 0xAC00) % 28 == 0 { Class::Lv } else { Class::Lvt },
        _ => {}
    }
    let table = crate::graphemes::TABLE;
    let index = table.partition_point(|entry| entry >> 11 <= code);
    let Some(entry) = index.checked_sub(1).map(|i| table[i]) else { return Class::Other };
    if code > (entry >> 11) + ((entry >> 4) & 0x7F) {
        return Class::Other;
    }
    match entry & 0xF {
        1 => Class::Extend,
        2 => Class::SpacingMark,
        3 => Class::Control,
        4 => Class::Prepend,
        5 => Class::Pictographic,
        6 => Class::Regional,
        7 => Class::Linker,
        8 => Class::Consonant,
        9 => Class::Zwj,
        10 => Class::L,
        11 => Class::V,
        12 => Class::T,
        _ => Class::Other,
    }
}

/// Où en est une suite d'émoji joints (règle GB11) : une image, ses marques, puis un ZWJ.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Emoji {
    None,
    Picture,
    Joined,
}

/// Où en est une conjointe indienne (règle GB9c) : une consonne, puis un virama.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Conjunct {
    None,
    Consonant,
    Linked,
}

/// Le caractère `current` reste-t-il avec celui d'avant ? Les règles d'UAX #29, dans leur ordre.
fn joins(previous: Class, current: Class, regional_run: usize, emoji: Emoji, conjunct: Conjunct) -> bool {
    use Class::*;
    match (previous, current) {
        (Cr, Lf) => true,
        (Control | Cr | Lf, _) | (_, Control | Cr | Lf) => false,
        (L, L | V | Lv | Lvt) | (Lv | V, V | T) | (Lvt | T, T) => true,
        (_, Extend | Zwj | Linker | SpacingMark) => true,
        (Prepend, _) => true,
        (_, Consonant) if conjunct == Conjunct::Linked => true,
        (Zwj, Pictographic) => emoji == Emoji::Joined,
        // Deux drapeaux de suite : les lettres régionales vont par deux.
        (Regional, Regional) => regional_run % 2 == 1,
        _ => false,
    }
}

/// Où commence chaque caractère d'un texte, en octets.
pub fn starts(text: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut previous: Option<Class> = None;
    let (mut regional_run, mut emoji, mut conjunct) = (0usize, Emoji::None, Conjunct::None);
    for (i, c) in text.char_indices() {
        let current = class(c);
        if !previous.is_some_and(|p| joins(p, current, regional_run, emoji, conjunct)) {
            starts.push(i);
        }
        emoji = match current {
            Class::Pictographic => Emoji::Picture,
            Class::Extend | Class::Linker if emoji == Emoji::Picture => Emoji::Picture,
            Class::Zwj if emoji == Emoji::Picture => Emoji::Joined,
            _ => Emoji::None,
        };
        regional_run = if current == Class::Regional { regional_run + 1 } else { 0 };
        conjunct = match current {
            Class::Consonant => Conjunct::Consonant,
            Class::Linker if conjunct != Conjunct::None => Conjunct::Linked,
            Class::Extend | Class::Zwj if conjunct != Conjunct::None => conjunct,
            _ => Conjunct::None,
        };
        previous = Some(current);
    }
    starts
}

/// Le nombre de caractères d'un texte, comme une personne les compte.
pub fn count(text: &str) -> usize {
    starts(text).len()
}

/// Au plus `max` caractères (`max` ≥ 2), « … » compris. Un texte plus long est coupé à la fin
/// d'un mot si l'on garde ainsi au moins la moitié de la place, sinon au milieu d'un mot, mais
/// jamais au milieu d'une lettre ; puis « … » dit qu'il continue. Un texte assez court reste tel quel.
pub fn cut(text: &str, max: usize) -> String {
    let starts = starts(text);
    if starts.len() <= max {
        return text.to_string();
    }
    let keep = max.max(CUT_MIN) - 1;
    let space_at = |rank: usize| text[starts[rank]..].starts_with(char::is_whitespace);
    let end = (keep.div_ceil(2)..=keep).rev().find(|rank| *rank > 0 && space_at(*rank)).map_or(starts[keep], |rank| starts[rank]);
    let trimmed = text[..end].trim_end_matches(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | ':' | '-' | '–' | '—' | '(' | '[' | '«' | '“' | '‘' | '"' | '\''));
    let kept = if trimmed.is_empty() { &text[..starts[keep]] } else { trimmed };
    format!("{kept}…")
}

/// Le début d'un texte qui tient en `limit` caractères écrits (des points de code), sans couper
/// une lettre : la longueur d'un élément de liste (`ELEMENT_MAX`).
fn within(text: &str, limit: usize) -> &str {
    let bounds = starts(text);
    let (mut written, mut end) = (0, 0);
    for (rank, start) in bounds.iter().enumerate() {
        let next = bounds.get(rank + 1).copied().unwrap_or(text.len());
        written += text[*start..next].chars().count();
        if written > limit {
            break;
        }
        end = next;
    }
    &text[..end]
}

// ---------------------------------------------------------------- majuscules et minuscules

/// La langue, sans sa région : « tr-TR » → « tr ».
fn primary(language: &str) -> String {
    language.split(['-', '_']).next().unwrap_or("").to_ascii_lowercase()
}

/// En majuscules, selon la langue de la page : « ß » devient « SS » ; en turc et en azéri,
/// « i » devient « İ » ; en grec, les accents tombent (« Ελλάδα » → « ΕΛΛΑΔΑ »).
pub fn upper(text: &str, language: &str) -> String {
    match primary(language).as_str() {
        "tr" | "az" => text.replace('i', "İ").to_uppercase(),
        "el" => {
            let mut output = String::with_capacity(text.len());
            let mut after_greek = false;
            for c in text.chars() {
                let bare = match c {
                    'ά' | 'Ά' => 'α',
                    'έ' | 'Έ' => 'ε',
                    'ή' | 'Ή' => 'η',
                    'ί' | 'Ί' => 'ι',
                    'ό' | 'Ό' => 'ο',
                    'ύ' | 'Ύ' => 'υ',
                    'ώ' | 'Ώ' => 'ω',
                    'ΐ' => 'ϊ',
                    'ΰ' => 'ϋ',
                    // L'accent écrit à part, après une lettre grecque.
                    '\u{301}' if after_greek => continue,
                    other => other,
                };
                after_greek = matches!(bare, '\u{370}'..='\u{3FF}' | '\u{1F00}'..='\u{1FFF}');
                output.extend(bare.to_uppercase());
            }
            output
        }
        _ => text.to_uppercase(),
    }
}

/// En minuscules, selon la langue de la page : le sigma final du grec s'écrit « ς » ; en turc et
/// en azéri, « I » devient « ı » et « İ » devient « i ».
pub fn lower(text: &str, language: &str) -> String {
    match primary(language).as_str() {
        "tr" | "az" => text.replace("I\u{307}", "i").replace('İ', "i").replace('I', "ı").to_lowercase(),
        _ => text.to_lowercase(),
    }
}

// ---------------------------------------------------------------- les valeurs montrées

/// La langue de la page : `Page(lang:)`, le français sinon.
fn language_of(program: &Program) -> String {
    match program.root.argument("lang").map(|a| &a.value) {
        Some(Value::Text(language)) => language.clone(),
        _ => "fr".into(),
    }
}

/// Les textes travaillés que la page montre, chacun une fois : `{code:upper}` → (`code`, `upper`).
/// Seulement les valeurs de la page (pas `{item.title:upper}`, que la ligne écrit elle-même).
pub fn page_formats(program: &Program) -> Vec<(String, String)> {
    fn visit(value: &Value, found: &mut Vec<(String, String)>) {
        match value {
            Value::Text(text) => {
                for (name, format) in crate::format::formats_in(text) {
                    if is_format(format) && !found.iter().any(|(n, f)| n == name && f == format) {
                        found.push((name.to_string(), format.to_string()));
                    }
                }
            }
            Value::List(elements) => elements.iter().for_each(|e| visit(e, found)),
            Value::Block(block) => block.arguments.iter().for_each(|a| visit(&a.value, found)),
            _ => {}
        }
    }
    let mut found = Vec::new();
    program.root.arguments.iter().for_each(|a| visit(&a.value, &mut found));
    found
}

/// Les valeurs montrées des textes travaillés, pour ces textes : (`code:upper`, « ADA »). Un texte
/// que la page ne connaît pas ne montre rien.
pub fn shown_values(program: &Program, texts: &Texts) -> Vec<(String, String)> {
    let language = language_of(program);
    page_formats(program)
        .into_iter()
        .filter_map(|(name, format)| {
            let text = texts.iter().find(|(known, _)| *known == name).map(|(_, t)| t.as_str())?;
            Some((format!("{name}:{format}"), apply(text, &format, &language)))
        })
        .collect()
}

/// Les valeurs montrées, écrites comme l'état : `code:upper='ADA;message:length=12`. Un nombre de
/// caractères voyage comme un nombre ; un texte, codé, après une apostrophe.
pub fn written(program: &Program, texts: &Texts) -> String {
    shown_values(program, texts)
        .into_iter()
        .map(|(key, shown)| if key.ends_with(":length") { format!("{key}={shown}") } else { format!("{key}='{}", crate::state::encode(&shown)) })
        .collect::<Vec<_>>()
        .join(";")
}

/// Le texte montré d'une valeur travaillée, pour un texte sans balises (le titre de la page).
pub fn shown_in(name: &str, format: &str, texts: &Texts) -> Option<String> {
    if !is_format(format) {
        return None;
    }
    let text = texts.iter().find(|(known, _)| known == name).map(|(_, t)| t.as_str())?;
    Some(apply(text, format, &crate::format::language()))
}

// ---------------------------------------------------------------- découper en liste

/// Les réglages de `Split` (ADR-103).
pub const SPLIT_PARAMS: &[&str] = &["name", "from", "by"];

/// Le séparateur d'un `Split`.
#[derive(Debug, Clone, PartialEq)]
pub enum Separator {
    /// `by: " "` : les blancs, espaces, tabulations et retours à la ligne ; plusieurs n'en font qu'un.
    Spaces,
    /// `by: lines` : une ligne par élément.
    Lines,
    /// `by: ","` : les virgules de toutes les écritures (, ، 、 ， ﹐ ﹑ ､ ՝ ߸ ፣) ; `by: ";"`, les
    /// points-virgules (; ؛ ； ፤ ⁏) ; sinon le texte écrit, tel quel.
    Text(String),
}

const COMMAS: &[char] = &[',', '\u{60C}', '\u{3001}', '\u{FF0C}', '\u{FE50}', '\u{FE51}', '\u{FF64}', '\u{55D}', '\u{7F8}', '\u{1363}'];
const SEMICOLONS: &[char] = &[';', '\u{61B}', '\u{FF1B}', '\u{1364}', '\u{204F}', '\u{37E}'];

/// Lit le séparateur écrit : `by: ","`, `by: " "`, `by: lines`.
pub fn separator(value: &Value) -> Option<Separator> {
    match value {
        Value::Name(word) if word == "lines" => Some(Separator::Lines),
        Value::Text(text) if !text.is_empty() && text.chars().all(char::is_whitespace) && !text.contains(['\n', '\r']) => Some(Separator::Spaces),
        Value::Text(text) if !text.trim().is_empty() && text.chars().count() <= 10 && !text.contains(['\n', '\r']) => Some(Separator::Text(text.clone())),
        _ => None,
    }
}

/// Découpe un texte : chaque morceau sans ses blancs autour, les morceaux vides oubliés, deux
/// cents au plus, chacun dans la longueur d'un élément de liste.
pub fn split(text: &str, by: &Separator) -> Vec<String> {
    let pieces: Vec<&str> = match by {
        Separator::Spaces => text.split(char::is_whitespace).collect(),
        Separator::Lines => text.split(['\n', '\r', '\u{2028}', '\u{2029}', '\u{85}']).collect(),
        Separator::Text(sign) if sign == "," => text.split(COMMAS).collect(),
        Separator::Text(sign) if sign == ";" => text.split(SEMICOLONS).collect(),
        Separator::Text(sign) => text.split(sign.as_str()).collect(),
    };
    pieces
        .into_iter()
        .map(|piece| piece.chars().filter(|c| !c.is_control()).collect::<String>())
        .map(|piece| within(piece.trim(), crate::lists::ELEMENT_MAX).trim_end().to_string())
        .filter(|piece| !piece.is_empty())
        .take(crate::lists::ELEMENTS_MAX)
        .collect()
}

/// Un `Split` lu dans le fichier : (nom, texte de départ, séparateur).
pub fn read_split(block: &Block) -> (String, String, Option<Separator>) {
    let name = |param: &str| match block.argument(param).map(|a| &a.value) {
        Some(Value::Name(n)) => n.clone(),
        _ => String::new(),
    };
    (name("name"), name("from"), block.argument("by").and_then(|a| separator(&a.value)))
}

/// Vérifie un `Split`. `taken` : ce nom est-il déjà celui d'une valeur ?
pub fn check_split(program: &Program, block: &Block, taken: &dyn Fn(&str) -> bool) -> Result<String, Error> {
    let example = "Split(name: tags, from: keywords, by: \",\")";
    let error = |message: String| Err(Error { message, pos: block.pos });
    for a in &block.arguments {
        match a.name.as_deref() {
            Some(n) if SPLIT_PARAMS.contains(&n) => {}
            Some(n) => return Err(Error { message: format!("« Split » n'a pas de paramètre « {n} » ; paramètres possibles : {}", SPLIT_PARAMS.join(", ")), pos: a.pos }),
            None => return Err(Error { message: format!("chaque paramètre de « Split » est nommé : {example}"), pos: a.pos }),
        }
    }
    let name = match block.argument("name").map(|a| &a.value) {
        Some(Value::Name(n)) if n.starts_with(|c: char| c.is_ascii_lowercase()) && n.chars().all(|c| c.is_ascii_alphanumeric()) => n.clone(),
        _ => return error(format!("« Split » attend « name », le nom de la liste des morceaux, en minuscules : {example}")),
    };
    if taken(&name) {
        return error(format!("« {name} » est déjà le nom d'une valeur : la liste des morceaux a son propre nom"));
    }
    let texts = crate::state::initial_texts(program);
    match block.argument("from").map(|a| &a.value) {
        Some(Value::Name(from)) if texts.iter().any(|(t, _)| t == from) => {}
        Some(Value::Name(from)) if crate::lists::is_list(program, from) => return error(format!("« Split(from: {from}) » : « {from} » est déjà une liste ; on découpe un texte")),
        Some(Value::Name(from)) if crate::state::initial(program).unwrap_or_default().iter().any(|(n, _)| n == from) => {
            return error(format!("« Split(from: {from}) » : « {from} » est un nombre ; on découpe un texte, state: State({from}: \"\")"))
        }
        Some(Value::Name(from)) => return error(format!("« Split(from: {from}) » : aucun texte ne s'appelle « {from} » ; déclare-le, state: State({from}: \"\")")),
        _ => return error(format!("« Split » attend « from », le texte à découper : {example}")),
    }
    match block.argument("by") {
        Some(a) if separator(&a.value).is_some() => {}
        Some(a) => {
            return Err(Error {
                message: "« Split(by: …) » attend le séparateur entre guillemets, de 1 à 10 signes : by: \",\" (les virgules de toutes les écritures), by: \" \" (les blancs), ou by: lines (une ligne par morceau)".into(),
                pos: a.pos,
            })
        }
        None => return error(format!("« Split » attend « by », ce qui sépare les morceaux : {example}")),
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_are_counted_as_a_person_counts_them() {
        // Une lettre accentuée, écrite d'un seul signe ou en deux (e + accent) : une lettre.
        assert_eq!(count("é"), 1);
        assert_eq!(count("e\u{301}"), 1);
        // Un émoji, sa couleur de peau, une famille jointe par des ZWJ, un drapeau, l'Écosse.
        assert_eq!(count("👍"), 1);
        assert_eq!(count("👍🏽"), 1);
        assert_eq!(count("👨‍👩‍👧"), 1);
        assert_eq!(count("🇫🇷🇨🇩"), 2);
        assert_eq!(count("🇫🇷🇨"), 2);
        assert_eq!(count("🏴\u{E0067}\u{E0062}\u{E0073}\u{E0063}\u{E0074}\u{E007F}"), 1);
        assert_eq!(count("❤️"), 1);
        // Le devanagari : une conjointe et sa voyelle restent une lettre (Unicode 15.1) ; le tamoul,
        // le thaï, l'arabe avec ses voyelles, le coréen en jamos ou en syllabes.
        assert_eq!(count("क्षि"), 1);
        assert_eq!(count("नमस्ते"), 3);
        assert_eq!(count("தமிழ்"), 3);
        assert_eq!(count("กำ"), 1);
        assert_eq!(count("مَرْحَبًا"), 5);
        assert_eq!(count("한국어"), 3);
        assert_eq!(count("\u{1112}\u{1161}\u{11AB}"), 1);
        // Une fin de ligne Windows, des blancs, rien.
        assert_eq!(count("a\r\nb"), 3);
        assert_eq!(count("a b"), 3);
        assert_eq!(count(""), 0);
        assert_eq!(count("Bonjour à toi 👋"), 15);
    }

    #[test]
    fn a_text_is_written_in_capitals_or_small_letters_in_its_language() {
        assert_eq!(upper("Straße, été", "fr"), "STRASSE, ÉTÉ");
        assert_eq!(upper("istanbul ılık", "tr"), "İSTANBUL ILIK");
        assert_eq!(upper("istanbul", "en"), "ISTANBUL");
        assert_eq!(upper("Ελλάδα, Αθήνα", "el"), "ΕΛΛΑΔΑ, ΑΘΗΝΑ");
        assert_eq!(upper("Ελλα\u{301}δα", "el"), "ΕΛΛΑΔΑ");
        assert_eq!(lower("ΟΔΥΣΣΕΥΣ", "el"), "οδυσσευς");
        assert_eq!(lower("İSTANBUL, IŞIK", "tr"), "istanbul, ışık");
        assert_eq!(lower("ÉCOLE", "fr"), "école");
        assert_eq!(upper("ǆ ﬁ", "fr"), "Ǆ FI");
    }

    #[test]
    fn a_text_is_cut_at_the_end_of_a_word_never_inside_a_letter() {
        assert_eq!(cut("Bonjour à tous, voici mon tableau", 16), "Bonjour à tous…");
        assert_eq!(cut("Bonjour", 7), "Bonjour");
        assert_eq!(cut("Bonjour", 6), "Bonjo…");
        // Un mot trop long pour la moitié de la place : coupé au milieu.
        assert_eq!(cut("Anticonstitutionnellement", 10), "Anticonst…");
        // Un émoji ou un drapeau n'est jamais coupé en deux.
        assert_eq!(cut("👨‍👩‍👧👨‍👩‍👧👨‍👩‍👧", 2), "👨‍👩‍👧…");
        assert_eq!(cut("🇫🇷🇨🇩🇯🇵🇧🇷", 3), "🇫🇷🇨🇩…");
        assert_eq!(cut("e\u{301}te\u{301}", 2), "e\u{301}…");
        // Le chinois n'a pas d'espaces : coupé entre deux caractères.
        assert_eq!(cut("我爱巴黎的春天", 4), "我爱巴…");
        assert_eq!(count(&cut("Bonjour à tous, voici mon tableau", 16)), 15);
    }

    #[test]
    fn a_text_is_split_into_a_list() {
        assert_eq!(split(" art, peinture ,, Paris ,", &Separator::Text(",".into())), ["art", "peinture", "Paris"]);
        // Les virgules et les points-virgules de toutes les écritures.
        assert_eq!(split("北京，上海、广州", &Separator::Text(",".into())), ["北京", "上海", "广州"]);
        assert_eq!(split("قلم، كتاب", &Separator::Text(",".into())), ["قلم", "كتاب"]);
        assert_eq!(split("a; b؛c", &Separator::Text(";".into())), ["a", "b", "c"]);
        assert_eq!(split("un  deux\ttrois\nquatre", &Separator::Spaces), ["un", "deux", "trois", "quatre"]);
        assert_eq!(split("Farine\r\n\r\n Œufs \nLait", &Separator::Lines), ["Farine", "Œufs", "Lait"]);
        assert_eq!(split("a - b - c", &Separator::Text(" - ".into())), ["a", "b", "c"]);
        // Deux cents morceaux au plus, chacun dans la longueur d'un élément, sans couper une lettre.
        assert_eq!(split(&"x,".repeat(300), &Separator::Text(",".into())).len(), 200);
        let long = split(&"👍🏽".repeat(150), &Separator::Spaces);
        assert!(long[0].chars().count() <= 200 && count(&long[0]) == 100, "{}", long[0].chars().count());
        assert_eq!(separator(&Value::Text("".into())), None);
        assert_eq!(separator(&Value::Text("\n".into())), None);
        assert_eq!(separator(&Value::Text("  ".into())), Some(Separator::Spaces));
    }

    const PAGE: &str = r##"Page(
  title: "Code {code:upper}",
  state: State(code: "ab-12", message: "Bonjour à tous, voici mon premier tableau, peint au bord de la rivière.", keywords: "art, peinture, Paris", posts: []),
  computed: [
    Split(name: words, from: message, by: " "),
    Split(name: tags, from: keywords, by: ","),
    Filter(name: sorted, from: tags, sortBy: item),
  ],
  children: [
    Input(value: code, label: "Code"),
    P("Tel qu'on l'imprime : {code:upper} ({code:lower})"),
    Input(value: message, label: "Message", lines: 3, max: 140),
    P("{message:length} caractères sur 140, {words} mot(s)."),
    P("Aperçu : {message:max40}"),
    Button(name: Publish, text: "Publier"),
    Repeat(over: posts, children: [ P("{item:max20}") ]),
    Input(value: keywords, label: "Mots-clés"),
    P("{tags} étiquette(s)"),
    Repeat(over: tags, children: [ Text("#{item:lower}") ]),
  ],
  rules: [ On(Publish.tap, effect: posts.push(message)) ],
)"##;

    #[test]
    fn a_page_shows_a_worked_text_and_keeps_what_was_written() {
        let html = crate::flat_view(PAGE, "").unwrap();
        for expected in [
            "Tel qu'on l'imprime : <span data-state=\"code:upper\">AB-12</span> (<span data-state=\"code:lower\">ab-12</span>)",
            "<span data-state=\"message:length\">71</span> caractères sur 140, <span data-state=\"words\">13</span> mot(s).",
            "Aperçu : <span data-state=\"message:max40\">Bonjour à tous, voici mon premier…</span>",
            "<span data-state=\"tags\">3</span> étiquette(s)",
            "<div class=\"holo-Text\">#paris</div>",
            "data-title=\"Code AB-12\"",
        ] {
            assert!(html.contains(expected), "manque : {expected}\n{html}");
        }
        // L'état porte les valeurs montrées sous leur propre nom ; ce que le visiteur a écrit reste.
        let start = crate::initial_state(PAGE);
        assert!(start.contains("code='ab%2D12") && start.contains("code:upper='AB%2D12") && start.contains("message:length=71"), "{start}");
        assert!(start.contains("tags=[art,peinture,Paris]") && start.contains("sorted=[art,Paris,peinture]"), "{start}");
        // Le visiteur écrit : la valeur montrée suit, dans la langue de la page.
        let typed = crate::input(PAGE, &start, "code", "straße");
        assert!(typed.contains("code='stra%C3%9Fe") && typed.contains("code:upper='STRASSE"), "{typed}");
        let turkish = PAGE.replacen("Page(", "Page(lang: \"tr\", ", 1);
        assert!(crate::input(&turkish, &crate::initial_state(&turkish), "code", "istanbul").contains("code:upper='%C4%B0STANBUL"));
        // Un émoji, un drapeau, une lettre accentuée en deux morceaux : un caractère chacun.
        let emoji = crate::input(PAGE, &start, "message", "👍🏽🇫🇷e\u{301}");
        assert!(emoji.contains("message:length=3"), "{emoji}");
        // Les virgules du chinois découpent aussi.
        let chinese = crate::input(PAGE, &start, "keywords", "北京，上海、 广州,,");
        assert!(chinese.contains("tags=[%E5%8C%97%E4%BA%AC,%E4%B8%8A%E6%B5%B7,%E5%B9%BF%E5%B7%9E]"), "{chinese}");
        // Publier : la ligne montre l'aperçu coupé de l'élément, jamais relu de l'état.
        let published = crate::arbitrate(PAGE, &start, "Publish.tap");
        assert!(crate::list_html(PAGE, "", &published, "posts").contains("<p class=\"holo-P\">Bonjour à tous…</p>"), "{published}");
        let forged = format!("{start};code:upper='HACK;message:length=999");
        assert!(crate::input(PAGE, &forged, "code", "ok").contains("code:upper='OK") && !crate::input(PAGE, &forged, "code", "ok").contains("999"));
        assert_eq!(crate::page_title(PAGE, &typed), "Code STRASSE");
        // Un champ d'une fiche, travaillé lui aussi.
        let records = r#"Page(state: State(works: [ Item(title: "Le lever du soleil sur la rivière", price: 120) ]), children: [ Repeat(over: works, children: [ P("{item.title:max12} · {item.title:upper} · {item.title:length}") ]) ])"#;
        assert!(crate::flat_view(records, "").unwrap().contains("Le lever du… · LE LEVER DU SOLEIL SUR LA RIVIÈRE · 33"), "{}", crate::flat_view(records, "").unwrap());
    }

    #[test]
    fn what_a_worked_text_refuses() {
        for (wrong, message) in [
            (PAGE.replace("{code:upper}", "{posts:upper}"), "« posts » est une liste ; son nombre d'éléments s'écrit {posts}"),
            (PAGE.replace("{code:upper}", "{code:uppercase}"), "écris {code:upper}"),
            (PAGE.replace("{code:upper}", "{code:max1}"), "de 2 à 2000"),
            (PAGE.replace("{code:upper}", "{code:capitalize}"), "text-transform: capitalize"),
            (PAGE.replace("{code:upper}", "{item:upper}"), "n'a de sens que dans une répétition"),
            (PAGE.replace("{item:max20}", "{item:00}"), "les formats sont upper, lower, length et max40"),
            (PAGE.replace("by: \",\"", "by: \"\""), "attend le séparateur"),
            (PAGE.replace(", by: \",\"", ""), "attend « by »"),
            (PAGE.replace("from: keywords", "from: posts"), "est déjà une liste"),
            (PAGE.replace("from: keywords", "from: nothing"), "aucun texte ne s'appelle « nothing »"),
            (PAGE.replace("Split(name: tags", "Split(name: code"), "« code » est déjà le nom d'une valeur"),
            (PAGE.replace("by: \",\")", "by: \",\", trim: true)"), "n'a pas de paramètre « trim »"),
            (PAGE.replace("posts.push(message)", "tags.push(message)"), "« tags » est découpée dans le texte « keywords »"),
            (PAGE.replace("Repeat(over: tags,", "Repeat(over: tags, reorder: true,"), "découpée dans le texte « keywords »"),
            (PAGE.replace("{item:lower}", "{item.title}"), "« {item} » est le texte de l'élément"),
        ] {
            let error = crate::check_page(&wrong).unwrap_err();
            assert!(error.message.contains(message), "{wrong}\n→ {error}");
        }
        let numbers = "Page(state: State(cart: 2), children: [ P(\"{cart:upper}\") ])";
        assert!(crate::check_page(numbers).unwrap_err().message.contains("« cart » est un nombre"));
        let date = "Page(state: State(due: \"2026-12-24\"), children: [ P(\"{due:upper}\") ])";
        assert!(crate::check_page(date).unwrap_err().message.contains("est une date"));
    }

    #[test]
    fn the_formats_of_a_text() {
        assert!(is_format("upper") && is_format("lower") && is_format("length") && is_format("max40") && is_format("max2") && is_format("max2000"));
        assert!(!is_format("max1") && !is_format("max2001") && !is_format("max") && !is_format("max040") && !is_format("uppercase") && !is_format("00"));
        assert_eq!(apply("Ada 👋", "length", "fr"), "5");
        assert!(misspelled("code", "uppercase").unwrap().contains("{code:upper}"));
        assert!(misspelled("bio", "max1").unwrap().contains("de 2 à 2000"));
        assert!(misspelled("bio", "truncate40").unwrap().contains("{bio:max40}"));
        assert!(misspelled("n", "number").is_none());
    }
}
