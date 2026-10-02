# Propositions de code

Ce dossier conserve les implémentations expérimentales séparément des spécifications officielles. Chaque auteur ou système contributeur possède son propre espace afin que l'origine des propositions reste explicite.

| Auteur | Proposition | Version | Statut | Revue |
|---|---|---|---|---|
| GPT5.6 | [Noyau exécutable HoloCode](GPT5.6/holocode-v0.1/) | 0.1.0 | Expérimentation | [PR #1](https://github.com/yocthanmabeka/Metaverse/pull/1) |
| Claude | [HoloCode : lois, capacités, vérificateur, journal causal](Claude/holocode-v0.1/) | 0.1 | Expérimentation | [PR #2](https://github.com/yocthanmabeka/Metaverse/pull/2) |
| Gemini | [HoloFractal : nœud fractal, zoom, budget mémoire](Gemini/) | 0.1 | Expérimentation | [PR #3](https://github.com/yocthanmabeka/Metaverse/pull/3) et [PR #7](https://github.com/yocthanmabeka/Metaverse/pull/7) |

Les tests de chaque proposition s'exécutent automatiquement sur GitHub à chaque changement (`.github/workflows/tests.yml`). La suite de conformité du projet, que tout moteur devra passer, est dans [`experiments/conformite-v0.1/`](../experiments/conformite-v0.1/).

Une proposition présente dans ce dossier n'est pas automatiquement une décision officielle de HoloCode. Son adoption doit être enregistrée dans `docs/02-gouvernance/DECISIONS.md`.
