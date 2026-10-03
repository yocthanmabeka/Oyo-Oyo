# Vision de Holoverse

**Statut : `PROPOSITION`**

> **Mise à jour du 2026-10-03.** La vision de Yocthan, cadrée dans `HC-011` et `HC-013`, et les décisions `ADR-007` à `ADR-015` complètent ce document. Les sections ajoutées sont marquées ; le texte d'origine de ChatGPT est conservé.

## Ambition

Holoverse vise à rendre programmables des mondes numériques persistants dans lesquels l'espace, le temps, les relations, les règles et les phénomènes sont décrits explicitement. L'ambition va du langage jusqu'au runtime et, à long terme seulement, à des interfaces holographiques et architectures matérielles spécialisées.

## La vision de Yocthan (ajouté, `ADR-007`, `HC-013`)

- **Le métavers est une mise à jour du web, pas un jeu.** Les métavers précédents ont proposé un jeu dans lequel il fallait entrer ; or bien plus de gens vivent sur Internet que dans les jeux. « Quelqu'un verra un web normal, mais pourtant c'est le métavers. » Un même fichier s'affiche de deux façons : **à plat**, comme une page ordinaire, et **en profondeur**, comme un lieu où l'on zoome et où l'on entre.
- **Le Big Bang.** Tout part d'un petit point lumineux en 3D qui se morcelle en d'autres points ; à l'intérieur de chaque point se trouve un monde, ou une multitude de mondes. Ce point, une sphère, est aussi le premier personnage et peut prendre n'importe quelle apparence. Un point contient des points comme un bloc contient des blocs.
- **La limite de perception.** Le premier test tient dans 1 Go au maximum et tourne sur n'importe quel téléphone actuel, dans un navigateur (`ADR-005`). Un monde se calcule à partir d'une graine ; il n'est pas stocké.
- **Créer est à la portée de quelqu'un qui n'a jamais programmé.** L'auteur n'écrit jamais de HTML, de CSS ni de JavaScript (`ADR-009`).
- Horizon 2030, par sprints de 24 heures.

Première preuve exécutable : le [sprint Big Bang](../../moteur/README.md), un moteur en Rust piloté par un fichier `.holo` de huit lignes.

## Problème de départ

Les moteurs et langages actuels savent déjà construire des jeux, simulations, jumeaux numériques et univers partagés. Ils le font cependant en combinant de nombreuses couches : langage généraliste, moteur 3D, physique, réseau, base de données, sécurité et outils de création.

La question fondatrice n'est donc pas « peut-on créer un métavers avec C++ ou JavaScript ? » — oui. La question est :

> Peut-on définir un modèle de programmation dans lequel les mondes et leur évolution sont des abstractions natives, formelles et vérifiables ?

## Objectifs

- Définir le paradigme holoscénique avec une sémantique testable.
- Concevoir HoloCode sans dépendre d'un matériel encore inexistant.
- Produire une représentation intermédiaire qui préserve les informations spatiales et temporelles utiles.
- Séparer clairement simulation, rendu, réseau, persistance et autorité.
- Tester les bénéfices face à des solutions existantes : ECS, dataflow, programmation réactive, acteurs et moteurs 3D.
- Préparer des recherches à long terme sur l'algèbre géométrique, le calcul volumétrique, les réseaux déterministes et le matériel spatial.

## Non-objectifs immédiats

- Promettre un monde holographique grandeur nature avec la technologie actuelle.
- Inventer des performances sans preuve.
- Réécrire tous les logiciels existants avant d'avoir démontré la valeur du modèle.
- Faire de HoloCode un langage universel pour chaque type de programme.

## Critère de réussite initial

Un premier prototype est utile s'il permet d'exprimer plus clairement qu'une solution de référence un scénario comprenant :

1. un monde et deux espaces ;
2. plusieurs entités composées ;
3. une relation spatiale typée ;
4. une loi indépendante des entités ;
5. un phénomène déclenché de manière déterministe ;
6. une trace expliquant pourquoi l'état a changé.
