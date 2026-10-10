# ADR-108 — Les filtres d'image dans les styles : `grayscale`, `saturate`, `brightness`, `contrast`, `hue`, `blur`, et `backdrop-blur`

- Statut : ACCEPTÉ (fait et validé : Yocthan, 2026-10-09, « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-10
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #237 (« Dernière dette du web : les filtres d'image dans les styles (flou, gris, luminosité) »), dans la file des dernières dettes du web ouverte par Yocthan le 2026-10-09 ; le grand tableau du web, où `filter` était « en partie » (« `blur`, `hue` dans `Enter` et `Loop` », `docs/01-holocode/TABLEAU-WEB.md`) ; les styles qui ne disent que l'apparence, tout vérifié (`ADR-017`) ; le lot 4 du CSS utile, où `rotate` et `scale` sont déjà des réglages à part (`ADR-041`) ; le mouvement, où `blur` et `hue` existent déjà (`ADR-034`) ; le contraste vérifié par le moteur (`ADR-055`) ; l'écriture des noms (`ADR-016`, `ADR-037`) ; la règle de parité de Yocthan (2026-10-07).
- Validation : faite par Yocthan le 2026-10-09 (« tu le valides déjà, tu le fais déjà »).
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Une image en gris qui reprend ses couleurs au survol, une photo assombrie sous un titre, la page qui devient floue derrière une fenêtre : des effets de tous les sites, que HoloCode ne savait écrire que pour un bloc qui bouge (`Enter(blur: 10px, hue: 90deg)`), jamais dans un style.
- Le web le fait par `filter: grayscale(100%) blur(4px)` et `backdrop-filter: blur(8px)`. Ses défauts :
  - `filter` est **une liste** : un état qui veut en changer un seul doit la réécrire entière ; `.photo:hover { filter: grayscale(0); }` fait disparaître le flou écrit à côté, sans un mot ;
  - **deux écritures** pour la même chose, `grayscale(1)` et `grayscale(100%)`, `brightness(0.5)` et `brightness(50%)` ;
  - **un filtre touche tout ce que le bloc contient** : posé sur une carte, il assombrit ou floute aussi ses textes et ses boutons, et rien ne vérifie qu'ils se lisent encore ; le contraste mesuré par le moteur (`ADR-055`) deviendrait faux, puisqu'il mesure les couleurs écrites, pas les couleurs filtrées ;
  - **une vidéo filtrée l'est avec ses commandes et ses sous-titres** (vu dans Chrome : `blur(3px)` les brouille, `brightness(0.3)` les éteint presque) ;
  - **le cadre de focus est filtré avec le bloc** (vu dans Chrome : un flou le brouille, `brightness(0.2)` l'efface presque) : au clavier, on ne voit plus où l'on est ;
  - `backdrop-filter` ne se voit qu'à travers un fond transparent, et Safari a longtemps exigé `-webkit-backdrop-filter`.
- HoloCode a déjà tranché la même question pour la pose : `rotate` et `scale` sont deux réglages, jamais la liste `transform` (`ADR-041`).

## Décision

```holo
Page(
  title: "The lake",
  children: [
    Image.gray(source: "lake.jpg", alt: "The lake at dawn"),
    Stack(children: [ Image.dimmed(source: "lake.jpg", alt: ""), H2("The lake, at dawn") ]),
    Button(name: Open, text: "Open"),
    Dialog(name: Window, children: [ P("Behind, the page is blurred."), Button(name: Close, text: "Close") ]),
  ],
  rules: [ On(Open.tap, effect: Window.open), On(Close.tap, effect: Window.close) ],
)

.gray { grayscale: 1; transition: 0.4s; hover: { grayscale: 0; } active: { grayscale: 0; } }
.dimmed { brightness: 0.5; }
Dialog { backdrop-blur: 6px; }
```

1. **Six réglages de style**, écrits comme en CSS, dans un style ou dans l'un de ses états (`hover:`, `active:`, `dark:`, `phone:`, `computer:`, `narrow:`, `print:`) :
   - `grayscale` : de 0 (les couleurs) à 1 (tout gris), un nombre comme `opacity` ;
   - `saturate` : de 0 (gris) à 3 (des couleurs plus vives) ; 1 ne change rien ;
   - `brightness` : de 0.2 à 3 ; 1 ne change rien, moins assombrit, plus éclaircit ;
   - `contrast` : de 0.2 à 3 ; 1 ne change rien ;
   - `hue` : un angle, de -360deg à 360deg ; les couleurs tournent sur le cercle des teintes ; le mot est celui d'`Enter` et de `Loop` ;
   - `blur` : un flou, de 0 à 100px.
2. **Sur une image, une forme ou un dessin** : `Image`, `Shape`, `Drawing`, ou un nom de style porté seulement par eux. Ces blocs ne portent pas de texte. Un filtre est refusé, avec la raison, sur tout autre bloc : un texte, un bloc qui contient des textes ou des boutons, un composant (le texte se lirait mal, et le contraste vérifié deviendrait faux), une vidéo (ses commandes et ses sous-titres). Les couleurs d'un texte se changent par `color` et `background`, que le moteur mesure.
3. **Chaque réglage est à part.** Le moteur les compose en un seul `filter`, toujours dans le même ordre (gris, saturation, luminosité, contraste, teinte, flou). Dans un état, un réglage change sans effacer les autres : `hover: { grayscale: 0; }` rend les couleurs et garde le flou écrit à côté, parce que le moteur réécrit la liste entière à partir du style et de l'état.
4. **Au clavier, un bloc filtré qui a le focus se montre sans filtre** (`:focus-visible { filter: none }`, écrit par le moteur après les états) : une forme qu'on touche (`Shape(name:)`), un bloc qu'une règle écoute au survol. Son cadre de focus reste net. Un filtre écrit dans `focus:` est donc refusé, avec la raison.
5. **`backdrop-blur`** : de 0 à 100px, sur une fenêtre (`Dialog`, ou un nom porté seulement par des fenêtres) : la page, derrière elle, devient floue, en plus de s'assombrir comme aujourd'hui. Le moteur l'écrit sur le `::backdrop` de la fenêtre, avec `-webkit-backdrop-filter`. Il s'écrit dans le style de la fenêtre lui-même : refusé dans un état, et sur tout autre bloc.
6. **Les passages d'une allure à l'autre** (`transition`, ou le survol et l'appui sans `transition` écrit) adoucissent aussi les filtres ; un visiteur qui demande moins de mouvement (`prefers-reduced-motion`) ne voit aucun passage, comme pour le reste.
7. **Refusés, avec le bon mot** : `filter` et `backdrop-filter` écrits comme en CSS (le message donne les réglages) ; une valeur hors des bornes, ou en `%` ; un filtre ailleurs que sur une image, une forme ou un dessin ; un filtre au focus ; `backdrop-blur` ailleurs que sur une fenêtre, ou dans un état.
8. **Sans JavaScript** : ce n'est que du CSS, écrit dans la page que le serveur fabrique ; les images y sont filtrées de la même façon. Une fenêtre, elle, ne s'ouvre qu'avec JavaScript, comme aujourd'hui (`ADR-042`) : sans lui, il n'y a pas de page derrière elle à flouter.
9. **Parité** : un filtre est une apparence, la même sur tous les appareils. Le survol n'existe qu'avec une souris, comme tous les `hover:` (`ADR-036`) ; au doigt, l'état `active:` fait la même chose pendant l'appui, et la leçon l'écrit ; au clavier, le focus montre le bloc sans filtre.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| La forme | A, le CSS tel quel : `filter: grayscale(1) blur(4px);`, une liste de fonctions ; B, **un réglage par effet** : `grayscale: 1; blur: 4px;` ; C, un paramètre d'`Image(filter: gray)` | B : c'est le choix déjà fait pour la pose (`rotate`, `scale` plutôt que `transform`, `ADR-041`) ; dans un état, un réglage change sans effacer les autres ; un débutant écrit `grayscale: 1` sans parenthèses ni liste. A répète le défaut du CSS ; C ne couvrirait ni le survol, ni le thème, ni une forme |
| Les noms | les mots du CSS (`grayscale`, `saturate`, `brightness`, `contrast`, `blur`) ; des mots plus courts (`gray`, `dim`) ; `hue-rotate` ou `hue` | les mots du CSS, qu'un programmeur connaît et qui gardent leur sens (`ADR-016`) ; `hue` plutôt que `hue-rotate`, parce que c'est déjà le mot d'`Enter` et de `Loop` (`ADR-034`) : un seul mot pour une seule idée |
| Les valeurs | `100%` et `1`, comme le CSS ; **un nombre seulement** | une seule écriture par réglage (`ADR-037`) ; le nombre est celui d'`opacity`, et 1 veut dire « rien ne change » là où c'est le cas ; des pixels pour le flou, des degrés pour la teinte ; toutes bornées |
| Les blocs filtrés | tout bloc, à l'auteur de voir ; tout bloc, et le contraste mesuré après le filtre quand le même style donne les deux couleurs ; **les images, les formes, les dessins** | Un filtre sur un bloc touche aussi ses textes, ses boutons, et ceux de ses enfants, dont les couleurs viennent d'autres styles ou de la page : la mesure ne verrait qu'une partie des cas, et un texte assombri passerait. Les trois blocs choisis ne portent jamais de texte : un texte n'est jamais filtré, et le contraste vérifié reste juste. Les demandes de la file (une image en gris, une photo assombrie, la page floue derrière une fenêtre) tiennent toutes dedans |
| La vidéo | permise, comme une image ; **refusée** | ses commandes et ses sous-titres font partie d'elle, et seraient filtrés avec elle |
| Le cadre de focus | ne rien faire ; refuser un filtre sur un bloc qu'on touche ; **montrer le bloc sans filtre au focus du clavier** | un filtre sur une forme qu'on touche reste possible (une pièce grise dans un jeu), et le clavier voit toujours un cadre net ; seul le focus du clavier (`:focus-visible`) le fait, pas un toucher ni un clic |
| Le flou derrière | `Dialog { blur: 8px; }` qui voudrait dire « derrière » ; `backdrop-filter: blur(8px)` ; **`backdrop-blur: 8px`**, sur une fenêtre ; le même sur tout bloc à demi transparent (le verre dépoli) | « derrière » n'est pas « le bloc » : le même mot pour les deux tromperait ; `backdrop` est le mot du web (le `::backdrop` d'une fenêtre, `backdrop-filter`, les classes `backdrop-blur` de Tailwind) ; la file demandait « un fond flou derrière une fenêtre ». Le verre dépoli sur un bloc posé sur une image laisserait un texte sur un fond qu'on ne connaît pas : il attend une mesure du contraste au pire (sur ce qui peut passer derrière) |
| L'ordre des filtres | celui de l'écriture ; **toujours le même** | deux auteurs qui écrivent les mêmes réglages voient la même chose ; l'ordre ne change presque rien à l'œil, mais une règle vaut mieux qu'un hasard |
| D'autres filtres | `sepia`, `invert`, `drop-shadow` ; **les six de la demande, et le flou derrière une fenêtre** | `sepia` est un gris teinté (`grayscale` et `hue`) ; `invert` retourne toutes les couleurs ; `drop-shadow` attend la forme découpée (`ADR-111`) pour servir. Laissés en dette |

## Les défauts du web évités

- **La liste `filter` qu'un état réécrit entière** : ici, un réglage à la fois, et le moteur recompose la liste.
- **Deux écritures** (`1` et `100%`) : une seule.
- **Un texte assombri ou flou**, sans que rien ne le voie : un filtre ne se pose jamais sur un bloc qui porte un texte.
- **Les commandes et les sous-titres d'une vidéo filtrés** : pas de filtre sur une vidéo.
- **Un cadre de focus brouillé ou effacé** : au clavier, le bloc qui a le focus se montre sans filtre.
- **`backdrop-filter` qui ne fait rien** sur un fond opaque, et le préfixe oublié : ici, le flou va sur le `::backdrop` de la fenêtre, avec `-webkit-backdrop-filter`.
- **Le passage brusque** d'un filtre à l'autre au survol : adouci d'office, comme la couleur et l'ombre ; et arrêté pour qui demande moins de mouvement.

## Dettes

- `sepia`, `invert`, et une ombre qui suit la forme (`drop-shadow`).
- Le verre dépoli (`backdrop-blur` sur un bloc à demi transparent posé sur une image) : il faudrait mesurer le contraste de son texte au pire, sur du blanc et sur du noir derrière lui.
- Dans `Enter` et `Loop`, seuls `blur` et `hue` bougent ; un gris ou une luminosité qui s'animent restent à faire.
- Un bloc qui bouge (`Enter(blur:)`) et porte aussi un filtre de style : pendant le mouvement, c'est le mouvement qui tient le `filter` ; au repos, le style.
- Une image qui ne se charge pas montre son texte de remplacement à sa place, filtré avec elle ; un lecteur d'écran, lui, le lit toujours.
- La ligne `filter` du grand tableau du web peut passer à « oui » à la prochaine publication.

## Critères de validation

- Tests du moteur (`moteur/src/filters.rs`, `styles.rs`, `flat.rs`) : les six réglages et `backdrop-blur` acceptés ; la composition en un seul `filter`, dans l'ordre, un état qui en change un et garde les autres ; `:focus-visible { filter: none }` après les états ; `backdrop-filter`, avec son préfixe, sur le `::backdrop` de la fenêtre seulement ; `filter` et `backdrop-filter` refusés avec les bons mots ; les bornes ; un filtre refusé sur un paragraphe, une carte, un bouton qui porte le même nom qu'une image, une vidéo, et dans `focus:` ; `backdrop-blur` refusé hors d'une fenêtre et dans un état ; la transition qui couvre `filter`.
- Dans Chrome : « des filtres d'image : gris, puis les couleurs sous la souris ; assombri, vif, flou ; sans filtre au focus du clavier ; la page floue derrière une fenêtre ; sans JavaScript aussi (leçon 131) ». L'image grise porte `grayscale(1)` et passe à `grayscale(0)` sous la souris ; les trois autres portent `brightness(0.6) contrast(1.2)`, `saturate(1.8) hue-rotate(30deg)`, `blur(3px)` ; rendue atteignable au clavier, l'image grise qui reçoit le focus par Tab n'a plus de filtre ; la fenêtre ouverte, son `::backdrop` porte `blur(6px)` ; la transition couvre `filter` ; sans JavaScript, la page fabriquée par le serveur porte les mêmes filtres. Avec l'ancien moteur, la leçon est refusée (« réglage inconnu « grayscale » »), et l'essai rate.
- Leçon `131-des-filtres-d-image.holo`.
