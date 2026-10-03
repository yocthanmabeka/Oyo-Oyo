# HC-012 — Revue critique du paradigme holoscénique par Claude

## Métadonnées

- Date : 2026-09-21
- Statut : synthétisée pour la revue du 2026-09-21 ; la transcription couvre toute la session, jusqu'au 2026-10-03
- Plateforme : Claude (Claude Code dans VS Code)
- URL : aucune — une session Claude Code est locale et n'a pas de lien de partage. Transcription : [transcriptions/HC-012.md](transcriptions/HC-012.md). Elle contient aussi la suite de la session : les décisions du 2026-09-21 (`HC-013`), la comparaison des trois propositions, la correction de Gemini, et le sprint Big Bang du 2026-10-03 (`moteur/`).
- Parents : HC-002, HC-003, HC-009, HC-011
- Enfants : HC-013

## Problème étudié

Après lecture complète du dépôt dans l'ordre imposé par le [protocole](../02-gouvernance/PROTOCOLE-IA.md), Yocthan demande à Claude si le paradigme holoscénique est meilleur et plus utile que la POO, puis exige une réponse « 100 % franche » : y a-t-il une différence majeure et un apport propre ?

## Conclusions

Tout ce qui suit est l'avis de Claude. Statut : `EXPLORATION`. Rien n'a été validé par Yocthan.

**Verdict.** L'approche diffère réellement de la POO, mais cette différence vient de l'ECS, des systèmes de règles et de la programmation relationnelle, qui existent depuis longtemps. À ce jour, le paradigme holoscénique n'a aucun apport propre démontré, et le mot « paradigme » est trop grand pour ce qui existe.

**Chaque concept a un précédent.**

| Concept holoscénique | Précédent |
|---|---|
| Entité, archétype | ECS ; « archetype » y est déjà un terme technique (Unity DOTS, Bevy, Flecs) |
| Relation interrogeable | Datalog, bases relationnelles, relations de Flecs |
| Phénomène (`when … effect …`) | Règles de production et règles événement-condition-action (OPS5, Rete) |
| Loi | Contraintes et invariants ; Modelica |
| Capacité | Modèle « object-capability » (Dennis et Van Horn, 1966 ; langage E) |
| Unités typées | F# |
| Temps typé, déterminisme | Langages synchrones (Esterel, Lustre) ; Croquet |
| Journal causal | Event sourcing |

**Deux précédents absents du dépôt.** Verse, d'Epic Games (2023), est un langage conçu explicitement pour le métavers, avec transactions, pour du code écrit par des milliers d'auteurs. Inform 7 (2006) est un langage où mondes, lieux, relations, règles et scènes sont natifs. Le projet doit se situer par rapport à eux, et par rapport à Flecs.

**Où un apport reste possible.** Aucun de ces outils ne réunit espace, temps, unités, autorité et causalité dans une seule sémantique vérifiable qui tourne sur téléphone. L'apport serait une synthèse, comme Rust en a été une. Il reposerait sur trois problèmes difficiles :

1. résoudre les conflits entre lois de façon déterministe et compréhensible ;
2. maintenir des relations spatiales sur des milliers d'entités avec un budget fixe ;
3. garantir l'autorité à la compilation pour du code écrit par des inconnus.

**Ce qui justifie un langage plutôt qu'une bibliothèque.** Un paradigme ne rend rien calculable de plus ; il change ce que le compilateur sait du programme, donc ce qu'il peut vérifier et optimiser (comme SQL pour les requêtes ou Rust pour la mémoire). Si HoloCode n'est qu'une syntaxe traduite en ECS sans exploiter cette information, une bibliothèque suffit.

**Défauts relevés dans l'exemple `AutomaticDoor` du [paradigme](../01-holocode/PARADIGME-HOLOSCENIQUE.md).**

- Il ne contient aucune loi, et la frontière entre loi et phénomène n'est définie nulle part.
- `Near.active` est ambigu : une relation est un ensemble de paires, pas un booléen. `User` n'est jamais déclaré et la porte ne se referme jamais.
- Le phénomène écrit `Door.opened = true` directement, alors que l'[architecture](../01-holocode/ARCHITECTURE.md) interdit de modifier un état sans capacité.

**Comparaison POO / holoscénique.** Le même scénario (deux portes automatiques, des personnes, une règle « verrouillée signifie fermée ») a été écrit en Dart et en pseudo-HoloCode. Elle figure dans la transcription. Il y manque la version ECS et le scénario complet de la [vision](../00-vision/VISION.md).

**Recommandations de Claude.**

1. Ne plus revendiquer un « paradigme » tant qu'il n'est pas prouvé ; parler d'un langage de description de mondes.
2. Étudier Verse, Inform 7 et Flecs, puis écrire ce que HoloCode fait et qu'aucun des trois ne fait.
3. Choisir un seul des trois problèmes difficiles et construire le plus petit prototype qui le résout.
4. Comparer sur papier POO, ECS et pseudo-HoloCode avant de construire un parseur (la feuille de route place cette comparaison en phase 3, après l'interpréteur).
5. Construire la démonstration du Big Bang avec des outils existants, en parallèle, sans attendre le langage.

## Désaccords et limites

- **Deux langages dans une ambition.** HC-011 vise un langage « mieux que C, Rust, Zig », c'est-à-dire un langage système. Le dépôt décrit un langage de mondes et prévoit d'écrire HoloCode-Core d'abord en Rust ou C++. Claude recommande de s'en tenir au dépôt. Yocthan n'a pas tranché.
- **Deux projets dans une vision.** Second Life, Roblox et Fortnite ont réussi avec des langages banals ; les gens viennent pour le contenu. Claude estime que le langage n'est probablement pas le goulot d'étranglement du métavers, et que Yocthan devra choisir entre un projet de métavers et un projet de recherche en langages. Yocthan n'a pas répondu.
- **Limite de la comparaison.** La colonne POO décrit l'existant ; la colonne holoscénique décrit des intentions. L'exemple HoloCode de Claude est élégant en partie parce qu'il n'a pas eu à fonctionner. Les mots `law`, `forall`, `forbid` et `for 3s` sont de Claude et ne figurent pas dans le dépôt.
- Le style déclaratif a un défaut connu : conflits de règles et cascades difficiles à anticiper. Le journal causal prévu par l'architecture n'est donc pas optionnel.
- Claude n'a vérifié aucune de ces affirmations par une expérience. Les précédents sont cités de mémoire et doivent être contrôlés (rôle « recherche comparative » de Gemini).

## Décisions produites ou affectées

- Aucune ADR produite ni modifiée ; aucun statut changé.
- Critiquées : `ADR-003` (primitives candidates), `ADR-004` (archétypes et capacités), `ADR-005` (matériel existant, non mesurable sans les chiffres de HC-011).
- Propositions évoquées, non rédigées et non validées : une ADR sur le budget de 1 Go et la cible téléphone ; une section « Objections » dans le paradigme ; une comparaison papier ajoutée à la phase 1 de la feuille de route.

## Projets affectés

- HoloCode, HoloCompiler, gouvernance.

## Documents mis à jour

- [Index des discussions](../02-gouvernance/DISCUSSIONS.md) : ajout de HC-010 à HC-012.
- Création de `docs/05-discussions/` et de `outils/exporter_sessions_claude.py`.

## Questions ouvertes

- Que fait HoloCode qu'aucun de Verse, Inform 7 et Flecs ne fait ?
- Quelle est la différence sémantique entre une loi et un phénomène ?
- Lequel des trois problèmes difficiles attaquer en premier ?
- Le vrai projet de Yocthan est-il le métavers ou le langage ?
