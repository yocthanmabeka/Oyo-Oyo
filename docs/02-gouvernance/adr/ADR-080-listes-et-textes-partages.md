# ADR-080 — Lot 6, la suite : des listes partagées (une ligne désignée par sa clé), un texte partagé confirmé, un frein des touchers

- Statut : ACCEPTÉ
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue 182 (les dettes de l'`ADR-079`) ; le travail de Codex, PR 207 (`codex/fin-partage-2026-10-08`, compte rendu `proposals/GPT5.6/fin-partage-2026-10-08/README.md`) ; le travail laissé en cours par Gemini (`gemini/fin-partage-fiches-2026-10-08`, 5211a68 : « identifiants stables, retrait de lignes, fiches partagées ») ; tous deux relus, corrigés et finis par Claude le 2026-10-09 (branche `reprise/codex-fin`) ; la correction de `Module(output: …)` de la PR 208 ; l'`ADR-079` (les valeurs partagées), les `ADR-044`, `ADR-051`, `ADR-057` et `ADR-065` (les listes, leurs fiches, leurs lignes et leurs clés).
- Validation : Yocthan, le 2026-10-09 (« tu valides tout ce qu'on avait fait avec Codex » ; continuer le travail de Codex et de Gemini).
- Projets affectés : HoloCode, HoloEngine, le serveur

## Contexte

L'`ADR-079` partageait des nombres et des textes, changés seulement par un toucher. Elle laissait des dettes : une liste partagée, un champ qui prépare une valeur partagée, et une limite aux touchers d'un visiteur. Codex a construit une première liste partagée (des textes, `push` et `clear`) et refusait de retirer une ligne : entre la page vue et le toucher, d'autres ont pu ajouter ou retirer des lignes, et le rang de la ligne ne dit plus laquelle. Gemini a commencé à retrouver la ligne par sa clé, et à admettre les fiches.

## Décision

1. **Une liste partagée** se déclare dans `Shared`, comme une liste de `State` : des textes (`Shared(names: [])`) ou des fiches (`Shared(groceries: [ Item(what: "Du pain", done: 0) ])`). Cinquante éléments au plus, deux cents caractères par texte ou par champ, 16 Kio pour toutes les valeurs partagées d'une page, codées. Un geste qui dépasserait une limite est refusé tout entier : aucune de ses valeurs ne change.
2. **Elle se change comme une liste à soi** (`ADR-044`, `ADR-057`), seulement par un toucher, arbitré par le serveur avec le même moteur : `groceries.push(Item(what: what, done: 0))`, `groceries.remove(item)` et `item.done.set(1)` dans les règles de `Repeat(over: groceries, …)`, `groceries.clear()`. Un ajout part toujours de la liste du serveur, même si la page envoie une autre copie. Changer un champ d'une ligne (`item.done.set(1)`) part aussi au serveur.
3. **Une ligne touchée se désigne par sa clé, jamais par son rang.** La clé est celle que la page donne déjà à chaque ligne (`data-key`, `ADR-057`, `ADR-065`) : la valeur du champ nommé par `Repeat(…, key: id)`, sinon une empreinte de l'élément, avec son rang parmi les éléments pareils. Le geste la porte : `Remove.tap@2#<clé>` ; avec JavaScript dans l'envoi du geste, sans JavaScript dans la valeur du bouton du formulaire des gestes. Le serveur la cherche dans la liste qu'il garde, et joue le geste au rang qu'elle y a maintenant. Sans clé, ou si la ligne n'y est plus (retirée, ou changée entre-temps : une fiche cochée a une autre empreinte), le geste est refusé, et la page montre la liste du moment. On ne retire donc jamais la mauvaise ligne. Pour une liste calculée d'après une liste partagée (`Filter(name: found, from: names, …)`, `ADR-062`), la clé se cherche dans la liste calculée, telle que ce visiteur la voit, puis le geste change l'élément de la source.
4. **Un texte partagé se prépare dans un champ, puis se confirme** : `Input(value: title)` n'est permis que si un toucher le confirme, `On(Save.tap, effect: title.set(title))`. La frappe reste un brouillon à soi ; les textes et les conditions de la page montrent toujours la valeur publiée. Le toucher confirme : le serveur vérifie d'abord le bouton d'après **ses** valeurs (un brouillon ne peut pas faire apparaître un bouton caché), puis ne remplace que l'argument de cette demande-là par le texte saisi, borné à deux cents caractères. Sans JavaScript, de même, par le formulaire des gestes. Une publication venue d'une autre page ne remplace pas le brouillon d'un champ ; une réponse tardive ne remplace pas ce que le visiteur a écrit depuis son envoi.
5. **Un frein des touchers partagés** : soixante par visiteur et par adresse, et cent quatre-vingts par adresse IP (celle du visiteur réel, `ADR-083`), par minute ; au-delà, `429` et `Retry-After: 60`, et le geste refusé n'écrit rien. Les compteurs sont bornés (cinquante mille) et nettoyés. Effacer son cookie ne contourne donc pas le second frein.
6. **Un module ne change pas une valeur partagée**, qu'il la rende seule (`output: likes`) ou dans une liste (`output: [best, likes]`, `ADR-077`) : refusé, avec la raison (la même correction que la PR 208).

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Désigner la ligne touchée | son rang ; la copie de la liste envoyée par la page (le premier pas de Gemini) ; un numéro de version de la liste, refuser dès qu'elle a changé ; **la clé de la ligne, portée par le geste** | le rang ment dès qu'un autre a changé la liste ; la copie de la page n'existe pas sans JavaScript (le formulaire n'envoie que le bouton) et le serveur ne la lit pas pour un membre (son état vient de la base) ; une version refuserait tout geste dès qu'une autre ligne change ; la clé existe déjà dans la page, marche avec et sans JavaScript, et ne refuse que si la ligne touchée elle-même a changé |
| Les fiches partagées | des textes seulement (Codex) ; **les fiches, comme pour une liste à soi** | aucun mot nouveau ; une liste de courses, des tâches à cocher, un livre d'or avec un nom et un message |
| Un champ et une valeur partagée | interdit (`ADR-079`) ; chaque frappe envoyée au serveur ; **un brouillon à soi, confirmé par un toucher** | chaque frappe changerait la valeur de tous, lettre après lettre ; le toucher garde la règle de l'`ADR-079` : seul un toucher change une valeur partagée |

## Ce qui est refusé, et pourquoi

- `Input(value: title)` sur un texte partagé sans un toucher qui le confirme par `title.set(title)` ; un champ, une case, un choix ou une glissière sur un nombre ou une liste partagés.
- Un geste de ligne sans clé, ou dont la ligne n'existe plus ; un geste de ligne auquel répondent deux répétitions d'une liste partagée.
- Une liste partagée de plus de cinquante éléments, d'un texte ou d'un champ de plus de deux cents caractères, ou des valeurs partagées de plus de 16 Kio ; un élément qui n'est ni un texte ni une fiche `Item(…)`.
- `Module(output: …)` qui nomme une valeur partagée.

## Sécurité, honnêtement

- Une condition sur l'élément d'une ligne (`If(item.done, is: 0, children: [ Button(name: Remove, …) ])`) n'est pas encore vérifiée par le serveur : il vérifie les conditions autour du bouton, pas celles qui regardent l'élément (une limite de l'`ADR-079`). Un geste forgé peut donc toucher le bouton d'une ligne qui ne le montre pas. Une dette.
- Deux éléments identiques (même texte, mêmes champs) sont interchangeables : retirer « l'un des deux » retire le premier.
- Le frein des touchers n'arrête pas une attaque répartie sur des milliers d'adresses ; derrière un proxy, il suppose que le proxy écrit `X-Forwarded-For` (`ADR-083`).
- L'état personnel d'un visiteur sans compte reste celui qu'envoie sa page (`ADR-079`) ; pour un besoin de droit, une page réservée (`access: members`, `ADR-081`).

## Les défauts du web évités

- **Supprimer « la ligne 3 »** (un index envoyé par la page, comme le font tant d'API) : ici la ligne se désigne par ce qu'elle est, et jamais la mauvaise ne part.
- **Une liste en direct qui ne marche plus sans JavaScript** : sans lui, les boutons des lignes portent leur clé, et la page revient à jour.
- **Un champ qui écrit chez tout le monde à chaque frappe** : un brouillon à soi, publié par un toucher.

## Dettes

- Vérifier au serveur les conditions qui regardent l'élément d'une ligne partagée.
- Un nombre à virgule dans une fiche partagée (comme pour une fiche à soi, `ADR-066`).
- Un « une fois par compte » sûr (un J'aime par membre), avec les comptes de l'`ADR-081`.
- L'essai de la leçon 102 sur le téléphone de Yocthan et son ordinateur, en même temps.

## Critères de validation

- Tests du moteur (`cargo test --release --locked`) : dans `shared.rs`, `concurrent_list_appends_start_from_server_and_stay_bounded` (cinquante ajouts depuis une copie forgée, le cinquante et unième refusé tout entier), `a_shared_field_is_a_draft_until_its_explicit_confirmation`, `a_line_of_a_shared_list_is_found_by_its_key` (une page en retard retire la ligne qu'elle touche, au rang qu'elle a au serveur ; sans clé, une clé inventée ou une ligne partie : refusé ; les boutons sans JavaScript portent la clé ; deux éléments pareils), `shared_records_are_added_ticked_and_removed_by_their_key`, `a_line_of_a_list_computed_from_a_shared_list_is_found_in_its_source`, `what_would_change_a_shared_value_without_the_server_is_refused` (dont `Module(output: …)`) ; dans `gestures.rs`, la clé dans le bouton et `is_tap` ; dans le serveur, `a_visitors_brake_is_also_bound_to_the_real_ip`.
- Dans Chrome (`node outils/browser-tests.mjs partage`) : deux profils, deux ajouts au même instant, aucun perdu ; la frappe gardée pendant la réponse ; une page sans JavaScript, en retard, retire le lait qu'elle touche et non les oeufs, puis son geste sur une ligne cochée entre-temps est refusé ; le brouillon gardé pendant une publication étrangère, confirmé avec et sans JavaScript ; soixante touchers, puis `429`.
