# ADR-105 — Réordonner les lignes d'une liste : `Repeat(over:, reorder: true)`

- Statut : ACCEPTÉ (fait et validé : Yocthan, 2026-10-09, « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #234 (« Dernière dette du web : glisser pour réordonner une liste »), dans la file des dernières dettes du web ouverte par Yocthan le 2026-10-09 ; le grand tableau du web, où le glisser-déposer était « en partie » (« Sur un plateau seulement », `docs/01-holocode/TABLEAU-WEB.md`) ; `drag: true` sur un plateau (`ADR-028`) ; les listes qui changent (`ADR-044`, `ADR-051`) et les clés de leurs lignes (`ADR-057`, `ADR-065`) ; les gestes sans JavaScript (`ADR-074`) ; le panier qui suit le compte (`ADR-081`) ; la règle de parité de Yocthan (2026-10-07 : ce qui existe sur l'ordinateur existe sur le téléphone, et l'inverse).
- Validation : faite par Yocthan le 2026-10-09 (« tu le valides déjà, tu le fais déjà »).
- Projets affectés : HoloCode, HoloEngine, le serveur (`holo serve`)

## Contexte

- Ranger une liste à la main : ses tâches par ordre d'importance, ses tableaux préférés, les étapes d'un voyage. HoloCode faisait glisser un bloc sur un plateau (`drag: true`), pas une ligne dans une liste ; une liste ne changeait que par `push`, `remove(item)`, `clear()` et les champs de ses lignes.
- Le web le fait par le glisser-déposer de HTML : `draggable="true"`, puis `dragstart`, `dragover` (qu'il faut annuler par `preventDefault()`, sinon rien ne se dépose), `drop` et `dataTransfer`. Conçu pour la souris, il ne marche pas au clavier, ne dit rien au lecteur d'écran (`aria-grabbed` et `aria-dropeffect` sont dépréciés depuis ARIA 1.1), et reste inégal au doigt d'un navigateur à l'autre. Surtout, il ne range rien : l'auteur écrit le code qui déplace l'élément dans son tableau, puis dans la page. D'où les bibliothèques : SortableJS pour le doigt, dnd-kit qui ajoute le clavier et les annonces ; et toujours du code à écrire.
- WCAG 2.2 demande (critère 2.5.7, niveau AA) que tout ce qui se fait en glissant se fasse aussi d'un simple toucher, sans glisser : une liste qu'on ne range qu'en glissant n'est pas accessible, même avec un clavier.
- En Flutter, `ReorderableListView(onReorder: (oldIndex, newIndex) { … })` : le développeur déplace lui-même l'élément, et corrige le rang quand l'élément descend (`newIndex -= 1`), un piège connu.

## Décision

1. **`Repeat(over: tasks, reorder: true, children: [ … ])`** : le visiteur réordonne les lignes. Rien d'autre à écrire, ni règle ni bouton. `reorder: false`, ou rien, laisse la liste comme avant.
2. **Le moteur pose sur chaque ligne** :
   - au début, une **poignée** `⠿`, qu'on fait glisser à la souris ou au doigt ; cachée au lecteur d'écran (`aria-hidden`), et cachée sans JavaScript ;
   - au milieu, le contenu de la ligne, rangé comme une colonne ;
   - à la fin, deux vrais boutons, **« Monter »** (↑) et **« Descendre »** (↓), nommés avec la ligne (« Monter « La rivière » »), dans la langue de la page (« Move up “The river” » en anglais). On les touche au doigt, à la souris, au clavier (Tab, puis Entrée ou Espace) et au lecteur d'écran ; sans JavaScript, ils partent à `holo serve`. Ils font au moins 32 pixels de côté (WCAG 2.5.8 en demande 24).
3. **C'est l'arbitre qui change la liste.** La page lui envoie le geste `move:tasks@2:0` (l'élément du rang 2 va au rang 0), comme un toucher ; il déplace l'élément, les autres gardent leur ordre, et rien d'autre ne change. Il ne fait rien pour une liste qui ne se réordonne pas, un rang hors de la liste ou un geste mal formé. Le même arbitre sert dans la page, dans `holo serve` sans JavaScript, et pour un membre connecté : le déplacement est renvoyé au serveur comme ses touchers (`ADR-081`), sinon le « Retirer » suivant aurait retiré une autre ligne de la liste que garde son compte.
4. **Pendant qu'on glisse, rien ne change dans les valeurs** : les autres lignes s'écartent autour de celle qu'on tient, qui ne quitte jamais sa place dans la page (le doigt et le clavier la gardent) ; on pose, et c'est un seul geste pour l'arbitre. Échap, ou un geste que le navigateur reprend, la ramène à sa place. Près du haut ou du bas de l'écran, la page défile toute seule : une longue liste se range aussi au doigt, qui ne peut pas faire défiler pendant qu'il tient la poignée.
5. **Chaque déplacement est annoncé** par la zone que lit le lecteur d'écran (`aria-live`) : « « La rivière » : position 1 sur 4. » En tête, « Monter » ne bouge rien, et le dit (« … est déjà en haut. »).
6. **Le clavier reste** sur le bouton touché, dans la ligne déplacée : trois fois Entrée sur « Descendre », trois places plus bas.
7. **Le nom d'une ligne** (celui des boutons et de l'annonce) : son texte ; pour une fiche, son premier champ de texte qui n'est ni sa clé (`key:`) ni un nombre.
8. **Refusés, avec la raison** : sur une liste calculée (son ordre vient d'elle, de `sortBy`, ou de sa source) ; sur une liste partagée (pas encore) ; sur `Repeat(items: …)`, écrit dans le fichier ; une autre valeur que `true` ou `false`.
9. Une page qui réordonne une liste fait venir le moteur tout de suite (`data-live`), comme une page où l'on fait glisser un bloc : un glissement ne se rejoue pas.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le mot | `drag: true`, déjà sur un plateau ; `draggable: true`, le mot de HTML ; `sortable: true` (jQuery UI, SortableJS) ; **`reorder: true`** | `reorder` dit ce qui change, l'ordre, et non un geste : on réordonne aussi au clavier et sans JavaScript, sans rien glisser. C'est le mot de Flutter, que Yocthan connaît (`ReorderableListView`), et de Framer Motion (`Reorder`) ; un mot connu garde son sens (`ADR-016`). `drag` garde le sens qu'il a sur un plateau (une place de 0 à 100) ; `draggable` rappellerait le glisser-déposer de HTML, dont on refuse les défauts ; `sortable` se confondrait avec `sortBy`, qui trie tout seul |
| Où l'écrire | un bloc à part, `Reorderable(children: [ … ])` ; des demandes dans des règles, `On(Up.tap, effect: tasks.moveUp(item))`, avec des boutons écrits par l'auteur ; **un réglage de `Repeat(over:)`** | un bloc de plus autour d'une liste qui existe déjà ; des règles et des boutons à recopier sur chaque page, et l'accessibilité laissée à chacun. Le réglage est la forme la plus courte, et un débutant la lit sans l'apprendre |
| Les gestes | glisser seulement, comme HTML ; glisser, et au clavier « prendre, flèches, poser » (dnd-kit) ; **une poignée qu'on glisse, et deux boutons** | glisser seulement ne passe pas WCAG 2.5.7, ni le clavier, ni le lecteur d'écran. « Prendre, flèches, poser » ne se devine pas, et ne sert pas au doigt d'un lecteur d'écran (TalkBack n'a pas de flèches). Deux boutons se voient, se comprennent, se touchent partout et marchent sans JavaScript : ce sont eux qui donnent la parité ; la poignée est le raccourci de la souris et du doigt |
| Toute la ligne, ou une poignée | toute la ligne, prise par un appui long (Flutter au doigt) ; **une poignée** | toute la ligne gênerait le défilement au doigt, la sélection du texte et les boutons de la ligne ; l'appui long ne se voit pas, et fait attendre. La poignée se trouve, et le reste de la ligne défile comme avant (`touch-action: none` sur elle seule) |
| Quand la liste change | à chaque ligne franchie, comme un bloc d'un plateau ; **au moment de poser** | un seul geste, une seule annonce, un seul renvoi au serveur pour un membre ; pendant le glissement, la page ne fait que montrer où la ligne ira |
| Sans JavaScript | rien ; **« Monter » et « Descendre », reçus par `holo serve`** | ce sont de vrais boutons : le formulaire des gestes (`ADR-074`) les envoie avec la liste et les rangs (`move:tasks@1:0`), et le serveur passe le geste au même arbitre |
| Une liste partagée | arbitrée par le serveur, déplacement après déplacement ; **refusée pour l'instant** | l'ordre d'une liste partagée se jouerait entre plusieurs visiteurs à la fois : il faudrait désigner les lignes par leur clé (`ADR-080`) et décider ce que devient un glissement pendant que d'autres changent la liste. Laissé en dette, avec la raison dans le message |

## Les défauts du web évités

- **Le glisser-déposer qui ne marche qu'à la souris** : ici, la souris, le doigt, le clavier, le lecteur d'écran et la page sans JavaScript font la même chose.
- **Un glissement sans autre moyen** (WCAG 2.5.7) : « Monter » et « Descendre » sont toujours là.
- **Le code qui range le tableau, puis la page**, et le rang à corriger quand on descend : ici, l'arbitre déplace l'élément ; `keep:` garde l'ordre ; le serveur le connaît pour un membre.
- **`dragover` qu'il faut annuler**, `dataTransfer` et ses formats : rien de tout cela à écrire.
- **Rien n'est dit au lecteur d'écran** : ici, la nouvelle place est annoncée.
- **Le focus perdu** quand la ligne déplacée est refaite : ici, le clavier reste sur le bouton touché, et le doigt sur la ligne qu'il tient.
- **Le toucher de la ligne pris pour celui du bloc qui l'entoure** : la poignée et les deux boutons appartiennent au moteur ; les toucher ne touche pas le bloc nommé qui contient la liste.

## Dettes

- Une liste partagée ne se réordonne pas encore (voir plus haut).
- Une ligne ne passe pas d'une liste à une autre (des colonnes de tâches).
- Ni « tout en haut » ni « tout en bas » d'un seul geste au clavier : une place à la fois.
- La poignée et les boutons ont l'allure du moteur (la couleur du texte, une bordure légère) : un style ne les change pas encore.
- Un essai écrit (`holo test`) ne sait pas encore déplacer une ligne.
- Des données reçues (`Data`) remplacent la liste, et son ordre.
- Le nom d'une ligne se devine (son texte, ou le premier champ de texte d'une fiche) : on ne le choisit pas.
- Le renvoi d'un membre passe par la même fonction que les gestes sans JavaScript (`visitor_gesture`), vérifiée par les tests ; il n'a pas été essayé sur deux appareils.

## Critères de validation

- Tests du moteur (`moteur/src/reorder.rs`) : `a_line_moves_through_the_arbiter` (la troisième ligne va en tête, la liste calculée d'après elle suit ; neuf gestes qui ne changent rien : une liste sans `reorder`, des rangs hors de la liste, un nom en majuscule, un rang de sept chiffres… ; « Retirer » retire le bon élément après un déplacement) ; `the_lines_carry_a_grip_and_two_named_buttons` (la poignée cachée au lecteur d'écran, les deux boutons nommés avec la ligne, en français et en anglais, un nom échappé ; une répétition sans `reorder` inchangée ; la page vivante ; les lignes refaites par `list_html` portent les mêmes boutons) ; `without_javascript_the_buttons_send_the_move` (les boutons de la page servie portent `move:paintings@1:0` et `@1:2` ; le serveur passe le geste à l'arbitre ; un geste forgé sur une autre liste ne change rien) ; `what_is_refused` (quatre refus ; `reorder: false` comme rien).
- Dans Chrome : « réordonner une liste : la poignée à la souris et au doigt, Monter et Descendre au clavier, annoncés ; sans JavaScript, holo serve (leçon 128, serve) ». La souris glisse « La porte bleue » en tête (« En tête » suit, `keep` garde l'ordre écrit par l'arbitre, l'annonce est entendue) ; Entrée sur « Descendre » la descend et le clavier la suit ; Espace sur « Monter » en tête ne bouge rien et le dit ; le lecteur d'écran trouve huit boutons nommés et n'entend pas la poignée ; au doigt, sur un écran de téléphone, la dernière ligne monte au rang 2 ; sans JavaScript, avec `holo serve`, la poignée est cachée et « Descendre » range la liste. Sans les ajouts de `page-engine.js`, l'essai rate (rien ne bouge, ni à la souris, ni au clavier, ni au doigt).
- Leçon `128-reordonner-une-liste.holo`.
