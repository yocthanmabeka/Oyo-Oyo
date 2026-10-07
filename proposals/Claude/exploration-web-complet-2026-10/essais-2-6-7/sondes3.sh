#!/bin/bash
# Troisième série : comparer deux textes, et ce que voit la page au départ. Lecture seule.
H=/c/Users/mokea/Documents/IA_creation/Metaverse/moteur/target/release/holo.exe
here="$(dirname "$0")"
probe() {
  local label="$1"; local src="$2"
  printf '%s\n  → ' "$label"
  printf '%s' "$src" | "$H" check - 2>&1 | head -c 400
  echo
}
html() {
  local label="$1"; local src="$2"; local pattern="$3"
  printf '%s\n  → ' "$label"
  printf '%s' "$src" > "$here/sonde-html.holo"
  MSYS_NO_PATHCONV=1 "$H" html "$here/sonde-html.holo" /essai/ 2>&1 | grep -o "$pattern" | head -6 | tr '\n' ' '
  echo
}
probe "P2-18 comparer deux textes : If(size, is: medium)" 'Page(title: "a", state: State(size: "S", medium: "M"), children: [ H1("a"), Choice(value: size, label: "Size", options: ["S", "M"]), If(size, is: medium, children: [ P("Medium chosen") ]) ])'
html "P2-18b au départ size = S, medium = M : le bloc est-il caché ?" 'Page(title: "a", state: State(size: "S", medium: "M"), children: [ H1("a"), Choice(value: size, label: "Size", options: ["S", "M"]), If(size, is: medium, children: [ P("Medium chosen") ]) ])' 'data-if="[^"]*"[^>]*>'
probe "P2-19 champ de nombre : valeur négative impossible (min=0)" 'Page(title: "a", state: State(n: 0), children: [ H1("a"), Input(value: n, label: "Temperature") ])'
