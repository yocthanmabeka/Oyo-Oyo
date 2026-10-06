# Veille du 2026-10-06 : la 3D réaliste dans le navigateur et sur téléphone

Demandée par Yocthan le 2026-10-06 : « vérifier ce que pensent les humains en 2026, exactement le 6 octobre », avant de prendre les décisions de la 3D (`ADR-048`, `ADR-049`). Recherche faite par Claude sur Internet le jour même.

**Limites, à lire d'abord.** Reddit, polyhaven.com, caniuse, web3dsurvey et une partie des forums (Hacker News, three.js, Babylon) n'ont pas pu être lus directement ; ce qui en vient est tiré d'extraits de recherche, paraphrasé, jamais cité mot pour mot. Peu de discussions humaines datées d'août à octobre 2026 ont été trouvées. Légende : **[V]** vérifié sur une source primaire ; **[S]** vu dans un extrait ou un site secondaire ; **[NV]** non vérifié.

## 1. Les technologies, au 6 octobre 2026

### WebGPU

- **Chrome** : actif sur Android depuis la v121 (ARM, Qualcomm, Intel), v139 pour Imagination ; Samsung Xclipse encore en cours. [V] https://github.com/gpuweb/gpuweb/wiki/Implementation-Status (modifiée le 2026-10-02)
- **Safari 26** : actif par défaut sur iPhone, iPad et Mac. [V] même source
- **Firefox** : Windows et Mac oui ; **Android encore derrière un réglage caché**, prévu en 2026. [V] même source ; notes de Firefox 147 (2026-01-13)
- **Chrome 146** : un « mode compatibilité » de WebGPU pour les appareils plus anciens (OpenGL ES 3.1), en commençant par Android. [V] https://developer.chrome.com/blog/new-in-webgpu-146 (2026-02-25)
- Part des appareils avec WebGPU : environ 81 % (Android 73 %, iOS 82 %). [S] web3dsurvey.com, sans date
- Les textures compressées sont des options : BC sur PC, ETC2 et ASTC sur téléphone ; il faut transcoder selon l'appareil. [NV] d'après la spécification

### Moteurs (dernières versions)

- three.js r186.1 (2026-09-24) [V] ; Babylon.js 9.29 (2026-10-01), la 9.0 a apporté splats et OpenPBR [V] ; PlayCanvas 2.23 (2026-10-01) [V] ; Bevy 0.19.1, 0.20 en préparation [V].
- **wgpu 30.0.1** (le nôtre) : espace de couleur de la surface (HDR) au choix ; lancer de rayons expérimental, en natif seulement. [V] crates.io, https://wgpu.rs
- Unreal 5.8 (2026-06-17) : « Lumen Lite » ; Lumen sur téléphone reste expérimental, Android haut de gamme seulement. [S] et [V] https://dev.epicgames.com/documentation/unreal-engine/using-lumen-global-illumination-on-mobile-in-unreal-engine
- Unity 6.6 : WebGPU supporté, mais **WebGL 2 reste le choix par défaut**. [S]

### Formats et techniques

- **Gaussian splats** : l'extension glTF **`KHR_gaussian_splatting` est ratifiée** par Khronos. [V] https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_gaussian_splatting/README.md ; compression SPZ et SOG (15 à 20 fois plus petit que PLY). [S] Aucun chiffre fiable sur téléphone trouvé.
- **glTF 2.1** annoncé le 2026-06-11 (scènes en plusieurs fichiers, livraison progressive). [S] https://www.khronos.org/news/permalink/introducing-gltf-2.1-with-complex-scenes
- **KTX2 / Basis Universal 2.0** ; le décodeur pèse environ 500 Ko non compressé. [S]
- **meshoptimizer 1.3** (2026-09-25) [V] ; `KHR_meshopt_compression` en candidate [V]. Avis courant : meshopt décode plus vite que Draco, avec un décodeur plus petit. [NV]
- **Rendu des couleurs** : model-viewer utilise par défaut **Khronos PBR Neutral** [V] ; Bevy, TonyMcMapface ; three.js, aucun par défaut ; AgX et ACES disponibles partout. [S]
- **Lancer de rayons sur le web** : aucune proposition officielle trouvée. [NV] **Géométrie virtuelle (type Nanite)** : impossible sur le web aujourd'hui (pas d'entiers atomiques 64 bits dans WebGPU). [S]

## 2. Ce que disent les humains

1. **Le réalisme sur téléphone est faisable, mais la mémoire et la chauffe commandent.** Sous iOS 26, des pages qui plantent vers 100 Mo sur iPhone SE et 200 Mo sur iPad (blog lapcatsoftware, 2026-01-07) ; des applis three.js tuées en changeant de gros modèle (forum three.js, fin 2025) ; une scène à 60 images par seconde sur Mac tombe à 30 sur un Android de deux ans, puis à 20 après cinq minutes de chauffe (blog, 2026). [S]
2. **WebGPU en production : tout le monde garde WebGL 2.** Sur certains Android, WebGL 2 est plus rapide que WebGPU (Pixel 5, forum Babylon, 2026) ; WebGPU cassé sur Pixel 10, PlayCanvas repasse en WebGL 2 (issue 8874, ~juin 2026) ; des « Device Lost » sur téléphone sous charge (forum Babylon, 2026). Le repli WebGL 2 de wgpu n'a pas de calcul général (compute). [S]
3. **Moteur maison ou moteur existant** : three.js domine ; un moteur maison se justifie pour le contrôle, au prix des contournements de pilotes et du poids du WebAssembly (Bevy sur le web : 22 Mo pour une démo, HN 2024). [S], discussions de 2026 rares.
4. **Gaussian splats** : impressionnants pour des scènes capturées, mais lourds, sans possibilité de changer la lumière, fragiles sur téléphone. [S]
5. **Qualité adaptative** : pratique standard (three.js et drei : `PerformanceMonitor`, `AdaptiveDpr` ; Babylon : `setHardwareScalingLevel`). **Ne jamais dessiner à la densité native de l'écran (3x)** ; ajuster sur une fenêtre glissante du temps d'image. Personne ne le conteste. [S]
6. **Le mot « métavers »** : largement perçu comme mort ou moqué depuis la fermeture d'Horizon Worlds en réalité virtuelle (15 juin 2026 ; la version téléphone et web continue) et les coupes chez Meta Reality Labs (CNBC, 2026-01-24). Ce qui survit : les mondes 3D sociaux, accessibles sur téléphone et dans le navigateur. [S]

## 3. Ce que cela change pour nos décisions

- **Confirmé** : préparer à l'avance, KTX2, lumière précalculée, paliers pilotés par le temps d'image, repli WebGL 2, splats plus tard et sur PC seulement. Les praticiens font exactement cela.
- **À ajouter** :
  1. **Le chemin de base n'utilise pas de calcul général (compute)**, pour rester identique en WebGL 2 et sur les vieux pilotes.
  2. **Le moteur ne prend pas « WebGPU si disponible » d'office** : il mesure, et peut garder WebGL 2 sur un appareil où WebGPU est plus lent ou instable.
  3. **La densité de l'écran est plafonnée** : premier levier de la qualité adaptative (le moteur plafonne déjà à 2).
  4. **Les budgets mémoire valent aussi pour l'iPhone**, où les plantages commencent vers 100 à 200 Mo : le palier léger vise moins de 160 Mo, et l'iPhone entre dans les mesures.
  5. **Rendu des couleurs par défaut : Khronos PBR Neutral**, conçu pour montrer fidèlement les matériaux ; AgX en option.
  6. **Géométrie compressée avec meshoptimizer**, pas Draco.
  7. Le décodeur KTX2 pèse environ 500 Ko : à mesurer, et à charger seulement quand une page a de la 3D.
- **Pour la communication** (pas une décision technique) : le mot « métavers » fait fuir en 2026 ; « mondes 3D » ou « Holoverse » passent mieux.
