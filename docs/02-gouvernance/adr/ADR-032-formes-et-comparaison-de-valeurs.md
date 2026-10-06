# ADR-032 — Les formes (`Shape`), et comparer ou fixer d'après une autre valeur

- Statut : ACCEPTÉ
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 ; `docs/01-holocode/COMPARATIF-CONCURRENTS.md` (planning, étapes 6 et 7)
- Validation : Yocthan, le 2026-10-04, après avoir écouté les sons : « J'ai kiffé. Fais ce qui reste à faire. » L'écriture est une proposition de Claude ; à juger après essai. Validé par Yocthan le 2026-10-06 : « Qu'est-ce que tu attends pour valider tous ceux qui sont à l'essai ? »
- Projets affectés : HoloCode, HoloEngine

## Contexte

Deux manques revenaient dans les limites notées depuis le premier jeu : tout objet d'un jeu était un point lumineux, et l'on ne pouvait pas garder un meilleur score, faute de comparer deux valeurs. Le premier appartient à l'étape 6 (dans les mondes : des formes), le second à l'étape 7 (garder la partie).

## Décision

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
  ],
  rules: [
    On(Target.tap, effect: score.add(1)),
    When(score, over: best, effect: best.set(score)),
  ],
)
```

1. **`Shape(form:, color:, size:)`** : une forme simple, d'une seule couleur. `form` : `circle`, `square`, `triangle`, `diamond`. `size` : de `8px` à `400px`.
2. Une forme qui a un nom se touche (`tap`), comme un bouton ; elle se place sur un plateau (`x`, `y`, `drag`) et peut être guettée par une rencontre.
3. **Une comparaison peut porter sur une autre valeur** : `If(score, over: best, …)`, `When(score, over: best, …)`. La valeur est lue au moment où l'on compare.
4. **Une demande aussi** : `best.set(score)`, `total.add(bonus)`.
5. Ensemble, avec `keep` : un meilleur score, gardé d'une visite à l'autre. C'est « garder la partie », sans mot nouveau.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Des objets qui ne soient pas des points | un bloc `Shape` avec quelques formes ; du SVG écrit par l'auteur ; seulement des images | `Shape`. Quatre formes couvrent un jeu simple sans fichier à fournir. Le SVG est un langage entier, où l'on peut glisser du code. Les images restent possibles : `Image` se place déjà sur un plateau. |
| Le mot du réglage | `form` ; `kind` ; `shape` | `form`. `Shape(shape: …)` bégaie ; `kind` ne dit rien. |
| Comparer deux valeurs | accepter un nom là où l'on attend un nombre ; des expressions (`score > best`) | Un nom à la place d'un nombre. Aucune écriture nouvelle : `over: 10` et `over: best` se lisent pareil. Pas d'expression, donc pas de porte ouverte au code libre. |
| Le meilleur score | une règle écrite avec ce qui existe ; un mot dédié (`best:`) | Une règle : `When(score, over: best, effect: best.set(score))`. Un mot dédié ne servirait qu'à cela. |

## Conséquences

### Positives

- Le jeu de la pomme a un panier carré et une pomme ronde ; le jeu de l'étoile garde le meilleur score.
- Deux limites notées depuis `ADR-026` sont levées sans ajouter de sorte de règle.

### Négatives et risques

- Quatre formes seulement, d'une seule couleur, sans bordure ni dégradé.
- Une forme reste plate. Les formes en volume et les modèles 3D ne sont pas faits : voir `proposals/Claude/modeles-3d-et-jeu-a-plusieurs-2026-10/`.
- Comparer à une valeur ne donne toujours pas de calcul : pas de « score fois deux », pas de « ou ».
- `keep` garde sur cet appareil : le meilleur score n'est pas partagé entre joueurs. Le jeu à plusieurs n'est pas fait.

## Ce qui reste à faire

- Les modèles 3D et le jeu à plusieurs : deux propositions, à discuter avant de construire.

## Critères de validation

- `exemples/lecons/29-comparer-deux-valeurs.holo` et `30-formes.holo`.
- Tests du moteur : `etat.rs` (`on_compare_et_on_fixe_d_apres_une_autre_valeur`), `plat.rs` (`une_forme_est_un_dessin_ou_un_bouton`).

## Conditions de réexamen

- Quand Yocthan aura essayé et jugé : `Shape`, `form`, et l'emploi d'un nom de valeur à la place d'un nombre.
