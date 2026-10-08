# Politique de synchronisation GitHub

## Principe

GitHub est la source visible et versionnée du projet Metaverse. Une modification n'est considérée comme livrée que lorsqu'elle existe sur le dépôt `yocthanmabeka/Oyo-Oyo` (nommé `Metaverse` jusqu'au 2026-10-08 ; GitHub redirige l'ancienne adresse).

## Workflow obligatoire

1. Créer ou réutiliser une branche consacrée au travail en cours.
2. Enregistrer les modifications dans Git.
3. Publier la branche sur GitHub.
4. Créer ou actualiser une pull request.
5. Indiquer les tests exécutés et leurs résultats.
6. Fusionner dans `main` seulement après validation.

## Traçabilité des contributions IA

- Les propositions de code sont rangées sous `proposals/<auteur>/`.
- Leur auteur, version, statut et pull request sont indiqués dans `proposals/README.md`.
- Une IA ne présente jamais une modification uniquement locale comme publiée.
- Les décisions validées restent enregistrées séparément dans `docs/02-gouvernance/DECISIONS.md`.

## Règle pour les travaux futurs

Les futures modifications réalisées dans le cadre de ce projet doivent être synchronisées sur GitHub au cours du même travail lorsque l'accès au dépôt est disponible. Si la publication échoue, le blocage doit être signalé explicitement avec la liste des fichiers restant locaux.
