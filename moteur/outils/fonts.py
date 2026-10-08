"""Les polices libres du moteur (ADR-092), préparées depuis des paquets npm de Fontsource.

Usage : python3 -I moteur/outils/fonts.py <paquets> <sortie>

Pour chaque paquet de <paquets> (un dossier par paquet, son archive déballée dans
<paquet>/package), copie les fichiers woff2 de l'axe des graisses (droit et italique), ou de
la graisse 400 d'une police fixe, dans <sortie>/<nom>/ ; écrit <sortie>/<nom>/font.css, une
règle @font-face par morceau (unicode-range : le navigateur ne télécharge que les morceaux
dont la page a besoin) ; copie la licence ; et affiche la fiche de chaque police en JSON.
Refuse une police qui n'est pas sous la licence SIL Open Font License 1.1.
"""
import json
import os
import re
import shutil
import sys

downloads, output = sys.argv[1], sys.argv[2]
catalogue = []

FACE = re.compile(r"@font-face\s*\{(.*?)\}", re.S)


def field(block, name):
    match = re.search(rf"{name}\s*:\s*([^;]+);", block)
    return match.group(1).strip() if match else None


for folder in sorted(os.listdir(downloads)):
    package = os.path.join(downloads, folder, "package")
    meta = json.load(open(os.path.join(package, "package.json"), encoding="utf-8"))
    info = json.load(open(os.path.join(package, "metadata.json"), encoding="utf-8"))
    licence = meta.get("license")
    if licence != "OFL-1.1":
        sys.exit(f"{folder} : licence inattendue, {licence}")
    variable = os.path.exists(os.path.join(package, "wght.css"))
    sheets = ["wght.css", "wght-italic.css"] if variable else ["400.css", "400-italic.css"]
    family = info["family"]
    slug = info["id"]
    target = os.path.join(output, slug)
    os.makedirs(target, exist_ok=True)
    faces = []
    size = 0
    for sheet in sheets:
        path = os.path.join(package, sheet)
        if not os.path.exists(path):
            continue
        for block in FACE.findall(open(path, encoding="utf-8").read()):
            source = re.search(r"url\(\./files/([^)]+\.woff2)\)", block)
            if not source:
                sys.exit(f"{folder} : pas de fichier woff2 dans {sheet}")
            file = source.group(1)
            if not re.fullmatch(r"[a-z0-9-]+\.woff2", file):
                sys.exit(f"{folder} : nom de fichier inattendu, {file}")
            shutil.copyfile(os.path.join(package, "files", file), os.path.join(target, file))
            size += os.path.getsize(os.path.join(target, file))
            style = field(block, "font-style") or "normal"
            weight = field(block, "font-weight") or "400"
            ranges = field(block, "unicode-range")
            if style not in ("normal", "italic") or not re.fullmatch(r"\d+( \d+)?", weight):
                sys.exit(f"{folder} : style ou graisse inattendus, {style} {weight}")
            if ranges and not re.fullmatch(r"[U+0-9A-Fa-f?, -]+", ranges):
                sys.exit(f"{folder} : unicode-range inattendu")
            face = f'@font-face{{font-family:"{family}";font-style:{style};font-display:swap;font-weight:{weight};src:url("{file}") format("woff2");'
            if ranges:
                face += f"unicode-range:{ranges};"
            faces.append(face + "}")
    if not faces:
        sys.exit(f"{folder} : aucune police trouvée")
    with open(os.path.join(target, "font.css"), "w", encoding="utf-8") as css:
        css.write(f"/* {family} : {info.get('license', {}).get('type', licence) if isinstance(info.get('license'), dict) else licence}, via Fontsource {meta['version']} (voir LICENSE.txt). */\n")
        css.write("\n".join(faces) + "\n")
    shutil.copyfile(os.path.join(package, "LICENSE"), os.path.join(target, "LICENSE.txt"))
    weights = info.get("weights", [])
    catalogue.append({
        "family": family,
        "slug": slug,
        "category": info.get("category"),
        "subsets": info.get("subsets", []),
        "weights": f"{min(weights)}-{max(weights)}" if variable and weights else ",".join(str(w) for w in weights),
        "italic": "italic" in info.get("styles", []),
        "version": meta["version"],
        "licence": licence,
        "bytes": size,
        "files": len(faces),
    })

print(json.dumps(catalogue, ensure_ascii=False, indent=1))
