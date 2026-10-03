# Revue critique du langage, du moteur et des passages distants — GPT5.6

## Contribution

- Sujet : revue adversariale des ajouts HoloCode du 2026-10-03.
- Source examinée : `main` au commit `09cdb57acfaf2ebaf13aaf9fccd1f124eb575785`.
- Discussions sources : journal du 2026-10-03, `HC-013`.
- Décisions concernées : `ADR-005`, `ADR-008`, `ADR-009`, `ADR-015`, `ADR-016`, `ADR-017`, `ADR-020`, `ADR-021`, `ADR-022`.
- Statut proposé : `PROPOSITION`.
- Portée des modifications : ce dossier et une sonde d'exécution dans `proposals/GPT5.6/holocode-v0.1/tests/`. Aucun statut de décision ni fichier du moteur n'est modifié.

## Méthode et résultats exécutés

Les mots « observé » et « déduit » ne sont pas interchangeables dans cette revue.

| Action | Résultat |
|---|---|
| Lecture dans l'ordre demandé | `AGENTS.md`, `GUIDE.md`, `COMPARAISON-WEB.md`, `ADR-020` à `ADR-022`, la boutique, la maison, puis les trente premières entrées du journal ont été lus. `ADR-016` et les documents obligatoires d'`AGENTS.md` ont ensuite été relus. |
| `cargo test` dans `moteur/` | **Exécuté localement, échec d'environnement** : `/bin/bash: line 1: cargo: command not found`, code 127. Rien n'a été installé. Le job GitHub « Moteur Rust » de la pull request est donc la preuve exécutable attendue. |
| Tests Python de GPT5.6 | **Exécutés** : 9 tests découverts, 8 réussis, 1 ignoré parce que `cargo` est absent ; code 0. |
| Fichiers hostiles | Sept fichiers sont dans [`hostiles/`](hostiles/). Le harnais Rust [`harness.rs`](harness.rs) décrit leurs résultats vérifiables. **Non exécuté localement**, faute de `cargo` ; la sonde Python le lance dans GitHub Actions lorsque `cargo` est disponible. |
| Audit d'injection | Inspection de tous les chemins de génération HTML/CSS dans `plat.rs`, des règles, des URL et des trois usages de `innerHTML` dans `page.html`. Aucun chemin d'injection par un fichier `.holo` vérifié n'a été démontré. |

## Verdict court

Le langage progresse, mais sa documentation promet actuellement plus que ses garde-fous. Les trois défauts les plus importants sont :

1. `Zoom(max:)` ne borne pas le zoom de la page vivante ;
2. `Portals(count:)` ne borne pas le nombre de portails écrits par l'auteur ;
3. les passages distants n'ont ni limite d'octets, ni délai maximal, ni consentement fiable par origine.

Le moteur résiste correctement aux injections HTML évidentes. Le risque principal n'est pas « du texte qui devient du code », mais **du contenu valide qui force trop de réseau, de DOM ou de mémoire**, ou qui se présente sous l'identité du site de départ.

---

# A. Le langage

## A1. Revue des noms proposés dans `ADR-021`

Règle appliquée : `ADR-016`, lignes 16 à 18 — un mot déjà connu des programmeurs garde son sens. Une proposition cohérente serait :

```holo
Page(
  zoom: Zoom(active: true, maxScale: 1000000, minSize: 1px),
  pointView: PointView(
    threshold: 4,
    pointSize: 6px,
    subdivideAt: 40px,
    divisions: 4,
    levels: 20,
    density: 2,
  ),
  depth: Depth(elevation: 10px, maxTilt: 52deg),
  portalView: PortalView(layout: grid, maxVisible: 12, size: 170px),
  points: [
    Point(name: Secret, anchor: Open, placement: topRight, inside: World(...)),
  ],
)
```

Cette syntaxe est une proposition de vocabulaire, pas une modification de décision.

| Nom actuel | Avis | Pourquoi | Remplacement proposé |
|---|---|---|---|
| `pixels:` | Mauvais | Il contient des blocs `Point`, pas des valeurs de pixels. Il brouille aussi `px`, qui désigne correctement une unité d'écran. | `points:` après avoir renommé le réglage actuel `points:` en `pointView:`. |
| `above:` | Mauvais | Le moteur ne place pas le point « au-dessus » au sens général : il l'ancre au bloc puis le met au bord droit (`page.html`, lignes 281 à 287). Le mot cache deux notions. | `anchor:` pour la cible et `placement:` pour `topRight`, puis plus tard `offset:` ou des coordonnées. |
| `Zoom` | Acceptable, à amaigrir | Le mot garde son sens pour la politique de zoom. En revanche, l'emboîtement de sites n'est pas du zoom. | Garder `Zoom`, sortir `levels`, renommer `max` en `maxScale`. |
| `Points` | Fragile | Un pluriel de données est employé comme bloc de politique de rendu. `Point` désigne déjà une entité du langage. | `PointView` ou `PointRendering`; préférence : `PointView`. |
| `Relief` | Mauvais | C'est un mot français emprunté, peu usuel en API anglaise, et le bloc mélange la profondeur de surface avec la limite de rotation de caméra. | `Depth(elevation:)` et `maxTilt:` dans la vue ; à défaut, `Depth(elevation:, maxTilt:)`. |
| `Portals` | Fragile | Le pluriel désigne normalement une collection, pas les réglages d'une vue. La capacité `Shop.portals` est un nom employé comme un verbe. | `PortalView(...)`; capacité `Shop.openPortals`. |
| `shrink` | Mauvais | En CSS/Flexbox, `flex-shrink` est un facteur de compression. Ici c'est un booléen qui autorise la page à devenir un point. | `minSize: 1px`; sans réduction, valeur par défaut `minSize: page`. |
| `after` | Mauvais | « Après 4 » ne dit ni après quoi ni quelle unité logique est mesurée. | `threshold:` ou `startAt:`; préférence : `threshold:`. |
| `levels` | Mauvais à cet endroit | Il limite l'emboîtement des sites, pas les niveaux de zoom. | `maxNesting:` dans une politique de navigation/site, pas dans `Zoom`. |
| `fragment` | Mauvais | En rendu graphique, un *fragment* est déjà une étape/pixel potentiel du pipeline. Ici la valeur est un seuil qui déclenche une subdivision. | `subdivideAt:`. |
| `grid` | Mauvais | Pour un programmeur web, `grid` signifie un système de mise en page. Ici c'est le nombre de divisions par axe. | `divisions:`. |
| `depth` | Mauvais | Le mot signifie naturellement l'axe Z ou une profondeur spatiale, alors qu'il compte des itérations de subdivision. | `levels:` ou `maxSubdivisions:`; préférence : `levels:`. |
| `tilt` | Presque bon | La valeur n'est pas l'inclinaison demandée, mais sa limite maximale. | `maxTilt:`. |

Deux autres noms sont trompeurs : `Portals(count:)` se lit comme un compte exact ou maximal alors qu'il ne limite pas les portails écrits, et `Point(fragments:)` exprime un nombre d'enfants alors que `Points(fragment:)` exprime une taille. Même racine, deux types incompatibles.

## A2. Deux écritures pour la même chose

1. **Paragraphe :** une phrase nue et `P("…")` produisent le même rôle et le même HTML (`GUIDE.md`, lignes 44 à 47 et 54 à 58 ; `plat.rs`, lignes 107 à 126). La différence actuelle est seulement que `P.card` peut recevoir un style nommé. C'est du sucre syntaxique acceptable, mais la documentation ne doit pas le présenter comme « une seule écriture » : `P` doit être la forme canonique, la phrase nue une abréviation explicite.
2. **Entrée dans un point :** toucher un `Point` déclenche implicitement `enter` si aucune règle ne correspond (`regles.rs`, lignes 179 à 198), mais l'auteur peut écrire `On(Point.tap, effect: Point.enter)`. Même comportement, deux écritures. Choisir : soit l'implicite est une règle par défaut formalisée et surchargeable, soit toute action doit apparaître dans `rules` comme Yocthan le demande.
3. **`A` et `Point(inside:)` ne sont pas des doublons.** Ils ont des contrats de navigation différents ; il faut conserver les deux, mais leur sécurité et leur identité d'origine doivent être visibles.

## A3. Notions qui n'ont pas encore de mot correct

- **Un site distinct d'un monde.** `World` est rendu comme une page lorsqu'il est ouvert (`plat.rs`, lignes 31 à 58). Le guide appelle tour à tour ce contenu « monde » et « site ». Il faut soit autoriser `inside: Page(...)`, soit introduire `Site`, soit définir formellement qu'un `World` peut avoir une représentation de page.
- **La position d'un point planté.** `above` ne couvre qu'un placement codé en dur. Il manque ancrage, placement et décalage.
- **La confiance envers un autre serveur.** Aucun mot ne décrit consentement, origine approuvée, politique de chargement, taille maximale, délai ou repli.
- **Le chargement d'un passage.** Il manque les états `loading`, `failed`, `retry` et `cancel` accessibles aux règles et à l'interface.
- **L'accessibilité.** `Image` n'a pas de texte alternatif ; il manque aussi langue, régions de navigation et ordre de focus.
- **Un budget réel.** `budget` et `weight` représentent des déclarations, pas les octets effectivement chargés.

## A4. L'ordre de la partie 6 de `COMPARAISON-WEB.md`

Non, l'ordre proposé n'est pas encore le bon. Mettre la disposition en premier est compréhensible visuellement, mais faire un « vrai site » signifie d'abord ne pas exclure les utilisateurs et ne pas charger un serveur distant sans contrat.

Ordre recommandé :

1. sécurité des passages et limites de ressources : octets, délai, origine, consentement, profondeur de lecture ;
2. accessibilité et sémantique : `alt`, langue, navigation, focus, clavier, citations/code ;
3. disposition : `Row`, `Column`, `Grid`, avec comportement adaptatif décidé avant l'API ;
4. réutilisation/imports : composants et styles partagés avant de multiplier les pages ;
5. état et formulaires : valeurs typées, validation, soumission, confidentialité ;
6. signaux d'interaction : focus, clavier, survol, approche, zoom ;
7. tableaux et données structurées ;
8. audio et vidéo, seulement avec budgets et alternatives accessibles ;
9. transitions et durées, avec réduction de mouvement ;
10. calcul pur et modules enfermés.

Le `Br` proposé au rang 2 est un mauvais réflexe HTML : pour du texte en Markdown, un saut de ligne ou un bloc sémantique est préférable à un nouveau bloc `Br`.

---

# B. Le moteur

## B1. Résultat exact de `cargo test`

Commande exécutée dans `moteur/` :

```text
$ cargo test
/bin/bash: line 1: cargo: command not found
exit code: 127
```

Ce n'est pas un échec des tests du moteur ; aucun test Rust local n'a commencé. Je n'ai rien installé. Le résultat du job GitHub de la pull request devra être cité séparément comme résultat CI, et non maquillé en résultat local.

## B2. Défauts constatés ou fortement étayés

### B-01 — `Zoom(max:)` est contourné par le zoom ordinaire — élevée

- **Fichiers et lignes :** `moteur/src/vue.rs:139-161`; `moteur/src/web.rs:90-104`; `moteur/web/page.html:181-188` et `473-496`.
- **Ce que j'ai fait :** lecture de la valeur dans `vue.rs`, puis suivi de son transfert vers JavaScript. `reglages_de_vue` ne renvoie pas `zoom_max`; `page.html` ne le reçoit donc jamais. Le fichier hostile [`03-zoom-max-contourne.holo`](hostiles/03-zoom-max-contourne.holo) combine `Zoom(max: 1)` et `Points(after: 16)`. Le vérificateur accepte séparément les deux valeurs.
- **Observé ou déduit :** acceptation vérifiable par le harnais ; dépassement visuel déduit du chemin JavaScript. Le zoom vivant peut monter vers ×16 avant d'entrer dans la mosaïque, malgré `max: 1`.
- **Proposition :** transmettre `zoom_max` au navigateur, borner `zoomVif`, et refuser ou normaliser `Points(threshold:) > Zoom(maxScale:)`.

### B-02 — `Portals(count:)` ne borne pas le nombre de portails — élevée

- **Fichiers et lignes :** `moteur/src/vue.rs:173-185`; `moteur/web/page.html:550-580`.
- **Ce que j'ai fait :** suivi du nombre validé jusqu'à `ouvrirCarrefour`. Tous les sites écrits sont ajoutés aux lignes 563 à 565 ; `nombreDePortails` ne sert qu'à calculer les mondes générés aux lignes 569 à 572. [`04-portals-count-contourne.holo`](hostiles/04-portals-count-contourne.holo) demande `count: 1` mais contient trois points.
- **Observé ou déduit :** le harnais vérifie que le moteur accepte la page et produit trois points pour une limite de 1 ; l'explosion DOM/réseau avec des milliers de points est déduite.
- **Proposition :** renommer `count` en `maxVisible`, limiter aussi les sites écrits, afficher une pagination/carte progressive, et fixer une limite dure de blocs/portails par fichier.

### B-03 — Le budget fait confiance à l'auteur — élevée

- **Fichiers et lignes :** `moteur/src/regles.rs:145-177`; `moteur/src/plat.rs:135-146`; `moteur/web/page.html:329-353`.
- **Ce que j'ai fait :** suivi de `budget` et `weight`. Le moteur additionne les valeurs écrites, sans lire le poids réel de l'image ou du fichier. [`05-budget-declaratif.holo`](hostiles/05-budget-declaratif.holo) déclare une image potentiellement énorme à `1B` dans un budget de `1B`; il est accepté.
- **Observé ou déduit :** acceptation vérifiable ; taille réelle non fournie dans la sonde, donc l'épuisement mémoire est déduit.
- **Proposition :** traiter `weight` comme une assertion contrôlée par le chargeur, mesurer les octets reçus, interrompre au-delà du budget et inclure source, images, fichiers distants et mémoire décodée dans des budgets séparés.

### B-04 — La mémoire est bornée en nombre de fichiers, pas en octets — élevée

- **Fichier et lignes :** `moteur/web/page.html:108-112` et `152-178`, puis `638-646`.
- **Ce que j'ai fait :** vérification du cache et des deux chemins de lecture. `response.text()` lit tout le corps ; aucune limite de `Content-Length`, aucun lecteur en flux, aucun `AbortController`, aucun délai. Le cache conserve jusqu'à 32 chaînes, quelle que soit leur taille.
- **Observé ou déduit :** constat statique certain ; attaque réseau non exécutée pour ne pas contacter un serveur tiers.
- **Proposition :** limite par fichier et limite totale en octets, lecture en flux avec arrêt, délai maximal, type MIME attendu, bouton d'annulation, et éviction LRU pondérée par octets.

### B-05 — `density: 3` crée un pic mémoire incompatible avec la cible téléphone — élevée

- **Fichiers et lignes :** `moteur/src/vue.rs:154-165`; `moteur/web/page.html:340-353` et `429-445`; `moteur/src/web.rs:16-18`.
- **Ce que j'ai fait :** calcul à partir du code. Sur un écran 1080 × 2640, `density: 3` produit 3240 × 7920 × 4 = **102 643 200 octets** pour une seule copie RGBA. Il faut aussi le canevas, l'image décodée et la copie dans la mémoire WebAssembly. La constante Rust `DENSITE_MAX = 2` ne borne que le canevas du rendu, pas l'`OffscreenCanvas` créé par `page.html`.
- **Observé ou déduit :** calcul exact des octets ; pic total supérieur à 200 Mo déduit des copies et surfaces, non mesuré sur téléphone.
- **Proposition :** plafond global de pixels (pas seulement une densité), adaptation à la mémoire de l'appareil, tuilage de la capture et suppression immédiate des surfaces temporaires.

### B-06 — `above` accepte un repère que l'affichage ne peut pas trouver — moyenne

- **Fichiers et lignes :** `moteur/src/regles.rs:82-105`; `moteur/web/page.html:279-287`.
- **Ce que j'ai fait :** comparaison des portées. Le vérificateur cherche le nom dans tous les blocs, y compris un `World` imbriqué. L'affichage cherche seulement dans le `<main>` du site courant. [`06-above-hors-portee.holo`](hostiles/06-above-hors-portee.holo) vise un bouton `Inner` situé dans un monde imbriqué : le fichier est accepté, puis `placerLesPixels` ne trouve pas le repère et abandonne silencieusement.
- **Observé ou déduit :** acceptation vérifiable ; échec de placement déduit directement du sélecteur limité à `page`.
- **Proposition :** résolution lexicale par site (`Page` ou `World`) dans le vérificateur ; un repère doit appartenir au même site. Remplacer aussi `above` par `anchor` + `placement`.

### B-07 — Un passage distant peut être déclenché par l'URL sans geste — élevée

- **Fichier et lignes :** `moteur/web/page.html:638-646` et `648-655`.
- **Ce que j'ai fait :** suivi de `#@https://…`. Au chargement, le fragment est décodé puis transmis à `ouvrirFichier`, qui lance `fetch` sans ouverture préalable du carrefour. Cela contredit la protection décrite dans `ADR-022` et le guide.
- **Observé ou déduit :** constat statique ; aucune requête de pistage n'a été envoyée.
- **Proposition :** une adresse distante peut préparer un passage, mais le réseau ne part qu'après un geste et un écran de consentement indiquant l'origine. Employer `credentials: "omit"`, `referrerPolicy: "no-referrer"` et HTTPS par défaut.

### B-08 — L'origine distante disparaît après le passage — élevée

- **Fichier et lignes :** `moteur/web/page.html:193-205`, `235-242`, `514-539`.
- **Ce que j'ai fait :** suivi du nom d'origine. L'hôte distant est ajouté au libellé du portail avant le passage, mais après ouverture le document garde l'origine et l'interface du site de départ ; le titre et tout le contenu visible viennent du serveur distant. La barre affiche seulement une adresse encodée dans `#@`.
- **Observé ou déduit :** constat statique ; contenu trompeur non servi réellement.
- **Proposition :** bandeau d'origine persistant et non stylable par l'auteur, changement d'origine clairement annoncé, isolation des aperçus et du site distant, et interdiction de masquer ce bandeau.

### B-09 — Pas de limites lexicales avant le vérificateur — moyenne

- **Fichier et lignes :** `moteur/src/holo.rs:178-224`, `451-541`, `556-561`.
- **Ce que j'ai fait :** lecture du lecteur et de l'analyseur : le fichier entier devient un `Vec<Jeton>`, puis blocs et listes sont analysés récursivement. Les limites `Zoom(levels:)` arrivent seulement après cette étape.
- **Observé ou déduit :** structure constatée ; débordement de pile ou épuisement mémoire avec une source géante/profondément imbriquée déduit, non provoqué localement.
- **Proposition :** limites avant allocation : octets, jetons, longueur de texte, nombre de blocs, longueur de liste et profondeur syntaxique, avec diagnostics déterministes.

### B-10 — Rétention mémoire après la vue points — faible à moyenne

- **Fichier et lignes :** `moteur/web/page.html:329-353` et `457-468`.
- **Ce que j'ai fait :** suivi de la durée de vie de `image.src`. Sortir de la vue retire la mosaïque WASM, mais ne vide pas la grande image `data:` ni `pixelsPlantes`. Elles restent attachées à la page jusqu'à la prochaine capture ou au rechargement.
- **Observé ou déduit :** rétention certaine par les références DOM ; quantité mémoire réellement conservée par Chrome non mesurée.
- **Proposition :** libérer `image.src`, dimensions et tableaux à la sortie, ou conserver explicitement un cache borné mesuré si la réentrée rapide est voulue.

### B-11 — `verifier_page()` accepte un lien `javascript:` — moyenne

- **Fichiers et lignes :** `moteur/src/lib.rs:44-50` ; `moteur/src/plat.rs:163-175` et `269-290`.
- **Ce que j'ai fait :** le harnais a soumis `A("Run code", to: "javascript:alert(1)")` à `verifier_page()` dans la CI de la PR. Le test qui attendait un refus a échoué : la fonction a renvoyé un `Programme`. La lecture du chemin montre que `adresse_sure()` n'est appelée que pendant `vue_a_plat()`.
- **Observé ou déduit :** **observé en CI** : la couche annoncée comme vérification de la page accepte l'URL ; **observé dans le code et désormais testé** : le rendu à plat la refuse avant de produire le lien HTML. Aucune exécution JavaScript n'a été démontrée, mais un appelant qui traite `verifier_page()` comme une validation complète reçoit un faux positif.
- **Proposition :** déplacer la validation des URL dans `verifier_page()` ou dans un validateur commun appelé par toutes les sorties ; conserver la vérification de rendu en défense redondante.

## B3. Injection : ce qui tient et ce qui manque

### Ce qui tient dans le code relu

- textes et attributs passent par `echapper` (`plat.rs:269-302`) ;
- le Markdown est appliqué après échappement (`plat.rs:304-319`) ;
- au moment du rendu, les URL refusent guillemets, chevrons, barre oblique inverse et schémas autres que HTTP(S) (`plat.rs:269-290`) ; `verifier_page()` ne réalise pas encore ce contrôle ;
- les valeurs de style sont validées par forme et liste blanche (`styles.rs:107-164`) ;
- les usages d'`innerHTML` reçoivent la sortie générée par le moteur, pas le texte brut de l'auteur (`page.html:235` et `538-539`).

Le fichier [`01-injection-texte.holo`](hostiles/01-injection-texte.holo) est accepté mais rendu sous forme de texte échappé. Pour [`02-lien-javascript.holo`](hostiles/02-lien-javascript.holo), la CI a établi un comportement en deux temps : `verifier_page()` l'accepte, puis `vue_a_plat()` refuse l'URL. Le harnais code maintenant ces deux résultats séparément. Ils ont été **exécutés dans GitHub Actions** ; localement, ils restent non exécutés faute de `cargo`.

### Défense en profondeur absente

Le serveur de démonstration ne pose pas de `Content-Security-Policy` (`moteur/outils/serveur.mjs:69-75`). Ce n'est pas une injection démontrée, mais une erreur future dans le générateur aurait alors tout le même pouvoir que la page. Ajouter une CSP stricte, sans `unsafe-inline` à terme, réduirait le rayon d'explosion.

---

# C. Passer vers un autre serveur

## Contenu trompeur

Le rendu empêche ici qu'une URL `javascript:` devienne un lien actif, mais `verifier_page()` ne l'empêche pas encore et aucune de ces deux couches n'empêche le mensonge. Un fichier distant peut reprendre le titre, les couleurs et les mots du site d'origine, afficher un faux avertissement ou conduire vers un lien web. L'aperçu affiche l'hôte, mais celui-ci n'est plus visible après le passage. Il faut une identité d'origine persistante appartenant au moteur, non au fichier.

## Suivi du visiteur

Une requête distante révèle au minimum l'adresse IP, l'heure et les caractéristiques réseau usuelles. Le chemin `#@` déclenche actuellement cette requête au chargement. Ouvrir le carrefour déclenche aussi la lecture de tous les fichiers distants visibles, pas seulement celui finalement traversé (`page.html:550-556`). Une demande de consentement par origine et une politique « un clic, une origine, un fichier » sont nécessaires.

## Boucles entre fichiers

Une boucle `A.holo → B.holo → A.holo` est légitime pour une maison. Le cache de deux fichiers reste petit. En revanche :

- une chaîne de plus de 32 fichiers provoque évictions puis relectures ;
- chaque passage ajoute une entrée d'historique ;
- un serveur lent peut bloquer l'ouverture du carrefour, car les lectures sont attendues séquentiellement et sans délai ;
- une boucle changeante (`?nonce=…` ou plusieurs URL équivalentes) peut contourner l'utilité du cache.

Ne pas interdire les cycles. Afficher plutôt le chemin, canoniser les URL, borner les octets et la durée, permettre d'annuler, et détecter les relectures répétées pour avertir.

## Taille du fichier lu

Aujourd'hui, un seul fichier suffit à annuler la promesse de mémoire bornée : `response.text()` matérialise tout avant vérification. La limite de 32 entrées ne protège donc pas la mémoire. Proposition minimale : 256 Ko de source `.holo` par fichier, 2 Mo de sources dans le cache, limites séparées pour les médias, puis mesures avant d'augmenter ces valeurs.

## HTTP et réseau local

[`07-passage-distant.holo`](hostiles/07-passage-distant.holo) montre qu'une adresse `http://127.0.0.1:…` est acceptée par le générateur. Sur une page HTTPS, le navigateur la bloquera souvent comme contenu mixte ; sur HTTP, elle peut provoquer une requête vers le réseau local du visiteur. CORS protège la lecture de la réponse, pas l'envoi de la requête. Autoriser HTTPS par défaut et soumettre HTTP, `localhost`, les adresses privées et link-local à une politique distincte.

---

## Objections et limites de cette revue

- Aucun navigateur réel ni téléphone n'a été piloté dans cet environnement.
- Rust n'est pas installé localement ; les conclusions d'exécution Rust attendent GitHub Actions.
- Les estimations mémoire distinguent les octets calculables des copies internes du navigateur, qui doivent être mesurées.
- Je n'ai pas essayé de contourner CORS ni contacté un serveur de pistage.
- Les nouveaux noms sont des propositions ; ils ne changent aucun statut d'ADR.

## Expérience ou preuve requise

1. Exécuter le harnais de ce dossier et le `cargo test` du moteur dans GitHub Actions.
2. Ajouter une sonde navigateur pour `Zoom(max: 1)` + `Points(after: 16)`.
3. Mesurer sur le Samsung Z Flip 5 l'entrée en vue points avec `density: 1`, `2`, puis `3` : pic mémoire, délai et fermeture éventuelle de l'onglet.
4. Servir un fichier distant de 300 Ko, puis un flux lent, et vérifier limite, délai, annulation et absence de requête avant consentement.
5. Afficher 1, 64 et 1 000 points contenant des sites afin de fixer une limite de portails fondée sur une mesure.

## Documents à mettre à jour si Yocthan retient les corrections

- `ADR-021` : vocabulaire et relations entre garde-fous ;
- `ADR-022` : consentement, identité d'origine, limites réseau et cycles ;
- `GUIDE.md` : syntaxe retenue et contrat des passages distants ;
- `COMPARAISON-WEB.md` : nouvel ordre de travail ;
- suite de conformité : cas contradictoires et limites de ressources.
