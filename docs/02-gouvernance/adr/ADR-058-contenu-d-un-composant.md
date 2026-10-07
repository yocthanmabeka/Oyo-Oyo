# ADR-058 — Un emplacement pour du contenu dans un composant : `children`

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : la revue de Codex (l'emplacement, remis à plus tard dans `ADR-056`) ; `ADR-050`
- Validation : Yocthan, le 2026-10-07 : « un emplacement pour contenu dans un composant. Oui […] dans Flutter, il y avait children […] et child […] il faut un peu voir de la bonne manière comment est-ce qu'il faudra le placer. »
- Projets affectés : HoloCode, HoloEngine

## Décision

1. **`params: [title, children]`** déclare l'emplacement. Le mot `children` posé seul dans une liste du contenu marque où va le contenu : `Column(children: [ H2("{title}"), children ])`.
2. **À l'appel, `children: [ … ]`**, une liste de blocs, le même mot que partout ailleurs dans HoloCode. Sans elle, l'emplacement reste vide.
3. **Un seul mot.** Flutter a `child` et `children` ; HoloCode garde `children`, même pour un seul bloc, comme il le fait déjà pour `Column` ou `Page`. `child` est refusé avec le bon mot. Un auteur n'a pas à choisir.
4. **Le contenu appartient à la page** : il est déplié là où il est écrit (il peut contenir des composants, et le même), puis posé dans la copie sans être renommé ni touché par les paramètres ; ses règles restent celles de la page.
5. Refusés, avec le bon mot : `children` déclaré mais pas posé, posé mais pas déclaré, une valeur par défaut, un contenu qui n'est pas une liste, un contenu donné à un composant qui n'a pas d'emplacement.

## Ce qui n'est pas fait

- Plusieurs emplacements nommés (`header`, `footer`) : quand un exemple réel le demandera.

## Critères de validation

- Tests : contenu posé à la bonne place, noms gardés, carte dans une carte, emplacement vide, règles du contenu et de la copie ; les sept refus.
- La leçon 75 et son essai écrit ; dans Chrome, avec `?valeurs`.
- L'audit axe-core : les 75 leçons, 0 défaut.
