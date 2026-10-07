#!/bin/bash
# Deuxième série de sondes (pistes 2, 6, 7). Lecture seule.
H=/c/Users/mokea/Documents/IA_creation/Metaverse/moteur/target/release/holo.exe
probe() {
  local label="$1"; local src="$2"
  printf '%s\n  → ' "$label"
  printf '%s' "$src" | "$H" check - 2>&1 | head -c 400
  echo
}
html() {
  local label="$1"; local src="$2"; local pattern="$3"
  printf '%s\n  → ' "$label"
  local here="$(dirname "$0")"
  printf '%s' "$src" > "$here/sonde-html.holo"
  MSYS_NO_PATHCONV=1 "$H" html "$here/sonde-html.holo" /essai/ 2>&1 | grep -o "$pattern" | head -6 | tr '\n' ' '
  echo
}

probe "P2-15 If sur un texte de Choice : If(size, is: \"M\")" 'Page(title: "a", state: State(size: ""), children: [ H1("a"), Choice(value: size, label: "Size", options: ["S", "M"]), If(size, is: "M", children: [ P("Medium") ]) ])'
probe "P6-15 menu.toggle(1)" 'Page(title: "a", state: State(menu: 0), children: [ H1("a"), Button(name: MenuButton, text: "Menu") ], rules: [ On(MenuButton.tap, effect: menu.toggle(1)) ])'
probe "P6-16 un menu déroulant sans moteur : Details dans Nav" 'Page(title: "a", children: [ Header(children: [ Nav(children: [ Details(summary: "Menu", children: [ List(children: [ A("Home", to: "home.holo"), A("Shop", to: "shop.holo") ]) ]) ]) ]), H1("a") ])'
probe "P6-17 un formulaire dans une fenêtre" 'Page(title: "a", state: State(mail: ""), children: [ H1("a"), Button(name: Ask, text: "Subscribe"), Dialog(name: Sub, children: [ H2("Subscribe"), Form(name: F, children: [ Input(value: mail, label: "E-mail"), Button(name: Send, text: "Send") ]) ]) ], rules: [ On(Ask.tap, effect: Sub.open), On(Send.tap, effect: F.send), On(F.sent, effect: Sub.close) ])'
probe "P6-18 Key.escape et fenêtre ouverte (accepté)" 'Page(title: "a", state: State(n: 0), children: [ H1("a"), Dialog(name: D, children: [ P("x") ]) ], rules: [ On(Key.escape, effect: n.add(1)) ])'
probe "P6-19 Input dans un Repeat(over:) (un champ par ligne)" 'Page(title: "a", state: State(items: [ Item(title: "A", qty: 1) ]), children: [ H1("a"), Repeat(over: items, children: [ Input(value: item.qty, label: "Quantity") ]) ])'
probe "P2-16 un champ nombre dans un formulaire (quantité commandée)" 'Page(title: "a", state: State(qty: 1, name: ""), prices: Prices(qty: 120), children: [ H1("a"), Form(name: Order, children: [ Input(value: qty, label: "How many paintings?", max: 10), Input(value: name, label: "Your name"), Button(name: Buy, text: "Order") ]), P("{total} euros") ], rules: [ On(Buy.tap, effect: Order.send) ])'
probe "P7-16 Main nommé, visé par un lien d évitement" 'Page(title: "a", children: [ Header(children: [ A("Skip to content", to: "#Content"), Nav(children: [ A("Home", to: "home.holo") ]) ]), Main(name: Content, children: [ H1("a"), P("text") ]) ])'
html "P7-16b le même : l id Content est-il posé dans le HTML ?" 'Page(title: "a", children: [ Header(children: [ A("Skip to content", to: "#Content"), Nav(children: [ A("Home", to: "home.holo") ]) ]), Main(name: Content, children: [ H1("a"), P("text") ]) ])' 'id="[^"]*"\|<main[^>]*>\|href="[^"]*"'
html "P7-17 lien vers un H2 nommé : id posé ?" 'Page(title: "a", children: [ A("Hours", to: "#Hours"), H1("a"), H2("Hours", name: Hours) ])' 'id="[^"]*"\|href="[^"]*"'
html "P7-18 deux Nav : quels attributs ?" 'Page(title: "a", children: [ Header(children: [ Nav(children: [ A("Home", to: "home.holo") ]) ]), H1("a"), Footer(children: [ Nav(children: [ A("Legal", to: "legal.holo") ]) ]) ])' '<nav[^>]*>\|<header[^>]*>\|<footer[^>]*>'
html "P7-19 Dialog : nom accessible ?" 'Page(title: "a", lang: "en", children: [ H1("a"), Dialog(name: Thanks, children: [ H2("Thank you"), P("It arrived.") ]) ])' '<dialog[^>]*>\|aria-label="[^"]*"'
html "P2-17 Form rendu : attributs du formulaire et des champs" 'Page(title: "a", state: State(name: "", n: 0), children: [ H1("a"), Form(name: Contact, children: [ Input(value: name, label: "Your name"), Input(value: n, label: "How many", max: 5), Button(name: Send, text: "Send") ]) ], rules: [ On(Send.tap, effect: Contact.send) ])' '<form[^>]*>\|<input[^>]*>\|<button[^>]*>'
html "P7-20 tableau : th et scope" 'Page(title: "a", children: [ H1("a"), Table(caption: "Hours", head: ["Day", "Hours"], rows: [ ["Mon", "9-18"] ]) ])' '<th[^>]*>\|<td[^>]*>\|<caption>'
html "P7-21 page sans titre : data-title" 'Page(children: [ H1("a") ])' 'data-title="[^"]*"'
html "P7-22 A externe : attributs" 'Page(title: "a", children: [ H1("a"), A("Elsewhere", to: "https://example.com") ])' '<a [^>]*>'
