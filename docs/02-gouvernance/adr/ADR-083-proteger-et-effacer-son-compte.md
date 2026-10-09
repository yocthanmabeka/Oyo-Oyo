# ADR-083 — Protéger et effacer son compte : un QR local, dix codes de secours, l'effacement confirmé, un frein par adresse

- Statut : ACCEPTÉ
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue 180 (les dettes de l'`ADR-081`) ; le travail de Codex, PR 205 (`codex/fin-comptes-2026-10-08`, compte rendu `proposals/GPT5.6/fin-comptes-2026-10-08/README.md`), relu puis corrigé par Claude le 2026-10-09 (branche `reprise/codex-fin`) ; l'`ADR-081` (les comptes chez l'auteur) et l'`ADR-076` (les sauvegardes).
- Validation : Yocthan, le 2026-10-09 (« tu valides tout ce qu'on avait fait avec Codex »), y compris la bibliothèque `qrcode`.
- Projets affectés : le serveur (`holo serve`), HoloEngine

## Contexte

L'`ADR-081` a donné des comptes à `holo serve` : un mot de passe, puis un code à 6 chiffres. Elle laissait quatre dettes : un QR pour activer le code (il fallait recopier une clé de 32 lettres), une voie de secours quand le téléphone est perdu, un compte que son membre efface lui-même, et un frein qui ne se contourne pas en changeant de nom à chaque essai.

## Décision

1. **Un QR, fabriqué sur le PC de l'auteur.** La page d'activation (`/account/code/setup`) montre un QR en SVG (`id="setup-qr"`, `role="img"`, une étiquette pour le lecteur d'écran), qui porte la même adresse `otpauth://` que le lien et la clé écrite. Il est fabriqué par le serveur, avec la bibliothèque `qrcode` 0.14.1, sans ses dépendances d'image : la clé n'est envoyée à aucun service. La clé écrite reste là pour qui ne peut pas scanner.
2. **Dix codes de secours, montrés une seule fois.** Quand le code à 6 chiffres s'active, la page montre dix codes de secours (128 bits chacun, tirés au hasard par le système, écrits en quatre groupes de huit), avec la consigne de les garder ailleurs que sur le téléphone ; puis elle mène au compte (`/account?done=code`), qui annonce le code activé. Un rechargement ne les montre plus. La base n'en garde que l'empreinte (HMAC-SHA-1, avec la clé du compte et une séparation de domaine, `holocode-recovery-v1:`). Chacun sert **une seule fois, après le mot de passe**, dans le même champ que le code du moment ; il est retiré sous le verrou de la base : deux connexions ne partent pas du même code. Retirer le code à 6 chiffres retire aussi les secours.
3. **Effacer son compte.** `/account/delete` montre une confirmation ; un `GET` n'efface jamais. L'envoi (`POST`) demande le nom exact, le mot de passe et, si le code est activé, un code du moment ou un secours inutilisé ; cinq essais ratés, puis une attente, comme pour se connecter. Le serveur retire alors, **dans la base et dans les sauvegardes locales du moteur** (`holo-data/backups/site-*.sqlite`) : le compte, ses sessions, ses clés d'accès et leurs défis (`ADR-082`), ses secours, ses visites (son panier), les messages envoyés depuis son compte et les fichiers privés qu'ils nomment. Il vérifie d'abord chaque sauvegarde : une sauvegarde illisible arrête tout, et le compte reste. Le numéro d'un compte effacé n'est jamais redonné. L'effacement prend le verrou des gestes avant celui de la base : un geste en vol ne réécrit pas le panier effacé.
4. **Un fichier qui ne s'efface pas tout de suite** (pris par un autre programme, sous Windows) reste en attente, et le serveur réessaie à chaque démarrage, fichier par fichier. **Un échec n'empêche jamais `holo serve` de démarrer** : il le dit, et continue (correction de la relecture du 2026-10-09).
5. **Un frein par adresse**, en plus du frein par nom et par compte de l'`ADR-081` : trente envois de compte par minute au plus pour une même adresse (créer un compte, se connecter, le code, l'activer ou le retirer, se déconnecter, effacer, les clés d'accès), même sous des noms différents ; au-delà, `429` et `Retry-After: 60`, et le message le dit. Dix mille adresses suivies au plus par minute ; au-delà, refus. L'adresse est celle du visiteur réel :
   - le pair de la connexion TCP, et jamais un en-tête écrit par le visiteur ;
   - derrière le proxy HTTPS de l'auteur (`HOLO_ORIGIN` fixé, et la demande vient de ce PC même, donc du proxy), la dernière adresse de `X-Forwarded-For`, celle que ce proxy a écrite. Sans cela, tous les visiteurs arriveraient de 127.0.0.1 et partageraient un seul frein de trente envois par minute : un seul curieux bloquerait les comptes de tout le site.

## Les bibliothèques, et pourquoi (en clair)

| Bibliothèque | Ce qu'elle fait ici | Pourquoi elle |
|---|---|---|
| `qrcode` 0.14.1, sans ses fonctions par défaut | écrire l'adresse `otpauth://` en QR (l'encodage, la correction d'erreurs), dessiné ici en SVG | un QR mal fait ne se lit pas ; son encodage et sa correction d'erreurs (Reed-Solomon) ne s'écrivent pas à la main. Sans ses fonctions par défaut, elle n'amène aucune bibliothèque d'image. Seulement dans le serveur, jamais dans le moteur de la page. |

Les essais relisent le QR avec un décodeur indépendant, jsQR 1.4.0 (licence Apache-2.0), gardé dans le dépôt avec sa licence (`moteur/outils/vendor/jsqr-1.4.0/`) : les essais ne vont plus chercher de code sur Internet.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le secours quand le téléphone est perdu | un e-mail ou un SMS ; une question secrète ; l'auteur qui retire le code à la main ; **des codes à usage unique, montrés une fois** | aucun prestataire (règle « chez soi d'abord ») ; une question secrète se devine ; les codes à usage unique sont ce que conseille l'OWASP et ce que font GitHub, Google ou Django (`django-otp`, ses « static tokens ») |
| Garder les secours | en clair ; **leur empreinte** | une copie de la base (une sauvegarde) ne doit pas suffire pour entrer |
| Effacer | marquer le compte « effacé » ; **tout retirer, sauvegardes locales comprises** | ce qui est effacé ne doit pas revenir par une sauvegarde ; un compte marqué garde ses données |
| Le frein par adresse | aucun ; l'adresse du pair seule ; **le pair, ou l'adresse que le proxy de l'auteur écrit quand il y en a un** | derrière un proxy, l'adresse du pair est celle du proxy, la même pour tous ; l'en-tête n'est cru que là où personne d'autre ne peut l'écrire |

## Ce qui est refusé, et pourquoi

- Effacer par un lien (`GET`) : seulement par le bouton (`POST`), après confirmation.
- Effacer sans le mot de passe, ou sans le code quand il est activé : un ordinateur resté ouvert ne suffit pas.
- Un code de secours réutilisé, ou donné sans le mot de passe.
- Un en-tête `X-Forwarded-For` venu d'un visiteur qui parle directement au serveur (sur le Wi-Fi), ou quand l'auteur n'a pas déclaré de proxy : ignoré.

## Sécurité, honnêtement

- Un effacement dans SQLite est logique : les copies sorties du dossier (une clé USB, un nuage), les journaux d'un autre programme et les anciens octets du disque ne sont pas couverts. Les messages anciens, envoyés avant qu'ils portent le numéro du compte, ne peuvent pas lui être attribués. Le contenu public partagé (`ADR-079`, `ADR-080`) appartient à la page, pas au compte.
- Le frein par adresse est un frein contre un curieux, pas contre une attaque répartie sur des milliers d'adresses. Un réseau partagé (une école, une box) peut l'atteindre. Derrière un proxy, il suppose que le proxy écrit `X-Forwarded-For` (Caddy le fait d'office ; nginx avec `proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;`) : un proxy qui laisse passer l'en-tête du visiteur tel quel le laisserait choisir son adresse, et seuls les freins par nom et par compte resteraient.
- Le QR demande encore un essai avec une vraie application sur le téléphone de Yocthan : le décodeur des essais n'est pas un téléphone.

## Dettes

- Changer son mot de passe ; retrouver un compte quand le mot de passe et tous les secours sont perdus (aujourd'hui, l'auteur seul, à la main).
- Générer de nouveaux codes de secours sans retirer puis remettre le code à 6 chiffres.
- L'essai du QR et de l'effacement sur le téléphone de Yocthan.

## Critères de validation

- Tests du moteur (`cargo test --release --locked`) : `recoveries_are_shown_once_hashed_and_single_use_after_password`, `ip_limits_ignore_changed_account_names_and_expire`, `behind_the_authors_proxy_each_visitor_has_its_own_brake`, `deleting_requires_confirmation_and_cleans_live_data_backups_and_files` (un autre compte, une sauvegarde, un fichier privé), `a_private_file_that_cannot_be_erased_does_not_stop_the_server`, et dans le serveur `the_real_client_is_read_only_behind_the_authors_proxy`.
- Dans Chrome (`node outils/browser-tests.mjs comptes`) : le QR du serveur relu par jsQR, les dix secours montrés une fois, le premier accepté puis refusé, avec et sans JavaScript ; l'effacement confirmé au clavier, la base et sa sauvegarde relues ; trente et un vrais formulaires sous trente et un noms, le trente et unième freiné (`429`, `Retry-After: 60`) ; et l'essai du lot 7, qui lit « Le code à 6 chiffres est activé ».
