# ADR-065 — Les lignes d'une liste : une clé choisie, le clavier gardé, le total avant de couper

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-08, après avoir essayé les leçons : « tout doit être en décidé car je les ai validés »)
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : l'ordre de Yocthan du 2026-10-07 (« terminer les données : … pagination, identifiants stables ») ; l'exploration de l'issue #82 (piste 1 : deux `Repeat(over:)` sur la même liste se mêlent ; le focus se perd dans une ligne refaite).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Une ligne de `Repeat(over:)` était reconnue par tout son contenu (`ADR-057`). Quand une tâche était marquée « faite », sa ligne devenait une autre ligne : la page la refaisait, et le clavier, posé sur le bouton touché, se retrouvait nulle part. Une liste triée qui changeait l'ordre rendait la chose pire.

Trois défauts trouvés en chemin, et corrigés ici :

1. Deux `Repeat(over:)` sur la même liste : après le premier changement, la seconde recevait les lignes de la première (mesuré : « B: Pain » devenait « A: Pain »).
2. Une règle écrite dans la répétition d'une liste calculée (`ADR-062`) était acceptée, mais ne changeait rien : `item.done.set(1)` sur une ligne de `ordered` laissait `tasks` tel quel.
3. Pour « montrer plus », la page ne savait pas combien d'éléments étaient trouvés avant de couper : le bouton restait là quand tout était montré.

## Décision

1. **Une clé choisie** : `Repeat(over: tasks, key: id, …)`. La ligne est reconnue par le champ `id` de son élément, même quand le reste change ou qu'elle change de place. Le champ doit exister. Une liste de textes n'en a pas besoin : chaque texte est sa propre clé.
2. **Le clavier reste** quand une ligne est refaite. Il passe au même bouton de la nouvelle ligne (même nom, même place), ou au premier bouton de la ligne si celui-là n'y est plus (« Fait » devenu « Rouvrir »). La nouvelle ligne est retrouvée par sa clé quand l'auteur en a choisi une, sinon par son rang. Une ligne retirée laisse le clavier à la ligne qui prend sa place. Il n'y a rien à écrire.
3. **Chaque répétition est redessinée avec son propre modèle**. La page sait où elle est écrite dans le fichier.
4. **Une ligne d'une liste calculée change l'élément d'origine** :
   - `item.done.set(1)` change l'élément de la source ;
   - `tasks.remove(item)` le retire de la source.

   L'élément d'origine est l'élément identique de la source, avec autant d'identiques avant lui.
5. **Le total avant de couper** : `Filter(…, limit: shown, total: matching)`.
   - `{matching}` donne le nombre trouvé avant `limit` : « 4 sur 6 ».
   - `If(shown, under: matching, children: [ Button(…) ])` cache « Montrer plus » quand tout est montré.
   - Le total se montre et se compare dans `If` ; une règle `When` ne le guette pas.

## Comparaison faite avant de choisir

| Option | Écriture | Pour | Contre |
|---|---|---|---|
| **A. Une clé choisie, `key: id`** (proposée) | `Repeat(over: tasks, key: id)` | le même mot que `Item(key:)` (`ADR-040`) et que la `key` de React ou Vue ; l'auteur sait quel champ identifie | un réglage de plus |
| B. Des numéros cachés donnés par le moteur | rien à écrire | aucun effort | une liste reçue du serveur n'a pas ces numéros : chaque relecture refait toutes les lignes |
| C. Seulement le rang | rien à écrire | simple | une liste triée met le clavier sur une autre tâche |

Pour le total : `total: matching` plutôt qu'un nom fabriqué (`{found.total}`). C'est un nom choisi, comme `name:`, qui se montre et se compare comme une valeur.

Défauts du web évités :

- En React, une `key` oubliée donne un avertissement, et l'index pris comme clé mélange les lignes. Ici, sans clé, le moteur prend le contenu, et le clavier est gardé quand même.
- `filtered.length` avant `slice` s'écrit à la main. Ici, le total est un réglage.
- Une vue filtrée qu'on modifie, c'est souvent une copie modifiée en vain : ici, la source change.

## Conséquences

- La clé d'une ligne choisie par l'auteur commence par « k: » dans la page. Une clé tirée du contenu ne change pas (`ADR-057`).
- Les mots `key` (dans `Repeat(over:)`) et `total` (dans `Filter`) attendent la validation de Yocthan.

## Critères de validation

- Tests du moteur : `two_repeats_of_one_list_keep_their_own_lines`, `a_chosen_key_stays_when_the_element_changes`, `a_line_of_a_computed_list_changes_its_source`, `the_total_before_the_limit_is_shown_and_compared`.
- Dans Chrome :
  - les deux répétitions gardent « A: » et « B: » après un ajout ;
  - dans la leçon 85, « Fait » au clavier fait descendre la tâche, et le clavier la suit (`k:t2-0`, sur « Rouvrir ») ; « Retirer » la retire, et le clavier passe à la ligne qui prend sa place ;
  - dans la leçon 82, « 4 sur 6 », puis « Montrer plus » disparaît.
- Vérifié dans les deux sens : sans la reprise du clavier, l'essai de la leçon 85 rate.
