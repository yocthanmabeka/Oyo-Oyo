// Pincer une page au doigt, avec le moteur arrivé : les erreurs du navigateur, et la page grossie ou non.
//   node pincer.mjs
import { spawn } from "node:child_process";
import { join } from "node:path";
import { openChrome, pause } from "./cdp.mjs";

const repo = "C:/Users/mokea/Documents/IA_creation/Metaverse";
const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const PORT = 8094;
const base = `http://localhost:${PORT}`;
const server = spawn(process.execPath, [join(repo, "moteur/outils/server.mjs")], { env: { ...process.env, PORT: String(PORT), HOLO_REPO: join(here, "site") }, stdio: "ignore" });
const out = {};
try {
  for (let i = 0; i < 50; i++) { await pause(200); try { if ((await fetch(`${base}/page.html`)).ok) break; } catch {} }
  const c = await openChrome();
  try {
    await c.ask("Runtime.enable");
    await c.ask("Page.enable");
    const errors = [];
    c.on("Runtime.exceptionThrown", (p) => errors.push((p.exceptionDetails.exception?.description ?? p.exceptionDetails.text ?? "").split("\n").slice(0, 2).join(" | ")));
    await c.ask("Page.addScriptToEvaluateOnNewDocument", { source: "window.__errs=[];addEventListener('error',e=>window.__errs.push(e.message+' @'+(e.lineno||'')),true);" });
    await c.ask("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 3, mobile: true });
    await c.ask("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
    for (const [name, url] of [["page legere", `${base}/exemples/media/pincer.holo`], ["moteur force par l'adresse", `${base}/exemples/media/pincer.holo?essai`]]) {
      errors.length = 0;
      await c.ask("Page.navigate", { url });
      await pause(9000);
      const state = await c.evaluate("({ moteur: performance.getEntriesByType('resource').some(r => r.name.includes('page-engine.js')), pageReprise: !!document.querySelector('#page .holo-Page'), erreursAvant: window.__errs.slice() })");
      const t = (type, pts) => c.ask("Input.dispatchTouchEvent", { type, touchPoints: pts });
      await t("touchStart", [{ x: 180, y: 400, id: 1 }, { x: 210, y: 400, id: 2 }]);
      for (let k = 1; k <= 8; k++) { await t("touchMove", [{ x: 180 - k * 12, y: 400, id: 1 }, { x: 210 + k * 12, y: 400, id: 2 }]); await pause(50); }
      await t("touchEnd", []);
      await pause(2000);
      out[name] = { ...state, erreursApres: await c.evaluate("window.__errs.slice()"), exceptionsCDP: errors.slice(), largeurPage: await c.evaluate("document.getElementById('page').style.width || 'pas grossie'"), vuePoints: await c.evaluate("document.body.classList.contains('in-points')") };
    }
  } finally { await c.close(); }
} finally { server.kill(); }
console.log(JSON.stringify(out, null, 1));
