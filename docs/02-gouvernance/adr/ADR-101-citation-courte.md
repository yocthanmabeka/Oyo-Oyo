# ADR-101 — Une citation courte et le titre d'une œuvre : `<<bonjour>>`, `_Les Misérables_`, `Quote(work:)`

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-09, après avoir tout essayé : « j'ai tout essayé et c'est bon »)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #217 (« Dette du web : la citation courte au milieu d'une phrase (q, cite) »), ajoutée à la file à la demande de Yocthan le 2026-10-09 ; le grand tableau du web, où `q, cite` était « en partie » (`Quote("…", by:)` ne faisait que la citation en bloc).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Citer quelqu'un au milieu d'une phrase (« il a dit “oui” ») est courant. Un texte HoloCode s'écrit entre guillemets droits, `"…"`, et ne peut pas en contenir : il fallait taper « » ou “ ” soi-même, ceux de sa langue, à la main.
- En français, une espace sépare le guillemet du mot. Une espace ordinaire laisse le navigateur couper la ligne juste après « : le guillemet reste seul en bout de ligne. Il faut une espace fine insécable, que presque personne ne sait taper.
- HTML a `q` : le navigateur y ajoute des guillemets par le style. Mais ces guillemets ne se copient pas avec le texte, et une page lue sans style les perd.
- Le titre d'une œuvre (un livre, un film) a `cite` en HTML ; HoloCode n'avait que l'italique (`*…*`), qui ne dit pas que c'est un titre.

## Décision

1. **`<<bonjour>>`** dans un texte : une citation courte, en `<q>`.
   - Le moteur écrit les guillemets de la langue de la page (`Page(lang:)`), pour de vrai : ils se lisent, se copient, et restent sans style. En français, « bonjour », avec une espace fine insécable de chaque côté. En allemand, „ “. Sinon, “hello”.
   - Une citation dans une citation prend les guillemets du second niveau : « il a dit “oui” ». En anglais, “… ‘yes’ ”.
   - Les espaces que l'auteur a mises contre les marques sont retirées : le moteur met les siennes.
   - Le navigateur n'ajoute pas ses propres guillemets par-dessus (`quotes: none`).
   - Une marque sans sa paire reste du texte. Rien dans du code.
2. **`_Les Misérables_`** dans un texte : le titre d'une œuvre, en `<cite>` (en italique au départ).
   - Le trait bas ouvre au début d'un mot et ferme à sa fin : `mon_fichier_final` reste tel quel. Rien dans du code.
3. **`Quote("…", by: "Victor Hugo", work: "Les Châtiments")`** : l'œuvre d'où vient une citation en bloc, en `<cite>`, après son auteur : « — Victor Hugo, *Les Châtiments* ». `work:` sans `by:` donne l'œuvre seule.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Écrire une citation courte | un bloc `Q(…)` (impossible : un texte n'a pas de blocs dedans) ; reconnaître « » et “ ” tapés à la main ; **une marque, `<<…>>`** | taper « » est difficile sur bien des claviers ; les reconnaître changerait toutes les pages qui en ont déjà ; `<<…>>` est neuf, facile à taper partout, et le même texte donne « » sur une page française et “ ” sur une page anglaise |
| Les guillemets | ceux du navigateur, par le style (`q` de HTML) ; **écrits par le moteur** | ceux du style ne se copient pas et disparaissent sans style ; écrits par le moteur, ils sont là pour tous, avec l'espace fine insécable du français |
| Le titre d'une œuvre | l'italique `*…*` ; un paramètre seulement dans `Quote` ; **`_…_`, comme l'italique de Markdown, et `Quote(work:)`** | un titre se nomme aussi au milieu d'une phrase ; `_…_` se lit comme de l'italique, que le titre garde au départ ; `*…*` reste l'emphase |
| Le mot du paramètre | `source:` (déjà le fichier d'une image) ; `title:` (déjà le titre de la page) ; **`work:`** | un mot qui ne sert pas déjà ailleurs |

## Ce qui est refusé, et pourquoi

- `Quote(work:)` qui n'est pas un texte, ou un texte vide : le titre de l'œuvre est un texte.

## Les défauts du web évités

- **Le guillemet seul en bout de ligne**, après une espace ordinaire : ici, l'espace est fine et insécable.
- **Les guillemets du style**, perdus au copier-coller et sans style : ici, écrits dans le texte.
- **Les guillemets d'une autre langue** tapés par habitude : ici, ceux de la page.
- **L'italique qui fait semblant d'être un titre** : ici, `cite` le dit aux machines.

## Dettes

- Une citation courte n'a pas encore sa source (`cite=` de HTML : une adresse).
- D'autres langues que le français, l'allemand et l'anglais prennent les guillemets anglais.

## Critères de validation

- Tests du moteur :
  - `a_short_quote_takes_the_marks_of_the_page_language` : le français et l'anglais, deux niveaux, les espaces retirées, une marque seule, le code, `quotes: none` ;
  - `the_title_of_a_work_is_a_cite` : un titre, un nom de fichier gardé, le code ;
  - `a_quote_names_the_work_it_comes_from` : avec et sans auteur, et un refus.
- Dans Chrome : « une citation courte et le titre d'une œuvre, avec les guillemets de la langue (leçon 124) ».
  - Le texte lu (`innerText`) contient les guillemets et les espaces fines.
  - La citation intérieure a les guillemets du second niveau.
  - Le navigateur n'en ajoute pas.
  - Deux titres d'œuvres sont marqués.
- Leçon `124-une-citation-courte.holo`.
