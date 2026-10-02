"""Suite de tests unitaires pour HoloFractal (Python 3.11+).

Validation formelle :
- Isomorphisme du Big Bang (Genèse d'un point vers des sous-mondes)
- Polymorphisme Nœud = Avatar = Lettre = Monde
- Stabilité mémoire stricte sous le plafond d'1 Go lors d'une traversée de 5 niveaux
- Contrôle des quotas (global et par nœud)
- Résistance aux univers profonds (2 000 niveaux sans RecursionError)
- Cohérence du registre lors de mutations d'état
- Navigation bidirectionnelle (descente et remontée avec déchargement)
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
    scale_threshold: float = 0.1,
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
        scale_threshold=scale_threshold,
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
        """Toute tentative d'instanciation dépassant le quota global ou par nœud est rejetée."""
        # 1. Quota global
        tight_ledger = MemoryLedger(global_limit_bytes=500 * 1024)
        root = make_test_node("root", "Root", NodeType.SEED_POINT, triangles=10, texture_kb=10)
        tight_ledger.register_node(root)

        huge_node = make_test_node("huge", "Monstre3D", NodeType.COMPOUND_OBJECT, triangles=100000, texture_kb=2048)
        with self.assertRaises(MemoryQuotaExceededError):
            tight_ledger.register_node(huge_node)

        # 2. Quota individuel par nœud
        permissive_ledger = MemoryLedger(global_limit_bytes=1_000_000_000)
        envelope_over_budget = VisualEnvelope(
            geometry_type=NodeType.SPHERE,
            color_rgb=(1.0, 1.0, 1.0),
            roughness=0.5,
            estimated_triangles=60000,  # Dépasse max_triangles (50 000)
            texture_bytes=1024,
        )
        strict_budget = NodeBudget(max_memory_bytes=10 * 1024 * 1024, max_triangles=50000)
        node_bad_triangles = FractalNode("bad", "Bad", NodeType.SPHERE, envelope_over_budget, strict_budget)

        with self.assertRaises(MemoryQuotaExceededError):
            permissive_ledger.register_node(node_bad_triangles)

    def test_russian_doll_nested_traversal_o1_memory(self):
        """Test de la Poupée Russe : 5 niveaux imbriqués traversés sans fuite mémoire."""
        ledger = MemoryLedger(global_limit_bytes=100 * 1024 * 1024)

        nodes = [
            make_test_node(f"node_{i}", f"Niveau_{i}", NodeType.WORLD_PORTAL, triangles=2000, texture_kb=512)
            for i in range(5)
        ]

        for i in range(4):
            nodes[i].add_child(nodes[i + 1])
            nodes[i + 1].transform.position = Vector3(0.0, 0.0, 0.0)

        runtime = FractalRuntime(nodes[0], ledger)
        mem_initial = ledger.current_allocated_bytes

        for i in range(4):
            events = runtime.zoom_into(f"node_{i + 1}", max_steps=20, ratio=0.5)
            self.assertTrue(any("Entrée dans le monde intérieur" in e.action_description for e in events))
            self.assertTrue(nodes[i].is_frozen_as_impostor)

        mem_at_deepest = ledger.current_allocated_bytes

        # Vérification : au niveau le plus profond, la mémoire active reste bornée
        self.assertLess(mem_at_deepest, mem_initial)
        self.assertLess(mem_at_deepest, 1_000_000_000)

    def test_paging_memory_bounded_during_descent_and_ascent(self):
        """Vérifie que la mémoire reste bornée à la descente et se rétablit fidèlement à la remontée."""
        ledger = MemoryLedger(global_limit_bytes=100 * 1024 * 1024)

        nodes = [
            make_test_node(f"n_{i}", f"N_{i}", NodeType.WORLD_PORTAL, triangles=1500, texture_kb=256)
            for i in range(4)
        ]

        for i in range(3):
            nodes[i].add_child(nodes[i + 1])
            nodes[i + 1].transform.position = Vector3(0.0, 0.0, 0.0)

        runtime = FractalRuntime(nodes[0], ledger)
        initial_cost = ledger.current_allocated_bytes

        # Descente vers le niveau 3
        for i in range(3):
            runtime.zoom_into(f"n_{i + 1}", max_steps=20, ratio=0.5)

        self.assertEqual(runtime.active_context_node.node_id, "n_3")

        # Remontée complète vers la racine
        events_out = runtime.zoom_out(steps=3)
        self.assertEqual(len(events_out), 3)
        self.assertEqual(runtime.active_context_node.node_id, "n_0")

        # Vérification du solde : la mémoire allouée revient exactement à son état initial
        self.assertEqual(ledger.current_allocated_bytes, initial_cost)

    def test_deep_universe_no_recursion_error(self):
        """Vérifie qu'un univers de 2 000 niveaux linéaires ne provoque pas de RecursionError."""
        ledger = MemoryLedger(global_limit_bytes=1_000_000_000)

        root = make_test_node("n_0", "N_0", NodeType.WORLD_PORTAL, triangles=10, texture_kb=10)
        current = root
        for i in range(1, 2000):
            child = make_test_node(f"n_{i}", f"N_{i}", NodeType.WORLD_PORTAL, triangles=10, texture_kb=10)
            current.add_child(child)
            current = child

        # Ne doit pas lever RecursionError grâce au chargement paresseux à la racine
        runtime = FractalRuntime(root, ledger)
        self.assertIsNotNone(runtime)
        self.assertEqual(runtime.active_context_node.node_id, "n_0")

    def test_state_mutation_updates_memory_ledger(self):
        """Vérifie que modifier l'état réajuste le registre et ne génère jamais de montant négatif."""
        ledger = MemoryLedger()
        node = make_test_node("dyn_node", "DynNode", NodeType.SPHERE, triangles=100, texture_kb=10)
        runtime = FractalRuntime(node, ledger)

        base_cost = ledger.current_allocated_bytes

        # Ajout de 2 000 variables d'état
        for i in range(2000):
            runtime.set_node_state(node.node_id, f"prop_{i}", i)

        cost_with_state = ledger.current_allocated_bytes
        self.assertEqual(cost_with_state, base_cost + (2000 * 128))

        # Congélation en imposteur : le coût doit devenir strictement égal au coût réservé d'imposteur
        ledger.freeze_node(node)
        self.assertEqual(ledger.current_allocated_bytes, node.budget.reserved_impostor_bytes)
        self.assertGreater(ledger.current_allocated_bytes, 0)

        # Dégel : retour au montant avec état
        ledger.unfreeze_node(node)
        self.assertEqual(ledger.current_allocated_bytes, cost_with_state)

    def test_pulse_signal_propagation(self):
        """Vérifie que l'action PULSE_SIGNAL a un effet causal effectif sur le nœud cible."""
        ledger = MemoryLedger()
        node = make_test_node("pulse_node", "PulseNode", NodeType.SPHERE)
        node.reactions.append(
            Reaction(
                reaction_id="pulse_react",
                expected_signal=SignalKind.TOUCH,
                action=Action(
                    kind=ActionKind.PULSE_SIGNAL,
                    target_node_id="pulse_node",
                    property_name="",
                    target_value=None,
                ),
            )
        )
        runtime = FractalRuntime(node, ledger)
        events = runtime.emit_signal(Signal(SignalKind.TOUCH, "pulse_node", {}))

        self.assertEqual(len(events), 1)
        self.assertTrue(node.state.get("pulse_active"))
        self.assertIn("pulse actif émis", events[0].action_description)


if __name__ == "__main__":
    unittest.main()
