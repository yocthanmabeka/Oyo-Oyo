# ADR-066 — Des nombres à virgule, exacts

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-08, après avoir essayé les leçons : « tout doit être en décidé car je les ai validés »)
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : l'ordre de Yocthan du 2026-10-07 (« terminer les données : … nombres décimaux ») ; l'exploration de l'issue #82 ; `ADR-043` (multiplier et diviser des nombres entiers ; les prix en centimes, `{price:cents}`).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Une valeur de la page n'était qu'un nombre entier, de 0 à un milliard. Un prix s'écrivait en centimes (`price: 1250` et `{price:cents}`) ; un taux, une note sur 20 avec une demie, une taille en mètres étaient impossibles. `State(cart: 1.5)` était refusé.

## Décision

1. **Une valeur à virgule se déclare avec ses chiffres** : `State(price: 12.50)` en garde deux, `State(rate: 0.5)` un. De 1 à 6 chiffres après la virgule.
2. **Elle est exacte.** Le moteur la garde en nombre entier « à l'échelle » (12,50 est gardé 1250) : 12,50 × 3 font 37,50, jamais 37,4999. C'est ce que font les logiciels de comptabilité, pas les nombres flottants de JavaScript (`0.1 + 0.2` y vaut `0.30000000000000004`).
3. **Les demandes** :
   - `add`, `sub` et `set` prennent un nombre qui n'a pas plus de chiffres après la virgule que la valeur : `sum.add(0.25)`. Plus de chiffres en perdraient : c'est refusé, avec la raison. Une autre valeur aussi : `sum.set(price)`, si elle n'a pas plus de chiffres ; un nombre entier va dans une valeur à virgule.
   - `mul` et `div` prennent un facteur, entier ou à virgule : `sum.mul(qty)`, `sum.mul(1.1)` (10 % de plus), `sum.div(3)`. Le résultat est arrondi au plus proche, la moitié vers le haut, à l'échelle de la valeur.
   - Entre nombres entiers, rien ne change : la division arrondit vers le bas (`ADR-043`).
   - `random` tire un nombre entier : il est refusé sur une valeur à virgule.
4. **Les comparaisons sont exactes**, même entre un entier et un nombre à virgule : `If(sum, over: 49.99)`, `If(sum, over: 30)`, `If(sum, is: price)`.
5. **L'affichage suit la langue de la page.**
   - `{price}` montre « 12,50 » en français, « 12.50 » en anglais.
   - `{price:number}` groupe par milliers : « 1 234,50 ».
   - Les autres formats (`00`, `cents`, `name`) sont refusés sur une valeur à virgule.
6. **Un champ** qui présente une valeur à virgule a le clavier décimal et un pas de 0,01. « 12,5 » et « 12.5 » sont compris ; un chiffre de trop est arrondi au plus proche.
7. **Des données reçues** : `{"price": 12.5}` va dans une valeur à virgule, arrondi à son échelle. Un nombre à virgule ne va pas dans un nombre entier, comme avant.
8. **Pas encore** : une valeur à virgule ne règle pas une glissière, une barre de progression, une case ni une place sur un plateau. Elle ne compte pas pour les prix (`Prices`) et ne sert pas de `limit:`. Une fiche de liste (`Item`) ne prend pas de nombre à virgule ; un prix de fiche s'écrit en centimes, avec `{item.price:cents}`. Chaque refus le dit. Les nombres négatifs ne sont pas encore là.

## Comparaison faite avant de choisir

| Option | Écriture | Pour | Contre |
|---|---|---|---|
| **A. Des chiffres fixés à la déclaration** (proposée) | `price: 12.50` | exact ; le nombre de chiffres se voit ; pas de surprise d'arrondi | une valeur ne change pas de précision |
| B. Des nombres flottants, comme JavaScript | `price: 12.5` | rien à décider | `0.1 + 0.2 ≠ 0.3` ; des prix faux d'un centime |
| C. Garder les centimes | `price: 1250`, `{price:cents}` | rien à faire | un taux, une note, une taille restent impossibles |

Défauts du web évités :

- `0.1 + 0.2` ;
- `toFixed` qui arrondit mal (`1.005.toFixed(2)` donne « 1.00 ») ;
- le séparateur décimal oublié pour le français ;
- un champ `type=number` qui refuse « 12,5 » selon la langue du navigateur.

## Conséquences

- Le lecteur garde le nombre de chiffres écrits après la virgule (`12.50` en a deux).
- L'état voyage toujours en nombres entiers (`price=1250`) : la page, les valeurs gardées et les conditions n'ont rien changé.
- La page écrit le format d'une valeur à virgule (`d2`, ou `nd2` groupé) : c'est le moteur qui l'ajoute, pas l'auteur.

## Critères de validation

- Test du moteur : `decimals_are_kept_exact`. Il vérifie :
  - le départ, l'affichage « 12,50 », le champ ;
  - × 3 = 37,50, × 1,1 = 41,25, ÷ 3 = 13,75 ;
  - les comparaisons ;
  - la saisie « 12,345 », qui donne 12,35 ;
  - les données reçues ;
  - les formats en français et en anglais ;
  - sept refus.
- Dans Chrome, la leçon 86 : « 12,50 » ; × 4 = « 50,00 » et la livraison offerte ; + 10 % = « 55,00 » ; le prix changé en « 9,99 ».
