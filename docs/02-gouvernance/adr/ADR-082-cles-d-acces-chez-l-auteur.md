# ADR-082 — Se connecter par une clé d'accès (WebAuthn), vérifiée chez l'auteur

- Statut : ACCEPTÉ
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue 181 et l'accord de Yocthan sur `p256` et `sha2` (commentaire de l'issue, 2026-10-08) ; le travail de Codex, PR 206 (`codex/fin-passkeys-2026-10-08`, comptes rendus `proposals/GPT5.6/fin-comptes-2026-10-08/PASSKEYS.md` et `proposals/GPT5.6/fin-passkeys-2026-10-08/README.md`), relu puis corrigé par Claude le 2026-10-09 (branche `reprise/codex-fin`) ; l'`ADR-081` (les comptes), l'`ADR-083` (le frein par adresse, l'effacement).
- Validation : Yocthan, le 2026-10-09 (« tu valides tout ce qu'on avait fait avec Codex »), y compris les bibliothèques `p256` et `sha2`.
- Projets affectés : le serveur (`holo serve`), HoloEngine

## Contexte

L'`ADR-081` a choisi d'abord le mot de passe et le code à 6 chiffres, et les clés d'accès (« passkeys ») ensuite. Une clé d'accès est une paire de clés que l'appareil du visiteur crée pour un site : la clé privée ne quitte jamais l'appareil, qui la débloque par l'empreinte, le visage ou le code de l'appareil ; le site garde la clé publique et vérifie une signature. Rien à retenir, rien à voler sur le serveur, et le navigateur refuse de signer pour un faux site.

## Décision

1. **Dans le compte, `/account/passkeys`** : un membre ajoute une clé (huit au plus), lui donne un nom, et la retire. Ajouter ou retirer demande de confirmer le mot de passe, et le code à 6 chiffres (ou un code de secours, `ADR-083`) s'il est activé : un cookie seul ne suffit pas.
2. **Se connecter par une clé**, depuis « Se connecter » : sans nom ni mot de passe. Le navigateur propose les clés qu'il connaît pour ce site (clés « découvrables ») ; le serveur vérifie la signature, puis que la clé appartient bien au compte qu'elle annonce, et ouvre la session ordinaire de l'`ADR-081` (le panier suit, les pages réservées s'ouvrent).
3. **Tout est vérifié chez l'auteur**, dans le serveur en Rust ; le navigateur ne fait que parler à sa clé (`navigator.credentials`), par un petit script servi seulement sur cette page (`/account/passkeys/script.js`). Le serveur vérifie :
   - un **défi** de 32 octets tirés au hasard, lié à l'opération (créer ou se connecter), à l'origine, à un cookie de défi (`holo_passkey`, `HttpOnly`, `SameSite=Strict`), et, pour créer, à la session ; cinq minutes, une seule fois, même quand la réponse est refusée ; dix mille défis en attente au plus ;
   - la **réponse du navigateur** (`clientDataJSON`), lue strictement : le type, le défi, l'origine exacte, aucun cadre étranger, aucune clé répétée ;
   - les **données de l'appareil** : l'empreinte du domaine, la présence et la vérification de la personne (empreinte, visage ou code), la cohérence des indicateurs de sauvegarde, et le compteur de signatures ;
   - pour créer : une clé ES256 (P-256), au format COSE attendu, sans attestation (`none` : le serveur n'apprend pas le modèle de l'appareil) ; pour se connecter : la signature, sur les données de l'appareil et l'empreinte SHA-256 de la réponse.
   Les autres algorithmes, formats et indicateurs sont refusés. La réponse pèse 16 Ko au plus.
4. **Où ça marche** : une clé d'accès demande un « contexte sûr » au navigateur.
   - Sur le PC de l'auteur : `http://localhost:<port>`. Sans configuration, `holo serve` n'accepte les clés que là, et seulement d'une demande venue de ce PC.
   - Sur un téléphone, par le Wi-Fi : **HTTPS est obligatoire** ; l'adresse `http://<adresse du PC>:8080` ne suffit pas (le navigateur n'offre pas les clés en HTTP, hors localhost). L'auteur choisit un nom de domaine, sert HTTPS par un proxy sur son PC (Caddy, nginx…), qui transmet au serveur Rust, et fixe **`HOLO_ORIGIN=https://son-domaine`** : seule cette origine est acceptée, et seulement d'une demande venue du proxy, sur ce PC. Aucun compte chez un hébergeur n'est obligatoire ; le certificat peut venir de l'auteur.
   - Derrière ce proxy, toutes les demandes arrivent de 127.0.0.1 : sans précaution, tout le site partagerait un seul frein de trente envois de compte par minute. Le frein prend alors la dernière adresse de `X-Forwarded-For`, celle qu'écrit le proxy (`ADR-083`) ; le proxy doit l'écrire (Caddy le fait d'office ; nginx : `proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;`).
5. **Sans JavaScript**, la clé d'accès n'existe pas (le navigateur ne la donne qu'à un script) ; le mot de passe, le code et les secours gardent leur parcours sans JavaScript, et la page le dit.
6. **Effacer son compte** (`ADR-083`) retire aussi ses clés et ses défis, dans la base et dans les sauvegardes locales du moteur.

## Les bibliothèques, et pourquoi (en clair)

La cryptographie ne s'écrit jamais à la main (`ADR-081`). Deux bibliothèques de RustCrypto, acceptées par Yocthan, compilées **seulement pour le serveur**, jamais dans le moteur de la page :

| Bibliothèque | Ce qu'elle fait ici | Pourquoi elle |
|---|---|---|
| `p256` 0.13.2 (`ecdsa`, `std`, sans ses fonctions par défaut) | vérifier une signature ECDSA sur la courbe P-256 (ES256), l'algorithme que toutes les clés d'accès savent faire | la vérification d'une signature sur une courbe elliptique est le cœur de la sécurité ; RustCrypto est la référence en Rust |
| `sha2` 0.10.9 (`std`) | l'empreinte SHA-256 du domaine, de la réponse du navigateur et du cookie de défi | SHA-256 est imposé par WebAuthn |

Elles amènent d'autres petites bibliothèques de RustCrypto (`ecdsa`, `elliptic-curve`, `primeorder`, `crypto-bigint`, `ff`, `group`, `sec1`, `der`, `spki`, `rfc6979`, `signature`, `digest` 0.10, `hmac` 0.12, `generic-array`, `subtle`, `zeroize`…), toutes dans `Cargo.lock` : la construction avec `--locked` passe. Deux générations de `digest` vivent donc côte à côte dans le serveur (0.10 pour celles-ci, 0.11 pour `hmac` 0.13 et `sha1` 0.11 de l'`ADR-081`).

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Qui vérifie | un service (Auth0, Clerk, Firebase, Hanko) ; une grande bibliothèque WebAuthn (`webauthn-rs`) ; **le serveur, avec deux primitives de RustCrypto et une lecture stricte écrite ici** | aucun prestataire ; `webauthn-rs` amène OpenSSL et beaucoup de code qu'on ne relit pas ; ici, le protocole est court quand on n'accepte qu'ES256, sans attestation, et chaque refus est éprouvé |
| Les algorithmes | ES256, RS256, EdDSA… ; **ES256 seul** | toutes les clés d'accès d'aujourd'hui le savent ; chaque algorithme de plus est du code à vérifier |
| L'attestation | demander le modèle de l'appareil ; **aucune (`none`)** | le site n'a pas à savoir quel téléphone a le visiteur |
| Le nom au moment de se connecter | le demander ; **ne rien demander (clé découvrable)** | rien ne dit à un curieux quels noms existent, et le visiteur n'a rien à taper |

## Ce qui est refusé, et pourquoi

- Une clé d'accès en HTTP ailleurs que sur localhost, une origine qui n'est pas celle du site, une demande venue d'ailleurs que du PC (ou de son proxy) : `403`.
- Un défi périmé, déjà servi, d'une autre opération ou d'une autre session ; une signature, une origine, un type, des indicateurs ou un compteur faux ; une clé d'un autre compte que celui qu'elle annonce : refusés, sans session.
- Une clé sans vérification de la personne (empreinte, visage ou code de l'appareil), ou d'un autre algorithme qu'ES256 ; une neuvième clé. La clé est demandée « découvrable » au navigateur, pour se connecter sans nom.
- Ajouter ou retirer une clé sans le mot de passe (et le code, s'il est activé).

## Sécurité, honnêtement

- Deux primitives cryptographiques ne suffisent pas à faire un WebAuthn sûr : la lecture stricte du protocole, les défis, les sessions et les refus sont le reste, et ils sont éprouvés par les tests (une vraie signature, chaque altération refusée). Une relecture extérieure du protocole reste souhaitable.
- La clé d'accès ne remplace pas encore le mot de passe : il faut un compte, donc un mot de passe, pour en ajouter une. Perdre son appareil laisse le mot de passe, le code et les secours.
- Le téléphone de Yocthan, sa biométrie, la synchronisation des clés entre appareils (Google, Apple) et le proxy HTTPS de l'auteur ne sont pas encore éprouvés en vrai : l'authentificateur virtuel de Chrome n'est pas un téléphone.

## Les défauts du web évités

- **Un prestataire pour se connecter** : aucun ; la clé publique reste dans la base de l'auteur.
- **Un mot de passe qu'on peut voler sur le serveur** : la clé d'accès n'en a pas ; la base ne garde que des clés publiques.
- **Le modèle de l'appareil demandé au visiteur** (l'attestation) : jamais.

## Dettes

- Un compte créé par une clé seule, sans mot de passe.
- Le parcours sur le téléphone de Yocthan, avec un proxy HTTPS et `HOLO_ORIGIN`, et un guide pas à pas pour l'auteur.
- D'autres algorithmes (EdDSA) si une clé réelle en a besoin.

## Critères de validation

- Tests du moteur : `a_real_es256_signature_and_each_rejection` (une vraie signature p256, puis chaque altération), `client_data_is_exact_and_json_has_no_ambiguous_keys`, `registration_checks_cose_identity_curve_and_flags`, `a_challenge_is_single_use_bound_and_expiring`.
- Dans Chrome (`node outils/browser-tests.mjs "clés"`), avec l'authentificateur virtuel du protocole de Chrome (CTAP2, clé résidente, vérification de la personne) : un compte, une clé ajoutée après confirmation, une connexion sans mot de passe, une signature, une origine et un compte altérés refusés, un défi refusé une seconde fois, le retrait refusé sans le bon mot de passe puis accepté, la clé retirée refusée ensuite ; et `http://127.0.0.1:<port>` refusé (`403`) quand `http://localhost:<port>` est admis.
