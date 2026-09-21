# Registre des décisions

Ce registre empêche une proposition séduisante de devenir accidentellement une « vérité officielle ».

| ID | Décision | Statut | Sources | Projets affectés |
|---|---|---|---|---|
| `ADR-001` | Utiliser un référentiel Git privé comme source de vérité documentaire | `ACCEPTÉ` | `HC-009` | Tous |
| `ADR-002` | Séparer discussions, décisions, spécifications et implémentations | `ACCEPTÉ` | `HC-009` | Gouvernance |
| `ADR-003` | Considérer le monde, l'espace, les relations, lois et phénomènes comme primitives candidates | `PROPOSITION` | `HC-002`, `HC-003` | HoloCode |
| `ADR-004` | Explorer les archétypes et capacités plutôt que l'héritage de classes comme mécanisme central | `PROPOSITION` | `HC-003`, `HC-004` | HoloCode, HoloIR |
| `ADR-005` | Faire fonctionner la première version sur le matériel existant | `PROPOSITION` | `HC-005`, `HC-007` | Tous les prototypes |
| `ADR-006` | Préserver l'information spatiale et temporelle dans HoloIR | `PROPOSITION` | `HC-007` | HoloCompiler, HoloIR |
| [`ADR-007`](adr/ADR-007-web-mis-a-jour-deux-vues.md) | Le métavers est une mise à jour du web : une seule description, deux vues (à plat, en profondeur) | `ACCEPTÉ` | `HC-013` | Tous |
| [`ADR-008`](adr/ADR-008-fichier-vraie-source.md) | Le fichier est une vraie source, toujours lue de la même façon ; l'IA aide à écrire, jamais à lire | `ACCEPTÉ` | `HC-013` | HoloCode, HoloCompiler, HoloRuntime |
| [`ADR-009`](adr/ADR-009-format-holo.md) | Format `.holo` : blocs nommés par leur sens, à la Flutter ; texte en Markdown dans les blocs ; ni HTML, ni CSS, ni JavaScript pour l'auteur | `ACCEPTÉ` | `HC-013` | HoloCode, HoloCompiler |
| [`ADR-010`](adr/ADR-010-moteur-rust-deux-enveloppes.md) | Un moteur écrit en Rust, sous les navigateurs actuels (WebAssembly) puis dans un navigateur propre | `ACCEPTÉ` | `HC-013`, `HC-007` | HoloRuntime, HoloEngine, HoloCode-Core |
| [`ADR-011`](adr/ADR-011-rendu-par-vue.md) | Vue à plat par génération de HTML et CSS, vue en profondeur par le moteur | `PROPOSITION` | `HC-013` | HoloCompiler, HoloEngine |
| [`ADR-012`](adr/ADR-012-ponts-javascript-css.md) | Première version : des ponts vers JavaScript et CSS seulement, comme outils de transition | `ACCEPTÉ` | `HC-013` | HoloCode, HoloCompiler |
| [`ADR-013`](adr/ADR-013-deux-etages-trois-imports.md) | Deux étages (HoloCode, modules WebAssembly enfermés) et trois sortes d'import (`import`, `module`, `pont`) | `PROPOSITION` | `HC-013`, `HC-006` | HoloCode, HoloCode-Core, HoloRuntime |
| [`ADR-014`](adr/ADR-014-place-de-l-ia.md) | L'IA agit à la création et passe le vérificateur ; le hasard passe par des graines | `PROPOSITION` | `HC-013` | HoloCode, outils de création |
| [`ADR-015`](adr/ADR-015-regle-des-appels.md) | Tout changement d'état passe par un arbitre ; pas de code libre dans un bloc ; description honnête du paradigme | `PROPOSITION` | `HC-013`, `HC-003` | HoloCode, HoloRuntime |

Les décisions `ADR-007` à `ADR-015` ont chacune une fiche détaillée dans [`adr/`](adr/). Celles marquées `ACCEPTÉ` ont été tranchées par Yocthan le 2026-09-21 ; celles marquées `PROPOSITION` viennent de Claude et attendent sa validation.

## Autorité de validation

Le responsable du projet est **Yocthan Mabeka**. Une IA peut proposer, tester, critiquer ou recommander une décision ; elle ne peut pas convertir seule une proposition en décision officielle.

## Dette décisionnelle actuelle

- Définir l'identité et le cycle de vie d'une entité.
- Choisir le modèle de composition des archétypes.
- Définir la causalité, l'atomicité et les conflits des phénomènes.
- Définir la portée, la priorité et la résolution des lois.
- Choisir la première stratégie d'exécution de HoloIR.
- Choisir le langage d'implémentation du prototype : traité par `ADR-010` pour le moteur (Rust).
- Définir comment chaque bloc se présente dans la vue en profondeur (`ADR-007`).
- Faire converger la syntaxe `.holo` des prototypes des PR n° 1 et n° 2 avec le format en blocs (`ADR-009`).
