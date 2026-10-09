# ADR-063 — Comparer des textes

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-08, après avoir essayé les leçons : « tout doit être en décidé car je les ai validés »)
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : l'ordre de Yocthan du 2026-10-07 (« terminer les données : … texte … ») ; l'exploration de l'issue #82 (un texte ne se comparait qu'au vide) ; les réponses de Codex et de Gemini sur « tout le web ».
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Une valeur de texte (`State(size: "M")`) ne se comparait qu'au vide : `If(size, is: "")`. On ne pouvait pas montrer un message pour la taille « L », vérifier une réponse, ni comparer deux e-mails. Une règle qui guette (`When`) ne regardait que des nombres.

Deux défauts trouvés en chemin, et corrigés ici :

- une règle rangée sous une condition sur un texte, `If(mode, is: "play", rules: [ Every(…) ])`, ne valait jamais : l'arbitre ne voyait pas les textes ;
- `If(found, is: 0)` sur une liste calculée (`ADR-062`) répondait d'après une liste vide après le premier changement, parce qu'une liste calculée n'est pas relue de l'état.

## Décision

1. **Un texte se compare à un texte écrit entre guillemets** : `If(size, is: "L", children: [ … ])` (égal), `If(size, not: "M", children: [ … ])` (différent). `is: ""` et `not: ""` gardent leur sens : vide, rempli.
2. **Ou à une autre valeur de texte** : `If(again, is: email, children: [ … ])`.
3. **Une règle qui guette peut regarder un texte** : `When(answer, is: "Paris", effect: score.add(1))`. Elle agit au moment où le texte devient « Paris », que le visiteur l'écrive ou qu'une règle le change, et pas tant qu'il le reste. Ses effets restent des nombres ou des sons (`ADR-044` : un texte ou une liste ne change que par un geste).
4. **À la lettre près** : majuscules, accents et espaces comptent. « paris » n'est pas « Paris ». C'est ce que fait `===` en JavaScript. Une comparaison plus souple pourra venir plus tard, sous un autre mot.
5. **Pas de mélange** : un texte avec un texte, un nombre avec un nombre. `over` et `under` ne servent qu'aux nombres ; pour ranger des textes, on trie une liste (`Filter(sortBy:)`, `ADR-062`). Chaque refus dit pourquoi, avec l'écriture qui marche.
6. **Les règles sous condition** (`If(…, rules: [ … ])`) valent aussi quand la condition regarde un texte.

## Comparaison faite avant de choisir

| Option | Écriture | Pour | Contre |
|---|---|---|---|
| **A. Les mots de `If`, avec un texte** (proposée) | `If(size, is: "L")`, `When(answer, is: "Paris", …)` | rien de nouveau à apprendre ; les éléments d'une ligne le font déjà (`If(item.kind, is: "huile")`, `ADR-057`) | — |
| B. Un mot à part pour les textes | `If(size, equals: "L")` | dit que c'est un texte | deux mots pour la même idée |
| C. Insensible aux majuscules d'office | `If(answer, is: "paris")` vrai pour « Paris » | plus doux pour un quiz | faux pour un mot de passe, un code, un identifiant ; le web compare à la lettre |

Défauts du web évités : en JavaScript, `"1" == 1` est vrai et `"10" < "9"` aussi (comparaison de textes lettre par lettre) ; ici, comparer un texte à un nombre est refusé avec la raison, et plus grand ou plus petit est réservé aux nombres.

## Conséquences

- Le nom d'une condition, dans la page, porte le texte entre guillemets : `size|is="L"`. Les signes qui séparent ces noms (`;`, `|`, `=`, `"`, `%`) et les caractères de contrôle y sont écrits `%XX`.
- La page fabriquée par le serveur cache déjà ce qui est faux au départ.
- Reste à faire plus tard, si le besoin vient : une comparaison sans majuscules ni accents, `contains:` dans une condition (les formulaires du lot 3 vérifieront les e-mails par leur propre mot).

## Critères de validation

- Tests du moteur : `a_text_is_compared_to_a_text` (égal, différent, deux valeurs, les signes dans le nom, la règle qui guette au clavier et par un geste, les six refus) ; `rules_under_a_text_condition` (horloge et attente sous une condition sur un texte) ; `the_count_of_a_computed_list_is_compared_after_a_change`.
- Dans Chrome, la leçon 83 : « L » montre son message ; « paris » ne compte pas, « Paris » donne « Bravo ! » et 1 ; deux e-mails différents, puis pareils.

## Suite

- 2026-10-07 : `over` et `under` servent aussi à deux dates, `If(arrival, under: today)` (`ADR-067`, proposition). Pour les autres textes, rien ne change.
