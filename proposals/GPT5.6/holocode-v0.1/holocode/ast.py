"""Abstract syntax tree for the minimal HoloCode language."""

from dataclasses import dataclass, field
from typing import Any


@dataclass(frozen=True)
class Vector3:
    x: float
    y: float
    z: float
    unit: str = "m"


@dataclass
class EntityDecl:
    name: str
    properties: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class RelationDecl:
    name: str
    left: str
    right: str
    max_distance: float
    unit: str = "m"


@dataclass(frozen=True)
class Condition:
    kind: str
    subject: str
    property_name: str | None = None
    expected: Any = None


@dataclass(frozen=True)
class Effect:
    entity: str
    property_name: str
    value: Any


@dataclass
class PhenomenonDecl:
    name: str
    conditions: list[Condition] = field(default_factory=list)
    effects: list[Effect] = field(default_factory=list)


@dataclass
class WorldDecl:
    name: str
    entities: dict[str, EntityDecl] = field(default_factory=dict)
    relations: dict[str, RelationDecl] = field(default_factory=dict)
    phenomena: list[PhenomenonDecl] = field(default_factory=list)

