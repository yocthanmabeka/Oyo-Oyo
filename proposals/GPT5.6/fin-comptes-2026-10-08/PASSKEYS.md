# Clés d'accès : construction proposée, préalable encore ouvert

Sujet : issue #181, après les dettes #180. Statut proposé : PROPOSITION, aucun statut de décision changé. Ce fichier prépare un résultat à relire ; aucun code de clés d'accès ni bibliothèque p256/sha2 ajouté.

## Dépendances proposées

p256 pour vérifier une signature ECDSA P-256/ES256, sha2 pour les empreintes SHA-256 du protocole. Ces bibliothèques RustCrypto restent dans le serveur natif ; elles n'alourdissent pas le WASM. Pas de paiement, prestataire, e-mail ou compte Google obligatoire. Leur poids et les dépendances transitives seront mesurés au build, pas estimés comme un résultat.

L'issue #181 impose : « attendre l'accord de Yocthan », écrit en commentaire de l'issue, avant ces ajouts. L'accord n'est pas encore présent à la lecture du 2026-10-08.

## Comportement à construire après accord

1. Depuis /account, un membre vérifié peut ajouter une clé d'accès. Le nom de la clé, sa clé publique et son identifiant restent dans la base de l'auteur. La clé privée reste dans l'authentificateur. Huit clés au plus par compte.
2. Le serveur tire un défi aléatoire de 32 octets, le lie à la session, au compte, à l'opération et à l'origine configurée ; il expire en cinq minutes et n'est utilisable qu'une fois. Une session n'a qu'un défi actif.
3. Enregistrement par navigateur : attestation none, ES256, présence et vérification de l'utilisateur exigées. Le serveur vérifie le défi, le type webauthn.create, l'origine exacte, le rpIdHash et la clé COSE attendue ; il rejette algorithme, courbe, longueur et format inconnus.
4. Connexion : défi différent, type webauthn.get, mêmes contrôles de contexte, drapeaux UP/UV, signature sur authenticatorData || SHA256(clientDataJSON), appartenance de la clé et compteur selon les authentificateurs synchronisés. Une clé de quelqu'un d'autre ou une réponse rejouée ne connecte personne.
5. Le serveur ouvre sa session ordinaire après vérification complète ; le panier et les pages réservées utilisent cette session. Les routes ne donnent jamais le contenu des comptes anonymement.
6. Révoquer une clé demande une connexion récente ; la suppression du compte retire ses clés et défis des mêmes sauvegardes que #180. Perdre la dernière clé garde une voie de récupération explicite par mot de passe et second facteur.

Les choix d'écriture ne modifient pas le langage : ces fonctions restent dans les pages de compte du moteur. Rien ne transforme une signature reçue en code exécutable.

## Preuves avant déclaration de réussite

Tests Rust d'un vrai vecteur ES256 accepté, mauvaise signature, mauvais rpIdHash, défi périmé/rejoué, origine différente, type erroné, drapeaux manquants et clé d'un autre compte. Chrome : authentificateur virtuel du protocole DevTools, création puis connexion et révocation avec une vraie signature de cet authentificateur. Aucun stub de « signature valide ».

Le Galaxy réel est une preuve différente. Une page distante doit être en HTTPS ; un simple HTTP sur l'IP Wi-Fi du PC ne suffit pas. Localhost permet le développement sur le PC. Le projet n'a pas encore un certificat HTTPS local installé sur le Samsung ; sa configuration doit être traitée et exécutée avant d'annoncer la parité téléphone.

## Risques et limites

Deux primitives cryptographiques ne suffisent pas à prouver une implantation WebAuthn sûre : la lecture du protocole, les limites, les sessions et la vérification complète restent nécessaires. La passkey ne promet ni liaison automatique à tous les appareils, ni déverrouillage sans action du visiteur. Un environnement sans cette API garde les comptes existants. La récupération ne doit pas rendre le second facteur inutile.

Sources : [WebAuthn du W3C](https://www.w3.org/TR/webauthn-3/), [authentificateur virtuel Chrome](https://chromedevtools.github.io/devtools-protocol/tot/WebAuthn/), [RustCrypto signatures](https://github.com/RustCrypto/signatures), [RustCrypto hashes](https://github.com/RustCrypto/hashes).
