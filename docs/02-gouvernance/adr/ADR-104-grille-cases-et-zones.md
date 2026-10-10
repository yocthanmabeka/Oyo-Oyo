# ADR-104 — Une grille qui place ses cases : `columnSpan:`, `rowSpan:`, `Grid(areas:)` et `area:`

- Statut : ACCEPTÉ (fait et validé : Yocthan, 2026-10-09, « tu le valides déjà, tu le fais déjà » ; essayé et validé par Yocthan le 2026-10-10 : « je viens de faire tous les essais et je valide »)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #233 (« Dernière dette du web : une grille : une case sur plusieurs colonnes, des zones nommées ») ; le grand tableau du web, où `grid` était « en partie » (« Pas de zones nommées ni de case sur deux colonnes », `docs/01-holocode/TABLEAU-WEB.md`) ; `ADR-024` (la disposition), `ADR-069` (la mise en page, `narrow:`) ; la piste 4 de l'exploration du web complet (`proposals/Claude/exploration-web-complet-2026-10/piste-04.md`), qui écartait `span: 2` « tant qu'on ne sait pas éviter » le débordement sur un téléphone.
- Validation : Yocthan, le 2026-10-09 : « tu le valides déjà, tu le fais déjà ».
- Projets affectés : HoloCode, HoloEngine

## Contexte

- `Grid(columns: 3)` range des cases de même taille, et en perd sur un écran étroit (`ADR-024`). Il manquait deux choses courantes sur le web :
  - **une case plus grande que les autres** : le tableau du mois sur deux colonnes et deux lignes, une photo en long ;
  - **une page dessinée par zones** : un bandeau en haut, un menu à gauche, le texte à droite, un pied.
- En CSS, cela s'écrit `grid-column: span 2`, `grid-template-areas` et `grid-area`. Trois défauts viennent avec :
  - **le débordement.** Une case sur deux colonnes, dans une grille qui n'en a plus qu'une sur un téléphone, fait ajouter une colonne au navigateur. Vérifié dans Chrome, sur un téléphone plié de 280px : la vraie colonne rétrécit à 120px, la case déborde dans une colonne ajoutée, et les autres cases ne prennent plus que la moitié de la largeur. C'est la raison pour laquelle la piste 4 écartait `span: 2`.
  - **l'ordre.** Les zones placent un bloc ailleurs que là où il est écrit : l'œil voit le menu d'abord, la touche Tab et le lecteur d'écran le trouvent après le texte (WCAG 1.3.2, « Ordre séquentiel logique », et 2.4.3, « Parcours du focus »). Le CSS en est à inventer `reading-flow` pour le réparer.
  - **le silence.** Une faute dans un nom de zone, une zone qui n'est pas un rectangle, des lignes de longueurs différentes : le navigateur ignore la règle, sans rien dire, et la page se range n'importe comment.
- Sur un téléphone, il faut en CSS récrire les zones dans une `@media` : la page n'est lisible que si l'auteur y a pensé.

## Décision

1. **`columnSpan: 2`** sur un bloc rangé dans `Grid` : sa case prend deux colonnes, de 2 jusqu'au nombre de colonnes de la grille (`columns:`, 2 sans rien écrire). **`rowSpan: 2`** : deux lignes, de 2 à 12.
2. **Rien ne déborde.** Quand la grille n'a plus assez de colonnes pour la case, celle-ci prend toute la ligne, et une seule ligne. Une case qui prend seulement plusieurs lignes les garde toujours : elle ne peut pas déborder.
3. **`Grid(areas: ["top top top", "menu main main", "foot foot foot"])`** dessine la grille avec des mots : une ligne de texte par rangée, les noms des zones séparés par des espaces, autant de cases à chaque ligne (de 1 à 12, et de 1 à 12 lignes). Une zone prend les cases qui portent son nom, et forme un rectangle. Un point (`.`) laisse une case vide. Les colonnes ont la même largeur : un menu plus étroit s'écrit moins de fois (`menu main main`).
4. **`area: menu`** : chaque bloc d'une grille à zones dit sa zone ; chaque zone reçoit un bloc, un seul (pour en ranger plusieurs, un `Column`). Un nom de zone s'écrit comme une valeur (`ADR-037`) : `menu`, `sideMenu`.
5. **Les blocs s'écrivent dans l'ordre des zones**, de gauche à droite puis de haut en bas (dans une page en arabe ou en hébreu, de droite à gauche : la grille suit le sens de la langue). L'œil, le clavier et le lecteur d'écran suivent ainsi le même chemin. Un bloc hors de son tour est refusé, avec l'ordre attendu.
6. **Sur un téléphone, les zones passent l'une sous l'autre, dans cet ordre.** Elles restent côte à côte tant que la grille a au moins 480px de large (30rem) et que chaque zone y garde au moins 120px (7,5rem, la largeur la plus petite d'une colonne de `Grid`). C'est la largeur de la grille qui compte, pas celle de l'écran : une grille à zones posée dans une case étroite s'empile aussi.
7. **Une case est mesurée par sa grille.** Le moteur pose sur la grille `container-type: inline-size`, et, pour chaque largeur employée dans le fichier, une règle `@container (min-width: …)` qui donne aux cases leur place. Tout est du CSS fabriqué par le moteur : la page marche sans JavaScript (avec `holo serve` aussi), et les seuils sont en `rem`, qui suivent la taille du texte choisie par le visiteur.
8. **Un `If` peut remplir une zone** ; quand il est faux, sa case ne laisse pas de trou. Les champs (`Input`, `Checkbox`, `Choice`, `Slider`), un chronomètre, et tous les blocs qui se voient prennent ces réglages ; un bloc qui bouge (`enter:`) aussi : la case l'enveloppe.
9. La place se dit dans les blocs, jamais dans un style (`ADR-017`) : `grid-column`, `grid-row`, `grid-area` et `grid-template-areas` sont refusés dans un style, avec le bon mot.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Où dire la place d'une case | dans un style, comme en CSS (`.featured { grid-column: span 2; }`) ; **sur le bloc**, comme `grow:` dans un `Row` | sur le bloc : un style ne dit que l'apparence (`ADR-017`), et on lit la forme de la page dans son plan |
| Le mot d'une case sur plusieurs colonnes | `span: 2`, celui du CSS ; `columns: 2`, le mot de `Grid` ; `wide: 2` et `tall: 2`, des mots nouveaux ; **`columnSpan: 2` et `rowSpan: 2`** | `span` seul ne dit pas si ce sont des colonnes ou des lignes, et évoque la balise `<span>` du HTML. `columns: 2` serait le plus court, mais une `Grid` rangée dans une autre a déjà son `columns:` : le même mot dirait deux choses sur le même bloc. `wide` et `tall` ne disent rien à un programmeur. `colspan` et `rowspan` sont les mots des tableaux de HTML (`colSpan`, `rowSpan` en JavaScript et en React), `columnSpan` et `rowSpan` ceux d'Android, de XAML et de Qt : un mot connu garde son sens (`ADR-016`), écrit en entier comme `columns:`, à la manière de Flutter (`ADR-037`) |
| Les zones | les lignes numérotées du CSS (`grid-column: 1 / 3`) ; des listes de noms (`[[top, top], [menu, main]]`) ; **un dessin en texte**, une ligne par rangée (`"top top"`) | on voit la page dans le dessin, comme dans `grid-template-areas` ; les numéros de lignes ne se lisent pas ; les listes de listes sont plus longues et ne se lisent pas mieux |
| Les mots des zones | `template:`, `layout:` (déjà pris par `Portals`), `map:`, `zones:` ; **`areas:` et `area:`** | les mots du CSS (`grid-template-areas`, `grid-area`), avec le même sens |
| L'ordre de lecture | libre, comme en CSS ; le moteur remet la page dans l'ordre de l'œil ; **l'ordre des blocs imposé : celui des zones** | libre, c'est le défaut ; un moteur qui déplace les blocs changerait en silence ce que l'auteur a écrit ; imposé, l'auteur voit l'ordre dans son fichier, et le téléphone empile les zones dans ce même ordre |
| Le téléphone | des zones à récrire pour lui, comme une `@media` ; **le moteur empile les zones, dans l'ordre** | des zones différentes sur le téléphone pourraient changer l'ordre ; empilées dans l'ordre de lecture, elles restent lisibles sans rien écrire |
| Quand empiler | selon l'écran (`phone:`, 640px) ; **selon la largeur de la grille** (au moins 480px, et 120px par zone) | une grille posée dans une case étroite d'un ordinateur doit s'empiler aussi ; 480px sépare un téléphone tenu droit (de 320 à 430px de large) d'un téléphone couché, d'une tablette et d'un ordinateur ; 120px par zone est la règle des colonnes de `Grid` |
| Une case trop large | refusée (la piste 4) ; la grille garde ses colonnes et défile de côté ; **la case prend toute la ligne** | refuser priverait l'ordinateur de la grande case ; défiler de côté est le défaut qu'on évite partout (WCAG 1.4.10) ; toute la ligne garde la page lisible |
| Combler les trous | `grid-auto-flow: dense`, qui remplit les trous avec les cases suivantes ; **les cases dans l'ordre, au risque d'un trou** | `dense` change l'ordre à l'écran sans changer l'ordre de lecture : le même défaut que les zones libres |
| Mesurer la grille | en JavaScript, comme `narrow:` ; **en CSS, `container-type` et `@container`** | sans JavaScript, la page serait fausse ; les requêtes de conteneur marchent dans tous les navigateurs visés, et le plateau (`Board`) s'en sert déjà |

## Ce qui est refusé, et pourquoi

- `columnSpan:` plus grand que les colonnes de la grille, ou `1` (une case prend déjà une colonne) ; `rowSpan:` au-delà de 12.
- `columnSpan:`, `rowSpan:` ou `area:` sur un bloc qui n'est pas rangé dans une `Grid` : le message dit de le ranger dans `Grid(children: [ … ])`.
- `columns:` et `areas:` ensemble : avec des zones, les colonnes sont celles du dessin.
- Dans le dessin : des lignes qui n'ont pas le même nombre de cases ; une zone qui n'est pas un rectangle ; une ligne faite seulement de cases vides ; une seule zone (pour un seul bloc, pas besoin de zones) ; un nom avec une majuscule au début, un accent, `_` ou `-` (le message donne le bon mot, `topBar`) ; plus de 12 lignes ou de 12 colonnes.
- Dans une grille à zones : un bloc sans `area:`, ou une phrase seule ; une zone qui n'existe pas (le message donne les zones) ; deux blocs dans la même zone ; une zone sans bloc ; un bloc hors de l'ordre des zones ; `columnSpan:` ou `rowSpan:` (la zone dit déjà la place) ; `area: "menu"` entre guillemets.
- Une fenêtre (`Dialog`), qui s'ouvre par-dessus la page, ou un son sans lecteur : ils ne prennent pas de place, leur case serait vide.
- Dans un style : `grid-column`, `grid-row`, `grid-area`, `grid-template-areas` (la place vient des blocs).

## Les défauts du web évités

- **Le débordement d'une case trop large**, qui ajoute une colonne et fait glisser la page de côté sur un téléphone : ici, la case prend toute la ligne.
- **L'ordre de l'œil qui n'est pas celui du clavier** : ici, les blocs s'écrivent dans l'ordre des zones, et `dense` n'existe pas.
- **Une règle de zones ignorée en silence** (une faute, un dessin qui n'est pas un rectangle, des lignes inégales) : ici, refusée avec sa raison.
- **Des zones à récrire pour le téléphone** : ici, elles s'empilent seules, dans l'ordre.
- **Un écart qui déborde** : douze colonnes séparées de 64px font plus de 700px, plus qu'un téléphone. Ici, l'écart entre les colonnes d'une grille à zones rétrécit quand la grille est étroite (au plus sa largeur divisée par le nombre de colonnes).
- **Une `@container` à déclarer à la main**, avec `container-type` sur le bon parent : ici, rien à écrire.

## Dettes

- Une case d'une liste qui change (`Repeat(over:)`) ne prend pas plusieurs colonnes, et un composant posé dans une grille ne prend pas ces réglages : on le range dans un `Column(columnSpan: 2, children: [ … ])`.
- Des colonnes de largeurs choisies (`1fr 2fr`) : on répète le nom d'une zone pour la rendre plus large.
- Le seuil de 480px est le même pour toutes les grilles à zones : l'auteur ne le choisit pas.
- Une grille à zones placée dans une rangée (`Row`) prend toute la largeur de la rangée : pour se mesurer, elle ne peut pas prendre la largeur de son contenu.

## Critères de validation

- Tests du moteur (`moteur/src/grid.rs`) :
  - `the_areas_are_read_in_reading_order_with_their_rectangles` : le dessin lu, les zones dans l'ordre de lecture, avec leurs lignes et leurs colonnes ;
  - `the_thresholds_keep_every_column_and_every_area_wide_enough` : les seuils des cases et des zones ;
  - `a_cell_takes_several_columns_and_rows` : le HTML et le CSS d'une case sur deux colonnes et deux lignes, d'une case haute, d'une case qui bouge ; une grille qui ne place rien ne change pas ;
  - `named_areas_follow_the_reading_order` : les zones et leurs places, un `If` dans une zone, les champs dans une grille ;
  - `a_grid_refuses_what_would_overflow_or_mislead` : vingt-sept refus, chacun avec sa raison.
- Dans Chrome : « une grille : une case sur deux colonnes et deux lignes, des zones dans l'ordre de lecture ; rien ne déborde sur un téléphone (leçon 127) ». Sur un ordinateur (1280px), la grande case prend deux colonnes et deux lignes, le menu est à gauche du texte, sur une colonne de trois ; sur un téléphone (360px) et un téléphone plié (280px, le Galaxy Z Fold fermé), rien ne déborde, la grande case prend la ligne, les zones s'empilent dans l'ordre. Il rate quand la case trop large ne prend plus la ligne (à 280px, la grille ajoute une colonne et les cases rétrécissent à 120px) et quand les zones ne s'empilent plus sur un téléphone.
- Leçon `127-une-grille-et-ses-zones.holo`.
