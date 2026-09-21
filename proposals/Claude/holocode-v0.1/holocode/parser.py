"""Analyse syntaxique de HoloCode v0.1 (descente récursive)."""

from .arbre import (
    Archetype,
    Assignment,
    Binder,
    Capability,
    EntityDecl,
    Field,
    FieldAtom,
    Law,
    Literal,
    Phenomenon,
    Pos,
    Program,
    Relation,
    RelationAtom,
    Space,
    Term,
    World,
)
from .lexer import HoloSyntaxError, tokenize
from .units import TYPES

RESERVED = {
    "archetype", "state", "capability", "world", "space", "entity", "at",
    "relation", "when", "distance", "law", "forall", "where", "forbid",
    "phenomenon", "effect", "and", "not", "some", "no", "for", "true", "false",
}


class Parser:
    def __init__(self, source):
        self.tokens = tokenize(source)
        self.index = 0

    # --- outils -----------------------------------------------------------

    @property
    def current(self):
        return self.tokens[self.index]

    def advance(self):
        token = self.current
        self.index += 1
        return token

    def error(self, message, token=None):
        token = token or self.current
        return HoloSyntaxError(message, token.line, token.col)

    def at_word(self, *words):
        return self.current.kind == "ident" and self.current.text in words

    def at_sym(self, symbol):
        return self.current.kind == "sym" and self.current.text == symbol

    def word(self, word):
        if not self.at_word(word):
            raise self.error(f"« {word} » attendu, « {self.current.text} » trouvé")
        return self.advance()

    def sym(self, symbol):
        if not self.at_sym(symbol):
            raise self.error(f"« {symbol} » attendu, « {self.current.text} » trouvé")
        return self.advance()

    def name(self, what):
        token = self.current
        if token.kind != "ident" or token.text == "_":
            raise self.error(f"{what} attendu, « {token.text} » trouvé")
        if token.text in RESERVED:
            raise self.error(f"« {token.text} » est un mot réservé et ne peut pas nommer {what}")
        return self.advance()

    def skip_comma(self):
        if self.at_sym(","):
            self.advance()

    @staticmethod
    def pos(token):
        return Pos(token.line, token.col)

    # --- valeurs ----------------------------------------------------------

    def literal(self):
        token = self.advance()
        if token.kind == "number":
            return Literal(token.type, token.value, self.pos(token))
        if token.kind == "string":
            return Literal("Text", token.text[1:-1], self.pos(token))
        if token.kind == "ident" and token.text in ("true", "false"):
            return Literal("Bool", token.text == "true", self.pos(token))
        raise self.error(f"valeur attendue, « {token.text} » trouvé", token)

    def archetype_set(self):
        names = [self.name("un archétype").text]
        while self.at_sym("+"):
            self.advance()
            names.append(self.name("un archétype").text)
        return tuple(names)

    def binder(self):
        var = self.name("une variable")
        self.sym(":")
        return Binder(var.text, self.archetype_set(), self.pos(var))

    def assignments(self):
        """`{ champ = valeur ... }`"""
        self.sym("{")
        found = []
        while not self.at_sym("}"):
            field = self.name("un champ")
            self.sym("=")
            found.append(Assignment(field.text, self.literal(), self.pos(field)))
            self.skip_comma()
        self.sym("}")
        return tuple(found)

    # --- déclarations -----------------------------------------------------

    def program(self):
        archetypes, world = [], None
        while self.current.kind != "eof":
            if self.at_word("archetype"):
                archetypes.append(self.archetype())
            elif self.at_word("world"):
                if world is not None:
                    raise self.error("HoloCode v0.1 accepte un seul monde par fichier")
                world = self.world()
            else:
                raise self.error(f"« archetype » ou « world » attendu, « {self.current.text} » trouvé")
        if world is None:
            raise self.error("aucun monde déclaré")
        return Program(tuple(archetypes), world)

    def archetype(self):
        start = self.word("archetype")
        name = self.name("un archétype").text
        self.sym("{")
        fields, capabilities = [], []
        while not self.at_sym("}"):
            if self.at_word("state"):
                self.advance()
                self.sym("{")
                while not self.at_sym("}"):
                    fields.append(self.field())
                    self.skip_comma()
                self.sym("}")
            elif self.at_word("capability"):
                token = self.advance()
                cap_name = self.name("une capacité").text
                capabilities.append(Capability(cap_name, self.assignments(), self.pos(token)))
            else:
                raise self.error(f"« state » ou « capability » attendu, « {self.current.text} » trouvé")
        self.sym("}")
        return Archetype(name, tuple(fields), tuple(capabilities), self.pos(start))

    def field(self):
        name = self.name("un champ")
        self.sym(":")
        type_token = self.advance()
        if type_token.text not in TYPES:
            raise self.error(f"type inconnu « {type_token.text} » ; types possibles : {', '.join(TYPES)}", type_token)
        self.sym("=")
        return Field(name.text, type_token.text, self.literal(), self.pos(name))

    def world(self):
        start = self.word("world")
        name = self.name("un monde").text
        self.sym("{")
        spaces, relations, laws, phenomena = [], [], [], []
        while not self.at_sym("}"):
            if self.at_word("space"):
                spaces.append(self.space())
            elif self.at_word("relation"):
                relations.append(self.relation())
            elif self.at_word("law"):
                laws.append(self.law())
            elif self.at_word("phenomenon"):
                phenomena.append(self.phenomenon())
            else:
                raise self.error(
                    f"« space », « relation », « law » ou « phenomenon » attendu, « {self.current.text} » trouvé"
                )
        self.sym("}")
        return World(name, tuple(spaces), tuple(relations), tuple(laws), tuple(phenomena), self.pos(start))

    def space(self):
        start = self.word("space")
        name = self.name("un espace").text
        self.sym("{")
        entities = []
        while not self.at_sym("}"):
            entities.append(self.entity(name))
        self.sym("}")
        return Space(name, tuple(entities), self.pos(start))

    def entity(self, space):
        start = self.word("entity")
        name = self.name("une entité").text
        self.sym(":")
        archetypes = self.archetype_set()
        self.word("at")
        self.sym("(")
        x = self.literal()
        self.sym(",")
        y = self.literal()
        self.sym(",")
        z = self.literal()
        self.sym(")")
        overrides = self.assignments() if self.at_sym("{") else ()
        return EntityDecl(name, archetypes, (x, y, z), overrides, space, self.pos(start))

    def relation(self):
        start = self.word("relation")
        name = self.name("une relation").text
        self.sym("(")
        left = self.binder()
        self.sym(",")
        right = self.binder()
        self.sym(")")
        self.word("when")
        self.word("distance")
        self.sym("(")
        first = self.name("une variable").text
        self.sym(",")
        second = self.name("une variable").text
        self.sym(")")
        op = self.advance()
        if op.kind != "op":
            raise self.error(f"comparaison attendue, « {op.text} » trouvé", op)
        return Relation(name, left, right, (first, second), op.text, self.literal(), self.pos(start))

    def law(self):
        start = self.word("law")
        name = self.name("une loi").text
        self.sym("{")
        self.word("forall")
        binder = self.binder()
        where = ()
        if self.at_word("where"):
            self.advance()
            where = self.condition()
        self.word("forbid")
        var, capability = self.capability_call()
        self.sym("}")
        return Law(name, binder, where, var, capability, self.pos(start))

    def phenomenon(self):
        start = self.word("phenomenon")
        name = self.name("un phénomène").text
        self.sym("{")
        self.word("forall")
        binder = self.binder()
        self.word("when")
        when = self.condition()
        self.word("effect")
        var, capability = self.capability_call()
        self.sym("}")
        return Phenomenon(name, binder, when, var, capability, self.pos(start))

    def capability_call(self):
        var = self.name("une variable").text
        self.sym(".")
        return var, self.name("une capacité").text

    # --- conditions -------------------------------------------------------

    def condition(self):
        terms = [self.term()]
        while self.at_word("and"):
            self.advance()
            terms.append(self.term())
        return tuple(terms)

    def term(self):
        start = self.current
        negated = False
        if self.at_word("not"):
            self.advance()
            negated = True
        atom = self.atom()
        duration = None
        if self.at_word("for"):
            self.advance()
            duration = self.literal()
        return Term(atom, negated, duration, self.pos(start))

    def atom(self):
        start = self.current
        if self.at_word("some", "no"):
            quantifier = self.advance().text
            relation = self.name("une relation").text
            self.sym("(")
            first = self.argument()
            self.sym(",")
            second = self.argument()
            self.sym(")")
            return RelationAtom(quantifier, relation, (first, second), self.pos(start))
        var = self.name("une variable").text
        self.sym(".")
        field = self.name("un champ").text
        if self.current.kind == "op":
            op = self.advance().text
            return FieldAtom(var, field, op, self.literal(), self.pos(start))
        return FieldAtom(var, field, None, None, self.pos(start))

    def argument(self):
        if self.current.kind == "ident" and self.current.text == "_":
            return self.advance().text
        return self.name("une variable ou « _ »").text


def parse(source):
    return Parser(source).program()
