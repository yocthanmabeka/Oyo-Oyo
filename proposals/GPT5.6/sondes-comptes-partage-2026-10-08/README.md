# Comptes et valeurs partagées : sondes avant validation

## Contribution

- Auteur : Codex, 2026-10-08.
- Sujet : vérifier que le serveur conserve son autorité sur les gestes d'un membre.
- Source : demande de Yocthan (« Travaille sur la partie restante », puis « Continue »), HC-013, ADR-015, ADR-074, ADR-079, ADR-081.
- Statut proposé : EXPÉRIMENTATION. Aucun statut de décision modifié.
- Coordination : [issue #197](https://github.com/yocthanmabeka/Oyo-Oyo/issues/197).
- Code examiné : [PR #177](https://github.com/yocthanmabeka/Oyo-Oyo/pull/177), commit `9522483e0d58ba4f86895f7021c208b95c3fd7fa`. Une correction ultérieure peut changer le résultat.
- Périmètre : ce dossier uniquement ; les lots de Claude restent à leurs auteurs.

## Résultat

La contribution apporte une scène et cinq sondes HTTP, sans bibliothèque ajoutée. Elles testent le serveur natif sur un site temporaire, avec des comptes créés pour l'essai.

**Deux défauts sont déduits du code ; ils ne sont pas encore reproduits par HTTP dans cette session.**

| Point | Ce que le code montre | Scénario à éprouver | Correction recommandée |
|---|---|---|---|
| R-01, priorité haute | `server.rs:423` appelle `share` avec `state` reçu du navigateur ; le compte ne fait que choisir la clé de sauvegarde. | Après une réservation, le membre renvoie `booked=0` : la condition « une place par compte » pourrait être contournée. | Charger l'état du compte sur le serveur, appliquer uniquement les saisies autorisées, puis arbitrer avec les valeurs partagées courantes. |
| R-02, priorité haute | `server.rs:429–432` remplace et sauvegarde l'état personnel même lorsque `accepted=false`. | Un bouton déjà caché est refusé, mais sa demande porte `cart=777` ; la valeur pourrait apparaître après rechargement. | Un geste refusé doit laisser l'état personnel gardé intact ; traiter les saisies valides par un contrat séparé et explicite. |

Sources figées : [server.rs](https://github.com/yocthanmabeka/Oyo-Oyo/blob/9522483e0d58ba4f86895f7021c208b95c3fd7fa/moteur/src/server.rs#L407), [shared.rs](https://github.com/yocthanmabeka/Oyo-Oyo/blob/9522483e0d58ba4f86895f7021c208b95c3fd7fa/moteur/src/shared.rs). Les valeurs partagées sont remplacées par celles du serveur, mais les valeurs personnelles du JSON servent encore aux conditions. Authentifier un compte ne rend pas son état envoyé fiable.

La relecture rejoint le principe de [validation côté serveur de l'OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html). Vérifier le format et les bornes d'un nombre ne prouve pas que le visiteur a le droit de le modifier.

## Sondes livrées

| Sonde | Exigence vérifiée |
|---|---|
| P01 | Une page réservée et sa source ne se lisent pas sans session. |
| P02 | Deux formulaires ordinaires du même compte ne réservent qu'une place. |
| P03 | Le même compte ne réserve pas une deuxième fois en forgeant son état JSON. |
| P04 | Une demande refusée ne change pas le panier enregistré. |
| P05 | Une valeur partagée forgée dans le JSON ne remplace pas celle du serveur. |

P01, P02 et P05 servent aussi de contrôles : une préparation incorrecte ne doit pas être interprétée comme la preuve de R-01 ou R-02. Sur le code examiné, j'attends un échec de P03 et P04 ; ce sont des prédictions, pas des résultats obtenus.

Le script vérifie la scène avec `holo check`, choisit un port disponible, démarre son propre `holo serve`, vérifie un marqueur unique avant toute création de compte et arrête ce processus à la fin. Le serveur 8080 et les données du site de Yocthan ne sont pas utilisés. Le serveur natif écoute actuellement toutes les interfaces ; les requêtes des sondes visent seulement 127.0.0.1. Les comptes et la base temporaires sont supprimés après l'arrêt ; un nettoyage raté conserve le dossier et le signale.

## Expérience ou preuve requise

Avec Node.js 20+ et un binaire construit depuis la PR 177 ou sa version corrigée, lancer depuis la racine du dépôt :

```powershell
node proposals/GPT5.6/sondes-comptes-partage-2026-10-08/probes.mjs C:\chemin\vers\holo.exe
```

Remplacer le chemin du binaire ; ne pas lancer sur une base réelle. Pour éprouver le commit exact, utiliser un dossier et une branche indépendants ; ne pas déplacer le dossier principal sur la branche de la PR.

La sortie JSON rapporte chaque assertion. Code de sortie : 0 si les cinq sondes réussissent, 1 si une exigence échoue, 2 si la préparation ou le nettoyage échoue. Conserver la sortie brute et le SHA du binaire examiné, puis relancer après correction. Les essais habituels du moteur et du navigateur restent requis avant relecture.

## Vérifié et non vérifié

- Effectué : lecture de la passation, des règles sur GitHub, des cinq documents de cadrage et du code figé cité.
- Exécuté dans l'outil JavaScript : contrôle de syntaxe du corps du script, après retrait des imports et substitution d'`import.meta.url` ; quatre cas du lecteur de nombres HTML, dont le refus d'une valeur absente.
- Non exécuté : Node.js, `holo check`, sondes HTTP, compilation Rust/WebAssembly, suite Chrome, téléphone. Ces contrôles JavaScript limités ne valent pas une exécution du script dans Node.js.
- Blocage : le terminal ne démarre pas ; l'approbation automatique de son lancement échoue sur le quota. Aucun résultat de test de Claude n'est présenté comme un résultat obtenu par Codex.

## Objections, limites et suite

Ces sondes HTTP couvrent l'autorité du serveur, pas les gestes au doigt, TalkBack, les codes TOTP, les clés d'accès ou la batterie. Elles visent le contrat de la PR 177 ; un nouveau protocole peut demander leur adaptation.

Le dossier ne corrige pas encore le moteur. Claude pourra reproduire R-01 et R-02, intégrer la correction dans son lot et relancer les sondes. Après ces preuves, mettre à jour les dettes des ADR-079/081 et le compte rendu de validation, sans déclarer le métavers terminé.

Ordre recommandé : 1. reproduire P03/P04 ; 2. corriger l'autorité et la sauvegarde ; 3. obtenir les cinq sondes vertes et la suite normale ; 4. reprendre les fonctions restantes des comptes, puis les mesures sur téléphone.
