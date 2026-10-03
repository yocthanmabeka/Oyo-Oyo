# Ce qui manque au projet

L'état réel est meilleur que le prompt initial : le moteur Rust/WASM, la CI et une suite de conformité existent déjà. Voici les prochains manques, par priorité.

| Domaine | Manque | Plus petite action qui le comble |
|---|---|---|
| Preuve | Aucune mesure publiée sur le Samsung Z Flip 5 | Un seul protocole de 10 minutes, un fichier de résultats brut, versions appareil/Chrome, capture et commandes |
| Langage | La grammaire, le parseur Rust et la conformité peuvent diverger | Faire d'une commande `holo check fichier.holo` l'autorité et l'exécuter sur tous les exemples en CI |
| Moteur | Le budget « moins de 1 Go » n'est pas encore un budget par sous-système | Fixer des plafonds mesurables : moteur, monde, textures procédurales, journal, cache, pic au zoom |
| Preuve | Pas de scénario visuel reproductible du Big Bang | Un test Playwright : ouvrir, attendre, capturer, zoomer, capturer, comparer avec une tolérance documentée |
| Moteur | Le repli WebGPU/WebGL est annoncé mais la matrice de preuve manque | Tester deux modes forcés sur ordinateur et le mode réellement choisi sur téléphone |
| Sécurité | Les entrées `.holo`, URL et modules n'ont pas encore de corpus hostile | Ajouter 20 petits fichiers invalides : profondeur, taille, nombres extrêmes, imports, chemins, UTF-8 |
| Langage | Pas d'expérience d'édition `.holo` | Extension VS Code minimale : association `.holo`, commentaires, chaînes, nombres, mots-clés ; aucun serveur de langage au début |
| Organisation | Une PR a déjà été fusionnée en étant confondue avec une autre | Checklist obligatoire : auteur, branche source, branche cible, fichiers, tests, avis humain explicite |
| Organisation | Claude « programmeur principal » et ChatGPT « synthèse/gouvernance » ne disent pas qui peut modifier quel fichier | Une issue par tâche avec `paths:` réservés ; un seul propriétaire actif par chemin |
| Documentation | Les résultats annoncés peuvent vieillir | Une page `ETAT-ACTUEL.md` générée à partir des tests/mesures, datée et liée au commit |

## Sécurité gratuite à ajouter progressivement

- permissions GitHub Actions minimales (`contents: read`) ;
- actions tierces épinglées à un SHA complet, pas seulement `@v4` ;
- dépendances Rust auditées dans une tâche séparée ;
- aucun secret dans une PR, un exemple ou un fichier `.holo` ;
- modules WebAssembly refusés par défaut jusqu'à ce que capacités, mémoire et temps soient effectivement imposés.
