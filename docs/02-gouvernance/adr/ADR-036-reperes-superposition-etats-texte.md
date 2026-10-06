# ADR-036 — Les repères, les titres jusqu'à H6, le texte qui grandit, les états, la superposition

- Statut : EXPÉRIMENTATION
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : `proposals/Claude/pourquoi-ces-refus-2026-10/SYNTHESE.md` ; avis des humains, de Gemini et de ChatGPT du 2026-10-06
- Validation : Yocthan, le 2026-10-06 : « Oui, vas-y. » Les noms restent à revoir avec les autres (majuscules et casse : question posée à ChatGPT et Gemini).
- Projets affectés : HoloCode, HoloEngine

## Décision (à l'essai)

1. **Les repères** : `Header`, `Nav`, `Main`, `Footer`, en blocs. `Header` et `Footer` posés directement dans la page sortent du contenu principal ; `Main` ne se pose que directement dans la page. Les quatre avis l'ont demandé ; 63 % des utilisateurs de lecteurs d'écran se servent des repères au moins parfois (WebAIM 2024).
2. **Les titres jusqu'à `H6`** (correction d'`ADR-020`). Même règle : le numéro dit la place dans le plan ; on ne saute pas de niveau.
3. **Le texte qui grandit**, sans rien écrire : une taille de texte en `px` est écrite en `rem` par le moteur (16px = 1rem), donc suit le réglage du visiteur. Au-delà de 24px, un titre rétrécit avec un écran plus étroit que 640px, sans passer sous 24px : `clamp(1.5rem, …vw, …rem)`.
4. **Les états dans un style** : `hover: { … }`, `focus: { … }`, `active: { … }` (correction d'`ADR-017`, sans sélecteur). Le survol n'existe qu'avec une souris (`@media (hover: hover)`) ; le focus est celui du clavier (`:focus-visible`) ; le changement se fait en 0,15 s.
5. **La superposition** : `Stack(children: [ … ])` ; le premier enfant donne la taille, les autres se posent dessus à la place dite par `align:` (neuf places nommées).

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Les repères | des blocs ; un rôle sur un morceau (`Part(role: nav)`) | Des blocs : un non-programmeur comprend `Nav(…)` (Gemini, ChatGPT). |
| Les titres | `H4` au besoin ; `H6` tout de suite | `H6` : trois avis sur quatre ; le besoin des longs documents est connu. |
| Le texte qui grandit | une unité nouvelle (`1text`, `rem`) ; **convertir les `px`** | Convertir : aucun mot nouveau, et le défaut disparaît pour tous les fichiers déjà écrits. Une unité naturelle pourra venir ensuite (idée de ChatGPT). |
| Les états | des réglages sur le bloc (`hover_color:`) ; un signal (`On(B.hover)`) ; **des sous-blocs du style** | Des sous-blocs : la recommandation de Gemini et de ChatGPT, dans l'écriture CSS choisie par Yocthan (`ADR-017`). |
| La superposition | `Stack`, `Overlay`, `Badge`, `Anchor` | `Stack`, le mot de Flutter, que Yocthan connaît ; une place nommée (`align: top_right`) au lieu de coordonnées. `Anchor` (une bulle attachée à un bloc) viendra plus tard. |

Défauts du web évités : `position: absolute` et `z-index: 99999` ; un survol qui reste collé sur un téléphone ; des titres en pixels qui ignorent le réglage du visiteur ; `section` et `article`, que personne ne sait distinguer (non repris).

## Conséquences

### Négatives et risques

- Les noms (`Stack`, `align`, `top_right`, `hover`) attendent la revue des noms et la question des majuscules.
- La valeur en `vw` d'un grand titre ne suit pas le réglage du visiteur sur un écran étroit ; le minimum de 24px et le maximum, eux, le suivent.
- `aside` n'est pas repris.

## Critères de validation

- Leçons 35 à 38 ; le site de référence (en-tête, menu et pied en repères ; une pastille sur l'accueil ; un bouton qui réagit au survol).
- Tests du moteur : `plat.rs` (`les_reperes_les_etats_la_superposition`).
