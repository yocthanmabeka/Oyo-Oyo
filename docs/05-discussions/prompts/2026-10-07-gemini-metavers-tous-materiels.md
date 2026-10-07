# Prompt pour Gemini — Holoverse de 1970 à 2026

- Écrit par Claude le 2026-10-07, à la demande de Yocthan (« Envoie-moi la proposition du prompt faite par Codex pour que je l'envoie à Gemini »).
- Codex le demande dans sa proposition (PR #153, `proposals/GPT5.6/metavers-tous-materiels-2026-10-07/README.md`) : un avis indépendant de Gemini. Gemini n'a pas accès au dépôt : la proposition est résumée dans le texte. La réponse est dans `../reponses/2026-10-07-gemini-metavers-tous-materiels.md`.

---

Bonjour. Je m'appelle Yocthan. Je construis Holoverse, un métavers, et HoloCode, son langage, avec l'aide de Claude (Anthropic), de Codex (OpenAI) et de toi, Gemini. Codex a fait une proposition, et il demande ton avis **indépendant**. Je ne suis pas programmeur de métier : **réponds en français simple, et explique chaque mot technique la première fois que tu l'emploies.**

## 1. Holoverse et HoloCode en quelques lignes

- Mon idée : le métavers n'est pas un jeu, c'est une mise à jour du web. Un même fichier `.holo` s'affiche à plat (une page ordinaire, en vrai HTML et CSS) et en profondeur (un lieu où l'on zoome ; chaque point contient un monde).
- L'auteur écrit seulement du `.holo` : des blocs (`Page`, `Text`, `Button`…), des styles comme en CSS, des règles. **Aucun code libre** : un moteur écrit en Rust, compilé en WebAssembly, lit le fichier et fabrique la page. HTML, CSS et JavaScript sont des **sorties** du moteur, jamais des langages que l'auteur doit écrire.
- Un monde naît d'une **graine** (un nombre) : le même fichier donne le même monde, sur toutes les machines.
- Aujourd'hui, la cible est le navigateur d'un téléphone actuel. Une page simple pèse 8 Ko à l'ouverture ; le moteur ne vient que si la page en a besoin.

## 2. Ma question

Peut-on visiter Holoverse **quel que soit le matériel**, d'un ordinateur des années 1970 jusqu'au téléphone de 2026 ?

## 3. La proposition de Codex (résumée fidèlement)

**Son idée centrale** : un monde dont les lieux, les liens et les actions essentielles existent indépendamment de son dessin. On entre dans un point par un zoom, par un lien ou par un choix numéroté ; l'identité du lieu reste la même. Une machine puissante le dessine en profondeur ; une machine modeste le décrit en texte. Ce n'est pas une troisième vue : le texte est une forme de la vue à plat.

**Son tableau, par période** (une possibilité d'architecture, pas un logiciel testé) :

| Période | Forme de visite | HTML / CSS / JavaScript | Travail à prévoir |
|---|---|---|---|
| 1970–1979 | Terminal texte : nom du lieu, description, sorties numérotées ; dessin vectoriel si le matériel le permet | Aucun : le Web n'existe pas | Petit programme natif, données paginées, calcul local ou sur un ordinateur central |
| 1980–1989 | Texte, plan en caractères, points ou contours sur une machine graphique | Aucun Web | Lecteur natif adapté à la machine ; lecture par morceaux |
| 1990–1994 | Pages reliées, illustrations facultatives | HTML simple (titres, listes, liens), ni CSS ni JavaScript | Pages préparées ou serveur compatible ; navigation d'abord |
| 1995–1999 | Même base, formulaires, apparence facultative | Formulaires ; JavaScript dès 1995, CSS1 dès 1996, pris en charge en partie | Réponse fabriquée par le serveur ; aucun script obligatoire ; VRML en option |
| 2000–2009 | Pages utilisables, puis carte et interactions enrichies | HTML, CSS, JavaScript classiques | Garder une navigation complète quand les enrichissements échouent |
| 2010–2019 | Pages et visite graphique progressive | HTML5, CSS, JavaScript ; Canvas et WebGL | Charger le moteur graphique seulement à la demande |
| 2020–2026 | Page, points et objets en profondeur selon le matériel | HTML préparé par le serveur, CSS, petit JavaScript, Rust/WebAssembly, WebGL2 ou WebGPU | Un vrai parcours sans JavaScript pour les actions retenues |

**Son exemple** : un point ouvre une bibliothèque avec un livre et un passage vers un atelier.
- Terminal : « Bibliothèque. 1 : lire le livre. 2 : entrer dans l'atelier. 0 : revenir. »
- HTML simple : un titre, le texte du livre, un lien vers l'atelier.
- Vue en profondeur : une pièce, le livre, et un point qui ouvre l'atelier.
Les trois retrouvent les mêmes identifiants et les mêmes contenus. Emprunter le livre produit la même demande, avec les mêmes droits et le même résultat. Sans moyen d'agir, la visite est en lecture seule, et le dit.

**Ses cinq règles** :
1. Séparer contenu, identité, règles et présentation : moins de détails ne change ni les droits, ni les prix, ni les résultats.
2. Ne charger que le lieu demandé et un voisinage borné ; pas d'animation permanente en mode texte.
3. Garder la graine, la version du générateur, l'ordre des demandes et les règles de calcul ; une machine trop faible reçoit une description préparée, elle ne crée pas un autre monde.
4. Distinguer ce qui se régénère (le décor) et ce qui s'enregistre (messages, achats, droits).
5. Choisir la présentation selon les capacités et la préférence du visiteur, pas selon l'année.

**Ses limites, dites par lui** : « peu importe le matériel » veut dire un accès adapté, pas la même image partout ; le moteur Rust actuel n'est pas démontré sur les machines des années 1970–1980 ; il préfère des descriptions préparées à un second moteur complet ; fabriquer avec les outils de 2026 puis lancer en émulation prouve seulement la compatibilité du résultat ; un jeu de réflexes ne devient pas équivalent avec un menu ; il déconseille la vidéo à distance comme accès minimum, et une 3D obligatoire.

**Sa première expérience** : la bibliothèque, l'atelier, un livre, une action permise et une action interdite, testés avec la même source en terminal, en HTML sans JavaScript et dans la vue actuelle ; mesurer, pour chaque cible, la machine ou l'émulateur, les octets transférés, la mémoire, le délai et le nombre de lieux visitables.

**Son ordre recommandé** : 1. navigation et contenu sans moteur graphique ; 2. une action sans JavaScript ; 3. borner mémoire et chargement ; 4. comparer les états entre les rendus ; 5. essayer une cible ancienne précise. Finir la partie web actuelle reste prioritaire.

## 4. Ce que je te demande

Propose **ta propre réponse, indépendante** : ne recopie pas Codex. Tu peux être d'accord, en désaccord, ou proposer une autre architecture.

Pour **chaque période** (1970–1979, 1980–1989, 1990–1994, 1995–1999, 2000–2009, 2010–2019, 2020–2026), donne :
1. **la machine cible**, précise (un modèle réel) ;
2. **le support** : un programme natif, ou HTML / CSS / JavaScript, et lesquels ;
3. **ce qui est gardé du monde** (lieux, liens, actions, graine, apparence) ;
4. **ce qui est perdu** ;
5. **la preuve à exécuter** pour montrer que ça marche vraiment ;
6. **la différence avec ma vision actuelle**.

Puis dis-moi :
- ce que tu ferais **en premier**, en cinq lignes au plus ;
- les **risques** ou les erreurs que tu vois chez Codex ;
- si cela doit passer **avant ou après** la fin de la partie web et la 3D (aujourd'hui : on finit les neuf lots du web, puis la 3D).

Sois franc : si une idée est mauvaise, dis-le, et dis ce que tu ferais à la place. Aucun avis ne devient une décision sans moi.
