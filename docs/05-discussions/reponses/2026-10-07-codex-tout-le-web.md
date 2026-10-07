# Réponse de Codex (GPT-5.6) — tout ce que le web sait faire, avant le métavers (2026-10-07)

Réponse au prompt [`../prompts/2026-10-07-codex-gemini-tout-le-web.md`](../prompts/2026-10-07-codex-gemini-tout-le-web.md), collée par Yocthan le 2026-10-07. Sa revue complète est dans la PR 144 : [`proposals/GPT5.6/tout-le-web-avant-3d-2026-10-07/README.md`](../../../proposals/GPT5.6/tout-le-web-avant-3d-2026-10-07/README.md). Texte gardé tel quel ; la lecture de Claude est à la fin.

---

## 1. Peut-on atteindre « tout le web » sans code libre ?

**Non, pas avec une liste fermée de mots et les modules actuels « un nombre entre, un nombre sort » : le web évolue sans cesse et certains auteurs auront besoin de calculs que le moteur ne connaît pas.** **Oui, HoloCode peut couvrir toutes les grandes familles utiles s'il conserve une liste publique de ses dettes et permet des modules généraux, mais enfermés, limités et sans accès direct à la page, au réseau ou à l'appareil.**

## 2. Les lots, dans l'ordre

Une séance signifie ici : décider un petit contrat, le construire, écrire les tests et l'essayer.

| Lot | Résultat utilisable | Moyen principal | Estimation |
|---|---|---|---:|
| 1. Tests et limites | Les régressions du navigateur deviennent visibles ; les fichiers et listes trop gros sont arrêtés | Moteur et tests | 3–4 séances |
| 2. Données et calcul | Un catalogue cherchable, filtrable, triable et paginé, visible aussi par les moteurs de recherche | Fonctions pures et quelques mots | 6–8 |
| 3. Formulaires | Validation, erreurs accessibles, Entrée, envoi unique, délai maximal | Mots du langage et serveur | 4–6 |
| 4. Mise en page adaptative | Vraies pages de téléphone et d'ordinateur, composants qui réagissent à leur propre place | Mots de disposition | 5–7 |
| 5. Premier vrai serveur | `holo serve`, adresses comme `/profil/123`, données SQLite et sauvegardes | Serveur Rust | 6–9 |
| 6. Valeurs partagées | Réservation et jeu à plusieurs, sans croire le navigateur | Arbitre serveur et WebSocket | 5–8 |
| 7. Comptes et droits | Compte local, panier sur plusieurs appareils, double authentification, droits | Serveur et base locale | 7–10 |
| 8. Capacités larges | Dessin, appareil, médias avancés, fonctionnement hors ligne | Modules typés et permissions | 8–12 |

Un **WebSocket** est une connexion qui reste ouverte entre la page et le serveur pour recevoir immédiatement les changements.

Je ne mettrais pas le serveur avant les données, les formulaires et la disposition. En revanche, je le construirais avant la 3D publique : la 3D à plusieurs dépendra de son modèle de droits et d'autorité.

## 3. Réponses courtes aux autres questions

### Quelles dettes payer avant la 3D ?

À payer avant :

- données, recherche, filtre, tri et pagination ;
- formulaires complets et accessibles ;
- vraie disposition téléphone/ordinateur ;
- limites de taille, de temps et de mémoire ;
- adresses dynamiques comme `/profil/123` ;
- serveur local, valeurs partagées, comptes et droits ;
- chargement différé des images et sous-titres vidéo.

Peuvent attendre :

- filtres graphiques rares ;
- découpe avancée des formes ;
- dessin libre très avancé ;
- particules ;
- paiement ;
- fonctionnement hors ligne complet.

À refuser comme mécaniques, mais pas comme capacités :

- JavaScript libre ;
- modification directe et arbitraire de la page ;
- réseau libre depuis un module ;
- caméra ou position sans geste clair du visiteur ;
- stockage secret incontrôlé ;
- modules sans limite de temps ou de mémoire.

### Le serveur et les comptes

La direction de Claude est bonne et respecte « chez soi d'abord ». SQLite — une base de données contenue dans un fichier — est un excellent premier choix local.

Mais quatre corrections sont nécessaires :

1. Le « même arbitre » ne suffit pas : il faut aussi la même version du moteur, le même fichier, le même état initial, le même ordre des gestes et les mêmes valeurs de temps et de hasard.
2. Envoyer un geste au lieu d'une valeur n'est pas une protection : le serveur doit vérifier le droit, la fréquence et l'objet visé.
3. Les promesses de « quelques microsecondes » et de « milliers de demandes par seconde » ne sont pas mesurées.
4. Un mot de passe ne doit pas être « chiffré », mais **haché** : transformé en empreinte irréversible, avec Argon2id.

La distinction `State`, `keep`, `shared` et `mine` est bonne. En revanche, autoriser un toucher pour modifier `shared` mais interdire le clavier serait arbitraire : les deux viennent du navigateur et doivent subir les mêmes contrôles.

### Les grandes capacités de JavaScript

Oui, il faut élargir les modules enfermés :

- entrées et sorties de type texte, nombre, booléen, liste et élément à champs ;
- lecture seule d'une zone de dessin ;
- production d'une liste d'ordres de dessin validés ;
- quantité maximale de données ;
- temps maximal ;
- mémoire maximale ;
- résultat déterministe ;
- aucun réseau, appareil ou accès direct à la page.

Pour la caméra, la position, le presse-papiers ou le partage, le module ne doit pas recevoir l'appareil. Il demande une capacité au moteur ; le moteur vérifie le geste du visiteur, affiche la permission et ne remet que le résultat autorisé.

Pour le hors-ligne, utiliser un **service worker**, c'est-à-dire un petit responsable du cache fourni par le navigateur, mais entièrement produit et contrôlé par HoloCode.

### La mise en page

Je proposerais trois niveaux :

1. une disposition normale qui s'adapte sans seuil ;
2. quelques variantes nommées selon la largeur réellement disponible au bloc, pas seulement selon l'écran ;
3. des limites explicites comme largeur minimale, largeur idéale et passage en colonne.

Cela correspond aux « requêtes de conteneur » du CSS : un composant regarde la place qu'il reçoit. HoloCode doit masquer leur complexité, pas supprimer leur capacité.

Je déconseille une multiplication de seuils numériques libres. Commencer avec trois situations stables — étroit, normal, large — puis permettre des seuils précis seulement lorsqu'un besoin réel l'exige.

### La règle du même résultat

« Un même fichier donne toujours le même résultat » est trop fort. La formule correcte serait :

> Même fichier, même version du moteur, même état initial et même journal ordonné des entrées donnent le même résultat logique.

La langue, la taille de l'écran, les polices disponibles, les permissions, les réponses du serveur, l'heure et le hasard peuvent changer l'affichage. Il faut les déclarer comme entrées, puis pouvoir les enregistrer et les rejouer.

### Ce qui a été oublié ou mal mesuré

- Le « 77 % du web » n'est pas une mesure reproductible : il mélange des capacités de tailles très différentes.
- `TABLEAU-WEB.md` annonce 130 éléments, mais les sous-totaux donnent 129 : `89 + 21 + 13 + 6`.
- Il manque notamment l'impression, l'historique avant/arrière, les téléchargements interrompus, le glisser-déposer de fichiers, les notifications, les tâches en arrière-plan, WebRTC et l'importation/exportation de données.
- **WebRTC** est la technique du navigateur permettant une communication directe, notamment pour la voix et la vidéo.
- Le rapport Claude `proposals/Claude/exploration-web-complet-2026-10/` n'était pas présent dans le commit examiné.

## 4. Principaux risques

- **Régressions invisibles :** le journal montre que le pincement et les modules ont déjà cassé alors que les tests Rust restaient verts. Il faut des tests dans un vrai navigateur.
- **Faux sentiment de sécurité :** un bouton caché n'est pas un droit. Toutes les actions sensibles doivent être vérifiées par le serveur.
- **Épuisement des ressources :** listes, fichiers, images, récursions et modules ont besoin de limites réelles en octets, temps, profondeur et mémoire.
- **Accessibilité :** les variantes de disposition, dessins, erreurs de formulaires et mises à jour en direct pourraient casser ce qui fonctionne déjà.
- **Poids :** ne chargez ni dessin, ni appareil, ni direct, ni 3D dans une page qui n'en a pas besoin.
- **Explosion du vocabulaire :** une dette ne doit pas automatiquement créer un nouveau mot. Les calculs généraux appartiennent plutôt aux fonctions pures ou aux modules typés.
- **Dépendance au serveur central :** `holo serve` doit rester complet sur le PC de l'auteur ; les prestataires doivent rester facultatifs.
- **Sauvegardes et évolution :** SQLite exige des transactions — modifications faites entièrement ou pas du tout —, une évolution contrôlée de sa structure et des sauvegardes vérifiées.

## 5. Ce que je ferais cette semaine

1. Écrire dix parcours web obligatoires servant de vraie définition de « web viable ».
2. Ajouter les tests navigateur du pincement, des modules, des formulaires et du clavier.
3. Fixer les limites de taille, profondeur, mémoire, durée et réseau.
4. Décider les types communs : texte, décimal exact, date, durée, liste et erreur.
5. Construire un catalogue cherchable, filtrable, triable et paginé avant toute 3D.

## Vérifications réellement exécutées

- Localement, `cd moteur && cargo test` a rendu exactement `cargo: command not found`, code de sortie 127. Je n'ai rien installé.
- Tests GPT5.6 : 8 réussis, 1 échec volontaire signalant l'absence de Cargo.
- Tests Claude : 27 réussis.
- Tests Gemini : 8 réussis.
- Conformité : 22 cas, dont 6 acceptés et 16 refusés ; suite bien formée.
- Dans GitHub Actions : les cinq contrôles sont verts. Le contrôle Rust a exécuté les **124 tests** et a aussi réussi les deux compilations WebAssembly.

Pull request ouverte : PR #144 — Revue GPT5.6 : toutes les dettes du web avant la 3D.

---

## Lecture de Claude

- **Vrai, et à corriger chez nous** : le mot de passe est **haché** (une empreinte qu'on ne peut pas défaire, Argon2id), pas chiffré. L'erreur venait de notre prompt ; Gemini l'a reprise sans la voir.
- **Vrai** : le tableau compte 130 éléments, mais son résumé n'en additionne que 129 : il oublie le seul élément « sans objet » (`requestAnimationFrame`, que le moteur fait seul). À écrire dans le résumé.
- **Vrai** : « 77 % » est une estimation de Claude, élément par élément, pas une mesure. Le tableau le disait ; le prompt l'a présenté trop vite comme un chiffre.
- **Juste, et confirmé par l'exploration de l'issue #82** : les tests dans un vrai navigateur d'abord. Trois pannes du navigateur ont été trouvées le même jour (le pincement, les modules, Échap), alors que les 124 tests restaient verts.
- **Une bonne idée, à reprendre telle quelle** : dix parcours obligatoires comme définition de « web viable ».
- **Ses estimations (44 à 64 séances) sont bien plus hautes que celles de Gemini (24)** : voir la synthèse.
