# Les essais sur le téléphone du 2026-10-07 (le soir)

- **Le téléphone** : Galaxy Z Flip 5 (SM-F731N), Android 16, Chrome 153.0.8010.52.
- **Le câble** : `adb reverse tcp:8080` (le téléphone ouvre `localhost:8080`, le serveur du PC) et `adb forward tcp:9222` (le PC lit le Chrome du téléphone). Les pages viennent du serveur 8080 du PC, avec le code de `main`.
- **Comment** : d'abord, Yocthan essaie les leçons avec ses doigts, une par une ; ensuite, la suite automatique passe par le câble (`node outils/browser-tests.mjs --telephone`, dans `moteur/`).
- **On ne mêle pas les deux** : ce qui est **vu par Yocthan** est séparé de ce qui est **mesuré par le câble**. Ce soir, seule la première partie est faite.

## Ce que Yocthan a vu, leçon par leçon

Le 2026-10-07 au soir, Yocthan a essayé avec ses doigts les leçons 1 à 88, une par une.

| Leçon | Ce qui marche | Ce qui ne va pas | Suite |
|---|---|---|---|
| 7 — un point et son monde | les zooms | **la page quitte sa place quand on zoome**, et cela peut gêner des visiteurs | Décidé par Yocthan : comme pour tourner. **Construit dans le lot 4** : la page reste accrochée, le zoom est celui du navigateur ; `Zoom(detach: true)` offre « Décrocher / Accrocher » (leçon 94). Détail plus bas. |
| 26 — le Big Bang | — | **sur le téléphone, rien ne mène à une autre leçon** : le Big Bang est un point seul, sans lien. Sur l'ordinateur, Yocthan passe par la pile (`/stack`). | Ce soir, la leçon 27 lui a été ouverte par le câble. **Construit dans le lot 4** : en haut de chaque monde, « Les leçons » et « La pile », et « Leçon 27 → » quand le monde est la leçon 26 ; au doigt comme à la souris. |
| 48 — l'heure | les heures et les minutes | **ni secondes ni millisecondes** | Expliqué : l'heure est donnée chaque minute, pour la batterie. **Proposé pour le lot 7** : `{second}`, chaque seconde, seulement si la page l'affiche ; les millisecondes par un bloc chronomètre (démarrer, arrêter, remettre à zéro) que la page dessine au rythme de l'écran, le moteur ne recevant que le temps final. |
| 54 — sa propre police | — | Yocthan voudrait **toutes les polices libres, par défaut** | Expliqué : toutes, ce sont des milliers de polices, des gigaoctets, ou un service extérieur ; ce n'est pas raisonnable. Aujourd'hui, n'importe quelle police libre marche en rangeant son fichier à côté de la page. **Proposé pour le lot 7** : une trentaine de polices libres (licences vérifiées, toutes les écritures), gardées sur notre serveur, chargées seulement quand une page les nomme : `Font(family: "Inter")`. |
| 63 — la fenêtre | vider le panier, le bouton | **le lien vers la leçon 64 ne se laisse pas toucher** | Cause : la fenêtre fermée restait posée, invisible, sur le bas de la page. **Corrigé et fusionné** (PR 160, 6 tests verts). |
| 77 — toutes les touches | au clavier, sur l'ordinateur | **rien ne marche au doigt** : « P ajoute 1 », mais aucun clavier n'apparaît | Normal pour un vrai clavier : celui de l'écran n'apparaît que dans un champ où l'on écrit. Le défaut : la leçon ne le disait pas et n'offrait rien au doigt. **Construit dans le lot 4** : sur un appareil tactile sans souris, le moteur montre en bas de l'écran les touches que la page écoute (ici P, M, 5, Entrée, Échap) ; gardée enfoncée, une touche se répète ; le bas de la page reste atteignable. |

**Bilan** : leçons 1 à 88, « jusqu'à la 88e leçon, tout est bon », hormis ces six lignes. Yocthan a aussi demandé que le lecteur d'écran du téléphone soit expliqué comme le Narrateur de Windows : TalkBack (Android) et VoiceOver (iPhone) lisent les pages sans rien ajouter au moteur, et les leçons 35 et 81 disent maintenant comment les lancer.

### La page qui quitte sa place (leçon 7), reproduite sur le PC

Chrome sans fenêtre, à la taille d'un téléphone (400 × 800), zoom vers le point (320, 260) :

- 0 cran : la page est à (0, 0), large de 400 px ;
- 3 crans : la page est grossie 4 fois (1 600 px de large) et a glissé à (−960, −779). On ne voit plus que son coin bas-droit (captures `captures/pc-lecon-07-zoom-0-cran.png` et `pc-lecon-07-zoom-3-crans.png`).

Sur `main`, c'est le moteur qui grossit la page (`grow()` dans `moteur/web/page-engine.js`) : ce qui est sous les doigts y reste, donc la page glisse vers le côté où l'on zoome.

### Décrocher, accrocher : l'idée de Yocthan (pour la 3D)

- **Décrocher** la page du web normal, c'est passer dans le métavers.
- **Accrocher** une feuille sur l'écran, c'est revenir au web normal.
- Derrière une page décrochée, il peut y avoir d'autres pages alignées, prêtes à être accrochées, comme une pile de papiers. Elles ne sont pas obligatoires : il peut y en avoir zéro.

L'avis de Claude :

- L'idée tient : le geste se comprend sans explication, et il suit la règle « le site 2D d'abord ».
- **Maintenant**, pour le web : la page accrochée par défaut, et le bouton « Décrocher / Accrocher » quand l'auteur l'écrit.
- **Avec la 3D principale** : la pile de feuilles, et le vrai passage au métavers. Proposition : la pile contient les pages du même site vers lesquelles la page mène, et zéro si l'auteur n'en met pas.

**La décision de Yocthan** (2026-10-07) : comme pour tourner. Par défaut, la page reste accrochée ; l'auteur permet de la décrocher dans le fichier.

**Construit dans le lot 4** (branche `langage/lot4-mise-en-page`) :

- sans rien écrire, pincer à deux doigts (ou Ctrl + molette) fait le zoom du navigateur, comme sur n'importe quel site : la page grossit sur place ;
- `zoom: Zoom(detach: true)` met « Décrocher » dans le menu ☰ : la page se détache de l'écran comme une feuille, et le zoom l'approche ; « Accrocher » la remet à sa place, à sa taille ;
- le moteur ne grossit lui-même la page que si le fichier le demande (la vue points, `Zoom(shrink:)`, `Zoom(active: false)`) ou si le visiteur l'a décrochée ;
- la leçon 94, « Décrocher la page », l'enseigne.

**Pas encore construit** : la pile de feuilles et le passage au métavers, qui attendent la 3D principale.

## La règle de parité téléphone / ordinateur

Posée par Yocthan ce soir-là : « si la fonction existe sur téléphone, elle doit strictement aussi exister sur ordinateur et vice-versa ».

- Ce que le clavier ou la souris font sur l'ordinateur, le doigt doit pouvoir le faire sur le téléphone, et l'inverse.
- Appliquée dans le lot 4 : les touches à l'écran (leçon 77) ; TalkBack (Android) et VoiceOver (iPhone) expliqués à côté du Narrateur de Windows (leçons 35 et 81) ; « Décrocher / Accrocher » dans le même menu ☰ sur les deux.
- À trancher : la leçon 89 (lot 4) permet de cacher un bloc sur un seul appareil (`display: none` dans `phone:` ou `computer:`). Pour une phrase, c'est l'allure qui change ; pour un bouton, ce serait une fonction d'un seul côté.

## Ce qui a été mesuré par le câble

**Rien encore.** La suite automatique (`node outils/browser-tests.mjs --telephone`, dans la même pull request que ce compte rendu) a été arrêtée pendant que Yocthan essayait, pour ne pas changer les pages sous ses doigts. Elle sera relancée après.

À mesurer :

- le temps de chargement, sans cache : leçon 1, site de référence, leçon 84, Big Bang ;
- rien ne déborde de l'écran : `scrollWidth ≤ innerWidth`, sur toutes les leçons ;
- le délai entre une lettre tapée et la liste refaite (leçon 82) : cible, moins de 50 ms ;
- le clavier à chiffres avec la virgule, et « 9,99 » bien lu (leçon 86) ; le `min` du champ date (leçon 87) ;
- les images par seconde et l'image la plus lente, dans le Big Bang et dans la vue points ;
- la mémoire de l'onglet (`dumpsys meminfo`) ;
- la batterie et la chaleur, avant et après ;
- des captures d'écran, rangées dans `captures/`.

Pour comparer, la mesure du 2026-10-04 sur ce même téléphone (détail dans `moteur/README.md`) : 59,8 images par seconde en zoomant dans le Big Bang à travers sept mondes (image la plus lente : 16,9 ms) ; 59,7 en vue points sur la boutique (image la plus lente : 50,1 ms) ; 88 Mo pour l'onglet, vue points ouverte ; première image en 3,5 s au tout premier chargement.

## Défauts et dettes

- **La page quitte sa place au zoom** (leçon 7) : lot 4, construit (leçon 94).
- **Les touches ne marchent pas au doigt** (leçon 77) : lot 4, construit.
- **Les leçons 35 et 81 ne parlaient que du Narrateur de Windows** : lot 4, construit.
- **La fenêtre fermée couvrait les liens** (leçon 63) : corrigé et fusionné (PR 160).
- **Aucun chemin au doigt du Big Bang vers les leçons** (leçon 26) : lot 4, construit.
- **Ni secondes ni millisecondes** (leçon 48) : proposé pour le lot 7.
- **Pas de polices libres prêtes à l'emploi** (leçon 54) : proposé pour le lot 7.
- **Rien de mesuré par le câble** : la suite `--telephone` est prête, à relancer avec le téléphone branché.
- **Le lot 4 n'a pas encore été vu au doigt** (leçons 7, 26, 35, 77, 81, 89 à 94) : à revoir sur le téléphone.
- **Cacher un bouton sur un seul appareil** (leçon 89) contredirait la parité : à trancher.
