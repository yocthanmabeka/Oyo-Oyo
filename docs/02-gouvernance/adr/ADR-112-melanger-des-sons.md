# ADR-112 — Mélanger des sons : plusieurs à la fois, un fondu (`fade:`), un volume qui suit une valeur (`volume: pluie`)

- Statut : ACCEPTÉ (Yocthan, 2026-10-09 : « tu le valides déjà, tu le fais déjà » ; essayé et validé par Yocthan le 2026-10-10 : « je viens de faire tous les essais et je valide »)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #241 (« Dernière dette du web : mélanger des sons ») ; le grand tableau du web, où le son était « en partie » : `Sound(volume:, loop:)`, `play`, `stop` (`ADR-031`, `ADR-061`).
- Validation : Yocthan a validé d'avance les douze dernières dettes du web, le 2026-10-09.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Aujourd'hui : `Sound(name:, source:, weight:, label:, volume:, loop:)`, et les capacités `play` et `stop` (`ADR-031`, `ADR-061`). Chaque `Sound` est un `<audio>` à lui. `play` le met en pause, le ramène au début et le joue ; `stop` le met en pause et le ramène au début. Aucun des deux ne touche aux autres sons.
- **Vérifié dans Chrome avant d'écrire** (la leçon 79, avec le moteur de `main`) :
  - deux sons différents jouent déjà ensemble : la pluie en boucle continue pendant que « Fort » joue ; aucun des deux n'est en pause ;
  - un même son relancé repart du début (de 0,12 s à 0,07 s) : il ne se superpose pas à lui-même, il se coupe.
- Ce qui manquait :
  - un fondu : un son qui part ou s'arrête d'un coup claque, et une ambiance qui se coupe net surprend ;
  - un volume qu'on change pendant que le son joue : une glissière par son, une vraie table de mixage.
- Le volume d'un `<audio>` ne se règle qu'en JavaScript, d'un coup (`ADR-061`). Sur iPhone et iPad, Safari l'ignore tout à fait : `audio.volume` y vaut toujours 1. Le web a pour cela Web Audio : des gains qu'on fait glisser.
- La politique des navigateurs : un son ne part qu'après un geste du visiteur ; avant, le navigateur refuse, et la page continue sans lui. Mais un navigateur peut le laisser partir plus tôt : un site souvent écouté, un réglage, ou nos essais, qui lancent Chrome avec `--autoplay-policy=no-user-gesture-required`. Le moteur s'en remettait au navigateur.

## Décision

```holo
Page(
  state: State(pluie: 60, vent: 30),
  children: [
    Sound(name: Pluie, source: "pluie.wav", loop: true, fade: 2s, volume: pluie),
    Sound(name: Vent, source: "vent.wav", loop: true, fade: 3s, volume: vent),
    Slider(value: pluie, label: "Volume de la pluie", min: 0, max: 100),
    Slider(value: vent, label: "Volume du vent", min: 0, max: 100),
    Button(name: Lancer, text: "La pluie et le vent"),
    Button(name: Silence, text: "Tout arrêter"),
  ],
  rules: [
    On(Lancer.tap, effect: [Pluie.play, Vent.play]),
    On(Silence.tap, effect: [Pluie.stop, Vent.stop]),
  ],
)
```

1. **Plusieurs sons à la fois : rien de nouveau.** Chaque `Sound` est une piste. `[Pluie.play, Vent.play]` les fait entendre ensemble, `[Pluie.stop, Vent.stop]` arrête tout.
2. **`fade: 2s`** : un fondu, à l'entrée et à la sortie.
   - `play` : le son repart du début et monte du silence jusqu'à son volume, en 2 secondes.
   - `stop` : il descend jusqu'au silence en 2 secondes, en jouant encore ; puis il se met en pause et revient au début (`ADR-061`).
   - Une durée de `100ms` à `5s`, en ligne droite.
   - Un son sans boucle qui finit tout seul finit comme il est enregistré.
3. **`volume: pluie`** : le volume suit une valeur de la page.
   - Un nombre entier, de 0 (muet) à 100 (le plus fort), comme une glissière, une barre de progression ou une place sur un plateau.
   - Quand la valeur change (une glissière, une règle `pluie.add(10)`), le volume glisse jusqu'à elle en un dixième de seconde, que le son joue ou non.
   - La valeur ne dépasse jamais 100, même sans glissière pour la borner.
   - Le volume écrit garde son écriture de 0 à 1 : `volume: 0.4` (`ADR-061`).
4. **La table de mixage** : une glissière par son, `Slider(value: pluie, label: "Volume de la pluie", min: 0, max: 100)`. C'est un mot qui existe déjà. Son étiquette est obligatoire, et le lecteur d'écran la dit. Elle marche au doigt, à la souris et au clavier : les flèches, Page précédente et Page suivante (de 10 en 10), Début et Fin.
5. **Le mélangeur de la page.** Un son qui a `fade:` ou un volume suivi passe par Web Audio : son volume, puis son fondu, deux gains que le moteur fait glisser.
   - Le mélangeur naît au premier son mélangé qui joue. Il s'endort quand aucun de ses sons ne joue.
   - Un son venu d'un autre serveur n'y passe pas : Web Audio le rendrait muet. Il joue à son volume, sans fondu.
   - Les autres sons jouent comme avant (`ADR-061`).
6. **Jamais un son avant un geste.** Le moteur garde la règle des navigateurs, et il est plus strict qu'eux.
   - Avant le premier geste du visiteur sur la page (un toucher, un clic, une touche), il ne joue aucun son, même si une règle de temps le demande, même si le navigateur le permettrait.
   - La demande est oubliée, pas gardée pour plus tard. Le mélangeur n'est même pas créé.
   - Le mouvement réduit (`prefers-reduced-motion`) ne concerne pas le son : un fondu n'est pas un mouvement à l'écran, il reste.
7. **Un lecteur ne change pas.** `Sound(label:)` est dans la main du visiteur : il le lance, l'arrête et règle son volume lui-même, avec les boutons du navigateur, au clavier et au lecteur d'écran. `fade:` et un volume suivi y sont refusés.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Jouer plusieurs sons ensemble | un bloc de plus, `Mix(children: [ … ])` ; **rien de nouveau** | vérifié dans Chrome : deux `Sound` jouent déjà ensemble ; un bloc de plus n'apprendrait rien |
| Le fondu | **`fade: 2s`**, à l'entrée et à la sortie ; `fadeIn:` et `fadeOut:` séparés ; une capacité `Pluie.fadeIn` ; `Pluie.play(fade: 2s)` | un mot, une durée écrite comme celles des règles (`Every(1s)`) ; l'entrée et la sortie sont le plus souvent égales ; une capacité ne prend pas de paramètre dans le langage ; deux mots séparés restent une dette |
| Les bornes du fondu | sans borne ; jusqu'à 30 s ; **de 100ms à 5s** | en deçà de 100 ms, on ne l'entend pas ; au-delà de 5 s, un son qu'on arrête ne se tait pas assez vite, et la personne qui attend le silence pour écouter son lecteur d'écran attend trop |
| La forme du fondu | une courbe ; **une droite** | la plus simple à prévoir ; c'est elle que l'essai mesure |
| Changer le volume pendant que le son joue | une capacité `Pluie.volume(50)` ; un bloc `Mixer` avec ses glissières à lui ; **`volume: pluie`, une valeur de la page** | une valeur se change déjà par une glissière (`Slider`) ou par une règle (`pluie.add(10)`) : rien de nouveau à apprendre ; une capacité ne prend pas de nombre |
| L'échelle de cette valeur | de 0 à 1, comme `volume: 0.4` ; **de 0 à 100** | une glissière, une barre de progression et une place sur un plateau vont de 0 à 100 ; une glissière ne prend pas de nombre à virgule (`ADR-066`) ; le volume écrit garde son écriture (`ADR-061`) |
| Le changement de volume | d'un coup ; **en glissant, en un dixième de seconde** | d'un coup, le son claque ; plus lent, la glissière semble en retard |
| Ce qui règle le volume, dans le moteur | `audio.volume`, comme `ADR-061`, changé par petits pas avec une minuterie ; **Web Audio : un gain pour le volume, un gain pour le fondu** | sur iPhone, `audio.volume` ne se règle pas, un gain si ; un gain glisse sur l'horloge du son, alors qu'une minuterie ralentit dans un onglet caché |
| Avant un geste du visiteur | s'en remettre au navigateur ; **le moteur refuse lui aussi** | un navigateur peut laisser partir un son tout seul ; le langage promet « jamais de lecture à l'ouverture » (`ADR-061`) |

## Ce qui est refusé, et pourquoi

- `fade:` qui n'est pas une durée de 100ms à 5s : `fade: 2`, `fade: 0s`, `fade: 50ms`, `fade: 6s`.
- `volume:` d'une valeur qui n'est pas un nombre entier déclaré de 0 à 100 : un texte, une liste, un nombre à virgule, une valeur inconnue, ou une valeur qui part au-dessus de 100. Le message dit l'échelle et montre la glissière.
- `fade:` et un volume suivi sur un lecteur (`label:`) : le lecteur est dans la main du visiteur.
- Un écho et les autres effets : plus tard (voir les dettes).

## Les défauts du web évités

- **Le son qui part tout seul** : le moteur ne joue rien avant un geste du visiteur, même là où le navigateur le permettrait.
- **Le volume qui ne se règle pas sur iPhone** (`audio.volume`, ignoré par Safari) : un son mélangé passe par un gain Web Audio.
- **Le son qui claque** quand il part, s'arrête ou change de volume d'un coup : il glisse.
- **Le fondu écrit à la main**, une minuterie qui change `audio.volume` par petits pas et ralentit dans un onglet caché : les glissements de Web Audio, sur l'horloge du son.
- **Un mélangeur qui tourne pour rien** et tient l'appareil éveillé : il naît au premier son mélangé et s'endort dès qu'aucun ne joue.
- **Le son d'un autre serveur rendu muet** par Web Audio : il ne passe pas par le mélangeur.

## Dettes

- Un écho, une réverbération, un son placé dans l'espace (à gauche, au loin) : pour les jeux, plus tard.
- Un fondu d'entrée et un fondu de sortie différents (`fadeIn:`, `fadeOut:`) ; un fondu à la fin naturelle d'un son sans boucle.
- Le volume écrit seul (`volume: 0.4`, sans fondu ni valeur) passe encore par `audio.volume` : sur iPhone, il reste sans effet. Il passera par le mélangeur quand on pourra l'essayer sur un iPhone.
- Rien n'a été essayé dans Safari ni sur un iPhone : seulement dans Chromium. À essayer à l'oreille sur le téléphone de Yocthan (Chrome sur Android).
- Un son ne dit pas encore qu'il a fini (un signal `Pluie.ended`).
- Un son n'est toujours pas décrit pour qui n'entend pas (`ADR-031`) : la leçon montre un mot à l'écran, mais rien ne l'oblige.

## Critères de validation

- Tests du moteur :
  - `a_sound_fades_in_and_out` : le fondu en millisecondes ; le volume écrit d'un son mélangé donné au mélangeur ; un son ordinaire inchangé ; les fondus refusés ; un lecteur refusé ;
  - `a_sound_follows_a_value_of_the_page` : la valeur suivie ; jamais au-dessus de 100 ; les refus (inconnue, texte, liste, virgule, au-dessus de 100).
- Dans Chrome :
  - « mélanger des sons : deux à la fois, le fondu qui monte puis descend, le volume qui suit sa glissière (leçon 135) » :
    - les deux glissières nommées pour le lecteur d'écran, et l'audit axe-core sans défaut ;
    - le volume de la pluie mesuré à plusieurs moments : il monte ;
    - la pluie et le vent jouent ensemble, aucun n'est en pause ;
    - la glissière au clavier (Page suivante) : le volume rejoint 20 sur 100 ;
    - `stop` : le volume descend pendant que la pluie joue encore, puis elle est en pause, au début ; le vent continue ;
    - tout arrêté, le mélangeur s'endort ;
  - « un son ne part jamais avant un geste du visiteur, même là où le navigateur le permettrait (ADR-112) » : la règle de temps demande deux sons au moins trois fois, aucun n'est entendu et le mélangeur n'existe pas ; après un toucher, les deux sont entendus.
  - Chacun sait échouer : sans le fondu, sans le volume suivi, sans le fondu de sortie, ou sans la règle stricte, l'essai rate.
- Leçon `135-melanger-des-sons.holo`.
