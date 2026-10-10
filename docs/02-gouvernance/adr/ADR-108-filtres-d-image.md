# ADR-108 — Les filtres d'image dans les styles : `grayscale`, `saturate`, `brightness`, `contrast`, `hue`, `blur`, et `backdrop-blur`

- Statut : ACCEPTÉ (fait et validé : Yocthan, 2026-10-09, « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-10
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #237 (« Dernière dette du web : les filtres d'image dans les styles (flou, gris, luminosité) »), dans la file des dernières dettes du web ouverte par Yocthan le 2026-10-09 ; le grand tableau du web, où `filter` était « en partie » (« `blur`, `hue` dans `Enter` et `Loop` », `docs/01-holocode/TABLEAU-WEB.md`) ; les styles qui ne disent que l'apparence, tout vérifié (`ADR-017`) ; le lot 4 du CSS utile, où `rotate` et `scale` sont déjà des réglages à part (`ADR-041`) ; le mouvement, où `blur` et `hue` existent déjà (`ADR-034`) ; le contraste vérifié par le moteur (`ADR-055`) ; l'écriture des noms (`ADR-016`, `ADR-037`) ; la règle de parité de Yocthan (2026-10-07).
- Validation : faite par Yocthan le 2026-10-09 (« tu le valides déjà, tu le fais déjà »).
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Une image en gris qui reprend ses couleurs au survol, une photo assombrie sous un titre, la page qui devient floue derrière une fenêtre : des effets de tous les sites, que HoloCode ne savait écrire que pour un bloc qui bouge (`Enter(blur: 10px, hue: 90deg)`), jamais dans un style.
- Le web le fait par `filter: grayscale(100%) blur(4px)` et `backdrop-filter: blur(8px)`. Trois défauts :
  - `filter` est **une liste** : un état qui veut en changer un seul doit la réécrire entière ; `.photo:hover { filter: grayscale(0); }` fait disparaître le flou écrit à côté, sans un mot ;
  - **deux écritures** pour la même chose, `grayscale(1)` et `grayscale(100%)`, `brightness(0.5)` et `brightness(50%)` ;
  - **rien ne vérifie** qu'un texte reste lisible une fois le bloc assombri (`brightness(0.3)` sur un texte blanc sur vert), ni qu'un texte flou se lit ; `backdrop-filter` ne fait rien sur un fond opaque, et Safari a longtemps exigé `-webkit-backdrop-filter`.
- HoloCode a déjà tranché la même question pour la pose : `rotate` et `scale` sont deux réglages, jamais la liste `transform` (`ADR-041`).

## Décision

1. **Six réglages de style**, écrits comme en CSS, dans un style ou dans l'un de ses états (`hover:`, `active:`, `focus:`, `dark:`, `phone:`, `computer:`, `narrow:`, `print:`), sur tout bloc qui se voit :
   - `grayscale` : de 0 (les couleurs) à 1 (tout gris), un nombre comme `opacity` ;
   - `saturate` : de 0 (gris) à 3 (des couleurs plus vives) ; 1 ne change rien ;
   - `brightness` : de 0.2 à 3 ; 1 ne change rien, moins assombrit, plus éclaircit ;
   - `contrast` : de 0.2 à 3 ; 1 ne change rien ;
   - `hue` : un angle, de -360deg à 360deg ; les couleurs tournent sur le cercle des teintes ; le mot est celui d'`Enter` et de `Loop` ;
   - `blur` : un flou, de 0 à 100px.
2. **Chaque réglage est à part.** Le moteur les compose en un seul `filter`, toujours dans le même ordre (gris, saturation, luminosité, contraste, teinte, flou). Dans un état, un réglage change sans effacer les autres : `hover: { grayscale: 0; }` rend les couleurs et garde le flou écrit à côté, parce que le moteur réécrit la liste entière à partir du style et de l'état.
3. **`backdrop-blur`** : de 0 à 100px, ce qui est derrière le bloc devient flou. Sur une fenêtre (`Dialog`), c'est toute la page derrière elle (`::backdrop`) ; sur un bloc au fond à demi transparent, l'effet de verre dépoli. Le moteur écrit aussi `-webkit-backdrop-filter`. Avec un fond opaque dans le même style, rien ne se verrait : refusé, avec la raison.
4. **Le contraste reste vérifié, filtre compris** (`ADR-055`) : quand un style donne la couleur du texte et celle du fond, le moteur applique aux deux couleurs les mêmes matrices que le navigateur (gris, saturation, luminosité, contraste, teinte, dans l'espace sRGB), puis la règle du WCAG, 4,5 pour 1 (3 pour un grand texte). Le message dit le contraste mesuré et le réglage en cause.
5. **Le flou se pose sur une image, une forme, une vidéo ou un dessin** (`Image`, `Shape`, `Video`, `Drawing`, ou un nom de style porté seulement par eux), jamais sur un bloc qui porte un texte ni sur un composant : un texte flou ne se lit pas. Pour estomper, `opacity` ; pour cacher, `If` ; pour ce qui est derrière une fenêtre, `backdrop-blur`.
6. **Les passages d'une allure à l'autre** (`transition`, ou le survol et l'appui sans `transition` écrit) adoucissent aussi les filtres et le flou de derrière ; un visiteur qui demande moins de mouvement (`prefers-reduced-motion`) ne voit aucun passage, comme pour le reste.
7. **Refusés, avec le bon mot** : `filter` et `backdrop-filter` écrits comme en CSS (le message donne les réglages) ; une valeur hors des bornes, ou en `%` ; un flou sur un texte ; un flou de derrière sur un fond opaque.
8. **Parité** : un filtre est une apparence, la même sur tous les appareils. Le survol n'existe qu'avec une souris, comme tous les `hover:` (`ADR-036`) ; au doigt, l'état `active:` fait la même chose pendant l'appui, et la leçon l'écrit.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| La forme | A, le CSS tel quel : `filter: grayscale(1) blur(4px);`, une liste de fonctions ; B, **un réglage par effet** : `grayscale: 1; blur: 4px;` ; C, un paramètre d'`Image(filter: gray)` | B : c'est le choix déjà fait pour la pose (`rotate`, `scale` plutôt que `transform`, `ADR-041`) ; dans un état, un réglage change sans effacer les autres ; un débutant écrit `grayscale: 1` sans parenthèses ni liste. A répète le défaut du CSS ; C ne couvrirait ni le survol, ni un bloc, ni une fenêtre |
| Les noms | les mots du CSS (`grayscale`, `saturate`, `brightness`, `contrast`, `blur`) ; des mots plus courts (`gray`, `dim`) ; `hue-rotate` ou `hue` | les mots du CSS, qu'un programmeur connaît et qui gardent leur sens (`ADR-016`) ; `hue` plutôt que `hue-rotate`, parce que c'est déjà le mot d'`Enter` et de `Loop` (`ADR-034`) : un seul mot pour une seule idée |
| Les valeurs | `100%` et `1`, comme le CSS ; **un nombre seulement** | une seule écriture par réglage (`ADR-037`) ; le nombre est celui d'`opacity`, et 1 veut dire « rien ne change » là où c'est le cas ; des pixels pour le flou, des degrés pour la teinte ; toutes bornées |
| Le flou derrière | `Dialog { blur: 8px; }` qui voudrait dire « derrière » ; `backdrop-filter: blur(8px)` ; **`backdrop-blur: 8px`** | « derrière » n'est pas « le bloc » : le même mot pour les deux tromperait ; `backdrop` est le mot du web (le `::backdrop` d'une fenêtre, `backdrop-filter`, les classes `backdrop-blur` de Tailwind) ; un seul réglage, pas une seconde liste ; seul le flou sert vraiment derrière un bloc |
| Le contraste | ne rien vérifier ; des bornes très serrées (de 0.8 à 1.2) ; **mesurer après le filtre** | les bornes serrées interdiraient l'image assombrie, l'usage le plus courant ; la mesure garde la règle (`ADR-055`) sans rien demander à l'auteur, qui lit le contraste obtenu |
| Le flou sur un texte | permis, à l'auteur de voir ; **refusé** | un texte flou ne se lit pas, et un lecteur d'écran le lirait quand même : deux visiteurs ne verraient pas la même page. Le flou se pose sur l'image ou la forme elle-même |
| L'ordre des filtres | celui de l'écriture ; **toujours le même** | deux auteurs qui écrivent les mêmes réglages voient la même chose ; l'ordre ne change presque rien à l'œil, mais une règle vaut mieux qu'un hasard |
| D'autres filtres | `sepia`, `invert`, `drop-shadow` ; **les six de la demande, et le flou de derrière** | `sepia` est un gris teinté (`grayscale` et `hue`) ; `invert` retourne toutes les couleurs, texte compris ; `drop-shadow` attend la forme découpée (`ADR-111`) pour servir. Laissés en dette |

## Les défauts du web évités

- **La liste `filter` qu'un état réécrit entière** : ici, un réglage à la fois, et le moteur recompose la liste.
- **Deux écritures** (`1` et `100%`) : une seule.
- **Un texte devenu illisible** par `brightness` ou `contrast` : mesuré, refusé avec le chiffre.
- **Un texte flou** : refusé.
- **`backdrop-filter` qui ne fait rien** sur un fond opaque : refusé, avec la raison ; et `-webkit-backdrop-filter` écrit par le moteur, pour les Safari qui l'exigent encore.
- **Le passage brusque** d'un filtre à l'autre au survol : adouci d'office, comme la couleur et l'ombre ; et arrêté pour qui demande moins de mouvement.

## Dettes

- `sepia`, `invert`, et une ombre qui suit la forme (`drop-shadow`).
- Dans `Enter` et `Loop`, seuls `blur` et `hue` bougent ; un gris ou une luminosité qui s'animent restent à faire.
- Derrière un bloc, seul le flou ; pas de `backdrop-brightness`.
- Le contraste n'est mesuré que quand le même style donne les deux couleurs (la limite d'`ADR-055`) : un bloc filtré dont le texte hérite ses couleurs de la page n'est pas mesuré.
- Un bloc qui bouge (`Enter(blur:)`) et porte aussi un filtre de style : pendant le mouvement, c'est le mouvement qui tient le `filter` ; au repos, le style.
- La ligne `filter` du grand tableau du web peut passer à « oui » à la prochaine publication.

## Critères de validation

- Tests du moteur (`moteur/src/filters.rs`, `styles.rs`, `flat.rs`) : les six réglages et `backdrop-blur` acceptés dans un style et dans ses états ; la composition en un seul `filter`, dans l'ordre, un état qui en change un et garde les autres ; `backdrop-filter` avec son préfixe, et sur le `::backdrop` d'une `Dialog` ; `filter` et `backdrop-filter` refusés avec les bons mots ; les bornes ; le contraste mesuré après `brightness: 0.4` (refusé, 2,6 pour 1) et après `grayscale: 1` (gardé) ; le flou refusé sur un paragraphe et sur un nom porté par un texte, accepté sur une image ; le flou de derrière refusé sur un fond opaque ; la transition qui couvre `filter`.
- Dans Chrome : « des filtres d'image : gris, puis les couleurs au survol ; assombri, vif, flou ; la page floue derrière une fenêtre (leçon 131) ». L'image grise porte `grayscale(1)` et passe à `grayscale(0)` sous la souris ; les trois autres portent `brightness(0.6) contrast(1.2)`, `saturate(1.8) hue-rotate(30deg)`, `blur(3px)` ; la fenêtre ouverte, son `::backdrop` porte `blur(6px)` ; la transition couvre `filter`. Avec l'ancien moteur, la leçon est refusée (« réglage inconnu « grayscale » »), et l'essai rate.
- Leçon `131-des-filtres-d-image.holo`.
