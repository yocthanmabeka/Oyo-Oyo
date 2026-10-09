# Le planning de la 3D

- Auteur : Claude
- Date : 2026-10-07
- Statut : **PROPOSITION**, à valider par Yocthan avant toute construction.
- **Quand** : après les neuf lots du web (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`), décidé par Yocthan le 2026-10-07 : « que la 3D commence après les lots 9. Au lieu de s'empresser, c'est mieux qu'on puisse terminer le lot 9 et ensuite commencer la 3D tranquillement ». Le contenu des étapes reste à valider.
- Repose sur : `ADR-048` (une qualité, jamais une technique), `ADR-049` (objets préparés à l'avance, paliers qui bougent, WebGL 2 obligatoire), `ADR-053` (le dessin à part, chargé quand il sert), `ADR-056` (le mot `Part` gardé pour la 3D) ; la veille du 2026-10-06 ; les avis de Gemini, ChatGPT et Codex.

## Le but

Une **chaise réaliste** (Rockingchair 01 de Poly Haven, CC0), posée dans un monde, qu'on regarde de près, de loin et de dos, à 60 images par seconde sur le Flip 3, en WebGPU comme en WebGL 2. Puis plusieurs objets, puis un monde qu'on a envie de visiter. Chaque étape se voit, se mesure, et ne casse pas ce qui marche.

## Une contrainte à connaître

Dans le nuage, Chrome n'a pas de carte graphique : Claude peut écrire et tester le code, mais ne voit pas toujours l'image. L'étape 0 essaie de lever cette limite ; sinon, **chaque étape se regarde sur le PC et le téléphone de Yocthan**, avec la pile (`/pile`) et des captures.

## Les étapes

| # | Ce qu'on construit | Ce qu'on voit à la fin | Qui vérifie | Durée |
|---|---|---|---|---|
| 0 | **Voir dans le nuage** : demander à la carte graphique de secours de Chrome (SwiftShader) les limites qu'elle a vraiment, au lieu de limites fixes ; si ça marche, Claude voit les images sans le PC. Et Yocthan vérifie la leçon 9 zoomée sur son PC (le dessin séparé, `ADR-053`). | La leçon 9 en points, dans le nuage et sur le PC | Claude, Yocthan | ½ séance |
| 1 | **La profondeur** : un tampon de profondeur, et le dessin des objets pleins séparé de celui des points (`ADR-049` § 4). Rien ne change à l'écran. | Le Big Bang et la leçon 9 identiques à avant (captures avant/après) | Claude, Yocthan | 1 séance |
| 2 | **Les conventions** : 1 unité = 1 mètre, l'axe vertical de glTF, une caméra en perspective, les couleurs calculées en linéaire, le rendu des couleurs Khronos PBR Neutral à la sortie | Toujours rien de changé pour les points ; un essai chiffré des matrices | Claude | 1 séance |
| 3 | **Un premier objet plein, sans fichier** : un cube puis une sphère, fabriqués par le moteur, éclairés par une lumière, dans un monde d'essai (`moteur/mondes/essai-3d.holo`, ouvert par une adresse d'essai, sans mot nouveau dans le langage) | Un cube éclairé qu'on tourne au doigt, en WebGPU et en WebGL 2 ; les faces cachées le sont bien | Yocthan (PC, Flip) | 1 à 2 séances |
| 4 | **L'outil de préparation** : `holo prepare chaise.gltf` écrit `chaise.holo3d` v1 (versionné, borné : `ADR-049` § 2) ; géométrie seule d'abord, plusieurs niveaux de détail simplifiés | Le fichier de la chaise : son poids, ses triangles par niveau ; des refus pour un fichier hostile | Claude (tests) | 2 séances |
| 5 | **Lire `.holo3d` dans le moteur** : vérifier les tailles avant d'allouer, poser la chaise grise dans le monde d'essai | La chaise grise, de face et de dos : les barreaux arrière cachés par l'assise | Yocthan | 1 séance |
| 6 | **La matière** : couleur, rugosité, métal (le modèle de glTF), et l'éclairage par une image du ciel | La chaise en bois, avec ses reflets ; « est-ce que ça fait réel ? » | Yocthan | 2 séances |
| 7 | **Les textures** : KTX2, transcodées selon l'appareil (ASTC ou ETC2 sur téléphone, BC sur PC) ; le décodeur chargé seulement pour une page en 3D, son poids mesuré | La chaise texturée ; le poids téléchargé et la mémoire, mesurés | Yocthan (mesures), Claude | 2 séances |
| 8 | **Les niveaux de détail et le lointain** : de près l'objet complet, de loin simplifié, de très loin en points ; changer sans saut | Une marche arrière depuis la chaise sans à-coup jusqu'aux points | Yocthan | 1 à 2 séances |
| 9 | **Les paliers qui bougent** : le moteur suit le temps d'image et règle la densité d'écran, les ombres, les textures, puis les formes | Le palier choisi et ses changements, visibles dans `?values` ; 60 images par seconde tenues | Yocthan | 1 séance |
| 10 | **Les mesures** : Flip 3 en WebGPU et en WebGL 2 ; un téléphone modeste ; un iPhone si possible ; 15 minutes de rotation (images par seconde, mémoire, chauffe) | Le tableau des mesures, dans `moteur/README.md` | Yocthan | 1 séance |
| 11 | **Les mots du langage**, choisis après la chaise, selon la règle « la forme la plus courte » : sans doute `Model(source: "chaise.holo3d")`, peut-être une lumière et une matière ; une leçon | Une page `.holo` avec une chaise, écrite en une ligne | Yocthan décide | 1 séance |

**En tout : environ 15 séances**, à ajuster après les étapes 3 et 6, qui diront si l'image tient ses promesses.

### Précisions pour les étapes 8 à 10

Elles viennent d'une réflexion de Yocthan avec Claude, le 2026-10-09, jugée bonne par la session Claude du PC et ajoutée ici avec son accord.

- **Étape 8, « de très loin en points »** :
  - de loin, l'objet est dessiné dans une petite image, et cette image devient des points par le mécanisme de la vue points (`mosaic.rs`) ;
  - le parent s'efface pendant que les points apparaissent, comme aujourd'hui pendant un morcellement : rien de nouveau à inventer.
- **Étape 9, le garde des paliers** :
  - **Il réagit à une baisse qui dure, pas à un à-coup.** Un téléphone qui chauffe se ralentit tout seul après quelques minutes, et la fenêtre glissante d'`ADR-049` est faite pour le voir.
  - **Il surveille aussi la mémoire, mais celle qu'il connaît.** Une page ne peut pas lire de façon fiable la mémoire réelle de son onglet. Le moteur compte donc ce qu'il réserve lui-même (textures, tampons, mémoire WebAssembly), et il descend d'un palier quand il approche du budget de ce palier, même si l'image reste fluide.
  - La vraie mémoire de l'onglet se mesure par le câble pendant les essais (`ADR-049` : au-delà de 300 Mo, ou si elle monte encore après cinq minutes, la configuration est refusée).
- **Étape 10, les mesures** : la durée de la fenêtre et les seuils du garde se règlent avec le script de 15 minutes de batterie et de chaleur, `moteur/outils/mesures/duree.js` (images par seconde minute par minute).

## Après la chaise (pas dans ce planning)

**À la fin de la 3D : l'IA dans le langage.** Yocthan, le 2026-10-09 : « à la fin de la 3D, on réfléchira à l'intégration de l'IA ». Il n'a jamais vu de langage qui l'inclue. Les contraintes déjà connues :

- aucun prestataire obligatoire ;
- le déterminisme (`ADR-008`) ;
- le budget d'un onglet de téléphone ;
- un point d'appui possible : les modules enfermés (`ADR-045`, `ADR-077`).

**La piste retenue pour cette réflexion** (2026-10-09) : l'IA intervient **à la préparation, chez l'auteur, jamais pendant la visite**, comme `holo prepare`. Son résultat est écrit dans le fichier, relu et figé, et le visiteur ne reçoit jamais d'IA.

- Une IA sur l'appareil du visiteur donnerait des résultats différents selon les appareils. Au mieux, ce serait un décor dont la page ne dépend jamais.
- Les modules enfermés ne conviennent pas à un vrai modèle : mémoire plafonnée, temps limité, pas de carte graphique, réponse de 64 Ko au plus. À la préparation, l'IA tourne chez l'auteur, en dehors du navigateur, sans module.
- Une seule nuance à étudier : une toute petite IA figée, calculée en nombres entiers, pourrait tenir dans un module et donner le même résultat partout.

**Les surfaces : planètes et terrains** (une note, pas une décision : ce n'est pas dans le plan).

- **Une tuile égale une graine**, tirée de la graine du parent et de la position de la tuile, comme les enfants d'un `Point` (`seed.rs`). Rien n'est stocké.
- **Le relief de fond vient d'une seule fonction continue pour toute la planète** (la graine de la planète et la position) : deux tuiles voisines calculent ainsi la même hauteur sur leur bord commun. La graine d'une tuile sert à placer des objets (rochers, arbres), jamais à changer le sol près des bords.
- **Le calcul se fait en nombres entiers**, comme les graines, pour que la même planète naisse sur tous les appareils.
- **Les fentes se recousent** : deux tuiles voisines de détail différent laissent des fentes entre elles.
- **Le nombre de tuiles à l'écran est plafonné**, comme les 6 344 points de la vue points, pour que l'onglet reste borné quelle que soit la taille de la planète.

L'idée d'un monde en fragments (quadtree sphérique, chargement des voisins, garde par seuils) a reçu sa réponse dans `docs/05-discussions/prompts/2026-10-09-claude-etat-du-projet-et-monde-en-fragments.md` : ses parties utiles rejoignent les étapes 8 et 9.

Les ombres ; plusieurs objets et un budget de scène ; **`Part`** (les pièces d'un objet, ou les blocs d'un monde à la Roblox, mot gardé par `ADR-056`) ; un personnage et la vue qui le suit ; le jeu à plusieurs ; les splats sur PC.

## Ce qui peut faire échouer ce planning

- **Le rendu dans le nuage reste impossible** : chaque étape attend alors un essai de Yocthan ; le planning s'allonge, sans changer.
- **La chaise ne fait pas réel au palier normal** : on le saura à l'étape 6 ; alors on revoit les budgets (`ADR-049`) avant d'aller plus loin.
- **Le décodeur KTX2 pèse trop** : il est chargé seulement pour la 3D ; s'il reste trop lourd, des textures plus simples au palier léger.
- **Trop de mots** : aucun mot nouveau avant l'étape 11.

## Ce que Yocthan a à décider

1. Ce planning, et son ordre.
2. Commencer par l'étape 0 (voir dans le nuage, et vérifier la leçon 9 sur le PC).
