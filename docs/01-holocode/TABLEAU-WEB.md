# HoloCode, et HTML, CSS, JavaScript : le grand tableau

- Relevé de Claude, tenu à jour à chaque changement du langage (dernier : 2026-10-06). La même chose, à filtrer, sur la page en ligne tenue à jour pour Yocthan.
- D’abord **tous les mots de HoloCode** (224 mots : 224 décidés, 0 à l’essai), puis **chaque élément de HTML, CSS et JavaScript** (130) et ce que HoloCode en a.
- **Couverture** : la part de ce que fait l’élément web qu’on obtient en HoloCode aujourd’hui. Estimation de Claude, non mesurée.
- Les refus sont expliqués dans [`proposals/Claude/pourquoi-ces-refus-2026-10/`](../../proposals/Claude/pourquoi-ces-refus-2026-10/README.md).

## Résumé

| | Mesure | Détail |
|---|---|---|
| **HoloCode** | 224 mots | 224 décidés, 0 à l’essai |
| HTML | 52 % de couverture | 25 oui, 11 en partie, 23 non, 3 refusés |
| CSS | 53 % de couverture | 14 oui, 9 en partie, 9 non, 2 refusés |
| JavaScript | 32 % de couverture | 6 oui, 11 en partie, 15 non, 1 refusés |
| HTML, CSS, JS ensemble | 47 % de couverture | 45 oui, 31 en partie, 47 non, 6 refusés |

# Partie 1 — Les mots de HoloCode

## Blocs : la page

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Page` | La page entière, le bloc racine d'un fichier | `html, head, body, main` | Décidé (ADR-009) |
| `World` | Le monde qui est dans un point | — | Décidé (ADR-009) |
| `Part` | Un morceau réutilisable, dans un fichier importé | `template` | Décidé (ADR-029) |
| `Use` | Pose un morceau importé, Use(Menu) | `slot, include` | Décidé (ADR-029) |

## Blocs : les repères

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Header` | L'en-tête de la page | `header` | Décidé (ADR-036) |
| `Nav` | Un menu de navigation | `nav` | Décidé (ADR-036) |
| `Main` | Le contenu principal | `main` | Décidé (ADR-036) |
| `Footer` | Le pied de page | `footer` | Décidé (ADR-036) |

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

## Blocs : les médias

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `Image` | Une image | `img` | Décidé (ADR-009) |
| `Sound` | Un son qu'une règle fait entendre | `audio` | Décidé (ADR-031) |
| `Shape` | Une forme : rond, carré, triangle, losange | `div + CSS, svg` | Décidé (ADR-032) |

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

## Paramètres : le texte

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `text` | Le texte d'un bouton | `le texte de button` | Décidé (ADR-009) |
| `to` | L'adresse d'un lien | `href` | Décidé (ADR-022) |
| `by` | L'auteur d'une citation | `cite` | Décidé (ADR-025) |
| `ordered` | Une liste numérotée | `ol` | Décidé (ADR-009) |

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

## Paramètres : agir

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `value, label, max` | La valeur liée, l'étiquette (obligatoire), le maximum | `value, label, max` | Décidé (ADR-027) |

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
| `meets, within` | Une rencontre entre deux blocs, au contact ou à un écart | `un calcul de collision` | Décidé (ADR-028) |
| `from, every` | Le fichier de données et son rythme | `fetch + setInterval` | Décidé (ADR-030) |

## Paramètres : le mouvement

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `enter, loop` | Le mouvement d'un bloc | `animation` | Décidé (ADR-034) |
| `at, for, ease` | Quand il part, combien de temps, quel caractère | `animation-delay, -duration, -timing-function` | Décidé (ADR-034) |
| `letters, each, back, repeat` | Lettre à lettre, enfant après enfant, sans retour, sans fin | `du JavaScript` | Décidé (ADR-034) |
| `opacity, x, y, scale, rotate, flip, tilt, blur, hue, round` | Ce qui bouge | `opacity, transform, filter, border-radius` | Décidé (ADR-034) |

## Paramètres : la vue

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `active, max, shrink, levels, speed` | Les réglages de Zoom | — | Décidé (ADR-021) |
| `after, size, fragment, grid, depth, density` | Les réglages de Points | — | Décidé (ADR-021) |
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
| `enter, leave` | Entrer dans un point, en sortir | `un routeur` | Décidé (ADR-022) |
| `portals` | Ouvrir le carrefour | — | Décidé (ADR-021) |
| `play` | Faire entendre un son | `audio.play()` | Décidé (ADR-031) |

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
| `padding, margin, width, height, max-width` | Les marges et les tailles | `les mêmes` | Décidé (ADR-017) |
| `opacity` | La transparence | `opacity` | Décidé (ADR-017) |
| `hover, focus, active` | Ce qui change au survol, au clavier, pendant l'appui | `:hover, :focus-visible, :active` | Décidé (ADR-036) |

## Unités

| Mot HoloCode | Ce qu’il fait | Sur le web | État |
|---|---|---|---|
| `px, deg` | Pixels, degrés | `px, deg` | Décidé (ADR-009) |
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

# Partie 2 — HoloCode face à HTML, CSS et JavaScript

## HTML — Structure de la page

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `Page(…)` | `html, head, body, main` | la page entière | Oui | 100 % | Déjà là | Une page HoloCode est un seul bloc Page. |
| `Page(title:)` | `title` | le titre de l'onglet | Oui | 100 % | Déjà là | — |
| `toujours UTF-8` | `meta charset` | l'encodage | Oui | 100 % | Déjà là | Rien à écrire. |
| `Zoom(active:, max:)` | `meta viewport` | le zoom sur téléphone | Oui | 100 % | Déjà là | Fait autrement : le zoom est au cœur de HoloCode. |
| — | `meta description, image de partage` | ce que montrent Google et les réseaux | Non | 0 % | Oui, en priorité | Sans elle, un site HoloCode est mal présenté dans les recherches et les partages. |
| — | `link rel=icon` | la petite image de l'onglet | Non | 0 % | Oui, utile | Tout vrai site en a une. |
| — | `lang` | la langue de la page | Non | 0 % | Oui, en priorité | Un lecteur d'écran prononce mal une page sans langue. |
| `Header, Footer, Main` | `header, footer, main` | en-tête, pied, contenu principal | Oui | 100 % | Déjà là | Ajoutés le 2026-10-06 (ADR-036). |
| `Nav` | `nav` | le menu de navigation | Oui | 100 % | Déjà là | Ajouté le 2026-10-06 (ADR-036). |
| — | `aside` | un encadré à part | Non | 0 % | Plus tard | — |
| `les titres H1 à H3` | `section, article` | des parties de page | Refusé exprès | — | Non | Jugement de Claude, sans décision : les titres donnent déjà le plan. Soumis à Codex et Gemini. |
| — | `div` | une boîte sans sens | Refusé exprès | — | Non | Refusé par Yocthan (ADR-009) : chaque bloc dit ce qu'il est. |
| `Text` | `span` | un bout de texte | Oui | 100 % | Déjà là | — |

## HTML — Texte

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `H1, H2, H3` | `h1, h2, h3` | les titres | Oui | 100 % | Déjà là | — |
| `H4, H5, H6` | `h4, h5, h6` | titres plus profonds | Oui | 100 % | Déjà là | Ajoutés le 2026-10-06 (correction d'ADR-020). |
| `P, ou une phrase seule` | `p` | un paragraphe | Oui | 100 % | Déjà là | — |
| `**gras**` | `strong, b` | le gras | Oui | 100 % | Déjà là | — |
| `*italique*` | `em, i` | l'italique | Oui | 100 % | Déjà là | — |
| `texte entre trois guillemets` | `br` | retour à la ligne | Oui | 100 % | Déjà là | — |
| `Hr()` | `hr` | un trait de séparation | Oui | 100 % | Déjà là | — |
| `Quote("…", by:)` | `blockquote, q, cite` | les citations | En partie | 80 % | Déjà là | Pas de citation courte au milieu d'une phrase. |
| `Code("…"), `accents graves`` | `pre, code, kbd` | du code montré tel quel | Oui | 90 % | Déjà là | — |
| — | `u, s, mark` | souligné, barré, surligné | Non | 0 % | Oui, utile | Barrer un ancien prix, surligner un mot. |
| — | `small, sub, sup` | petit, indice, exposant | Non | 0 % | Oui, utile | m², H₂O, notes de bas de page. |
| — | `abbr, time, address` | sigle, date, adresse | Non | 0 % | Plus tard | — |

## HTML — Listes et liens

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `List(children:)` | `ul, li` | liste à puces | Oui | 100 % | Déjà là | — |
| `List(ordered: true)` | `ol` | liste numérotée | Oui | 100 % | Déjà là | — |
| — | `dl, dt, dd` | liste de définitions | Non | 0 % | Plus tard | Un glossaire, une fiche technique. |
| `A("…", to:)` | `a href` | un lien | Oui | 100 % | Déjà là | — |
| `A(to: "#Monde") vers un monde seulement` | `a vers un endroit de la page` | sauter plus bas dans la page | En partie | 40 % | Oui, utile | Un sommaire qui mène à un titre de la page n'existe pas. |
| — | `a target, download` | nouvel onglet, télécharger | Non | 0 % | Plus tard | — |

## HTML — Images et médias

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `Image(source:)` | `img` | une image | Oui | 100 % | Déjà là | — |
| `Image(alt:)` | `img alt` | le texte pour qui ne voit pas | En partie | 80 % | Oui, en priorité | Il est encore facultatif : il devrait être obligatoire. |
| — | `picture, srcset` | une image plus légère sur téléphone | Non | 0 % | Oui, utile | Pour tenir la promesse de légèreté avec de vraies photos. |
| — | `figure, figcaption` | une image et sa légende | Non | 0 % | Oui, utile | — |
| — | `video` | une vidéo | Non | 0 % | Oui, en priorité | Très demandé ; votre vision parle aussi de vidéo devenue monde. |
| `Sound(…) et .play` | `audio` | un son, un lecteur | En partie | 40 % | Oui, utile | Un son court, oui ; un lecteur de musique, non. |
| `Point, World, Shape` | `canvas, WebGL` | un dessin libre, de la 3D | En partie | 40 % | Oui, utile | Des points et quatre formes ; pas de dessin libre ni de modèles 3D. |
| `comme fichier d'image` | `svg dans la page` | un dessin vectoriel | En partie | 30 % | Plus tard | — |
| `Point(inside: "x.holo")` | `iframe, embed, object` | une page dans la page | En partie | 60 % | Déjà là | On entre dans un autre fichier HoloCode ; pas dans un autre site. |

## HTML — Tableaux

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| — | `table, tr, td, th, caption` | un tableau de données | Non | 0 % | Oui, en priorité | Horaires, tarifs, comparatifs : beaucoup de sites en ont besoin. |

## HTML — Formulaires

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `Button(name:, text:)` | `button` | un bouton | Oui | 100 % | Déjà là | — |
| — | `form (envoyer)` | envoyer des réponses à un serveur | Non | 0 % | Oui, en priorité | Contact, commande, inscription : le passage de « lire » à « agir ». |
| `Input(value:, label:, max:)` | `input texte, nombre` | un champ | Oui | 100 % | Déjà là | — |
| `Checkbox(value:, label:)` | `input checkbox` | une case à cocher | Oui | 100 % | Déjà là | — |
| — | `input radio` | un choix parmi plusieurs | Non | 0 % | Oui, en priorité | Taille S, M ou L : on en a besoin partout. |
| — | `input range` | un curseur à glisser | Non | 0 % | Oui, utile | Volume, quantité ; utile aussi dans les mondes. |
| — | `input date, heure, couleur` | choisir une date, une couleur | Non | 0 % | Oui, utile | — |
| — | `input email, mot de passe, fichier` | adresse, secret, envoi de fichier | Non | 0 % | Plus tard | Seulement avec l'envoi au serveur et des comptes. |
| `label: (obligatoire)` | `label` | le nom d'un champ | Oui | 100 % | Déjà là | Mieux que HTML : impossible de l'oublier. |
| — | `textarea` | un texte long | Non | 0 % | Oui, en priorité | Le message d'un formulaire de contact. |
| — | `select, option` | une liste déroulante | Non | 0 % | Oui, en priorité | Choisir un pays, une taille. |
| — | `fieldset, legend, datalist` | regrouper, suggérer | Non | 0 % | Plus tard | — |
| `{valeur}, {total}` | `output` | afficher un résultat | Oui | 90 % | Déjà là | — |
| — | `progress, meter` | une barre de progression | Non | 0 % | Oui, utile | Une jauge de vie dans un jeu, un téléchargement. |

## HTML — Ouvrir et fermer

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `If + un bouton` | `details, summary` | un pli qui s'ouvre | En partie | 30 % | Oui, utile | Faisable avec une valeur et If, mais long ; une FAQ en a besoin. |
| — | `dialog` | une fenêtre par-dessus | Non | 0 % | Oui, utile | Confirmer, montrer un détail. |

## HTML — Code

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `styles après la page` | `style` | les styles | Oui | 100 % | Déjà là | — |
| `import "commun.holo"` | `link stylesheet` | un fichier de styles partagé | En partie | 70 % | Déjà là | Les styles viennent avec un morceau ; pas de fichier de styles seul. |
| — | `script` | du code dans la page | Refusé exprès | — | Non | Refusé (ADR-015) : aucun code libre, pour la sécurité. |
| `Part et Use` | `template, slot` | un morceau réutilisable | En partie | 60 % | Oui, utile | Pas encore de morceau avec des paramètres (une carte produit réutilisée). |
| `les repères, et quelques attributs ajoutés par le moteur` | `attributs aria, tabindex` | l'accessibilité fine | En partie | 45 % | Oui, en priorité | En vue points, un lecteur d'écran ne voit toujours rien. |

## CSS — Couleurs et texte

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `color, background` | `color, background-color` | couleur du texte et du fond | Oui | 100 % | Déjà là | — |
| `font-family, font-size (en px)` | `font-family, font-size` | la police et sa taille | Oui | 90 % | Déjà là | — |
| `font-weight, font-style` | `font-weight, font-style` | gras, italique | Oui | 100 % | Déjà là | — |
| `text-align` | `text-align` | l'alignement | Oui | 100 % | Déjà là | — |
| — | `line-height, letter-spacing` | l'interligne, l'espacement | Non | 0 % | Oui, utile | Pour des textes longs agréables à lire. |
| — | `text-transform, text-decoration` | majuscules, souligné | Non | 0 % | Oui, utile | — |
| — | `@font-face` | charger sa propre police | Non | 0 % | Oui, utile | Une marque a sa police. |
| — | `dégradés, image de fond` | fond en dégradé ou en image | Non | 0 % | Oui, utile | — |
| — | `box-shadow, text-shadow` | les ombres | Non | 0 % | Oui, utile | — |
| — | `variables (--couleur)` | une couleur nommée, réutilisée | Non | 0 % | Oui, utile | Changer le thème d'un site en une ligne. |
| — | `prefers-color-scheme` | le mode sombre | Non | 0 % | Oui, utile | Le visiteur a choisi le sombre : la page devrait suivre. |

## CSS — Boîtes

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `margin, padding` | `margin, padding` | les marges | Oui | 100 % | Déjà là | — |
| `width, height, max-width` | `width, height, max-width` | les tailles | Oui | 80 % | Déjà là | Pas de min-width ni max-height. |
| `border, border-radius` | `border, border-radius` | bordure et coins arrondis | Oui | 100 % | Déjà là | — |
| `opacity` | `opacity` | la transparence | Oui | 100 % | Déjà là | — |
| — | `overflow` | ce qui dépasse | Non | 0 % | Plus tard | — |
| — | `cursor` | la forme du curseur | Non | 0 % | Plus tard | — |
| `automatique pour le texte : px → rem, grands titres en clamp` | `unités %, rem, vw, clamp` | des tailles qui s'adaptent à l'écran | En partie | 60 % | Oui, utile | Le texte suit le réglage du visiteur ; les autres tailles restent en px. |

## CSS — Disposition

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| — | `display, position, float, z-index` | placer à la main | Refusé exprès | — | Non | Refusé (ADR-017) : la disposition vient des blocs. |
| `Row, Column (gap, align)` | `flexbox (en ligne, en colonne)` | côte à côte, l'un sous l'autre | Oui | 80 % | Déjà là | Pas d'élément qui prend la place qui reste. |
| `Grid(columns:, gap:)` | `grid` | une grille | En partie | 70 % | Déjà là | Pas de zones nommées ni de case sur deux colonnes. |
| `automatique : la grille perd des colonnes` | `@media (s'adapter à l'écran)` | changer selon la taille | En partie | 50 % | Oui, utile | On ne peut pas dire « sur téléphone, cache ceci ». |
| `Board (automatique)` | `aspect-ratio` | garder des proportions | En partie | 30 % | Plus tard | — |
| `Stack et align:` | `position: absolute (badge, pastille)` | poser un bloc sur un autre | Oui | 90 % | Déjà là | Ajouté le 2026-10-06 (ADR-036). Pas encore de bulle attachée à un bloc. |
| `P { }, .carte { }` | `sélecteurs par balise et par nom` | viser des blocs | Oui | 100 % | Déjà là | — |
| — | `sélecteurs composés, cascade, !important` | viser finement, forcer | Refusé exprès | — | Non | Refusé (ADR-017) : source de conflits sans fin en CSS. |

## CSS — États et mouvement

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `hover:, focus:, active: dans un style` | `hover, focus, active` | l'apparence au survol, au clic | Oui | 100 % | Déjà là | Ajoutés le 2026-10-06 (ADR-036). |
| `Enter à l'apparition seulement` | `transition` | passer en douceur d'un état à l'autre | En partie | 30 % | Oui, utile | — |
| `Enter, Loop, Scenes` | `animation, @keyframes` | une animation | Oui | 75 % | Déjà là | Dix propriétés et sept courbes ; pas d'étapes intermédiaires libres. |
| `x, y, rotate, scale dans Enter et Loop` | `transform 2D` | déplacer, tourner, grandir | En partie | 50 % | Oui, utile | Seulement pendant un mouvement, pas une pose fixe. |
| `Relief(tilt:), flip, tilt` | `transform 3D, perspective` | la profondeur | En partie | 50 % | Déjà là | Fait autrement : la page entière tourne. |
| `blur, hue dans Enter et Loop` | `filter (flou, couleurs)` | flouter, teinter | En partie | 30 % | Plus tard | — |
| `Shape(form:) : quatre formes` | `clip-path` | découper une forme | En partie | 20 % | Plus tard | — |
| `automatique` | `prefers-reduced-motion` | moins de mouvement | Oui | 100 % | Déjà là | Mieux que CSS : on n'a rien à écrire. |

## JavaScript — Gestes

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `On(Nom.tap, effect:)` | `clic` | réagir à un toucher | Oui | 100 % | Déjà là | — |
| `On(Key.left…)` | `clavier` | réagir aux touches | En partie | 40 % | Oui, utile | Seulement les flèches et l'espace. |
| — | `survol, approche (mouseenter)` | quand la souris passe dessus | Non | 0 % | Oui, en priorité | Prévu sous le nom near : le même signal à plat et en profondeur. |
| — | `défilement (scroll)` | réagir quand on descend | Non | 0 % | Oui, utile | Faire apparaître en descendant. |
| `drag: true sur un plateau` | `glisser-déposer` | faire glisser | En partie | 50 % | Déjà là | Sur un plateau seulement. |
| `le zoom du moteur` | `pincer, zoomer` | le zoom à deux doigts | Oui | 100 % | Déjà là | Le cœur du métavers. |

## JavaScript — Données et calcul

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `State(…)` | `variables` | garder une valeur | En partie | 70 % | Déjà là | Nombres entiers et textes ; pas de décimaux. |
| `add, sub, set, random` | `calcul (+ − × ÷)` | calculer | En partie | 30 % | Oui, utile | Pas de multiplication ni de division (sauf le total d'un panier). |
| `If(…), When(…)` | `if, else` | décider | En partie | 70 % | Oui, utile | Pas de « sinon » : il faut écrire deux If. |
| — | `tableaux, objets` | des listes de valeurs | Non | 0 % | Oui, en priorité | Sans liste, douze produits s'écrivent un par un. |
| — | `for, map` | répéter pour chaque élément | Non | 0 % | Oui, en priorité | Va avec les listes. |
| — | `fonctions` | du calcul réutilisable | Non | 0 % | Plus tard | Prévu : des fonctions pures enfermées (ADR-013). |
| `{nom} dans un texte` | `texte (majuscules, longueur, découper)` | travailler un texte | En partie | 20 % | Plus tard | — |
| `random, rejouable` | `Math.random` | le hasard | Oui | 90 % | Déjà là | Mieux pour un jeu : la même partie se rejoue. |
| — | `Date` | la date et l'heure du jour | Non | 0 % | Oui, utile | « Ouvert aujourd'hui », un compte à rebours. |
| — | `Intl (formats)` | 1 234,50 €, dates en français | Non | 0 % | Oui, utile | — |

## JavaScript — Temps

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `Every(1s, effect:)` | `setInterval` | répéter toutes les N secondes | Oui | 90 % | Déjà là | — |
| — | `setTimeout` | une seule fois, plus tard | Non | 0 % | Oui, utile | « Dans 3 secondes, montre ceci » n'existe pas seul. |
| `le moteur` | `requestAnimationFrame` | dessiner image par image | Sans objet | — | Non | Le moteur s'en charge. |

## JavaScript — Réseau et mémoire

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `Data(from:, every:)` | `fetch (lire)` | lire des données d'un serveur | En partie | 40 % | Déjà là | JSON plat, même serveur, pas de liste. |
| — | `fetch (envoyer)` | envoyer au serveur | Non | 0 % | Oui, en priorité | Va avec form : commander, contacter. |
| — | `WebSocket` | parler en direct avec un serveur | Non | 0 % | Plus tard | Pour le jeu à plusieurs, après votre validation locale. |
| `keep: [panier]` | `localStorage` | garder dans le navigateur | Oui | 80 % | Déjà là | — |
| — | `cookies, sessionStorage` | se souvenir le temps d'une visite | Non | 0 % | Plus tard | — |
| — | `service worker (hors ligne)` | marcher sans réseau | Non | 0 % | Plus tard | — |

## JavaScript — Page et navigation

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| — | `modifier la page (DOM)` | changer la page à la main | Refusé exprès | — | Non | Refusé (ADR-015) : c'est le moteur qui change la page. |
| `Point(inside:), enter, leave` | `routeur, historique` | changer de page sans recharger | Oui | 100 % | Déjà là | Chaque monde a son adresse ; « retour » marche. |
| `import "commun.holo"` | `import de modules` | découper son code | En partie | 60 % | Déjà là | — |

## JavaScript — Médias et appareil

| En HoloCode | Élément du web | Rôle | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `Sound.play` | `Web Audio` | jouer, régler un son | En partie | 30 % | Oui, utile | Pas de volume, pas de boucle, pas d'arrêt. |
| `Shape` | `Canvas 2D` | dessiner librement | En partie | 10 % | Plus tard | — |
| `le moteur dessine des points` | `WebGL, WebGPU` | la 3D | En partie | 30 % | Oui, utile | Pas encore d'objets pleins : l'essai de la chaise. |
| — | `géolocalisation, caméra, vibration` | l'appareil du visiteur | Non | 0 % | Plus tard | — |
| — | `presse-papiers, partage` | copier, partager | Non | 0 % | Plus tard | — |
| — | `paiement, comptes` | payer, se connecter | Non | 0 % | Plus tard | Bien plus tard, et jamais sans un serveur sûr. |
