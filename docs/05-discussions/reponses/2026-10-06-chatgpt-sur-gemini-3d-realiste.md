# Avis de ChatGPT sur la réponse de Gemini — la 3D réaliste (2026-10-06)

Collé par Yocthan le 2026-10-06, avec la réponse de Codex (PR 118, `proposals/GPT5.6/3d-realiste-evolutive-2026-10-06/`). ChatGPT y relit la [réponse de Gemini](2026-10-06-gemini-3d-realiste.md). Résumé fidèle, puis la lecture de Claude ; le texte complet est dans la conversation du 2026-10-06.

## Ce que dit ChatGPT

- **Direction : 8/10 ; certains chiffres : 6/10.** Le cœur à garder : précalcul, compression, plusieurs niveaux de détail, un téléphone qui ne fait que des choses simples, un moteur qui s'adapte, aucune plomberie graphique dans HoloCode.
- **Confirmé avec sources** : KTX2 / Basis (Khronos : textures 6 à 8 fois plus petites en mémoire que des JPEG décompressés) ; le modèle de matériaux de glTF, à ne pas réinventer ; Poly Haven en CC0.
- **Corrigé** :
  - Fortnite : Nanite reste absent des téléphones, mais Lumen a un support expérimental sur certains Android haut de gamme. Holoverse ne doit pas en dépendre.
  - Le lancer de rayons n'est pas « impossible » : il ne doit pas être une **dépendance**. Un futur palier pourra s'en servir sans changer le `.holo`.
  - Les **350–450 Mo** par onglet : aucune source Chrome ; à retirer. Fixer nos propres budgets, plus bas : 150–200 Mo pour le palier léger, puis mesurer.
  - La chauffe : le phénomène est réel (Google le documente), les chiffres universels non (un exemple de Kakao : 16 puis 22 minutes). D'où l'idée principale : **les paliers sont des états qui bougent pendant la visite**, pas trois profils choisis au départ (temps d'image, mémoire, batterie, saccades).
  - Le rendu des couleurs ne fait pas une photo d'une mauvaise chaise : le réalisme vient de l'ensemble.
  - « 2 Mo de JPEG = 16 Mo » est un cas (2048 × 2048), pas une règle.
  - Les budgets de la chaise : des cibles d'essai, pas un contrat du format ; 1 500 triangles est très agressif pour un objet réaliste vu de près.
  - Les **Gaussian splats** : pas maintenant, mais ne pas les interdire ; Khronos a ouvert en 2026 un groupe sur les médias volumétriques.
- **Plus profond** : `weight: 250KB` ne doit pas être une propriété de l'objet. Une chaise est une identité visuelle avec plusieurs représentations (300 Ko sur un téléphone à 100 €, 2 Mo sur le Flip, 20 Mo sur un PC), sans changer une ligne du site. Au plus, l'auteur dit une intention de coût (`quality: normal`, `budget: light`).
- **Principe proposé pour une ADR** : « Le fichier HoloCode décrit une qualité visuelle souhaitée, jamais une technique graphique particulière. Toute technique — rastérisation, lancer de rayons, splats ou autre technologie future — reste remplaçable par le moteur. »

## Lecture de Claude

- D'accord sur presque tout, et en particulier sur les deux idées neuves : **les paliers qui bougent pendant la visite**, et **le principe d'ADR** ci-dessus, qui protège le langage contre la technique de 2026.
- Une contradiction dans le texte : il approuve d'abord `material: wood, finish: polished, weight: 250KB`, puis rejette `weight`. On retient le rejet.
- Les sources citées (Epic sur Lumen, Kakao, groupe Khronos) n'ont pas été vérifiées par Claude.
