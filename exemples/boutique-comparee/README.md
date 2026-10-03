# La même boutique, écrite deux fois

Demandé par Yocthan le 2026-10-03 : un même exemple écrit en HoloCode et dans le trio HTML, CSS et JavaScript, qui emploie tout le vocabulaire du langage, de la façon dont on s'en servirait pour un métavers.

| | Fichiers | Lignes utiles (sans les lignes vides ni les commentaires) |
|---|---|---|
| HoloCode | [`boutique.holo`](boutique.holo) | 62 |
| Web | [`web/index.html`](web/index.html), [`web/style.css`](web/style.css), [`web/script.js`](web/script.js) | 178 (36 + 61 + 81) |

Les deux décrivent la même chose : une page de boutique (titres, paragraphes, image, liste, bouton), un point lumineux, et le monde dans lequel on entre par ce point, avec ses six points nés du morcellement.

## Où en est chaque version

- **La version web fonctionne aujourd'hui** : ouvrir `web/index.html` dans un navigateur. Le bouton fait entrer dans l'atelier, « Back to the shop » en fait sortir. Les graines des six points sont calculées comme dans le moteur (vérifié : mêmes valeurs que `moteur/src/graine.rs`).
- **La version HoloCode est lue et vérifiée par le moteur** (un test la relit à chaque changement et contrôle qu'aucun mot du langage n'y manque), **mais elle n'est pas encore affichée** : le moteur ne sait afficher que le Big Bang. L'affichage de cette boutique est l'étape suivante.

La comparaison porte donc sur l'écriture, pas encore sur le résultat à l'écran.

## Terme par terme

| HoloCode | HTML, CSS, JavaScript | Remarque |
|---|---|---|
| `Page(title: "…")` | `<html>`, `<head>`, `<title>`, `<body>`, `<main>` | |
| `H1`, `H2`, `H3` | `<h1>`, `<h2>`, `<h3>` | HoloCode refuse de sauter un niveau |
| `"Une phrase"` ou `P("…")` | `<p>…</p>` | |
| `Text("…")` | `<span>…</span>` | Du texte sans rôle |
| `**gras**` dans un texte | `<strong>gras</strong>` | Markdown |
| `Image(source: …, weight: …)` | `<img src="…">` | Le poids n'existe pas en HTML |
| `List(children: […])` | `<ul>` et `<li>` | |
| `Button(name: Open, text: "…")` | `<button id="open">…</button>` | |
| `P.card(…)` | `<p class="card">` | Une classe inconnue est refusée en HoloCode |
| `.card { … }`, `P { … }` | Identique | Mêmes noms de réglages |
| `Page { … }` (le thème) | `body { … }` et `.page { … }` | |
| `World { … }` | `.world { … }` | |
| `Point(name, seed, brightness, fragments, color, palette)` | Un `<div>`, 13 lignes de CSS pour la boule, 30 lignes de JavaScript pour les graines et les points enfants | Le mot n'existe pas dans le web |
| `budget: 500KB` et `weight: 1KB` | Une fonction `checkBudget` écrite à la main | Rien ne l'impose dans le web |
| `inside: World(children: […])` | Une `<section hidden>` déjà chargée dans la page | |
| `On(Open.tap, effect: Workshop.enter)` | `addEventListener("click", enter)` et la fonction `enter` | |
| `On(Back.tap, effect: Workshop.leave)` | `addEventListener("click", leave)` et la fonction `leave` | |
| `import`, `module`, `bridge js`, `bridge css` | `<link>`, `<script>` | En commentaire dans `boutique.holo` : lus par le moteur, pas encore appliqués |

## Ce que la comparaison montre, sans embellir

- **Pour la page seule, l'écart est faible.** Titres, paragraphes et styles s'écrivent presque pareil : c'est voulu, les styles reprennent l'écriture du CSS (`ADR-017`).
- **L'écart vient du point et du monde.** Le web n'a pas de mot pour « un point dans lequel on entre » : il faut le fabriquer avec un `div`, un dégradé et du JavaScript. C'est là que se trouvent les deux tiers des lignes de la version web.
- **La version web triche sur un point** : le monde de l'atelier est chargé dès le départ et simplement caché, et ses points enfants sont posés sur un cercle. Le vrai moteur ne calcule un monde que lorsqu'on y entre, et place les points sur une sphère où l'on zoome.
- **Ce que le web tolère et que HoloCode refuse** : dans la version web, rien n'empêche d'écrire `class="carte"` sans style correspondant, d'oublier un `;`, ou de mettre n'importe quel code dans le clic du bouton.

## Termes du langage qui n'ont pas encore d'emploi

Ils existent dans la grammaire mais aucun bloc ne s'en sert aujourd'hui, donc ils ne figurent pas dans l'exemple :

- les unités de longueur et de durée (`mm`, `cm`, `m`, `km`, `ms`, `s`, `min`, `h`) et les tailles `B`, `MB`, `GB` (seul `KB` est employé) ;
- `true` et `false` ;
- `seed: auto`, qui n'est permis qu'au moment de la création, avant d'être remplacé par un nombre.

## Choix faits pour cet exemple

- `List(children: […])` : le bloc `List` existait dans le vocabulaire sans exemple ; il prend ses éléments dans `children`, comme les autres blocs.
- Les règles du monde intérieur sont écrites dans ce monde (`World(rules: […])`).
