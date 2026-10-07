#!/bin/bash
# Sondes de l'exploration (pistes 2, 6, 7) : chaque cas est envoyé à `holo check -`.
# Lecture seule : rien n'est écrit dans le dépôt.
H=/c/Users/mokea/Documents/IA_creation/Metaverse/moteur/target/release/holo.exe
probe() {
  local label="$1"; local src="$2"
  printf '%s\n  → ' "$label"
  printf '%s' "$src" | "$H" check - 2>&1 | head -c 400
  echo
}

# --- Piste 2 : formulaires ---
probe "P2-01 attente, erreur, confirmation avec l'écriture existante" 'Page(title: "Contact", state: State(name: "", message: "", sending: 0, sent: 0, failed: 0), children: [
  H1("Contact"),
  Form(name: Contact, children: [
    Input(value: name, label: "Your name"),
    Input(value: message, label: "Your message", lines: 4),
    If(sending, is: 0, children: [ Button(name: Send, text: "Send") ], else: [ Text("Sending…") ]),
  ]),
  If(sent, is: 1, children: [ P("Thank you, it arrived.") ]),
  If(failed, is: 1, children: [ P("It did not leave. Try again.") ]),
], rules: [
  On(Send.tap, effect: [sending.set(1), failed.set(0), Contact.send]),
  On(Contact.sent, effect: [sending.set(0), sent.set(1)]),
  On(Contact.failed, effect: [sending.set(0), failed.set(1)]),
])'
probe "P2-02 champ obligatoire : required" 'Page(title: "a", state: State(name: ""), children: [ H1("a"), Form(name: F, children: [ Input(value: name, label: "Name", required: true) ]) ])'
probe "P2-03 courriel : type email" 'Page(title: "a", state: State(mail: ""), children: [ H1("a"), Form(name: F, children: [ Input(value: mail, label: "E-mail", type: email) ]) ])'
probe "P2-04 longueur minimale : min sur un texte" 'Page(title: "a", state: State(name: ""), children: [ H1("a"), Form(name: F, children: [ Input(value: name, label: "Name", min: 2) ]) ])'
probe "P2-05 aide sous un champ : hint" 'Page(title: "a", state: State(name: ""), children: [ H1("a"), Form(name: F, children: [ Input(value: name, label: "Name", hint: "As on your card") ]) ])'
probe "P2-06 signal submit (Entrée dans le formulaire)" 'Page(title: "a", state: State(name: ""), children: [ H1("a"), Form(name: F, children: [ Input(value: name, label: "Name") ]) ], rules: [ On(F.submit, effect: F.send) ])'
probe "P2-07 Key.enter qui envoie (accepté, mais ignoré dans un champ)" 'Page(title: "a", state: State(name: ""), children: [ H1("a"), Form(name: F, children: [ Input(value: name, label: "Name") ]) ], rules: [ On(Key.enter, effect: F.send) ])'
probe "P2-08 une valeur du panier dans l envoi : Form(send:)" 'Page(title: "a", state: State(cart: 2, name: ""), children: [ H1("a"), Form(name: F, send: [cart], children: [ Input(value: name, label: "Name") ]) ])'
probe "P2-09 une liste dans un formulaire : Repeat(over:) dans Form" 'Page(title: "a", state: State(name: "", items: [ Item(title: "Sunrise", qty: 1) ]), children: [ H1("a"), Form(name: F, children: [ Input(value: name, label: "Name"), Repeat(over: items, children: [ Text("{item.title} x {item.qty}") ]) ]) ])'
probe "P2-10 vider le formulaire : F.reset" 'Page(title: "a", state: State(name: ""), children: [ H1("a"), Form(name: F, children: [ Input(value: name, label: "Name"), Button(name: Clear, text: "Clear") ]) ], rules: [ On(Clear.tap, effect: F.reset) ])'
probe "P2-11 Choice requis sans valeur : options vides" 'Page(title: "a", state: State(size: ""), children: [ H1("a"), Form(name: F, children: [ Choice(value: size, label: "Size", options: []) ]) ])'
probe "P2-12 date bornée : Input(type: date, min:)" 'Page(title: "a", state: State(day: ""), children: [ H1("a"), Form(name: F, children: [ Input(value: day, label: "Day", type: date, min: "2026-10-01") ]) ])'
probe "P2-13 deux conditions ensemble (ET) par des If emboîtés" 'Page(title: "a", state: State(name: "", message: ""), children: [ H1("a"), Form(name: F, children: [ Input(value: name, label: "Name"), Input(value: message, label: "Message", lines: 3),
  If(name, not: "", children: [ If(message, not: "", children: [ Button(name: Send, text: "Send") ], else: [ Text("Write a message.") ]) ], else: [ Text("Write your name.") ]) ]) ], rules: [ On(Send.tap, effect: F.send) ])'
probe "P2-14 When sur un texte qui devient vide" 'Page(title: "a", state: State(name: "", warn: 0), children: [ H1("a"), Input(value: name, label: "Name") ], rules: [ When(name, is: "", effect: warn.set(1)) ])'

# --- Piste 6 : interactions ---
probe "P6-01 changement de champ : On(Size.change)" 'Page(title: "a", state: State(size: ""), children: [ H1("a"), Choice(name: Size, value: size, label: "Size", options: ["S", "M"]) ], rules: [ On(Size.change, effect: size.set("S")) ])'
probe "P6-02 réagir à une valeur choisie : When(size, is: \"M\")" 'Page(title: "a", state: State(size: "", note: 0), children: [ H1("a"), Choice(value: size, label: "Size", options: ["S", "M"]) ], rules: [ When(size, is: "M", effect: note.set(1)) ])'
probe "P6-03 focus seul : On(Name.focus)" 'Page(title: "a", state: State(name: "", tip: 0), children: [ H1("a"), Input(name: NameField, value: name, label: "Name") ], rules: [ On(NameField.focus, effect: tip.set(1)) ])'
probe "P6-04 survol d un champ : On(NameField.hover)" 'Page(title: "a", state: State(name: "", tip: 0), children: [ H1("a"), Input(name: NameField, value: name, label: "Name") ], rules: [ On(NameField.hover, effect: tip.set(1)), On(NameField.hoverEnd, effect: tip.set(0)) ])'
probe "P6-05 ouvrir un pli par une règle : Faq.open" 'Page(title: "a", children: [ H1("a"), Button(name: Go, text: "Open"), Details(name: Faq, summary: "Q?", children: [ P("A.") ]) ], rules: [ On(Go.tap, effect: Faq.open) ])'
probe "P6-06 la fenêtre fermée : On(Thanks.closed)" 'Page(title: "a", state: State(n: 0), children: [ H1("a"), Dialog(name: Thanks, children: [ P("Hi") ]) ], rules: [ On(Thanks.closed, effect: n.add(1)) ])'
probe "P6-07 un menu qui s ouvre : Button + If + Escape" 'Page(title: "a", state: State(menu: 0), children: [ H1("a"),
  Button(name: MenuButton, text: "Menu"),
  If(menu, is: 1, children: [ Nav(children: [ A("Home", to: "home.holo"), A("Shop", to: "shop.holo") ]) ]),
], rules: [ On(MenuButton.tap, effect: menu.set(1)), On(Key.escape, effect: menu.set(0)) ])'
probe "P6-08 basculer une valeur : menu.toggle" 'Page(title: "a", state: State(menu: 0), children: [ H1("a"), Button(name: MenuButton, text: "Menu") ], rules: [ On(MenuButton.tap, effect: menu.toggle) ])'
probe "P6-09 basculer par deux boutons dans un If/else" 'Page(title: "a", state: State(menu: 0), children: [ H1("a"),
  If(menu, is: 0, children: [ Button(name: OpenMenu, text: "Menu") ], else: [ Button(name: CloseMenu, text: "Close the menu") ]),
  If(menu, is: 1, children: [ Nav(children: [ A("Home", to: "home.holo") ]) ]),
], rules: [ On(OpenMenu.tap, effect: menu.set(1)), On(CloseMenu.tap, effect: menu.set(0)) ])'
probe "P6-10 un accordéon exclusif : Details(group:)" 'Page(title: "a", children: [ H1("a"), Details(summary: "Q1", group: faq, children: [ P("A1") ]), Details(summary: "Q2", group: faq, children: [ P("A2") ]) ])'
probe "P6-11 défilement : On(Page.scroll)" 'Page(name: Home, title: "a", state: State(top: 0), children: [ H1("a") ], rules: [ On(Home.scroll, effect: top.set(1)) ])'
probe "P6-12 donner le focus : NameField.focus comme capacité" 'Page(title: "a", state: State(name: ""), children: [ H1("a"), Input(name: NameField, value: name, label: "Name"), Button(name: Go, text: "Go") ], rules: [ On(Go.tap, effect: NameField.focus) ])'
probe "P6-13 bouton désactivé : Button(disabled:)" 'Page(title: "a", children: [ H1("a"), Button(name: Go, text: "Go", disabled: true) ])'
probe "P6-14 survol qui ouvre une fenêtre (refusé exprès)" 'Page(title: "a", children: [ H1("a"), Button(name: Go, text: "Go"), Dialog(name: D, children: [ P("x") ]) ], rules: [ On(Go.hover, effect: D.open) ])'

# --- Piste 7 : structure et publication ---
probe "P7-01 nom d un menu : Nav(label:)" 'Page(title: "a", children: [ H1("a"), Nav(label: "Main menu", children: [ A("Home", to: "home.holo") ]) ])'
probe "P7-02 page en cours : A(current: true)" 'Page(title: "a", children: [ H1("a"), Nav(children: [ A("Home", to: "home.holo", current: true) ]) ])'
probe "P7-03 adresse canonique : Page(url:)" 'Page(title: "a", url: "https://example.com/a.holo", children: [ H1("a") ])'
probe "P7-04 image de partage en SVG (acceptée)" 'Page(title: "a", image: "share.svg", description: "d", children: [ H1("a") ])'
probe "P7-05 texte de l image de partage : Page(imageAlt:)" 'Page(title: "a", image: "share.png", imageAlt: "A sun", children: [ H1("a") ])'
probe "P7-06 en-tête de ligne d un tableau : Table(rowHead:)" 'Page(title: "a", children: [ H1("a"), Table(caption: "c", head: ["Day", "Hours"], rowHead: true, rows: [ ["Mon", "9-18"] ]) ])'
probe "P7-07 tableau rempli par une liste : Repeat dans Table" 'Page(title: "a", state: State(orders: [ Item(day: "Mon", total: 120) ]), children: [ H1("a"), Table(caption: "c", head: ["Day", "Total"], rows: [ Repeat(over: orders, children: [ ["{item.day}", "{item.total}"] ]) ]) ])'
probe "P7-08 encadré : Aside" 'Page(title: "a", children: [ H1("a"), Aside(children: [ P("x") ]) ])'
probe "P7-09 deux Nav dans la page (accepté, sans nom)" 'Page(title: "a", children: [ Header(children: [ Nav(children: [ A("Home", to: "home.holo") ]) ]), H1("a"), Footer(children: [ Nav(children: [ A("Legal", to: "legal.holo") ]) ]) ])'
probe "P7-10 une valeur dans un tableau (accepté)" 'Page(title: "a", state: State(stock: 3), children: [ H1("a"), Table(caption: "Stock", head: ["Painting", "Left"], rows: [ ["Sunrise", "{stock}"] ]) ])'
probe "P7-11 description trop longue (refus attendu)" "Page(title: \"a\", description: \"$(printf 'x%.0s' $(seq 1 301))\", children: [ H1(\"a\") ])"
probe "P7-12 lien externe nouvel onglet : A(newTab:)" 'Page(title: "a", children: [ H1("a"), A("Elsewhere", to: "https://example.com", newTab: true) ])'
probe "P7-13 titre de page vide (accepté ?)" 'Page(title: "", children: [ H1("a") ])'
probe "P7-14 page sans titre (accepté ?)" 'Page(children: [ H1("a") ])'
probe "P7-15 page sans H1 (accepté ?)" 'Page(title: "a", children: [ P("no heading") ])'
