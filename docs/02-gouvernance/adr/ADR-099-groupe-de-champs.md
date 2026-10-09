# ADR-099 — Un groupe de champs et son nom : `Fields(label: "Adresse de livraison", children: [ … ])`

- Statut : PROPOSITION (construit et essayé ; à valider par Yocthan)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #215 (« Dette du web : regrouper les champs d'un formulaire (fieldset, legend) »), ajoutée à la file à la demande de Yocthan le 2026-10-09 ; le grand tableau du web, où `fieldset, legend, datalist` était « non » (`docs/01-holocode/TABLEAU-WEB.md`) ; `ADR-042`, qui les laissait pour plus tard ; les formulaires qui vérifient (`ADR-068`) ; la proposition de Codex d'ajouter `Fieldset` et `Legend` aux formulaires (`proposals/GPT5.6/tout-le-web-avant-3d-2026-10-07/`).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- **Ce qui existait déjà.** Un `Choice` en boutons ronds est un `fieldset` avec sa `legend` depuis le lot 1 (`ADR-038`) : son `label:` est la légende. Vérifié dans Chrome, sur l'arbre que lit le lecteur d'écran : « Jour de livraison » est un groupe qui contient ses boutons. L'issue le croyait sans légende : de ce côté, il n'y avait rien à gagner.
- **Ce qui manquait vraiment : réunir plusieurs champs sous un nom.**
  - Une adresse (la rue, le code postal, la ville) ; des cases à cocher qui répondent à une même question (« Pour te prévenir : par e-mail, par SMS ») ; deux adresses dans un même formulaire, dont les champs s'appellent pareil (« Rue », « Ville »).
  - Une `Checkbox` était toujours seule : un groupe de cases n'avait pas de nom.
  - Un titre (`H2`) posé avant les champs ne suffit pas : quand on passe d'un champ à l'autre (Tab, ou le mode formulaire d'un lecteur d'écran), le titre n'est pas dit. Le nom d'un groupe, lui, est annoncé en y entrant : « Adresse de livraison, groupe ; Rue ».
- **Trouvé en route** : `Choice(required: true)`, décidé par `ADR-068` (rendu en `radiogroup`, vérifié à l'envoi, écrit dans le guide), était refusé par la vérification de `Choice` comme « mal écrit ». Aucune leçon ne l'employait. Réparé ici : ce sont les boutons ronds obligatoires d'un formulaire, que le lecteur d'écran annonce comme un groupe obligatoire (`radiogroup`).

## Décision

1. **`Fields(label: "Adresse de livraison", children: [ … ])`** : des champs qui vont ensemble, et le nom de leur groupe. Le moteur écrit `<fieldset>` et `<legend>` : le lecteur d'écran (TalkBack, Narrateur) annonce le nom en entrant dans le groupe, puis chaque champ.
2. **Au moins deux champs** (`Input`, `Checkbox`, `Choice`, `Slider`), rangés comme on veut, même plus bas (dans un `Row`, un `If`), avec ce qui les accompagne (un `P` d'aide, un bouton).
3. **Le nom s'écrit `label:`**, comme celui de chaque champ et comme la légende d'un `Choice` : un texte entre guillemets, avec le texte enrichi (`**…**`).
4. **Partout.** Dans un `Form`, il ne change ni l'envoi ni les vérifications : ses champs partent et sont vérifiés comme les autres, et les messages s'écrivent sous le champ, dans le groupe, avec ou sans JavaScript. Hors d'un formulaire, il réunit des réglages qui agissent tout de suite.
5. **L'allure de départ** : ni bordure ni retrait ; le nom en gras, au-dessus des champs ; le groupe ne s'élargit jamais au-delà de l'écran. Le style l'encadre s'il veut (`Fields { border: … }`, `Fields.adresse(…)`) : la bordure entoure alors le nom et les champs, et le `padding` vaut pour les deux.
6. **Refusé**, avec la raison : un groupe sans nom, ou avec un nom vide ; un paramètre sans nom ; un groupe sans champ, ou avec un seul ; un `Choice` seul dans un groupe ; `legend:` ; les mots `Fieldset` et `Legend`.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Faut-il un mot | rien de nouveau : un titre `H2` avant les champs ; un `label:` sur un `Column` ; un nom sur le `Form` ; **un bloc qui regroupe** | un titre n'est pas dit quand on passe d'un champ à l'autre ; `Column` range, il ne nomme pas, et sert hors des formulaires ; un formulaire peut avoir deux groupes (la livraison, la facturation). Le `Choice` est déjà un groupe nommé, mais pour des boutons ronds seulement |
| Le mot | `Fieldset`, le nom de HTML (proposé par Codex, avec `Legend`) ; `Group`, ce que dit le lecteur d'écran ; `FieldGroup` ; **`Fields`** | `Fieldset` colle deux mots que l'écriture de Flutter séparerait (`FieldSet`, `ADR-037`), et appelle un second mot, `legend`. `Group` veut dire ailleurs des formes qu'on déplace ensemble (le `<g>` de SVG, le `Group` de Three.js, « Grouper » dans les logiciels de dessin) : un mot connu garde son sens (`ADR-016`), et la 3D pourrait en avoir besoin. `FieldGroup` dit la même chose que `Fields`, en plus long, avec une majuscule au milieu qui coûte un geste sur un téléphone. `Fields`, « les champs » : un mot, au pluriel comme `Scenes`, qui range des `Scene` |
| Le nom du groupe | `legend:`, le mot de HTML ; **`label:`** | `label:` nomme déjà ce que lit le lecteur d'écran, champ par champ, et `Choice(label:)` écrit déjà une `legend`. Un second mot pour la même chose serait à apprendre pour rien |
| L'allure | celle du navigateur (une bordure en creux, le nom à cheval dessus, des marges) ; une carte encadrée ; **le nom en gras au-dessus des champs, sans bordure** | lisible sur un téléphone, sans retrait qui serre les champs, comme les formulaires de GOV.UK ; une bordure s'ajoute par le style |
| Ce qu'il réunit | n'importe quoi, comme en HTML ; **au moins deux champs** | un groupe sans champ est un encadré (`Aside`) ou un titre ; un champ seul a déjà son nom, et le lecteur d'écran dirait deux noms pour une seule question |
| Où il se place | dans un `Form` seulement ; **partout** | des réglages qui agissent tout de suite (le son, l'allure de la page) se regroupent aussi, sans formulaire |

## Ce qui est refusé, et pourquoi

- `Fields` sans `label:`, avec un nom vide, ou un nom qui n'est pas un texte : un groupe sans nom n'est, pour le lecteur d'écran, qu'un « groupe » de plus.
- `Fields("Adresse", …)` : chaque paramètre est nommé, comme pour `Choice`.
- `Fields(legend: …)` : le nom s'écrit `label:` ; le message donne les paramètres possibles.
- `Fieldset(…)`, `Legend(…)` : refusés avec le bon mot, `Fields(label: …)`.
- Un groupe sans champ : ce n'est pas un groupe de champs ; pour un titre, `H2` ; pour un encadré, `Aside`.
- Un groupe d'un seul champ : le champ a déjà son nom, que le groupe répéterait.
- Un `Choice` seul dans un groupe : il est déjà un groupe, et son `label:` est sa légende.

## Les défauts du web évités

- **La bordure en creux du navigateur**, ses marges et son retrait, qui serrent les champs sur un téléphone : ici, rien ; le nom en gras, puis les champs.
- **Un `fieldset` plus large que l'écran.** En HTML, il ne rétrécit pas sous la largeur de son contenu (`min-inline-size: min-content`). Vu dans Chromium, sur un écran de 360 px : un tableau large dans un groupe élargit toute la page, à 527 px, au lieu de défiler dans sa boîte. Ici, le groupe rétrécit (`min-width: 0`), et le tableau défile.
- **La légende à cheval sur la bordure**, qui ne suit pas le `padding` et se style mal : ici, le nom est la première ligne du groupe (posé en flottant sur toute la largeur, comme le fait Bootstrap) ; une bordure ajoutée l'entoure, avec les champs. Un nom long revient à la ligne.
- **Un `fieldset` sans `legend`**, ou employé comme un simple cadre : refusés.
- **Un titre posé avant les champs**, que le lecteur d'écran ne dit pas quand on passe d'un champ à l'autre : ici, le nom du groupe est annoncé en y entrant.
- **Deux champs « Rue »** (la livraison, la facturation) que rien ne distingue au lecteur d'écran : chacun dans son groupe, ils ne se confondent plus.

## Dettes

- Un groupe de cases dont au moins une doit être cochée (« choisis au moins un jour ») : `required:` sur un groupe n'existe pas encore.
- Éteindre d'un coup tous les champs d'un groupe (`fieldset disabled`) : aucun champ de HoloCode ne s'éteint encore.
- Le nom du groupe prend l'allure du groupe : il ne se style pas à part (sa taille, sa couleur), comme la légende d'un `Choice`.

## Critères de validation

- Tests du moteur :
  - `a_group_of_fields_carries_its_name` : le `fieldset`, sa `legend` avec du gras, des champs rangés dans un `Row`, un groupe de cases, un `Choice` qui garde son propre groupe dedans ; le style de base ;
  - `a_group_has_a_name_and_at_least_two_fields` : dix refus, et deux champs rangés plus bas, acceptés ;
  - `the_fields_of_a_group_are_sent_and_checked_with_their_form` : « Ce champ est obligatoire. » pour un champ du groupe, l'envoi, le message sans JavaScript écrit sous le champ ;
  - `a_required_choice_is_a_radio_group_checked_at_send` : `Choice(required: true)` accepté, en `radiogroup`, « Choisis une réponse. », deux refus. Il ratait avant la réparation (« est mal écrit »).
- Dans Chrome : « un groupe de champs : son nom dit par le lecteur d'écran, ses champs vérifiés à l'envoi (leçon 122) ».
  - L'arbre d'accessibilité : « Adresse de livraison » (Rue, Code postal, Ville), « Pour te prévenir » (Par e-mail, Par SMS), et le `radiogroup` « Jour de livraison ».
  - L'allure : sans bordure, `min-width` à 0, le nom 13 px sous le haut du cadre ; rien ne déborde à 360 px.
  - À l'envoi : trois messages dans les groupes, le clavier sur « Rue ».
  - Il rate quand le moteur écrit une boîte et un titre à la place de `fieldset` et `legend` (le lecteur d'écran n'entend plus que le groupe du `Choice`), et quand le style de base perd la légende flottante et `min-width: 0`.
- Leçon `122-un-groupe-de-champs.holo`.
