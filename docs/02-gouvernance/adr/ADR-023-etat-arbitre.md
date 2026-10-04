# ADR-023 — Les valeurs d'une page : `State`, `{cart}`, et les demandes `add`, `sub`, `set`

- Statut : EXPÉRIMENTATION
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04
- Validation : Yocthan, le 2026-10-04 : « Fais le panier. Comme ça, on va voir ce que ça donne. Et après, on va en juger. » L'écriture est une proposition de Claude ; rien n'est accepté tant que Yocthan n'a pas jugé.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Jusqu'ici une page ne retenait rien : un bouton savait seulement faire entrer dans un point ou en sortir. On ne pouvait écrire ni panier, ni compteur, ni « j'aime ». Codex avait placé « une première action avec état arbitré » en troisième dans son ordre de chantiers, après la boutique et la mesure sur téléphone, toutes deux faites. `ADR-015` fixe la règle : pas de code libre dans un bloc, tout changement d'état passe par un arbitre.

## Décision (à l'essai)

```holo
Page(
  title: "My shop",
  state: State(cart: 0),
  children: [
    Text("{cart} paintings in your cart"),
    Button(name: Add, text: "Add a painting"),
    Button(name: Empty, text: "Empty the cart"),
  ],
  rules: [
    On(Add.tap, effect: cart.add(1)),
    On(Empty.tap, effect: cart.set(0)),
  ],
)
```

1. **`state: State(cart: 0)`** déclare une valeur de la page et son départ. Une valeur est un nombre entier, de 0 à 1 000 000 000. Une page en déclare au plus 32. Son nom s'écrit en minuscules.
2. **`{cart}` dans un texte** affiche la valeur, et le texte se met à jour tout seul. Cela vaut pour tous les textes du fichier, y compris dans les mondes de ses points.
3. **Une demande** s'écrit dans l'effet d'une règle : `cart.add(1)` (ajouter), `cart.sub(1)` (retirer), `cart.set(0)` (fixer). Il n'y a pas d'autre demande.
4. **L'arbitre est dans le moteur** (`moteur/src/etat.rs`). Le bouton émet un signal ; la règle demande ; le moteur fait le changement. Une valeur ne descend pas sous 0 et ne dépasse pas son plafond : elle s'arrête à la borne.
5. **Tout est vérifié avant l'affichage** : un `{nom}` qui ne correspond à aucune valeur, une demande inconnue, une valeur déclarée deux fois ou dans un monde font refuser le fichier, avec la ligne.
6. **La valeur suit le visiteur** tant qu'il ne recharge pas la page : il entre dans un monde, passe dans un autre fichier, revient, le panier est le même. Rechargée, la page repart du départ.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Où déclarer | un bloc `State` sur la page ; une valeur posée sur chaque bloc ; aucune déclaration | `State` sur la page. Toutes les valeurs se lisent au même endroit ; une faute de frappe dans un nom est une erreur, pas une nouvelle valeur. |
| Comment afficher | `{cart}` dans le texte ; un bloc `Value(cart)` ; un texte recomposé par une règle | `{cart}`. C'est l'écriture la plus répandue (Vue, Svelte, les gabarits Python) et elle garde la phrase entière. |
| Comment changer | des demandes fixes (`add`, `sub`, `set`) ; une expression (`cart = cart + 1`) ; une fonction écrite par l'auteur | Des demandes fixes. Une expression ouvre la porte au code libre, que `ADR-015` interdit. |
| Le mot de la soustraction | `sub`, `remove`, `minus` | `sub`, le pendant d'`add` chez les programmeurs. À juger par Yocthan. |

Défauts de JavaScript évités :

- N'importe quel bout de code peut modifier n'importe quelle variable : ici, seules les règles du fichier demandent, et seul le moteur change.
- L'affichage se remet à jour à la main, et l'oublier fait mentir l'écran : ici, il suit la valeur.
- Une faute dans un nom crée `undefined` en silence : ici, le fichier est refusé.
- `NaN`, les nombres négatifs imprévus, `"2" + 1 = "21"` : ici, une valeur est un entier borné.

## Conséquences

### Positives

- Première page qui « agit » : la boutique a un panier, dans le fichier `.holo` et dans sa jumelle en HTML et JavaScript, pour comparer.
- L'état est une simple liste de nombres : il se copie, se compare, et pourra se rejouer (`ADR-008`).

### Négatives et risques

- Des nombres entiers seulement : ni texte, ni oui/non, ni liste. Un vrai panier (quels articles, à quel prix) n'est pas encore possible.
- Pas de condition (« si le panier est vide, cacher le bouton »), pas de calcul entre valeurs (un total).
- Rien n'est gardé après un rechargement, rien n'est partagé entre visiteurs.
- Les accolades autour d'un mot en minuscules désignent désormais toujours une valeur : un texte qui voudrait afficher `{mot}` tel quel ne le peut pas.
- Une demande s'écrit avec une minuscule et une parenthèse (`cart.add(1)`), alors que la règle « ce qui ouvre une parenthèse commence par une majuscule » valait partout. L'exception est limitée à l'effet d'une règle.

## Ce qui reste à faire

- Les autres sortes de valeurs (texte, oui/non), les conditions, les totaux.
- Garder une valeur après un rechargement, et la partager entre visiteurs (le vrai rôle de l'arbitre à plusieurs).
- Des cas dans la suite de conformité, quand l'écriture sera jugée.

## Critères de validation

- `exemples/boutique-comparee/boutique.holo` : ajouter, retirer, vider ; le panier suit dans l'atelier.
- Tests du moteur : `moteur/src/etat.rs` (arbitrage, bornes, état falsifié, affichage, refus).

## Conditions de réexamen

- Quand Yocthan aura essayé le panier et jugé l'écriture : `State`, `state`, `{cart}`, `add`, `sub`, `set`.
- Quand Codex et Gemini auront donné leur avis.
- Au premier besoin d'une valeur qui ne soit pas un nombre.
