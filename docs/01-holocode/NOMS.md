# Les noms de HoloCode, face à ceux de HTML, CSS et JavaScript

Deux listes. La première part de HoloCode : pour chaque mot, celui du web qui lui correspond, et ce qu'on en a fait. La seconde part du web : les mots qu'on n'a pas pris.

Mis à jour le 2026-10-06.

**L'écriture des noms** (`ADR-037`, décidé par Yocthan le 2026-10-06) : celle de Flutter. La casse compte ; une seule écriture par mot ; deux mots se joignent par une majuscule (`appleX`, `topRight`, `BlueDoor`), jamais par `_` ; les styles gardent l'écriture du CSS (`font-size`).

| Sorte de mot | Écriture | Exemple |
|---|---|---|
| Bloc, et nom donné à un bloc | une majuscule au début et à chaque mot | `Button`, `BlueDoor`, `name: AddSunrise` |
| Paramètre, mot-valeur, signal, demande | une minuscule au début, une majuscule à chaque mot suivant | `title`, `topRight`, `tap`, `add` |
| Nom de valeur | pareil | `cart`, `appleX`, `blueDoor` |
| Style | comme en CSS : minuscules, mots joints par `-` | `font-size`, `.carte` |
| Unité d'octets | majuscules (B = octet, b = bit) | `KB`, `MB` | Le détail balise par balise est dans [`COMPARAISON-WEB.md`](COMPARAISON-WEB.md) ; le mode d'emploi dans [`GUIDE.md`](GUIDE.md).

Comment lire la colonne « Ce qu'on a fait » :

- **repris** : le même mot, le même sens ;
- **changé** : la même idée, un autre mot ;
- **nouveau** : le web n'a pas de mot pour cela.

Les quatorze noms que Codex et Gemini contestaient sont tranchés depuis le 2026-10-06 (`ADR-047`) : douze sont gardés ; `Points(grid:)` devient `divisions:` et `Points(depth:)` devient `levels:`. Les anciennes écritures sont refusées, avec le bon mot.

## 1. Les mots de HoloCode

### Les blocs

| HoloCode | HTML, CSS ou JavaScript | Ce qu'on a fait |
|---|---|---|
| `Page` | `html`, `head`, `body`, `main` | changé : un seul bloc au lieu de quatre balises |
| `H1` à `H6` | `h1` à `h6` | repris, avec une majuscule |
| `Video` | `video controls` | repris ; jamais de lecture automatique ; `label` obligatoire |
| `Table` et `caption:`, `head:`, `rows:` | `table`, `caption`, `thead`, `th`, `tbody`, `tr`, `td` | changé : trois paramètres au lieu de sept balises |
| `Choice` et `options:`, `menu:` | `input type="radio"`, `select`, `option` | changé : un seul bloc pour les deux ; `menu: true` pour la liste déroulante |
| `Input(lines:)` | `textarea` | changé : le même champ, sur plusieurs lignes |
| `Page(lang:, description:, image:)` | `html lang`, `meta name="description"`, `meta property="og:image"` | repris, sur la page |
| `Header`, `Nav`, `Main`, `Footer` | `header`, `nav`, `main`, `footer` | repris, avec une majuscule |
| `Stack` et `align:` sur ses enfants | `position: absolute`, `z-index`, `top`, `right` | changé : le nom de Flutter ; une place nommée au lieu de coordonnées |
| `hover:`, `focus:`, `active:` dans un style | `:hover`, `:focus-visible`, `:active` | repris, sans sélecteur : un état appartient à son style |
| `P` | `p` | repris, avec une majuscule |
| `Text` | `span` | changé : le mot dit ce que c'est |
| `A` | `a` | repris, avec une majuscule |
| `Button` | `button` | repris, avec une majuscule |
| `Image` | `img` | changé : le mot entier |
| `Shape` | `div` avec du CSS, ou `svg` | changé : quatre formes nommées |
| `Enter` | `@keyframes` + `animation` (de… vers…), ou `element.animate()` | changé : on écrit seulement d'où le bloc part |
| `Loop` | `animation: … infinite alternate` | changé : on écrit seulement où il va |
| `Scenes`, `Scene` | une suite de `animation-delay` calculés à la main, ou un chef d'orchestre en JavaScript | changé : les scènes s'enchaînent seules |
| `at:`, `for:`, `ease:` | `animation-delay`, `animation-duration`, `animation-timing-function` | changés : courts ; sept caractères nommés au lieu de courbes chiffrées |
| `letters:`, `each:` | du JavaScript qui coupe le texte en `span`, et un délai par `span` | nouveaux |
| `If(…, rules:)` | `if (…) { … }` autour d'un `setInterval` ; `clearInterval` à ne pas oublier | changé : les règles rangées dedans ne valent que si la condition est vraie |
| `If(size, is: "L")`, `When(answer, is: "Paris", …)` | `size === "L"`, un écouteur `input` qui compare | changé : à la lettre près ; un texte ne se compare pas à un nombre (`ADR-063`, à valider) |
| `State(price: 12.50)`, `{price}` | `Number`, `toFixed(2)`, `Intl.NumberFormat` | changé : exact (gardé à l'échelle), les chiffres fixés à la déclaration, la virgule de la langue d'office (`ADR-066`, à valider) |
| `Repeat(over: tasks, key: id)` | la `key` de React, `:key` de Vue | repris ; sans clé, le moteur prend le contenu, et le clavier est gardé quand même (`ADR-065`, à valider) |
| `total:` dans `Filter` | `filtered.length` avant `slice` | changé : un réglage, qui se montre et se compare (`ADR-065`, à valider) |
| `Data(name: Shop)`, `Shop.done`, `Shop.failed`, `Shop.refresh` | `fetch().then().catch()`, `response.ok`, `AbortController`, un bouton qui rappelle `fetch` | changé : un échec couvre l'erreur du serveur, le fichier trop gros ou illisible, et 10 secondes ; relectures espacées d'office (`ADR-064`, à valider) |
| `Sound` | `audio`, `new Audio().play()` | changé : un bruit qu'une règle déclenche, pas un lecteur |
| `List` | `ul`, `ol`, `li` | changé : un bloc au lieu de trois balises |
| `Row` | `display: flex` | changé : mot de Flutter |
| `Column` | `display: flex; flex-direction: column` | changé : mot de Flutter |
| `Grid` | `display: grid` | repris de CSS, devenu un bloc |
| `Point` | aucun (`canvas`, WebGL, à écrire soi-même) | nouveau |
| `World` | aucun | nouveau |
| `On` | `addEventListener`, `onclick` | changé : une règle, pas du code |
| `Input` | `input type="number"`, `input type="text"` | repris, avec une majuscule ; l'étiquette est obligatoire ; le genre du champ vient de la valeur |
| `Checkbox` | `input type="checkbox"` | changé : un mot à lui |
| `When` | un `if` testé à chaque image ; un test de collision écrit à la main | changé : une règle, déclenchée une fois, au moment où c'est vrai |
| `Every` | `setInterval` | changé : une règle, qui s'arrête seule quand on ne regarde pas |
| `After` | `setTimeout`, et `clearTimeout` à ne pas oublier | changé : une règle ; sous une condition, elle part quand la condition devient vraie et s'arrête seule si elle redevient fausse |
| `hover`, `hoverEnd` (signaux) | `mouseenter`, `mouseleave`, `focus`, `blur` | changé : un seul couple pour la souris, le clavier et le doigt |
| `else:` dans `If` | `else` | repris, devenu un paramètre du `If` |
| `Repeat`, `Item`, `item` | `for`, `map`, `v-for`, `{#each}`, `template` | changé : la liste et le modèle dans un bloc ; dépliés à la lecture, la page reste du HTML ordinaire |
| `line-height`, `letter-spacing`, `text-transform`, `text-decoration`, `box-shadow`, `text-shadow`, `rotate`, `scale`, `transition` | les mêmes | repris ; `line-height` sans unité seulement ; `transition` ne prend qu'une durée |
| `--or` (une variable) | `--or` et `var(--or)` | repris, employé sans `var( )` ; refusé s'il n'est défini nulle part |
| `dark:`, `phone:` dans un style | `@media (prefers-color-scheme: dark)`, `@media (max-width: 640px)` | changé : des états du style, comme `hover:` |
| `Font(family:, source:)` et `fonts:` | `@font-face` | changé : déclaré sur la page ; toujours `font-display: swap` |
| `url("fond.jpg")` dans `background` | `background-image`, `background-size`, `background-repeat` | changé : l'image couvre toujours le bloc |
| `State(tasks: [])`, `push`, `remove(item)`, `clear` | un tableau JavaScript, `push`, `splice`, `length = 0` | changé : une valeur de la page, changée par des demandes ; on retire la ligne touchée |
| `Repeat(over: tasks)` | `map` qui fabrique du HTML à chaque changement | changé : le moteur fabrique les lignes, et échappe ce que le visiteur a écrit |
| `text.set("")` | `input.value = ""` | changé : une demande |
| `module "…"`, `Module`, `run`, `done`, `failed` | `<script src>`, `new Worker`, `WebAssembly.instantiate` | changé : annoncé en haut du fichier, enfermé (un fil à part, une mémoire plafonnée, rien d'autre), arrêté s'il dure trop |
| `bridge js`, `bridge css` | `<script>`, `<link>` vers du code existant | refusés (`ADR-011`, partie B) |
| `mul`, `div` | `*=`, `/=`, `Math.floor` | changé : deux demandes, en nombres entiers |
| `computed:`, `Filter(name:, from:)` | `array.filter()`, `array.sort()`, `array.slice()` | changé : une liste nommée, refaite seule à chaque changement (`ADR-062`) |
| `contains:`, `in:` dans `Filter` | `string.includes()`, `toLowerCase()`, `normalize()` | changé : sans majuscules ni accents, d'office |
| `field:`, `is:`, `sortBy:`, `reverse:`, `limit:` dans `Filter` | `filter`, `sort` et `localeCompare`, `reverse`, `slice` | changé : des réglages ; `field`, `is`, `reverse` à valider |
| `empty:` dans `Repeat` | un `if (list.length === 0)` écrit à la main | nouveau : annoncé par un lecteur d'écran |
| `Key.enter`, `Key.escape`, `Key.a` à `Key.z`, `Key.digit0` à `Key.digit9` | `KeyboardEvent.key`, `KeyboardEvent.code` | changé : un nom par touche ; les lettres par ce qui est écrit sur la touche, les chiffres par leur place (un clavier français marche sans Maj) ; jamais Tab ; les touches à une lettre se coupent (WCAG 2.1.4) |
| `inView:` dans `Enter` | `IntersectionObserver`, `animation-timeline: view()` | nouveau : un mot, et rien qui tourne pendant qu'on défile |
| `volume:`, `loop:` dans `Sound`, `stop` | `audio.volume` (seulement en JavaScript), `loop`, `pause()` puis `currentTime = 0` | repris, et `stop` qui manque au web |
| `screen` dans `height:` | `100vh`, `100dvh` | changé : un mot ; la hauteur vraiment visible sur un téléphone |
| les pixels écrits en `rem` | `rem`, `em`, `vw`, `ch`… (douze unités) | changé : on écrit des pixels, le navigateur reçoit des `rem` |
| `{n:00}`, `{n:number}`, `{n:cents}`, `{weekday:name}` | `padStart`, `Intl.NumberFormat`, `toLocaleDateString` | changé : un mot après deux-points ; la langue vient de la page |
| `Form`, `send`, `sent`, `failed` | `form`, `fetch(…, { method: "POST" })`, `.then`, `.catch` | changé : une règle envoie, deux signaux répondent ; la page ne recharge jamais |
| `Dialog`, `open`, `close` | `dialog`, `showModal()`, `close()` | repris, ouvert et fermé par des règles |
| `Details(summary:)` | `details`, `summary` | repris |
| `Slider` | `input type="range"` | changé : le mot de Flutter |
| `Progress` | `progress` | repris, avec une étiquette obligatoire |
| `type: date`, `time`, `color` dans `Input` | `input type="date"`, `"time"`, `"color"` | repris |
| `today`, `{arrival:date}`, `due.add(7)`, `Days(name:, from:, to:)` | `new Date()`, `toLocaleDateString`, `setDate`, une différence de millisecondes | changé : un jour du calendrier, sans heure ni fuseau ; exact (`ADR-067`, à valider) |
| `min:` dans `Input` | `min`, `minlength` | repris : pour un nombre, une date (`ADR-067`), et la longueur d'un texte (`ADR-068`) |
| `required: true`, `type: email` | `required`, `type="email"`, `setCustomValidity`, `aria-invalid` | changé : vérifié par le moteur, dans la page puis au serveur ; messages sous le champ, lus par un lecteur d'écran (`ADR-068`, à valider) |
| `type: file`, `accept: image`, `max: 2MB` dans `Input` | `input type="file" accept="image/png,…"`, `FormData` | changé : des sortes nommées, une taille, vérifiées par la page et par le serveur |
| `caption:`, `phone:` dans `Image` | `figure`, `figcaption` ; `picture`, `source media` | changé : deux paramètres de l'image |
| `label:` dans `Sound` | `audio controls` | changé : avec une étiquette, le son devient un lecteur |
| `icon:` dans `Page` | `link rel="icon"` | changé |
| `~~…~~`, `==…==`, `^…^`, `~…~` | `s`, `mark`, `sup`, `sub` | changé : écrits dans le texte, comme le gras |
| `key:` dans `Item` | `key` de React et de Vue | repris : c'est aussi le nom d'une valeur de la page, et il donne leur nom aux blocs de la copie |
| `year`, `month`, `day`, `weekday`, `hour`, `minute` | `new Date()`, `getFullYear()`, `getMonth() + 1`, `getDay()` | changé : six valeurs qu'on lit, sans objet ni calcul ; le mois va de 1 à 12, la semaine commence le lundi |
| `Board` | `position: relative` et `absolute` | changé : un bloc, des places de 0 à 100 |
| `Component`, `Use` | les composants de React, Vue, Angular ; `template` | changés : un composant nommé, posé par son nom ; `Use(Menu)` pour un composant sans paramètres. Anciennement `Part`, renommé pour garder `Part` à la 3D (ADR-056) |
| `grow` | `flex-grow` (CSS), `Expanded` (Flutter) | changé : un réglage d'un bloc rangé dans `Row` ou `Column` (ADR-052) |
| `components`, `params` (avec des valeurs par défaut), `emits`, `emit`, `onAdd`, et le composant posé par son nom : `ArticleCard(title: …)` | les composants de React, Vue, Svelte ; les widgets de Flutter ; les Web Components | changés : un composant se pose comme un bloc, à la manière de Flutter, et se restyle par le CSS (`ArticleCard { … }`, `ArticleCard.promo(…)`, ses variables) ; `props` et `slot` ne sont pas repris ; les signaux émis comme `emit` de Vue, branchés par `onAdd:` (ADR-050, ADR-056) |
| `If` | `if` en JavaScript, `v-if`, `{#if}` | repris, devenu un bloc |
| `Hr` | `hr` | repris, avec une majuscule |
| `Quote` | `blockquote`, `q`, `cite` | changé : un mot au lieu de trois |
| `Code` | `pre`, `code` | repris |
| `State` | `let`, `useState`, les signaux | changé : une déclaration, pas une variable libre |
| `Data` | `fetch`, puis `JSON.parse`, puis la mise à jour de la page | changé : une déclaration ; l'arbitre range ce qui arrive |
| `Prices` | un objet JavaScript `{ sunrise: 120 }` | changé : une table déclarée |
| `Zoom` | `meta viewport`, la propriété `zoom` | changé |
| `Points` | aucun | nouveau |
| `Relief` | `transform: perspective() rotate3d()` | changé |
| `Portals` | aucun (un routeur en JavaScript) | nouveau |

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
| `state:`, `prices:`, `data:` | aucun | nouveaux |
| `from:` | l'adresse donnée à `fetch` | changé : un fichier rangé à côté, rien d'autre |
| `every:` | `setInterval` | changé |
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
| `pixels:`, `above:` | `position: absolute` | changés |
| `zoom:`, `points:`, `relief:`, `portals:` | aucun | nouveaux |
| `active:` | `user-scalable=no` | changé |
| `max:` | `maximum-scale` | changé |
| `shrink:` | `minimum-scale` ; `flex-shrink` dit autre chose | changé |
| `levels:`, `speed:` | aucun | nouveaux ; `levels:` dans `Zoom` et dans `Points`, avec la même idée : combien de fois l'un dans l'autre |
| `after:` | aucun ; `::after` dit autre chose | nouveau |
| `size:` | `width`, `height` | changé |
| `fragment:` | aucun ; le « fragment » d'une adresse dit autre chose | nouveau |
| `divisions:` | aucun ; les `divisions` d'une grille de Three.js | nouveau ; s'appelait `grid:` jusqu'au 2026-10-06 |
| `density:` | aucun | nouveau ; `Points(levels:)` s'appelait `depth:` jusqu'au 2026-10-06, mot gardé pour la profondeur de la 3D |
| `height:` | `height` | repris |
| `tilt:` | `rotate3d()` | changé ; le mot dit « incliner », pas « faire le tour » |
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
| `play` | `audio.play()` | repris |
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
| Variables CSS, dégradés, ombres, `@font-face` | Le reste de l'apparence. |
| `if`, `for`, `map` | Afficher sous condition, répéter sur une liste. |
| `fetch`, `localStorage`, `setTimeout` | Les données venues d'ailleurs, gardées, et le temps. |
| Les événements de survol, de clavier, de défilement | D'autres signaux que `tap`. |

## 3. Le compte

- Mots repris tels quels ou presque : environ 35 (les titres, `P`, `A`, `Button`, les quinze réglages de style, les unités).
- Mots changés : environ 30.
- Mots nouveaux : environ 30, presque tous autour du point, du monde et de la façon de regarder la page.
- Noms contestés : aucun. Les quatorze sont tranchés le 2026-10-06 (`ADR-047`).
