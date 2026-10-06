# Le site de référence pour mettre Holoverse à l'épreuve

## Contribution

- Sujet : caractéristiques du site à construire et critères pour vérifier les promesses du projet.
- Auteur : Codex, pour Yocthan, le 2026-10-06.
- Discussions sources : `HC-011`, `HC-013` ; demande de Yocthan : « Envois dans Github les caractéristiques du sites à construire pour tester le projet à 100% ».
- Décisions concernées : `ADR-005`, `ADR-007` à `ADR-010`, `ADR-014` à `ADR-034`, selon les fonctions testées.
- Statut proposé de cette contribution : **PROPOSITION**. Aucun statut de décision n'est changé.
- Dépôt examiné : `yocthanmabeka/Metaverse`, `main` au commit `a142a4c9b90282f680bbbf3dd5c30ed83a23ca12`.
- Résultat livré ici : un cahier des charges et une [recette](RECETTE.md). Le site reste à construire.

## Résultat : ce que « 100 % » voudra dire

**Construire un site utile, puis prouver qu'il reste utile à plat et en profondeur, sur un vrai téléphone modeste, sans que son auteur écrive du HTML, du CSS ou du JavaScript.** Une belle capture ne suffit pas.

« 100 % » signifie que tous les critères du périmètre annoncé ont leur preuve et sont réussis. Cela ne signifie ni zéro défaut possible, ni tout le web remplacé, ni tous les téléphones du monde vérifiés. La couverture des tests et la satisfaction de Yocthan sont deux résultats séparés.

Deux étapes évitent de cacher les manques :

1. **V1 locale** : éprouver les capacités présentes et leurs limites. C'est le prochain site à construire.
2. **Extensions** : éprouver les fonctions encore absentes après choix d'architecture. Leur absence reste visible dans le bilan du projet. Une V1 réussie ne les valide pas.

Le [journal du 2026-10-04](../../../docs/06-journal/JOURNAL.md) donne la priorité : le serveur de jeu et le jeu en ligne attendent que le métavers plaise à Yocthan en local. Le serveur local qui sert les fichiers de démonstration reste nécessaire ; ce n'est pas un serveur de partie.

## Le site : « L'atelier des mondes »

Une petite boutique de créations, avec leurs histoires et les lieux où elles sont exposées. Le visiteur peut seulement lire, composer un panier fictif ou explorer. Il n'a pas besoin de connaître le mot « métavers » pour comprendre le site.

L'accueil commence comme un site ordinaire : titre, trois créations, leurs prix, une navigation claire. « Explorer l'atelier » est un choix visible. Les mesures et les détails du moteur restent dans les outils de recette, hors du parcours du visiteur.

| Page ou lieu à construire | Contenu précis | Ce que cela met à l'épreuve |
|---|---|---|
| Accueil | Promesse en une phrase, menu commun, trois créations, accès au catalogue et à l'atelier. | Page lisible avant le moteur ; entrée spatiale facultative. |
| Catalogue | Douze créations écrites explicitement ; grille qui se replie sur téléphone ; images légères avec descriptions. | `Grid`, `Row`, `Column`, styles vérifiés, images, largeur étroite. Ce n'est pas une liste dynamique. |
| Fiche d'une création | Titre, image, texte long en Markdown, liste, citation, séparateur ; prix et lien vers son point d'exposition. | `H1` à `H3`, `P`, `Text`, `List`, `Quote`, `Hr`, `A`, lecture et sélection du texte. |
| Panier et préférences | Trois articles à prix entiers : 120, 90 et 50 ; ajouter, retirer, vider ; quantités, nombre et total ; nom, option cadeau, quantité bornée ; message quand le panier est vide. | `State`, `Prices`, `On`, `If`, `Input`, `Checkbox`, `keep`. Le panier est fictif, sans paiement ni commande envoyée. |
| Atelier et jardin | Deux lieux voisins, dont un dans un autre fichier ; panneaux de texte, points nommés, point planté dans un pixel ; retour, carrefour et vue personnage. | `Point`, `World`, `Zoom`, `Points`, `Relief`, `Portals`, passages et conservation de l'état. |
| Journal de l'atelier | Article long, trois niveaux de titres, liens entre pages, exemple de code affiché comme texte. | Navigation, Markdown, `Code`, partage des morceaux par `Part`, `Use` et `import`. |
| Disponibilité des créations | Trois nombres déclarés, actualisés depuis un petit JSON du même serveur ; affichage de la dernière valeur utilisable. | `Data`, `from`, `every`, données mal formées et panne réseau. Cette lecture ne réserve aucun stock. |
| Jeu local | Reprendre le panier qui attrape une pomme ; démarrer, mettre en pause, reprendre ; clavier et glissement ; meilleur score local ; son après un geste. | `Board`, `Every`, `When`, `meets`, `within`, `drag`, hasard rejouable, `Shape`, `Sound`, règles conditionnelles. |
| Galerie animée | Une entrée de titre, une boucle discrète et trois scènes ; lettres et enfants décalés. Version arrêtée quand le visiteur demande moins de mouvement. | `Enter`, `Loop`, `Scenes`, `Scene` ; contenu lisible sans télécharger le WASM pour ces seuls mouvements. |

Les textes peuvent être en français. Les mots du langage restent ceux du [guide](../../../docs/01-holocode/GUIDE.md) et de [NOMS.md](../../../docs/01-holocode/NOMS.md). Ce cahier ne les renomme pas.

### Le parcours de référence

1. Ouvrir l'accueil par un lien dans Chrome sur téléphone, sans installation ni compte.
2. Lire une fiche et mettre deux créations à 120 et une à 90 dans le panier : **3 articles, total 330**.
3. Entrer dans l'atelier, viser un point et traverser sept mondes ; ouvrir le jardin et revenir.
4. Revenir à la page et retrouver les mêmes quantités et le même total.
5. Retirer une création à 120 : **2 articles, total 210**. Vider : **0 article, total 0**.
6. Recharger : seules les valeurs annoncées dans `keep` sont conservées. Les autres suivent les règles du guide.

Le site doit aussi fonctionner pour celui qui s'arrête à l'étape 2. La vue en profondeur suit [ADR-018](../../../docs/02-gouvernance/adr/ADR-018-vue-en-profondeur.md) : les points ont de la profondeur ; les textes et contrôles restent lisibles sur un panneau. On n'exige pas un paragraphe transformé en objet 3D.

### Des fichiers d'auteur simples

Emplacement proposé pour la construction : `exemples/site-reference/`, à réserver dans une tâche avant d'y écrire. Il n'est pas créé par cette proposition.

- Une source `.holo` par page, un `commun.holo` pour le menu et les morceaux partagés, des images et un son courts, un `stock.json` de démonstration.
- Les réglages de vue vivent dans le fichier de la page. Une page sans `points` ni relief reste une page ordinaire ; le site comporte ce cas témoin.
- Aucun script libre, aucune génération par IA à l'ouverture, aucune seconde définition du panier pour la vue en profondeur. Le JavaScript interne du moteur n'est pas du code demandé à l'auteur.
- Pas de police distante, de vidéo automatique, de bibliothèque publicitaire ou de compte pour la V1. Les médias portent leur poids réel dans le bilan.
- Les fichiers `.holo` doivent passer `holo check` ; le HTML doit être produit par `holo html` ou le serveur de démonstration, pas réparé à la main après génération.
- Un index de couverture relie chaque bloc et réglage de `NOMS.md` à un fichier et à un cas positif ou négatif. Un terme non appliqué par le moteur est marqué absent, jamais couvert par sa seule présence dans la grammaire. Les réglages rares peuvent vivre dans les fixtures de recette : ils n'encombrent pas l'accueil.

## Ce qu'il faut prouver

Les critères détaillés, leurs identifiants et le format des preuves sont dans [RECETTE.md](RECETTE.md). Voici les conditions qui rendent le site convaincant.

### Un vrai site, même avant la 3D

Texte, images décrites, titres et liens restent lisibles avec JavaScript coupé. Les contrôles qui exigent le moteur annoncent leur indisponibilité si celui-ci ne peut pas démarrer ; ils ne prétendent pas avoir enregistré une action.

À 320 pixels CSS de large et avec le texte agrandi à 200 %, aucun texte ni contrôle essentiel ne sort de l'écran. Le clavier donne accès aux tâches essentielles ; le focus reste visible. Un lecteur d'écran doit permettre de lire le catalogue et de composer le panier. Ces exigences sont une recette fonctionnelle, pas une déclaration de certification.

La bascule entre les vues conserve l'état, le contexte de navigation et une sortie claire. Le bouton retour du navigateur reste utilisable. Un lien externe garde une destination identifiable. Une perte du contexte graphique laisse au minimum la lecture et un moyen de revenir à plat.

### Le chargement léger doit résister à l'attente

Tester séparément : visite neuve statique, premier geste, visite avec valeurs gardées, page vivante, entrée directe dans un monde. Une page `Data` ou `Every` ne peut pas servir de preuve du poids d'une page statique : elle demande le moteur dès le départ selon [ADR-033](../../../docs/02-gouvernance/adr/ADR-033-site-leger.md).

Retarder le téléchargement du moteur de cinq secondes, puis toucher trois fois « Ajouter » et commencer une saisie. Après démarrage : exactement trois ajouts, le texte complet, un focus utilisable. Une erreur réseau n'efface ni la saisie ni le contenu déjà lisible. Aucun bouton n'envoie une fausse confirmation.

### La même partie et le même monde

À graine, version et suite d'événements logiques identiques, le cœur produit le même état. Les événements de temps sont enregistrés : des horloges réelles lancées sur deux appareils ne sont pas, à elles seules, une entrée identique.

La rencontre se calcule dans le plateau logique, indépendamment de l'écran. Le journal indique que ce défaut est déjà corrigé : on écrit une régression, on ne le présente pas comme encore ouvert. `within` se teste selon son sens documenté, par axe, et non comme un rayon inventé. Les invariants numériques sont vérifiés exactement ; les différences de positions flottantes sont mesurées séparément, sans promettre une égalité graphique au bit près.

Les appels de l'arbitre ne doivent pas laisser ressortir un son d'une action antérieure. Les limites de saisie, les graines voisines au-delà de `2^53`, les imports erronés et les données non déclarées ont des cas de recette. Les anciennes sondes sont des références à relancer sur le commit testé, pas des preuves de défauts actuels.

### Des budgets proposés, à mesurer

**Ces seuils sont des propositions de recette ; ce ne sont pas des mesures faites sur ce nouveau site ni des décisions validées.** Un dépassement reste un échec à traiter ; on ne relève pas le seuil après avoir vu le résultat sans expliquer et soumettre ce changement.

Ko = 1 000 octets ; Mo = 1 000 000 octets. Le poids transféré est la somme des corps de réponse réellement reçus, avec leur compression indiquée. Rapporter aussi les tailles brutes et les en-têtes séparément. Le dossier de développement, les cibles Rust et `.git` sont un autre bilan.

| Mesure | Cible proposée pour la V1 |
|---|---|
| Accueil statique, visite neuve, avant un geste | Au plus **25 Ko** pour HTML, styles et amorçage ; aucun WASM ni module du moteur demandé. Au plus **150 Ko** avec toutes les vignettes de l'accueil. |
| Ensemble du parcours, cache vide | Au plus **2 Mo transférés**, moteur et médias compris ; chaque ressource comptée une fois si elle est réutilisée. Rapporter séparément la page vivante et les préchargements. |
| Sources du site et médias | Au plus **100 Ko de `.holo` brut**, **1 Mo de médias**. Ce budget ne comprend pas le moteur partagé. |
| Première page lisible | Au plus **1 s**, réseau à 10 Mbit/s, aller-retour 100 ms, cache vide, sur téléphone réel ; du début de navigation à l'affichage du titre et des premières créations. |
| Première image spatiale | Au plus **1 s** à partir de la demande d'exploration, sur le même réseau, y compris l'arrivée du moteur s'il est absent. Mesurer aussi à cache chaud. C'est un objectif exigeant, pas un résultat acquis. |
| Première action pendant le chargement | Indication d'attente en moins de **100 ms** ; aucun geste perdu ou doublé. À 1 Mbit/s et 300 ms d'aller-retour, mesurer l'attente réelle sans promettre une seconde. |
| Fluidité en exploration à 60 Hz | Au moins **55 images/s en moyenne**, avec une cible de 60 ; temps entre images : 95e percentile au plus **20 ms**, 99e au plus **33,3 ms** ; aucune pause de plus de **50 ms** lors d'un passage déjà chargé. |
| Mémoire en session locale | **PSS de l'onglet inférieur à 300 Mo** ; rapporter également RSS, tas JS et mémoire WASM. Après 100 allers-retours, croissance retenue au plus **10 %** du niveau stabilisé initial ; pas de croissance continue avec la profondeur. |
| Session de 15 minutes | Aucun plantage ni arrêt thermique ; cadence des cinq dernières minutes au moins **90 %** de celle des cinq premières. Relever température, batterie et état thermique avec leur méthode. |
| Batterie | Comparer à 15 minutes de lecture du même site à plat, luminosité et réseau fixes. Cible proposée : surcoût au plus **3 points de batterie** ; publier la résolution de la mesure. Plus de **20 points perdus** en 15 minutes reprend le signal d'abandon du sprint, pas une cible de bonne consommation. |

Si la résolution de batterie ne permet pas de distinguer trois points, répéter ou mesurer l'énergie avec un outil adapté : le résultat est « non concluant », pas « réussi ». Une température brute sans température ambiante ni état thermique ne prouve pas l'absence de chauffe.

La cible de 55 images/s est ici un plancher de recette proposé ; le sprint vise toujours 60. La limite de 300 Mo vient de son [README](../../../moteur/README.md), et reste plus utile que le seul plafond de 1 Go de la vision. Une matrice de quelques appareils ne suffit jamais à démontrer « n'importe quel téléphone actuel ».

### Créer sans savoir programmer

Cinq personnes qui n'ont jamais programmé essaient le site, puis le guide. On consigne leurs blocages et leurs demandes d'aide, sans IA qui écrit les modifications à leur place.

- Au moins quatre sur cinq réalisent le parcours lecture → panier → atelier → retour en cinq minutes, sans aide.
- Au moins quatre sur cinq changent un titre, un prix et la destination d'un point en trente minutes avec le guide seulement.
- Chacun reçoit une erreur volontaire, puis doit retrouver la ligne et corriger le fichier. Aucun besoin de modifier le HTML produit.
- Yocthan juge personnellement la lecture, les gestes, la beauté et l'intérêt de l'exploration. Les motifs de refus sont conservés, même si les chiffres sont bons.

Cinq personnes donnent un premier signal, pas une preuve universelle de facilité. Compter les lignes écrites ne remplace pas cet essai.

## Extensions : ce que la V1 ne peut pas valider

| Besoin réel | Expérience à prévoir | Décision ou travail préalable |
|---|---|---|
| Envoyer une demande | Un formulaire envoie une demande, le serveur confirme une seule réception ; coupure et nouvelle tentative ne créent pas un doublon. | Écriture de l'envoi, validation serveur, autorité et stockage. L'`Input` local et le JSON lu par `Data` ne le font pas. |
| Catalogue alimenté par des données | Charger, filtrer et afficher une liste, y compris liste vide ou longue. | Listes reçues, répétition, limites de taille et erreurs. Douze fiches copiées ne prouvent pas cette capacité. |
| Objets 3D pleins | Montrer trois objets identiques en points, puis dans la voie 3D choisie ; comparer silhouette, occultation, poids, mémoire et fluidité sur téléphone. | Choisir la représentation après comparaison des options. Les points actuels et les rotations CSS ne prouvent pas un rendu de modèles pleins. |
| Partie à plusieurs | Deux écrans de tailles différentes ; latence, reconnexion, doublons et joueur qui forge temps, score ou déplacement ; un état autorisé commun. | Après validation locale par Yocthan : autorité, temps logique, ordre des événements, propriété des objets, protocole versionné, persistance et quotas. Voir la [revue PR 74](https://github.com/yocthanmabeka/Metaverse/pull/74). |

Pas de vrais achats, de comptes ni de partie en ligne dans le premier lot. La V1 affiche honnêtement ses simulations. Si ces extensions restent absentes, le bilan dit « V1 locale validée, objectifs étendus non validés », jamais « tout le projet atteint à 100 % ».

## Vérifications et hypothèses

| Nature | Ce qui est établi pour cette contribution |
|---|---|
| Vérifié par lecture | Vision, protocole, décisions, journal récent, guide, inventaire des noms, comparaison web, exemples boutique et mouvement, limites de points dans `mosaique.rs` et `rendu.rs`. Base exacte indiquée plus haut. |
| Résultat publié par Claude, pas remesuré ici | Journal : boutique statique passée de 579 à 8 Ko au départ ; première saisie encore susceptible d'être effacée ; plateau logique corrigé. Exemples : animations produites en CSS ; fluidité de la galerie non mesurée sur téléphone. |
| Vérifié en lançant le nouveau site | **Rien : il n'est pas encore construit.** Les cases de recette sont initialement non exécutées. Les contrôles documentaires de cette PR ne prouvent pas le site. |
| Proposition | Thème, pages, budgets, panel d'appareils, essais utilisateurs et ordre des lots. |

La [boutique comparée](../../../exemples/boutique-comparee/README.md) et le [duel de mouvement](../../../exemples/motion/README.md) servent de départ. Leurs anciennes comparaisons ont des différences de fonctions : on compare à nouveau le même contenu et les mêmes tâches, sans attribuer au langage les bénéfices d'une version qui en fait moins.

## Objections et limites

- Ce site couvre un usage de référence ; il ne remplace pas une conformité complète du langage ni les tests du moteur.
- La V1 utilise certains choix encore à l'essai. Elle sert à les éprouver, pas à valider leurs statuts par une PR.
- Les limites actuelles sont visibles dans le bilan : `POINTS_MAX = 200_000` pour la mosaïque et `INSTANCES_MAX = 262_144` pour le rendu au commit examiné. Ce ne sont pas des budgets conseillés par objet. Il faut compter points générés, visibles et effectivement dessinés, et surveiller une troncature silencieuse.
- Le chargement à la demande aide le lecteur mais peut retarder l'explorateur. Il faut les deux mesures.
- Le parcours à 2 Mo et les objectifs de mémoire ne prouvent rien sur les services futurs. Les données humaines persistantes ne se recréent pas avec une graine.
- Aucun résultat de PC, d'émulateur ou de Chrome sans écran ne sera présenté comme une mesure de téléphone.

## Expérience ou preuve requise

Construire la V1, exécuter la [recette](RECETTE.md) sur un commit figé, joindre les mesures brutes et les parcours filmés, puis soumettre les échecs et le verdict de Yocthan. Les fonctions absentes reçoivent l'état `BLOQUÉ`, les cas non lancés `NON EXÉCUTÉ` ; aucun n'est compté comme réussi.

## Documents à mettre à jour après construction

Par le responsable de chaque document : guide et leçons pour toute notion ajoutée ; `COMPARAISON-WEB.md` et `NOMS.md` pour les capacités et limites ; `moteur/README.md` pour les mesures reproductibles ; journal pour les résultats et les erreurs. Toute proposition de nouvelle décision reste séparée et attend Yocthan.

## Priorités, du plus important au moins important

1. Livrer le parcours local complet et vérifier qu'il plaît à Yocthan, avec le même état dans les deux vues.
2. Vérifier le téléphone modeste, le poids réel, le premier geste, la première saisie et quinze minutes d'exploration.
3. Assurer lecture, clavier, zoom de lecture, réduction des mouvements et retour à plat.
4. Tester déterminisme, limites, passages répétés et erreurs ; conserver une preuve par cas.
5. Faire créer et corriger les fichiers par des débutants ; améliorer le guide selon leurs blocages.
6. Ajouter envoi de demandes et listes reçues quand leur écriture est choisie ; éprouver de vrais besoins de site.
7. Comparer les objets 3D pleins sur téléphone avant de choisir leur technique.
8. Décider puis éprouver le jeu à plusieurs, après la validation locale ; ne pas commencer par son serveur.
