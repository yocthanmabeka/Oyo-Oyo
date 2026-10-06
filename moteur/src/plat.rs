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
:where(.holo-Page>main,.holo-Page>header,.holo-Page>footer){display:block;max-width:640px;margin:0 auto;position:relative}\
:where(.holo-Page>main,.holo-Page>header,.holo-Page>footer,.holo-panneau,.holo-Header,.holo-Footer,.holo-Main)>*{display:block;box-sizing:border-box;margin:0 0 16px 0}\
:where(.holo-Nav)>*{margin:0}\
:where(.holo-Stack){display:inline-grid;position:relative;max-width:100%;vertical-align:top}:where(.holo-Stack>:first-child .holo-Image){width:100%;display:block}:where(.holo-Stack)>*{grid-area:1/1;min-width:0;margin:0}\
:where(.holo-pose){z-index:1;margin:6px}\
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
:where(.holo-Input textarea){font:inherit;color:inherit;background:transparent;border:1px solid currentColor;border-radius:6px;padding:6px 10px;width:min(100%,480px);box-sizing:border-box;resize:vertical}\
:where(.holo-Choice){border:0;padding:0;margin:0 0 16px 0;display:flex;flex-wrap:wrap;gap:8px 18px;align-items:center}\
:where(.holo-Choice legend){padding:0;margin:0 0 6px 0;width:100%}:where(.holo-Choice label){display:flex;gap:6px;align-items:center;cursor:pointer}\
:where(.holo-Choice input){accent-color:currentColor;width:18px;height:18px;margin:0}\
:where(label.holo-Choice){flex-direction:column;align-items:flex-start;gap:4px}:where(.holo-Choice select){font:inherit;color:inherit;background:transparent;border:1px solid currentColor;border-radius:6px;padding:6px 10px}\
:where(.holo-Video){display:block;width:100%;max-width:640px;border-radius:12px;background:black}\
:where(.holo-tableau){overflow-x:auto;max-width:100%}:where(.holo-Table){border-collapse:collapse;min-width:100%}\
:where(.holo-Table caption){text-align:left;font-weight:bold;padding:0 0 8px 0}:where(.holo-Table th,.holo-Table td){text-align:left;padding:8px 12px;border-bottom:1px solid color-mix(in srgb,currentColor 25%,transparent)}\
:where(.holo-Table th){font-weight:bold}\
:where(.holo-Board){position:relative;overflow:hidden;border-radius:12px;container-type:inline-size;margin-inline:auto}\
:where(.holo-place){position:absolute;left:calc(var(--x)*1%);top:calc(var(--y)*1%);transform:translate(calc(var(--x)*-1%),calc(var(--y)*-1%));\
transition:left .12s linear,top .12s linear,transform .12s linear}\
:where(.holo-place[data-drag]){touch-action:none;cursor:grab}.holo-place.holo-glisse{transition:none;cursor:grabbing}\
@media (prefers-reduced-motion:reduce){.holo-place{transition:none}.holo-Page,.holo-Page *{transition:none!important}}\
:where(.holo-Sound){display:none}:where(audio.holo-Sound[controls]){display:block;width:100%;max-width:480px}\
:where(.holo-figure){margin:0 0 16px 0}:where(.holo-figure figcaption){font-size:0.9em;opacity:0.8;margin-top:6px}:where(picture){display:contents}\
:where(.holo-Slider,.holo-Progress){display:flex;flex-direction:column;gap:4px;align-items:flex-start}:where(.holo-Slider input){width:min(100%,320px);accent-color:currentColor}\
:where(.holo-Progress progress){width:min(100%,320px);accent-color:currentColor}\
:where(.holo-Details summary){cursor:pointer;font-weight:bold}:where(.holo-Details[open] summary){margin-bottom:8px}\
:where(.holo-Dialog){max-width:min(90vw,480px);border:1px solid currentColor;border-radius:12px;padding:16px 20px;color:inherit;background:var(--fond,Canvas)}\
:where(.holo-Dialog)::backdrop{background:rgba(0,0,0,0.5)}:where(.holo-Dialog>*){margin:0 0 12px 0}:where(.holo-fermer){display:flex;justify-content:flex-end;margin:0}\
:where(.holo-fermer button){font:inherit;color:inherit;background:transparent;border:0;cursor:pointer;font-size:1.2em;line-height:1}\
:where(.holo-Liste,.holo-ligne){display:contents}:where(.holo-Form){display:block}:where(.holo-Form>*){display:block;box-sizing:border-box;margin:0 0 16px 0}\
:where(.holo-Shape){display:block;width:var(--holo-size,48px);height:var(--holo-size,48px);padding:0;border:0;background:var(--holo-color,currentColor)}\
:where(button.holo-Shape){cursor:pointer}\
:where(.holo-forme-circle){border-radius:50%}\
:where(.holo-Board .holo-Shape){width:calc(var(--holo-n,48)*100cqw/640);height:calc(var(--holo-n,48)*100cqw/640)}\
:where(.holo-Board .holo-Point){width:10cqw;height:10cqw}\
:where(.holo-forme-triangle){clip-path:polygon(50% 0,100% 100%,0 100%)}\
:where(.holo-forme-diamond){clip-path:polygon(50% 0,100% 50%,50% 100%,0 50%)}\
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
    // Les mouvements de la page deviennent du CSS, ajouté à son style (ADR-034).
    crate::mouvement::commencer();
    let html = site_html_brut(programme, page, base, titre);
    let mouvements = crate::mouvement::terminer();
    let html = html?;
    if mouvements.is_empty() && !html.contains("holo-Scene") {
        return Ok(html);
    }
    Ok(html.replacen("</style>", &format!("{}{mouvements}</style>", crate::mouvement::BASE), 1))
}

fn site_html_brut(programme: &Programme, page: &Bloc, base: &str, titre: &str) -> Result<String, Erreur> {
    // Les listes, à leur départ : les lignes de `Repeat(over:)` sont fabriquées d'après elles.
    crate::listes::mettre_en_cours(crate::listes::initiales(programme));
    if page.nom != "Page" && page.nom != "World" {
        return Err(Erreur { message: format!("la vue à plat affiche une « Page » ; ce fichier commence par « {} »", page.nom), pos: page.pos });
    }
    let mut corps = String::new();
    let mut mondes = String::new();
    // Les repères (ADR-036) : un `Header` et un `Footer` posés directement dans la page en
    // sont l'en-tête et le pied, hors du contenu principal ; un `Main` dit où est ce contenu.
    let mut entete = String::new();
    let mut pied = String::new();
    if let Some(Valeur::Liste(elements)) = page.argument("children").map(|a| &a.valeur) {
        for element in elements {
            match element {
                Valeur::Bloc(b) if b.nom == "Header" => rendre(element, &mut entete, &mut mondes, base, page)?,
                Valeur::Bloc(b) if b.nom == "Footer" => rendre(element, &mut pied, &mut mondes, base, page)?,
                Valeur::Bloc(b) if b.nom == "Main" => {
                    reperes_permis(b)?;
                    enfants(b, &mut corps, &mut mondes, base)?;
                }
                _ => rendre(element, &mut corps, &mut mondes, base, page)?,
            }
        }
    }
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
    let classes = match page.nom.as_str() {
        "World" => format!("holo-Page holo-monde-ouvert{}", noms_de_style(page)),
        _ => classes(page),
    };
    // Les valeurs de la page, à leur départ, là où un texte les montre : « {cart} » (ADR-023).
    let depart = crate::etat::initial(programme).unwrap_or_default();
    let mut montrees = crate::etat::a_montrer(programme, &depart);
    // Une liste montre son nombre d'éléments, et une condition le compare (ADR-044).
    montrees.extend(crate::listes::comptes(&crate::listes::en_cours()));
    // Les conditions, à leur départ : ce qui est faux est caché dès le premier affichage (ADR-025).
    // La réponse vient de `etat::conditions`, comme après chaque changement : une condition
    // n'est décidée qu'à un seul endroit.
    let textes = crate::etat::textes_initiaux(programme);
    let reponses = crate::etat::conditions(programme, &crate::etat::avec_textes(&montrees, &textes));
    let conditions = |html: String| remplir_marques(html, &montrees, &textes, &reponses);
    corps = conditions(corps);
    mondes = conditions(mondes);
    entete = conditions(entete);
    pied = conditions(pied);
    // Les textes, à leur départ, là où un texte les montre.
    for (nom, texte) in &textes {
        let (vide, pleine) = (format!("<span data-state=\"{nom}\"></span>"), format!("<span data-state=\"{nom}\">{}</span>", echapper(texte)));
        corps = corps.replace(&vide, &pleine);
        mondes = mondes.replace(&vide, &pleine);
        entete = entete.replace(&vide, &pleine);
        pied = pied.replace(&vide, &pleine);
    }
    // Les valeurs à format, à leur départ, dans la langue de la page.
    let langue = match programme.racine.argument("lang").map(|a| &a.valeur) {
        Some(Valeur::Texte(l)) => l.as_str(),
        _ => "fr",
    };
    corps = crate::format::remplir(&corps, &montrees, langue);
    mondes = crate::format::remplir(&mondes, &montrees, langue);
    entete = crate::format::remplir(&entete, &montrees, langue);
    pied = crate::format::remplir(&pied, &montrees, langue);
    for (nom, valeur) in montrees.clone() {
        let (vide, pleine) = (format!("<span data-state=\"{nom}\"></span>"), format!("<span data-state=\"{nom}\">{valeur}</span>"));
        corps = corps.replace(&vide, &pleine);
        mondes = mondes.replace(&vide, &pleine);
        entete = entete.replace(&vide, &pleine);
        pied = pied.replace(&vide, &pleine);
    }
    // Une page vivante bouge ou écoute sans qu'on la touche : une horloge, le clavier, des
    // données à recevoir, un bloc à faire glisser. (Des valeurs gardées, `keep`, ne la rendent
    // pas vivante : la page légère regarde s'il y a vraiment quelque chose de gardé.) Le moteur doit
    // alors arriver tout de suite. Les autres pages s'affichent seules : le moteur n'est
    // téléchargé qu'au premier geste qui en a besoin (ADR-033).
    let vivante = !crate::etat::horloges(programme).is_empty()
        || !crate::etat::delais(programme, &depart).is_empty()
        || crate::etat::lit_l_heure(programme)
        || !crate::etat::touches(programme).is_empty()
        || crate::etat::source_de_donnees(programme).ok().flatten().is_some()
        || corps.contains("data-drag=");
    let vivante = if vivante { " data-vivant" } else { "" };
    // Un bloc qu'une règle écoute au survol le dit à la page (ADR-039) : la page légère fait
    // venir le moteur quand la souris arrive dessus.
    // Un bloc vers lequel un lien de la page mène (ADR-042) reçoit son nom comme `id` : le
    // navigateur y descend tout seul, sans le moteur.
    for nom in crate::regles::ancres(programme) {
        let seul = format!(" data-name=\"{}\"", echapper(&nom));
        for html in [&mut corps, &mut entete, &mut pied, &mut mondes] {
            if let Some(place) = html.find(&seul) {
                html.insert_str(place + seul.len(), &format!(" id=\"{}\"", echapper(&nom)));
                break;
            }
        }
    }
    // Un bloc qui ne reçoit pas le focus de lui-même (une carte, un texte) le reçoit alors, pour
    // qu'on le survole aussi au clavier, avec Tab.
    for nom in crate::regles::survoles(programme) {
        let seul = format!(" data-name=\"{}\"", echapper(&nom));
        for html in [&mut corps, &mut entete, &mut pied, &mut mondes] {
            let Some(place) = html.find(&seul) else { continue };
            let balise: String = html[..place].rsplit('<').next().unwrap_or("").chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
            let focus = if ["button", "a", "label", "fieldset", "video"].contains(&balise.as_str()) { "" } else { " tabindex=\"0\"" };
            html.replace_range(place..place + seul.len(), &format!("{seul} data-hover{focus}"));
        }
    }
    // La langue, la description et l'image de partage (ADR-038) : le serveur et le moteur les
    // reprennent dans l'en-tête de la page, pour les lecteurs d'écran, Google et les réseaux.
    let mut partage = String::new();
    for (parametre, attribut) in [("lang", "data-lang"), ("description", "data-description"), ("image", "data-image"), ("icon", "data-icon")] {
        match page.argument(parametre).map(|a| &a.valeur) {
            None => {}
            Some(Valeur::Texte(texte)) if parametre == "lang" && est_langue(texte) => partage.push_str(&format!(" {attribut}=\"{}\"", echapper(texte))),
            Some(Valeur::Texte(texte)) if parametre == "description" && texte.chars().count() <= 300 => partage.push_str(&format!(" {attribut}=\"{}\"", echapper(texte))),
            Some(Valeur::Texte(texte)) if parametre == "image" && chemin_sur(texte) => partage.push_str(&format!(" {attribut}=\"{}{}\"", echapper(base), echapper(texte))),
            // La petite image de l'onglet (ADR-042).
            Some(Valeur::Texte(texte)) if parametre == "icon" && chemin_sur(texte) && [".png", ".svg", ".ico"].iter().any(|fin| texte.ends_with(fin)) => {
                partage.push_str(&format!(" {attribut}=\"{}{}\"", echapper(base), echapper(texte)))
            }
            Some(_) => {
                let pos = page.argument(parametre).map_or(page.pos, |a| a.pos);
                return Err(Erreur {
                    message: match parametre {
                        "lang" => "« Page(lang: …) » attend une langue, comme \"fr\", \"en\" ou \"fr-CA\"".into(),
                        "description" => "« Page(description: …) » attend un texte de 300 caractères au plus : ce que Google montre sous le titre".into(),
                        "icon" => "« Page(icon: …) » attend une petite image rangée à côté du fichier, en .png, .svg ou .ico : celle de l'onglet".into(),
                        _ => "« Page(image: …) » attend une image rangée à côté du fichier, comme \"partage.png\" : celle qu'on voit quand on partage le lien".into(),
                    },
                    pos,
                });
            }
        }
    }
    Ok(format!(
        "<style>{}{BASE}{}</style><div class=\"{classes}\" data-title=\"{titre}\"{vivante}{partage}>{entete}<main>{corps}</main>{pied}{mondes}</div>",
        polices(&programme.racine, base)?,
        css(programme, base)
    ))
}

/// Remplit les marques laissées dans le HTML en cours de fabrication : la condition d'un `If`,
/// le genre et la valeur d'un champ, une case cochée, une place sur un plateau.
fn remplir_marques(html: String, montrees: &crate::etat::Etat, textes: &crate::etat::Textes, reponses: &[(String, bool)]) -> String {
    let texte_de_depart = |nom: &str| textes.iter().find(|(connu, _)| connu == nom).map(|(_, texte)| texte.as_str());
    {
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
            } else if let Some(choix) = morceau.strip_prefix('=') {
                // Un choix : l'option cochée (ou choisie) au départ est celle de la valeur.
                let mut parts = choix.splitn(3, '|');
                let (attribut, nom, option) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""), parts.next().unwrap_or(""));
                if texte_de_depart(nom).is_some_and(|t| echapper(t) == option && !t.is_empty()) {
                    sortie.push_str(&format!(" {attribut}"));
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
            } else if let Some(cle) = morceau.strip_prefix('^') {
                // Le « sinon » d'une condition : caché quand elle est vraie (ADR-039).
                if reponses.iter().any(|(connue, vraie)| connue == cle && *vraie) {
                    sortie.push_str(" hidden");
                }
            } else if !reponses.iter().any(|(cle, vraie)| cle == morceau && *vraie) {
                sortie.push_str(" hidden");
            }
        }
        sortie
    }
}

/// Le CSS des styles du fichier. Le thème d'abord, puis les types, puis les styles nommés :
/// à précision égale, le dernier écrit l'emporte, ce qui donne la priorité voulue (ADR-017).
/// Les polices de la page (ADR-041) : `fonts: [ Font(family: "Carlito", source: "carlito.woff2") ]`.
/// Le texte s'affiche tout de suite avec la police de secours, puis prend la sienne quand elle
/// arrive (`font-display: swap`) : jamais de texte invisible en attendant.
fn polices(page: &Bloc, base: &str) -> Result<String, Erreur> {
    let Some(argument) = page.argument("fonts") else { return Ok(String::new()) };
    let exemple = "fonts: [ Font(family: \"Carlito\", source: \"carlito.woff2\") ]";
    let Valeur::Liste(liste) = &argument.valeur else {
        return Err(Erreur { message: format!("« fonts » est une liste de polices : {exemple}"), pos: argument.pos });
    };
    if liste.len() > 8 {
        return Err(Erreur { message: "une page charge au plus 8 polices".into(), pos: argument.pos });
    }
    let mut css = String::new();
    let mut familles: Vec<&str> = Vec::new();
    for element in liste {
        let Valeur::Bloc(police) = element else {
            return Err(Erreur { message: format!("« fonts » contient des « Font(…) » : {exemple}"), pos: argument.pos });
        };
        if police.nom != "Font" {
            return Err(Erreur { message: format!("« fonts » contient des « Font(…) », pas des « {} »", police.nom), pos: police.pos });
        }
        let (mut famille, mut source) = (None, None);
        for a in &police.arguments {
            match (a.nom.as_deref(), &a.valeur) {
                (Some("family"), Valeur::Texte(t)) if !t.is_empty() && t.len() <= 40 && t.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-') => famille = Some(t.as_str()),
                (Some("family"), _) => return Err(Erreur { message: "« Font(family: …) » attend le nom de la police entre guillemets : lettres, chiffres, espaces".into(), pos: a.pos }),
                (Some("source"), Valeur::Texte(s)) if chemin_sur(s) && [".woff2", ".woff", ".ttf", ".otf"].iter().any(|fin| s.ends_with(fin)) => source = Some(s.as_str()),
                (Some("source"), _) => return Err(Erreur { message: "« Font(source: …) » attend un fichier de police rangé à côté : .woff2, .woff, .ttf ou .otf".into(), pos: a.pos }),
                (Some(autre), _) => return Err(Erreur { message: format!("« Font » n'a pas de paramètre « {autre} » ; paramètres possibles : family, source"), pos: a.pos }),
                (None, _) => return Err(Erreur { message: format!("chaque paramètre de « Font » est nommé : {exemple}"), pos: a.pos }),
            }
        }
        let (Some(famille), Some(source)) = (famille, source) else {
            return Err(Erreur { message: format!("« Font » attend « family » et « source » : {exemple}"), pos: police.pos });
        };
        if familles.contains(&famille) {
            return Err(Erreur { message: format!("la police « {famille} » est chargée deux fois"), pos: police.pos });
        }
        familles.push(famille);
        css.push_str(&format!("@font-face{{font-family:\"{famille}\";src:url(\"{}{}\");font-display:swap}}", echapper(base), echapper(source)));
    }
    Ok(css)
}

fn css(programme: &Programme, base: &str) -> String {
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
            Cible::Type(t) if programme.composants.contains(t) => format!(".holo-c-{t}"),
            Cible::Type(t) => format!(".holo-{t}"),
            Cible::Nom(n) => format!(".holo-s-{n}"),
        };
        sortie.push_str(&selecteur);
        sortie.push('{');
        for reglage in &regle.reglages {
            sortie.push_str(&format!("{}:{};", reglage.nom, valeur_css(reglage, base)));
        }
        // Un style qui change au survol ou à l'appui passe d'un aspect à l'autre en douceur,
        // à moins que l'auteur n'ait dit sa propre durée (`transition:`).
        if regle.etats.iter().any(|(etat, ..)| etat == "hover" || etat == "active") && !regle.reglages.iter().any(|r| r.nom == "transition") {
            sortie.push_str("transition:background .15s,color .15s,border-color .15s,opacity .15s,box-shadow .15s,scale .15s,rotate .15s;");
        }
        sortie.push('}');
        // Les états (ADR-036). Le survol n'existe qu'avec une souris : sur un écran tactile, il
        // resterait collé après un toucher. Le focus est celui du clavier. Le thème sombre suit
        // le choix du visiteur ; « phone » vaut pour un écran plus étroit que la page (ADR-041).
        for (etat, reglages, _) in &regle.etats {
            let corps: String = reglages.iter().map(|r| format!("{}:{};", r.nom, valeur_css(r, base))).collect();
            let regle_etat = match etat.as_str() {
                "hover" => format!("@media (hover:hover){{{}:hover{{{corps}}}}}", selecteur.split(',').map(str::to_string).collect::<Vec<_>>().join(":hover,")),
                "focus" => format!("{}:focus-visible{{{corps}}}", selecteur.split(',').collect::<Vec<_>>().join(":focus-visible,")),
                "dark" => format!("@media (prefers-color-scheme:dark){{{selecteur}{{{corps}}}}}"),
                "phone" => format!("@media (max-width:{LARGEUR_D_AUTEUR}px){{{selecteur}{{{corps}}}}}"),
                _ => format!("{}:active{{{corps}}}", selecteur.split(',').collect::<Vec<_>>().join(":active,")),
            };
            sortie.push_str(&regle_etat);
        }
    }
    sortie
}

/// La largeur de page pour laquelle un auteur écrit ses tailles (`main` fait 640px au plus).
const LARGEUR_D_AUTEUR: f64 = 640.0;

/// La valeur d'un réglage, telle que le navigateur la reçoit. Une taille de texte écrite en
/// pixels suit le réglage « texte plus grand » du visiteur (en rem : 16px = 1rem) ; un grand
/// titre rétrécit sur un écran plus étroit que la page, sans jamais passer sous 24px (ADR-036).
fn valeur_css(reglage: &crate::holo::Reglage, base: &str) -> String {
    let mut valeur = reglage.valeur.replace('<', "");
    // Une variable (ADR-041) : `--or` devient `var(--or)` ; sa définition reste telle quelle.
    for variable in crate::styles::variables_de(&reglage.valeur) {
        valeur = valeur.replacen(variable, &format!("var({variable})"), 1);
    }
    // Une image de fond couvre toujours le bloc, centrée, sans se répéter en mosaïque.
    if reglage.nom == "background" {
        if let Some(image) = crate::styles::image_de_fond(&reglage.valeur) {
            return format!("url(\"{}{}\") center/cover no-repeat", echapper(base), echapper(image));
        }
    }
    // Une durée de passage : elle vaut pour tout ce qui change d'allure.
    if reglage.nom == "transition" && valeur != "none" {
        return ["background", "color", "border-color", "opacity", "box-shadow", "scale", "rotate", "letter-spacing"].iter().map(|p| format!("{p} {valeur}")).collect::<Vec<_>>().join(",");
    }
    if reglage.nom != "font-size" {
        return valeur;
    }
    let Some(px) = valeur.strip_suffix("px").and_then(|n| n.trim().parse::<f64>().ok()) else { return valeur };
    let rem = px / 16.0;
    if px <= 24.0 {
        return format!("{}rem", arrondi(rem));
    }
    format!("clamp(1.5rem,{}vw,{}rem)", arrondi(px * 100.0 / LARGEUR_D_AUTEUR), arrondi(rem))
}

fn arrondi(n: f64) -> String {
    let n = (n * 1000.0).round() / 1000.0;
    if n.fract() == 0.0 { format!("{n:.0}") } else { format!("{n}") }
}

/// Une langue, comme « fr », « en » ou « fr-CA ».
fn est_langue(texte: &str) -> bool {
    let (langue, region) = texte.split_once('-').unwrap_or((texte, ""));
    (2..=3).contains(&langue.len()) && langue.chars().all(|c| c.is_ascii_lowercase()) && (region.is_empty() || (region.len() == 2 && region.chars().all(|c| c.is_ascii_uppercase())))
}

/// Un repère ne prend que des enfants et un nom.
fn reperes_permis(bloc: &Bloc) -> Result<(), Erreur> {
    for argument in &bloc.arguments {
        match argument.nom.as_deref() {
            Some("name" | "children" | "enter" | "loop") => {}
            Some(autre) => return Err(Erreur { message: format!("« {} » n'a pas de paramètre « {autre} » ; paramètres possibles : children, name", bloc.nom), pos: argument.pos }),
            None => return Err(Erreur { message: format!("« {0} » range des blocs : {0}(children: [ … ])", bloc.nom), pos: argument.pos }),
        }
    }
    Ok(())
}

/// Les places d'un bloc posé dans un `Stack`.
const COINS: &[&str] = &["topLeft", "top", "topRight", "left", "center", "right", "bottomLeft", "bottom", "bottomRight"];

/// `topRight` → (haut, côté) en CSS.
fn coin(mot: &str) -> Option<(&'static str, &'static str)> {
    Some(match mot {
        "topLeft" => ("start", "start"),
        "top" => ("start", "center"),
        "topRight" => ("start", "end"),
        "left" => ("center", "start"),
        "center" => ("center", "center"),
        "right" => ("center", "end"),
        "bottomLeft" => ("end", "start"),
        "bottom" => ("end", "center"),
        "bottomRight" => ("end", "end"),
        _ => return None,
    })
}

fn classes(bloc: &Bloc) -> String {
    format!("holo-{}{}", bloc.nom, noms_de_style(bloc))
}

/// Les classes des noms de style d'un bloc : ` holo-s-card` ; la marque d'un composant (un nom
/// qui commence par une majuscule, posé par le moteur) : ` holo-c-ArticleCard` (ADR-050).
fn noms_de_style(bloc: &Bloc) -> String {
    bloc.styles.iter().map(|s| if s.starts_with(|c: char| c.is_ascii_uppercase()) { format!(" holo-c-{s}") } else { format!(" holo-s-{s}") }).collect()
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
    // Un bloc qui bouge (enter:, loop:) : on le fabrique sans ses mouvements, puis on
    // l'enveloppe dans eux (ADR-034).
    if let Some((mouvements, reste)) = crate::mouvement::du_bloc(bloc)? {
        let mut dedans = String::new();
        let enfants = match reste.argument("children").map(|a| &a.valeur) {
            Some(Valeur::Liste(liste)) => liste.len(),
            _ => 0,
        };
        rendre(&Valeur::Bloc(reste), &mut dedans, mondes, base, parent)?;
        crate::mouvement::envelopper(&mouvements, dedans, enfants, sortie);
        return Ok(());
    }
    let classes = classes(bloc);
    let nom = nom_de(bloc).map(|n| format!(" data-name=\"{}\"", echapper(n))).unwrap_or_default();
    match bloc.nom.as_str() {
        // Des scènes qui s'enchaînent, l'une après l'autre, au même endroit (ADR-034).
        "Scenes" => {
            let mut hauteur = 480.0;
            let mut toujours = false;
            for argument in &bloc.arguments {
                match (argument.nom.as_deref(), &argument.valeur) {
                    (Some("name" | "children"), _) => {}
                    (Some("height"), Valeur::Nombre { valeur, unite: Some(unite) }) if unite == "px" && (80.0..=2000.0).contains(valeur) => hauteur = *valeur,
                    (Some("height"), _) => return Err(Erreur { message: "« Scenes(height: …) » attend une hauteur entre 80px et 2000px".into(), pos: argument.pos }),
                    (Some("repeat"), Valeur::Nom(mot)) if mot == "forever" => toujours = true,
                    (Some("repeat"), _) => return Err(Erreur { message: "« Scenes(repeat: …) » attend « forever » : les scènes recommencent sans fin".into(), pos: argument.pos }),
                    (Some(autre), _) => return Err(Erreur { message: format!("« Scenes » n'a pas de paramètre « {autre} » ; paramètres possibles : children, height, repeat, name"), pos: argument.pos }),
                    (None, _) => return Err(Erreur { message: "« Scenes » range des scènes : Scenes(children: [ Scene(for: 3s, children: [ … ]) ])".into(), pos: argument.pos }),
                }
            }
            let mut scenes = Vec::new();
            if let Some(Valeur::Liste(elements)) = bloc.argument("children").map(|a| &a.valeur) {
                for element in elements {
                    let scene = match element {
                        Valeur::Bloc(scene) if scene.nom == "Scene" => scene,
                        _ => return Err(Erreur { message: "« Scenes » ne range que des « Scene(for: …, children: [ … ]) »".into(), pos: bloc.pos }),
                    };
                    let mut duree = None;
                    for argument in &scene.arguments {
                        match (argument.nom.as_deref(), &argument.valeur) {
                            (Some("name" | "children"), _) => {}
                            (Some("for"), valeur) => {
                                duree = Some(crate::mouvement::duree_de_scene(valeur).ok_or_else(|| Erreur { message: "« Scene(for: …) » attend une durée de 200ms à 600s, comme for: 4s".into(), pos: argument.pos })?)
                            }
                            (Some(autre), _) => return Err(Erreur { message: format!("« Scene » n'a pas de paramètre « {autre} » ; paramètres possibles : for, children, name"), pos: argument.pos }),
                            (None, _) => return Err(Erreur { message: "« Scene » range des blocs : Scene(for: 3s, children: [ … ])".into(), pos: argument.pos }),
                        }
                    }
                    let duree = duree.ok_or_else(|| Erreur { message: "« Scene » dit combien de temps elle dure : Scene(for: 3s, children: [ … ])".into(), pos: scene.pos })?;
                    scenes.push((scene, duree));
                }
            }
            let tour = toujours.then(|| scenes.iter().map(|(_, d)| d).sum::<f64>());
            sortie.push_str(&format!("<div class=\"{classes}\"{nom} style=\"height:{hauteur}px\">"));
            let mut debut = 0.0;
            let derniere = scenes.len().saturating_sub(1);
            for (rang, (scene, duree)) in scenes.into_iter().enumerate() {
                let nom_de_scene = crate::mouvement::nouvelle_scene(debut, duree, tour, rang == derniere);
                let nom = nom_de(scene).map(|n| format!(" data-name=\"{}\"", echapper(n))).unwrap_or_default();
                sortie.push_str(&format!("<div class=\"{} {nom_de_scene}\"{nom}>", self::classes(scene)));
                crate::mouvement::dans_la_scene(debut, tour, || enfants(scene, sortie, mondes, base))?;
                sortie.push_str("</div>");
                debut += duree;
            }
            sortie.push_str("</div>");
        }
        // Les repères, pour qui navigue avec un lecteur d'écran (ADR-036).
        "Nav" | "Header" | "Footer" => {
            reperes_permis(bloc)?;
            let balise = bloc.nom.to_ascii_lowercase();
            sortie.push_str(&format!("<{balise} class=\"{classes}\"{nom}>"));
            enfants(bloc, sortie, mondes, base)?;
            sortie.push_str(&format!("</{balise}>"));
        }
        "Main" => return Err(Erreur { message: "« Main » se place directement dans la page : Page(children: [ Header(…), Main(children: [ … ]), Footer(…) ])".into(), pos: bloc.pos }),
        // La superposition (ADR-036) : le premier enfant donne la taille ; les autres se posent
        // dessus, chacun à sa place (align:), comme un badge sur une image.
        "Stack" => {
            for argument in &bloc.arguments {
                match argument.nom.as_deref() {
                    Some("name" | "children") => {}
                    Some(autre) => return Err(Erreur { message: format!("« Stack » n'a pas de paramètre « {autre} » ; paramètres possibles : children, name"), pos: argument.pos }),
                    None => return Err(Erreur { message: "« Stack » superpose des blocs : Stack(children: [ Image(…), Text(\"Promo\", align: topRight) ])".into(), pos: argument.pos }),
                }
            }
            sortie.push_str(&format!("<div class=\"{classes}\"{nom}>"));
            if let Some(Valeur::Liste(elements)) = bloc.argument("children").map(|a| &a.valeur) {
                for (rang, element) in elements.iter().enumerate() {
                    let (pose, style) = match element {
                        Valeur::Bloc(enfant) => match enfant.argument("align") {
                            Some(argument) if enfant.nom != "Row" && enfant.nom != "Column" => {
                                let (haut, cote) = match &argument.valeur {
                                    Valeur::Nom(mot) if mot.contains('_') && coin(&crate::etat::en_flutter(mot)).is_some() => {
                                        return Err(Erreur { message: format!("« {mot} » : deux mots se joignent comme en Flutter ; écris « {} » (ADR-037)", crate::etat::en_flutter(mot)), pos: argument.pos })
                                    }
                                    Valeur::Nom(mot) => coin(mot).ok_or_else(|| Erreur { message: format!("« align: » dans « Stack » attend l'une de ces places : {}", COINS.join(", ")), pos: argument.pos })?,
                                    _ => return Err(Erreur { message: format!("« align: » dans « Stack » attend l'une de ces places : {}", COINS.join(", ")), pos: argument.pos }),
                                };
                                let mut reste = enfant.clone();
                                reste.arguments.retain(|a| a.nom.as_deref() != Some("align"));
                                (Valeur::Bloc(reste), format!(" class=\"holo-pose\" style=\"align-self:{haut};justify-self:{cote}\""))
                            }
                            _ => (element.clone(), if rang == 0 { String::new() } else { " class=\"holo-pose\" style=\"align-self:center;justify-self:center\"".to_string() }),
                        },
                        _ => (element.clone(), String::new()),
                    };
                    sortie.push_str(&format!("<div{style}>"));
                    rendre(&pose, sortie, mondes, base, bloc)?;
                    sortie.push_str("</div>");
                }
            }
            sortie.push_str("</div>");
        }
        "Scene" => return Err(Erreur { message: "« Scene » se range dans des scènes : Scenes(children: [ Scene(for: 3s, children: [ … ]) ])".into(), pos: bloc.pos }),
        "H1" | "H2" | "H3" | "H4" | "H5" | "H6" | "P" | "Text" => {
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
            // `alt` : le texte qui remplace l'image pour qui ne la voit pas. Il est obligatoire
            // (ADR-038) : pour un simple décor, on l'écrit vide, alt: "", et un lecteur d'écran
            // la passe. On ne l'oublie plus sans le savoir.
            let alt = match bloc.argument("alt").map(|a| &a.valeur) {
                None => return Err(Erreur { message: "« Image » attend « alt » : ce que montre l'image, pour qui ne la voit pas ; pour un simple décor, alt: \"\"".into(), pos: bloc.pos }),
                Some(Valeur::Texte(texte)) => texte.as_str(),
                Some(_) => return Err(Erreur { message: "« Image(alt: …) » attend un texte entre guillemets : ce que montre l'image".into(), pos: bloc.pos }),
            };
            let mut image = format!("<img class=\"{classes}\"{nom} src=\"{}{}\" alt=\"{}\">", echapper(base), echapper(source), echapper(alt));
            // Une image plus légère pour un téléphone (ADR-042) : le navigateur ne télécharge que
            // celle qu'il montre.
            match bloc.argument("phone").map(|a| &a.valeur) {
                None => {}
                Some(Valeur::Texte(petite)) if chemin_sur(petite) => {
                    image = format!("<picture><source media=\"(max-width:{LARGEUR_D_AUTEUR}px)\" srcset=\"{}{}\">{image}</picture>", echapper(base), echapper(petite))
                }
                Some(_) => return Err(Erreur { message: "« Image(phone: …) » attend une image plus légère, rangée à côté : phone: \"photo-petite.jpg\"".into(), pos: bloc.pos }),
            }
            // Une légende, sous l'image.
            match bloc.argument("caption").map(|a| &a.valeur) {
                None => sortie.push_str(&image),
                Some(Valeur::Texte(legende)) => sortie.push_str(&format!("<figure class=\"holo-figure\">{image}<figcaption>{}</figcaption></figure>", markdown(legende))),
                Some(_) => return Err(Erreur { message: "« Image(caption: …) » attend un texte entre guillemets : la légende".into(), pos: bloc.pos }),
            }
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
            if bloc.argument("rules").is_some() {
                return Err(Erreur { message: "un « If » qui range des règles se place dans « rules », pas dans « children »".into(), pos: bloc.pos });
            }
            let (valeur, comparaisons) = crate::etat::condition(bloc)?;
            let cle = crate::etat::cle(valeur, &comparaisons);
            sortie.push_str(&format!("<div class=\"{classes}\"{nom} data-if=\"{}\"{MARQUE}{cle}{MARQUE}>", echapper(&cle)));
            enfants(bloc, sortie, mondes, base)?;
            sortie.push_str("</div>");
            // Le « sinon » : ce qui se montre quand la condition est fausse (ADR-039).
            if let Some(Valeur::Liste(sinon)) = bloc.argument("else").map(|a| &a.valeur) {
                sortie.push_str(&format!("<div class=\"holo-If\" data-else=\"{}\"{MARQUE}^{cle}{MARQUE}>", echapper(&cle)));
                for element in sinon {
                    rendre(element, sortie, mondes, base, bloc)?;
                }
                sortie.push_str("</div>");
            }
        }
        // Un champ où le visiteur écrit un nombre, et une case qu'il coche. Chacun présente une
        // valeur de la page ; l'étiquette est obligatoire (ADR-027). `etat.rs` les a vérifiés.
        "Input" | "Checkbox" => {
            let (Some(Valeur::Nom(valeur)), Some(Valeur::Texte(etiquette))) = (bloc.argument("value").map(|a| &a.valeur), bloc.argument("label").map(|a| &a.valeur)) else {
                return Err(Erreur { message: format!("« {} » attend « value » et « label »", bloc.nom), pos: bloc.pos });
            };
            let valeur = echapper(valeur);
            if bloc.nom == "Input" && bloc.argument("lines").is_some() {
                // Un texte long : plusieurs lignes, ses retours à la ligne gardés (ADR-038).
                let lignes = match bloc.argument("lines").map(|a| &a.valeur) {
                    Some(Valeur::Entier(n)) => *n,
                    _ => 4,
                };
                let max = match bloc.argument("max").map(|a| &a.valeur) {
                    Some(Valeur::Entier(max)) => (*max as usize).min(crate::etat::TEXTE_MAX),
                    _ => crate::etat::TEXTE_LONG,
                };
                sortie.push_str(&format!(
                    "<label class=\"{classes}\"{nom}><span>{}</span><textarea rows=\"{lignes}\" maxlength=\"{max}\" data-bind=\"{valeur}\">{MARQUE}#{valeur}{MARQUE}</textarea></label>",
                    markdown(etiquette)
                ));
            } else if let (true, Some(Valeur::Nom(sorte))) = (bloc.nom == "Input", bloc.argument("type").map(|a| &a.valeur)) {
                // Une date, une heure, une couleur : le navigateur montre son propre choisisseur (ADR-042).
                sortie.push_str(&format!(
                    "<label class=\"{classes}\"{nom}><span>{}</span><input type=\"{}\" value=\"{MARQUE}#{valeur}{MARQUE}\" data-bind=\"{valeur}\"></label>",
                    markdown(etiquette),
                    echapper(sorte)
                ));
            } else if bloc.nom == "Input" {
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
        // Un choix : des boutons ronds (radio), ou une liste déroulante avec menu: true (ADR-038).
        "Choice" => {
            let (Some(Valeur::Nom(valeur)), Some(Valeur::Texte(etiquette))) = (bloc.argument("value").map(|a| &a.valeur), bloc.argument("label").map(|a| &a.valeur)) else {
                return Err(Erreur { message: "« Choice » attend « value », « label » et « options »".into(), pos: bloc.pos });
            };
            let valeur = echapper(valeur);
            let options = crate::etat::options_du_choix(bloc);
            if matches!(bloc.argument("menu").map(|a| &a.valeur), Some(Valeur::Bool(true))) {
                sortie.push_str(&format!("<label class=\"{classes}\"{nom}><span>{}</span><select data-bind=\"{valeur}\"><option value=\"\">—</option>", markdown(etiquette)));
                for option in options {
                    let o = echapper(option);
                    sortie.push_str(&format!("<option value=\"{o}\"{MARQUE}=selected|{valeur}|{o}{MARQUE}>{o}</option>"));
                }
                sortie.push_str("</select></label>");
            } else {
                sortie.push_str(&format!("<fieldset class=\"{classes}\"{nom}><legend>{}</legend>", markdown(etiquette)));
                for option in options {
                    let o = echapper(option);
                    sortie.push_str(&format!("<label><input type=\"radio\" name=\"choix-{valeur}\" value=\"{o}\" data-bind=\"{valeur}\"{MARQUE}=checked|{valeur}|{o}{MARQUE}><span>{o}</span></label>"));
                }
                sortie.push_str("</fieldset>");
            }
        }
        // Une vidéo : avec ses commandes, jamais lancée toute seule (ADR-038).
        "Video" => {
            let source = match bloc.argument("source").map(|a| &a.valeur) {
                Some(Valeur::Texte(s)) if chemin_sur(s) && (s.ends_with(".mp4") || s.ends_with(".webm")) => s,
                _ => return Err(Erreur { message: "« Video » attend « source » : une vidéo rangée à côté du fichier, en .mp4 ou .webm, comme \"film.mp4\"".into(), pos: bloc.pos }),
            };
            let Some(Valeur::Texte(etiquette)) = bloc.argument("label").map(|a| &a.valeur) else {
                return Err(Erreur { message: "« Video » attend « label » : ce que montre la vidéo, pour qui ne la voit pas".into(), pos: bloc.pos });
            };
            sortie.push_str(&format!(
                "<video class=\"{classes}\"{nom} src=\"{}{}\" controls preload=\"metadata\" playsinline aria-label=\"{}\"></video>",
                echapper(base),
                echapper(source),
                echapper(etiquette)
            ));
        }
        // Un tableau de données : une légende, une ligne de titres, des lignes (ADR-038).
        "Table" => {
            let ligne = |valeur: &Valeur, pos| -> Result<Vec<String>, Erreur> {
                match valeur {
                    Valeur::Liste(cellules) => cellules.iter().map(|c| match c {
                        Valeur::Texte(t) => Ok(markdown(t)),
                        _ => Err(Erreur { message: "une case de « Table » est un texte entre guillemets".into(), pos }),
                    }).collect(),
                    _ => Err(Erreur { message: "une ligne de « Table » s'écrit entre crochets : [\"Lundi\", \"9 h – 18 h\"]".into(), pos }),
                }
            };
            let tete = match bloc.argument("head") {
                Some(argument) => Some(ligne(&argument.valeur, argument.pos)?),
                None => None,
            };
            let Some(argument_lignes) = bloc.argument("rows") else {
                return Err(Erreur { message: "« Table » attend « rows » : rows: [ [\"Lundi\", \"9 h\"], [\"Mardi\", \"9 h\"] ]".into(), pos: bloc.pos });
            };
            let Valeur::Liste(rangees) = &argument_lignes.valeur else {
                return Err(Erreur { message: "« Table(rows: …) » est une liste de lignes : rows: [ [\"Lundi\", \"9 h\"] ]".into(), pos: argument_lignes.pos });
            };
            let largeur = tete.as_ref().map(Vec::len);
            sortie.push_str(&format!("<div class=\"holo-tableau\"><table class=\"{classes}\"{nom}>"));
            if let Some(Valeur::Texte(legende)) = bloc.argument("caption").map(|a| &a.valeur) {
                sortie.push_str(&format!("<caption>{}</caption>", markdown(legende)));
            }
            if let Some(tete) = &tete {
                sortie.push_str("<thead><tr>");
                for cellule in tete {
                    sortie.push_str(&format!("<th scope=\"col\">{cellule}</th>"));
                }
                sortie.push_str("</tr></thead>");
            }
            sortie.push_str("<tbody>");
            for rangee in rangees {
                let cellules = ligne(rangee, argument_lignes.pos)?;
                if largeur.is_some_and(|l| l != cellules.len()) {
                    return Err(Erreur { message: format!("chaque ligne de « Table » a autant de cases que « head » ({}) ; celle-ci en a {}", largeur.unwrap_or(0), cellules.len()), pos: argument_lignes.pos });
                }
                sortie.push_str("<tr>");
                for cellule in cellules {
                    sortie.push_str(&format!("<td>{cellule}</td>"));
                }
                sortie.push_str("</tr>");
            }
            sortie.push_str("</tbody></table></div>");
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
            // Un plateau garde ses proportions : 640 de large, `height` de haut. Il s'agrandit ou
            // rétrécit avec l'écran, et ce qu'il contient avec lui : une partie est la même sur
            // un téléphone et sur un grand écran. Il ne dépasse pas les quatre cinquièmes de la
            // hauteur de l'écran.
            let largeur = crate::etat::LARGEUR_DU_PLATEAU;
            let plus_large = 80.0 * largeur / hauteur;
            sortie.push_str(&format!("<div class=\"{classes}\"{nom} style=\"aspect-ratio:{largeur}/{hauteur};width:min(100%,{plus_large:.1}vh)\">"));
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
        // Une forme simple, d'une seule couleur : un rond, un carré, un triangle, un losange.
        "Shape" => {
            let (mut forme, mut allure) = (None, String::new());
            for argument in &bloc.arguments {
                match (argument.nom.as_deref(), &argument.valeur) {
                    (Some("name" | "x" | "y" | "drag"), _) => {}
                    (Some("form"), Valeur::Nom(mot)) if ["circle", "square", "triangle", "diamond"].contains(&mot.as_str()) => forme = Some(mot.as_str()),
                    (Some("form"), _) => return Err(Erreur { message: "« Shape(form: …) » attend l'un de ces mots : circle, square, triangle, diamond".into(), pos: argument.pos }),
                    (Some("color"), Valeur::Texte(couleur)) if est_couleur(couleur) => allure.push_str(&format!("--holo-color:{couleur};")),
                    (Some("color"), _) => return Err(Erreur { message: "« Shape(color: …) » attend une couleur entre guillemets, comme \"#E9B44C\"".into(), pos: argument.pos }),
                    (Some("size"), Valeur::Nombre { valeur, unite: Some(unite) }) if unite == "px" && (8.0..=400.0).contains(valeur) => allure.push_str(&format!("--holo-size:{valeur}px;--holo-n:{valeur};")),
                    (Some("size"), _) => return Err(Erreur { message: "« Shape(size: …) » attend une taille entre 8px et 400px".into(), pos: argument.pos }),
                    (Some(autre), _) => return Err(Erreur { message: format!("« Shape » n'a pas de paramètre « {autre} » ; paramètres possibles : form, color, size, name"), pos: argument.pos }),
                    (None, _) => return Err(Erreur { message: "chaque paramètre de « Shape » est nommé : Shape(form: circle, color: \"#E9B44C\", size: 48px)".into(), pos: argument.pos }),
                }
            }
            let Some(forme) = forme else {
                return Err(Erreur { message: "« Shape » attend « form » : Shape(form: circle)".into(), pos: bloc.pos });
            };
            // Une forme qui a un nom peut être touchée : c'est un vrai bouton, qu'on atteint au
            // clavier et qu'un lecteur d'écran nomme. Sans nom, c'est un dessin.
            match nom_de(bloc) {
                Some(n) => sortie.push_str(&format!("<button type=\"button\" class=\"{classes} holo-forme-{forme}\"{nom} aria-label=\"{}\" style=\"{allure}\"></button>", echapper(n))),
                None => sortie.push_str(&format!("<div class=\"{classes} holo-forme-{forme}\" style=\"{allure}\"></div>")),
            }
        }
        // Un son, qu'une règle fait entendre : Ding.play. Il ne se voit pas.
        "Sound" => {
            let mut source = None;
            for argument in &bloc.arguments {
                match (argument.nom.as_deref(), &argument.valeur) {
                    (Some("name" | "weight"), _) | (Some("label"), Valeur::Texte(_)) => {}
                    (Some("source"), Valeur::Texte(s)) if chemin_sur(s) && [".wav", ".mp3", ".ogg"].iter().any(|fin| s.ends_with(fin)) => source = Some(s),
                    (Some("source"), _) => {
                        return Err(Erreur { message: "« Sound(source: …) » attend un fichier de son rangé à côté du .holo : \"ding.wav\" (.wav, .mp3 ou .ogg)".into(), pos: argument.pos })
                    }
                    (Some(autre), _) => return Err(Erreur { message: format!("« Sound » n'a pas de paramètre « {autre} » ; paramètres possibles : name, source, weight"), pos: argument.pos }),
                    (None, _) => return Err(Erreur { message: "chaque paramètre de « Sound » est nommé : Sound(name: Ding, source: \"ding.wav\")".into(), pos: argument.pos }),
                }
            }
            // Avec une étiquette, le son est un lecteur, avec ses boutons, jamais lancé seul (ADR-042).
            if let (Some(source), Some(Valeur::Texte(etiquette))) = (source, bloc.argument("label").map(|a| &a.valeur)) {
                sortie.push_str(&format!("<audio class=\"{classes}\"{nom} controls preload=\"metadata\" src=\"{}{}\" aria-label=\"{}\"></audio>", echapper(base), echapper(source), echapper(etiquette)));
                return Ok(());
            }
            let (Some(source), false) = (source, nom.is_empty()) else {
                return Err(Erreur { message: "un son a un nom, pour qu'une règle puisse le jouer, et un fichier : Sound(name: Ding, source: \"ding.wav\") ; avec label:, c'est un lecteur".into(), pos: bloc.pos });
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
        // Une liste qui change pendant la visite (ADR-044) : une ligne par élément ; la page les
        // redessine quand la liste change.
        "Repeat" => {
            let Some(Valeur::Nom(liste)) = bloc.argument("over").map(|a| &a.valeur) else {
                return Err(Erreur { message: "« Repeat » a été déplié à la lecture ; ici, il attend « over: » : Repeat(over: tasks, children: [ … ])".into(), pos: bloc.pos });
            };
            sortie.push_str(&format!("<div class=\"holo-Liste\" data-liste=\"{}\">", echapper(liste)));
            sortie.push_str(&lignes(bloc, liste, base)?);
            sortie.push_str("</div>");
        }
        // Une glissière : choisir un nombre entre deux bornes (ADR-042).
        "Slider" => {
            let (Some(Valeur::Nom(valeur)), Some(Valeur::Texte(etiquette))) = (bloc.argument("value").map(|a| &a.valeur), bloc.argument("label").map(|a| &a.valeur)) else {
                return Err(Erreur { message: "« Slider » attend « value » et « label »".into(), pos: bloc.pos });
            };
            let borne = |p: &str, defaut: u64| match bloc.argument(p).map(|a| &a.valeur) {
                Some(Valeur::Entier(n)) => *n,
                _ => defaut,
            };
            let valeur = echapper(valeur);
            sortie.push_str(&format!(
                "<label class=\"{classes}\"{nom}><span>{}</span><input type=\"range\" min=\"{}\" max=\"{}\" value=\"{MARQUE}#{valeur}{MARQUE}\" data-bind=\"{valeur}\"></label>",
                markdown(etiquette),
                borne("min", 0),
                borne("max", 100)
            ));
        }
        // Une barre de progression : une jauge de vie, un téléchargement (ADR-042).
        "Progress" => {
            let Some(Valeur::Texte(etiquette)) = bloc.argument("label").map(|a| &a.valeur) else {
                return Err(Erreur { message: "« Progress » attend « label » : ce que mesure la barre".into(), pos: bloc.pos });
            };
            let max = match bloc.argument("max").map(|a| &a.valeur) {
                Some(Valeur::Entier(n)) => *n,
                _ => 100,
            };
            let (lien, depart) = match bloc.argument("value").map(|a| &a.valeur) {
                Some(Valeur::Nom(v)) => (format!(" data-progress=\"{}\"", echapper(v)), format!("{MARQUE}#{}{MARQUE}", echapper(v))),
                Some(Valeur::Entier(n)) => (String::new(), n.min(&max).to_string()),
                _ => return Err(Erreur { message: "« Progress » attend « value » : un nombre de la page, ou un nombre".into(), pos: bloc.pos }),
            };
            sortie.push_str(&format!("<label class=\"{classes}\"{nom}><span>{}</span><progress max=\"{max}\" value=\"{depart}\"{lien}></progress></label>", markdown(etiquette)));
        }
        // Un pli qui s'ouvre : une question, sa réponse (ADR-042). Il marche sans le moteur.
        "Details" => {
            let Some(Valeur::Texte(resume)) = bloc.argument("summary").map(|a| &a.valeur) else {
                return Err(Erreur { message: "« Details » attend « summary » : ce qu'on voit fermé, Details(summary: \"Livrez-vous ?\", children: [ … ])".into(), pos: bloc.pos });
            };
            let ouvert = match bloc.argument("open").map(|a| &a.valeur) {
                None | Some(Valeur::Bool(false)) => "",
                Some(Valeur::Bool(true)) => " open",
                Some(_) => return Err(Erreur { message: "« Details(open: …) » attend true ou false".into(), pos: bloc.pos }),
            };
            sortie.push_str(&format!("<details class=\"{classes}\"{nom}{ouvert}><summary>{}</summary>", markdown(resume)));
            enfants(bloc, sortie, mondes, base)?;
            sortie.push_str("</details>");
        }
        // Une fenêtre par-dessus la page (ADR-042) : une règle l'ouvre (Confirm.open) ; la croix,
        // la touche Échap ou une règle (Confirm.close) la ferment.
        "Dialog" => {
            if nom.is_empty() {
                return Err(Erreur { message: "« Dialog » a un nom, pour qu'une règle l'ouvre : Dialog(name: Confirm, children: [ … ]), puis On(Ask.tap, effect: Confirm.open)".into(), pos: bloc.pos });
            }
            sortie.push_str(&format!("<dialog class=\"{classes}\"{nom}><form method=\"dialog\" class=\"holo-fermer\"><button aria-label=\"Fermer\">✕</button></form>"));
            enfants(bloc, sortie, mondes, base)?;
            sortie.push_str("</dialog>");
        }
        // Un formulaire qu'on envoie (ADR-042) : une règle l'envoie (Contact.send) ; il dit ensuite
        // si l'envoi est arrivé (Contact.sent) ou non (Contact.failed).
        "Form" => {
            if nom.is_empty() {
                return Err(Erreur { message: "« Form » a un nom, pour qu'une règle l'envoie : Form(name: Contact, children: [ … ]), puis On(Send.tap, effect: Contact.send)".into(), pos: bloc.pos });
            }
            if let Some(Valeur::Liste(dedans)) = bloc.argument("children").map(|a| &a.valeur) {
                let mut imbrique = false;
                for valeur in dedans {
                    if let Valeur::Bloc(enfant) = valeur {
                        let _ = crate::regles::pour_chaque_bloc(enfant, &mut |b| {
                            imbrique |= b.nom == "Form";
                            Ok(())
                        });
                    }
                }
                if imbrique {
                    return Err(Erreur { message: "un formulaire dans un formulaire n'est pas permis".into(), pos: bloc.pos });
                }
            }
            sortie.push_str(&format!("<form class=\"{classes}\"{nom} novalidate>"));
            enfants(bloc, sortie, mondes, base)?;
            sortie.push_str("</form>");
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

/// Le caractère qui tient la place de l'élément d'une ligne pendant la fabrication. Il ne peut
/// pas venir d'un texte du visiteur : un texte saisi est nettoyé de ses caractères invisibles.
const ELEMENT: char = '\u{2}';
/// La marque d'un champ d'élément dans une ligne : `\u{3}0\u{3}` pour le premier champ montré.
const CHAMP: char = '\u{3}';

/// Les lignes d'une répétition dynamique, pour les éléments en cours de sa liste. Le texte de
/// l'élément est posé après la fabrication, échappé : ce qu'un visiteur a écrit ne devient jamais
/// une balise, ni du gras, ni une valeur montrée.
fn lignes(repetition: &Bloc, liste: &str, base: &str) -> Result<String, Erreur> {
    for argument in &repetition.arguments {
        match argument.nom.as_deref() {
            Some("over" | "children" | "rules" | "name") => {}
            Some(autre) => return Err(Erreur { message: format!("« Repeat(over: …) » n'a pas de paramètre « {autre} » ; paramètres possibles : over, children, rules"), pos: argument.pos }),
            None => return Err(Erreur { message: "chaque paramètre de « Repeat » est nommé : Repeat(over: tasks, children: [ … ])".into(), pos: argument.pos }),
        }
    }
    let Some(Valeur::Liste(modele)) = repetition.argument("children").map(|a| &a.valeur) else {
        return Err(Erreur { message: "« Repeat(over: …) » attend « children » : le modèle d'une ligne".into(), pos: repetition.pos });
    };
    let listes = crate::listes::en_cours();
    let Some((_, elements)) = listes.iter().find(|(nom, _)| nom == liste) else {
        return Err(Erreur { message: format!("« Repeat(over: {liste}) » : aucune liste ne s'appelle « {liste} » ; déclare-la, state: State({liste}: [])"), pos: repetition.pos });
    };
    // Chaque élément est posé dans le modèle après la fabrication, échappé : `{item}` et
    // `{item.title}` deviennent des marques, remplacées par le texte de l'élément ou du champ ;
    // `item.image` à la place d'une valeur devient le champ (ADR-051).
    fn marquer(valeur: &mut Valeur, champs: &[(String, String)], montres: &mut Vec<(String, Option<String>)>) {
        match valeur {
            Valeur::Texte(t) => {
                let mut sortie = String::new();
                let mut reste = t.as_str();
                while let Some(debut) = reste.find("{item") {
                    sortie.push_str(&reste[..debut]);
                    let apres = &reste[debut..];
                    let Some(fin) = apres.find('}') else {
                        sortie.push_str(apres);
                        reste = "";
                        break;
                    };
                    let dedans = &apres[1..fin];
                    if dedans == "item" {
                        sortie.push(ELEMENT);
                    } else if let Some(champ) = dedans.strip_prefix("item.") {
                        let (nom, format) = champ.split_once(':').map_or((champ, None), |(n, f)| (n, Some(f.to_string())));
                        sortie.push_str(&format!("{CHAMP}{}{CHAMP}", montres.len()));
                        montres.push((nom.to_string(), format));
                    } else {
                        sortie.push_str(&apres[..=fin]);
                    }
                    reste = &apres[fin + 1..];
                }
                sortie.push_str(reste);
                *t = sortie;
            }
            Valeur::Nom(n) if n.starts_with("item.") => {
                let champ = &n["item.".len()..];
                if let Some((_, v)) = champs.iter().find(|(c, _)| c == champ) {
                    *valeur = match v.parse::<u64>() {
                        Ok(e) if !v.starts_with('0') || v == "0" => Valeur::Entier(e),
                        _ => Valeur::Texte(v.clone()),
                    };
                }
            }
            Valeur::Liste(l) => l.iter_mut().for_each(|v| marquer(v, champs, montres)),
            Valeur::Bloc(b) => b.arguments.iter_mut().for_each(|a| marquer(&mut a.valeur, champs, montres)),
            _ => {}
        }
    }
    let mut sortie = String::new();
    let mut mondes = String::new();
    for (rang, element) in elements.iter().enumerate() {
        let champs = crate::listes::champs(element);
        let mut montres = Vec::new();
        let mut ligne = String::new();
        for valeur in modele {
            let mut copie = valeur.clone();
            marquer(&mut copie, &champs, &mut montres);
            rendre(&copie, &mut ligne, &mut mondes, base, repetition)?;
        }
        let mut ligne = ligne.replace(ELEMENT, &echapper(&crate::listes::texte_de(element)));
        for (i, (nom, format)) in montres.iter().enumerate() {
            let brut = champs.iter().find(|(c, _)| c == nom).map(|(_, v)| v.clone()).unwrap_or_default();
            let montre = match (format, brut.parse::<u64>()) {
                (Some(f), Ok(n)) => crate::format::formater(nom, n, f, &crate::format::langue()),
                _ => brut,
            };
            ligne = ligne.replace(&format!("{CHAMP}{i}{CHAMP}"), &echapper(&montre));
        }
        sortie.push_str(&format!("<div class=\"holo-ligne\" data-rang=\"{rang}\">{ligne}</div>"));
    }
    Ok(sortie)
}

/// Les lignes d'une liste pour cet état : la page les pose à la place des anciennes (ADR-044).
pub fn lignes_de_liste(programme: &Programme, base: &str, nombres: &crate::etat::Etat, textes: &crate::etat::Textes, listes: &crate::listes::Listes, nom: &str) -> String {
    crate::listes::mettre_en_cours(listes.clone());
    let Some((repetition, _)) = crate::listes::repetitions(programme).into_iter().find(|(_, l)| l == nom) else { return String::new() };
    let mut montrees = crate::etat::a_montrer(programme, nombres);
    montrees.extend(crate::listes::comptes(listes));
    let reponses = crate::etat::conditions(programme, &crate::etat::avec_textes(&montrees, textes));
    lignes(repetition, nom, base).map(|html| remplir_marques(html, &montrees, textes, &reponses)).unwrap_or_default()
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
pub(crate) fn chemin_sur(source: &str) -> bool {
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
    let mut html = alterner(&alterner(&alterner(&echapper(texte), "`", "code"), "**", "strong"), "*", "em");
    // `~~barré~~`, `==surligné==`, `m^2^` et `H~2~O` (ADR-042).
    html = alterner(&alterner(&alterner(&alterner(&html, "~~", "s"), "==", "mark"), "^", "sup"), "~", "sub").replace('\n', "<br>");
    // `{cart}` : l'endroit où s'affiche une valeur de la page. `site_html` y écrit son départ,
    // la page d'entrée la tient à jour.
    for nom in crate::etat::noms_dans(texte) {
        html = html.replace(&format!("{{{nom}}}"), &format!("<span data-state=\"{nom}\"></span>"));
    }
    // `{minute:00}` : la valeur, avec son format (ADR-043).
    for (nom, format) in crate::format::formats_dans(texte) {
        html = html.replace(&format!("{{{nom}:{format}}}"), &format!("<span data-state=\"{nom}\" data-format=\"{format}\"></span>"));
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
            "<div class=\"holo-Page\" data-title=\"My shop\"><header class=\"holo-Header\">",
            "<nav class=\"holo-Nav\">",
            "</header><main><h1 class=\"holo-H1\">My shop</h1>",
            "<div class=\"holo-Stack\"><div><img class=\"holo-Image\"",
            "<div class=\"holo-Text holo-s-badge\">New</div>",
            "<h4 class=\"holo-H4\">Weekdays</h4>",
            "<footer class=\"holo-Footer\"><hr class=\"holo-Hr\">",
            "@media (hover:hover){.holo-Button:hover{background:#2a2a4e;}}",
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
        // Un décor s'écrit alt: "" ; sans alt, l'image est refusée (ADR-038).
        assert!(page("Page(children: [ Image(source: \"a.png\", alt: \"\") ])").unwrap().contains("alt=\"\""));
        assert!(page("Page(children: [ Image(source: \"a.png\") ])").unwrap_err().message.contains("attend « alt »"));
        assert!(page("Page(children: [ Image(source: \"a.png\", alt: 3) ])").unwrap_err().message.contains("attend un texte"));
        assert!(page("Page(children: [ Hr(color: red) ])").unwrap_err().message.contains("s'écrit « Hr() »"));
        assert!(page("Page(children: [ Quote(\"x\", by: 3) ])").unwrap_err().message.contains("qui l'a dit"));
        // Le caractère qui sert de marque aux conditions ne passe pas par un texte.
        assert!(!page("Page(children: [ \"a\u{1}b\" ])").unwrap().contains('\u{1}'));
    }

    #[test]
    fn une_forme_est_un_dessin_ou_un_bouton() {
        let html = page("Page(children: [ Shape(form: circle, color: \"#E9B44C\", size: 40px), Shape(name: Cible, form: triangle) ])").unwrap();
        assert!(html.contains("<div class=\"holo-Shape holo-forme-circle\" style=\"--holo-color:#E9B44C;--holo-size:40px;--holo-n:40;\"></div>"), "{html}");
        assert!(html.contains("<button type=\"button\" class=\"holo-Shape holo-forme-triangle\" data-name=\"Cible\" aria-label=\"Cible\" style=\"\"></button>"), "{html}");
        // Une forme nommée se touche, comme un bouton, et se place sur un plateau.
        page("Page(state: State(n: 0, sx: 5), children: [ Board(children: [ Shape(name: S, form: square, x: sx, y: 50, drag: true) ]) ], rules: [ On(S.tap, effect: n.add(1)) ])").unwrap();
        for (source, message) in [
            ("Page(children: [ Shape(color: \"red\") ])", "attend « form »"),
            ("Page(children: [ Shape(form: hexagon) ])", "circle, square, triangle, diamond"),
            ("Page(children: [ Shape(form: circle, color: \"url(x)\") ])", "attend une couleur"),
            ("Page(children: [ Shape(form: circle, size: 5000px) ])", "entre 8px et 400px"),
            ("Page(children: [ Shape(form: circle, border: 2) ])", "n'a pas de paramètre « border »"),
        ] {
            let erreur = page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn un_plateau_place_ses_blocs_ou_l_on_veut() {
        let html = page(
            "Page(state: State(sx: 70, sy: 250), children: [ Board(height: 200px, children: [ Point(name: Star, seed: 7, x: sx, y: sy), Button(name: B, text: \"b\", x: 10, y: 90), P(\"libre\") ]) ])",
        )
        .unwrap();
        // L'étoile suit deux valeurs ; au départ elle est à leur place, sans dépasser le plateau.
        assert!(html.contains("<div class=\"holo-Board\" style=\"aspect-ratio:640/200;width:min(100%,256.0vh)\"><div class=\"holo-place\" data-x=\"sx\" data-y=\"sy\" style=\"--x:70;--y:100\"><button type=\"button\" class=\"holo-Point\" data-name=\"Star\""), "{html}");
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

    #[test]
    fn une_page_qui_bouge_seule_demande_le_moteur_tout_de_suite() {
        // La boutique attend qu'on la touche : elle s'affiche sans le moteur.
        let boutique = crate::vue_a_plat(include_str!("../../exemples/boutique-comparee/boutique.holo"), "").unwrap();
        assert!(!boutique.contains("data-vivant"));
        // Un jeu a des horloges et le clavier : le moteur doit arriver tout de suite.
        for jeu in [include_str!("../../exemples/jeu/attraper.holo"), include_str!("../../exemples/jeu/panier.holo")] {
            assert!(crate::vue_a_plat(jeu, "").unwrap().contains("data-vivant>"));
        }
    }


    #[test]
    fn le_mouvement_devient_du_css_et_ne_bouge_que_si_on_l_ecrit() {
        // Sans mouvement écrit, rien de plus dans la page.
        let calme = crate::vue_a_plat("Page(children: [ H1(\"Hi\") ])", "").unwrap();
        assert!(!calme.contains("holo-anime") && !calme.contains("@keyframes"));
        // Une entrée : le bloc est enveloppé, ses images clés vont de la pose écrite au repos.
        let html = crate::vue_a_plat("Page(children: [ H1(\"Hi\", enter: Enter(y: 40px, opacity: 0, at: 1s, for: 0.5s, ease: linear)) ])", "").unwrap();
        assert!(html.contains("<div class=\"holo-anime hm1\"><h1 class=\"holo-H1\">Hi</h1></div>"), "{html}");
        assert!(html.contains("@keyframes hm1{from{opacity:0;translate:0px 40px;}to{opacity:1;translate:0px 0px;}}"), "{html}");
        assert!(html.contains(".hm1{animation:hm1 0.5s linear 1s both}"), "{html}");
        // Lettre à lettre : chaque lettre a son rang, et son délai.
        let html = crate::vue_a_plat("Page(children: [ P(\"Oh\", enter: Enter(scale: 0, letters: 0.1s)) ])", "").unwrap();
        assert!(html.contains("<span class=\"holo-lettre\" style=\"--i:1\">h</span>"), "{html}");
        assert!(html.contains("animation-delay:calc(0s + var(--i) * 0.1s)"), "{html}");
        // Des scènes qui recommencent : la deuxième apparaît à la moitié du tour.
        let html = crate::vue_a_plat("Page(children: [ Scenes(repeat: forever, children: [ Scene(for: 2s, children: [ \"a\" ]), Scene(for: 2s, children: [ P(\"b\", enter: Enter(opacity: 0, at: 1s, for: 1s)) ]) ]) ])", "").unwrap();
        assert!(html.contains("@keyframes hs2{0%{opacity:0;visibility:hidden}50.000%{opacity:0;visibility:visible}"), "{html}");
        // L'entrée de la deuxième scène part à 3 s sur un tour de 4 s : 75 %.
        assert!(html.contains("75.000%{opacity:0;animation-timing-function:"), "{html}");
        assert!(html.contains("prefers-reduced-motion:reduce"), "{html}");
        // Ce qui est refusé, avec une phrase qui dit quoi faire.
        for (source, message) in [
            ("Page(children: [ H1(\"a\", enter: Enter()) ])", "dit ce qui bouge"),
            ("Page(children: [ H1(\"a\", enter: Enter(y: 40)) ])", "un nombre en px"),
            ("Page(children: [ H1(\"a\", enter: Enter(opacity: 3)) ])", "de 0 à 1"),
            ("Page(children: [ H1(\"a\", enter: Enter(y: 4px, ease: wobble)) ])", "l'une de ces courbes"),
            ("Page(children: [ H1(\"a\", enter: Enter(y: 4px, back: false)) ])", "n'a pas de paramètre « back »"),
            ("Page(children: [ H1(\"a\", enter: Loop(y: 4px)) ])", "attend Enter"),
            ("Page(children: [ Shape(form: circle, enter: Enter(y: 4px, letters: 0.1s)) ])", "coupe un texte en lettres"),
            ("Page(children: [ H1(\"a\", enter: Enter(y: 4px, each: 0.1s)) ])", "a des « children »"),
            ("Page(children: [ Scenes(children: [ P(\"a\") ]) ])", "ne range que des"),
            ("Page(children: [ Scenes(children: [ Scene(children: []) ]) ])", "combien de temps elle dure"),
            ("Page(children: [ Scene(for: 1s, children: []) ])", "se range dans des scènes"),
        ] {
            let erreur = crate::vue_a_plat(source, "").unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }


    #[test]
    fn les_reperes_les_etats_la_superposition() {
        // L'en-tête et le pied sortent du contenu principal ; Main dit où il est.
        let html = crate::vue_a_plat("Page(children: [ Header(children: [ Nav(children: [ A(\"Home\", to: \"a.holo\") ]) ]), Main(children: [ H1(\"Hi\") ]), Footer(children: [ \"End\" ]) ])", "").unwrap();
        assert!(html.contains("<header class=\"holo-Header\"><nav class=\"holo-Nav\"><a class=\"holo-A\" href=\"a.holo\">Home</a></nav></header><main><h1 class=\"holo-H1\">Hi</h1></main><footer class=\"holo-Footer\"><p class=\"holo-P\">End</p></footer>"), "{html}");
        // Les titres jusqu'à H6.
        assert!(crate::vue_a_plat("Page(children: [ H1(\"a\"), H2(\"b\"), H3(\"c\"), H4(\"d\"), H5(\"e\"), H6(\"f\") ])", "").unwrap().contains("<h6 class=\"holo-H6\">f</h6>"));
        // Le texte suit le réglage du visiteur ; un grand titre rétrécit sur un petit écran.
        let html = crate::vue_a_plat("Page(children: [ H1(\"a\"), P(\"b\") ])\nP { font-size: 18px; }\nH1 { font-size: 64px; }", "").unwrap();
        assert!(html.contains(".holo-P{font-size:1.125rem;}"), "{html}");
        assert!(html.contains(".holo-H1{font-size:clamp(1.5rem,10vw,4rem);}"), "{html}");
        // Les états d'un style.
        let html = crate::vue_a_plat("Page(children: [ Button.go(name: G, text: \"Go\") ])\n.go { background: blue; hover: { background: navy; } focus: { border: 2px solid white; } active: { opacity: 0.5; } }", "").unwrap();
        assert!(html.contains(".holo-s-go{background:blue;transition:"), "{html}");
        assert!(html.contains("@media (hover:hover){.holo-s-go:hover{background:navy;}}"), "{html}");
        assert!(html.contains(".holo-s-go:focus-visible{border:2px solid white;}"), "{html}");
        assert!(html.contains(".holo-s-go:active{opacity:0.5;}"), "{html}");
        // La superposition.
        let html = crate::vue_a_plat("Page(children: [ Stack(children: [ Image(source: \"a.png\", alt: \"x\"), Text(\"New\", align: topRight) ]) ])", "").unwrap();
        assert!(html.contains("<div class=\"holo-Stack\"><div><img"), "{html}");
        assert!(html.contains("<div class=\"holo-pose\" style=\"align-self:start;justify-self:end\"><div class=\"holo-Text\">New</div></div></div>"), "{html}");
        // Ce qui est refusé.
        for (source, message) in [
            ("Page(children: [ Row(children: [ Main(children: []) ]) ])", "se place directement dans la page"),
            ("Page(children: [ Nav(color: red, children: []) ])", "n'a pas de paramètre « color »"),
            ("Page(children: [ Stack(children: [ P(\"a\"), P(\"b\", align: middle) ]) ])", "l'une de ces places"),
            ("Page(children: [ P(\"a\") ])\nP { hover: { color: red; } hover: { color: blue; } }", "donné deux fois"),
            ("Page(children: [ P(\"a\") ])\nP { visited: { color: red; } }", "n'est pas un état"),
            ("Page(children: [ P(\"a\") ])\nP { hover: { position: absolute; } }", "la disposition vient des blocs"),
            ("Page(children: [ H1(\"a\"), H7(\"b\") ])", "de « H1 » à « H6 »"),
        ] {
            let erreur = crate::verifier_page(source).err().or_else(|| crate::vue_a_plat(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }


    #[test]
    fn le_lot_1_langue_video_tableau_texte_choix() {
        // La langue, la description et l'image de partage.
        let html = crate::vue_a_plat("Page(lang: \"fr\", description: \"Une boutique\", image: \"p.png\", children: [ H1(\"a\") ])", "/ex/").unwrap();
        assert!(html.contains(" data-lang=\"fr\" data-description=\"Une boutique\" data-image=\"/ex/p.png\">"), "{html}");
        // La vidéo : avec ses boutons, jamais lancée seule.
        let html = crate::vue_a_plat("Page(children: [ Video(source: \"f.mp4\", label: \"Un tour\") ])", "").unwrap();
        assert!(html.contains("<video class=\"holo-Video\" src=\"f.mp4\" controls preload=\"metadata\" playsinline aria-label=\"Un tour\"></video>"), "{html}");
        assert!(!html.contains("autoplay"));
        // Le tableau.
        let html = crate::vue_a_plat("Page(children: [ Table(caption: \"Horaires\", head: [\"Jour\", \"Heures\"], rows: [ [\"Lundi\", \"**9 h**\"] ]) ])", "").unwrap();
        assert!(html.contains("<div class=\"holo-tableau\"><table class=\"holo-Table\"><caption>Horaires</caption><thead><tr><th scope=\"col\">Jour</th><th scope=\"col\">Heures</th></tr></thead><tbody><tr><td>Lundi</td><td><strong>9 h</strong></td></tr></tbody></table></div>"), "{html}");
        // Le texte long et le choix, avec leur valeur de départ.
        let html = crate::vue_a_plat("Page(state: State(message: \"Bonjour\", taille: \"M\"), children: [ Input(value: message, label: \"Message\", lines: 4), Choice(value: taille, label: \"Taille\", options: [\"S\", \"M\"]), Choice(value: taille, label: \"Encore\", options: [\"S\", \"M\"], menu: true) ])", "").unwrap();
        assert!(html.contains("<textarea rows=\"4\" maxlength=\"1000\" data-bind=\"message\">Bonjour</textarea>"), "{html}");
        assert!(html.contains("<input type=\"radio\" name=\"choix-taille\" value=\"S\" data-bind=\"taille\"><span>S</span>"), "{html}");
        assert!(html.contains("<input type=\"radio\" name=\"choix-taille\" value=\"M\" data-bind=\"taille\" checked><span>M</span>"), "{html}");
        assert!(html.contains("<option value=\"M\" selected>M</option>"), "{html}");
        // Le choix n'accepte que ses options ; le texte long garde ses retours à la ligne.
        let source = "Page(state: State(taille: \"\", message: \"\"), children: [ Choice(value: taille, label: \"T\", options: [\"S\", \"M\"]), Input(value: message, label: \"M\", lines: 3) ])";
        let depart = crate::etat_initial(source);
        assert!(crate::saisir(source, &depart, "taille", "M").contains("taille='M"));
        assert!(!crate::saisir(source, &depart, "taille", "XXL").contains("XXL"));
        let deux_lignes = crate::saisir(source, &depart, "message", "a\nb");
        assert!(deux_lignes.contains("message='a%0Ab"));
        // Relu pour le geste suivant, le texte long garde ses deux lignes.
        assert!(crate::saisir(source, &deux_lignes, "taille", "S").contains("message='a%0Ab"));
        // Ce qui est refusé.
        for (source, message) in [
            ("Page(lang: \"français\", children: [])", "attend une langue"),
            ("Page(children: [ Video(source: \"f.avi\", label: \"x\") ])", "en .mp4 ou .webm"),
            ("Page(children: [ Video(source: \"f.mp4\") ])", "attend « label »"),
            ("Page(children: [ Table(head: [\"a\", \"b\"], rows: [ [\"1\"] ]) ])", "autant de cases"),
            ("Page(state: State(n: 0), children: [ Input(value: n, label: \"x\", lines: 3) ])", "sa valeur est un texte"),
            ("Page(state: State(n: 0), children: [ Choice(value: n, label: \"x\", options: [\"a\", \"b\"]) ])", "un choix présente un texte"),
            ("Page(state: State(t: \"\"), children: [ Choice(value: t, label: \"x\", options: [\"a\"]) ])", "de 2 à 20 textes"),
            ("Page(state: State(t: \"\"), children: [ Choice(value: t, label: \"x\", options: [\"a\", \"a\"]) ])", "le même texte"),
        ] {
            let erreur = crate::verifier_page(source).err().or_else(|| crate::vue_a_plat(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn le_lot_6_calculer_et_formats() {
        crate::regler_maintenant([2026, 10, 6, 2, 9, 5]);
        let source = "Page(state: State(n: 10, total: 123450), children: [ P(\"{weekday:name} {day} {month:name}, {hour} h {minute:00} ; {total:cents} ; {n:number}\"), Repeat(items: [ Item(price: 1999) ], children: [ P(\"{item.price:cents}\") ]) ], rules: [])";
        let html = crate::vue_a_plat(source, "").unwrap();
        assert!(html.contains("<span data-state=\"weekday\" data-format=\"name\">mardi</span> <span data-state=\"day\">6</span> <span data-state=\"month\" data-format=\"name\">octobre</span>, <span data-state=\"hour\">9</span> h <span data-state=\"minute\" data-format=\"00\">05</span> ; <span data-state=\"total\" data-format=\"cents\">1\u{202F}234,50</span> ; <span data-state=\"n\" data-format=\"number\">10</span>"), "{html}");
        assert!(html.contains("<p class=\"holo-P\">19,99</p>"), "{html}");
        // En anglais, les séparateurs et les noms changent.
        let anglais = crate::vue_a_plat(&source.replace("Page(state", "Page(lang: \"en\", state"), "").unwrap();
        assert!(anglais.contains(">Tuesday<") && anglais.contains(">1,234.50<") && anglais.contains("<p class=\"holo-P\">19.99</p>"), "{anglais}");
        // Multiplier, diviser.
        let source = "Page(state: State(a: 7, b: 0), children: [ Button(name: M, text: \"m\"), Button(name: D, text: \"d\"), Button(name: Z, text: \"z\") ], rules: [ On(M.tap, effect: a.mul(3)), On(D.tap, effect: a.div(2)), On(Z.tap, effect: a.div(b)) ])";
        assert_eq!(crate::arbitrer(source, "a=7;b=0", "M.tap"), "a=21;b=0");
        assert_eq!(crate::arbitrer(source, "a=7;b=0", "D.tap"), "a=3;b=0");
        assert_eq!(crate::arbitrer(source, "a=7;b=0", "Z.tap"), "a=7;b=0");
        for (source, message) in [
            ("Page(state: State(a: 1), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: a.div(0)) ])", "on ne divise pas par 0"),
            ("Page(state: State(a: 1), children: [ P(\"{a:euros}\") ])", "format inconnu"),
            ("Page(state: State(a: 1), children: [ P(\"{a:name}\") ])", "seuls weekday et month"),
            ("Page(state: State(t: \"\"), children: [ P(\"{t:00}\") ])", "un format s'applique à un nombre"),
            ("Page(children: [ Repeat(items: [ Item(t: \"x\") ], children: [ P(\"{item.t:cents}\") ]) ])", "doit être un nombre entier"),
        ] {
            let erreur = crate::verifier_page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn le_lot_5_le_html_utile_et_le_formulaire() {
        let source = "Page(icon: \"i.svg\", state: State(taille: 60, vies: 2, jour: \"\", nom: \"Ada\", message: \"\"), children: [ H1(\"a\"), A(\"bas\", to: \"#Bas\"), P(\"~~120~~ ==oui== m^2^ H~2~O\"), Image(source: \"g.jpg\", alt: \"x\", phone: \"p.jpg\", caption: \"Une *légende*\"), Sound(source: \"s.wav\", label: \"Le son\"), Slider(value: taille, label: \"T\", min: 20, max: 120), Input(value: jour, label: \"J\", type: date), Progress(value: vies, max: 3, label: \"Vies\"), Details(summary: \"Q ?\", open: true, children: [ P(\"R\") ]), Dialog(name: Fenetre, children: [ P(\"D\") ]), Form(name: Contact, children: [ Input(value: nom, label: \"N\"), Input(value: message, label: \"M\", lines: 3), Slider(value: taille, label: \"T2\", min: 20, max: 120) ]), H2(\"b\", name: Bas) ])";
        let html = crate::vue_a_plat(source, "/ex/").unwrap();
        for attendu in [
            " data-icon=\"/ex/i.svg\"",
            "<a class=\"holo-A\" href=\"#Bas\">bas</a>",
            "<h2 class=\"holo-H2\" data-name=\"Bas\" id=\"Bas\">b</h2>",
            "<s>120</s> <mark>oui</mark> m<sup>2</sup> H<sub>2</sub>O",
            "<figure class=\"holo-figure\"><picture><source media=\"(max-width:640px)\" srcset=\"/ex/p.jpg\"><img class=\"holo-Image\" src=\"/ex/g.jpg\" alt=\"x\"></picture><figcaption>Une <em>légende</em></figcaption></figure>",
            "<audio class=\"holo-Sound\" controls preload=\"metadata\" src=\"/ex/s.wav\" aria-label=\"Le son\"></audio>",
            "<input type=\"range\" min=\"20\" max=\"120\" value=\"60\" data-bind=\"taille\">",
            "<input type=\"date\" value=\"\" data-bind=\"jour\">",
            "<progress max=\"3\" value=\"2\" data-progress=\"vies\"></progress>",
            "<details class=\"holo-Details\" open><summary>Q ?</summary><p class=\"holo-P\">R</p></details>",
            "<dialog class=\"holo-Dialog\" data-name=\"Fenetre\"><form method=\"dialog\" class=\"holo-fermer\">",
            "<form class=\"holo-Form\" data-name=\"Contact\" novalidate>",
        ] {
            assert!(html.contains(attendu), "manque : {attendu}\n{html}");
        }
        // La glissière reste dans ses bornes ; la date n'accepte qu'une date.
        let depart = crate::etat_initial(source);
        assert!(crate::saisir(source, &depart, "taille", "500").starts_with("taille=120;"));
        assert!(crate::saisir(source, &depart, "taille", "3").starts_with("taille=20;"));
        assert!(crate::saisir(source, &depart, "jour", "2026-10-06").contains("jour='2026%2D10%2D06"));
        assert!(!crate::saisir(source, &depart, "jour", "demain").contains("demain"));
        // Le formulaire envoie les valeurs de ses champs, et seulement elles.
        let message = crate::saisir(source, &depart, "message", "Bonjour \"toi\"\nà bientôt");
        assert_eq!(crate::envoi(source, &message, "Contact"), "{\"form\":\"Contact\",\"values\":{\"nom\":\"Ada\",\"message\":\"Bonjour \\\"toi\\\"\\nà bientôt\",\"taille\":60}}");
        assert_eq!(crate::envoi(source, &message, "Personne"), "");
        // Les règles : ouvrir, fermer, envoyer ; l'envoi arrivé ou non.
        crate::verifier_page("Page(state: State(ok: 0), children: [ Button(name: B, text: \"b\"), Dialog(name: D, children: [ P(\"x\") ]), Form(name: F, children: [ P(\"y\") ]) ], rules: [ On(B.tap, effect: [D.open, F.send]), On(F.sent, effect: [ok.set(1), D.close]), On(F.failed, effect: ok.set(2)) ])").unwrap();
        for (source, message) in [
            ("Page(children: [ A(\"x\", to: \"#Nulle\") ])", "aucun bloc ne s'appelle « Nulle »"),
            ("Page(icon: \"i.gif\", children: [])", "en .png, .svg ou .ico"),
            ("Page(children: [ Image(source: \"a.png\", alt: \"\", caption: 3) ])", "la légende"),
            ("Page(state: State(t: \"\"), children: [ Slider(value: t, label: \"x\") ])", "présente un nombre de la page"),
            ("Page(state: State(n: 5), children: [ Slider(value: n, label: \"x\", min: 9, max: 3) ])", "min doit être plus petit que max"),
            ("Page(state: State(n: 0), children: [ Input(value: n, label: \"x\", type: date) ])", "écrit un texte"),
            ("Page(state: State(t: \"\"), children: [ Input(value: t, label: \"x\", type: week) ])", "date, time ou color"),
            ("Page(children: [ Dialog(children: [ P(\"x\") ]) ])", "« Dialog » a un nom"),
            ("Page(children: [ Form(name: A, children: [ Form(name: B, children: []) ]) ])", "un formulaire dans un formulaire"),
            ("Page(state: State(n: 0), children: [ Button(name: B, text: \"b\"), Form(name: F, children: []) ], rules: [ On(B.tap, effect: F.open) ])", "un « Form » offre send"),
        ] {
            let erreur = crate::verifier_page(source).err().or_else(|| crate::vue_a_plat(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn le_lot_4_le_css_utile() {
        let source = "Page(fonts: [ Font(family: \"Carlito\", source: \"carlito.woff2\") ], children: [ P.carte(\"x\") ])\n\
            Page { --or: #E9B44C; --marge: 16px; background: --or; dark: { --or: #806020; } }\n\
            P { line-height: 1.6; letter-spacing: -0.5px; text-transform: uppercase; text-decoration: line-through; text-shadow: 0 2px 6px #000000AA; phone: { display: none; } }\n\
            .carte { background: url(\"fond.jpg\"); box-shadow: 0 8px 24px #00000080, 0 1px 2px black; padding: --marge; rotate: -2deg; scale: 1.05; transition: 0.3s; hover: { rotate: 0deg; } }";
        let html = crate::vue_a_plat(source, "/ex/").unwrap();
        for attendu in [
            "@font-face{font-family:\"Carlito\";src:url(\"/ex/carlito.woff2\");font-display:swap}",
            ".holo-Page{--or:#E9B44C;--marge:16px;background:var(--or);}",
            "@media (prefers-color-scheme:dark){.holo-Page{--or:#806020;}}",
            "@media (max-width:640px){.holo-P{display:none;}}",
            "line-height:1.6;letter-spacing:-0.5px;text-transform:uppercase;text-decoration:line-through;text-shadow:0 2px 6px #000000AA;",
            "background:url(\"/ex/fond.jpg\") center/cover no-repeat;",
            "padding:var(--marge);rotate:-2deg;scale:1.05;transition:background 0.3s,color 0.3s,",
            "@media (hover:hover){.holo-s-carte:hover{rotate:0deg;}}",
        ] {
            assert!(html.contains(attendu), "manque : {attendu}\n{html}");
        }
        // Avec sa propre durée, la carte ne reçoit pas la durée automatique du survol.
        assert!(!html.contains("scale .15s"), "{html}");
        for (styles, message) in [
            ("P { color: --rouge; }", "« --rouge » n'est définie nulle part"),
            ("Page { --rouge: url(x); }", "une couleur ou une taille"),
            ("P { line-height: 24px; }", "un nombre sans unité"),
            ("P { display: none; }", "phone: { display: none; }"),
            ("P { phone: { display: flex; } }", "ne prend que « none »"),
            ("P { background: linear-gradient(red); }", "un dégradé"),
            ("P { background: url(\"../secret.png\"); }", "une image rangée à côté"),
            ("P { box-shadow: 0 4px; }", "une ombre"),
            ("P { rotate: 3turn; }", "un angle"),
            ("P { transition: 9s; }", "de 0 à 2s"),
            ("P { night: { color: red; } }", "n'est pas un état"),
        ] {
            let source = format!("Page(children: [ P(\"x\") ])\n{styles}");
            let erreur = crate::verifier_page(&source).unwrap_err();
            assert!(erreur.message.contains(message), "{styles}\n→ {erreur}");
        }
        for (source, message) in [
            ("Page(fonts: [ Font(family: \"A\", source: \"a.exe\") ], children: [])", ".woff2, .woff"),
            ("Page(fonts: [ Font(source: \"a.woff2\") ], children: [])", "attend « family » et « source »"),
            ("Page(fonts: Font(family: \"A\", source: \"a.woff2\"), children: [])", "une liste de polices"),
        ] {
            let erreur = crate::verifier_page(source).unwrap_err();
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

    #[test]
    fn le_lot_2_survol_sinon_attente_heure() {
        // Le survol : la page sait quels blocs l'écoutent ; il change des valeurs.
        let source = "Page(state: State(vu: 0), children: [ Column(name: Carte, children: [ P(\"a\") ]), If(vu, is: 1, children: [ P(\"oui\") ], else: [ P(\"non\") ]) ], rules: [ On(Carte.hover, effect: vu.set(1)), On(Carte.hoverEnd, effect: vu.set(0)) ])";
        let html = crate::vue_a_plat(source, "").unwrap();
        assert!(html.contains("<div class=\"holo-Column\" data-name=\"Carte\" data-hover tabindex=\"0\""), "{html}");
        // Le sinon : montré au départ, puisque la condition est fausse ; le « oui » est caché.
        assert!(html.contains("data-if=\"vu|is=1\" hidden><p class=\"holo-P\">oui</p></div><div class=\"holo-If\" data-else=\"vu|is=1\"><p class=\"holo-P\">non</p></div>"), "{html}");
        let survole = crate::arbitrer(source, &crate::etat_initial(source), "Carte.hover");
        assert_eq!(survole, "vu=1");
        assert_eq!(crate::arbitrer(source, &survole, "Carte.hoverEnd"), "vu=0");
        // Une attente : une seule fois ; sous une condition, elle ne court que si la condition est vraie.
        let source = "Page(state: State(bonjour: 0, message: 0), children: [ Text(\"{bonjour}\") ], rules: [ After(2s, effect: bonjour.set(1)), If(message, is: 1, rules: [ After(3s, effect: message.set(0)) ]) ])";
        let depart = crate::etat_initial(source);
        assert_eq!(crate::delais(source, &depart), "2000:1;3000:0");
        assert_eq!(crate::delais(source, "bonjour=0;message=1"), "2000:1;3000:1");
        assert_eq!(crate::arbitrer(source, &depart, "after:0"), "bonjour=1;message=0");
        assert_eq!(crate::arbitrer(source, "bonjour=1;message=1", "after:1"), "bonjour=1;message=0");
        assert!(crate::vue_a_plat(source, "").unwrap().contains(" data-vivant"));
        // L'heure du visiteur : donnée au moteur, lue par la page, jamais changée par elle.
        crate::regler_maintenant([2026, 10, 6, 2, 14, 5]);
        let source = "Page(children: [ Sound(name: Ding, source: \"d.wav\"), P(\"{hour} h {minute}\"), If(hour, over: 8, under: 18, children: [ P(\"ouvert\") ], else: [ P(\"fermé\") ]) ], rules: [ When(hour, is: 15, effect: Ding.play) ])\n".to_string();
        let html = crate::vue_a_plat(&source, "").unwrap();
        assert!(html.contains("<span data-state=\"hour\">14</span> h <span data-state=\"minute\">5</span>"), "{html}");
        assert!(html.contains("data-else=\"hour|over=8|under=18\" hidden>"), "{html}");
        assert_eq!(crate::etat_initial(&source), "hour=14;minute=5");
        // Une minute plus tard, l'heure avance ; à 15 h, la règle qui guette l'heure sonne.
        crate::regler_maintenant([2026, 10, 6, 2, 15, 0]);
        assert_eq!(crate::avancer_l_horloge(&source, "hour=14;minute=59"), "hour=15;minute=0;!=Ding.play");
        // L'état écrit ne change pas l'heure : c'est celle donnée au moteur.
        assert_eq!(crate::arbitrer(&source, "hour=3;minute=3", "rien"), "hour=15;minute=0");
        assert_eq!(crate::etat::depuis_secondes_unix(0), [1970, 1, 1, 4, 0, 0]);
        assert_eq!(crate::etat::depuis_secondes_unix(1_791_244_800 + 14 * 3600 + 5 * 60), [2026, 10, 6, 2, 14, 5]);
        // Ce qui est refusé.
        for (source, message) in [
            ("Page(state: State(hour: 0), children: [])", "est l'heure du visiteur"),
            ("Page(state: State(n: 0), children: [ Button(name: B, text: \"b\") ], rules: [ On(B.tap, effect: hour.add(1)) ])", "on ne la change pas"),
            ("Page(state: State(n: 0), children: [ Input(value: hour, label: \"h\") ])", "on ne l'écrit pas"),
            ("Page(state: State(n: 0), keep: [hour], children: [ P(\"{hour}\") ])", "ne se garde pas"),
            ("Page(state: State(n: 0), children: [ If(n, is: 0, rules: [ Every(1s, effect: n.add(1)) ], else: [ P(\"x\") ]) ])", "va avec children"),
            ("Page(state: State(n: 0), children: [ If(n, is: 0, children: [ P(\"a\") ], else: P(\"x\")) ])", "va avec children"),
            ("Page(state: State(n: 0), children: [], rules: [ After(effect: n.add(1)) ])", "une attente s'écrit"),
            ("Page(state: State(n: 0), children: [], rules: [ After(2h, effect: n.add(1)) ])", "une attente s'écrit"),
            ("Page(state: State(n: 0), children: [ Point(name: A, seed: 1), P(\"x\") ], rules: [ On(A.hover, effect: A.enter) ])", "demande que le visiteur touche"),
            ("Page(state: State(n: 0), children: [ Main(name: M, children: [ P(\"x\") ]) ], rules: [ On(M.hover, effect: n.add(1)) ])", "signal inconnu « hover »"),
        ] {
            let erreur = crate::verifier_page(source).err().or_else(|| crate::vue_a_plat(source, "").err()).unwrap_or_else(|| panic!("accepté : {source}"));
            assert!(erreur.message.contains(message), "{source}\n→ {erreur}");
        }
    }

}
