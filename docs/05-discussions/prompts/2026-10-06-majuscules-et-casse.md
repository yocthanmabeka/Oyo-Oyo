# Prompt pour ChatGPT et pour Gemini — les majuscules et la casse de HoloCode (2026-10-06)

À copier en entier, une fois chez ChatGPT (conversation simple) et une fois chez Gemini. Tout ce qu'il faut est dans ce texte.

---

**Consigne : ne travaille pas.** Pas de code, pas de mode agent, pas de fichier. On te demande **un avis tranché et une comparaison**, en français, en phrases simples. Sois franc.

## Le projet en quelques lignes

Holoverse veut réinventer le web en métavers léger. Un fichier `.holo`, écrit en **HoloCode**, est un site normal par défaut ; quand on zoome, les pixels deviennent des points, et chaque point contient un monde où l'on entre. On doit pouvoir créer **sans programmer**. Le responsable du projet, Yocthan, vient de Flutter ; les personnes visées ne programment pas. L'auteur n'écrit jamais de HTML, de CSS ni de JavaScript.

```
import "commun.holo"

Page(
  title: "Ma boutique",
  state: State(panier: 0, apple_x: 50),
  children: [
    Use(Menu),
    H1("Ma boutique"),
    Stack(children: [
      Image(source: "tableau.svg", alt: "Un tableau"),
      Text.pastille("Nouveau", align: top_right),
    ]),
    Button.action(name: Ajouter, text: "+"),
    Text("{panier} au panier"),
  ],
  rules: [
    On(Ajouter.tap, effect: panier.add(1)),
    On(Key.left, effect: apple_x.sub(8)),
  ],
)

.action {
  background: #8a3b12;
  font-size: 18px;
  hover: { background: #a5481a; }
}
```

## La question

**HoloCode doit-il respecter la casse** (faire la différence entre `Page`, `page` et `PAGE`), oui ou non ? Et **pour chaque sorte de mot, majuscule ou minuscule** ?

## Les règles d'aujourd'hui

Aujourd'hui, HoloCode **respecte la casse**, et le vérificateur corrige avec un message :

| Ce qu'on écrit | Ce que répond le moteur aujourd'hui |
|---|---|
| `h1("a")` | refusé : « un nom de bloc commence par une majuscule, écris `H1` » |
| `page(…)` | refusé : « écris `Page` » |
| `State(Cart: 0)` | refusé : « le nom d'une valeur s'écrit en minuscules, comme `cart` » |
| `p { color: red; }` (style) | refusé : « écris `P { … }` » |
| `H1 { Color: red; }` | refusé : « réglage inconnu `Color` ; réglages possibles : color, … » |
| `On(B.Tap, …)` | refusé : « signal inconnu `Tap` : un `Button` émet `tap` » |
| `Shape(form: Circle)` | refusé : « attend l'un de ces mots : circle, square, triangle, diamond » |
| `weight: 1kb` | refusé : « unité inconnue `kb` ; unités possibles : …, B, KB, MB, GB, px, deg » |
| `Page(Title: "a", …)` | **accepté sans rien dire, et le titre est perdu** (un défaut) |
| `Button(name: buy, …)` | **accepté** : un nom de bloc peut commencer par une minuscule, alors que l'usage est une majuscule |

## Tous les mots du langage, par sorte

| Sorte de mot | Écriture d'aujourd'hui | Les mots |
|---|---|---|
| **Blocs** (ce qui existe) | Majuscule au début, mots collés | `Page`, `World`, `Part`, `Use`, `Text`, `P`, `H1` à `H6`, `A`, `Button`, `Image`, `List`, `Hr`, `Quote`, `Code`, `Header`, `Nav`, `Main`, `Footer`, `Row`, `Column`, `Grid`, `Stack`, `Board`, `Point`, `Shape`, `Sound`, `Input`, `Checkbox`, `If`, `On`, `Every`, `When`, `State`, `Prices`, `Data`, `Zoom`, `Points`, `Relief`, `Portals`, `Scenes`, `Scene`, `Enter`, `Loop` |
| **Réglages** (paramètres) | Minuscules, un seul mot | `name`, `title`, `children`, `rules`, `effect`, `text`, `to`, `source`, `alt`, `weight`, `by`, `ordered`, `value`, `label`, `max`, `gap`, `align`, `columns`, `height`, `x`, `y`, `drag`, `form`, `color`, `size`, `seed`, `brightness`, `fragments`, `palette`, `budget`, `inside`, `above`, `pixels`, `state`, `prices`, `keep`, `data`, `from`, `every`, `is`, `not`, `over`, `under`, `meets`, `within`, `enter`, `loop`, `at`, `for`, `ease`, `letters`, `each`, `back`, `repeat`, `opacity`, `scale`, `rotate`, `flip`, `tilt`, `blur`, `hue`, `round`, `zoom`, `points`, `relief`, `portals`, `active`, `shrink`, `levels`, `speed`, `after`, `fragment`, `grid`, `depth`, `density`, `layout`, `count`, `duration` |
| **Mots-valeurs** (choix tout faits) | Minuscules ; deux mots joints par `_` | `circle`, `square`, `triangle`, `diamond` ; `start`, `center`, `end`, `between` ; `top_left`, `top`, `top_right`, `left`, `right`, `bottom_left`, `bottom`, `bottom_right` ; `linear`, `smooth`, `out`, `in`, `back`, `spring`, `bounce` ; `forever` ; `grid`, `row`, `column`, `diagonal` ; `true`, `false` |
| **Signaux et capacités** (après un point) | Minuscules | `tap` ; `enter`, `leave`, `portals`, `play` |
| **Le clavier** | `Key` avec majuscule, touche en minuscules | `Key.left`, `Key.right`, `Key.up`, `Key.down`, `Key.space` |
| **Demandes** (changer une valeur) | Minuscules, avec parenthèses | `add`, `sub`, `set`, `random` |
| **Valeurs calculées** | Minuscules, entre accolades | `{count}`, `{total}` |
| **Noms choisis par l'auteur** | Blocs nommés : majuscule (`Ajouter`, `Workshop`) ; valeurs : minuscules, mots joints par `_` (`panier`, `apple_x`) ; styles : minuscules (`.action`) | |
| **Réglages de style** | Minuscules, mots joints par `-`, comme en CSS | `color`, `background`, `font-size`, `font-weight`, `font-style`, `font-family`, `text-align`, `border`, `border-radius`, `padding`, `margin`, `width`, `height`, `max-width`, `opacity` |
| **États de style** | Minuscules | `hover`, `focus`, `active` |
| **Unités** | Minuscules, sauf les octets | `px`, `deg`, `ms`, `s`, `min`, `h`, `mm`, `cm`, `m`, `km` ; `B`, `KB`, `MB`, `GB` |
| **Mot-clé du fichier** | Minuscules | `import` |

## L'avis de Claude (l'IA qui programme le projet)

**Sur la casse : la respecter, et corriger avec un message.** Ne pas l'ignorer. Raisons :

1. **Une seule écriture par mot.** Si `Page`, `page` et `PAGE` étaient tous acceptés, trois personnes écriraient la même page de trois façons ; un débutant qui copie un exemple ne saurait plus laquelle est la bonne, et la recherche dans un fichier deviendrait incertaine.
2. **La majuscule porte un sens.** `Ajouter` est un bloc (une chose qu'on touche), `panier` est une valeur (un nombre qui change) : on voit la différence d'un coup d'œil dans `On(Ajouter.tap, effect: panier.add(1))`.
3. **Le défaut à éviter n'est pas la casse, c'est le silence.** HTML accepte `<P>` et `<p>`, mais CSS ignore sans rien dire `Color: red`. Une erreur de casse doit être **refusée avec le bon mot à écrire**, pas avalée.

**Ce avec quoi Claude n'est pas d'accord dans le langage d'aujourd'hui :**

- **`Page(Title: …)` accepté sans rien dire** : c'est exactement le défaut de CSS que HoloCode voulait éviter. Il faut le refuser : « écris `title` ».
- **`name: buy` accepté** : un nom de bloc devrait avoir une majuscule, comme un bloc, pour garder la règle « ce qu'on touche a une majuscule, ce qui change n'en a pas ».
- **Trois façons de joindre deux mots** : `apple_x` et `top_right` (avec `_`), `font-size` (avec `-`), `BlueDoor` (collés, pour les noms de blocs). Le `-` vient de CSS, et Yocthan a voulu écrire les styles comme du CSS. Mais `top_right` et `apple_x` pourraient s'écrire autrement. Claude n'a pas de certitude : c'est la question la plus ouverte.
- **Les unités d'octets en majuscules** (`KB`) quand toutes les autres sont en minuscules (`px`, `ms`). C'est l'usage (`kb` voudrait dire kilobit), donc Claude le garderait, mais c'est une exception à expliquer.
- **`H1` à `H6` avec une majuscule**, quand HTML écrit `h1` : Claude le garde, parce que tous les blocs commencent par une majuscule. C'est une seule règle, sans exception.

**Ce avec quoi Claude est d'accord :** blocs en majuscule, réglages en minuscules, mots-valeurs en minuscules, styles en minuscules comme en CSS, `Key` en majuscule (c'est une chose, le clavier).

## Ce qu'on te demande

Réponds en moins de 900 mots :

1. **Respecter la casse, oui ou non ?** Compare trois choix : (a) la respecter et refuser avec le bon mot, comme aujourd'hui ; (b) l'ignorer (`page` = `Page`) ; (c) la respecter, mais qu'un éditeur ou le moteur corrige tout seul. Donne des langages réels qui ont fait chacun de ces choix (par exemple SQL, HTML, Python, Visual Basic, Nim, Flutter), et ce qu'en pensent leurs utilisateurs. Lequel convient à des **non-programmeurs** ?
2. **Pour chaque ligne du tableau des mots**, garde-t-on l'écriture d'aujourd'hui ? Si non, laquelle proposes-tu, et pourquoi ?
3. **Joindre deux mots** : `apple_x` / `top_right`, `font-size`, ou `appleX` / `topRight` ? Une seule règle pour tout le langage, ou des règles différentes selon la sorte de mot ?
4. **Les trois désaccords de Claude** (`Title:` accepté en silence, `name: buy` accepté, `KB` en majuscules) : as-tu le même avis ?
5. **Un mot mal écrit** : faut-il seulement refuser avec le bon mot, ou aussi proposer de le corriger d'un clic dans l'éditeur ?

Termine par **un tableau** : sorte de mot, l'écriture que tu recommandes, un exemple, ta raison en une phrase.
