# Piste 7 — Structure et publication

> Statut : EXPLORATION. Avis de Claude, pas une décision.

## Ce que Codex demandait

Sa ligne, dans l'issue #82 :

> | **7** | **Structure et publication** : navigation identifiable, tableaux, légendes, description de page, image de partage, adresses partageables. | Faire un site accessible, compréhensible par les moteurs de recherche et correctement présenté lorsqu'on partage son lien. |

## État vérifié (main, 7a48def, 2026-10-07)

Remarque de relecture : `main` est passé à `1119361` (PR 139) ; ce changement ne touche aucune ligne citée ici (vérifié par `git diff --stat 7a48def 1119361` : `JOURNAL.md`, `bin/holo.rs`, `lib.rs`, et des tests à partir de `flat.rs:1566`).

**Presque fait sur le papier** : les repères, le tableau, les légendes, la description, l'image de partage, la langue, l'icône et une adresse pour chaque lieu existent et sont décidés. **Mais ce que voient vraiment les réseaux et les robots est en retard**, et un lien d'évitement peut mener nulle part sans que le moteur le dise. Ce qui manque vraiment tient en quatre points : l'aperçu d'un lien partagé, le titre obligatoire, une navigation nommée, et des tableaux qui se remplissent d'une liste.

### Ce qui existe

| Demande de Codex | Où c'est | Décision, leçon |
|---|---|---|
| Navigation identifiable | `Header`, `Nav`, `Main`, `Footer` → `header`, `nav`, `main`, `footer` (`moteur/src/flat.rs:111-128`, `605-613`) ; la page a toujours un `<main>` (`flat.rs:246`) | `ADR-036`, leçon 35 |
| Tableaux | `Table(caption:, head:, rows:)` → `caption`, `thead`, `th scope="col"`, `tbody` (`flat.rs:831-876`) ; défile de côté sur un téléphone (`flat.rs:47`) ; une valeur dans une case (sonde P7-10) | `ADR-038`, leçon 42 |
| Légendes | `Image(caption:)` → `figure`, `figcaption` (`flat.rs:696-701`) ; la légende d'un tableau (`flat.rs:853-855`) ; celle d'un choix, `legend` (`flat.rs:806`) | `ADR-038`, `ADR-042`, leçon 57 |
| Description de page | `Page(description:)`, 300 caractères au plus (`flat.rs:225`) → `meta description` et `og:description` (`moteur/outils/server.mjs:75`) | `ADR-038`, leçon 40 |
| Image de partage | `Page(image:)` (`flat.rs:226`) → `og:image` (`server.mjs:76`) | `ADR-038`, leçon 40 |
| Langue, icône, titre | `lang:` (`flat.rs:224`, `457-461`) → `html lang` (`server.mjs:78`) ; `icon:` (`flat.rs:228-230`) → `link rel="icon"` (`server.mjs:74`) ; `title:` → `<title>` et `og:title` (`server.mjs:67`, `72`) | `ADR-038`, `ADR-042`, leçons 40, 65 |
| Adresses partageables | chaque monde a son adresse, `fichier.holo#Secret/Tresor` (`moteur/web/page-engine.js:567-569`) ; un fichier traversé prend sa propre adresse (`page-engine.js:213`) ; « retour » remonte (`page-engine.js:1285-1295`) ; un lien vers un bloc nommé, refusé si le nom n'existe pas (`moteur/src/rules.rs:113-124`) | `ADR-022`, `ADR-042`, leçons 8, 56 |
| Pour les robots | le serveur envoie la page déjà fabriquée par le moteur (`server.mjs:53-83`) | `ADR-033` |

**Mesure : ce que reçoit un réseau ou un robot**, leçon 40 (qui écrit `lang`, `description` et `image`) :

```text
$ curl -s -H 'Accept: text/html' http://localhost:8080/exemples/lecons/40-langue-et-partage.holo | grep -o '<html[^>]*>\|<title>[^<]*</title>\|<meta[^>]*>\|<link[^>]*>'
<html lang="fr">
<meta charset="utf-8">
<title>Leçon 40 : la langue et le partage</title>
<meta property="og:title" content="Leçon 40 : la langue et le partage">
<meta name="description" content="Une leçon de HoloCode : dire la langue de sa page et ce qu'elle montre quand on la partage.">
<meta property="og:description" content="Une leçon de HoloCode : dire la langue de sa page et ce qu'elle montre quand on la partage.">
<meta property="og:image" content="/exemples/lecons/etoile.svg">
<meta name="viewport" content="width=device-width, initial-scale=1">
```

### Ce qui manque (sondes `essais-2-6-7/sondes.sh`, `sondes2.sh`, et mesures)

1. **L'aperçu d'un lien partagé.** `og:image` est une adresse sans le nom du site (`/exemples/lecons/etoile.svg`, mesuré ci-dessus) : le protocole Open Graph demande l'adresse d'une image, et les grands réseaux attendent une adresse complète. L'image est ici un SVG, accepté par le moteur (sonde P7-04, `flat.rs:1402-1407` n'exige qu'un fichier voisin), que les grands réseaux ne montrent pas en aperçu (non vérifié réseau par réseau ici : pas d'accès aux réseaux depuis cette exploration). Manquent aussi `og:url`, `og:type`, `og:locale`, `og:image:alt`, `twitter:card` et l'adresse de référence (`link rel="canonical"`).
2. **Rien sans le moteur natif.** Ces lignes ne sont écrites que si le serveur trouve le moteur compilé (`server.mjs:57`, `60`) ; sinon le titre est « HoloCode » et rien d'autre. Dans le navigateur, la page ne remet que le titre et la langue (`page-engine.js:557-559`).
3. **Les robots ne voient pas ce que `Data` apporte.** Le serveur fabrique la page avec les valeurs de départ (`server.mjs:66` lance `holo html` sur le fichier seul ; `flat.rs:144` prend `state::initial`) ; le fichier JSON n'arrive qu'ensuite, dans le navigateur. Et une page qui a `Data` est « vivante » : le moteur arrive tout de suite (`flat.rs:187-192`). Mesure (`essais-2-6-7/poids.sh`, Chrome sans fenêtre, PC) : la leçon 27 transfère **195,2 Ko** à l'ouverture, contre **8,7 Ko** pour une page sans données (leçons 40 et 62). Attention : la leçon 27 écrit `Data(from: "27-donnees.json", every: 2s)` (son JSON est demandé trois fois en 5 s, `poids-sortie.txt`) ; l'option E1, qui ne vise qu'un `Data` sans `every`, ne l'allégerait pas.
4. **Le titre est facultatif.** `Page(children: [ H1("a") ])` est accepté (P7-14), comme `title: ""` (P7-13) : l'onglet s'appelle alors « HoloCode » (`server.mjs:67`) (WCAG 2.4.2). Mesure : sur les 98 pages versionnées du dépôt, **0 sans titre** ; le rendre obligatoire ne casserait rien dans le dépôt. Commande (une boucle sur `git ls-files 'exemples/*.holo' 'moteur/mondes/*.holo'`, commentaires retirés, en ne gardant que les fichiers dont le bloc racine est `Page`) et sortie : `pages: 98, sans title: 0, sans lang: 97`. Refait à la relecture, plus strictement, avec le HTML du moteur : pour chacune de ces 98 pages, `holo html <fichier> /x/` puis le `data-title` rendu ; sortie : `pages: 98, titre vide: 0, refusées par holo html: 0`. Et `lang:` n'est écrit que dans `exemples/lecons/40-langue-et-partage.holo`.
5. **La langue par défaut est le français** (`moteur/web/page.html:17` ; `server.mjs:78` ne la change que si elle est écrite). Mesure : 97 pages sur 98 n'écrivent pas `lang` ; c'est juste pour des leçons en français, faux pour une page anglaise oubliée (WCAG 3.1.1).
6. **Une navigation sans nom, sans « vous êtes ici ».** Deux `Nav` donnent deux `<nav class="holo-Nav">` sans nom (P7-18, mesuré) : un lecteur d'écran dit « navigation » deux fois. `Nav(label:)` est refusé (P7-01). Aucun lien ne porte `aria-current="page"` (P7-02 refusé ; `flat.rs:1006-1018` ; mesuré sur l'accueil du site de référence) ; pourtant les outils du moteur le font déjà pour eux-mêmes (`moteur/web/editor.js:614`, `moteur/web/stack.html:153`).
7. **Un lien d'évitement qui mène nulle part (défaut).** `A("Skip to content", to: "#Content")` vers `Main(name: Content)` est accepté par `holo check` (P7-16), mais le HTML fabriqué n'a pas d'`id="Content"` (P7-16b : `href="#Content"`, puis `<main>` sans `id`). C'est pire que « nulle part » (mesuré à la relecture, Chrome 154 sans fenêtre, moteur réel, page d'essai `relecture-2-6-7/fauxdepot/exemples/evitement.holo` servie par un second serveur local, `HOLO_REPO` pointé sur ce dossier, port 8099, arrêté ensuite) : Entrée sur le lien laisse le focus sur le lien, met `#Content` dans l'adresse, fait venir le moteur, qui prend `Content` pour un monde et affiche « Le moteur a refusé ce fichier : ligne 1, colonne 1 : aucun point nommé « Content » ne contient un monde, à cet endroit du fichier » ; le texte de la page reste lisible, mais le démarrage du moteur s'arrête là (`page-engine.js:1274-1279`, `1513-1516`). La cause : le nom de `Main` n'est pas écrit dans la page (`flat.rs:121-124`, `246`), alors que l'`id` se pose après un `data-name` (`flat.rs:198-206`). C'est exactement le défaut qu'`ADR-042` promettait de refuser (« une ancre qui ne mène nulle part : refusée », `ADR-042:40`). Même défaut, vu à la relecture, pour le nom de la page : `Page(name: Top, …)` avec `A("Haut", to: "#Top")` passe `holo check`, et le HTML n'a aucun `id="Top"` ; `rules.rs:113-124` accepte le nom de n'importe quel bloc, même d'un bloc que la page n'écrit pas (un `Header` ou une `Dialog` nommés reçoivent bien leur `id`). Aucun lien d'évitement n'est fabriqué tout seul non plus.
8. **Des tableaux figés.** Pas d'en-tête de ligne (`th scope="row"`, P7-06), pas de case sur deux colonnes (`docs/01-holocode/TABLEAU-WEB.md:511`), et surtout pas de tableau rempli par une liste : `rows:` n'accepte que des lignes écrites à la main (P7-07). Un stock, des commandes ou un catalogue venus du serveur ne peuvent pas être un `Table`.
9. **Le reste du web structuré.** `Aside` n'existe pas (P7-08). Pas de `sitemap.xml`, pas de `robots.txt`, pas de données structurées (JSON-LD). Une page refusée par le moteur est servie avec le code 200 et le titre « HoloCode » (mesuré : `pied.holo` → 200 ; un fichier absent → 404) : un robot l'indexe comme une page vide (`server.mjs:80-82`).
10. **Une adresse ne garde aucune valeur.** Rien ne lit ni n'écrit `?format=Grand`. Les `?` sont pris par les outils du moteur (`?values`, `?bare`, `?view`, `?enter`, `?zoom`, `?webgl`, `?world`… : `page-engine.js:30`, `34`, `433`, `1456`, `1498-1512`), et tout `?` fait venir le moteur à l'ouverture (`page.html:265`).
11. **Le site de référence n'emploie rien de tout cela.** Aucune de ses pages n'écrit `lang`, `description`, `image`, `icon` ni `Main` (recherche dans `exemples/site-reference/` : aucun résultat) ; son accueil ne donne que `og:title` (mesuré : `curl` de `accueil.holo`).

### Documents en retard

- `docs/01-holocode/COMPARAISON-WEB.md` : `:3` (« État au 2026-10-03 ») ; `:39` (`u, s, mark, small, sub, sup` : « manque », contredit par `:97` pour `s`, `mark`, `sub`, `sup` ; `u` et `small` manquent toujours) ; `:52` (lien vers un endroit de la page « en partie », contredit par `:98`) ; `:60` (« `Image(alt:)`, facultatif » : obligatoire depuis `ADR-038`) ; `:61-62` (`picture`, `figure` : « manque », contredit par `:94`) ; `:105-106` (`details`, `dialog`) ; `:116` (`Part`, devenu `Component`) ; `:187-188` (verdict : « Pas de tableau »).
- `docs/01-holocode/NOMS.md` : `:201` (`import`, `module` « lus, pas encore appliqués » : appliqués ; `bridge js`, `bridge css` sur la même ligne sont refusés, `ADR-011` partie B) ; `:210` (`h4, h5, h6` « refusés exprès » : ils existent, `ADR-036`) ; `:224-236` (« Pas encore là » : `alt`, `header`, `nav`, `table`, `meta description`… presque tous faits ; `aside` manque encore, P7-08).
- `docs/01-holocode/GUIDE.md` : `:5` (« État : 2026-10-03 ») ; `:1719` (« H1 à H3 ») ; `:1803` (« seuls le gras et l'italique sont rendus » : le code, le barré, le surligné, l'exposant et l'indice le sont aussi, `flat.rs:1429-1431`).
- `TABLEAU-WEB.md:455` donne description et image de partage à 100 % : avis environ 60 % tant que l'aperçu est incomplet (estimation) ; `:459` `Nav` à 100 % : avis environ 80 % (estimation).
- `exemples/site-reference/README.md:16` (« `Part`, `Use`, `import` ») ; `exemples/site-reference/catalogue.holo:1-2` (« le langage n'a pas encore de liste répétée » : `Repeat` existe, `ADR-040`, `ADR-044`).

## Le scénario du site de référence

**Les cas de Codex :** l'accueil a « un menu commun » (`proposals/GPT5.6/site-reference-2026-10-06/README.md:34`) ; le parcours commence par « Ouvrir l'accueil par un lien dans Chrome sur téléphone » (`README.md:48`) ; F01, page lue JavaScript coupé (`RECETTE.md:15`) ; F06 et F07, « bon lieu et bonne adresse », « adresse honnête » (`RECETTE.md:20-21`) ; F08, « Titres dans l'ordre » (`RECETTE.md:22`) ; F15, le parcours au lecteur d'écran (`RECETTE.md:29`).

**La tâche concrète sur le site construit :** la fiche « Lever sur le fleuve » (`exemples/site-reference/fiche.holo`).

1. Quelqu'un colle le lien de la fiche dans WhatsApp : l'aperçu montre le titre, la description et l'image du tableau.
2. Un moteur de recherche trouve la fiche avec sa description, et les deux formats du tableau (venus d'un fichier JSON) sont dans le HTML qu'il lit.
3. Au lecteur d'écran : « Aller au contenu » en premier ; le menu s'appelle « Menu principal » ; « Lever sur le fleuve » y est marqué comme la page en cours ; le tableau des formats se lit ligne par ligne, chaque ligne nommée par son format.
4. Le visiteur choisit « Petit » et partage le lien : son ami ouvre la fiche avec « Petit » déjà choisi.

**Ce qu'il faut :** l'aperçu complet (manque 1), `Data` fabriquée par le serveur (3), `Nav(label:)`, `aria-current` et le lien d'évitement (6, 7), les en-têtes de ligne et un tableau rempli par une liste (8), une valeur gardée dans l'adresse (10). Et d'abord, sur le site lui-même : écrire `lang`, `description`, `image` et `Main` (11), ce qui ne demande aucun mot nouveau.

## Options comparées

### A. L'aperçu d'un lien partagé

| Option | Écriture HoloCode | HTML | Avantages | Défauts |
|---|---|---|---|---|
| **A1. Le serveur complète, l'auteur écrit le texte de l'image** | `image: Image(source: "lever-partage.png", alt: "…")` (la forme `image: "…"` reste acceptée) ; l'adresse publique du site est un réglage du serveur (par exemple `HOLO_PUBLIC=https://atelier.example`, puis la configuration de `holo serve`) | `og:image` complet, `og:image:alt`, `og:url`, `og:type`, `og:locale`, `twitter:card`, `link rel="canonical"` | rien de plus à écrire pour l'auteur ; un SVG ou un ICO refusé avec un message (« les réseaux ne montrent pas les SVG : PNG, JPEG ou WebP ») | il faut connaître l'adresse publique : en local, l'aperçu reste inutile (un réseau ne voit pas `localhost`) |
| A2. Un bloc de partage | `share: Share(title:, description:, image:, alt:)` | les mêmes balises | tout au même endroit | répète `title` et `description` ; des mots en plus |
| A3. Rien (aujourd'hui) | `image: "etoile.svg"` | `og:image` relatif | — | aperçus vides ou faux |

Une précaution : **ne jamais fabriquer l'adresse complète à partir de l'en-tête `Host` de la requête** (un visiteur malveillant peut le falsifier) : l'adresse publique doit être écrite dans la configuration du serveur.

### B. Le titre et la langue

| Option | Ce qui change | Avantages | Défauts |
|---|---|---|---|
| **B1. Le titre obligatoire** | `Page` sans `title`, ou avec `title: ""`, est refusée : « « Page » attend « title » : le nom de l'onglet, que lit d'abord un lecteur d'écran » | comme `alt` (`ADR-038`) ; 0 page du dépôt à corriger (mesuré) | une page écrite hors du dépôt, sans titre, serait refusée (avec le bon message) |
| B2. Le titre pris au premier `H1` | rien à écrire | aucune page refusée | caché ; un `H1` long fait un onglet illisible |
| **L1. La langue reste « fr » par défaut** | rien | 97 pages restent justes (mesuré) | une page anglaise oubliée est lue en français |
| L2. La langue obligatoire | 97 pages refusées (mesuré) | aucun oubli | casse presque tout le dépôt |
| L3. Une langue par défaut pour le site | réglage du serveur, puis de `holo serve` | une fois pour tout le site | attend le serveur |

### C. Une navigation identifiable

| Option | Écriture HoloCode | HTML | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **C1. Nommer, et le reste tout seul** | `Nav(label: "Menu principal", …)`, obligatoire dès qu'une page a deux `Nav` ; `aria-current="page"` posé par le moteur sur un lien vers la page même ; un lien « Aller au contenu » (dans la langue de la page) fabriqué devant l'en-tête, visible au focus ; l'`id` posé sur `<main>` | `<nav aria-label>`, `aria-current`, `<a class="skip" href="#content">` | `Semantics(label:)` | un mot (`label:`, déjà employé ailleurs) ; marche aussi dans un menu partagé par `Component`, à condition de donner au moteur le nom du fichier qu'il fabrique : aujourd'hui il ne reçoit que son dossier (`flat.rs:84`, `lib.rs:195`, `bin/holo.rs:151`) | un lien ajouté à toutes les pages qui ont un en-tête : à montrer à Yocthan |
| C2. À la main | `A(…, current: true)` ; l'auteur écrit `A("Aller au contenu", to: "#Content")` (une fois le défaut corrigé) | les mêmes | — | rien d'automatique | `current:` est faux dès qu'un menu est partagé entre pages ; l'évitement s'oublie |
| C3. Un fil d'Ariane | `Breadcrumb(…)` | `nav aria-label="Fil d'Ariane"` + `ol` | aucun | un motif connu | se fait déjà avec `Nav(label:)` + `List(ordered: true)` |

### D. Des tableaux

| Option | Écriture HoloCode | HTML | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **D1. Des en-têtes de ligne** | `Table(…, rowHeads: true)` : la première case de chaque ligne la nomme | `<th scope="row">` | `DataTable` n'en a pas | un mot ; le lecteur d'écran dit « Petit, Taille, 20 × 12 cm » | — |
| **D2. Un tableau rempli par une liste** | `Table(caption:, head:, over: formats, cells: ["{item.title}", "{item.size}", "{item.price:cents} €"])` | une boucle en JS qui fabrique des `tr` | `DataTable(rows: list.map(…))`, `DataRow(cells: […])` | `over:` a déjà ce sens (`ADR-044`) ; `cells:` est le mot de Flutter ; échappé comme les lignes de `Repeat` | dépend des listes (piste 1) pour trier et filtrer |
| D3. `Repeat` dans `rows:` | `rows: [ Repeat(over: formats, children: [ … ]) ]` | — | — | aucun mot | mélange des lignes et des blocs ; plus dur à vérifier |
| D4. Une case sur deux colonnes | `Cell("Fermé", span: 2)` | `colspan` | — | complet | plus tard : rare dans un site de vitrine |

### E. Ce que lisent les robots

| Option | Ce qui change | Avantages | Défauts |
|---|---|---|---|
| **E1. `Data` fabriquée par le serveur** | pour un `Data` sans `every`, le serveur lit le JSON voisin et fabrique la page avec ; la page n'est plus « vivante » pour si peu | les robots et les aperçus voient le contenu ; la page reste légère (environ 8,7 Ko au lieu de 195,2 Ko : estimation tirée de deux pages mesurées) | le serveur doit lire un fichier de plus (borné) |
| E2. Des codes justes | une page refusée répond 500 avec le message du moteur, au lieu de 200 | les robots n'indexent pas une page cassée | aucun |
| E3. `sitemap.xml`, `robots.txt`, données structurées | fabriqués par `holo serve` | référencement complet | attend le serveur ; plus tard |

### F. Une adresse qui garde un choix

| Option | Écriture HoloCode | Web | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| F1. Une page par fichier (aujourd'hui) | une fiche par tableau, faite d'un `Component` partagé (`ADR-050`, `ADR-058`) | une page par adresse | une route par page | rien de nouveau ; le meilleur pour les robots | douze fichiers pour douze tableaux |
| **F2. Des valeurs dans l'adresse** | `query: [format]` : `fiche.holo?format=Petit` est lu au départ, et récrit quand le choix change ; le serveur fabrique la page avec | `URLSearchParams`, `history.replaceState` | `queryParameters` de go_router | un lien partagé redonne la même page ; même fichier, même adresse, même résultat | les noms pris par les outils du moteur (`values`, `view`, `x`, `y`…) sont à interdire ; un `?` ne doit plus faire venir le moteur à lui seul (`page.html:265`) |
| F3. Des pages fabriquées d'une liste | `Pages(from: "catalogue.json", …)` | un générateur de site | — | un fichier pour cent fiches | un gros chantier ; plus tard |

### Noms (ADR-016)

| Nom proposé | Sens sur le web | Sens dans Flutter | Risque de confusion |
|---|---|---|---|
| `Nav(label:)` | `aria-label` | `Semantics(label:)` | faible : `label:` nomme déjà un champ, une vidéo, un son, une barre |
| `image: Image(source:, alt:)` | `og:image`, `og:image:alt` | aucun | moyen : une `Image` qui ne s'affiche pas ; mais `fonts: [ Font(…) ]` et `data: Data(…)` sont déjà des blocs posés en réglage de page |
| `rowHeads:` | `th scope="row"` | aucun | faible ; fait pendant à `head:` |
| `over:` (dans `Table`) | — | — | aucun : le sens de `Repeat(over:)` |
| `cells:` | les cases d'un tableau (`td`, `th`) | `DataRow(cells:)` : le même sens | faible |
| `query:` | la partie `?…` d'une adresse (`location.search`, `URLSearchParams`) | `queryParameters` | moyen : « query » veut aussi dire « requête » (une base de données, une recherche). Écartés : `url:` (on y lirait l'adresse de référence de la page), `address:` (une adresse postale, dans un formulaire), `share:` (le partage sur les réseaux, `navigator.share`) |
| `Aside` (plus tard) | `aside` : un encadré à part | aucun | faible |

## Recommandation

1. **Corriger le lien vers `Main`** (défaut 7) : poser l'`id` sur `<main>`, ou refuser le lien. Aucune décision de langage.
2. **Mettre le site de référence à jour** : `lang`, `description`, `image`, `icon`, `Main` sur ses pages (aucun mot nouveau).
3. **B1 : le titre obligatoire.** Rien à corriger dans le dépôt (mesuré).
4. **C1 : `Nav(label:)`, `aria-current` et le lien d'évitement fabriqués par le moteur.**
5. **D1 : `rowHeads:`.** Puis **D2 (`over:` et `cells:`)**, avec la piste 1.
6. **E1 et E2 : `Data` fabriquée par le serveur, et des codes justes.**
7. **A1 : l'aperçu complet**, avec `image: Image(source:, alt:)`, dès qu'il existe une adresse publique ; avant, seulement le refus du SVG.
8. Plus tard : F2 (`query:`), E3, `Aside`, D4. En attendant F2, documenter F1 (une page par fichier, faite d'un composant).

## Exemple d'auteur

Les lignes marquées `// proposé` sont une **écriture proposée** : elle n'existe pas. Sans rien écrire, et proposé aussi : le lien « Aller au contenu », `aria-current` sur « Lever sur le fleuve », l'`id` de `<main>`, l'aperçu complet (adresses complètes, `og:url`, `twitter:card`, `canonical`) d'après l'adresse publique réglée sur le serveur.

```holo
Page(
  title: "Lever sur le fleuve — L'atelier des mondes",
  lang: "fr",
  description: "Huile sur toile peinte en une matinée au bord du fleuve. Deux formats, dès 60 euros.",
  image: Image(source: "lever-partage.png", alt: "Un soleil jaune se lève au-dessus d'un fleuve bleu"),  // proposé : image: Image(…)
  state: State(format: "Grand", formats: [ Item(title: "Petit", size: "20 × 12 cm", price: 6000) ]),
  query: [format],                                                                                        // proposé : query
  data: Data(from: "formats.json"),
  children: [
    Header(children: [
      Text("L'atelier des mondes"),
      Nav(label: "Menu principal", children: [                                                            // proposé : label
        List(children: [ A("Accueil", to: "accueil.holo"), A("Catalogue", to: "catalogue.holo"), A("Lever sur le fleuve", to: "fiche.holo") ]),
      ]),
    ]),
    Main(children: [
      H1("Lever sur le fleuve"),
      Image(source: "lever.svg", alt: "Un soleil jaune se lève au-dessus d'un fleuve bleu", caption: "Peint en une matinée, en mai."),
      Table(caption: "Les formats", head: ["Format", "Taille", "Prix"], rowHeads: true,                     // proposé : rowHeads
        over: formats, cells: ["{item.title}", "{item.size}", "{item.price:cents} €"]),                  // proposé : over, cells
      Choice(value: format, label: "Votre format", options: ["Petit", "Grand"]),
    ]),
    Footer(children: [ Nav(label: "Informations", children: [ A("Mentions légales", to: "mentions.holo") ]) ]),   // proposé : label
  ],
)
```

Le fichier `formats.json`, à côté :

```text
{ "formats": [ { "title": "Petit", "size": "20 × 12 cm", "price": 6000 }, { "title": "Grand", "size": "40 × 25 cm", "price": 12000 } ] }
```

Vérifié : le fichier est refusé (le moteur signale d'abord le réglage inconnu `query`) ; `image: Image(…)` seul est refusé aussi ; et, les mots proposés retirés (`image: "lever-partage.png"`, `Nav` sans `label`, `rows:` écrit à la main), le reste est accepté :

```text
$ holo.exe check essais-2-6-7/piste-07-exemple.holo
essais-2-6-7/piste-07-exemple.holo : ligne 7, colonne 3 : « Page » n'a pas de paramètre « query » ; paramètres possibles : name, title, children, …
$ holo.exe check essais-2-6-7/image-bloc.holo      (Page(title: "a", image: Image(source: "x.png", alt: "y"), …))
image-bloc.holo : ligne 1, colonne 18 : « Page(image: …) » attend une image rangée à côté du fichier, comme "partage.png" : celle qu'on voit quand on partage le lien
$ holo.exe check essais-2-6-7/piste-07-sans-propose.holo
ok
```

**Le même en HTML, CSS et JavaScript**, qui fait la même chose qu'HoloCode **aujourd'hui** pour les données : le tableau est rempli dans le navigateur, et le choix lu et récrit dans l'adresse par le navigateur. Avec les options E1 et F2, HoloCode le ferait en plus sur le serveur, pour les robots ; le jumeau ne le fait pas sans un générateur de site ou du code serveur. L'adresse publique est écrite à la main dans l'en-tête. Fichier : `essais-2-6-7/piste-07-jumeau.html`.

```html
<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Lever sur le fleuve — L'atelier des mondes</title>
<meta name="description" content="Huile sur toile peinte en une matinée au bord du fleuve. Deux formats, dès 60 euros.">
<link rel="canonical" href="https://atelier.example/fiche.holo">
<meta property="og:type" content="website">
<meta property="og:locale" content="fr_FR">
<meta property="og:url" content="https://atelier.example/fiche.holo">
<meta property="og:title" content="Lever sur le fleuve — L'atelier des mondes">
<meta property="og:description" content="Huile sur toile peinte en une matinée au bord du fleuve. Deux formats, dès 60 euros.">
<meta property="og:image" content="https://atelier.example/lever-partage.png">
<meta property="og:image:alt" content="Un soleil jaune se lève au-dessus d'un fleuve bleu">
<meta name="twitter:card" content="summary_large_image">
<style>
  .skip { position: absolute; left: 8px; top: -40px; }
  .skip:focus { top: 8px; }
  .menu { display: flex; flex-wrap: wrap; gap: 14px; list-style: none; padding: 0; }
  .table-wrap { overflow-x: auto; max-width: 100%; }
  caption, th { text-align: left; }
</style>
</head>
<body>
<a class="skip" href="#content">Aller au contenu</a>
<header>
  <span>L'atelier des mondes</span>
  <nav aria-label="Menu principal">
    <ul class="menu">
      <li><a href="accueil.holo">Accueil</a></li>
      <li><a href="catalogue.holo">Catalogue</a></li>
      <li><a href="fiche.holo" aria-current="page">Lever sur le fleuve</a></li>
    </ul>
  </nav>
</header>
<main id="content" tabindex="-1">
  <h1>Lever sur le fleuve</h1>
  <figure>
    <img src="lever.svg" alt="Un soleil jaune se lève au-dessus d'un fleuve bleu">
    <figcaption>Peint en une matinée, en mai.</figcaption>
  </figure>
  <div class="table-wrap">
    <table>
      <caption>Les formats</caption>
      <thead><tr><th scope="col">Format</th><th scope="col">Taille</th><th scope="col">Prix</th></tr></thead>
      <tbody id="formats"><tr><th scope="row">Petit</th><td>20 × 12 cm</td><td>60,00 €</td></tr></tbody>
    </table>
  </div>
  <fieldset>
    <legend>Votre format</legend>
    <label><input type="radio" name="format" value="Petit"> Petit</label>
    <label><input type="radio" name="format" value="Grand"> Grand</label>
  </fieldset>
</main>
<footer>
  <nav aria-label="Informations"><a href="mentions.holo">Mentions légales</a></nav>
</footer>
<script>
  const address = new URL(location.href);
  const chosen = address.searchParams.get("format") ?? "Grand";
  for (const radio of document.querySelectorAll("input[name=format]")) {
    radio.checked = radio.value === chosen;
    radio.addEventListener("change", () => {
      address.searchParams.set("format", radio.value);
      history.replaceState(history.state, "", address);
    });
  }
  const cents = new Intl.NumberFormat("fr-FR", { minimumFractionDigits: 2 });
  fetch("formats.json").then((r) => (r.ok ? r.json() : null)).then((data) => {
    if (!Array.isArray(data?.formats)) return; // un fichier mal écrit ne change rien
    const rows = data.formats.slice(0, 100).map((f) => {
      const tr = document.createElement("tr");
      const head = Object.assign(document.createElement("th"), { scope: "row", textContent: String(f.title ?? "") });
      const size = Object.assign(document.createElement("td"), { textContent: String(f.size ?? "") });
      const price = Object.assign(document.createElement("td"), { textContent: `${cents.format(Math.trunc(Number(f.price) || 0) / 100)} €` });
      tr.append(head, size, price);
      return tr;
    });
    document.getElementById("formats").replaceChildren(...rows);
  }).catch(() => { /* pas de réseau : la page garde ses lignes */ });
</script>
</body>
</html>
```

Mesuré (`node mesure-jumeaux.mjs`) : **4 017 octets bruts, 1 381 octets compressés, 22 lignes de JavaScript** ; syntaxe vérifiée par `node --check`. Ces chiffres sont ceux du fichier `essais-2-6-7/piste-07-jumeau.html`, avec ses commentaires ; le bloc montré ci-dessus (même code) pèse 3 843 octets bruts, 1 292 compressés, 22 lignes (relecture : `node relecture-2-6-7/mesure.mjs`). Essayé dans Chrome sans fenêtre, depuis le disque (`node jumeau7-essai.mjs`) :

```text
7 · ?format=Petit : coché → Petit
7 · après « Grand » : adresse → ?format=Grand
7 · premier Tab → Aller au contenu
7 · Entrée sur le lien d'évitement : focus → MAIN#content
```

Le remplissage du tableau par `formats.json` n'a pas été éprouvé (il faut un serveur). La page HoloCode d'aujourd'hui qui ferait la même chose (avec `Data`) charge le moteur dès l'ouverture : environ 195 Ko, comme la leçon 27 (mesuré) ; avec E1, environ 9 Ko (estimation, d'après les 8,7 Ko mesurés d'une page sans données).

## Par couche

- **Langage** : `Nav(label:)` ; `image: Image(source:, alt:)` ; `Table(rowHeads:)`, puis `over:` et `cells:` ; le titre obligatoire ; plus tard `query:` et `Aside`.
- **Moteur (Rust)** : `flat.rs:121-124` et `246` (l'`id` de `<main>`) ; `flat.rs:605-613` (`aria-label` d'un `Nav`, refus de deux `Nav` sans nom) ; `flat.rs:1006-1018` (`aria-current` : le moteur compare l'adresse du lien à celle de la page qu'il fabrique ; il faudra lui donner le nom du fichier, que `holo html` et la page connaissent déjà) ; le lien d'évitement devant l'en-tête ; `flat.rs:218-244` (refuser un SVG pour `image:`, lire `Image(…)`) ; `flat.rs:831-876` (`rowHeads`, `over`, `cells`) ; `blocks.rs:23` (refuser une `Page` sans titre) ; une entrée « fabriquer avec ces données » pour `holo html`.
- **Enveloppe navigateur** : presque rien, la page arrive faite ; `page-engine.js` remet à jour le tableau rempli par une liste, comme les lignes de `Repeat(over:)` ; avec F2, lire et récrire l'adresse sans charger le moteur pour un simple `?`.
- **Services serveur** (`server.mjs`, puis `holo serve`) : l'adresse publique en réglage ; les balises complètes de l'aperçu ; lire le JSON d'un `Data` sans `every` et le donner au moteur ; répondre 500 pour une page refusée ; plus tard `sitemap.xml` et `robots.txt`.

## Dépendances

- D2 dépend des listes à champs (`ADR-051`, fait) et rejoint la piste 1 (trier, filtrer, paginer).
- A1 et E3 n'ont d'effet que sur un serveur public : la proposition « serveur et comptes » (`proposals/Claude/serveur-et-comptes-2026-10-07.md`) et la règle « d'abord me plaire en local ».
- `Nav(label:)` sert à la piste 6 (le menu dans un `Popover`).
- F2 dépend d'E1 (le serveur fabrique la page avec des valeurs) et touche aux paramètres des outils (`ADR-054`, `ADR-060`).
- La piste 8 (médias) porte les sous-titres d'une vidéo, autre « légende » (`TABLEAU-WEB.md:501`).

## Coût

| Travail | Moteur | Poids transféré | Travail |
|---|---|---|---|
| Défaut du lien vers `Main` | ~10 lignes de Rust, un test (estimation) | 0 | 0,25 séance (estimation) |
| B1 titre obligatoire | ~10 lignes de Rust, un test (estimation) | 0 | 0,25 séance (estimation) |
| C1 navigation | ~80 lignes de Rust (estimation) | +0,2 Ko par page pour le lien d'évitement (estimation) | 1 séance (estimation) |
| D1 `rowHeads` | ~15 lignes (estimation) | 0 | 0,25 séance (estimation) |
| D2 `over`, `cells` | ~150 lignes de Rust, ~30 de JS (estimation) | +1 Ko sur le moteur léger (estimation) | 1,5 séance (estimation) |
| E1 `Data` par le serveur | ~60 lignes de Rust, ~20 du serveur (estimation) | **environ −184 Ko** à l'ouverture d'une page à données sans `every` (estimation tirée de deux mesures sur deux pages différentes : 195,2 Ko pour la leçon 27, qui a `every: 2s`, et 8,7 Ko pour une page sans moteur) | 1 séance (estimation) |
| E2 codes justes | ~5 lignes du serveur (estimation) | 0 | 0,1 séance (estimation) |
| A1 aperçu complet | ~40 lignes du serveur, ~20 de Rust (estimation) | +0,4 Ko dans l'en-tête, avant compression (estimation) | 0,5 à 1 séance (estimation) |
| F2 `query:` | ~120 lignes de Rust, de JS et du serveur (estimation) | 0 si la page arrive faite | 1,5 séance (estimation) |

## Accessibilité, déterminisme, budgets

- **Accessibilité** : un titre pour chaque page (WCAG 2.4.2) ; la bonne langue (WCAG 3.1.1) ; un moyen de sauter le menu (WCAG 2.4.1) ; des repères nommés et la page en cours dite (`aria-current`) ; des en-têtes de ligne et de colonne (WCAG 1.3.1) ; une image de partage décrite (`og:image:alt`). Le lien d'évitement est visible au focus, jamais caché pour de bon. Rien ne bouge : pas d'enjeu de mouvement.
- **Déterminisme** : le même fichier, les mêmes données et la même adresse donnent la même page, sur le serveur et dans le navigateur ; `aria-current` se déduit du fichier et de son adresse ; l'adresse publique vient de la configuration, jamais de la requête.
- **Budgets** : les balises de l'aperçu pèsent quelques centaines d'octets (estimation) ; E1 retire environ 184 Ko à une page qui n'a besoin du moteur que pour lire un JSON (estimation tirée des mesures ci-dessus) ; un fichier `Data` lu par le serveur doit être borné (par exemple 256 Ko, comme un fichier `.holo`, `GUIDE.md:403` ; à décider).

## Recette qui peut échouer

| Essai | Ce qui doit se passer | Échec si |
|---|---|---|
| R1. `holo html` d'une page avec `A(to: "#Content")` et `Main(name: Content)` (sonde P7-16b) | `<main id="Content">`, ou un refus clair | `<main>` sans `id` (aujourd'hui) |
| R2. `curl` de l'en-tête de la leçon 40, adresse publique réglée | `og:image` commence par l'adresse publique ; `og:url`, `og:image:alt`, `twitter:card`, `canonical` présents | une adresse relative (aujourd'hui) |
| R3. `Page(image: "share.svg")` (sonde P7-04) | refusé, avec le message sur les formats | accepté (aujourd'hui) |
| R4. `Page(children: [ H1("a") ])` (sonde P7-14) | refusé : « « Page » attend « title » » | accepté (aujourd'hui) |
| R5. Deux `Nav` sans `label` (sonde P7-18) ; puis avec | refusé ; puis deux `<nav aria-label="…">` | accepté sans nom (aujourd'hui) |
| R6. L'accueil du site de référence, `curl` | le lien « Accueil » porte `aria-current="page"` | rien (aujourd'hui, mesuré) |
| R7. Une page à en-tête, Tab dès l'ouverture, puis Entrée | « Aller au contenu » reçoit le focus, puis le focus est dans `main` | Tab va droit au menu |
| R8. `Table(rowHeads: true)` | un `<th scope="row">` par ligne ; audit axe-core à 0 défaut | des `td` partout |
| R9. La leçon 27 sans `every` (copie d'essai), `curl` puis `poids.sh` | le HTML reçu contient déjà les valeurs du JSON ; moins de 10 Ko transférés, pas de moteur | 195 Ko (aujourd'hui) |
| R10. Une page refusée par le moteur, `curl -w '%{http_code}'` | 500 (ou le code décidé) | 200 (aujourd'hui, mesuré sur `pied.holo`) |
| R11. `fiche.holo?format=Petit`, `curl` puis dans Chrome | « Petit » coché dans le HTML reçu ; aucun moteur chargé à l'ouverture ; changer le choix récrit l'adresse | « Grand » coché, ou le moteur chargé pour rien |
| R12. Le lien de la fiche collé dans WhatsApp, Facebook (outil de débogage du partage) et LinkedIn | titre, description, image | `NON EXÉCUTÉ` tant qu'aucun serveur public n'existe : un réseau ne lit pas `localhost` |
| R13. TalkBack, liste des repères | « Menu principal, navigation » ; « Informations, navigation » ; « principal » | deux « navigation » sans nom |

R12 et R13 demandent un vrai téléphone et, pour R12, un serveur public : ni l'un ni l'autre ici.

## Objection

**La meilleure raison d'attendre :** l'aperçu d'un lien et le référencement n'ont de sens qu'en ligne. Or Yocthan veut d'abord que tout lui plaise en local, avant toute mise en ligne. A1, E3 et F2 peuvent donc attendre le serveur public. Ce qui sert dès maintenant, en local, c'est l'accessibilité : le titre obligatoire, la navigation nommée, le lien d'évitement corrigé, les en-têtes de ligne.

**Autre façon de faire :** ne rien fabriquer tout seul (ni lien d'évitement, ni `aria-current`), et seulement refuser ce qui manque. Plus prévisible pour l'auteur ; mais `aria-current` est presque impossible à écrire à la main dans un menu partagé par un composant.

## Expérience requise

1. Corriger le site de référence (`lang`, `description`, `image`, `Main`) et relever son en-tête avec `curl` : c'est gratuit, et cela montre ce qui manque encore.
2. Un essai TalkBack sur le téléphone de Yocthan : la liste des repères avant et après `Nav(label:)` ; le lien d'évitement.
3. Comparer la fiche HoloCode et son jumeau avec l'audit Lighthouse de Chrome (accessibilité et référencement ; il est livré avec Chrome, rien à installer), sur le PC.
4. Dès qu'un serveur d'essai public existe : coller le lien dans WhatsApp, Facebook et LinkedIn, et noter l'aperçu.
5. Décider avec Yocthan : le titre obligatoire ; le lien d'évitement automatique ; `image: Image(…)` ; le nom `query:` (ou un autre) ; le code d'une page refusée.

## Mises à jour de documents à prévoir

- **Guide** : corriger `GUIDE.md:5`, `:1719`, `:1803` ; après construction, une partie « Être bien présenté et bien trouvé » (aperçu, titre, langue, navigation nommée) et l'aide-mémoire de `Nav`, `Table`, `Page`.
- **Leçons** : « Nommer un menu », « Un tableau rempli par une liste », « Une adresse qui garde un choix » (si F2) ; corriger la leçon 40 (une image de partage en PNG, et son texte).
- **`NOMS.md`** : corriger `:201`, `:210`, `:224-236` ; ajouter les mots retenus.
- **`COMPARAISON-WEB.md`** : corriger `:3`, `:39`, `:52`, `:60-62`, `:105-106`, `:116`, `:187-188`.
- **`TABLEAU-WEB.md`** : `:455` et `:459` à revoir (estimations ci-dessus) ; ajouter les lignes `aria-current`, lien d'évitement, `og:url`, `canonical`, `sitemap` ; republier la page en ligne du tableau.
- **Site de référence** : `README.md:16` (`Component`), `catalogue.holo:1-2` ; les métadonnées des neuf pages.
- **Journal** et **`AGENTS.md`** : le défaut du lien vers `Main`, et l'état de l'aperçu.

Relu le 2026-10-07 : 12 corrections.
