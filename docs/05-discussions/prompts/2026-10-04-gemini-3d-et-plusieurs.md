Bonjour Gemini. C'est Yocthan Mabeka, pour le projet Holoverse / HoloCode. Tu n'as pas accès au dépôt : tout ce qu'il te faut est dans ce message. Réponds en français, en phrases simples. Cite tes sources, et dis clairement quand tu n'es pas sûr.

# Où en est le projet

- Un site s'écrit dans un fichier `.holo`. Par défaut, c'est un site web normal. Quand l'auteur l'active et que le visiteur zoome, chaque pixel de la page devient un point lumineux ; un point peut contenir un monde, ou un autre site.
- Le moteur est en Rust, compilé en WebAssembly, et dessine avec WebGPU (repli WebGL 2). Il pèse environ 570 Ko. Il ne sait dessiner qu'une chose : des points lumineux. La page elle-même est fabriquée en HTML et CSS par le moteur.
- Mesuré sur deux téléphones (Galaxy Z Flip 5 et Z Flip 3) : 60 images par seconde, y compris avec un million de points au repos, en WebGPU comme en WebGL 2. Aucun téléphone d'entrée de gamme n'a été mesuré.
- L'auteur n'écrit jamais de code. Il écrit des blocs, des valeurs et des règles. Exemple d'un jeu entier :

```holo
Page(
  name: Orchard,
  state: State(lives: 3, score: 0, basket: 50, apple_x: 50, apple_y: 0),
  children: [
    Text("Score: {score}. Lives: {lives}"),
    Board(height: 360px, children: [
      Shape(name: Apple, form: circle, color: "#E4572E", size: 44px, x: apple_x, y: apple_y),
      Shape(name: Basket, form: square, color: "#E9B44C", size: 64px, x: basket, y: 96, drag: true),
    ]),
  ],
  rules: [
    On(Key.left, effect: basket.sub(8)),
    Every(100ms, effect: apple_y.add(3)),
    When(Basket, meets: Apple, effect: [score.add(1), apple_y.set(0)]),
    When(apple_y, over: 99, effect: [lives.sub(1), apple_y.set(0)]),
  ],
)
```

- Tout changement de valeur passe par un « arbitre » : une fonction pure (un état, un signal, un nouvel état), écrite en Rust. Le même code tourne dans le navigateur et en ligne de commande. Le hasard est rejouable (il sort d'une graine).
- Aujourd'hui l'arbitre vit dans le navigateur de chaque visiteur : chacun a sa partie. Une page sait déjà demander des valeurs à son propre serveur, à un rythme donné.

# Les deux questions

## Question 1 — Montrer des objets en volume

Trois options sont sur la table :

- **A. Des triangles**, comme tous les moteurs 3D : lire un fichier glTF, dessiner des faces, des textures, une lumière. C'est un second moteur de dessin à écrire.
- **B. Des objets faits de points** : transformer un modèle en nuage de points et le donner au moteur tel qu'il est. Fidèle à la vision « tout est fait de points », mais de près on voit à travers.
- **C. Un lecteur 3D tout fait** posé dans la page (comme `model-viewer`). Du JavaScript étranger au moteur ; on n'entre pas dans l'objet.

Claude recommande B d'abord.

Ce que je te demande :

1. Qui a déjà montré des objets en nuages de points dans un navigateur, et qu'est-ce que ça a donné ? Regarde Potree, les « Gaussian splats » (3D Gaussian Splatting, et leurs lecteurs web), les démos de la scène démo, Luma, Polycam. Pour chacun : le poids, la fluidité sur téléphone, ce que les gens en disent.
2. Les « Gaussian splats » sont-ils une quatrième option sérieuse pour nous ? Ils ressemblent à des points, mais donnent des objets pleins. Que coûtent-ils (taille des fichiers, calcul, tri des points) ?
3. Pour l'option A : quel est le plus petit moteur à triangles honnête en WebGPU ou WebGL 2 ? Combien pèsent Three.js, Babylon.js, PlayCanvas, Bevy, Wonderland Engine, une fois compressés ?
4. Ta recommandation, avec l'ordre dans lequel le faire.

## Question 2 — Jouer à plusieurs

Trois options :

- **A. Des valeurs partagées, par questions répétées** : la page dit quelles valeurs sont communes ; chaque geste est envoyé au serveur, qui arbitre ; toutes les pages redemandent l'état chaque seconde.
- **B. Une liaison ouverte (WebSocket)** : le serveur pousse chaque changement aussitôt.
- **C. De joueur à joueur (WebRTC)**, sans serveur.

Claude recommande A d'abord, puis B.

Ce que je te demande :

1. Comment font les jeux et les mondes en ligne existants ? Regarde Roblox, Croquet (et Multisynq), Colyseus, Nakama, Photon, le « rollback netcode » des jeux de combat, Liveblocks, Yjs et les CRDT. Lesquels reposent, comme nous, sur un arbitre déterministe et rejouable ?
2. Notre arbitre est une fonction pure et rejouable. Qu'est-ce que cela nous permet que les autres n'ont pas (rejeu, vérification d'une partie, moins de données envoyées) ? Qu'est-ce que cela ne règle pas (la triche, l'ordre des messages, les joueurs lents) ?
3. Depuis peu, deux objets se rencontrent quand leurs bords se touchent à l'écran, et cela dépend de la largeur de l'écran. Est-ce un problème pour un jeu à plusieurs ? Comment les jeux en ligne règlent-ils cela (un monde de taille fixe, des unités de monde plutôt que des pixels) ?
4. Combien coûte, par mois, de faire tourner un petit serveur pour l'option A, puis pour B, pour cent joueurs en même temps ? Donne des ordres de grandeur et tes sources.
5. Ta recommandation, et ce qu'il faut absolument décider avant d'écrire la première ligne.

Termine par un tableau : une ligne par recommandation, avec « à faire maintenant », « plus tard » ou « à ne pas faire ».
