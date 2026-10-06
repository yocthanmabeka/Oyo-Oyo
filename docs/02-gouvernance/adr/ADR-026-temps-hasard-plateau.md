# ADR-026 — Le temps (`Every`), le hasard (`random`), le plateau (`Board`) : un premier jeu

- Statut : ACCEPTÉ — noms et écriture tranchés le 2026-10-06 (`ADR-047`)
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 ; `docs/01-holocode/COMPARATIF-CONCURRENTS.md` (planning, étape 2)
- Validation : Yocthan, le 2026-10-04, après avoir joué : « Techniquement, c'est pas mal. Je valide les jeux. » Avant cela : Yocthan, le 2026-10-04 : « Il faudra qu'on pense à rajouter des mots dans le langage pour pouvoir construire un vrai jeu à 100 % sur HoloCode », puis, sur le planning : « Oui, commence », et « après on passe au point 2 ». L'écriture est une proposition de Claude ; à juger après essai.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Un jeu était impossible : rien ne bougeait tout seul, rien n'était laissé au hasard, rien ne se plaçait librement. La méthode retenue : choisir un jeu très petit, et n'ajouter au langage que ce qu'il demande. Le jeu : « attraper l'étoile », dans `exemples/jeu/attraper.holo`. On touche une étoile le plus de fois possible en trente secondes ; elle change de place chaque seconde, et chaque fois qu'on la touche.

Ce jeu demandait trois choses, et pas une de plus.

## Décision

```holo
Page(
  name: Catch,
  title: "Catch the star",
  state: State(time: 0, score: 0, star_x: 50, star_y: 50),
  children: [
    If(time, is: 0, children: [
      Button(name: Play, text: "Play"),
    ]),
    If(time, over: 0, children: [
      Text("Score: {score}. Time: {time} s"),
      Board(height: 320px, children: [
        Point(name: Star, seed: 7, x: star_x, y: star_y),
      ]),
    ]),
  ],
  rules: [
    On(Play.tap, effect: time.set(30)),
    On(Star.tap, effect: score.add(1)),
    On(Star.tap, effect: star_x.random(100)),
    On(Star.tap, effect: star_y.random(100)),
    Every(1s, effect: time.sub(1)),
    Every(1s, effect: star_x.random(100)),
    Every(1s, effect: star_y.random(100)),
  ],
)
```

1. **Le temps : `Every(1s, effect: …)`.** Une règle qui se répète à un rythme, donné en `s` ou en `ms`, de `100ms` à `3600s`. Son effet est une demande faite à l'arbitre, comme pour `On`.
2. **Le hasard : la demande `random`.** `star_x.random(100)` donne à la valeur un nombre de 0 à 100, bornes comprises.
3. **Le plateau : `Board(height: 320px, children: [...])`.** Un bloc qu'il contient se place par `x` et `y`, de 0 (à gauche, en haut) à 100 (à droite, en bas). `x` et `y` sont un nombre, ou le nom d'une valeur de la page : le bloc suit alors cette valeur quand elle change.
4. **Le hasard est rejouable.** Ce n'est pas un vrai hasard : c'est le énième nombre d'une suite fixée par le nom de la page. Les mêmes gestes aux mêmes moments redonnent la même partie (`ADR-008`).
5. **L'horloge se tait quand on ne regarde pas** : fenêtre cachée, vue points, carrefour ouvert.
6. Arrêter et relancer se fait sans mot nouveau : le temps ne descend pas sous zéro, et les conditions (`ADR-025`) montrent le jeu ou l'écran de fin selon qu'il reste du temps.

## Correction du 2026-10-04, après le premier essai de Yocthan

Yocthan a joué et n'a pas pu gagner : « on commence avec un petit handicap », et « l'étoile est ultra rapide sur PC ». Deux défauts :

- **Une horloge par rythme.** Toutes les règles à une seconde partageaient la même horloge, qui battait sans s'occuper des gestes. En appuyant sur « Play » juste avant un battement, la première seconde durait un dixième de seconde.
- **L'étoile bougeait chaque seconde**, y compris juste après avoir été touchée.

Corrigé : **chaque règle `Every` a sa propre horloge**, et **quand un geste change une valeur, l'horloge de cette valeur repart de zéro**. Après « Play », la première seconde est une vraie seconde ; une étoile qu'on vient de toucher reste deux vraies secondes à sa nouvelle place. Dans le jeu, l'étoile bouge maintenant toutes les deux secondes.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le temps | une règle `Every(1s, …)` ; un bloc `Timer(name:, every:)` qui émet un signal ; une boucle écrite par l'auteur | Une règle. Elle se range avec les autres règles et se lit comme elles. Un bloc `Timer` aurait été un bloc qu'on ne voit pas, posé parmi ceux qu'on voit. Une boucle est du code libre. |
| Le mot | `Every` ; `Each` ; `Timer` ; `Tick` | `Every`. « Toutes les secondes » : la phrase se lit. |
| Le hasard | une demande `random(n)` ; une valeur spéciale `{random}` ; une fonction | Une demande. Elle change une valeur déclarée, comme `add` ; rien d'autre ne change. |
| Les bornes du hasard | de 0 à n − 1 (l'usage des programmeurs) ; de 0 à n | De 0 à n. « Un nombre jusqu'à 100 » se comprend sans explication ; « jusqu'à 99 » est un piège connu. |
| Vrai hasard ou suite fixée | `Math.random()` ; une suite tirée d'une graine | Une suite. Tout le projet repose sur des graines : on peut rejouer, tester, et deux joueurs pourront plus tard voir la même partie. |
| Placer librement | un bloc `Board` et `x`, `y` sur ses enfants ; `position: absolute` dans un style | Un bloc. La disposition vient des blocs (`ADR-017`), et les places vont de 0 à 100 : rien ne sort du plateau. |
| Le mot du plateau | `Board` ; `Field` ; `Stage` ; `Stack` (Flutter) | `Board`, le plateau de jeu. `Field` désigne un champ de formulaire sur le web, qui arrive à l'étape 3. |

Défauts de JavaScript et de CSS évités :

- `setInterval` qu'on oublie d'arrêter, et qui tourne dans un onglet caché : ici l'horloge se tait d'elle-même.
- `Math.random()` qu'on ne peut ni rejouer ni tester.
- `position: absolute` avec des pixels, qui sort de l'écran sur un téléphone : ici, de 0 à 100, toujours dans le plateau.

## Conséquences

### Positives

- Un jeu entier, 60 lignes, sans une ligne de code. Il se joue au doigt comme à la souris.
- Le temps et le hasard passent par l'arbitre : une partie se rejoue à l'identique.

### Négatives et risques

- Une règle `Every` tourne tant que la page est ouverte, même quand le jeu est fini : elle demande, et la demande ne change rien. On ne peut pas dire « seulement pendant la partie ». (Corrigé le 2026-10-04 : `If(lives, over: 0, rules: [ … ])`, voir `ADR-028`.)
- Un geste ne peut faire qu'une demande par règle : toucher l'étoile demande trois règles.
- On ne peut pas comparer deux valeurs : pas de « meilleur score ».
- Rien ne bouge de façon continue : l'étoile saute d'une place à l'autre (avec un court glissement). Pas de vitesse, pas de trajectoire.
- `x` et `y` écrits sur un bloc qui n'est pas dans un `Board` sont ignorés en silence. C'est un défaut : il faudrait le refuser.
- L'horloge est tenue par le navigateur : entre deux appareils, les battements ne tombent pas au même instant. Le rejeu exact suppose de noter aussi les instants.

## Ce qui reste à faire

- Une condition dans une règle (`On(…, if: …)`), pour dire « seulement pendant la partie ».
- Comparer deux valeurs (un meilleur score).
- Le mouvement continu, le clavier, les rencontres entre objets : c'est l'étape 4 du planning.
- Refuser `x` et `y` hors d'un plateau.

## Critères de validation

- `exemples/jeu/attraper.holo` : « Play » lance trente secondes ; l'étoile bouge chaque seconde ; la toucher donne un point ; à zéro, l'écran de fin montre le score.
- Tests du moteur : `etat.rs` (`le_jeu_se_joue_par_des_regles_le_temps_et_le_hasard`, `le_hasard_est_rejouable_et_reste_dans_ses_bornes`), `plat.rs` (`un_plateau_place_ses_blocs_ou_l_on_veut`).

## Conditions de réexamen

- Quand Yocthan aura joué et jugé : `Every`, `random`, `Board`, `x`, `y`.
- Au deuxième jeu : s'il demande autre chose, ce qui manque se verra.
