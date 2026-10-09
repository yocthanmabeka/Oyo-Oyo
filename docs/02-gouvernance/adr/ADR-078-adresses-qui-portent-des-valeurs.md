# ADR-078 — Lot 5 du web : des adresses qui portent des valeurs (`profil/{id}.holo`)

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-08, après avoir essayé les leçons : « tout doit être en décidé car je les ai validés »)
- Date : 2026-10-07 (la décision de Yocthan), 2026-10-08 (la construction)
- Responsable : Yocthan Mabeka
- Discussions sources : l'ordre de Yocthan du 2026-10-07, lot 5 (« holo serve, adresses dynamiques, rendu serveur ») ; son choix, par une question à choix : « Le nom du fichier » ; la passation de la session du nuage (`proposals/Claude/passation-lot5-2026-10-07.md`), et le serveur des `ADR-074` à `ADR-076`.
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine, le serveur

## Contexte

- Une adresse comme `/profil/123` ne menait nulle part : il fallait un fichier par profil.
- Le serveur, `holo serve`, est celui de la session du nuage (`ADR-074` : les pages fabriquées par le serveur, les boutons sans JavaScript ; `ADR-075` : les formulaires ; `ADR-076` : les sauvegardes). La session du PC avait écrit en même temps un autre serveur : il est retiré, pour qu'il n'y en ait qu'un. Les adresses s'ajoutent à celui des `ADR-074` à `ADR-076`.

## Décision

1. **Le nom du fichier dit l'adresse** (décision de Yocthan) : un fichier `profil/{id}.holo` sert `/profil/123`, `/profil/ada` ; les dossiers aussi, `{author}/notes/{note}.holo`. Un vrai fichier passe avant le modèle : `profil/moi.holo` avant `profil/{id}.holo`.
2. **La page lit `{id}` comme ses autres valeurs**, un texte : `H1("Profil n° {id}")`, `If(id, is: "ada", …)`. Elle ne la change jamais : `id.set(…)`, `Input(value: id)` et `State(id: …)` sont refusés, avec la raison. Le nom s'écrit comme une valeur (`id`, `userName`) ; la valeur est le morceau d'adresse décodé, non vide, de 200 caractères au plus.
3. **Un seul chemin pour tous** : les valeurs sont jointes au texte de la page comme un petit fichier, `@adresse` (`id=123`), comme les fichiers importés. Le serveur, le moteur de la page, le serveur d'essai, l'éditeur et l'extension VS Code passent tous par là : ils ne peuvent pas diverger.
   - `holo serve` fabrique la page de l'adresse, avec et sans JavaScript ; l'état d'un visiteur est gardé pour chaque adresse ; un formulaire envoyé de l'adresse est vérifié avec ses valeurs, et rangé avec son adresse et son modèle (le quota se compte par modèle).
   - La page fabriquée nomme toujours son fichier (`<meta name="holo-file">`) ; le moteur de la page y lit son texte, même quand l'adresse ne le dit pas (`/contact` mène aussi à `contact.holo`).
   - `holo check profil/{id}.holo` vérifie le modèle, chaque nom valant un texte vide ; `HOLO_ADDRESS=id=123 holo html …` le fabrique pour une adresse.
4. **Les liens remontent d'un dossier**, comme sur le web : `A(to: "../accueil.holo")`, et portent des lettres accentuées, `A(to: "profils/Adé")`, que le navigateur encode. C'est **un retour sur une ancienne règle**, qui refusait `../` dans un lien (« un lien ne peut pas sortir de son dossier ») : retirer cette capacité du web allait contre l'ordre de Yocthan, et un modèle rangé dans un dossier ne pouvait plus mener à sa leçon. Les serveurs ne sortent jamais du site ; les fichiers que la page lit (images, polices, passages) gardent la règle stricte. À valider par Yocthan.

## Ce qui est refusé, et pourquoi

- Une adresse qui ne correspond à aucun fichier ni modèle (`/profil/1/2`) : 404.
- Un morceau vide, mal encodé, ou de plus de 200 caractères ; un nom avec `_` ou `-` (les noms s'écrivent comme les autres valeurs, `ADR-037`).
- Changer une valeur d'adresse, la déclarer aussi dans `State`, la poser sur un monde (`Point`) au lieu d'une page.
- Un « & » ou un « = » dans une valeur n'ouvre pas une autre valeur : ils sont encodés.

## Les défauts du web évités

- **Une écriture à part pour les adresses** (`[id]` chez Next.js, `:id` chez Express) : ici, les accolades de HoloCode, les mêmes que dans un texte.
- **Deux programmes qui lisent l'adresse chacun à sa façon** (le serveur, le code de la page) : ici, le même moteur, la même règle.

## Dettes

- La proposition de la session du nuage reste ouverte : remplir une page de profil par `Data(from: "profils/{id}.json")`, lu par le serveur, et répondre « introuvable » sans ce fichier. Elle demande de restreindre `{id}` à des lettres, des chiffres, `-` et `_` (un nom de fichier) : à décider avec Yocthan.
- ~~Les valeurs gardées d'une page (`keep`) sont rangées sous le nom du modèle : `/profil/ada` et `/profil/yocthan` les partagent.~~ Réglé par l'`ADR-090` : une par adresse.
- ~~Le titre d'une page (`Page(title:)`) ne lit pas encore `{id}`.~~ Réglé par l'`ADR-090`.
- Dans un modèle rangé dans un dossier `{x}`, un lien relatif part du dossier écrit (`…/{x}/`), pas de l'adresse.
- L'extension VS Code est corrigée (`holo check` d'un modèle ; le bouton ▶ et Ctrl+Alt+H, cassés depuis l'`ADR-060`), mais à réemballer : `python outils/vscode-holocode/package.py`.

## Critères de validation

- Tests du moteur : `an_address_carries_values`, `the_page_reads_its_address_and_never_changes_it`, et, dans le serveur, `an_address_carries_a_value_with_and_without_javascript` (la page fabriquée, une valeur accentuée, le texte du modèle, `/contact`, une adresse sans modèle, un toucher sans JavaScript gardé pour cette adresse seule, un formulaire rangé avec son modèle) ; le test des liens.
- Dans Chrome : « une adresse porte une valeur (leçon 100) » (le serveur d'essai, le moteur, l'éditeur, un formulaire envoyé de l'adresse, un message forgé refusé) ; « holo serve sert une adresse qui porte une valeur » (la page fabriquée, le moteur qui garde la valeur, le message rangé avec son modèle).
