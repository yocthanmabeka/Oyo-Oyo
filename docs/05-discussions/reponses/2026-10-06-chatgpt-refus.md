# Réponse de ChatGPT — les refus de HoloCode (2026-10-06)

Réponse au prompt [`../prompts/2026-10-06-chatgpt-refus.md`](../prompts/2026-10-06-chatgpt-refus.md), collée par Yocthan le 2026-10-06. ChatGPT remplaçait Codex, dont le quota était épuisé. Il a, de lui-même, cherché sur Reddit, Stack Overflow et X. Résumé fidèle ; la synthèse des quatre avis est dans [`proposals/Claude/pourquoi-ces-refus-2026-10/SYNTHESE.md`](../../../proposals/Claude/pourquoi-ces-refus-2026-10/SYNTHESE.md).

## Son idée centrale

> « Le Web aime l'objectif de HoloCode (simplicité, sens, sécurité, accessibilité), mais les développeurs deviennent hostiles dès qu'une simplification les empêche d'exprimer un besoin réel. »

**La doctrine qu'il propose d'écrire** : ne jamais refuser une **capacité**, seulement la **mécanique historique** qui la donnait. `position: absolute`, non ; la superposition, oui. JavaScript libre, non ; une logique avancée, oui. Modifier la page à la main, non ; une interface qui change, évidemment oui. La cascade, non ; des états et des thèmes, oui.

> « HoloCode peut supprimer de la complexité accidentelle. Il ne doit jamais supprimer de la puissance utile. »

## Refus par refus

| Refus | Verdict de ChatGPT | Sa raison |
|---|---|---|
| `div` | Garder | À condition d'avoir assez de conteneurs neutres : `Row`, `Column`, `Grid`, `Stack`. Un `Row` peut n'être qu'une disposition. |
| `section`, `article` | Pas un refus définitif : inutiles en V1, à introduire seulement si un besoin HoloCode précis apparaît | Des questions sur leur usage reviennent sur Stack Overflow de 2016 à 2024, et sur Reddit encore en juin 2026 : copier HTML ici serait une erreur. |
| `Main`, `Nav`, `Header`, `Footer` | **Ajouter tout de suite**, en blocs | Directement compréhensibles et utiles à la navigation assistée. |
| `h4` à `h6` | **`H1` à `H6` tout de suite** | Le besoin est déjà connu ; ajouter trois blocs presque identiques coûte peu. `H4` veut dire « quatrième niveau du plan », jamais « plus petit ». |
| `script` | Pas de code libre ; **oui au calcul enfermé** | Il a trouvé sur Reddit lui-même des applications qui exécutent du code dans QuickJS compilé en WebAssembly, enfermé : sans réseau, sans cookies, sans stockage, arrêté s'il boucle. « Presque exactement notre modèle. » Il faut aussi des règles plus riches : `Else`, `ForEach`, fonctions pures, calculs, collections, requêtes et stockage contrôlés, modules WebAssembly avec permissions. |
| Modifier la page à la main | Garder, très fermement | Aucun courant moderne ne le réclame ; HoloCode va ici dans le sens du web moderne. |
| `position`, `z-index`, `float` | Garder la mécanique refusée, **donner la capacité** | Des blocs `Stack`, `Overlay`, `Anchor`, `Badge`. |
| Cascade, `!important` | Garder | Pour un non-programmeur, importer ce problème est « difficilement défendable ». Des états explicites : `.carte { hover: { … } focus: { … } }`. |
| `requestAnimationFrame` | Sans objet | L'auteur dit « tourne pendant deux secondes » ; le moteur choisit comment. |
| Texte en pixels | **Erreur de conception** | Il irait plus loin que le web : ne pas obliger un débutant à apprendre `rem`, `em`, `vw`, mais offrir des unités naturelles (par exemple `1text`, `50%`, `screen`, `auto`), converties par le moteur. |

## Sur la règle de Yocthan

ChatGPT n'est **pas d'accord** pour que la majorité des développeurs décide :

> « En 2026, des développeurs demandent encore sur les forums comment distinguer `section`, `article` et `div`. Si nous reproduisons leurs outils exactement, nous reproduirons leurs problèmes. La bonne question n'est pas “les développeurs veulent-ils `div` ?”, mais “quel travail essaient-ils de faire avec `div`, et HoloCode leur permet-il de le faire plus simplement ?” »

## Deux mises en garde utiles

- **WebAssembly n'est pas sûr par magie** : des chercheurs en sécurité le rappellent en 2026 ; c'est l'enfermement et les permissions qui protègent, pas l'extension `.wasm`. (Codex disait la même chose le 2026-10-03.)
- **X (Twitter) est mal indexé** : ChatGPT, comme Claude, refuse de prétendre avoir mesuré « l'avis de Twitter ».

## Sa conclusion

HoloCode ne doit pas être « HTML, CSS et JavaScript avec une syntaxe plus jolie », mais une **couche qui décrit ce que l'auteur veut construire**, pendant que le moteur décide comment le web le réalise. Plus HoloCode connaît le sens de ce qu'on crée (`Nav`, `Main`, `H1` à `H6`, `State`, `Row`), mieux le moteur pourra transformer le site plat en monde où l'on entre.

## Sources citées par ChatGPT

[Reddit r/webdev, sémantique](https://www.reddit.com/r/webdev/comments/sn79ea) · [Reddit, pourquoi tant de pages sans balises sémantiques](https://www.reddit.com/r/webdev/comments/wpujzv/why_do_so_many_pages_not_use_semantic_html_tags/) · [Reddit, juin 2026](https://www.reddit.com/r/webdev/comments/1ttja6p/having_a_hard_time_with_semantics_and_structure/) · [Stack Overflow, section et article](https://stackoverflow.com/questions/38715285/best-practices-on-usage-of-section-article-semantic-elements-in-html5) · [Stack Overflow, titres](https://stackoverflow.com/questions/45073757/semantic-html5-element-properties) · [Blog Stack Overflow, accessibilité](https://stackoverflow.blog/2024/08/09/how-we-re-making-stack-overflow-more-accessible/) · [Reddit, le moins de JavaScript possible](https://www.reddit.com/r/webdev/comments/vnme0g/a_community_of_people_who_dont_want_js_on_the_web/) · [Reddit for Developers, code enfermé (1)](https://developers.reddit.com/apps/rerollgame) · [Reddit for Developers, code enfermé (2)](https://developers.reddit.com/apps/echo-wiki) · [Blog Stack Overflow, composants encapsulés](https://stackoverflow.blog/2024/10/23/improve-developer-experience-ecommerce-fastlane-paypal-sponsored/) · [Reddit, spécificité CSS](https://www.reddit.com/r/webdev/comments/1fyyg9n) · [Stack Overflow, rem et px](https://stackoverflow.com/questions/19956490/rem-px-mediaqueries-for-browsers-ie9) · [Blog X, accessibilité](https://blog.x.com/en_us/topics/company/2020/making-twitter-more-accessible) · [X, Praetorian sur WebAssembly](https://x.com/praetorianlabs/status/2062543166905962957)
