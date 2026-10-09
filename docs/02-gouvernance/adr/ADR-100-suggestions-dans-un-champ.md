# ADR-100 — Des suggestions dans un champ : `Input(suggestions:)`

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-09, après avoir tout essayé : « j'ai tout essayé et c'est bon »)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #216 (« Dette du web : des suggestions dans un champ (datalist) »), ajoutée à la file à la demande de Yocthan le 2026-10-09 ; le grand tableau du web, où `fieldset, legend, datalist` était « non » (`docs/01-holocode/TABLEAU-WEB.md`) ; les champs (`ADR-027`, `ADR-068`) ; les listes de la page (`ADR-044`, `ADR-062`).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Une ville, un pays, un fruit : on aide le visiteur en lui proposant des textes pendant qu'il écrit, sans l'obliger à en prendre un. HTML a pour cela `<datalist>` et l'attribut `list` d'un `<input>`. HoloCode n'avait que `Choice`, qui oblige à prendre l'une de ses options (`ADR-038`), et `Input`, qui ne propose rien.
- Les suggestions viennent souvent de la page elle-même : les villes déjà retenues, une liste reçue du serveur.
- En HTML, le champ et ses suggestions sont deux éléments reliés par un `id` recopié à la main ; quand les suggestions changent, il faut les refaire en JavaScript.

## Décision

1. **`Input(value: city, label: "Ville", suggestions: ["Paris", "Lyon", "Marseille"])`** : un paramètre du champ. Le navigateur montre les suggestions pendant qu'on écrit ; on en touche une, ou on écrit autre chose.
2. **`suggestions: villes`** : les suggestions viennent d'une liste de textes de la page (`State(villes: [ … ])`, une liste calculée, une liste remplie par `Data`). Quand elle change pendant la visite (`villes.push(ville)`), le moteur refait les suggestions : la page les lui demande (`suggestions_html`) et les pose, comme pour un graphique (`ADR-087`). Sans JavaScript, `holo serve` fabrique la page avec la liste du visiteur.
3. **Le moteur écrit le `datalist`**, une seule fois pour la page, après son contenu, et y relie chaque champ par `list` : `holo-list-villes` pour une liste de la page, `holo-suggestions-…` (tiré du texte des suggestions) pour des suggestions écrites. Plusieurs champs partagent les mêmes ; un champ dans les lignes d'une liste (`Repeat(over:)`) aussi, sans refaire le `datalist` à chaque ligne.
4. **Une suggestion est un seul texte** : `<option value="Paris">`, sans `label`. Un texte répété n'est proposé qu'une fois ; un texte vide, jamais. Un élément à champs, ajouté à une liste déclarée vide, propose son premier champ, comme `{item}`.
5. **Le champ reste un champ de texte d'une ligne** : `max:`, `min:` et `required:` s'y appliquent, et le moteur garde ce qui est écrit, suggestion ou non. Une suggestion écrite tient dans le champ (80 caractères, ou `max:`).
6. Bornes : de 1 à 200 suggestions écrites, comme les éléments d'une liste (`ADR-044`).

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Rien de nouveau ? | `Choice(menu: true)`, la liste déroulante (`select`) | non : elle oblige à prendre l'une des options. Une ville absente de la liste ne peut pas s'écrire, et une liste de 200 villes se fait défiler au lieu de se taper |
| Où l'écrire | un bloc à part, `Datalist(name: Cities, …)`, relié au champ par son nom, comme en HTML ; **un paramètre du champ** | un bloc à part demande un nom de plus, à recopier sans faute, pour un bloc qui ne se voit pas. Pour donner les mêmes suggestions à deux champs, une liste de la page suffit déjà : `suggestions: villes` dans les deux |
| Le mot | `list:`, le nom de l'attribut de HTML ; `datalist:`, celui de l'élément ; `options:`, comme `Choice` ; `autocomplete:` ; **`suggestions:`** | `list:` se confondrait avec `List` et avec les listes de la page (`suggestions: villes` en est justement une) ; `datalist` ne dit pas ce qu'on voit ; `options:` ferait croire qu'il faut choisir, comme dans `Choice` ; en HTML, `autocomplete` veut dire autre chose (ce que le navigateur retient du visiteur, `autocomplete="email"`, que le moteur écrit déjà), et un mot connu garde son sens (`ADR-016`). « Suggestions » dit ce qu'on voit, et qu'on n'est pas obligé de les prendre |
| D'où viennent les suggestions | écrites seulement ; **écrites, ou une liste de la page** ; demandées au serveur à chaque lettre | une liste de la page suit ce que fait le visiteur (les villes retenues) et ce qu'envoie le serveur (`Data`), sans rien de nouveau à apprendre. Demander au serveur à chaque lettre enverrait tout ce qu'on écrit : laissé en dette |
| Le rendu | une liste faite en JavaScript (le motif « combobox » d'ARIA, celui des bibliothèques d'autocomplétion) ; **le `datalist` du navigateur** | le `datalist` marche sans JavaScript (`holo serve`), sur un téléphone comme sur un ordinateur ; le navigateur le dit au lecteur d'écran (Chrome : une « combobox » qu'on écrit, `autocomplete="list"`, vu dans son arbre d'accessibilité) ; un navigateur qui ne le connaît pas garde un champ ordinaire. Une liste faite à la main demande du JavaScript, et le motif d'ARIA est difficile à rendre juste au lecteur d'écran. Le prix : la liste qui s'ouvre a l'allure du navigateur |
| Quand la liste change | des suggestions figées au départ ; la page relit la liste elle-même ; **le moteur refait les options** | comme un graphique (`ADR-087`) et les formes d'un dessin (`ADR-088`) : la page ne décide rien, elle pose ce que le moteur a fabriqué |

## Ce qui est refusé, et pourquoi

- Des suggestions sur un nombre : elles proposent des textes ; la valeur du champ se déclare comme un texte (`State(city: "")`).
- Sur un champ `type:` (date, heure, couleur, fichier, e-mail) ou `lines:` : un texte d'une ligne seulement. HTML ne propose rien à un texte long, et les autres sortes de champs ont déjà leur aide (le choisisseur de date, le clavier des adresses).
- `suggestions: []`, ou plus de 200 : rien à proposer, ou trop.
- Une suggestion vide, sur plusieurs lignes, plus longue que le champ, écrite deux fois, ou qui n'est pas un texte entre guillemets.
- Une liste à champs (`Item(name: …, zip: …)`) : on ne sait pas quel champ proposer.
- Un nom qui n'est pas une liste de la page (une valeur de texte, une faute de frappe) : la raison est dite, avec l'écriture attendue.
- `suggestions:` sur un autre bloc qu'`Input` (`Checkbox`, `Choice`) : refusé comme tout paramètre inconnu.

## Les défauts du web évités

- **L'`id` recopié** : en HTML, `list="villes"` doit répéter l'`id` du `datalist`, et une faute de frappe fait disparaître les suggestions sans un mot. Ici, le moteur fait le lien.
- **Un `id` en double** quand le même champ revient (dans les lignes d'une liste, dans plusieurs champs) : ici, un `datalist` par source de suggestions, écrit une seule fois.
- **Une option dont le `label` diffère de sa `value`** : selon le navigateur, on voit l'un, l'autre ou les deux. Ici, ce qu'on voit est ce qui s'écrit.
- **Des suggestions qui ne suivent pas les données** sans JavaScript écrit à la main : ici, elles suivent la liste.
- **Le mot `autocomplete`**, qui veut dire en HTML « ce que le navigateur retient du visiteur » et non « des suggestions » : ici, un mot qui dit ce qu'il fait.

## Dettes

- Des suggestions demandées au serveur pendant qu'on écrit (chercher dans une très grande liste) : pas encore.
- Une liste à champs : choisir le champ proposé (le nom d'une boutique, pas son code postal).
- La liste qui s'ouvre a l'allure du navigateur (police, couleurs, taille) : un style ne la change pas.
- Une suggestion venue d'une liste peut être plus longue que le champ : choisie, le moteur n'en garde que la longueur du champ, comme pour tout ce qu'on écrit.
- Pas de suggestions pour un nombre, une date, un e-mail ni un texte long.

## Critères de validation

- Tests du moteur : `written_suggestions_give_one_datalist` (le champ relié par `list` ; un seul `datalist` pour deux champs, après le contenu ; un texte ne devient jamais une balise ; un champ sans suggestions ne change pas) ; `suggestions_from_a_list_follow_it_with_or_without_javascript` (une ville répétée proposée une fois ; « Grenoble », qui n'était pas proposée, s'écrit ; retenue, elle est proposée par `suggestions_html` et par la page servie sans JavaScript) ; `a_computed_list_gives_suggestions_too` ; `an_element_with_fields_proposes_its_first_field` ; `a_field_in_the_lines_of_a_list_uses_the_datalist_of_the_page` ; `suggestions_are_checked` (quinze refus, et une liste vide au départ permise).
- Dans Chrome : « des suggestions dans un champ, écrites ou venues d'une liste qui change ; on écrit autre chose (leçon 123) » (`input.list.options` des deux champs ; « Grenoble » écrit et gardé ; après « Retenir cette ville », six villes proposées ; pour le lecteur d'écran, le champ est une « combobox », `autocomplete="list"`). Sans la mise à jour du `datalist` dans `page-engine.js`, l'essai rate.
- axe-core 4.10.3 sur la leçon 123 : 0 défaut (thème clair à 1 000 px, sombre à 390 px).
- Leçon `123-des-suggestions-dans-un-champ.holo`.
