// Lit les mesures du moteur sur le téléphone, par le câble (adb forward tcp:9222 localabstract:chrome_devtools_remote).
//     node mesurer_telephone.mjs <morceau d'adresse de l'onglet> <expression JavaScript> [attente ms]
import { readFileSync } from "node:fs";
let [chunk, expression, waiting = "0"] = process.argv.slice(2);
if (expression.startsWith("@")) expression = readFileSync(expression.slice(1), "utf8"); // @fichier : le script à exécuter dans la page
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const tabs = await (await fetch("http://127.0.0.1:9222/json")).json();
const tab = tabs.find((o) => o.type === "page" && o.url.includes(chunk));
if (!tab) {
  console.log("onglet introuvable ; onglets ouverts :", tabs.filter((o) => o.type === "page").map((o) => o.url));
  process.exit(1);
}
const bindGroup = new WebSocket(tab.webSocketDebuggerUrl);
await new Promise((ok, rate) => { bindGroup.onopen = ok; bindGroup.onerror = rate; });
let number_ = 0;
const waitings = new Map();
bindGroup.onmessage = (m) => { const r = JSON.parse(m.data); waitings.get(r.id)?.(r); };
const ask = (method, params = {}) => new Promise((ok) => { waitings.set(++number_, ok); bindGroup.send(JSON.stringify({ id: number_, method, params })); });
await pause(Number(waiting));
const response = await ask("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
console.log(typeof response.result?.result?.value === "string" ? response.result.result.value : JSON.stringify(response.result?.result?.value ?? response, null, 1));
bindGroup.close();
