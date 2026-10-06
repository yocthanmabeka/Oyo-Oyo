# La même boutique, écrite deux fois

Demandé par Yocthan le 2026-10-03 : un même exemple écrit en HoloCode et dans le trio HTML, CSS et JavaScript, qui emploie tout le vocabulaire du langage, de la façon dont on s'en servirait pour un métavers.

| | Fichiers | Lignes utiles (sans les lignes vides ni les commentaires) |
|---|---|---|
| HoloCode | [`boutique.holo`](boutique.holo) | 176 |
| Web | [`web/index.html`](web/index.html), [`web/style.css`](web/style.css), [`web/script.js`](web/script.js) | 348 (90 + 103 + 155) |

Compte du 2026-10-06 (lignes non vides, sans les lignes de commentaire). Les deux versions sont identiques à l'écran, vérifié par captures côte à côte sur ordinateur et sur téléphone.

Les deux décrivent la même chose : une page de boutique (titres, paragraphes, image, liste, bouton), un point lumineux, et le monde dans lequel on entre par ce point, avec ses six points nés du morcellement.

## Où en est chaque version

- **La version web fonctionne aujourd'hui** : ouvrir `web/index.html` dans un navigateur. Le bouton fait entrer dans l'atelier, « Back to the shop » en fait sortir. Les graines des six points sont calculées comme dans le moteur (vérifié : mêmes valeurs que `moteur/src/graine.rs`).
- **La version HoloCode s'affiche aussi** : lancer `node outils/serveur.mjs` dans `moteur/`, puis ouvrir `http://localhost:8080/page.html`. Le moteur fabrique la page à partir de `boutique.holo` ; le bouton fait entrer dans le point, dont le monde est celui du moteur (les points sur une sphère où l'on zoome), avec son contenu lisible sur un panneau. Un test relit le fichier à chaque changement et contrôle qu'aucun mot du langage n'y manque.
- Pour comparer côte à côte : `http://localhost:8080/exemples/boutique-comparee/web/index.html`.

## Terme par terme

| HoloCode | HTML, CSS, JavaScript | Remarque |
|---|---|---|
| `Page(title: "…")` | `<html>`, `<head>`, `<title>`, `<body>`, `<main>` | |
| `H1`, `H2`, `H3` | `<h1>`, `<h2>`, `<h3>` | HoloCode refuse de sauter un niveau |
| `"Une phrase"` ou `P("…")` | `<p>…</p>` | |
| `Text("…")` | `<span>…</span>` | Du texte sans rôle |
| `**gras**` dans un texte | `<strong>gras</strong>` | Markdown |
| `Image(source: …, weight: …)` | `<img src="…">` | Le poids n'existe pas en HTML |
| `List(children: […])` | `<ul>` et `<li>` | |
| `Button(name: Open, text: "…")` | `<button id="open">…</button>` | |
| `P.card(…)` | `<p class="card">` | Une classe inconnue est refusée en HoloCode |
| `.card { … }`, `P { … }` | Identique | Mêmes noms de réglages |
| `Page { … }` (le thème) | `body { … }` et `.page { … }` | |
| `World { … }` | `.world { … }` | |
| `Point(name, seed, brightness, fragments, color, palette)` | Un `<div>`, 13 lignes de CSS pour la boule, 30 lignes de JavaScript pour les graines et les points enfants | Le mot n'existe pas dans le web |
| `budget: 500KB` et `weight: 1KB` | Une fonction `checkBudget` écrite à la main | Rien ne l'impose dans le web |
| `inside: World(children: […])` | Une `<section hidden>` déjà chargée dans la page | |
| `On(Open.tap, effect: Workshop.enter)` | `addEventListener("click", enter)` et la fonction `enter` | |
| `On(Back.tap, effect: Workshop.leave)` | `addEventListener("click", leave)` et la fonction `leave` | |
| `import`, `module` | `<link>`, `<script>` | En commentaire dans `boutique.holo` : appliqués depuis (leçons 25 et 69) ; `bridge js` et `bridge css` sont refusés (ADR-011, partie B) |

## Ce que la comparaison montre, sans embellir

- **Pour la page seule, l'écart est faible.** Titres, paragraphes et styles s'écrivent presque pareil : c'est voulu, les styles reprennent l'écriture du CSS (`ADR-017`).
- **L'écart vient du point et du monde.** Le web n'a pas de mot pour « un point dans lequel on entre » : il faut le fabriquer avec un `div`, un dégradé et du JavaScript. C'est là que se trouvent les deux tiers des lignes de la version web.
- **La version web triche sur un point** : le monde de l'atelier est chargé dès le départ et simplement caché, et ses points enfants sont posés sur un cercle. Le vrai moteur ne calcule un monde que lorsqu'on y entre, et place les points sur une sphère où l'on zoome.
- **Ce que le web tolère et que HoloCode refuse** : dans la version web, rien n'empêche d'écrire `class="carte"` sans style correspondant, d'oublier un `;`, ou de mettre n'importe quel code dans le clic du bouton.

## Termes du langage qui n'ont pas encore d'emploi

Ils existent dans la grammaire mais aucun bloc ne s'en sert aujourd'hui, donc ils ne figurent pas dans l'exemple :

- les unités de longueur et de durée (`mm`, `cm`, `m`, `km`, `ms`, `s`, `min`, `h`) et les tailles `B`, `MB`, `GB` (seul `KB` est employé) ;
- `seed: auto`, qui n'est permis qu'au moment de la création, avant d'être remplacé par un nombre.

## Ce que la version web ne fait pas

Depuis le 2026-10-03, `boutique.holo` écrit aussi comment sa page se regarde (`Zoom`, `Points`, `Relief`, voir `ADR-021`) : en zoomant, chaque pixel devient un point qui se morcelle, et la page prend du relief quand on la tourne. La version web n'a pas d'équivalent : il faudrait écrire un moteur de rendu. Ces dix-sept lignes de `boutique.holo` ne comptent donc pour rien du côté web.

## Choix faits pour cet exemple

- `List(children: […])` : le bloc `List` existait dans le vocabulaire sans exemple ; il prend ses éléments dans `children`, comme les autres blocs.
- Les règles du monde intérieur sont écrites dans ce monde (`World(rules: […])`).

Depuis le 2026-10-04, la boutique a un panier (`ADR-023`, décidé depuis). En HoloCode : une ligne pour déclarer (`state: State(cart: 0)`), `{cart}` dans le texte, et une règle par bouton (`On(Add.tap, effect: cart.add(1))`). En JavaScript : une variable, une fonction `showCart()` à rappeler après chaque changement, et un garde-fou écrit à la main pour ne pas descendre sous zéro. Oublier un seul appel à `showCart()`, et l'écran ne dit plus la vérité.

Le même jour, la disposition (`ADR-024`, décidé depuis) : `Grid(columns: 3)` et `Row(gap: 8px)` en HoloCode ; en CSS, `display: grid` avec une formule `minmax` pour le téléphone, et `display: flex` sans oublier `flex-wrap`.

## Mesure du 2026-10-04 : est-ce plus rapide que le web ?

Yocthan a demandé de comparer, « histoire de voir si le langage est plus rapide que la majorité des langages ou pas ». Première mesure, sur le PC, dans Chrome sans rien en cache, trois essais chacun, serveur local avec compression Brotli. Les deux pages font la même chose à plat (texte, grille, panier, atelier) ; seule la version HoloCode a les points, la rotation et le carrefour.

| | `boutique.holo` | `web/` (HTML, CSS, JavaScript) |
|---|---|---|
| Premier affichage | 368 à 556 ms | 288 à 384 ms |
| Octets transférés | 579 253 | 6 115 |
| Fichiers demandés | 7 | 6 |
| Mémoire JavaScript | 1,8 Mo (sans compter la mémoire du moteur) | 0,8 Mo |
| Éléments dans la page | 81 | 67 |
| Lignes utiles écrites par l'auteur | 124, dans 1 fichier | 236, dans 3 fichiers |

Ce que cela dit, sans détour :

- **HoloCode n'est pas plus rapide.** Le premier affichage est à peu près le même, parce que le serveur envoie la page déjà fabriquée ; sans cela, il serait plus lent.
- **HoloCode est près de cent fois plus lourd à la première visite** : le moteur pèse environ 570 Ko. Il est téléchargé une seule fois, puis gardé par le navigateur ; un second site `.holo` ne le retélécharge pas. Un navigateur qui saurait lire le `.holo` n'aurait rien à télécharger d'autre que les 9 Ko du fichier.
- **L'auteur écrit deux fois moins de lignes**, dans un seul fichier, et sans JavaScript.
- **La comparaison n'est pas complète** : la version web n'a ni points, ni rotation, ni carrefour. Pour les avoir, il faudrait une bibliothèque 3D (Three.js pèse autour de 170 Ko compressé, à vérifier) et beaucoup de code.

Ce qui n'est pas mesuré : un site écrit avec React, Vue ou Svelte (il faudrait les installer, ce qui demande l'accord de Yocthan) ; le téléphone ; une page bien plus grande ; la vitesse des interactions.


## Mise à jour du 2026-10-06 : repères, survol, pastille, titres, texte qui grandit

Les deux versions ont reçu les mêmes ajouts, et restent identiques à l'écran (captures côte à côte, ordinateur et téléphone) :

| Ajout | HoloCode | HTML, CSS, JavaScript |
|---|---|---|
| En-tête et menu, pied de page | `Header(…)`, `Nav(…)`, `Footer(…)` | `<header>`, `<nav>`, `<footer>`, et une règle CSS pour leur largeur |
| Pastille « New » sur le tableau | `Stack(children: [ Image(…), Text.badge("New", align: topRight) ])` | un conteneur en grille, `grid-area`, `align-self`, `justify-self`, `z-index` |
| Survol, clavier, appui | `Button { hover: { … } focus: { … } active: { … } }` | `@media (hover: hover) { button:hover { … } }`, `:focus-visible`, `:active`, `transition` |
| Titre de niveau 4 | `H4("Weekdays")` | `<h4>` |
| Texte qui suit le réglage du visiteur | rien à écrire : le moteur change les `px` en `rem` et `clamp()` | `font-size: 1rem`, `clamp(1.5rem, 5vw, 2rem)` à écrire à la main |

**Mesuré dans Chrome (PC, cache vide, trois essais chacun)** :

| | `boutique.holo` | `web/` |
|---|---|---|
| Téléchargé à l'ouverture | **9 Ko**, sans le moteur | 8 Ko |
| Fichiers demandés | 3 | 5 |
| Premier affichage | 340 à 604 ms | 332 à 568 ms |
| Éléments dans la page | 122 | 97 |
| Lignes utiles écrites par l'auteur | **176**, dans 1 fichier | 348, dans 3 fichiers |

Ce que cela dit :

- **Le poids est maintenant le même** : depuis le site léger (`ADR-033`), la boutique HoloCode ne télécharge plus le moteur pour être lue (579 Ko le 2026-10-04, 9 Ko aujourd'hui). Le moteur n'arrive qu'au premier bouton touché.
- **Le premier affichage est le même**, aux variations près d'un essai à l'autre.
- **L'auteur écrit environ deux fois moins**, dans un seul fichier, sans JavaScript.
- **HoloCode met plus d'éléments dans la page** (122 contre 97) : le moteur enveloppe certains blocs (la pastille, les marques des valeurs).
- **Une erreur faite en écrivant la version web**, qui montre le défaut que HoloCode évite : une règle CSS ajoutée pour les marges était plus « forte » que celles de la grille, du menu et de la pastille, et les écrasait. La grille passait à une colonne, la pastille tombait sous l'image. Il a fallu `:where()` pour l'affaiblir. En HoloCode, ce conflit ne peut pas arriver : un style ne vise qu'un type de bloc ou un nom.
