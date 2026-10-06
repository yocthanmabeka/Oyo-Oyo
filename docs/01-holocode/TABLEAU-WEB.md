# HoloCode face à HTML, CSS et JavaScript : le grand tableau

- Relevé de Claude, le 2026-10-06, mis à jour après `ADR-036` (repères, titres jusqu’à H6, texte qui suit le réglage du visiteur, états, superposition). 130 éléments.
- **Couverture** : la part de ce que fait l’élément web qu’on obtient en HoloCode aujourd’hui. Estimation de Claude, élément par élément, non mesurée.
- **Doit exister** : avis de Claude, après les avis des humains, de Gemini et de ChatGPT. Les refus sont expliqués dans [`proposals/Claude/pourquoi-ces-refus-2026-10/`](../../proposals/Claude/pourquoi-ces-refus-2026-10/README.md).
- Le détail plus ancien, balise par balise : [`COMPARAISON-WEB.md`](COMPARAISON-WEB.md).

## Résumé

| | Couverture moyenne | Oui | En partie | Non | Refusés exprès |
|---|---|---|---|---|---|
| HTML | 52 % | 25 | 11 | 23 | 3 |
| CSS | 53 % | 14 | 9 | 9 | 2 |
| JavaScript | 32 % | 6 | 11 | 15 | 1 |
| **Ensemble** | 47 % | 45 | 31 | 47 | 6 |

Les moyennes ne comptent ni les refus exprès, ni ce qui est sans objet.

## HTML — Structure de la page

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `html, head, body, main` | la page entière | `Page(…)` | Oui | 100 % | Déjà là | Une page HoloCode est un seul bloc Page. |
| `title` | le titre de l'onglet | `Page(title:)` | Oui | 100 % | Déjà là | — |
| `meta charset` | l'encodage | `toujours UTF-8` | Oui | 100 % | Déjà là | Rien à écrire. |
| `meta viewport` | le zoom sur téléphone | `Zoom(active:, max:)` | Oui | 100 % | Déjà là | Fait autrement : le zoom est au cœur de HoloCode. |
| `meta description, image de partage` | ce que montrent Google et les réseaux | — | Non | 0 % | Oui, en priorité | Sans elle, un site HoloCode est mal présenté dans les recherches et les partages. |
| `link rel=icon` | la petite image de l'onglet | — | Non | 0 % | Oui, utile | Tout vrai site en a une. |
| `lang` | la langue de la page | — | Non | 0 % | Oui, en priorité | Un lecteur d'écran prononce mal une page sans langue. |
| `header, footer, main` | en-tête, pied, contenu principal | `Header, Footer, Main` | Oui | 100 % | Déjà là | Ajoutés le 2026-10-06 (ADR-036). |
| `nav` | le menu de navigation | `Nav` | Oui | 100 % | Déjà là | Ajouté le 2026-10-06 (ADR-036). |
| `aside` | un encadré à part | — | Non | 0 % | Plus tard | — |
| `section, article` | des parties de page | `les titres H1 à H3` | Refusé exprès | — | Non | Jugement de Claude, sans décision : les titres donnent déjà le plan. Soumis à Codex et Gemini. |
| `div` | une boîte sans sens | — | Refusé exprès | — | Non | Refusé par Yocthan (ADR-009) : chaque bloc dit ce qu'il est. |
| `span` | un bout de texte | `Text` | Oui | 100 % | Déjà là | — |

## HTML — Texte

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `h1, h2, h3` | les titres | `H1, H2, H3` | Oui | 100 % | Déjà là | — |
| `h4, h5, h6` | titres plus profonds | `H4, H5, H6` | Oui | 100 % | Déjà là | Ajoutés le 2026-10-06 (correction d'ADR-020). |
| `p` | un paragraphe | `P, ou une phrase seule` | Oui | 100 % | Déjà là | — |
| `strong, b` | le gras | `**gras**` | Oui | 100 % | Déjà là | — |
| `em, i` | l'italique | `*italique*` | Oui | 100 % | Déjà là | — |
| `br` | retour à la ligne | `texte entre trois guillemets` | Oui | 100 % | Déjà là | — |
| `hr` | un trait de séparation | `Hr()` | Oui | 100 % | Déjà là | — |
| `blockquote, q, cite` | les citations | `Quote("…", by:)` | En partie | 80 % | Déjà là | Pas de citation courte au milieu d'une phrase. |
| `pre, code, kbd` | du code montré tel quel | `Code("…"), `accents graves`` | Oui | 90 % | Déjà là | — |
| `u, s, mark` | souligné, barré, surligné | — | Non | 0 % | Oui, utile | Barrer un ancien prix, surligner un mot. |
| `small, sub, sup` | petit, indice, exposant | — | Non | 0 % | Oui, utile | m², H₂O, notes de bas de page. |
| `abbr, time, address` | sigle, date, adresse | — | Non | 0 % | Plus tard | — |

## HTML — Listes et liens

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `ul, li` | liste à puces | `List(children:)` | Oui | 100 % | Déjà là | — |
| `ol` | liste numérotée | `List(ordered: true)` | Oui | 100 % | Déjà là | — |
| `dl, dt, dd` | liste de définitions | — | Non | 0 % | Plus tard | Un glossaire, une fiche technique. |
| `a href` | un lien | `A("…", to:)` | Oui | 100 % | Déjà là | — |
| `a vers un endroit de la page` | sauter plus bas dans la page | `A(to: "#Monde") vers un monde seulement` | En partie | 40 % | Oui, utile | Un sommaire qui mène à un titre de la page n'existe pas. |
| `a target, download` | nouvel onglet, télécharger | — | Non | 0 % | Plus tard | — |

## HTML — Images et médias

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `img` | une image | `Image(source:)` | Oui | 100 % | Déjà là | — |
| `img alt` | le texte pour qui ne voit pas | `Image(alt:)` | En partie | 80 % | Oui, en priorité | Il est encore facultatif : il devrait être obligatoire. |
| `picture, srcset` | une image plus légère sur téléphone | — | Non | 0 % | Oui, utile | Pour tenir la promesse de légèreté avec de vraies photos. |
| `figure, figcaption` | une image et sa légende | — | Non | 0 % | Oui, utile | — |
| `video` | une vidéo | — | Non | 0 % | Oui, en priorité | Très demandé ; votre vision parle aussi de vidéo devenue monde. |
| `audio` | un son, un lecteur | `Sound(…) et .play` | En partie | 40 % | Oui, utile | Un son court, oui ; un lecteur de musique, non. |
| `canvas, WebGL` | un dessin libre, de la 3D | `Point, World, Shape` | En partie | 40 % | Oui, utile | Des points et quatre formes ; pas de dessin libre ni de modèles 3D. |
| `svg dans la page` | un dessin vectoriel | `comme fichier d'image` | En partie | 30 % | Plus tard | — |
| `iframe, embed, object` | une page dans la page | `Point(inside: "x.holo")` | En partie | 60 % | Déjà là | On entre dans un autre fichier HoloCode ; pas dans un autre site. |

## HTML — Tableaux

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `table, tr, td, th, caption` | un tableau de données | — | Non | 0 % | Oui, en priorité | Horaires, tarifs, comparatifs : beaucoup de sites en ont besoin. |

## HTML — Formulaires

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `button` | un bouton | `Button(name:, text:)` | Oui | 100 % | Déjà là | — |
| `form (envoyer)` | envoyer des réponses à un serveur | — | Non | 0 % | Oui, en priorité | Contact, commande, inscription : le passage de « lire » à « agir ». |
| `input texte, nombre` | un champ | `Input(value:, label:, max:)` | Oui | 100 % | Déjà là | — |
| `input checkbox` | une case à cocher | `Checkbox(value:, label:)` | Oui | 100 % | Déjà là | — |
| `input radio` | un choix parmi plusieurs | — | Non | 0 % | Oui, en priorité | Taille S, M ou L : on en a besoin partout. |
| `input range` | un curseur à glisser | — | Non | 0 % | Oui, utile | Volume, quantité ; utile aussi dans les mondes. |
| `input date, heure, couleur` | choisir une date, une couleur | — | Non | 0 % | Oui, utile | — |
| `input email, mot de passe, fichier` | adresse, secret, envoi de fichier | — | Non | 0 % | Plus tard | Seulement avec l'envoi au serveur et des comptes. |
| `label` | le nom d'un champ | `label: (obligatoire)` | Oui | 100 % | Déjà là | Mieux que HTML : impossible de l'oublier. |
| `textarea` | un texte long | — | Non | 0 % | Oui, en priorité | Le message d'un formulaire de contact. |
| `select, option` | une liste déroulante | — | Non | 0 % | Oui, en priorité | Choisir un pays, une taille. |
| `fieldset, legend, datalist` | regrouper, suggérer | — | Non | 0 % | Plus tard | — |
| `output` | afficher un résultat | `{valeur}, {total}` | Oui | 90 % | Déjà là | — |
| `progress, meter` | une barre de progression | — | Non | 0 % | Oui, utile | Une jauge de vie dans un jeu, un téléchargement. |

## HTML — Ouvrir et fermer

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `details, summary` | un pli qui s'ouvre | `If + un bouton` | En partie | 30 % | Oui, utile | Faisable avec une valeur et If, mais long ; une FAQ en a besoin. |
| `dialog` | une fenêtre par-dessus | — | Non | 0 % | Oui, utile | Confirmer, montrer un détail. |

## HTML — Code

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `style` | les styles | `styles après la page` | Oui | 100 % | Déjà là | — |
| `link stylesheet` | un fichier de styles partagé | `import "commun.holo"` | En partie | 70 % | Déjà là | Les styles viennent avec un morceau ; pas de fichier de styles seul. |
| `script` | du code dans la page | — | Refusé exprès | — | Non | Refusé (ADR-015) : aucun code libre, pour la sécurité. |
| `template, slot` | un morceau réutilisable | `Part et Use` | En partie | 60 % | Oui, utile | Pas encore de morceau avec des paramètres (une carte produit réutilisée). |
| `attributs aria, tabindex` | l'accessibilité fine | `les repères, et quelques attributs ajoutés par le moteur` | En partie | 45 % | Oui, en priorité | En vue points, un lecteur d'écran ne voit toujours rien. |

## CSS — Couleurs et texte

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `color, background-color` | couleur du texte et du fond | `color, background` | Oui | 100 % | Déjà là | — |
| `font-family, font-size` | la police et sa taille | `font-family, font-size (en px)` | Oui | 90 % | Déjà là | — |
| `font-weight, font-style` | gras, italique | `font-weight, font-style` | Oui | 100 % | Déjà là | — |
| `text-align` | l'alignement | `text-align` | Oui | 100 % | Déjà là | — |
| `line-height, letter-spacing` | l'interligne, l'espacement | — | Non | 0 % | Oui, utile | Pour des textes longs agréables à lire. |
| `text-transform, text-decoration` | majuscules, souligné | — | Non | 0 % | Oui, utile | — |
| `@font-face` | charger sa propre police | — | Non | 0 % | Oui, utile | Une marque a sa police. |
| `dégradés, image de fond` | fond en dégradé ou en image | — | Non | 0 % | Oui, utile | — |
| `box-shadow, text-shadow` | les ombres | — | Non | 0 % | Oui, utile | — |
| `variables (--couleur)` | une couleur nommée, réutilisée | — | Non | 0 % | Oui, utile | Changer le thème d'un site en une ligne. |
| `prefers-color-scheme` | le mode sombre | — | Non | 0 % | Oui, utile | Le visiteur a choisi le sombre : la page devrait suivre. |

## CSS — Boîtes

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `margin, padding` | les marges | `margin, padding` | Oui | 100 % | Déjà là | — |
| `width, height, max-width` | les tailles | `width, height, max-width` | Oui | 80 % | Déjà là | Pas de min-width ni max-height. |
| `border, border-radius` | bordure et coins arrondis | `border, border-radius` | Oui | 100 % | Déjà là | — |
| `opacity` | la transparence | `opacity` | Oui | 100 % | Déjà là | — |
| `overflow` | ce qui dépasse | — | Non | 0 % | Plus tard | — |
| `cursor` | la forme du curseur | — | Non | 0 % | Plus tard | — |
| `unités %, rem, vw, clamp` | des tailles qui s'adaptent à l'écran | `automatique pour le texte : px → rem, grands titres en clamp` | En partie | 60 % | Oui, utile | Le texte suit le réglage du visiteur ; les autres tailles restent en px. |

## CSS — Disposition

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `display, position, float, z-index` | placer à la main | — | Refusé exprès | — | Non | Refusé (ADR-017) : la disposition vient des blocs. |
| `flexbox (en ligne, en colonne)` | côte à côte, l'un sous l'autre | `Row, Column (gap, align)` | Oui | 80 % | Déjà là | Pas d'élément qui prend la place qui reste. |
| `grid` | une grille | `Grid(columns:, gap:)` | En partie | 70 % | Déjà là | Pas de zones nommées ni de case sur deux colonnes. |
| `@media (s'adapter à l'écran)` | changer selon la taille | `automatique : la grille perd des colonnes` | En partie | 50 % | Oui, utile | On ne peut pas dire « sur téléphone, cache ceci ». |
| `aspect-ratio` | garder des proportions | `Board (automatique)` | En partie | 30 % | Plus tard | — |
| `position: absolute (badge, pastille)` | poser un bloc sur un autre | `Stack et align:` | Oui | 90 % | Déjà là | Ajouté le 2026-10-06 (ADR-036). Pas encore de bulle attachée à un bloc. |
| `sélecteurs par balise et par nom` | viser des blocs | `P { }, .carte { }` | Oui | 100 % | Déjà là | — |
| `sélecteurs composés, cascade, !important` | viser finement, forcer | — | Refusé exprès | — | Non | Refusé (ADR-017) : source de conflits sans fin en CSS. |

## CSS — États et mouvement

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `hover, focus, active` | l'apparence au survol, au clic | `hover:, focus:, active: dans un style` | Oui | 100 % | Déjà là | Ajoutés le 2026-10-06 (ADR-036). |
| `transition` | passer en douceur d'un état à l'autre | `Enter à l'apparition seulement` | En partie | 30 % | Oui, utile | — |
| `animation, @keyframes` | une animation | `Enter, Loop, Scenes` | Oui | 75 % | Déjà là | Dix propriétés et sept courbes ; pas d'étapes intermédiaires libres. |
| `transform 2D` | déplacer, tourner, grandir | `x, y, rotate, scale dans Enter et Loop` | En partie | 50 % | Oui, utile | Seulement pendant un mouvement, pas une pose fixe. |
| `transform 3D, perspective` | la profondeur | `Relief(tilt:), flip, tilt` | En partie | 50 % | Déjà là | Fait autrement : la page entière tourne. |
| `filter (flou, couleurs)` | flouter, teinter | `blur, hue dans Enter et Loop` | En partie | 30 % | Plus tard | — |
| `clip-path` | découper une forme | `Shape(form:) : quatre formes` | En partie | 20 % | Plus tard | — |
| `prefers-reduced-motion` | moins de mouvement | `automatique` | Oui | 100 % | Déjà là | Mieux que CSS : on n'a rien à écrire. |

## JavaScript — Gestes

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `clic` | réagir à un toucher | `On(Nom.tap, effect:)` | Oui | 100 % | Déjà là | — |
| `clavier` | réagir aux touches | `On(Key.left…)` | En partie | 40 % | Oui, utile | Seulement les flèches et l'espace. |
| `survol, approche (mouseenter)` | quand la souris passe dessus | — | Non | 0 % | Oui, en priorité | Prévu sous le nom near : le même signal à plat et en profondeur. |
| `défilement (scroll)` | réagir quand on descend | — | Non | 0 % | Oui, utile | Faire apparaître en descendant. |
| `glisser-déposer` | faire glisser | `drag: true sur un plateau` | En partie | 50 % | Déjà là | Sur un plateau seulement. |
| `pincer, zoomer` | le zoom à deux doigts | `le zoom du moteur` | Oui | 100 % | Déjà là | Le cœur du métavers. |

## JavaScript — Données et calcul

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `variables` | garder une valeur | `State(…)` | En partie | 70 % | Déjà là | Nombres entiers et textes ; pas de décimaux. |
| `calcul (+ − × ÷)` | calculer | `add, sub, set, random` | En partie | 30 % | Oui, utile | Pas de multiplication ni de division (sauf le total d'un panier). |
| `if, else` | décider | `If(…), When(…)` | En partie | 70 % | Oui, utile | Pas de « sinon » : il faut écrire deux If. |
| `tableaux, objets` | des listes de valeurs | — | Non | 0 % | Oui, en priorité | Sans liste, douze produits s'écrivent un par un. |
| `for, map` | répéter pour chaque élément | — | Non | 0 % | Oui, en priorité | Va avec les listes. |
| `fonctions` | du calcul réutilisable | — | Non | 0 % | Plus tard | Prévu : des fonctions pures enfermées (ADR-013). |
| `texte (majuscules, longueur, découper)` | travailler un texte | `{nom} dans un texte` | En partie | 20 % | Plus tard | — |
| `Math.random` | le hasard | `random, rejouable` | Oui | 90 % | Déjà là | Mieux pour un jeu : la même partie se rejoue. |
| `Date` | la date et l'heure du jour | — | Non | 0 % | Oui, utile | « Ouvert aujourd'hui », un compte à rebours. |
| `Intl (formats)` | 1 234,50 €, dates en français | — | Non | 0 % | Oui, utile | — |

## JavaScript — Temps

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `setInterval` | répéter toutes les N secondes | `Every(1s, effect:)` | Oui | 90 % | Déjà là | — |
| `setTimeout` | une seule fois, plus tard | — | Non | 0 % | Oui, utile | « Dans 3 secondes, montre ceci » n'existe pas seul. |
| `requestAnimationFrame` | dessiner image par image | `le moteur` | Sans objet | — | Non | Le moteur s'en charge. |

## JavaScript — Réseau et mémoire

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `fetch (lire)` | lire des données d'un serveur | `Data(from:, every:)` | En partie | 40 % | Déjà là | JSON plat, même serveur, pas de liste. |
| `fetch (envoyer)` | envoyer au serveur | — | Non | 0 % | Oui, en priorité | Va avec form : commander, contacter. |
| `WebSocket` | parler en direct avec un serveur | — | Non | 0 % | Plus tard | Pour le jeu à plusieurs, après votre validation locale. |
| `localStorage` | garder dans le navigateur | `keep: [panier]` | Oui | 80 % | Déjà là | — |
| `cookies, sessionStorage` | se souvenir le temps d'une visite | — | Non | 0 % | Plus tard | — |
| `service worker (hors ligne)` | marcher sans réseau | — | Non | 0 % | Plus tard | — |

## JavaScript — Page et navigation

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `modifier la page (DOM)` | changer la page à la main | — | Refusé exprès | — | Non | Refusé (ADR-015) : c'est le moteur qui change la page. |
| `routeur, historique` | changer de page sans recharger | `Point(inside:), enter, leave` | Oui | 100 % | Déjà là | Chaque monde a son adresse ; « retour » marche. |
| `import de modules` | découper son code | `import "commun.holo"` | En partie | 60 % | Déjà là | — |

## JavaScript — Médias et appareil

| Élément du web | Rôle | En HoloCode | Existe ? | Couverture | Doit exister ? | Pourquoi |
|---|---|---|---|---|---|---|
| `Web Audio` | jouer, régler un son | `Sound.play` | En partie | 30 % | Oui, utile | Pas de volume, pas de boucle, pas d'arrêt. |
| `Canvas 2D` | dessiner librement | `Shape` | En partie | 10 % | Plus tard | — |
| `WebGL, WebGPU` | la 3D | `le moteur dessine des points` | En partie | 30 % | Oui, utile | Pas encore d'objets pleins : l'essai de la chaise. |
| `géolocalisation, caméra, vibration` | l'appareil du visiteur | — | Non | 0 % | Plus tard | — |
| `presse-papiers, partage` | copier, partager | — | Non | 0 % | Plus tard | — |
| `paiement, comptes` | payer, se connecter | — | Non | 0 % | Plus tard | Bien plus tard, et jamais sans un serveur sûr. |
