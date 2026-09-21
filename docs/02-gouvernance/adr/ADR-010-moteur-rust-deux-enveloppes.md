# ADR-010 — Un moteur écrit en Rust, sous les navigateurs actuels puis dans un navigateur propre

- Statut : ACCEPTÉ
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013, HC-007
- Projets affectés : HoloRuntime, HoloEngine, HoloCode-Core
- Validation : décidé par Yocthan le 2026-09-21. La fusion de la pull request qui introduit cette fiche vaut confirmation.

## Contexte

Un navigateur ne comprend que HTML, CSS, JavaScript et WebAssembly ; il ne comprendra jamais HoloCode directement. Yocthan veut que son langage tourne dans les navigateurs actuels, et aussi dans un futur navigateur propre au projet, parce qu'on ne peut pas faire un métavers en restant seulement dans les navigateurs. Les prototypes des PR n° 1 à n° 3 sont en Python, qui ne tourne pas sur un téléphone avec un poids et une vitesse acceptables.

## Décision

On écrit une seule fois un **moteur en Rust**, qui lit les fichiers `.holo` et dessine le résultat. Il a deux enveloppes :

1. **dans les navigateurs actuels** : compilé en WebAssembly, il dessine avec WebGPU, ou WebGL en secours ;
2. **dans un navigateur propre au projet** : le même moteur compilé en application native, qui ouvre directement une adresse vers un fichier `.holo`, sans aucun HTML.

Les prototypes Python restent des brouillons des règles du langage (grammaire, vérificateur, lois, journal) ; leurs idées se transposent.

## Alternatives étudiées

| Critère | Zig | Rust | C++ |
|---|---|---|---|
| Poids du moteur en WebAssembly | Le plus petit | Petit avec des réglages | Moyen à gros |
| Bugs de mémoire | Possibles | Refusés par le compilateur | Possibles et silencieux |
| Dessin web et natif avec le même code | Peu de bibliothèques | `wgpu`, mûr | Beaucoup, lourd à assembler |
| Code écrit par des IA | Peu fiable, le langage change | Fiable | Fiable mais dangereux |
| Stabilité | Pas de version 1.0 | Stable depuis 2015 | Stable |

- **Zig** écarté par Yocthan : excellent en poids, mais pas stable, et on ne sait pas quand il le sera.
- **C++** écarté : le plus piégeux pour une personne seule.
- **Écrire le moteur en HoloCode** : impossible, un langage ne peut pas faire tourner son propre moteur avant qu'il existe.
- **TypeScript** : ne donne pas le navigateur natif.

Le langage du moteur ne compresse pas les mondes : leur petitesse vient du format (graines, formes par formules, version binaire compacte). Il joue sur le poids du moteur à télécharger, sa vitesse et sa mémoire.

## Conséquences

### Positives

- Une famille entière de plantages, les erreurs de mémoire, est refusée avant l'exécution : important quand ce sont des IA qui écrivent le code.
- Un seul code pour WebGPU, WebGL et le natif.

### Négatives et risques

- Moteur un peu plus lourd qu'en Zig ; cible : moins de 1 à 2 Mo compressé.
- Compilation lente, langage exigeant à lire.
- Limites des navigateurs actuels, qui justifient la seconde enveloppe : un onglet de téléphone est tué bien avant 1 Go (souvent 300 à 500 Mo) ; WebGPU absent de certains téléphones ; onglet ralenti puis fermé en arrière-plan ; stockage local effaçable ; pas d'accès direct au réseau ; gestes réservés par le navigateur (pincer, glisser) ; il reste une page HTML d'une dizaine de lignes comme porte d'entrée, générée automatiquement.

## Critères de validation

- Le moteur compilé en WebAssembly affiche le Big Bang sur un vrai téléphone ; on mesure son poids, la fluidité et la mémoire.

## Conditions de réexamen

- Si le moteur Rust ne tient pas sous 2 Mo compressé malgré les réglages.
- À la sortie de Zig 1.0.
