import { readFileSync } from "node:fs";
import * as holo from "../essais/pkg-light/holo_engine.js";
holo.initSync({ module: readFileSync(new URL("../essais/pkg-light/holo_engine_bg.wasm", import.meta.url)) });
console.log("Data(from: a, from: b) → data() =", JSON.stringify(holo.data(`Page(title: "a", state: State(n: 0), data: Data(from: "a.json", from: "b.json"), children: [ H1("x") ])`)));
console.log("Button(text x2) →", (holo.flat_view(`Page(title: "a", children: [ H1("x"), Button(name: B, text: "x", text: "y") ])`, "", "").match(/<button[^>]*data-name="B"[^>]*>[^<]*</) ?? [""])[0]);
