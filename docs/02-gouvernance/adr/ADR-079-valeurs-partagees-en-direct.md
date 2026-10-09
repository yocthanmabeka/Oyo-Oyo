# ADR-079 — Lot 6 du web : des valeurs partagées, en direct (`shared: Shared(seats: 20)`)

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-08, après avoir essayé les leçons : « tout doit être en décidé car je les ai validés »)
- Date : 2026-10-08
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`, lot 6) ; la proposition du serveur (`proposals/Claude/serveur-et-comptes-2026-10-07.md`) ; la décision de Yocthan du 2026-10-08, transmise par la session principale ; le serveur des `ADR-074` à `ADR-076` et les adresses de l'`ADR-078`.
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine, le serveur

## Contexte

- Jusqu'ici, chaque visiteur avait ses valeurs à lui seul (`State`, `ADR-023`) : rien ne permettait d'écrire des places restantes, ou un compteur de « J'aime » que tout le monde voit.
- **Décidé par Yocthan (2026-10-08)** : « partager » veut d'abord dire une valeur que le serveur garde pour tout le monde ; chaque visiteur la voit changer en direct, sans recharger ; c'est toujours le serveur qui arbitre, avec le même moteur que la page ; rien d'extérieur, tout chez l'auteur (`holo serve` sur son PC) ; ce qui marche sur un téléphone marche sur un ordinateur, et l'inverse ; et sans JavaScript, la page reste juste.
- La forme, `shared: Shared(…)` à côté de `state: State(…)`, est celle proposée à Yocthan avec cette décision.

## Décision

1. **`shared: Shared(seats: 20, likes: 0)`**, sur la page, à côté de `state:`. Les mêmes sortes de valeurs que `State` : des nombres entiers, des nombres à virgule, des textes. La valeur de départ est celle du fichier.
2. **Elles se lisent et se changent comme les autres** : les textes et les conditions les lisent (`{seats}`, `If(seats, over: 0, …)`), les règles les changent (`On(Book.tap, effect: seats.sub(1))`). À la lecture, le moteur les range avec celles de `State`, comme les valeurs de l'adresse (`ADR-078`) : tout le moteur les connaît sans rien apprendre de nouveau, et il garde leurs noms (`program.shared`).
3. **Une valeur partagée vaut pour une adresse** : avec `concert/{id}.holo`, `/concert/12` et `/concert/13` ont chacune leurs places. L'adresse d'une page ordinaire est celle de son fichier (`/salle.holo`, même ouverte par `/salle`).
4. **Seul un toucher la change, et c'est le serveur qui l'arbitre.** Quand un toucher change une valeur partagée (une règle `On(….tap, …)` qui la demande), la page ne décide rien :
   - avec JavaScript, le moteur de la page envoie le signal et son état au serveur, en JSON, à l'adresse de la page (`{"signal":"Book.tap","state":"…"}`), montre qu'il attend, et prend l'état que le serveur rend ; ce que le visiteur a écrit pendant l'attente reste ;
   - sans JavaScript, le toucher part par le formulaire des gestes (`ADR-074`), et la page revient à jour (`303`) ;
   - le serveur prend sa base, relit l'état du visiteur avec méfiance et **remplace ses valeurs partagées par celles qu'il garde**, fait tourner le même arbitre, range les nouvelles valeurs, puis les **envoie en direct à toutes les pages ouvertes à cette adresse**. Deux visiteurs en même temps : chacun son tour, rien n'est perdu.
5. **Un bouton caché ne se touche pas.** Le serveur refuse le toucher d'un bouton que la page ne montre pas pour ces valeurs (rangé dans un `If` faux, ou dans le `else` d'un `If` vrai) : c'est ainsi qu'une condition garde une valeur partagée. Avec `If(seats, over: 0, children: [ Button(name: Book, …) ])`, deux visiteurs qui touchent la dernière place en même temps : le premier l'a, le second reçoit « refusé » (`409`) et voit « Complet ». La règle est celle du web ordinaire, qu'un bouton caché par le CSS ne respecte pas : on ne touche que ce qui se voit.
6. **Le direct, par un flux du serveur** (Server-Sent Events, l'`EventSource` du navigateur) : la page ouvre `GET <son adresse>` avec `Accept: text/event-stream` ; le serveur envoie d'abord les valeurs du moment, puis chaque changement, numéroté (`id:`) : une page ne reprend jamais un changement plus ancien que celui qu'elle a. Elle se reconnecte seule (`retry: 3000`) et repart des valeurs du moment. Une page fermée cesse d'écouter : le navigateur ferme la connexion, et le serveur l'oublie à son prochain envoi.
7. **`holo serve`** garde les valeurs dans sa base, `holo-data/site.sqlite`, table `shared` : l'adresse, le nom, la valeur, le numéro du changement, la date. La page arrive fabriquée avec les valeurs du moment (`data-shared`) ; elle est vivante (`data-live`) : le moteur arrive tout de suite pour écouter. Chaque page en direct a son propre fil, qui écrit pour elle : les quatre fils du serveur restent libres, et une page lente ne retient personne (au-delà de 64 changements en retard, elle est coupée, puis se reconnecte). Le serveur écrit lui-même la réponse sur la connexion (`Request::into_writer` de `tiny_http` 0.12 ; `Request::upgrade`, qui rend aussi le flux brut, ajoute `Connection: upgrade`, fait pour un WebSocket).
8. **Le serveur d'essai** (`outils/server.mjs`, le port 8080 des leçons) fait pareil avec **le même moteur** : `holo share page.holo`, qui reçoit `{"shared":…,"state":…,"signal":…}` et rend `{"accepted":…,"changed":…,"state":…,"shared":…}`. Il garde les valeurs en mémoire, les oublie à son arrêt, et les envoie en direct comme `holo serve` ; `holo html` les reçoit par `HOLO_SHARED`.
9. **Les limites** : 16 valeurs partagées par page (32 avec celles de `State`) ; un texte partagé de 200 caractères au plus (coupé à cette longueur, aussi pour celui qui l'écrit) ; un nombre de 0 à 1 milliard ; un geste et son état de 64 Ko au plus ; 128 pages en direct pour un site ; 20 touchers en attente du serveur dans une page, et 10 secondes pour sa réponse : sans réponse, rien ne change, et la page le dit.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le direct | relire la page à intervalles (« polling ») ; un **WebSocket** ; **un flux du serveur (Server-Sent Events, `EventSource`)** | **le flux** : une seule direction suffit (la page envoie ses gestes par un `POST` ordinaire, qui marche aussi sans JavaScript) ; c'est du HTTP ordinaire, sur le même port, avec les mêmes cookies et la même règle d'origine ; le navigateur se reconnecte seul ; une quarantaine de lignes dans le serveur, sans bibliothèque. Un WebSocket demande une poignée de main (SHA-1 et base64, à écrire ou à prendre dans une bibliothèque), des trames masquées, ses battements, et la reconnexion à écrire dans la page : plus de code pour une direction qui ne sert pas. Le polling arrive en retard ou charge le serveur pour rien. À reconsidérer avec le jeu à plusieurs (des messages dans les deux sens, très fréquents) |
| Qui arbitre un geste partagé | la page tout de suite, puis le serveur corrige (« optimiste ») ; **le serveur, et la page attend** | la page ne montre jamais « réservé » pour le reprendre ensuite ; sur le Wi-Fi de l'auteur, l'attente est de quelques centièmes de seconde, et elle se voit (le bouton attend) |
| Ce qu'une condition garde | rien (un geste forgé passe toujours) ; une écriture nouvelle (`On(…, when: …)`) ; **le bouton caché ne se touche pas** | aucun mot de plus : l'auteur cache le bouton, comme il le fait déjà ; le serveur, qui fabrique la page, sait ce qu'il cache |
| Où le serveur les garde | en mémoire ; **dans la base SQLite** | elles survivent à un redémarrage et partent dans les sauvegardes (`ADR-076`) ; le serveur d'essai, fait pour les leçons, les garde en mémoire |
| L'écriture | `State(seats: shared(20))` ; `State(shared: [seats])` ; **`shared: Shared(seats: 20)`** | un bloc de plus, à côté de `State`, comme `Prices` : on voit d'un coup d'œil ce qui est à tous ; `State` garde son sens (à chaque visiteur) |

## Ce qui est refusé, et pourquoi

- Le même nom dans `State` et dans `Shared` (« une valeur est à chaque visiteur ou à tous, pas aux deux ») ; un nom qui vient de l'adresse (`{id}`) ; `Shared` ailleurs que sur la page, ou deux fois.
- Ce qui changerait une valeur partagée sans passer par le serveur : une horloge (`Every`, `After` : chaque page ouverte la ferait changer), une règle qui guette (`When` : écrire le changement dans la règle du toucher), une touche (`Key.…`), un survol, la réponse d'un formulaire ou de données (`Contact.sent`, `Shop.done`), un module (`output:`), un bloc qu'on fait glisser. Des données reçues (`Data`) ne la changent pas : le moteur garde celle du serveur.
- Un champ lié à une valeur partagée (`Input(value: seats)`) : pas encore, une dette.
- `keep: [seats]` : le serveur la garde pour tous, `keep` garde ce qui est à un seul visiteur.
- Une liste partagée (`Shared(tasks: [])`) : pas encore, une dette.
- Au serveur : un signal qui n'est pas un toucher, ou qui ne change aucune valeur partagée (`400`) ; le toucher d'un bouton caché (`409`) ; une demande de plus de 64 Ko (`413`) ; un geste ou une écoute venus d'un autre site (`403`) ; au-delà de 128 pages en direct (`503`).

## Sécurité, honnêtement

- **L'état personnel qu'envoie le visiteur n'est pas sûr** : avec JavaScript, il l'envoie avec son geste, et il peut le forger. Le serveur ne croit que ses valeurs partagées et les règles de la page. Mais tant qu'il n'y a pas de comptes (le lot 7, construit en même temps par une autre session), **une condition sur une valeur personnelle qui garde une valeur partagée se contourne** : dans la leçon 101, un visiteur qui écrit `booked=0` dans l'état qu'il envoie peut réserver une seconde place, et `booked=1` lui fait rendre une place qu'il n'a jamais prise. Sans JavaScript, l'état personnel est celui que le serveur garde sous le numéro du visiteur ; mais un visiteur qui efface son cookie revient comme un autre.
- **Ce que le serveur garantit** : l'ordre (un geste partagé à la fois pour tout le site, sous le verrou de la base : deux visiteurs en même temps, chacun son tour, rien de perdu ; essayé avec huit gestes au même instant) ; les bornes (de 0 à 1 milliard, des textes de 200 caractères, seize valeurs) ; seuls les touchers des règles de la page ; un bouton caché pour ses valeurs ne se touche pas ; la taille des demandes ; l'origine des gestes et des écoutes ; le nombre de pages en direct.
- **Ce qui n'est pas encore limité** : le nombre de touchers d'un visiteur. Un petit programme peut ajouter mille « J'aime ». Avec les comptes, « un J'aime par compte » deviendra possible.

## Les défauts du web évités

- **Deux programmes pour une valeur** : sur le web, une valeur partagée demande une base et une API côté serveur, puis du code dans la page pour l'envoyer et l'écouter, dans deux langages, avec deux vérifications qui divergent. Ici, une ligne, `Shared(…)`, et le même moteur des deux côtés.
- **La page qui ment** : l'interface « optimiste » montre « réservé » puis le reprend ; ici, la page attend le serveur, et les autres pages changent d'elles-mêmes.
- **Le bouton caché qui se clique encore** : sur le web, un bouton caché par le CSS reste joignable par une requête forgée ; ici, le serveur refuse ce qu'il ne montre pas.
- **Le site en direct qui ne marche plus sans JavaScript** (les applications faites avec Firebase ou Meteor) : ici, sans JavaScript, la page reste juste ; seul le direct manque, et la page est à jour à chaque toucher.
- **Un service extérieur obligatoire** pour le temps réel : ici, rien d'autre que le PC de l'auteur.

## Dettes

- Un champ lié à une valeur partagée, une liste partagée.
- Une limite au nombre de touchers d'un visiteur, et une valeur « une fois par visiteur » sûre : avec les comptes (lot 7).
- Le navigateur n'ouvre que six connexions à la fois vers un même site en HTTP/1.1, et chaque page en direct en tient une : au-delà de cinq onglets du même site dans un même navigateur, le suivant attend. Il faudrait HTTP/2 (que `tiny_http` ne parle pas), ou un seul flux partagé entre les onglets.
- Une page fermée n'est oubliée par `holo serve` qu'au second envoi qui ne passe plus (un changement, ou le battement de dix secondes) : jusqu'à vingt secondes.
- Le serveur d'essai fabrique une page qui partage des valeurs et reçoit des données (`Data`) sans ses données ; il oublie les valeurs à son arrêt.
- Un geste partagé sans JavaScript ne joue pas ses sons (comme les autres gestes, `ADR-074`).
- L'extension VS Code propose les valeurs de `Shared` (par `fixes.js`) ; elle est à réemballer : `python outils/vscode-holocode/package.py`.

## Critères de validation

- Tests du moteur : `shared_values_join_the_page_values`, `what_would_change_a_shared_value_without_the_server_is_refused` (chaque refus, avec sa raison), `the_server_values_replace_the_visitor_ones` (un état forgé, un texte trop long, un nombre trop grand), `the_server_arbitrates_and_a_hidden_button_is_not_touched` (deux places, trois visiteurs) ; et dans le serveur, `shared_values_are_kept_for_everyone_with_and_without_javascript` (sans JavaScript, puis en JSON avec un état forgé, un bouton caché refusé, deux adresses d'un modèle, la base relue par un nouveau serveur, les refus) et `a_page_listens_live_and_a_closed_one_is_forgotten` (les valeurs du moment puis chaque changement, huit gestes au même instant reçus dans l'ordre, une page fermée oubliée, la 129e page refusée).
- Dans Chrome (`node outils/browser-tests.mjs partag`) : « une valeur partagée change en direct dans deux pages, avec le serveur d'essai » (deux onglets à la même adresse, réserver vu en direct sans recharger, cinq « J'aime » des deux côtés, aucun perdu, un survol et un bouton caché refusés, l'onglet fermé n'écoute plus, sans réponse rien ne change et la page le dit) ; « holo serve : une valeur partagée en direct, avec et sans JavaScript » (un troisième onglet sans JavaScript, la dernière place complète partout, une place forgée refusée, la base).
- Par Yocthan : la leçon 101 sur son téléphone et sur son ordinateur, en même temps.
