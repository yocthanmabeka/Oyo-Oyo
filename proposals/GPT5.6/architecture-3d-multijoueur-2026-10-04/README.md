# Modèles 3D et jeu à plusieurs : revue exécutée de Codex

## Contribution

- Sujet : vérifier les deux choix d'architecture de Claude, les frontières de l'arbitre et les nouveaux noms.
- Auteur : Codex, pour Yocthan, le 2026-10-04.
- Discussions sources : `HC-013` pour le moteur et le format ; `HC-009` pour la remise entre IA ; demande de Yocthan et journal du 2026-10-04 pour ces deux choix. Aucun nouvel identifiant HC inventé.
- Décisions concernées : `ADR-005`, `ADR-008`, `ADR-010`, `ADR-013`, `ADR-015` à `ADR-017`, `ADR-020`, `ADR-023` à `ADR-032`.
- Statut proposé de cette contribution : **EXPLORATION**. Aucun statut de décision modifié.
- Base examinée : `76628678f5939e95001062c92c6b35e373f1ebf3`, branche créée depuis `origin/main`. La [contre-revue des noms, PR 72](https://github.com/yocthanmabeka/Metaverse/pull/72), encore ouverte, a été relue sur sa branche.
- Sources principales : [proposition de Claude](../../Claude/modeles-3d-et-jeu-a-plusieurs-2026-10/README.md), [journal](../../../docs/06-journal/JOURNAL.md), [arbitre](../../../moteur/src/etat.rs), [API du moteur](../../../moteur/src/lib.rs), [rendu](../../../moteur/src/rendu.rs), [shader](../../../moteur/src/rendu.wgsl), [mosaïque](../../../moteur/src/mosaique.rs), [CLI](../../../moteur/src/bin/holo.rs), [serveur](../../../moteur/outils/serveur.mjs), [page d'entrée](../../../moteur/web/page.html), [noms](../../../docs/01-holocode/NOMS.md). Les dix ADR demandées ont été relues, avec leurs ajouts du même jour.

## Résultat

Je recommande un petit essai comparatif pour les modèles, plutôt que choisir B sur la promesse du « million de points ». Pour le partage, A est un bon premier exercice **si le serveur possède réellement la partie**. Un compteur partagé suffit pour cet exercice ; le jeu de la pomme exige d'abord un plateau commun et un temps commun.

Trois phrases de la proposition de Claude sont trop fortes : le million n'est pas un million de points dessinés ; les triangles n'exigent pas un second moteur GPU complet ; l'arbitre actuel ne peut pas être exposé sur un serveur sans adaptation.

**Réponse de Gemini arrivée pendant la revue.** `main` a avancé à `0c9884d391b5682aa88cd6f3b1b11f5bad5cb1bc` par la PR 73 ; seuls AGENTS, le journal et la réponse de Gemini ont changé, pas le moteur examiné. J'ai relu [cette réponse et la lecture critique de Claude](https://github.com/yocthanmabeka/Metaverse/blob/0c9884d391b5682aa88cd6f3b1b11f5bad5cb1bc/docs/05-discussions/reponses/2026-10-04-gemini-3d-et-plusieurs.md). Nous convergons sur le plateau logique. Agrandir les points peut cacher des trous visuellement ; cela ne résout pas leur mélange additif ni l'absence d'occlusion. Les coûts, poids et « moins de 30 ms » proposés par Gemini ne sont pas vérifiés dans cette revue. Son réflecteur de gestes est une autre architecture : ordonner et diffuser ne remplace pas le contrôle des droits et des intentions. Des clients honnêtes peuvent recalculer un état commun, mais il reste à définir qui valide les actions et les résultats qui font foi. Si Yocthan choisit directement le jeu d'action, B est cohérent ; mon A vise seulement le premier compteur partagé, comme celui de Claude. Aucun refus général du polling ne découle du besoin de la pomme.

### Ce qui a réellement été lancé

| Vérification | Résultat observé | Ce que cela ne prouve pas |
|---|---|---|
| `cargo test --manifest-path moteur/Cargo.toml` dans le dossier Windows | **89 réussis, 1 échec** : le test du guide trouve zéro exemple. Le test des leçons passe. | Ne pas annoncer 90 tests verts dans ce dossier. |
| Même base extraite dans une copie temporaire, fichiers texte normalisés en LF, sans changement des fichiers suivis du moteur | **90 réussis, 0 échec**. La recherche du test emploie une chaîne multiligne ; le guide du dossier Windows est en CRLF. | Une correction portable du test reste à proposer par Claude ; cette revue ne la fait pas. |
| `cargo build --release --target wasm32-unknown-unknown`, puis `wasm-bindgen` 0.2.100 | Compilation réussie. Artefact servi `holo_moteur_bg.wasm` : **2 142 776 octets** ; Brotli qualité 11 : **560 601 octets**. Le `.wasm` avant traitement : 3 247 123 octets. | Ce poids actuel n'est ni le poids d'une future 3D ni celui de ses modèles, textures et sons. |
| CLI recompilé, `holo check` sur les deux exemples demandés | Les deux répondent `ok`. Le binaire `holo` sait vérifier et fabriquer le HTML ; il ne propose pas de boucle de partie ni d'API de jeu. | Un CLI présent ne signifie pas serveur multijoueur prêt. |
| [15 sondes Rust](sondes.rs), en debug et en release, avec `--locked --offline` après résolution initiale des dépendances | **15 réussies dans chaque profil**. Ce sont des assertions sur des comportements actuels, dont des comportements dangereux si on les ouvre au réseau. | Le succès des sondes n'est pas une correction ni une certification du serveur. |
| [Sonde Chrome](navigateur.mjs), Chrome 154.0.8037.95 sans fenêtre, profil vide, viewport 360 × 777 | Jeu arrêté avant Play ; pomme à 9 après départ ; panier 50 → 42 à gauche ; première prise, score 1. Leçon 1 : titre, deux paragraphes, séparateur présents. Aucune exception JavaScript captée. [Rapport](preuves/navigateur.json), [capture](preuves/panier.png). | Ni test sur téléphone, ni mesure GPU, ni preuve du moment exact du contact, ni validation de l'audio. Le départ est un clic de script. |

Le serveur déjà actif sur `localhost:8080` répond aux requêtes des exemples, mais sert un WASM différent du build courant, malgré la même taille. La sonde a refusé de mélanger les versions. Les essais Chrome du build courant ont donc utilisé **le même `serveur.mjs` sur `localhost:8082`**, sans arrêter le serveur existant. L'empreinte du WASM servi est contrôlée dans la sonde. La tentative restreinte de Chrome a expiré ; la relance autorisée a fonctionné. Mon premier sélecteur cherchait la position sur la forme au lieu de son enveloppe : corrigé dans la sonde avant les résultats ci-dessus.

Machine de vérification : Windows, Rust 1.99.0, Node 22.21.0. Aucun téléphone mesuré pendant cette revue. Le workflow actuel ne lance pas automatiquement ce nouveau harnais : ses résultats sont locaux et sa commande est fournie ci-dessous.

### 1. Option A : ce qu'il faut ajouter pour des triangles

**Vérifié par compilation et lecture.** `rendu.rs` utilise déjà `TriangleList`. Chaque point est un carré de **deux triangles**, envoyé par `draw(0..6, instances)`. Le problème est de dessiner des surfaces de modèles en 3D. L'instance actuelle ne contient que `position: [f32; 2]`, un rayon et une couleur. Le shader place tous ses sommets à `z = 0`. Il additionne les couleurs ; il n'a ni tampon de profondeur ni texture de matériau.

Il faut conserver `Device`, `Queue`, `Surface` et le repli WebGL 2. Ajouter une autre configuration de dessin, pas réécrire l'accès au GPU. Pour un modèle opaque :

| Ajout | Travail réel | Estimation de code écrit par nous, hors tests et bibliothèques |
|---|---|---|
| Sommets et indices | Positions 3D, normales, éventuellement UV ; buffers de géométrie chargés une fois, limites et contrôle des indices. | 80–140 lignes Rust |
| Caméra et transformations | Matrices modèle, vue, projection ; conventions d'axes, unités et faces avant. | 50–90 lignes Rust |
| Configuration des faces | Attributs par sommet, pipeline opaque, retrait éventuel des faces arrière, création du tampon de profondeur et recréation au redimensionnement. | 90–150 lignes Rust |
| Commandes de dessin | `set_index_buffer`, `draw_indexed`, choix des objets visibles, partage de la couleur et de la profondeur avec les points. | 40–80 lignes Rust |
| Shader minimal | Projection 3D et une lumière diffuse ; pas de textures, animation, ombres ni transparence. | 60–100 lignes WGSL |

**Estimation, pas réalisation.** Cela donne **260–460 lignes supplémentaires dans le rendu Rust et 60–100 dans un shader** pour le noyau minimal. Il faut encore environ **300–700 lignes** pour un petit format préparé, son chargement, ses validations et son raccordement au monde. Le choix A décrit par Claude, avec glTF et textures, dépasse ce noyau : compter grossièrement **1 000–3 000 lignes de code d'intégration au total**, hors bibliothèques, pour un profil statique limité. Un lecteur complet avec PBR, peau, animation et extensions est un autre chantier ; cette fourchette ne le couvre pas.

Estimation de poids ajouté au **WASM traité par wasm-bindgen**, à confirmer par un prototype compilé : noyau sans nouveau décodeur, **+30–150 Ko bruts**, environ **+10–50 Ko Brotli** ; profil glTF statique avec décodage d'images, **+200–1 200 Ko bruts**, environ **+60–400 Ko Brotli**. Ce sont des ordres de grandeur de planification à faible confiance, pas des mesures ni des bornes garanties. `wgpu` est déjà présent ; les décodeurs et les fonctions effectivement conservées par l'édition de liens décideront du delta. Je n'ai pas écrit de pipeline triangles pour mesurer ce delta. Dire « sans doute plus du double » du transfert actuel n'est donc pas établi.

Les modèles et textures sont un coût séparé. Un fichier léger peut aussi entraîner de grosses allocations après décodage. Il faut borner octets reçus, nombres de sommets/indices, dimensions des images et mémoire GPU avant allocation. glTF définit plusieurs sortes de données, matériaux et extensions ; **« tous les modèles existants sont dans ce format » est faux**. Un profil supporté doit être déclaré, et les extensions requises non supportées refusées. [Spécification Khronos, géométrie et extensions](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#geometry).

**Fluidité supposée.** Je ne peux pas donner honnêtement un nouveau nombre d'images par seconde. Le [README du moteur](../../../moteur/README.md) rapporte environ 60 sur Flip 5 et Flip 3, sur des scènes antérieures. À 60, une image dispose d'environ 16,7 ms. Ces relevés ne donnent pas la réserve disponible pour des modèles. Quelques triangles opaques peuvent coûter moins que de gros halos superposés ; beaucoup de textures, d'objets et de transparence peuvent coûter beaucoup plus. Le type de primitive seul ne décide pas de la vitesse. Mesurer le delta sur les deux chemins GPU, puis sur un téléphone modeste, avec une scène fixée et les temps d'image élevés, la mémoire et la chauffe.

### 2. Option B : des objets faits de points

**Vérifié.** On peut préparer des `Sprite` projetés et réutiliser l'envoi actuel. Mais il manque un lecteur de modèle, une géométrie d'objet, ses transformations, sa sélection et des détails adaptés à la distance. `Mosaique` produit des points depuis une image 2D avec relief ; ce n'est pas un chargeur de nuages 3D. Le rendu additif sans profondeur ne masque pas l'arrière d'une chaise derrière son avant. Il convient à un objet lumineux et translucide, pas immédiatement à un objet plein.

Le test `au_repos_l_image_suffit_puis_chaque_pixel_devient_un_point` passe : au repos, `sprites()` est vide, l'image ordinaire suffit. Le « million » est le nombre de pixels représentés, pas celui des instances dessinées en parallèle. Les mesures rapportées par Claude pendant le zoom indiquent seulement **5 980 ou 6 344 points visibles au maximum** dans ces essais. Ni 200 000 points ni un million de points 3D n'ont ici une preuve de 60 images/s sur téléphone.

**Limites actuelles.** `POINTS_MAX = 200_000` borne la production d'une mosaïque ; `INSTANCES_MAX = 262_144` borne le dessin. Le rendu prend les premières instances et coupe le reste silencieusement. Il n'existe **aucune limite par objet**. En passant directement une liste, le plafond est 262 144 pour l'appel entier ; en passant par la mosaïque, c'est 200 000. Ce ne sont pas des objectifs de fluidité.

Une instance GPU a 28 octets selon ses champs : 200 000 instances représentent 5,6 Mo de données, 262 144 représentent 7,34 Mo de buffer réservé. À 60 en réécrivant toute une liste de 200 000, cela ferait 336 Mo/s de données envoyées, hors copies, construction de la liste et fragments du halo. **Ce sont des calculs de volume, pas des mesures de bande passante ou de mémoire d'onglet.**

**Proposition à tester.** Commencer avec **8 000 points visibles par objet au maximum et 50 000 pour toute la scène**, comme paramètres d'expérience, sans promesse de 60. Les autres objets et la mosaïque consomment la même enveloppe ; cinq objets de 8 000 laissent 10 000 au reste. Charger des versions graduelles, réduire les objets lointains, écarter les objets hors vue, répartir le budget avant de construire les listes. Ne pas faire 200 000 par objet, ni laisser le dernier disparaître parce que le premier a tout pris.

`POINTS_MAX` doit rester un garde-fou de producteur pendant l'essai. Il faut ensuite une enveloppe **globale** de la scène, inférieure ou égale à la capacité du rendu, plus des quotas par objet et une limite de données résidentes. Un fichier peut contenir davantage de points répartis en niveaux, mais ses tailles et allocations doivent rester bornées. Pas de relèvement de plafond sans mesure. Les seuls sommets d'un maillage peuvent être très inégalement répartis ; échantillonner ses surfaces de façon reproductible est un meilleur point de départ.

Le morcellement d'un nuage ne découle pas automatiquement de celui de la page. Il faut définir l'identité stable d'un point échantillonné, sa graine et le contenu qu'il révèle. Multiplier arbitrairement les points en zoomant ne reconstruit pas la surface d'origine. Des points opaques avec profondeur constituent déjà une évolution du rendu.

### 3. Une quatrième voie : préparer les modèles avant la lecture

**Vérifié.** Le projet sépare déjà auteur, préparation, moteur et page d'entrée. Aucun convertisseur de modèle ni format de maillage préparé n'est livré ici.

**Proposition D.** Un outil hors navigateur importe un modèle, vérifie ses droits et ses fichiers, puis produit un petit format borné avec plusieurs détails : triangles opaques de près et aperçu en points de loin. Le moteur lit ce format simple, sans lecteur glTF complet ni décodeurs complexes obligatoires au runtime. Variante pour des formes procédurales : produire ces mêmes surfaces à partir de paramètres et d'une graine. Ce format contient des données, pas du code à exécuter.

Cette voie sépare **le sens de l'objet** (`Point`, graine, monde intérieur, capacités) de **sa façon d'être dessiné**. Un objet dessiné en triangles peut encore contenir un monde et être traversable. Ni A ni C ne rendent cette navigation impossible par nature ; elle reste à raccorder. Inversement, un nuage de points ne l'offre pas gratuitement.

D évite le lecteur universel dans le téléphone et conserve des surfaces pleines. Elle coûte un outil, un format/version, des conversions parfois imparfaites et une chaîne de publication des ressources. Une image tournée vers la caméra serait encore plus petite, mais ne remplace pas un modèle vu de tous côtés. Des champs de distance rendus par marche de rayons seraient une autre piste procédurale ; leur coût par pixel et leur difficulté pour les modèles existants rendent l'essai moins immédiat.

Je ferais comparer **B limité** et **D avec maillage opaque simple** sur la même chaise, vue de près et de dos, avec les mêmes contraintes de chargement. Ne pas figer maintenant `Point(model:)`. C reste utile pour un comparatif de qualité : une bibliothèque interne gérée par le moteur n'oblige pas l'auteur à écrire du JavaScript. Le pont et son poids devront être discutés selon ADR-013/015 ; l'interdiction de code libre auteur ne suffit pas à condamner toute dépendance interne.

### 4. L'arbitre est-il pur ?

**Vérifié par les sondes.** Le retour numérique de `etat::arbitrer(programme, état, signal)` est déterministe pour ces mêmes entrées, programme et métadonnées compris. Il ne lit pas directement l'horloge ou le DOM. Mais la fonction entière a un effet observable dans `CAPACITES`, et le système qui la pilote dépend de la page.

| Dépendance | Où elle se trouve | Conséquence constatée ou limite |
|---|---|---|
| Horloge du navigateur | `page.html`, `lancer`, `setInterval`, visibilité et modes de vue ; redémarrage par `touchees` | P01 traite 100 `every:0` sans attente. L'arbitre ne contrôle ni la durée écoulée ni la provenance d'un battement. `performance.now()` sert aussi à la navigation ; pas au cœur numérique d'`arbitrer`. |
| Largeur | `avecLaLargeur`, puis `<` dans l'état, `relire`, `corps` | P06 donne deux scores selon 360 ou 640. C'est une entrée explicite du cœur, mais issue d'un écran extérieur. Sans elle, valeur par défaut 640. Une largeur bornée n'est pas authentifiée. |
| Capacités | `thread_local CAPACITES`, `noter_les_capacites`, `capacites_demandees` | P10 : même retour d'état, deux sons accumulés ; la lecture vide la file. Donc fonction avec effet de bord, pas pure au sens strict. |
| Enveloppes de l'API | `lib.rs` | `arbitrer` et `etat_initial` vident la file avant appel ; `saisir`, `glisser`, `recevoir` ne le font pas. P11 récupère un son d'un appel direct précédent sur une autre page. Mélange d'API reproduit, pas attaque réseau démontrée. |
| Programme et ordre | Parcours des règles, ordre des effets, séquence des appels, compteur `~` | P03 : Add puis Reset donne 0 ; l'inverse donne 1. Ce n'est pas une impureté : l'ordre fait partie du contrat. Le serveur doit l'enregistrer et le partager. |
| Hasard | Graine issue du nom de page, compteur `~` | P08 rejoue le même tirage. Le compteur doit appartenir à l'état autoritaire. La suite publique est prévisible ; elle n'assure pas un hasard secret. |
| Réception et saisie | `recevoir`, `saisir`, `glisser`, puis `suites` et `guetter` | P12/P13 montrent des changements sans signal `On`. Ces voies doivent faire partie du journal de partie et avoir leurs propres permissions. |

`guetter` prend une photo pour reconnaître le passage faux → vrai, applique les effets dans l'ordre, puis recommence **au plus huit tours**. Deux gestes regroupés en un changement peuvent donc différer de deux appels séparés. L'état numérique ne contient pas les échéances des horloges ni leur phase ; rejouer les résultats numériques à partir des signaux enregistrés est plus simple que reconstruire leurs instants et l'affichage.

**Proposition.** Faire une transition sans file cachée : `(programme fixé, état complet, événement accepté, contexte de simulation) → (nouvel état, capacités, diagnostic)`. Le contexte porte un pas logique, des échéances et les dimensions communes ; pas le DOM. La graine/compteur, les textes utiles aux conditions et la version du programme/générateur font partie du rejeu. Les sons sortent comme des données avec un identifiant d'événement, pour éviter de les rejouer deux fois à la reconnexion. Un serveur natif peut réutiliser beaucoup de Rust existant, mais il faut cette enveloppe et un ordonnanceur.

### 5. Ce qu'un joueur malhonnête pourrait envoyer

**Vérifié localement.** Les sondes appellent les fonctions existantes, sans exposer de port de jeu. Il n'existe pas aujourd'hui d'endpoint multijoueur à attaquer. Voici ce qui arriverait **si le serveur faisait confiance à ces paramètres client**.

| Sonde | Entrée ou séquence hostile | Résultat actuel | Frontière à ajouter au serveur |
|---|---|---|---|
| P01 | 100 `every:0` immédiatement | Score 100 sans une seconde écoulée. | Interdire les battements client ; temps décidé au serveur. |
| P02 | `score=999999999`, puis Add | Score 1 000 000 000. | Ne jamais accepter le snapshot du joueur comme état courant. |
| P03 | Même Add deux fois ; Add/Reset inversés | Double ajout ; états finaux différents. | Identifiant unique, déduplication, ordre total par partie, version d'état. |
| P04 | Signal du bouton sous `If` faux | Effet appliqué, score +5. | Bouton caché ≠ permission. Garder les règles sensibles sous condition et vérifier les droits/phase au serveur. |
| P05 | Glisser Basket de 0 à 100 | Accepté sans durée ni vitesse. | Propriétaire d'objet et mouvement permis ; plafonner cadence et distance par pas pour un jeu d'action. |
| P06/P07 | Annoncer une autre largeur, même dans les bornes | Contact et score changent. | Géométrie décidée par la partie, jamais par l'écran du joueur. |
| P08 | Envoyer `~=123456` | Prochain tirage 123457. | Compteur et graine autoritaires, hors messages joueurs. |
| P09 | `~=18446744073709551615`, puis un tirage | Panique capturée en debug ; compteur revient à 0 en release par défaut. | Refuser les métadonnées client et définir le comportement au débordement même pour un snapshot restauré. |
| P10/P11 | Laisser une capacité non consommée avant un autre appel | Un son peut suivre la requête d'une autre page sur le même thread. | Capacités dans le retour de transition ; ne pas lier leur consommation au thread d'un worker. |
| P12 | Saisir le champ caché, valeur 999 | Champ changé, limité à 10. | Autoriser le champ et son propriétaire/phase ; une borne n'est pas un droit. |
| P13 | JSON avec score 999 | Valeur déclarée remplacée ; `~`, `<` et clé inconnue ignorés. | `recevoir` est une voie de données serveur vers page ; ne pas en faire une voie joueur vers état autoritaire. Séparer les valeurs provenant de données et du jeu. |
| P14 | Signal inconnu ; Every avant Play | Sans effet ; les règles sous `If` faux sont bien inactives. | Conserver ces protections ; elles ne remplacent pas l'identité et les quotas. |
| P15 | `within: 9`, écarts 8 et 8 | Rencontre, quelle que soit la largeur. | Contrat de distance à expliquer : carré par axes, pas rayon euclidien. |

**Hypothèses supplémentaires, non attaquées.** Un joueur pourrait présenter la source d'un autre programme, l'identité d'un autre joueur, le nom d'une autre partie, une version périmée ou un très gros paquet. Le serveur doit choisir son programme/version depuis sa propre configuration, déduire l'identité de la session vérifiée et valider une petite intention permise. Ne jamais prendre des effets `score.set(...)` envoyés par le client. Un schéma de message, une limite d'octets, des quotas par joueur/partie et des refus explicites doivent précéder l'appel Rust.

Une forme possible de message est `{partie, action, paramètres_bornés, id, version_attendue}` ; l'identité authentifiée reste dans l'enveloppe de connexion. Aucun mot `shared` ni cette forme de message n'est implémenté ici. HTTP et WebSocket ont besoin des mêmes permissions. Pour WebSocket, vérifier aussi l'origine, la session et chaque message, puis limiter taille et cadence. [OWASP, sécurité WebSocket](https://cheatsheetseries.owasp.org/cheatsheets/WebSocket_Security_Cheat_Sheet.html).

### 6. Le contact dépendant de l'écran

**Vérifié.** P06 est un contre-exemple complet : deux carrés de 40, même hauteur, mêmes positions et même geste. Sur une largeur 360, le score augmente de 10 ; sur 640, il reste à 0. La borne 120–2000 de `<` ne corrige pas cette différence. La page ne fournit en outre qu'une largeur, celle du premier Board visible, pour toutes les rencontres. Le cœur ne vérifie pas ici que les deux corps appartiennent au même Board.

Pour un jeu local, ce choix peut être acceptable : il suit ce que le joueur voit. Mais pour des résultats comparables, même en solo, la largeur doit alors entrer dans l'enregistrement. **Pour une partie partagée, je le refuse comme règle d'autorité.** Le téléphone ne doit pas agrandir les zones de prise simplement parce qu'il a moins de pixels.

**Proposition.** Définir un plateau logique, par exemple 640 × 360 unités, avec tailles et formes de collision dans ces unités. Le serveur juge tous les contacts dans cet espace. Chaque écran affiche ce plateau avec une échelle uniforme et des marges, ou une caméra choisie ; il convertit le doigt vers les coordonnées logiques. L'image et le contact restent ainsi alignés. Garder provisoirement `x` et `y` de 0 à 100 reste possible si on définit précisément leur conversion, actuellement dépendante de la taille de chaque objet. Fixer seulement `< = 640` au serveur, sans adapter l'affichage et la saisie, serait une correction incomplète.

Associer chaque objet à son plateau, refuser une rencontre entre plateaux différents, définir les limites des corps et les formes de collision. Triangle et losange utilisent actuellement un cercle approché ; ils n'ont pas un contact exact avec leur contour. `within` utilise `abs_diff(x) ≤ n` **et** `abs_diff(y) ≤ n`, pas une distance circulaire (P15). Choisir et documenter cette métrique, plutôt que la remplacer discrètement par un rayon. Pour les objets rapides, ajouter un contrôle du trajet entre deux pas ou une sous-division bornée ; le test de la position finale peut rater une traversée.

### 7. Ce qu'il faut décider avant le serveur

**Vérifié.** `Data` reçoit un JSON plat et peut le redemander à partir d'une seconde. Il n'envoie pas d'action. `serveur.mjs` sert des fichiers, fabrique du HTML par le CLI et compresse ; il n'a ni session de partie, ni authentification, ni arbitre partagé. Le CLI ne remplace pas ces éléments.

**Décisions proposées, à faire prendre par Yocthan avant l'implémentation :**

| Décision | Proposition de départ | Pourquoi elle précède le transport |
|---|---|---|
| Premier usage et autorité | Un compteur commun de salon, serveur autoritaire ; pas une extension immédiate du jeu d'action. | Donne une expérience petite et vérifiable. |
| Qui est qui et qui possède quoi | Session attribuée par serveur ; permissions par action, partie et objet. Tester deux sessions distinctes. | Un nom de joueur dans un paquet ne prouve rien. Un vote par personne demande plus qu'une session anonyme. |
| Données communes, personnelles et d'affichage | Séparer ces catégories ; ne pas partager tout `State`. Garder `<`, sons et stockage local hors de l'état joueur reçu. | `shared: [votes]` ne dit pas qui peut voter ni si le client peut remplacer `votes`. |
| Programme et ressources | Source validée et version fixées par le serveur, limites d'exécution et de ressources. | Aucun client ne choisit des règles nouvelles pour une partie existante. |
| Temps, ordre et géométrie | Horloge serveur, journal ordonné, échéances explicites, plateau logique commun. | Même résultat malgré écran, délai et ordre d'arrivée ; pas de pause de tous quand un onglet est caché. |
| Atomicité, doublons et reconnexion | Une transition à la fois par partie ; id d'action dédupliqué ; réponses avec révision ; reconnexion par snapshot/révision. | Deux lectures concurrentes puis deux écritures perdent un ajout, même sans triche. |
| Persistance et départ des joueurs | Durée de vie des salons, sauvegarde, récupération après panne, effacement et version des snapshots. | Une graine ne reconstitue pas les actions humaines. `keep` du navigateur ne fait pas foi. |
| Hasard et équité | Pour le premier compteur, pas de hasard. Pour un jeu, décider si la suite publique rejouable convient ou si un secret serveur doit être enregistré/protégé. | Déterminisme et imprévisibilité sont deux propriétés différentes. |
| Exploitation et limites | Prototype local sur boucle locale, puis hébergement, coût, surveillance, limites de connexions, TLS et conservation des journaux avant mise en ligne. | Le serveur démo écoute `0.0.0.0` et publie les sources avec CORS `*` ; ses réglages statiques ne sont pas ceux d'un service de partie. |
| Contrat d'échec | Action refusée, reconnexion, file pleine, horloge en retard, chaîne de When au plafond de huit tours. | Des erreurs silencieuses et des effets partiels empêchent un rejeu fiable. |

Je suis d'accord avec **A d'abord**, pour cet usage limité. La réponse à une action peut contenir immédiatement la nouvelle révision ; les autres visiteurs lisent les mises à jour par polling. « Une seconde de retard » décrit un rythme, pas une garantie : ajouter réseau, traitement et éventuels échecs. Les snapshots de jeu reçus ne doivent pas refaire localement des effets autoritaires de `When`.

Puis choisir B si le jeu demande une faible latence. WebSocket réduit l'attente due au polling ; il ne retire ni le délai réseau, ni les limites d'une liaison ordonnée, ni les droits, ni la reconnexion. HTTP pour les intentions et SSE pour les mises à jour seraient aussi possibles ; le choix du transport peut rester derrière la même autorité.

Pour C, WebRTC demande généralement signalisation et parfois relais TURN : « pas de serveur à payer » n'est pas acquis. [Protocoles WebRTC](https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API/Protocols). Un joueur hôte peut rester arbitre au sens d'ADR-015, mais cela ne donne pas la confiance d'un serveur indépendant. Le refus d'un hôte pour une compétition est une exigence d'équité à décider, pas une conséquence automatique de cet ADR. Une partie coopérative entre personnes consentantes peut avoir un autre modèle de confiance.

### 8. Les noms ajoutés : même règle, même sévérité

**Vérifié.** Les blocs figurent dans `BLOCS`, les réglages sont utilisés par les validateurs et le générateur ; les tests du moteur exercent leur usage. La contre-revue PR 72 conserve les mots anglais simples, **bloc avec initiale majuscule**, **réglage, action ou valeur symbolique en minuscules**, et les graphies externes conventionnelles (CSS, unités). Les nouvelles familles suivent cette forme. Aucun camelCase, suffixe `View` ou renommage des mots déjà lisibles n'est nécessaire.

**Hypothèse linguistique.** Aucun essai débutant n'a été fait ici. Les appréciations de compréhension ci-dessous supposent au moins un peu d'anglais ; elles ne sont pas des résultats utilisateurs. Un même mot dans une autre API n'est pas toujours une confusion réelle. Mais je ne peux pas appliquer ADR-016 sévèrement à mes noms puis ignorer `form` ou le `When` explicitement écarté dans cet ADR. Une exception éventuelle doit être proposée à Yocthan, pas déclarée acceptée.

| Mot actuel | Ma proposition | Ce que tu y gagnes | Ce que tu y perds / sens existant à respecter | Recommandation finale |
|---|---|---|---|---|
| `If` | Identique | Condition familière en JavaScript ; « si » est simple. | Un bloc déclaratif n'est pas une instruction libre ; pas de sinon/or implicite. | **Garder**. |
| `Hr` | Identique ; `Line` écarté | Même séparation thématique que HTML si on enseigne son rôle. | Un débutant ne devine pas l'abréviation. `Line` serait plus simple, mais évoquerait un trait géométrique en 3D. Un `hr` est une rupture thématique, pas toute décoration horizontale. | **Garder**, expliquer le rôle. |
| `Quote` | Identique | Citation : plus lisible que `blockquote` pour un novice. | Le moteur fabrique une citation en bloc ; il ne fusionne pas les rôles distincts de `q` et `cite`. | **Garder**, ne pas annoncer « remplace les trois sens de HTML ». |
| `Code` | Identique | Texte de code montré sans interprétation, comme `<pre><code>`. | Si son rôle devient tout texte littéral sans rôle de code, ce nom ment. `Literal` serait plus exact dans ce cas, plus difficile pour le novice. | **Garder** pour le code préformaté ; **à discuter** si le contrat reste « tout texte littéral ». |
| `Every` | Identique ; `Timer` écarté | `Every(1s)` se lit mieux qu'un nom de mécanisme pour un débutant. | JavaScript `Array.every` teste tous les éléments ; ici le temps est explicite. L'arrêt lié à la visibilité n'est pas dans le nom et ne doit pas gouverner une partie serveur. | **Garder** dans ce contexte de durée, réserve ADR-016 à expliciter. |
| `Board` | Identique | Plateau, mot concret ; meilleur que `Stack` ou `Stage` pour le jeu. | Ne signifie ni pixel d'écran ni grille ; ne choisit pas les unités ou les contacts. | **Garder**. |
| `Input` | Identique | Champ de saisie connu en HTML ; `Field` ne serait pas un gain assuré pour le débutant. | HTML offre plus de types ; ici le type vient de l'état. Dire « sous-ensemble », pas « même couverture ». | **Garder**. |
| `Checkbox` | Identique | Case à cocher, sens HTML/Flutter conservé. | État 0/1 à enseigner ; pas le booléen ou trois états de toutes les autres API. | **Garder**. |
| `When` | `Watch` comme candidat seulement | Watch dirait « surveiller », distinct de If et On. | ADR-016 cite déjà `when` de Kotlin comme choix entre cas et l'écarte. `When` se lit mieux pour le novice. Watch rappelle les observateurs réactifs, mais n'annonce pas non plus le seul passage faux → vrai. Mon candidat n'est pas sans collision. | **À discuter en priorité** : retirer la contradiction explicite, sans valider Watch par défaut. |
| `Part` | Identique ; `Component` à comparer | « Morceau » est plus simple que component. | `part`/`::part` en HTML/CSS nomment une partie exposée d'un Shadow DOM ; ici c'est un gabarit importé. Component suggère aussi des paramètres absents. | **À discuter**, Part a l'avantage débutant, collision réelle mais contextes différents. |
| `Use` | Identique | Utiliser un morceau nommé ; proche de la réutilisation par SVG `<use>`. | Ne promet ni import Rust, ni hook React `use...`, ni composant paramétrable. `Include` serait moins courant et pourrait annoncer la lecture du fichier à cet endroit. | **Garder**. |
| `Data` | Identique ; `Feed` écarté pour l'instant | Mot courant pour les données, moins technique que Fetch. | HTML `<data>` associe une valeur lisible par machine à un texte ; il ne charge rien. Feed ferait mieux entendre une source reçue, mais suggère un flux/listes absents. | **À discuter** sous ADR-016 ; pas de gain débutant démontré pour Feed. |
| `Sound` | Identique | Un son, clair ; plus simple qu'une ressource audio pour un novice. | L'élément HTML audio peut aussi jouer un son sans contrôles. Sound ne justifie pas une fausse distinction « audio = toujours lecteur ». | **Garder** ; le contrat actuel n'offre pas toute une API audio. |
| `Shape` | Identique | Forme simple, sens familier en dessin et en 3D. | Les quatre formes sont plates, pas une géométrie 3D ni un contour physique exact. | **Garder**. |
| `keep` | Identique ; `persist` écarté | « Garder » est plus simple pour le novice. | Ne dit pas où ni combien de temps ; ce n'est pas un enregistrement de confiance au serveur. Persist serait du jargon et pourrait promettre trop. | **Garder**, préciser stockage local et effacement. |
| `prices` | Identique | Prix pluriels, lecture directe ; cohérent avec Prices. | Entiers sans devise/unité implicite ; une table locale n'est pas une preuve de prix commercial. | **Garder**. |
| `data` | Identique | Couple `data: Data` conforme à `state: State`, sans composé. | Sens générique ; mêmes réserves que Data, et `data-*` HTML porte des données arbitraires plutôt qu'une source. | **Garder la graphie**, contrat **à discuter** avec Data. |
| `meets` | Identique | « Rencontre », concret pour un débutant. | Peut vouloir dire contact de contours, test approché, ou proximité avec within. Le mot ne distingue pas ces trois contrats. `collides` serait plus technique et suggérerait un moteur physique. | **Garder**, spécifier les formes et la transition. |
| `within` | Identique ; `radius` écarté | « À l'intérieur de cette limite » s'accorde avec la proximité. | Le novice peut croire à un cercle. P15 prouve une limite sur chaque axe ; radius serait actuellement faux. | **Garder pour la limite par axes**, **à discuter** si on veut une distance euclidienne. |
| `drag` | Identique | Geste connu ; un mot suffit. | HTML draggable concerne aussi le transfert de données, pas ce déplacement précis. `move` perdrait le geste et serait moins exact. | **Garder**, qualifier le glissement sur Board. |
| `form` dans Shape | `shape` | Retire la collision HTML : `form` associe un contrôle à un formulaire. `Shape(shape: circle)` répète un mot mais dit bien la géométrie. | Form est court et probablement aussi simple pour le novice ; la répétition est réelle. `kind` évite la répétition mais dit moins ce qu'on choisit. | **Changer**, si Yocthan retient le contrat strict d'ADR-016 ; préférer l'explicite à une nouvelle ambiguïté. |
| `from` dans Data | Identique | Provenance, phrase facile à lire. | JavaScript `Array.from` crée un tableau ; pas un mot réservé standalone. `source` annoncerait mieux une ressource mais créerait une autre incohérence avec les usages existants. | **Garder** dans ce contexte, sans affirmer absence de tout homonyme. |
| `every` dans Data | Identique | Même rythme que Every, un seul mot de durée. | Collision avec `Array.every`, comme Every. `interval` est plus technique et ne simplifie pas forcément le débutant. | **Garder**, réserve de contexte identique à Every. |
| `value` | Identique | Même idée de valeur du champ. | Ici on donne le **nom** de la valeur liée ; HTML value peut être un contenu littéral. Un débutant a besoin d'un exemple de liaison. `bind` serait plus technique. | **Garder**, ne pas prétendre à une syntaxe identique de HTML. |
| `label` | Identique | Étiquette, rôle HTML conservé ; obligatoire, bonne aide pour tous. | Un réglage n'est pas un élément HTML label indépendant. Aucun gain à `caption` ou `text`, déjà d'autres rôles. | **Garder**. |

Cette règle vaut aussi pour les mots de la liste complète : je maintiens le retrait de mes camelCase, Depth, PointView et PortalView. Les trois propositions de la PR 72 restent des avis non validés. Je ne les applique pas en douce à ces nouveaux noms. Je reconnais que **When et Part sont plus simples que Watch et Component** pour un novice ; ce seul avantage n'efface pas leur collision. `Hr` demeure moins transparent que Line, sans qu'un remplacement général soit automatiquement meilleur.

Sources primaires de cette comparaison : [HTML : hr, blockquote, pre](https://html.spec.whatwg.org/multipage/grouping-content.html), [HTML : attribut form](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#attr-fae-form), [CSS Shadow : part](https://drafts.csswg.org/css-shadow-1/#part-attr) (brouillon courant), [SVG : use](https://www.w3.org/TR/SVG2/struct.html#UseElement), [JavaScript : Array.every](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/every), [Kotlin : when](https://kotlinlang.org/docs/control-flow.html#when-expressions-and-statements). Elles documentent les sens existants ; elles ne prouvent pas la compréhension des débutants.

## Objections et limites

Le réseau, deux joueurs réels, une panne de serveur, des triangles et un nuage importé **n'ont pas été exécutés**. Les sondes prouvent des comportements d'API et des risques conditionnels pour un serveur naïf. Elles ne prouvent pas qu'un joueur peut aujourd'hui modifier la partie d'un autre : aucune partie commune n'est encore hébergée.

Les fourchettes de code et de poids sont des estimations de périmètres précis, avec forte incertitude. Le protocole mobile reste indispensable pour départager les rendus. La sémantique du contact actuel est améliorée pour le jeu solo ; sa transposition au serveur doit préserver le rendu autant que le calcul. Le vocabulaire n'est pas un correctif de sécurité.

## Expérience ou preuve requise

Pour reproduire les sondes depuis la racine, sans changer le moteur :

```powershell
C:\Users\mokea\.cargo\bin\cargo.exe test --locked --manifest-path proposals/GPT5.6/architecture-3d-multijoueur-2026-10-04/Cargo.toml -- --nocapture --test-threads=1
C:\Users\mokea\.cargo\bin\cargo.exe test --locked --release --manifest-path proposals/GPT5.6/architecture-3d-multijoueur-2026-10-04/Cargo.toml -- --nocapture --test-threads=1
# Après construction du paquet web/pkg et démarrage du serveur :
node proposals/GPT5.6/architecture-3d-multijoueur-2026-10-04/navigateur.mjs
# Si 8080 est déjà occupé, serveur.mjs accepte PORT ; la sonde accepte HOLO_URL.
```

Le harnais dépend du moteur actuel. Si une protection est ajoutée, retourner les assertions concernées en tests de refus : ne pas supprimer une sonde pour conserver du vert. P09 attrape volontairement la panique debug. P11 est un mélange d'API sur un même thread ; `--test-threads=1` simplifie les sorties, sans créer à lui seul ce défaut. Le profil release testé est celui du harnais, sans `panic=abort` ; ne pas en déduire le comportement d'un futur serveur compilé avec d'autres options.

Preuves futures : même snapshot après rejeu ordonné ; deux sessions concurrentes sans ajout perdu ; même action reçue deux fois ne comptant qu'une fois ; refus d'une action d'un autre propriétaire ; reconnexion sans répéter les sons ; même résultat à deux tailles d'écran ; mouvement rapide ne traversant pas un objet. Pour le rendu : chaise de référence en B et D, proche/loin, plusieurs objets, WebGPU et WebGL 2, téléphone modeste, payload et mémoire réels après décodage, temps d'image et chauffe sur une durée annoncée. Rien de cette liste n'est présenté comme déjà réussi.

## Documents à mettre à jour

Après choix de Yocthan, Claude pourra mettre à jour le contrat de simulation (ADR-026/028), les limites et unités du rendu, le protocole de données (ADR-030), le guide, NOMS et les leçons. Pour un renommage éventuel de form : `moteur/src/plat.rs`, `moteur/src/etat.rs`, leurs tests, les `.holo` concernés dans `exemples/jeu/` et `exemples/lecons/`, GUIDE, NOMS, ADR-032 et cette fixture hostile. Les substitutions doivent être contextualisées ; les attributs HTML `form` ne sont pas à renommer. La forme native de l'arbitre, les entrées réseau et leurs refus doivent recevoir des cas de conformité. Les statuts restent l'affaire de Yocthan. Aucune de ces migrations n'est faite ici.

## Priorités, du plus important au moins important

1. **Définir l'autorité et les droits.** Source, état, temps, graine et objets appartiennent au serveur ; le joueur envoie une intention limitée. Interdire immédiatement les entrées reproduites par P01/P02/P05/P08/P09 à cette frontière.
2. **Fixer la simulation commune.** Plateau et corps en unités logiques, ordre total, échéances, atomicité, doublons, versions et rejeu. Un transport rapide ne répare pas des règles différentes selon l'écran.
3. **Rendre les capacités explicites.** Retirer la file cachée de la transition, définir les diagnostics et la limite des huit tours ; préserver les effets uniques au rejeu et à la reconnexion.
4. **Faire un seul compteur partagé local avec deux sessions.** Mesurer les refus, la concurrence et la reconnexion avant mise en ligne ; décider persistance et exploitation. A convient à cet essai, B viendra si le besoin le demande.
5. **Comparer B limité et D préparé sur téléphone.** Respecter un budget global, mesurer les fichiers et les données décodées ; ne pas choisir sur le million de pixels ou sur une promesse de doublement du WASM.
6. **Trancher les conflits de noms les plus nets.** form en premier ; contradiction When/ADR-016 ensuite ; tester Part/Data et les autres ambiguïtés avec des débutants. Garder les mots simples qui font leur travail.
7. **Rendre le test du guide portable et préciser les chiffres documentés.** Consigner Windows 89/90 et copie LF 90/90 ; distinguer pixels, points dessinés, poids brut, transfert et mesure téléphone.
