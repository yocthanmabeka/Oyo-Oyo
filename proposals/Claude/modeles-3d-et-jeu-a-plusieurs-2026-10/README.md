# Propositions : les modèles 3D, et le jeu à plusieurs

- Auteur : Claude
- Date : 2026-10-04
- Statut : propositions à discuter. Rien n'est construit.
- Origine : ce qui reste des étapes 6 et 7 du planning (`docs/01-holocode/COMPARATIF-CONCURRENTS.md`). Yocthan a dit « fais ce qui reste à faire » ; ces deux morceaux-là sont des choix d'architecture, les plus lourds depuis le choix de Rust (`ADR-010`). Claude les pose ici avant de construire, comme le veut la règle du projet.

## 1. Les modèles 3D

### Le problème

Le moteur ne dessine qu'une chose : des points lumineux. Une page, elle, est fabriquée en HTML. Il n'existe aucun chemin pour montrer un objet en volume (une chaise, un personnage).

### Trois façons de faire

**A — Des triangles, comme tous les moteurs 3D.** Lire un fichier de modèle (glTF, le format ouvert), et le dessiner avec des faces, des textures, une lumière.

- Pour : c'est ce que tout le monde attend de la 3D ; tous les modèles existants sont dans ce format.
- Contre : un second moteur de dessin à écrire à côté du premier (faces, profondeur, lumière, textures), et un lecteur de glTF. Le moteur grossirait beaucoup : aujourd'hui 570 Ko, sans doute plus du double. C'est le plus gros chantier depuis le début.

**B — Des modèles faits de points.** Transformer un modèle en nuage de points (on garde ses sommets, ou on en sème sur ses faces), et le donner au moteur tel qu'il est.

- Pour : le moteur sait déjà le faire, et vite (un million de points au repos, 60 images par seconde mesurées). C'est fidèle à la vision : « tout est fait de points, et chaque point contient d'autres points ». Un modèle peut se morceler quand on s'approche, comme le reste.
- Contre : un nuage de points ne ressemble pas à un objet plein. De près, on voit à travers. Il faut un outil qui prépare les modèles.

**C — Poser un lecteur 3D tout fait dans la page** (comme `model-viewer` de Google).

- Pour : presque rien à écrire.
- Contre : une bibliothèque JavaScript de plusieurs centaines de Ko, étrangère au moteur ; l'objet reste enfermé dans son cadre, on n'y entre pas. Contraire à « l'auteur n'écrit ni ne charge de JavaScript ».

### Comparaison

| | A : triangles | B : points | C : lecteur tout fait |
|---|---|---|---|
| Ressemble à un objet réel | oui | de loin | oui |
| Fidèle à la vision « tout est points » | non | oui | non |
| Poids ajouté au moteur | gros | presque rien | gros, et hors du moteur |
| Travail | très gros | moyen | petit |
| Tient sur un téléphone modeste | à mesurer | probable | à mesurer |
| On peut y entrer, le morceler | non | oui | non |

### Recommandation

**B d'abord.** C'est le seul des trois qui prolonge ce que le projet a d'unique, et le seul que le moteur sait déjà porter. Écriture possible, à discuter :

```text
Point(name: Chair, model: "chair.points", inside: World(...))
```

A pourra venir plus tard, si des mondes « pleins » deviennent nécessaires ; ce sera alors une décision du niveau d'`ADR-010`, avec une mesure sur téléphone avant de s'engager.

### Ce que Yocthan a à décider

1. A, B ou C ?
2. Si B : accepter qu'un objet soit un nuage de points, visible comme tel de près ?

## 2. Le jeu à plusieurs

### Le problème

Aujourd'hui l'arbitre vit dans le navigateur de chaque visiteur. Chacun a son panier, son score, sa partie. Pour que deux personnes voient la même chose, il faut un arbitre qu'elles partagent : sur un serveur.

### Ce qui est déjà prêt

- L'arbitre est une fonction pure : un état, un signal, un nouvel état. Le même code Rust tourne dans le navigateur et en ligne de commande (`holo`). Il peut tourner sur un serveur sans changement.
- Le hasard est rejouable : deux arbitres qui reçoivent les mêmes signaux dans le même ordre arrivent au même état.
- La page sait déjà demander des valeurs à son serveur (`Data`).

### Trois façons de faire

**A — Des valeurs partagées, par questions répétées.** La page dit quelles valeurs sont communes (`shared: [score]`). Quand un visiteur fait un geste, la page l'envoie au serveur ; le serveur arbitre et garde l'état ; toutes les pages redemandent l'état chaque seconde.

- Pour : petit ; réutilise `Data` ; suffit pour un compteur commun, un vote, un tableau des scores.
- Contre : une seconde de retard. Pas un jeu d'action.

**B — Une liaison ouverte (WebSocket).** Le serveur pousse chaque changement aussitôt à tous.

- Pour : presque sans retard ; un vrai jeu à plusieurs devient possible.
- Contre : un serveur qui garde une liaison par joueur ; reconnexions, ordre des messages, triche à surveiller. Un vrai service à faire tourner.

**C — De joueur à joueur, sans serveur** (WebRTC).

- Pour : pas de serveur à payer.
- Contre : qui arbitre ? Si c'est un joueur, il peut tricher. Contraire à `ADR-015`.

### Recommandation

**A d'abord**, pour apprendre ce que demande un état partagé (qui a le droit de changer quoi, que devient l'état quand tout le monde part), puis B quand un jeu le demandera. Écriture possible, à discuter :

```text
state: State(votes: 0, mine: 0),
shared: [votes],
```

### Ce que cela demande, que le projet n'a pas

- **Un vrai serveur**, qui tourne en permanence. Aujourd'hui il n'y a qu'un serveur de démonstration sur le PC de Yocthan. Où tournera-t-il, qui le paie, qui le surveille ?
- **Une idée de « qui est qui »** : sans cela, n'importe qui peut envoyer n'importe quel signal.
- **Une relecture de sécurité par Codex** avant toute mise en ligne.

### Ce que Yocthan a à décider

1. Commencer par A (valeurs partagées, sur le serveur de démonstration, pour voir) ?
2. Où tournera le serveur le jour où ce sera en ligne ?
