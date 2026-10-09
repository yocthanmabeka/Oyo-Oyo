# Finir les protections des comptes

- Sujet : issue #180, mandat de Yocthan pour finir les lots restants. Discussions : reprise de la session PC, contributions #177, #199, #201, #204. Aucun numéro HC inventé.
- Décisions concernées : ADR-081 et proposition réservée ADR-083. Statut proposé : EXPÉRIMENTATION ; aucun statut modifié.
- Base empilée : #204. Entrée sur main après #177, uniquement par Claude du PC.

## Résultat proposé

Dix codes de secours aléatoires de 128 bits, montrés une seule fois à l'activation du TOTP. La base garde des empreintes HMAC avec séparation de domaine. Le mot de passe reste requis. Un code utilisé est retiré sous le verrou de la base ; il ne donne pas deux connexions.

Un QR SVG local donne la même URI otpauth que le lien et la clé écrite. qrcode 0.14.1 est ajouté uniquement au serveur natif, sans ses bibliothèques d'image ni feature optionnelle. Ni dépendance WASM ni prestataire. Ajouter une bibliothèque pour le QR est préférable à inventer l'encodage et la correction d'erreur ; le décodage de test est indépendant.

Le frein par IP réelle du pair limite tous les POST de compte à trente par minute, indépendamment du nom annoncé. Dix mille IP au plus dans la fenêtre ; au-delà, refus. Les en-têtes X-Forwarded-For ne changent pas cette identité. Le frein par compte reste présent.

L'effacement est un POST confirmé par le nom exact et le mot de passe, plus un code TOTP neuf ou un secours quand le second facteur est actif. Il retire les sessions, les secours, les visites, les messages associés et les fichiers privés. Les sauvegardes SQLite locales du moteur sont nettoyées. Un verrou précède l'identification et le geste de page : une demande en vol ne peut pas réécrire le panier effacé. Les numéros effacés ne sont pas réutilisés.

## Preuves exécutées

Le terminal du PC ne démarre pas. Aucune exécution locale annoncée. Tests Rust de secours/réemploi, fenêtre IP et effacement avec un autre compte, une sauvegarde et un fichier ; parcours Chrome au clavier, avec et sans JS, QR relu par jsQR, réemploi refusé et base relue. Résultats GitHub Actions à ajouter après leur exécution.

## Objections et limites

Une suppression SQLite est logique ; les copies sorties du dossier, les journaux externes et la récupération physique des anciens octets du disque ne sont pas couverts. Les messages anciens sans identifiant de compte ne peuvent pas être attribués rétroactivement. Les données publiques partagées appartiennent à la page. Une sauvegarde illisible bloque la fin de l'effacement et garde le compte actif ; une copie déjà nettoyée ne lui redonne pas ses données. Un fichier qui ne s'efface pas est mis en file de reprise et signalé.

Les pages QR et secours portent les en-têtes privés du compte. Le QR demande l'essai d'une application réelle sur le Samsung ; le décodeur indépendant de CI ne prouve pas le matériel. Pas de mesure de batterie.

Les passkeys (#181) restent séparées : cette issue exige l'accord écrit de Yocthan avant p256 et sha2. Le QR n'ajoute aucun de ces deux modules cryptographiques.

## Documents et sources

Leçon 108, guide et noms modifiés. Journal, DECISIONS et ADR restent à Claude du PC et Yocthan.
[qrcode](https://github.com/kennytm/qrcode-rust), [index exact 0.14.1](https://github.com/rust-lang/crates.io-index/blob/master/qr/co/qrcode), [OWASP sur les codes de récupération](https://cheatsheetseries.owasp.org/cheatsheets/Multifactor_Authentication_Cheat_Sheet.html).
