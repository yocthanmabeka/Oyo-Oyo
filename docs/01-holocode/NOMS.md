# Les noms de HoloCode, face à ceux de HTML, CSS et JavaScript

Deux listes. La première part de HoloCode : pour chaque mot, celui du web qui lui correspond, et ce qu'on en a fait. La seconde part du web : les mots qu'on n'a pas pris.

Mis à jour le 2026-10-08.

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
| `Video(captions:)` | `track kind="captions"` | changé : un paramètre, la langue prise dans la page |
| `Aside` | `aside` | repris |
| `A(newTab: true)`, `A(download: true)` | `target="_blank" rel="noopener"`, `download` | changé : `noopener` et l'annonce au lecteur d'écran viennent d'office |
| `print: { … }` dans un style | `@media print` | changé : un état, comme `dark:` et `phone:` |
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
| `Drawing` | `svg role="img" aria-label` | changé : le nom (`label`) est obligatoire ; les mesures sont celles du dessin, qui garde ses proportions |
| `Rect`, `Circle`, `Line`, `Path` | `rect`, `circle`, `line`, `path` | repris, avec des mots lisibles : `x`, `y`, `r`, `radius`, `from`, `to`, `thickness` au lieu de `cx`, `rx`, `x1`, `stroke-width` ; un tracé filtré |
| `Chart` | une bibliothèque de graphiques en JavaScript (Chart.js), ou `svg` à la main | nouveau : un mot pour une intention fréquente ; dessiné par le moteur, avec un tableau caché pour le lecteur d'écran |
| `Stopwatch`, `start`, `stop`, `reset`, `stopped` | `performance.now()`, `requestAnimationFrame`, `role="timer"` | nouveau : un chronomètre que la page fait tourner ; le moteur ne reçoit que le temps final |
| `Transfer(file:, values:)`, `export`, `import` | un lien `download` et un `Blob`, `input type="file"`, `FileReader`, `JSON.parse` | nouveau : un fichier JSON des seules valeurs annoncées ; l'import est relu en entier, puis pris tout entier ou refusé (`ADR-093`) |
| `Device(kind: position \| clipboard \| camera \| microphone)`, `request`, `write`, `stop` | `navigator.geolocation`, `navigator.clipboard`, `getUserMedia()` | nouveau : un mot pour l'appareil, quatre sortes nommées ; sur le toucher d'un bouton ; rien n'est envoyé ; l'arrêt vient d'office (`ADR-094`) |
| `Notification(title:, body:, after:)`, `show`, `stop` | `Notification.requestPermission()`, `showNotification()`, `setTimeout` | repris : le nom du web ; un rappel seulement tant que la page est ouverte, sans « push » (`ADR-095`) |
| `Offline(files:)`, `save`, `remove` | un service worker et `CacheStorage` écrits à la main | nouveau : une copie d'une page publique, demandée par le visiteur ; le réseau d'abord ; rien n'est mis en attente ni rejoué (`ADR-096`) |
| `abbreviations: [ Abbreviation("MJC", "…") ]` | `<abbr title="…">` à chaque venue | changé : déclarée une fois pour la page ; le moteur la marque partout et écrit son sens à sa première venue dans un paragraphe, lu et vu aussi au doigt (`ADR-098`) |
| (rien : `{ouverture:date}`) | `<time datetime="…">` | nouveau sans mot : chaque date montrée est aussi lisible par les machines (`ADR-098`) |
| `Address(children: [ … ])` | `address` | repris : le nom de HTML, les moyens de joindre l'auteur ; ni titre ni repère dedans ; le texte reste droit (`ADR-098`) |
| `Fields(label:, children:)` | `fieldset`, `legend` | changé : un groupe de champs et son nom, écrit `label:` comme celui d'un champ ; au moins deux champs ; sans la bordure du navigateur, jamais plus large que l'écran (`ADR-099`) |
| `Input(suggestions: ["Paris", "Lyon"])`, `suggestions: villes` | `input list="…"`, `datalist`, `option` | changé : un paramètre du champ au lieu d'un élément relié par un `id` ; écrites, ou une liste de la page suivie pendant la visite ; on peut toujours écrire autre chose (`ADR-100`) |
| `<<bonjour>>` dans un texte | `q` | nouveau : une marque du texte enrichi ; le moteur écrit les guillemets de la langue de la page, « » en français avec une espace fine insécable, “ ” en anglais, ceux du second niveau dans une citation (`ADR-101`) |
| `_Les Misérables_` dans un texte ; `Quote(work:)` | `cite` | nouveau : le titre d'une œuvre ; un trait bas au milieu d'un mot reste un trait bas (`ADR-101`) |
| `negative: [temperature]`, `State(temperature: -2)`, `If(temperature, under: -20)` | tout nombre de JavaScript est signé ; `input type="number" min="-50"` ; `Intl.NumberFormat` pour le signe | nouveau : seule une valeur nommée descend sous zéro, le panier reste à 0 ; exact, calculé comme sans le signe ; le signe moins de la langue de la page, jamais « -0 » ; un champ dont le clavier a le signe moins (`ADR-102`) |
| `columnSpan:`, `rowSpan:` sur un bloc de `Grid` | `grid-column: span 2`, `grid-row: span 2` ; `colspan`, `rowspan` des tableaux | changé : un réglage du bloc, avec les mots des tableaux de HTML ; une grille trop étroite donne toute la ligne à la case au lieu de déborder (`ADR-104`) |
| `Grid(areas: ["top top", "menu main"])`, `area: menu` | `grid-template-areas`, `grid-area` | repris, dans les blocs : les blocs s'écrivent dans l'ordre des zones, celui de la lecture ; une faute de nom, une zone qui n'est pas un rectangle sont refusées ; sur un téléphone, les zones s'empilent dans cet ordre (`ADR-104`) |
| `Device(kind: share)`, `request` | `navigator.share()`, `navigator.canShare()`, `navigator.clipboard.writeText()` | repris : le mot du web, comme une sorte d'appareil ; le titre et l'adresse de la page, dans le toucher même ; sans feuille de partage, l'adresse copiée ; la feuille fermée n'est pas une panne (`ADR-107`) |
| `reorder: true` dans `Repeat(over:)` | `draggable="true"`, `dragstart`, `dragover`, `drop`, `dataTransfer`, et le code qui range le tableau ; ou une bibliothèque (SortableJS) | changé : le mot de Flutter (`ReorderableListView`) ; un réglage, rien à programmer ; la poignée, « Monter » et « Descendre » viennent du moteur, au doigt, à la souris, au clavier, au lecteur d'écran qui entend la nouvelle place, et sans JavaScript ; l'arbitre change la liste (`ADR-105`) |
| `Device(kind: vibration, for: 200ms)`, `play`, `stop` | `navigator.vibrate()` | repris : le mot du web, comme une sorte d'appareil ; se joue comme un son, là où un son se joue, après le premier toucher ; des durées avec leur unité, une seconde en tout ; rien sous le mouvement réduit ; ne dit rien en retour (`ADR-110`) |
| `Sound(fade: 2s)` ; `Sound(volume: pluie)` | Web Audio : `createMediaElementSource()`, `GainNode`, `linearRampToValueAtTime()` ; `audio.volume` | nouveau : un fondu à l'entrée et à la sortie, en un mot ; un volume qui suit une valeur de la page, de 0 à 100, et glisse jusqu'à elle : une glissière par son fait une table de mixage ; jamais un son avant un geste du visiteur (`ADR-112`) |
| `Enter` | `@keyframes` + `animation` (de… vers…), ou `element.animate()` | changé : on écrit seulement d'où le bloc part |
| `Loop` | `animation: … infinite alternate` | changé : on écrit seulement où il va |
| `Scenes`, `Scene` | une suite de `animation-delay` calculés à la main, ou un chef d'orchestre en JavaScript | changé : les scènes s'enchaînent seules |
| `at:`, `for:`, `ease:` | `animation-delay`, `animation-duration`, `animation-timing-function` | changés : courts ; sept caractères nommés au lieu de courbes chiffrées |
| `letters:`, `each:` | du JavaScript qui coupe le texte en `span`, et un délai par `span` | nouveaux |
| `If(…, rules:)` | `if (…) { … }` autour d'un `setInterval` ; `clearInterval` à ne pas oublier | changé : les règles rangées dedans ne valent que si la condition est vraie |
| `If(size, is: "L")`, `When(answer, is: "Paris", …)` | `size === "L"`, un écouteur `input` qui compare | changé : à la lettre près ; un texte ne se compare pas à un nombre (`ADR-063`) |
| `State(price: 12.50)`, `{price}` | `Number`, `toFixed(2)`, `Intl.NumberFormat` | changé : exact (gardé à l'échelle), les chiffres fixés à la déclaration, la virgule de la langue d'office (`ADR-066`) |
| `Repeat(over: tasks, key: id)` | la `key` de React, `:key` de Vue | repris ; sans clé, le moteur prend le contenu, et le clavier est gardé quand même (`ADR-065`) |
| `total:` dans `Filter` | `filtered.length` avant `slice` | changé : un réglage, qui se montre et se compare (`ADR-065`) |
| `Data(name: Shop)`, `Shop.done`, `Shop.failed`, `Shop.refresh` | `fetch().then().catch()`, `response.ok`, `AbortController`, un bouton qui rappelle `fetch` | changé : un échec couvre l'erreur du serveur, le fichier trop gros ou illisible, et 10 secondes ; relectures espacées d'office (`ADR-064`) |
| `Sound` | `audio`, `new Audio().play()` | changé : un bruit qu'une règle déclenche, pas un lecteur |
| `List` | `ul`, `ol`, `li` | changé : un bloc au lieu de trois balises |
| `Term` dans `List` | `dl`, `dt`, `dd` | changé : une liste dont les éléments sont des termes ; un terme porte sa définition, ils ne se séparent pas (`ADR-097`) |
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
| `computer:` dans un style | `@media (min-width: 1024px)` | changé : un état du style, comme `phone:` (`ADR-069`) |
| `narrow:` dans un style | `@container (max-width: …)`, avec `container-type` déclaré à la main | changé : la case de `Grid`, ou la part d'un `Row` ou d'un `Column` (`grow:`, une largeur en %), fait moins de 320px ; rien à déclarer, la page mesure chaque case (`ADR-069`, `ADR-090`) |
| `Font(family:, source:)` et `fonts:` | `@font-face` | changé : déclaré sur la page ; toujours `font-display: swap` |
| `Font(family: "Inter")`, sans fichier | Google Fonts (`<link href="https://fonts.googleapis.com/…">`) | changé : une police du moteur, gardée dans le projet, sans service extérieur (`ADR-092`) |
| `url("fond.jpg")` dans `background` | `background-image`, `background-size`, `background-repeat` | changé : l'image couvre toujours le bloc |
| `State(tasks: [])`, `push`, `remove(item)`, `clear` | un tableau JavaScript, `push`, `splice`, `length = 0` | changé : une valeur de la page, changée par des demandes ; on retire la ligne touchée |
| `Repeat(over: tasks)` | `map` qui fabrique du HTML à chaque changement | changé : le moteur fabrique les lignes, et échappe ce que le visiteur a écrit |
| `text.set("")` | `input.value = ""` | changé : une demande |
| `module "…"`, `Module`, `run`, `done`, `failed` | `<script src>`, `new Worker`, `WebAssembly.instantiate` | changé : annoncé en haut du fichier, enfermé (un fil à part, une mémoire plafonnée, rien d'autre), arrêté s'il dure trop ; il reçoit et rend des valeurs de la page en JSON, relu avec méfiance (`ADR-077`) |
| `bridge js`, `bridge css` | `<script>`, `<link>` vers du code existant | refusés (`ADR-011`, partie B) |
| `mul`, `div` | `*=`, `/=`, `Math.floor` | changé : deux demandes, en nombres entiers |
| `computed:`, `Filter(name:, from:)` | `array.filter()`, `array.sort()`, `array.slice()` | changé : une liste nommée, refaite seule à chaque changement (`ADR-062`) |
| `contains:`, `in:` dans `Filter` | `string.includes()`, `toLowerCase()`, `normalize()` | changé : sans majuscules ni accents, d'office |
| `field:`, `is:`, `sortBy:`, `reverse:`, `limit:` dans `Filter` | `filter`, `sort` et `localeCompare`, `reverse`, `slice` | changé : des réglages ; `field`, `is`, `reverse` (`ADR-062`) |
| `offset:` dans `Filter` | `OFFSET` (SQL), `slice(start)`, la page d'un `Paginator` (Django) | repris : le mot usuel du décalage ; après la recherche et le tri, avant `limit` ; le total compte avant les deux (`ADR-084`) |
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
| `today`, `{arrival:date}`, `due.add(7)`, `Days(name:, from:, to:)` | `new Date()`, `toLocaleDateString`, `setDate`, une différence de millisecondes | changé : un jour du calendrier, sans heure ni fuseau ; exact (`ADR-067`) |
| `min:` dans `Input` | `min`, `minlength` | repris : pour un nombre, une date (`ADR-067`), et la longueur d'un texte (`ADR-068`) |
| `required: true`, `type: email` | `required`, `type="email"`, `setCustomValidity`, `aria-invalid` | changé : vérifié par le moteur, dans la page puis au serveur ; messages sous le champ, lus par un lecteur d'écran (`ADR-068`) |
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
| `Shared`, `shared:` | une base et une API côté serveur, puis `fetch`, un `WebSocket` ou un `EventSource` côté page (Firebase, Meteor, Phoenix LiveView) | nouveau : une déclaration ; le serveur de l'auteur garde la valeur pour tous, l'arbitre avec le même moteur que la page, et l'envoie en direct (`ADR-079`) ; une liste aussi, dont la ligne touchée se désigne par sa clé (`ADR-080`) |
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
| `address:` (sur `Page`) | `history.pushState`, `URLSearchParams`, `popstate` | changé : une liste de noms ; un toucher fait un pas d'historique, le serveur lit les mêmes valeurs (`ADR-091`) |
| `access: members` (et `everyone`, qu'on n'écrit pas) | `@login_required` (Django), `before_action :authenticate_user!` (Rails), un « middleware » (Next.js) | nouveau dans la page : ce qui la réserve est écrit sur elle, pas dans un programme à part (`ADR-081`) |
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
| `Zoom(detach:)` | aucun : sur le web, le zoom du navigateur grossit toujours la page sur place | nouveau : le bouton « Décrocher » du menu ☰ ; la page se détache comme une feuille, et le zoom l'approche ; « Accrocher » la remet à sa place (`ADR-069`) |
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
| `min-width`, `min-height`, `max-height` | les mêmes | repris ; en px ou en %, écrits en `rem` ; `max-width` sur `Page` élargit la page (`ADR-069`) |
| `overflow`, `overflow-x`, `overflow-y`, `white-space` | les mêmes | repris ; `white-space` : `normal`, `nowrap`, `pre-line`, `pre-wrap` (`pre` refusé : il déborde sur un téléphone) ; un mot trop long passe à la ligne sans rien écrire (`ADR-069`) |
| `line-clamp` | `-webkit-line-clamp`, avec `display: -webkit-box`, `-webkit-box-orient: vertical` et `overflow: hidden` | changé : un réglage au lieu de quatre (`ADR-069`) |
| `aspect-ratio`, `object-fit`, `object-position` | les mêmes | repris ; une image est `cover` sans rien écrire : jamais déformée (`ADR-069`) |
| `cursor` | `cursor` | repris : 22 formes, ou `url("viseur.svg")` ; le moteur ajoute la forme de secours `auto` (`ADR-069`) |
| `justify` dans `text-align` | `text-align: justify`, `hyphens: auto` | repris ; les mots se coupent seuls, dans la langue de la page (`ADR-069`) |

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
| `{id}` dans le nom d'un fichier, `profil/{id}.holo` | `[id]` (Next.js, SvelteKit), `:id` (Express) | changé : les accolades de HoloCode, les mêmes que dans un texte ; la page lit `{id}`, sans pouvoir le changer (`ADR-078`) |
| `signedIn`, `{account}` | `request.user.is_authenticated` et `{{ user.username }}` (Django), `user_signed_in?` et `current_user` (Rails), `useSession()` (Auth.js) | changés : deux valeurs que le serveur donne, lues comme les autres, jamais changées par la page (`ADR-081`) |
| `/account`, `/account/signin`, `/account/signup` | `/accounts/login/` (Django), `/users/sign_in` (Devise), `/api/auth/signin` (Auth.js) | repris : les pages de compte du serveur, fabriquées par le moteur ; les seuls liens qui partent de la racine du site (`ADR-081`) |

## 2. Les mots du web qu'on n'a pas pris

### Refusés exprès

| Web | Pourquoi |
|---|---|
| `div` | Un bloc dit ce qu'il est. |
| `h4`, `h5`, `h6` | Trois niveaux de titres suffisent. |
| `ul`, `ol`, `li` | Un seul bloc, `List`. |
| `section`, `article` | Les titres donnent le plan. |
| `script`, `onclick`, le DOM modifié à la main | Pas de code libre dans un bloc. |
| `display`, `position`, `float` dans un style | La disposition vient des blocs. Seule exception : `display: none` dans `phone:`, `computer:` ou `narrow:`, pour cacher. |
| Sélecteurs composés, cascade, `!important` | Un style vise un type ou un nom, rien d'autre. |
| `justify-content`, `align-items`, `flex-wrap` | Le moteur s'en charge ; un seul mot, `align`. |
| `@media`, `@container` écrits à la main | Des états du style : `phone:`, `computer:`, `narrow:`, `dark:`. |
| `text-overflow` | Seul, il ne fait rien : `line-clamp: 1` fait le tout. |
| `ew-resize`, `col-resize` et les autres curseurs d'étirement | Rien à étirer sans disposition à la main. |
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

## Comptes et partage, la suite : aucun mot nouveau (`ADR-080`, `ADR-082`, `ADR-083`, décidées le 2026-10-09)

- **Une liste partagée** (`ADR-080`) : `Shared(groceries: [ Item(what: "Du pain", done: 0) ])`, ou des textes, `Shared(names: [])` ; `push`, `remove(item)`, `item.done.set(1)` et `clear()` gardent le sens qu'ils ont pour une liste à soi (`ADR-044`, `ADR-057`). La ligne touchée se désigne par la clé qu'elle a déjà dans la page (`data-key`, `ADR-065`) : aucun mot à écrire. Leçon 102.
- **Un texte partagé confirmé** (`ADR-080`) : `Input(value: title)`, puis `On(Save.tap, effect: title.set(title))` ; aucun mot nouveau. Leçon 103.
- **Les clés d'accès** (`ADR-082`) : aucun mot de HoloCode ; `/account/passkeys` est une page du serveur, comme les autres pages de compte. `HOLO_ORIGIN` est un réglage du serveur, chez l'auteur (son adresse HTTPS), pas du langage. Leçon 107.
- **Le QR, les codes de secours, l'effacement** (`ADR-083`) : aucun mot de HoloCode ; `/account/code/setup`, `/account/code` et `/account/delete` sont des pages du serveur, et les champs de leurs formulaires ne deviennent pas des réglages du langage. Leçon 108.
