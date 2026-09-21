# ADR-003 — Le monde, l'espace, les relations, les lois et les phénomènes comme briques du langage

- Statut : ACCEPTÉ
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-002, HC-003, HC-013
- Projets affectés : HoloCode
- Proposé par : ChatGPT. Reformulé par Claude, validé par Yocthan le 2026-09-21. La fusion de la pull request qui introduit cette fiche vaut confirmation. ChatGPT est invité à réagir à la reformulation.

## Contexte

La proposition d'origine considérait le monde, l'espace, les relations, les lois et les phénomènes comme « primitives candidates ». Les prototypes des PR n° 1 et n° 2 les ont mises à l'épreuve : elles reçoivent une sémantique déterministe et testable. `HC-013` a précisé ce qu'elles sont, et ce qu'elles ne sont pas.

## Décision

Ces cinq notions sont les briques de HoloCode pour décrire **ce qui arrive**. Ce qui existe est décrit par des blocs (`ADR-009`).

- **Monde** et **espace** : le cadre ; deux espaces ne se comparent jamais.
- **Relation** : un ensemble de paires d'entités, interrogeable, et non un booléen.
- **Loi** : une règle permanente qui ne change jamais l'état ; elle interdit.
- **Phénomène** : une règle qui, quand sa condition tient, formule une demande de capacité (`ADR-015`).

Un phénomène propose, une loi dispose.

Reformulation ajoutée : ces briques ne constituent pas un paradigme nouveau. La description retenue pour les documents est **des objets sans méthodes, des règles au niveau du monde, et des relations** ; chacune a un précédent (ECS, Datalog, moteurs de règles, règles « Instead » d'Inform 7). La valeur est dans ce que le langage interdit, et donc dans ce que le moteur peut vérifier.

## Alternatives étudiées

- Classes et méthodes, comme en programmation orientée objet : les règles se dispersent dans les objets et plus rien n'est vérifiable d'un seul regard.
- Un ECS utilisé comme bibliothèque : performant, mais sans garantie donnée par un langage.

## Conséquences

### Positives

- Toutes les règles d'un monde se lisent au même endroit.

### Négatives et risques

- Une loi ne sait aujourd'hui qu'interdire ; les lois continues (gravité, propagation) restent à concevoir.
- Un conflit entre deux phénomènes arrête le pas ; l'arbitrage reste à concevoir.

## Critères de validation

- Le scénario de [VISION.md](../../00-vision/VISION.md) s'exprime avec ces seules briques ; c'est le cas dans la PR n° 2.

## Conditions de réexamen

- Si la comparaison avec un ECS de référence ne montre aucun gain de clarté ni de vérification.
