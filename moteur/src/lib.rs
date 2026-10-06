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
pub mod composants;
pub mod etat;
pub mod format;
pub mod graine;
pub mod holo;
pub mod listes;
pub mod modules;
pub mod mosaique;
pub mod mouvement;
pub mod navigation;
pub mod outils;
pub mod plat;
pub mod regles;
pub mod repetition;
pub mod styles;
pub mod univers;
pub mod vue;

#[cfg(all(target_arch = "wasm32", feature = "dessin"))]
mod rendu;
#[cfg(all(target_arch = "wasm32", feature = "dessin"))]
mod web;
#[cfg(target_arch = "wasm32")]
mod web_page;

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
    // Les modules enfermés, et leur annonce en haut du fichier (ADR-045).
    modules::modules(&programme, &etat::initial(&programme)?)?;
    // Ce que l'affichage refuserait (une adresse en `javascript:`, une image hors du dossier)
    // est refusé dès la vérification : on fabrique la page à blanc (revue Codex, B-11).
    if programme.racine.nom == "Page" {
        plat::page_html(&programme, "")?;
    }
    Ok(programme)
}

/// La page a-t-elle besoin du dessin ? Oui si elle montre des points (`Point`, un monde), si ses
/// pixels deviennent des points au zoom (`points:`, `pixels:`), si elle tourne (`Relief(tilt:)`),
/// ou si c'est un monde seul. Sinon, le moteur léger suffit (ADR-053).
pub fn a_besoin_du_dessin(source: &str) -> bool {
    let Ok(programme) = holo::lire(source) else { return true };
    if programme.racine.nom != "Page" {
        return true;
    }
    if let Ok(r) = vue::reglages(&programme) {
        if r.points_actifs || r.angle_max > 0.0 {
            return true;
        }
    }
    let mut dessin = false;
    let _ = regles::pour_chaque_bloc(&programme.racine, &mut |bloc| {
        if matches!(bloc.nom.as_str(), "Point" | "World") {
            dessin = true;
        }
        Ok(())
    });
    dessin
}

/// L'éditeur (ADR-046) : le fichier est-il juste ? `ok`, ou la première faute, telle que le
/// moteur la refuse : `ligne 7, colonne 5 : « h1 » : … écris « H1 »`. Un point seul (`Point(…)`)
/// se vérifie comme un monde ; un morceau importé (`Part(…)`), pour ses blocs et ses styles :
/// le reste se vérifie dans la page qui l'importe.
pub fn verifier_texte(source: &str) -> String {
    // Un fichier de styles seuls (ADR-052) : ses styles se vérifient comme ceux d'un morceau.
    if holo::lire(source).is_err() {
        let styles = format!("Part(name: HoloStyles, children: []) {source}");
        if let Ok(p) = holo::lire(&styles) {
            if !p.styles.is_empty() {
                return match styles::verifier_styles(&p) {
                    Ok(()) => "ok : un fichier de styles, à importer dans une page".into(),
                    Err(e) => e.to_string(),
                };
            }
        }
    }
    let racine = holo::lire(source).map(|p| p.racine.nom);
    let resultat = match racine.as_deref() {
        Ok("Point") => verifier(source).map(|_| ()),
        Ok("Part") => holo::lire(source).and_then(|p| blocs::verifier_blocs(&p).and_then(|()| styles::verifier_styles(&p))),
        _ => verifier_page(source).map(|_| ()),
    };
    match resultat {
        Ok(()) if racine.as_deref() == Ok("Part") => "ok : un morceau, à vérifier aussi dans la page qui l'importe".into(),
        Ok(()) => "ok".into(),
        Err(erreur) => erreur.to_string(),
    }
}

/// Tous les mots du langage, pour l'éditeur (ADR-046) : il les propose pendant qu'on écrit, pour
/// qu'on les touche au lieu de les taper (une majuscule au milieu d'un mot coûte cher sur un
/// téléphone, ADR-037). En JSON.
pub fn vocabulaire() -> String {
    fn liste(mots: &[&str]) -> String {
        format!("[{}]", mots.iter().map(|m| format!("\"{m}\"")).collect::<Vec<_>>().join(","))
    }
    const MOUVEMENT: &[&str] = &["opacity", "x", "y", "scale", "rotate", "flip", "tilt", "blur", "hue", "round", "at", "for", "ease", "letters", "each"];
    let mut boucle: Vec<&str> = MOUVEMENT.to_vec();
    boucle.push("back");
    let autres: [(&str, &[&str]); 10] = [
        ("Repeat", &["items", "over", "children", "rules"]),
        ("Item", &["key"]),
        ("Data", &["from", "every"]),
        ("Enter", MOUVEMENT),
        ("Loop", &boucle),
        ("Zoom", &["active", "max", "shrink", "levels", "speed"]),
        ("Points", &["after", "size", "fragment", "divisions", "levels", "density"]),
        ("Relief", &["height", "tilt"]),
        ("Portals", &["layout", "count", "size", "brightness", "duration"]),
        ("Font", &["family", "source"]),
    ];
    let mut parametres: Vec<String> = blocs::parametres_des_blocs().iter().map(|(bloc, p)| format!("\"{bloc}\":{}", liste(p))).collect();
    for (bloc, p) in autres {
        if !blocs::parametres_des_blocs().iter().any(|(b, _)| *b == bloc) {
            parametres.push(format!("\"{bloc}\":{}", liste(p)));
        }
    }
    let mut reglages = styles::noms_des_reglages();
    reglages.push("display");
    format!(
        "{{\"blocs\":{},\"parametres\":{{{}}},\"reglages\":{},\"etats\":{},\"demandes\":{},\"signaux\":{},\"capacites\":{},\"touches\":{},\"mots\":{},\"calculees\":{},\"formats\":{}}}",
        liste(blocs::BLOCS),
        parametres.join(","),
        liste(&reglages),
        liste(holo::ETATS),
        liste(&["add", "sub", "set", "random", "mul", "div", "push", "remove", "clear"]),
        liste(&["tap", "hover", "hoverEnd", "sent", "failed", "done"]),
        liste(&["enter", "leave", "play", "portals", "open", "close", "send", "run"]),
        liste(etat::TOUCHES),
        liste(&[
            "true", "false", "item", "circle", "square", "triangle", "diamond", "start", "center", "end", "between", "topLeft", "top", "topRight", "left", "right", "bottomLeft", "bottom",
            "bottomRight", "linear", "smooth", "out", "in", "back", "spring", "bounce", "forever", "grid", "row", "column", "diagonal", "date", "time", "color", "none", "uppercase",
            "lowercase", "capitalize", "underline", "line-through", "bold", "italic", "normal", "solid", "dashed", "dotted",
        ]),
        liste(&["count", "total", "year", "month", "day", "weekday", "hour", "minute"]),
        liste(format::FORMATS),
    )
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
fn ecrire_tout(programme: &Programme, nombres: &etat::Etat, textes: &etat::Textes, listes: &listes::Listes) -> String {
    // Les sons demandés par une règle de temps ou une règle qui guette suivent l'état, sous le
    // nom « ! » : ce n'est pas une valeur, la page le lit et le retire.
    let capacites = etat::capacites_demandees();
    let sons = if capacites.is_empty() { String::new() } else { format!("!={}", capacites.join(",")) };
    [etat::ecrire(&etat::a_montrer(programme, nombres)), etat::ecrire_textes(textes), listes::ecrire(listes), sons].into_iter().filter(|morceau| !morceau.is_empty()).collect::<Vec<_>>().join(";")
}

/// Les valeurs d'une page à leur départ, écrites `cart=0;likes=3`, suivies de celles que le
/// moteur calcule quand la page donne des prix (`count`, `total`), puis des textes.
pub fn etat_initial(source: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    etat::capacites_demandees();
    ecrire_tout(&programme, &etat::initial(&programme).unwrap_or_default(), &etat::textes_initiaux(&programme), &listes::initiales(&programme))
}

/// L'arbitre : ce que deviennent les valeurs d'une page quand un signal est émis. L'état
/// reçu est relu avec méfiance : rien n'y passe que la page ne déclare.
pub fn arbitrer(source: &str, etat: &str, signal: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    etat::capacites_demandees();
    // Un geste d'une ligne (`Done.tap@2`) : les nombres changent comme pour `Done.tap` ; les
    // listes et les textes savent de quelle ligne il vient (ADR-044).
    let base = listes::signal_et_ligne(signal).0;
    let nombres = etat::arbitrer(&programme, &etat::relire(&programme, etat), base);
    let (textes, listes) = listes::arbitrer(&programme, &nombres, &etat::relire_textes(&programme, etat), &listes::relire(&programme, etat), signal);
    ecrire_tout(&programme, &nombres, &textes, &listes)
}

/// Les horloges d'une page, une par règle `Every` : son rythme en millisecondes et la valeur
/// qu'elle fait changer. `1000:time;2000:starX`.
pub fn horloges(source: &str) -> String {
    verifier_page(source).map(|programme| etat::horloges(&programme).iter().map(|(ms, valeur)| format!("{ms}:{valeur}")).collect::<Vec<_>>().join(";")).unwrap_or_default()
}

/// Les attentes d'une page (`After`), et si chacune court pour cet état : `3000:1;5000:0` (ADR-039).
pub fn delais(source: &str, etat: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    etat::delais(&programme, &etat::relire(&programme, etat)).iter().map(|(ms, court)| format!("{ms}:{}", u8::from(*court))).collect::<Vec<_>>().join(";")
}

/// Ce qu'un formulaire de la page envoie au serveur, en JSON (ADR-042). Vide s'il n'existe pas.
pub fn envoi(source: &str, etat: &str, formulaire: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    etat::envoi(&programme, &etat::relire(&programme, etat), &etat::relire_textes(&programme, etat), formulaire).unwrap_or_default()
}

/// Une valeur écrite avec son format (ADR-043) : `formater("minute", 5, "00", "fr")` → `05`.
pub fn formater(nom: &str, valeur: u64, format: &str, langue: &str) -> String {
    format::formater(nom, valeur, format, langue)
}

/// La page lit-elle l'heure du visiteur ? Elle la tient alors à jour, minute après minute.
pub fn lit_l_heure(source: &str) -> bool {
    verifier_page(source).is_ok_and(|programme| etat::lit_l_heure(&programme))
}

/// Donne au moteur l'heure du visiteur : année, mois, jour, jour de la semaine (1 lundi), heure, minute.
pub fn regler_maintenant(valeurs: [u64; 6]) {
    etat::regler_maintenant(valeurs);
}

/// Une minute a passé : l'état avec la nouvelle heure, après les règles qui la guettent.
pub fn avancer_l_horloge(source: &str, etat: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    etat::capacites_demandees();
    ecrire_tout(&programme, &etat::avancer_l_horloge(&programme, etat), &etat::relire_textes(&programme, etat), &listes::relire(&programme, etat))
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

/// Ce que la page doit savoir pour faire tourner un module (ADR-045) : son fichier, le nombre
/// qu'il reçoit (pour cet état), son temps en millisecondes et sa mémoire en pages de 64 Ko.
/// `somme.wasm|10|100|16`. Vide si aucun module ne porte ce nom.
pub fn module_info(source: &str, etat: &str, nom: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    let nombres = etat::relire(&programme, etat);
    let Some(module) = modules::modules(&programme, &nombres).ok().and_then(|m| m.into_iter().find(|m| m.nom == nom)) else { return String::new() };
    let entree = module.entree.and_then(|e| nombres.iter().find(|(c, _)| c == e)).map_or(0, |(_, v)| *v);
    format!("{}|{entree}|{}|{}", module.source, module.temps, module.pages)
}

/// Le module a rendu son nombre : le nouvel état, après `Nom.done`.
pub fn module_fini(source: &str, etat: &str, nom: &str, valeur: u64) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    etat::capacites_demandees();
    ecrire_tout(&programme, &modules::fini(&programme, &etat::relire(&programme, etat), nom, valeur), &etat::relire_textes(&programme, etat), &listes::relire(&programme, etat))
}

/// Les lignes d'une liste pour cet état (ADR-044) : la page les pose à la place des anciennes.
pub fn liste_html(source: &str, base: &str, etat: &str, nom: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    plat::lignes_de_liste(&programme, base, &etat::relire(&programme, etat), &etat::relire_textes(&programme, etat), &listes::relire(&programme, etat), nom)
}

/// Les valeurs qu'un signal fait changer (`time;score`) : leurs horloges repartent de zéro.
pub fn touchees(source: &str, signal: &str) -> String {
    let signal = listes::signal_et_ligne(signal).0;
    verifier_page(source).map(|programme| etat::touchees(&programme, signal).join(";")).unwrap_or_default()
}

/// Le visiteur a écrit dans un champ ou coché une case : l'arbitre rend le nouvel état.
pub fn saisir(source: &str, etat: &str, nom: &str, ecrit: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    etat::capacites_demandees();
    let (nombres, textes) = (etat::relire(&programme, etat), etat::relire_textes(&programme, etat));
    if textes.iter().any(|(connu, _)| connu == nom) {
        ecrire_tout(&programme, &nombres, &etat::saisir_texte(&programme, &textes, nom, ecrit), &listes::relire(&programme, etat))
    } else {
        ecrire_tout(&programme, &etat::saisir(&programme, &nombres, nom, ecrit), &textes, &listes::relire(&programme, etat))
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
    etat::capacites_demandees();
    let (nombres, textes) = etat::recevoir(&programme, &etat::relire(&programme, etat), &etat::relire_textes(&programme, etat), json);
    // Les listes aussi : un tableau de textes, ou d'objets (ADR-051).
    let listes = listes::recevoir(&programme, &listes::relire(&programme, etat), json);
    ecrire_tout(&programme, &nombres, &textes, &listes)
}

/// Le visiteur fait glisser un bloc d'un plateau : l'arbitre rend le nouvel état.
pub fn glisser(source: &str, etat: &str, nom: &str, x: u32, y: u32) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    etat::capacites_demandees();
    ecrire_tout(&programme, &etat::glisser(&programme, &etat::relire(&programme, etat), nom, u64::from(x), u64::from(y)), &etat::relire_textes(&programme, etat), &listes::relire(&programme, etat))
}

/// Ce que la page garde d'une visite à l'autre (`keep:`), tiré de cet état : `cart=2;buyer='Ada`.
pub fn a_garder(source: &str, etat: &str) -> String {
    let Ok(programme) = verifier_page(source) else { return String::new() };
    let gardees = etat::gardees(&programme).unwrap_or_default();
    let nombres: etat::Etat = etat::relire(&programme, etat).into_iter().filter(|(nom, _)| gardees.contains(nom)).collect();
    let textes: etat::Textes = etat::relire_textes(&programme, etat).into_iter().filter(|(nom, _)| gardees.contains(nom)).collect();
    let listes: listes::Listes = listes::relire(&programme, etat).into_iter().filter(|(nom, _)| gardees.contains(nom)).collect();
    [etat::ecrire(&nombres), etat::ecrire_textes(&textes), listes::ecrire(&listes)].into_iter().filter(|morceau| !morceau.is_empty()).collect::<Vec<_>>().join(";")
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
    // Seules les listes que la page dit garder sont reprises (ADR-044).
    let listes: listes::Listes = listes::relire(&programme, garde).into_iter().map(|(nom, elements)| if gardees.contains(&nom) { (nom, elements) } else { (nom.clone(), listes::initiales(&programme).into_iter().find(|(n, _)| *n == nom).map(|(_, e)| e).unwrap_or_default()) }).collect();
    ecrire_tout(&programme, &etat::reprendre(&programme, garde), &textes, &listes)
}

/// Les conditions d'une page (`If`), avec leur réponse pour cet état : `count|is=0:1;total|over=299:0`.
pub fn conditions(source: &str, etat: &str) -> String {
    match verifier_page(source) {
        Ok(programme) => {
            let mut montrees = etat::avec_textes(&etat::a_montrer(&programme, &etat::relire(&programme, etat)), &etat::relire_textes(&programme, etat));
            montrees.extend(listes::comptes(&listes::relire(&programme, etat)));
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
mod editeur {
    #[test]
    fn l_editeur_recoit_la_faute_et_le_vocabulaire() {
        assert_eq!(crate::verifier_texte("Page(children: [ H1(\"a\") ])"), "ok");
        assert_eq!(crate::verifier_texte("Page(children: [ h1(\"a\") ])"), "ligne 1, colonne 18 : « h1 » : un nom de bloc commence par une majuscule, écris « H1 »");
        assert!(crate::verifier_texte("Point(name: A, seed: 1)").starts_with("ok"));
        assert!(crate::verifier_texte("Part(name: Menu, children: [ P(\"x\") ])").starts_with("ok : un morceau"));
        let mots = crate::vocabulaire();
        for attendu in ["\"blocs\":[\"Page\",", "\"Page\":[\"name\",", "\"Repeat\":[\"items\"", "\"hoverEnd\"", "\"topRight\"", "\"letter-spacing\"", "\"dark\"", "\"cents\""] {
            assert!(mots.contains(attendu), "manque {attendu} dans {mots}");
        }
    }
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
        // Un dépôt extrait sous Windows peut avoir des fins de ligne « \r\n » (relevé par Codex).
        let guide = include_str!("../../docs/01-holocode/GUIDE.md").replace("\r\n", "\n");
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
            // Un fichier de styles seuls (ADR-052) : un thème, importé par une autre leçon.
            let verdict = verifier_texte(&source);
            let resultat = if verdict.starts_with("ok : un fichier de styles") {
                Ok(())
            } else if debut.starts_with("Point(") {
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
