# ADR-116 — Les données d'un autre site, lues par le serveur de l'auteur : `Data(from: "https://…")`

- Statut : ACCEPTÉ (Yocthan a dit « Oui » le 2026-10-09 à l'ouverture sous ces conditions)
- Date : 2026-10-10
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #248 (« les données d'un autre serveur, que le serveur de l'auteur va chercher »), l'une des huit fonctions ouvertes sous conditions (issues 246 à 253) ; les deux avis, avec leurs sources : `proposals/Claude/contraintes-2026-10-09/README.md` (fonction 3 : le jugement de Munich du 20 janvier 2022 sur Google Fonts, S11 à S13) et `proposals/Gemini/contraintes-2026-10-09/README.md` (fonction 3 : CORS, l'arrêt Fashion ID) ; `ADR-030` (les données d'un fichier, qui excluait un autre serveur), `ADR-051` (les listes reçues), `ADR-064` (arrivées ou échec), `ADR-074` (`holo serve`), `ADR-082` (une bibliothèque expliquée en clair).
- Validation : Yocthan, le 2026-10-09, « Oui », à l'ouverture des huit fonctions, sous les conditions de l'issue.
- Projets affectés : HoloCode, HoloEngine, le serveur (`holo serve`)

## Contexte

- `Data(from: "stock.json")` (`ADR-030`) ne lisait qu'un fichier rangé à côté de la page. Un autre serveur était exclu : une page qui contacte elle-même un autre site lui dit qui la lit et quand.
- Un vrai site montre pourtant les données d'un autre : la météo, un cours, le résumé d'un article.
- Si c'est le navigateur du visiteur qui va les chercher :
  - l'autre site reçoit l'adresse IP du visiteur, une donnée personnelle. Le 20 janvier 2022, le tribunal de Munich a condamné un site qui faisait charger Google Fonts par ses visiteurs, parce qu'il pouvait servir les polices lui-même (avis Claude, S11 à S13) ;
  - CORS bloque la plupart des services ;
  - une clé d'API écrite dans la page est publique.
- Les deux avis disent la même chose : le serveur de l'auteur va chercher les données, et le moteur tient les règles de sécurité.
- Un serveur qui va chercher une adresse a ses propres dangers :
  - qu'on lui fasse lire le réseau privé : la box, une imprimante, `169.254.169.254` dans un nuage (SSRF) ;
  - un nom qui change d'adresse entre la vérification et la connexion (DNS rebinding) ;
  - une redirection vers une adresse interne ;
  - une réponse lente ou sans fin ;
  - des visiteurs qui font marteler l'autre site ;
  - une clé qui fuit dans la page, une erreur ou le journal.

## Décision

```holo
Page(
  title: "Kinshasa",
  state: State(loading: 1, broken: 0, title: "", extract: ""),
  data: Data(name: Wiki, from: "https://fr.wikipedia.org/api/rest_v1/page/summary/Kinshasa", every: 3600s),
  children: [
    If(loading, is: 1, children: [ P("Loading…") ]),
    If(broken, is: 1, children: [ P("The summary did not arrive."), Button(name: Retry, text: "Try again") ]),
    H2("{title}"),
    P("{extract}"),
  ],
  rules: [
    On(Wiki.done, effect: [loading.set(0), broken.set(0)]),
    On(Wiki.failed, effect: [loading.set(0), broken.set(1)]),
    On(Retry.tap, effect: [loading.set(1), broken.set(0), Wiki.refresh]),
  ],
)
```

Et, dans le dossier servi, le fichier `holo-data/sites.txt` :

```text
fr.wikipedia.org
api.exemple.org ?appid=ta-clé
autre.exemple.org X-Api-Key: ta-clé
```

1. **`Data(from: "https://…")`** : le même bloc, les mêmes mots (`name`, `every`, `done`, `failed`, `refresh`). Seule l'adresse change.
2. **Le serveur de l'auteur lit l'autre site, jamais le navigateur du visiteur.**
   - Le moteur de la page demande les données à son propre serveur, à sa propre adresse : `/kinshasa.holo?remote-data`.
   - Le visiteur ne nomme jamais l'adresse de l'autre site : elle vient de la page, écrite par l'auteur.
   - L'autre site voit le serveur de l'auteur, jamais le visiteur.
3. **Les sites permis, et leurs clés**, dans `holo-data/sites.txt` : un site par ligne, son nom exact.
   - Si le site demande une clé, elle suit sur la même ligne, comme sa documentation la montre : `?appid=…` (un paramètre de l'adresse) ou `X-Api-Key: …` (un en-tête).
   - La page ne nomme jamais une clé. Le dossier `holo-data/` n'est jamais servi, ni versionné.
   - Le fichier se relit quand il change ; le journal dit ce qui ne va pas.
4. **Ce qui arrive est gardé.**
   - Chaque adresse est demandée au plus une fois par `every` (une minute au moins ; dix minutes sans `every`), quel que soit le nombre de visiteurs.
   - Un échec est gardé une minute. Une seule demande à la fois pour une même adresse : les autres visiteurs l'attendent.
   - `refresh`, sans ou avec JavaScript, relit ce que le serveur garde : un visiteur ne force jamais une demande.
5. **Ce qui arrive est vérifié, puis réduit.** C'est un objet JSON, comme le fichier de `Data` (64 Ko, trois niveaux). Puis le serveur ne garde, pour la page, que ce qu'elle déclare : ses valeurs, ses textes, ses listes et leurs champs. Le reste de la réponse (un numéro de compte, l'adresse IP du serveur) ne part ni dans la page ni vers le navigateur.
6. **Sans JavaScript**, la page arrive avec ses données (`Wiki.done`), ou dit l'échec (`Wiki.failed`). « Réessayer » relit ce qui est gardé.
7. **Une panne donne `failed`** (`ADR-064`). Le visiteur reçoit `502`, sans détail ; la raison va au journal de l'auteur, en clair.
8. **Au démarrage**, `holo serve` dit :
   - les sites permis, jamais leur clé ;
   - ce qui ne va pas dans `holo-data/sites.txt` ;
   - chaque page qui lit un site non permis, ou dont la clé manque.

## Les règles de sécurité, tenues par le moteur

| Règle | Comment | Essai |
|---|---|---|
| 1. HTTPS seulement, vers un site déclaré, comparé exactement ; jamais une adresse IP à la place d'un nom | le moteur lit l'adresse strictement (dans `holo check`, la page et le serveur) : `https://`, un nom de site, ni port, ni `nom:mot-de-passe@`, ni `#`, ni `{…}` ; le dernier morceau du nom commence par une lettre (`127.1`, `0x7f.0.0.1`, `2130706433` sont refusés) ; le serveur compare le nom exact (aucun sous-domaine deviné) | `another_site_is_https_and_a_name_never_an_ip`, `only_a_declared_site_is_read_compared_exactly`, `the_sites_file_is_read_strictly_and_never_repeats_a_key` |
| 2. Rien vers ce PC ni le réseau privé, IPv4 et IPv6 ; connexion à l'adresse vérifiée | le résolveur résout le nom une fois, vérifie chaque adresse (bouclage, privé, CGNAT, lien local, multicast, réservé, documentation ; une IPv4 portée par une IPv6, `::ffff:` ou NAT64, est jugée comme l'IPv4) et rend seulement des adresses vérifiées ; une seule adresse privée suffit à tout refuser ; ureq se connecte à celles-là, sans seconde résolution | `this_pc_and_private_networks_are_refused_in_ipv4_and_ipv6`, `a_name_that_leads_here_is_refused_before_any_connection`, `the_connection_goes_to_the_checked_address_even_if_the_name_changes_afterwards` |
| 3. Aucune redirection suivie | ureq réglé à zéro redirection ; une réponse 3xx donne `failed`, et le journal dit d'écrire l'adresse finale dans la page | `redirects_are_never_followed`, et dans Chrome |
| 4. Des bornes | 4 s pour se connecter, 8 s en tout ; 64 Ko lus au plus, coupés au-delà (une taille annoncée trop grande est refusée avant de lire) ; un objet JSON de trois niveaux au plus ; 32 sites, 64 adresses gardées, 8 par site | `an_answer_is_bounded_in_time_size_and_json`, `the_kept_addresses_are_bounded` |
| 5. Gardé un moment ; un visiteur ne force jamais une demande | une demande au plus par adresse et par `every` (une minute au moins) ; un échec, une minute ; une place du cache ne sert qu'une demande par minute, donc 8 demandes par minute au plus vers un même site | `each_address_is_asked_at_most_once_per_interval_whatever_the_visitors`, `another_site_is_read_by_this_server_with_and_without_javascript`, et dans Chrome (une demande pour sept lectures) |
| 6. Les clés ne sortent jamais du serveur | rangées sur la ligne de leur site, envoyées seulement à lui ; jamais écrites (pas même par `Debug`), ni dans un message, ni au journal ; une clé vide ou mal écrite : `failed` sans demande, et le démarrage le dit ; une adresse de page qui contient la clé, ou son paramètre : refusée ; une réponse qui recopie la clé : refusée | `a_key_goes_only_to_its_site_and_never_into_a_page_a_message_or_the_journal`, `the_server_says_at_startup_what_is_permitted_and_what_is_missing` |
| 7. Un `User-Agent` honnête ; ni cookie, ni rien du visiteur | la demande est faite de rien : `User-Agent: HoloCode/0.1.0 (holo serve; +https://github.com/yocthanmabeka/Oyo-Oyo)`, `Accept: application/json`, et la clé ; ureq est compilé sans cookies ni compression ; aucun proxy, même si l'environnement en donne un | `the_request_says_who_asks_and_carries_nothing_of_the_visitor`, et dans Chrome |

## La bibliothèque, et pourquoi (en clair)

Un client HTTPS ne s'écrit pas à la main : TLS est de la cryptographie (`ADR-081`). Une seule bibliothèque, compilée **seulement pour le serveur**, jamais dans le moteur de la page :

| Bibliothèque | Ce qu'elle fait ici | Pourquoi elle |
|---|---|---|
| `ureq` 3.4.2 (`=3.4.2`), avec la seule fonction `rustls` | une demande GET en HTTPS, sans rien d'asynchrone | petite, très employée (plus de 220 millions de téléchargements), entretenue (septembre 2026) ; elle laisse remplacer la résolution des noms, ce qu'exige la règle 2 |

Elle amène `rustls` 0.23 (TLS écrit en Rust, sans OpenSSL), `ring` 0.17 (sa cryptographie), `webpki-roots` (les certificats racines de Mozilla : le même résultat sous Windows et sous Linux), `rustls-webpki`, `ureq-proto`, `http`, `httparse` et quelques autres petites bibliothèques, toutes dans `Cargo.lock` : la construction avec `--locked` passe.

- `ring` compile un peu de C, comme `rusqlite` le fait déjà pour SQLite : aucun outil de plus à installer.
- Les réglages : aucun proxy, aucune redirection, les délais, aucune connexion gardée ouverte, notre `User-Agent`.
- La résolution des noms est la nôtre (`SafeResolver`). Elle est dans une partie de ureq qui peut changer d'une version à l'autre (« unversioned ») : la version fixée l'empêche de changer en silence.

Écartées :

- `ureq` 2.12 : plus mise à jour depuis décembre 2024, et elle amène `url` et `idna`, lourds.
- `reqwest` : un moteur asynchrone entier (`tokio`, `hyper`) pour une demande par minute.
- `curl` : une bibliothèque en C, avec OpenSSL.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Qui lit l'autre site | le navigateur du visiteur ; un service de relais (un prestataire) ; **le serveur de l'auteur** | le navigateur donnerait l'adresse IP du visiteur, ses cookies et la page d'où il vient, et bute sur CORS ; un prestataire est exclu (« chez soi d'abord ») |
| Où déclarer les sites permis | dans la page (`Allowed(sources: […])`, l'avis Claude ; `Page(sites:)`) ; sur la ligne de commande ou dans l'environnement (`holo serve --allow …`, comme Deno) ; **un fichier, `holo-data/sites.txt`** | dans la page, la liste redirait l'adresse déjà écrite, et une page (ou un morceau importé) pourrait l'élargir ; une ligne de commande se retape à chaque démarrage ; le fichier se lit d'un coup d'œil, n'est jamais servi ni versionné, et l'auteur seul l'écrit |
| Comment nommer une clé rangée sur le serveur | un nom dans la page, `key: secret("METEO_KEY")` (l'avis Claude), et sa valeur dans une variable d'environnement ; un fichier par clé dans `holo-data/` ; **par son site, sur la ligne du site** | une clé nommée par la page peut être envoyée par la page à n'importe quel site permis, et lire une variable d'environnement au choix d'une page (`PATH`…) est un danger ; une variable d'environnement se redonne à chaque démarrage, ce qui est difficile pour un débutant sous Windows ; sur la ligne du site, la clé ne peut aller nulle part ailleurs, et la page n'en sait rien |
| Comment l'insérer dans la demande | un en-tête ; un paramètre de l'adresse ; **les deux, comme la documentation du site le montre** | la plupart des services pour débutants veulent un paramètre (`appid`, `key`) ; d'autres, un en-tête (`X-Api-Key`, `Authorization: Bearer …`). L'en-tête est plus sûr (une adresse se retrouve dans des journaux) : la leçon le conseille quand le site l'accepte. Un en-tête du protocole ou du visiteur (`Cookie`, `Host`, `User-Agent`…) est refusé |
| Le temps gardé | `keep: 10min` (l'avis Claude) ; `cached: 15min` (l'avis Gemini) ; **`every:`, déjà là** | `keep:` dit déjà « les valeurs gardées d'une visite à l'autre » ; `cached` est un mot de programmeur ; `every: 600s` dit déjà « les données se renouvellent à ce rythme » : la page relit au même rythme que le serveur |
| Ce que reçoit la page | toute la réponse ; **ce que la page déclare** | une réponse peut contenir un numéro de compte, un quota, l'adresse IP du serveur : rien de cela ne doit arriver chez le visiteur |
| Une panne | montrer la dernière valeur avec son âge (l'avis Claude) ; une valeur de secours (`fallback:`, l'avis Gemini) ; **`failed`, comme aujourd'hui** | `On(Meteo.failed)` reste la réponse (`ADR-064`), et l'auteur écrit ce qu'il veut ; montrer l'âge demanderait un mot de plus (une dette) |
| Le faux « autre site » des essais dans Chrome | permettre `http://127.0.0.1` dans les pages, le temps d'un essai ; **renvoyer un nom en `.test` vers ce PC, par une variable d'environnement** | la première affaiblirait la lecture de l'adresse partout (`holo check`, la page) ; la seconde ne touche qu'à holo serve, et seulement pour un nom que l'Internet n'a jamais (`.test`) |

L'interrupteur des essais, `HOLO_TEST_ONLY_INSECURE_SITE=meteo.test:43210` :

- il est éteint par défaut ;
- il est lu seulement dans l'environnement de holo serve, au démarrage : aucune page ni aucune demande ne l'allume ;
- il est annoncé au démarrage (« ESSAIS SEULEMENT : … Jamais pour un vrai site. ») ;
- il n'accepte qu'un nom en `.test`, et toutes les autres règles restent ;
- un essai montre qu'il est éteint par défaut (`the_test_switch_is_off_by_default_and_takes_only_a_test_name`, et dans Chrome : sans la variable, le faux site ne reçoit rien).

## Ce qui est refusé, et pourquoi

- **Dans la page**, avec sa raison, par `holo check` :
  - `http://`, `ftp://`, `//…` ;
  - une adresse IP, `localhost` ;
  - un port, `nom:mot-de-passe@` ;
  - `#`, `{…}` (aucune valeur ne s'écrit dans l'adresse) ;
  - une espace, un accent, un `%` mal écrit ;
  - une adresse de plus de 2 048 caractères ;
  - `every:` de moins de 60 s.
- **Dans `holo-data/sites.txt`**, avec le numéro de la ligne, et jamais ce qui suit le nom d'un site (une clé mal placée y serait) :
  - `*`, une adresse au lieu d'un nom, une adresse IP, `localhost` ;
  - un site écrit deux fois, plus de 32 sites ;
  - un en-tête du protocole ou du visiteur pour une clé ;
  - une clé vide ou mal écrite (le site est alors lu comme « clé manquante » : `failed`, sans demande).
- **Au moment de lire**, avec la raison au journal :
  - un site non permis, une clé manquante ;
  - la clé écrite dans l'adresse de la page, ou son paramètre ;
  - un nom qui mène à ce PC ou au réseau privé ;
  - une redirection, un code autre que 200 (401 et 403 rappellent où s'écrit la clé) ;
  - plus de 64 Ko, autre chose qu'un objet JSON que la page sait lire ;
  - une réponse qui recopie la clé ;
  - plus de 64 adresses gardées, ou de 8 pour un site.

## Les défauts du web évités

- **Le navigateur qui contacte un autre site** : son adresse IP, ses cookies et sa page d'origine partent chez un tiers ; ici, jamais.
- **La clé d'API dans le JavaScript de la page** : ici, elle ne quitte pas le serveur.
- **Le proxy écrit à la main qui va chercher l'adresse que le visiteur donne** (la porte du SSRF) : ici, l'adresse vient de la page de l'auteur, le site doit être permis, et le réseau privé est refusé après résolution.
- **Une redirection suivie sans y penser, une réponse lue en entier, aucun délai** : ici, rien de cela.
- **Une demande par visiteur**, qui épuise le quota ou fait tomber le petit site d'à côté : ici, une par minute au plus et par adresse.
- **Une erreur qui affiche l'adresse avec sa clé** : ici, les erreurs de la bibliothèque ne sont jamais recopiées.

## Limites, honnêtement

- **Seules les valeurs du premier niveau de la réponse**, aux noms que la page déclare, sont reprises, comme pour un fichier (`ADR-030`).
  - Le résumé de Wikipédia s'y prête : `title`, `description`, `extract`.
  - La plupart des services de météo, non : leurs valeurs sont rangées plus bas (`current.temperature_2m`), avec des noms à `_` que HoloCode refuse (`ADR-037`).
- **Aucun proxy.** Un réseau d'entreprise qui oblige à passer par un proxy empêche holo serve de lire un autre site. Le prendre dans l'environnement laisserait le proxy résoudre les noms, et la règle 2 ne tiendrait plus.
- **Un site lent occupe l'un des quatre fils de holo serve** jusqu'à 8 s, au plus une fois par minute et par adresse. La demande ne se fait jamais sous le verrou des gestes ni sous celui de la base.
- **Ce qui est gardé l'est en mémoire** : redémarrer holo serve l'oublie.
- **Le port est toujours celui de HTTPS**, et une clé ne s'écrit pas dans le chemin de l'adresse.
- **Le serveur de test de Node** (`outils/server.mjs`) ne lit pas d'autre site : il répond 501, et la page reçoit `failed`.
- L'autre site voit l'adresse IP du serveur de l'auteur : c'est le prix de la protection du visiteur.

## Dettes

- Choisir et nommer une valeur rangée plus bas dans la réponse (`current.temperature_2m` → `temperature`).
- Montrer la dernière valeur, avec son âge, pendant une panne.
- Une clé donnée par une variable d'environnement, pour un serveur hébergé.
- Un proxy choisi par l'auteur, qui garderait la règle 2.
- Garder les réponses dans la base, pour qu'un redémarrage ne les oublie pas.

## Critères de validation

- Tests du moteur (`cargo test --release --locked`) :
  - `state::remote_address_tests::another_site_is_https_and_a_name_never_an_ip` ;
  - `remote::remote_tests` : 14 essais, dont ceux cités plus haut (une règle au moins chacun) et le démarrage silencieux quand aucune page ne lit d'autre site ;
  - `tests::the_data_of_another_site_is_reduced_to_what_the_page_declares` ;
  - `server::tests::another_site_is_read_by_this_server_with_and_without_javascript`.
- Les essais savent échouer. Chaque mutation a été essayée puis retirée, et un essai au moins a raté :
  - le proxy de l'environnement ;
  - un résolveur qui ne vérifie plus ;
  - dix redirections ;
  - la lecture non coupée ;
  - plus rien de gardé ;
  - la clé dans la page acceptée ;
  - la clé recopiée acceptée ;
  - un sous-domaine deviné.
- Dans Chrome : « les données d'un autre site, lues par holo serve et jamais par le navigateur, avec et sans JavaScript ; l'interrupteur des essais éteint par défaut (leçon 139, serve) ».
- La leçon `139-les-donnees-d-un-autre-site.holo`.
