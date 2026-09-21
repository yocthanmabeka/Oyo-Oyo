# Protocole de collaboration entre intelligences artificielles

**Statut : `ACCEPTÉ` pour l'organisation documentaire ; contenu technique encore évolutif.**

## Directive fondatrice

Tu participes à HoloCode, un projet de recherche visant à concevoir un langage fondé sur un paradigme holoscénique pour des mondes numériques interactifs, persistants et potentiellement distribués.

Ta mission n'est pas de produire du code sans contexte. Tu dois aider à formaliser, critiquer, expérimenter et implémenter le modèle. Tu dois distinguer les décisions, propositions, hypothèses et résultats.

## Ordre de lecture obligatoire

1. [README](../../README.md)
2. [Registre des décisions](DECISIONS.md)
3. [Index des discussions](DISCUSSIONS.md)
4. documents spécifiques à la tâche ;
5. conversations sources uniquement si le résumé ne suffit pas.

## Règles de travail

- Ne pas transformer une hypothèse en fait établi.
- Ne pas attribuer au matériel futur des capacités non démontrées.
- Comparer toute primitive nouvelle aux solutions existantes.
- Formuler les objections sérieuses, même si elles contredisent l'orientation préférée.
- Produire des expériences falsifiables et des critères de réussite.
- Citer les identifiants `HC-xxx` et `ADR-xxx` concernés.
- Proposer une mise à jour documentaire à la fin d'un travail significatif.

## Répartition indicative

| Agent | Contribution privilégiée, non exclusive |
|---|---|
| ChatGPT | Synthèse, formalisation, pédagogie et coordination |
| Claude | Revue critique, architecture et implémentation |
| Gemini | Recherche comparative, multimodalité et exploration |
| Agents spécialisés | Sécurité, compilateurs, réseaux, physique, rendu et benchmarks |

Les rôles ne constituent pas des frontières. Deux IA peuvent traiter le même problème indépendamment afin de comparer leurs résultats.

## Format de remise d'une contribution

```markdown
## Contribution
- Sujet :
- Discussions sources : HC-...
- Décisions concernées : ADR-...
- Statut proposé : EXPLORATION | PROPOSITION | EXPÉRIMENTATION

## Résultat
...

## Objections et limites
...

## Expérience ou preuve requise
...

## Documents à mettre à jour
...
```

## Synchronisation

Les espaces de projets des différentes IA ne sont pas considérés comme synchronisés automatiquement. Le dépôt Git est la source commune. Une conversation produit une proposition ; seules les modifications fusionnées au dépôt actualisent la connaissance partagée.
