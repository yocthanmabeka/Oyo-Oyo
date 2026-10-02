# ADR-007 — Le métavers est une mise à jour du web : une description, deux vues

- Statut : ACCEPTÉ
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013, HC-001
- Projets affectés : tous
- Validation : décidé par Yocthan le 2026-09-21. La fusion de la pull request qui introduit cette fiche vaut confirmation.

## Contexte

Les métavers précédents, celui de Meta en tête, ont proposé un jeu dans lequel il fallait entrer, souvent avec un casque. Or bien plus de gens vivent sur Internet que dans les jeux : on est sur le web toute la journée, on joue de temps en temps. La technique n'a pas fait échouer ces projets ; c'est la forme proposée.

## Décision

Le métavers est une mise à jour du web, pas un monde à part. « Quelqu'un verra un web normal, mais pourtant c'est le métavers. »

Un même fichier décrit un arbre de blocs, et cet arbre s'affiche de deux façons :

- **à plat** : une page web ordinaire ;
- **en profondeur** : un lieu où l'on zoome et où l'on entre.

Une page et un point sont la même chose : un point contient des points, comme un bloc contient des blocs.

## Alternatives étudiées

- Un deuxième web en 3D, à côté du vrai : VRML (1994), X3D, A-Frame. Aucun n'a pris.
- Une plateforme de type jeu, avec casque ou application dédiée : Horizon Worlds, Second Life, Decentraland.
- Une application seule, sans le web : perd le lien partageable et les milliards de gens déjà présents.

## Conséquences

### Positives

- On amène le métavers là où les gens sont déjà.
- Un monde reste un lien qu'on partage et que les moteurs de recherche peuvent lire.
- Les téléphones anciens affichent la vue à plat : personne n'est exclu.

### Négatives et risques

- Deux rendus à maintenir.
- Chaque bloc doit avoir un sens dans les deux vues.
- La vue en profondeur peut n'être qu'un gadget si elle n'apporte aucun usage réel.

## Critères de validation

- Un même fichier `.holo` s'affiche dans les deux vues sur un vrai téléphone.
- Une personne non prévenue utilise la vue à plat sans rien remarquer d'inhabituel.

## Conditions de réexamen

- S'il s'avère impossible de donner un sens aux blocs dans les deux vues sans doubler le travail de l'auteur.
