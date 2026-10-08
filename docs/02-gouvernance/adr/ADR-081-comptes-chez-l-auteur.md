# ADR-081 — Lot 7 du web : des comptes chez l'auteur (un mot de passe, un code à 6 chiffres), une page réservée, le panier qui suit le compte

- Statut : PROPOSITION (construit et essayé ; se connecter par un mot de passe puis un code à 6 chiffres, tout chez l'auteur, est le choix de Yocthan ; l'écriture dans la page et le reste attendent sa validation)
- Date : 2026-10-08
- Responsable : Yocthan Mabeka
- Discussions sources : la proposition `proposals/Claude/serveur-et-comptes-2026-10-07.md` et ses questions ; la réponse de Yocthan du 2026-10-08 (« je suis d'accord avec tes recommandations ») ; le plan en neuf lots, lot 7 (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md` : « un compte local, son panier sur deux appareils, un code à 6 chiffres ») ; sa règle « chez soi d'abord » (2026-10-07) et sa règle de parité (ce qui marche sur un téléphone marche sur un ordinateur, et l'inverse) ; le serveur des `ADR-074` (les boutons sans JavaScript), `ADR-075` (les formulaires) et `ADR-078` (les adresses).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine, le serveur

## Contexte

- `holo serve` gardait l'état de chaque visiteur sous un numéro tiré au hasard, dans un cookie (`ADR-074`) : sur un autre appareil, le visiteur repartait de zéro. Personne n'avait de compte.
- Yocthan veut des comptes **gardés par le serveur de l'auteur, dans sa propre base, comme avec Django**, sans aucun prestataire obligatoire (Google, Apple, Microsoft, un service d'e-mails) ; **se connecter par un mot de passe et un code à 6 chiffres d'abord**, les clés d'accès ensuite ; des pages de compte qui marchent **aussi sans JavaScript**.

## Décision

1. **Les comptes sont gardés par `holo serve`**, dans la base du site (`holo-data/site.sqlite`, jamais servie) : un nom, l'empreinte du mot de passe, la clé du code à 6 chiffres s'il est activé. Rien ne part ailleurs.
2. **Se connecter** : un nom et un mot de passe, puis, si le membre l'a activé, **le code à 6 chiffres** que calcule une application d'authentification de son téléphone (Aegis, FreeOTP, 2FAS, Google Authenticator…), sans Internet ni SMS : la norme ouverte TOTP (RFC 6238). Pour l'activer, la page du compte montre une clé à recopier dans l'application (et un lien `otpauth://` qui l'ouvre directement sur le téléphone), puis demande le premier code.
3. **Dans la page**, trois mots :
   - **`Page(access: members)`** : la page n'est montrée qu'aux personnes connectées. Les autres sont menées à « Se connecter », qui dit pourquoi, puis ramenées. Rien de la page ne leur arrive avant : ni la page fabriquée, ni son texte (le moteur du navigateur ne peut pas le lire), ni ses gestes, ni ses formulaires. Sans `access`, la page est à tout le monde : c'est `access: everyone`, qu'on n'écrit pas.
   - **`signedIn`** (1 quand le visiteur est connecté, 0 sinon) et **`{account}`** (son nom, vide sinon) : la page les lit comme ses autres valeurs (`If(signedIn, is: 1, …)`, `P("Bonjour, {account}")`), **sans jamais pouvoir les changer** : `signedIn.set(1)`, `Input(value: account)`, `State(account: …)`, `keep: [account]`, un module ou un glissement qui les écrirait sont refusés, avec la raison ; des données reçues (`Data`) ne les écrivent pas ; un état écrit ne les relit jamais (comme l'heure). C'est le serveur qui les donne, à chaque visite.
   - Le nom arrive au moteur comme les valeurs d'une adresse (`ADR-078`) : un petit fichier joint après la page, `@account`, qui dit `name=Ada`. `holo serve` le joint ; le moteur de la page le joint aussi, d'après l'en-tête que le serveur a posé (`<meta name="holo-account">`) ; partout ailleurs (`holo check`, l'éditeur, le serveur d'essai), rien n'est joint et la page se lit comme pour un visiteur qui n'est pas connecté. Une page qui ne lit pas ces valeurs ne les reçoit pas.
4. **Les pages de compte sont fabriquées par le moteur**, en HTML ordinaire, sans aucun script : `/account/signup` (créer un compte), `/account/signin` (se connecter), `/account/code` (le code, après le mot de passe), `/account` (le compte : activer ou retirer le code, se déconnecter). Elles marchent de la même façon sur un téléphone et sur un ordinateur, avec ou sans JavaScript, en clair comme en sombre. Accessibles : une étiquette par champ, les explications et l'erreur reliées au champ (`aria-describedby`, `aria-invalid`), l'erreur en tête lue tout de suite par un lecteur d'écran (`role="alert"`), le champ fautif qui prend le focus, des cibles de 48 pixels, `autocomplete` (`username`, `new-password`, `current-password`, `one-time-code`) pour que le navigateur ou le gestionnaire de mots de passe sache quoi proposer. **Ce sont les seuls liens qui partent de la racine du site** : `A("Se connecter", to: "/account/signin")`, `A("Créer un compte", to: "/account/signup")`, `A("Mon compte", to: "/account")` ; les autres liens gardent leur règle (`/secret.holo` reste refusé). Un lien « Se connecter » ramène à la page d'où l'on vient.
5. **Le panier suit le compte.** Quand le visiteur est connecté, l'état de chaque page est gardé sous son compte (`account:7`), et non plus sous son cookie : il le retrouve sur son téléphone et sur son ordinateur. Ce qu'il avait fait avant de se connecter le suit (pour chaque page où le compte n'avait encore rien), puis quitte son cookie : après la déconnexion, l'appareil repart d'une visite neuve. Ce que garde un compte n'est pas oublié au bout de trente jours, comme une visite.
   - Sans JavaScript, rien de nouveau : chaque toucher part au serveur, qui calcule avec le même arbitre (`ADR-074`).
   - Avec JavaScript, la page calcule tout de suite, puis renvoie le toucher, avec ses champs, au serveur (`POST …?mirror`), un après l'autre ; le serveur le rejoue avec le même arbitre sur l'état du compte, sans rien envoyer d'autre (le moteur s'est chargé de l'envoi d'un formulaire), et répond `204`. **Le serveur ne reçoit jamais de valeurs, seulement des gestes** : le visiteur ne peut pas écrire l'état qu'il veut. Seulement pour un membre connecté : pour un visiteur, rien ne change.
6. **La sécurité**, essayée par les tests :
   - **Les mots de passe ne sont jamais gardés en clair** : seulement leur empreinte Argon2id (19 Mo de mémoire, deux passes, un sel tiré au hasard), qu'on ne peut pas défaire. 12 caractères au moins (une phrase est un bon mot de passe), 128 au plus.
   - **Le message ne dit jamais si c'est le nom ou le mot de passe qui est faux** (« Ce nom et ce mot de passe ne vont pas ensemble. »), et le serveur prend le même temps dans les deux cas (un nom inconnu est vérifié contre une empreinte pour rien).
   - **Un frein contre les essais répétés** : cinq essais ratés pour un nom (qu'il existe ou non), puis une minute d'attente, qui double à chaque nouvel échec, une heure au plus ; pendant l'attente, même le bon mot de passe est refusé. De même pour le code, par compte.
   - **Un code ne sert qu'une fois** : le serveur accepte le code des 30 secondes en cours et de leurs deux voisines (une horloge de téléphone un peu en avance ou en retard), jamais un code déjà servi ni un plus ancien.
   - **Une session** est un numéro de 128 bits tiré au hasard par le système, dans un cookie `HttpOnly` (la page ne le lit pas) et `SameSite=Lax` (un autre site ne l'envoie pas avec un formulaire). La base n'en garde que l'empreinte : une copie de la base, ou une sauvegarde, ne permet pas d'entrer. Un nouveau numéro à chaque connexion. Une session volée expire : oubliée après 14 jours sans visite, et 30 jours après la connexion au plus. **Se déconnecter l'efface** de la base et du navigateur. Entre le mot de passe et le code, une demi-session de cinq minutes, qui n'ouvre aucune page.
   - Les formulaires de compte venus d'un autre site sont refusés (`Origin`) ; `next` ne ramène qu'à une adresse de ce site, jamais d'ailleurs (`//ailleurs.example` est refusé). Les pages de compte ne sont jamais gardées en cache, jamais posées dans le cadre d'un autre site, sans aucun script ni ressource d'ailleurs (`Content-Security-Policy`). Une page faite pour un membre (elle porte son nom et ses valeurs) n'est jamais gardée en cache.
7. **Le serveur d'essai** (`node outils/server.mjs`) n'a pas de comptes : à la place des pages de compte (`501`) et d'une page réservée (`401`), il dit qu'il faut `holo serve`, avec la commande.

## Les bibliothèques, et pourquoi (en clair)

La cryptographie ne s'écrit jamais à la main : une petite erreur ne se voit pas, et ouvre la porte. Rust n'a ni hachage de mot de passe ni HMAC dans sa bibliothèque standard. Trois bibliothèques de **RustCrypto**, le groupe qui écrit la cryptographie de référence en Rust (libres, relues, employées par des milliers de projets), compilées **seulement pour le PC** (comme `tiny_http` et `rusqlite`), jamais dans le moteur de la page :

| Bibliothèque | Ce qu'elle fait ici | Pourquoi elle |
|---|---|---|
| `argon2` 0.6 | l'empreinte des mots de passe (Argon2id) | Argon2 a gagné le concours mondial des fonctions de hachage de mots de passe (2015) ; c'est le premier choix de l'OWASP. Lent exprès, et gourmand en mémoire : deviner un mot de passe à partir de la base coûte très cher. |
| `hmac` 0.13 et `sha1` 0.11 | le code à 6 chiffres (HMAC-SHA-1, RFC 4226 et 6238) ; l'empreinte des numéros de session | c'est ce que calculent toutes les applications d'authentification. La faiblesse connue de SHA-1 (deux textes qui donnent la même empreinte) ne joue pas pour un HMAC ni pour l'empreinte d'un numéro tiré au hasard. |

Elles amènent avec elles d'autres petites bibliothèques de RustCrypto (`blake2`, `password-hash`, `phc`, `digest`, `base64ct`, `cpufeatures`, `ctutils`, `cmov`, `hybrid-array`, `typenum`…). Le hasard vient du système (`getrandom`, déjà là). Le reste est écrit dans le moteur, parce que ce n'est pas de la cryptographie : l'écriture de la clé en base 32 (RFC 4648), les pages, les sessions, le frein.

## Comparaison faite avant de choisir

| Question | Django | Rails | Next.js | Choix, et pourquoi |
|---|---|---|---|---|
| Où sont les comptes | dans la base du projet (`django.contrib.auth`) | dans la base, avec un paquet (Devise) ou `has_secure_password` | ailleurs le plus souvent : un service (Auth0, Clerk, Firebase), ou Auth.js | **dans la base du site, chez l'auteur**, comme Django : aucun prestataire |
| Le mot de passe | PBKDF2 par défaut (Argon2 en option) | bcrypt | selon la bibliothèque choisie | **Argon2id d'office**, le premier choix de l'OWASP |
| Le code à 6 chiffres | un paquet à ajouter (`django-otp`, `django-two-factor-auth`) | un paquet (`devise-two-factor`) | selon le service | **dans le moteur**, sans rien ajouter |
| Les pages de compte | à écrire soi-même (`registration/login.html`) : « TemplateDoesNotExist » est l'erreur classique du débutant | générées une fois par Devise, puis à entretenir | des pages toutes faites avec Auth.js, sinon à écrire | **fabriquées par le moteur** : le débutant n'écrit rien, elles sont accessibles et essayées ; les restyler viendra après (dette) |
| Une page réservée | `@login_required` sur la vue (du code Python) | `before_action :authenticate_user!` (du code Ruby) | un « middleware » et une liste d'adresses, à tenir à part | **`access: members` dans la page elle-même** : ce qui protège la page est écrit sur la page |
| Le visiteur connecté | `request.user`, `{{ user.username }}` | `current_user` | `useSession()` côté page, autre chose côté serveur | **`signedIn` et `{account}`**, lus comme les autres valeurs, le même mot pour le serveur et pour la page |
| Se déconnecter | par un lien (GET) jusqu'à Django 4 : un autre site pouvait déconnecter quelqu'un ; un bouton (POST) depuis Django 5 | un bouton (DELETE) | selon la bibliothèque | **un bouton** (POST), jamais un lien |

La forme des pages de compte, comparée avant de choisir : (a) des pages écrites par l'auteur avec un bloc dédié (`SignIn()`, `Account()`), qu'il pose et restyle dans son site ; (b) **des pages fabriquées par le moteur**, à leurs adresses. Le choix (b) est le plus simple pour un débutant : il n'écrit rien, il ne peut pas oublier une étiquette ou un `autocomplete`, il ne peut pas garder un mot de passe dans `State` par mégarde (une valeur de la page part au serveur et dans la base, un mot de passe ne doit jamais y être), et les pages sont les mêmes partout, essayées une fois pour toutes. La forme (a) reste possible plus tard, en plus.

## Ce qui est refusé, et pourquoi

- `access:` autre que `members` ou `everyone` ; `access` ailleurs que sur `Page`.
- Changer `signedIn` ou `account` (une demande, un champ, un module, un glissement), les déclarer dans `State`, les garder par `keep` : c'est le serveur qui dit qui est connecté, jamais la page.
- Un nom de compte avec des accents ou des espaces : deux écritures d'un « é » (une lettre, ou un « e » suivi d'un accent) se ressemblent sans être le même nom ; les noms se comparent sans les majuscules (« Ada » et « ADA » sont le même).
- Un mot de passe de moins de 12 caractères ; deux mots de passe différents à la création.
- Se déconnecter par un lien (GET) : seulement par le bouton (POST).
- Un lien qui part de la racine du site, sauf les trois pages de compte.
- Un fichier ou un dossier du site nommé `account`, à sa racine : ses adresses sont celles des pages de compte, il n'est plus servi par `holo serve`. Le ranger sous un autre nom.

## Les défauts du web évités

- **Deux programmes qui vérifient chacun à sa façon** (la page et le serveur) : ici, le même arbitre rejoue les gestes sur le serveur ; la page n'envoie jamais de valeurs.
- **Le mot de passe gardé dans une valeur de la page**, envoyé avec elle, rangé dans la base : impossible ici, les pages de compte sont à part.
- **« Mauvais mot de passe » ou « nom inconnu »**, qui dit à un curieux quels noms existent : un seul message, et le même temps de réponse.
- **Un prestataire obligatoire** pour se connecter (Google, Firebase, un service d'e-mails ou de SMS) : aucun.
- **Une page réservée protégée ailleurs que là où elle est écrite** (une liste d'adresses dans un fichier à part, qu'on oublie de tenir à jour) : `access: members` est sur la page.
- **Une session gardée en clair dans la base** (Django garde la clé de session telle quelle) : ici, son empreinte seulement.

## Le lien avec le lot 6 (les valeurs partagées)

Le lot 6 (une autre session, en même temps) rend des valeurs partagées par tous les visiteurs. Ce lot n'en dépend pas. La suite naturelle, écrite comme une dette : **une valeur partagée que seuls les membres changent** (par exemple un livre d'or où seuls les membres écrivent, ou un « réserver » réservé aux membres) ; l'écriture serait à choisir avec Yocthan, par exemple une règle sous condition, `If(signedIn, is: 1, rules: [ … ])`, que le serveur vérifierait lui-même.

## Dettes

- **Les clés d'accès (passkeys, WebAuthn)** : la prochaine étape, pas faite. Il faut vérifier une signature (ECDSA P-256) : deux bibliothèques de RustCrypto de plus (`p256`, `sha2`), à accepter par Yocthan ; le navigateur demande JavaScript pour une clé d'accès (le mot de passe et son code restent pour qui n'en a pas) ; et une page sécurisée (https) ailleurs que sur le PC lui-même.
- **Un QR code** pour activer le code d'un coup d'appareil photo : pas fait (aucune bibliothèque ajoutée). Aujourd'hui, la clé se recopie, ou s'ouvre par le lien `otpauth://` sur le téléphone. À choisir : l'écrire dans le moteur (ce n'est pas de la cryptographie, environ 300 lignes), ou une bibliothèque (`qrcode`).
- **Un téléphone perdu** : pas de codes de secours. L'auteur du site, qui a la base, pourra retirer le code d'un compte (une commande `holo accounts` à écrire).
- Pas encore : changer son mot de passe ou son nom, effacer son compte et tout ce qu'il garde (RGPD), se déconnecter de tous ses appareils, fermer les inscriptions (aujourd'hui, tout visiteur peut créer un compte ; 10 000 comptes au plus), un frein par adresse IP (le frein est par nom et par compte), un frein sur la création de comptes.
- La clé du code est gardée telle quelle dans la base (le serveur doit pouvoir calculer le code), comme le fait `django-otp` : qui a la base peut calculer les codes. La base et ses sauvegardes doivent rester chez l'auteur.
- Un mot de passe avec des accents n'est pas mis sous une forme unique (é composé ou non) : il faut l'écrire de la même façon sur tous ses appareils.
- En ligne : le cookie de session n'a pas encore `Secure` (le serveur parle http sur le PC) ; à poser derrière un relais https (Caddy).
- Les pages de compte sont en français seulement, et ne se restylent pas encore avec le thème du site.
- Une page réservée protège sa page et son texte, pas les fichiers rangés à côté : un composant importé, une image, des données (`Data(from:)`) restent lisibles par tous. N'y mettre rien de secret.
- Avec JavaScript, ce que le compte garde suit les touchers et les champs (comme sans JavaScript) ; ce qui change par le temps, le clavier, un glissement ou l'arrivée d'un envoi change la page, pas ce que garde le compte. Deux appareils ouverts en même temps ne se voient pas changer (le direct, plus tard).

## Critères de validation

- Tests du moteur : `the_codes_follow_rfc_6238` (les six valeurs de la RFC 6238, annexe B, et les dix de la RFC 4226) ; `the_key_is_written_in_base_32` (RFC 4648) ; `a_password_is_never_kept_in_clear` ; `an_account_with_its_code_and_its_brake` (mot de passe trop court, nom mal écrit, deux mots de passe différents, nom déjà pris ; l'empreinte Argon2id ; le code activé avec la clé montrée ; se déconnecter ; le même message pour un mauvais mot de passe et un nom inconnu ; la demi-session qui n'ouvre rien ; un mauvais code, un code déjà servi, puis le suivant ; le frein du mot de passe, même pour un nom inconnu, et celui du code ; un formulaire venu d'un autre site ; la base jamais servie) ; `a_page_for_members_and_a_cart_that_follows_the_account` (la page réservée, son texte, ses gestes et ses messages refusés ; le panier d'avant la connexion qui suit le compte ; deux appareils ; un toucher renvoyé par le moteur, sans envoi de formulaire ; un nouveau serveur sur le même dossier) ; `a_stolen_session_expires` (14 jours sans visite, 30 jours au plus, l'empreinte seule gardée, un numéro inventé, se déconnecter) ; `the_page_knows_who_is_signed_in`, `the_visitor_values_are_read_never_changed`, `a_page_for_members` (le langage, et les liens vers les pages de compte).
- Dans Chrome (`node outils/browser-tests.mjs compte`) : « les comptes demandent holo serve » (le serveur d'essai) ; « un compte, son code à 6 chiffres, une page réservée, avec et sans JavaScript (serve) » (le code calculé par l'essai, comme l'application) ; « le panier suit le compte, sur deux appareils, avec et sans JavaScript (serve) ».
