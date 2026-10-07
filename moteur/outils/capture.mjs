// Prend une capture d'écran d'une page de la démo, après lui avoir laissé le temps de démarrer.
//
//     node outils/capture.mjs <adresse> <fichier.png> [largeur] [hauteur] [attente en ms] [densité]
//
// « densité » : 2 pour une image deux fois plus fine (2560 × 1440 pour une vue de 1280 × 720).
// Lance un Chrome sans fenêtre, attend en temps réel (la carte graphique ne suit pas le
// « temps simulé » de l'option --screenshot, qui capturait avant le premier dessin), puis
// demande la capture. Sert au journal et aux vérifications ; ne mesure rien.

import { spawn } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const [address, output, width = "1280", height = "720", waiting = "6000", density = "1"] = process.argv.slice(2);
if (!address || !output) {
  console.error("usage : node outils/capture.mjs <adresse> <fichier.png> [largeur] [hauteur] [attente en ms]");
  process.exit(2);
}
const chrome = process.env.CHROME ?? "C:/Program Files/Google/Chrome/Application/chrome.exe";
const port = 9300 + Math.floor(Math.random() * 500);
const pause = (ms) => new Promise((r) => setTimeout(r, ms));

const browser = spawn(chrome, [
  // Sans le lissage coloré des écrans plats : il laisserait des franges bleues et orange autour des lettres.
  "--headless=new", "--enable-unsafe-swiftshader", "--hide-scrollbars", "--no-first-run", "--disable-lcd-text",
  `--remote-debugging-port=${port}`, `--window-size=${width},${height}`,
  `--user-data-dir=${mkdtempSync(join(tmpdir(), "holo-capture-"))}`, "about:blank",
], { stdio: "ignore" });

try {
  let tab;
  for (let test = 0; test < 50 && !tab; test++) {
    await pause(200);
    try {
      tab = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((o) => o.type === "page");
    } catch { /* Chrome n'écoute pas encore */ }
  }
  if (!tab) throw new Error("Chrome n'a pas démarré");
  const bindGroup = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((ok, failed) => { bindGroup.onopen = ok; bindGroup.onerror = failed; });
  let number_ = 0;
  const waitings = new Map();
  bindGroup.onmessage = (message) => {
    const response = JSON.parse(message.data);
    waitings.get(response.id)?.(response);
  };
  const ask = (method, params = {}) => new Promise((ok) => {
    waitings.set(++number_, ok);
    bindGroup.send(JSON.stringify({ id: number_, method, params }));
  });
  await ask("Emulation.setDeviceMetricsOverride", { width: Number(width), height: Number(height), deviceScaleFactor: Number(density), mobile: false });
  await ask("Page.navigate", { url: address });
  await pause(Number(waiting));
  // HOLO_GESTURES : une liste de commandes du protocole de Chrome à jouer avant la capture, pour
  // essayer un geste (molette, clic) comme le ferait une main. Exemple dans moteur/README.md.
  for (const gesture of JSON.parse(process.env.HOLO_GESTURES ?? "[]")) {
    const response = await ask(gesture.method, gesture.params);
    // Une valeur demandée à la page (Runtime.evaluate) est affichée : pour mesurer.
    if (response.result?.result) console.log(`${gesture.params.expression} → ${response.result.result.value}`);
    await pause(gesture.waiting ?? 400);
  }
  const { result } = await ask("Page.captureScreenshot", { format: "png" });
  writeFileSync(output, Buffer.from(result.data, "base64"));
  console.log(`capture écrite : ${output}`);
  bindGroup.close();
} finally {
  browser.kill();
}
