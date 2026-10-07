import { spawn } from "node:child_process";
import { join } from "node:path";
import { openChrome, pause } from "./cdp.mjs";
const repo = "C:/Users/mokea/Documents/IA_creation/Metaverse";
const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const PORT = 8095, base = `http://localhost:${PORT}`;
const server = spawn(process.execPath, [join(repo, "moteur/outils/server.mjs")], { env: { ...process.env, PORT: String(PORT), HOLO_REPO: join(here, "site") }, stdio: "ignore" });
try {
  for (let i = 0; i < 50; i++) { await pause(200); try { if ((await fetch(`${base}/page.html`)).ok) break; } catch {} }
  const c = await openChrome();
  try {
    await c.ask("Runtime.enable"); await c.ask("Page.enable");
    await c.ask("Page.addScriptToEvaluateOnNewDocument", { source: "window.__errs=[];addEventListener('error',e=>window.__errs.push(e.message+' @'+e.filename.split('/').pop()+':'+e.lineno),true);" });
    await c.ask("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 3, mobile: true });
    await c.ask("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
    await c.ask("Page.navigate", { url: `${base}/exemples/media/pincer.holo?essai` });
    await pause(9000);
    const r = await c.ask("Runtime.evaluate", { expression: "JSON.stringify(Object.fromEntries(['touchstart','touchmove','touchend','wheel'].map(n => [n, (getEventListeners(window)[n]||[]).length])))", includeCommandLineAPI: true, returnByValue: true });
    console.log("écouteurs sur window :", r.result.result.value);
    const synth = await c.evaluate(`(() => { const t1 = new Touch({ identifier: 1, target: document.body, clientX: 180, clientY: 400 }); const t2 = new Touch({ identifier: 2, target: document.body, clientX: 210, clientY: 400 });
      document.body.dispatchEvent(new TouchEvent('touchstart', { touches: [t1, t2], targetTouches: [t1, t2], changedTouches: [t1, t2], bubbles: true, cancelable: true }));
      return { erreurs: window.__errs.slice(), touchesExiste: 'touches' in TouchEvent.prototype, keypressesExiste: 'keypresses' in TouchEvent.prototype }; })()`);
    console.log("toucher à deux doigts fabriqué dans la page :", JSON.stringify(synth));
  } finally { await c.close(); }
} finally { server.kill(); }
