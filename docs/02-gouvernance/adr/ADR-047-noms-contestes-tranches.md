# ADR-047 — Les quatorze noms contestés : douze gardés, deux changés

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : la première revue de Codex (PR 39, ses noms « A1 ») ; sa contre-revue (PR 72, `proposals/GPT5.6/autocritique-noms-2026-10-04/`), où il retire presque toutes ses propositions ; sa revue des noms ajoutés (PR 74, § 8) ; les réserves de Gemini (`Grid` face à `Points(grid:)`) ; les noms marqués ⚠ dans `docs/01-holocode/NOMS.md`
- Validation : Yocthan, le 2026-10-06 : « Il y a 14 mots à nommer […] je suis tes recommandations. »
- Projets affectés : HoloCode, HoloEngine

## Contexte

Quatorze noms étaient marqués « contestés » depuis le 2026-10-04 : `Grid`, `Zoom`, `Points`, `Relief`, `Portals`, `pixels:`, `above:`, `shrink:`, `levels:`, `after:`, `fragment:`, `grid:`, `depth:`, `tilt:`. Plusieurs décisions restaient « acceptées pour l'instant, écriture à revoir avec les noms ». La 3D commence : c'est le moment de fixer les mots, avant que d'autres pages ne s'en servent.

## Décision

1. **Douze noms sont gardés** : `Grid`, `Zoom`, `Points`, `Relief`, `Portals`, `pixels:`, `above:`, `shrink:`, `Zoom(levels:)`, `after:`, `fragment:`, `tilt:`.
2. **Deux changent** :
   - `Points(grid:)` devient **`Points(divisions:)`** : un point se morcelle en `divisions` × `divisions` (avec `4`, chaque côté coupé en quatre, soit 16 morceaux).
   - `Points(depth:)` devient **`Points(levels:)`** : combien de fois de suite un point se morcelle.
3. **Les anciennes écritures sont refusées, avec le bon mot** (`ADR-037`) : « « Points(grid:) » s'appelle maintenant « divisions » : écris « divisions » ». L'éditeur et l'extension VS Code le corrigent d'un clic.
4. **Les noms ajoutés que Codex a relus** (PR 74, § 8) sont tous gardés : `If`, `Hr`, `Quote`, `Code`, `Every`, `Board`, `Input`, `Checkbox`, `When`, `Part`, `Use`, `Data`, `Sound`, `Shape`, `keep`, `prices`, `data`, `meets`, `within`, `drag`, `from`, `every`, `value`, `label`, et `form` dans `Shape`.
5. Les décisions qui attendaient les noms (`ADR-017`, `019`, `021`, `022`, `023`, `024`, `026`, `028`, `031`) sont acceptées, écriture comprise.

## Comparaison faite avant de choisir

| Nom | Options | Choix, et pourquoi |
|---|---|---|
| `Points(depth:)` | garder ; **`levels`** (Codex) ; `maxSubdivisions` | Dans un projet en 3D, « depth » doit garder son sens d'espace : la profondeur. `levels` est le mot de Blender pour les niveaux de subdivision, et la même idée que `Zoom(levels:)` : combien de fois l'un dans l'autre. `maxSubdivisions` est long. |
| `Points(grid:)` | garder ; **`divisions`** (Codex) | `grid` était déjà un bloc (`Grid`) et une valeur (`Portals(layout: grid)`) : trois sens pour un mot, le défaut du CSS (`display: grid`, `grid-area`, `grid-template`…). `divisions` est le mot de Three.js. Risque connu : compter les traits au lieu des cases ; le guide dit « chaque côté coupé en quatre ». |
| `above:` | **garder** ; `anchor` (Codex, son choix « le moins solide ») | `above` dit ce que fait le moteur : le point est posé au-dessus du repère. Sur le web, « anchor » est le nom des liens (`<a>`), et HoloCode a ses liens vers un endroit de la page. |
| `tilt:` | **garder** ; `maxTilt` | Codex retire `maxTilt` : à partir d'un demi-tour la rotation est libre, une « borne » serait fausse. Le guide dit ce que fait chaque angle. On le reverra seulement si la 3D change ce contrat. |
| `fragment:` | **garder** ; `subdivideAt` | Codex retire `subdivideAt` (jargon). Défaut connu : il ressemble à `Point(fragments:)` ; le guide dit « la taille où un point se morcelle ». |
| `Grid`, `Zoom`, `Points`, `Relief`, `Portals`, `pixels:`, `shrink:`, `Zoom(levels:)`, `after:` | **garder** | Codex lui-même recommande de les garder après sa contre-revue ; aucun remplaçant n'est plus clair pour un débutant. |
| `When`, `Part`, `Data` (PR 74) | **garder** ; `Watch`, `Component`, `Feed` | Codex reconnaît que `When` et `Part` sont plus simples ; `Feed` promettrait un flux. |
| `Shape(form:)` (PR 74) | **garder** ; `shape` | `Shape(form: circle)` se lit bien ; `Shape(shape: circle)` répète le mot. |

## Conséquences

- Trois fichiers du dépôt changent d'écriture (la leçon 9, la boutique comparée, `exemples/zoom/reduire.holo`), ainsi que le guide.
- `levels` existe dans deux blocs, avec la même idée ; le guide le dit.
- `depth` est libre pour la profondeur de la 3D.
- Plus aucun nom n'est contesté dans `NOMS.md`. Si l'essai avec cinq débutants montre qu'un nom gêne, on le rouvre.

## Critères de validation

- Tests du moteur : `Points(divisions:, levels:)` lus ; `Points(grid:)` et `Points(depth:)` refusés avec le bon mot.
- La leçon 9 s'ouvre dans Chrome et se morcelle comme avant.
