// Mesure les octets réellement transférés pour ouvrir quelques pages, dans un Chrome sans fenêtre,
// cache vide : la somme des « encodedDataLength » du protocole de Chrome (ce qui passe sur le
// réseau, compressé). Sur ce PC seulement : ce n'est pas une mesure de téléphone.
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const BASE = process.env.BASE ?? "http://localhost:8080";
const PAGES = [
  ["leçon 1 (une page simple), sans geste", "/exemples/lecons/01-page.holo", null],
  ["site de référence, accueil, sans geste", "/exemples/site-reference/accueil.holo", null],
  ["leçon 71 (liste à champs reçue du serveur), sans geste", "/exemples/lecons/71-liste-a-champs.holo", null],
  ["leçon 1, après un toucher sur le menu (le moteur arrive)", "/exemples/lecons/01-page.holo", "#toggle"],
  ["leçon 81, en vue points (le dessin arrive)", "/exemples/lecons/81-vue-points-et-lecteur-d-ecran.holo", "zoom"],
];
const port = 9700 + Math.floor(Math.random() * 200);
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const chrome = spawn("C:/Program Files/Google/Chrome/Application/chrome.exe", [
  "--headless=new", "--enable-unsafe-swiftshader", "--no-first-run", `--remote-debugging-port=${port}`, "--window-size=1000,700",
  `--user-data-dir=${mkdtempSync(join(tmpdir(), "holo-poids-"))}`, "about:blank",
], { stdio: "ignore" });
try {
  let onglet;
  for (let i = 0; i < 50 && !onglet; i++) {
    await pause(200);
    try { onglet = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((o) => o.type === "page"); } catch {}
  }
  const ws = new WebSocket(onglet.webSocketDebuggerUrl);
  await new Promise((ok, ko) => { ws.onopen = ok; ws.onerror = ko; });
  let n = 0;
  const attentes = new Map();
  let octets = 0;
  const fichiers = new Map();
  const urls = new Map();
  ws.onmessage = (m) => {
    const r = JSON.parse(m.data);
    if (r.method === "Network.requestWillBeSent") urls.set(r.params.requestId, r.params.request.url);
    if (r.method === "Network.loadingFinished") {
      octets += r.params.encodedDataLength;
      const url = (urls.get(r.params.requestId) ?? "?").replace(BASE, "");
      fichiers.set(url, (fichiers.get(url) ?? 0) + r.params.encodedDataLength);
    }
    attentes.get(r.id)?.(r);
  };
  const cdp = (method, params = {}) => new Promise((ok) => { attentes.set(++n, ok); ws.send(JSON.stringify({ id: n, method, params })); });
  await cdp("Network.enable");
  await cdp("Network.setCacheDisabled", { cacheDisabled: true });
  console.log(`Chrome sans fenêtre, cache vide, serveur ${BASE} (Brotli). Octets transférés (compressés) :`);
  for (const [nom, chemin, geste] of PAGES) {
    await cdp("Network.clearBrowserCache");
    octets = 0;
    fichiers.clear();
    await cdp("Page.navigate", { url: BASE + chemin });
    await pause(4000);
    if (geste === "#toggle") {
      await cdp("Runtime.evaluate", { expression: `document.querySelector("#toggle").click()` });
      await pause(14000);
    }
    if (geste === "zoom") {
      for (let i = 0; i < 14; i++) { await cdp("Input.dispatchMouseEvent", { type: "mouseWheel", x: 500, y: 200, deltaX: 0, deltaY: -500, modifiers: 2 }); await pause(i === 0 ? 14000 : 300); }
      await pause(3000);
    }
    const gros = [...fichiers].sort((a, b) => b[1] - a[1]).slice(0, 4).map(([u, o]) => `${u.split("?")[0]} ${(o / 1000).toFixed(1)} Ko`).join(" ; ");
    console.log(`- ${nom} : ${(octets / 1000).toFixed(1)} Ko (${fichiers.size} fichiers ; les plus lourds : ${gros})`);
  }
  ws.close();
} finally {
  chrome.kill();
}
