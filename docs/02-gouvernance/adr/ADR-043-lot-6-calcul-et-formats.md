# ADR-043 — Lot 6 : multiplier, diviser, et écrire un nombre joliment

- Statut : EXPÉRIMENTATION
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : le grand tableau (le calcul à 30 %, `Intl` absent) ; Yocthan, le 2026-10-06 : « Oui, travaille sur ce qui reste »
- Validation : à donner par Yocthan après essai.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Une page ne savait qu'ajouter, retirer, fixer et tirer au hasard. Et un nombre s'affichait brut : « 18 h 5 », « 1234567 », « 123450 » pour 1 234,50 €.

## Décision (à l'essai)

1. **Deux demandes** : `prix.mul(2)` (multiplier) et `part.div(3)` (diviser, en nombres entiers, arrondi vers le bas). La quantité peut être une autre valeur : `total.mul(quantite)`. Diviser par `0` écrit dans le fichier est refusé ; par une valeur qui vaut 0, rien ne change.
2. **Un format après le nom d'une valeur** : `{minute:00}` (des zéros devant, de `00` à `000000`), `{visites:number}` (les milliers séparés), `{total:cents}` (un prix en centimes), `{weekday:name}` et `{month:name}` (le nom du jour et du mois). Dans une répétition : `{item.price:cents}`.
3. **La langue de la page** (`Page(lang:)`) choisit les séparateurs et les noms : français par défaut (1 234,50 ; mardi), anglais (1,234.50 ; Tuesday) ; l'allemand, l'espagnol, l'italien, le portugais et le néerlandais ont leurs séparateurs, et les noms anglais pour l'instant.
4. Le moteur écrit le format, à la fabrication de la page comme à chaque changement : une seule façon de faire.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le calcul | des expressions (`a * b + 1`) ; **deux demandes de plus** | Les demandes : pas de code libre (`ADR-015`), et chaque pas se lit seul. |
| Les nombres à virgule | des décimaux dans l'état ; **des entiers, et le format `cents`** | Des entiers : jamais d'erreur d'arrondi sur un prix (0,1 + 0,2 en JavaScript). |
| Le format | `Intl.NumberFormat`, `toLocaleDateString` et leurs options ; **un mot après deux-points** | Un mot, lisible dans le texte même ; la langue vient de la page, pas d'un code. |
| Défaut du web évité | un prix calculé en virgule flottante ; une date affichée selon le réglage de l'ordinateur, pas la langue de la page | refusés tous deux. |

## Conséquences

- Pas encore de calcul sur les dates (combien de jours jusqu'au…), ni de pourcentage, ni de racine : à demander quand un vrai site en aura besoin.

## Critères de validation

- Leçons 66 et 67 ; tests du moteur `format.rs` et `plat.rs` (`le_lot_6_calculer_et_formats`) ; dans Chrome : la date en mots, les milliers, les centimes, le calcul et le partage.
