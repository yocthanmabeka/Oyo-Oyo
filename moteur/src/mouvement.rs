//! Le mouvement (ADR-034) : faire entrer un bloc, le faire vivre en boucle, enchaîner des scènes.
//!
//! ```holo
//! Scenes(height: 480px, repeat: forever, children: [
//!   Scene(for: 4s, children: [
//!     H1("Hello", enter: Enter(y: 40px, opacity: 0, letters: 0.04s, ease: spring)),
//!     Shape(form: circle, loop: Loop(scale: 1.2, for: 1s)),
//!   ]),
//! ])
//! ```
//!
//! Rien ne bouge si l'auteur ne l'écrit pas. Tout devient du CSS fabriqué par le moteur : une
//! page qui bouge n'a pas besoin du moteur dans le navigateur pour bouger, et le visiteur qui
//! demande moins de mouvement (prefers-reduced-motion) voit la page arrêtée.

use crate::holo::{Bloc, Erreur, Valeur};
use std::cell::RefCell;
use std::fmt::Write;

/// Les paramètres de mouvement qu'accepte tout bloc qui se voit.
pub const PARAMETRES: &[&str] = &["enter", "loop"];

/// Les courbes, du plus simple au plus vivant.
const COURBES: &[(&str, &str)] = &[
    ("linear", "linear"),
    ("smooth", "cubic-bezier(.65,0,.35,1)"),
    ("out", "cubic-bezier(.16,1,.3,1)"),
    ("in", "cubic-bezier(.7,0,.84,0)"),
    ("back", "cubic-bezier(.34,1.56,.64,1)"),
    ("spring", "linear(0,.009,.035 2.1%,.141,.281 6.7%,.723 12.9%,.938 16.7%,1.017,1.077,1.121,1.149 24.3%,1.159,1.163,1.161,1.154 29.9%,1.129 32.8%,1.051 39.6%,1.017 43.1%,.991,.977 51%,.974 53.8%,.975 57.1%,.997 69.8%,1.003 76.9%,1.004 83.8%,1)"),
    ("bounce", "linear(0,.004,.016,.035,.063,.098,.141 13.6%,.25,.391,.563,.765,1,.891 40.9%,.848,.813,.785,.766,.754,.75,.754,.766,.785,.813,.848,.891 68.2%,1 72.7%,.973,.953,.941,.938,.941,.953,.973,1,.988,.984,.988,1)"),
];

/// Ce qu'on peut faire bouger, avec son unité et ses bornes.
const PROPRIETES: &[(&str, Option<&str>, f64, f64)] = &[
    ("opacity", None, 0.0, 1.0),
    ("x", Some("px"), -4000.0, 4000.0),
    ("y", Some("px"), -4000.0, 4000.0),
    ("scale", None, 0.0, 20.0),
    ("rotate", Some("deg"), -36000.0, 36000.0),
    ("flip", Some("deg"), -36000.0, 36000.0),
    ("tilt", Some("deg"), -36000.0, 36000.0),
    ("blur", Some("px"), 0.0, 200.0),
    ("hue", Some("deg"), -36000.0, 36000.0),
    ("round", None, 0.0, 50.0),
];

/// Une pose : les valeurs de départ d'une entrée, ou celles qu'une boucle va chercher.
#[derive(Debug, Default, Clone, PartialEq)]
struct Pose(Vec<(&'static str, f64)>);

impl Pose {
    fn a(&self, nom: &str) -> Option<f64> {
        self.0.iter().find(|(connu, _)| *connu == nom).map(|(_, v)| *v)
    }

    /// Les déclarations CSS de cette pose ; `naturel` : celles du bloc au repos.
    fn css(&self, naturel: bool) -> String {
        let v = |nom: &str, repos: f64| if naturel { repos } else { self.a(nom).unwrap_or(repos) };
        let a = |nom: &str| self.a(nom).is_some();
        let mut css = String::new();
        if a("opacity") {
            let _ = write!(css, "opacity:{};", v("opacity", 1.0));
        }
        if a("x") || a("y") {
            let _ = write!(css, "translate:{}px {}px;", v("x", 0.0), v("y", 0.0));
        }
        if a("scale") {
            let _ = write!(css, "scale:{};", v("scale", 1.0));
        }
        if a("rotate") {
            let _ = write!(css, "rotate:{}deg;", v("rotate", 0.0));
        }
        if a("flip") || a("tilt") {
            let _ = write!(css, "transform:perspective(800px) rotateX({}deg) rotateY({}deg);", v("tilt", 0.0), v("flip", 0.0));
        }
        if a("blur") || a("hue") {
            let _ = write!(css, "filter:blur({}px) hue-rotate({}deg);", v("blur", 0.0), v("hue", 0.0));
        }
        if a("round") {
            let _ = write!(css, "border-radius:{}%;", v("round", 0.0));
        }
        css
    }
}

/// Un mouvement lu dans le fichier : `Enter(…)` ou `Loop(…)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Mouvement {
    boucle: bool,
    pose: Pose,
    /// Quand il commence, en secondes, depuis le début de la scène (ou de la page).
    a: f64,
    /// Combien de temps il dure, en secondes (pour une boucle : un aller).
    duree: f64,
    courbe: &'static str,
    /// Lettre après lettre : l'écart entre deux lettres.
    lettres: Option<f64>,
    /// Enfant après enfant : l'écart entre deux enfants.
    chacun: Option<f64>,
    /// Une boucle revient-elle à son point de départ (aller-retour) ? Sinon elle recommence.
    retour: bool,
}

fn erreur(message: String, bloc: &Bloc) -> Erreur {
    Erreur { message, pos: bloc.pos }
}

fn nombre(valeur: &Valeur) -> Option<(f64, Option<&str>)> {
    match valeur {
        Valeur::Entier(n) => Some((*n as f64, None)),
        Valeur::Nombre { valeur, unite } => Some((*valeur, unite.as_deref())),
        _ => None,
    }
}

/// Une durée, `0.4s` ou `400ms`, en secondes.
fn duree(valeur: &Valeur) -> Option<f64> {
    match nombre(valeur)? {
        (n, Some("s")) => Some(n),
        (n, Some("ms")) => Some(n / 1000.0),
        _ => None,
    }
}

/// La durée d'une scène, de 200ms à 600s.
pub fn duree_de_scene(valeur: &Valeur) -> Option<f64> {
    duree(valeur).filter(|d| (0.2..=600.0).contains(d))
}

/// Lit `Enter(…)` ou `Loop(…)`.
pub fn lire(valeur: &Valeur, parametre: &str) -> Result<Mouvement, Erreur> {
    let attendu = if parametre == "enter" { "Enter" } else { "Loop" };
    let bloc = match valeur {
        Valeur::Bloc(bloc) if bloc.nom == attendu => bloc,
        _ => {
            return Err(Erreur {
                message: format!("« {parametre}: » attend {attendu}(…), comme {parametre}: {attendu}(y: 40px, opacity: 0)"),
                pos: crate::holo::Pos { ligne: 0, colonne: 0 },
            })
        }
    };
    let boucle = attendu == "Loop";
    let mut mouvement = Mouvement { boucle, pose: Pose::default(), a: 0.0, duree: if boucle { 1.0 } else { 0.8 }, courbe: if boucle { COURBES[1].1 } else { COURBES[2].1 }, lettres: None, chacun: None, retour: true };
    let possibles = || {
        let mut noms: Vec<&str> = PROPRIETES.iter().map(|(n, ..)| *n).collect();
        noms.extend(["at", "for", "ease", "letters", "each"]);
        if boucle {
            noms.push("back");
        }
        noms.join(", ")
    };
    for argument in &bloc.arguments {
        let Some(nom) = argument.nom.as_deref() else {
            return Err(erreur(format!("chaque paramètre de « {attendu} » est nommé : {attendu}(y: 40px, opacity: 0)"), bloc));
        };
        let v = &argument.valeur;
        let hors = |quoi: String| Erreur { message: quoi, pos: argument.pos };
        match nom {
            "at" => mouvement.a = duree(v).filter(|d| (0.0..=600.0).contains(d)).ok_or_else(|| hors(format!("« {attendu}(at: …) » attend un moment de 0s à 600s, comme at: 1.5s")))?,
            "for" => mouvement.duree = duree(v).filter(|d| (0.05..=600.0).contains(d)).ok_or_else(|| hors(format!("« {attendu}(for: …) » attend une durée de 50ms à 600s, comme for: 0.8s")))?,
            "letters" => mouvement.lettres = Some(duree(v).filter(|d| (0.005..=2.0).contains(d)).ok_or_else(|| hors(format!("« {attendu}(letters: …) » attend l'écart entre deux lettres, de 5ms à 2s, comme letters: 0.04s")))?),
            "each" => mouvement.chacun = Some(duree(v).filter(|d| (0.01..=10.0).contains(d)).ok_or_else(|| hors(format!("« {attendu}(each: …) » attend l'écart entre deux enfants, de 10ms à 10s, comme each: 0.1s")))?),
            "ease" => {
                mouvement.courbe = match v {
                    Valeur::Nom(mot) => COURBES.iter().find(|(connu, _)| connu == mot).map(|(_, css)| *css),
                    _ => None,
                }
                .ok_or_else(|| hors(format!("« {attendu}(ease: …) » attend l'une de ces courbes : {}", COURBES.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", "))))?
            }
            "back" if boucle => match v {
                Valeur::Bool(b) => mouvement.retour = *b,
                _ => return Err(hors("« Loop(back: …) » attend true ou false".into())),
            },
            _ => match PROPRIETES.iter().find(|(connu, ..)| *connu == nom) {
                Some((propriete, unite, min, max)) => {
                    let lu = nombre(v).filter(|(n, u)| *u == *unite && (*min..=*max).contains(n)).map(|(n, _)| n);
                    let exemple = match unite {
                        Some(u) => format!("un nombre en {u}, de {min} à {max}"),
                        None => format!("un nombre de {min} à {max}"),
                    };
                    let n = lu.ok_or_else(|| hors(format!("« {attendu}({propriete}: …) » attend {exemple}")))?;
                    mouvement.pose.0.push((propriete, n));
                }
                None => return Err(hors(format!("« {attendu} » n'a pas de paramètre « {nom} » ; paramètres possibles : {}", possibles()))),
            },
        }
    }
    if mouvement.pose.0.is_empty() {
        return Err(erreur(format!("« {attendu} » dit ce qui bouge : {attendu}(y: 40px), {attendu}(scale: 1.2), {attendu}(opacity: 0)…"), bloc));
    }
    Ok(mouvement)
}

/// La scène où l'on est en train de fabriquer la page : son début, et la durée d'un tour
/// quand les scènes recommencent sans fin.
#[derive(Debug, Clone, Copy, Default)]
struct Scene {
    debut: f64,
    tour: Option<f64>,
}

#[derive(Default)]
struct Fabrique {
    profondeur: u32,
    css: String,
    numero: u32,
    scene: Scene,
}

thread_local! {
    static FABRIQUE: RefCell<Fabrique> = RefCell::new(Fabrique::default());
}

/// On commence à fabriquer une page.
pub fn commencer() {
    FABRIQUE.with(|f| {
        let mut f = f.borrow_mut();
        if f.profondeur == 0 {
            *f = Fabrique::default();
        }
        f.profondeur += 1;
    });
}

/// La page est fabriquée : le CSS des mouvements qu'elle contient.
pub fn terminer() -> String {
    FABRIQUE.with(|f| {
        let mut f = f.borrow_mut();
        f.profondeur = f.profondeur.saturating_sub(1);
        if f.profondeur == 0 {
            std::mem::take(&mut f.css)
        } else {
            String::new()
        }
    })
}

fn numero() -> u32 {
    FABRIQUE.with(|f| {
        let mut f = f.borrow_mut();
        f.numero += 1;
        f.numero
    })
}

fn ajouter(css: &str) {
    FABRIQUE.with(|f| f.borrow_mut().css.push_str(css));
}

fn scene() -> Scene {
    FABRIQUE.with(|f| f.borrow().scene)
}

/// Fabrique ce qui est dans une scène : ses mouvements partent de son début.
pub fn dans_la_scene<T>(debut: f64, tour: Option<f64>, f: impl FnOnce() -> T) -> T {
    let avant = FABRIQUE.with(|fab| std::mem::replace(&mut fab.borrow_mut().scene, Scene { debut, tour }));
    let resultat = f();
    FABRIQUE.with(|fab| fab.borrow_mut().scene = avant);
    resultat
}

fn pourcent(t: f64, tour: f64) -> String {
    format!("{:.3}%", (t / tour * 100.0).clamp(0.0, 100.0))
}

/// L'animation d'un mouvement : la règle `animation:` et ses images clés, sous ce nom.
fn animation(m: &Mouvement, nom: &str) -> (String, String) {
    let Scene { debut, tour } = scene();
    let depart = debut + m.a;
    let (depuis, vers) = if m.boucle { (m.pose.css(true), m.pose.css(false)) } else { (m.pose.css(false), m.pose.css(true)) };
    match (m.boucle, tour) {
        // Une boucle tourne à son rythme, une fois sa scène commencée.
        (true, _) => {
            let sens = if m.retour { "alternate" } else { "normal" };
            (format!("{nom} {}s {} {}s infinite {sens} both", m.duree, m.courbe, depart), format!("@keyframes {nom}{{from{{{depuis}}}to{{{vers}}}}}"))
        }
        // Une entrée, une seule fois.
        (false, None) => (format!("{nom} {}s {} {}s both", m.duree, m.courbe, depart), format!("@keyframes {nom}{{from{{{depuis}}}to{{{vers}}}}}")),
        // Une entrée dans des scènes qui recommencent : elle se rejoue à chaque tour.
        (false, Some(tour)) => (
            format!("{nom} {tour}s linear infinite both"),
            format!(
                "@keyframes {nom}{{0%{{{depuis}}}{}{{{depuis}animation-timing-function:{}}}{}{{{vers}}}100%{{{vers}}}}}",
                pourcent(depart, tour),
                m.courbe,
                pourcent(depart + m.duree, tour)
            ),
        ),
    }
}

/// Le décalage de chaque lettre ou de chaque enfant, ajouté au départ.
fn decale(regle: &str, ecart: f64, rang: &str) -> String {
    format!("{regle};animation-delay:calc({} + {rang} * {ecart}s)", delai_de(regle))
}

/// Le départ écrit dans une règle `animation:` (le dernier temps, ou 0s).
fn delai_de(regle: &str) -> String {
    let temps: Vec<&str> = regle.split(' ').filter(|m| m.ends_with('s') && m.trim_end_matches('s').parse::<f64>().is_ok()).collect();
    if temps.len() >= 2 { temps[1].to_string() } else { "0s".to_string() }
}

/// Coupe un HTML en lettres, chacune dans sa boîte, en gardant les mots entiers.
fn en_lettres(html: &str) -> String {
    let mut sortie = String::with_capacity(html.len() * 4);
    let mut rang = 0;
    let mut dans_un_mot = false;
    let mut caracteres = html.char_indices().peekable();
    while let Some((i, c)) = caracteres.next() {
        match c {
            '<' => {
                if dans_un_mot {
                    sortie.push_str("</span>");
                    dans_un_mot = false;
                }
                let fin = html[i..].find('>').map_or(html.len(), |f| i + f + 1);
                sortie.push_str(&html[i..fin]);
                while caracteres.peek().is_some_and(|(j, _)| *j < fin) {
                    caracteres.next();
                }
            }
            c if c.is_whitespace() => {
                if dans_un_mot {
                    sortie.push_str("</span>");
                    dans_un_mot = false;
                }
                sortie.push(' ');
            }
            _ => {
                if !dans_un_mot {
                    sortie.push_str("<span class=\"holo-mot\">");
                    dans_un_mot = true;
                }
                // Une entité (&amp;) reste une seule lettre.
                let lettre = if c == '&' {
                    let fin = html[i..].find(';').map_or(i + 1, |f| i + f + 1);
                    while caracteres.peek().is_some_and(|(j, _)| *j < fin) {
                        caracteres.next();
                    }
                    &html[i..fin]
                } else {
                    &html[i..i + c.len_utf8()]
                };
                let _ = write!(sortie, "<span class=\"holo-lettre\" style=\"--i:{rang}\">{lettre}</span>");
                rang += 1;
            }
        }
    }
    if dans_un_mot {
        sortie.push_str("</span>");
    }
    sortie
}

/// Les mouvements d'un bloc, et le bloc sans eux (pour le fabriquer comme d'habitude).
pub fn du_bloc(bloc: &Bloc) -> Result<Option<(Vec<Mouvement>, Bloc)>, Erreur> {
    // Un son ne se voit pas : rien à faire bouger (son « loop » serait autre chose).
    if bloc.nom == "Sound" || !bloc.arguments.iter().any(|a| a.nom.as_deref().is_some_and(|n| PARAMETRES.contains(&n))) {
        return Ok(None);
    }
    let mut mouvements = Vec::new();
    let mut reste = bloc.clone();
    reste.arguments.retain(|a| !a.nom.as_deref().is_some_and(|n| PARAMETRES.contains(&n)));
    // La boucle d'abord : elle enveloppe l'entrée, pour que les deux ne se gênent pas.
    for parametre in ["loop", "enter"] {
        if let Some(argument) = bloc.argument(parametre) {
            let m = lire(&argument.valeur, parametre).map_err(|e| Erreur { pos: argument.pos, ..e })?;
            if m.lettres.is_some() && !matches!(bloc.nom.as_str(), "H1" | "H2" | "H3" | "H4" | "H5" | "H6" | "P" | "Text" | "Button" | "Quote") {
                return Err(Erreur { message: "« letters: » coupe un texte en lettres : il va sur un titre (H1 à H6), P, Text, Button ou Quote".into(), pos: argument.pos });
            }
            if m.chacun.is_some() && !matches!(bloc.argument("children").map(|a| &a.valeur), Some(Valeur::Liste(_))) {
                return Err(Erreur { message: "« each: » fait bouger les enfants l'un après l'autre : il va sur un bloc qui a des « children »".into(), pos: argument.pos });
            }
            mouvements.push(m);
        }
    }
    Ok(Some((mouvements, reste)))
}

/// Enveloppe le HTML d'un bloc dans ses mouvements.
pub fn envelopper(mouvements: &[Mouvement], dedans: String, enfants: usize, sortie: &mut String) {
    let mut html = dedans;
    for m in mouvements.iter().rev() {
        let n = numero();
        let nom = format!("hm{n}");
        let (regle, images) = animation(m, &nom);
        let mut css = images;
        let arrondi = if m.pose.a("round").is_some() { "overflow:hidden;" } else { "" };
        if let Some(ecart) = m.lettres {
            let _ = write!(css, ".{nom}{{{arrondi}}}.{nom} .holo-lettre{{animation:{}}}", decale(&regle, ecart, "var(--i)"));
            html = en_lettres(&html);
        } else if let Some(ecart) = m.chacun {
            let _ = write!(css, ".{nom}{{{arrondi}}}");
            for rang in 0..enfants {
                let _ = write!(css, ".{nom}>*>:nth-child({}){{animation:{}}}", rang + 1, decale(&regle, ecart, &rang.to_string()));
            }
        } else {
            let _ = write!(css, ".{nom}{{{arrondi}animation:{regle}}}");
        }
        ajouter(&css);
        html = format!("<div class=\"holo-anime {nom}\">{html}</div>");
    }
    sortie.push_str(&html);
}

/// Le CSS d'une scène : elle apparaît à son début et s'efface à sa fin ; la dernière reste,
/// sauf quand les scènes recommencent.
fn scene_css(nom: &str, debut: f64, duree: f64, tour: Option<f64>, derniere: bool) -> String {
    let fondu = (duree / 4.0).min(0.4);
    match tour {
        None => {
            let f = pourcent(fondu, duree);
            let fin = if derniere { "100%{opacity:1;visibility:visible}".to_string() } else { format!("{}{{opacity:1}}100%{{opacity:0;visibility:hidden}}", pourcent(duree - fondu, duree)) };
            format!("@keyframes {nom}{{0%{{opacity:0;visibility:visible}}{f}{{opacity:1}}{fin}}}.{nom}{{animation:{nom} {duree}s linear {debut}s both}}")
        }
        Some(tour) => format!(
            "@keyframes {nom}{{0%{{opacity:0;visibility:hidden}}{}{{opacity:0;visibility:visible}}{}{{opacity:1}}{}{{opacity:1}}{}{{opacity:0;visibility:hidden}}100%{{opacity:0;visibility:hidden}}}}.{nom}{{animation:{nom} {tour}s linear infinite both}}",
            pourcent(debut, tour),
            pourcent(debut + fondu, tour),
            pourcent(debut + duree - fondu, tour),
            pourcent(debut + duree, tour)
        ),
    }
}

/// Une nouvelle scène : son CSS est ajouté à la page, et son nom rendu.
pub fn nouvelle_scene(debut: f64, duree: f64, tour: Option<f64>, derniere: bool) -> String {
    let nom = format!("hs{}", numero());
    ajouter(&scene_css(&nom, debut, duree, tour, derniere));
    nom
}

/// Le CSS de base du mouvement, ajouté à celui de la page.
pub const BASE: &str = ":where(.holo-anime){display:block}\
:where(.holo-mot){display:inline-block;white-space:nowrap}:where(.holo-lettre){display:inline-block}\
:where(.holo-Scenes){position:relative;overflow:hidden;width:100%}\
:where(.holo-Scene){position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:16px;text-align:center;opacity:0;padding:16px;box-sizing:border-box}\
:where(.holo-Scene)>*{margin:0}\
@media (prefers-reduced-motion:reduce){.holo-anime,.holo-anime *,.holo-Scene{animation:none!important}.holo-Scene{opacity:0;visibility:hidden}.holo-Scene:last-child{opacity:1;visibility:visible}}";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_lettre_par_boite_et_les_mots_entiers() {
        assert_eq!(
            en_lettres("Hi <strong>you</strong> &amp; me"),
            "<span class=\"holo-mot\"><span class=\"holo-lettre\" style=\"--i:0\">H</span><span class=\"holo-lettre\" style=\"--i:1\">i</span></span> <strong><span class=\"holo-mot\"><span class=\"holo-lettre\" style=\"--i:2\">y</span><span class=\"holo-lettre\" style=\"--i:3\">o</span><span class=\"holo-lettre\" style=\"--i:4\">u</span></span></strong> <span class=\"holo-mot\"><span class=\"holo-lettre\" style=\"--i:5\">&amp;</span></span> <span class=\"holo-mot\"><span class=\"holo-lettre\" style=\"--i:6\">m</span><span class=\"holo-lettre\" style=\"--i:7\">e</span></span>"
        );
    }
}
