"""Deterministic runtime for the HoloCode v0.1 world graph."""

from copy import deepcopy
from dataclasses import dataclass
from math import dist
from typing import Any

from .ast import Condition, PhenomenonDecl, RelationDecl, Vector3, WorldDecl


class RuntimeError(ValueError):
    pass


@dataclass(frozen=True)
class Event:
    tick: int
    phenomenon: str
    entity: str
    property_name: str
    previous: Any
    current: Any


class Runtime:
    """Executes relations first, then applies phenomenon effects atomically per tick."""

    def __init__(self, world: WorldDecl):
        self.world = deepcopy(world)
        self.tick_number = 0
        self.relation_state: dict[str, bool] = {}
        self.events: list[Event] = []
        self._validate()

    def _validate(self) -> None:
        for relation in self.world.relations.values():
            for name in (relation.left, relation.right):
                if name not in self.world.entities:
                    raise RuntimeError(
                        f"La relation {relation.name} reference l'entite inconnue {name}"
                    )
                position = self.world.entities[name].properties.get("position")
                if not isinstance(position, Vector3):
                    raise RuntimeError(
                        f"L'entite {name} doit posseder une position 3D typee"
                    )
        for phenomenon in self.world.phenomena:
            self._validate_phenomenon(phenomenon)

    def _validate_phenomenon(self, phenomenon: PhenomenonDecl) -> None:
        for condition in phenomenon.conditions:
            if condition.kind == "relation" and condition.subject not in self.world.relations:
                raise RuntimeError(f"Relation inconnue: {condition.subject}")
            if condition.kind == "property" and condition.subject not in self.world.entities:
                raise RuntimeError(f"Entite inconnue: {condition.subject}")
        for effect in phenomenon.effects:
            if effect.entity not in self.world.entities:
                raise RuntimeError(f"Entite inconnue: {effect.entity}")

    def entity(self, name: str) -> dict[str, Any]:
        try:
            return self.world.entities[name].properties
        except KeyError as error:
            raise RuntimeError(f"Entite inconnue: {name}") from error

    def move(self, name: str, position: Vector3) -> None:
        self.entity(name)["position"] = position

    def _relation_active(self, relation: RelationDecl) -> bool:
        left = self.entity(relation.left)["position"]
        right = self.entity(relation.right)["position"]
        if left.unit != right.unit or left.unit != relation.unit:
            raise RuntimeError(f"Unites incompatibles dans {relation.name}")
        distance = dist((left.x, left.y, left.z), (right.x, right.y, right.z))
        return distance < relation.max_distance

    def _condition_true(self, condition: Condition) -> bool:
        if condition.kind == "relation":
            return self.relation_state[condition.subject]
        if condition.kind == "property":
            return self.entity(condition.subject).get(condition.property_name) == condition.expected
        raise RuntimeError(f"Type de condition inconnu: {condition.kind}")

    def tick(self) -> list[Event]:
        """Evaluate a stable snapshot and commit all resulting effects."""
        self.tick_number += 1
        self.relation_state = {
            name: self._relation_active(relation)
            for name, relation in self.world.relations.items()
        }
        scheduled: list[tuple[str, str, str, Any]] = []
        targets: dict[tuple[str, str], Any] = {}
        for phenomenon in self.world.phenomena:
            if all(self._condition_true(condition) for condition in phenomenon.conditions):
                for effect in phenomenon.effects:
                    target = (effect.entity, effect.property_name)
                    if target in targets and targets[target] != effect.value:
                        raise RuntimeError(
                            f"Conflit au tick {self.tick_number} sur "
                            f"{effect.entity}.{effect.property_name}"
                        )
                    targets[target] = effect.value
                    scheduled.append(
                        (phenomenon.name, effect.entity, effect.property_name, effect.value)
                    )
        emitted: list[Event] = []
        for phenomenon, entity, property_name, value in scheduled:
            state = self.entity(entity)
            previous = state.get(property_name)
            if previous != value:
                state[property_name] = value
                event = Event(
                    self.tick_number, phenomenon, entity, property_name, previous, value
                )
                emitted.append(event)
                self.events.append(event)
        return emitted

    def snapshot(self) -> dict[str, Any]:
        def serialise(value: Any) -> Any:
            if isinstance(value, Vector3):
                return {"x": value.x, "y": value.y, "z": value.z, "unit": value.unit}
            return value

        return {
            "world": self.world.name,
            "tick": self.tick_number,
            "relations": dict(self.relation_state),
            "entities": {
                name: {key: serialise(value) for key, value in entity.properties.items()}
                for name, entity in self.world.entities.items()
            },
        }

