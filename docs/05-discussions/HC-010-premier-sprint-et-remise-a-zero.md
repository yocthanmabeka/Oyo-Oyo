# HC-010 — Premier sprint et remise à zéro

## Métadonnées

- Date : 2026-09-19
- Statut : archivée
- Plateforme : Claude (Claude Code dans VS Code)
- URL : aucune — une session Claude Code est locale et n'a pas de lien de partage. Transcription : [transcriptions/HC-010.md](transcriptions/HC-010.md)
- Brief de départ : artefact Claude privé de Yocthan, `https://claude.ai/artifact/CNua2q73Q1cohHq8Rd4jN8`
- Parents : aucun dans ce dépôt
- Enfants : HC-011

## Problème étudié

Première prise de contact entre Claude et le projet. Yocthan transmet son brief pour que Claude s'en imprègne. Claude l'interprète comme un ordre de démarrage et construit seul un premier sprint.

## Conclusions

- Claude a produit sans discussion préalable une démonstration dans le navigateur : JavaScript natif, WebGL2 sans bibliothèque, une sphère qui se module et se morcelle selon les 20 faces d'un icosaèdre, un monde décrit par un fichier JSON de paramètres (graine, relief, couleurs). Poids mesuré : 24,7 Ko. Claude avait aussi commencé à nommer et à amorcer un langage.
- Yocthan a refusé cette façon de faire : rien n'avait été discuté, ni la stratégie, ni le nom du langage, ni son contenu. Il a demandé de tout effacer et de repartir de zéro.
- Claude a supprimé tous les fichiers qu'il avait créés. **Ce code n'existe plus** ; les liens vers `src/` dans la transcription sont morts.
- Règle de travail retenue avec Claude : on discute d'abord ; aucune création de fichier, aucune décision technique et aucun nom sans le feu vert explicite de Yocthan.

## Désaccords et limites

- Les choix techniques de ce sprint (navigateur, JavaScript, WebGL2, icosaèdre) n'ont jamais été validés. Ils ne constituent ni une décision ni une proposition en cours.
- La démonstration n'a pas été testée sur téléphone.

## Décisions produites ou affectées

- Aucune ADR. La règle « discuter avant de construire » rejoint l'autorité de validation décrite dans le [registre des décisions](../02-gouvernance/DECISIONS.md).

## Projets affectés

- Aucun.

## Documents mis à jour

- Aucun.

## Questions ouvertes

- D'où viennent la « limite de perception » de 1 Go et la sphère qui se morcelle ? Traité dans HC-011.
