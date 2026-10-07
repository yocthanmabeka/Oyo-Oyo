#[cfg(test)]
mod tests {
    use holo_engine::{check_page, flat_view};

    const INJECTION: &str = include_str!("hostiles/01-injection-texte.holo");
    const JAVASCRIPT: &str = include_str!("hostiles/02-lien-javascript.holo");
    const ZOOM: &str = include_str!("hostiles/03-zoom-max-contourne.holo");
    const PORTAILS: &str = include_str!("hostiles/04-portals-count-contourne.holo");
    const BUDGET: &str = include_str!("hostiles/05-budget-declaratif.holo");
    const ABOVE: &str = include_str!("hostiles/06-above-hors-portee.holo");
    const DISTANT: &str = include_str!("hostiles/07-passage-distant.holo");

    #[test]
    fn le_texte_hostile_est_echappe() {
        let html = flat_view(INJECTION, "").expect("le texte est valide mais doit être échappé");
        assert!(!html.contains("<script>"), "un script brut a atteint la sortie HTML");
        assert!(!html.contains("<img src=x onerror"), "un gestionnaire d'événement a atteint la sortie HTML");
        assert!(html.contains("&lt;script&gt;alert"), "le texte hostile n'est pas visible sous forme échappée");
    }

    // Les trois sondes marquées « corrigé » décrivaient des défauts constatés par cette revue.
    // Claude les a corrigés le 2026-10-04 et a retourné ici l'attente, avec l'accord de Yocthan :
    // elles vérifient maintenant que le défaut ne revient pas.

    #[test]
    fn corrige_la_verification_refuse_javascript_comme_le_rendu() {
        check_page(JAVASCRIPT).expect_err("B-11 corrigé : check_page refuse le schéma javascript:");
        let erreur = flat_view(JAVASCRIPT, "")
            .expect_err("le rendu doit refuser une adresse javascript:");
        assert!(erreur.message.contains("to"), "diagnostic inattendu : {erreur}");
    }

    #[test]
    fn corrige_zoom_max_et_after_incoherents_sont_refuses() {
        let erreur = check_page(ZOOM).expect_err("B-01 corrigé : Points(after:) ne peut pas dépasser Zoom(max:)");
        assert!(erreur.message.contains("Zoom(max: 1)"), "diagnostic inattendu : {erreur}");
    }

    #[test]
    fn portals_count_ne_borne_pas_les_points_ecrits() {
        let programme = check_page(PORTAILS).expect("la page est actuellement acceptée");
        let r = holo_engine::view::settings(&programme).expect("les réglages sont actuellement acceptés");
        let html = flat_view(PORTAILS, "").expect("la page est actuellement rendue");
        assert_eq!(r.portals_count, 1);
        assert_eq!(html.matches("class=\"holo-Point\"").count(), 3);
    }

    #[test]
    fn le_budget_fait_confiance_au_poids_declare() {
        check_page(BUDGET).expect("le moteur ne compare pas weight au fichier réel");
    }

    #[test]
    fn corrige_above_hors_de_la_page_visible_est_refuse() {
        let erreur = flat_view(ABOVE, "").expect_err("B-06 corrigé : le repère doit être dans le même site");
        assert!(erreur.message.contains("above"), "diagnostic inattendu : {erreur}");
    }

    #[test]
    fn une_adresse_distante_http_est_acceptee() {
        let html = flat_view(DISTANT, "").expect("les adresses http distantes sont actuellement acceptées");
        assert!(html.contains("data-file=\"http://127.0.0.1:8081/tracker.holo\""));
    }
}
