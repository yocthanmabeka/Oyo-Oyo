# ADR-012 — Première version : des ponts vers JavaScript et CSS seulement

- Statut : ACCEPTÉ
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013
- Projets affectés : HoloCode, HoloCompiler
- Validation : décidé par Yocthan le 2026-09-21. La fusion de la pull request qui introduit cette fiche vaut confirmation.

## Contexte

Yocthan imaginait pouvoir importer n'importe quel langage dans HoloCode : HTML, CSS, JavaScript, C, Python. Sans accès à l'existant, un langage naît dans un désert : personne ne réécrit le paiement, les cartes ou les lecteurs vidéo.

## Décision

Dans la première version, les seuls ponts vers du code étranger sont **JavaScript et CSS**. Sur la cible web ils sont presque gratuits, puisqu'on compile déjà vers eux.

Ce sont des **outils de transition** :

- ils sont réservés au propriétaire de la page ; dans un monde partagé, ils sont interdits ou enfermés ;
- un fichier qui en utilise est marqué « web actuel seulement », car le navigateur propre au projet n'aura pas de moteur JavaScript ;
- le cœur du langage ne doit jamais en dépendre.

## Alternatives étudiées

- **Importer du code source C ou Python** : Python ne tourne pas sur un téléphone ; le C demande de faire correspondre les types et la mémoire. Les autres langages entreraient plutôt comme modules compilés (voir `ADR-013`).
- **Aucun pont** : langage pur, mais inutilisable pour un vrai site avant des années.

## Conséquences

### Positives

- Accès immédiat à tout l'écosystème du web.

### Négatives et risques

- Chaque pont perce un trou dans les garanties : le code importé peut boucler sans fin, ignore le budget mémoire, n'est pas déterministe et peut modifier ce qu'il veut.

## Critères de validation

- Une page `.holo` utilise une bibliothèque JavaScript existante sans que l'auteur écrive de JavaScript.

## Conditions de réexamen

- Quand les modules (`ADR-013`) couvrent les besoins, les ponts peuvent être retirés.
