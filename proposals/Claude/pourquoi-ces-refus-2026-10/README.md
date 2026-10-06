# Pourquoi HoloCode refuse certains éléments du web

## Contribution

- Auteur : Claude, à la demande de Yocthan, le 2026-10-06.
- Demande : « Dis-moi pourquoi tu as refusé certains éléments, écris-le explicitement sur GitHub, que Codex et Gemini puissent le voir et avoir un aperçu d'ensemble du pourquoi, du comment, de ta réflexion. » Yocthan décidera ensuite, après leurs avis, s'il faut admettre tout ou partie de ce qui est refusé.
- Statut proposé : **EXPLORATION**. Aucune décision n'est changée par ce document.
- Le grand tableau des 129 éléments : [`docs/01-holocode/TABLEAU-WEB.md`](../../../docs/01-holocode/TABLEAU-WEB.md).

## Ce que veut dire « refusé exprès »

Un élément **manque** quand personne ne l'a encore construit. Il est **refusé exprès** quand une décision dit qu'il ne doit pas exister, parce qu'il cause un défaut connu du web. Huit éléments du tableau sont dans ce cas, ou presque.

**Qui a décidé.** Six des huit refus viennent de fiches de décision **acceptées par Yocthan**, sur proposition ou après discussion avec Claude. Un seul (`section`, `article`) est seulement un jugement de Claude, écrit dans la comparaison sans fiche de décision. Un autre (`h4` à `h6`) n'est pas un refus définitif : sa fiche dit « on en ajoutera si un vrai besoin apparaît ». Ce document dit pour chacun qui a décidé, et où.

**La raison commune.** HoloCode veut trois choses que le web n'a pas réussi à tenir ensemble :

1. **Une seule façon de faire chaque chose**, pour qu'on puisse réapprendre le langage en une heure après un mois d'absence (`ADR-009`).
2. **Aucun code caché** : le moteur doit pouvoir lire toutes les règles, les vérifier avant d'afficher, et laisser des inconnus partager un monde sans danger (`ADR-015`).
3. **La même description vue à plat et en profondeur** : un monde ne peut pas se construire sur une page dont la mise en forme est faite à la main, pixel par pixel.

Chaque refus ci-dessous protège l'une de ces trois choses. Chacun a aussi un coût, dit sans détour.

## Les huit refus, un par un

### 1. `div` : la boîte sans signification

| | |
|---|---|
| **Sur le web** | `<div>` est une boîte qui ne dit pas ce qu'elle contient. On en empile des dizaines pour placer les choses. |
| **En HoloCode** | Chaque bloc dit ce qu'il est : `P`, `H1`, `Button`, `List`, `Row`, `Column`, `Grid`. Pour du texte sans rôle : `Text`. |
| **Pourquoi** | Une page en `div` est illisible pour un lecteur d'écran, pour un moteur de recherche, pour une IA, et pour la vue en profondeur : le moteur ne sait pas ce qui est un titre, un bouton, une liste, donc ne sait pas quoi mettre en grand dans un monde. C'est aussi ce que Yocthan trouvait le plus pénible en HTML. |
| **Exemple** | Web : `<div class="title">Ma boutique</div>`. HoloCode : `H1("Ma boutique")`. Le moteur sait que c'est le titre principal. |
| **Qui a décidé** | Yocthan, le 2026-09-21 (`ADR-009`, ACCEPTÉ) : « Pas de `div`. » |
| **Ce que cela coûte** | Quand aucun bloc ne convient, on ne peut pas « bricoler » une boîte. Il faut attendre un bloc nouveau. |
| **Avis de Claude aujourd'hui** | **Garder le refus.** C'est le refus le mieux fondé. |

### 2. `section`, `article`, `aside` : les parties de page

| | |
|---|---|
| **Sur le web** | Des balises qui découpent une page en parties ; un lecteur d'écran s'en sert pour sauter d'une partie à l'autre. |
| **En HoloCode** | Les titres `H1` à `H3` donnent le plan de la page. |
| **Pourquoi** | Éviter deux façons de dire la même chose : le plan est déjà dans les titres. |
| **Qui a décidé** | **Personne par une fiche.** C'est le jugement de Claude, écrit dans `COMPARAISON-WEB.md`. |
| **Ce que cela coûte** | Les **repères** pour aveugles manquent : un lecteur d'écran ne peut pas aller directement au menu, au contenu principal, au pied de page. |
| **Avis de Claude aujourd'hui** | **Revoir ce refus.** Je pense m'être trompé à moitié : `section` et `article` sont bien redondants avec les titres, mais `nav`, `header`, `footer` et `aside` sont des repères que les titres ne remplacent pas. Proposition à discuter : pas de nouveaux blocs, mais un rôle donné aux morceaux existants, par exemple `Part(name: Menu, role: nav, …)`. |

### 3. `h4`, `h5`, `h6` : les titres profonds

| | |
|---|---|
| **Sur le web** | Six niveaux de titres. |
| **En HoloCode** | Trois : `H1`, `H2`, `H3`. Le vérificateur refuse un titre qui saute un niveau. |
| **Pourquoi** | Sur le web, on choisit souvent `h4` « parce que c'est plus petit », ce qui casse le plan de la page. Les niveaux 4 à 6 sont presque jamais utilisés correctement. Ici, le numéro dit la place dans le plan, jamais la taille. |
| **Qui a décidé** | Yocthan, le 2026-10-03 (`ADR-020`, ACCEPTÉ) : « Les titres s'arrêtent à `H3`. **On en ajoutera si un vrai besoin apparaît.** » |
| **Ce que cela coûte** | Un document très long (une documentation technique, un mode d'emploi) manque de niveaux. |
| **Avis de Claude aujourd'hui** | **Pas un vrai refus : une attente.** Le tableau l'a classé « refusé exprès » ; il aurait dû dire « pas encore ». Ajouter `H4` le jour où un vrai document en a besoin. |

### 4. `script` : du code dans la page

| | |
|---|---|
| **Sur le web** | `<script>` permet d'écrire n'importe quel programme dans la page. |
| **En HoloCode** | Aucun code libre. Ce qui arrive s'écrit en **règles** visibles : `On(Add.tap, effect: cart.add(1))`. Le calcul viendra par des fonctions pures enfermées (`ADR-013`). |
| **Pourquoi** | Trois raisons. **La sécurité** : dans un métavers, on entre dans les mondes d'inconnus ; un script peut voler, espionner, tromper. **La vérification** : le moteur lit toutes les règles et refuse une faute avant d'afficher ; il ne peut pas lire un programme libre. **Le jeu à plusieurs** : deux téléphones qui appliquent les mêmes règles arrivent au même résultat ; avec du code libre, rien ne le garantit. |
| **Exemple** | Web : `button.onclick = () => { cart++; render(); }`, à recopier et à tenir à jour. HoloCode : `On(Add.tap, effect: cart.add(1))` ; le moteur fait le reste. |
| **Qui a décidé** | Yocthan (`ADR-015`, ACCEPTÉ) : « Le code libre caché dans un bloc : interdit. » |
| **Ce que cela coûte** | Tout ce que les règles ne savent pas encore dire est impossible : calculs libres, listes, effets inédits. Le film de motion design l'a montré : 520 particules physiques, impossibles en HoloCode aujourd'hui. |
| **Avis de Claude aujourd'hui** | **Garder le refus**, mais **rattraper son coût** : il faut des règles plus riches (listes, calcul) et, plus tard, des modules enfermés. Sans eux, le refus devient un mur. |

### 5. Modifier la page à la main (le DOM)

| | |
|---|---|
| **Sur le web** | JavaScript peut changer n'importe quel élément de la page, de n'importe où : `element.style.color = "red"`. |
| **En HoloCode** | Seul le moteur change la page, à partir des valeurs (`State`) et des règles. |
| **Pourquoi** | C'est la cause du « code spaghetti » du web : on ne sait plus qui a changé quoi, ni quand. Ici, chaque changement passe par l'arbitre, qui le borne et peut l'expliquer. C'est aussi ce qui permet de voir la même page à plat et en profondeur : elle est toujours décrite, jamais bricolée. |
| **Qui a décidé** | Yocthan (`ADR-015`, ACCEPTÉ), et la règle 7 de `ADR-017` : « L'apparence ne change en cours de route que par une règle du monde. » |
| **Ce que cela coûte** | Un effet que les règles ne prévoient pas est impossible. |
| **Avis de Claude aujourd'hui** | **Garder le refus.** |

### 6. `display`, `position`, `float`, `z-index` dans un style

| | |
|---|---|
| **Sur le web** | Le CSS place les éléments à la main : en ligne, en absolu, flottant, l'un par-dessus l'autre. |
| **En HoloCode** | La disposition vient des **blocs** : `Row`, `Column`, `Grid`, `Board`. Un style ne règle que l'apparence (couleur, taille, bordure). |
| **Pourquoi** | En CSS, la disposition et l'apparence sont mêlées : changer une couleur peut casser une mise en page, et un élément en `position: absolute` sort du flux, chevauche le reste, et devient illisible sur un petit écran. Ici, on voit la disposition en lisant les blocs. Et le même placement doit avoir un sens dans un monde en profondeur, où « 20 pixels du bord » ne veut rien dire. |
| **Exemple** | Web : `.cartes { display: flex; gap: 16px }` dans un fichier, `<div class="cartes">` dans un autre. HoloCode : `Row(gap: 16px, children: [ … ])`, à un seul endroit. |
| **Qui a décidé** | Yocthan, le 2026-10-03 (`ADR-017`, règle 3, ACCEPTÉ). |
| **Ce que cela coûte** | Pas de superposition libre (un badge sur le coin d'une image), pas d'élément collé en haut de l'écran pendant qu'on défile. |
| **Avis de Claude aujourd'hui** | **Garder le refus dans les styles**, mais **ajouter des blocs** pour les vrais besoins : un badge posé sur une image, une barre qui reste en haut. |

### 7. Sélecteurs composés, cascade, `!important`

| | |
|---|---|
| **Sur le web** | `.menu > li:first-child a:hover` ; l'ordre des règles et leur « spécificité » décident qui gagne ; `!important` force. |
| **En HoloCode** | Deux façons de viser seulement : par type de bloc (`P { }`) et par nom (`.carte { }`). Un bloc porte un seul nom de style. Le plus précis gagne, toujours dans le même ordre. |
| **Pourquoi** | La cascade est le défaut le plus coûteux du CSS : un style ajouté ici casse une page là-bas, et personne ne sait pourquoi. `!important` est le pansement qu'on met dessus, puis un autre `!important` par-dessus. Ici, on sait toujours d'où vient une couleur. |
| **Qui a décidé** | Yocthan, le 2026-10-03 (`ADR-017`, règle 2, ACCEPTÉ) : « Pas de sélecteur composé. » |
| **Ce que cela coûte** | On ne peut pas dire « le premier élément d'une liste » ou « les liens du menu » sans leur donner un nom. Et le survol (`:hover`), qui est aussi un sélecteur, **n'existe pas encore** : ce manque-là doit être comblé autrement (un état du bloc, pas un sélecteur). |
| **Avis de Claude aujourd'hui** | **Garder le refus**, et ajouter les **états** (survol, focus, appui) par une écriture simple, sans rouvrir la cascade. |

### 8. `requestAnimationFrame` : sans objet

Ce n'est pas un refus : c'est le moteur qui dessine image par image. L'auteur n'a jamais à le faire. Le tableau le classe « sans objet ».

## Ce que ce document ne refuse pas

Tout le reste du tableau qui est marqué **Non** n'est pas refusé : il **manque**. Les tableaux, la vidéo, l'envoi d'un formulaire, les listes, le survol sont tous à ajouter. Le détail et l'ordre proposé sont dans [`TABLEAU-WEB.md`](../../../docs/01-holocode/TABLEAU-WEB.md).

## Ce que Claude propose de changer, en résumé

| Refus | Proposition |
|---|---|
| `div` | Garder |
| `section`, `article` | Garder pour ces deux-là |
| `nav`, `header`, `footer`, `aside` | **Admettre**, sous forme de rôle donné à un morceau, pour les lecteurs d'écran |
| `h4` à `h6` | **Admettre `H4`** le jour d'un vrai besoin ; ce n'était pas un refus |
| `script` | Garder ; rattraper son coût par des règles plus riches et des modules enfermés |
| Modifier la page à la main | Garder |
| `display`, `position`… dans un style | Garder ; ajouter des blocs pour le badge et la barre fixe |
| Sélecteurs composés, cascade | Garder ; ajouter les états (survol, focus, appui) autrement |

## Questions posées à Codex et à Gemini

1. Parmi ces huit refus, lesquels sont **justes**, lesquels sont **de trop**, et pourquoi ? Donnez un exemple concret de site qu'un refus rendrait impossible.
2. Les repères pour lecteurs d'écran (`nav`, `header`, `footer`, `main`, `aside`) : faut-il des blocs, un rôle sur un morceau, ou autre chose ?
3. Interdire `script` dans un métavers ouvert aux inconnus : d'autres langages ou plateformes l'ont-ils fait (Roblox, Second Life, Decentraland, les pages AMP, les e-mails HTML) ? Qu'ont-ils mis à la place, et qu'en ont pensé leurs créateurs ?
4. Comment ajouter le **survol** et les **états** d'un bouton sans rouvrir la cascade du CSS ?
5. Y a-t-il un refus **manquant** : un défaut du web que HoloCode répète encore sans le savoir ?
