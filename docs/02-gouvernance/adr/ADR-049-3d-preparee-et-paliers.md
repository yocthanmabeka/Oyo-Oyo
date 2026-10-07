# ADR-049 — La 3D : des objets préparés à l'avance, des paliers qui bougent pendant la visite

- Statut : ACCEPTÉ (direction ; construction à faire, après le feu vert de Yocthan)
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : les mêmes que `ADR-048` ; la revue de Codex du 2026-10-04 (PR 74, sa voie D) ; la proposition de Claude du 2026-10-04 (`proposals/Claude/modeles-3d-et-jeu-a-plusieurs-2026-10/`), dont l'option B (des objets faits de points) est **remplacée** par cette décision pour les objets proches
- Validation : Yocthan, le 2026-10-06 : « tu mets décision prise » ; le chantier attend son feu vert.
- Projets affectés : HoloEngine, outils, HoloCode

## Contexte

Le moteur ne dessine que des points, additionnés, sans profondeur : il ne sait pas cacher l'arrière d'un objet derrière son avant (Codex, 2026-10-04). Gemini, ChatGPT et Codex, consultés le 2026-10-06, jugent l'idée de Yocthan possible sans revoir la vision. La veille du même jour confirme que les praticiens font ainsi, et ajoute des contraintes (WebGL 2 toujours nécessaire, WebGPU parfois plus lent sur Android, mémoire serrée sur iPhone).

## Décision

### 1. Ce que l'on dessine

- **Les objets proches sont pleins** : des surfaces, avec un tampon de profondeur, des matières et une lumière.
- **Les points restent** pour le lointain, les passages, le morcellement et pour « entrer » dans un objet. Un point dessiné n'est pas un `Point` du langage (Codex).

### 2. Tout ce qui est lourd se prépare à l'avance

- Un **outil sur PC** lit un modèle glTF, le vérifie (droits compris), le simplifie et produit un fichier **`.holo3d`** : plusieurs niveaux de détail, une version en points pour le lointain, textures KTX2 avec mipmaps, géométrie compressée avec **meshoptimizer**, lumière et ombres de contact précalculées.
- Le `.holo3d` contient des **données, jamais du code** ; il est **versionné** (sections optionnelles ignorées par un ancien lecteur, sections obligatoires refusées si inconnues) et **borné** : le moteur vérifie toutes les tailles et la mémoire attendue **avant** d'allouer, et refuse au-delà.
- Les niveaux de détail se choisissent par **erreur géométrique en mètres**, convertie en pixels selon la caméra.

### 3. Les conventions, dès le premier jour

- **1 unité = 1 mètre** ; les axes de glTF, convertis une seule fois par l'outil.
- **Couleurs calculées en linéaire**, rendu des couleurs à la fin : **Khronos PBR Neutral** par défaut, AgX en option.
- Caméra, matrices et matières sont des ressources communes, séparées des objets.

### 4. Un seul moteur, des étapes de dessin séparées

- Le dessin est découpé à la main en étapes : préparation, objets pleins, points, ciel, effets, sortie. On ne généralise qu'après deux ou trois vraies étapes (Codex).
- **Le chemin de base n'utilise pas de calcul général (compute)** : il doit être le même en WebGPU et en WebGL 2.
- **WebGL 2 reste obligatoire.** Le moteur ne prend pas WebGPU d'office : il mesure, et peut garder WebGL 2 sur un appareil où WebGPU est plus lent ou instable (veille du 2026-10-06).

### 5. Les paliers bougent pendant la visite

- Trois paliers, **léger**, **normal**, **haut**, sont trois réglages du même moteur, pas trois moteurs.
- Le palier de départ dépend des capacités (WebGL 2 : léger ; WebGPU : normal ; PC à l'aise : haut), **jamais du nom de l'appareil**.
- Ensuite, le moteur suit le temps d'image sur une fenêtre glissante, et descend ou remonte **un réglage à la fois**, dans cet ordre : densité de l'écran, ombres, textures, puis formes. Il descend vite et remonte lentement. Au palier léger, 30 images par seconde stables valent mieux que 45 à 60 instables.
- Un changement de niveau de détail ne se fait que quand le nouveau est prêt, avec un écart entre les seuils de montée et de descente, et une courte transition si la différence se voit.

### 6. Les extensions

- **Un module enfermé (`ADR-045`) calcule, mais ne touche jamais au GPU** : il reçoit des données bornées et rend des données bornées, que le moteur vérifie avant de les dessiner.
- **Une extension de rendu** (ciel avancé, splats sur PC, effets du palier haut) est du **code du moteur**, chargé seulement quand l'appareil le permet et qu'une page en a besoin, comme `page-engine.js` (`ADR-033`). Aucune extension ne supprime le chemin de repli d'un objet.
- Pas de pont JavaScript (`ADR-011`, partie B).

### 7. Ce qui n'est pas pour maintenant

- Le lancer de rayons, les Gaussian splats, la géométrie virtuelle (type Nanite), les ombres en direct au palier léger. Rien de cela n'est interdit pour toujours (`ADR-048`).

## Budgets de départ (paramètres d'essai, pas des lois)

| | Léger | Normal | Haut (PC) |
|---|---|---|---|
| Chaise : triangles | 1 500 à 2 000 | 5 000 à 7 000 | environ 12 000 |
| Chaise : textures | 512 px | 1 024 px | 2 048 px |
| Chaise : téléchargement | ≤ 300 Ko | ≤ 1 Mo | ≤ 4 Mo |
| Mémoire propre de l'onglet | < 160 Mo | < 220 Mo | — |

Au-delà de 300 Mo sur un téléphone, ou si la mémoire monte encore après cinq minutes, la configuration est rejetée. Le chiffre de Gemini (« Chrome tue l'onglet à 350–450 Mo ») est écarté : aucune source.

## Premier chantier (après le feu vert)

1. Le tampon de profondeur, et le dessin des objets pleins séparé de celui des points.
2. Les conventions : mètres, axes, caméra, couleurs linéaires.
3. Un premier objet plein, sans texture, en WebGPU et en WebGL 2.
4. L'outil de préparation glTF → `.holo3d` version 1.
5. La matière (couleur, rugosité, métal), puis KTX2, puis le ciel et le rendu des couleurs.
6. **La chaise** : Rockingchair 01 de Poly Haven (CC0 ; environ 12 000 triangles, à confirmer), mesurée sur le Flip 3 en WebGPU et en WebGL 2, puis sur un téléphone modeste et un iPhone.

Les mots du langage (`Model`, `Material`, `Light`…) sont choisis **après** la chaise. Pour l'essai, une seule ligne : `Model(source: "chair.holo3d")`.

## Critères de validation

- La chaise vue de dos : les barreaux arrière sont cachés par l'assise.
- Le même fichier `.holo` en WebGPU et en WebGL 2.
- Aucune saccade au passage des points à l'objet plein ; mémoire stable après cinq minutes de rotation.
- Le budget du palier tenu sur le Flip 3, puis sur un téléphone modeste.
- L'œil de Yocthan : est-ce que ça fait réel ?
