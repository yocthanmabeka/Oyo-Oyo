"""Moteur d'exécution spatial fractal, zoom logarithmique et journal causal."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

try:
    from core import FractalNode, Vector3
    from memory import MemoryLedger
    from reactive import Action, ActionKind, Reaction, Signal, SignalKind
except ImportError:
    from .core import FractalNode, Vector3
    from .memory import MemoryLedger
    from .reactive import Action, ActionKind, Reaction, Signal, SignalKind


@dataclass(frozen=True)
class CausalEvent:
    """Trace explicative d'une transition ou d'une action causale."""
    tick: int
    subject_node_id: str
    action_description: str
    reason: str


class Camera:
    """Caméra spatiale continue avec profondeur logarithmique."""

    def __init__(self, position: Vector3 = Vector3(0.0, 0.0, 5.0)):
        self.position = position
        self.zoom_level = 1.0

    def advance_towards(self, target: Vector3, step_ratio: float) -> None:
        direction = target - self.position
        self.position = self.position + (direction * step_ratio)


class FractalRuntime:
    """Coordinateur d'exécution du monde fractal."""

    def __init__(self, root_node: FractalNode, memory_ledger: MemoryLedger):
        self.root_node = root_node
        self.ledger = memory_ledger
        self.camera = Camera()
        self.active_context_node: FractalNode = root_node
        self.current_tick = 0
        self.causal_trace: list[CausalEvent] = []

        self._register_subtree(self.root_node)

    def _register_subtree(self, node: FractalNode) -> None:
        self.ledger.register_node(node)
        for child in node.children.values():
            self._register_subtree(child)

    def emit_signal(self, signal: Signal) -> list[CausalEvent]:
        """Traite un signal et applique les actions réactives associées."""
        events: list[CausalEvent] = []
        target_node = self._find_node(self.root_node, signal.source_node_id)
        if not target_node:
            return events

        for reaction in target_node.reactions:
            action = reaction.evaluate(signal, target_node)
            if action:
                event = self._apply_action(action, f"Déclenché par signal {signal.kind.name}")
                events.append(event)
                self.causal_trace.append(event)
        return events

    def _apply_action(self, action: Action, reason: str) -> CausalEvent:
        target = self._find_node(self.root_node, action.target_node_id)
        if not target:
            raise RuntimeError(f"Nœud cible introuvable: {action.target_node_id}")

        if action.kind == ActionKind.SET_STATE:
            target.state[action.property_name] = action.target_value
            desc = f"state['{action.property_name}'] = {action.target_value}"
        elif action.kind == ActionKind.MORPH_APPEARANCE:
            target.state["appearance"] = action.target_value
            desc = f"morph appearance -> {action.target_value}"
        else:
            desc = f"action {action.kind.name} sur {target.name}"

        return CausalEvent(
            tick=self.current_tick,
            subject_node_id=target.node_id,
            action_description=desc,
            reason=reason,
        )

    def step_zoom_towards(self, target_node_id: str, ratio: float = 0.2) -> list[CausalEvent]:
        """Simule l'avancée continue de la caméra vers un nœud jusqu'à franchir le seuil."""
        self.current_tick += 1
        events: list[CausalEvent] = []
        target = self._find_node(self.active_context_node, target_node_id)
        if not target:
            return events

        self.camera.advance_towards(target.transform.position, ratio)
        distance = self.camera.position.distance_to(target.transform.position)

        if distance <= target.scale_threshold and target.children:
            events.extend(self._enter_node_interior(target, distance))

        return events

    def _enter_node_interior(self, entered_node: FractalNode, distance: float) -> list[CausalEvent]:
        """Sas spatial : congèle le parent et active le monde intérieur."""
        events: list[CausalEvent] = []
        parent = self.active_context_node

        reclaimed = self.ledger.freeze_node(parent)

        self.active_context_node = entered_node
        self.camera.position = Vector3(0.0, 0.0, 2.0)

        event = CausalEvent(
            tick=self.current_tick,
            subject_node_id=entered_node.node_id,
            action_description=f"Entrée dans le monde intérieur de '{entered_node.name}'",
            reason=(
                f"Distance ({distance:.4f}m) <= seuil ({entered_node.scale_threshold}m) ; "
                f"Parent '{parent.name}' congelé en imposteur (gain: {reclaimed} octets)."
            ),
        )
        events.append(event)
        self.causal_trace.append(event)
        return events

    def _find_node(self, current: FractalNode, node_id: str) -> FractalNode | None:
        if current.node_id == node_id:
            return current
        for child in current.children.values():
            found = self._find_node(child, node_id)
            if found:
                return found
        return None
