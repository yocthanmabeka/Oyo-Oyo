# Écrire en `.holo` : le guide de l'auteur

Ce guide montre comment écrire un fichier `.holo` aujourd'hui, avec ce que le moteur sait vraiment faire. Tous les exemples entre balises `holo` sont relus par un test du moteur à chaque changement : s'ils cessaient de marcher, le test échouerait.

- État : langage en construction (2026-10-03). Ce qui n'existe pas encore est listé à la fin.
- Les décisions derrière chaque règle : [`docs/02-gouvernance/DECISIONS.md`](../02-gouvernance/DECISIONS.md).

## 1. Voir ce qu'on écrit

1. Lancer le serveur local, une fois, dans un terminal : `node moteur/outils/serveur.mjs`
2. Ranger son fichier dans `exemples/` (une page) ou dans `moteur/mondes/` (un monde seul).
3. Dans VS Code, avec l'extension HoloCode ([`outils/vscode-holocode/`](../../outils/vscode-holocode/README.md)) : ouvrir le fichier et cliquer sur ▶ en haut à droite (ou `Ctrl+Alt+H`). Le fichier s'ouvre dans Chrome, à sa propre adresse.

Sans l'extension : ouvrir `http://localhost:8080/exemples/mon-dossier/ma-page.holo` dans Chrome.

Si le fichier contient une erreur, la page affiche le message du moteur, avec la ligne et la colonne.

## 2. La première page

```holo
Page(
  title: "Hello",
  children: [
    "Hello World",
  ],
)
```

- `Page(...)` est un **bloc**. Un bloc commence par une majuscule et ouvre une parenthèse.
- `title:` et `children:` sont des **réglages**. Un réglage commence par une minuscule et finit par deux-points.
- `children` contient ce que la page montre, entre crochets, séparé par des virgules.
- Une phrase entre guillemets, seule, est un paragraphe.
- Un commentaire commence par `//` ; on y écrit ce qu'on veut, dans la langue qu'on veut.

Un fichier contient un seul bloc racine : une `Page`, ou un `Point`.

## 3. Le texte

```holo
Page(
  title: "My shop",
  children: [
    H1("My shop"),
    "A sentence alone is a paragraph.",
    P("So is this one, with **bold** and *italic*."),
    H2("The paintings"),
    H3("Today"),
    Text("Open until 6 pm"),
  ],
)
```

| Bloc | Ce que c'est |
|---|---|
| `H1`, `H2`, `H3` | Un titre. Le numéro dit sa place dans le plan, jamais sa taille. |
| `P`, ou une phrase nue | Un paragraphe. |
| `Text` | Du texte sans rôle : une étiquette, une ligne d'état. |

Règles :

- Le premier titre est `H1`. On ne saute pas de niveau : `H3` ne suit pas `H1`.
- La taille d'un titre se règle par le style, pas en changeant de numéro.
- Dans un texte, `**gras**` et `*italique*`.

## 4. Images, listes, boutons

```holo
Page(
  title: "My shop",
  children: [
    H1("My shop"),
    Image(source: "painting.svg", weight: 1KB),
    List(children: [
      "Sunrise over the river",
      "The blue door",
    ]),
    Button(name: Open, text: "Enter the workshop"),
  ],
)
```

- `Image(source:)` : un fichier rangé à côté du `.holo`. Ni adresse complète, ni `../`. `weight` est son poids déclaré.
- `List(children: [...])` : une liste.
- `Button(name:, text:)` : un bouton. `name` lui donne un nom, qui sert aux règles. Deux blocs ne portent pas le même nom.

### Les liens et les listes numérotées

```holo
Page(
  title: "My shop",
  children: [
    H1("My shop"),
    A("The garden", to: "garden.holo"),
    A("Somewhere else on the web", to: "https://example.com"),
    List(ordered: true, children: [
      A("First link", to: "one.holo"),
      A("Second link", to: "two.holo"),
    ]),
  ],
)
```

- `A("texte", to: "adresse")` est le lien classique, celui de HTML (`<a href>`) : on quitte la page pour une autre adresse. `to` accepte un fichier rangé à côté (`"garden.holo"`), un site de la page (`"#Workshop"`), ou une adresse du web en `http` ou `https`. Rien d'autre : pas de `javascript:`.
- `List(ordered: true, …)` numérote la liste. Un élément de liste peut être une phrase ou un bloc, par exemple un lien.
- Il n'y a ni `UL`, ni `OL`, ni `LI` : un seul bloc `List` suffit, et ses éléments n'ont pas besoin de balise.

## 5. Les styles

Les styles s'écrivent comme en CSS, **après** le bloc racine.

```holo
Page(
  title: "My shop",
  children: [
    H1("My shop"),
    P("Paintings made by hand."),
    P.card("Free delivery from 30 euros."),
  ],
)

Page { background: #101020; color: white; font-family: Georgia, serif; padding: 24px 16px; }

H1 { color: #E9B44C; font-size: 32px; text-align: center; }

.card {
  background: #1a1a2e;
  border: 1px solid #E9B44C;
  border-radius: 12px;
  padding: 8px 16px;
}
```

Un style vise deux choses, pas plus :

- un **type de bloc** : `P { ... }` touche tous les paragraphes ;
- un **nom à point** : `.card { ... }` touche les blocs marqués, qu'on marque en écrivant `P.card(...)`.

Le style de `Page` est le thème : son contenu le reprend. Priorité, du plus faible au plus fort : thème, type, nom.

Réglages connus :

| Réglage | Valeur |
|---|---|
| `color`, `background` | Une couleur : `white`, `gold`, `#E9B44C` |
| `font-size`, `border-radius`, `width`, `height`, `max-width` | Une taille : `16px`, `50%` |
| `padding`, `margin` | D'une à quatre tailles : `8px 16px` |
| `font-weight` | `normal`, `bold` |
| `font-style` | `normal`, `italic` |
| `font-family` | Un ou plusieurs noms de police |
| `text-align` | `left`, `center`, `right` |
| `border` | `1px solid gray` |
| `opacity` | Un nombre de 0 à 1 |

Ce que le moteur refuse, alors que le CSS le laisse passer :

- un réglage inconnu ou mal orthographié ;
- un `;` oublié en fin de ligne ;
- un style défini deux fois ;
- un nom posé sur un bloc (`P.card`) sans style `.card` ;
- un réglage de disposition (`display`, `position`, `float`) : un style ne dit que l'apparence ;
- un sélecteur composé (`.card P { ... }`).

## 6. Un point, son monde, et les règles

Un `Point` est le pixel de l'Holoverse : de loin, une lumière ; quand on y entre, un monde.

```holo
Page(
  title: "My shop",
  children: [
    H1("My shop"),
    Button(name: Open, text: "Enter the workshop"),
    Point(
      name: Workshop,
      seed: 42,
      brightness: 0.8,
      fragments: 6,
      color: "#E9B44C",
      palette: ["#E9B44C", "#245C45"],
      budget: 500KB,
      inside: World(
        children: [
          H1("The workshop"),
          Image(source: "easel.svg", weight: 1KB),
          Button(name: Back, text: "Back to the shop"),
        ],
        rules: [
          On(Back.tap, effect: Workshop.leave),
        ],
      ),
    ),
  ],
  rules: [
    On(Open.tap, effect: Workshop.enter),
  ],
)
```

| Réglage du `Point` | Sens |
|---|---|
| `name` | Son nom. |
| `seed` | Sa graine : un nombre entier. La même graine redonne toujours le même monde. |
| `brightness` | Sa lumière, de 0 à 1. |
| `fragments` | En combien de points il se morcelle, de 1 à 64. |
| `color` | Sa couleur. Sans elle, la graine décide. |
| `palette` | Les couleurs de ses enfants. |
| `budget` | Ce que son contenu a le droit de peser. Dépassé, le fichier est refusé. |
| `inside` | Le monde qu'il contient : `World(children: [...], rules: [...])`. |

**Passer dans un autre fichier.** Le monde d'un point peut être un autre fichier, rangé à côté :

```holo
Page(
  name: LivingRoom,
  title: "The living room",
  children: [
    H1("The living room"),
    Button(name: Out, text: "Go to the garden"),
    Point(name: Garden, seed: 12, color: "#3FA34D", inside: "garden.holo"),
  ],
  rules: [
    On(Out.tap, effect: Garden.enter),
  ],
)
```

- On y passe **sans changer de page** : le carrefour s'ouvre, le portail montre l'autre fichier, on le franchit par une animation. L'adresse de la barre devient celle de l'autre fichier, et le bouton « retour » ramène.
- Les fichiers où mènent les points d'une page sont lus d'avance (ils sont petits) : le passage est immédiat. Un fichier introuvable ou refusé par le moteur laisse le passage fermé.
- **Dézoomer** alors que la page est déjà à sa taille normale fait ressortir du monde où l'on est : on revient au site, ou au fichier, d'où l'on venait.
- La différence avec `A` : `A` fait changer de page, à l'ancienne ; un `Point` se traverse à pied.

L'exemple complet : [`exemples/maison/`](../../exemples/maison/salon.holo), un salon et un jardin.

**Les règles.** Un bloc ne contient jamais de code. Une règle relie un signal à une capacité :

- `On(Open.tap, effect: Workshop.enter)` : quand le bouton `Open` est touché, on entre dans le point `Workshop`.
- Signaux : `tap` (un `Button` ou un `Point` touché).
- Capacités : `enter` et `leave` (pour un `Point`), `portals` (pour la `Page` : ouvrir son carrefour).
- Toucher un point y fait entrer, sans règle à écrire.

### Planter un site dans un pixel de la page

Quand on zoome sur une page, chacun de ses pixels devient un point. L'auteur peut planter un site dans l'un d'eux, avec `pixels:`. Au repos, ce point occupe un seul pixel : on ne le remarque qu'en s'approchant. En vue points, on clique dessus : on s'en approche, puis on entre dans le site qu'il contient.

```holo
Page(
  title: "My site",
  children: [
    H1("My site"),
    Button(name: Open, text: "A button"),
  ],
  pixels: [
    Point(
      name: Secret,
      above: Open,
      seed: 77,
      color: "#FF4D6D",
      inside: World.secret(
        children: [
          H1("The hidden site"),
          Button(name: Out, text: "Leave"),
        ],
        rules: [
          On(Out.tap, effect: Secret.leave),
        ],
      ),
    ),
  ],
)

.secret { background: #3a0d1a; color: #FFD6DE; }
```

- `above: Open` place le point juste au-dessus du bloc nommé `Open`, à l'extrémité droite de la page. C'est, pour l'instant, la seule façon de dire où il est.
- `color` lui donne une couleur qui le distingue de la page.
- `World.secret(...)` donne au site du dedans son propre style.
- Entrer dans un point qui contient un site ouvre le **carrefour** : des portails ronds, chacun montrant le site où il mène. Celui vers lequel on va est au milieu, en grand ; autour, les autres sites contenus dans la page, le site où l'on est, et celui d'où l'on vient. Le bouton « Carrefour », en haut à droite, l'ouvre à tout moment.
- Un clic sur un portail l'ouvre : il grandit jusqu'à remplir la fenêtre et devient le site, sans recharger la page. C'est alors un site comme un autre, avec ses propres pixels où l'on peut zoomer. Le `World` d'un point accepte lui aussi `pixels:`, donc un site peut en contenir un autre, qui en contient un autre, sans fin.
- Chaque site a son adresse : celle du fichier, puis `#` et le chemin des points traversés, comme `mon-site.holo#Secret/Tresor`. Le bouton « retour » du navigateur remonte d'un site.
- `On(Out.tap, effect: Secret.leave)` fait ressortir du site `Secret`.
- `Zoom(levels: 8)` limite le nombre de sites emboîtés : au-delà, le fichier est refusé.

Cette écriture est provisoire : elle sert à voir l'effet, et sera revue.

## 7. Comment la page se regarde : `Zoom`, `Points`, `Relief`

Quand le visiteur zoome sur la page (Ctrl + molette, ou pincer), elle grossit d'abord comme n'importe quel site : le texte reste du texte, on le lit, on le sélectionne, on le copie. Au-delà du grossissement fixé par `Points(after:)`, chaque pixel devient un point lumineux, qui se morcelle ensuite. Dès que la page est grossie, glisser la déplace, dans tous les sens ; c'est le même geste avant et après le passage aux points. Quand il tourne la page, elle prend du relief. Ces trois blocs règlent cela. Ils sont facultatifs : sans eux, la page prend les valeurs ci-dessous.

```holo
Page(
  title: "My shop",

  zoom: Zoom(
    max: 1000000,
    shrink: false,
    levels: 8,
  ),

  points: Points(
    after: 4,
    size: 6px,
    fragment: 40px,
    grid: 4,
    depth: 20,
    density: 2,
  ),

  relief: Relief(
    height: 10px,
    tilt: 52deg,
  ),

  children: [
    H1("My shop"),
  ],
)
```

| Réglage | Sens | Bornes |
|---|---|---|
| `Zoom(max:)` | Combien de fois on peut grossir la page, au plus. | 1 à 1 000 000 000 000 |
| `Zoom(shrink:)` | `true` : dézoomer réduit la page jusqu'à un seul point. `false` : la page reste entière. | `true`, `false` |
| `Zoom(levels:)` | Combien de sites peuvent s'emboîter, au plus. | 1 à 16 |
| `Points(after:)` | Jusqu'à ce grossissement, la page reste un site ordinaire. | 1 à 16 |
| `Points(size:)` | La taille où un pixel devient un point. | 2px à 32px |
| `Points(fragment:)` | La taille où un point se morcelle. | 8px à 400px, au moins `size` × `grid` |
| `Points(grid:)` | Un point se morcelle en `grid` × `grid`. | 2 à 8 |
| `Points(depth:)` | Combien de fois de suite. | 0 à 20 |
| `Points(density:)` | Points par pixel d'écran, dans chaque sens. | 1 à 3 |
| `Relief(height:)` | La hauteur du relief. | 0px à 40px |
| `Relief(tilt:)` | Jusqu'où l'on peut tourner la page. | 0deg à 80deg |

Ce sont des garde-fous : le visiteur ne dépasse pas ceux de l'auteur, et l'auteur ne dépasse pas ceux du langage.

### Le carrefour : `Portals`

Le carrefour montre les mondes voisins sous forme de portails. Il s'ouvre quand on entre dans un point qui contient un site, par le bouton « Carrefour », ou par une règle : `On(Map.tap, effect: Shop.portals)`, où `Shop` est le nom de la page.

```holo
Page(
  name: Shop,
  title: "My shop",

  zoom: Zoom(active: true),

  portals: Portals(
    layout: grid,
    count: 12,
    size: 170px,
    brightness: 0.15,
  ),

  children: [
    H1("My shop"),
    Button(name: Map, text: "See the other worlds"),
  ],
  rules: [
    On(Map.tap, effect: Shop.portals),
  ],
)
```

| Réglage | Sens | Bornes |
|---|---|---|
| `Portals(layout:)` | `grid` : en grille sur toute la fenêtre. `row` : une ligne, on défile de gauche à droite. `column` : une colonne, de haut en bas. `diagonal` : en diagonale. | l'un de ces quatre mots |
| `Portals(count:)` | Combien de mondes on montre. | 1 à 64 |
| `Portals(size:)` | La taille d'un portail. | 80px à 400px |
| `Portals(brightness:)` | La lumière du fond, pour y voir même sur un site sombre. | 0 à 1 |
| `Zoom(active:)` | `false` : le visiteur ne peut pas zoomer dans la page. | `true`, `false` |

- Les sites écrits dans le fichier passent d'abord, et montrent leur contenu. Le reste de la place est rempli par des mondes calculés à partir d'une graine : des boules de lumière, dans lesquelles on entre comme dans le Big Bang.
- Là où le curseur se pose, le monde s'avance : il grandit et devient une feuille lisible.
- Un monde calculé a lui aussi son adresse : `my-shop.holo#~` suivi de sa graine.

## 8. Un monde seul

Un fichier peut ne contenir qu'un `Point` : il s'ouvre alors directement en profondeur. C'est le Big Bang.

```holo
Point(
  name: Origin,
  seed: 1,
  brightness: 1.0,
  fragments: 12,
)
```

On zoome : le point se morcelle en ses fragments. On zoome encore sur l'un d'eux : on y entre, et il se morcelle à son tour, sans fin. Chaque monde se calcule à partir de sa graine ; rien n'est stocké.

## 9. Les unités

Une unité se colle au nombre : `500KB`, jamais `500 KB`.

| Unités | Pour |
|---|---|
| `px`, `%` | Les tailles à l'écran |
| `deg` | Les angles |
| `B`, `KB`, `MB`, `GB` | Les poids (décimaux : 1 KB = 1 000 octets) |
| `mm`, `cm`, `m`, `km`, `ms`, `s`, `min`, `h` | Longueurs et durées : lues par le moteur, pas encore employées |

## 10. Aide-mémoire

| Bloc | Réglages | Où |
|---|---|---|
| `Page` | `name`, `title`, `children`, `pixels`, `rules`, `zoom`, `points`, `relief`, `portals` | À la racine |
| `H1`, `H2`, `H3`, `P`, `Text` | le texte entre guillemets ; `name` | Dans `children` |
| `A` | le texte entre guillemets, `to` | Dans `children` |
| `Image` | `source`, `weight`, `name` | Dans `children` |
| `List` | `children`, `ordered`, `name` | Dans `children` |
| `Button` | `name`, `text` | Dans `children` |
| `Point` | `name`, `seed`, `brightness`, `fragments`, `color`, `palette`, `budget`, `inside` ; `above` quand il est planté dans un pixel | Dans `children` ou `pixels`, ou à la racine |
| `World` | `children`, `pixels`, `rules` | Dans `inside:` d'un `Point` |
| `On` | le signal, puis `effect:` | Dans `rules` |
| `Zoom`, `Points`, `Relief`, `Portals` | voir la partie 7 | Dans `zoom:`, `points:`, `relief:`, `portals:` d'une `Page` |

## 10 bis. Chaque notion et son mot

Tout ce que le moteur sait faire doit avoir son mot dans le langage. Voici où l'on en est.

| Notion | Son mot dans le langage | État |
|---|---|---|
| Une page, un texte, un titre | `Page`, `Text`, `P`, `H1` à `H3` | fait |
| Une image | `Image(source:, weight:)` | fait |
| Une liste, une liste numérotée, un bouton | `List`, `List(ordered: true)`, `Button` | fait |
| Un lien classique (on change de page) | `A("texte", to: "adresse")` | fait |
| Passer dans un autre fichier sans changer de page | `Point(inside: "fichier.holo")` | fait |
| Ressortir d'un monde | dézoomer, ou la capacité `leave` | fait |
| L'apparence | les styles : `P { color: … }`, `.card { … }` | fait |
| Un point, un monde | `Point`, `World`, `seed`, `brightness`, `color`, `palette` | fait |
| Le morcellement d'un point | `fragments` | fait |
| Le pixel d'une page | `pixels:` et `above:` | fait, écriture provisoire |
| Le pixel qui devient un point | `Points(after:, size:)` | fait |
| Le morcellement des points d'une page | `Points(fragment:, grid:, depth:)` | fait |
| Le relief | `Relief(height:, tilt:)` | fait |
| Activer ou désactiver le zoom | `Zoom(active:)` | fait |
| Les limites du zoom | `Zoom(max:, shrink:)` | fait |
| Le nombre de sites emboîtés | `Zoom(levels:)` | fait |
| Entrer dans un site, en sortir | `enter`, `leave` | fait |
| Le carrefour, les portails | `Portals(layout:, count:, size:, brightness:)`, et la capacité `portals` | fait |
| Le poids permis | `budget`, `weight` | fait |
| Le toucher | le signal `tap` | fait |
| Le son | aucun | à faire |
| La vidéo | aucun | à faire |
| Le survol, l'approche | aucun | à faire |
| Réagir au zoom par une règle (« quand on zoome, alors… ») | aucun | à faire |
| La disposition côte à côte | aucun | à faire |
| Réutiliser un morceau de fichier (les imports) | `import` est lu, pas appliqué | à faire |
| Les formulaires, les valeurs qui changent | aucun | à faire |
| Le personnage | aucun | à faire |

L'exemple le plus complet : [`exemples/boutique-comparee/boutique.holo`](../../exemples/boutique-comparee/boutique.holo).

## 11. Ce qui n'existe pas encore

- Les imports (`import`, `module`, `bridge js`, `bridge css`) : le moteur les lit mais ne les applique pas.
- La disposition (côte à côte, en grille) : tout est l'un sous l'autre.
- Les formulaires, les valeurs qui changent (un panier), les données venues d'ailleurs.
- Le reste du Markdown (seuls le gras et l'italique sont rendus).
- Entrer dans un point écrit à l'intérieur d'un monde.
- Les garde-fous de zoom pour un `Point` seul : ils sont encore fixés dans le moteur.
- Un seul nom de style par bloc ; pas de fichier de styles à part.
