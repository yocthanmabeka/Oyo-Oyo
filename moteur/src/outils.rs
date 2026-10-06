//! Les outils de l'auteur (ADR-054) : remettre un fichier en forme, et jouer des essais écrits.
//!
//! ```text
//! holo fmt page.holo                  écrit le fichier remis en forme (deux espaces par niveau)
//! holo essai page.holo page.essai     joue les gestes écrits, vérifie les valeurs attendues
//! ```
//!
//! Un fichier d'essai, une ligne par geste ou par vérification :
//!
//! ```text
//! // Ajouter deux fois le tableau « Night » remplit le panier.
//! tap AddNight
//! tap AddNight
//! expect cart = 12000
//! type name "Forest"
//! expect name = "Forest"
//! expect articles = 4
//! receive {"articles": []}
//! ```
//!
//! `tap Nom` touche un bouton (le signal `Nom.tap`) ; `signal Nom.hover` envoie un autre signal ;
//! `type valeur "texte"` écrit dans un champ ; `receive {…}` fait comme si le serveur envoyait ces
//! données ; `expect valeur = …` vérifie un nombre, un texte entre guillemets, ou le nombre
//! d'éléments d'une liste. Les essais utilisent l'arbitre de la page : le même code que dans le
//! navigateur.

/// Remet un fichier `.holo` en forme, comme les leçons du dépôt : deux espaces de plus après
/// une ligne qui ouvre (une parenthèse, un crochet, une accolade, ou plusieurs à la fois), deux
/// de moins quand ce qu'elle a ouvert se referme ; les espaces de fin de ligne retirés ; jamais
/// plus d'une ligne vide de suite. Rien d'autre ne change : ni les mots, ni l'ordre, ni les
/// commentaires, ni le contenu d'un texte, même sur plusieurs lignes.
pub fn mettre_en_forme(source: &str) -> String {
    let mut sortie = String::with_capacity(source.len());
    // Chaque niveau ouvert : combien de signes ouvrants il attend encore qu'on referme.
    let mut niveaux: Vec<usize> = Vec::new();
    let mut dans_un_long_texte = false;
    let mut lignes_vides = 0;
    for ligne in source.lines() {
        let ligne = ligne.trim_end_matches('\r');
        if dans_un_long_texte {
            // Le contenu d'un texte long est gardé tel quel, à la lettre.
            sortie.push_str(ligne);
            sortie.push('\n');
            let mut signes = Vec::new();
            dans_un_long_texte = parcourir(ligne, true, &mut |c| signes.push(c));
            fermer_et_ouvrir(&mut niveaux, &signes);
            continue;
        }
        let propre = ligne.trim();
        if propre.is_empty() {
            lignes_vides += 1;
            if lignes_vides <= 1 && !sortie.is_empty() {
                sortie.push('\n');
            }
            continue;
        }
        lignes_vides = 0;
        let mut signes = Vec::new();
        dans_un_long_texte = parcourir(propre, false, &mut |c| signes.push(c));
        let signes: Vec<char> = signes.into_iter().filter(|c| "()[]{}".contains(*c)).collect();
        // Une ligne qui commence par des fermetures se range au niveau de ce qu'elles referment.
        let en_tete = propre.chars().take_while(|c| matches!(c, ')' | ']' | '}')).count();
        // Un niveau entamé par ces fermetures, même sans être refermé tout à fait, se range aussi.
        let mut restants = niveaux.clone();
        let mut touches = 0;
        for _ in 0..en_tete {
            let rang = restants.len();
            let Some(dernier) = restants.last_mut() else { break };
            if *dernier == niveaux.get(rang - 1).copied().unwrap_or(0) {
                touches += 1;
            }
            *dernier -= 1;
            if *dernier == 0 {
                restants.pop();
            }
        }
        let retrait = niveaux.len().saturating_sub(touches) * 2;
        // Un commentaire seul, aligné plus loin par l'auteur (la suite d'un commentaire de fin de
        // ligne), garde sa place.
        let deja = ligne.len() - ligne.trim_start().len();
        if propre.starts_with("//") && deja > retrait {
            sortie.push_str(&ligne[..deja]);
        } else {
            sortie.push_str(&" ".repeat(retrait));
        }
        sortie.push_str(propre);
        sortie.push('\n');
        fermer_et_ouvrir(&mut niveaux, &signes);
    }
    while sortie.ends_with("\n\n") {
        sortie.pop();
    }
    sortie
}

/// Ferme puis ouvre les niveaux d'une ligne : ce qu'une ligne ouvre et referme elle-même ne
/// compte pas ; ce qu'elle laisse ouvert fait un seul niveau, quel qu'en soit le nombre.
fn fermer_et_ouvrir(niveaux: &mut Vec<usize>, signes: &[char]) {
    let mut ouverts_ici = 0usize;
    for c in signes {
        match c {
            '(' | '[' | '{' => ouverts_ici += 1,
            ')' | ']' | '}' if ouverts_ici > 0 => ouverts_ici -= 1,
            ')' | ']' | '}' => {
                if let Some(dernier) = niveaux.last_mut() {
                    *dernier -= 1;
                    if *dernier == 0 {
                        niveaux.pop();
                    }
                }
            }
            _ => {}
        }
    }
    if ouverts_ici > 0 {
        niveaux.push(ouverts_ici);
    }
}

/// Parcourt les signes d'une ligne qui sont du code (ni texte, ni commentaire) ; rend vrai si la
/// ligne se termine à l'intérieur d'un texte long (`"""`).
fn parcourir(ligne: &str, deja_dans_un_long_texte: bool, f: &mut dyn FnMut(char)) -> bool {
    let signes: Vec<char> = ligne.chars().collect();
    let mut i = 0;
    let mut long = deja_dans_un_long_texte;
    let mut court = false;
    while i < signes.len() {
        let triple = signes.get(i..i + 3).is_some_and(|t| t.iter().all(|c| *c == '"'));
        if long {
            if triple {
                long = false;
                i += 3;
            } else {
                i += 1;
            }
            continue;
        }
        if court {
            match signes[i] {
                '\\' => i += 2,
                '"' => {
                    court = false;
                    i += 1;
                }
                _ => i += 1,
            }
            continue;
        }
        if triple {
            long = true;
            i += 3;
            continue;
        }
        match signes[i] {
            '"' => court = true,
            '/' if signes.get(i + 1) == Some(&'/') => break,
            c => f(c),
        }
        i += 1;
    }
    long
}

/// Le résultat d'un essai : chaque ligne jouée, ou la première qui échoue.
#[derive(Debug, PartialEq)]
pub struct Essai {
    pub reussies: usize,
    pub echec: Option<(usize, String)>,
}

/// Joue un fichier d'essai sur une page (voir le haut de ce fichier).
pub fn jouer(source: &str, essai: &str) -> Result<Essai, crate::holo::Erreur> {
    let programme = crate::verifier_page(source)?;
    let mut etat = crate::etat_initial(source);
    let mut reussies = 0;
    for (rang, ligne) in essai.lines().enumerate() {
        let numero = rang + 1;
        let ligne = ligne.trim();
        if ligne.is_empty() || ligne.starts_with("//") {
            continue;
        }
        let (verbe, reste) = ligne.split_once(' ').map_or((ligne, ""), |(v, r)| (v, r.trim()));
        let echec = |message: String| Ok(Essai { reussies, echec: Some((numero, message)) });
        match verbe {
            "tap" if !reste.is_empty() => etat = crate::arbitrer(source, &etat, &format!("{reste}.tap")),
            "signal" if reste.contains('.') => etat = crate::arbitrer(source, &etat, reste),
            "type" => {
                let Some((nom, texte)) = reste.split_once(' ') else { return echec("« type » s'écrit : type name \"Forest\"".into()) };
                let texte = texte.trim();
                let Some(texte) = texte.strip_prefix('"').and_then(|t| t.strip_suffix('"')) else { return echec("le texte de « type » s'écrit entre guillemets : type name \"Forest\"".into()) };
                etat = crate::saisir(source, &etat, nom, texte);
            }
            "receive" => etat = crate::recevoir(source, &etat, reste),
            "expect" => {
                let Some((nom, attendu)) = reste.split_once('=') else { return echec("« expect » s'écrit : expect cart = 12000".into()) };
                let (nom, attendu) = (nom.trim(), attendu.trim());
                let Some(vu) = valeur_dans(&programme, &etat, nom) else { return echec(format!("la page n'a pas de valeur « {nom} »")) };
                let attendu_lisible = attendu.strip_prefix('"').and_then(|t| t.strip_suffix('"')).unwrap_or(attendu);
                if vu != attendu_lisible {
                    return echec(format!("« {nom} » vaut {}, et l'essai attendait {attendu}", if attendu.starts_with('"') { format!("\"{vu}\"") } else { vu }));
                }
            }
            _ => return echec(format!("ligne inconnue « {ligne} » ; on écrit : tap, signal, type, receive, expect")),
        }
        reussies += 1;
    }
    Ok(Essai { reussies, echec: None })
}

/// Ce que vaut une valeur dans un état écrit : un nombre, un texte décodé, ou le nombre
/// d'éléments d'une liste.
fn valeur_dans(programme: &crate::holo::Programme, etat: &str, nom: &str) -> Option<String> {
    let _ = programme;
    etat.split(';').find_map(|morceau| {
        let (n, v) = morceau.split_once('=')?;
        if n != nom {
            return None;
        }
        Some(if let Some(texte) = v.strip_prefix('\'') {
            crate::etat::decoder(texte).unwrap_or_default()
        } else if let Some(liste) = v.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            liste.split(',').filter(|e| !e.is_empty()).count().to_string()
        } else {
            v.to_string()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_mise_en_forme_ne_change_que_les_blancs() {
        let source = "Page(\ntitle: \"A (b\",   \n  children: [\n        H1(\"x\"), // un commentaire (\n P(\"\"\"\n   garde\n mes  espaces\n\"\"\"),\n\n\n\n Row(children: [\nText(\"]\")\n])\n],\n)\n\nP { color: red; }\n";
        let attendu = "Page(\n  title: \"A (b\",\n  children: [\n    H1(\"x\"), // un commentaire (\n    P(\"\"\"\n   garde\n mes  espaces\n\"\"\"),\n\n    Row(children: [\n      Text(\"]\")\n    ])\n  ],\n)\n\nP { color: red; }\n";
        assert_eq!(mettre_en_forme(source), attendu);
        // Deux fois de suite, le même résultat ; et le moteur lit la même page.
        assert_eq!(mettre_en_forme(attendu), attendu);
        for lecon in ["68-liste-qui-change.holo", "70-composants.holo", "43-texte-long.holo"] {
            let texte = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../exemples/lecons").join(lecon)).unwrap();
            let forme = mettre_en_forme(&texte);
            assert_eq!(crate::vue_a_plat(&forme, "").unwrap(), crate::vue_a_plat(&texte, "").unwrap(), "{lecon}");
            assert_eq!(mettre_en_forme(&forme), forme, "{lecon}");
        }
    }

    #[test]
    fn un_essai_ecrit_joue_les_gestes_et_verifie_les_valeurs() {
        let page = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../exemples/lecons/70-composants.holo")).unwrap();
        let essai = "// deux tableaux Night\ntap AddNight\ntap AddNight\nexpect night = 2\nexpect cart = 12000\n";
        assert_eq!(jouer(&page, essai).unwrap(), Essai { reussies: 4, echec: None });
        let rate = jouer(&page, "tap AddNight\nexpect cart = 1\n").unwrap();
        assert_eq!(rate.echec, Some((2, "« cart » vaut 6000, et l'essai attendait 1".into())));
        assert!(jouer(&page, "dance\n").unwrap().echec.unwrap().1.contains("ligne inconnue"));
        let liste = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../exemples/lecons/68-liste-qui-change.holo")).unwrap();
        let essai = "type tache \"Pain\"\ntap Ajouter\nexpect taches = 2\nexpect tache = \"\"\ntap Vider\nexpect taches = 0\n";
        assert_eq!(jouer(&liste, essai).unwrap().echec, None);
    }

    #[test]
    fn chaque_essai_des_exemples_passe() {
        // Un fichier `x.essai` à côté de `x.holo` : il doit passer, comme les leçons.
        let dossier = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../exemples/lecons");
        let mut joues = 0;
        for entree in std::fs::read_dir(&dossier).unwrap() {
            let chemin = entree.unwrap().path();
            if chemin.extension().and_then(|e| e.to_str()) != Some("essai") {
                continue;
            }
            let page = std::fs::read_to_string(chemin.with_extension("holo")).unwrap();
            let resultat = jouer(&page, &std::fs::read_to_string(&chemin).unwrap()).unwrap();
            assert_eq!(resultat.echec, None, "{}", chemin.display());
            joues += 1;
        }
        assert!(joues >= 2, "des essais ont disparu : {joues}");
    }
}
