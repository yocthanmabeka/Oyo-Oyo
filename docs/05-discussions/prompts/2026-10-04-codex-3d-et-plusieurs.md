Bonjour Codex. C'est Yocthan. Deux choix d'architecture attendent ton avis. Tu as accès au dépôt `yocthanmabeka/Metaverse` : lis, lance, et réponds en français, en phrases simples. Ne fusionne rien et ne change aucun statut de décision : range ta réponse dans `proposals/GPT5.6/`, par une pull request.

# Ce que tu dois lire

1. `proposals/Claude/modeles-3d-et-jeu-a-plusieurs-2026-10/README.md` : les deux propositions de Claude, trois options chacune, et sa recommandation.
2. `moteur/src/etat.rs` : l'arbitre (`arbitrer`, `guetter`, `saisir`, `glisser`, `recevoir`), le hasard rejouable, et la rencontre de deux objets (`corps`, `se_touchent`).
3. `moteur/src/rendu.rs` et `moteur/src/mosaique.rs` : le moteur de dessin, qui ne dessine que des points.
4. `moteur/src/bin/holo.rs` et `moteur/outils/serveur.mjs` : le moteur en ligne de commande, et le serveur de démonstration.
5. `docs/02-gouvernance/adr/ADR-023` à `ADR-032`, et le journal du 2026-10-04.
6. Pour lancer : `moteur/README.md`, puis `http://localhost:8080/exemples/jeu/panier.holo` et `…/exemples/lecons/01-page.holo`.

# Ce que je te demande

## Les modèles 3D

1. Dans `rendu.rs`, que faudrait-il vraiment ajouter pour dessiner des triangles (option A) ? Donne une estimation en lignes, en poids du fichier `.wasm`, et dis ce que cela ferait à la fluidité mesurée sur téléphone.
2. L'option B (des objets faits de points) tient-elle avec le moteur tel qu'il est ? Quelle limite de points par objet, et que devient `POINTS_MAX` ?
3. Y a-t-il une quatrième voie que Claude n'a pas vue ?

## Le jeu à plusieurs

4. L'arbitre est-il vraiment une fonction pure ? Cherche tout ce qui la rend dépendante d'autre chose que son entrée : l'horloge du navigateur, la largeur du plateau donnée par la page (le nom « < » dans l'état), le `thread_local` des capacités demandées, l'ordre des appels.
5. Si l'arbitre tournait sur un serveur pour l'option A, que pourrait envoyer un joueur malhonnête ? Écris des sondes, comme dans ta revue du 2026-10-03.
6. La rencontre de deux objets dépend maintenant de la largeur de l'écran. Dis si c'est acceptable pour un jeu à plusieurs, et ce que tu proposes à la place.
7. Qu'est-ce qu'il faut décider avant d'écrire la première ligne du serveur ?

## Et les noms

8. Depuis ta contre-revue des noms, le langage a reçu : `If`, `Hr`, `Quote`, `Code`, `Every`, `Board`, `Input`, `Checkbox`, `When`, `Part`, `Use`, `Data`, `Sound`, `Shape`, et les réglages `keep`, `prices`, `data`, `meets`, `within`, `drag`, `form`, `from`, `every`, `value`, `label`. Applique-leur la même règle qu'aux autres. La liste complète est dans `docs/01-holocode/NOMS.md`.

Pour chaque point : sépare ce que tu as vérifié en lançant le code de ce que tu supposes. Termine par une liste classée, du plus important au moins important.
