# Piste 4 — S'adapter à la place disponible

> Statut : EXPLORATION. Avis de Claude, pas une décision.

## Ce que Codex demandait

Sa ligne, dans le texte de l'issue #82 (copie locale non versionnée : `moteur/target/pistes-langage-pour-claude-2026-10-06.md:53`) :

> | **4** | **Adaptation à l'espace disponible** : proportions des colonnes, éléments qui occupent la place restante, variantes selon la largeur du conteneur. | Composer des pages différentes sur téléphone et ordinateur, avec un contrôle plus fin que le repli automatique actuel. |

Et plus bas (même fichier, ligne 63) : « Pour l'adaptation fine, les requêtes de conteneur CSS constituent un bon modèle à comparer : un composant s'adapte à la place qu'il reçoit. »

Une **requête de conteneur** (*container query*) est une règle CSS qui dépend de la largeur d'un bloc parent, et non de la largeur de l'écran. Exemple : une carte posée dans une colonne étroite se met en vertical ; la même carte posée en grand se met en horizontal.

## État vérifié (main, 7a48def, 2026-10-07)

**Franchement : à moitié faite.** Le repli automatique marche bien, et la place qui reste existe (`grow`). Mais il manque l'essentiel du but de Codex : **une page d'ordinateur différente d'une page de téléphone.** Aujourd'hui, une page fait 640 px de large au plus, et l'auteur ne peut pas l'élargir. Sur un écran de 1280 px, on voit la page du téléphone, centrée.

Pendant l'exploration, `main` est passé de 7a48def à 1119361 (PR 139, vérification d'un fichier de thème seul). Les lignes citées de `flat.rs` n'ont pas bougé ; j'ai relancé les essais : mêmes réponses.

**Ce qui existe**

| Besoin | Ce que fait le moteur | Où |
|---|---|---|
| Passer à la ligne | `Row` : ce qui ne tient pas passe à la ligne, jamais de débordement sur le côté | `moteur/src/flat.rs:29` ; `ADR-024` § 1 ; leçon 6 |
| Grille qui perd des colonnes | `Grid(columns: 3)` : la grille calcule ses colonnes d'après **sa propre largeur** (au moins 7,5 rem par colonne). C'est déjà une adaptation « au conteneur » | `flat.rs:31-32` ; `ADR-024` § 3 |
| La place qui reste | `grow: 1` à `12` dans `Row` ou `Column`, comme `Expanded` de Flutter | `flat.rs:536-549` ; `ADR-052` § 1 ; leçon 72 |
| Une largeur | les styles `width`, `max-width`, en `px` (écrits en `rem`) ou en `%` | `styles.rs:52-54` ; `ADR-052` § 2 |
| Tout l'écran en hauteur | `height: screen` | `flat.rs:424-428` ; `ADR-061` § 4 ; leçon 80 |
| Le téléphone | l'état `phone: { … }` = `@media (max-width: 640px)` ; `display: none` permis seulement là ; `Image(phone: "petite.jpg")` | `flat.rs:391`, `styles.rs:286-292` ; `ADR-041` § 6, `ADR-042` ; leçons 53 et 57 |
| Superposer | `Stack` et neuf places nommées | `flat.rs:19`, `flat.rs:476-492` ; `ADR-036` § 5 ; leçon 38 |
| Les titres | un titre de plus de 24 px rétrécit sur un écran étroit (`clamp`) | `flat.rs:437-440` ; `ADR-036` § 3 |

Le moteur emploie **déjà** les requêtes de conteneur, pour lui-même : le plateau d'un jeu (`Board`) est un conteneur (`container-type: inline-size`, `flat.rs:50`) et ses formes se mesurent en `cqw` (`flat.rs:67-68`). Les navigateurs visés savent donc le faire.

**Ce qui manque** (mesuré)

1. **La largeur de la page est bloquée à 640 px.** La règle de base `:where(.holo-Page>main, …>header, …>footer){max-width:640px}` (`flat.rs:16`) limite le contenu. L'auteur ne peut pas la changer :
   - `Main { max-width: 1100px; }` est accepté et fabrique `.holo-Main{max-width:68.75rem;}`, mais l'élément `<main>` n'a pas de classe (`flat.rs:246`). Le style ne vise rien. Il est ignoré sans rien dire. Un nom de style posé sur `Main` (`Main.large(…)`) est perdu de même : le moteur ne fabrique que les enfants de `Main` (`flat.rs:121-124`) ; vérifié à la relecture, `.holo-s-large{…}` est écrit, mais aucun bloc ne porte la classe.
   - `Page { max-width: 1100px; }` est accepté, mais vise le bloc extérieur ; `<main>` reste à 640 px. Ce réglage n'est pas sans effet : mesuré à la relecture à 1280 px, le bloc de la page fait 1 100 px, collé à gauche (de 0 à 1 100), et `main` se décale de 90 px vers la gauche (posé à 230 au lieu de 320). L'effet existe, et il est mauvais.
   - Mesuré à 1280 px : `main` fait 640 px.
2. **Pas de bandeau d'un bord à l'autre.** Un en-tête ou un pied de page coloré s'arrête à la colonne. Mesuré à 1280 px : l'en-tête fait 672 px (640 + ses marges intérieures, posé à 304 px) et le contenu 640 px (posé à 320 px). Les bords colorés ne sont même pas alignés ; le texte de l'en-tête, lui, commence bien à 320 px. La cause : la règle de base ne met pas `box-sizing: border-box` sur `header` et `footer` (`flat.rs:16` ; elle ne le met que sur leurs enfants, `flat.rs:17`), donc leurs marges intérieures s'ajoutent aux 640 px. C'est un petit défaut à part, facile à corriger.
3. **Deux blocs avec `grow` ne passent jamais l'un sous l'autre.** `grow: N` devient `flex: N 1 0` (`flat.rs:546`) : une base nulle ne passe jamais à la ligne. Mesuré à 360 px : l'image garde 115 px de large et le texte 229 px, côte à côte. (Des largeurs fixes, elles, passent à la ligne : voir le scénario plus bas.)
4. **Aucune variante selon la place reçue.** Le seul seuil est celui de l'écran, à 640 px (`flat.rs:391`, `flat.rs:400-401`). Une carte posée dans une colonne étroite d'un grand écran se croit sur un grand écran.
5. **La grille n'a que des colonnes égales.** La largeur minimale d'une colonne est fixée à 7,5 rem (`flat.rs:32`) ; pas de bloc sur deux colonnes (`TABLEAU-WEB.md:583`).
6. **Pas d'alignement en hauteur.** Dans une `Row`, les blocs sont toujours centrés en hauteur (`align-items:center`, `flat.rs:29`). `ADR-024`, ligne 67, le note dans « Ce qui reste à faire ».

**Documents en retard**

- `docs/02-gouvernance/adr/ADR-024-disposition.md:61` et `:67` : « pas de largeur par élément, pas d'élément qui prend « tout le reste » » ; `width` et `grow` existent depuis (`ADR-052`).
- `docs/01-holocode/GUIDE.md` ne dit nulle part que la page fait 640 px au plus : il dit seulement « un écran plus étroit que la page » (`GUIDE.md:312`). Le nombre n'apparaît que dans un commentaire du code (`flat.rs:400`), et, à travers le seuil de `phone:`, dans la leçon 53 (ligne 4 : « un écran plus étroit que la page (640 pixels) »), dans `ADR-041` § 6 et dans `NOMS.md:71`.
- `docs/01-holocode/NOMS.md:216` range `@media` dans « Refusés exprès » (« Le moteur s'en charge »). C'est vrai pour l'auteur, qui n'écrit jamais `@media` ; mais `NOMS.md:71` donne `@media` comme l'équivalent, changé, de `phone:` et `dark:`. Les deux lignes se contredisent à moitié ; et `align-items`, rangé à la même ligne, reviendrait avec `valign`.
- `docs/01-holocode/TABLEAU-WEB.md` n'a ni ligne pour la largeur de la page, ni ligne pour les requêtes de conteneur ; ligne 584 : « Un seul seuil ».
- `docs/01-holocode/COMPARAISON-WEB.md:187-189` : « pas de mise en page : tout est l'un sous l'autre » ; c'est faux depuis `ADR-024`.

**Mesures** (PC Windows, Chrome sans fenêtre, largeur simulée ; ce ne sont pas des mesures de téléphone : il n'y a pas d'appareil ici)

```bash
H=moteur/target/release/holo.exe          # essais dans essais-3-4-5/p4/
"$H" html p4-01-main-large.holo | grep -o '\.holo-Main{[^}]*}\|<main>'      # Main { max-width: 1100px; }
.holo-Main{max-width:68.75rem;}
<main>
"$H" check - < p4-30-width-screen.holo        # .b { width: screen; }
ligne 2, colonne 6 : « width: screen » : ce réglage attend une taille, comme « 16px » ou « 50% »
"$H" check - < p4-31-valign.holo              # Row(valign: top, …)
ligne 1, colonne 22 : « Row » n'a pas de paramètre « valign » ; paramètres possibles : name, children, gap, align
"$H" check - < p4-33-span.holo                # P("a", span: 2) dans une Grid
ligne 1, colonne 54 : « P » n'a pas de paramètre « span » ; paramètres possibles : name
```

Les largeurs, mesurées par `moteur/outils/capture.mjs` (Chrome sans fenêtre ; une mesure faite dans la page par le protocole de Chrome) :

```text
# p4-10 : une image (grow: 1) et un texte (grow: 2) dans une Row, puis une Grid(columns: 3) de six cartes
node moteur/outils/capture.mjs file:///…/p4-10-fiche.html p4-10-fiche-360.png 360 800 1500
→ {"viewport":360,"main":360,"grow_left_top_width":[[0,53,115],[131,73,229]],"gridColumns":2}
node moteur/outils/capture.mjs file:///…/p4-10-fiche.html p4-10-fiche-1280.png 1280 800 1500
→ {"viewport":1280,"main":640,"grow_left_top_width":[[320,53,208],[544,82,416]],"gridColumns":3}
# p4-11 : un en-tête, un bandeau et un pied colorés
node moteur/outils/capture.mjs file:///…/p4-11-bandeaux.html p4-11-bandeaux-1280.png 1280 720 1500
→ {"viewport":1280,"header":[304,672],"hero":[320,640],"main":[320,640],"footer":[304,672],"gridColumns":4}
```

La mesure passe par la variable `HOLO_GESTURES` de `capture.mjs` (`Runtime.evaluate`) ; l'expression n'est pas recopiée ici. Relu le 2026-10-07 : `p4-10` refait par `holo html` et remesuré (dossier `relecture-3-4-5/`), mêmes nombres : à 360 px, `{"main":360,"grow":[[0,53,115],[131,73,229]]}` ; à 1280 px, `{"main":640,"grow":[[320,53,208],[544,82,416]]}`. Et `Page { max-width: 1100px; }` seul, à 1280 px : `{"page":[0,1100],"main":[230,640]}`.

**Un prototype isolé** (fichier `essais-3-4-5/p4/p4-20-prototype.html` : le HTML du moteur, retouché à la main — trois classes et deux enveloppes `proto-cq` ajoutées —, plus quatre règles CSS écrites à la main, hors du dépôt). Il essaie trois choses : la page élargie à 1100 px ; une ligne qui passe en colonne sous 30 rem **de sa propre largeur**, par une astuce CSS sans requête de conteneur (`flex-basis: calc((30rem - 100%) * 999)`) ; une carte qui se met à l'horizontale au-dessus de 26 rem **de sa propre largeur**, par une requête de conteneur.

```text
node moteur/outils/capture.mjs file:///…/p4-20-prototype.html p4-20-prototype-360.png 360 900 1500
→ {"viewport":360,"main":360,"ligne":[[0,53,360],[0,229,360]],"cartes":[[172,"column"],[172,"column"]]}
→ (768)  {"viewport":768,"main":768,"ligne":[[0,53,251],[267,91,501]],"cartes":[[376,"column"],[376,"column"]]}
→ (1280) {"viewport":1280,"main":1100,"ligne":[[90,53,361],[467,91,723]],"cartes":[[542,"row"],[542,"row"]]}
```

Lecture : à 360 px, la ligne passe en colonne (même gauche, hauts différents) ; à 768 et 1280 px, elle reste côte à côte, en parts 1 et 2 (251/501, 361/723) ; à 1280 px, la page fait bien 1100 px, et les cartes, assez larges (542 px), passent à l'horizontale. Les captures sont à côté des fichiers.

## Le scénario du site de référence

La recette demande la page à 320, 360, 768 et 1280 px, puis le texte agrandi à 200 % (`proposals/GPT5.6/site-reference-2026-10-06/RECETTE.md`, cas F02). Le premier passage n'a joué que 360 px (`exemples/site-reference/RECETTE-2026-10-06.md:15`).

La tâche : **la fiche d'une création (`exemples/site-reference/fiche.holo`) sur ordinateur et sur téléphone.**

- Sur ordinateur : un bandeau de titre d'un bord à l'autre ; l'image à gauche et le texte à droite, en parts 1 et 2, alignés en haut ; en dessous, « Dans la même série » en quatre colonnes.
- Sur téléphone : l'image au-dessus du texte ; deux colonnes, puis une seule.
- La même carte d'œuvre, posée dans une colonne étroite à côté d'un filtre, se met en vertical ; posée seule en large, en horizontal.

Aujourd'hui, la fiche est une seule colonne de 640 px (`fiche.holo:9-35`). Un détour existe déjà pour l'image à côté du texte : des largeurs fixes dans une `Row` (`.photo { width: 240px; }`, `.texte { width: 360px; }`) restent côte à côte à 1280 px et passent l'une sous l'autre à 360 px, sans défilement de côté (mesuré à la relecture : à 360 px, `[[0,53,240],[0,227,360]]`, `scrollWidth` 360 ; à 1280 px, `[[320,53,240],[584,84,360]]`). Mais ni en parts 1 et 2, ni alignés en haut, ni au-delà de 640 px, ni avec un bandeau d'un bord à l'autre : pour cela, il faut les ajouts ci-dessous. Les quatre colonnes de « Dans la même série », elles, tiennent déjà dans 640 px, et passent à deux sur un téléphone (à une seule sous 256 px environ, ou quand le visiteur grossit son texte : calcul d'après `flat.rs:32`, non mesuré).

## Options comparées

| Manque | Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|---|
| Largeur de la page | A | `Page { max-width: 1100px; }` règle la colonne de la page (`main`, en-tête, pied) ; 640 px sans rien écrire | `main { max-width: 1100px; margin-inline: auto }` | `ConstrainedBox(constraints: BoxConstraints(maxWidth: 1100))` | aucun mot nouveau ; déjà accepté aujourd'hui (avec un mauvais effet : la page s'arrête à 1 100 px, collée à gauche ; mesuré) ; aucune page du dépôt n'écrit `Page { max-width }` (vérifié par `git grep`) | change l'effet d'un réglage déjà accepté |
| Largeur de la page | B | `Page(width: 1100px)` | idem | idem | la disposition dite dans un bloc, comme le veut `ADR-017` (règle 3) | un paramètre de plus ; deux façons si `max-width` reste accepté |
| Largeur de la page | C | garder 640 px pour le texte, et laisser certains blocs déborder : `width: wide` (la page) ou `width: screen` (l'écran) | la mise en page « en débord » (*breakout*) : une grille à lignes nommées | aucun | le texte reste lisible (environ 70 caractères par ligne) | deux idées à apprendre ; la règle de base de toutes les pages change |
| Bandeau d'un bord à l'autre | A | `.bandeau { width: screen; }`, comme `height: screen` ; le fond va d'un bord à l'autre, le contenu reste aligné sur la page | `main` en grille `[full] [content] [full]`, le bandeau sur `full` | `Container(width: double.infinity)` hors du `ConstrainedBox` | un mot déjà connu de HoloCode ; pas le défaut de `100vw` (une barre de défilement de côté) | la règle de base de `main` change : à vérifier sur les 81 leçons |
| Bandeau d'un bord à l'autre | B | un bloc `Band(children: …)` | idem | idem | se voit dans le plan de la page | un bloc de plus pour une seule allure |
| En colonne sous une largeur | A | `Row(columnBelow: 560px, …)` : côte à côte, puis l'un sous l'autre quand **la ligne** a moins de 560 px | l'astuce `flex-basis: calc((35rem - 100%) * 999)`, ou `@container (width < 35rem) { flex-direction: column }` | `LayoutBuilder` qui choisit `Row` ou `Column` | dépend de la place reçue, pas de l'écran ; garde les parts de `grow` ; pur CSS ; mesuré dans le prototype | un réglage de plus sur `Row` |
| En colonne sous une largeur | B | `phone: { … }` seulement (aujourd'hui), ou d'autres seuils d'écran : `tablet:`, `wide:` | `@media (max-width: …)` | `MediaQuery.of(context).size` | simple | dépend de l'écran : une carte dans une colonne étroite se trompe ; un style ne peut pas changer la disposition (`ADR-017`) |
| En colonne sous une largeur | C | deux versions du contenu, l'une cachée selon la largeur | `display: none` dans une requête | deux arbres dans `LayoutBuilder` | rien de nouveau | contenu écrit deux fois, deux noms pour un même bouton ; à refuser |
| Allure selon la place | A | un état de style `narrow: { … }`, avec le seuil écrit sur le bloc qui reçoit la place (`Column.carte(narrowBelow: 420px)`) | `container-type: inline-size` et `@container (max-width: 420px)` | `LayoutBuilder` | change l'allure (taille du titre, marges) selon la place reçue | un conteneur CSS ne peut pas prendre la taille de son contenu : dans une `Row`, sans `grow`, il s'écraserait à 0 ; le moteur devrait le refuser là |
| Allure selon la place | B | rien : `columnBelow` suffit pour la disposition, et la grille s'adapte déjà | — | — | rien à apprendre | un titre trop grand dans une carte étroite reste trop grand |
| Grille | A | `Grid(columns: 4, columnWidth: 200px)` : au moins 200 px par colonne avant d'en perdre une | `repeat(auto-fill, minmax(200px, 1fr))` | `SliverGridDelegateWithMaxCrossAxisExtent` | l'auteur choisit quand la grille se replie ; reste « au conteneur » | un réglage de plus |
| Grille | A' | `span: all` sur un enfant : il prend toute la ligne | `grid-column: 1 / -1` | `SliverToBoxAdapter` | sûr sur un téléphone | — |
| Grille | B | `span: 2` | `grid-column: span 2` | `StaggeredGrid` (paquet) | plus fin | sur un téléphone à une colonne, un bloc sur deux colonnes déborde : à refuser tant qu'on ne sait pas l'éviter |
| Alignement en hauteur | A | `Row(valign: top)` ; dans une `Column` haute (`height: screen`), `valign: center` centre en hauteur | `align-items` (ligne), `justify-content` (colonne) | `crossAxisAlignment` (ligne), `mainAxisAlignment` (colonne) | `align` dit toujours la largeur, `valign` toujours la hauteur : jamais d'axes qui s'échangent | un mot de plus |
| Alignement en hauteur | B | `Row(crossAlign: start)`, comme Flutter | idem | `crossAxisAlignment` | le mot de Flutter que Yocthan connaît | dans une `Column`, l'axe « cross » devient la largeur : la confusion qu'`ADR-024` a voulu éviter |

**Noms (`ADR-016`)**

- `max-width` sur `Page` : même sens en CSS ; Flutter dit `maxWidth`. Le seul risque est le changement d'effet d'un réglage déjà accepté ; aucune page du dépôt ne l'emploie.
- `width: screen` : HoloCode a déjà `height: screen` (« tout l'écran, au moins »). Ici, ce serait « d'un bord à l'autre » : le fond touche les bords, le contenu reste dans la colonne. Une petite différence à écrire dans le guide. Le web dit `100vw`, qui a un défaut connu.
- `columnBelow` : `Column` veut dire « l'un sous l'autre » dans HoloCode, en Flutter et en CSS (`flex-direction: column`). Risque faible. Je l'ai préféré à `stackBelow` (dans HoloCode, `Stack` veut dire « superposer » : vraie confusion) et à `wrapBelow` (une `Row` passe déjà à la ligne seule : on ne verrait pas la différence). `breakpoint` est écarté : sur le web, il désigne un seuil d'écran, pas de conteneur.
- `valign` : l'ancien attribut HTML `valign="top"` d'une case de tableau ; même sens (aligner en hauteur). Risque faible. `crossAlign` reprend Flutter, mais change d'axe entre `Row` et `Column`.
- `columnWidth` : en CSS, `column-width` (texte en colonnes de journal) veut dire « largeur idéale d'une colonne, au moins à peu près » ; sens voisin. Risque faible, comme pour `columns` (`ADR-024`, « Négatives »). `min` serait plus court, mais `Slider(min:)` veut déjà dire une valeur minimale.
- `span: all` : en CSS, `column-span: all` veut dire « sur toute la largeur » ; même sens. `span` seul évoque aussi la balise `<span>` du HTML, sans rapport : c'est pourquoi je ne propose que `span: all`.
- `narrow`, `narrowBelow` (plus tard) : aucun sens établi sur le web ni en Flutter. Risque faible, mais c'est un mot nouveau.

## Recommandation

Dans cet ordre :

1. **Corriger le défaut** : un style écrit pour `Main` doit viser `<main>` (une classe à poser, avec les noms de style de `Main`). Aujourd'hui, il est ignoré sans rien dire. (Ajout de la relecture : dans le même petit correctif, `box-sizing: border-box` sur `header` et `footer`, `flat.rs:16`.)
2. **La largeur de la page, option A** : `Page { max-width: 1100px; }` règle la colonne de la page. Sans rien écrire, on garde 640 px : aucune page existante ne change. Le seuil de `phone:` reste à 640 px.
3. **`Row(columnBelow:)`** : c'est la vraie « requête de conteneur » dont la page a besoin, écrite dans le bloc, sans état ni CSS à apprendre. Elle couvre la fiche, et aussi la carte qui change d'orientation selon la place reçue (une `Row` dans la carte). Le seuil s'écrit en pixels et le moteur l'écrit en `rem` : quand le visiteur grossit son texte, la ligne passe en colonne plus tôt, ce qui est voulu.
4. **`valign:`** dans `Row` et `Column`.
5. **`width: screen`** pour un bandeau d'un bord à l'autre, après une vérification par captures des 81 leçons, puisque la règle de base de `<main>` change.
6. **`Grid(columnWidth:)`** et **`span: all`**.
7. **Plus tard, si un vrai exemple le demande** : l'état de style `narrow:` (l'allure selon la place). `columnBelow` et la grille couvrent déjà la disposition.

Ne pas faire : d'autres seuils d'écran (`tablet:`), deux versions du contenu, `span: 2`.

## Exemple d'auteur

« proposé » marque ce qui n'existe pas aujourd'hui. Le reste est accepté par le moteur actuel.

```holo
// La fiche d'une œuvre, sur ordinateur et sur téléphone.
Page(
  title: "Lever sur le fleuve",
  children: [
    Column.bandeau(children: [ H1("Lever sur le fleuve") ]),
    Row(gap: 24px, columnBelow: 560px, valign: top, children: [   // proposé : columnBelow, valign
      Column(grow: 1, children: [ Image.photo(source: "lever.jpg", alt: "Un soleil jaune se lève au-dessus d'un fleuve bleu") ]),
      Column(grow: 2, gap: 8px, children: [
        Text.prix("120 euros"),
        "Peint en une seule matinée, au bord du fleuve.",
        A("Le mettre au panier", to: "panier.holo"),
      ]),
    ]),
    H2("Dans la même série"),
    Grid(columns: 4, columnWidth: 200px, gap: 16px, children: [   // proposé : columnWidth
      P.carte("Le phare"), P.carte("La barque"), P.carte("Dunes"), P.carte("Le verger"),
    ]),
  ],
)

Page { max-width: 1100px; background: #f7f2ea; color: #2a2118; }   // proposé : l'effet de max-width (accepté aujourd'hui, mais il colle la page à gauche)
.bandeau { width: screen; background: #2a2118; color: #fffaf3; padding: 32px 16px; }   // proposé : width: screen
.photo { width: 100%; border-radius: 14px; }
.prix { font-weight: bold; }
.carte { background: #fffaf3; padding: 12px; border-radius: 14px; }
```

Vérifié avec le moteur actuel :

```text
"$H" check - < essais-3-4-5/p4/exemple-p4-propose.holo
ligne 5, colonne 20 : « Row » n'a pas de paramètre « columnBelow » ; paramètres possibles : name, children, gap, align
"$H" check - < essais-3-4-5/p4/exemple-p4-existant.holo      # le même, sans columnBelow, valign, columnWidth, width: screen
ok
```

Le même, en HTML et CSS (aucun JavaScript n'est nécessaire). Le jumeau reprend ce que le moteur ajouterait seul : les `rem`, la grille qui perd des colonnes, le contenu aligné sur la colonne de la page, les 16 px entre les blocs. Il ajoute aussi une marge de côté de 1rem (`minmax(1rem, 1fr)`, `100% - 2rem`) que le moteur ne pose pas aujourd'hui (mesuré : `main` fait 360 px sur un écran de 360 px) : sans elle, le titre du bandeau (décalé de 16 px par son `padding`) et le contenu (collé au bord) ne seraient pas alignés sur un téléphone. Cette marge fait donc partie de la proposition `width: screen`, à décider avec elle.

```html
<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Lever sur le fleuve</title>
<style>
body { margin: 0; }
.page { min-height: 100vh; background: #f7f2ea; color: #2a2118; }
/* La colonne de la page (68,75 rem = 1100 px) ; le bandeau va d'un bord à l'autre. */
main { display: grid; grid-template-columns: [full-start] minmax(1rem, 1fr) [content-start] min(100% - 2rem, 68.75rem) [content-end] minmax(1rem, 1fr) [full-end]; }
main > * { grid-column: content; margin: 0 0 16px; box-sizing: border-box; } /* 16px, comme le moteur (flat.rs:17) */
main > .bandeau { grid-column: full; background: #2a2118; color: #fffaf3;
                  padding: 2rem max(1rem, calc((100% - 68.75rem) / 2)); }
.bandeau > * { margin: 0; }
/* Côte à côte, puis l'un sous l'autre quand la ligne a moins de 35 rem (560 px). */
.ligne { display: flex; flex-wrap: wrap; gap: 1.5rem; align-items: flex-start; }
.ligne > .part-1 { flex-grow: 1; flex-basis: calc((35rem - 100%) * 999); min-width: 0; }
.ligne > .part-2 { flex-grow: 2; flex-basis: calc((35rem - 100%) * 999); min-width: 0; }
.colonne { display: flex; flex-direction: column; gap: 1rem; }
.colonne.serree { gap: 0.5rem; }
.colonne > * { margin: 0; }
.photo { width: 100%; border-radius: 0.875rem; }
.prix { font-weight: bold; }
/* Quatre colonnes au plus, 12,5 rem (200 px) au moins chacune. */
.grille { display: grid; gap: 1rem;
          grid-template-columns: repeat(auto-fill, minmax(min(100%, max(12.5rem, calc((100% - 3 * 1rem) / 4))), 1fr)); }
.grille > * { margin: 0; min-width: 0; }
.carte { background: #fffaf3; padding: 0.75rem; border-radius: 0.875rem; }
</style>
</head>
<body>
<div class="page">
  <main>
    <div class="bandeau"><h1>Lever sur le fleuve</h1></div>
    <div class="ligne">
      <div class="part-1"><div class="colonne"><img class="photo" src="lever.jpg" alt="Un soleil jaune se lève au-dessus d'un fleuve bleu"></div></div>
      <div class="part-2"><div class="colonne serree">
        <span class="prix">120 euros</span>
        <p>Peint en une seule matinée, au bord du fleuve.</p>
        <a href="panier.html">Le mettre au panier</a>
      </div></div>
    </div>
    <h2>Dans la même série</h2>
    <div class="grille">
      <p class="carte">Le phare</p><p class="carte">La barque</p><p class="carte">Dunes</p><p class="carte">Le verger</p>
    </div>
  </main>
</div>
</body>
</html>
```

Ce qu'il faut savoir pour écrire le jumeau : une grille à lignes nommées, `min()`, `max()`, `calc()`, et une astuce de `flex-basis` multipliée par 999. Ce qu'il faut savoir en HoloCode : `columnBelow`, `valign`, `columnWidth`, `width: screen`.

## Par couche

- **Langage** : `Row(columnBelow:)`, `valign:` dans `Row` et `Column`, `Grid(columnWidth:)`, `span: all` sur un enfant de `Grid`, la valeur `screen` pour `width`, l'effet de `Page { max-width }`. Plus tard : l'état `narrow:` et `narrowBelow:`.
- **Moteur** : `flat.rs` (la classe de `<main>` ; une variable `--holo-page` dans la règle de base au lieu de 640 px ; la règle de `columnBelow`, qui pose la base de chaque enfant ; `valign` ; la grille de `<main>` pour `width: screen`, fabriquée seulement si la page s'en sert) ; `styles.rs` (`width: screen`) ; les vérifications : `columnBelow` refusé hors de `Row`, `span: all` hors de `Grid`, `valign` hors de `Row` et `Column`.
- **Enveloppe navigateur** : rien de nouveau dans `page-engine.js`. La vue points copie l'écran, pas la page : son budget ne change pas avec une page plus large (`page-engine.js:771`, `POINTS_MAX = 8e6` à `page-engine.js:49`). La vue personnage, où la page est une feuille dans un monde, n'a pas été essayée avec une page de 1100 px : non vérifié.
- **Services serveur** : aucun.

## Dépendances

- La conversion en `rem` (`ADR-061`) sert aux seuils (`columnBelow`, `columnWidth`), pour qu'ils suivent le texte du visiteur.
- La piste 5 (composants) en profite : une carte d'œuvre qui contient une `Row(columnBelow:)` s'adapte à la place que chaque copie reçoit.
- La piste 3 (design) : `valign: center` dans une `Column` en `height: screen` donne le bandeau d'accueil centré en hauteur.
- La 3D (`docs/04-roadmap/PLAN-3D.md`, en attente de Yocthan) : décider si la feuille de la vue personnage garde 640 px ou suit la page.
- L'absence de défilement de côté à 320 px ne se vérifie pas avec axe-core : aucune de ses règles ne la mesure. Elle se mesure à part (`scrollWidth` contre `clientWidth`), ce que `moteur/outils/capture.mjs` sait déjà faire avec `HOLO_GESTURES`, sans rien installer. L'audit axe-core (point 5, en attente de Yocthan) servira au reste (contrastes, noms des contrôles).

## Coût

- **Moteur** : environ 200 à 300 lignes de Rust, essais compris (estimation). La classe de `<main>` : quelques lignes (estimation).
- **Poids transféré** : 0 octet pour une page qui n'emploie rien de nouveau, si les règles nouvelles ne sont fabriquées que pour les pages qui s'en servent (à vérifier : HTML des 81 leçons identique avant et après). Pour une page qui s'en sert : quelques centaines d'octets de CSS au plus (estimation). Pour comparer : le CSS de l'accueil du site de référence fait 8 459 octets bruts, balises `<style>` comprises (mesure : `holo html exemples/site-reference/accueil.holo | grep -o '<style>.*</style>' | wc -c` → 8459). Aucun JavaScript.
- **Travail** : 2 séances pour les ajouts, plus une pour la vérification par captures de toutes les leçons à 360 et 1280 px (estimation), et deux leçons (la largeur de la page et le bandeau ; en colonne sous une largeur, aligné en hauteur).

## Accessibilité, déterminisme, budgets

- **Pas de défilement de côté** (WCAG 1.4.10, à 320 px) : `columnBelow` garde la promesse d'`ADR-024` (« rien ne déborde ») ; `span: 2` est écarté pour cette raison.
- **Texte agrandi** (WCAG 1.4.4) : les seuils en `rem` font passer la ligne en colonne plus tôt quand le texte grossit. À vérifier par la recette.
- **Ordre de lecture** : `columnBelow` ne change jamais l'ordre ; le lecteur d'écran et le clavier suivent l'ordre du fichier (WCAG 1.3.2, 2.4.3). Changer l'ordre à l'écran resterait refusé.
- **Lisibilité** : une page large garde des lignes de texte trop longues si l'auteur n'y prend pas garde. Le moteur pourrait limiter un paragraphe seul à environ 70 caractères par défaut (proposition à discuter).
- **Déterminisme** : le HTML et le CSS sont les mêmes pour tous les écrans ; seule la mise en page du navigateur dépend de la largeur. Même fichier, même page.
- **Budgets** : rien qui tourne pendant la visite (pas d'écoute de la taille de la fenêtre en JavaScript). Le coût des requêtes de conteneur dans le navigateur est négligeable à cette échelle (estimation). Aucune mesure de téléphone : pas d'appareil ici.

## Recette qui peut échouer

1. **Largeur de la page** : `Page { max-width: 1100px; }` à 1280 px. Doit : `main` fait 1100 px (± 1). Échec : 640 px (aujourd'hui, mesuré).
2. **Style de `Main`** : `Main { max-width: 900px; }`. Doit : `<main>` fait 900 px. Échec : le style ne vise rien (aujourd'hui).
3. **En colonne** : `Row(columnBelow: 560px)` avec `grow: 1` et `grow: 2`. Doit : à 360 px, même bord gauche et hauts différents ; à 768 px, côte à côte en parts 1 et 2 (± 2 px). Échec : côte à côte à 360 px (aujourd'hui avec `grow`, mesuré : 115 et 229 px).
4. **Texte à 200 %** : police par défaut à 32 px, largeur 768 px. Doit : la ligne précédente passe en colonne (le seuil de 35 rem vaut alors 1 120 px). Échec : côte à côte, avec un texte serré.
5. **Au conteneur, pas à l'écran** : la même carte (une `Row(columnBelow: 420px)` dedans) posée seule à 1280 px, puis dans une colonne de 300 px à 1280 px. Doit : horizontale, puis verticale. Échec : la même orientation dans les deux cas.
6. **Bandeau** : `width: screen` à 1280 px. Doit : le fond va de 0 à 1280 px (sans barre de défilement : `scrollWidth` égal à `clientWidth`), et le titre commence au même bord que le contenu (± 1 px). Échec : une barre de défilement de côté, ou un bord décalé (aujourd'hui, le fond de l'en-tête commence à 304 px et le contenu à 320 px ; le texte de l'en-tête, lui, est à 320 px).
7. **Alignement** : `valign: top`. Doit : les hauts des enfants égaux (± 1 px). Échec : centrés.
8. **Refus** : `columnBelow` sur une `Grid`, `span: all` hors d'une `Grid`, `valign` sur un `P`. Doit : refusés, avec l'endroit où les mettre.
9. **Rien ne change pour les pages anciennes** : captures des 81 leçons à 360 et 1280 px, avant et après. Doit : aucune différence de pixel. Échec : une seule.

## Objection

**La colonne de 640 px est peut-être une qualité, pas un manque.** Elle donne des lignes de texte lisibles, une page qui ressemble à une feuille (la vue personnage, le zoom vers les points), et un seul seuil à comprendre. Ouvrir les grandes largeurs, c'est ouvrir la porte aux mises en page d'ordinateur que beaucoup de débutants rateront : des lignes de 150 caractères, des grilles trop vides. Et le repli automatique (la ligne qui passe à la ligne, la grille qui perd des colonnes) couvre déjà la plupart des besoins. L'autre chemin : garder 640 px pour tout le texte, et n'autoriser que des blocs « en débord » (une grille, un bandeau) plus larges que le texte. Je garde ma recommandation, parce qu'une page de catalogue sur ordinateur à 640 px se voit comme un site de téléphone, mais l'option C (le débord) mérite d'être montrée à Yocthan à côté.

## Expérience requise

- **Montrer à Yocthan** la fiche et le catalogue en trois versions (aujourd'hui ; page large et `columnBelow` ; débord seulement), sur son PC à 1280 px et sur son téléphone. Le téléphone n'est pas branché ici : c'est à faire avec lui. Son verdict tranche entre A et C.
- **Essayer la vue personnage et la vue points** avec une page de 1100 px : la feuille dans le monde a-t-elle encore une bonne allure ?
- **Mesurer** à 320 px et à 200 % de texte : le défilement de côté avec `capture.mjs` (déjà là), le reste avec l'audit axe-core quand il sera installé (point 5).
- **Demander à deux débutants** de passer la fiche en deux colonnes sur ordinateur, avec le guide seul : comprennent-ils `columnBelow` sans aide ?

## Mises à jour de documents à prévoir

- `docs/01-holocode/GUIDE.md` § 4 bis : la largeur de la page (640 px sans rien écrire), `columnBelow`, `valign`, `columnWidth`, `span: all` ; § 5 : `width: screen` ; § 9 : `screen` pour une largeur ; § 10 (aide-mémoire) : les réglages de `Row`, `Column`, `Grid` ; § 11.
- Leçons nouvelles dans `exemples/lecons/` : la largeur de la page et le bandeau ; en colonne sous une largeur, et aligné en hauteur (numéros à choisir après 81). Leçon 6 et leçon 72 : un renvoi.
- `docs/01-holocode/NOMS.md` : les nouveaux noms ; corriger la ligne 216 (`@media`).
- `docs/01-holocode/COMPARAISON-WEB.md` § 2 : `@media` et les requêtes de conteneur ; le verdict du § 5.
- `docs/01-holocode/TABLEAU-WEB.md` : lignes 582 à 585 ; nouvelles lignes pour la largeur de la page et les requêtes de conteneur.
- `docs/02-gouvernance/adr/ADR-024-disposition.md` : « Ce qui reste à faire » (lignes 61 et 67).
- `docs/06-journal/JOURNAL.md` : l'entrée de l'étape, avec les captures (360, 768, 1280 px).

Relu le 2026-10-07 : 17 corrections.
