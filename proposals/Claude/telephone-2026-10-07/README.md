# Les essais sur le téléphone du 2026-10-07 (le soir)

- **Le téléphone** : Galaxy Z Flip 5 (SM-F731N), Android 16, Chrome 153.0.8010.52. Branché par câble (`adb reverse tcp:8080`, `adb forward tcp:9222`). Les pages viennent du serveur 8080 du PC, avec le code de `main`.
- **Comment** : d'abord les retours de Yocthan, qui essaie les leçons avec ses doigts ; ensuite les essais par le câble (`node outils/browser-tests.mjs --telephone`).
- On ne mêle pas les deux : ce qui est **vu par Yocthan** est séparé de ce qui est **mesuré par le câble**. La suite par le câble a été arrêtée pendant que Yocthan essayait, pour ne pas changer les pages sous ses doigts.

## Ce que Yocthan a vu, leçon par leçon

| Leçon | Ce qui marche | Ce qui ne va pas | Suite |
|---|---|---|---|
| 7 — un point et son monde | les zooms | **la page quitte sa place quand on zoome**, et cela peut gêner des visiteurs | Décision de Yocthan : **comme pour tourner**. Par défaut, la page reste accrochée ; l'auteur permet de la décrocher dans le fichier (par exemple `Relief(detach: true)`), et un bouton « Décrocher » apparaît. |

### La page qui quitte sa place (leçon 7), reproduite sur le PC

Chrome sans fenêtre, à la taille d'un téléphone (400 × 800), zoom vers le point (320, 260) :

- 0 cran : la page est à (0, 0), large de 400 px ;
- 3 crans : la page est grossie 4 fois (1 600 px de large) et a glissé à (−960, −779). On ne voit plus que son coin bas-droit (captures `captures/pc-lecon-07-zoom-0-cran.png` et `pc-lecon-07-zoom-3-crans.png`).

C'est le zoom ordinaire de la page (`grow()` dans `moteur/web/page-engine.js`) : ce qui est sous les doigts y reste, comme dans un navigateur, donc la page glisse vers le côté où l'on zoome.

### Une idée de Yocthan : décrocher, accrocher (une suggestion, pour la 3D)

- **Décrocher** la page du web normal, c'est passer dans le métavers.
- **Accrocher** une feuille sur l'écran, c'est revenir au web normal.
- Derrière une page décrochée, il peut y avoir d'autres pages alignées, prêtes à être accrochées, comme une pile de papiers. Elles ne sont pas obligatoires : il peut y en avoir zéro.

L'avis de Claude :

- L'idée tient : le geste se comprend sans explication, et il suit la règle « le site 2D d'abord ».
- **Maintenant**, pour le web : la page accrochée par défaut, et le bouton « Décrocher / Accrocher » quand l'auteur l'écrit.
- **Avec la 3D principale** : la pile de feuilles, et le vrai passage au métavers. Proposition : la pile contient les pages du même site vers lesquelles la page mène, et zéro si l'auteur n'en met pas.

## Ce qui a été mesuré par le câble

(à venir, après les essais de Yocthan)
