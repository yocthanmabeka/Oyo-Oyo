# Prompt pour Codex — les refus de HoloCode (2026-10-06)

À copier dans Codex, qui a accès au dépôt `yocthanmabeka/Metaverse`.

---

Bonjour Codex. Claude a écrit pourquoi HoloCode refuse certains éléments du web : [`proposals/Claude/pourquoi-ces-refus-2026-10/README.md`](../../../proposals/Claude/pourquoi-ces-refus-2026-10/README.md). Le grand tableau des 129 éléments comparés est dans [`docs/01-holocode/TABLEAU-WEB.md`](../../01-holocode/TABLEAU-WEB.md). Yocthan décidera, après ton avis et celui de Gemini, s'il faut admettre certains éléments refusés.

Ce qu'on te demande, avec la même rigueur que tes revues précédentes (ce qui est vérifié, ce qui est supposé) :

1. **Relis chaque refus contre les fiches citées** (`ADR-009`, `ADR-015`, `ADR-017`, `ADR-020`) : Claude dit-il juste sur qui a décidé et ce que la fiche dit ? Signale toute affirmation que le dépôt ne soutient pas.
2. **Juge chaque refus** : garder, assouplir ou supprimer, avec un exemple concret de site qu'un refus rendrait impossible. Une sonde exécutable est bienvenue si elle tranche une question.
3. **Le tableau** : vérifie quelques lignes au hasard contre le moteur (`moteur/src/`), en particulier les pourcentages et les « Oui » ; signale les erreurs.
4. **Les repères pour lecteurs d'écran** et **les états d'un bouton** (survol, focus, appui) : propose une écriture, sans rouvrir la cascade du CSS ni la contradiction avec `ADR-016`.
5. **Un refus manquant** : un défaut du web que HoloCode répète encore ?

Travaille sur une branche et présente tes conclusions dans une pull request, comme d'habitude : un dossier dans `proposals/GPT5.6/`. Ne change aucun statut de décision.
