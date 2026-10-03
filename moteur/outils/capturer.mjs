// Prend une capture d'écran d'une page de la démo, après lui avoir laissé le temps de démarrer.
//
//     node outils/capturer.mjs <adresse> <fichier.png> [largeur] [hauteur] [attente en ms] [densité]
//
// « densité » : 2 pour une image deux fois plus fine (2560 × 1440 pour une vue de 1280 × 720).
// Lance un Chrome sans fenêtre, attend en temps réel (la carte graphique ne suit pas le
// « temps simulé » de l'option --screenshot, qui capturait avant le premier dessin), puis
// demande la capture. Sert au journal et aux vérifications ; ne mesure rien.

import { spawn } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const [adresse, sortie, largeur = "1280", hauteur = "720", attente = "6000", densite = "1"] = process.argv.slice(2);
if (!adresse || !sortie) {
  console.error("usage : node outils/capturer.mjs <adresse> <fichier.png> [largeur] [hauteur] [attente en ms]");
  process.exit(2);
}
const chrome = process.env.CHROME ?? "C:/Program Files/Google/Chrome/Application/chrome.exe";
const port = 9300 + Math.floor(Math.random() * 500);
const pause = (ms) => new Promise((r) => setTimeout(r, ms));

const navigateur = spawn(chrome, [
  // Sans le lissage coloré des écrans plats : il laisserait des franges bleues et orange autour des lettres.
  "--headless=new", "--enable-unsafe-swiftshader", "--hide-scrollbars", "--no-first-run", "--disable-lcd-text",
  `--remote-debugging-port=${port}`, `--window-size=${largeur},${hauteur}`,
  `--user-data-dir=${mkdtempSync(join(tmpdir(), "holo-capture-"))}`, "about:blank",
], { stdio: "ignore" });

try {
  let onglet;
  for (let essai = 0; essai < 50 && !onglet; essai++) {
    await pause(200);
    try {
      onglet = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((o) => o.type === "page");
    } catch { /* Chrome n'écoute pas encore */ }
  }
  if (!onglet) throw new Error("Chrome n'a pas démarré");
  const liaison = new WebSocket(onglet.webSocketDebuggerUrl);
  await new Promise((ok, raté) => { liaison.onopen = ok; liaison.onerror = raté; });
  let numero = 0;
  const attentes = new Map();
  liaison.onmessage = (message) => {
    const reponse = JSON.parse(message.data);
    attentes.get(reponse.id)?.(reponse);
  };
  const demander = (method, params = {}) => new Promise((ok) => {
    attentes.set(++numero, ok);
    liaison.send(JSON.stringify({ id: numero, method, params }));
  });
  await demander("Emulation.setDeviceMetricsOverride", { width: Number(largeur), height: Number(hauteur), deviceScaleFactor: Number(densite), mobile: false });
  await demander("Page.navigate", { url: adresse });
  await pause(Number(attente));
  // HOLO_GESTES : une liste de commandes du protocole de Chrome à jouer avant la capture, pour
  // essayer un geste (molette, clic) comme le ferait une main. Exemple dans moteur/README.md.
  for (const geste of JSON.parse(process.env.HOLO_GESTES ?? "[]")) {
    await demander(geste.method, geste.params);
    await pause(geste.attente ?? 400);
  }
  const { result } = await demander("Page.captureScreenshot", { format: "png" });
  writeFileSync(sortie, Buffer.from(result.data, "base64"));
  console.log(`capture écrite : ${sortie}`);
  liaison.close();
} finally {
  navigateur.kill();
}
