# Vision de Holoverse

**Statut : `PROPOSITION`**

## Ambition

Holoverse vise à rendre programmables des mondes numériques persistants dans lesquels l'espace, le temps, les relations, les règles et les phénomènes sont décrits explicitement. L'ambition va du langage jusqu'au runtime et, à long terme seulement, à des interfaces holographiques et architectures matérielles spécialisées.

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
