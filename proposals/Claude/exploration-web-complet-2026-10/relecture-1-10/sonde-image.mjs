// Relecture piste 1 : une seule image refusée dans un catalogue reçu du serveur.
import { readFileSync } from "node:fs";
import * as holo from "../essais/pkg-light/holo_engine.js";
holo.initSync({ module: readFileSync(new URL("../essais/pkg-light/holo_engine_bg.wasm", import.meta.url)) });
const source = `Page(title: "x", state: State(articles: [ Item(title: "", price: 0, image: "vide.svg") ]), data: Data(from: "c.json"), children: [ H1("x"), P("{articles} articles"), Repeat(over: articles, children: [ Column(children: [ Image(source: item.image, alt: "{item.title}"), P("{item.title}") ]) ]) ])`;
const show = (label, json) => {
  const s = holo.receive(source, holo.initial_state(source), JSON.stringify(json));
  const html = holo.list_html(source, "", s, "articles");
  const n = (s.split(";").find((c) => c.startsWith("articles=[")) ?? "").split(",").length;
  console.log(`${label} : ${n} éléments dans l'état ; lignes rendues = ${(html.match(/data-rank/g) ?? []).length} ; HTML ${html.length} octets`);
};
show("trois images justes", { articles: [ { title: "A", price: 1, image: "a.svg" }, { title: "B", price: 2, image: "b.svg" }, { title: "C", price: 3, image: "c.svg" } ] });
show("une image « javascript:alert(1) » sur trois", { articles: [ { title: "A", price: 1, image: "a.svg" }, { title: "B", price: 2, image: "javascript:alert(1)" }, { title: "C", price: 3, image: "c.svg" } ] });
show("une image « ../x.svg » sur trois", { articles: [ { title: "A", price: 1, image: "a.svg" }, { title: "B", price: 2, image: "../x.svg" }, { title: "C", price: 3, image: "c.svg" } ] });
show("une image manquante (champ absent) sur trois", { articles: [ { title: "A", price: 1, image: "a.svg" }, { title: "B", price: 2 }, { title: "C", price: 3, image: "c.svg" } ] });
const page = holo.flat_view(source, "", "");
console.log("page fabriquée (élément d'attente) contient des lignes :", (page.match(/data-rank/g) ?? []).length);
