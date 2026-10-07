// Sonde de l'exploration (pistes 1 et 10) : appelle le moteur léger (copie de moteur/web/pkg-light,
// construit depuis main) dans Node, sans navigateur. Lecture seule : rien n'est écrit dans le dépôt.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import * as holo from "./pkg-light/holo_engine.js";

const here = dirname(fileURLToPath(import.meta.url));
holo.initSync({ module: readFileSync(join(here, "pkg-light", "holo_engine_bg.wasm")) });

const line = (s) => console.log(s);

// N1 — deux Repeat(over:) sur la même liste : à la fabrication, chacun a son modèle ; quand la liste
// change, la page redemande les lignes par le nom de la liste (list_html), qui ne connaît que le premier.
{
  const source = `Page(title: "x", state: State(articles: [ Item(title: "a", cat: 1) ]), children: [ H1("x"), Button(name: Add, text: "+"), Repeat(over: articles, children: [ P("A {item.title}") ]), Repeat(over: articles, children: [ P("B {item.title}") ]) ], rules: [ On(Add.tap, effect: articles.push(Item(title: "z", cat: 2))) ])`;
  const html = holo.flat_view(source, "", "");
  line("N1 page fabriquée : modèle A présent = " + html.includes(">A a<") + ", modèle B présent = " + html.includes(">B a<"));
  const after = holo.arbitrate(source, holo.initial_state(source), "Add.tap");
  const lines = holo.list_html(source, "", after, "articles");
  line("N1 après un ajout, lignes rendues pour « articles » : " + lines.replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim());
  line("N1 => le second Repeat recevrait-il son modèle B ? " + lines.includes("B "));
}

// N3 — ce que Data fait d'un catalogue trop long, d'un prix à virgule, d'un prix négatif.
{
  const source = `Page(title: "x", state: State(articles: [ Item(title: "", price: 0) ]), data: Data(from: "c.json"), children: [ H1("x"), P("{articles} articles"), Repeat(over: articles, children: [ P("{item.title} : {item.price:cents} euros") ]) ])`;
  const many = JSON.stringify({ articles: Array.from({ length: 150 }, (_, i) => ({ title: "T" + i, price: 1000 + i })) });
  const s150 = holo.receive(source, holo.initial_state(source), many);
  const count = (s150.split(";").find((c) => c.startsWith("articles=[")) ?? "").split(",").length;
  line("N3 reçu 150 articles, gardés : " + count);
  const odd = JSON.stringify({ articles: [ { title: "Virgule", price: 12.5 }, { title: "Moins", price: -300 }, { title: "Texte", price: "1200" }, { title: "Juste", price: 1250 } ] });
  const sOdd = holo.receive(source, holo.initial_state(source), odd);
  line("N3 prix étranges : " + holo.list_html(source, "", sOdd, "articles").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim());
  const big = JSON.stringify({ articles: Array.from({ length: 100 }, (_, i) => ({ title: "Titre " + i, price: i, note: "x".repeat(700) })) });
  const sBig = holo.receive(source, holo.initial_state(source), big);
  const kept = (sBig.split(";").find((c) => c.startsWith("articles=[")) ?? "");
  line("N3 fichier de " + big.length + " octets (plus de 65 536) : articles gardés = " + (kept.slice(10, -1) ? kept.slice(10, -1).split(",").length : 0) + " ; l'état reste celui de départ ? " + (sBig === holo.initial_state(source)));
}

// N2 — combien coûte un geste, sur ce PC, dans Node (V8 + WebAssembly), pour 12 et 100 articles.
function catalogue(n) {
  const source = `Page(
  title: "Catalogue",
  state: State(search: "", cart: 0, articles: [ Item(title: "", price: 0, image: "vide.svg", cat: 0) ]),
  data: Data(from: "c.json"),
  children: [
    H1("Catalogue"),
    Input(value: search, label: "Chercher"),
    P("{articles} créations, {cart:cents} euros au panier"),
    Grid(columns: 3, children: [
      Repeat(over: articles, children: [
        Column(children: [
          Image(source: item.image, alt: "{item.title}"),
          H2("{item.title}"),
          Text("{item.price:cents} euros"),
          If(item.cat, is: 1, children: [ Text("Fleuve") ], else: [ Text("Autre") ]),
          Button(name: Add, text: "Ajouter"),
        ]),
      ], rules: [ On(Add.tap, effect: cart.add(1)) ]),
    ]),
  ],
)`;
  const json = JSON.stringify({ articles: Array.from({ length: n }, (_, i) => ({ title: "Création " + i, price: 4000 + i * 10, image: "img" + i + ".svg", cat: i % 2 })) });
  const state = holo.receive(source, holo.initial_state(source), json);
  return { source, state };
}
function time(label, f, runs = 200) {
  for (let i = 0; i < 20; i++) f(); // chauffe
  const t = [];
  for (let i = 0; i < runs; i++) { const a = performance.now(); f(); t.push(performance.now() - a); }
  t.sort((x, y) => x - y);
  line(`${label} : médiane ${t[runs >> 1].toFixed(3)} ms, 95e centile ${t[Math.floor(runs * 0.95)].toFixed(3)} ms`);
}
for (const n of [12, 100]) {
  const { source, state } = catalogue(n);
  line(`N2 source ${source.length} octets, état ${state.length} octets, ${n} articles`);
  time(`N2 [${n}] input (une lettre tapée)`, () => holo.input(source, state, "search", "fle"));
  time(`N2 [${n}] arbitrate (Add.tap@3)`, () => holo.arbitrate(source, state, "Add.tap@3"));
  time(`N2 [${n}] list_html (toutes les lignes)`, () => holo.list_html(source, "", state, "articles"));
  time(`N2 [${n}] conditions`, () => holo.conditions(source, state));
  const lines = holo.list_html(source, "", state, "articles");
  line(`N2 [${n}] HTML des lignes : ${lines.length} octets`);
}
line("Node " + process.version + ", " + process.platform + " " + process.arch);
