# Pour toute IA ou tout outil branché sur ce projet

Tu arrives sur **Holoverse / HoloCode**, le projet de Yocthan Mabeka : réinventer l'usage du web sous la forme d'un métavers léger. Ce fichier te dit où tu es, ce qui est décidé, comment aider et ce qu'il ne faut pas faire. Il est tenu à jour par Claude ; la date de son dernier état figure plus bas.

## Lis dans cet ordre

1. [`README.md`](README.md) : la carte du dépôt.
2. [`docs/02-gouvernance/PROTOCOLE-IA.md`](docs/02-gouvernance/PROTOCOLE-IA.md) : les règles de travail entre IA, et le format de remise d'une contribution.
3. [`docs/02-gouvernance/DECISIONS.md`](docs/02-gouvernance/DECISIONS.md) : ce qui est décidé, et avec quel statut.
4. [`docs/06-journal/JOURNAL.md`](docs/06-journal/JOURNAL.md) : ce qui s'est passé, étape par étape, avec les captures et les erreurs.
5. [`docs/00-vision/VISION.md`](docs/00-vision/VISION.md) : la vision de Yocthan.

Si tu n'as pas accès à GitHub (c'est le cas de Gemini), demande à Yocthan le fichier unique qui rassemble ces documents : Claude sait le préparer.

## Ce que tu dois savoir en une minute

- **L'idée de Yocthan** : le métavers n'est pas un jeu, c'est une mise à jour du web. Un même fichier `.holo` s'affiche à plat (une page ordinaire) et en profondeur (un lieu où l'on zoome et où l'on entre). Tout part d'un point lumineux qui se morcelle en points, chacun contenant un monde. Un monde naît de sa graine, rien n'est stocké. Cible : n'importe quel téléphone actuel, dans un navigateur, premier test sous 1 Go.
- **Le langage HoloCode** : des blocs nommés par leur sens, imbriqués comme en Flutter, avec le texte en Markdown dans les blocs (`ADR-009`). L'auteur n'écrit jamais de HTML, de CSS ni de JavaScript. Tout changement d'état passe par un arbitre ; pas de code libre dans un bloc (`ADR-015`). Description honnête du paradigme : des objets sans méthodes, des règles au niveau du monde, et des relations. Rien de nouveau dans les briques ; la valeur est dans ce qui est interdit.
- **Le moteur** : en Rust, compilé en WebAssembly pour les navigateurs d'aujourd'hui, en natif demain pour un navigateur propre au projet (`ADR-010`, en expérimentation). Première réalisation : [`moteur/`](moteur/README.md), le sprint Big Bang.
- **La règle de fusion** : on ne fusionne dans `main` que ce qui marche. Les tests s'exécutent automatiquement sur chaque pull request (`.github/workflows/tests.yml`) ; un README qui annonce un résultat ne vaut rien, seul le test exécuté compte.

## Comment contribuer

- Travaille sur une branche, présente une pull request. Ne pousse jamais dans `main`.
- Range ton code dans `proposals/<ton nom>/` ; les expériences du projet sont dans `experiments/`.
- Chaque contribution suit le format du protocole : sujet, discussions sources `HC-xxx`, décisions concernées `ADR-xxx`, statut proposé, résultat, objections et limites, expérience ou preuve requise, documents à mettre à jour.
- Compare toute idée à ce qui existe déjà. Cite tes sources. N'invente ni URL ni contenu de fichier que tu n'as pas lu.
- Si tu ne peux pas exécuter de code, dis-le, et n'écris jamais qu'un test « passe » : Claude l'exécutera sur la machine de Yocthan et rapportera le résultat réel.
- Ne change jamais le statut d'une décision : seul Yocthan valide. Propose `EXPLORATION`, `PROPOSITION` ou `EXPÉRIMENTATION`.

## Où l'aide est la bienvenue

- **Le langage** : la grammaire du format en blocs (brouillon dans [`experiments/conformite-v0.1/README.md`](experiments/conformite-v0.1/README.md)), les blocs d'une page (`Page`, `Texte`, `Bouton`, `Image`, `Liste`), la façon d'écrire un site entier en `.holo`, les messages d'erreur. C'est le chantier ouvert.
- **Les cas de conformité** : des fichiers `.holo` avec leur résultat attendu, que tout moteur devra passer.
- **Images et textures** définies par des formules plutôt que stockées (quelques octets, pas des mégaoctets).
- **Le son** : comment un monde sonne, sans fichiers lourds.
- **La 3D** : des formes définies par des formules (champs de distance signés), inspirées des modificateurs de Blender, et leur coût sur un téléphone.
- **La mesure** : protocoles reproductibles pour la fluidité, la mémoire, la batterie sur téléphone.

## État au 2026-10-03

- 12 pull requests, 11 fusionnées, `main` au vert avec cinq tâches de test.
- Décisions : `ADR-001` à `ADR-015` ; dix acceptées, quatre en expérimentation (`ADR-010` à `ADR-013`), une en proposition (`ADR-006`).
- Trois prototypes Python dans `proposals/` (ChatGPT 8 tests, Claude 27, Gemini 8), des brouillons des règles.
- Le sprint Big Bang est livré dans `moteur/` : 489 Ko transférés, 18 tests. On touche une boule pour la viser et y entrer. **Les mesures sur le téléphone de Yocthan restent à faire** ; elles décident des décisions en expérimentation.
- Prochain chantier annoncé par Yocthan : le langage, ce que les humains écriront chaque jour.

## Équipe

- **Yocthan Mabeka** : responsable, seul à valider. Dicte souvent à la voix ; vient de Flutter ; demande qu'on discute avant de construire.
- **Claude** : programmeur principal, revue critique, tient ce fichier et le journal.
- **ChatGPT** : synthèse, formalisation, gouvernance du dépôt (il l'a créé).
- **Gemini** : recherche comparative, exploration ; sans accès à GitHub.
