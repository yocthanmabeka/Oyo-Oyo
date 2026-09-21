# ADR-011 — Rendu : la vue à plat par génération de HTML et CSS, la vue en profondeur par le moteur

- Statut : ACCEPTÉ
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013
- Projets affectés : HoloCompiler, HoloEngine
- Proposé par : Claude. Validé par Yocthan le 2026-09-21, après lecture. La fusion de la pull request qui introduit cette fiche vaut confirmation.

## Contexte

Il existe deux routes pour faire tourner HoloCode dans un navigateur. **La traduction** : le compilateur transforme le fichier `.holo` en HTML, CSS et JavaScript, que l'auteur ne voit jamais. **Le moteur apporté** : un moteur compilé en WebAssembly lit le `.holo` et dessine lui-même chaque pixel dans une zone de dessin ; c'est ce que font Flutter Web, Figma et Google Earth.

Yocthan préférerait, si possible, qu'il n'y ait aucun HTML ni CSS du tout.

## Décision

Utiliser les deux routes, une par vue (`ADR-007`) :

- **vue à plat** : traduction en HTML et CSS générés ;
- **vue en profondeur** : moteur Rust en WebAssembly (`ADR-010`).

Dans les deux cas, le fichier source ne contient ni HTML, ni CSS, ni JavaScript. Dans le navigateur propre au projet, le moteur assure les deux vues et il n'y a plus aucun HTML.

## Alternatives étudiées

- **Tout par le moteur**, comme Flutter Web. On perd alors ce que le navigateur offre gratuitement : le texte ne se sélectionne plus, les moteurs de recherche et les lecteurs d'écran ne lisent plus rien, le clavier du téléphone et les formulaires deviennent pénibles, il faut télécharger le moteur avant de voir la moindre page, et la batterie chauffe plus.
- **Tout par traduction** : pas de 3D, pas de zoom continu.
- **Aucun HTML du tout** : impossible dans un navigateur actuel. WebAssembly ne peut parler seul ni à l'écran ni à la carte graphique ; il restera toujours une page d'une dizaine de lignes, identique pour tous les mondes, générée automatiquement.

## Conséquences

### Positives

- La vue à plat est légère, lisible partout, y compris sur un téléphone ancien, et trouvable par un moteur de recherche.

### Négatives et risques

- Deux rendus à garder cohérents.
- Le passage d'une vue à l'autre doit être fluide ; c'est là que l'idée se joue.

## Critères de validation

- Une même page passe de la vue à plat à la vue en profondeur sans rechargement visible.

## Conditions de réexamen

- Si la cohérence entre les deux rendus coûte plus cher que de tout dessiner avec le moteur.
