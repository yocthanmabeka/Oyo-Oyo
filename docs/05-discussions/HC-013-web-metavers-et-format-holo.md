# HC-013 — Le web devient le métavers : format `.holo`, moteur, imports, place de l'IA

## Métadonnées

- Date : 2026-09-21
- Statut : synthétisée
- Plateforme : Claude (Claude Code dans VS Code)
- URL : aucune — une session Claude Code est locale et n'a pas de lien de partage
- Parents : HC-001, HC-002, HC-003
- Enfants : aucun pour l'instant
- Note : les identifiants `HC-010` à `HC-012` sont réservés à des fiches Claude antérieures, pas encore publiées

## Problème étudié

Après la comparaison des trois propositions de code (PR n° 1 GPT5.6, n° 2 Claude, n° 3 Gemini), Yocthan a recentré le projet sur sa vision et a pris les premières décisions techniques : ce qu'est le métavers par rapport au web, ce qu'est un fichier source, dans quoi est écrit le moteur, ce qu'on a le droit d'importer, et où se place l'IA.

## Rappel de la vision de Yocthan

Elle ne figure pas encore dans [VISION.md](../00-vision/VISION.md).

- Remodeler notre façon d'utiliser le web : au lieu de pages, on explore un monde, on y entre et on y construit. Objectif 2030, par sprints de 24 heures.
- Big Bang : un petit point lumineux en 3D se morcelle en d'autres points ; à l'intérieur de chaque point se trouve un monde, ou une multitude de mondes. Ce point, une sphère, est aussi le premier personnage.
- Limite de perception : le premier test doit tenir dans 1 Go au maximum, sur n'importe quel téléphone actuel.
- Créer doit être à la portée de quelqu'un qui n'a jamais programmé.

## Conclusions

### Décidé par Yocthan

1. **Le métavers est une mise à jour du web, pas un jeu** (`ADR-007`). Son raisonnement : les métavers précédents, celui de Meta en tête, proposaient un jeu ; or bien plus de gens vivent sur Internet que dans les jeux. « Quelqu'un verra un web normal, mais pourtant c'est le métavers. » Formule retenue : une seule description, deux vues — à plat et en profondeur.
2. **Le fichier est une vraie source, pas un prompt** (`ADR-008`) : le même fichier donne toujours le même résultat.
3. **Format `.holo`** (`ADR-009`) : blocs nommés par leur sens, à la manière de Flutter ; le texte s'écrit en Markdown dans les blocs ; pas de `div` ; l'auteur n'écrit jamais de HTML, de CSS ni de JavaScript.
4. **Moteur en Rust** (`ADR-010`), qui tourne sous les navigateurs actuels, puis dans un navigateur propre au projet. Zig a été écarté : plus léger, mais pas stable, sans date de stabilité connue.
5. **Ponts vers JavaScript et CSS seulement, dans la première version** (`ADR-012`).

### Proposé par Claude, non validé

6. **Rendu par vue** (`ADR-011`) : vue à plat par génération de HTML et CSS, vue en profondeur par le moteur dans une zone de dessin.
7. **Deux étages et trois sortes d'import** (`ADR-013`) : HoloCode en haut, modules compilés en WebAssembly en bas.
8. **L'IA agit à la création, jamais à la lecture** (`ADR-014`).
9. **Règle des appels** (`ADR-015`) : calculs purs et demandes de capacité, jamais de code libre caché dans un bloc.

### Description honnête du paradigme

Yocthan a observé que l'holoscénique et l'orienté objet lui semblaient identiques. Claude lui a donné en grande partie raison : tout ce qui décrit « ce qui existe » (entités, archétypes, blocs) est de l'objet sans héritage. La différence porte sur « ce qui arrive » : en objet, n'importe quel objet peut en modifier un autre directement ; en HoloCode, tout changement d'état passe par un arbitre, le moteur, qui vérifie, peut refuser, et note. Formule proposée pour les documents : **des objets sans méthodes, des règles au niveau du monde, et des relations.** La valeur est dans ce qui est interdit, comme pour Rust face au C. L'originalité du projet est ailleurs : le point qui contient des mondes, le zoom, le web qui devient métavers, le budget de 1 Go.

### Comparaison des trois propositions de code

Notes données par Claude, qui est juge et partie ; ChatGPT et Gemini sont invités à remplir la même grille.

| Critère | GPT5.6 | Claude | Gemini |
|---|---|---|---|
| Facilité | 45 % | 35 % | 25 % |
| Petit stockage | 0 % | 0 % | 20 % |
| Fiabilité | 70 % | 75 % | 30 % |
| Fidélité à la vision | 5 % | 10 % | 50 % |
| Base pour un vrai langage | 15 % | 30 % | 5 % |

Aucune ne couvre plus de 30 % des objectifs ; le meilleur de chacune en couvre environ 44 %. Gemini a la meilleure direction et la preuve la plus faible (voir la revue dans la PR n° 3).

## Désaccords et limites

- Yocthan voudrait, si possible, aucun HTML ni CSS du tout. Claude a répondu qu'un navigateur actuel exige une page HTML d'une dizaine de lignes comme porte d'entrée, générée automatiquement ; l'absence totale de HTML n'est possible que dans le navigateur propre au projet.
- Un langage généraliste « mieux que C, Rust et Zig » reste l'ambition de Yocthan. Claude estime ses chances à 2 % d'ici 2030 pour une personne seule, et propose d'y revenir une fois le premier étage vivant.
- Dans un onglet de téléphone, le budget mémoire réel est plutôt de 300 à 500 Mo que de 1 Go. À vérifier par la mesure.
- Aucune de ces décisions n'a encore été mise à l'épreuve d'un prototype.

## Décisions produites ou affectées

- Produites : `ADR-007` à `ADR-015`.
- Affectées : `ADR-003` et `ADR-004` (la règle des appels précise ce que sont une capacité et un phénomène) ; `ADR-005` (le matériel existant devient : tout téléphone actuel, dans un navigateur).
- Dette décisionnelle : « choisir le langage d'implémentation » est traité par `ADR-010` pour le moteur.

## Projets affectés

- Holoverse, HoloCode, HoloCompiler, HoloRuntime, HoloEngine, HoloCode-Core.

## Documents mis à jour

- [Registre des décisions](../02-gouvernance/DECISIONS.md), [index des discussions](../02-gouvernance/DISCUSSIONS.md), fiches `docs/02-gouvernance/adr/ADR-007` à `ADR-015`.
- À mettre à jour ensuite, sur décision de Yocthan : [VISION.md](../00-vision/VISION.md) (Big Bang, deux vues, 1 Go, téléphone), [PARADIGME-HOLOSCENIQUE.md](../01-holocode/PARADIGME-HOLOSCENIQUE.md) (description honnête, règle des appels), [ARCHITECTURE.md](../01-holocode/ARCHITECTURE.md) (moteur Rust, deux enveloppes, deux étages), [ROADMAP.md](../04-roadmap/ROADMAP.md) (sprint Big Bang).

## Questions ouvertes

- Que vaut la vue en profondeur pour une page ordinaire : un vrai usage, ou un gadget ?
- Comment chaque bloc (`Texte`, `Bouton`, `Liste`…) se présente-t-il dans la vue en profondeur ?
- Les formes 3D définies par des formules (SDF), inspirées des modificateurs de Blender, tiennent-elles la chauffe d'un téléphone ?
- Premier sprint proposé, non validé : le Big Bang sur un vrai téléphone — point lumineux, zoom, morcellement, entrée dans un monde né d'une graine, piloté par un fichier `.holo` d'une dizaine de lignes, avec mesure du poids du moteur, de la fluidité et de la mémoire.
