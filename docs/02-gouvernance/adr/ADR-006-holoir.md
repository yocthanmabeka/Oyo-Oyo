# ADR-006 — Préserver l'information spatiale et temporelle dans HoloIR

- Statut : PROPOSITION
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-007, HC-013
- Projets affectés : HoloCompiler, HoloIR
- Proposé par : ChatGPT. Laissé en `PROPOSITION` par Yocthan le 2026-09-21, sur la recommandation de Claude.

## Contexte

HoloIR serait le format intermédiaire entre le fichier source et l'exécution. Les compilateurs classiques aplatissent tout en simples nombres : ils oublient que `2m` était une longueur et `for 3s` une durée. L'idée est de garder ces informations pour que le moteur s'en serve : indexer l'espace, planifier le temps.

## Décision proposée

Conserver dans HoloIR les unités, les espaces, les relations et les durées, au lieu de les aplatir.

## Pourquoi elle reste en proposition

- Aucun HoloIR n'existe. Avec `ADR-008` et `ADR-010`, le moteur lit directement le fichier `.holo`.
- La version binaire compacte du `.holo`, envisagée pour réduire le poids des mondes, jouerait ce rôle : c'est à ce moment-là que la question se posera concrètement.
- La décision n'est pas nécessaire au sprint Big Bang.

L'idée est bonne ; elle est prématurée.

## Critères de validation

- Une mesure montrant qu'un moteur qui garde ces informations fait mieux qu'un moteur qui les aplatit : relations spatiales moins coûteuses, erreurs mieux expliquées.

## Conditions de réexamen

- Quand le moteur Rust existe et qu'un format binaire du `.holo` devient nécessaire.
