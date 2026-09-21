# ADR-014 — L'IA agit à la création, jamais à la lecture

- Statut : PROPOSITION
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013
- Projets affectés : HoloCode, HoloCompiler, outils de création
- Proposé par : Claude. Non validé.

## Contexte

Yocthan veut que l'IA participe à la composition des mondes, à la manière de Genie 3 : un monde apparaît à partir de très peu de données. Il veut que l'IA respecte des mesures pour que le monde ne se dégrade pas, que les couleurs et les apparences soient choisies par l'utilisateur, et qu'un mode automatique tire des valeurs au hasard pour donner de la diversité. Il pensait qu'il faudrait importer Python.

## Décision proposée

- **Pas besoin de Python.** Python sert à entraîner les modèles. Pour en utiliser un, on l'appelle par le réseau ou on fait tourner un petit modèle dans le téléphone ; le moteur en Rust fait les deux.
- **L'IA agit à la création.** L'utilisateur dit « fais-moi une forêt avec une rivière » ; l'IA écrit du `.holo` ; le fichier est enregistré. Ensuite tout le monde voit le même monde, sans IA. C'est la conséquence directe de `ADR-008`.
- **Les mesures à respecter, c'est le vérificateur.** Ce que l'IA produit est du `.holo` ordinaire, soumis aux mêmes contrôles qu'un humain : budget mémoire, unités, lois. L'IA propose, le vérificateur dispose.
- **Le hasard passe par des graines.** `graine: auto` tire un nombre à la création, puis l'écrit dans le fichier : aléatoire, mais reproductible.
- **Plus tard, des personnages animés en direct par une IA** sont traités comme un joueur extérieur : ils agissent par les capacités, et les lois s'appliquent à eux.

## Alternatives étudiées

- **Générer les images en direct, comme Genie 3** : très peu de données mais énormément de calcul, sur des machines de centre de données ; mondes de quelques minutes, ni programmables, ni partageables ; impossible sur un téléphone à 1 Go aujourd'hui. Genie 3 génère des pixels ; ici on génère la description, et le moteur dessine.
- **Des images générées par IA comme ressources** (textures, ciels) : possibles à la création, mais elles pèsent lourd et comptent dans le budget ; une texture définie par une formule pèse quelques octets.

## Conséquences

### Positives

- L'IA ne peut pas dégrader un monde : tout ce qu'elle écrit passe le vérificateur.
- Un monde créé par IA se lit sans IA, hors ligne, et pèse quelques kilo-octets.

### Négatives et risques

- Une IA doit savoir écrire un langage qui n'existe nulle part dans ses données d'entraînement : il faut une grammaire petite et des messages d'erreur qui lui permettent de se corriger.

## Critères de validation

- À partir d'une phrase, une IA produit un fichier `.holo` que le vérificateur accepte, et que deux téléphones affichent à l'identique.

## Conditions de réexamen

- Le jour où un modèle générateur de mondes tourne sur un téléphone dans le budget.
