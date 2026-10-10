# ADR-113 — Se souvenir le temps d'une visite : `visit: [prenom]`

- Statut : ACCEPTÉ (Yocthan, 2026-10-09 : « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #242 (« Dernière dette du web : se souvenir le temps d'une visite (sessionStorage) »), l'une des douze dernières dettes du web, validées d'avance par Yocthan le 2026-10-09 ; le grand tableau du web, où `cookies, sessionStorage` était le seul « non » ; les valeurs gardées (`keep`, `ADR-027`), rangées sous l'adresse de la page (`ADR-090`) ; l'historique dans une page (`address:`, `ADR-091`) ; les valeurs partagées (`shared`, `ADR-079`) ; l'import d'un fichier (`ADR-093`).
- Validation : Yocthan, le 2026-10-09 (« tu le valides déjà, tu le fais déjà »).
- Projets affectés : HoloCode, HoloEngine, les deux serveurs (un en-tête)

## Contexte

- Un formulaire en plusieurs pages (une inscription, une commande, une réservation) doit retrouver à l'étape 2 ce qu'on a écrit à l'étape 1, et le garder quand on revient en arrière.
- `keep` (`ADR-027`) garde des valeurs dans le navigateur d'une visite à l'autre, sous l'adresse de la page (`ADR-090`) : chaque page garde les siennes, rien ne passe à une autre. Et elles restent des jours, même sur un ordinateur partagé.
- `address:` (`ADR-091`) met des valeurs dans l'adresse : elles partent avec le lien, restent dans l'historique et dans les journaux du serveur. Un prénom, un e-mail n'ont rien à y faire.
- Sur le web, il faut écrire `sessionStorage` à la main, en texte, et relire sans rien vérifier ; ou poser un cookie de session et garder le brouillon sur le serveur. Le grand tableau disait « non ».

## Décision

```holo
Page(
  title: "Step 1",
  state: State(firstName: "", people: 1, workshop: ""),
  visit: [firstName, people, workshop],
  children: [
    Input(value: firstName, label: "Your first name", max: 40),
    Input(value: people, label: "How many people?", min: 1, max: 6),
    Choice(value: workshop, label: "The workshop", options: ["Watercolour", "Pottery"]),
    A("Step 2", to: "step-2.holo"),
  ],
)
```

1. **`visit: [firstName, people, workshop]`**, sur la page, à côté de `keep:` : ces valeurs sont retenues le temps de la visite. Ce sont des valeurs déclarées dans `State` : des nombres, des nombres à virgule, des textes, des listes.
2. **Elles suivent le visiteur d'une page à l'autre du site**, dans le même onglet. Une autre page du site qui retient le même nom (`visit: [firstName]`) retrouve la même valeur. Le site, c'est l'adresse du serveur : le même `localhost:8080`, le même nom de domaine.
3. **Le temps d'une visite** : le navigateur les garde tant que l'onglet est ouvert, rechargement et « Précédent » compris. Il ne les donne à aucun autre onglet, et les efface quand l'onglet se ferme. C'est `sessionStorage`.
4. **Chaque valeur est rangée sous son nom**, `holo-visit:firstName`, en JSON écrit par le moteur : `"Ada"`, `3`, `12.50`, `["Pain"]`. Une page n'écrit que les valeurs qui changent chez elle (par le visiteur, ou par une règle) : elle n'écrase jamais, sans qu'on y touche, ce qu'une autre page a écrit.
5. **Ce qui revient est vérifié par la page qui le lit, comme un import (`ADR-093`)**, chaque valeur seule :
   - de sa sorte : un texte dans un texte, un nombre dans un nombre, une liste dans une liste ;
   - dans les bornes de cette page : le `min` et le `max` d'un champ, 0 ou 1 pour une case, les chiffres après la virgule sans arrondir ;
   - un texte comme le prendrait le champ de cette page : sa longueur, une option du `Choice`, une vraie date ; sans champ (un récapitulatif), 2 000 caractères sans caractère invisible ;
   - une liste de 200 éléments au plus, des fiches avec exactement leurs champs ; 64 Ko par valeur.

   Une valeur qui ne va pas est ignorée, sans erreur : la page garde la sienne, et la valeur reste telle quelle pour les pages qui la comprennent. Jamais une valeur fausse (un texte coupé, un nombre arrondi ou ramené à sa borne), jamais d'erreur à l'écran.
6. **Deux pages, deux sortes** : si une page fait de `prenom` un texte et une autre un nombre, chacune ne reprend que ce qui est de sa sorte. Le moteur ne voit qu'une page à la fois : il ne peut pas le refuser, mais il ne se trompe jamais.
7. **L'ordre, à l'arrivée.** La page part :
   - de ses valeurs de départ ;
   - puis de ce qu'elle garde (`keep`), de ce que le serveur sait du visiteur (`ADR-074`, `ADR-081`), de ses données (`ADR-064`), de son adresse (`ADR-091`), des valeurs partagées (`ADR-079`) ;
   - et enfin de la mémoire de visite : ce que le visiteur a donné dans cet onglet l'emporte.

   Revenue par « Précédent », la page reprend ce qui a changé sur les autres pages : qu'elle sorte du cache du navigateur telle qu'on l'avait quittée, ou qu'elle soit rechargée avec, dans ses champs, ce qu'on y avait écrit avant.
8. **Le serveur** (`holo serve`, le serveur d'essai, `holo html`) fabrique la page avec ses valeurs de départ : la mémoire de visite est dans le navigateur seulement. La page la reprend après le chargement, comme `keep`. La page légère (`ADR-033`) fait venir le moteur tout de suite si l'onglet retient déjà une de ses valeurs (elle lit leurs noms dans `data-visit-names`) ; sinon au premier geste, comme d'habitude.
9. **Sans JavaScript, rien n'est retenu** : chaque page part de ses valeurs de départ. Avec `holo serve`, chaque page garde ce qu'on y a touché (`ADR-074`), mais rien ne passe d'une page à l'autre.
10. **La vie privée** :
    - rien ne part au serveur : ni cookie, ni en-tête, ni demande, la mémoire de visite n'ajoute aucun envoi ;
    - rien n'est partagé entre deux onglets ;
    - tout est effacé à la fermeture de l'onglet ;
    - aucun cookie.

    Une valeur retenue reste une valeur de la page : elle ne part au serveur que par les chemins de toujours, quand la page l'envoie (un formulaire, `ADR-042` ; le toucher d'une valeur partagée, qui envoie l'état, `ADR-079` ; les touchers d'un membre connecté, `ADR-081`). Un fichier d'un autre serveur, ouvert par un passage, n'y touche pas : ce n'est pas le même site.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le mot | `session:` (le nom du web, `sessionStorage`) ; `keepVisit:` ; `remember:` ; `tab:` ; un bloc `visit: Visit(firstName: "")`, comme `Shared` ; **`visit:`** | **`visit:`** : un seul mot, le plus court, celui de la demande (« le temps d'une visite »), à côté de `keep:` qu'il complète. `session:` est un défaut pour un débutant : le web donne trois durées à ce mot (la session d'un compte, sur le serveur, que HoloCode a déjà avec le cookie `holo_session`, `ADR-081` ; un cookie « de session », jusqu'à la fermeture du navigateur ; `sessionStorage`, jusqu'à la fermeture de l'onglet), et le mot ne dit pas quand elle finit. `keepVisit:` est plus long et se confond avec `keep:` dans les propositions de l'éditeur. `remember:` ne dit pas combien de temps, et `keep` se souvient aussi. `tab:` dit l'onglet, mais se confond avec les onglets d'une page (leçon 114). Un bloc `Visit(…)` déclarerait les valeurs une seconde fois à côté de `State` : `Shared` a son bloc parce que la valeur change de propriétaire (le serveur, pour tous) ; ici elle reste au visiteur, seule sa durée change, comme pour `keep`, qui est une liste de noms |
| Rien de nouveau | `keep:` ; `address:` ; le serveur | `keep:` garde trop longtemps, et par adresse ; `address:` met un prénom dans l'adresse, l'historique, le lien partagé et les journaux du serveur ; le serveur demande un cookie, un envoi, et rien sans `holo serve` |
| La clé | l'adresse de la page, comme `keep` (`ADR-090`) ; son dossier ; **le nom de la valeur, pour tout le site** | la valeur doit suivre le visiteur d'une page à l'autre : son nom est ce que les pages ont en commun. Le site est celui du navigateur, une adresse de serveur. Un dossier couperait un formulaire dont les pages sont rangées à part (la leçon 136 a sa seconde page dans un dossier) |
| Une clé, ou une par valeur | tout sous une clé (`holo-visit`) ; **une clé par valeur** | chaque page n'écrit que ses noms, sans relire ni réécrire ceux des autres ; une valeur abîmée ne touche qu'elle |
| Le format | l'état du moteur (`prenom='Ada`, un nombre à virgule à l'échelle de la page : 1250) ; **du JSON** | le JSON dit la sorte (un texte, un nombre, une liste) ; un nombre à virgule s'écrit `12.50`, juste quelle que soit l'échelle de la page qui le lit ; le moteur sait déjà relire un JSON avec méfiance (`ADR-093`) |
| Une valeur qui ne va pas | la ramener à sa borne ; la couper ; refuser toute la visite, comme un import ; **l'ignorer, seule** | ramenée ou coupée, elle serait fausse ; un import est un seul fichier, la mémoire de visite est écrite par plusieurs pages : une valeur ignorée laisse passer les autres |
| Ce que la page écrit | tout, à chaque changement, comme `keep` ; **ce qui change chez elle** | une page qui ne comprend pas une valeur (une autre sorte), ou qui n'y touche pas, ne l'écrase pas avec son départ |
| Où la garder | un cookie ; le serveur ; **`sessionStorage`** | ni cookie, ni envoi, ni serveur ; le navigateur fait la durée (l'onglet) et la séparation (un onglet n'en voit pas un autre) |
| Sans JavaScript | le serveur garde la visite sous un cookie ; **rien** | ce serait un cookie et un envoi de chaque valeur, que la vie privée refuse ; la page reste juste, avec ses valeurs de départ |

## Ce qui est refusé, et pourquoi

- Une valeur à la fois dans `keep` et dans `visit` : « une valeur est gardée, ou retenue le temps de la visite, pas les deux ».
- Une valeur partagée (`shared`) : le serveur la garde pour tous ; `visit` retient ce qui est à un seul visiteur, dans son onglet.
- Une valeur de l'adresse : dans `address:`, elle viendrait de deux endroits, l'adresse et la visite (comme `keep`, `ADR-091`) ; une valeur du nom du fichier (`{id}`, `ADR-078`) vient déjà de l'adresse.
- Un nom inconnu, un nom écrit deux fois, autre chose qu'une liste de noms.
- L'heure du visiteur (`minute`, `today`), que le moteur redonne ; ce que le serveur dit du membre connecté (`account`, `signedIn`) ; une liste calculée, qui se refait d'après sa source.
- `visit:` dans un monde (`World`) : un monde partage les valeurs de sa page.

## Les défauts du web évités

- **`sessionStorage` écrit et relu à la main** (`setItem`, `getItem`, `JSON.parse`), sans rien vérifier : une valeur d'une autre sorte, abîmée ou trop grande casse la page, ou y entre telle quelle. Ici, une liste de noms, et le moteur vérifie chaque valeur.
- **Un cookie pour un brouillon** : envoyé au serveur avec chaque demande, gardé parfois des semaines, lisible par des tiers. Ici, aucun cookie, rien n'est envoyé.
- **Le brouillon dans l'adresse** : un prénom ou un e-mail dans l'historique, le lien partagé et les journaux du serveur. Ici, il reste dans l'onglet.
- **`localStorage` pour une étape de formulaire** : la saisie reste des jours sur un ordinateur partagé. Ici, elle part avec l'onglet.
- **Le formulaire qui remet, quand on revient en arrière, ce qu'on avait écrit avant d'aller le changer plus loin** (le navigateur remet les champs de la page rechargée) : ici, la mémoire de visite, plus récente, passe par-dessus.

## Corrigé en route

- `holo serve` et le serveur d'essai rendaient, à la même adresse `.holo`, soit la page (`text/html`), soit le fichier lui-même (`text/plain`, lu par le moteur), sans dire `Vary: Accept`. Quand le navigateur n'avait pas gardé la page dans son cache de « Précédent », il la rechargeait depuis son cache HTTP : il montrait le texte du fichier, reçu en dernier. Les deux serveurs disent maintenant `Vary: Accept`. Le défaut touchait toute page où le moteur était venu ; l'essai de la leçon 136 l'a montré en rechargeant l'étape 1 par « Précédent ».

## Dettes

- La page arrive avec ses valeurs de départ, puis prend celles de la visite : on peut voir « 1 » un instant avant « 3 », comme avec `keep`.
- Le navigateur peut rendre la mémoire de visite d'un onglet fermé qu'on rouvre (Ctrl+Maj+T, ou la reprise de la session du navigateur), et la copie dans un onglet qu'on duplique. C'est lui qui le fait, avec l'onglet.
- Le moteur ne voit qu'une page à la fois : deux pages qui donnent deux sortes au même nom ne sont pas refusées, chacune ignore la valeur de l'autre. `holo check` sur un dossier pourrait le dire.
- Une page n'écrit que ce qui change chez elle : une règle qui remet une valeur à son départ, quand la page a ignoré celle de la visite (une autre sorte), ne l'efface pas.
- Un fichier choisi (`Input(type: file)`) ne suit pas : seul son nom resterait. Il s'envoie depuis la page où on le choisit.
- Rien pour oublier toute la visite d'un coup : une règle remet chaque valeur à son départ (`prenom.set("")`), comme « Recommencer » dans la leçon.

## Critères de validation

- Tests du moteur (`moteur/src/visit.rs`) :
  - `a_value_follows_the_visitor_from_one_page_to_the_other` : le JSON écrit (un texte avec des guillemets, un nombre, un nombre à virgule, un texte de deux lignes, un choix, une liste), repris par une autre page à son échelle (`9.50` → `9.5`), et revenu ;
  - `a_value_of_another_sort_or_out_of_bounds_is_ignored` : dix-sept valeurs fausses ou abîmées ignorées une à une, un texte trop long jamais coupé, une case, une date ;
  - `a_list_is_read_back_like_an_import` : des textes, des fiches avec exactement leurs champs, 201 éléments refusés, une liste de plus de 64 Ko écrite vide ;
  - `what_the_visit_cannot_remember_is_refused_with_its_reason` : chaque refus, avec sa raison et sa ligne ;
  - `the_light_page_knows_what_the_page_remembers` : `data-visit-names`, la page ne devient pas vivante ; les deux pages de la leçon acceptées.
- Dans le serveur (`moteur/src/server.rs`) : `a_page_and_its_source_say_they_vary_with_accept`, sans cookie.
- Dans Chrome : « se souvenir le temps d'une visite : d'une page à l'autre du site, pas dans un autre onglet (leçon 136) » :
  - la page 1 retient ce qu'on écrit, même avant l'arrivée du moteur ;
  - la page 2, par le lien, dans le même onglet, a les valeurs ;
  - « Précédent » deux fois : la page 1 suit, rendue par le cache puis rechargée ;
  - un nouvel onglet ne les a pas, et n'y fait même pas venir le moteur ;
  - « Recommencer » est retenu ;
  - des valeurs abîmées ou étrangères sont ignorées, sans erreur, et laissées telles quelles ;
  - une page où `prenom` est un nombre l'ignore ;
  - aucun cookie, rien d'envoyé au serveur.
- Leçon `136-se-souvenir-le-temps-d-une-visite.holo`, et sa seconde page `136-inscription/etape-2.holo`.
