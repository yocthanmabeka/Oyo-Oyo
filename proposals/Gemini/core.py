"""Structures fondamentales de l'ontologie spatiale fractale."""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Any


@dataclass(frozen=True)
class Vector3:
    """Position ou vecteur dimensionné dans l'espace euclidien local (en mètres)."""
    x: float
    y: float
    z: float

    def distance_to(self, other: Vector3) -> float:
        return math.dist((self.x, self.y, self.z), (other.x, other.y, other.z))

    def magnitude(self) -> float:
        return math.sqrt(self.x * self.x + self.y * self.y + self.z * self.z)

    def __add__(self, other: Vector3) -> Vector3:
        return Vector3(self.x + other.x, self.y + other.y, self.z + other.z)

    def __sub__(self, other: Vector3) -> Vector3:
        return Vector3(self.x - other.x, self.y - other.y, self.z - other.z)

    def __mul__(self, factor: float) -> Vector3:
        return Vector3(self.x * factor, self.y * factor, self.z * factor)


@dataclass
class Transform:
    """Orientation, position et facteur d'échelle relatif au parent."""
    position: Vector3 = field(default_factory=lambda: Vector3(0.0, 0.0, 0.0))
    scale: float = 1.0


class NodeType(Enum):
    """Polymorphie de la primitive spatiale."""
    SEED_POINT = auto()      # Le point Big Bang originel
    SPHERE = auto()          # Sphère modulable / Avatar potentiel
    GLYPH = auto()           # Caractère textuel tridimensionnel (ex: lettre A)
    COMPOUND_OBJECT = auto() # Assemblage d'objets
    WORLD_PORTAL = auto()    # Nœud hébergeant un univers intérieur majeur


@dataclass(frozen=True)
class VisualEnvelope:
    """Enveloppe externe visible depuis le monde parent."""
    geometry_type: NodeType
    color_rgb: tuple[float, float, float]
    roughness: float
    estimated_triangles: int
    texture_bytes: int

    @property
    def estimated_geometry_bytes(self) -> int:
        # Encodage quantifié théorique : 16 octets par sommet, 3 sommets par triangle
        return self.estimated_triangles * 3 * 16

    @property
    def total_memory_cost(self) -> int:
        return self.estimated_geometry_bytes + self.texture_bytes


@dataclass(frozen=True)
class NodeBudget:
    """Contrat budgétaire étanche alloué au nœud (en octets)."""
    max_memory_bytes: int
    max_triangles: int
    reserved_impostor_bytes: int = 64 * 1024  # 64 Ko pour l'imposteur cubemap


NodeId = str


class FractalNode:
    """Entité unifiée : Nœud = Avatar = Objet = Monde intérieur."""

    def __init__(
        self,
        node_id: NodeId,
        name: str,
        node_type: NodeType,
        envelope: VisualEnvelope,
        budget: NodeBudget,
        scale_threshold: float = 0.1,  # Seuil métrique d'entrée par défaut (10 cm)
        parent: FractalNode | None = None,
    ):
        self.node_id = node_id
        self.name = name
        self.node_type = node_type
        self.transform = Transform()
        self.envelope = envelope
        self.budget = budget
        self.scale_threshold = scale_threshold
        self.parent = parent
        self.children: dict[NodeId, FractalNode] = {}
        self.state: dict[str, Any] = {}
        self.reactions: list[Any] = []
        self.is_frozen_as_impostor = False

    def add_child(self, child: FractalNode) -> None:
        child.parent = self
        self.children[child.node_id] = child

    def remove_child(self, node_id: NodeId) -> FractalNode:
        return self.children.pop(node_id)

    def compute_local_active_bytes(self) -> int:
        """Calcule le coût mémoire déclaré du nœud selon sa résolution actuelle."""
        if self.is_frozen_as_impostor:
            return self.budget.reserved_impostor_bytes
        state_overhead = len(self.state) * 128
        return self.envelope.total_memory_cost + state_overhead
