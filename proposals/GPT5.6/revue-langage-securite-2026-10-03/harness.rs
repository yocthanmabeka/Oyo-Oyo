#[cfg(test)]
mod tests {
    use holo_moteur::{verifier_page, vue_a_plat};

    const INJECTION: &str = include_str!("hostiles/01-injection-texte.holo");
    const JAVASCRIPT: &str = include_str!("hostiles/02-lien-javascript.holo");
    const ZOOM: &str = include_str!("hostiles/03-zoom-max-contourne.holo");
    const PORTAILS: &str = include_str!("hostiles/04-portals-count-contourne.holo");
    const BUDGET: &str = include_str!("hostiles/05-budget-declaratif.holo");
    const ABOVE: &str = include_str!("hostiles/06-above-hors-portee.holo");
    const DISTANT: &str = include_str!("hostiles/07-passage-distant.holo");

    #[test]
    fn le_texte_hostile_est_echappe() {
        let html = vue_a_plat(INJECTION, "").expect("le texte est valide mais doit être échappé");
        assert!(!html.contains("<script>"), "un script brut a atteint la sortie HTML");
        assert!(!html.contains("<img src=x onerror"), "un gestionnaire d'événement a atteint la sortie HTML");
        assert!(html.contains("&lt;script&gt;alert"), "le texte hostile n'est pas visible sous forme échappée");
    }

    #[test]
    fn la_verification_accepte_javascript_mais_le_rendu_le_refuse() {
        verifier_page(JAVASCRIPT)
            .expect("constat CI : verifier_page accepte actuellement le schéma javascript:");
        let erreur = vue_a_plat(JAVASCRIPT, "")
            .expect_err("le rendu doit refuser une adresse javascript:");
        assert!(erreur.message.contains("to"), "diagnostic inattendu : {erreur}");
    }

    #[test]
    fn zoom_max_et_after_sont_acceptes_dans_un_ordre_incoherent() {
        let programme = verifier_page(ZOOM).expect("la combinaison est actuellement acceptée");
        let r = holo_moteur::vue::reglages(&programme).expect("les réglages sont actuellement acceptés");
        assert_eq!(r.zoom_max, 1.0);
        assert_eq!(r.apres, 16.0);
        assert!(r.apres > r.zoom_max, "la sonde exige un seuil points supérieur au maximum annoncé");
    }

    #[test]
    fn portals_count_ne_borne_pas_les_points_ecrits() {
        let programme = verifier_page(PORTAILS).expect("la page est actuellement acceptée");
        let r = holo_moteur::vue::reglages(&programme).expect("les réglages sont actuellement acceptés");
        let html = vue_a_plat(PORTAILS, "").expect("la page est actuellement rendue");
        assert_eq!(r.portails_nombre, 1);
        assert_eq!(html.matches("class=\"holo-Point\"").count(), 3);
    }

    #[test]
    fn le_budget_fait_confiance_au_poids_declare() {
        verifier_page(BUDGET).expect("le moteur ne compare pas weight au fichier réel");
    }

    #[test]
    fn above_peut_designer_un_bloc_hors_de_la_page_visible() {
        let html = vue_a_plat(ABOVE, "").expect("le repère imbriqué est actuellement accepté");
        assert!(html.contains("data-above=\"Inner\""));
        assert!(html.contains("data-name=\"Inner\""));
    }

    #[test]
    fn une_adresse_distante_http_est_acceptee() {
        let html = vue_a_plat(DISTANT, "").expect("les adresses http distantes sont actuellement acceptées");
        assert!(html.contains("data-file=\"http://127.0.0.1:8081/tracker.holo\""));
    }
}
