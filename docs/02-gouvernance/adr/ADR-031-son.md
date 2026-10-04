# ADR-031 — Le son : `Sound` et la capacité `play`

- Statut : EXPÉRIMENTATION
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 ; `docs/01-holocode/COMPARATIF-CONCURRENTS.md` (planning, étape 6)
- Validation : Yocthan, le 2026-10-04 : « Oui, fais le 5, le 6 et le 7. » L'écriture est une proposition de Claude ; à juger après essai.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Étape 6 du planning : dans les mondes, des formes, des images, du son, puis des modèles 3D. Le son est le premier morceau : c'est le plus petit, et un jeu sans son n'en est pas tout à fait un. Les images, elles, se placent déjà sur un plateau (`Image(..., x:, y:)`), sans rien ajouter.

## Décision (à l'essai)

```holo
Page(
  title: "A sound",
  state: State(count: 0),
  children: [
    Sound(name: Ding, source: "ding.wav"),
    Button(name: Ring, text: "Ring"),
  ],
  rules: [
    On(Ring.tap, effect: [count.add(1), Ding.play]),
    When(count, is: 5, effect: Ding.play),
  ],
)
```

1. **`Sound(name: Ding, source: "ding.wav")`** : un son. Il a un nom et un fichier rangé à côté (`.wav`, `.mp3` ou `.ogg`). Il ne se voit pas.
2. **`Ding.play`** : une capacité, comme `enter` pour un point. Elle s'écrit dans l'effet d'une règle, seule ou dans une liste, à côté des demandes.
3. Les trois sortes de règles peuvent faire entendre un son : `On`, `Every`, `When`.
4. Une règle de temps ou une règle qui guette ne peut demander **que** cela en dehors des demandes : `Workshop.enter` y est refusé, parce qu'on n'emmène pas le visiteur ailleurs sans qu'il ait rien touché.
5. Un navigateur ne joue un son qu'après un premier geste du visiteur. Avant, il refuse ; la page continue sans le son.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Comment jouer | une capacité `Ding.play` dans une règle ; un réglage `sound:` sur chaque bouton ; une demande comme `add` | Une capacité. Elle s'écrit là où s'écrivent déjà `enter` et `leave`, et marche avec les trois sortes de règles. Un réglage sur le bouton ne servirait pas à une rencontre ou à une fin de partie. |
| Le mot | `Sound` ; `Audio` (HTML) | `Sound`. `audio` en HTML est un lecteur, avec ses boutons ; ici c'est un bruit qu'une règle déclenche. Un lecteur de musique sera autre chose. |
| D'où vient le fichier | à côté seulement ; n'importe quelle adresse | À côté, comme les images. |
| Boucle, volume, arrêt | les offrir ; rien | Rien pour l'instant. Aucun des deux jeux n'en a besoin. |

Défauts du web évités : `autoplay` et les sons qui partent à l'ouverture d'une page (le navigateur les bloque, et le langage n'offre pas de quoi essayer) ; un son joué par du code glissé dans un bouton.

## Conséquences

### Positives

- Les deux jeux ont leurs sons : un quand on attrape, un quand on perd une pomme.
- Les sons demandés par une règle de temps ou une règle qui guette passent par l'arbitre : une partie rejouée fait entendre les mêmes sons aux mêmes moments.

### Négatives et risques

- Pas de musique : ni boucle, ni volume, ni arrêt.
- Le son n'est pas placé dans l'espace : il ne vient ni de la gauche ni de loin. Gemini l'avait demandé pour un métavers ; ce n'est pas fait.
- Le son n'est pas décrit pour qui n'entend pas : rien n'oblige l'auteur à doubler un son d'un signe visible.
- `weight` est accepté mais, comme pour les images, n'est pas comparé au poids réel du fichier.

## Ce qui reste à faire dans l'étape 6

- Des formes simples dans les mondes ; des images dans la vue en profondeur ; des modèles 3D. C'est le gros de l'étape : il faut étendre le moteur de dessin, qui ne sait aujourd'hui dessiner que des points.

## Critères de validation

- `exemples/lecons/28-son.holo` : un appui, un son ; au cinquième, un son de plus.
- `exemples/jeu/panier.holo` : un son quand la pomme est rattrapée, un autre quand elle est perdue.
- Tests du moteur : `etat.rs` (`un_son_se_joue_par_une_regle`).

## Conditions de réexamen

- Quand Yocthan aura écouté et jugé : `Sound`, `play`.
- Au premier besoin de musique.
