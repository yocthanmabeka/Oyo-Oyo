// Les jumeaux HTML des pistes 2 et 6 font-ils ce qu'on dit ? Chrome sans fenêtre, fichiers ouverts
// depuis ce dossier (file://), profil jetable rangé ici. Lecture seule pour le dépôt.
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const port = 9700 + Math.floor(Math.random() * 90);
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
  const ENTER = String.fromCharCode(13);

  // Piste 6.
  await ask("Page.navigate", { url: pathToFileURL(join(here, "piste-06-jumeau.html")).href });
  await pause(1000);
  await evaluate(`document.querySelector("[popovertarget]").click()`);
  console.log("6 · menu ouvert après le bouton ? →", await evaluate(`document.getElementById("site-menu").matches(":popover-open")`));
  await press("Escape", "Escape", 27);
  await pause(200);
  console.log("6 · menu ouvert après Échap ? →", await evaluate(`document.getElementById("site-menu").matches(":popover-open")`));
  await evaluate(`document.querySelectorAll("details")[0].open = true; document.querySelectorAll("details")[1].open = true; ""`);
  await pause(200);
  console.log("6 · plis ouverts après avoir ouvert le second →", await evaluate(`[...document.querySelectorAll("details")].map((d) => d.open).join(",")`));
  await evaluate(`document.querySelector("input[value=Grand]").click()`);
  await pause(300);
  console.log("6 · note visible après « Grand » →", await evaluate(`!document.getElementById("note").hidden`), "; annonce →", JSON.stringify(await evaluate(`document.getElementById("live").textContent`)));
  await evaluate(`document.getElementById("ask").click()`);
  await pause(200);
  await press("Escape", "Escape", 27);
  await pause(300);
  console.log("6 · après Échap dans la fenêtre : bouton caché ? →", await evaluate(`document.getElementById("ask").hidden`), "; focus →", await evaluate(`document.activeElement.tagName`));

  // Piste 2 : la vérification, sans serveur (rien ne doit partir quand un champ est faux).
  await ask("Page.navigate", { url: pathToFileURL(join(here, "piste-02-jumeau.html")).href });
  await pause(1000);
  await evaluate(`window.__fetches = 0; const f = window.fetch; window.fetch = (...a) => { window.__fetches++; return f(...a); }; ""`);
  await evaluate(`document.getElementById("name").focus()`);
  await press("Enter", "Enter", 13, ENTER);
  await pause(300);
  console.log("2 · Entrée, tout vide : envois →", await evaluate("window.__fetches"), "; focus →", await evaluate(`document.activeElement.id`), "; erreurs →", JSON.stringify(await evaluate(`[...document.querySelectorAll(".error")].map((e) => e.id + " : " + e.textContent).join(" | ")`)));
  await evaluate(`const n = document.getElementById("name"); n.value = "Ada"; n.dispatchEvent(new Event("input", { bubbles: true })); const m = document.getElementById("mail"); m.value = "ada@"; m.dispatchEvent(new Event("input", { bubbles: true })); document.querySelectorAll("input[name=delivery]")[1].click(); ""`);
  await evaluate(`document.querySelector("#booking button").click()`);
  await pause(300);
  console.log("2 · courriel « ada@ » : envois →", await evaluate("window.__fetches"), "; focus →", await evaluate(`document.activeElement.id`), "; erreurs →", JSON.stringify(await evaluate(`[...document.querySelectorAll(".error")].map((e) => e.textContent).join(" | ")`)), "; adresse cachée ? →", await evaluate(`document.getElementById("address-row").hidden`));
  // (un bloc : « const m » existe déjà dans la page depuis l'étape précédente)
  await evaluate(`{ const m = document.getElementById("mail"); m.value = "ada@exemple.fr"; m.dispatchEvent(new Event("input", { bubbles: true })); } ""`);
  await evaluate(`document.querySelector("#booking button").click()`);
  await pause(1500);
  console.log("2 · tout juste (sans serveur, en file://) : envois →", await evaluate("window.__fetches"), "; message d'échec visible ? →", await evaluate(`!document.getElementById("failed").hidden`));
  ws.close();
} finally { browser.kill(); }
