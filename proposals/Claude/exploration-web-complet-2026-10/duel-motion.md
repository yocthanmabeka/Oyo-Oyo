# Le duel du motion design : répétition, particules, chemins, doigt

> Statut : EXPLORATION. Avis de Claude, pas une décision.

## Ce que Codex demandait

Dans l'issue #82 (texte transmis par Yocthan le 2026-10-06) :

> Pour ton duel de motion design, les ajouts les plus directs seraient **la répétition, les particules, les chemins vectoriels et les réactions au doigt**. Le comparatif actuel montre précisément ces écarts.

Le duel (`exemples/motion/README.md`) a trois fichiers :

- `exemples/motion/holocode/showreel.holo` : le film en HoloCode, sept scènes, aucune ligne de JavaScript ;
- `exemples/motion/web/showreel.html` : son **jumeau exact** en HTML et CSS (7 lignes de JavaScript) ;
- `exemples/motion/web/showreel-max.html` : **hors comparaison**, ce que le web sait faire de plus (520 particules, un carré qui se change en cœur, des courbes qui se dessinent, un cube, la souris).

Une remarque d'abord, pour lire la suite : **face au jumeau exact, l'écart n'est plus dans ce qu'on voit.** Les deux films sont identiques à l'écran (README, captures du 2026-10-06). L'écart est dans le poids, et dans ce qu'il faut savoir. Les quatre manques de Codex sont l'écart avec la version « max ».

## Le duel aujourd'hui, mesuré (main, 7a48def, 2026-10-07)

| | HoloCode (`showreel.holo`) | HoloCode, scène 1 réécrite avec `Repeat` (essai) | Jumeau HTML (`showreel.html`) |
|---|---|---|---|
| Lignes utiles | 206 | **140** | 184 (selon le README) |
| Octets du fichier (fins de ligne du dépôt) | 14 193 | **10 975** | 21 377 |
| Page fabriquée par le moteur | 49 843 octets | la même, octet pour octet | — |
| Envoyé à l'ouverture par le serveur local (Brotli : niveau 5 pour une page fabriquée, `server.mjs:392` ; niveau 11 pour un fichier tel quel, `server.mjs:90`) | 12 053 octets | 12 053 (déduit : même page) | **3 849** |
| Le même, compressé au même niveau (Brotli 11) | 10 878 | 10 878 (déduit) | 3 849 |
| dont la porte d'entrée seule (`page.html`, Brotli 11) | 5 578 | 5 578 | — |
| `@keyframes` dans la page | 109 | 109 | 5 |
| JavaScript écrit | **0** | 0 | 7 lignes |
| Les titres coupés en lettres, au lecteur d'écran | « C L A U D E » | pareil | **pareil** |
| Moins de mouvement | automatique : 0 animation, la dernière scène seule visible | pareil | écrit à la main (`@media`) |

Ce que cela dit :

- Le poids envoyé est trois fois celui du jumeau. Près de la moitié est la porte d'entrée (`page.html`) : elle porte le zoom, la vue points, le menu et l'attente du moteur, que le jumeau n'a pas. À fonctions égales, une partie de cet écart achète des fonctions.
- L'autre moitié est la page fabriquée. Chaque bloc qui bouge y a ses propres `@keyframes` : 24 139 octets bruts d'animations sur 49 843 (mesuré). Le jumeau, lui, a 5 `@keyframes` en tout : deux servent à tous les blocs qui bougent (`entre` pour les entrées, `vit` pour les boucles), réglées par des variables ; trois servent aux scènes.
- Les deux films ont le même défaut d'accessibilité (les lettres), découvert pendant cette exploration.

## 1. La répétition

### État vérifié

- `Repeat(items:)` existe depuis le lot 3 (ADR-040, `moteur/src/repeat.rs`, leçon 49). Un champ mis à la place d'une valeur, `x: item.x`, devient cette valeur (`repeat.rs:250-260`), même dans `Enter(…)` et `Loop(…)`.
- **Mesuré : les 36 éclats de la scène 1 s'écrivent déjà avec un seul modèle et 36 lignes `Item(…)`.** `holo check` répond « ok », et la page fabriquée est **identique, octet pour octet**, à celle d'aujourd'hui. Le fichier passe de 206 à 140 lignes utiles, et de 14 193 à 10 975 octets : plus court que le jumeau (184 lignes, 21 377 octets).
- Donc deux documents sont en retard : `docs/02-gouvernance/adr/ADR-034-mouvement.md:69` (« Sans listes répétées, 36 éclats s'écrivent un par un ») et `exemples/motion/README.md:25` (« Les 36 éclats prennent 3 lignes chacun »).
- Limites : 200 éléments par répétition (`repeat.rs:26`) ; 20 000 blocs par page (`repeat.rs:28`) ; pas de répétition dans une répétition (`repeat.rs:124-126`) ; chaque éclat garde ses sept nombres (taille, `x`, `y`, `dx`, `dy`, `at`, `beat`), sa forme et sa couleur, écrits à la main ; le jumeau, lui, écrit douze variables par éclat, plus sa forme.

### Options

| Option | Écriture HoloCode | Équivalent web | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Ce qui existe | `Repeat(items: [ Item(form: diamond, color: "#E9B44C", size: 14px, x: 96, y: 50, dx: -270px, dy: 0px, at: 1.4s, beat: 0.5s), … ], children: [ Shape(form: item.form, …, enter: Enter(x: item.dx, …), loop: Loop(scale: 1.6, for: item.beat)) ])` | 36 lignes de HTML avec des variables CSS (le jumeau) | une liste et `for` dans `children` | rien à construire ; mesuré | sept nombres, une forme et une couleur par éclat |
| B. Une répétition calculée | `Ring(count: 36, radius: 150px, children: [ … ])` : le moteur place les copies en cercle (écriture proposée) | une boucle en JavaScript avec `Math.cos` ; ou `cos()` et `sin()` en CSS avec un `--i` par copie | `Flow` et un `FlowDelegate` | une ligne au lieu de 36 | un mot nouveau par disposition ; HoloCode refuse les expressions (ADR-032), il faut donc des dispositions toutes faites |
| C. Des places tirées d'une graine | `Scatter(count: 36, seed: 7, children: [ … ])` (écriture proposée) | `Math.random()`, jamais deux fois pareil ; ou une suite de hasard écrite à la main | `Random(seed)` | même graine, même film | l'auteur ne choisit plus chaque place |
| D. Le moteur, sans mot nouveau | une seule `@keyframes` par sorte de mouvement, et les valeurs de chaque copie en variables CSS | c'est ce que fait le jumeau (`--x`, `--y`, `--d`…) | — | allège la page fabriquée | du travail dans `moteur/src/movement.rs` ; vérifier que les leçons 32 à 34 ne changent pas à l'œil |

### Recommandation

A tout de suite : réécrire la scène 1 du film avec `Repeat` (c'est déjà possible), et corriger le README et ADR-034. Puis D, qui travaille sur le poids sans mot nouveau. B ou C seulement si un besoin revient (un motif en cercle) ; pour la poussière, les particules (partie 2) feront mieux.

### Ce qui comblerait l'écart avec le jumeau

- En écriture : **l'écart est déjà comblé** (140 lignes contre 184, mesuré).
- En poids : D. Estimation : les animations brutes passeraient d'environ 24 Ko à 10 Ko. Mais Brotli compresse déjà très bien ces répétitions : le gain envoyé serait modeste, de l'ordre de 1 à 2 Ko sur 12 (estimation). Le reste de l'écart est la porte d'entrée, qui porte des fonctions que le jumeau n'a pas.

## 2. Les particules

### État vérifié

- Rien dans le langage. Le jumeau n'en a pas non plus : ses 36 éclats sont des boîtes animées en CSS, comme dans HoloCode.
- La version « max » en a (`exemples/motion/web/showreel-max.html:155-233`) : 520 particules sur un canevas, avec une vitesse, un frottement, une gravité, une durée de vie et des traînées ; et 220 étoiles qui scintillent et suivent un peu la souris.
- Avec ce qui existe, on peut écrire 200 grains par `Repeat` et `Shape`. Mesuré : `holo check` répond « ok » ; 115 536 octets de HTML fabriqué (6 779 compressés), 400 `@keyframes`, 4 boîtes par grain. Les grains vont en ligne droite, avec une courbe : ni gravité, ni frottement. Et une répétition s'arrête à 200 éléments (`repeat.rs:26`) ; on peut en poser plusieurs, jusqu'à 20 000 blocs par page (`repeat.rs:28`).
- ADR-035 dit que `requestAnimationFrame` (dessiner image par image) est remplacé par `Enter`, `Loop`, `Scenes`, `Every`. Pour des particules, ce n'est pas vrai : c'est une dette au sens d'ADR-035.

### Options

| Option | Écriture HoloCode | Équivalent web | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Un bloc de particules | `Particles(height: 240px, count: 520, colors: […], speed: 400px, gravity: 300px, life: 2s, seed: 7)` (écriture proposée), dessiné sur un canevas par un petit script chargé seulement s'il sert | un `<canvas>` et 30 à 50 lignes de JavaScript (`requestAnimationFrame`) | `CustomPainter` et un `Ticker` ; paquet `confetti` | des centaines de grains pour une ligne ; une graine donne le même éclat | un mot nouveau ; un script de plus |
| B. Ce qui existe | `Repeat` et `Shape` avec `Enter` et `Loop` | des `div` animées en CSS | des widgets animés | aucun mot nouveau | lourd (mesuré plus haut) ; pas de physique ; 200 par répétition |
| C. Les points du moteur | un `Point(fragments:)` qui éclate | WebGL ou WebGPU | Flutter GPU (expérimental) | des centaines de milliers de points | fait venir le dessin (633 554 octets mesurés) pour un effet de page à plat |

### Recommandation

A, après une mesure sur téléphone (cadence et batterie à 120, 300 et 520 grains). Garder B, documenté, pour quelques dizaines d'éclats décoratifs.

### Ce qui comblerait l'écart

- Avec le jumeau : rien à combler, il n'en a pas.
- Avec la version « max » : A, avec `count: 520`, `gravity:` et `life:`. Les étoiles qui suivent la souris : A, plus `follow: pointer` (partie 4). Les traînées : un réglage de plus, plus tard (non proposé ici).

## 3. Les chemins vectoriels, et la forme qui se change

### État vérifié

- Quatre formes seulement : `circle`, `square`, `triangle`, `diamond` (`moteur/src/flat.rs:929-930`), découpées en CSS (`flat.rs:64-70`).
- Une forme qui change : seulement `round`, un carré qui s'arrondit (`moteur/src/movement.rs:80-82`). C'est ce que fait la scène 4 du film, et le jumeau fait pareil (`border-radius`).
- Pas de trajet à suivre, pas de chemin dessiné (GUIDE.md:1513). Le SVG n'entre que comme image : elle peut bouger en entier (`Enter`, `Loop`), mais son dessin ne change pas. ADR-032 a écarté le SVG écrit par l'auteur, car on peut y glisser du code.
- La version « max » change un carré en rond, en goutte, puis en cœur (`showreel-max.html:260-273`, l'animation de `d`), et dessine les courbes du laboratoire trait après trait (`showreel-max.html:274-295`, `stroke-dashoffset`).
- Mesuré dans Chrome 154 (bureau) : le navigateur passe bien d'un polygone de 32 sommets à un autre (à mi-chemin, un polygone nouveau ; 444 octets par polygone). `offset-path: path()` place bien un objet sur un trajet. `offset-path: circle()` est reconnu, mais mon essai l'a laissé au coin : non concluant.

### Options

| Option | Écriture HoloCode | Équivalent web | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Changer de forme nommée | `Loop(form: heart)`, avec des formes de plus : `star`, `heart`, `hexagon`, `ring` (écriture proposée) ; chaque forme devient un polygone de 32 sommets | des `@keyframes` de `clip-path: polygon(…)`, les sommets calculés à la main | `ShapeBorder.lerp` | tout en CSS, sans moteur dans le navigateur ; mesuré possible | seulement entre formes nommées |
| B. Un chemin libre | `Shape(path: "M 50 90 C …")`, et le changer : `Loop(path: "M …")` (écriture proposée) | `<svg><path d>`, et l'animation de `d` comme la version « max » | `CustomPaint` et `Path` | toute forme | pour changer de chemin, les deux doivent avoir les mêmes commandes : le moteur doit les ré-échantillonner ; gros travail |
| C. Un chemin qui se dessine | `Shape(path: "…", line: 2px, enter: Enter(draw: 0))` (écriture proposée) | `pathLength="1"`, `stroke-dasharray`, et l'animation de `stroke-dashoffset` | `PathMetric.extractPath` | le laboratoire des courbes de la version « max » | deux mots ; il faut un trait, pas un remplissage |
| D. Un trajet | `Loop(along: circle)` ou `Loop(along: "M …")` (écriture proposée) | `offset-path` et `offset-distance` | `PathMetric.getTangentForOffset` | un mot | en `path()`, le trajet compte en pixels et ne suit pas la taille du texte choisie par le visiteur (ADR-061) |

### Recommandation

A d'abord : peu cher, tout en CSS, mesuré possible. Puis la partie fixe de B (`Shape(path:)` sans animation) et C, pour le laboratoire. D ensuite. Changer un chemin libre en un autre (B animé) en dernier, ou jamais.

### Ce qui comblerait l'écart

- Avec le jumeau : rien, sa forme qui change est le même `border-radius`.
- Avec la version « max » : A (`form: circle`, puis `heart`) pour le cœur ; C pour les courbes qui se dessinent ; D pour un objet qui suit une courbe.

## 4. Les réactions au doigt (et au pointeur)

### État vérifié

- Ce qui existe : `tap`, `hover`, `hoverEnd` (`moteur/src/rules.rs:10-22`), aussi au doigt et au clavier (ADR-039) ; glisser un bloc sur un plateau (`moteur/web/page-engine.js:1375-1402`) ; les états `hover:` et `active:` d'un style (ADR-036).
- Ni le film ni le jumeau ne réagissent au doigt. La version « max » oui : le bouton « Engagez-moi » suit un peu la souris (`showreel-max.html:311-317`), le plateau penche avec elle (`showreel-max.html:320-321`), les étoiles se décalent (`showreel-max.html:193`).
- **Défaut trouvé : le pincement est cassé sur téléphone.** La traduction du code en anglais (commit `172931b`, ADR-060, le 2026-10-07) a changé `event.touches` (les doigts posés) en `event.keypresses`, qui n'existe pas. Six endroits : `moteur/web/page.html:219` et `moteur/web/page-engine.js:872-891`. Mesuré dans Chrome sans fenêtre : pincer la page légère lève une erreur (« TypeError … reading 'length' ») et ne fait pas venir le moteur ; avec le moteur arrivé, même erreur à `page-engine.js:879`. D'après la lecture du code, le bouton « Vue points » du menu (un clic) et le pincement dans la vue points (en Rust, avec des évènements de pointeur, `moteur/src/web.rs:243-330`) ne passent pas par ces lignes ; je ne les ai pas essayés. Aucun test ne pince la page : rien ne l'a vu. Et comme la page interdit au navigateur son propre pincement (`touch-action: pan-x pan-y`, `page.html:20-22`), deux doigts ne grossissent plus rien sur un téléphone, même pour lire (d'après la lecture du code ; non essayé sur un téléphone).

### Options

| Option | Écriture HoloCode | Équivalent web | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Suivre le pointeur | `Loop(x: 12px, flip: 8deg, follow: pointer)` : la pose est atteinte quand le pointeur est au bord de l'écran, le repos au centre ; au doigt, seulement pendant le toucher (écriture proposée) | `pointermove` en JavaScript, des variables CSS, une transition | `MouseRegion(onHover:)` et `Transform` | quelques lignes dans la page légère, sans le moteur ; ne touche aucune valeur | un mot ; l'effet « aimant » (suivre le pointeur **près** du bloc) demanderait une variante |
| B. Des valeurs pour les règles | `pointerX`, `pointerY`, comme `hour` (écriture proposée) | `pointermove` et du code | `setState` à chaque mouvement | un jeu peut s'en servir | chaque mouvement passe par l'arbitre : lourd, et une valeur qui change sans cesse |
| C. Ce qui existe | `hover:` et `active:` dans un style ; `On(Bouton.hover, …)` ; `drag` sur un plateau | `:hover`, `mouseenter`, `pointermove` | `MouseRegion`, `Draggable` | rien à faire | ni parallaxe, ni aimant |
| D. Pencher le téléphone | `follow: tilt` (écriture proposée), d'après le gyroscope | `deviceorientation` (une permission à demander sur iPhone) | paquet `sensors_plus` | l'équivalent de la souris sur un téléphone | permission ; batterie ; non vérifié ici |

### Recommandation

Corriger le pincement d'abord, avec un essai qui pince la page dans Chrome sans fenêtre. Puis A. B seulement pour un jeu qui en aurait besoin. D plus tard, après un essai sur téléphone.

### Ce qui comblerait l'écart

- Avec le jumeau : rien, il ne réagit pas.
- Avec la version « max » : A pour le plateau qui penche et les étoiles ; la variante « aimant » de A pour le bouton.

## Deux défauts trouvés en chemin

1. **Le pincement cassé** (partie 4). C'est le geste du métavers sur téléphone. À corriger avant les mesures sur le téléphone de Yocthan.
2. **Les titres lus lettre par lettre.** `letters:` met chaque lettre dans sa boîte (`moteur/src/movement.rs:312-362`) sans rien pour le lecteur d'écran. Mesuré dans l'arbre d'accessibilité de Chrome, pour le film HoloCode **et** pour le jumeau : « C L A U D E », « M o t i o n d e s i g n e r » (l'espace entre les mots est même perdu), « A n i m e r . », « H o l o v e r s e ». Correction sans mot nouveau : le moteur écrit le texte entier pour le lecteur d'écran et cache les boîtes de lettres. Ce serait un point où HoloCode fait mieux que le web pour tous les auteurs, sans qu'ils y pensent (comme le mouvement réduit). La galerie du site de référence a le même titre coupé (`exemples/site-reference/galerie.holo`).

## Ordre proposé pour le duel

1. Corriger le pincement (hors film, mais c'est « au doigt »).
2. Corriger la lecture des lettres (quatre titres du film).
3. Réécrire la scène 1 avec `Repeat` ; corriger le README du duel et ADR-034.
4. Une seule `@keyframes` par sorte de mouvement (le poids).
5. Changer de forme (`form:` dans `Loop`), avec `star` et `heart`.
6. `follow: pointer`.
7. `Particles`, après une mesure sur téléphone.
8. `Shape(path:)`, le chemin qui se dessine, le trajet.

Quand 5 à 8 existeront, refaire un **second duel** contre `showreel-max.html`, avec un nouveau jumeau exact. Le jumeau actuel reste juste pour le film actuel.

## Accessibilité, déterminisme, budgets

- Tout ce qui est décoratif est caché au lecteur d'écran ; un texte qui bouge reste un texte entier. Qui demande moins de mouvement voit tout immobile, particules et pointeur compris (mesuré aujourd'hui pour le film : 0 animation).
- Une graine donne le même éclat ; la place d'un grain vient du temps écoulé, par une formule, pas d'un calcul pas à pas. Le pointeur ne change que l'apparence, jamais une valeur de la page.
- Un plafond de grains à fixer après mesure sur téléphone ; un canevas à la densité 2 au plus ; aucun effet ne fait venir le dessin d'une page à plat. Cadence et batterie sur téléphone : non mesurées (pas d'appareil ici).

## Mesures (commandes et sorties)

Toutes faites sur le PC de Yocthan, le 2026-10-07, avec `moteur/target/release/holo.exe` (le fichier est daté du 2026-10-07 à 13:56, relevé à la relecture : sans doute construit après la fusion de la PR #139 (13:54, commit `1119361`), qui ne change que la vérification d'un fichier de thème seul ; les lignes du moteur citées ici sont les mêmes qu'à `7a48def`) et Chrome 154.0.8037.95 sans fenêtre (SwiftShader, sans carte graphique). **Aucune n'est une mesure de téléphone.** Les fichiers d'essai sont dans `scratchpad/exploration/essais/`. Le serveur de Yocthan (port 8080) n'a été lu qu'en lecture ; les essais à moi ont lancé un second serveur du dépôt (ports 8093 à 8097), arrêté à la fin de chaque essai.

1. Les éclats par `Repeat` :
   `node` (réécriture de la scène 1 dans `showreel-repeat.holo`), puis `cat showreel-repeat.holo | holo.exe check -` → `ok` ; `holo.exe html showreel-repeat.holo /exemples/motion/holocode/ > showreel-repeat-flat.html` et `cmp showreel-lf-flat.html showreel-repeat-flat.html` → `PAGE IDENTIQUE octet pour octet` ; `wc -c` → `14193 showreel-lf.holo`, `10975 showreel-repeat.holo` ; lignes utiles → 206 et 140.
2. Le poids envoyé :
   `curl -s -H "Accept: text/html" -H "Accept-Encoding: br" … /exemples/motion/holocode/showreel.holo` → `br=12053 brut=68370` ; `… /exemples/motion/web/showreel.html` → `br=3849 brut=21586` ; `… /page.html` → `br=5578`. Recompressé par `node` (Brotli 11) : `showreel-servi.html brotli q11 10878`.
3. Les animations de la page fabriquée :
   `grep -o '@keyframes' showreel-flat.html | wc -l` → 109 (jumeau : 5) ; `node` → `octets des @keyframes hm 17228 ; des règles .hm 6911 ; sur 49839` (49 839 est le nombre de caractères ; la page fait 49 843 octets : remesuré à la relecture, 17 228 + 6 911 = 24 139 octets sur 49 843).
4. 200 grains écrits avec `Repeat` :
   `cat particules-200.holo | holo.exe check -` → `ok` ; `holo.exe html particules-200.holo > particules-200.html` ; `wc -c` → `115536` ; Brotli 11 → `6779` ; `@keyframes` → 400.
5. Ce que Chrome sait faire (`node soutien.mjs`) → `offsetPath: true`, `clipPathPath: true`, `clipPathShape: true`, `dPath: true`, `animationTimelineView: true`, `animationTimelineScroll: true` ; `morphPolygone: { octetsDUnPolygone32: 444, interpole: true }` ; `sizesAuto: { affichee: 155, choisie: "1280w" }` (donc `sizes="auto"` non concluant).
6. Le trajet (`node soutien3.mjs`) → `{"cercle":"0,0","cerclePositionne":"0,0","chemin":"200,100","cheminPositionne":"200,100"}`.
7. Les lettres et le mouvement réduit (`node mouvement-a11y.mjs`, sur le serveur 8080) → HoloCode : `["C L A U D E","M o t i o n d e s i g n e r","Penser.","Concevoir.","A n i m e r .","H o l o v e r s e"]` ; jumeau : la même liste ; moins de mouvement : `{"animationsEnCours":0,"scenesVisibles":1,"derniereVisible":"visible/1"}`.
8. Le pincement (`node pincer.mjs`) → page légère : `"TypeError: Cannot read properties of undefined (reading 'length') | at pinchZoom (…/pincer.holo:219:57)"` (deux fois), `moteur: false`. Puis `node pincer2.mjs` (moteur arrivé) → `écouteurs sur window : {"touchstart":1,"touchmove":1,"touchend":1,"wheel":3}` ; un toucher à deux doigts fabriqué dans la page → `"Uncaught TypeError: Cannot read properties of undefined (reading 'length') @page-engine.js:879"`, `touchesExiste: true`, `keypressesExiste: false`. L'historique : `git show 6acbaf2:moteur/web/page-moteur.js` → `evenement.touches.length === 2` ; depuis `172931b` → `event.keypresses.length === 2`.
9. Le jumeau de l'exemple de la piste 9 (`node jumeau-09.mjs`) → `erreurs: []`, `titreEntendu: ["L'atelier des mondes"]`, `pixelsDePoussiereA05s: 5614`, `formeSommets: 32`, `luneBouge: true`, `defile: "view()"` ; moins de mouvement → `animationsEnCours: 0`, `poussiere: "none"`, `penche: "au repos"`, `texteVisible: "1"`.
10. Le dessin du moteur : `curl -s -H "Accept-Encoding: br" -o /dev/null -w "%{size_download}" http://localhost:8080/pkg/holo_engine_bg.wasm` → `633554` ; le moteur léger `…/pkg-light/holo_engine_bg.wasm` → `156511`. Ces fichiers avaient été reconstruits le même jour à 13:56 : la taille peut encore bouger.
11. *(Relecture, mêmes outils, fichiers dans `scratchpad/exploration/relecture-8-9-duel/`.)* `git show 7a48def:exemples/motion/holocode/showreel.holo > showreel-7a48def.holo`, puis `cat … | holo.exe check -` → `ok` pour lui et pour `showreel-repeat.holo` ; `holo.exe html … /exemples/motion/holocode/` pour les deux, puis `cmp` → aucune différence, 49 843 octets chacune ; `grep -o '@keyframes' | wc -l` → 109. `curl` sur le serveur 8080 (lecture) : `page.html` 5 578, `showreel.holo` 12 053, `showreel.html` 3 849, `/pkg/` 633 554, `/pkg-light/` 156 511. La page servie (68 370 octets) recompressée par `node` : Brotli 11 → 10 878, Brotli 5 → 12 053.

Relu le 2026-10-07 : 9 corrections.
