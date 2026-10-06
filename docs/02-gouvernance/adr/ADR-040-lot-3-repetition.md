# ADR-040 — Lot 3 : écrire une fois, répéter pour chaque élément (`Repeat`)

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : le grand tableau (`docs/01-holocode/TABLEAU-WEB.md`, « Sans liste, douze produits s'écrivent un par un ») ; `ADR-025` (les listes répétées, reportées) ; Yocthan, le 2026-10-06 : « tu travailles sur le lot 2 jusqu'au lot 5… je suis tes recommandations »
- Validation : validé par Yocthan le 2026-10-06, après avoir tout essayé : « En fait, j'ai tout testé de tout ce qui était à laisser [à l'essai] et je trouve que c'est bon. Donc, euh, valide-le. »
- Projets affectés : HoloCode, HoloEngine

## Contexte

Une boutique de dix produits s'écrivait dix fois : la même carte, le même bouton, la même règle. Le tableau classe en priorité « des listes de valeurs » et « répéter pour chaque élément » (`for`, `map`).

## Décision

```holo
Repeat(
  items: [
    Item(key: sunrise, title: "Sunrise", image: "sunrise.png"),
    Item(key: river, title: "The river", image: "river.png"),
  ],
  children: [ Column(children: [ Image(source: item.image, alt: ""), H2("{item.title}"), Text("In the cart: {item}"), Button(name: Add, text: "Add") ]) ],
  rules: [ On(Add.tap, effect: item.add(1)) ],
)
```

1. **`Repeat(items:, children:, rules:)`** pose le modèle (`children`) une fois pour chaque `Item`. Elle se place partout où l'on range des blocs : dans une page, une `Grid`, une `List`, un `Row`.
2. **`Item(…)`** donne les champs d'un élément : des textes, des nombres ou des mots. **`key:`** est la clé de l'élément : le nom d'une valeur de la page.
3. Dans le modèle, **`item`** désigne l'élément : `{item.title}` dans un texte, `item.image` à la place d'une valeur, `{item}` pour montrer la valeur de la clé, `item.add(1)` pour la changer.
4. **Un bloc nommé reçoit le nom de son élément** : `Add` devient `AddSunrise`, partout dans le modèle et ses règles. Une règle de la page peut donc viser `AddSunrise`.
5. **Les règles de la répétition** (`rules:`) sont écrites une fois par élément et rejoignent celles de la page (ou du monde).
6. **Garde-fous** : de 1 à 200 éléments ; 20 000 blocs dépliés au plus par page ; pas de répétition dans une répétition ; un champ ne porte pas le nom d'un mot du langage (`add`, `tap`…) ; un champ absent est refusé avec son nom.

La répétition est **dépliée à la lecture**, comme `Use` (`ADR-029`) : le reste du moteur, la page fabriquée d'avance et les vérifications ne voient que les blocs qu'on aurait écrits à la main. Aucun code ne tourne dans la page pour répéter.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Où vit la liste | une valeur de la page qui serait une liste (`State(products: […])`) ; **dans la répétition elle-même** | Dans la répétition : rien à changer à l'état, et la page fabriquée d'avance montre déjà tous les produits (un robot de recherche les lit). Une liste qui change pendant la visite (venue du serveur, ou que le visiteur remplit) viendra ensuite. |
| Comment désigner l'élément | `{title}` seul ; **`{item.title}`** | `item.` : un champ ne peut pas se confondre avec une valeur de la page (`{cart}`). |
| Le bouton de chaque carte | une règle écrite à la main par produit ; un effet posé sur le bouton ; **des règles dans la répétition, des noms dérivés** | Les règles restent à part (`ADR-015`), mais s'écrivent une fois ; `AddSunrise` reste un vrai nom, qu'on peut viser ailleurs. |
| Le mot | `For`, `Each`, `Map` ; **`Repeat`** | `Repeat` dit ce qu'on voit, sans vocabulaire de programmeur. |
| Défaut du web évité | en JavaScript, `map` sans `key` fait recréer ou mélanger les éléments ; une liste fabriquée par le script reste invisible aux moteurs de recherche | Ici, la clé est un nom ; la page dépliée est du HTML ordinaire, fabriqué d'avance. |

## Conséquences

- Le prix d'un produit s'écrit encore deux fois quand la page a un panier : dans `Prices` et dans `Item`. À réunir.
- Un morceau importé avec des paramètres (`Use(Card, title: …)`) n'existe pas encore ; `Repeat` à un seul élément en tient lieu.
- Les listes qui changent pendant la visite (un panier qui affiche ses lignes, des messages reçus) demandent des valeurs qui soient des listes : plus tard.

## Critères de validation

- Leçon 49 ; tests du moteur `repetition.rs` ; dans Chrome, trois cartes écrites une fois, chacune avec son bouton, et le total du panier qui suit.
