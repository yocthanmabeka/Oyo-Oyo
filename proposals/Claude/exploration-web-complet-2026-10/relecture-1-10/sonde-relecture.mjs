// Relecture des pistes 1 et 10 : quelques appels au moteur léger (copie de moteur/web/pkg-light) dans Node.
import { readFileSync } from "node:fs";
import * as holo from "../essais/pkg-light/holo_engine.js";
holo.initSync({ module: readFileSync(new URL("../essais/pkg-light/holo_engine_bg.wasm", import.meta.url)) });
const say = (s) => console.log(s);
// D9 : deux data:
const two = readFileSync(new URL("../essais/deux-data.holo", import.meta.url), "utf8");
say("D9 data(deux data:) = " + JSON.stringify(holo.data(two)));
// Le détour : chosen.set(item) puis cart.push(Item(title: chosen, price: 0)).
const detour = readFileSync(new URL("./detour.holo", import.meta.url), "utf8");
let s = holo.initial_state(detour);
s = holo.arbitrate(detour, s, "Pick.tap@1");
say("détour : état = " + s);
say("détour : lignes du panier = " + holo.list_html(detour, "", s, "cart").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim());
// Liste déclarée vide : tous les champs du JSON sont repris.
const free = `Page(title: "x", state: State(articles: []), data: Data(from: "c.json"), children: [ H1("x"), Repeat(over: articles, children: [ P("{item.title}") ]) ])`;
const got = holo.receive(free, holo.initial_state(free), JSON.stringify({ articles: [ { title: "T", secret: "s", price: 5 } ] }));
say("liste vide + Data : état = " + got);
// D1 : à l'affichage dans le navigateur, redrawLists() redemande les lignes par le nom de la liste.
const twoRepeats = `Page(title: "x", state: State(articles: [ "a" ]), children: [ H1("x"), Repeat(over: articles, children: [ P("A {item}") ]), Repeat(over: articles, children: [ P("B {item}") ]) ])`;
const first = holo.list_html(twoRepeats, "", holo.initial_state(twoRepeats), "articles");
say("D1 lignes rendues pour « articles », état de départ : " + first.replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim());
