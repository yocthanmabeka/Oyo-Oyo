# ADR-110 — Faire vibrer le téléphone : `Device(kind: vibration)`

- Statut : ACCEPTÉ (Yocthan, 2026-10-09 : « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #239 (« Dernière dette du web : la vibration du téléphone »), l'une des douze dernières dettes du web, validées d'avance par Yocthan le 2026-10-09 ; le grand tableau du web, où « géolocalisation, caméra, vibration » était « en partie » (« Pas encore la vibration ») ; `ADR-094` (l'appareil), `ADR-061` (un son joué par une règle de temps ou une règle qui guette), `ADR-107` (le partage, la sorte d'avant).
- Validation : Yocthan, le 2026-10-09, d'avance, avec les douze dernières dettes du web.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Un jeu, une minuterie, un bouton qui répond : sur un téléphone, une courte vibration dit « c'est fait » sans qu'on regarde l'écran. Le web l'offre par `navigator.vibrate(…)`.
- Écrite à la main, la vibration du web a ses pièges :
  - elle n'existe pas partout : jamais sur un iPhone (Safari n'a pas `navigator.vibrate`), ni sur un ordinateur. Une page qui l'appelle sans vérifier casse sur l'iPhone ;
  - Chrome la refuse tant que le visiteur n'a pas touché la page, avec un avertissement dans la console ;
  - son motif est une liste de nombres sans unité, où vibrations et silences alternent, et Chrome le coupe sans rien dire s'il est trop long (10 secondes par durée, 99 durées) ;
  - rien ne la relie au réglage du visiteur qui demande moins de mouvement ;
  - elle sert souvent de seul signe : un visiteur sans vibreur (iPhone, ordinateur, vibreur coupé) ne sait pas ce qui s'est passé.

## Décision

```holo
Page(
  title: "Catch the diamond",
  state: State(caught: 0, x: 10, y: 50, cx: 80, cy: 50),
  children: [
    Device(name: Buzz, kind: vibration, for: [100ms, 80ms, 100ms], label: "Two buzzes when you catch the diamond"),
    P("Caught: {caught}"),
    Board(height: 220px, children: [
      Shape(name: Target, form: diamond, color: "#FF4D6D", size: 48px, x: cx, y: cy),
      Shape(name: Me, form: square, color: "#E9B44C", size: 48px, x: x, y: y),
    ]),
  ],
  rules: [
    On(Key.left, effect: x.sub(5)), On(Key.right, effect: x.add(5)),
    When(Me, meets: Target, effect: [caught.add(1), cx.random(100), Buzz.play]),
  ],
)
```

1. **`Device(kind: vibration)`**, une sorte d'appareil de plus : `Buzz.play` fait vibrer, `Buzz.stop` arrête.
2. **`for:`** dit combien de temps : une durée (`for: 200ms`), ou une liste qui alterne vibration et silence (`for: [100ms, 80ms, 100ms]` : vibre, se tait, vibre). 200 ms si rien n'est écrit. Dix durées au plus, une seconde en tout au plus.
3. **D'où elle part** : de partout où un son se joue (`ADR-061`) : un toucher, une touche, un survol, une règle qui guette (`When`, une rencontre), une règle de temps (`Every`, `After`), une fin (`Shop.done`). Il n'y a pas de permission à demander : le navigateur n'en demande pas.
4. **Jamais** :
   - avant que le visiteur ait touché la page une fois : le navigateur l'interdit, et une page ne secoue pas un téléphone posé sur la table, ou ouvert dans un onglet qu'on ne regarde pas. Une vibration demandée avant est oubliée, sans rien dire ;
   - quand le visiteur demande moins de mouvement (`prefers-reduced-motion: reduce` : « Supprimer les animations » sur Android, « Réduire les animations » sur l'iPhone) : celui qui demande à son téléphone de moins bouger ne veut pas qu'une page le secoue. La zone d'état du bloc le dit ;
   - quand la page est cachée : le navigateur l'arrête.
5. **Là où le navigateur ne sait pas vibrer** (un iPhone, un ordinateur) : rien ne vibre, rien ne casse, la page continue. La zone d'état dit « Ce navigateur ne fait pas vibrer. ».
6. **Elle ne dit rien en retour** : ni `done` ni `failed` ; `On(Buzz.done, …)` est refusé. Ce qui compte se montre à l'écran, dans la même règle (`caught.add(1)`) : **la vibration n'est jamais le seul signe**.
7. Sans JavaScript, rien ne vibre ; le bloc le dit, et le reste de la page reste lisible.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Où l'écrire | un bloc `Vibration` ; un réglage du son, `Sound(vibrate: 200ms)` ; **une sorte de plus pour `Device`** | `ADR-094` : une sorte de plus ne fait pas un mot de plus. Une vibration sans son est courante (un téléphone en silencieux), et un son sans vibration aussi |
| Le nom de la sorte | `vibrate` (le verbe du web) ; `haptic` ; `buzz` ; **`vibration`** | un nom, comme les autres sortes (`position`, `camera`, `share`) ; celui de l'API du web et du réglage du téléphone |
| L'action | `Buzz.request` ; `Buzz.vibrate` ; **`Buzz.play`, `Buzz.stop`** | rien de nouveau : une vibration se joue et s'arrête comme un son (`Ding.play`, `Ding.stop`), là où un son se joue ; `request` dit qu'on demande quelque chose au visiteur, et rien n'est demandé |
| La durée | `pattern:` (le nom du web : des nombres sans unité) ; `duration:` ; **`for:`** | `for:` est déjà la durée d'un mouvement et d'une scène (`Enter(for: 0.8s)`, `Scene(for:)`) ; l'unité s'écrit, `200ms` ; une liste pour un motif |
| La borne | celle de Chrome (10 s par durée, 99 durées) ; **une seconde en tout, dix durées** | une vibration est un signal, pas une alarme ; plus longue, elle gêne plus qu'elle ne prévient. Elle se répète par une règle (`Every`) |
| D'où elle part | seulement d'un bouton, comme une permission (`ADR-094`) ; **partout où un son se joue, après le premier toucher** | un jeu vibre quand deux formes se rencontrent, sans bouton. Le premier toucher montre que le visiteur est là, son téléphone en main ; c'est aussi la règle de Chrome |
| Le mouvement réduit | l'ignorer, comme le web ; un réglage de l'auteur ; **pas de vibration** | le visiteur l'a déjà dit à son téléphone ; la page n'a pas à le lui redemander |
| Ce qu'elle dit en retour | `done` à chaque vibration, `failed` sur un iPhone ; **rien** | une vibration n'attend rien du visiteur. Un `failed` sur l'iPhone ferait croire qu'ailleurs on peut s'en remettre à elle : le signe à l'écran se montre toujours |

## Ce qui est refusé, et pourquoi

- `for:` sans unité, nul, de plus de dix durées ou de plus d'une seconde en tout ; `for:` sur une autre sorte d'appareil : la raison est dite, avec la bonne écriture.
- `Buzz.request`, `Buzz.write` : une vibration se joue (`play`) et s'arrête (`stop`). `play` sur une autre sorte d'appareil : seule une vibration se joue.
- `On(Buzz.done, …)`, `On(Buzz.failed, …)` : une vibration ne dit rien en retour.
- `value:` : une vibration ne rend rien à la page.

## Les défauts du web évités

- **La page qui casse sur un iPhone** (`navigator.vibrate` absent) : ici, rien ne casse, et la zone d'état le dit.
- **La vibration avant tout geste** (une page ouverte en fond qui secoue le téléphone) : ici, jamais avant le premier toucher, et sans avertissement dans la console.
- **Le réglage « moins de mouvement » ignoré** : ici, respecté.
- **Un motif de nombres sans unité, coupé en silence** : ici, des durées avec leur unité, bornées, refusées avec la raison.
- **La vibration comme seul signe** : ici, elle ne dit rien en retour ; le guide et la leçon montrent toujours le signe à l'écran.

## Dettes

- Une règle `Every` rapide peut faire vibrer souvent : la borne vaut pour une vibration, pas pour leur suite.
- Les vibrations plus fines (la force, le retour haptique de l'iPhone) : le web ne les offre pas.
- Un vrai téléphone : le vibreur du Samsung de Yocthan, avec « Supprimer les animations » ; l'essai remplace `navigator.vibrate`.
- Le grand tableau du web : « géolocalisation, caméra, vibration » passe de « En partie » à « Oui », à la fusion.

## Critères de validation

- Tests du moteur :
  - `a_vibration_plays_like_a_sound_and_says_nothing_back` : accepté d'un bouton, d'une touche, d'une règle de temps, d'une rencontre ; le motif en millisecondes ; refusés `request`, `play` sur une autre sorte, `On(Buzz.done, …)`, `value:` ;
  - `a_vibration_is_short` : 200 ms au départ ; refusés, avec la raison, une durée sans unité, nulle, plus d'une seconde en tout, plus de dix durées, `for:` sur une autre sorte.
- Dans Chrome : « faire vibrer le téléphone : un toucher, une rencontre, le mouvement réduit, un navigateur sans vibreur (leçon 133) ».
  - Avant tout toucher, rien ne vibre ; un toucher vibre `[200]` ; la rencontre, `[100, 80, 100]`.
  - Le mouvement réduit émulé : rien ne vibre, la zone d'état le dit, le compte à l'écran avance.
  - Sans `navigator.vibrate` : aucune erreur, la page continue, la zone d'état le dit.
  - axe-core : zéro défaut.
- Leçon `133-faire-vibrer-le-telephone.holo`.
