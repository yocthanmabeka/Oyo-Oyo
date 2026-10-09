# ADR-092 — Des polices libres pour toutes les écritures : `Font(family: "Inter")`

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-09, après avoir tout essayé : « j'ai tout essayé et c'est bon »)
- Date : 2026-10-08
- Responsable : Yocthan Mabeka
- Discussions sources : la demande de Yocthan (leçon 54), confiée à la session du nuage le 2026-10-08 (`AGENTS.md`, « une sélection d'une trentaine de polices libres, licences vérifiées, toutes les écritures, gardées dans le projet, chargées seulement quand une page les nomme ») ; l'`ADR-041` (`Font(family:, source:)`, une police rangée à côté).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Une page ne pouvait avoir sa police qu'en rangeant un fichier à côté (`Font(family: "Carlito", source: "carlito.woff2")`, `ADR-041`) : il fallait la trouver, vérifier sa licence, et choisir soi-même les morceaux à garder.
- Sur le web, on charge souvent ses polices depuis Google Fonts : le visiteur est alors connu d'un service de plus, et la page dépend de lui. « Chez soi d'abord » : rien ne doit l'exiger.
- Une police qui couvre toutes les écritures pèse des dizaines de mégaoctets ; le chinois, le japonais et le coréen comptent des dizaines de milliers de caractères.

## Décision

1. **Le moteur garde 32 polices libres**, dans `moteur/web/fonts/`, chacune dans son dossier avec sa licence. Une page en nomme une sans fichier : `Page(fonts: [ Font(family: "Inter") ])`, puis `font-family: Inter, sans-serif;` dans un style. Le nom se compare sans les majuscules (`"inter"` charge Inter).
2. **Chargées seulement quand une page les nomme, et seulement les morceaux utiles** : chaque police a sa feuille, `font.css`, qui dit pour chaque morceau (latin, cyrillique, arabe, et les 100 à 124 morceaux du chinois, du japonais et du coréen) les caractères qu'il contient (`unicode-range`). Le navigateur ne télécharge que ceux dont la page a besoin : la phrase japonaise de la leçon 115 fait venir 1 morceau sur 124. Une page qui ne nomme aucune police n'en charge aucune.
3. **Toutes les écritures courantes** : le latin (avec le vietnamien), le cyrillique, le grec, l'arabe, l'hébreu, le devanagari, le bengali, le tamoul, le thaï, l'éthiopien, l'adlam, le n'ko, le tifinagh, le chinois simplifié, le japonais, le coréen. Les genres : 22 sans empattements (dont Atkinson Hyperlegible Next et Lexend, faites pour être lues par tous), 5 avec empattements, 2 à chasse fixe pour le code, 3 écrites à la main. Toutes les graisses sont dans un seul fichier quand la police est variable ; l'italique dans un second, quand elle en a un. La liste est dans `moteur/web/fonts/README.md`.
4. **Licences vérifiées** : toutes sous la licence SIL Open Font License 1.1, avec le nom de leurs auteurs, et **sans « nom réservé »** : les fichiers découpés pour le web en sont une version modifiée, et la licence interdit alors de garder un nom réservé. Lora, Merriweather, Playfair Display et Dancing Script en ont un : elles sont remplacées par Literata, Noto Serif, Fraunces et Kalam. Un test vérifie que chaque police a sa licence et chacun des fichiers que nomme sa feuille. Le fichier `LICENSE` du dépôt dit qu'elles gardent leur licence.
5. **D'où elles viennent** : les fichiers de Google Fonts, tels que les distribue Fontsource (paquets npm, version 5.3.0, le 2026-10-08), sans modification. `moteur/outils/fonts.py` prépare une nouvelle police à partir de son paquet, et refuse une licence autre que l'OFL.
6. Une police rangée à côté (`source:`) marche comme avant, et passe avant une police du moteur du même nom.

## Ce qui est refusé, et pourquoi

- Une police que le moteur n'a pas, nommée sans fichier : le message donne la liste, ou demande le fichier (`source: "police.woff2"`).
- La même police nommée deux fois, même avec d'autres majuscules.
- Plus de 8 polices sur une page (la règle de l'`ADR-041`).

## Les défauts du web évités

- **Un service extérieur pour les polices** (Google Fonts, Adobe Fonts) : ici, chez soi, sans traceur.
- **Une police entière téléchargée pour trois caractères** : ici, seulement les morceaux utiles.
- **Une licence oubliée** : ici, vérifiée avant d'entrer dans le projet, et gardée à côté de chaque police.
- **Le texte invisible le temps que la police arrive** : toujours `font-display: swap`, comme pour une police rangée à côté.

## Dettes

- Les polices pèsent 20 Mo dans le dépôt, dont 12,6 Mo pour le chinois, le japonais et le coréen. Aucun visiteur ne les télécharge toutes.
- Le chinois traditionnel, l'arménien, le géorgien, le khmer, le birman, le cinghalais, le télougou et d'autres écritures n'ont pas encore leur police.
- Une phrase dans une autre langue n'a pas de `lang` à elle : un lecteur d'écran la lit avec la voix de la page.
- Une page exportée hors du moteur doit emporter le dossier `fonts/`.
- Une police nommée seulement dans un style (`font-family: Inter`), sans `Font(…)`, ne se charge pas : la page doit la nommer.

## Critères de validation

- Tests du moteur : `the_free_fonts_of_the_engine_load_by_their_name` (deux polices du moteur, dont une écrite en minuscules, et un fichier rangé à côté ; leurs feuilles en tête ; une page sans police n'en charge aucune) ; les refus (une police inconnue, deux fois la même) ; `every_font_of_the_library_is_kept_with_its_licence`.
- Dans Chrome : « des polices libres pour toutes les écritures (leçon 115) » (Literata, le japonais et l'éthiopien prêts ; 1 morceau japonais sur 124 ; le latin d'Inter, ni cyrillique ni grec ; aucune police pour une page qui n'en nomme pas).
- Leçon `115-des-polices-pour-toutes-les-ecritures.holo`.
