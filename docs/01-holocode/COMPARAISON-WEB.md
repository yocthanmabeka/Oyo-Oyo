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
| `meta description`, image de partage | `Page(description:, image:)` | fait, à l'essai |
| `html lang` | `Page(lang:)` | fait, à l'essai |
| `header`, `footer`, `main` | `Header`, `Footer`, `Main` | fait, à l'essai |
| `nav` | `Nav` | fait, à l'essai |
| `aside` | | manque |
| `section`, `article` | le titre suffit : `H1`, `H2`, `H3` donnent le plan | exprès |
| `div` | refusé : un bloc dit ce qu'il est (`ADR-009`) | exprès |
| `span` | `Text` | fait |

### Le texte

| HTML | HoloCode | État |
|---|---|---|
| `h1`, `h2`, `h3` | `H1`, `H2`, `H3` | fait |
| `h4`, `h5`, `h6` | `H4`, `H5`, `H6` (correction d'`ADR-020`, 2026-10-06) | fait, à l'essai |
| `p` | `P`, ou une phrase nue | fait |
| `strong`, `b` | `**gras**` dans un texte | fait |
| `em`, `i` | `*italique*` dans un texte | fait |
| `br` (retour à la ligne) | un texte entre trois guillemets garde ses retours à la ligne | fait, à l'essai |
| `hr` (trait de séparation) | `Hr()` | fait, à l'essai |
| `u`, `s`, `mark`, `small`, `sub`, `sup` | | manque |
| `blockquote`, `q`, `cite` (citations) | `Quote("…", by: "…")` | fait, à l'essai |
| `pre`, `code`, `kbd` (code, texte tel quel) | `Code("…")`, et les accents graves dans une phrase | fait, à l'essai |
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
| `img alt` (le texte pour qui ne voit pas l'image) | `Image(alt:)`, facultatif | fait, à l'essai |
| `picture`, `source`, `srcset` (plusieurs tailles) | | manque |
| `figure`, `figcaption` (image et légende) | | manque |
| `video` | `Video(source:, label:)`, sans lecture automatique | fait, à l'essai |
| `audio` | `Sound(name:, source:)` et `Ding.play` : un bruit déclenché par une règle ; pas un lecteur | fait, à l'essai |
| `canvas`, WebGL | `Point`, `World` | fait |
| `svg` | comme fichier d'image seulement | en partie |
| `iframe`, `embed`, `object` | `Point(inside: "fichier.holo")` : on y entre | fait |

### Les tableaux

| HTML | HoloCode | État |
|---|---|---|
| `table`, `tr`, `td`, `th`, `thead`, `tbody`, `caption` | `Table(caption:, head:, rows:)` | fait, à l'essai |

### Les formulaires

| HTML | HoloCode | État |
|---|---|---|
| `button` | `Button(name:, text:)` | fait |
| `form` | | manque |
| `input` : nombre, case à cocher | `Input(value:, label:, max:)`, `Checkbox(value:, label:)` | fait, à l'essai |
| `input` : texte | `Input(value: buyer, …)` quand la valeur est un texte | fait, à l'essai |
| `input` : date, bouton radio, fichier… | | manque |
| `label` | le réglage `label:`, obligatoire | fait, à l'essai |
| `textarea` | `Input(…, lines: 5)` | fait, à l'essai |
| `select`, `option`, `input radio` | `Choice(value:, label:, options:)`, `menu: true` | fait, à l'essai |
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
| `link rel="stylesheet"` | les styles d'un morceau importé : `import "commun.holo"` | fait, à l'essai |
| `script` | refusé dans un bloc (`ADR-015`) | exprès |
| `noscript` | sans objet : rien ne dépend d'un script | exprès |
| `template`, `slot` (morceaux réutilisables) | `Part(name: Menu, …)` et `Use(Menu)` ; un modèle répété avec ses champs : `Repeat` | fait, à l'essai |

## 2. CSS

| CSS | HoloCode | État |
|---|---|---|
| Couleurs, fond, police, taille, graisse, italique, alignement | 24 réglages (voir le guide) | fait |
| `line-height`, `letter-spacing`, `text-transform`, `text-decoration` | les mêmes ; `line-height` sans unité | fait, à l'essai |
| Bordure, coins arrondis, marges, largeur, hauteur, opacité | idem | fait |
| Sélecteur par balise, par classe | `P { }`, `.card { }` | fait |
| Sélecteurs composés, cascade, `!important` | refusés (`ADR-017`) | exprès |
| `display`, `position`, `float` | refusés dans un style : la disposition vient des blocs | exprès |
| La disposition elle-même : `flex`, `grid`, colonnes | `Row`, `Column`, `Grid`, avec `gap`, `align`, `columns` | fait, à l'essai |
| `:hover`, `:focus`, `:active` (l'apparence selon l'état) | `hover: { … }`, `focus: { … }`, `active: { … }` dans un style | fait, à l'essai |
| `position: absolute` pour un badge, une pastille | `Stack(children: [ … ])` et `align:` | fait, à l'essai |
| Tailles de texte qui suivent le réglage du visiteur (`rem`) | automatique : les `px` d'une taille de texte deviennent des `rem` ; les grands titres rétrécissent sur un petit écran | fait, à l'essai |
| `transition`, `animation`, `@keyframes` | `enter: Enter(…)`, `loop: Loop(…)`, `Scenes` : d'où il part ou où il va, quand, combien de temps, quel caractère ; lettre à lettre et enfant après enfant sans JavaScript | fait, à l'essai |
| `@media` (s'adapter à l'écran) | le moteur le fait seul (une ligne passe à la ligne, une grille perd des colonnes) ; et `phone: { … }`, avec `display: none` pour cacher | fait, à l'essai |
| `prefers-color-scheme` (le thème sombre) | `dark: { … }` dans un style | fait, à l'essai |
| Variables (`--couleur`) | `Page { --or: #E9B44C; }`, puis `color: --or;` | fait, à l'essai |
| Dégradés, ombres, images de fond | `linear-gradient`, `radial-gradient`, `box-shadow`, `text-shadow`, `url("fond.jpg")` | fait, à l'essai |
| `@font-face` (charger une police) | `fonts: [ Font(family:, source:) ]` | fait, à l'essai |
| `transition`, `transform` 2D en pose fixe | `transition: 0.3s`, `rotate`, `scale` | fait, à l'essai |
| `transform` 3D | `Relief(height:, tilt:)` | fait, autrement |

## 3. JavaScript

| Ce qu'on fait en JavaScript | HoloCode | État |
|---|---|---|
| Réagir à un clic | `On(Open.tap, effect: ...)` | fait |
| Changer de page sans recharger (un routeur) | `Point(inside:)`, `enter`, `leave`, le dézoom | fait |
| L'historique, le bouton « retour » | automatique : chaque site a son adresse | fait |
| Le clavier | `On(Key.left, effect: …)` : les flèches et l'espace | fait, à l'essai |
| Survol (`mouseenter`, `mouseleave`) | `On(Carte.hover, …)`, `On(Carte.hoverEnd, …)` : aussi au clavier et au doigt | fait, à l'essai |
| Approche, défilement | | manque |
| `else` | `If(…, children: [ … ], else: [ … ])` | fait, à l'essai |
| `setTimeout` | `After(3s, effect: …)` ; sous une condition, part quand elle devient vraie | fait, à l'essai |
| `Date` (la date et l'heure du jour) | `{year}`, `{month}`, `{day}`, `{weekday}`, `{hour}`, `{minute}` | fait, à l'essai ; pas encore de calcul sur les dates |
| Garder une valeur, l'afficher (un panier) | `State(cart: 0)`, `{cart}`, `cart.add(1)` | fait, à l'essai : des nombres entiers ; avec `Prices`, le moteur calcule `{count}` et `{total}` |
| Afficher sous condition | `If(cart, is: 0, children: [...])` | fait, à l'essai |
| Répéter sur une liste (`for`, `map`) | `Repeat(items: [ Item(…) ], children: [ … ])` : une liste écrite dans le fichier | fait, à l'essai ; une liste qui change pendant la visite reste à faire |
| Chercher des données (`fetch`) | `data: Data(from: "stock.json", every: 30s)` : des valeurs, du même serveur | fait, à l'essai ; pas de liste, pas d'envoi |
| Durées, minuteries | `Every(1s, effect: …)` ; `Portals(duration:)` | fait, à l'essai |
| Le hasard (`Math.random`) | la demande `random`, rejouable | fait, à l'essai |
| Animations écrites par l'auteur | | manque |
| Garder des données dans le navigateur | `keep: [cart]` | fait, à l'essai |
| Calculer librement | prévu : fonctions pures, modules WebAssembly (`ADR-013`) | manque |
| Modifier la page à la main (le DOM) | refusé (`ADR-015`) | exprès |

## 4. Ce que HoloCode a, et que le web classique n'a pas

| HoloCode | Ce que c'est |
|---|---|
| `Point`, `World`, `seed` | Un monde entier dans un nombre ; rien à télécharger |
| `fragments` | Un point qui se morcelle en mondes, sans fin |
| `pixels:` | Un site planté dans un pixel d'une page |
| `Points(after:, size:, fragment:, grid:, depth:)` | Les pixels d'une page qui deviennent des points quand on zoome |
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
| 1 | La disposition : `Row`, `Column`, `Grid` (fait le 2026-10-04, à l'essai) | Sans elle, pas de vrai site. Yocthan connaît ces mots (Flutter). |
| 2 | Le texte qui manque : `Br`, `Hr`, citation, code | Petit, et l'on en a besoin partout |
| 3 | Le texte de remplacement d'une image (`alt`) | Accessibilité : une image sans texte est invisible pour un aveugle |
| 4 | L'état et les formulaires : `State` (fait le 2026-10-04, à l'essai), `Input`, `Form` | Le passage de « lire » à « agir » |
| 5 | Le survol et l'approche : un signal `near` | Le même signal à plat et en profondeur |
| 6 | Le son et la vidéo : `Audio`, `Video` | Les médias ; le son doit se placer dans l'espace |
| 7 | Les tableaux : `Table` | Utile, mais moins urgent |
| 8 | Les transitions et les durées | Fait à l'essai : `Enter`, `Loop`, `Scenes` (`ADR-034`) |
| 9 | Les imports : réutiliser un morceau, un fichier de styles | Pour les sites de plus d'une page |
| 10 | Le deuxième étage : calcul libre en module enfermé | Le plus gros chantier ; il ouvre les applications et les jeux |
