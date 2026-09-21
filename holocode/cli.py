"""Command line interface for executing HoloCode source files."""

import argparse
import json
from pathlib import Path

from .parser import parse
from .runtime import Runtime


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="holocode")
    parser.add_argument("source", type=Path, help="fichier .holo a executer")
    parser.add_argument("--ticks", type=int, default=1, help="nombre de ticks")
    return parser


def main() -> None:
    arguments = build_parser().parse_args()
    world = parse(arguments.source.read_text(encoding="utf-8"))
    runtime = Runtime(world)
    for _ in range(arguments.ticks):
        events = runtime.tick()
        for event in events:
            print(
                f"tick={event.tick} phenomenon={event.phenomenon} "
                f"target={event.entity}.{event.property_name} "
                f"{event.previous!r}->{event.current!r}"
            )
    print(json.dumps(runtime.snapshot(), ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()

