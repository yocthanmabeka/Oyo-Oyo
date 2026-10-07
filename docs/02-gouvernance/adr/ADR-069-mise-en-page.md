# ADR-069 — La mise en page : téléphone, ordinateur, la place, ce qui dépasse, les proportions, le curseur, le texte justifié, décrocher

- Statut : PROPOSITION (construit et essayé ; attend la validation de Yocthan)
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : l'ordre de Yocthan du 2026-10-07, lot 4 (« téléphone, ordinateur, adaptation à la place disponible, débordement, proportions, curseur et texte justifié ») ; sa décision du même jour, sur son téléphone : décrocher la page « comme pour tourner » ; sa règle du même jour : « si la fonction existe sur téléphone, elle doit strictement aussi exister sur ordinateur et vice-versa » ; `ADR-017` (la disposition vient des blocs), `ADR-021` (`Zoom`), `ADR-041` (`phone:`), `ADR-061` (les touches).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Un style savait dire « sur un téléphone » (`phone:`), pas « sur un ordinateur », ni « dans une petite case ». La page restait large de 640px, même sur un grand écran.
- Sur un téléphone, un mot trop long faisait déborder la page.
- Rien ne coupait un texte trop long, ne bornait une hauteur, ne gardait des proportions ni ne changeait le curseur. Pas de texte justifié.
- Quand on zoomait, le moteur prenait le zoom de toute page, même d'un site ordinaire : celui du navigateur était coupé.
- Sur un téléphone, une page faite pour le clavier (`On(Key.left, …)`) ne se jouait pas : le clavier de l'écran n'apparaît que dans un champ où l'on écrit.

## Décision

1. **Téléphone, ordinateur** : dans un style, `phone: { … }` (un écran plus étroit que la page, 640px) et, nouveau, `computer: { … }` (un écran de 1024px ou plus).
   - `Page { max-width: 960px; }`, ou la même chose dans `computer:`, élargit la colonne de la page (640px sans rien écrire).
   - `display: none` cache un bloc, dans `phone:`, `computer:` et `narrow:` seulement.
2. **La place** : `narrow: { … }` vaut quand la case de `Grid` où se trouve le bloc fait moins de 320px de large (20rem : le seuil grandit avec le texte du visiteur), quel que soit l'écran. La page mesure elle-même les cases ; le style vaut pour la case et pour ce qu'elle contient.
3. **Ce qui dépasse** :
   - sans rien écrire, un mot trop long passe à la ligne ;
   - `line-clamp: 3` arrête le texte après 3 lignes (de 1 à 20), avec « … » ; un lecteur d'écran lit tout ;
   - `min-width`, `min-height`, `max-height`, en px ou en %, écrits en `rem` comme les autres tailles ;
   - `overflow`, `overflow-x`, `overflow-y` : `visible`, `hidden`, `auto`, `scroll` ;
   - `white-space` : `normal`, `nowrap`, `pre-line`, `pre-wrap`.
4. **Les proportions** :
   - `aspect-ratio: 16/9`, `4 / 3` ou `1` (la largeur sur la hauteur, des nombres positifs) ;
   - `object-fit` : `cover`, `contain`, `fill`, `none`, `scale-down`. Sans rien écrire, une image est `cover` ;
   - `object-position` : `center`, `top`, `bottom`, `left`, `right`, ou un coin (`top left`…).
5. **Le curseur** : `cursor:` prend 22 formes (`auto`, `default`, `pointer`, `text`, `move`, `grab`, `grabbing`, `not-allowed`, `help`, `wait`, `progress`, `crosshair`, `zoom-in`, `zoom-out`, `none`, `copy`, `alias`, `no-drop`, `cell`, `context-menu`, `vertical-text`, `all-scroll`), ou une image rangée à côté, `url("viseur.svg")` (.png, .svg, .cur). Le moteur ajoute la forme de secours `auto`. Au doigt, il n'y a pas de curseur : ce n'est qu'une indication.
6. **Le texte justifié** : `text-align: justify`. Le moteur ajoute `hyphens: auto` : les mots se coupent en fin de ligne, dans la langue de la page.
7. **Décrocher la page** (décision de Yocthan, « comme pour tourner ») :
   - sans rien écrire, quand on zoome (pincer, Ctrl + molette), le navigateur grossit la page sur place, comme n'importe quel site : elle reste accrochée ;
   - `zoom: Zoom(detach: true)` ajoute au menu ☰ le bouton « Décrocher » : la page se détache de l'écran comme une feuille, et le zoom l'approche. « Accrocher » la remet à sa place, à sa taille. Un lecteur d'écran annonce l'un et l'autre ;
   - avec des points (`points:`, `pixels:`), `Zoom(shrink: true)` ou `Zoom(active: false)`, le moteur garde le zoom, comme avant ;
   - on grossit au doigt autant qu'on veut, mais la page ne se réduit pas sous sa taille normale (`minimum-scale=1`). Sans cela, quand le moteur grossit une page décrochée, Chrome sur téléphone élargit sa zone d'affichage, et le menu ☰ (avec « Accrocher ») sort de l'écran. Trouvé par l'essai dans Chrome.

   Vision de Yocthan, gardée pour la 3D : décrocher, c'est passer du web au métavers ; accrocher, revenir ; derrière la page, une pile de feuilles (zéro ou plus).
8. **Les touches à l'écran** (règle de Yocthan : ce qui existe sur l'ordinateur existe sur le téléphone, et l'inverse) : sur un appareil tactile sans souris, une page qui écoute des touches (`On(Key.p, …)`, `Key.left`, `Key.enter`…) les montre en bas de l'écran.
   - Le doigt fait ce que fait le clavier ; gardée enfoncée, une touche se répète.
   - Un lecteur d'écran lit « Touche flèche gauche ».
   - Les touches ne cachent jamais le bas de la page. Rien à écrire.
9. **En chemin**, vu par Yocthan sur son téléphone :
   - une fenêtre `Dialog` fermée restait posée, invisible, sur le bas de la page, et prenait le doigt à la place des liens (leçon 63) : corrigé à part, PR 160 ;
   - depuis le Big Bang, rien ne menait aux leçons. La page des mondes a maintenant, en haut à gauche, « Les leçons » et « La pile », à toucher au doigt ; et « Leçon 27 → » quand le monde est la leçon 26.

## Ce qui est refusé, et pourquoi

Chaque refus dit pourquoi, ou donne le bon mot.

- `text-overflow` : seul, il ne fait rien. Le message dit d'écrire `line-clamp: 1`.
- `display` hors de `phone:`, `computer:` et `narrow:`, ou autre chose que `none` : la disposition vient des blocs (`ADR-017`).
- Les 14 formes d'étirement du curseur (`ew-resize`, `col-resize`…) : rien à étirer sans disposition à la main. Sept autres formes du CSS ne sont pas offertes non plus : `copy`, `alias`, `no-drop`, `cell`, `context-menu`, `vertical-text`, `all-scroll`.
- Une image de curseur qui n'est ni .png, ni .svg, ni .cur.
- `aspect-ratio: 16:9` : le message donne `16/9`.
- `white-space: pre` : il ne passe jamais à la ligne, et le texte sort de l'écran d'un téléphone. Le message donne `pre-wrap`, qui garde les espaces et les retours à la ligne, et passe à la ligne quand il le faut.
- `Zoom(detach: true)` avec des points ou `shrink: true` (la page s'y décroche déjà), ou avec `active: false` (on ne zoome pas).
- Un état inconnu, comme `wide:` : le message donne la liste, `computer` compris.

## Les défauts du CSS évités

- **`@container`** : il faut déclarer un conteneur à la main, et une case ne peut pas se mesurer elle-même. Ici, rien à déclarer.
- **Les « … »** demandent quatre réglages ensemble, et `text-overflow` seul ne fait rien. Ici, un seul.
- **Un mot trop long** fait déborder la page sur un téléphone. Ici, il passe à la ligne d'office.
- **Une image** à qui l'on donne une largeur et une hauteur se déforme. Ici, jamais, à moins d'écrire `object-fit: fill`.
- **Un curseur dessiné** sans forme de secours est ignoré en entier. Ici, le moteur l'ajoute.
- **Un texte justifié** ouvre de grands trous entre les mots sur un écran étroit. Ici, les mots se coupent.
- Et du web : un site qui veut son propre zoom coupe souvent celui du navigateur ; une page faite pour le clavier ne se joue pas au doigt.

## Dettes

- `narrow:` ne mesure que les cases de `Grid`, pas celles de `Row` ni de `Column`. Sans JavaScript, il ne vaut pas : c'est la page qui mesure.
- `computer:` n'a qu'un seuil (1024px).
- Les touches à l'écran sont une rangée, pas une croix dessinée, et l'auteur ne peut pas les cacher.
- La pile de feuilles derrière une page décrochée attend la 3D.
- La taille d'une image de curseur n'est pas vérifiée (le navigateur la limite).
- Une zone qui défile (`overflow: auto`) ne reçoit pas le clavier dans tous les navigateurs : Chrome le fait seul depuis 2024, pas Safari.
- À trancher avec Yocthan : `display: none` dans `phone:` ou `computer:` peut cacher un bouton sur un seul appareil, ce que sa règle de parité interdirait. Pour une phrase, seule l'allure change.

## Critères de validation

- Tests du moteur :
  - `the_layout_settings_of_lot_4` : chaque réglage accepté, sept refus ;
  - `the_layout_of_lot_4_reaches_the_browser` : ce que reçoit le navigateur (la largeur de la page, `computer`, `narrow`, `hyphens`, les quatre réglages de `line-clamp`, la forme de secours du curseur, le zoom laissé au navigateur) ;
  - `a_page_can_be_detached_only_where_it_means_something` : `detach`, et quatre refus ;
  - `the_compared_shop_uses_the_whole_vocabulary` : chaque réglage apparaît dans une leçon.
- Dans Chrome, sur un ordinateur de 1280px et un téléphone de 400px (leçons 89 à 93) : la page à 960px sur l'ordinateur ; ce qui se cache selon l'écran ; les trois cases étroites marquées ; rien ne déborde ; trois lignes puis « … » ; la boîte qui défile ; un carré, l'image coupée ou entière ; le curseur dessiné et sa forme de secours ; le texte justifié, les mots coupés.
- Dans Chrome, le zoom et les touches :
  - une page ordinaire (leçon 1) : le navigateur la grossit, et le moteur n'est même pas demandé ; avec des points (leçon 9), le zoom est au moteur ;
  - leçon 94 : accrochée, « Décrocher » offert, la page approchée au pincement, puis accrochée à sa taille ;
  - leçon 77 au doigt : cinq touches, P, 5 et Entrée donnent 16, Échap remet à 0 ; à la souris, aucune touche à l'écran.
- Dans Chrome, à la taille d'un téléphone : depuis le Big Bang, « Les leçons » se touche et mène à la leçon 1 ; la leçon 26 mène à la 27.
- Reste : voir le lot au doigt, sur le téléphone de Yocthan (leçons 7, 26, 35, 77, 81, 89 à 94).
