# HoloCode face aux meilleurs de chaque famille

Écrit le 2026-10-04 par Claude, à la demande de Yocthan : prendre le meilleur de chaque famille, l'aligner avec HoloCode, et dire à combien de pour cent HoloCode s'en approche ou le dépasse.

**À lire avant les chiffres.** Ces pourcentages sont le jugement de Claude, pas des mesures, sauf là où c'est écrit « mesuré ». 100 % veut dire « égal au champion » ; plus de 100 %, « HoloCode fait mieux » ; moins, « HoloCode fait moins bien ». Claude juge ici son propre travail : il n'est pas neutre. Le même tableau rempli par Codex et par Gemini dira plus que celui-ci seul.

## Les champions retenus

| Famille | Champion | Pourquoi lui |
|---|---|---|
| 1. Les sites web | **SolidJS** | Le plus rapide des frameworks courants : il ne redessine que ce qui change. (Svelte est le plus simple à écrire ; il sert de second repère.) |
| 2. Les langages hors JavaScript | **Rust** (avec Leptos pour le web) | Le plus rapide et le plus sûr de ceux qui tournent en WebAssembly. C'est aussi le langage de notre moteur. |
| 3a. La 3D dans le navigateur | **Three.js** | La référence : presque toute la 3D du web passe par lui. (Wonderland Engine est plus rapide, mais bien moins répandu.) |
| 3b. Les moteurs de jeu | **Unreal Engine** | Le plus puissant pour l'image. Il ne tourne plus dans un navigateur : un jeu Unreal s'installe, ou s'envoie en vidéo depuis un serveur. |

## 1. Face à SolidJS : faire un site

| Critère | HoloCode face à SolidJS | Pourquoi |
|---|---|---|
| Vitesse du premier affichage | 90 % | Mesuré contre du HTML pur : à peu près égal, parce que le serveur envoie la page déjà fabriquée. |
| Poids à la première visite | 3 % | Notre moteur pèse environ 570 Ko ; un site Solid, 10 à 20 Ko. Trente fois plus lourd. Il n'est téléchargé qu'une fois. |
| Vitesse quand une valeur change | 80 % | Même principe que Solid (seul le texte concerné change), mais on passe par le moteur à chaque clic. Non mesuré. |
| Facilité pour quelqu'un qui ne programme pas | 250 % | Pas de JavaScript, pas d'outil à installer, un seul fichier. |
| Erreurs attrapées avant l'affichage | 150 % | Tout est vérifié, avec la ligne ; Solid laisse passer une faute dans un style ou un nom. |
| Ce qu'on peut construire | 15 % | Pas de formulaire, pas de condition, pas de liste répétée, pas de données venues d'un serveur. |
| Bibliothèques, exemples, entraide | 0 % | Rien n'existe en dehors de ce dépôt. |

**En tout : HoloCode vaut environ 35 % de SolidJS pour faire un vrai site aujourd'hui.** Il le dépasse sur la facilité et la vérification, et il est loin derrière sur ce qu'on peut construire.

## 2. Face à Rust : un langage sérieux

| Critère | HoloCode face à Rust | Pourquoi |
|---|---|---|
| Vitesse de calcul | 100 % | Ce qui calcule, chez nous, c'est du Rust. |
| Sûreté (pas de plantage, pas de fuite) | 110 % | Rust est très sûr ; HoloCode interdit en plus le code libre, donc il y a moins à casser. |
| Facilité | 400 % | Rust est l'un des langages les plus difficiles à apprendre. |
| Ce qu'on peut exprimer | 3 % | Rust sait tout faire ; HoloCode ne sait écrire ni une boucle, ni un calcul, ni une fonction. C'est voulu, mais c'est un fait. |
| Outils (éditeur, tests, débogueur) | 5 % | Une coloration dans VS Code, et un vérificateur. |

**En tout : environ 30 %.** La comparaison est un peu injuste dans les deux sens : HoloCode n'est pas un langage généraliste et ne cherche pas à l'être.

## 3a. Face à Three.js : la 3D dans le navigateur

| Critère | HoloCode face à Three.js | Pourquoi |
|---|---|---|
| Fluidité sur téléphone, pour ce que nous dessinons | 100 % | Mesuré : 60 images par seconde sur deux téléphones, en WebGPU et en WebGL 2. Three.js y arriverait aussi. |
| Poids | 30 % | 570 Ko contre environ 170 Ko (chiffre à vérifier). |
| Facilité | 300 % | Avec Three.js, il faut programmer la caméra, la boucle, les gestes. Ici, on écrit `Point(seed: 42)`. |
| Ce qu'on peut montrer | 5 % | Nous dessinons des points lumineux. Three.js a les formes, les lumières, les textures, les modèles 3D, les ombres, les animations. |
| Le même fichier est aussi un site lisible | hors comparaison | Three.js ne le fait pas : la 3D est une image dans la page. C'est notre seule vraie avance. |
| Lunettes de réalité virtuelle | 0 % | Three.js les gère (WebXR) ; nous, pas du tout. |

**En tout : environ 20 %.**

## 3b. Face à Unreal Engine : un vrai jeu

| Critère | HoloCode face à Unreal | Pourquoi |
|---|---|---|
| Qualité de l'image | 1 % | Unreal fait des films ; nous faisons des points. |
| Ce qu'il faut pour un jeu (personnage, physique, son, niveaux, réseau) | 2 % | Nous n'avons rien de tout cela. |
| Poids | des milliers de fois plus léger | Un jeu Unreal pèse des gigaoctets ; un monde `.holo`, quelques kilo-octets et un moteur de 570 Ko. |
| Tourner sur un téléphone ordinaire, dans un navigateur, sans rien installer | HoloCode le fait, Unreal non | C'est tout le pari du projet. |
| Facilité | 300 % | Unreal demande des mois d'apprentissage. |

**En tout : environ 3 %.** Unreal n'est pas un concurrent : c'est une autre planète. La comparaison utile est avec Roblox, où des non-programmeurs créent des mondes reliés entre eux ; là aussi nous sommes très loin (peut-être 3 %), mais sur le même terrain.

## Le tableau d'ensemble

| Face à | HoloCode aujourd'hui | Où il dépasse | Où il est le plus loin |
|---|---|---|---|
| SolidJS | 35 % | facilité, vérification | ce qu'on peut construire, le poids |
| Rust | 30 % | facilité | ce qu'on peut exprimer |
| Three.js | 20 % | facilité, site et monde dans le même fichier | ce qu'on peut montrer |
| Unreal | 3 % | poids, téléphone, navigateur | presque tout le reste |

Ce que ces chiffres disent : HoloCode ne gagne nulle part sur la puissance. Il gagne partout sur la facilité, et il est seul sur une chose : un fichier court qui est à la fois un site lisible et un monde où l'on entre. C'est là qu'il faut pousser, pas sur la vitesse.

## Ce qu'il faudrait pour écrire un vrai jeu à 100 % en HoloCode

Yocthan : « Il faudra qu'on pense à créer des outils, ou à rajouter des mots dans le langage, pour pouvoir construire un vrai jeu à 100 % sur HoloCode. »

Aujourd'hui, un jeu est impossible : rien ne bouge tout seul, rien ne se décide. Voici ce qui manque, du plus petit au plus gros. Les mots entre parenthèses sont des pistes, pas des propositions arrêtées.

| Rang | Ce qui manque | À quoi ça sert dans un jeu |
|---|---|---|
| 1 | Les conditions (« si le score atteint 10 ») | Gagner, perdre, ouvrir une porte |
| 2 | Le temps (« toutes les secondes », « au bout de 30 secondes ») | Un chronomètre, des ennemis qui avancent |
| 3 | Le hasard maîtrisé (tiré d'une graine) | Faire apparaître des choses à des endroits différents |
| 4 | D'autres signaux que `tap` : clavier, glissement, approche | Diriger un personnage |
| 5 | Des objets qui ont une place et qui bougent | Un personnage, une balle, un ennemi |
| 6 | Les rencontres entre objets (toucher, ramasser, heurter) | Presque toutes les règles d'un jeu |
| 7 | Des valeurs plus riches : texte, oui/non, listes | Un inventaire, un nom de joueur |
| 8 | Le son | Indispensable |
| 9 | Des images et des formes qui s'animent | Un personnage qui marche |
| 10 | Garder la partie, et jouer à plusieurs | Le vrai rôle de l'arbitre |

La contrainte reste la même : pas de code libre. Chaque rang doit s'écrire par des règles (`On(...)`), des valeurs (`State`) et des demandes faites au moteur.

La méthode que Claude recommande : choisir un premier jeu très petit, et n'ajouter au langage que ce que ce jeu demande. Par exemple « attraper les points » : des points apparaissent, on les touche avant qu'ils s'éteignent, le score monte, la partie dure trente secondes. Ce jeu demande les rangs 1, 2 et 3, et rien d'autre.

## Jusqu'où viser dans chaque famille, et dans quel ordre

Yocthan, le 2026-10-04 : « HoloCode ne cherche pas seulement à être un langage, mais un langage qui permet de faire à la fois du web et du jeu dans le web. Donc, en gros, un métavers. » Il demande, pour chaque famille, le maximum à atteindre, et un planning pour y arriver vite, avec un équilibre entre les familles.

Ce qui suit est une proposition de Claude. Les pourcentages sont des jugements, comme plus haut.

### Les cibles

| Face à | Aujourd'hui | Cible | Pourquoi pas plus |
|---|---|---|---|
| SolidJS (le web) | 35 % | **70 %** | Les 30 % restants sont l'écosystème : des milliers de bibliothèques, d'exemples, de gens qui savent. Cela ne se programme pas, cela vient avec des utilisateurs. |
| Three.js (la 3D dans le navigateur) | 20 % | **50 %** | Three.js a quinze ans d'effets visuels. La moitié suffit pour des mondes qu'on a envie de visiter : des formes, des images, des modèles, un personnage. |
| Roblox, puis Unreal (le jeu) | 3 % | **30 % de Roblox, 10 % d'Unreal** | Unreal vise l'image de cinéma sur une machine puissante ; nous visons le téléphone ordinaire. Le bon repère est Roblox : des jeux simples, faits par des non-programmeurs. |
| Rust (le langage) | 30 % | **40 %, sans le chercher** | HoloCode ne doit pas devenir un langage généraliste : c'est en interdisant le code libre qu'il reste facile et sûr. Le calcul libre passera par des modules enfermés (`ADR-013`), écrits en Rust ou autre. |

L'équilibre : le web d'abord, parce que c'est la porte d'entrée de tout le monde (« web d'abord ») ; le jeu ensuite, parce que c'est lui qui donne une raison d'entrer dans les mondes ; la 3D riche en dernier, parce que c'est la plus coûteuse.

### Le planning

Une « séance » est une séance de travail comme celles de ces jours-ci. Les durées sont des estimations.

| Étape | Ce qu'on construit | Ce que ça débloque | Effet attendu | Durée |
|---|---|---|---|---|
| 1 | Les conditions (« si le panier est vide »), les listes répétées, le texte de remplacement des images, le texte qui manque (retour à la ligne, trait, citation, code) | Un site qui s'adapte à ce qui se passe | Web : 35 → 50 % | 3 séances |
| 2 | Le temps, le hasard tiré d'une graine, et un premier jeu entier : « attraper les points » | La preuve qu'un jeu s'écrit sans code | Jeu : 3 → 8 % de Roblox | 3 séances |
| 3 | Les champs de saisie et les formulaires ; garder une valeur après un rechargement | Recherche, inscription, panier qui reste | Web : 50 → 60 % | 3 séances |
| 4 | Des objets qui ont une place et qui bougent ; le clavier et le glissement ; les rencontres entre objets | Un personnage, une balle, un ennemi | Jeu : 8 → 18 % ; 3D : 20 → 28 % | 5 séances |
| 5 | Les imports (réutiliser un morceau de page, un fichier de styles) ; les données venues d'un serveur, par un pont surveillé | Des sites de plusieurs pages, un vrai catalogue | Web : 60 → 70 % | 4 séances |
| 6 | Dans les mondes : des formes simples, des images, le son, puis des modèles 3D | Des mondes qui ressemblent à quelque chose | 3D : 28 → 45 % ; jeu : 18 → 25 % | 6 séances |
| 7 | Garder la partie ; jouer à plusieurs, avec l'arbitre sur un serveur | Le métavers au sens propre : s'y retrouver à plusieurs | Jeu : 25 → 30 % ; 3D : 45 → 50 % | 6 séances et plus |

**Où l'on en est.** Étape 1 faite le 2026-10-04, sauf les listes répétées (`ADR-025`) : conditions, trait, citation, texte tel quel, retour à la ligne, `alt`. Les listes répétées attendent des valeurs plus riches que des nombres. Étape 2 faite le même jour (`ADR-026`) : le temps (`Every`), le hasard (`random`), le plateau (`Board`), et un premier jeu entier, `exemples/jeu/attraper.holo`. Étape 3 faite (`ADR-027`) : la case à cocher, le champ de nombre et de texte, les valeurs de texte, les valeurs gardées (`keep`). Étape 4 faite pour l'essentiel (`ADR-028`) : le clavier (`Key`), les règles qui guettent (`When`, y compris les rencontres), le glissement (`drag`), et un deuxième jeu, `exemples/jeu/panier.holo`. Étape 4 faite.

En tout : une trentaine de séances. À chaque étape, on refait ce tableau, et on demande à Codex et à Gemini de le remplir de leur côté : l'écart entre les trois dira si l'on avance vraiment.

### Ce que le planning ne contient pas, et pourquoi

- **Alléger le moteur** (570 Ko). On peut le couper en deux : une petite partie pour la page, la grosse (le dessin des points) seulement quand on zoome. À faire dès qu'une étape le permet ; cela remonterait la ligne « poids » face à SolidJS.
- **Les lunettes de réalité virtuelle.** Possible après l'étape 6.
- **Les mondes créés à partir d'une vidéo par une IA** (ce que montre Genie). C'est un calcul énorme, qui se fait sur des serveurs, pas sur un téléphone. La place de HoloCode serait d'accueillir le résultat (une vidéo, un modèle 3D) dans un monde, par un bloc ou par un pont. Cela suppose d'abord l'étape 6.
- **Le téléphone d'entrée de gamme**, les **trois testeurs**, **les noms** : ils ne dépendent pas de ce planning, et le planning ne les remplace pas.

### Ce qui peut faire échouer ce planning

- Chaque étape ajoute des mots au langage. Si l'on en ajoute trop, il cesse d'être facile, et c'est sa seule avance. Règle à tenir : pas de mot nouveau sans un exemple réel qui le demande.
- Le jeu pousse vers le code libre (« si ceci, alors cela, sinon… »). Il faudra tenir la ligne : des règles et des demandes, jamais de programme.
