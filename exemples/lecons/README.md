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
| 25 | [Un morceau commun à plusieurs pages](25-imports.holo) | `import`, `Component`, `Use` |
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
| 40 | [La langue, la description, l'image de partage](40-langue-et-partage.holo) | `Page(lang:, description:, image:)` |
| 41 | [Une vidéo](41-video.holo) | `Video` |
| 42 | [Un tableau de données](42-tableau.holo) | `Table` |
| 43 | [Un texte long](43-texte-long.holo) | `Input(lines:)` |
| 44 | [Un choix parmi plusieurs](44-choix.holo) | `Choice` |
| 45 | [Le survol qui agit](45-survol-qui-agit.holo) | `On(Carte.hover)`, `hoverEnd` |
| 46 | [Sinon](46-sinon.holo) | `If(…, else: [ … ])` |
| 47 | [Une seule fois, plus tard](47-plus-tard.holo) | `After` |
| 48 | [L'heure du visiteur](48-heure.holo) | `year`, `month`, `day`, `weekday`, `hour`, `minute` |
| 49 | [Écrire une carte une fois, la répéter](49-repeter.holo) | `Repeat`, `Item`, `item` |
| 50 | [Un texte soigné](50-texte-soigne.holo) | `line-height`, `letter-spacing`, `text-transform`, `text-decoration`, `text-shadow` |
| 51 | [Ombres, fonds, et une pose qui bouge](51-ombres-et-fonds.holo) | `box-shadow`, `linear-gradient`, `url(…)`, `rotate`, `scale`, `transition` |
| 52 | [Des couleurs nommées, et le thème sombre](52-variables-et-theme-sombre.holo) | `--or`, `dark:` |
| 53 | [Sur un téléphone](53-telephone.holo) | `phone:`, `display: none` |
| 54 | [Sa propre police](54-police.holo) | `fonts`, `Font` |
| 55 | [Barré, surligné, exposant, indice](55-petits-textes.holo) | `~~…~~`, `==…==`, `^…^`, `~…~` |
| 56 | [Aller plus bas dans la page](56-aller-plus-bas.holo) | `A(to: "#Horaires")` |
| 57 | [Une image et sa légende](57-image-et-legende.holo) | `Image(caption:, phone:)` |
| 58 | [Un lecteur de son](58-lecteur-de-son.holo) | `Sound(label:)` |
| 59 | [Une glissière](59-glissiere.holo) | `Slider` |
| 60 | [Une date, une heure, une couleur](60-date-heure-couleur.holo) | `Input(type: date | time | color)` |
| 61 | [Une barre de progression](61-progression.holo) | `Progress` |
| 62 | [Des plis qui s'ouvrent](62-plis.holo) | `Details` |
| 63 | [Une fenêtre par-dessus la page](63-fenetre.holo) | `Dialog`, `open`, `close` |
| 64 | [Envoyer un message](64-formulaire.holo) | `Form`, `send`, `sent`, `failed` |
| 65 | [L'icône de l'onglet](65-icone-de-l-onglet.holo) | `Page(icon:)` |
| 66 | [Multiplier, diviser](66-calculer.holo) | `mul`, `div` |
| 67 | [Écrire un nombre joliment](67-formats.holo) | `{minute:00}`, `{n:number}`, `{n:cents}`, `{weekday:name}` |
| 68 | [Une liste qui change pendant la visite](68-liste-qui-change.holo) | `State(taches: [])`, `push`, `remove(item)`, `clear`, `Repeat(over:)` |
| 69 | [Du code enfermé : un module](69-module-enferme.holo) | `module "…"`, `Module`, `run`, `done`, `failed` |
| 70 | [Les composants](70-composants.holo) | `components`, `Component(params:, rules:)`, `ArticleCard(…)`, `ArticleCard { }`, `ArticleCard.promo(…)` |
| 71 | [Une liste à champs](71-liste-a-champs.holo) | `State(articles: [ Item(…) ])`, `{item.title}`, `Data` qui remplit une liste, `push(Item(…))` |
| 72 | [La place qui reste, et un thème partagé](72-place-et-theme.holo) | `grow`, un fichier de styles importé, `Text.titre.discret(…)` |
| 73 | [Valeurs par défaut et signaux](73-defauts-et-signaux.holo) | `params: [title, price: 0]`, `emits`, `emit:`, `onAdd:` |
| 74 | [Un champ dans une ligne](74-champ-dans-une-ligne.holo) | `If(item.done, …)`, `item.done.set(1)`, les lignes gardées |
| 75 | [Le contenu d'un composant](75-contenu-d-un-composant.holo) | `params: [title, children]`, `children` posé seul, `Encadre(children: [ … ])` |
| 76 | [Envoyer un fichier](76-envoyer-un-fichier.holo) | `Input(type: file, accept: image, max: 2MB)` dans un `Form` |
| 77 | [Toutes les touches utiles du clavier](77-toutes-les-touches.holo) | `Key.enter`, `Key.escape`, `Key.a` à `Key.z`, `Key.digit0` à `Key.digit9` ; sur un téléphone, les touches à l'écran (rien à écrire) |
| 78 | [Apparaître en descendant](78-apparaitre-en-descendant.holo) | `Enter(…, inView: true)` |
| 79 | [Régler un son](79-regler-un-son.holo) | `Sound(volume:, loop:)`, `stop` |
| 80 | [Des tailles qui suivent le visiteur](80-tailles-qui-suivent.holo) | les pixels écrits en rem, `height: screen` |
| 81 | [La vue points se lit aussi](81-vue-points-et-lecteur-d-ecran.holo) | le lecteur d'écran en vue points (rien à écrire) |
| 82 | [Chercher, filtrer, trier, montrer plus](82-chercher-filtrer-trier.holo) | `computed: [ Filter(…, total: matching) ]`, `Repeat(over: found, empty:)`, `{found} sur {matching}` |
| 83 | [Comparer des textes](83-comparer-des-textes.holo) | `If(size, is: "L")`, `If(again, is: email)`, `When(answer, is: "Paris", …)` |
| 84 | [Des données qui arrivent, ou pas](84-donnees-arrivees-ou-pas.holo) | `Data(name: Shop, …)`, `On(Shop.done, …)`, `On(Shop.failed, …)`, `Shop.refresh` |
| 85 | [Une clé pour chaque élément](85-une-cle-pour-chaque-element.holo) | `Repeat(over: ordered, key: id)`, le clavier gardé, une ligne d'une liste calculée qui change sa source |
| 86 | [Des nombres à virgule](86-nombres-a-virgule.holo) | `State(price: 12.50)`, `{price}`, `sum.mul(1.1)`, `If(sum, over: 49.99)` |
| 87 | [Des dates](87-des-dates.holo) | `today`, `{arrival:date}`, `If(departure, over: arrival)`, `departure.add(7)`, `Days(…)`, `Input(type: date, min: today)` |
| 88 | [Un formulaire qui vérifie](88-un-formulaire-qui-verifie.holo) | `required: true`, `type: email`, `min:`, les messages sous les champs, Entrée, un envoi, 15 secondes |
| 89 | [L'écran et la place](89-telephone-et-ordinateur.holo) | `phone:`, `computer:`, `Page { max-width: 960px; }`, `narrow:`, `display: none` |
| 90 | [Ce qui dépasse](90-ce-qui-depasse.holo) | `line-clamp`, `max-height`, `min-height`, `min-width`, `overflow`, `white-space`, le mot trop long qui passe à la ligne |
| 91 | [Garder des proportions](91-garder-des-proportions.holo) | `aspect-ratio`, `object-fit`, `object-position` |
| 92 | [Le curseur](92-le-curseur.holo) | `cursor`, `url("viseur.svg")` |
| 93 | [Le texte justifié](93-texte-justifie.holo) | `text-align: justify` |
| 94 | [Décrocher la page](94-decrocher-la-page.holo) | `Zoom(detach: true)`, « Décrocher », « Accrocher » |
| 95 | [Un article long](95-un-article-long.holo) | `Aside`, `A(newTab: true)`, `A(download: true)`, `print: { … }`, les images qui viennent en approchant |
| 96 | [Une vidéo sous-titrée](96-une-video-sous-titree.holo) | `Video(captions: "film.vtt")` |
| 100 | [Une adresse qui porte une valeur](100-une-adresse-qui-porte-une-valeur.holo) | un fichier nommé `100-profils/{nom}.holo`, `{nom}` dans un texte, `If(nom, is: "yocthan")` ; la valeur se lit, ne se change pas |

Règle du projet : chaque notion ajoutée au langage reçoit sa leçon, dans la même pull request.

Certaines leçons ont un **essai écrit** à côté (`68-liste-qui-change.test`, `70-composants.test`, `71-liste-a-champs.test`, `73-defauts-et-signaux.test`, `74-champ-dans-une-ligne.test`, `75-contenu-d-un-composant.test`, `76-envoyer-un-fichier.test`) : des gestes et les valeurs attendues, que `holo test` joue et que les tests du moteur vérifient (`ADR-054`). Pour voir les valeurs pendant qu'on essaie une leçon : ajouter `?values` à son adresse.
