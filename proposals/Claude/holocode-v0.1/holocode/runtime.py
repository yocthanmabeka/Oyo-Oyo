"""Exécution déterministe de HoloCode v0.1.

Un pas de temps (`tick`) se déroule toujours dans le même ordre :

1. le temps avance ;
2. les relations sont recalculées sur un instantané stable ;
3. chaque phénomène examine chaque entité concernée et formule des intentions ;
4. les lois refusent les intentions interdites ;
5. deux intentions qui se contredisent arrêtent le pas, sans rien écrire ;
6. les capacités autorisées s'appliquent ;
7. chaque changement et chaque refus entre dans le journal causal.
"""

import operator
from dataclasses import dataclass
from math import dist, floor
from types import MappingProxyType

from .arbre import FieldAtom, Law
from .checker import CheckError, check, compose
from .parser import parse
from .units import show

OPS = {
    "==": operator.eq, "!=": operator.ne,
    "<": operator.lt, "<=": operator.le, ">": operator.gt, ">=": operator.ge,
}
EPSILON = 1e-9
NEIGHBOURS = [(dx, dy, dz) for dx in (-1, 0, 1) for dy in (-1, 0, 1) for dz in (-1, 0, 1)]


class HoloRuntimeError(Exception):
    pass


class ConflictError(HoloRuntimeError):
    """Deux intentions du même instant écrivent des valeurs différentes."""


@dataclass(frozen=True)
class Change:
    field: str
    before: str
    after: str


@dataclass(frozen=True)
class Event:
    """Une ligne du journal causal : un effet appliqué ou un refus."""

    tick: int
    time: float
    kind: str  # effet | refus
    source: str  # « phénomène X » ou « invocation externe »
    entity: str
    capability: str
    changes: tuple[Change, ...]
    reasons: tuple[str, ...]
    law: str | None = None

    def format(self):
        head = f"[t={self.time:g} s, pas {self.tick}] {self.entity}.{self.capability}"
        if self.kind == "refus":
            lines = [f"{head} REFUSÉ par la loi {self.law} (demandé par {self.source})"]
        else:
            lines = [f"{head} par {self.source}"]
            lines += [f"    {c.field} : {c.before} -> {c.after}" for c in self.changes]
        lines += [f"    parce que {reason}" for reason in self.reasons]
        return "\n".join(lines)


class Entity:
    """État d'une entité. `state` est en lecture seule : seules les capacités,
    appelées par le monde, peuvent le modifier."""

    def __init__(self, decl, fields, capabilities):
        self.name = decl.name
        self.space = decl.space
        self.archetypes = frozenset(decl.archetypes)
        self.position = tuple(coordinate.value for coordinate in decl.position)
        self.types = {name: field.type for name, (_, field) in fields.items()}
        self.capabilities = {name: capability for name, (_, capability) in capabilities.items()}
        self._state = {name: field.default.value for name, (_, field) in fields.items()}
        for override in decl.overrides:
            self._state[override.field] = override.value.value
        self.state = MappingProxyType(self._state)

    def shown(self, field):
        return show(self.types[field], self._state[field])


@dataclass(frozen=True)
class Intention:
    source: str
    entity: Entity
    capability: str
    reasons: tuple[str, ...]


class World:
    def __init__(self, program):
        diagnostics = check(program)
        errors = [d for d in diagnostics if d.severity == "erreur"]
        if errors:
            raise CheckError(errors)
        self.warnings = [d for d in diagnostics if d.severity == "avertissement"]
        self.name = program.world.name
        self.time = 0.0
        self.tick_count = 0
        self.trace = []
        self.stats = {"distance_tests": 0, "naive_tests": 0}

        archetypes = {a.name: a for a in program.archetypes}
        self.entities = {}
        for space in program.world.spaces:
            for decl in space.entities:
                fields, capabilities, _ = compose(decl.archetypes, archetypes)
                self.entities[decl.name] = Entity(decl, fields, capabilities)

        self._relation_decls = program.world.relations
        self._laws = program.world.laws
        self._phenomena = program.world.phenomena
        self._matching = {}  # Binder -> entités concernées, dans l'ordre de déclaration
        for relation in self._relation_decls:
            self._index(relation.left)
            self._index(relation.right)
        for rule in (*self._laws, *self._phenomena):
            self._index(rule.binder)

        self.relations = {}  # nom -> paires triées (gauche, droite)
        self._distances = {}  # (nom, gauche, droite) -> distance en mètres
        self._by_side = {}  # (nom, côté, entité) -> paires où elle figure
        self._since = {}  # (règle, n° du terme, entité) -> instant où le terme est devenu vrai
        self._refused = set()  # refus du pas précédent, pour ne pas les répéter
        self._observe()

    @classmethod
    def load(cls, source):
        return cls(parse(source))

    def _index(self, binder):
        wanted = set(binder.archetypes)
        self._matching[binder] = [e for e in self.entities.values() if wanted <= e.archetypes]

    # --- relations spatiales -------------------------------------------------

    def _compute_relations(self):
        """Grille uniforme par espace : une entité n'est comparée qu'à celles des
        27 cases voisines, au lieu de toutes. Deux espaces ne se comparent jamais."""
        self.relations, self._distances, self._by_side = {}, {}, {}
        order = {name: i for i, name in enumerate(self.entities)}
        for relation in self._relation_decls:
            lefts = self._matching[relation.left]
            rights = self._matching[relation.right]
            radius = relation.threshold.value
            within = OPS[relation.op]
            grid = {}
            for entity in rights:
                grid.setdefault(self._cell(entity, radius), []).append(entity)
            pairs = []
            for a in lefts:
                space, ix, iy, iz = self._cell(a, radius)
                for dx, dy, dz in NEIGHBOURS:
                    for b in grid.get((space, ix + dx, iy + dy, iz + dz), ()):
                        if a is b:
                            continue
                        self.stats["distance_tests"] += 1
                        distance = dist(a.position, b.position)
                        if within(distance, radius):
                            pairs.append((a.name, b.name))
                            self._distances[relation.name, a.name, b.name] = distance
            self.stats["naive_tests"] += len(lefts) * len(rights)
            pairs.sort(key=lambda pair: (order[pair[0]], order[pair[1]]))
            self.relations[relation.name] = pairs
            for pair in pairs:
                self._by_side.setdefault((relation.name, 0, pair[0]), []).append(pair)
                self._by_side.setdefault((relation.name, 1, pair[1]), []).append(pair)

    @staticmethod
    def _cell(entity, size):
        x, y, z = entity.position
        return entity.space, floor(x / size), floor(y / size), floor(z / size)

    # --- conditions ----------------------------------------------------------

    def _atom(self, atom, entity):
        """Retourne (vrai ?, texte de l'atome, témoins)."""
        if isinstance(atom, FieldAtom):
            value = entity.state[atom.field]
            if atom.op is None:
                return bool(value), f"{entity.name}.{atom.field}", ""
            text = f"{entity.name}.{atom.field} {atom.op} {show(atom.value.type, atom.value.value)}"
            return OPS[atom.op](value, atom.value.value), text, f" (vaut {entity.shown(atom.field)})"
        side = 0 if atom.args[0] != "_" else 1
        pairs = self._by_side.get((atom.relation, side, entity.name), [])
        shown_args = ", ".join(entity.name if arg != "_" else "_" for arg in atom.args)
        text = f"{atom.quantifier} {atom.relation}({shown_args})"
        if atom.quantifier == "no":
            return not pairs, text, ""
        witnesses = ", ".join(
            f"{atom.relation}({a}, {b}) à {self._distances[atom.relation, a, b]:.2f} m" for a, b in pairs[:3]
        )
        more = f" et {len(pairs) - 3} autre(s)" if len(pairs) > 3 else ""
        return bool(pairs), text, f" : {witnesses}{more}" if pairs else ""

    def _term_now(self, term, entity):
        truth, text, detail = self._atom(term.atom, entity)
        if term.negated:
            return not truth, f"not {text}", detail
        return truth, text, detail

    def _condition(self, rule, terms, entity):
        """Retourne (tient ?, raisons). Les raisons alimentent le journal causal."""
        reasons = []
        for number, term in enumerate(terms):
            truth, text, detail = self._term_now(term, entity)
            if not truth:
                return False, ()
            if term.duration is not None:
                since = self._since.get((rule.name, number, entity.name))
                if since is None or self.time - since < term.duration.value - EPSILON:
                    return False, ()
                detail += f" depuis {self.time - since:g} s (exigé : {term.duration.value:g} s)"
            reasons.append(text + detail)
        return True, tuple(reasons)

    def _observe(self):
        """Recalcule les relations, puis note depuis quand chaque terme `for` est vrai."""
        self._compute_relations()
        for rule in (*self._laws, *self._phenomena):
            terms = rule.where if isinstance(rule, Law) else rule.when
            for number, term in enumerate(terms):
                if term.duration is None:
                    continue
                for entity in self._matching[rule.binder]:
                    key = (rule.name, number, entity.name)
                    if self._term_now(term, entity)[0]:
                        self._since.setdefault(key, self.time)
                    else:
                        self._since.pop(key, None)

    # --- lois ------------------------------------------------------------------

    def _forbidden(self, entity, capability):
        """Première loi, dans l'ordre de déclaration, qui interdit cette capacité."""
        for law in self._laws:
            if law.capability != capability or not set(law.binder.archetypes) <= entity.archetypes:
                continue
            holds, reasons = self._condition(law, law.where, entity)
            if holds:
                return law.name, reasons
        return None

    # --- exécution -------------------------------------------------------------

    def tick(self, dt=1.0):
        """Avance le monde de `dt` secondes et retourne les événements du pas."""
        if dt <= 0:
            raise ValueError("dt doit être positif")
        saved = (self.time, self.tick_count, dict(self._since), set(self._refused))
        self.time += dt
        self.tick_count += 1
        self._observe()
        intentions = []
        for phenomenon in self._phenomena:
            for entity in self._matching[phenomenon.binder]:
                holds, reasons = self._condition(phenomenon, phenomenon.when, entity)
                if holds:
                    intentions.append(Intention(f"le phénomène {phenomenon.name}", entity, phenomenon.capability, reasons))
        try:
            return self._resolve(intentions, repeat_refusals=False)
        except ConflictError:
            self.time, self.tick_count, self._since, self._refused = saved
            raise

    def invoke(self, entity_name, capability):
        """Demande venue de l'extérieur (joueur, script, autre programme).
        Elle passe par les mêmes lois que les phénomènes."""
        entity = self._entity(entity_name)
        if capability not in entity.capabilities:
            raise HoloRuntimeError(f"« {entity_name} » n'a pas la capacité « {capability} »")
        intention = Intention("une invocation externe", entity, capability, ())
        return self._resolve([intention], repeat_refusals=True)

    def move(self, entity_name, x, y, z):
        """Déplace une entité (coordonnées en mètres). En v0.1 le mouvement est une
        entrée du monde, pas un état : il sera pris en compte au prochain pas."""
        self._entity(entity_name).position = (float(x), float(y), float(z))

    def _entity(self, name):
        try:
            return self.entities[name]
        except KeyError:
            raise HoloRuntimeError(f"entité inconnue « {name} »") from None

    def _resolve(self, intentions, repeat_refusals):
        allowed, refusals, refused_now = [], {}, set()
        for intention in intentions:
            verdict = self._forbidden(intention.entity, intention.capability)
            if verdict is None:
                allowed.append(intention)
                continue
            law, reasons = verdict
            key = (intention.source, intention.entity.name, intention.capability, law)
            refused_now.add(key)
            if repeat_refusals or key not in self._refused:
                refusals[id(intention)] = self._event("refus", intention, (), reasons, law)

        writes = {}
        for intention in allowed:
            for assignment in intention.entity.capabilities[intention.capability].assignments:
                key = (intention.entity.name, assignment.field)
                earlier = writes.setdefault(key, (assignment.value.value, intention))
                if earlier[0] != assignment.value.value:
                    raise ConflictError(
                        f"conflit à t={self.time:g} s sur {key[0]}.{key[1]} : {earlier[1].source} "
                        f"({earlier[1].capability}) et {intention.source} ({intention.capability}) "
                        f"demandent des valeurs différentes ; aucun effet de ce pas n'a été appliqué"
                    )

        events, permitted = [], {id(intention) for intention in allowed}
        for intention in intentions:  # le journal suit l'ordre des intentions
            if id(intention) not in permitted:
                if id(intention) in refusals:
                    events.append(refusals[id(intention)])
                continue
            entity, changes = intention.entity, []
            for assignment in entity.capabilities[intention.capability].assignments:
                before = entity.shown(assignment.field)
                if entity._state[assignment.field] != assignment.value.value:
                    entity._state[assignment.field] = assignment.value.value
                    changes.append(Change(assignment.field, before, entity.shown(assignment.field)))
            if changes:
                events.append(self._event("effet", intention, tuple(changes), intention.reasons, None))

        if not repeat_refusals:
            self._refused = refused_now
        self.trace.extend(events)
        return events

    def _event(self, kind, intention, changes, reasons, law):
        return Event(
            self.tick_count, self.time, kind, intention.source,
            intention.entity.name, intention.capability, changes, reasons, law,
        )

    # --- interrogation ---------------------------------------------------------

    def why(self, entity_name, field):
        """Dernier événement qui a changé ce champ, ou None s'il n'a jamais changé."""
        self._entity(entity_name)
        for event in reversed(self.trace):
            if event.kind == "effet" and event.entity == entity_name and any(c.field == field for c in event.changes):
                return event
        return None

    def snapshot(self):
        return {
            "world": self.name,
            "time": self.time,
            "tick": self.tick_count,
            "relations": {name: list(pairs) for name, pairs in self.relations.items()},
            "entities": {
                e.name: {"space": e.space, "position": e.position, "state": dict(e.state)}
                for e in self.entities.values()
            },
        }
