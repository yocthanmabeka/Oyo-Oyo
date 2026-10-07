// R15 : le catalogue de l'essai, une image fautive sur douze, avec la page essais/p1/aujourdhui.holo.
import { readFileSync } from "node:fs";
import * as holo from "../essais/pkg-light/holo_engine.js";
holo.initSync({ module: readFileSync(new URL("../essais/pkg-light/holo_engine_bg.wasm", import.meta.url)) });
const source = readFileSync(new URL("../essais/p1/aujourdhui.holo", import.meta.url), "utf8");
const json = JSON.parse(readFileSync(new URL("../essais/p1/catalogue.json", import.meta.url), "utf8"));
const count = (s) => (holo.list_html(source, "", s, "articles").match(/data-rank/g) ?? []).length;
const good = holo.receive(source, holo.initial_state(source), JSON.stringify(json));
json.articles[4].image = "../x.svg";
const bad = holo.receive(source, holo.initial_state(source), JSON.stringify(json));
console.log("douze images justes : lignes affichées =", count(good), "; une image « ../x.svg » : lignes affichées =", count(bad));
