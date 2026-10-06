// Vérifie l'accessibilité des pages d'un dossier avec axe-core, dans un Chrome sans fenêtre
// (ADR-055). Le serveur d'essai doit tourner (node outils/serveur.mjs).
//
//     npm install --no-save playwright axe-core
//     node outils/accessibilite.mjs exemples/lecons            # thème clair, largeur de PC
//     SOMBRE=1 LARGEUR=390 node outils/accessibilite.mjs exemples/site-reference
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
const dossier = process.argv[2] ?? "exemples/lecons";
const depot = resolve(new URL("../..", import.meta.url).pathname);
const chemin = resolve(depot, dossier);
const adresse = `http://localhost:${process.env.PORT ?? 8080}/${chemin.slice(depot.length + 1).replaceAll("\\", "/")}/`;
// Les morceaux et les thèmes s'importent dans une page : ils ne s'ouvrent pas seuls.
const pages = readdirSync(chemin).filter((f) => f.endsWith(".holo") && !/^\s*(\/\/[^\n]*\n|\s)*(import[^\n]*\n\s*)*Part\s*\(/.test(readFileSync(join(chemin, f), "utf8")) && /\(/.test(readFileSync(join(chemin, f), "utf8"))).sort();
const navigateur = await chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {});
const page = await navigateur.newPage({ viewport: { width: Number(process.env.LARGEUR ?? 1000), height: 800 }, colorScheme: process.env.SOMBRE ? "dark" : "light" });
let defauts = 0;
for (const fichier of pages) {
  await page.goto(adresse + fichier, { waitUntil: "networkidle" });
  await page.addScriptTag({ content: axe });
  const trouves = await page.evaluate(async () => (await window.axe.run(document, { resultTypes: ["violations"] })).violations.map((v) => `[${v.impact}] ${v.id} : ${v.help} — ${v.nodes[0]?.html.slice(0, 120) ?? ""}`));
  for (const t of trouves) console.log(`${fichier} : ${t}`);
  defauts += trouves.length;
}
console.log(`${pages.length} page(s) vérifiée(s), ${defauts} défaut(s)`);
await navigateur.close();
process.exit(defauts ? 1 : 0);
