# Réunir les lots du web avant de les déclarer terminés

- Sujet : terminer les lots restants, mandat de Yocthan du 2026-10-08, reprise explicite du lot 9.
- Discussions sources : issues #180 à #189, #202 ; contributions #177, #199, #201, #203 et chaîne du nuage jusqu'à #195. Aucun numéro HC inventé.
- Décisions concernées : ADR-074 à ADR-079, ADR-081, ADR-086 à ADR-096. Statut proposé : EXPÉRIMENTATION, aucun statut changé.

## Résultat

Branche propre basée sur #203. Le code des comptes #177, les protections du partage #199 et la pagination/parcours #201 sont repris ensemble, avec les graphiques, les polices locales, l'historique et les quatre dernières capacités du lot 9. Le parcours 10 est ajouté : graphiques en barres et en parts, chiffres accessibles, ajout par clavier avec et sans JavaScript.

Les conflits ont été relus dans les noms, les leçons, les tests, les arguments de Page et le serveur. Les deux réglages address et access sont conservés. Le serveur reçoit le membre et les valeurs de l'adresse. Le miroir du compte garde sa réponse JSON ; un geste sans JavaScript conserve sa redirection avec l'adresse à jour.

## Preuves

Les résultats distincts de #201 et #203 ne prouvent pas cette réunion. La suite entière est demandée sur cette branche : Rust debug/release, WASM complet/léger et Chrome. Le résultat sera ajouté après exécution. Le terminal du PC ne démarre pas ; ces exécutions utilisent GitHub Actions.

## Limites et suite

Les comptes de base sont construits, les dettes de #180 (secours, effacement, frein par IP, QR local), #181 (passkeys) et #182 (partage étendu) restent distinctes. Les passkeys demandent l'accord explicite de Yocthan pour p256 et sha2, selon l'issue #181. Le vrai Samsung, TalkBack, batterie et mémoire restent à essayer. Aucun pourcentage de finition globale n'est inventé.

Les statuts et les documents transversaux restent à Claude du PC. Aucune fusion ni fermeture d'issue.
