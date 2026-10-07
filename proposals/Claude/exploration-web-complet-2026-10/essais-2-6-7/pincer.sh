#!/bin/bash
# Piste 6 : pincer à deux doigts, dans un Chrome sans fenêtre, sur le serveur local (8080).
# Lecture seule : l'outil de capture du dépôt est lancé tel quel ; son profil Chrome et les
# images vont dans le dossier d'essai (TEMP et TMP y pointent).
HERE="essais-2-6-7"
REPO=/c/Users/mokea/Documents/IA_creation/Metaverse
export TEMP="$HERE/tmp" TMP="$HERE/tmp"
mkdir -p "$HERE/tmp"

COLLECT='{"method":"Runtime.evaluate","params":{"expression":"window.__errs=[];addEventListener(\"error\",(e)=>__errs.push(e.message));\"collecteur prêt\""}}'
TOUCH_ON='{"method":"Emulation.setTouchEmulationEnabled","params":{"enabled":true,"maxTouchPoints":5}}'
START='{"method":"Input.dispatchTouchEvent","params":{"type":"touchStart","touchPoints":[{"x":450,"y":300,"id":0},{"x":550,"y":300,"id":1}]}}'
MOVE1='{"method":"Input.dispatchTouchEvent","params":{"type":"touchMove","touchPoints":[{"x":350,"y":300,"id":0},{"x":650,"y":300,"id":1}]}}'
MOVE2='{"method":"Input.dispatchTouchEvent","params":{"type":"touchMove","touchPoints":[{"x":250,"y":300,"id":0},{"x":750,"y":300,"id":1}]}}'
END='{"method":"Input.dispatchTouchEvent","params":{"type":"touchEnd","touchPoints":[]}}'
WHEEL='{"method":"Input.dispatchMouseEvent","params":{"type":"mouseWheel","x":500,"y":300,"deltaX":0,"deltaY":-400,"modifiers":2}}'
STATE_ENGINE='{"method":"Runtime.evaluate","params":{"expression":"JSON.stringify({erreurs: window.__errs, moteur: !!document.querySelector(\"script[src*=page-engine]\"), zoom: document.querySelector(\".holo-Page\")?.style.transform || \"(aucun)\"})"},"waiting":1500}'

echo "=== A. Leçon 1 (page légère, sans moteur) : pincer, puis Ctrl + molette ==="
HOLO_GESTURES="[$COLLECT,$TOUCH_ON,$START,$MOVE1,$MOVE2,$END,$STATE_ENGINE,$WHEEL,$STATE_ENGINE]" \
  node "$REPO/moteur/outils/capture.mjs" "http://localhost:8080/exemples/lecons/01-page.holo" "$HERE/pincer-01.png" 1000 700 2500

echo "=== B. Leçon 9 (moteur chargé par ?values) : pincer, puis Ctrl + molette ==="
HOLO_GESTURES="[$COLLECT,$TOUCH_ON,$START,$MOVE1,$MOVE2,$END,$STATE_ENGINE,$WHEEL,$STATE_ENGINE]" \
  node "$REPO/moteur/outils/capture.mjs" "http://localhost:8080/exemples/lecons/09-zoom-et-points.holo?values" "$HERE/pincer-09.png" 1000 700 5000
