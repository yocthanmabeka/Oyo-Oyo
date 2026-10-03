# ADR-019 — Le texte s'écrit sans artifice ; un fichier `.md` ne s'importe que pour un long texte

- Statut : ACCEPTÉ pour le principe ; l'écriture exacte est une proposition de Claude
- Date : 2026-10-03
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013, revue Codex du 2026-10-03 (`proposals/GPT5.6/revue-2026-10-03/`), journal du 2026-10-03
- Validation : décidé par Yocthan le 2026-10-03, en discussion avec Claude.
- Projets affectés : HoloCode, HoloCompiler

## Contexte

Yocthan : écrire un texte doit être aussi simple qu'écrire « Hello World ». Un paragraphe est un paragraphe. Et importer un fichier `.md` pour écrire « bonjour » n'a pas de sens.

## Décision

- **Le texte s'écrit sans artifice.** Écriture proposée : dans une liste `children`, une phrase entre guillemets est un paragraphe à elle seule ; elle vaut un `P` (précisé par `ADR-020` : `Text` est le texte sans rôle, `P` un paragraphe).

```holo
Page(
  title: "My shop",
  children: [
    "Hello World",
    "A second paragraph, with **bold** if I want.",
    Button(name: Open, text: "Enter"),
  ],
)
```

  `P(...)` ne sert que lorsqu'on veut lui ajouter quelque chose, un style par exemple. Le Markdown reste la façon de mettre en forme le texte (`ADR-009`).
- **Un fichier `.md` s'importe seulement dans des cas précis**, un long texte qui rendrait le `.holo` illisible : `Text(import "article.md")`. Ce sera fait après le texte écrit sur place.

## Conséquences

### Positives

- La première page d'un débutant tient en trois lignes.

### Négatives et risques

- Une chaîne nue est un paragraphe dans `children`, mais reste une simple valeur ailleurs (`title: "..."`) : la règle doit être dite clairement.

## Critères de validation

- Le cas `01-page-simple` de la suite de conformité, qui contient un paragraphe nu.

## Conditions de réexamen

- Si la double écriture (chaîne nue et `Text`) trouble les débutants.
