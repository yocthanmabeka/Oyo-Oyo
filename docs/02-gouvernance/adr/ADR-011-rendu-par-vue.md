# ADR-011 — L'architecture : le rendu par vue, les ponts, les deux étages, HoloIR

- Fiche réunie le 2026-10-06, à la demande de Yocthan : elle contient, sans rien perdre, les anciennes fiches **ADR-006** (HoloIR), **ADR-012** (les ponts vers JavaScript et CSS) et **ADR-013** (les deux étages et les trois sortes d'import), dont les fichiers ont été supprimés. Leurs numéros restent valables : ailleurs dans le dépôt, « ADR-012 » veut dire « la partie B de cette fiche ».
- Responsable : Yocthan Mabeka
- Discussions sources : HC-007, HC-013, HC-006

## Ce qu'il y a à décider, en un coup d'œil

| Partie | Ce que c'est | Où en est-on | Statut aujourd'hui | Recommandation de Claude |
|---|---|---|---|---|
| **A** (ADR-011) | La page à plat en HTML et CSS fabriqués ; la vue en profondeur par le moteur | **Construit et mesuré** : 60 images par seconde sur deux téléphones ; le site léger (8 Ko) ; les deux vues décrites par le même fichier | **ACCEPTÉ** | **Valider.** Tout ce qui est construit depuis repose dessus. |
| **B** (ADR-012) | Des ponts vers du JavaScript et du CSS existants (`bridge js`, `bridge css`) | **Jamais construit.** Le moteur lit les mots, mais ne les applique pas. | **REJETÉ** | **Ne pas valider, ne pas construire pour l'instant.** Un pont fait entrer du code sans garantie, contre la règle « pas de code libre » (ADR-015, ADR-035) ; Codex l'a signalé. À réexaminer seulement si un vrai site en a besoin. |
| **C** (ADR-013) | Deux étages : HoloCode, et des modules WebAssembly enfermés (mémoire plafonnée, temps limité, droits déclarés) | **Jamais construit**, mais c'est la réponse que Gemini, ChatGPT et Claude ont donnée à la plainte des humains contre l'interdiction du code (le 2026-10-06) | **ACCEPTÉ** pour la direction ; construction à faire | **Valider la direction**, et la construire plus tard. Codex demande de prouver d'abord qu'on peut arrêter un module qui boucle sans fin. |
| **D** (ADR-006) | Garder les unités, l'espace et le temps dans un format intermédiaire (HoloIR) | **Rien n'existe** : le moteur lit directement le `.holo` | PROPOSITION | **Laisser en proposition.** La question se posera avec un format binaire du `.holo`. |

**Décidé par Yocthan le 2026-10-06 : « Personnellement, je suis tes recommandations. »** A est accepté, B est rejeté, la direction de C est acceptée (sa construction reste à faire), D reste une proposition.

**Précisé le même jour, après ses questions sur ce que le web offre gratuitement et sur HoloIR, et confirmé par Yocthan (« on fait comme tu l'as dit »)** : la vue à plat reste du vrai HTML, **même dans le navigateur propre au projet** ; la vue en profondeur recevra un jour une couche invisible pour les lecteurs d'écran. B, C et D restent comme décidé.

---

## Partie A — Le rendu : la vue à plat par génération de HTML et CSS, la vue en profondeur par le moteur (ADR-011)

- Statut : ACCEPTÉ — Décidé par Yocthan le 2026-10-06 : « Personnellement, je suis tes recommandations. »
- Précision acceptée le 2026-10-06 (voir « Ce que le navigateur offre gratuitement », plus bas) — Décidé par Yocthan le 2026-10-06 : « Bon, premièrement, tu travailles sur le lot 2 jusqu'au lot 5. Et ensuite, tu valides les différentes parties qu'on vient de voir. A, B, C et D. […] je suis des recommandations. Donc, du coup, on fait comme tu l'as dit. »
- Date : 2026-09-21
- Projets affectés : HoloCompiler, HoloEngine
- Proposé par : Claude. Validé par Yocthan le 2026-09-21, après lecture. La fusion de la pull request qui introduit cette fiche vaut confirmation.
- Statut révisé le 2026-09-21, sur la remarque de ChatGPT et avec l'accord de Yocthan : la direction est retenue et le travail commence dans ce sens, mais elle reste une hypothèse tant qu'elle n'a pas été mesurée sur un vrai téléphone.
- Depuis (ajout du 2026-10-06) : mesurée sur deux téléphones (Flip 5 et Flip 3, environ 60 images par seconde, 86 à 99 Mo pour l'onglet) ; la page à plat s'ouvre sans le moteur (`ADR-033`).

### Contexte

Il existe deux routes pour faire tourner HoloCode dans un navigateur. **La traduction** : le compilateur transforme le fichier `.holo` en HTML, CSS et JavaScript, que l'auteur ne voit jamais. **Le moteur apporté** : un moteur compilé en WebAssembly lit le `.holo` et dessine lui-même chaque pixel dans une zone de dessin ; c'est ce que font Flutter Web, Figma et Google Earth.

Yocthan préférerait, si possible, qu'il n'y ait aucun HTML ni CSS du tout.

### Décision

Utiliser les deux routes, une par vue (`ADR-007`) :

- **vue à plat** : traduction en HTML et CSS générés ;
- **vue en profondeur** : moteur Rust en WebAssembly (`ADR-010`).

Dans les deux cas, le fichier source ne contient ni HTML, ni CSS, ni JavaScript.

~~Dans le navigateur propre au projet, le moteur assure les deux vues et il n'y a plus aucun HTML.~~ Remplacé le 2026-10-06 : **dans le navigateur propre au projet aussi, la vue à plat reste du vrai HTML.**

### Ce que le navigateur offre gratuitement (précision du 2026-10-06)

Le navigateur ne connaît que ce qu'il a construit lui-même. Une page en vrai HTML lui dit « ceci est un mot, ceci est un bouton » : il offre alors, sans rien coder, la sélection du texte, la recherche (Ctrl+F), la traduction, la lecture par un lecteur d'écran, le remplissage des formulaires et la lecture par les moteurs de recherche. Une page dessinée par le moteur n'est, pour lui, qu'une image : il ne peut plus rien offrir, et il faudrait tout refaire à la main, plus lourd et moins bien (Flutter Web pose pour cela une couche invisible de HTML derrière son dessin).

D'où trois conséquences, acceptées par Yocthan :

1. **La vue à plat reste du vrai HTML, partout**, y compris dans le futur navigateur propre au projet. On ne perd pas les cadeaux du navigateur sans rien gagner en échange.
2. **La vue en profondeur**, dessinée par le moteur, recevra un jour **une couche invisible** décrivant ce qu'on voit (comme Flutter), pour qu'une personne aveugle puisse aussi parcourir un monde. À faire ; pas encore commencé.
3. Ce qui se construit pour la vue à plat (`ADR-033` à `ADR-042`) continue de passer par du HTML et du CSS fabriqués, jamais par un dessin.

### Alternatives étudiées

- **Tout par le moteur**, comme Flutter Web. On perd alors ce que le navigateur offre gratuitement : le texte ne se sélectionne plus, les moteurs de recherche et les lecteurs d'écran ne lisent plus rien, le clavier du téléphone et les formulaires deviennent pénibles, il faut télécharger le moteur avant de voir la moindre page, et la batterie chauffe plus.
- **Tout par traduction** : pas de 3D, pas de zoom continu.
- **Aucun HTML du tout** : impossible dans un navigateur actuel. WebAssembly ne peut parler seul ni à l'écran ni à la carte graphique ; il restera toujours une page d'une dizaine de lignes, identique pour tous les mondes, générée automatiquement.

### Conséquences

- Positives : la vue à plat est légère, lisible partout, y compris sur un téléphone ancien, et trouvable par un moteur de recherche.
- Négatives et risques : deux rendus à garder cohérents ; le passage d'une vue à l'autre doit être fluide, c'est là que l'idée se joue. Le futur navigateur propre au projet devra savoir afficher du HTML (précision du 2026-10-06).

### Critères de validation

- Une même page passe de la vue à plat à la vue en profondeur sans rechargement visible.

### Conditions de réexamen

- Si la cohérence entre les deux rendus coûte plus cher que de tout dessiner avec le moteur.

---

## Partie B — Première version : des ponts vers JavaScript et CSS seulement (ancienne ADR-012)

- Statut : REJETÉ — Décidé par Yocthan le 2026-10-06 : « Personnellement, je suis tes recommandations. » Raison gardée : un pont fait entrer du code sans garantie, contre ADR-015 et ADR-035 ; les modules enfermés (partie C) répondent au même besoin avec des garanties. À réexaminer seulement si un vrai site ne peut pas s'en passer. Les mots `bridge js` et `bridge css` restent lus par le moteur, sans effet, en attendant d'être retirés.
- Date : 2026-09-21
- Projets affectés : HoloCode, HoloCompiler
- Validation : décidé par Yocthan le 2026-09-21. La fusion de la pull request qui introduit cette fiche vaut confirmation.
- Statut révisé le 2026-09-21, sur la remarque de ChatGPT et avec l'accord de Yocthan : la direction est retenue et le travail commence dans ce sens, mais elle reste une hypothèse tant qu'elle n'a pas été mesurée sur un vrai téléphone.
- Depuis (ajout du 2026-10-06) : jamais construite. `bridge js` et `bridge css` sont lus par le moteur mais ne font rien. Codex (revue du 2026-10-03) : « contradiction potentielle avec ADR-015 ».

### Contexte

Yocthan imaginait pouvoir importer n'importe quel langage dans HoloCode : HTML, CSS, JavaScript, C, Python. Sans accès à l'existant, un langage naît dans un désert : personne ne réécrit le paiement, les cartes ou les lecteurs vidéo.

### Décision

Dans la première version, les seuls ponts vers du code étranger sont **JavaScript et CSS**. Sur la cible web ils sont presque gratuits, puisqu'on compile déjà vers eux.

Ce sont des **outils de transition** :

- ils sont réservés au propriétaire de la page ; dans un monde partagé, ils sont interdits ou enfermés ;
- un fichier qui en utilise est marqué « web actuel seulement », car le navigateur propre au projet n'aura pas de moteur JavaScript ;
- le cœur du langage ne doit jamais en dépendre.

### Alternatives étudiées

- **Importer du code source C ou Python** : Python ne tourne pas sur un téléphone ; le C demande de faire correspondre les types et la mémoire. Les autres langages entreraient plutôt comme modules compilés (partie C).
- **Aucun pont** : langage pur, mais inutilisable pour un vrai site avant des années.

### Conséquences

- Positives : accès immédiat à tout l'écosystème du web.
- Négatives et risques : chaque pont perce un trou dans les garanties ; le code importé peut boucler sans fin, ignore le budget mémoire, n'est pas déterministe et peut modifier ce qu'il veut.

### Critères de validation

- Une page `.holo` utilise une bibliothèque JavaScript existante sans que l'auteur écrive de JavaScript.

### Conditions de réexamen

- Quand les modules (partie C) couvrent les besoins, les ponts peuvent être retirés.

---

## Partie C — Deux étages et trois sortes d'import (ancienne ADR-013)

- Construction commencée le 2026-10-06 (`ADR-045`) : la preuve demandée par Codex est faite. Un module qui boucle sans fin est arrêté après son temps, sans bloquer la page ; un module qui réclame trop de mémoire est arrêté par son plafond. Les ponts `bridge js` et `bridge css` sont refusés à la lecture.
- Statut : ACCEPTÉ pour la direction — Décidé par Yocthan le 2026-10-06 : « Personnellement, je suis tes recommandations. » La construction des modules enfermés reste à faire ; elle commencera par la preuve demandée par Codex : arrêter un module qui boucle sans fin. Le troisième import (`bridge`) tombe avec la partie B.
- Date : 2026-09-21
- Projets affectés : HoloCode, HoloCode-Core, HoloRuntime
- Proposé par : Claude. Validé par Yocthan le 2026-09-21, après lecture. La fusion de la pull request qui introduit cette fiche vaut confirmation.
- Statut révisé le 2026-09-21, sur la remarque de ChatGPT et avec l'accord de Yocthan : la direction est retenue et le travail commence dans ce sens, mais elle reste une hypothèse tant qu'elle n'a pas été mesurée sur un vrai téléphone.
- Depuis (ajouts du 2026-10-03 et du 2026-10-06) : l'import de HoloCode existe (`ADR-029`) ; les modules n'existent pas. Codex : « sain ; confinement pas encore prouvé : tester une boucle infinie avant de promettre une boîte fermée ». Le 2026-10-06, Gemini et ChatGPT recommandent précisément ce code enfermé en réponse à la plainte des humains contre l'interdiction du code ; ChatGPT a trouvé un exemple qui marche (Reddit fait tourner du code dans une boîte WebAssembly, sans réseau, arrêtée si elle boucle).

### Contexte

Yocthan a demandé comment écrire en HoloCode un programme d'IA, un programme système ou un jeu vidéo. HoloCode tire ses garanties de ses limites : pas de boucle libre, budget vérifié, état protégé. Un système d'exploitation ou l'entraînement d'un réseau de neurones ont besoin de boucles libres et d'un accès direct à la mémoire. Un même étage ne peut pas être à la fois totalement sûr et totalement libre.

Tous les grands systèmes ont deux étages : Python et le C de NumPy et PyTorch ; Luau et le C++ de Roblox ; C# et le C++ d'Unity ; GDScript et le C++ de Godot ; Verse et Unreal.

### Décision

**Deux étages.**

- Étage 1, **HoloCode** : pages, mondes, règles de jeu, comportements de personnages, utilisation d'une IA. Sûr, petit, réapprenable.
- Étage 2, **modules** : rendu, physique lourde, réseaux de neurones, accès au système. Écrits en Rust, ou dans tout langage qui se compile en WebAssembly (C, C++, Zig, Go). Un module est enfermé : le moteur plafonne sa mémoire, peut l'arrêter s'il prend trop de temps, et ne lui ouvre que ce qui a été déclaré.

Un module doit **se présenter en holoscénique** : quel que soit son intérieur, il expose des archétypes, des lois et des capacités, jamais du Rust. Le moteur lui-même ne s'importe pas ; ses blocs de base sont toujours là.

**Trois sortes d'import**, avec trois mots différents pour que le risque se lise en haut du fichier :

```
import "boutons.holo"          // du HoloCode pur : toutes les garanties
module "physique"              // une boîte fermée : risque contrôlé
bridge js "carte-interactive"  // vieux web : aucune garantie (partie B)
```

On n'importe jamais de code source étranger. La première version contient `import` et le pont ; les modules viennent ensuite, quand le moteur existe.

| Ce qu'on veut écrire | Où | Verdict |
|---|---|---|
| Page, site, monde | HoloCode | Oui, c'est le cœur |
| Jeu vidéo | HoloCode, plus des modules pour le lourd | Oui pour la majorité des jeux |
| Personnages et comportements | HoloCode (lois et phénomènes) | Oui |
| Utiliser une IA | HoloCode, bloc `IA` | Oui |
| Entraîner une IA, calcul lourd | Module | Pas à l'étage 1 |
| Programme système, pilote, compilateur | Rust aujourd'hui | Pas à l'étage 1 |

### Alternatives étudiées

- **Un seul langage pour tout** : il faudrait y remettre les boucles libres et la mémoire brute, et perdre toutes les garanties.
- **Un langage système holoscénique** pour l'étage 2 : gérer des octets ne ressemble pas à un monde avec des entités et des lois ; le « tout est objet » de Java a montré le coût d'un paradigme appliqué partout. À reconsidérer seulement une fois l'étage 1 vivant.

### Conséquences

- Positives : les autres langages entrent par un format universel, et les modules marchent aussi dans le navigateur propre au projet, contrairement aux ponts.
- Négatives et risques : la boîte fermée (plafond mémoire, limite de temps, droits déclarés) est un vrai travail dans le moteur. Il manquait aussi à HoloCode, pour un jeu, le calcul, les capacités avec paramètres, la création et la destruction d'entités, les signaux du joueur, le mouvement continu, le son ; depuis, le son, le mouvement, le clavier et le temps existent.

### Critères de validation

- Un module de démonstration, écrit en Rust, est utilisé depuis un fichier `.holo` sans que l'auteur voie autre chose que des blocs ; le moteur l'arrête s'il dépasse sa mémoire ou son temps.

### Conditions de réexamen

- Si la limite de temps d'un module ne peut pas être imposée dans les navigateurs actuels.

---

## Partie D — Préserver l'information spatiale et temporelle dans HoloIR (ancienne ADR-006)

- Statut : PROPOSITION
- Date : 2026-09-21
- Projets affectés : HoloCompiler, HoloIR
- Proposé par : ChatGPT. Laissé en `PROPOSITION` par Yocthan le 2026-09-21, sur la recommandation de Claude.

### Contexte

HoloIR serait le format intermédiaire entre le fichier source et l'exécution. Les compilateurs classiques aplatissent tout en simples nombres : ils oublient que `2m` était une longueur et `for 3s` une durée. L'idée est de garder ces informations pour que le moteur s'en serve : indexer l'espace, planifier le temps.

### Décision proposée

Conserver dans HoloIR les unités, les espaces, les relations et les durées, au lieu de les aplatir.

### Pourquoi elle reste en proposition

- Aucun HoloIR n'existe. Avec `ADR-008` et `ADR-010`, le moteur lit directement le fichier `.holo`.
- La version binaire compacte du `.holo`, envisagée pour réduire le poids des mondes, jouerait ce rôle : c'est à ce moment-là que la question se posera concrètement.
- La décision n'est pas nécessaire au sprint Big Bang.

L'idée est bonne ; elle est prématurée.

### Critères de validation

- Une mesure montrant qu'un moteur qui garde ces informations fait mieux qu'un moteur qui les aplatit : relations spatiales moins coûteuses, erreurs mieux expliquées.

### Conditions de réexamen

- Quand le moteur Rust existe et qu'un format binaire du `.holo` devient nécessaire.
