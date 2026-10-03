# ADR-016 — Le vocabulaire du langage est en anglais, et un mot connu garde son sens

- Statut : ACCEPTÉ
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013, revue Codex du 2026-10-03 (`proposals/GPT5.6/revue-2026-10-03/`), journal du 2026-10-04
- Validation : décidé par Yocthan le 2026-10-04, en discussion avec Claude.
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

## Question laissée ouverte

Le nom du bloc `Point`. Pour un programmeur, `Point` est d'abord une coordonnée `Point(x, y)`, ce qui contredit la règle 2. Candidats : garder `Point` (le mot de la vision, « un point lumineux »), `Sphere`, ou `Orb`. Yocthan n'a pas tranché ; `Point` reste en attendant.

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
