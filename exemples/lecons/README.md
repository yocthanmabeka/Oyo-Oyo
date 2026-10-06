# Les leçons

Une leçon par notion. Chaque leçon est un petit fichier `.holo` : il explique **une seule chose** (les commentaires du haut), la montre (le code), et dit quoi essayer.

Pour les suivre : ouvre `http://localhost:8080/exemples/lecons/01-page.holo` dans Chrome, et le même fichier dans VS Code. En bas de chaque page, un lien mène à la leçon suivante.

La référence complète reste le [guide](../../docs/01-holocode/GUIDE.md).

| N° | Leçon | Les mots qu'on y apprend |
|---|---|---|
| 1 | [Une page](01-page.holo) | `Page`, `title`, `children`, `H1` |
| 2 | [Le texte](02-texte.holo) | `H2`, `H3`, `P`, `Text`, gras, italique, code, trois guillemets |
| 3 | [Une image, une liste, un lien](03-image-liste-lien.holo) | `Image`, `alt`, `List`, `ordered`, `A`, `to` |
| 4 | [Un trait, une citation, du texte tel quel](04-trait-citation-code.holo) | `Hr`, `Quote`, `by`, `Code` |
| 5 | [Les styles](05-styles.holo) | `P { }`, `.carte { }`, `P.carte(...)` |
| 6 | [Ranger : Row, Column, Grid](06-disposition.holo) | `Row`, `Column`, `Grid`, `gap`, `align`, `columns` |
| 7 | [Un point, et le monde qu'il contient](07-point-et-monde.holo) | `Point`, `seed`, `World`, `inside`, `Button`, `On`, `tap`, `enter`, `leave` |
| 8 | [Passer dans un autre fichier](08-passage.holo) | `inside: "fichier.holo"` |
| 9 | [Le zoom, et les pixels qui deviennent des points](09-zoom-et-points.holo) | `Zoom`, `Points`, `after` |
| 10 | [Tourner la page](10-relief-et-rotation.holo) | `Relief`, `tilt`, `height` |
| 11 | [Un site planté dans un pixel](11-pixels.holo) | `pixels`, `above` |
| 12 | [Le carrefour et ses portails](12-carrefour.holo) | `Portals`, `layout`, `portals` |
| 13 | [Ce que la page retient](13-valeurs.holo) | `State`, `{compte}`, `add`, `sub`, `set`, les crochets |
| 14 | [Des prix, un nombre d'articles, un total](14-prix.holo) | `Prices`, `{count}`, `{total}` |
| 15 | [Montrer selon une valeur](15-conditions.holo) | `If`, `is`, `not`, `over`, `under` |
| 16 | [Écrire dans un champ, cocher une case](16-saisie.holo) | `Input`, `Checkbox`, `value`, `label`, `max`, les textes |
| 17 | [Garder une valeur d'une visite à l'autre](17-garder.holo) | `keep` |
| 18 | [Le temps](18-temps.holo) | `Every` |
| 19 | [Le hasard](19-hasard.holo) | `random` |
| 20 | [Un plateau, où l'on place les choses](20-plateau.holo) | `Board`, `x`, `y` |
| 21 | [Le clavier](21-clavier.holo) | `Key` |
| 22 | [Une règle qui guette](22-guetter.holo) | `When` |
| 23 | [La rencontre de deux objets](23-rencontre.holo) | `When(…, meets:)`, `within` |
| 24 | [Faire glisser](24-glisser.holo) | `drag` |
| 25 | [Un morceau commun à plusieurs pages](25-imports.holo) | `import`, `Part`, `Use` |
| 26 | [Un point seul : le Big Bang](26-point-seul.holo) | `Point` à la racine, `fragments`, `brightness` |
| 27 | [Des données venues du serveur](27-donnees.holo) | `data`, `Data`, `from`, `every` |
| 28 | [Un son](28-son.holo) | `Sound`, `play` |
| 29 | [Comparer deux valeurs : le meilleur score](29-comparer-deux-valeurs.holo) | `over: record`, `record.set(score)` |
| 30 | [Des formes](30-formes.holo) | `Shape`, `form`, `size` |
| 31 | [Des règles sous condition](31-regles-sous-condition.holo) | `If(…, rules: [ … ])` |
| 32 | [Faire entrer un bloc](32-entrer.holo) | `Enter` |
| 33 | [Un mouvement en boucle](33-boucle.holo) | `Loop` |
| 34 | [Des scènes qui s'enchaînent](34-scenes.holo) | `Scenes` |
| 35 | [Les repères de la page](35-reperes.holo) | `Header, Nav, Main, Footer` |
| 36 | [Des titres plus profonds](36-titres-profonds.holo) | `H4, H5, H6` |
| 37 | [Le survol, le focus, l'appui](37-survol.holo) | `hover:, focus:, active:` |
| 38 | [Poser un bloc sur un autre](38-superposition.holo) | `Stack, align:` |
| 39 | [Écrire les noms](39-ecrire-les-noms.holo) | l'écriture de Flutter |

Règle du projet : chaque notion ajoutée au langage reçoit sa leçon, dans la même pull request.
