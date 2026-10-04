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
:where(.holo-pixel){position:absolute;width:1px;height:1px;margin:0;padding:0;border:0;background:var(--holo-color,white);cursor:pointer}:where(.holo-World){position:fixed;inset:0;margin:0;pointer-events:none}\
:where(.holo-World[hidden]){display:none}\
:where(.holo-panneau){position:absolute;z-index:1;left:0;right:0;bottom:0;max-height:46vh;overflow:auto;padding:16px max(16px,calc(50% - 320px));\
box-sizing:border-box;background:rgba(0,0,0,0.6);pointer-events:auto}\
:where(.holo-If){display:contents}:where(.holo-If)>*{display:block;box-sizing:border-box;margin:0 0 16px 0}\
:where(.holo-Row){display:flex;flex-wrap:wrap;align-items:center;gap:var(--holo-gap,16px);justify-content:var(--holo-align,flex-start)}\
:where(.holo-Column){display:flex;flex-direction:column;gap:var(--holo-gap,16px);align-items:var(--holo-align,stretch)}\
:where(.holo-Grid){display:grid;gap:var(--holo-gap,16px);\
grid-template-columns:repeat(auto-fill,minmax(min(100%,max(120px,calc((100% - (var(--holo-columns,2) - 1)*var(--holo-gap,16px))/var(--holo-columns,2)))),1fr))}\
:where(.holo-Row,.holo-Column,.holo-Grid)>*{margin:0;box-sizing:border-box;min-width:0}\
:where(.holo-Row,.holo-Column,.holo-Grid)>.holo-If>*{margin:0}:where(.holo-If[hidden]){display:none}\
:where(.holo-Input){display:flex;flex-direction:column;gap:4px;align-items:flex-start}\
:where(.holo-Input input){font:inherit;color:inherit;background:transparent;border:1px solid currentColor;border-radius:6px;padding:6px 10px;width:120px}\
:where(.holo-Input input[type=text]){width:min(100%,280px);box-sizing:border-box}\
:where(.holo-Checkbox){display:flex;align-items:center;gap:8px;cursor:pointer}\
:where(.holo-Checkbox input){width:18px;height:18px;margin:0;accent-color:currentColor}\
:where(.holo-Board){position:relative;overflow:hidden;border-radius:12px}\
:where(.holo-place){position:absolute;left:calc(var(--x)*1%);top:calc(var(--y)*1%);transform:translate(calc(var(--x)*-1%),calc(var(--y)*-1%));\
transition:left .12s linear,top .12s linear,transform .12s linear}\
:where(.holo-place[data-drag]){touch-action:none;cursor:grab}.holo-place.holo-glisse{transition:none;cursor:grabbing}\
@media (prefers-reduced-motion:reduce){.holo-place{transition:none}}\
:where(.holo-Sound){display:none}\
:where(.holo-Hr){border:0;border-top:1px solid currentColor;opacity:0.4;height:0}\
:where(.holo-Quote){border-left:3px solid currentColor;padding:0 0 0 12px;font-style:italic}\
:where(.holo-Quote>p){margin:0 0 4px 0}:where(.holo-Quote>footer){font-style:normal;font-size:0.9em;opacity:0.7}\
:where(.holo-Code){font-family:ui-monospace,Consolas,monospace;background:rgba(127,127,127,0.18);padding:8px 12px;border-radius:6px;overflow:auto;white-space:pre-wrap}\
:where(.holo-Page code){font-family:ui-monospace,Consolas,monospace;background:rgba(127,127,127,0.18);padding:0 4px;border-radius:4px}:where(.holo-Code code){background:none;padding:0}";

/// Entoure, dans le HTML en cours de fabrication, la condition d'un bloc `If` : `site_html`
/// la remplace par « hidden » quand elle est fausse au départ. Ce caractère ne peut pas venir
/// d'un texte de l'auteur : `echapper` le retire.
const MARQUE: char = '\u{1}';

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
    // Les valeurs de la page, à leur départ, là où un texte les montre : « {cart} » (ADR-023).
    let depart = crate::etat::initial(programme).unwrap_or_default();
    let montrees = crate::etat::a_montrer(programme, &depart);
    // Les conditions, à leur départ : ce qui est faux est caché dès le premier affichage (ADR-025).
    // La réponse vient de `etat::conditions`, comme après chaque changement : une condition
    // n'est décidée qu'à un seul endroit.
    let textes = crate::etat::textes_initiaux(programme);
    let reponses = crate::etat::conditions(programme, &crate::etat::avec_textes(&montrees, &textes));
    let texte_de_depart = |nom: &str| textes.iter().find(|(connu, _)| connu == nom).map(|(_, texte)| texte.as_str());
    let conditions = |html: String| -> String {
        let mut sortie = String::with_capacity(html.len());
        for (rang, morceau) in html.split(MARQUE).enumerate() {
            if rang % 2 == 0 {
                sortie.push_str(morceau);
            } else if let Some(champ) = morceau.strip_prefix('!') {
                // Un champ : de texte ou de nombre, selon la valeur qu'il présente.
                let (nom, max) = champ.split_once('|').unwrap_or((champ, ""));
                if texte_de_depart(nom).is_some() {
                    let longueur = max.parse::<usize>().map_or(crate::etat::TEXTE_COURANT, |m| m.min(crate::etat::TEXTE_MAX));
                    sortie.push_str(&format!(" type=\"text\" maxlength=\"{longueur}\""));
                } else {
                    sortie.push_str(" type=\"number\" inputmode=\"numeric\" min=\"0\"");
                    if !max.is_empty() {
                        sortie.push_str(&format!(" max=\"{max}\""));
                    }
                }
            } else if let Some(nom) = morceau.strip_prefix('#') {
                // Un champ : la valeur de départ, telle quelle.
                match texte_de_depart(nom) {
                    Some(texte) => sortie.push_str(&echapper(texte)),
                    None => sortie.push_str(&montrees.iter().find(|(connu, _)| connu == nom).map_or(0, |(_, v)| *v).to_string()),
                }
            } else if let Some(nom) = morceau.strip_prefix('?') {
                // Une case : cochée au départ si la valeur n'est pas zéro.
                if montrees.iter().any(|(connu, v)| connu == nom && *v > 0) {
                    sortie.push_str(" checked");
                }
            } else if let Some(nom) = morceau.strip_prefix('@') {
                // La place d'un bloc sur un plateau : la valeur de départ, de 0 à 100.
                let valeur = montrees.iter().find(|(connu, _)| connu == nom).map_or(0, |(_, v)| *v);
                sortie.push_str(&valeur.min(100).to_string());
            } else if !reponses.iter().any(|(cle, vraie)| cle == morceau && *vraie) {
                sortie.push_str(" hidden");
            }
        }
        sortie
    };
    corps = conditions(corps);
    mondes = conditions(mondes);
    // Les textes, à leur départ, là où un texte les montre.
    for (nom, texte) in &textes {
        let (vide, pleine) = (format!("<span data-state=\"{nom}\"></span>"), format!("<span data-state=\"{nom}\">{}</span>", echapper(texte)));
        corps = corps.replace(&vide, &pleine);
        mondes = mondes.replace(&vide, &pleine);
    }
    for (nom, valeur) in montrees.clone() {
        let (vide, pleine) = (format!("<span data-state=\"{nom}\"></span>"), format!("<span data-state=\"{nom}\">{valeur}</span>"));
        corps = corps.replace(&vide, &pleine);
        mondes = mondes.replace(&vide, &pleine);
    }
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
            // `alt` : le texte qui remplace l'image pour qui ne la voit pas. Sans lui, l'image
            // est tenue pour un décor, et un lecteur d'écran la passe.
            let alt = match bloc.argument("alt").map(|a| &a.valeur) {
                None => "",
                Some(Valeur::Texte(texte)) => texte.as_str(),
                Some(_) => return Err(Erreur { message: "« Image(alt: …) » attend un texte entre guillemets : ce que montre l'image".into(), pos: bloc.pos }),
            };
            sortie.push_str(&format!("<img class=\"{classes}\"{nom} src=\"{}{}\" alt=\"{}\">", echapper(base), echapper(source), echapper(alt)));
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
        // Une condition : ce qu'elle contient ne se montre que si elle est vraie (ADR-025).
        "If" => {
            let (valeur, comparaisons) = crate::etat::condition(bloc)?;
            let cle = crate::etat::cle(valeur, &comparaisons);
            sortie.push_str(&format!("<div class=\"{classes}\"{nom} data-if=\"{}\"{MARQUE}{cle}{MARQUE}>", echapper(&cle)));
            enfants(bloc, sortie, mondes, base)?;
            sortie.push_str("</div>");
        }
        // Un champ où le visiteur écrit un nombre, et une case qu'il coche. Chacun présente une
        // valeur de la page ; l'étiquette est obligatoire (ADR-027). `etat.rs` les a vérifiés.
        "Input" | "Checkbox" => {
            let (Some(Valeur::Nom(valeur)), Some(Valeur::Texte(etiquette))) = (bloc.argument("value").map(|a| &a.valeur), bloc.argument("label").map(|a| &a.valeur)) else {
                return Err(Erreur { message: format!("« {} » attend « value » et « label »", bloc.nom), pos: bloc.pos });
            };
            let valeur = echapper(valeur);
            if bloc.nom == "Input" {
                let max = match bloc.argument("max").map(|a| &a.valeur) {
                    Some(Valeur::Entier(max)) => max.to_string(),
                    _ => String::new(),
                };
                sortie.push_str(&format!(
                    "<label class=\"{classes}\"{nom}><span>{}</span><input{MARQUE}!{valeur}|{max}{MARQUE} value=\"{MARQUE}#{valeur}{MARQUE}\" data-bind=\"{valeur}\"></label>",
                    markdown(etiquette)
                ));
            } else {
                sortie.push_str(&format!(
                    "<label class=\"{classes}\"{nom}><input type=\"checkbox\" data-bind=\"{valeur}\"{MARQUE}?{valeur}{MARQUE}><span>{}</span></label>",
                    markdown(etiquette)
                ));
            }
        }
        // Un plateau : ce qu'il contient se place où l'on veut, par x et y, de 0 à 100 (ADR-026).
        "Board" => {
            let mut hauteur = 320.0;
            for argument in &bloc.arguments {
                match (argument.nom.as_deref(), &argument.valeur) {
                    (Some("name" | "children"), _) => {}
                    (Some("height"), Valeur::Nombre { valeur, unite: Some(unite) }) if unite == "px" && (80.0..=800.0).contains(valeur) => hauteur = *valeur,
                    (Some("height"), _) => return Err(Erreur { message: "« Board(height: …) » attend une taille entre 80px et 800px".into(), pos: argument.pos }),
                    (Some(autre), _) => return Err(Erreur { message: format!("« Board » n'a pas de paramètre « {autre} » ; paramètres possibles : name, children, height"), pos: argument.pos }),
                    (None, _) => return Err(Erreur { message: "« Board » range des blocs : Board(children: [ … ])".into(), pos: argument.pos }),
                }
            }
            sortie.push_str(&format!("<div class=\"{classes}\"{nom} style=\"height:{hauteur}px\">"));
            if let Some(Valeur::Liste(elements)) = bloc.argument("children").map(|a| &a.valeur) {
                for element in elements {
                    match element {
                        Valeur::Bloc(pose) if pose.argument("x").is_some() || pose.argument("y").is_some() => {
                            let (attribut_x, x) = place(pose, "x")?;
                            let (attribut_y, y) = place(pose, "y")?;
                            // `drag: true` : le visiteur peut faire glisser ce bloc, et ses valeurs suivent.
                            let glisse = match (pose.argument("drag").map(|a| &a.valeur), nom_de(pose)) {
                                (None | Some(Valeur::Bool(false)), _) => String::new(),
                                (Some(Valeur::Bool(true)), Some(nom)) if !attribut_x.is_empty() || !attribut_y.is_empty() => format!(" data-drag=\"{}\"", echapper(nom)),
                                _ => {
                                    return Err(Erreur {
                                        message: "« drag: true » demande un bloc qui a un nom, et dont « x » ou « y » est une valeur de la page : Point(name: Basket, x: basket, y: 96, drag: true)".into(),
                                        pos: pose.pos,
                                    })
                                }
                            };
                            sortie.push_str(&format!("<div class=\"holo-place\"{attribut_x}{attribut_y}{glisse} style=\"--x:{x};--y:{y}\">"));
                            rendre(element, sortie, mondes, base, bloc)?;
                            sortie.push_str("</div>");
                        }
                        autre => rendre(autre, sortie, mondes, base, bloc)?,
                    }
                }
            }
            sortie.push_str("</div>");
        }
        // Un son, qu'une règle fait entendre : Ding.play. Il ne se voit pas.
        "Sound" => {
            let mut source = None;
            for argument in &bloc.arguments {
                match (argument.nom.as_deref(), &argument.valeur) {
                    (Some("name" | "weight"), _) => {}
                    (Some("source"), Valeur::Texte(s)) if chemin_sur(s) && [".wav", ".mp3", ".ogg"].iter().any(|fin| s.ends_with(fin)) => source = Some(s),
                    (Some("source"), _) => {
                        return Err(Erreur { message: "« Sound(source: …) » attend un fichier de son rangé à côté du .holo : \"ding.wav\" (.wav, .mp3 ou .ogg)".into(), pos: argument.pos })
                    }
                    (Some(autre), _) => return Err(Erreur { message: format!("« Sound » n'a pas de paramètre « {autre} » ; paramètres possibles : name, source, weight"), pos: argument.pos }),
                    (None, _) => return Err(Erreur { message: "chaque paramètre de « Sound » est nommé : Sound(name: Ding, source: \"ding.wav\")".into(), pos: argument.pos }),
                }
            }
            let (Some(source), false) = (source, nom.is_empty()) else {
                return Err(Erreur { message: "un son a un nom, pour qu'une règle puisse le jouer, et un fichier : Sound(name: Ding, source: \"ding.wav\")".into(), pos: bloc.pos });
            };
            sortie.push_str(&format!("<audio class=\"{classes}\"{nom} preload=\"auto\" src=\"{}{}\"></audio>", echapper(base), echapper(source)));
        }
        // Un trait de séparation.
        "Hr" => {
            if let Some(argument) = bloc.arguments.iter().find(|a| a.nom.as_deref() != Some("name")) {
                return Err(Erreur { message: "« Hr » est un trait de séparation : il s'écrit « Hr() »".into(), pos: argument.pos });
            }
            sortie.push_str(&format!("<hr class=\"{classes}\"{nom}>"));
        }
        // Une citation, avec son auteur si on le donne : Quote("…", by: "…").
        "Quote" => {
            let auteur = match bloc.argument("by").map(|a| &a.valeur) {
                None => String::new(),
                Some(Valeur::Texte(auteur)) => format!("<footer>— {}</footer>", markdown(auteur)),
                Some(_) => return Err(Erreur { message: "« Quote(by: …) » attend un texte entre guillemets : qui l'a dit".into(), pos: bloc.pos }),
            };
            sortie.push_str(&format!("<blockquote class=\"{classes}\"{nom}><p>{}</p>{auteur}</blockquote>", markdown(texte_de(bloc)?)));
        }
        // Du texte montré tel quel, lettre pour lettre : un code, une commande, une adresse.
        "Code" => sortie.push_str(&format!("<pre class=\"{classes}\"{nom}><code>{}</code></pre>", echapper(texte_de(bloc)?))),
        // La disposition : côte à côte, l'un sous l'autre, en grille (ADR-024).
        "Row" | "Column" | "Grid" => {
            sortie.push_str(&format!("<div class=\"{classes}\"{nom} style=\"{}\">", disposition(bloc)?));
            enfants(bloc, sortie, mondes, base)?;
            sortie.push_str("</div>");
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
                Some(Valeur::Texte(adresse)) if adresse_de_passage(adresse) && adresse.ends_with(".holo") => format!(" data-file=\"{}\"", echapper(adresse)),
                Some(Valeur::Texte(_)) => {
                    return Err(Erreur {
                        message: "« inside » attend un monde, un fichier .holo rangé à côté (\"garden.holo\"), ou l'adresse complète d'un fichier .holo en https (\"https://…/garden.holo\")".into(),
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

/// La place d'un bloc sur un plateau, le long d'un axe : un nombre de 0 à 100 écrit dans le
/// fichier, ou le nom d'une valeur de la page, que le bloc suit alors quand elle change.
/// Rend l'attribut qui dit à la page quelle valeur suivre, et la place de départ.
fn place(bloc: &Bloc, axe: &str) -> Result<(String, String), Erreur> {
    match bloc.argument(axe).map(|a| &a.valeur) {
        Some(Valeur::Entier(nombre)) if *nombre <= 100 => Ok((String::new(), nombre.to_string())),
        Some(Valeur::Nom(valeur)) => Ok((format!(" data-{axe}=\"{}\"", echapper(valeur)), format!("{MARQUE}@{valeur}{MARQUE}"))),
        _ => Err(Erreur {
            message: format!("sur un plateau, « {} » se place par x et y : un nombre de 0 à 100, ou le nom d'une valeur de la page (il manque « {axe} », ou il est mal écrit)", bloc.nom),
            pos: bloc.pos,
        }),
    }
}

/// Les réglages d'un bloc de disposition : l'écart entre ses éléments, leur placement, et le
/// nombre de colonnes d'une grille. Tout est borné, et un réglage inconnu est refusé.
fn disposition(bloc: &Bloc) -> Result<String, Erreur> {
    let connus: &[&str] = if bloc.nom == "Grid" { &["name", "children", "gap", "columns"] } else { &["name", "children", "gap", "align"] };
    let mut style = String::new();
    for argument in &bloc.arguments {
        let erreur = |attendu: &str| Erreur { message: format!("« {}({}: …) » attend {attendu}", bloc.nom, argument.nom.as_deref().unwrap_or("")), pos: argument.pos };
        match (argument.nom.as_deref(), &argument.valeur) {
            (Some("name" | "children"), _) => {}
            (Some("gap"), valeur) => match valeur {
                Valeur::Nombre { valeur, unite: Some(unite) } if unite == "px" && (0.0..=64.0).contains(valeur) => style.push_str(&format!("--holo-gap:{valeur}px;")),
                _ => return Err(erreur("une taille entre 0px et 64px")),
            },
            (Some("columns"), valeur) if bloc.nom == "Grid" => match valeur {
                Valeur::Entier(colonnes) if (1..=12).contains(colonnes) => style.push_str(&format!("--holo-columns:{colonnes};")),
                _ => return Err(erreur("un nombre entier entre 1 et 12")),
            },
            (Some("align"), valeur) if bloc.nom != "Grid" => {
                let place = match (bloc.nom.as_str(), valeur) {
                    (_, Valeur::Nom(mot)) if mot == "start" => "flex-start",
                    (_, Valeur::Nom(mot)) if mot == "center" => "center",
                    (_, Valeur::Nom(mot)) if mot == "end" => "flex-end",
                    ("Row", Valeur::Nom(mot)) if mot == "between" => "space-between",
                    ("Row", _) => return Err(erreur("l'un de ces mots : start, center, end, between")),
                    _ => return Err(erreur("l'un de ces mots : start, center, end")),
                };
                style.push_str(&format!("--holo-align:{place};"));
            }
            (Some(nom), _) => {
                return Err(Erreur { message: format!("« {} » n'a pas de paramètre « {nom} » ; paramètres possibles : {}", bloc.nom, connus.join(", ")), pos: argument.pos })
            }
            (None, _) => return Err(Erreur { message: format!("« {} » range des blocs : {}(children: [ … ])", bloc.nom, bloc.nom), pos: argument.pos }),
        }
    }
    Ok(style)
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
    // Un vrai bouton : on l'atteint au clavier, et un lecteur d'écran dit son nom.
    sortie.push_str(&format!(
        "<button type=\"button\" class=\"holo-pixel\" data-name=\"{nom}\" aria-label=\"{nom}\" data-above=\"{}\" style=\"{}\"></button>",
        echapper(repere),
        allure_du_point(point)?,
        nom = echapper(nom)
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

/// L'adresse d'un passage vers un autre serveur : en https. Le http n'est accepté que vers
/// sa propre machine (`localhost`, `127.0.0.1`), pour les essais : sinon, une page publique
/// pourrait faire partir des requêtes vers le réseau privé du visiteur (revue Codex).
fn adresse_de_passage(adresse: &str) -> bool {
    let chez_soi = ["http://localhost", "http://127.0.0.1"].iter().any(|debut| {
        adresse.strip_prefix(debut).is_some_and(|suite| suite.starts_with(':') || suite.starts_with('/'))
    });
    adresse_web(adresse) && (adresse.starts_with("https://") || chez_soi)
}

/// Une image se range à côté du fichier : ni adresse complète, ni remontée de dossier.
fn chemin_sur(source: &str) -> bool {
    !source.is_empty()
        && !source.starts_with('/')
        && !source.contains("..")
        && source.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
}

fn echapper(texte: &str) -> String {
    texte.replace(MARQUE, "").replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
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
    // `code` entre accents graves, puis le gras et l'italique. Dans un texte écrit sur
    // plusieurs lignes (entre trois guillemets), chaque retour à la ligne est gardé.
    let mut html = alterner(&alterner(&alterner(&echapper(texte), "`", "code"), "**", "strong"), "*", "em").replace('\n', "<br>");
    // `{cart}` : l'endroit où s'affiche une valeur de la page. `site_html` y écrit son départ,
    // la page d'entrée la tient à jour.
    for nom in crate::etat::noms_dans(texte) {
        html = html.replace(&format!("{{{nom}}}"), &format!("<span data-state=\"{nom}\"></span>"));
    }
    html
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
            "<img class=\"holo-Image\" src=\"/ex/painting.svg\" alt=\"A painting: a yellow sun over green hills\">",
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
    fn le_texte_qui_manquait_trait_citation_code_retour_a_la_ligne_et_alt() {
        let html = page(
            "Page(children: [ Hr(), Quote(\"A **fine** shop.\", by: \"A visitor\"), Code(\"<b> WELCOME10 & co\"), P(\"\"\"\n  First line\n  Second `line`\n\"\"\"), Image(source: \"a.png\", alt: \"A red \\\"door\\\"\") ])"
                .replace("\\\"", "'")
                .as_str(),
        )
        .unwrap();
        assert!(html.contains("<hr class=\"holo-Hr\">"), "{html}");
        assert!(html.contains("<blockquote class=\"holo-Quote\"><p>A <strong>fine</strong> shop.</p><footer>— A visitor</footer></blockquote>"), "{html}");
        // Dans Code, rien n'est interprété : ni balise, ni gras.
        assert!(html.contains("<pre class=\"holo-Code\"><code>&lt;b&gt; WELCOME10 &amp; co</code></pre>"), "{html}");
        assert!(html.contains("<p class=\"holo-P\">First line<br>Second <code>line</code></p>"), "{html}");
        assert!(html.contains("<img class=\"holo-Image\" src=\"a.png\" alt=\"A red 'door'\">"), "{html}");
        // Sans alt, l'image est un décor.
        assert!(page("Page(children: [ Image(source: \"a.png\") ])").unwrap().contains("alt=\"\""));
        assert!(page("Page(children: [ Image(source: \"a.png\", alt: 3) ])").unwrap_err().message.contains("attend un texte"));
        assert!(page("Page(children: [ Hr(color: red) ])").unwrap_err().message.contains("s'écrit « Hr() »"));
        assert!(page("Page(children: [ Quote(\"x\", by: 3) ])").unwrap_err().message.contains("qui l'a dit"));
        // Le caractère qui sert de marque aux conditions ne passe pas par un texte.
        assert!(!page("Page(children: [ \"a\u{1}b\" ])").unwrap().contains('\u{1}'));
    }

    #[test]
    fn un_plateau_place_ses_blocs_ou_l_on_veut() {
        let html = page(
            "Page(state: State(sx: 70, sy: 250), children: [ Board(height: 200px, children: [ Point(name: Star, seed: 7, x: sx, y: sy), Button(name: B, text: \"b\", x: 10, y: 90), P(\"libre\") ]) ])",
        )
        .unwrap();
        // L'étoile suit deux valeurs ; au départ elle est à leur place, sans dépasser le plateau.
        assert!(html.contains("<div class=\"holo-Board\" style=\"height:200px\"><div class=\"holo-place\" data-x=\"sx\" data-y=\"sy\" style=\"--x:70;--y:100\"><button type=\"button\" class=\"holo-Point\" data-name=\"Star\""), "{html}");
        // Un bloc posé à une place fixe, et un bloc sans place.
        assert!(html.contains("<div class=\"holo-place\" style=\"--x:10;--y:90\"><button type=\"button\" class=\"holo-Button\" data-name=\"B\">b</button></div><p class=\"holo-P\">libre</p></div>"), "{html}");
        for (source, message) in [
            ("Page(children: [ Board(height: 5000px, children: []) ])", "entre 80px et 800px"),
            ("Page(children: [ Board(width: 10px, children: []) ])", "n'a pas de paramètre « width »"),
            ("Page(children: [ Board(children: [ P(\"a\", x: 10) ]) ])", "il manque « y »"),
            ("Page(children: [ Board(children: [ P(\"a\", x: 10, y: 500) ]) ])", "un nombre de 0 à 100"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn la_disposition_range_cote_a_cote_en_colonne_et_en_grille() {
        let html = page(
            "Page(children: [ Row(gap: 8px, align: between, children: [ H1(\"Shop\"), Button(name: Menu, text: \"Menu\") ]), Grid(columns: 3, children: [ \"a\", Column(align: center, children: [ P(\"b\"), P(\"c\") ]) ]) ])",
        )
        .unwrap();
        assert!(html.contains("<div class=\"holo-Row\" style=\"--holo-gap:8px;--holo-align:space-between;\"><h1 class=\"holo-H1\">Shop</h1><button"), "{html}");
        assert!(html.contains("<div class=\"holo-Grid\" style=\"--holo-columns:3;\"><p class=\"holo-P\">a</p><div class=\"holo-Column\" style=\"--holo-align:center;\">"), "{html}");
        // Un bouton rangé dans une ligne reste un bouton : sa règle le trouve.
        assert!(html.contains("data-name=\"Menu\""));
        for (source, message) in [
            ("Page(children: [ Row(gap: 8, children: []) ])", "entre 0px et 64px"),
            ("Page(children: [ Row(gap: 500px, children: []) ])", "entre 0px et 64px"),
            ("Page(children: [ Row(align: middle, children: []) ])", "start, center, end, between"),
            ("Page(children: [ Column(align: between, children: []) ])", "start, center, end"),
            ("Page(children: [ Grid(columns: 40, children: []) ])", "entre 1 et 12"),
            ("Page(children: [ Row(columns: 2, children: []) ])", "n'a pas de paramètre « columns »"),
            ("Page(children: [ Grid(align: center, children: []) ])", "n'a pas de paramètre « align »"),
            ("Page(children: [ Row(wrap: false, children: []) ])", "n'a pas de paramètre « wrap »"),
            ("Page(children: [ Row(\"a\", \"b\") ])", "range des blocs"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn un_site_se_plante_dans_un_pixel_de_la_page() {
        let html = page(
            "Page(children: [ Button(name: Open, text: \"x\") ], pixels: [ Point(name: Secret, above: Open, seed: 7, color: \"#FF4D6D\", inside: World(children: [ H1(\"Hidden\") ])) ])",
        )
        .unwrap();
        assert!(html.contains("<button type=\"button\" class=\"holo-pixel\" data-name=\"Secret\" aria-label=\"Secret\" data-above=\"Open\" style=\"--holo-color:#FF4D6D;\"></button>"), "{html}");
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
        assert!(html.contains("class=\"holo-pixel\" data-name=\"B\" aria-label=\"B\" data-above=\"Out\""));
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
        // En http, seulement vers sa propre machine ; jamais vers le réseau privé de quelqu'un.
        assert!(page("Page(children: [ Point(name: G, seed: 1, inside: \"http://127.0.0.1:8081/x.holo\") ])").is_ok());
        assert!(page("Page(children: [ Point(name: G, seed: 1, inside: \"http://localhost/x.holo\") ])").is_ok());
        for refuse in ["http://friend.example/x.holo", "http://192.168.1.1/x.holo", "http://localhost.evil.example/x.holo", "http://127.0.0.1.evil.example/x.holo"] {
            assert!(page(&format!("Page(children: [ Point(name: G, seed: 1, inside: \"{refuse}\") ])")).is_err(), "{refuse}");
        }
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
