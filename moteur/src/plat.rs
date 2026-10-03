//! La vue à plat : d'un fichier `.holo` vérifié, le moteur fabrique une page web ordinaire,
//! du HTML et du CSS (ADR-007, ADR-011). L'auteur n'en écrit jamais lui-même.
//!
//! Le monde à l'intérieur d'un point est rendu comme un panneau lisible, caché tant qu'on
//! n'y est pas entré (ADR-018, option A) : la page qui accueille le moteur le pose devant
//! la vue en profondeur.

use crate::holo::{Bloc, Cible, Erreur, Programme, Valeur};
use crate::regles::nom_de;
use crate::styles::est_couleur;

/// La disposition et l'allure de base, que l'auteur n'a pas à écrire. `:where` laisse
/// toujours le dernier mot aux styles du fichier.
const BASE: &str = "\
:where(.holo-Page){min-height:100vh;box-sizing:border-box;margin:0}\
:where(.holo-Page>main){max-width:640px;margin:0 auto;position:relative}:where(.holo-Page>main,.holo-panneau)>*{display:block;box-sizing:border-box;margin:0 0 16px 0}\
:where(.holo-Button){font:inherit;color:inherit;cursor:pointer;background:transparent;border:1px solid currentColor;border-radius:6px;padding:6px 12px}\
:where(.holo-Point){width:64px;height:64px;padding:0;border:0;border-radius:50%;cursor:pointer;\
background:radial-gradient(circle,white 0%,var(--holo-color,white) 35%,transparent 70%);opacity:var(--holo-brightness,1)}\
:where(.holo-pixel){position:absolute;width:1px;height:1px;margin:0;padding:0;background:var(--holo-color,white);cursor:pointer}:where(.holo-World){position:fixed;inset:0;margin:0;pointer-events:none}\
:where(.holo-World[hidden]){display:none}\
:where(.holo-panneau){position:absolute;z-index:1;left:0;right:0;bottom:0;max-height:46vh;overflow:auto;padding:16px max(16px,calc(50% - 320px));\
box-sizing:border-box;background:rgba(0,0,0,0.6);pointer-events:auto}";

/// Fabrique la page. `base` est le dossier du fichier `.holo`, pour retrouver ses images.
/// Le fichier doit avoir passé les vérifications (`crate::verifier_page`).
pub fn page_html(programme: &Programme, base: &str) -> Result<String, Erreur> {
    site_html(programme, &programme.racine, base, "")
}

/// Fabrique la page d'un site. Un site est la `Page` du fichier, ou le monde contenu dans
/// l'un de ses points : ouvert en grand, ce monde se regarde exactement comme une page, avec
/// ses propres points, dans lesquels on peut entrer à leur tour. `titre` sert au monde, qui
/// n'en a pas.
pub fn site_html(programme: &Programme, page: &Bloc, base: &str, titre: &str) -> Result<String, Erreur> {
    if page.nom != "Page" && page.nom != "World" {
        return Err(Erreur { message: format!("la vue à plat affiche une « Page » ; ce fichier commence par « {} »", page.nom), pos: page.pos });
    }
    let mut corps = String::new();
    let mut mondes = String::new();
    enfants(page, &mut corps, &mut mondes, base)?;
    if let Some(Valeur::Liste(plantes)) = page.argument("pixels").map(|a| &a.valeur) {
        for plante in plantes {
            pixel_plante(plante, &mut corps, &mut mondes, base, page)?;
        }
    }
    let titre = match page.argument("title").map(|a| &a.valeur) {
        Some(Valeur::Texte(t)) => echapper(t),
        _ => echapper(titre),
    };
    // Un monde ouvert en grand prend le thème de la page, puis son propre style.
    let classes = match (page.nom.as_str(), &page.style) {
        ("World", Some(style)) => format!("holo-Page holo-monde-ouvert holo-s-{style}"),
        ("World", None) => "holo-Page holo-monde-ouvert".to_string(),
        _ => classes(page),
    };
    Ok(format!(
        "<style>{BASE}{}</style><div class=\"{classes}\" data-title=\"{titre}\"><main>{corps}</main>{mondes}</div>",
        css(programme)
    ))
}

/// Le CSS des styles du fichier. Le thème d'abord, puis les types, puis les styles nommés :
/// à précision égale, le dernier écrit l'emporte, ce qui donne la priorité voulue (ADR-017).
fn css(programme: &Programme) -> String {
    let rang = |cible: &Cible| match cible {
        Cible::Type(t) if t == "Page" || t == "World" => 0,
        Cible::Type(_) => 1,
        Cible::Nom(_) => 2,
    };
    let mut regles: Vec<_> = programme.styles.iter().collect();
    regles.sort_by_key(|r| rang(&r.cible));
    let mut sortie = String::new();
    for regle in regles {
        let selecteur = match &regle.cible {
            // Le style d'un monde vaut aussi quand ce monde est ouvert en grand.
            Cible::Type(t) if t == "World" => ".holo-World,.holo-monde-ouvert".to_string(),
            Cible::Type(t) => format!(".holo-{t}"),
            Cible::Nom(n) => format!(".holo-s-{n}"),
        };
        sortie.push_str(&selecteur);
        sortie.push('{');
        for reglage in &regle.reglages {
            sortie.push_str(&format!("{}:{};", reglage.nom, reglage.valeur.replace('<', "")));
        }
        sortie.push('}');
    }
    sortie
}

fn classes(bloc: &Bloc) -> String {
    match &bloc.style {
        Some(style) => format!("holo-{} holo-s-{style}", bloc.nom),
        None => format!("holo-{}", bloc.nom),
    }
}

fn enfants(bloc: &Bloc, sortie: &mut String, mondes: &mut String, base: &str) -> Result<(), Erreur> {
    if let Some(Valeur::Liste(elements)) = bloc.argument("children").map(|a| &a.valeur) {
        for element in elements {
            rendre(element, sortie, mondes, base, bloc)?;
        }
    }
    Ok(())
}

fn rendre(valeur: &Valeur, sortie: &mut String, mondes: &mut String, base: &str, parent: &Bloc) -> Result<(), Erreur> {
    let bloc = match valeur {
        // Une phrase seule est un paragraphe (ADR-019, ADR-020).
        Valeur::Texte(texte) => {
            sortie.push_str(&format!("<p class=\"holo-P\">{}</p>", markdown(texte)));
            return Ok(());
        }
        Valeur::Bloc(bloc) => bloc,
        _ => return Err(Erreur { message: "« children » contient des blocs ou des phrases entre guillemets".into(), pos: parent.pos }),
    };
    let classes = classes(bloc);
    let nom = nom_de(bloc).map(|n| format!(" data-name=\"{}\"", echapper(n))).unwrap_or_default();
    match bloc.nom.as_str() {
        "H1" | "H2" | "H3" | "P" | "Text" => {
            let balise = match bloc.nom.as_str() {
                "P" => "p".to_string(),
                "Text" => "div".to_string(),
                titre => titre.to_ascii_lowercase(),
            };
            sortie.push_str(&format!("<{balise} class=\"{classes}\"{nom}>{}</{balise}>", markdown(texte_de(bloc)?)));
        }
        "Button" => {
            let texte = match bloc.argument("text").map(|a| &a.valeur) {
                Some(Valeur::Texte(t)) => t,
                _ => return Err(Erreur { message: "« Button » attend un paramètre « text » entre guillemets".into(), pos: bloc.pos }),
            };
            sortie.push_str(&format!("<button type=\"button\" class=\"{classes}\"{nom}>{}</button>", markdown(texte)));
        }
        "Image" => {
            let source = match bloc.argument("source").map(|a| &a.valeur) {
                Some(Valeur::Texte(s)) if chemin_sur(s) => s,
                _ => {
                    return Err(Erreur {
                        message: "« Image » attend un paramètre « source » : un fichier rangé à côté du .holo, comme \"painting.png\"".into(),
                        pos: bloc.pos,
                    })
                }
            };
            sortie.push_str(&format!("<img class=\"{classes}\"{nom} src=\"{}{}\" alt=\"\">", echapper(base), echapper(source)));
        }
        "List" => {
            // `ordered: true` : une liste numérotée.
            let balise = if matches!(bloc.argument("ordered").map(|a| &a.valeur), Some(Valeur::Bool(true))) { "ol" } else { "ul" };
            sortie.push_str(&format!("<{balise} class=\"{classes}\"{nom}>"));
            if let Some(Valeur::Liste(elements)) = bloc.argument("children").map(|a| &a.valeur) {
                for element in elements {
                    sortie.push_str("<li>");
                    match element {
                        Valeur::Texte(texte) => sortie.push_str(&markdown(texte)),
                        autre => rendre(autre, sortie, mondes, base, bloc)?,
                    }
                    sortie.push_str("</li>");
                }
            }
            sortie.push_str(&format!("</{balise}>"));
        }
        // Le lien classique : on quitte la page pour une autre adresse, comme <a href> en HTML.
        "A" => {
            let adresse = match bloc.argument("to").map(|a| &a.valeur) {
                Some(Valeur::Texte(adresse)) => adresse_sure(adresse, base),
                _ => None,
            };
            let Some(adresse) = adresse else {
                return Err(Erreur {
                    message: "« A » attend un paramètre « to » : un fichier rangé à côté (\"garden.holo\") ou une adresse du web (\"https://…\")".into(),
                    pos: bloc.pos,
                });
            };
            sortie.push_str(&format!("<a class=\"{classes}\"{nom} href=\"{}\">{}</a>", echapper(&adresse), markdown(texte_de(bloc)?)));
        }
        "Point" => {
            let allure = allure_du_point(bloc)?;
            let etiquette = nom_de(bloc).map(|n| format!(" aria-label=\"{}\"", echapper(n))).unwrap_or_default();
            // `inside: "garden.holo"` : le monde de ce point est un autre fichier. On y passe
            // sans changer de page ; la page d'entrée va le chercher.
            let fichier = match bloc.argument("inside").map(|a| &a.valeur) {
                Some(Valeur::Texte(fichier)) if chemin_sur(fichier) && fichier.ends_with(".holo") => format!(" data-file=\"{}{}\"", echapper(base), echapper(fichier)),
                // Le fichier d'un autre auteur, sur un autre serveur : son adresse complète.
                Some(Valeur::Texte(adresse)) if adresse_web(adresse) && adresse.ends_with(".holo") => format!(" data-file=\"{}\"", echapper(adresse)),
                Some(Valeur::Texte(_)) => {
                    return Err(Erreur {
                        message: "« inside » attend un monde, un fichier .holo rangé à côté (\"garden.holo\"), ou l'adresse complète d'un fichier .holo (\"https://…/garden.holo\")".into(),
                        pos: bloc.pos,
                    })
                }
                _ => String::new(),
            };
            sortie.push_str(&format!("<button type=\"button\" class=\"{classes}\"{nom}{etiquette}{fichier} style=\"{allure}\"></button>"));
            monde_interieur(bloc, mondes, base)?;
        }
        autre => return Err(Erreur { message: format!("« {autre} » ne se place pas dans « children »"), pos: bloc.pos }),
    }
    Ok(())
}

/// La couleur et la lumière d'un point, pour la page : celles que l'auteur impose, sinon
/// celles que donne la graine, la même que dans la vue en profondeur.
fn allure_du_point(bloc: &Bloc) -> Result<String, Erreur> {
    let mut allure = String::new();
    if let Some(Valeur::Texte(couleur)) = bloc.argument("color").map(|a| &a.valeur) {
        if !est_couleur(couleur) {
            return Err(Erreur { message: format!("« color: \"{couleur}\" » : une couleur est attendue, comme \"#E9B44C\""), pos: bloc.pos });
        }
        allure.push_str(&format!("--holo-color:{couleur};"));
    } else if let Some(Valeur::Entier(graine)) = bloc.argument("seed").map(|a| &a.valeur) {
        let [r, v, b] = crate::univers::Monde::depuis_graine(*graine).couleur.map(|c| (c * 255.0).round() as u8);
        allure.push_str(&format!("--holo-color:rgb({r},{v},{b});"));
    }
    match bloc.argument("brightness").map(|a| &a.valeur) {
        Some(Valeur::Nombre { valeur, unite: None }) => allure.push_str(&format!("--holo-brightness:{valeur};")),
        Some(Valeur::Entier(entier)) => allure.push_str(&format!("--holo-brightness:{entier};")),
        _ => {}
    }
    Ok(allure)
}

/// Le monde à l'intérieur d'un point : son contenu, caché tant qu'on n'y est pas entré.
fn monde_interieur(point: &Bloc, mondes: &mut String, base: &str) -> Result<(), Erreur> {
    if let (Some(Valeur::Bloc(monde)), Some(nom_du_point)) = (point.argument("inside").map(|a| &a.valeur), nom_de(point)) {
        let mut panneau = String::new();
        let mut plus_profond = String::new();
        enfants(monde, &mut panneau, &mut plus_profond, base)?;
        mondes.push_str(&format!(
            "<section class=\"{}\" data-world=\"{}\" hidden><div class=\"holo-panneau\">{panneau}</div></section>{plus_profond}",
            classes(monde),
            echapper(nom_du_point)
        ));
    }
    Ok(())
}

/// Un point planté dans un pixel de la page (`pixels:` d'une `Page`). Au repos il occupe un
/// seul pixel : on ne le remarque qu'en s'approchant. La page d'entrée le place juste
/// au-dessus du bloc nommé par `above`, à l'extrémité droite.
fn pixel_plante(valeur: &Valeur, sortie: &mut String, mondes: &mut String, base: &str, page: &Bloc) -> Result<(), Erreur> {
    let point = match valeur {
        Valeur::Bloc(bloc) if bloc.nom == "Point" => bloc,
        _ => return Err(Erreur { message: "« pixels » contient des blocs « Point(...) »".into(), pos: page.argument("pixels").map_or(page.pos, |a| a.pos) }),
    };
    let (Some(nom), Some(Valeur::Nom(repere))) = (nom_de(point), point.argument("above").map(|a| &a.valeur)) else {
        return Err(Erreur { message: "un point planté dans un pixel a un nom et un repère : Point(name: Secret, above: Open, ...)".into(), pos: point.pos });
    };
    sortie.push_str(&format!(
        "<i class=\"holo-pixel\" data-name=\"{}\" data-above=\"{}\" style=\"{}\"></i>",
        echapper(nom),
        echapper(repere),
        allure_du_point(point)?
    ));
    monde_interieur(point, mondes, base)
}

/// Le texte d'un bloc : `P("Bonjour")`.
fn texte_de(bloc: &Bloc) -> Result<&str, Erreur> {
    bloc.arguments
        .iter()
        .find_map(|a| match (&a.nom, &a.valeur) {
            (None, Valeur::Texte(t)) => Some(t.as_str()),
            _ => None,
        })
        .ok_or_else(|| Erreur { message: format!("« {} » attend un texte entre guillemets : {}(\"…\")", bloc.nom, bloc.nom), pos: bloc.pos })
}

/// L'adresse d'un lien : un fichier rangé à côté (rendu relatif au dossier du `.holo`), un
/// site de la page (`#Workshop`), ou une adresse du web en http ou https. Rien d'autre : pas
/// de `javascript:`, pas de caractères qui sortiraient de l'attribut.
fn adresse_sure(adresse: &str, base: &str) -> Option<String> {
    if adresse.starts_with("https://") || adresse.starts_with("http://") {
        return adresse_web(adresse).then(|| adresse.to_string());
    }
    let (fichier, ancre) = adresse.split_once('#').unwrap_or((adresse, ""));
    let ancre_sure = ancre.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/'));
    match (fichier.is_empty(), ancre.is_empty()) {
        (true, false) if ancre_sure => Some(format!("#{ancre}")),
        (false, _) if chemin_sur(fichier) && ancre_sure => Some(if ancre.is_empty() { format!("{base}{fichier}") } else { format!("{base}{fichier}#{ancre}") }),
        _ => None,
    }
}

/// Une adresse du web, en http ou https, sans caractère qui sortirait d'un attribut.
fn adresse_web(adresse: &str) -> bool {
    (adresse.starts_with("https://") || adresse.starts_with("http://"))
        && adresse.len() > 8
        && adresse.chars().all(|c| c.is_ascii_graphic() && !matches!(c, '"' | '<' | '>' | '\\' | '`'))
}

/// Une image se range à côté du fichier : ni adresse complète, ni remontée de dossier.
fn chemin_sur(source: &str) -> bool {
    !source.is_empty()
        && !source.starts_with('/')
        && !source.contains("..")
        && source.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
}

fn echapper(texte: &str) -> String {
    texte.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Le Markdown d'une ligne : `**gras**` et `*italique*`. Le reste du Markdown viendra.
fn markdown(texte: &str) -> String {
    fn alterner(texte: &str, marque: &str, balise: &str) -> String {
        let morceaux: Vec<&str> = texte.split(marque).collect();
        if morceaux.len() % 2 == 0 {
            // Une marque jamais refermée reste du texte ordinaire.
            return texte.to_string();
        }
        morceaux
            .iter()
            .enumerate()
            .map(|(i, m)| if i % 2 == 1 { format!("<{balise}>{m}</{balise}>") } else { m.to_string() })
            .collect()
    }
    alterner(&alterner(&echapper(texte), "**", "strong"), "*", "em")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holo::lire;

    fn page(src: &str) -> Result<String, Erreur> {
        page_html(&lire(src)?, "")
    }

    #[test]
    fn la_boutique_devient_une_page_web() {
        let html = page_html(&lire(include_str!("../../exemples/boutique-comparee/boutique.holo")).unwrap(), "/ex/").unwrap();
        for attendu in [
            "<div class=\"holo-Page\" data-title=\"My shop\"><main>",
            "<h1 class=\"holo-H1\">My shop</h1>",
            "<p class=\"holo-P\">Paintings made by hand, one at a time.</p>",
            "<p class=\"holo-P holo-s-card\">Free delivery from <strong>30 euros</strong>.</p>",
            "<img class=\"holo-Image\" src=\"/ex/painting.svg\" alt=\"\">",
            "<ul class=\"holo-List\"><li>Sunrise over the river</li>",
            "<div class=\"holo-Text holo-s-note\">Open until 6 pm</div>",
            "<button type=\"button\" class=\"holo-Button holo-s-card\" data-name=\"Open\">Enter the workshop</button>",
            "data-name=\"Workshop\" aria-label=\"Workshop\" style=\"--holo-color:#E9B44C;--holo-brightness:0.8;\"",
            "<section class=\"holo-World\" data-world=\"Workshop\" hidden><div class=\"holo-panneau\"><h1 class=\"holo-H1\">The workshop</h1>",
            "data-name=\"Back\">Back to the shop</button>",
            ".holo-s-card{background:#1a1a2e;border:1px solid #E9B44C;border-radius:12px;padding:8px 16px;}",
        ] {
            assert!(html.contains(attendu), "manque : {attendu}\n{html}");
        }
        // Le thème avant les types, les types avant les styles nommés.
        let place = |morceau: &str| html.find(morceau).unwrap();
        assert!(place(".holo-Page{") < place(".holo-H1{") && place(".holo-World,") < place(".holo-H1{") && place(".holo-P{") < place(".holo-s-card{"));
    }

    #[test]
    fn un_site_se_plante_dans_un_pixel_de_la_page() {
        let html = page(
            "Page(children: [ Button(name: Open, text: \"x\") ], pixels: [ Point(name: Secret, above: Open, seed: 7, color: \"#FF4D6D\", inside: World(children: [ H1(\"Hidden\") ])) ])",
        )
        .unwrap();
        assert!(html.contains("<i class=\"holo-pixel\" data-name=\"Secret\" data-above=\"Open\" style=\"--holo-color:#FF4D6D;\"></i>"), "{html}");
        assert!(html.contains("data-world=\"Secret\" hidden><div class=\"holo-panneau\"><h1 class=\"holo-H1\">Hidden</h1>"));
        assert!(page("Page(pixels: [ P(\"x\") ])").unwrap_err().message.contains("des blocs « Point(...) »"));
        assert!(page("Page(pixels: [ Point(name: A, seed: 1) ])").unwrap_err().message.contains("un nom et un repère"));
    }

    #[test]
    fn le_monde_d_un_point_s_ouvre_comme_une_page() {
        let programme = lire(
            "Page(title: \"Top\", children: [ Point(name: A, seed: 1, inside: World.rose(children: [ H1(\"Inside\"), Button(name: Out, text: \"x\") ], pixels: [ Point(name: B, above: Out, seed: 2, inside: World(children: [ P(\"Deeper\") ])) ])) ])\n.rose { background: #3a0d1a; }\nWorld { color: white; }",
        )
        .unwrap();
        let monde = crate::regles::site_de(&programme, "A").unwrap();
        let html = site_html(&programme, monde, "", "A").unwrap();
        assert!(html.contains("<div class=\"holo-Page holo-monde-ouvert holo-s-rose\" data-title=\"A\"><main><h1 class=\"holo-H1\">Inside</h1>"), "{html}");
        assert!(html.contains(".holo-World,.holo-monde-ouvert{color:white;}"));
        // Il a ses propres points plantés, et leurs mondes : la boucle continue.
        assert!(html.contains("<i class=\"holo-pixel\" data-name=\"B\" data-above=\"Out\""));
        assert!(html.contains("data-world=\"B\" hidden>"));
        let plus_profond = crate::regles::site_de(&programme, "A/B").unwrap();
        assert!(site_html(&programme, plus_profond, "", "B").unwrap().contains("Deeper"));
        assert!(crate::regles::site_de(&programme, "A/Nobody").unwrap_err().message.contains("aucun point"));
        assert_eq!(crate::regles::site_de(&programme, "").unwrap().nom, "Page");
    }

    #[test]
    fn les_liens_et_les_portes_vers_un_autre_fichier() {
        let html = page_html(
            &lire("Page(children: [ A(\"The garden\", to: \"garden.holo\"), A(\"Elsewhere\", to: \"https://example.com/a?b=1\"), A(\"Inside\", to: \"#Workshop\"), List(ordered: true, children: [ A(\"x\", to: \"a/b.holo#S\") ]), Point(name: Garden, seed: 3, inside: \"garden.holo\") ])").unwrap(),
            "/ex/",
        )
        .unwrap();
        for attendu in [
            "<a class=\"holo-A\" href=\"/ex/garden.holo\">The garden</a>",
            "<a class=\"holo-A\" href=\"https://example.com/a?b=1\">Elsewhere</a>",
            "<a class=\"holo-A\" href=\"#Workshop\">Inside</a>",
            "<ol class=\"holo-List\"><li><a class=\"holo-A\" href=\"/ex/a/b.holo#S\">x</a></li></ol>",
            "data-name=\"Garden\" aria-label=\"Garden\" data-file=\"/ex/garden.holo\"",
        ] {
            assert!(html.contains(attendu), "manque : {attendu}\n{html}");
        }
        // Un lien ne peut pas cacher de code, ni sortir de son dossier.
        for mauvais in ["javascript:alert(1)", "../secret.holo", "https://a.example/\\\"onclick=", "data:text/html,x", ""] {
            assert!(page(&format!("Page(children: [ A(\"x\", to: \"{mauvais}\") ])")).is_err(), "{mauvais}");
        }
        assert!(page("Page(children: [ A(\"x\") ])").unwrap_err().message.contains("« to »"));
        assert!(page("Page(children: [ Point(name: G, seed: 1, inside: \"garden.txt\") ])").unwrap_err().message.contains("fichier .holo"));
        // Le fichier d'un autre auteur, sur un autre serveur.
        let ailleurs = page("Page(children: [ Point(name: G, seed: 1, inside: \"https://friend.example/home/garden.holo\") ])").unwrap();
        assert!(ailleurs.contains("data-file=\"https://friend.example/home/garden.holo\""), "{ailleurs}");
        assert!(page("Page(children: [ Point(name: G, seed: 1, inside: \"https://friend.example/x.html\") ])").is_err());
        assert!(page("Page(children: [ Point(name: G, seed: 1, inside: \"javascript:x.holo\") ])").is_err());
    }

    #[test]
    fn le_meme_fichier_donne_la_meme_page() {
        let source = include_str!("../../exemples/boutique-comparee/boutique.holo");
        assert_eq!(page_html(&lire(source).unwrap(), ""), page_html(&lire(source).unwrap(), ""));
    }

    #[test]
    fn rien_de_ce_qu_ecrit_l_auteur_ne_devient_du_code() {
        let html = page("Page(title: \"<script>\", children: [ \"<script>alert(1)</script> & *ok*\" ])").unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt; &amp; <em>ok</em>"));
        assert!(page("Page(children: [ Image(source: \"https://ailleurs.example/x.png\") ])").unwrap_err().message.contains("à côté du .holo"));
        assert!(page("Page(children: [ Image(source: \"../secret.png\") ])").is_err());
        assert!(page("Page(children: [ Point(name: A, seed: 1, color: \"red;background:url(x)\") ])").is_err());
    }

    #[test]
    fn les_erreurs_sont_dites() {
        assert!(page("Point(name: A, seed: 1)").unwrap_err().message.contains("affiche une « Page »"));
        assert!(page("Page(children: [ H1() ])").unwrap_err().message.contains("attend un texte"));
        assert!(page("Page(children: [ Button(name: A) ])").unwrap_err().message.contains("« text »"));
        assert!(page("Page(children: [ Point(name: A, seed: 7) ])").unwrap().contains("--holo-color:rgb("));
        assert!(page("Page(children: [ World(children: []) ])").unwrap_err().message.contains("ne se place pas"));
        assert_eq!(markdown("2 * 3"), "2 * 3");
    }
}
