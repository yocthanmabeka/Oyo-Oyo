# ADR-020 — `Text` est le texte de base ; `P` et `H1` à `H3` sont un `Text` avec un rôle ; une seule écriture par mot

- Statut : ACCEPTÉ
- Date : 2026-10-03
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-03
- Validation : décidé par Yocthan le 2026-10-03, après comparaison demandée à Claude.
- Projets affectés : HoloCode, HoloCompiler, suite de conformité

## Contexte

Yocthan veut écrire le texte « exactement comme sur HTML et CSS, sans les défauts » : on écrit `P` ou `H1`, puis on stylise facilement. Il a relevé lui-même le problème : `P` et `H1` servent sur un site web, mais dans un terminal, une application ou un monde, c'est `Text` qui a du sens. Il a demandé une comparaison avant de choisir.

| Critère | `Text` | `P`, `H1`… |
|---|---|---|
| Marche partout (page, terminal, monde, bouton, jeu) | Oui | Non |
| Se comprend sans avoir appris HTML | Oui | Non |
| Dit ce que le texte est (titre, paragraphe) | Non | Oui |
| Lecteurs d'écran, sommaire, moteurs de recherche, IA | Rien à exploiter | Tout est donné |
| La vue en profondeur sait quoi mettre en grand | Non | Oui |

Aucun ne remplace l'autre. `Text` seul répète le défaut de Flutter et des pages remplies de `div` : la machine ne sait plus ce qui est un titre. `P` et `H1` seuls ne couvrent ni le terminal ni les mondes.

## Décision

1. **`Text` est le bloc de base : du texte sans rôle.** Une étiquette, une ligne d'état, un message dans un terminal.
2. **`P`, `H1`, `H2` et `H3` sont un `Text` avec un rôle** : paragraphe, titre de niveau 1, 2 ou 3. Ce ne sont pas des doublons : chacun dit quelque chose de différent. Un style posé sur `Text` les touche tous ; un style posé sur `H1` ne touche que les grands titres.
3. **Une phrase nue entre guillemets, dans `children`, vaut un `P`** (précise `ADR-019`, qui disait « vaut un `Text` »).
4. **Le numéro d'un titre dit sa place dans le plan, jamais sa taille.** La taille se règle par le style. Le vérificateur refuse un titre qui saute un niveau (`H3` après `H1`) et un premier titre qui n'est pas `H1`. Chaque `Page` et chaque `World` a son propre plan. Remonter (`H3` puis `H1`) est libre.
5. **Les titres s'arrêtent à `H3`.** On en ajoutera si un vrai besoin apparaît.
6. **Une seule écriture par mot.** Tout ce qui ouvre une parenthèse est un bloc et commence par une majuscule (`Text`, `P`, `H1`) ; tout réglage est en minuscules (`text:`, `title:`). `h1(...)` est refusé, et le moteur répond « écris `H1` ».

```holo
Page(
  title: "Download",
  children: [
    H1("Download"),
    Text("Loading…"),
    "A sentence alone is a paragraph.",
    P("So is this one."),
  ],
)
```

## Correction du 2026-10-06 : jusqu'à `H6`

Le point 5 (« les titres s'arrêtent à `H3` ») est levé, après l'avis des humains, de Gemini et de ChatGPT : les longs documents (techniques, juridiques) ont besoin de plus de niveaux, et 71,6 % des utilisateurs de lecteurs d'écran naviguent d'abord par les titres. Les titres vont de `H1` à `H6`, avec la même règle : le numéro dit la place dans le plan, jamais la taille (`ADR-036`).

## Défauts de HTML, CSS et JavaScript évités

- HTML : choisir `h3` « parce que c'est plus petit » ; sauter des niveaux ; six niveaux dont trois presque jamais utilisés ; tout écrire en `div`, sans dire ce que les choses sont.
- HTML et JavaScript : deux écritures pour le même mot (`<P>` et `<p>` acceptés d'un côté, `onClick` et `onclick` distingués de l'autre).
- CSS : aucun ajout ici ; la place des styles reste ouverte dans `ADR-017`.

## Conséquences

### Positives

- Le même mot `Text` sert dans une page, un terminal et un monde.
- La structure d'une page (son plan) est connue de la machine sans effort de l'auteur.

### Négatives et risques

- Cinq blocs de texte au lieu d'un : il faut expliquer la différence entre `Text` et `P`.
- Refuser un saut de niveau est plus strict que HTML ; cela peut gêner quand on assemblera des morceaux de page importés.
- La majuscule est une convention, pas une nécessité technique : le moteur reconnaît un bloc à sa parenthèse. Elle sert le lecteur (`Text(...)` le bloc, `text:` le réglage).

## Ce qui reste à instruire

- Le Markdown écrit dans un texte (`# Titre`) doit donner les mêmes rôles que les blocs (`H1`). Aujourd'hui le moteur ne lit pas le Markdown : la règle des niveaux ne s'applique qu'aux blocs.
- Marquer un bloc d'un style nommé (`P.card(...)`) dépend de la place des styles (`ADR-017`) ; le lecteur ne l'accepte pas encore.
- Aucun de ces blocs n'est encore affiché : le moteur les vérifie seulement.

## Critères de validation

- Suite de conformité : `01-page-simple`, `05-texte-sans-role` acceptés ; `E09-bloc-en-minuscules`, `E10-titre-saute-un-niveau`, `E11-titre-trop-profond` refusés à la bonne ligne.
- Tests du moteur (`moteur/src/blocs.rs`) qui lisent ces cas directement.

## Conditions de réexamen

- Si les débutants confondent `Text` et `P`.
- Si le refus des sauts de niveau gêne l'assemblage de pages importées : le remplacer par un avertissement.
- Si un vrai besoin de `H4` apparaît.
