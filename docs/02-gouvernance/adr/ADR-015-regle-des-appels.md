# ADR-015 — Règle des appels : tout changement d'état passe par un arbitre

- Statut : PROPOSITION
- Date : 2026-09-21
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013, HC-003, HC-004
- Projets affectés : HoloCode, HoloCompiler, HoloRuntime
- Proposé par : Claude. Non validé.

## Contexte

L'écriture en blocs retenue par `ADR-009` vient de Flutter, qui est orienté objet. Yocthan a observé que l'holoscénique et l'orienté objet lui semblaient identiques, et a demandé s'il y aurait des appels, oui ou non.

Tout ce qui décrit « ce qui existe » est effectivement de l'objet sans héritage : une entité avec son état est un objet avec ses champs, un archétype est une classe, un bloc est la création d'un objet. La différence porte sur « ce qui arrive ».

## Décision proposée

Oui, il y a des appels. Deux sortes sont permises, une troisième est interdite.

1. **Les appels de calcul : permis.** Une fonction reçoit des valeurs et rend un résultat, sans rien modifier dans le monde : `distance(p, d)`, `min(a, b)`, le total d'un panier.
2. **Les demandes de capacité : permises, elles passent par l'arbitre.** `effet: Atelier.entrer` ne va pas directement à l'atelier : la demande passe par le moteur, qui vérifie les lois, détecte les conflits, peut refuser, et inscrit l'opération au journal. C'est un virement bancaire, pas une main dans la caisse.
3. **Le code libre caché dans un bloc : interdit.** Pas de `onPressed: () { commande.etat = "payée" }`.

```holo
fonction total(panier) = somme(panier.prix)          // 1. calcul
Quand(Payer.touche, effet: Commande.valider)         // 2. demande via l'arbitre
```

Pour que HoloCode reste holoscénique, trois choses sont interdites : les méthodes qu'on appelle d'un bloc à l'autre, l'héritage, et le code libre dans un bloc. Les blocs disent ce qui existe ; les règles, toutes visibles au même endroit, disent ce qui arrive.

**Description honnête du paradigme, proposée pour les documents :** des objets sans méthodes, des règles au niveau du monde, et des relations. Ce n'est pas un paradigme nouveau ; c'est de la programmation objet restreinte et recombinée avec des idées anciennes (moteurs de règles, ECS). La valeur est dans ce qui est interdit, comme pour Rust face au C : c'est parce qu'aucun code n'est caché dans les objets que le moteur peut lire toutes les règles, détecter les conflits avant l'exécution, expliquer chaque changement, et laisser des inconnus partager un monde.

## Alternatives étudiées

- **Des rappels libres, comme en Flutter** : pratique, mais plus rien n'est vérifiable, et HoloCode devient de la programmation objet avec un nouveau nom.
- **Aucun appel du tout** : impossible d'écrire un score ou le total d'un panier.

## Conséquences

### Positives

- Précise `ADR-003` et `ADR-004` : une capacité est une demande arbitrée, un phénomène est une règle qui formule des demandes. Conforme au prototype de la PR n° 2.
- La ressemblance avec l'objet rend le langage familier à des millions de gens.

### Négatives et risques

- La tentation sera forte d'autoriser « juste un peu » de code dans un bloc.
- Revendiquer un paradigme révolutionnaire exposerait le projet à une réfutation facile.

## Critères de validation

- Dix programmes d'exemple s'écrivent sans aucun code dans un bloc.
- Le vérificateur refuse un bloc qui contient du code libre.

## Conditions de réexamen

- Si des comportements courants s'avèrent inexprimables sans code libre.
