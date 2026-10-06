Bonjour Codex. C'est Yocthan. Nous voulons commencer la 3D, et je veux ton avis avant la première ligne. Tu as accès au dépôt `yocthanmabeka/Metaverse` : lis, lance, et réponds en français, en phrases simples. Ne fusionne rien et ne change aucun statut de décision : range ta réponse dans `proposals/GPT5.6/`, par une pull request.

# Ce que nous voulons

Le but est d'être réaliste, **ultra réaliste si possible**. Mais on ne peut pas être ultra réaliste, « toutes options », dès le départ : on ferait exploser les machines pour rien. Nous voulons donc **concevoir la 3D pour qu'elle puisse croître petit à petit** : commencer bas, sur un téléphone ordinaire, puis monter en qualité étape par étape, sans tout réécrire. Si cela aide à atteindre l'illusion de la perfection, nous sommes prêts à utiliser des **extensions (plugins)** en plus du langage : le langage seul n'y arrivera pas d'un coup.

Dis-moi franchement si c'est possible ainsi, ou si je dois revoir ma façon de penser.

# Ce que tu dois lire

1. Ta revue du 2026-10-04 : `proposals/GPT5.6/architecture-3d-multijoueur-2026-10-04/README.md`, surtout ta voie D (modèles préparés avant la lecture) et ton budget de points.
2. `proposals/Claude/modeles-3d-et-jeu-a-plusieurs-2026-10/README.md` et la réponse de Gemini : `docs/05-discussions/reponses/2026-10-04-gemini-3d-et-plusieurs.md`.
3. `moteur/src/rendu.rs`, `moteur/src/rendu.wgsl`, `moteur/src/mosaique.rs` : le dessin d'aujourd'hui (points additifs, sans profondeur).
4. `docs/02-gouvernance/adr/` : `ADR-005` (téléphone, navigateur, 1 Go), `ADR-010` (Rust, WebAssembly, `wgpu`), `ADR-011` (rendu par vue ; ponts vers JavaScript rejetés ; modules enfermés acceptés, partie C ; HoloIR encore proposé, partie D), `ADR-037` (écriture des noms), `ADR-045` (premier module enfermé).
5. `moteur/README.md` pour les mesures sur téléphone et pour lancer.

# La direction proposée par Claude (2026-10-06)

- **Abandonner les points pour les objets proches** : sans profondeur ni surface, ils ne seront jamais réalistes. Les garder pour le lointain et pour « entrer » dans un objet.
- **Ta voie D** : un outil, sur PC, transforme un modèle glTF en un petit format propre au projet, avec plusieurs niveaux de détail (complet de près, simplifié de loin, en points de très loin), textures compressées, **lumière calculée à l'avance**. Le téléphone ne fait que lire.
- **Des paliers de qualité automatiques** : léger (téléphone modeste), normal (Flip 3 / Flip 5), haut (PC). L'auteur écrit « ici, une chaise » ; le moteur choisit le palier, et descend tout seul s'il n'arrive plus à 60 images par seconde.
- **Le réalisme par la lumière et les matériaux**, pas par le nombre de triangles.
- Pas maintenant : lancer de rayons en direct, Gaussian splats, Three.js ou Babylon.
- Premier essai : **une chaise**, en points (B limité) et en objet plein préparé (D), sur le Flip 3, de près, de loin, de dos.

# Ce que je te demande

1. **Croître sans réécrire** : dans `rendu.rs`, quelle découpe permet d'ajouter plus tard un effet (ombres, reflets, occlusion ambiante, anticrénelage, ciel) comme un module, sans casser ce qui existe ni le repli WebGL 2 ? Propose les étages du rendu (passes) et ce qui doit exister dès le premier jour (tampon de profondeur, espace de couleur linéaire, rendu des couleurs, unités du monde en mètres ?) pour ne pas devoir tout reprendre.
2. **Les paliers** : comment le moteur mesure-t-il l'appareil et choisit-il son palier ? Que mettre dans chaque palier ? Comment descendre de palier sans saccade ni objet qui saute ?
3. **Le format préparé** : que doit-il contenir dès la version 1 pour accueillir plus tard des matériaux plus riches, plus de niveaux de détail, la lumière précalculée, sans casser les anciens fichiers ? Comment le borner (octets, sommets, textures, mémoire après décodage) ?
4. **Les extensions** : avec `ADR-011` (pas de pont JavaScript ; modules WebAssembly enfermés), où peut vivre une extension de rendu ? Un module enfermé peut-il fournir un effet ou un type d'objet (par exemple des Gaussian splats au palier PC), ou faut-il que ce soit du code du moteur, chargé à la demande ? Quels risques (sécurité, poids, plantage du GPU) et quelles bornes ?
5. **Le langage** : le moins de mots possible. L'auteur décrit une intention (« bois ciré », « soleil couchant ») ou des réglages (rugosité, métal) ? Propose une écriture courte pour un objet, une matière et une lumière, en respectant `ADR-037`, et dis ce qui doit rester caché au moteur.
6. **Le premier essai** : précise la chaise de référence (où la prendre, avec quels droits), les budgets de départ par palier (triangles, mémoire de textures, objets dessinés, mémoire de l'onglet), et le protocole de mesure (WebGPU et WebGL 2, Flip 3, et un téléphone modeste si possible).
7. **Les limites dures** : ce qui restera impossible dans un navigateur de téléphone en 2026, quoi qu'on fasse.

Pour chaque point : sépare ce que tu as vérifié en lançant le code de ce que tu supposes, et donne les estimations comme des estimations. Termine par une liste classée, du plus important au moins important, et une réponse claire, en une phrase, à ma question : possible ainsi, ou à repenser ?
