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
| 48 | [L'heure du visiteur](48-heure.holo) | `year`, `month`, `day`, `weekday`, `hour`, `minute`, `second` |
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
| 97 | [Un module qui reçoit une liste](97-un-module-qui-recoit-une-liste.holo) | `Module(input: [notes], output: [moyenne, meilleure, nombre])`, le second contrat, une réponse refusée |
| 98 | [Un dessin](98-un-dessin.holo) | `Drawing`, `Rect`, `Circle`, `Line`, `Path`, `fill`, `stroke`, `thickness`, une mesure qui suit une valeur |
| 99 | [Un tableau de bord](99-un-tableau-de-bord.holo) | `Chart(kind: bars \| line \| pie, over:, value:, label:, title:)`, des données reçues, dessinées |
| 100 | [Une adresse qui porte une valeur](100-une-adresse-qui-porte-une-valeur.holo) | un fichier nommé `100-profils/{nom}.holo`, `{nom}` dans un texte, `If(nom, is: "yocthan")` ; la valeur se lit, ne se change pas |
| 101 | [Une valeur partagée](101-une-valeur-partagee.holo) | `shared: Shared(seats: 20, likes: 0)`, `{seats}`, `If(seats, over: 0, …)`, `seats.sub(1)` par un toucher ; le serveur arbitre et l'envoie en direct à toutes les pages ouvertes |
| 102 | [Une liste partagée](102-une-liste-partagee.holo) | `shared: Shared(groceries: [ Item(…) ])`, `push`, `item.done.set(1)`, `remove(item)`, `clear()` : la liste de tous, arbitrée par le serveur ; une ligne touchée se désigne par sa clé, jamais par son rang |
| 103 | [Confirmer un texte partagé](103-un-texte-partage-confirme.holo) | `Input(value: title)` et `On(Save.tap, effect: title.set(title))` : un brouillon à toi, publié par un toucher, avec ou sans JavaScript |
| 104 | [Se connecter : un compte gardé chez toi](104-se-connecter.holo) | `signedIn`, `{account}`, `A(to: "/account/signin")` ; les pages de compte et le code à 6 chiffres, fabriqués par le moteur (avec `holo serve`) |
| 105 | [Une page réservée aux membres](105-une-page-reservee.holo) | `Page(access: members)` ; sans être connecté, on est mené à « Se connecter », puis ramené |
| 106 | [Le panier qui suit le compte](106-le-panier-qui-suit-le-compte.holo) | rien à écrire : connecté, le panier est gardé par le compte, sur le téléphone comme sur l'ordinateur, avec ou sans JavaScript |
| 107 | [Une clé d’accès](107-se-connecter-par-une-cle.holo) | WebAuthn vérifié localement ; localhost ou HTTPS ; ajouter et retirer après confirmation |
| 108 | [Protéger et effacer son compte](108-proteger-et-effacer-son-compte.holo) | QR local, dix codes de secours à usage unique, effacement confirmé, frein par IP |
| 109 | [Un catalogue, page par page](109-un-catalogue-page-par-page.holo) | `Filter(…, offset: offset, limit: 20, total: matching)` : deux cents produits, vingt par page, « Page suivante » et « Page précédente » ; le total compté avant la coupe (de Codex) |
| 110 | [Un module qui dessine](110-un-module-qui-dessine.holo) | `Drawing(shapes: fleur)`, une liste de formes rendue par un module |
| 111 | [Un chronomètre](111-un-chronometre.holo) | `Stopwatch`, `start`, `stop`, `reset`, `stopped`, `{temps:stopwatch}` |
| 112 | [Une adresse qui se souvient](112-une-adresse-qui-se-souvient.holo) | `keep` dans un modèle d'adresse, `112-carnets/{nom}.holo` : chaque adresse garde ses valeurs ; `title: "Le carnet de {nom} : {pages} page(s)"` |
| 113 | [Une rangée qui se serre](113-une-rangee-qui-se-serre.holo) | `narrow: { … }` dans les cases d'un `Row` : `.carte { width: 45%; narrow: { padding: 8px; } }` |
| 114 | [L'historique dans une page](114-l-historique-dans-une-page.holo) | `address: [onglet, page]` : les valeurs dans l'adresse, un pas d'historique par toucher, une adresse qui se partage |
| 115 | [Des polices pour toutes les écritures](115-des-polices-pour-toutes-les-ecritures.holo) | `Font(family: "Inter")` sans fichier : les polices libres du moteur, l'arabe, le devanagari, le japonais, l'éthiopien |
| 116 | [Importer et exporter](116-importer-et-exporter.holo) | `Transfer(file: "notes.json", values: [note, notes])`, `File.export`, `File.import` : un fichier JSON des valeurs annoncées ; l'import est complet ou refusé (de Codex) |
| 117 | [L'appareil sur permission](117-appareil-sur-permission.holo) | `Device(kind: position \| clipboard \| camera \| microphone)`, `request`, `write`, `stop` : sur un bouton, jamais à l'ouverture ; rien n'est envoyé ; arrêt d'office (de Codex) |
| 118 | [Des notifications locales](118-notifications-locales.holo) | `Notification(title:, body:, after: 3s)`, `show`, `stop` : un rappel tant que la page reste ouverte ; permission et refus (de Codex) |
| 119 | [Une page hors-ligne](119-une-page-hors-ligne.holo) | `Offline(files: [])`, `save`, `remove` : une copie publique, rechargée sans réseau, puis effacée ; sans JavaScript, la page reste une page comme les autres (de Codex) |
| 120 | [Une liste de définitions](120-une-liste-de-definitions.holo) | `List(children: [ Term("Poids", "2 kg") ])` : un terme et sa définition, toujours ensemble ; une fiche technique, un glossaire |
| 121 | [Une abréviation, une date, une adresse](121-une-abreviation-une-date-une-adresse.holo) | `Page(abbreviations: [ Abbreviation("MJC", "…") ])` : le sens écrit à la première venue ; une date montrée lisible par les machines (`<time>`) ; `Address(children: [ … ])` |
| 122 | [Un groupe de champs](122-un-groupe-de-champs.holo) | `Fields(label: "Adresse de livraison", children: [ … ])` : des champs qui vont ensemble, et le nom du groupe que le lecteur d'écran annonce ; des cases sur une même question |
| 123 | [Des suggestions dans un champ](123-des-suggestions-dans-un-champ.holo) | `Input(suggestions: ["Pomme", "Poire"])`, `suggestions: villes` : le champ propose, on peut écrire autre chose ; les suggestions suivent la liste pendant la visite |
| 124 | [Une citation courte, le titre d'une œuvre](124-une-citation-courte.holo) | `<<bonjour>>` : les guillemets de la langue de la page ; `_Les Misérables_` ; `Quote(by:, work:)` |
| 125 | [Des nombres négatifs](125-des-nombres-negatifs.holo) | `negative: [temperature]`, `State(temperature: -2)`, `sub` sous zéro, `If(temperature, under: -20)`, `Input(min: -50)` : le signe moins de la langue de la page, un clavier qui l'a |
| 127 | [Une grille et ses zones](127-une-grille-et-ses-zones.holo) | `columnSpan: 2`, `rowSpan: 2` : une case sur deux colonnes, deux lignes ; `Grid(areas: ["haut haut", "menu texte"])`, `area: menu` : des zones, dans l'ordre de lecture ; sur un téléphone, rien ne déborde |
| 130 | [Partager la page](130-partager-la-page.holo) | `Device(kind: share)`, `Partage.request` : la feuille de partage du téléphone, avec le titre et l'adresse ; sur un ordinateur, l'adresse copiée ; `done`, `failed`, et la feuille fermée qui n'est pas une panne |
| 128 | [Réordonner une liste](128-reordonner-une-liste.holo) | `Repeat(over: tableaux, reorder: true, …)` : la poignée ⠿ à la souris et au doigt ; « Monter » et « Descendre » au doigt, au clavier et au lecteur d'écran, qui annonce la nouvelle place ; sans JavaScript aussi |
| 133 | [Faire vibrer le téléphone](133-faire-vibrer-le-telephone.holo) | `Device(kind: vibration, for: 200ms)`, `Petite.play` : un toucher, une rencontre, `for: [100ms, 80ms, 100ms]` ; rien avant le premier toucher, ni sous le mouvement réduit ; le signe toujours à l'écran |
| 135 | [Mélanger des sons](135-melanger-des-sons.holo) | Plusieurs sons à la fois ; `Sound(fade: 2s)` : un fondu à l'entrée et à la sortie ; `Sound(volume: pluie)` et une glissière par son : une table de mixage ; jamais un son avant un geste |
| 136 | [Se souvenir le temps d'une visite](136-se-souvenir-le-temps-d-une-visite.holo) | `visit: [prenom, personnes, atelier]` : un formulaire en deux pages (la seconde : `136-inscription/etape-2.holo`) ; les valeurs suivent le visiteur dans le même onglet, effacées à sa fermeture ; aucun cookie |
| 140 | [Une page dans la page](140-une-page-dans-la-page.holo) | `Embed(from:, label:, image:)` et `embeds: [ … ]` : une carte d'OpenStreetMap, une vidéo de YouTube ; rien ne leur est envoyé avant ton toucher ; enfermée, avec son titre, le clavier dedans ; sans JavaScript, un lien |

Règle du projet : chaque notion ajoutée au langage reçoit sa leçon, dans la même pull request.

Certaines leçons ont un **essai écrit** à côté (`68-liste-qui-change.test`, `70-composants.test`, `71-liste-a-champs.test`, `73-defauts-et-signaux.test`, `74-champ-dans-une-ligne.test`, `75-contenu-d-un-composant.test`, `76-envoyer-un-fichier.test`) : des gestes et les valeurs attendues, que `holo test` joue et que les tests du moteur vérifient (`ADR-054`). Pour voir les valeurs pendant qu'on essaie une leçon : ajouter `?values` à son adresse.
