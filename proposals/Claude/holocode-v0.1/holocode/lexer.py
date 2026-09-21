"""Analyse lexicale de HoloCode v0.1."""

import re
from dataclasses import dataclass

from .units import dimension


class HoloSyntaxError(Exception):
    def __init__(self, message, line, col):
        super().__init__(f"ligne {line}, colonne {col} : {message}")
        self.line = line
        self.col = col


@dataclass(frozen=True)
class Token:
    kind: str  # ident, number, string, op, sym, eof
    text: str
    line: int
    col: int
    value: float | None = None  # nombre converti en unité SI
    type: str | None = None  # Number, Length ou Duration


TOKEN = re.compile(
    r"""
      (?P<space>[ \t\r]+)
    | (?P<newline>\n)
    | (?P<comment>//[^\n]*)
    | (?P<number>-?\d+(?:\.\d+)?)(?P<unit>[A-Za-z]+)?
    | (?P<ident>[A-Za-z_][A-Za-z0-9_]*)
    | (?P<string>"[^"\n]*")
    | (?P<op><=|>=|==|!=|<|>)
    | (?P<sym>[{}(),:.=+])
    """,
    re.X,
)


def tokenize(source):
    tokens, pos, line, line_start = [], 0, 1, 0
    while pos < len(source):
        col = pos - line_start + 1
        match = TOKEN.match(source, pos)
        if match is None:
            raise HoloSyntaxError(f"caractère inattendu « {source[pos]} »", line, col)
        pos = match.end()
        kind = match.lastgroup
        if kind == "unit":  # le dernier groupe d'un nombre suffixé est son unité
            kind = "number"
        if kind == "newline":
            line, line_start = line + 1, pos
        elif kind == "number":
            found = dimension(match["unit"])
            if found is None:
                raise HoloSyntaxError(f"unité inconnue « {match['unit']} »", line, col)
            type_, factor = found
            tokens.append(Token("number", match.group(), line, col, float(match["number"]) * factor, type_))
        elif kind not in ("space", "comment"):
            tokens.append(Token(kind, match.group(), line, col))
    tokens.append(Token("eof", "", line, len(source) - line_start + 1))
    return tokens
