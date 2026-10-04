# Contre-revue de mes propositions de noms — A1 de la PR 39

## Contribution

- Sujet : appliquer à mes propres noms les objections adressées à ceux de Claude ; répondre aux six questions de Yocthan.
- Auteur : Codex, contribution dans `proposals/GPT5.6/`, 2026-10-04.
- Discussions sources : `HC-013` ; demande de Yocthan dans cette conversation ; [PR 39](https://github.com/yocthanmabeka/Metaverse/pull/39), [PR 40](https://github.com/yocthanmabeka/Metaverse/pull/40) et entrée du journal « La seconde revue de Codex : ses défauts corrigés » du 2026-10-04.
- Décisions concernées : `ADR-016`, `ADR-017`, `ADR-020`, `ADR-021`, `ADR-022`, `ADR-023`, `ADR-024`.
- Statut proposé : **PROPOSITION**, à juger par Yocthan. Aucun statut de décision modifié.
- Référence relue : `origin/main`, `518c56485d6b2dabdd101c40e786f680927320d0` (PR 50 incluse).
- Sources locales : [A1 initiale](../revue-langage-securite-2026-10-03/README.md), [journal](../../../docs/06-journal/JOURNAL.md), [noms](../../../docs/01-holocode/NOMS.md), [guide](../../../docs/01-holocode/GUIDE.md), [réglages et validation](../../../moteur/src/vue.rs), [placement réel](../../../moteur/web/page.html).
- Périmètre de cette contribution : ce document seulement. Les migrations décrites sont hypothétiques, pas réalisées.

## Résultat

Ma proposition A1 n'était pas assez cohérente pour être adoptée en bloc. Elle remplaçait parfois un mot simple par du jargon, introduisait une nouvelle convention d'écriture sans la justifier et mélangeait renommage et changement de fonctionnement. Le fait d'avoir trouvé des défauts du moteur ne valide pas mes noms.

Les corrections de la PR 40 changent aussi l'argument : `Portals(count:)` borne désormais les portails montrés. Je ne peux plus invoquer l'ancien défaut pour imposer `maxVisible`. J'ai relu les trois attentes retournées et exécuté le harnais : **7 tests réussis**. Cela ne constitue pas une nouvelle validation dans Chrome des dix défauts cités par Yocthan ; les preuves navigateur restent celles rapportées dans le journal et la PR 40.

### 1. Une règle commune, appliquée aussi à mes mots

**Règle proposée : un mot anglais simple pour chaque notion nouvelle ; la catégorie grammaticale fixe son écriture : initiale majuscule pour les blocs, minuscules pour les réglages, valeurs symboliques, signaux et demandes. Les noms repris d'un standard conservent leur graphie et leur sens. Une seule graphie canonique par nom.**

Cela conserve `font-size`, `max-width`, `KB` et `H1` : les normaliser en mots nouveaux détruirait précisément la familiarité recherchée. Ce sont des catégories explicites, pas des exceptions accordées à mes propositions. Pour un nouveau réglage dont le sens exige deux mots, on doit d'abord discuter la structure ou la convention ; on n'introduit ni camelCase ni concaténation illisible au cas par cas.

`name`, `title`, `seed`, `children` et `fragments` sont compatibles aussi bien avec des mots simples qu'avec lowerCamelCase : leur graphie ne prouve pas, à elle seule, une règle. Mais ADR-020 dit « en minuscules », le journal refuse les mots collés, et ma série `maxScale`, `minSize`, `pointSize`, `subdivideAt`, `maxTilt`, `maxNesting`, `maxVisible` ne respecte pas cette direction. **Je retire cette série comme convention à adopter.**

Si Yocthan choisissait au contraire lowerCamelCase, ces cinq mots existants resteraient identiques : on ne doit pas les allonger artificiellement. Il faudrait alors préciser ADR-020, le guide et la coloration de l'éditeur, sans convertir les propriétés CSS en `fontSize`. Ce choix ne réglerait aucun des problèmes de sens ci-dessous. Ici, je recommande de conserver la convention actuelle.

Application à tout le vocabulaire inventorié, y compris les ajouts de Claude après la PR 39 :

| Famille | Application de la règle | Conséquence |
|---|---|---|
| Blocs de contenu | `Page`, `Text`, `P`, `H1`, `H2`, `H3`, `A`, `Button`, `Image`, `List`, `Point`, `World` | Garder leur graphie ; `A`, `P`, `H1` sont des noms repris, à apprendre, pas des mots transparents pour un novice. |
| Blocs de règles, données, disposition et vue | `On`, `State`, `Prices`, `Row`, `Column`, `Grid`, `Zoom`, `Points`, `Relief`, `Portals` | Même règle. Le pluriel est possible pour une configuration collective ; il ne prouve pas une collection. Pas de suffixe `View` obligatoire. |
| Contenu et relations | `name`, `title`, `children`, `text`, `to`, `source`, `ordered`, `rules`, `effect`, `inside` | Garder. Pas de `targetName`, `childNodes` ou `sourceUrl` introduits pour imiter mes noms composés. |
| État et disposition | `state`, `prices`, `gap`, `align`, `columns` | Garder l'écriture. `Grid(columns:)` compte des colonnes ; ce n'est pas une promesse d'implémenter le raccourci CSS `columns`. `align` demande encore une explication de l'axe concerné. |
| Monde et ressources | `seed`, `fragments`, `palette`, `brightness`, `color`, `budget`, `weight` | Garder. Le poids déclaré et le budget réel restent deux contrats différents ; un renommage ne corrige pas B-03. |
| Vue et points plantés | `pixels`, `above`, `zoom`, `points`, `relief`, `portals`, `active`, `max`, `shrink`, `levels`, `speed`, `after`, `size`, `fragment`, `grid`, `depth`, `density`, `height`, `tilt`, `layout`, `count`, `duration` | Tous sont conformes à la graphie ; leur sens est évalué séparément dans le tableau principal. Conformité de forme ne signifie pas bon nom. |
| Valeurs symboliques | `start`, `center`, `end`, `between`, `grid`, `row`, `column`, `diagonal`, `true`, `false`, `auto` | Minuscules. `auto` est reconnu lexicalement, pas une valeur universellement admise par tous les réglages. `topRight`, proposé par moi, n'appartient pas à cette convention ni au langage actuel. |
| Signaux, capacités, demandes et valeurs calculées | `tap`, `enter`, `leave`, `portals`, `add`, `sub`, `set`, `{count}`, `{total}` | Même règle de minuscules. `cart.add(1)` est une demande, pas un bloc : l'exception introduite par ADR-023 doit être expliquée, plutôt que répéter « toute parenthèse ouvre un bloc ». |
| Les quinze réglages de style | `color`, `background`, `font-size`, `font-weight`, `font-style`, `font-family`, `text-align`, `padding`, `margin`, `border`, `border-radius`, `width`, `height`, `max-width`, `opacity` | Garder exactement l'écriture CSS. Garder aussi les valeurs admises (`normal`, `bold`, `italic`, `left`, `center`, `right`, `solid` et noms de couleurs). Ne pas appliquer un remplacement global à `height` ou `size`. |
| Unités | `px`, `deg`, `ms`, `s`, `%`, `B`, `KB`, `MB`, `GB`, `mm`, `cm`, `m`, `km`, `min`, `h` | Graphies conventionnelles conservées, dont les majuscules des octets. Reconnaissance d'une unité ne signifie pas prise en charge partout. |
| Directives et ponctuation | `import`, `module`, `bridge js`, `bridge css`, `//`, Markdown, interpolation `{…}` | Garder la graphie ; les imports/ponts restent distincts de capacités effectivement implémentées. Aucun renommage induit. |
| Noms choisis par l'auteur | `Open`, `Workshop`, `sunrise`, `blue_door`, `.card` | Hors du vocabulaire réservé : ne pas renommer les identifiants des utilisateurs pour uniformiser les mots du langage. |

### 2 et 3. Tableau principal : mes propositions face au sens et au débutant

« Débutant » signifie ici non-programmeur, éventuellement francophone. Je n'ai effectué aucun essai utilisateur : les comparaisons sont des hypothèses linguistiques, **pas des résultats mesurés**. Un mot anglais courant n'est pas forcément compris d'une personne qui ne connaît pas l'anglais. Aucun des noms ne suffit à faire deviner toutes les bornes et tous les comportements.

La colonne « ma proposition » conserve mes candidats de la PR 39, même ceux que je retire. `Changer` signifie « je recommande ce changement à Yocthan », pas « décision prise ».

| Mot actuel | Ma proposition (PR 39) | Ce que tu y gagnes | Ce que tu y perds | Ma recommandation finale |
|---|---|---|---|---|
| `pixels:` | `points:` | Le type des éléments `Point` devient plus visible. | `points:` est déjà la configuration de rendu ; ma solution exige deux autres renommages. « Pixel » parle déjà à beaucoup de débutants et décrit la taille initiale. Aucun des deux ne dit « sites plantés ». | **Garder** `pixels` pour cette étape ; retirer la migration en chaîne. |
| `above:` | `anchor:` et `placement:` | `anchor` nomme le bloc servant de repère ; `placement` distinguerait sa position. | `above` est plus simple à lire. `anchor` évoque aussi un lien ou une coordonnée ; `placement` ajoute un choix inexistant. Le code place bien le point au-dessus du repère, mais au bord droit de la page : ma critique initiale était trop absolue. | **Changer** seulement `above` en `anchor` comme nom de référence, en conservant le placement actuel et la portée locale. Ne pas ajouter `placement` avec ce renommage. Risque pour le novice à vérifier. |
| `Zoom` | `Zoom`, mais déplacer `levels` ailleurs | Le nom actuel exprime bien l'interaction ; séparer la navigation pourrait clarifier l'emboîtement. | Déplacer `levels` exige un conteneur que je n'ai pas défini. Pour le novice, `Zoom` est déjà le plus familier. | **Garder** `Zoom`. Discuter séparément la structure de navigation. |
| `Zoom(max:)` | `maxScale:` | Précise que la borne porte sur une échelle. | `Zoom` donne déjà ce contexte ; le composé allonge une expression courte. « Scale » est aussi polysémique et n'aide pas nécessairement un novice. | **Garder** `max`, documenté comme facteur maximal de grossissement. |
| `Zoom(shrink:)` | `minSize: 1px` / `minSize: page` | Rendrait la taille minimale explicite. | Ce n'est plus un booléen : j'invente un domaine mêlant longueur et symbole `page`, ainsi que des tailles intermédiaires et un axe à définir. Le booléen actuel est plus simple à manipuler après une phrase d'explication. | **Garder** `shrink`. Retirer `minSize` : ce serait une nouvelle fonctionnalité. |
| `Zoom(levels:)` | `maxNesting:` hors de `Zoom` | Dit « emboîtement maximal ». | Terme technique plus long ; conteneur absent ; ne dit pas encore si la page racine compte. `levels` est plus simple pour un novice, mais imprécis sans contexte. | **À discuter** : conserver provisoirement `Zoom(levels:)`, ne pas déplacer sa donnée par simple remplacement de texte. |
| `points: Points(...)` | `pointView: PointView(...)` ou `PointRendering` | Affiche l'intention de réglage du rendu. | Deux mots, deux changements et un suffixe pouvant annoncer une nouvelle vue à instancier. `Points` est plus facile à lire ; la valeur est déjà un bloc de réglages, pas une liste. | **Garder** `points: Points`. Retirer les deux variantes. |
| `Points(after:)` | `threshold:` ou `startAt:` | `threshold` indique un seuil ; `startAt` un début. | Aucun ne dit « facteur de zoom » ; `startAt` peut être temporel et casse la convention. `after` est plus courant que `threshold` pour un novice anglophone. | **Garder** `after` pour cette étape. Retirer l'affirmation que `threshold` serait nettement meilleur. |
| `Points(size:)` | `pointSize:` | Précise l'objet dimensionné hors de son contexte. | `Points` dit déjà lequel ; le vrai problème est que c'est une taille d'apparition, pas une taille fixe. Le novice comprend plus facilement `size`. | **Garder** `size`, en expliquant ce seuil en pixels. |
| `Points(fragment:)` | `subdivideAt:` | Dit qu'une subdivision démarre à la taille donnée. | Deux mots ; « subdivide » est du jargon ; conserve l'ambiguïté de la grandeur sans `px`. Le mot de Claude est plus court, mais laisse croire à une quantité ou à un fragment. Aucun gagnant sans explication. | **À discuter**. Retirer `subdivideAt` comme choix prêt à adopter ; garder provisoirement `fragment`, sans prétendre que le problème est résolu. |
| `Points(grid:)` | `divisions:` | Distingue la quantité par axe de la mise en page `Grid`. Usage proche de Three.js ; `4` signifie quatre cellules par axe, donc seize enfants. | Plus long ; le novice peut compter quatre enfants au total, ou quatre traits donnant cinq cases. Le mot `grid` donne mieux l'image d'une grille. | **Changer** en `divisions`, avec schéma `4 × 4 = 16` et définition « cellules par axe », pas « traits de coupe ». |
| `Points(depth:)` | `levels:` ou `maxSubdivisions:` | `levels` désigne des étapes successives de subdivision plutôt qu'une distance. | `levels` existe dans `Zoom` ; deux comptes distincts demandent leur bloc. `maxSubdivisions` est long et pourrait compter toutes les subdivisions plutôt que leur succession. Pour un novice, « niveaux » est plus concret, mais pas autonome. | **Changer** en `Points(levels:)` ; ne pas changer simultanément `Zoom(levels:)`. Rejeter `maxSubdivisions`. Expliquer les deux comptes, ci-dessous. |
| `Relief` / `relief:` | `Depth` / `depth:` | Évoque une dimension spatiale. | `Depth` évoque aussi le test/tampon de profondeur, sans mieux décrire la rotation. `Relief` est particulièrement accessible au francophone. Son origine linguistique ne prouve pas un défaut d'API. | **Garder** `Relief` et `relief`. Retirer `Depth` ; ma justification initiale était insuffisante. |
| `Relief(height:)` | `elevation:` | Évoque un déplacement vertical ou selon Z. | En Flutter, l'élévation situe une surface et commande notamment son ombre ; ici la luminance module le relief. `height` est plus simple, `elevation` ajoute une attente. | **Garder** `height`. |
| `Relief(tilt:)` | `maxTilt:` | Dit qu'il s'agit d'une borne plutôt que d'un angle imposé. | Ne corrige pas « incliner » face à la rotation libre. Le code actuel rend la rotation libre dès un demi-tour : une borne maximale uniforme serait une description fausse. `tilt` reste plus court, ni l'un ni l'autre n'explique l'activation. | **À discuter** : retirer `maxTilt`. Il faut d'abord clarifier le contrat 0°, angle limité, rotation libre, puis choisir un nom. |
| `portals: Portals(...)` | `portalView: PortalView(...)` | Explicite une vue de portails. | Un suffixe ne dit pas si l'on crée la vue ou règle son comportement. Le pluriel actuel n'est pas une erreur grammaticale ; `Portals` est plus court pour le novice. | **Garder** `portals: Portals`. |
| `Page.portals` (capacité) | `Page.openPortals` | Le verbe explicite l'ouverture. | Composé en conflit avec la règle ; un second nom à apprendre pour le même carrefour. Ici le gain pour un novice connaissant l'anglais est réel, mais insuffisant pour changer seul le style du langage. | **À discuter** ; retirer `openPortals` comme solution immédiate. Garder provisoirement la capacité actuelle. |
| `Portals(count:)` | `maxVisible:` | Indique une capacité d'affichage bornée. | Plus long et ne dit pas « portails » hors contexte. Après PR 40, `count` compte effectivement les places de mondes montrées ; « + N autres » est un indicateur, pas un monde de plus. `count` est plus simple. | **Garder** `count`. La correction du comportement répond au grief principal. |
| Placement fixe, aucun réglage | `placement: topRight`, puis `offset:` | Permettrait de choisir un côté et un décalage. | Nouvelle API et nouvelles conventions, pas un renommage. `topRight` suggère le coin du bloc, alors que X est actuellement le bord de la page. `offset` est encore du jargon. | **À discuter** uniquement comme fonctionnalité séparée ; ne pas l'inclure dans la migration de `above`. |

Deux rectifications de fond : une collection et ses réglages peuvent partager une racine sans tromper (`prices: Prices`, `portals: Portals`). Et un mot déjà employé ailleurs n'est pas automatiquement interdit : c'est le sens promis, dans son contexte, qu'il faut comparer. J'avais traité certaines ressemblances comme des preuves de mauvais nom, puis oublié de faire le même contrôle sur mes candidats.

### 2 bis. ADR-016 appliquée aux neuf mots demandés

Les références ci-dessous sont des spécifications ou documentations des projets concernés, consultées le 2026-10-04. Je distingue un nom réellement présent dans une API d'une simple association ; je ne prétends pas prouver l'absence d'un nom dans toutes les bibliothèques.

| Mot que j'ai proposé | Sens existant vérifié | Risque pour HoloCode et verdict |
|---|---|---|
| `threshold` | Dans [Intersection Observer](https://www.w3.org/TR/intersection-observer/), le seuil est un rapport d'aire entre 0 et 1. | Ce n'est pas un mot-clé JavaScript. L'idée générale de seuil est conservée, mais `threshold: 4` n'a pas ce contrat et ne précise toujours pas la grandeur. Pas un progrès suffisant sur `after`. |
| `anchor` | [HTML](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element) : ancre hypertexte ; [CSS Anchor Positioning](https://www.w3.org/TR/css-anchor-position-1/) : élément de référence pour le placement ; [Flutter Viewport.anchor](https://api.flutter.dev/flutter/widgets/Viewport/anchor.html) : position relative de l'origine du défilement. | Collision réelle avec lien et coordonnée. Pour `Point(anchor: Open)`, une référence locale de placement reste cohérente avec le sens CSS, sans être une implémentation de sa syntaxe. Ne jamais interpréter ce nom comme une URL, un `href` ou une fraction de viewport. Candidat défendable, pas universellement évident. |
| `placement` | [Floating UI, bibliothèque JavaScript](https://floating-ui.com/docs/computeposition#placement) : position relative à un élément de référence, avec côtés et alignements logiques. | Sens proche ; pas une propriété native de JavaScript que j'aurais « reprise ». Mon `topRight` n'est pas sa valeur `top-end`, et le moteur n'offre pas ces placements. Reporter la fonctionnalité. |
| `divisions` | [Flutter Slider](https://api.flutter.dev/flutter/material/Slider/divisions.html) : subdivisions discrètes de l'intervalle ; [Three.js GridHelper](https://threejs.org/docs/pages/GridHelper.html) : divisions de la grille. | Même idée de partitions, pas d'itérations récursives. Compatible sous réserve de dire par axe et de distinguer divisions, traits et enfants. Ne pas faire croire que la valeur est le total des enfants. |
| `levels` | [Blender, Subdivision Surface, documentation 3.6](https://docs.blender.org/manual/en/3.6/modeling/modifiers/generate/subdivision_surface.html) : niveaux successifs de subdivision. | Sens proche de `Points(levels:)`, sans promettre l'algorithme de lissage de Blender. Collision locale avec `Zoom(levels:)`, conservée et explicitée ; le seul mot ne dit pas ce qu'on compte. |
| `elevation` | [Flutter Material](https://api.flutter.dev/flutter/material/Material/elevation.html) : coordonnée Z relative au parent, qui influe sur l'ombre et l'apparence. | Notre amplitude de relief selon la luminance n'est pas l'élévation uniforme d'une carte Material. Proximité géométrique, attente de comportement différente : retirer. |
| `Depth` | [WebGPU, état profondeur/stencil (édition 2022)](https://www.w3.org/TR/2022/WD-webgpu-20220531/#depth-stencil-state) : comparaison et écriture de profondeur. Le terme existe aussi comme dimension spatiale. | Je ne prétends pas qu'il existe un bloc natif `Depth` identique. Mais ce nom nu laisse attendre une profondeur ou un réglage d'occlusion ; il décrit mal relief et rotation ensemble. Retirer. |
| `PointView` | Pas d'équivalent exact établi dans les sources examinées. [Flutter View](https://api.flutter.dev/flutter/widgets/View-class.html) construit un arbre de rendu associé à une `FlutterView`. | L'association avec une vue concrète est plausible, pas une collision exacte démontrée. Le singulier `Point` peut évoquer une seule entité. Le composé n'apporte pas assez pour remplacer `Points`. |
| `PortalView` | Pas d'équivalent exact établi ; même association documentée avec `View`. | Peut se lire comme une vue à travers un portail unique, plutôt que les réglages du carrefour entier. Ne pas inventer un consensus technique autour de ce nom : retirer. |

ADR-016 ne peut pas signifier « aucun mot ne possède un autre sens dans aucun domaine » : `Point` a précisément fait l'objet d'une décision explicite malgré son sens de coordonnée. Je propose d'appliquer partout le même examen : même concept général, même nature de valeur, attentes non trompeuses dans le bloc. C'est une interprétation soumise à Yocthan, pas une réécriture de l'ADR.

Pour les deux `levels`, la documentation devra toujours qualifier le bloc :

| Expression proposée ou actuelle | Ce qui se compte | Cas limite |
|---|---|---|
| `Points(levels: 3)` — proposé | Trois subdivisions successives d'un point de la mosaïque | `0` signifie aucune subdivision ; `divisions: 4` signifie 4 × 4 enfants à chaque subdivision. |
| `Zoom(levels: 3)` — actuel | Trois niveaux de sites emboîtés, page racine comprise | `1` interdit un monde inline supplémentaire ; ce n'est ni le nombre de gestes de zoom ni une limite universelle sur les passages entre fichiers. |

Ce doublon reste un coût. Je préfère le reconnaître plutôt que prétendre que `levels` devient soudain univoque lorsqu'il vient de moi. Il est comparable à `size` dans `Points` et `Portals` : le bloc qualifie le nom. Déplacer la limite de sites reste une question d'architecture séparée.

### 4. Coût : ce qui casse et où

Un remplacement strict rend les anciens `.holo` incompatibles avec le nouveau moteur et les nouveaux fichiers incompatibles avec l'ancien. Les validateurs refusent déjà les paramètres inconnus. Plus dangereux, certains chemins pourraient ignorer une ancienne clé de page : une migration doit exiger un diagnostic, jamais simplement faire disparaître les points plantés.

Il faut distinguer **la migration complète de mon A1 initiale**, aujourd'hui déconseillée, de **trois substitutions lexicales**. `shrink → minSize`, l'ajout de `placement` et le déplacement de `Zoom(levels:)` ne sont pas des substitutions : il faut spécifier des comportements, types et valeurs par défaut nouveaux. Aucun remplacement global ne sait les faire correctement.

Inventaire vérifié par recherche dans les fichiers suivis au commit de référence. Chemins relatifs à la racine ; les fichiers personnels non suivis ne sont ni audités ni modifiés. Les noms d'implémentation Rust/JS, attributs HTML et classes CSS peuvent rester stables : changer un mot de l'auteur n'oblige pas à renommer toutes les variables internes.

| Fichier | Adoption de l'A1 complète : travail requis | Seulement les trois changements recommandés en §5 |
|---|---|---|
| `moteur/src/vue.rs` | Noms de blocs et clés, listes autorisées, types, bornes, diagnostics, doctexte et tests ; traitement spécifique de `minSize` et des niveaux déplacés. | `grid → divisions`, `depth → levels`, diagnostics dont la formule de taille ; commentaire et source de test utilisant `above`. Garder les champs Rust et le calcul. |
| `moteur/src/blocs.rs` | Liste `BLOCS`, reconnaissance des nouveaux blocs et tests qui vérifient la couverture du vocabulaire. | Aucun changement de bloc. |
| `moteur/src/regles.rs` | Parcours `pixels → points`, portée du repère, capacité `portals → openPortals`, diagnostics et sources de tests. | Lire `anchor` à la place de `above` ; préserver le contrôle du même site et adapter les assertions de diagnostic. |
| `moteur/src/plat.rs` | Lecture de la collection et du repère, génération des données de placement, diagnostics et tests intégrés. | Lire `anchor`, actualiser messages/tests/commentaires. Garder `data-above` en interne évite de coupler une migration DOM au changement du langage. |
| `moteur/src/web.rs` | Relire la transmission des réglages et les commentaires ; modifier l'interface seulement si types/ordre changent avec la nouvelle politique. | Pas de changement requis si les champs et l'interface internes restent identiques. |
| `moteur/src/mosaique.rs` | Adapter le calcul du minimum si `minSize` est adopté ; actualiser commentaires/tests concernés. | Commentaire `Points(grid: 2, depth: 3)` à actualiser ; aucun changement de subdivision. |
| `moteur/web/page.html` | Nouvelle capacité, placement et minimum de zoom ; maintenir l'accord avec les données du générateur. | Commentaire `above` à actualiser. Conserver le calcul actuel et `dataset.above` si `data-above` reste interne. Si cet attribut est renommé, le producteur Rust et ce consommateur doivent changer ensemble. |
| `moteur/src/lib.rs` | Point de vérification : relancer notamment le test des exemples du guide et des sources incluses ; pas de renommage direct imposé dans ce fichier par les clés A1. | Même exigence de vérification, sans modification automatique. |
| `docs/01-holocode/GUIDE.md` | Toutes les explications et les sources embarquées concernées : elles sont exécutées par les tests du moteur. | Exemples et définitions de `grid`, `depth`, `above`, plus distinction des deux `levels`. |
| `docs/01-holocode/NOMS.md` | Inventaire, correspondances et avertissements. | Trois entrées et avertissement sur les deux comptes de niveaux. |
| `docs/01-holocode/COMPARAISON-WEB.md` | Tables des vues et des sites plantés ; ne pas réécrire les mots du web dans la colonne de comparaison. | Les mentions de paramètres remplacés, notamment `Points(... grid:, depth:)`. |
| `docs/02-gouvernance/adr/ADR-021-reglages-de-vue.md` | Syntaxe et contrat de vue à faire mettre à jour par Claude après décision de Yocthan. | Syntaxe des subdivisions ; aucun statut modifié. |
| `docs/02-gouvernance/adr/ADR-024-disposition.md` | Relire l'objection `Grid`/`Points(grid:)`. | Actualiser cette objection si `divisions` est retenu ; aucun statut modifié. |
| `exemples/boutique-comparee/boutique.holo` | Réglages `Zoom`, `Points`, `Relief`, `Portals`. | `grid` et `depth` dans `Points`, avec leurs commentaires. |
| `exemples/zoom/reduire.holo` | Idem, notamment la réduction : vrai changement de contrat sous A1. | `grid` et `depth` dans `Points`. |
| `exemples/boutique-comparee/README.md` | Texte explicatif du vocabulaire concerné à relire ; pas de changement de la jumelle HTML/CSS/JS pour un simple renommage HoloCode. | Pas de modification requise pour les trois clés si aucune mention n'est ajoutée ; vérifier les liens et l'exemple. |
| `outils/vscode-holocode/syntaxes/holo.tmLanguage.json` | Le motif des paramètres n'accepte pas les majuscules internes : adapter pour camelCase ; ajouter `openPortals` au motif des capacités. | Les trois mots minuscules sont déjà couverts par le motif générique : aucun changement requis. |
| `proposals/GPT5.6/revue-langage-securite-2026-10-03/harness.rs` | Actualiser les attentes de diagnostics et les fixtures chargées, sans affaiblir les assertions corrigées. | L'assertion cherchant `above` doit chercher `anchor` ; nom du test/constante à actualiser pour la lisibilité. |
| `proposals/GPT5.6/revue-langage-securite-2026-10-03/hostiles/03-zoom-max-contourne.holo` | `max`, `points`, `Points`, `after` ; garder une incohérence de bornes réellement atteinte par le validateur. | Inchangé ; son diagnostic ne doit pas changer. |
| `proposals/GPT5.6/revue-langage-securite-2026-10-03/hostiles/04-portals-count-contourne.holo` | `portals`, `Portals`, `count`. | Inchangé. |
| `proposals/GPT5.6/revue-langage-securite-2026-10-03/hostiles/06-above-hors-portee.holo` | `pixels`, `above`, éventuellement placement. | `above → anchor` ; conserver la référence hors du site. Le nom de fichier peut rester historique ; s'il change, adapter `include_str!` dans le harnais. |

Autres frontières à respecter :

- Les quatre hostiles `01-injection-texte.holo`, `02-lien-javascript.holo`, `05-budget-declaratif.holo`, `07-passage-distant.holo` n'emploient pas les clés A1 à remplacer. Ne pas les transformer pour le principe. La sonde Python `proposals/GPT5.6/holocode-v0.1/tests/test_engine_adversarial_review.py` lance le harnais par son manifeste : inchangée si son chemin reste stable.
- Aucun cas actuel de `experiments/conformite-v0.1/cas/` n'emploie ces réglages de vue : pas de fichier existant à convertir pour ces trois noms. Il faudra créer des cas valides/invalides avec résultats attendus et mettre à jour `experiments/conformite-v0.1/README.md`. Leurs futurs noms ne sont pas présentés comme des fichiers existants.
- `moteur/src/holo.rs` lit déjà les identifiants contenant des majuscules internes : camelCase ne demande pas à lui seul de refaire l'analyseur. `moteur/src/styles.rs` conserve ses propriétés CSS. `moteur/src/etat.rs`, `moteur/src/univers.rs`, `moteur/src/navigation.rs`, les shaders et les fichiers `moteur/mondes/` n'ont pas à changer pour ces trois substitutions.
- `moteur/src/bin/holo.rs`, `moteur/outils/serveur.mjs` et `outils/vscode-holocode/extension.js` n'exigent pas de changement de logique pour ces mots. Le CLI et le paquet `moteur/web/pkg/` doivent être reconstruits et servis ensemble : un moteur ancien face aux sources migrées casse la lecture. Les anciennes sources `.holo` distantes demeurent un problème de compatibilité même après mise à jour du dépôt.
- Le rapport de PR 39 et `proposals/GPT5.6/revue-2026-10-03/README.md` sont des preuves historiques : ne pas réécrire rétroactivement leurs propositions comme si elles avaient toujours porté les nouveaux noms. Ajouter une note de suivi/lien si nécessaire. Même règle pour les prompts, transcriptions, captures et entrées anciennes du journal.
- `AGENTS.md`, `docs/06-journal/JOURNAL.md`, `docs/02-gouvernance/DECISIONS.md` et les ADR restent tenus par Claude. Après validation éventuelle : note de migration et état courant ; **aucun statut ne change du seul fait d'un renommage**. `ADR-016`, `ADR-017` et `ADR-020` ne nécessitent pas de révision pour les trois substitutions minuscules. L'A1 camelCase demanderait, elle, une clarification d'ADR-020 par Yocthan.
- Les occurrences génériques « pixels », `maxPoints` ou « relief » dans `moteur/outils/mesures/vue-points.js`, `moteur/web/accueil.html`, le rendu et les README ne sont pas toutes des mots du langage : une recherche textuelle fournit des candidats à relire, pas une liste de remplacements automatiques. Les archives/prototypes plus anciens conservent leur syntaxe propre.

Politique de migration proposée si Yocthan valide : une version cible avec trois substitutions contextualisées, diagnostics guidant depuis l'ancien nom, anciennes et nouvelles clés simultanées refusées. Pour les sites externes non migrés, il faudra décider explicitement d'une période/version de compatibilité ou annoncer la rupture ; ne pas installer des alias silencieux permanents. Rien de cela n'est implémenté ici.

### 5. Si je ne pouvais changer que trois noms

| Rang | Mot actuel | Proposition retenue | Pourquoi avant les autres | Limite reconnue |
|---|---|---|---|---|
| 1 | `Points(grid:)` | `divisions:` | Écarte la confusion avec la disposition `Grid`, déjà présente ; précise une quantité plutôt qu'un système de placement. Changement local, sans nouvelle fonctionnalité. | Expliquer « par axe », avec 4 × 4 = 16, et ne pas remplacer `Portals(layout: grid)` ni `Grid(columns:)`. |
| 2 | `Points(depth:)` | `levels:` | Sépare visuellement le nombre de subdivisions de la profondeur spatiale, importante dans ce projet ; cohérent avec l'usage de niveaux de subdivision. | Accepter et documenter les deux `levels` qualifiés par bloc. Ce n'est pas un changement de `Zoom(levels:)`. |
| 3 | `Point(above:)` dans `pixels:` | `anchor:` | Nomme le rôle de référence sans prétendre offrir un choix de position. Migration limitée du validateur, du rendu et de ma propre sonde hostile. | Plus difficile pour le novice ; collision connue avec l'ancre hypertexte. Préserver exactement le placement actuel, pas inventer `topRight`. |

Le troisième choix est le moins solide : si l'essai débutant montre que `anchor` dégrade la compréhension sans réduire les erreurs de placement, je garderai `above`. Une priorité proposée n'est pas une preuve utilisateur. Je ne dépenserais pas trois changements supplémentaires sur `Points`, `Relief`, `Portals` : leur coût dépasse le gain établi. `tilt` et `fragment` restent discutables, mais je n'ai pas encore fourni de meilleur remplacement à un mot dont le contrat soit suffisamment défini.

### 6. Ce que je garde chez Claude et que ma critique ne mettait pas en valeur

| Mot actuel | Ma proposition | Ce que tu y gagnes | Ce que tu y perds | Ma recommandation finale |
|---|---|---|---|---|
| `name`, `title`, `text` | Identiques | Mots courts, rôles stables, faciles à expliquer. | `name` n'est pas exactement l'attribut HTML `name` ; le contexte doit le dire. | **Garder**. |
| `children` | Identique | Composition imbriquée familière, même idée que les enfants d'un arbre. | Métaphore à apprendre pour un novice. | **Garder**. |
| `to`, `source`, `inside` | Identiques | Destination, provenance et contenu intérieur se distinguent ; plus lisibles que des abréviations inventées. | Ne suffisent pas à deviner les schémas d'URL autorisés. | **Garder**. |
| `seed`, `palette`, `brightness` | Identiques | Termes cohérents avec génération, couleurs et luminosité. | `seed` ne fait pas deviner le déterminisme ; `brightness` est long. | **Garder** : raccourcir n'aiderait pas forcément. |
| `Point(fragments:)` | Identique | Un pluriel qui compte les morceaux ; plus concret que mes termes de rendu. | Voisin de `fragment` au singulier, dont le contrat diffère ; ne pas masquer cette dette. | **Garder**. |
| `Page`, `Text`, `Button`, `Image`, `List`, `World` | Identiques | Noms de choses identifiables, sans suffixes techniques. | `World` doit expliquer sa représentation de page ; `List` précise séparément si elle est ordonnée. | **Garder**. |
| `P`, `A`, `H1` à `H3` | Identiques | Familiarité web et rôles sémantiques. | Un novice ne devine pas les abréviations ; une limite à trois niveaux reste un choix du projet. | **Garder**, sans les vendre comme intuitifs. |
| `Row`, `Column`, `Grid`, `gap` | Identiques | Image spatiale concrète ; utile sur téléphone comme sur grand écran. | Le retour à la ligne et l'adaptation ne se devinent pas dans les noms seuls. | **Garder**. |
| `On`, `rules`, `effect`, `tap`, `enter`, `leave` | Identiques | Relation événement/effet lisible et gestes de navigation courts. | `tap` couvre aussi souris/clavier ; cela doit être enseigné. | **Garder**. |
| `State`, `Prices`, `add`, `set`, `count`, `total` | Identiques | Déclaration et demandes explicites ; vocabulaire utile au panier. | `count` et `total` deviennent réservés avec `Prices` ; un nom clair n'annule pas ce coût. | **Garder**. |
| `sub` | Identique pour l'instant | Court, familier à certains programmeurs. | Moins clair pour un novice que les autres demandes : abréviation à apprendre. | **À discuter**, sans introduire opportunément un quatrième renommage. |
| `speed`, `duration`, `size`, `height`, `color`, `ordered` | Identiques | Grandeurs ou propriétés reconnaissables, unités explicites quand elles sont requises. | `speed` est ici un multiplicateur de vitesse, `size` un seuil dans `Points` : l'aide doit préciser le contrat. | **Garder**. |

## Objections et limites

- Cette proposition optimise surtout la précision contextuelle et la stabilité ; elle ne démontre pas une meilleure compréhension par des non-programmeurs.
- Les collisions de vocabulaire ne se résument pas à une recherche de mots identiques. Mes choix `anchor` et `levels` restent discutables ; leurs collisions sont exposées au même titre que celles de Claude.
- Le vocabulaire est examiné au commit indiqué. Aucun inventaire ne couvre les `.holo` privés, distants ou non suivis par Git.
- Ni le changement de noms ni le succès des sept sondes ne corrigent B-03 ou l'absence de CSP. Le test dont le nom dit `portals_count_ne_borne_pas_les_points_ecrits` compte les points du HTML, pas les portails visibles du carrefour : son succès ne dément pas B-02 corrigé dans le navigateur.

## Expérience ou preuve requise

1. Faire lire à des débutants les paires sans commentaires, ordre alterné et niveau d'anglais relevé ; demander ce que changent les valeurs, pas seulement quel mot plaît. Puis donner un même exemple et mesurer les erreurs de modification. Aucune passation réalisée ici.
2. Pour `divisions`, demander le nombre d'enfants lorsque la valeur vaut 4 ; pour les deux `levels`, demander ce que comptent 0, 1 et 3 ; pour `above`/`anchor`, faire dessiner le placement attendu. Tester spécialement l'erreur « coin du bouton » face au bord de la page.
3. Si les trois noms sont validés, vérifier à données internes égales les anciennes sources et leurs versions migrées : même rendu, même navigation et mêmes limites ; les anciens noms doivent donner le diagnostic de migration prévu.
4. Exécuter le moteur, le harnais hostile, le test Python qui le lance et la conformité. Pour la sonde 06, contrôler que le refus provient toujours du repère hors portée, jamais seulement d'un ancien nom inconnu. Pour la sonde 03, maintenir le refus pour incohérence de zoom. Ne pas affaiblir les assertions pour obtenir du vert.
5. Recompiler CLI et WebAssembly ; essayer la boutique, la réduction, le point planté et son refus hors portée dans Chrome. Ces vérifications de migration sont **requises à l'avenir**, pas annoncées comme faites dans cette PR documentaire.

### Vérifications réellement exécutées pour cette contribution

| Vérification | Résultat et portée |
|---|---|
| Lecture de PR 39, PR 40 et du journal | A1 et correctifs relus ; PR 40 fusionnée ; trois sondes retournées constatées dans le harnais. |
| Recherche des chemins et des consommateurs | `git grep`, `rg`, lecture des validateurs, générateur HTML, interface navigateur, grammaire VS Code et workflow CI ; inventaire de migration ci-dessus. |
| Contrôle documentaire | Six liens locaux résolus ; présence contrôlée des seize noms explicitement demandés ; fichier UTF-8 lisible. Ce contrôle ne prouve pas la justesse des recommandations. |
| `cargo test --manifest-path proposals/GPT5.6/revue-langage-securite-2026-10-03/Cargo.toml` | Première tentative restreinte bloquée par le réseau ; réexécution autorisée sous Windows : **7 réussis, 0 échec, 0 ignoré**, 0 doctest. Aucun test ni fichier hostile modifié. |
| Rendu navigateur / essai débutant | Non exécutés pour cette contre-revue documentaire. |

## Documents à mettre à jour

Si Yocthan valide une migration : guide, inventaire des noms, comparaison web et exemples, puis annotations d'ADR-021/024 et journal par Claude, comme détaillé au §4. Si les noms restent : ce document suffit à consigner le retrait de mes propositions A1 les moins défendables. L'ancienne revue reste disponible comme historique.
