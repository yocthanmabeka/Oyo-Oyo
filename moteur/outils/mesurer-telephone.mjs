// Lit les mesures du moteur sur le téléphone, par le câble (adb forward tcp:9222 localabstract:chrome_devtools_remote).
//     node mesurer_telephone.mjs <morceau d'adresse de l'onglet> <expression JavaScript> [attente ms]
import { readFileSync } from "node:fs";
let [morceau, expression, attente = "0"] = process.argv.slice(2);
if (expression.startsWith("@")) expression = readFileSync(expression.slice(1), "utf8"); // @fichier : le script à exécuter dans la page
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const onglets = await (await fetch("http://127.0.0.1:9222/json")).json();
const onglet = onglets.find((o) => o.type === "page" && o.url.includes(morceau));
if (!onglet) {
  console.log("onglet introuvable ; onglets ouverts :", onglets.filter((o) => o.type === "page").map((o) => o.url));
  process.exit(1);
}
const liaison = new WebSocket(onglet.webSocketDebuggerUrl);
await new Promise((ok, rate) => { liaison.onopen = ok; liaison.onerror = rate; });
let numero = 0;
const attentes = new Map();
liaison.onmessage = (m) => { const r = JSON.parse(m.data); attentes.get(r.id)?.(r); };
const demander = (method, params = {}) => new Promise((ok) => { attentes.set(++numero, ok); liaison.send(JSON.stringify({ id: numero, method, params })); });
await pause(Number(attente));
const reponse = await demander("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
console.log(typeof reponse.result?.result?.value === "string" ? reponse.result.result.value : JSON.stringify(reponse.result?.result?.value ?? reponse, null, 1));
liaison.close();
