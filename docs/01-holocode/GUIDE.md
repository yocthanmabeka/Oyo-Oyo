# Écrire en `.holo` : le guide de l'auteur

Ce guide montre comment écrire un fichier `.holo` aujourd'hui, avec ce que le moteur sait vraiment faire. Tous les exemples entre balises `holo` sont relus par un test du moteur à chaque changement : s'ils cessaient de marcher, le test échouerait.

- État : langage en construction (2026-10-03). Ce qui n'existe pas encore est listé à la fin.
- Ce que HoloCode couvre et ne couvre pas du web classique, balise par balise : [`COMPARAISON-WEB.md`](COMPARAISON-WEB.md).
- Les décisions derrière chaque règle : [`docs/02-gouvernance/DECISIONS.md`](../02-gouvernance/DECISIONS.md).

> Pour apprendre pas à pas : [les leçons](../../exemples/lecons/README.md), une notion par fichier. Ce guide est la référence complète.

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

## 4 a. Trait, citation, texte tel quel, retour à la ligne

```holo
Page(
  title: "My shop",
  children: [
    H1("My shop"),
    Image(source: "painting.svg", weight: 1KB, alt: "A yellow sun over green hills"),

    Hr(),

    Quote("I still stop to look at it.", by: "A customer"),

    "Use the code `WELCOME` at checkout.",
    Code("WELCOME"),

    P("""
      Open Monday to Friday.
      Closed on Sunday.
    """),
  ],
)
```

| Écriture | Ce que ça donne |
|---|---|
| `Hr()` | Un trait de séparation. |
| `Quote("…", by: "…")` | Une citation. `by` dit qui l'a dit ; on peut l'omettre. |
| `Code("…")` | Du texte montré tel quel, lettre pour lettre : rien n'y est interprété. |
| des accents graves dans une phrase | Le même effet, pour un mot dans une phrase. |
| un texte entre trois guillemets | Il peut tenir sur plusieurs lignes, et chaque retour à la ligne est gardé. |
| `Image(alt: "…")` | Ce que montre l'image, pour qui ne la voit pas. Sans `alt`, l'image est tenue pour un décor. |

## 4 bis. La disposition : `Row`, `Column`, `Grid`

Sans rien écrire, les blocs se rangent l'un sous l'autre. Trois blocs les rangent autrement.

```holo
Page(
  title: "My shop",
  children: [
    Row(gap: 8px, align: between, children: [
      H1("My shop"),
      Button(name: Menu, text: "Menu"),
    ]),

    Grid(columns: 3, gap: 12px, children: [
      Column(gap: 4px, align: center, children: [ P("Sunrise over the river"), Text("120 euros") ]),
      Column(gap: 4px, align: center, children: [ P("The blue door"), Text("90 euros") ]),
      Column(gap: 4px, align: center, children: [ P("Market day"), Text("150 euros") ]),
    ]),
  ],
)
```

| Bloc | Ce qu'il fait |
|---|---|
| `Row` | Range côte à côte. Ce qui ne tient pas dans la largeur passe à la ligne : la page ne déborde jamais sur le côté. |
| `Column` | Range l'un sous l'autre. |
| `Grid` | Range en colonnes de même largeur. Sur un écran étroit, il y a moins de colonnes, sans rien écrire. |

| Réglage | Sens | Valeurs |
|---|---|---|
| `gap` | L'écart entre les éléments. | 0px à 64px ; 16px sans rien écrire |
| `align` (`Row`, `Column`) | Où se placent les éléments, dans le sens de la largeur. | `start`, `center`, `end` ; pour `Row` aussi `between` (écartés d'un bord à l'autre) |
| `columns` (`Grid`) | Le nombre de colonnes, au plus. | 1 à 12 ; 2 sans rien écrire |

- Ces blocs se rangent les uns dans les autres.
- Ils prennent un style nommé comme les autres : `Column.card(...)`.
- Un bouton ou un point rangé dans une ligne garde son nom et ses règles.

Cette écriture est à l'essai (`ADR-024`).

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

- On y passe **sans changer de page**. Par un bouton, c'est un passage ordinaire, comme sur le web : la page s'efface, l'autre apparaît. En touchant le point lui-même, il s'ouvre sur place, grandit jusqu'à remplir la fenêtre, et devient l'autre fichier. L'adresse de la barre devient celle de l'autre fichier, et le bouton « retour » ramène.
- Les fichiers où mènent les points d'une page sont lus d'avance (ils sont petits) : le passage est immédiat. Un fichier introuvable ou refusé par le moteur laisse le passage fermé.
- **Dézoomer** alors que la page est déjà à sa taille normale fait ressortir du monde où l'on est : on revient au site, ou au fichier, d'où l'on venait.
- La différence avec `A` : `A` fait changer de page, à l'ancienne ; un `Point` se traverse à pied.
- **Vers le site de quelqu'un d'autre** : `inside` accepte aussi l'adresse complète d'un fichier, en `https` : `inside: "https://ami.example/jardin.holo"`. Le serveur d'en face doit autoriser la lecture (l'en-tête `access-control-allow-origin`). Quatre protections :
  - le portail affiche le nom du serveur, sans aperçu : le fichier n'est lu que lorsque le visiteur clique sur ce portail. Un clic, un serveur, un fichier ;
  - une adresse ouverte directement vers un autre serveur ne le contacte pas non plus : le moteur propose le passage, et le visiteur décide ;
  - une fois chez quelqu'un d'autre, un bandeau « Vous êtes chez … » reste affiché, avec un bouton « Revenir ». Il appartient au moteur : le fichier ne peut ni le cacher ni le styler ;
  - le `http` n'est accepté que vers sa propre machine (`localhost`), pour les essais.
- Un navigateur interdit à une page d'afficher l'adresse d'un autre serveur comme si elle y était : l'adresse garde donc le fichier de départ, suivi de `#@` et de l'adresse réelle.
- On peut passer d'un fichier à l'autre sans fin. La mémoire, elle, est bornée : le moteur ne garde que les 32 derniers fichiers lus, un fichier de plus de 256 Ko n'est pas lu, et un serveur qui ne répond pas en 8 secondes est abandonné.

L'exemple complet : [`exemples/maison/`](../../exemples/maison/salon.holo), un salon et un jardin.

**Les règles.** Un bloc ne contient jamais de code. Une règle relie un signal à une capacité :

- `On(Open.tap, effect: Workshop.enter)` : quand le bouton `Open` est touché, on entre dans le point `Workshop`.
- Signaux : `tap` (un `Button` ou un `Point` touché).
- Capacités : `enter` et `leave` (pour un `Point`), `portals` (pour la `Page` : ouvrir son carrefour), `play` (pour un `Sound`).
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
- Entrer dans un point qui contient un site y mène **directement**. Il y a deux façons d'y aller, et chacune a son mouvement :
  - par un **bouton** de la page (`On(Open.tap, effect: Workshop.enter)`) : c'est un lien. On passe d'un site à l'autre comme sur le web, par un fondu bref ;
  - en touchant **le point lui-même** : là, on est dans le métavers. Le point s'ouvre où il est, grandit jusqu'à remplir la fenêtre, et devient le site.
- Le **carrefour** ne s'ouvre que si on le demande : par le bouton « Carrefour », dans le menu du moteur (le bouton rond en bas à droite), ou par une règle (`portals`). Il montre des portails ronds, chacun avec le site où il mène : les sites contenus dans la page, celui où l'on est, celui d'où l'on vient. Seule exception : un point qui mène au fichier d'un autre serveur passe par le carrefour, pour qu'on voie le nom du serveur avant d'y aller.
- Un clic sur un portail l'ouvre : il grandit jusqu'à remplir la fenêtre et devient le site, sans recharger la page. C'est alors un site comme un autre, avec ses propres pixels où l'on peut zoomer. Le `World` d'un point accepte lui aussi `pixels:`, donc un site peut en contenir un autre, qui en contient un autre, sans fin.
- Chaque site a son adresse : celle du fichier, puis `#` et le chemin des points traversés, comme `mon-site.holo#Secret/Tresor`. Le bouton « retour » du navigateur remonte d'un site.
- `On(Out.tap, effect: Secret.leave)` fait ressortir du site `Secret`.
- `Zoom(levels: 8)` limite le nombre de sites emboîtés : au-delà, le fichier est refusé.

Cette écriture est provisoire : elle sert à voir l'effet, et sera revue.

## 6 bis. Ce que la page retient : `State`

Une page peut retenir des valeurs : un panier, un compteur, des « j'aime ».

```holo
Page(
  title: "My shop",

  state: State(cart: 0),

  children: [
    H1("My shop"),
    Text("{cart} paintings in your cart"),
    Button(name: Add, text: "Add a painting"),
    Button(name: Remove, text: "Remove one"),
    Button(name: Empty, text: "Empty the cart"),
  ],

  rules: [
    On(Add.tap, effect: cart.add(1)),
    On(Remove.tap, effect: cart.sub(1)),
    On(Empty.tap, effect: cart.set(0)),
  ],
)
```

- `state: State(cart: 0)` déclare une valeur, `cart`, qui part de 0. On peut en déclarer plusieurs : `State(cart: 0, likes: 3)`.
- `{cart}` dans un texte affiche la valeur. Quand elle change, le texte suit tout seul.
- Un bouton ne change rien lui-même. Une règle fait une **demande** au moteur, et c'est lui qui change la valeur.

| Demande | Sens |
|---|---|
| `cart.add(1)` | Ajouter 1. |
| `cart.sub(1)` | Retirer 1. La valeur ne descend jamais sous 0. |
| `cart.set(0)` | Fixer à 0. |

Les limites :

- Une valeur est un nombre entier, de 0 à 1 000 000 000. Une page en déclare au plus 32.
- Son nom s'écrit en minuscules : `cart`, `items_seen`.
- Les valeurs se déclarent sur la `Page`. Elles valent pour tout le fichier : un texte écrit dans le monde d'un point peut montrer `{cart}`, et une règle de ce monde peut le changer.
- La valeur suit le visiteur d'un monde à l'autre et d'un fichier à l'autre. Si la page est rechargée, elle repart du départ.

Ce que le moteur refuse :

- `{car}` quand aucune valeur ne s'appelle `car` ;
- une demande inconnue, comme `cart.double(1)` ;
- une demande sans sa quantité, comme `cart.add` ;
- une demande écrite ailleurs que dans `effect:`.

**Des prix, un nombre d'articles, un total.** Quand la page donne des prix, le moteur calcule deux valeurs de plus : `{count}`, le nombre d'articles, et `{total}`, ce qu'ils coûtent ensemble. L'auteur n'écrit aucun calcul.

```holo
Page(
  title: "My shop",

  state: State(sunrise: 0, blue_door: 0),
  prices: Prices(sunrise: 120, blue_door: 90),

  children: [
    Text("Sunrise over the river, 120 euros. {sunrise} in your cart."),
    Button(name: AddSunrise, text: "Add"),
    Text("The blue door, 90 euros. {blue_door} in your cart."),
    Button(name: AddBlueDoor, text: "Add"),

    Text("{count} paintings, {total} euros"),
    Button(name: Empty, text: "Empty the cart"),
  ],

  rules: [
    On(AddSunrise.tap, effect: sunrise.add(1)),
    On(AddBlueDoor.tap, effect: blue_door.add(1)),
    On(Empty.tap, effect: sunrise.set(0)),
    On(Empty.tap, effect: blue_door.set(0)),
  ],
)
```

- Dans `State`, chaque valeur est la quantité d'un article.
- Dans `Prices`, chaque prix porte le nom d'une valeur : c'est le prix de cet article. Un prix est un nombre entier.
- Une valeur sans prix (des « j'aime », par exemple) ne compte ni dans `{count}` ni dans `{total}`.
- Avec `prices:`, les noms `count` et `total` sont pris par le moteur : on ne les déclare pas dans `State`, et on ne peut pas les changer par une demande.
- Un même signal peut avoir plusieurs règles : `Empty.tap` remet ici deux valeurs à zéro.

Limites : les articles sont écrits d'avance dans le fichier ; pas de centimes ; et ce panier est un affichage, pas une commande : rien n'est envoyé à un serveur.

Cette écriture est à l'essai (`ADR-023`).

## 6 ter. Montrer selon une valeur : `If`

Une condition montre ce qu'elle contient seulement quand elle est vraie.

```holo
Page(
  title: "My shop",
  state: State(cart: 0),
  children: [
    Button(name: Add, text: "Add a painting"),

    If(cart, is: 0, children: [
      "Your cart is empty.",
    ]),

    If(cart, over: 0, children: [
      Text("{cart} paintings in your cart"),
      Button(name: Empty, text: "Empty the cart"),
    ]),

    If(cart, over: 2, under: 10, children: [
      "Delivery is free.",
    ]),
  ],
  rules: [
    On(Add.tap, effect: cart.add(1)),
    On(Empty.tap, effect: cart.set(0)),
  ],
)
```

| Comparaison | Sens |
|---|---|
| `is: 0` | égal à 0 |
| `not: 0` | différent de 0 |
| `over: 0` | plus grand que 0 |
| `under: 10` | plus petit que 10 |

- Le premier mot est le nom d'une valeur : une valeur de `State`, ou `count` et `total` quand la page donne des prix.
- Plusieurs comparaisons valent ensemble : `over: 2, under: 10` veut dire « de 3 à 9 ».
- Quand la valeur change, la page suit toute seule.
- Il n'y a pas de « sinon » : on écrit une seconde condition, comme ci-dessus.
- Une condition se place dans `children`, y compris dans une ligne, une colonne ou une grille.

Cette écriture est à l'essai (`ADR-025`).

## 6 quater. Un jeu : le temps, le hasard, le plateau

Trois mots suffisent pour un premier jeu. Le jeu entier est dans `exemples/jeu/attraper.holo` : on touche une étoile le plus de fois possible en trente secondes.

```holo
Page(
  name: Catch,
  title: "Catch the star",

  state: State(time: 0, score: 0, star_x: 50, star_y: 50),

  children: [
    If(time, is: 0, children: [
      Button(name: Play, text: "Play"),
    ]),
    If(time, over: 0, children: [
      Text("Score: {score}. Time: {time} s"),
      Board(height: 320px, children: [
        Point(name: Star, seed: 7, x: star_x, y: star_y),
      ]),
    ]),
  ],

  rules: [
    On(Play.tap, effect: time.set(30)),
    On(Star.tap, effect: [score.add(1), star_x.random(100), star_y.random(100)]),
    Every(1s, effect: time.sub(1)),
    Every(2s, effect: [star_x.random(100), star_y.random(100)]),
  ],
)
```

**Le temps : `Every`.** Une règle qui se répète. `Every(1s, effect: time.sub(1))` : toutes les secondes, retirer 1 à `time`.

- Le rythme s'écrit en `s` ou en `ms`, de `100ms` à `3600s`.
- L'effet est une demande, comme pour `On`.
- L'horloge se tait quand la fenêtre est cachée, en vue points et devant le carrefour.
- Chaque règle `Every` a sa propre horloge. Quand un geste change une valeur, l'horloge de cette valeur repart de zéro : après `On(Play.tap, effect: time.set(30))`, la première seconde dure une vraie seconde.

**Le hasard : `random`.** `star_x.random(100)` donne à `star_x` un nombre de 0 à 100, bornes comprises. Ce hasard est rejouable : les mêmes gestes, aux mêmes moments, redonnent la même partie.

**Le plateau : `Board`.** Ce qu'il contient se place où l'on veut.

| Réglage | Sens | Valeurs |
|---|---|---|
| `Board(height:)` | La hauteur du plateau. | 80px à 800px ; 320px sans rien écrire |
| `x:` sur un bloc du plateau | Sa place de gauche à droite. | un nombre de 0 à 100, ou le nom d'une valeur |
| `y:` sur un bloc du plateau | Sa place de haut en bas. | un nombre de 0 à 100, ou le nom d'une valeur |

- Avec un nombre, le bloc reste à sa place. Avec le nom d'une valeur, il suit cette valeur quand elle change.
- De 0 à 100, un bloc ne sort jamais du plateau.

**Commencer et finir sans mot de plus.** Le temps ne descend pas sous zéro. Les conditions font le reste : `If(time, is: 0)` montre le bouton « Play » et le score, `If(time, over: 0)` montre le plateau.

Cette écriture est à l'essai (`ADR-026`).

## 6 quater bis. Un jeu qui bouge : le clavier, le glissement, `When`

Le second jeu est dans `exemples/jeu/panier.holo` : une pomme tombe, on la rattrape avec un panier.

```holo
Page(
  name: Orchard,
  title: "Catch the apple",

  state: State(lives: 3, score: 0, basket: 50, apple_x: 50, apple_y: 0),

  children: [
    Text("Score: {score}. Lives: {lives}"),
    Board(height: 360px, children: [
      Point(name: Apple, seed: 3, x: apple_x, y: apple_y),
      Point(name: Basket, seed: 9, x: basket, y: 96, drag: true),
    ]),
  ],

  rules: [
    On(Key.left, effect: basket.sub(8)),
    On(Key.right, effect: basket.add(8)),

    Every(100ms, effect: apple_y.add(3)),

    When(Basket, meets: Apple, within: 9, effect: [score.add(1), apple_x.random(100), apple_y.set(0)]),
    When(apple_y, over: 99, effect: [lives.sub(1), apple_x.random(100), apple_y.set(0)]),
  ],
)
```

**Trois sortes de règles, pas plus.**

| Règle | Elle répond à | Exemple |
|---|---|---|
| `On` | un geste du visiteur | `On(Play.tap, …)`, `On(Key.left, …)` |
| `Every` | le temps | `Every(100ms, …)` |
| `When` | un moment : quelque chose devient vrai | `When(apple_y, over: 99, …)`, `When(Basket, meets: Apple, …)` |

**Plusieurs demandes dans une règle.** On les met entre crochets : `effect: [score.add(1), apple_y.set(0)]`. Elles sont faites dans l'ordre. Une seule demande s'écrit sans crochets.

**Le clavier : `Key`.** `On(Key.left, effect: basket.sub(8))`. `Key` est le clavier du visiteur.

| Signal | Touche |
|---|---|
| `Key.left`, `Key.right`, `Key.up`, `Key.down` | les quatre flèches |
| `Key.space` | la barre d'espace |

Seules les touches que le fichier écoute sont prises. Les autres gardent leur rôle.

**Faire glisser : `drag: true`.** Sur un bloc posé dans un `Board`, le visiteur peut le faire glisser, au doigt ou à la souris. Ses places, quand ce sont des valeurs de la page, suivent le doigt. Ici `basket` suit ; `y: 96` est un nombre fixe, donc le panier ne monte pas. Aucune règle à écrire.

**Ce qui bouge tout seul.** `Every(100ms, effect: apple_y.add(3))` : dix fois par seconde, la pomme descend un peu.

**Une valeur qui sert de place reste sur le plateau** : de 0 à 100. Le panier ne sort jamais.

**Une règle qui guette : `When`.** Elle se déclenche au moment où ce qu'elle guette **devient** vrai, pas tant qu'il le reste. Elle guette :

- une valeur : `When(apple_y, over: 99, effect: …)`, avec les comparaisons de `If` (`is`, `not`, `over`, `under`) ;
- une rencontre : `When(Basket, meets: Apple, within: 9, effect: …)`. Les deux blocs ont un nom, et un `x` et un `y` dans un `Board`. `within` est la distance de rencontre, sur l'échelle du plateau (de 0 à 100) ; 10 sans rien écrire.

Cette écriture est à l'essai (`ADR-028`).

## 6 quinquies. Saisir, et garder : `Input`, `Checkbox`, `keep`

```holo
Page(
  title: "My shop",

  state: State(cart: 0, gift: 0, tip: 0),
  keep: [cart, gift, tip],

  children: [
    Button(name: Add, text: "Add a painting"),
    Text("{cart} paintings in your cart"),

    Checkbox(value: gift, label: "Gift wrap"),
    If(gift, is: 1, children: [
      "We will wrap your paintings.",
    ]),

    Input(value: tip, label: "A tip, in euros", max: 50),
    If(tip, over: 0, children: [
      Text("Thank you for the {tip} euros."),
    ]),
  ],

  rules: [
    On(Add.tap, effect: cart.add(1)),
  ],
)
```

| Bloc | Ce que c'est | Réglages |
|---|---|---|
| `Checkbox` | Une case à cocher. Cochée, la valeur vaut 1 ; sinon 0. | `value`, `label` |
| `Input` | Un champ où l'on écrit : un nombre entier, ou un texte, selon la valeur qu'il présente. | `value`, `label`, `max` |

- `value` est le nom d'une valeur de `State`. Le champ la montre, et la change quand le visiteur écrit. Il n'y a pas de règle à écrire.
- `label` est obligatoire : il dit ce qu'on attend.
- `max` borne ce qu'on peut écrire : au-delà, la valeur s'arrête à `max`.
- Ce qui n'est pas un nombre ne change rien.

**Garder d'une visite à l'autre.** `keep: [cart, gift, tip]`, sur la page, nomme les valeurs que le navigateur du visiteur garde. Il recharge la page, ou revient demain : elles sont encore là. Les valeurs qui ne sont pas dans `keep` repartent de leur départ.

**Une valeur peut être un texte.** On la déclare avec des guillemets, et le même bloc `Input` devient un champ de texte.

```holo
Page(
  title: "My shop",

  state: State(buyer: ""),
  keep: [buyer],

  children: [
    Input(value: buyer, label: "Your first name", max: 20),

    If(buyer, is: "", children: [
      "Tell us your first name.",
    ]),
    If(buyer, not: "", children: [
      Text("Welcome, {buyer}."),
    ]),
  ],
)
```

- `State(buyer: "")` : un texte, vide au départ. On peut aussi lui donner un départ : `city: "Paris"`.
- `{buyer}` le montre, comme un nombre.
- `Input(value: buyer, …)` : comme la valeur est un texte, le champ est un champ de texte. `max` borne alors sa longueur (80 caractères sans rien écrire, 200 au plus).
- Un texte ne se compare qu'au vide : `is: ""` (il est vide) et `not: ""` (il est rempli).
- Un texte ne change que par un champ : il n'y a pas de demande pour lui.
- Ce que le visiteur écrit ne devient jamais du code : la page le montre lettre pour lettre.

Limites : pas de liste de choix, pas d'envoi à un serveur ; ce qui est gardé reste sur cet appareil.

Cette écriture est à l'essai (`ADR-027`).

## 6 sexies. Un site de plusieurs pages : `import`, `Part`, `Use`

Le menu et le thème d'un site s'écrivent une seule fois, dans un fichier à part. L'exemple est dans `exemples/site/`.

Le fichier commun, `commun.holo`, est un **morceau** :

```holo
Part(
  name: Menu,
  children: [
    Row(gap: 16px, children: [
      A("Home", to: "accueil.holo"),
      A("Contact", to: "contact.holo"),
    ]),
    Hr(),
  ],
)

Page { background: #f6f1e7; color: #2b2118; font-family: Georgia, serif; }
H1 { color: #8a3b12; }
```

Une page l'importe, puis le pose :

```text
import "commun.holo"

Page(
  title: "The little studio",
  children: [
    Use(Menu),
    H1("Welcome"),
    "We paint, we frame, we deliver.",
  ],
)
```

- `import "commun.holo"` s'écrit tout en haut. Le fichier est rangé à côté de la page.
- `Part(name: Menu, children: [...])` : un morceau a un nom et un contenu. Ce n'est pas une page.
- `Use(Menu)` pose les blocs du morceau à cet endroit.
- Les styles du morceau viennent avec lui. Si la page écrit le même style, c'est le sien qui reste.
- Un bouton écrit dans un morceau garde son nom : une règle de la page peut l'écouter.

Limites : un morceau n'a ni valeurs ni règles, et n'importe pas d'autres fichiers ; seize imports au plus.

Cette écriture est à l'essai (`ADR-029`).

## 6 septies. Des données venues du serveur : `Data`

Une page peut aller chercher des valeurs dans un fichier rangé à côté d'elle. La leçon est `exemples/lecons/27-donnees.holo`.

```holo
Page(
  title: "My shop",

  state: State(stock: 0, message: ""),
  data: Data(from: "stock.json", every: 30s),

  children: [
    Text("{message}"),
    If(stock, over: 0, children: [
      Text("{stock} paintings left."),
    ]),
    If(stock, is: 0, children: [
      "Sold out.",
    ]),
  ],
)
```

Le fichier `stock.json` :

```text
{ "stock": 4, "message": "Open until 6 pm" }
```

| Réglage | Sens | Valeurs |
|---|---|---|
| `Data(from:)` | Le fichier de données, rangé à côté de la page. | un fichier `.json` |
| `Data(every:)` | À quel rythme la page le redemande. Sans lui : une seule fois, à l'ouverture. | 1s à 3600s |

- Chaque nom du fichier remplit la valeur de `State` du même nom : un nombre entier dans un nombre, un texte dans un texte.
- Ce qui ne correspond à rien est laissé de côté. Un fichier mal écrit ne change rien, et la page garde ses valeurs.
- La page ne parle qu'au serveur d'où elle vient.
- Rien n'est demandé quand la fenêtre est cachée.

Limites : pas de liste (on ne reçoit pas « tous les articles ») ; la page n'envoie rien au serveur.

Cette écriture est à l'essai (`ADR-030`).

## 6 octies. Un son : `Sound`

```holo
Page(
  title: "A sound",
  state: State(count: 0),
  children: [
    Sound(name: Ding, source: "ding.wav"),
    Text("{count} rings"),
    Button(name: Ring, text: "Ring"),
  ],
  rules: [
    On(Ring.tap, effect: [count.add(1), Ding.play]),
    When(count, is: 5, effect: Ding.play),
  ],
)
```

- `Sound(name: Ding, source: "ding.wav")` : un son. Le fichier est rangé à côté (`.wav`, `.mp3`, `.ogg`). Il ne se voit pas.
- `Ding.play` le fait entendre. Cela s'écrit dans l'effet d'une règle, seul ou dans une liste.
- Les trois sortes de règles peuvent jouer un son : `On`, `Every`, `When`.
- Un navigateur ne joue un son qu'après un premier geste du visiteur.

Limites : ni boucle, ni volume, ni arrêt.

Cette écriture est à l'essai (`ADR-031`). La leçon est `exemples/lecons/28-son.holo`.

## 7. Comment la page se regarde : `Zoom`, `Points`, `Relief`

Quand le visiteur zoome sur la page (Ctrl + molette, ou pincer), elle grossit d'abord comme n'importe quel site : le texte reste du texte, on le lit, on le sélectionne, on le copie. Au-delà du grossissement fixé par `Points(after:)`, chaque pixel devient un point lumineux, qui se morcelle ensuite. Dès que la page est grossie, glisser la déplace, dans tous les sens ; c'est le même geste avant et après le passage aux points. Quand il tourne la page, elle prend du relief. Ces trois blocs règlent cela.

**Ce qui met la page en 3D s'active.** Sans rien écrire, une page est un site ordinaire : on peut la grossir pour mieux lire, et rien d'autre ne se passe.

| Pour avoir | L'auteur écrit |
|---|---|
| les pixels qui deviennent des points au zoom | `points: Points()` (ou un site planté dans un pixel, avec `pixels:`) |
| la page qui tourne, et son relief | `relief: Relief(tilt: 360deg)`, en plus de `points:` |

`relief:` sans `points:` est refusé : le relief est celui des points. Dans `Points()` et `Relief()`, chaque réglage est facultatif : sans lui, il prend la valeur ci-dessous.

```holo
Page(
  title: "My shop",

  zoom: Zoom(
    max: 1000000,
    shrink: false,
    levels: 8,
    speed: 1,
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
    tilt: 360deg,
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
| `Zoom(speed:)` | La vitesse du zoom à la molette. `1` : la vitesse ordinaire ; `2` : deux fois plus vite. Au doigt, la page suit toujours l'écartement des doigts. | 0.25 à 4 |
| `Points(after:)` | Jusqu'à ce grossissement, la page reste un site ordinaire. Jamais moins de 2 : tout visiteur peut au moins doubler la taille du texte. | 2 à 16 |
| `Points(size:)` | La taille où un pixel devient un point. | 2px à 32px |
| `Points(fragment:)` | La taille où un point se morcelle. | 8px à 400px, au moins `size` × `grid` |
| `Points(grid:)` | Un point se morcelle en `grid` × `grid`. | 2 à 8 |
| `Points(depth:)` | Combien de fois de suite. | 0 à 20 |
| `Points(density:)` | Points par pixel d'écran, dans chaque sens. | 1 à 3 |
| `Relief(height:)` | La hauteur du relief. | 0px à 40px |
| `Relief(tilt:)` | Jusqu'où l'on peut tourner la page, de chaque côté. Sans `tilt`, la page ne tourne pas : c'est un site ordinaire. L'écrire **active** la rotation, et le bouton « Tourner » apparaît, dès la page de face. `360deg` : on en fait le tour, et on la voit par derrière, à l'envers comme une feuille. `52deg` : elle s'arrête à cet angle. | 0deg à 360deg |

Ce sont des garde-fous : le visiteur ne dépasse pas ceux de l'auteur, et l'auteur ne dépasse pas ceux du langage.

Le visiteur a aussi son mot à dire. S'il a choisi « réduire les animations » dans les réglages de son téléphone ou de son ordinateur, la page ne devient pas des points toute seule quand il zoome, et les portails s'ouvrent sans transition. Le bouton « Vue points » reste là pour qui veut y aller. Sans ce choix, tout est au niveau normal.

- `Zoom(max:)` borne le zoom entier : le zoom ordinaire de la page et la vue points ensemble. `Points(after:)` ne peut donc pas dépasser `Zoom(max:)` : le fichier serait refusé.
- Quelle que soit `density`, la vue points ne dépasse jamais huit millions de points, pour tenir dans la mémoire d'un téléphone.

### Le carrefour : `Portals`

Le carrefour montre les mondes voisins sous forme de portails. Il s'ouvre par le bouton « Carrefour », ou par une règle : `On(Map.tap, effect: Shop.portals)`, où `Shop` est le nom de la page.

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
    duration: 450ms,
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
| `Portals(duration:)` | Le temps que met un portail à s'ouvrir. `0ms` : tout de suite. | 0ms à 2000ms |
| `Zoom(active:)` | `false` : le visiteur ne peut pas zoomer dans la page. | `true`, `false` |

- `count` borne tout le carrefour, y compris les sites écrits dans le fichier : s'il y en a davantage, un dernier rond dit combien ne sont pas montrés.
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

### Les limites d'un fichier

Un fichier `.holo` est un texte court. Le moteur refuse, avant toute analyse : un fichier de plus de 262 144 octets, de plus de 100 000 mots, ou dont les blocs et les listes s'emboîtent sur plus de 64 niveaux. Le repère d'un point planté (`above:`) doit être un bloc du même site que lui.

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
| `Page` | `name`, `title`, `children`, `pixels`, `rules`, `state`, `prices`, `keep`, `data`, `zoom`, `points`, `relief`, `portals` | À la racine |
| `H1`, `H2`, `H3`, `P`, `Text` | le texte entre guillemets ; `name` | Dans `children` |
| `A` | le texte entre guillemets, `to` | Dans `children` |
| `Image` | `source`, `weight`, `alt`, `name` | Dans `children` |
| `Sound` | `name`, `source`, `weight` | Dans `children` |
| `Hr` | aucun | Dans `children` |
| `Quote` | le texte entre guillemets, `by` | Dans `children` |
| `Code` | le texte entre guillemets | Dans `children` |
| `If` | le nom d'une valeur, puis `is`, `not`, `over`, `under`, et `children` | Dans `children` |
| `List` | `children`, `ordered`, `name` | Dans `children` |
| `Button` | `name`, `text` | Dans `children` |
| `Point` | `name`, `seed`, `brightness`, `fragments`, `color`, `palette`, `budget`, `inside` ; `above` quand il est planté dans un pixel | Dans `children` ou `pixels`, ou à la racine |
| `World` | `children`, `pixels`, `rules` | Dans `inside:` d'un `Point` |
| `Row`, `Column` | `children`, `gap`, `align`, `name` | Dans `children` |
| `Grid` | `children`, `gap`, `columns`, `name` | Dans `children` |
| `Board` | `children`, `height`, `name` ; ses enfants prennent `x`, `y` et `drag` | Dans `children` |
| `Input` | `value`, `label`, `max`, `name` | Dans `children` |
| `Checkbox` | `value`, `label`, `name` | Dans `children` |
| `Part` | `name`, `children` | À la racine d'un fichier importé |
| `Use` | le nom d'un morceau importé | Dans `children` |
| `On` | le signal, puis `effect:` | Dans `rules` |
| `Every` | le rythme, puis `effect:` | Dans `rules` |
| `When` | le nom d'une valeur, puis `is`, `not`, `over`, `under` ; ou le nom d'un bloc, puis `meets` et `within` ; et `effect:` | Dans `rules` |
| `State` | les valeurs et leur départ : `cart: 0` | Dans `state:` d'une `Page` |
| `Data` | `from`, `every` | Dans `data:` d'une `Page` |
| `Prices` | le prix de chaque article : `sunrise: 120` | Dans `prices:` d'une `Page` |
| `Zoom`, `Points`, `Relief`, `Portals` | voir la partie 7 | Dans `zoom:`, `points:`, `relief:`, `portals:` d'une `Page` |

## 10 bis. Chaque notion et son mot

La liste de tous les mots, face à ceux de HTML, CSS et JavaScript : [`NOMS.md`](NOMS.md).

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
| Activer les points au zoom | `points: Points()` | fait |
| Le pixel qui devient un point | `Points(after:, size:)` | fait |
| Le morcellement des points d'une page | `Points(fragment:, grid:, depth:)` | fait |
| Le relief | `Relief(height:)` | fait |
| Activer la rotation de la page, en faire le tour | `Relief(tilt:)` | fait |
| Une valeur que la page retient | `state: State(cart: 0)` | fait, à l'essai |
| Afficher une valeur | `{cart}` dans un texte | fait, à l'essai |
| Changer une valeur | les demandes `add`, `sub`, `set` | fait |
| Recevoir des valeurs d'un serveur | `data: Data(from: "stock.json", every: 30s)` | fait, à l'essai |
| Réutiliser un morceau de page et un thème | `import "commun.holo"`, `Part(name:)`, `Use(Menu)` | fait, à l'essai |
| Répéter une règle dans le temps | `Every(1s, effect:)` | fait |
| Le clavier | `On(Key.left, effect:)` | fait, à l'essai |
| Agir au moment où une valeur atteint quelque chose | `When(lives, is: 0, effect:)` | fait, à l'essai |
| La rencontre de deux objets | `When(Basket, meets: Apple, within:, effect:)` | fait, à l'essai |
| Faire glisser un objet | `drag: true` sur un bloc d'un `Board` | fait, à l'essai |
| Plusieurs demandes dans une règle | `effect: [a.add(1), b.set(0)]` | fait, à l'essai |
| Le hasard | la demande `random` | fait |
| Placer librement | `Board`, et `x:`, `y:` sur ses enfants | fait |
| Une valeur qui est un texte | `State(buyer: "")`, `{buyer}`, `If(buyer, not: "")` | fait, à l'essai |
| Écrire un texte | `Input(value: buyer, label:, max:)` | fait, à l'essai |
| Écrire un nombre, cocher une case | `Input(value:, label:, max:)`, `Checkbox(value:, label:)` | fait, à l'essai |
| Garder une valeur d'une visite à l'autre | `keep: [cart]` | fait, à l'essai |
| Montrer ou cacher selon une valeur | `If(cart, is:, not:, over:, under:)` | fait, à l'essai |
| Un trait, une citation, du texte tel quel | `Hr()`, `Quote(by:)`, `Code`, les accents graves | fait, à l'essai |
| Le retour à la ligne | un texte entre trois guillemets | fait, à l'essai |
| Le texte qui remplace une image | `Image(alt:)` | fait, à l'essai |
| Des prix, un nombre d'articles, un total | `prices: Prices(...)`, `{count}`, `{total}` | fait, à l'essai |
| Activer ou désactiver le zoom | `Zoom(active:)` | fait |
| Les limites du zoom | `Zoom(max:, shrink:)` | fait |
| La vitesse du zoom | `Zoom(speed:)` | fait |
| Le nombre de sites emboîtés | `Zoom(levels:)` | fait |
| Entrer dans un site, en sortir | `enter`, `leave` | fait |
| Le carrefour, les portails | `Portals(layout:, count:, size:, brightness:)`, et la capacité `portals` | fait |
| La durée d'ouverture d'un portail | `Portals(duration:)` | fait |
| Le poids permis | `budget`, `weight` | fait |
| Le toucher | le signal `tap` | fait |
| Le son | `Sound(name:, source:)`, et la capacité `play` | fait, à l'essai |
| La vidéo | aucun | à faire |
| Le survol, l'approche | aucun | à faire |
| Réagir au zoom par une règle (« quand on zoome, alors… ») | aucun | à faire |
| Ranger côte à côte, l'un sous l'autre, en grille | `Row`, `Column`, `Grid` | fait, à l'essai |
| L'écart et le placement | `gap:`, `align:`, `columns:` | fait, à l'essai |
| Réutiliser un morceau de fichier (les imports) | `import` est lu, pas appliqué | à faire |
| Les formulaires, les valeurs qui changent | aucun | à faire |
| Le personnage | aucun | à faire |

L'exemple le plus complet : [`exemples/boutique-comparee/boutique.holo`](../../exemples/boutique-comparee/boutique.holo).

## 11. Ce qui n'existe pas encore

- `module`, `bridge js`, `bridge css` : le moteur les lit mais ne les applique pas. Les listes, et l'envoi de données à un serveur.
- Pour la disposition : pas de largeur par élément, pas d'élément qui prend la place restante.
- La liste de choix, l'envoi d'un formulaire ; les données venues d'ailleurs.
- Pour les valeurs : des nombres entiers et des textes. Pas de liste, pas d'autre calcul que le nombre et le total d'un panier, pas de comparaison entre deux valeurs.
- Le reste du Markdown (seuls le gras et l'italique sont rendus).
- Entrer dans un point écrit à l'intérieur d'un monde.
- Les garde-fous de zoom pour un `Point` seul : ils sont encore fixés dans le moteur.
- Un seul nom de style par bloc ; pas de fichier de styles à part.
