# Piste 3 — Un design plus précis

> Statut : EXPLORATION. Avis de Claude, pas une décision.

## Ce que Codex demandait

Sa ligne, dans le texte de l'issue #82 (copie locale non versionnée : `moteur/target/pistes-langage-pour-claude-2026-10-06.md:52`) :

> | **3** | **Design plus précis** : dégradés, ombres, typographie, couleurs partagées, états de survol/focus, dimensions souples. | Reproduire une identité visuelle professionnelle plutôt que seulement une disposition générale. |

Il ajoutait : « L'accessibilité doit accompagner chaque ajout : libellés, clavier, focus, annonces d'erreur et réduction des mouvements » (même fichier, ligne 61).

## État vérifié (main, 7a48def, 2026-10-07)

**Franchement : cette piste est en grande partie faite.** Les six mots de Codex ont tous une réponse dans le moteur. Il reste des finitions, et trois défauts que j'ai mesurés. Ce sont ces finitions qui séparent une page « propre » d'une identité de marque.

Pendant l'exploration, `main` est passé de 7a48def à 1119361 (PR 139 : la vérification d'un fichier de thème seul ; seuls `moteur/src/lib.rs`, `moteur/src/bin/holo.rs`, des tests de `flat.rs` (après sa ligne 1566) et le journal ont changé : `git diff --stat 7a48def 1119361`). J'ai relancé tous les essais avec le moteur reconstruit : mêmes réponses.

**Ce qui existe**

| Mot de Codex | Ce que fait le moteur | Où |
|---|---|---|
| Dégradés | `linear-gradient` (direction ou angle) et `radial-gradient`, de 2 à 5 couleurs ; une image de fond qui couvre toujours le bloc | `moteur/src/styles.rs:370-394` ; `ADR-041` § 3 ; leçon 51 |
| Ombres | `box-shadow`, `text-shadow`, trois ombres au plus | `styles.rs:404-412` ; `ADR-041` § 2 ; leçon 51 |
| Typographie | `font-size` (écrit en `rem`, et en `clamp` pour un titre de plus de 24px), `font-weight` (`normal`, `bold`), `font-style`, `font-family`, sa propre police par `Font`, `line-height` sans unité, `letter-spacing`, `text-transform`, `text-decoration`, `text-align` | `styles.rs:40-66` ; `flat.rs:313-352`, `flat.rs:432-440` ; `ADR-036` § 3, `ADR-041` ; leçons 50 et 54 |
| Couleurs partagées | les variables `--or`, définies dans le style de la page, d'un composant ou d'un nom ; un fichier de styles seuls importé comme thème | `styles.rs:73-106`, `styles.rs:273-284` ; `ADR-041` § 5, `ADR-050` § 6, `ADR-052` § 3 ; leçons 52 et 72 |
| États survol/focus | `hover:` (souris seulement), `focus:` (clavier, `:focus-visible`), `active:`, `dark:`, `phone:` ; un passage en douceur de 0,15 s ajouté seul, quand le style a `hover:` ou `active:` (pas pour `focus:` seul, `flat.rs:378`) ; sans rien écrire, l'anneau de focus du navigateur reste, car le moteur ne retire jamais `outline` (aucune occurrence dans `moteur/src`) | `moteur/src/holo.rs:101` ; `flat.rs:378-395` ; `ADR-036` § 4 ; leçon 37 |
| Dimensions souples | `%` ; les `px` écrits en `rem` pour les marges, largeurs, hauteurs et coins ; `height: screen` ; les grands titres qui rétrécissent | `flat.rs:424-441` ; `ADR-061` § 4 ; leçon 80 |
| (en plus) | un texte trop peu contrasté refusé, quand un même style donne la couleur et le fond | `styles.rs:163-201` ; `ADR-055` |

**Trois défauts mesurés**

1. **Le style de focus d'un champ ne s'applique jamais.** Le nom de style posé sur un `Input` va sur l'étiquette qui l'entoure (`<label class="holo-Input holo-s-champ">`, `flat.rs:781`), et `focus:` devient `.holo-s-champ:focus-visible` (`flat.rs:389`). Une étiquette ne reçoit jamais le focus : le champ, lui, le reçoit. Mesuré dans Chrome sans fenêtre, après la touche Tab : le champ a le focus, la bordure du style reste à 1px au lieu de 3px. Le focus reste visible, mais c'est l'anneau du navigateur (mesuré : `outline-style: auto` sur la case), jamais celui de l'auteur. Le style est ignoré sans rien dire, ce que HoloCode promet d'éviter (`ADR-017`, « Défauts … évités », ligne 69 : « une classe inconnue qui ne fait rien et ne dit rien » ; la règle 6, elle, ne parle que de la vérification des réglages). La bordure du style entoure aussi l'étiquette entière, pas la case du champ. Le même défaut touche tous les champs que le moteur enveloppe : `Checkbox`, `Choice` en menu, `Slider`, `Progress` (une étiquette, `flat.rs:756-799`, `:1040`, `:1060`) et `Choice` en boutons (un `fieldset`, `flat.rs:806`).
2. **Une taille donnée par une variable ne suit plus le visiteur.** `--pas: 8px` puis `padding: --pas` donne `padding:var(--pas)` avec `--pas:8px` : des pixels, pas des `rem`. `--titre: 40px` puis `font-size: --titre` donne `font-size:var(--titre)`, alors qu'écrit directement, `40px` donne `clamp(1.5rem,6.25vw,2.5rem)`. La cause : la variable est remplacée par `var(…)` avant la conversion (`flat.rs:409-411`, `flat.rs:429-434`). Les « jetons » de taille (une échelle d'espacement nommée une fois) perdent donc la promesse d'`ADR-061` § 4.
3. **Le contrôle du contraste ne voit pas tout.** Il ne mesure que les couleurs écrites dans un même style (`styles.rs:176`). Mesuré : `Page { background: white; }` et `H1 { color: #dddddd; }` (contraste 1,3 pour 1) passent ; c'est une limite connue (`ADR-055`, ligne 21). Il ne voit pas non plus le thème sombre défini dans `Page` : un bouton dont le texte et le fond deviennent la même couleur en sombre (contraste 1 pour 1) passe.

**Ce qui manque encore** (chaque ligne est un refus mesuré par `holo check`)

- Les graisses intermédiaires : `font-weight: 600` refusé (`styles.rs:44`). Et deux fichiers pour une même police sont refusés (« la police « Carlito » est chargée deux fois », `flat.rs:345-347`) : le gras d'une police de marque est alors fabriqué par le navigateur, un « faux gras » (comportement connu des navigateurs ; non mesuré ici).
- Un anneau de focus net qui ne pousse rien : `outline` refusé ; une ombre pleine `0 0 0 3px` refusée (quatre longueurs). Pour un anneau net et régulier, il ne reste que `border`, qui change la taille du bouton. Mesuré sur le style `.action` du site de référence (`exemples/site-reference/commun.holo:45`) : à la touche Tab, le bouton passe de 99,3 à 103,3 px de large, et son voisin se déplace de 4 px. Un contournement existe pourtant : deux ombres décalées (`focus: { box-shadow: 3px 3px 0 #2a2118, -3px -3px 0 #2a2118; }`) ou un halo flou (`0 0 6px #2a2118`) sont acceptés et ne poussent rien (mesuré à la relecture : 94,9 px avant et après Tab, voisin immobile) ; mais l'anneau est irrégulier aux coins, et une ombre disparaît en mode « contraste forcé » de Windows.
- Un fond en couches (un voile sombre sur une photo, pour que le titre se lise) : `background: linear-gradient(…), url("…")` refusé. Le contournement par `Stack` ne marche pas : mesuré, le voile ne couvre que 282 × 37 px d'une image de 300 × 188 px (chaque enfant d'un `Stack`, sauf le premier, est enveloppé dans un bloc centré, avec une marge de 6 px : `flat.rs:20`, `flat.rs:639-641`). Mais un autre contournement marche déjà : deux blocs imbriqués, la photo en fond du bloc extérieur (`background: url("…")`), le voile en fond du bloc intérieur, qui porte les marges intérieures (mesuré à la relecture : le voile couvre toute la photo, 640 × 159 px). Le contraste du voile n'est alors pas mesuré (un dégradé ne l'est jamais, `ADR-055`).
- Les proportions d'une image : `aspect-ratio` et `object-fit` refusés (`aspect-ratio` est déjà rangé « Plus tard » dans `TABLEAU-WEB.md:585` et dans `ADR-041`, ligne 43 ; `object-fit` n'est cité nulle part dans le dépôt).
- Les bordures fines : `border: none`, `border-bottom`, des coins différents (`border-radius: 12px 12px 0 0`) refusés.
- Les variables qui portent une ombre ou un dégradé : refusées (`styles.rs:280-282` : une couleur ou une taille).
- Les hauteurs souples : `min-height`, `min-width` refusés (`TABLEAU-WEB.md:570` cite déjà `min-width` et `max-height`).
- La courbe d'un passage : `transition: 0.3s ease-out` refusé ; seule la durée est permise.
- D'autres états : `disabled:` refusé ; `dark:` qui contient `hover:` refusé. Ce dernier cas se résout déjà avec des variables (mesuré : le CSS produit est juste).
- Refusés exprès, et c'est bien : `clamp`, `vw`, `rem`, `ch` écrits par l'auteur (`ADR-061`, tableau de comparaison). `text-align: justify` est refusé aussi (`styles.rs:47`), mais aucun document ne dit que c'est voulu : c'est seulement absent de la liste. Mon avis : garder ce refus, car un texte justifié se lit moins bien.
- Les couleurs `rgb()`, `rgba()`, `hsl()` sont refusées, mais `#8a3b1280` (8 chiffres, avec transparence) est accepté : la capacité existe.

**Documents en retard**

- `docs/01-holocode/TABLEAU-WEB.md:555` donne `font-weight` à 100 % : seules deux graisses existent.
- `TABLEAU-WEB.md:571` donne `border, border-radius` à 100 % : `none`, un côté seul, un coin seul sont refusés.
- `TABLEAU-WEB.md:594` donne les états à 100 % : le style de focus d'un champ ne s'applique pas.
- `docs/01-holocode/NOMS.md:189` ne range que les quinze réglages de base (« repris, quinze réglages ») ; les neuf du lot 4 sont ailleurs, à la ligne 69. Il y en a 24 en tout (`styles.rs:40-66`, et la liste du message de `p3-02` ci-dessous). « Le compte » (`NOMS.md:241`) dit encore « les quinze réglages de style ».
- `docs/01-holocode/NOMS.md:233-234` range encore `:hover`, `:focus`, `:active`, les variables CSS, les dégradés, les ombres et `@font-face` dans « Pas encore là », alors que `NOMS.md:40` et `:69-72` les donnent comme repris.
- `docs/01-holocode/COMPARAISON-WEB.md:3` (« État au 2026-10-03 »). Sa ligne 122 (« 24 réglages ») est juste.
- `docs/02-gouvernance/adr/ADR-017-forme-et-couleurs.md:95` : « pas encore appliqués à l'écran » ; ils le sont. Même fiche, ligne 96 : « seuls quinze réglages de base » (24 aujourd'hui) ; ligne 55 : « Un bloc porte un seul nom de style » (quatre depuis `ADR-050` § 7).
- `ADR-055`, ligne 12 : « quand un même style donne la couleur du texte et celle du fond (… variables comprises, dans chaque état : … sombre …) ». La fiche dit bien « un même style » ; mais une variable redéfinie dans le `dark:` de `Page` n'est pas vue par les autres styles qui l'emploient : seule sa première valeur compte (`styles.rs:77-91`). C'est le cas `p3-38` ci-dessous.

**Mesures** (PC Windows, Chrome sans fenêtre ; ce ne sont pas des mesures de téléphone : il n'y a pas d'appareil ici)

```bash
H=moteur/target/release/holo.exe          # essais dans essais-3-4-5/p3/
"$H" check - < p3-01.holo   # .c { font-weight: 600; }
ligne 2, colonne 6 : « font-weight: 600 » : ce réglage attend l'un de ces mots : normal, bold
"$H" check - < p3-02.holo   # .c { outline: 2px solid #8a3b12; }
ligne 2, colonne 6 : réglage inconnu « outline » ; réglages possibles : color, background, … transition
"$H" check - < p3-03.holo   # .c { background: linear-gradient(#00000080, #00000000), url("fond.jpg"); }
ligne 2, colonne 6 : « background: … » : ce réglage attend une couleur, un dégradé …, ou une image rangée à côté
"$H" check - < p3-10.holo   # .c { --ombre: 0 4px 12px #00000033; }
ligne 2, colonne 6 : « --ombre: 0 4px 12px #00000033 » : une variable porte une couleur ou une taille
"$H" check - < p3-12.holo   # .c { border: none; }
ligne 2, colonne 6 : « border: none » : ce réglage attend une épaisseur, un trait et une couleur
"$H" check - < p3-32-sombre-et-survol-par-variables.holo   # focus: { box-shadow: 0 0 0 3px --accent; }
ligne 4, colonne 102 : « box-shadow: 0 0 0 3px #8a3b12 » : ce réglage attend une ombre : décalage, flou et couleur
"$H" check - < p3-30-contraste-entre-styles.holo   # Page { background: white; } H1 { color: #dddddd; }
ok
"$H" check - < p3-31-contraste-meme-style.holo     # les deux couleurs dans un seul style : H1 { color: #dddddd; background: white; }
ligne 4, colonne 1 : « H1 » : le texte « #dddddd » sur le fond « white » a un contraste de 1,3 pour 1 ; il faut 4,5 pour 1 au moins …
"$H" check - < p3-38-contraste-sombre.holo   # en sombre : texte #e9b44c sur fond #e9b44c
ok
"$H" check - < p3-52-deux-graisses.holo
ligne 3, colonne 62 : la police « Carlito » est chargée deux fois
"$H" html p3-37.holo        # Page { --pas: 8px; } .c { padding: --pas; margin: --pas 0; }
.holo-Page{--pas:8px;}
.holo-s-c{padding:var(--pas);margin:var(--pas) 0;}
"$H" html p3-39.holo        # --titre: 40px ; font-size: --titre
.holo-s-t{font-size:var(--titre);}
"$H" html p3-40.holo        # font-size: 40px écrit directement
.holo-s-t{font-size:clamp(1.5rem,6.25vw,2.5rem);}
```

Le décalage dû au focus, et le focus d'un champ, mesurés dans Chrome par `moteur/outils/capture.mjs` (la touche Tab envoyée par le protocole de Chrome, puis une mesure dans la page) :

```text
node moteur/outils/capture.mjs file:///…/p3-50-focus.html p3-50-focus-tab.png 640 200 1200
→ {"largeurDuBouton":99.3,"placeDuVoisin":111.3,"focusVisible":false,"bordure":"1px"}   (avant Tab)
→ {"largeurDuBouton":103.3,"placeDuVoisin":115.3,"focusVisible":true,"bordure":"3px"}   (après Tab)
node moteur/outils/capture.mjs file:///…/p3-51-focus-champ.html p3-51-focus-champ.png 640 200 1200
→ {"focusSurLeChamp":true,"champFocusVisible":true,"bordureDuStyle":"1px","bordureDuChamp":"1px"}
node moteur/outils/capture.mjs file:///…/p3-54-voile.html p3-54-voile.png 640 480 1500
→ {"image":[300,188],"voile":[282,37]}
```

Les gestes et la mesure passent par la variable `HOLO_GESTURES` de `capture.mjs` (`Input.dispatchKeyEvent` pour Tab, puis `Runtime.evaluate`) ; les expressions ne sont pas recopiées ici. Relu le 2026-10-07 : remesuré avec des pages refabriquées par `holo html` (dossier `relecture-3-4-5/`), mêmes nombres (99,3 → 103,3 ; voisin 111,3 → 115,3 ; bordure du style 1px ; 300 × 188 contre 282 × 37).

Les 28 sondes de style (`p3-01` à `p3-28`) sont dans `essais-3-4-5/p3/` ; leurs réponses ne sont pas enregistrées à côté, mais je les ai relancées à la relecture : mêmes réponses.

## Le scénario du site de référence

Le cahier de Codex dit que Yocthan « juge personnellement la lecture, les gestes, la beauté » (`proposals/GPT5.6/site-reference-2026-10-06/README.md:124`). La recette demande un focus visible (`RECETTE.md`, cas F15) et un texte agrandi à 200 % sans perte (cas F02).

La tâche : **donner à « L'atelier des mondes » l'allure d'une vraie marque.** Concrètement :

1. la police de la marque en deux graisses, normale et grasse, avec ses vrais fichiers ;
2. un bandeau d'accueil sur une photo, avec un voile brun pour que le titre blanc se lise ;
3. des cartes dont les images ont toutes la même proportion (4/3), quelle que soit l'image ;
4. un anneau de focus net autour des boutons, qui ne fait rien bouger ;
5. un champ de recherche dont le focus se voit dans les couleurs de la marque ;
6. un thème sombre complet, sans texte illisible ;
7. une échelle d'espacement nommée une fois (`--rythme`) qui grandit avec le texte du visiteur.

Aujourd'hui : 3 et 5 sont impossibles. 1 ne passe que par un détour (déclarer le fichier gras comme une seconde famille, « Carlito Gras » : accepté par `holo check` à la relecture ; mais le gras d'un `**…**` reste un faux gras). 2 est possible en imbriquant deux blocs (mesuré à la relecture), pas par `Stack`. 4 ne passe que par un anneau d'ombres, irrégulier aux coins. 6 est possible mais pas vérifié. 7 marche, mais en pixels fixes. Le style actuel du site (`commun.holo:27-48`) montre le problème du point 4 : son focus est une bordure de 3px qui pousse les voisins.

## Options comparées

Trois façons générales d'avancer :

- **A. Les mots du CSS, avec des garde-fous** : la ligne choisie par `ADR-041` (« Les mots restent ceux du CSS ; ce qui change, ce sont les garde-fous »).
- **B. Des mots d'intention**, plus courts : `ring:`, `veil:`, `ratio:`.
- **C. Ne rien ajouter** : contourner, et documenter le contournement.

| Manque | Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|---|
| Graisses | A | `font-weight: 600;` et `Font(family: "Carlito", source: "carlito-bold.woff2", fontWeight: 700)` | `font-weight: 600` ; `@font-face { font-weight: 700 }` | `FontWeight.w600` ; `weight: 700` dans `pubspec.yaml` | les mots connus ; le vrai gras de la marque | un paramètre de plus pour `Font` |
| Graisses | B | `font-weight: semibold;` | aucun mot CSS « semibold » | aucun | se lit sans chiffres | un mot inventé, qui n'existe nulle part en CSS |
| Graisses | C | deux familles : `Font(family: "Carlito Gras", …)` et `font-family: Carlito Gras` | une seconde `font-family` | idem | rien à construire | trompeur ; le gras d'un mot en `**…**` reste un faux gras |
| Anneau de focus | A | `focus: { outline: 3px solid --encre; outline-offset: 3px; }` | `outline`, `outline-offset` | `FocusableActionDetector`, `InputDecoration.focusedBorder` | ne pousse rien ; mot connu ; contraste vérifiable | deux réglages de plus |
| Anneau de focus | A' | `box-shadow: 0 0 0 3px --encre;` (accepter la 4e longueur) | `box-shadow` avec étalement | `BoxShadow(spreadRadius:)` | aucun réglage nouveau | disparaît en mode « contraste forcé » de Windows, où `outline` reste |
| Anneau de focus | B | `ring: 3px --encre;` | `outline` + `outline-offset` | idem | un seul mot | un mot nouveau, à apprendre |
| Anneau de focus | C | rien de nouveau : `focus: { box-shadow: 3px 3px 0 --encre, -3px -3px 0 --encre; }` (accepté aujourd'hui ; mesuré à la relecture : rien ne bouge) | deux `box-shadow` décalées | `BoxShadow` | existe | anneau irrégulier aux coins ; disparaît en mode « contraste forcé » ; contraste non vérifié |
| Focus d'un champ | défaut à corriger | le style d'un `Input` décore la case du champ ; `focus:` suit le champ | `:focus-within` ou le style posé sur `input` | `InputDecoration` | le style de focus de l'auteur s'applique enfin (l'anneau du navigateur, lui, se voit déjà) | changer où va le style d'un champ (case ou étiquette) est un choix à faire |
| Fond en couches | A | `background: linear-gradient(#2a2118b3, #2a2118b3), url("atelier.jpg");` (deux couches au plus, l'image dessous) | la même écriture | `DecorationImage(colorFilter:)` ou `Stack` | l'écriture du CSS ; un voile qui se mesure | une syntaxe plus longue |
| Fond en couches | B | `background: url("atelier.jpg"); veil: #2a2118b3;` | idem | idem | se lit comme une phrase | un mot nouveau |
| Fond en couches | C | `Stack` qui étire un voile sur l'image (aujourd'hui, le voile ne s'étire pas : mesuré) | `position: absolute; inset: 0` | `Positioned.fill` | des blocs connus | à construire aussi ; plus de blocs pour un effet d'apparence |
| Fond en couches | C' | deux blocs imbriqués : `Column.photo(children: [ Column.voile(children: [ … ]) ])`, avec `.photo { background: url("atelier.jpg"); }` et `.voile { background: linear-gradient(#2a2118b3, #2a2118b3); padding: 48px 24px; }` (accepté aujourd'hui ; mesuré à la relecture : le voile couvre toute la photo) | un `div` dans un `div` | un `Container` dans un `Container` | rien à construire | deux blocs pour un effet d'apparence ; le contraste du voile n'est pas mesuré |
| Proportions d'image | A | `aspect-ratio: 4/3; object-fit: cover;` sur un style posé sur une `Image` | les mêmes | `AspectRatio`, `BoxFit.cover` | mots connus | `object-fit` n'a de sens que sur une image : le moteur doit le refuser ailleurs |
| Proportions d'image | B | `Image(ratio: 4/3)` | idem | `AspectRatio` | la proportion est dite sur l'image même | la disposition dans un bloc, l'apparence dans un style : la frontière se brouille |
| Bordures fines | A | `border: none;`, `border-bottom: 1px solid --brun;`, `border-radius: 14px 14px 0 0;` | les mêmes | `Border(bottom: BorderSide())`, `BorderRadius.only()` | mots connus ; la conversion en `rem` sait déjà lire plusieurs tailles (`flat.rs:430`) | quatre réglages de plus |
| Bordures fines | C | une `Hr()` sous un titre | `hr` | `Divider` | existe | ne remplace ni un onglet ni un coin |
| Jetons (variables) | A | `--ombre-douce: 0 4px 12px #2a211833;` accepté ; `--rythme: 8px` écrit en `rem` | variables CSS | `ThemeData`, `ColorScheme`, `TextTheme` | une identité écrite une fois, vérifiée | le moteur doit deviner la sorte de chaque variable (ombre, dégradé, taille) |
| Jetons (variables) | B | un bloc `Theme(colors: …, shadows: …)` | aucun | `ThemeData` | très lisible | `ADR-016`/`ADR-017` ont retiré `Theme` : deux façons de dire la même chose |
| Contraste | A | rien à écrire : le moteur calcule, pour chaque texte, sa couleur et son fond réels dans l'arbre des blocs, en clair et en sombre | axe-core dans le navigateur, après coup | aucun | possible parce qu'il n'y a ni cascade cachée ni sélecteur composé : une vraie avance sur le web | un calcul de plus à chaque vérification |
| Contraste | C | l'audit axe-core seulement (`moteur/outils/accessibility.mjs`) | idem | idem | existe | ne vérifie que les pages qu'on lui donne ; demande Playwright et axe-core (point 5 en attente de Yocthan) |
| Courbe du passage | A | `transition: 0.2s out;` avec les sept courbes d'`Enter` (`ADR-034`) | `transition-timing-function` | `Curves.easeOut` | une seule liste de courbes dans tout HoloCode | `ease-out` du CSS doit être corrigé en `out` |
| Courbe du passage | C | garder la durée seule | — | — | rien à apprendre | les boutons restent « mécaniques » |
| Hauteurs souples | A | `min-height: 320px;` (et `min-width`, `max-height`) | les mêmes | `ConstrainedBox` | mots connus ; écrits en `rem` | trois réglages de plus |

**Noms (`ADR-016`)**

- `outline`, `outline-offset` : sur le web, un trait tracé autour d'un élément, qui ne prend pas de place. Même sens. Flutter n'a pas ce mot. Risque faible.
- `font-weight: 600` : même sens en CSS ; Flutter dit `FontWeight.w600`. Aucun risque.
- **`Font(weight:)` serait un piège** : dans HoloCode, `weight:` veut déjà dire un poids en octets (`Image(source:, weight: 1KB)`, `GUIDE.md:116`, `:1720`). Une police a aussi un poids en octets. Je propose `fontWeight:` (et `fontStyle:`), qui reprend le mot CSS `font-weight` qu'on écrit juste après, dans le style.
- `aspect-ratio`, `object-fit` : même sens en CSS ; `AspectRatio` et `BoxFit` en Flutter. Aucun risque.
- `border-top` … `border-left` : même sens en CSS ; `BorderSide` en Flutter. Aucun risque.
- `out`, `in`, `smooth`… dans `transition` : ce sont déjà les noms d'`Enter` et `Loop` (`ADR-034`). Le web dit `ease-out`. Risque faible si le moteur refuse `ease-out` avec « écris « out » ».
- `ring`, `veil`, `ratio`, `semibold` (option B) : mots nouveaux. `ring` n'a pas de sens établi sur le web (Tailwind l'emploie pour l'anneau de focus) ; `veil` aucun ; `semibold` vient des logiciels de dessin, pas du CSS. Risque moyen : on s'éloigne du CSS que Yocthan a voulu garder (`ADR-017`).

## Recommandation

**Option A partout, dans cet ordre**, parce qu'elle suit la ligne déjà décidée (`ADR-041`) et n'apprend aucun mot nouveau à qui connaît le CSS :

1. **Corriger d'abord les trois défauts** : le focus des champs, les tailles par variable écrites en `rem`, le contraste calculé sur l'arbre entier (en clair et en sombre). Ce sont des promesses déjà faites (`ADR-017`, `ADR-055`, `ADR-061`) et non tenues.
2. **L'anneau de focus** : `outline` et `outline-offset`. Et, sans rien écrire, un anneau par défaut de 2px au moins, contrasté à 3 pour 1, sur tout ce qui se touche. (Aujourd'hui, c'est l'anneau du navigateur qui s'affiche : il se voit, mais ne suit ni la marque ni un contraste vérifié.)
3. **Les graisses** : `font-weight` de 100 à 900 (par centaines), `Font(fontWeight:, fontStyle:)`, une même famille en plusieurs fichiers.
4. **Les proportions d'image** et **les bordures fines**.
5. **Le fond en couches**, deux au plus, avec une règle mesurable : un texte posé sur une image doit avoir un voile qui garde le contraste, même si l'image était toute blanche ou toute noire. Le moteur peut le calculer sans voir l'image. (Moins pressant qu'il n'y paraît : deux blocs imbriqués font déjà le voile, mesuré ; l'ajout apporte surtout la mesure du contraste.)
6. **Les variables d'ombre et de dégradé**, puis la courbe des passages et les hauteurs souples.

Ne pas prendre l'option B : chaque mot d'intention serait plus court, mais ferait un deuxième vocabulaire à côté du CSS. Garder les refus de `clamp`, `vw`, `rem` écrits à la main et de `justify`.

## Exemple d'auteur

« proposé » marque ce qui n'existe pas aujourd'hui. Tout le reste est accepté par le moteur actuel.

```holo
// Un accueil aux couleurs de la marque.
Page(
  title: "L'atelier des mondes",
  lang: "fr",
  fonts: [
    Font(family: "Carlito", source: "carlito.woff2"),
    Font(family: "Carlito", source: "carlito-bold.woff2", fontWeight: 700),   // proposé : deux fichiers, une police
  ],
  children: [
    Column.hero(gap: 16px, children: [
      H1("L'atelier des mondes"),
      P("Des tableaux faits à la main."),
      A.action("Voir le catalogue", to: "catalogue.holo"),
    ]),
    Grid(columns: 2, gap: 16px, children: [
      Column.carte(gap: 6px, children: [
        Image.vignette(source: "lever.jpg", alt: "Un soleil jaune se lève au-dessus d'un fleuve bleu"),
        P("Lever sur le fleuve"),
        Text.prix("120 euros"),
      ]),
      Column.carte(gap: 6px, children: [
        Image.vignette(source: "porte.jpg", alt: "Une porte bleue dans un mur couleur sable"),
        P("La porte bleue"),
        Text.prix("90 euros"),
      ]),
    ]),
  ],
)

Page {
  --encre: #2a2118; --fond: #f7f2ea; --carte: #fffaf3;
  --brun: #8a3b12; --brun-survol: #a5481a; --sur-brun: #fffaf3;
  --ombre-douce: 0 4px 12px #2a211833;   // proposé : une variable qui porte une ombre
  --rythme: 8px;                         // existe ; proposé : écrite en rem, elle suit le texte du visiteur
  background: --fond; color: --encre; font-family: Carlito, Georgia, serif;
  dark: { --encre: #f5efe6; --fond: #1d1712; --carte: #2a2118; --brun: #e9b44c; --brun-survol: #ffd27a; --sur-brun: #1d1712; }
}
H1 { font-size: 40px; font-weight: 700; }   // proposé : 700 (aujourd'hui : bold)
.hero {
  background: linear-gradient(#2a2118b3, #2a2118b3), url("atelier.jpg");   // proposé : un voile sur la photo
  color: #fffaf3; padding: 48px 24px; border-radius: 16px;
}
.carte { background: --carte; border-radius: 14px; padding: --rythme; box-shadow: --ombre-douce; }
.vignette { aspect-ratio: 4/3; object-fit: cover; border-radius: 14px 14px 0 0; }   // proposé : ces trois réglages
.prix { font-weight: 600; }                 // proposé : la graisse 600
.action {
  background: --brun; color: --sur-brun; border-radius: 999px; padding: 10px 22px;
  transition: 0.2s out;                     // proposé : une courbe nommée
  hover: { background: --brun-survol; }
  focus: { outline: 3px solid --encre; outline-offset: 3px; }   // proposé : un anneau qui ne pousse rien
}
```

Les variables `--carte` et `--sur-brun` ne sont pas décoratives : sans elles, la carte claire garderait en sombre un texte clair (contraste 1,1 pour 1). Le contrôle proposé (défaut 3) le refuserait ; le contrôle actuel le laisse passer.

Vérifié avec le moteur actuel : l'exemple, privé de ses lignes « proposé » (remplacées par ce qui existe : `bold`, une seule police, `border` au focus, le dégradé seul), passe ; et la même page sans `--carte` en sombre passe aussi, alors qu'elle est illisible en sombre.

```text
"$H" check - < essais-3-4-5/p3/exemple-p3-existant.holo
ok
"$H" check - < essais-3-4-5/p3/exemple-p3-sans-carte-sombre.holo
ok
```

Le même, en HTML, CSS et JavaScript. Le jumeau reprend aussi ce que le moteur ajoute seul (la colonne de 640 px, les marges de 16 px, la grille qui perd des colonnes, les `rem`, le titre en `clamp`, le survol réservé à la souris, l'arrêt des passages pour qui demande moins de mouvement). Sans cela, il ferait moins.

```html
<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>L'atelier des mondes</title>
<style>
@font-face { font-family: "Carlito"; src: url("carlito.woff2"); font-weight: 400; font-display: swap; }
@font-face { font-family: "Carlito"; src: url("carlito-bold.woff2"); font-weight: 700; font-display: swap; }
body { margin: 0; }
.page {
  --encre: #2a2118; --fond: #f7f2ea; --carte: #fffaf3;
  --brun: #8a3b12; --brun-survol: #a5481a; --sur-brun: #fffaf3;
  --ombre-douce: 0 4px 12px #2a211833; --rythme: 0.5rem;
  min-height: 100vh; background: var(--fond); color: var(--encre); font-family: Carlito, Georgia, serif;
}
@media (prefers-color-scheme: dark) {
  .page { --encre: #f5efe6; --fond: #1d1712; --carte: #2a2118; --brun: #e9b44c; --brun-survol: #ffd27a; --sur-brun: #1d1712; }
}
main { display: block; max-width: 640px; margin: 0 auto; }
main > * { display: block; box-sizing: border-box; margin: 0 0 16px; } /* le moteur écrit 16px ici, pas 1rem (flat.rs:17) */
h1 { font-size: clamp(1.5rem, 6.25vw, 2.5rem); font-weight: 700; }
.colonne { display: flex; flex-direction: column; }
.colonne > * { margin: 0; }
.hero { gap: 1rem; background: linear-gradient(#2a2118b3, #2a2118b3), url("atelier.jpg") center / cover no-repeat;
        color: #fffaf3; padding: 3rem 1.5rem; border-radius: 1rem; }
.grille { display: grid; gap: 1rem;
          grid-template-columns: repeat(auto-fill, minmax(min(100%, max(7.5rem, calc((100% - 1rem) / 2))), 1fr)); }
.grille > * { margin: 0; min-width: 0; box-sizing: border-box; }
.carte { gap: 0.375rem; background: var(--carte); border-radius: 0.875rem; padding: var(--rythme); box-shadow: var(--ombre-douce); }
.vignette { aspect-ratio: 4 / 3; object-fit: cover; border-radius: 0.875rem 0.875rem 0 0; }
.prix { font-weight: 600; }
.action { background: var(--brun); color: var(--sur-brun); border-radius: 62.4375rem; padding: 0.625rem 1.375rem;
          transition: background .2s ease-out, color .2s ease-out, border-color .2s ease-out, opacity .2s ease-out,
                      box-shadow .2s ease-out, scale .2s ease-out, rotate .2s ease-out, letter-spacing .2s ease-out; }
@media (hover: hover) { .action:hover { background: var(--brun-survol); } }
.action:focus-visible { outline: 3px solid var(--encre); outline-offset: 3px; }
@media (prefers-reduced-motion: reduce) { .page, .page * { transition: none !important; } }
</style>
</head>
<body>
<div class="page">
  <main>
    <div class="colonne hero">
      <h1>L'atelier des mondes</h1>
      <p>Des tableaux faits à la main.</p>
      <a class="action" href="catalogue.html">Voir le catalogue</a>
    </div>
    <div class="grille">
      <div class="colonne carte">
        <img class="vignette" src="lever.jpg" alt="Un soleil jaune se lève au-dessus d'un fleuve bleu">
        <p>Lever sur le fleuve</p>
        <span class="prix">120 euros</span>
      </div>
      <div class="colonne carte">
        <img class="vignette" src="porte.jpg" alt="Une porte bleue dans un mur couleur sable">
        <p>La porte bleue</p>
        <span class="prix">90 euros</span>
      </div>
    </div>
  </main>
</div>
</body>
</html>
```

Ce que le jumeau ne fait pas, et que HoloCode ferait : refuser un oubli (`--carte` absent en sombre, un réglage mal écrit, une variable jamais définie). Ce que HoloCode ne fait pas, et que le web fait : rien de plus dans cet exemple.

## Par couche

- **Langage** : une dizaine de réglages CSS de plus (`outline`, `outline-offset`, `aspect-ratio`, `object-fit`, `border-top/right/bottom/left`, `min-height`, `min-width`, `max-height`) ; des valeurs de plus (`font-weight` 100 à 900, `border: none`, coins multiples, deux couches de fond, une courbe dans `transition`) ; `Font(fontWeight:, fontStyle:)`. Aucun bloc nouveau.
- **Moteur** : `styles.rs` (les vérifications, la sorte des variables, `object-fit` refusé hors d'une image) ; `flat.rs` (les `rem` pour les variables de taille, par exemple en écrivant deux versions d'une même variable, l'une en `rem`, l'autre en `px` pour les traits ; le focus d'un champ ; les `@font-face` avec `font-weight`) ; un nouveau calcul du contraste sur l'arbre des blocs, en clair et en sombre.
- **Enveloppe navigateur** : rien de nouveau dans `page-engine.js`. Tout devient du CSS fabriqué à l'avance ; la page légère reste légère (`ADR-033`).
- **Services serveur** : aucun. Les polices et les images sont des fichiers servis comme aujourd'hui.

## Dépendances

- L'état `current:` (le lien de la page où l'on est) dépend de la piste 5 (le moteur doit savoir quelle page il fabrique).
- Un état `disabled:` n'a de sens qu'avec un bouton qu'on peut désactiver : piste 6 (interactions).
- Les variables d'ombre et de dégradé dépendent de la correction des variables de taille (défaut 2), qui pose la façon de typer une variable.
- Le fond en couches dépend du calcul de contraste sur l'arbre (défaut 3), pour mesurer le voile.
- `width: screen` et l'alignement en hauteur sont dans la piste 4.
- La vue points n'est pas touchée : elle copie les pixels affichés, quels qu'ils soient (`page-engine.js:768-772`).

## Coût

- **Moteur** : environ 400 à 500 lignes de Rust, essais compris (estimation). Les réglages simples coûtent quelques lignes chacun dans `styles.rs:40-66` ; le calcul du contraste sur l'arbre est le plus gros morceau, environ 150 lignes (estimation).
- **Poids transféré** : 0 octet pour une page qui n'emploie rien de nouveau, à vérifier en comparant le HTML des 81 leçons avant et après (il doit être identique). Une page qui emploie un réglage nouveau porte quelques dizaines d'octets de CSS par réglage (estimation). Pour comparer : l'accueil du site de référence fabriqué fait 10 974 octets bruts, 3 171 octets compressés, dont 8 459 octets de CSS, balises `<style>` comprises (mesure, refaite à la relecture : `holo html exemples/site-reference/accueil.holo > a.txt` ; `wc -c < a.txt` → 10974 ; `gzip -9 -c a.txt | wc -c` → 3171 ; `grep -o '<style>.*</style>' a.txt | wc -c` → 8459). Un second fichier de police coûte autant que le premier : `carlito.woff2` pèse 30 156 octets (mesure : `wc -c exemples/lecons/carlito.woff2`) ; un fichier gras pèserait à peu près autant (estimation), seulement pour la page qui le déclare.
- **Moteur WebAssembly** : quelques Ko de plus au plus (estimation) ; à mesurer à la construction.
- **Travail** : 2 à 3 séances, avec trois leçons (les graisses ; l'anneau de focus ; le fond en couches et les proportions), le guide, les tableaux et les tests (estimation).

## Accessibilité, déterminisme, budgets

- **Focus** : l'anneau proposé suit WCAG 2.4.7 (focus visible) et 1.4.11 (contraste de 3 pour 1 pour ce qui n'est pas du texte). Le moteur peut vérifier ce contraste contre le fond de la page, comme il le fait pour le texte (aujourd'hui, à l'intérieur d'un même style seulement : défaut 3). L'anneau ne déplace rien : moins de surprise pour les personnes qui naviguent au clavier.
- **Contraste** : le calcul sur l'arbre entier tient enfin la promesse d'`ADR-055`, en clair et en sombre. Il est possible parce que HoloCode n'a ni cascade cachée ni sélecteur composé (seulement l'ordre fixe thème, type, nom, `ADR-017`) : la couleur réelle d'un texte se déduit du fichier seul, état par état.
- **Texte agrandi** : les tailles par variable suivront le réglage du visiteur (WCAG 1.4.4). Les traits et les ombres restent en pixels, comme aujourd'hui.
- **Moins de mouvement** : une courbe ne change rien ; les passages s'arrêtent toujours (`flat.rs:54`).
- **Déterminisme** : tout est fabriqué à la vérification. Même fichier, même CSS, octet pour octet. Une police qui arrive en retard change l'affichage un instant (`font-display: swap`), jamais le résultat final.
- **Budgets** : aucun JavaScript ajouté. Les polices restent huit au plus par page (`flat.rs:319`) ; il faudrait compter les fichiers, pas les familles, et peut-être un poids annoncé par police. Aucune mesure de téléphone : pas d'appareil ici.

## Recette qui peut échouer

1. **Focus d'un champ** : `Input.champ(…)` avec `.champ { focus: { outline: 3px solid #8a3b12; } }`, puis Tab. Doit : le contour du champ mesure 3px. Échec : rien ne change (c'est le résultat d'aujourd'hui, mesuré).
2. **Anneau sans décalage** : deux boutons `.action` côte à côte, Tab sur le premier. Doit : la largeur du premier et la place du second ne bougent pas de plus de 0,5 px. Échec : +4 px (aujourd'hui, avec `border`).
3. **Contraste en sombre** : l'exemple sans `--carte` dans `dark:`. Doit : `holo check` refuse, avec « en sombre » et le contraste mesuré. Échec : « ok » (aujourd'hui).
4. **Contraste entre deux styles** : `Page { background: white; }` et `H1 { color: #dddddd; }`. Doit : refusé. Échec : « ok » (aujourd'hui).
5. **Variables qui suivent le visiteur** : `--rythme: 8px` puis `padding: --rythme` ; régler la police par défaut du navigateur à 32px (`Page.setFontSizes` du protocole de Chrome). Doit : le `padding` calculé passe à 16px. Échec : il reste à 8px (aujourd'hui).
6. **Vraies graisses** : deux `Font` de même famille. Doit : `document.fonts.check("700 16px Carlito")` vrai, et deux fichiers demandés. Échec : refus « chargée deux fois » (aujourd'hui) ou un seul fichier chargé.
7. **Voile mesuré** : `color: #fffaf3` sur `linear-gradient(#2a2118b3, #2a2118b3), url(…)`. Doit : accepté (5,6 pour 1 au pire, calcul fait avec une image toute blanche). Avec un voile à `33` (20 %) : refusé. Échec : l'inverse.
8. **`object-fit` hors d'une image** : `.x { object-fit: cover; }` posé sur un `P`. Doit : refusé avec l'endroit où le mettre.
9. **Rien ne change pour les pages anciennes** : `holo html` des 81 leçons et des pages du site de référence, avant et après. Doit : identique octet pour octet. Échec : une seule différence non voulue.
10. **Moins de mouvement** : avec `prefers-reduced-motion`, `transition-duration` calculé vaut `0s` sur `.action`.

## Objection

**Chaque mot CSS ajouté nous rapproche du CSS qu'on voulait simplifier.** On passerait de 24 à 35 réglages (les onze de l'option A). Un expert du web trouvera toujours qu'il en manque ; un débutant trouvera qu'il y en a trop. L'autre chemin : investir dans de **bons choix par défaut** plutôt que dans des réglages. Un anneau de focus soigné sans rien écrire, une échelle de tailles de texte toute faite, des cartes et des boutons déjà beaux. L'identité d'une marque tient souvent à trois couleurs, une police et des coins arrondis, ce que HoloCode sait déjà faire. Je garde ma recommandation pour l'anneau de focus, les graisses et les trois défauts (ce sont des promesses, pas du confort) ; le reste peut attendre qu'un vrai site le demande.

## Expérience requise

- **Refaire une vraie identité** : prendre une maquette de designer (ou une page de marque publique) et la reproduire en HoloCode avec les 24 réglages d'aujourd'hui. Compter ce qui bloque et les contournements. Si moins de trois manques gênent vraiment, l'objection a raison.
- **Montrer à Yocthan** le site de référence avant et après l'anneau de focus, les graisses et le voile, sur son PC et sur son téléphone. Le téléphone n'est pas branché ici : c'est à faire avec lui.
- **Mesurer le faux gras** : une capture du titre en Carlito avec un seul fichier, puis avec le vrai fichier gras, pour voir si la différence se remarque.
- **Faire passer l'audit axe-core** sur le site de référence en sombre, à 390 et 1000 px, quand Yocthan aura dit oui à Playwright et axe-core (point 5).

## Mises à jour de documents à prévoir

- `docs/01-holocode/GUIDE.md` § 5 : le tableau des réglages, les états, les variables (tailles en `rem`, ombres), le focus d'un champ ; § 11 à relire.
- Leçons nouvelles dans `exemples/lecons/` : les graisses ; l'anneau de focus ; le fond en couches et les proportions d'image (numéros à choisir après 81). Leçon 37 (survol et focus) : montrer l'anneau.
- `docs/01-holocode/NOMS.md` : les réglages (lignes 189 et 241 : 15 → 24, puis les nouveaux ; lignes 233-234 à retirer de « Pas encore là »), `fontWeight`/`fontStyle` dans `Font`, et pourquoi pas `weight`.
- `docs/01-holocode/COMPARAISON-WEB.md` § 2 : le nombre de réglages, la date, les nouveaux réglages.
- `docs/01-holocode/TABLEAU-WEB.md` : lignes 555, 570, 571, 585, 594, 595 ; nouvelles lignes pour `outline` et le fond en couches.
- `docs/02-gouvernance/adr/ADR-017` (lignes 55, 95 et 96) et `ADR-055` (ligne 12) : préciser ce qui est fait.
- `docs/06-journal/JOURNAL.md` : l'entrée de l'étape, avec les captures et les erreurs.

Relu le 2026-10-07 : 26 corrections.
