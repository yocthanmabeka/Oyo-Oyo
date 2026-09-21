"""Scénario commenté sur exemples/maison.holo.

    python exemples/demo_maison.py
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

from holocode import World  # noqa: E402

world = World.load((Path(__file__).parent / "maison.holo").read_text(encoding="utf-8"))


def step(title, action=None, ticks=1):
    print(f"\n--- {title}")
    events = list(action() if action else [])
    for _ in range(ticks):
        events += world.tick(1.0)
    for event in events:
        print(event.format())
    if not events:
        print("(rien ne change)")


step("t=1 s : Alice est devant FrontDoor, Bob devant BackDoor verrouillée, Gate est dans un autre espace")
step("Un joueur déverrouille BackDoor, puis le temps avance", lambda: world.invoke("BackDoor", "unlock"))
step("Alice s'éloigne ; FrontDoor doit rester ouverte 3 s", lambda: world.move("Alice", 10, 0, 0) or [], ticks=3)
step("t=6 s : trois secondes sans personne")
step("Un joueur verrouille FrontDoor puis tente de l'ouvrir de force",
     lambda: world.invoke("FrontDoor", "lock") + world.invoke("FrontDoor", "open"), ticks=0)

print("\n--- Pourquoi BackDoor est-elle ouverte ?")
print(world.why("BackDoor", "opened").format())

print(f"\nTests de distance effectués : {world.stats['distance_tests']} "
      f"(une double boucle en aurait fait {world.stats['naive_tests']})")
