// 1) Le trou : If(texte, is: nombre) compare « vide ou non » au nombre.
import { readFileSync } from "node:fs";
import * as holo from "../essais/pkg-light/holo_engine.js";
holo.initSync({ module: readFileSync(new URL("../essais/pkg-light/holo_engine_bg.wasm", import.meta.url)) });
const src = readFileSync(new URL("./text-vs-number.holo", import.meta.url), "utf8");
const s0 = holo.initial_state(src);
const s1 = holo.input(src, s0, "size", "M");
console.log("If(size, is: n), n = 1 : size vide →", holo.conditions(src, s0), "; size = « M » →", holo.conditions(src, s1));
// 2) Le placement des lignes, recopié de page-engine.js:373-394, sur un faux conteneur qui compte les déplacements.
function container(nodes) {
  const c = { children: [...nodes], moved: [] };
  c.insertBefore = (n, ref) => { const i = c.children.indexOf(n); if (i >= 0) { c.children.splice(i, 1); c.moved.push(n.id); } const j = ref ? c.children.indexOf(ref) : c.children.length; c.children.splice(j < 0 ? c.children.length : j, 0, n); };
  Object.defineProperty(c, "lastElementChild", { get: () => c.children[c.children.length - 1] });
  return c;
}
function placeLines(c, lines) {
  lines.forEach((line, rank) => { if (c.children[rank] !== line) c.insertBefore(line, c.children[rank] ?? null); });
  while (c.children.length > lines.length) c.children.pop();
}
const old = [0, 1, 2, 3].map((i) => ({ id: "ancienne" + i }));
let c = container(old);
placeLines(c, [{ id: "neuve0" }, old[1], old[2], old[3]]);
console.log("ligne 0 refaite : lignes inchangées déplacées =", c.moved.join(", ") || "aucune");
c = container(old);
placeLines(c, [old[1], old[2], old[3]]);
console.log("ligne 0 retirée : lignes inchangées déplacées =", c.moved.join(", ") || "aucune");
c = container(old);
placeLines(c, [old[0], old[1], old[2], { id: "neuve3" }]);
console.log("dernière ligne refaite : lignes inchangées déplacées =", c.moved.join(", ") || "aucune");
