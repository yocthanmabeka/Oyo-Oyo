# ADR-016 — Le vocabulaire du langage est en anglais, et un mot connu garde son sens

- Statut : ACCEPTÉ
- Date : 2026-10-03
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013, revue Codex du 2026-10-03 (`proposals/GPT5.6/revue-2026-10-03/`), journal du 2026-10-03
- Validation : décidé par Yocthan le 2026-10-03, en discussion avec Claude.
- Projets affectés : HoloCode, HoloCompiler, suite de conformité

## Contexte

Les premiers fichiers `.holo` mélangeaient le français et l'anglais (`Page`, `Texte`, `graine`, `import`). Codex recommandait un vocabulaire français d'abord. Yocthan a tranché autrement : la plupart des programmeurs écrivent en anglais. En relisant la première traduction, il a relevé que `split`, proposé pour « morceler », désigne ailleurs le découpage d'un texte.

## Décision

1. **Les mots du langage sont en anglais.** Les commentaires et les noms choisis par l'auteur restent libres, dans la langue qu'il veut.
2. **Un mot que les programmeurs connaissent déjà garde le sens qu'ils connaissent.** Sinon, on en choisit un autre.

| Avant | Maintenant | Remarque |
|---|---|---|
| `Texte`, `Bouton`, `Monde`, `Liste` | `Text`, `Button`, `World`, `List` | |
| `nom`, `titre`, `graine` | `name`, `title`, `seed` | `seed` a déjà ce sens partout |
| `lumiere` | `brightness` | `light` désigne une source de lumière en 3D |
| `morceler` | `fragments` | `split` découpe un texte ailleurs ; un nom plutôt qu'une action |
| `contenu` | `children` | le mot de Flutter, pour la même chose |
| `interieur` | `inside` | |
| `phenomenes`, `Quand`, `effet` | `rules`, `On`, `effect` | `When` est un choix entre plusieurs cas en Kotlin ; `On` rappelle `onTap` |
| `touche`, `entrer`, `sortir` | `tap`, `enter`, `leave` | `exit` quitte un programme ailleurs |
| `pont js` | `bridge js` | |
| `o`, `Ko`, `Mo`, `Go` | `B`, `KB`, `MB`, `GB` | toujours décimales : 1 KB = 1 000 octets |

`Page`, `Point`, `Image`, `Theme`, `Style`, `budget`, `import`, `module`, `auto` ne changent pas.

Le moteur refuse les anciens mots français et indique le mot à écrire : « le paramètre « graine » s'écrit « seed » ».

## Le nom du bloc `Point` : il reste, et voici ce qu'il veut dire

Question soulevée par Claude : pour un programmeur, `Point` est d'abord une coordonnée `Point(x, y)`. Candidats examinés : `Point`, `Sphere`, `Orb`, puis `Pixel`.

Yocthan a expliqué d'où vient le mot : **sa réflexion part du pixel.** Sur un écran, une image est faite de pixels ; dans l'Holoverse, un monde est fait de points. Un point est la plus petite chose que l'on voit, et quand on zoome dessus, il se divise en mondes. Prendre une image ou une vidéo, en découper un petit carré : c'est cela, un point. Il a laissé Claude trancher selon ce que le mot désigne en 3D.

**Décision : le bloc s'appelle `Point`.**

- `Pixel` désigne déjà l'élément d'un écran, et `px` est l'unité des tailles dans les styles : dans un langage qui met aussi des pages en forme, le mot créerait une vraie confusion.
- `Voxel` est le mot de la 3D pour « pixel en volume », mais il désigne un cube dans une grille régulière, ce que le bloc n'est pas.
- `Sphere` ne décrit qu'une forme ; `Orb` est peu connu.
- Dans Blender et les logiciels 3D, un point est une position sans épaisseur (un sommet, un élément de nuage de points) : c'est le sens le plus proche de l'idée, une chose minuscule qui grandit quand on s'en approche.

Définition retenue : **un `Point` est le pixel de l'Holoverse : la plus petite unité visible, qui révèle un monde quand on zoome dessus.**

Conséquence technique à instruire : la « limite de perception » peut se mesurer en pixels d'écran. Un point ne développe son monde que lorsqu'il occupe assez de pixels pour qu'on y voie quelque chose ; en dessous, il reste un point de lumière. Aujourd'hui le moteur utilise des seuils de zoom fixes ; les remplacer par une taille à l'écran est proposé pour un prochain sprint.

## Alternatives étudiées

- Le français d'abord, puis un profil anglais par table d'alias (recommandation de Codex) : écarté par Yocthan.
- Les deux langues acceptées dans un même fichier : multiplie les erreurs, les exemples et les diagnostics.

## Conséquences

### Positives

- Familier pour les programmeurs et pour les IA qui écriront du `.holo`.

### Négatives et risques

- Un créateur francophone qui n'a jamais programmé doit apprendre une vingtaine de mots anglais. À mesurer.
- Les prototypes des `proposals/` gardent leurs anciennes syntaxes ; ce sont des brouillons.

## Critères de validation

- Le moteur et la suite de conformité n'emploient que ce vocabulaire ; c'est le cas (cas `E08-ancien-vocabulaire`).

## Conditions de réexamen

- Si un essai avec des créateurs non programmeurs montre que l'anglais les arrête.
