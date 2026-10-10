# ADR-117 — Une page dans la page : `Embed(from:, label:, image:)` et `Page(embeds:)`

- Statut : ACCEPTÉ (Yocthan a dit « Oui » le 2026-10-09 à l'ouverture sous ces conditions ; essayé et validé par Yocthan le 2026-10-10 : « J'ai fait les essais, c'est déjà parfait »)
- Date : 2026-10-10
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #249 (« une page dans la page (vidéos, cartes, publications) »), l'une des huit fonctions ouvertes sous conditions (issues 246 à 253) ; les deux avis, avec leurs sources : `proposals/Claude/contraintes-2026-10-09/README.md` (fonction 1 : le détournement de clic et `frame-ancestors`, S1 à S3 ; le pistage d'une vidéo intégrée, même en `youtube-nocookie.com`, S4 et S5 ; la façade, S6 ; le titre d'un cadre, WCAG 4.1.2, S7) et `proposals/Gemini/contraintes-2026-10-09/README.md` (fonction 1 : OWASP, le `sandbox` de HTML, la CNIL sur les traceurs, WCAG 4.1.2, la façade de Lighthouse) ; `ADR-017` (l'apparence dans les styles), `ADR-038` (une vidéo jamais lancée seule, le texte d'une image obligatoire), `ADR-073` (un lien vers un nouvel onglet), `ADR-074` (`holo serve`), `ADR-116` (les données d'un autre site, lues par le serveur).
- Validation : Yocthan, le 2026-10-09, « Oui », à l'ouverture des huit fonctions, sous les conditions de l'issue.
- Projets affectés : HoloCode, HoloEngine, le serveur (`holo serve`)

## Contexte

- Un vrai site montre une carte, une vidéo ou une publication d'un autre site. En HTML, c'est `<iframe>` : la page entière d'un autre site, posée dans la sienne. HoloCode l'avait laissée de côté exprès.
- Le web, par défaut :
  - le cadre se charge à l'ouverture de la page. L'autre site reçoit l'adresse IP du visiteur, ses cookies et la page qu'il lit, même s'il ne regarde jamais la vidéo (avis Claude, S4). `youtube-nocookie.com` garde un identifiant dans le navigateur (S5) ;
  - le cadre a tous les droits que le navigateur lui laisse : ouvrir des fenêtres, emmener la page de l'auteur ailleurs d'un geste, demander la caméra si on la lui délègue ;
  - un cadre sans titre est muet pour un lecteur d'écran (WCAG 4.1.2, S7) ;
  - un lecteur vidéo complet télécharge de 500 Ko à 3 Mo dès l'ouverture (avis Gemini, Lighthouse).
- La façade (une image et un bouton ; la vraie page ne vient qu'au toucher) règle le pistage avant le choix du visiteur, et le poids (S6).
- Les deux avis concluent : ouvrir sous conditions, et c'est le moteur qui tient les règles, jamais l'auteur.

## Décision

```holo
Page(
  title: "A trip to the zoo",
  embeds: ["www.openstreetmap.org", "www.youtube-nocookie.com"],
  children: [
    Embed(from: "https://www.openstreetmap.org/export/embed.html?bbox=-117.1570%2C32.7310%2C-117.1410%2C32.7400&layer=mapnik",
          label: "Map: the San Diego Zoo", image: "map.svg"),
    Embed.video(from: "https://www.youtube-nocookie.com/embed/jNQXAC9IVRw",
                label: "Video: Me at the zoo, the first video on YouTube (2005)", image: "video.svg"),
  ],
)
.video { aspect-ratio: 4/3; }
```

1. **`Embed(from:, label:, image:)`** : la page d'un autre site, posée dans la page.
   - `from:` : son adresse, en HTTPS.
   - `label:` : son titre, obligatoire. Le lecteur d'écran le dit ; c'est aussi le texte du bouton qui la charge, et le titre du cadre.
   - `image:` : l'image de la façade, rangée à côté du fichier. Facultative : sans elle, la façade montre le titre sur un fond uni.
2. **`embeds: [ … ]`**, sur la page : les seuls sites permis, écrits une fois, en haut. De 1 à 16 noms de site ; chacun sert à au moins un `Embed`.
3. **La façade.** La page arrive avec une image du site de l'auteur, le titre, et le site qui se chargera (« Charger depuis www.openstreetmap.org »), sur un vrai bouton.
   - Rien ne part vers l'autre site avant le toucher : ni cadre, ni connexion préparée, ni image chargée chez lui.
   - L'adresse de l'autre site n'est que dans un attribut `data-`. Aucun `src` ni `href` que le navigateur suivrait de lui-même.
4. **Au toucher** (au doigt, à la souris, ou avec Entrée et Espace), la page légère (`web/page.html`) pose la page de l'autre site à la place de la façade.
   - Elle est enfermée : voir « Les droits du cadre ».
   - Son titre est celui de la façade, et le clavier y entre. La page de l'autre site arrive dans un autre processus du navigateur, qui peut perdre le clavier en route (le Chrome des machines de GitHub le perdait) : quand elle a fini d'arriver, si le clavier est toujours sur le cadre, la page légère le lui redonne.
   - Le moteur (560 Ko) n'a pas besoin d'être arrivé : le toucher répond tout de suite.
5. **Sans JavaScript**, la façade est un lien vers la page de l'autre site, avec le titre, qui s'ouvre dans un nouvel onglet, comme `A(newTab:)`. Le lien est dans `noscript` : quand JavaScript marche, le navigateur ne le lit pas.
6. **`holo serve`** dit au navigateur de n'accepter un cadre que de ces sites : `Content-Security-Policy: frame-src https://www.openstreetmap.org https://www.youtube-nocookie.com`, et `frame-src 'none'` pour une page qui n'en liste pas. C'est le navigateur qui tient alors la règle, même si une page se trompait.
7. **Une taille qui suit l'écran** : toute la largeur, en 16/9 au départ. Un style la change (`aspect-ratio: 4/3`, `ADR-017`).

## Les règles de sécurité, tenues par le moteur

| Règle | Comment | Essai (ce qui doit être refusé l'est) |
|---|---|---|
| 1. HTTPS seulement, vers un site listé, comparé exactement | l'adresse est lue strictement, partout où la page est lue (`holo check`, la page, le serveur) : `https://`, un nom de site, sans port ni `nom@`, sans valeur `{…}`, des caractères d'adresse ; le dernier morceau du nom commence par une lettre (une adresse IP est refusée sous toutes ses formes) ; ni `localhost` ni un nom du réseau local ; le nom est comparé à ceux de la liste, en entier | `only_https_to_a_listed_site_compared_exactly` : `http://`, `javascript:`, `data:`, un fichier, `//…`, quatre formes d'IPv4, `[::1]`, `localhost`, `.local`, `.home.arpa`, un port, `nom@`, `{…}`, une espace, `<`, un accent, `%zz`, 2 049 caractères, un site non listé, un sous-domaine, le domaine au-dessus, un nom qui finit pareil, une page sans liste ; `the_list_of_sites_is_read_strictly` ; `an_embed_has_a_title_and_a_local_image` (titre vide, absent, d'espaces, trop long, avec une valeur) |
| 2. Rien vers l'autre site avant le toucher | la page arrive sans cadre ; aucun `preconnect` ni `dns-prefetch` ; l'image de la façade est un fichier du site de l'auteur ; l'adresse n'est que dans `data-embed` et dans `noscript` | `an_embed_waits_for_a_tap_behind_a_local_image` ; dans Chrome, **zéro** demande vers l'autre site avant le toucher (comptées deux fois : par l'interception et par le réseau), aucun cadre, aucune adresse de l'autre site dans un `src` ou un `href`, les images de la façade servies par `holo serve` |
| 3. Enfermée | les droits sont écrits dans le code de la page légère, jamais lus dans la page : `sandbox="allow-scripts allow-same-origin"` (`allow-scripts` seul pour une adresse de la même origine que la page), `allow="fullscreen"`, `referrerpolicy="strict-origin"`, `title` ; tout est réglé avant que le cadre entre dans la page | dans Chrome, la fausse page intégrée : la page de l'auteur lui est fermée ; pendant une touche (un vrai geste), elle ne peut ni ouvrir de fenêtre ni emmener la page de l'auteur ailleurs ; seul `fullscreen` lui est permis (ni caméra, ni micro, ni position, ni lecture automatique, ni paiement) ; la position lui est refusée ; elle reçoit le site de l'auteur, jamais l'adresse de la page ; sur une page en HTTPS, une adresse de la même origine n'a plus que `allow-scripts` |
| 4. `frame-src` ne laisse passer que les sites listés | `holo serve` envoie `Content-Security-Policy: frame-src` avec les sites de la liste, `'none'` sinon ; un nom qui n'est pas un nom de site n'entre jamais dans l'en-tête | `the_frame_policy_names_only_the_listed_sites`, `server::embed_tests::a_page_says_which_sites_it_may_embed` ; dans Chrome, un cadre vers un site non listé, posé par un script, est refusé par le navigateur, et rien ne part vers lui |
| 5. L'accessibilité | la façade est un `<button>` : on l'atteint avec Tab, Entrée ou Espace la touche, le doigt aussi ; son nom est le titre, puis le site qui se chargera ; le cadre porte le titre ; après le toucher, le clavier est dans la page intégrée | dans Chrome : deux boutons nommés (« Carte : le zoo de San Diego, charger depuis www.openstreetmap.org ») ; Tab jusqu'à la façade, Entrée, le clavier dans le cadre, la touche « k » reçue par la page intégrée ; au doigt, la vidéo ; axe-core 4.10.3 sur la leçon, avant et après le toucher : zéro défaut |
| 6. Sans JavaScript, un lien | `noscript` : un lien avec le titre, `target="_blank"`, `rel="noopener noreferrer"`, nommé « …, ouvrir sur www.openstreetmap.org, dans un nouvel onglet » | `an_embed_waits_for_a_tap_behind_a_local_image` ; dans Chrome, JavaScript coupé : deux liens, nommés, aucun bouton, aucun cadre, aucune demande vers l'autre site |

## Les droits du cadre, un par un

Le moins de droits pour qu'une vidéo ou une carte marche.

| Droit | Donné ? | Pourquoi |
|---|---|---|
| `allow-scripts` | oui | un lecteur vidéo et une carte sont des programmes : sans scripts, rien ne marche. Ils tournent chez l'autre site, dans son cadre. |
| `allow-same-origin` | oui, sauf pour une adresse de la même origine que la page | l'autre site garde son origine à lui : ses réglages et son stockage, chez lui. Sans ce droit, le cadre a une origine vide : lire son stockage y lève une erreur (`SecurityError`), et un lecteur vidéo ou une carte qui s'en sert s'arrête (non essayé sur les vrais sites : ce conteneur ne les atteint pas). Ce droit ne donne aucun accès à la page de l'auteur : l'autre site est une autre origine. Le moteur ne laisse passer qu'un site nommé en HTTPS, et la page légère retire ce droit si l'adresse avait l'origine de la page (un auteur qui listerait son propre site), car un cadre de la même origine pourrait alors défaire son enfermement. |
| `allow-forms` | non | ni une vidéo ni une carte n'envoient de formulaire ; un formulaire dans le cadre pourrait imiter une page de connexion. |
| `allow-popups`, `allow-popups-to-escape-sandbox` | non | le cadre n'ouvre ni fenêtre, ni publicité, ni onglet. Le lien « Regarder sur YouTube » du lecteur ne s'ouvre pas : le lien de la façade sans JavaScript, ou l'adresse, y mènent. |
| `allow-top-navigation`, `…-by-user-activation`, `…-to-custom-protocols` | non | le cadre n'emmène jamais la page de l'auteur ailleurs, même pendant un geste du visiteur. L'essai dans Chrome le montre : sans `sandbox`, une touche suffit. |
| `allow-modals` | non | ni `alert` ni `confirm` par-dessus la page de l'auteur. |
| `allow-downloads`, `allow-pointer-lock`, `allow-orientation-lock`, `allow-presentation`, `allow-storage-access-by-user-activation` | non | rien de cela n'est nécessaire pour regarder une vidéo ou une carte. |
| `allow="fullscreen"` | oui, seul | une vidéo se regarde en plein écran sur un téléphone. Le cadre ne le demande que d'un geste dans son lecteur, et le navigateur dit toujours comment en sortir. |
| caméra, micro, position, paiement, presse-papiers, lecture automatique… | non | aucun n'est délégué : un cadre d'un autre site ne les a pas sans `allow`. Sans lecture automatique, la vidéo attend un second toucher, dans son lecteur : elle n'est jamais lancée toute seule (`ADR-038`). |
| `referrerpolicy="strict-origin"` | — | l'autre site apprend le site de l'auteur (`https://exemple.org/`), jamais l'adresse de la page : ni son chemin, ni ses valeurs (`/profil/ada`, `?tab=photos`). Pas `no-referrer` : le lecteur intégré de YouTube refuse de jouer sans savoir quel site l'intègre (« erreur 153 »), et `Referrer-Policy: same-origin`, l'en-tête de `holo serve`, suffit à la déclencher. Sources vues par une recherche le 2026-10-10, pages non joignables depuis ce conteneur : Simon Willison, « YouTube embed 153 error », 1er décembre 2025 (https://simonwillison.net/2025/Dec/1/youtube-embed-153-error/) ; université du Michigan, « Fixing YouTube Player Error 153 with Referrer Policy Settings » (https://teamdynamix.umich.edu/TDClient/30/Portal/KB/Article/14491/Fixing-YouTube-Player-Error-153-with-Referrer-Policy-Settings). |
| `title` | — | le titre de la façade : le lecteur d'écran annonce le cadre par lui (WCAG 4.1.2). |

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le nom du bloc | `Iframe` (le mot de HTML) ; `Frame` ; `Window` ; **`Embed`** (les deux avis) | `Iframe` est un mot de programmeur (« inline frame »), et il porte le défaut du web : un cadre chargé tout de suite, avec tous les droits. `Frame` fait penser à un cadre de tableau, et `<frame>` est un vieux mot de HTML. `Window` serait une fenêtre du navigateur. **`Embed`** est le mot que les sites donnent eux-mêmes à l'auteur : le bouton « Intégrer » (« Embed ») de YouTube, de Vimeo, d'OpenStreetMap. L'élément `<embed>` de HTML, fait pour les greffons (Flash), est mort ; `Embed` fabrique un `<iframe>`. |
| L'adresse | `src:` (HTML) ; `source:` ; `url:` ; **`from:`** | dans HoloCode, `source:` est un fichier rangé à côté (une image, une vidéo, un module). `from:` dit d'où vient le contenu, comme `Data(from:)`, qui lit aussi un autre site depuis `ADR-116`. |
| Le titre | `title:` (HTML, les deux avis) ; **`label:`** | `label:` est déjà le nom que dit le lecteur d'écran pour une vidéo, un son, un dessin, une capacité (`Video(label:)`). `title:` est le titre de la page (`ADR-101` l'a refusé pour la même raison). L'issue demande de reprendre les mots qui existent. |
| L'image de la façade | `facade:` (avis Gemini) ; `poster:` (la vidéo de HTML) ; `preview:` ; `thumbnail:` ; **`image:`** | « façade » est un mot des audits de vitesse ; `poster:` ne parle qu'aux programmeurs, et seulement pour la vidéo. `image:` est déjà une image rangée à côté (`Page(image:)`). |
| Où lister les sites permis | dans chaque `Embed` seulement (pas de liste) ; un bloc `Allowed(embeds: […])` (avis Claude) ; `holo-data/sites.txt` sur le serveur (`ADR-116`) ; la ligne de commande ; **`Page(embeds: […])`** | Sans liste, un morceau importé, ou une faute de frappe, ferait venir un site sans que l'auteur le voie ; la liste dit d'un coup d'œil quels sites la page peut appeler, et la règle des cadres en a besoin. Un bloc pour une liste est plus lourd qu'un réglage de la page, comme `negative:` ou `visit:`. `holo-data/sites.txt` est juste pour `ADR-116` : des clés, et le serveur qui lit l'autre site. Ici, c'est le navigateur du visiteur qui charge la page : la liste doit être connue partout où la page est fabriquée (`holo check`, le navigateur, `holo serve`, le serveur d'essai), elle n'a rien de secret, et le visiteur peut la lire. Un site permis pour des données n'est pas pour autant permis pour une page intégrée : ce n'est pas la même confiance. |
| Charger quand | tout de suite (le web) ; à l'approche de l'écran (`loading="lazy"`) ; **au toucher, derrière une façade** ; `load: on-tap` (avis Claude) | tout de suite, ou à l'approche, l'autre site apprend la visite sans que le visiteur l'ait choisi. La façade : rien avant son choix, et une page plus légère. Pas de réglage `load:` : la façade est la seule façon, on ne peut pas l'éteindre. |
| La façade sans JavaScript | un bouton qui ne fait rien ; **un lien, dans `noscript`** | le lien mène à la page de l'autre site, avec le titre. Dans `noscript`, il n'existe pas quand JavaScript marche : aucun `href` de l'autre site n'est dans la page (un navigateur peut chercher d'avance l'adresse du site d'un lien, sur une page en HTTP). |
| Où poser le cadre | dans le moteur en WebAssembly ; **dans la page légère (`page.html`)** | le toucher doit répondre tout de suite, avant l'arrivée du moteur (560 Ko). Une trentaine de lignes de JavaScript, et les droits y sont écrits en dur, jamais lus dans la page. |
| La taille | `width:` et `height:` (HTML) ; un réglage `ratio:` ; **16/9 au départ, `aspect-ratio` dans un style** | l'apparence se dit dans un style (`ADR-017`) ; une taille en pixels déborderait d'un téléphone. |
| Ce que dit `holo serve` | rien (pas de politique aujourd'hui) ; une politique de sécurité complète ; **seulement `frame-src`** | `holo serve` n'envoyait aucune politique de sécurité du contenu. Une politique complète (scripts, styles) serait une autre décision, et pourrait casser le moteur. `frame-src` seul ne touche qu'aux cadres : les sites listés, ou aucun. |
| Ce que l'autre site apprend de la page | tout (`unsafe-url`) ; le défaut du navigateur (`strict-origin-when-cross-origin`) ; rien (`no-referrer`) ; **le site seulement (`strict-origin`)** | voir « Les droits du cadre » : le site, jamais l'adresse de la page ; rien du tout empêcherait YouTube de jouer. |
| Le faux « autre site » des essais | permettre `http://127.0.0.1` dans les pages, le temps d'un essai ; un nom en `.test` renvoyé vers ce PC (`ADR-116`) ; **Chrome arrête chaque demande vers l'autre site et rend une fausse page (Fetch)** | rien ne change dans le moteur pour les essais ; la fausse page dit au parent ce qu'elle voit et ce qu'on lui refuse. |

## Ce qui est refusé, et pourquoi

- **Dans la page**, avec sa raison, par `holo check` :
  - une adresse qui n'est pas en HTTPS : `http://` (n'importe qui sur le chemin lit et change la page), `javascript:`, `data:`, un fichier, `//…` ;
  - une adresse IP (`1.2.3.4`, `127.1`, `0x7f.0.0.1`, `2130706433`, `[::1]`), `localhost`, un nom du réseau local ou d'essai (`.local`, `.lan`, `.home`, `.internal`, `.arpa`, `.test`, `.example`, `.invalid`) ;
  - un port, `nom@`, une valeur `{…}`, une espace, un caractère qui n'est pas d'une adresse, un `%` mal écrit, plus de 2 048 caractères ;
  - un site qui n'est pas dans `embeds:`, même un sous-domaine ou le domaine au-dessus d'un site listé ;
  - un titre absent, vide, de plus de 200 caractères, sur deux lignes, ou qui montre une valeur ;
  - une image qui n'est pas rangée à côté du fichier, ou qui n'est pas une image (`.svg`, `.png`, `.jpg`, `.jpeg`, `.webp`, `.avif`, `.gif`) ;
  - un paramètre sans nom, ou inconnu (`autoplay:`, `sandbox:`… : les droits ne se règlent pas) ;
  - un `Embed` hors d'une page (dans un fichier `Point`).
- **Dans `embeds:`** :
  - ce qui n'est pas une liste de 1 à 16 noms de site entre guillemets ;
  - `https://…` ou un chemin (le nom seul suffit), `*` (rien n'est deviné), une adresse IP, un nom local ;
  - un site écrit deux fois ;
  - un site dont aucun `Embed` ne vient : il élargirait la règle des cadres pour rien ;
  - `embeds:` dans un monde : la liste se règle sur la page.

## Les défauts du web évités

- **Le cadre chargé à l'ouverture**, qui piste avant tout choix : ici, rien avant le toucher.
- **La miniature chargée chez l'autre site** (celle de YouTube donne déjà l'adresse IP) : ici, une image du site de l'auteur.
- **`preconnect` « pour aller plus vite »**, qui contacte l'autre site d'avance : jamais.
- **Un cadre avec tous les droits** : ici, deux droits et le plein écran, écrits par le moteur.
- **Un cadre sans titre** : ici, refusé.
- **Une taille fixe en pixels**, qui déborde d'un téléphone : ici, la largeur de la page et une proportion.
- **Un site appelé sans que l'auteur le sache** (par un morceau importé, une faute) : ici, la liste.
- **L'adresse de la page envoyée à l'autre site** : ici, le site seulement.

## Limites, honnêtement

- **Après le toucher, le visiteur est chez l'autre site** : il voit son adresse IP, et peut le pister avec ses cookies (cloisonnés ou non selon le navigateur). La façade dit quel site va se charger : le choix est éclairé.
- **L'autre site apprend le site de l'auteur** (`strict-origin`), jamais l'adresse de la page.
- **Un nom public qui mène au réseau du visiteur** (par son DNS) : le moteur ne vérifie que le nom ; c'est le navigateur du visiteur qui protège son réseau local (Chrome le fait), pas le moteur.
- **Rien n'a été essayé sur les vrais sites** : ce conteneur n'atteint ni YouTube, ni Vimeo, ni OpenStreetMap ; une fausse page les remplace. À essayer sur un vrai téléphone.
- **`frame-src` vient de `holo serve`** ; le serveur d'essai de Node n'en envoie pas.
- **Une page intégrée qui va d'elle-même vers un autre site, dans son cadre** (une page de consentement, une connexion), y est arrêtée par `frame-src`, et la liste ne permet pas un site dont aucun `Embed` ne vient.
- **Dans la vue points**, une page intégrée chargée se voit vide : l'image de la page ne contient pas les cadres.
- **Une page intégrée dans une ligne d'une liste qui change** revient à sa façade quand la ligne est refaite.
- On ne referme pas une page intégrée : recharger la page rend la façade.

## Dettes

- Refermer une page intégrée, et revenir à sa façade.
- `credentialless` (Chrome) : la page intégrée sans les cookies que le visiteur a déjà chez ce site ; à essayer sur les vrais sites avant de l'ajouter.
- Protéger les pages HoloCode d'être mises dans le cadre d'un autre site (`frame-ancestors`, avis Claude, S1) : une autre décision.
- `frame-src` aussi dans le serveur d'essai de Node.
- La miniature d'une vidéo, cherchée par le serveur de l'auteur et servie par lui (avis Claude) ; aujourd'hui, l'auteur range lui-même une image.
- Partager la lecture stricte des noms de site avec `ADR-116`, quand les deux seront dans `main`.

## Critères de validation

- Tests du moteur (`cargo test --release --locked`) :
  - `embed::embed_tests` : `an_embed_waits_for_a_tap_behind_a_local_image`, `only_https_to_a_listed_site_compared_exactly`, `an_embed_has_a_title_and_a_local_image`, `the_list_of_sites_is_read_strictly`, `the_frame_policy_names_only_the_listed_sites` ;
  - `server::embed_tests::a_page_says_which_sites_it_may_embed`.
- Les essais savent échouer. Chaque mutation a été faite puis retirée :
  - dans le moteur, `http://` accepté, puis un sous-domaine accepté : `only_https_to_a_listed_site_compared_exactly` rate ;
  - dans Chrome, le cadre posé d'emblée, comme le web par défaut : deux demandes vers l'autre site avant le toucher ;
  - sans `frame.focus()` : le clavier n'entre pas dans la page intégrée ;
  - sans `sandbox` : pendant une touche, la page intégrée emmène la page de l'auteur vers un autre site ;
  - sans l'en-tête de `holo serve` : pas de `frame-src`, et un cadre non listé n'est pas refusé ;
  - sans la façade de la page légère : le bouton reste caché, le clavier ne l'atteint pas, aucun cadre.
- Dans Chrome : « une page dans la page : rien vers l'autre site avant le toucher ; au toucher, au clavier comme au doigt, la page intégrée enfermée, avec son titre, et le clavier y entre ; frame-src ; sans JavaScript, un lien (leçon 140, serve) ».
- La leçon `140-une-page-dans-la-page.holo`, et ses deux images de façade (`140-carte.svg`, `140-video.svg`, moins de 600 octets chacune).
