# Les majuscules et la casse : les réponses de ChatGPT et de Gemini (2026-10-06)

Réponses au prompt [`../prompts/2026-10-06-majuscules-et-casse.md`](../prompts/2026-10-06-majuscules-et-casse.md), collées par Yocthan le 2026-10-06. Résumé fidèle, puis la synthèse de Claude.

## ChatGPT

- **Respecter la casse : oui**, avec le choix (c) : une seule écriture officielle, une erreur claire, et **une correction en un clic dans l'éditeur**. « Langage strict, outil indulgent. » Le moteur ne corrige jamais en silence. HTML, SQL et Visual Basic ignorent la casse ; Python et Dart la respectent ; Nim l'ignore en partie, et sa documentation le reconnaît comme inhabituel. Pour des non-programmeurs : moins de variantes.
- **Garder presque tout** : blocs avec majuscule, réglages en minuscules, `Key.left`, `add`/`sub`/`set`/`random`, styles en minuscules et avec `-` comme en CSS, `KB` en majuscules (B = octet, b = bit).
- **Changer `_` pour le camelCase** : `topRight` au lieu de `top_right`, `appleX` au lieu de `apple_x`. Deux règles seulement : camelCase dans le langage, `-` dans les styles. « Même concept syntaxique, même convention. »
- **Les trois désaccords de Claude** : `Title:` accepté en silence est « un bug sérieux » ; `name: buy` doit être refusé avec la suggestion `Buy` (« les choses commencent par une majuscule ; les valeurs par une minuscule ») ; `KB` reste en majuscules.

## Gemini

- **Respecter la casse : oui**, choix (a) : strict, avec un message pédagogique. Ignorer la casse (HTML, SQL, Visual Basic) rend le code des débutants illisible ; Nim a été critiqué pour cela. « Pour un non-programmeur, l'ambiguïté est pire que l'interdiction. » Et oui à la correction d'un clic dans l'éditeur, le moteur restant strict.
- **Garder tout le tableau d'aujourd'hui**, ligne par ligne.
- **Garder `_`** (`apple_x`, `top_right`) et refuser le camelCase : **sur un téléphone, les claviers virtuels gèrent mal une majuscule au milieu d'un mot** ; le `_` est plus facile à taper pour un débutant. Le `-` reste réservé aux styles CSS.
- **Les trois désaccords de Claude** : d'accord sur les trois.

## Synthèse de Claude

| Question | ChatGPT | Gemini | Claude | Accord ? |
|---|---|---|---|---|
| Respecter la casse | Oui | Oui | Oui | **Les trois** |
| Refuser une faute avec le bon mot | Oui | Oui | Oui | **Les trois** |
| Corriger d'un clic dans l'éditeur | Oui | Oui | Oui | **Les trois** : **fait le 2026-10-06, à l'essai** (`ADR-046` : l'éditeur du navigateur et l'extension VS Code) |
| `Page(Title: …)` accepté en silence | Bug sérieux | Inacceptable | Défaut | **Les trois** : **corrigé le 2026-10-06** |
| `name: buy` accepté | Refuser, proposer `Buy` | Refuser | Refuser | **Les trois** : **corrigé le 2026-10-06** |
| `KB` en majuscules | Garder | Garder | Garder | **Les trois** |
| Blocs avec majuscule, réglages en minuscules, styles comme le CSS | Garder | Garder | Garder | **Les trois** |
| **Joindre deux mots** (`apple_x`, `top_right`) | **camelCase** : `appleX`, `topRight` | **`_`** : `apple_x`, `top_right` | Pas tranché | **Désaccord** : à Yocthan |

**Le seul désaccord : `_` ou camelCase.** Les deux arguments tiennent :

- **camelCase** (ChatGPT) : on retire un signe ; c'est l'usage de Flutter, de Dart et de JavaScript.
- **`_`** (Gemini) : plus facile à taper sur un téléphone, et c'est l'écriture d'aujourd'hui, donc rien à changer dans les fichiers.

L'avis de Claude, sans certitude : **garder `_`**. HoloCode doit pouvoir s'écrire sur un téléphone, Yocthan y teste tout, et les mots à joindre sont surtout des valeurs (`apple_x`) que le débutant tape lui-même. Mais c'est un choix de goût autant que de raison.

## Ce qui a été corrigé tout de suite

Les trois avis étant d'accord, Claude a corrigé les deux défauts le jour même :

- **Plus rien n'est avalé en silence.** Chaque bloc a la liste de ses paramètres ; un paramètre inconnu est refusé, et une faute de casse reçoit le bon mot : `Page(Title: …)` → « un paramètre s'écrit en minuscules, écris `title` ». Au passage, deux défauts connus disparaissent : `x` et `y` hors d'un plateau, et `align` hors d'un `Stack`, sont maintenant refusés avec la phrase qui dit où les mettre.
- **Un nom de bloc commence par une majuscule** : `Button(name: buy)` → « écris `name: Buy` ».
- La preuve : une ancienne proposition écrite en français, `Page(titre: …)`, passait jusqu'ici sans rien dire ; elle est maintenant refusée.
