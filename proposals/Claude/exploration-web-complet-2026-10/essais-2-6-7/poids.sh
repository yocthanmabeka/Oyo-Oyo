#!/bin/bash
# Octets réellement transférés à l'ouverture (sans geste), mesurés par le navigateur
# (Resource Timing : transferSize), sur le serveur local en Brotli. PC, pas téléphone.
HERE="essais-2-6-7"
REPO=/c/Users/mokea/Documents/IA_creation/Metaverse
export TEMP="$HERE/tmp" TMP="$HERE/tmp"
mkdir -p "$HERE/tmp"
READ='{"method":"Runtime.evaluate","params":{"expression":"(() => { const n = performance.getEntriesByType(\"navigation\")[0]; const r = performance.getEntriesByType(\"resource\"); const total = n.transferSize + r.reduce((s, e) => s + e.transferSize, 0); return JSON.stringify({ total_ko: Math.round(total / 100) / 10, page_ko: Math.round(n.transferSize / 100) / 10, moteur: r.some((e) => /page-engine|holo_engine/.test(e.name)), fichiers: r.map((e) => e.name.split(\"/\").pop() + \" \" + Math.round(e.transferSize / 100) / 10).join(\", \") }); })()"}}'
for page in 62-plis.holo 63-fenetre.holo 64-formulaire.holo 27-donnees.holo 40-langue-et-partage.holo; do
  echo "=== $page, ouvert sans geste, 5 s ==="
  HOLO_GESTURES="[$READ]" node "$REPO/moteur/outils/capture.mjs" "http://localhost:8080/exemples/lecons/$page" "$HERE/poids-$page.png" 1000 700 5000 | grep -v "capture écrite"
done
