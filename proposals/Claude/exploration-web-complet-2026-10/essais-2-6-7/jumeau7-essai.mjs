// Le jumeau HTML de la piste 7 : le choix lu et récrit dans l'adresse ; le lien d'évitement.
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const port = 9600 + Math.floor(Math.random() * 90);
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const browser = spawn("C:/Program Files/Google/Chrome/Application/chrome.exe", ["--headless=new", "--no-first-run", `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(here, "tmp", "profil-"))}`, "about:blank"], { stdio: "ignore" });
try {
  let tab;
  for (let i = 0; i < 50 && !tab; i++) { await pause(200); try { tab = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((o) => o.type === "page"); } catch {} }
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((ok, ko) => { ws.onopen = ok; ws.onerror = ko; });
  let id = 0; const waiting = new Map();
  ws.onmessage = (m) => { const r = JSON.parse(m.data); waiting.get(r.id)?.(r); };
  const ask = (method, params = {}) => new Promise((ok) => { waiting.set(++id, ok); ws.send(JSON.stringify({ id, method, params })); });
  const evaluate = async (expression) => (await ask("Runtime.evaluate", { expression, awaitPromise: true })).result?.result?.value;
  const press = async (key, code, keyCode, text) => { await ask("Input.dispatchKeyEvent", { type: "keyDown", key, code, windowsVirtualKeyCode: keyCode, ...(text ? { text, unmodifiedText: text } : {}) }); await ask("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: keyCode }); };
  await ask("Page.navigate", { url: pathToFileURL(join(here, "piste-07-jumeau.html")).href + "?format=Petit" });
  await pause(1000);
  console.log("7 · ?format=Petit : coché →", await evaluate(`document.querySelector("input[name=format]:checked")?.value`));
  await evaluate(`document.querySelector("input[value=Grand]").click()`);
  console.log("7 · après « Grand » : adresse →", await evaluate(`location.search`));
  await press("Tab", "Tab", 9);
  console.log("7 · premier Tab →", await evaluate(`document.activeElement.textContent`));
  await press("Enter", "Enter", 13, String.fromCharCode(13));
  await pause(200);
  console.log("7 · Entrée sur le lien d'évitement : focus →", await evaluate(`document.activeElement.tagName + "#" + document.activeElement.id`));
  ws.close();
} finally { browser.kill(); }
