# ADR-094 — Lot 9 : l'appareil sur permission, `Device`

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-09 : intégrer le travail de Codex quand il est bien fait)
- Date : 2026-10-08 (la construction, par Codex) ; 2026-10-09 (la relecture, l'intégration et la validation)
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`, point 5 : « l'appareil (caméra, position, presse-papiers) n'est jamais donné à un module : le moteur demande la permission, sur un geste clair du visiteur, et ne remet que le résultat ») ; la reprise du lot 9 par Codex (issue 202) ; la PR 203 et son compte rendu (`proposals/GPT5.6/fin-lot9-2026-10-08/README.md`) ; les modules enfermés (`ADR-045`, `ADR-077`).
- Validation : Yocthan, le 2026-10-09.
- Projets affectés : HoloCode, HoloEngine
- Auteur de la construction : **Codex** (PR 203) ; relu, intégré et documenté par Claude. Voir aussi `ADR-093`, `ADR-095`, `ADR-096`.

## Contexte

La position, le presse-papiers, la caméra et le microphone sont les capacités les plus sensibles du navigateur. Sur le web, une page peut les demander dès son ouverture, une bibliothèque peut les demander à la place de l'auteur, et une capture oubliée continue de tourner (le voyant de la caméra reste allumé).

## Décision

```holo
Page(
  state: State(where: ""),
  children: [
    Device(name: Position, label: "My position", kind: position, value: where),
    Button(name: Locate, text: "Ask for my position"),
    P("{where}"),
    Device(name: Camera, label: "Local preview of my camera", kind: camera),
    Button(name: Start, text: "Allow the camera"), Button(name: Stop, text: "Stop the camera"),
  ],
  rules: [ On(Locate.tap, effect: Position.request), On(Start.tap, effect: Camera.request), On(Stop.tap, effect: Camera.stop) ],
)
```

1. **`Device(name:, label:, kind:)`**, quatre sortes nommées :
   - `kind: position` : `request` écrit dans un texte de `State` (`value:`) la latitude, la longitude et la précision, en JSON (le langage n'a pas encore de nombres négatifs) ;
   - `kind: clipboard` : `request` lit le presse-papiers dans un texte de `State` (`value:`) ; `write` y copie ce texte ;
   - `kind: camera` : `request` montre un aperçu local de la caméra (une vidéo nommée par l'étiquette) ;
   - `kind: microphone` : `request` active le microphone localement, sans rien enregistrer.
2. **Rien ne part** : ni au serveur, ni à un module. La caméra et le microphone ne donnent ni photo ni fichier ; la position et le presse-papiers ne donnent qu'un texte de la page, vérifié comme un import (`ADR-093`).
3. **La permission reste au navigateur et au visiteur** : la demande ne part que du toucher d'un bouton (`On(Locate.tap, …)`), jamais du démarrage, d'un minuteur ni d'une autre fin ; le moteur vérifie en plus que le navigateur a bien vu un geste du visiteur. Il faut `localhost` ou HTTPS. Un refus est dit dans l'état (`Position.failed`, la raison annoncée) et la page reste utilisable ; la dernière valeur reste.
4. **Une capture s'arrête d'office** : par `Camera.stop` (de n'importe quelle règle : arrêter est toujours permis), après une minute, quand la page change, quand elle est cachée ou fermée. Une permission accordée trop tard, après « Arrêter » ou un changement de page, arrête aussitôt les pistes reçues.
5. Comme `Transfer` : directement dans les enfants de `Page`, un nom et une étiquette ; `Position.done` ou `Position.failed`.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Un bloc par appareil, ou un seul | `Position`, `Clipboard`, `Camera`, `Microphone` ; **`Device(kind:)`** | un seul mot à apprendre, et quatre sortes nommées et contrôlées par le moteur ; une sorte de plus ne fera pas un mot de plus |
| L'appareil donné à un module | un accès libre ; **jamais** | un module enfermé ne touche pas au navigateur (`ADR-045`) ; le moteur demande et ne remet qu'un résultat vérifié |
| La capture | enregistrer, envoyer ; **un aperçu local** | le premier pas n'expose rien ; prendre une photo ou un son est une autre décision |

## Les défauts du web évités

- **La permission demandée à l'ouverture de la page** : ici, seulement sur un bouton touché.
- **La caméra qui reste allumée** : ici, arrêt d'office (une minute, page cachée, page changée) et arrêt d'une permission arrivée trop tard.
- **Un refus qui casse la page** : ici, il est annoncé, et le reste marche.

## Dettes

- La caméra ne prend pas encore de photo ; le microphone ne donne pas de son.
- La position est un texte JSON tant que le langage n'a pas de nombres négatifs.
- Les essais simulent l'appareil (un flux de canvas, la position de DevTools) : une permission réelle sur le Samsung de Yocthan, avec TalkBack, reste à faire.

## Critères de validation

- Test du moteur : `a_permission_never_comes_from_a_timer_or_completion` (une demande sur `Location.done`, une sorte inconnue, une valeur absente : refusées).
- Dans Chrome : « lot9 : presse-papiers Chrome, écriture puis lecture » ; « lot9 : position via API Chrome et refus de permission » ; « lot9 : caméra et microphone, arrêt des pistes (appareils simulés) », dont la permission tardive.
- Leçon `117-appareil-sur-permission.holo`.
