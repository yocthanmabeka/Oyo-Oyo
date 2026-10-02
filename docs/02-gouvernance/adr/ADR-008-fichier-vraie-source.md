# ADR-008 — Le fichier est une vraie source, pas un prompt

- Statut : ACCEPTÉ
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013
- Projets affectés : HoloCode, HoloCompiler, HoloRuntime
- Validation : décidé par Yocthan le 2026-09-21. La fusion de la pull request qui introduit cette fiche vaut confirmation.

## Contexte

En 2026, beaucoup de sites sont fabriqués par une IA à partir d'un texte (Lovable, par exemple). On pouvait donc imaginer qu'un fichier du projet soit un simple texte d'intention, transformé en page ou en monde par une IA à chaque ouverture.

## Décision

Le fichier est une vraie source : le moteur le lit et produit toujours exactement le même résultat, comme un compilateur. L'IA peut aider à écrire le fichier ; elle n'intervient jamais au moment où il est lu.

## Alternatives étudiées

- **Le fichier comme prompt** : le même fichier donne un résultat différent à chaque fois. On ne peut plus rien déboguer, deux personnes qui ouvrent le même lien ne voient pas le même monde, et le principe « même graine, même monde » tombe.

## Conséquences

### Positives

- Un monde est reproductible, vérifiable avant son exécution, et léger : il ne faut aucune IA pour le lire.
- Le journal causal reste possible : on peut expliquer pourquoi quelque chose est arrivé.

### Négatives et risques

- Il faut un vrai langage, avec sa grammaire, son vérificateur et ses messages d'erreur.
- L'expérience « une phrase, et un monde apparaît » doit passer par une IA qui écrit du `.holo` (voir `ADR-014`).

## Critères de validation

- Le même fichier, lu deux fois sur deux machines, produit le même état et le même journal.

## Conditions de réexamen

- Aucune prévue : c'est un principe fondateur.
