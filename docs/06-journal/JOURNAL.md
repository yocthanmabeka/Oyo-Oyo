# Journal d'évolution du projet

Ce journal raconte ce qui a été fait, ce qui a raté et ce qui a été décidé, étape par étape, avec les captures. Il est tenu par Claude après chaque étape, à la demande de Yocthan, sans le lui annoncer. La dernière entrée est en haut. Les captures sont dans [`images/`](images/).

Pour l'état courant en un coup d'œil, voir [`AGENTS.md`](../../AGENTS.md) à la racine.

---

## 2026-10-07 — Lot 9 : la fin du web

- Yocthan, sur les manques du grand tableau : « Le seul manque en priorité […] si tu le fais, tu me dis quand est-ce que je veux le voir » ; « Plus de touches du clavier. Faire apparaître un bloc quand on descend. Régler un son. Bref, oui, oui. Ça, il faut le faire » ; « tu valides déjà le tout ».
- Fait et décidé (`ADR-061`) :
  - **Toutes les touches utiles** : `Key.enter`, `Key.escape`, les lettres `Key.a` à `Key.z` (celles écrites sur la touche), les chiffres `Key.digit0` à `Key.digit9` (par leur place : un clavier français marche sans Maj). Jamais Tab. Le menu ☰ offre « Touches à une lettre », pour les couper (la dictée vocale, WCAG 2.1.4).
  - **Apparaître en descendant** : `Enter(…, inView: true)`, guetté par la page légère, sans écouter le défilement ; sans JavaScript ou avec « moins de mouvement », tout se voit d'emblée.
  - **Le son réglé** : `volume:` de 0 à 1, `loop: true`, et `stop`.
  - **Des tailles qui suivent le visiteur** : l'auteur écrit des pixels, le navigateur reçoit des `rem` (marges, largeurs, hauteurs, coins, écarts) ; `height: screen` remplit l'écran sans le défaut de `100vh` sur un téléphone.
  - **La vue points se lit au lecteur d'écran** : la page reste sous les points, invisible mais lisible ; « Vue points » et « Vue web » sont annoncés ; Tab ramène la vue web.
- Leçons 77 à 81. Le grand tableau, remis à jour aussi pour les ajouts de la nuit (`ADR-050` à `ADR-059`, que la session du cloud n'y avait pas portés) : 333 mots, tous décidés ; il ne reste que deux manques « utiles », tous deux pour la 3D (dessiner des objets pleins).
- Vérifié dans Chrome sans fenêtre, d'abord sur l'ancien code, puis sur le code en anglais (voir plus bas).
- Pour l'entendre : ouvrir la leçon 81 dans la pile, lancer le Narrateur de Windows (Ctrl + Windows + Entrée), zoomer jusqu'aux points : il dit « Vue points » et lit toujours le texte.

![La leçon 78 : la carte arrivée à l'écran est entrée](images/2026-10-07-lot9-apparaitre.png)

![La leçon 81 en vue points : le lecteur d'écran lit toujours la page](images/2026-10-07-lot9-vue-points-lue.png)

**Erreurs en route**

- Pendant que je construisais ce lot, une autre session (dans le cloud) a fusionné dix-neuf PR, dont le passage de tout le code en anglais (`ADR-060`) et une décision qui prenait déjà le numéro `ADR-048`. Ma première PR (137) ne pouvait plus fusionner : j'ai repris le lot sur le nouveau code, sous le numéro `ADR-061`, avec les leçons 77 à 81 (70 à 76 étaient prises). Je n'avais pas regardé `main` avant de commencer : désormais, je le relis avant chaque lot.
- J'avais écrit le volume en pourcentage (`volume: 40%`) : le langage n'accepte pas `%` dans un paramètre de bloc. Choisi plutôt de 0 à 1, comme l'opacité.
- Mon premier essai attendait qu'une seule carte entre ; en centrant la première, la deuxième était aussi à l'écran, et elle est entrée elle aussi, comme il se doit : l'erreur était dans mon test.
- La construction du moteur web est lente sur ce PC (jusqu'à seize minutes) ; le script `build.ps1`, lancé avec ses messages redirigés, s'arrête sur les messages de cargo : j'ai lancé les mêmes étapes à la main.

---

## 2026-10-07 — Tout le code du moteur passe en anglais

- Yocthan : « le code doit être 100 % en anglais, bien sûr. […] le commentaire, tu le mets en français pour que je comprenne ». Puis : « Oui, fais comme je le dis. »
- Fait (`ADR-060`) : environ 1 500 noms traduits dans le moteur Rust, les scripts du navigateur, le serveur d'essai, les outils et l'extension VS Code ; les fichiers sources renommés (`etat.rs` → `state.rs`, `page-moteur.js` → `page-engine.js`…) ; les attributs, classes et variables CSS du moteur ; les commandes (`holo test`, `holo files`, `holo vocabulary`) et les paramètres d'adresse (`?values`, `/editor?key=`, `/stack`). Les commentaires, les messages d'erreur, les textes de l'interface, les leçons et la documentation restent en français.
- Comment : un outil a relevé chaque identifiant hors commentaires et textes ; la table de traduction a été écrite à la main ; le compilateur Rust a signalé les `{nom}` restés dans les messages ; puis chaque page a été rouverte dans Chrome.
- Raté puis corrigé : deux pages HTML dupliquées par l'outil (positions non triées) ; `move` est un mot réservé de Rust (`translate`) ; des noms importés du wasm mis en camelCase ; `history` qui masquait `window.history` dans la pile ; l'audit qui cherchait `axis` au lieu d'`axe`.
- Vérifié : 123 tests ; les deux paquets wasm ; dans Chrome, le monde d'accueil, neuf leçons, le site de référence, l'éditeur et la pile ; l'audit axe-core, 76 leçons et le site de référence, 0 défaut.

---

## 2026-10-07 — Un serveur et des comptes, proposés

- Yocthan demande si les langages issus de JavaScript passent par Node.js pour le serveur et les comptes. Réponse : oui, presque toujours, avec deux programmes à garder d'accord.
- Écrit, en proposition : `proposals/Claude/serveur-et-comptes-2026-10-07.md`. Un serveur en Rust (`holo serve`) qui fait tourner le même arbitre que la page ; puis des valeurs partagées, des comptes sans mot de passe (clés d'accès), le direct à plusieurs. Quatre questions attendent Yocthan ; rien n'est construit.

---

## 2026-10-07 — Envoyer un fichier

- Yocthan : « L'envoi d'un fichier […] travaille aussi sur ça. »
- Fait (`ADR-059`) : `Input(type: file, value: photo, label: "…", accept: image, max: 2MB)` dans un `Form`. La valeur est le nom du fichier choisi. La page refuse tout de suite un fichier d'une autre sorte ou trop lourd ; le serveur demande au moteur ce que la page permet (`holo fichiers`), lit la sorte dans les premiers octets, et range le fichier dans `messages/fichiers/` sous un nom tiré au hasard.
- Corrigé au passage : dans un formulaire, l'étiquette d'un champ était sur la même ligne que lui (la leçon 64 aussi) ; elle est maintenant au-dessus, comme hors d'un formulaire.
- Vérifié : tous les tests du moteur ; dans Chrome, un fichier texte et une image de 2,1 Mo refusés, une vraie image envoyée et rangée ; au serveur, sans la page : faux PNG 415, trop lourd 413, champ inconnu 400, chemin dans le nom ignoré ; l'audit axe-core, 76 leçons et le site de référence, 0 défaut.

![La leçon 76 : une image choisie, prête à partir](images/2026-10-07-envoyer-un-fichier.png)

---

## 2026-10-07 — Un emplacement pour du contenu dans un composant

- Yocthan : « un emplacement pour contenu dans un composant. Oui […] dans Flutter, il y avait children […] et child ».
- Choisi : un seul mot, `children`, comme partout dans HoloCode ; `child` est refusé avec le bon mot.
- Fait (`ADR-058`) : `params: [title, children]` ; le mot `children` posé seul dans une liste du composant ; `Panel(title: "…", children: [ … ])` à l'appel. Le contenu est déplié chez la page, puis posé dans la copie sans être renommé. Une carte peut contenir une carte.
- Vérifié : tous les tests du moteur (sept refus) ; la leçon 75 et son essai écrit ; dans Chrome avec `?valeurs` (le bouton posé dans l'encadré change la valeur de la page) ; l'audit axe-core, 75 pages, 0 défaut.

![La leçon 75 : trois encadrés, dont un dans un autre](images/2026-10-07-contenu-d-un-composant.png)

---

## 2026-10-07 — Un champ dans une ligne, et des lignes qui restent en place

- Yocthan : « travaille sur une condition sur un champ dans une ligne des listes. Les clés stables des listes, ok. »
- Fait (`ADR-057`) : `If(item.done, is: 1, children:, else:)` dans les lignes d'un `Repeat(over:)` (nombre : `is`, `not`, `over`, `under` ; texte : `is`, `not`) ; `item.done.set(1)`, `item.likes.add(1)` dans les règles de la ligne ; chaque ligne porte une clé tirée de son contenu, et la page ne remplace que les lignes nouvelles ou changées.
- Raté puis corrigé : la première version comparait le HTML du moment ; un pli ouvert y ajoute `open`, et la ligne était refaite. La page compare maintenant au HTML que le moteur avait fabriqué.
- Vérifié : tous les tests du moteur ; la leçon 74 et son essai écrit ; dans Chrome, le pli ouvert de la première tâche reste ouvert (le même nœud) quand on change la deuxième ; l'audit axe-core, 74 pages, 0 défaut, en clair et en sombre sur téléphone.

![La leçon 74 : le pli de la première tâche est resté ouvert](images/2026-10-07-champ-dans-une-ligne.png)

---

## 2026-10-07 — Le planning de la 3D, proposé

- Écrit : `docs/04-roadmap/PLAN-3D.md`, en proposition. Douze étapes, de la profondeur à la chaise mesurée sur le Flip 3, puis les mots du langage choisis après la chaise ; environ 15 séances. Une contrainte dite d'emblée : sans carte graphique dans le nuage, Claude ne voit pas l'image ; l'étape 0 essaie de lever cette limite.

---

## 2026-10-07 — `Component`, des valeurs par défaut, des signaux branchés par la page

- Yocthan : les propositions de Gemini et Codex validées ; « on doit prendre Component, vu qu'on aura besoin de Part pour la 3D » ; et pour l'essai au TalkBack : « c'est déjà bon » (noté tel quel ; aucun tableau de résultats n'a été rendu).
- Fait (`ADR-056`) : `Component` et `components:` remplacent `Part` et `parts:` (l'ancienne écriture est refusée avec le bon mot) ; `params: [title, price: 0, image: "…"]` ; `emits: [add]`, `On(Add.tap, emit: add)`, et `onAdd: cart.add(1)` à l'appel ; le panneau `?valeurs` montre le dernier geste et ce qu'il a changé.
- Migrés : les leçons 25 et 70, `exemples/site/commun.holo`, `site-reference/commun.holo` et `pied.holo`, le guide, `NOMS.md`, le serveur et l'audit d'accessibilité. Leçon 73 et son essai écrit.
- Vérifié : tous les tests du moteur ; la leçon 73 dans Chrome avec `?valeurs` (« Dernier geste : AddGift.tap ; cart : 12000 → 13000 ; likes : 0 → 1 »).

![La leçon 73 et le panneau du dernier geste](images/2026-10-07-defauts-et-signaux.png)

---

## 2026-10-07 — Une idée gardée au chaud : le langage, le socle et le framework

- Yocthan demande s'il faudra séparer HoloCode en un langage et un framework, comme Dart et Flutter, et si les blocs de base doivent devenir des composants (réponse : non, ce sont les briques des composants).
- Claude propose trois couches : le langage (la grammaire), le socle (les blocs du moteur, avec leurs garanties) et un framework de composants écrits en HoloCode. Yocthan : à garder au chaud, sans en faire une proposition ni un essai ; on continue ce qui est en cours.
- Rangé : `proposals/Claude/idees/langage-socle-framework-2026-10-07.md`.

---

## 2026-10-06 — Gemini répond sur les composants ; les chiffres d'opinion corrigés

- Gemini a répondu (`docs/05-discussions/reponses/2026-10-06-gemini-composants-et-comparatif.md`), sur l'état du matin : sa « combinaison gagnante » pour le restylage (variables et classe à l'appel) est celle qui était construite ; il propose en plus des valeurs par défaut, des signaux émis (`emits`, `onAdd:`), des noms internes privés, et `Component` plutôt que `Part`. Il jugeait le moteur léger et le fichier de styles prématurés : déjà faits, rien à défaire. Son tableau : HoloCode 68,5 %, Svelte 83 %.
- Le comparatif corrigé avec les pages officielles de Stack Overflow 2026 lues par Codex : en 2025, Phoenix était le framework le plus admiré, pas Svelte (mon erreur) ; les chiffres de State of JS restent non vérifiés. Ajouté : les trois tableaux côte à côte (§ 11).

---

## 2026-10-06 — L'accessibilité vérifiée ; les six points sont faits

- Point 6 du comparatif. Fait (`ADR-055`) : axe-core passé sur toutes les leçons et tous les sites d'exemple, dans Chrome. Avant : 69 leçons sur 70 sans défaut ; trois défauts de contraste (le badge de la boutique, la pastille de la leçon 38, les liens de la leçon 52 en thème clair) et la page des mondes (zoom interdit, ni repère ni titre). Tout est corrigé : **0 défaut sur 72 pages**, en thème clair et sombre, à 1000 et 390 pixels.
- Le moteur refuse maintenant un texte trop peu contrasté quand le même style donne les deux couleurs (4,5 pour 1, ou 3 pour 1 en grand). Il a trouvé les deux badges ; il ne pouvait pas voir les liens de la leçon 52 (deux styles différents) : c'est l'audit qui les a trouvés.
- Écrits : `moteur/outils/accessibilite.mjs` (l'audit) ; `docs/01-holocode/ESSAI-LECTEUR-D-ECRAN.md` (le protocole de l'essai humain, au TalkBack, à faire par Yocthan).
- Le comparatif est mis à jour (§ 10) : HoloCode passe de **70 % à 78,5 %** pour son public, selon Claude (Svelte 82 %).

**Erreurs en route**

- Mon contrôle de contraste lisait la valeur claire d'une variable redéfinie dans le thème sombre : faux refus de l'exemple du guide. Corrigé.
- En arrêtant le serveur d'essai, ma commande a encore arrêté le terminal qui la lançait.

---

## 2026-10-06 — Les outils de l'auteur

- Point 5 du comparatif. Fait (`ADR-054`) : `?valeurs` dans l'adresse montre les valeurs de la page à chaque geste ; `holo fmt` remet un fichier en forme (seuls les blancs changent) ; `holo essai page.holo page.essai` joue des gestes écrits et vérifie les valeurs (`tap`, `signal`, `type`, `receive`, `expect`).
- Trois essais écrits pour les leçons 68, 70 et 71, joués par les tests du moteur.
- `holo fmt` passé sur 74 fichiers d'exemples : 72 déjà en forme ; deux remis en forme, dont les lignes étaient vraiment mal alignées (`site-reference/commun.holo`, `pied.holo`).
- Vérifié : 117 tests ; le panneau dans Chrome sur les leçons 70 (`cart = 18000` après deux clics) et 71 (les trois articles et leurs champs).

![Le panneau des valeurs, en bas à gauche de la leçon 70](images/2026-10-06-panneau-des-valeurs.png)

**Erreurs en route**

- La première mise en forme comptait un niveau par parenthèse ; les leçons en comptent un par ligne (`Row(children: [` ne décale que de deux espaces) : refaite.
- Elle déplaçait aussi des commentaires que l'auteur de la boutique avait alignés exprès : un commentaire seul, aligné plus loin, garde maintenant sa place.

---

## 2026-10-06 — Codex relit les composants ; deux styles importés ne se gênent plus en silence

- Codex a répondu au prompt sur les composants (PR 124, `proposals/GPT5.6/web-assez-utilisable-2026-10-07/`). Il constate que les composants à paramètres existaient déjà, refait le tableau (HoloCode à **74,6 %** pour son public, Svelte 85,4 %), et relève un défaut : les noms de style d'un fichier importé valent pour toute la page, et deux fichiers qui écrivent le même style se gênaient, le premier gagnant en silence.
- Corrigé (`ADR-050`, correction du jour) : deux fichiers importés qui écrivent le même style sont refusés, avec leurs deux noms. Les noms de style importés restent partagés avec la page, parce que le site de référence s'en sert comme d'un thème commun ; un composant qui ne veut rien partager se style par son nom.
- Vérifié : 115 tests ; toutes les pages de `exemples/site/` et `exemples/site-reference/` passent `holo check`.

---

## 2026-10-06 — Un moteur léger, le dessin à part

- Point 4 du comparatif. Fait (`ADR-053`) : une option de compilation `dessin` ; sans elle, le moteur léger lit, fabrique la page et arbitre, sans `wgpu`. La page prend le léger (`/pkg-leger/`) et ne fait venir le dessin (`/pkg/`) que si elle montre des points ou des mondes. L'éditeur prend le léger.
- Mesuré : moteur entier 626 Ko transférés, **moteur léger 149 Ko**, quatre fois moins. **L'objectif de 100 Ko n'est pas atteint** : ce qui reste est le cœur (lecture, vérifications et leurs messages, page, arbitre).
- Vérifié dans Chrome sans fenêtre : la leçon 70 ne télécharge que le moteur léger, ses boutons comptent (deux « Night » : 120,00 euros) ; la leçon 9, zoomée à la molette avec Ctrl, fait venir le dessin et passe en vue points.

**Limites et erreurs**

- Dans le nuage, Chrome n'a pas de carte graphique : le dessin des points n'a pas pu être regardé (« ni WebGPU ni WebGL 2 ne sont utilisables »). À vérifier sur le PC et le téléphone de Yocthan.
- En relançant le serveur d'essai, ma commande d'arrêt a aussi arrêté le terminal qui la lançait : relancé à part.

---

## 2026-10-06 — La place qui reste, et un thème partagé

- Point 3 du comparatif. Fait (`ADR-052`) : `grow:` sur un bloc rangé dans `Row` ou `Column` (comme `Expanded` en Flutter, de 1 à 12 parts) ; un fichier qui ne contient que des styles s'importe comme un thème (`import "theme.holo"`). Les noms de style multiples étaient déjà là avec les composants.
- Leçon 72 (`72-place-et-theme.holo`, avec son thème `72-theme.holo`) ; guide § 4 bis et § 5.
- Vérifié : 114 tests ; la leçon 72 dans Chrome, sur PC et en largeur de téléphone (390 pixels).

![La leçon 72 : le champ qui prend la place qui reste, une part, deux parts](images/2026-10-06-place-et-theme.png)

**Erreurs en route**

- La pull request des listes à champs (122) est d'abord tombée en rouge sur GitHub : j'avais écrit l'exemple du guide après avoir lancé les tests, et il commençait par un `H2` sans `H1`. Corrigé, puis fusionnée au vert. Leçon retenue : relancer les tests après chaque retouche du guide.
- Le champ qui grandit restait à 280 pixels : une règle CSS du champ passait devant la mienne. Corrigé.
- Le moteur vérifiait les réglages d'un champ de saisie avant de voir `grow:`, et le refusait. Corrigé.

---

## 2026-10-06 — Les listes à champs, aussi reçues du serveur

- Point 2 du comparatif, avec le feu vert de Yocthan. Fait (`ADR-051`) : `State(articles: [ Item(title: "Sunrise", price: 12000) ])`, `{item.title}` et `{item.price:cents}` dans les lignes, `articles.push(Item(title: name, price: price))`, et `Data` qui remplit une liste depuis un tableau JSON (objets ou textes).
- Leçon 71 (`exemples/lecons/71-liste-a-champs.holo` et son `71-catalogue.json`) ; guide § 6 septendecies.
- Vérifié : 113 tests (dont un texte saisi `<Night>` qui reste du texte, un champ « secret » envoyé par le serveur qui n'apparaît pas, une image en `javascript:` refusée) ; la leçon 71 dans Chrome : trois articles arrivés du fichier JSON, « Retirer » en enlève un, « Forest » à 9500 centimes s'ajoute et se montre « 95,00 euros ».

![La leçon 71 : le catalogue venu du serveur, et un article ajouté](images/2026-10-06-liste-a-champs.png)

**Erreur en route**

- Le moteur refusait un `Item` hors d'une répétition (`ADR-040`) : il est maintenant permis dans `State` et dans une demande `push`.

---

## 2026-10-06 — Les composants, faits pour le web

- Yocthan : « J'aime le composant. C'est une notion de Flutter que j'adore […] il faudra des composants faits vraiment pour le web […] grâce au CSS, tu vas l'utiliser facilement » ; puis le feu vert pour les six points du comparatif.
- Fait (`ADR-050`) : un composant s'écrit une fois, `Part(name: ArticleCard, params: [title, price, qty], children: [ … ], rules: [ … ])`, dans `parts:` de la page ou dans un fichier importé ; il se pose comme un bloc, `ArticleCard(name: Sunrise, title: "Sunrise", price: 12000, qty: sunrise)` ; il se restyle de l'extérieur : `ArticleCard { … }` pour toutes les copies, `ArticleCard.promo(…)` et `.promo { … }` pour une seule, et ses variables (`.promo { --accent: crimson; }`). Un paramètre donné par le nom d'une valeur de la page la suit : c'est ainsi que le bouton d'une carte remplit le panier. Plusieurs noms de style par bloc (`P.card.big`), quatre au plus.
- Comme `Use` et `Repeat`, un composant est déplié à la lecture : le reste du moteur n'a pas changé. Un composant peut être répété (`Repeat(… children: [ ArticleCard(title: item.title, …) ])`) ; ses règles vont alors dans la répétition.
- Leçon 70 (`exemples/lecons/70-composants.holo`) ; guide § 6 sexies bis ; `NOMS.md`.
- Vérifié : 110 tests ; la leçon 70 dans Chrome sans fenêtre, sur le moteur compilé dans le nuage : trois cartes, la troisième rouge, deux clics sur « Night » et un sur « Sunrise » donnent 240,00 euros.

![La leçon 70 : trois copies du même composant, la troisième restylée par une ligne](images/2026-10-06-composants.png)

**Erreurs en route**

- Les règles d'un composant posé dans une répétition partaient dans la page, où `item` n'existe pas : elles vont maintenant dans la répétition.
- Un composant dont la racine est un autre composant n'était pas déplié : corrigé, avec le refus d'un composant qui se pose lui-même.
- Le premier essai dans Chrome cliquait trop vite : le moteur se charge au premier geste (`ADR-033`) ; avec une attente, les clics comptent.

---

## 2026-10-06 — HoloCode face aux langages et frameworks du web

- Yocthan, avant la 3D : comparer HoloCode « avec les meilleurs langages et frameworks web, en commençant par TypeScript et en terminant par Flutter », par pourcentage, avec l'avis des gens et celui de Claude, et dire pourquoi certaines notions ne sont pas reprises.
- Écrit : `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md`. Neuf retenus : TypeScript, React, Angular, Vue, Svelte, SolidJS, Astro, Elm, Flutter. La même liste de tâches écrite dans les dix (HoloCode : 22 lignes, aucun code, rien à installer ; Svelte : 15 ; Flutter : 38 ; Elm : 50). Les avis des enquêtes (Stack Overflow 2025, State of JS 2025). Une note par critère.
- Résultat, selon Claude : HoloCode à **70 %** pour son public (Svelte 82 %, Vue 77 %, React 71 %, Flutter 70 %), **58 %** pour un développeur professionnel. Premier sur la facilité, la concision et l'accessibilité ; dernier sur les composants (un `Part` n'a pas de paramètres), ce qu'on peut construire, les outils et l'entraide.
- Yocthan, le même jour : les composants sont sa notion préférée de Flutter ; il les veut « faits pour le web », réutilisables deux fois avec une autre allure grâce au CSS ; puis les points 2 à 6. Avant de continuer, deux prompts : `docs/05-discussions/prompts/2026-10-06-gemini-composants-et-comparatif.md` et `2026-10-06-codex-composants-et-comparatif.md` (composants, même tableau rempli par eux, reste du plan).
- Proposé avant la 3D : des morceaux à paramètres, des listes à champs, la disposition qui manque, un moteur allégé pour les pages qui bougent, des outils, un essai au lecteur d'écran ; environ 81 % avec ces six points. Rien n'est décidé.

**Limites**

- Les chiffres des enquêtes viennent d'extraits de recherche : les pages de Stack Overflow et de State of JS étaient bloquées depuis la session. Stack Overflow 2026, sortie le jour même, n'a pas pu être lue.
- Les notes sont le jugement de Claude, qui juge son propre travail ; le nombre de lignes est la seule mesure.

---

## 2026-10-06 — La veille sur la 3D, et les décisions prises

- Yocthan : « vérifier ce que pensent les humains en 2026, exactement le 6 octobre […] et enfin tu mets décision prise ».
- Veille faite sur Internet (`docs/05-discussions/veille/2026-10-06-3d-web-et-mobile.md`) : WebGPU actif par défaut dans Chrome Android et Safari 26, pas dans Firefox Android ; tout le monde garde WebGL 2 ; sur certains Android, WebGPU est plus lent que WebGL 2 ; sur iPhone, des pages plantent dès 100 à 200 Mo ; la qualité adaptative est une pratique courante ; `KHR_gaussian_splatting` est ratifié ; glTF 2.1 est annoncé ; le mot « métavers » est mal vu depuis la fermeture d'Horizon Worlds en réalité virtuelle.
- Décisions écrites : **`ADR-048`** (HoloCode décrit une qualité, jamais une technique) et **`ADR-049`** (objets préparés à l'avance, paliers qui bougent pendant la visite, WebGL 2 obligatoire, chemin de base sans calcul général, Khronos PBR Neutral, meshoptimizer), acceptées. Le chantier attend le feu vert de Yocthan.

**Limites**

- Reddit, Poly Haven et plusieurs forums n'ont pas pu être lus directement : ce qui en vient est tiré d'extraits de recherche. Peu de discussions d'août à octobre 2026 trouvées. Le nombre de triangles de la chaise reste à confirmer.

---

## 2026-10-06 — La 3D réaliste : deux prompts avant de construire

- Yocthan veut commencer la 3D, « ultra réaliste si possible », mais sans faire exploser les machines : commencer bas, croître petit à petit, et s'aider d'extensions s'il le faut. Il demande l'avis de Codex et de Gemini avant la première ligne.
- Direction proposée par Claude, dans la discussion : abandonner les points pour les objets proches (sans profondeur, jamais réalistes) ; des modèles préparés à l'avance sur PC (la voie D de Codex), avec niveaux de détail et lumière précalculée ; des paliers de qualité automatiques (léger, normal, haut) ; le réalisme par la lumière et les matériaux. Premier essai envisagé : une chaise, en points et en objet plein, sur le Flip 3. Rien n'est décidé ni construit.
- Écrits : `docs/05-discussions/prompts/2026-10-06-gemini-3d-realiste.md` (sans accès au dépôt, tout le contexte dedans) et `2026-10-06-codex-3d-realiste.md` (réponse attendue par PR dans `proposals/GPT5.6/`). Les extensions y sont cadrées par `ADR-011` : pas de pont JavaScript, modules enfermés acceptés.
- Gemini a répondu le même jour (`docs/05-discussions/reponses/2026-10-06-gemini-3d-realiste.md`, lecture critique de Claude à la fin) : « oui, à condition de tout préparer hors ligne ». À vérifier avant d'y croire : l'onglet qui serait abattu à 350–450 Mo sur un téléphone d'entrée de gamme (ce qui remettrait en cause le 1 Go d'`ADR-005`), et le coût réel de KTX2. Codex a répondu par la PR 118 (`proposals/GPT5.6/3d-realiste-evolutive-2026-10-06/`), et ChatGPT a relu Gemini (`docs/05-discussions/reponses/2026-10-06-chatgpt-sur-gemini-3d-realiste.md`). Les trois disent : possible, sans revoir la vision. Synthèse de Claude, avec ce que Yocthan a à décider (`ADR-048` le principe, `ADR-049` la direction, puis le premier chantier) : `proposals/Claude/3d-realiste-2026-10/SYNTHESE.md`.
- Fait depuis une session dans le nuage, sans le PC de Yocthan : rien n'a été lancé ni mesuré.

**Erreur en route**

- Au début de la session, le dépôt `Metaverse` était invisible (Claude n'y avait pas encore accès) ; Claude a d'abord cherché dans les autres dépôts.

---

## 2026-10-06 — La pile : tout voir dans un seul onglet

- Yocthan, à court de batterie, avait fermé des fenêtres : « faire une stack […] qui me permettra de consommer moins de charges […] pour voir tout ce que tu crées ».
- Fait : **la pile**, `http://localhost:8080/pile`. Tout ce qui s'ouvre dans le navigateur (96 pages aujourd'hui : 69 leçons, les sites, les jeux, les jumeaux en HTML, les mondes), le plus récent en haut, rangé par jour, avec une recherche. La page choisie s'ouvre à côté de la liste ; sur un téléphone, à sa place, avec « ← La pile », et elle est arrêtée quand on revient. La date est celle du dernier commit du fichier (celle du disque pour un fichier pas encore versionné). Un morceau (`Part`), qui ne s'ouvre pas seul, se montre dans l'éditeur. « nouveau » marque ce qui a changé depuis la dernière visite.
- Pour montrer une page : `node outils/montrer.mjs /exemples/…`. La pile ouverte l'affiche elle-même, sans nouvel onglet ; s'il n'y en a pas, Chrome s'ouvre une fois, sur la pile. Seul ce PC peut le demander, pas un autre appareil du Wi-Fi. Rien ne tourne tant qu'on ne choisit rien ; la liste est relue quand on revient sur l'onglet.
- Corrigé en passant : l'éditeur ouvrait par défaut une leçon qui n'existe pas (`01-bonjour.holo`) ; c'est `01-page.holo`.
- Vérifié dans Chrome sans fenêtre : la liste (96 liens), la leçon 9 ouverte par l'adresse, puis la leçon 68 montrée par le serveur dans le même onglet ; sur un écran de téléphone, la page à la place de la liste, le retour qui l'arrête, rien qui dépasse de l'écran.

![La pile sur un PC : la liste, et la page montrée par Claude](images/2026-10-06-pile.png)

![La pile sur un téléphone](images/2026-10-06-pile-telephone.png)

**Erreurs en route**

- Sur un écran de téléphone, la barre au-dessus de la page dépassait de 150 pixels (une colonne de grille qui prenait la largeur de son texte) : corrigé avant la PR.
- Mes essais depuis Git Bash envoyaient `C:/Program Files/Git/exemples/…` au lieu de `/exemples/…` (Git Bash convertit les chemins) : `montrer.mjs` retrouve maintenant le bon chemin.
- En relançant le serveur d'essai, l'ancien tenait encore le port : arrêté à la main.

---

## 2026-10-06 — Les PR de Codex fusionnées ; les quatorze noms tranchés

- Yocthan : « Il y a 14 mots à nommer […] tu fais le pull request des codex […] je suis tes recommandations. »
- Fusionnées, par le script et tests verts : les PR 72 (la contre-revue des noms par Codex), 74 (modèles 3D, arbitre partagé, noms ajoutés) et 79 (le cahier du site de référence). Ce sont des propositions, rangées dans `proposals/GPT5.6/` ; aucune ne change le moteur. Le dossier principal est revenu sur `main`, et le moteur en ligne de commande y est construit (l'extension VS Code s'en sert).
- Rangé : la branche `revert/gemini-v0.1` (et sa copie de travail) supprimée ; les issues #16, #20 et #84 fermées avec un mot d'explication. Il reste une issue ouverte, #82.
- Les noms (`ADR-047`) : douze gardés, deux changés. `Points(grid:)` devient `divisions:` (trois sens pour `grid`, c'était le défaut du CSS) ; `Points(depth:)` devient `levels:` (`depth` reste libre pour la profondeur de la 3D ; « levels » est le mot de Blender). `above` reste : `anchor`, que Codex lui-même jugeait son choix le moins solide, est le nom des liens sur le web. Les anciennes écritures sont refusées avec le bon mot, que l'éditeur remplace d'un clic.
- Les décisions qui attendaient les noms sont acceptées, écriture comprise : `ADR-017`, `019`, `021`, `022`, `023`, `024`, `026`, `028`, `031`.
- Vérifié : 106 tests ; les fichiers `.holo` du dépôt donnent les mêmes 23 refus voulus qu'avant ; `Points(grid: 4)` refusé ligne 3, colonne 18, avec « écris « divisions » » ; la leçon 9 dans Chrome, zoomée : 3 102 points, morcelés deux fois.

![La leçon 9 avec les nouveaux noms : la page devenue des points](images/2026-10-06-lecon9-noms.png)

**Erreur en route**

- Ma première capture zoomait avant l'arrivée du moteur (il lui faut une dizaine de secondes dans le Chrome de test) : la page restait plate. Refaite en attendant le moteur.

---

## 2026-10-06 — Tout ce qui est construit et essayé est validé ; deux installations

- Yocthan, sur le relevé de ce qui restait : « valide le point 1, 2, j'ai testé et ça marche » ; « tu mets trop de trucs en essai, trop de trucs en attente […] tu valides déjà le tout » ; et, pour la suite, « fais-le et valide ».
- Validé : les lots 6, 7, 8 et l'éditeur (`ADR-043` à `ADR-046` : `ACCEPTÉ`). Le grand tableau compte 311 mots, **tous décidés** ; plus aucune mention « à l'essai » dans le guide, la comparaison, le tableau ou `AGENTS.md`.
- La spécification Python de Codex (`SPECIFICATION-V0.1.md`) est `REMPLACÉE` par le moteur et le guide (« je suis tes recommandations »). Seul HoloIR reste une proposition : rien n'est construit, on en reparlera avec la 3D.
- La règle de décision (`ADR-035`) est choisie : on part du travail à faire de l'auteur (la question de ChatGPT), la majorité des humains sert de vérification ; s'ils se contredisent, Yocthan tranche.
- La recette du site de référence : le verdict de Yocthan est bon (« mon verdict, il est déjà bon ») ; restent cinq débutants à trouver et le téléphone.
- **Installé**, avec l'accord de Yocthan (« Oui, installe ») : Chrome DevTools MCP pour Claude Code (issue #20), à la portée de l'utilisateur, avec `--isolated` (un profil Chrome vide, effacé à la fermeture). Essai minimal, Chrome sans fenêtre : la leçon 1 ouverte, son titre lu, tout fermé, en 10 secondes ; aucun processus resté ouvert. Les outils n'apparaissent qu'au démarrage d'une nouvelle session de Claude Code.
- **Installé** : l'extension VS Code 0.2.0 (la faute soulignée, l'ampoule, les mots proposés). Il faut recharger la fenêtre de VS Code.

**Erreurs en route**

- J'avais laissé à l'essai des lots que Yocthan avait déjà essayés et trouvés bons : c'est à lui de le dire une fois, pas à moi d'attendre une deuxième fois.
- En présentant la règle de décision, j'avais résumé l'option de ChatGPT par « ce qui demande le moins de travail » : c'était faux. Il s'agit du travail que l'auteur veut accomplir, pas de notre travail à nous.

---

## 2026-10-06 — Ce qui reste à faire, revérifié

- Yocthan : « Révérifie et dis-moi ce qui reste à faire… sans brûler tous mes tokens. » Un premier relevé par quinze agents a été arrêté à sa demande ; le relevé a été refait à la main, source par source : les statuts (`DECISIONS.md`), GitHub (pull requests, issues, branches), le dossier principal, les propositions, le grand tableau, les prompts, `AGENTS.md`.
- Trouvé en plus de ce que Claude avait annoncé de mémoire : les issues ouvertes #82 (une exploration des dix pistes de Codex, demandée à Claude, jamais rendue), #84 (la revue de Codex sur les refus, jamais publiée), #20 (installer Chrome DevTools MCP, autorisé par Yocthan le 2026-10-03, jamais fait) et #16 (la revue de Codex du 2026-10-03, dont les points B-01 à B-11 sont traités dans le code) ; quatorze noms contestés par Codex, et non deux ; la branche `revert/gemini-v0.1`, devenue sans objet (les tests de Gemini passent : 8 sur 8) ; le prompt de Codex sur les animations, sans réponse.
- Corrigé : deux phrases périmées d'`AGENTS.md` (les listes, le calcul et le code enfermé « restaient à faire » ; la synthèse des refus « attendait une décision ») et le statut de la synthèse des refus.

**Erreur en route**

- Ma première réponse, de mémoire, comptait deux noms contestés au lieu de quatorze et oubliait les issues ouvertes : c'est pour cela que Yocthan a demandé de revérifier.

---

## 2026-10-06 — L'éditeur : la faute à sa place, la correction d'un clic

- Yocthan, citant la ligne du tableau « Corriger d'un clic — dans l'éditeur ; le moteur reste strict — « Remplacer par H1 » — à faire : il faut un éditeur » : « Travaille sur l'éditeur now ».
- Fait (`ADR-046`, à l'essai) : **l'éditeur du navigateur**, `/editeur`, sur le PC et le téléphone, sans rien installer (le texte en couleurs, la page à côté mise à jour pendant qu'on écrit, la faute soulignée, le bouton « Remplacer « h1 » par « H1 » », les mots à toucher, l'enregistrement avec la clé affichée par le serveur, une sauvegarde de l'ancienne version) ; et **l'extension VS Code 0.2.0** (la faute soulignée avant même d'enregistrer, l'ampoule, les mots proposés). Une seule logique de correction, `moteur/web/corrections.js`. Le moteur donne `verifier_texte` et `vocabulaire` (en ligne de commande : `holo check -`, `holo vocabulaire`).
- Vérifié dans Chrome : « h1 » → « H1 » d'un clic et l'aperçu qui montre « Bonjour » ; « textx » → « text » ; les blocs proposés dans `children: [` ; l'enregistrement avec la clé, refusé sans elle ; les onglets sur un écran de téléphone. L'extension, avec une imitation de VS Code et le vrai moteur : la faute soulignée (ligne 4, « h1 »), l'ampoule et son remplacement, 241 mots proposés. 106 tests.
- L'extension n'est pas encore installée dans le VS Code de Yocthan : c'est une commande, qu'il décide de lancer.

![L'éditeur : le texte, la page, les mots à toucher, la faute en bas](images/2026-10-06-editeur.png)

**Erreurs en route**

- Mes premiers essais de l'éditeur agissaient avant l'arrivée du moteur (8 secondes dans le Chrome de test) : fausses alertes, corrigées dans le test.
- Dans une liste `children: [`, l'éditeur proposait d'abord les réglages de la page au lieu des blocs : corrigé (il sait maintenant dans quelle sorte de liste on écrit).
- J'ai voulu réécrire `holo.rs` sans l'avoir relu depuis l'ajout de l'heure : l'outil l'a refusé ; relu, puis écrit.

---

## 2026-10-06 — Lot 8 : le premier module enfermé, et la preuve demandée par Codex

- Fait (`ADR-045`, à l'essai ; construction de `ADR-011`, partie C) : `module "…"` en haut du fichier ; `Module(name:, source:, input:, output:, time:, memory:)` ; `run`, `done`, `failed`. La boîte : un fil à part, une mémoire donnée par le moteur et plafonnée, rien d'autre ; arrêté au-delà de son temps. Les ponts `bridge js` et `bridge css` sont maintenant refusés. Leçon 69, et trois modules d'essai en Rust (`exemples/lecons/modules/`, 68 à 105 octets une fois compilés). Nouveau module du moteur : `modules.rs`.
- La preuve, dans Chrome : `compter` rend 5 050 ; `boucle` est arrêté après 2 000 ms, et la page a compté trois touchers pendant qu'il tournait ; `memoire` demande 64 Mo sous un plafond de 1 Mo et s'arrête. 105 tests.
- Rien n'a été installé : les modules sont compilés avec Rust et la cible WebAssembly déjà présents pour le moteur.

**Erreur en route**

- La pull request du lot 8 a été fusionnée sans `moteur/web/page-moteur.js` : j'avais oublié ce fichier dans la liste à commiter, et le script de fusion l'a signalé (« 1 uncommitted change ») après coup. Sans lui, la boîte du module n'existe pas dans la page. Réparé par une pull request de suite, aussitôt.

![Le module enfermé : somme, boucle arrêtée, mémoire refusée](images/2026-10-06-lot8-module.png)

---

## 2026-10-06 — Lot 7 : une liste qui change pendant la visite

- Fait (`ADR-044`, à l'essai) : `State(taches: [])` ; `taches.push(tache)`, `taches.remove(item)` (la ligne touchée), `taches.clear()` ; `tache.set("")` ; `Repeat(over: taches, …)`, une ligne par élément, redessinée par le moteur quand la liste change ; `{taches}` et `If(taches, is: 0)` ; `keep:` garde la liste. Leçon 68. Nouveau module du moteur : `listes.rs`.
- Vérifié dans Chrome : trois ajouts (un texte d'espaces refusé), le champ vidé, la ligne du milieu retirée, « Rien à faire. Bravo ! » après « Tout effacer », la liste retrouvée après avoir rechargé la page. Un texte piégé, `Livrer <b>{tache}</b>`, reste du texte. 104 tests.

![La liste de tâches](images/2026-10-06-lot7-liste.png)

**Erreurs en route**

- Après un rechargement, la liste gardée était bien relue, mais la page fabriquée d'avance montrait encore l'ancienne : le moteur ne redessinait les listes qu'à un changement. Corrigé : il les redessine aussi à son arrivée.
- La leçon écrivait `border-bottom`, que HoloCode n'a pas : le moteur l'a refusé ; remplacé par `border`.
- Deux anciens tests attendaient des messages devenus plus précis (une valeur peut maintenant être une liste ; « a » est un texte) : mis à jour.

---

## 2026-10-06 — Lot 6 : multiplier, diviser, et écrire un nombre joliment

- Yocthan : « Oui, travaille sur ce qui reste. » Fait (`ADR-043`, à l'essai) : les demandes `mul` et `div` ; les formats `{minute:00}`, `{n:number}`, `{n:cents}`, `{weekday:name}`, `{month:name}`, et `{item.price:cents}` dans une répétition ; la langue de la page choisit les séparateurs et les noms. Leçons 66 et 67. Nouveau module du moteur : `format.rs`.
- Vérifié dans Chrome : « Nous sommes mardi 6 octobre 2026, il est 19 h 45. », « Visites : 1 234 567 » puis « 1 236 567 » après deux clics, « 1 234,50 € » ; 3 cartes à 12 € font 36 €, 18 € chacun pour deux. 103 tests.

![La date en mots, les milliers, les centimes](images/2026-10-06-lot6-formats.png)

---

## 2026-10-06 — Ce qui restait « à l'essai » ailleurs

- Yocthan : « valide tout ce qui est à laisser [à l'essai] si tu n'as pas encore validé ».
- Recherché dans tout le dépôt. Mis à jour : le statut du moteur (`moteur/README.md`, `ACCEPTÉ`, mesuré à 60 images par seconde sur deux téléphones) ; l'en-tête de l'architecture (`ARCHITECTURE.md`, qui disait encore `EXPÉRIMENTATION` et « sans aucun HTML ») ; les titres « Décision (à l'essai) » des fiches déjà acceptées ; deux mentions dans le README de la boutique.
- **Non validé, exprès** : `docs/01-holocode/SPECIFICATION-V0.1.md`, la proposition en Python de Codex (GPT5.6). Le projet l'a remplacée par le moteur en Rust et le guide ; l'accepter dirait le contraire de ce qui est construit. Recommandation de Claude à Yocthan : la marquer `REMPLACÉ`. Décision attendue.

---

## 2026-10-06 — Les lots 1 à 5 validés

- Yocthan : « En fait, j'ai tout testé de tout ce qui était à laisser [à l'essai] et je trouve que c'est bon. Donc, euh, valide-le. »
- `ADR-038` à `ADR-042` passent en `ACCEPTÉ`. Toutes les fiches sont désormais acceptées, sauf HoloIR (`ADR-011`, partie D, proposition) et les ponts vers JavaScript (partie B, rejetés).
- Le guide, la comparaison, `AGENTS.md` et le grand tableau ne disent plus « à l'essai » : les 290 mots de HoloCode sont décidés. Tableau en ligne republié.
- Erreur trouvée en route : le guide disait encore « à l'essai » pour des écritures déjà validées le matin (`ADR-023` à `ADR-036`) ; corrigé en même temps.

---

## 2026-10-06 — ADR-011 précisée : la vue à plat reste du HTML, partout

- Yocthan a demandé pourquoi on perdrait ce que le web offre gratuitement, puis ce qu'est HoloIR ; après les réponses : « tu valides les différentes parties… A, B, C et D… on fait comme tu l'as dit ».
- Inscrit dans `ADR-011`, partie A : la vue à plat reste du vrai HTML, **même dans le navigateur propre au projet** (la phrase qui disait le contraire est barrée, pas effacée) ; la vue en profondeur recevra un jour une couche invisible pour les lecteurs d'écran. B (rejetée), C (direction acceptée) et D (proposition) ne changent pas.

---

## 2026-10-06 — Lot 5 : le HTML utile, et envoyer un message

- Fait (`ADR-042`, à l'essai) : `Page(icon:)` ; `~~barré~~`, `==surligné==`, `m^2^`, `H~2~O` ; `A(to: "#Horaires")` vers un bloc nommé (refusé s'il n'existe pas) ; `Image(caption:, phone:)` ; `Sound(label:)`, un lecteur ; `Slider` ; `Input(type: date | time | color)` ; `Progress` ; `Details` ; `Dialog` avec `open` et `close` ; `Form` avec `send`, puis `sent` ou `failed`. Leçons 55 à 65.
- L'envoi : décision de Yocthan, sur recommandation, le 2026-10-06 : les messages vont dans un fichier de son serveur local. Le serveur d'essai les range dans `messages/` (un fichier par page, une ligne par message, 16 Ko au plus, jamais versionné).
- Vérifié dans Chrome : les marques du texte ; le lien qui descend sans faire venir le moteur ; l'image légère choisie à 390 pixels de large ; le lecteur de son, à l'arrêt ; la glissière bornée (5 devient 20) ; la date, l'heure et la couleur ; la barre de progression ; les plis, sans moteur ; la fenêtre modale ouverte, puis fermée par « Oui, vider » ; **un vrai message arrivé** : `{"page":"/exemples/lecons/64-formulaire.holo","form":"Contact","values":{"nom":"Yocthan","message":"Bonjour, je voudrais le tableau de la rivière."}}`, et « Merci, ton message est arrivé. » à l'écran ; l'icône dans l'en-tête. 101 tests.
- Le moteur a refusé un exemple que j'écrivais pour le guide : une valeur appelée `day`, nom réservé depuis le lot 2 à l'heure du visiteur. Renommée `visit`.
- Couverture estimée après les cinq lots : HTML 82 %, CSS 78 %, JavaScript 48 % ; ensemble 72 % (53 % avant le lot 2, 47 % avant le lot 1) ; 290 mots, dont 66 à l'essai.

![La fenêtre par-dessus la page](images/2026-10-06-lot5-fenetre.png)

**Erreurs en route**

- La glissière semblait ne pas répondre : mon test lisait le nombre avant l'arrivée du moteur (plus de 9 secondes dans le Chrome de test). Vérifiée ensuite : 100, puis 110, puis 20.

---

## 2026-10-06 — Lot 4 : le CSS utile

- Fait (`ADR-041`, à l'essai) : `line-height` (sans unité), `letter-spacing`, `text-transform`, `text-decoration`, `box-shadow`, `text-shadow`, `rotate`, `scale`, `transition` ; `background` en dégradé ou en image (`url("fond.jpg")`, qui couvre toujours le bloc) ; les variables (`Page { --or: … }`, puis `color: --or;`) ; deux états de style, `dark:` et `phone:` (avec `display: none` seulement là) ; sa propre police, `fonts: [ Font(family:, source:) ]`, toujours affichée avec `font-display: swap`. Leçons 50 à 54. Le serveur sait maintenant servir les polices et les images `.png`, `.jpg`, `.webp`.
- La police de la leçon 54 est Carlito, libre (SIL Open Font License, « Copyright 2013 The Carlito Project Authors », lu dans le fichier lui-même), copiée depuis un paquet déjà présent sur le PC : rien n'a été téléchargé.
- Vérifié dans Chrome, par les styles calculés : majuscules et espacement, interligne, barré ; dégradé, ombre, image qui couvre ; la carte qui se redresse et grandit au survol ; les thèmes clair et sombre ; le bandeau caché sur un écran de 390 pixels ; Carlito chargée. 100 tests.
- Couverture estimée : CSS à 78 % ; ensemble de 56 % à 63 % ; 269 mots.

![Ombres, dégradé, image de fond](images/2026-10-06-lot4-ombres-et-fonds.png)

**Erreurs en route**

- Le message de refus de `display` avait perdu la phrase qu'un cas de conformité attend (« la disposition vient des blocs ») : remise.
- Mon premier essai du thème clair montrait le sombre : le Chrome de test suit le thème sombre de Windows. Le thème clair a été forcé pour le vérifier ; la page n'était pas en cause.

---

## 2026-10-06 — Lot 3 : écrire une carte une fois, la répéter

- Fait (`ADR-040`, à l'essai) : `Repeat(items: [ Item(…) ], children: [ … ], rules: [ … ])`. Dans le modèle, `item` désigne l'élément : `{item.title}`, `item.image`, `{item}`, `item.add(1)`. Un bloc nommé reçoit le nom de son élément (`Add` → `AddSunrise`) ; les règles du modèle sont écrites une fois par élément. Déplié à la lecture, comme `Use` : la page fabriquée d'avance est du HTML ordinaire. Leçon 49.
- Vérifié dans Chrome : trois cartes écrites une fois, chacune son bouton ; après trois ajouts, « Panier : 3 tableau(x), 330 € ». 99 tests.
- Couverture : `for, map` à 90 %, `template` à 85 %, les listes de valeurs à 50 % (une liste qui change pendant la visite reste à faire).

![Trois cartes écrites une fois](images/2026-10-06-lot3-repeter.png)

**Erreurs en route**

- Mon test oubliait un `H1` avant les `H2` : le langage l'a refusé, à raison.
- La première vérification dans Chrome cliquait avant l'arrivée du moteur (8 secondes dans le Chrome de test, sans carte graphique) : fausse alerte ; l'arbitre, appelé directement dans la page, répondait juste.

---

## 2026-10-06 — Lot 2 : le survol qui agit, le « sinon », plus tard, l'heure du visiteur

- Yocthan : « tu travailles sur le lot 2 jusqu'au lot 5… je suis tes recommandations. » Le lot 2 est fait (`ADR-039`, à l'essai) ; la page pour écouter ADR-011 est en ligne : https://claude.ai/artifact/28bt9BUqNTg7Bfq1DDm5kc
- Fait : `On(Carte.hover, …)` et `On(Carte.hoverEnd, …)` sur tout bloc nommé qui se voit ; `If(…, else: [ … ])` ; `After(3s, effect: …)`, qui part à l'ouverture ou, sous une condition, quand elle devient vraie ; l'heure du visiteur (`year`, `month`, `day`, `weekday`, `hour`, `minute`), tenue à jour à chaque minute. Leçons 45 à 48. Le serveur donne son heure au moteur (`HOLO_MAINTENANT`) pour la page fabriquée d'avance.
- Le choix du nom : `hover` plutôt que `near` (proposé pour la profondeur), parce que les styles disent déjà `hover:`.
- Contre les défauts du web : le survol est atteignable au clavier (le bloc reçoit le focus avec Tab) et au doigt (toucher survole, toucher ailleurs quitte) ; un survol ne peut pas emmener ailleurs.
- Vérifié dans Chrome : la page légère fait venir le moteur au premier survol, puis le rejoue (souris, clavier, doigt) ; le « sinon » suit le panier ; le bonjour arrive à 2 s et le message s'efface 3 s après l'ajout ; l'heure affichée passe de 18 h 33 à 18 h 34 à la seconde près. 97 tests. Tous les fichiers du dépôt passent, sauf les 23 refus voulus.
- Couverture estimée : ensemble de 53 % à 55 % ; 246 mots (22 à l'essai). Tableau en ligne republié.

![Le survol au doigt, sur un téléphone](images/2026-10-06-lot2-survol-au-doigt.png)

![L'heure du visiteur et le « sinon »](images/2026-10-06-lot2-heure.png)

**Erreurs en route**

- Un script de modification passé directement au shell a été mal lu (une apostrophe) : rien n'a été écrit. Les scripts passent désormais par un fichier.
- Mon premier test du survol plaçait la souris à côté de la carte, et celui du « sinon » regardait avant l'arrivée du moteur : deux fausses alertes, corrigées dans le test, pas dans le moteur.
- Le premier Tab sur la page légère réveillait le moteur sans rejouer le survol : corrigé (le focus est noté puis rejoué comme la souris).
- `{minute}` s'affiche sans zéro devant (`18 h 5`) : les formats de date restent à faire.

---

## 2026-10-06 — Les quatre parties d'ADR-011 décidées ; une page pour l'écouter

- Yocthan : « Personnellement, je suis tes recommandations. » A (rendu par vue) : `ACCEPTÉ` ; B (ponts vers JavaScript et CSS) : `REJETÉ`, raison gardée ; C (deux étages, modules enfermés) : `ACCEPTÉ` pour la direction, construction à faire ; D (HoloIR) : reste `PROPOSITION`.
- Il demande aussi d'entendre la fiche à voix haute : une page avec un bouton « Écouter » lit tout le fichier, paragraphe par paragraphe, avec la voix française du navigateur.

---

## 2026-10-06 — ADR-006, 012 et 013 réunies dans ADR-011

- Yocthan : voir ADR-011, 012, 013 et 006 pour savoir ce qu'il y a à décider ; les réunir dans un seul fichier, ADR-011, et supprimer les autres.
- Fait : `docs/02-gouvernance/adr/ADR-011-rendu-par-vue.md` contient tout, sans rien perdre, en quatre parties (A : le rendu par vue ; B : les ponts, ancienne 012 ; C : les deux étages, ancienne 013 ; D : HoloIR, ancienne 006), avec en tête un tableau de ce qu'il y a à décider et la recommandation de Claude : valider A, ne pas construire B, valider la direction de C, laisser D en proposition. Chaque partie garde son statut jusqu'à la décision de Yocthan. Les fichiers 006, 012 et 013 sont supprimés ; le registre pointe vers les parties ; les numéros cités ailleurs restent compréhensibles.

---

## 2026-10-06 — Lot 1 : la langue et le partage, la vidéo, le tableau, le texte long, le choix

- Yocthan : construire tout ce qu'on a décidé d'ajouter. 48 éléments restaient (14 en priorité, 34 utiles) ; ils se construisent par lots. Le lot 1 prend les urgences sans choix d'architecture (`ADR-038`, à l'essai).
- Fait : `Page(lang:, description:, image:)` (repris dans l'en-tête par le serveur) ; `alt` obligatoire sur `Image` (`alt: ""` pour un décor) ; `Video(source:, label:)`, jamais lancée seule ; `Table(caption:, head:, rows:)` ; `Input(…, lines:)` pour un texte long ; `Choice(value:, label:, options:)`, et `menu: true` pour une liste déroulante. Leçons 40 à 44. Une petite vidéo d'essai fabriquée avec ffmpeg, déjà sur le PC (62 Ko).
- Vérifié dans Chrome : la langue, la description et l'image dans l'en-tête ; la vidéo a ses boutons, ne part pas seule, et joue ; le tableau a sa légende, ses titres et ses lignes ; un texte de deux lignes est gardé tel quel ; un bouton rond et une option de liste changent la valeur. 96 tests.
- Couverture estimée : HTML de 52 % à 63 % ; ensemble de 47 % à 53 %. Restent 6 urgences (l'envoi d'un formulaire, les listes et la répétition, le survol comme signal, l'accessibilité fine) et 34 éléments utiles.

**Erreurs en route**

- La page légère ne réveillait pas le moteur quand on cochait un bouton rond ou choisissait une option : elle n'écoutait que les boutons nommés et l'entrée dans un champ. Corrigé.
- En relisant l'état d'une page, le moteur enlevait les retours à la ligne d'un texte long. Corrigé, avec un test.
- Deux fois, le script d'essai de Claude a coupé sa propre sortie au retour à la ligne, et fait croire à une perte de texte.
- Le script de fusion a refusé la première fois : une sonde de Codex (`proposals/GPT5.6/revue-langage-securite-2026-10-03`) avait une image sans `alt`, et le nouveau refus l'arrêtait avant qu'elle teste le poids déclaré. Claude a seulement ajouté `alt: ""` à cette image ; la sonde vérifie toujours la même chose.

---

## 2026-10-06 — Tout ce qui était à l'essai est validé

- Yocthan : « Qu'est-ce que tu attends pour valider tous ceux qui sont à l'essai ? Que tout passe au vert. » Claude attendait son accord : le statut d'une décision est à lui.
- Validées (`ACCEPTÉ`) : `ADR-025` (conditions, texte), `ADR-027` (saisie, valeurs gardées), `ADR-029` (imports), `ADR-030` (données du serveur), `ADR-032` (formes, comparaison de valeurs), `ADR-033` (site léger), `ADR-034` (mouvement), `ADR-035` (une mécanique refusée, jamais une capacité), `ADR-036` (repères, titres, texte qui grandit, états, superposition). Les 224 mots de HoloCode sont tous décidés.
- Laissées telles quelles, et dites à Yocthan : `ADR-011` (le rendu par vue : construit, mais c'est un choix d'architecture), `ADR-012` et `ADR-013` (les ponts vers JavaScript et les modules enfermés : jamais construits), `ADR-006` (une proposition).

---

## 2026-10-06 — Le grand tableau : HoloCode d'abord, avec tous ses mots

- Yocthan : HoloCode doit venir en premier, avant HTML, CSS et JavaScript, et on doit pouvoir voir tous ses mots-clés, comme ceux du web.
- Fait, sur la page en ligne et dans `docs/01-holocode/TABLEAU-WEB.md` : HoloCode en premier dans le filtre de langage, dans les résumés et dans la première colonne ; une partie nouvelle, « Les mots de HoloCode » : **224 mots** en 106 lignes, rangés par sorte (blocs, paramètres, mots-valeurs, signaux, demandes, styles, unités, le fichier), avec ce qu'ils font, leur équivalent sur le web, et leur état (142 décidés, 82 à l'essai, d'après les fiches de décision).
- Erreur de Claude : le premier résumé disait « 106 sortes de mots » ; c'étaient des lignes, dont certaines regroupent plusieurs mots (`H1, H2, H3`). Corrigé : on compte les mots.

---

## 2026-10-06 — L'écriture de Flutter, décidée par Yocthan

- Yocthan : « On peut garder les deux, mais Flutter, c'est la base pour moi. » Claude a signalé que garder les deux ferait deux écritures du même mot, contre l'avis unanime des trois IA, et lui a proposé trois façons ; il a choisi « Flutter seul + correction » (`ADR-037`, accepté).
- Fait : un nom de valeur s'écrit `appleX` (sans `_`), un nom de bloc `AddSunrise`, une place `topRight`. Une ancienne écriture est refusée avec le bon mot, y compris entre accolades dans un texte (`{apple_x}` → « écris `{appleX}` »). Exemples, leçons, guide et moteur convertis. Leçon 39. 95 tests ; tous les fichiers `.holo` du dépôt vérifiés.

---

## 2026-10-06 — Les majuscules : les avis de ChatGPT et Gemini ; plus rien d'avalé en silence

- Réponses et synthèse : `docs/05-discussions/reponses/2026-10-06-majuscules-et-casse.md`. ChatGPT, Gemini et Claude s'accordent : respecter la casse, refuser une faute avec le bon mot, corriger d'un clic dans un éditeur, garder `KB`. **Seul désaccord** : joindre deux mots par `_` (Gemini : plus facile sur un téléphone) ou en camelCase (ChatGPT). Claude penche pour `_`. À Yocthan de trancher.
- **Corrigé tout de suite**, puisque les trois avis étaient d'accord : chaque bloc a la liste de ses paramètres, et un paramètre inconnu ou mal écrit est refusé (`Page(Title: …)` → « écris `title` ») ; un nom de bloc doit commencer par une majuscule (`name: buy` → « écris `name: Buy` ») ; `x`/`y` hors d'un plateau et `align` hors d'un `Stack` sont refusés avec la phrase qui dit où les mettre. 95 tests du moteur ; les 79 fichiers `.holo` du dépôt vérifiés : seuls échouent ceux qui doivent échouer (pages piégées exprès, anciennes propositions en français, dont une qui écrivait `titre:` et passait jusqu'ici en silence).
- Le tableau en ligne a une section nouvelle sur l'écriture des mots. Yocthan demande qu'il soit **toujours** tenu à jour.

---

## 2026-10-06 — La boutique comparée, mise à jour des deux côtés

- Demande de Yocthan : actualiser le site qui compare HoloCode à HTML, CSS et JavaScript. Les deux versions de la boutique ont reçu les mêmes ajouts : en-tête et menu, pied de page, pastille « New » sur le tableau, survol et focus des boutons, un titre de niveau 4, des textes qui suivent le réglage du visiteur.
- Vérifié par captures côte à côte, sur ordinateur et sur téléphone : identiques.
- Mesuré : 9 Ko contre 8 Ko téléchargés (579 Ko le 2026-10-04, avant le site léger) ; même premier affichage ; 176 lignes en HoloCode contre 348 en web (90 + 103 + 155).

**Erreurs de Claude**

- Dans la version web, une règle de marges ajoutée était plus « forte » que celles de la grille, du menu et de la pastille : la grille passait à une colonne et la pastille tombait sous l'image. Corrigé avec `:where()`. C'est la cascade que HoloCode refuse, et Claude y est tombé en écrivant le jumeau.
- Le point lumineux était centré côté web et à gauche côté HoloCode depuis le 2026-10-03 ; aligné.
- Le script qui écrivait ce journal s'est arrêté sur une variable réutilisée ; la pull request est partie sans lui. Ajouté à part.

![Les deux boutiques côte à côte (HoloCode à gauche, web à droite)](images/2026-10-06-boutiques-jumelles.png)

---

## 2026-10-06 — Un prompt sur les majuscules et la casse

- Demande de Yocthan : un prompt pour ChatGPT et Gemini sur les majuscules, avec tous les mots du langage, la question « le langage respecte-t-il la casse ? », et le désaccord de Claude.
- `docs/05-discussions/prompts/2026-10-06-majuscules-et-casse.md`. Claude a d'abord essayé dix fautes de casse dans le moteur : huit sont refusées avec le bon mot à écrire ; **deux passent** : `Page(Title: …)` est accepté sans rien dire et le titre est perdu (un défaut), et `Button(name: buy, …)` est accepté avec une minuscule.
- Rien n'est changé avant les réponses et la décision de Yocthan.

---

## 2026-10-06 — Les repères, les titres jusqu'à H6, le texte qui grandit, le survol, la superposition

**Ce que Yocthan a dit** : « Oui, vas-y, commence la construction de tout ce qu'on vient de décider. »

**Fait** (`ADR-035` et `ADR-036`, à l'essai)

- Le principe : refuser une mécanique, jamais une capacité (`ADR-035`).
- Les repères `Header`, `Nav`, `Main`, `Footer`. L'en-tête et le pied posés dans la page sortent du contenu principal, comme il se doit.
- Les titres jusqu'à `H6` (correction d'`ADR-020`).
- Le texte qui suit le réglage « texte plus grand » du visiteur, sans rien écrire : le moteur écrit les `px` d'une taille de texte en `rem` ; un titre de plus de 24px rétrécit sur un écran plus étroit que la page.
- Les états dans un style : `hover: { … }`, `focus: { … }`, `active: { … }` (correction d'`ADR-017`).
- La superposition : `Stack(children: [ … ])` et `align: top_right` sur un enfant.
- Les leçons 35 à 38 ; le site de référence en profite (en-tête, menu et pied en repères, une pastille « Nouveau », un bouton qui réagit au survol). 94 tests du moteur.
- Le grand tableau mis à jour : couverture estimée de 43 % à 47 % (HTML 52 %, CSS 53 %).

**Vérifié dans Chrome** : un en-tête, un menu, un contenu principal et un pied de page, l'en-tête hors du contenu ; la souris sur le bouton change sa couleur, la touche Tab y met une bordure blanche ; la pastille dans le coin de l'image ; un titre de 18px passe à 27px quand le visiteur règle son texte à 24 au lieu de 16 ; le titre de 96px du film fait 58,5px sur un téléphone de 390px et ne déborde plus.

**Erreurs en route**

- La pastille se posait au bord de la page : le `Stack` prenait toute la largeur. Puis, corrigé trop vite, l'image de la carte rétrécissait. Le `Stack` prend maintenant la place qu'on lui donne, et son image la remplit.
- Mon premier essai du survol visait un endroit vide de la page.

![L'accueil avec ses repères et la pastille « Nouveau »](images/2026-10-06-reperes-et-pastille.png)

---

## 2026-10-06 — La réponse de ChatGPT, et la synthèse des quatre avis

- Réponse de ChatGPT : `docs/05-discussions/reponses/2026-10-06-chatgpt-refus.md`. Synthèse : `proposals/Claude/pourquoi-ces-refus-2026-10/SYNTHESE.md`.
- Les quatre avis (humains, Gemini, ChatGPT, Claude) s'accordent sur presque tout : garder les refus de `div`, de la page modifiée à la main, de `position` pour la mise en page et de la cascade ; ajouter `Main`, `Nav`, `Header`, `Footer`, les états (`hover`, `focus`), la superposition en blocs et des tailles de texte qui suivent le réglage du visiteur ; jamais de code libre, mais du calcul enfermé. Trois sur quatre veulent les titres jusqu'à `H6` tout de suite ; Claude se range à cet avis.
- Le principe proposé par ChatGPT : refuser une mécanique, jamais une capacité.
- ChatGPT conteste la règle de Yocthan (suivre la majorité des humains) : sur les forums parlent surtout des développeurs. Sur ces huit refus, les deux méthodes donnent le même résultat.
- Rien n'est construit : Yocthan décide.

---

## 2026-10-06 — Un prompt pour ChatGPT, à la place de Codex

- Le quota de Codex est épuisé ; celui de ChatGPT (conversation simple) ne l'est pas. Yocthan veut un avis rapide, sans travail sur le dépôt : une comparaison et un jugement de nécessité.
- `docs/05-discussions/prompts/2026-10-06-chatgpt-refus.md` : tout est dedans (le projet, le tableau, les huit refus, l'avis de Gemini, ce que disent les humains, la recommandation de Claude), avec la consigne de ne pas travailler.

---

## 2026-10-06 — Les refus face aux avis des humains

**Ce que Yocthan a demandé** : vérifier sur Stack Overflow, Reddit, Twitter et les forums ce que les humains pensent des éléments refusés. Sa règle : si la majorité montre que l'élément manque ou le défend, on lève le refus ; si elle en dit surtout du mal, on le garde.

**Fait** : `proposals/Claude/pourquoi-ces-refus-2026-10/AVIS-DES-HUMAINS.md`, avec ses sources. Trois enquêtes chiffrées (State of CSS 2026, WebAIM 2024, Stack Overflow 2025), des textes d'experts, des discussions de développeurs.

**Résultat selon la règle** : garder les refus de `div`, `section`/`article`, la page modifiée à la main, `position` pour la mise en page, la cascade et `!important`. Lever ceux des repères (`nav`, `header`, `footer`, `main` : 63 % des utilisateurs de lecteurs d'écran s'en servent au moins parfois), de `h4` à `h6`, et de la superposition (en bloc). Assouplir celui de `script` : la plainte contre l'interdiction est forte (AMP abandonné par la plupart des grands éditeurs), mais Claude recommande du code enfermé plutôt que du code libre. Ajouter : des tailles de texte qui suivent le réglage du visiteur.

**Limites dites dans le document** : ce n'est pas un vote mondial ; Twitter n'a pas pu être fouillé directement ; les forums parlent surtout pour des développeurs, pas pour des débutants.

---

## 2026-10-06 — La réponse de Gemini sur les refus

- Gardée dans `docs/05-discussions/reponses/2026-10-06-gemini-refus.md`, avec la lecture de Claude.
- Gemini garde `div`, `script`, la page modifiée à la main et la cascade ; il assouplirait les titres (jusqu'à `H6`), la superposition (un bloc `Stack` ou `Badge`) et les repères (des blocs `Header`, `Nav`, `Main`, `Footer`). Pour le survol : des états écrits dans le style (`hover: { … }`). Refus manquant selon lui : les tailles de texte en pixels.
- Claude change d'avis sur un point : des blocs plutôt qu'un rôle pour les repères, parce qu'un non-programmeur comprend `Nav(…)`.
- On attend Codex, puis Yocthan décide.

---

## 2026-10-06 — Le grand tableau face au web, et pourquoi ces refus

**Ce que Yocthan a demandé**

- Un grand tableau lisible : chaque élément de HTML, CSS et JavaScript, s'il existe en HoloCode, à combien de pourcents, et s'il doit exister. Puis : « Dis-moi pourquoi tu as refusé certains éléments, écris-le sur GitHub, que Codex et Gemini puissent le voir. » Il décidera après leurs avis s'il faut admettre ce qui est refusé.

**Fait**

- Le tableau, 129 éléments : une page à filtrer pour Yocthan, et la même chose dans le dépôt, `docs/01-holocode/TABLEAU-WEB.md`. Couverture moyenne estimée : HTML 47 %, CSS 47 %, JavaScript 32 %, ensemble 43 %. Dix-sept manques mis en priorité (formulaire envoyé, listes, tableaux, vidéo, survol, tailles qui s'adaptent, repères pour lecteurs d'écran…).
- Le pourquoi de chaque refus : `proposals/Claude/pourquoi-ces-refus-2026-10/README.md`. Pour chacun : ce qu'il évite, un exemple, qui l'a décidé, ce qu'il coûte, l'avis de Claude aujourd'hui.
- Deux prompts : `docs/05-discussions/prompts/2026-10-06-gemini-refus.md` (tout est dedans) et `2026-10-06-codex-refus.md`.

**Erreurs de Claude, trouvées en écrivant le pourquoi**

- Le tableau classait `h4` à `h6` « refusés exprès » ; la fiche `ADR-020` dit « on en ajoutera si un vrai besoin apparaît ». Corrigé : « pas encore ».
- Le refus de `section` et `article` n'a jamais été décidé par Yocthan : c'est un jugement de Claude. Le tableau l'attribuait à `ADR-009`. Corrigé, et Claude propose maintenant d'admettre les repères (`nav`, `header`, `footer`, `aside`) pour les lecteurs d'écran.

---

## 2026-10-06 — La page qui attend le moteur (suggestions de Codex)

**Ce que Yocthan a demandé** : « Travaille sur ce qu'a suggéré Codex. » Claude a pris dans ses deux revues ce qui ne demande aucune décision de Yocthan : les cas E02, E03 et F14 de la recette du site de référence.

**La cause commune** : en arrivant, le moteur redessinait toute la page. Une saisie commencée était effacée, le focus perdu, le film relancé.

**Fait** (correction de `ADR-033`)

- Le moteur reprend la page fabriquée par le serveur au lieu de la redessiner ; ce qui a été écrit en l'attendant passe par l'arbitre.
- Un bouton touché pendant l'attente le montre aussitôt ; le signe disparaît quand le toucher est rejoué.
- Si le moteur ne peut pas arriver : un bandeau honnête, aucun toucher rejoué en cachette, un bouton « Réessayer ».
- Le serveur local sait simuler un moteur lent (`HOLO_MOTEUR=lent:5000`) ou en panne (`HOLO_MOTEUR=panne`).

**Vérifié dans Chrome (PC)** : trois touchers pendant 5 s d'attente donnent exactement 3 créations ; « Éloïse 🌍 » tapé avant le moteur reste, focus compris ; la panne est dite ; le film continue. Le site de référence, la boutique et le jeu marchent comme avant.

**Erreur en route** : le premier banc d'essai retardait chacune des trois pièces du moteur (15 s au lieu de 5), et le serveur neuf compressait le moteur pour la première fois : les premières lectures se faisaient avant son arrivée.

![Le moteur en panne : la page le dit](images/2026-10-06-moteur-en-panne.png)

---

## 2026-10-06 — Les deux films du motion design, identiques point par point

**Ce que Yocthan a relevé**

- « Pourquoi les deux showreels ne se ressemblent pas point par point ? Je t'avais dit de construire exactement le même, ligne par ligne ; seule l'extension devait changer. Tu n'as pas rempli les conditions. »

**Erreur de Claude**

- Il avait compris « tous les curseurs à 100 % » comme « chaque langage à son maximum », et avait donné au film web des particules, un morphing en cœur, un cube et la souris que HoloCode n'a pas. La comparaison n'était donc pas à fonctions égales, ce que Codex demande aussi (`C01`).

**Corrigé**

- `exemples/motion/web/showreel.html` est maintenant le jumeau exact du film HoloCode : les 36 éclats sont relus dans le fichier `.holo`, et tout le reste reprend les mêmes valeurs. Vérifié sur sept images prises aux mêmes instants : identiques.
- L'ancienne version est gardée, hors comparaison : `showreel-max.html`.
- Mesures honnêtes : 206 lignes contre 184 ; 14 193 octets contre 21 377 ; 10 Ko téléchargés contre 4 Ko ; 0 ligne de JavaScript contre 7.

**Défaut trouvé en comparant** : quand le moteur arrive dans la page, il la redessine, et le film HoloCode repart de zéro. À corriger.

![Les deux films côte à côte, scènes 1 à 4](images/2026-10-06-motion-jumeaux-1.png)

---

## 2026-10-06 — Le site de référence de Codex, construit

**Ce que Yocthan a demandé**

- « Construis le site d'après les conditions de Codex et lance-le sur Chrome. » Le cahier des charges est la PR 79 de Codex (`proposals/GPT5.6/site-reference-2026-10-06/`), encore ouverte : Claude ne l'a pas fusionnée.

**Fait**

- « L'atelier des mondes », dans `exemples/site-reference/` : accueil (page témoin, ordinaire), catalogue de douze créations, fiche, panier et atelier (le même panier à plat et dans le monde ; un jardin dans un autre fichier ; un monde planté dans un pixel ; points, relief, carrefour), journal, disponibilité (`Data`), jeu (pause, reprise, meilleur score gardé, `within`), galerie animée. Menu et pied de page en morceaux importés. Aucun mot ajouté au langage.
- Premier passage de la recette, sur PC seulement : `exemples/site-reference/RECETTE-2026-10-06.md`. Le parcours du panier donne `(3,330)`, `(2,210)`, `(0,0)`, `(0,0)` ; l'accueil pèse 8 Ko sans le moteur.

**Erreurs en route**

- Un fichier ne peut contenir qu'un seul morceau (`Part`) : le pied de page a dû aller dans son propre fichier, `pied.holo`.
- Les premiers essais automatiques disaient que rien ne marchait : huit Chrome lancés en même temps, et des lectures faites 350 ms après le premier toucher, avant l'arrivée du moteur. Relancés un par un, en attendant le moteur : tout passe.

**Pas fait** : les mesures sur téléphone, les essais avec des débutants, le verdict de Yocthan, et les quatre extensions (formulaire envoyé, listes, objets pleins, jeu à plusieurs), qui restent bloquées.

![L'accueil du site de référence](images/2026-10-06-site-reference-accueil.png)
![Le panier sur un écran de téléphone](images/2026-10-06-site-reference-panier-telephone.png)

---

## 2026-10-04 — Le mouvement en HoloCode, et le duel du motion design

**Ce que Yocthan a demandé**

- Faire du motion design en HoloCode et en HTML, CSS, JavaScript, séparément, « tous les curseurs à 100 % », comme un CV pour être embauché ; puis : « fais une amélioration, sinon HoloCode sera battu ».

**Fait** (`ADR-034`, à l'essai)

- Le mouvement dans le langage : `enter: Enter(…)` (on écrit d'où le bloc part), `loop: Loop(…)` (où il va), `Scenes` et `Scene(for:)`. Dix choses bougent (`opacity`, `x`, `y`, `scale`, `rotate`, `flip`, `tilt`, `blur`, `hue`, `round`), sept caractères nommés (`linear` à `bounce`, dont le ressort), `letters:` et `each:`. Le moteur en fait du CSS : la page bouge sans le moteur.
- Les leçons 32, 33, 34. Guide, noms, comparaison avec le web.
- Deux films en sept scènes : `exemples/motion/holocode/showreel.holo` et `exemples/motion/web/showreel.html`, et leur comparaison dans `exemples/motion/README.md`.
- 93 tests du moteur.

**Le verdict**

- En puissance, le web gagne : 520 particules physiques, morphing en n'importe quelle forme, cube en 3D, réaction à la souris.
- En écriture, HoloCode gagne : environ 100 lignes sans programmer (206 avec les 36 éclats écrits un par un), contre 311 dont 200 de JavaScript. Téléchargé : 10 Ko contre 7 Ko.

**Erreurs en route**

- Dans le film web, les particules filaient hors de l'écran (trop rapides), et des cercles fantômes restaient sur le ciel (le voile sombre ne s'efface jamais tout à fait). Corrigé : vitesses réduites, et on estompe ce qui est dessiné au lieu de peindre du noir par-dessus.
- Un son a un réglage `loop` à lui : le mouvement le prenait pour un `Loop`. Les sons sont laissés hors du mouvement.

![Le Big Bang en HoloCode](images/2026-10-04-motion-holocode-big-bang.png)
![Le morphing en HoloCode](images/2026-10-04-motion-holocode-morph.png)
![Le Big Bang en HTML, CSS, JavaScript](images/2026-10-04-motion-web-big-bang.png)
![Le cube en HTML, CSS, JavaScript](images/2026-10-04-motion-web-cube.png)

---

## 2026-10-04 — Trois avis sur 100 : Claude 50 %, Gemini 65 %, Codex 75 %

- Les réponses de Codex et de Gemini à la même question, et la lecture de Claude : `docs/05-discussions/reponses/2026-10-04-avis-sur-100.md`.
- Tous trois : pas encore les objectifs. Ce qui manque, d'un même avis : un téléphone modeste mesuré dans la durée ; des débutants qui créent seuls ; des objets pleins ; l'envoi d'un formulaire et les listes ; plus tard, le jeu à plusieurs.
- Gemini, sans accès au dépôt, demande des choses qui existent déjà (`Row`, `Column`, le panier à plusieurs articles).
- Codex relève la première image 3D (environ 3,5 s au premier chargement). Le site léger risque de l'allonger : à mesurer.

---

## 2026-10-04 — Le site léger : 8 Ko au lieu de 579

**Ce que Yocthan a demandé**

- « Sur 100 %, le projet te convainc à combien ? Est-ce ultra léger ? » Claude a répondu environ 45 %, et non : la boutique pesait 579 Ko, contre 6 Ko pour sa jumelle en HTML, parce que tout le moteur partait dès l'ouverture. Sa recommandation numéro un : ne télécharger le moteur qu'au besoin. Yocthan : « Oui, vas-y. »

**Fait** (`ADR-033`, à l'essai)

- La page d'entrée est coupée en deux : `page.html` (la page légère, 12 Ko non compressés) et `page-moteur.js` (le moteur de la page, chargé à la demande).
- Le moteur n'est demandé qu'au premier geste qui en a besoin : un bouton nommé, un lien vers un point, un champ, le menu, le zoom. Ce qui a été touché en l'attendant est rejoué ; le menu touché s'ouvre.
- Une page vivante (horloge, clavier, données, glissement) le demande tout de suite : c'est le moteur qui la marque, `data-vivant`. Une page aux valeurs gardées aussi, s'il y a vraiment quelque chose de gardé.
- 91 tests du moteur.

**Mesuré dans Chrome**

| | Avant | Après |
|---|---|---|
| Boutique, à l'ouverture | 579 Ko | **8 Ko** |
| Boutique, après le premier « + » | 579 Ko | 594 Ko (le moteur arrive ; le panier passe à 1) |
| Jeu de la pomme | moteur tout de suite | moteur tout de suite ; la partie se joue comme avant |

- Le menu touché avant l'arrivée du moteur s'ouvre ; « Enter the workshop » touché avant l'arrivée du moteur mène bien à l'atelier.

**Ce que cela coûte**

- Le premier toucher attend le moteur : imperceptible ici, en local ; pas mesuré sur un réseau lent ni sur un téléphone bon marché.
- Une saisie commencée avant l'arrivée du moteur peut être effacée quand il redessine la page.

![La boutique, légère : seul le HTML est téléchargé à l'ouverture](images/2026-10-04-boutique-legere.png)

---

## 2026-10-04 — Un plateau aux proportions fixes

- Yocthan, après la réponse de Gemini : le serveur et le jeu en ligne attendront ; d'abord que le métavers lui plaise, en local. Claude n'ajoute donc aucune bibliothèque réseau : elles ne serviraient qu'au serveur.
- Fait, à l'essai : un plateau garde ses proportions (640 de large, `height` de haut). Il rétrécit sur un téléphone, ses formes et ses points avec lui. Les rencontres se calculent dans les unités du plateau : la même partie partout. La page ne mesure plus la largeur de l'écran.
- Vérifié dans Chrome : sur un écran d'ordinateur (1280 de large), juste avant la prise, le bas de la pomme est 6,4 pixels au-dessus du panier ; sur un écran de téléphone simulé (360 de large), 2,9 pixels. C'est le même écart, à l'échelle du plateau : la partie est la même. Avant « Play », rien ne bouge.
- Ce que cela coûte : sur un téléphone en hauteur, un plateau large laisse du vide en dessous. L'auteur peut choisir un plateau plus haut.
- Codex a rendu sa revue pendant ce temps (PR 74). Quatre défauts qu'il a trouvés sont corrigés ici, sans décision à prendre : le test du guide échouait sur un dépôt aux fins de ligne Windows ; un son demandé lors d'un appel pouvait ressortir avec la réponse d'un glissement, d'une saisie ou de données ; un compte de tirages falsifié au maximum faisait planter le moteur en mode test ; `within` mesure l'écart sur chaque axe, et le guide le dit maintenant.
- Erreur de Claude : la branche était partie d'une copie de `main` pas à jour ; la pull request était en conflit, et les tests en ligne ne démarraient pas. Le script de fusion a attendu dix minutes avant que Claude regarde pourquoi.

![Le jeu de la pomme sur un écran de téléphone : le plateau a rétréci avec ses formes](images/2026-10-04-plateau-telephone.png)

---

## 2026-10-04 — La réponse de Gemini sur les objets en volume et le jeu à plusieurs

- Gardée dans `docs/05-discussions/reponses/2026-10-04-gemini-3d-et-plusieurs.md`, avec la lecture critique de Claude.
- Gemini recommande : des points dont la taille grandit de près (l'objet paraît plein) ; un jeu à plusieurs par WebSocket, sur le modèle de Croquet (le serveur ne calcule rien, il numérote et renvoie les gestes) ; et, avant tout, des rencontres calculées dans des unités de plateau fixes, jamais en pixels.
- Ce dernier point contredit ce que Claude venait de fusionner (la rencontre dépend de la largeur de l'écran). Claude est d'accord avec Gemini.
- Rien n'est construit : on attend la réponse de Codex, puis le choix de Yocthan.

---

## 2026-10-04 — La rencontre se fait au contact ; des règles sous condition ; deux prompts pour Gemini et Codex

**Ce que Yocthan a vu**

- Dans le jeu de la pomme : « Le rond, quand il s'approche du carré, au lieu de le toucher, il pénètre vraiment en profondeur, et après ça déclenche. Il fallait que dès que sa circonférence touche l'un des côtés, ça réagisse. C'est comme ça qu'on construit du bon. »
- Pour les modèles 3D et le jeu à plusieurs : il veut l'avis de Gemini et de ChatGPT avant de choisir, et demande un prompt.

**La cause**

- La rencontre était jugée sur l'écart entre les places des deux objets, sans regarder leur taille ni leur forme. Tant que tout était des points lumineux, cela ne se voyait pas ; avec un rond et un carré, c'est flagrant.

**Corrigé**

- Sans `within`, deux objets se rencontrent au moment où le bord de l'un touche le bord de l'autre. L'arbitre connaît la taille et la forme de chacun et la hauteur du plateau ; la page lui donne la largeur du plateau, qui dépend de l'écran.
- Un test le fixe au pixel près : la pomme (44) est prise quand son bas atteint le dessus du panier (64), pas avant, pas après.
- `within` reste, pour juger sur l'écart entre les places.

**Un second défaut, trouvé en vérifiant à l'écran**

- Le jeu jouait tout seul avant « Play » et après la fin : la pomme, cachée, tombait, était « rattrapée » (un point, un son demandé), puis repartait d'une place au hasard. « Play » ne remettait pas cette place : la première pomme tombait loin du panier.
- Corrigé sans mot nouveau : `If(lives, over: 0, rules: [ … ])`. Le même `If` que pour montrer des blocs, avec `rules` à la place de `children` : les règles rangées dedans ne valent que si la condition est vraie. On y range `Every` et `When`. « Play » remet aussi la pomme et le panier au milieu.
- La leçon 31, `exemples/lecons/31-regles-sous-condition.holo` : un chronomètre qui ne compte que lancé.
- Mesuré dans Chrome, image par image : avant « Play », rien ne bouge et le score reste à 0 ; à l'image qui précède la prise, le bas de la pomme est 6 pixels au-dessus du panier. Elle n'entre plus dedans. (La pomme descend par pas d'environ 9 pixels : la prise se fait au pas où elle touche.)
- 90 tests du moteur.

**Ce que cela coûte**

- La même partie ne se joue plus tout à fait pareil sur un écran large et sur un téléphone. Pour un jeu à plusieurs, il faudra un plateau de taille fixe, ou des unités de monde : question posée à Gemini et à Codex.

**Les prompts**

- `docs/05-discussions/prompts/2026-10-04-gemini-3d-et-plusieurs.md` (sans accès au dépôt, tout est dedans) et `2026-10-04-codex-3d-et-plusieurs.md` (avec le dépôt ; il lui demande aussi de relire les noms ajoutés depuis sa contre-revue).

**Erreur de Claude**

- Claude avait joué la partie par un test qui ne regarde que les nombres : le score montait, donc « ça marchait ». Il n'avait pas regardé à l'écran le moment du contact.
- Le jeu qui joue tout seul était noté dans `ADR-026` comme une limite sans gravité. Avec le son et les rencontres, c'était un défaut, et Claude ne l'avait pas relu.

---

## 2026-10-04 — Les formes, le meilleur score ; deux propositions pour ce qui reste

**Ce que Yocthan a dit**

- Le son : « J'ai kiffé. » Puis : « Fais ce qui reste à faire. »

**Fait** (`ADR-032`, à l'essai ; `ADR-031`, le son, passe à « accepté pour l'instant »)

- **Les formes** : `Shape(form: circle, color: "#E9B44C", size: 48px)`. Quatre formes : rond, carré, triangle, losange. Une forme nommée se touche, se place sur un plateau, se fait glisser, et peut être guettée par une rencontre. Dans le jeu de la pomme, la pomme est un rond et le panier un carré.

![La leçon 30 : quatre formes, et deux formes sur un plateau](images/2026-10-04-formes.png)

- **Comparer deux valeurs, fixer d'après une autre** : là où l'on écrit un nombre, on peut écrire le nom d'une valeur. `When(score, over: best, effect: best.set(score))`. Le jeu de l'étoile garde le meilleur score d'une visite à l'autre.
- Deux leçons : la 29 (comparer deux valeurs) et la 30 (les formes).
- 90 tests du moteur. Vérifié dans Chrome : le record suit le score dès qu'il est dépassé, et reste après rechargement.

**Ce que Claude n'a pas construit, et pourquoi**

- **Les modèles 3D** et **le jeu à plusieurs**. Ce sont les deux derniers morceaux des étapes 6 et 7, et les deux choix d'architecture les plus lourds depuis le choix de Rust : un second moteur de dessin ou non ; un serveur qui tourne en permanence, avec la question de qui il laisse faire quoi. Yocthan a donné le champ libre pour construire ce qui a été recommandé et expliqué ; ces deux-là ne l'ont pas encore été. Claude les a posés dans `proposals/Claude/modeles-3d-et-jeu-a-plusieurs-2026-10/`, avec trois options chacun et une recommandation.

---

## 2026-10-04 — Étape 6, premier morceau : le son

**Fait** (`ADR-031`, à l'essai)

- `Sound(name: Ding, source: "ding.wav")` : un son, qui ne se voit pas. `Ding.play` le fait entendre, dans l'effet d'une règle, seul ou dans une liste. Les trois sortes de règles peuvent jouer un son.
- Une règle de temps ou une règle qui guette ne peut demander que cela hors des demandes : `Workshop.enter` y est refusé (on n'emmène pas le visiteur ailleurs sans geste).
- Les deux jeux ont leurs sons : un quand on attrape, un quand une pomme est perdue. Les trois petits fichiers de son ont été fabriqués par un script (deux notes qui s'éteignent, sept à onze kilo-octets).
- La leçon 28, `exemples/lecons/28-son.holo`.
- 88 tests du moteur. Vérifié dans Chrome avec un vrai clic : un appui, un son ; dans le jeu de la pomme, « Pop » à la pomme rattrapée, « Lost » à la pomme perdue.

**Erreur en route**

- Les premiers essais automatiques ne comptaient aucun son joué. Le moteur n'y était pour rien : un navigateur ne joue un son qu'après un vrai geste, et un clic déclenché par un script n'en est pas un. Refait avec un clic de souris simulé par le navigateur lui-même.

**Ce qui reste de l'étape 6, et l'étape 7**

- Les formes, les images dans la vue en profondeur, les modèles 3D : c'est le gros de l'étape 6. Le moteur de dessin ne sait aujourd'hui dessiner que des points ; il faut l'étendre. Claude ne l'a pas commencé.
- L'étape 7 (partie gardée, jeu à plusieurs avec un arbitre sur un serveur) n'est pas commencée.

---

## 2026-10-04 — Étape 5, fin : les données venues du serveur

**Ce que Yocthan a dit**

- « On y va. »

**Fait** (`ADR-030`, à l'essai)

- `data: Data(from: "stock.json", every: 30s)` : la page va chercher un fichier de données rangé à côté d'elle, à l'ouverture puis à un rythme. Le fichier est un objet JSON à plat ; chaque nom remplit la valeur de `State` du même nom.
- C'est l'arbitre qui range : une valeur déclarée, de la bonne sorte, dans ses bornes. Le reste est laissé de côté ; un fichier mal formé ne change rien. Le lecteur du fichier est écrit dans le moteur, sans bibliothèque.
- La page ne parle qu'au serveur d'où elle vient.
- La leçon 27, `exemples/lecons/27-donnees.holo`.
- 87 tests du moteur. Vérifié dans Chrome : le fichier `.json` changé sur le disque, la page est passée de « Ouvert aujourd'hui jusqu'à 18 h » à « Fermé pour la soirée » en deux secondes, sans rechargement.

**Ce que l'étape 5 ne donne pas**

- Les listes : on ne reçoit pas « tous les articles du catalogue ».
- L'envoi : la page n'envoie rien au serveur.
- Ces deux manques pèsent sur la cible « 70 % de SolidJS » : Claude l'estimerait plutôt à 60 % après cette étape.

---

## 2026-10-04 — Les leçons : un fichier par notion

**Ce que Yocthan a dit**

- « À chaque étape, à chaque nouveau code injecté, qu'il y ait un fichier qui explique exactement une seule chose. » Il aurait voulu le demander dès le départ. « Dans le guide, vu que c'est dense, comprendre sera compliqué. » Il demande aussi de le faire pour tout ce qui existe déjà, de la première notion à aujourd'hui.
- Continuer avec les étapes 5, 6 et 7, lui envoyer les corrections à voir, et ouvrir dans Chrome ce qui s'exécute.

**Fait**

- `exemples/lecons/` : vingt-six leçons, de la première page au Big Bang. Chaque leçon est un petit fichier `.holo` : les commentaires du haut disent ce qu'on apprend et quoi essayer, le code le montre, et un lien en bas mène à la leçon suivante. On les suit dans Chrome et dans VS Code côte à côte.
- L'index est `exemples/lecons/README.md`. L'accueil des démonstrations y mène.
- Un test du moteur vérifie toutes les leçons : une leçon qui ne marche plus fait échouer les tests.
- La règle est notée dans `AGENTS.md` : chaque notion ajoutée reçoit sa leçon, dans la même pull request.

**Ce qui n'est pas fait**

- Les étapes 5 (fin), 6 et 7 : elles restent à faire, une par une.

---

## 2026-10-04 — Étape 5, première moitié : les imports, et un site de deux pages

**Ce que Yocthan a dit**

- Le jeu de la pomme : « c'est bon ». Il n'avait pas vu que le second onglet ouvert était le jeu de l'étoile, déjà joué.
- Codex : attendre qu'il ait fini sa réflexion sur les noms (son quota revient vers 13 h 40), puis trancher l'ensemble.
- Ensuite, l'étape 5.

**Trouvé dans le dossier de Yocthan**

- La contre-revue de Codex sur les noms, `proposals/GPT5.6/autocritique-noms-2026-10-04/README.md`, ajoutée mais pas encore poussée. Claude ne l'a pas touchée. Codex y retire presque toutes ses propositions et en garde trois (`divisions`, `levels` dans `Points`, `anchor`).
- Une espace en trop dans `exemples/jeu/attraper.holo` bloquait la mise à jour du dossier : retirée, copie gardée.

**Fait** (`ADR-029`, à l'essai ; `ADR-028` passe à « accepté pour l'instant »)

- **Les imports.** Un fichier importé est un morceau : `Part(name: Menu, children: [...])`, avec ses styles. La page écrit `import "commun.holo"` en haut, puis `Use(Menu)` là où elle veut le morceau. Les styles du morceau viennent avec lui ; si la page écrit le même style, le sien reste.
- **Un site de deux pages**, `exemples/site/` : le menu et le thème sont écrits une fois, dans `commun.holo`.

![La page de contact : le menu vient de commun.holo, la couleur du titre est la sienne](images/2026-10-04-site-deux-pages.png)

- Le moteur ne lit toujours aucun fichier lui-même : la page d'entrée et le moteur en ligne de commande vont chercher les imports et les joignent au texte.
- 85 tests du moteur. Vérifié : `holo check`, la page fabriquée d'avance par le serveur, et Chrome.

**Ce qui n'est pas fait de l'étape 5**

- Les données venues d'un serveur.

**Limites**

- Un morceau n'a ni paramètres, ni valeurs, ni règles.
- Une erreur dans un morceau est signalée à la ligne de l'`import`.

---

## 2026-10-04 — Des règles plus courtes, une sorte de règle en moins, et le glissement

**Ce que Yocthan a dit**

- « À chaque fois qu'il y a un truc à exécuter, tu vas l'exécuter sur Chrome. Tu as ma permission. » Trop de messages à répéter sinon.
- Finir l'étape 4 avec le glissement.
- Sur les quatre sortes de règles : « Il faudra que tu puisses voir comment les diminuer. Plus c'est verbeux, plus on s'éloigne de l'objectif du langage, qui est de faire des trucs de manière simple. Quand c'est utile, c'est normal que ça soit verbeux ; quand c'est pas utile, il faut chercher une manière de faire correctement la chose. »

**Fait**

- **Plusieurs demandes dans une règle**, entre crochets : `On(Play.tap, effect: [score.set(0), lives.set(3), apple_y.set(0)])`. Le jeu de la pomme passe de treize règles à six ; celui de l'étoile, de neuf à cinq ; « vider le panier », de trois à une.
- **Une sorte de règle en moins.** `Meet` disparaît : une rencontre s'écrit `When(Basket, meets: Apple, within: 9, effect: …)`. Il reste `On` (un geste), `Every` (le temps), `When` (un moment).
- **Le glissement** : `drag: true` sur un bloc d'un plateau. Sa valeur suit le doigt ou la souris ; rattraper la pomme en glissant compte. Les deux boutons du jeu ne servent plus : retirés.
- Les fichiers `.holo` à essayer sont désormais ouverts dans Chrome sans attendre qu'il le demande.

**Erreur de Claude**

- La première version de l'étape 4 ajoutait un mot par besoin (`When`, puis `Meet`) et une ligne par demande. Claude l'avait signalé comme un risque sans le corriger ; c'est Yocthan qui a tranché.

---

## 2026-10-04 — Étape 4 : le clavier, les règles qui guettent, les rencontres, et un deuxième jeu

**Fait** (`ADR-028`, à l'essai)

- **Un deuxième jeu sans code** : `exemples/jeu/panier.holo`. Une pomme tombe ; on la rattrape avec un panier, au clavier ou par deux boutons ; trois pommes perdues, la partie est finie.

![Le deuxième jeu : la pomme et le panier](images/2026-10-04-deuxieme-jeu.png)

- Ce que ce jeu a demandé au langage :
  - **`Key`** : le clavier, dans une règle `On`. Les flèches et l'espace.
  - **`When(apple_y, over: 99, effect: …)`** : une règle qui guette. Elle se déclenche au moment où la condition devient vraie.
  - **`Meet(Basket, Apple, within: 9, effect: …)`** : la rencontre de deux blocs posés sur un plateau.
  - Une valeur qui sert de place reste entre 0 et 100 : le panier ne sort pas.
- Faire tomber la pomme n'a demandé aucun mot : `Every(100ms, effect: apple_y.add(3))`.
- 84 tests du moteur ; la partie y est rejouée battement par battement. Joué dans Chrome avec de vraies touches : le panier va à droite et revient, la pomme rattrapée donne un point, la suivante, manquée, coûte une vie.

**Ce qui n'est pas fait dans l'étape 4**

- Le glissement du doigt.

**Ce qui inquiète**

- Le langage a maintenant quatre sortes de règles : `On`, `Every`, `When`, `Meet`. Et une règle ne fait qu'une demande, donc rattraper la pomme prend trois lignes. Si un troisième jeu demande encore une nouvelle sorte de règle, il faudra chercher une forme plus générale plutôt que d'ajouter un mot.

---

## 2026-10-04 — Étape 3, fin : les valeurs de texte et le champ de texte

**Ce que Yocthan a dit**

- Le jeu : « J'ai fait un score de 7, donc ça marche vraiment. » Puis : « Tu termines l'étape 3, et après on passe à l'étape 4. »

**Fait** (`ADR-027`, à l'essai)

- Une valeur peut être un texte : `State(buyer: "")`. `{buyer}` le montre. `If(buyer, is: "")` et `If(buyer, not: "")` disent s'il est vide ou rempli.
- `Input(value: buyer, label: "…", max: 20)` : le même bloc que pour un nombre. Comme la valeur est un texte, le champ est un champ de texte.
- Un texte se garde par `keep`, comme un nombre.
- La boutique demande le prénom de l'acheteur et écrit « This order is for … ». Sa jumelle web aussi.
- 83 tests du moteur. Vérifié dans Chrome avec « Zoé <b>&; Arc » : montré lettre pour lettre, aucune balise créée, retrouvé après rechargement.

**Choix**

- Un seul bloc `Input`, dont le genre vient de la valeur. En HTML, `input` a vingt-deux genres.
- Un texte ne se compare qu'au vide. Comparer deux textes appellerait vite « contient », « commence par », puis des expressions.

**Erreur en route**

- Un test attendait qu'un prénom de treize caractères passe dans un champ borné à douze. C'est le test qui avait tort : le moteur coupait bien.

---

## 2026-10-04 — Le jeu corrigé et validé ; étape 3 : la case, le champ, les valeurs gardées

**Ce que Yocthan a dit, après avoir joué**

- « Techniquement, c'est pas mal. Je valide les jeux. » Mais il n'a pas pu gagner une partie : « on commence avec un petit handicap », et « l'étoile est ultra rapide sur PC ». Sur téléphone, toucher est un seul geste ; à la souris, il en faut deux.
- « Tu corriges, ensuite tu fais l'étape 3. »

**Le jeu, corrigé** (`ADR-026`, qui passe à « accepté pour l'instant » sur sa décision)

- Claude avait d'abord joué une partie entière automatiquement dans Chrome : 42 touchers, l'étoile jamais hors du plateau, écran de fin, rejouer. C'est là qu'il a vu la première seconde trop courte. Mais un test automatique vise sans effort : il n'a pas vu que le jeu était trop dur pour une main.
- Chaque règle `Every` a maintenant sa propre horloge, et quand un geste change une valeur, l'horloge de cette valeur repart de zéro. Mesuré dans Chrome : après « Play », 30 à 0,8 s, 29 à 1,2 s.
- L'étoile bouge toutes les deux secondes, et reste deux vraies secondes là où elle arrive après un toucher.

**Étape 3, première moitié** (`ADR-027`, à l'essai)

- **`Checkbox(value: gift, label: "…")`** : une case ; cochée, la valeur vaut 1.
- **`Input(value: tip, label: "…", max: 50)`** : un champ où l'on écrit un nombre. L'étiquette est obligatoire. C'est l'arbitre qui change la valeur, et il la borne.
- **`keep: [noms]`** : les valeurs que le navigateur du visiteur garde. On recharge, le panier est encore là.
- La boutique : une case « Gift wrap » et un pourboire, qui apparaissent dès que le panier n'est plus vide. Sa jumelle web fait de même, avec `localStorage` écrit et relu à la main.

![La case cochée et le champ, borné à 50](images/2026-10-04-case-et-champ.png)

- 82 tests du moteur. Vérifié dans Chrome : 999 écrit dans le champ devient 50 ; après rechargement, la case, le pourboire et le panier sont encore là.

**Ce qui reste de l'étape 3**

- Le champ de texte (un nom, une recherche). Il demande des valeurs qui soient du texte : c'est le prochain morceau.

**Limite vue**

- La page arrive du serveur avec les valeurs de départ, puis prend les valeurs gardées : on peut voir « 0 » un instant.

---

## 2026-10-04 — Étape 2 du planning : le temps, le hasard, le plateau, et un premier jeu

**Fait** (`ADR-026`, à l'essai)

- **Un jeu entier, sans une ligne de code** : `exemples/jeu/attraper.holo`. On touche une étoile le plus de fois possible en trente secondes ; elle change de place chaque seconde, et chaque fois qu'on la touche. Soixante lignes, commentaires compris.

![Le jeu en cours : score 2, 26 secondes](images/2026-10-04-premier-jeu.png)

- Trois mots de plus, ceux que le jeu demandait :
  - **`Every(1s, effect: …)`** : une règle qui se répète. L'horloge se tait quand on ne regarde pas.
  - **`random`** : `star_x.random(100)` tire un nombre de 0 à 100. Le hasard est rejouable : une suite fixée par le nom de la page.
  - **`Board`** : un plateau où l'on place un bloc par `x` et `y`, de 0 à 100. Si `x` est le nom d'une valeur, le bloc la suit.
- Commencer et finir n'ont demandé aucun mot : le temps ne descend pas sous zéro, et les conditions montrent le jeu ou l'écran de fin.
- 81 tests du moteur. Joué dans Chrome : « Play », 30 secondes ; l'étoile bouge ; deux touchers, score 2.

**Limites**

- Les règles `Every` tournent tant que la page est ouverte, même le jeu fini : elles demandent, et rien ne change.
- Pas de meilleur score : on ne compare pas deux valeurs.
- L'étoile saute d'une place à l'autre : pas de mouvement continu.
- `x` et `y` écrits hors d'un plateau sont ignorés en silence. À refuser.

---

## 2026-10-04 — Les conditions décidées à un seul endroit

**Ce que Yocthan a dit**

- Sur la limite signalée (les conditions calculées à deux endroits) : « Tu réunis, tu fais ce qu'il y a à faire, et après on passe au point 2. »

**Fait**

- Une condition n'est plus décidée qu'à un endroit : dans le moteur (`etat::conditions`), au premier affichage comme après chaque changement. La page d'entrée ne compare plus rien : elle demande au moteur et cache ce qu'il dit faux.
- Au passage : l'aperçu d'un autre fichier, dans un portail, montre ses propres valeurs et ses propres conditions, et non celles du fichier où l'on est.

---

## 2026-10-04 — Étape 1 du planning : les conditions, et le texte qui manquait

**Ce que Yocthan a dit**

- Sur les cibles et le planning en sept étapes : « Oui, commence. »

**Fait** (`ADR-025`, à l'essai)

- **Les conditions** : `If(count, is: 0, children: [...])`. Quatre comparaisons, en mots : `is`, `not`, `over`, `under`. Plusieurs valent ensemble. Pas de « sinon » : on écrit une seconde condition.
- **Le texte** : `Hr()` (un trait), `Quote("…", by: "…")` (une citation), `Code("…")` (du texte tel quel), les accents graves dans une phrase, le retour à la ligne gardé dans un texte entre trois guillemets, `Image(alt:)`.
- La boutique : « Your cart is empty » disparaît au premier ajout ; un message annonce la livraison offerte, et un code apparaît à partir de 300 euros. Sa jumelle web fait de même, en montrant et cachant à la main.

![Le panier à 420 euros : le code de livraison est apparu](images/2026-10-04-conditions.png)

- 78 tests du moteur. Vérifié dans Chrome : les quatre conditions de la boutique changent bien au fil des ajouts.

**Ce qui n'est pas fait, alors que c'était annoncé**

- **Les listes répétées.** Répéter un bloc pour chaque article demande des valeurs qui soient des listes ; nous n'avons que des nombres. Claude l'avait mis dans l'étape 1 sans voir cette dépendance. Reporté.
- `alt` reste facultatif : l'obliger casserait la suite de conformité partagée avec les prototypes des autres IA.

**Limites**

- Les conditions sont évaluées deux fois : en Rust au premier affichage, en JavaScript ensuite. Deux copies d'une même règle peuvent diverger.
- On ne compare qu'à un nombre écrit dans le fichier. Pas de « ou ».

**Erreurs en route**

- Une ligne rangée dans une condition s'affichait en colonne : la règle de style de la condition passait après celle de la ligne. Vu sur la capture, corrigé.
- Le texte tel quel avait deux fonds superposés. Vu sur la capture, corrigé.

---

## 2026-10-04 — Les cibles par famille et un planning en sept étapes

**Ce que Yocthan a demandé**

- Se rapprocher au maximum dans chaque famille, avec un équilibre : « HoloCode ne cherche pas seulement à être un langage, mais un langage qui permet de faire à la fois du web et du jeu dans le web. Donc, en gros, un métavers. » Il évoque les mondes créés à partir d'une vidéo. Il veut la cible pour chaque famille et un planning pour l'atteindre vite.

**Proposé par Claude** (dans `docs/01-holocode/COMPARATIF-CONCURRENTS.md`)

- Cibles : 70 % de SolidJS pour le web, 50 % de Three.js pour la 3D, 30 % de Roblox et 10 % d'Unreal pour le jeu, 40 % de Rust sans le chercher.
- Sept étapes, une trentaine de séances : conditions et listes ; le temps, le hasard et un premier jeu ; formulaires ; objets qui bougent ; imports et données ; formes, images, son et modèles ; partie gardée et jeu à plusieurs.
- Les mondes tirés d'une vidéo ne sont pas dans le planning : le calcul se fait sur des serveurs ; HoloCode pourrait en accueillir le résultat après l'étape 6.

**Rien n'est décidé ni construit.** L'ordre des étapes attend l'accord de Yocthan.

---

## 2026-10-04 — Le garde-fou de fusion avait un trou

**Erreur**

- La pull request 56 (un document) a été fusionnée alors qu'un de ses cinq tests n'était pas fini : le script a affiché « 5 terminés, tous verts » avec seulement quatre résultats. Le test est passé ensuite, et `main` est au vert. Mais le garde-fou n'a pas tenu son rôle.

**La cause**

- `outils/fusionner.sh` lisait l'état des tests en trois appels séparés. Le dernier test s'est terminé entre deux lectures : compté « terminé » par l'une, sans résultat pour l'autre, et un résultat vide n'était pas refusé.

**Corrigé**

- Une seule lecture par tour. Un test sans résultat compte comme « en cours ». La fusion est refusée tant qu'il en reste un.

---

## 2026-10-04 — HoloCode face aux meilleurs de chaque famille ; ce qui manque pour un jeu

**Ce que Yocthan a demandé**

- La liste des frameworks et des langages concurrents de JavaScript, les plus rapides, proches ou non du métavers. Puis : prendre le meilleur de chaque famille, l'aligner, et dire à combien de pour cent HoloCode s'en approche ou le dépasse. Il a relevé qu'Unreal manquait.
- « Il faudra qu'on pense à créer des outils, ou à rajouter des mots dans le langage, pour pouvoir construire un vrai jeu à 100 % sur HoloCode. »

**Fait**

- `docs/01-holocode/COMPARATIF-CONCURRENTS.md` : HoloCode face à SolidJS (35 %), Rust (30 %), Three.js (20 %) et Unreal (3 %), critère par critère. Ce sont des jugements de Claude, pas des mesures, sauf deux lignes. Conclusion : HoloCode ne gagne nulle part sur la puissance, partout sur la facilité, et il est seul à faire d'un même fichier un site lisible et un monde.
- Dans le même document : les dix choses qui manquent pour écrire un jeu, et une méthode (un premier jeu très petit, qui ne demande que les conditions, le temps et le hasard).

**Rien n'est construit.** Le premier jeu et ses mots attendent le choix de Yocthan.

---

## 2026-10-04 — Yocthan valide le panier et la disposition ; première comparaison de vitesse avec le web

**Ce que Yocthan a dit**

- Il a essayé : le bouton unique (« c'est pas mal, j'ai bien aimé »), le Big Bang (« ça fonctionne comme prévu »), le panier (« ça fonctionne »).
- Sur ce qui était à l'essai : « Oui, j'ai kiffé. Les paniers, les états, tout ça. Valide-le pour l'instant. »
- Ensuite : comparer avec d'autres sites, « histoire de voir si le langage est plus rapide que la majorité des langages ou pas ».
- Le téléphone : déjà essayé deux fois aujourd'hui, inutile d'insister. « Si quelque chose passe, ça veut dire que ça marche. » Un téléphone d'entrée de gamme : pas avant trois mois.
- Les trois personnes : plus tard. Pour l'instant, le cycle reste entre lui et les IA.
- Codex : il attend le retour du quota. Il va dormir.

**Fait**

- `ADR-023` (le panier) et `ADR-024` (la disposition) passent d'`EXPÉRIMENTATION` à « `ACCEPTÉ` pour l'instant », sur sa décision. L'écriture reste à revoir avec les noms.
- Première mesure, boutique en HoloCode contre la même en HTML, CSS et JavaScript, dans `exemples/boutique-comparee/README.md`. Résultat dit sans détour : HoloCode n'est pas plus rapide (premier affichage à peu près égal), il est près de cent fois plus lourd à la première visite (le moteur, 570 Ko, téléchargé une fois), et l'auteur écrit deux fois moins de lignes, sans JavaScript.

**À ne pas refaire**

- Ne plus redemander le téléphone à chaque message. Yocthan le rebranchera quand il voudra une mesure.

**Suite possible**

- Comparer avec React, Vue ou Svelte demande de les installer : à lui de le dire.

---

## 2026-10-04 — Un seul bouton pour les outils du moteur ; un prompt pour Codex sur les animations

**Ce que Yocthan a dit, après avoir essayé les sept fichiers**

- Les animations du salon et du jardin sont presque celles qu'il cherchait. « Après révision, je me suis rendu compte que c'est moi qui avais tort. C'est pas vraiment mal, mais il y a quand même des changements à faire. » Il veut faire évaluer les animations par ChatGPT (Codex) et avoir sa proposition.
- Il y a trop de boutons, partout. Il en veut un seul, en bas à droite, « comme une superposition sur Android », « comme les menus dans les jeux » : on appuie, les autres apparaissent, et une croix referme.

**Fait**

- Les outils du moteur (Vue points, Carrefour, Tourner, De face) sont rangés derrière un seul bouton rond, en bas à droite. On appuie : ils apparaissent au-dessus ; le bouton devient une croix. Échap referme aussi. Changer de vue ou ouvrir le carrefour referme le menu ; « Tourner » le laisse ouvert, pour garder « De face » sous la main. Seuls les outils utiles à la page sont listés : sur le salon, il n'y a que « Carrefour ».

![Le menu ouvert sur la boutique](images/2026-10-04-menu-unique.png)

- Le prompt pour Codex : `docs/05-discussions/prompts/2026-10-04-codex-animations.md`.

**Ce qui n'est pas fait**

- La porte du Big Bang (`index.html`) garde ses deux boutons, « pause » et « mesures ».
- Le bouton rond recouvre le coin du site : un site qui a quelque chose à cet endroit sera gêné. À voir avec l'avis de Codex.

---

## 2026-10-04 — Un bouton est un lien : un passage ordinaire, pas un point qui s'ouvre

**Ce que Yocthan a dit**

- Le site fait ce qui était prévu, sauf « Enter the workshop » : l'animation part du point lumineux sous le bouton, pas du bouton, et grossit comme si elle absorbait le visiteur. « Quand tu entres par une porte, est-ce que tu as besoin d'une animation bizarre ? Par défaut, il faut des animations normales. » « C'est même pire que le web ancien. » « Il y a certains endroits où ça doit fonctionner comme le web normal, et d'autres où ça doit fonctionner comme le métavers. Là, tu chamboules tout. C'est un désordre, visuellement. »

**L'erreur de Claude**

- En retirant le carrefour, Claude a fait partir du point l'animation d'un clic sur le bouton. Le visiteur appuie à un endroit et voit quelque chose bouger ailleurs : il ne peut pas comprendre. Et un bouton qui mène ailleurs est un lien ; il n'a pas à déclencher un effet de métavers.

**Corrigé**

- Par un **bouton** : un passage ordinaire, comme sur le web. La page s'efface, la suivante apparaît, en un tiers de seconde. Pareil pour revenir (« Back to the shop »).
- En touchant **le point lui-même** : le point s'ouvre où il est et grandit. C'est là qu'on est dans le métavers, et le mouvement part de ce qu'on a touché.
- Avec les animations réduites, le changement est immédiat.

**À retenir**

- Le mouvement part toujours de ce que le visiteur a touché.
- Web normal par défaut ; l'effet de métavers seulement là où le visiteur agit sur un objet du métavers (un point, le zoom).
- Yocthan dit qu'une version de la veille, « vers 2 heures », était parfaite. Claude ne sait pas laquelle ; à lui demander ce qu'elle faisait.

---

## 2026-10-04 — Entrer dans un point y mène directement

**Ce que Yocthan a vu**

- En appuyant sur « Enter the workshop », il y a bien une transition, mais il arrive sur le carrefour au lieu d'arriver dans l'atelier. « Le carrefour, c'est utile quand l'utilisateur a choisi de mettre d'autres mondes comme destination. Quelqu'un qui est déjà habitué au web ne va pas trouver ça normal. »

**Corrigé**

- Entrer dans un point mène directement au site : le point s'ouvre là où il est sur la page, grandit jusqu'à remplir la fenêtre, et devient le site. Pareil pour un point touché, un point planté dans un pixel, et un point qui mène à un autre fichier du même serveur.
- Le carrefour ne s'ouvre plus que sur demande : son bouton, ou une règle `portals`.
- Exception gardée : vers le fichier d'un autre serveur, on passe encore par le carrefour, qui affiche le nom du serveur et attend un clic. C'est une protection demandée par Codex.
- Guide et `ADR-022` mis à jour.
- Vérifié dans Chrome : de la boutique à l'atelier, et du salon au jardin (un autre fichier), sans carrefour. Le panier suit.

![L'atelier, atteint directement, avec le panier](images/2026-10-04-atelier-directement.png)

**Erreur de Claude**

- Ce passage obligé par le carrefour datait du 3 octobre. Il mélangeait deux choses : aller quelque part, et choisir où aller. Claude ne l'avait pas vu comme un défaut.

---

## 2026-10-04 — Big Bang : le point nous absorbe, au lieu de disparaître

**Ce que Yocthan a vu**

- Il a demandé d'ouvrir tous les sites pour les essayer. Dans le Big Bang : « Le point devrait s'agrandir et nous faire immerger à l'intérieur. Ici, en grossissant, au lieu de nous absorber, il disparaît. C'est un gros problème. »

**La cause**

- On entrait dans le point visé à un zoom fixe, alors qu'il n'occupait encore qu'un quart de l'écran. L'image était alors remplacée d'un coup par le nouveau monde, dessiné comme un petit point entier. Le point n'avait pas le temps de nous entourer : il sautait.

![Avant : juste avant l'entrée, le point visé n'occupe qu'un quart de l'écran](images/2026-10-04-big-bang-avant-correction.png)

**Corrigé**

- On entre quand le point visé déborde de l'écran de tous les côtés.
- Le monde qu'il contient se voit dedans et grandit avec lui ; ses points sont placés exactement là où le nouveau monde les dessine une fois entré. Un test le vérifie : aucun point n'apparaît d'un coup.
- La couleur du point, qui remplissait l'écran, se dissipe autour de nous en moins d'une demi-seconde.
- En ressortant, on retrouve le point quitté à la taille qu'avait son monde, refermé.

![Juste avant d'entrer : le point remplit l'écran, son monde grandit dedans](images/2026-10-04-big-bang-absorbe.png)

![Juste après : les mêmes points, aux mêmes places](images/2026-10-04-big-bang-dedans.png)

- 76 tests du moteur.

**Ce qui change par ailleurs**

- Il faut zoomer plus longtemps pour entrer dans un point (environ une fois et demie plus).
- Le téléphone n'était plus détecté par le câble : les sites n'ont été ouverts que sur l'ordinateur.

---

## 2026-10-04 — Le mode de secours mesuré sur le Flip 3

**Fait**

- Yocthan a déverrouillé le Flip 3. Mesure en WebGL 2, forcé par `?webgl` : 60,2 images par seconde en zoomant dans le Big Bang, 59,8 en vue points sur la boutique (image la plus lente : 33 ms), entrée en vue points en 318 ms, 86 Mo pour l'onglet. Détail dans `moteur/README.md`.
- Le réglage « rester allumé sur USB » a été activé pour la mesure, puis remis.

- Corrigé au passage, le défaut noté plus bas : un bouton touché avant que le moteur soit prêt n'est plus perdu. Le toucher est noté, puis rejoué dès que le moteur est là.

**Ce que cela dit, et ne dit pas**

- Le chemin de secours marche et n'est pas plus lent que WebGPU sur ce téléphone.
- Le Flip 3 reste un téléphone puissant : la moitié du risque relevé par Gemini est levée, pas l'autre. Un téléphone d'entrée de gamme reste à mesurer.

---

## 2026-10-04 — Le panier a des prix et un total

**Ce que Yocthan a dit**

- « Tu as des champs libres. » Dit une seconde fois, après la proposition de panier.

**Fait**

- Claude a construit l'option qu'il recommandait (la plus petite), à l'essai : `prices: Prices(sunrise: 120, blue_door: 90)` donne le prix de chaque article ; le moteur calcule `{count}` (le nombre d'articles) et `{total}` (ce qu'ils coûtent). L'auteur n'écrit aucun calcul. Détail dans `ADR-023`.
- La boutique : chaque tableau a ses boutons « + » et « - », et le panier affiche le nombre et le total. Sa jumelle en JavaScript fait la même chose, avec une boucle et un affichage remis à jour à la main.

![Le panier : deux tableaux, 270 euros](images/2026-10-04-panier-prix-total.png)

- Vérifié dans Chrome : 120, 240, 390 euros ; retirer un tableau absent ne change rien ; puis 270. 74 tests du moteur.

**Choix faits par Claude, à juger par Yocthan**

- Les mots `Prices`, `{count}`, `{total}`. Avec `prices:`, les noms `count` et `total` sont pris par le moteur ; sans, ils restent libres.
- Un prix est un nombre entier : pas de centimes.

**Limites**

- Les articles sont écrits d'avance dans le fichier. Le panier est un affichage, pas une commande.
- Vider le panier demande une règle par article.
- Trouvé en essayant : depuis que le serveur envoie la page déjà fabriquée, les boutons sont visibles avant que le moteur soit prêt, et un clic fait pendant ce temps est perdu. Sur téléphone, au tout premier chargement, cela peut durer quelques secondes. À corriger.
- Le mode de secours n'est toujours pas mesuré sur le Flip 3 : téléphone verrouillé.

**Erreur en route**

- Le premier essai du panier affichait « 0 » après chaque clic : c'était le défaut ci-dessus, la capture cliquait avant que le moteur soit prêt.

---

## 2026-10-04 — Les points s'activent ; un plancher de lisibilité ; proposition pour le panier

**Ce que Yocthan a dit**

- Il n'avait pas compris les quatre points posés après la réponse de Gemini. Claude les a réexpliqués avec un exemple chacun.
- Puis : « J'ai confiance en toi. Je suis toutes tes recommandations. Tu as le champ totalement libre pour faire ce que tu trouves bien pour le projet. »

**Fait, sur ces quatre points**

1. **Les points s'activent.** Sans `points:` dans le fichier, une page ne devient jamais des points : on la grossit pour lire, c'est tout. Planter un site dans un pixel (`pixels:`) les active aussi. `relief:` sans points est refusé.
2. **Plancher de lisibilité.** `Points(after:)` va de 2 à 16 (avant : de 1 à 16). On peut toujours doubler la taille du texte.
3. **Le panier avec articles et total : une proposition, rien de construit.** Trois options comparées dans `proposals/Claude/panier-articles-2026-10/README.md` ; Claude recommande la plus petite (des quantités et une table de prix).
4. **Le mode de secours.** `?webgl` dans l'adresse fait passer le moteur en WebGL 2, pour le mesurer. Essayé sur le PC : 60 images par seconde dans le Big Bang.

- Vérifié dans Chrome : le salon (sans `points:`) s'arrête à un zoom de 4 et reste une page, sans bouton « Vue points » ; la boutique (avec `points:`) passe en points comme avant.
- 72 tests du moteur.

**Ce qui n'est pas fait**

- La mesure du mode de secours sur le Flip 3 : le téléphone était verrouillé.

**Jusqu'où va le « champ libre », tel que Claude le comprend**

- Il vaut pour construire ce qui a été recommandé et expliqué. Il ne change pas les règles écrites : les statuts des décisions restent à Yocthan, les noms attendent Codex, une pull request d'une autre IA attend son accord.

---

## 2026-10-04 — La seconde réponse de Gemini (au prompt du jour), lue par Claude

**Ce que Yocthan a apporté**

- La réponse de Gemini au prompt du 4 octobre : cohérence 2D et 3D, lisibilité, l'état, les mots, trois risques, un tableau de recommandations. Le prompt datait d'avant la disposition : son troisième risque (« pas de `Row` ») est déjà levé.

**Ce que Claude retient**

- Gemini donne raison à Yocthan sur le fond : partout (WebXR, Google Maps, visionOS), l'auteur offre la 3D et le visiteur la déclenche par un geste. Il répond aussi à la question laissée ouverte : pour lui, le passage en points doit s'activer par l'auteur, comme la rotation.
- Une règle d'accessibilité précise (WCAG 1.4.4) : le texte doit pouvoir grossir jusqu'à 200 % en restant du texte. Notre défaut est à 400 %, mais le langage permet `Points(after: 1)` : la borne basse devrait être 2.
- Pour le panier : la suite la plus petite serait une liste d'articles tenue par le moteur, avec un nombre et un total qu'il calcule lui-même (`{cart.count}`, `{cart.total}`), sans formule écrite par l'auteur.
- Les noms : il garde `State`, `add`, `sub`, `set`, `Relief`, `Points`, `Portals`. Il conteste `tilt`, `after`, `fragment`, `grid`, `depth`, `shrink`, `levels`. Parmi ses propositions, celles en un seul mot vont avec le style du langage : `turn`, `from`, `divisions`, `steps`.
- Le risque numéro un reste le téléphone d'entrée de gamme, jamais mesuré, et le repli sans WebGPU, jamais mesuré non plus.
- Une idée simple : faire essayer la boutique à trois personnes qui ne connaissent pas le projet.

**Ce que Claude conteste**

- « visionOS exige un bouton » et « `model-viewer` ne s'éveille qu'au clic » sont trop affirmés : une application visionOS peut s'ouvrir directement en immersif, et `model-viewer` se révèle par défaut dès qu'il est chargé. La tendance qu'il décrit est juste, pas la règle absolue.
- « L'onglet plantera dès le premier million de points » : au repos, ce million est une image ; le moteur ne dessine jamais plus de quelques milliers de points. Cela reste à mesurer, mais ce n'est pas acquis.
- Ignorer `tilt` quand les animations sont réduites : tourner est un geste volontaire, par un bouton. Claude le laisserait offert.
- `zoomAt`, `splitAt`, `maxNesting` : deux mots collés, contre le style du langage.

**Rien n'est décidé.** Les noms attendent Codex. Le passage en points à activer, la borne de `after` et la suite du panier attendent le feu vert de Yocthan.

---

## 2026-10-04 — La disposition ; la page fabriquée d'avance en Rust ; animations réduites ; un garde-fou de fusion

**Ce que Yocthan a dit, après la lecture de Gemini**

- Réduire les animations : cela s'active ou se désactive, « ça dépendra de chacun », et par défaut on reste au niveau normal, pour ne pas perturber.
- « Il faut vraiment qu'on voie le Row et les colonnes. »
- La sortie HTML côté serveur : d'accord, « mais c'est mieux de connecter avec un langage bas niveau ».
- Les noms : faire ce qu'il y a à faire en attendant Codex, puis lui donner la liste de tous les noms face à ceux de HTML, CSS et JavaScript.
- Les fusions : corriger ce qui peut conduire en erreur.

**Fait**

- **La disposition** (`ADR-024`, à l'essai) : `Row`, `Column`, `Grid`, avec `gap`, `align`, `columns`. Une ligne trop longue passe à la ligne ; une grille perd des colonnes sur un écran étroit. La boutique a une grille de trois tableaux et les boutons du panier côte à côte ; sa jumelle web aussi.

![La boutique sur un écran large : trois colonnes](images/2026-10-04-disposition-grand-ecran.png)

![La même sur un téléphone : deux colonnes, les boutons passent à la ligne](images/2026-10-04-disposition-telephone.png)

- **La page fabriquée d'avance, en Rust.** `moteur/src/bin/holo.rs` : le même moteur, compilé pour le PC. `holo check` vérifie un fichier, `holo html` écrit sa page. Le serveur de démonstration s'en sert : la page arrive avec son contenu et son titre, lisible par un robot ou un navigateur qui ne lance pas le moteur. Sans ce programme, tout marche comme avant.
- **Animations réduites.** C'est le réglage que chacun a déjà dans son téléphone ou son ordinateur. Sans lui, niveau normal. Avec lui : la page ne devient pas des points toute seule au zoom, pas de transition de portail. Le bouton « Vue points » reste offert.
- **Un pixel planté est un vrai bouton** : on l'atteint au clavier, un lecteur d'écran dit son nom.
- **`outils/fusionner.sh`** attend la fin des tests et refuse la fusion s'ils ne sont pas tous verts.
- **`docs/01-holocode/NOMS.md`** : chaque mot de HoloCode face à celui du web (repris, changé, nouveau), puis les mots du web qu'on n'a pas pris.
- 72 tests du moteur.

**Ce qui n'est pas fait**

- En vue points, un lecteur d'écran ne lit toujours rien.
- La sortie HTML d'avance ne vaut que pour le serveur de démonstration ; il n'y a pas encore de vrai hébergement.
- « Réduire les animations » suit le réglage de l'appareil ; il n'y a pas de bouton dans la page pour le changer. À voir avec Yocthan si c'est ce qu'il voulait.
- Le nouveau bloc `Grid` porte le même mot que `Points(grid:)` : collision à trancher avec les noms.

**Erreurs en route**

- Le test du vocabulaire a échoué dès l'ajout de `Row` : la boutique ne l'employait pas encore. C'est son rôle.
- Le serveur tenait une variable `HOLO_DEPOT` vide pour un dossier : corrigé.

---

## 2026-10-04 — La réponse de Gemini (au premier prompt), lue par Claude

**Ce que Yocthan a apporté**

- La réponse de Gemini : comparaison avec dix technologies, antécédents historiques, critique de l'ordre des chantiers, critique des noms, accessibilité, trois risques.
- Elle répond au prompt d'avant : elle dit que `State` n'existe pas, alors que le panier a été fusionné le même jour. Le prompt du 4 octobre (`docs/05-discussions/prompts/2026-10-04-gemini.md`) reste à lui donner.

**Ce que Claude retient comme juste**

- Les antécédents vont dans le sens de Yocthan : JanusVR a échoué parce qu'il fallait marcher vers une porte pour lire la page suivante ; Prezi a donné le mal des transports. Donc le site normal d'abord, et la 3D activée par l'auteur.
- Un site caché dans un pixel ne se devine pas : le défaut de `pixels:` est la découvrabilité.
- Noms : `grid` heurtera la future disposition en grille ; `tilt` ne dit plus « faire le tour » ; `depth` et `levels` se ressemblent trop.
- Accessibilité : la vue points ne tient pas compte de « réduire les animations » ; en vue points, rien n'est dit à un lecteur d'écran ; un pixel planté n'est pas atteignable au clavier.
- Un robot ou un vieux navigateur qui n'exécute pas le moteur ne voit rien : il manque une sortie HTML faite côté serveur.
- Les imports arrivent trop tard dans l'ordre des chantiers.

**Ce que Claude conteste**

- « Batterie cinq à dix fois plus élevée », « onglet tué à 300 ou 400 Mo » : aucun chiffre sourcé. Mesuré ici : 88 et 99 Mo, et aucune image dessinée par le moteur tant qu'on lit la page.
- « Activer le moteur seulement au seuil de zoom » : c'est déjà le cas.
- Les noms proposés (`attachTo`, `maxRecursion`, `triggerScale`) sont en deux mots collés, contre le style du langage ; et `levels` proposé pour `depth` existe déjà dans `Zoom`.
- « Le compilateur refuse qu'on cache une information essentielle dans un point » : un vérificateur ne sait pas ce qui est essentiel.
- Une erreur de date : Pad est de 1993 (Perlin et Fox), Pad++ de 1994 (Bederson et Hollan).

**Rien n'est décidé.** Les noms attendent Codex ; les suites proposées par Claude attendent le feu vert de Yocthan.

---

## 2026-10-04 — Le panier, première action avec état ; la rotation s'active

**Ce que Yocthan a dit**

- « De face, la rotation ne marchait pas. » Il veut qu'elle marche de face, mais qu'elle soit activée, pour la cohérence : « Si d'autres peuvent donner tout le temps en 3D, ça va vraiment déranger la vision et la lisibilité du site. »
- « Fais le panier. On va voir ce que ça donne. Et après, on va en juger. »
- Les noms attendront le retour de Codex, dont les quotas sont épuisés. Le téléphone : essayé, c'est bon.
- Il donne maintenant le prompt à Gemini, et veut le texte dans un fichier.

**Fait**

- **Le panier** (`ADR-023`, à l'essai). `state: State(cart: 0)` déclare une valeur ; `{cart}` dans un texte l'affiche ; `cart.add(1)`, `cart.sub(1)`, `cart.set(0)` sont des demandes écrites dans l'effet d'une règle. L'arbitre est dans le moteur (`moteur/src/etat.rs`) : le bouton ne change rien lui-même. La boutique a son panier, et sa jumelle en HTML et JavaScript aussi, pour comparer. Le panier suit le visiteur dans l'atelier.

![Le panier de la boutique, après trois ajouts et un retrait](images/2026-10-04-le-panier.png)

- **La rotation s'active.** Sans `Relief(tilt:)`, une page ne tourne plus : c'est un site ordinaire. Avec, le bouton « Tourner » est offert dès la page de face ; la page devient ses points sans que rien ne change à l'écran, puis tourne sous le doigt.

![La boutique tournée directement depuis la page de face](images/2026-10-04-tourner-depuis-la-page.png)

- Le prompt pour Gemini est dans `docs/05-discussions/prompts/2026-10-04-gemini.md`. Il se suffit à lui-même, Gemini n'ayant pas accès au dépôt.
- 71 tests du moteur. Vérifié dans Chrome : 0, 1, 2, 3, puis 2 après un retrait ; un retrait à zéro reste à zéro.

**Ce que Claude a compris, et qui reste à confirmer**

- « La rotation ne marche pas de face » a été lu ainsi : sur le site ordinaire, rien ne permettait de tourner ; il fallait d'abord passer en vue points. Si Yocthan voulait dire autre chose, c'est à reprendre.
- Le défaut de `tilt` a changé deux fois dans la journée : 52deg, puis 360deg, puis 0deg (éteint). Le dernier suit sa demande de cohérence.
- Le passage en points au zoom reste offert d'office. Doit-il lui aussi s'activer ? Question posée.

**Limites du panier**

- Des nombres entiers seulement ; pas de condition, pas de total ; rien n'est gardé après un rechargement.
- `cart.add(1)` ouvre une parenthèse avec une minuscule : une exception à la règle des majuscules, limitée à l'effet d'une règle.
- Pas encore de cas dans la suite de conformité.

**Erreurs en route**

- La première capture du panier était blanche : le serveur d'essai compressait encore le moteur quand la capture est partie. Refaite avec une attente plus longue.
- Le serveur d'essai lancé avec `HOLO_DEPOT` vide ne trouvait aucun fichier : une variable vide n'est pas une variable absente.

---

## 2026-10-04 — Faire le tour de la page ; deux réglages de plus ; le Flip 3 mesuré

**Ce que Yocthan a demandé**

- Mesurer sur son Galaxy Z Flip 3.
- « La rotation n'est pas à 360 degrés, elle est bloquée à un certain angle. »
- Créer les réglages utiles pour ce qui existe, et les mettre dans le langage s'ils n'y sont pas.
- Un téléphone modeste sera difficile à trouver : régler d'abord tous les paramètres, on verra ensuite.

**Fait**

- `Relief(tilt:)` va jusqu'à `360deg`, et c'est la valeur par défaut. On fait le tour de la page ; par derrière, on la voit à l'envers, comme une feuille tenue devant une lampe. Un auteur qui écrit `tilt: 52deg` garde sa limite.

![La page du salon vue par derrière](images/2026-10-04-page-vue-par-derriere.png)

![Ses points, vus par derrière et de près](images/2026-10-04-points-vus-par-derriere.png)

- Par derrière, pointer et glisser restent justes : le zoom vise l'endroit sous le doigt, la page suit le doigt. Vue par la tranche, le moteur ne calcule que les points proches de l'endroit regardé.
- Deux réglages existaient dans le moteur sans mot pour les écrire : `Zoom(speed:)` (vitesse du zoom à la molette, 0.25 à 4) et `Portals(duration:)` (temps d'ouverture d'un portail, 0ms à 2000ms). Guide, inventaire, boutique et `ADR-021` mis à jour ; 67 tests.
- Mesure sur le Flip 3 (Snapdragon 888, 2021), WebGPU : 60,3 images par seconde en zoomant dans le Big Bang, 60,2 en vue points (image la plus lente : 33 ms), entrée en vue points en 221 ms, 99 Mo pour l'onglet. Pas moins bien que le Flip 5.

**Ce qui n'est pas vérifié**

- Le pincement à deux doigts n'a été essayé que par simulation, pas avec de vrais doigts : à Yocthan de l'essayer.
- Tourner la page se fait par le bouton « Tourner », le bouton droit ou Maj. Au doigt, il faut passer par le bouton : pas de geste à deux doigts pour tourner.
- Toujours pas de téléphone modeste.

**Erreurs en route**

- Un test existant se servait de `speed` comme exemple de paramètre inconnu : il a échoué quand `speed` est devenu un vrai mot. Remplacé par un mot qui n'existe pas.
- La première capture « par derrière » montrait la boutique arrêtée à 52 degrés : le serveur lisait le fichier de la branche principale, où `tilt: 52deg` est encore écrit. C'était la limite de l'auteur qui jouait, pas un défaut. Capture refaite sur le salon.
- `sur_la_page` refusait tout regard venant de derrière la page : corrigé, avec un test.

---

## 2026-10-04 — Au doigt, le pincement ne menait plus aux points ; `ADR-010` accepté

**Ce que Yocthan a montré**

- Une vidéo de son téléphone : en pinçant la page de la boutique, elle grossit, mais ne devient jamais des points. « Les zooms ne donnent plus comme avant. »
- Sur la mesure : d'accord pour cocher la décision du moteur en Rust, tout en la vérifiant plus tard sur un téléphone plus modeste. Un Galaxy Z Flip 3 conviendrait-il ?

**La cause**

- Claude a lu la vidéo image par image. Au doigt, c'était le zoom de Chrome qui prenait le pincement : il grossit tout l'écran à sa façon, sans rien dire à la page. Le zoom de la page n'écoutait que la molette. La limite était écrite dans le journal du 3 octobre (« sur un écran tactile, le premier pincement n'est pas encore capté »), mais avec le zoom ordinaire ajouté ensuite, elle est devenue un vrai défaut : le chemin vers les points était fermé au doigt.

**Fait**

- La page suit maintenant le pincement à deux doigts, par le même chemin que la molette : la page vivante grossit jusqu'à `Points(after:)`, puis ses pixels deviennent des points, sans lever les doigts ; et dans l'autre sens au retour. Le zoom de Chrome est retiré sur ces pages ; on défile toujours avec un doigt.
- Vérifié dans Chrome avec deux doigts simulés : en les écartant, la page passe à un agrandissement de 4, puis en vue points.
- `ADR-010` (le moteur en Rust) passe de `EXPÉRIMENTATION` à `ACCEPTÉ`, sur décision de Yocthan après la mesure. Une condition de réexamen est ajoutée : la mesure sur un téléphone d'entrée de gamme.

**Réponse donnée sur le Z Flip 3**

- Il est utile à mesurer (deux ans plus ancien), mais ce n'est pas un téléphone modeste : c'était un haut de gamme en 2021 (Snapdragon 888, 8 Go). « Modeste » veut dire 4 Go de mémoire ou moins et un processeur d'entrée de gamme, comme un Galaxy A de la série basse.

---

## 2026-10-04 — La mesure sur téléphone, enfin

**Fait**

- Yocthan a branché son Galaxy Z Flip 5 par câble et activé le débogage USB. Claude a relié le téléphone au serveur du PC (`adb reverse`), puis lu les mesures du moteur directement dans le Chrome du téléphone (`adb forward` et le protocole de débogage de Chrome). L'outil est rangé dans `moteur/outils/mesurer-telephone.mjs`.
- Résultats, WebGPU actif :

| Mesure | Résultat |
|---|---|
| Big Bang, zoom continu à travers 7 mondes | 59,8 images par seconde, image la plus lente 16,9 ms |
| Première image, moteur en cache | 336 ms (3,5 s au tout premier chargement) |
| Boutique, page normale | prête en 122 ms, le moteur ne dessine rien |
| Boutique, entrée en vue points | 264 ms pour 1 118 880 points |
| Boutique, zoom en vue points, 4 morcellements | 59,7 images par seconde, jamais plus de 6 344 points à l'écran |
| Mémoire de l'onglet | 88 Mo en part propre, 10 Mo de tas JavaScript |

- C'est la mesure qu'attendaient `ADR-005` (tourner sur le matériel existant) et `ADR-010` (le moteur en Rust) depuis le premier jour. Elle est favorable. Leur statut reste à Yocthan.

**Ce que la mesure ne dit pas**

- Une seule série, sur un téléphone haut de gamme. Ni la batterie, ni l'échauffement dans la durée, ni le mode sans WebGPU, ni un téléphone modeste.
- Une image a pris 50 ms pendant le zoom en vue points : un accroc, à surveiller.

**Erreurs en route**

- L'ancienne adresse Wi-Fi donnée à Yocthan (`10.105.109.17`) n'était plus la bonne, et un VPN tournait sur le PC : par le câble, on évite les deux.
- Une première mesure a été prise pendant que l'écran du téléphone était en veille : une image en trois minutes. Jetée et refaite, écran allumé. Claude a activé puis remis le réglage « rester allumé sur USB ».

---

## 2026-10-04 — La seconde revue de Codex : ses défauts corrigés

**Ce qui s'est passé**

- Yocthan a envoyé à Codex et à Gemini les messages de relecture préparés par Claude. Codex a répondu par la pull request n° 39 : une revue critique, sept fichiers hostiles et un harnais de test. Yocthan a demandé de vérifier et de fusionner : fait, après lecture complète. Puis : « pour le reste, fais ce qu'il y a de mieux ».
- La revue est sérieuse. Elle montre que le guide promettait plus que le moteur ne tenait. Claude a relu son propre code et confirme chaque défaut repris ci-dessous.

**Corrigé**

| Défaut trouvé par Codex | Correction |
|---|---|
| `Zoom(max:)` ne bornait pas le zoom ordinaire | Il borne le zoom entier ; `Points(after:)` plus grand que `Zoom(max:)` est refusé |
| `Portals(count:)` ne bornait pas les sites écrits | Il borne tout le carrefour ; un rond « + N autres » dit ce qui n'est pas montré |
| La vérification acceptait `A(to: "javascript:…")` | Elle refuse tout ce que l'affichage refuserait |
| Une adresse en `#@…` contactait un autre serveur sans geste ; le carrefour lisait tous les fichiers distants | Un clic, un serveur, un fichier ; une adresse distante propose le passage |
| Chez quelqu'un d'autre, rien ne le disait plus | Un bandeau du moteur, « Vous êtes chez … », avec « Revenir » |
| `http` vers n'importe quelle adresse, y compris le réseau privé | `https` ; `http` seulement vers sa propre machine |
| Mémoire bornée en nombre de fichiers, pas en taille ; aucun délai | 256 Ko par fichier, 8 secondes, lecture arrêtée au-delà |
| `density: 3` pouvait demander des centaines de Mo | Huit millions de points au plus |
| `above:` acceptait un repère hors de l'écran | Le repère doit être dans le même site |
| Aucune limite avant l'analyse d'un fichier | 262 144 octets, 100 000 mots, 64 niveaux d'emboîtement |
| L'image de la page restait en mémoire après la vue points | Elle est rendue à la sortie |

- Les trois sondes de Codex qui constataient des défauts sont retournées : elles vérifient maintenant que ces défauts ne reviennent pas. Ce changement touche son dossier, avec l'accord de Yocthan.
- Vérifié avec deux serveurs locaux et de vrais gestes : zéro requête vers l'autre serveur à l'ouverture de la page, à l'ouverture du carrefour, et à l'arrivée par une adresse en `#@…` ; une seule après le clic ; le bandeau s'affiche, « Revenir » ramène.
- Tests du cœur : 66 sur 66 ; harnais de Codex : 7 sur 7.

![Chez quelqu'un d'autre : le bandeau du moteur](images/2026-10-04-chez-quelqu-un-d-autre.png)

**Non corrigé, et dit**

- Le poids déclaré d'une image (`weight`) n'est pas comparé à son poids réel.
- Pas d'en-tête `Content-Security-Policy` sur le serveur de démonstration.
- Les noms : Codex en juge une dizaine mauvais et propose des remplacements. Rien n'est renommé : c'est à Yocthan de trancher. Claude lui a préparé les questions à poser à Codex.

**Leçon**

- Claude avait écrit dans le guide et dans `ADR-022` des protections qu'il n'avait pas vérifiées dans tous les chemins (l'adresse ouverte directement). Une promesse de sécurité s'écrit après l'avoir mise à l'épreuve, pas avant.

---

## 2026-10-03 — Les deux limites revues ; HoloCode comparé au web, balise par balise

**Ce que Yocthan a demandé**

- Revoir les deux limites annoncées : si elles sont bonnes, les laisser ; si elles sont mauvaises, les corriger. Il en donne l'autorisation.
- Que Claude fasse à fond sa part de la mise à l'épreuve face aux balises du web classique, puis lui donne la liste de ce qu'il doit faire de son côté.

**Les deux limites**

- « Seuls les fichiers rangés à côté sont acceptés » : mauvaise limite, corrigée. `Point(inside: "https://…/jardin.holo")` mène au fichier d'un autre serveur. Le portail affiche le nom du serveur ; un tel fichier n'est lu qu'à l'ouverture du carrefour, jamais d'avance ; il passe par le vérificateur. Le serveur local envoie l'en-tête qui autorise cette lecture.
- « Rien ne borne le nombre de passages » : bonne limite pour les passages (on tourne dans une maison autant qu'on veut), mauvaise pour la mémoire. Le moteur ne garde plus que les 32 derniers fichiers lus.
- Vérifié avec deux serveurs locaux sur deux ports, donc deux sites différents pour Chrome : bouton, carrefour (le portail affiche « Ailleurs · 127.0.0.1:8081 »), franchissement, arrivée dans le jardin de l'autre serveur sans rechargement ; un cran de dézoom ramène au départ ; l'adresse en `#@…` ouvre directement le fichier distant.
- Une contrainte des navigateurs, écrite dans `ADR-022` : la barre d'adresse ne peut pas montrer l'adresse d'un autre serveur ; elle affiche le fichier de départ suivi de `#@` et de l'adresse réelle.

**La comparaison**

- [`docs/01-holocode/COMPARAISON-WEB.md`](../01-holocode/COMPARAISON-WEB.md) : HTML balise par balise, puis CSS et JavaScript, avec pour chaque ligne « fait », « exprès » ou « manque ». Verdict écrit sans détour : HoloCode sait « lire et se promener », pas encore « agir » ; le manque le plus gênant est la disposition. Suit un ordre proposé en dix rangs pour combler les manques.

**Ce qui revient à Yocthan** (liste donnée dans la conversation)

- Mesurer sur le téléphone ; écrire un vrai petit site pour sentir ce qui gêne ; relire les noms proposés ; choisir l'ordre des manques ; faire relire par Codex et Gemini.

---

## 2026-10-03 — Aller ailleurs : le lien `A`, et le point qu'on traverse d'un fichier à l'autre

**Ce que Yocthan a posé**

- Entre les mondes, pas de rechargement ni de redirection : « on traverse la porte du salon et on va au jardin, avec les mêmes jambes ». Seuls l'animation et le zoom font la transition. Le lien normal reste pour un vrai changement de site, à l'ancienne. « Porte » était une façon de parler.
- Pourquoi le lien ne s'écrit-il pas `A`, comme en HTML ? Et les listes, les `UL`, `LI` ?
- Dézoomer doit faire ressortir du monde où l'on est.
- À noter en mémoire : tout ce qui se fait doit être documenté, et avoir son élément dans le langage, pour que lui ou quelqu'un d'autre puisse reprendre le travail.

**Fait** (`ADR-022`)

- `A("texte", to: "adresse")` : le lien classique. Claude avait proposé `Link` ; Yocthan a demandé `A`, et il a raison : c'est le mot de HTML, et `link` y désigne autre chose.
- `Point(inside: "jardin.holo")` : le monde d'un point peut être un autre fichier. On y passe sans changer de page : carrefour, portail, animation ; l'adresse devient celle de l'autre fichier ; « retour » ramène. Les fichiers voisins sont lus d'avance.
- Dézoomer, quand la page est à sa taille normale, fait ressortir : au site, au fichier, ou hors du monde calculé d'où l'on venait.
- `List(ordered: true)` numérote une liste ; un élément de liste peut être un lien. Pas de `UL`, `OL`, `LI`.
- Nouvel exemple, `exemples/maison/` : un salon sombre et un jardin clair, deux fichiers.
- Vérifié avec de vrais gestes envoyés à Chrome, une marque posée dans la page prouvant qu'elle n'a pas été rechargée : bouton « Aller au jardin », le carrefour s'ouvre sur le jardin ; clic sur le portail, l'adresse devient `jardin.holo`, le fond devient clair, la page n'a pas été rechargée ; un cran de dézoom, retour à `salon.holo`.
- Guide mis à jour (liens, listes, passage entre fichiers, inventaire). Tests du cœur : 64 sur 64.

![Le carrefour du salon : au milieu, le jardin, qui est un autre fichier](images/2026-10-03-maison-2-carrefour.png)

**Erreur trouvée en vérifiant**

- Les aperçus des portails laissaient déborder leurs styles : l'aperçu du jardin (clair) repeignait celui du salon. Chaque aperçu est maintenant enfermé.

**Limites**

- Seuls les fichiers rangés à côté sont acceptés, pas le site d'un autre auteur sur un autre serveur.
- Rien ne borne le nombre de passages entre fichiers.

---

## 2026-10-03 — Chaque notion a son mot ; un carrefour rempli de mondes

**Ce que Yocthan a demandé**

- Vérifier que chaque notion apportée au site a son équivalent, son mot-clé, dans le code source : l'image, le pixel, la fragmentation, les zooms ; et chaque action : activer ou désactiver le zoom, passer d'un site à l'autre. Le son n'a pas encore été travaillé.
- Le carrefour lui plaît, mais : il n'y a que deux ou trois mondes ; il veut que des mondes remplissent toute la page. Là où le curseur se pose, le monde doit apparaître comme avant (la feuille). Une petite lumière au fond, pour y voir s'il fait noir. Et un sens de défilement à choisir : à l'horizontale, à la verticale, en diagonale, en liste ou en grille.

**Fait**

- L'inventaire est dans le guide, partie « Chaque notion et son mot » : dix-sept notions ont leur mot, huit n'en ont pas encore (le son, la vidéo, le survol, réagir au zoom par une règle, la disposition, les liens entre fichiers, les formulaires, le personnage).
- Deux mots manquaient pour ce qui existait déjà, ils sont ajoutés : `Zoom(active:)` pour permettre ou interdire le zoom, et le bloc `Portals(layout:, count:, size:, brightness:)` pour le carrefour. La page gagne la capacité `portals` : `On(Map.tap, effect: Shop.portals)` ouvre le carrefour par une règle.
- Le carrefour remplit la fenêtre : les sites écrits dans le fichier d'abord, avec leur contenu, puis des mondes calculés à partir d'une graine, jusqu'à `count`. Chaque monde calculé est une boule de lumière de sa couleur ; un clic l'ouvre en profondeur (le Big Bang), à l'adresse `fichier.holo#~graine`, et « Retour » ramène au site.
- Là où le curseur se pose, le monde grandit et devient une feuille lisible.
- Le fond est éclairci par une lueur (`brightness`).
- Quatre dispositions : `grid`, `row` (on défile de gauche à droite), `column` (de haut en bas), `diagonal`.
- Vérifié avec de vrais gestes envoyés à Chrome : douze portails ; le survol agrandit ; un clic sur un monde calculé donne l'adresse `#~13044733080473193766` et le moteur dessine ; « Retour » ramène au site.
- Tests du cœur : 62 sur 62.

![Le carrefour rempli de mondes ; le curseur est sur le monde jaune](images/2026-10-03-carrefour-grille.png)

**Limites**

- Les mondes calculés ne contiennent pas de site : ce sont des univers de points. Y poser des sites écrits par d'autres personnes demande les liens entre fichiers.
- Les dispositions `row`, `column` et `diagonal` sont lues et appliquées, mais Claude n'a capturé que la grille.

---

## 2026-10-03 — Glisser déplace la page pendant tout le zoom

**Ce que Yocthan a relevé**

- Depuis le zoom ordinaire, on ne pouvait plus déplacer le site en glissant, à gauche ou à droite, comme on le faisait en vue points. Il ne savait plus comment se rapprocher d'un point : « ça devient n'importe quoi ».

**La cause**

- Claude avait fait deux zooms qui se suivent (la page vivante jusqu'à × 4, les points ensuite) avec deux façons de se déplacer : la barre de défilement d'abord, le glissement ensuite. Et en revenant des points, la page reprenait à l'endroit d'où l'on était parti, pas à celui où l'on était arrivé.

**Fait**

- Dès que la page est grossie, glisser la déplace, dans tous les sens : le geste est le même du début à la fin du zoom. Une main l'indique. Un double clic sélectionne toujours un mot.
- En revenant de la vue points, la page vivante reprend là où l'on se trouvait.
- Vérifié avec de vrais gestes envoyés à Chrome (voir la pull request).

---

## 2026-10-03 — Le carrefour à portails, le zoom ordinaire avant les points, la limite des niveaux

**Retour de Yocthan sur la boucle**

- Le bref noir à l'ouverture d'un site dérange. Son idée : à la place, des points gros et proches, prêts à être cliqués, « comme les portails de Strange dans Avengers Endgame » : des mondes déjà là, sur toute la page, qui incitent à changer de monde. C'est ce qu'il appelait la « roadmap métaverse ».
- Pas de fond noir quand le site est clair : il faut suivre le style du site.
- Les points autour de la feuille ne doivent pas être seulement calculés par la graine.
- Mettre une limite au nombre de niveaux.
- Un visiteur est habitué à zoomer pour lire, à sélectionner, à copier. Le zoom doit d'abord rester normal ; la « métaversification » ne commence qu'à partir d'une certaine profondeur.
- Question : une photo, une vidéo se décomposent-elles de la même façon ?

**Fait**

- **Le carrefour.** Entrer dans un point qui contient un site n'ouvre plus une feuille dans un monde noir, mais des portails ronds posés sur le fond du site où l'on est. Chaque portail montre le site où il mène, en petit. Autour du portail visé : les autres sites contenus dans la page, le site où l'on est, celui d'où l'on vient. Ce sont de vrais sites, écrits dans le fichier, pas des points tirés d'une graine. Un bouton « Carrefour » l'ouvre à tout moment.
- **Plus de noir.** Un clic sur un portail le fait grandir jusqu'à remplir la fenêtre, et il devient le site, sans recharger la page. Le fond de la fenêtre prend la couleur du site.
- **Le zoom ordinaire d'abord.** Ctrl + molette grossit d'abord la page vivante, jusqu'à `Points(after: 4)` : le texte reste du texte. Au-delà, ses pixels deviennent des points. En revenant, on retrouve la page grossie, puis sa taille normale. Pendant le zoom ordinaire, le moteur ne dessine rien.
- **La limite.** `Zoom(levels: 8)` : un fichier qui emboîte plus de sites est refusé, avec la ligne du point de trop.
- Vérifié avec de vrais gestes envoyés à Chrome : un cran de zoom, page vivante grossie 1,9 fois, zéro image dessinée ; trois crans, passage aux points ; retour à la taille normale en dézoomant. Zoom sur le pixel rose, clic : le carrefour s'ouvre sur « Secret » ; clic sur le portail : adresse `#Secret`, sans rechargement.
- Guide et `ADR-021` mis à jour. Tests du cœur : 60 sur 60.

![Le carrefour : trois portails vers trois sites](images/2026-10-03-pixel-2-dedans.png)

**Réponse donnée sur la photo et la vidéo**

- Une photo dans la page se décompose déjà comme le reste : ses pixels font partie de l'image de la page. Une vidéo, pas encore : il faudrait redécomposer chaque image, vingt-cinq fois par seconde. C'est faisable pour la partie visible à l'écran, mais c'est un chantier à mesurer.

**Retiré**

- La « vue personnage » (la page en feuille dans un monde noir) et la feuille d'un site dans le monde de son point : le carrefour les remplace. Yocthan avait précisé que la vue personnage vaut pour un jeu, où le monde est l'environnement du joueur.

**Erreurs en route**

- Le zoom ordinaire, fait d'abord avec la propriété `zoom` du CSS, déplaçait la mise en page : ce qui était sous la souris n'y restait pas. Refait avec un vrai agrandissement.
- À la fusion de la pull request n° 33, une coupure de réseau a masqué le résultat des vérifications ; la fusion est partie quand même. Contrôlé juste après : tout était vert. À l'avenir, vérifier avant, dans une commande séparée.

**Limites**

- En vue points sur un site clair, les points sont lumineux sur fond noir : le blanc de la page devient des points blancs.
- Les portails ne montrent que les sites à un pas de distance ; il n'y a pas encore de carte d'ensemble.
- La première entrée en vue points prend un instant (la page est redessinée dans une image).

---

## 2026-10-03 — La boucle : un site dans un pixel, dans un site, dans un pixel…

**Retour de Yocthan sur le site planté dans un pixel**

- Le site caché apparaît bien, mais il s'arrête à la vue personnage. Il veut qu'un deuxième clic, ou un zoom, l'agrandisse jusqu'à sa taille normale, et qu'on retrouve alors le même mécanisme que pour le site d'origine : zoomer, voir les pixels, cliquer, entrer. « La boucle va continuer jusqu'à l'infini. »
- Il a aimé les points derrière la feuille. Il les veut répartis sur toute la fenêtre plutôt qu'un fond noir : l'utilisateur cliquera sur d'autres sites, d'autres mondes. « Ce sera vraiment le multivers. »

**Fait**

- Le monde d'un point s'ouvre en grand comme une page. Un deuxième clic sur la feuille, ou un zoom qui la grossit assez, et le site caché prend toute la fenêtre : c'est alors un site comme un autre, avec ses boutons, sa vue points, sa vue personnage.
- Un `World` accepte `pixels:` : un site peut en contenir un autre, qui en contient un autre. Le site de Yocthan a maintenant trois niveaux (son site, « Le site caché » dans un pixel rose, « Le trésor » dans un pixel doré du site caché).
- Chaque site a son adresse : le fichier, puis `#` et le chemin des points, comme `mon-site.holo#Secret/Tresor`. Le bouton « retour » du navigateur remonte d'un niveau, et une règle `leave` aussi.
- Les points du monde entourent la feuille sur toute la fenêtre : la feuille est plus petite, et l'on part d'un peu plus près.
- Vérifié avec de vrais gestes envoyés à Chrome : zoom sur le pixel rose, clic, feuille ; clic sur la feuille, adresse `#Secret` et site en grand ; zoom sur le pixel doré, clic, feuille du trésor ; clic, adresse `#Secret/Tresor` ; bouton « Remonter », adresse `#Secret`.
- Guide mis à jour. Tests du cœur : 59 sur 59.

![Le site caché ouvert en grand, avec son propre pixel doré à droite](images/2026-10-03-pixel-3-en-grand.png)

![Le troisième site, dans un pixel du site caché](images/2026-10-03-pixel-4-tresor.png)

**Limites, dites à Yocthan**

- Ouvrir un site en grand recharge la page : il y a un bref noir entre la feuille qui grossit et le site en grand.
- Les points autour de la feuille sont ceux que la graine calcule : on peut y entrer (c'est le Big Bang), mais ils ne contiennent pas de site écrit. Y poser de vrais sites est le chantier A.
- Le nombre de niveaux n'est pas limité : le garde-fou que Yocthan a évoqué reste à décider.

---

## 2026-10-03 — Un site planté dans un pixel de la page

**Ce que Yocthan a demandé**

- Le chantier A (voir la source d'un point, écrire dans un point) est confirmé en premier. Pour la police du rendu natif : « fais ce qui est le mieux », donc Noto Serif.
- Il a d'abord voulu apprendre à écrire : Claude lui a fait un site de départ à lui, `exemples/mon-site/mon-site.holo` (dans son dossier, hors GitHub), et une leçon en cinq règles.
- Sur la vue personnage, il a précisé : elle vaut pour un jeu, où le monde est l'environnement du joueur (Trevor, son téléphone et sa ville sont dans un point dédié à GTA). Pour un site, il veut voir ceci : choisir un pixel de la page, y planter un site d'une autre couleur, zoomer, cliquer dessus, s'en approcher, et y entrer ; et ce site du dedans doit se voir en vue personnage, puisqu'il est à l'intérieur d'un point. « Tu le fais, et après on en parle. »

**Fait**

- `Page(pixels: [ Point(name:, above:, color:, inside: World(...)) ])` : un point planté dans un pixel de la page, juste au-dessus du bloc nommé par `above`, à l'extrémité droite. Au repos il fait un pixel. En vue points il grossit avec les autres ; un clic dessus fait s'approcher, puis entrer.
- Le site contenu dans un point (celui-ci, mais aussi l'atelier de la boutique) ne s'affiche plus sur un panneau en bas de l'écran : c'est une feuille posée dans le monde du point, qui tourne et grandit avec lui, comme la page en vue personnage.
- Vérifié avec de vrais gestes envoyés à Chrome sur le site de Yocthan : trois crans de zoom sur le pixel, un clic, et l'on se retrouve devant « Le site caché », rose sombre, dans son monde.
- Le guide a une nouvelle partie, avec un exemple relu par le test. Tests du cœur : 58 sur 58.

![Le pixel rose, après trois crans de zoom](images/2026-10-03-pixel-1-zoom.png)

![Après le clic : le site caché, posé dans le monde du point](images/2026-10-03-pixel-2-dedans.png)

**Provisoire, dit à Yocthan**

- `pixels:` et `above:` sont une écriture d'essai, faite pour voir l'effet. On ne sait placer un point que « au-dessus d'un bloc, à droite ».
- Les millions d'autres points de la page restent calculés : on ne peut rien écrire dedans.
- L'approche est un zoom rapide, pas encore une vraie traversée ; en sortant du site caché on revient à la page entière, pas à l'endroit où l'on avait zoomé.

---

## 2026-10-03 — Le guide de l'auteur : comment écrire en `.holo`

**Ce que Yocthan a relevé**

- « Supposons que je veux écrire en `.holo`. Qu'est-ce que je fais ? Je n'ai pas encore vu la documentation. » Il avait raison : le langage avançait, mais rien n'expliquait à un auteur comment s'en servir.
- Le fichier du Big Bang n'avait pas bougé depuis le premier jour.
- Il ne voyait pas à l'écran les fichiers sur lesquels on travaille.

**Fait**

- [`docs/01-holocode/GUIDE.md`](../01-holocode/GUIDE.md) : le guide de l'auteur. Onze parties, de la première page aux réglages de vue, avec un aide-mémoire et la liste de ce qui n'existe pas encore. Ses sept exemples sont relus par un test du moteur : un exemple qui ne marcherait plus ferait échouer le test.
- `moteur/mondes/big-bang.holo` : chaque ligne est maintenant expliquée, et les deux réglages facultatifs (`color`, `palette`) sont montrés en commentaire, prêts à être essayés. Le comportement du Big Bang, validé par Yocthan, n'est pas changé.
- Le guide, la boutique et le Big Bang sont ouverts dans le VS Code de Yocthan.
- Yocthan a donné son accord pour le rendu natif des lettres (le moteur dessine lui-même les lettres, sans passer par une image). Police proposée : Noto Serif, à défaut d'autre choix de sa part. Rien n'est commencé.
- Tests du cœur : 57 sur 57.

**Erreur de méthode, à retenir**

- Claude a ajouté au langage pendant une journée entière sans écrire la notice à mesure. À l'avenir : chaque ajout au langage met à jour le guide dans la même pull request.

---

## 2026-10-03 — Sobriété : le moteur ne dessine que lorsqu'on bouge ; la question du « natif »

**Ce que Yocthan a relevé**

- Il voit beaucoup de fichiers `.png` liés aux images quand Claude travaille. Il veut un langage vraiment natif, qui ne passe pas par des images, même si tout peut devenir image.
- Vérifier la consommation : l'énergie ne doit être dépensée que lorsqu'on entre dans la « métaversification », pas quand on lit un site.
- Il ne savait plus dans quel fichier se trouve la boutique : `exemples/boutique-comparee/boutique.holo`.

**Mesuré, puis corrigé**

- Sur le site normal, le moteur ne fait rien : aucune zone de dessin n'existe, zéro image dessinée. C'était déjà le cas.
- En vue points, le moteur redessinait soixante fois par seconde même quand rien ne bougeait. Il s'arrête maintenant de lui-même après quelques images immobiles, et repart au premier geste. Le suivi côté page fait de même.
- Mesure avec de vrais gestes envoyés à Chrome, sur `boutique.holo` : site normal pendant 5 s, 0 image ; trois crans de zoom, 12 images ; puis 8 s sans toucher, toujours 12 ; un cran de plus, 16. Avant la correction, 8 s d'attente coûtaient environ 480 images.
- Un monde (le Big Bang) continue de se dessiner en permanence : ses points pulsent. La pause existante le coupe quand l'onglet est caché.

**Sur les images**

- Les `.png` de `docs/06-journal/images/` sont les captures que Claude prend pour vérifier son travail et tenir ce journal. Le langage ne s'en sert pas.
- L'essai sur image fixe (`mosaique.html` et son `boutique.png`) est supprimé : il n'avait plus de raison d'être depuis que la vue points part du fichier `.holo`.
- Il reste un vrai passage par l'image, dit à Yocthan : pour obtenir les points, la page est redessinée en mémoire dans une image dont on lit les pixels. Aucun fichier n'est écrit, mais le moteur ne sait pas quel point appartient à quelle lettre. Pour que ce soit natif, le moteur doit dessiner lui-même les lettres et les formes à partir des blocs. C'est un chantier à part, proposé à Yocthan ; rien n'est construit.

---

## 2026-10-03 — Ce que fait le moteur s'écrit maintenant dans le fichier : `Zoom`, `Points`, `Relief`

**Ce que Yocthan a relevé**

- Tout ce qu'on a ajouté n'apparaît pas dans les fichiers `.holo`. Il regardait `big-bang.holo`, six lignes : « on a programmé tout un monde, mais je ne vois même pas les traces de ces mondes-là. Comment les gens vont-ils programmer ? » Il avait raison : les seuils, la grille, le relief et les limites étaient des nombres écrits dans le moteur.
- En dézoomant trop fort, le site rétrécissait et sortait de son cadre.
- Il faut des garde-fous des deux côtés (zoomer sans fin, dézoomer sans fin), programmés par l'auteur. Réduire le site jusqu'à la taille d'un pixel doit être possible, mais seulement si l'auteur l'active.

**Fait**

- Trois blocs s'écrivent dans la page (`ADR-021`) : `Zoom(max:, shrink:)`, `Points(size:, fragment:, grid:, depth:, density:)`, `Relief(height:, tilt:)`. Le moteur les lit et s'y tient. Deux unités de plus : `px` et `deg`. Chaque réglage a des bornes que l'auteur ne peut pas dépasser ; une valeur hors bornes est refusée avec sa ligne.
- `exemples/boutique-comparee/boutique.holo` écrit ces réglages, avec un commentaire par ligne. Nouveau fichier `exemples/zoom/reduire.holo`, qui les change : la page s'y réduit jusqu'à devenir un seul point.
- Le rétrécissement du site : c'était le zoom de Chrome lui-même (Ctrl + molette vers l'arrière), que la page ne retenait que dans certains cas. Elle le retient maintenant toujours. Sans `shrink: true`, dézoomer sur la page ne fait rien.
- Vérifié avec de vrais gestes envoyés à Chrome : trois crans en arrière sur la boutique ne changent rien ; les mêmes sur `reduire.holo` réduisent la page ; douze crans en font un point.
- Tests du cœur : 56 sur 56.

**Réponse donnée à Yocthan sur les six lignes**

- Un fichier `.holo` dit ce que l'on veut, pas comment le faire : `big-bang.holo` dit « un point, graine 1, douze fragments », et le moteur (environ 3 500 lignes de Rust) fait le reste. C'est voulu, pour qu'un non-programmeur puisse écrire. Mais il avait raison sur le fond : ce qui se règle doit se voir dans le fichier.

**Limites**

- Les noms sont une proposition de Claude, à confirmer par Yocthan.
- Les seuils de zoom d'un `Point` seul (le Big Bang) sont encore dans le moteur ; la vue personnage et la vue « roadmap » ne sont pas décrites dans le langage.

---

## 2026-10-03 — La mosaïque gagne la profondeur (l'axe Z) et de belles lettres

**Retour de Yocthan sur la mosaïque**

- Il a beaucoup aimé (« je suis à l'extase »). La pull request n° 28 est fusionnée.
- Deux défauts : on ne voit qu'en 2D, il manque le Z ; et les lettres sont « quasiment horribles », presque illisibles.

**Fait**

- **Le Z.** Chaque point a maintenant trois coordonnées. De face, la page reste plate comme une feuille, et les points sont exactement à la place des pixels. Quand on la fait tourner (bouton « Tourner », bouton droit de la souris, ou Maj), on la voit de biais, en perspective, et ce qui est lumineux se soulève au-dessus du fond : les lettres deviennent des objets en relief. Les points nés d'un morcellement ne sont pas tous à la même hauteur. L'image ordinaire, tant qu'on la voit, tourne de la même façon : c'est la feuille de papier dans l'espace.
- **Les lettres.** Trois causes de laideur, trois corrections. L'image d'origine portait des franges bleues et orange (le lissage coloré des écrans plats) : elle est reprise sans. Elle était trop grossière : elle est reprise deux fois plus fine (2560 × 1440, soit 3 686 400 points au repos). Et en se morcelant, chaque pixel devenait un carré aux couleurs tirées au hasard : la couleur d'un point né d'un morcellement est maintenant celle de l'image à cet endroit précis, fondue entre les pixels voisins, et sa graine ne la déplace que peu aux premiers niveaux.
- Tests du cœur : 52 sur 52, dont : le zoom garde sous le doigt le même pixel même quand la page est de biais ; un aller et retour entre la page et l'écran retombe au même endroit.

![Les lettres, un point par pixel](images/2026-10-03-mosaique-5-lettres.png)

![La même vue de biais : les lettres se soulèvent](images/2026-10-03-mosaique-6-de-biais.png)

- **Une seule adresse, une seule page.** Yocthan a relevé deux choses : la barre d'adresse montrait `mosaique.html` au lieu du fichier `.holo`, puis, après une première correction, que `boutique.holo?` et `mosaique.html` ne se comportaient pas de la même façon, avec un `?` en trop. Il avait raison : c'étaient deux portes séparées. Il n'y en a plus qu'une. On ouvre `http://localhost:8080/exemples/boutique-comparee/boutique.holo` : c'est le site normal, vivant. On zoome dessus (Ctrl + molette, ou pincer) : la même page devient des points, sans changer d'adresse ni recharger. On dézoome jusqu'au bout : on retrouve le site normal. C'est le zoom qui déclenche la vue, comme il le demandait ; un bouton « Vue points » reste pour les écrans tactiles.
- Pour y arriver, le moteur fabrique la page, puis on la redessine dans une image (un SVG qui la contient, possible parce que c'est le moteur qui l'a fabriquée) dont chaque pixel devient un point. Ce n'est plus une image fixe prise d'avance : si l'on change le fichier, les points changent.
- Vérifié avec de vrais gestes envoyés à Chrome par l'outil de capture (variable `HOLO_GESTES`) : cinq crans de Ctrl + molette sur le titre font apparaître les points ; deux crans en avant puis quatre en arrière ramènent au site normal.

![Après cinq crans de Ctrl + molette sur le titre de boutique.holo](images/2026-10-03-points-3-ctrl-molette.png)

**Erreur en route**

- Le relief était compté en pixels de l'image : en zoomant très profond, il devenait immense et le moteur cherchait des points sur des milliards de cases. Un test est resté bloqué. Le relief est maintenant borné à 80 pixels d'écran.

**Limites**

- Le relief vient de la lumière : ce qui est clair se soulève. Sur une page à fond blanc et texte noir, ce serait l'inverse de ce qu'on veut. Il faudra partir de la page elle-même, où le moteur sait ce qui est une lettre.
- On tourne la page jusqu'à 52° environ, pas au-delà : on ne peut pas encore passer derrière ni se placer entre deux lettres.
- Dans la vue points, la page est une image d'elle-même : ses boutons ne répondent pas tant qu'on n'est pas revenu au site normal.
- Ctrl + molette est le geste de zoom de Chrome : la page l'intercepte. Sur un écran tactile, le premier pincement n'est pas encore capté ; il faut le bouton « Vue points ».
- Tourner la page à la souris et les gestes au doigt ne sont pas essayés par Claude.

---

## 2026-10-03 — VS Code reconnaît le langage : extension HoloCode

**Fait**

- Yocthan a demandé que VS Code reconnaisse le langage, et que ce soit le fichier `.holo` qu'on lance, pas un fichier `.html`.
- `outils/vscode-holocode/` : une extension VS Code. Elle colore les fichiers `.holo` (blocs, paramètres, styles, textes, unités, couleurs, commentaires) et ajoute un bouton ▶ en haut à droite de l'éditeur (ou `Ctrl+Alt+H`) qui enregistre le fichier et l'ouvre dans le navigateur à sa propre adresse.
- Le paquet se fabrique avec un script Python (`empaqueter.py`), sans rien installer d'autre. L'extension est installée dans le VS Code de Yocthan (`holoverse.holocode`).
- Le serveur local accepte une variable `HOLO_DEPOT` : il affiche alors les fichiers `.holo` d'un autre dossier que celui où le moteur a été construit. Il est relancé ainsi, pour que ce que Yocthan écrit dans son dossier principal soit ce qui s'affiche.

**Limites**

- Le bouton ▶ n'ouvre que les fichiers de `exemples/` et de `moteur/mondes/`.
- Les erreurs ne sont pas soulignées dans l'éditeur ; elles s'affichent dans le navigateur. Les souligner demandera de brancher le vérificateur du moteur sur VS Code (c'est le `holo check` de la liste des manques de Codex).
- La coloration n'a pas été contrôlée par Claude à l'écran : seules la validité des fichiers et l'installation ont été vérifiées.
- L'essai de mosaïque reste une page à part (`mosaique.html`), parce qu'il part d'une image et non d'un fichier `.holo`.

---

## 2026-10-03 — La mosaïque : la page est faite de points, un par pixel

**Ce que Yocthan a précisé après avoir vu la vue personnage**

- La vue lui plaît, mais passer de « vue web » à « vue personnage » par un bouton donne l'impression de changer de lien, pas d'être dans un métavers. À revoir.
- Il ne veut pas de points lumineux décoratifs au fond. **Tous les éléments de la page sont des points** : le titre « My shop », le fond, les couleurs. Le site est une image découpée en pixels, et chaque pixel est un point, rangé exactement à sa place. Le nombre de points est celui des pixels visibles. Chaque point contient d'autres points ; c'est le zoom qui les révèle.
- Une lettre est traversée par plusieurs pixels : elle contient donc plusieurs points, donc plusieurs mondes, et elle peut aussi être vue comme un objet (son exemple de la personne en casque placée entre le « 3 » et le « D »). Le monde d'un point doit donc se rattacher au caractère auquel il appartient.
- La vue « roadmap » : une carte qui répertorie les mondes actifs à un moment donné, sous forme de gros points, comme la carte des niveaux d'un jeu. Plus tard.
- Le zoom du Big Bang, déjà validé, reste tel quel ; il faut le « redisposer » pour que ce soit performant. Cela servira aussi pour la vidéo et pour des mondes générés par une IA.

**Fait**

- Pull request n° 27 (vue personnage) fusionnée.
- `moteur/src/mosaique.rs` : une image vue comme un ensemble de points. Au repos, un point fait exactement un pixel et l'image ordinaire suffit. En zoomant, les pixels grossissent et deviennent des points lumineux alignés ; au-delà de 40 pixels, chaque point se morcelle en une grille de 4 × 4, et ainsi de suite jusqu'à vingt niveaux. La graine d'un point se calcule à partir de sa place : rien n'est stocké.
- Les calculs donnés à Yocthan se vérifient : on ne dessine jamais plus de points que l'écran ne peut en montrer (un test le contrôle sur soixante zooms successifs). Sur l'image de 1280 × 720, soit 921 600 points au repos, il y a entre 4 000 et 12 000 points à l'écran quel que soit le niveau.
- Essai visible : `http://localhost:8080/mosaique.html`, sur une image fixe de la boutique. Molette ou pincement pour zoomer, glisser pour se déplacer.
- Les points décoratifs autour de la feuille, en vue personnage, sont éteints.
- Nouvel outil `moteur/outils/capturer.mjs` : prend une capture après une attente en temps réel. Les captures précédentes étaient prises trop tôt, avant le premier dessin.
- Correction : un titre dont le style fixait les marges n'était plus centré dans la page. Le centrage vient maintenant du conteneur.
- Tests du cœur : 49 sur 49.

![Les pixels du titre deviennent des points](images/2026-10-03-mosaique-2-points.png)

![Chaque point se morcelle en grille](images/2026-10-03-mosaique-3-morcele.png)

![Quatre niveaux plus bas](images/2026-10-03-mosaique-4-profond.png)

**Limites, dites à Yocthan**

- C'est une image fixe : Chrome ne donne pas au moteur les pixels d'une page vivante. Les boutons ne marchent donc pas dans cet essai.
- Les mondes des points ne sont pas encore rattachés aux caractères : avec une image, le moteur ne sait pas quel pixel appartient à quelle lettre. Il faudra partir de la page elle-même.
- On zoome et on se déplace à plat. Regarder la page de biais, et la vue « roadmap », ne sont pas faits.
- Le bord des lettres montre des points bleus et orange : c'est le lissage des polices de l'écran, présent dans l'image d'origine.

---

## 2026-10-03 — La vision se précise (le téléphone de Trevor) ; première vue « personnage »

**Ce que Yocthan a précisé**

- Par défaut, on est devant un site web normal. Le métavers vient en plus.
- Un site est comme une feuille de papier : plate, mais c'est un vrai objet dans un espace en 3D.
- Dans GTA, quand Trevor regarde son téléphone, le menu est en 2D à l'intérieur du monde en 3D. Pareil ici : si l'utilisateur est un **personnage**, il voit le site comme une surface posée dans le monde ; sinon il voit le site en 2D classique. Trois situations : il utilise le web (vue classique), il joue son propre personnage (par ses yeux), il joue à un jeu (personnage vu de dos).
- La peau paraît lisse ; en zoomant, on découvre les microbes. Le zoom révèle les mondes.
- Une option « rendre en 3D » donnera du volume à un élément important, comme des lettres entre lesquelles on peut se placer.
- Il a ainsi corrigé la proposition de Claude (tout piloté par le zoom, la page qui « devient » un lieu) : rien ne change de nature, c'est le même objet vu d'une autre place. Et il a relevé, avec raison, que dans la première démo la page disparaissait pour laisser place à un autre écran.

**Fait**

- Yocthan a confirmé qu'il voit bien les points en 3D dans l'atelier : la pull request n° 26 est fusionnée.
- Feu vert pour un premier pas, à corriger ensuite petit à petit. `moteur/web/page.html` a maintenant un bouton « Vue personnage » : la même page, sans être rechargée, devient une feuille posée au centre d'un monde, avec les points lumineux autour. Elle tourne avec le monde quand on le fait tourner, grandit quand on s'approche, et reste une vraie page : le texte se lit, le bouton marche. « Vue web » ramène au site classique.
- Moteur : `Navigation::surface` dit où poser une surface plate dans le monde ; `changer_de_monde` passe d'un monde à l'autre sans relancer la carte graphique. Tests du cœur : 42 sur 42.
- Un sommaire, `http://localhost:8080/accueil.html`, ouvre chaque démonstration à part.

![La boutique vue en personnage : une feuille dans le monde](images/2026-10-03-boutique-holo-personnage.png)

- Yocthan a demandé de lancer le fichier `.holo` lui-même dans Chrome, et non `page.html`. Le serveur local ouvre maintenant un `.holo` par sa propre adresse (`http://localhost:8080/exemples/boutique-comparee/boutique.holo`) : quand le navigateur demande le fichier pour l'afficher, il reçoit la porte d'entrée du moteur, qui va chercher le fichier. C'est le rôle que tiendra plus tard un navigateur qui sait lire le `.holo`.

**Erreur en route**

- Le serveur renvoyait encore le texte brut : l'outil qui a écrit la modification avait transformé deux `` en caractères invisibles dans le test. Trouvé en lançant une copie du serveur avec un affichage de contrôle.

**Provisoire, à décider**

- Le monde où la page est posée n'est écrit nulle part dans le fichier : sa graine est tirée du nom de la page. Le langage ne sait pas encore dire « un monde qui contient une page ».
- C'est une vue par les yeux (première personne). Le personnage vu de dos demande un corps à dessiner : chantier à part.
- La feuille est toujours dessinée devant les points, même ceux qui devraient passer devant elle.

**Ce qui n'est pas vérifié**

- Les captures montrent l'affichage des trois états. Les gestes (faire tourner le monde, zoomer, passer d'une vue à l'autre plusieurs fois) n'ont pas été essayés par Claude dans un vrai navigateur. La pull request reste ouverte jusqu'au retour de Yocthan.

**Note sur les captures**

- Les captures sans écran sortaient noires parce qu'elles étaient prises trop tôt : avec un délai de 15 secondes de temps simulé, la vue en profondeur apparaît.

---

## 2026-10-03 — Proposition : le même web en 3D, et la comparaison avec les concurrents

**Fait**

- Yocthan a demandé de s'inspirer des éléments de HTML, CSS et JavaScript pour créer ceux de HoloCode, de voir comment chacun se représente en 3D, d'innover, et de comparer avec les frameworks JavaScript et les autres langages. Claude a écrit une proposition : [`proposals/Claude/web-en-3d-2026-10/`](../../proposals/Claude/web-en-3d-2026-10/README.md). Rien n'y est décidé, rien n'est ajouté au moteur.
- Idée principale, « option C » : le plan de la page (`H1`, `H2`, `H3`) devient la profondeur. Chaque titre avec son contenu devient un point ; zoomer révèle les sous-titres. La règle d'`ADR-020` qui interdit de sauter un niveau y trouve son utilité.
- Deuxième idée : le lien est une porte. `Link(to: "autre.holo")` est un lien à plat et un point dans lequel on entre en profondeur.
- Comparaison écrite sans embellir : les signaux (Solid, Vue, Svelte) et l'arbitre (Elm) existent ailleurs ; A-Frame déclare déjà de la 3D en balises ; VRML a échoué dans les années 1990 ; Elm est juste sur le fond et peu adopté. Ce qui est propre à HoloCode est l'assemblage : une description pour deux vues, la profondeur tirée du sens, un monde dans une graine, tout vérifié avant d'exécuter.

- Yocthan veut observer le site et le point séparément avant de donner sa réflexion. Ajout d'un sommaire (`http://localhost:8080/accueil.html`) qui ouvre chaque démonstration à part : la boutique en HoloCode, la boutique en web, le Big Bang seul, et le point de l'atelier seul (`moteur/mondes/atelier.holo`).

**En attente**

- La pull request n° 26 (vue à plat et entrée dans un point) n'est pas fusionnée : Yocthan n'a pas encore dit s'il voit les points en 3D derrière le panneau.

---

## 2026-10-03 — La boutique en HoloCode s'affiche dans Chrome : vue à plat et entrée dans le point

**Fait**

- Yocthan a demandé à voir les deux versions tourner dans Chrome. La version web tournait déjà ; pour la version HoloCode, il a fallu construire l'affichage. C'est l'étape « vue à plat » prévue, avancée d'un cran.
- `moteur/src/plat.rs` : d'un fichier `.holo` vérifié, le moteur fabrique une page web ordinaire, HTML et CSS (`ADR-011`). L'auteur n'en écrit pas. Tout ce qu'il écrit est échappé : aucun texte ne peut devenir du code.
- `moteur/src/regles.rs` : les noms en double, les règles (`On(Open.tap, effect: Workshop.enter)`), les capacités inconnues et les budgets dépassés sont maintenant vérifiés. La page d'accueil (`moteur/web/page.html`) ne décide de rien : chaque toucher est envoyé au moteur, qui répond par les effets demandés.
- Entrer dans le point ouvre la vue en profondeur du point (le Big Bang, avec sa graine, sa couleur et sa palette) ; le contenu du monde se lit sur un panneau devant (option A d'`ADR-018`). « Back to the shop » ramène à la page et met la vue en profondeur en pause.
- `color` et `palette` d'un `Point` sont lus par le moteur (`ADR-017`) ; sans couleur imposée, la graine décide, dans les deux vues.
- Le serveur local sert aussi `exemples/`. Adresses : `http://localhost:8080/page.html` (HoloCode) et `http://localhost:8080/exemples/boutique-comparee/web/index.html` (web).
- Tests du cœur : 40 sur 40. Les seize cas refusés de la suite de conformité sont maintenant tous refusés par le moteur.

![La boutique en HoloCode, vue à plat](images/2026-10-03-boutique-holo-page.png)

![L'atelier : le panneau lisible devant la vue en profondeur](images/2026-10-03-boutique-holo-monde.png)

**Ce qui n'est pas vérifié**

- Sur les captures prises sans écran, la zone de dessin 3D reste noire (le Big Bang seul aussi, avec cette méthode de capture) : les points derrière le panneau n'ont pas pu être contrôlés par Claude. À Yocthan de dire ce qu'il voit dans son Chrome.

**Limites**

- Le point `Storeroom`, écrit dans le monde de l'atelier, apparaît sur le panneau mais on ne peut pas encore y entrer : la vue en profondeur ne connaît que les points nés de la graine.
- Le fond donné au monde par son style est caché par la zone de dessin 3D.
- Une seule vue en profondeur par page pour l'instant.
- Du Markdown, seuls le gras et l'italique sont rendus.
- L'option B d'`ADR-018` (chaque bloc devient une boule) n'est pas faite.

**Erreurs en route**

- Premier essai : zoom de départ à 3,7, qui faisait entrer dans un enfant du point au lieu du point lui-même. Corrigé à 2.
- Le thème de la page n'atteignait pas le monde intérieur (police différente) : le monde est maintenant rangé dans la page.

---

## 2026-10-03 — La même boutique écrite deux fois : HoloCode, et HTML, CSS, JavaScript

**Fait**

- Avant d'afficher la boutique, Yocthan a demandé un même exemple dans les deux écritures, avec tout le vocabulaire du langage, tel qu'on s'en servirait pour un métavers. Il est dans [`exemples/boutique-comparee/`](../../exemples/boutique-comparee/README.md).
- `boutique.holo` : 62 lignes utiles. La version web : 178 (36 de HTML, 61 de CSS, 81 de JavaScript). Pour la page seule, l'écart est faible ; il vient presque entièrement du point et du monde, que le web doit fabriquer à la main.
- La version web fonctionne dans un navigateur. Ses graines sont calculées en JavaScript avec `BigInt` et donnent les mêmes valeurs que le moteur (vérifié sur quatre valeurs de référence).
- La version HoloCode est relue par un test du moteur, qui contrôle aussi qu'aucun bloc, aucun paramètre et aucun réglage de style ne manque dans l'exemple. Tests du cœur : 29 sur 29.

![La version web de la boutique](images/2026-10-03-boutique-web.png)

**Limites, dites dans le README de l'exemple**

- La version HoloCode n'est pas encore affichée : la comparaison porte sur l'écriture, pas sur le résultat à l'écran.
- Les imports y sont en commentaire (lus par le moteur, pas appliqués). Les unités de longueur et de durée, `true` et `false` n'ont pas encore d'emploi dans le langage.
- La version web triche : le monde intérieur est chargé d'avance et caché, et ses points sont sur un cercle, pas sur une sphère.

**Erreur corrigée en route**

- Claude avait écrit `margin: 0 auto` dans le thème de la page : c'est de la disposition (centrer), que le vérificateur refuse à juste titre. Retiré de la version HoloCode, et rangé dans la partie « disposition » de la version CSS.

---

## 2026-10-03 — Les styles s'écrivent comme en CSS, avec sept règles pour servir aussi aux jeux

**Fait**

- Ordre de travail validé par Yocthan (point B) : d'abord la boutique dans les deux vues, les mesures sur téléphone en parallèle, le langage « système » plus tard.
- Yocthan a relevé que le style d'un jeu n'est pas celui d'un site, et demandé une solution cohérente dès le départ. Comparaison faite : le cœur commun est « un paquet nommé de réglages d'apparence qu'on pose sur une chose » (`.card` en CSS, un matériau dans Unity). Sept règles proposées et validées, écrites dans `ADR-017`.
- Sa condition : l'écriture reste celle du CSS de base, avec des accolades, pas un bloc `Style(...)` répété. Claude recommandait l'écriture en blocs ; Yocthan a décidé autrement, et sa raison tient : le CSS est simple à écrire, c'est son mécanisme qui pose problème. La question ouverte d'`ADR-017` est fermée.
- Moteur : le lecteur accepte `P.card(...)` et lit les styles après le bloc racine ; nouveau fichier `moteur/src/styles.rs` qui les vérifie. Tests du cœur : 28 sur 28. Les styles ne sont pas encore appliqués à l'écran.
- Suite de conformité : 22 cas (6 acceptés, 16 refusés), dont la boutique `06` et cinq refus (`E12` à `E16`).

**Choix faits par Claude, à réexaminer si besoin**

- Les blocs `Theme(...)` et `Style(...)` sont retirés : avec l'écriture CSS, le thème est le style de `Page` ou de `World`. Les garder aurait fait deux écritures pour la même chose.
- Les noms des réglages sont ceux du CSS (`font-size`, `border-radius`), mais un seul nom par réglage : `background-color` est refusé avec « écris `background` ».
- Le vérificateur ne connaît que quinze réglages, les tailles en `px` et en `%`, et une trentaine de noms de couleur. C'est plus strict que le CSS ; la liste grandira.
- Un bloc ne porte qu'un nom de style (`P.card`, pas `P.card.big`), pour ne pas avoir à départager deux styles nommés.

**Limites**

- Les accolades restent interdites dans un bloc (`ADR-015`) ; elles ne servent qu'aux styles.
- Importer un fichier de styles n'est pas encore possible : le moteur refuse tous les imports.

---

## 2026-10-03 — `Text`, `P` et `H1` : le texte comme en HTML, sans ses défauts

**Fait**

- Yocthan a demandé une comparaison avant de choisir entre `P`/`H1` et `Text`, en relevant lui-même que `Text` est le mot qui sert dans un terminal. Résultat : `Text` est le bloc de base (du texte sans rôle) ; `P` et `H1` à `H3` sont un `Text` avec un rôle. Décision écrite dans `ADR-020`, acceptée par Yocthan.
- Il a demandé pourquoi les minuscules seraient un mal. Réponse franche donnée : la majuscule n'est pas une nécessité technique, c'est une aide à la lecture (`Text(...)` le bloc, `text:` le réglage) ; le vrai défaut serait d'accepter deux écritures. Règle retenue : un bloc commence par une majuscule, un réglage par une minuscule ; `h1` est refusé avec « écris `H1` ».
- Nouvelle consigne de travail : à chaque ajout au langage, comparer d'abord les options et vérifier qu'on ne répète pas un défaut de HTML, de CSS ou de JavaScript. Écrite dans `AGENTS.md`.
- Moteur : nouveau fichier `moteur/src/blocs.rs`. Il vérifie que chaque bloc d'un fichier existe et que les titres ne sautent pas de niveau. Rien n'est encore affiché pour ces blocs. Tests du cœur : 23 sur 23.
- Suite de conformité : 16 cas (5 acceptés, 11 refusés). Le cas `01` utilise `H1` et `P` ; nouveaux cas `05`, `E09`, `E10`, `E11`.
- Question de Yocthan : le point et le pixel sont-ils la même chose, et que font Unreal et Unity ? Réponse : un pixel est sur l'écran et ne contient rien ; les moteurs 3D parlent de sommets, de particules, de voxels, de nuages de points. Deux techniques proches de sa vision, à garder en tête : le *Gaussian splatting* (un monde fait de points lumineux, tiré de photos ou de vidéos) et Nanite d'Unreal (le détail ajusté au pixel d'écran).

**Choix fait par Claude, à réexaminer si besoin**

- Yocthan avait approuvé « le vérificateur signale » un saut de niveau. Claude en a fait un refus, pas un simple avertissement : la suite de conformité ne connaît que « accepté » et « refusé », et tolérer l'erreur est justement ce qui a rendu ce défaut courant en HTML. La fiche prévoit de revenir à un avertissement si cela gêne.

**Limites**

- Le Markdown écrit dans un texte (`# Titre`) n'est pas lu par le moteur : la règle des niveaux ne porte que sur les blocs.
- `P.card(...)` n'est pas encore accepté par le lecteur ; cela attend la décision sur la place des styles.

---

## 2026-10-03 — Le point est un pixel ; la proposition de coordination de Codex est intégrée

**Fait**

- Yocthan a expliqué d'où vient le mot « point » : sa réflexion part du pixel. Une image est faite de pixels ; un monde est fait de points ; quand on zoome sur un point, il se divise en mondes. Il a laissé Claude trancher le nom selon ce que le mot désigne en 3D : **le bloc s'appelle `Point`**, défini comme « le pixel de l'Holoverse ». `Pixel` aurait heurté l'unité `px` des styles ; `Voxel` désigne un cube dans une grille. Fiche `ADR-016` complétée.
- Idée technique qui en découle, notée pour un prochain sprint : mesurer la limite de perception en pixels d'écran, et ne développer le monde d'un point que lorsqu'il occupe assez de pixels pour qu'on y voie quelque chose. Autre idée de Yocthan : partir d'une image ou d'une vidéo dont chaque pixel serait un point.
- Sur le texte, Yocthan veut l'écriture de HTML et CSS sans leurs défauts : on écrit un paragraphe, un titre, et on le stylise facilement. Claude propose des blocs `P` et `H1`, et des styles par type de bloc ou par nom à point ; à confirmer.
- Codex a répondu au prompt sur l'outillage (PR n° 19) : pas de pont direct entre IA, GitHub comme boîte aux lettres, pas de serveur MCP pour l'instant, la mesure sur téléphone par `adb` et `chrome://inspect`, et une liste priorisée des manques du projet. Yocthan a délégué la décision à Claude : la proposition est fusionnée après lecture complète, et ses parties légères sont appliquées. Un modèle de pull request et un modèle d'issue sont actifs dans `.github/`, onze étiquettes sont créées, la règle de fusion et la boîte aux lettres sont écrites dans `AGENTS.md`.

**Erreur commise**

- Claude avait daté du 4 octobre l'étape précédente et les fiches `ADR-016` à `ADR-019`, alors que tout s'est passé le 3. Les dates sont corrigées.

**Ce qui n'a pas été repris de Codex, et pourquoi**

- Le commentaire obligatoire « VALIDÉ POUR FUSION » de Yocthan sur chaque pull request : Yocthan veut aller vite et n'avoir presque rien à faire. Son accord n'est exigé que pour les pull requests des autres IA et pour celles qui changent une décision.
- Le contrôle automatique qui refuse une pull request sans les champs du modèle : gardé en exemple dans le dossier de Codex, pas activé, tant qu'on n'a pas vu le modèle servir.

---

## 2026-10-03 — Le langage passe en anglais ; quatre décisions sur la forme

**Fait**

- Yocthan a pris point par point les décisions en attente sur le langage. Les mots sont **en anglais** (contre l'avis de Codex, qui proposait le français d'abord), avec une règle née de sa remarque sur `split` : un mot que les programmeurs connaissent garde son sens. D'où `fragments` et non `split`, `leave` et non `exit`, `On` et non `When`, `brightness` et non `light`, `children` comme en Flutter. `ADR-016`.
- La forme : un `Theme`, des styles nommés avec le point du CSS (`.card`), et des réglages par bloc ; trois façons de colorer un point. `ADR-017`. La vue en profondeur : option A, seuls les points ont de la profondeur ; l'option B sera montrée pour comparer. `ADR-018`. Le texte s'écrit sans artifice, comme « Hello World » ; un `.md` ne s'importe que pour un long texte. `ADR-019`.
- Le moteur et la suite de conformité sont traduits. Le Big Bang s'écrit `Point(name: Origin, seed: 1, brightness: 1.0, fragments: 12)`. Les unités de taille deviennent `KB`, `MB`, `GB`. Le moteur refuse les anciens mots français en indiquant le mot à écrire (cas `E08`). La suite compte douze cas, et le moteur en lit trois directement dans ses tests : c'est le premier lien réel entre les deux. 19 tests.

**Reste ouvert**

- Le nom du bloc `Point` (`Point`, `Sphere` ou `Orb`) : pour un programmeur, `Point` est une coordonnée.
- La place des styles dans le fichier.

**Leçon**

- Un outil de Claude altérait les barres obliques inverses dans les commandes longues ; un test a ainsi été écrit avec un vrai saut de ligne au lieu de `\\n`. Sans gravité, mais les modifications de code passent désormais par des fichiers de script ou par l'éditeur, plus par des commandes en ligne.

---

## 2026-10-03 — La revue de Codex, et trois défauts corrigés

**Fait**

- Codex (ChatGPT) a répondu au prompt de Yocthan par une pull request (n° 15) rangée dans `proposals/GPT5.6/revue-2026-10-03/` : une revue exécutée (18 tests, compilation WebAssembly, suite de conformité, 501 856 octets Brotli mesurés), un avis sur chaque décision, des réponses aux trois questions du langage, une boutique complète en `.holo` et des sondes reproductibles. Lecture conseillée : `proposals/GPT5.6/revue-2026-10-03/README.md`.
- Trois défauts réels relevés par Codex sont corrigés dans le moteur : les graines passaient par un nombre flottant (2^53 + 1 devenait 2^53), elles sont désormais lues comme des entiers exacts ; un `import` était accepté puis ignoré en silence, il est refusé tant qu'il n'est pas appliqué ; le test qui devait figer les valeurs des graines était une tautologie, il fige maintenant de vraies valeurs. 19 tests.
- Les chiffres sont mis au propre : 19 tests et non 17 ; poids en octets exacts avec Ko = 1 000 octets (502 435 octets transférés) ; « rien n'est stocké » devient « le décor est régénérable » ; la pile de navigation coûte 16 octets par niveau ; la mémoire affichée par la page n'est que le tas JavaScript.
- Codex recommande, pour le langage : un vocabulaire français canonique en v0.1 (`import`, `module`, `pont` gardés), puis éventuellement un profil anglais par table d'alias ; un `Theme` de page plus des paramètres locaux, avec une priorité simple ; `couleur: graine` par défaut, `couleur: "#..."` ou `palette: [...]` pour l'auteur. Il propose de réordonner le travail : la boutique dans les deux vues, la navigation fiable, la mesure téléphone, puis une action avec état arbitré ; les ponts avancés et le navigateur natif ensuite. Ces choix restent à Yocthan.

**Erreurs et leçons**

- Claude a fusionné la pull request de Codex par erreur : elle portait le numéro 15 que Claude croyait être celui de sa propre pull request « pause », ouverte au même moment ; il l'a sortie du mode brouillon et fusionnée sans l'avoir lue. Aucun dégât, parce que Codex avait respecté la règle et n'avait rien écrit hors de son dossier, mais la règle « ne fusionner que ce qui a été lu et testé » a été enfreinte. Leçon : vérifier l'auteur et la branche d'une pull request avant de la fusionner, jamais seulement son numéro.
- Les trois défauts corrigés étaient dans du code que Claude avait écrit et testé : un test tautologique passe toujours. Leçon : un test doit pouvoir échouer.

---

## 2026-10-03 — Un bouton pause, et le dossier de Yocthan remis sur `main`

**Fait**

- Yocthan a demandé une pause pour le Big Bang, parce qu'il sentait la mémoire monter. Un bouton « pause » est ajouté à côté de « mesures » (PR n° 15) : le moteur ne calcule ni ne dessine plus rien jusqu'à « reprendre ». Le monde se met aussi en pause tout seul quand l'onglet est caché. Ce que la pause économise, c'est le processeur, la carte graphique et la batterie ; la mémoire du moteur, elle, est fixe (le moteur lui-même, environ 2 Mo de WebAssembly plus ce que Chrome réserve) et ne grandit pas avec la profondeur : un niveau traversé pèse 16 octets.
- Le dossier VS Code de Yocthan était resté sur une vieille branche du 21 septembre : il ne voyait ni le moteur ni les fichiers `.holo`. Il est remis sur `main`. Codex (ChatGPT) a déposé sur sa machine un dossier `TestByYou/` (une démonstration pédagogique qui extrait un bloc `holo` d'un `.md`) et une copie `revue-codex-2026-10-03/` ; ils ne sont pas dans Git.
- Explication en arbres, à la demande de Yocthan : l'arbre d'un fichier `.holo` (l'équivalent du DOM) et l'arbre du chemin, du fichier aux pixels de Chrome.

**Leçon**

- Un serveur lancé par Claude s'arrête au bout de deux heures ; il est maintenant lancé détaché de la session, et tient jusqu'au redémarrage du PC.

---

## 2026-10-03 — Le journal, AGENTS.md, et GitHub comme canal entre les IA

**Fait**

- Création de ce journal et d'`AGENTS.md` (PR n° 13), à la demande de Yocthan : un rapport tenu à chaque étape, et un point d'entrée pour toute IA branchée sur le projet.
- Rafraîchissement de mémoire sur le langage, à la demande de Yocthan : origines, décisions, correspondance HTML/CSS/JavaScript → blocs/paramètres/règles, démonstration pas à pas depuis un fichier vide, comparaison avec Dart/Flutter et avec le web classique. Trois questions restent à trancher : les mots du langage (français ou anglais), la forme (paramètres, `Theme`, ou les deux), les couleurs du Big Bang (par la graine, par l'auteur, ou les deux).
- Yocthan a demandé si Claude pouvait contrôler l'ordinateur avec un curseur visible : non depuis Claude Code ; c'est « Claude Cowork » ou « Claude in Chrome ». Pour relier les IA, il a choisi GitHub comme canal unique (commentaires de pull requests, issues) plutôt qu'un pont MCP. Gemini Code Assist est à installer sur le dépôt par Yocthan (une application GitHub ne s'installe pas en ligne de commande).

**À faire**

- Yocthan va soumettre la ligne du projet à Codex (ChatGPT) avec un prompt préparé par Claude ; la réponse arrivera dans le dépôt.
- Trancher les trois questions du langage, puis donner un sens à `Page`, `Texte`, `Bouton`, `Theme`, `Quand` dans le moteur.

---

## 2026-10-03 — Corrections après le premier essai, et publication de tout ce qui manquait

**Fait**

- Yocthan a essayé le Big Bang et a demandé trois corrections, livrées dans la PR n° 11 : les mesures sont cachées par défaut (bouton « mesures » en bas à droite, ou `?mesures=1`) ; on touche une boule pour la viser, et la rotation ne change plus une boule touchée ; la boule visée est signalée par un halo blanc qui respire.
- Les documents de fond (`VISION`, `PARADIGME`, `ARCHITECTURE`, `ROADMAP`, README) sont mis en accord avec les décisions et le sprint (PR n° 10). Le texte d'origine de ChatGPT est conservé ; les ajouts sont marqués et datés.
- Les fiches `HC-010` à `HC-012` et les transcriptions des sessions Claude sont publiées (PR n° 12). L'export est fait par `outils/exporter_sessions_claude.py`, qui ne garde que les messages de Yocthan et les réponses de Claude.
- Ce journal et `AGENTS.md` sont créés (PR n° 13).

![La boule visée, signalée par son halo ; les mesures sont cachées](images/5-boule-visee-mesures-cachees.png)

**Erreurs et leçons**

- Le moteur choisissait la cible tout seul (la boule la plus proche du centre) sans le montrer : pour Yocthan, l'entrée semblait aléatoire. Leçon : tout ce que le moteur décide à la place de l'utilisateur doit être visible, ou laissé à l'utilisateur.
- Les mesures affichées en permanence gênaient l'expérience. Un outil de sprint n'a pas sa place devant l'utilisateur.
- Le serveur de démonstration lancé par Claude s'arrête au bout de deux heures : Yocthan doit le relancer lui-même (`node outils/serveur.mjs` dans `moteur/`).

**À faire**

- Les mesures sur le Samsung Z Flip 5 (Chrome) : elles décident d'`ADR-005` et d'`ADR-010`.
- Yocthan veut maintenant parler du langage : ce que les humains écriront chaque jour, les extensions, l'écriture de sites.

---

## 2026-10-03 — Le sprint Big Bang : la première preuve exécutable

**Fait** (PR n° 9, fusionnée)

- Rust 1.99 installé sur le PC de Yocthan (profil minimal, cible WebAssembly). La connexion était lente (80 Ko/s) ; l'installation par défaut, qui téléchargeait la documentation, a été remplacée par une installation minimale.
- Un moteur en Rust (`moteur/`, 1 529 lignes), compilé en WebAssembly, qui dessine avec `wgpu` : WebGPU, ou WebGL 2 en repli. Il lit un fichier `.holo` de huit lignes au format en blocs d'`ADR-009`, le cas `02-big-bang` de la suite de conformité.
- Un point lumineux se morcelle en douze points quand on zoome ; on s'approche d'un point, on aperçoit déjà le monde qu'il contient, on y entre ; ce monde contient lui-même des points, sans fin ; en dézoomant on ressort. Rien n'est stocké : chaque monde naît de sa graine, un niveau traversé coûte 16 octets.
- 17 tests du cœur en Rust pur, qui passent aussi sur GitHub (job ajouté au flux de tests).
- Mesuré sur le PC : 1 893 Ko de moteur réel, **489 Ko transférés** (cible : moins de 2 Mo), page HTML de 1 Ko identique pour tous les mondes.

| Zoom 0 | Zoom 0,8 | Zoom 3,4 | Zoom 4,6 |
|---|---|---|---|
| ![le point entier](images/1-point-entier.png) | ![morcellement](images/2-morcellement.png) | ![plongée, monde intérieur visible](images/3-plongee-apercu-du-monde-interieur.png) | ![entré, profondeur 2](images/4-entre-profondeur-2.png) |

**Erreurs et leçons**

- `wgpu` ne retombe pas tout seul sur WebGL 2 quand WebGPU existe mais ne donne aucun adaptateur : le repli a dû être écrit à la main, en recréant la zone de dessin (une zone qui a reçu un contexte WebGPU n'en accepte plus un autre).
- Deux tests du cœur ont échoué au premier passage : avance octet par octet dans un texte accentué, et points pas exactement sur la sphère. Corrigés avant toute publication.
- Les images par seconde mesurées sur le PC sans carte graphique n'ont aucun sens ; elles n'ont pas été reportées. La seule mesure qui compte est celle du téléphone, et elle n'est pas faite.

---

## 2026-10-02 — Tout fusionner, puis ne garder que ce qui marche

**Fait**

- Sur ordre de Yocthan, les cinq pull requests en attente depuis le 21 septembre ont été fusionnées dans `main` : les trois prototypes, les décisions, les tests automatiques et la suite de conformité.
- Les tests automatiques ont alors montré ce que les README ne disaient pas : ChatGPT 8 sur 8, Claude 27 sur 27, Gemini 3 sur 4. `main` était rouge.
- Yocthan a précisé la règle : on ne fusionne que ce qui marche, et on revoit le reste. Gemini a reçu un fichier de consignes avec les résultats réels et la revue ; il a rendu une version corrigée (PR n° 7) : 6 tests sur 8, cinq défauts sur huit réellement corrigés. Les deux échecs venaient d'une seule ligne, qui empêchait d'entrer dans un nœud sans enfants. Sur ordre de Yocthan, Claude a corrigé cette ligne dans un commit à son nom : 8 sur 8, `main` au vert.
- Le README de Gemini a été corrigé là où la mesure le contredisait (PR n° 8) : la mémoire comptée n'est pas constante, elle croît de 64 Ko par niveau descendu.

**Erreurs et leçons**

- Claude a fusionné la proposition de Gemini alors qu'un test échouait, parce que l'ordre « tout fusionner » est arrivé avant la règle « seulement ce qui marche ». Une pull request de retrait a été préparée puis fermée sans fusion, la correction ayant suffi.
- Gemini a deux fois déroulé ses tests « à la main » en suivant son intention plutôt que son code. Leçon pour toutes les IA : un résultat de test ne s'annonce pas, il s'exécute.
- Le copier-coller depuis l'interface de Gemini abîme le code (indentation, `__init__` en gras) ; la demande de mettre chaque fichier dans un bloc de code a réglé le problème.

---

## 2026-09-21 — Les décisions, les trois prototypes et la revue de ChatGPT

**Fait**

- Claude a lu le dépôt créé par ChatGPT et donné son avis franc : le paradigme holoscénique n'a pas de primitive nouvelle (ECS, Datalog, règles de production, F#, Inform 7, Verse), et sa valeur possible est la synthèse ; l'originalité du projet est dans la vision de Yocthan (le point qui contient des mondes, le zoom, le web qui devient métavers, le budget de 1 Go).
- Trois prototypes en Python, un par IA : ChatGPT (noyau minimal), Claude (lois, capacités, vérificateur, journal causal), Gemini (nœud fractal, zoom, budget mémoire). Comparés par Claude, juge et partie ; ChatGPT a relevé qu'ils ne testent pas le même problème.
- Yocthan a pris ses premières décisions techniques, inscrites dans `ADR-007` à `ADR-015` : le métavers est une mise à jour du web avec deux vues ; le fichier `.holo` est une vraie source ; des blocs nommés par leur sens, texte Markdown dans les blocs ; moteur en Rust ; ponts JavaScript et CSS seulement en première version ; l'IA crée mais le monde se lit sans elle ; tout changement d'état passe par un arbitre. Il a aussi tranché les propositions de ChatGPT (`ADR-003` à `ADR-005` acceptées, `ADR-006` en proposition).
- ChatGPT a relu l'ensemble (note globale 5,5 sur 10, preuve sur téléphone 0 sur 10) : trop de décisions acceptées avant mesure (quatre sont passées en `EXPÉRIMENTATION`), `ADR-014` trop absolue (reformulée), dossiers non uniformes (uniformisés), pas de tests automatiques (ajoutés), pas de banc commun (suite de conformité créée).

**Erreurs et leçons**

- Claude a d'abord répondu « non » à la question « y aura-t-il des appels ? » avant de se corriger : il y en a, de deux sortes, et une troisième est interdite. Les absolus ne tiennent pas.
- Yocthan a observé que l'holoscénique ressemblait à l'objet : c'est vrai, et la description honnête l'a remplacé dans les documents. Ne pas revendiquer une révolution que le premier programmeur venu démonterait.
- « La documentation court plus vite que le moteur » (ChatGPT) : les trois IA et Yocthan ont fait de la preuve sur téléphone la priorité.

---

## 2026-09-20 — Le cadrage de la vision

Yocthan expose à Claude sa vision (`HC-011`) : remodeler l'usage du web en monde explorable, par sprints de 24 heures, horizon 2030. Le Big Bang : un point lumineux qui se morcelle en points, chacun contenant un monde ; la sphère est aussi le premier personnage. Le premier test doit tenir dans 1 Go et tourner sur n'importe quel téléphone actuel. Genie 3 n'est accessible qu'en inspiration. Yocthan veut un langage entièrement nouveau, verbeux mais simple, dans l'esprit de Dart et Flutter. Claude propose la génération par graine et objecte que battre C, Rust et Zig comme langage généraliste est hors de portée d'une personne seule.

---

## 2026-09-19 — Le faux départ

Claude a reçu le brief de Yocthan et a construit, sans en discuter, une première démonstration (sphère en WebGL, JavaScript, 24 Ko), en commençant même à nommer un langage. Yocthan l'a arrêté : rien n'avait été discuté. Tout a été effacé. Règle née ce jour-là, et depuis toujours respectée : **on discute d'abord ; aucune création de fichier, aucune décision technique, aucun nom sans son feu vert.** Détail dans `HC-010`.
