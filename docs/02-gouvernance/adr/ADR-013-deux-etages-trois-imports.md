# ADR-013 — Deux étages et trois sortes d'import

- Statut : PROPOSITION
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013, HC-006
- Projets affectés : HoloCode, HoloCode-Core, HoloRuntime
- Proposé par : Claude. Non validé.

## Contexte

Yocthan a demandé comment écrire en HoloCode un programme d'IA, un programme système ou un jeu vidéo. HoloCode tire ses garanties de ses limites : pas de boucle libre, budget vérifié, état protégé. Un système d'exploitation ou l'entraînement d'un réseau de neurones ont besoin de boucles libres et d'un accès direct à la mémoire. Un même étage ne peut pas être à la fois totalement sûr et totalement libre.

Tous les grands systèmes ont deux étages : Python et le C de NumPy et PyTorch ; Luau et le C++ de Roblox ; C# et le C++ d'Unity ; GDScript et le C++ de Godot ; Verse et Unreal.

## Décision proposée

**Deux étages.**

- Étage 1, **HoloCode** : pages, mondes, règles de jeu, comportements de personnages, utilisation d'une IA. Sûr, petit, réapprenable.
- Étage 2, **modules** : rendu, physique lourde, réseaux de neurones, accès au système. Écrits en Rust, ou dans tout langage qui se compile en WebAssembly (C, C++, Zig, Go). Un module est enfermé : le moteur plafonne sa mémoire, peut l'arrêter s'il prend trop de temps, et ne lui ouvre que ce qui a été déclaré.

Un module doit **se présenter en holoscénique** : quel que soit son intérieur, il expose des archétypes, des lois et des capacités, jamais du Rust. Le moteur lui-même ne s'importe pas ; ses blocs de base sont toujours là.

**Trois sortes d'import**, avec trois mots différents pour que le risque se lise en haut du fichier :

```holo
import "boutons.holo"          // du HoloCode pur : toutes les garanties
module "physique"              // une boîte fermée : risque contrôlé
pont js "carte-interactive"    // vieux web : aucune garantie (ADR-012)
```

On n'importe jamais de code source étranger. La première version contient `import` et `pont` ; les modules viennent ensuite, quand le moteur existe.

| Ce qu'on veut écrire | Où | Verdict |
|---|---|---|
| Page, site, monde | HoloCode | Oui, c'est le cœur |
| Jeu vidéo | HoloCode, plus des modules pour le lourd | Oui pour la majorité des jeux |
| Personnages et comportements | HoloCode (lois et phénomènes) | Oui |
| Utiliser une IA | HoloCode, bloc `IA` | Oui |
| Entraîner une IA, calcul lourd | Module | Pas à l'étage 1 |
| Programme système, pilote, compilateur | Rust aujourd'hui | Pas à l'étage 1 |

## Alternatives étudiées

- **Un seul langage pour tout** : il faudrait y remettre les boucles libres et la mémoire brute, et perdre toutes les garanties.
- **Un langage système holoscénique** pour l'étage 2 : gérer des octets ne ressemble pas à un monde avec des entités et des lois ; le « tout est objet » de Java a montré le coût d'un paradigme appliqué partout. À reconsidérer seulement une fois l'étage 1 vivant.

## Conséquences

### Positives

- Les autres langages entrent par un format universel, et les modules marchent aussi dans le navigateur propre au projet, contrairement aux ponts.

### Négatives et risques

- La boîte fermée (plafond mémoire, limite de temps, droits déclarés) est un vrai travail dans le moteur.
- Pour écrire un jeu, il manque encore à HoloCode : le calcul, les capacités avec paramètres, la création et la destruction d'entités, les signaux du joueur, le mouvement continu, le son.

## Critères de validation

- Un module de démonstration, écrit en Rust, est utilisé depuis un fichier `.holo` sans que l'auteur voie autre chose que des blocs ; le moteur l'arrête s'il dépasse sa mémoire ou son temps.

## Conditions de réexamen

- Si la limite de temps d'un module ne peut pas être imposée dans les navigateurs actuels.
