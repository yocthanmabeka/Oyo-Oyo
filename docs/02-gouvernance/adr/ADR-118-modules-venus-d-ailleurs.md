# ADR-118 — Des modules venus d'ailleurs, avec leur empreinte : `Module(from:, sha256:, license:)`

- Statut : ACCEPTÉ (Yocthan a dit « Oui » le 2026-10-09 à l'ouverture sous ces conditions)
- Date : 2026-10-10
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #250 (« des modules venus d'ailleurs, avec leur empreinte »), l'une des huit fonctions ouvertes sous conditions (issues 246 à 253) ; les deux avis, avec leurs sources : `proposals/Claude/contraintes-2026-10-09/README.md` (fonction 5 : event-stream en 2018, ua-parser-js en 2021, xz en 2024, S16 à S20 ; « la copie rangée dans le projet, avec son empreinte ») et `proposals/Gemini/contraintes-2026-10-09/README.md` (fonction 5 : les mêmes attaques, l'intégrité des sous-ressources du W3C, la boîte de WebAssembly ; « une copie locale ou une adresse HTTPS avec son empreinte ») ; `ADR-011` (partie C, les deux étages), `ADR-045` et `ADR-077` (les modules enfermés et leurs deux contrats), `ADR-116` (le chemin sûr vers un autre site, `holo serve`), `ADR-117` (rien vers l'autre site sans le choix du visiteur), `ADR-037` (l'écriture des noms), `ADR-017` (l'apparence dans les styles).
- Validation : Yocthan, le 2026-10-09, « Oui », à l'ouverture des huit fonctions, sous les conditions de l'issue.
- Projets affectés : HoloCode, HoloEngine, le serveur (`holo serve`)

## Contexte

- Un module (`ADR-045`, `ADR-077`) fait ce que HoloCode ne fait pas lui-même : un calcul lourd, une physique, une IA. Jusqu'ici, la page n'employait que des modules à soi : un fichier `.wasm` rangé à côté, sans rien dire d'où il venait.
- Un vrai projet emploie du code écrit par d'autres. Le web le fait de deux façons, et chacune a son défaut :
  - **le script d'un autre site** (un CDN) : le navigateur de chaque visiteur va le chercher chez lui. L'autre site apprend l'adresse IP de chaque visiteur, à chaque visite, et la page casse le jour où il disparaît. Le tribunal de Munich l'a jugé pour Google Fonts en 2022 (`ADR-116`). L'intégrité des sous-ressources (SRI) protège le fichier, pas le visiteur ;
  - **le gestionnaire de paquets** (npm) : une version ou un intervalle (`^1.2`), qui installe la dernière version permise, avec ses dépendances, et les leurs. Les trois attaques citées par les avis ont le même point commun : une nouvelle version, installée automatiquement, différente de ce qu'on croyait (event-stream, ua-parser-js, xz ; avis Claude, S16 à S20).
- La boîte des modules tient déjà : un fil à part, une mémoire plafonnée, un temps compté, ni réseau ni page. Mais elle ne regardait pas le fichier : un module fabriqué sans `--import-memory` aurait eu sa propre mémoire, hors du plafond ; un module qui demandait une fonction à la page échouait sans dire pourquoi.
- Les deux avis concluent : ouvrir sous conditions. Une copie rangée dans le projet, ou une adresse avec son empreinte ; toujours enfermé ; pas de catalogue à dépendances sans fin ; la licence et le poids dits. Et c'est le moteur qui tient les règles, jamais l'auteur.

## Décision

```holo
module "primes.wasm"
Page(
  title: "Primes",
  state: State(n: 1000, count: 0, refused: 0),
  modules: [
    Module(
      name: Primes,
      source: "primes.wasm",
      from: "https://modules.example.org/primes/1.0/primes.wasm",
      sha256: "128c62eaf00efe6eb9f2a1a2f95b311367166bfdae9c38c4a90cca7e8c8d0561",
      license: "MIT",
      input: n,
      output: count,
    ),
  ],
  children: [ Button(name: Go, text: "Count"), P("{count} prime numbers up to {n}") ],
  rules: [ On(Go.tap, effect: Primes.run), On(Primes.failed, effect: refused.set(1)) ],
)
```

1. **Trois mots de plus pour `Module`**, et le reste ne change pas (`source`, `input`, `output`, `time`, `memory`, `run`, `done`, `failed`) :
   - `sha256:` : l'empreinte du fichier, en 64 chiffres hexadécimaux, celle que donne l'auteur du module ;
   - `from:` : l'adresse d'où il vient, en HTTPS, un fichier `.wasm` ;
   - `license:` : sa licence, telle que son auteur la donne.
2. **La copie est toujours rangée à côté de la page** : `source:`, comme pour tout module. C'est elle, et seulement elle, que reçoit le navigateur du visiteur. Les deux formes de l'issue sont donc une seule :
   - la copie rangée dans le projet : `source:` et `sha256:` (et `license:`) ;
   - une adresse avec son empreinte : `from:` en plus. Si la copie manque, **`holo serve` la télécharge une fois**, à la première demande, par le chemin sûr d'`ADR-116` ; il vérifie son empreinte et ce qu'elle demande à la boîte, la range à côté de la page, et son journal le dit (« garde-la avec ton projet »). Le navigateur du visiteur ne va jamais chez l'autre site.
3. **L'empreinte est vérifiée avant chaque lancement**, à trois endroits :
   - **par le navigateur lui-même** : la page demande la copie avec `fetch(…, { integrity: "sha256-…" })`, l'intégrité des sous-ressources du W3C. Le navigateur refuse de donner un fichier qui diffère ; il le fait à chaque lecture, même depuis son cache, et aussi dans un contexte non sûr (voir plus bas) ;
   - **par `holo check`** : il refuse une copie qui a changé, et dit son empreinte réelle ;
   - **par `holo serve`**, avant de ranger une copie téléchargée : une copie qui ne correspond pas n'est jamais rangée.
4. **La même boîte, et rien de plus.** Avant le fil à part, le moteur lit le fichier (`modules::check_wasm`, les mêmes règles dans la page, dans `holo check` et dans `holo serve`). Il refuse un fichier :
   - qui importe autre chose que `env.memory` : une fonction (le réseau, la page, l'heure, un autre module…), une table, une valeur globale ;
   - qui fabrique sa propre mémoire (elle échapperait à son plafond), ou demande une mémoire partagée, de 64 bits, ou plus grande que son plafond ;
   - qui emploie des objets gérés par le navigateur (WasmGC), qui vivraient hors de sa mémoire ;
   - qui a une table sans plafond, ou de plus de 100 000 éléments ;
   - qui n'offre pas `run`, ou une fonction `run` d'aucun des deux contrats.

   Un module n'importe que sa mémoire : il ne peut donc jamais en charger un autre. Pas de dépendances en chaîne, pas de catalogue. Ces règles valent pour tous les modules, ceux d'ici comme ceux d'ailleurs : les sept modules des leçons les suivent déjà.
5. **Ce qui est dit, et à qui** :
   - **aux visiteurs**, en bas de la page, sans JavaScript aussi : « Module « primes.wasm », venu de modules.example.org — licence : MIT ». Un repère nommé (`aside`, « Modules de cette page »), du texte : le lecteur d'écran le lit. Si le moteur refuse de le lancer, la même ligne dit pourquoi (« refusé : ce fichier n'est pas celui que l'auteur a vérifié »), et le lecteur d'écran l'annonce (`role="status"`). Dans la langue de la page ;
   - **à l'auteur**, par `holo check` : « module primes.wasm : 205 octets ; premier contrat, run(nombre) ; licence : MIT ; empreinte vérifiée ; venu de https://… ». Le poids est mesuré, jamais écrit à la main. L'éditeur (`holo check -`) montre les mêmes erreurs ;
   - **à l'auteur**, par le journal de `holo serve`, quand il télécharge une copie, ou qu'il la refuse.

## Les règles de sécurité, tenues par le moteur

| Règle | Comment | Essai (ce qui doit être refusé l'est) |
|---|---|---|
| 1. Une adresse ne va jamais sans son empreinte ; une empreinte, jamais sans sa licence | `a_module` refuse `from:` sans `sha256:`, et `sha256:` sans `license:`, avec la raison, partout où la page est lue (`holo check`, la page, le serveur) | `a_module_from_elsewhere_says_its_fingerprint_its_address_and_its_license` : 26 refus, dont une adresse sans empreinte, une empreinte sans licence, une empreinte de 8 chiffres, avec `zz`, en `sha256-…` (le base64 du web) ou `sha256:…`, un nombre ; `http://`, une adresse IP, `localhost`, un port, `nom@`, `{n}`, `.js`, `.wasm.html` ; une licence vide, d'espaces, de 65 caractères, avec `{n}`, sans guillemets ; `version: "^1.2"` ; une copie dans un dossier ; un même fichier avec deux empreintes |
| 2. Le navigateur vérifie l'empreinte avant chaque lancement, même dans un contexte non sûr | `module_info` donne l'empreinte écrite pour SRI (`sha256-` et le base64 des 32 octets) ; la page demande la copie avec `fetch(…, { integrity })`. Refusée : `Nom.failed`, et le bas de la page le dit | `the_fingerprint_is_written_for_the_integrity_of_fetch` (les vecteurs de FIPS 180-2 ; une empreinte mal écrite ne donne jamais une intégrité que le navigateur ignorerait) ; dans Chrome, la page ouverte sur `http://192.0.2.2:…` (`isSecureContext` faux, `crypto.subtle` absent) : la bonne copie tourne, une copie remplacée (la somme de la leçon 69 au lieu des nombres premiers) n'est jamais lancée |
| 3. Le navigateur du visiteur ne va jamais chez l'autre site | la page ne connaît que la copie (`source:`) ; `from:` ne sert qu'à `holo serve` | dans Chrome, chaque demande du navigateur est comptée (le réseau) et toute demande vers l'autre site serait arrêtée (Fetch) : **zéro** ; le faux autre site ne voit que `holo serve` (son `User-Agent`, ni cookie ni `Referer`) |
| 4. `holo serve` télécharge une fois, par le chemin sûr, et ne range que ce qui correspond | `remote::download` : l'adresse lue strictement ; le nom résolu, chaque adresse vérifiée, la connexion faite à l'adresse vérifiée (`SafeResolver`, `ADR-116`) ; ni proxy, ni cookie, ni compression ; 4 Mo au plus, coupés au-delà ; 4 s pour se connecter, 60 s en tout ; cinq redirections au plus, chacune en HTTPS et vérifiée comme la première. Puis l'empreinte et la boîte (`check_copy`) ; rangée par un fichier caché, renommé d'un coup ; jamais un fichier remplacé. Une seule à la fois ; un échec gardé une minute par adresse | `holo_serve_downloads_a_missing_copy_once_checks_it_and_keeps_it_next_to_the_page`, `a_downloaded_copy_that_does_not_match_is_never_kept_and_the_other_site_is_not_hammered`, `the_download_takes_the_safe_road_of_the_other_sites` (le vrai client contre un faux site sur ce PC : une redirection suivie, une boucle refusée, 4 Mo coupés avec ou sans taille annoncée, 404), `holo_serve_answers_with_the_downloaded_copy_and_the_page_tells_its_license` ; dans Chrome, **une** demande à l'autre site pour deux lancements et un rechargement ; un fichier changé en silence chez l'autre site n'est pas rangé |
| 5. Rien de plus que sa mémoire | `check_wasm` lit les sections du fichier (types, importations, tables, mémoire, exportations) avant le fil à part, dans la page et sur le PC | `the_box_refuses_a_file_that_asks_for_more_than_its_memory` : les sept modules des leçons acceptés, avec leur contrat ; 19 refus, dont `env.fetch`, une fonction de WASI, une valeur globale, une table importée, une seconde mémoire, `autre.memory`, une mémoire à soi, 17 pages pour un plafond de 16, une mémoire qui veut moins que la boîte, une mémoire partagée, un `struct` de WasmGC, un `anyref`, une table sans plafond, une table de 199 936 éléments, pas de `run`, un `run` à trois paramètres, une section inconnue, une page HTML, un fichier coupé ; dans Chrome, le module qui demande `env.fetch` est refusé avant de tourner, et le bas de la page dit pourquoi |
| 6. `holo check` refuse ce que la page refuserait | la copie lue (4 Mo au plus), son empreinte, `check_wasm` ; une copie qui manque sans adresse ; l'erreur à la ligne du `Module(…)` | `holo_check_says_the_weight_the_license_and_refuses_a_copy_that_changed` |

## L'empreinte dans le navigateur, et l'expérience faite

- `crypto.subtle.digest("SHA-256", …)` n'existe que dans un contexte sûr : HTTPS, ou `localhost`. Sur `http://192.168.…`, quand on essaie la page sur un téléphone du même Wi-Fi, il n'existe pas.
- L'intégrité de `fetch` ne dépend pas de `crypto.subtle` : le navigateur calcule l'empreinte lui-même.
- L'expérience, le 2026-10-10, dans le Chromium 1194 du conteneur, la page servie sur l'adresse du réseau (`http://192.0.2.2:…`) puis sur `127.0.0.1` :
  - sur `192.0.2.2` : `isSecureContext` faux, `crypto.subtle` absent ; le bon fichier arrive (88 octets), un fichier qui diffère est refusé (« Failed to fetch ») ;
  - sur `127.0.0.1` : la même chose, avec `crypto.subtle` présent.
- **Décision** : dans un contexte non sûr, rien ne change. L'empreinte est vérifiée par le navigateur, et un fichier qui diffère n'est jamais lancé. Si un navigateur ne savait pas vérifier l'intégrité, `fetch` échouerait : le module ne tournerait pas (`failed`). Jamais un module non vérifié.
- Le navigateur ne dit pas pourquoi `fetch` a échoué (une empreinte qui diffère, ou pas de réseau). La page redemande alors le fichier, sans le lire (`HEAD`) : s'il est là, c'est qu'il a changé, et le bas de la page le dit.
- L'essai Chrome l'éprouve à chaque fois : la page y est ouverte sur l'adresse de la machine sur son réseau, un contexte non sûr.

## Les bibliothèques

**Aucune bibliothèque nouvelle.**

- `ureq` (accepté par Yocthan le 2026-10-10 pour `holo serve` seulement, `ADR-116`) sert aussi à télécharger un module, toujours dans `holo serve`, et nulle part ailleurs : ni dans une autre commande de `holo`, ni dans le WebAssembly. Le même client, réglé de même, avec deux différences, écrites et justifiées plus haut : cinq redirections et une minute.
- `sha2` (déjà là, sur le PC seulement, pour les clés d'accès) calcule l'empreinte dans `holo check` et `holo serve`. Rien dans le WebAssembly : dans la page, c'est le navigateur qui calcule l'empreinte.
- Le WebAssembly de la page ne reçoit aucune bibliothèque. Il reçoit `check_wasm` (une lecture des sections du fichier, écrite à la main, comme le reste du moteur), les trois mots, la mention aux visiteurs. Le moteur léger grossit de 18,7 Ko (6,9 Ko compressé : de 285 à 292 Ko), le moteur entier de 8 Ko compressé.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Qui va chercher le module | le navigateur du visiteur, chez l'autre site, avec SRI (l'avis Gemini, comme un CDN) ; une commande `holo add` qui télécharge et écrit un fichier verrou (l'avis Claude) ; l'auteur, à la main ; **`holo serve`, une fois, à la première demande** | le navigateur donnerait l'adresse IP de chaque visiteur à l'autre site, à chaque visite, et la page casserait le jour où il disparaît : le défaut des CDN. `holo add` demanderait `ureq` dans une autre commande que `holo serve` (non accepté), et un fichier de plus à comprendre. À la main, l'auteur peut toujours le faire (la copie seule suffit). `holo serve` a déjà le chemin sûr (`ADR-116`) ; la copie qu'il range est un fichier du projet comme un autre |
| Où ranger la copie | `holo-data/` (jamais servi, jamais versionné) ; un dossier `modules/` ; **à côté de la page, sous le nom de `source:`** | dans `holo-data/`, un projet copié ailleurs perdrait le module, et devrait le retélécharger d'un site qui a peut-être disparu : le défaut des CDN, chez l'auteur. À côté de la page, la copie est versionnée avec le projet, servie par n'importe quel serveur, et lue hors ligne ; c'est déjà là que vit un module (`ADR-045`). Sans dossier : la page qui la déclare est dans le même dossier, `holo serve` la trouve sans chercher partout |
| Le mot de l'empreinte | `hash: "sha256-…"` (l'avis Gemini) ; `integrity: "sha384-…"` (HTML, npm : du base64) ; `fingerprint:` ; `checksum:` (Cargo) ; **`sha256:`, en hexadécimal** | ce qu'un débutant a sous les yeux : la page de téléchargement d'un module, `sha256sum`, `Get-FileHash` (Windows), `shasum -a 256` (macOS), GitHub, qui écrivent tous l'empreinte SHA-256 en 64 chiffres hexadécimaux. Le base64 de SRI ne se calcule pas avec ces outils : le moteur l'écrit lui-même pour le navigateur. Le nom dit l'algorithme : si SHA-256 faiblit un jour, un autre mot viendra, sans ambiguïté. `hash` et `checksum` sont des mots de programmeur ; `checksum` fait penser à une erreur de transmission, pas à une ruse |
| Le mot de l'adresse | `url:` ; `source: "https://…"` (un seul mot pour les deux, comme `Data(from:)`) ; **`from:`** | `source:` est déjà la copie à côté de la page, et doit le rester : c'est elle que reçoit le visiteur. `from:` dit d'où vient le contenu, comme `Data(from:)` et `Embed(from:)` |
| Une version | `version: "1.2.0"` ; un intervalle (`^1.2`) ; « la dernière » ; **une empreinte exacte** | une version ou un intervalle est exactement ce qui a laissé passer event-stream, ua-parser-js et xz : une nouvelle version, prise sans qu'on la regarde. L'empreinte exacte ne laisse passer que le fichier vérifié |
| Les dépendances | un module qui en importe d'autres (le lien dynamique, un catalogue qui les résout) ; **aucune : un module n'importe que sa mémoire** | « pas de catalogue automatique à dépendances sans fin ». Un module qui a besoin d'un autre code l'inclut dans son fichier : son empreinte le couvre aussi |
| La licence | dans un fichier verrou (l'avis Claude) ; lue dans le fichier (WebAssembly n'a pas de place pour elle) ; **`license:` dans la page** | une ligne de la page, qu'on lit avec le reste ; obligatoire avec une empreinte, pour que l'auteur la cherche (« la licence interdit l'usage commercial, et l'auteur ne le sait pas », avis Claude). Le moteur ne peut pas vérifier qu'elle est vraie : il la dit |
| À qui dire la licence | à l'auteur seulement ; **à l'auteur et aux visiteurs** | le navigateur de chaque visiteur reçoit une copie du module : la plupart des licences (MIT, BSD, Apache) demandent que leur mention accompagne les copies. Et le visiteur sait quel code venu d'ailleurs tourne chez lui |
| Où le dire aux visiteurs | rien d'automatique (l'auteur le fait s'il y pense) ; un bloc `Credits()` à poser ; une valeur `{Primes.license}` ; **une ligne du moteur, en bas de la page** | le moteur tient la règle, l'auteur ne peut pas l'oublier. Un bloc ou une valeur demanderaient un mot de plus. La ligne est un repère nommé, du texte, présent sans JavaScript ; elle suit la police et les couleurs de la page (`ADR-017`) |
| Le poids | écrit par l'auteur (`weight: 20KB`, l'avis Gemini pour `Embed`) ; **mesuré, dit à l'auteur** ; dit aussi aux visiteurs | un poids écrit à la main ment dès que le fichier change. Le moteur le mesure (`holo check`, le journal de `holo serve`) et le borne à 4 Mo. La page ne connaît pas le fichier avant de le télécharger : le dire aux visiteurs demanderait une demande de plus à chaque visite. Le visiteur ne télécharge le module que lorsqu'il tourne |
| Vérifier dans le navigateur | ne rien vérifier (faire confiance au serveur) ; `crypto.subtle.digest` (seulement en contexte sûr) ; une empreinte calculée par le moteur en WebAssembly (une bibliothèque de plus, ou de la cryptographie écrite à la main : refusé, `ADR-081`) ; **l'intégrité de `fetch`** | un serveur intermédiaire, un cache ou un hébergeur peuvent changer un fichier ; le navigateur doit vérifier. SRI est fait pour cela, marche dans un contexte non sûr, et n'ajoute rien au moteur |
| Lire le fichier avant de le lancer | se fier à l'échec de `WebAssembly.instantiate` ; lire seulement les importations (`WebAssembly.Module.imports`) ; **lire les sections du fichier, dans le moteur** | l'échec ne dit pas pourquoi. Les importations seules ne voient pas une mémoire à soi, ni une seconde mémoire (le multi-mémoire est dans Chrome depuis 2023), ni WasmGC. Le moteur en Rust lit le fichier : les mêmes règles dans la page, `holo check` et `holo serve` |
| Une liste de sites permis | `holo-data/sites.txt` (`ADR-116`) ; `Page(embeds:)` (`ADR-117`) ; **aucune : l'empreinte décide** | pour les données, la liste range les clés et dit quels sites le serveur lit, puisque le contenu change ; un module n'a pas de clé, et son contenu est épinglé. Le réseau privé reste refusé (`SafeResolver`) |
| Les redirections | aucune (`ADR-116`) ; **cinq au plus, en HTTPS, chacune vérifiée** | les fichiers d'une version sur GitHub passent toujours par une redirection, vers une adresse signée qui change ; on ne peut pas l'écrire dans la page. Le fichier est épinglé par son empreinte : une redirection ne peut pas en changer le contenu. Chaque saut repasse par `SafeResolver` et doit être en HTTPS |
| Le faux « autre site » des essais | permettre `http://127.0.0.1` dans les pages, le temps d'un essai ; **l'interrupteur d'`ADR-116`** (`HOLO_TEST_ONLY_INSECURE_SITE`, un nom en `.test`) | rien ne change dans la lecture des adresses ; l'essai Chrome arrête et compte en plus toute demande du navigateur vers l'autre site |

## Ce qui est refusé, et pourquoi

- **Dans la page**, avec sa raison, par `holo check` :
  - une adresse (`from:`) sans empreinte : on ne saurait pas ce qu'on reçoit ;
  - une empreinte sans licence ;
  - une empreinte qui n'a pas 64 chiffres hexadécimaux, ou écrite `sha256-…` (le base64 du web) ou `sha256:…` ;
  - une adresse qui n'est pas en HTTPS, une adresse IP, `localhost`, un port, `nom@`, une valeur `{…}`, une adresse qui ne finit pas par `.wasm` ;
  - une licence vide, de plus de 64 caractères, sur deux lignes, ou qui lit une valeur ;
  - la copie d'un module venu d'une adresse rangée dans un dossier ;
  - un même fichier avec deux empreintes, ou deux adresses ;
  - un paramètre inconnu (`version:`, `hash:`…).
- **Dans le fichier** (dans la page, `holo check`, `holo serve`) : voir la règle 5 ; plus de 4 Mo.
- **Une copie qui a changé** : `holo check` la refuse, avec son empreinte réelle et ce qu'il faut faire (« si tu ne l'as pas changé toi-même, efface-le : holo serve retéléchargera le bon »). Le navigateur ne la donne pas à la page ; le module ne tourne pas.
- **Un téléchargement** : un fichier qui ne correspond pas à l'empreinte, ou qui demande plus que sa mémoire, n'est jamais rangé ; une adresse qui mène au réseau privé, trop lente, trop lourde, un code autre que 200, plus de cinq redirections, une redirection hors de HTTPS. Le journal dit la raison.

## Les défauts du web évités

- **Le script chargé chez un autre site par chaque visiteur** (le CDN) : ici, le navigateur ne reçoit que la copie de l'auteur.
- **La page qui casse quand le CDN disparaît** : ici, la copie est dans le projet.
- **SRI oubliée**, ou mise sur un fichier qui change à chaque version : ici, une adresse sans empreinte est refusée, et l'empreinte est l'adresse exacte du contenu.
- **La version qui change en silence** (`^1.2`, « latest ») : ici, une empreinte exacte, rien d'autre.
- **Les dépendances des dépendances** (`node_modules`), et les scripts qui tournent à l'installation : ici, un fichier, qui n'importe que sa mémoire, et rien ne tourne avant le lancement.
- **Le code d'un autre qui lit la page, les cookies, le réseau** : ici, la boîte ne lui donne que sa mémoire et ses valeurs.
- **La licence qu'on ne lit jamais** : ici, elle s'écrit dans la page, et elle est dite.

## Limites, honnêtement

- **L'empreinte protège d'un fichier changé, pas d'un premier fichier déjà piégé** (avis Claude). La boîte borne alors le mal : un module piégé peut rendre de fausses valeurs (relues avec méfiance, `ADR-077`) et user son temps et sa mémoire, rien de plus.
- **La licence est celle que l'auteur écrit** : le moteur ne peut pas vérifier qu'elle est vraie.
- **Le premier visiteur attend le téléchargement** (une minute au plus), une seule fois ; pendant ce temps, une autre demande du même fichier reçoit 404 (le module échoue, et réussira au prochain lancement).
- **Le serveur d'essai de Node ne télécharge rien** : la copie doit être là.
- **Aucune mise à jour** : si l'auteur du module publie une nouvelle version, rien ne le dit. Changer de version, c'est changer l'adresse et l'empreinte, puis effacer l'ancienne copie.
- **Les modules faits pour WasmGC** (Kotlin, Dart compilés pour le web), les mémoires de 64 bits et les fils (`threads`) sont refusés : la boîte ne sait pas les plafonner.
- **La leçon 141 dit d'où vient son module** : l'adresse de son fichier dans le dépôt de HoloCode, sur GitHub (`raw.githubusercontent.com`). Elle ne répond qu'une fois la leçon dans `main`. Sa licence est celle du dépôt : « Tous droits réservés ».
- **Le poids n'est pas dit aux visiteurs** (voir la comparaison).
- **L'éditeur** (`holo check -`) montre les erreurs des fichiers des modules, pas leurs lignes d'information (le poids, la licence).

## Dettes

- Dire à l'auteur quand l'adresse d'un module sert un autre fichier (une nouvelle version), avec son accord pour la prendre : demanderait `ureq` dans une autre commande que `holo serve`.
- Le poids aux visiteurs, sans demande de plus.
- Le téléchargement dans le serveur d'essai de Node.
- Le grand tableau du web (`COMPARAISON-WEB.md`, `script`) sera mis à jour avec les autres fonctions ouvertes.

## Critères de validation

- Tests du moteur (`cargo test --release --locked`) :
  - `modules::foreign_tests` : `a_module_from_elsewhere_says_its_fingerprint_its_address_and_its_license`, `the_fingerprint_is_written_for_the_integrity_of_fetch`, `the_box_refuses_a_file_that_asks_for_more_than_its_memory`, `the_visitors_read_the_license_and_where_the_module_comes_from` ;
  - `copies::copies_tests` : `holo_check_says_the_weight_the_license_and_refuses_a_copy_that_changed`, `holo_serve_downloads_a_missing_copy_once_checks_it_and_keeps_it_next_to_the_page`, `a_downloaded_copy_that_does_not_match_is_never_kept_and_the_other_site_is_not_hammered`, `the_download_takes_the_safe_road_of_the_other_sites`, `holo_serve_answers_with_the_downloaded_copy_and_the_page_tells_its_license` ;
  - les essais des modules d'avant (`a_module_is_declared_and_announced`, `a_module_receives_and_returns_several_values`), inchangés.
- Les essais savent échouer. Chaque mutation a été faite puis retirée :
  - dans le moteur, une adresse acceptée sans empreinte : `a_module_from_elsewhere_says_its_fingerprint_its_address_and_its_license` rate ;
  - dans le moteur, toute importation acceptée : `the_box_refuses_a_file_that_asks_for_more_than_its_memory` rate ;
  - dans Chrome, la copie demandée sans son empreinte : la copie remplacée tourne (« jamais lancée : false ») ;
  - dans Chrome, le fichier lancé sans être lu : le module qui demande `env.fetch` n'échoue que par l'erreur brute du navigateur, et le visiteur n'en sait rien ;
  - dans `holo serve`, la copie téléchargée rangée sans être vérifiée : le fichier changé en silence chez l'autre site est rangé.
- Dans Chrome : « un module venu d'ailleurs : son empreinte vérifiée par le navigateur avant chaque lancement, même sur une adresse du réseau local ; holo serve le télécharge une fois, jamais le navigateur ; ce qui demande plus que sa mémoire est refusé ; sa licence dite aux visiteurs (leçon 141, serve) ».
- `holo check` sur la leçon : `ok`, puis « module 141-premiers.wasm : 205 octets ; premier contrat, run(nombre) ; licence : Tous droits réservés ; empreinte vérifiée ; venu de https://raw.githubusercontent.com/… ».
- La leçon `141-un-module-venu-d-ailleurs.holo`, et son module, `modules/premiers.rs` (205 octets une fois compilé).
