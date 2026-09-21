"""Arbre syntaxique de HoloCode v0.1."""

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class Pos:
    line: int
    col: int


@dataclass(frozen=True)
class Literal:
    type: str  # Bool, Number, Text, Length, Duration
    value: Any  # longueurs en mètres, durées en secondes
    pos: Pos


@dataclass(frozen=True)
class Field:
    name: str
    type: str
    default: Literal
    pos: Pos


@dataclass(frozen=True)
class Assignment:
    field: str
    value: Literal
    pos: Pos


@dataclass(frozen=True)
class Capability:
    """Seul moyen de modifier l'état d'une entité."""

    name: str
    assignments: tuple[Assignment, ...]
    pos: Pos


@dataclass(frozen=True)
class Archetype:
    name: str
    fields: tuple[Field, ...]
    capabilities: tuple[Capability, ...]
    pos: Pos


@dataclass(frozen=True)
class Binder:
    """`d: Openable + Lockable` : toute entité composée d'au moins ces archétypes."""

    var: str
    archetypes: tuple[str, ...]
    pos: Pos


@dataclass(frozen=True)
class EntityDecl:
    name: str
    archetypes: tuple[str, ...]
    position: tuple[Literal, Literal, Literal]
    overrides: tuple[Assignment, ...]
    space: str
    pos: Pos


@dataclass(frozen=True)
class Space:
    name: str
    entities: tuple[EntityDecl, ...]
    pos: Pos


@dataclass(frozen=True)
class Relation:
    """`relation Near(p: A, d: B) when distance(p, d) <= 2m`."""

    name: str
    left: Binder
    right: Binder
    distance_args: tuple[str, str]
    op: str
    threshold: Literal
    pos: Pos


@dataclass(frozen=True)
class FieldAtom:
    """`d.locked` ou `d.width > 2m`."""

    var: str
    field: str
    op: str | None
    value: Literal | None
    pos: Pos


@dataclass(frozen=True)
class RelationAtom:
    """`some Near(_, d)` ou `no Near(_, d)`."""

    quantifier: str  # some | no
    relation: str
    args: tuple[str, str]
    pos: Pos


@dataclass(frozen=True)
class Term:
    atom: FieldAtom | RelationAtom
    negated: bool
    duration: Literal | None  # `for 3s` : vrai sans interruption depuis 3 s
    pos: Pos


@dataclass(frozen=True)
class Law:
    """Règle permanente du monde : elle ne change jamais l'état, elle interdit."""

    name: str
    binder: Binder
    where: tuple[Term, ...]
    var: str
    capability: str
    pos: Pos


@dataclass(frozen=True)
class Phenomenon:
    """Transformation : quand la condition tient, demande une capacité."""

    name: str
    binder: Binder
    when: tuple[Term, ...]
    var: str
    capability: str
    pos: Pos


@dataclass(frozen=True)
class World:
    name: str
    spaces: tuple[Space, ...]
    relations: tuple[Relation, ...]
    laws: tuple[Law, ...]
    phenomena: tuple[Phenomenon, ...]
    pos: Pos


@dataclass(frozen=True)
class Program:
    archetypes: tuple[Archetype, ...]
    world: World
