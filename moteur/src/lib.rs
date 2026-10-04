//! Moteur HoloCode, sprint Big Bang.
//!
//! Le cœur est en Rust pur et se teste sur le PC (`cargo test`) :
//! - `holo` lit un fichier `.holo` ;
//! - `blocs` vérifie que chaque bloc existe et que les titres ne sautent pas de niveau ;
//! - `styles` vérifie les styles, écrits comme en CSS ;
//! - `regles` vérifie les noms, les règles et les budgets, et dit ce qu'un signal demande ;
//! - `etat` tient les valeurs d'une page et arbitre les demandes qui les changent ;
//! - `plat` fabrique la page web ordinaire d'un fichier (la vue à plat) ;
//! - `univers` en fait un monde, entièrement calculé à partir d'une graine ;
//! - `navigation` gère le morcellement, le zoom, l'entrée et la sortie.
//!
//! La partie qui parle au navigateur et à la carte graphique (`web`, `rendu`) n'est
//! compilée que pour WebAssembly.

pub mod blocs;
pub mod etat;
pub mod graine;
pub mod holo;
pub mod mosaique;
pub mod navigation;
pub mod plat;
pub mod regles;
pub mod styles;
pub mod univers;
pub mod vue;

#[cfg(target_arch = "wasm32")]
mod rendu;
#[cfg(target_arch = "wasm32")]
mod web;

use holo::{Erreur, Programme, Valeur};
use univers::PointDecl;

/// Lit et vérifie un fichier `.holo`, sans rien exécuter. Toute erreur est rendue avec sa
/// ligne et sa colonne.
pub fn verifier(source: &str) -> Result<PointDecl, Erreur> {
    let programme = holo::lire(source)?;
    blocs::verifier_blocs(&programme)?;
    styles::verifier_styles(&programme)?;
    univers::point_depuis(&programme)
}

/// Lit et vérifie un fichier `.holo` entier : blocs, titres, styles, noms, règles, budgets.
pub fn verifier_page(source: &str) -> Result<Programme, Erreur> {
    let programme = holo::lire(source)?;
    blocs::verifier_blocs(&programme)?;
    styles::verifier_styles(&programme)?;
    etat::verifier_etat(&programme)?;
    regles::verifier_regles(&programme)?;
    vue::reglages(&programme)?;
    // Ce que l'affichage refuserait (une adresse en `javascript:`, une image hors du dossier)
    // est refusé dès la vérification : on fabrique la page à blanc (revue Codex, B-11).
    if programme.racine.nom == "Page" {
        plat::page_html(&programme, "")?;
    }
    Ok(programme)
}

/// La vue à plat d'un fichier `.holo` : une page web ordinaire, fabriquée par le moteur.
pub fn vue_a_plat(source: &str, base: &str) -> Result<String, Erreur> {
    vue_a_plat_de(source, base, "")
}

/// La vue à plat d'un site du fichier : sa page (chemin vide), ou le monde d'un de ses points
/// (`Shop/Secret`), ouvert en grand comme une page.
pub fn vue_a_plat_de(source: &str, base: &str, chemin: &str) -> Result<String, Erreur> {
    let programme = verifier_page(source)?;
    let site = regles::site_de(&programme, chemin)?;
    plat::site_html(&programme, site, base, chemin.rsplit('/').next().unwrap_or(""))
}

/// Les effets que les règles du fichier demandent pour un signal, comme `Open.tap`.
pub fn effets(source: &str, signal: &str) -> Vec<String> {
    verifier_page(source).map(|programme| regles::effets(&programme, signal)).unwrap_or_default()
}

/// L'état entier, tel qu'il voyage entre le moteur et la page : les nombres, ce que le moteur
/// calcule, puis les textes. `cart=2;count=2;total=240;buyer='Ada`.
fn ecrire_tout(programme: &Programme, nombres: &etat::Etat, textes: &etat::Textes) -> String {
    // Les sons demandés par une règle de temps ou une règle qui guette suivent l'état, sous le
    // nom « ! » : ce n'est pas une valeur, la page le lit et le retire.
    let capacites = etat::capacites_demandees();
    let sons = if capacites.is_empty() { String::new() } else { format!("!={}", capacites.join(",")) };
    [etat::ecrire(&etat::a_montrer(programme, nombres)), etat::ecrire_textes(textes), sons].into_iter().filter(|morceau| !morceau.is_empty()).collect::<Vec<_>>().join(";")
}

/// Les valeurs d'une page à leur départ, écrites `cart=0;likes=3`, suivies de celles que le
/// moteur calcule quand la page donne des prix (`count`, `total`), puis des textes.
pub fn etat_initial(source: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    etat::capacites_demandees();
    ecrire_tout(&programme, &etat::initial(&programme).unwrap_or_default(), &etat::textes_initiaux(&programme))
}

/// L'arbitre : ce que deviennent les valeurs d'une page quand un signal est émis. L'état
/// reçu est relu avec méfiance : rien n'y passe que la page ne déclare.
pub fn arbitrer(source: &str, etat: &str, signal: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    etat::capacites_demandees();
    ecrire_tout(&programme, &etat::arbitrer(&programme, &etat::relire(&programme, etat), signal), &etat::relire_textes(&programme, etat))
}

/// Les horloges d'une page, une par règle `Every` : son rythme en millisecondes et la valeur
/// qu'elle fait changer. `1000:time;2000:star_x`.
pub fn horloges(source: &str) -> String {
    verifier_page(source).map(|programme| etat::horloges(&programme).iter().map(|(ms, valeur)| format!("{ms}:{valeur}")).collect::<Vec<_>>().join(";")).unwrap_or_default()
}

/// Les fichiers qu'une page importe (`commun.holo;pied.holo`), pour qu'on aille les chercher et
/// qu'on les joigne à son texte avant de le donner au moteur.
pub fn imports(source: &str) -> String {
    holo::imports_de(source).map(|noms| noms.join(";")).unwrap_or_default()
}

/// Les touches du clavier que la page écoute (`left;right`).
pub fn touches(source: &str) -> String {
    verifier_page(source).map(|programme| etat::touches(&programme).join(";")).unwrap_or_default()
}

/// Les valeurs qu'un signal fait changer (`time;score`) : leurs horloges repartent de zéro.
pub fn touchees(source: &str, signal: &str) -> String {
    verifier_page(source).map(|programme| etat::touchees(&programme, signal).join(";")).unwrap_or_default()
}

/// Le visiteur a écrit dans un champ ou coché une case : l'arbitre rend le nouvel état.
pub fn saisir(source: &str, etat: &str, nom: &str, ecrit: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    let (nombres, textes) = (etat::relire(&programme, etat), etat::relire_textes(&programme, etat));
    if textes.iter().any(|(connu, _)| connu == nom) {
        ecrire_tout(&programme, &nombres, &etat::saisir_texte(&programme, &textes, nom, ecrit))
    } else {
        ecrire_tout(&programme, &etat::saisir(&programme, &nombres, nom, ecrit), &textes)
    }
}

/// D'où viennent les données de la page, et à quel rythme : `stock.json|30000` (0 : une seule
/// fois). Vide si la page n'en demande pas.
pub fn donnees(source: &str) -> String {
    verifier_page(source).ok().and_then(|programme| etat::source_de_donnees(&programme).ok().flatten()).map(|(fichier, rythme)| format!("{fichier}|{rythme}")).unwrap_or_default()
}

/// Les données viennent d'arriver du serveur : l'arbitre les range et rend le nouvel état.
pub fn recevoir(source: &str, etat: &str, json: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    let (nombres, textes) = etat::recevoir(&programme, &etat::relire(&programme, etat), &etat::relire_textes(&programme, etat), json);
    ecrire_tout(&programme, &nombres, &textes)
}

/// Le visiteur fait glisser un bloc d'un plateau : l'arbitre rend le nouvel état.
pub fn glisser(source: &str, etat: &str, nom: &str, x: u32, y: u32) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    ecrire_tout(&programme, &etat::glisser(&programme, &etat::relire(&programme, etat), nom, u64::from(x), u64::from(y)), &etat::relire_textes(&programme, etat))
}

/// Ce que la page garde d'une visite à l'autre (`keep:`), tiré de cet état : `cart=2;buyer='Ada`.
pub fn a_garder(source: &str, etat: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    let gardees = etat::gardees(&programme).unwrap_or_default();
    let nombres: etat::Etat = etat::relire(&programme, etat).into_iter().filter(|(nom, _)| gardees.contains(nom)).collect();
    let textes: etat::Textes = etat::relire_textes(&programme, etat).into_iter().filter(|(nom, _)| gardees.contains(nom)).collect();
    [etat::ecrire(&nombres), etat::ecrire_textes(&textes)].into_iter().filter(|morceau| !morceau.is_empty()).collect::<Vec<_>>().join(";")
}

/// L'état de départ d'une page, avec ce qu'elle avait gardé d'une visite précédente.
pub fn reprendre(source: &str, garde: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    let gardees = etat::gardees(&programme).unwrap_or_default();
    let relus = etat::relire_textes(&programme, garde);
    // Seuls les textes que la page dit garder sont repris ; les autres partent de leur départ.
    let textes: etat::Textes = etat::textes_initiaux(&programme).into_iter().map(|(nom, depart)| {
        let repris = relus.iter().find(|(connu, _)| *connu == nom && gardees.contains(&nom)).map(|(_, texte)| texte.clone());
        (nom, repris.unwrap_or(depart))
    }).collect();
    ecrire_tout(&programme, &etat::reprendre(&programme, garde), &textes)
}

/// Les conditions d'une page (`If`), avec leur réponse pour cet état : `count|is=0:1;total|over=299:0`.
pub fn conditions(source: &str, etat: &str) -> String {
    match verifier_page(source) {
        Ok(programme) => {
            let montrees = etat::avec_textes(&etat::a_montrer(&programme, &etat::relire(&programme, etat)), &etat::relire_textes(&programme, etat));
            etat::conditions(&programme, &montrees).iter().map(|(cle, vraie)| format!("{cle}:{}", u8::from(*vraie))).collect::<Vec<_>>().join(";")
        }
        Err(_) => String::new(),
    }
}

/// Le fichier `.holo` d'un seul point de la page, pour ouvrir sa vue en profondeur.
pub fn source_du_point(source: &str, nom: &str) -> Option<String> {
    let programme = verifier_page(source).ok()?;
    let point = regles::bloc_nomme(&programme, nom).filter(|bloc| bloc.nom == "Point")?;
    let ecrire = |valeur: &Valeur| match valeur {
        Valeur::Nom(n) => Some(n.clone()),
        Valeur::Entier(e) => Some(e.to_string()),
        Valeur::Nombre { valeur, unite: None } => Some(valeur.to_string()),
        Valeur::Texte(t) => Some(format!("\"{t}\"")),
        Valeur::Liste(elements) => {
            let textes: Option<Vec<String>> = elements
                .iter()
                .map(|e| match e {
                    Valeur::Texte(t) => Some(format!("\"{t}\"")),
                    _ => None,
                })
                .collect();
            textes.map(|t| format!("[{}]", t.join(", ")))
        }
        _ => None,
    };
    let reglages: Vec<String> = ["name", "seed", "brightness", "fragments", "color", "palette"]
        .iter()
        .filter_map(|param| Some(format!("{param}: {}", ecrire(&point.argument(param)?.valeur)?)))
        .collect();
    Some(format!("Point({})", reglages.join(", ")))
}

/// Les mondes voisins d'un site, calculés à partir d'une graine : ils remplissent le carrefour
/// autour des sites écrits dans le fichier. La graine vient du nom du site, pour que le même
/// fichier montre toujours les mêmes mondes (ADR-008). Rend, pour chacun, sa graine et sa couleur.
pub fn mondes_voisins(source: &str, chemin: &str, nombre: u32) -> Vec<(u64, [f32; 3])> {
    let Ok(programme) = verifier_page(source) else { return Vec::new() };
    let nom = chemin.rsplit('/').next().filter(|n| !n.is_empty()).or_else(|| regles::nom_de(&programme.racine)).unwrap_or("Home");
    let graine = nom.bytes().fold(0u64, |g, octet| graine::melanger(g ^ u64::from(octet)));
    (0..nombre)
        .map(|i| {
            let voisin = graine::graine_enfant(graine, i);
            (voisin, univers::Monde::depuis_graine(voisin).enfants[0].couleur)
        })
        .collect()
}

/// Le monde où la page est posée quand on la regarde en personnage. Provisoire : tant que
/// le langage ne sait pas écrire « un monde qui contient une page », la graine de ce monde
/// se tire du nom de la page, pour que le même fichier redonne le même lieu (ADR-008).
pub fn monde_d_accueil(source: &str) -> Option<String> {
    let programme = verifier_page(source).ok()?;
    let nom = regles::nom_de(&programme.racine).unwrap_or("Home");
    let graine = nom.bytes().fold(0u64, |g, octet| graine::melanger(g ^ u64::from(octet)));
    // Yocthan : pas de points décoratifs autour de la page. Le lieu est éteint ; ce sont les
    // éléments de la page eux-mêmes qui deviendront des points (voir `mosaique.rs`).
    Some(format!("Point(name: {nom}, seed: {graine}, brightness: 0, fragments: 12)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOUTIQUE: &str = include_str!("../../exemples/boutique-comparee/boutique.holo");

    #[test]
    fn la_boutique_passe_de_la_page_au_point() {
        assert!(vue_a_plat(BOUTIQUE, "").unwrap().contains("<h1 class=\"holo-H1\">My shop</h1>"));
        assert_eq!(effets(BOUTIQUE, "Open.tap"), ["Workshop.enter"]);
        let point = source_du_point(BOUTIQUE, "Workshop").unwrap();
        assert_eq!(point, "Point(name: Workshop, seed: 42, brightness: 0.8, fragments: 6, color: \"#E9B44C\", palette: [\"#E9B44C\", \"#245C45\"])");
        // Ce point seul est un fichier que la vue en profondeur accepte.
        let decl = verifier(&point).unwrap();
        assert_eq!((decl.graine, decl.morceler, decl.palette.len()), (42, 6, 2));
        assert_eq!(source_du_point(BOUTIQUE, "Open"), None);
    }

    #[test]
    fn le_monde_d_accueil_est_toujours_le_meme() {
        let accueil = monde_d_accueil(BOUTIQUE).unwrap();
        assert_eq!(Some(accueil.clone()), monde_d_accueil(BOUTIQUE));
        let decl = verifier(&accueil).unwrap();
        assert_eq!((decl.nom.as_str(), decl.morceler), ("Shop", 12));
        // Une autre page a un autre lieu.
        assert_ne!(monde_d_accueil("Page(name: Blog)").unwrap(), accueil);
        assert!(monde_d_accueil("Page(children: [ Div() ])").is_none());
    }

    #[test]
    fn les_exemples_du_guide_sont_acceptes_par_le_moteur() {
        let guide = include_str!("../../docs/01-holocode/GUIDE.md");
        let exemples: Vec<&str> = guide.split("```holo
").skip(1).map(|suite| suite.split("```").next().unwrap()).collect();
        assert!(exemples.len() >= 11, "le guide a perdu ses exemples : {}", exemples.len());
        for exemple in exemples {
            // Une page passe toutes les vérifications et se fabrique ; un point seul s'ouvre en
            // profondeur ; un morceau (un fichier fait pour être importé) est vérifié sans être affiché.
            let debut = exemple.trim_start();
            let resultat = if debut.starts_with("Point(") {
                verifier(exemple).map(|_| ())
            } else if debut.starts_with("Part(") {
                verifier_page(exemple).map(|_| ())
            } else {
                vue_a_plat(exemple, "").map(|_| ())
            };
            if let Err(erreur) = resultat {
                panic!("un exemple du guide est refusé : {erreur}
{exemple}");
            }
        }
    }

    #[test]
    fn chaque_lecon_est_acceptee_par_le_moteur() {
        // Une leçon par notion, dans exemples/lecons/ : chacune doit marcher telle qu'elle est écrite.
        let dossier = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../exemples/lecons");
        let mut lecons = 0;
        for entree in std::fs::read_dir(&dossier).unwrap() {
            let chemin = entree.unwrap().path();
            if chemin.extension().and_then(|e| e.to_str()) != Some("holo") {
                continue;
            }
            let mut source = std::fs::read_to_string(&chemin).unwrap();
            // Comme le fait la page d'entrée : les fichiers importés sont joints au texte.
            for nom in imports(&source).split(';').filter(|n| !n.is_empty()).map(str::to_string).collect::<Vec<_>>() {
                source.push(holo::FICHIER_SUIVANT);
                source.push_str(&nom);
                source.push(holo::SEPARE_LE_NOM);
                source.push_str(&std::fs::read_to_string(dossier.join(&nom)).unwrap());
            }
            let debut = source.lines().find(|l| !l.trim().is_empty() && !l.trim_start().starts_with("//") && !l.starts_with("import")).unwrap_or("");
            let resultat = if debut.starts_with("Point(") {
                verifier(&source).map(|_| ())
            } else if debut.starts_with("Part(") {
                verifier_page(&source).map(|_| ())
            } else {
                vue_a_plat(&source, "").map(|_| ())
            };
            if let Err(erreur) = resultat {
                panic!("la leçon {} est refusée : {erreur}", chemin.display());
            }
            lecons += 1;
        }
        assert!(lecons >= 27, "des leçons ont disparu : {lecons}");
    }

    #[test]
    fn les_mondes_voisins_sont_toujours_les_memes() {
        let voisins = mondes_voisins(BOUTIQUE, "", 9);
        assert_eq!(voisins.len(), 9);
        assert_eq!(voisins, mondes_voisins(BOUTIQUE, "", 9));
        let graines: std::collections::HashSet<u64> = voisins.iter().map(|(g, _)| *g).collect();
        assert_eq!(graines.len(), 9, "neuf mondes différents");
        // Un autre site a d'autres voisins ; un fichier refusé n'en a pas.
        assert_ne!(mondes_voisins(BOUTIQUE, "Workshop", 9), voisins);
        assert!(mondes_voisins("Page(children: [ Div() ])", "", 9).is_empty());
        // Chacun est un monde que la vue en profondeur sait ouvrir.
        assert!(verifier(&format!("Point(name: World, seed: {}, fragments: 12)", voisins[0].0)).is_ok());
    }

    #[test]
    fn la_maison_et_son_jardin_sont_deux_fichiers_valables() {
        for fichier in [include_str!("../../exemples/maison/salon.holo"), include_str!("../../exemples/maison/jardin.holo")] {
            vue_a_plat(fichier, "/exemples/maison/").unwrap();
        }
        // Le salon a une porte vers le jardin : toucher le point demande d'y entrer.
        assert_eq!(effets(include_str!("../../exemples/maison/salon.holo"), "Jardin.tap"), ["Jardin.enter"]);
    }

    #[test]
    fn la_verification_refuse_ce_que_l_affichage_refuserait() {
        assert!(verifier_page("Page(children: [ A(\"x\", to: \"javascript:alert(1)\") ])").unwrap_err().message.contains("« to »"));
        assert!(verifier_page("Page(children: [ Image(source: \"../secret.png\") ])").is_err());
        assert!(verifier_page("Page(children: [ Point(name: G, seed: 1, inside: \"http://192.168.1.1/x.holo\") ])").is_err());
        assert!(verifier_page("Page(children: [ A(\"x\", to: \"garden.holo\") ])").is_ok());
    }

    #[test]
    fn un_fichier_refuse_ne_donne_ni_page_ni_effet() {
        let casse = BOUTIQUE.replace("Workshop.enter", "Workshop.fly");
        assert!(vue_a_plat(&casse, "").unwrap_err().message.contains("capacité inconnue"));
        assert!(effets(&casse, "Open.tap").is_empty());
    }
}
