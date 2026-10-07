// Relecture (pistes 8 et 9) : où une grande image déborde, et ce que lit un lecteur d'écran
// d'un titre coupé en lettres. Chrome sans fenêtre, taille de téléphone (390 × 844, densité 3).
// Ce n'est PAS un téléphone.
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { openChrome, pause } from "./cdp.mjs";
const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const wrap = (name) => {
  const flat = readFileSync(join(here, `${name}.flat.html`), "utf8");
  const file = join(here, `${name}.page.html`);
  writeFileSync(file, `<!doctype html><html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><style>html,body{margin:0}</style></head><body><div id="page">${flat}</div></body></html>`);
  return pathToFileURL(file).href;
};
const c = await openChrome();
const out = {};
try {
  await c.ask("Page.enable"); await c.ask("Runtime.enable");
  await c.ask("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 3, mobile: true });
  for (const kind of ["direct", "grid", "row", "column", "stack"]) {
    await c.ask("Page.navigate", { url: wrap(`img-${kind}`) });
    await pause(1500);
    out[kind] = await c.evaluate(`(() => { const i = document.querySelector("img"); const r = i.getBoundingClientRect(); return { largeurEcran: innerWidth, largeurPage: document.documentElement.scrollWidth, imageAffichee: Math.round(r.width) + "x" + Math.round(r.height), chargee: i.complete && i.naturalWidth }; })()`);
  }
  await c.ask("Accessibility.enable");
  await c.ask("Page.navigate", { url: wrap("lettres") });
  await pause(1500);
  const { result } = await c.ask("Accessibility.getFullAXTree");
  out.lettres = result.nodes.filter((n) => ["heading", "paragraph"].includes(n.role?.value)).map((n) => ({ role: n.role.value, nom: n.name?.value ?? "" }));
  out.lettresTexte = result.nodes.filter((n) => n.role?.value === "StaticText").map((n) => n.name?.value).slice(0, 40);
} finally { await c.close(); }
console.log(JSON.stringify(out, null, 1));
