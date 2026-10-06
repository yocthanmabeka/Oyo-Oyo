# Recette du site de référence

Cette recette accompagne le [cahier des charges](README.md). **Aucun cas ci-dessous n'a encore été exécuté sur ce site à construire.** Les résultats de projets ou de commits antérieurs ne remplissent pas ses cases.

## Définir le périmètre avant de lancer

La V1 locale comprend les cas `F`, `E`, `R`, `M`, `U` et `C` ci-dessous. Les quatre extensions `X` sont suivies séparément. Une fonction de la V1 absente est `BLOQUÉ`, pas hors périmètre. Les cas et budgets sont fixés avant la mesure ; une modification du périmètre doit être expliquée.

Chaque résultat porte le commit du site et du moteur, la version du générateur, l'appareil, le navigateur, le mode graphique, le réseau, le cache et l'état des valeurs gardées. L'état initial et la suite d'événements sont joints pour les cas qui changent une valeur.

## Fonctions et parcours

| ID | Action et situation | Résultat attendu | Preuve requise |
|---|---|---|---|
| F01 | Ouvrir accueil, catalogue et fiche, JavaScript coupé. | Titres, paragraphes, images décrites et liens lisibles ; aucune réussite fictive pour une action indisponible. | HTML reçu, captures, relevé des requêtes. |
| F02 | À 320, 360, 768 et 1280 pixels CSS ; puis texte à 200 % et changement d'orientation. | Pas de contenu essentiel coupé ni de défilement latéral de la page ; grille adaptée. | Captures et parcours des contrôles. |
| F03 | Remplir le panier : 2 × 120 + 1 × 90 ; retirer un article à 120 ; vider ; retirer encore. | Successivement `(count,total)` : `(3,330)`, `(2,210)`, `(0,0)`, `(0,0)` ; affichage et état d'accord. | Trace d'état et capture après chaque étape. |
| F04 | Saisir texte avec accents et emoji, nombre au-delà du maximum, cocher et décocher ; recharger avec puis sans `keep`. | Libellés reliés aux champs ; bornes documentées ; seules les valeurs gardées reviennent. Le stockage refusé ne casse pas la page. | Valeurs avant/après, capture, essai avec stockage indisponible. |
| F05 | Refaire F03 dans le panneau du monde ; basculer plusieurs fois à plat/en profondeur. | Même état de panier et même résultat ; aucun total calculé par une deuxième logique. | Trace des deux vues. |
| F06 | Entrer par bouton, point visible et pixel planté ; traverser 7 mondes ; visiter un autre fichier ; sortir et utiliser Retour/Avancer. | Bon lieu et bonne adresse ; sortie claire ; état conservé selon le guide ; pas de boucle de retour. | Vidéo, adresses et états successifs. |
| F07 | Ouvrir le carrefour et un portail ; visiter un autre serveur de test. | Destination réelle identifiable avant le passage ; adresse honnête ; pas de confusion entre origine et destination. | Captures du carrefour et de la barre d'adresse. |
| F08 | Lire le journal long ; sélectionner/copier du texte ; suivre ses liens. | Titres dans l'ordre ; Markdown correctement rendu ; `Code` reste du texte ; copie fidèle. | HTML, texte copié et captures. |
| F09 | Changer le menu dans `commun.holo`, puis ouvrir les pages qui l'importent. | Changement partagé ; styles du morceau et priorité de la page conformes au guide ; aucun code produit édité à la main. | Diff du morceau, HTML et captures. |
| F10 | Remplacer le JSON valide par valeurs inconnues, valeurs hors borne, fichier mal formé, erreur HTTP et réponse retardée. | Seules les valeurs déclarées sont actualisées selon leurs bornes ; erreur sans perte des dernières valeurs utilisables ; reprise possible. | JSON exacts, réponses HTTP et traces. |
| F11 | Jeu avant Play, pendant Play, pause/reprise, onglet caché puis visible ; clavier et glissement. | Aucun mouvement avant Play ; pause effective ; reprise sans rattrapage incontrôlé ; mêmes règles pour doigt et clavier. | Vidéo et trace des événements de temps. |
| F12 | Rejouer la même rencontre sur plateau logique à 360 et 1280 pixels CSS, puis en paysage. | Même événement et même score pour les mêmes positions logiques ; différence visuelle due seulement à l'échelle. | Positions, dimensions logiques, traces, captures. |
| F13 | Déclencher le son par un geste ; couper/refuser la sortie audio ; enchaîner arbitrage, saisie, glissement et réception. | Action correcte même sans son ; aucune demande audio ancienne ne ressort d'un autre appel. | Liste des capacités par appel et enregistrement audio si permis. |
| F14 | Jouer entrée, boucle et trois scènes ; activer « réduire les animations ». | Bon ordre et bonnes durées ; dernière scène lisible et arrêtée ; aucun téléchargement du moteur pour ces seuls mouvements. | Vidéo et requêtes réseau dans les deux réglages. |
| F15 | Parcours lecture, panier, atelier et retour au clavier puis au lecteur d'écran. | Contrôles nommés, focus visible, pas de piège ; retour à plat atteignable ; catalogue et panier utilisables. | Compte rendu des touches, annonce du lecteur et blocages. |

## Chargement et erreurs

| ID | Action et situation | Résultat attendu | Preuve requise |
|---|---|---|---|
| E01 | Profil neuf, sans fragment/requête ni stockage ; accueil statique ; attendre 10 s sans geste. | Aucun module du moteur ni WASM demandé. Vérifier ensuite page vivante, valeurs gardées et entrée directe : chargements conformes à ADR-033. | Relevé réseau complet et état de stockage. |
| E02 | Retarder le moteur de 5 s ; trois ajouts, saisie de « Éloïse 🌍 », puis ouverture du menu. | Exactement trois ajouts ; texte complet conservé ; menu demandé ouvert ; focus cohérent après reprise. | Enregistrement, chronologie des requêtes et trace d'état. |
| E03 | Faire échouer la requête du moteur ; retenter ; couper le réseau après le chargement. | Page reçue encore lisible ; attente/erreur compréhensible ; pas de faux succès ni doublon lors de la reprise. Aucun fonctionnement hors ligne non implémenté n'est promis. | Réponses en erreur, vidéo et état. |
| E04 | Point vers fichier absent ; import absent, circulaire ou invalide ; image et son absents. | Erreur explicite selon le cas ; contenu encore utilisable ; aucun passage vers un lieu faux. | Fixtures, messages, ligne/colonne lorsqu'il s'agit du source. |
| E05 | Perte/recréation du contexte graphique ; WebGPU indisponible ; absence des deux API graphiques. | Reprise ou retour à plat sans perte d'état ; WebGL 2 utilisable quand disponible ; lecture et navigation à plat sinon. | Mode annoncé, vidéo et état avant/après. |

## Moteur, rejouabilité et limites

| ID | Action et situation | Résultat attendu | Preuve requise |
|---|---|---|---|
| R01 | `holo check` sur toutes les sources, imports compris ; produire leur HTML ; tests natifs et compilation WASM sur le commit testé. | Codes de sortie corrects ; aucune correction manuelle du HTML ; tous les tests requis exécutés. | Commandes, versions et sorties complètes. Le validateur de structure des cas n'est pas une exécution du moteur. |
| R02 | Graine fixe, événements logiques numérotés identiques ; rejouer en natif et WASM ; revenir dans un monde quitté. | Identité des états et graines entiers ; positions flottantes comparées avec écarts publiés ; générateur versionné. | Source, journal d'événements, sorties et comparaison. |
| R03 | Graines `9007199254740992`, `9007199254740993` et limite entière ; valeurs négatives, maximales et état falsifié, y compris compteur de tirages maximal. | Graines voisines distinctes ; bornes/refus documentés ; aucun panic ni débordement différent entre debug et release. | Sondes et sorties dans les deux compilations. |
| R04 | Pour `When` : vrai maintenu, retour faux puis vrai, plusieurs effets ; tester `within` sur chaque axe. | Déclenchement selon la transition et le guide ; effets dans l'ordre défini ; distance jamais supposée radiale si elle est par axe. | Trace par appel et positions choisies. |
| R05 | Points au repos, zoom/morcellement progressif puis cas de saturation ; grandes images et taille de source excessive. | Budgets respectés ou refus explicite ; pas de disparition essentielle par troncature silencieuse ; site normal sans saturation. | Nombres générés, visibles et dessinés, limites de mémoire et messages. |
| R06 | Source invalide : nom, classe ou propriété inconnus, unité erronée, `javascript:` dans un lien ; texte de saisie ressemblant à du HTML/script. | Refus clair des sources interdites ; saisie affichée littéralement, jamais exécutée. | Fixtures exactes, erreurs et DOM rendu. |

R03 et R04 testent la robustesse locale. Ils ne prouvent pas qu'un futur serveur authentifie un joueur ou filtre les messages ; cela relève de X04.

## Mesure réelle sur téléphone

Appareils requis : un téléphone de Yocthan, un Android modeste avec au plus 4 Go de RAM, et un iPhone/Safari. Consigner le modèle exact, le système et le navigateur ; ne pas confondre mémoire du téléphone et mémoire de l'onglet. Pour Android, couvrir WebGPU quand présent et le mode WebGL 2 forcé. Une machine de bureau sert aux régressions et à la comparaison, sans remplacer ces téléphones.

Un environnement impossible à instrumenter laisse la mesure `NON EXÉCUTÉ` ou `NON CONCLUANT` ; on ne remplit pas sa case avec les chiffres d'un autre appareil. Pour les appareils sans mesure PSS accessible, indiquer la métrique du système disponible et sa limite ; ne pas la rebaptiser PSS. Le verdict de mémoire reste séparé si les métriques ne sont pas comparables.

| ID | Expérience | Critère |
|---|---|---|
| M01 | Mesurer l'accueil froid, tout le parcours froid, puis cache chaud ; séparer HTML, amorçage, moteur, sources, préchargements et médias. | Budgets de poids du cahier respectés ; octets réellement reçus, compression et tailles brutes publiés. Aucun `Content-Length` brut pris pour le transfert compressé. |
| M02 | Réseau à 10 Mbit/s et 100 ms d'aller-retour ; première page, premier geste et première image spatiale, cache vide puis chaud. | Seuils de 1 s et indication sous 100 ms selon le cahier. Origines et fins des chronomètres indiquées. |
| M03 | Réseau à 1 Mbit/s et 300 ms ; répéter E02 et E03. | Pas de geste/saisie perdus ; attente réelle publiée, sans retirer le coût différé du bilan. |
| M04 | Exploration pendant 15 min : zoom, rotation activée, passages entre 7 mondes, retour à plat ; répéter en WebGL 2. | Moyenne, percentiles et pauses conformes au cahier ; publier les trames brutes et les phases, pas seulement une moyenne globale. |
| M05 | 100 entrées/sorties, puis retour au même lieu ; descente automatisée de 1 000 niveaux en banc séparé. | PSS sous 300 Mo ; croissance retenue au plus 10 % après stabilisation ; profondeur sans accumulation des mondes. Le banc n'est pas une promesse d'ergonomie de 1 000 gestes. |
| M06 | Session à plat de 15 min puis exploration de 15 min, ordre alterné, conditions identiques. | Batterie, température et état thermique enregistrés ; cibles de consommation et de cadence du cahier. Résolution insuffisante = non concluant. |

Faire cinq ouvertures froides indépendantes pour M01/M02 et trois sessions pour M04/M06. Rapporter chaque essai, médiane et pire valeur. Les bornes de démarrage s'appliquent à chaque essai ; les percentiles de fluidité sont calculés sur les intervalles de trames de chaque session active. Toute anomalie est expliquée, jamais supprimée du tableau sans trace.

Fixer luminosité, fréquence d'écran à 60 Hz pour le seuil de fluidité, température ambiante, état de charge, chargeur débranché pour la batterie et applications en arrière-plan. Laisser refroidir entre les sessions. Les ouvertures froides effacent cache et valeurs gardées ; les essais de persistance gardent explicitement ces dernières. Utiliser un réseau limité et mesuré : un câble USB vers localhost ne simule pas un réseau mobile lent.

ADB et les outils de Chrome peuvent servir sur Android, conformément à l'outillage du dépôt. Employer les outils disponibles du système sur iPhone et décrire leur méthode. Un compteur JS ne mesure pas toute la mémoire de l'onglet. L'enregistrement d'écran peut modifier la fluidité : mesurer séparément, ou indiquer son coût.

## Public et comparaison

| ID | Expérience | Critère |
|---|---|---|
| U01 | Cinq débutants : lecture, panier, atelier, retour, sans explication orale. | Au moins 4/5 terminent en 5 min ; temps et demandes d'aide consignés. |
| U02 | Les mêmes débutants changent titre, prix et destination d'un point, avec le guide ; puis corrigent une erreur volontaire. | Au moins 4/5 font les trois modifications en 30 min sans HTML/CSS/JS ni IA qui écrit à leur place ; réussite de correction et messages compris rapportés séparément. |
| U03 | Yocthan essaie les deux vues sur son téléphone. | Avis écrit : ce qui est utile, agréable, confus ou refusé. Un refus n'est pas effacé par les tests verts. |
| C01 | Même contenu, médias et parcours à plat en HoloCode et HTML/CSS/JS ; comparer aussi une scène animée commune. | Poids, démarrage, mémoire, fluidité et temps de modification comparés à fonctions égales. Écarts de fonctions explicités ; aucune victoire déduite seulement des lignes. |

La version web comparative peut contenir du JavaScript ; l'interdiction concerne l'auteur du site HoloCode. Les effets web sans équivalent HoloCode restent des manques signalés, pas des tâches réputées couvertes.

## Extensions suivies, non lancées dans la V1

| ID | Test futur | État initial |
|---|---|---|
| X01 | Envoi réel d'un formulaire ; confirmation serveur, validation, coupure et reprise sans doublon. | BLOQUÉ — capacité à définir et construire. |
| X02 | Catalogue reçu comme liste ; filtres, pagination ou limite définie, vide, long et invalide. | BLOQUÉ — listes et répétition à définir. |
| X03 | Objets pleins comparables, occultation correcte, budgets et fluidité mesurés sur téléphone. | BLOQUÉ — représentation 3D à choisir. |
| X04 | Deux joueurs ; messages malhonnêtes, doublons, ordre inversé, délai, perte et reconnexion ; état commun faisant autorité. | BLOQUÉ — reporté jusqu'à validation locale par Yocthan et choix d'architecture. |

## Format de remise des preuves

Après construction, créer un dossier de résultats daté, dans la contribution de l'exécutant. Conserver : manifestes de versions/appareils, sources testées, événements rejouables, sorties de commandes, traces réseau, mesures brutes, captures/vidéos, blocages et comparaison web. Ne pas joindre de vrais noms de clients, de compte ou de secrets.

Une ligne par **cas et environnement** :

| Cas | Commit | Appareil / navigateur / mode | Réseau / cache / stockage | Résultat | Valeur attendue / observée | Preuve | Limite ou défaut |
|---|---|---|---|---|---|---|---|
| F03 | À renseigner | À renseigner | À renseigner | NON EXÉCUTÉ | `(3,330)` après les trois ajouts / non mesuré | Aucune | Site non construit |

Résultats permis : `RÉUSSI`, `ÉCHEC`, `NON EXÉCUTÉ`, `BLOQUÉ`, `NON CONCLUANT`. On calcule la validation sur les combinaisons requises définies avant l'essai : **combinaisons réussies / combinaisons requises**. Les cas de départ E01 et les mesures M se déclinent sur chaque téléphone et mode applicable ; les tests de cœur R se déclinent en natif et WASM quand pertinent ; les essais U se déclinent par personne. Une API graphique absente est un cas de secours E05, pas un test 3D réussi.

Publier séparément la V1 locale, les extensions et les objectifs qui dépassent le panel testé. La V1 vaut 100 % seulement si toutes ses combinaisons requises sont réussies et si le verdict de Yocthan est donné séparément. Un `NON EXÉCUTÉ`, un `BLOQUÉ` ou un `NON CONCLUANT` reste dans le dénominateur ; les extensions ne disparaissent pas du bilan général.
