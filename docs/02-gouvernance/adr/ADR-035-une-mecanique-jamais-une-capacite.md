# ADR-035 — Refuser une mécanique, jamais une capacité

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : `proposals/Claude/pourquoi-ces-refus-2026-10/` (le pourquoi des refus, l'avis des humains, la synthèse) ; réponses de Gemini et de ChatGPT du 2026-10-06
- Validation : Yocthan, le 2026-10-06 : « Oui, vas-y, commence la construction de tout ce qu'on vient de décider. » Principe formulé par ChatGPT, partagé par Gemini et Claude. Validé par Yocthan le 2026-10-06 : « Qu'est-ce que tu attends pour valider tous ceux qui sont à l'essai ? »
- Projets affectés : HoloCode

## Contexte

HoloCode refuse des éléments du web (`div`, `script`, la cascade, `position`…). Quatre avis (les humains, Gemini, ChatGPT, Claude) ont jugé ces refus le 2026-10-06. Ils s'accordent sur une même ligne.

## Décision (à l'essai)

**HoloCode refuse une mécanique, jamais une capacité.** Il supprime la complexité accidentelle, jamais la puissance utile. Avant d'ajouter ou de refuser un élément, on se demande : *quel travail sert-il à faire, et comment HoloCode le permet-il plus simplement ?*

| Mécanique refusée | Capacité donnée par |
|---|---|
| `div` | `Row`, `Column`, `Grid`, `Stack` |
| `position`, `z-index` | `Stack` et `align:` (`ADR-036`) |
| la cascade, `!important` | un nom de style et des états (`ADR-036`) |
| modifier la page à la main | `State` et les règles |
| JavaScript libre | des règles plus riches, des fonctions pures, des modules enfermés avec permissions (`ADR-013`) |
| `requestAnimationFrame` | `Enter`, `Loop`, `Scenes`, `Every` |

Un refus qui laisse un besoin sans réponse est une dette : elle s'inscrit dans `docs/01-holocode/TABLEAU-WEB.md` jusqu'à ce qu'une capacité la paie.

## Conséquences

- Les refus de `div`, de `script`, de la page modifiée à la main, de la cascade et de `position` dans un style sont maintenus.
- Il reste des capacités à donner : le « sinon », les listes et la répétition, le calcul, puis les modules enfermés.

## Conditions de réexamen

- Quand Yocthan aura choisi la règle de décision : suivre la majorité des humains, ou la question du travail à faire (ChatGPT). Sur les huit refus examinés, les deux ont donné le même résultat.
