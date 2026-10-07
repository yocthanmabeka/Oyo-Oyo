# ADR-057 — Un champ dans une ligne : montrer selon lui, le changer ; des clés stables pour les lignes

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : la revue de Codex sur les composants (les clés stables, remises à plus tard dans `ADR-056`) ; `ADR-044`, `ADR-051`
- Validation : Yocthan, le 2026-10-07 : « travaille sur une condition sur un champ dans une ligne des listes. Les clés stables des listes, ok. »
- Projets affectés : HoloCode, HoloEngine

## Décision

1. **`If(item.<champ>, …)` dans les lignes d'un `Repeat(over:)`** : un nombre se compare avec `is`, `not`, `over`, `under` ; un texte avec `is` et `not`. La branche est choisie pour chaque ligne, quand le moteur la fabrique. Le champ doit exister ; sinon l'erreur donne la liste des champs.
2. **`item.<champ>.set(…)`, `.add(n)`, `.sub(n)`** dans les règles de la ligne changent ce champ de l'élément touché. `set` prend un nombre, un texte, ou le nom d'une valeur de la page ; `add` et `sub` un nombre, sur un champ nombre.
3. **Des clés stables** : chaque ligne porte `data-cle`, tirée de son contenu (avec un rang pour deux éléments pareils). Quand la liste change, la page garde les lignes dont la clé et le HTML fabriqué n'ont pas changé, ne les déplace que s'il le faut, et ne remplace que les autres. Un pli ouvert, un champ où l'on écrit et le focus restent.

## Ce qui n'est pas fait

- Une clé choisie par l'auteur (`key: item.id`) : la clé tirée du contenu suffit tant qu'aucun exemple ne demande de garder une ligne dont le contenu change.
- Les conditions sur plusieurs champs à la fois : on imbrique deux `If`.

## Critères de validation

- Tests : condition sur un nombre et sur un texte, champ inconnu refusé, `item.done.set` / `add` / `sub`, clés stables d'un rendu à l'autre.
- La leçon 74 et son essai écrit ; dans Chrome, le pli ouvert d'une ligne reste ouvert (le même nœud) quand une autre ligne change.
- L'audit axe-core : les 74 leçons, 0 défaut.
