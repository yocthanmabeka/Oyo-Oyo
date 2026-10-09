# ADR-097 — Une liste de définitions : `List(children: [ Term("Poids", "2 kg") ])`

- Statut : PROPOSITION (construit et essayé ; à valider par Yocthan)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #213 (« Dette du web : les listes de définitions »), ajoutée à la file à la demande de Yocthan le 2026-10-09 ; le grand tableau du web, où `dl, dt, dd` était « non » (`docs/01-holocode/TABLEAU-WEB.md`).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Une fiche technique (« Poids : 2 kg »), un glossaire, les conditions d'une offre : des termes, chacun avec sa définition. Le web a pour cela `dl`, `dt` et `dd`. HoloCode n'avait que des listes à puces ou numérotées (`List`), et des tableaux (`Table`).
- Un tableau à deux colonnes y ressemble, mais il se lit autrement au lecteur d'écran (des lignes et des colonnes, pas un terme et sa définition), et il ne se plie pas sur un téléphone.

## Décision

1. **`Term("Poids", "2 kg")`** : un terme, puis sa définition, deux textes entre guillemets. Les deux vont toujours ensemble.
2. **Une `List` dont les éléments sont des `Term` devient une liste de définitions** : le moteur écrit `<dl>`, et pour chaque terme `<div><dt>…</dt><dd>…</dd></div>`. Rien d'autre à apprendre : une liste reste une `List`.
3. La définition est un texte comme celui d'un `P` : elle lit les valeurs de la page (`{prix}`) et le texte enrichi (`**cuivre**`). Le terme aussi.
4. Le style : `Term { … }` vise chaque terme et sa définition ; la liste se nomme comme les autres (`List.fiche(…)`). Au départ, le terme est en gras et la définition dessous, sans retrait : lisible sur un téléphone.
5. **Refusé**, avec la raison : un `Term` hors d'une liste ; une liste qui mélange des `Term` et autre chose ; `ordered:` sur une liste de termes ; un `Term` qui n'a pas exactement un terme et une définition ; un `Term` qui bouge seul (`enter:`, `loop:`), ajouté à la relecture.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Les mots | les noms de HTML (`Dl`, `Dt`, `Dd`) ; trois mots plus clairs (`Definitions`, `Term`, `Definition`) ; **un seul mot nouveau, `Term`, dans une `List`** | une liste de définitions est une liste : `List` le dit déjà. Les noms de HTML sont obscurs (`dd` ne dit rien), et trois blocs de plus seraient trois mots à apprendre pour une seule idée |
| Le terme et sa définition | deux blocs côte à côte, comme `dt` et `dd` ; **un seul bloc qui porte les deux** | en HTML, un `dd` peut se perdre sans son `dt`, ou passer devant lui ; ici c'est impossible à écrire |
| Plusieurs définitions pour un terme | permises (`dt`, `dd`, `dd`) ; **une seule, qui peut être une phrase longue** | le cas courant ; une définition peut énumérer (« Bleu nuit, ou cuivre ») |
| Le rendu | un tableau ; une liste à puces avec du gras ; **`dl`, `dt`, `dd`** | le seul que les lecteurs d'écran annoncent comme des termes et leurs définitions |

## Ce qui est refusé, et pourquoi

- `Term` hors d'une `List` : un terme seul n'a pas de liste où être lu.
- Une liste qui mélange des `Term` et d'autres éléments : `dl` ne contient que des termes et leurs définitions.
- `List(ordered: true, …)` avec des `Term` : une liste de définitions ne se numérote pas.
- `Term("Poids")`, ou trois textes : le terme sans sa définition, ou une définition en trop.
- `enter:` ou `loop:` sur un `Term` : un mouvement enveloppe son bloc d'une boîte, et `dl` n'en accepte pas autour de ses termes. C'est la liste qui bouge, chaque terme à son tour : `List(enter: Enter(opacity: 0, each: 0.1s), children: [ … ])`. Trouvé à la relecture de la PR 220 (session du PC) : la vérification laissait passer ces deux réglages, et le moteur les avalait en silence, contre la règle d'`ADR-037`.

## Les défauts du web évités

- **Un `dd` sans son `dt`**, ou dans le mauvais ordre : ici, un terme porte sa définition.
- **Un tableau détourné** pour des paires terme-définition : ici, la vraie structure, lue comme il faut.
- **Le retrait de 40 pixels** d'un `dd` dans le navigateur, trop large sur un téléphone : ici, la définition est sous son terme, sans retrait.

## Dettes

- Une liste de définitions qui change pendant la visite (`Repeat(over: liste)`) n'est pas encore possible : les termes s'écrivent dans la page.
- Une définition faite de plusieurs paragraphes, ou d'un lien seul.

## Critères de validation

- Tests du moteur : `a_list_of_terms_gives_a_description_list` (le `dl`, chaque terme et sa définition, une valeur `{weight}`, du gras, le style de la liste ; une liste ordinaire ne change pas) ; `a_term_goes_with_its_definition_inside_a_list` (les quatre refus) ; `a_term_does_not_move_alone` (`enter:` et `loop:` refusés sur un `Term` ; la liste qui entre, un terme après l'autre).
- Dans Chrome : « une liste de définitions : un terme et sa définition, lus ensemble (leçon 120) » (la fiche dans l'ordre, la valeur `{prix}` montrée « 189,00 € » ; l'arbre d'accessibilité : 6 termes, 6 définitions, aucun élément de liste à puces).
- Leçon `120-une-liste-de-definitions.holo`.
