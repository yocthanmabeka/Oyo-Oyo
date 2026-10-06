# Les refus de HoloCode : quatre avis côte à côte, et ce qui reste à décider

- Auteur : Claude, le 2026-10-06. Pour Yocthan, qui décide.
- Les avis : [Claude](README.md), [ce que disent les humains](AVIS-DES-HUMAINS.md), [Gemini](../../../docs/05-discussions/reponses/2026-10-06-gemini-refus.md), [ChatGPT](../../../docs/05-discussions/reponses/2026-10-06-chatgpt-refus.md) (à la place de Codex, dont le quota était épuisé). Codex pourra encore répondre au prompt qui l'attend.
- Statut : **PROPOSITION**. Aucune décision n'est prise.

## Les quatre avis

| Élément | Les humains (règle de Yocthan) | Gemini | ChatGPT | Claude | **Accord ?** |
|---|---|---|---|---|---|
| `div` | Garder le refus | Garder | Garder | Garder | **Les quatre** |
| `section`, `article` | Garder le refus | Assouplir | Pas en V1 ; plus tard si un besoin apparaît | Garder | Trois sur quatre : pas maintenant |
| `Main`, `Nav`, `Header`, `Footer` | Lever le refus | Ajouter en blocs | Ajouter tout de suite, en blocs | Ajouter en blocs | **Les quatre** |
| `H4` à `H6` | Lever le refus | Jusqu'à `H6` | `H1` à `H6` tout de suite | `H4` au premier besoin | Trois sur quatre : **jusqu'à `H6` maintenant**. Claude se range à cet avis. |
| `script` | Assouplir | Garder | Pas de code libre ; calcul enfermé | Pas de code libre ; code enfermé | **Les quatre** : jamais de code libre, mais du calcul enfermé et des règles plus riches |
| Modifier la page à la main | Garder le refus | Garder | Garder, très fermement | Garder | **Les quatre** |
| `position` pour la mise en page | Garder le refus | Garder dans les styles | Garder la mécanique | Garder | **Les quatre** |
| Superposition (badge, bulle, barre fixe) | Lever le refus | Bloc `Stack` ou `Badge` | Blocs `Stack`, `Overlay`, `Anchor`, `Badge` | Un bloc de superposition | **Les quatre** |
| Cascade, `!important` | Garder le refus | Garder | Garder | Garder | **Les quatre** |
| Survol, focus, appui | (manque) | États dans le style | États dans le style | États dans le style | **Les quatre** |
| Texte en pixels | À corriger | Unités relatives | Erreur de conception ; des unités naturelles | Unités relatives | **Les quatre** |

## Le principe que les quatre avis dessinent

ChatGPT l'a formulé le plus clairement :

> **HoloCode refuse une mécanique, jamais une capacité.** Il supprime la complexité accidentelle, jamais la puissance utile.

| Le web offre une mécanique | HoloCode refuse la mécanique | Et donne la capacité par |
|---|---|---|
| `div` | oui | `Row`, `Column`, `Grid`, et un bloc de superposition |
| `position`, `z-index` | oui | des blocs de superposition et d'ancrage |
| la cascade, `!important` | oui | un nom de style et des états (`hover`, `focus`) |
| modifier la page à la main | oui | `State` et les règles |
| JavaScript libre | oui | des règles plus riches, des fonctions pures, des modules enfermés avec permissions |
| `requestAnimationFrame` | oui | `Enter`, `Loop`, `Scenes`, `Every` |

Ce principe pourrait devenir une fiche de décision, si Yocthan le veut.

## Un désaccord sur la méthode

La règle de Yocthan dit de suivre la majorité des humains. **ChatGPT n'est pas d'accord** : les gens qui parlent sur les forums sont surtout des développeurs, et ils demandent encore en 2026 comment distinguer `section`, `article` et `div`. Copier leurs outils, c'est copier leurs problèmes. Il propose une autre question : *quel travail l'élément sert-il à faire, et HoloCode permet-il de le faire plus simplement ?*

Sur ces huit refus, les deux méthodes donnent **le même résultat**. Elles pourraient diverger plus tard. À Yocthan de choisir laquelle sert de règle.

## Ce que Claude propose de construire, si Yocthan dit oui

Dans cet ordre, chaque morceau avec sa comparaison, sa leçon et sa fiche, comme d'habitude :

1. **Les repères** : `Main`, `Nav`, `Header`, `Footer`.
2. **Les titres jusqu'à `H6`**, avec la même règle de plan.
3. **Des tailles de texte qui suivent le réglage du visiteur** (unités relatives, et peut-être des unités naturelles comme le propose ChatGPT).
4. **Les états d'un bloc dans le style** : `hover`, `focus`, `active`.
5. **La superposition** : un badge ou une bulle posés sur un bloc, une barre qui reste en place.
6. Ensuite, le plus gros : **le « sinon », les listes et la répétition, le calcul** (des frais de port), puis le code enfermé.

## Ce que Yocthan doit trancher

1. Adopter le principe « une mécanique refusée, jamais une capacité » ?
2. La règle de décision : la majorité des humains, ou la question du travail à faire ?
3. Les mots : `Main`, `Nav`, `Header`, `Footer` ; `Stack`, `Overlay`, `Anchor` ou `Badge` pour la superposition ; et l'écriture des unités relatives. (Codex a souvent demandé de vérifier les noms contre `ADR-016` : à faire avant de construire.)
4. Lancer la construction des points 1 à 5 ?
