# Piste 1 — Données structurées et répétition

> Statut : EXPLORATION. Avis de Claude, pas une décision.

## Ce que Codex demandait

Sa ligne, citée telle quelle (issue #82, texte de Codex transmis par Yocthan, écrit sur la base `a142a4c`) :

> | **1** | **Données structurées et répétition** : listes, objets, identifiants stables, filtrage, tri, pagination. | Afficher un catalogue reçu du serveur, rechercher un produit, montrer des résultats sans recopier chaque fiche. |

Il demandait aussi, pour toutes les pistes : l'accessibilité à chaque ajout, et les fonctions coûteuses chargées seulement quand elles servent.

## État vérifié (main, 7a48def, 2026-10-07)

**En une phrase : la moitié est faite.** Recevoir un catalogue du serveur et l'afficher sans recopier les fiches : c'est fait. Chercher, filtrer, trier, paginer : rien n'existe. Et, découverte de cette vérification, une fiche reçue du serveur ne peut pas aller dans le panier avec son prix : seul son premier champ passe, comme texte, par un détour (point 6).

**Ce qui existe** (lu dans le code, puis essayé) :

| Capacité | Où | Décision, leçon |
|---|---|---|
| Une liste de textes ou d'éléments à champs, valeur de la page : `State(articles: [ Item(title: "…", price: 0) ])` ; 100 éléments, 200 caractères par champ, 16 champs au plus | `moteur/src/lists.rs:16-25`, `:47-50`, `:245-287` | ADR-044, ADR-051 ; leçons 68, 71 |
| Changer une liste par un geste : `push`, `remove(item)`, `clear()` ; seulement dans une règle `On` | `lists.rs:209`, `:561-563`, `:704-742` | ADR-044 |
| Le serveur remplit la liste : `data: Data(from: "catalogue.json", every: 30s)` ; seuls les champs déclarés sont repris (une liste déclarée vide, `State(articles: [])`, reprend tous les champs reçus, seize au plus : `lists.rs:347`, `:65`) ; trois niveaux de JSON | `lists.rs:328-359`, `:361-362` ; `state.rs:256-287` | ADR-030, ADR-051 ; leçon 71 |
| Écrire une fiche une fois : `Repeat(items:)` déplié à la lecture (200 éléments, 20 000 blocs, pas de répétition dans une répétition) ; `Repeat(over:)` redessiné quand la liste change | `repeat.rs:25-28`, `:124-126` ; `flat.rs:1021-1028`, `:1144-1237` | ADR-040, ADR-044 ; leçon 49 |
| Un champ dans une ligne : `If(item.done, is: 1)`, `item.done.set(1)`, `.add`, `.sub` | `lists.rs:93-184`, `:674-703` | ADR-057 ; leçon 74 |
| Des lignes gardées : une clé tirée du contenu (`data-key`) ; la page garde les lignes qui n'ont pas changé | `flat.rs:1229-1234` ; `moteur/web/page-engine.js:368-394` | ADR-057 |
| La sûreté : tout texte reçu est échappé ; une image hors du dossier est refusée, mais ce refus efface toute la liste (D10 plus bas) | `lists.rs:816-834` (essai ; il vérifie seulement que « javascript » n'apparaît pas, ce qu'une liste vide satisfait aussi) | ADR-051 |
| La fiche comme composant : `ArticleCard(title: item.title, …)` dans une répétition | `docs/01-holocode/GUIDE.md:921` (avec `Repeat(items:)`) ; essayé aussi dans `Repeat(over:)` : `relecture-1-10/comp-over.holo`, « ok », la fiche s'affiche | ADR-050, ADR-056, ADR-058 |

**Ce qui manque** (chaque point vérifié par une commande, voir plus bas) :

1. **Chercher un mot.** Il n'existe aucun mot pour « contient ». Un texte de la page ne se compare qu'au vide (`state.rs:78-85`, `:1305-1313`) : `If(size, is: "M")` est refusé, même pour la valeur d'un `Choice` (essai E1). Avec un trou : `If(size, is: n)`, où `n` est un nombre de la page, est accepté, et compare seulement « vide ou non » (0 ou 1) à `n` (`state.rs:82-84`, `:836-840` ; `relecture-1-10/text-vs-number.holo` : « ok » ; avec `n: 1`, la condition est fausse pour un texte vide et vraie pour « M », `sonde-trou-et-deplacement.mjs`). Dans une ligne, un champ ne se compare qu'à un nombre ou à un texte écrit, jamais à une valeur de la page (`lists.rs:174`) : `If(item.cat, is: cat)` est refusé (essai E2).
2. **Filtrer selon un choix du visiteur.** Impossible, sauf un contournement : des catégories numérotées et un `If` par catégorie (fichier `essais/contournement.holo`, accepté). Ce contournement ne donne ni le nombre de résultats, ni le message « aucun résultat », et il grossit vite : pour N catégories, la fiche s'écrit N fois, chacune sous N − 1 conditions `If(cat, not: …)` imbriquées (la forme de `contournement.holo`).
3. **Trier.** Rien. L'ordre est celui de l'arrivée ou de l'ajout.
4. **Limiter, paginer, « afficher plus ».** Rien. `Repeat(over:)` montre toute la liste (`flat.rs:1206`). `Data` garde 100 éléments sans rien dire (`lists.rs:338` ; mesure N3 : 150 reçus, 100 gardés). Un fichier de plus de 65 536 octets est ignoré en entier, sans signal (`state.rs:252`, `lists.rs:330` ; mesure N3). `from:` n'accepte ni `?` ni `=` (`state.rs:267`) : on ne peut pas demander « la page 2 » au serveur. Une page n'a qu'un seul `data:`.
5. **Des identifiants stables choisis par l'auteur.** Non faits (ADR-057, « Ce qui n'est pas fait »). La clé d'une ligne vient de son contenu : quand une ligne change (un prix, une quantité), elle est refaite à neuf (`page-engine.js:376-394`) ; si le visiteur avait le focus sur un bouton de cette ligne, le focus disparaît. Plus encore : les lignes inchangées placées après une ligne refaite ou retirée gardent leur nœud, mais sont déplacées une à une par `insertBefore` (`page-engine.js:390-392`), et un nœud déplacé perd le focus. Le placement, recopié dans Node sur un faux conteneur, déplace bien les trois lignes qui suivent une ligne refaite ou retirée (`relecture-1-10/sonde-trou-et-deplacement.mjs`). La perte du focus, elle, est déduite ; non vérifiée dans un navigateur.
6. **Du catalogue au panier.** Impossible avec le prix, pour une liste reçue du serveur. Dans les règles d'une ligne de `Repeat(over:)`, ces cinq écritures sont refusées : `cart.add(item.price)` (T1), `cart.push(Item(title: item.title, price: item.price))` (T4), `cart.push(item)` (T8), `chosen.set(item.title)` (T11), `amount.set(item.price)` (T12). Seul `chosen.set(item)` passe (T10) : pour une liste de textes, il copie le texte ; pour une liste à champs, il copie le premier champ seulement, comme texte (`lists.rs:626`, `:669-671`, `:76-82` ; `relecture-1-10/t10b` : « ok », `chosen` vaut « Barque »). Par ce détour, `effect: [chosen.set(item), cart.push(Item(title: chosen, price: 0))]` met au panier le titre de la fiche touchée, mais jamais son prix (`relecture-1-10/detour.holo` et `detour.test` : « ok, 6 ligne(s) jouée(s) »). La même chose marche avec une répétition écrite à la main (`Repeat(items:)`, T2, à condition que chaque `Item` ait sa clé, `key:`, puisque la ligne a un bouton nommé). `Prices` ne s'applique pas à une liste (T3). La raison : une demande n'accepte comme quantité que le nom d'une valeur de la page (`state.rs:1116-1119` pour les nombres ; `lists.rs:592-613` pour `push` ; `lists.rs:624-627` pour un texte).
7. **Défauts trouvés en vérifiant** (aucun n'était signalé) :
   - **D1** : deux `Repeat(over:)` sur la même liste sont acceptés (E4), mais le second reçoit le modèle du premier (mesure N1 après un ajout). Cause : `flat.rs:1242` prend la première répétition qui porte ce nom. Dans le navigateur, cela arrive dès que le moteur démarre : `displaySite()` redessine toutes les listes (`page-engine.js:552`, puis `:357-367`) ; déduit du code, et `list_html` rend déjà le modèle A seul pour l'état de départ (`relecture-1-10/sonde-relecture.mjs`).
   - **D2** : `Repeat(over:)` posé dans une `List` donne un seul `<li>` pour toutes les lignes (E5 ; `flat.rs:703-715`). Un lecteur d'écran devrait annoncer « liste, 1 élément » (déduit du HTML ; non essayé au lecteur d'écran). ADR-044 le notait déjà dans « Restent à faire » (`ADR-044:36`).
   - **D3** : `Data` n'a ni nom ni signal (`rules.rs:10-25`) ; un échec est avalé : un fichier absent (réponse 404) à `page-engine.js:278`, un serveur muet à `:283`. La page ne peut dire ni « chargement », ni « échec », ni réessayer à la demande (seul `every:` redemande le fichier, à son rythme). L'élément d'attente de la leçon 71 (« Chargement… ») reste à l'écran si le fichier manque.
   - **D4** : la page fabriquée d'avance par le serveur montre l'élément d'attente, pas le catalogue. `holo html` sur la leçon 71 donne « Chargement… 0,00 euros » (le serveur fabrique sans les données : `moteur/outils/server.mjs:66`). Un robot de recherche, ou un visiteur sans JavaScript, ne voit aucune fiche.
   - **D5** : un prix à virgule ou négatif devient vide, sans rien dire (N3 : « Virgule :  euros »).
   - **D6** (petit) : avec `Image(source: item.image)`, l'élément d'attente doit porter un vrai nom de fichier ; `image: ""` est refusé, avec un message qui parle de l'image et pas de l'élément d'attente. Une liste déclarée vide, `State(articles: [])`, évite l'élément d'attente (accepté : `essais/n2b.holo`) ; mais ses champs ne sont plus vérifiés, et tous les champs reçus sont repris (`GUIDE.md:1331`, `lists.rs:347`). Le langage n'a aucune façon de déclarer les champs d'une liste sans y mettre un élément.
   - **D7** (petit) : `holo test` ne vérifie que le nombre d'éléments d'une liste, pas leurs champs (`expect cart.qty = 2` : « la page n'a pas de valeur »). `tap` ne sait pas viser une ligne ; il faut `signal More.tap@1`.
   - **D8** (message) : l'erreur de T1 conseille « déclare-le sur la page, state: State(item.price: 0) », ce qui est impossible.
   - **D9** (général) : un paramètre écrit deux fois est accepté, et le second est ignoré sans rien dire (`Block::argument` prend le premier : `holo.rs:116-118` ; `blocks.rs:83-123` ne cherche pas les doublons). Deux `data:` sur une page : `holo check` répond « ok », et le moteur ne lit que le premier (`data()` rend `a.json|0`). Même chose pour `Page(title: "a", title: "b")` (la page fabriquée porte `data-title="a"`) et `Button(text: "x", text: "y")` (le bouton dit « x »). À l'inverse, dans `Data(from: "a.json", from: "b.json")`, c'est le second qui gagne (`state.rs:263-269` ; `data()` rend `b.json|0`, `relecture-1-10/sonde-data2.mjs`) : la règle n'est même pas la même partout. Seuls `State`, les champs d'un `Item` et `Prices` refusent un nom donné deux fois (`state.rs:1070-1071`, `lists.rs:269-270`, `state.rs:903-904`). Cela contredit l'esprit d'ADR-037 (« Une faute n'est jamais avalée », `ADR-037:19` ; `AGENTS.md` le résume en « aucun paramètre inconnu n'est avalé en silence », et `blocks.rs:18` en « jamais avalé en silence »).
   - **D10** (trouvé à la relecture) : une seule image refusée efface toute la liste, sans rien dire. Si une fiche reçue a une image hors du dossier (`javascript:…`, `../x.svg`) ou pas d'image du tout, `Image(source: item.image)` échoue, `lines()` rend une erreur, et `list_lines` rend une liste vide (`flat.rs:670-676`, `:1216`, `:1246` `unwrap_or_default`). Mesure dans Node (`relecture-1-10/sonde-image.mjs`) : trois fiches reçues, une image fautive, 3 éléments dans l'état, 0 ligne affichée ; `{articles}` dit pourtant 3. Avec `essais/p1/aujourdhui.holo` et les douze fiches de l'essai, une seule image `../x.svg` fait passer de 12 lignes à 0 (`relecture-1-10/sonde-r15.mjs`). L'essai `lists.rs:833-834` ne le voit pas.

**Documents en retard** (lus, non modifiés) :

- `docs/01-holocode/GUIDE.md:590` dit « Il n'y a pas de « sinon » » (faux depuis ADR-039) ; `:816` « pas de liste de choix, pas d'envoi à un serveur » (faux depuis ADR-038 et ADR-042) ; § 11 (`:1801-1806`) annonce comme absents l'envoi d'un fichier, la condition sur un champ et l'emplacement d'un composant, tous construits (ADR-059, ADR-057, ADR-058) ; il n'y dit pas ce qui manque vraiment (chercher, filtrer, trier, paginer).
- `docs/01-holocode/COMPARAISON-WEB.md:159-160` : « fait : des textes », « pas de liste, pas d'envoi » (faux depuis ADR-051 et ADR-042) ; `:116` parle encore de `Part` ; la partie 5 (`:186-189`) dit qu'il n'y a ni formulaire, ni panier, ni tableau, ni disposition.
- `docs/01-holocode/TABLEAU-WEB.md:621` donne 90 % à « tableaux, objets » sans ligne pour filtrer, trier, paginer : à mon avis trop haut.
- `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md:419` dit que `Data` donne des signaux (`done`, `failed`) : il n'en donne aucun.
- `docs/01-holocode/NOMS.md:210` range `h4`, `h5`, `h6` parmi les refus (ils existent depuis ADR-036) ; la table « Pas encore là » (`:220-237`) est presque entièrement construite ; la date « Mis à jour le 2026-10-06 » (`:5`) ; `:201` dit que `import` et `module` sont « lus, pas encore appliqués ».
- `exemples/site-reference/catalogue.holo:1-2` : « le langage n'a pas encore de liste répétée » ; `exemples/site-reference/README.md:16` cite `Part`, devenu `Component`.

**Ce que j'ai exécuté** (moteur en ligne de commande `moteur/target/release/holo.exe`, reconstruit à 13:56 depuis `main` à `1119361` = `7a48def` plus la PR 139, qui ne touche que la vérification d'un fichier de thème seul ; moteur léger WebAssembly copié de `moteur/web/pkg-light/`, sha256 `76c8fc6a…`, dans Node 22.21.0 sur le PC, Windows 11). Fichiers d'essai : `essais/` à côté de ce rapport.

```text
E1 $ echo 'Page(… State(size: "") … Choice(value: size, …), If(size, is: "M", …) ])' | holo check -
   ligne 1, colonne 129 : « If(size, is: …) » attend un nombre entier, ou le nom d'une autre valeur
E2 $ echo 'Page(… Repeat(over: articles, children: [ If(item.cat, is: cat, …) ]) ])' | holo check -
   ligne 1, colonne 136 : « If(item.cat, is: …) » attend un nombre entier, ou un texte entre guillemets
E4 deux Repeat(over: articles) : ok          E5 Repeat(over:) dans List : ok
$ holo html e5.holo
   <ul class="holo-List"><li><div class="holo-Lines" data-list="articles"><div class="holo-line" …>a</div><div class="holo-line" …>b</div></div></li></ul>
T1 « cart.add(item.price) » : aucun nombre ne s'appelle « item.price » ; déclare-le sur la page, state: State(item.price: 0)
T2 (Repeat(items:)) : ok
T3 le prix « articles » ne correspond à aucune valeur : déclare sa quantité, state: State(articles: 0)
T4 le champ « title » prend un texte, un nombre, ou le nom d'une valeur de la page
T8 « cart » a des éléments à champs : cart.push(Item(…))
T10 chosen.set(item) (liste de textes) : ok
T11 « chosen » est un texte : on demande seulement « chosen.set("") », ou « chosen.set(autreTexte) »
T12 « amount.set(item.price) » : aucun nombre ne s'appelle « item.price » ; …
$ holo html exemples/lecons/71-liste-a-champs.holo /exemples/lecons/   (lignes de la liste, balises retirées)
   data-list="articles"> Chargement… 0,00 euros <button …
$ node essais/sonde.mjs
   N1 après un ajout, lignes rendues pour « articles » : A a A z
   N1 => le second Repeat recevrait-il son modèle B ? false
   N3 reçu 150 articles, gardés : 100
   N3 prix étranges : Virgule : euros Moins : euros Texte : 12,00 euros Juste : 12,50 euros
   N3 fichier de 74194 octets (plus de 65 536) : articles gardés = 1 ; l'état reste celui de départ ? true
   N2 [12] input 0.307 ms ; arbitrate 0.294 ms ; list_html 0.530 ms ; conditions 0.233 ms (médianes)
   N2 [100] input 0.965 ms ; arbitrate 0.969 ms ; list_html 2.673 ms ; conditions 0.646 ms (médianes) ; HTML des lignes 35 603 octets ; état 8 506 octets
$ holo check essais/p1/aujourdhui.holo && holo test essais/p1/aujourdhui.holo essais/p1/aujourdhui.test
   ok / aujourdhui.test : ok, 4 ligne(s) jouée(s)      (taper « fleuve » ne change rien à la liste)
$ holo check essais/p1/propose.holo
   ligne 13, colonne 3 : « Page » n'a pas de paramètre « computed » ; …
   (et, chacun seul : « Data » n'a pas de paramètre « name » ; bloc inconnu « Status » ; « Repeat(over: …) » n'a pas de paramètre « limit »)
$ echo 'Page(title: "a", … data: Data(from: "a.json"), data: Data(from: "b.json"), …)' | holo check -
   ok                      (puis, dans Node, data(source) rend « a.json|0 » : le second est ignoré)
```

## Le scénario du site de référence

**La tâche** (recette de Codex, cas X02, aujourd'hui `BLOQUÉ` : « Catalogue reçu comme liste ; filtres, pagination ou limite définie, vide, long et invalide » ; `proposals/GPT5.6/site-reference-2026-10-06/RECETTE.md:91`). Le catalogue construit (`exemples/site-reference/catalogue.holo`) écrit ses douze fiches une à une, et chaque bouton « La mettre au panier » n'est qu'un lien vers la page du panier, où trois articles seulement sont écrits à la main (`panier.holo:21-22`).

Le parcours visé, sur la page « Catalogue » :

1. Les douze créations arrivent de `catalogue.json` (essai : `essais/p1/catalogue.json`).
2. Le visiteur tape « aout » : il voit « Nuit d'août » (sans souci des accents ni des majuscules).
3. Il choisit le thème « Eau » : trois créations, du moins cher au plus cher (La barque, Le phare, Lever sur le fleuve).
4. Il tape « zzz » : « Aucune création ne correspond. »
5. Sans recherche ni thème, il voit six fiches, puis six de plus avec « Afficher six de plus ».
6. Il ajoute deux fois « La barque » : le panier compte 2, pour 150,00 €. Le panier reste après un rechargement.
7. Si `catalogue.json` manque : « Le catalogue n'a pas pu être chargé », et « Réessayer ».

**Ce qu'il faut pour le faire** : chercher (« contient », sans accents ni majuscules), comparer un champ à une valeur de la page, trier, limiter, un message quand il n'y a rien, mettre l'élément d'une ligne dans une autre liste, la somme d'une liste (piste 10), l'état du chargement, et une annonce du nombre de résultats pour le lecteur d'écran.

## Options comparées

**A. Chercher et filtrer**

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A1. Élargir `If` dans les lignes | `If(item.title, contains: search, children: [ … ])` et `If(item.theme, is: theme, …)` | `articles.filter(a => …)` avant le rendu | `articles.where((a) => …)` | Un seul mot nouveau (`contains`) ; rien d'autre à apprendre | Les lignes non retenues sont seulement cachées : pas de nombre de résultats, pas de message « aucun résultat » ; le filtre est éparpillé dans le modèle |
| A2. Paramètres de `Repeat(over:)` | `Repeat(over: articles, where: [ … ], empty: [ … ])` | idem | `ListView.builder` sur une liste filtrée | Tout au même endroit ; message vide facile | Le nombre de résultats reste inaccessible ailleurs (« 3 créations ») ; deux répétitions qui filtrent pareil écrivent deux fois le filtre |
| **A3. Une liste calculée** (recommandée) | `computed: Computed(found: Filter(articles, where: [ … ]))`, puis `Repeat(over: found)`, `{found}`, `If(found, is: 0)` | `const found = computed(() => …)` (Vue), `useMemo` (React) | un `Provider` dérivé (Riverpod), ou un getter | `{found}` donne le nombre ; `If(found, is: 0)` le message ; tout ce que le langage sait faire d'une liste vaut pour elle ; même notion que la piste 10 | Une notion nouvelle ; le moteur doit relier une ligne de `found` à son élément de `articles` |
| A4. Laisser le serveur filtrer | `Data(from: "catalogue.json", with: [search, theme])` | `fetch("/catalogue?q=…")` | `http.get(…)` | Marche pour des milliers de fiches ; le robot de recherche peut lire chaque résultat | Il faut un serveur qui répond (aujourd'hui, des fichiers seulement) ; un aller-retour par lettre tapée ; les mots cherchés quittent l'appareil |

**B. Trier, limiter, « afficher plus »**

| Option | Écriture HoloCode | HTML/CSS/JS | Avantages | Défauts |
|---|---|---|---|---|
| B1. Dans la liste calculée (tri) et dans `Repeat` (limite), recommandée | `Filter(articles, sortBy: item.price, reverse: true)` ; `Repeat(over: found, limit: more)` ; `more.add(6)` | `toSorted((a, b) => a.price - b.price).slice(0, n)` | Le tri est une donnée, la limite un affichage : chacun à sa place ; « afficher plus » tient en une demande | Deux mots (`sortBy`, `limit`) |
| B2. Pages numérotées | `Repeat(over: found, page: page, size: 12)` | `slice((p - 1) * 12, p * 12)` | Familier sur ordinateur | Plus de mots ; moins commode au doigt ; `size:` veut déjà dire une taille à l'écran (`Shape`, `Points`) |
| B3. Le serveur envoie la liste déjà triée | rien à écrire | `ORDER BY` côté serveur | Aucun mot | Le visiteur ne peut pas choisir l'ordre |

**C. Du catalogue au panier (le passage d'une ligne vers la page)**

| Option | Écriture HoloCode | HTML/CSS/JS | Avantages | Défauts |
|---|---|---|---|---|
| **C1. `item.<champ>` partout où l'on écrit une valeur de la page** (recommandée) | `cart.push(item)`, `cart.add(item.price)`, `chosen.set(item.title)` | `cart.push({ ...a })` | Aucun mot nouveau ; prolonge ADR-032 (un nom de valeur là où l'on écrit un nombre) | Il faut dire ce que copie `push(item)` : les champs de même nom |
| C2. Un signal émis par la ligne, branché par la page | `On(Add.tap, emit: add)` puis `onAdd: cart.push(…)` | un événement personnalisé | Réutilise ADR-056 | Plus long ; le signal ne transporte pas l'élément |

**K. Des identifiants stables (K comme « key », la clé)**

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| K1. Garder le focus sans nouveau mot (recommandée d'abord) | rien : quand une ligne est refaite, la page remet le focus sur le bloc du même nom dans la même ligne | `key` de React, puis `focus()` à la main | `Key` | Répare l'accessibilité tout de suite | Ne sert pas à fusionner un catalogue rafraîchi |
| K2. Une clé choisie | `Repeat(over: found, key: item.id)` | `key={a.id}` (React), `:key` (Vue) | `ValueKey(a.id)` | Une ligne reste la même quand son prix change ; deux ajouts du même article se reconnaissent | `key:` a déjà un sens voisin mais différent dans `Item(key: sunrise)` : la clé y est le nom d'une valeur de la page (ADR-040, point 2) ; et un champ de liste ne peut pas s'appeler `key` (`lists.rs:85`, `repeat.rs:31`) |

**E. L'état du chargement**

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **E1. Nommer `Data`, deux signaux, une capacité** (recommandée) | `Data(name: Catalogue, from: …)`, `On(Catalogue.failed, …)`, `On(Catalogue.done, …)`, `Catalogue.refresh` | `fetch().then().catch()` | `FutureBuilder`, `RefreshIndicator` | `done` existe déjà pour `Module`, `failed` pour `Module` et `Form` (`rules.rs:14-16`) ; `Form` dit `sent`, pas `done` | Un paramètre et une capacité de plus |
| E2. Une valeur tenue par le moteur | `{Catalogue.state}` qui vaut « loading », « ready », « failed » | idem | idem | Se montre avec un `If` | Un texte magique de plus ; les textes ne se comparent qu'au vide aujourd'hui |

**F. La page fabriquée avec ses données**

| Option | Où | Avantages | Défauts |
|---|---|---|---|
| **F1. `holo html` lit le fichier de `Data(from:)` rangé à côté** (recommandée) | moteur en ligne de commande, `server.mjs` | Le robot et le visiteur sans JavaScript lisent le catalogue ; aucun mot nouveau | La page fabriquée peut avoir un instant de retard sur le fichier |
| F2. Ne rien changer, écrire les fiches à la main pour le référencement | — | Rien à faire | C'est ce que Codex voulait éviter |

**Noms (ADR-016)** : le sens que chaque nom proposé a déjà ailleurs, et le risque de confusion.

| Nom proposé | Sur le web | Dans Flutter et Dart | Risque |
|---|---|---|---|
| `Computed`, `computed:` | Vue `computed`, Angular `computed()`, MobX : une valeur tirée d'autres, recalculée seule ; CSS « valeur calculée » | rien de direct (Riverpod : un `Provider` qui en lit d'autres) | faible |
| `Filter` | JS `filter()`, Python `filter` : garder ce qui passe un test. Mais CSS `filter:` est un effet visuel (flou, luminosité) | Dart dit `where` | moyen, à cause du CSS ; autre choix : `Where(articles, …)` |
| `where:` | SQL `WHERE`, C# `Where` : les conditions des éléments gardés | Dart `where` : même sens | faible ; à l'oral, proche de `When` (Yocthan dicte souvent) |
| `contains:` | Java, Kotlin, C#, Swift : « contient ce texte » ; JS dit `includes` ; DOM `Node.contains` parle de nœuds | Dart `String.contains` : même sens | faible |
| `sortBy:`, `reverse:` | lodash `sortBy`, Kotlin `sortedBy` ; Python `sorted(…, reverse=True)` | Dart `sort((a, b) => …)`, `reversed` | faible |
| `limit:` | SQL `LIMIT`, les API (`?limit=20`) | Dart `take(n)` | faible ; ne pas prendre `max:`, qui veut dire « la plus grande valeur » dans HoloCode |
| `empty:` | Django `{% empty %}` ; CSS `:empty` | rien | faible ; `else:` serait trompeur (en Python, `for … else` veut dire autre chose) |
| `key:` sur `Repeat` | React et Vue `key` | `Key`, `ValueKey` | moyen : `Item(key:)` existe, avec un sens voisin (le nom d'une valeur de la page) |
| `Data(name:)`, `done`, `failed`, `refresh` | `fetch` puis `then` et `catch` | `FutureBuilder`, `RefreshIndicator(onRefresh:)` | faible ; `done` et `failed` existent déjà |
| `Status` | ARIA `role="status"` : une zone lue poliment quand elle change | `Semantics(liveRegion: true)` | faible ; en HTTP, « status » est un code de réponse |

## Recommandation

Mon avis, dans cet ordre :

1. **Réparer d'abord, sans aucun mot nouveau** : D1 (deux répétitions sur une liste), D2 (une ligne, un `<li>`), le focus d'une ligne refaite (option K1), D5 (dire à l'auteur, dans `?values`, qu'un champ reçu a été vidé ou que la liste a été coupée à 100), D6 et D8 (messages), D7 (`expect` sur un champ), D9 (refuser un paramètre écrit deux fois, avec le bon message), D10 (une image refusée ne doit retirer que sa fiche, ou prendre une image de remplacement, et le dire dans `?values`). Ce sont des défauts, pas des choix.
2. **Ouvrir le passage d'une ligne vers la page (option C1)** : `cart.push(item)`, `cart.add(item.price)`, `chosen.set(item.title)`. Aucun mot nouveau, et c'est ce qui débloque « du catalogue au panier ». Cela élargit une règle décidée (ADR-044, ADR-051, ADR-057) : à Yocthan de dire oui.
3. **Chercher et filtrer par une liste calculée (option A3)**, avec `contains:` (sans accents ni majuscules, la même règle partout), la comparaison d'un champ à une valeur de la page, `sortBy:`, puis `Repeat(limit:, empty:)`. La liste calculée est la même notion que les valeurs calculées de la piste 10 : **les deux pistes se décident ensemble**.
4. **Dire l'état du chargement (option E1)** et **fabriquer la page avec ses données (option F1)**.
5. **Plus tard** : la clé choisie (option K2), puis les catalogues de plus de 100 fiches (option A4), après le serveur en Rust (`proposals/Claude/serveur-et-comptes-2026-10-07.md`).

Pourquoi pas A1 seule : elle cache des lignes, mais ne sait ni compter ni dire « rien trouvé », ce que la recette X02 demande (« vide »). Pourquoi pas A4 d'abord : le serveur de démonstration ne sert que des fichiers, et un catalogue de cent fiches devrait se filtrer sans peine dans la page (estimation, appuyée sur la mesure N2 : environ 4 ms par lettre sur le PC pour 100 fiches, sans le filtre lui-même ; téléphone non mesuré).

## Exemple d'auteur

Le parcours du scénario. Les lignes marquées `// proposé` n'existent pas. Fichier `essais/p1/propose.holo` (2 502 octets, 45 lignes utiles) ; `holo check` le refuse à son premier mot nouveau (« « Page » n'a pas de paramètre « computed » »).

```holo
// Le catalogue avec l'écriture PROPOSÉE (EXPLORATION, rien n'est décidé).
// Chaque ligne marquée « proposé » n'existe pas dans le moteur (main, 2026-10-07).
Page(
  title: "Catalogue",
  lang: "fr",
  state: State(
    search: "", theme: "", more: 6, failed: 0,
    articles: [ Item(id: "", title: "", theme: "", price: 0, image: "vide.svg", alt: "") ],
    cart: [ Item(id: "", title: "", price: 0) ],
  ),
  keep: [cart],
  data: Data(name: Catalogue, from: "catalogue.json"),                       // proposé : name
  computed: Computed(                                                       // proposé
    found: Filter(articles,                                                 // proposé
      where: [ If(item.title, contains: search), If(item.theme, is: theme) ], // proposé : where, contains, is: valeur
      sortBy: item.price),                                                  // proposé
    total: Sum(cart, of: item.price),                                       // proposé (piste 10)
  ),
  children: [
    H1("Le catalogue"),
    Input(value: search, label: "Chercher une création"),
    Row(gap: 8px, children: [
      Choice(value: theme, label: "Thème", options: ["Eau", "Terre", "Ville", "Ciel"]),
      Button(name: AllThemes, text: "Tous les thèmes"),
    ]),
    Status("{found} créations sur {articles}. Panier : {cart} ({total:cents} €)."), // proposé : Status
    If(failed, is: 1, children: [ P("Le catalogue n'a pas pu être chargé."), Button(name: Retry, text: "Réessayer") ]),
    Grid(columns: 3, gap: 16px, children: [
      Repeat(over: found, limit: more, children: [                          // proposé : limit
        Column(gap: 6px, children: [
          Image(source: item.image, alt: "{item.alt}"),
          H2("{item.title}"),
          Text("{item.price:cents} €"),
          Button(name: Add, text: "Ajouter au panier"),
        ]),
      ], rules: [ On(Add.tap, effect: cart.push(item)) ],                   // proposé : push(item)
      empty: [ P("Aucune création ne correspond.") ]),                     // proposé : empty
    ]),
    If(found, over: more, children: [ Button(name: More, text: "Afficher six de plus") ]),
  ],
  rules: [
    On(More.tap, effect: more.add(6)),
    On(AllThemes.tap, effect: theme.set("")),
    On(Catalogue.failed, effect: failed.set(1)),                            // proposé : signal
    On(Retry.tap, effect: [failed.set(0), Catalogue.refresh]),              // proposé : refresh
  ],
)
```

Trois règles à fixer avec l'écriture : dans `where:`, une comparaison à une valeur vide ne retient rien de moins (« tous les thèmes ») ; `cart.push(item)` copie les champs qui portent le même nom (`id`, `title`, `price`) ; et un `Item(…)` tout vide écrit dans `State` doit seulement déclarer les champs. Aujourd'hui, c'est un vrai élément : cette page commencerait avec une fiche vide (« 1 créations sur 1 ») et un panier de 1, puis compterait 3 après deux « La barque » (`lists.rs:212-221` ; `holo test essais/p1/propose-sans-nouveau.holo relecture-1-10/depart.test`, avec `expect articles = 1` et `expect cart = 1` : « ok, 2 ligne(s) jouée(s) »). Ce qui existe déjà dans cet exemple et passe aujourd'hui : `If(found, over: more)` sur une liste (vérifié avec une liste ordinaire), le `Choice`, `keep:` d'une liste, `more.add(6)`, `State(failed: 0)`. Sans ses mots proposés (sans filtre, sans limite, sans panier), le reste passe `holo check` (`essais/p1/propose-sans-nouveau.holo` : « ok »).

**Le même en HTML, CSS et JavaScript**, qui fait la même chose, à deux écarts près, relevés à la relecture : (1) la version web commence avec un catalogue et un panier vides, alors que la page HoloCode, écrite avec les `Item(…)` vides d'aujourd'hui, commencerait avec une fiche vide et un panier de 1, et les garderait en cas d'échec (troisième règle ci-dessus) ; (2) la version web remplace une image fautive par `vide.svg`, alors que le moteur d'aujourd'hui efface toute la liste (D10). Sinon : même fichier JSON lu avec la même prudence (100 éléments, champs déclarés, prix entier, image du dossier), même recherche sans accents ni majuscules, même tri stable, six puis six de plus, même message vide, même échec et « Réessayer », même panier gardé dans le navigateur, mêmes lignes gardées (clé tirée du contenu), même annonce par `role="status"`. Fichiers `essais/p1/web/` : 6 562 octets, 108 lignes utiles (30 + 7 + 71), contre 2 502 octets et 45 lignes utiles pour la page HoloCode (mesure, `wc -c`, lignes sans les vides ni les commentaires ; les octets comptent les commentaires des deux côtés, dont les marques « // proposé » ; sans aucun commentaire : 1 772 octets contre 6 201, mesure à la relecture avec `sed` puis `wc -c`).

```html
<!doctype html>
<html lang="fr">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Catalogue</title>
  <link rel="stylesheet" href="style.css">
  <script type="module" src="catalogue.js"></script>
</head>
<body>
  <main>
    <h1>Le catalogue</h1>
    <label class="field"><span>Chercher une création</span><input id="search" type="text" maxlength="80"></label>
    <div class="row">
      <fieldset><legend>Thème</legend>
        <label><input type="radio" name="theme" value="Eau"><span>Eau</span></label>
        <label><input type="radio" name="theme" value="Terre"><span>Terre</span></label>
        <label><input type="radio" name="theme" value="Ville"><span>Ville</span></label>
        <label><input type="radio" name="theme" value="Ciel"><span>Ciel</span></label>
      </fieldset>
      <button id="all" type="button">Tous les thèmes</button>
    </div>
    <p id="count" role="status">0 créations sur 0. Panier : 0 (0,00 €).</p>
    <div id="failed" hidden><p>Le catalogue n'a pas pu être chargé.</p><button id="retry" type="button">Réessayer</button></div>
    <div id="grid" class="grid"></div>
    <p id="empty" hidden>Aucune création ne correspond.</p>
    <button id="more" type="button" hidden>Afficher six de plus</button>
  </main>
</body>
</html>
```

```css
/* La disposition que le moteur fabrique pour Grid(columns: 3, gap: 16px), Row(gap: 8px), Column(gap: 6px). */
main { max-width: 60rem; margin: 0 auto; padding: 1rem; font-family: system-ui, sans-serif; }
.row { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; }
.grid { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fill, minmax(min(100%, max(7.5rem, calc((100% - 2 * 1rem) / 3))), 1fr)); }
.card { display: flex; flex-direction: column; gap: 0.375rem; min-width: 0; }
.card img { max-width: 100%; height: auto; }
.field { display: flex; flex-direction: column; gap: 0.25rem; }
button:focus-visible, input:focus-visible { outline: 3px solid; outline-offset: 2px; }
```

```js
// catalogue.js — ce que le moteur HoloCode ferait seul est écrit ici à la main.
export const fold = (t) => t.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
const FIELDS = { id: "", title: "", theme: "", price: 0, image: "vide.svg", alt: "" };
// Comme l'arbitre : 100 éléments au plus, les champs déclarés seulement, un prix entier, une image du dossier.
export function clean(json) {
  const list = Array.isArray(json?.articles) ? json.articles.slice(0, 100) : [];
  return list.map((a) => Object.fromEntries(Object.entries(FIELDS).map(([k, d]) => {
    const v = a?.[k];
    if (typeof d === "number") return [k, Number.isInteger(v) && v >= 0 && v <= 1e9 ? v : ""];
    if (k === "image" && !(typeof v === "string" && /^[\w.\/-]+$/.test(v) && !v.includes(".."))) return [k, d];
    return [k, typeof v === "string" ? v.slice(0, 200) : typeof v === "number" ? String(v) : ""];
  })));
}
export function compute(articles, search, theme) {
  const s = fold(search);
  return articles
    .map((a, rank) => ({ a, rank }))
    .filter(({ a }) => fold(a.title).includes(s) && (theme === "" || a.theme === theme))
    .sort((x, y) => (Number(x.a.price) || 0) - (Number(y.a.price) || 0) || x.rank - y.rank)
    .map(({ a }) => a);
}
export const sum = (list, field) => list.reduce((n, a) => Math.min(n + (Number(a[field]) || 0), 1e9), 0);
const format = new Intl.NumberFormat("fr-FR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
export const cents = (n) => (n === "" ? "" : format.format(n / 100));

if (typeof document !== "undefined") {
  const $ = (id) => document.getElementById(id);
  const state = { search: "", theme: "", more: 6, failed: false, articles: [], cart: [] };
  try { state.cart = clean({ articles: JSON.parse(localStorage.getItem("cart") ?? "[]") }).map(({ id, title, price }) => ({ id, title, price })); } catch {}
  const built = new Map(); // clé tirée du contenu → nœud de la ligne
  function card(a) {
    const box = document.createElement("div");
    box.className = "card";
    const img = Object.assign(document.createElement("img"), { src: a.image, alt: a.alt });
    const h2 = Object.assign(document.createElement("h2"), { textContent: a.title });
    const price = Object.assign(document.createElement("span"), { textContent: `${cents(a.price)} €` });
    const add = Object.assign(document.createElement("button"), { type: "button", textContent: "Ajouter au panier" });
    add.addEventListener("click", () => { state.cart.push({ id: a.id, title: a.title, price: a.price }); render(); });
    box.append(img, h2, price, add);
    return box;
  }
  function render() {
    const found = compute(state.articles, state.search, state.theme);
    $("count").textContent = `${found.length} créations sur ${state.articles.length}. Panier : ${state.cart.length} (${cents(sum(state.cart, "price"))} €).`;
    $("failed").hidden = !state.failed;
    const shown = found.slice(0, state.more);
    const grid = $("grid");
    const lines = shown.map((a, rank) => {
      const key = JSON.stringify(a) + "#" + shown.slice(0, rank).filter((b) => JSON.stringify(b) === JSON.stringify(a)).length;
      if (!built.has(key)) built.set(key, card(a));
      return built.get(key);
    });
    lines.forEach((line, rank) => { if (grid.children[rank] !== line) grid.insertBefore(line, grid.children[rank] ?? null); });
    while (grid.children.length > lines.length) grid.lastElementChild.remove();
    $("empty").hidden = found.length > 0;
    $("more").hidden = !(found.length > state.more);
    try { localStorage.setItem("cart", JSON.stringify(state.cart)); } catch {}
  }
  async function load() {
    try {
      const response = await fetch("catalogue.json", { cache: "no-store" });
      if (!response.ok) throw new Error(response.status);
      state.articles = clean(JSON.parse((await response.text()).slice(0, 65536)));
    } catch { state.failed = true; }
    render();
  }
  $("search").addEventListener("input", (e) => { state.search = e.target.value; render(); });
  for (const radio of document.querySelectorAll("input[name=theme]")) radio.addEventListener("change", (e) => { state.theme = e.target.value; render(); });
  $("all").addEventListener("click", () => { state.theme = ""; document.querySelectorAll("input[name=theme]").forEach((r) => (r.checked = false)); render(); });
  $("more").addEventListener("click", () => { state.more += 6; render(); });
  $("retry").addEventListener("click", () => { state.failed = false; load(); });
  render();
  load();
}
```

Vérifié : la logique de la version web, jouée dans Node avec le même `catalogue.json` (`node essais/p1/essai-catalogue.mjs`) donne les résultats du scénario (onze vérifications « OK », dont « aout » → « Nuit d'août », thème Eau → « La barque | Le phare | Lever sur le fleuve », deux barques → « 150,00 », 150 reçus → 100 gardés). **Non vérifié** : la page web n'a pas été ouverte dans un navigateur. **Une différence assumée, hors comparaison** : sans JavaScript, la version web ne montre aucune fiche ; la page HoloCode non plus aujourd'hui (D4), et l'option F1 le corrigerait côté HoloCode seulement.

## Par couche

- **Langage** : `computed: Computed(…)` et `Filter(liste, where:, sortBy:, reverse:)` ; `contains:` ; comparer un champ à une valeur de la page, et un texte à un texte écrit ; `Repeat(over:, limit:, empty:)` puis `key:` ; `item.<champ>` partout où l'on écrit une valeur de la page dans une règle de ligne, `push(item)` ; `Data(name:)`, `done`, `failed`, `refresh` ; `Status`.
- **Moteur (Rust)** : recalculer les listes calculées après chaque changement, comme `count` et `total` aujourd'hui (`state.rs:916-937`), sans les ranger dans l'état ; relier chaque ligne de `found` à son rang dans `articles`, pour que `Add.tap@2` agisse sur le bon élément ; une table de lettres sans accents écrite dans le moteur (la bibliothèque standard de Rust ne retire pas les accents ; une bibliothèque Unicode complète demanderait une installation) ; un tri stable (à prix égal, l'ordre d'arrivée) ; les vérifications (champ connu, sortes, pas de liste calculée qui dépend d'elle-même) ; D1, D2, D8.
- **Enveloppe navigateur** (`page-engine.js`) : redessiner chaque répétition avec son propre modèle (D1) ; remettre le focus dans une ligne refaite ; annoncer un `Status` par la zone `#announcement` qui existe déjà (`moteur/web/page.html:119`) ; émettre `Catalogue.done` et `Catalogue.failed` ; `refresh`.
- **Services serveur** : `holo html` remplit les listes avec le fichier de `Data(from:)` avant de fabriquer la page (F1) ; plus tard, le serveur en Rust répond à une recherche pour les grands catalogues (A4) et recalcule lui-même un panier envoyé, sans croire la page.

## Dépendances

- **Piste 10** : la liste calculée et `Sum` sont la même notion que les valeurs calculées. À décider ensemble, sinon on aura deux façons de calculer.
- **Pistes 2 et 6** : comparer un texte à un texte écrit sert aussi aux formulaires (un `Choice` qui change ce qu'on montre). La recherche « à chaque lettre » ne demande pas de signal nouveau : chaque lettre passe déjà par l'arbitre (`page-engine.js:1428-1433` à `7a48def`, `:1432-1437` sur `9df3b2b` ; l'événement `input`), et une liste calculée suivrait seule. Un signal de changement de champ, que la piste 6 examine, ne servirait qu'à une règle (option A3 de la piste 10).
- **Piste 7** : la page fabriquée avec ses données sert le référencement et le partage.
- **Le serveur en Rust** (proposition du 2026-10-07 ; quatre questions pour Yocthan à `1119361`, trois sur `9df3b2b`, car l'hébergement attendra : commit `5b18fb9`, pendant la relecture) : nécessaire pour plus de cent fiches et pour une vraie commande (recette X01).
- **ADR touchées** : ADR-027 (« un texte ne se compare qu'au vide ») et ADR-057 (comparaisons d'un champ) seraient élargies ; ADR-023 avait écarté `sum(price)` comme « un pas vers les formules ».

## Coût

- **Moteur** (estimation) : réparations D1, D2, focus, messages : 100 à 200 lignes de Rust et de JavaScript ; passage ligne → page : 120 à 180 lignes ; comparaisons et `contains` : 150 à 250 ; liste calculée, `limit`, `empty` : 400 à 600 ; `Data` nommé : 60 à 100 ; `Status` : 40 à 60 ; page fabriquée avec ses données : 40 à 80 ; clé choisie : 100 à 150. En tout, environ 1 000 à 1 600 lignes, essais compris en plus.
- **Poids transféré** : **mesure** du moteur léger aujourd'hui : 479 290 octets bruts, **156 511 octets en Brotli** (qualité 11, comme le serveur de démonstration), 182 719 en gzip (`gzip -9` ; 183 508 au réglage par défaut) ; colle JavaScript 4 385, `page-engine.js` 22 538, `page.html` 5 578 (Brotli). Commande : `wc -c` et `brotli -c -q 11 … | wc -c` sur `moteur/web/` (refait à la relecture : mêmes nombres). Ce moteur compte 9 790 lignes de Rust hors essais (`wc -l`, mesure : les modules du moteur léger, sans `renderer.rs` ni `web.rs`, lignes avant `#[cfg(test)]` ; refait à la relecture), soit environ 16 octets compressés par ligne en moyenne (une moyenne, pas le coût réel d'une ligne ajoutée). **Estimation** : + 16 à 26 Ko compressés pour toute la piste, soit un moteur léger d'environ 173 à 183 Ko. Une page qui lit `Data` charge ce moteur dès l'ouverture (ADR-033) ; la version web de l'exemple, elle, pèse 6 562 octets bruts en tout (mesure, `wc -c`) et n'a besoin de rien d'autre. À fonctions égales, et si l'écriture proposée existait, HoloCode ferait écrire 2,6 fois moins d'octets à l'auteur (2,4 fois moins de lignes), mais ferait télécharger environ 189 Ko compressés de moteur à la première visite (somme des quatre mesures ci-dessus ; que le cache du navigateur l'évite aux visites suivantes : non vérifié). Ces 189 Ko sont une taille de fichiers, pas un transfert réseau mesuré.
- **Temps de calcul** : **mesure** dans Node sur le PC (moteur léger WebAssembly), trois appels mesurés séparément, médianes : 12 fiches : saisie 0,31 + conditions 0,23 + lignes 0,53 ms ; 100 fiches : 0,97 + 0,65 + 2,67 ms. Leur somme (environ 1,1 ms et 4,3 ms) est une **estimation** du coût d'une lettre qui redessinerait la liste : aujourd'hui, une lettre ne la redessine pas, car la liste ne change pas (`page-engine.js:361-362`). Sans le filtre, sans la page (DOM). Mesure refaite à la relecture (`node essais/sonde.mjs`) : 0,971 + 0,649 + 2,662 ms pour 100 fiches. **Estimation** : le filtre ajoute moins de 0,5 ms pour 100 fiches. Sur téléphone : **non mesuré, aucun appareil ici**.
- **Travail** (estimation) : 8 à 10 séances, dont 2 à 3 pour la liste calculée seule ; les réparations du point 1 tiennent en une séance.

## Accessibilité, déterminisme, budgets

- **Libellés** : la recherche garde son `label` obligatoire ; le thème reste un `fieldset` avec `legend` (`flat.rs:806`).
- **Clavier et focus** : « Ajouter au panier » ne doit jamais perdre le focus. Aujourd'hui, une ligne inchangée garde son nœud, mais elle est déplacée (et perd le focus) si une ligne placée avant elle a été refaite ou retirée ; une ligne changée perd son nœud (point 5, déduit du code). La réparation de l'option K1 est une condition, pas un bonus : remettre le focus dans une ligne refaite, et ne plus déplacer les lignes inchangées (retirer d'abord les anciennes).
- **Annonces** : le nombre de résultats doit être lu une fois après la frappe, pas à chaque lettre (WCAG 4.1.3). `Status` le fait par une zone polie ; à régler : attendre une demi-seconde sans frappe avant d'annoncer (estimation, à essayer au TalkBack).
- **Structure** : une liste de résultats doit être une vraie liste (D2), pour que le lecteur dise « 12 éléments ».
- **Moins de mouvement** : une ligne qui apparaît (`Enter(…, inView:)`) suit déjà le réglage du visiteur ; rien de plus.
- **Déterminisme** : même fichier, mêmes données, mêmes gestes, même résultat. Le tri doit être stable ; la règle « sans accents ni majuscules » doit être écrite dans le moteur, la même dans le navigateur, sur le serveur et dans `holo test` (pas `toLocaleLowerCase`, qui dépend de la langue de l'appareil).
- **Budgets** (propositions, à mesurer) : 100 fiches au plus par liste (inchangé) ; un fichier de données de 64 Ko au plus (inchangé) ; moins de 50 ms entre une lettre tapée et la grille à jour sur un téléphone modeste, au 95e centile ; un état gardé de moins de 10 Ko (aujourd'hui, l'état écrit pèse 8 506 octets pour 100 fiches de quatre champs, `title`, `price`, `image`, `cat` : mesure N2, `essais/sonde.mjs`).

## Recette qui peut échouer

Avec `essais/p1/catalogue.json`. Chaque essai dit ce qui doit se passer et ce qui le fait échouer.

| N° | Essai | Attendu | Échec si |
|---|---|---|---|
| R1 | Taper « FLEUVE », puis « aout » | « Lever sur le fleuve » ; puis « Nuit d'août » | 0 résultat, ou une différence entre le navigateur, `holo test` et la version web |
| R2 | Choisir « Eau » | La barque, Le phare, Lever sur le fleuve, dans cet ordre | un autre ordre ; un ordre qui change d'un essai à l'autre |
| R3 | Taper « zzz » | « Aucune création ne correspond. » visible, annoncé une fois | message absent, ou annoncé à chaque lettre |
| R4 | Sans filtre | 6 fiches et « Afficher six de plus » ; après un toucher, 12 fiches et plus de bouton | 12 fiches d'emblée, ou le bouton reste |
| R5 | Ajouter deux fois « La barque », recharger | Panier : 2 (150,00 €), gardé | refus à la lecture du fichier (aujourd'hui : T8) ; un total faux |
| R6 | Au clavier : Entrée sur « Ajouter au panier » de la 2e fiche (sa ligne ne change pas) ; puis Entrée sur « Un de plus » d'une ligne du panier, dont la quantité change (exemple de la piste 10) | Le focus reste sur le bouton touché, les deux fois | le focus part en haut de la page (attendu aujourd'hui dans le second cas, déduit du code) |
| R7 | Deux `Repeat(over: articles)` aux modèles différents, puis un ajout | Chacun garde son modèle | **échoue aujourd'hui** (N1) |
| R8 | `Repeat(over:)` dans une `List` | Un `<li>` par ligne ; TalkBack dit « liste, 12 éléments » | **échoue aujourd'hui** (E5) |
| R9 | Renommer `catalogue.json`, puis le remettre et toucher « Réessayer » | Le message d'échec, puis le catalogue | **échoue aujourd'hui** : rien ne s'affiche (D3) |
| R10 | `holo html catalogue.holo` | Le HTML contient les douze titres | **échoue aujourd'hui** pour un catalogue reçu par `Data` (D4 ; `essais/p1/aujourdhui.holo` donne « Chargement… ») ; le `catalogue.holo` actuel, écrit à la main, le réussit |
| R11 | Envoyer 150 fiches, puis un fichier de 70 Ko | 100 fiches et un avertissement dans `?values` ; puis l'ancienne liste et un avertissement | silence (aujourd'hui) |
| R12 | Rejouer les mêmes gestes dans `holo test` et dans Node | Le même état, octet pour octet | une différence |
| R13 | 100 fiches, taper une lettre, sur le PC puis sur le téléphone de Yocthan et un Android modeste | moins de 50 ms au 95e centile (proposition) | au-delà ; ou mesure sur PC présentée comme une mesure de téléphone |
| R14 | Écrire deux fois `data:` sur la page | refusé, avec la ligne du second | « ok » (**échoue aujourd'hui**, D9) |
| R15 | Recevoir les douze fiches, dont une avec `"image": "../x.svg"` | onze fiches (ou douze, avec une image de remplacement) et un avertissement dans `?values` | aucune fiche affichée (**échoue aujourd'hui**, D10) |

## Objection

**La meilleure raison de ne pas le faire : HoloCode deviendrait un petit langage de requêtes.** Il faudrait au moins sept mots (`Computed`, `Filter`, `where`, `contains`, `sortBy`, `limit`, `empty`), plus des règles cachées (« une valeur vide ne filtre pas »). Yocthan veut un langage peu verbeux. Et un vrai site de vente filtre sur son serveur dès qu'il a plus de quelques centaines de produits : la limite de cent fiches rendra la liste calculée inutile pour un grand catalogue. Le faire autrement : se contenter des réparations, du passage ligne → page, de `contains:` et de `empty:`, et laisser le serveur envoyer la liste déjà triée et déjà coupée.

Ma réponse : pour un artisan, un musée, une association (le public de HoloCode), cent fiches suffisent ; et le nombre de résultats (« 3 créations ») est demandé par la recette X02 et par l'accessibilité. Mais c'est à Yocthan de peser.

## Expérience requise

1. **Montrer deux écritures à Yocthan** sur le même catalogue : la liste calculée (A3) et les paramètres de `Repeat` (A2). Lui demander laquelle il lit sans aide.
2. **Un prototype isolé, sur une branche, non fusionné** : la liste calculée et `contains` sur la page de l'exemple. Mesurer le temps entre une lettre et la grille à jour : sur le PC (Chrome), puis sur le Galaxy Z Flip 5 de Yocthan et sur un Android modeste. Sans téléphone, le résultat reste « non mesuré ».
3. **L'essai au TalkBack** (protocole `docs/01-holocode/ESSAI-LECTEUR-D-ECRAN.md`) : le nombre de résultats annoncé une fois ; le focus gardé après « Ajouter ».
4. **Cinq débutants** (déjà prévus par la recette U02) : « ajoute un thème "Nuit" au catalogue et vérifie qu'il filtre », avec le guide seulement.
5. **Fabriquer la page avec ses données** sur le site de référence, puis la lire sans JavaScript.

## Mises à jour de documents à prévoir

À lister, pas à faire :

- `docs/01-holocode/GUIDE.md` : § 6 septies (`Data` nommé, ses signaux, ses limites : 100 éléments, 64 Ko, coupure annoncée) ; § 6 quaterdecies et § 6 septendecies (liste calculée, `contains`, `sortBy`, `limit`, `empty`, `push(item)`, `item.<champ>` dans une règle de ligne, `key`) ; § 10 aide-mémoire (`Repeat`, `Data`, `computed`, `Status`) ; § 11 à refaire (ce qui manque vraiment) ; corriger `:590`, `:816` et `:1801-1806`.
- **Leçons** (une notion chacune) : « chercher dans une liste » ; « trier et afficher plus » ; « un catalogue qui se charge, ou non » ; « mettre une fiche au panier » ; mettre à jour la leçon 71 (son élément d'attente) et son essai.
- `docs/01-holocode/NOMS.md` : les nouveaux mots ; corriger `:5`, `:201`, `:210` et la table `:220-237`.
- `docs/01-holocode/COMPARAISON-WEB.md` : lignes `:116`, `:159-160`, la partie 5 entière ; ajouter filtrer, trier, paginer.
- `docs/01-holocode/TABLEAU-WEB.md` : ajouter « filtrer, trier, paginer » (`filter`, `sort`, `slice`) ; revoir les 90 % de `:621` ; republier la page en ligne tenue pour Yocthan.
- `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md:419` (les signaux de `Data`).
- `exemples/site-reference/catalogue.holo` (récrire avec la liste reçue) et `README.md:16` ; repasser X02 dans la recette.
- `docs/06-journal/JOURNAL.md` : les défauts D1 à D10, avec les commandes.
- Une fiche de décision proposée (« valeurs et listes calculées »), commune avec la piste 10, après l'accord de Yocthan.

Relu le 2026-10-07 : 30 corrections.
