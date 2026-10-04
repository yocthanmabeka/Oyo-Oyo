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
| `meta description`, mots-clés, image de partage | | manque |
| `header`, `footer`, `aside` | | manque |
| `nav` | | manque |
| `section`, `article` | le titre suffit : `H1`, `H2`, `H3` donnent le plan | exprès |
| `div` | refusé : un bloc dit ce qu'il est (`ADR-009`) | exprès |
| `span` | `Text` | fait |

### Le texte

| HTML | HoloCode | État |
|---|---|---|
| `h1`, `h2`, `h3` | `H1`, `H2`, `H3` | fait |
| `h4`, `h5`, `h6` | refusés : trois niveaux (`ADR-020`) | exprès |
| `p` | `P`, ou une phrase nue | fait |
| `strong`, `b` | `**gras**` dans un texte | fait |
| `em`, `i` | `*italique*` dans un texte | fait |
| `br` (retour à la ligne) | | manque |
| `hr` (trait de séparation) | | manque |
| `u`, `s`, `mark`, `small`, `sub`, `sup` | | manque |
| `blockquote`, `q`, `cite` (citations) | | manque |
| `pre`, `code`, `kbd` (code, texte tel quel) | | manque |
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
| `img alt` (le texte pour qui ne voit pas l'image) | | manque |
| `picture`, `source`, `srcset` (plusieurs tailles) | | manque |
| `figure`, `figcaption` (image et légende) | | manque |
| `video` | | manque |
| `audio` | | manque |
| `canvas`, WebGL | `Point`, `World` | fait |
| `svg` | comme fichier d'image seulement | en partie |
| `iframe`, `embed`, `object` | `Point(inside: "fichier.holo")` : on y entre | fait |

### Les tableaux

| HTML | HoloCode | État |
|---|---|---|
| `table`, `tr`, `td`, `th`, `thead`, `tbody`, `caption` | | manque |

### Les formulaires

| HTML | HoloCode | État |
|---|---|---|
| `button` | `Button(name:, text:)` | fait |
| `form` | | manque |
| `input` (texte, nombre, date, case, bouton radio, fichier…) | | manque |
| `textarea`, `select`, `option`, `label` | | manque |
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
| `link rel="stylesheet"` | un fichier de styles à part | manque |
| `script` | refusé dans un bloc (`ADR-015`) | exprès |
| `noscript` | sans objet : rien ne dépend d'un script | exprès |
| `template`, `slot` (morceaux réutilisables) | `import` est lu, pas appliqué | manque |

## 2. CSS

| CSS | HoloCode | État |
|---|---|---|
| Couleurs, fond, police, taille, graisse, italique, alignement | 15 réglages (voir le guide) | fait |
| Bordure, coins arrondis, marges, largeur, hauteur, opacité | idem | fait |
| Sélecteur par balise, par classe | `P { }`, `.card { }` | fait |
| Sélecteurs composés, cascade, `!important` | refusés (`ADR-017`) | exprès |
| `display`, `position`, `float` | refusés dans un style : la disposition vient des blocs | exprès |
| La disposition elle-même : `flex`, `grid`, colonnes | `Row`, `Column`, `Grid`, avec `gap`, `align`, `columns` | fait, à l'essai |
| `:hover`, `:focus`, `:active` (l'apparence selon l'état) | | manque |
| `transition`, `animation`, `@keyframes` | | manque |
| `@media` (s'adapter à l'écran) | le moteur le fait seul : une ligne passe à la ligne, une grille perd des colonnes | en partie |
| Variables (`--couleur`) | | manque |
| Dégradés, ombres, images de fond | | manque |
| `@font-face` (charger une police) | | manque |
| `transform` 3D | `Relief(height:, tilt:)` | fait, autrement |

## 3. JavaScript

| Ce qu'on fait en JavaScript | HoloCode | État |
|---|---|---|
| Réagir à un clic | `On(Open.tap, effect: ...)` | fait |
| Changer de page sans recharger (un routeur) | `Point(inside:)`, `enter`, `leave`, le dézoom | fait |
| L'historique, le bouton « retour » | automatique : chaque site a son adresse | fait |
| Survol, approche, défilement, clavier | | manque |
| Garder une valeur, l'afficher (un panier) | `State(cart: 0)`, `{cart}`, `cart.add(1)` | fait, à l'essai : des nombres entiers seulement |
| Afficher sous condition, répéter sur une liste | | manque |
| Chercher des données (`fetch`) | | manque |
| Durées, minuteries, animations | les unités `ms`, `s` existent, sans emploi | manque |
| Garder des données dans le navigateur | | manque |
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
| 8 | Les transitions et les durées | Les unités `ms` et `s` attendent leur emploi |
| 9 | Les imports : réutiliser un morceau, un fichier de styles | Pour les sites de plus d'une page |
| 10 | Le deuxième étage : calcul libre en module enfermé | Le plus gros chantier ; il ouvre les applications et les jeux |
