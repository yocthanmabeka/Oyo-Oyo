# HC-011 — Cadrage de la vision : Big Bang, 1 Go, nouveau langage

## Métadonnées

- Date : 2026-09-20
- Statut : synthétisée
- Plateforme : Claude (Claude Code dans VS Code)
- URL : aucune — une session Claude Code est locale et n'a pas de lien de partage. Transcription : [transcriptions/HC-011.md](transcriptions/HC-011.md)
- Parents : HC-010
- Enfants : HC-012

## Problème étudié

Yocthan expose à Claude sa vision du métavers et répond à quatre questions de cadrage : l'accès à Genie 3, le sens du budget de 1 Go, ce que signifie « modifier les langages », et le résultat attendu du premier sprint.

## Conclusions

Ce qui vient de Yocthan :

- **Vision.** Remodeler notre façon d'utiliser le web sous la forme d'un monde explorable, par sprints de 24 heures, pour une technologie proposable au plus tard en 2030.
- **Big Bang.** Tout part d'un petit point lumineux en 3D qui se morcelle en d'autres points. À l'intérieur de chaque point se trouve un monde, ou une multitude de mondes. La sphère est aussi le premier personnage et peut prendre n'importe quelle apparence.
- **Budget de 1 Go.** Le premier test doit tenir dans 1 Go au maximum. Ce n'est pas la limite définitive du métavers. Le but est qu'un monde tourne de façon stable sur n'importe quel téléphone actuel ; les téléphones anciens seront traités plus tard.
- **Genie 3.** Yocthan n'y a pas accès. Il sert d'inspiration seulement ; le moteur est à programmer.
- **Langage.** Yocthan veut un nouveau langage entièrement refait, pas une couche posée sur un langage existant. Il souhaite combiner les philosophies de C, Rust et Zig avec une forme d'écriture proche de Dart/Flutter : verbeux mais simple, fait pour créer des mondes. Les ponts vers d'autres langages viendront plus tard. Il reproche au C sa verbosité et ses pièges ; il n'a jamais essayé Zig.
- **Équipe.** Claude est le programmeur principal ; ChatGPT et Gemini viennent en appui. Des croquis de Yocthan préciseront la vision.
- **Sprint 1.** Le langage ne se crée pas pendant le premier sprint.

Ce que Claude a proposé, sans validation à ce jour (`EXPLORATION`) :

- Générer chaque monde à partir d'une graine de quelques octets plutôt que de le stocker : la même graine redonne le même monde, et le budget de 1 Go sert alors aux créations humaines.
- Pour les téléphones anciens, garder le même monde et calculer moins de détails.
- Une esquisse de syntaxe en arbre imbriqué, à la manière des widgets de Flutter (`Univers( origine: Point( … interieur: Monde( … ) ) )`).
- Pour la démonstration du Big Bang : navigateur, TypeScript et WebGL, le langage étant conçu à part.

## Désaccords et limites

- Yocthan pensait que C, Rust et Zig tiennent mal dans de petits espaces de stockage. Claude a répondu que ce sont au contraire les langages qui produisent les programmes les plus légers : le poids d'un monde vient de ses données, pas du langage.
- Claude a objecté que faire mieux que C, Rust et Zig comme langage généraliste demande des années à des équipes entières, et a proposé un langage spécialisé dans les mondes. Yocthan n'a pas renoncé à son ambition. Le point reste ouvert et il est repris dans HC-012.
- La session s'arrête sur deux questions de Claude restées sans réponse (voir plus bas).

## Décisions produites ou affectées

- Aucune ADR produite.
- `ADR-005` (matériel existant) : cette discussion lui donne des chiffres — 1 Go pour le premier test, n'importe quel téléphone actuel — qui ne figurent pas encore dans le registre.

## Projets affectés

- Holoverse, HoloCode.

## Documents mis à jour

- Aucun. La [vision](../00-vision/VISION.md) ne mentionne pas encore le Big Bang, la sphère, le budget de 1 Go, la cible téléphone ni la forme d'écriture à la Dart/Flutter.

## Questions ouvertes

- Le premier sprint est-il bien « le Big Bang d'abord, avec une technologie existante, et le langage conçu à part » ?
- Navigateur, TypeScript et WebGL conviennent-ils pour cette première démonstration ?
- Le croquis de Yocthan reste à fournir.
