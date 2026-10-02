# ADR-009 — Format `.holo` : des blocs nommés par leur sens, le texte en Markdown dans les blocs

- Statut : ACCEPTÉ
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013
- Projets affectés : HoloCode, HoloCompiler
- Validation : décidé par Yocthan le 2026-09-21. La fusion de la pull request qui introduit cette fiche vaut confirmation.

## Contexte

Yocthan vient de Flutter et aime son écriture en blocs imbriqués. Il trouve le HTML pénible, n'aime pas JavaScript, et constate qu'il oublie vite l'informatique après une pause. Il avait d'abord imaginé un fichier `.md`.

## Décision

- L'extension est **`.holo`**. L'extension `.md` existe déjà et serait trompeuse pour un fichier qui contient autre chose que du Markdown.
- Le squelette du fichier est fait de **blocs imbriqués, à la manière de Flutter**, avec des paramètres nommés.
- Les blocs sont **nommés par leur sens** : `Page`, `Texte`, `Bouton`, `Image`, `Liste`, `Point`, `Monde`. Pas de `div`.
- **Le texte va dans les blocs** : partout où un bloc attend du texte, on l'écrit en Markdown. Ce n'est pas un document Markdown dans lequel on insère des blocs.
- L'auteur n'écrit et ne voit jamais de HTML, de CSS ni de JavaScript.

Règle de conception associée : on doit pouvoir réapprendre le langage en une heure après un mois d'absence. Peu de mots-clés, une seule façon de faire chaque chose, des messages d'erreur qui disent quoi corriger.

## Esquisse non normative

```holo
Page(
  titre: "Ma boutique",
  contenu: [
    Texte("""
      # Bienvenue
      Voici **mes créations**. Touchez un point pour y entrer.
    """),
    Point(
      nom: Atelier,
      graine: 42,
      interieur: Monde(contenu: [ Texte("## L'atelier") ]),
    ),
  ],
)
```

Cette syntaxe illustre l'intention ; ce n'est pas une grammaire officielle.

## Alternatives étudiées

- Un fichier `.md` pur : excellent pour le texte, incapable de décrire une structure ou un comportement.
- Des blocs dans le texte, à la manière de MDX : agréable pour un long article, mais cela fait deux façons d'écrire.
- Des `div` : c'est ce qui a rendu le HTML pénible, des boîtes sans signification empilées à l'infini.

## Conséquences

### Positives

- La forme du langage épouse celle de l'univers : un point contient des points, comme un bloc contient des blocs.
- Familier pour quiconque connaît Flutter, SwiftUI ou Compose.

### Négatives et risques

- Un fichier `.holo` ressemble à du code orienté objet ; la différence est dans ce qu'on a le droit de mettre dans un bloc (voir `ADR-015`).
- Les extensions `.holo` des prototypes des PR n° 1 et n° 2 utilisent une autre syntaxe ; elles devront converger.

## Critères de validation

- Dix pages et dix mondes d'exemple s'écrivent sans jamais recourir à du HTML, du CSS ou du JavaScript.
- Yocthan relit un fichier `.holo` après un mois de pause et le comprend en moins d'une heure.

## Conditions de réexamen

- Si les pages à fort contenu textuel deviennent pénibles à écrire sous cette forme.
