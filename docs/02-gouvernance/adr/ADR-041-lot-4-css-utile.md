# ADR-041 — Lot 4 : le CSS utile (texte, ombres, fonds, variables, thème sombre, téléphone, police)

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : le grand tableau (`docs/01-holocode/TABLEAU-WEB.md`) ; Yocthan, le 2026-10-06 : « tu travailles sur le lot 2 jusqu'au lot 5… je suis tes recommandations »
- Validation : validé par Yocthan le 2026-10-06, après avoir tout essayé : « En fait, j'ai tout testé de tout ce qui était à laisser [à l'essai] et je trouve que c'est bon. Donc, euh, valide-le. »
- Projets affectés : HoloCode, HoloEngine

## Contexte

Le tableau compte dix éléments du CSS jugés utiles et absents : l'interligne et l'espacement, les majuscules et le barré, sa propre police, les dégradés et les images de fond, les ombres, les variables, le thème sombre, l'adaptation au téléphone, les transitions, et une pose fixe (tourner, agrandir).

## Décision (à l'essai)

Les mots restent ceux du CSS ; ce qui change, ce sont les garde-fous.

1. **Le texte** : `line-height` (un nombre sans unité, de 0.8 à 3), `letter-spacing` (de -10px à 40px), `text-transform` (`uppercase`, `lowercase`, `capitalize`, `none`), `text-decoration` (`underline`, `line-through`, `none`), `text-shadow`.
2. **Les ombres** : `box-shadow` et `text-shadow` : deux décalages, un flou facultatif et une couleur obligatoire ; trois ombres au plus ; ou `none`.
3. **Le fond** : `background` prend une couleur, un dégradé (`linear-gradient(…)`, avec une direction `to right` ou un angle, `radial-gradient(…)`, de deux à cinq couleurs), ou une image rangée à côté, `url("fond.jpg")`, qui couvre toujours le bloc, centrée, sans mosaïque.
4. **La pose** : `rotate` (de -360deg à 360deg) et `scale` (de 0.1 à 5), utiles aussi au survol ; `transition` (de 0 à 2s, ou `none`) dit la durée du passage d'une allure à l'autre.
5. **Les variables** : `Page { --or: #E9B44C; }`, puis `color: --or;` partout, sans `var( )`. Une variable porte une couleur ou une taille, se définit dans le style de la page, et une variable inconnue est refusée avec son nom.
6. **Deux nouveaux états de style**, à côté de `hover`, `focus`, `active` : `dark: { … }` quand le visiteur a choisi le thème sombre, `phone: { … }` sur un écran plus étroit que la page (640 pixels). Dans `phone:`, et seulement là, `display: none` cache un bloc.
7. **Sa propre police** : `Page(fonts: [ Font(family: "Carlito", source: "carlito.woff2") ])`, puis `font-family: Carlito, Georgia, serif;`. Huit polices au plus, rangées à côté (`.woff2`, `.woff`, `.ttf`, `.otf`), affichées avec `font-display: swap`.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Les mots | des mots à nous ; **ceux du CSS** | Ceux du CSS : on les retrouve partout, et ce sont des noms d'apparence, pas de disposition (`ADR-017`). |
| La hauteur de ligne | px, em, sans unité ; **sans unité seulement** | Une hauteur en px ne suit pas le texte quand le visiteur le grossit : défaut du CSS refusé. |
| Une image de fond | `background-image` et ses six réglages ; **`url(…)` qui couvre toujours** | Le défaut du CSS : sans `background-size`, l'image se répète en mosaïque ou déborde. |
| Les variables | `var(--or)` comme en CSS ; **`--or` seul** | Plus court, et une variable inconnue est refusée (en CSS, elle échoue sans rien dire). |
| Le thème sombre | une règle `@media` à part ; **un état `dark:` dans le style** | Le sombre d'un bloc s'écrit à côté de son clair ; avec les variables, il tient en une ligne. |
| Le téléphone | `@media (max-width: …)` à écrire soi-même ; **un état `phone:`** | Un seul seuil, celui de la page ; le reste de l'adaptation est déjà automatique. |
| Cacher sur téléphone | refuser `display` partout ; **`display: none` dans `phone:` seulement** | Cacher n'est pas disposer ; ailleurs, on cache par `If`. |
| La police | `@font-face` dans les styles ; **un bloc `Font` sur la page** | Comme `Data` et `Prices` : ce que la page charge se déclare sur la page. `font-display: swap` évite le texte invisible pendant le chargement. |
| Les mouvements réduits | rien ; **les transitions s'arrêtent** quand le visiteur demande moins de mouvement | Comme pour `Enter` et `Loop` (`ADR-034`). |

## Conséquences

- La page légère reste légère : tout ce lot est du CSS fabriqué par le moteur, sans script.
- Restent en partie : les unités (`rem`, `vw`, `clamp` pour autre chose que le texte), `overflow`, `cursor`, `aspect-ratio`, `filter`, `clip-path` (classés « plus tard »).
- La leçon 54 apporte une police libre, Carlito (licence SIL Open Font License, mention dans `exemples/lecons/carlito-LICENCE.txt`).

## Critères de validation

- Leçons 50 à 54 ; test du moteur `plat.rs` (`le_lot_4_le_css_utile`) ; dans Chrome : les styles calculés (texte, ombres, fond en image, pose au survol), les thèmes clair et sombre, le bandeau caché sur un écran de téléphone, la police chargée.
