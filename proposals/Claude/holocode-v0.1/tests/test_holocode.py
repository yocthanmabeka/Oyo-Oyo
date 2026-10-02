"""Tests de HoloCode v0.1 (proposition Claude).

    python -m unittest discover -s tests -v
"""

import sys
import unittest
from math import dist
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from holocode import CheckError, ConflictError, HoloSyntaxError, World, check, parse  # noqa: E402

MAISON = (ROOT / "exemples" / "maison.holo").read_text(encoding="utf-8")

ARCHETYPES = """
archetype Person { }
archetype Openable {
    state { opened: Bool = false }
    capability open  { opened = true }
    capability close { opened = false }
}
archetype Lockable {
    state { locked: Bool = false }
    capability lock   { locked = true }
    capability unlock { locked = false }
}
"""


def errors(source):
    return [d.message for d in check(parse(source)) if d.severity == "erreur"]


def warnings(source):
    return [d.message for d in check(parse(source)) if d.severity == "avertissement"]


def run(world, ticks, dt=1.0):
    return [event for _ in range(ticks) for event in world.tick(dt)]


class Exemple(unittest.TestCase):
    def test_la_maison_est_bien_formee_et_sans_avertissement(self):
        self.assertEqual(check(parse(MAISON)), [])

    def test_le_contre_exemple_est_refuse_avant_execution(self):
        source = (ROOT / "exemples" / "erreurs.holo").read_text(encoding="utf-8")
        self.assertEqual(len(errors(source)), 6)
        with self.assertRaises(CheckError):
            World.load(source)


class Relations(unittest.TestCase):
    def test_une_relation_est_un_ensemble_de_paires(self):
        world = World.load(MAISON)
        self.assertEqual(world.relations["Near"], [("Alice", "FrontDoor"), ("Bob", "BackDoor")])

    def test_chaque_porte_reagit_a_sa_propre_paire(self):
        world = World.load(MAISON)
        world.invoke("BackDoor", "unlock")
        world.move("Alice", 100, 0, 0)  # seul Bob reste près d'une porte
        run(world, 1)
        self.assertFalse(world.entities["FrontDoor"].state["opened"])
        self.assertTrue(world.entities["BackDoor"].state["opened"])

    def test_deux_espaces_ne_se_comparent_jamais(self):
        world = World.load(MAISON)
        run(world, 3)
        # Gate a les coordonnées de FrontDoor, à 1 m d'Alice, mais dans Garden.
        self.assertFalse(world.entities["Gate"].state["opened"])
        self.assertNotIn(("Alice", "Gate"), world.relations["Near"])

    def test_la_grille_donne_le_meme_resultat_qu_une_double_boucle(self):
        people = "\n".join(f"entity P{i}: Person at ({i * 7 % 53}m, {i * 3 % 11}m, 0m)" for i in range(60))
        doors = "\n".join(f"entity D{i}: Openable at ({i * 5 % 47}m, {i * 2 % 13}m, 1m)" for i in range(60))
        world = World.load(ARCHETYPES + f"""
            world Big {{
                space S {{ {people} {doors} }}
                relation Near(p: Person, d: Openable) when distance(p, d) <= 2m
            }}""")
        expected = {
            (p.name, d.name)
            for p in world.entities.values() if "Person" in p.archetypes
            for d in world.entities.values() if "Openable" in d.archetypes
            if dist(p.position, d.position) <= 2
        }
        self.assertTrue(expected)
        self.assertEqual(set(world.relations["Near"]), expected)

    def test_la_grille_evite_la_double_boucle(self):
        people = "\n".join(f"entity P{i}: Person at ({i * 10}m, 0m, 0m)" for i in range(500))
        doors = "\n".join(f"entity D{i}: Openable at ({i * 10}m, 0m, 1m)" for i in range(500))
        world = World.load(ARCHETYPES + f"""
            world Big {{
                space S {{ {people} {doors} }}
                relation Near(p: Person, d: Openable) when distance(p, d) <= 2m
            }}""")
        self.assertEqual(len(world.relations["Near"]), 500)
        self.assertEqual(world.stats["naive_tests"], 250_000)
        self.assertLess(world.stats["distance_tests"], 2_500)  # moins de 1 % de la double boucle


class LoisEtCapacites(unittest.TestCase):
    def test_la_loi_refuse_et_le_refus_est_trace_une_seule_fois(self):
        world = World.load(MAISON)
        events = run(world, 4)
        refusals = [e for e in events if e.kind == "refus"]
        self.assertEqual(len(refusals), 1)
        self.assertEqual((refusals[0].entity, refusals[0].law), ("BackDoor", "LockedStaysClosed"))
        self.assertFalse(world.entities["BackDoor"].state["opened"])

    def test_la_porte_s_ouvre_des_que_la_loi_ne_s_applique_plus(self):
        world = World.load(MAISON)
        run(world, 2)
        world.invoke("BackDoor", "unlock")
        run(world, 1)
        self.assertTrue(world.entities["BackDoor"].state["opened"])

    def test_la_loi_vaut_aussi_pour_une_invocation_externe(self):
        world = World.load(MAISON)
        [event] = world.invoke("BackDoor", "open")
        self.assertEqual((event.kind, event.law), ("refus", "LockedStaysClosed"))
        self.assertFalse(world.entities["BackDoor"].state["opened"])

    def test_la_loi_ignore_une_entite_qui_n_a_pas_ses_archetypes(self):
        world = World.load(MAISON)
        world.move("Carol", 0, 0, 0)  # Gate est Openable mais pas Lockable
        run(world, 1)
        self.assertTrue(world.entities["Gate"].state["opened"])

    def test_l_etat_ne_s_ecrit_pas_hors_capacite(self):
        world = World.load(MAISON)
        with self.assertRaises(TypeError):
            world.entities["BackDoor"].state["opened"] = True


class Temps(unittest.TestCase):
    def test_la_porte_se_referme_apres_trois_secondes_sans_personne(self):
        world = World.load(MAISON)
        run(world, 1)
        world.move("Alice", 50, 0, 0)
        run(world, 2)  # t=3 : Alice absente depuis t=2, soit 1 s
        self.assertTrue(world.entities["FrontDoor"].state["opened"])
        run(world, 2)  # t=5 : absente depuis 3 s
        self.assertFalse(world.entities["FrontDoor"].state["opened"])

    def test_un_retour_remet_le_compteur_a_zero(self):
        world = World.load(MAISON)
        run(world, 1)
        world.move("Alice", 50, 0, 0)
        run(world, 2)
        world.move("Alice", 0, 0, 0)
        run(world, 1)
        world.move("Alice", 50, 0, 0)
        run(world, 2)
        self.assertTrue(world.entities["FrontDoor"].state["opened"])

    def test_la_duree_ne_depend_pas_de_la_taille_du_pas(self):
        world = World.load(MAISON)
        run(world, 1)
        world.move("Alice", 50, 0, 0)
        run(world, 34, dt=0.1)  # t=4,4 : absence observée depuis t=1,1, soit 3,3 s
        self.assertFalse(world.entities["FrontDoor"].state["opened"])


class Causalite(unittest.TestCase):
    def test_le_journal_dit_pourquoi(self):
        world = World.load(MAISON)
        run(world, 1)
        event = world.why("FrontDoor", "opened")
        self.assertEqual(event.source, "le phénomène AutoOpening")
        self.assertIn("Near(Alice, FrontDoor) à 1.00 m", event.reasons[0])
        self.assertIsNone(world.why("Gate", "opened"))

    def test_deux_executions_donnent_le_meme_journal(self):
        def scenario():
            world = World.load(MAISON)
            run(world, 2)
            world.invoke("BackDoor", "unlock")
            world.move("Alice", 9, 0, 0)
            run(world, 6)
            return [event.format() for event in world.trace], world.snapshot()

        self.assertEqual(scenario(), scenario())


CONFLIT = ARCHETYPES + """
world Clash {
    space S {
        entity Alice: Person at (0m, 0m, 0m)
        entity Door: Openable at (0m, 0m, 1m)
    }
    relation Near(p: Person, d: Openable) when distance(p, d) <= 2m
    phenomenon Welcome { forall d: Openable when some Near(_, d) effect d.open }
    phenomenon Curfew  { forall d: Openable when some Near(_, d) effect d.close }
}
"""


class Conflits(unittest.TestCase):
    def test_le_verificateur_annonce_le_conflit_possible(self):
        [message] = warnings(CONFLIT)
        self.assertIn("Welcome", message)
        self.assertIn("Curfew", message)

    def test_le_conflit_arrete_le_pas_sans_rien_ecrire(self):
        world = World.load(CONFLIT)
        with self.assertRaises(ConflictError):
            world.tick()
        self.assertEqual((world.time, world.tick_count, world.trace), (0.0, 0, []))
        self.assertFalse(world.entities["Door"].state["opened"])

    def test_des_conditions_contraires_ne_declenchent_pas_d_avertissement(self):
        self.assertEqual(warnings(CONFLIT.replace("when some Near(_, d) effect d.close", "when no Near(_, d) effect d.close")), [])


class Verification(unittest.TestCase):
    def world(self, body):
        return ARCHETYPES + f"world W {{ space S {{ entity A: Person at (0m, 0m, 0m) entity D: Openable at (0m, 0m, 1m) }} {body} }}"

    def test_une_distance_ne_se_mesure_pas_en_secondes(self):
        [message] = errors(self.world("relation Near(p: Person, d: Openable) when distance(p, d) <= 3s"))
        self.assertIn("Length, pas Duration", message)

    def test_for_attend_une_duree(self):
        source = self.world("""
            relation Near(p: Person, d: Openable) when distance(p, d) <= 2m
            phenomenon P { forall d: Openable when no Near(_, d) for 2m effect d.close }""")
        self.assertIn("Duration, pas Length", errors(source)[0])

    def test_les_unites_se_convertissent(self):
        world = World.load(self.world("relation Near(p: Person, d: Openable) when distance(p, d) <= 150cm"))
        self.assertEqual(world.relations["Near"], [("A", "D")])

    def test_une_relation_exige_les_bons_archetypes(self):
        source = self.world("""
            relation Near(p: Person, d: Openable) when distance(p, d) <= 2m
            law L { forall x: Person where some Near(_, x) forbid x.open }""")
        found = errors(source)
        self.assertTrue(any("n'offre pas la capacité" in m for m in found))
        self.assertTrue(any("doit être composé de Openable" in m for m in found))

    def test_les_noms_inconnus_sont_refuses(self):
        source = self.world("phenomenon P { forall d: Window when d.opened effect d.close }")
        self.assertIn("archétype inconnu « Window »", errors(source)[0])

    def test_une_erreur_de_syntaxe_donne_sa_position(self):
        with self.assertRaises(HoloSyntaxError) as caught:
            parse("world W { space S { entity A Person at (0m, 0m, 0m) } }")
        self.assertEqual((caught.exception.line, caught.exception.col), (1, 30))

    def test_une_unite_inconnue_est_refusee(self):
        with self.assertRaises(HoloSyntaxError):
            parse("world W { space S { entity A: Person at (0m, 0m, 3parsecs) } }")


if __name__ == "__main__":
    unittest.main()
