"""Lexer for HoloCode v0.1."""

from dataclasses import dataclass
import re


@dataclass(frozen=True)
class Token:
    kind: str
    value: str
    offset: int


TOKEN_RE = re.compile(
    r"(?P<SPACE>\s+)"
    r"|(?P<COMMENT>//[^\n]*)"
    r"|(?P<NUMBER>\d+(?:\.\d+)?)"
    r"|(?P<EQ>==)"
    r"|(?P<SYMBOL>[{}(),:;.=<>])"
    r"|(?P<IDENT>[A-Za-z_][A-Za-z0-9_]*)"
)


class LexError(ValueError):
    pass


def tokenize(source: str) -> list[Token]:
    tokens: list[Token] = []
    cursor = 0
    while cursor < len(source):
        match = TOKEN_RE.match(source, cursor)
        if not match:
            excerpt = source[cursor : cursor + 20]
            raise LexError(f"Caractere inattendu a l'offset {cursor}: {excerpt!r}")
        kind = match.lastgroup
        if kind not in {"SPACE", "COMMENT"}:
            tokens.append(Token(kind or "", match.group(), cursor))
        cursor = match.end()
    tokens.append(Token("EOF", "", len(source)))
    return tokens

