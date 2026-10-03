"""Exporte les sessions Claude Code d'un projet en transcriptions Markdown.

Les sessions Claude Code (extension VS Code ou terminal) ne possèdent pas
d'URL de partage : elles sont enregistrées localement au format JSONL. Cet
outil en extrait uniquement les messages de l'utilisateur et les réponses
textuelles de Claude, afin que le dépôt reste la source commune (ADR-001).

Sont retirés : appels et résultats d'outils, réflexions internes, rappels
système, images, adresses e-mail et nom d'utilisateur de la machine.

Usage, depuis la racine du dépôt :

    python outils/exporter_sessions_claude.py

Le fichier docs/05-discussions/transcriptions/sessions.json associe le début
de l'identifiant de session à son identifiant HC-xxx. Une session absente de
ce fichier est signalée et n'est pas exportée.
"""
import argparse
import getpass
import json
import re
from pathlib import Path

RACINE = Path(__file__).resolve().parent.parent
SORTIE = RACINE / "docs" / "05-discussions" / "transcriptions"

BALISES = re.compile(
    r"<(system-reminder|ide_selection|ide_opened_file|local-command-stdout|"
    r"local-command-caveat|command-name|command-message|command-args|"
    r"task-notification)>.*?</\1>",
    re.S,
)
EMAIL = re.compile(r"[\w.+-]+@[\w-]+\.[\w.-]+")
UTILISATEUR = re.compile(re.escape(getpass.getuser()), re.I)
CLOTURE = re.compile(r"^\s*(```|~~~)")
TITRE = re.compile(r"^(#{1,4})\s")


def dossier_sessions() -> Path:
    """Dossier où Claude Code range les sessions de ce dépôt."""
    nom = re.sub(r"[^A-Za-z0-9]", "-", str(RACINE))
    return Path.home() / ".claude" / "projects" / nom


def nettoyer(texte: str) -> str:
    texte = BALISES.sub("", texte)
    texte = EMAIL.sub("[e-mail retiré]", texte)
    texte = UTILISATEUR.sub("[utilisateur]", texte)
    return texte.strip()


def abaisser_titres(texte: str) -> str:
    """Décale les titres de deux niveaux, hors blocs de code, pour qu'ils
    restent sous le titre du tour de parole."""
    lignes, dans_code = [], False
    for ligne in texte.split("\n"):
        if CLOTURE.match(ligne):
            dans_code = not dans_code
        elif not dans_code and TITRE.match(ligne):
            ligne = "##" + ligne
        lignes.append(ligne)
    return "\n".join(lignes)


def texte_du_message(contenu, role):
    if isinstance(contenu, str):
        return nettoyer(contenu), 0
    parties, images = [], 0
    for bloc in contenu or []:
        if bloc.get("type") == "text":
            parties.append(bloc.get("text", ""))
        elif bloc.get("type") == "image" and role == "user":
            images += 1
    return nettoyer("\n\n".join(parties)), images


def lire_session(chemin: Path):
    tours = []
    for ligne in chemin.read_text(encoding="utf-8").splitlines():
        try:
            rec = json.loads(ligne)
        except json.JSONDecodeError:
            continue
        if rec.get("type") not in ("user", "assistant"):
            continue
        if rec.get("isMeta") or rec.get("isSidechain") or rec.get("isCompactSummary"):
            continue
        role = rec["type"]
        texte, images = texte_du_message(rec.get("message", {}).get("content"), role)
        if images:
            texte = f"{texte}\n\n*[{images} image(s) jointe(s), non exportée(s)]*".strip()
        if not texte:
            continue
        horodatage = (rec.get("timestamp") or "")[:16].replace("T", " ")
        if tours and tours[-1][0] == role:
            tours[-1] = (role, tours[-1][1], tours[-1][2] + "\n\n" + texte)
        else:
            tours.append((role, horodatage, texte))
    return tours


def ecrire(hc: str, session: str, tours, sortie: Path) -> Path:
    cible = sortie / f"{hc}.md"
    with cible.open("w", encoding="utf-8", newline="\n") as f:
        f.write(f"# {hc} — Transcription\n\n")
        f.write(
            "> Fichier généré par `outils/exporter_sessions_claude.py` ; ne pas le modifier à la main.\n"
            f"> Source : session Claude Code `{session}`, sans URL de partage.\n"
            "> Seuls les messages de Yocthan et les réponses textuelles de Claude sont conservés.\n"
            "> Les liens internes des réponses sont relatifs à la racine du dépôt tel qu'il était\n"
            "> pendant la session ; certains peuvent ne plus exister. Heures en UTC.\n\n"
        )
        for role, horodatage, texte in tours:
            qui = "Yocthan" if role == "user" else "Claude"
            f.write(f"## {qui} — {horodatage}\n\n{abaisser_titres(texte)}\n\n")
    return cible


def main():
    analyseur = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    analyseur.add_argument("--source", type=Path, default=dossier_sessions())
    analyseur.add_argument("--sortie", type=Path, default=SORTIE)
    args = analyseur.parse_args()

    correspondance = json.loads((args.sortie / "sessions.json").read_text(encoding="utf-8"))
    for chemin in sorted(args.source.glob("*.jsonl")):
        hc = next((v for k, v in correspondance.items() if chemin.stem.startswith(k)), None)
        if hc is None:
            print(f"IGNORÉE  {chemin.stem} : absente de sessions.json")
            continue
        tours = lire_session(chemin)
        cible = ecrire(hc, chemin.stem, tours, args.sortie)
        print(f"{hc}  {len(tours):3d} tours  ->  {cible.relative_to(RACINE)}")


if __name__ == "__main__":
    main()
