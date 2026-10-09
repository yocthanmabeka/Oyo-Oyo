# ADR-091 — L'historique dans une page : `address: [onglet, page]`

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-09, après avoir tout essayé : « j'ai tout essayé et c'est bon »)
- Date : 2026-10-08
- Responsable : Yocthan Mabeka
- Discussions sources : les tâches confiées par Yocthan à la session du nuage le 2026-10-08 (`AGENTS.md`, « L'historique dans une page, laissé par le lot 8 en attendant le serveur ») ; l'`ADR-073` (« garder l'état d'une page dans son adresse viendra avec le premier vrai serveur ») ; le serveur des `ADR-074` à `ADR-076` et les adresses de l'`ADR-078`.
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine, le serveur

## Contexte

- « Précédent » marchait entre les pages et les mondes, pas à l'intérieur d'une page : changer d'onglet, passer à la page 2 d'une liste, puis revenir en arrière quittait la page.
- Une adresse ne pouvait pas dire « l'onglet des aquarelles, page 2 » : un lien partagé ramenait toujours au début.
- Sur le web, il faut écrire soi-même `history.pushState`, lire `location.search`, écouter `popstate`, et le serveur doit savoir lire les mêmes paramètres : trois endroits qui divergent.

## Décision

1. **La page nomme les valeurs que son adresse porte** : `Page(address: [onglet, page], …)`. Elles s'écrivent après le `?`, dans l'ordre : `galerie.holo?onglet=aquarelles&page=2`. Une valeur à son départ n'y est pas écrite : la page du début garde son adresse nue.
2. **Un toucher ou une touche qui les change fait un pas dans l'historique** : « Précédent » (le bouton du navigateur, le geste retour du téléphone) revient aux valeurs d'avant, « Suivant » y retourne. Ce qu'on écrit dans un champ, le temps qui passe et les données reçues mettent l'adresse à jour sans faire de pas : pas un pas par lettre.
3. **L'adresse se partage** : qui l'ouvre arrive sur les mêmes valeurs. Le serveur d'essai, `holo serve` et `holo html` (`HOLO_QUERY=onglet=dessins`) fabriquent la page avec elles ; le moteur de la page les reprend.
4. **Sans JavaScript aussi** : un toucher envoyé à `holo serve` mène à l'adresse des nouvelles valeurs (`303`), et le navigateur redemande l'adresse d'avant quand on revient en arrière. L'adresse dit ses valeurs, même si le visiteur a touché d'autres boutons depuis.
5. **Ce qui arrive par l'adresse vient de n'importe qui** : comme les valeurs gardées (`keep`), seules celles que la page nomme sont reprises, dans leurs bornes (le `min` et le `max` d'un champ, les chiffres après la virgule) ; un texte de 200 caractères au plus ; un texte que la page n'écrit qu'avec des mots fixes (ses `set("…")`, les options d'un `Choice`) n'en prend pas d'autre. Une valeur mal écrite ou inconnue part de son départ, sans erreur : `?onglet=pirate` montre l'onglet du début.
6. **Un seul chemin** : le moteur dit quelles valeurs l'adresse porte (`address_query`) et les reprend (`from_query`) ; le navigateur, les deux serveurs et `holo html` passent tous par lui.

## Ce qui est refusé, et pourquoi

- Une liste (`address: [panier]`) : une adresse peut devenir trop longue ; garder une liste, c'est `keep`.
- Une valeur gardée (`keep`) : elle viendrait de deux endroits, l'adresse et le navigateur.
- L'heure du visiteur (`minute`, `today`), une valeur qui vient déjà du nom du fichier (`{id}`, `ADR-078`), un nom inconnu, un nom écrit deux fois.
- Les noms que le moteur lit déjà dans une adresse : `values`, `bare`, `webgl`, `world`, `view`, `enter`, `zoom`, `x`, `y`, `yaw`, `pitch`, `measures` (`?values` ouvre le panneau des valeurs, `?view=points` la vue points). Le message demande un autre nom.
- `address:` dans un monde (`World`) : un monde partage l'adresse et les valeurs de sa page.

## Les défauts du web évités

- **L'état de l'adresse écrit à la main** (`history.pushState`, `URLSearchParams`, `popstate`, et leur équivalent côté serveur) : ici, une liste de noms.
- **Un pas d'historique par lettre tapée**, ou des pas que « Précédent » ne défait pas : ici, seuls les touchers et les touches font un pas.
- **Des paramètres d'adresse acceptés tels quels** (`?role=admin`) : ici, seulement les valeurs nommées, dans leurs bornes, et les mots que la page sait écrire.

## Dettes

- Un lien vers une page avec ses valeurs (`A(to: "galerie.holo?onglet=dessins")`) n'est pas encore permis : on y arrive par l'adresse, pas par un lien de la page.
- Une page fabriquée avec ses données (`Data(from:)`, `ADR-064`) ne lit pas encore l'adresse sur le serveur : le moteur de la page la reprend en arrivant.
- Un monde ouvert en grand (`#Atelier`) ne change pas l'adresse de sa page.
- Le nom `address:` est une proposition : `history:` dirait ce que l'on gagne, `address:` dit où vont les valeurs. À choisir par Yocthan.

## Critères de validation

- Tests du moteur : `the_address_carries_the_values_the_page_names` (l'adresse nue au départ, puis `page=3` ; un texte accentué, un nombre à virgule ; le retour au départ ; des valeurs forgées refusées ; un texte à mots fixes ; la page fabriquée avec ses valeurs) ; `the_address_refuses_what_it_cannot_carry` ; dans le serveur, `the_address_carries_the_history_of_a_page_without_javascript` (une adresse partagée, un toucher sans JavaScript mené à l'adresse de ses nouvelles valeurs, « Précédent » qui redemande l'adresse d'avant).
- Dans Chrome : « l'historique dans une page : address: [onglet, page] (leçon 114) » (deux touchers, deux pas ; « Précédent » deux fois, « Suivant » une fois ; une adresse partagée fabriquée par le serveur ; `?values` gardé dans l'adresse ; des valeurs forgées).
- Leçon `114-l-historique-dans-une-page.holo`.
