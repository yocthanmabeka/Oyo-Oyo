# Écrire en `.holo` : le guide de l'auteur

Ce guide montre comment écrire un fichier `.holo` aujourd'hui, avec ce que le moteur sait vraiment faire. Tous les exemples entre balises `holo` sont relus par un test du moteur à chaque changement : s'ils cessaient de marcher, le test échouerait.

- État : langage en construction (2026-10-03). Ce qui n'existe pas encore est listé à la fin.
- Ce que HoloCode couvre et ne couvre pas du web classique, balise par balise : [`COMPARAISON-WEB.md`](COMPARAISON-WEB.md).
- Les décisions derrière chaque règle : [`docs/02-gouvernance/DECISIONS.md`](../02-gouvernance/DECISIONS.md).

> Pour apprendre pas à pas : [les leçons](../../exemples/lecons/README.md), une notion par fichier. Ce guide est la référence complète.

## 1. Voir ce qu'on écrit

1. Lancer le serveur local, une fois, dans un terminal : `node moteur/outils/server.mjs`
2. Ranger son fichier dans `exemples/` (une page) ou dans `moteur/mondes/` (un monde seul).
3. Dans VS Code, avec l'extension HoloCode ([`outils/vscode-holocode/`](../../outils/vscode-holocode/README.md)) : ouvrir le fichier et cliquer sur ▶ en haut à droite (ou `Ctrl+Alt+H`). Le fichier s'ouvre dans Chrome, à sa propre adresse.

Sans l'extension : ouvrir `http://localhost:8080/exemples/mon-dossier/ma-page.holo` dans Chrome.

**L'éditeur** (`ADR-046`), sur le PC et sur le téléphone : `http://localhost:8080/editor?key=…` (le serveur affiche l'adresse exacte, avec sa clé, à son démarrage). Le texte à gauche, la page à droite, mise à jour pendant qu'on écrit ; la faute soulignée à sa place, et, quand le moteur dit le bon mot, un bouton **« Remplacer « h1 » par « H1 » »** ; en bas, les mots du langage à toucher, pour ne pas taper de majuscule au milieu d'un mot sur un téléphone. Dans VS Code, l'extension (version 0.2.0) souligne la même faute et propose la même correction dans l'ampoule (`Ctrl+.`).

Si le fichier contient une erreur, la page affiche le message du moteur, avec la ligne et la colonne.

**Les outils de l'auteur** (`ADR-054`) :

- **Voir les valeurs** : ajouter `?values` à l'adresse (`…/ma-page.holo?values`). Un petit panneau, en bas à gauche, montre les valeurs de la page (`cart = 18000`, une liste et ses éléments), puis le dernier geste et ce qu'il a changé (`cart : 12000 → 13000`).
- **Remettre en forme** : `moteur/target/release/holo fmt ma-page.holo`. Deux espaces de plus après une ligne qui ouvre, deux de moins quand elle se referme ; seuls les blancs changent.
- **Des essais écrits** : un fichier `ma-page.test` à côté de la page, puis `holo test ma-page.holo ma-page.test`. Une ligne par geste ou par vérification :

```text
// Ajouter deux fois le tableau « Night » remplit le panier.
tap AddNight
tap AddNight
expect cart = 12000
type name "Forest"
expect name = "Forest"
receive {"articles": []}
expect articles = 0
```

`tap Nom` touche un bouton ; `signal Nom.hover` envoie un autre signal ; `type valeur "texte"` écrit dans un champ ; `receive {…}` fait comme si le serveur envoyait ces données ; `expect valeur = …` vérifie un nombre, un texte entre guillemets, ou le nombre d'éléments d'une liste. Les essais rangés dans `exemples/lecons/` sont joués par les tests du moteur.

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

## 2 bis. Écrire les noms : l'écriture de Flutter

HoloCode fait la différence entre une majuscule et une minuscule : `Page` et `page` ne sont pas le même mot. Chaque mot a **une seule** écriture, celle de Flutter : deux mots se joignent par une majuscule, jamais par `_`.

| Sorte de mot | Écriture | Exemple |
|---|---|---|
| Bloc, et nom donné à un bloc | une majuscule au début et à chaque mot | `Button`, `BlueDoor`, `name: AddSunrise` |
| Paramètre, mot-valeur, signal, demande | une minuscule au début, une majuscule à chaque mot suivant | `title`, `topRight`, `tap`, `add` |
| Nom de valeur | pareil | `cart`, `appleX`, `blueDoor` |
| Style | comme en CSS : minuscules, mots joints par `-` | `font-size`, `.carte` |
| Unité d'octets | majuscules (B = octet, b = bit) | `KB`, `MB` |

- **Ce qu'on touche a une majuscule, ce qui change n'en a pas** : dans `On(AddSunrise.tap, effect: sunrise.add(1))`, on voit d'un coup d'œil le bouton et la valeur.
- Une faute n'est jamais avalée : le moteur la refuse et donne le bon mot. `apple_x` → « écris `appleX` » ; `Page(Title: …)` → « écris `title` » ; `name: buy` → « écris `name: Buy` ».
- Les styles gardent l'écriture du CSS, pour qu'on n'ait pas à réapprendre le CSS.

Décidé par Yocthan (`ADR-037`). La leçon est `39-ecrire-les-noms.holo`.

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
| `H1` à `H6` | Un titre. Le numéro dit sa place dans le plan, jamais sa taille. |
| `P`, ou une phrase nue | Un paragraphe. |
| `Text` | Du texte sans rôle : une étiquette, une ligne d'état. |

Règles :

- Le premier titre est `H1`. On ne saute pas de niveau : `H3` ne suit pas `H1`.
- La taille d'un titre se règle par le style, pas en changeant de numéro.
- Un paramètre inconnu ou mal écrit est refusé avec le bon mot : `Page(Title: …)` → « écris `title` ». Un nom de bloc commence par une majuscule : `name: Ajouter`, jamais `name: ajouter`.
- Les titres vont jusqu'à `H6`, pour les longs documents (leçon 36).
- Dans un texte, `**gras**` et `*italique*`.

## 4. Images, listes, boutons

```holo
Page(
  title: "My shop",
  children: [
    H1("My shop"),
    Image(source: "painting.svg", weight: 1KB, alt: "A painting: a sun over the hills"),
    List(children: [
      "Sunrise over the river",
      "The blue door",
    ]),
    Button(name: Open, text: "Enter the workshop"),
  ],
)
```

- `Image(source:, alt:)` : un fichier rangé à côté du `.holo`. Ni adresse complète, ni `../`. `alt` est **obligatoire** : ce que montre l'image, pour qui ne la voit pas ; pour un simple décor, `alt: ""`. `weight` est son poids déclaré.
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

Cette écriture est décidée (`ADR-024`).

### La place qui reste : `grow`

Dans `Row` ou `Column`, un bloc qui porte **`grow: 1`** prend la place qui reste, comme `Expanded` en Flutter. Avec `grow: 2` à côté d'un `grow: 1`, il en prend deux parts. Un bloc sans `grow` garde sa taille. Un champ de saisie qui grandit s'étire jusqu'au bout.

```holo
Page(
  state: State(search: ""),
  children: [
    H1("Shop"),
    Row(gap: 8px, children: [
      Input(value: search, label: "Search", grow: 1),
      Button(name: Go, text: "Go"),
    ]),
  ],
)
```

Pour une largeur précise, le style `width` : `.narrow { width: 80px; }`. La leçon est `72-place-et-theme.holo` ; cette écriture est décidée (`ADR-052`).

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

**Un fichier de styles à part** (`ADR-052`) : un fichier `.holo` qui ne contient que des styles est un thème, partagé par toutes les pages du site. Une page le prend par `import "theme.holo"`, en haut du fichier ; si elle écrit le même style, c'est le sien qui reste.

**Plusieurs noms de style** sur un bloc : `Text.title.muted("…")`, quatre au plus (`ADR-050`). Un nom de style s'écrit en minuscules.

Réglages connus :

| Réglage | Valeur |
|---|---|
| `color` | Une couleur : `white`, `gold`, `#E9B44C` |
| `background` | Une couleur ; un dégradé, `linear-gradient(to right, #E9B44C, #1a1a2e)` ou `radial-gradient(white, navy)` ; ou une image rangée à côté, `url("fond.jpg")`, qui couvre le bloc |
| `font-size`, `border-radius`, `width`, `height`, `max-width` | Une taille : `16px`, `50%` |
| `padding`, `margin` | D'une à quatre tailles : `8px 16px` |
| `font-weight` | `normal`, `bold` |
| `font-style` | `normal`, `italic` |
| `font-family` | Un ou plusieurs noms de police |
| `text-align` | `left`, `center`, `right` |
| `border` | `1px solid gray` |
| `opacity` | Un nombre de 0 à 1 |
| `line-height` | Un nombre sans unité, de 0.8 à 3 : `1.6` |
| `letter-spacing` | Un écart de -10px à 40px : `2px` |
| `text-transform` | `uppercase`, `lowercase`, `capitalize`, `none` |
| `text-decoration` | `underline`, `line-through`, `none` |
| `box-shadow`, `text-shadow` | Décalage, flou, couleur : `0 4px 12px #00000066` ; trois au plus ; ou `none` |
| `rotate`, `scale` | Une pose : `-3deg` ; `1.05` |
| `transition` | La durée du passage d'une allure à l'autre : `0.3s` ; ou `none` |

**Les variables.** Une couleur ou une taille nommée une fois, dans le style de `Page`, puis employée partout, sans `var( )` :

```holo
Page(children: [ H1("Sunrise"), P.card("Hand-painted.") ])

Page { --gold: #E9B44C; --ink: #1a1a2e; background: white; color: --ink; dark: { --ink: #F5F5F5; background: #101020; } }
H1 { color: --gold; font-size: 40px; phone: { font-size: 28px; } }
.card { border: 1px solid --gold; box-shadow: 0 8px 24px #00000040; }
```

**Les états** d'un style : `hover: { … }` (la souris), `focus: { … }` (le clavier), `active: { … }` (l'appui), `dark: { … }` (le visiteur a choisi le thème sombre), `phone: { … }` (un écran plus étroit que la page). Dans `phone:`, et seulement là, `display: none;` cache un bloc.

**Sa propre police** : `Page(fonts: [ Font(family: "Carlito", source: "carlito.woff2") ])`, puis `font-family: Carlito, Georgia, serif;`. Le texte s'affiche tout de suite avec la police de secours.

**Une police du moteur**, sans fichier : `Page(fonts: [ Font(family: "Inter") ])`, puis `font-family: Inter, sans-serif;`. Le moteur garde 32 polices libres pour les écritures du monde entier (le latin, le cyrillique, le grec, l'arabe, l'hébreu, le devanagari, le bengali, le tamoul, le thaï, l'éthiopien, l'adlam, le n'ko, le tifinagh, le chinois, le japonais, le coréen) ; le navigateur ne télécharge que les morceaux dont la page a besoin. La liste est dans `moteur/web/fonts/README.md` ; cette écriture est proposée (`ADR-092`), la leçon est `115-des-polices-pour-toutes-les-ecritures.holo`.

Ces ajouts sont décidés (`ADR-041`). Les leçons sont `50-texte-soigne.holo` à `54-police.holo`.

Ce que le moteur refuse, alors que le CSS le laisse passer :

- un réglage inconnu ou mal orthographié ;
- un `;` oublié en fin de ligne ;
- un style défini deux fois ;
- un nom posé sur un bloc (`P.card`) sans style `.card` ;
- un réglage de disposition (`display`, `position`, `float`) : un style ne dit que l'apparence (seule exception : `display: none` dans `phone:`) ;
- une hauteur de ligne en pixels (elle ne suivrait pas le texte grossi), une variable jamais définie, une image de fond hors du dossier ;
- un sélecteur composé (`.card P { ... }`).
- un texte trop peu contrasté sur son fond, quand le même style donne les deux couleurs : il faut 4,5 pour 1 au moins (3 pour 1 pour un grand texte), pour qu'il soit lu par tous (`ADR-055`).

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
          Image(source: "easel.svg", weight: 1KB, alt: "An easel"),
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
- Capacités : `enter` et `leave` (pour un `Point`), `portals` (pour la `Page` : ouvrir son carrefour), `play` et `stop` (pour un `Sound`).
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

- Une valeur est un nombre, de 0 à 1 000 000 000 : entier, ou à virgule (`price: 12.50`, voir « Des nombres à virgule »). Une page en déclare au plus 32.
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

  state: State(sunrise: 0, blueDoor: 0),
  prices: Prices(sunrise: 120, blueDoor: 90),

  children: [
    Text("Sunrise over the river, 120 euros. {sunrise} in your cart."),
    Button(name: AddSunrise, text: "Add"),
    Text("The blue door, 90 euros. {blueDoor} in your cart."),
    Button(name: AddBlueDoor, text: "Add"),

    Text("{count} paintings, {total} euros"),
    Button(name: Empty, text: "Empty the cart"),
  ],

  rules: [
    On(AddSunrise.tap, effect: sunrise.add(1)),
    On(AddBlueDoor.tap, effect: blueDoor.add(1)),
    On(Empty.tap, effect: sunrise.set(0)),
    On(Empty.tap, effect: blueDoor.set(0)),
  ],
)
```

- Dans `State`, chaque valeur est la quantité d'un article.
- Dans `Prices`, chaque prix porte le nom d'une valeur : c'est le prix de cet article. Un prix est un nombre entier.
- Une valeur sans prix (des « j'aime », par exemple) ne compte ni dans `{count}` ni dans `{total}`.
- Avec `prices:`, les noms `count` et `total` sont pris par le moteur : on ne les déclare pas dans `State`, et on ne peut pas les changer par une demande.
- Un même signal peut avoir plusieurs règles : `Empty.tap` remet ici deux valeurs à zéro.

Limites : les articles sont écrits d'avance dans le fichier ; pas de centimes ; et ce panier est un affichage, pas une commande : rien n'est envoyé à un serveur.

Cette écriture est décidée (`ADR-023`).

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

- Le premier mot est le nom d'une valeur : une valeur de `State`, ou `count` et `total` quand la page donne des prix. Une valeur de texte se compare à un texte : `If(size, is: "L")` (voir « Comparer des textes »).
- Plusieurs comparaisons valent ensemble : `over: 2, under: 10` veut dire « de 3 à 9 ».
- Quand la valeur change, la page suit toute seule.
- Le « sinon » s'écrit `else: [ … ]` (`ADR-039`) : ce qui se montre quand la condition est fausse.
- Une condition se place dans `children`, y compris dans une ligne, une colonne ou une grille.

Cette écriture est décidée (`ADR-025`).

## 6 quater. Un jeu : le temps, le hasard, le plateau

Trois mots suffisent pour un premier jeu. Le jeu entier est dans `exemples/jeu/attraper.holo` : on touche une étoile le plus de fois possible en trente secondes.

```holo
Page(
  name: Catch,
  title: "Catch the star",

  state: State(time: 0, score: 0, starX: 50, starY: 50),

  children: [
    If(time, is: 0, children: [
      Button(name: Play, text: "Play"),
    ]),
    If(time, over: 0, children: [
      Text("Score: {score}. Time: {time} s"),
      Board(height: 320px, children: [
        Point(name: Star, seed: 7, x: starX, y: starY),
      ]),
    ]),
  ],

  rules: [
    On(Play.tap, effect: time.set(30)),
    On(Star.tap, effect: [score.add(1), starX.random(100), starY.random(100)]),
    Every(1s, effect: time.sub(1)),
    Every(2s, effect: [starX.random(100), starY.random(100)]),
  ],
)
```

**Le temps : `Every`.** Une règle qui se répète. `Every(1s, effect: time.sub(1))` : toutes les secondes, retirer 1 à `time`.

- Le rythme s'écrit en `s` ou en `ms`, de `100ms` à `3600s`.
- L'effet est une demande, comme pour `On`.
- L'horloge se tait quand la fenêtre est cachée, en vue points et devant le carrefour.
- Chaque règle `Every` a sa propre horloge. Quand un geste change une valeur, l'horloge de cette valeur repart de zéro : après `On(Play.tap, effect: time.set(30))`, la première seconde dure une vraie seconde.

**Le hasard : `random`.** `starX.random(100)` donne à `starX` un nombre de 0 à 100, bornes comprises. Ce hasard est rejouable : les mêmes gestes, aux mêmes moments, redonnent la même partie.

**Le plateau : `Board`.** Ce qu'il contient se place où l'on veut.

| Réglage | Sens | Valeurs |
|---|---|---|
| `Board(height:)` | La hauteur du plateau, pour 640 de large. | 80px à 800px ; 320px sans rien écrire |
| `x:` sur un bloc du plateau | Sa place de gauche à droite. | un nombre de 0 à 100, ou le nom d'une valeur |
| `y:` sur un bloc du plateau | Sa place de haut en bas. | un nombre de 0 à 100, ou le nom d'une valeur |

- Avec un nombre, le bloc reste à sa place. Avec le nom d'une valeur, il suit cette valeur quand elle change.
- De 0 à 100, un bloc ne sort jamais du plateau.

**Commencer et finir sans mot de plus.** Le temps ne descend pas sous zéro. Les conditions font le reste : `If(time, is: 0)` montre le bouton « Play » et le score, `If(time, over: 0)` montre le plateau.

Cette écriture est décidée (`ADR-026`).

## 6 quater bis. Un jeu qui bouge : le clavier, le glissement, `When`

Le second jeu est dans `exemples/jeu/panier.holo` : une pomme tombe, on la rattrape avec un panier.

```holo
Page(
  name: Orchard,
  title: "Catch the apple",

  state: State(lives: 3, score: 0, basket: 50, appleX: 50, appleY: 0),

  children: [
    Text("Score: {score}. Lives: {lives}"),
    Board(height: 360px, children: [
      Point(name: Apple, seed: 3, x: appleX, y: appleY),
      Point(name: Basket, seed: 9, x: basket, y: 96, drag: true),
    ]),
  ],

  rules: [
    On(Key.left, effect: basket.sub(8)),
    On(Key.right, effect: basket.add(8)),

    Every(100ms, effect: appleY.add(3)),

    When(Basket, meets: Apple, effect: [score.add(1), appleX.random(100), appleY.set(0)]),
    When(appleY, over: 99, effect: [lives.sub(1), appleX.random(100), appleY.set(0)]),
  ],
)
```

**Trois sortes de règles, pas plus.**

| Règle | Elle répond à | Exemple |
|---|---|---|
| `On` | un geste du visiteur | `On(Play.tap, …)`, `On(Key.left, …)` |
| `Every` | le temps | `Every(100ms, …)` |
| `When` | un moment : quelque chose devient vrai | `When(appleY, over: 99, …)`, `When(Basket, meets: Apple, …)` |

**Plusieurs demandes dans une règle.** On les met entre crochets : `effect: [score.add(1), appleY.set(0)]`. Elles sont faites dans l'ordre. Une seule demande s'écrit sans crochets.

**Le clavier : `Key`.** `On(Key.left, effect: basket.sub(8))`. `Key` est le clavier du visiteur.

| Signal | Touche |
|---|---|
| `Key.left`, `Key.right`, `Key.up`, `Key.down` | les quatre flèches |
| `Key.space` | la barre d'espace |
| `Key.enter`, `Key.escape` | Entrée, Échap |
| `Key.a` à `Key.z` | une lettre : celle écrite sur la touche |
| `Key.digit0` à `Key.digit9` | un chiffre : rangée du haut ou pavé numérique, avec ou sans Maj (un clavier français marche sans Maj) |

Seules les touches que le fichier écoute sont prises. Les autres gardent leur rôle. Jamais Tab : elle sert à passer d'un bouton à l'autre. Une page qui écoute des lettres ou des chiffres ajoute au menu ☰ « Touches à une lettre » : le visiteur peut les couper, pour qu'un logiciel de dictée ne les tape pas sans le vouloir (`ADR-061`).

Un plateau garde ses proportions : 640 de large, `height` de haut. Sur un téléphone il rétrécit, et tout ce qu'il contient avec lui ; sur un grand écran il ne dépasse pas la largeur de la page, ni les quatre cinquièmes de la hauteur de l'écran. Une partie se joue donc pareil partout : la pomme touche le panier au même moment sur un téléphone et sur un ordinateur.

**Faire glisser : `drag: true`.** Sur un bloc posé dans un `Board`, le visiteur peut le faire glisser, au doigt ou à la souris. Ses places, quand ce sont des valeurs de la page, suivent le doigt. Ici `basket` suit ; `y: 96` est un nombre fixe, donc le panier ne monte pas. Aucune règle à écrire.

**Ce qui bouge tout seul.** `Every(100ms, effect: appleY.add(3))` : dix fois par seconde, la pomme descend un peu.

**Une valeur qui sert de place reste sur le plateau** : de 0 à 100. Le panier ne sort jamais.

**Une règle qui guette : `When`.** Elle se déclenche au moment où ce qu'elle guette **devient** vrai, pas tant qu'il le reste. Elle guette :

- une valeur : `When(appleY, over: 99, effect: …)`, avec les comparaisons de `If` (`is`, `not`, `over`, `under`) ; ou un texte : `When(answer, is: "Paris", effect: …)` (`ADR-063`) ;
- une rencontre : `When(Basket, meets: Apple, effect: …)`. Les deux blocs ont un nom, et un `x` et un `y` dans un `Board`. Ils se rencontrent au moment où le bord de l'un touche le bord de l'autre : un rond et un carré se touchent comme à l'œil. Avec `within: 20`, on juge autrement : sur l'écart entre leurs places, de 1 à 100, sans regarder leur taille. L'écart est mesuré sur chaque axe : `x` à 20 près **et** `y` à 20 près (un carré autour de l'objet, pas un cercle).

**Des règles sous condition.** Une règle de temps tourne tant que la page est ouverte. Pour qu'elle ne vaille que pendant la partie, on la range sous une condition, avec le même `If` que pour montrer des blocs, et `rules` à la place de `children` :

```holo
Page(
  title: "A stopwatch",
  state: State(running: 0, seconds: 0),
  children: [
    Text("{seconds} seconds"),
    Button(name: Start, text: "Start"),
    Button(name: Stop, text: "Stop"),
  ],
  rules: [
    On(Start.tap, effect: [seconds.set(0), running.set(1)]),
    On(Stop.tap, effect: running.set(0)),
    If(running, is: 1, rules: [
      Every(1s, effect: seconds.add(1)),
      When(seconds, is: 10, effect: running.set(0)),
    ]),
  ],
)
```

- Les règles rangées dans `If(…, rules: [ … ])` ne valent que si la condition est vraie.
- On y range des règles de temps (`Every`) et des règles qui guettent (`When`). Une règle `On` répond à un geste : pour elle, on cache le bouton.
- Dans le jeu de la pomme, tout ce qui fait tomber, rattraper et perdre est rangé sous `If(lives, over: 0, rules: [ … ])` : avant « Play » et après la fin, rien ne bouge et aucun son ne part.

Cette écriture est décidée (`ADR-028`).

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
- Un texte se compare au vide, `is: ""` (il est vide) et `not: ""` (il est rempli), ou à n'importe quel texte : voir « Comparer des textes » (`ADR-063`).
- Un texte ne change que par un champ : il n'y a pas de demande pour lui.
- Ce que le visiteur écrit ne devient jamais du code : la page le montre lettre pour lettre.

Limite : ce qui est gardé reste sur cet appareil. (Une liste de choix : `Choice`, `ADR-038` ; un envoi au serveur : `Form`, `ADR-042`.)

Cette écriture est décidée (`ADR-027`).

## 6 sexies. Un site de plusieurs pages : `import`, `Component`, `Use`

Le menu et le thème d'un site s'écrivent une seule fois, dans un fichier à part. L'exemple est dans `exemples/site/`.

Le fichier commun, `commun.holo`, est un **composant** (un morceau, sans paramètres) :

```holo
Component(
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
- `Component(name: Menu, children: [...])` : un morceau a un nom et un contenu. Ce n'est pas une page.
- `Use(Menu)` pose les blocs du morceau à cet endroit.
- Les styles du morceau viennent avec lui. Si la page écrit le même style, c'est le sien qui reste.
- Un bouton écrit dans un morceau garde son nom : une règle de la page peut l'écouter.

Limites : un morceau n'a ni valeurs ni règles, et n'importe pas d'autres fichiers ; seize imports au plus.

Cette écriture est décidée (`ADR-029`).

## 6 sexies bis. Les composants : écrire un bloc une fois, le poser partout

Un composant est un bloc qu'on écrit soi-même, une fois, avec des **paramètres**, puis qu'on pose comme n'importe quel bloc, à la manière d'un widget Flutter. Sa différence avec Flutter : son apparence se change **par le CSS**, de l'extérieur, sans toucher au composant. La leçon est `exemples/lecons/70-composants.holo`.

```holo
Page(
  title: "Shop",
  state: State(cart: 0, sunrise: 0, night: 0),
  components: [
    Component(
      name: ArticleCard,
      params: [title, price, qty],
      children: [
        Column(gap: 8px, children: [
          H2("{title}"),
          Text("{price:cents} euros, {qty} in the cart"),
          Button(name: Add, text: "Add"),
        ]),
      ],
      rules: [ On(Add.tap, effect: [qty.add(1), cart.add(price)]) ],
    ),
  ],
  children: [
    H1("Shop"),
    ArticleCard(name: Sunrise, title: "Sunrise", price: 12000, qty: sunrise),
    ArticleCard.promo(name: Night, title: "Night", price: 6000, qty: night),
    P("Total: {cart:cents} euros."),
  ],
)

Page { --accent: #E9B44C; }
ArticleCard { border: 1px solid --accent; border-radius: 12px; padding: 12px 16px; }
.promo { --accent: crimson; }
```

**Écrire le composant.**

- `components: [ Component(…) ]` dans la page, ou un fichier importé qui commence par `Component(…)` (§ 6 sexies). L'ancien mot `Part` est refusé avec « écris Component » : il est gardé pour la 3D (`ADR-056`).
- `name:` son nom, comme un bloc : une majuscule au début et à chaque mot (`ArticleCard`). Un mot du langage (`Text`, `Button`…) est refusé.
- `params:` ses paramètres, en minuscules (`title`, `oldPrice`) : ce qui change d'une copie à l'autre. Seize au plus. Un paramètre ne peut pas porter le nom d'une valeur de la page, ni un mot du langage.
- `children:` son contenu : **un seul bloc racine**, comme le widget que rend Flutter. Pour plusieurs blocs, range-les dans `Column(children: [ … ])`.
- `rules:` ses règles : elles sont écrites une fois pour chaque copie.

**Employer un paramètre**, dans le composant :

- dans un texte : `"{title}"`, et avec un format pour un nombre : `"{price:cents}"` ;
- à la place d'une valeur : `Image(source: image, alt: title)` ;
- donné par le **nom d'une valeur de la page** (`qty: sunrise`), il la suit : `{qty}` montre la valeur, et `qty.add(1)` la change. C'est ainsi qu'un composant agit sur la page : son bouton remplit le panier.

**Poser une copie** : `ArticleCard(name: Sunrise, title: "Sunrise", price: 12000, qty: sunrise)`.

- Tous les paramètres sont donnés, chacun nommé ; un paramètre mal écrit est refusé avec le bon mot.
- `name:` donne un nom à la copie : le bouton `Add` du composant devient `AddSunrise`, qu'une règle de la page peut écouter (`On(AddSunrise.tap, …)`). Si le composant nomme des blocs, chaque copie doit avoir son nom (une seule peut s'en passer).
- Un composant peut en poser un autre, mais jamais lui-même ; huit niveaux au plus.
- Dans une répétition, on lui donne les champs de l'élément : `Repeat(items: [ … ], children: [ ArticleCard(title: item.title, price: item.price, qty: item) ])`.

**Des valeurs par défaut** (`ADR-056`) : `params: [title, price: 0, image: "placeholder.svg"]`. Un paramètre qui a une valeur par défaut peut être oublié à l'appel.

**Des signaux, que la page branche** (`ADR-056`) : le composant déclare ce qu'il émet, `emits: [add]`, et sa règle émet au lieu d'agir, `On(Add.tap, emit: add)`. À l'appel, la page décide : `ArticleCard(title: "Sunrise", onAdd: cart.add(12000))`, ou une liste de demandes, `onAdd: [cart.add(1000), likes.add(1)]`. Un signal que la page ne branche pas ne fait rien. La leçon est `73-defauts-et-signaux.holo`.

**Un emplacement pour du contenu** (`ADR-058`) : comme le `children` d'un widget Flutter, le composant laisse une place, et la page la remplit à l'appel.

```holo
Page(
  title: "Panels",
  components: [
    Component(name: Panel, params: [title, children], children: [
      Column(children: [ H2("{title}"), children ]),
    ]),
  ],
  children: [
    H1("Panels"),
    Panel(title: "Opening hours", children: [ P("Tuesday to Saturday."), P("Free entry.") ]),
    Panel(title: "Empty"),
  ],
)
```

- `params: [title, children]` déclare l'emplacement ; le mot `children` posé seul dans une liste du composant marque où va le contenu.
- À l'appel, `children: [ … ]` est une liste de blocs ; sans elle, l'emplacement reste vide.
- Un seul mot, `children`, même pour un seul bloc : `child` est refusé avec le bon mot.
- Le contenu appartient à la page : ses noms ne sont pas changés, ses règles sont celles de la page ; il peut contenir d'autres composants, et le même.

**Changer son apparence, de l'extérieur.**

| On écrit | Ce qui change |
|---|---|
| `ArticleCard { … }` | toutes les copies |
| `ArticleCard.promo(…)` puis `.promo { … }` | cette copie seulement |
| `--accent` employé dans le style du composant, puis `.promo { --accent: crimson; }` | une seule valeur, sans réécrire le style |
| `P.card.big(…)` | plusieurs noms de style sur un bloc (quatre au plus) |

Une variable se définit dans le thème (`Page { --accent: … }`) pour valoir partout, ou dans le style d'un composant ou d'un nom pour valoir sur ce bloc et ce qu'il contient.

**Ce qui est partagé.** Les noms de style d'un fichier importé (`.card`) valent pour toute la page, comme un thème ; deux fichiers importés qui écrivent le même style sont refusés. Pour qu'un composant ne partage rien, style-le par son nom : `ArticleCard { … }` ne vise que ses copies.

Ce que le moteur fabrique : du vrai HTML, la racine de chaque copie portant la classe du composant et celles de ses noms de style ; et du vrai CSS. Rien n'est envoyé en JavaScript.

Cette écriture est décidée (`ADR-050`).

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
| `Data(name:)` | Un nom, pour que les règles sachent si les données sont arrivées. | un nom de bloc : `Shop` |

- Chaque nom du fichier remplit la valeur de `State` du même nom : un nombre entier dans un nombre, un texte dans un texte.
- Ce qui ne correspond à rien est laissé de côté. Un fichier mal écrit ne change rien, et la page garde ses valeurs.
- La page ne parle qu'au serveur d'où elle vient.
- Rien n'est demandé quand la fenêtre est cachée.

Une liste se reçoit aussi : un tableau d'objets remplit une liste à champs (§ 6 septendecies, `ADR-051`). Limite : la page n'envoie rien au serveur, sauf par un formulaire (`Form`).

Cette écriture est décidée (`ADR-030`).

**Arrivées, ou pas.** Des données qui ont un nom disent ce qui leur est arrivé, et se relisent :

```holo
Page(
  title: "Shop news",
  state: State(loading: 1, broken: 0, news: ""),
  data: Data(name: Shop, from: "shop.json"),
  children: [
    If(loading, is: 1, children: [ "Loading…" ]),
    If(broken, is: 1, children: [ "The news did not arrive.", Button(name: Retry, text: "Try again") ]),
    Text("{news}"),
  ],
  rules: [
    On(Shop.done, effect: [loading.set(0), broken.set(0)]),
    On(Shop.failed, effect: [loading.set(0), broken.set(1)]),
    On(Retry.tap, effect: [loading.set(1), broken.set(0), Shop.refresh]),
  ],
)
```

- `Shop.done` : les données sont arrivées et rangées dans les valeurs.
- `Shop.failed` : elles ne sont pas arrivées, faute de réseau, sur une erreur du serveur, ou parce que le fichier est trop gros (plus de 64 Ko), illisible (autre chose qu'un objet JSON) ou trop lent (plus de 10 secondes).
- `Shop.refresh` les relit. Une seule lecture à la fois : pendant une lecture, la lecture en cours répondra. Une seconde au moins entre deux lectures : une demande trop proche attend son tour, elle n'est pas perdue.
- « Loading… » est une valeur de la page, à 1 au départ, que les deux signaux remettent à 0.
- Sans `name`, rien ne change : la page lit ses données sans rien dire.
- **La page arrive déjà avec ses données.** Le serveur lit le fichier de `Data(from:)`, rangé à côté du `.holo`, et fabrique la page avec, puis `Shop.done`. Un robot de recherche, ou un visiteur dont le moteur tarde, voit donc les nouvelles tout de suite, sans « Loading… ». Le navigateur rejoue la même réception et part du même état, puis relit les données comme d'habitude. Un fichier absent, trop gros ou illisible : la page de départ.

Cette écriture est proposée (`ADR-064`) et attend la validation de Yocthan. La leçon est `84-donnees-arrivees-ou-pas.holo`.

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
- `volume: 0.4` : de 0 (muet) à 1 (le plus fort), comme l'opacité. `loop: true` : il recommence sans fin. `Rain.stop` l'arrête et le remet au début, aussi dans une règle `Every`, `After` ou `When` (`ADR-061`).

Cette écriture est décidée (`ADR-031`). La leçon est `exemples/lecons/28-son.holo`.

## 6 nonies. Des formes, et comparer deux valeurs

```holo
Page(
  title: "Best score",

  state: State(score: 0, best: 0),
  keep: [best],

  children: [
    Text("Score: {score}. Best: {best}."),
    Board(height: 200px, children: [
      Shape(name: Target, form: diamond, color: "#FF4D6D", size: 56px, x: 80, y: 30),
    ]),
    Button(name: Again, text: "Start again"),
  ],

  rules: [
    On(Target.tap, effect: score.add(1)),
    On(Again.tap, effect: score.set(0)),
    When(score, over: best, effect: best.set(score)),
  ],
)
```

**Une forme : `Shape`.**

| Réglage | Sens | Valeurs |
|---|---|---|
| `form` | La forme. | `circle`, `square`, `triangle`, `diamond` |
| `color` | Sa couleur, entre guillemets. | `"#E9B44C"` |
| `size` | Sa taille. | 8px à 400px ; 48px sans rien écrire |

- Une forme qui a un nom se touche, comme un bouton : `On(Target.tap, …)`.
- Elle se place sur un plateau comme un point : `x`, `y`, `drag`.

**Comparer à une autre valeur, fixer d'après une autre valeur.** Là où l'on écrit un nombre, on peut écrire le nom d'une valeur :

- `If(score, over: best, …)` et `When(score, over: best, …)` comparent deux valeurs ;
- `best.set(score)` donne à `best` la valeur de `score` ; `total.add(bonus)` ajoute `bonus`.

Le meilleur score tient alors en une règle : au moment où le score dépasse le meilleur, le meilleur devient ce score. Avec `keep: [best]`, il reste d'une visite à l'autre.

Cette écriture est décidée (`ADR-032`). Les leçons sont `29-comparer-deux-valeurs.holo` et `30-formes.holo`.

## 6 duodecies. La langue, la vidéo, le tableau, le texte long, le choix

```holo
Page(
  title: "Opening hours",
  lang: "en",
  description: "When the studio is open, and how to write to us.",
  image: "share.png",
  state: State(message: "", size: ""),
  children: [
    Video(source: "tour.mp4", label: "A walk through the studio"),
    Table(
      caption: "Opening hours",
      head: ["Day", "Hours"],
      rows: [ ["Monday", "9 am – 6 pm"], ["Friday", "closed"] ],
    ),
    Input(value: message, label: "Your message", lines: 5),
    Choice(value: size, label: "Size", options: ["S", "M", "L"]),
  ],
)
```

- **`lang:`, `description:`, `image:`** sur la `Page` : la langue (un lecteur d'écran prend la bonne voix), le texte que Google montre sous le titre, et l'image qu'on voit quand on partage le lien. Rien ne change à l'écran.
- **`Video(source:, label:)`** : une vidéo `.mp4` ou `.webm` rangée à côté, avec ses boutons. `label` est obligatoire. Elle ne démarre jamais toute seule.
- **`Table(caption:, head:, rows:)`** : un tableau de données. Chaque ligne a autant de cases que `head`. Sur un téléphone, il défile de côté sans déborder de la page.
- **`Input(…, lines: 5)`** : un texte long, sur plusieurs lignes ; sa valeur est un texte. Les retours à la ligne sont gardés ; 1000 caractères au plus sans `max`.
- **`Choice(value:, label:, options:)`** : un choix parmi des options, en boutons ronds ; `menu: true` en fait une liste déroulante. La valeur est un texte, et n'accepte que ses options.
- **`Image` demande maintenant toujours `alt`** : `alt: ""` pour un décor.

Ces ajouts sont décidés (`ADR-038`). Les leçons sont `40-langue-et-partage.holo` à `44-choix.holo`.

## 6 terdecies. Le survol qui agit, le « sinon », plus tard, l'heure

```holo
Page(
  title: "The studio",
  state: State(tip: 0, added: 0, cart: 0),
  children: [
    Column(name: Card, children: [
      H1("Sunrise over the river"),
      If(tip, is: 1, children: [ P("Delivered in three days.") ]),
    ]),
    If(cart, is: 0, children: [ P("Your cart is empty.") ], else: [ P("{cart} in your cart.") ]),
    Button(name: Add, text: "Add to cart"),
    If(added, is: 1, children: [ P("Added.") ]),
    If(hour, over: 8, under: 18,
      children: [ P("Open now: it is {hour} h {minute}.") ],
      else: [ P("Closed. We open at 9 am.") ],
    ),
  ],
  rules: [
    On(Card.hover, effect: tip.set(1)),
    On(Card.hoverEnd, effect: tip.set(0)),
    On(Add.tap, effect: [cart.add(1), added.set(1)]),
    If(added, is: 1, rules: [ After(3s, effect: added.set(0)) ]),
  ],
)
```

- **`On(Card.hover, …)` et `On(Card.hoverEnd, …)`** : la souris arrive sur un bloc nommé, puis le quitte. Le clavier y arrive aussi (Tab), et le doigt sur un téléphone : toucher le bloc le survole, toucher ailleurs le quitte. Un survol change des valeurs ou joue un son ; pour entrer dans un monde, il faut toucher. Pour changer seulement l'allure, un style suffit : `hover: { … }`.
- **`else: [ … ]`** dans un `If` : ce qu'on montre quand la condition est fausse.
- **`After(3s, effect: …)`** : une seule fois, plus tard. Dans les règles de la page, l'attente part à l'ouverture ; sous une condition, elle part quand la condition devient vraie. Ici, « Added. » s'efface trois secondes après l'ajout.
- **L'heure du visiteur** : `year`, `month`, `day`, `weekday` (1 lundi … 7 dimanche), `hour`, `minute`, `second`. On les montre et on les compare ; on ne les change pas. La page se tient à jour à chaque minute ; à chaque seconde si elle montre `{second}` (`ADR-089`).
- **Un chronomètre** (`ADR-089`) : `Stopwatch(name: Chrono, value: time, label: "My race")`, et `Chrono.start`, `Chrono.stop`, `Chrono.reset`. La page le fait tourner au centième ; le moteur reçoit le temps final, en millisecondes, dans `time`, puis `Chrono.stopped`. `{time:stopwatch}` l'écrit `01:23,45`. Leçon `111-un-chronometre.holo`.

Ces ajouts sont décidés (`ADR-039`). Les leçons sont `45-survol-qui-agit.holo` à `48-heure.holo`.

## 6 quaterdecies. Écrire une fois, répéter : `Repeat`

```holo
Page(
  title: "Shop",
  state: State(sunrise: 0, river: 0),
  prices: Prices(sunrise: 120, river: 90),
  children: [
    H1("Shop"),
    Grid(columns: 2, children: [
      Repeat(
        items: [
          Item(key: sunrise, title: "Sunrise", price: 120, image: "sunrise.png"),
          Item(key: river, title: "The river", price: 90, image: "river.png"),
        ],
        children: [
          Column(children: [
            Image(source: item.image, alt: ""),
            H2("{item.title}"),
            Text("{item.price} euros, {item} in the cart"),
            Button(name: Add, text: "Add"),
          ]),
        ],
        rules: [ On(Add.tap, effect: item.add(1)) ],
      ),
    ]),
    P("Cart: {count} paintings, {total} euros."),
  ],
)
```

- **`Repeat(items:, children:)`** : le modèle (`children`) est posé une fois pour chaque `Item`.
- **`Item(…)`** : les champs d'un élément, des textes, des nombres ou des mots. `key:` est sa clé : le nom d'une valeur de la page.
- Dans le modèle, **`item`** désigne l'élément : `{item.title}` dans un texte, `item.image` à la place d'une valeur, `{item}` pour montrer la valeur de la clé, `item.add(1)` pour la changer.
- **Un bloc nommé reçoit le nom de son élément** : `Add` devient `AddSunrise` et `AddRiver`. **`rules:`** : les règles du modèle, écrites une fois par élément.
- De 1 à 200 éléments ; pas de répétition dans une répétition.

Cette écriture est décidée (`ADR-040`). La leçon est `49-repeter.holo`.

## 6 quindecies. Le HTML utile, et envoyer un message

```holo
Page(
  title: "Contact",
  icon: "star.svg",
  state: State(name: "", message: "", visit: "", size: 60, done: 0),
  children: [
    H1("Contact"),
    A("Opening hours", to: "#Hours"),
    P("Price: ~~120~~ 90 euros. ==Free delivery==. A 40 m^2^ studio."),
    Image(source: "studio.jpg", alt: "The studio", caption: "The studio, in spring.", phone: "studio-small.jpg"),
    Sound(source: "chime.wav", label: "The studio chime"),
    Details(summary: "Do you ship abroad?", children: [ P("Yes, everywhere in Europe.") ]),
    Form(name: Contact, children: [
      Input(value: name, label: "Your name"),
      Input(value: visit, label: "Preferred day", type: date),
      Slider(value: size, label: "Size (cm)", min: 20, max: 120),
      Input(value: message, label: "Your message", lines: 4),
      Button(name: Send, text: "Send"),
    ]),
    Progress(value: done, max: 1, label: "Sent"),
    Dialog(name: Thanks, children: [ H2("Thank you"), P("Your message has arrived.") ]),
    H2("Opening hours", name: Hours),
    P("Tuesday to Saturday, 9 am to 6 pm."),
  ],
  rules: [
    On(Send.tap, effect: Contact.send),
    On(Contact.sent, effect: [done.set(1), Thanks.open]),
  ],
)
```

- **`icon:`** : l'image de l'onglet. **`~~barré~~`**, **`==surligné==`**, **`m^2^`**, **`H~2~O`** dans un texte.
- **`A(to: "#Hours")`** mène au bloc nommé `Hours` ; un nom qui n'existe pas est refusé.
- **`Image(caption:, phone:)`** : une légende ; une image plus légère pour un téléphone.
- **`Sound(label:)`** : un lecteur, avec ses boutons. **`Details(summary:, children:)`** : un pli qui s'ouvre.
- **`Slider(value:, label:, min:, max:)`**, **`Input(type: date | time | color)`**, **`Progress(value:, max:, label:)`**.
- **`Dialog(name:)`** : une fenêtre par-dessus la page ; `Thanks.open`, `Thanks.close`, la croix, Échap.
- **`Form(name:)`** : `Contact.send` envoie au serveur d'où vient la page les valeurs de ses champs ; puis `Contact.sent` (arrivé) ou `Contact.failed`. Chez soi, chaque message est rangé dans `messages/`, à la racine du dépôt.

Ces ajouts sont décidés (`ADR-042`). Les leçons sont `55-petits-textes.holo` à `65-icone-de-l-onglet.holo`.

### Envoyer un fichier

```holo
Page(
  title: "Apply",
  state: State(name: "", photo: "", done: 0),
  children: [
    H1("Apply"),
    Form(name: Application, children: [
      Input(value: name, label: "Your name"),
      Input(type: file, value: photo, label: "A photo of your painting", accept: image, max: 2MB),
      If(photo, not: "", children: [ P("Chosen: {photo}") ]),
      Button(name: Send, text: "Send"),
    ]),
    If(done, is: 1, children: [ P("Thank you, it arrived.") ]),
  ],
  rules: [ On(Send.tap, effect: Application.send), On(Application.sent, effect: [done.set(1), photo.set("")]) ],
)
```

- **`Input(type: file, value:, label:, accept:, max:)`**, dans un `Form` seulement, quatre au plus par formulaire.
- **`accept:`** est obligatoire : `image` (PNG, JPEG, WebP, GIF), `pdf`, ou `[image, pdf]`. **`max:`** de `1KB` à `10MB` ; `2MB` si on ne l'écrit pas.
- **La valeur est un texte** : le nom du fichier choisi, `""` sinon. `{photo}` le montre ; `photo.set("")` vide le champ.
- **La page vérifie tout de suite** la sorte et la taille, et le dit avec le message du navigateur ; un fichier refusé est retiré.
- **Le serveur vérifie tout à nouveau** : il demande au moteur ce que la page permet, lit la sorte dans les premiers octets du fichier (jamais dans son nom), et range le fichier dans `messages/files/`, sous un nom tiré au hasard. Le message garde le nom donné par le visiteur, le chemin et la taille.

Cette écriture est décidée (`ADR-059`). La leçon est `76-envoyer-un-fichier.holo`.

## 6 sedecies. Multiplier, diviser, et écrire un nombre joliment

```holo
Page(
  title: "Postcards",
  state: State(count: 3, price: 0, friends: 2, share: 0, total: 123450),
  children: [
    H1("Postcards"),
    P("{weekday:name} {day} {month:name} {year}, {hour} h {minute:00}"),
    Input(value: count, label: "How many postcards (12 euros each)?", max: 100),
    Button(name: Compute, text: "Compute"),
    P("Price: {price} euros, {share} euros each."),
    P("This month: {total:cents} euros."),
  ],
  rules: [ On(Compute.tap, effect: [price.set(12), price.mul(count), share.set(price), share.div(friends)]) ],
)
```

- **`mul`** multiplie, **`div`** divise (en nombres entiers, arrondi vers le bas) ; la quantité peut être une autre valeur.
- **Un format après deux-points** : `{minute:00}` (05), `{n:number}` (1 234 567), `{n:cents}` (1 234,50), `{weekday:name}` et `{month:name}` (mardi, octobre). La langue de la page choisit les séparateurs et les noms ; dans une répétition, `{item.price:cents}`.

Ces ajouts sont décidés (`ADR-043`). Les leçons sont `66-calculer.holo` et `67-formats.holo`.

## 6 septendecies. Une liste qui change pendant la visite

```holo
Page(
  title: "Wish list",
  state: State(wish: "", wishes: []),
  keep: [wishes],
  children: [
    H1("Wish list"),
    Input(value: wish, label: "A wish"),
    Button(name: Add, text: "Add"),
    If(wishes, is: 0, children: [ P("Nothing yet.") ], else: [ P("{wishes} wishes:") ]),
    Repeat(over: wishes, children: [
      Row(children: [ Text("{item}"), Button(name: Remove, text: "Remove") ]),
    ], rules: [ On(Remove.tap, effect: wishes.remove(item)) ]),
  ],
  rules: [ On(Add.tap, effect: [wishes.push(wish), wish.set("")]) ],
)
```

- **`State(wishes: [])`** : une liste de textes, deux cents au plus.
- **`wishes.push(wish)`** ajoute le texte d'une valeur ; **`wishes.remove(item)`** retire l'élément de la ligne touchée ; **`wishes.clear()`** vide tout. Une liste ne change que par un geste.
- **`wish.set("")`** vide un texte.
- **`Repeat(over: wishes, …)`** : une ligne par élément ; `{item}` est son texte. `{wishes}` montre le nombre ; `If(wishes, is: 0)` le compare ; `keep:` la garde.

Ces ajouts sont décidés (`ADR-044`). La leçon est `68-liste-qui-change.holo`.

### Une liste à champs : des articles, pas seulement des textes

Un élément peut avoir des **champs**, comme un `Item` de `Repeat(items:)`. La leçon est `71-liste-a-champs.holo`.

```holo
Page(
  title: "Shop",
  state: State(name: "", price: 0, articles: [ Item(title: "Sunrise", price: 12000) ]),
  data: Data(from: "catalog.json"),
  children: [
    H1("Shop"),
    Repeat(over: articles, children: [
      Column(children: [ H2("{item.title}"), Text("{item.price:cents} euros"), Button(name: Remove, text: "Remove") ]),
    ], rules: [ On(Remove.tap, effect: articles.remove(item)) ]),
    Input(value: name, label: "Title"), Input(value: price, label: "Price"), Button(name: Add, text: "Add"),
  ],
  rules: [ On(Add.tap, effect: [articles.push(Item(title: name, price: price)), name.set("")]) ],
)
```

- **`State(articles: [ Item(title: "…", price: 0) ])`** : tous les éléments ont les mêmes champs, dans le même ordre ; un champ est un texte (deux cents caractères au plus) ou un nombre entier ; seize champs au plus.
- **`{item.title}`**, **`{item.price:cents}`** dans les lignes ; **`item.image`** à la place d'une valeur (`Image(source: item.image)`). Un champ inconnu est refusé, avec la liste des champs.
- **`articles.push(Item(title: name, price: price))`** : chaque champ prend un texte, un nombre, ou le nom d'une valeur de la page. Un élément tout vide n'est pas ajouté.
- **Le serveur peut remplir la liste** : `catalog.json` contient `{ "articles": [ { "title": "Sunrise", "price": 12000 } ] }`. Seuls les champs déclarés sont repris ; ceux qui manquent valent "" ou 0. Un tableau de textes remplit une liste de textes : `{ "news": ["Open today"] }`.
- `State(articles: [])` : une liste vide au départ peut recevoir des textes ou des éléments à champs ; ses champs ne sont alors pas vérifiés.
- Ce que le visiteur saisit, ou ce que le serveur envoie, n'est jamais pris pour une balise : tout est échappé.

Cette écriture est décidée (`ADR-051`).

### Un champ dans une ligne : montrer selon lui, le changer depuis la ligne

```holo
Page(
  title: "Tasks",
  state: State(tasks: [ Item(title: "Frame the painting", done: 0), Item(title: "Deliver Sunrise", done: 1) ]),
  children: [
    H1("Tasks"),
    Repeat(over: tasks, children: [
      Column(children: [
        If(item.done, is: 1, children: [ Text("✓ {item.title}") ], else: [ Text("{item.title}") ]),
        If(item.done, is: 1, children: [ Button(name: Undo, text: "Undo") ], else: [ Button(name: Done, text: "Done") ]),
        Details(summary: "Details", children: [ P("Added by hand.") ]),
      ]),
    ], rules: [
      On(Done.tap, effect: item.done.set(1)),
      On(Undo.tap, effect: item.done.set(0)),
    ]),
  ],
)
```

- **`If(item.done, is: 1, …)`** dans une ligne : on montre l'un ou l'autre selon un champ de l'élément. Un nombre se compare avec `is`, `not`, `over`, `under` ; un texte avec `is` et `not` (`If(item.state, is: "late")`). Le champ doit exister.
- **`item.done.set(1)`** dans les règles de la ligne change ce champ de l'élément touché ; **`item.likes.add(1)`** et **`.sub(1)`** pour un nombre ; `set` prend aussi un texte ou le nom d'une valeur de la page.
- **Les lignes qui n'ont pas changé restent telles quelles.** Chaque ligne a une clé tirée de son contenu ; quand la liste change, la page ne remplace que les lignes nouvelles ou changées. Un pli ouvert, un champ où l'on écrit, le focus restent où ils sont. Une ligne refaite garde aussi le clavier : il passe au même bouton de la nouvelle ligne (voir « Les lignes d'une liste », `ADR-065`).

Cette écriture est décidée (`ADR-057`). La leçon est `74-champ-dans-une-ligne.holo`.

## 6 duodevicies. Du code enfermé : un module

Pour ce que HoloCode ne fait pas lui-même (un calcul lourd, une physique, une IA), un module compilé en WebAssembly, écrit en Rust, en C ou en Zig, tourne dans une boîte fermée.

```holo
module "sum.wasm"
Page(
  title: "Sum",
  state: State(n: 100, total: 0, stopped: 0),
  modules: [ Module(name: Sum, source: "sum.wasm", input: n, output: total, time: 100ms, memory: 1MB) ],
  children: [
    H1("Sum"),
    Button(name: Go, text: "Compute"),
    P("{total:number}"),
    If(stopped, is: 1, children: [ P("The module was stopped.") ]),
  ],
  rules: [ On(Go.tap, effect: Sum.run), On(Sum.failed, effect: stopped.set(1)) ],
)
```

- **`module "sum.wasm"`** en haut du fichier : chaque module y est annoncé.
- **`Module(name:, source:, input:, output:, time:, memory:)`** : il reçoit un nombre et en rend un, ou plusieurs valeurs (voir plus bas) ; `time` de 10ms à 5s, `memory` de 64KB à 16MB.
- **`Sum.run`** le lance ; **`Sum.done`** : le nombre est arrivé ; **`Sum.failed`** : il a été arrêté.
- La boîte : un fil à part (la page ne se bloque jamais), une mémoire plafonnée, rien d'autre (ni réseau, ni page, ni heure). Au-delà de son temps, il est arrêté.
- `bridge js` et `bridge css` sont refusés : un pont ferait entrer du code sans garantie.

Cette écriture est décidée (`ADR-045`). La leçon est `69-module-enferme.holo` ; ses trois modules, dans `exemples/lecons/modules/`.

**Plusieurs valeurs, des textes, des listes** (`ADR-077`) : `input:` et `output:` acceptent une liste de noms.

```holo
module "marks.wasm"
Page(
  title: "Marks",
  state: State(marks: [ Item(subject: "Maths", mark: "15.5") ], average: 0.0, best: "", count: 0),
  modules: [ Module(name: Report, source: "marks.wasm", input: [marks], output: [average, best, count]) ],
  children: [ Button(name: Go, text: "Compute"), P("{count} marks, average {average}, best: {best}") ],
  rules: [ On(Go.tap, effect: Report.run) ],
)
```

- Le module reçoit un texte JSON, `{"marks":[{"subject":"Maths","mark":"15.5"}]}`, et rend un objet JSON, `{"average":15.5,"best":"Maths","count":1}`.
- Il offre `alloc(taille)` (où écrire ce qu'il reçoit) et `run(adresse, taille)` (l'adresse et la taille de sa réponse) : c'est le second contrat. Un module du premier, `run(nombre)`, marche toujours.
- Le moteur relit la réponse comme des données d'un serveur : une valeur non annoncée dans `output`, ou de la mauvaise sorte, et toute la réponse est refusée (`Report.failed`) ; rien ne change.

La leçon est `97-un-module-qui-recoit-une-liste.holo` ; ses modules, `bulletin.rs` et `menteur.rs`, dans `exemples/lecons/modules/`.

## 6 undetricies. Des adresses qui portent des valeurs : `profil/{id}.holo`

Un fichier nommé `profil/{id}.holo` sert toutes les adresses `/profil/123`, `/profil/ada` :

```holo
// profil/{id}.holo
Page(
  title: "Profile",
  children: [
    H1("Hello, {id}"),
    If(id, is: "ada", children: [ P("Ada wrote the first program.") ]),
    A("← All the profiles", to: "../profiles.holo"),
  ],
)
```

- **`{id}` dans le nom du fichier** : la page le lit comme ses autres valeurs, un texte, `{id}` ou `If(id, is: "ada")`. Des dossiers aussi : `{author}/notes/{note}.holo`. Un vrai fichier passe avant le modèle.
- On la lit, on ne la change pas : `id.set(…)`, `Input(value: id)` et `State(id: …)` sont refusés.
- `holo serve` (`ADR-074`) fabrique la page de l'adresse, avec ou sans JavaScript, garde l'état de chaque visiteur pour cette adresse, et range un formulaire envoyé de là avec son adresse ; `/contact` mène aussi à `contact.holo`.
- `holo check profil/{id}.holo` vérifie le modèle, chaque nom valant un texte vide.
- **Le titre lit les valeurs**, comme un texte : `title: "Le profil de {id}"` écrit « Le profil de ada » dans l'onglet ; un titre qui lit une valeur qui change (`{pages}`) suit ses changements (`ADR-090`).
- **Les valeurs gardées (`keep`) sont rangées par adresse** : `/profil/ada` retrouve les siennes, `/profil/bob` part de zéro (`ADR-090` ; la leçon est `112-une-adresse-qui-se-souvient.holo`).
- **Un lien remonte d'un dossier**, comme sur le web : `A(to: "../profiles.holo")` ; il porte aussi des lettres accentuées, `A(to: "profil/Adé")`.

Cette écriture est proposée (`ADR-078`) ; la forme, l'adresse dite par le nom du fichier, est celle choisie par Yocthan. La leçon est `100-une-adresse-qui-porte-une-valeur.holo`.

### L'historique dans une page : `address: [onglet, page]`

```holo
Page(
  title: "Gallery",
  state: State(tab: "paintings", page: 1),
  address: [tab, page],
  children: [
    Row(gap: 8px, children: [
      Button(name: Paintings, text: "Paintings"),
      Button(name: Drawings, text: "Drawings"),
    ]),
    If(tab, is: "drawings", children: [ P("Charcoal, ink, red chalk.") ]),
    P("Page {page}"),
    Button(name: Next, text: "Next page"),
  ],
  rules: [
    On(Paintings.tap, effect: [tab.set("paintings"), page.set(1)]),
    On(Drawings.tap, effect: [tab.set("drawings"), page.set(1)]),
    On(Next.tap, effect: page.add(1)),
  ],
)
```

- **`address: [tab, page]`** écrit ces valeurs dans l'adresse, après le `?` : `gallery.holo?tab=drawings&page=2`. Une valeur à son départ n'y est pas : la page du début garde son adresse nue.
- **Un toucher ou une touche qui les change fait un pas** : « Précédent » revient à l'onglet d'avant, « Suivant » y retourne. Ce qu'on écrit dans un champ, le temps, les données reçues mettent l'adresse à jour sans faire de pas.
- **L'adresse se partage** : la page arrive avec ses valeurs, fabriquée par le serveur, et même sans JavaScript avec `holo serve`.
- Ce qui arrive par l'adresse vient de n'importe qui : seules les valeurs nommées sont reprises, dans leurs bornes ; un texte que la page n'écrit qu'avec des mots fixes (`tab.set("drawings")`, les options d'un `Choice`) n'en prend pas d'autre. Une valeur mal écrite part de son départ.
- Refusés : une liste, une valeur gardée (`keep`), l'heure, une valeur du nom du fichier, et les noms que le moteur lit déjà dans une adresse (`values`, `view`, `zoom`, `x`, `y`…).

Cette écriture est proposée (`ADR-091`) ; le nom `address:` est à valider par Yocthan. La leçon est `114-l-historique-dans-une-page.holo`.

## 6 tricies. Des valeurs partagées, en direct : `Shared`

Une valeur que le serveur garde pour tout le monde : les places restantes, un compteur de « J'aime ». Chaque visiteur la voit changer en direct, sans recharger la page.

```holo
Page(
  title: "Concert",
  state: State(booked: 0),
  shared: Shared(seats: 20, likes: 0),
  children: [
    H1("Tonight's concert"),
    P("Seats left: {seats}"),
    If(seats, over: 0,
      children: [ If(booked, is: 0, children: [ Button(name: Book, text: "Book a seat") ]) ],
      else: [ P("Sold out.") ]),
    If(booked, is: 1, children: [ P("Your seat is kept.") ]),
    Button(name: Like, text: "Like ({likes})"),
  ],
  rules: [
    On(Book.tap, effect: [seats.sub(1), booked.set(1)]),
    On(Like.tap, effect: likes.add(1)),
  ],
)
```

- **`shared: Shared(seats: 20, likes: 0)`**, à côté de `state:` : des nombres entiers, des nombres à virgule, des textes (200 caractères au plus) ; seize au plus. La valeur de départ est celle du fichier. `booked`, dans `State`, n'est qu'à un visiteur ; `seats`, dans `Shared`, est la même pour tous.
- Elles se lisent comme les autres valeurs (`{seats}`, `If(seats, over: 0, …)`) et se changent **par un toucher** : `On(Book.tap, effect: seats.sub(1))`.
- **C'est le serveur qui arbitre**, avec le même moteur : la page lui envoie le geste et attend sa réponse ; il range la nouvelle valeur, puis l'envoie à toutes les pages ouvertes à cette adresse. Deux visiteurs en même temps : chacun son tour, rien n'est perdu. **Un bouton caché ne se touche pas** : quand `If(seats, over: 0, …)` cache « Book a seat », le serveur refuse ce toucher, même forgé ; c'est ainsi qu'une condition garde la dernière place.
- Une valeur partagée vaut **pour une adresse** : avec `concert/{id}.holo` (§ 6 undetricies), `/concert/12` et `/concert/13` ont chacune leurs places.
- **Sans JavaScript**, la page reste juste : le toucher part au serveur (`holo serve`), et la page revient à jour.
- Refusé, avec la raison : le même nom dans `State` ou dans l'adresse ; une horloge (`Every`, `After`), une règle qui guette (`When`), une touche, un survol, des données, un module ou un glissement qui la changerait ; `keep` ; un champ lié à elle et une liste partagée (pas encore).
- `holo serve` les garde dans sa base (`holo-data/site.sqlite`, table `shared`) ; le serveur d'essai, en mémoire, jusqu'à son arrêt.

Cette écriture est proposée (`ADR-079`). La leçon est `101-une-valeur-partagee.holo` : ouvre-la sur ton téléphone et sur ton ordinateur.

## 6 untricies. Des comptes : se connecter, une page réservée, le panier qui suit le compte

Avec `holo serve`, un site a des comptes, gardés par le serveur de l'auteur, dans la base du site : ni Google, ni Apple, ni adresse e-mail. Une page sait si le visiteur est connecté, et sous quel nom :

```holo
Page(
  title: "Shop",
  state: State(cart: 0),
  children: [
    H1("The shop"),
    P("Cart: {cart}"),
    Button(name: Add, text: "Add"),
    If(signedIn, is: 1, children: [
      P("Hello, {account}: your cart follows you."),
      A("My account", to: "/account"),
    ], else: [
      A("Sign in", to: "/account/signin"),
    ]),
  ],
  rules: [ On(Add.tap, effect: cart.add(1)) ],
)
```

Et une page peut être réservée aux personnes connectées :

```holo
Page(title: "Members", access: members, children: [ H1("Hello, {account}") ])
```

- **`signedIn`** (1 quand le visiteur est connecté, 0 sinon) et **`{account}`** (son nom, vide sinon) : la page les lit comme ses autres valeurs, sans pouvoir les changer ni les garder (`signedIn.set(1)`, `Input(value: account)`, `State(account: …)` et `keep: [account]` sont refusés). C'est le serveur qui les donne.
- **`access: members`** : la page n'est montrée qu'aux personnes connectées ; les autres sont menées à « Se connecter », puis ramenées. Rien de la page ne leur arrive avant, pas même son texte. Sans `access`, la page est à tout le monde.
- **Les pages de compte sont fabriquées par le moteur**, en HTML ordinaire, accessibles, sans JavaScript : `/account/signup` (créer un compte), `/account/signin` (se connecter), `/account` (activer le code à 6 chiffres, se déconnecter). Ce sont les seuls liens qui partent de la racine du site.
- **Se connecter** : un nom et un mot de passe (12 caractères au moins), puis, si on l'a activé, le code à 6 chiffres d'une application d'authentification du téléphone (Aegis, FreeOTP, Google Authenticator…), calculé sans Internet ni SMS. Un code ne sert qu'une fois ; cinq essais ratés, puis une attente.
- **Le panier suit le compte** : connecté, les valeurs d'une page sont gardées par le compte ; on les retrouve sur son téléphone et sur son ordinateur, avec ou sans JavaScript. Ce qu'on avait fait avant de se connecter suit aussi.
- **Le serveur ne croit jamais l'état qu'envoie la page d'un membre** : il part de celui qu'il garde pour son compte, et n'y prend que ce que le visiteur a écrit dans les champs (la correction de Codex). Une condition sur une valeur du membre garde donc une valeur partagée : `If(booked, is: 0, children: [ Button(name: Book, …) ])` réserve une seule place par compte, même avec une page forgée.
- Le mot de passe n'est jamais gardé en clair (son empreinte Argon2id seulement) ; la session est un numéro tiré au hasard, dans un cookie que la page ne lit pas, oublié après 14 jours sans visite ; se déconnecter l'efface.
- Le serveur d'essai (`node outils/server.mjs`) n'a pas de comptes : à la place des pages de compte et des pages réservées, il dit qu'il faut `holo serve`.

Cette écriture est décidée (`ADR-081`, validée par Yocthan le 2026-10-09) ; se connecter par un mot de passe puis un code à 6 chiffres, tout chez l'auteur, est son choix du 2026-10-08. Les leçons sont `104-se-connecter.holo`, `105-une-page-reservee.holo` et `106-le-panier-qui-suit-le-compte.holo`.

## 6 sexvicies. La mise en page : téléphone, ordinateur, la place, ce qui dépasse, les proportions, le curseur, justifié, décrocher

```holo
Page(
  title: "Workshop",
  children: [
    H1("The workshop"),
    P.banner("Open every morning."),
    Grid(columns: 3, gap: 16px, children: [
      Column.card(gap: 8px, children: [ H2("Paint"), P.detail("Colours, canvas, brushes.") ]),
      Column.card(gap: 8px, children: [ H2("Draw"), P.detail("Pencils and paper.") ]),
      Column.card(gap: 8px, children: [ H2("Look"), P.detail("Take your time.") ]),
    ]),
  ],
)

Page { padding: 24px 16px; computer: { max-width: 960px; } }
H2 { font-size: 22px; narrow: { font-size: 16px; } }
.banner { padding: 16px; phone: { font-size: 14px; padding: 8px; } computer: { font-size: 22px; } }
.card { padding: 16px; narrow: { padding: 8px; } }
.detail { narrow: { display: none; } }
```

- **`phone: { … }`** vaut sur un écran plus étroit que la page (640px) ; **`computer: { … }`**, sur un écran de 1024px ou plus. `Page { max-width: 960px; }`, ou la même chose dans `computer:`, élargit la page (640px sans rien écrire).
- **`narrow: { … }`** vaut quand la case de `Grid` où se trouve le bloc fait moins de 320px, quel que soit l'écran : c'est la place du bloc qui compte. Rien à déclarer : la page mesure chaque case. Dans un `Row` ou un `Column`, les cases mesurées sont celles qui reçoivent une part de la place, `grow:` ou une largeur en % : une carte de `width: 45%` se serre sur un téléphone, pas sur un ordinateur (`ADR-090` ; la leçon est `113-une-rangee-qui-se-serre.holo`).
- **`display: none`** cache un bloc, dans `phone:`, `computer:` et `narrow:` seulement ; jamais ce qui agit (un bouton, un lien, un champ, un formulaire, un bloc qu'une règle écoute) : décision de Yocthan du 2026-10-07, sa règle de parité. Une phrase ou une image peuvent se cacher sur un seul appareil.

```holo
Page(
  title: "Gallery",
  zoom: Zoom(detach: true),
  children: [
    H1("Gallery"),
    P.summary("A long summary: the workshop, its tools, its colours, its hours and the way to get there. Three lines are enough; a screen reader reads it all."),
    P.box("A box of 120 pixels at most. What does not fit scrolls, with a finger or the wheel."),
    P.price("12,345.00 €"),
    Image.square(source: "landscape.svg", alt: "Green hills under a yellow sun"),
    P.book("The workshop opens every morning. You learn to prepare colours, to stretch a canvas, and to look for a long time before you start."),
  ],
)

.summary { line-clamp: 3; }
.box { max-height: 120px; overflow: auto; }
.price { white-space: nowrap; }
.square { width: 160px; aspect-ratio: 1; object-position: left; cursor: zoom-in; }
.book { text-align: justify; }
```

- **Un mot trop long passe à la ligne**, sans rien écrire : la page ne déborde pas sur un téléphone.
- **`line-clamp: 3`** arrête le texte après 3 lignes, avec « … » ; un lecteur d'écran lit tout. `text-overflow` est refusé : seul, il ne fait rien.
- **`max-height`**, **`min-height`**, **`min-width`** : en px ou en %. **`overflow`** (et `overflow-x`, `overflow-y`) : `visible`, `hidden`, `auto` (on fait défiler), `scroll`. **`white-space`** : `normal`, `nowrap` (jamais à la ligne, comme un prix), `pre-line` (les retours à la ligne écrits sont gardés), `pre-wrap` (les espaces aussi) ; `pre` est refusé : il ne passe jamais à la ligne, et le texte sort de l'écran d'un téléphone.
- **`aspect-ratio: 16/9`**, ou `1` pour un carré, garde des proportions. **Une image n'est jamais déformée** : sans rien écrire, elle remplit son cadre, coupée sur les bords (`object-fit: cover`) ; `contain` la montre en entier ; seul `fill` la déforme. **`object-position: left`** dit ce qu'on garde.
- **`cursor:`** `pointer`, `help`, `grab`, `zoom-in`… (22 formes), ou une image rangée à côté, `url("viseur.svg")`. Au doigt, il n'y a pas de curseur : ce n'est qu'une indication.
- **`text-align: justify`** : le texte touche les deux bords, et les mots se coupent en fin de ligne, dans la langue de la page.
- **Le zoom** : sans rien écrire, pincer ou Ctrl + molette grossit la page sur place, comme pour tout site. **`zoom: Zoom(detach: true)`** ajoute au menu ☰ « Décrocher » : la page se détache comme une feuille, et le zoom l'approche ; « Accrocher » la remet à sa place (voir la partie 7).
- **Les touches à l'écran** : sur un téléphone, une page qui écoute des touches (`On(Key.left, …)`, `On(Key.p, …)`) les montre en bas de l'écran, et le doigt fait ce que fait le clavier. Rien à écrire.

Cette écriture est proposée (`ADR-069`) et attend la validation de Yocthan. Les leçons vont de `89-telephone-et-ordinateur.holo` à `94-decrocher-la-page.holo` ; les touches à l'écran sont dans `77-toutes-les-touches.holo`.

## 6 duodetricies. Un article long : un encadré, des liens, des sous-titres, l'impression

```holo
Page(
  title: "Shooting stars",
  lang: "en",
  children: [
    H1("Shooting stars"),
    Image(source: "star.png", alt: "A golden star"),
    Aside(children: [ H2("Did you know?"), P("A shooting star is a grain smaller than a pea.") ]),
    A("Read more on Wikipedia", to: "https://en.wikipedia.org/wiki/Perseids", newTab: true),
    A("Download the programme", to: "programme.txt", download: true),
    Video(source: "film.mp4", label: "The sky at night", captions: "film.vtt"),
  ],
)

Aside { print: { display: none; } }
```

- **`Aside(children: [ … ])`** : un encadré à part ; le lecteur d'écran l'annonce comme un contenu complémentaire.
- **`newTab: true`** : un nouvel onglet ; le lecteur d'écran l'annonce, et la page ouverte ne peut pas toucher à celle-ci.
- **`download: true`** : un fichier rangé à côté se télécharge ; ni une page `.holo`, ni une adresse du web.
- **`captions: "film.vtt"`** : des sous-titres WebVTT, montrés d'emblée, dans la langue de la page.
- Les images plus bas dans la page ne viennent qu'en approchant de l'écran ; la première vient tout de suite. Rien à écrire.
- **`print: { … }`** dans un style : ce qui change sur papier ; `print: { display: none; }` cache un bloc. Le moteur cache de lui-même ses outils et écrit l'adresse des liens du web.

Cette écriture est décidée (`ADR-073`). Les leçons sont `95-un-article-long.holo` et `96-une-video-sous-titree.holo`.

## 6 duotricies. Un dessin : `Drawing` et ses formes

Un dessin vectoriel, net à toute taille, que le lecteur d'écran lit par son nom (`ADR-086`).

```holo
Page(
  title: "A landscape",
  state: State(sun: 70),
  children: [
    Drawing(label: "A house, a hill and the sun", width: 320, height: 160, children: [
      Rect(x: 0, y: 0, width: 320, height: 160, radius: 12, fill: "#16213e"),
      Circle(x: 250, y: sun, r: 18, fill: "#E9B44C"),
      Path(d: "M0 120 Q160 92 320 120 L320 160 L0 160 Z", fill: "#1f4037"),
      Line(from: [0, 132], to: [320, 132], stroke: "#8fd3ff", thickness: 1),
    ]),
    Button(name: Rise, text: "Raise the sun"),
  ],
  rules: [ On(Rise.tap, effect: sun.sub(20)) ],
)
```

- **`Drawing(label:, width:, height:, children:)`** : `label` dit ce que montre le dessin, pour qui ne le voit pas ; `width` et `height` sont les unités du dessin, qui garde ces proportions sur tous les écrans.
- **Les formes**, seulement dans un `Drawing` : `Rect(x:, y:, width:, height:, radius:)`, `Circle(x:, y:, r:)`, `Line(from: [x, y], to: [x, y])`, `Path(d: "M… L… Z")` (un tracé SVG : M pour aller à un point, L pour tracer jusqu'à un autre, Q pour une courbe, Z pour fermer).
- **`fill`** remplit, **`stroke`** trace le bord, **`thickness`** dit son épaisseur, **`opacity`** va de 0 à 1. Les formes se dessinent dans l'ordre : la dernière passe devant.
- **Une mesure peut être le nom d'un nombre entier de la page** (`y: sun`) : la forme le suit.
- Pas de dessin trait par trait en JavaScript : refusé.

La leçon est `98-un-dessin.holo`.

**Des formes venues d'une liste** (`ADR-088`) : `Drawing(…, shapes: flower)`, une forme par élément, `Item(form: "circle", x: 10, y: 20, r: 5, fill: "#E9B44C")` (`form` vaut `"rect"`, `"circle"`, `"line"` ou `"path"` ; un trait prend `x1`, `y1`, `x2`, `y2`). Un module enfermé peut rendre cette liste : il « dessine » sans toucher au dessin du navigateur, et le moteur vérifie chaque forme. La leçon est `110-un-module-qui-dessine.holo`.

## 6 tertricies. Un tableau de bord : `Chart`

Un graphique, dessiné par le moteur d'après une liste à champs (`ADR-087`).

```holo
Page(
  title: "Sales",
  state: State(sales: [], period: "", amount: ""),
  data: Data(from: "sales.json"),
  children: [
    Chart(kind: bars, over: sales, value: amount, label: period, title: "Sales of the week"),
    Chart(kind: pie, over: sales, value: amount, label: period, title: "Share of each day"),
    Input(value: period, label: "Period"),
    Input(value: amount, label: "Amount"),
    Button(name: Add, text: "Add a sale"),
  ],
  rules: [ On(Add.tap, effect: [sales.push(Item(period: period, amount: amount)), period.set(""), amount.set("")]) ],
)
```

- **`kind`** : `bars` (des barres), `line` (une courbe) ou `pie` (des parts) ; **`over`** : la liste ; **`value`** : le champ du nombre ; **`label`** : le champ du nom ; **`title`** : le titre, obligatoire ; **`color`** : la couleur des barres ou de la courbe.
- Le lecteur d'écran lit un tableau caché, avec les mêmes chiffres.
- Le graphique suit sa liste : des données reçues, une liste calculée, un élément ajouté.

La leçon est `99-un-tableau-de-bord.holo`.

## 6 duodequadragies. Une liste de définitions : `Term`

Un terme et sa définition, toujours ensemble (`ADR-097`) : une fiche technique, un glossaire.

```holo
Page(
  title: "The workshop lamp",
  state: State(price: 189.00),
  children: [
    H1("The workshop lamp"),
    List(children: [
      Term("Height", "45 cm"),
      Term("Weight", "2 kg"),
      Term("Colour", "Night blue, or **copper**"),
      Term("Price", "{price} €"),
    ]),
  ],
)
```

- **`Term("Weight", "2 kg")`** : le terme, puis sa définition. Une `List` dont les éléments sont des `Term` devient une liste de définitions (`dl`, `dt`, `dd`) ; le lecteur d'écran annonce chaque terme, puis sa définition.
- La définition lit les valeurs de la page (`{price}`) et le texte enrichi (`**copper**`).
- Refusés : un `Term` hors d'une liste ; une liste qui mélange des `Term` et autre chose ; `ordered: true` (une liste de termes ne se numérote pas) ; un `Term` sans sa définition.

La leçon est `120-une-liste-de-definitions.holo`.

## 6 quinvicies. Des formulaires qui vérifient

```holo
Page(
  title: "Contact",
  state: State(name: "", email: "", message: "", accept: 0, sent: 0),
  children: [
    Form(name: Contact, children: [
      Input(value: name, label: "Your name", required: true, min: 2),
      Input(value: email, label: "Your e-mail", type: email, required: true),
      Input(value: message, label: "Your message", lines: 4, required: true, min: 10, max: 500),
      Checkbox(value: accept, label: "You may answer me by e-mail", required: true),
      Button(name: Send, text: "Send"),
    ]),
    If(sent, is: 1, children: [ "Thank you." ]),
  ],
  rules: [ On(Send.tap, effect: Contact.send), On(Contact.sent, effect: sent.set(1)) ],
)
```

- **`required: true`** : un texte rempli, une case cochée, une réponse choisie (`Choice`). Un nombre n'est jamais vide : on le borne, `min: 1`. Un champ obligatoire vit dans un `Form`.
- **`type: email`** : le clavier des adresses ; l'adresse doit être plausible (`nom@exemple.fr`).
- **`min:` et `max:`** sur un texte : sa longueur, en caractères.
- À l'envoi, le moteur vérifie chaque champ. **Ce qui ne va pas s'écrit sous le champ**, dans la langue de la page, et un lecteur d'écran le lit ; le premier champ à corriger reçoit le clavier. Les messages suivent ce qu'on corrige.
- **Entrée** dans un champ envoie le formulaire (son premier bouton). **Un seul envoi à la fois**, **15 secondes au plus** : sinon, `Contact.failed`.
- **Le serveur vérifie à nouveau**, avec les mêmes règles : un message forgé reçoit 422.

Cette écriture est proposée (`ADR-068`) et attend la validation de Yocthan. La leçon est `88-un-formulaire-qui-verifie.holo`.

## 6 quatervicies. Des dates

```holo
Page(
  title: "Booking",
  state: State(arrival: "", departure: "", price: 80.00, total: 0.00),
  computed: [ Days(name: nights, from: arrival, to: departure) ],
  children: [
    P("Today is {today:date}."),
    Input(value: arrival, label: "Arrival", type: date, min: today),
    Input(value: departure, label: "Departure", type: date, min: today),
    Button(name: Week, text: "One week"),
    If(departure, over: arrival, children: [
      P("{nights} night(s), arriving on {arrival:weekday} {arrival:date}."),
      Button(name: Compute, text: "Price"),
    ]),
    If(total, over: 0, children: [ P("Total: {total} €") ]),
  ],
  rules: [
    On(Week.tap, effect: [departure.set(arrival), departure.add(7)]),
    On(Compute.tap, effect: [total.set(price), total.mul(nights)]),
  ],
)
```

- **Une date est un texte « AAAA-MM-JJ »**, celui que donne un champ date. Une valeur est une date quand c'est `today`, quand un `Input(type: date)` la présente, ou quand elle est déclarée avec une date, `State(due: "2026-12-24")`.
- **`today`** est la date du jour, donnée par l'appareil du visiteur ; la page la tient à jour, et passe minuit.
- **`{arrival:date}`** la montre dans la langue de la page, « 10 octobre 2026 » ; **`{arrival:weekday}`**, « samedi ».
- **Deux dates se comparent dans le temps** : `If(departure, over: arrival)`, `If(arrival, under: today)`, `If(arrival, over: "2026-12-24")`. Une date vide ne compare rien.
- **Une date avance ou recule de jours** : `due.add(7)`, `due.sub(1)`, `due.set(today)`.
- **`Days(name: nights, from: arrival, to: departure)`**, dans `computed: [ … ]`, compte les jours entre deux dates : `{nights}`, `If(nights, over: 6)`, `total.mul(nights)`. Il vaut 0 si une date manque, ou si le départ vient avant l'arrivée.
- **Le champ date a des bornes** : `min: today`, `max: "2026-12-31"`. Le navigateur grise les autres jours ; le moteur les refuse, comme un jour qui n'existe pas (« 2026-02-30 »). `min:` vaut aussi pour un champ de nombre : `Input(value: quantity, label: "…", min: 1)`.

Cette écriture est proposée (`ADR-067`) et attend la validation de Yocthan. La leçon est `87-des-dates.holo`.

## 6 tervicies. Des nombres à virgule

```holo
Page(
  title: "Order",
  state: State(price: 12.50, qty: 1, sum: 12.50),
  children: [
    P("Price: {price} €"),
    Input(value: price, label: "Change the price"),
    Input(value: qty, label: "How many?"),
    Button(name: Compute, text: "Compute"),
    P("Total: {sum} € ({sum:number} €)"),
    Button(name: Tip, text: "Add 10 %"),
    If(sum, over: 49.99, children: [ "Free delivery." ]),
  ],
  rules: [
    On(Compute.tap, effect: [sum.set(price), sum.mul(qty)]),
    On(Tip.tap, effect: sum.mul(1.1)),
  ],
)
```

- **`price: 12.50`** déclare une valeur à virgule, avec deux chiffres après la virgule (de 1 à 6). Elle est **exacte** : 12,50 × 3 font 37,50, jamais 37,4999.
- **`{price}`** la montre dans la langue de la page : « 12,50 » en français, « 12.50 » en anglais ; **`{price:number}`** la groupe par milliers, « 1 234,50 ».
- **`add`, `sub`, `set`** prennent un nombre qui n'a pas plus de chiffres après la virgule que la valeur (`sum.add(0.25)`), ou une autre valeur qui n'en a pas plus. **`mul` et `div`** prennent un facteur, entier ou à virgule (`sum.mul(1.1)` : 10 % de plus) ; le résultat est arrondi au plus proche, à l'échelle de la valeur. Entre nombres entiers, la division arrondit toujours vers le bas.
- **Les comparaisons sont exactes**, même entre un entier et un nombre à virgule : `If(sum, over: 49.99)`.
- **Un champ** présente une valeur à virgule avec le clavier décimal ; « 12,5 » et « 12.5 » sont compris.
- **Des données reçues** : `{"price": 12.5}` va dans une valeur à virgule.
- Pas encore : une glissière, une barre, une case, une place sur un plateau, les prix (`Prices`) et `limit:` prennent un nombre entier ; une fiche de liste aussi (un prix de fiche s'écrit en centimes, `{item.price:cents}`) ; pas de nombre négatif.

Cette écriture est proposée (`ADR-066`) et attend la validation de Yocthan. La leçon est `86-nombres-a-virgule.holo`.

## 6 duovicies. Les lignes d'une liste : une clé, le clavier gardé, le total

```holo
Page(
  title: "Tasks",
  state: State(shown: 2, tasks: [
    Item(id: "t1", title: "Buy bread", done: 0),
    Item(id: "t2", title: "Call Ada", done: 0),
    Item(id: "t3", title: "Water the plants", done: 0),
  ]),
  computed: [ Filter(name: ordered, from: tasks, sortBy: done, limit: shown, total: all) ],
  children: [
    P("{ordered} of {all}"),
    Repeat(over: ordered, key: id, children: [
      Text("{item.title}"),
      If(item.done, is: 1, children: [ Button(name: Undo, text: "Reopen") ], else: [ Button(name: Done, text: "Done") ]),
      Button(name: Drop, text: "Remove"),
    ], rules: [
      On(Done.tap, effect: item.done.set(1)),
      On(Undo.tap, effect: item.done.set(0)),
      On(Drop.tap, effect: tasks.remove(item)),
    ]),
    If(shown, under: all, children: [ Button(name: More, text: "Show more") ]),
  ],
  rules: [ On(More.tap, effect: shown.add(2)) ],
)
```

- **`key: id`** : chaque ligne est reconnue par le champ `id` de son élément, même quand le reste change ou qu'elle change de place. Le champ doit exister ; une liste de textes n'en a pas besoin.
- **Le clavier reste** quand une ligne est refaite : il passe au même bouton de la nouvelle ligne, ou au premier bouton de la ligne. Avec une clé, il suit l'élément là où il va (une tâche faite descend) ; sans clé, il reste au même rang. Une ligne retirée laisse le clavier à la ligne qui prend sa place. Rien à écrire.
- **Une règle des lignes d'une liste calculée change l'élément d'origine** : `item.done.set(1)` change la tâche de `tasks`, `tasks.remove(item)` la retire de `tasks`.
- **`total: all`** dans un `Filter` : le nombre trouvé avant de couper. `{all}` le montre ; `If(shown, under: all, …)` cache « Show more » quand tout est montré.
- Deux répétitions de la même liste gardent chacune leur modèle.

Cette écriture est proposée (`ADR-065`) et attend la validation de Yocthan. La leçon est `85-une-cle-pour-chaque-element.holo` ; le total est dans `82-chercher-filtrer-trier.holo`.

## 6 unvicies. Comparer des textes

```holo
Page(
  title: "Shop",
  state: State(size: "M", answer: "", score: 0, email: "", again: ""),
  children: [
    Choice(value: size, label: "Size", options: [ "S", "M", "L" ]),
    If(size, is: "L", children: [ "Large is roomy." ], else: [ Text("Size {size}.") ]),
    Input(value: answer, label: "Capital of France"),
    Text("Right answers: {score}"),
    Input(value: email, label: "Your e-mail"),
    Input(value: again, label: "Once more"),
    If(again, not: "", children: [
      If(again, not: email, children: [ "The two e-mails differ." ]),
    ]),
  ],
  rules: [ When(answer, is: "Paris", effect: score.add(1)) ],
)
```

- `If(size, is: "L")` : le texte `size` est-il « L » ? `not: "M"` : est-il autre chose que « M » ? `is: ""` demande toujours s'il est vide.
- `If(again, not: email)` compare deux valeurs de texte entre elles.
- `When(answer, is: "Paris", effect: …)` agit au moment où le texte devient « Paris », que le visiteur l'écrive ou qu'une règle le change ; pas tant qu'il le reste. Ses effets changent des nombres ou jouent un son.
- **À la lettre près** : majuscules, accents et espaces comptent. « paris » n'est pas « Paris ».
- **Pas de mélange** : un texte se compare à un texte, un nombre à un nombre. `over` et `under` ne servent qu'aux nombres ; pour ranger des textes, on trie une liste (`Filter(sortBy:)`).
- Des règles rangées sous une condition sur un texte, `If(mode, is: "play", rules: [ Every(…) ])`, valent tant qu'elle est vraie.

Cette écriture est proposée (`ADR-063`) et attend la validation de Yocthan. La leçon est `83-comparer-des-textes.holo`.

## 6 vicies. Chercher, filtrer, trier : les listes calculées

```holo
Page(
  title: "Gallery",
  state: State(search: "", chosen: "", shown: 4, works: [
    Item(title: "Sunrise", kind: "oil", price: 120),
    Item(title: "The river", kind: "watercolour", price: 90),
  ]),
  computed: [
    Filter(name: found, from: works, contains: search, in: [title], field: kind, is: chosen, sortBy: price, reverse: true, limit: shown),
  ],
  children: [
    Input(value: search, label: "Search"),
    P("{found} work(s)"),
    Repeat(over: found, empty: "Nothing matches.", children: [ Text("{item.title}") ]),
    Button(name: More, text: "Show more"),
  ],
  rules: [ On(More.tap, effect: shown.add(4)) ],
)
```

- **Une liste calculée** se déclare dans `computed: [ … ]` : `Filter(name: found, from: works, …)`. Elle se refait d'après sa source à chaque changement, et se montre comme une liste : `Repeat(over: found)`, `{found}` pour son nombre d'éléments.
- `contains: search` cherche la valeur de texte `search` dans les champs `in: [title]` (dans tous, si `in` manque), sans majuscules ni accents : « ELAN » trouve « Élan ».
- `field: kind, is: chosen` ne garde que les éléments dont le champ vaut cette valeur. `sortBy: price` trie (des nombres comme des nombres) ; `reverse: true`, du plus grand au plus petit. `limit: shown` n'en montre que `shown` : une règle l'augmente pour « montrer plus ».
- **Une valeur vide ne filtre pas** : un champ de recherche vide montre tout.
- `Repeat(empty: "…")` dit ce qu'on écrit quand la liste est vide ; un lecteur d'écran l'annonce.
- Une liste calculée ne se change pas par une demande, et ne se garde pas : on change ou on garde sa source. Mais une règle écrite dans ses lignes change l'élément d'origine : `item.done.set(1)`, `tasks.remove(item)` (`ADR-065`).
- `total: matching` donne le nombre trouvé avant de couper : « {found} sur {matching} » ; `If(shown, under: matching, …)` cache « Show more » quand tout est montré (`ADR-065`).
- **Une page d'une liste** : `offset: start` saute les `start` premiers éléments trouvés, après la recherche, le filtre et le tri, puis `limit:` coupe. Avec `Filter(…, offset: start, limit: 20)`, c'est une page de vingt ; `start.add(20)` mène à la suivante, `start.sub(20)` à la précédente, `start.set(0)` au début. Le total (`total:`) compte avant les deux coupes : « page de {found} sur {matching} ». Au-delà de la fin, la page est vide. `offset` prend un entier, ou le nom d'un nombre entier de la page (`ADR-084`).
- Une liste garde **deux cents éléments au plus** (une liste de `State`, une liste reçue par `Data`, une répétition) ; au-delà, les données reçues ne gardent que les deux cents premiers, et un fichier qui en déclare plus est refusé (`ADR-084`).

```holo
Page(
  title: "Catalogue",
  state: State(search: "", start: 0, products: [ Item(title: "Pinceau", price: 300), Item(title: "Toile", price: 1200) ]),
  computed: [ Filter(name: page, from: products, contains: search, sortBy: price, offset: start, limit: 20, total: matching) ],
  children: [
    Input(value: search, label: "Search"),
    P("{page} on {matching}"),
    Repeat(over: page, children: [ Text("{item.title}") ]),
    If(start, over: 0, children: [ Button(name: Previous, text: "Previous page") ]),
    Button(name: Next, text: "Next page"),
  ],
  rules: [ On(Next.tap, effect: start.add(20)), On(Previous.tap, effect: start.sub(20)) ],
)
```

L'écriture `Filter`, `computed`, `contains`, `in`, `sortBy`, `limit`, `empty` est choisie par Yocthan (`ADR-062`) ; `field`, `is` et `reverse` attendent sa validation. `offset` et la borne de deux cents sont décidés (`ADR-084`, validé par Yocthan le 2026-10-09 ; proposés par Codex). Les leçons sont `82-chercher-filtrer-trier.holo` et `109-un-catalogue-page-par-page.holo`.

## 6 undevicies. La fin du web : toutes les touches, apparaître en descendant, le son réglé, des tailles qui suivent le visiteur

```holo
Page(
  title: "Rain",
  state: State(score: 0),
  children: [
    Column.hero(children: [ H1("Rain") ]),
    P("Press P to count, Escape to start again: {score}.", enter: Enter(y: 40px, opacity: 0, inView: true)),
    Sound(name: Rain, source: "rain.mp3", volume: 0.4, loop: true),
    Row(gap: 12px, children: [
      Button(name: Start, text: "Rain"),
      Button(name: Quiet, text: "Silence"),
    ]),
  ],
  rules: [
    On(Key.p, effect: score.add(1)),
    On(Key.digit5, effect: score.add(5)),
    On(Key.escape, effect: score.set(0)),
    On(Start.tap, effect: Rain.play),
    On(Quiet.tap, effect: Rain.stop),
  ],
)

.hero { height: screen; padding: 24px; }
```

- **Toutes les touches utiles** : en plus des flèches et de l'espace, `Key.enter`, `Key.escape`, les lettres `Key.a` à `Key.z`, les chiffres `Key.digit0` à `Key.digit9`. Jamais Tab. Les lettres et les chiffres se coupent dans le menu ☰ (« Touches à une lettre »).
- **Apparaître en descendant** : `inView: true` dans une entrée. Le bloc entre quand il arrive à l'écran ; sans JavaScript, ou pour qui demande moins de mouvement, tout se voit d'emblée.
- **Le son réglé** : `volume:` de 0 à 1, `loop: true`, et la capacité `stop`.
- **Des tailles qui suivent le visiteur** : on écrit des pixels ; le moteur écrit des `rem` pour les marges, les largeurs, les hauteurs, les coins et les écarts, qui grandissent avec le texte choisi par le visiteur. Les traits, les ombres et l'écart entre les lettres restent en pixels. `height: screen` : tout l'écran, au moins ; sur un téléphone, la hauteur vraiment visible.
- **La vue points se lit au lecteur d'écran** : rien à écrire. La page reste sous les points, invisible mais lisible ; le lecteur annonce « Vue points » et « Vue web » ; Tab ramène la vue web.

Ces ajouts sont décidés (`ADR-061`). Les leçons vont de `77-toutes-les-touches.holo` à `81-vue-points-et-lecteur-d-ecran.holo`.

## 6 undecies. Les repères, la superposition, le survol, le texte qui grandit

**Les repères.** Une personne aveugle saute d'un repère à l'autre avec son lecteur d'écran. Rien ne change à l'œil.

```holo
Page(
  title: "Landmarks",
  children: [
    Header(children: [
      Text("My studio"),
      Nav(children: [ A("Home", to: "home.holo") ]),
    ]),
    Main(children: [ H1("Welcome"), "The main content." ]),
    Footer(children: [ Text("Made in HoloCode") ]),
  ],
)
```

- `Header` : l'en-tête ; `Nav` : un menu ; `Main` : le contenu principal ; `Footer` : le pied de page.
- `Header` et `Footer` posés directement dans la page sortent du contenu principal, comme il se doit. `Main` ne se pose que directement dans la page. Sans `Main`, tout le contenu de la page est le contenu principal.
- Ils vont bien dans un composant partagé (`Component`) : le menu d'un site s'écrit une fois.

**La superposition.** `Stack` pose ses enfants les uns sur les autres. Le premier donne la taille ; les autres se posent dessus, à la place dite par `align:` (`topLeft`, `top`, `topRight`, `left`, `center`, `right`, `bottomLeft`, `bottom`, `bottomRight` ; au centre sans rien dire).

```holo
Page(
  title: "A badge",
  children: [
    Stack(children: [
      Image(source: "star.svg", alt: "A golden star"),
      Text("New", align: topRight),
    ]),
  ],
)
```

**Le survol, le focus, l'appui.** Dans un style, un état dit ce qui change : `hover: { … }` (la souris passe dessus), `focus: { … }` (on y arrive au clavier), `active: { … }` (pendant l'appui).

```holo
Page(
  title: "Hover",
  children: [ Button.cta(name: Go, text: "Go") ],
)

.cta {
  background: #E9B44C;
  hover: { background: #ffd27a; }
  focus: { border: 3px solid white; }
  active: { opacity: 0.7; }
}
```

- Le changement se fait en douceur (0,15 s). Sur un écran tactile, le survol n'existe pas : il ne reste pas collé après un toucher.
- Dans un état, les mêmes réglages qu'ailleurs. Pas de sélecteur : un état appartient à son style.

**Le texte qui grandit.** Rien à écrire : une taille de texte écrite en pixels suit le réglage « texte plus grand » du visiteur (le moteur l'écrit en `rem` : 16px = 1rem). Un titre de plus de 24px rétrécit sur un écran plus étroit que la page, sans passer sous 24px : un grand titre ne déborde plus d'un téléphone.

Ces quatre ajouts sont décidés (`ADR-036`). Les leçons sont `35-reperes.holo`, `36-titres-profonds.holo`, `37-survol.holo`, `38-superposition.holo`.

## 6 decies. Le mouvement : `Enter`, `Loop`, `Scenes`

```holo
Page(
  title: "Motion",
  children: [
    H1("Hello", enter: Enter(y: 40px, opacity: 0, letters: 0.05s, ease: spring)),
    Shape(form: circle, color: "#E9B44C", size: 60px,
      enter: Enter(scale: 0, at: 0.6s, ease: back),
      loop: Loop(scale: 1.2, for: 0.8s)),
    Scenes(height: 200px, repeat: forever, children: [
      Scene(for: 2s, children: [ H2("One", enter: Enter(x: -200px, opacity: 0)) ]),
      Scene(for: 2s, children: [ H2("Two", enter: Enter(scale: 3, opacity: 0, blur: 10px)) ]),
    ]),
  ],
)
```

- **`enter: Enter(…)`**, sur n'importe quel bloc qui se voit : il arrive. On écrit **d'où il part** ; il arrive à sa place.
- **`loop: Loop(…)`** : il vit sans fin. On écrit **où il va** ; il y va et revient. `back: false` : il recommence sans revenir (pour tourner).
- Ce qui bouge : `opacity` (0 à 1), `x`, `y` (en px), `scale`, `rotate` (en deg), `flip` et `tilt` (tourner en profondeur, en deg), `blur` (en px), `hue` (la couleur, en deg), `round` (0 à 50 : un carré devient rond).
- Quand et comment : `at:` (le moment où il part), `for:` (combien de temps), `ease:` (le caractère : `linear`, `smooth`, `out`, `in`, `back`, `spring`, `bounce`).
- **`letters: 0.05s`** sur un texte : lettre après lettre. **`each: 0.1s`** sur un bloc qui a des enfants : l'un après l'autre.
- **`Scenes(height:, repeat: forever, children: [ Scene(for: 3s, children: [ … ]) ])`** : des scènes qui passent l'une après l'autre, au même endroit. Dans une scène, `at:` compte depuis son début.
- Rien ne bouge si l'on n'écrit rien. Le visiteur qui demande moins de mouvement voit la page arrêtée (pour des scènes : la dernière).
- Tout devient du CSS fabriqué par le moteur : la page bouge sans attendre le moteur, et ne pèse rien de plus.

Limites : pas de chemin à suivre (une courbe dessinée), pas de mouvement qui répond à la souris, pas de particules par centaines ; la couleur change par `hue`, pas vers une couleur choisie.

Cette écriture est décidée (`ADR-034`). Les leçons sont `32-entrer.holo`, `33-boucle.holo`, `34-scenes.holo`. Le film complet : `exemples/motion/holocode/showreel.holo`, et son jumeau en HTML, CSS et JavaScript.

## 7. Comment la page se regarde : `Zoom`, `Points`, `Relief`

Quand le visiteur zoome sur la page (Ctrl + molette, ou pincer), elle grossit d'abord comme n'importe quel site : le texte reste du texte, on le lit, on le sélectionne, on le copie. Au-delà du grossissement fixé par `Points(after:)`, chaque pixel devient un point lumineux, qui se morcelle ensuite. Dès que la page est grossie, glisser la déplace, dans tous les sens ; c'est le même geste avant et après le passage aux points. Quand il tourne la page, elle prend du relief. Ces trois blocs règlent cela.

**Ce qui met la page en 3D s'active.** Sans rien écrire, une page est un site ordinaire : on peut la grossir pour mieux lire, et rien d'autre ne se passe. C'est alors le navigateur qui la grossit, sur place : elle reste accrochée (`ADR-069`).

| Pour avoir | L'auteur écrit |
|---|---|
| les pixels qui deviennent des points au zoom | `points: Points()` (ou un site planté dans un pixel, avec `pixels:`) |
| la page qui tourne, et son relief | `relief: Relief(tilt: 360deg)`, en plus de `points:` |
| la page qui se décroche, comme une feuille, quand on zoome | `zoom: Zoom(detach: true)` : le bouton « Décrocher » du menu ☰ |

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
    divisions: 4,
    levels: 20,
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
| `Zoom(detach:)` | `true` : le menu ☰ offre « Décrocher » ; la page se détache comme une feuille, et le zoom l'approche ; « Accrocher » la remet à sa place. Refusé avec les points, `shrink: true` ou `active: false`, qui gardent le zoom au moteur. Décidé (`ADR-069`). | `true`, `false` |
| `Points(after:)` | Jusqu'à ce grossissement, la page reste un site ordinaire. Jamais moins de 2 : tout visiteur peut au moins doubler la taille du texte. | 2 à 16 |
| `Points(size:)` | La taille où un pixel devient un point. | 2px à 32px |
| `Points(fragment:)` | La taille où un point se morcelle. | 8px à 400px, au moins `size` × `divisions` |
| `Points(divisions:)` | Un point se morcelle en `divisions` × `divisions` : avec `4`, chaque côté est coupé en quatre, soit 16 morceaux. | 2 à 8 |
| `Points(levels:)` | Combien de fois de suite. Même idée que `Zoom(levels:)` : combien de fois l'un dans l'autre. | 0 à 20 |
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

Toutes les limites, telles que le moteur les applique (chacune refusée avec un message, ou coupée sans danger) :

| Quoi | La limite |
|---|---|
| Le fichier | 262 144 octets ; 100 000 mots ; 64 niveaux de blocs et de listes l'un dans l'autre ; 16 fichiers importés |
| Les valeurs (`State`) | 32 valeurs par page ; un nombre jusqu'à 1 000 000 000, avec au plus 6 chiffres après la virgule (`ADR-066`) ; un texte de 2 000 caractères |
| Une liste qui change (`State(tasks: [])`) | 200 éléments ; un texte de 200 caractères ; 16 champs par élément |
| `Repeat(items:)` | 200 éléments ; 20 000 blocs une fois déplié |
| Les composants | 16 paramètres ; 8 composants l'un dans l'autre ; 2 000 copies |
| Le temps | `Every` et `After` : de 100 ms à 3 600 s ; `Data(every:)` : de 1 s à 3 600 s |
| Les données reçues (`Data`) | 64 Ko : au-delà, elles sont refusées, et la lecture s'arrête dès qu'elles dépassent ; 10 secondes pour arriver ; une lecture à la fois, une seconde au moins entre deux (`ADR-064`) |
| Les modules | 8 par page ; un fichier de 4 Mo, refusé dès qu'il dépasse ; un temps de 10 ms à 5 s ; une mémoire de 64 Ko à 16 Mo |
| Un fichier envoyé par un formulaire | 10 Mo au plus (`max:` de 1 KB à 10 MB) |
| La vue points | 200 000 points à l'écran |

## 9. Les unités

Une unité se colle au nombre : `500KB`, jamais `500 KB`.

| Unités | Pour |
|---|---|
| `px`, `%` | Les tailles à l'écran. Écrites en pixels, elles suivent la taille du texte choisie par le visiteur : le moteur les écrit en `rem` (`ADR-061`) |
| `screen` | `height: screen` : tout l'écran, au moins |
| `deg` | Les angles |
| `B`, `KB`, `MB`, `GB` | Les poids (décimaux : 1 KB = 1 000 octets) |
| `mm`, `cm`, `m`, `km`, `ms`, `s`, `min`, `h` | Longueurs et durées : lues par le moteur, pas encore employées |

## 10. Aide-mémoire

| Bloc | Réglages | Où |
|---|---|---|
| `Page` | `name`, `title`, `lang`, `description`, `image`, `icon`, `fonts`, `children`, `pixels`, `rules`, `state`, `shared`, `prices`, `keep`, `data`, `zoom`, `points`, `relief`, `portals` | À la racine |
| `H1` à `H6`, `P`, `Text` | le texte entre guillemets ; `name` | Dans `children` |
| `Header`, `Nav`, `Footer` | `children`, `name` | Dans `children` ; `Header` et `Footer` posés directement dans la page en sont l'en-tête et le pied |
| `Main` | `children`, `name` | Directement dans la page |
| `Stack` | `children`, `name` ; ses enfants prennent `align` | Dans `children` |
| `A` | le texte entre guillemets, `to` | Dans `children` |
| `Image` | `source`, `alt` (obligatoire), `caption`, `phone`, `weight`, `name` | Dans `children` |
| `Video` | `source`, `label`, `weight`, `name` | Dans `children` |
| `Table` | `caption`, `head`, `rows`, `name` | Dans `children` |
| `Choice` | `value`, `label`, `options`, `menu`, `name` | Dans `children` |
| `Sound` | `name`, `source`, `weight` ; avec `label`, un lecteur | Dans `children` |
| `Slider` | `value`, `label`, `min`, `max`, `name` | Dans `children` |
| `Progress` | `value`, `max`, `label`, `name` | Dans `children` |
| `Details` | `summary`, `children`, `open`, `name` | Dans `children` |
| `Dialog` | `name`, `children` ; capacités `open`, `close` | Dans `children` |
| `Form` | `name`, `children` ; capacité `send` ; signaux `sent`, `failed` | Dans `children` |
| `Shape` | `form`, `color`, `size`, `name` | Dans `children` |
| `Scenes` | `children`, `height`, `repeat`, `name` | Dans `children` |
| `Scene` | `for`, `children`, `name` | Dans `Scenes` |
| `Enter`, `Loop` | `opacity`, `x`, `y`, `scale`, `rotate`, `flip`, `tilt`, `blur`, `hue`, `round`, `at`, `for`, `ease`, `letters`, `each` ; `back` pour `Loop` | Dans `enter:` et `loop:`, sur tout bloc qui se voit |
| `Hr` | aucun | Dans `children` |
| `Quote` | le texte entre guillemets, `by` | Dans `children` |
| `Code` | le texte entre guillemets | Dans `children` |
| `If` | le nom d'une valeur, puis `is`, `not`, `over`, `under`, et `children` (avec `else`) ou `rules` | Dans `children`, ou dans `rules` |
| `List` | `children`, `ordered`, `name` | Dans `children` |
| `Button` | `name`, `text` | Dans `children` |
| `Point` | `name`, `seed`, `brightness`, `fragments`, `color`, `palette`, `budget`, `inside` ; `above` quand il est planté dans un pixel | Dans `children` ou `pixels`, ou à la racine |
| `World` | `children`, `pixels`, `rules` | Dans `inside:` d'un `Point` |
| `Row`, `Column` | `children`, `gap`, `align`, `name` ; leurs enfants prennent `grow` | Dans `children` |
| `Grid` | `children`, `gap`, `columns`, `name` | Dans `children` |
| `Board` | `children`, `height`, `name` ; ses enfants prennent `x`, `y` et `drag` | Dans `children` |
| `Input` | `value`, `label`, `max`, `lines`, `type` (`date`, `time`, `color`), `name` | Dans `children` |
| `Checkbox` | `value`, `label`, `name` | Dans `children` |
| `Component` | `name`, `params` (avec des valeurs par défaut), `emits`, `children`, `rules` | Dans `components:` d'une `Page`, ou à la racine d'un fichier importé |
| un composant (`ArticleCard`) | `name`, et ses paramètres ; des noms de style à l'appel (`ArticleCard.promo`) | Dans `children` |
| `Use` | le nom d'un morceau importé | Dans `children` |
| `On` | le signal, puis `effect:` | Dans `rules` |
| `Every` | le rythme, puis `effect:` | Dans `rules` |
| `After` | la durée, puis `effect:` | Dans `rules` |
| `Repeat` | `items`, `children`, `rules` | Dans `children` |
| `Font` | `family`, `source` | Dans `fonts:` d'une `Page` |
| `Module` | `name`, `source`, `input`, `output` (un nom, ou une liste de noms, `ADR-077`), `time`, `memory` ; capacité `run` ; signaux `done`, `failed` | Dans `modules:` d'une `Page` ; annoncé en haut du fichier, `module "…"` |
| `Drawing` | `label`, `width`, `height`, `children`, `shapes` (une liste de formes, `ADR-088`) | Partout dans `children` ; contient des formes |
| `Rect`, `Circle`, `Line`, `Path` | `x`, `y`, `width`, `height`, `radius` ; `r` ; `from`, `to` ; `d` ; et `fill`, `stroke`, `thickness`, `opacity` | Seulement dans un `Drawing` |
| `Chart` | `kind` (`bars`, `line`, `pie`), `over`, `value`, `label`, `title`, `color` | Partout dans `children` |
| `Item` | `key`, et les champs de l'élément | Dans `items` d'un `Repeat` |
| `Repeat(over:)` | `over` (une liste de la page), `children`, `rules` | Dans `children` |
| `When` | le nom d'une valeur, puis `is`, `not`, `over`, `under` ; ou le nom d'un bloc, puis `meets` et `within` ; et `effect:` | Dans `rules` |
| `State` | les valeurs et leur départ : `cart: 0` | Dans `state:` d'une `Page` |
| `Shared` | les valeurs que le serveur garde pour tous, et leur départ : `seats: 20` | Dans `shared:` d'une `Page` |
| `Data` | `name`, `from`, `every` | Dans `data:` d'une `Page` |
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
| Le morcellement des points d'une page | `Points(fragment:, divisions:, levels:)` | fait |
| Le relief | `Relief(height:)` | fait |
| Activer la rotation de la page, en faire le tour | `Relief(tilt:)` | fait |
| Une valeur que la page retient | `state: State(cart: 0)` | fait |
| Afficher une valeur | `{cart}` dans un texte | fait |
| Changer une valeur | les demandes `add`, `sub`, `set` | fait |
| Recevoir des valeurs d'un serveur | `data: Data(from: "stock.json", every: 30s)` | fait |
| Réutiliser un morceau de page et un thème | `import "commun.holo"`, `Component(name:)`, `Use(Menu)` | fait |
| Un composant à paramètres, restylé par le CSS | `Component(name:, params:, emits:, children:, rules:)`, `ArticleCard(…)`, `ArticleCard { }`, `ArticleCard.promo(…)` | fait |
| Plusieurs noms de style sur un bloc | `P.card.big(…)` | fait |
| Répéter une règle dans le temps | `Every(1s, effect:)` | fait |
| Le clavier | `On(Key.left, effect:)`, `Key.enter`, `Key.escape`, `Key.a` à `Key.z`, `Key.digit0` à `Key.digit9` | fait |
| Agir au moment où une valeur atteint quelque chose | `When(lives, is: 0, effect:)` | fait |
| La rencontre de deux objets | `When(Basket, meets: Apple, within:, effect:)` | fait |
| Faire glisser un objet | `drag: true` sur un bloc d'un `Board` | fait |
| Plusieurs demandes dans une règle | `effect: [a.add(1), b.set(0)]` | fait |
| Le hasard | la demande `random` | fait |
| Placer librement | `Board`, et `x:`, `y:` sur ses enfants | fait |
| Une valeur qui est un texte | `State(buyer: "")`, `{buyer}`, `If(buyer, not: "")` | fait |
| Écrire un texte | `Input(value: buyer, label:, max:)` | fait |
| Écrire un nombre, cocher une case | `Input(value:, label:, max:)`, `Checkbox(value:, label:)` | fait |
| Garder une valeur d'une visite à l'autre | `keep: [cart]` | fait |
| Montrer ou cacher selon une valeur | `If(cart, is:, not:, over:, under:)` | fait |
| Un trait, une citation, du texte tel quel | `Hr()`, `Quote(by:)`, `Code`, les accents graves | fait |
| Le retour à la ligne | un texte entre trois guillemets | fait |
| Le texte qui remplace une image | `Image(alt:)` | fait |
| Des prix, un nombre d'articles, un total | `prices: Prices(...)`, `{count}`, `{total}` | fait |
| Activer ou désactiver le zoom | `Zoom(active:)` | fait |
| Les limites du zoom | `Zoom(max:, shrink:)` | fait |
| La vitesse du zoom | `Zoom(speed:)` | fait |
| Le nombre de sites emboîtés | `Zoom(levels:)` | fait |
| Entrer dans un site, en sortir | `enter`, `leave` | fait |
| Le carrefour, les portails | `Portals(layout:, count:, size:, brightness:)`, et la capacité `portals` | fait |
| La durée d'ouverture d'un portail | `Portals(duration:)` | fait |
| Le poids permis | `budget`, `weight` | fait |
| Le toucher | le signal `tap` | fait |
| Le son | `Sound(name:, source:, volume:, loop:)`, les capacités `play` et `stop` | fait |
| Apparaître en descendant | `Enter(…, inView: true)` | fait |
| Des tailles qui suivent le visiteur, tout l'écran | les pixels écrits en `rem`, `height: screen` | fait |
| La vue points au lecteur d'écran | rien à écrire | fait |
| Une forme simple | `Shape(form:, color:, size:)` | fait |
| Comparer deux valeurs, fixer d'après une autre | `over: best`, `best.set(score)` | fait |
| Le survol | `On(Carte.hover, …)`, `On(Carte.hoverEnd, …)` | fait |
| Le « sinon » | `If(…, else: [ … ])` | fait |
| Une seule fois, plus tard | `After(3s, effect: …)` | fait |
| La date et l'heure du visiteur | `year`, `month`, `day`, `weekday`, `hour`, `minute` | fait |
| L'approche d'un personnage, en profondeur | aucun | à faire |
| Écrire une fois, répéter pour chaque élément | `Repeat(items: [ Item(…) ], children: [ … ])`, `item` | fait |
| Une couleur nommée, le thème sombre, le téléphone | `--or`, `dark: { … }`, `phone: { … }` | fait |
| Sa propre police | `fonts: [ Font(family:, source:) ]` | fait |
| Une police du moteur, pour toutes les écritures | `fonts: [ Font(family: "Inter") ]` | fait (`ADR-092`) |
| Envoyer un formulaire | `Form(name:)`, `Contact.send`, `sent`, `failed` | fait |
| Un compte, une page réservée aux membres | `Page(access: members)`, `signedIn`, `{account}`, `A(to: "/account/signin")` | fait, avec `holo serve` (`ADR-081`) |
| Multiplier, diviser | les demandes `mul`, `div` | fait |
| Écrire un nombre joliment | `{minute:00}`, `{n:number}`, `{n:cents}`, `{weekday:name}` | fait |
| Une liste qui change pendant la visite | `State(tasks: [])`, `push`, `remove(item)`, `clear`, `Repeat(over:)` | fait |
| Une liste d'articles à champs, aussi reçue du serveur | `State(articles: [ Item(…) ])`, `{item.title}`, `push(Item(…))`, `Data` | fait |
| Une valeur partagée par tous les visiteurs, vue en direct | `shared: Shared(seats: 20)`, `seats.sub(1)` par un toucher | fait |
| Du code enfermé (un module WebAssembly) | `module "…"`, `Module(…)`, `run`, `done`, `failed` | fait |
| Une fenêtre, un pli, une glissière, une barre | `Dialog`, `Details`, `Slider`, `Progress` | fait |
| Réagir au zoom par une règle (« quand on zoome, alors… ») | aucun | à faire |
| Ranger côte à côte, l'un sous l'autre, en grille | `Row`, `Column`, `Grid` | fait |
| L'écart et le placement | `gap:`, `align:`, `columns:` | fait |
| La place qui reste | `grow:` dans `Row` ou `Column` | fait |
| Un thème partagé par les pages | un fichier de styles seuls, `import "theme.holo"` | fait |
| Réutiliser un morceau de fichier (les imports) | `import "commun.holo"`, `Component`, `Use` | fait |
| Le personnage | aucun | à faire |

L'exemple le plus complet : [`exemples/boutique-comparee/boutique.holo`](../../exemples/boutique-comparee/boutique.holo).

## 11. Ce qui n'existe pas encore

- Un dessin n'a pas encore de texte ni de dégradé ; une liste de formes en garde deux cents au plus.
- Les données venues d'un autre serveur.
- Pour les valeurs partagées : un champ qui en change une, une liste partagée, une limite au nombre de touchers d'un visiteur.
- Pour les comptes (`ADR-081`) : pas encore de clés d'accès (passkeys), de QR code pour activer le code, de codes de secours, ni de mot de passe changé ou de compte effacé par son membre.
- Pour les valeurs : pas de nombre négatif ; une heure seule (« 14:30 ») ne se compare pas ; une valeur calculée d'après d'autres (un total qui suit tout seul) reste à faire, hors `Filter` et `Days` ; une fiche de liste ne prend pas de nombre à virgule (son prix s'écrit en centimes, `{item.price:cents}`).
- Le reste du Markdown (seuls le gras et l'italique sont rendus).
- Entrer dans un point écrit à l'intérieur d'un monde.
- Les garde-fous de zoom pour un `Point` seul : ils sont encore fixés dans le moteur.
