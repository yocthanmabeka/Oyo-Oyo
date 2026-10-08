# Reprise du PC : comptes et valeurs partagées

- Sujet : empêcher la copie d'état envoyée par une page de remplacer l'état gardé d'un membre.
- Auteur : Codex, à la demande de Yocthan du 2026-10-08 (« Continue la partie de Claude, la session sur PC »).
- Discussions sources : tâches #187 et #197, sondes de la PR #198 ; aucune nouvelle référence HC créée.
- Décisions concernées : ADR-074, ADR-079, ADR-081. Aucun statut modifié.
- Statut de la contribution : EXPÉRIMENTATION, PR en brouillon.
- Base : PR #177, branche `langage/lot7-comptes`, commit `9522483e0d58ba4f86895f7021c208b95c3fd7fa`.

## Problème

`shared_gesture` arbitrait le JSON avec l'état personnel du navigateur, puis le gardait même après un refus. Pour un membre qui avait déjà réservé, envoyer `booked=0` rendait son bouton à nouveau visible. Envoyer `cart=777` pouvait également remplacer son panier. Les valeurs de `Shared` venaient correctement de la base ; cela ne suffisait pas à protéger les conditions qui lisaient l'état personnel.

Les formulaires relisaient la visite avant de prendre le verrou des valeurs partagées et l'enregistraient après l'avoir rendu. Deux demandes du même compte pouvaient donc calculer avec la même ancienne valeur de `booked`.

## Correction proposée

1. Pour un membre, le JSON part de l'état de son compte dans la base. Les autres valeurs personnelles envoyées sont ignorées.
2. Les valeurs liées à `Input`, `Checkbox`, `Slider` et `Choice` restent modifiables. Elles repassent par la validation des saisies existante. Le texte est décodé, les nombres à virgule gardent leur échelle ; les champs absents ou mal codés ne sont pas remis à zéro.
3. Un JSON refusé ne garde aucun nouvel état, même pour un visiteur sans compte. Pour un membre, sa réponse rend l'état gardé, avec les valeurs partagées actuelles.
4. Les gestes JSON, les formulaires ordinaires et les miroirs sont exécutés un à un, de la lecture de l'état jusqu'à son enregistrement. Ce verrou est distinct de celui de la base ; les lectures et les pages de compte ne le prennent pas.
5. Le navigateur met les miroirs personnels et les gestes partagés dans la même file. Un geste partagé attend donc le miroir précédent ; les miroirs ont une limite d'attente de dix secondes.
6. Un état JSON trop lourd est refusé avant de modifier les valeurs partagées.

Il n'y a aucun nouveau mot du langage, aucune dépendance et aucune migration de base. Les demandes JSON existantes restent lisibles. Le moteur WebAssembly et son dessin ne changent pas.

## Preuves ajoutées

| Essai | Ce qu'il vérifie |
|---|---|
| Rust : compte falsifié | Première réservation par formulaire ; seconde en JSON avec `booked=0` refusée ; `cart=777` ignoré même sur un geste accepté ; un autre compte peut réserver. |
| Rust : refus sans écriture | `note=Eve`, `cart=777` et un prix falsifié ne remplacent pas l'état enregistré après un refus. |
| Rust : saisies normales | Texte borné à cinq caractères, montant 13,50 exact, case cochée, glissière bornée, option inconnue refusée ; champs absents ou mal codés ignorés. |
| Rust : visiteur refusé | Le refus JSON ne crée pas de visite enregistrée. |
| Rust : huit demandes simultanées | Quatre formulaires et quatre JSON du même compte, démarrés ensemble : une seule place consommée, une seule version partagée. |
| Chrome et HTTP réel | Compte créé, réservation sans JavaScript, attaque JSON du même compte refusée, rechargement qui prouve la conservation en base, puis toucher et saisie ordinaires avec JavaScript ; miroir tenu en attente, toucher partagé qui attend, ordre et panier conservés. |

La même page `concert.holo` est utilisée par les essais Rust et Chrome. Elle est une sonde, pas une nouvelle leçon officielle.

## Résultats exécutés

Exécution sur GitHub Actions, Ubuntu 24.04, Rust 1.99 et Node 22, à partir du commit `5516a09767782e3b529406e8b6271df9ac3d74b1` (la CI vérifie sa fusion virtuelle dans la branche de la PR #177 ; elle ne fusionne aucune branche réelle).

[Journaux du run 37745368544](https://github.com/yocthanmabeka/Oyo-Oyo/actions/runs/37745368544).

| Commande ou contrôle | Résultat lu dans les journaux |
|---|---|
| `cargo test` (debug) | 178 réussis, 0 échec ; les cinq nouveaux tests sont exécutés. |
| Compilation WebAssembly complète, en release | Réussie. |
| Compilation WebAssembly légère, sans les fonctions de dessin, en release | Réussie. |
| `cargo build --release --bin holo` | Réussie ; ce binaire sert le parcours Chrome sur HTTP. |
| Vérification de structure de la suite de conformité | 22 cas (6 acceptés, 16 refusés) ; ce contrôle ne teste pas à lui seul la conformité du moteur. |
| `node outils/browser-tests.mjs` | 37 parcours réussis, 0 échec, 101 leçons ouvertes ; 274 s pour cette exécution. |

Le nouveau parcours rapporte :

```text
réservation sans JavaScript ; seconde forgée refusée (409) ; cart=777 et note=Eve ignorés ;
état du compte intact après rechargement ; toucher et saisie normaux gardés ;
miroir retardé : ordre et panier gardés
```

Commit de fusion virtuelle effectivement testé : `ad8a10e0a2e3266bd7bab3a207e4a01addaabba9`. Les corrections du moteur sont vérifiées par cette exécution ; les résultats ne sont pas des mesures sur le téléphone.

Poids des `.wasm` bruts lus dans ce run : complet 3 671 486 octets, léger 636 038 octets. Ils sont identiques à la première exécution de cette correction du serveur. Ce ne sont ni des poids compressés transférés par HTTP ni des mesures de mémoire.

Le premier passage Chrome (run 37744480404) avait donné 36 parcours réussis et un échec dans ma sonde : compteur absent de la page d'essai, et texte ajouté au nom au lieu de le remplacer. La sonde a été corrigée, puis la suite entière ci-dessus a été réexécutée. Les défauts initiaux du serveur sont établis par lecture du commit de base ; je ne présente pas une exécution de ce commit sans correction comme effectuée.

`cargo test --release` n'a pas été exécuté dans cette session : la CI existante lance `cargo test` en debug. Le vrai serveur en release est en revanche compilé et exercé par le parcours HTTP. Le contrôle en release demandé par la tâche et l'essai sur le PC de Yocthan restent à faire avant intégration.

Le terminal local ne démarre pas (`helper_unknown_error: setup refresh had errors`). La revue automatique d'une tentative d'accès local n'a pas pu aboutir à cause de la limite d'utilisation. Aucun résultat du PC de Yocthan ou de son téléphone n'est revendiqué.

## Objections et limites

- Un visiteur sans compte fournit toujours son état personnel au protocole JSON d'ADR-079. Cette correction ne promet aucune identité ni réservation unique pour une personne anonyme. Un site qui exige une réservation par compte utilise `access: members`.
- Un auteur qui rend une valeur sensible modifiable par un champ, ou par une règle déclenchée par ce champ, l'autorise lui-même. La correction ne remplace pas les règles métier du site.
- Un miroir qui échoue ou dépasse dix secondes est encore ignoré par la page, comme avant : la synchronisation du compte n'est pas garantie hors connexion. Il ne bloque plus la file sans fin.
- Le serveur garde les gestes des membres qu'il reçoit. Le temps, certaines touches et les glissements ne sont toujours pas rejoués : dette déjà annoncée par la PR #177.
- Le verrou des gestes est global au site, dans cette première correction. Il favorise la justesse et peut limiter le débit ; aucune mesure de débit n'est annoncée. Une file par compte/adresse demande une expérience séparée.
- L'atomicité face à une panne SQLite ou à plusieurs processus serveur n'est pas ajoutée. Les écritures existantes ignorent encore certains échecs. Le verrou protège les demandes d'une même instance ; il n'est pas une transaction SQLite.
- La compatibilité avec les modifications concurrentes de la session Claude du nuage n'est pas supposée. La PR vise la branche de comptes de Claude pour isoler sa correction ; seul Claude PC peut intégrer puis fusionner.

## Remise et suite

Fichiers de code modifiés : `moteur/src/server.rs`, `moteur/web/page-engine.js`, `moteur/outils/browser-tests.mjs`. Fichiers propres à Codex : ce compte rendu et `concert.holo`.

Claude PC doit relire cette correction, l'intégrer à la PR #177, reprendre `main` dans sa branche et exécuter ses contrôles avant une éventuelle fusion. Le journal et le guide pourront alors préciser l'autorité du compte ; leurs mises à jour restent à Claude. Aucun fichier transversal n'est modifié ici.

Après cette jonction viennent les dettes de partage (#182), les parcours du web (#183/#188), puis les dettes des comptes (#180). Les clés d'accès (#181) attendent le choix explicite de leurs dépendances. La mesure sur le Samsung et les tâches de la session du nuage restent distinctes.
