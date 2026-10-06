# ADR-048 — HoloCode décrit une qualité, jamais une technique de dessin

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : le prompt et les réponses du 2026-10-06 sur la 3D réaliste (`docs/05-discussions/prompts/2026-10-06-*-3d-realiste.md`, `docs/05-discussions/reponses/2026-10-06-*`) ; la proposition de Codex (PR 118) ; la synthèse de Claude (`proposals/Claude/3d-realiste-2026-10/SYNTHESE.md`) ; la veille du 2026-10-06 (`docs/05-discussions/veille/2026-10-06-3d-web-et-mobile.md`)
- Validation : Yocthan, le 2026-10-06, après la veille : « tu mets décision prise ».
- Projets affectés : HoloCode, HoloEngine

## Contexte

Yocthan veut une 3D « ultra réaliste si possible », qui commence bas et croît petit à petit, avec des extensions s'il le faut. La technique de dessin change vite : en 2026, le lancer de rayons n'existe pas sur le web, les Gaussian splats viennent d'être normalisés dans glTF (`KHR_gaussian_splatting`), glTF 2.1 est annoncé. Si le langage nomme une technique, chaque technique nouvelle oblige à réécrire les fichiers des auteurs.

## Décision

1. **Un fichier HoloCode décrit ce que l'auteur veut voir** : un objet, une matière, une lumière, une ambiance, au plus une intention de qualité. **Jamais une technique de dessin.**
2. **Toute technique reste remplaçable par le moteur** : triangles, points, lancer de rayons, splats, ou une technique future. Le moteur choisit selon l'appareil ; un fichier `.holo` ne change pas quand le moteur apprend à mieux dessiner.
3. **Aucun mot de plomberie dans le langage** : triangles, mipmaps, compression de textures, ombres par carte, anticrénelage, nombre d'échantillons, format de texture, sRGB, WebGPU, WebGL, shader, poids en octets. Une faute qui en utilise un est refusée avec une explication, comme les autres (`ADR-037`).
4. **Aucune technique n'est interdite pour toujours.** Une technique que le parc d'appareils ne porte pas aujourd'hui (lancer de rayons, splats sur téléphone) n'est simplement pas une dépendance ; elle peut devenir un palier plus tard.

## Alternatives écartées

- **Laisser l'auteur choisir la technique** (`render: splats`, `shadows: high`) : plus de contrôle, mais le fichier vieillit avec la technique, et l'auteur doit apprendre le métier du moteur.
- **Un poids par objet dans le langage** (`weight: 250KB`, proposé par Gemini) : une chaise n'a pas un poids, elle a plusieurs versions (ChatGPT, Codex). Le budget relève de l'outil de préparation et du moteur.

## Conséquences

- Les mots de la 3D seront peu nombreux, et choisis après l'essai de la chaise (`ADR-049`).
- Un réglage précis de matière (`roughness`, `metallic`) pourra exister comme échappatoire artistique, s'il est demandé ; jamais comme l'écriture principale.

## Critères de validation

- Un même fichier `.holo` s'affiche sur un téléphone modeste, un Flip 3 et un PC, sans changement, avec une qualité différente.
- Le jour où le moteur gagne une nouvelle technique, aucun fichier d'exemple ni aucune leçon ne change.
