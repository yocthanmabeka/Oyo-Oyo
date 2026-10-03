# ADR-021 — La façon dont une page se regarde s'écrit dans le fichier : `Zoom`, `Points`, `Relief`

- Statut : ACCEPTÉ pour le principe ; les noms et l'écriture exacte sont une proposition de Claude
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
    grid: 4,          // …en une grille de 4 × 4…
    depth: 20,        // …et cela au plus vingt fois de suite
    density: 2,       // points par pixel d'écran, dans chaque sens
  ),

  relief: Relief(
    height: 10px,     // la hauteur du relief quand la page est de biais
    tilt: 52deg,      // jusqu'où l'on peut tourner la page
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
| `Points(size:)` | 2px à 32px | 6px |
| `Points(fragment:)` | 8px à 400px, et au moins `size` × `grid` | 40px |
| `Points(grid:)` | 2 à 8 | 4 |
| `Points(depth:)` | 0 à 20 | 20 |
| `Points(density:)` | 1 à 3 | 2 |
| `Relief(height:)` | 0px à 40px | 10px |
| `Relief(tilt:)` | 0deg à 80deg | 52deg |

La borne « `fragment` au moins égal à `size` × `grid` » garantit qu'il n'y a jamais plus de points à dessiner que l'écran ne peut en montrer (`ADR-005`).

**Deux unités de plus dans le langage** : `px` (pixels d'écran) et `deg` (degrés).

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
- Tests du moteur : `moteur/src/vue.rs` (lecture et bornes), `moteur/src/mosaique.rs` (les limites sont respectées).

## Conditions de réexamen

- Quand Yocthan aura lu ces noms : `Zoom`, `Points`, `Relief`, `shrink`, `size`, `fragment`, `grid`, `depth`, `density`, `height`, `tilt`.
