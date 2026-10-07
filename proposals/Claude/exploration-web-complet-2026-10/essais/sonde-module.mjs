// Sonde : la boîte d'un module (ADR-045) reçoit-elle ce que la page lui envoie ?
// On lit le code de la boîte tel qu'il est dans moteur/web/page-engine.js (commit donné en argument,
// par `git show`), on le fait tourner dans Node comme le ferait un Worker (postMessage capturé),
// et on lui envoie le message exactement comme la page l'envoie (page-engine.js, box.postMessage).
// Lecture seule : rien n'est écrit dans le dépôt.
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import vm from "node:vm";

const repo = "C:/Users/mokea/Documents/IA_creation/Metaverse";
const commit = process.argv[2] ?? "7a48def";
const engine = execFileSync("git", ["-C", repo, "show", `${commit}:moteur/web/page-engine.js`], { encoding: "utf8" });
const code = engine.match(/const SANDBOX_CODE = `([\s\S]*?)`;/)[1];
const host = engine.match(/box\.postMessage\((\{[^}]*\})/)[1];
console.log("commit", commit);
console.log("la page envoie :", host);
console.log("la boîte attend :", code.match(/data: (\{[^}]*\})/)[1]);

async function run(message) {
  const received = [];
  const context = { postMessage: (m) => received.push(m), WebAssembly, String };
  vm.createContext(context);
  vm.runInContext(code, context);
  await context.onmessage({ data: message });
  return received;
}

const wasm = readFileSync(`${repo}/exemples/lecons/69-compter.wasm`);
const copy = () => wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength);
// 1. Le message tel que la page l'envoie aujourd'hui.
const asSent = await run({ bytes: copy(), entry: 100, pages: 16 });
console.log("message de la page (bytes, entry) →", JSON.stringify(asSent));
console.log("  la page lit data.start :", asSent.some((m) => m.start), "; result.output :", asSent.find((m) => "output" in m)?.output);
// 2. Le message avec les clés que la boîte attend (octets, entree) : le module lui-même marche-t-il ?
const asExpected = await run({ octets: copy(), entree: 100, pages: 16 });
console.log("message avec octets, entree →", JSON.stringify(asExpected));
