# ADR-017 — La forme : un thème, des styles nommés à point, des réglages par bloc ; les couleurs d'un point

- Statut : ACCEPTÉ pour le principe ; la place des styles dans le fichier reste à décider
- Date : 2026-10-03
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013, revue Codex du 2026-10-03 (`proposals/GPT5.6/revue-2026-10-03/`), journal du 2026-10-03
- Validation : décidé par Yocthan le 2026-10-03, en discussion avec Claude.
- Projets affectés : HoloCode, HoloCompiler, HoloEngine

## Contexte

Le CSS réglait la forme de l'ancien web. Yocthan l'appréciait pour une raison précise : on définit un style une fois, on lui donne un nom avec un point (`.card`), et on l'applique partout. Par ailleurs, les couleurs du Big Bang ne sont écrites nulle part : le moteur les tire de la graine.

## Décision

**Trois niveaux pour la forme**, du plus général au plus précis ; le plus précis l'emporte :

1. le `Theme` règle toute la page ;
2. un `Style(.card, ...)` est un style nommé, avec le point du CSS ;
3. un paramètre écrit sur le bloc corrige un seul élément.

```holo
Page(
  theme: Theme(background: night, accent: amber),
  styles: [ Style(.card, background: "#1a1a2e", radius: 12) ],
  children: [
    Button(name: Open, text: "Enter", style: .card, background: "#245C45"),
  ],
)
```

Ni sélecteurs compliqués, ni cascade cachée.

**Trois possibilités pour la couleur d'un point :**

```holo
Point(name: A, seed: 42)                                    // la graine décide
Point(name: B, seed: 42, color: "#E9B44C")                  // l'auteur impose la couleur de ce point
Point(name: C, seed: 42, palette: ["#E9B44C", "#245C45"])   // l'auteur impose les couleurs de ses enfants
```

## Question laissée ouverte

Où ranger les styles. Trois écritures ont été montrées à Yocthan, qui veut y réfléchir : dans la page (`styles: [...]`) ; dans un fichier à part importé (`import "styles.holo"`) ; à la manière exacte du CSS, avec des accolades. Claude recommande les deux premières, qui gardent une seule écriture.

## Conséquences

### Positives

- La priorité se lit d'un coup d'œil ; un thème évite vingt corrections répétées.

### Négatives et risques

- Rien de tout cela n'est encore lu par le moteur. La notation `.card` demande un ajout à la grammaire.
- Une couleur tirée d'une graine peut donner un texte illisible : les textes et les boutons prendront leurs couleurs du thème, pas du hasard (remarque de Codex).

## Critères de validation

- La boutique de démonstration s'écrit avec un thème, deux styles nommés et une exception locale, et s'affiche comme prévu dans les deux vues.

## Conditions de réexamen

- Si les styles nommés ne suffisent pas à mettre en page un vrai site.
