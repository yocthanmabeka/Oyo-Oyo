# Piste 9 — Graphismes avancés

> Statut : EXPLORATION. Avis de Claude, pas une décision.
>
> **Réparé le 2026-10-07, après cette exploration** : le pincement au doigt (PR 141). Vérifié dans Chrome sans fenêtre, avant et après.

## Ce que Codex demandait

Sa ligne, dans l'issue #82 (texte transmis par Yocthan le 2026-10-06) :

> | **9** | **Graphismes avancés** : chemins vectoriels, transformation des formes, particules, mouvement lié au défilement ou au pointeur, objets 3D pleins. | Rivaliser avec les démonstrations visuelles les plus ambitieuses. |

## État vérifié (main, 7a48def, 2026-10-07)

**En bref : la piste est faite au quart.** Le mouvement fabriqué en CSS est solide (ADR-034). Une forme peut s'arrondir. Un bloc peut entrer quand il arrive à l'écran. Tout le reste manque : chemins, formes qui se changent en une autre, particules, mouvement qui suit le défilement ou le pointeur, objets pleins. En vérifiant, j'ai trouvé **deux défauts** : le pincement au doigt est cassé depuis la traduction du code en anglais, et un titre coupé en lettres est lu lettre par lettre.

### Ce qui existe (vérifié dans le code)

| Notion | Écriture | Où dans le moteur | Décision, leçon |
|---|---|---|---|
| Le mouvement | `Enter`, `Loop`, `Scenes`, `Scene` ; 10 propriétés, 7 courbes, `letters`, `each` | `moteur/src/movement.rs:35-46` (propriétés), `24-32` (courbes), `276-299` (le CSS fabriqué) | ADR-034 ; leçons 32 à 34 |
| Une forme qui s'arrondit | `round: 50` donne `border-radius` | `movement.rs:80-82` | ADR-034 |
| Quatre formes | `Shape(form: circle \| square \| triangle \| diamond)` : le rond par `border-radius: 50%` (`flat.rs:66`), le triangle et le losange par `clip-path: polygon` (`flat.rs:69-70`), le carré tel quel | `moteur/src/flat.rs:924-948` et `flat.rs:64-70` | ADR-032 ; leçon 30 |
| La profondeur d'un bloc | `flip`, `tilt` donnent `perspective(800px) rotateX/rotateY` | `movement.rs:74-76` | ADR-034 |
| Entrer en arrivant à l'écran | `Enter(…, inView: true)` | `movement.rs:179-182` et `410-412` ; `moteur/web/page.html:146-164` (un `IntersectionObserver`) | ADR-061 ; leçon 78 |
| Moins de mouvement | rien à écrire | `movement.rs:451`. Mesuré sur le film du duel : 0 animation en cours, seule la dernière scène visible | ADR-034 |
| Répéter un modèle | `Repeat(items:)` porte des nombres jusque dans `x`, `y`, `Enter`, `Loop` | `moteur/src/repeat.rs:250-260`. Mesuré : les 36 éclats du duel réécrits ainsi donnent **la même page, octet pour octet** | ADR-040 ; leçon 49 |
| Le doigt | `tap`, `hover`, `hoverEnd`, `drag` sur un `Board`, le pincement de la page (cassé depuis ADR-060 : voir plus bas) | `moteur/src/rules.rs:10-22` ; `moteur/web/page-engine.js:1375-1402` (glisser) et `869-892` (pincer) | ADR-026, ADR-028, ADR-039 |
| Des points dessinés | le moteur de dessin (WebGPU ou WebGL 2) | `/pkg/` : 633 554 octets transférés (mesuré) | ADR-010, ADR-053 |
| La 3D pleine | direction décidée, rien de construit | `docs/04-roadmap/PLAN-3D.md` : 12 étapes, environ 15 séances (estimation de ce document) | ADR-048, ADR-049 |

### Ce qui manque encore (vérifié : absent du code)

1. **Les chemins vectoriels.** `Shape` refuse toute forme hors des quatre (`flat.rs:929-930`). Le SVG n'entre que comme fichier d'image (`Image(source: "x.svg")`) : l'image peut bouger en entier (`Enter`, `Loop` : vérifié avec `holo check`), mais son dessin ne change pas, ni ses traits ni ses couleurs. ADR-032 a écarté le SVG écrit par l'auteur : « un langage entier, où l'on peut glisser du code ». Pas de trajet à suivre non plus (GUIDE.md:1513).
2. **La transformation des formes.** Seulement `round` : un carré qui s'arrondit. Rien pour passer d'un losange à une étoile.
3. **Les particules.** Rien. Avec ce qui existe, 200 « particules » s'écrivent par `Repeat` : mesuré, 115 536 octets de HTML fabriqué, 400 `@keyframes`, 4 boîtes par particule, et aucune physique (ni gravité, ni frottement).
4. **Le défilement.** `inView` déclenche une entrée, une fois. La position du défilement ne pilote rien (TABLEAU-WEB.md:610).
5. **Le pointeur.** Aucune position du pointeur. `hover` dit seulement « dessus » ou « pas dessus ». Glisser n'existe que sur un plateau.
6. **Les objets 3D pleins.** Rien de construit. Le planning attend le feu vert de Yocthan.

### Deux défauts trouvés en chemin, et un troisième à la relecture (hors langage, à corriger)

- **Le pincement est cassé sur téléphone depuis le 2026-10-07.** La traduction du code (commit `172931b`, ADR-060) a changé `event.touches` (la liste des doigts posés) en `event.keypresses`, qui n'existe pas. Six endroits : `moteur/web/page.html:219` et `moteur/web/page-engine.js:872-891`. Mesuré dans Chrome sans fenêtre, en taille de téléphone : pincer la page légère lève « TypeError: Cannot read properties of undefined (reading 'length') » et ne fait pas venir le moteur ; avec le moteur arrivé, la même erreur part de `page-engine.js:879`. Aucun test ne touche la page au doigt : rien ne l'a vu (vérifié : aucun fichier suivi du dépôt, hors `page.html` et `page-engine.js`, ne fabrique d'évènement `touch`). Conséquence, d'après la lecture du code : la page interdit au navigateur son propre pincement (`touch-action: pan-x pan-y`, `page.html:20-22`) ; sur un téléphone, deux doigts ne grossissent donc plus rien, même pour lire un texte trop petit (un enjeu d'accessibilité) ; non essayé sur un téléphone. D'après la lecture du code, le bouton « Vue points » du menu (un clic) et le pincement dans la vue points (écrit en Rust, avec des évènements de pointeur, `moteur/src/web.rs:243-330`) ne passent pas par ces lignes ; je ne les ai pas essayés.
- **Un titre coupé en lettres se lit lettre par lettre.** `letters:` met chaque lettre dans sa boîte (`movement.rs:312-362`), sans rien pour le lecteur d'écran. Mesuré dans l'arbre d'accessibilité de Chrome : « C L A U D E », « M o t i o n d e s i g n e r » (l'espace entre les deux mots est même perdu). Le jumeau web a le même défaut. La galerie du site de référence aussi (`exemples/site-reference/galerie.holo`, le titre « L'atelier des mondes »). L'audit axe-core d'ADR-055 ne l'a pas signalé : ses résultats du 2026-10-06 disent « 0 défaut » sur tous les sites d'exemple, et la galerie existait déjà (je ne l'ai pas relancé).
- *(Ajouté à la relecture, d'après la lecture du code ; non essayé dans un navigateur.)* **Le carrefour en diagonale ne descend plus.** La même traduction (commit `172931b`) a renommé la classe `diagonale` en `diagonal` dans le script (`page-engine.js:197` et `994`) et dans la première règle (`page.html:74`), mais pas dans la seconde : `#crossroads.diagonale .portal` (`page.html:75`) ne vise plus rien. Avec `Portals(layout: diagonal)`, les portails ne descendent donc plus d'un cran chacun. Avant la traduction, les deux règles disaient `diagonale` (`git show 6acbaf2:moteur/web/page.html`, lignes 70-71).

### Comment c'est mesuré

Sur le PC de Yocthan, le 2026-10-07, avec Chrome 154.0.8037.95 sans fenêtre et `moteur/target/release/holo.exe`. Le détail des commandes et de leurs sorties est dans `duel-motion.md` (partie « Mesures », points 1 à 9). Les principales :

- `cat particules-200.holo | holo.exe check -` → `ok` ; `holo.exe html particules-200.holo` → 115 536 octets, 400 `@keyframes`.
- `node pincer.mjs` → `TypeError: Cannot read properties of undefined (reading 'length') | at pinchZoom (…:219:57)` ; `node pincer2.mjs` → `Uncaught TypeError … @page-engine.js:879`, `keypressesExiste: false`.
- `node mouvement-a11y.mjs` → `"C L A U D E"`, `"M o t i o n d e s i g n e r"` (HoloCode et jumeau) ; moins de mouvement → `animationsEnCours: 0`.
- `node soutien.mjs` → `animationTimelineView: true`, `offsetPath: true`, `morphPolygone: { octetsDUnPolygone32: 444, interpole: true }`.
- `curl -s -H "Accept-Encoding: br" -o /dev/null -w "%{size_download}" http://localhost:8080/pkg/holo_engine_bg.wasm` → `633554`.

### Les documents en retard

- `docs/02-gouvernance/adr/ADR-034-mouvement.md:68-69` : « Les tailles de texte sont fixes » et « Sans listes répétées, 36 éclats s'écrivent un par un » sont dépassés (ADR-036, ADR-061, ADR-040). Ligne 62 : « 10 Ko téléchargés pour le film HoloCode » ; mesuré aujourd'hui, 12 053 octets.
- `exemples/motion/README.md:30-32` : le défaut « le film repart de zéro » est corrigé (`exemples/site-reference/RECETTE-2026-10-06.md`, cas F14 « le film continue quand le moteur arrive »). Lignes 17 et 26 : « 10 Ko » ; mesuré aujourd'hui, 12 053 octets.
- `docs/01-holocode/NOMS.md:210` dit encore `h4`, `h5`, `h6` « refusés exprès » (ils existent depuis ADR-036).
- `docs/01-holocode/TABLEAU-WEB.md:596-600`, `610`, `661-662` : justes. Mais la ligne 612 (« pincer, zoomer », « Oui », 100 %) est fausse tant que le pincement reste cassé.

## Le scénario du site de référence

La galerie animée existe (`exemples/site-reference/galerie.holo`) : une entrée de titre, une boucle et trois scènes. L'extension « objets 3D pleins » du cahier (`proposals/GPT5.6/site-reference-2026-10-06/README.md:134`) est `BLOQUÉ` dans le premier passage de recette (`RECETTE-2026-10-06.md`, cas X01 à X04).

**Le scénario** : on ouvre la galerie. Une poussière d'or éclate une fois. Le rond doré de la scène 3 devient une étoile, et revient. Une petite lune tourne autour d'un disque pâle. Le tableau de la scène 2 penche vers le doigt ou la souris. Sur la fiche, le texte arrive au rythme du défilement. Le titre se lit en entier au lecteur d'écran. Qui demande moins de mouvement voit une page immobile et complète.

Ce qu'il faut pour cela : des particules, des formes qui se changent l'une en l'autre, un trajet, un mouvement lié au pointeur et au défilement, et la correction des lettres. Puis, pour l'extension du cahier, trois objets pleins (le PLAN-3D).

## Options comparées

### Manque 1 : les chemins vectoriels, et le trajet à suivre

| Option | Écriture HoloCode | HTML, CSS, JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Un chemin libre | `Shape(path: "M 50 90 C 20 65 …", color:, size:)` (écriture proposée) ; la grammaire des chemins SVG, vérifiée par le moteur | `<svg viewBox><path d="…"></svg>`, ou CSS `clip-path: path()` | `CustomPaint` et `Path` ; `ClipPath` ; le paquet `flutter_svg` | toute forme, un logo ; copié depuis Figma ou Inkscape | illisible pour un débutant ; il faut borner (nombre de commandes) |
| B. Plus de formes nommées | `form: star \| heart \| hexagon \| ring \| arrow` (écriture proposée) | `clip-path: polygon(…)` écrit à la main | `StarBorder`, `CircleBorder` | se lit comme une phrase ; quelques octets | jamais un logo |
| C. Un fichier SVG lu comme des données | `Shape(source: "logo.svg")` (écriture proposée) : le moteur ne garde que les `path` et leurs couleurs, refuse le reste | `<img src="logo.svg">`, ou le SVG collé dans la page | `SvgPicture.asset` | l'auteur dessine dans son outil | un lecteur de SVG dans le moteur ; ce qui est refusé doit être expliqué |
| D. Ce qui existe | `Image(source: "logo.svg", alt:)` | `<img src>` | `Image.asset` | rien à faire | le dessin ne change pas (l'image entière peut bouger) ; pas de couleur du thème |
| Trajet T1 | `Loop(along: circle)` ou `Loop(along: "M …")` (écriture proposée) | CSS `offset-path: path(…)` et `offset-distance` | `PathMetric.getTangentForOffset`, à la main | un mot ; tout en CSS | un trajet ne suit pas la taille du texte du visiteur (voir plus bas) |
| Trajet T2 | `Loop(along: Orbit)`, `Orbit` étant une forme nommée (écriture proposée) | pareil | pareil | pas de chemin recopié | aujourd'hui, une forme nommée devient un bouton (`flat.rs:944-946`) : un faux bouton pour le lecteur d'écran |

Mesuré dans Chrome 154 (bureau) : `offset-path: path()` place bien la lune (à 25 % d'un cercle de 200 px, son centre est au point le plus à droite, 200,100). `offset-path: circle()` est reconnu, mais mon essai l'a laissé au coin (0,0) : non concluant. Le moteur doit donc écrire le trajet lui-même, en `path()`. Or `path()` compte en pixels, et le moteur écrit les tailles en `rem` (ADR-061) : si le visiteur grossit son texte, la boîte grandit et le trajet non. `shape()`, qui accepte des pourcentages, est reconnu par Chrome 154 dans `clip-path` (mesuré) ; dans `offset-path`, et son comportement : non vérifiés.

### Manque 2 : une forme qui se change en une autre

| Option | Écriture HoloCode | HTML, CSS, JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Changer de forme nommée | `Loop(form: star)`, `Enter(form: circle)` (écriture proposée) ; chaque forme devient un polygone de 32 sommets | `clip-path: polygon(…)` dans des `@keyframes`, les sommets calculés à la main | `ShapeBorder.lerp`, `AnimatedContainer` | tout en CSS, sans moteur dans le navigateur. Mesuré dans Chrome 154 : le navigateur passe bien d'un polygone de 32 sommets à un autre (à mi-chemin, un polygone nouveau) ; 444 octets par polygone | seulement entre formes nommées |
| B. Entre deux chemins libres | `Loop(path: "M …")` (écriture proposée) | `d: path()` en CSS, ou `element.animate({ d })`, comme `showreel-max.html` | à la main | le cœur du film « max » | les deux chemins doivent avoir les mêmes commandes : le moteur doit les ré-échantillonner ; gros travail |
| C. Ce qui existe | `round` | `border-radius` | `BorderRadius.lerp` | rien à faire | un carré qui s'arrondit, rien d'autre |

### Manque 3 : les particules

| Option | Écriture HoloCode | HTML, CSS, JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Un bloc de particules | `Particles(height:, count:, colors:, speed:, gravity:, life:, seed:)` (écriture proposée), dessiné sur un canevas par un petit script chargé seulement s'il sert | un `<canvas>` et 30 à 50 lignes de JavaScript (`showreel-max.html:155-233` : 520 particules) | `CustomPainter` et un `Ticker` ; paquets `confetti` | des centaines de grains pour quelques octets de fichier ; une graine donne le même éclat | un mot nouveau et ses réglages ; du JavaScript de plus dans le moteur |
| B. Ce qui existe : `Repeat` et `Shape` | 200 `Item(…)` et un modèle `Shape` avec `Enter` et `Loop` | des `div` animées en CSS | des widgets animés | aucun mot nouveau. Mesuré : `holo check` accepte | mesuré : 115 536 octets de HTML (6 779 compressés), 400 `@keyframes`, 4 boîtes par grain ; pas de gravité ; 200 éléments par répétition (`repeat.rs:26`), plusieurs répétitions possibles, 20 000 blocs par page au plus (`repeat.rs:28`) |
| C. Les points du moteur | un `Point(fragments:)` qui éclate (existe, dans la vue en profondeur) | WebGL ou WebGPU | Flutter GPU (expérimental) | des centaines de milliers de points, déjà là | télécharge le dessin (633 554 octets mesurés) pour un effet décoratif d'une page à plat |
| D. Un module enfermé | `Module(…)` (ADR-045) | un Worker | un isolate | aucun mot nouveau | un module n'échange qu'un nombre contre un nombre (GUIDE.md:1800) : il ne dessine rien |

### Manques 4 et 5 : le défilement et le pointeur

| Option | Écriture HoloCode | HTML, CSS, JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Un mot, `follow:` | `Enter(y: 80px, opacity: 0, follow: scroll)` : l'entrée avance avec le défilement. `Loop(x: 12px, flip: 8deg, follow: pointer)` : la pose est atteinte quand le pointeur est au bord, le repos au centre (écriture proposée) | défilement : CSS `animation-timeline: view()` et `animation-range` ; pointeur : `pointermove` en JavaScript et des variables CSS | `ScrollController` et `AnimatedBuilder` ; la recette « parallax » avec `Flow` ; `MouseRegion(onHover:)` et `Transform` | un seul mot pour deux idées ; le défilement reste en CSS pur (mesuré : `animation-timeline: view()` et `scroll()` reconnus par Chrome 154) ; le pointeur demande quelques lignes dans la page légère, pas le moteur | Safari et Firefox : non vérifiés ; sans prise en charge, le bloc doit rester visible |
| B. Des valeurs pour les règles | `pointerX`, `pointerY`, `scrollY`, comme `hour` (écriture proposée), employées dans `If`, `When` | `pointermove`, `scroll`, et du code | `setState` à chaque mouvement | puissant : un jeu peut s'en servir | chaque mouvement passe par l'arbitre : lourd ; une valeur qui change sans cesse rend la page difficile à prévoir |
| C. Ce qui existe | `inView`, `hover`, `drag` sur un plateau | `IntersectionObserver`, `mouseenter`, `pointermove` | `VisibilityDetector`, `MouseRegion`, `Draggable` | rien à faire | pas de parallaxe, pas d'aimant, pas de suivi |

À savoir pour l'option A *(ajouté à la relecture)* : ADR-061 a déjà comparé `animation-timeline: view()` pour « apparaître en descendant », et l'a écarté : « le CSS seul ne marche pas encore partout » (`ADR-061-lot-9-fin-du-web.md:29`). `follow: scroll` devra dire ce qui a changé depuis, ou garder un repli visible (R9-6).

### Manque 6 : les objets 3D pleins

| Option | Écriture HoloCode | HTML, CSS, JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Le PLAN-3D | `Model(source: "chaise.holo3d")`, choisi seulement à l'étape 11 (`PLAN-3D.md`) | three.js, Babylon.js, `<model-viewer>` | paquet `flutter_scene` (expérimental) | décidé (ADR-049) ; profondeur, matières, niveaux de détail | environ 15 séances (estimation du PLAN-3D) ; attend le feu vert |
| B. Un cube en CSS 3D, en attendant | `Shape(form: cube)` (écriture proposée) | six faces et `transform-style: preserve-3d` (`showreel-max.html`, scène « cube ») | `Transform` et `Matrix4` | rapide ; égale le film « max » | une impasse : ni lumière, ni objets qui se cachent l'un l'autre ; contraire à l'esprit d'ADR-048 et d'ADR-049 |

### Noms (ADR-016)

| Nom proposé | Son sens sur le web | Dans Flutter et ailleurs | Risque de confusion |
|---|---|---|---|
| `path` | `<path d>` en SVG, `path()` en CSS : un chemin dessiné, le même sens | `Path` (dessin) | moyen : pour un programmeur, `path` est aussi le chemin d'un fichier ; HoloCode dit `source:` pour les fichiers, ce qui aide |
| `along` | aucun mot ; l'idée est `offset-path` en CSS, `<animateMotion>` en SVG | aucun | faible ; se lit comme une phrase |
| `form` dans `Enter` et `Loop` | en HTML, `<form>` est un formulaire | — | moyen : déjà accepté pour `Shape(form:)` (ADR-032), même sens qu'avant dans HoloCode ; mais HoloCode a aussi, depuis, le bloc `Form`, un formulaire (ADR-042) : `form:` (une forme) et `Form` se côtoient déjà, et l'ajout à `Enter` et `Loop` les rapproche encore |
| `star`, `heart`, `hexagon`, `ring`, `arrow`, `cube` | aucun | `StarBorder` dans Flutter | faible |
| `Particles` | aucun élément ; des bibliothèques (particles.js) | Unity `ParticleSystem`, Godot `GPUParticles2D` | faible |
| `count` | — | Godot dit `amount` | aucun : HoloCode a déjà `Portals(count:)`, même sens |
| `life` | — | Godot dit `lifetime` | faible |
| `speed`, `gravity`, `colors` | — | les mêmes, dans les moteurs de jeu | faible ; `Zoom(speed:)` existe, même idée (une vitesse) |
| `seed` | — | — | aucun : déjà le mot de la graine d'un monde, même sens |
| `follow` | aucun mot du web | « Follow » d'une caméra (Cinemachine) | faible |
| `scroll`, `pointer` | l'évènement `scroll`, `scroll()` en CSS ; `PointerEvent`, `cursor: pointer` | `ScrollController`, `PointerEvent` | faible ; même sens |
| `Model` | aucun en HTML ; « model » dans three.js et glTF | — | faible ; à trancher à l'étape 11 du PLAN-3D |
| À éviter | `Part` est gardé pour la 3D (ADR-056) ; `depth` aussi (ADR-047) | — | — |

## Recommandation

Dans cet ordre :

1. **Corriger le pincement** (six lignes) et ajouter un essai qui pince la page dans Chrome sans fenêtre, pour qu'il ne casse plus en silence. Avant toute mesure sur le téléphone de Yocthan. (Ajouté à la relecture : la classe `diagonale` de `page.html:75`, cassée par le même commit, se corrige en même temps.)
2. **Corriger la lecture des lettres**, sans mot nouveau : le moteur écrit le texte entier pour le lecteur d'écran et cache les boîtes de lettres.
3. **Les formes** : manque 2, option A (`Loop(form:)`), avec quelques formes nommées de plus (manque 1, option B). Tout en CSS, mesuré possible, sans moteur dans le navigateur.
4. **Le défilement, puis le pointeur** : option A, un seul mot, `follow:`.
5. **Les particules** : option A, avec une graine. Mesurer sur un téléphone avant de fixer le plafond (`count`).
6. **Les chemins libres et le trajet** : manque 1, options A et T1, avec une grammaire de chemin bornée. Revoir avant cela la règle « une forme nommée est un bouton », qui gênerait T2.
7. **Les objets pleins** : suivre le PLAN-3D (ADR-049). Pas de faux cube en CSS.

## Exemple d'auteur

```holo
Page(
  title: "Galerie",
  lang: "fr",
  children: [
    // Existe. Correction proposée dans le moteur : le lecteur d'écran lit
    // « L'atelier des mondes », et non plus une lettre après l'autre.
    H1("L'atelier des mondes", enter: Enter(y: 40px, opacity: 0, letters: 0.04s, ease: spring)),

    // Écriture proposée : une poussière d'or qui éclate une fois, tirée de la graine 7.
    // Chaque grain part du centre, à une vitesse de 30 % à 100 % de speed (en px par seconde),
    // tombe de gravity (en px par seconde, chaque seconde) et s'efface en life.
    Particles(height: 240px, count: 120, colors: ["#E9B44C", "#FFE9A8"],
      speed: 300px, gravity: 200px, life: 2s, seed: 7),

    // Écriture proposée : form: dans Loop, et la forme star. Le rond devient une étoile, et revient.
    Shape(form: circle, color: "#E9B44C", size: 80px,
      loop: Loop(form: star, rotate: 72deg, for: 1.2s)),

    // Écriture proposée : path:, un chemin libre (ici, un cœur).
    Shape(path: "M 50 90 C 20 65 0 45 0 25 C 0 5 25 -5 50 20 C 75 -5 100 5 100 25 C 100 45 80 65 50 90 Z",
      color: "#F472B6", size: 120px),

    // Écriture proposée : along:. Une lune fait le tour du disque pâle (Stack existe).
    Stack(children: [
      Shape(form: circle, color: "#FFFFFF22", size: 200px),
      Shape(form: circle, color: "#4ECDC4", size: 16px,
        loop: Loop(along: circle, for: 4s, back: false, ease: linear)),
    ]),

    // Écriture proposée : follow: pointer. L'image penche vers le pointeur, ou vers le doigt posé.
    Image(source: "lever.svg", alt: "Un soleil jaune se lève au-dessus d'un fleuve bleu",
      loop: Loop(x: 12px, y: 8px, flip: 8deg, tilt: 6deg, follow: pointer)),

    // Écriture proposée : follow: scroll. Le texte arrive au rythme du défilement.
    P("Le texte arrive au rythme du défilement.",
      enter: Enter(y: 80px, opacity: 0, follow: scroll)),
  ],
)
```

Vérifié avec `holo check -` : sans les lignes proposées, le fichier est accepté ; chaque ligne proposée est refusée avec un message clair (« « Loop » n'a pas de paramètre « form » », « bloc inconnu « Particles » »…). Ces mots n'existent donc pas aujourd'hui.

Le même en HTML, CSS et JavaScript. Il fait la même chose : le titre entier pour le lecteur d'écran, les mêmes polygones de 32 sommets, la même graine, les mêmes vitesses et la même gravité, le même trajet, le pointeur suivi seulement pendant le toucher au doigt, tout arrêté pour qui demande moins de mouvement. Deux écarts *(relevés à la relecture)* : la même graine ne donne pas les mêmes grains, car le jumeau tire ses nombres avec un générateur congruentiel, et le moteur avec la finale de SplitMix64 (`moteur/src/seed.rs:5-12`, employée par `random` à `moteur/src/state.rs:1531`), que la proposition veut reprendre (« la même suite de hasard que `random` ») ; un jumeau exact devrait recopier ce calcul, sur 64 bits (`BigInt` en JavaScript). Et le jumeau fait deux choix que l'écriture HoloCode ne dit pas : le défilement avance en ligne droite (`linear`) et l'entrée finit à 40 % de la traversée (`animation-range`) ; aujourd'hui, une entrée prend la courbe `out` par défaut (`movement.rs:146`).

```html
<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Galerie</title>
<style>
  :root { --spring: linear(0,.009,.035 2.1%,.141,.281 6.7%,.723 12.9%,.938 16.7%,1.017,1.077,1.121,1.149 24.3%,1.159,1.163,1.161,1.154 29.9%,1.129 32.8%,1.051 39.6%,1.017 43.1%,.991,.977 51%,.974 53.8%,.975 57.1%,.997 69.8%,1.003 76.9%,1.004 83.8%,1); }
  main { max-width: 640px; margin: 0 auto; }
  img { max-width: 100%; height: auto; }
  .lecteur { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  .mot { display: inline-block; white-space: nowrap; }
  .lettre { display: inline-block; animation: titre .8s var(--spring) calc(var(--i) * .04s) both; }
  @keyframes titre { from { opacity: 0; translate: 0 40px; } }
  #poussiere { display: block; width: 100%; height: 240px; }
  .forme { width: 80px; height: 80px; background: #E9B44C; animation: etoile 1.2s cubic-bezier(.65,0,.35,1) infinite alternate both; }
  .coeur { display: block; width: 120px; height: 120px; }
  .pile { display: inline-grid; } .pile > * { grid-area: 1 / 1; }
  .disque { width: 200px; height: 200px; border-radius: 50%; background: #FFFFFF22; }
  .lune { width: 16px; height: 16px; border-radius: 50%; background: #4ECDC4; offset-rotate: 0deg;
    offset-path: path("M 100 0 A 100 100 0 1 1 100 200 A 100 100 0 1 1 100 0");
    animation: tour 4s linear infinite both; }
  @keyframes tour { from { offset-distance: 0%; } to { offset-distance: 100%; } }
  .penche { display: block; transition: translate .2s, transform .2s;
    translate: calc(var(--px, 0) * 12px) calc(var(--py, 0) * 8px);
    transform: perspective(800px) rotateX(calc(var(--py, 0) * 6deg)) rotateY(calc(var(--px, 0) * 8deg)); }
  .defile { animation: arrive linear both; animation-timeline: view(); animation-range: entry 0% cover 40%; }
  @keyframes arrive { from { opacity: 0; translate: 0 80px; } }
  @media (prefers-reduced-motion: reduce) {
    .lettre, .forme, .lune, .defile { animation: none !important; }
    .penche { translate: none; transform: none; transition: none; }
    #poussiere { display: none; }
  }
</style>
</head>
<body>
<main>
  <h1 id="titre">L'atelier des mondes</h1>
  <canvas id="poussiere" aria-hidden="true"></canvas>
  <div class="forme" aria-hidden="true"></div>
  <svg class="coeur" viewBox="0 0 100 100" aria-hidden="true">
    <path d="M 50 90 C 20 65 0 45 0 25 C 0 5 25 -5 50 20 C 75 -5 100 5 100 25 C 100 45 80 65 50 90 Z" fill="#F472B6"/>
  </svg>
  <div class="pile" aria-hidden="true"><div class="disque"></div><div class="lune"></div></div>
  <img class="penche" src="lever.svg" alt="Un soleil jaune se lève au-dessus d'un fleuve bleu">
  <p class="defile">Le texte arrive au rythme du défilement.</p>
</main>
<script>
  const calme = matchMedia("(prefers-reduced-motion: reduce)").matches;
  // 1. Le titre, lettre par lettre ; le lecteur d'écran lit le titre entier.
  const titre = document.getElementById("titre");
  const texte = titre.textContent;
  let rang = 0;
  titre.innerHTML = `<span class="lecteur">${texte}</span><span aria-hidden="true">` + texte.split(" ")
    .map((mot) => `<span class="mot">${[...mot].map((l) => `<span class="lettre" style="--i:${rang++}">${l}</span>`).join("")}</span>`)
    .join(" ") + "</span>";
  // 2. Le rond qui devient une étoile : deux polygones de 32 sommets.
  const anneau = (point) => Array.from({ length: 32 }, (_, k) => point(k))
    .map(([x, y]) => `${(50 + 50 * x).toFixed(2)}% ${(50 + 50 * y).toFixed(2)}%`).join(", ");
  const rond = anneau((k) => { const a = (k / 32) * 2 * Math.PI - Math.PI / 2; return [Math.cos(a), Math.sin(a)]; });
  const pointes = Array.from({ length: 10 }, (_, k) => { const a = (k / 10) * 2 * Math.PI - Math.PI / 2, r = k % 2 ? 0.45 : 1; return [r * Math.cos(a), r * Math.sin(a)]; });
  const etoile = anneau((k) => { const t = (k / 32) * 10, n = Math.floor(t), f = t - n, a = pointes[n], b = pointes[(n + 1) % 10];
    return [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]; });
  document.head.insertAdjacentHTML("beforeend", `<style>.forme{clip-path:polygon(${rond})}@keyframes etoile{to{clip-path:polygon(${etoile});rotate:72deg}}</style>`);
  // 3. La poussière : 120 grains tirés de la graine 7 ; leur place vient du temps écoulé (une formule), pas d'un calcul pas à pas.
  if (!calme) {
    const toile = document.getElementById("poussiere"), ctx = toile.getContext("2d");
    const ratio = Math.min(devicePixelRatio || 1, 2), L = toile.clientWidth, H = 240;
    toile.width = L * ratio; toile.height = H * ratio; ctx.scale(ratio, ratio);
    let graine = 7;
    const hasard = () => (graine = (graine * 1103515245 + 12345) % 2147483648) / 2147483648;
    const grains = Array.from({ length: 120 }, (_, k) => { const a = hasard() * 2 * Math.PI, v = 300 * (0.3 + 0.7 * hasard());
      return { vx: Math.cos(a) * v, vy: Math.sin(a) * v, couleur: ["#E9B44C", "#FFE9A8"][k % 2] }; });
    let debut = null;
    const image = (t) => {
      debut ??= t;
      const s = (t - debut) / 1000;
      ctx.clearRect(0, 0, L, H);
      if (s > 2) return;
      ctx.globalAlpha = 1 - s / 2;
      for (const g of grains) { ctx.fillStyle = g.couleur; ctx.fillRect(L / 2 + g.vx * s, H / 2 + g.vy * s + 100 * s * s, 3, 3); }
      requestAnimationFrame(image);
    };
    requestAnimationFrame(image);
  }
  // 4. L'image penche vers le pointeur ; au doigt, seulement pendant qu'il touche l'écran.
  const penche = document.querySelector(".penche");
  if (!calme) {
    addEventListener("pointermove", (e) => {
      if (e.pointerType === "touch" && e.buttons === 0) return;
      penche.style.setProperty("--px", ((e.clientX / innerWidth) * 2 - 1).toFixed(3));
      penche.style.setProperty("--py", ((e.clientY / innerHeight) * 2 - 1).toFixed(3));
    });
    const repos = () => { penche.style.removeProperty("--px"); penche.style.removeProperty("--py"); };
    document.documentElement.addEventListener("pointerleave", repos);
    addEventListener("pointerup", (e) => { if (e.pointerType === "touch") repos(); });
  }
</script>
</body>
</html>
```

Ce jumeau a été ouvert dans Chrome sans fenêtre (PC, taille de téléphone) : aucune erreur ; le titre est lu « L'atelier des mondes » ; 5 614 pixels de poussière dessinés à 0,5 s ; la forme est un polygone de 32 sommets ; la lune avance ; le texte suit `view()` ; le pointeur change la pose. Avec « moins de mouvement » : 0 animation en cours, la poussière cachée, l'image au repos, le texte visible. (Commande et sortie dans `duel-motion.md`, partie « Mesures ».)

L'écart, à fonctions égales : 32 lignes non vides de HoloCode (dont 10 de commentaires) contre 100 lignes de HTML, CSS et JavaScript. Surtout, le jumeau demande de savoir écrire un canevas, des polygones calculés, `offset-path`, `animation-timeline`, des évènements de pointeur et la règle du mouvement réduit. Mais l'écriture HoloCode ci-dessus **n'existe pas encore** : ce n'est pas une comparaison de ce que HoloCode sait faire aujourd'hui.

## Par couche

- **Langage** : `form:` dans `Enter` et `Loop` ; des formes nommées de plus ; `Shape(path:)` ; `Loop(along:)` ; `follow: scroll | pointer` ; le bloc `Particles` ; plus tard `Model` (PLAN-3D, étape 11).
- **Moteur (Rust)** : `movement.rs` écrit les formes en polygones de 32 sommets, le trajet (`offset-path`), le défilement (`animation-timeline`) ; `flat.rs` écrit un chemin en SVG dans la page, après avoir vérifié sa grammaire (seulement les commandes `M L H V C S Q T A Z` et des nombres, un nombre de commandes borné) ; la correction des lettres (le texte entier pour le lecteur, les boîtes cachées) ; un bloc `Particles` devient un `<canvas aria-hidden="true">` avec ses réglages en attributs `data-*`. Utile pour tout le reste : écrire une seule `@keyframes` par sorte de mouvement, et les valeurs en variables CSS (voir `duel-motion.md`).
- **Enveloppe navigateur** : `page.html`, corriger le pincement ; quelques lignes pour `follow: pointer` (des variables CSS, sans le moteur) ; un petit script de particules, chargé seulement si la page en a ; quand `animation-timeline` manque, le bloc reste visible.
- **Services serveur** : rien pour les effets. Pour la 3D, les fichiers `.holo3d` préparés (PLAN-3D, étapes 4 et 5).

## Dépendances

- Le pincement : rien. Il doit passer avant les mesures sur téléphone (point 7 de la liste de Yocthan).
- Les lettres : rien.
- Changer de forme : réécrire les quatre formes en polygones de 32 sommets (`flat.rs:64-70`) ; vérifier que les leçons 30 et 32 à 34 ne changent pas à l'œil.
- `along:` dépend de `Shape(path:)` pour les chemins libres ; `along: circle` n'en dépend pas.
- `follow: pointer` : la page légère (ADR-033) doit le faire sans le moteur.
- Les particules : une décision sur le nom et les réglages ; la même suite de hasard que `random`, pour qu'une graine donne partout le même éclat ; une mesure sur téléphone.
- Les objets pleins : le feu vert de Yocthan sur le PLAN-3D (ADR-049).

## Coût

- **Pincement** : six lignes et un essai ; une demi-séance (estimation).
- **Lettres** : environ 60 octets de plus par texte coupé (estimation) ; une demi-séance (estimation).
- **Changer de forme** : 444 octets par polygone de 32 sommets (mesuré), donc environ 900 octets par boucle de forme (estimation) ; une séance avec la leçon (estimation).
- **Défilement** : environ 100 octets de CSS par bloc (estimation) ; une séance (estimation).
- **Pointeur** : environ 1 Ko de JavaScript dans `page.html` (estimation) ; une séance (estimation).
- **Particules** : un script de 3 à 5 Ko compressés, chargé seulement s'il sert (estimation) ; deux à trois séances, mesures comprises (estimation). À comparer : 200 grains écrits avec `Repeat` font 6 779 octets compressés de HTML (mesuré), sans physique.
- **Chemins libres et trajet** : vérification des chemins, 5 à 10 Ko de WebAssembly en plus (estimation) ; deux séances (estimation).
- **Objets pleins** : environ 15 séances (estimation du PLAN-3D).

## Accessibilité, déterminisme, budgets

- **Accessibilité.** Tout ce qui est décoratif est caché au lecteur d'écran (`aria-hidden`) : particules, formes sans nom, trajets. Un texte qui bouge reste un texte entier pour le lecteur (la correction des lettres). Qui demande moins de mouvement voit tout immobile : particules absentes, rien ne suit le pointeur ni le défilement, et le bloc qui « entre au défilement » est à sa place dès le départ. Pas d'éclat qui clignote plus de trois fois par seconde (règle 2.3.1 des WCAG) : le moteur peut le refuser.
- **Déterminisme.** Une graine donne le même éclat partout. La place d'un grain se calcule d'après le temps écoulé, par une formule : au même instant, la même image, quelle que soit la cadence de l'écran. Nuance *(relecture)* : la graine garantit les places des grains, pas les pixels. Le lissage du canevas peut varier d'un navigateur à l'autre, et `Math.cos` n'est pas garanti identique partout : non vérifié. Pour des places identiques partout, le tirage doit se faire en nombres entiers, comme `seed.rs` (« tout est en arithmétique entière », `seed.rs:2-3`). Le pointeur et le défilement changent seulement l'apparence ; ils ne touchent jamais une valeur de la page (`State`) : l'arbitre ne les voit pas.
- **Budgets.** Un plafond de grains (par exemple 500, à fixer après mesure sur téléphone) ; un canevas à la densité 2 au plus, comme `showreel-max.html` ; un chemin borné en commandes ; rien qui fasse venir le dessin (633 554 octets mesurés) pour un effet d'une page à plat. Sur téléphone, cadence et batterie : non mesurées ici (pas d'appareil).

## Recette qui peut échouer

| Essai | Ce qui doit se passer | Ce qui fait échouer |
|---|---|---|
| R9-1 Pincer | dans Chrome sans fenêtre, en taille de téléphone : un pincement à deux doigts sur la page légère ne lève aucune erreur, fait venir le moteur, et grossit la page | une erreur, ou rien. **Échoue aujourd'hui** (mesuré : TypeError à `page.html:219` et à `page-engine.js:879`) |
| R9-2 Lettres | `H1("CLAUDE", enter: Enter(opacity: 0, letters: 0.07s))` : le nom lu dans l'arbre d'accessibilité est « CLAUDE » ; « Motion designer » garde son espace (sans `opacity: 0` ou une autre pose, `holo check` refuse : « « Enter » dit ce qui bouge ») | « C L A U D E ». **Échoue aujourd'hui** (mesuré ; remesuré à la relecture : « C L A U D E », et « Motion designer » lu en lettres, sans espace) |
| R9-3 Changer de forme | `Loop(form: star)` sur un rond : à mi-chemin, `clip-path` est un polygone différent des deux bouts ; au départ, le rond ressemble aux leçons d'avant | un saut d'un coup ; ou la leçon 30 qui change à l'œil |
| R9-4 Moins de mouvement | avec la préférence : 0 animation en cours, aucun canevas qui dessine, le bloc au défilement visible, l'image au repos | une seule chose qui bouge encore |
| R9-5 Chemin refusé | `Shape(path: "M0 0 L10 10 <script>")`, un chemin vide, ou plus de 500 commandes : refusés par `holo check`, avec la ligne | accepté |
| R9-6 Sans prise en charge | un navigateur sans `animation-timeline` (à simuler, ou Firefox) : le texte est visible et à sa place | un texte resté invisible |
| R9-7 Graine | deux ouvertures de la même page, à la même seconde d'une horloge figée : les deux images du canevas ont la même empreinte | deux images différentes |
| R9-8 Cadence (téléphone) | 300 grains : au moins 55 images par seconde sur le Flip 3 et sur un Android modeste | moins de 55 : abaisser le plafond |
| R9-9 Poids | une page à plat avec `Particles` : ni `/pkg/` ni le moteur léger téléchargés pour l'effet seul | le dessin ou le moteur demandé |

## Objection

La meilleure raison de ne pas le faire : c'est la piste la moins utile pour un site. Codex le dit lui-même : « Ajouter des animations seules ne suffira pas. » Chaque effet est un mot de plus, et le langage peut s'alourdir d'effets que peu d'auteurs emploieront. Les démonstrations les plus ambitieuses du web viennent du code libre (un canevas et du JavaScript), que HoloCode refuse (ADR-015).

Autre façon de faire : au lieu de dix mots d'effets, un seul mécanisme, des **modules d'effet enfermés** (comme ADR-045) qui dessinent sur un canevas qu'on leur prête, sans rien voir d'autre de la page. Plus puissant, mais plus long, et la sécurité serait à prouver ; un module n'échange aujourd'hui qu'un nombre contre un nombre.

## Expérience requise

- Après la correction : Yocthan pince la page sur son Flip, page légère et page avec le moteur.
- Un prototype de particules sur canevas, à 120, 300 et 520 grains, sur le Flip 3 et un Android modeste : cadence (au moins 55 images par seconde), batterie sur 5 minutes. Le comparer aux 200 grains écrits avec `Repeat`. Cela décide le plafond, et entre les options A et B. Pas d'appareil ici.
- TalkBack sur un titre coupé en lettres, avant et après la correction.
- Le défilement lié et le trajet sur Safari d'un iPhone, si l'on en trouve un : non vérifiés ici.
- Le trajet quand le visiteur grossit son texte à 200 % : `path()` suit-il ? Sinon, essayer `shape()`.
- Yocthan juge la beauté : la forme qui change, l'image qui penche, la poussière.

## Mises à jour de documents à prévoir

À lister, pas à faire maintenant :

- `docs/01-holocode/GUIDE.md` : § 6 decies (les nouveaux mots, et la phrase des limites, ligne 1513) ; § 6 nonies (`Shape` : les formes, `path:`) ; aide-mémoire (`Shape`, `Enter`, `Loop`, `Particles` ; il y manque déjà `inView`, ligne 1680).
- Leçons, à partir de 82 : changer de forme ; un chemin libre ; suivre un trajet ; entrer au défilement ; suivre le pointeur ; des particules. Une notion par leçon, et l'index `exemples/lecons/README.md`.
- `docs/01-holocode/NOMS.md` : chaque mot nouveau face au web ; corriger la ligne 210.
- `docs/01-holocode/COMPARAISON-WEB.md` : le CSS (`transform`, `animation`, `clip-path`), le JavaScript (« Approche, position du défilement », ligne 152) et `canvas`.
- `docs/01-holocode/TABLEAU-WEB.md` : lignes 503-504, 596-600, 609-612, 661-662 ; puis republier le tableau en ligne.
- `docs/02-gouvernance/adr/ADR-034-mouvement.md` : les conséquences des lignes 62 et 68-69 ; une fiche nouvelle pour les mots choisis ; ADR-032 si `Shape` change.
- `exemples/motion/README.md` : lignes 17, 26 et 30-32 ; le duel à rejouer (voir `duel-motion.md`).
- Le journal : le pincement cassé par la traduction (ADR-060), et la lecture des lettres ; l'état dans `AGENTS.md`.

Relu le 2026-10-07 : 15 corrections.
