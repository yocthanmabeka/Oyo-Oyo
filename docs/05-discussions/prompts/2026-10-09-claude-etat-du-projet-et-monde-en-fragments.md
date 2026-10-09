# Pour la conversation Claude (claude.ai) : l'état du projet, et la réponse à « un monde en fragments »

- Écrit par la session Claude Code du PC, le 2026-10-09, à la demande de Yocthan (« donne un prompt complet de ce qu'on a déjà fait, en répondant à ces réflexions ; à la fin de la 3D, on réfléchira à l'intégration de l'IA »).
- À coller dans la conversation Claude de claude.ai, qui n'a pas accès au dépôt.
- La proposition à laquelle il répond est résumée au § 5.

---

Tu n'as pas accès au dépôt. Voici ce qui existe, puis la réponse de la session Claude Code à ta proposition d'un monde en fragments, puis ce que Yocthan aimerait réfléchir avec toi. Réponds simplement, en français : Yocthan n'est pas programmeur.

## 1. Le projet en bref

- **Holoverse**, et son langage **HoloCode**, sont menés par Yocthan Mabeka. Le dépôt est public (`yocthanmabeka/Oyo-Oyo`), sous la mention « tous droits réservés ».
- **L'idée** : le métavers n'est pas un jeu, c'est une mise à jour du web. Un même fichier `.holo` s'affiche de deux façons :
  - à plat : une page web ordinaire, en HTML fabriqué par le moteur ;
  - en profondeur : un lieu où l'on zoome et où l'on entre.
- **Le Big Bang** : tout part d'un point lumineux, une sphère, qui se morcelle en points, et chaque point contient un monde. Un monde naît d'une graine de 64 bits. La même graine redonne toujours le même monde (`ADR-008`) et rien n'est stocké : « un point pèse une graine ».
- **La limite de perception** (`ADR-005`) : le premier test tient dans 1 Go au maximum, sur n'importe quel téléphone actuel, dans un navigateur. Mais le vrai budget est plus bas : dans un onglet de téléphone, l'onglet est souvent tué vers 300 à 500 Mo. Pour la 3D, l'onglet entier vise moins de 160 Mo au palier léger et 220 Mo au palier normal (`ADR-049`).

## 2. Ce qui est construit

### Le langage HoloCode

- **Des blocs en anglais, imbriqués** : `Page`, `Text`, `H1`, `Button`, `Row`, `Grid`, `Form`, `Repeat`, `Data`, `Point`, `World`, `Module`, `Drawing`, `Chart`…
- **Des styles écrits comme du CSS** : ils ne disent que l'apparence, et le moteur vérifie tout.
- **Des règles** (`On`, `Every`, `When`, `If`) : elles demandent des changements (`cart.add(1)`) à un arbitre, qui tient l'état (`State`). Il n'y a aucun code libre, aucun pont vers JavaScript.
- **En chiffres** : environ 395 mots, presque tous décidés ; plus de cent leçons (une par notion) ; les décisions écrites `ADR-001` à `ADR-092`.

### Le moteur

- **En Rust, compilé en WebAssembly** (`moteur/`) : wgpu, WebGPU avec un repli en WebGL 2.
- **Deux moteurs** : un moteur léger pour les pages (environ 150 Ko), et le dessin, qui n'est chargé que si une page montre des points ou des mondes (environ 500 Ko transférés).
- **Les fichiers clés** :
  - `seed.rs` : les graines ;
  - `universe.rs` : un `Point` devient un monde ;
  - `navigation.rs` : le morcellement, le zoom, l'entrée et la sortie ;
  - `mosaic.rs` : la vue points ;
  - `renderer.rs` et `renderer.wgsl` : le dessin ;
  - `flat.rs` : la page en HTML et CSS ;
  - `rules.rs` : les noms, les règles et les budgets ;
  - `state.rs` : l'arbitre ;
  - `server.rs` : le serveur.

### Le morcellement aujourd'hui

- **Les enfants** : un `Point(fragments: n)` se morcelle en au plus 64 enfants. Chacun a une graine tirée de celle de son parent.
- **Le monde intérieur à l'avance** : en s'approchant, le monde intérieur est déjà calculé, en aperçu, et on le voit avant d'entrer. On entre quand le point couvre l'écran, et rien ne saute.
- **La mémoire** : la pile des niveaux traversés ne garde que deux nombres par niveau. En ressortant, le monde parent est recalculé depuis sa graine.
- **La vue points d'une page** :
  - chaque pixel devient un point, qui se morcelle en grille (`Points(divisions:, levels:)`) ; en 2 × 2, c'est un quadtree ;
  - seuls les points visibles sont dessinés : 1,1 million au repos, jamais plus de 6 344 à l'écran ;
  - le parent s'efface à mesure qu'il se morcelle, et les enfants gardent d'abord sa couleur.
- **Le budget d'un point** : avec `Point(budget: 500KB)`, le moteur refuse un point dont le contenu pèse plus que ce qu'il déclare.
- **Entrer dans un autre fichier** : `Point(inside: "x.holo")`, lu sur le réseau.

### Le web d'abord

Décision de Yocthan : tout le web avant le métavers. Neuf lots sont faits ou en cours de fusion :

- des essais dans un vrai navigateur ;
- les données et le calcul, les formulaires, la mise en page téléphone et ordinateur ;
- un vrai serveur, `holo serve` (Rust, SQLite), qui marche aussi sans JavaScript ;
- des valeurs partagées en direct ;
- des comptes gardés chez l'auteur : mot de passe et code à 6 chiffres, les clés d'accès en cours ;
- les médias, les modules enfermés, le dessin vectoriel, les graphiques, le chronomètre, l'historique, 32 polices libres.

Face au web : sur 131 éléments de HTML, CSS et JavaScript, 98 « oui », 18 « en partie », 8 « non » et 6 refusés exprès.

### Mesuré sur le téléphone de Yocthan

Galaxy Z Flip 5, le 2026-10-04 :

- 59,8 images par seconde en zoomant à travers sept mondes emboîtés (WebGPU) ;
- 59,7 en vue points ;
- 88 Mo pour l'onglet.

Galaxy Z Flip 3 : 60 images par seconde, aussi en WebGL 2.

La batterie et la chaleur dans la durée ne sont pas encore mesurées. Le script est prêt.

## 3. La 3D : décidée, pas encore commencée

- **`ADR-048`** : on vise une qualité, jamais une technique.
- **`ADR-049`** :
  - des objets préparés à l'avance (`holo prepare chaise.gltf` écrit `chaise.holo3d`, avec plusieurs niveaux de détail) ;
  - trois paliers, léger, normal et haut, qui bougent pendant la visite selon le temps d'image. Ils changent un réglage à la fois, avec un écart entre les seuils de montée et de descente, et une courte transition ;
  - WebGL 2 obligatoire ;
  - des budgets de départ : la chaise téléchargée pèse au plus 300 Ko, 1 Mo ou 4 Mo selon le palier ; l'onglet reste sous 160 Mo ou 220 Mo.
- **Le plan** (`docs/04-roadmap/PLAN-3D.md`, à valider par Yocthan) : une chaise réaliste à 60 images par seconde sur le Flip 3, en 12 étapes. Les principales :
  - la profondeur, les conventions, un premier objet plein ;
  - l'outil de préparation ;
  - la matière, les textures ;
  - **les niveaux de détail et le lointain** (étape 8 : de près l'objet complet, de loin simplifié, de très loin en points, sans saut) ;
  - **les paliers qui bougent** (étape 9) ;
  - les mesures, puis les mots du langage.

  La 3D commence après le lot 9 du web.

## 4. Les règles qui comptent pour ta proposition

- **Chez soi d'abord** : aucun prestataire (un hébergeur, un service d'IA…) n'est obligatoire pour qu'un site HoloCode tourne sur le PC de son auteur.
- **Le déterminisme** : le fichier est la vraie source. La même graine redonne le même monde, sur tous les appareils.
- **La 3D s'active dans le fichier** : une page reste une page web normale par défaut.
- **La parité** : tout existe sur le téléphone comme sur l'ordinateur, et tout se mesure sur un vrai téléphone.

## 5. Ta proposition, et la réponse de la session Claude Code

**1. Le fragment comme unité de chargement et de budget**

- C'est déjà le principe : les graines, `Point(budget:)`, la pile de deux nombres par niveau.
- Une correction : 1 Go n'est pas un budget de mémoire réaliste pour un fragment sur un téléphone. Le budget tenu aujourd'hui est celui de l'onglet entier : 88 Mo mesurés, moins de 160 à 220 Mo visés.

**2. Le quadtree sphérique**

- L'arbre existe déjà : le point se morcelle en enfants, et la vue points se coupe en grilles (en 2 × 2, c'est un quadtree).
- Le quadtree *sphérique* découpe la surface d'une planète en tuiles, comme Google Earth. Il ne sert que si un monde a une surface à parcourir, un terrain.
- Ce n'est pas le modèle actuel, où un monde est fait de points. À garder pour plus tard, si on fait des planètes après la chaise.

**3. Le streaming par zones**

- Les mondes se calculent au lieu de se charger : il n'y a pas de « disque » à lire. Le calcul en avance remplace le chargement, et le monde intérieur est déjà calculé en aperçu.
- Le streaming vaudra pour deux choses : les fichiers `Point(inside: "x.holo")`, et les objets 3D préparés (étapes 4 à 8 du plan).

**4. Les transitions fluides**

- C'est déjà une règle (`ADR-049`) : changer de détail seulement quand le nouveau est prêt, avec un écart entre les seuils et une courte transition.
- C'est déjà fait pour les points : le parent s'efface pendant le morcellement, et à l'entrée rien ne saute.
- Applicable tout de suite, en petit : lire à l'avance le fichier `inside` du point vers lequel on zoome.

**5. L'optimisation par seuils**

- Oui pour le garde : c'est l'étape 9 du plan, « les paliers qui bougent », et il ne coûte rien quand tout va bien.
- Non pour une IA réveillée pendant la visite :
  - elle rendrait le monde non reproductible (le déterminisme) ;
  - elle demanderait un service extérieur, ou un modèle de plusieurs centaines de Mo (le budget de l'onglet) ;
  - le garde fait déjà ce travail avec des règles simples.
- Une IA aurait plutôt sa place avant la visite, chez l'auteur : préparer un modèle, simplifier ses niveaux de détail. C'est la réflexion prévue après la 3D.

**Par quoi commencer** : rien de nouveau avant le plan 3D. Ses étapes 8 et 9 reprennent les points 2, 4 et 5 pour les objets. Le seul petit pas utile tout de suite est de lire à l'avance le fichier `inside` du point visé.

## 6. Ce que Yocthan veut réfléchir avec toi

- **Le plan 3D** : ce qui, dans ta proposition, change ou complète ses étapes 8 et 9, et ce qui doit attendre.
- **Les surfaces** : si un jour les mondes ont des surfaces (planètes, terrains), comment un quadtree sphérique s'accorde avec les graines (une tuile = une graine ?) et avec le budget d'un onglet.
- **L'IA dans le langage** : Yocthan n'a jamais vu un langage qui inclut l'IA. Ce sera réfléchi **à la fin de la 3D**. Les contraintes sont déjà connues :
  - aucun prestataire obligatoire : donc un modèle chez l'auteur ou sur l'appareil, ou rien ;
  - le déterminisme : une même page donne un même résultat ;
  - le budget d'un onglet de téléphone ;
  - un point d'appui possible : les modules enfermés (`ADR-045`, `ADR-077`), du code WebAssembly qui reçoit et rend des données bornées, sans pont vers JavaScript.
