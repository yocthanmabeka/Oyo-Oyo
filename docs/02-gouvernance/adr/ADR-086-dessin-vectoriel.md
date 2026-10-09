# ADR-086 — Lot 9, deuxième pas : le dessin vectoriel déclaré, `Drawing`

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`, lot 9 ; « la capacité gardée, la mécanique refusée : un dessin vectoriel déclaré ») ; les réponses de Gemini (« remplacer le dessin par script par des blocs vectoriels déclaratifs ») et de Codex (« dessin vectoriel », lot des capacités larges) ; le plan du lot 9 montré à Yocthan le 2026-10-07
- Validation : Yocthan, le 2026-10-07, en confiant le lot 9 à la session du nuage ; les mots proposés (`Drawing`, `Rect`, `Circle`, `Line`, `Path`) lui ont été montrés avant d'être construits ; vérifié dans Chrome avant la fusion
- Projets affectés : HoloCode, HoloEngine

## Décision

```holo
Drawing(label: "Un paysage", width: 320, height: 160, children: [
  Rect(x: 0, y: 0, width: 320, height: 160, radius: 12, fill: "#16213e"),
  Circle(x: 250, y: sun, r: 18, fill: "#E9B44C"),
  Line(from: [0, 132], to: [320, 132], stroke: "#8fd3ff", thickness: 1),
  Path(d: "M52 82 L90 54 L128 82 Z", fill: "#c0392b"),
])
```

1. **`Drawing(label:, width:, height:, children: [ … ])`** : un dessin, fabriqué en SVG dans la page. `label` est obligatoire, comme `alt` pour une image : le lecteur d'écran lit le dessin comme une image qui porte ce nom. `width` et `height` sont les unités du dessin (de 1 à 4 000) ; le dessin garde ces proportions et rétrécit avec l'écran.
2. **Quatre formes**, et seulement dans un `Drawing` :
   - `Rect(x:, y:, width:, height:, radius:)` : un rectangle, aux coins arrondis par `radius` ;
   - `Circle(x:, y:, r:)` : un rond, par son centre et son rayon ;
   - `Line(from: [x, y], to: [x, y])` : un trait ;
   - `Path(d: "M10 80 L50 20 Z")` : un tracé SVG, qui commence par M ; seulement des lettres de tracé, des nombres, des espaces et des virgules, 4 000 signes au plus.
3. **Leurs couleurs** : `fill` (le remplissage, la couleur du texte sans rien dire), `stroke` (le bord ; un trait est tracé de la couleur du texte sans rien dire), `thickness` (l'épaisseur du trait), `opacity` (de 0 à 1). Les couleurs sont celles des styles, ou `"none"`.
4. **Une mesure peut être le nom d'un nombre entier de la page** (`y: sun`) : la forme le suit quand il change, sans que la page soit refaite. La page fabriquée par le serveur part des mêmes valeurs.
5. **Refusé** : une forme hors d'un `Drawing` ; une couleur qui n'en est pas une (`url(…)`) ; un tracé qui contient autre chose que des lettres de tracé et des nombres ; un nom qui n'est pas un nombre entier de la page ; plus de 500 formes.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le dessin libre | un canevas où l'on trace trait par trait (Canvas 2D, en JavaScript) ; **des formes déclarées** | les trois avis refusent le dessin impératif ; des formes se lisent, se vérifient et se lisent au lecteur d'écran |
| Les mots | ceux du SVG (`rect`, `cx`, `stroke-width`) ; **des mots lisibles** (`Rect`, `x`, `thickness`) | un débutant n'a pas à connaître le SVG ; le moteur écrit les vrais attributs |
| Les tracés | un petit langage à nous ; **le tracé SVG, filtré** | le format que tous les outils de dessin exportent ; filtré, il ne peut rien contenir d'autre |
| Les mesures liées | des nombres à virgule ; **des nombres entiers** | le cas courant ; une mesure à virgule viendra si un exemple le demande |

## Ce qui n'est pas fait

- Du texte dans un dessin, des dégradés, des formes qu'on touche.
- Les graphiques d'un tableau de bord (`Chart`) : l'étape 3 du lot 9. Un module qui rend des ordres de dessin : l'étape 4.

## Critères de validation

- Tests du moteur : le SVG d'un dessin (rôle d'image, `label` échappé, `viewBox`), un rectangle aux coins arrondis, un rond qui suit deux valeurs, un trait dont une extrémité suit une valeur, un tracé sans remplissage ; la page fabriquée avec les données du serveur ; refusés : sans `label`, une taille nulle, une forme hors d'un dessin, un rond sans rayon, `url(#a)`, `<script>` dans un tracé, un réglage inconnu, une valeur inconnue, un nombre à virgule lié, un trait à une seule mesure, un texte dans un dessin.
- Dans Chrome (leçon 98) : un SVG à sept formes, nommé « Un paysage : une maison, une colline, et le soleil » pour le lecteur d'écran ; le soleil passe de 70 à 30 au toucher de « Lever le soleil » ; le dessin garde ses proportions (2 pour 1).
