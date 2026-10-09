# ADR-088 — Lot 9, quatrième pas : des formes venues d'une liste, et un module qui dessine

- Statut : ACCEPTÉ
- Date : 2026-10-08
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (lot 9 : « des modules qui rendent des ordres de dessin vérifiés ») ; la revue de Codex (« un module de dessin ne reçoit pas le vrai Canvas du navigateur : il rend une liste bornée […] le moteur vérifie la quantité, puis dessine ») ; `ADR-077`, `ADR-086` ; le plan du lot 9 montré à Yocthan le 2026-10-07
- Validation : Yocthan, le 2026-10-07, en confiant le lot 9 à la session du nuage ; vérifié dans Chrome avant la fusion
- Projets affectés : HoloCode, HoloEngine

## Décision

```holo
module "flower.wasm"
Page(
  state: State(petals: 6, flower: []),
  modules: [ Module(name: Flower, source: "flower.wasm", input: [petals], output: [flower]) ],
  children: [
    Drawing(label: "A flower", width: 320, height: 180, shapes: flower, children: [ Rect(x: 0, y: 0, width: 320, height: 180, fill: "#16213e") ]),
    Button(name: Draw, text: "Draw"),
  ],
  rules: [ On(Draw.tap, effect: Flower.run) ],
)
```

1. **`Drawing(…, shapes: flower)`** : les formes d'un dessin peuvent venir d'une liste à champs, une forme par élément, devant les formes écrites. Un élément : `form` (`"rect"`, `"circle"`, `"line"` ou `"path"`), les champs des formes écrites (`x`, `y`, `width`, `height`, `radius`, `r`, `d`, `fill`, `stroke`, `thickness`, `opacity`), et `x1`, `y1`, `x2`, `y2` pour un trait.
2. **Un module qui dessine rend cette liste**, comme n'importe quelle valeur (le second contrat, `ADR-077`) : il ne reçoit ni le dessin du navigateur, ni la page. Des données reçues (`Data`) ou une règle (`push`) peuvent la remplir aussi.
3. **Le moteur vérifie chaque élément** comme une forme écrite : la sorte, des mesures de 0 à 4 000, des couleurs, un tracé filtré. Un élément faux est laissé de côté, sans arrêter les autres : ces formes arrivent pendant la visite. Une liste garde ses bornes : cent éléments au plus.
4. **Le dessin suit sa liste** : la page redessine les seules formes de la liste quand elle change.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Comment un module dessine | lui donner un canevas ; une suite d'ordres dans un format à part ; **une liste de formes, une valeur comme une autre** | rien de nouveau à apprendre ni à garder : le contrat des modules et les listes existent ; le moteur vérifie comme d'habitude |
| Un élément faux | tout refuser ; **le laisser de côté** | les formes arrivent pendant la visite : une forme fausse ne doit pas effacer les autres. Le module, lui, est déjà refusé en entier s'il rend une valeur non annoncée (`ADR-077`) |

## Ce qui n'est pas fait

- Plus de cent formes par liste (la borne des listes).
- Des animations image par image : un module ne tourne qu'à la demande (`run`).

## Critères de validation

- Tests du moteur : des formes reçues, dessinées après les formes écrites ; laissées de côté : une mesure négative, un tracé avec `<script>`, une sorte inconnue ; une couleur fausse remplacée par la couleur du texte ; redessinées pour un nouvel état ; refusés à la lecture : une liste absente, une liste de textes.
- Dans Chrome (leçon 110) : aucune forme au départ ; le module dessine 6 pétales et le cœur ; 9 pétales ensuite, « 11 formes dessinées ».
