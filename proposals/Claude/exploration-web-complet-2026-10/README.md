# Les dix pistes de Codex pour un site web complet : l'exploration

## Contribution

- Sujet : l'issue #82, « explorer les dix pistes Codex pour un site web complet ».
- Auteur : Claude, avec une équipe de huit agents (quatre explorent, quatre relisent chaque affirmation dans le code), le 2026-10-07.
- Discussions sources : `HC-011`, `HC-013` ; l'issue #82 et le texte de Codex ; le but redit par Yocthan le 2026-10-07 : « le code doit d'abord faire tout ce que les HTML CSS JavaScript savent faire et ensuite faire le métaverse ».
- Décisions concernées : `ADR-005`, `009`, `013`, `015` à `018`, `023`, `024`, `027`, `029`, `030`, `033`, `034`, et depuis : `035` (une mécanique, jamais une capacité), `038` à `061`.
- Statut proposé : **EXPLORATION**. Les noms et les priorités sont des avis ; aucun statut de décision n'est changé.
- Référence relue : `main` à `7a48def`, puis `1119361` (la PR 139, qui ne touche aucune ligne citée). Rust 1.99.0, Node 22.21.0, Chrome 154.0.8037.95, Windows 11, le PC de Yocthan.

## En bref

**Le web de HoloCode n'est pas fini**, au sens du but de Yocthan. Sur les dix pistes de Codex, une est presque faite (les composants), cinq le sont à moitié ou aux deux tiers (les données, les formulaires, l'espace, les médias, les interactions), deux en grande partie (le design, la publication), une au quart (les graphismes), et la dernière n'a que ses briques (les calculs).

**Trois pannes ont été trouvées en vérifiant, et sont déjà réparées** : le pincement au doigt (cassé par la traduction du code, `ADR-060`), la boîte des modules (même cause), et Échap qui ne fermait plus une fenêtre (mon erreur du lot 9). PR 141 et 145, vérifiées dans Chrome avant et après. **D'autres défauts restent à réparer**, sans aucun mot nouveau (voir plus bas).

**L'ordre recommandé** : d'abord les réparations, puis les données et le calcul (pistes 1 et 10 ensemble), les formulaires sûrs, la mise en page d'ordinateur, puis le reste. Le serveur et les comptes suivent leur propre proposition. Les graphismes avancés viennent avec la 3D.

## Le tableau des dix pistes

| # | Piste | État vérifié | Options comparées | Recommandation | Coût | Objection | Expérience requise |
|---|---|---|---|---|---|---|---|
| 1 | [Données et répétition](piste-01.md) | À moitié : listes à champs reçues du serveur, fiche écrite une fois. Manquent : chercher, filtrer, trier, paginer, une clé choisie, l'état du chargement ; une fiche ne va pas au panier. Neuf défauts. | élargir `If` dans les lignes ; paramètres de `Repeat` ; **une liste calculée** ; laisser filtrer le serveur | réparer d'abord ; le passage ligne → page ; puis une liste calculée, commune avec la piste 10 | estimation : 8 à 10 séances, +16 à 26 Ko compressés ; mesuré : 4,3 ms par lettre pour 100 fiches (PC, Node) | sept mots nouveaux, la pente d'un langage de requêtes | deux écritures du même catalogue montrées à Yocthan ; mesure lettre → grille sur téléphone |
| 2 | [Formulaires](piste-02.md) | La moitié : choix, texte long, dates, fichiers, envoi. Manquent : vérifier les champs, annoncer les erreurs, envoyer sans doublon, un délai, Entrée. | vérification déclarée sur le champ ; par des règles ; par le serveur seul | ni doublon ni attente sans fin d'abord ; le serveur vérifie avec le moteur ; puis `required`, `type: email`, `min:` | mesuré : 8,8 Ko à l'ouverture, +184 Ko au premier focus ; estimation : 5 à 6 séances | le serveur n'est pas encore choisi ; chaque mot alourdit | la réservation du site de référence, avec et sans les mots ; TalkBack sur une erreur |
| 3 | [Design précis](piste-03.md) | En grande partie : dégradés, ombres, typographie, variables, thèmes, états. Trois défauts : le style de focus d'un champ ignoré, les tailles en variable pas en rem, le contraste mesuré dans un seul style. | des réglages CSS de plus ; de meilleurs choix par défaut | corriger les trois défauts ; puis l'anneau de focus, les graisses 100 à 900, `aspect-ratio`, `object-fit` | estimation : 400 à 500 lignes ; rien pour une page qui n'emploie rien de nouveau | chaque mot CSS rapproche du CSS | reproduire une vraie maquette de designer, compter ce qui bloque |
| 4 | [S'adapter à la place](piste-04.md) | À moitié : `Row`, `Grid`, `grow`, `phone:`. Mais la page est bloquée à 640 px, et le style `Main` ne vise rien. | requêtes de conteneur ; `Row(columnBelow:)` ; garder 640 px | poser la classe sur `<main>`, `Page { max-width }`, puis `Row(columnBelow:)` | estimation : 200 à 300 lignes ; un prototype vérifié à 360, 768 et 1 280 px | 640 px est peut-être une qualité (des lignes lisibles) | la fiche et le catalogue en trois versions, sur PC et téléphone |
| 5 | [Composants](piste-05.md) | Presque fait : paramètres, défauts, `children`, signaux, restylage. | — | documents à jour ; la page courante marquée ; `If` sur un paramètre | estimation : 400 à 450 lignes | chaque ajout rapproche d'un framework | deux ou trois débutants : une pastille « Nouveau » |
| 6 | [Interactions](piste-06.md) | Une bonne partie : survol, focus, touches, plis, fenêtres, apparition. Pincement cassé, réparé (PR 141). | `Details` dans `Nav` ; `Popover` | un essai dans un navigateur à chaque PR ; la fenêtre nommée, le focus tenu, `If` sur un texte | mesuré : 8,7 Ko, +184 Ko au premier geste | trois façons d'ouvrir, une de trop | pincer sur les Flip après la réparation |
| 7 | [Structure et publication](piste-07.md) | Presque fait sur le papier. Mesuré : `og:image` relatif, un SVG accepté ; le lien d'évitement vers `Main` casse la page ; une navigation sans nom. | réglé par le moteur ; par l'auteur | réparer le lien vers `Main` ; `Nav(label:)` et `aria-current` ; le titre obligatoire | mesuré : une page avec `Data` pèse 195 Ko à l'ouverture, contre 8,7 Ko | l'aperçu d'un lien n'a de sens qu'en ligne | relever l'en-tête du site de référence ; TalkBack sur les repères |
| 8 | [Médias](piste-08.md) | Aux deux tiers. Manquent : sous-titres, tailles d'image, chargement différé, image d'attente ; les sons sont chargés avant d'être joués. | `phone:` et un conseil ; des tailles préparées par l'éditeur | d'abord sans mot nouveau : `lazy`, `decoding`, sons chargés au premier geste ; puis `Video(subtitles:)` | mesuré : 12 photos, 1 272 Ko → 158 Ko sur téléphone ; 18,6 Ko de sons évités | un outil de préparation de plus | le catalogue avant et après, sur le Flip 5 |
| 9 | [Graphismes avancés](piste-09.md), [duel](duel-motion.md) | Au quart : le mouvement en CSS existe. Manquent : chemins, changement de forme, particules, défilement et pointeur, objets 3D. | CSS seul ; un canevas chargé quand il sert | `Loop(form:)`, `follow: scroll`, puis particules et chemins ; la 3D par son plan | mesuré : 200 grains par `Repeat` = 115 Ko de HTML | la moins utile pour un site ; chaque effet est un mot | particules à 120, 300 et 520 grains sur le Flip 3 |
| 10 | [Calculs purs](piste-10.md) | Les briques : entiers exacts, `mul`, `div`, formats, modules. Aucune valeur ne se calcule seule ; pas de dates. Modules cassés, réparés (PR 141). | formules ; **valeurs calculées nommées** ; modules élargis | les valeurs calculées, communes avec la piste 1 ; les dates ; des modules à plusieurs nombres | estimation : 4 à 5 séances, +11 à 17 Ko | les formules sont la pente vers le code | le même panier écrit trois fois |

## Les défauts trouvés en vérifiant

**Réparés le 2026-10-07** (vérifiés avant et après dans Chrome sans fenêtre) :

- le pincement au doigt : `event.touches` traduit en `event.keypresses` (PR 141) ;
- la boîte des modules : les noms des messages traduits d'un seul côté (PR 141) ;
- Échap ne fermait plus une fenêtre quand la page écoute `Key.escape` (PR 145).

**À réparer, sans mot nouveau** (le détail, avec les commandes, est dans chaque fiche) :

- les listes : deux `Repeat(over:)` sur la même liste se mélangent au redessin ; un seul `<li>` pour toute une liste ; le focus perdu dans une ligne refaite ; un paramètre écrit deux fois avalé sans rien dire ; **une seule image fautive dans un catalogue reçu efface toute la liste**, sans rien dire (piste 1) ;
- `Data` : un échec avalé, et la page fabriquée par le serveur ne contient pas les données (pistes 1 et 7) ;
- le lien d'évitement vers `Main(name:)` casse la page ; le style `Main { … }` ne vise rien (pistes 4 et 7) ;
- le style de focus d'un champ ignoré ; le contraste mesuré dans un seul style (piste 3) ;
- les sons d'une règle chargés avant d'être joués ; les images sans chargement différé (piste 8) ;
- un texte découpé en lettres mal lu par un lecteur d'écran (piste 9) ;
- et des documents en retard (le guide, `NOMS.md`, `COMPARAISON-WEB.md`, `ADR-017`, le site de référence) : liste précise dans chaque fiche.

## Les mesures réellement faites

**Par Claude** : Chrome sans fenêtre, cache vide, serveur local en Brotli, `main` à `7a48def`. On additionne les octets transférés (`encodedDataLength` du protocole de Chrome). Script : `essais-poids/mesurer_poids.mjs`.

| Page | Octets transférés |
|---|---|
| leçon 1, sans geste | 8,4 Ko |
| accueil du site de référence, sans geste | 10,4 Ko |
| leçon 71 (une liste reçue du serveur) | 194,3 Ko (le moteur léger arrive tout de suite) |
| leçon 1, après un toucher sur le menu | 193,1 Ko |
| leçon 81, en vue points | 845,6 Ko (le moteur complet, 634 Ko, arrive **en plus** du léger, 157 Ko) |

Tailles des fichiers du moteur, en Brotli qualité 11 : `pkg` 633,7 Ko ; `pkg-light` 156,4 Ko ; `page-engine.js` 22,5 Ko ; `page.html` 5,6 Ko.

**Par les agents** : chaque mesure est dans sa fiche, avec sa commande et sa sortie ; leurs scripts sont dans `essais*/` et `relecture-*/`.

**Pas exécuté** : aucune mesure sur un téléphone (aucun appareil ici) ; aucun essai avec un lecteur d'écran par une personne (TalkBack, NVDA) ; aucun débutant. Toute affirmation de fluidité sur téléphone reste à mesurer.

## La liste classée des travaux

| Ordre | Lot | Ce qu'on pourra faire à la fin | Dépend de | À décider avant |
|---|---|---|---|---|
| 1 | **Réparations**, sans mot nouveau | la même chose qu'aujourd'hui, sans les défauts listés plus haut | — | rien (ce sont des défauts) |
| 2 | **Un essai dans un navigateur à chaque PR** qui touche `moteur/web/` | ne plus casser le doigt ou les modules sans le voir | — | accepter un essai de plus dans GitHub (Chrome est déjà sur les machines de GitHub : rien à installer) |
| 3 | **Données et calcul** (pistes 1 et 10) : le passage d'une ligne vers la page, comparer des textes, `contains:`, des valeurs et des listes calculées (chercher, filtrer, trier, couper), `Data(name:)` avec `done` et `failed`, les dates | un catalogue où l'on cherche et trie ; un panier qui calcule la TVA ; « dans trois jours » | 1 | introduire les valeurs calculées (une seule notion pour les deux pistes) ; lever « un texte ne se compare qu'au vide » (`ADR-027`) ; les noms |
| 4 | **Formulaires sûrs** (piste 2) : sans doublon, un délai, le serveur qui vérifie, `required`, `type: email`, `min:`, les erreurs annoncées, Entrée | une inscription ou une réservation qu'on ne peut pas envoyer de travers | 1, et le serveur pour la vérification côté serveur | les mots de vérification |
| 5 | **Mise en page d'ordinateur** (piste 4) : `Main` stylable, `Page { max-width }`, `Row(columnBelow:)`, `valign`, `Grid(columnWidth:)` | une vraie page d'ordinateur et une vraie page de téléphone | 1 | 640 px : une qualité ou une limite ? |
| 6 | **Structure, médias, design** (pistes 7, 8, 3) : `Nav(label:)`, la page courante, le titre obligatoire, les images différées, `Video(subtitles:)`, l'anneau de focus, les graisses, `aspect-ratio` | un site qui se partage bien, plus léger sur téléphone, à l'identité plus fine | 1 | les sous-titres obligatoires ou non |
| 7 | **Interactions et composants** (pistes 6 et 5) : la fenêtre nommée, le focus tenu, un menu, un accordéon, `If` sur un paramètre | un menu de site complet, au doigt et au clavier | 3 (pour `If` sur un texte) | `Popover` ou `Details` pour le menu |
| 8 | **Le serveur et les comptes** (proposition à part) | des valeurs partagées, des comptes, le direct | — | les quatre questions de la proposition « chez soi d'abord » |
| 9 | **Graphismes** (piste 9) : `Loop(form:)`, `follow: scroll`, puis particules et chemins | le duel de motion design à égalité | 2 | les faire avant ou après la 3D |
| 10 | **Les autres dettes du grand tableau**, absentes des dix pistes : `aside`, `abbr`, listes de définitions, `target`, `download`, `fieldset`, `datalist`, `overflow`, `cursor`, filtres, `clip-path`, le hors ligne, la géolocalisation, la caméra, le presse-papiers, le partage | ce qui manque encore pour « tout le web » | 3 à 8 selon le cas | lesquelles refuser franchement (`ADR-035`) |

## Ce que Yocthan doit décider avant de construire

1. **Les valeurs calculées** : une seule notion pour chercher, filtrer, trier et calculer (pistes 1 et 10), ou des paramètres de `Repeat` ? C'est la décision la plus structurante.
2. **Comparer des textes** : lever la règle « un texte ne se compare qu'au vide ».
3. **La largeur de la page** : garder 640 px par défaut et permettre plus large, ou changer le défaut ?
4. **Le serveur** : les quatre questions de `proposals/Claude/serveur-et-comptes-2026-10-07.md`.
5. **Les graphismes avancés** : avant ou après la 3D ?
6. **Les styles isolés dans un composant** à paramètres : cela revient sur une correction d'`ADR-050`.
7. **Les sous-titres** : obligatoires pour une vidéo parlée, ou conseillés ?

## Mises à jour de documents à prévoir

Rien n'est changé par ce rapport. Chaque fiche finit par la liste précise. En résumé : le guide (les parties 6 septies, 6 quaterdecies, 6 septendecies, 10 et 11), `NOMS.md` (des mots rangés à tort dans « pas encore là »), `COMPARAISON-WEB.md`, `TABLEAU-WEB.md` (ajouter chercher, filtrer, trier, paginer), `ADR-017` (des phrases périmées), le site de référence (`catalogue.holo`, son en-tête), et le journal (les défauts, avec leurs commandes).

## Comment ce rapport a été fait

- Quatre explorateurs, chacun deux ou trois pistes, ont lu le code et lancé le moteur ; quatre relecteurs ont repris chaque affirmation dans le code, en partant du principe qu'elle était fausse. Ils ont fait **206 corrections** (63, 39, 63 et 41 par groupe). Chaque fiche finit par une ligne « Relu le 2026-10-07 ».
- Environ 1 480 actions, une heure trois quarts. Les preuves gardées ici sont les petits fichiers (essais `.holo`, scripts, sorties) ; les profils de Chrome temporaires et les copies du moteur n'ont pas été gardés.
- Claude a vérifié lui-même les trois pannes avant de les réparer, et a fait les mesures de poids.

## Les fichiers

- [`piste-01.md`](piste-01.md) à [`piste-10.md`](piste-10.md) : une fiche par piste (ce que Codex demandait, l'état vérifié, le scénario du site de référence, les options, la recommandation, l'exemple d'auteur et son jumeau web, les couches, le coût, la recette qui peut échouer, l'objection, l'expérience requise, les documents à mettre à jour).
- [`duel-motion.md`](duel-motion.md) : la répétition, les particules, les chemins vectoriels et les réactions au doigt, face au jumeau HTML.
- `essais/`, `essais-2-6-7/`, `essais-3-4-5/`, `relecture-*/`, `essais-poids/` : les preuves.
