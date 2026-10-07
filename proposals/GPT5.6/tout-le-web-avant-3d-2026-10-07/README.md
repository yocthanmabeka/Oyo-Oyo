# Revue GPT-5.6 — payer les dettes du web avant la 3D

- Auteur : GPT-5.6, pour Yocthan Mabeka
- Date : 2026-10-07
- Source examinée : `main` au commit `9df3b2b99972949fd31a47e994111a9ef60b5bea`
- Discussions sources : prompt du 2026-10-07 sur « tout le web », revue `web-assez-utilisable-2026-10-07`, proposition serveur et comptes de Claude
- Décisions concernées : `ADR-005`, `ADR-008`, `ADR-013`, `ADR-015`, `ADR-033`, `ADR-035`, `ADR-045`, `ADR-048`, `ADR-053`, `ADR-055`, `ADR-059`, `ADR-061`
- Statut proposé : **PROPOSITION / REVUE**
- Aucun statut de décision n'est changé.

## 1. Peut-on atteindre « tout le web » sans code libre ?

**Non, pas littéralement avec une liste fermée de mots et les modules actuels “un nombre entre, un nombre sort” : le web invente sans cesse de nouvelles capacités et certains auteurs auront besoin d'algorithmes que le moteur ne connaît pas encore.** **Oui, HoloCode peut couvrir toutes les grandes familles utiles si le langage garde un registre public de ses dettes et offre une extension générale mais enfermée : des modules typés, limités et sans accès direct à la page, au réseau ou à l'appareil.**

La condition importante est donc la suivante : HoloCode peut refuser le JavaScript libre, mais il ne peut pas refuser la possibilité d'écrire un calcul nouveau. Il doit donner cette possibilité dans une boîte sûre.

---

## 2. Les lots, dans l'ordre

Une **séance** signifie ici un cycle court : décider un petit contrat, le construire, écrire les tests et l'essayer. Les chiffres sont des estimations, pas des promesses.

### Lot 1 — Fermer les trous dans les tests et les limites — 3 à 4 séances

**À la fin :** une modification du navigateur ne peut plus casser silencieusement le pincement, les modules, les formulaires ou le clavier ; un fichier ou une liste trop gros est arrêté avant d'épuiser la mémoire.

**Contenu (7 éléments) :**

1. tests automatiques dans un vrai navigateur pour chaque modification de `moteur/web/` ;
2. test du pincement à deux doigts ;
3. test d'un module WebAssembly ;
4. limites d'octets, de blocs, de profondeur et d'éléments de liste ;
5. délai maximal et annulation des lectures réseau ;
6. budget réel mesuré, au lieu de croire seulement `weight:` ;
7. même vérification dans l'éditeur, `holo check` et le serveur.

**Moyen :** moteur et outils de test. Aucun nouveau mot du langage, sauf si une limite doit être déclarée par l'auteur.

**Pourquoi en premier :** le journal du 7 octobre montre deux régressions graves après le passage du code en anglais : le pincement et les modules ne marchaient plus, alors que les tests Rust restaient verts. Ajouter des capacités sur cette base multiplierait les pannes invisibles.

### Lot 2 — Données, recherche et calcul utiles — 6 à 8 séances

**À la fin :** un catalogue peut charger des produits, afficher « chargement » ou « échec », chercher un mot, filtrer, trier et montrer 20 résultats par page ; un robot de recherche reçoit déjà le catalogue dans le HTML.

**Contenu (10 éléments) :**

1. identifiant stable choisi par l'auteur pour chaque élément ;
2. recherche de texte ;
3. filtre ;
4. tri avec ordre stable ;
5. pagination ;
6. états `loading`, `ready`, `empty`, `failed` ;
7. rendu serveur des données locales pour les moteurs de recherche ;
8. nombres décimaux exacts pour mesures et pourcentages ;
9. fonctions de texte ;
10. dates, durées et fuseaux horaires explicites.

**Moyen :** fonctions pures du moteur, plus quelques mots courts autour de `Data` et `Repeat`.

Une **fonction pure** est un calcul qui donne toujours la même sortie pour la même entrée et ne modifie rien ailleurs. Pour l'argent, continuer à recommander les centimes entiers. Pour les autres valeurs, ajouter un nombre décimal exact, pas un nombre flottant binaire imprécis. Pour le texte, compter les caractères visibles, pas les octets : un emoji peut occuper plusieurs morceaux en mémoire.

### Lot 3 — Formulaires vraiment sûrs et accessibles — 4 à 6 séances

**À la fin :** une inscription, une recherche ou une commande explique chaque erreur, fonctionne avec Entrée, n'est jamais envoyée deux fois et réagit proprement quand le serveur est lent.

**Contenu (8 éléments) :**

1. champ obligatoire ;
2. adresse e-mail ;
3. longueur minimale et maximale ;
4. message près du champ ;
5. annonce au lecteur d'écran ;
6. envoi par Entrée ;
7. identifiant d'envoi pour empêcher les doublons ;
8. délai maximal, annulation et nouvel essai.

**Moyen :** mots du langage pour les contraintes, moteur pour la vérification, serveur pour refaire exactement la même vérification.

Ajouter dans le même lot `Fieldset`, `Legend` et les suggestions d'un champ. Ce ne sont pas des décorations : ils rendent les formulaires longs compréhensibles.

### Lot 4 — Une vraie disposition adaptative — 5 à 7 séances

**À la fin :** le même composant fonctionne dans une grande page, une colonne étroite ou une fenêtre partagée, sans supposer qu'un téléphone fait toujours 640 pixels.

**Contenu (8 éléments) :**

1. `Row` qui peut revenir à la ligne ;
2. grille automatique avec largeur minimale d'une carte ;
3. variantes selon la place réellement reçue par un composant ;
4. proportions d'une image ou vidéo ;
5. ce qui dépasse : montrer, couper ou faire défiler ;
6. tailles minimales et maximales ;
7. curseur et texte justifié ;
8. flou, filtres et découpes bornés.

**Moyen :** blocs de disposition et choix nommés du moteur, pas propriétés CSS libres.

Je remplacerais le seul état `phone:` par des états liés au **conteneur**, c'est-à-dire la boîte qui donne sa place au composant : par exemple `compact:` et `wide:`. Le moteur choisit selon l'espace disponible, pas selon le nom de l'appareil. Pour une grille, une écriture fondée sur la taille souhaitée d'une carte est plus durable qu'un seuil fixé une fois pour toutes.

### Lot 5 — `holo serve`, adresses et base locale — 6 à 9 séances

**À la fin :** `holo serve mon-site/` lance sur le PC un vrai site avec des adresses comme `/profil/123`, des pages fabriquées sur le serveur, des formulaires, des fichiers et des données gardées dans SQLite.

**Contenu (8 éléments) :**

1. serveur Rust ;
2. routes, c'est-à-dire adresses avec paramètres ;
3. base SQLite ;
4. transactions, c'est-à-dire changements enregistrés entièrement ou pas du tout ;
5. évolution de la structure de la base ;
6. sauvegarde et restauration ;
7. journaux d'erreur et page d'erreur ;
8. rendu HTML des données avant l'arrivée du navigateur.

**Moyen :** serveur de l'auteur et même moteur Rust des deux côtés.

Ce lot respecte entièrement « chez soi d'abord ». SQLite est un fichier local. Aucun compte Google, hébergeur ou service d'e-mail n'est nécessaire.

### Lot 6 — Valeurs partagées et direct — 5 à 8 séances

**À la fin :** Ada réserve un tableau ; Bob voit immédiatement qu'il est réservé ; un geste répété par erreur n'est appliqué qu'une fois.

**Contenu (8 éléments) :**

1. `shared:` ;
2. numéro unique par geste ;
3. ordre décidé par le serveur ;
4. une seule application de chaque geste ;
5. règle vérifiée à nouveau sur le serveur ;
6. diffusion en direct par WebSocket ;
7. reprise après coupure ;
8. limites de fréquence et de taille.

**Moyen :** serveur faisant autorité et mots du langage pour déclarer la portée des valeurs.

Un **WebSocket** est une connexion qui reste ouverte pour que serveur et navigateur puissent se parler immédiatement dans les deux sens. Je le construirais dès ce lot : promettre à Bob une mise à jour dans la seconde sans choisir le canal de diffusion reporte seulement le même travail.

Je ne garderais pas la restriction de Claude « un partage peut venir d'un toucher, jamais du clavier ». Le clavier n'est ni plus ni moins fiable qu'un toucher : les deux viennent d'un navigateur que le serveur ne doit pas croire. Le serveur doit vérifier la demande et les droits, pas la forme du geste.

### Lot 7 — Comptes, droits et données personnelles — 7 à 10 séances

**À la fin :** un visiteur peut créer un compte local, se connecter, retrouver son panier sur un autre appareil, activer une deuxième preuve de connexion, puis télécharger ou effacer ses données.

**Contenu (10 éléments) :**

1. `mine:` ;
2. compte et session ;
3. mot de passe stocké par empreinte Argon2id, un calcul lent conçu pour protéger les mots de passe ;
4. code temporaire TOTP, c'est-à-dire six chiffres renouvelés toutes les 30 secondes ;
5. clés d'accès ensuite ;
6. droits par action et par objet ;
7. limitation des essais ;
8. réinitialisation locale journalisée ;
9. suppression et export des données ;
10. cookies de session sûrs.

**Moyen :** serveur et vocabulaire déclaratif de droits. Ne jamais déléguer la décision des droits au navigateur.

Une **empreinte** est le résultat irréversible utilisé pour vérifier un mot de passe sans le conserver. La proposition Claude dit « mot de passe chiffré » : le bon mot est **haché**, avec Argon2id et un sel aléatoire. Un **cookie de session** est un petit identifiant envoyé automatiquement au serveur ; il doit être inaccessible au JavaScript de la page, limité au site et envoyé seulement sur une connexion sûre en ligne.

### Lot 8 — Capacités larges, médias et hors ligne — 8 à 12 séances

**À la fin :** HoloCode possède une réponse générale pour les dessins, les calculs nouveaux et les appareils, sans donner au module les clés de toute la page.

**Contenu (10 éléments) :**

1. modules acceptant texte, booléens, nombres, listes et éléments à champs ;
2. retour de ces mêmes types ;
3. dessin vectoriel ;
4. zone de dessin par liste bornée de commandes ;
5. particules bornées ;
6. position du pointeur ;
7. caméra, position, vibration, presse-papiers et partage avec permission ;
8. sous-titres et image d'attente d'une vidéo ;
9. chargement tardif des images et sons ;
10. site hors ligne, avec mise à jour sûre.

**Moyen :** fonctions pures, modules WebAssembly typés et courtier de permissions du moteur.

Un **courtier de permissions** est la partie du moteur qui demande l'accord du visiteur et ne donne au fichier que la capacité précise autorisée. Un module de dessin ne reçoit pas le vrai Canvas du navigateur : il rend une liste bornée comme « ligne de A à B, couleur rouge ». Le moteur vérifie la quantité, puis dessine. Le module ne reçoit ni réseau, ni page, ni fichiers, ni heure, ni hasard caché.

### Ce qui est nécessaire avant la 3D

- Lots 1 à 4 : oui, sans discussion ; ce sont les fondations d'un web viable.
- Lot 5 : oui ; sinon la 3D commencera au-dessus d'un serveur d'essai à jeter.
- Lot 6 : oui au moins jusqu'à la réservation partagée et à la reprise après coupure ; c'est aussi la fondation du métavers à plusieurs.
- Lot 7 : oui pour comptes, sessions et droits de base si HoloCode prétend couvrir les applications web ; les clés d'accès peuvent arriver juste après la première version mot de passe + TOTP.
- Lot 8 : faire avant la 3D le contrat des modules typés, le dessin vectoriel, les sous-titres et le chargement tardif. Caméra, géolocalisation, vibration, partage et vrai hors-ligne peuvent attendre le premier objet 3D, car ils ne changent pas l'architecture de l'arbitre s'ils passent par le courtier.

### Ce que je refuserais franchement

Je refuserais les **mécaniques** suivantes, pas leurs capacités :

- JavaScript libre dans la page ;
- modification directe du HTML ;
- cascade CSS générale et `!important` ;
- accès brut au Canvas, au GPU, à la caméra ou au réseau depuis un module ;
- requêtes SQL écrites dans `.holo` ;
- cryptographie inventée par l'auteur ;
- autorisation décidée par un bouton caché ;
- paiement fabriqué par HoloCode lui-même.

Pour le paiement, HoloCode doit donner un contrat commun, mais l'argent réel passera nécessairement par une banque ou un prestataire choisi par l'auteur. « Chez soi d'abord » doit couvrir le développement et un faux paiement local ; cela ne peut pas supprimer le système financier extérieur.

---

## 3. Réponses courtes aux autres questions

### Question 2 — Quelles dettes avant la 3D ?

Avant la 3D : données, formulaires, vraie adaptation des pages, tests navigateur, serveur local, routes, valeurs partagées, comptes et droits de base. Après une première tranche 3D : appareils, paiement réel, fonctions HTML rares, dessin libre très avancé et hors-ligne complet. Ne refusez que les mécaniques dangereuses listées plus haut ; chaque capacité conserve une réponse ou une dette visible.

### Question 3 — Dans quel ordre ?

L'ordre est celui des huit lots. Il va du plus transversal au plus risqué : d'abord voir les pannes, ensuite manipuler les données, ensuite recevoir correctement les entrées, puis disposer la page, puis seulement garder et partager les valeurs sur un serveur.

### Question 4 — Le serveur et les comptes

La direction est bonne et respecte « chez soi d'abord », mais quatre phrases de la proposition Claude sont trop optimistes :

1. « même arbitre » ne garantit pas à lui seul l'absence de divergence ; il faut aussi même version, même fichier, même ordre de gestes et mêmes entrées de temps et de hasard ;
2. « un geste, pas une valeur » ne suffit pas : un visiteur peut envoyer `Reserve.tap` mille fois ou pour un objet qu'il n'a pas le droit de réserver ;
3. « quelques microsecondes » et « des milliers de demandes par seconde » ne sont pas mesurés dans le dépôt ;
4. « mot de passe chiffré » est incorrect : il doit être haché.

Je ferais `holo serve` tôt, après les lots données, formulaires et disposition. Je ferais `shared:` avant les comptes, puis les comptes avant la 3D publique. SQLite est un excellent premier choix local, à condition d'ajouter transactions, évolution de schéma et sauvegardes.

### Question 5 — Les larges capacités de JavaScript

Oui, élargir les modules enfermés, mais par types et par capacités précises :

- texte, nombres décimaux, booléens, listes et éléments à champs en entrée et sortie ;
- mémoire et durée maximales ;
- compteur d'instructions ;
- aucune entrée-sortie cachée ;
- format de module versionné ;
- dessin rendu comme commandes vérifiées ;
- appareil fourni uniquement par le courtier de permissions ;
- résultat de l'appareil transformé en geste enregistré pour pouvoir rejouer.

Il faut aussi un moyen plus simple que les modules pour 95 % des besoins : fonctions pures intégrées pour chercher, trier, formater, découper un texte et calculer une date.

### Question 6 — La mise en page

Ne copiez ni les 500 propriétés du CSS, ni le seuil unique `phone`. Donnez des blocs qui expriment l'intention : ligne avec retour, grille avec taille minimale de carte, pile, zone qui défile, proportions ; et des états `compact` / `wide` décidés par la place réellement reçue. Le moteur peut toujours produire du Flexbox, du Grid et des requêtes de conteneur CSS en dessous, mais l'auteur ne dépend pas de leur mécanique.

### Question 7 — Les risques

Voir la section suivante. Le point le plus important : « un même fichier donne toujours le même résultat » est faux dès qu'une page lit l'heure, un serveur, une caméra ou un autre visiteur. La garantie correcte est : **même fichier, même version du moteur, même état initial et même journal ordonné des entrées donnent le même résultat.**

### Question 8 — Ce qui est oublié ou mal compté

Le chiffre **77 %** est utile comme impression, pas comme mesure. Les 130 lignes n'ont pas le même poids, la sélection oublie de grandes familles, et le résumé compte 89 « oui » + 21 « en partie » + 13 « non » + 6 « refusés » = **129**, pas 130.

Capacités absentes ou insuffisamment visibles dans la liste :

- droits et rôles, distincts des comptes ;
- adresses avec paramètres, requête et historique ;
- référencement complet : adresse canonique, plan du site, règles pour robots ;
- plusieurs langues et formats régionaux ;
- notifications ;
- impression et création de fichiers à télécharger ;
- tâches longues en arrière-plan ;
- envoi d'e-mail et webhooks, c'est-à-dire appel d'un autre serveur après un événement ;
- import/export, sauvegarde, restauration et évolution des données ;
- journaux, mesures et diagnostic en production ;
- flux audio/vidéo en direct et WebRTC, le mécanisme du navigateur pour relier directement deux appareils ;
- annuler/refaire ;
- gestion des conflits quand deux personnes changent le même objet ;
- politique de confidentialité, durée de conservation et consentement ;
- mises à jour du site hors ligne sans garder une ancienne page cassée.

Je ne transformerais pas chacune en mot HoloCode. Plusieurs appartiennent au serveur, aux outils ou à un futur ensemble de composants standard.

---

## 4. Les risques principaux

### 4.1 Le projet peut ne jamais atteindre « 100 % »

Le web change chaque année. Si « aucune dette avant la 3D » signifie zéro ligne non couverte, la 3D ne commencera jamais. Je fixerais un seuil falsifiable : les lots 1 à 7 terminés, un catalogue, un compte, une réservation partagée et un site adaptatif construits uniquement en HoloCode, passés sur téléphone et ordinateur, avec cinq débutants.

### 4.2 Le vocabulaire peut exploser

333 mots en quatre jours est déjà beaucoup pour un débutant. Chaque capacité ne doit pas devenir un mot. Règle proposée : mot seulement pour une intention fréquente ; fonction pure pour un calcul ; module pour un algorithme rare ; serveur pour l'autorité et les secrets.

### 4.3 Le déterminisme peut devenir un mensonge

Le temps, le hasard, le réseau et l'appareil doivent être des entrées explicites données à l'arbitre et écrites dans le journal. Le hasard doit être produit par le serveur lorsqu'il influence une valeur partagée. Les nombres flottants ordinaires peuvent varier ou accumuler des erreurs : employer des entiers ou des décimaux exacts dans l'arbitre.

### 4.4 « Même code des deux côtés » ne suffit pas pour la sécurité

Le serveur doit posséder l'état et les droits. Chaque demande doit vérifier : qui agit, sur quel objet, avec quelle permission, dans quelle version, après quel geste et avec quelle limite. Cacher un bouton ou refuser un mot dans le navigateur n'est jamais une permission.

### 4.5 Les comptes sont un chantier de sécurité, pas un bloc de formulaire

Risques : vol de session, essais massifs de mots de passe, récupération de compte, fuite de la base, code TOTP volé, clé d'accès perdue, administrateur local trop puissant. Utiliser des bibliothèques Rust auditées, ne pas inventer la cryptographie, journaliser les commandes d'administration et faire une revue de menace avant Internet.

### 4.6 Le poids peut remonter très vite

Des dates, du texte international, SQLite, la cryptographie, le hors-ligne et les appareils peuvent gonfler le moteur. Garder plusieurs paquets chargés à la demande : page plate, interaction, dessin, appareils, serveur. Mesurer chaque lot en octets transférés, mémoire maximale et temps de démarrage sur le téléphone modeste visé par `ADR-005`.

### 4.7 L'accessibilité peut casser après le premier affichage

L'audit automatique ne prouve pas que le focus, les erreurs et les changements dynamiques sont compréhensibles. Chaque lot doit tester clavier, lecteur d'écran, zoom du texte, contraste, mouvement réduit et toucher. Le formulaire doit annoncer l'erreur ; une liste triée doit garder un focus logique ; un Dialog doit rendre le focus à son bouton d'origine.

### 4.8 Le rendu serveur et le navigateur peuvent montrer deux vérités

Si le serveur fabrique un catalogue à 10 h 00 et que le navigateur charge des données à 10 h 01, le HTML et l'état peuvent différer. Ajouter une version de données au HTML ; le navigateur reprend cette version puis applique seulement les mises à jour suivantes.

### 4.9 Le hors-ligne peut garder une vieille application

Un service worker, c'est-à-dire le petit programme du navigateur qui garde les fichiers hors ligne, peut servir un ancien moteur avec un nouveau fichier `.holo`. Les fichiers, le moteur et le contrat des modules doivent porter une version compatible et être remplacés ensemble.

### 4.10 Les capacités de l'appareil menacent la vie privée

La caméra et la position ne doivent jamais partir au chargement de la page. Elles exigent un geste clair, l'accord du visiteur, un témoin visible pendant l'usage, une durée courte et aucun envoi réseau implicite.

---

## 5. Ce que je ferais cette semaine

1. Ajouter les tests navigateur qui auraient détecté les pannes du pincement et des modules.
2. Écrire le contrat du lot données : recherche, filtre, tri, pagination, identifiant stable, chargement et échec.
3. Construire ce lot avec un vrai catalogue rendu aussi par le serveur.
4. Ajouter la validation complète des formulaires et l'empêchement des doubles envois.
5. Mesurer sur PC et téléphone, puis décider l'API de disposition adaptative avant `holo serve`.

---

## Résultats exécutés dans cette revue

Les commandes suivantes ont été lancées dans la copie exacte du commit indiqué plus haut :

| Commande | Résultat exact |
|---|---|
| `cd moteur && cargo test` | non lancé : `/bin/bash: cargo: command not found`, code 127 ; rien n'a été installé |
| tests Python GPT5.6 | 9 tests : 8 réussis, 1 échec volontaire car Cargo est absent ; code 1 |
| tests Python Claude | 27 réussis ; code 0 |
| tests Python Gemini | 8 réussis ; code 0 |
| suite de conformité | `22 cas : 6 acceptés, 16 refusés` puis `Suite bien formée.` ; code 0 |

Le rapport Claude `proposals/Claude/exploration-web-complet-2026-10/` n'était pas présent dans ce commit. Je n'en suppose donc pas le contenu.

## Objections et limites

- Les estimations de séances sont les miennes et doivent être révisées après le premier lot.
- Cargo manque dans cet environnement : aucun des 124 tests Rust annoncés par le journal n'a été relancé ici.
- Je n'ai pas piloté un téléphone ni un lecteur d'écran dans cette revue.
- Une partie des capacités oubliées peut appartenir à une bibliothèque standard future plutôt qu'au cœur du langage.

## Expérience ou preuve requise

Avant de déclarer le web « viable », construire quatre sites de preuve sans JavaScript libre :

1. catalogue avec recherche, filtre, tri, pagination et données rendues par le serveur ;
2. formulaire accessible avec fichier, validation, délai et reprise ;
3. réservation partagée entre deux navigateurs concurrents ;
4. compte local avec deuxième preuve de connexion et droits différents.

Les quatre doivent fonctionner sur ordinateur et téléphone, clavier et lecteur d'écran, réseau lent et coupé. Mesurer octets, mémoire, premier affichage, première interaction et temps serveur.

## Documents à mettre à jour si Yocthan retient cette direction

- `docs/01-holocode/TABLEAU-WEB.md` : corriger 130/129 et distinguer couverture, dette et refus ;
- nouvelle ADR : critère mesurable de « web viable » ;
- proposition serveur : hachage Argon2id, droits, versions, transactions, sauvegardes et WebSocket au lot partagé ;
- `docs/04-roadmap/` : huit lots et porte d'entrée vers la 3D ;
- suite de conformité : déterminisme défini par fichier + moteur + état + journal d'entrées.
