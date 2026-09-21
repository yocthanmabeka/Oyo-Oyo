# Proposition code by GPT5.6 — HoloCode v0.1

Première implémentation expérimentale du paradigme holoscénique.

## Exécution

Depuis ce dossier :

```bash
python -m holocode examples/automatic_door.holo --ticks 2
```

## Tests

```bash
python -m unittest discover -s tests -v
```

## Structure

- `holocode/` : lexer, parseur, AST, CLI et runtime ;
- `examples/` : scènes HoloCode exécutables ;
- `tests/` : tests de la sémantique ;
- `pyproject.toml` : paquet et commande `holocode`.

Cette proposition est expérimentale. Elle ne devient une partie officielle du langage qu'après validation et enregistrement dans le registre des décisions.
