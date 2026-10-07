# ADR-056 — `Component` au lieu de `Part` ; des valeurs par défaut ; des signaux que la page branche

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : la réponse de Gemini sur les composants (`docs/05-discussions/reponses/2026-10-06-gemini-composants-et-comparatif.md`) ; la revue de Codex (PR 124) ; `ADR-050`
- Validation : Yocthan, le 2026-10-07 : « j'ai validé tes propositions » ; et sur le nom : « je pense qu'on doit prendre Component, vu qu'on aura besoin de Part pour la 3D ».
- Projets affectés : HoloCode, HoloEngine, outils

## Décision

1. **`Component` remplace `Part`**, et `components:` remplace `parts:`. Le mot `Part` est gardé libre pour la 3D : une pièce d'un objet, et le bloc de base des mondes de Roblox ; on a fait de même pour `depth` (`ADR-047`). L'ancienne écriture est refusée avec le bon mot (« écris « Component » »), que l'éditeur corrige d'un clic.
2. **Des valeurs par défaut** : `params: [title, price: 0, image: "placeholder.svg"]`. Un paramètre qui a une valeur par défaut peut être oublié à l'appel. Une valeur nommée dans une liste n'est permise que là.
3. **Des signaux émis** : `emits: [add]` déclare les signaux du composant ; une règle émet au lieu d'agir, `On(Add.tap, emit: add)` ; la page branche à l'appel, `onAdd: cart.add(1)` ou une liste de demandes. Un signal non branché ne fait rien. L'écriture d'`ADR-050` (`qty: sunrise`, `qty.add(1)`) reste permise.
4. **Les noms internes restent visibles** (`AddSunrise`), pour ne rien casser ; les rendre privés est remis à plus tard.
5. **Le panneau `?valeurs`** (`ADR-054`) montre aussi le dernier geste et ce qu'il a changé.

## Ce qui n'est pas fait

- Les clés stables des éléments d'une liste (Codex) : plus tard, quand une liste devra garder un élément ouvert ou le focus pendant qu'elle change.
- Un emplacement pour du contenu (`slot`) : quand un exemple réel le demandera.

## Critères de validation

- Tests : `Component` lu, `Part` et `parts` refusés avec le bon mot ; valeurs par défaut ; signaux branchés, en liste, non branchés ; refus (signal non déclaré, branchement qui n'est pas une demande, `onGoo` mal écrit).
- Les leçons, le site de référence et `exemples/site/` passent avec `Component`.
- La leçon 73 dans Chrome, avec le panneau `?valeurs` ; son essai écrit passe.
