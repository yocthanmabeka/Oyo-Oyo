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

    @property
    def current_allocated_bytes(self) -> int:
        return self._current_allocated_bytes

    @property
    def remaining_bytes(self) -> int:
        return self.global_limit_bytes - self._current_allocated_bytes

    def register_node(self, node: FractalNode) -> None:
        """Enregistre un nœud et réserve son coût initial."""
        cost = node.compute_local_active_bytes()
        if self._current_allocated_bytes + cost > self.global_limit_bytes:
            raise MemoryQuotaExceededError(
                f"Dépassement de quota mémoire global ({self.global_limit_bytes} octets) : "
                f"l'ajout du nœud '{node.name}' requiert {cost} octets, "
                f"disponible = {self.remaining_bytes} octets."
            )
        self._tracked_nodes[node.node_id] = node
        self._current_allocated_bytes += cost

    def unregister_node(self, node_id: str) -> None:
        if node_id in self._tracked_nodes:
            cost = self._tracked_nodes[node_id].compute_local_active_bytes()
            self._current_allocated_bytes = max(0, self._current_allocated_bytes - cost)
            del self._tracked_nodes[node_id]

    def freeze_node(self, node: FractalNode) -> int:
        """Congèle un nœud parent en imposteur proxy, libérant sa géométrie lourde."""
        if node.is_frozen_as_impostor:
            return 0
        before = node.compute_local_active_bytes()
        node.is_frozen_as_impostor = True
        after = node.compute_local_active_bytes()
        reclaimed = before - after
        self._current_allocated_bytes -= reclaimed
        return reclaimed

    def unfreeze_node(self, node: FractalNode) -> int:
        """Restaure un nœud en mémoire pleine résolution."""
        if not node.is_frozen_as_impostor:
            return 0
        before = node.compute_local_active_bytes()
        node.is_frozen_as_impostor = False
        after = node.compute_local_active_bytes()
        added = after - before
        if self._current_allocated_bytes + added > self.global_limit_bytes:
            node.is_frozen_as_impostor = True  # Rollback
            raise MemoryQuotaExceededError(
                f"Impossible de dégeler '{node.name}' : mémoire insuffisante."
            )
        self._current_allocated_bytes += added
        return added
