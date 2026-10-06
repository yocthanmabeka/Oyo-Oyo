# Réponse de Gemini — la 3D réaliste, qui croît petit à petit (2026-10-06)

Réponse au prompt [`../prompts/2026-10-06-gemini-3d-realiste.md`](../prompts/2026-10-06-gemini-3d-realiste.md), collée par Yocthan le 2026-10-06. Gemini n'a pas accès au dépôt. Texte gardé tel quel ; la lecture critique de Claude est à la fin.

---

Voici l'analyse méthodique et sans détour pour aborder la 3D réaliste et évolutive dans HoloCode.

## Réponse directe à ta question principale

Oui, ton idée est réaliste, à une condition stricte : le réalisme sur téléphone ne s'obtient pas en ajoutant du calcul en direct, mais en préparant des données parfaites hors-ligne pour que le GPU du mobile ne fasse que des opérations simples et prévisibles.

## 1. Commencer bas et croître : est-ce ainsi que font les grands ? Où est le piège ?

- Ce que font les grands moteurs :
  - Fortnite (Unreal Engine sur mobile) : Epic Games désactive purement et simplement le calcul lourd (Nanite et Lumen ne tournent pas sur mobile classique). Le jeu bascule sur un pipeline mobile dépouillé : ombres projetées simples, géométrie simplifiée et textures compressées.
  - Genshin Impact (Unity sur mobile) : Le jeu mise tout sur des shaders de surface stylisés très légers, une distance d'affichage strictement bornée et un brouillard dense pour éviter de dessiner au loin.
  - Roblox : Le curseur de qualité automatique agit principalement sur trois leviers stricts : la distance d'affichage, la résolution des ombres et le niveau de détail des maillages (LOD - Level of Detail).
- Le piège mortel :
  - Le piège est de croire qu'on peut concevoir un moteur « générique » auquel on ajoute simplement des options graphiques plus tard. En programmation graphique sur mobile, un pipeline mal conçu au niveau de l'architecture mémoire ne s'optimise pas par des options. Si ton moteur alloue de la mémoire à chaque frame ou gère mal le tampon de profondeur (Z-buffer), le téléphone plantera même sur le palier le plus bas. L'architecture de base doit être pensée pour le pire des téléphones dès le premier jour.

## 2. Les ingrédients du réalisme, classés par rapport qualité/coût

Sur téléphone, le « photo-réalisme » est une illusion d'optique produite par la justesse de la lumière, pas par des millions de triangles.

```
+-------------------------------------------------------------------------+
| RAPPEL CAPITAL : Sur mobile, 1 calcul de texture lourd = chauffe immédiate |
| Priorité : Transférer l'effort sur le précalcul hors-ligne              |
+-------------------------------------------------------------------------+
```

Voici le classement des techniques, de la plus rentable à la plus coûteuse :

- Le Tone Mapping (Rendu des couleurs) : Coût quasi nul (5 lignes de code dans le shader), gain immense. Utiliser une courbe de couleur cinématographique (comme ACES ou AgX) empêche les zones claires de brûler en blanc pur artificiel.
- L'IBL (Image-Based Lighting / Ciel HDR précalculé) : Coût très faible, réalisme immédiat. L'objet reflète les couleurs du ciel et du décor via une petite texture cubique floutée à l'avance.
- La compression de textures (KTX2 / Basis Universal) : Indispensable. Une texture PNG/JPEG de 2 Mo pèse 16 Mo une fois décompressée dans la RAM du GPU. Le format KTX2 reste compressé dans la mémoire de la carte graphique (gain de mémoire de 70 à 80 %).
- L'occlusion ambiante précalculée (Baked AO) : Les zones de contact (les pieds de la chaise sur le sol) sont assombries directement dans la texture. Zéro calcul dynamique sur mobile.
- Le matériau PBR basique (Physically Based Rendering) : Calcul standard des reflets selon la rugosité (Roughness) et l'aspect métallique (Metallic).
- Les LODs géométriques : Réduction automatique du nombre de triangles dès que l'objet s'éloigne.
- Les ombres dynamiques (Shadow maps) : Coûteux sur mobile car la scène doit être dessinée deux fois (une fois du point de vue de la lumière, une fois pour la caméra).

Répartition sur tes trois paliers :

| Technique | Palier Léger (Entrée de gamme) | Palier Normal (Flip 3 / Flip 5) | Palier Haut (PC) |
|---|---|---|---|
| Matériau | PBR simplifié sans reflets complexes | PBR complet (Rugosité + Métal) | PBR complet + détails fins |
| Éclairage | Ciel IBL basse résolution seul | Ciel IBL + 1 lumière directionnelle | IBL HD + lumières multiples |
| Ombres | Fausse ombre ovale au sol (texture plate) | 1 ombre portée dynamique douce | Ombres dynamiques haute résolution |
| Occlusion (AO) | Précalculée dans la texture | Précalculée dans la texture | SSAO dynamique (calculé en direct) |
| Textures | KTX2 (512x512 max) | KTX2 (1024x1024) | KTX2 (2048x2048) |
| Anticrénelage | Aucun (ou FXAA très léger) | MSAA 2x ou FXAA | TAA ou MSAA 4x |

## 3. L'architecture : que décrit le langage, que cache le moteur ?

Pour que le système ne devienne pas un fouillis technique, la séparation doit être totale :

- Ce que le langage .holo décrit (l'intention de l'auteur) : l'auteur ne manipule jamais de termes comme normal map, triangles, rasterizer ou MSAA.

```
Model(
  source: "chaise.holo3d",
  material: wood,
  finish: polished,  // "ciré / poli"
  weight: 250KB,
)
```

  L'auteur exprime la nature de l'objet et son budget.
- Ce que le moteur Rust cache (la plomberie) : le moteur inspecte le matériel au chargement. S'il détecte un GPU modeste, il charge le LOD 2 de la chaise avec la texture en 512x512. Sur PC, il charge le LOD 0 avec la texture en 2048x2048.
- Comment organiser le moteur en interne (Pattern Render Pipeline) : en Rust avec wgpu, sépare rigoureusement la scène de son rendu :
  - SceneGraph : La liste des objets, leurs positions et leurs boîtes d'englobement (Bounding Boxes).
  - RenderPass Trait : Une interface Rust pour chaque module visuel (PointsPass, OpaqueMeshPass, ShadowPass, PostProcessPass). Pour ajouter un nouvel effet plus tard, il suffira d'ajouter une RenderPass sans toucher à la façon dont les fichiers .holo sont lus.

## 4. Les extensions (Plugins) sans pont JavaScript

Puisque tu as justement rejeté les ponts JavaScript pour préserver la sécurité et la souveraineté du runtime, les extensions ont deux modes d'existence possibles :

- Les extensions internes au moteur (Features Rust compilées) : c'est la méthode la plus propre pour le socle. Des fonctionnalités lourdes (comme un ciel procédural physique ou le support glTF complet) peuvent être compilées sous forme de modules activables (Cargo features). Si le build cible un navigateur mobile léger, la feature est exclue à la compilation pour garder le binaire sous les 600 Ko.
- Les extensions en modules WebAssembly enfermés (ADR-013) : un module WebAssembly tiers ne doit jamais toucher directement à WebGPU ni au contexte graphique. Il doit agir comme un coprocesseur de calcul pur : il reçoit des données brutes en entrée, renvoie un tableau de calcul en sortie (par exemple : calculer la position de 500 particules de feu), et c'est le moteur Rust hôte qui décide comment les dessiner en toute sécurité.
- Ce qui peut raisonnablement être une extension :
  - Un ciel atmosphérique dynamique.
  - Un module de physique simple (collisions complexes).
  - Un décodeur pour formats volumineux réservés au PC (Gaussian Splatting expérimental sur PC uniquement).

## 5. Les limites dures en 2026 sur navigateur mobile

Voici les murs techniques infranchissables sur un smartphone dans un navigateur web :

- Le Ray Tracing en direct (Lancer de rayons WebGPU) : Totalement impossible sur mobile. Même si le matériel récent (Snapdragon 8 Gen 2/3) commence à supporter le ray tracing en natif, l'API WebGPU dans les navigateurs mobiles ne l'expose pas encore de manière stable et efficace.
- La mémoire vive réelle d'un onglet : Bien que tu vises 1 Go, sur un smartphone d'entrée de gamme, le navigateur Android (Chrome) abat brutalement l'onglet (tab discard) dès qu'il franchit 350 à 450 Mo de mémoire vive réelle pour protéger le reste du système.
- Le thermal throttling après 10 minutes : Si ton shader exécute plus de 3 ou 4 passes de post-traitement (flou, occlusion dynamique, ombres multiples), la puce mobile chauffe à 42°C, réduit sa cadence de 40 %, et le framerate s'effondre de 60 à 25 FPS.

## 6. D'où viendront les modèles réalistes ?

| Source | Utilisation pour HoloCode | Poids moyen | Droits et licences |
|---|---|---|---|
| Poly Haven (Recommandation n°1) | Idéal pour démarrer. Modèles PBR optimisés, propres, formats ouverts (glTF). | 2 à 10 Mo brut (compressable à 300 Ko avec ton outil). | CC0 (Domaine public) : utilisable partout, sans restriction, sans royalties. |
| Sketchfab | Très vaste catalogue, mais qualité d'optimisation très inégale (souvent trop de triangles). | Variable (souvent trop lourd, 20 à 80 Mo). | Licences diverses (CC-BY, standard payant). Nécessite de créditer les auteurs. |
| Photogrammétrie / Scans | Surfaces ultra-réalistes, mais géométrie brute ingérable sans nettoyage préalable. | Très lourd (centaines de Mo). | Dépend de la source. Inutilisable directement sans retopologie sur PC. |
| Génération par IA (Meshy, Tripo, CSM) | Prometteur pour générer des objets à la volée, mais maillages souvent sales et textures floues de près. | 5 à 15 Mo. | Selon les plateformes (souvent libre pour usage commercial sur abonnements payants). |

Règle pour le projet : les modèles bruts (glTF de Poly Haven) doivent impérativement passer par ton outil PC de conversion pour être réduits, nettoyés et compressés en .holo3d avant distribution.

## 7. Le premier essai : la chaise

Oui, la chaise est le test parfait. Elle contient des surfaces planes (l'assise), des formes fines (les pieds, barreaux) qui révèlent immédiatement les bugs d'anticrénelage, et des occlusions naturelles (le dessous du siège, les zones de contact avec le sol).

Budgets de départ stricts pour le test de la chaise :

| Métrique | Palier Léger | Palier Normal | Palier Haut (PC) |
|---|---|---|---|
| Triangles du modèle | ≤ 1 500 triangles | 5 000 à 8 000 triangles | 25 000 triangles |
| Mémoire textures GPU (compressé) | ≤ 512 Ko (cartes 512x512) | ≤ 2 Mo (cartes 1024x1024) | ≤ 8 Mo (cartes 2048x2048) |
| Nombre de draw calls (appels de dessin) | 1 appel unique | 1 à 2 appels | 2 à 4 appels |
| Poids du fichier téléchargé | ≤ 150 Ko | ≤ 500 Ko | ≤ 2 Mo |

Ce que vous devez mesurer sur le Galaxy Z Flip 3 :

- Le framerate stable (Frame Pacing) : vérifier qu'il n'y a aucun micro-gel (stutter) au moment précis où la caméra passe de la vue lointaine en points à la vue proche en objet plein.
- L'illusion de matière à 30 cm de l'écran : le bois ressemble-t-il à du bois grâce aux reflets du ciel IBL, sans que l'on voie de polygones anguleux ?
- Le Z-Buffer : vérifier que les barreaux arrière de la chaise sont masqués proprement par l'assise avant.
- La mémoire allouée dans le gestionnaire Chrome : elle ne doit pas augmenter après 5 minutes de rotation continue autour de la chaise.

## Tableau récapitulatif des recommandations

| Action | Statut | Justification en une phrase |
|---|---|---|
| Intégrer le Z-buffer dans le moteur Rust/wgpu | À faire maintenant | Prérequis incontournable pour dessiner n'importe quel objet plein sans voir à travers. |
| Créer l'outil PC de conversion glTF → format compact optimisé | À faire maintenant | Le smartphone doit consommer des données prêtes à afficher sans faire d'optimisation lui-même. |
| Implémenter l'IBL (éclairage par ciel) et le Tone Mapping ACES | À faire maintenant | C'est ce qui apporte 80 % du réalisme visuel pour un coût de calcul négligeable sur mobile. |
| Tester sur un modèle Poly Haven en licence CC0 (la chaise) | À faire maintenant | Données géométriques et textures propres disponibles immédiatement sans problème juridique. |
| Créer les 3 paliers automatiques de LOD et textures KTX2 | Plus tard | Nécessaire dès qu'une scène comportera plus de 5 objets simultanés. |
| Ajouter des ombres portées dynamiques douces | Plus tard | Coûteux sur mobile ; l'occlusion ambiante précalculée suffit largement au début. |
| Système d'extensions WebAssembly isolées pour le calcul | Plus tard | À brancher lorsque le moteur de rendu de base sera parfaitement stabilisé. |
| Intégrer des Gaussian Splats (3DGS) pour le mobile | À ne pas faire | Fichiers trop lourds et tri de profondeur intenable pour la batterie d'un smartphone. |
| Intégrer du Ray Tracing en direct | À ne pas faire | Incompatible avec le parc de smartphones ordinaires et hors de portée de WebGPU mobile aujourd'hui. |
| Autoriser des ponts JavaScript pour les graphismes | À ne pas faire | Détruirait la sécurité du métavers et briserait le contrôle strict des performances du moteur Rust. |

---

## Lecture critique de Claude

**D'accord**

- Le verdict : possible, à condition de **préparer hors ligne** et de laisser au téléphone des opérations simples. C'est la voie D de Codex et la direction de Claude ; les trois avis convergent.
- Le classement des ingrédients : rendu des couleurs (ACES ou AgX), éclairage par une image du ciel (IBL), occlusion précalculée dans la texture, matériaux physiques. Ce sont les plus rentables ; ils font le gros du réalisme pour presque rien.
- Le **tampon de profondeur d'abord** : c'est le défaut que Codex avait montré (le moteur ne cache pas l'arrière d'un objet derrière son avant).
- Le piège : penser au pire téléphone dès le premier jour, et ne rien allouer à chaque image.
- Les **modules enfermés comme calculateurs purs**, qui ne touchent jamais au GPU : c'est exactement le cadre d'`ADR-045` (le premier module enfermé), et la bonne réponse à la question des extensions.
- La fausse ombre au sol pour le palier léger ; Poly Haven en CC0 pour la chaise ; les mesures demandées (pas de saccade au passage des points à l'objet plein, mémoire stable après cinq minutes).

**Pas d'accord, ou à nuancer**

- **Aucune source citée**, alors que le prompt le demandait. Plusieurs chiffres sont donc des estimations sans appui : l'onglet abattu à 350–450 Mo, la chauffe à 42 °C, la chute de 40 %, les « 80 % du réalisme ».
- **L'onglet abattu à 350–450 Mo** : si c'est vrai sur un téléphone d'entrée de gamme, le budget de 1 Go d'`ADR-005` ne tient pas pour le palier léger. C'est la phrase la plus lourde de conséquences de la réponse ; à mesurer, pas à croire.
- **KTX2** : la texture ne reste compressée sur le GPU que si celui-ci accepte le format visé (ASTC ou ETC2 sur téléphone, BC sur PC), ce que WebGPU expose comme des options facultatives. Et le décodeur Basis a un poids que Gemini ne compte pas. À mesurer avant de l'adopter.
- **Le langage** : `material: wood` et `finish: polished` répètent ce que le fichier du modèle sait déjà, et `weight: 250KB` est de la technique, ce que Gemini dit lui-même vouloir cacher à l'auteur. Pour le premier essai, un seul mot devrait suffire (l'objet et sa source) ; le budget relève de l'outil de préparation.
- **Les paliers « plus tard »** : d'accord pour ne remplir qu'un palier pendant l'essai de la chaise, mais la place du palier doit exister dès le premier jour, sinon on retombe dans le piège que Gemini décrit lui-même.
- **Les modules compilés à part (Cargo features)** : un seul moteur est servi à tous. Exclure une fonction à la compilation veut dire servir deux moteurs, ou charger une partie du moteur à la demande, comme le fait déjà `page-moteur.js` (`ADR-033`). À décider avec Codex.
- Les budgets de la chaise (1 500 / 5 000–8 000 / 25 000 triangles) sont un bon point de départ, à prendre comme les paramètres de l'essai, pas comme des vérités.
