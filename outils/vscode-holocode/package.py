"""Fabrique le manifest de l'extension VS Code, sans rien installer d'autre que Python.

    python outils/vscode-holocode/package.py
    code --install-extension outils/vscode-holocode/holocode-0.2.0.vsix

Un file .vsix est une archive zip : l'extension dans un dossier « extension », plus deux
fichiers de description que VS Code attend.
"""
import json
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
# La correction d'un clic est la même que dans l'éditeur du navigateur (ADR-046) : on en met une
# copie dans le manifest.
FIXES = HERE.parent.parent / "moteur" / "web" / "fixes.js"
FILES = ["package.json", "extension.js", "language-configuration.json", "syntaxes/holo.tmLanguage.json", "README.md"]

manifest = json.loads((HERE / "package.json").read_text(encoding="utf-8"))
identity = f'Id="{manifest["name"]}" Version="{manifest["version"]}" Publisher="{manifest["publisher"]}"'

MANIFEST = f"""<?xml version="1.0" encoding="utf-8"?>
<PackageManifest Version="2.0.0" xmlns="http://schemas.microsoft.com/developer/vsx-schema/2011">
  <Metadata>
    <Identity Language="en-US" {identity} />
    <DisplayName>{manifest["displayName"]}</DisplayName>
    <Description xml:space="preserve">{manifest["description"]}</Description>
    <Categories>Programming Languages</Categories>
    <Properties>
      <Property Id="Microsoft.VisualStudio.Code.Engine" Value="{manifest["engines"]["vscode"]}" />
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

output = HERE / f'{manifest["name"]}-{manifest["version"]}.vsix'
with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as archive:
    archive.writestr("extension.vsixmanifest", MANIFEST)
    archive.writestr("[Content_Types].xml", TYPES)
    for file in FILES:
        json.loads((HERE / file).read_text(encoding="utf-8")) if file.endswith(".json") else None
        archive.write(HERE / file, f"extension/{file}")
    archive.writestr("extension/fixes.mjs", FIXES.read_text(encoding="utf-8"))
print(f"manifest écrit : {output}")
