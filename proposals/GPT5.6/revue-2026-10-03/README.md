# Revue Codex du 2026-10-03

## Contribution

- Sujet : vérification exécutée du sprint Big Bang, avis sur la direction et propositions pour le langage.
- Discussions sources : HC-013 (lue), HC-001/002/003/004/006/007/011/012 comme filiations citées par les ADR ; leurs transcriptions n'ont pas été relues pour cette revue.
- Décisions concernées : ADR-007 à ADR-015 ; également ADR-003 à ADR-006.
- Statut proposé : **PROPOSITION** pour les choix de langage ; les mesures ci-dessous sont des observations, pas des validations d'ADR.
- Auteur : Codex. Seul Yocthan valide. Aucun statut de décision modifié, aucune fusion.
- Référence vérifiée : `origin/main`, commit `ac6a10dac5772ed8d16c544a6f9041f4ae7246b8`, copie isolée pour préserver les modifications locales.

## Résultat

### Vérifications réellement exécutées

Windows x86_64, Rust 1.99.0 (`b940084d7`, 2026-09-28), Python 3.14.0, Node 22.21.0, wasm-bindgen 0.2.100. Rust était installé hors PATH : commandes cargo exécutées avec `C:\Users\mokea\.cargo\bin\cargo.exe`. Cible wasm32 disponible. Ni téléphone ni rendu dans un navigateur testés dans cette revue.

| Commande et dossier | Résultat exécuté | Comparaison documentaire |
|---|---|---|
| `cargo test`, `moteur/` | **18 réussis, 0 échec**, 0 doctest | README : 17/17, donc un test de moins que l'état actuel ; journal initial : 17 ; AGENTS : 18 |
| `cargo build --release --target wasm32-unknown-unknown`, `moteur/` | **Réussite** | Compilation confirmée, comportement graphique non vérifié |
| `python verifier_suite.py`, `experiments/conformite-v0.1/` | **11 cas, 4 acceptés, 7 refusés ; suite bien formée**, sortie 0 | Conforme au README ; ne prouve aucun verdict d'un moteur |
| `python -m unittest discover -s tests`, `proposals/GPT5.6/holocode-v0.1/` | **8 tests, OK**, sortie 0 | Conforme aux 8 tests annoncés dans le journal |

Le test `toucher_un_point_le_rend_cible_et_la_rotation_ne_change_plus_la_cible` figure parmi les 18 exécutés. Le chiffre 17 du journal est un compte historique du sprint initial : le conserver comme tel, puis ajouter le compte après correction ; corriger le tableau courant du README.

Après compilation, j'ai exécuté wasm-bindgen avec `--target web --no-typescript`, puis Brotli qualité 11, comme le script de construction et le serveur :

| Fichier produit | Octets bruts | Octets Brotli |
|---|---:|---:|
| `holo_moteur_bg.wasm` | 1 941 798 | **501 856** |
| `holo_moteur.js` | 111 093 | 14 247 |

Le WASM fait **1 941,798 Ko décimaux / 1 896,287 Kio**, et son corps Brotli **501,856 Ko / 490,094 Kio**. Le README annonce 1 893 Ko réels et 489 Ko transférés : je ne reproduis pas exactement ces chiffres. Le script et l'affichage divisent par 1024 tout en écrivant « Ko », alors que la suite définit Ko = 1000 octets. Si 489 signifie Kio, l'écart est environ +1,094 Kio (+0,22 %) ; si c'est Ko décimal, +12,856 Ko (+2,63 %). Le JS correspond à environ 14 Kio arrondis. WASM + JS compressés = 516 103 octets ; HTML, mesures.js, monde, en-têtes et autres échanges ne sont pas inclus.

**C'est une mesure locale du corps compressé, pas un transfert HTTP observé.** Le budget sous 2 Mo pour ce paquet est respecté ; première image, mémoire, batterie et fluidité sur téléphone restent inconnues. Faute du manifeste de mesure initial, je n'attribue pas l'écart à une cause certaine : modification du moteur ou environnement de construction sont des hypothèses.

Sources : [README moteur][moteur], [journal][journal], [AGENTS][agents], [construction][construction], [serveur][serveur], [mesures][mesures], [suite][suite].

### Avis franc sur la ligne du projet

**Je soutiens la direction, mais Big Bang prouve aujourd'hui un noyau de navigation procédurale, pas encore la mise à jour du web.** Le progrès depuis les prototypes Python est concret : compilation pour navigateur et cœur testé. L'étape décisive suivante est une même boutique utilisable dans les deux vues, avec une action et un état communs.

- **ADR-007, deux vues : solide comme objectif produit.** Garder les liens, le contenu et une vue à plat est une bonne façon de rendre le projet utile. Fragilité : « une page et un point sont la même chose » est une analogie d'arbre, pas une sémantique d'interface. Un texte long, un formulaire ou une liste ont besoin d'un comportement propre ; les transformer en sphères ne suffit pas. Je définirais pour chaque bloc contenu, rôle accessible et actions communs, puis un rendu par vue. Un achat ou une demande de visite doit se retrouver dans le même état après bascule.
- **ADR-008, source réelle : à conserver.** C'est la base du débogage et du partage. Ajouter une version du langage et du générateur : une graine seule ne préserve pas un monde si l'algorithme change. Séparer égalité de l'état et du journal, géométrie sous tolérance documentée, et pixels qui peuvent varier selon le rendu. Les tests actuels comparent deux exécutions sur la même machine, pas deux téléphones.
- **ADR-009, blocs et Markdown : bon compromis**, comparable dans sa forme aux interfaces déclaratives citées dans la fiche. Mais parenthèses, virgules, références et règles restent de la programmation pour un débutant. Le gain devra se mesurer : exemples courts, complétion et erreurs correctives. Pas besoin de revendiquer un paradigme inédit ; la reformulation d'ADR-003 et la composition d'ADR-004 sont honnêtes et utiles. ADR-006 peut rester à discuter au moment où un besoin d'IR devient concret.
- **ADR-010, Rust/WASM : désormais étayée pour compilation et taille.** Elle reste à éprouver pour chauffe, démarrage et WebGL/WebGPU. Le natif futur ne constitue pas encore un navigateur : réseau, URL, sécurité, texte, accessibilité et mises à jour restent du travail. Je différerais ce navigateur jusqu'à une preuve d'usage web. Rust ne garantit ni absence de bug logique ni respect du budget.
- **ADR-011, HTML/CSS à plat : choix pragmatique.** J'en ferais le prochain sprint, avec chargement du moteur 3D à la demande. Le contrat doit préserver ordre de lecture, clavier, focus, navigation arrière et liens profonds. Ces besoins ont une référence testable dans [WCAG 2.2](https://www.w3.org/TR/WCAG22/). Une page accessible ne découle pas automatiquement de HTML généré.
- **ADR-012, ponts : utiles, mais contradiction potentielle avec ADR-015.** Un pont ne devrait recevoir que des messages et proposer des demandes à l'arbitre ; aucun accès direct à l'état autoritaire. Un pont propriétaire privilégié doit annoncer quelles garanties il abandonne. Je commencerais par un seul adaptateur démontré, sans construire tout l'écosystème.
- **ADR-013, deux étages : sain ; confinement : pas encore prouvé.** WebAssembly apporte une isolation de mémoire et passe par les interfaces de l'hôte ([modèle de sécurité officiel](https://webassembly.org/docs/security/)). Il faut encore réaliser limites de mémoire, interruption, droits et protocole d'échange. L'étiquette WASM ne démontre pas qu'un module infini est interruptible ni qu'il ne bloque pas l'interface. Tester une boucle infinie avant de promettre une « boîte fermée ».
- **ADR-014, IA extérieure : très bon principe.** Sauvegarder la source et journaliser les événements extérieurs évite de rendre la lecture dépendante d'une IA. « L'IA ne peut pas dégrader un monde » est toutefois trop fort : des actions autorisées peuvent être inutiles ou abusives, et un contenu valide peut être mauvais. Définir quotas, droits et relecture du journal ; `graine: auto` doit être matérialisée à la création.
- **ADR-015, arbitre : la bonne restriction à protéger.** Pour une commande réelle, il faudra un acteur identifié, des paramètres, une autorité serveur et des transactions ; l'arbitre du client ne valide pas un paiement. Conflits, atomicité et portée des lois sont encore des questions documentées, pas résolues par Big Bang. Commencer par des demandes simples et un journal explicable, sans promettre la détection statique de tous les conflits.

Sources : [registre et dette décisionnelle][decisions], [fiches ADR][adr], [HC-013][hc], [univers][univers], [navigation][navigation]. Les comparaisons ECS/règles/interfaces déclaratives viennent des alternatives déjà exposées dans les ADR ; aucune note comparative de performance n'est déduite des tests ici.

### Défauts et limites observés

Les sondes de cette PR ont été exécutées via `cargo run --manifest-path proposals/GPT5.6/revue-2026-10-03/Cargo.toml --offline` :

```text
seed 9007199254740992 => Ok(9007199254740992)
seed 9007199254740993 => Ok(9007199254740992)
ignored import => Ok(1)
unit with whitespace => Err(... colonne: 38 ...)
boutique syntax => Ok("Page")
boutique semantics => Err(... ce sprint ne lit que « Point » ...)
```

1. **Graines : perte d'identité confirmée.** Le lecteur passe les nombres par f64 : les deux littéraux distincts ci-dessus deviennent la même graine. Lire les graines comme entiers exacts u64, définir bornes et sérialisation sans passage flottant. Le moteur n'accepte pas aujourd'hui toute la plage u64 (borne `1.8e19`).
2. **Imports : promesse silencieuse.** `import "absent.holo" Point(nom: A, graine: 1)` est accepté. Le lecteur enregistre l'import, mais la conversion en monde ne le traite pas. Tant que la résolution manque, refuser explicitement un import dans le sprint ; ne pas faire croire qu'il a été appliqué.
3. **Test tautologique.** Dans `les_valeurs_sont_figees`, l'assertion `graine_enfant(1, 0) == constante ^ graine_enfant(1, 0) ^ constante` se réduit à `x == x`. Elle ne fige aucune valeur enfant. Remplacer par une vraie valeur attendue et plusieurs vecteurs indépendants. Le test passe actuellement ; sa portée est insuffisante. [Source][graine].
4. **Mémoire : formulation à corriger.** La pile de navigation est un `Vec<(u64, usize)>`, donc croît avec la profondeur ; le journal dit 16 octets par niveau, le README parle de 8 octets de graine. Le test de pile est natif et ne mesure ni capacité allouée, ni heap WASM, ni GPU. `usedJSHeapSize` n'est pas la mémoire totale de l'onglet. Remplacer « stable avec la profondeur » par un budget du monde actif, une pente autorisée pour le chemin et une profondeur maximale ; mesurer réellement les allocations. Sources : [navigation][navigation], [mesures][mesures].
5. **« Rien n'est stocké » doit devenir « décor régénérable ».** Source, chemin, monde actif et aperçu existent en mémoire. Dans une boutique, stocks et commandes devront être persistés ; une graine ne recrée pas les achats. [Source][univers].
6. **L'EBNF autorise un espace avant une unité par sa présentation, le lecteur exige la contiguïté** : `500Ko` fonctionne dans les tests, `500 Ko` est rejeté dans la sonde. Définir précisément les règles lexicales plutôt que laisser cette différence implicite.

### a. Les mots : français d'abord, deux profils explicites à terme

Je recommande **un vocabulaire français canonique pour v0.1** (`Page`, `Texte`, `Bouton`, `graine`, `Quand`, `touche`), avec `import`, `module`, `pont` conservés comme mots techniques fixés par ADR-013. Les mots « import » et « module » ne nécessitent pas une seconde grammaire. Paramètres sans accents comme aujourd'hui ; accents dans les textes. Puis, si l'essai le justifie, accepter un profil anglais par table d'alias exhaustive, normalisée vers le même arbre : `Texte/Text`, `Bouton/Button`, `graine/seed`, `Quand/When`, `touche/tap`.

Exemples de la même intention : `Bouton(nom: Ouvrir, texte: "Entrer")` et, dans un futur profil anglais, `Button(name: Ouvrir, text: "Entrer")`. Les noms créés par l'auteur, comme `Ouvrir`, ne sont pas traduits. Le signal du profil est normalisé vers la même opération interne.

Pour un novice francophone, le français réduit l'effort de vocabulaire, mais ne supprime pas la difficulté des références et de l'état. Pour un anglophone, un profil anglais rendra la documentation plus accueillante. Accepter librement tous les synonymes mélangés dès maintenant multiplie les erreurs, les exemples et les diagnostics : je l'éviterais. Un formateur émet un seul profil, un doublon `graine`/`seed` est refusé après normalisation. Un futur `langage: "fr"`/`"en"` dans la racine est une option de conception à spécifier, pas une syntaxe déjà validée.

Pour les IA, l'anglais n'est pas une preuve de fiabilité : HoloCode est neuf dans les deux langues. Fournir grammaire, catalogue typé de blocs, version et quelques exemples conformes dans un profil unique ; tester génération et réparation automatique. Français seul est moins coûteux aujourd'hui ; deux profils bien bornés sont préférables à deux langages concurrents.

### b. La forme : Theme et exceptions locales

Je recommande **les deux**, avec priorité simple : défauts du moteur < Theme de la page < paramètres locaux. Aucun sélecteur CSS ni cascade cachée. Theme contient des valeurs de présentation typées, les blocs ne peuvent surcharger que les propriétés déclarées pour eux. Séparer taille de texte en vue à plat et dimensions spatiales d'un monde ; ne pas prétendre qu'un pixel est un mètre.

```holo
Page(
  titre: "Atelier",
  theme: Theme(
    fond: "#101820",
    texte: "#F4F1EA",
    accent: "#E9B44C",
    espacement: 16,
    taille_texte: 18,
  ),
  contenu: [
    Texte("Bienvenue"),
    Bouton(nom: Ouvrir, texte: "Entrer", fond: "#245C45"),
  ],
)
```

Cette proposition emploie uniquement les formes de valeurs déjà permises par l'EBNF. **Theme et ces paramètres n'ont pas de sémantique implémentée.** Ici `espacement` et `taille_texte` seraient des unités logiques de présentation à définir, pas des longueurs de monde. Prévoir contraste et zoom du texte ; montrer une erreur précise pour un paramètre inconnu. Theme évite vingt corrections répétées ; l'exception locale permet un bouton particulier sans créer un langage de styles complet.

### c. Couleurs d'un Point : graine par défaut, auteur prioritaire

Je recommande **les deux**, avec des modes explicites :

```holo
Point(nom: A, graine: 42, couleur: graine)
Point(nom: B, graine: 42, couleur: "#E9B44C")
Point(nom: C, graine: 42, palette: ["#E9B44C", "#245C45", "#F4F1EA"])
```

Paramètre absent = `couleur: graine`. Couleur fixe = apparence du point lui-même ; palette = choix déterministe pour ses enfants. `couleur` et `palette` peuvent coexister puisqu'ils ciblent des niveaux différents. L'intérieur ne change pas de palette par héritage implicite : une palette de son Monde est une déclaration séparée. Pas de `couleur: auto` à la lecture. Un tirage auteur automatique est enregistré à la création, comme la graine. Utiliser un flux dérivé distinct pour l'apparence afin que changer une palette ne déplace pas les enfants ; versionner cet algorithme. Les textes et boutons utilisent des couleurs accessibles de Theme, jamais un contraste tiré au hasard. Ce sont des paramètres **proposés**, actuellement rejetés par `point_depuis`.

### Une page complète de boutique

Le fichier [boutique.holo](boutique.holo) contient trois objets (lampe 45 USD, carnet 12 USD, bol 18 USD), une prise de contact en Markdown et un atelier avec entrée et retour via `Quand`. L'adresse `atelier@example.com` est un emplacement de démonstration explicite, pas une adresse réelle attribuée à Yocthan. C'est une boutique vitrine réaliste, **pas une vente avec paiement implémenté**.

Le fichier utilise les blocs et les formes du brouillon actuel, sans y glisser Theme, Produit ou Panier comme s'ils existaient. Il est accepté **syntaxiquement** par `holo::lire`, et refusé **sémantiquement** par le moteur qui ne traite que Point. Les descriptions de l'établi et du tour sont du texte : aucune géométrie 3D de mobilier n'est exprimée.

Ce qui manque pour le rendre utilisable :

- **Sémantique, plutôt qu'EBNF** : types et paramètres de Page/Texte/Bouton/Monde/Quand, rendu à plat et en profondeur, portée des noms. Le Retour dans le Monde est référencé depuis les règles de Page : je propose une portée des noms à la page, avec interdiction des doublons ; définir activation des signaux, autorisation d'entrer/sortir et remise du focus.
- **Markdown** : dialecte, liens autorisés, échappement des guillemets et triples guillemets, indentation et sécurité du rendu. Les liens mailto sont une intention de ce brouillon, pas une capacité vérifiée aujourd'hui.
- **Routage** : URL stable vers l'atelier, historique arrière, comportement d'entrer en vue à plat. Je proposerais que l'entrée sélectionne le contenu intérieur dans les deux vues, avec transition spatiale seulement en profondeur.
- **Commerce véritable** : blocs de produit et prix typés (montant exact, devise), panier et quantités, capacités paramétrées, messages de refus, persistance et autorité serveur. L'EBNF actuelle ne décrit ni déclaration de capacité, ni calcul monétaire, ni fonction pure générale. Le catalogue textuel contourne ces absences ; il ne les résout pas.
- **Présentation et espace** : Theme, mise en page adaptative, images avec texte alternatif, objets/formes et positions pour un atelier réellement visitable. Les blocs génériques sont déjà analysables ; leur ajout demande surtout un schéma et une sémantique, pas forcément plus de ponctuation.
- **Lexique** : plage exacte des entiers, NOM et Unicode, échappements, blancs devant les unités, nombre maximal de niveaux et version du fichier. L'EBNF n'offre qu'un segment après le point, le lecteur accepte davantage : harmoniser avant de publier une grammaire normative.

## Objections et limites

Cette revue ne confirme ni « tout téléphone actuel », ni le repli graphique, ni les captures, ni un monde partagé. Elle ne relit pas toutes les transcriptions. Les huit tests Python concernent une syntaxe antérieure ; ils ne démontrent pas les comportements de la boutique. Les 18 tests natifs ne remplacent pas l'exécution WASM. Les exemples Theme/couleur/profil anglais sont des propositions et ne sont pas déclarés conformes.

Le sprint apporte une preuve réelle mais limitée ; développer le langage généraliste, un navigateur, la 3D procédurale, les modules et le commerce simultanément disperserait cette preuve. Je changerais l'ordre du travail, sans changer les statuts : **boutique dans les deux vues, navigation fiable, mesure téléphone, puis une action avec état arbitré**. Les ponts avancés et le navigateur natif attendraient ce résultat.

## Expérience ou preuve requise

1. Mesure reproductible Z Flip 5, WebGL 2 puis WebGPU sécurisé : trois sessions de 15 minutes, cache froid/chaud, luminosité et profondeur fixées ; octets exacts, démarrage, p95/p99 des images et pics à l'entrée, mémoire totale disponible et limites de l'outil, batterie. Distinguer coût du monde actif et coût du chemin. Relever également un appareil moins puissant avant de généraliser.
2. Un exécuteur de conformité Rust qui charge chaque paire `.holo`/JSON, compare arbre, diagnostics, scénario et journal ; rapporter explicitement les cas non implémentés. Premier objectif : les 11 cas, sans annoncer que le contrôle JSON constitue ce passage.
3. Boutique : même source, trois produits, entrée/sortie, URL directe, arrière, clavier et lecture d'écran ; état conservé pendant la bascule. Si entrer n'aide personne à comprendre l'atelier, revoir l'usage de la profondeur, pas seulement l'effet visuel.
4. Essai langage : cinq novices francophones écrivent ou modifient une page après un exemple ; retour après un mois. Comparer compréhension et erreurs des mots et du Theme. IA : un corpus de vingt tâches, taux de conformité au premier essai puis après correction, même documentation et budget pour chaque profil.
5. Régressions ciblées : graines 2^53 et 2^53+1 distinctes, limites u64, vrais vecteurs figés, import manquant refusé, portée du Retour, paramètres inconnus, unités avec ou sans espace. Ce sont des cas proposés à ajouter, pas des tests déjà présents et passés.
6. Modules/ponts : boucle infinie interrompue, dépassement mémoire contenu, écriture directe refusée, résultat externe journalisé, interface toujours réactive. Rejeu d'un même journal sur deux machines, sans IA active.

## Documents à mettre à jour

- `moteur/README.md` : 18 tests, unités Ko/Kio, octets exacts et commit de mesure, périmètre du poids, mémoire de chemin et limites du compteur JS.
- `docs/06-journal/JOURNAL.md` : ajouter cette revue et les résultats après corrections, conserver les 17 tests historiques.
- `experiments/conformite-v0.1/README.md` : lexique, portée des noms, contrat des blocs et différence validation de suite / conformité moteur ; après décision de Yocthan, ajouter boutique et régressions.
- `docs/02-gouvernance/DECISIONS.md` et fiches ADR : enregistrer uniquement les choix validés par Yocthan ; aucune transition de statut demandée automatiquement. Nuancer les promesses absolues d'ADR-014 et documenter les garanties des ponts.
- `docs/05-discussions/HC-013-web-metavers-et-format-holo.md` : l'annoter comme synthèse historique ; son « aucune décision mise à l'épreuve » précède maintenant Big Bang.
- `AGENTS.md` : pointer l'issue de revue et le chantier retenu une fois discuté.

## Reproduire les preuves jointes

Depuis la racine :

```powershell
cargo run --manifest-path proposals/GPT5.6/revue-2026-10-03/Cargo.toml --offline
```

Les sondes affichent les résultats et sortent avec code 0 : **ce n'est pas une suite attestant la correction des défauts**. Pour la taille, après la construction WASM et wasm-bindgen décrite dans le README moteur :

```powershell
node proposals/GPT5.6/revue-2026-10-03/mesurer.cjs
```

Les fichiers joints sont exclusivement dans `proposals/GPT5.6/revue-2026-10-03/`. Aucun code moteur n'est modifié.

[agents]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/AGENTS.md
[journal]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/docs/06-journal/JOURNAL.md
[decisions]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/docs/02-gouvernance/DECISIONS.md
[adr]: https://github.com/yocthanmabeka/Metaverse/tree/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/docs/02-gouvernance/adr
[hc]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/docs/05-discussions/HC-013-web-metavers-et-format-holo.md
[moteur]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/moteur/README.md
[suite]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/experiments/conformite-v0.1/README.md
[construction]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/moteur/outils/construire.ps1
[serveur]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/moteur/outils/serveur.mjs
[mesures]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/moteur/web/mesures.js
[graine]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/moteur/src/graine.rs
[univers]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/moteur/src/univers.rs
[navigation]: https://github.com/yocthanmabeka/Metaverse/blob/ac6a10dac5772ed8d16c544a6f9041f4ae7246b8/moteur/src/navigation.rs
