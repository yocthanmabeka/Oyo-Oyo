# Tout le web avant le métavers : la synthèse des avis

- Auteur : Claude, le 2026-10-07.
- Statut : **PROPOSITION**. Le plan en lots et les dix parcours attendent la décision de Yocthan.
- Sources : le prompt [`docs/05-discussions/prompts/2026-10-07-codex-gemini-tout-le-web.md`](../../../docs/05-discussions/prompts/2026-10-07-codex-gemini-tout-le-web.md) ; la réponse de Gemini ([`2026-10-07-gemini-tout-le-web.md`](../../../docs/05-discussions/reponses/2026-10-07-gemini-tout-le-web.md)) ; la réponse de Codex ([`2026-10-07-codex-tout-le-web.md`](../../../docs/05-discussions/reponses/2026-10-07-codex-tout-le-web.md)) et sa revue complète ([`proposals/GPT5.6/tout-le-web-avant-3d-2026-10-07/`](../../GPT5.6/tout-le-web-avant-3d-2026-10-07/README.md)) ; l'exploration des dix pistes par Claude ([`proposals/Claude/exploration-web-complet-2026-10/`](../exploration-web-complet-2026-10/README.md)).
- Le but, redit par Yocthan le 2026-10-07 : « le code doit d'abord faire tout ce que les HTML CSS JavaScript savent faire et ensuite faire le métaverse » ; la règle `ADR-035` : une mécanique, jamais une capacité.

## Ce sur quoi les trois sont d'accord

1. **Le but est atteignable pour les sites courants, sans code libre**, à une condition : élargir les modules enfermés. Ils doivent pouvoir échanger des textes, des listes et des éléments à champs, toujours bornés en temps et en mémoire, et sans accès direct à la page, au réseau ou à l'appareil. Codex le dit ainsi : « non avec une liste fermée de mots », « oui avec une liste publique des dettes et des modules généraux mais enfermés ».
2. **L'ordre des premières dettes** : d'abord les données et le calcul, puis les formulaires, puis la vraie mise en page d'ordinateur et de téléphone.
3. **Le serveur avant la 3D** : la 3D à plusieurs dépendra de ses droits. « Chez soi d'abord » est respecté, et SQLite, une base de données dans un fichier, est le bon premier choix.
4. **Des nombres décimaux exacts, des dates et du travail sur les textes**, fournis par le moteur (des fonctions pures), pas par des formules libres.
5. **L'appareil (caméra, position, presse-papiers) n'est jamais donné à un module** : le moteur demande la permission, sur un geste clair du visiteur, et ne remet que le résultat.
6. **Pour la mise en page, pas de requêtes média libres comme en CSS.** Il faut quelques paliers nommés, selon la place que reçoit le bloc.

## Là où ils divergent

| Sujet | Gemini | Codex | Claude (exploration) | Recommandation |
|---|---|---|---|---|
| Des essais dans un vrai navigateur, et des limites (taille, temps, mémoire) | absents | le lot 1 | les lots 1 et 2 (réparer, puis un essai dans Chrome à chaque PR) | **d'abord** : trois pannes du navigateur ont été trouvées le même jour, alors que les 124 tests restaient verts |
| Le total des séances avant la 3D | environ 24 | 44 à 64 | même ordre que Codex | **compter sur 45 à 65 séances**, l'estimation de Gemini est trop optimiste |
| Le dessin libre (Canvas 2D) | refusé (« du code spaghetti ») | un module qui lit une zone de dessin et rend des ordres de dessin vérifiés | un canevas chargé seulement quand il sert | **la capacité gardée, la mécanique refusée** : un dessin vectoriel déclaré, et des modules qui rendent des ordres de dessin vérifiés |
| La mise en page | `Page(maxWidth:)`, `Row(phone: Column)` | trois situations, étroit, normal et large, selon la place reçue | `Page { max-width }`, `Row(columnBelow:)` | **trois paliers nommés selon la place reçue**, et une largeur de page réglable |
| Le hors-ligne | le lot 8 | peut attendre, s'il est complet | le dernier lot | à la fin, avant la 3D |
| Les valeurs partagées changées seulement par un toucher | — | « arbitraire » : le clavier vient aussi du navigateur | — | **Codex a raison** : les mêmes contrôles pour tous les gestes |

## Ce qu'il faut corriger chez nous

- **Un mot de passe est haché, pas chiffré** : une empreinte qu'on ne peut pas défaire (Argon2id). L'erreur était dans notre prompt et dans la proposition de serveur ; Gemini l'a reprise sans la voir.
- **La règle du même résultat** était trop forte. Formule proposée par Codex, à adopter : *même fichier, même version du moteur, même état initial et même suite ordonnée des gestes donnent le même résultat logique.* La langue, l'écran, l'heure, le hasard et les réponses du serveur sont des entrées, à enregistrer pour pouvoir rejouer.
- **« 77 % du web » est une estimation de Claude**, pas une mesure. Et le résumé du grand tableau additionne 129 éléments sur 130 : il oublie le seul « sans objet » (`requestAnimationFrame`, que le moteur fait seul).
- **Il manquait à la liste des dettes**, d'après Codex : l'impression, l'historique avant et arrière, les téléchargements interrompus, le glisser-déposer de fichiers, les notifications, les tâches en arrière-plan, WebRTC (la voix et la vidéo en direct entre deux visiteurs), l'import et l'export de données.

## Le plan proposé : neuf lots avant la 3D

| # | Lot | Ce qu'on pourra faire à la fin | Moyen | Estimation (séances) |
|---|---|---|---|---|
| 1 | **Essais dans un navigateur, limites, réparations** | ne plus casser le doigt, les modules ou le clavier sans le voir ; les fichiers et les listes trop gros arrêtés ; les défauts trouvés par l'exploration réparés | moteur et essais | 3 à 4 |
| 2 | **Données et calcul** : les types communs (texte, décimal exact, date, durée, liste, erreur), chercher, filtrer, trier, couper en pages, des valeurs calculées, un `Data` qui dit « chargement » et « échec » | un catalogue de 200 produits où l'on cherche et trie, lisible par un robot ; un panier qui calcule la TVA | fonctions pures et quelques mots | 6 à 8 |
| 3 | **Formulaires sûrs** | une inscription vérifiée, les erreurs annoncées, un envoi unique, un délai | mots et serveur | 4 à 6 |
| 4 | **Mise en page d'ordinateur et de téléphone** | une vraie page large et sa version téléphone | mots de disposition | 4 à 6 |
| 5 | **Premier vrai serveur** : `holo serve`, SQLite, sauvegardes, adresses comme `/profil/123` | un site qui tourne sur le serveur en Rust, sur le PC | serveur | 6 à 9 |
| 6 | **Valeurs partagées et direct** | une réservation que tout le monde voit tout de suite | arbitre sur le serveur, WebSocket | 5 à 8 |
| 7 | **Comptes et droits** | un compte local, son panier sur deux appareils, un code à 6 chiffres | serveur et base locale | 7 à 10 |
| 8 | **Le HTML et les médias qui manquent** : `aside`, nouvel onglet, téléchargement, sous-titres, images différées, impression, historique | un article long, accessible, imprimable, avec une vidéo sous-titrée | mots du langage | 3 à 5 |
| 9 | **Les capacités larges** : modules typés, dessin vectoriel, appareil sur permission, hors-ligne, import et export, notifications | un tableau de bord qui reçoit, calcule et dessine ses données | modules et permissions | 8 à 12 |

**En tout : environ 46 à 68 séances**, estimées. Les paiements et WebRTC viendraient après, avec un prestataire au choix pour le paiement.

## Les dix parcours du « web viable » (l'idée de Codex)

Le web sera dit viable quand ces dix parcours marcheront, chacun au clavier seul, au lecteur d'écran et sur téléphone :

1. Un catalogue de 200 produits : chercher, filtrer, trier, page par page ; un robot de recherche le lit.
2. Une fiche, le panier, le total avec la TVA, en euros et centimes exacts.
3. Une inscription : champs vérifiés, erreurs annoncées, un seul envoi.
4. Un message de contact avec un fichier, et sa confirmation.
5. Une page d'ordinateur large et la même sur téléphone, avec son menu.
6. Une réservation partagée : Ada réserve, Bob le voit sans recharger.
7. Un compte : se connecter, retrouver son panier sur un autre appareil.
8. Un profil public à son adresse (`/profil/123`), partagé avec un bel aperçu.
9. Un article long, accessible, imprimable, avec une vidéo sous-titrée et des images légères.
10. Un tableau de bord qui reçoit des données, les calcule et les dessine.

## Ce que Yocthan doit décider

1. **Ce plan en neuf lots, et son ordre.** Et quand la 3D commence : après le lot 7 (le serveur et les comptes), ou après le lot 9 ?
   **Décidé par Yocthan le 2026-10-07 : la 3D commence après le lot 9.** « que la 3D commence après les lots 9. Au lieu de s'empresser, c'est mieux qu'on puisse terminer le lot 9 et ensuite commencer la 3D tranquillement ».
2. **Les dix parcours** comme définition du web viable.
3. **La règle du même résultat**, reformulée.
4. **Les valeurs calculées** : une seule notion pour chercher, filtrer, trier et calculer (pistes 1 et 10 de l'exploration).
5. **Les quatre questions du serveur**, avec les corrections de Codex.
   **Décidé par Yocthan le 2026-10-07, pour le lot 5** : `holo serve` en Rust, avec deux bibliothèques éprouvées, `tiny_http` (le serveur web) et `rusqlite` (la base SQLite, comprise dans `holo.exe`) ; rien d'autre à installer, aucun service extérieur. Les adresses comme `/profil/123` se disent par le nom du fichier : `profil/{id}.holo`, et `{id}` s'écrit dans la page comme les autres valeurs.

Le lot 1 ne demande aucune décision de langage : il peut commencer tout de suite.
