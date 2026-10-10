# ADR-111 — Découper une forme : `form:` dans les styles, huit formes nommées

- Statut : ACCEPTÉ (Yocthan, 2026-10-09 : « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-10
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #240 (« Dernière dette du web : découper une forme (clip-path) »), l'une des douze dernières dettes du web, validées d'avance par Yocthan le 2026-10-09 ; le grand tableau du web, où `clip-path` était « en partie » (« `Shape(form:)` : quatre formes ») ; `ADR-032` (`Shape` et ses quatre formes), `ADR-017` (l'apparence dans les styles), `ADR-036` (les états d'un style), `ADR-041` (le CSS utile, où `clip-path` était « plus tard »), `ADR-069` (`aspect-ratio`), `ADR-108` (les filtres, et leur cadre de focus) ; une remarque de la session du PC, en faisant les filtres : « une forme qu'on touche en triangle ou en losange n'a pas de cadre de focus visible : `clip-path` le coupe (vu dans Chrome). Noté pour la 240. »
- Validation : Yocthan, le 2026-10-09, d'avance, avec les douze dernières dettes du web.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Une photo ronde à côté d'un nom, une galerie en hexagones, une bannière dont le bas ondule : des formes de tous les sites. HoloCode ne savait que la `Shape`, en quatre formes (rond, carré, triangle, losange), et `border-radius: 50%` pour arrondir une image.
- Le web le fait par `clip-path: polygon(25% 0, 75% 0, 100% 50%, 75% 100%, 25% 100%, 0 50%)`. Ses défauts :
  - **un tracé écrit à la main**, en coordonnées, qu'on copie d'un générateur sans pouvoir le relire ; `path()` compte en pixels et ne suit pas le bloc quand il change de taille ;
  - **la découpe coupe le cadre de focus** : il est dessiné autour de la boîte du bloc, hors de la forme. Vu dans Chrome : une `Shape(name: Cible, form: triangle)` qui a le focus du clavier ne montre aucun cadre, pas un pixel ne change autour d'elle (WCAG 2.4.7) ;
  - **elle coupe l'ombre et le bord sans un mot** : une ombre (`box-shadow`) disparaît, un bord (`border`) ne reste que sur les côtés droits ;
  - **elle coupe le texte** qui touche les bords : le coin d'un hexagone retire un quart de la largeur en haut et en bas ;
  - **une forme qui change au survol clignote** : le pointeur, au bord, tombe hors de la forme nouvelle, qui rend l'ancienne, et ainsi de suite ;
  - `clip`, l'ancien réglage, existe encore à côté de `clip-path`.

## Décision

```holo
Page(
  title: "Our team",
  children: [
    H1("Our team"),
    Row(gap: 16px, children: [
      Image.round(source: "ada.jpg", alt: "Ada, who paints"),
      Image.hive(source: "lin.jpg", alt: "Lin, who frames"),
    ]),
    Stack(children: [ Image.banner(source: "lake.jpg", alt: ""), H2("The lake, at dawn") ]),
    Shape(name: Star, form: star, color: "#E9B44C", size: 64px),
  ],
)

Image { width: 120px; aspect-ratio: 1; }
.round { form: circle; border: 3px solid white; }
.hive { form: hexagon; }
.banner { width: 100%; aspect-ratio: 3/1; form: wave; }
```

1. **`form:` dans un style** découpe une image ou un dessin en une forme nommée. C'est le mot de `Shape(form:)` (`ADR-032`), avec les mêmes valeurs ; il s'écrit dans un style ou dans ses états d'écran (`phone:`, `computer:`, `narrow:`, `print:`) et le thème sombre (`dark:`).
2. **Huit formes**, chacune un seul mot (la même écriture dans un style et sur un bloc, `ADR-037`) :
   - `circle` : un rond ; un ovale sur un bloc plus large que haut (`aspect-ratio: 1` en fait un rond) ;
   - `square` : rien n'est découpé, le bloc à angles droits (pour défaire une forme donnée par un style plus général) ;
   - `triangle`, `diamond` (un losange), `hexagon` (une pointe à gauche, une à droite), `star` (cinq branches), `heart` ;
   - `wave` : le bas du bloc ondule, sur son dixième, en deux vagues.
   Chaque forme est une découpe fixe du moteur, en pourcentages de la boîte : elle suit la taille du bloc et touche ses quatre bords. Aucun tracé ne s'écrit à la main.
3. **`Shape` prend les huit formes** : `Shape(form: heart, color: "#FF4D6D")`.
4. **Le rond n'est pas une découpe** : il arrondit les coins (`border-radius: 50%`). Son bord, son ombre et son cadre de focus suivent sa courbe. Les six autres formes sont des polygones (`clip-path`), écrits par le moteur.
5. **Le cadre de focus reste visible** (WCAG 2.4.7) :
   - une `Shape` en polygone se dessine dans son bouton (`::before`), qui n'est jamais découpé lui-même : son cadre se voit autour d'elle, sa forme reste visible pendant le focus, et tout son carré se touche, au doigt comme à la souris (une cible plus grande) ;
   - une image ou un dessin découpés, qui ne peuvent rien porter à l'intérieur, se montrent entiers quand ils ont le focus du clavier : le moteur écrit `:focus-visible { clip-path: none }` après les états, comme il retire un filtre au focus (`ADR-108`). Une image n'a le focus que si une règle l'écoute au survol (`ADR-039`) ; un toucher ou un clic ne la montrent pas entière.
6. **Sur une image ou un dessin seulement** : `Image`, `Drawing`, ou un nom de style porté seulement par eux. Refusé sur un bloc qui porte un texte ou un bouton, sur un conteneur, un plateau, un composant, une vidéo (ses commandes et ses sous-titres). La forme d'une `Shape` s'écrit sur elle, jamais dans un style.
7. **Les coins, l'ombre, le bord** :
   - `border-radius` avec `form` dans le même style (ou le même état) est refusé : la forme donne déjà les coins, un rond ou des angles droits ; l'un des deux serait effacé ;
   - avec un polygone, `box-shadow` et `border` sont refusés : la découpe couperait l'ombre (il n'en resterait rien) et le bord (des morceaux). Aussi quand un autre style les donne au même bloc : le moteur suit la cascade, et dit lequel retirer, ou d'écrire `box-shadow: none;` et `border: none;` (nouveau : comme en CSS) dans le style qui découpe ;
   - une `Shape` en polygone n'a ni fond, ni ombre, ni bord donnés par un style : ils rempliraient ou entoureraient son carré. Sa couleur s'écrit `Shape(color:)`.
8. **Ni sous la souris, ni sous le doigt, ni au focus** : `form` dans `hover:` et `active:` est refusé (le clignotement) ; dans `focus:`, aussi (le moteur montre déjà le bloc entier).
9. **Pas de tracé** : `clip-path` et `clip` sont refusés, avec le bon mot et la liste des formes ; `form: polygon(…)`, ou un mot inconnu, aussi.
10. **Sans JavaScript** : ce n'est que du CSS, dans la page que le serveur fabrique (`holo serve`) ; le cadre de focus aussi. Seules les formes de `Shape` dont la page se sert sont écrites dans son style.
11. **Parité** : une forme est une apparence, la même sur tous les appareils. Un lecteur d'écran n'y perd rien : la `Shape` reste un bouton nommé, l'image garde son texte (`alt`).

Ce qui change pour les pages d'avant : une `Shape` en triangle ou en losange se touche sur tout son carré, et non plus seulement dans sa forme ; à l'œil, rien ne change.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| La forme du mot | A, le CSS tel quel : `clip-path: polygon(…)`, `circle(40%)`, `path("M…")` ; B, le nom du web avec des mots : `clip-path: hexagon;` ; C, **le mot qui existe : `form: hexagon;`** ; D, `shape: hexagon;` ; E, `cut: hexagon;` ; F, un paramètre, `Image(form: hexagon)` ; G, un bloc, `Cut(form:, children:)` | C. Le mot existe déjà pour la même idée : `Shape(form: hexagon)` (`ADR-032`), et `Item(form:)` (`ADR-088`) ; un seul mot pour une seule idée, comme `hue` repris d'`Enter` par `ADR-108`. Il se lit comme une phrase, et c'est une apparence : la place du bloc ne change pas (`ADR-017`). A est le tracé à la main que l'issue refuse ; B promet un tracé (« path ») et ressemble au CSS sans en être ; D dirait la même chose que le bloc `Shape` avec un autre mot, et `shape-outside` veut dire autre chose en CSS (l'habillage d'un flottant) ; E est d'abord couper-coller, et ne dit pas ce qui reste ; F n'aurait ni thème (`Image { … }`), ni écran (`phone:`), ni nom partagé ; G ajoute un bloc pour une apparence, et laisse couper du texte |
| Les formes | les quatre de `Shape` ; **huit** ; une vingtaine (les modèles de Clippy : pentagone, octogone, flèches, chevrons, croix, bulle…) ; des formes réglables (`star(points: 6)`, une vague plus haute) | Huit. L'issue demande le rond, l'hexagone et la vague ; l'étoile et le cœur sont ceux des jeux, des badges et des « j'aime ». Chacune a un seul mot anglais, écrit pareil en CSS et en Flutter (`ADR-037`), donc pas de `wave-top`. Les autres restent en dette ; des formes réglables rouvriraient le tracé libre |
| Le rond | `clip-path: circle()` (un vrai rond, des vides sur les côtés) ; `clip-path: ellipse()` ; **`border-radius: 50%`** | Le dernier : son bord, son ombre et son cadre de focus suivent la courbe (vu dans Chrome), et il remplit la place du bloc. `aspect-ratio: 1` (`ADR-069`) en fait un rond parfait ; c'est déjà le rond de `Shape(form: circle)` |
| Les blocs découpés | tout bloc, à l'auteur de voir ; les conteneurs, avec une marge intérieure ajoutée par le moteur ; **une image, un dessin** (et `Shape`, qui porte sa forme) | Un polygone coupe ce qui touche ses bords : des lettres disparaîtraient, une vidéo perdrait ses commandes, un plateau ses pièces (qu'on ne toucherait plus dans les coins coupés). Une marge exacte ne se calcule pas en CSS (un pourcentage vertical compte sur la largeur). Les demandes tiennent dans le choix : la photo ronde, la galerie, la bannière en vague, avec un titre posé dessus par `Stack` (la leçon le montre). Même choix que les filtres (`ADR-108`) |
| Le cadre de focus | ne rien faire (le web) ; un cadre dessiné autrement, `outline` vers l'intérieur ou une ombre intérieure ; une ombre qui suit la forme (`filter: drop-shadow()`) ; **la `Shape` dessinée dans son bouton ; l'image ou le dessin entiers au focus du clavier** | Un cadre vers l'intérieur est coupé aussi, sauf là où la forme touche la boîte ; un filtre passe avant la découpe et part avec elle, ou demande un bloc de plus autour de chaque forme (la disposition changerait, et les filtres d'`ADR-108` s'y mêleraient). Dessinée dans son bouton, la `Shape` garde sa forme pendant le focus : on sait que c'est l'étoile qui l'a ; et son carré entier se touche (WCAG 2.5.8). Une image ne peut rien porter dedans : entière au focus, avec son cadre, comme un filtre retiré (`ADR-108`) |
| Une forme qui change | dans tous les états ; **pas sous la souris ni le doigt, ni au focus** | `hover:` et `active:` feraient clignoter le bloc au bord de la forme ; au focus, le moteur montre déjà le bloc entier. Les écrans, le thème sombre et le papier changent la forme sans pointeur : permis |
| L'ombre et le bord d'un polygone | laisser faire (le CSS) ; les dessiner en suivant la forme ; **les refuser, avec la raison** | Les dessiner demande un bloc autour (en dette). Les refuser dit au moins pourquoi l'ombre n'est pas là ; le rond les garde. Le moteur suit la cascade, pour dire quel style retirer |

## Ce qui est refusé, et pourquoi

- `clip-path`, `clip`, `form: polygon(…)` : un tracé écrit à la main ; le message donne `form:` et les huit formes.
- `form` sur un texte, un conteneur, un plateau, un composant, une vidéo : la découpe couperait des lettres, des boutons, des pièces, des commandes.
- `form` dans un style qui vise une `Shape` : sa forme s'écrit sur elle.
- `form` dans `hover:`, `active:`, `focus:` : le clignotement ; le bloc entier au focus.
- `border-radius` avec `form` dans le même style ou le même état : l'un des deux serait effacé.
- `box-shadow` et `border` avec un polygone, dans le même style ou donnés par un autre style au même bloc : coupés ; `none` les retire.
- Un fond, une ombre ou un bord sur une `Shape` en polygone : ils rempliraient ou entoureraient son carré.

## Les défauts du web évités

- **Le tracé à la main** : huit formes nommées, en pourcentages, qui suivent le bloc.
- **Le cadre de focus coupé** : la `Shape` n'est jamais découpée elle-même ; l'image et le dessin se montrent entiers au focus du clavier.
- **L'ombre et le bord coupés sans un mot** : refusés, avec la raison et le remède ; le rond les garde.
- **Le texte coupé** : un texte n'est jamais découpé.
- **Le clignotement au survol** : une forme ne change pas sous la souris.
- **Deux réglages pour la même chose** (`clip`, `clip-path`), et **deux façons de faire un rond** (`border-radius: 50%` ou `clip-path: circle()`) : `form:` seul.

## Dettes

- D'autres formes (pentagone, octogone, flèche, chevron, bulle, croix) ; une forme qu'on oriente (le triangle pointe vers le haut : `rotate: 180deg` le retourne en attendant) ; la vague en haut d'un bloc.
- Une ombre et un bord qui suivent un polygone : il faudrait un bloc autour, et `drop-shadow`, laissé en dette par `ADR-108`.
- Découper un conteneur sans couper son texte : il faudrait que le texte suive la forme (`shape-inside`), ce que les navigateurs n'ont pas.
- La vérification croisée (une ombre ou un bord venus d'un autre style) ne regarde que les réglages hors des états : une ombre qu'un autre style donne au survol (`Image { hover: { box-shadow: … } }`), ou une forme écrite seulement dans un état d'écran (`phone: { form: hexagon; }`) face à l'ombre d'un autre style, ne sont pas vues. Dans un même style, les états comptent.
- Une image qui ne se charge pas montre son texte de remplacement à sa place, découpé avec elle ; un lecteur d'écran, lui, le lit toujours.
- Passer d'une forme à l'autre en douceur (`Loop(form:)`, proposé dans la piste 9 de l'exploration du web complet).
- Le cœur et la vague sont des polygones de 40 et 35 points : à 400 px, on devine leurs facettes ; `shape()` du CSS tracera des courbes quand tous les navigateurs l'auront.
- La ligne `clip-path` du grand tableau du web peut passer à « oui » à la prochaine publication.

## Critères de validation

- Tests du moteur : `forms.rs` (`every_form_is_a_named_bounded_cut` : six polygones dans la boîte, qui touchent ses bords, le cœur symétrique, la vague dans le dixième du bas ; `a_form_cuts_an_image_or_a_drawing_and_nothing_is_cut_silently` : les acceptés et chaque refus, avec sa raison, la cascade comprise ; `only_the_shapes_of_the_page_are_drawn_inside_their_button`), `flat.rs` (`a_shape_is_drawn_inside_its_button_and_a_style_cuts_an_image` : le bouton jamais découpé, la découpe et les coins dans un style, `:focus-visible { clip-path: none }` après les états, rien pour le rond ; `a_shape_is_a_drawing_or_a_button`, où la forme inconnue devient `pentagon`).
- Dans Chrome : « découper une forme : … (leçon 134) ». Les découpes reçues (rond, 6, 10, 40 et 35 sommets, et les huit formes de `Shape` dessinées dans leur bouton) ; le lecteur d'écran (un bouton « Etoile », les images nommées) ; le cadre de focus de l'étoile et de l'image en hexagone, vu sur des captures : les pixels qui changent dans la bande de 8 px autour du bloc quand il reçoit le focus par Tab ; l'étoile touchée au clavier (Entrée, Espace), à la souris (le milieu, puis un coin du carré, hors de la branche) et au doigt ; le survol venu du clavier ; axe-core ; un téléphone de 360 px ; sans JavaScript, par `holo serve`, les mêmes découpes et l'image entière au focus. L'essai sait échouer : la `Shape` découpée elle-même (comme avant) → « le cadre de focus de l'étoile : …, 0 pixels du cadre autour » ; sans la règle du focus → « le cadre de focus de l'image découpée : …, 0 pixels du cadre autour ».
- Leçon `134-decouper-une-forme.holo`.
