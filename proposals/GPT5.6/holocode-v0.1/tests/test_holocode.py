import unittest

from holocode.ast import Vector3
from holocode.parser import ParseError, parse
from holocode.runtime import Runtime, RuntimeError


PROGRAM = """
world Building {
    entity User { position: (0m, 0m, 0m) }
    entity Door {
        position: (0m, 0m, 1m)
        opened: false
        locked: false
    }
    relation Near(User, Door) { distance < 2m }
    phenomenon AutomaticOpening {
        when Near active and Door.locked == false
        effect Door.opened = true
    }
}
"""


class HoloCodeTests(unittest.TestCase):
    def test_parses_world_graph(self):
        world = parse(PROGRAM)
        self.assertEqual(world.name, "Building")
        self.assertEqual(set(world.entities), {"User", "Door"})
        self.assertEqual(world.relations["Near"].max_distance, 2)
        self.assertEqual(world.phenomena[0].name, "AutomaticOpening")

    def test_relation_triggers_phenomenon(self):
        runtime = Runtime(parse(PROGRAM))
        events = runtime.tick()
        self.assertTrue(runtime.relation_state["Near"])
        self.assertTrue(runtime.entity("Door")["opened"])
        self.assertEqual(events[0].phenomenon, "AutomaticOpening")

    def test_distant_entity_does_not_trigger(self):
        runtime = Runtime(parse(PROGRAM))
        runtime.move("User", Vector3(10, 0, 0))
        events = runtime.tick()
        self.assertFalse(runtime.relation_state["Near"])
        self.assertFalse(runtime.entity("Door")["opened"])
        self.assertEqual(events, [])

    def test_locked_door_does_not_open(self):
        runtime = Runtime(parse(PROGRAM))
        runtime.entity("Door")["locked"] = True
        runtime.tick()
        self.assertFalse(runtime.entity("Door")["opened"])

    def test_effects_are_idempotent(self):
        runtime = Runtime(parse(PROGRAM))
        self.assertEqual(len(runtime.tick()), 1)
        self.assertEqual(runtime.tick(), [])

    def test_conflicting_effects_are_rejected(self):
        conflicting = PROGRAM.replace(
            "}\n}",
            "}\nphenomenon ForcedClosing {\n"
            "when Near active\n"
            "effect Door.opened = false\n}\n}",
        )
        runtime = Runtime(parse(conflicting))
        with self.assertRaisesRegex(RuntimeError, "Conflit"):
            runtime.tick()

    def test_unknown_entity_is_rejected(self):
        invalid = PROGRAM.replace("Near(User, Door)", "Near(Ghost, Door)")
        with self.assertRaisesRegex(RuntimeError, "Ghost"):
            Runtime(parse(invalid))

    def test_units_other_than_meters_are_rejected(self):
        invalid = PROGRAM.replace("distance < 2m", "distance < 2km")
        with self.assertRaisesRegex(ParseError, "unite 'm'"):
            parse(invalid)


if __name__ == "__main__":
    unittest.main()

