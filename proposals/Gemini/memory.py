"""Gestionnaire d'arène et de plafond mémoire étanche (Garantie < 1 Go)."""

from __future__ import annotations

try:
    from core import FractalNode
except ImportError:
    from .core import FractalNode


class MemoryQuotaExceededError(RuntimeError):
    """Levée dès qu'une allocation d'entité ou d'échelle viole le budget alloué."""
    pass


class MemoryLedger:
    """Registre strict des allocations mémoire actives dans l'arène du smartphone."""

    def __init__(self, global_limit_bytes: int = 1_000_000_000):  # 1 Go par défaut
        self.global_limit_bytes = global_limit_bytes
        self._current_allocated_bytes = 0
        self._tracked_nodes: dict[str, FractalNode] = {}
        self._node_allocations: dict[str, int] = {}

    @property
    def current_allocated_bytes(self) -> int:
        return self._current_allocated_bytes

    @property
    def remaining_bytes(self) -> int:
        return self.global_limit_bytes - self._current_allocated_bytes

    def register_node(self, node: FractalNode) -> None:
        """Enregistre un nœud et réserve son coût initial après vérification des quotas."""
        # 1. Vérification des quotas propres au nœud
        if node.envelope.estimated_triangles > node.budget.max_triangles:
            raise MemoryQuotaExceededError(
                f"Le nœud '{node.name}' dépasse son quota de triangles : "
                f"{node.envelope.estimated_triangles} > {node.budget.max_triangles}"
            )
        cost = node.compute_local_active_bytes()
        if cost > node.budget.max_memory_bytes:
            raise MemoryQuotaExceededError(
                f"Le nœud '{node.name}' dépasse son budget mémoire alloué : "
                f"{cost} > {node.budget.max_memory_bytes} octets"
            )

        # 2. Vérification du quota global
        if self._current_allocated_bytes + cost > self.global_limit_bytes:
            raise MemoryQuotaExceededError(
                f"Dépassement du quota mémoire global ({self.global_limit_bytes} octets) : "
                f"l'ajout du nœud '{node.name}' requiert {cost} octets, "
                f"disponible = {self.remaining_bytes} octets."
            )

        self._tracked_nodes[node.node_id] = node
        self._node_allocations[node.node_id] = cost
        self._current_allocated_bytes += cost

    def unregister_node(self, node_id: str) -> None:
        """Décharge un nœud et restitue l'intégralité exacte de sa mémoire allouée."""
        if node_id in self._node_allocations:
            allocated = self._node_allocations.pop(node_id)
            self._current_allocated_bytes = max(0, self._current_allocated_bytes - allocated)
            self._tracked_nodes.pop(node_id, None)

    def is_registered(self, node_id: str) -> bool:
        return node_id in self._tracked_nodes

    def update_node_cost(self, node: FractalNode) -> None:
        """Réajuste le compteur suite à une modification d'état ou d'attributs."""
        if node.node_id not in self._node_allocations:
            return

        old_cost = self._node_allocations[node.node_id]
        new_cost = node.compute_local_active_bytes()
        diff = new_cost - old_cost

        if diff > 0 and self._current_allocated_bytes + diff > self.global_limit_bytes:
            raise MemoryQuotaExceededError(
                f"Dépassement du quota mémoire global lors de la mise à jour de '{node.name}'."
            )

        self._node_allocations[node.node_id] = new_cost
        self._current_allocated_bytes += diff

    def freeze_node(self, node: FractalNode) -> int:
        """Congèle un nœud en imposteur proxy, réduisant son coût au montant réservé."""
        if node.is_frozen_as_impostor or node.node_id not in self._node_allocations:
            return 0

        old_cost = self._node_allocations[node.node_id]
        node.is_frozen_as_impostor = True
        new_cost = node.compute_local_active_bytes()
        reclaimed = old_cost - new_cost

        self._node_allocations[node.node_id] = new_cost
        self._current_allocated_bytes -= reclaimed
        return reclaimed

    def unfreeze_node(self, node: FractalNode) -> int:
        """Restaure un nœud congelé à pleine résolution."""
        if not node.is_frozen_as_impostor or node.node_id not in self._node_allocations:
            return 0

        old_cost = self._node_allocations[node.node_id]
        node.is_frozen_as_impostor = False
        new_cost = node.compute_local_active_bytes()
        added = new_cost - old_cost

        if self._current_allocated_bytes + added > self.global_limit_bytes:
            node.is_frozen_as_impostor = True  # Rollback
            raise MemoryQuotaExceededError(
                f"Impossible de dégeler '{node.name}' : mémoire globale insuffisante."
            )

        self._node_allocations[node.node_id] = new_cost
        self._current_allocated_bytes += added
        return added
