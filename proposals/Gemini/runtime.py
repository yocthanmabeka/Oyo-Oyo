"""Moteur d'exécution spatial fractal, zoom continu et journal causal."""

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
    """Caméra spatiale continue pour la navigation par zoom."""

    def __init__(self, position: Vector3 = Vector3(0.0, 0.0, 5.0)):
        self.position = position

    def advance_towards(self, target: Vector3, step_ratio: float) -> None:
        direction = target - self.position
        self.position = self.position + (direction * step_ratio)

    def retreat_from(self, target: Vector3, step_ratio: float) -> None:
        direction = self.position - target
        self.position = self.position + (direction * step_ratio)


class FractalRuntime:
    """Coordinateur d'exécution du monde fractal avec paging hiérarchique."""

    def __init__(self, root_node: FractalNode, memory_ledger: MemoryLedger):
        self.root_node = root_node
        self.ledger = memory_ledger
        self.camera = Camera(position=Vector3(0.0, 0.0, 5.0))
        self.active_context_node: FractalNode = root_node
        self.current_tick = 0
        self.causal_trace: list[CausalEvent] = []

        # Paging initial : enregistrement de la racine et de ses enfants directs visibles uniquement
        self.ledger.register_node(self.root_node)
        for child in self.root_node.children.values():
            self.ledger.register_node(child)

    def set_node_state(self, node_id: str, key: str, value: Any) -> None:
        """Modifie une variable d'état et réajuste immédiatement le registre mémoire."""
        node = self.find_node(node_id)
        if not node:
            raise KeyError(f"Nœud inconnu : {node_id}")
        node.state[key] = value
        self.ledger.update_node_cost(node)

    def emit_signal(self, signal: Signal) -> list[CausalEvent]:
        """Traite un signal et applique les actions réactives associées."""
        events: list[CausalEvent] = []
        target_node = self.find_node(signal.source_node_id)
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
        target = self.find_node(action.target_node_id)
        if not target:
            raise RuntimeError(f"Nœud cible introuvable : {action.target_node_id}")

        if action.kind == ActionKind.SET_STATE:
            self.set_node_state(target.node_id, action.property_name, action.target_value)
            desc = f"state['{action.property_name}'] = {action.target_value}"
        elif action.kind == ActionKind.MORPH_APPEARANCE:
            self.set_node_state(target.node_id, "appearance", action.target_value)
            desc = f"morph appearance -> {action.target_value}"
        elif action.kind == ActionKind.PULSE_SIGNAL:
            self.set_node_state(target.node_id, "pulse_active", True)
            self.set_node_state(target.node_id, "last_pulse_tick", self.current_tick)
            desc = f"pulse actif émis sur {target.name}"
        else:
            desc = f"action {action.kind.name} sur {target.name}"

        return CausalEvent(
            tick=self.current_tick,
            subject_node_id=target.node_id,
            action_description=desc,
            reason=reason,
        )

    def step_zoom_towards(self, target_node_id: str, ratio: float = 0.2) -> list[CausalEvent]:
        """Avance la caméra d'un pas vers un nœud et déclenche l'entrée si le seuil est franchi."""
        self.current_tick += 1
        events: list[CausalEvent] = []
        target = self._find_child_of_active(target_node_id)
        if not target:
            return events

        self.camera.advance_towards(target.transform.position, ratio)
        distance = self.camera.position.distance_to(target.transform.position)

        if distance <= target.scale_threshold:
            events.extend(self._enter_node_interior(target, distance))

        return events

    def zoom_into(self, target_node_id: str, max_steps: int = 50, ratio: float = 0.5) -> list[CausalEvent]:
        """Effectue une approche continue par zoom jusqu'à entrer dans le sous-monde."""
        events: list[CausalEvent] = []
        for _ in range(max_steps):
            step_events = self.step_zoom_towards(target_node_id, ratio=ratio)
            events.extend(step_events)
            if self.active_context_node.node_id == target_node_id:
                break
        return events

    def zoom_out(self, steps: int = 1, ratio: float = 0.5) -> list[CausalEvent]:
        """Recule la caméra, sort du monde intérieur actif et restaure le monde parent."""
        events: list[CausalEvent] = []
        if self.active_context_node.parent is None:
            return events  # Déjà au niveau racine

        for _ in range(steps):
            self.current_tick += 1
            exited_node = self.active_context_node
            parent_node = exited_node.parent
            if parent_node is None:
                break

            # 1. Décharger les enfants du monde que l'on quitte pour libérer la mémoire
            for child in exited_node.children.values():
                self.ledger.unregister_node(child.node_id)

            # 2. Dégeler le parent et ses enfants (frères du nœud quitté)
            self.ledger.unfreeze_node(parent_node)
            for sibling in parent_node.children.values():
                self.ledger.unfreeze_node(sibling)

            # 3. Basculer le contexte actif vers le parent
            self.active_context_node = parent_node
            self.camera.position = Vector3(0.0, 0.0, 5.0)

            # 4. Émettre le signal de sortie et consigner l'événement
            event = CausalEvent(
                tick=self.current_tick,
                subject_node_id=exited_node.node_id,
                action_description=f"Sortie du monde intérieur de '{exited_node.name}'",
                reason=f"Recul de caméra vers le monde parent '{parent_node.name}' ; enfants déchargés.",
            )
            events.append(event)
            self.causal_trace.append(event)

            self.emit_signal(Signal(SignalKind.SCALE_EXIT, exited_node.node_id, {}))

        return events

    def _enter_node_interior(self, entered_node: FractalNode, distance: float) -> list[CausalEvent]:
        """Sas spatial d'entrée : congèle le parent et ses frères, charge les enfants."""
        events: list[CausalEvent] = []
        parent = self.active_context_node

        # 1. Congeler le parent en imposteur
        reclaimed_parent = self.ledger.freeze_node(parent)

        # 2. Congeler tous les frères du nœud pénétré
        reclaimed_siblings = 0
        for sibling in parent.children.values():
            if sibling.node_id != entered_node.node_id:
                reclaimed_siblings += self.ledger.freeze_node(sibling)

        # 3. Charger à la demande les enfants du nœud pénétré dans le registre
        for child in entered_node.children.values():
            if not self.ledger.is_registered(child.node_id):
                self.ledger.register_node(child)

        # 4. Basculer le contexte actif
        self.active_context_node = entered_node
        self.camera.position = Vector3(0.0, 0.0, 2.0)

        total_reclaimed = reclaimed_parent + reclaimed_siblings
        event = CausalEvent(
            tick=self.current_tick,
            subject_node_id=entered_node.node_id,
            action_description=f"Entrée dans le monde intérieur de '{entered_node.name}'",
            reason=(
                f"Distance ({distance:.4f}m) <= seuil ({entered_node.scale_threshold}m) ; "
                f"Parent et frères congelés en imposteurs (gain : {total_reclaimed} octets)."
            ),
        )
        events.append(event)
        self.causal_trace.append(event)

        # 5. Émettre le signal SCALE_ENTER
        self.emit_signal(Signal(SignalKind.SCALE_ENTER, entered_node.node_id, {}))

        return events

    def _find_child_of_active(self, target_node_id: str) -> FractalNode | None:
        return self.active_context_node.children.get(target_node_id)

    def find_node(self, node_id: str) -> FractalNode | None:
        """Recherche itérative sans récursion d'un nœud dans tout l'arbre."""
        stack = [self.root_node]
        while stack:
            current = stack.pop()
            if current.node_id == node_id:
                return current
            stack.extend(current.children.values())
        return None
