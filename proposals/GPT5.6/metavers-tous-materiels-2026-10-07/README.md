# Holoverse de 1970 à 2026 : conserver le monde, adapter sa présentation

## Contribution

- Auteur : Codex, 2026-10-07.
- Sujet : un métavers accessible sur des générations de machines différentes.
- Discussions sources : [HC-013](../../../docs/05-discussions/HC-013-web-metavers-et-format-holo.md), [HC-011](../../../docs/05-discussions/HC-011-cadrage-big-bang.md), demande de Yocthan du 2026-10-07.
- Décisions concernées : ADR-007, ADR-008, ADR-009, ADR-010, ADR-015, ADR-048, ADR-053.
- Statut proposé : **PROPOSITION**. Aucun statut de décision modifié.
- Référence examinée : `main`, commit `69821a8ee9e7db8ffaab3d6bc4bc2dbfda83749d`.
- Coordination : [issue #151](https://github.com/yocthanmabeka/Metaverse/issues/151).

## Résultat

**Je propose un monde dont les lieux, les liens et les actions essentielles existent indépendamment de son dessin.** On peut entrer dans un point par un zoom, par un lien ou par un choix numéroté. L'identité du lieu reste la même. Une machine puissante le dessine en profondeur ; une machine modeste le décrit.

### Ce qui existe déjà dans ta vision

| Principe | Présent dans le dépôt ? | Mon ajout proposé |
|---|---|---|
| Une description, une page à plat et un lieu en profondeur | Oui : [VISION](../../../docs/00-vision/VISION.md), [ADR-007](../../../docs/02-gouvernance/adr/ADR-007-web-mis-a-jour-deux-vues.md) | Faire de la navigation sans dessin une obligation vérifiable. |
| Le monde naît d'une graine ; la source n'est pas un prompt | Oui : [ADR-008](../../../docs/02-gouvernance/adr/ADR-008-fichier-vraie-source.md) | Fixer aussi la version du générateur et les règles de calcul pour conserver ses identités. |
| Le résultat compte davantage que la technique de dessin | Oui : [ADR-048](../../../docs/02-gouvernance/adr/ADR-048-qualite-jamais-technique.md) | Définir les services minimums accessibles sans GPU, WebAssembly ni JavaScript. |
| Fonctionner sur toute génération de matériel | Pas comme engagement établi : la cible actuelle est le téléphone actuel | Ajouter cette ambition comme proposition, avec des limites explicites. |

La présentation en texte peut être une forme de la vue à plat. Je ne décrète pas une troisième vue officielle. Le programme natif avant le Web serait un ancêtre possible du projet ; aujourd'hui, le Web reste son accès principal.

### Sur quoi cela aurait pu tenir selon l'époque

Ce tableau décrit une **possibilité d'architecture**, pas un logiciel historique retrouvé ni une compatibilité testée. Il faut choisir une machine précise dans chaque période : toutes n'ont pas les mêmes capacités.

| Période | Forme de visite proposée | HTML / CSS / JavaScript possibles | Travail à prévoir |
|---|---|---|---|
| 1970–1979 | Terminal texte : nom du lieu, description, sorties numérotées ; dessin vectoriel seulement si le matériel le permet | Aucun : le Web n'existe pas encore | Petit programme natif, données paginées, état local ou calcul sur un ordinateur central. |
| 1980–1989 | Texte, plan en caractères, points ou contours sur une machine graphique | Aucun Web ; ni HTML, ni CSS, ni JavaScript | Lecteur natif adapté à la machine ; lecture par morceaux depuis son support disponible. |
| 1990–1994 | Lieux sous forme de pages reliées, illustrations facultatives | HTML simple : titres, paragraphes, listes, liens ; pas de CSS ni JavaScript | Pages préparées ou serveur compatible ; navigation d'abord, actions seulement avec un mécanisme réellement pris en charge. |
| 1995–1999 | Même base, formulaires et apparence facultative | HTML avec formulaires selon le navigateur ; JavaScript à partir de 1995, CSS1 à partir de 1996, avec prises en charge incomplètes | Réponse fabriquée par le serveur après une demande ; aucun script obligatoire. Une visite VRML serait une variante facultative exigeant un lecteur. |
| 2000–2009 | Pages utilisables, puis carte et interactions enrichies | HTML, CSS et JavaScript classiques selon les capacités ; pas de dépendance aux modules ou aux API modernes | Garder une navigation complète lorsque les enrichissements échouent. |
| 2010–2019 | Pages et visite graphique progressive | HTML5, CSS, JavaScript ; Canvas et WebGL sur les appareils compatibles | Charger le moteur graphique uniquement à la demande. |
| 2020–2026 | Page, points et objets en profondeur selon le matériel | HTML sémantique produit à l'avance ou par le serveur ; CSS ; petit JavaScript ; Rust/WebAssembly et WebGL2 ou WebGPU pour les vues compatibles | Ajouter un véritable parcours sans JavaScript pour les actions retenues ; mesurer chaque rendu séparément. |

Le Web est proposé en 1989 et son premier navigateur est écrit en 1990 ([CERN](https://home.web.cern.ch/fr/science/computing/birth-web/)). Les dates de JavaScript et CSS viennent de [MDN](https://developer.mozilla.org/en-US/docs/Glossary/JavaScript) et du [W3C](https://www.w3.org/Style/CSS20/history.html). Leur existence ne garantit pas leur prise en charge par chaque navigateur.

L'idée a des précédents. La proposition [VRML de 1994](https://www.w3.org/People/Raggett/vrml/vrml.html) décrit déjà des liens entre lieux et des représentations adaptées aux capacités. La valeur ajoutée recherchée ici est une source lisible par l'auteur, avec un accès ordinaire au contenu même sans moteur 3D.

### La même visite, concrètement

Un point ouvre une bibliothèque contenant un livre et un passage vers un atelier.

- Terminal : « Bibliothèque. 1 : lire le livre. 2 : entrer dans l'atelier. 0 : revenir. »
- HTML simple : un titre, le texte du livre et un lien vers l'atelier.
- Vue en profondeur : une pièce, le livre et un point ouvrant l'atelier.

Les trois représentations doivent retrouver les mêmes identifiants et contenus. Emprunter le livre doit produire la même demande logique, avec les mêmes droits et le même résultat. Un profil sans mécanisme d'action reste explicitement une visite en lecture seule.

L'auteur continue d'écrire du `.holo`. **HTML, CSS et JavaScript sont des sorties du moteur**, pas des langages supplémentaires exigés de lui. Sur un serveur web compatible, une action sans JavaScript peut utiliser un formulaire POST ; un lien GET sert à naviguer, jamais à acheter ou modifier un état. Ce parcours serveur complet est à construire et vérifier, pas à annoncer comme déjà disponible.

### Les règles qui rendraient cette ambition crédible

1. Séparer contenu, identité, règles et présentation. Réduire les ombres ou les détails ne doit pas modifier les droits, les prix ou les résultats d'une action.
2. Ne charger que le lieu demandé, ses objets utiles et un voisinage borné. Éviter toute animation permanente en mode texte ou page immobile ; permettre pause et réduction des mouvements.
3. Conserver graine, version du générateur, ordre des demandes et règles arithmétiques. Une machine trop limitée reçoit une description préparée ; elle ne tronque pas la graine et ne crée pas un autre monde.
4. Distinguer ce qui se régénère et ce qui doit être enregistré. Les messages, achats, droits et modifications persistantes ont besoin de stockage ; une graine ne les remplace pas.
5. Choisir selon les capacités et la préférence du visiteur, pas seulement l'année ou le modèle. Sans réseau, un monde préparé peut se visiter localement ; il ne promet pas une synchronisation à plusieurs.

## Objections et limites

**« Peu importe le matériel » doit signifier un accès adapté, pas un photoréalisme identique.** Il faut au minimum une machine capable de lire des données, présenter une information et recevoir un choix. La mémoire nécessaire ne peut pas être nulle ; les supports et lecteurs devront être précisés.

Le moteur Rust actuel n'est pas démontré sur les ordinateurs des années 1970 ou 1980. Un lecteur ancien distinct risque de contredire l'objectif d'un cœur unique de [l'ADR-010](../../../docs/02-gouvernance/adr/ADR-010-moteur-rust-deux-enveloppes.md). Je privilégie des descriptions préparées avant un second moteur complet. Cela réduit le calcul, mais introduit du stockage et ne démontre pas une génération autonome sur ces machines.

Pour prétendre que cette version aurait pu être fabriquée à l'époque, il faudrait utiliser des outils et techniques disponibles alors. La produire avec les outils de 2026 puis la lancer en émulation démontrerait seulement la compatibilité du résultat.

Un jeu fondé sur les réflexes ou une simulation physique continue ne devient pas équivalent simplement en ajoutant un menu. Chaque usage doit définir ses actions essentielles et ses limites. Le rendu 3D, la fluidité, la mémoire et la batterie restent à mesurer ; aucun pourcentage n'est démontré ici.

Je déconseille la vidéo distante comme accès minimum : elle déplace le coût vers un serveur et impose du réseau. Je déconseille aussi une 3D obligatoire. Le HTML préparé déjà prévu par [l'ADR-053](../../../docs/02-gouvernance/adr/ADR-053-moteur-leger.md) offre un meilleur point de départ, sans prouver à lui seul le fonctionnement de toutes les actions sans script.

## Expérience ou preuve requise

La première expérience proposée est petite : la bibliothèque, l'atelier, un livre et une action autorisée. Tester la même source en terminal, en HTML sans JavaScript et dans une vue actuelle. Vérifier les identifiants, les retours, les contenus et le résultat d'un même journal de demandes. Vérifier aussi une demande interdite.

Pour chaque cible, publier machine ou émulateur, logiciel, octets transférés, mémoire maximale, délai d'entrée et nombre de lieux visitables. Mesurer la batterie sur téléphone réel, à luminosité et parcours fixés, avec répétitions. Une émulation ne démontre pas la consommation électrique de la machine ancienne.

**Vérifié pour cette contribution :** lecture des documents cités et des sources historiques. **Non exécuté :** lecteur rétro, génération terminal, parcours serveur sans JavaScript et mesures de performance. La validation documentaire et les contrôles de publication sont rapportés dans la pull request.

## Demande à Claude et Gemini

Claude : propose ta propre architecture, critique celle-ci, puis transmets la même demande à Gemini avec les documents utiles. Gemini : propose une autre réponse indépendante. Pour chaque période, précisez la machine cible, le support natif ou HTML/CSS/JavaScript, les éléments du monde conservés, les pertes, la preuve à exécuter et la différence avec la vision actuelle. Répondez dans vos dossiers réservés par l'issue #151, par des pull requests. Si Gemini n'a pas accès au dépôt, utiliser le fichier de transmission du protocole. Aucun avis ne devient une décision sans Yocthan.

## Documents à mettre à jour après décision

Claude pourra proposer les ajouts dans VISION, le guide, les critères de la 3D et l'index des propositions. Ce travail ne les modifie pas.

**Ordre recommandé :** 1. garantir navigation et contenu sans moteur graphique ; 2. prouver une action sans JavaScript ; 3. borner mémoire et chargement ; 4. comparer les états entre rendus ; 5. expérimenter une cible rétro précise. Finir la partie web actuelle reste prioritaire sur un portage historique complet.
