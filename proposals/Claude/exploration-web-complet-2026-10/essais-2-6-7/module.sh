#!/bin/bash
# Hors des pistes 2, 6, 7 : le module enfermé de la leçon 69 rend-il encore son nombre ?
# Lecture seule : l'outil de capture du dépôt, sur le serveur local (8080).
HERE="essais-2-6-7"
REPO=/c/Users/mokea/Documents/IA_creation/Metaverse
export TEMP="$HERE/tmp" TMP="$HERE/tmp"
mkdir -p "$HERE/tmp"
TAP='{"method":"Runtime.evaluate","params":{"expression":"document.querySelector(\"[data-name=Calculer]\").click(); \"touché\""},"waiting":3000}'
READ='{"method":"Runtime.evaluate","params":{"expression":"JSON.stringify({modules: window.__holoModules ?? null, texte: [...document.querySelectorAll(\".holo-P\")].map((p) => p.textContent).join(\" | \").slice(0, 300)})"}}'
HOLO_GESTURES="[$TAP,$READ]" node "$REPO/moteur/outils/capture.mjs" "http://localhost:8080/exemples/lecons/69-module-enferme.holo?values" "$HERE/module-69.png" 1000 800 5000
