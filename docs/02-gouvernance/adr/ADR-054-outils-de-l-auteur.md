# ADR-054 — Les outils de l'auteur : voir les valeurs, remettre en forme, des essais écrits

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md` (point 5 : « voir les valeurs pendant qu'on essaie la page, une mise en forme automatique, des essais écrits » ; HoloCode noté 35 sur les outils)
- Validation : Yocthan, le 2026-10-06 : « je suis tes propositions, je les valide, tu as mon feu vert ».
- Projets affectés : outils, HoloEngine

## Décision

1. **`?values`** dans l'adresse d'une page : un panneau montre ses valeurs à chaque changement (les nombres, les textes, les listes et leurs éléments, champs compris). Il ne sert qu'à l'essai ; sans `?values`, rien ne change.
2. **`holo fmt page.holo`** remet un fichier en forme comme les leçons du dépôt : deux espaces de plus après une ligne qui ouvre (une ou plusieurs parenthèses, crochets ou accolades), deux de moins quand ce qu'elle a ouvert se referme. Seuls les blancs changent : ni les mots, ni l'ordre, ni les commentaires, ni le contenu d'un texte long ; un commentaire aligné plus loin par l'auteur garde sa place. Le moteur lit exactement la même page avant et après.
3. **`holo test page.holo page.test`** joue un essai écrit avec l'arbitre de la page : `tap`, `signal`, `type`, `receive`, `expect`. Les mots sont anglais, comme le vocabulaire du langage (`ADR-016`). Un fichier d'essai n'est pas du HoloCode : c'est un outil, il n'ajoute aucun mot au langage. Les essais de `exemples/lecons/` sont joués par les tests du moteur.

## Alternatives écartées

- **Un débogueur pas à pas** : trop lourd pour le public de HoloCode ; voir les valeurs suffit à comprendre une règle.
- **Une mise en forme qui réécrit tout le fichier depuis l'arbre lu** : elle perdrait les commentaires et l'alignement voulu ; on ne touche qu'aux blancs.
- **Des essais écrits dans le fichier `.holo`** : de nouveaux mots dans le langage ; un fichier à part garde le langage petit.

## Critères de validation

- Tests : la mise en forme ne change que les blancs, deux fois de suite donne le même résultat, et le moteur fabrique la même page ; un essai réussi, un essai raté avec sa ligne et la valeur vue ; chaque essai des leçons passe.
- Sur 74 fichiers d'exemples, la mise en forme n'en change que deux, dont les lignes étaient vraiment mal alignées (`site-reference/commun.holo`, `pied.holo`).
- Le panneau `?values` dans Chrome, sur les leçons 70 et 71.
