# Paradigme holoscénique

**Statut : `PROPOSITION`**
**Discussions sources : `HC-001`, `HC-002`, `HC-003`**

## Définition fondatrice

Le paradigme holoscénique est un modèle de programmation proposé dans lequel les **mondes**, les **espaces**, les **entités**, les **relations**, les **lois**, le **temps** et les **phénomènes** constituent des abstractions fondamentales.

Il décrit des systèmes computationnels spatiaux et temporels comme des environnements dont les éléments évoluent et interagissent selon des règles explicites.

**Formule de travail :**

> Monde + Entités + Relations + Lois + Temps → Phénomènes → Évolution du monde

## Abstractions proposées

| Concept | Rôle provisoire | Question à formaliser |
|---|---|---|
| Monde | Conteneur cohérent de lois, espaces et états | Frontières, cycle de vie et identité |
| Espace | Référentiel spatial typé | Géométrie, unités et conversion |
| Archétype | Structure réutilisable et contraintes | Différence exacte avec type/prototype |
| Entité | Élément identifiable du monde | Identité, composition et persistance |
| Capacité | Action ou aptitude composable | Contrats et contrôle d'accès |
| Relation | Lien explicite entre éléments | Indexation, durée et invalidation |
| Loi | Règle générale applicable au monde | Priorité, portée et conflits |
| Phénomène | Transformation déclenchée sous conditions | Atomicité, causalité et annulation |
| Temps | Dimension typée de l'évolution | Horloges, déterminisme et simulation |

## Dix principes de travail

1. Le monde est une abstraction native.
2. Les entités ne dépendent pas obligatoirement de classes.
3. Les relations sont explicites et interrogeables.
4. Les lois peuvent être indépendantes des entités.
5. Les phénomènes représentent des transformations.
6. L'espace, le temps et les unités sont typés.
7. Les changements d'état respectent cohérence et autorité.
8. La persistance suit des politiques explicites.
9. L'exécution peut être parallèle et distribuée, sans prétendre abolir les difficultés des systèmes distribués.
10. La première implémentation reste compatible avec le matériel existant.

## Description honnête (ajouté le 2026-10-03, `ADR-003`, `ADR-015`)

Yocthan a observé que l'holoscénique et l'orienté objet lui semblaient identiques, et il a en grande partie raison. Tout ce qui décrit **ce qui existe** (entités, archétypes, blocs) est de l'objet sans héritage. La différence porte sur **ce qui arrive** : en objet, n'importe quel objet peut en modifier un autre directement ; ici, tout changement d'état passe par un arbitre, le moteur, qui vérifie les lois, détecte les conflits, peut refuser, et inscrit l'opération au journal.

La formule retenue pour ce dépôt est donc : **des objets sans méthodes, des règles au niveau du monde, et des relations.** Ce n'est pas un paradigme nouveau. Chaque brique a un précédent : ECS pour les entités composées, Datalog et Flecs pour les relations interrogeables, moteurs de règles et règles « Instead » d'Inform 7 pour les lois et phénomènes, F# pour les unités, event sourcing pour le journal. La valeur est dans ce que le langage interdit, comme pour Rust face au C : c'est parce qu'aucun code n'est caché dans les objets que le moteur peut tout vérifier avant l'exécution et tout expliquer après.

Règle des appels (`ADR-015`) : les calculs purs sont permis ; les demandes de capacité sont permises et passent par l'arbitre ; le code libre caché dans un bloc est interdit, de même que les méthodes appelées d'un bloc à l'autre et l'héritage.

L'originalité du projet est ailleurs que dans le paradigme : le point qui contient des mondes, le zoom, le web qui devient métavers, le budget de 1 Go (voir la [vision](../00-vision/VISION.md)).

## Différence recherchée avec la POO

La programmation orientée objet organise principalement le logiciel autour d'objets, de responsabilités et de collaborations. HoloCode cherche à organiser une partie du programme autour des mondes, des relations et des règles qui produisent leur évolution.

Cette différence reste à démontrer. Les mêmes applications peuvent déjà être créées avec la POO, un ECS, des acteurs ou un moteur de simulation. HoloCode ne sera original que si sa sémantique apporte un gain mesurable de clarté, de sûreté, de vérifiabilité ou d'optimisation.

## Exemple non normatif

> **Note du 2026-10-03.** Cet exemple garde la syntaxe de départ. Trois défauts y ont été relevés (`HC-012`) : il ne contient aucune loi ; `Near.active` est ambigu dès qu'il y a plusieurs utilisateurs ou plusieurs portes, car une relation est un ensemble de paires et non un booléen ; et `Door.opened = true` écrit l'état directement, ce que `ADR-015` interdit (il faudrait `effect Door.open`, une demande de capacité). La forme définitive sera celle des blocs nommés d'`ADR-009` ; la [suite de conformité](../../experiments/conformite-v0.1/README.md) en donne la grammaire brouillon, et la [proposition de Claude](../../proposals/Claude/holocode-v0.1/README.md) une sémantique exécutable des lois et des capacités.

```holocode
archetype AutomaticDoor {
    state {
        opened: Bool = false
        locked: Bool = false
    }
    capabilities { open, close, lock }
}

world Earth {
    space House {
        entity Door: AutomaticDoor at (0m, 0m, 5m)

        relation Near(User, Door) when distance <= 2m

        phenomenon AutoOpening {
            when Near.active and not Door.locked
            effect Door.opened = true
        }
    }
}
```

La syntaxe ci-dessus illustre l'intention ; elle n'est pas encore une grammaire officielle.

## Ce qu'il reste à prouver

- Les relations peuvent être surveillées sans coût quadratique incontrôlé.
- Les conflits entre lois et phénomènes ont une résolution déterministe.
- La syntaxe reste lisible quand un monde devient complexe.
- HoloIR conserve suffisamment d'information utile pour optimiser l'exécution.
- Le modèle apporte un avantage réel face à une architecture ECS/réactive bien conçue.
