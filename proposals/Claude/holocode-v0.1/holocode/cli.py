"""Ligne de commande : vérifier un monde, puis le faire tourner."""

import argparse
import sys
from pathlib import Path

from .checker import check
from .lexer import HoloSyntaxError, tokenize
from .parser import parse
from .runtime import ConflictError, World
from .units import show


def duration(text):
    try:
        token = tokenize(text)[0]
    except HoloSyntaxError as error:
        raise argparse.ArgumentTypeError(str(error)) from None
    if token.type != "Duration" or token.value <= 0:
        raise argparse.ArgumentTypeError("durée attendue, par exemple 1s ou 500ms")
    return token.value


def main(argv=None):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    parser = argparse.ArgumentParser(prog="holocode", description=__doc__)
    parser.add_argument("fichier", type=Path)
    parser.add_argument("--check", action="store_true", help="vérifier sans exécuter")
    parser.add_argument("--ticks", type=int, default=5, help="nombre de pas (défaut : 5)")
    parser.add_argument("--dt", type=duration, default=1.0, help="durée d'un pas (défaut : 1s)")
    args = parser.parse_args(argv)

    try:
        program = parse(args.fichier.read_text(encoding="utf-8"))
    except HoloSyntaxError as error:
        print(f"erreur de syntaxe, {error}")
        return 1
    diagnostics = check(program)
    for diagnostic in diagnostics:
        print(diagnostic)
    if any(d.severity == "erreur" for d in diagnostics):
        return 1
    if args.check:
        print("Programme bien formé.")
        return 0

    world = World(program)
    try:
        for _ in range(args.ticks):
            for event in world.tick(args.dt):
                print(event.format())
    except ConflictError as error:
        print(f"arrêt : {error}")
        return 2
    print(f"\nÉtat du monde {world.name} à t={world.time:g} s")
    for entity in world.entities.values():
        state = ", ".join(f"{field} = {show(entity.types[field], value)}" for field, value in entity.state.items())
        print(f"  {entity.space}/{entity.name} : {state or 'sans état'}")
    return 0
