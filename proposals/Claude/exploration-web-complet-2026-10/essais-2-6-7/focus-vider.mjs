// Piste 6 : le bouton « Vider le panier » est rangé dans If(count, over: 0) ; une fois touché, il
// disparaît. Où va le focus ? Chrome sans fenêtre, profil jetable dans ce dossier, serveur local 8080.
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { join } from "node:path";
const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const port = 9900 + Math.floor(Math.random() * 90);
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const browser = spawn("C:/Program Files/Google/Chrome/Application/chrome.exe", ["--headless=new", "--no-first-run", "--enable-unsafe-swiftshader", `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(here, "tmp", "profil-"))}`, "about:blank"], { stdio: "ignore" });
try {
  let tab;
  for (let i = 0; i < 50 && !tab; i++) { await pause(200); try { tab = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((o) => o.type === "page"); } catch {} }
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((ok, ko) => { ws.onopen = ok; ws.onerror = ko; });
  let id = 0; const waiting = new Map();
  ws.onmessage = (m) => { const r = JSON.parse(m.data); waiting.get(r.id)?.(r); };
  const ask = (method, params = {}) => new Promise((ok) => { waiting.set(++id, ok); ws.send(JSON.stringify({ id, method, params })); });
  const evaluate = async (expression) => (await ask("Runtime.evaluate", { expression, awaitPromise: true })).result?.result?.value;
  const ENTER = String.fromCharCode(13);
  const enter = async () => { await ask("Input.dispatchKeyEvent", { type: "keyDown", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13, text: ENTER, unmodifiedText: ENTER }); await ask("Input.dispatchKeyEvent", { type: "keyUp", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13 }); };
  const focused = `(() => { const a = document.activeElement; return a ? a.tagName.toLowerCase() + (a.dataset?.name ? "[" + a.dataset.name + "]" : "") : "rien"; })()`;
  await ask("Page.navigate", { url: "http://localhost:8080/exemples/site-reference/panier.holo" });
  await pause(3000);
  await evaluate(`document.querySelector('[data-name="PlusLever"]').focus()`);
  await enter();
  await pause(3000); // le moteur arrive au premier geste
  console.log("Après « + » : texte du panier →", await evaluate(`[...document.querySelectorAll(".holo-s-prix")].map((t) => t.textContent).find((t) => t.includes("créations")) ?? "?"`));
  await evaluate(`document.querySelector('[data-name="Vider"]').focus()`);
  console.log("Focus avant « Vider le panier » →", await evaluate(focused));
  await enter();
  await pause(800);
  console.log("Après « Vider le panier » : bouton visible ? →", await evaluate(`document.querySelector('[data-name="Vider"]').checkVisibility()`), "; focus →", await evaluate(focused));
  ws.close();
} finally { browser.kill(); }
