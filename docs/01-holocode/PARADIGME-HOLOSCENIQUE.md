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

## Différence recherchée avec la POO

La programmation orientée objet organise principalement le logiciel autour d'objets, de responsabilités et de collaborations. HoloCode cherche à organiser une partie du programme autour des mondes, des relations et des règles qui produisent leur évolution.

Cette différence reste à démontrer. Les mêmes applications peuvent déjà être créées avec la POO, un ECS, des acteurs ou un moteur de simulation. HoloCode ne sera original que si sa sémantique apporte un gain mesurable de clarté, de sûreté, de vérifiabilité ou d'optimisation.

## Exemple non normatif

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
