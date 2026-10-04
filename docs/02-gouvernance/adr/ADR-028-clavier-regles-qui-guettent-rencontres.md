# ADR-028 — Le clavier (`Key`), les règles qui guettent (`When`), les rencontres (`Meet`)

- Statut : EXPÉRIMENTATION
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 ; `docs/01-holocode/COMPARATIF-CONCURRENTS.md` (planning, étape 4)
- Validation : Yocthan, le 2026-10-04 : « Tu termines l'étape 3 et après on passe à l'étape 4. Si tu peux déjà le commencer et le finir aussi. » L'écriture est une proposition de Claude ; à juger après essai.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Étape 4 du planning : des objets qui ont une place et qui bougent, le clavier, les rencontres entre objets. Même méthode qu'à l'étape 2 : un jeu très petit, et seulement les mots qu'il demande. Le jeu : `exemples/jeu/panier.holo`. Une pomme tombe ; on la rattrape avec un panier qu'on déplace au clavier ou par deux boutons ; trois pommes perdues, la partie est finie.

## Décision (à l'essai)

```holo
Page(
  name: Orchard,
  title: "Catch the apple",
  state: State(lives: 3, score: 0, basket: 50, apple_x: 50, apple_y: 0),
  children: [
    Text("Score: {score}. Lives: {lives}"),
    Board(height: 360px, children: [
      Point(name: Apple, seed: 3, x: apple_x, y: apple_y),
      Point(name: Basket, seed: 9, x: basket, y: 96),
    ]),
  ],
  rules: [
    On(Key.left, effect: basket.sub(8)),
    On(Key.right, effect: basket.add(8)),
    Every(100ms, effect: apple_y.add(3)),
    Meet(Basket, Apple, within: 9, effect: score.add(1)),
    Meet(Basket, Apple, within: 9, effect: apple_y.set(0)),
    When(apple_y, over: 99, effect: lives.sub(1)),
    When(apple_y, over: 99, effect: apple_y.set(0)),
  ],
)
```

1. **Le clavier : `Key`.** `On(Key.left, effect: …)`. `Key` est le clavier du visiteur ; ses signaux sont `left`, `right`, `up`, `down`, `space`. Seules les touches que le fichier écoute sont prises ; les autres gardent leur rôle, et rien n'est pris à un champ où l'on écrit.
2. **Ce qui bouge tout seul** ne demande aucun mot nouveau : `Every(100ms, effect: apple_y.add(3))`, et la pomme descend.
3. **Une valeur qui sert de place reste sur le plateau** : de 0 à 100. Le panier ne sort jamais, même si l'on insiste sur la flèche.
4. **Une règle qui guette : `When(apple_y, over: 99, effect: …)`.** Elle se déclenche au moment où la condition devient vraie, pas tant qu'elle le reste. Mêmes comparaisons que `If` : `is`, `not`, `over`, `under`.
5. **Une rencontre : `Meet(Basket, Apple, within: 9, effect: …)`.** Elle se déclenche au moment où deux blocs posés sur un plateau arrivent à moins de `within` l'un de l'autre (10 sans rien écrire), sur l'échelle de 0 à 100.
6. Plusieurs règles qui guettent la même chose se déclenchent ensemble : elles jugent sur la même photo de l'état, puis leurs effets s'appliquent dans l'ordre où elles sont écrites.
7. Un effet peut déclencher une autre règle qui guette ; l'arbitre s'arrête après huit tours, pour qu'un fichier mal écrit ne tourne pas sans fin.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le clavier | une source réservée `Key`, dans une règle `On` ; un bloc `Keyboard` à poser dans la page ; un réglage `key:` sur les boutons | `Key`. Une touche est un signal comme un autre : la règle s'écrit comme pour un bouton. |
| Quelles touches | les quatre flèches et l'espace ; toutes les lettres | Les flèches et l'espace. Elles suffisent à un jeu simple, et elles existent sur tous les claviers. Les lettres changent de place d'un pays à l'autre. |
| « Quand ceci arrive » | une règle `When(valeur, comparaison, effect:)` ; un réglage `if:` sur `Every` ; un « si » écrit par l'auteur | `When`. Elle se lit comme une phrase et reprend les mots de `If`. Elle guette un moment (« devient vrai »), ce qui évite qu'un effet se répète à chaque battement. |
| La rencontre | une règle `Meet(A, B, effect:)` ; comparer à la main quatre valeurs ; un moteur physique | `Meet`. Comparer `x` et `y` de deux objets demanderait de comparer deux valeurs entre elles et des « et », donc des expressions. Un moteur physique est hors de proportion. |
| La distance | des pixels ; l'échelle du plateau, de 0 à 100 | L'échelle du plateau. Elle ne dépend pas de l'écran : la même partie se joue pareil sur un téléphone. |
| Les bornes d'une place | laisser la valeur filer (et brider l'affichage) ; la borner | La borner. Sinon le panier, poussé dix fois contre le bord, met dix appuis à revenir. |

Défauts de JavaScript évités : `keydown` écouté sur toute la page, qui vole les flèches au défilement et à la saisie ; les tests de collision écrits à la main, en pixels, faux sur un autre écran ; une condition testée à chaque image, dont l'effet se répète soixante fois par seconde.

## Conséquences

### Positives

- Deux jeux entiers sans code. Le second se joue au clavier et au doigt.
- Tout passe par l'arbitre, donc tout se rejoue et se teste : la partie est vérifiée battement par battement dans `etat.rs`.

### Négatives et risques

- **Le langage grossit** : quatre sortes de règles (`On`, `Every`, `When`, `Meet`). C'est le risque annoncé dans le planning.
- Une règle ne fait qu'une demande : rattraper la pomme demande trois lignes `Meet`.
- Le mouvement va par petits sauts (dix par seconde), adoucis à l'affichage. Pas de vitesse ni de trajectoire.
- La rencontre est jugée sur des places, pas sur des formes : un grand et un petit objet se rencontrent à la même distance.
- Le clavier ne vaut que pour la page ; pas de manette, pas de glissement du doigt.
- La rencontre est testée après chaque battement : un objet très rapide peut traverser sans être vu.

## Ce qui reste à faire

- Une règle qui fait plusieurs demandes.
- Le glissement du doigt ; des objets créés en nombre (dix pommes) : il faut des listes.
- Les formes, les images et le son dans les mondes : étape 6.

## Critères de validation

- `exemples/jeu/panier.holo` : les flèches déplacent le panier ; la pomme rattrapée donne un point ; trois pommes perdues finissent la partie.
- Tests du moteur : `etat.rs` (`le_clavier_ce_qui_tombe_et_les_rencontres`).

## Conditions de réexamen

- Quand Yocthan aura joué et jugé : `Key`, `When`, `Meet`, `within`.
- Si un troisième jeu demande encore une nouvelle sorte de règle : il faudra alors chercher une forme plus générale.
