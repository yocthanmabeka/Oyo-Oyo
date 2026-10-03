# Feuille de route

> **Réordonnée le 2026-10-03.** Les trois IA et Yocthan s'accordent : la documentation courait plus vite que le moteur. La priorité est devenue la preuve sur un vrai téléphone. Les phases d'origine sont conservées plus bas.

## Sprint Big Bang : réalisé le 2026-10-03, mesures téléphone en attente

- [x] Moteur en Rust compilé en WebAssembly, rendu `wgpu`, repli WebGPU vers WebGL 2 (`moteur/`).
- [x] Lecteur du format `.holo` en blocs (`ADR-009`), avec erreurs à la ligne et à la colonne.
- [x] Un point lumineux qui se morcelle, l'entrée dans un point, son monde né de sa graine, la sortie.
- [x] 17 tests du cœur, exécutés aussi sur GitHub ; poids transféré 489 Ko.
- [ ] **Mesures sur le téléphone de Yocthan** : images par seconde, pire image, première image, mémoire, batterie. Elles décident d'`ADR-005` et d'`ADR-010`.
- [x] Toucher un point pour le viser.
- [ ] Continuité de l'entrée.

## Ensuite, dans l'ordre proposé

- [ ] La vue à plat (`ADR-007`, `ADR-011`) : `Page`, `Texte`, `Bouton`, `Image` ; la même source dans les deux vues.
- [ ] Les lois, phénomènes et capacités dans le moteur, en reprenant la sémantique de `proposals/Claude/` dans le format en blocs.
- [ ] Faire passer au moteur la [suite de conformité](../../experiments/conformite-v0.1/README.md), et l'étendre.
- [ ] La trace humaine : modifier un monde, le partager par un lien.
- [ ] Les modules enfermés (`ADR-013`), puis la présence des autres.

## Prototype v0.1 — réalisé

- [x] Lexer minimal
- [x] Parseur et AST
- [x] Monde et entités
- [x] Positions 3D en mètres
- [x] Relation de proximité
- [x] Phénomènes conditionnels
- [x] Effets atomiques par tick
- [x] Détection des effets contradictoires
- [x] CLI, exemple et tests automatisés

## Phase 0 — Fondation documentaire

- [x] Créer le référentiel central.
- [x] Définir les statuts documentaires.
- [x] Créer l'index des discussions.
- [x] Créer le registre des décisions.
- [x] Ajouter les URL réelles des conversations existantes (ChatGPT ; les sessions Claude Code n'ont pas d'URL, voir l'index).
- [x] Valider ou corriger les propositions initiales (`ADR-003` à `ADR-005` acceptées, `ADR-006` en proposition).

## Phase 1 — Sémantique minimale

- [ ] Formaliser monde, espace, entité, relation, loi et phénomène.
- [ ] Définir le système d'identité et les unités.
- [ ] Définir les erreurs et conflits.
- [x] Écrire une grammaire EBNF minimale (brouillon dans la suite de conformité).
- [ ] Construire dix programmes d'exemple et dix contre-exemples (quatre et sept à ce jour).

## Phase 2 — Premier exécuteur

- [ ] Construire le parseur.
- [ ] Construire l'analyseur sémantique.
- [ ] Sérialiser HoloIR.
- [ ] Créer un interpréteur déterministe.
- [ ] Produire une trace causale des phénomènes.

## Phase 3 — Comparaison scientifique

- [ ] Implémenter les mêmes scénarios avec HoloCode et une référence ECS/réactive.
- [ ] Mesurer lisibilité, erreurs détectées, temps CPU, mémoire et latence.
- [ ] Publier les protocoles et résultats reproductibles.
- [ ] Réviser le paradigme selon les résultats.

## Phase 4 — Monde partagé

- [ ] Ajouter autorité, persistance et réplication.
- [ ] Tester pertes de connexion et événements désordonnés.
- [ ] Définir les permissions par capacités.
- [ ] Prototyper un petit espace multi-utilisateur.

## Phase 5 — Recherche avancée

- [ ] Étudier l'algèbre géométrique native.
- [ ] Expérimenter calcul spatial/volumétrique spécialisé.
- [ ] Évaluer NDN, TSN/DetNet et architectures distribuées pertinentes.
- [ ] N'envisager une ISA spatiale qu'après identification de charges de travail stables.
