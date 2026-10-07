# ADR-053 — Un moteur léger pour les pages qui bougent, le dessin à part

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md` (point 4 : « alléger le moteur d'une page qui bouge : l'arbitre seul, sans le dessin des points ; cible : moins de 100 Ko ») ; `ADR-033` (la page légère, le moteur chargé au premier geste)
- Validation : Yocthan, le 2026-10-06 : « je suis tes propositions, je les valide, tu as mon feu vert ».
- Projets affectés : HoloEngine

## Contexte

Une page qu'on ne fait que lire pèse 8 Ko (`ADR-033`). Mais dès qu'elle bouge (un bouton, un panier, une liste), elle téléchargeait le moteur entier, avec le dessin des points et des mondes (WebGPU, WebGL 2) : environ 626 Ko compressés, même pour une page sans aucun point.

## Décision

1. **Deux moteurs, un seul code** : une option de compilation, `dessin`, active par défaut. Sans elle (`--no-default-features`), le **moteur léger** lit le fichier, fabrique la page et arbitre les valeurs ; il ne dessine rien et n'embarque pas `wgpu`.
2. **La page prend le moteur léger** (`/pkg-light/`). Le **dessin** (`/pkg/`) n'est téléchargé que si la page s'en sert : des points, un monde, la vue points, la rotation. Le moteur le dit (`a_besoin_du_dessin`) ; une page qui en aura besoin le fait venir tout de suite, sans l'attendre ; une autre ne le télécharge jamais.
3. **L'éditeur** prend aussi le moteur léger : il ne dessine rien.
4. Le moteur léger est optimisé pour la taille (`opt-level = "z"`).

## Mesures (2026-10-06, dans le nuage)

| | Réel | Transféré (Brotli 11) |
|---|---|---|
| Moteur entier (`/pkg/`) | 2 449 Ko | 626 Ko |
| **Moteur léger** (`/pkg-light/`) | 458 Ko | **149 Ko** |
| Colle JavaScript du moteur léger | 39 Ko | 5 Ko |

**L'objectif de 100 Ko n'est pas atteint.** Le moteur léger est quatre fois plus petit, mais ce qui reste est le cœur lui-même : la lecture du fichier, toutes ses vérifications et leurs messages d'erreur, la fabrication de la page, l'arbitre. Le prochain levier serait un moteur d'exécution qui fait confiance à la page déjà vérifiée par le serveur, sans les messages : un autre chantier, à décider.

## Conséquences

- `outils/build.ps1` construit les deux paquets ; les tests GitHub compilent les deux.
- Le dessin n'a pas pu être regardé dans le nuage (pas de carte graphique) : la leçon 9 y fait bien venir le dessin au zoom et passe en vue points, mais l'image elle-même est à vérifier sur un vrai PC et un vrai téléphone.

## Critères de validation

- La leçon 70 (une page qui bouge) ne télécharge que `/pkg-light/` ; ses boutons marchent.
- La leçon 9 (des points au zoom) télécharge `/pkg/` au zoom.
- 114 tests ; le moteur léger compile sans `wgpu`.
