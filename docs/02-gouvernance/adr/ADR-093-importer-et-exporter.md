# ADR-093 — Lot 9 : importer et exporter un fichier de ses valeurs, `Transfer`

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-09 : intégrer le travail de Codex quand il est bien fait)
- Date : 2026-10-08 (la construction, par Codex) ; 2026-10-09 (la relecture, l'intégration et la validation)
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`, lot 9 : « appareil sur permission, hors-ligne, import et export, notifications ») ; la reprise du lot 9 par Codex (issue 202, « Codex reprend aussi le lot 9 ») ; la PR 203 et son compte rendu (`proposals/GPT5.6/fin-lot9-2026-10-08/README.md`).
- Validation : Yocthan, le 2026-10-09.
- Projets affectés : HoloCode, HoloEngine
- Auteur de la construction : **Codex** (PR 203) ; relu, intégré et documenté par Claude. Les trois autres capacités : `ADR-094` (l'appareil), `ADR-095` (les notifications), `ADR-096` (hors-ligne).

## Contexte

Un visiteur veut garder ses notes, ou les passer d'un navigateur à l'autre, sans compte ni serveur : un fichier sur son appareil. En JavaScript, il faut un `Blob`, un lien `download`, un `input type="file"`, `FileReader`, `JSON.parse`, puis vérifier à la main ce qui revient ; la plupart des pages ne vérifient rien, et un fichier faux casse la page à moitié.

## Décision

```holo
Page(
  state: State(note: "Hello", notes: ["First note"]),
  children: [
    Input(label: "My note", value: note),
    Transfer(name: File, label: "My notes", file: "notes.json", values: [note, notes]),
    Button(name: Export, text: "Export my notes"),
    Button(name: Import, text: "Import my notes"),
  ],
  rules: [ On(Export.tap, effect: File.export), On(Import.tap, effect: File.import) ],
)
```

1. **`Transfer(name:, label:, file:, values:)`** annonce un fichier JSON et les valeurs qu'il porte : de 1 à 16 noms, chacun une fois, tous déclarés dans `State`. Jamais une valeur partagée (`ADR-079`), calculée, l'heure, ni le compte.
2. **`File.export`** télécharge le fichier `file` (un simple nom en `.json`, 80 caractères au plus) avec ces valeurs, et elles seules : 64 Ko au plus.
3. **`File.import`** ouvre le choix d'un fichier (64 Ko au plus). Le moteur le relit en entier **avant** de changer quoi que ce soit : un objet JSON, toutes les valeurs annoncées, une fois chacune, aucune autre ; chacune de sa sorte et dans ses bornes (un nombre entier ou à virgule de la page, un texte de 200 caractères sans caractère nul, une liste de 200 éléments, des éléments à champs avec exactement leurs champs). **Tout ou rien** : un fichier faux ne change aucune valeur.
4. Chaque transfert dit sa fin : `File.done`, ou `File.failed` avec la raison, écrite dans un état que le lecteur d'écran annonce (`role="status"`).
5. Comme les quatre capacités du lot 9 : le bloc se range directement dans les enfants de `Page`, avec un nom et une étiquette (`label`, 200 caractères au plus) ; `export` et `import` ne partent que du toucher d'un bouton (`On(Export.tap, …)`), jamais d'un minuteur, du démarrage ou d'une autre fin. L'auteur n'écrit aucun JavaScript ; le code du navigateur (`capabilities.js`) n'est chargé que si la page déclare une capacité.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Quelles valeurs | toute la page ; **celles que l'auteur annonce** | le fichier ne dit que ce que l'auteur a écrit, et l'import ne peut rien changer d'autre |
| Un fichier en partie faux | prendre ce qui va ; **tout refuser** | une page à moitié importée est pire qu'un refus : on ne sait plus ce qu'on a |
| Le format | CSV ; **JSON** | les listes et les éléments à champs s'y écrivent ; le moteur sait déjà relire un JSON avec méfiance (`ADR-077`) |
| Qui déclenche | n'importe quelle règle ; **un bouton touché** | le navigateur exige un geste du visiteur pour ouvrir un fichier ; l'écrire dans le langage évite une page qui demande un fichier à l'ouverture |

## Les défauts du web évités

- **`JSON.parse` sans vérification**, qui laisse entrer une clé inconnue (`admin: 1`), un nombre en texte ou un fichier énorme : ici, chaque clé et chaque sorte sont vérifiées, la taille d'abord.
- **L'import à moitié** : ici, tout ou rien.
- **Le téléchargement de tout l'état de la page** (des valeurs internes, un jeton) : ici, seulement les valeurs annoncées.

## Dettes

- Le fichier est un JSON ; un CSV (pour un tableur) n'existe pas encore.
- Glisser-déposer un fichier sur la page n'importe pas encore.

## Critères de validation

- Tests du moteur : `imports_are_atomic_declared_and_bounded` (une clé de trop, une clé répétée, un nombre en texte, une valeur manquante, un texte de 201 caractères : tout refusé ; un import par minuteur refusé dès la vérification) ; `imports_do_not_fire_completion_twice_or_drop_record_fields`.
- Dans Chrome : « lot9 : importer et exporter, limites, import atomique » (un vrai téléchargement, un vrai sélecteur de fichier ; le fichier piégé refusé sans rien changer ; 64 Ko).
- Leçon `116-importer-et-exporter.holo`.
