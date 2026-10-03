# Le même web, en 3D : ce que HoloCode reprend de HTML, CSS et JavaScript, et ce qu'il change

- Statut : `PROPOSITION` (Claude, 2026-10-03). Rien ici n'est décidé ; aucun bloc nouveau n'est ajouté au moteur.
- Demande de Yocthan : s'inspirer des éléments de HTML, CSS et JavaScript pour créer ceux de HoloCode, voir comment chacun se représente en 3D (« exactement le même web, maintenant en 3D »), innover, et comparer avec les frameworks JavaScript et les autres langages qui font le même travail, pour bâtir quelque chose de solide.
- Décisions sur lesquelles cette proposition s'appuie : `ADR-007` (une description, deux vues), `ADR-015` (pas de code libre), `ADR-017` (styles), `ADR-018` (vue en profondeur), `ADR-020` (texte et titres).

## 1. L'idée centrale : le plan de la page devient la profondeur

`ADR-018` hésite entre deux options : A (seuls les points ont de la profondeur, le reste sur un panneau) et B (chaque bloc devient une boule). A est lisible mais plat ; B est spectaculaire mais illisible.

Proposition, **option C : c'est le plan du document qui donne la profondeur.**

- Une page a déjà une structure en arbre : un `H1`, sous lui des `H2`, sous eux des `H3`, et sous chaque titre son contenu. `ADR-020` interdit de sauter un niveau : cet arbre est donc toujours propre.
- En profondeur, **chaque titre avec son contenu devient un point**. De loin on voit le `H1` comme une enseigne. En zoomant, le point se morcelle en ses `H2`. En entrant dans un `H2`, on lit son contenu sur un panneau, et l'on voit ses `H3` comme des points.
- C'est le Big Bang appliqué à un texte : un point qui se morcelle, et le zoom qui révèle le détail. C'est aussi l'idée du pixel de Yocthan : on ne montre un détail que lorsqu'il occupe assez de place à l'écran pour être lu.

Ce que cela donne : l'auteur écrit une page ordinaire, sans penser à la 3D, et obtient un lieu visitable dont la forme vient du sens. Aucun outil web actuel ne fait cela : dans tous, la scène 3D s'écrit à part.

## 2. Les éléments de HTML : ce qu'ils deviennent

Trois familles suffisent à ranger tout HTML : les **lieux** (ce qui contient), les **objets** (ce qu'on lit ou regarde), les **passages** (ce qui mène ailleurs), plus les **commandes** (ce qu'on actionne).

| HTML | HoloCode | À plat | En profondeur | État |
|---|---|---|---|---|
| `<html>`, `<body>`, `<main>` | `Page` | La page | Un lieu | existe |
| `<h1>` à `<h3>` | `H1` à `H3` | Titres | Enseignes ; chaque titre ouvre un point (option C) | existe, 3D proposée |
| `<p>` | `P`, ou une phrase nue | Paragraphe | Texte sur un panneau, lisible de près | existe |
| `<span>`, texte nu | `Text` | Texte sans rôle | Étiquette posée sur un objet | existe |
| `<img>` | `Image` | Image | Tableau accroché | existe |
| `<ul>`, `<ol>`, `<li>` | `List` | Liste | Étagère : les éléments alignés dans l'espace | existe |
| `<button>` | `Button` | Bouton | Objet qu'on presse | existe |
| `<a href>` | `Link` | Lien | **Une porte : un point dont l'intérieur est un autre fichier** | proposé |
| `<section>`, `<article>` | (aucun bloc : le titre suffit) | | Le point du titre | proposé |
| `<nav>` | `Nav` | Menu | Panneaux indicateurs, toujours visibles | proposé |
| `<header>`, `<footer>` | (réglés par le thème) | | | à discuter |
| `<div>` | **rien** | | | refusé (`ADR-009`) |
| `<video>`, `<audio>` | `Video`, `Audio` | Lecteur | Écran ; son situé dans l'espace, plus fort quand on s'approche | proposé |
| `<canvas>`, WebGL | `Point`, `World` | Point lumineux | Monde | existe |
| `<model>` (proposé par Apple) | `Model` | Image de l'objet | L'objet lui-même, en volume | proposé, plus tard |
| `<table>` | `Table` | Tableau | Mur quadrillé | proposé, plus tard |
| `<form>` | `Form` | Formulaire | Guichet | proposé |
| `<input>`, `<textarea>` | `Input` | Champ | Champ sur un panneau (on n'écrit pas dans le vide) | proposé |
| `<select>`, cases, boutons radio | `Choice` | Liste de choix | Les choix étalés devant soi | proposé |
| `<iframe>` | (c'est `Link` : on entre) | | | proposé |
| `<script>` | **rien** dans un bloc ; `module`, `bridge js` | | | existe (`ADR-013`, `ADR-015`) |

Deux remarques.

**Le lien est un point.** Dans le web, tout repose sur le lien. En profondeur, un lien devient une porte : `Link(to: "atelier.holo")` s'affiche à plat comme un lien, et en profondeur comme un point dans lequel on entre, dont l'intérieur est l'autre fichier. Le web entier devient alors ce que décrit la vision : des points qui contiennent des mondes, reliés entre eux. Et l'adresse d'un lieu est le chemin des points traversés, ce que le moteur garde déjà (`Origin › 7 › 2`) : on peut partager un endroit précis, comme une URL.

**La disposition.** HTML et CSS mêlent disposition et apparence ; `ADR-017` les sépare. Proposition : des blocs de disposition repris de Flutter, que Yocthan connaît, chacun avec un sens dans l'espace.

| Bloc | À plat | En profondeur |
|---|---|---|
| `Row` | Côte à côte | Alignés de gauche à droite |
| `Column` | L'un sous l'autre | Empilés |
| `Grid` | Grille | Mur |
| `Ring` | Carrousel | En cercle autour du visiteur |

L'ordre d'écriture reste l'ordre de lecture à plat, et devient l'ordre de visite en profondeur.

## 3. CSS : les mêmes styles, avec la matière en plus

Les quinze réglages actuels valent dans les deux vues (une couleur reste une couleur ; un fond devient la matière d'un panneau). À ajouter pour la profondeur, dans les mêmes styles (`ADR-017`, règle 1) :

| Réglage proposé | Sens |
|---|---|
| `glow` | Lumière émise, de 0 à 1 |
| `texture` | Une image plaquée sur l'objet |
| `shape` | `panel`, `sphere`, `box` : la forme que prend un bloc en profondeur |

Ce qu'on ne reprend pas de CSS, et pourquoi : la cascade et la spécificité (résultat imprévisible) ; les sélecteurs composés ; `!important` ; la disposition par `float` ou `position` ; les requêtes d'écran écrites à la main (`@media`), que le moteur doit gérer seul.

## 4. JavaScript : ce qu'on en fait sans écrire de code

JavaScript sert à huit choses dans une page. Voici comment chacune s'écrirait.

| Besoin | En JavaScript | En HoloCode | État |
|---|---|---|---|
| Réagir à un geste | `addEventListener("click", …)` | `On(Open.tap, effect: …)` | existe |
| Survol, approche | `mouseover`, `IntersectionObserver` | Un seul signal, `near` : survol à plat, approche en profondeur | proposé |
| Garder une valeur | variables, `useState` | `State(name: Cart, count: 0)`, que seul le monde modifie | proposé |
| Afficher une valeur | manipuler le DOM | `Text("{Cart.count} articles")` : le texte suit la valeur | proposé |
| Afficher sous condition, répéter | `if`, `map` | `when:` sur un bloc ; `List(each: Products, …)` | proposé |
| Chercher des données | `fetch` | `Source(name: Products, from: "produits.json")`, déclarée et comptée dans le budget | proposé |
| Durées, animations | `setTimeout`, `requestAnimationFrame` | Une durée sur une règle (`after: 3s`) ; des transitions dans les styles | proposé |
| Changer de page, revenir | routeur, `history` | `enter`, `leave` ; le chemin des points est l'historique | existe |
| Calculer vraiment | du code libre | Fonctions pures, ou module WebAssembly enfermé | décidé (`ADR-013`, `ADR-015`) |

Le principe ne change pas : un bloc ne contient pas de code. Les changements passent par des règles que le moteur arbitre, et tout le reste est une déclaration que le vérificateur contrôle avant d'exécuter.

**Limite, dite franchement.** Ce modèle couvre une boutique, un blog, un portfolio, un formulaire. Il ne couvre pas un tableur ni un jeu d'action : pour cela il faudra le deuxième étage (les modules), qui n'existe pas encore. Tant qu'il n'existe pas, HoloCode fait moins de choses que JavaScript.

## 5. Les concurrents : ce qu'ils font, ce qu'on leur prend, où ils nous battent

### Les frameworks du web à plat

| Outil | Son idée | Ce qu'on reprend | Son défaut, qu'on évite |
|---|---|---|---|
| React | Des composants ; l'écran est une fonction de l'état | L'écran découle de l'état, on ne le modifie pas à la main | Beaucoup de code d'accompagnement, des pièges (`useEffect`), des pages lourdes |
| Vue, Solid, Svelte 5 | Des « signaux » : une valeur change, seul ce qui en dépend se met à jour | **Exactement notre modèle** : signal, règle, mise à jour ciblée | Restent du JavaScript libre : rien n'empêche le désordre |
| Svelte | Un compilateur : presque rien n'est envoyé au navigateur | Tout vérifier et préparer avant, envoyer peu | |
| Angular | Tout fourni, très cadré | Un seul cadre officiel | Lourd, long à apprendre |
| htmx | Pas de JavaScript à écrire : des attributs dans le HTML | La preuve que beaucoup de sites n'ont pas besoin de code | Dépend d'un serveur pour tout |
| Elm | Aucun code libre ne touche l'état : tout passe par une fonction centrale ; zéro erreur à l'exécution | **Le plus proche de notre arbitre** (`ADR-015`) | Resté confidentiel : trop strict, trop peu d'ouverture vers l'existant. C'est l'avertissement le plus sérieux pour nous |
| Flutter (web) | Des blocs imbriqués, dessinés par son propre moteur | L'écriture en blocs, la disposition par `Row` et `Column` | Sur le web, il dessine tout lui-même : texte non sélectionnable, mauvais référencement, accessibilité faible. Notre vue à plat fabrique du vrai HTML pour ne pas tomber dans ce piège |
| TypeScript | Des types vérifiés avant d'exécuter | Vérifier avant d'exécuter | |
| Blazor, Leptos, Yew | Un autre langage (C#, Rust) compilé en WebAssembly | Notre moteur fait de même | Téléchargement lourd |

### Le web en 3D

| Outil | Son idée | Ce qu'on reprend | Ce qui lui manque |
|---|---|---|---|
| three.js, Babylon.js | Des bibliothèques pour dessiner en 3D | Rien pour l'auteur : c'est le niveau de notre moteur | Tout s'écrit en JavaScript ; rien pour un non-programmeur |
| A-Frame | **Des balises HTML pour la 3D** (`<a-box>`) | La 3D déclarée, sans code | Seulement la 3D : la page ordinaire et la scène restent deux mondes séparés |
| React Three Fiber | Des composants React pour three.js | | Même séparation, plus toute la complexité de React |
| `<model-viewer>`, `<model>` | Un objet 3D posé dans une page | Notre futur bloc `Model` | Un objet isolé, pas un lieu |
| visionOS (Apple) | Des fenêtres plates posées dans l'espace, et des volumes | **Confirme l'option A** : Apple garde le texte sur des panneaux | Fermé, lié à un casque |
| VRML, X3D (années 1990) | Un langage déclaratif pour un web en 3D | | **A échoué** : trop lourd pour les machines de l'époque, pas de contenu, et aucune raison de préférer la 3D à une page. La leçon : la 3D doit venir en plus d'un web qui marche déjà, jamais à sa place |
| Roblox, Decentraland, Horizon | Des mondes fermés, chacun chez soi | | Pas un web : pas de lien d'un monde à l'autre, pas de fichier qu'on possède |

### Ce que cette comparaison dit de HoloCode

**Ce qui est vraiment à nous**, et que personne ne propose ensemble :

1. une seule description pour la page et pour le lieu ;
2. la profondeur tirée du plan du document, sans que l'auteur pense à la 3D ;
3. un monde entier dans une graine, donc presque rien à télécharger ;
4. tout est vérifié avant d'exécuter, et rien ne s'exécute hors de l'arbitre.

**Ce qui n'est pas à nous** : les blocs imbriqués (Flutter), les signaux (Solid, Vue), l'arbitre (Elm), la 3D déclarée (A-Frame, VRML). Aucune brique n'est neuve ; c'est l'assemblage qui l'est.

**Où nous sommes battus aujourd'hui**, sans détour :

- l'écosystème : des millions de paquets et de développeurs d'un côté, zéro de l'autre ;
- l'expressivité : sans le deuxième étage, on ne peut pas tout faire ;
- la preuve : le moteur n'a pas encore été mesuré sur un téléphone ;
- le risque d'Elm et de VRML : être juste sur le fond et ne pas être adopté.

**Ce qui nous protège de ces risques** : la vue à plat fabrique du vrai HTML (un site HoloCode reste un site normal, référencé, accessible) ; les ponts vers JavaScript et CSS (`ADR-012`) laissent une porte vers l'existant ; et l'on ne demande à personne de choisir la 3D : elle vient en plus.

## 6. Ce que je propose de faire, dans l'ordre

1. **Montrer l'option C** sur la boutique : ses titres deviennent des points, à comparer avec A et B.
2. **`Link`** : le lien qui est une porte. C'est ce qui fait d'un fichier isolé un web.
3. **`Row`, `Column`, `Grid`** : sans disposition, on ne fait pas de vrai site.
4. **`State`, `{valeur}` dans un texte, `when:`** : le minimum pour un panier.
5. **`Input`, `Form`, `Source`** : les données.
6. Puis `Nav`, `Video`, `Audio`, `Model`, `Table`, `Ring`, et les réglages `glow`, `texture`, `shape`.

À chaque ajout : la comparaison d'abord, et la vérification qu'on ne répète pas un défaut de HTML, de CSS ou de JavaScript.

## 7. Questions pour Yocthan

1. L'option C (le plan de la page devient la profondeur) : on la construit pour la voir, à côté de A ?
2. Le lien qui est une porte (`Link`) : d'accord sur le principe ?
3. L'ordre de la partie 6 te convient-il ?
