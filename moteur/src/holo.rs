//! Lecture d'un fichier `.holo` : des blocs nommés par leur sens, imbriqués à la manière
//! de Flutter (ADR-009). Ce lecteur suit la grammaire brouillon de
//! `experiments/conformite-v0.1/README.md`. Il ne fait rien d'autre que lire : donner un
//! sens aux blocs est le travail de `univers.rs`.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub ligne: u32,
    pub colonne: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Valeur {
    Bloc(Bloc),
    Liste(Vec<Valeur>),
    Texte(String),
    /// Un entier écrit sans point ni unité, gardé exact : une graine ne passe jamais par
    /// un nombre flottant (revue Codex : 2^53 + 1 devenait 2^53).
    Entier(u64),
    Nombre { valeur: f64, unite: Option<String> },
    Bool(bool),
    /// Un nom, éventuellement à points : `Atelier`, `Ouvrir.touche`, `auto`.
    Nom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub nom: Option<String>,
    pub valeur: Valeur,
    pub pos: Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bloc {
    pub nom: String,
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
    pub sorte: String,
    pub cible: String,
    pub pos: Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Programme {
    pub imports: Vec<Import>,
    pub racine: Bloc,
    /// Les styles, écrits comme en CSS après le bloc racine (ADR-017).
    pub styles: Vec<RegleStyle>,
    /// Les noms des composants du fichier (ADR-050) : un style peut les viser, `ArticleCard { … }`.
    pub composants: Vec<String>,
}

/// Ce qu'un style vise : un type de bloc (`P`) ou un nom à point (`.card`). Rien d'autre.
#[derive(Debug, Clone, PartialEq)]
pub enum Cible {
    Type(String),
    Nom(String),
}

impl fmt::Display for Cible {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cible::Type(nom) => write!(f, "{nom}"),
            Cible::Nom(nom) => write!(f, ".{nom}"),
        }
    }
}

/// `color: gray;` La valeur est gardée telle qu'elle est écrite ; `styles.rs` la vérifie.
#[derive(Debug, Clone, PartialEq)]
pub struct Reglage {
    pub nom: String,
    pub valeur: String,
    pub pos: Pos,
}

/// `.card { color: gray; }`
#[derive(Debug, Clone, PartialEq)]
pub struct RegleStyle {
    pub cible: Cible,
    pub reglages: Vec<Reglage>,
    /// Les états du bloc : `hover: { … }`, `focus: { … }`, `active: { … }` (ADR-036).
    pub etats: Vec<(String, Vec<Reglage>, Pos)>,
    pub pos: Pos,
}

/// Les états qu'un style peut décrire : au survol, au focus du clavier, pendant l'appui ; puis
/// quand le visiteur a choisi le thème sombre, et sur un écran de téléphone (ADR-041).
pub const ETATS: &[&str] = &["hover", "focus", "active", "dark", "phone"];

#[derive(Debug, Clone, PartialEq)]
pub struct Erreur {
    pub message: String,
    pub pos: Pos,
}

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ligne {}, colonne {} : {}", self.pos.ligne, self.pos.colonne, self.message)
    }
}

impl Bloc {
    pub fn argument(&self, nom: &str) -> Option<&Argument> {
        self.arguments.iter().find(|a| a.nom.as_deref() == Some(nom))
    }
}

/// Le nombre de noms de style qu'un bloc peut porter, au plus (ADR-051).
pub const STYLES_PAR_BLOC: usize = 4;

const UNITES: &[&str] = &["mm", "cm", "m", "km", "ms", "s", "min", "h", "B", "KB", "MB", "GB", "px", "deg"];

// ---------------------------------------------------------------- découpage en mots

#[derive(Debug, Clone, PartialEq)]
enum Mot {
    Nom(String),
    Entier(u64),
    Nombre(f64, Option<String>),
    Texte(String),
    Signe(char),
    Fin,
}

#[derive(Debug, Clone)]
struct Jeton {
    mot: Mot,
    pos: Pos,
}

struct Lecteur<'a> {
    src: &'a [u8],
    texte: &'a str,
    i: usize,
    ligne: u32,
    debut_ligne: usize,
}

impl<'a> Lecteur<'a> {
    fn pos(&self) -> Pos {
        Pos { ligne: self.ligne, colonne: (self.i - self.debut_ligne) as u32 + 1 }
    }

    fn erreur(&self, message: impl Into<String>) -> Erreur {
        Erreur { message: message.into(), pos: self.pos() }
    }

    /// Avance d'un caractère entier (les lettres accentuées font plusieurs octets).
    fn avancer(&mut self) {
        let c = self.src[self.i];
        if c == b'\n' {
            self.ligne += 1;
            self.debut_ligne = self.i + 1;
        }
        self.i += match c {
            c if c < 0x80 => 1,
            c if c >= 0xF0 => 4,
            c if c >= 0xE0 => 3,
            _ => 2,
        };
    }

    /// Passe les blancs et les commentaires.
    fn sauter_blancs(&mut self) {
        while self.i < self.src.len() {
            let c = self.src[self.i];
            if c == b'/' && self.src.get(self.i + 1) == Some(&b'/') {
                while self.i < self.src.len() && self.src[self.i] != b'\n' {
                    self.i += 1;
                }
            } else if c.is_ascii_whitespace() {
                self.avancer();
            } else {
                break;
            }
        }
    }

    /// Découpe les imports et le bloc racine. S'arrête à la parenthèse qui referme le bloc
    /// racine : ce qui suit est fait de styles, lus par `styles`.
    fn jetons(&mut self) -> Result<Vec<Jeton>, Erreur> {
        let mut jetons = Vec::new();
        let mut profondeur = 0i32;
        loop {
            self.sauter_blancs();
            if self.i >= self.src.len() {
                jetons.push(Jeton { mot: Mot::Fin, pos: self.pos() });
                return Ok(jetons);
            }
            let pos = self.pos();
            let c = self.src[self.i];
            let mot = if c.is_ascii_digit() || (c == b'-' && self.src.get(self.i + 1).is_some_and(|d| d.is_ascii_digit())) {
                self.nombre()?
            } else if c.is_ascii_alphabetic() || c == b'_' {
                let debut = self.i;
                while self.i < self.src.len() && (self.src[self.i].is_ascii_alphanumeric() || self.src[self.i] == b'_' || self.src[self.i] == b'.') {
                    self.i += 1;
                }
                Mot::Nom(self.texte[debut..self.i].to_string())
            } else if c == b'"' {
                self.texte()?
            } else if b"(),:[]".contains(&c) {
                self.i += 1;
                match c {
                    b'(' | b'[' => profondeur += 1,
                    b')' | b']' => profondeur -= 1,
                    _ => {}
                }
                if c == b')' && profondeur == 0 {
                    jetons.push(Jeton { mot: Mot::Signe(')'), pos });
                    jetons.push(Jeton { mot: Mot::Fin, pos: self.pos() });
                    return Ok(jetons);
                }
                Mot::Signe(c as char)
            } else if c == b'{' || c == b'}' || c == b'=' || c == b'>' || c == b';' {
                return Err(self.erreur(format!(
                    "caractère « {} » : du code libre dans un bloc est interdit (ADR-015) ; un bloc ne contient que des valeurs",
                    c as char
                )));
            } else {
                let ch = self.texte[self.i..].chars().next().unwrap_or('?');
                return Err(self.erreur(format!("caractère inattendu « {ch} »")));
            };
            jetons.push(Jeton { mot, pos });
            if jetons.len() > JETONS_MAX {
                return Err(Erreur { message: format!("fichier trop long : plus de {JETONS_MAX} mots"), pos });
            }
        }
    }

    fn nombre(&mut self) -> Result<Mot, Erreur> {
        let debut = self.i;
        if self.src[self.i] == b'-' {
            self.i += 1;
        }
        while self.i < self.src.len() && (self.src[self.i].is_ascii_digit() || self.src[self.i] == b'.') {
            self.i += 1;
        }
        let texte_nombre = &self.texte[debut..self.i];
        let debut_unite = self.i;
        while self.i < self.src.len() && self.src[self.i].is_ascii_alphabetic() {
            self.i += 1;
        }
        let unite = &self.texte[debut_unite..self.i];
        if unite.is_empty() {
            if let Ok(entier) = texte_nombre.parse::<u64>() {
                return Ok(Mot::Entier(entier));
            }
            let valeur: f64 = texte_nombre.parse().map_err(|_| self.erreur("nombre mal formé"))?;
            return Ok(Mot::Nombre(valeur, None));
        }
        let valeur: f64 = texte_nombre.parse().map_err(|_| self.erreur("nombre mal formé"))?;
        if !UNITES.contains(&unite) {
            return Err(Erreur {
                message: format!("unité inconnue « {unite} » ; unités possibles : {}", UNITES.join(", ")),
                pos: Pos { ligne: self.ligne, colonne: (debut_unite - self.debut_ligne) as u32 + 1 },
            });
        }
        Ok(Mot::Nombre(valeur, Some(unite.to_string())))
    }

    /// Lit les styles qui suivent le bloc racine. L'écriture est celle du CSS de base :
    /// `P { color: gray; }` ou `.card { border-radius: 8px; }` (ADR-017).
    fn styles(&mut self) -> Result<Vec<RegleStyle>, Erreur> {
        let mut regles = Vec::new();
        loop {
            self.sauter_blancs();
            if self.i >= self.src.len() {
                return Ok(regles);
            }
            let pos = self.pos();
            let nomme = self.src[self.i] == b'.';
            if nomme {
                self.i += 1;
            }
            let nom = self.mot_de_style(false);
            if nom.is_empty() {
                return Err(Erreur {
                    message: "après le bloc racine viennent seulement des styles : un type de bloc (« P { … } ») ou un nom à point (« .card { … } »)".into(),
                    pos,
                });
            }
            let cible = if nomme { Cible::Nom(nom) } else { Cible::Type(nom) };
            self.sauter_blancs();
            match self.src.get(self.i) {
                Some(b'{') => self.i += 1,
                Some(b'(') => {
                    return Err(Erreur {
                        message: "un seul bloc racine par fichier ; après lui viennent seulement des styles, écrits comme en CSS : « P { color: gray; } »".into(),
                        pos,
                    })
                }
                _ => {
                    return Err(self.erreur(format!(
                        "après « {cible} », une accolade « {{ » est attendue : un style vise un type de bloc ou un nom à point, rien d'autre (ADR-017)"
                    )))
                }
            }
            let mut reglages = Vec::new();
            let mut etats = Vec::new();
            loop {
                self.sauter_blancs();
                match self.src.get(self.i) {
                    None => return Err(Erreur { message: format!("le style « {cible} » n'est jamais refermé : « }} » manquant"), pos }),
                    Some(b'}') => {
                        self.i += 1;
                        break;
                    }
                    _ => {}
                }
                let pos_reglage = self.pos();
                let nom = self.mot_de_style(true);
                self.sauter_blancs();
                if nom.is_empty() || self.src.get(self.i) != Some(&b':') {
                    return Err(Erreur { message: "un réglage s'écrit « nom: valeur; », comme « color: gray; »".into(), pos: pos_reglage });
                }
                self.i += 1;
                // Un état : `hover: { background: navy; }`. Ses réglages valent pendant cet état.
                while matches!(self.src.get(self.i), Some(b' ' | b'\t')) {
                    self.i += 1;
                }
                if self.src.get(self.i) == Some(&b'{') {
                    if !ETATS.contains(&nom.as_str()) {
                        return Err(Erreur { message: format!("« {nom} » n'est pas un état ; un style décrit ces états : {} (ADR-036)", ETATS.join(", ")), pos: pos_reglage });
                    }
                    self.i += 1;
                    let mut dedans = Vec::new();
                    loop {
                        self.sauter_blancs();
                        match self.src.get(self.i) {
                            None => return Err(Erreur { message: format!("l'état « {nom} » de « {cible} » n'est jamais refermé : « }} » manquant"), pos: pos_reglage }),
                            Some(b'}') => {
                                self.i += 1;
                                break;
                            }
                            _ => {}
                        }
                        let pos_dedans = self.pos();
                        let reglage = self.mot_de_style(true);
                        self.sauter_blancs();
                        if reglage.is_empty() || self.src.get(self.i) != Some(&b':') {
                            return Err(Erreur { message: "dans un état, un réglage s'écrit « nom: valeur; », comme « background: navy; »".into(), pos: pos_dedans });
                        }
                        self.i += 1;
                        let debut = self.i;
                        while self.i < self.src.len() && !b";}\n{".contains(&self.src[self.i]) {
                            self.avancer();
                        }
                        let valeur = self.texte[debut..self.i].trim().to_string();
                        match self.src.get(self.i) {
                            Some(b';') => self.i += 1,
                            Some(b'}') => {}
                            _ => return Err(Erreur { message: format!("« ; » manquant à la fin du réglage « {reglage} »"), pos: pos_dedans }),
                        }
                        if valeur.is_empty() {
                            return Err(Erreur { message: format!("le réglage « {reglage} » n'a pas de valeur"), pos: pos_dedans });
                        }
                        dedans.push(Reglage { nom: reglage, valeur, pos: pos_dedans });
                    }
                    self.sauter_blancs();
                    if self.src.get(self.i) == Some(&b';') {
                        self.i += 1;
                    }
                    etats.push((nom, dedans, pos_reglage));
                    continue;
                }
                let debut = self.i;
                while self.i < self.src.len() && !b";}\n{".contains(&self.src[self.i]) {
                    self.avancer();
                }
                let valeur = self.texte[debut..self.i].trim().to_string();
                match self.src.get(self.i) {
                    Some(b';') => self.i += 1,
                    Some(b'}') => {}
                    // En CSS, un « ; » oublié avale la ligne suivante sans rien dire.
                    _ => return Err(Erreur { message: format!("« ; » manquant à la fin du réglage « {nom} »"), pos: pos_reglage }),
                }
                if valeur.is_empty() {
                    return Err(Erreur { message: format!("le réglage « {nom} » n'a pas de valeur"), pos: pos_reglage });
                }
                reglages.push(Reglage { nom, valeur, pos: pos_reglage });
            }
            regles.push(RegleStyle { cible, reglages, etats, pos });
        }
    }

    /// Un nom dans un style : lettres, chiffres et `_` ; le tiret en plus pour un réglage (`font-size`).
    fn mot_de_style(&mut self, tiret: bool) -> String {
        let debut = self.i;
        while self.i < self.src.len() && (self.src[self.i].is_ascii_alphanumeric() || self.src[self.i] == b'_' || (tiret && self.src[self.i] == b'-')) {
            self.i += 1;
        }
        self.texte[debut..self.i].to_string()
    }

    fn texte(&mut self) -> Result<Mot, Erreur> {
        let pos = self.pos();
        let long = self.texte[self.i..].starts_with("\"\"\"");
        let fin: &str = if long { "\"\"\"" } else { "\"" };
        self.i += fin.len();
        let debut = self.i;
        loop {
            if self.i >= self.src.len() || (!long && self.src[self.i] == b'\n') {
                return Err(Erreur { message: "texte jamais refermé".into(), pos });
            }
            if self.texte[self.i..].starts_with(fin) {
                let contenu = self.texte[debut..self.i].to_string();
                for _ in 0..fin.len() {
                    self.i += 1;
                }
                return Ok(Mot::Texte(if long { detacher(&contenu) } else { contenu }));
            }
            self.avancer();
        }
    }
}

/// Retire l'indentation commune d'un texte long, pour que le Markdown reste propre.
fn detacher(texte: &str) -> String {
    let lignes: Vec<&str> = texte.lines().collect();
    let marge = lignes
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut sortie: Vec<&str> = lignes.iter().map(|l| if l.len() >= marge { &l[marge..] } else { l.trim_start() }).collect();
    while sortie.first().is_some_and(|l| l.trim().is_empty()) {
        sortie.remove(0);
    }
    while sortie.last().is_some_and(|l| l.trim().is_empty()) {
        sortie.pop();
    }
    sortie.join("\n")
}

// ---------------------------------------------------------------- analyse

struct Analyseur {
    jetons: Vec<Jeton>,
    i: usize,
    /// Combien de blocs et de listes sont ouverts les uns dans les autres, en ce moment.
    imbrication: u32,
}

/// Un fichier `.holo` est un texte court. Ces trois limites s'appliquent avant toute analyse,
/// pour qu'un fichier hostile ne puisse ni remplir la mémoire ni faire déborder la pile
/// (revue Codex du 2026-10-03, B-09).
pub const OCTETS_MAX: usize = 262_144;
pub const JETONS_MAX: usize = 100_000;
pub const IMBRICATION_MAX: u32 = 64;

impl Analyseur {
    fn courant(&self) -> &Jeton {
        &self.jetons[self.i]
    }

    fn avancer(&mut self) -> Jeton {
        let j = self.jetons[self.i].clone();
        if self.i + 1 < self.jetons.len() {
            self.i += 1;
        }
        j
    }

    fn est_signe(&self, c: char) -> bool {
        self.courant().mot == Mot::Signe(c)
    }

    fn signe(&mut self, c: char) -> Result<(), Erreur> {
        if self.est_signe(c) {
            self.avancer();
            Ok(())
        } else {
            Err(self.erreur(format!("« {c} » attendu, {} trouvé", decrire(&self.courant().mot))))
        }
    }

    fn erreur(&self, message: String) -> Erreur {
        Erreur { message, pos: self.courant().pos }
    }

    fn programme(mut self) -> Result<Programme, Erreur> {
        let mut imports = Vec::new();
        while let Mot::Nom(n) = &self.courant().mot {
            if n != "import" && n != "module" && n != "bridge" {
                break;
            }
            let pos = self.courant().pos;
            let sorte = n.clone();
            self.avancer();
            if sorte == "bridge" {
                return Err(Erreur {
                    message: "« bridge » est refusé : un pont ferait entrer du code sans garantie (ADR-011, partie B) ; pour du code venu d'ailleurs, un module enfermé : module \"calcul.wasm\" (ADR-045)".into(),
                    pos,
                });
            }
            match self.avancer().mot {
                Mot::Texte(cible) => imports.push(Import { sorte, cible, pos }),
                _ => return Err(Erreur { message: format!("« {sorte} » doit être suivi d'un texte entre guillemets"), pos }),
            }
        }
        let racine = match &self.courant().mot {
            Mot::Nom(_) => self.bloc()?,
            Mot::Fin => return Err(self.erreur("fichier vide : un bloc est attendu".into())),
            autre => return Err(self.erreur(format!("un bloc est attendu, {} trouvé", decrire(autre)))),
        };
        if self.courant().mot != Mot::Fin {
            return Err(self.erreur(format!("un seul bloc racine par fichier ; {} trouvé après lui", decrire(&self.courant().mot))));
        }
        Ok(Programme { imports, racine, styles: Vec::new(), composants: Vec::new() })
    }

    fn bloc(&mut self) -> Result<Bloc, Erreur> {
        let jeton = self.avancer();
        let nom = match jeton.mot {
            Mot::Nom(n) => n,
            autre => return Err(Erreur { message: format!("nom de bloc attendu, {} trouvé", decrire(&autre)), pos: jeton.pos }),
        };
        // `cart.add(1)` : une demande faite à l'arbitre (ADR-023). Elle commence par une
        // minuscule et porte un point ; elle garde son nom entier. `blocs.rs` vérifie sa place.
        let est_demande = nom.contains('.') && nom.starts_with(|c: char| c.is_ascii_lowercase());
        // `P.card(...)` : le bloc `P`, avec le style nommé `card` (ADR-017) ; `P.card.big(...)`,
        // avec deux (ADR-051).
        let (nom, styles) = match nom.split_once('.') {
            Some((bloc, styles)) if !est_demande && !bloc.is_empty() && !styles.is_empty() => (bloc.to_string(), styles.split('.').map(str::to_string).collect::<Vec<_>>()),
            _ => (nom, Vec::new()),
        };
        if styles.len() > STYLES_PAR_BLOC {
            return Err(Erreur { message: format!("« {nom} » porte trop de noms de style : {STYLES_PAR_BLOC} au plus"), pos: jeton.pos });
        }
        for style in &styles {
            if style.is_empty() || !style.starts_with(|c: char| c.is_ascii_lowercase()) || !style.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return Err(Erreur { message: format!("« .{style} » : un nom de style s'écrit en minuscules, comme « card » ou « big-card » (ADR-037)"), pos: jeton.pos });
            }
        }
        if !est_demande && (!nom.chars().next().is_some_and(|c| c.is_ascii_uppercase()) || nom.contains('.')) {
            // Une seule écriture par bloc : `h1` n'est pas accepté à côté de `H1` (ADR-020).
            let mut lettres = nom.chars();
            let message = match lettres.next() {
                Some(c) if c.is_ascii_lowercase() && !nom.contains('.') => {
                    format!("« {nom} » : un nom de bloc commence par une majuscule, écris « {}{} »", c.to_ascii_uppercase(), lettres.as_str())
                }
                _ => format!("« {nom} » : un nom de bloc commence par une majuscule"),
            };
            return Err(Erreur { message, pos: jeton.pos });
        }
        self.signe('(')?;
        let mut arguments = Vec::new();
        while !self.est_signe(')') {
            let pos = self.courant().pos;
            let nom_arg = match (&self.courant().mot, self.jetons.get(self.i + 1)) {
                (Mot::Nom(n), Some(Jeton { mot: Mot::Signe(':'), .. })) => Some(n.clone()),
                _ => None,
            };
            if nom_arg.is_some() {
                self.avancer();
                self.avancer();
            }
            let valeur = self.valeur()?;
            arguments.push(Argument { nom: nom_arg, valeur, pos });
            if self.est_signe(',') {
                self.avancer();
            } else if !self.est_signe(')') {
                return Err(self.erreur(format!("« , » ou « ) » attendu, {} trouvé", decrire(&self.courant().mot))));
            }
        }
        self.signe(')')?;
        Ok(Bloc { nom, styles, arguments, pos: jeton.pos })
    }

    fn valeur(&mut self) -> Result<Valeur, Erreur> {
        self.imbrication += 1;
        if self.imbrication > IMBRICATION_MAX {
            return Err(self.erreur(format!("trop de blocs et de listes les uns dans les autres : la limite est de {IMBRICATION_MAX}")));
        }
        let valeur = self.valeur_simple();
        self.imbrication -= 1;
        valeur
    }

    fn valeur_simple(&mut self) -> Result<Valeur, Erreur> {
        let suivant_est_parenthese = self.jetons.get(self.i + 1).is_some_and(|j| j.mot == Mot::Signe('('));
        let jeton = self.courant().clone();
        match jeton.mot {
            Mot::Nom(_) if suivant_est_parenthese => Ok(Valeur::Bloc(self.bloc()?)),
            Mot::Nom(n) => {
                self.avancer();
                Ok(match n.as_str() {
                    "true" => Valeur::Bool(true),
                    "false" => Valeur::Bool(false),
                    _ => Valeur::Nom(n),
                })
            }
            Mot::Entier(entier) => {
                self.avancer();
                Ok(Valeur::Entier(entier))
            }
            Mot::Nombre(valeur, unite) => {
                self.avancer();
                Ok(Valeur::Nombre { valeur, unite })
            }
            Mot::Texte(t) => {
                self.avancer();
                Ok(Valeur::Texte(t))
            }
            Mot::Signe('[') => {
                self.avancer();
                let mut elements = Vec::new();
                while !self.est_signe(']') {
                    elements.push(self.valeur()?);
                    if self.est_signe(',') {
                        self.avancer();
                    } else if !self.est_signe(']') {
                        return Err(self.erreur(format!("« , » ou « ] » attendu, {} trouvé", decrire(&self.courant().mot))));
                    }
                }
                self.signe(']')?;
                Ok(Valeur::Liste(elements))
            }
            autre => Err(Erreur { message: format!("valeur attendue, {} trouvé", decrire(&autre)), pos: jeton.pos }),
        }
    }
}

fn decrire(mot: &Mot) -> String {
    match mot {
        Mot::Nom(n) => format!("« {n} »"),
        Mot::Entier(v) => format!("« {v} »"),
        Mot::Nombre(v, Some(u)) => format!("« {v}{u} »"),
        Mot::Nombre(v, None) => format!("« {v} »"),
        Mot::Texte(_) => "un texte".into(),
        Mot::Signe(c) => format!("« {c} »"),
        Mot::Fin => "la fin du fichier".into(),
    }
}

/// Lit un fichier `.holo` entier.
/// Ce qui sépare, dans le texte donné au moteur, un fichier de ceux qu'il importe : le fichier,
/// puis pour chaque import ce signe, son nom, `SEPARE_LE_NOM`, et son texte. C'est la page
/// d'entrée (ou le moteur en ligne de commande) qui va chercher les fichiers et les joint :
/// le moteur, lui, ne lit jamais rien tout seul.
pub const FICHIER_SUIVANT: char = '\u{1e}';
pub const SEPARE_LE_NOM: char = '\u{1f}';

/// Le nombre de fichiers qu'une page peut importer, au plus.
pub const IMPORTS_MAX: usize = 16;

/// Un nom de fichier importé : rangé à côté, sans adresse complète ni remontée de dossier.
fn import_sur(nom: &str) -> bool {
    nom.ends_with(".holo") && !nom.starts_with('/') && !nom.contains("..") && nom.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
}

/// Les fichiers qu'un fichier importe (`import "commun.holo"`), pour que celui qui appelle le
/// moteur aille les chercher.
pub fn imports_de(source: &str) -> Result<Vec<String>, Erreur> {
    let principal = source.split(FICHIER_SUIVANT).next().unwrap_or("");
    let programme = lire_seul(principal)?;
    let mut noms = Vec::new();
    for import in programme.imports.iter().filter(|i| i.sorte == "import") {
        if !import_sur(&import.cible) {
            return Err(Erreur { message: format!("« import \"{}\" » : on importe un fichier .holo rangé à côté, comme \"commun.holo\"", import.cible), pos: import.pos });
        }
        if !noms.contains(&import.cible) {
            noms.push(import.cible.clone());
        }
    }
    if noms.len() > IMPORTS_MAX {
        return Err(Erreur { message: format!("trop d'imports : une page en fait au plus {IMPORTS_MAX}"), pos: programme.racine.pos });
    }
    Ok(noms)
}

/// Lit un fichier et ceux qu'il importe, joints à sa suite. Un fichier importé est un morceau :
/// `Part(name: Menu, children: [...])`, avec ses styles. Dans la page, `Use(Menu)` pose les
/// blocs du morceau à cet endroit. Les styles du morceau viennent avec lui ; si la page écrit
/// le même style, c'est le sien qui reste.
pub fn lire(source: &str) -> Result<Programme, Erreur> {
    let mut fichiers = source.split(FICHIER_SUIVANT);
    let mut programme = lire_seul(fichiers.next().unwrap_or(""))?;
    let fournis: Vec<(&str, &str)> = fichiers.filter_map(|f| f.split_once(SEPARE_LE_NOM)).collect();
    let mut morceaux: Vec<(String, Vec<Valeur>)> = Vec::new();
    let mut composants: Vec<crate::composants::Composant> = Vec::new();
    let mut styles_importes: Vec<(String, RegleStyle)> = Vec::new();
    for nom in imports_de(source)? {
        let pos = programme.imports.iter().find(|i| i.cible == nom).map_or(programme.racine.pos, |i| i.pos);
        let Some((_, texte)) = fournis.iter().find(|(fourni, _)| *fourni == nom) else {
            return Err(Erreur { message: format!("le fichier importé « {nom} » n'a pas été trouvé à côté de celui-ci"), pos });
        };
        // Un fichier qui ne contient que des styles (ADR-052) : un thème partagé par les pages.
        let morceau = match lire_seul(texte) {
            Ok(morceau) => morceau,
            Err(e) => match lire_seul(&format!("Part(name: HoloStyles, children: []) {texte}")) {
                Ok(styles) if !styles.styles.is_empty() && styles.imports.is_empty() => {
                    styles_importes.extend(styles.styles.into_iter().map(|r| (nom.clone(), r)));
                    continue;
                }
                _ => return Err(Erreur { message: format!("dans « {nom} », ligne {} : {}", e.pos.ligne, e.message), pos }),
            },
        };
        let refus = |message: String| Erreur { message: format!("« {nom} » : {message}"), pos };
        if morceau.racine.nom != "Part" {
            return Err(refus(format!("un fichier importé est un morceau, il commence par « Part(name: Menu, children: [ … ]) » ; celui-ci commence par « {} »", morceau.racine.nom)));
        }
        if !morceau.imports.is_empty() {
            return Err(refus("un morceau n'importe pas lui-même d'autres fichiers".into()));
        }
        let composant = crate::composants::lire_part(&morceau.racine).map_err(|e| refus(e.message))?;
        if morceaux.iter().any(|(connu, _)| *connu == composant.nom) || composants.iter().any(|c: &crate::composants::Composant| c.nom == composant.nom) {
            return Err(refus(format!("deux morceaux importés s'appellent « {} »", composant.nom)));
        }
        // Sans paramètres ni règles, un morceau se pose aussi par `Use(Menu)`, avec tous ses blocs.
        if composant.parametres.is_empty() && composant.regles.is_empty() {
            morceaux.push((composant.nom.clone(), composant.enfants.clone()));
        }
        composants.push(composant);
        styles_importes.extend(morceau.styles.into_iter().map(|r| (nom.clone(), r)));
    }
    poser_les_morceaux(&mut programme.racine, &morceaux)?;
    // Les composants (ADR-050) : ceux de la page, puis ceux des fichiers importés. Ils sont posés
    // avant les répétitions, pour qu'un composant puisse être répété.
    for composant in crate::composants::retirer_les_parts(&mut programme.racine)? {
        if composants.iter().any(|c| c.nom == composant.nom) {
            return Err(Erreur { message: format!("deux composants s'appellent « {} »", composant.nom), pos: composant.pos });
        }
        composants.push(composant);
    }
    crate::composants::poser_site(&mut programme.racine, &composants)?;
    programme.composants = composants.iter().map(|c| c.nom.clone()).collect();
    if programme.racine.nom == "Part" {
        if let Some(Valeur::Nom(nom)) = programme.racine.argument("name").map(|a| &a.valeur) {
            programme.composants.push(nom.clone());
        }
    }
    // Les répétitions sont dépliées à leur tour, comme les morceaux (ADR-040).
    crate::format::regler_langue(match programme.racine.argument("lang").map(|a| &a.valeur) {
        Some(Valeur::Texte(l)) => l,
        _ => "fr",
    });
    crate::repetition::deplier_site(&mut programme.racine, &mut 0)?;
    // Deux fichiers importés qui écrivent le même style se gêneraient : l'un gagnerait en silence.
    // C'est refusé, avec les deux noms (revue de Codex, PR 124). Un composant qui ne veut rien
    // partager se style par son nom, `ArticleCard { … }`, qui ne vise que ses copies.
    for (i, (fichier, regle)) in styles_importes.iter().enumerate() {
        if let Some((autre, _)) = styles_importes[..i].iter().find(|(f, r)| f != fichier && r.cible == regle.cible) {
            let pos = programme.imports.iter().find(|imp| imp.cible == *fichier).map_or(programme.racine.pos, |imp| imp.pos);
            return Err(Erreur { message: format!("« {autre} » et « {fichier} » écrivent tous deux le style « {} » : l'un effacerait l'autre ; renomme-le dans l'un des deux", regle.cible), pos });
        }
    }
    // Les styles des morceaux d'abord, ceux de la page ensuite : à cible égale, la page garde le sien.
    styles_importes.retain(|(_, importe)| !programme.styles.iter().any(|propre| propre.cible == importe.cible));
    let mut styles: Vec<RegleStyle> = styles_importes.into_iter().map(|(_, r)| r).collect();
    styles.append(&mut programme.styles);
    programme.styles = styles;
    programme.imports.retain(|i| i.sorte != "import");
    Ok(programme)
}

/// Remplace chaque `Use(Menu)` par les blocs du morceau importé de ce nom.
fn poser_les_morceaux(bloc: &mut Bloc, morceaux: &[(String, Vec<Valeur>)]) -> Result<(), Erreur> {
    fn dans(valeur: &mut Valeur, morceaux: &[(String, Vec<Valeur>)]) -> Result<(), Erreur> {
        match valeur {
            Valeur::Bloc(bloc) => poser_les_morceaux(bloc, morceaux),
            Valeur::Liste(elements) => {
                let mut poses = Vec::with_capacity(elements.len());
                for mut element in std::mem::take(elements) {
                    match &element {
                        Valeur::Bloc(appel) if appel.nom == "Use" => {
                            let nom = match appel.arguments.as_slice() {
                                [Argument { nom: None, valeur: Valeur::Nom(nom), .. }] => nom,
                                _ => return Err(Erreur { message: "un morceau se pose par son nom : Use(Menu)".into(), pos: appel.pos }),
                            };
                            let Some((_, enfants)) = morceaux.iter().find(|(connu, _)| connu == nom) else {
                                return Err(Erreur { message: format!("« Use({nom}) » : aucun morceau importé ne s'appelle « {nom} » ; importe son fichier en haut de la page, import \"commun.holo\""), pos: appel.pos });
                            };
                            poses.extend(enfants.iter().cloned());
                        }
                        _ => {
                            dans(&mut element, morceaux)?;
                            poses.push(element);
                        }
                    }
                }
                *elements = poses;
                Ok(())
            }
            _ => Ok(()),
        }
    }
    bloc.arguments.iter_mut().try_for_each(|a| dans(&mut a.valeur, morceaux))
}

/// Lit un seul fichier, sans ses imports.
fn lire_seul(source: &str) -> Result<Programme, Erreur> {
    if source.len() > OCTETS_MAX {
        return Err(Erreur {
            message: format!("fichier trop gros : {} octets, la limite est de {OCTETS_MAX}", source.len()),
            pos: Pos { ligne: 1, colonne: 1 },
        });
    }
    let mut lecteur = Lecteur { src: source.as_bytes(), texte: source, i: 0, ligne: 1, debut_ligne: 0 };
    let mut programme = Analyseur { jetons: lecteur.jetons()?, i: 0, imbrication: 0 }.programme()?;
    programme.styles = lecteur.styles()?;
    Ok(programme)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BIG_BANG: &str = include_str!("../mondes/big-bang.holo");

    #[test]
    fn lit_le_big_bang() {
        let p = lire(BIG_BANG).unwrap();
        assert_eq!(p.racine.nom, "Point");
        assert_eq!(p.racine.argument("name").unwrap().valeur, Valeur::Nom("Origin".into()));
        assert_eq!(p.racine.argument("seed").unwrap().valeur, Valeur::Entier(1));
        assert_eq!(p.racine.argument("fragments").unwrap().valeur, Valeur::Entier(12));
        assert_eq!(p.racine.argument("brightness").unwrap().valeur, Valeur::Nombre { valeur: 1.0, unite: None });
    }

    #[test]
    fn lit_des_blocs_imbriques_du_texte_et_des_unites() {
        let p = lire_seul(r#"
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
        assert_eq!(p.imports[0].sorte, "import");
        let contenu = match &p.racine.argument("children").unwrap().valeur {
            Valeur::Liste(l) => l,
            _ => panic!(),
        };
        assert_eq!(contenu[0], Valeur::Texte("Un paragraphe s'écrit tel quel.".into()));
        let contenu = &contenu[1..];
        match &contenu[0] {
            Valeur::Bloc(b) => assert_eq!(b.arguments[0].valeur, Valeur::Texte("# Bienvenue\nVoici **mes créations**.".into())),
            _ => panic!(),
        }
        match &contenu[1] {
            Valeur::Bloc(b) => assert_eq!(b.argument("budget").unwrap().valeur, Valeur::Nombre { valeur: 500.0, unite: Some("KB".into()) }),
            _ => panic!(),
        }
    }

    #[test]
    fn refuse_le_code_libre_avec_la_bonne_ligne() {
        let e = lire("Page(\n  children: [\n    Button(name: Pay, on_tap: () { x = 1 }),\n  ],\n)").unwrap_err();
        assert_eq!(e.pos.ligne, 3);
        assert!(e.message.contains("ADR-015"), "{e}");
    }

    #[test]
    fn refuse_une_unite_inconnue_et_un_texte_ouvert() {
        assert!(lire("Point(seed: 3parsecs)").unwrap_err().message.contains("unité inconnue"));
        assert_eq!(lire("Point(name: \"oups)").unwrap_err().message, "texte jamais refermé");
    }

    #[test]
    fn les_grands_entiers_restent_exacts() {
        // Revue Codex : 9007199254740993 (2^53 + 1) devenait 9007199254740992 en passant par f64.
        let p = lire("Point(seed: 9007199254740993, max: 18446744073709551615)").unwrap();
        assert_eq!(p.racine.argument("seed").unwrap().valeur, Valeur::Entier(9_007_199_254_740_993));
        assert_eq!(p.racine.argument("max").unwrap().valeur, Valeur::Entier(u64::MAX));
        assert!(lire("Point(seed: 18446744073709551616)").is_ok(), "au-delà de u64, c'est un nombre flottant, refusé plus loin comme graine");
        assert!(lire("Point(budget: 500 KB)").is_err(), "l'unité se colle au nombre : « 500 KB » n'est pas accepté");
    }

    #[test]
    fn un_fichier_hostile_est_arrete_avant_l_analyse() {
        // Trop gros.
        let gros = format!("Page(title: \"{}\")", "x".repeat(OCTETS_MAX));
        assert!(lire(&gros).unwrap_err().message.contains("fichier trop gros"));
        // Trop de blocs les uns dans les autres : refusé, sans faire déborder la pile.
        let profond = format!("Page(children: {}{})", "[".repeat(200), "]".repeat(200));
        assert!(lire(&profond).unwrap_err().message.contains("les uns dans les autres"));
        // Un fichier ordinaire, même bien rempli, passe.
        let large = format!("Page(children: [{}])", "P(\"x\"), ".repeat(2000));
        assert!(lire(&large).is_ok());
    }

    #[test]
    fn deux_fichiers_importes_ne_peuvent_pas_ecrire_le_meme_style() {
        let page = "import \"a.holo\"\nimport \"b.holo\"\nPage(children: [ Use(Alpha), Use(Beta) ])";
        let source = format!("{page}{s}a.holo{n}Part(name: Alpha, children: [ P.card(\"a\") ])\n.card {{ color: red; }}{s}b.holo{n}Part(name: Beta, children: [ P.card(\"b\") ])\n.card {{ color: blue; }}", s = FICHIER_SUIVANT, n = SEPARE_LE_NOM);
        let erreur = lire(&source).unwrap_err();
        assert!(erreur.message.contains("« a.holo » et « b.holo » écrivent tous deux le style « .card »"), "{erreur}");
        // La page, elle, peut toujours réécrire un style importé : c'est le sien qui reste.
        let source = format!("import \"a.holo\"\nPage(children: [ Use(Alpha) ])\n.card {{ color: green; }}{s}a.holo{n}Part(name: Alpha, children: [ P.card(\"a\") ])\n.card {{ color: red; }}", s = FICHIER_SUIVANT, n = SEPARE_LE_NOM);
        assert_eq!(lire(&source).unwrap().styles.len(), 1);
    }

    #[test]
    fn un_fichier_importe_est_un_morceau_qu_on_pose() {
        let commun = "Part(name: Menu, children: [ P(\"menu\"), Hr() ])\nP { color: gray; }\nH1 { color: red; }";
        let page = "import \"commun.holo\"\nPage(children: [ Use(Menu), H1(\"a\"), List(children: [ Use(Menu) ]) ])\nH1 { color: blue; }";
        let joint = |page: &str, nom: &str, texte: &str| format!("{page}{FICHIER_SUIVANT}{nom}{SEPARE_LE_NOM}{texte}");
        assert_eq!(imports_de(page).unwrap(), ["commun.holo"]);
        let programme = lire(&joint(page, "commun.holo", commun)).unwrap();
        // Les blocs du morceau sont posés là où il est appelé, partout où il l'est.
        let Some(Valeur::Liste(enfants)) = programme.racine.argument("children").map(|a| &a.valeur) else { panic!() };
        let noms: Vec<&str> = enfants.iter().map(|e| match e { Valeur::Bloc(b) => b.nom.as_str(), _ => "?" }).collect();
        assert_eq!(noms, ["P", "Hr", "H1", "List"]);
        // Les styles du morceau viennent avec lui ; à cible égale, la page garde le sien.
        let styles: Vec<String> = programme.styles.iter().map(|r| format!("{} {}", r.cible, r.reglages[0].valeur)).collect();
        assert_eq!(styles, ["P gray", "H1 blue"]);
        assert!(programme.imports.is_empty());
        for (source, message) in [
            (page.to_string(), "n'a pas été trouvé"),
            (joint(page, "commun.holo", "Page(children: [])"), "un fichier importé est un morceau"),
            (joint(page, "commun.holo", "Part(name: Other, children: [])"), "aucun morceau importé ne s'appelle « Menu »"),
            (joint(page, "commun.holo", "Part(name: Menu)"), "un morceau a un nom et un contenu"),
            (joint(page, "commun.holo", "Part(name: Menu, children: [ H9( ])"), "dans « commun.holo », ligne 1"),
            (joint(page, "commun.holo", "import \"x.holo\"\nPart(name: Menu, children: [])"), "n'importe pas lui-même"),
            ("Page(children: [ Use(Menu) ])".to_string(), "aucun morceau importé ne s'appelle « Menu »"),
            ("import \"../secret.holo\"\nPage(children: [])".to_string(), "rangé à côté"),
            ("import \"https://x.example/a.holo\"\nPage(children: [])".to_string(), "rangé à côté"),
        ] {
            let erreur = lire(&source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn refuse_deux_blocs_racines() {
        assert!(lire("Point(seed: 1) Point(seed: 2)").unwrap_err().message.contains("un seul bloc racine"));
    }
}
