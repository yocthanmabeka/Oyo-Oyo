# ADR-021 — La façon dont une page se regarde s'écrit dans le fichier : `Zoom`, `Points`, `Relief`

- Statut : ACCEPTÉ — noms et écriture tranchés le 2026-10-06 (`ADR-047`) : `Points(grid:)` devient `divisions:`, `Points(depth:)` devient `levels:`
- Date : 2026-10-03
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-03 (la mosaïque, la profondeur, « ajoute tout ce qu'on vient de faire au langage »)
- Validation : principe demandé par Yocthan le 2026-10-03 ; écriture à confirmer par lui.
- Projets affectés : HoloCode, HoloEngine

## Contexte

En une journée, le moteur a appris à transformer une page en points : chaque pixel devient un point quand on zoome, les points se morcellent, la page prend du relief quand on la tourne. Tout cela était réglé par des nombres écrits dans le moteur. Yocthan a relevé deux choses.

- Rien de tout cela n'apparaissait dans les fichiers `.holo` : « on a programmé tout un monde, mais ici je ne vois même pas les traces de ces mondes-là. Comment les gens vont-ils faire pour programmer ? »
- Il faut des garde-fous. Un visiteur qui ne fait que zoomer, ou que dézoomer, ne doit pas pouvoir dépasser des limites ; et ces limites doivent être programmées par l'auteur. Réduire le site jusqu'à la taille d'un pixel doit être possible, mais seulement si l'auteur l'active.

## Décision

**Ce qui règle la vue d'une page s'écrit dans la page**, avec trois blocs. Sans eux, la page prend les réglages habituels.

```holo
Page(
  title: "My shop",

  zoom: Zoom(
    max: 1000000,     // on ne grossit pas la page plus d'un million de fois
    shrink: false,    // true : dézoomer réduit la page jusqu'à ce qu'elle soit un seul point
  ),

  points: Points(
    size: 6px,        // un pixel devient un point lumineux quand il atteint cette taille
    fragment: 40px,   // un point se morcelle quand il atteint cette taille…
    divisions: 4,     // …en une grille de 4 × 4…
    levels: 20,       // …et cela au plus vingt fois de suite
    density: 2,       // points par pixel d'écran, dans chaque sens
  ),

  relief: Relief(
    height: 10px,     // la hauteur du relief quand la page est de biais
    tilt: 360deg,     // jusqu'où l'on peut tourner la page ; 360deg : on en fait le tour
  ),
)
```

**Deux étages de garde-fous.**

1. Ceux de l'auteur, écrits ci-dessus : le visiteur ne peut pas les dépasser.
2. Ceux du langage, que l'auteur ne peut pas dépasser : chaque réglage a des bornes, et le vérificateur refuse le fichier s'il en sort.

| Réglage | Bornes | Par défaut |
|---|---|---|
| `Zoom(max:)` | 1 à 1 000 000 000 000 | pas de limite autre que la profondeur |
| `Zoom(shrink:)` | `true` ou `false` | `false` |
| `Zoom(levels:)` | 1 à 16 | 8 |
| `Zoom(speed:)` | 0.25 à 4 | 1 |
| `Points(after:)` | 2 à 16 | 4 |
| `Zoom(active:)` | `true` ou `false` | `true` |
| `Portals(layout:)` | `grid`, `row`, `column`, `diagonal` | `grid` |
| `Portals(count:)` | 1 à 64 | 12 |
| `Portals(size:)` | 80px à 400px | 170px |
| `Portals(brightness:)` | 0 à 1 | 0,15 |
| `Portals(duration:)` | 0ms à 2000ms | 450ms |
| `Points(size:)` | 2px à 32px | 6px |
| `Points(fragment:)` | 8px à 400px, et au moins `size` × `divisions` | 40px |
| `Points(divisions:)` (`grid:` jusqu'au 2026-10-06) | 2 à 8 | 4 |
| `Points(levels:)` (`depth:` jusqu'au 2026-10-06) | 0 à 20 | 20 |
| `Points(density:)` | 1 à 3 | 2 |
| `Relief(height:)` | 0px à 40px | 10px |
| `Relief(tilt:)` | 0deg à 360deg | 0deg : la page ne tourne pas tant que l'auteur ne l'écrit pas |

La borne « `fragment` au moins égal à `size` × `divisions` » garantit qu'il n'y a jamais plus de points à dessiner que l'écran ne peut en montrer (`ADR-005`).

**Deux unités de plus dans le langage** : `px` (pixels d'écran) et `deg` (degrés).

**Ajouts du même jour.** `Zoom(levels:)` limite le nombre de sites emboîtés les uns dans les autres ; un fichier qui en emboîte davantage est refusé. `Points(after:)` fixe le grossissement jusqu'où la page reste un site ordinaire, qu'on lit et qu'on copie : Yocthan veut que la « métaversification » ne commence qu'à partir d'une certaine profondeur de zoom, pour qu'un visiteur qui zoome seulement pour mieux lire garde l'expérience qu'il connaît.

**Ajouts du 2026-10-04, demandés par Yocthan.** « La rotation n'est pas à 360 degrés, elle est bloquée à un certain angle » : `Relief(tilt:)` va maintenant jusqu'à `360deg`, et c'est la valeur par défaut (avant : 80deg au plus, 52deg par défaut). À partir de `180deg`, la rotation est libre : on fait le tour de la page et on la voit par derrière, à l'envers, comme une feuille. En dessous, l'auteur garde sa limite. Deux réglages s'ajoutent, parce qu'ils existaient dans le moteur sans mot pour les écrire : `Zoom(speed:)`, la vitesse du zoom à la molette, et `Portals(duration:)`, le temps que met un portail à s'ouvrir, première utilisation de l'unité `ms`. Options écartées : un bloc à part pour la rotation (`Rotation(max:)`), qui aurait fait deux mots pour une seule chose, et un mot `free` à la place d'un angle, qui aurait fait deux écritures. Défaut évité : en CSS, `transition-duration` accepte `s` et `ms`, et un nombre sans unité est ignoré en silence ; ici une seule unité, et l'oubli est refusé.

**La rotation s'active (2026-10-04, plus tard le même jour).** Yocthan : « Il y a certaines propriétés ou fonctions qui doivent être activées, pour qu'il y ait de la cohérence entre les sites et le métavers. Si d'autres peuvent donner tout le temps en 3D, ça va déranger la lisibilité du site. » Sans `tilt`, une page ne tourne donc plus (avant : 360deg d'office). Écrire `Relief(tilt: 360deg)` active la rotation ; le bouton « Tourner » apparaît alors dès la page de face, sans passer d'abord par la vue points. Option écartée : un interrupteur à part (`Relief(active: true)`), qui aurait fait deux façons de dire « ne tourne pas » (`active: false` et `tilt: 0deg`). Question laissée à Yocthan : le passage en points au zoom (`Points(after:)`) reste offert d'office ; doit-il lui aussi s'activer ?

**Le carrefour et l'interrupteur du zoom.** `Zoom(active:)` permet ou interdit le zoom. Un quatrième bloc, `Portals(layout:, count:, size:, brightness:)`, règle le carrefour : la disposition des portails (`grid`, `row`, `column`, `diagonal`, donc aussi le sens où on les fait défiler), leur nombre, leur taille, la lumière du fond. La page gagne une capacité, `portals`, pour ouvrir le carrefour par une règle. Demandé par Yocthan le 2026-10-03 : « chaque action doit être dans le code ».

**Les points s'activent eux aussi (2026-10-04, le soir).** Claude avait laissé la question à Yocthan ; Gemini a répondu que le passage en points doit être demandé par l'auteur, comme la rotation, et Yocthan a suivi cette recommandation. Sans `points:`, une page ne devient plus jamais des points : le zoom reste un zoom de lecture. Planter un site dans un pixel (`pixels:`) active aussi les points, puisqu'on ne le trouve qu'ainsi. `relief:` sans points est refusé. Et `Points(after:)` ne descend plus sous 2 : une règle d'accessibilité (WCAG 1.4.4) demande que le texte puisse doubler de taille en restant du texte.

**Corrections après la revue de Codex (2026-10-04).** `Zoom(max:)` ne bornait que la vue points : le zoom ordinaire passait outre. Il borne maintenant le zoom entier, et un fichier où `Points(after:)` dépasse `Zoom(max:)` est refusé. `Portals(count:)` ne bornait que les mondes calculés : il borne tout le carrefour. `density` est plafonnée par un nombre total de points (huit millions). Codex propose aussi de renommer la plupart des mots de cette fiche (`maxScale`, `threshold`, `subdivideAt`, `divisions`, `PointView`, `Depth`…) : c'est à Yocthan de trancher, rien n'est renommé.

## Comparaison faite avant de choisir

| Option | Pour | Contre |
|---|---|---|
| Des blocs dans la page (retenue) | Même écriture que le reste ; vérifiés comme le reste ; un bloc par sujet | Trois mots de plus |
| Des réglages dans le style, à la CSS (`Page { zoom-max: … }`) | Rien de nouveau à apprendre | Un style ne dit que l'apparence (`ADR-017`, règle 3) ; ce sont des comportements |
| Rien dans le fichier, tout dans le moteur | Fichiers plus courts | C'est ce que Yocthan a refusé : l'auteur ne voit pas ce qu'il peut régler |

Défauts de HTML, CSS et JavaScript évités : le zoom d'une page web se règle par une balise `meta viewport` au format obscur (`user-scalable=no, maximum-scale=1`), sans vérification, et que les navigateurs ignorent souvent ; ici chaque réglage a un nom, une unité et des bornes, et une valeur refusée est dite avec sa ligne.

Mots : `fragment` reprend `fragments` (`ADR-016`), le mot retenu pour « morceler » ; `split`, refusé par Yocthan, n'est pas employé.

## Conséquences

### Positives

- L'auteur voit et règle ce que le moteur fait de sa page.
- Les mots `true` et `false` du langage ont enfin un emploi.

### Négatives et risques

- Le fichier d'exemple s'allonge de vingt lignes, toutes facultatives.
- Ces réglages décrivent la vue d'aujourd'hui (des points sur une image de la page). Si la vue change de nature, certains réglages changeront.

## Ce qui reste à faire

- Les mêmes garde-fous pour un `Point` seul (le Big Bang) : ses seuils de zoom sont encore écrits dans le moteur.
- La vue personnage et la vue « roadmap » ne sont pas encore décrites dans le langage.

## Critères de validation

- `exemples/boutique-comparee/boutique.holo` écrit les réglages habituels ; `exemples/zoom/reduire.holo` les change, et la page se réduit bien à un point.
- Tests du moteur : `moteur/src/view.rs` (lecture et bornes), `moteur/src/mosaic.rs` (les limites sont respectées).

## Conditions de réexamen

- ~~Quand Yocthan aura lu ces noms~~ Tranchés le 2026-10-06 (`ADR-047`).
