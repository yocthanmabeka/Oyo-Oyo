# Proposition : un panier avec des articles, des prix et un total

- Auteur : Claude
- Date : 2026-10-04
- Statut : l'option C a été construite à l'essai le 2026-10-04, après que Yocthan a donné le champ libre (« je suis toutes tes recommandations »). Voir `ADR-023`. Les options A et B restent des propositions.
- Origine : Gemini, le 2026-10-04 : « Dans la vraie vie, un panier n'est pas un nombre : c'est une liste d'articles, avec un nom, un prix et une quantité. » Yocthan a demandé une proposition avant de construire.

## Le problème

Aujourd'hui (`ADR-023`), une page retient des nombres : `State(cart: 0)`. Le panier dit « 2 tableaux », mais ni lesquels, ni combien ils coûtent. Un vrai panier doit savoir :

- quels articles, et combien de chacun ;
- le prix de chacun ;
- le nombre total d'articles et le prix total.

La contrainte reste celle d'`ADR-015` : l'auteur n'écrit aucun calcul. S'il pouvait écrire `total = total + price`, ce serait du code libre.

## Trois façons de l'écrire

### Option A — Un bloc `Cart` tout fait

```holo
state: State(cart: Cart(currency: "euros")),

Text("{cart.count} paintings, {cart.total} euros"),
Button(name: AddSunrise, text: "Add to cart"),

On(AddSunrise.tap, effect: cart.add("Sunrise over the river", price: 120)),
On(Empty.tap, effect: cart.clear()),
```

- Le moteur connaît le panier : il compte et additionne.
- Simple à écrire. Mais c'est un mot du langage pour un seul usage : demain il faudra `Favorites`, `Playlist`, `Votes`, chacun avec ses règles.

### Option B — Une liste, avec ce que le moteur sait en tirer

```holo
state: State(cart: List(of: Item(name: "", price: 0))),

Text("{cart.count} paintings, {cart.sum(price)} euros"),

On(AddSunrise.tap, effect: cart.add(Item(name: "Sunrise over the river", price: 120))),
```

- Plus général : une liste de fiches, et le moteur offre `count` et `sum`.
- Mais `Item(...)`, `of:`, `sum(price)` font beaucoup de notions d'un coup, et `sum(price)` ressemble déjà à une formule.

### Option C — Les articles sont déclarés, le panier ne retient que des quantités

```holo
state: State(
  sunrise: 0,
  blue_door: 0,
),
prices: Prices(sunrise: 120, blue_door: 90),

Text("{count} paintings, {total} euros"),

On(AddSunrise.tap, effect: sunrise.add(1)),
```

- Rien de nouveau dans les demandes : c'est le `State` d'aujourd'hui, plus une table de prix.
- `{count}` et `{total}` sont calculés par le moteur à partir de la table.
- Limite : les articles doivent être écrits d'avance dans le fichier. C'est le cas d'une petite boutique ; ce n'est pas le cas d'un grand catalogue venu d'un serveur.

## Comparaison

| | A : `Cart` tout fait | B : une liste générale | C : quantités et table de prix |
|---|---|---|---|
| Facile pour quelqu'un qui ne programme pas | oui | non | oui |
| Notions nouvelles | 1 bloc, 2 demandes | 3 ou 4 | 1 bloc |
| Sert à autre chose qu'un panier | non | oui | en partie (des votes, un score) |
| Risque de glisser vers du code libre | faible | réel (`sum(price)` appelle `avg`, `max`, puis des formules) | faible |
| Marche avec un catalogue venu d'ailleurs | oui | oui | non |
| Travail dans le moteur | moyen | gros | petit |

## Défauts du web à ne pas répéter

- En JavaScript, le total est recalculé à la main à chaque changement ; l'oublier une fois, et le total affiché est faux. Ici, le moteur le calcule toujours.
- `0.1 + 0.2 = 0.30000000000000004` : les prix en nombres à virgule donnent des centimes faux. Ici, un prix serait un nombre entier de centimes, ou un entier tout court.
- Un prix lu dans la page peut être modifié par le visiteur. Tant qu'il n'y a pas de serveur qui vérifie, un panier `.holo` est un affichage, pas une commande : il faudra le dire clairement.

## Ma recommandation

Commencer par **C**, parce que c'est le plus petit pas : elle garde tout ce qui existe et n'ajoute qu'une table de prix et deux valeurs calculées. Elle suffit pour la boutique d'exemple et pour ce que Yocthan peut essayer lui-même.

Garder **A** pour le jour où il y aura un catalogue venu d'un serveur. Écarter **B** tant qu'on n'a pas décidé jusqu'où va le « calcul sans code ».

## Ce que Yocthan a à décider

1. A, B ou C ?
2. Les mots : `Prices`, `{count}`, `{total}` (option C), ou d'autres ?
3. Un prix : un nombre entier (120), ou avec des centimes (119,90) ?
