"""Contrôle que la suite de conformité est complète et bien formée.

Ne fait tourner aucun moteur : vérifie seulement que chaque cas `.holo` a son
fichier `.attendu.json`, et que celui-ci respecte le format décrit dans le README.

    python verifier_suite.py
"""
import json
import re
import sys
from pathlib import Path

CAS = Path(__file__).resolve().parent / "cas"
CATEGORIES = {
    "code-libre", "unite", "budget", "graine", "bloc-inconnu", "capacite-inconnue", "nom-en-double", "vocabulaire",
    "casse", "niveau-de-titre",
}
PROPRIETES = {
    "meme-fichier-meme-resultat", "lisible-a-plat", "visitable-en-profondeur",
    "graines-enfants-distinctes", "graines-enfants-reproductibles", "lisible-sans-ia",
}
ADR = re.compile(r"^ADR-\d{3}$")


def controler(holo, problemes):
    attendu_path = holo.with_name(holo.stem + ".attendu.json")
    if not attendu_path.exists():
        problemes.append(f"{holo.name} : fichier .attendu.json manquant")
        return
    try:
        attendu = json.loads(attendu_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as erreur:
        problemes.append(f"{attendu_path.name} : JSON invalide ({erreur})")
        return

    def exiger(condition, message):
        if not condition:
            problemes.append(f"{attendu_path.name} : {message}")

    decisions = attendu.get("decisions", [])
    exiger(decisions and all(ADR.match(d) for d in decisions), "« decisions » doit citer au moins une ADR-xxx")

    dans_valides = holo.parent.name == "valides"
    verdict = attendu.get("verdict")
    exiger(verdict == ("accepté" if dans_valides else "refusé"), f"verdict « {verdict} » incohérent avec le dossier {holo.parent.name}")

    if verdict == "accepté":
        exiger(isinstance(attendu.get("arbre"), dict) and "bloc" in attendu.get("arbre", {}), "« arbre » manquant ou sans « bloc »")
        proprietes = attendu.get("proprietes", [])
        exiger(proprietes, "au moins une propriété est attendue")
        exiger(set(proprietes) <= PROPRIETES, f"propriété inconnue : {sorted(set(proprietes) - PROPRIETES)}")
        exiger(("journal" in attendu) == ("scenario" in attendu), "« scenario » et « journal » vont ensemble")
    elif verdict == "refusé":
        exiger(attendu.get("etape") in ("syntaxe", "vérification"), "« etape » doit valoir syntaxe ou vérification")
        exiger(attendu.get("categorie") in CATEGORIES, f"catégorie inconnue : {attendu.get('categorie')}")
        lignes = holo.read_text(encoding="utf-8").splitlines()
        ligne = attendu.get("ligne")
        if not isinstance(ligne, int) or not 1 <= ligne <= len(lignes):
            problemes.append(f"{attendu_path.name} : « ligne » hors du fichier")
        else:
            contenu = lignes[ligne - 1].strip()
            exiger(contenu and not contenu.startswith("//"), f"la ligne {ligne} est vide ou un commentaire")


def main():
    problemes = []
    cas = sorted(CAS.glob("*/*.holo"))
    for holo in cas:
        controler(holo, problemes)
    for orphelin in sorted(CAS.glob("*/*.attendu.json")):
        if not orphelin.with_name(orphelin.name.replace(".attendu.json", ".holo")).exists():
            problemes.append(f"{orphelin.name} : aucun fichier .holo correspondant")

    valides = sum(1 for c in cas if c.parent.name == "valides")
    print(f"{len(cas)} cas : {valides} acceptés, {len(cas) - valides} refusés")
    for probleme in problemes:
        print("PROBLÈME :", probleme)
    if not cas:
        print("PROBLÈME : aucun cas trouvé")
        return 1
    print("Suite bien formée." if not problemes else f"{len(problemes)} problème(s).")
    return 1 if problemes else 0


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.exit(main())
