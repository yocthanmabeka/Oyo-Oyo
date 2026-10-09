"""Refait docs/01-holocode/TABLEAU-WEB.md depuis la page en ligne du tableau.

La page en ligne, « HoloCode face au web » (https://claude.ai/artifact/PAuybWugSLUEEngQ1PPcPg),
garde les données du tableau dans son code : MOTS (les mots de HoloCode), D (les éléments de
HTML, CSS et JavaScript). C'est elle qu'on change d'abord ; ce fichier relit une copie
enregistrée (par une session Claude, l'outil Artifact en lecture) et écrit le tableau du dépôt,
comptes compris, comptés comme la page les compte : personne ne compte à la main.

    python3 outils/web_table.py page.html docs/01-holocode/TABLEAU-WEB.md --date 2026-10-09
    python3 outils/web_table.py page.html docs/01-holocode/TABLEAU-WEB.md --check

Avec --check, rien n'est écrit : le fichier doit être déjà celui que la page donne.
"""

import argparse
import json
import re
import sys

# Les états d'un élément du web, et ce qu'il faut en faire, tels que le fichier les écrit.
EXISTS = {"oui": "Oui", "partie": "En partie", "non": "Non", "expres": "Refusé exprès", "objet": "Sans objet"}
SHOULD = {"deja": "Déjà là", "urgent": "Oui, en priorité", "utile": "Oui, utile", "tard": "Plus tard", "pas": "Non"}
LANGUAGES = ["HTML", "CSS", "JavaScript"]


def array(page, name):
    """Le tableau JavaScript `const <name> = [ … ];` de la page, relu comme du JSON."""
    start = page.index(f"const {name} = [") + len(f"const {name} = ")
    depth, inside, escaped = 0, False, False
    for end in range(start, len(page)):
        char = page[end]
        if inside:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                inside = False
        elif char == '"':
            inside = True
        elif char == "[":
            depth += 1
        elif char == "]":
            depth -= 1
            if depth == 0:
                break
    text = page[start : end + 1]
    # Les virgules qui ferment une liste en JavaScript (`…],\n  ];`) ne sont pas du JSON.
    cleaned, inside, escaped = [], False, False
    for index, char in enumerate(text):
        if inside:
            cleaned.append(char)
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                inside = False
            continue
        if char == '"':
            inside = True
        elif char == ",":
            rest = text[index + 1 :].lstrip()
            if rest.startswith("]"):
                continue
        cleaned.append(char)
    return json.loads("".join(cleaned))


def words(cell):
    """Le nombre de mots d'une case de la partie 1, compté comme la page en ligne le compte :
    `H1, H2, H3` en compte trois ; une virgule entre parenthèses ne sépare rien
    (`Filter(name:, from:)` est un seul mot)."""
    return len(re.split(r",\s+(?![^(]*\))", cell))


def cell(text):
    """Une case de tableau : une barre verticale y est écrite `\\|`, sinon elle couperait la case."""
    return text.replace("|", "\\|")


def code(text):
    """Une case de code, ou un tiret quand il n'y a rien. Un texte qui contient un accent grave
    est entouré de deux (`` `code` ``), sinon il fermerait le code trop tôt."""
    if text in ("—", ""):
        return "—"
    return cell(f"`` {text} ``" if "`" in text else f"`{text}`")


def summary(words_rows, elements):
    """Les comptes du haut : les mots de HoloCode, puis les éléments du web, langage par langage."""
    decided = sum(words(row[1]) for row in words_rows if row[5] == "decide")
    trying = sum(words(row[1]) for row in words_rows if row[5] != "decide")

    def line(rows):
        count = {state: sum(1 for row in rows if row[5] == state) for state in EXISTS}
        detail = f"{count['oui']} oui, {count['partie']} en partie, {count['non']} non, {count['expres']} refusés"
        if count["objet"]:
            detail += f", {count['objet']} sans objet"
        return len(rows), detail

    lines = []
    for language in LANGUAGES:
        total, detail = line([row for row in elements if row[0] == language])
        lines.append(f"| {language} | {total} éléments | {detail} |")
    total, detail = line(elements)
    lines.append(f"| HTML, CSS, JS ensemble | {total} éléments | {detail} |")
    return decided, trying, lines


def table(page_text, date):
    words_rows, elements = array(page_text, "MOTS"), array(page_text, "D")
    decided, trying, lines = summary(words_rows, elements)
    total = decided + trying
    out = [
        "# HoloCode, et HTML, CSS, JavaScript : le grand tableau",
        "",
        f"- Relevé de Claude, tenu à jour à chaque changement du langage (dernier : {date}). La même chose, à filtrer, sur la page en ligne tenue à jour pour Yocthan.",
        f"- D’abord **tous les mots de HoloCode** ({total} mots : {decided} décidés, {trying} à l’essai), puis **chaque élément de HTML, CSS et JavaScript** ({len(elements)}) et ce que HoloCode en a.",
        "- **Existe ?** : le jugement de Claude, élément par élément (oui, en partie, non) ; ce n’est pas une mesure. Le tableau ne donne pas de pourcentage : aucune méthode reproductible ne mesure la part d’un élément du web qu’on obtient en HoloCode (consigne de Yocthan du 2026-10-07). Les comptes se refont en comptant les lignes.",
        "- Les refus sont expliqués dans [`proposals/Claude/pourquoi-ces-refus-2026-10/`](../../proposals/Claude/pourquoi-ces-refus-2026-10/README.md).",
        "",
        "## Résumé",
        "",
        "| | Mesure | Détail |",
        "|---|---|---|",
        f"| **HoloCode** | {total} mots | {decided} décidés, {trying} à l’essai |",
        *lines,
        "",
        "# Partie 1 — Les mots de HoloCode",
    ]
    # Les mots, par sorte, dans l'ordre de la page : une sorte revient quand ses mots sont venus plus tard.
    previous = None
    for kind, word, does, web, decision, state in words_rows:
        if kind != previous:
            out += ["", f"## {kind}", "", "| Mot HoloCode | Ce qu’il fait | Sur le web | État |", "|---|---|---|---|"]
            previous = kind
        status = "Décidé" if state == "decide" else "À l’essai"
        out.append(f"| {code(word)} | {cell(does)} | {code(web)} | {status} ({decision}) |")
    out += ["", "# Partie 2 — HoloCode face à HTML, CSS et JavaScript"]
    previous = None
    for language, group, element, role, holo, exists, _, should, why in elements:
        if (language, group) != previous:
            out += ["", f"## {language} — {group}", "", "| En HoloCode | Élément du web | Rôle | Existe ? | Doit exister ? | Pourquoi |", "|---|---|---|---|---|---|"]
            previous = (language, group)
        out.append(f"| {code(holo)} | {code(element)} | {cell(role)} | {EXISTS[exists]} | {SHOULD[should]} | {cell(why) or '—'} |")
    return "\n".join(out) + "\n"


def main():
    parser = argparse.ArgumentParser(description="Refait TABLEAU-WEB.md depuis la page en ligne du tableau.")
    parser.add_argument("page", help="la page en ligne, enregistrée (HTML)")
    parser.add_argument("table", help="docs/01-holocode/TABLEAU-WEB.md")
    parser.add_argument("--date", help="la date du relevé (AAAA-MM-JJ) ; sans elle, celle du fichier")
    parser.add_argument("--check", action="store_true", help="vérifier sans écrire")
    arguments = parser.parse_args()
    page = open(arguments.page, encoding="utf-8").read()
    try:
        current = open(arguments.table, encoding="utf-8").read()
    except FileNotFoundError:
        current = ""
    date = arguments.date or (current.split("(dernier : ", 1)[1][:10] if "(dernier : " in current else "")
    fresh = table(page, date)
    if arguments.check:
        if fresh != current:
            sys.exit("TABLEAU-WEB.md n'est pas celui que donne la page en ligne : relance sans --check.")
        print("TABLEAU-WEB.md est celui que donne la page en ligne.")
        return
    open(arguments.table, "w", encoding="utf-8").write(fresh)
    print(f"TABLEAU-WEB.md refait : {fresh.count(chr(10))} lignes.")


if __name__ == "__main__":
    main()
