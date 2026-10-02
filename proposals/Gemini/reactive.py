"""Système causal réactif pur : zéro code Turing-complet, création sûre."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum, auto
from typing import Any

try:
    from core import FractalNode
except ImportError:
    from .core import FractalNode


class SignalKind(Enum):
    TOUCH = auto()         # Pression tactile de l'utilisateur
    SCALE_ENTER = auto()   # La caméra franchit le seuil vers l'intérieur
    SCALE_EXIT = auto()    # La caméra recule et sort du monde intérieur
    STATE_CHANGE = auto()  # Une variable d'état a muté


@dataclass(frozen=True)
class Signal:
    """Événement déclencheur émis par l'environnement ou l'utilisateur."""
    kind: SignalKind
    source_node_id: str
    payload: dict[str, Any]


class ActionKind(Enum):
    SET_STATE = auto()         # Mutation d'une variable déclarative
    MORPH_APPEARANCE = auto()  # Changement de couleur ou de maillage
    PULSE_SIGNAL = auto()      # Émission d'une impulsion locale réactive


@dataclass(frozen=True)
class Action:
    """Effet causal appliqué de manière déterministe."""
    kind: ActionKind
    target_node_id: str
    property_name: str
    target_value: Any


@dataclass
class Reaction:
    """Liaison réactive sans code : Quand [Signal] -> Alors [Action]."""
    reaction_id: str
    expected_signal: SignalKind
    action: Action

    def evaluate(self, signal: Signal, node: FractalNode) -> Action | None:
        if signal.kind == self.expected_signal and signal.source_node_id == node.node_id:
            return self.action
        return None
