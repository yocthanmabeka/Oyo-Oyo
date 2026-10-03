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
    pub arguments: Vec<Argument>,
    pub pos: Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// `import`, `module` ou `bridge js` / `bridge css` (ADR-013, ADR-016).
    pub sorte: String,
    pub cible: String,
    pub pos: Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Programme {
    pub imports: Vec<Import>,
    pub racine: Bloc,
}

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

const UNITES: &[&str] = &["mm", "cm", "m", "km", "ms", "s", "min", "h", "B", "KB", "MB", "GB"];

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

    fn jetons(mut self) -> Result<Vec<Jeton>, Erreur> {
        let mut jetons = Vec::new();
        loop {
            // blancs et commentaires
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
}

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
            let mut sorte = n.clone();
            self.avancer();
            if sorte == "bridge" {
                match self.avancer().mot {
                    Mot::Nom(l) if l == "js" || l == "css" => sorte = format!("bridge {l}"),
                    _ => return Err(Erreur { message: "« bridge » doit être suivi de « js » ou « css » (ADR-012)".into(), pos }),
                }
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
        Ok(Programme { imports, racine })
    }

    fn bloc(&mut self) -> Result<Bloc, Erreur> {
        let jeton = self.avancer();
        let nom = match jeton.mot {
            Mot::Nom(n) => n,
            autre => return Err(Erreur { message: format!("nom de bloc attendu, {} trouvé", decrire(&autre)), pos: jeton.pos }),
        };
        if !nom.chars().next().is_some_and(|c| c.is_ascii_uppercase()) || nom.contains('.') {
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
        Ok(Bloc { nom, arguments, pos: jeton.pos })
    }

    fn valeur(&mut self) -> Result<Valeur, Erreur> {
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
pub fn lire(source: &str) -> Result<Programme, Erreur> {
    let lecteur = Lecteur { src: source.as_bytes(), texte: source, i: 0, ligne: 1, debut_ligne: 0 };
    Analyseur { jetons: lecteur.jetons()?, i: 0 }.programme()
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
        let p = lire(r#"
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
    fn refuse_deux_blocs_racines() {
        assert!(lire("Point(seed: 1) Point(seed: 2)").unwrap_err().message.contains("un seul bloc racine"));
    }
}
