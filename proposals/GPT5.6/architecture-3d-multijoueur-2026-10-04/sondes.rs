//! Ces sondes décrivent le comportement actuel. Leur succès n'est pas un feu vert réseau.
#[cfg(test)]
mod tests {
    use holo_moteur::{self as h, etat};
    const SOURCE: &str = include_str!("hostiles/entrees.holo");
    const PANIER: &str = include_str!("../../../exemples/jeu/panier.holo");
    fn nombre(s: &str, nom: &str) -> u64 {
        s.split(';').find_map(|p| p.split_once('=').filter(|(k, _)| *k == nom)
            .and_then(|(_, v)| v.parse().ok())).unwrap()
    }
    fn depart() -> String { h::etat_initial(SOURCE) }

    #[test]
    fn p01_cent_battements_sans_attendre() {
        let mut s = depart();
        for _ in 0..100 { s = h::arbitrer(SOURCE, &s, "every:0"); }
        assert_eq!(nombre(&s, "score"), 100);
        println!("P01 : 100 battements => score=100, sans attente ni horodatage");
    }
    #[test]
    fn p02_etat_entier_falsifie() {
        let s = h::arbitrer(SOURCE, "score=999999999", "Add.tap");
        assert_eq!(nombre(&s, "score"), 1_000_000_000);
        println!("P02 : score déclaré falsifié accepté, puis saturé à 1000000000");
    }
    #[test]
    fn p03_repetition_et_ordre() {
        let a = h::arbitrer(SOURCE, &depart(), "Add.tap");
        assert_eq!(nombre(&h::arbitrer(SOURCE, &a, "Add.tap"), "score"), 2);
        let ar = h::arbitrer(SOURCE, &a, "Reset.tap");
        let ra = h::arbitrer(SOURCE, &h::arbitrer(SOURCE, &depart(), "Reset.tap"), "Add.tap");
        assert_eq!((nombre(&ar, "score"), nombre(&ra, "score")), (0, 1));
        println!("P03 : duplication => 2 ; Add/Reset => 0 ; Reset/Add => 1");
    }
    #[test]
    fn p04_bouton_cache() {
        let s = h::arbitrer(SOURCE, &depart(), "Hidden.tap");
        assert_eq!((nombre(&s, "live"), nombre(&s, "score")), (0, 5));
        println!("P04 : bouton sous If faux, signal direct => score=5");
    }
    #[test]
    fn p05_glissement_teleporte() {
        let s = h::glisser(PANIER, &h::arbitrer(PANIER, &h::etat_initial(PANIER), "Play.tap"), "Basket", 0, 0);
        assert_eq!(nombre(&s, "basket"), 0);
        let s = h::glisser(PANIER, &s, "Basket", 100, 0);
        assert_eq!(nombre(&s, "basket"), 100);
        println!("P05 : basket=0 puis 100 sans délai, sans propriétaire ni vitesse");
    }
    #[test]
    fn p06_ecran_change_le_score() {
        let jouer = |l| h::arbitrer(SOURCE, &format!("{};<={l}", depart()), "Move.tap");
        assert_eq!((nombre(&jouer(360), "score"), nombre(&jouer(640), "score")), (10, 0));
        println!("P06 : mêmes objets/geste, largeur=360 => score10 ; largeur=640 => score0");
    }
    #[test]
    fn p07_largeur_bornee_pas_authentifiee() {
        let p = h::verifier_page(SOURCE).unwrap();
        assert_eq!(etat::relire(&p, "<=1").iter().find(|(k, _)| k == "<").unwrap().1, 120);
        assert_eq!(etat::relire(&p, "<=99999").iter().find(|(k, _)| k == "<").unwrap().1, 2000);
        println!("P07 : largeur bornée 120..2000 ; 360 et 640 restent toutes deux acceptées");
    }
    #[test]
    fn p08_compteur_de_hasard_falsifiable() {
        let p = h::verifier_page(SOURCE).unwrap();
        let s = etat::relire(&p, "~=123456");
        assert_eq!(s.iter().find(|(k, _)| k == "~").unwrap().1, 123456);
        let a = h::arbitrer(SOURCE, &depart(), "every:1");
        assert_eq!(a, h::arbitrer(SOURCE, &depart(), "every:1"));
        assert_eq!(nombre(&h::arbitrer(SOURCE, &format!("{};~=123456", depart()), "every:1"), "~"), 123457);
        println!("P08 : suite rejouable ; compteur injecté=123456 => 123457");
    }
    #[test]
    fn p09_compteur_max_panique_en_debug() {
        let r = std::panic::catch_unwind(|| h::arbitrer(SOURCE,
            &format!("{};~=18446744073709551615", depart()), "every:1"));
        if cfg!(debug_assertions) {
            assert!(r.is_err());
            println!("P09 : débordement capturé, panic debug (pas de serveur lancé)");
        } else {
            assert_eq!(nombre(&r.unwrap(), "~"), 0);
            println!("P09 : compteur reboucle à 0 en release par défaut");
        }
    }
    #[test]
    fn p10_capacites_accumulees_hors_retour() {
        let p = h::verifier_page(SOURCE).unwrap();
        let s = etat::initial(&p).unwrap();
        etat::capacites_demandees();
        assert_eq!(etat::arbitrer(&p, &s, "every:2"), s);
        assert_eq!(etat::arbitrer(&p, &s, "every:2"), s);
        assert_eq!(etat::capacites_demandees(), ["Ding.play", "Ding.play"]);
        assert!(etat::capacites_demandees().is_empty());
        println!("P10 : même état de retour, file extérieure=2 sons puis vide");
    }
    #[test]
    fn p11_capacite_de_la_requete_precedente() {
        let p = h::verifier_page(SOURCE).unwrap();
        etat::capacites_demandees();
        etat::arbitrer(&p, &etat::initial(&p).unwrap(), "every:2");
        let autre = "Page(state: State(n: 0), children: [Input(value: n, label: \"N\")])";
        let s = h::saisir(autre, "n=0", "n", "1");
        assert!(s.contains("!=Ding.play"), "{s}");
        println!("P11 : saisir d'une autre page récupère Ding.play non vidé");
    }
    #[test]
    fn p12_champ_cache_et_borne() {
        let s = h::saisir(SOURCE, &depart(), "field", "999");
        assert_eq!((nombre(&s, "live"), nombre(&s, "field")), (0, 10));
        println!("P12 : champ caché accepté, plafond10 appliqué");
    }
    #[test]
    fn p13_json_peut_remplacer_score_mais_pas_metadonnees() {
        let s = h::recevoir(SOURCE, &depart(), r#"{"score":999,"~":99,"<":120,"unknown":7}"#);
        assert_eq!(nombre(&s, "score"), 999);
        assert!(!s.contains("~=99") && !s.contains("<=120") && !s.contains("unknown"));
        println!("P13 : JSON score999 accepté ; clés inconnues et métadonnées ignorées");
    }
    #[test]
    fn p14_signal_inconnu_et_condition_des_regles() {
        assert_eq!(h::arbitrer(SOURCE, &depart(), "NotAPlayer.tap"), depart());
        let s = h::etat_initial(PANIER);
        assert_eq!(h::arbitrer(PANIER, &s, "every:0"), s);
        println!("P14 : signal inconnu sans effet ; If rules bloque Every avant Play");
    }

    #[test]
    fn p15_within_est_un_carre_pas_un_rayon() {
        let s = "Page(state: State(x: 90, y: 90, score: 0), children: [
          Button(name: Move, text: \"Move\"), Board(children: [
            Shape(name: A, form: square, x: x, y: y),
            Shape(name: B, form: square, x: 50, y: 50)])], rules: [
          On(Move.tap, effect: [x.set(58), y.set(58)]),
          When(A, meets: B, within: 9, effect: score.add(1))])";
        h::verifier_page(s).unwrap();
        for largeur in [360, 640] {
            let e = h::arbitrer(s, &format!("{};<={largeur}", h::etat_initial(s)), "Move.tap");
            assert_eq!(nombre(&e, "score"), 1);
        }
        assert!((8_f64 * 8.0 + 8.0 * 8.0).sqrt() > 9.0);
        println!("P15 : within9 accepte (8,8), hors rayon9, indépendamment de la largeur");
    }
}
