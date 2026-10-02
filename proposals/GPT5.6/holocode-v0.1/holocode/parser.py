"""Recursive-descent parser for HoloCode v0.1."""

from typing import Any

from .ast import Condition, Effect, EntityDecl, PhenomenonDecl, RelationDecl, Vector3, WorldDecl
from .lexer import Token, tokenize


class ParseError(ValueError):
    pass


class Parser:
    def __init__(self, source: str):
        self.tokens = tokenize(source)
        self.index = 0

    @property
    def current(self) -> Token:
        return self.tokens[self.index]

    def advance(self) -> Token:
        token = self.current
        self.index += 1
        return token

    def accept(self, value: str) -> bool:
        if self.current.value == value:
            self.advance()
            return True
        return False

    def expect(self, value: str) -> Token:
        if self.current.value != value:
            raise ParseError(
                f"Attendu {value!r}, recu {self.current.value!r} "
                f"a l'offset {self.current.offset}"
            )
        return self.advance()

    def identifier(self) -> str:
        if self.current.kind != "IDENT":
            raise ParseError(f"Identifiant attendu a l'offset {self.current.offset}")
        return self.advance().value

    def number(self) -> float:
        if self.current.kind != "NUMBER":
            raise ParseError(f"Nombre attendu a l'offset {self.current.offset}")
        return float(self.advance().value)

    def parse(self) -> WorldDecl:
        self.expect("world")
        world = WorldDecl(self.identifier())
        self.expect("{")
        while not self.accept("}"):
            if self.current.value == "entity":
                entity = self.parse_entity()
                if entity.name in world.entities:
                    raise ParseError(f"Entite dupliquee: {entity.name}")
                world.entities[entity.name] = entity
            elif self.current.value == "relation":
                relation = self.parse_relation()
                if relation.name in world.relations:
                    raise ParseError(f"Relation dupliquee: {relation.name}")
                world.relations[relation.name] = relation
            elif self.current.value == "phenomenon":
                world.phenomena.append(self.parse_phenomenon())
            else:
                raise ParseError(
                    f"Declaration inconnue {self.current.value!r} "
                    f"a l'offset {self.current.offset}"
                )
        if self.current.kind != "EOF":
            raise ParseError(f"Contenu apres la fin du monde a l'offset {self.current.offset}")
        return world

    def parse_entity(self) -> EntityDecl:
        self.expect("entity")
        entity = EntityDecl(self.identifier())
        self.expect("{")
        while not self.accept("}"):
            key = self.identifier()
            self.expect(":")
            entity.properties[key] = self.value()
            self.accept(";")
        return entity

    def parse_relation(self) -> RelationDecl:
        self.expect("relation")
        name = self.identifier()
        self.expect("(")
        left = self.identifier()
        self.expect(",")
        right = self.identifier()
        self.expect(")")
        self.expect("{")
        self.expect("distance")
        self.expect("<")
        distance = self.number()
        unit = self.identifier()
        if unit != "m":
            raise ParseError("HoloCode v0.1 accepte uniquement l'unite 'm'")
        self.accept(";")
        self.expect("}")
        return RelationDecl(name, left, right, distance, unit)

    def parse_phenomenon(self) -> PhenomenonDecl:
        self.expect("phenomenon")
        phenomenon = PhenomenonDecl(self.identifier())
        self.expect("{")
        self.expect("when")
        phenomenon.conditions.append(self.condition())
        while self.accept("and"):
            phenomenon.conditions.append(self.condition())
        self.accept(";")
        self.expect("effect")
        phenomenon.effects.append(self.effect())
        while self.accept(","):
            phenomenon.effects.append(self.effect())
        self.accept(";")
        self.expect("}")
        return phenomenon

    def condition(self) -> Condition:
        subject = self.identifier()
        if self.accept("."):
            property_name = self.identifier()
            self.expect("==")
            return Condition("property", subject, property_name, self.value())
        self.expect("active")
        return Condition("relation", subject)

    def effect(self) -> Effect:
        entity = self.identifier()
        self.expect(".")
        property_name = self.identifier()
        self.expect("=")
        return Effect(entity, property_name, self.value())

    def value(self) -> Any:
        if self.accept("("):
            values = []
            for index in range(3):
                coordinate = self.number()
                unit = self.identifier()
                if unit != "m":
                    raise ParseError("Les positions v0.1 doivent etre exprimees en metres")
                values.append(coordinate)
                if index < 2:
                    self.expect(",")
            self.expect(")")
            return Vector3(*values)
        if self.current.kind == "NUMBER":
            return self.number()
        if self.current.value in {"true", "false"}:
            return self.advance().value == "true"
        return self.identifier()


def parse(source: str) -> WorldDecl:
    return Parser(source).parse()

