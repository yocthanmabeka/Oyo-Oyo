# Clés d’accès vérifiées chez l’auteur — 2026-10-08

Sujet : finir #181 après #180 (#205), sans fournisseur extérieur.
Décisions concernées : ADR-081, numéro réservé ADR-082, ADR-060.
Statut proposé : EXPÉRIMENTATION ; aucun statut de décision modifié.
Sources : issue #181 et accord explicite de Yocthan sur p256 et sha2, commentaire 6066367876.

## Résultat proposé

Le compte reçoit jusqu’à huit clés ES256. L’ajout et le retrait confirment le mot de passe et le second facteur actif, y compris un secours à usage unique. La connexion se fait ensuite par une clé découvrable, sans mot de passe. Les clés, défis et compteurs sont dans SQLite chez l’auteur. Effacer le compte les retire aussi des sauvegardes locales gérées par le moteur.

Un défi aléatoire de 32 octets est lié à l’opération, à l’origine et au cookie de demande ; l’ajout est aussi lié à la session connectée. Cinq minutes, une seule utilisation, y compris après une réponse refusée. Dix mille défis en attente au plus, huit clés par compte, corps de réponse 16 Ko. Pas de compte annoncé par nom : le navigateur fournit la clé découvrable et le serveur vérifie son propriétaire.

Vérifications : type, défi, origine exacte, absence de cadre étranger, empreinte du domaine, présence, vérification de l’utilisateur, cohérence des indicateurs de sauvegarde, clé COSE P-256/ES256, identifiant de clé, signature DER et compteur. L’attestation demandée est « none », sans identification du modèle d’appareil. Les données JSON et CBOR sont bornées et les clés répétées sont refusées.

## Adresse et installation

Sur ce PC, ouvrir **http://localhost:8080**, jamais l’adresse IP HTTP. Sur téléphone, l’auteur choisit un domaine et sert lui-même HTTPS ; `HOLO_ORIGIN=https://son-domaine` fixe l’origine et le domaine de la clé. Un proxy HTTPS local transmet le même Host au serveur Rust. Le serveur n’accepte pour WebAuthn que les demandes de son pair local : les en-têtes d’un visiteur ne configurent ni le domaine ni le protocole. Un certificat peut venir de l’auteur ; aucun compte chez Google, Apple ou un hébergeur n’est imposé. Le moteur Rust actuel ne termine pas TLS lui-même.

Le navigateur demande JavaScript pour WebAuthn. Le mot de passe, les codes et les secours gardent leur parcours sans JavaScript. La biométrie n’est jamais envoyée au site. Clés découvrables et vérification de l’utilisateur sont requises ; les anciennes clés sans ces fonctions ne sont pas admises.

## Preuves

Les tests sont écrits, **résultats CI en attente**. Le terminal du PC n’est pas utilisable dans cette session. Les tests Rust signent réellement par p256 et refusent des signatures, origines, indicateurs et compteurs altérés. Chrome utilise son authentificateur virtuel CTAP2, crée une vraie clé et signe : enregistrement, connexion, altérations, réemploi de défi et révocation. Aucun résultat n’est encore annoncé vert ici.

Le téléphone Samsung, sa biométrie, son dialogue de permission, la synchronisation entre appareils et le proxy HTTPS de l’auteur restent à éprouver en vrai.

## Fichiers et limites

Serveur natif : passkeys.rs, comptes, configuration d’origine, p256 0.13.2 et sha2 0.10.9. Navigateur : petit script chargé seulement sur la page des clés. Leçon 107, guide, noms, ce compte rendu et essais. Aucune dépendance ajoutée au WASM.

Les sauvegardes externes, la récupération d’un mot de passe perdu sans secours, la gestion de fournisseurs de clés et l’attestation matérielle ne sont pas construites ici. Un algorithme ES256 suffit à cette première livraison ; les autres algorithmes sont refusés explicitement.

Références : [WebAuthn, W3C](https://www.w3.org/TR/webauthn-3/), [RustCrypto P-256](https://github.com/RustCrypto/elliptic-curves/tree/master/p256), [SHA-2](https://github.com/RustCrypto/hashes/tree/master/sha2), [authentificateurs virtuels de Chrome](https://chromedevtools.github.io/devtools-protocol/tot/WebAuthn/).

## Remise

Branche Codex empilée après #205 ; PR en brouillon. Claude du PC relit et fusionne après les preuves. Le journal, AGENTS et les statuts des ADR/DECISIONS restent à Claude et Yocthan.
