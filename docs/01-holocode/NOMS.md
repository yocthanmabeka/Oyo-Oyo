# Les noms de HoloCode, face à ceux de HTML, CSS et JavaScript

Deux listes. La première part de HoloCode : pour chaque mot, celui du web qui lui correspond, et ce qu'on en a fait. La seconde part du web : les mots qu'on n'a pas pris.

Mis à jour le 2026-10-04. Le détail balise par balise est dans [`COMPARAISON-WEB.md`](COMPARAISON-WEB.md) ; le mode d'emploi dans [`GUIDE.md`](GUIDE.md).

Comment lire la colonne « Ce qu'on a fait » :

- **repris** : le même mot, le même sens ;
- **changé** : la même idée, un autre mot ;
- **nouveau** : le web n'a pas de mot pour cela.

Les noms marqués ⚠ sont contestés (par Codex ou Gemini) et attendent la décision de Yocthan.

## 1. Les mots de HoloCode

### Les blocs

| HoloCode | HTML, CSS ou JavaScript | Ce qu'on a fait |
|---|---|---|
| `Page` | `html`, `head`, `body`, `main` | changé : un seul bloc au lieu de quatre balises |
| `H1`, `H2`, `H3` | `h1`, `h2`, `h3` | repris, avec une majuscule |
| `P` | `p` | repris, avec une majuscule |
| `Text` | `span` | changé : le mot dit ce que c'est |
| `A` | `a` | repris, avec une majuscule |
| `Button` | `button` | repris, avec une majuscule |
| `Image` | `img` | changé : le mot entier |
| `List` | `ul`, `ol`, `li` | changé : un bloc au lieu de trois balises |
| `Row` | `display: flex` | changé : mot de Flutter |
| `Column` | `display: flex; flex-direction: column` | changé : mot de Flutter |
| `Grid` ⚠ | `display: grid` | repris de CSS, devenu un bloc ; même mot que `Points(grid:)` |
| `Point` | aucun (`canvas`, WebGL, à écrire soi-même) | nouveau |
| `World` | aucun | nouveau |
| `On` | `addEventListener`, `onclick` | changé : une règle, pas du code |
| `Input` | `input type="number"`, `input type="text"` | repris, avec une majuscule ; l'étiquette est obligatoire ; le genre du champ vient de la valeur |
| `Checkbox` | `input type="checkbox"` | changé : un mot à lui |
| `When` | un `if` testé à chaque image ; un test de collision écrit à la main | changé : une règle, déclenchée une fois, au moment où c'est vrai |
| `Every` | `setInterval` | changé : une règle, qui s'arrête seule quand on ne regarde pas |
| `Board` | `position: relative` et `absolute` | changé : un bloc, des places de 0 à 100 |
| `If` | `if` en JavaScript, `v-if`, `{#if}` | repris, devenu un bloc |
| `Hr` | `hr` | repris, avec une majuscule |
| `Quote` | `blockquote`, `q`, `cite` | changé : un mot au lieu de trois |
| `Code` | `pre`, `code` | repris |
| `State` | `let`, `useState`, les signaux | changé : une déclaration, pas une variable libre |
| `Prices` | un objet JavaScript `{ sunrise: 120 }` | changé : une table déclarée |
| `Zoom` ⚠ | `meta viewport`, la propriété `zoom` | changé |
| `Points` ⚠ | aucun | nouveau |
| `Relief` ⚠ | `transform: perspective() rotate3d()` | changé |
| `Portals` ⚠ | aucun (un routeur en JavaScript) | nouveau |

### Les réglages des blocs

| HoloCode | HTML, CSS ou JavaScript | Ce qu'on a fait |
|---|---|---|
| `name:` | `id` | changé : le nom sert aux règles, pas aux styles |
| `title:` | `title` | repris |
| `children:` | les balises imbriquées | changé : mot de Flutter |
| `text:` | le texte entre les balises | changé |
| `to:` | `href` | changé : « vers » se comprend sans l'apprendre |
| `source:` | `src` | changé : le mot entier |
| `ordered:` | `ol` | changé : un réglage au lieu d'une balise |
| `rules:` | les balises `script` | changé |
| `effect:` | le corps d'une fonction | changé : une demande, pas du code |
| `state:`, `prices:` | aucun | nouveaux |
| `alt:` | `alt` | repris |
| `value:` | `value`, `v-model`, `bind:value` | repris ; lié dans les deux sens |
| `label:` | `label` | repris, devenu un réglage obligatoire |
| `max:` (d'un champ) | `max` | repris ; vraiment appliqué |
| `meets:`, `within:` | aucun | nouveaux |
| `drag:` | `draggable`, `pointermove` | changé : un réglage, rien à programmer |
| `keep:` | `localStorage` | changé : une liste de noms, rien à programmer |
| `x:`, `y:` | `left`, `top` | changés : de 0 à 100, jamais hors du plateau |
| `by:` | `cite` | changé |
| `is:`, `not:`, `over:`, `under:` | `===`, `!==`, `>`, `<` | changés : des mots, pas des signes |
| `gap:` | `gap` | repris |
| `align:` | `justify-content`, `align-items` | changé : un mot au lieu de deux |
| `columns:` | `grid-template-columns` | changé ; en CSS, `columns` dit autre chose |
| `start`, `center`, `end` | `flex-start`, `center`, `flex-end` | repris, raccourcis |
| `between` | `space-between` | repris, raccourci |
| `seed:`, `fragments:`, `palette:`, `inside:` | aucun | nouveaux |
| `brightness:` | `filter: brightness()` | repris |
| `color:` | `color` | repris |
| `budget:`, `weight:` | aucun | nouveaux |
| `pixels:` ⚠, `above:` ⚠ | `position: absolute` | changés, écriture provisoire |
| `zoom:`, `points:`, `relief:`, `portals:` | aucun | nouveaux |
| `active:` | `user-scalable=no` | changé |
| `max:` | `maximum-scale` | changé |
| `shrink:` ⚠ | `minimum-scale` ; `flex-shrink` dit autre chose | changé |
| `levels:` ⚠, `speed:` | aucun | nouveaux |
| `after:` ⚠ | aucun ; `::after` dit autre chose | nouveau |
| `size:` | `width`, `height` | changé |
| `fragment:` ⚠ | aucun ; le « fragment » d'une adresse dit autre chose | nouveau |
| `grid:` ⚠ | aucun ; `display: grid` dit autre chose | nouveau |
| `depth:` ⚠, `density:` | aucun | nouveaux |
| `height:` | `height` | repris |
| `tilt:` ⚠ | `rotate3d()` | changé ; le mot dit « incliner », pas « faire le tour » |
| `layout:` | aucun | nouveau |
| `count:` | aucun | nouveau |
| `duration:` | `transition-duration` | repris, raccourci |
| `grid`, `row`, `column`, `diagonal` (valeurs de `layout:`) | aucun | nouveaux |

### Les signaux, les capacités, les demandes

| HoloCode | HTML, CSS ou JavaScript | Ce qu'on a fait |
|---|---|---|
| `tap` | `click`, `touchend` | changé : un mot pour la souris et le doigt |
| `Key.left`, `Key.right`, `Key.up`, `Key.down`, `Key.space` | `keydown`, `ArrowLeft`… | changés : un signal comme les autres |
| `enter`, `leave` | un routeur ; `history.pushState` | changés |
| `portals` | aucun | nouveau |
| `{cart}` | `${cart}` en JavaScript, `{cart}` en Svelte, `{{ cart }}` en Vue | repris |
| `add`, `sub`, `set` | `+=`, `-=`, `=` | changés : des mots, pas des signes |
| `random` | `Math.random()` | repris ; rejouable, et de 0 à n compris |
| `{count}`, `{total}` | une boucle `for` ou `reduce` écrite à la main | nouveaux : calculés par le moteur |

### Les styles

| HoloCode | CSS | Ce qu'on a fait |
|---|---|---|
| `P { … }` | `p { … }` | repris |
| `.card { … }` | `.card { … }` | repris |
| `P.card(...)` | `class="card"` | changé |
| `color`, `background`, `font-size`, `font-weight`, `font-style`, `font-family`, `text-align`, `padding`, `margin`, `border`, `border-radius`, `width`, `height`, `max-width`, `opacity` | les mêmes | repris, quinze réglages |

### Les unités et le reste

| HoloCode | HTML, CSS ou JavaScript | Ce qu'on a fait |
|---|---|---|
| `px`, `deg`, `ms`, `s`, `%` | les mêmes | repris |
| `B`, `KB`, `MB`, `GB` | aucun | nouveaux |
| `mm`, `cm`, `m`, `km`, `min`, `h` | en partie (`mm`, `cm`) | repris ou nouveaux ; sans emploi aujourd'hui |
| `true`, `false` | les mêmes | repris |
| `//` | `//` en JavaScript | repris |
| `**gras**`, `*italique*` | `strong`, `em` | changés : l'écriture de Markdown |
| `import`, `module`, `bridge js`, `bridge css` | `import`, `link`, `script` | repris ou changés ; lus, pas encore appliqués |

## 2. Les mots du web qu'on n'a pas pris

### Refusés exprès

| Web | Pourquoi |
|---|---|
| `div` | Un bloc dit ce qu'il est. |
| `h4`, `h5`, `h6` | Trois niveaux de titres suffisent. |
| `ul`, `ol`, `li` | Un seul bloc, `List`. |
| `section`, `article` | Les titres donnent le plan. |
| `script`, `onclick`, le DOM modifié à la main | Pas de code libre dans un bloc. |
| `display`, `position`, `float` dans un style | La disposition vient des blocs. |
| Sélecteurs composés, cascade, `!important` | Un style vise un type ou un nom, rien d'autre. |
| `justify-content`, `align-items`, `flex-wrap`, `@media` | Le moteur s'en charge ; un seul mot, `align`. |
| `var`, `let`, `function`, `this`, `null`, `undefined`, `NaN` | Pas de variables libres ; une valeur est déclarée et bornée. |
| `noscript` | Sans objet. |

### Pas encore là

| Web | Ce qui est prévu |
|---|---|
| `img alt` | Le texte de remplacement d'une image. |
| `br`, `hr`, `blockquote`, `pre`, `code` | Le texte qui manque. |
| `header`, `footer`, `nav`, `aside` | Les parties d'une page. |
| `table`, `tr`, `td`, `th` | Les tableaux. |
| `form`, `input`, `textarea`, `select`, `label` | Les formulaires. |
| `video`, `audio` | Les médias. |
| `details`, `summary`, `dialog` | Ce qui s'ouvre et se ferme. |
| `meta description`, l'image de partage | Ce que lisent les moteurs de recherche. |
| `link rel="stylesheet"`, `template`, `slot` | Les imports. |
| `:hover`, `:focus`, `:active` | L'apparence selon l'état. |
| `transition`, `animation`, `@keyframes` | Les animations. |
| Variables CSS, dégradés, ombres, `@font-face` | Le reste de l'apparence. |
| `if`, `for`, `map` | Afficher sous condition, répéter sur une liste. |
| `fetch`, `localStorage`, `setTimeout` | Les données venues d'ailleurs, gardées, et le temps. |
| Les événements de survol, de clavier, de défilement | D'autres signaux que `tap`. |

## 3. Le compte

- Mots repris tels quels ou presque : environ 35 (les titres, `P`, `A`, `Button`, les quinze réglages de style, les unités).
- Mots changés : environ 30.
- Mots nouveaux : environ 30, presque tous autour du point, du monde et de la façon de regarder la page.
- Noms contestés, à trancher : `Grid` face à `grid:`, `pixels`, `above`, `Zoom`, `Points`, `Relief`, `Portals`, `shrink`, `levels`, `after`, `fragment`, `depth`, `tilt`.
