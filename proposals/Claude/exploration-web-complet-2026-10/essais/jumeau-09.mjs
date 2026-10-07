// Vérifie que le jumeau web de l'exemple de la piste 9 fait bien ce qu'il annonce (Chrome de PC, sans fenêtre).
import { spawn } from "node:child_process";
import { join } from "node:path";
import { openChrome, pause } from "./cdp.mjs";
const repo = "C:/Users/mokea/Documents/IA_creation/Metaverse";
const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const PORT = 8097, url = `http://localhost:${PORT}/exemples/media/exemple-09.html`;
const server = spawn(process.execPath, [join(repo, "moteur/outils/server.mjs")], { env: { ...process.env, PORT: String(PORT), HOLO_REPO: join(here, "site") }, stdio: "ignore" });
const out = {};
try {
  for (let i = 0; i < 50; i++) { await pause(200); try { if ((await fetch(`http://localhost:${PORT}/page.html`)).ok) break; } catch {} }
  const c = await openChrome();
  try {
    await c.ask("Page.enable"); await c.ask("Runtime.enable"); await c.ask("Accessibility.enable");
    const errors = [];
    c.on("Runtime.exceptionThrown", (p) => errors.push(p.exceptionDetails.exception?.description?.split("\n")[0] ?? p.exceptionDetails.text));
    await c.ask("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 3, mobile: true });
    await c.ask("Page.navigate", { url });
    await pause(500);
    const grains = await c.evaluate(`(() => { const t = document.getElementById('poussiere'); const d = t.getContext('2d').getImageData(0, 0, t.width, t.height).data; let n = 0; for (let k = 3; k < d.length; k += 4) if (d[k]) n++; return n; })()`);
    const lune1 = await c.evaluate("JSON.stringify(document.querySelector('.lune').getBoundingClientRect().toJSON())");
    await pause(1000);
    const lune2 = await c.evaluate("JSON.stringify(document.querySelector('.lune').getBoundingClientRect().toJSON())");
    const { result } = await c.ask("Accessibility.getFullAXTree");
    out.normal = {
      erreurs: errors.slice(),
      titreEntendu: (result?.nodes ?? []).filter((n) => n.role?.value === "heading").map((n) => n.name?.value),
      pixelsDePoussiereA05s: grains,
      formeSommets: await c.evaluate("(getComputedStyle(document.querySelector('.forme')).clipPath.match(/%/g) || []).length / 2"),
      luneBouge: lune1 !== lune2,
      defile: await c.evaluate("getComputedStyle(document.querySelector('.defile')).animationTimeline"),
      animationsEnCours: await c.evaluate("document.getAnimations().filter(a => a.playState === 'running').length"),
    };
    await c.ask("Input.dispatchMouseEvent", { type: "mouseMoved", x: 380, y: 100 });
    await pause(300);
    out.normal.pencheApresSouris = await c.evaluate("document.querySelector('.penche').style.getPropertyValue('--px') + ' / ' + document.querySelector('.penche').style.getPropertyValue('--py')");
    await c.ask("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
    await c.ask("Page.navigate", { url });
    await pause(1500);
    await c.ask("Input.dispatchMouseEvent", { type: "mouseMoved", x: 380, y: 100 });
    await pause(300);
    out.moinsDeMouvement = {
      animationsEnCours: await c.evaluate("document.getAnimations().filter(a => a.playState === 'running').length"),
      poussiere: await c.evaluate("getComputedStyle(document.getElementById('poussiere')).display"),
      penche: await c.evaluate("document.querySelector('.penche').style.getPropertyValue('--px') || 'au repos'"),
      texteVisible: await c.evaluate("getComputedStyle(document.querySelector('.defile')).opacity"),
    };
  } finally { await c.close(); }
} finally { server.kill(); }
console.log(JSON.stringify(out, null, 1));
