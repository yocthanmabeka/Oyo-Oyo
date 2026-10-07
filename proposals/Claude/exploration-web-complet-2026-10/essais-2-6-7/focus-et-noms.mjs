// Pistes 6 et 7 : ce que Chrome expose vraiment (nom d'une fenêtre, focus après un passage).
// Lecture seule : un Chrome sans fenêtre, un profil jetable rangé dans ce dossier d'essai, le
// serveur local déjà lancé (8080). Rien n'est écrit dans le dépôt.
//
//     node focus-et-noms.mjs
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { join } from "node:path";

const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const chrome = "C:/Program Files/Google/Chrome/Application/chrome.exe";
const port = 9800 + Math.floor(Math.random() * 100);
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const browser = spawn(chrome, ["--headless=new", "--no-first-run", "--enable-unsafe-swiftshader", `--remote-debugging-port=${port}`,
  `--user-data-dir=${mkdtempSync(join(here, "tmp", "profil-"))}`, "about:blank"], { stdio: "ignore" });
try {
  let tab;
  for (let i = 0; i < 50 && !tab; i++) {
    await pause(200);
    try { tab = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((o) => o.type === "page"); } catch {}
  }
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((ok, ko) => { ws.onopen = ok; ws.onerror = ko; });
  let id = 0;
  const waiting = new Map();
  ws.onmessage = (m) => { const r = JSON.parse(m.data); waiting.get(r.id)?.(r); };
  const ask = (method, params = {}) => new Promise((ok) => { waiting.set(++id, ok); ws.send(JSON.stringify({ id, method, params })); });
  const evaluate = async (expression) => (await ask("Runtime.evaluate", { expression, awaitPromise: true })).result?.result?.value;
  // `text` fait naître l'appui (keypress) : sans lui, Entrée n'active pas un bouton.
  const ENTER = String.fromCharCode(13);
  const key = async (k, code, keyCode, text) => {
    await ask("Input.dispatchKeyEvent", { type: "keyDown", key: k, code, windowsVirtualKeyCode: keyCode, ...(text ? { text, unmodifiedText: text } : {}) });
    await ask("Input.dispatchKeyEvent", { type: "keyUp", key: k, code, windowsVirtualKeyCode: keyCode });
  };
  const focused = `(() => { const a = document.activeElement; return a ? a.tagName.toLowerCase() + (a.dataset?.name ? "[" + a.dataset.name + "]" : "") + (a.textContent ? " « " + a.textContent.trim().slice(0, 40) + " »" : "") : "rien"; })()`;
  await ask("Accessibility.enable");

  // 1. Leçon 63 : la fenêtre a-t-elle un nom pour un lecteur d'écran ? Où revient le focus ?
  await ask("Page.navigate", { url: "http://localhost:8080/exemples/lecons/63-fenetre.holo" });
  await pause(2500);
  await evaluate(`document.querySelector('[data-name="Demander"]').focus()`);
  await key("Enter", "Enter", 13, ENTER);
  await pause(3000); // le moteur arrive au premier geste, puis la fenêtre s'ouvre
  console.log("Leçon 63, après Entrée sur « Vider le panier » : fenêtre ouverte ? →", await evaluate(`document.querySelector("dialog").open`));
  const tree = await ask("Accessibility.getFullAXTree");
  const dialogs = (tree.result?.nodes ?? []).filter((n) => n.role?.value === "dialog").map((n) => ({ role: n.role.value, nom: n.name?.value ?? "", ignore: !!n.ignored }));
  console.log("Leçon 63 : nœuds « dialog » de l'arbre d'accessibilité →", JSON.stringify(dialogs));
  console.log("Leçon 63, focus pendant que la fenêtre est ouverte →", await evaluate(focused));
  const closeLabel = await evaluate(`document.querySelector("dialog .holo-close button")?.getAttribute("aria-label")`);
  console.log("Leçon 63, nom du bouton de fermeture →", closeLabel);
  await key("Escape", "Escape", 27);
  await pause(500);
  console.log("Leçon 63, après Échap : fenêtre ouverte ? →", await evaluate(`document.querySelector("dialog").open`), "; focus →", await evaluate(focused));

  // 2. Leçon 7 : après « Entrer dans l'atelier » au clavier, où est le focus ? Qu'annonce-t-on ?
  await ask("Page.navigate", { url: "http://localhost:8080/exemples/lecons/07-point-et-monde.holo" });
  await pause(2500);
  await evaluate(`document.querySelector('[data-name="Entrer"]').focus()`);
  await key("Enter", "Enter", 13, ENTER);
  await pause(4000);
  console.log("Leçon 7, après l'entrée dans le monde : titre →", await evaluate("document.title"), "; adresse →", await evaluate("location.hash"), "; focus →", await evaluate(focused), "; annonce →", JSON.stringify(await evaluate(`document.getElementById("announcement")?.textContent ?? ""`)));
  ws.close();
} finally {
  browser.kill();
}
