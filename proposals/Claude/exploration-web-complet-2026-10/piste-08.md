# Piste 8 — Médias complets

> Statut : EXPLORATION. Avis de Claude, pas une décision.

## Ce que Codex demandait

Sa ligne, dans l'issue #82 (texte transmis par Yocthan le 2026-10-06) :

> | **8** | **Médias complets** : images adaptées à l'écran, vidéo avec sous-titres, lecteur audio. | Présenter des contenus riches sans envoyer systématiquement les fichiers les plus lourds. |

## État vérifié (main, 7a48def, 2026-10-07)

**En bref : la piste est faite aux deux tiers.** Le lecteur audio existe. La vidéo existe, mais sans sous-titres. L'image a sa légende et une version pour téléphone, mais une seule taille de rechange, et rien contre les sauts de la page. En vérifiant, j'ai trouvé un défaut : une grande image posée dans la page la fait déborder sur un téléphone.

### Ce qui existe (vérifié dans le code)

| Notion | Écriture | Où dans le moteur | Décision, leçon |
|---|---|---|---|
| Une image et son texte | `Image(source:, alt:)`, `alt` obligatoire | `moteur/src/flat.rs:668-702` ; le `<img src alt>` est écrit à `flat.rs:686` | ADR-038 ; leçon 03 |
| Une légende | `Image(caption:)` donne `figure` et `figcaption` | `flat.rs:697-701` | ADR-042 ; leçon 57 |
| Une image plus légère sur téléphone | `Image(phone: "petite.jpg")` donne `<picture><source media="(max-width:640px)">` | `flat.rs:689-695` ; le seuil `AUTHOR_WIDTH = 640` est à `flat.rs:401` | ADR-042 ; leçon 57 |
| Le poids déclaré | `weight: 1KB` | seulement additionné pour le budget d'un `Point` : `moteur/src/rules.rs:340-356` | ADR-005 |
| Une vidéo | `Video(source:, label:)`, `.mp4` ou `.webm`, jamais lancée seule | `flat.rs:815-829` : `<video controls preload="metadata" playsinline aria-label>` | ADR-038 ; leçon 41 |
| Un lecteur audio | `Sound(source:, label:)` donne `<audio controls preload="metadata">` | `flat.rs:971-975` | ADR-042 ; leçon 58 |
| Un son joué par une règle | `Sound(name:, source:)`, `play`, `stop`, `volume`, `loop` | `flat.rs:950-980` (`preload="auto"` à `flat.rs:979`) ; `rules.rs:34` | ADR-031, ADR-061 ; leçons 28, 79 |
| Les réglages permis | des listes fermées | `moteur/src/blocks.rs:36-37` (`Image`, `Sound`) et `blocks.rs:60` (`Video`) | ADR-037 |

### Ce qui manque encore (vérifié : absent du code)

1. **Les sous-titres d'une vidéo.** Le moteur n'écrit aucun `<track>` (`flat.rs:823-828`). `Video` n'accepte que `name`, `source`, `label`, `weight` (`blocks.rs:60`). Le serveur ne connaît pas l'extension `.vtt` (`moteur/outils/server.mjs:29-50`) : il l'envoie comme un fichier quelconque, `application/octet-stream` (mesuré, plus bas).
2. **Plusieurs tailles d'une même image.** Il n'y a qu'une image de rechange, à un seul seuil (640 px). Pas de liste de largeurs (`srcset`), pas de largeur d'affichage (`sizes`). Le navigateur ne peut donc pas choisir selon la finesse de l'écran.
3. **Les dimensions de l'image.** Le `<img>` n'a ni `width` ni `height` (`flat.rs:686`). La page saute quand l'image arrive. Mesuré : un décalage (CLS) de 0,132 sur le catalogue d'essai. (Le CLS mesure les sauts de la page pendant le chargement ; Google le juge bon à 0,1 ou moins.)
4. **Le débordement.** Le CSS de base n'a pas de `max-width: 100%` pour les images (`flat.rs:14-75`). Seule l'image posée **en premier** dans un `Stack` est bornée (`width:100%`, `flat.rs:19`). Mesuré : posée directement dans la page, une photo de 1 280 px rend la page large de 1 280 px sur un écran de 390 px. Mesuré à la relecture (Chrome sans fenêtre, 390 × 844, densité 3, `node banc-relecture.mjs` dans `scratchpad/exploration/relecture-8-9-duel/`) : la même photo fait aussi déborder la page posée directement dans une `Grid` (page de 1 280 px) ou en second dans un `Stack` (page de 835 px) ; dans une `Row` ou une `Column`, elle est tenue (affichée en 390 × 219).
5. **Le chargement différé.** Aucun `loading="lazy"` (l'image attend d'approcher de l'écran) ni `decoding="async"` dans le moteur. Recherche dans `moteur/src` et `moteur/web` : aucun résultat.
6. **Les sons joués par une règle sont téléchargés à l'ouverture**, même si personne ne les joue (`preload="auto"`, `flat.rs:979`). Mesuré sur le jeu du site de référence : 18 620 octets reçus en sons avant tout geste (en-têtes compris ; les deux fichiers, `pop.wav` et `perdu.wav`, pèsent 7 100 et 11 068 octets). Or un navigateur ne joue aucun son avant un geste du visiteur (GUIDE.md, § 6 octies).
7. **Le poids déclaré n'est jamais comparé au vrai fichier.** Ni le serveur, ni le moteur dans le navigateur, ni les tests ne le font (recherche de `weight` dans `server.mjs`, `page-engine.js`, `.github/workflows/tests.yml` : aucun résultat). La leçon 41 déclare 80KB pour un fichier de 61 715 octets : juste par chance.
8. Pas d'image d'attente pour une vidéo (`poster`). Pas de transcription pour un son. Pour la transcription, `Details(summary:, children:)` (ADR-042) suffit déjà : il manque seulement de l'écrire dans le guide.
9. Le serveur ne connaît ni `.avif` ni `.gif` (`server.mjs:29-50`), alors que `.gif` est accepté pour l'envoi d'un fichier (`moteur/src/files.rs:27`, et `server.mjs:166`). Que Chrome affiche quand même une telle image : non vérifié.
10. *(Ajouté à la relecture.)* **Le serveur local n'est pas fait pour les gros médias.** Il ignore les demandes de morceau (`Range`) : mesuré, `curl -s -D - -o /dev/null -H "Range: bytes=0-99" http://localhost:8080/exemples/lecons/41-video.mp4` répond `HTTP/1.1 200 OK` et envoie les 61 715 octets, sans `Accept-Ranges`. Sans morceaux, on ne saute pas loin dans une vidéo pas encore arrivée ; et Apple demande des morceaux pour lire une vidéo sur iPhone (non vérifié ici : pas d'iPhone). Il n'envoie ni `ETag` ni `Last-Modified`, seulement `cache-control: no-cache` (mesuré sur `lever.svg`, avec `If-Modified-Since` : `200 OK` et les 318 octets) : chaque visite retélécharge chaque image. Enfin, il lit tout fichier en mémoire et le compresse en Brotli 11 avant de répondre (`server.mjs:85-94`), même une vidéo déjà compressée. Mesuré sur ce PC avec `node` : 5 Mo d'octets au hasard prennent 2 513 ms en Brotli 11, pour rien gagner (5 000 000 → 5 000 017 octets). Une grosse vidéo bloquerait donc le serveur plusieurs secondes à sa première demande (estimation).
11. *(Ajouté à la relecture.)* **Recadrer une image.** Les styles refusent `object-fit` et `aspect-ratio` (`moteur/src/styles.rs:40-66`). Vérifié : `Image { width: 160px; height: 160px; object-fit: cover; }` → « réglage inconnu « object-fit » ». Des vignettes de même taille faites de photos de formes différentes (celles de l'essai : 1 280 × 720 en largeur, 540 × 960 en hauteur, 1 000 × 900 presque carrée…) sont donc déformées, ou à recadrer à la main. Cela touche aussi la piste du design précis.

### Les documents en retard

- `docs/01-holocode/COMPARAISON-WEB.md:60-64` dit encore `alt` « facultatif », `picture` et `figure` « manque », et le son « pas un lecteur ». Ses lignes 94-95 disent l'inverse (« fait »). L'en-tête date du 2026-10-03.
- `docs/01-holocode/GUIDE.md:1671` (aide-mémoire) : `Sound` sans `volume` ni `loop`.
- `docs/01-holocode/NOMS.md:229` range encore `video`, `audio` dans « Pas encore là », comme presque toutes les lignes de ce tableau (`NOMS.md:224-237` : `img alt`, `table`, `form`, `details`, `:hover`, `fetch`… existent aujourd'hui).
- `docs/01-holocode/TABLEAU-WEB.md:499-501` est juste (« Un seul seuil » ; « Pas encore de sous-titres »).

## Le scénario du site de référence

Le cahier de Codex demande un catalogue de douze créations, avec des « images légères avec descriptions » (`proposals/GPT5.6/site-reference-2026-10-06/README.md:35`). Il fixe deux budgets : 150 Ko avec toutes les vignettes de l'accueil, 1 Mo de médias pour tout le site (`README.md:102-104`).

Le site construit contourne la question, honnêtement : ses images sont douze dessins SVG de 318 octets (`exemples/site-reference/lever.svg` et les autres). Une vraie boutique de tableaux montre des photos.

**Le scénario** : remplacer les douze dessins par douze vraies photos, et tenir les budgets, sur téléphone comme sur PC. Puis ajouter, sur la fiche, une courte vidéo de l'atelier avec ses sous-titres. Il faut pour cela : la bonne taille d'image pour chaque écran, des dimensions connues d'avance, une image qui ne déborde jamais, et une piste de sous-titres.

J'ai joué la première moitié sur le PC, dans un Chrome sans fenêtre, avec le réseau du cahier (10 Mbit/s, 100 ms d'aller-retour, cache vide). Douze captures du journal servent de photos : 1 311 209 octets en PNG. **Ce n'est pas un téléphone** : seulement la taille d'écran d'un téléphone (390 × 844, densité 3). Pas d'appareil ici.

| Version du catalogue | Images à l'ouverture, écran de téléphone | Images à l'ouverture, PC (1 280 × 800) | Sauts (CLS) téléphone / PC | La page déborde |
|---|---|---|---|---|
| Aujourd'hui, photos PNG : `Image(source:, alt:)` | 1 272 325 octets | 1 272 325 octets | 0,132 / 0,14 | non (tenue par la `Column` de chaque carte ; posée directement dans la `Grid`, elle déborderait) |
| Aujourd'hui, au mieux : `phone:` et WebP préparés à la main | 65 939 octets | 298 391 octets | 0 / 0,065 | non |
| Proposé : largeurs, `sizes`, dimensions, `lazy`, `max-width` | 158 340 octets | 65 939 octets | 0 / 0 | non |

Ce que cela montre :

- Avec des photos telles quelles, le site dépasse le budget de 1 Mo de médias dès le catalogue.
- `phone:` marche déjà très bien sur un téléphone, si l'auteur prépare ses fichiers lui-même. Mais sur PC, il envoie l'image de 1 280 px pour une case de 187 px : 4,5 fois trop.
- La proposition envoie la bonne taille partout. Sur un écran de densité 3, elle prend l'image de 640 px au lieu de 320 px : un peu plus lourde que `phone:`, mais nette (une case de 155 px y occupe 465 pixels réels).
- Le chargement différé n'a rien changé ici : Chrome télécharge d'avance les images proches de l'écran, et les douze le sont. Il servira sur une longue page.

### Comment c'est mesuré

Sur le PC de Yocthan, le 2026-10-07, avec Chrome 154.0.8037.95 sans fenêtre et `moteur/target/release/holo.exe`. Les fichiers d'essai sont dans `scratchpad/exploration/essais/` ; le dépôt n'a pas été touché. Un second serveur du dépôt (`node moteur/outils/server.mjs`, avec `PORT=8093` et `HOLO_REPO` pointé sur le dossier d'essai) a servi les pages, puis a été arrêté.

- `node preparer-images.mjs` : douze captures de `docs/06-journal/images/` copiées, puis encodées en WebP par Chrome. Sortie : `Total PNG : 1311209 octets ; WebP 1280 : 311454 ; WebP 640 : 160888 ; WebP 320 : 65580`.
- `cat catalogue-png.holo | holo.exe check -` → `ok` (de même pour `catalogue-phone.holo`, `grande-image.holo`).
- `node banc.mjs` puis `node banc2.mjs` (la page proposée est la vraie page de `catalogue-phone.holo`, dont seules les balises d'image sont réécrites ; `sizes` y est écrit à la main, `(max-width:640px) 50vw, 210px`, là où le moteur devrait le calculer ; `loading="lazy"` à partir de la 3e image ; `max-width:100%;height:auto` ajouté). Les octets sont ceux que Chrome a reçus, en-têtes compris (`encodedDataLength`). Sorties : `png-telephone : imagesDemandees 12, octetsImages 1272325, cls 0.132` ; `phone-telephone : 65939, cls 0` ; `propose-telephone : 12, 158340, cls 0, largeurPage 390` ; `png-pc : 1272325, cls 0.14` ; `phone-pc : 298391, cls 0.065` ; `propose-pc : 65939, cls 0` ; `grande-image-telephone : largeurEcran 1280, largeurPage 1280, image affichée 1280x720` ; `grande-image-corrigee-telephone : largeurPage 390, image affichée 390x219`.
- Même banc, la vidéo : `video-sous-titres : pistes { mode: "showing", repliques: 2 }, typeVtt "application/octet-stream"` (la page d'essai, écrite à la main, porte `<track kind="captions">`, pas `kind="subtitles"`) ; `video-holo : octetsMedia 61941` (le fichier de 61 715 octets arrive entier à l'ouverture, malgré `preload="metadata"`. Le serveur ignore les demandes de morceau, `Range` : mesuré à la relecture, voir le manque 10. Ce que Chrome télécharge d'un gros fichier : non mesuré).
- `node sons.mjs` (serveur 8080, en lecture) : `jeu.holo : pop.wav 7326 octets, perdu.wav 11294 octets, totalSons 18620` ; `58-lecteur-de-son.holo : 7326`. Pour la leçon 79, le même fichier a été demandé trois fois (21 978 octets), mais l'outil avait coupé le cache : à revérifier avec un cache vide ordinaire.
- La première version de la page proposée oubliait `max-width` : avec `width` et `height` écrits, les images s'affichaient à 1 280 px et la page débordait (1 491 px sur le téléphone). C'est la preuve que la borne de largeur doit venir avec les dimensions.

## Options comparées

### Manque 1 : les sous-titres

| Option | Écriture HoloCode | HTML, CSS, JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Un fichier de sous-titres | `Video(source: "atelier.mp4", label: "…", subtitles: "atelier.fr.vtt")` (écriture proposée) | `<track kind="subtitles" src="atelier.fr.vtt" srclang="fr" label="Français" default>` | paquet `video_player` : `closedCaptionFile:` et le widget `ClosedCaption` | WebVTT est le format standard (un texte avec des horaires), produit par les logiciels de montage et par YouTube ; rien à recopier | un format de plus ; le moteur doit le vérifier |
| B. Les répliques dans le `.holo` | `subtitles: [ Cue(at: 0s, to: 2s, "Bienvenue."), … ]` (écriture proposée) | écrire le `.vtt` à part, ou du JavaScript (`addTextTrack`, `VTTCue`) | pareil, en Dart | un seul fichier, tout vérifié, lisible | long pour une vidéo longue (un `.holo` est limité à 262 144 octets) ; le moteur doit fabriquer la piste |
| C. Obligatoires, comme `alt` | A ou B, et `subtitles: none` pour une vidéo sans parole (écriture proposée) | rien : le web ne l'impose jamais | rien | on ne peut plus les oublier (ADR-035, ADR-038) | refuse les fichiers d'aujourd'hui (la leçon 41) ; une contrainte de plus pour un débutant |

### Manques 2 à 5 : la bonne image, sans saut ni débordement

| Option | Écriture HoloCode | HTML, CSS, JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Des tailles préparées, le moteur choisit | rien de nouveau : `Image(source: "lever.jpg", alt: "…")` ; un outil prépare `lever.320.webp`, `lever.640.webp`, `lever.1280.webp` et note les dimensions (proposé) | `<img srcset="… 320w, … 640w, … 1280w" sizes="…" width height loading decoding>`, plus un outil (sharp, Squoosh) | images selon la densité : dossiers `2.0x/` et `3.0x/` à côté de l'image ; `cacheWidth` pour décoder plus petit | aucun mot nouveau ; le moteur calcule `sizes` d'après la disposition, qu'il connaît (`Grid(columns: 3)` dans 640 px) : c'est là que le web se trompe le plus ; même principe que la 3D (ADR-049 : « tout ce qui est lourd se prépare à l'avance ») | un outil à écrire ; les dimensions à garder à jour |
| B. L'auteur donne ses tailles | `Image(source: "lever-1280.webp", variants: ["lever-320.webp", "lever-640.webp"])` (écriture proposée) ; ou `phone:` seul (existe) | `srcset` écrit à la main | pareil | pas d'outil | l'auteur fabrique les fichiers et devine les largeurs : dur pour qui ne programme pas |
| C. Seulement ce qui est gratuit | rien : `max-width: 100%`, `decoding="async"`, `loading="lazy"` après les premières images | les mêmes, à la main | — | corrige le débordement ; presque rien à faire | ne réduit pas le poids ; sans dimensions, la page saute encore |

Pour l'outil de l'option A, trois façons :

- **A1. Dans `holo`, en Rust.** Il faut des bibliothèques d'images : `holo.exe` grossirait de 1 à 3 Mo (estimation), et le WebP avec perte demande une bibliothèque en C (libwebp).
- **A2. Dans le navigateur, par l'éditeur (`/editor`).** Chrome sait encoder du WebP depuis une image (`OffscreenCanvas.convertToBlob`). Mesuré dans Chrome 154 : douze PNG de 1 311 209 octets donnent 311 454 octets en 1 280 px, 160 888 en 640 px, 65 580 en 320 px. Rien à installer.
- **A3. Sur le serveur, à la demande.** Il faut un encodeur sur le serveur et un cache.

`phone:` garde un rôle dans l'option A : une **autre** image pour le téléphone (un autre cadrage), comme `<picture media>` sur le web. Les tailles, elles, servent la même image à plusieurs finesses.

### Manque 6 : les sons des règles

| Option | Écriture HoloCode | HTML, CSS, JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. Télécharger au premier geste | rien de nouveau : `preload="none"`, puis chargement au premier toucher de la page | `preload="none"` et du JavaScript au premier `pointerdown` | `AudioPlayer.setSource` au moment voulu | aucun son téléchargé avant un geste ; le navigateur ne pourrait pas le jouer de toute façon | un petit retard possible au tout premier son (non mesuré) |
| B. L'auteur choisit | `Sound(…, preload: false)` (écriture proposée) | l'attribut `preload` | pareil | contrôle fin | un mot de plus pour un détail |

### Manque 7 : le poids qui dit vrai

| Option | Écriture HoloCode | HTML, CSS, JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| A. `holo check page.holo` compare `weight` au fichier | rien de nouveau | rien : un budget se vérifie à part (Lighthouse, budgets de webpack) | rien | les budgets d'un monde deviennent vrais | l'éditeur du navigateur (`/editor`) ne voit pas les fichiers ; mais `holo check -` reçoit le dossier quand VS Code l'appelle (`outils/vscode-holocode/extension.js:68`, `moteur/src/bin/holo.rs:106-119`) : là, il peut comparer aussi |
| B. Le serveur le signale en fabriquant la page | rien de nouveau | — | — | marche aussi depuis l'éditeur | dépend du serveur |
| C. Retirer `weight`, mesurer le vrai poids | moins à écrire | — | — | plus d'oubli possible | casse le budget d'un `Point` (ADR-005) |

### Noms (ADR-016)

| Nom proposé | Son sens sur le web | Dans Flutter | Risque de confusion |
|---|---|---|---|
| `subtitles` | `<track kind="subtitles">` : la traduction, pour qui entend ; le grand public dit « sous-titres » pour tout | `ClosedCaption`, `closedCaptionFile` | faible |
| `captions` (écarté) | `<track kind="captions">` : pour les sourds, avec les bruits ; le mot le plus juste | `ClosedCaption` | **fort** : HoloCode a déjà `caption:`, la légende d'une image (`flat.rs:697`) et d'un tableau (`flat.rs:853`). `caption:` et `captions:` côte à côte sont un piège |
| `track` (écarté) | l'élément HTML lui-même | — | moyen : technique, et vague (piste de quoi ?) |
| `Cue` (option B) | `VTTCue`, une réplique dans WebVTT | — | faible, mais peu connu |
| `none` | `display: none` en CSS : « rien » | `BoxFit.none` | faible ; même sens |
| `variants` (option B) | aucun | aucun | faible |
| `sizes` (écarté pour B) | en HTML, `sizes` dit la **largeur d'affichage**, pas les fichiers | — | fort : même mot, autre sens |
| `holo prepare` (une commande, pas un mot du langage) | — | — | aucun ; déjà prévu pour la chaise (`docs/04-roadmap/PLAN-3D.md`, étape 4) |

## Recommandation

1. **Tout de suite, sans mot nouveau** (manques 4, 5 et 6, option A ; manque 9) : `max-width: 100%` et `height: auto` sur les images ; `decoding="async"` partout ; `loading="lazy"` à partir de la troisième image de la page ; les sons des règles téléchargés au premier geste ; `.vtt`, `.avif` et `.gif` dans le serveur. C'est une correction, de la taille du « petit défaut » du jour.
2. **Puis les sous-titres** : option A, avec le mot `subtitles:`. Les rendre obligatoires (option C, `subtitles: none` pour une vidéo sans parole) est cohérent avec `alt` ; c'est à Yocthan de trancher, car la leçon 41 serait refusée telle quelle.
3. **Puis les tailles préparées** (option A), d'abord dans l'éditeur du navigateur (A2 : mesuré, rien à installer). Un petit fichier de dimensions, écrit par l'outil, est lu par le serveur (la page fabriquée d'avance) et par le moteur (les morceaux refaits dans le navigateur). Le même outil servira la chaise (PLAN-3D, étape 4).
4. **Vérifier le poids déclaré** dans `holo check` (option A).
5. Plus tard : `poster:` ; écrire dans le guide comment donner la transcription d'un son avec `Details`.

## Exemple d'auteur

```holo
Page(
  title: "Lever sur le fleuve",
  lang: "fr",
  children: [
    H1("Lever sur le fleuve"),
    // Écriture existante. Ce qui change est dans le moteur (proposé) : il choisit
    // lever.320.webp, lever.640.webp ou lever.1280.webp, préparés par l'éditeur,
    // écrit les dimensions, et borne l'image à la largeur de la page.
    Image(source: "lever.jpg", alt: "Un soleil jaune se lève au-dessus d'un fleuve bleu",
      caption: "Huile sur toile, 40 × 25 cm."),
    Video(source: "atelier.mp4", label: "Une visite de l'atelier, en une minute",
      subtitles: "atelier.fr.vtt"),   // écriture proposée
    Sound(source: "carillon.mp3", label: "Le carillon de l'atelier"),
    Details(summary: "Ce qu'on entend", children: [
      P("Trois notes de carillon, puis le silence."),
    ]),
  ],
)
```

Le même en HTML et CSS. Il fait exactement la même chose : mêmes fichiers préparés (il faut aussi un outil côté web pour les fabriquer), mêmes dimensions, même borne de largeur, même piste de sous-titres dans la langue de la page, mêmes lecteurs qui ne démarrent jamais seuls. Le serveur doit aussi envoyer le `.vtt` comme `text/vtt`.

```html
<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Lever sur le fleuve</title>
<style>
  /* Les mêmes règles que le CSS de base du moteur (flat.rs:16-17, 46, 55-56, 59), plus la borne proposée. */
  main { display: block; max-width: 640px; margin: 0 auto; }
  main > * { display: block; box-sizing: border-box; margin: 0 0 16px 0; }
  img { max-width: 100%; height: auto; }
  figure { margin: 0 0 16px 0; }
  figcaption { font-size: .9em; opacity: .8; margin-top: 6px; }
  video { display: block; width: 100%; max-width: 640px; border-radius: 12px; background: black; }
  audio { display: block; width: 100%; max-width: 480px; }
  summary { cursor: pointer; font-weight: bold; }
</style>
</head>
<body>
<main>
  <h1>Lever sur le fleuve</h1>
  <figure>
    <img src="lever.640.webp"
         srcset="lever.320.webp 320w, lever.640.webp 640w, lever.1280.webp 1280w"
         sizes="(max-width: 640px) 100vw, 640px"
         width="1280" height="800" decoding="async"
         alt="Un soleil jaune se lève au-dessus d'un fleuve bleu">
    <figcaption>Huile sur toile, 40 × 25 cm.</figcaption>
  </figure>
  <video src="atelier.mp4" controls preload="metadata" playsinline
         aria-label="Une visite de l'atelier, en une minute">
    <track kind="subtitles" src="atelier.fr.vtt" srclang="fr" label="Français" default>
  </video>
  <audio src="carillon.mp3" controls preload="metadata" aria-label="Le carillon de l'atelier"></audio>
  <details>
    <summary>Ce qu'on entend</summary>
    <p>Trois notes de carillon, puis le silence.</p>
  </details>
</main>
</body>
</html>
```

L'écart, à fonctions égales : l'auteur HoloCode n'écrit ni les largeurs, ni `sizes` (qu'il faut calculer d'après la mise en page), ni les dimensions, ni la borne de largeur. Il écrit une ligne de plus pour les sous-titres. Le web les laisse oublier ; HoloCode pourrait les exiger.

## Par couche

- **Langage** : `subtitles:` sur `Video` (et `subtitles: none` si on les exige). Rien d'autre : les images n'ont aucun mot nouveau.
- **Moteur (Rust)** : dans `flat.rs`, le `<track>` ; pour `Image`, `srcset`, `sizes` calculé d'après la disposition, `width` et `height` lus dans le fichier de dimensions, `loading` et `decoding`. Vérifier le `.vtt` : l'en-tête `WEBVTT`, des horaires qui avancent, une taille bornée, aucune balise. `holo check page.holo` compare `weight` au fichier.
- **Enveloppe navigateur** : le CSS de base (`max-width`) ; dans `moteur/web/page.html`, charger les sons au premier geste ; l'éditeur prépare les tailles d'une image (canevas, puis WebP).
- **Services serveur** : les types `.vtt` (`text/vtt`), `.avif`, `.gif` dans `server.mjs` ; lire le fichier de dimensions pour la page fabriquée d'avance (ADR-033). Plus tard, peut-être : préparer à la demande, avec un cache. Et le manque 10, ajouté à la relecture : les morceaux (`Range`), de quoi garder une image en cache, pas de Brotli sur un média déjà compressé.

## Dépendances

- La correction de l'étape 1 ne dépend de rien.
- Les sous-titres : décider du mot et de l'obligation ; compléter ADR-038 ou écrire une fiche nouvelle.
- Les tailles préparées : le fichier de dimensions doit être le même pour le serveur et pour le moteur, sinon la page n'est pas la même selon qui la fabrique (ADR-008). L'outil peut être partagé avec PLAN-3D (étape 4, `holo prepare`), qui attend le feu vert de la 3D ; il peut aussi commencer seul, dans l'éditeur.
- La vérification du poids ne dépend de rien.

## Coût

- **Étape 1 (correction)** : moteur, environ 120 octets de CSS de base et 30 octets par image (estimation) ; travail, une demi-séance (estimation). Gain mesuré : débordement supprimé (390 px au lieu de 1 280 px) ; sauts de 0,132 à 0 une fois les dimensions connues.
- **Sous-titres** : moteur, 1 à 2 Ko de WebAssembly pour vérifier le `.vtt` (estimation) ; poids transféré, le fichier `.vtt` lui-même (126 octets pour l'essai ; environ 1 Ko par minute de parole, estimation) ; travail, une séance avec la leçon (estimation).
- **Tailles préparées** : rien de plus à l'ouverture d'une page ; éditeur, environ 3 Ko de JavaScript (estimation) ; moteur, environ 2 Ko (estimation) ; travail, deux à trois séances (estimation). Gain mesuré sur le catalogue d'essai : de 1 272 325 à 158 340 octets d'images sur un écran de téléphone (−88 %) et à 65 939 octets sur PC (−95 %).
- **Sons au premier geste** : une dizaine de lignes de JavaScript (estimation). Gain mesuré : 18 620 octets sur le jeu du site de référence ; davantage pour une musique (non mesuré).
- **Poids vérifié** : une demi-séance (estimation).

## Accessibilité, déterminisme, budgets

- **Accessibilité.** `alt` reste obligatoire. Les sous-titres servent les sourds et ceux qui regardent sans le son. Le navigateur montre la piste de lui-même (mesuré dans Chrome : piste montrée, avec ses 2 répliques, sur une page d'essai en `kind="captions"`) ; le bouton des sous-titres dans ses commandes : non vérifié. Nuance *(relecture)* : en HTML, la sorte faite pour les sourds est `kind="captions"` (les paroles et les bruits) ; `kind="subtitles"` vise qui entend mais ne comprend pas la langue. Le mot HoloCode (`subtitles:`) et la sorte écrite dans la page sont deux choix séparés : le moteur peut écrire `kind="captions"` même si le mot est `subtitles:`. Une image qui ne déborde plus garde le texte lisible à 320 px de large, comme l'exige le cahier (`proposals/GPT5.6/site-reference-2026-10-06/README.md:76`). La transcription d'un son passe par `Details`. Rien ne bouge : pas d'enjeu de mouvement réduit.
- **Déterminisme.** Le fichier choisi dépend de l'écran, comme sur tout site ; le contenu reste le même. Les fichiers préparés sont produits une fois et rangés : la page lit toujours les mêmes octets. Attention : deux versions de Chrome n'encodent pas forcément le même WebP. On garde donc les fichiers ; on ne les refait jamais à l'ouverture.
- **Budgets.** Le cahier fixe 150 Ko envoyés pour l'accueil (trois vignettes) et 1 Mo de médias rangés avec le site (`README.md:104`, « Sources du site et médias » : un poids de fichiers, pas un poids envoyé). Mesuré : avec la proposition, les douze photos du catalogue pèsent 158 340 octets envoyés sur un écran de téléphone ; sans elle, 1 272 325. Rangées, leurs trois tailles pèsent 537 922 octets (311 454 + 160 888 + 65 580, mesuré) : sous 1 Mo, si les originaux ne sont pas rangés avec le site. Un poids déclaré vérifié rend ces budgets vrais.

## Recette qui peut échouer

| Essai | Ce qui doit se passer | Ce qui fait échouer |
|---|---|---|
| R8-1 Débordement | une image de 1 280 px posée directement dans la page, écran de 390 px : `document.documentElement.scrollWidth` vaut 390 | plus de 390. **Échoue aujourd'hui** (mesuré : 1 280) |
| R8-2 Sauts | catalogue de douze photos, réseau 10 Mbit/s et 100 ms : CLS au plus 0,01 | plus de 0,01. **Échoue aujourd'hui** (0,132) |
| R8-3 Poids | le même catalogue, préparé : images au plus 200 000 octets sur 390 px (densité 3), au plus 100 000 sur 1 280 px (densité 1) ; Yocthan ne voit pas de flou | l'original ou la plus grande taille envoyés ; ou une image floue |
| R8-4 Sous-titres | `Video(…, subtitles: "film.fr.vtt")` : la piste est de sorte `subtitles`, dans la langue de la page, avec ses 2 répliques, montrée ; le serveur répond `text/vtt` | 0 réplique ; ou le type `application/octet-stream` (c'est le cas aujourd'hui) |
| R8-5 Refus | un `.vtt` sans l'en-tête `WEBVTT`, ou avec des horaires qui reculent, ou avec une balise : refusé par `holo check`, avec sa ligne ; `subtitles: "film.srt"` : refusé, « écris un fichier .vtt » | accepté en silence |
| R8-6 Obligation (si Yocthan la choisit) | `Video(source:, label:)` sans `subtitles` : refusé, avec « subtitles: none pour une vidéo sans parole » | accepté |
| R8-7 Sons | une page avec `Sound(name:, source:)` : 0 octet de son avant le premier geste ; après un toucher, `Ding.play` s'entend en moins de 300 ms sur PC | du son téléchargé avant un geste ; ou un premier son en retard |
| R8-8 Poids vrai | `Image(source: "photo-01.png", alt: "Une capture", weight: 1KB)` : `holo check page.holo` refuse, « le fichier pèse 247 804 octets, 1 000 déclarés » | accepté. **Échoue aujourd'hui** (mesuré à la relecture : `holo check` répond `ok`, la photo de 247 804 octets à côté). Sans `alt`, la ligne serait refusée, mais pour une autre raison |

## Objection

La meilleure raison de ne pas le faire : un outil de préparation est un chantier de plus, alors que `phone:` existe. Avec un bon conseil dans le guide (« exporte tes photos en WebP de 1 280 px, et une de 640 px pour `phone:` »), on gagne déjà presque tout sur un téléphone (mesuré : 65 939 octets). Le PC reçoit plus que nécessaire (298 391 octets au lieu de 65 939), mais il a en général le réseau pour.

Autre objection : exiger des sous-titres peut décourager un débutant qui veut juste montrer une vidéo de vacances. Le web ne l'exige pas, et la plupart des vidéos d'un petit site n'en ont pas.

Autre façon de faire : confier les images à un service qui les redimensionne à la volée (un CDN d'images). Mais le site dépendrait alors d'un service extérieur, et la page ne serait plus la même partout.

## Expérience requise

- Sur le téléphone de Yocthan (Flip 5) et sur un Android modeste : ouvrir le catalogue d'essai, version d'aujourd'hui et version proposée ; relever les octets par `chrome://inspect` ; Yocthan juge la netteté (320 px contre 640 px). Pas d'appareil ici : à faire.
- Avec TalkBack : la vidéo sous-titrée. Le bouton des sous-titres est-il annoncé ? Les répliques sont-elles lues ?
- Deux ou trois débutants : ajouter des sous-titres à une vidéo, avec le guide seul.
- Mesurer le retard du premier son quand il n'est téléchargé qu'au premier geste, sur PC et sur téléphone.
- Vérifier sur Safari (iPhone) et Firefox que la piste `.vtt` et les images `srcset` se comportent comme dans Chrome : non vérifié ici.

## Mises à jour de documents à prévoir

À lister, pas à faire maintenant :

- `docs/01-holocode/GUIDE.md` : § 6 duodecies (la vidéo et ses sous-titres), § 6 quindecies (l'image : ce que le moteur fait seul ; le rôle de `phone:`), aide-mémoire (`Video`, `Sound` avec `volume` et `loop`), § 11 (sa ligne 1801 dit encore que « l'envoi d'un fichier » n'existe pas, alors qu'ADR-059 l'a fait).
- Leçons : `82-une-video-sous-titree.holo` (le mot nouveau) ; peut-être une leçon qui montre l'image qui s'adapte (rien à écrire, mais à voir). Index `exemples/lecons/README.md`.
- `docs/01-holocode/NOMS.md` : `subtitles`, et pourquoi pas `captions` ; corriger § 2 (lignes 210 et 229).
- `docs/01-holocode/COMPARAISON-WEB.md` : corriger les lignes 60-64 ; ajouter `track`, `srcset`, `sizes`, `loading`.
- `docs/01-holocode/TABLEAU-WEB.md` : lignes 499 (`srcset`) et 501 (sous-titres) ; puis republier le tableau en ligne.
- Décisions : compléter ADR-038 (`Video`) et ADR-042 (`Image`), ou une fiche nouvelle ; ADR-005 si le poids est vérifié.
- `moteur/README.md` : les mesures de poids du catalogue.
- Le journal (`docs/06-journal/JOURNAL.md`) et l'état dans `AGENTS.md`.

Relu le 2026-10-07 : 17 corrections.
