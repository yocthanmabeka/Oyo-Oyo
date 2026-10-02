# ADR-005 — La première version tourne sur le matériel existant : tout téléphone actuel, dans un navigateur

- Statut : ACCEPTÉ
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-005, HC-007, HC-013
- Projets affectés : tous les prototypes
- Proposé par : ChatGPT. Complété par Claude avec les chiffres de la vision de Yocthan, validé par Yocthan le 2026-09-21. La fusion de la pull request qui introduit cette fiche vaut confirmation. ChatGPT est invité à réagir aux ajouts.

## Contexte

La proposition d'origine disait seulement de « faire fonctionner la première version sur le matériel existant ». Telle quelle, elle n'était pas mesurable. La vision de Yocthan donne les chiffres.

## Décision

On n'attend ni processeur spatial, ni interface holographique. La cible de la première version est :

- **n'importe quel téléphone actuel** ; les téléphones anciens affichent la vue à plat (`ADR-007`) ;
- **dans un navigateur**, sans rien installer (`ADR-010`) ;
- **un premier test qui tient dans 1 Go au maximum**. Dans un onglet de téléphone, le budget réel est plus bas : l'onglet est souvent tué entre 300 et 500 Mo. Ce chiffre est à vérifier par la mesure. Le plafond de 1 Go est la cible fixée par Yocthan, pas un résultat : c'est sa faisabilité que les sprints mesurent.

Le matériel spécialisé reste un axe de recherche à long terme, jamais une capacité supposée acquise.

## Alternatives étudiées

- Exiger un casque ou un ordinateur puissant : c'est ce qui a limité les métavers précédents.
- Une application à installer dès la première version : on perd le lien partageable ; elle viendra avec le navigateur propre au projet.

## Conséquences

### Positives

- Chaque sprint se vérifie sur un vrai téléphone, avec un chronomètre et un compteur de mémoire.

### Négatives et risques

- La chauffe et la batterie d'un téléphone limitent le rendu 3D continu (signalé par Gemini).
- WebGPU est absent de certains téléphones ; il faut un mode de secours en WebGL.

## Critères de validation

- Le sprint Big Bang tourne sur le téléphone de Yocthan ; on mesure le poids du moteur, la fluidité, la mémoire et la batterie.

## Conditions de réexamen

- Si la mesure montre qu'un onglet de téléphone ne tient pas un monde minimal.
