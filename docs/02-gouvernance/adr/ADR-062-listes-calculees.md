# ADR-062 — Les listes calculées : chercher, filtrer, trier, montrer plus

- Statut : ACCEPTÉ pour l'écriture choisie par Yocthan (option A : `computed: [ Filter(name:, from:, contains:, in:, sortBy:, limit:) ]`, `Repeat(over:, empty:)`, `{found}`). Les trois réglages ajoutés par Claude dans la même idée, `field:`, `is:` et `reverse:`, sont **à valider**.
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : l'ordre de Yocthan du 2026-10-07 (« terminer les données : recherche, filtre, tri, pagination… ») ; l'exploration de l'issue #82 (pistes 1 et 10) ; les réponses de Codex et de Gemini sur « tout le web » ; la question posée à Yocthan le 2026-10-07, réponse : « A. Une liste calculée, nommée ».
- Validation : Yocthan, le 2026-10-07, a choisi l'écriture A parmi trois (une liste calculée nommée ; des réglages sur `Repeat` ; un filtre fait par le serveur).
- Projets affectés : HoloCode, HoloEngine

## Contexte

Une liste ne pouvait que s'afficher en entier. Chercher un produit, garder une sorte, trier par prix ou montrer les suivants était impossible sans code.

## Décision

1. **Une liste calculée** se déclare dans `computed: [ … ]` de la page, par `Filter(name: found, from: articles, …)`. Elle se refait d'après sa source et les valeurs qu'elle lit, chaque fois que l'état change. Elle se montre comme une liste : `Repeat(over: found, …)`, et `{found}` donne son nombre d'éléments.
2. **Chercher** : `contains: search` (une valeur de texte, comme le champ de recherche), dans les champs `in: [title, note]` (tous les champs si `in` manque). Sans majuscules ni accents : « ELAN » trouve « Élan ».
3. **Filtrer par un champ** (à valider) : `field: kind, is: chosen` (`is` : un texte, un nombre, ou le nom d'une valeur).
4. **Trier** : `sortBy: price` ; des nombres comme des nombres, des textes sans majuscules ni accents ; à égalité, l'ordre de départ est gardé. `reverse: true` (à valider) : du plus grand au plus petit.
5. **Montrer plus** : `limit: 12`, ou `limit: shown`, le nom d'un nombre de la page, qu'une règle augmente (`shown.add(12)`).
6. **Une valeur vide ne filtre pas** : un champ de recherche vide, ou une sorte non choisie, montre tout.
7. **`Repeat(empty: "Aucun résultat")`** : ce qu'on écrit quand la liste est vide, annoncé par un lecteur d'écran (`role="status"`).
8. Une liste calculée **ne se change pas par une demande** (`found.push` est refusé avec la raison) et **ne se garde pas** (`keep`) : on change, ou on garde, sa source. Elle peut partir d'une autre liste calculée écrite avant elle.

## Comparaison faite avant de choisir

| Option | Écriture | Pour | Contre |
|---|---|---|---|
| **A. Une liste calculée nommée** (choisie) | `computed: [ Filter(name: found, from: articles, contains: search) ]` | le résultat a un nom : son nombre, plusieurs listes d'une même source ; la même idée servira aux valeurs calculées (TVA, totaux) | un mot nouveau, `Filter` |
| B. Des réglages sur `Repeat` | `Repeat(over: articles, contains: search, sortBy: price)` | rien de nouveau à apprendre | le résultat n'a pas de nom : ni « 3 résultats », ni réutilisation |
| C. Le serveur filtre | `Data(from: "/articles", query: [search])` | rien dans la page | attend `holo serve` ; chaque lettre part sur le réseau |

Défauts du web évités : en JavaScript, un filtre s'écrit à la main (`filter`, `sort`, `slice`), et la page se redessine en entier, ce qui fait perdre le focus ; un tri de textes qui ignore les accents demande `localeCompare` ou `Intl.Collator`, souvent oubliés. Ici, le moteur trie et cherche sans accents d'office, et la page ne refait que les lignes qui ont changé.

## Conséquences

- La page fabriquée par le serveur contient déjà la liste calculée, à son état de départ : un robot de recherche la lit.
- Les noms `field`, `is` et `reverse` attendent la validation de Yocthan ; s'il en choisit d'autres, le moteur les refusera avec le bon mot.
- Restent pour le lot 2 : comparer des textes dans `If` et `When`, `Data` qui dit « chargement » et « échec », les nombres décimaux, les dates, une clé choisie, le nombre total avant de couper.

## Critères de validation

- Tests du moteur : chercher, filtrer, trier, couper, la liste vide, les refus (`found.clear()`, `keep: [found]`, une source, un champ ou une valeur inconnus, `limit: 500`, un nom déjà pris).
- Dans Chrome, la leçon 82 : « RI » trouve « La rivière » et « Rizières » ; « huile » garde trois œuvres ; « Montrer plus » en montre six ; « zzz » écrit « Aucune œuvre ne correspond. ».
