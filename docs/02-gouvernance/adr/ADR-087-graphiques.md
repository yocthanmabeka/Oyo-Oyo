# ADR-087 — Lot 9, troisième pas : les graphiques d'un tableau de bord, `Chart`

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`, lot 9, et le dixième parcours : « un tableau de bord qui reçoit des données, les calcule et les dessine ») ; la règle de Codex : un mot pour une intention fréquente ; le plan du lot 9 montré à Yocthan le 2026-10-07
- Validation : Yocthan, le 2026-10-07, en confiant le lot 9 à la session du nuage ; le mot proposé (`Chart(kind: bars, over:, value:, label:, title:)`) lui a été montré avant d'être construit ; vérifié dans Chrome avant la fusion
- Projets affectés : HoloCode, HoloEngine

## Décision

```holo
Page(
  state: State(sales: [], period: "", amount: ""),
  data: Data(from: "sales.json"),
  children: [
    Chart(kind: bars, over: sales, value: amount, label: period, title: "Sales of the week"),
    Chart(kind: pie, over: sales, value: amount, label: period, title: "Share of each day"),
  ],
)
```

1. **`Chart(kind:, over:, value:, label:, title:, color:)`** : un graphique, dessiné par le moteur en SVG, d'après une liste à champs (`over`). `value` est le champ du nombre, `label` celui du nom de chaque élément.
2. **Trois sortes** : `bars` (des barres, le nombre au-dessus, le nom dessous), `line` (une courbe), `pie` (des parts, et leur légende). `color` change la couleur des barres ou de la courbe ; les parts prennent une suite de couleurs distinctes.
3. **Pour tous** : `title` est obligatoire et se lit au-dessus du graphique ; le dessin est caché au lecteur d'écran, qui lit à la place un tableau caché avec les mêmes chiffres (une colonne par champ).
4. **Le graphique suit sa liste** : des données reçues (`Data`), une liste calculée (`Filter`), un élément ajouté ; le moteur le redessine. La page fabriquée par le serveur le montre déjà rempli : un robot, ou un navigateur sans JavaScript, le voit.
5. **Les nombres** sont lus dans le champ, « 12 », « 12.5 » ou « 12,5 » ; ce qui n'en est pas compte pour 0. Une liste vide montre un tiret.

## Corrigé en chemin

Le moteur relisait les données d'une page au démarrage, même quand le serveur venait de les mettre dans la page. Une vente ajoutée avant leur retour était effacée. Maintenant, la page fabriquée par le serveur avec ses données (`data-received`) ne les relit qu'au rythme de la page, jamais aussitôt (`ADR-064`, le comportement change seulement dans ce cas).

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Dessiner des données | le dessin à la main (`Drawing` et `Repeat`) ; un module qui dessine ; **un mot, `Chart`** | l'intention la plus fréquente d'un tableau de bord ; une ligne au lieu de dizaines |
| L'accessibilité | une description à écrire ; **un tableau caché, fait par le moteur** | les vrais chiffres, toujours à jour, sans rien à écrire |
| Redessiner | toute la page ; **le dessin seul, quand sa liste change** | la page ne bouge pas, le focus reste où il est |

## Ce qui n'est pas fait

- Des axes gradués, plusieurs séries, des nombres négatifs, toucher une barre.
- Un module qui rend des ordres de dessin : l'étape 4 du lot 9.

## Critères de validation

- Tests du moteur : les barres (la plus haute prend toute la hauteur), une courbe, des parts et leur légende ; le tableau caché, échappé ; redessiné pour un nouvel état ; un réglage de page trafiqué ou une liste absente ne dessinent rien ; une liste vide ; refusés : un réglage manquant, une sorte inconnue, une liste absente, une liste de textes, un champ absent, un titre vide, une couleur qui n'en est pas une.
- Dans Chrome (leçon 99) : cinq barres et cinq parts d'après les données du serveur ; une vente ajoutée, six et six, et gardée une seconde et demie plus tard ; le tableau caché lit « Samedi 180 », sans faire défiler la page.
- L'audit axe-core : 99 leçons, 0 défaut en clair.
