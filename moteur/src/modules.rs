//! Les modules enfermés (ADR-011, partie C ; ADR-045) : du code compilé en WebAssembly, que le
//! moteur fait tourner dans une boîte fermée.
//!
//! ```holo
//! module "somme.wasm"
//! Page(
//!   state: State(n: 10, total: 0),
//!   modules: [ Module(name: Sum, source: "somme.wasm", input: n, output: total, time: 100ms, memory: 1MB) ],
//!   children: [ Button(name: Go, text: "Compute"), P("{total}") ],
//!   rules: [ On(Go.tap, effect: Sum.run), On(Sum.failed, effect: …) ],
//! )
//! ```
//!
//! La boîte : le module n'a accès à rien d'autre que ce que la page lui donne (ni réseau, ni
//! page, ni heure, ni stockage). Il tourne à part, sans jamais bloquer la page ; s'il dépasse son
//! temps (`time`), il est arrêté ; sa mémoire ne peut pas grandir au-delà de son plafond
//! (`memory`). Il se présente en holoscénique : une capacité (`run`) et deux signaux (`done`,
//! `failed`), jamais du Rust. Chaque module est annoncé en haut du fichier, `module "somme.wasm"`,
//! pour que le risque se lise d'un coup d'œil.
//!
//! Deux contrats (ADR-077) :
//!
//! - le premier (ADR-045) : `run(nombre) -> nombre`. Le module reçoit au plus un nombre de la
//!   page (`input: n`) et en rend un (`output: total`) ;
//! - le second : le module reçoit des valeurs de la page (`input: [notes, titre]`) et en rend
//!   (`output: [moyenne, avis]`) : des nombres, à virgule aussi, des textes, des listes. Elles
//!   voyagent en un texte JSON, `{"notes":[…],"titre":"…"}`. Le module offre `alloc(taille)`, qui
//!   dit où écrire ce texte dans sa mémoire, et `run(adresse, taille)`, qui rend l'adresse et la
//!   taille de sa réponse. Le moteur relit la réponse avec méfiance, comme des données venues
//!   d'un serveur : seulement les valeurs annoncées dans `output`, de la bonne sorte, dans leurs
//!   bornes, 64 Ko au plus ; sinon, `failed`.
//!
//! Un module venu d'ailleurs (ADR-118), écrit par quelqu'un d'autre :
//!
//! ```holo
//! module "somme.wasm"
//! Page(
//!   state: State(n: 10, total: 0),
//!   modules: [ Module(name: Sum, source: "somme.wasm", from: "https://exemple.org/somme-1.0.wasm",
//!                     sha256: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08", license: "MIT",
//!                     input: n, output: total) ],
//!   …
//! )
//! ```
//!
//! - `sha256` : l'empreinte du fichier, en 64 chiffres hexadécimaux, celle que donne son auteur.
//!   Le fichier est vérifié avant chaque lancement : par le navigateur lui-même (l'intégrité de
//!   `fetch`, qui marche aussi sur une adresse du réseau local, là où `crypto.subtle` manque), par
//!   `holo check`, et par `holo serve` avant de ranger une copie téléchargée. S'il diffère, il
//!   n'est pas lancé.
//! - `from` : l'adresse d'où il vient, en HTTPS, lue strictement (ADR-116), seulement avec son
//!   empreinte. Le navigateur du visiteur n'y va jamais : si la copie manque à côté de la page,
//!   `holo serve` la télécharge une fois, la vérifie et la range, puis la sert.
//! - `license` : sa licence, dite à l'auteur (`holo check`) et aux visiteurs, en bas de la page.
//! - La même boîte que les autres, et rien de plus : un fichier qui demande autre chose que sa
//!   mémoire (une fonction, une table, un autre module), qui fabrique sa propre mémoire, qui
//!   emploie des objets du navigateur hors de sa mémoire (WasmGC) ou une table sans plafond, est
//!   refusé avant de tourner (`check_wasm`). Un module n'en charge donc jamais un autre.

use crate::holo::{Block, Error, Program, Value};

/// Les bornes d'un module : son temps, et sa mémoire (en pages de 64 Ko, celles de WebAssembly).
pub const TIME_MIN: u64 = 10;
pub const TIME_MAX: u64 = 5_000;
pub const CURRENT_TIME: u64 = 100;
pub const PAGE: u64 = 65_536;
pub const PAGES_MAX: u64 = 256;
pub const CURRENT_PAGES: u64 = 16;
pub const MODULES_MAX: usize = 8;

/// Un module déclaré par la page.
#[derive(Debug, PartialEq)]
pub struct Module<'a> {
    pub name: &'a str,
    pub source: &'a str,
    /// Les valeurs de la page qu'il reçoit (aucune, une ou plusieurs).
    pub inputs: Vec<&'a str>,
    /// Les valeurs de la page qu'il rend (une ou plusieurs).
    pub outputs: Vec<&'a str>,
    pub time: u64,
    pub pages: u64,
    /// Un module venu d'ailleurs (ADR-118) : l'empreinte de son fichier (64 chiffres
    /// hexadécimaux), l'adresse d'où il vient, sa licence.
    pub sha256: Option<&'a str>,
    pub from: Option<&'a str>,
    pub license: Option<&'a str>,
}

/// Un fichier de module pèse 4 Mo au plus (ADR-118) : la page n'en lit pas plus, `holo serve`
/// n'en télécharge pas plus, et `holo check` refuse un fichier plus lourd.
pub const BYTES_MAX: usize = 4_000_000;
/// Une licence s'écrit en 64 caractères au plus, sur une ligne.
pub const LICENSE_MAX: usize = 64;

/// Une réponse de module, au plus (ADR-077) : celle des données d'un serveur.
pub const ANSWER_MAX: usize = crate::state::DATA_BYTES;
/// Les valeurs qu'un module reçoit, ou rend, au plus.
pub const VALUES_MAX: usize = 16;

/// La sorte d'une valeur de la page, pour un module.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sort {
    Number,
    Text,
    List,
}

/// La sorte d'une valeur que la page déclare ; rien pour un nom inconnu. Une liste calculée
/// (ADR-062) est une liste.
pub fn sort_of(program: &Program, name: &str) -> Option<Sort> {
    if crate::state::initial(program).unwrap_or_default().iter().any(|(known, _)| known == name) {
        Some(Sort::Number)
    } else if crate::state::initial_texts(program).iter().any(|(known, _)| known == name) {
        Some(Sort::Text)
    } else if crate::lists::is_list(program, name) {
        Some(Sort::List)
    } else {
        None
    }
}

impl Module<'_> {
    /// Le premier contrat suffit-il : au plus un nombre reçu, et un seul nombre rendu ?
    pub fn simple(&self, program: &Program) -> bool {
        let number = |name: &&str| sort_of(program, name) == Some(Sort::Number);
        self.inputs.len() <= 1 && self.inputs.iter().all(number) && self.outputs.len() == 1 && self.outputs.iter().all(number)
    }
}

fn example() -> &'static str {
    "modules: [ Module(name: Sum, source: \"somme.wasm\", input: n, output: total) ]"
}

/// Les modules de la page, vérifiés.
pub fn modules(program: &Program) -> Result<Vec<Module<'_>>, Error> {
    let announcements: Vec<&crate::holo::Import> = program.imports.iter().filter(|i| i.kind == "module").collect();
    let Some(argument) = program.root.argument("modules") else {
        if let Some(announcement) = announcements.first() {
            return Err(Error { message: format!("« module \"{}\" » est annoncé en haut du fichier, mais la page ne le déclare pas : {}", announcement.target, example()), pos: announcement.pos });
        }
        return Ok(Vec::new());
    };
    let Value::List(list) = &argument.value else {
        return Err(Error { message: format!("« modules » est une liste : {}", example()), pos: argument.pos });
    };
    if list.len() > MODULES_MAX {
        return Err(Error { message: format!("une page fait tourner au plus {MODULES_MAX} modules"), pos: argument.pos });
    }
    let mut read_ones = Vec::new();
    for value in list {
        let Value::Block(block) = value else {
            return Err(Error { message: format!("« modules » contient des « Module(…) » : {}", example()), pos: argument.pos });
        };
        if block.name != "Module" {
            return Err(Error { message: format!("« modules » contient des « Module(…) », pas des « {} »", block.name), pos: block.pos });
        }
        read_ones.push(a_module(program, block)?);
    }
    // Chaque module se lit en haut du fichier ; chaque annonce a son module.
    for module in &read_ones {
        if !announcements.iter().any(|a| a.target == module.source) {
            return Err(Error { message: format!("le module « {} » doit être annoncé en haut du fichier, pour que le risque se lise d'un coup d'œil : module \"{}\"", module.name, module.source), pos: argument.pos });
        }
    }
    for announcement in &announcements {
        if !read_ones.iter().any(|m| m.source == announcement.target) {
            return Err(Error { message: format!("« module \"{}\" » est annoncé, mais aucun Module ne l'emploie", announcement.target), pos: announcement.pos });
        }
    }
    // Un même fichier a une seule empreinte et une seule adresse (ADR-118).
    for (i, module) in read_ones.iter().enumerate() {
        if let Some(other) = read_ones[..i].iter().find(|m| m.source == module.source && (m.sha256.map(str::to_ascii_lowercase) != module.sha256.map(str::to_ascii_lowercase) || m.from != module.from)) {
            return Err(Error { message: format!("« {} » et « {} » emploient le même fichier, « {} », avec une autre empreinte ou une autre adresse : un fichier n'a qu'une empreinte", other.name, module.name, module.source), pos: argument.pos });
        }
    }
    Ok(read_ones)
}

/// L'empreinte d'un module (64 chiffres hexadécimaux), écrite pour l'intégrité de `fetch` (SRI du
/// W3C) : `sha256-` puis les 32 octets en base64. Le navigateur vérifie lui-même le fichier avec
/// elle, et refuse de le donner s'il diffère ; il le fait aussi dans un contexte non sûr (une
/// page sur `http://192.168.…`), où `crypto.subtle` n'existe pas. Vide pour une empreinte mal
/// écrite : le moteur l'a déjà refusée (`a_module`).
pub fn integrity(sha256: &str) -> String {
    const LETTERS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    if sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return String::new();
    }
    let bytes: Vec<u8> = (0..32).map(|i| u8::from_str_radix(&sha256[2 * i..2 * i + 2], 16).unwrap_or(0)).collect();
    let mut out = String::from("sha256-");
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8) | u32::from(*chunk.get(2).unwrap_or(&0));
        for (k, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            out.push(if k <= chunk.len() { LETTERS[(n >> shift) as usize & 63] as char } else { '=' });
        }
    }
    out
}

/// Ce qu'un fichier de module offre (ADR-118) : le premier contrat (`run(nombre)`, ADR-045), ou
/// le second (`alloc`, puis `run(adresse, taille)`, ADR-077).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Offer {
    First,
    Second,
}

/// Les éléments des tables d'un module, au plus. Une table sert aux appels indirects du code (Rust
/// en fait une petite) ; sans plafond, elle pourrait grandir hors de la boîte.
pub const TABLE_ENTRIES_MAX: u64 = 100_000;

/// Un fichier coupé, ou qui ne se lit pas comme un module.
const CUT: &str = "le fichier du module est coupé ou abîmé : ce n'est pas un module WebAssembly entier";
/// Des objets gérés par le navigateur (WasmGC) : ils vivent hors de la mémoire plafonnée.
const MANAGED: &str = "le module emploie des objets gérés par le navigateur (WasmGC), qui vivraient hors de sa mémoire plafonnée : la boîte ne les permet pas";

/// Lit un fichier WebAssembly, octet par octet : juste assez pour savoir ce qu'il demande.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn byte(&mut self) -> Result<u8, String> {
        let byte = *self.bytes.get(self.at).ok_or(CUT)?;
        self.at += 1;
        Ok(byte)
    }

    /// Un nombre écrit en LEB128, comme WebAssembly les écrit : 32 bits au plus.
    fn number(&mut self) -> Result<u32, String> {
        let mut value: u64 = 0;
        for shift in (0..35).step_by(7) {
            let byte = self.byte()?;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return u32::try_from(value).map_err(|_| CUT.to_string());
            }
        }
        Err(CUT.into())
    }

    /// Un nom (celui d'une importation, d'une exportation), coupé à 64 caractères pour les messages.
    fn name(&mut self) -> Result<String, String> {
        let length = self.number()? as usize;
        let end = self.at.checked_add(length).filter(|end| *end <= self.bytes.len()).ok_or(CUT)?;
        let name = String::from_utf8_lossy(&self.bytes[self.at..end]).chars().filter(|c| !c.is_control()).take(64).collect();
        self.at = end;
        Ok(name)
    }

    /// Un type de valeur : un nombre (entier, à virgule, vecteur), ou une référence simple à une
    /// fonction ou à une valeur de l'hôte. Tout autre type est celui des objets gérés (WasmGC).
    fn value_type(&mut self) -> Result<(), String> {
        match self.byte()? {
            0x7f | 0x7e | 0x7d | 0x7c | 0x7b | 0x70 | 0x6f => Ok(()),
            _ => Err(MANAGED.into()),
        }
    }

    /// Des bornes (une mémoire, une table) : le minimum, et le maximum s'il est écrit. Une mémoire
    /// partagée, de 64 bits ou à pages d'une autre taille n'est pas celle que donne la boîte.
    fn limits(&mut self) -> Result<(u32, Option<u32>), String> {
        match self.byte()? {
            0x00 => Ok((self.number()?, None)),
            0x01 => {
                let least = self.number()?;
                Ok((least, Some(self.number()?)))
            }
            _ => Err("le module demande une mémoire ou une table partagée, de 64 bits ou à pages d'une autre taille : la boîte ne donne qu'une mémoire simple".into()),
        }
    }
}

/// Lit un fichier de module avant de le lancer (ADR-118). La boîte ne donne qu'une mémoire de
/// `pages` pages de 64 Ko, plafonnée, et rien d'autre ; le fichier ne doit rien demander de plus.
/// Il est refusé, avec la raison :
/// - si ce n'est pas un module WebAssembly entier ;
/// - s'il importe autre chose que `env.memory` : une fonction (le réseau, la page, l'heure, un
///   autre module…), une table, une valeur globale ; ou s'il demande une mémoire plus grande que
///   son plafond ;
/// - s'il fabrique sa propre mémoire : elle échapperait à son plafond ;
/// - s'il emploie des objets gérés par le navigateur (WasmGC), hors de sa mémoire ;
/// - s'il a une table sans plafond, ou de plus de 100 000 éléments ;
/// - s'il n'offre pas `run`, ou une fonction `run` d'aucun des deux contrats.
///
/// Les mêmes règles partout : dans la page (avant le fil à part), dans `holo check`, et dans
/// `holo serve` avant de ranger une copie téléchargée.
pub fn check_wasm(bytes: &[u8], pages: u64) -> Result<Offer, String> {
    if bytes.get(..8) != Some(b"\0asm\x01\0\0\0".as_slice()) {
        return Err("ce fichier n'est pas un module WebAssembly".into());
    }
    let mut reader = Reader { bytes, at: 8 };
    // Les fonctions du module : le type de chacune ; chaque type : son nombre de paramètres.
    let (mut types, mut functions, mut exports) = (Vec::new(), Vec::new(), Vec::new());
    let (mut memory, mut table_entries) = (false, 0_u64);
    while reader.at < bytes.len() {
        let id = reader.byte()?;
        let size = reader.number()? as usize;
        let end = reader.at.checked_add(size).filter(|end| *end <= bytes.len()).ok_or(CUT)?;
        let mut section = Reader { bytes: &bytes[..end], at: reader.at };
        match id {
            // Les noms, les outils qui l'ont fabriqué… : sans effet sur ce qu'il peut faire.
            0 => {}
            1 => {
                for _ in 0..section.number()? {
                    if section.byte()? != 0x60 {
                        return Err(MANAGED.into());
                    }
                    let parameters = section.number()?;
                    for _ in 0..parameters {
                        section.value_type()?;
                    }
                    for _ in 0..section.number()? {
                        section.value_type()?;
                    }
                    types.push(parameters);
                }
            }
            2 => {
                for _ in 0..section.number()? {
                    let (from, name, kind) = (section.name()?, section.name()?, section.byte()?);
                    if kind != 0x02 || from != "env" || name != "memory" || memory {
                        let what = match kind {
                            0x00 => "une fonction",
                            0x01 => "une table",
                            0x02 => "une seconde mémoire",
                            0x03 => "une valeur globale",
                            _ => "une chose inconnue",
                        };
                        return Err(format!("le module demande « {from}.{name} » ({what}) : la boîte ne donne rien d'autre que sa mémoire, ni le réseau, ni la page, ni un autre module"));
                    }
                    let (least, most) = section.limits()?;
                    if u64::from(least) > pages {
                        return Err(format!("le module demande au moins {least} pages de 64 Ko de mémoire, plus que son plafond ({pages}) : agrandis memory, 16MB au plus"));
                    }
                    if most.is_some_and(|most| u64::from(most) < pages) {
                        return Err(format!("le module accepte au plus {} pages de 64 Ko de mémoire, moins que la boîte ne lui en donne ({pages}) : réduis memory", most.unwrap_or(0)));
                    }
                    memory = true;
                }
            }
            3 => {
                for _ in 0..section.number()? {
                    functions.push(section.number()? as usize);
                }
            }
            4 => {
                for _ in 0..section.number()? {
                    if !matches!(section.byte()?, 0x70 | 0x6f) {
                        return Err(MANAGED.into());
                    }
                    match section.limits()? {
                        (_, Some(most)) => table_entries += u64::from(most),
                        (_, None) => return Err("le module a une table sans plafond, qui pourrait grandir hors de la boîte".into()),
                    }
                }
                if table_entries > TABLE_ENTRIES_MAX {
                    return Err(format!("les tables du module dépassent {TABLE_ENTRIES_MAX} éléments"));
                }
            }
            5 => {
                if section.number()? > 0 {
                    return Err("le module fabrique sa propre mémoire, qui échapperait à son plafond : il doit employer celle que le moteur lui donne (compilé avec --import-memory)".into());
                }
            }
            7 => {
                for _ in 0..section.number()? {
                    let (name, kind, index) = (section.name()?, section.byte()?, section.number()? as usize);
                    exports.push((name, kind, index));
                }
            }
            // Les valeurs globales, le départ, les éléments, le code, les données, leur nombre,
            // les exceptions : rien qui sorte de la boîte. Le navigateur vérifie le code lui-même.
            6 | 8..=13 => {}
            _ => return Err(format!("le module a une section inconnue ({id}) : la boîte ne lance que ce qu'elle sait lire")),
        }
        reader.at = end;
    }
    // Les fonctions exportées : aucune n'est importée, leur numéro est donc leur place.
    let parameters = |wanted: &str| {
        exports.iter().find(|(name, kind, _)| name == wanted && *kind == 0x00).and_then(|(_, _, index)| functions.get(*index)).and_then(|type_index| types.get(*type_index)).copied()
    };
    match (parameters("run"), parameters("alloc")) {
        (None, _) => Err("le module n'offre pas de fonction « run » : la boîte ne sait pas le lancer".into()),
        (Some(2), Some(1)) => Ok(Offer::Second),
        (Some(0 | 1), _) => Ok(Offer::First),
        _ => Err("la fonction « run » du module n'est d'aucun des deux contrats : run(nombre), ou alloc(taille) et run(adresse, taille)".into()),
    }
}

/// Ce qui est dit aux visiteurs, en bas de la page (ADR-118) : chaque module qui dit sa licence,
/// avec son fichier et le site d'où il vient. Lu par tous, lecteur d'écran compris : un repère
/// nommé (`aside`), du texte, sans JavaScript aussi. Si le moteur refuse de le lancer, il le dit à
/// la même place (`role="status"`), dans la langue de la page : la page montre le texte caché qui
/// convient (`data-why`).
pub fn notice(program: &Program, french: bool) -> String {
    use crate::flat::escape;
    let Ok(modules) = modules(program) else { return String::new() };
    let mut said = String::new();
    for module in &modules {
        let Some(license) = module.license else { continue };
        let host = module.from.and_then(|from| crate::state::remote_address(from).ok()).map(|address| address.host);
        let (file, license) = (escape(module.source), escape(license));
        let line = match (french, host) {
            (true, Some(host)) => format!("Module «\u{202F}{file}\u{202F}», venu de {host} — licence\u{a0}: {license}"),
            (true, None) => format!("Module «\u{202F}{file}\u{202F}» — licence\u{a0}: {license}"),
            (false, Some(host)) => format!("Module “{file}”, from {host} — license: {license}"),
            (false, None) => format!("Module “{file}” — license: {license}"),
        };
        let (changed, boxed) = if french {
            (" — refusé\u{a0}: ce fichier n'est pas celui que l'auteur a vérifié (son empreinte diffère)", " — refusé\u{a0}: il demande plus que ce que la boîte lui donne")
        } else {
            (" — refused: this file is not the one the author checked (its fingerprint differs)", " — refused: it asks for more than the box gives it")
        };
        said.push_str(&format!(
            "<p data-module=\"{}\" role=\"status\">{line}<span class=\"holo-refused\" data-why=\"changed\" hidden>{changed}</span><span class=\"holo-refused\" data-why=\"box\" hidden>{boxed}</span></p>",
            escape(module.name)
        ));
    }
    if said.is_empty() {
        return String::new();
    }
    format!("<aside class=\"holo-modules\" aria-label=\"{}\">{said}</aside>", if french { "Modules de cette page" } else { "Modules on this page" })
}

fn a_module<'a>(program: &Program, block: &'a Block) -> Result<Module<'a>, Error> {
    let error = |message: String, pos| Error { message, pos };
    let (mut name, mut source, mut inputs, mut outputs, mut time, mut pages) = (None, None, Vec::new(), None, CURRENT_TIME, CURRENT_PAGES);
    // Un module venu d'ailleurs (ADR-118) : son empreinte, son adresse, sa licence.
    let (mut sha256, mut from, mut license) = (None, None, None);
    for a in &block.arguments {
        match (a.name.as_deref(), &a.value) {
            (Some("name"), Value::Name(n)) => name = Some(n.as_str()),
            (Some("source"), Value::Text(s)) if s.ends_with(".wasm") && crate::flat::path_on(s) => source = Some(s.as_str()),
            (Some("source"), _) => return Err(error("« Module(source: …) » attend un fichier .wasm rangé à côté de la page, comme \"somme.wasm\"".into(), a.pos)),
            (Some(p @ ("input" | "output")), value) => {
                let names = value_names(program, p, value).map_err(|message| error(message, a.pos))?;
                if p == "input" {
                    inputs = names;
                } else {
                    outputs = Some(names);
                }
            }
            (Some("time"), Value::Number { value, unit: Some(u), .. }) if u == "ms" || u == "s" => {
                let ms = if u == "s" { value * 1000.0 } else { *value };
                if !(TIME_MIN as f64..=TIME_MAX as f64).contains(&ms) {
                    return Err(error(format!("« Module(time: …) » va de {TIME_MIN}ms à 5s : au-delà, le module est arrêté"), a.pos));
                }
                time = ms.round() as u64;
            }
            (Some("time"), _) => return Err(error("« Module(time: …) » attend une durée, comme 100ms".into(), a.pos)),
            (Some("memory"), Value::Number { value, unit: Some(u), .. }) if u == "KB" || u == "MB" => {
                let bytes = if u == "MB" { value * 1e6 } else { value * 1e3 };
                let wanted_ones = (bytes / PAGE as f64).ceil() as u64;
                if !(1..=PAGES_MAX).contains(&wanted_ones) {
                    return Err(error("« Module(memory: …) » va de 64KB à 16MB".into(), a.pos));
                }
                pages = wanted_ones;
            }
            (Some("memory"), _) => return Err(error("« Module(memory: …) » attend une taille, comme 1MB".into(), a.pos)),
            // L'empreinte du fichier (ADR-118) : 64 chiffres hexadécimaux, comme la donnent
            // sha256sum, Get-FileHash et holo check. Majuscules ou minuscules : la même.
            (Some("sha256"), Value::Text(s)) if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) => sha256 = Some(s.as_str()),
            (Some("sha256"), Value::Text(s)) if s.starts_with("sha256-") || s.starts_with("sha256:") => {
                return Err(error("« Module(sha256: …) » attend seulement les 64 chiffres hexadécimaux de l'empreinte (0 à 9, a à f), sans « sha256- » ni « sha256: » devant : sha256: \"9f86d081…\"".into(), a.pos))
            }
            (Some("sha256"), _) => return Err(error("« Module(sha256: …) » attend l'empreinte du fichier : 64 chiffres hexadécimaux (0 à 9, a à f) entre guillemets, comme la donnent sha256sum et holo check".into(), a.pos)),
            // L'adresse d'où vient le module (ADR-118), lue strictement, comme celle des données
            // d'un autre site (ADR-116) : HTTPS, un nom de site et jamais une adresse IP, ni port,
            // ni « nom@ », ni « # », ni valeur « {…} ». Elle finit par .wasm.
            (Some("from"), Value::Text(address)) => match crate::state::remote_address(address) {
                Ok(checked) if checked.target.split('?').next().unwrap_or("").ends_with(".wasm") => from = Some(address.as_str()),
                Ok(_) => return Err(error("« Module(from: …) » : l'adresse d'un module finit par .wasm, comme \"https://exemple.org/somme-1.0.wasm\"".into(), a.pos)),
                Err(reason) => return Err(error(format!("« Module(from: …) » : {reason}"), a.pos)),
            },
            (Some("from"), _) => return Err(error("« Module(from: …) » attend l'adresse HTTPS d'où vient le module, entre guillemets : from: \"https://exemple.org/somme-1.0.wasm\"".into(), a.pos)),
            // Sa licence (ADR-118), telle que son auteur la donne : « MIT », « Apache-2.0 »… Un
            // texte d'une ligne, qui ne lit aucune valeur.
            (Some("license"), Value::Text(l)) if !l.trim().is_empty() && l.chars().count() <= LICENSE_MAX && !l.chars().any(|c| c.is_control() || c == '{' || c == '}') => license = Some(l.as_str()),
            (Some("license"), _) => return Err(error(format!("« Module(license: …) » attend la licence du module, comme \"MIT\" ou \"Apache-2.0\" : un texte d'une ligne, {LICENSE_MAX} caractères au plus, sans « {{…}} »"), a.pos)),
            (Some(other), _) => return Err(error(format!("« Module » n'a pas de paramètre « {other} » ; paramètres possibles : name, source, input, output, time, memory, sha256, from, license"), a.pos)),
            (None, _) => return Err(error(format!("chaque paramètre de « Module » est nommé : {}", example()), a.pos)),
        }
    }
    // Les règles d'un module venu d'ailleurs (ADR-118), tenues par le moteur.
    if from.is_some() && sha256.is_none() {
        return Err(error("« Module(from: …) » : une adresse sans empreinte est refusée. Écris aussi sha256: \"…\", l'empreinte que donne l'auteur du module : le fichier est comparé à elle avant chaque lancement, et refusé s'il diffère".into(), block.pos));
    }
    if sha256.is_some() && license.is_none() {
        return Err(error("un module venu d'ailleurs dit sa licence, celle que son auteur lui donne : license: \"MIT\". Elle est dite à tes visiteurs, en bas de la page".into(), block.pos));
    }
    if from.is_some() && source.is_some_and(|s: &str| s.contains('/')) {
        return Err(error("« Module(source: …) » : la copie d'un module venu d'ailleurs se range à côté de la page, sans dossier, comme \"somme.wasm\"".into(), block.pos));
    }
    match (name, source, outputs) {
        (Some(name), Some(source), Some(outputs)) => Ok(Module { name, source, inputs, outputs, time, pages, sha256, from, license }),
        _ => Err(error(format!("« Module » attend name, source et output : {}", example()), block.pos)),
    }
}

/// Les valeurs d'`input` ou d'`output` : un nom, ou une liste de noms (ADR-077). Chacune est une
/// valeur que la page déclare ; un module n'écrit ni l'heure ni la date du visiteur, ni une liste
/// calculée.
fn value_names<'a>(program: &Program, parameter: &str, value: &'a Value) -> Result<Vec<&'a str>, String> {
    let names: Vec<&str> = match value {
        Value::Name(name) => vec![name.as_str()],
        Value::List(elements) if !elements.is_empty() => {
            let mut names = Vec::new();
            for element in elements {
                let Value::Name(name) = element else {
                    return Err(format!("« Module({parameter}: […]) » attend des noms de valeurs de la page, comme [notes, titre]"));
                };
                names.push(name.as_str());
            }
            names
        }
        _ => return Err(format!("« Module({parameter}: …) » attend le nom d'une valeur de la page, ou une liste de noms : {parameter}: total, ou {parameter}: [moyenne, avis]")),
    };
    if names.len() > VALUES_MAX {
        return Err(format!("« Module({parameter}: …) » : {VALUES_MAX} valeurs au plus"));
    }
    for (i, name) in names.iter().enumerate() {
        if names[..i].contains(name) {
            return Err(format!("« Module({parameter}: …) » nomme « {name} » deux fois"));
        }
        if sort_of(program, name).is_none() {
            return Err(format!("« Module({parameter}: …) » : « {name} » n'est pas une valeur de la page ; un module reçoit et rend des nombres, des textes ou des listes déclarés dans State"));
        }
        if parameter == "output" && (crate::state::CLOCK.contains(name) || *name == crate::dates::TODAY) {
            return Err(format!("« {name} » est l'heure ou la date du visiteur, donnée par le moteur : un module ne l'écrit pas"));
        }
        if parameter == "output" && crate::computed::is_computed(program, name) {
            return Err(format!("« {name} » est une liste calculée : le moteur la refait, un module ne l'écrit pas"));
        }
    }
    Ok(names)
}

/// Le module du premier contrat a rendu son nombre : il va dans sa valeur de sortie, bornée ;
/// puis `Nom.done`.
pub fn finished(program: &Program, state: &crate::state::State, texts: &crate::state::Texts, name: &str, value: u64) -> crate::state::State {
    let Some(module) = modules(program).ok().and_then(|m| m.into_iter().find(|m| m.name == name)) else { return state.clone() };
    if !module.simple(program) {
        return state.clone();
    }
    let mut state = state.clone();
    if let Some((_, place)) = state.iter_mut().find(|(known, _)| known == module.outputs[0]) {
        *place = value.min(crate::state::VALUE_MAX);
    }
    crate::state::arbitrate(program, &state, texts, &format!("{name}.done"))
}

/// Ce que le module du second contrat reçoit, en un texte JSON (ADR-077) : un nombre à virgule
/// écrit avec ses chiffres (`12.50`), un texte entre guillemets, une liste en tableau ; les champs
/// d'un élément arrivent en texte, comme dans la page.
pub fn input_json(program: &Program, numbers: &crate::state::State, texts: &crate::state::Texts, lists: &crate::lists::Lists, module: &Module) -> String {
    let computed = crate::computed::apply_with_totals(program, numbers, texts, lists).0;
    let mut written = Vec::new();
    for name in &module.inputs {
        let value = if let Some((_, n)) = numbers.iter().find(|(known, _)| known == name) {
            number_json(*n, crate::state::places(program, name))
        } else if let Some((_, t)) = texts.iter().find(|(known, _)| known == name) {
            text_json(t)
        } else if let Some((_, elements)) = lists.iter().chain(computed.iter()).find(|(known, _)| known == name) {
            let elements: Vec<String> = elements
                .iter()
                .map(|element| match crate::lists::fields(element) {
                    fields if element.starts_with(crate::lists::RECORD) => format!("{{{}}}", fields.iter().map(|(k, v)| format!("{}:{}", text_json(k), text_json(v))).collect::<Vec<_>>().join(",")),
                    _ => text_json(element),
                })
                .collect();
            format!("[{}]", elements.join(","))
        } else {
            continue;
        };
        written.push(format!("{}:{value}", text_json(name)));
    }
    format!("{{{}}}", written.join(","))
}

/// Un nombre de la page en JSON : 1250 à deux chiffres après la virgule s'écrit 12.50.
fn number_json(value: u64, places: u32) -> String {
    if places == 0 {
        return value.to_string();
    }
    let scale = crate::state::scale(places);
    format!("{}.{:0width$}", value / scale, value % scale, width = places as usize)
}

/// Un texte en JSON, entre guillemets, ses signes spéciaux échappés.
fn text_json(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// La réponse d'un module du second contrat (ADR-077), relue avec méfiance : un objet JSON de
/// 64 Ko au plus, dont chaque clé est une valeur annoncée dans `output`, de la bonne sorte. Puis
/// elle est rangée comme des données reçues : dans les bornes de chaque valeur.
pub fn received(program: &Program, numbers: &crate::state::State, texts: &crate::state::Texts, lists: &crate::lists::Lists, name: &str, json: &str) -> Result<(crate::state::State, crate::state::Texts, crate::lists::Lists), String> {
    use crate::lists::Json;
    let Some(module) = modules(program).ok().and_then(|m| m.into_iter().find(|m| m.name == name)) else { return Err(format!("aucun module « {name} »")) };
    if json.len() > ANSWER_MAX {
        return Err(format!("une réponse de plus de {} Ko", ANSWER_MAX / 1024));
    }
    let Some(Json::Object(keys)) = Json::read(json) else { return Err("la réponse n'est pas un objet JSON".into()) };
    for (key, value) in &keys {
        if !module.outputs.contains(&key.as_str()) {
            return Err(format!("le module a rendu « {key} », qu'il n'annonce pas dans output"));
        }
        let (fits, expected) = match sort_of(program, key) {
            Some(Sort::Number) => (matches!(value, Json::Number(_)) || (matches!(value, Json::Decimal(_)) && crate::state::places(program, key) > 0), "un nombre positif"),
            Some(Sort::Text) => (matches!(value, Json::Text(_)), "un texte"),
            Some(Sort::List) => (matches!(value, Json::Table(_)), "une liste"),
            None => (false, "une valeur de la page"),
        };
        if !fits {
            return Err(format!("« {key} » attend {expected}"));
        }
    }
    let (numbers, texts) = crate::state::take_values(program, numbers, texts, json);
    let lists = crate::lists::take_lists(program, lists, json);
    Ok((numbers, texts, lists))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_module_is_declared_and_announced() {
        let source = "module \"somme.wasm\"\nPage(state: State(n: 10, total: 0, ok: 0), modules: [ Module(name: Sum, source: \"somme.wasm\", input: n, output: total, time: 50ms, memory: 1MB) ], children: [ Button(name: Go, text: \"g\") ], rules: [ On(Go.tap, effect: Sum.run), On(Sum.done, effect: ok.set(1)), On(Sum.failed, effect: ok.set(2)) ])";
        crate::check_page(source).unwrap();
        assert_eq!(crate::module_info(source, "n=10;total=0;ok=0", "Sum"), "somme.wasm|10|50|16|1");
        assert_eq!(crate::module_finished(source, "n=10;total=0;ok=0", "Sum", 55), "n=10;total=55;ok=1");
        for (source, message) in [
            ("Page(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t) ], children: [])", "doit être annoncé en haut du fichier"),
            ("module \"s.wasm\"\nPage(children: [])", "la page ne le déclare pas"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t, time: 9s) ], children: [])", "de 10ms à 5s"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t, memory: 1GB) ], children: [])", "attend une taille"),
            ("module \"s.wasm\"\nPage(state: State(t: \"\"), modules: [ Module(name: S, source: \"s.wasm\", output: rien) ], children: [])", "« rien » n'est pas une valeur de la page"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: [t, t]) ], children: [])", "nomme « t » deux fois"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: [hour]) ], children: [ P(\"{hour}\") ])", "donnée par le moteur"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: [\"t\"]) ], children: [])", "attend des noms de valeurs"),
            ("module \"../s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"../s.wasm\", output: t) ], children: [])", "rangé à côté de la page"),
            ("module \"s.wasm\"\nPage(state: State(t: 0), modules: [ Module(name: S, source: \"s.wasm\", output: t) ], children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: S.fly) ])", "un « Module » offre run"),
        ] {
            let error = crate::check_page(source).unwrap_err();
            assert!(error.message.contains(message), "{source}\n→ {error}");
        }
    }

    #[test]
    fn a_module_receives_and_returns_several_values() {
        let source = "module \"notes.wasm\"\nPage(state: State(notes: [ Item(subject: \"Maths\", mark: \"15.5\") ], title: \"\", average: 0.0, best: \"\", count: 0, ok: 0), modules: [ Module(name: Marks, source: \"notes.wasm\", input: [notes, title, average], output: [average, best, count]) ], children: [ Input(value: title, label: \"Titre\"), Button(name: Go, text: \"g\"), P(\"{average} {best} {count}\") ], rules: [ On(Go.tap, effect: Marks.run), On(Marks.done, effect: ok.set(1)) ])";
        let program = crate::check_page(source).unwrap();
        let start = crate::input(source, &crate::initial_state(source), "title", r#"Trimestre "1" \ fin"#);
        // Le second contrat : pas le premier.
        assert!(crate::module_info(source, &start, "Marks").ends_with("|0"), "{}", crate::module_info(source, &start, "Marks"));
        assert!(!super::modules(&program).unwrap()[0].simple(&program));
        // Ce que le module reçoit : la liste à champs, le texte échappé, le nombre à virgule.
        assert_eq!(crate::module_input(source, &start, "Marks"), r#"{"notes":[{"subject":"Maths","mark":"15.5"}],"title":"Trimestre \"1\" \\ fin","average":0.0}"#);
        // Une bonne réponse : rangée, puis Marks.done.
        let after = crate::module_received(source, &start, "Marks", r#"{"average": 13.25, "best": "Maths", "count": 3}"#).unwrap();
        assert!(after.contains("average=133;") && after.contains("count=3;") && after.contains("ok=1") && after.contains("best='Maths"), "{after}");
        // Ce qui est refusé : une clé non annoncée, une mauvaise sorte, un nombre négatif, pas un objet, trop long.
        for (answer, message) in [
            (r#"{"admin": 1}"#.to_string(), "« admin », qu'il n'annonce pas"),
            (r#"{"count": "trois"}"#.to_string(), "« count » attend un nombre positif"),
            (r#"{"count": -3}"#.to_string(), "« count » attend un nombre positif"),
            (r#"{"best": ["Maths"]}"#.to_string(), "« best » attend un texte"),
            (r#"[1, 2]"#.to_string(), "pas un objet JSON"),
            (format!(r#"{{"best": "{}"}}"#, "x".repeat(super::ANSWER_MAX)), "plus de 64 Ko"),
        ] {
            let error = crate::module_received(source, &start, "Marks", &answer).unwrap_err();
            assert!(error.contains(message), "{answer:.80} → {error}");
        }
        // Une liste rendue : bornée comme des données reçues.
        let source = "module \"tri.wasm\"\nPage(state: State(words: [\"b\", \"a\"]), modules: [ Module(name: Sort, source: \"tri.wasm\", input: words, output: words) ], children: [ Button(name: Go, text: \"g\") ], rules: [ On(Go.tap, effect: Sort.run) ])";
        let after = crate::module_received(source, &crate::initial_state(source), "Sort", r#"{"words": ["a", "b", "c"]}"#).unwrap();
        assert!(after.starts_with("words=[a,b,c]"), "{after}");
    }
}

/// Les modules venus d'ailleurs (ADR-118) : leur empreinte, leur adresse, leur licence, et la
/// boîte qui ne leur donne rien de plus.
#[cfg(test)]
pub(crate) mod foreign_tests {
    use super::{check_wasm, integrity, Offer};

    const SUM: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    /// Une page avec un module, ses paramètres en plus de name, source, input et output.
    fn page(source: &str, extra: &str) -> String {
        format!("module \"{source}\"\nPage(state: State(n: 10, total: 0), modules: [ Module(name: Sum, source: \"{source}\", input: n, output: total{extra}) ], children: [ Button(name: Go, text: \"g\"), P(\"{{total}}\") ], rules: [ On(Go.tap, effect: Sum.run) ])")
    }

    // De quoi fabriquer un petit module WebAssembly, octet par octet, pour éprouver la boîte.
    fn number(out: &mut Vec<u8>, mut n: u32) {
        loop {
            let byte = (n & 0x7f) as u8;
            n >>= 7;
            if n == 0 {
                out.push(byte);
                return;
            }
            out.push(byte | 0x80);
        }
    }
    fn name(text: &str) -> Vec<u8> {
        let mut out = Vec::new();
        number(&mut out, text.len() as u32);
        out.extend(text.as_bytes());
        out
    }
    fn items(entries: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::new();
        number(&mut out, entries.len() as u32);
        entries.iter().for_each(|entry| out.extend(entry));
        out
    }
    pub(crate) fn wasm(sections: &[(u8, Vec<u8>)]) -> Vec<u8> {
        let mut out = b"\0asm\x01\0\0\0".to_vec();
        for (id, content) in sections {
            out.push(*id);
            number(&mut out, content.len() as u32);
            out.extend(content);
        }
        out
    }
    fn import(from: &str, field: &str, rest: &[u8]) -> Vec<u8> {
        [name(from), name(field), rest.to_vec()].concat()
    }
    fn export(field: &str, index: u8) -> Vec<u8> {
        [name(field), vec![0x00, index]].concat()
    }
    /// Le plus petit module du premier contrat : il reçoit sa mémoire, offre run(nombre) et rend
    /// son nombre. `imports` et `more` le changent pour chaque essai.
    pub(crate) fn module(types: Vec<Vec<u8>>, imports: Vec<Vec<u8>>, more: Vec<(u8, Vec<u8>)>, exports: Vec<Vec<u8>>) -> Vec<u8> {
        let mut sections = vec![(1, items(&types)), (2, items(&imports)), (3, items(&[vec![0x00]]))];
        sections.extend(more);
        sections.push((7, items(&exports)));
        sections.push((10, items(&[vec![0x04, 0x00, 0x20, 0x00, 0x0b]])));
        sections.sort_by_key(|(id, _)| if *id == 10 { 11 } else { *id });
        wasm(&sections)
    }
    pub(crate) fn one_number() -> Vec<u8> {
        vec![0x60, 1, 0x7f, 1, 0x7f]
    }
    pub(crate) fn memory(least: u8) -> Vec<u8> {
        import("env", "memory", &[0x02, 0x00, least])
    }
    /// Un module qui demande aussi une fonction à la page, `env.fetch` : le réseau, s'il l'avait.
    pub(crate) fn asking_for_the_network() -> Vec<u8> {
        module(vec![one_number()], vec![memory(1), import("env", "fetch", &[0x00, 0x00])], vec![], vec![export("run", 1)])
    }

    #[test]
    fn a_module_from_elsewhere_says_its_fingerprint_its_address_and_its_license() {
        let from = ", from: \"https://modules.exemple.org/somme/1.0/somme.wasm\"";
        let good = page("somme.wasm", &format!("{from}, sha256: \"{SUM}\", license: \"MIT\""));
        crate::check_page(&good).unwrap();
        // L'empreinte va au navigateur, écrite pour l'intégrité de fetch.
        assert!(crate::module_info(&good, "n=10;total=0", "Sum").ends_with("|sha256-n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg="), "{}", crate::module_info(&good, "n=10;total=0", "Sum"));
        // La copie seule, avec son empreinte et sa licence ; l'empreinte en majuscules aussi.
        crate::check_page(&page("somme.wasm", &format!(", sha256: \"{}\", license: \"Apache-2.0\"", SUM.to_ascii_uppercase()))).unwrap();
        // Un module à soi, sans rien de plus, comme avant : aucune empreinte, rien au navigateur.
        assert_eq!(crate::module_info(&page("somme.wasm", ""), "n=10;total=0", "Sum"), "somme.wasm|10|100|16|1");
        for (extra, message) in [
            (from.to_string(), "une adresse sans empreinte est refusée"),
            (format!("{from}, license: \"MIT\""), "une adresse sans empreinte est refusée"),
            (format!(", sha256: \"{SUM}\""), "un module venu d'ailleurs dit sa licence"),
            (format!("{from}, sha256: \"{SUM}\""), "un module venu d'ailleurs dit sa licence"),
            (", sha256: \"9f86d081\", license: \"MIT\"".to_string(), "64 chiffres hexadécimaux"),
            (format!(", sha256: \"{}zz\", license: \"MIT\"", &SUM[..62]), "64 chiffres hexadécimaux"),
            (", sha256: \"sha256-n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg=\", license: \"MIT\"".to_string(), "sans « sha256- » ni « sha256: » devant"),
            (format!(", sha256: \"sha256:{SUM}\", license: \"MIT\""), "sans « sha256- » ni « sha256: » devant"),
            (", sha256: 42, license: \"MIT\"".to_string(), "64 chiffres hexadécimaux"),
            (format!(", from: \"http://modules.exemple.org/somme.wasm\", sha256: \"{SUM}\", license: \"MIT\""), "HTTPS seulement"),
            (format!(", from: \"https://93.184.216.34/somme.wasm\", sha256: \"{SUM}\", license: \"MIT\""), "adresse IP"),
            (format!(", from: \"https://localhost/somme.wasm\", sha256: \"{SUM}\", license: \"MIT\""), "localhost"),
            (format!(", from: \"https://modules.exemple.org:8443/somme.wasm\", sha256: \"{SUM}\", license: \"MIT\""), "sans port"),
            (format!(", from: \"https://moi@modules.exemple.org/somme.wasm\", sha256: \"{SUM}\", license: \"MIT\""), "ni nom ni mot de passe"),
            (format!(", from: \"https://modules.exemple.org/{{n}}.wasm\", sha256: \"{SUM}\", license: \"MIT\""), "ne lit aucune valeur"),
            (format!(", from: \"https://modules.exemple.org/somme.js\", sha256: \"{SUM}\", license: \"MIT\""), "finit par .wasm"),
            (format!(", from: \"https://modules.exemple.org/somme.wasm.html\", sha256: \"{SUM}\", license: \"MIT\""), "finit par .wasm"),
            (format!(", from: modules, sha256: \"{SUM}\", license: \"MIT\""), "attend l'adresse HTTPS"),
            (format!(", sha256: \"{SUM}\", license: \"\""), "attend la licence du module"),
            (format!(", sha256: \"{SUM}\", license: \"   \""), "attend la licence du module"),
            (format!(", sha256: \"{SUM}\", license: \"{}\"", "M".repeat(65)), "64 caractères au plus"),
            (format!(", sha256: \"{SUM}\", license: \"MIT {{n}}\""), "sans « {…} »"),
            (format!(", sha256: \"{SUM}\", license: MIT"), "attend la licence du module"),
            (", version: \"^1.2\"".to_string(), "paramètres possibles : name, source, input, output, time, memory, sha256, from, license"),
        ] {
            let error = crate::check_page(&page("somme.wasm", &extra)).unwrap_err();
            assert!(error.message.contains(message), "{extra}\n→ {error}");
        }
        // La copie d'un module venu d'une adresse se range à côté de la page.
        let error = crate::check_page(&page("modules/somme.wasm", &format!("{from}, sha256: \"{SUM}\", license: \"MIT\""))).unwrap_err();
        assert!(error.message.contains("à côté de la page, sans dossier"), "{error}");
        // Un même fichier n'a qu'une empreinte.
        let twice = format!("module \"s.wasm\"\nPage(state: State(n: 1, a: 0, b: 0), modules: [ Module(name: A, source: \"s.wasm\", input: n, output: a, sha256: \"{SUM}\", license: \"MIT\"), Module(name: B, source: \"s.wasm\", input: n, output: b, sha256: \"{}\", license: \"MIT\") ], children: [ P(\"{{a}} {{b}}\") ])", "0".repeat(64));
        assert!(crate::check_page(&twice).unwrap_err().message.contains("un fichier n'a qu'une empreinte"));
    }

    #[test]
    fn the_fingerprint_is_written_for_the_integrity_of_fetch() {
        // Les empreintes de "" et de "abc" (FIPS 180-2), et ce que le navigateur attend pour elles.
        assert_eq!(integrity("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"), "sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=");
        assert_eq!(integrity("BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"), "sha256-ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0=");
        // Une empreinte mal écrite ne donne rien : jamais une intégrité que le navigateur ignorerait.
        assert_eq!(integrity("e3b0"), "");
        assert_eq!(integrity(&"g".repeat(64)), "");
    }

    #[test]
    fn the_box_refuses_a_file_that_asks_for_more_than_its_memory() {
        // Les modules des leçons : chacun offre l'un des deux contrats, et rien de plus.
        for (bytes, offer) in [
            (include_bytes!("../../exemples/lecons/69-compter.wasm").as_slice(), Offer::First),
            (include_bytes!("../../exemples/lecons/69-boucle.wasm").as_slice(), Offer::First),
            (include_bytes!("../../exemples/lecons/69-memoire.wasm").as_slice(), Offer::First),
            (include_bytes!("../../exemples/lecons/97-bulletin.wasm").as_slice(), Offer::Second),
            (include_bytes!("../../exemples/lecons/97-menteur.wasm").as_slice(), Offer::Second),
            (include_bytes!("../../exemples/lecons/110-fleur.wasm").as_slice(), Offer::Second),
            (include_bytes!("../../exemples/lecons/141-premiers.wasm").as_slice(), Offer::First),
        ] {
            assert_eq!(check_wasm(bytes, 16), Ok(offer));
        }
        // Le plus petit module, et le second contrat : alloc(taille), run(adresse, taille).
        assert_eq!(check_wasm(&module(vec![one_number()], vec![memory(1)], vec![], vec![export("run", 0)]), 16), Ok(Offer::First));
        let second = wasm(&[
            (1, items(&[one_number(), vec![0x60, 2, 0x7f, 0x7f, 1, 0x7e]])),
            (2, items(&[memory(1)])),
            (3, items(&[vec![0x00], vec![0x01]])),
            (7, items(&[export("alloc", 0), export("run", 1)])),
        ]);
        assert_eq!(check_wasm(&second, 16), Ok(Offer::Second));
        // Ce qui est refusé, avec sa raison.
        let table = |limits: Vec<u8>| module(vec![one_number()], vec![memory(1)], vec![(4, items(&[[vec![0x70], limits].concat()]))], vec![export("run", 0)]);
        let mut cut = module(vec![one_number()], vec![memory(1)], vec![], vec![export("run", 0)]);
        cut.pop();
        for (bytes, message) in [
            (asking_for_the_network(), "« env.fetch » (une fonction) : la boîte ne donne rien d'autre que sa mémoire"),
            (module(vec![one_number()], vec![memory(1), import("wasi_snapshot_preview1", "fd_write", &[0x00, 0x00])], vec![], vec![export("run", 1)]), "« wasi_snapshot_preview1.fd_write » (une fonction)"),
            (module(vec![one_number()], vec![memory(1), import("env", "g", &[0x03, 0x7f, 0x00])], vec![], vec![export("run", 0)]), "« env.g » (une valeur globale)"),
            (module(vec![one_number()], vec![memory(1), import("env", "t", &[0x01, 0x70, 0x00, 0x01])], vec![], vec![export("run", 0)]), "« env.t » (une table)"),
            (module(vec![one_number()], vec![memory(1), memory(1)], vec![], vec![export("run", 0)]), "(une seconde mémoire)"),
            (module(vec![one_number()], vec![import("autre", "memory", &[0x02, 0x00, 0x01])], vec![], vec![export("run", 0)]), "« autre.memory »"),
            (module(vec![one_number()], vec![], vec![(5, items(&[vec![0x00, 0x01]]))], vec![export("run", 0)]), "fabrique sa propre mémoire"),
            (module(vec![one_number()], vec![memory(17)], vec![], vec![export("run", 0)]), "au moins 17 pages"),
            (module(vec![one_number()], vec![import("env", "memory", &[0x02, 0x01, 0x01, 0x02])], vec![], vec![export("run", 0)]), "au plus 2 pages"),
            (module(vec![one_number()], vec![import("env", "memory", &[0x02, 0x03, 0x01, 0x10])], vec![], vec![export("run", 0)]), "partagée"),
            (module(vec![one_number(), vec![0x5f, 0x00]], vec![memory(1)], vec![], vec![export("run", 0)]), "WasmGC"),
            (module(vec![vec![0x60, 1, 0x6e, 1, 0x7f]], vec![memory(1)], vec![], vec![export("run", 0)]), "WasmGC"),
            (table(vec![0x00, 0x01]), "une table sans plafond"),
            (table(vec![0x01, 0x01, 0x80, 0x9a, 0x0c]), "dépassent 100000 éléments"),
            (module(vec![one_number()], vec![memory(1)], vec![], vec![export("go", 0)]), "n'offre pas de fonction « run »"),
            (module(vec![vec![0x60, 3, 0x7f, 0x7f, 0x7f, 1, 0x7f]], vec![memory(1)], vec![], vec![export("run", 0)]), "d'aucun des deux contrats"),
            (module(vec![one_number()], vec![memory(1)], vec![(14, vec![0x00])], vec![export("run", 0)]), "une section inconnue (14)"),
            (b"<!doctype html><title>404</title>".to_vec(), "pas un module WebAssembly"),
            (cut, "coupé ou abîmé"),
        ] {
            let refusal = check_wasm(&bytes, 16).unwrap_err();
            assert!(refusal.contains(message), "{message}\n→ {refusal}");
            assert_eq!(crate::module_check(&bytes, 16), refusal);
        }
        assert_eq!(crate::module_check(include_bytes!("../../exemples/lecons/141-premiers.wasm"), 16), "");
    }

    #[test]
    fn the_visitors_read_the_license_and_where_the_module_comes_from() {
        let source = page("somme.wasm", &format!(", from: \"https://modules.exemple.org/somme/1.0/somme.wasm\", sha256: \"{SUM}\", license: \"MIT <b>\""));
        let program = crate::check_page(&source).unwrap();
        let french = super::notice(&program, true);
        assert_eq!(
            french,
            "<aside class=\"holo-modules\" aria-label=\"Modules de cette page\"><p data-module=\"Sum\" role=\"status\">Module «\u{202F}somme.wasm\u{202F}», venu de modules.exemple.org — licence\u{a0}: MIT &lt;b&gt;<span class=\"holo-refused\" data-why=\"changed\" hidden> — refusé\u{a0}: ce fichier n'est pas celui que l'auteur a vérifié (son empreinte diffère)</span><span class=\"holo-refused\" data-why=\"box\" hidden> — refusé\u{a0}: il demande plus que ce que la boîte lui donne</span></p></aside>"
        );
        let english = super::notice(&program, false);
        assert!(english.contains("aria-label=\"Modules on this page\"") && english.contains("Module “somme.wasm”, from modules.exemple.org — license: MIT &lt;b&gt;") && english.contains(" — refused: this file is not the one the author checked"), "{english}");
        // Dans la page, en bas, sans JavaScript aussi ; dans sa langue.
        let html = crate::flat_view(&source, "").unwrap();
        assert!(html.contains("</main>") && html.find("</main>") < html.find("<aside class=\"holo-modules\""), "{html}");
        let html = crate::flat_view(&source.replace("Page(state:", "Page(lang: \"en\", state:"), "").unwrap();
        assert!(html.contains("Modules on this page"), "{html}");
        // Une copie avec sa licence, sans adresse : son fichier et sa licence.
        let copy = crate::check_page(&page("somme.wasm", &format!(", sha256: \"{SUM}\", license: \"MIT\""))).unwrap();
        assert!(super::notice(&copy, true).contains("Module «\u{202F}somme.wasm\u{202F}» — licence\u{a0}: MIT<span"));
        // Un module à soi, sans licence : rien n'est ajouté à la page.
        assert_eq!(super::notice(&crate::check_page(&page("somme.wasm", "")).unwrap(), true), "");
        assert!(!crate::flat_view(&page("somme.wasm", ""), "").unwrap().contains("<aside class=\"holo-modules\""));
    }
}
