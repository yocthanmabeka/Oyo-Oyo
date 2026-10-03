# Communication entre les IA

## Dispositif minimal

1. Yocthan ou Claude ouvre une **issue de tâche** avec un identifiant, un résultat attendu, les chemins réservés et la preuve exigée.
2. L'IA qui travaille ajoute un commentaire `CLAIM` et crée une branche contenant l'identifiant.
3. Toute réponse durable est une PR liée à l'issue. Une conversation seule ne vaut pas modification du projet.
4. Claude relit l'intégration technique ; Codex relit cohérence/architecture ; Gemini peut être sollicité dans la PR via Gemini Code Assist. Yocthan garde le dernier mot.
5. Une IA sans GitHub reçoit un fichier exporté qui contient l'issue, le SHA de base et les fichiers concernés. Sa réponse revient comme pièce jointe ou fichier dans une PR, avec son auteur déclaré.

## Boîte aux lettres

Pas de dossier contenant une file de messages qui grossit indéfiniment. Les **issues GitHub sont la boîte aux lettres** : elles ont un historique, des notifications, des étiquettes et un état ouvert/fermé. Le dépôt ne conserve que les décisions, propositions et preuves qui doivent survivre.

Étiquettes proposées :

- `ia:claude`, `ia:codex`, `ia:gemini`, `ia:externe` : auteur ou destinataire ;
- `etat:a-prendre`, `etat:en-cours`, `etat:revue`, `etat:bloque` ;
- `zone:langage`, `zone:moteur`, `zone:preuve`, `zone:organisation` ;
- `risque:conflit`, `telephone-requis`.

## Empêcher deux IA de modifier le même fichier

L'issue contient une liste `paths:`. Une IA ne commence que si ces chemins ne sont pas revendiqués dans une autre issue `etat:en-cours`. Si un chevauchement apparaît, la seconde IA commente et attend ; elle ne crée pas une version concurrente silencieuse. Les fichiers transversaux (`AGENTS.md`, journal, ADR, workflow) restent réservés au mainteneur d'intégration sauf demande explicite.

## Empêcher une fusion non lue

La fusion exige quatre preuves visibles dans la PR :

- auteur GitHub et auteur IA ;
- branche source et cible écrites en toutes lettres ;
- tests réellement exécutés, avec commande et résultat ;
- commentaire final de Yocthan : `VALIDÉ POUR FUSION`.

Le numéro de PR seul n'identifie jamais une contribution. Le vérificateur exemple refuse une description sans ces champs, mais il ne fusionne rien.

## Automatisation gratuite, sans clé d'API d'IA

GitHub Actions peut : vérifier la présence des champs ; contrôler que la contribution reste dans les chemins annoncés ; lancer tests Rust/Python/conformité ; valider YAML/JSON/liens internes ; produire un artefact de captures ; annuler les exécutions dépassées avec `concurrency` ; ignorer les tests lourds pour une PR de documentation.

GitHub Actions ne doit pas : décider qu'une architecture est bonne, accepter une décision, appeler un modèle payant, ou fusionner automatiquement.

## Flous ou contradictions à corriger plus tard

- `AGENTS.md` attribue des rôles, mais pas l'exclusivité des fichiers ni le protocole de réservation.
- Le protocole prévoit une contribution par fichier, mais pas un identifiant de tâche commun entre issue, branche, PR et journal.
- La politique demande validation avant fusion, sans définir la chaîne exacte qui prouve cette validation.
- « Claude tient le journal » et « chaque contributeur documente ses tests » nécessitent une frontière : le contributeur écrit sa preuve dans la PR, Claude reporte seulement après validation.
- Le statut des commentaires d'une IA GitHub App doit être explicite : **avis**, jamais décision.
