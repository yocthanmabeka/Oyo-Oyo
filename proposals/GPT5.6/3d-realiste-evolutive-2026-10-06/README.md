# Proposition GPT-5.6 — 3D réaliste, progressive et remplaçable sans réécriture

- Auteur : GPT-5.6, pour Yocthan
- Date : 2026-10-06
- Statut de ce document : **PROPOSITION / EXPLORATION**.
- Aucun statut d'ADR n'est modifié.
- Base lue : `main` au commit `ec0c6cd739f8339b578d11cadbad5ce796465b4b`.
- Documents relus : revue GPT-5.6 du 2026-10-04, proposition Claude sur les modèles 3D, réponse Gemini, `rendu.rs`, `rendu.wgsl`, `mosaique.rs`, `moteur/README.md`, ADR-005, ADR-010, ADR-011, ADR-037, ADR-045.

## Réponse courte

**Oui, cette manière de penser est bonne.** Il faut commencer avec un chemin de rendu très simple sur téléphone, mais poser dès le premier jour les invariants qui permettent de monter en qualité sans changer le langage ni casser le repli WebGL 2.

Le point critique est le suivant : les paliers léger, normal et haut ne doivent pas être trois moteurs. Ils doivent être **trois réglages du même moteur**, alimentés par **plusieurs représentations du même objet**.

La 3D proche doit quitter le rendu purement en points. Les points restent excellents pour le lointain, les transitions, le morcellement et l'identité holoscénique, mais un objet opaque réaliste a besoin de surfaces, d'un tampon de profondeur, de normales, de matériaux et d'une lumière.

---

# 0. Ce que cette revue a réellement vérifié, et ce qu'elle n'a pas vérifié

## Vérifié par lecture directe du code actuel

- `rendu.rs` crée un seul pipeline de points.
- Ce pipeline utilise déjà `TriangleList` : chaque point est un carré de deux triangles.
- `rendu.wgsl` place tous les sommets à `z = 0`.
- Le pipeline actuel a `depth_stencil: None`.
- Le mélange est additif.
- Le shader ne possède ni normale, ni UV, ni matériau, ni lumière physique.
- `Rendu::dessiner` construit un `Vec<Instance>` à chaque image et écrit les instances dans un buffer.
- Le chemin WebGPU et le repli WebGL 2 partagent déjà `wgpu`.
- Le repli WebGL 2 demande les limites `downlevel_webgl2_defaults()`.
- Le moteur ne demande aujourd'hui aucune feature GPU facultative.
- `mosaique.rs` calcule un relief et projette des points, mais ne crée pas une surface opaque.
- `POINTS_MAX` et `INSTANCES_MAX` sont des garde-fous actuels, pas des preuves de budget 3D réaliste.
- ADR-045 enferme les modules WASM dans un worker, sans réseau, sans page, sans stockage et avec temps/mémoire plafonnés.
- ADR-037 impose PascalCase pour les blocs/noms de blocs et camelCase pour valeurs/réglages, avec CSS conservé dans les styles.

## Vérifié par les mesures déjà présentes dans le dépôt

Je n'invente pas un nouveau lancement que je n'ai pas pu reproduire dans cette session. L'environnement qui me donne accès au dépôt ne me permet pas de cloner puis exécuter GitHub localement : la tentative d'accès réseau du runtime local n'a pas pu résoudre `github.com`.

Les seules exécutions que je considère comme établies ici sont donc celles déjà enregistrées dans le dépôt :

- Flip 5 : environ 60 FPS sur le Big Bang et le zoom continu.
- Flip 3 : environ 60 FPS en WebGPU.
- Flip 3 : environ 60 FPS aussi dans le chemin WebGL 2 actuel.
- Vue points du Flip 3 : environ 99 Mo PSS / 241 Mo RSS dans la mesure publiée.
- Vue points du Flip 5 : environ 88 Mo PSS / 179 Mo RSS.
- Le moteur compressé reste autour de 500 Ko dans les mesures récentes du README.

Ces résultats prouvent le socle actuel. Ils **ne prouvent pas** le coût d'un pipeline opaque PBR, des textures, des ombres, d'un format `.holo3d` ou d'une scène contenant plusieurs modèles.

Tout chiffre nouveau plus bas est donc explicitement marqué **estimation de départ à mesurer**.

---

# 1. Croître sans réécrire : découpe du rendu

## Verdict

Je ne transformerais pas immédiatement `rendu.rs` en système de plugins abstrait très général. Ce serait une abstraction avant le besoin.

Je le découperais en **noyau + passes explicites**.

Le noyau garde :

- `Surface`
- `Device`
- `Queue`
- `SurfaceConfiguration`
- gestion du redimensionnement
- création d'une image
- encodeur de commandes
- présentation
- mesures
- détection des capacités du GPU
- ressources communes de la frame

Les passes deviennent des modules séparés.

## Passes proposées

Ordre logique, pas obligation de tout construire maintenant :

1. **PreparePass** — côté CPU, avant le GPU  
   visibilité, choix de LOD, choix des textures/mips, budgets, uploads prêts.

2. **ShadowPass** — facultatif plus tard  
   dessine seulement la profondeur depuis une lumière.

3. **OpaquePass** — à construire en premier  
   maillages opaques, profondeur, matériau simple, lumière.

4. **PointsPass** — le rendu actuel adapté  
   points du lointain, transitions, halos, morcellement.

5. **TransparentPass** — plus tard  
   verre, particules, splats éventuels, transparences triées.

6. **SkyPass** — simple au début  
   fond/ciel ou environnement IBL.

7. **PostProcessPass** — facultatif et croissant  
   tone mapping si rendu intermédiaire HDR, FXAA/TAA, bloom très limité, etc.

8. **PresentPass** — sortie vers la surface  
   conversion finale vers l'espace de sortie.

Dans le code Rust, je privilégierais d'abord une structure du genre :

- `RendererCore`
- `FrameResources`
- `OpaquePass`
- `PointsPass`
- `PostProcessPass`

plutôt qu'un `Vec<Box<dyn RenderPass>>` totalement dynamique. Avec `wgpu`, les durées de vie des render passes et des ressources rendent vite un système de traits très abstrait plus compliqué que le problème qu'il résout.

On pourra généraliser après deux ou trois vraies passes.

## Ce qui doit exister dès le premier jour

### A. Tampon de profondeur

**Obligatoire maintenant.**

Je recommande un depth target commun recréé avec la surface au redimensionnement.

Format de départ : `Depth24Plus`, sauf mesure montrant un problème spécifique de compatibilité.

Les objets opaques écrivent la profondeur. Les points peuvent ensuite choisir :

- profondeur activée pour représenter un objet solide ;
- profondeur lue sans écriture pour certains halos ;
- profondeur ignorée pour un effet volontairement cosmique.

Ne pas mélanger ces comportements dans un seul pipeline additif.

### B. Unités du monde

**Oui : 1 unité Holo = 1 mètre. Dès maintenant.**

C'est important pour :

- caméra ;
- lumière ;
- vitesse ;
- physique future ;
- audio spatial ;
- proximité ;
- multijoueur ;
- conversion glTF ;
- LOD fondé sur une erreur géométrique réelle.

Je recommande aussi de figer l'orientation du monde dès le premier modèle et de la documenter.

Pour réduire les conversions, il est rationnel de rester proche des conventions de glTF : système cohérent, axe vertical unique, données converties une seule fois par l'outil PC.

### C. Espace de couleur linéaire

**Obligatoire dès le premier matériau.**

Règle :

- les couleurs de base venant d'images sRGB sont décodées en linéaire ;
- les calculs de lumière se font en linéaire ;
- roughness, metallic, AO, normales restent des données non colorimétriques ;
- le résultat est transformé avant affichage.

Si ce contrat est faux au début, tous les matériaux devront être retouchés plus tard.

### D. Tone mapping

Je veux le contrat dès le jour 1, mais pas forcément une grosse passe HDR.

Sur le chemin minimal, le shader peut calculer en linéaire et appliquer une courbe simple avant la sortie sRGB.

Sur les appareils capables, une future cible intermédiaire HDR (`Rgba16Float` par exemple) permettra une vraie PostProcessPass.

**Ne pas rendre la cible HDR obligatoire pour WebGL 2.** Certaines capacités de rendu flottant sont moins universelles sur le chemin de secours.

### E. Caméra et matrices communes

Le pipeline doit posséder très tôt :

- modèle ;
- vue ;
- projection ;
- position caméra ;
- plan proche/lointain raisonnables.

Ce sont des données communes, pas des détails propres au shader des points.

### F. Convention de matériaux

Même si V1 n'affiche que couleur + roughness + metallic, le moteur doit déjà traiter un matériau comme une ressource séparée d'un mesh.

Ne jamais coder « le bois » directement dans le mesh.

---

# 2. Les paliers de qualité

## Ne pas choisir seulement avec le nom du téléphone

Le moteur ne doit pas avoir une table du genre « Flip 3 = Normal ».

Elle vieillit immédiatement.

Le choix doit combiner :

1. **capacités** ;
2. **petit étalonnage réel** ;
3. **mesure continue**.

## Mesures disponibles au démarrage

Le moteur peut regarder :

- backend WebGPU ou WebGL 2 ;
- limites `wgpu` de l'adapter ;
- taille maximale des textures ;
- fonctionnalités de compression disponibles ;
- résolution réelle de la zone de dessin ;
- densité d'écran ;
- mémoire déjà consommée, quand elle peut être observée ;
- temps réel des premières frames.

Le navigateur peut masquer certaines informations matérielles pour la vie privée. Il ne faut donc jamais faire dépendre la correction du moteur d'un nom de GPU précis.

## Choix initial proposé

- WebGL 2 : commencer **Léger**, puis monter seulement si les mesures sont excellentes.
- WebGPU sans signal particulier : commencer **Normal**.
- PC clairement à l'aise après calibration : **Haut**.

C'est volontairement conservateur.

## Boucle adaptative

Cible : 60 FPS, soit environ 16,7 ms par frame.

Je recommande de regarder une fenêtre glissante plutôt que la dernière frame.

Exemple de politique à tester :

- si le p95 dépasse environ 20 ms pendant 2 à 3 secondes : baisser une étape ;
- si des frames > 50 ms apparaissent pendant un changement de LOD : considérer le changement comme raté ;
- ne jamais remonter avant au moins 10 secondes stables avec une vraie marge ;
- ne monter ou descendre qu'un levier à la fois ;
- si le palier Léger ne tient pas 60, viser une cadence **stable** de 30 plutôt qu'un 45–60 instable.

Ce sont des **paramètres d'expérience**, pas des valeurs acceptées.

## Contenu des trois paliers

| Élément | Léger | Normal | Haut |
|---|---|---|---|
| Résolution interne | ~65–80 % | ~85–100 % | 100 %, haute densité si marge |
| Mesh | LOD grossier | LOD moyen | LOD fin |
| Textures | 512 px typique | 1024 px typique | 2048 px typique |
| Compression | KTX2 obligatoire | KTX2 obligatoire | KTX2, qualité supérieure |
| Matériau | metallic/roughness simple | + normal map si marge | matériau complet autorisé |
| IBL | faible résolution | moyen | haute résolution |
| Lumières directes | 1 principale | 1 principale + quelques locales bornées | davantage, mais borné |
| Ombres | aucune ou ombre très simple | 1 shadow map si budget | meilleure résolution / plusieurs limitées |
| AO | baked | baked | SSAO facultatif |
| AA | FXAA ou rien | FXAA / MSAA 2x si mesuré | TAA/MSAA si disponible et utile |
| Post-traitement | quasi nul | tone mapping + AA | effets facultatifs |

## Descendre sans « saut »

Quatre règles :

1. **Hystérésis** : le seuil pour passer LOD0 → LOD1 n'est pas le même que LOD1 → LOD0.
2. **Préchargement** : ne jamais changer de LOD avant que le nouveau soit prêt.
3. **Même pivot, mêmes bounds, mêmes unités** pour tous les LOD.
4. Si la différence est visible, faire un **cross-fade court** ou une transition tramée sur 100–250 ms.

Le cross-fade dessine temporairement deux LOD : il ne doit donc être appliqué qu'à quelques objets à la fois.

Je recommande aussi de baisser d'abord la **résolution interne**, les ombres et les détails de texture avant de remplacer brutalement les formes proches. C'est souvent moins perceptible.

---

# 3. Format préparé `.holo3d` V1

## Principe

Le fichier doit contenir des **données**, jamais du code.

L'outil PC importe glTF, vérifie, simplifie, compresse et produit un conteneur que le téléphone peut lire avec peu d'allocations et peu de branches.

## V1 doit déjà être versionnée

Je recommande un petit conteneur binaire chunké :

- magic : `HOLO3D`
- version majeure
- version mineure
- taille totale annoncée
- table des sections
- type de section
- taille de section
- flags
- éventuellement hash/CRC par section

Règle de compatibilité :

- même version majeure : un lecteur ancien peut ignorer une section **optionnelle** inconnue ;
- section marquée **required** inconnue : refus explicite ;
- changement incompatible : nouvelle version majeure.

Cela permet d'ajouter plus tard lightmaps, animations, splats ou nouveaux matériaux sans casser les V1 simples.

## Sections minimales

### META

- unité : mètre ;
- système d'axes/version ;
- boîte englobante AABB ;
- sphère englobante ;
- pivot ;
- taille réelle ;
- version de l'outil convertisseur ;
- budget estimé après décodage.

### LODS

Pour chaque niveau :

- erreur géométrique estimée en mètres ;
- bounds ;
- références des chunks mesh ;
- références textures ;
- éventuel aperçu en points ;
- mémoire estimée.

Je préfère **erreur géométrique** à « LOD à 12 mètres » : le moteur peut convertir l'erreur en pixels selon la caméra.

### MESH

- positions ;
- normales ;
- UV0 ;
- tangentes seulement si nécessaires ;
- UV1 optionnelles pour futures lightmaps ;
- indices ;
- sous-maillages/material slots ;
- données quantifiées/compressées.

### MATERIAL

V1 :

- base color ;
- metallic ;
- roughness ;
- emissive simple ;
- référence texture ;
- normal map optionnelle ;
- AO optionnelle.

Les champs futurs passent par extensions/chunks, pas par changement du mesh de base.

### TEXTURES

KTX2 avec mipmaps.

Le format doit permettre soit :

- textures incluses ;
- soit références vers blobs séparés pour le streaming/cache.

### POINT_PREVIEW

Optionnel mais important pour Holoverse :

- représentation éloignée en points ;
- bounds ;
- densité ;
- identifiants visuels stables si nécessaire.

**Attention : un point visuel d'un mesh n'est pas automatiquement un `Point` sémantique contenant un monde.** Il faut garder la sémantique Holoverse séparée de la tessellation graphique.

### BAKE

Pas obligatoire en V1, mais prévoir son type de chunk dès la conception :

- AO baked ;
- lightmap ;
- probes ;
- données IBL préfiltrées éventuelles.

## Garde-fous du décodeur

Ne pas faire confiance aux tailles écrites dans le fichier.

Avant toute allocation :

- vérifier les multiplications ;
- vérifier les offsets ;
- vérifier les indices ;
- borner les nombres ;
- recalculer la mémoire réelle attendue ;
- refuser avant l'allocation si le budget est dépassé.

### Plafonds de sécurité proposés pour V1

Ce sont des **bornes de parser**, pas des objectifs visuels :

- 8 LOD au plus ;
- 64 matériaux par asset ;
- textures max 4096 × 4096 sur mobile ;
- sommets : 500 000 au plus par asset source préparé ;
- indices : 1 500 000 au plus ;
- chunk individuel : 32 MiB max sur mobile ;
- asset téléchargé : 32 MiB max sur mobile avant refus ;
- mémoire décodée/résidente annoncée : 128 MiB max par asset mobile avant refus.

Pour le PC, les plafonds peuvent être plus hauts.

Mais le **budget de scène** doit être bien inférieur aux plafonds de fichier.

---

# 4. Extensions et ADR-045

## Un module enfermé ADR-045 ne doit pas devenir une extension GPU

Je garderais la frontière actuelle.

Un module WASM non fiable peut :

- calculer ;
- simuler ;
- transformer un tableau de données ;
- produire des positions de particules ;
- générer une géométrie bornée ;
- calculer une animation.

Il ne doit pas recevoir :

- `Device` ;
- `Queue` ;
- une texture GPU ;
- un pipeline ;
- un shader arbitraire ;
- un handle WebGPU/WebGL.

Sinon on ouvre exactement la ressource la plus facile à faire planter : le GPU.

## Deux familles d'extension

### A. Modules enfermés de page

Non fiables.

Ils produisent des **données bornées**.

Quand ADR-045 acceptera des listes/buffers, le contrat pourrait devenir :

entrée bornée → calcul → sortie bornée.

Le moteur valide ensuite la sortie avant de la dessiner.

### B. Extensions de rendu du moteur

Fiables.

Exemples :

- ciel atmosphérique avancé ;
- renderer de Gaussian splats PC ;
- décodeur spécialisé ;
- SSAO ;
- backend volumétrique futur.

Elles doivent être :

- du code maintenu/audité par le moteur ;
- compilées comme feature ou composant officiel ;
- activées seulement si l'appareil le permet ;
- chargées avec un fallback connu.

Sur le web actuel, je préfère au début des **features du moteur** à un ABI de plugin GPU dynamique. C'est moins élégant, mais beaucoup plus sûr et plus simple avec `wgpu`/WASM.

Plus tard, le navigateur propre à Holoverse pourra avoir une vraie API de plugin moteur.

## Gaussian splats

Pas dans V1.

Mais je ne les inscrirais pas « interdits ».

Un futur asset peut contenir :

- mesh ;
- points ;
- splats.

Sur PC, un renderer officiel peut choisir les splats.

Sur mobile ou WebGL 2, il reprend le mesh ou les points.

Le langage HoloCode n'a pas à changer.

## Risques à borner

- OOM GPU ;
- device lost ;
- shader très long à compiler ;
- shader pathologique ;
- nombre de passes ;
- nombre de draw calls ;
- quantité de storage buffers ;
- taille des textures ;
- compute workloads ;
- sortie d'un module WASM ;
- temps de création de pipelines ;
- pics de mémoire pendant un changement de LOD.

Règle forte : **aucune extension ne peut supprimer le chemin de repli de l'objet.**

---

# 5. Le langage : le moins de mots possible

## Principe

Le langage décrit d'abord **l'intention**.

Le moteur/cache/outil décident :

- triangle count ;
- LOD ;
- mip ;
- KTX2 ;
- format GPU ;
- shadow map ;
- MSAA ;
- SSAO ;
- tone mapping ;
- shader ;
- backend ;
- taille du buffer ;
- draw calls.

Je n'exposerais aucun de ces mots à l'auteur.

## Proposition minimale

```holo
Material(
  name: WaxedWood,
  kind: wood,
  finish: waxed,
)

Model(
  source: "chair.holo3d",
  material: WaxedWood,
)

Light(
  kind: sun,
  mood: sunset,
)
```

Cela respecte ADR-037 :

- `Material`, `Model`, `Light`, `WaxedWood` : PascalCase ;
- `source`, `material`, `kind`, `finish`, `mood` : camelCase/minuscules ;
- `wood`, `waxed`, `sun`, `sunset` : mots-valeurs minuscules.

## Faut-il exposer roughness/metallic ?

Pas par défaut.

Je propose deux niveaux d'écriture :

### Simple

```holo
Material(name: WaxedWood, kind: wood, finish: waxed)
```

### Précis, seulement si un besoin réel apparaît

```holo
Material(
  name: CustomMetal,
  color: #7a7a7a,
  roughness: 0.2,
  metallic: 1,
)
```

Le premier est pour les non-programmeurs.

Le second est une échappatoire artistique, pas l'interface principale.

## Ce qui doit rester caché

Toujours cachés :

- normales/tangentes ;
- espace tangent ;
- mipmaps ;
- texture compression ;
- tessellation ;
- frustum culling ;
- occlusion culling ;
- LOD index ;
- shadow cascade ;
- sample count ;
- render target ;
- texture format ;
- gamma/sRGB ;
- bind groups ;
- pipeline ;
- shader ;
- WebGPU/WebGL 2.

---

# 6. Premier essai : la chaise

## Modèle de référence

Je recommande **Rockingchair 01 de Poly Haven** :

- source : https://polyhaven.com/a/Rockingchair_01
- licence : CC0 ;
- environ 12 000 triangles selon la fiche Poly Haven ;
- plusieurs barreaux fins et courbes : bon test d'aliasing et de simplification ;
- bois usé : bon test de roughness, normal map, lumière et texture.

Poly Haven publie ses assets en CC0 : usage commercial permis, attribution non obligatoire.

Pourquoi cette chaise plutôt qu'un cube ou une chaise trop simple : ses éléments fins feront immédiatement apparaître les mauvais LOD, les problèmes de profondeur et les scintillements.

## Trois versions préparées de départ

### Léger — estimation

- 1 500 à 2 000 triangles ;
- texture principale 512² ;
- KTX2 ;
- AO baked ;
- normal map supprimée si le test montre qu'elle coûte trop ;
- objectif mémoire GPU des textures de la chaise : ≤ 2 MiB ;
- objectif téléchargement total de l'asset : ≤ 300 KiB.

### Normal — estimation

- 5 000 à 7 000 triangles ;
- textures 1024² ;
- KTX2 ;
- base color + metallic/roughness + normal ;
- objectif mémoire GPU textures : ≤ 6 MiB ;
- objectif téléchargement : ≤ 1 MiB.

### Haut — estimation

- source autour de 12 000 triangles ;
- textures 2048² ;
- KTX2 haute qualité ;
- objectif mémoire GPU textures : ≤ 16 MiB ;
- objectif téléchargement : ≤ 4 MiB.

Ces nombres sont des **cibles de premier test**, pas des limites du futur format.

## Objets dessinés

Premier test fonctionnel : **1 chaise**.

Puis stress test avec la même chaise instanciée :

- Léger : 16, puis 32, puis 64 visibles ;
- Normal : 32, puis 64, puis 128 ;
- Haut PC : 128, puis 256, puis 512.

Le but n'est pas de promettre ces nombres. Le but est de trouver le coude où le frame time se dégrade.

Les copies doivent partager mesh et textures, sinon ce test mesurerait surtout le chargement mémoire.

## Budget mémoire de l'onglet — cibles de départ

À partir des mesures actuelles du moteur seul :

- Léger : viser ≤ **160 MiB PSS** après stabilisation ;
- Normal / Flip 3 : viser ≤ **220 MiB PSS** ;
- abandonner une configuration mobile si elle dépasse régulièrement **300 MiB PSS** sur l'appareil de test ou continue à croître.

Ce sont des **objectifs prudents**, pas des limites Chrome universelles.

Le RSS est utile en diagnostic, mais PSS est plus parlant pour comparer le coût propre du processus.

## Protocole Flip 3

Pour chaque configuration :

1. chargement froid ;
2. chargement chaud ;
3. WebGPU ;
4. WebGL 2 avec `?webgl` ;
5. caméra face ;
6. caméra dos ;
7. rotation continue ;
8. distance proche ;
9. distance moyenne ;
10. distance lointaine avec passage mesh → points ;
11. aller-retour répété entre deux LOD pendant 2 minutes ;
12. rotation/zoom continu pendant 15 minutes.

Mesurer :

- FPS moyen ;
- p95 du temps de frame ;
- pire frame ;
- nombre de frames > 33 ms ;
- nombre de frames > 50 ms ;
- temps première image ;
- PSS/RSS avec ADB ;
- mémoire JS pour information seulement ;
- batterie départ/arrivée ;
- température/thermal state par outils Android si disponible ;
- device lost / erreurs GPU ;
- poids téléchargé ;
- temps de transcoding KTX2 ;
- pic mémoire pendant changement de LOD.

## Téléphone modeste

Il faut au moins un téléphone :

- 4 Go de RAM ou moins ;
- SoC milieu/entrée de gamme récent ;
- Chrome à jour ;
- idéalement WebGPU disponible, sinon WebGL 2 est déjà un cas utile.

Ne pas choisir le palier via la marque. Noter le modèle seulement pour reproduire le test.

---

# 7. Limites dures du navigateur mobile en 2026

## Ce qu'on ne peut pas promettre

### 1. 60 FPS sur « n'importe quel téléphone » avec le même réalisme

Impossible à garantir.

La seule promesse réaliste est : **même contenu logique, qualité graphique adaptée**.

### 2. Ray tracing temps réel comme base portable

À ne pas prendre comme dépendance de V1.

Même quand du matériel mobile sait accélérer le ray tracing nativement, le chemin web portable et le coût thermique ne permettent pas d'en faire une exigence.

Le format et le langage ne doivent toutefois pas l'interdire pour un futur palier.

### 3. Accès fiable à la température depuis une page web

Les API thermiques Android sont surtout accessibles au monde natif. Le navigateur ne donne pas une mesure thermique universelle fiable.

Dans le web, le moteur doit surtout déduire la pression à partir du frame time, des pertes de device et du comportement soutenu.

### 4. Contrôle de la politique mémoire du navigateur

Le moteur peut réduire sa mémoire.

Il ne peut pas empêcher Android/Chrome de tuer ou suspendre l'onglet sous pression.

Il n'existe pas une frontière universelle du type « à X Mo Chrome tue toujours l'onglet ».

### 5. Exécution continue en arrière-plan

Un onglet caché peut être throttlé ou suspendu.

Le monde réseau ne doit jamais dépendre du fait qu'un téléphone continue son rendu en arrière-plan.

### 6. Même ensemble de features GPU partout

WebGPU et WebGL 2 n'ont pas les mêmes capacités.

Même entre deux implémentations WebGPU, les formats compressés, limites, timestamps et capacités avancées peuvent différer.

Le chemin de base doit rester volontairement bas.

### 7. Plugins GPU arbitraires non fiables

Le sandbox du navigateur protège le système, mais un shader ou workload GPU hostile peut toujours provoquer consommation extrême, perte du device ou fermeture de l'onglet.

HoloCode ne doit pas donner WebGPU directement aux modules d'ADR-045.

---

# 8. Sources techniques externes vérifiées pour cette proposition

- Khronos KTX 2.0 : les textures Basis Universal peuvent être transcodées vers les formats GPU natifs et réduire transmission/mémoire : https://www.khronos.org/ktx/
- Khronos glTF : PBR, metallic/roughness, normal, AO et extensions de matériaux : https://www.khronos.org/gltf/
- Spécification glTF 2.0 : versionnement, compatibilité et extensions : https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.pdf
- Android Developers, thermique/ADPF : les appareils throttlent et les réglages de qualité dynamiques doivent être testés sur plusieurs classes d'appareils : https://developer.android.com/games/optimize/adpf/thermal
- Poly Haven, licence : tous les assets sont CC0 : https://polyhaven.com/license
- Chaise de référence : https://polyhaven.com/a/Rockingchair_01

---

# 9. Ordre de travail recommandé

Du plus important au moins important :

1. **Ajouter un vrai depth buffer et séparer `OpaquePass` de `PointsPass`.**
2. **Figer les conventions : mètres, axes, caméra, linéaire/sRGB.**
3. **Dessiner un mesh opaque non texturé avec profondeur sur WebGPU et WebGL 2.**
4. **Créer le convertisseur PC glTF → `.holo3d` V1 versionné et borné.**
5. **Ajouter le matériau PBR minimal : base color + roughness + metallic.**
6. **Ajouter KTX2 + mipmaps, avec estimation mémoire avant allocation.**
7. **Ajouter IBL simple + tone mapping.**
8. **Préparer trois LOD de la chaise et le point-preview lointain.**
9. **Mesurer Flip 3 WebGPU puis WebGL 2 avec 1 chaise.**
10. **Ajouter la sélection automatique de palier par capacités + frame time.**
11. **Ajouter changements de LOD sans saut : préchargement + hystérésis + transition courte.**
12. **Faire le test soutenu 15 minutes et mesurer mémoire/batterie/chauffe.**
13. **Tester un téléphone modeste.**
14. **Seulement après : ombre dynamique simple.**
15. **Ensuite : plusieurs objets et budgets globaux de scène.**
16. **Plus tard : SSAO, TAA, lumières avancées, renderer splats PC, extensions de rendu officielles.**

---

# Réponse finale en une phrase

**Oui, c'est possible ainsi et je ne recommande pas de revoir la vision ; je recommande seulement de faire dès la première chaise un seul pipeline versionné, avec profondeur, unités, couleur linéaire, format préparé et qualité adaptative, afin que tout le réalisme futur s'ajoute par passes et données sans réécrire HoloCode.**
