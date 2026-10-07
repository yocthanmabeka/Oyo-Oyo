# HoloCode à l'épreuve du web classique

- État au 2026-10-03. Tenu par Claude ; à mettre à jour à chaque ajout au langage.
- But : dire, balise par balise, ce que HoloCode sait déjà faire, ce qu'il refuse exprès, et ce qui lui manque. Sans embellir.
- Pour apprendre à écrire : [`GUIDE.md`](GUIDE.md).

Légende : **fait** (un mot existe et le moteur l'affiche) ; **exprès** (absent par décision) ; **manque** (rien n'existe).

## 1. HTML, balise par balise

### La structure d'une page

| HTML | HoloCode | État |
|---|---|---|
| `html`, `head`, `body`, `main` | `Page(...)` | fait |
| `title` | `Page(title:)` | fait |
| `meta charset` | toujours UTF-8 | fait |
| `meta viewport` (le zoom) | `Zoom(active:, max:, shrink:)` | fait |
| `meta description`, image de partage | `Page(description:, image:)` | fait |
| `html lang` | `Page(lang:)` | fait |
| `header`, `footer`, `main` | `Header`, `Footer`, `Main` | fait |
| `nav` | `Nav` | fait |
| `aside` | | manque |
| `section`, `article` | le titre suffit : `H1`, `H2`, `H3` donnent le plan | exprès |
| `div` | refusé : un bloc dit ce qu'il est (`ADR-009`) | exprès |
| `span` | `Text` | fait |

### Le texte

| HTML | HoloCode | État |
|---|---|---|
| `h1`, `h2`, `h3` | `H1`, `H2`, `H3` | fait |
| `h4`, `h5`, `h6` | `H4`, `H5`, `H6` (correction d'`ADR-020`, 2026-10-06) | fait |
| `p` | `P`, ou une phrase nue | fait |
| `strong`, `b` | `**gras**` dans un texte | fait |
| `em`, `i` | `*italique*` dans un texte | fait |
| `br` (retour à la ligne) | un texte entre trois guillemets garde ses retours à la ligne | fait |
| `hr` (trait de séparation) | `Hr()` | fait |
| `u`, `s`, `mark`, `small`, `sub`, `sup` | | manque |
| `blockquote`, `q`, `cite` (citations) | `Quote("…", by: "…")` | fait |
| `pre`, `code`, `kbd` (code, texte tel quel) | `Code("…")`, et les accents graves dans une phrase | fait |
| `abbr`, `time`, `address` | | manque |

### Les listes et les liens

| HTML | HoloCode | État |
|---|---|---|
| `ul`, `li` | `List(children: [...])` | fait |
| `ol` | `List(ordered: true, ...)` | fait |
| `dl`, `dt`, `dd` (liste de définitions) | | manque |
| `a href` | `A("texte", to: "adresse")` | fait |
| `a` vers un endroit de la même page | `A(to: "#Site")` vers un site de la page seulement | en partie |
| `a target`, `download` | | manque |

### Les images et les médias

| HTML | HoloCode | État |
|---|---|---|
| `img` | `Image(source:, weight:)` | fait |
| `img alt` (le texte pour qui ne voit pas l'image) | `Image(alt:)`, facultatif | fait |
| `picture`, `source`, `srcset` (plusieurs tailles) | | manque |
| `figure`, `figcaption` (image et légende) | | manque |
| `video` | `Video(source:, label:)`, sans lecture automatique | fait |
| `audio` | `Sound(name:, source:, volume:, loop:)`, `Ding.play`, `Ding.stop` : un bruit déclenché par une règle ; pas un lecteur | fait (`ADR-061` : volume, boucle, arrêt) |
| `canvas`, WebGL | `Point`, `World` | fait |
| `aria-*`, `role`, `aria-live` | rien à écrire : les repères, les étiquettes obligatoires, et la vue points lue au lecteur d'écran (la page reste dessous, « Vue points » annoncé) | fait (`ADR-061`) |
| `svg` | comme fichier d'image seulement | en partie |
| `iframe`, `embed`, `object` | `Point(inside: "fichier.holo")` : on y entre | fait |

### Les tableaux

| HTML | HoloCode | État |
|---|---|---|
| `table`, `tr`, `td`, `th`, `thead`, `tbody`, `caption` | `Table(caption:, head:, rows:)` | fait |

### Les formulaires

| HTML | HoloCode | État |
|---|---|---|
| `button` | `Button(name:, text:)` | fait |
| `form` | | manque |
| `input` : nombre, case à cocher | `Input(value:, label:, max:)`, `Checkbox(value:, label:)` | fait |
| `input` : texte | `Input(value: buyer, …)` quand la valeur est un texte | fait |
| `input` : date, bouton radio, fichier… | | manque |
| `label` | le réglage `label:`, obligatoire | fait |
| `textarea` | `Input(…, lines: 5)` | fait |
| `select`, `option`, `input radio` | `Choice(value:, label:, options:)`, `menu: true` | fait |
| `form` (envoyer) | `Form(name: Contact, …)`, `On(Send.tap, effect: Contact.send)`, `Contact.sent`, `Contact.failed` | fait : vers un fichier du serveur local |
| `input range` | `Slider(value:, label:, min:, max:)` | fait |
| `input date`, `time`, `color` | `Input(…, type: date)` | fait |
| `progress` | `Progress(value:, max:, label:)` | fait |
| `details`, `summary` | `Details(summary:, children:)` | fait |
| `dialog` | `Dialog(name:)`, `open`, `close` | fait |
| `figure`, `figcaption`, `picture` | `Image(caption:, phone:)` | fait |
| `audio controls` | `Sound(source:, label:)` | fait |
| `link rel="icon"` | `Page(icon:)` | fait |
| `s`, `mark`, `sup`, `sub` | `~~…~~`, `==…==`, `^…^`, `~…~` dans un texte | fait |
| `a href="#…"` vers un endroit de la page | `A(to: "#Horaires")` vers un bloc nommé ; refusé s'il n'existe pas | fait |
| `fieldset`, `legend`, `datalist`, `output`, `progress`, `meter` | | manque |

### Ce qui s'ouvre et se ferme

| HTML | HoloCode | État |
|---|---|---|
| `details`, `summary` | | manque |
| `dialog` | | manque |

### Le code

| HTML | HoloCode | État |
|---|---|---|
| `style` | les styles, après la page | fait |
| `link rel="stylesheet"` | les styles d'un morceau importé : `import "commun.holo"` | fait |
| `script` | refusé dans un bloc (`ADR-015`) ; du code venu d'ailleurs passe par un module enfermé, `module "calcul.wasm"` (`ADR-045`) | exprès |
| `noscript` | sans objet : rien ne dépend d'un script | exprès |
| `template`, `slot` (morceaux réutilisables) | `Part(name: Menu, …)` et `Use(Menu)` ; un modèle répété avec ses champs : `Repeat` | fait |

## 2. CSS

| CSS | HoloCode | État |
|---|---|---|
| Couleurs, fond, police, taille, graisse, italique, alignement | 24 réglages (voir le guide) | fait |
| `line-height`, `letter-spacing`, `text-transform`, `text-decoration` | les mêmes ; `line-height` sans unité | fait |
| Bordure, coins arrondis, marges, largeur, hauteur, opacité | idem | fait |
| Sélecteur par balise, par classe | `P { }`, `.card { }` | fait |
| Sélecteurs composés, cascade, `!important` | refusés (`ADR-017`) | exprès |
| `display`, `position`, `float` | refusés dans un style : la disposition vient des blocs | exprès |
| La disposition elle-même : `flex`, `grid`, colonnes | `Row`, `Column`, `Grid`, avec `gap`, `align`, `columns` | fait |
| `:hover`, `:focus`, `:active` (l'apparence selon l'état) | `hover: { … }`, `focus: { … }`, `active: { … }` dans un style | fait |
| `position: absolute` pour un badge, une pastille | `Stack(children: [ … ])` et `align:` | fait |
| Tailles de texte qui suivent le réglage du visiteur (`rem`) | automatique : les `px` d'une taille de texte deviennent des `rem` ; les grands titres rétrécissent sur un petit écran | fait |
| Autres tailles qui suivent le visiteur (`rem`) ; toute la hauteur de l'écran (`100vh`, `100dvh`) | automatique : les `px` des marges, largeurs, hauteurs, coins et écarts deviennent des `rem` ; `height: screen` | fait (`ADR-061`) |
| `transition`, `animation`, `@keyframes` | `enter: Enter(…)`, `loop: Loop(…)`, `Scenes` : d'où il part ou où il va, quand, combien de temps, quel caractère ; lettre à lettre et enfant après enfant sans JavaScript | fait |
| `@media` (s'adapter à l'écran) | le moteur le fait seul (une ligne passe à la ligne, une grille perd des colonnes) ; et `phone: { … }`, avec `display: none` pour cacher | fait |
| `prefers-color-scheme` (le thème sombre) | `dark: { … }` dans un style | fait |
| Variables (`--couleur`) | `Page { --or: #E9B44C; }`, puis `color: --or;` | fait |
| Dégradés, ombres, images de fond | `linear-gradient`, `radial-gradient`, `box-shadow`, `text-shadow`, `url("fond.jpg")` | fait |
| `@font-face` (charger une police) | `fonts: [ Font(family:, source:) ]` | fait |
| `transition`, `transform` 2D en pose fixe | `transition: 0.3s`, `rotate`, `scale` | fait |
| `transform` 3D | `Relief(height:, tilt:)` | fait, autrement |

## 3. JavaScript

| Ce qu'on fait en JavaScript | HoloCode | État |
|---|---|---|
| Réagir à un clic | `On(Open.tap, effect: ...)` | fait |
| Changer de page sans recharger (un routeur) | `Point(inside:)`, `enter`, `leave`, le dézoom | fait |
| L'historique, le bouton « retour » | automatique : chaque site a son adresse | fait |
| Le clavier | `On(Key.left, effect: …)` : les flèches, l'espace, Entrée, Échap, les lettres, les chiffres ; jamais Tab ; les touches à une lettre se coupent dans le menu | fait (`ADR-061`) |
| Survol (`mouseenter`, `mouseleave`) | `On(Carte.hover, …)`, `On(Carte.hoverEnd, …)` : aussi au clavier et au doigt | fait |
| Chercher, filtrer, trier, montrer plus (`filter`, `sort`, `slice`) | `computed: [ Filter(name: found, from: articles, contains: search, sortBy: price, limit: shown) ]`, `Repeat(over: found, empty: "…")` | fait (`ADR-062`) |
| Défilement : apparaître quand on arrive dessus | `Enter(…, inView: true)` | fait (`ADR-061`) |
| Approche, position du défilement | | manque |
| `else` | `If(…, children: [ … ], else: [ … ])` | fait |
| `setTimeout` | `After(3s, effect: …)` ; sous une condition, part quand elle devient vraie | fait |
| `Date` (la date et l'heure du jour) | `{year}`, `{month}`, `{day}`, `{weekday}`, `{hour}`, `{minute}` | fait ; pas encore de calcul sur les dates |
| Garder une valeur, l'afficher (un panier) | `State(cart: 0)`, `{cart}`, `cart.add(1)` | fait : des nombres entiers ; avec `Prices`, le moteur calcule `{count}` et `{total}` |
| Afficher sous condition | `If(cart, is: 0, children: [...])` | fait |
| Des nombres à virgule (`Number`, `toFixed`, `Intl.NumberFormat`) | `State(price: 12.50)`, `{price}`, `sum.mul(1.1)`, `If(sum, over: 49.99)` | fait (`ADR-066`, à valider) : exacts ; pas encore de nombre négatif |
| Une clé stable par ligne (`key` de React), le focus gardé quand la liste change | `Repeat(over: tasks, key: id, …)` ; le clavier suit la ligne, rien à écrire | fait (`ADR-065`, à valider) |
| Pagination : le total avant de couper, « 4 sur 6 » | `Filter(…, limit: shown, total: matching)`, `{matching}`, `If(shown, under: matching, …)` | fait (`ADR-065`, à valider) |
| Dire « chargement » et « échec », réessayer (`fetch`, `response.ok`, `AbortController`) | `Data(name: Shop, …)`, `On(Shop.done, …)`, `On(Shop.failed, …)`, `Shop.refresh` | fait (`ADR-064`, à valider) ; 10 secondes au plus ; une lecture à la fois |
| Comparer des textes (`size === "L"`, `a !== b`) | `If(size, is: "L")`, `If(again, not: email)`, `When(answer, is: "Paris", effect: …)` | fait (`ADR-063`, à valider) ; à la lettre près ; plus grand et plus petit : pour les nombres |
| Répéter sur une liste (`for`, `map`) | `Repeat(items: [ Item(…) ], children: [ … ])` : une liste écrite dans le fichier ; `Repeat(over: tasks, …)` : une liste qui change pendant la visite | fait  |
| Un tableau qu'on remplit (`push`, `splice`) | `State(tasks: [])`, `tasks.push(task)`, `tasks.remove(item)`, `tasks.clear()` | fait : des textes |
| Chercher des données (`fetch`) | `data: Data(from: "stock.json", every: 30s)` : des valeurs, du même serveur | fait ; pas de liste, pas d'envoi |
| Durées, minuteries | `Every(1s, effect: …)` ; `Portals(duration:)` | fait |
| Le hasard (`Math.random`) | la demande `random`, rejouable | fait |
| Multiplier, diviser | les demandes `mul`, `div`, en nombres entiers | fait |
| `Intl`, `padStart` (formats) | `{n:00}`, `{n:number}`, `{n:cents}`, `{weekday:name}`, `{month:name}` | fait |
| Animations écrites par l'auteur | | manque |
| Garder des données dans le navigateur | `keep: [cart]` | fait |
| Calculer librement | les modules WebAssembly enfermés (`ADR-045`) | fait : un nombre contre un nombre |
| Modifier la page à la main (le DOM) | refusé (`ADR-015`) | exprès |

## 4. Ce que HoloCode a, et que le web classique n'a pas

| HoloCode | Ce que c'est |
|---|---|
| `Point`, `World`, `seed` | Un monde entier dans un nombre ; rien à télécharger |
| `fragments` | Un point qui se morcelle en mondes, sans fin |
| `pixels:` | Un site planté dans un pixel d'une page |
| `Points(after:, size:, fragment:, divisions:, levels:)` | Les pixels d'une page qui deviennent des points quand on zoome |
| `Relief(height:, tilt:)` | La page vue de biais, avec du relief |
| `Portals(layout:, count:, size:, brightness:)` | Le carrefour : des portails vers les mondes voisins |
| `Zoom(levels:)`, `budget`, `weight` | Des garde-fous écrits par l'auteur, vérifiés avant d'afficher |
| Le vérificateur | Une faute est refusée avec sa ligne, au lieu d'être ignorée en silence |
| Une adresse par site emboîté | `fichier.holo#Site/Site`, sans rien écrire |

## 5. Le verdict, sans détour

- **Ce qu'on peut faire aujourd'hui** : une page de présentation, un blog simple, une vitrine, avec des images, des listes, des liens, et des mondes où l'on entre. Tout ce qui est « lire et se promener ».
- **Ce qu'on ne peut pas faire** : tout ce qui est « agir ». Pas de formulaire, pas de panier, pas de recherche, pas de compte. Pas de tableau. Pas de son ni de vidéo. Et pas de mise en page : tout est l'un sous l'autre.
- **Par rapport à HTML seul** (sans CSS ni JavaScript), HoloCode couvre le cœur : titres, paragraphes, listes, liens, images, boutons. Il lui manque les tableaux, les formulaires, les médias, et une vingtaine de balises de texte.
- **Par rapport à un site moderne**, le manque le plus gênant est la disposition : sans elle, aucun site ne ressemble à un vrai site.

## 6. L'ordre que je propose pour combler les manques

L'avis de Codex (revue du 2026-10-03) : mettre avant la disposition la sécurité des passages, puis l'accessibilité. La sécurité est faite (2026-10-04). Pour l'accessibilité, il a raison sur le fond : le rang 3 ci-dessous devrait sans doute passer en premier. À Yocthan de trancher l'ordre.

| Rang | Quoi | Pourquoi d'abord |
|---|---|---|
| 1 | La disposition : `Row`, `Column`, `Grid` (fait le 2026-10-04) | Sans elle, pas de vrai site. Yocthan connaît ces mots (Flutter). |
| 2 | Le texte qui manque : `Br`, `Hr`, citation, code | Petit, et l'on en a besoin partout |
| 3 | Le texte de remplacement d'une image (`alt`) | Accessibilité : une image sans texte est invisible pour un aveugle |
| 4 | L'état et les formulaires : `State` (fait le 2026-10-04), `Input`, `Form` | Le passage de « lire » à « agir » |
| 5 | Le survol et l'approche : un signal `near` | Le même signal à plat et en profondeur |
| 6 | Le son et la vidéo : `Audio`, `Video` | Les médias ; le son doit se placer dans l'espace |
| 7 | Les tableaux : `Table` | Utile, mais moins urgent |
| 8 | Les transitions et les durées | Fait : `Enter`, `Loop`, `Scenes` (`ADR-034`) |
| 9 | Les imports : réutiliser un morceau, un fichier de styles | Pour les sites de plus d'une page |
| 10 | Le deuxième étage : calcul libre en module enfermé | Le plus gros chantier ; il ouvre les applications et les jeux |
