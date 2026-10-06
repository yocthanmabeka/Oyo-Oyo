# ADR-038 — Lot 1 : la langue et le partage, la vidéo, le tableau, le texte long, le choix

- Statut : EXPÉRIMENTATION
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : le grand tableau (`docs/01-holocode/TABLEAU-WEB.md`) ; Yocthan, le 2026-10-06 : « tous ceux qu'on a décidé de mettre… que tout passe au vert »
- Validation : à donner par Yocthan après essai.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Le tableau compte 48 éléments du web à ajouter (14 en priorité, 34 utiles). Ils sont construits par lots. Le lot 1 prend les urgences qui ne demandent aucun choix d'architecture.

## Décision (à l'essai)

1. **`Page(lang:, description:, image:)`** : la langue (`fr`, `en`, `fr-CA`), la description (300 caractères au plus) et l'image de partage. Le serveur les met dans l'en-tête ; le moteur reprend la langue.
2. **`alt` obligatoire sur `Image`**, `alt: ""` pour un décor. Avant, un oubli rendait l'image invisible aux aveugles sans rien dire.
3. **`Video(source:, label:)`** : `.mp4` ou `.webm`, avec ses boutons ; `label` obligatoire ; jamais de lecture automatique.
4. **`Table(caption:, head:, rows:)`** : chaque ligne a autant de cases que `head` ; défile de côté sur un téléphone.
5. **`Input(…, lines: 2 à 20)`** : un texte long, retours à la ligne gardés, 1000 caractères sans `max` (2000 au plus).
6. **`Choice(value:, label:, options:)`**, `menu: true` pour une liste déroulante : la valeur est un texte et n'accepte que ses options.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le tableau | sept blocs comme HTML (`Tr`, `Td`…) ; **un bloc et des listes** | Un bloc, trois paramètres : un débutant écrit des lignes, pas des balises. |
| Le texte long | un bloc `TextArea` ; **`Input(lines:)`** | `lines:` : c'est le même champ, en plus haut ; pas de mot nouveau. |
| Le choix | deux blocs (`Radio`, `Select`) ; **un `Choice`** et `menu: true` | Un bloc : c'est le même geste (choisir une option) ; l'auteur décide seulement de l'allure. |
| La vidéo | lecture automatique permise ; **jamais** | Jamais : le web en a fait un défaut (le son qui part tout seul, les données consommées). |
| `alt` | facultatif ; **obligatoire, vide pour un décor** | Obligatoire : un oubli ne doit plus passer en silence (`ADR-035`). |

## Conséquences

- Une page sans `alt` sur une image est refusée : tous les exemples du dépôt en avaient déjà un, sauf un cas de conformité, corrigé.
- Restent dans les lots suivants : l'envoi d'un formulaire (il faut décider où vont les messages), les listes et la répétition, le survol comme signal, le « sinon », le calcul, et les éléments utiles du CSS et de JavaScript.

## Critères de validation

- Leçons 40 à 44 ; test du moteur `plat.rs` (`le_lot_1_langue_video_tableau_texte_choix`).
