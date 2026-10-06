"""Fabrique le paquet de l'extension VS Code, sans rien installer d'autre que Python.

    python outils/vscode-holocode/empaqueter.py
    code --install-extension outils/vscode-holocode/holocode-0.2.0.vsix

Un fichier .vsix est une archive zip : l'extension dans un dossier « extension », plus deux
fichiers de description que VS Code attend.
"""
import json
import zipfile
from pathlib import Path

ICI = Path(__file__).resolve().parent
# La correction d'un clic est la même que dans l'éditeur du navigateur (ADR-046) : on en met une
# copie dans le paquet.
CORRECTIONS = ICI.parent.parent / "moteur" / "web" / "corrections.js"
FICHIERS = ["package.json", "extension.js", "language-configuration.json", "syntaxes/holo.tmLanguage.json", "README.md"]

paquet = json.loads((ICI / "package.json").read_text(encoding="utf-8"))
identite = f'Id="{paquet["name"]}" Version="{paquet["version"]}" Publisher="{paquet["publisher"]}"'

MANIFESTE = f"""<?xml version="1.0" encoding="utf-8"?>
<PackageManifest Version="2.0.0" xmlns="http://schemas.microsoft.com/developer/vsx-schema/2011">
  <Metadata>
    <Identity Language="en-US" {identite} />
    <DisplayName>{paquet["displayName"]}</DisplayName>
    <Description xml:space="preserve">{paquet["description"]}</Description>
    <Categories>Programming Languages</Categories>
    <Properties>
      <Property Id="Microsoft.VisualStudio.Code.Engine" Value="{paquet["engines"]["vscode"]}" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionKind" Value="workspace" />
    </Properties>
  </Metadata>
  <Installation>
    <InstallationTarget Id="Microsoft.VisualStudio.Code" />
  </Installation>
  <Dependencies />
  <Assets>
    <Asset Type="Microsoft.VisualStudio.Code.Manifest" Path="extension/package.json" Addressable="true" />
  </Assets>
</PackageManifest>
"""

TYPES = """<?xml version="1.0" encoding="utf-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension=".json" ContentType="application/json" />
  <Default Extension=".js" ContentType="application/javascript" />
  <Default Extension=".mjs" ContentType="application/javascript" />
  <Default Extension=".md" ContentType="text/markdown" />
  <Default Extension=".vsixmanifest" ContentType="text/xml" />
</Types>
"""

sortie = ICI / f'{paquet["name"]}-{paquet["version"]}.vsix'
with zipfile.ZipFile(sortie, "w", zipfile.ZIP_DEFLATED) as archive:
    archive.writestr("extension.vsixmanifest", MANIFESTE)
    archive.writestr("[Content_Types].xml", TYPES)
    for fichier in FICHIERS:
        json.loads((ICI / fichier).read_text(encoding="utf-8")) if fichier.endswith(".json") else None
        archive.write(ICI / fichier, f"extension/{fichier}")
    archive.writestr("extension/corrections.mjs", CORRECTIONS.read_text(encoding="utf-8"))
print(f"paquet écrit : {sortie}")
