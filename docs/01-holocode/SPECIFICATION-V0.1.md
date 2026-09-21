# HoloCode v0.1 — Spécification exécutable

**Statut : `EXPÉRIMENTATION`**  
**Implémentation de référence : Python 3.11+**

## Objectif

Cette version teste une seule hypothèse : un programme peut être organisé comme un graphe de monde dans lequel une relation évaluée par le runtime déclenche un phénomène et transforme l'état d'une entité.

```text
Monde → Entités → Relation active → Phénomène → Effet atomique
```

## Programme complet

```holocode
world Building {
    entity User {
        position: (0m, 0m, 0m)
    }

    entity Door {
        position: (0m, 0m, 1m)
        opened: false
        locked: false
    }

    relation Near(User, Door) {
        distance < 2m
    }

    phenomenon AutomaticOpening {
        when Near active and Door.locked == false
        effect Door.opened = true
    }
}
```

## Sémantique du tick

À chaque tick, le runtime :

1. calcule toutes les relations sur un instantané stable ;
2. évalue les conditions des phénomènes ;
3. programme leurs effets ;
4. détecte les écritures contradictoires ;
5. applique les changements ;
6. produit un journal d'événements.

Deux phénomènes qui attribuent des valeurs différentes à la même propriété pendant le même tick provoquent une erreur. Cette règle ne constitue pas encore un système complet d'arbitrage des lois ; elle empêche simplement un résultat dépendant de l'ordre de parcours.

## Constructions prises en charge

| Construction | v0.1 |
|---|---|
| `world` | Un monde par fichier |
| `entity` | Propriétés booléennes, numériques, symboliques et position 3D |
| `relation` | Proximité entre deux entités |
| `phenomenon` | Conditions relationnelles et propriétés |
| `effect` | Affectation d'une propriété |
| Unités | Mètre uniquement |
| Temps | Ticks discrets |
| Événements | Journal des changements effectifs |

## Garanties actuelles

- Les références vers des entités et relations inconnues sont rejetées.
- Les entités utilisées par une relation doivent posséder une position 3D.
- Une relation de proximité compare des positions dans la même unité.
- Les effets sont idempotents : réaffecter la même valeur ne produit pas un nouvel événement.
- Les écritures contradictoires d'un même tick sont rejetées.

## Ce que v0.1 ne fait pas

- archétypes et composition de capacités ;
- lois physiques continues ;
- plusieurs espaces et référentiels ;
- temps continu et planification ;
- persistance, réseau ou distribution ;
- sécurité par capacités ;
- rendu 3D ;
- compilation en HoloIR.

## Critère de réussite

Cette version ne prouve pas encore que le paradigme holoscénique surpasse la POO. Elle prouve que son noyau peut recevoir une sémantique déterministe et testable. La prochaine comparaison devra implémenter plusieurs scènes identiques en HoloCode et dans une architecture POO/ECS, puis mesurer la complexité, les erreurs détectables et le coût d'exécution.
