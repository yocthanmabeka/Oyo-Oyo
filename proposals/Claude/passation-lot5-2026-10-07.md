# Passation : ce que la session du nuage a fait du lot 5, pour la session du PC

- Auteur : Claude, session du nuage, le 2026-10-07.
- Pour : la session du PC, qui garde les lots 5, 6 et 7.
- Décision de Yocthan, le 2026-10-07 : « que vous ne puissiez plus vous entremêler entre les lots ; que chacun puisse avoir un lot différent ». La session du PC avait inscrit les lots 5 à 7 la première sur `main` (PR 166) : « vu que c'est lui qui avait commencé, qu'il puisse analyser tes informations, ensuite qu'il les scanne, et que toi tu fasses le lot auquel il n'a pas encore touché ». La session du nuage passe au lot 9 et ne touche plus aux lots 5 à 7.
- Statut : **PASSATION**. Rien ici n'oblige la session du PC : elle garde, change ou jette.

## Ce qui est déjà sur `main`

**PR 167, `ADR-074` : `holo serve`, un serveur en Rust avec SQLite ; les boutons marchent sans JavaScript.** Fusionnée par le programme de fusion automatique quand ses six vérifications sont passées au vert, quelques minutes avant la décision de Yocthan.

- `moteur/src/server.rs` : `holo serve [dossier] [port]`, avec `tiny_http` (quatre fils) et `rusqlite` (`bundled`, SQLite comprise dans `holo.exe`). La base est `holo-data/site.sqlite`, avec la table `visits(visitor, page, state, updated)`. Chaque visiteur a un numéro de 128 bits, dans un cookie `HttpOnly; SameSite=Lax` ; il est oublié après trente jours d'absence. `holo-data/` n'est jamais servi, et il est dans `.gitignore`.
- `moteur/src/gestures.rs` : la page d'un visiteur garde exactement son HTML. Un formulaire caché, `holo-gestures`, est posé en tête de page. Les boutons nommés (`Button`, `Shape`) et les champs `data-bind` s'y rattachent par `form=`. Le geste part en `signal=Add.tap`, ou `signal=Done.tap@2` dans une ligne de liste. Seuls les touchers passent (`is_tap`). Le bouton caché en tête reçoit la touche Entrée d'un champ.
- `moteur/src/lib.rs` : `visitor_page` fabrique la page avec les valeurs du visiteur, gardées dans `data-visit`. `visitor_gesture` applique les champs, puis le toucher, par le même arbitre que le navigateur.
- `moteur/web/page.html` : le formulaire des gestes ne part jamais avec JavaScript. `moteur/web/page-engine.js` : le moteur repart de `data-visit`.
- `moteur/outils/browser-tests.mjs` : l'essai « sans JavaScript, holo serve fait marcher les boutons (serve) », sur les leçons 68 et 14.
- Refusés par le serveur : un geste qui n'est pas un toucher, un geste venu d'un autre site (`Origin`), plus de 64 Ko, la base, un fichier caché, `..`.
- Ce qui ne marche pas sans JavaScript, et demande le moteur : les horloges, le clavier, le survol, le glissement, les points et les mondes, les modules.

## Ce qui reste en PR ouverte, non fusionnée : à toi de décider

**PR 168, `ADR-075` : les formulaires `Form` reçus par `holo serve`, avec ou sans JavaScript ; `holo messages`.** Branche `serveur/lot5-formulaires`.

- Un formulaire envoyé par le moteur, en JSON ou en plusieurs morceaux avec des fichiers, est reçu comme le fait `outils/server.mjs` : le moteur revérifie les champs, et les fichiers sont reconnus à leurs premiers octets. Le message va dans une table `messages`, les fichiers dans `holo-data/files/<page>/`.
- Sans JavaScript, les champs et le bouton d'un `Form` rejoignent le formulaire des gestes. Quand le toucher demande l'envoi (`On(Send.tap, effect: Contact.send)`), le serveur vérifie les champs :
  - s'il manque quelque chose, rien n'est rangé, et les messages d'erreur reviennent sous les champs (`gestures::with_errors`, colonne `tried`) ;
  - sinon, le message est rangé, puis `Contact.sent` passe par l'arbitre.
- Avec JavaScript, `page.html` détache tout du formulaire des gestes dès l'arrivée de la page : tout se passe comme avec `outils/server.mjs`.
- `holo messages [dossier]` affiche les messages reçus, une ligne JSON chacun.
- Essai : « un formulaire reçu par holo serve, avec ou sans JavaScript (serve) », sur la leçon 88.

**PR 169, `ADR-076` : les sauvegardes.** Branche `serveur/lot5-sauvegardes` ; elle contient la PR 168.

- `holo serve` copie la base (`VACUUM INTO`, cohérent même pendant les écritures) dans `holo-data/backups/site-AAAA-MM-JJ-HHMM-SS.sqlite` : au démarrage si la dernière copie a plus d'un jour, puis chaque jour.
- `holo backup [dossier]` fait une copie tout de suite. Les quatorze plus récentes sont gardées.

Ces deux PR changent aussi `AGENTS.md` et le journal avec l'ancienne répartition des lots : en les fusionnant, garde le tableau de `main`.

## Comment les vérifier

```text
cd moteur
cargo test --release
cargo build --release --bin holo
node outils/browser-tests.mjs serve
.\target\release\holo serve ..\exemples\lecons 8080
```

Ensuite, dans Chrome, coupe JavaScript (outils de développement, Ctrl+Maj+P, « Disable JavaScript »), puis ouvre `http://localhost:8080/14-prix.holo`, `68-liste-qui-change.holo` et `88-un-formulaire-qui-verifie.holo`.

Ce que la session du nuage a vérifié :

- les tests du moteur : 153 passent avec la PR 169 ;
- les deux moteurs WebAssembly se construisent ;
- les essais « (serve) » passent dans Chrome.

Dans la suite complète, en local, deux essais étaient rouges :

- « pincer », une fois, puis vert deux fois seul ;
- « la vue points se lit au lecteur d'écran », rouge aussi sur `main` dans ce conteneur.

Sur GitHub, la PR 167 est passée verte en entier.

## Ce qui reste du lot 5, et une question ouverte pour Yocthan

Il reste les adresses `profil/{id}.holo`, dont la forme a été décidée par Yocthan. La session du nuage lui a proposé ceci ; il n'a pas encore répondu :

1. `{id}` existe tout seul, sans `State` : il vient du nom du fichier.
2. Les données d'un profil viennent de `Data(from: "profils/{id}.json")`, lu par le serveur. La page arrive donc remplie, pour les robots et pour l'aperçu d'un lien partagé.
3. Un `id` ne contient que des lettres, des chiffres, `-` et `_`, 64 au plus. Sans fichier de données, le serveur répond « introuvable » (404).
4. Un vrai fichier passe avant le modèle : `profil/moi.holo` avant `profil/{id}.holo`.

À toi de la reprendre, de la changer, ou d'en proposer une autre à Yocthan.

## Les numéros

- `ADR-074` est sur `main`.
- `ADR-075` et `ADR-076` sont pris par les PR 168 et 169 : si tu les fermes, ne réutilise pas ces numéros.
- La session du nuage garde, pour le lot 9, `ADR-077` puis `ADR-086` à `ADR-092`, et les leçons 97 à 99 puis 110 à 119.
