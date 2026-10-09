# HoloCode, et HTML, CSS, JavaScript : le grand tableau

- Relevé de Claude, tenu à jour à chaque changement du langage (dernier : 2026-10-07). La même chose, à filtrer, sur la page en ligne tenue à jour pour Yocthan.
- D’abord **tous les mots de HoloCode** (378 mots : 344 décidés, 34 à l’essai), puis **chaque élément de HTML, CSS et JavaScript** (130) et ce que HoloCode en a.
- **Existe ?** : le jugement de Claude, élément par élément (oui, en partie, non) ; ce n’est pas une mesure. Le tableau ne donne pas de pourcentage : aucune méthode reproductible ne mesure la part d’un élément du web qu’on obtient en HoloCode (consigne de Yocthan du 2026-10-07). Les comptes se refont en comptant les lignes.
- Les refus sont expliqués dans [`proposals/Claude/pourquoi-ces-refus-2026-10/`](../../proposals/Claude/pourquoi-ces-refus-2026-10/README.md).

## Résumé

| | Mesure | Détail |
|---|---|---|
| **HoloCode** | 378 mots | 344 décidés, 34 à l’essai |
| HTML | 62 éléments | 52 oui, 4 en partie, 3 non, 3 refusés |
| CSS | 34 éléments | 28 oui, 4 en partie, 0 non, 2 refusés |
| JavaScript | 34 éléments | 16 oui, 10 en partie, 6 non, 1 refusés, 1 sans objet |
| HTML, CSS, JS ensemble | 130 éléments | 96 oui, 18 en partie, 9 non, 6 refusés, 1 sans objet |

# Partie 1 — Les mots de HoloCode

## Blocs : la page

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Page` | La page entière, le bloc racine d'un fichier | `html, head, body, main` | Décidé (ADR-009) |
| `World` | Le monde qui est dans un point | — | Décidé (ADR-009) |
| `Component` | Un composant : écrit une fois, posé comme un bloc, avec ses paramètres | `template, composants React ou Vue` | Décidé (ADR-050, ADR-056) |
| `Use` | Pose un morceau sans paramètres, Use(Menu) | `slot, include` | Décidé (ADR-029) |

## Blocs : les repères

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Header` | L'en-tête de la page | `header` | Décidé (ADR-036) |
| `Nav` | Un menu de navigation | `nav` | Décidé (ADR-036) |
| `Main` | Le contenu principal | `main` | Décidé (ADR-036) |
| `Footer` | Le pied de page | `footer` | Décidé (ADR-036) |
| `Aside` | Un encadré à part, annoncé comme un contenu complémentaire | `aside` | Décidé (ADR-073) |

## Blocs : le texte

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Text` | Du texte sans rôle : une étiquette, une ligne d'état | `span` | Décidé (ADR-020) |
| `P` | Un paragraphe (une phrase seule en est un aussi) | `p` | Décidé (ADR-020) |
| `H1, H2, H3` | Les titres ; le numéro dit la place dans le plan | `h1, h2, h3` | Décidé (ADR-020) |
| `H4, H5, H6` | Les titres plus profonds, pour les longs documents | `h4, h5, h6` | Décidé (ADR-036) |
| `A` | Un lien vers une autre page | `a href` | Décidé (ADR-022) |
| `Hr` | Un trait de séparation | `hr` | Décidé (ADR-025) |
| `Quote` | Une citation, et son auteur | `blockquote` | Décidé (ADR-025) |
| `Code` | Du code montré tel quel | `pre, code` | Décidé (ADR-025) |
| `List` | Une liste, à puces ou numérotée | `ul, ol, li` | Décidé (ADR-009) |
| `Table` | Un tableau de données | `table, caption, thead, tbody, tr, th, td` | Décidé (ADR-038) |

## Blocs : les médias

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Image` | Une image | `img` | Décidé (ADR-009) |
| `Sound` | Un son qu'une règle fait entendre | `audio` | Décidé (ADR-031) |
| `Shape` | Une forme : rond, carré, triangle, losange | `div + CSS, svg` | Décidé (ADR-032) |
| `Drawing` | Un dessin vectoriel, fait de formes, nommé pour le lecteur d'écran | `svg role=img aria-label` | Décidé (ADR-086) |
| `Rect, Circle, Line, Path` | Les formes d'un dessin : rectangle, rond, trait, tracé | `rect, circle, line, path` | Décidé (ADR-086) |
| `Chart` | Un graphique d'après une liste : barres, courbe ou parts ; un tableau caché pour le lecteur d'écran | `svg + table` (une bibliothèque de graphiques, en JavaScript) | Décidé (ADR-087) |
| `Video` | Une vidéo, avec ses boutons, jamais lancée seule | `video controls` | Décidé (ADR-038) |

## Blocs : la disposition

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Row` | Côte à côte | `display: flex` | Décidé (ADR-024) |
| `Column` | L'un sous l'autre | `flex-direction: column` | Décidé (ADR-024) |
| `Grid` | Une grille, qui perd des colonnes sur un téléphone | `display: grid` | Décidé (ADR-024) |
| `Stack` | Poser un bloc sur un autre (une pastille sur une image) | `position: absolute, z-index` | Décidé (ADR-036) |
| `Board` | Un plateau : on y place les blocs par x et y | `position + JavaScript` | Décidé (ADR-026) |

## Blocs : agir

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Button` | Un bouton | `button` | Décidé (ADR-009) |
| `Input` | Un champ où l'on écrit un nombre ou un texte | `input` | Décidé (ADR-027) |
| `Checkbox` | Une case à cocher | `input type=checkbox` | Décidé (ADR-027) |
| `Choice` | Un choix parmi des options : boutons ronds, ou liste avec menu: true | `input radio, select, option` | Décidé (ADR-038) |

## Blocs : les mondes

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Point` | Un point lumineux qui contient un monde | — | Décidé (ADR-009) |

## Blocs : la vue

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Zoom` | Les garde-fous du zoom | `meta viewport` | Décidé (ADR-021) |
| `Points` | Les pixels qui deviennent des points au zoom | — | Décidé (ADR-021) |
| `Relief` | La page qui tourne, avec du relief | `transform 3D` | Décidé (ADR-021) |
| `Portals` | Le carrefour : des portails vers les mondes voisins | — | Décidé (ADR-021) |

## Blocs : les valeurs

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `State` | Les valeurs de la page : State(cart: 0) | `variables JavaScript` | Décidé (ADR-023) |
| `Prices` | Les prix ; le moteur calcule {count} et {total} | — | Décidé (ADR-023) |
| `Data` | Des valeurs lues sur le serveur | `fetch` | Décidé (ADR-030) |
| `If` | Montrer, ou faire valoir des règles, selon une valeur | `if` | Décidé (ADR-025) |

## Blocs : les règles

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `On` | Réagir à un geste : On(Add.tap, effect: …) | `addEventListener` | Décidé (ADR-015) |
| `Every` | Faire quelque chose à un rythme | `setInterval` | Décidé (ADR-026) |
| `When` | Guetter une valeur ou une rencontre | `un observateur écrit à la main` | Décidé (ADR-028) |

## Blocs : le mouvement

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Enter` | Faire entrer un bloc : d'où il part | `@keyframes + animation` | Décidé (ADR-034) |
| `Loop` | Faire vivre un bloc sans fin | `animation infinite alternate` | Décidé (ADR-034) |
| `Scenes, Scene` | Des scènes qui s'enchaînent | `délais calculés à la main` | Décidé (ADR-034) |

## Paramètres : partout

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `name` | Le nom d'un bloc, pour le toucher dans une règle | `id` | Décidé (ADR-009) |
| `children` | Ce qu'un bloc contient | `les éléments enfants` | Décidé (ADR-009) |

## Paramètres : la page

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `title` | Le titre de la page | `title` | Décidé (ADR-009) |
| `rules` | Les règles | `le code JavaScript` | Décidé (ADR-015) |
| `state, prices, keep` | Les valeurs, leurs prix, ce qu'on garde d'une visite à l'autre | `variables, localStorage` | Décidé (ADR-023, ADR-027) |
| `data` | Les données du serveur | `fetch` | Décidé (ADR-030) |
| `zoom, points, relief, portals` | Comment la page se regarde | — | Décidé (ADR-021) |
| `pixels` | Des sites plantés dans des pixels | — | Décidé (ADR-021) |
| `lang, description, image` | La langue, la description pour Google, l'image de partage | `html lang, meta description, og:image` | Décidé (ADR-038) |

## Paramètres : le texte

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `text` | Le texte d'un bouton | `le texte de button` | Décidé (ADR-009) |
| `to` | L'adresse d'un lien | `href` | Décidé (ADR-022) |
| `newTab, download (dans A)` | Un nouvel onglet, annoncé, qui ne peut pas toucher à la page ; un fichier rangé à côté qui se télécharge | `target=_blank rel=noopener, download` | Décidé (ADR-073) |
| `by` | L'auteur d'une citation | `cite` | Décidé (ADR-025) |
| `ordered` | Une liste numérotée | `ol` | Décidé (ADR-009) |
| `caption, head, rows` | La légende, les titres de colonnes et les lignes d'un tableau | `caption, thead, tbody` | Décidé (ADR-038) |

## Paramètres : les médias

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `source, alt, weight` | Le fichier, son texte de remplacement, son poids | `src, alt` | Décidé (ADR-009, ADR-025) |
| `form, color, size` | La forme, la couleur et la taille d'une Shape | `CSS` | Décidé (ADR-032) |

## Paramètres : la disposition

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `gap, align, columns` | L'espace, l'alignement, le nombre de colonnes | `gap, align-items, grid-template-columns` | Décidé (ADR-024) |
| `height` | La hauteur d'un plateau ou de scènes | `height` | Décidé (ADR-026) |
| `x, y, drag` | La place sur un plateau, et le glissement | `left, top, draggable` | Décidé (ADR-026, ADR-028) |
| `align (dans Stack)` | La place d'un bloc posé sur un autre | `top, right, bottom, left` | Décidé (ADR-036) |
| `grow` | Prendre la place qui reste dans un Row ou une Column | `flex-grow` | Décidé (ADR-052) |

## Paramètres : agir

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `value, label, max` | La valeur liée, l'étiquette (obligatoire), le maximum | `value, label, max` | Décidé (ADR-027) |
| `lines` | Un texte long, sur plusieurs lignes | `textarea` | Décidé (ADR-038) |
| `options, menu` | Les options d'un choix ; la liste déroulante | `option, select` | Décidé (ADR-038) |

## Paramètres : les mondes

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `seed, brightness, fragments` | La graine du monde, sa lumière, son morcellement | — | Décidé (ADR-009) |
| `color, palette, budget` | La couleur du point, celle de ses enfants, le poids permis | — | Décidé (ADR-009) |
| `inside, above` | Le monde intérieur ; où se plante un point | — | Décidé (ADR-009, ADR-021) |

## Paramètres : les règles

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `effect` | Ce qu'une règle demande | `le corps d'une fonction` | Décidé (ADR-015) |
| `is, not, over, under` | Les comparaisons : égal, différent, plus grand, plus petit | `===, !==, >, <` | Décidé (ADR-025) |
| `is: "…", not: "…" (un texte)` | Comparer un texte à un texte, à la lettre près, dans If et When ; ou à une autre valeur de texte | `=== et !== sur des textes` | À l’essai (ADR-063) |
| `meets, within` | Une rencontre entre deux blocs, au contact ou à un écart | `un calcul de collision` | Décidé (ADR-028) |
| `from, every` | Le fichier de données et son rythme | `fetch + setInterval` | Décidé (ADR-030) |

## Paramètres : le mouvement

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `enter, loop` | Le mouvement d'un bloc | `animation` | Décidé (ADR-034) |
| `at, for, ease` | Quand il part, combien de temps, quel caractère | `animation-delay, -duration, -timing-function` | Décidé (ADR-034) |
| `letters, each, back, repeat` | Lettre à lettre, enfant après enfant, sans retour, sans fin | `du JavaScript` | Décidé (ADR-034) |
| `opacity, x, y, scale, rotate, flip, tilt, blur, hue, round` | Ce qui bouge | `opacity, transform, filter, border-radius` | Décidé (ADR-034) |
| `inView` | Une entrée qui attend que le bloc arrive à l'écran | `IntersectionObserver` | Décidé (ADR-061) |

## Paramètres : la vue

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `active, max, shrink, levels, speed` | Les réglages de Zoom | — | Décidé (ADR-021) |
| `detach` | Offre au visiteur « Décrocher » : la page se détache de l'écran et le zoom l'approche ; sinon, elle reste à sa place | — | À l’essai (ADR-069) |
| `after, size, fragment, divisions, levels, density` | Les réglages de Points | — | Décidé (ADR-021, ADR-047) |
| `height, tilt` | Les réglages de Relief | — | Décidé (ADR-021) |
| `layout, count, size, brightness, duration` | Les réglages de Portals | — | Décidé (ADR-021) |

## Mots-valeurs

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `circle, square, triangle, diamond` | Les formes d'une Shape | `border-radius, clip-path` | Décidé (ADR-032) |
| `start, center, end, between` | Les alignements de Row et Column | `flex-start, center, flex-end, space-between` | Décidé (ADR-024) |
| `topLeft, top, topRight, left, center, right, bottomLeft, bottom, bottomRight` | Les places dans un Stack | `top, right, bottom, left` | Décidé (ADR-036, ADR-037) |
| `linear, smooth, out, in, back, spring, bounce` | Le caractère d'un mouvement | `cubic-bezier(…), linear(…)` | Décidé (ADR-034) |
| `forever` | Des scènes qui recommencent | `animation-iteration-count: infinite` | Décidé (ADR-034) |
| `grid, row, column, diagonal` | La disposition du carrefour | — | Décidé (ADR-021) |
| `true, false` | Oui, non | `true, false` | Décidé (ADR-009) |

## Signaux et capacités

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `tap` | Un toucher, un clic | `click` | Décidé (ADR-015) |
| `Key.left, Key.right, Key.up, Key.down, Key.space` | Les touches du clavier | `keydown` | Décidé (ADR-028) |
| `Key.enter, Key.escape, Key.a … Key.z, Key.digit0 … Key.digit9` | Entrée, Échap, les lettres, les chiffres ; les touches à une lettre se coupent dans le menu ; jamais Tab | `KeyboardEvent.key, .code` | Décidé (ADR-061) |
| `enter, leave` | Entrer dans un point, en sortir | `un routeur` | Décidé (ADR-022) |
| `portals` | Ouvrir le carrefour | — | Décidé (ADR-021) |
| `play` | Faire entendre un son | `audio.play()` | Décidé (ADR-031) |
| `stop` | Arrêter un son et le remettre au début | `pause(), currentTime = 0` | Décidé (ADR-061) |

## Demandes

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `add, sub, set, random` | Ajouter, retirer, fixer, tirer au hasard (rejouable) | `+=, -=, =, Math.random` | Décidé (ADR-023, ADR-026) |

## Valeurs calculées

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `{count}, {total}` | Le nombre d'articles et le prix du panier | `un calcul écrit à la main` | Décidé (ADR-023) |

## Styles

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `P { }, .carte { }, P.carte(…)` | Viser un type de bloc, ou un nom, et le poser | `sélecteurs de balise et de classe` | Décidé (ADR-017) |
| `color, background` | Les couleurs | `color, background` | Décidé (ADR-017) |
| `font-size, font-weight, font-style, font-family, text-align` | Le texte (les tailles suivent le réglage du visiteur) | `les mêmes, en rem` | Décidé (ADR-017, ADR-036) |
| `border, border-radius` | La bordure, les coins arrondis | `les mêmes` | Décidé (ADR-017) |
| `padding, margin, width, height, max-width, min-width, min-height, max-height` | Les marges et les tailles, au plus et au moins | `les mêmes` | Décidé (ADR-017, ADR-069) |
| `opacity` | La transparence | `opacity` | Décidé (ADR-017) |
| `hover, focus, active` | Ce qui change au survol, au clavier, pendant l'appui | `:hover, :focus-visible, :active` | Décidé (ADR-036) |

## Unités

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `px, deg` | Pixels (le navigateur reçoit des rem : ils suivent le texte du visiteur), degrés | `px, deg` | Décidé (ADR-009, ADR-061) |
| `ms, s, min, h` | Les durées | `ms, s` | Décidé (ADR-009) |
| `mm, cm, m, km` | Les longueurs (pour les mondes) | `mm, cm` | Décidé (ADR-009) |
| `B, KB, MB, GB` | Les poids (B = octet) | — | Décidé (ADR-009, ADR-037) |

## Le fichier

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `import` | Importer un autre fichier .holo | `link, script src` | Décidé (ADR-029) |
| `//` | Un commentaire | `<!-- -->, /* */, //` | Décidé (ADR-009) |
| `**gras**, *italique*, `code`` | Le texte enrichi, en Markdown | `strong, em, code` | Décidé (ADR-019) |
| `""" … """` | Un texte qui garde ses retours à la ligne | `br` | Décidé (ADR-025) |

## Blocs : les règles

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `After` | Une seule fois, plus tard ; sous une condition, part quand elle devient vraie | `setTimeout` | Décidé (ADR-039) |

## Paramètres : les règles

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `else` | Ce qu'un If montre quand la condition est fausse | `else` | Décidé (ADR-039) |

## Signaux et capacités

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `hover, hoverEnd` | La souris, le clavier ou le doigt arrive sur un bloc, puis le quitte | `mouseenter, mouseleave, focus, blur` | Décidé (ADR-039) |

## Valeurs calculées

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `{year}, {month}, {day}, {weekday}, {hour}, {minute}, {second}` | L'heure de l'appareil du visiteur, tenue à jour ; chaque seconde pour une page qui montre la seconde | `new Date()` | Décidé (ADR-039, ADR-089) |
| `Stopwatch, start, stop, reset, stopped, {time:stopwatch}` | Un chronomètre au centième ; le moteur reçoit le temps final | `performance.now() + requestAnimationFrame` | Décidé (ADR-089) |

## Blocs : les valeurs

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Repeat` | Écrire un modèle une fois, le poser pour chaque élément | `for, map, template` | Décidé (ADR-040) |
| `Item` | Un élément d'une répétition, et ses champs | `un objet JavaScript` | Décidé (ADR-040) |

## Paramètres : les valeurs

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `items, key` | Les éléments d'une répétition ; la clé d'un élément | `un tableau, key` | Décidé (ADR-040) |

## Mots-valeurs

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `item` | L'élément en cours, dans le modèle : {item.title}, item.add(1) | `item => …` | Décidé (ADR-040) |
| `screen` | height: screen : tout l'écran, au moins | `100dvh` | Décidé (ADR-061) |

## Styles

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `line-height, letter-spacing, text-transform, text-decoration` | Le texte soigné | `les mêmes` | Décidé (ADR-041) |
| `box-shadow, text-shadow` | Les ombres | `les mêmes` | Décidé (ADR-041) |
| `rotate, scale, transition` | Une pose, et la durée du passage d'une allure à l'autre | `rotate, scale, transition` | Décidé (ADR-041) |
| `linear-gradient, radial-gradient, url(…)` | Un fond en dégradé ou en image | `background-image` | Décidé (ADR-041) |
| `--or (variables)` | Une couleur ou une taille nommée dans le style de la page | `--or, var(--or)` | Décidé (ADR-041) |
| `dark, phone` | Le thème sombre ; un écran de téléphone | `@media` | Décidé (ADR-041) |
| `print` | Ce qui change sur papier ; print: { display: none; } cache un bloc à l'impression | `@media print` | Décidé (ADR-073) |
| `computer, narrow` | Un écran d'ordinateur (1024px et plus) ; une case étroite (moins de 320px) de grille, ou une part étroite d'un `Row` ou d'un `Column`, quel que soit l'écran | `@media, @container` | À l’essai (ADR-069, ADR-090) |
| `aspect-ratio, object-fit, object-position` | Garder des proportions ; l'image remplit son cadre ou se voit en entier, jamais déformée | `les mêmes` | À l’essai (ADR-069) |
| `overflow, overflow-x, overflow-y, white-space, line-clamp` | Ce qui dépasse : couper, faire défiler, ne pas passer à la ligne, « … » après quelques lignes | `les mêmes, -webkit-line-clamp` | À l’essai (ADR-069) |
| `cursor, justify` | La forme du curseur, ou un curseur dessiné ; le texte justifié, les mots coupés | `cursor, text-align: justify, hyphens` | À l’essai (ADR-069) |

## Blocs : la page

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Font` | Une police rangée à côté, chargée par la page | `@font-face` | Décidé (ADR-041) |

## Paramètres : la page

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `fonts, family` | Les polices de la page ; le nom d'une police | `@font-face, font-family` | Décidé (ADR-041) |

## Blocs : agir

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Form` | Un formulaire qu'une règle envoie | `form, fetch POST` | Décidé (ADR-042) |
| `Slider` | Une glissière entre deux bornes | `input type=range` | Décidé (ADR-042) |
| `Progress` | Une barre de progression | `progress` | Décidé (ADR-042) |

## Blocs : ouvrir et fermer

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Details` | Un pli qui s'ouvre | `details, summary` | Décidé (ADR-042) |
| `Dialog` | Une fenêtre par-dessus la page | `dialog` | Décidé (ADR-042) |

## Paramètres : les médias

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `caption, phone` | La légende d'une image ; l'image pour un téléphone | `figcaption, picture` | Décidé (ADR-042) |
| `captions (dans Video)` | Des sous-titres WebVTT, montrés d'emblée, dans la langue de la page | `track kind=captions` | Décidé (ADR-073) |
| `volume, loop` | Le volume d'un son (de 0 à 1) ; un son qui recommence sans fin | `audio.volume, loop` | Décidé (ADR-061) |

## Paramètres : agir

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `type, min, summary, open` | date, time, color ; le minimum d'une glissière ; le résumé d'un pli, ouvert au départ | `type, min, summary, open` | Décidé (ADR-042) |
| `type: file, accept` | Choisir un fichier à envoyer par un formulaire : image ou pdf, avec une taille maximale | `input type=file, accept` | Décidé (ADR-059) |
| `required, type: email, min (longueur d'un texte)` | Un formulaire qui vérifie : obligatoire, e-mail, longueurs ; messages sous les champs ; vérifié aussi au serveur | `required, type=email, minlength` | À l’essai (ADR-068) |

## Paramètres : la page

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `icon` | L'image de l'onglet | `link rel=icon` | Décidé (ADR-042) |

## Signaux et capacités

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `send, sent, failed` | Envoyer un formulaire ; l'envoi est arrivé, ou non | `fetch, then, catch` | Décidé (ADR-042) |
| `open, close` | Ouvrir, fermer une fenêtre | `showModal, close` | Décidé (ADR-042) |

## Le fichier

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `~~barré~~, ==surligné==, ^exposant^, ~indice~` | Les petites marques du texte | `s, mark, sup, sub` | Décidé (ADR-042) |

## Demandes

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `mul, div` | Multiplier, diviser (en nombres entiers) | `*=, /=` | Décidé (ADR-043) |

## Le fichier

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `{n:00}, {n:number}, {n:cents}, {weekday:name}, {month:name}` | Écrire un nombre joliment, dans la langue de la page | `padStart, Intl` | Décidé (ADR-043) |

## Demandes

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `push, remove, clear` | Ajouter à une liste, retirer la ligne touchée, tout vider | `push, splice, length = 0` | Décidé (ADR-044) |
| `push(Item(…)), item.done.set(1)` | Ajouter un élément à champs ; changer un champ de la ligne touchée | `push, objet.champ = …` | Décidé (ADR-051, ADR-057) |

## Paramètres : les valeurs

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `over` | La liste qui change, montrée ligne par ligne : Repeat(over: tasks) | `map, innerHTML` | Décidé (ADR-044) |
| `onAdd:, emit:` | Brancher un signal d'un composant ; l'émettre dans sa règle | `addEventListener, dispatchEvent` | Décidé (ADR-056) |
| `Item (dans State), item.title` | Les éléments à champs d'une liste, et leurs champs dans une ligne | `objets JavaScript` | Décidé (ADR-051) |
| `computed, Filter(name:, from:, contains:, in:, sortBy:, limit:)` | Une liste calculée : chercher, trier, montrer plus (l'écriture A, choisie par Yocthan) | `filter, sort, slice` | Décidé (ADR-062) |
| `field, is, reverse (dans Filter)` | Garder une sorte d'éléments ; trier du plus grand au plus petit | `filter, reverse` | À l’essai (ADR-062) |
| `empty (dans Repeat)` | Ce qu'on écrit quand une liste est vide | `if (list.length === 0)` | Décidé (ADR-062) |
| `name (dans Data), Shop.done, Shop.failed, Shop.refresh` | Des données qui disent « arrivées » ou « échec », et se relisent ; 10 secondes au plus | `fetch, response.ok, AbortController` | À l’essai (ADR-064) |
| `key (dans Repeat(over:)), total (dans Filter)` | Une clé choisie pour chaque ligne, le clavier gardé ; le nombre trouvé avant de couper | `key de React, filtered.length` | À l’essai (ADR-065) |
| `12.50 (une valeur à virgule)` | Un nombre à virgule exact, ses chiffres fixés à la déclaration ; montré dans la langue de la page | `Number, toFixed, Intl.NumberFormat` | À l’essai (ADR-066) |
| `today, Days(name:, from:, to:), {d:date}, {d:weekday}, min (dans Input)` | Les dates : le jour même, compter les jours, les montrer dans la langue de la page, borner un champ | `new Date(), toLocaleDateString, min` | À l’essai (ADR-067) |

## Blocs : les règles

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Module` | Du code WebAssembly enfermé : un fil à part, une mémoire plafonnée, arrêté s'il dure trop ; il reçoit et rend des nombres, des textes et des listes, en JSON relu avec méfiance | `new Worker, WebAssembly` | Décidé (ADR-045, ADR-077) |

## Paramètres : la page

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `modules, input, output, time, memory` | Les modules de la page ; ce qu'un module reçoit, rend (un nom, ou une liste de noms), et ses limites | — | Décidé (ADR-045, ADR-077) |
| `components, params, emits` | Les composants de la page ; leurs paramètres (avec valeurs par défaut) ; les signaux qu'ils émettent | `props, emit` | Décidé (ADR-050, ADR-056) |
| `children (dans un composant)` | L'emplacement où va le contenu donné à l'appel | `slot, children` | Décidé (ADR-058) |

## Signaux et capacités

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `run, done, failed` | Lancer un module ; il a rendu son nombre ; il a été arrêté | `postMessage, terminate` | Décidé (ADR-045) |

## Le fichier

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `module "…"` | Annoncer un module en haut du fichier | `script src` | Décidé (ADR-045) |
| `{id} dans le nom du fichier` | Une adresse qui porte une valeur : profil/{id}.holo sert /profil/123 ; la page lit {id}, sans le changer | `[id] (Next.js), :id (Express)` | À l’essai (ADR-078) |

# Partie 2 — HoloCode face à HTML, CSS et JavaScript

## HTML — Structure de la page

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `Page(…)` | `html, head, body, main` | la page entière | Oui | Déjà là | Une page HoloCode est un seul bloc Page. |
| `Page(title:)` | `title` | le titre de l'onglet | Oui | Déjà là | Il lit les valeurs, dont celles d'une adresse (`{id}`), et suit leurs changements (ADR-090). |
| `toujours UTF-8` | `meta charset` | l'encodage | Oui | Déjà là | Rien à écrire. |
| `Zoom(active:, max:)` | `meta viewport` | le zoom sur téléphone | Oui | Déjà là | Fait autrement : le zoom est au cœur de HoloCode. |
| `Page(description:, image:)` | `meta description, image de partage` | ce que montrent Google et les réseaux | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-038). |
| `Page(icon:)` | `link rel=icon` | la petite image de l'onglet | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). |
| `Page(lang:)` | `lang` | la langue de la page | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-038). |
| `Header, Footer, Main` | `header, footer, main` | en-tête, pied, contenu principal | Oui | Déjà là | Ajoutés le 2026-10-06 (ADR-036). |
| `Nav` | `nav` | le menu de navigation | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-036). |
| `Aside(children:)` | `aside` | un encadré à part | Oui | Déjà là | Ajouté le 2026-10-07 (ADR-073). |
| `les titres H1 à H3` | `section, article` | des parties de page | Refusé exprès | Non | Jugement de Claude, sans décision : les titres donnent déjà le plan. Soumis à Codex et Gemini. |
| — | `div` | une boîte sans sens | Refusé exprès | Non | Refusé par Yocthan (ADR-009) : chaque bloc dit ce qu'il est. |
| `Text` | `span` | un bout de texte | Oui | Déjà là | — |

## HTML — Texte

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `H1, H2, H3` | `h1, h2, h3` | les titres | Oui | Déjà là | — |
| `H4, H5, H6` | `h4, h5, h6` | titres plus profonds | Oui | Déjà là | Ajoutés le 2026-10-06 (correction d'ADR-020). |
| `P, ou une phrase seule` | `p` | un paragraphe | Oui | Déjà là | — |
| `**gras**` | `strong, b` | le gras | Oui | Déjà là | — |
| `*italique*` | `em, i` | l'italique | Oui | Déjà là | — |
| `texte entre trois guillemets` | `br` | retour à la ligne | Oui | Déjà là | — |
| `Hr()` | `hr` | un trait de séparation | Oui | Déjà là | — |
| `Quote("…", by:)` | `blockquote, q, cite` | les citations | En partie | Déjà là | Pas de citation courte au milieu d'une phrase. |
| `Code("…"), `accents graves`` | `pre, code, kbd` | du code montré tel quel | Oui | Déjà là | — |
| `~~barré~~, ==surligné==` | `u, s, mark` | souligné, barré, surligné | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). Pas de souligné, exprès : il ressemble à un lien. |
| `m^2^, H~2~O ; petit par le style` | `small, sub, sup` | petit, indice, exposant | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). |
| — | `abbr, time, address` | sigle, date, adresse | Non | Plus tard | — |

## HTML — Listes et liens

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `List(children:)` | `ul, li` | liste à puces | Oui | Déjà là | — |
| `List(ordered: true)` | `ol` | liste numérotée | Oui | Déjà là | — |
| — | `dl, dt, dd` | liste de définitions | Non | Plus tard | Un glossaire, une fiche technique. |
| `A("…", to:)` | `a href` | un lien | Oui | Déjà là | — |
| `A(to: "#Horaires")` | `a vers un endroit de la page` | sauter plus bas dans la page | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). Un nom qui n'existe pas est refusé. |
| `A(newTab: true), A(download: true)` | `a target, download` | nouvel onglet, télécharger | Oui | Déjà là | Ajouté le 2026-10-07 (ADR-073) : noopener, et l'annonce au lecteur d'écran d'un nouvel onglet ; un téléchargement seulement pour un fichier rangé à côté. |

## HTML — Images et médias

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `Image(source:)` | `img` | une image | Oui | Déjà là | — |
| `Image(alt:), obligatoire` | `img alt` | le texte pour qui ne voit pas | Oui | Déjà là | Obligatoire depuis le 2026-10-06 ; alt: "" pour un décor (ADR-038). |
| `Image(phone: "petite.jpg")` | `picture, srcset` | une image plus légère sur téléphone | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). Un seul seuil, celui de la page. |
| `Image(caption:)` | `figure, figcaption` | une image et sa légende | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). |
| `Video(source:, label:, captions:)` | `video, track` | une vidéo, ses sous-titres | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-038) ; les sous-titres le 2026-10-07 (ADR-073). |
| `Sound(…) et .play ; Sound(label:) pour un lecteur` | `audio` | un son, un lecteur | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). |
| `Point, World, Shape` | `canvas, WebGL` | un dessin libre, de la 3D | En partie | Oui, utile | Des points et quatre formes ; pas de dessin libre ni de modèles 3D. |
| `Drawing(label:, width:, height:, children: [ Rect, Circle, Line, Path ])` | `svg dans la page` | un dessin vectoriel | Oui | Déjà là | Ajouté le 2026-10-07 (ADR-086) : en SVG, nommé pour le lecteur d'écran ; une mesure peut suivre un nombre de la page. |
| `Point(inside: "x.holo")` | `iframe, embed, object` | une page dans la page | En partie | Déjà là | On entre dans un autre fichier HoloCode ; pas dans un autre site. |

## HTML — Tableaux

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `Table(caption:, head:, rows:)` | `table, tr, td, th, caption` | un tableau de données | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-038). Pas de case sur deux colonnes. |

## HTML — Formulaires

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `Button(name:, text:)` | `button` | un bouton | Oui | Déjà là | — |
| `Form(name:), Contact.send, sent, failed` | `form (envoyer)` | envoyer des réponses à un serveur | Oui | Déjà là | Vers le serveur local (décision de Yocthan), un fichier compris (ADR-042, ADR-059). Avec holo serve, vérifié à nouveau par le moteur et rangé dans holo-data/site.sqlite, avec ou sans JavaScript, et ses copies (ADR-075, ADR-076). |
| `Input(value:, label:, max:)` | `input texte, nombre` | un champ | Oui | Déjà là | — |
| `Checkbox(value:, label:)` | `input checkbox` | une case à cocher | Oui | Déjà là | — |
| `Choice(value:, label:, options:)` | `input radio` | un choix parmi plusieurs | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-038). |
| `Slider(value:, label:, min:, max:)` | `input range` | un curseur à glisser | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). |
| `Input(type: date | time | color)` | `input date, heure, couleur` | choisir une date, une couleur | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). |
| `Input(type: file, accept:, max:) dans un Form` | `input email, mot de passe, fichier` | adresse, secret, envoi de fichier | En partie | Déjà là | Le fichier : ajouté le 2026-10-07 (ADR-059). Le courriel et le mot de passe viendront avec des comptes. |
| `label: (obligatoire)` | `label` | le nom d'un champ | Oui | Déjà là | Mieux que HTML : impossible de l'oublier. |
| `Input(…, lines: 5)` | `textarea` | un texte long | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-038). |
| `Choice(…, menu: true)` | `select, option` | une liste déroulante | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-038). |
| — | `fieldset, legend, datalist` | regrouper, suggérer | Non | Plus tard | — |
| `{valeur}, {total}` | `output` | afficher un résultat | Oui | Déjà là | — |
| `Progress(value:, max:, label:)` | `progress, meter` | une barre de progression | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). Pas de meter (zones bonne, moyenne, mauvaise). |

## HTML — Ouvrir et fermer

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `Details(summary:, children:)` | `details, summary` | un pli qui s'ouvre | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). |
| `Dialog(name:), open, close` | `dialog` | une fenêtre par-dessus | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). |

## HTML — Code

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `styles après la page` | `style` | les styles | Oui | Déjà là | — |
| `import "theme.holo"` | `link stylesheet` | un fichier de styles partagé | Oui | Déjà là | Un fichier de styles seuls est un thème ; la page garde les siens (ADR-052). |
| — | `script` | du code dans la page | Refusé exprès | Non | Refusé (ADR-015) : aucun code libre, pour la sécurité. |
| `Component(params:, emits:, children), Use ; Repeat` | `template, slot` | un morceau réutilisable | Oui | Déjà là | Des composants à paramètres, valeurs par défaut, signaux et emplacement (ADR-050, ADR-056, ADR-058). |
| `les repères, les étiquettes obligatoires ; la vue points lue au lecteur d'écran` | `attributs aria, tabindex` | l'accessibilité fine | Oui | Déjà là | En vue points, la page reste dessous, lisible, et « Vue points » est annoncé (ADR-061) ; le contraste trop faible est refusé, un audit vérifie chaque leçon (ADR-055). |

## CSS — Couleurs et texte

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `color, background` | `color, background-color` | couleur du texte et du fond | Oui | Déjà là | — |
| `font-family, font-size (en px)` | `font-family, font-size` | la police et sa taille | Oui | Déjà là | — |
| `font-weight, font-style` | `font-weight, font-style` | gras, italique | Oui | Déjà là | — |
| `text-align (left, center, right, justify)` | `text-align` | l'alignement | Oui | Déjà là | justify coupe aussi les mots en fin de ligne, dans la langue de la page : pas de trous entre les mots sur un téléphone (ADR-069). |
| `line-height (sans unité), letter-spacing` | `line-height, letter-spacing` | l'interligne, l'espacement | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-041). |
| `text-transform, text-decoration` | `text-transform, text-decoration` | majuscules, souligné | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-041). |
| `Page(fonts: [ Font(family:, source:) ])` | `@font-face` | charger sa propre police | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-041). Toujours font-display: swap ; une graisse par fichier. |
| `linear-gradient, radial-gradient, url("fond.jpg")` | `dégradés, image de fond` | fond en dégradé ou en image | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-041). L'image couvre toujours le bloc. |
| `box-shadow, text-shadow` | `box-shadow, text-shadow` | les ombres | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-041). |
| `Page { --or: … } puis color: --or` | `variables (--couleur)` | une couleur nommée, réutilisée | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-041). Une variable inconnue est refusée. |
| `dark: { … } dans un style` | `prefers-color-scheme` | le mode sombre | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-041). |

## CSS — Boîtes

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `margin, padding` | `margin, padding` | les marges | Oui | Déjà là | — |
| `width, height, min-width, max-width, min-height, max-height` | `width, height, min-width, max-width, min-height, max-height` | les tailles | Oui | Déjà là | min-width, min-height et max-height ajoutés le 2026-10-07 (ADR-069). |
| `border, border-radius` | `border, border-radius` | bordure et coins arrondis | Oui | Déjà là | — |
| `opacity` | `opacity` | la transparence | Oui | Déjà là | — |
| `overflow, overflow-x, overflow-y, white-space, line-clamp ; les mots trop longs passent à la ligne d'eux-mêmes` | `overflow, text-overflow, white-space, line-clamp` | ce qui dépasse | Oui | Déjà là | Ajouté le 2026-10-07 (ADR-069). « … » en un seul réglage (line-clamp) ; text-overflow refusé, il ne fait rien seul en CSS. |
| `cursor: help, grab… ou url("viseur.svg")` | `cursor` | la forme du curseur | Oui | Déjà là | Ajouté le 2026-10-07 (ADR-069). Le moteur ajoute la forme de secours, sans laquelle le navigateur ignore un curseur dessiné. |
| `px et % ; le navigateur reçoit des rem ; height: screen` | `unités %, rem, vw, clamp` | des tailles qui s'adaptent à l'écran | Oui | Déjà là | Le texte, les marges, les largeurs, les coins suivent le réglage du visiteur ; height: screen remplit l'écran sans le défaut de 100vh (ADR-061). |

## CSS — Disposition

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| — | `display, position, float, z-index` | placer à la main | Refusé exprès | Non | Refusé (ADR-017) : la disposition vient des blocs. |
| `Row, Column (gap, align, grow)` | `flexbox (en ligne, en colonne)` | côte à côte, l'un sous l'autre | Oui | Déjà là | grow: prend la place qui reste (ADR-052). |
| `Grid(columns:, gap:)` | `grid` | une grille | En partie | Déjà là | Pas de zones nommées ni de case sur deux colonnes. |
| `automatique, et phone:, computer:, narrow: { … } ; Page { max-width: }` | `@media, @container (s'adapter à l'écran, à la place)` | changer selon la taille | Oui | Déjà là | phone: (ADR-041) ; computer: (1024px et plus) et narrow: (une case de grille de moins de 320px, quel que soit l'écran) ajoutés le 2026-10-07 (ADR-069) ; narrow: aussi dans les parts d'un Row ou d'un Column (grow:, une largeur en %) le 2026-10-08 (ADR-090). Rien à déclarer : le moteur mesure les cases. |
| `aspect-ratio, object-fit, object-position` | `aspect-ratio, object-fit, object-position` | garder des proportions | Oui | Déjà là | Ajouté le 2026-10-07 (ADR-069). Une image n'est jamais déformée sans le demander (object-fit: cover au départ). |
| `Stack et align:` | `position: absolute (badge, pastille)` | poser un bloc sur un autre | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-036). Pas encore de bulle attachée à un bloc. |
| `P { }, .carte { }` | `sélecteurs par balise et par nom` | viser des blocs | Oui | Déjà là | — |
| — | `sélecteurs composés, cascade, !important` | viser finement, forcer | Refusé exprès | Non | Refusé (ADR-017) : source de conflits sans fin en CSS. |

## CSS — États et mouvement

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `hover:, focus:, active: dans un style` | `hover, focus, active` | l'apparence au survol, au clic | Oui | Déjà là | Ajoutés le 2026-10-06 (ADR-036). |
| `transition: 0.3s ; automatique au survol` | `transition` | passer en douceur d'un état à l'autre | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-041). |
| `Enter, Loop, Scenes` | `animation, @keyframes` | une animation | Oui | Déjà là | Dix propriétés et sept courbes ; pas d'étapes intermédiaires libres. |
| `rotate, scale ; x, y dans Enter et Loop` | `transform 2D` | déplacer, tourner, grandir | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-041). Pas de déplacement fixe : la place vient des blocs. |
| `Relief(tilt:), flip, tilt` | `transform 3D, perspective` | la profondeur | En partie | Déjà là | Fait autrement : la page entière tourne. |
| `blur, hue dans Enter et Loop` | `filter (flou, couleurs)` | flouter, teinter | En partie | Plus tard | — |
| `Shape(form:) : quatre formes` | `clip-path` | découper une forme | En partie | Plus tard | — |
| `automatique` | `prefers-reduced-motion` | moins de mouvement | Oui | Déjà là | Mieux que CSS : on n'a rien à écrire. |

## JavaScript — Gestes

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `On(Nom.tap, effect:)` | `clic` | réagir à un toucher | Oui | Déjà là | — |
| `On(Key.left…), Key.enter, Key.escape, Key.a … Key.z, Key.digit0 … Key.digit9` | `clavier` | réagir aux touches | Oui | Déjà là | Toutes les touches utiles ; jamais Tab ; les touches à une lettre se coupent dans le menu (ADR-061). Au doigt, les touches écoutées s'affichent à l'écran (ADR-069). |
| `On(Carte.hover), On(Carte.hoverEnd)` | `survol, approche (mouseenter)` | quand la souris passe dessus | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-039) : à la souris, au clavier et au doigt. L'approche d'un personnage, en profondeur, reste à faire. |
| `Enter(…, inView: true)` | `défilement (scroll)` | réagir quand on descend | En partie | Déjà là | Apparaître en arrivant à l'écran (ADR-061). Pas encore la position du défilement comme valeur. |
| `drag: true sur un plateau` | `glisser-déposer` | faire glisser | En partie | Déjà là | Sur un plateau seulement. |
| `le zoom du navigateur ; Zoom(detach: true) : « Décrocher » ; le zoom du moteur avec des points` | `pincer, zoomer` | le zoom à deux doigts | Oui | Déjà là | Depuis le 2026-10-07 (ADR-069, décision de Yocthan) : la page reste à sa place, comme tout site ; elle se décroche seulement si l'auteur l'offre et que le visiteur le demande. |

## JavaScript — Données et calcul

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `State(…)` | `variables` | garder une valeur | En partie | Déjà là | Nombres entiers, nombres à virgule exacts (ADR-066), textes, listes (à champs) ; pas encore de nombre négatif. |
| `add, sub, mul, div, set, random` | `calcul (+ − × ÷)` | calculer | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-043). En nombres entiers ; pas de pourcentage ni de racine. |
| `If(…, else: […]), When(…) ; un texte : If(size, is: "L")` | `if, else` | décider | Oui | Déjà là | Le « sinon » ajouté le 2026-10-06 (ADR-039) ; comparer des textes, If(size, is: "L") et When(answer, is: "Paris"), le 2026-10-07 (ADR-063, à valider). |
| `State(articles: [ Item(…) ]), push(Item(…)), item.done.set(1)` | `tableaux, objets` | des listes de valeurs | Oui | Déjà là | Des listes à champs, qui changent pendant la visite (ADR-044, ADR-051, ADR-057). |
| `Repeat(items:, children:, rules:) ; Filter(…) pour chercher, filtrer, trier` | `for, map` | répéter pour chaque élément | Oui | Déjà là | Déplié à la lecture (ADR-040) ; des listes calculées pour chercher, filtrer, trier et montrer plus (ADR-062). |
| `Module(…) : du code WebAssembly enfermé` | `fonctions` | du calcul réutilisable | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-045) : un nombre en entrée, un nombre en sortie, arrêté s'il dure trop. Le 2026-10-07 (ADR-077) : des nombres à virgule, des textes et des listes, en entrée et en sortie. |
| `{nom} dans un texte` | `texte (majuscules, longueur, découper)` | travailler un texte | En partie | Plus tard | — |
| `random, rejouable` | `Math.random` | le hasard | Oui | Déjà là | Mieux pour un jeu : la même partie se rejoue. |
| `{weekday:name} {day} {month:name} {year}, {hour} h {minute:00}` | `Date` | la date et l'heure du jour | En partie | Déjà là | ADR-039 et ADR-043. Pas encore de calcul sur les dates (un compte à rebours). |
| `{n:number}, {n:cents}, {minute:00}, {weekday:name}` | `Intl (formats)` | 1 234,50 €, dates en français | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-043). Langue de la page ; noms en français et en anglais. |

## JavaScript — Temps

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `Every(1s, effect:)` | `setInterval` | répéter toutes les N secondes | Oui | Déjà là | — |
| `After(3s, effect: …)` | `setTimeout` | une seule fois, plus tard | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-039) ; sous une condition, l'attente part quand elle devient vraie. |
| `le moteur` | `requestAnimationFrame` | dessiner image par image | Sans objet | Non | Le moteur s'en charge. |

## JavaScript — Réseau et mémoire

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `Data(from:, every:)` | `fetch (lire)` | lire des données d'un serveur | En partie | Déjà là | Des valeurs et des listes d'objets JSON (ADR-051) ; seulement le serveur d'où vient la page. |
| `Contact.send` | `fetch (envoyer)` | envoyer au serveur | Oui | Déjà là | Ajouté le 2026-10-06 (ADR-042). Les valeurs d'un formulaire, au serveur d'où vient la page. |
| — | `WebSocket` | parler en direct avec un serveur | Non | Plus tard | Pour le jeu à plusieurs, après votre validation locale. |
| `keep: [panier]` | `localStorage` | garder dans le navigateur | Oui | Déjà là | Rangé sous l'adresse : un modèle d'adresse garde des valeurs pour chacune (ADR-090). |
| — | `cookies, sessionStorage` | se souvenir le temps d'une visite | Non | Plus tard | — |
| — | `service worker (hors ligne)` | marcher sans réseau | Non | Plus tard | — |

## JavaScript — Page et navigation

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| — | `modifier la page (DOM)` | changer la page à la main | Refusé exprès | Non | Refusé (ADR-015) : c'est le moteur qui change la page. |
| `Point(inside:), enter, leave ; profil/{id}.holo` | `routeur, historique` | changer de page sans recharger | Oui | Déjà là | Chaque monde a son adresse ; « retour » marche. Une adresse qui porte une valeur : profil/{id}.holo sert /profil/123, fabriquée par le serveur (ADR-078). |
| `Page(address: [onglet, page])` | `history.pushState, URLSearchParams, popstate` | l'historique dans une page, une adresse qui se partage | Oui | Déjà là | Les valeurs nommées s'écrivent après le « ? » ; un toucher fait un pas que « Précédent » défait ; le serveur fabrique la page avec elles, même sans JavaScript (ADR-091). |
| `import "commun.holo"` | `import de modules` | découper son code | En partie | Déjà là | — |

## JavaScript — Médias et appareil

| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|
| `Sound(volume:, loop:), play, stop` | `Web Audio` | jouer, régler un son | En partie | Déjà là | Volume, boucle, arrêt (ADR-061). Pas de mélange ni d'effets. |
| `Shape`, `Drawing`, `Drawing(shapes:)` | `Canvas 2D` | dessiner librement | En partie | Plus tard | Le dessin déclaré existe (`Drawing`, ADR-086), et un module peut rendre une liste de formes vérifiées (ADR-088) ; le dessin trait par trait reste refusé ; cent formes par liste. |
| `le moteur dessine des points` | `WebGL, WebGPU` | la 3D | En partie | Oui, utile | Pas encore d'objets pleins : l'essai de la chaise. |
| — | `géolocalisation, caméra, vibration` | l'appareil du visiteur | Non | Plus tard | — |
| — | `presse-papiers, partage` | copier, partager | Non | Plus tard | — |
| — | `paiement, comptes` | payer, se connecter | Non | Plus tard | Bien plus tard, et jamais sans un serveur sûr. |
