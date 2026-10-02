# ADR-004 — Des archétypes composés et des capacités, plutôt que l'héritage de classes

- Statut : ACCEPTÉ
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-003, HC-004, HC-013
- Projets affectés : HoloCode, HoloIR
- Proposé par : ChatGPT. Validé tel quel par Yocthan le 2026-09-21, sur la recommandation de Claude. La fusion de la pull request qui introduit cette fiche vaut confirmation.

## Contexte

L'héritage de classes produit des hiérarchies rigides (« Porte hérite d'Objet, qui hérite de Chose »). L'industrie du jeu l'a abandonné au profit de la composition. La PR n° 2 a mis la proposition à l'épreuve.

## Décision

- Une entité est **composée** d'archétypes, à plat : `FrontDoor: Openable + Lockable`. Pas d'héritage, pas de priorité.
- Deux archétypes qui déclarent le même champ ou la même capacité ne se composent pas : c'est une erreur, pas une surcharge.
- Un état ne change que par une **capacité**, et une capacité n'écrit que dans les champs de son propre archétype.

Précision de vocabulaire, signalée par Gemini : une capacité est ici une demande arbitrée par le moteur (`ADR-015`). Ce n'est pas une capacité au sens du modèle object-capability, qui désigne un jeton de droit infalsifiable et transférable.

## Alternatives étudiées

- L'héritage de classes : écarté, voir le contexte.
- Un ECS pur, où les composants sont des données passives et où tout le comportement vit dans des systèmes : plus performant, mais l'auteur ne voit plus ce qu'une entité sait faire.

## Conséquences

### Positives

- Les collisions de composition sont détectées avant l'exécution.
- Cohérent avec `ADR-015`, qui interdit l'héritage et le code libre dans un bloc.

### Négatives et risques

- Les capacités n'acceptent pour l'instant que des constantes ; il faudra des paramètres pour écrire un jeu.

## Critères de validation

- Le vérificateur refuse une composition en collision et une capacité qui écrit hors de son archétype ; c'est le cas dans la PR n° 2.

## Conditions de réexamen

- Si la composition à plat s'avère trop pauvre pour décrire des entités réelles.
