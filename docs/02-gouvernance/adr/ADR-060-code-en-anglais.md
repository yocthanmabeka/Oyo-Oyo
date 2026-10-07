# ADR-060 — Le code en anglais, les commentaires et les messages en français

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : `ADR-016` (le vocabulaire de HoloCode en anglais), `ADR-037` (l'écriture des noms)
- Validation : Yocthan, le 2026-10-07 : « pour ce qui concerne le code, le code doit être 100 % en anglais, bien sûr. […] le commentaire, tu le mets en français pour que je comprenne et tout. Mais à part ça, rien d'autre. » ; puis « Oui, fais comme je le dis. »
- Projets affectés : HoloEngine, serveur d'essai, outils, extension VS Code

## Décision

1. **Tout le code est en anglais** : les noms de fonctions, de variables, de types, de modules et de fichiers sources ; les attributs `data-*` et les classes CSS que le moteur fabrique (`data-key`, `holo-line`, `holo-positioned`) ; les identifiants et classes des pages du moteur (`#toggle`, `#crossroads`, `.preview`) ; les variables CSS du moteur (`--background`, `--muted`) ; les commandes (`holo test`, `holo files`, `holo vocabulary`) ; les paramètres d'adresse (`?values`, `?bare`, `?measures`, `?world=`, `?key=`, `?file=`) ; les variables d'environnement (`HOLO_ENGINE`, `HOLO_REPO`, `HOLO_KEY`, `HOLO_NOW`, `DARK`, `WIDTH`) ; les clés JSON échangées entre le moteur et la page ; le paquet (`holo-engine`, `holo_engine.wasm`, `pkg-light`) et le réglage `drawing`.
2. **Restent en français** : les commentaires, les messages d'erreur du moteur, les textes que le visiteur lit, les leçons et leurs essais (`.test`), toute la documentation. Les dossiers du dépôt gardent leur nom (`moteur/`, `exemples/`, `outils/`, `mondes/`) : ce sont des rangements, pas du code.
3. **Les noms suivent les mots déjà choisis** dans HoloCode : `State` → `state`, `Repeat` → `repeat`, `Component` → `components`, `settings`, `rules`, `blocks`.

## Ce qui a changé de nom

| Avant | Après |
|---|---|
| `src/etat.rs`, `plat.rs`, `regles.rs`, `listes.rs`, `composants.rs`, `blocs.rs`, `repetition.rs`, `mouvement.rs`, `vue.rs`, `graine.rs`, `mosaique.rs`, `univers.rs`, `rendu.rs`, `outils.rs`, `fichiers.rs` | `state.rs`, `flat.rs`, `rules.rs`, `lists.rs`, `components.rs`, `blocks.rs`, `repeat.rs`, `movement.rs`, `view.rs`, `seed.rs`, `mosaic.rs`, `universe.rs`, `renderer.rs`, `tools.rs`, `files.rs` |
| `web/page-moteur.js`, `editeur.js`, `corrections.js`, `mesures.js`, `editeur.html`, `pile.html`, `accueil.html` | `page-engine.js`, `editor.js`, `fixes.js`, `measures.js`, `editor.html`, `stack.html`, `home.html` |
| `outils/serveur.mjs`, `accessibilite.mjs`, `capturer.mjs`, `montrer.mjs`, `mesurer-telephone.mjs`, `construire.ps1` | `server.mjs`, `accessibility.mjs`, `capture.mjs`, `show.mjs`, `measure-phone.mjs`, `build.ps1` |
| `holo essai`, `holo fichiers`, `holo vocabulaire`, `x.essai` | `holo test`, `holo files`, `holo vocabulary`, `x.test` |
| `/editeur?cle=`, `/pile`, `?valeurs`, `?nu`, `?mesures`, `?monde=` | `/editor?key=`, `/stack`, `?values`, `?bare`, `?measures`, `?world=` |
| `web/pkg-leger`, `holo_moteur.wasm`, feature `dessin`, `target/leger` | `web/pkg-light`, `holo_engine.wasm`, feature `drawing`, `target/light` |
| `messages/fichiers/`, `editeur-sauvegardes/` | `messages/files/`, `editor-backups/` |

## Critères de validation

- Les 123 tests du moteur passent ; les deux paquets WebAssembly se construisent ; `node --check` sur chaque script.
- Dans Chrome : le monde d'accueil, les leçons 9, 49, 52, 64 (message envoyé), 71, 74 (le pli reste ouvert), 76 (fichier envoyé), le site de référence, l'éditeur (liste des fichiers, aperçu, coloration) et la pile (ouvrir une page, revenir).
- L'audit axe-core : 76 leçons et le site de référence, 0 défaut, en clair et en sombre.
