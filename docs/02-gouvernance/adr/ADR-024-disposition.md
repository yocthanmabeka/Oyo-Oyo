# ADR-024 — La disposition : `Row`, `Column`, `Grid`

- Statut : ACCEPTÉ — noms et écriture tranchés le 2026-10-06 (`ADR-047`) : `Grid` reste ; `Points(grid:)` devient `divisions:`
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 ; revue de Codex du 2026-10-03 ; réponse de Gemini du 2026-10-04
- Validation : Yocthan, le 2026-10-04, après essai : « Oui, j'ai kiffé. Les paniers, les états, tout ça. Valide-le pour l'instant. » La disposition faisait partie de ce qui était à juger, et il a tout validé ensemble. Avant cela : Yocthan, le 2026-10-04 : « Il faut vraiment qu'on voie le Row et les colonnes. » L'écriture est une proposition de Claude ; à juger après essai.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Tout s'affichait l'un sous l'autre. Codex et Gemini placent tous deux la disposition en premier dans l'ordre des chantiers : sans elle, pas de vrai site. `ADR-017` a posé qu'un style ne dit que l'apparence, et que la disposition vient des blocs. Il manquait ces blocs.

## Décision

```holo
Page(
  title: "My shop",
  children: [
    Row(gap: 8px, align: between, children: [
      H1("My shop"),
      Button(name: Menu, text: "Menu"),
    ]),
    Grid(columns: 3, gap: 12px, children: [
      Column(gap: 4px, align: center, children: [ P("Sunrise"), Text("120 euros") ]),
      Column(gap: 4px, align: center, children: [ P("The blue door"), Text("90 euros") ]),
      Column(gap: 4px, align: center, children: [ P("Market day"), Text("150 euros") ]),
    ]),
  ],
)
```

1. **`Row`** range ses éléments côte à côte. S'ils ne tiennent pas dans la largeur, ils passent à la ligne : une page ne déborde jamais sur le côté.
2. **`Column`** les range l'un sous l'autre.
3. **`Grid(columns: 3)`** les range en colonnes de même largeur : trois au plus, moins quand l'écran est étroit, sans rien écrire de plus.
4. **`gap`** est l'écart entre les éléments : de `0px` à `64px`, `16px` sans rien écrire.
5. **`align`** dit où se placent les éléments, dans le sens de la largeur : `start`, `center`, `end` ; et pour `Row` seulement, `between` (écartés d'un bord à l'autre).
6. Ces blocs se rangent les uns dans les autres, et prennent un style nommé comme les autres (`Column.card(...)`).

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Les mots | `Row`, `Column`, `Grid` (Flutter, Compose) ; `HStack`, `VStack` (SwiftUI) ; `display: flex` (CSS) | `Row`, `Column`, `Grid`. Yocthan vient de Flutter ; ce sont des mots de tous les jours. |
| Où dire la disposition | dans les blocs ; dans les styles, comme en CSS | Dans les blocs (`ADR-017`). On lit la forme de la page dans son plan. |
| Ce qui se passe quand ça ne tient pas | déborder (Flutter affiche une erreur rayée, CSS fait défiler de côté) ; passer à la ligne | Passer à la ligne, toujours. C'est le défaut qu'on voit le plus sur les sites faits à la main. |
| Le téléphone | des règles `@media` écrites par l'auteur ; le moteur s'en charge | Le moteur s'en charge : une grille perd des colonnes d'elle-même. |
| Le placement | deux axes et deux mots (`justify-content`, `align-items`) ; un seul mot | Un seul, `align`, qui parle toujours de la largeur. Dans une ligne, les éléments sont centrés en hauteur. |

Défauts de CSS évités : le débordement horizontal par oubli de `flex-wrap` ; les deux axes qui s'échangent selon `flex-direction`, source d'erreurs sans fin entre `justify-content` et `align-items` ; les `@media` à écrire à la main ; une faute de frappe dans un réglage, ignorée en silence.

## Conséquences

### Positives

- Une vraie page : un en-tête sur une ligne, des produits en grille.
- La même description vaut pour l'ordinateur et le téléphone.

### Négatives et risques

- Peu de réglages : pas de largeur par élément, pas d'élément qui prend « tout le reste », pas d'alignement en hauteur au choix. Des mises en page courantes ne sont pas encore possibles.
- `Grid` porte le même mot que le réglage `Points(grid:)`, qui dit autre chose (le morcellement d'un point). Gemini avait prévenu de cette collision. Tranché le 2026-10-06 (`ADR-047`) : `Grid` reste, le réglage devient `Points(divisions:)`.
- `columns` désigne en CSS le texte en colonnes de journal : un programmeur du web peut s'y tromper.

## Ce qui reste à faire

- Un élément qui prend la place restante ; une largeur par élément ; l'alignement en hauteur.
- La disposition dans la vue en profondeur : aujourd'hui elle ne vaut que pour la page.

## Critères de validation

- `exemples/boutique-comparee/boutique.holo` : trois colonnes sur un écran large, deux sur un téléphone ; les boutons du panier côte à côte.
- Tests du moteur : `moteur/src/flat.rs` (`la_disposition_range_cote_a_cote_en_colonne_et_en_grille`).

## Conditions de réexamen

- Quand Yocthan aura écrit une page avec ces blocs et dit ce qui manque.
- ~~Quand les noms seront tranchés~~ Tranchés le 2026-10-06 (`ADR-047`).
