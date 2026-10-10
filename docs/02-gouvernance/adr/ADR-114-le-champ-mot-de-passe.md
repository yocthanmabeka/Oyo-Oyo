# ADR-114 — Le champ mot de passe : `Input(type: password, label:)`, que la page ne lit jamais

- Statut : ACCEPTÉ (Yocthan a dit « Oui » le 2026-10-09 à l'ouverture sous ces conditions)
- Date : 2026-10-10
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #246 (« le champ mot de passe »), l'une des huit fonctions ouvertes sous conditions (issues 246 à 253) ; les deux avis, avec leurs sources : `proposals/Claude/contraintes-2026-10-09/README.md` (fonction 2 : NIST SP 800-63B-4, S8 à S10 ; l'hameçonnage ; un mot de passe gardé en clair, dans un journal, l'historique ou une valeur partagée ; la forme `purpose:` et le type `Secret`) et `proposals/Gemini/contraintes-2026-10-09/README.md` (fonction 2 : NIST et l'ANSSI, le collage et l'affichage temporaire, 64 caractères ; la « boîte noire » ; `allowPaste:`, `revealable:`) ; les comptes `ADR-081` à `ADR-083` (Argon2id, le frein, les pages de compte du moteur) ; `ADR-082` (`HOLO_ORIGIN`, HTTPS derrière le proxy de l'auteur) ; les formulaires `ADR-042`, `ADR-068`, `ADR-075` ; `ADR-074` (sans JavaScript) ; `ADR-054` (`?values`), `ADR-091` (l'adresse), `ADR-113` (`visit`).
- Validation : Yocthan, le 2026-10-09, « Oui », à l'ouverture des huit fonctions, sous les conditions de l'issue.
- Projets affectés : HoloCode, HoloEngine, le serveur (`holo serve`)

## Contexte

- Seules les pages de compte, fabriquées par le moteur (`ADR-081`), avaient un champ mot de passe. Un auteur qui voulait en demander un n'avait qu'un champ ordinaire, `Input(value: motDePasse, label: "Mot de passe")` : le mot de passe se voyait à l'écran, entrait dans les valeurs de la page (et donc dans `keep`, `visit`, l'adresse, le panneau `?values`), partait dans le message, et `holo serve` le rangeait en clair. L'essai d'`ADR-063` comparait même deux « mots de passe » écrits dans des valeurs.
- Le web, par défaut : `<input type="password">` cache les lettres, mais le JavaScript de la page lit sa valeur, et tout script de la page avec lui. Trop de sites empêchent de coller (le gestionnaire de mots de passe ne marche plus), imposent des règles de composition (une majuscule, un chiffre, un signe), coupent en silence ce qui dépasse `maxlength`, oublient `autocomplete`, envoient le mot de passe dans un journal, un outil de mesure d'audience, ou en `http://`.
- Les cas réels, hors des pages de compte (avis Claude) : confirmer une action sensible avec le mot de passe de son compte ; protéger un document ; un espace qui a ses propres règles.
- NIST SP 800-63B-4 (finalisée en août 2025), d'après les sources de l'avis Claude (S8 à S10) : aucune règle de composition ; accepter au moins 64 caractères ; compter chaque caractère Unicode pour un ; ne jamais couper le mot de passe ; permettre le collage et les gestionnaires de mots de passe ; offrir de montrer ce qu'on tape ; 15 caractères au moins si le mot de passe est le seul facteur (8 avec un second facteur) ; comparer un nouveau mot de passe à une liste de mots de passe courants ou volés ; le garder salé et haché par une fonction faite pour cela ; freiner les essais. Le texte officiel (`pages.nist.gov/800-63-4/sp800-63b.html`) n'a pas pu être relu depuis ce conteneur, le 2026-10-10 : le proxy refuse ce site.
- Les deux avis concluent : ouvrir sous conditions, et c'est le moteur qui tient les règles, jamais l'auteur.

## Décision

```holo
Page(
  title: "Protéger un carnet",
  state: State(title: "", sent: 0),
  children: [
    Form(name: Protect, children: [
      Input(value: title, label: "Le nom du carnet", required: true),
      Input(type: password, new: true, label: "Un mot de passe pour ce carnet"),
      Button(name: Send, text: "Protéger"),
    ]),
    If(sent, is: 1, children: [ P("Le carnet est protégé.") ]),
  ],
  rules: [ On(Send.tap, effect: Protect.send), On(Protect.sent, effect: sent.set(1)) ],
)
```

```holo
Page(
  title: "Ma réservation",
  access: members,
  state: State(booked: 1),
  children: [
    Form(name: Cancel, children: [
      Input(type: password, label: "Ton mot de passe, pour confirmer"),
      Button(name: Confirm, text: "Annuler ma réservation"),
    ]),
  ],
  rules: [ On(Confirm.tap, effect: Cancel.send), On(Cancel.sent, effect: booked.set(0)) ],
)
```

1. **`Input(type: password, label: "…")`, dans un `Form`.** Un seul par formulaire. Deux sortes, que le moteur distingue pour le navigateur, le gestionnaire de mots de passe et le serveur :
   - **sans `new`, le mot de passe du compte du visiteur** (`ADR-081`) : la page est réservée aux membres (`access: members`) ; `holo serve` le vérifie, puis l'oublie. Le moteur pose `autocomplete="current-password"`. C'est la confirmation d'une action sensible ;
   - **`new: true`, un mot de passe que le visiteur choisit** : l'auteur n'en reçoit que l'empreinte Argon2id. Le moteur pose `autocomplete="new-password"`, et l'explication « 12 caractères au moins : une phrase que toi seul connais est un bon mot de passe. », reliée au champ.
2. **La page ne lit jamais ce qui est tapé.** Le champ n'a pas de valeur : `value:` est refusé. Aucun nom ne le désigne : ni une règle, ni un texte `{…}`, ni l'état, ni `keep`, `visit`, `Shared`, l'adresse (`address:`, `?…`), ni le panneau `?values` ne l'atteignent. Il n'a ni `name` ni `data-bind` dans la page : aucun formulaire ordinaire ne l'envoie, le moteur ne le range dans aucune valeur, et le toucher renvoyé d'un membre (`?mirror`, `ADR-081`) ne le porte pas.
3. **Il part seulement avec son formulaire, vers le serveur de la page.**
   - Avec JavaScript, le moteur du navigateur le lit une seule fois, au moment d'envoyer, et le met dans l'envoi, à côté des valeurs, jamais parmi elles : `{"form":"Protect","values":{"title":"Mon carnet"},"password":"…"}`. Montré, il est d'abord caché de nouveau ; parti ou refusé, il est effacé du champ.
   - Sans JavaScript, il est rattaché au formulaire des gestes (`ADR-074`) sous le nom de son formulaire, `holo-password-Protect`. `holo serve` le prend à part avant tout le reste : il ne passe jamais par l'arbitre, ni dans l'état, ni dans l'adresse. Il ne s'en sert que pour l'envoi de son formulaire.
4. **Seulement en HTTPS, ou sur ce PC.**
   - La page légère demande au navigateur si la page est sûre (`isSecureContext`) : sinon, le champ se ferme (`disabled`), son bouton « Montrer » disparaît, et une note, reliée au champ, dit pourquoi : « Ce champ ne s'ouvre qu'en HTTPS : un mot de passe ne part jamais en clair. »
   - `holo serve` fait de même pour un visiteur sans JavaScript, et refuse un envoi forgé : une demande est sûre si elle vient de ce PC (le pair TCP, jamais un en-tête) et que son adresse est `localhost`, ou celle que l'auteur a déclarée derrière son proxy HTTPS (`HOLO_ORIGIN`, comme les clés d'accès, `ADR-082`). Un téléphone qui parle au serveur en `http://`, par le Wi-Fi, ne l'est pas : son mot de passe passerait en clair sur le réseau. Le journal le dit à l'auteur, sans le mot de passe : « Mot de passe refusé : …, la demande n'est pas en HTTPS ; derrière un proxy HTTPS, fixer HOLO_ORIGIN ».
5. **`holo serve` ne le garde jamais en clair.**
   - Un nouveau mot de passe : son empreinte Argon2id, celle des comptes (`accounts::password_print` : 19 Mo, deux passes, un sel tiré au hasard), à la place du mot de passe dans le message : `"password":"$argon2id$v=19$m=19456,t=2,p=1$…"`. `holo messages` la montre ; c'est une écriture standard (PHC), que savent vérifier les bibliothèques Argon2 (et Django, par exemple).
   - Le mot de passe du compte : vérifié contre l'empreinte du compte, avec le frein des comptes (cinq essais ratés, puis une attente qui double à chaque échec, une heure au plus ; pendant l'attente, même le bon est refusé). Juste, le message part sans rien du mot de passe ; faux, « Ce n'est pas le mot de passe de ton compte. », sous le champ.
   - Le frein par adresse des comptes (`ADR-083`, trente envois par minute) vaut aussi : chaque envoi calcule une empreinte, lente exprès.
   - Sans JavaScript, l'empreinte se calcule avant le verrou des gestes : les autres visiteurs n'attendent pas. Un envoi refusé ne garde que le code du refus (`Protect:short`, `Cancel:wrong`), jamais le mot de passe, pour écrire le message sous le champ.
   - Ni la base, ni son journal d'écriture (WAL), ni une sauvegarde, ni une réponse, ni le journal de `holo serve` ne contiennent jamais le mot de passe. Les messages d'erreur disent ce qui ne va pas, jamais ce qui a été tapé.
6. **Les longueurs, sans règle de composition** (NIST SP 800-63B) : de 12 à 128 caractères pour un nouveau mot de passe, comme les comptes ; aucun minimum pour vérifier celui du compte (le compte l'a déjà fixé), 128 au plus. Chaque caractère Unicode compte pour un, partout (le navigateur et le serveur comptent les mêmes points de code). **Rien n'est coupé en silence** : pas de `maxlength` ; un mot de passe trop long est refusé, avec la raison.
7. **Ce que le moteur pose toujours**, sans réglage :
   - le collage n'est jamais empêché ;
   - un bouton « Montrer », à côté du champ : un vrai bouton, au doigt, à la souris, au clavier (Tab, puis Espace ou Entrée) ; son nom ne change pas (« Montrer le mot de passe ») et le lecteur d'écran dit son état, enfoncé ou non (`aria-pressed`) ; montré par la page légère, car sans JavaScript il ne pourrait rien faire ;
   - `autocomplete`, `aria-required`, l'étiquette ; un message sous le champ, relié à lui, qui suit ce qu'on corrige ; le clavier revient au champ, le lecteur d'écran dit le message.
8. **Un champ ordinaire qui demande un mot de passe est refusé**, avec la bonne écriture : son étiquette (« Mot de passe », « Password », « mdp »…) ou le nom de sa valeur (`motDePasse`, `password`, `pwd`…) le disent. « Passeport » ou « Code promo » restent des champs ordinaires.
9. **Le serveur d'essai** (`node outils/server.mjs`) ne sait ni vérifier ni hacher : il refuse un envoi qui porte un mot de passe (`501`, « un formulaire avec un mot de passe demande holo serve »), sans le passer au moteur ni l'écrire nulle part.
10. **Ce que devient l'existant : les pages de compte restent, et partagent la même règle.** Créer un compte, se connecter, effacer son compte restent les pages du moteur (`ADR-081` à `ADR-083`) : `Input(type: password)` ne crée jamais de compte et n'ouvre jamais de session ; il vérifie seulement le mot de passe d'un membre déjà connecté. Les deux partagent les longueurs (`password::MIN`, `password::MAX`, que les comptes lisent désormais), l'empreinte Argon2id et le frein. Les pages de compte gardent leurs champs HTML, sans script : leur politique de sécurité (`default-src 'none'`) n'en permet aucun, donc pas encore de bouton « Montrer » (dette).

## Les règles de sécurité, tenues par le moteur

| Règle | Comment | Essai (ce qui doit être refusé l'est) |
|---|---|---|
| 1. La page ne lit jamais ce qui est tapé | aucune valeur, aucun nom, ni `name` ni `data-bind` ; le moteur ne reçoit que la longueur, pour ses messages | `password_tests::a_password_field_is_never_a_value_of_the_page` : `value:` (même une valeur partagée), `{password}`, `{password:length}`, `If(password…)`, `keep:`, `visit:`, `address:`, une règle `password.set(…)` refusés ; ni `data-bind`, ni `name`, ni `value`, ni `maxlength` dans le champ fabriqué ; l'état de départ, l'envoi fabriqué, un geste sans JavaScript qui porte le champ, une adresse `?password=…` ne changent rien. Dans Chrome : le mot de passe collé est cherché dans le panneau `?values`, `localStorage`, `sessionStorage`, l'adresse, le texte et le HTML de la page, et dans les demandes : nulle part avant l'envoi, ni après |
| 2. Il part seulement vers le serveur de la page, une fois, à côté des valeurs | le moteur du navigateur le lit au moment d'envoyer ; sans JavaScript, `holo serve` le sort des champs avant tout le reste | dans Chrome, une seule demande le porte : `POST` à l'adresse de la page, `form,values,password`, les valeurs sans lui ; caché de nouveau avant, effacé après. `server::password_tests::a_new_password_is_kept_only_as_its_print_with_and_without_javascript` (sans JavaScript : l'adresse de retour et l'état sans lui) |
| 3. Seulement en HTTPS, ou sur ce PC | la page légère (`isSecureContext`) et `holo serve` (le pair TCP, `localhost`, ou `HOLO_ORIGIN`) ferment le champ ; `holo serve` refuse un envoi forgé | `server::password_tests::a_password_never_travels_in_clear` (un téléphone sur le Wi-Fi, un proxy en HTTP sur le PC, `HOLO_ORIGIN`, une autre adresse, un autre pair). Dans Chrome, la leçon ouverte par l'adresse du réseau local, servie par `holo serve` puis par le serveur d'essai (là, seule la page légère protège) : fermé, la note dite ; un envoi forgé refusé (`422`), rien rangé, le journal dit la raison ; sans JavaScript, fermé aussi |
| 4. Jamais gardé en clair | l'empreinte Argon2id d'un nouveau ; celui du compte vérifié, puis oublié ; un refus sans JavaScript garde son code, jamais le mot de passe | `server::password_tests` : la base, son WAL et une sauvegarde relus octet par octet ; l'empreinte se vérifie avec le mot de passe. Dans Chrome : `holo messages` (l'empreinte seule), les fichiers de `holo-data`, le journal de `holo serve` |
| 5. Celui du compte est vérifié, avec le frein | `accounts::confirm` : l'empreinte du compte, hors du verrou de la base ; cinq essais, puis l'attente | `server::password_tests::the_password_of_the_account_is_checked_with_the_brake_of_the_accounts` (avec et sans JavaScript ; le bon refusé pendant l'attente ; `401` sans compte). Dans Chrome : un compte créé, le faux refusé sous le champ (effacé, le clavier dessus), le juste envoie, le message rangé sans rien du mot de passe |
| 6. Les longueurs de NIST, rien de coupé | 12 à 128 pour un nouveau ; 128 au plus pour celui du compte ; des points de code ; pas de `maxlength` | `password_tests::the_lengths_follow_nist_without_composition_rules` (11, 12, 64, 128, 129 ; douze lettres accentuées ; douze fois la même) ; `server::password_tests::forged_submissions_are_refused_without_echoing_the_password` (129 refusé avant toute empreinte, 64 accepté, sans aucune règle de composition) ; dans Chrome, « court » refusé, rien ne part |
| 7. Le collage jamais empêché | aucun gestionnaire de collage dans le moteur | dans Chrome, un vrai collage (le presse-papiers, puis Ctrl+V) : le champ reçoit le texte, le collage n'est pas empêché |
| 8. « Montrer », au clavier et au lecteur d'écran | un vrai bouton, `aria-pressed`, `aria-controls`, son nom fixe | dans Chrome : Tab depuis le champ, Espace, puis Espace : l'arbre d'accessibilité dit « Montrer le mot de passe », `false`, puis `true`, puis `false` ; le champ en texte, puis caché ; au doigt aussi |
| 9. `autocomplete` juste | `current-password`, `new-password`, posé par le moteur | `password_tests::the_field_helps_password_managers_keyboards_and_screen_readers` ; dans Chrome, les deux pages |
| 10. Les envois forgés refusés, sans écho | le serveur relit l'envoi : un seul mot de passe, en texte, seulement si le formulaire en a un, jamais parmi les valeurs | `server::password_tests::forged_submissions_are_refused_without_echoing_the_password` (absent, deux fois, un nombre, parmi les valeurs, trop long, pour un formulaire qui n'en a pas) : chaque réponse est relue, elle ne contient jamais ce qui a été tapé |
| 11. Un champ ordinaire qui demande un mot de passe | son étiquette ou le nom de sa valeur le disent | `password_tests::a_plain_field_that_asks_for_a_password_is_refused` (sept écritures refusées ; « Passeport », « Code promo », « Passe-temps », « Le mot de la fin » acceptés) |

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| L'écriture | un bloc `Password(…)` ; `Secret(…)` ; **`Input(type: password)`** (HTML, l'issue, les deux avis) | un champ comme les autres (`type: email`, `type: date`), le mot que connaissent ceux qui ont vu le web, et ce qu'écrit l'issue ; un bloc nouveau ferait deux façons d'écrire un champ |
| Ce que le visiteur donne | rien (le moteur devine d'après la page) ; `purpose: current \| new \| confirm` (avis Claude) ; `autocomplete:` laissé à l'auteur ; **`new: true`** | deviner (« une page réservée, donc le mot de passe du compte ») se trompe sur une page réservée où l'on choisit un mot de passe ; `autocomplete:` laisse l'auteur se tromper, et c'est un mot de programmeur ; `purpose:` est un mot nouveau de plus pour une seule chose : nouveau ou pas. `new: true` se lit comme une phrase (« un nouveau mot de passe »), comme `newTab: true` ou `required: true` ; sans lui, c'est le mot de passe du compte, le cas qui ne garde rien, pas même une empreinte |
| Taper deux fois | `purpose: confirm` (avis Claude), le serveur compare les deux ; **un seul champ, et « Montrer »** | la page ne peut pas comparer deux mots de passe qu'elle ne lit pas ; NIST demande d'offrir de montrer ce qu'on tape, pas de le taper deux fois ; sur un téléphone, une seconde saisie coûte. Un second champ est refusé, avec la raison |
| Où vit le mot de passe dans la page | une valeur, comme le web ; une valeur spéciale que seul le serveur lit (le type `Secret` de l'avis Claude) ; **aucune valeur** | une valeur, même spéciale, a un nom : un jour, une règle, `keep` ou un module la liront. Sans nom, rien ne peut l'atteindre, et il n'y a rien à garder fermé |
| Ce que fait le serveur | un type `Secret` que l'auteur hache ou compare, avec son code (avis Claude) ; **le moteur décide** : un nouveau, son empreinte ; celui du compte, vérifié | HoloCode n'a pas de code de serveur écrit par l'auteur : le moteur fait les deux seules choses sûres, et l'auteur ne peut pas se tromper |
| Le collage et « Montrer » | `allowPaste:` et `revealable:` (avis Gemini) ; **toujours là, sans réglage** | un réglage qui les retire ne servirait qu'à refaire le défaut du web |
| HTTPS | rien ; seulement le navigateur ; seulement le serveur ; **les deux** | sans JavaScript, seul `holo serve` sait ; avec JavaScript, seul le navigateur sait vraiment si la page est sûre (un proxy HTTPS, le serveur d'essai) ; le serveur refuse aussi un envoi forgé. La même règle que les clés d'accès (`HOLO_ORIGIN`) |
| Fermer ou laisser écrire, hors HTTPS | laisser écrire puis refuser à l'envoi ; **fermer le champ, et dire pourquoi** | un mot de passe qu'on ne peut pas taper ne part pas par erreur ; la note dit pourquoi, au lecteur d'écran aussi |
| Les longueurs | 8 (NIST, avec un second facteur) ; 15 (NIST-4, un seul facteur) ; **12 à 128, la règle des comptes** | une seule règle pour tout le moteur (l'issue : ne pas créer une seconde façon de faire la même chose) ; 12 est sous les 15 de NIST-4 pour un mot de passe seul : la question est posée à Yocthan, pour les comptes et ce champ ensemble (dette) |
| Trop long | couper (`maxlength`, le web) ; **refuser, avec la raison** | NIST : ne jamais couper ; un mot de passe collé par un gestionnaire, coupé en silence, ne marcherait plus jamais |
| Ce que devient l'existant | refaire les pages de compte avec ce champ ; **les garder, avec la même règle, la même empreinte, le même frein** | les pages de compte n'ont aucun script, exprès (`default-src 'none'`) : elles marchent pareil partout, et rien ne s'y glisse. Ce champ ne crée ni compte ni session : il n'y a pas deux façons de se connecter |
| Le serveur d'essai | hacher aussi (Argon2id en JavaScript, ou demander à `holo`) ; garder en clair ; **refuser (`501`)** | une seconde écriture d'Argon2id serait une seconde cryptographie à tenir ; garder en clair, jamais. Les comptes y répondent déjà `501` |
| Un champ ordinaire qui demande un mot de passe | le laisser ; avertir ; **le refuser, avec la bonne écriture** | « le moteur tient les règles, jamais l'auteur » : un débutant écrirait `Input(value: motDePasse, …)` et ne verrait rien. Des mots sans ambiguïté seulement |

## Ce qui est refusé, et pourquoi

- **Dans la page**, avec sa raison, par `holo check` :
  - `value:` sur un mot de passe (même une valeur partagée) : la page ne le lit jamais ;
  - un mot de passe hors d'un `Form`, deux dans un `Form`, un mot de passe dans une liste répétée (`Repeat`) ;
  - le mot de passe du compte (sans `new`) sur une page qui n'est pas réservée aux membres ;
  - `min:`, `max:` (les longueurs sont celles du moteur), `required:` (il l'est toujours), `suggestions:`, `lines:`, `accept:`, un paramètre inconnu ou sans nom, une étiquette absente ou vide ;
  - `new:` autre que `true` ou `false`, ou sur un champ qui n'est pas un mot de passe ;
  - un champ ordinaire dont l'étiquette ou la valeur demande un mot de passe ;
  - et, faute de nom, tout ce qui voudrait le lire : `{…}`, `If`, `keep:`, `visit:`, `address:`, une règle.
- **À l'envoi**, par le moteur du navigateur puis par `holo serve` : vide, trop court (un nouveau), trop long, hors HTTPS ; par `holo serve` seulement : absent, deux fois, autre chose qu'un texte, parmi les valeurs, pour un formulaire qui n'en a pas ; ce n'est pas celui du compte ; pendant l'attente du frein ; trop d'envois depuis la même adresse ; un visiteur qui n'est plus connecté.

## Les défauts du web évités

- **Le script de la page qui lit le mot de passe** : ici, la page n'a aucun moyen de le nommer.
- **Le mot de passe dans l'état, l'adresse, l'historique, le stockage du navigateur, un outil de mesure** : aucune valeur, rien à garder.
- **Le collage empêché** : jamais.
- **Les règles de composition** (une majuscule, un chiffre, un signe) : aucune.
- **`maxlength` qui coupe en silence** : jamais ; trop long est refusé, avec la raison.
- **`autocomplete` oublié**, ou `autocomplete="off"` : le moteur pose toujours le bon.
- **Un « œil » sans nom ni état** pour le lecteur d'écran : un vrai bouton, nommé, `aria-pressed`.
- **Le mot de passe en `http://`** sur le Wi-Fi : le champ se ferme.
- **Le mot de passe gardé en clair, ou dans un journal** : l'empreinte seule, et le journal ne dit que la raison d'un refus.
- **Un champ texte qui demande un mot de passe** (`<input type="text" name="password">`) : refusé.

## Limites, honnêtement

- **L'hameçonnage** : un auteur peut imiter la page de connexion d'un autre site. HoloCode empêche le pire (le mot de passe ne part que vers le serveur de l'auteur, et seulement son empreinte est gardée), pas un faux site entier.
- **Un nouveau mot de passe n'est encore vérifié par rien dans HoloCode** : l'auteur reçoit son empreinte, dans une écriture standard (PHC), pour un outil à lui. Une page protégée par un mot de passe choisi serait une autre décision.
- **Sans JavaScript** : pas de bouton « Montrer » ; le mot de passe part avec tout toucher de la page (le formulaire des gestes envoie tous les champs), vers `holo serve` seulement, qui ne s'en sert que pour l'envoi de son formulaire, et l'oublie sinon ; le champ se vide à chaque toucher.
- **Le mot de passe traverse la mémoire** du navigateur et de `holo serve` le temps de l'envoi, comme partout ; il n'est écrit nulle part.
- **Un proxy en HTTP, sur le PC de l'auteur, qui récrirait l'adresse en `localhost`** serait cru sûr : la règle suppose que ce qui parle depuis ce PC, à `localhost`, est le navigateur de l'auteur.
- **Le frein par adresse est partagé avec les comptes** (trente envois par minute) : un réseau partagé (une école) peut l'atteindre.
- **Rien n'a été essayé avec un vrai gestionnaire de mots de passe** (celui de Chrome, Bitwarden, 1Password) ni sur un vrai téléphone : `autocomplete` et l'arbre d'accessibilité sont vérifiés, pas la proposition du gestionnaire.
- `type: password` sur une page qui n'est pas fabriquée par `holo serve` ni par le serveur d'essai (une page posée sur un hébergeur sans serveur) s'ouvre en HTTPS, mais son envoi ne trouve personne pour le recevoir (`failed`).

## Dettes

- **La liste des mots de passe courants ou volés** (NIST : un nouveau mot de passe est comparé à une telle liste), pour ce champ et pour les comptes : il faut choisir une liste et sa licence (question pour Yocthan).
- **15 caractères pour un mot de passe seul facteur** (NIST SP 800-63B-4) : aujourd'hui 12, pour les comptes et pour ce champ, une seule règle (question pour Yocthan).
- **Le bouton « Montrer » sur les pages de compte** : elles n'ont aucun script (`default-src 'none'`) ; il faudrait un petit script permis par son empreinte (`script-src 'sha256-…'`).
- Changer le mot de passe de son compte (dette d'`ADR-083`) : sa place est la page du compte, pas une page de l'auteur.
- Vérifier plus tard un mot de passe choisi (`new: true`) : une page ou un fichier protégés par lui.
- Le serveur d'essai ne reçoit pas de mot de passe.

## Critères de validation

- Tests du moteur (`cargo test --release --locked`) :
  - `password::password_tests` : `a_password_field_is_never_a_value_of_the_page`, `a_password_field_is_read_strictly`, `a_plain_field_that_asks_for_a_password_is_refused`, `the_lengths_follow_nist_without_composition_rules`, `the_field_helps_password_managers_keyboards_and_screen_readers` ;
  - `server::password_tests` : `a_new_password_is_kept_only_as_its_print_with_and_without_javascript`, `the_password_of_the_account_is_checked_with_the_brake_of_the_accounts`, `a_password_never_travels_in_clear`, `forged_submissions_are_refused_without_echoing_the_password`.
- Les essais savent échouer. Chaque mutation a été faite puis retirée :
  - dans le moteur, `value:` accepté sur un mot de passe : `a_password_field_is_never_a_value_of_the_page` rate ; toute demande crue sûre : `a_password_never_travels_in_clear` rate ;
  - dans Chrome, la page légère ne montre plus « Montrer » : « le bouton « Montrer », montré par la page légère : caché » ; elle ne ferme plus le champ hors HTTPS : « hors HTTPS, la page légère ferme le champ : false false true false » ; un empêchement du collage : « coller : false true » ; `holo serve` range l'envoi tel qu'il arrive : le mot de passe lu dans `holo messages` et dans `site.sqlite-wal`.
- Dans Chrome : « le champ mot de passe : la page ne le lit jamais ; il part seulement vers holo serve, qui n'en garde que l'empreinte ; le collage, « Montrer » au clavier et au lecteur d'écran, autocomplete ; celui du compte vérifié ; hors HTTPS, fermé ; sans JavaScript (leçon 137, serve) ».
- La leçon `137-un-mot-de-passe.holo`, et sa suite `137-mot-de-passe/annuler.holo` (le mot de passe du compte) : `holo check` les accepte.
