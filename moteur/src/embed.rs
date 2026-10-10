//! Une page dans la page (ADR-117) : la page d'un autre site, posée dans la sienne : une carte,
//! une vidéo, une publication.
//!
//! ```holo
//! Page(
//!   embeds: ["www.openstreetmap.org"],
//!   children: [
//!     Embed(from: "https://www.openstreetmap.org/export/embed.html?bbox=15.25,-4.37,15.38,-4.28&layer=mapnik",
//!           label: "Map: the centre of Kinshasa", image: "map.svg"),
//!   ],
//! )
//! ```
//!
//! Le moteur tient les règles, jamais l'auteur :
//! - seulement un site que la page liste (`embeds:`), comparé exactement, en HTTPS ; jamais une
//!   adresse IP, ce PC ni un nom du réseau local ;
//! - une façade : une image du site de l'auteur, le titre et le nom de l'autre site, sur un vrai
//!   bouton. Rien ne part vers l'autre site avant que le visiteur le touche : ni cadre, ni
//!   connexion préparée, ni image chargée chez lui. L'adresse n'est que dans un attribut `data-` ;
//! - au toucher, la page légère (`web/page.html`) pose la page intégrée, enfermée (`sandbox`), sans
//!   caméra, micro ni position, avec son titre pour le lecteur d'écran, et le clavier y entre ;
//! - sans JavaScript, la façade est un lien vers la page de l'autre site (`noscript`), qui s'ouvre
//!   dans un nouvel onglet ;
//! - `holo serve` dit au navigateur de n'accepter un cadre que de ces sites (`frame-src`).

use crate::flat::{escape, path_on};
use crate::holo::{Block, Error, Pos, Program, Value};
use crate::rules::for_each_block;

/// Le réglage de la page : `embeds: ["www.openstreetmap.org"]`.
pub const SETTING: &str = "embeds";
/// Les sites qu'une page peut intégrer, au plus.
pub const SITES_MAX: usize = 16;
/// L'adresse d'une page intégrée, au plus.
pub const ADDRESS_MAX: usize = 2048;
/// Le titre d'une page intégrée, au plus.
pub const LABEL_MAX: usize = 200;
/// Les images d'une façade.
const IMAGES: &[&str] = &[".svg", ".png", ".jpg", ".jpeg", ".webp", ".avif", ".gif"];
/// La fin des noms qui ne sont pas un site public : ce PC, le réseau local (une box, une
/// imprimante), les noms réservés aux essais et aux exemples.
const NOT_PUBLIC: &[&str] = &["localhost", "local", "internal", "lan", "home", "arpa", "test", "example", "invalid"];

/// Un nom de site, comme `www.openstreetmap.org` : des lettres, des chiffres et des tirets, en
/// morceaux séparés par des points. Jamais une adresse IP (`127.0.0.1`, `0x7f.1`, `2130706433`),
/// ni ce PC, ni un nom du réseau local. Rend le nom en minuscules, ou la raison d'un refus.
pub fn site_name(written: &str) -> Result<String, String> {
    let name = written.to_ascii_lowercase();
    let refusal = || format!("« {written} » n'est pas un nom de site : des lettres, des chiffres et des tirets, en morceaux séparés par des points, comme www.openstreetmap.org");
    if name.is_empty() || name.len() > 253 {
        return Err(refusal());
    }
    let parts: Vec<&str> = name.split('.').collect();
    let last = parts.last().copied().unwrap_or("");
    // Une adresse IP s'écrit en chiffres (en décimal, en octal ou en hexadécimal, `0x7f`) : son
    // dernier morceau commence par un chiffre ; celui d'un nom de site, par une lettre.
    if last.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(format!("« {written} » est une adresse IP : une page intégrée vient d'un site nommé, comme www.openstreetmap.org"));
    }
    if NOT_PUBLIC.contains(&last) {
        return Err(format!("« {written} » n'est pas un site public (ce PC, le réseau local ou un nom d'essai) : une page intégrée vient d'un site d'Internet"));
    }
    if parts.len() < 2 || parts.iter().any(|part| part.is_empty() || part.len() > 63 || part.starts_with('-') || part.ends_with('-') || !part.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')) {
        return Err(refusal());
    }
    Ok(name)
}

/// Le site d'une adresse de page intégrée, lue strictement : `https://`, un nom de site, puis le
/// chemin et les paramètres. Ni port, ni nom et mot de passe, ni valeur `{…}`. Rend le nom du site
/// en minuscules, ou la raison d'un refus.
pub fn site_of(address: &str) -> Result<String, String> {
    if address.len() > ADDRESS_MAX {
        return Err(format!("une adresse a {ADDRESS_MAX} caractères au plus"));
    }
    let Some(rest) = address.strip_prefix("https://") else {
        return Err(if address.to_ascii_lowercase().starts_with("http://") {
            "seul HTTPS est permis : en « http:// », n'importe qui sur le chemin lit et change la page ; écris « https:// »".to_string()
        } else {
            "une page intégrée vient d'un autre site, en HTTPS : « https:// » puis le nom du site ; ni « javascript: », ni « data: », ni un fichier".to_string()
        });
    };
    let cut = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, target) = rest.split_at(cut);
    if authority.starts_with('[') {
        return Err("une adresse IP n'est pas un site : écris le nom du site, comme www.openstreetmap.org".into());
    }
    if authority.contains('@') {
        return Err("ni nom ni mot de passe dans l'adresse (« nom@ ») : ils se liraient dans la page".into());
    }
    if authority.contains(':') {
        return Err("l'adresse s'écrit sans port (« :8080 ») : HTTPS, sur son port habituel".into());
    }
    if authority.is_empty() {
        return Err("le nom du site suit « https:// », comme https://www.openstreetmap.org/…".into());
    }
    let site = site_name(authority)?;
    // Ce qui suit le nom : les caractères d'une adresse (RFC 3986), rien d'autre. Un caractère
    // spécial s'écrit en %XX ; « {…} » ne lit aucune valeur : l'adresse est fixe.
    for c in target.chars() {
        match c {
            '{' | '}' => return Err("l'adresse d'une page intégrée est fixe : aucune valeur « {…} » ne s'écrit dedans".into()),
            c if c.is_ascii_alphanumeric() || "-._~!$&'()*+,;=:@/?#%[]".contains(c) => {}
            ' ' => return Err("une adresse s'écrit sans espace : une espace s'écrit %20".into()),
            c => return Err(format!("une adresse s'écrit sans « {c} » ni accent : un caractère spécial s'écrit en %XX")),
        }
    }
    let bytes = target.as_bytes();
    if bytes.iter().enumerate().any(|(i, b)| *b == b'%' && !(bytes.get(i + 1).is_some_and(u8::is_ascii_hexdigit) && bytes.get(i + 2).is_some_and(u8::is_ascii_hexdigit))) {
        return Err("un « % » dans l'adresse est suivi de deux chiffres hexadécimaux, comme %20".into());
    }
    Ok(site)
}

/// Les sites que la page liste, vérifiés, en minuscules, dans l'ordre où elle les écrit.
pub fn sites(program: &Program) -> Vec<String> {
    match program.root.argument(SETTING).map(|a| &a.value) {
        Some(Value::List(sites)) => sites.iter().filter_map(|site| if let Value::Text(t) = site { site_name(t).ok() } else { None }).collect(),
        _ => Vec::new(),
    }
}

/// La règle que `holo serve` donne au navigateur pour les cadres d'une page : seulement ces sites,
/// en HTTPS ; aucun, si la page n'en liste pas.
pub fn frame_policy(sites: &[String]) -> String {
    if sites.is_empty() {
        return "frame-src 'none'".into();
    }
    format!("frame-src {}", sites.iter().map(|site| format!("https://{site}")).collect::<Vec<_>>().join(" "))
}

/// Lit `embeds:` strictement : de 1 à 16 noms de site, chacun une fois.
fn listed(program: &Program) -> Result<Vec<(String, Pos)>, Error> {
    let Some(argument) = program.root.argument(SETTING) else { return Ok(Vec::new()) };
    let example = "embeds: [\"www.openstreetmap.org\", \"www.youtube-nocookie.com\"]";
    let Value::List(elements) = &argument.value else {
        return Err(Error { message: format!("« embeds » est la liste des sites que la page peut intégrer : {example}"), pos: argument.pos });
    };
    if elements.is_empty() || elements.len() > SITES_MAX {
        return Err(Error { message: format!("une page permet de 1 à {SITES_MAX} sites : {example}"), pos: argument.pos });
    }
    let mut found: Vec<(String, Pos)> = Vec::new();
    for element in elements {
        let Value::Text(written) = element else {
            return Err(Error { message: format!("« embeds » contient des noms de site entre guillemets : {example}"), pos: argument.pos });
        };
        let message = if written.contains("://") || written.contains('/') {
            Some(format!("« {written} » : écris seulement le nom du site, sans « https:// » ni chemin, comme \"www.openstreetmap.org\""))
        } else if written.contains('*') {
            Some(format!("« {written} » : chaque site s'écrit en entier, sans « * » : le moteur le compare exactement, et ne devine aucun autre site"))
        } else {
            site_name(written).err()
        };
        if let Some(message) = message {
            return Err(Error { message, pos: argument.pos });
        }
        let site = written.to_ascii_lowercase();
        if found.iter().any(|(known, _)| *known == site) {
            return Err(Error { message: format!("le site « {site} » est écrit deux fois dans « embeds »"), pos: argument.pos });
        }
        found.push((site, argument.pos));
    }
    Ok(found)
}

/// Ce qu'une page intégrée dit d'elle : son adresse et son site, son titre, son image.
struct Parts<'a> {
    address: &'a str,
    site: String,
    label: &'a str,
    image: Option<&'a str>,
}

/// Lit et vérifie les paramètres d'un `Embed`. Chaque refus dit pourquoi.
fn parts(block: &Block) -> Result<Parts<'_>, Error> {
    let refused = |message: String| Error { message, pos: block.pos };
    if block.arguments.iter().any(|a| a.name.is_none()) {
        return Err(refused("chaque paramètre de « Embed » est nommé : Embed(from: \"https://…\", label: \"…\", image: \"apercu.svg\")".into()));
    }
    let address = match block.argument("from").map(|a| &a.value) {
        Some(Value::Text(address)) => address.as_str(),
        _ => return Err(refused("« Embed » attend « from » : l'adresse de la page à intégrer, en HTTPS, comme from: \"https://www.openstreetmap.org/export/embed.html?…\"".into())),
    };
    let site = site_of(address).map_err(|reason| refused(format!("« Embed(from: …) » : {reason}")))?;
    let label = match block.argument("label").map(|a| &a.value) {
        Some(Value::Text(label)) if !label.trim().is_empty() => label.as_str(),
        _ => return Err(refused("« Embed » attend « label » : ce que montre la page intégrée, pour qui ne la voit pas ; c'est aussi le texte du bouton qui la charge, label: \"Carte : le centre de Kinshasa\"".into())),
    };
    if label.chars().count() > LABEL_MAX || label.contains('\n') {
        return Err(refused(format!("« Embed(label: …) » est un texte d'une ligne, de {LABEL_MAX} caractères au plus")));
    }
    if label.contains('{') {
        return Err(refused("« Embed(label: …) » est un texte fixe : il ne montre pas de valeur « {…} »".into()));
    }
    let image = match block.argument("image").map(|a| &a.value) {
        None => None,
        Some(Value::Text(file)) if path_on(file) && IMAGES.iter().any(|end| file.to_ascii_lowercase().ends_with(end)) => Some(file.as_str()),
        Some(_) => {
            return Err(refused("« Embed(image: …) » attend une image rangée à côté du fichier (.svg, .png, .jpg, .webp…) : elle se montre avant le toucher, sans rien demander à l'autre site".into()))
        }
    };
    Ok(Parts { address, site, label, image })
}

/// Vérifie les pages intégrées d'un fichier et la liste de ses sites. Chaque `Embed` vient d'un
/// site de la liste, comparé exactement ; chaque site de la liste sert. Un morceau à importer
/// (`Component`) n'a pas de liste : la page qui l'importe la donne.
pub fn check(program: &Program) -> Result<(), Error> {
    let listed = listed(program)?;
    let fragment = program.root.name == "Component";
    let mut used: Vec<String> = Vec::new();
    for_each_block(&program.root, &mut |block| {
        if block.name != "Embed" {
            return Ok(());
        }
        let Parts { site, .. } = parts(block)?;
        if !fragment && !listed.iter().any(|(known, _)| *known == site) {
            let near = listed.iter().find(|(known, _)| known.ends_with(&format!(".{site}")) || site.ends_with(&format!(".{known}"))).map(|(known, _)| known);
            let message = match near {
                Some(known) => format!("« {site} » n'est pas « {known} » : un site se compare exactement ; ajoute « {site} » à la liste de la page s'il le faut, embeds: [\"{site}\"]"),
                None if program.root.name != "Page" => "une page intégrée se pose dans une page, qui liste ses sites : Page(embeds: [\"www.openstreetmap.org\"], children: [ Embed(…) ])".to_string(),
                None => format!("« {site} » n'est pas un site permis : ajoute-le à la liste de la page, embeds: [\"{site}\"] (le nom exact, sans « https:// »)"),
            };
            return Err(Error { message, pos: block.argument("from").map_or(block.pos, |a| a.pos) });
        }
        used.push(site);
        Ok(())
    })?;
    if let Some((site, pos)) = listed.iter().find(|(site, _)| !used.contains(site)) {
        return Err(Error { message: format!("« {site} » est permis, mais aucune page intégrée ne vient de ce site : retire-le de « embeds » ; la liste ne garde que les sites dont la page a besoin"), pos: *pos });
    }
    Ok(())
}

/// La façade d'une page intégrée. Avec JavaScript, un vrai bouton : une image du site de l'auteur,
/// le titre, et le site qui se chargera ; la page légère le montre, puis pose la page intégrée au
/// toucher. Sans JavaScript, un lien vers la page de l'autre site, dans un nouvel onglet. Aucune
/// adresse de l'autre site n'est dans un `src` ni un `href` que le navigateur suivrait de lui-même :
/// le lien est dans `noscript`, que le navigateur ne lit pas quand JavaScript marche.
pub fn html(block: &Block, classes: &str, name: &str, base: &str, french: bool) -> Result<String, Error> {
    let Parts { address, site, label, image } = parts(block)?;
    let (load, open) = if french { ("Charger depuis", "Ouvrir sur") } else { ("Load from", "Open on") };
    let new_tab = if french { "dans un nouvel onglet" } else { "in a new tab" };
    let picture = image.map(|file| format!("<img class=\"holo-embed-image\" src=\"{}{}\" alt=\"\" loading=\"lazy\" decoding=\"async\">", escape(base), escape(file))).unwrap_or_default();
    let (address, label, site) = (escape(address), escape(label), escape(&site));
    // La virgule cachée sépare, pour le lecteur d'écran, le titre du site qui se chargera.
    let text = |line: String| format!("<span class=\"holo-embed-text\"><span class=\"holo-embed-label\">{label}</span><span class=\"holo-embed-site\"><span class=\"holo-hidden\">, </span>{line}</span></span>");
    Ok(format!(
        "<div class=\"{classes}\"{name}><button type=\"button\" class=\"holo-embed-load\" data-embed=\"{address}\" data-label=\"{label}\" hidden>{picture}{}</button><noscript><a class=\"holo-embed-link\" href=\"{address}\" target=\"_blank\" rel=\"noopener noreferrer\">{picture}{}</a></noscript></div>",
        text(format!("{load} {site}")),
        text(format!("{open} {site}, {new_tab}"))
    ))
}

#[cfg(test)]
mod embed_tests {
    use crate::holo::read;

    const MAP: &str = "https://www.openstreetmap.org/export/embed.html?bbox=15.25%2C-4.37%2C15.38%2C-4.28&layer=mapnik";

    fn page(embeds: &str, children: &str) -> String {
        format!("Page(title: \"Essai\", {embeds}children: [ H1(\"Essai\"), {children} ])")
    }

    fn map(from: &str) -> String {
        page("embeds: [\"www.openstreetmap.org\"], ", &format!("Embed(from: \"{from}\", label: \"Carte\", image: \"carte.svg\")"))
    }

    fn refused(source: &str) -> String {
        match crate::check_page(source) {
            Ok(_) => panic!("accepté, alors qu'il devait être refusé :\n{source}"),
            Err(error) => error.message,
        }
    }

    #[test]
    fn an_embed_waits_for_a_tap_behind_a_local_image() {
        // Règle 2 : rien ne part vers l'autre site avant le toucher. Ni cadre, ni connexion préparée,
        // ni image chargée chez lui : l'adresse n'est que dans des attributs `data-`, et dans un lien
        // que le navigateur ne lit que sans JavaScript.
        let source = page(
            "embeds: [\"www.openstreetmap.org\"], ",
            &format!("Embed(name: Map, from: \"{MAP}\", label: \"Carte : le \\\"centre\\\" <de> Kinshasa\", image: \"140-carte.svg\")"),
        )
        .replace("\\\"", "'");
        let program = crate::check_page(&source).unwrap();
        let html = crate::flat::page_html(&program, "/lecons/").unwrap();
        let address = MAP.replace('&', "&amp;");
        let (before, inside) = html.split_once("<noscript>").unwrap();
        let (inside, after) = inside.split_once("</noscript>").unwrap();
        for text in [before, after] {
            assert!(!text.contains("<iframe") && !text.contains("preconnect") && !text.contains("dns-prefetch") && !text.contains("prefetch") && !text.contains("preload"), "{html}");
            assert!(!text.contains("src=\"https://") && !text.contains("href=\"https://") && !text.contains("srcset"), "{html}");
        }
        assert!(before.contains(&format!("<div class=\"holo-Embed\" data-name=\"Map\"><button type=\"button\" class=\"holo-embed-load\" data-embed=\"{address}\" data-label=\"Carte : le 'centre' &lt;de&gt; Kinshasa\" hidden>")), "{html}");
        // L'image de la façade est un fichier du site de l'auteur ; le lecteur d'écran entend le titre.
        assert!(before.contains("<img class=\"holo-embed-image\" src=\"/lecons/140-carte.svg\" alt=\"\" loading=\"lazy\" decoding=\"async\">"), "{html}");
        assert!(before.contains("<span class=\"holo-embed-label\">Carte : le 'centre' &lt;de&gt; Kinshasa</span><span class=\"holo-embed-site\"><span class=\"holo-hidden\">, </span>Charger depuis www.openstreetmap.org</span>"), "{html}");
        // Règle 6 : sans JavaScript, un lien vers la page de l'autre site, avec le titre, dans un
        // nouvel onglet, sans dire à l'autre site d'où l'on vient.
        assert!(inside.starts_with(&format!("<a class=\"holo-embed-link\" href=\"{address}\" target=\"_blank\" rel=\"noopener noreferrer\">")), "{inside}");
        assert!(inside.contains("Ouvrir sur www.openstreetmap.org, dans un nouvel onglet</span>"), "{inside}");
        // En anglais, les mots de la façade suivent la langue de la page ; sans image, la façade reste.
        let english = crate::flat::page_html(&read(&source.replace("Page(title:", "Page(lang: \"en\", title:").replace(", image: \"140-carte.svg\"", "")).unwrap(), "").unwrap();
        assert!(english.contains("Load from www.openstreetmap.org</span>") && english.contains("Open on www.openstreetmap.org, in a new tab</span>") && !english.contains("<img"), "{english}");
        // Le style de base : une taille qui suit l'écran, le bouton caché tant que la page légère ne l'a pas pris.
        assert!(html.contains(":where(.holo-Embed){display:block;position:relative;width:100%;aspect-ratio:16/9;") && html.contains(":where(.holo-embed-load[hidden]){display:none}"), "{html}");
    }

    #[test]
    fn only_https_to_a_listed_site_compared_exactly() {
        // Règle 1 : HTTPS seulement, vers un site de la liste, comparé exactement.
        crate::check_page(&map(MAP)).unwrap();
        crate::check_page(&map("https://WWW.OpenStreetMap.org/export/embed.html")).unwrap();
        for (from, reason) in [
            ("http://www.openstreetmap.org/x", "seul HTTPS est permis"),
            ("javascript:alert(1)", "ni « javascript: »"),
            ("data:text/html,<script>alert(1)</script>", "ni « data: »"),
            ("carte.html", "en HTTPS"),
            ("//www.openstreetmap.org/x", "en HTTPS"),
            ("https://1.2.3.4/x", "est une adresse IP"),
            ("https://127.1/x", "est une adresse IP"),
            ("https://0x7f.0.0.1/x", "est une adresse IP"),
            ("https://2130706433/x", "est une adresse IP"),
            ("https://[::1]/x", "une adresse IP n'est pas un site"),
            ("https://localhost/x", "n'est pas un site public"),
            ("https://imprimante.local/x", "n'est pas un site public"),
            ("https://box.home.arpa/x", "n'est pas un site public"),
            ("https://www.openstreetmap.org:8443/x", "sans port"),
            ("https://www.openstreetmap.org@pirate.example.org/x", "ni nom ni mot de passe"),
            ("https:///x", "le nom du site suit"),
            ("https://www.openstreetmap.org/a b", "sans espace"),
            ("https://www.openstreetmap.org/<script>", "sans « < »"),
            ("https://www.openstreetmap.org/carte-é", "ni accent"),
            ("https://www.openstreetmap.org/%zz", "deux chiffres hexadécimaux"),
            ("https://www.vimeo.com/x", "« www.vimeo.com » n'est pas un site permis"),
            // Ni un sous-domaine, ni le domaine au-dessus, ni un nom qui finit pareil : le nom exact.
            ("https://pirate.www.openstreetmap.org/x", "un site se compare exactement"),
            ("https://openstreetmap.org/x", "un site se compare exactement"),
            ("https://www.openstreetmap.org.pirate.example.org/x", "n'est pas un site permis"),
            ("https://wwwopenstreetmap.org/x", "n'est pas un site permis"),
        ] {
            let message = refused(&map(from));
            assert!(message.contains(reason), "{from} → {message}");
        }
        // Une valeur de la page, même déclarée, ne s'écrit pas dans l'adresse : elle est fixe.
        let valued = map("https://www.openstreetmap.org/{place}").replace("Page(title: \"Essai\", ", "Page(title: \"Essai\", state: State(place: \"Kinshasa\"), ");
        assert!(refused(&valued).contains("aucune valeur"), "{valued}");
        let long = format!("https://www.openstreetmap.org/{}", "a".repeat(2048));
        assert!(refused(&map(&long)).contains("2048 caractères au plus"));
        // Un site permis par une autre page ne l'est pas ici : une page sans liste n'intègre rien.
        assert!(refused(&page("", &format!("Embed(from: \"{MAP}\", label: \"Carte\")"))).contains("n'est pas un site permis"));
        // Une page intégrée se pose dans une page, qui liste ses sites ; un morceau à importer n'a
        // pas de liste, la page qui l'importe la donne.
        assert!(refused(&format!("Point(name: P, seed: 1, inside: World(children: [ Embed(from: \"{MAP}\", label: \"Carte\") ]))")).contains("se pose dans une page"));
        crate::check_page(&format!("Component(name: MapCard, children: [ Embed(from: \"{MAP}\", label: \"Carte\") ])")).unwrap();
    }

    #[test]
    fn an_embed_has_a_title_and_a_local_image() {
        // Règle 5 : un titre obligatoire, pour le lecteur d'écran et pour le bouton ; une image de façade
        // rangée à côté du fichier, jamais chez l'autre site.
        let with = |params: &str| page("embeds: [\"www.openstreetmap.org\"], ", &format!("Embed({params})"));
        for (params, reason) in [
            (format!("from: \"{MAP}\""), "attend « label »"),
            (format!("from: \"{MAP}\", label: \"\""), "attend « label »"),
            (format!("from: \"{MAP}\", label: \"   \""), "attend « label »"),
            (format!("from: \"{MAP}\", label: 3"), "attend « label »"),
            (format!("from: \"{MAP}\", label: \"{}\"", "x".repeat(201)), "200 caractères au plus"),
            ("label: \"Carte\"".to_string(), "attend « from »"),
            ("from: 3, label: \"Carte\"".to_string(), "attend « from »"),
            (format!("\"{MAP}\", label: \"Carte\""), "chaque paramètre de « Embed » est nommé"),
            (format!("from: \"{MAP}\", label: \"Carte\", image: \"https://www.openstreetmap.org/a.png\""), "rangée à côté du fichier"),
            (format!("from: \"{MAP}\", label: \"Carte\", image: \"../a.png\""), "rangée à côté du fichier"),
            (format!("from: \"{MAP}\", label: \"Carte\", image: \"/a.png\""), "rangée à côté du fichier"),
            (format!("from: \"{MAP}\", label: \"Carte\", image: \"carte.holo\""), "rangée à côté du fichier"),
            (format!("from: \"{MAP}\", label: \"Carte\", autoplay: true"), "n'a pas de paramètre « autoplay »"),
            (format!("from: \"{MAP}\", label: \"Carte\", sandbox: \"allow-top-navigation\""), "n'a pas de paramètre « sandbox »"),
        ] {
            let message = refused(&with(&params));
            assert!(message.contains(reason), "{params} → {message}");
        }
        crate::check_page(&with(&format!("from: \"{MAP}\", label: \"Carte\", image: \"images/Carte.PNG\""))).unwrap();
        // Le titre est un texte fixe : une valeur de la page, même déclarée, ne s'y montre pas.
        let valued = with(&format!("from: \"{MAP}\", label: \"Carte de {{ville}}\"")).replace("Page(title: \"Essai\", ", "Page(title: \"Essai\", state: State(ville: \"Kinshasa\"), ");
        assert!(refused(&valued).contains("texte fixe"), "{valued}");
    }

    #[test]
    fn the_list_of_sites_is_read_strictly() {
        let embed = format!("Embed(from: \"{MAP}\", label: \"Carte\")");
        for (embeds, reason) in [
            ("embeds: \"www.openstreetmap.org\", ", "est la liste des sites"),
            ("embeds: [], ", "de 1 à 16 sites"),
            ("embeds: [www], ", "des noms de site entre guillemets"),
            ("embeds: [\"https://www.openstreetmap.org\"], ", "écris seulement le nom du site"),
            ("embeds: [\"www.openstreetmap.org/export\"], ", "écris seulement le nom du site"),
            ("embeds: [\"*.openstreetmap.org\"], ", "sans « * »"),
            ("embeds: [\"www.openstreetmap.org\", \"WWW.openstreetmap.org\"], ", "écrit deux fois"),
            ("embeds: [\"192.168.1.1\"], ", "est une adresse IP"),
            ("embeds: [\"localhost\"], ", "n'est pas un site public"),
            ("embeds: [\"routeur.lan\"], ", "n'est pas un site public"),
            ("embeds: [\"openstreetmap\"], ", "n'est pas un nom de site"),
            ("embeds: [\"www.open_streetmap.org\"], ", "n'est pas un nom de site"),
            // Un site permis dont la page n'a pas besoin élargirait la règle des cadres pour rien.
            ("embeds: [\"www.openstreetmap.org\", \"www.youtube-nocookie.com\"], ", "« www.youtube-nocookie.com » est permis, mais aucune page intégrée ne vient de ce site"),
        ] {
            let message = refused(&page(embeds, &embed));
            assert!(message.contains(reason), "{embeds} → {message}");
        }
        let seventeen: Vec<String> = (0..17).map(|n| format!("\"s{n}.example.org\"")).collect();
        assert!(refused(&page(&format!("embeds: [{}], ", seventeen.join(", ")), &embed)).contains("de 1 à 16 sites"));
        // La liste se règle sur la page, jamais dans un monde.
        let world = format!("Page(embeds: [\"www.openstreetmap.org\"], children: [ {embed}, Point(name: P, seed: 1, inside: World(embeds: [\"www.openstreetmap.org\"], children: [ {embed} ])) ])");
        assert!(refused(&world).contains("se règle sur la page"));
    }

    #[test]
    fn the_frame_policy_names_only_the_listed_sites() {
        // Règle 4 : la règle que holo serve donne au navigateur.
        let program = read(&page("embeds: [\"www.openstreetmap.org\", \"www.youtube-nocookie.com\"], ", &format!("Embed(from: \"{MAP}\", label: \"Carte\"), Embed(from: \"https://www.youtube-nocookie.com/embed/x\", label: \"Vidéo\")"))).unwrap();
        assert_eq!(super::frame_policy(&super::sites(&program)), "frame-src https://www.openstreetmap.org https://www.youtube-nocookie.com");
        assert_eq!(super::frame_policy(&super::sites(&read(&page("", "P(\"x\")")).unwrap())), "frame-src 'none'");
        // Un nom qui n'en est pas un n'entre jamais dans l'en-tête, même si la page est refusée.
        let forged = read(&page("embeds: [\"www.a.org\\r\\nSet-Cookie: x=1\", \"evil.example.org 'unsafe-inline'\", \"www.b.org\"], ", "P(\"x\")")).unwrap();
        assert_eq!(super::frame_policy(&super::sites(&forged)), "frame-src https://www.b.org");
    }
}
