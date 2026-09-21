"""Suite de tests unitaires pour HoloFractal (Python 3.11+).

Validation :

* Isomorphisme du Big Bang (Genèse d'un point vers des sous-mondes)
* Polymorphisme Nœud = Avatar = Lettre = Monde
* Stabilité mémoire stricte O(1) sous le plafond d'1 Go lors d'une traversée de 5 niveaux
* Contrôle budgétaire et rejet de dépassement de quota
* Réactivité causale sans code
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

try:
    from core import (
        FractalNode,
        NodeBudget,
        NodeType,
        Transform,
        Vector3,
        VisualEnvelope,
    )
    from memory import MemoryLedger, MemoryQuotaExceededError
    from reactive import (
        Action,
        ActionKind,
        Reaction,
        Signal,
        SignalKind,
    )
    from runtime import FractalRuntime
except ImportError:
    from holofractal.core import (
        FractalNode,
        NodeBudget,
        NodeType,
        Transform,
        Vector3,
        VisualEnvelope,
    )
    from holofractal.memory import MemoryLedger, MemoryQuotaExceededError
    from holofractal.reactive import (
        Action,
        ActionKind,
        Reaction,
        Signal,
        SignalKind,
    )
    from holofractal.runtime import FractalRuntime


def make_test_node(
    node_id: str,
    name: str,
    node_type: NodeType,
    triangles: int = 1000,
    texture_kb: int = 128,
) -> FractalNode:
    envelope = VisualEnvelope(
        geometry_type=node_type,
        color_rgb=(1.0, 1.0, 1.0),
        roughness=0.5,
        estimated_triangles=triangles,
        texture_bytes=texture_kb * 1024,
    )
    budget = NodeBudget(
        max_memory_bytes=10 * 1024 * 1024,  # 10 Mo
        max_triangles=50000,
        reserved_impostor_bytes=64 * 1024,   # 64 Ko
    )
    return FractalNode(
        node_id=node_id,
        name=name,
        node_type=node_type,
        envelope=envelope,
        budget=budget,
        scale_threshold=0.1,  # 10 cm
    )


class TestHoloFractalCore(unittest.TestCase):

    def test_big_bang_genesis(self):
        """Le point primordial se scinde en sous-mondes."""
        ledger = MemoryLedger(global_limit_bytes=500 * 1024 * 1024)
        big_bang_point = make_test_node("root", "BigBangPoint", NodeType.SEED_POINT, triangles=100, texture_kb=16)

        world_a = make_test_node("world_a", "UniversA", NodeType.WORLD_PORTAL)
        world_b = make_test_node("world_b", "UniversB", NodeType.WORLD_PORTAL)

        big_bang_point.add_child(world_a)
        big_bang_point.add_child(world_b)

        runtime = FractalRuntime(big_bang_point, ledger)
        self.assertEqual(len(runtime.root_node.children), 2)
        self.assertGreater(ledger.current_allocated_bytes, 0)
        self.assertLess(ledger.current_allocated_bytes, ledger.global_limit_bytes)

    def test_polymorphism_avatar_and_letter(self):
        """Un nœud peut représenter une lettre interactive ou un avatar modulable."""
        letter_a = make_test_node("letter_a", "Lettre_A", NodeType.GLYPH)
        letter_a.state["opened"] = False

        letter_a.reactions.append(
            Reaction(
                reaction_id="r1",
                expected_signal=SignalKind.TOUCH,
                action=Action(
                    kind=ActionKind.SET_STATE,
                    target_node_id="letter_a",
                    property_name="opened",
                    target_value=True,
                ),
            )
        )

        ledger = MemoryLedger()
        runtime = FractalRuntime(letter_a, ledger)

        events = runtime.emit_signal(Signal(SignalKind.TOUCH, "letter_a", {}))
        self.assertEqual(len(events), 1)
        self.assertTrue(letter_a.state["opened"])
        self.assertIn("TOUCH", events[0].reason)

    def test_memory_quota_enforcement(self):
        """Toute tentative d'instanciation dépassant le quota strict de l'arène est rejetée."""
        tight_ledger = MemoryLedger(global_limit_bytes=500 * 1024)

        root = make_test_node("root", "Root", NodeType.SEED_POINT, triangles=10, texture_kb=10)
        tight_ledger.register_node(root)

        huge_node = make_test_node("huge", "Monstre3D", NodeType.COMPOUND_OBJECT, triangles=100000, texture_kb=2048)

        with self.assertRaises(MemoryQuotaExceededError):
            tight_ledger.register_node(huge_node)

    def test_russian_doll_nested_traversal_o1_memory(self):
        """Test de la Poupée Russe : 5 niveaux imbriqués traversés sans fuite mémoire.

        Prouve que la mémoire active reste bornée grâce à la congélation en imposteurs.
        """
        ledger = MemoryLedger(global_limit_bytes=100 * 1024 * 1024)

        nodes = [
            make_test_node(f"node_{i}", f"Niveau_{i}", NodeType.WORLD_PORTAL, triangles=2000, texture_kb=512)
            for i in range(5)
        ]

        for i in range(4):
            nodes[i].add_child(nodes[i + 1])
            nodes[i + 1].transform.position = Vector3(0.0, 0.0, 0.05)

        runtime = FractalRuntime(nodes[0], ledger)
        mem_initial = ledger.current_allocated_bytes

        for i in range(4):
            events = runtime.step_zoom_towards(f"node_{i + 1}", ratio=0.9)
            self.assertTrue(any("Entrée dans le monde intérieur" in e.action_description for e in events))
            self.assertTrue(nodes[i].is_frozen_as_impostor)

        mem_at_deepest = ledger.current_allocated_bytes

        self.assertLess(mem_at_deepest, mem_initial)
        self.assertLess(mem_at_deepest, 1_000_000_000)


if __name__ == "__main__":
    unittest.main()
