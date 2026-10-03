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
- **Le langage HoloCode** : des blocs nommés par leur sens, imbriqués comme en Flutter, avec le texte en Markdown dans les blocs (`ADR-009`). Le vocabulaire est **en anglais**, et un mot que les programmeurs connaissent garde son sens (`ADR-016`) : `Page`, `Text`, `P`, `H1`, `Button`, `Point`, `World`, `On`, `name`, `seed`, `children`, `rules`. `Text` est du texte sans rôle ; `P` et `H1` à `H3` sont un `Text` avec un rôle ; un bloc commence par une majuscule, un réglage par une minuscule (`ADR-020`). Une phrase entre guillemets est un paragraphe (`ADR-019`). La façon dont une page se regarde (limites du zoom, pixels qui deviennent des points, relief) s'écrit dans la page : `Zoom`, `Points`, `Relief` (`ADR-021`). À chaque ajout au langage : comparer les options, et vérifier qu'on ne répète pas un défaut de HTML, de CSS ou de JavaScript. La forme : des styles écrits comme en CSS après le bloc racine (`P { … }`, `.card { … }`, posé par `P.card(...)`), qui ne disent que l'apparence, jamais la disposition, et où tout est vérifié (`ADR-017`). L'auteur n'écrit jamais de HTML, de CSS ni de JavaScript. Tout changement d'état passe par un arbitre ; pas de code libre dans un bloc (`ADR-015`). Description honnête du paradigme : des objets sans méthodes, des règles au niveau du monde, et des relations. Rien de nouveau dans les briques ; la valeur est dans ce qui est interdit.
- **Le moteur** : en Rust, compilé en WebAssembly pour les navigateurs d'aujourd'hui, en natif demain pour un navigateur propre au projet (`ADR-010`, en expérimentation). Première réalisation : [`moteur/`](moteur/README.md), le sprint Big Bang.
- **Le `Point`** est le pixel de l'Holoverse : la plus petite unité visible, qui révèle un monde quand on zoome dessus (`ADR-016`).
- **La règle de fusion** : on ne fusionne dans `main` que ce qui marche. Celui qui fusionne vérifie l'auteur et la branche, jamais seulement le numéro, et a lu la pull request en entier. Claude fusionne ses propres pull requests quand les tests sont verts ; une pull request d'une autre IA, ou qui change une décision, attend l'accord de Yocthan. Les tests s'exécutent automatiquement sur chaque pull request (`.github/workflows/tests.yml`) ; un README qui annonce un résultat ne vaut rien, seul le test exécuté compte.

## Comment contribuer

- Travaille sur une branche, présente une pull request. Ne pousse jamais dans `main`.
- Range ton code dans `proposals/<ton nom>/` ; les expériences du projet sont dans `experiments/`.
- Chaque contribution suit le format du protocole : sujet, discussions sources `HC-xxx`, décisions concernées `ADR-xxx`, statut proposé, résultat, objections et limites, expérience ou preuve requise, documents à mettre à jour.
- Compare toute idée à ce qui existe déjà. Cite tes sources. N'invente ni URL ni contenu de fichier que tu n'as pas lu.
- Si tu ne peux pas exécuter de code, dis-le, et n'écris jamais qu'un test « passe » : Claude l'exécutera sur la machine de Yocthan et rapportera le résultat réel.
- Ne change jamais le statut d'une décision : seul Yocthan valide. Propose `EXPLORATION`, `PROPOSITION` ou `EXPÉRIMENTATION`.

## Comment nous joindre

Le canal entre les IA est **GitHub**, décidé par Yocthan le 2026-10-03 : simple et tracé.

- **Si tu as accès au dépôt** (ChatGPT par son connecteur Codex, Gemini par Gemini Code Assist) : commente les pull requests, ou ouvre une *issue* intitulée « Revue <ton nom> du <date> ». Claude lit ces commentaires et répond au même endroit. Une proposition de code se fait par une pull request vers `proposals/<ton nom>/`.
- **Si tu n'as pas accès** : Yocthan te transmet un fichier unique préparé par Claude, qui rassemble les documents utiles. Réponds en mettant chaque fichier dans un bloc de code, entre une ligne `>>>>> FICHIER : chemin` et une ligne `<<<<< FIN : chemin`, pour que Claude puisse le placer dans le dépôt sans rien retaper.
- Les autres IA ne peuvent pas appeler Claude directement ; Claude ne peut pas les appeler non plus. Tout passe par le dépôt, ou par Yocthan.
- Les **issues** sont la boîte aux lettres : une tâche ou une revue par issue (modèle « Tâche »), avec les chemins réservés et la preuve exigée ; étiquettes `ia:*`, `etat:*`, `zone:*`. Une IA ne modifie pas des chemins déjà réservés par une tâche en cours. Les fichiers transversaux (`AGENTS.md`, le journal, les ADR, `.github/`) sont tenus par Claude.
- Les commentaires d'une IA sur GitHub sont des **avis**, jamais des décisions.
- Pour savoir où en est le projet avant d'écrire : [`docs/06-journal/JOURNAL.md`](docs/06-journal/JOURNAL.md).

## Où l'aide est la bienvenue

- **Le langage** : la grammaire du format en blocs (brouillon dans [`experiments/conformite-v0.1/README.md`](experiments/conformite-v0.1/README.md)), les blocs d'une page (`Page`, `Text`, `P`, `H1`, `Button`, `Image`, `List`), la façon d'écrire un site entier en `.holo`, les messages d'erreur. C'est le chantier ouvert.
- **Les cas de conformité** : des fichiers `.holo` avec leur résultat attendu, que tout moteur devra passer.
- **Images et textures** définies par des formules plutôt que stockées (quelques octets, pas des mégaoctets).
- **Le son** : comment un monde sonne, sans fichiers lourds.
- **La 3D** : des formes définies par des formules (champs de distance signés), inspirées des modificateurs de Blender, et leur coût sur un téléphone.
- **La mesure** : protocoles reproductibles pour la fluidité, la mémoire, la batterie sur téléphone.

## État au 2026-10-03

- 18 pull requests, `main` au vert avec cinq tâches de test. Codex (ChatGPT) a livré sa revue dans `proposals/GPT5.6/revue-2026-10-03/` ; trois défauts qu'il a relevés sont corrigés dans le moteur (graines exactes, imports refusés, test figé).
- Décisions : `ADR-001` à `ADR-022` ; dix-sept acceptées, quatre en expérimentation (`ADR-010` à `ADR-013`), une en proposition (`ADR-006`).
- Trois prototypes Python dans `proposals/` (ChatGPT 8 tests, Claude 27, Gemini 8), des brouillons des règles.
- Le sprint Big Bang est livré dans `moteur/` : 502 Ko transférés (Ko = 1 000 octets), 19 tests. On touche une boule pour la viser et y entrer ; un bouton met le monde en pause. **Les mesures sur le téléphone de Yocthan restent à faire** ; elles décident des décisions en expérimentation.
- Exemple de référence : [`exemples/boutique-comparee/`](exemples/boutique-comparee/README.md), la même boutique en HoloCode et en HTML, CSS, JavaScript, avec tout le vocabulaire.
- Chantier en cours : le langage. Le moteur affiche une page (`moteur/web/page.html`, vue à plat fabriquée en HTML et CSS) fait entrer dans un point, avec le contenu du monde sur un panneau, offre une « vue personnage » où la même page est une feuille posée dans un monde, et une « vue points » (Ctrl + molette ou pincer sur la page) où chaque pixel de la page devient un point, qui se morcelle au zoom et prend du relief quand on tourne la page ; prochaine étape, une boutique lisible dans les deux vues (`Page`, `Text`, `P`, `H1`, `Button`, `On`, et des styles).

## Outillage

Décidé le 2026-10-03 sur la proposition de Codex (`proposals/GPT5.6/outillage-et-coordination-2026-10/`) : aucun pont direct entre IA ; aucun serveur MCP installé pour l'instant. La mesure sur téléphone se fera par câble USB avec `adb` et `chrome://inspect`. Chrome DevTools MCP seulement à la demande, avec un profil Chrome de test vide. Ce qui manque au projet, par priorité : voir `MANQUES.md` dans le même dossier.

Pour apprendre à écrire du `.holo` : le guide de l'auteur, [`docs/01-holocode/GUIDE.md`](docs/01-holocode/GUIDE.md), dont les exemples sont relus par un test du moteur. Pour écrire du `.holo` dans VS Code : l'extension [`outils/vscode-holocode/`](outils/vscode-holocode/README.md) (couleurs du langage, bouton pour ouvrir le fichier dans le navigateur).

## Équipe

- **Yocthan Mabeka** : responsable, seul à valider. Dicte souvent à la voix ; vient de Flutter ; demande qu'on discute avant de construire.
- **Claude** : programmeur principal, revue critique, tient ce fichier et le journal.
- **ChatGPT** : synthèse, formalisation, gouvernance du dépôt (il l'a créé).
- **Gemini** : recherche comparative, exploration ; sans accès à GitHub.
