# Registre des décisions

Ce registre empêche une proposition séduisante de devenir accidentellement une « vérité officielle ».

| ID | Décision | Statut | Sources | Projets affectés |
|---|---|---|---|---|
| `ADR-001` | Utiliser un référentiel Git privé comme source de vérité documentaire | `ACCEPTÉ` | `HC-009` | Tous |
| `ADR-002` | Séparer discussions, décisions, spécifications et implémentations | `ACCEPTÉ` | `HC-009` | Gouvernance |
| [`ADR-003`](adr/ADR-003-briques-du-langage.md) | Le monde, l'espace, les relations, les lois et les phénomènes sont les briques du langage pour décrire ce qui arrive ; description honnête : des objets sans méthodes, des règles du monde, des relations | `ACCEPTÉ` | `HC-002`, `HC-003`, `HC-013` | HoloCode |
| [`ADR-004`](adr/ADR-004-composition-et-capacites.md) | Des archétypes composés et des capacités, plutôt que l'héritage de classes | `ACCEPTÉ` | `HC-003`, `HC-004`, `HC-013` | HoloCode, HoloIR |
| [`ADR-005`](adr/ADR-005-materiel-existant.md) | La première version tourne sur le matériel existant : tout téléphone actuel, dans un navigateur, premier test sous 1 Go | `ACCEPTÉ` | `HC-005`, `HC-007`, `HC-013` | Tous les prototypes |
| [`ADR-006`](adr/ADR-006-holoir.md) | Préserver l'information spatiale et temporelle dans HoloIR (bonne idée, prématurée : à trancher quand le moteur existera) | `PROPOSITION` | `HC-007`, `HC-013` | HoloCompiler, HoloIR |
| [`ADR-007`](adr/ADR-007-web-mis-a-jour-deux-vues.md) | Le métavers est une mise à jour du web : une seule description, deux vues (à plat, en profondeur) | `ACCEPTÉ` | `HC-013` | Tous |
| [`ADR-008`](adr/ADR-008-fichier-vraie-source.md) | Le fichier est une vraie source, toujours lue de la même façon ; l'IA aide à écrire, jamais à lire | `ACCEPTÉ` | `HC-013` | HoloCode, HoloCompiler, HoloRuntime |
| [`ADR-009`](adr/ADR-009-format-holo.md) | Format `.holo` : blocs nommés par leur sens, à la Flutter ; texte en Markdown dans les blocs ; ni HTML, ni CSS, ni JavaScript pour l'auteur | `ACCEPTÉ` | `HC-013` | HoloCode, HoloCompiler |
| [`ADR-010`](adr/ADR-010-moteur-rust-deux-enveloppes.md) | Un moteur écrit en Rust, sous les navigateurs actuels (WebAssembly) puis dans un navigateur propre | `ACCEPTÉ` (mesuré sur téléphone le 2026-10-04) | `HC-013`, `HC-007` | HoloRuntime, HoloEngine, HoloCode-Core |
| [`ADR-011`](adr/ADR-011-rendu-par-vue.md) | Vue à plat par génération de HTML et CSS, vue en profondeur par le moteur | `EXPÉRIMENTATION` | `HC-013` | HoloCompiler, HoloEngine |
| [`ADR-012`](adr/ADR-012-ponts-javascript-css.md) | Première version : des ponts vers JavaScript et CSS seulement, comme outils de transition | `EXPÉRIMENTATION` | `HC-013` | HoloCode, HoloCompiler |
| [`ADR-013`](adr/ADR-013-deux-etages-trois-imports.md) | Deux étages (HoloCode, modules WebAssembly enfermés) et trois sortes d'import (`import`, `module`, `pont`) | `EXPÉRIMENTATION` | `HC-013`, `HC-006` | HoloCode, HoloCode-Core, HoloRuntime |
| [`ADR-014`](adr/ADR-014-place-de-l-ia.md) | Un monde se lit sans IA ; l'IA crée du `.holo` qui passe le vérificateur, et peut agir en direct comme acteur extérieur, par des capacités journalisées ; le hasard passe par des graines | `ACCEPTÉ` | `HC-013` | HoloCode, outils de création |
| [`ADR-015`](adr/ADR-015-regle-des-appels.md) | Tout changement d'état passe par un arbitre ; pas de code libre dans un bloc ; description honnête du paradigme | `ACCEPTÉ` | `HC-013`, `HC-003` | HoloCode, HoloRuntime |
| [`ADR-016`](adr/ADR-016-vocabulaire-anglais.md) | Le vocabulaire du langage est en anglais ; un mot que les programmeurs connaissent garde son sens (`fragments`, `leave`, `On`, `brightness`, `children`) | `ACCEPTÉ` | `HC-013`, revue Codex | HoloCode, suite de conformité |
| [`ADR-017`](adr/ADR-017-forme-et-couleurs.md) | La forme : les styles s'écrivent comme en CSS (`P { … }`, `.card { … }`) après le bloc racine ; sept règles pour qu'ils servent aussi aux jeux (apparence seulement, deux façons de viser, tout est vérifié) ; couleur d'un point par la graine, par `color` ou par `palette` | `ACCEPTÉ` | `HC-013`, revue Codex, journal du 2026-10-03 | HoloCode, HoloEngine |
| [`ADR-018`](adr/ADR-018-vue-en-profondeur.md) | En profondeur, seuls les points ont de la profondeur ; le reste se lit sur un panneau (option A ; l'option B sera montrée) | `ACCEPTÉ` | `HC-013` | HoloEngine |
| [`ADR-019`](adr/ADR-019-texte-nu-et-import-md.md) | Le texte s'écrit sans artifice, comme « Hello World » ; un `.md` ne s'importe que pour un long texte | `ACCEPTÉ` (écriture exacte proposée) | `HC-013` | HoloCode |
| [`ADR-020`](adr/ADR-020-text-p-et-titres.md) | `Text` est le texte de base ; `P` et `H1` à `H3` sont un `Text` avec un rôle ; les titres ne sautent pas de niveau ; une seule écriture par mot (bloc en majuscule, réglage en minuscules) | `ACCEPTÉ` | journal du 2026-10-03 | HoloCode, HoloCompiler, suite de conformité |
| [`ADR-021`](adr/ADR-021-reglages-de-vue.md) | La façon dont une page se regarde s'écrit dans le fichier : `Zoom` (limites), `Points` (quand un pixel devient un point, comment il se morcelle), `Relief` ; chaque réglage a des bornes | `ACCEPTÉ` (écriture proposée) | journal du 2026-10-03 | HoloCode, HoloEngine |
| [`ADR-022`](adr/ADR-022-liens-et-passages.md) | Deux façons d'aller ailleurs : `A` (le lien classique, on change de page) et le `Point` dont le monde est un autre fichier (on traverse sans changer de page) ; dézoomer fait ressortir ; `List(ordered:)` | `ACCEPTÉ` (écriture proposée) | journal du 2026-10-03 | HoloCode, HoloEngine |
| [`ADR-023`](adr/ADR-023-etat-arbitre.md) | Les valeurs d'une page : `State(cart: 0)`, `{cart}` dans un texte, et trois demandes faites au moteur (`add`, `sub`, `set`) ; pas de code libre | `ACCEPTÉ` pour l'instant (écriture à revoir avec les noms) | journal du 2026-10-04 | HoloCode, HoloEngine |
| [`ADR-024`](adr/ADR-024-disposition.md) | La disposition vient des blocs : `Row`, `Column`, `Grid`, avec `gap`, `align`, `columns` ; rien ne déborde, le téléphone est géré par le moteur | `ACCEPTÉ` pour l'instant (écriture à revoir avec les noms) | journal du 2026-10-04 | HoloCode, HoloEngine |
| [`ADR-025`](adr/ADR-025-conditions-et-texte.md) | Les conditions : `If(valeur, is:, not:, over:, under:)`, en mots et sans « sinon » ; `Hr`, `Quote`, `Code`, le retour à la ligne, `Image(alt:)` | `EXPÉRIMENTATION` | journal du 2026-10-04 | HoloCode, HoloEngine |
| [`ADR-026`](adr/ADR-026-temps-hasard-plateau.md) | Un premier jeu : le temps (`Every(1s, effect:)`), le hasard rejouable (`random`), le plateau (`Board`, `x`, `y`) ; toujours sans code libre | `ACCEPTÉ` pour l'instant (écriture à revoir avec les noms) | journal du 2026-10-04 | HoloCode, HoloEngine |
| [`ADR-027`](adr/ADR-027-saisie-et-valeurs-gardees.md) | La saisie : `Input` (un nombre) et `Checkbox`, liés à une valeur par `value:`, étiquette obligatoire ; `keep: [noms]` garde des valeurs dans le navigateur du visiteur | `EXPÉRIMENTATION` | journal du 2026-10-04 | HoloCode, HoloEngine |
| [`ADR-028`](adr/ADR-028-clavier-regles-qui-guettent-rencontres.md) | Le clavier (`On(Key.left, …)`), les règles qui guettent (`When`), y compris la rencontre de deux blocs (`When(A, meets: B)`), le glissement (`drag`), plusieurs demandes par règle ; une place reste sur le plateau | `EXPÉRIMENTATION` | journal du 2026-10-04 | HoloCode, HoloEngine |

`ADR-003` à `ADR-006` ont été proposées par ChatGPT. Le 2026-09-21, Yocthan a accepté `ADR-003` (reformulée par Claude), `ADR-004` (telle quelle) et `ADR-005` (complétée par les chiffres de sa vision), et a laissé `ADR-006` en proposition. ChatGPT est invité à réagir aux reformulations.

Les décisions `ADR-003` à `ADR-015` ont chacune une fiche détaillée dans [`adr/`](adr/). Toutes ont été validées par Yocthan le 2026-09-21 : `ADR-007`, `008`, `009`, `010` et `012` sont ses propres décisions ; `ADR-011`, `013`, `014` et `015` ont été proposées par Claude, puis acceptées par Yocthan après lecture.

Après la revue de ChatGPT, Yocthan a classé en `EXPÉRIMENTATION` les choix techniques qu'aucune mesure n'a encore confirmés : `ADR-010`, `011`, `012` et `013`. La direction est retenue et le travail commence dans ce sens ; c'est la mesure sur un vrai téléphone qui les fera passer en `ACCEPTÉ`. `ADR-014` a été reformulée par ChatGPT.

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
