// Vérifie l'accessibilité des pages d'un dossier avec axe-core, dans un Chrome sans fenêtre
// (ADR-055). Le serveur d'essai doit tourner (node outils/server.mjs).
//
//     npm install --no-save playwright axe-core
//     node outils/accessibility.mjs exemples/lecons            # thème clair, largeur de PC
//     DARK=1 WIDTH=390 node outils/accessibility.mjs exemples/site-reference
//
// Rend la liste des défauts trouvés, page par page, et un code de sortie 1 s'il y en a.
// Ce que l'outil ne voit pas : ce qu'entend vraiment une personne avec un lecteur d'écran ;
// pour cela, voir docs/01-holocode/ESSAI-LECTEUR-D-ECRAN.md.

import { createRequire } from "node:module";
import { readFileSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";

const require = createRequire(join(process.cwd(), "x.js"));
const { chromium } = require("playwright");
const axe = readFileSync(require.resolve("axe-core/axe.min.js"), "utf8");
const folder = process.argv[2] ?? "exemples/lecons";
const repo = resolve(new URL("../..", import.meta.url).pathname);
const path = resolve(repo, folder);
const address = `http://localhost:${process.env.PORT ?? 8080}/${path.slice(repo.length + 1).replaceAll("\\", "/")}/`;
// Les morceaux et les thèmes s'importent dans une page : ils ne s'ouvrent pas seuls.
const pages = readdirSync(path).filter((f) => f.endsWith(".holo") && !/^\s*(\/\/[^\n]*\n|\s)*(import[^\n]*\n\s*)*Component\s*\(/.test(readFileSync(join(path, f), "utf8")) && /\(/.test(readFileSync(join(path, f), "utf8"))).sort();
const browser = await chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {});
const page = await browser.newPage({ viewport: { width: Number(process.env.WIDTH ?? 1000), height: 800 }, colorScheme: process.env.DARK ? "dark" : "light" });
let defaults = 0;
for (const file of pages) {
  await page.goto(address + file, { waitUntil: "networkidle" });
  await page.addScriptTag({ content: axe });
  const foundList = await page.evaluate(async () => (await window.axe.run(document, { resultTypes: ["violations"] })).violations.map((v) => `[${v.impact}] ${v.id} : ${v.help} — ${v.nodes[0]?.html.slice(0, 120) ?? ""}`));
  for (const t of foundList) console.log(`${file} : ${t}`);
  defaults += foundList.length;
}
console.log(`${pages.length} page(s) vérifiée(s), ${defaults} défaut(s)`);
await browser.close();
process.exit(defaults ? 1 : 0);
