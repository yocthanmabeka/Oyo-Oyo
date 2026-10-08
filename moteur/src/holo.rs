//! Lecture d'un fichier `.holo` : des blocs nommés par leur sens, imbriqués à la manière
//! de Flutter (ADR-009). Ce lecteur suit la grammaire brouillon de
//! `experiments/conformite-v0.1/README.md`. Il ne fait rien d'autre que lire : donner un
//! sens aux blocs est le travail de `univers.rs`.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Block(Block),
    List(Vec<Value>),
    Text(String),
    /// Un entier écrit sans point ni unité, gardé exact : une graine ne passe jamais par
    /// un nombre flottant (revue Codex : 2^53 + 1 devenait 2^53).
    Integer(u64),
    /// Un nombre à virgule, ou avec une unité. `places` : les chiffres écrits après la virgule
    /// (`12.50` en a deux) ; une valeur décimale garde cette précision (ADR-066).
    Number { value: f64, unit: Option<String>, places: u8 },
    Bool(bool),
    /// Un nom, éventuellement à points : `Atelier`, `Ouvrir.touche`, `auto`.
    Name(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: Option<String>,
    pub value: Value,
    pub pos: Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub name: String,
    /// Les noms de style posés sur le bloc : `card` dans `P.card(...)` (ADR-017), plusieurs
    /// depuis ADR-051 (`P.card.big(...)`). Un nom qui commence par une majuscule est la marque
    /// d'un composant, posée par le moteur sur la racine de chaque copie (ADR-050).
    pub styles: Vec<String>,
    pub arguments: Vec<Argument>,
    pub pos: Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// `import` (un autre fichier `.holo`) ou `module` (un module enfermé, ADR-045). Les ponts
    /// `bridge js` / `bridge css` ont été rejetés (ADR-011, partie B) : ils sont refusés.
    pub kind: String,
    pub target: String,
    pub pos: Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub imports: Vec<Import>,
    pub root: Block,
    /// Les styles, écrits comme en CSS après le bloc racine (ADR-017).
    pub styles: Vec<StyleRule>,
    /// Les noms des composants du fichier (ADR-050) : un style peut les viser, `ArticleCard { … }`.
    pub components: Vec<String>,
    /// Les valeurs qui viennent de l'adresse de la page (`profil/{id}.holo`, ADR-078) : la page
    /// les lit, sans pouvoir les changer.
    pub address: Vec<String>,
    /// Les valeurs partagées (`shared: Shared(seats: 20)`, ADR-079) : le serveur les garde pour
    /// tous les visiteurs ; elles rejoignent les valeurs de la page à la lecture.
    pub shared: Vec<String>,
}

/// Ce qu'un style vise : un type de bloc (`P`) ou un nom à point (`.card`). Rien d'autre.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Type(String),
    Name(String),
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Type(name) => write!(f, "{name}"),
            Target::Name(name) => write!(f, ".{name}"),
        }
    }
}

/// `color: gray;` La valeur est gardée telle qu'elle est écrite ; `styles.rs` la vérifie.
#[derive(Debug, Clone, PartialEq)]
pub struct Setting {
    pub name: String,
    pub value: String,
    pub pos: Pos,
}

/// `.card { color: gray; }`
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    pub target: Target,
    pub settings: Vec<Setting>,
    /// Les états du bloc : `hover: { … }`, `focus: { … }`, `active: { … }` (ADR-036).
    pub states: Vec<(String, Vec<Setting>, Pos)>,
    pub pos: Pos,
}

/// Les états qu'un style peut décrire : au survol, au focus du clavier, pendant l'appui ; puis
/// quand le visiteur a choisi le thème sombre, et sur un écran de téléphone (ADR-041).
pub const STATES: &[&str] = &["hover", "focus", "active", "dark", "phone", "computer", "narrow", "print"];

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub message: String,
    pub pos: Pos,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ligne {}, colonne {} : {}", self.pos.line, self.pos.column, self.message)
    }
}

impl Block {
    pub fn argument(&self, name: &str) -> Option<&Argument> {
        self.arguments.iter().find(|a| a.name.as_deref() == Some(name))
    }
}

/// La marque d'une valeur nommée dans une liste, `price: 0` : seuls les paramètres d'un
/// composant en ont (ADR-056).
pub const NAMED_VALUE: &str = "=";

/// Le nombre de noms de style qu'un bloc peut porter, au plus (ADR-051).
pub const STYLES_PER_BLOCK: usize = 4;

const UNITS: &[&str] = &["mm", "cm", "m", "km", "ms", "s", "min", "h", "B", "KB", "MB", "GB", "px", "deg"];

// ---------------------------------------------------------------- découpage en mots

#[derive(Debug, Clone, PartialEq)]
enum Word {
    Name(String),
    Integer(u64),
    Number(f64, Option<String>, u8),
    Text(String),
    Sign(char),
    End,
}

#[derive(Debug, Clone)]
struct Token {
    word: Word,
    pos: Pos,
}

struct Reader<'a> {
    src: &'a [u8],
    text: &'a str,
    i: usize,
    line: u32,
    line_start: usize,
}

impl<'a> Reader<'a> {
    fn pos(&self) -> Pos {
        Pos { line: self.line, column: (self.i - self.line_start) as u32 + 1 }
    }

    fn error(&self, message: impl Into<String>) -> Error {
        Error { message: message.into(), pos: self.pos() }
    }

    /// Avance d'un caractère entier (les lettres accentuées font plusieurs octets).
    fn advance(&mut self) {
        let c = self.src[self.i];
        if c == b'\n' {
            self.line += 1;
            self.line_start = self.i + 1;
        }
        self.i += match c {
            c if c < 0x80 => 1,
            c if c >= 0xF0 => 4,
            c if c >= 0xE0 => 3,
            _ => 2,
        };
    }

    /// Passe les blancs et les commentaires.
    fn skip_blanks(&mut self) {
        while self.i < self.src.len() {
            let c = self.src[self.i];
            if c == b'/' && self.src.get(self.i + 1) == Some(&b'/') {
                while self.i < self.src.len() && self.src[self.i] != b'\n' {
                    self.i += 1;
                }
            } else if c.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Découpe les imports et le bloc racine. S'arrête à la parenthèse qui referme le bloc
    /// racine : ce qui suit est fait de styles, lus par `styles`.
    fn tokens(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();
        let mut depth = 0i32;
        loop {
            self.skip_blanks();
            if self.i >= self.src.len() {
                tokens.push(Token { word: Word::End, pos: self.pos() });
                return Ok(tokens);
            }
            let pos = self.pos();
            let c = self.src[self.i];
            let word = if c.is_ascii_digit() || (c == b'-' && self.src.get(self.i + 1).is_some_and(|d| d.is_ascii_digit())) {
                self.number()?
            } else if c.is_ascii_alphabetic() || c == b'_' {
                let start = self.i;
                while self.i < self.src.len() && (self.src[self.i].is_ascii_alphanumeric() || self.src[self.i] == b'_' || self.src[self.i] == b'.') {
                    self.i += 1;
                }
                Word::Name(self.text[start..self.i].to_string())
            } else if c == b'"' {
                self.text()?
            } else if b"(),:[]".contains(&c) {
                self.i += 1;
                match c {
                    b'(' | b'[' => depth += 1,
                    b')' | b']' => depth -= 1,
                    _ => {}
                }
                if c == b')' && depth == 0 {
                    tokens.push(Token { word: Word::Sign(')'), pos });
                    tokens.push(Token { word: Word::End, pos: self.pos() });
                    return Ok(tokens);
                }
                Word::Sign(c as char)
            } else if c == b'{' || c == b'}' || c == b'=' || c == b'>' || c == b';' {
                return Err(self.error(format!(
                    "caractère « {} » : du code libre dans un bloc est interdit (ADR-015) ; un bloc ne contient que des valeurs",
                    c as char
                )));
            } else {
                let ch = self.text[self.i..].chars().next().unwrap_or('?');
                return Err(self.error(format!("caractère inattendu « {ch} »")));
            };
            tokens.push(Token { word, pos });
            if tokens.len() > TOKENS_MAX {
                return Err(Error { message: format!("fichier trop long : plus de {TOKENS_MAX} mots"), pos });
            }
        }
    }

    fn number(&mut self) -> Result<Word, Error> {
        let start = self.i;
        if self.src[self.i] == b'-' {
            self.i += 1;
        }
        while self.i < self.src.len() && (self.src[self.i].is_ascii_digit() || self.src[self.i] == b'.') {
            self.i += 1;
        }
        let number_text = &self.text[start..self.i];
        // Les chiffres écrits après la virgule : `12.50` en a deux, et les garde (ADR-066).
        let places = number_text.split_once('.').map_or(0, |(_, after)| after.len().min(255) as u8);
        let unit_start = self.i;
        while self.i < self.src.len() && self.src[self.i].is_ascii_alphabetic() {
            self.i += 1;
        }
        let unit = &self.text[unit_start..self.i];
        if unit.is_empty() {
            if let Ok(integer) = number_text.parse::<u64>() {
                return Ok(Word::Integer(integer));
            }
            let value: f64 = number_text.parse().map_err(|_| self.error("nombre mal formé"))?;
            return Ok(Word::Number(value, None, places));
        }
        let value: f64 = number_text.parse().map_err(|_| self.error("nombre mal formé"))?;
        if !UNITS.contains(&unit) {
            return Err(Error {
                message: format!("unité inconnue « {unit} » ; unités possibles : {}", UNITS.join(", ")),
                pos: Pos { line: self.line, column: (unit_start - self.line_start) as u32 + 1 },
            });
        }
        Ok(Word::Number(value, Some(unit.to_string()), places))
    }

    /// Lit les styles qui suivent le bloc racine. L'écriture est celle du CSS de base :
    /// `P { color: gray; }` ou `.card { border-radius: 8px; }` (ADR-017).
    fn styles(&mut self) -> Result<Vec<StyleRule>, Error> {
        let mut rules = Vec::new();
        loop {
            self.skip_blanks();
            if self.i >= self.src.len() {
                return Ok(rules);
            }
            let pos = self.pos();
            let named = self.src[self.i] == b'.';
            if named {
                self.i += 1;
            }
            let name = self.style_word(false);
            if name.is_empty() {
                return Err(Error {
                    message: "après le bloc racine viennent seulement des styles : un type de bloc (« P { … } ») ou un nom à point (« .card { … } »)".into(),
                    pos,
                });
            }
            let target = if named { Target::Name(name) } else { Target::Type(name) };
            self.skip_blanks();
            match self.src.get(self.i) {
                Some(b'{') => self.i += 1,
                Some(b'(') => {
                    return Err(Error {
                        message: "un seul bloc racine par fichier ; après lui viennent seulement des styles, écrits comme en CSS : « P { color: gray; } »".into(),
                        pos,
                    })
                }
                _ => {
                    return Err(self.error(format!(
                        "après « {target} », une accolade « {{ » est attendue : un style vise un type de bloc ou un nom à point, rien d'autre (ADR-017)"
                    )))
                }
            }
            let mut settings = Vec::new();
            let mut states = Vec::new();
            loop {
                self.skip_blanks();
                match self.src.get(self.i) {
                    None => return Err(Error { message: format!("le style « {target} » n'est jamais refermé : « }} » manquant"), pos }),
                    Some(b'}') => {
                        self.i += 1;
                        break;
                    }
                    _ => {}
                }
                let setting_pos = self.pos();
                let name = self.style_word(true);
                self.skip_blanks();
                if name.is_empty() || self.src.get(self.i) != Some(&b':') {
                    return Err(Error { message: "un réglage s'écrit « nom: valeur; », comme « color: gray; »".into(), pos: setting_pos });
                }
                self.i += 1;
                // Un état : `hover: { background: navy; }`. Ses réglages valent pendant cet état.
                while matches!(self.src.get(self.i), Some(b' ' | b'\t')) {
                    self.i += 1;
                }
                if self.src.get(self.i) == Some(&b'{') {
                    if !STATES.contains(&name.as_str()) {
                        return Err(Error { message: format!("« {name} » n'est pas un état ; un style décrit ces états : {} (ADR-036)", STATES.join(", ")), pos: setting_pos });
                    }
                    self.i += 1;
                    let mut inside = Vec::new();
                    loop {
                        self.skip_blanks();
                        match self.src.get(self.i) {
                            None => return Err(Error { message: format!("l'état « {name} » de « {target} » n'est jamais refermé : « }} » manquant"), pos: setting_pos }),
                            Some(b'}') => {
                                self.i += 1;
                                break;
                            }
                            _ => {}
                        }
                        let inner_pos = self.pos();
                        let setting = self.style_word(true);
                        self.skip_blanks();
                        if setting.is_empty() || self.src.get(self.i) != Some(&b':') {
                            return Err(Error { message: "dans un état, un réglage s'écrit « nom: valeur; », comme « background: navy; »".into(), pos: inner_pos });
                        }
                        self.i += 1;
                        let start = self.i;
                        while self.i < self.src.len() && !b";}\n{".contains(&self.src[self.i]) {
                            self.advance();
                        }
                        let value = self.text[start..self.i].trim().to_string();
                        match self.src.get(self.i) {
                            Some(b';') => self.i += 1,
                            Some(b'}') => {}
                            _ => return Err(Error { message: format!("« ; » manquant à la fin du réglage « {setting} »"), pos: inner_pos }),
                        }
                        if value.is_empty() {
                            return Err(Error { message: format!("le réglage « {setting} » n'a pas de valeur"), pos: inner_pos });
                        }
                        inside.push(Setting { name: setting, value, pos: inner_pos });
                    }
                    self.skip_blanks();
                    if self.src.get(self.i) == Some(&b';') {
                        self.i += 1;
                    }
                    states.push((name, inside, setting_pos));
                    continue;
                }
                let start = self.i;
                while self.i < self.src.len() && !b";}\n{".contains(&self.src[self.i]) {
                    self.advance();
                }
                let value = self.text[start..self.i].trim().to_string();
                match self.src.get(self.i) {
                    Some(b';') => self.i += 1,
                    Some(b'}') => {}
                    // En CSS, un « ; » oublié avale la ligne suivante sans rien dire.
                    _ => return Err(Error { message: format!("« ; » manquant à la fin du réglage « {name} »"), pos: setting_pos }),
                }
                if value.is_empty() {
                    return Err(Error { message: format!("le réglage « {name} » n'a pas de valeur"), pos: setting_pos });
                }
                settings.push(Setting { name, value, pos: setting_pos });
            }
            rules.push(StyleRule { target, settings, states, pos });
        }
    }

    /// Un nom dans un style : lettres, chiffres et `_` ; le tiret en plus pour un réglage (`font-size`).
    fn style_word(&mut self, dash: bool) -> String {
        let start = self.i;
        while self.i < self.src.len() && (self.src[self.i].is_ascii_alphanumeric() || self.src[self.i] == b'_' || (dash && self.src[self.i] == b'-')) {
            self.i += 1;
        }
        self.text[start..self.i].to_string()
    }

    fn text(&mut self) -> Result<Word, Error> {
        let pos = self.pos();
        let long = self.text[self.i..].starts_with("\"\"\"");
        let end: &str = if long { "\"\"\"" } else { "\"" };
        self.i += end.len();
        let start = self.i;
        loop {
            if self.i >= self.src.len() || (!long && self.src[self.i] == b'\n') {
                return Err(Error { message: "texte jamais refermé".into(), pos });
            }
            if self.text[self.i..].starts_with(end) {
                let content = self.text[start..self.i].to_string();
                for _ in 0..end.len() {
                    self.i += 1;
                }
                return Ok(Word::Text(if long { detach(&content) } else { content }));
            }
            self.advance();
        }
    }
}

/// Retire l'indentation commune d'un texte long, pour que le Markdown reste propre.
fn detach(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let margin = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut output: Vec<&str> = lines.iter().map(|l| if l.len() >= margin { &l[margin..] } else { l.trim_start() }).collect();
    while output.first().is_some_and(|l| l.trim().is_empty()) {
        output.remove(0);
    }
    while output.last().is_some_and(|l| l.trim().is_empty()) {
        output.pop();
    }
    output.join("\n")
}

// ---------------------------------------------------------------- analyse

struct Parser {
    tokens: Vec<Token>,
    i: usize,
    /// Combien de blocs et de listes sont ouverts les uns dans les autres, en ce moment.
    nesting: u32,
}

/// Un fichier `.holo` est un texte court. Ces trois limites s'appliquent avant toute analyse,
/// pour qu'un fichier hostile ne puisse ni remplir la mémoire ni faire déborder la pile
/// (revue Codex du 2026-10-03, B-09).
pub const BYTES_MAX: usize = 262_144;
pub const TOKENS_MAX: usize = 100_000;
pub const NESTING_MAX: u32 = 64;

impl Parser {
    fn current(&self) -> &Token {
        &self.tokens[self.i]
    }

    fn advance(&mut self) -> Token {
        let j = self.tokens[self.i].clone();
        if self.i + 1 < self.tokens.len() {
            self.i += 1;
        }
        j
    }

    fn is_sign(&self, c: char) -> bool {
        self.current().word == Word::Sign(c)
    }

    fn sign(&mut self, c: char) -> Result<(), Error> {
        if self.is_sign(c) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("« {c} » attendu, {} trouvé", describe(&self.current().word))))
        }
    }

    fn error(&self, message: String) -> Error {
        Error { message, pos: self.current().pos }
    }

    fn program(mut self) -> Result<Program, Error> {
        let mut imports = Vec::new();
        while let Word::Name(n) = &self.current().word {
            if n != "import" && n != "module" && n != "bridge" {
                break;
            }
            let pos = self.current().pos;
            let kind = n.clone();
            self.advance();
            if kind == "bridge" {
                return Err(Error {
                    message: "« bridge » est refusé : un pont ferait entrer du code sans garantie (ADR-011, partie B) ; pour du code venu d'ailleurs, un module enfermé : module \"calcul.wasm\" (ADR-045)".into(),
                    pos,
                });
            }
            match self.advance().word {
                Word::Text(target) => imports.push(Import { kind, target, pos }),
                _ => return Err(Error { message: format!("« {kind} » doit être suivi d'un texte entre guillemets"), pos }),
            }
        }
        let root = match &self.current().word {
            Word::Name(_) => self.block()?,
            Word::End => return Err(self.error("fichier vide : un bloc est attendu".into())),
            other => return Err(self.error(format!("un bloc est attendu, {} trouvé", describe(other)))),
        };
        if self.current().word != Word::End {
            return Err(self.error(format!("un seul bloc racine par fichier ; {} trouvé après lui", describe(&self.current().word))));
        }
        Ok(Program { imports, root, styles: Vec::new(), components: Vec::new(), address: Vec::new(), shared: Vec::new() })
    }

    fn block(&mut self) -> Result<Block, Error> {
        let token = self.advance();
        let name = match token.word {
            Word::Name(n) => n,
            other => return Err(Error { message: format!("nom de bloc attendu, {} trouvé", describe(&other)), pos: token.pos }),
        };
        // `cart.add(1)` : une demande faite à l'arbitre (ADR-023). Elle commence par une
        // minuscule et porte un point ; elle garde son nom entier. `blocs.rs` vérifie sa place.
        let is_requested = name.contains('.') && name.starts_with(|c: char| c.is_ascii_lowercase());
        // `P.card(...)` : le bloc `P`, avec le style nommé `card` (ADR-017) ; `P.card.big(...)`,
        // avec deux (ADR-051).
        let (name, styles) = match name.split_once('.') {
            Some((block, styles)) if !is_requested && !block.is_empty() && !styles.is_empty() => (block.to_string(), styles.split('.').map(str::to_string).collect::<Vec<_>>()),
            _ => (name, Vec::new()),
        };
        if styles.len() > STYLES_PER_BLOCK {
            return Err(Error { message: format!("« {name} » porte trop de noms de style : {STYLES_PER_BLOCK} au plus"), pos: token.pos });
        }
        for style in &styles {
            if style.is_empty() || !style.starts_with(|c: char| c.is_ascii_lowercase()) || !style.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return Err(Error { message: format!("« .{style} » : un nom de style s'écrit en minuscules, comme « card » ou « big-card » (ADR-037)"), pos: token.pos });
            }
        }
        if !is_requested && (!name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) || name.contains('.')) {
            // Une seule écriture par bloc : `h1` n'est pas accepté à côté de `H1` (ADR-020).
            let mut letters = name.chars();
            let message = match letters.next() {
                Some(c) if c.is_ascii_lowercase() && !name.contains('.') => {
                    format!("« {name} » : un nom de bloc commence par une majuscule, écris « {}{} »", c.to_ascii_uppercase(), letters.as_str())
                }
                _ => format!("« {name} » : un nom de bloc commence par une majuscule"),
            };
            return Err(Error { message, pos: token.pos });
        }
        self.sign('(')?;
        let mut arguments = Vec::new();
        while !self.is_sign(')') {
            let pos = self.current().pos;
            let arg_name = match (&self.current().word, self.tokens.get(self.i + 1)) {
                (Word::Name(n), Some(Token { word: Word::Sign(':'), .. })) => Some(n.clone()),
                _ => None,
            };
            if arg_name.is_some() {
                self.advance();
                self.advance();
            }
            let value = self.value()?;
            arguments.push(Argument { name: arg_name, value, pos });
            if self.is_sign(',') {
                self.advance();
            } else if !self.is_sign(')') {
                return Err(self.error(format!("« , » ou « ) » attendu, {} trouvé", describe(&self.current().word))));
            }
        }
        self.sign(')')?;
        Ok(Block { name, styles, arguments, pos: token.pos })
    }

    fn value(&mut self) -> Result<Value, Error> {
        self.nesting += 1;
        if self.nesting > NESTING_MAX {
            return Err(self.error(format!("trop de blocs et de listes les uns dans les autres : la limite est de {NESTING_MAX}")));
        }
        let value = self.simple_value();
        self.nesting -= 1;
        value
    }

    fn simple_value(&mut self) -> Result<Value, Error> {
        let next_is_paren = self.tokens.get(self.i + 1).is_some_and(|j| j.word == Word::Sign('('));
        let token = self.current().clone();
        match token.word {
            Word::Name(_) if next_is_paren => Ok(Value::Block(self.block()?)),
            Word::Name(n) => {
                self.advance();
                Ok(match n.as_str() {
                    "true" => Value::Bool(true),
                    "false" => Value::Bool(false),
                    _ => Value::Name(n),
                })
            }
            Word::Integer(integer) => {
                self.advance();
                Ok(Value::Integer(integer))
            }
            Word::Number(value, unit, places) => {
                self.advance();
                Ok(Value::Number { value, unit, places })
            }
            Word::Text(t) => {
                self.advance();
                Ok(Value::Text(t))
            }
            Word::Sign('[') => {
                self.advance();
                let mut elements = Vec::new();
                while !self.is_sign(']') {
                    // `price: 0` dans une liste : une valeur par défaut d'un paramètre de
                    // composant (ADR-056). Gardée comme un bloc marqué ; refusée ailleurs.
                    if let (Word::Name(name), Some(Token { word: Word::Sign(':'), .. })) = (&self.current().word, self.tokens.get(self.i + 1)) {
                        let (name, pos) = (name.clone(), self.current().pos);
                        self.advance();
                        self.advance();
                        let value = self.value()?;
                        elements.push(Value::Block(Block { name: NAMED_VALUE.into(), styles: Vec::new(), arguments: vec![Argument { name: Some(name), value, pos }], pos }));
                    } else {
                        elements.push(self.value()?);
                    }
                    if self.is_sign(',') {
                        self.advance();
                    } else if !self.is_sign(']') {
                        return Err(self.error(format!("« , » ou « ] » attendu, {} trouvé", describe(&self.current().word))));
                    }
                }
                self.sign(']')?;
                Ok(Value::List(elements))
            }
            other => Err(Error { message: format!("valeur attendue, {} trouvé", describe(&other)), pos: token.pos }),
        }
    }
}

fn describe(word: &Word) -> String {
    match word {
        Word::Name(n) => format!("« {n} »"),
        Word::Integer(v) => format!("« {v} »"),
        Word::Number(v, Some(u), _) => format!("« {v}{u} »"),
        Word::Number(v, None, _) => format!("« {v} »"),
        Word::Text(_) => "un texte".into(),
        Word::Sign(c) => format!("« {c} »"),
        Word::End => "la fin du fichier".into(),
    }
}

/// Lit un fichier `.holo` entier.
/// Ce qui sépare, dans le texte donné au moteur, un fichier de ceux qu'il importe : le fichier,
/// puis pour chaque import ce signe, son nom, `SEPARE_LE_NOM`, et son texte. C'est la page
/// d'entrée (ou le moteur en ligne de commande) qui va chercher les fichiers et les joint :
/// le moteur, lui, ne lit jamais rien tout seul.
pub const NEXT_FILE: char = '\u{1e}';
pub const NAME_SEPARATOR: char = '\u{1f}';

/// Le nombre de fichiers qu'une page peut importer, au plus.
pub const IMPORTS_MAX: usize = 16;

/// Un nom de fichier importé : rangé à côté, sans adresse complète ni remontée de dossier.
fn import_on(name: &str) -> bool {
    name.ends_with(".holo") && !name.starts_with('/') && !name.contains("..") && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
}

/// Les fichiers qu'un fichier importe (`import "commun.holo"`), pour que celui qui appelle le
/// moteur aille les chercher.
pub fn imports_of(source: &str) -> Result<Vec<String>, Error> {
    let main = source.split(NEXT_FILE).next().unwrap_or("");
    let program = read_single(main)?;
    let mut names = Vec::new();
    for import in program.imports.iter().filter(|i| i.kind == "import") {
        if !import_on(&import.target) {
            return Err(Error { message: format!("« import \"{}\" » : on importe un fichier .holo rangé à côté, comme \"commun.holo\"", import.target), pos: import.pos });
        }
        if !names.contains(&import.target) {
            names.push(import.target.clone());
        }
    }
    if names.len() > IMPORTS_MAX {
        return Err(Error { message: format!("trop d'imports : une page en fait au plus {IMPORTS_MAX}"), pos: program.root.pos });
    }
    Ok(names)
}

/// Lit un fichier et ceux qu'il importe, joints à sa suite. Un fichier importé est un morceau :
/// `Component(name: Menu, children: [...])`, avec ses styles. Dans la page, `Use(Menu)` pose les
/// blocs du morceau à cet endroit. Les styles du morceau viennent avec lui ; si la page écrit
/// le même style, c'est le sien qui reste.
pub fn read(source: &str) -> Result<Program, Error> {
    let mut files = source.split(NEXT_FILE);
    let mut program = read_single(files.next().unwrap_or(""))?;
    let provided: Vec<(&str, &str)> = files.filter_map(|f| f.split_once(NAME_SEPARATOR)).collect();
    let mut chunks: Vec<(String, Vec<Value>)> = Vec::new();
    let mut components: Vec<crate::components::Component> = Vec::new();
    let mut imported_styles: Vec<(String, StyleRule)> = Vec::new();
    for name in imports_of(source)? {
        let pos = program.imports.iter().find(|i| i.target == name).map_or(program.root.pos, |i| i.pos);
        let Some((_, text)) = provided.iter().find(|(provided, _)| *provided == name) else {
            return Err(Error { message: format!("le fichier importé « {name} » n'a pas été trouvé à côté de celui-ci"), pos });
        };
        // Un fichier qui ne contient que des styles (ADR-052) : un thème partagé par les pages.
        let chunk = match read_single(text) {
            Ok(chunk) => chunk,
            Err(e) => match read_single(&format!("Component(name: HoloStyles, children: []) {text}")) {
                Ok(styles) if !styles.styles.is_empty() && styles.imports.is_empty() => {
                    imported_styles.extend(styles.styles.into_iter().map(|r| (name.clone(), r)));
                    continue;
                }
                _ => return Err(Error { message: format!("dans « {name} », ligne {} : {}", e.pos.line, e.message), pos }),
            },
        };
        let refusal = |message: String| Error { message: format!("« {name} » : {message}"), pos };
        if chunk.root.name == "Part" {
            return Err(refusal(crate::components::OLD_PART.into()));
        }
        if chunk.root.name != "Component" {
            return Err(refusal(format!("un fichier importé est un composant, il commence par « Component(name: Menu, children: [ … ]) » ; celui-ci commence par « {} »", chunk.root.name)));
        }
        if !chunk.imports.is_empty() {
            return Err(refusal("un morceau n'importe pas lui-même d'autres fichiers".into()));
        }
        let component = crate::components::read_component(&chunk.root).map_err(|e| refusal(e.message))?;
        if chunks.iter().any(|(known, _)| *known == component.name) || components.iter().any(|c: &crate::components::Component| c.name == component.name) {
            return Err(refusal(format!("deux morceaux importés s'appellent « {} »", component.name)));
        }
        // Sans paramètres ni règles, un morceau se pose aussi par `Use(Menu)`, avec tous ses blocs.
        if component.params.is_empty() && component.rules.is_empty() {
            chunks.push((component.name.clone(), component.children.clone()));
        }
        components.push(component);
        imported_styles.extend(chunk.styles.into_iter().map(|r| (name.clone(), r)));
    }
    place_chunks(&mut program.root, &chunks)?;
    // Les composants (ADR-050) : ceux de la page, puis ceux des fichiers importés. Ils sont posés
    // avant les répétitions, pour qu'un composant puisse être répété.
    for component in crate::components::take_components(&mut program.root)? {
        if components.iter().any(|c| c.name == component.name) {
            return Err(Error { message: format!("deux composants s'appellent « {} »", component.name), pos: component.pos });
        }
        components.push(component);
    }
    crate::components::place_site(&mut program.root, &components)?;
    program.components = components.iter().map(|c| c.name.clone()).collect();
    if program.root.name == "Component" {
        if let Some(Value::Name(name)) = program.root.argument("name").map(|a| &a.value) {
            program.components.push(name.clone());
        }
    }
    // Les répétitions sont dépliées à leur tour, comme les morceaux (ADR-040).
    crate::format::set_language(match program.root.argument("lang").map(|a| &a.value) {
        Some(Value::Text(l)) => l,
        _ => "fr",
    });
    crate::repeat::unfold_site(&mut program.root, &mut 0)?;
    // Deux fichiers importés qui écrivent le même style se gêneraient : l'un gagnerait en silence.
    // C'est refusé, avec les deux noms (revue de Codex, PR 124). Un composant qui ne veut rien
    // partager se style par son nom, `ArticleCard { … }`, qui ne vise que ses copies.
    for (i, (file, rule)) in imported_styles.iter().enumerate() {
        if let Some((other, _)) = imported_styles[..i].iter().find(|(f, r)| f != file && r.target == rule.target) {
            let pos = program.imports.iter().find(|imp| imp.target == *file).map_or(program.root.pos, |imp| imp.pos);
            return Err(Error { message: format!("« {other} » et « {file} » écrivent tous deux le style « {} » : l'un effacerait l'autre ; renomme-le dans l'un des deux", rule.target), pos });
        }
    }
    // Les styles des morceaux d'abord, ceux de la page ensuite : à cible égale, la page garde le sien.
    imported_styles.retain(|(_, imported)| !program.styles.iter().any(|clean| clean.target == imported.target));
    let mut styles: Vec<StyleRule> = imported_styles.into_iter().map(|(_, r)| r).collect();
    styles.append(&mut program.styles);
    program.styles = styles;
    program.imports.retain(|i| i.kind != "import");
    // Les valeurs de l'adresse, jointes après la page et ses imports (ADR-078).
    if let Some((_, text)) = provided.iter().find(|(name, _)| *name == crate::address::ADDRESS_FILE) {
        crate::address::inject(&mut program, text)?;
    }
    // Les valeurs partagées rejoignent celles de la page, après celles de l'adresse (ADR-079).
    crate::shared::inject(&mut program)?;
    Ok(program)
}

/// Remplace chaque `Use(Menu)` par les blocs du morceau importé de ce nom.
fn place_chunks(block: &mut Block, chunks: &[(String, Vec<Value>)]) -> Result<(), Error> {
    fn inside(value: &mut Value, chunks: &[(String, Vec<Value>)]) -> Result<(), Error> {
        match value {
            Value::Block(block) => place_chunks(block, chunks),
            Value::List(elements) => {
                let mut placed_list = Vec::with_capacity(elements.len());
                for mut element in std::mem::take(elements) {
                    match &element {
                        Value::Block(call) if call.name == "Use" => {
                            let name = match call.arguments.as_slice() {
                                [Argument { name: None, value: Value::Name(name), .. }] => name,
                                _ => return Err(Error { message: "un morceau se pose par son nom : Use(Menu)".into(), pos: call.pos }),
                            };
                            let Some((_, children)) = chunks.iter().find(|(known, _)| known == name) else {
                                return Err(Error { message: format!("« Use({name}) » : aucun morceau importé ne s'appelle « {name} » ; importe son fichier en haut de la page, import \"commun.holo\""), pos: call.pos });
                            };
                            placed_list.extend(children.iter().cloned());
                        }
                        _ => {
                            inside(&mut element, chunks)?;
                            placed_list.push(element);
                        }
                    }
                }
                *elements = placed_list;
                Ok(())
            }
            _ => Ok(()),
        }
    }
    block.arguments.iter_mut().try_for_each(|a| inside(&mut a.value, chunks))
}

/// Lit un seul fichier, sans ses imports.
fn read_single(source: &str) -> Result<Program, Error> {
    if source.len() > BYTES_MAX {
        return Err(Error {
            message: format!("fichier trop gros : {} octets, la limite est de {BYTES_MAX}", source.len()),
            pos: Pos { line: 1, column: 1 },
        });
    }
    let mut reader = Reader { src: source.as_bytes(), text: source, i: 0, line: 1, line_start: 0 };
    let mut program = Parser { tokens: reader.tokens()?, i: 0, nesting: 0 }.program()?;
    program.styles = reader.styles()?;
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BIG_BANG: &str = include_str!("../mondes/big-bang.holo");

    #[test]
    fn reads_the_big_bang() {
        let p = read(BIG_BANG).unwrap();
        assert_eq!(p.root.name, "Point");
        assert_eq!(p.root.argument("name").unwrap().value, Value::Name("Origin".into()));
        assert_eq!(p.root.argument("seed").unwrap().value, Value::Integer(1));
        assert_eq!(p.root.argument("fragments").unwrap().value, Value::Integer(12));
        assert_eq!(p.root.argument("brightness").unwrap().value, Value::Number { value: 1.0, unit: None, places: 1 });
    }

    #[test]
    fn reads_nested_blocks_text_and_units() {
        let p = read_single(r#"
            import "buttons.holo"
            Page(
              title: "Ma boutique",
              children: [
                "Un paragraphe s'écrit tel quel.",
                Text("""
                  # Bienvenue
                  Voici **mes créations**.
                """),
                Point(name: Atelier, seed: 42, budget: 500KB),
              ],
              rules: [ On(Open.tap, effect: Atelier.enter) ],
            )
        "#)
        .unwrap();
        assert_eq!(p.imports[0].kind, "import");
        let content = match &p.root.argument("children").unwrap().value {
            Value::List(l) => l,
            _ => panic!(),
        };
        assert_eq!(content[0], Value::Text("Un paragraphe s'écrit tel quel.".into()));
        let content = &content[1..];
        match &content[0] {
            Value::Block(b) => assert_eq!(b.arguments[0].value, Value::Text("# Bienvenue\nVoici **mes créations**.".into())),
            _ => panic!(),
        }
        match &content[1] {
            Value::Block(b) => assert_eq!(b.argument("budget").unwrap().value, Value::Number { value: 500.0, unit: Some("KB".into()), places: 0 }),
            _ => panic!(),
        }
    }

    #[test]
    fn refuses_free_code_at_the_right_line() {
        let e = read("Page(\n  children: [\n    Button(name: Pay, on_tap: () { x = 1 }),\n  ],\n)").unwrap_err();
        assert_eq!(e.pos.line, 3);
        assert!(e.message.contains("ADR-015"), "{e}");
    }

    #[test]
    fn refuses_unknown_unit_and_open_text() {
        assert!(read("Point(seed: 3parsecs)").unwrap_err().message.contains("unité inconnue"));
        assert_eq!(read("Point(name: \"oups)").unwrap_err().message, "texte jamais refermé");
    }

    #[test]
    fn big_integers_stay_exact() {
        // Revue Codex : 9007199254740993 (2^53 + 1) devenait 9007199254740992 en passant par f64.
        let p = read("Point(seed: 9007199254740993, max: 18446744073709551615)").unwrap();
        assert_eq!(p.root.argument("seed").unwrap().value, Value::Integer(9_007_199_254_740_993));
        assert_eq!(p.root.argument("max").unwrap().value, Value::Integer(u64::MAX));
        assert!(read("Point(seed: 18446744073709551616)").is_ok(), "au-delà de u64, c'est un nombre flottant, refusé plus loin comme graine");
        assert!(read("Point(budget: 500 KB)").is_err(), "l'unité se colle au nombre : « 500 KB » n'est pas accepté");
    }

    #[test]
    fn a_hostile_file_is_stopped_before_parsing() {
        // Trop gros.
        let large = format!("Page(title: \"{}\")", "x".repeat(BYTES_MAX));
        assert!(read(&large).unwrap_err().message.contains("fichier trop gros"));
        // Trop de blocs les uns dans les autres : refusé, sans faire déborder la pile.
        let deep = format!("Page(children: {}{})", "[".repeat(200), "]".repeat(200));
        assert!(read(&deep).unwrap_err().message.contains("les uns dans les autres"));
        // Un fichier ordinaire, même bien rempli, passe.
        let wide = format!("Page(children: [{}])", "P(\"x\"), ".repeat(2000));
        assert!(read(&wide).is_ok());
    }

    #[test]
    fn two_imported_files_cannot_write_the_same_style() {
        let page = "import \"a.holo\"\nimport \"b.holo\"\nPage(children: [ Use(Alpha), Use(Beta) ])";
        let source = format!("{page}{s}a.holo{n}Component(name: Alpha, children: [ P.card(\"a\") ])\n.card {{ color: red; }}{s}b.holo{n}Component(name: Beta, children: [ P.card(\"b\") ])\n.card {{ color: blue; }}", s = NEXT_FILE, n = NAME_SEPARATOR);
        let error = read(&source).unwrap_err();
        assert!(error.message.contains("« a.holo » et « b.holo » écrivent tous deux le style « .card »"), "{error}");
        // La page, elle, peut toujours réécrire un style importé : c'est le sien qui reste.
        let source = format!("import \"a.holo\"\nPage(children: [ Use(Alpha) ])\n.card {{ color: green; }}{s}a.holo{n}Component(name: Alpha, children: [ P.card(\"a\") ])\n.card {{ color: red; }}", s = NEXT_FILE, n = NAME_SEPARATOR);
        assert_eq!(read(&source).unwrap().styles.len(), 1);
    }

    #[test]
    fn an_imported_file_is_a_chunk_we_place() {
        let common = "Component(name: Menu, children: [ P(\"menu\"), Hr() ])\nP { color: gray; }\nH1 { color: red; }";
        let page = "import \"commun.holo\"\nPage(children: [ Use(Menu), H1(\"a\"), List(children: [ Use(Menu) ]) ])\nH1 { color: blue; }";
        let joint = |page: &str, name: &str, text: &str| format!("{page}{NEXT_FILE}{name}{NAME_SEPARATOR}{text}");
        assert_eq!(imports_of(page).unwrap(), ["commun.holo"]);
        let program = read(&joint(page, "commun.holo", common)).unwrap();
        // Les blocs du morceau sont posés là où il est appelé, partout où il l'est.
        let Some(Value::List(children)) = program.root.argument("children").map(|a| &a.value) else { panic!() };
        let names: Vec<&str> = children.iter().map(|e| match e { Value::Block(b) => b.name.as_str(), _ => "?" }).collect();
        assert_eq!(names, ["P", "Hr", "H1", "List"]);
        // Les styles du morceau viennent avec lui ; à cible égale, la page garde le sien.
        let styles: Vec<String> = program.styles.iter().map(|r| format!("{} {}", r.target, r.settings[0].value)).collect();
        assert_eq!(styles, ["P gray", "H1 blue"]);
        assert!(program.imports.is_empty());
        for (source, message) in [
            (page.to_string(), "n'a pas été trouvé"),
            (joint(page, "commun.holo", "Page(children: [])"), "un fichier importé est un composant"),
            (joint(page, "commun.holo", "Component(name: Other, children: [])"), "aucun morceau importé ne s'appelle « Menu »"),
            (joint(page, "commun.holo", "Component(name: Menu)"), "un morceau a un nom et un contenu"),
            (joint(page, "commun.holo", "Component(name: Menu, children: [ H9( ])"), "dans « commun.holo », ligne 1"),
            (joint(page, "commun.holo", "import \"x.holo\"\nComponent(name: Menu, children: [])"), "n'importe pas lui-même"),
            ("Page(children: [ Use(Menu) ])".to_string(), "aucun morceau importé ne s'appelle « Menu »"),
            ("import \"../secret.holo\"\nPage(children: [])".to_string(), "rangé à côté"),
            ("import \"https://x.example/a.holo\"\nPage(children: [])".to_string(), "rangé à côté"),
        ] {
            let error = read(&source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn refuses_two_root_blocks() {
        assert!(read("Point(seed: 1) Point(seed: 2)").unwrap_err().message.contains("un seul bloc racine"));
    }
}
