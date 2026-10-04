# Réponse de Gemini — objets en volume et jeu à plusieurs (2026-10-04)

Réponse au prompt [`../prompts/2026-10-04-gemini-3d-et-plusieurs.md`](../prompts/2026-10-04-gemini-3d-et-plusieurs.md), collée par Yocthan le 2026-10-04. Gemini n'a pas accès au dépôt. Texte gardé tel quel ; la lecture critique de Claude est à la fin.

---

## Question 1 — Montrer des objets en volume

### 1. Nuages de points dans le navigateur : l'existant

- **Potree** (visualisation LiDAR open source) : le standard pour des relevés de centaines de millions de points. Visualiseur JavaScript/Three.js d'environ 500 Ko à 1 Mo. Sur ordinateur, 5 à 10 millions de points à 60 FPS grâce à un octree (`PotreeConverter`) qui charge selon la distance. Sur téléphone, la batterie chauffe vite. Principal défaut : pas de surface fermée ; de près, les points s'écartent et l'objet devient une passoire.
- **3D Gaussian Splatting** (antimatter15/splat, PlayCanvas) : des millions de taches ellipsoïdales transparentes qui reforment une surface photo-réaliste. Lecteur minuscule, mais fichier de 30 à 150 Mo par scène. Sur téléphone, très lourd : tri en profondeur de millions d'ellipsoïdes à chaque image.
- **Luma AI et Polycam** : visionneuses commerciales de scans. Passées des nuages et NeRF à des maillages texturés légers ou des splats très compressés. Sur téléphone, beaucoup de mémoire, rechargements forcés de l'onglet.
- **La scène démo (4K / 64K)** : rarement des millions de points ; des formules (raymarching, SDF) calculées pixel par pixel. Quelques kilo-octets de code, mais le processeur graphique du téléphone chauffe.

### 2. Les Gaussian Splats : quatrième option sérieuse ?

Non, pas pour 1 Go par monde et un téléphone ordinaire. Un objet compressé (`.spz`, 8 bits) pèse 3 à 15 Mo, quand le moteur entier pèse 570 Ko. Le tri arrière-avant de 500 000 éléments à 60 FPS vide la batterie en un quart d'heure.

### 3. Option A : le plus petit moteur à triangles honnête

| Moteur | Poids compressé moyen | Pour HoloCode |
|---|---|---|
| Babylon.js | ~1 000 à 3 500 Ko | Complet mais trop lourd. |
| Three.js (noyau) | ~150 à 180 Ko (glTF : +50 Ko) | Très mûr, mais JavaScript hors de la boucle Rust. |
| PlayCanvas Engine | ~200 à 250 Ko | Excellent sur téléphone, orienté jeu classique. |
| Bevy (Rust → wasm) | ~5 000 à 15 000 Ko | Inadapté. |
| Wonderland Engine | ~150 à 200 Ko | Rapide (WebXR), mais propriétaire. |
| Petit moteur Rust maison | ~30 à 60 Ko | Un chargeur glTF binaire minimal dans le binaire actuel. |

### 4. Recommandation

B d'abord, puis A minimal en interne.

1. Nuage de points dont la taille à l'écran dépend de la distance (*point size attenuation*) : de près, les points grossissent et se chevauchent, l'objet paraît plein, sans nouveau moteur.
2. Plus tard : un chargeur glTF minimal écrit en Rust, qui dessine des triangles indexés. Rejeter C (Three.js, Babylon, model-viewer) : casse la cohérence de l'arbitre, ajoute 1 Mo de code tiers.

## Question 2 — Jouer à plusieurs

### 1. Comment font les autres

- **Croquet / Multisynq** (le plus proche) : réplication déterministe par réflecteur. Aucun calcul sur le serveur : il horodate et renvoie les gestes à tous. Chaque client fait tourner la même machine à états ; mêmes signaux dans le même ordre, même état.
- **Roblox, Photon, Nakama, Colyseus** : serveur qui fait autorité ; il calcule le monde et renvoie les positions, le client n'est qu'un écran.
- **Rollback (GGPO)** : prédiction locale ; si un signal arrive en retard, on remonte le temps, on rejoue, on réaffiche.
- **Yjs, Liveblocks (CRDT)** : faits pour des documents partagés, pas pour une pomme qui tombe à 60 images par seconde.

### 2. L'arbitre pur : forces et angles morts

Forces : presque rien à envoyer (« au pas 142, le joueur 1 appuie sur gauche », moins de 20 octets) ; rejeu et sauvegarde parfaits ; un score falsifié se voit, les autres recalculent.

Angles morts :
- **L'ordre des messages** : deux gestes au même instant doivent être résolus dans le même ordre partout. Le serveur doit donner un numéro d'ordre officiel à chaque signal.
- **Le retard réseau** : avec 200 ms de ping, soit tout le monde attend (saccades), soit il faut un rollback.

### 3. La rencontre liée à la largeur de l'écran

Problème éliminatoire pour le jeu à plusieurs : sur 1 200 px la pomme touche, sur 360 px elle tombe à côté ; deux réalités. Solution des jeux : un espace logique fixe (par exemple 1000 × 1000 unités, ou un rapport 16:9), mis à l'échelle à l'affichage (bandes sur les côtés). Les collisions se calculent en unités logiques, jamais en pixels.

### 4. Coût pour 100 joueurs simultanés

- **A, questions répétées chaque seconde** : 100 requêtes par seconde, 260 millions par mois ; 5 à 15 €/mois (petit serveur ou fonctions sans serveur). Défaut : une seconde de retard, injouable quand la pomme bouge toutes les 100 ms.
- **B, WebSocket en Rust** : 100 connexions, moins de 20 Mo de mémoire, moins de 1 % d'un cœur (`tokio-tungstenite` ou `axum`) ; 4 à 6 €/mois (Hetzner CX22, Scaleway Stardust). Sources citées : TechEmpower, tarifs Hetzner.

### 5. Recommandation

Aller directement au WebSocket léger (B), sur le modèle de Croquet. Le polling est une fausse simplification : plus de requêtes, une seconde de retard.

À trancher avant la première ligne de réseau :
1. L'unité de monde : des coordonnées normalisées (0 à 100) au lieu des pixels.
2. Le maître du temps : le serveur seul attribue à chaque message un numéro de pas.

## Tableau récapitulatif de Gemini

| Recommandation | Statut | Justification |
|---|---|---|
| Objets 3D : points de taille adaptative (B) | À faire maintenant | Aucun moteur de plus. |
| Séparer coordonnées logiques et pixels | À faire maintenant | Sans cela, le jeu à plusieurs se désynchronise. |
| Jeu à plusieurs par WebSocket en Rust (réflecteur de gestes) | À faire maintenant | Moins de 6 €/mois pour 100 joueurs ; moins de 30 ms de retard. |
| Chargeur glTF minimal en Rust | Plus tard | Quand les points seront un frein avéré. |
| Pair à pair WebRTC (C) | Plus tard | STUN/TURN derrière les réseaux mobiles. |
| Gaussian Splats | À ne pas faire | Fichiers de 30 à 100 Mo, tri trop lourd. |
| Moteur JS tiers (Three.js, Babylon, model-viewer) | À ne pas faire | Casse l'indépendance du binaire, ne permet pas d'entrer dans l'objet. |
| Questions répétées chaque seconde (A) | À ne pas faire | Une seconde de retard. |

---

## Lecture critique de Claude

**D'accord**

- Les points d'abord, et l'idée de points qui grossissent de près : elle ne coûte presque rien et répond au défaut de la passoire. C'est la meilleure idée concrète de la réponse.
- Le jeu à plusieurs sur le modèle de Croquet va avec ce qu'on a : l'arbitre est déjà une fonction pure et le hasard est déjà rejouable. Il faut alors que le serveur donne aussi les **pas d'horloge** : aujourd'hui, chaque page a ses propres horloges (`Every`), ce qui suffit pour désynchroniser deux téléphones.
- La rencontre ne doit pas dépendre de l'écran. Claude venait de la faire dépendre de la largeur du plateau ; Gemini a raison, c'est à reprendre : un plateau aux proportions fixes, et les rencontres calculées dans ses unités.

**Pas d'accord, ou à nuancer**

- Le polling écarté : juste pour un jeu d'action comme la pomme. La proposition A de Claude visait d'abord des valeurs partagées lentes (un compteur de visites, un tour par tour), pour lesquelles une seconde suffit. Mais si le premier jeu à plusieurs doit être la pomme, Gemini a raison.
- Les chiffres ne sont pas vérifiés (Claude ne les a pas recoupés) : le « petit moteur Rust de 30 à 60 Ko » n'existe pas, c'est une estimation ; les coûts d'hébergement sont des ordres de grandeur.
- « 570 Ko » est le poids compressé du moteur ; non compressé, il pèse 2,1 Mo.
- Le WebSocket demande un serveur à louer et des bibliothèques Rust à ajouter : deux décisions de Yocthan.
