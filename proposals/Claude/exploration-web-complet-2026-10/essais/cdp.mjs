// Petit pilote de Chrome sans fenêtre, par le protocole de DevTools (comme moteur/outils/capture.mjs).
// Fichier d'essai de l'exploration : il n'écrit que dans le dossier d'essais.
import { spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { join } from "node:path";

const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
export const pause = (ms) => new Promise((r) => setTimeout(r, ms));

export async function openChrome() {
  const chrome = process.env.CHROME ?? "C:/Program Files/Google/Chrome/Application/chrome.exe";
  const port = 9800 + Math.floor(Math.random() * 150);
  const profile = mkdtempSync(join(here, "profil-chrome-"));
  const browser = spawn(chrome, [
    "--headless=new", "--enable-unsafe-swiftshader", "--hide-scrollbars", "--no-first-run",
    "--autoplay-policy=user-gesture-required", `--remote-debugging-port=${port}`,
    `--user-data-dir=${profile}`, "about:blank",
  ], { stdio: "ignore" });
  let tab;
  for (let i = 0; i < 60 && !tab; i++) {
    await pause(200);
    try { tab = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((o) => o.type === "page"); } catch { /* pas encore */ }
  }
  if (!tab) throw new Error("Chrome n'a pas démarré");
  const ws = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((ok, ko) => { ws.onopen = ok; ws.onerror = ko; });
  let n = 0;
  const waiting = new Map();
  const listeners = new Map();
  ws.onmessage = (m) => {
    const r = JSON.parse(m.data);
    if (r.id) waiting.get(r.id)?.(r);
    else for (const f of listeners.get(r.method) ?? []) f(r.params);
  };
  const ask = (method, params = {}) => new Promise((ok) => { waiting.set(++n, ok); ws.send(JSON.stringify({ id: n, method, params })); });
  const on = (method, f) => { if (!listeners.has(method)) listeners.set(method, []); listeners.get(method).push(f); };
  const close = async () => {
    try { ws.close(); } catch {}
    browser.kill();
    await pause(800);
    try { rmSync(profile, { recursive: true, force: true }); } catch {}
  };
  const evaluate = async (expression) => {
    const r = await ask("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
    if (r.result?.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400));
    return r.result?.result?.value;
  };
  return { ask, on, close, evaluate };
}
