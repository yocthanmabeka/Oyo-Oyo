"""Vérification statique de HoloCode v0.1.

Tout ce qui est contrôlé ici l'est avant la première seconde de simulation :
noms, types, unités physiques, composition des archétypes, écritures hors
capacité, et conflits possibles entre phénomènes.
"""

from dataclasses import dataclass

from .arbre import FieldAtom, Pos, RelationAtom
from .units import ORDERED_TYPES


@dataclass(frozen=True)
class Diagnostic:
    severity: str  # erreur | avertissement
    message: str
    pos: Pos

    def __str__(self):
        return f"{self.severity} ligne {self.pos.line}, colonne {self.pos.col} : {self.message}"


class CheckError(Exception):
    def __init__(self, diagnostics):
        super().__init__("\n".join(str(d) for d in diagnostics))
        self.diagnostics = diagnostics


def compose(names, archetypes):
    """Réunit plusieurs archétypes.

    Retourne (champs, capacités, collisions). La composition est plate : pas
    d'héritage, pas de priorité. Deux archétypes qui déclarent le même champ
    ou la même capacité ne peuvent pas être composés.
    """
    fields, capabilities, collisions = {}, {}, []
    for name in names:
        archetype = archetypes.get(name)
        if archetype is None:
            continue
        for field in archetype.fields:
            if field.name in fields:
                collisions.append(f"le champ « {field.name} » est déclaré par {fields[field.name][0]} et par {name}")
            else:
                fields[field.name] = (name, field)
        for capability in archetype.capabilities:
            if capability.name in capabilities:
                collisions.append(
                    f"la capacité « {capability.name} » est déclarée par {capabilities[capability.name][0]} et par {name}"
                )
            else:
                capabilities[capability.name] = (name, capability)
    return fields, capabilities, collisions


class Checker:
    def __init__(self, program):
        self.program = program
        self.world = program.world
        self.diagnostics = []
        self.archetypes = {}
        self.relations = {}

    def error(self, message, pos):
        self.diagnostics.append(Diagnostic("erreur", message, pos))

    def warn(self, message, pos):
        self.diagnostics.append(Diagnostic("avertissement", message, pos))

    def run(self):
        self.check_archetypes()
        self.check_entities()
        self.check_relations()
        for law in self.world.laws:
            self.check_rule(law, "loi", law.where)
        for phenomenon in self.world.phenomena:
            self.check_rule(phenomenon, "phénomène", phenomenon.when)
        self.check_unique([*self.world.laws, *self.world.phenomena], "la règle")
        self.check_possible_conflicts()
        return self.diagnostics

    def check_unique(self, items, what):
        seen = set()
        for item in items:
            if item.name in seen:
                self.error(f"{what} « {item.name} » est déclaré(e) deux fois", item.pos)
            seen.add(item.name)

    def check_type(self, literal, expected, what):
        if literal.type != expected:
            self.error(f"{what} attend une valeur de type {expected}, pas {literal.type}", literal.pos)

    def known(self, names, pos):
        ok = True
        for name in names:
            if name not in self.archetypes:
                self.error(f"archétype inconnu « {name} »", pos)
                ok = False
        if len(set(names)) != len(names):
            self.error("le même archétype est cité deux fois", pos)
            ok = False
        return ok

    # --- archétypes et entités ---------------------------------------------

    def check_archetypes(self):
        self.check_unique(self.program.archetypes, "l'archétype")
        for archetype in self.program.archetypes:
            self.archetypes.setdefault(archetype.name, archetype)
            self.check_unique(archetype.fields, "le champ")
            self.check_unique(archetype.capabilities, "la capacité")
            own = {field.name: field for field in archetype.fields}
            for field in archetype.fields:
                self.check_type(field.default, field.type, f"le champ « {field.name} »")
            for capability in archetype.capabilities:
                for assignment in capability.assignments:
                    field = own.get(assignment.field)
                    if field is None:
                        self.error(
                            f"la capacité « {capability.name} » écrit « {assignment.field} », "
                            f"qui n'est pas un champ de {archetype.name} : une capacité ne modifie que son propre archétype",
                            assignment.pos,
                        )
                    else:
                        self.check_type(assignment.value, field.type, f"le champ « {field.name} »")

    def check_entities(self):
        self.check_unique(self.world.spaces, "l'espace")
        entities = [entity for space in self.world.spaces for entity in space.entities]
        self.check_unique(entities, "l'entité")
        for entity in entities:
            if not self.known(entity.archetypes, entity.pos):
                continue
            fields, _, collisions = compose(entity.archetypes, self.archetypes)
            for collision in collisions:
                self.error(f"composition impossible pour « {entity.name} » : {collision}", entity.pos)
            for axis, coordinate in zip("xyz", entity.position):
                self.check_type(coordinate, "Length", f"la coordonnée {axis} de « {entity.name} »")
            for override in entity.overrides:
                if override.field not in fields:
                    self.error(f"« {entity.name} » n'a pas de champ « {override.field} »", override.pos)
                else:
                    self.check_type(override.value, fields[override.field][1].type, f"le champ « {override.field} »")

    # --- relations ---------------------------------------------------------

    def check_relations(self):
        self.check_unique(self.world.relations, "la relation")
        for relation in self.world.relations:
            self.relations.setdefault(relation.name, relation)
            self.known(relation.left.archetypes, relation.left.pos)
            self.known(relation.right.archetypes, relation.right.pos)
            if relation.left.var == relation.right.var:
                self.error("les deux paramètres d'une relation doivent porter des noms différents", relation.pos)
            if set(relation.distance_args) != {relation.left.var, relation.right.var}:
                self.error("distance(…) doit porter sur les deux paramètres de la relation", relation.pos)
            if relation.op not in ("<", "<="):
                self.error(
                    f"une relation de proximité v0.1 s'écrit avec < ou <=, pas « {relation.op} »", relation.pos
                )
            self.check_type(relation.threshold, "Length", "la distance d'une relation")
            if relation.threshold.type == "Length" and relation.threshold.value <= 0:
                self.error("la distance d'une relation doit être positive", relation.threshold.pos)

    # --- lois et phénomènes --------------------------------------------------

    def check_rule(self, rule, what, terms):
        if not self.known(rule.binder.archetypes, rule.binder.pos):
            return
        fields, capabilities, _ = compose(rule.binder.archetypes, self.archetypes)
        if rule.var != rule.binder.var:
            self.error(f"variable inconnue « {rule.var} » dans {what} « {rule.name} »", rule.pos)
        if rule.capability not in capabilities:
            offered = ", ".join(sorted(capabilities)) or "aucune"
            self.error(
                f"« {' + '.join(rule.binder.archetypes)} » n'offre pas la capacité « {rule.capability} » "
                f"(capacités offertes : {offered})",
                rule.pos,
            )
        for term in terms:
            self.check_term(term, rule.binder, fields)

    def check_term(self, term, binder, fields):
        atom = term.atom
        if term.duration is not None:
            self.check_type(term.duration, "Duration", "« for »")
            if term.duration.type == "Duration" and term.duration.value <= 0:
                self.error("la durée de « for » doit être positive", term.duration.pos)
        if isinstance(atom, FieldAtom):
            if atom.var != binder.var:
                self.error(f"variable inconnue « {atom.var} »", atom.pos)
                return
            if atom.field not in fields:
                self.error(f"« {' + '.join(binder.archetypes)} » n'a pas de champ « {atom.field} »", atom.pos)
                return
            type_ = fields[atom.field][1].type
            if atom.op is None:
                if type_ != "Bool":
                    self.error(f"« {atom.var}.{atom.field} » est de type {type_} : une comparaison est nécessaire", atom.pos)
                return
            self.check_type(atom.value, type_, f"la comparaison avec « {atom.field} »")
            if atom.op not in ("==", "!=") and type_ not in ORDERED_TYPES:
                self.error(f"« {atom.op} » ne s'applique pas au type {type_}", atom.pos)
            return
        relation = self.relations.get(atom.relation)
        if relation is None:
            self.error(f"relation inconnue « {atom.relation} »", atom.pos)
            return
        bound = [i for i, arg in enumerate(atom.args) if arg != "_"]
        if len(bound) != 1 or atom.args[bound[0]] != binder.var:
            self.error(
                f"v0.1 : « {atom.relation}(…) » doit citer une fois « {binder.var} » et une fois « _ »", atom.pos
            )
            return
        required = (relation.left, relation.right)[bound[0]].archetypes
        missing = [name for name in required if name not in binder.archetypes]
        if missing:
            self.error(
                f"« {binder.var} » doit être composé de {' + '.join(required)} pour occuper cette place "
                f"de « {atom.relation} » (manque : {', '.join(missing)})",
                atom.pos,
            )

    # --- conflits possibles entre phénomènes ---------------------------------

    @staticmethod
    def polarity(term):
        """Clé d'un terme indépendante du nom de la variable, et son signe."""
        atom = term.atom
        if isinstance(atom, RelationAtom):
            key = ("relation", atom.relation, tuple("_" if arg == "_" else "$" for arg in atom.args))
            positive = atom.quantifier == "some"
        else:
            value = None if atom.value is None else (atom.value.type, atom.value.value)
            key = ("field", atom.field, atom.op, value)
            positive = True
        return key, positive != term.negated

    def exclusive(self, first, second):
        """Vrai si les deux conditions ne peuvent pas tenir ensemble : l'une exige
        un terme et l'autre son contraire. Analyse syntaxique, donc prudente."""
        signs = dict(self.polarity(term) for term in first.when)
        return any(key in signs and signs[key] != sign for key, sign in map(self.polarity, second.when))

    def check_possible_conflicts(self):
        entities = [entity for space in self.world.spaces for entity in space.entities]
        phenomena = self.world.phenomena
        for i, first in enumerate(phenomena):
            for second in phenomena[i + 1:]:
                shared = [
                    entity.name for entity in entities
                    if set(first.binder.archetypes) <= set(entity.archetypes)
                    and set(second.binder.archetypes) <= set(entity.archetypes)
                ]
                if not shared or self.exclusive(first, second):
                    continue
                writes = []
                for phenomenon in (first, second):
                    _, capabilities, _ = compose(phenomenon.binder.archetypes, self.archetypes)
                    capability = capabilities.get(phenomenon.capability)
                    writes.append({a.field: a.value.value for a in capability[1].assignments} if capability else {})
                clash = sorted(f for f in writes[0] if f in writes[1] and writes[0][f] != writes[1][f])
                if clash:
                    self.warn(
                        f"conflit possible : « {first.name} » et « {second.name} » peuvent écrire des valeurs "
                        f"différentes dans « {', '.join(clash)} » au même instant (par exemple sur {shared[0]}) "
                        f"et leurs conditions ne s'excluent pas",
                        second.pos,
                    )


def check(program):
    return Checker(program).run()
