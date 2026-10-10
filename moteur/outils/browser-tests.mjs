// Les essais dans un vrai navigateur : Chrome sans fenêtre, piloté par son protocole (DevTools),
// sans rien installer. Le serveur d'essai est lancé sur un port libre ; chaque essai ouvre une
// page, joue des gestes comme une main (souris, clavier, doigts) et vérifie ce qui doit se passer.
// Les tests du moteur en Rust ne voient pas la page : le pincement, les modules et la touche
// Échap ont cassé le 2026-10-07 alors qu'ils restaient verts.
//
//     node outils/browser-tests.mjs              (depuis moteur/)
//     node outils/browser-tests.mjs pinch        (seulement les essais dont le nom contient « pinch »)
//
// Il faut le moteur construit (web/pkg, web/pkg-light, target/release/holo) et Chrome : CHROME
// donne son chemin ; sinon l'emplacement habituel sous Windows, ou google-chrome sous Linux.
// Rend « OK » ou « RATÉ » par essai, et un code de sortie 1 s'il y a un raté.

import { accountDebtTests } from "../../proposals/GPT5.6/fin-comptes-2026-10-08/browser-tests.mjs";
import { passkeyTests } from "../../proposals/GPT5.6/fin-passkeys-2026-10-08/browser-tests.mjs";
import { sharingTests } from "../../proposals/GPT5.6/fin-partage-2026-10-08/browser-tests.mjs";
import { spawn, spawnSync } from "node:child_process";
import { createHmac } from "node:crypto";
import { createServer } from "node:http";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { webTests } from "../../proposals/GPT5.6/web-viable-2026-10-08/browser-tests.mjs";
import { capabilityTests } from "../../proposals/GPT5.6/fin-lot9-2026-10-08/browser-tests.mjs";

const engine = fileURLToPath(new URL("..", import.meta.url));
const repo = resolve(engine, "..");
// `--telephone` : le Chrome du téléphone de Yocthan, par le câble. Avant :
//   adb reverse tcp:8080 tcp:8080 ; adb forward tcp:9222 localabstract:chrome_devtools_remote ;
//   adb shell am start -a android.intent.action.VIEW -d http://localhost:8080/stack com.android.chrome
// Les pages viennent alors du serveur 8080 du PC (le code de main), et l'outil ne pilote que
// l'onglet ouvert sur localhost:8080 : jamais les autres onglets du téléphone.
const phone = process.argv.includes("--telephone");
const only = process.argv.slice(2).find((a) => !a.startsWith("--")) ?? "";
const pause = (ms) => new Promise((r) => setTimeout(r, ms));

function findChrome() {
  const candidates = [
    process.env.CHROME,
    "C:/Program Files/Google/Chrome/Application/chrome.exe",
    "C:/Program Files (x86)/Google/Chrome/Application/chrome.exe",
  ];
  for (const c of candidates) if (c && existsSync(c)) return c;
  for (const name of ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser"]) {
    const found = spawnSync("which", [name], { encoding: "utf8" });
    if (found.status === 0 && found.stdout.trim()) return found.stdout.trim();
  }
  throw new Error("Chrome introuvable : donner son chemin dans CHROME");
}

// Le serveur d'essai, sur un port libre ; on attend qu'il annonce son adresse.
async function startServer() {
  if (phone) {
    const reached = await fetch("http://localhost:8080/exemples/lecons/01-page.holo").then((r) => r.ok, () => false);
    if (!reached) throw new Error("le serveur 8080 du PC ne répond pas : node outils/server.mjs, avec PORT=8080");
    return { base: "http://localhost:8080", log: () => "", stop() {} };
  }
  const port = 18000 + Math.floor(Math.random() * 2000);
  const env = { ...process.env, PORT: String(port) };
  delete env.HOLO_KEY;
  delete env.HOLO_REPO;
  const server = spawn(process.execPath, ["outils/server.mjs"], { cwd: engine, env, stdio: ["ignore", "pipe", "pipe"] });
  let output = "";
  server.stdout.on("data", (d) => { output += d; });
  server.stderr.on("data", (d) => { output += d; });
  for (let i = 0; i < 100 && !output.includes(`localhost:${port}`); i++) await pause(100);
  if (!output.includes(`localhost:${port}`)) throw new Error(`le serveur d'essai ne démarre pas :\n${output}`);
  return { base: `http://localhost:${port}`, log: () => output, stop: () => server.kill() };
}

// holo serve (ADR-074), sur un dossier d'essai à part : sa base ne salit pas le dépôt.
async function startHoloServe(files) {
  const folder = mkdtempSync(join(tmpdir(), "holo-serve-"));
  for (const file of files) {
    // Une page rangée dans un dossier (la suite d'une leçon) : son dossier d'abord.
    mkdirSync(join(folder, file, ".."), { recursive: true });
    writeFileSync(join(folder, file), readFileSync(join(repo, "exemples", "lecons", file)));
  }
  const port = 20000 + Math.floor(Math.random() * 2000);
  const binary = ["holo", "holo.exe"].map((name) => join(engine, "target", "release", name)).find(existsSync);
  const server = spawn(binary, ["serve", folder, String(port)], { cwd: engine, stdio: ["ignore", "pipe", "pipe"] });
  let output = "";
  server.stdout.on("data", (d) => { output += d; });
  server.stderr.on("data", (d) => { output += d; });
  for (let i = 0; i < 100 && !output.includes(`localhost:${port}`); i++) await pause(100);
  if (!output.includes(`localhost:${port}`)) throw new Error(`holo serve ne démarre pas :\n${output}`);
  // Sous Windows, la base reste prise un instant après l'arrêt du serveur : l'effacement réessaie.
  return { base: `http://localhost:${port}`, folder, log: () => output, stop: () => { server.kill(); try { rmSync(folder, { recursive: true, force: true, maxRetries: 20, retryDelay: 100 }); } catch { /* le dossier temporaire restera */ } } };
}

// Lance Chrome sans fenêtre. Il choisit lui-même un port libre (`--remote-debugging-port=0`)
// et l'écrit dans son dossier, dans le fichier DevToolsActivePort : pas de port qui se heurte
// à un autre. On attend jusqu'à une minute ; ce qu'il dit est gardé pour comprendre un échec.
async function launchChrome() {
  const profile = mkdtempSync(join(tmpdir(), "holo-essais-"));
  const args = [
    "--headless=new", "--no-first-run", "--no-default-browser-check", "--enable-unsafe-swiftshader",
    "--autoplay-policy=no-user-gesture-required", "--hide-scrollbars", "--remote-debugging-port=0",
    "--window-size=1000,700", `--user-data-dir=${profile}`, "about:blank",
  ];
  // Sur les machines de GitHub, le bac à sable de Chrome n'a pas les droits du système.
  if (process.env.CI) args.unshift("--no-sandbox");
  const chrome = spawn(findChrome(), args, { stdio: ["ignore", "ignore", "pipe"] });
  let said = "";
  chrome.stderr.on("data", (chunk) => { said = (said + chunk).slice(-4000); });
  let target;
  let port;
  for (let i = 0; i < 300 && !target && chrome.exitCode === null; i++) {
    await pause(200);
    try { port = readFileSync(join(profile, "DevToolsActivePort"), "utf8").split("\n")[0].trim(); } catch { continue; }
    try { target = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((t) => t.type === "page"); } catch { /* pas encore */ }
  }
  if (!target) chrome.kill();
  return { chrome, profile, target, port, said: said.trim() || "(rien)" };
}

// Le Chrome du téléphone : l'onglet ouvert sur localhost:8080 (par adb shell am start).
async function connectPhone() {
  const tabs = await (await fetch("http://127.0.0.1:9222/json")).json().catch(() => []);
  const target = tabs.find((t) => t.type === "page" && t.url.startsWith("http://localhost:8080/"));
  if (!target) throw new Error("aucun onglet sur localhost:8080 dans le Chrome du téléphone : adb shell am start -a android.intent.action.VIEW -d http://localhost:8080/stack com.android.chrome");
  const version = await (await fetch("http://127.0.0.1:9222/json/version")).json();
  console.log(`Le téléphone : ${version.Browser} (${version["User-Agent"]})`);
  return { target, port: 9222 };
}

async function startChrome() {
  let launched = phone ? await connectPhone() : await launchChrome();
  // Sur une machine de GitHub qui vient de démarrer, Chrome tarde parfois : un second essai.
  if (!launched.target) {
    console.log(`Chrome ne répond pas ; second essai. Ce qu'il a dit : ${launched.said}`);
    launched = await launchChrome();
  }
  if (!launched.target) throw new Error(`Chrome ne démarre pas. Ce qu'il a dit : ${launched.said}`);
  const { chrome, profile, target, port } = launched;
  const first = await session(target);
  return {
    ...first,
    // Un autre onglet du même Chrome (les mêmes cookies) : deux pages ouvertes à la fois (ADR-079).
    async tab() {
      const created = await (await fetch(`http://127.0.0.1:${port}/json/new?about:blank`, { method: "PUT" })).json();
      const tab = await session(created);
      return { ...tab, async close() { tab.close(); await fetch(`http://127.0.0.1:${port}/json/close/${created.id}`).catch(() => {}); } };
    },
    stop() {
      first.close();
      // Le Chrome du téléphone reste ouvert : on ne fait que s'en détacher.
      if (!chrome) return;
      chrome.kill();
      // Chrome garde son dossier un instant après s'être arrêté.
      setTimeout(() => { try { rmSync(profile, { recursive: true, force: true }); } catch { /* tant pis */ } }, 1500);
    },
  };
}

// Une session du protocole avec un onglet : lui envoyer une commande, l'écouter.
async function session(target) {
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((ok, ko) => { ws.onopen = ok; ws.onerror = ko; });
  let n = 0;
  const waiting = new Map();
  const errors = [];
  // Ce que Chrome annonce de lui-même (une demande retenue, par exemple), pour un essai qui l'écoute.
  const listeners = new Map();
  ws.onmessage = (m) => {
    const r = JSON.parse(m.data);
    if (r.method === "Runtime.exceptionThrown") {
      const d = r.params.exceptionDetails;
      errors.push((d.exception?.description ?? d.text ?? "erreur").split("\n")[0]);
    }
    if (r.method) listeners.get(r.method)?.(r.params);
    waiting.get(r.id)?.(r);
  };
  const send = (method, params = {}) => new Promise((ok) => { waiting.set(++n, ok); ws.send(JSON.stringify({ id: n, method, params })); });
  await send("Runtime.enable");
  await send("Page.enable");
  await send("Accessibility.enable");
  return {
    send,
    errors,
    on(method, handler) {
      if (handler) listeners.set(method, handler);
      else listeners.delete(method);
    },
    close: () => ws.close(),
  };
}

// Les gestes et les questions à la page.
function page(browser, base) {
  const { send } = browser;
  const value = async (expression) => {
    const r = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
    if (r.result?.exceptionDetails) throw new Error(`la question à la page a échoué : ${expression}`);
    return r.result?.result?.value;
  };
  const p = {
    value,
    async open(path, wait = 1200) {
      browser.errors.length = 0;
      await send("Page.navigate", { url: base + path });
      for (let i = 0; i < 50; i++) {
        await pause(100);
        if ((await value("document.readyState").catch(() => "")) === "complete") break;
      }
      await pause(wait);
    },
    // Attend qu'une condition soit vraie dans la page (le moteur arrive en quelques secondes).
    async until(expression, timeout = 40000) {
      const end = Date.now() + timeout;
      while (Date.now() < end) {
        if (await value(`(() => { try { return Boolean(${expression}); } catch { return false; } })()`).catch(() => false)) return true;
        await pause(250);
      }
      return false;
    },
    async click(selector) {
      const box = await value(`(() => { const e = document.querySelector(${JSON.stringify(selector)}); if (!e) return null; e.scrollIntoView({ block: "center" }); const b = e.getBoundingClientRect(); return [b.left + b.width / 2, b.top + b.height / 2]; })()`);
      if (!box) throw new Error(`rien à toucher : ${selector}`);
      for (const type of ["mousePressed", "mouseReleased"]) await send("Input.dispatchMouseEvent", { type, x: box[0], y: box[1], button: "left", clickCount: 1 });
      await pause(150);
    },
    async key(key, code, keyCode, text) {
      await send("Input.dispatchKeyEvent", { type: "keyDown", key, code, windowsVirtualKeyCode: keyCode, text });
      await send("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: keyCode });
      await pause(200);
    },
    async type(selector, text) {
      await value(`document.querySelector(${JSON.stringify(selector)}).focus()`);
      await send("Input.insertText", { text });
      await pause(200);
    },
    text: () => value("document.getElementById('page').innerText"),
  };
  return p;
}

// Le code à 6 chiffres (ADR-081), calculé ici comme le fait l'application d'authentification du
// visiteur : la clé lue sur la page (base 32, RFC 4648), puis la RFC 6238 (HMAC-SHA-1, 30 secondes).
function keyBytes(key) {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
  const out = [];
  let value = 0, bits = 0;
  for (const letter of key.replace(/[\s=]/g, "").toUpperCase()) {
    value = ((value << 5) | alphabet.indexOf(letter)) & 0xffff;
    bits += 5;
    if (bits >= 8) { bits -= 8; out.push((value >>> bits) & 255); }
  }
  return Buffer.from(out);
}
function totp(key, step) {
  const counter = Buffer.alloc(8);
  counter.writeBigUInt64BE(BigInt(step));
  const print = createHmac("sha1", keyBytes(key)).update(counter).digest();
  return String((print.readUInt32BE(print[19] & 15) & 0x7fffffff) % 1e6).padStart(6, "0");
}
const stepNow = () => Math.floor(Date.now() / 30000);
// Un code qui n'est celui d'aucun pas proche : le serveur doit le refuser.
function wrongCode(key) {
  const near = [-2, -1, 0, 1, 2].map((d) => totp(key, stepNow() + d));
  for (let n = 1; ; n++) {
    const code = String((Number(near[0]) + n * 7919) % 1e6).padStart(6, "0");
    if (!near.includes(code)) return code;
  }
}

// Ce que dit un serveur des pages qui écoutent une adresse en direct (ADR-079) : sa dernière
// ligne « Direct : …, 2 page(s) à l'écoute » pour cette adresse ; et l'attente d'un nombre.
function lastListening(log, address) {
  return log.trim().split("\n").filter((line) => line.startsWith("Direct") && line.includes(address)).at(-1) ?? "(rien)";
}
async function listeners(log, address, count, timeout) {
  const end = Date.now() + timeout;
  while (Date.now() < end) {
    if (lastListening(log(), address).endsWith(`, ${count} page(s) à l'écoute`)) return true;
    await pause(200);
  }
  return false;
}

// Les leçons qui s'ouvrent seules : une page ou un monde (ni un morceau, ni un thème).
function lessons() {
  const folder = join(repo, "exemples", "lecons");
  return readdirSync(folder).filter((f) => f.endsWith(".holo")).sort().filter((f) => {
    const source = readFileSync(join(folder, f), "utf8").replace(/\/\/.*$/gm, "");
    return /^\s*((import|module)\s[^\n]*\n\s*)*(Page|Point)\s*\(/.test(source);
  });
}

const tests = [
  ["toutes les leçons s'ouvrent, sans erreur", async (p, b) => {
    const faults = [];
    for (const file of lessons()) {
      await p.open(`/exemples/lecons/${file}`, 600);
      const title = await p.value("document.title");
      if (b.errors.length) faults.push(`${file} : ${b.errors.join(" | ")}`);
      else if (!title || title === "HoloCode") faults.push(`${file} : pas de titre`);
      // Sur un téléphone, une page ne déborde pas de l'écran (on ne glisse pas de côté).
      else if (phone) {
        const [wide, screen] = await p.value("[document.documentElement.scrollWidth, innerWidth]");
        if (wide > screen + 1) faults.push(`${file} : déborde (${wide} px pour un écran de ${screen} px)`);
      }
    }
    return [faults.length === 0, faults.length ? faults.join("\n      ") : `${lessons().length} leçons${phone ? ", aucune ne déborde de l'écran" : ""}`];
  }],
  ["le moteur arrive au premier geste", async (p) => {
    await p.open("/exemples/lecons/01-page.holo");
    await p.click("#toggle");
    const ok = await p.until(`document.getElementById("tools") && !document.getElementById("tools").hidden`);
    return [ok, ok ? "les outils s'ouvrent" : "les outils ne s'ouvrent pas"];
  }],
  ["pincer à deux doigts grossit la page (pinch)", async (p, b) => {
    if (!phone) {
      await b.send("Emulation.setDeviceMetricsOverride", { width: 400, height: 800, deviceScaleFactor: 2, mobile: true });
      await b.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
    }
    try {
      await p.open("/exemples/lecons/09-zoom-et-points.holo");
      const fingers = (gap) => [{ x: 200 - gap, y: 300, id: 1 }, { x: 200 + gap, y: 300, id: 2 }];
      let zoomed = false;
      for (let i = 0; i < 40 && !zoomed; i++) {
        await b.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: fingers(40) });
        for (let g = 50; g <= 160; g += 10) await b.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: fingers(g) });
        await b.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
        await pause(800);
        zoomed = await p.value(`(() => { const m = document.querySelector("#page main, #page .holo-Page"); return document.body.classList.contains("in-points") || (m && getComputedStyle(m).transform !== "none"); })()`);
      }
      return [zoomed && b.errors.length === 0, zoomed ? (b.errors.length ? b.errors.join(" | ") : "la page grossit") : `aucun zoom ; ${b.errors.join(" | ") || "aucune erreur"}`];
    } finally {
      if (!phone) {
        await b.send("Emulation.setTouchEmulationEnabled", { enabled: false });
        await b.send("Emulation.clearDeviceMetricsOverride");
      }
    }
  }],
  ["un module reçoit une liste et rend trois valeurs ; une réponse non annoncée est refusée (leçon 97)", async (p) => {
    await p.open("/exemples/lecons/97-un-module-qui-recoit-une-liste.holo");
    await p.click('[data-name="Calculer"]');
    const computed = await p.until(`document.getElementById("page").innerText.includes("3 notes ; moyenne : 13,5 ; la meilleure : Maths.")`, 40000);
    // Une note de plus, avec des signes qui pourraient tromper un lecteur de JSON.
    await p.type('[data-bind="matiere"]', 'Arts "plastiques" {x}');
    await p.type('[data-bind="note"]', "18");
    await p.click('[data-name="Ajouter"]');
    await p.click('[data-name="Calculer"]');
    const again = await p.until(`document.getElementById("page").innerText.includes('4 notes ; moyenne : 14,6 ; la meilleure : Arts "plastiques" {x}.')`, 10000);
    const before = await p.text();
    await p.click('[data-name="Mentir"]');
    const refused = await p.until(`document.getElementById("page").innerText.includes("Refusé : ce module a rendu une valeur")`, 10000);
    const after = await p.text();
    const reason = await p.value(`(window.__holoModules ?? []).filter((m) => m.name === "Menteur").map((m) => m.reason).join(" | ")`);
    const unchanged = after.includes("4 notes ; moyenne : 14,6");
    return [computed && again && refused && unchanged && reason.includes("admin"), `calculé : ${computed} ; avec une note de plus : ${again} ; refusé : ${refused} (${reason}) ; rien changé : ${unchanged}${before ? "" : ""}`];
  }],
  ["un dessin en SVG, lu par le lecteur d'écran, dont une forme suit une valeur (leçon 98)", async (p, b) => {
    await p.open("/exemples/lecons/98-un-dessin.holo");
    const svg = await p.value(`(() => { const s = document.querySelector("svg.holo-Drawing"); return s && s.getAttribute("role") + "|" + s.getAttribute("aria-label") + "|" + s.querySelectorAll("rect,circle,line,path").length; })()`);
    const { result } = await b.send("Accessibility.getFullAXTree");
    const named = result.nodes.some((n) => ["image", "img"].includes(n.role?.value) && n.name?.value === "Un paysage : une maison, une colline, et le soleil" && !n.ignored);
    const sun = () => p.value(`document.querySelector("svg.holo-Drawing circle").getAttribute("cy")`);
    const start = await sun();
    await p.click('[data-name="Lever"]');
    await p.until(`document.querySelector("svg.holo-Drawing circle").getAttribute("cy") === "50"`, 40000);
    await p.click('[data-name="Lever"]');
    const raised = await p.until(`document.querySelector("svg.holo-Drawing circle").getAttribute("cy") === "30"`, 5000);
    // Le dessin rétrécit avec l'écran, sans se déformer.
    const shape = await p.value(`(() => { const r = document.querySelector("svg.holo-Drawing").getBoundingClientRect(); return Math.round(r.width / r.height * 100) / 100; })()`);
    const ok = svg === "img|Un paysage : une maison, une colline, et le soleil|7" && named && start === "70" && raised && shape === 2;
    return [ok, `SVG : ${svg} ; nommé pour le lecteur d'écran : ${named} ; soleil ${start} → 30 : ${raised} ; proportions : ${shape}`];
  }],
  ["un tableau de bord : des données reçues, dessinées en barres et en parts, qui suivent la liste (leçon 99)", async (p) => {
    await p.open("/exemples/lecons/99-un-tableau-de-bord.holo");
    const count = (selector) => p.value(`document.querySelectorAll(${JSON.stringify(selector)}).length`);
    const bars = await count('.holo-Chart:first-of-type svg rect');
    const parts = await count('.holo-Chart:nth-of-type(2) svg path');
    const captions = await p.value(`[...document.querySelectorAll(".holo-Chart figcaption")].map((c) => c.textContent).join(" | ")`);
    await p.type('[data-bind="jour"]', "Samedi");
    await p.type('[data-bind="montant"]', "180");
    await p.click('[data-name="Ajouter"]');
    const grown = await p.until(`document.querySelectorAll(".holo-Chart:first-of-type svg rect").length === 6 && document.querySelectorAll(".holo-Chart:nth-of-type(2) svg path").length === 6`, 40000);
    // Les données du serveur ne reviennent pas effacer la vente ajoutée.
    await pause(1500);
    const kept = await p.value(`document.querySelectorAll(".holo-Chart:first-of-type svg rect").length === 6`);
    const table = await p.value(`document.querySelector(".holo-Chart table").textContent`);
    const hidden = await p.value(`(() => { const t = document.querySelector(".holo-Chart .holo-hidden").getBoundingClientRect(); return t.width <= 1 && t.height <= 1 && document.documentElement.scrollWidth <= innerWidth; })()`);
    const ok = bars === 5 && parts === 5 && captions === "Les ventes de la semaine, en euros | La part de chaque jour" && grown && kept && table.includes("Samedi180") && hidden;
    return [ok, `5 barres et 5 parts au départ : ${bars}, ${parts} ; titres : ${captions} ; une vente ajoutée, 6 et 6 : ${grown}, gardée : ${kept} ; tableau caché, « Samedi 180 » : ${table.includes("Samedi180")}, caché : ${hidden}`];
  }],
  ["un module qui dessine : il rend une liste de formes, le moteur les vérifie et les dessine (leçon 110)", async (p) => {
    await p.open("/exemples/lecons/110-un-module-qui-dessine.holo");
    const shapes = () => p.value(`document.querySelectorAll("svg.holo-Drawing .holo-shapes > *").length`);
    const before = await shapes();
    await p.click('[data-name="Dessiner"]');
    const six = await p.until(`document.querySelectorAll("svg.holo-Drawing .holo-shapes circle").length === 7`, 40000);
    await p.value(`(() => { const s = document.querySelector('[data-bind="petales"]'); s.value = "9"; s.dispatchEvent(new Event("input", { bubbles: true })); })()`);
    await p.click('[data-name="Dessiner"]');
    const nine = await p.until(`document.querySelectorAll("svg.holo-Drawing .holo-shapes circle").length === 10 && document.getElementById("page").innerText.includes("11 formes dessinées.")`, 10000);
    const ok = before === 0 && six && nine;
    return [ok, `avant : ${before} forme ; 6 pétales et le cœur : ${six} ; 9 pétales, « 11 formes dessinées » : ${nine}`];
  }],
  ["les secondes défilent, et un chronomètre au centième (leçons 48 et 111)", async (p) => {
    await p.open("/exemples/lecons/48-heure.holo");
    const second = () => p.value(`Number(document.querySelector('[data-state="second"]').textContent)`);
    await p.until("window.__holoStarted === true");
    const first = await second();
    const ticked = await p.until(`Number(document.querySelector('[data-state="second"]').textContent) !== ${first}`, 3000);
    await p.open("/exemples/lecons/111-un-chronometre.holo");
    const dial = () => p.value(`document.querySelector(".holo-Stopwatch").textContent`);
    const atRest = await dial();
    await p.click('[data-name="Partir"]');
    await p.until("window.__holoStarted === true");
    await pause(700);
    const running = await dial();
    await pause(300);
    const later = await dial();
    await p.click('[data-name="Stop"]');
    const stopped = await dial();
    await pause(400);
    const still = await dial();
    const written = await p.until(`document.getElementById("page").innerText.includes("Dernier temps : " + document.querySelector(".holo-Stopwatch").textContent)`, 3000);
    const timer = await p.value(`document.querySelector(".holo-Stopwatch").getAttribute("role")`);
    await p.click('[data-name="Zero"]');
    const reset = await dial();
    const ok = ticked && atRest === "00:00,00" && running !== atRest && later !== running && stopped === still && written && timer === "timer" && reset === "00:00,00";
    return [ok, `la seconde change : ${ticked} ; au repos ${atRest}, en marche ${running} puis ${later}, arrêté ${stopped} (fixe : ${stopped === still}) ; écrit dessous : ${written} ; role=${timer} ; remis à zéro : ${reset}`];
  }],
  ["un titre qui lit les valeurs, des valeurs gardées par adresse, narrow dans un Row (leçons 112 et 113)", async (p, b) => {
    const pages = () => p.value(`document.getElementById("page").innerText.match(/(\\d+) page\\(s\\)/)?.[1]`);
    await p.open("/exemples/lecons/112-carnets/ada", 300);
    await p.value(`localStorage.clear()`);
    await p.open("/exemples/lecons/112-carnets/ada");
    const titleAda = await p.value("document.title");
    await p.click('[data-name="Ecrire"]');
    await p.until(`document.getElementById("page").innerText.includes("1 page(s)")`, 40000);
    await p.click('[data-name="Ecrire"]');
    await p.until(`document.getElementById("page").innerText.includes("2 page(s)")`, 5000);
    const titleFollows = await p.until(`document.title === "Le carnet de ada : 2 page(s)"`, 3000);
    await p.open("/exemples/lecons/112-carnets/bob");
    const titleBob = await p.value("document.title");
    const bob = await pages();
    await p.open("/exemples/lecons/112-carnets/ada");
    await p.until("window.__holoStarted === true", 40000);
    const ada = await p.until(`document.getElementById("page").innerText.includes("2 page(s)")`, 5000);
    // Une case de Row de moins de 320px est « étroite » : sur un ordinateur non, sur un téléphone oui.
    await b.send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 800, deviceScaleFactor: 1, mobile: false });
    await p.open("/exemples/lecons/113-une-rangee-qui-se-serre.holo");
    const wide = await p.until(`[...document.querySelectorAll(".holo-s-carte")].length === 2 && ![...document.querySelectorAll(".holo-s-carte")].some((c) => c.classList.contains("holo-narrow"))`, 5000);
    await b.send("Emulation.setDeviceMetricsOverride", { width: 390, height: 800, deviceScaleFactor: 1, mobile: true });
    const narrow = await p.until(`[...document.querySelectorAll(".holo-s-carte")].every((c) => c.classList.contains("holo-narrow") && getComputedStyle(c).paddingTop === "8px")`, 5000);
    await b.send("Emulation.clearDeviceMetricsOverride");
    const ok = titleAda === "Le carnet de ada : 0 page(s)" && titleFollows && titleBob === "Le carnet de bob : 0 page(s)" && bob === "0" && ada && wide && narrow;
    return [ok, `titres : ${titleAda} / ${titleBob} ; le titre suit les pages : ${titleFollows} ; Bob : ${bob} page ; Ada garde ses 2 pages : ${ada} ; cartes larges sur ordinateur : ${wide} ; étroites sur téléphone : ${narrow}`];
  }],
  ["l'historique dans une page : address: [onglet, page] (leçon 114)", async (p) => {
    const lesson = "/exemples/lecons/114-l-historique-dans-une-page.holo";
    const shows = (text) => `document.getElementById("page").innerText.includes(${JSON.stringify(text)})`;
    const at = (search) => `location.search === ${JSON.stringify(search)}`;
    await p.open(lesson);
    // Chaque toucher qui change l'onglet ou la page fait un pas dans l'historique.
    await p.click('[data-name="Aquarelles"]');
    const tab = await p.until(`${at("?onglet=aquarelles")} && ${shows("Les aquarelles")}`, 40000);
    await p.click('[data-name="Apres"]');
    const next = await p.until(`${at("?onglet=aquarelles&page=2")} && ${shows("Page 2")}`, 5000);
    // « Précédent » défait un pas, puis l'autre ; « Suivant » le refait.
    await p.value("history.back()");
    const back = await p.until(`${at("?onglet=aquarelles")} && ${shows("Page 1")} && ${shows("Les aquarelles")}`, 5000);
    await p.value("history.back()");
    const start = await p.until(`${at("")} && ${shows("Les toiles")}`, 5000);
    await p.value("history.forward()");
    const again = await p.until(`${at("?onglet=aquarelles")} && ${shows("Les aquarelles")}`, 5000);
    // Une adresse partagée : la page fabriquée par le serveur a ses valeurs, avant le moteur ;
    // les réglages du moteur (?values) restent dans l'adresse.
    await p.open(`${lesson}?values&onglet=dessins&page=3`);
    const shared = await p.value(`${shows("Les dessins")} && ${shows("Page 3")}`);
    await p.click('[data-name="Apres"]');
    const kept = await p.until(`${at("?values&onglet=dessins&page=4")} && ${shows("Page 4")}`, 40000);
    // Une valeur mal écrite part de son départ, sans erreur.
    await p.open(`${lesson}?page=abc&onglet=pirate`);
    const forged = await p.until(`window.__holoStarted === true && ${shows("Page 1")} && ${shows("Les toiles")}`, 40000);
    const ok = tab && next && back && start && again && shared && kept && forged;
    return [ok, `onglet : ${tab} ; page suivante : ${next} ; précédent : ${back}, puis le début : ${start} ; suivant : ${again} ; adresse partagée : ${shared} ; ?values gardé : ${kept} ; valeurs forgées : ${forged}`];
  }],
  ["passer d'un fichier à l'autre : l'adresse et le chronomètre suivent la page", async (p) => {
    const folder = "/exemples/.essais-navigateur";
    const shows = (text) => `document.getElementById("page").innerText.includes(${JSON.stringify(text)})`;
    const dial = () => p.value(`document.querySelector(".holo-Stopwatch")?.textContent`);
    await p.open(`${folder}/passage-onglets.holo`);
    await p.click('[data-name="Aquarelles"]');
    const tab = await p.until(`location.search === "?onglet=aquarelles" && ${shows("Onglet aquarelles")}`, 40000);
    // Par un point, le moteur passe dans un autre fichier sans recharger la page ; on y lance le chronomètre.
    await p.click('[data-name="Suite"]');
    const there = await p.until(`location.pathname.endsWith("passage-chrono.holo") && !!document.querySelector('[data-name="Partir"]')`, 8000);
    await p.click('[data-name="Partir"]');
    // Revenue par un autre point, la page reprend ses valeurs, et l'adresse les dit encore.
    await p.click('[data-name="Retour"]');
    const reopened = await p.until(`location.pathname.endsWith("passage-onglets.holo") && location.search === "?onglet=aquarelles" && ${shows("Onglet aquarelles")}`, 8000);
    // Le chronomètre a tourné pendant ce temps ; revenu sur sa page, son cadran redessiné suit.
    await p.click('[data-name="Suite"]');
    await p.until(`location.pathname.endsWith("passage-chrono.holo") && !!document.querySelector(".holo-Stopwatch")`, 8000);
    const first = await dial();
    await pause(400);
    const later = await dial();
    const follows = first !== "00:00,00" && later !== first;
    const ok = tab && there && reopened && follows;
    return [ok, `onglet : ${tab} ; passé dans l'autre fichier : ${there} ; revenu, l'adresse dit l'onglet : ${reopened} ; le cadran suit : ${first} puis ${later}`];
  }],
  ["une page servie avec ses données, ouverte sur un endroit (#Bas) ou sur un nom inconnu, relit ses données", async (p, b) => {
    const has = (words) => `document.getElementById("page").innerText.includes(${JSON.stringify(words)})`;
    const opened = async (hash) => {
      // Une autre page d'abord : changer seulement le # ne recharge pas la page.
      await p.open("/exemples/lecons/01-page.holo");
      await p.open(`/exemples/.essais-navigateur/donnees-et-endroit.holo${hash}`);
      const started = await p.until("window.__holoStarted === true");
      const refused = await p.value(`document.getElementById("error")?.textContent ?? ""`);
      // Le navigateur ne rejoue pas la réception du serveur (l'adresse a un nom) : il relit.
      const read = await p.until(`${has("lectures 1")} && ${has("Trois")} && !${has("Chargement")}`, 8000);
      return [started && !refused && read, `${started && !refused ? "moteur arrivé" : `moteur arrêté ${refused}`}, données relues : ${read}`];
    };
    // Un vrai endroit (#Bas) ; puis un nom que rien ne porte (#rien), comme une ancre inconnue.
    const [spot, spotSaid] = await opened("#Bas");
    const [unknown, unknownSaid] = await opened("#rien");
    // Une adresse changée à la main vers un nom inconnu : la page reste où elle est.
    await p.value(`location.hash = "#ailleurs"`);
    await pause(500);
    const stays = b.errors.length === 0 && (await p.value(has("lectures 1")));
    const ok = spot && unknown && stays;
    return [ok, `#Bas : ${spotSaid} ; #rien : ${unknownSaid} ; #ailleurs tapé ensuite : ${stays ? "la page reste" : b.errors.join(" | ") || "la page a changé"}`];
  }],
  ["une valeur d'adresse donnée par les données reste, sur une adresse nue", async (p) => {
    const shows = (text) => `document.getElementById("page").innerText.includes(${JSON.stringify(text)})`;
    const page = "/exemples/.essais-navigateur/donnees-et-adresse.holo";
    // Sans « ? » : la valeur reçue (5), sur la page du serveur puis avec le moteur.
    await p.open(page);
    const served = await p.value(shows("Page 5"));
    const started = await p.until("window.__holoStarted === true");
    await pause(500);
    const kept = await p.value(shows("Page 5"));
    // Avec ?page=9 : l'adresse l'emporte, des deux côtés.
    await p.open("/exemples/lecons/01-page.holo");
    await p.open(`${page}?page=9`);
    const asked = await p.until(`window.__holoStarted === true && ${shows("Page 9")}`);
    const ok = served && started && kept && asked;
    return [ok, `servie : ${served} ; gardée avec le moteur : ${kept} ; ?page=9 : ${asked}`];
  }],
  ["des polices libres pour toutes les écritures : Font(family: \"Inter\") (leçon 115)", async (p) => {
    await p.open("/exemples/lecons/115-des-polices-pour-toutes-les-ecritures.holo");
    // Les polices du moteur arrivent ; pour le japonais, seulement les morceaux de la phrase.
    const loaded = await p.until(`document.fonts.check('16px "Noto Sans JP"', "アトリエへようこそ") && document.fonts.check('16px Literata', "Literata") && document.fonts.check('16px "Noto Sans Ethiopic"', "እንኳን")`, 20000);
    const files = await p.value(`performance.getEntriesByType("resource").map((r) => new URL(r.name).pathname).filter((n) => n.startsWith("/fonts/") && n.endsWith(".woff2"))`);
    const japanese = files.filter((file) => file.startsWith("/fonts/noto-sans-jp/")).length;
    const latin = files.includes("/fonts/inter/inter-latin-wght-normal.woff2");
    const unused = files.some((file) => file.includes("-cyrillic") || file.includes("-greek"));
    // Une page qui ne nomme aucune police n'en charge aucune.
    await p.open("/exemples/lecons/01-page.holo");
    const none = await p.value(`performance.getEntriesByType("resource").filter((r) => r.name.includes("/fonts/")).length`);
    const ok = loaded && japanese > 0 && japanese <= 8 && latin && !unused && none === 0;
    return [ok, `polices prêtes : ${loaded} ; morceaux japonais : ${japanese} sur 124 ; latin d'Inter : ${latin} ; cyrillique ou grec téléchargés : ${unused} ; page sans police : ${none} fichier`];
  }],
  ["une liste de définitions : un terme et sa définition, lus ensemble (leçon 120)", async (p, b) => {
    await p.open("/exemples/lecons/120-une-liste-de-definitions.holo");
    // Chaque terme garde sa définition, dans l'ordre ; une définition lit une valeur de la page.
    const sheet = await p.value(`[...document.querySelector("dl.holo-List").querySelectorAll(".holo-Term")].map((t) => t.querySelector("dt").textContent + "=" + t.querySelector("dd").textContent).join(" | ")`);
    // Le lecteur d'écran : des termes et des définitions, pas des éléments de liste à puces.
    const { result } = await b.send("Accessibility.getFullAXTree");
    const roles = (role) => result.nodes.filter((n) => n.role?.value === role && !n.ignored).length;
    const [terms, definitions, items] = [roles("term"), roles("definition"), roles("listitem")];
    const ok = sheet === "Hauteur=45 cm | Poids=2 kg | Couleur=Bleu nuit, ou cuivre | Prix=189,00 €" && terms === 6 && definitions === 6 && items === 0;
    return [ok, `fiche : ${sheet} ; lecteur d'écran : ${terms} termes, ${definitions} définitions, ${items} éléments à puces`];
  }],
  ["un module enfermé rend son nombre", async (p) => {
    await p.open("/exemples/lecons/69-module-enferme.holo");
    await p.click('[data-name="Calculer"]');
    const ok = await p.until(`(window.__holoModules ?? []).some((m) => m.ok && m.output === 5050)`);
    return [ok, ok ? "5 050" : JSON.stringify(await p.value("window.__holoModules ?? null"))];
  }],
  ["une abréviation expliquée une fois, une date lisible par les machines, une adresse (leçon 121)", async (p) => {
    await p.open("/exemples/lecons/121-une-abreviation-une-date-une-adresse.holo");
    // La première fois qu'elle vient dans un paragraphe, son sens est écrit : le lecteur d'écran
    // le lit, le téléphone le montre ; ailleurs, elle est seulement marquée.
    const first = await p.value(`document.querySelector("main p").textContent`);
    const marked = await p.value(`document.querySelectorAll('abbr[title="Maison des jeunes et de la culture"]').length`);
    const written = await p.value(`(document.body.textContent.match(/\\(Maison des jeunes et de la culture\\)/g) ?? []).length`);
    // Une date montrée se lit par les machines, et suit sa valeur quand un geste la change.
    const times = `[...document.querySelectorAll("main time")].map((t) => t.getAttribute("datetime") + "=" + t.textContent).join(" | ")`;
    const before = await p.value(times);
    await p.click('[data-name="Plus"]');
    const moved = await p.until(`document.querySelector("main time")?.getAttribute("datetime") === "2026-11-21"`);
    const after = await p.value(times);
    // Les moyens de joindre l'auteur, dans le pied de page.
    const address = await p.value(`document.querySelector("footer address.holo-Address")?.textContent ?? ""`);
    const ok = first.includes("MJC (Maison des jeunes et de la culture) ouvre") && marked === 4 && written === 1
      && before === "2026-11-14=14 novembre 2026 | 2026-11-14=samedi" && moved && after === "2026-11-21=21 novembre 2026 | 2026-11-21=samedi"
      && address.includes("12 rue des Arts");
    return [ok, `premier paragraphe : « ${first} » ; ${marked} abréviations marquées, sens écrit ${written} fois ; dates : ${before} → ${after} ; adresse : « ${address} »`];
  }],
  ["un groupe de champs : son nom dit par le lecteur d'écran, ses champs vérifiés à l'envoi (leçon 122)", async (p, b) => {
    await p.open("/exemples/lecons/122-un-groupe-de-champs.holo");
    // Le lecteur d'écran : chaque groupe a son nom, et ses champs sont dedans. Un Choice est déjà un groupe.
    const { result } = await b.send("Accessibility.getFullAXTree");
    const nodes = new Map(result.nodes.map((n) => [n.nodeId, n]));
    const fields = (group) => {
      const found = [];
      const next = [...(group.childIds ?? [])];
      while (next.length) {
        const node = nodes.get(next.shift());
        if (!node) continue;
        if (!node.ignored && ["textbox", "checkbox", "radio"].includes(node.role?.value)) found.push(node.name?.value);
        next.push(...(node.childIds ?? []));
      }
      return found.join(", ");
    };
    const groups = result.nodes.filter((n) => !n.ignored && ["group", "radiogroup"].includes(n.role?.value)).map((n) => `${n.role.value} « ${n.name?.value ?? ""} » : ${fields(n)}`);
    const heard = ["group « Adresse de livraison » : Rue, Code postal, Ville", "group « Pour te prévenir » : Par e-mail, Par SMS", "radiogroup « Jour de livraison » : Mardi, Samedi"].every((g) => groups.includes(g));
    // L'allure de base : sans la bordure du navigateur ; encadré par un style, le nom reste dedans ;
    // sur un téléphone (360 de large), rien ne déborde.
    const look = await p.value(`(() => {
      const [boxed, plain] = document.querySelectorAll(".holo-Fields");
      const legend = boxed?.querySelector(":scope > legend");
      if (!plain || !legend) return "pas de groupe nommé dans la page";
      const top = Math.round(legend.getBoundingClientRect().top - boxed.getBoundingClientRect().top);
      return getComputedStyle(plain).borderTopStyle + " / " + getComputedStyle(plain).minWidth + " / " + top;
    })()`);
    let wide;
    try {
      await b.send("Emulation.setDeviceMetricsOverride", { width: 360, height: 760, deviceScaleFactor: 2, mobile: true });
      await pause(300);
      wide = await p.value("document.documentElement.scrollWidth");
    } finally {
      await b.send("Emulation.clearDeviceMetricsOverride");
    }
    // À l'envoi, les champs du groupe sont vérifiés comme les autres (ADR-068) : le message sous le
    // champ, dans le groupe ; le clavier sur le premier à corriger. (Le toucher fait venir le moteur.)
    await p.click('[data-name="Send"]');
    const checked = await p.until(`document.querySelectorAll(".holo-Fields .holo-error").length === 3 && document.querySelectorAll(".holo-error").length === 4`);
    const first = await p.value(`document.activeElement?.dataset.bind ?? ""`);
    const ok = heard && look === "none / 0px / 13" && wide <= 360 && checked && first === "street";
    return [ok, `lecteur d'écran : ${groups.join(" | ")} ; allure (bordure / largeur la plus petite / nom sous le cadre) : ${look} ; téléphone : ${wide} px pour 360 ; à l'envoi, messages dans les groupes : ${checked}, le clavier sur « ${first} »`];
  }],
  ["des suggestions dans un champ, écrites ou venues d'une liste qui change ; on écrit autre chose (leçon 123)", async (p, b) => {
    await p.open("/exemples/lecons/123-des-suggestions-dans-un-champ.holo");
    // Chaque champ porte ses suggestions (input.list) : écrites dans la page, ou venues d'une liste.
    const options = (bind) => `[...(document.querySelector('[data-bind="${bind}"]').list?.options ?? [])].map((o) => o.value).join(", ")`;
    const fruits = await p.value(options("fruit"));
    const before = await p.value(options("ville"));
    // On écrit autre chose qu'une suggestion : le champ le garde, la page le montre.
    await p.type('[data-bind="ville"]', "Grenoble");
    const kept = await p.until(`document.querySelector("main").textContent.includes("Tu vas à Grenoble.")`);
    // Retenue, la ville rejoint la liste, et les suggestions la suivent pendant la visite.
    await p.click('[data-name="Retenir"]');
    const followed = await p.until(`[...(document.querySelector('[data-bind="ville"]').list?.options ?? [])].some((o) => o.value === "Grenoble")`, 10000);
    const after = await p.value(options("ville"));
    // Le lecteur d'écran : un champ qu'on écrit et qui propose une liste (pas le texte de l'étiquette).
    const { nodes } = (await b.send("Accessibility.getFullAXTree")).result;
    const field = nodes.find((n) => !n.ignored && n.name?.value === "La ville où tu vas" && !["StaticText", "InlineTextBox"].includes(n.role?.value));
    const [role, proposes] = [field?.role?.value, field?.properties?.find((q) => q.name === "autocomplete")?.value?.value];
    const ok = fruits === "Pomme, Poire, Abricot, Mirabelle" && before === "Paris, Lyon, Marseille, Lille, Bordeaux" && kept && followed
      && after === "Paris, Lyon, Marseille, Lille, Bordeaux, Grenoble" && role === "combobox" && proposes === "list";
    return [ok, `fruits : ${fruits} ; villes : ${before} → ${after} ; « Grenoble » écrit : ${kept ? "gardé" : "perdu"} ; lecteur d'écran : ${role}, autocomplete ${proposes}`];
  }],
  ["une citation courte et le titre d'une œuvre, avec les guillemets de la langue (leçon 124)", async (p) => {
    await p.open("/exemples/lecons/124-une-citation-courte.holo");
    // Les guillemets du français, écrits pour de vrai : ils se lisent, se copient, et une
    // citation dans une citation prend ceux du second niveau ; une espace fine insécable les tient.
    const said = await p.value(`document.querySelectorAll("main p")[1].innerText`);
    const quotes = await p.value(`[...document.querySelectorAll("main q.holo-q")].map((q) => q.textContent).join(" | ")`);
    // Le navigateur n'en ajoute pas d'autres par-dessus.
    const added = await p.value(`getComputedStyle(document.querySelector("q.holo-q")).quotes`);
    const titles = await p.value(`[...document.querySelectorAll("cite")].map((c) => c.textContent).join(" | ")`);
    const ok = said === "Jean a résumé le livre en une phrase : «\u202FValjean devient bon parce qu'un évêque l'a appelé “mon frère”.\u202F»"
      && quotes === "«\u202FValjean devient bon parce qu'un évêque l'a appelé “mon frère”.\u202F» | “mon frère”"
      && added === "none" && titles === "Les Misérables | Les Châtiments";
    return [ok, `lu : « ${said} » ; citations : ${quotes} ; guillemets du navigateur : ${added} ; œuvres : ${titles}`];
  }],
  ["le champ mot de passe : la page ne le lit jamais ; il part seulement vers holo serve, qui n'en garde que l'empreinte ; le collage, « Montrer » au clavier et au lecteur d'écran, autocomplete ; celui du compte vérifié ; hors HTTPS, fermé ; sans JavaScript (leçon 137, serve)", async (p, b) => {
    // ADR-114. Le mot de passe tapé est cherché partout où il ne doit pas être : l'état de la page
    // (le panneau ?values), le stockage du navigateur, l'adresse, le texte et le HTML de la page, les
    // demandes envoyées, la base de holo serve (et son journal d'écriture), son journal à l'écran.
    const lesson = "137-un-mot-de-passe.holo";
    const members = "137-mot-de-passe/annuler.holo";
    const secret = "une phrase-de-passe introuvable 🔑";
    const account = "une phrase assez longue pour Ada";
    const served = await startHoloServe([lesson, members]);
    const q = page(b, served.base);
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    const sent = [];
    const field = `document.getElementById("holo-password-Protect")`;
    const reveal = `document.querySelector('[aria-controls="holo-password-Protect"]')`;
    // Le bouton « Montrer », tel que le lecteur d'écran le reçoit : son nom, et s'il est enfoncé.
    const revealSaid = async () => {
      const { nodes } = (await b.send("Accessibility.getFullAXTree")).result;
      const node = nodes.find((n) => !n.ignored && n.role?.value === "button" && n.name?.value === "Montrer le mot de passe");
      return node ? `${node.name.value}, ${node.properties?.find((x) => x.name === "pressed")?.value?.value ?? "?"}` : "(absent)";
    };
    // Partout où le mot de passe ne doit pas être, dans la page et le navigateur.
    const leaks = (what) => q.value(`(() => {
      const found = [];
      const has = (text) => String(text ?? "").includes(${JSON.stringify(what)});
      if (has(document.getElementById("holo-values")?.textContent)) found.push("l'état (?values)");
      try { for (const store of [localStorage, sessionStorage]) for (let i = 0; i < store.length; i++) if (has(store.key(i)) || has(store.getItem(store.key(i)))) found.push("le stockage"); } catch { /* refusé */ }
      if (has(decodeURIComponent(location.href))) found.push("l'adresse");
      if (has(document.getElementById("page").innerText)) found.push("le texte de la page");
      if (has(document.documentElement.outerHTML)) found.push("le HTML");
      return found.join(", ") || "nulle part";
    })()`);
    const base = served.folder;
    const written = (what) => {
      const found = [];
      const visit = (dir) => { for (const entry of readdirSync(dir, { withFileTypes: true })) { const path = join(dir, entry.name); if (entry.isDirectory()) visit(path); else if (readFileSync(path).includes(Buffer.from(what))) found.push(path.slice(base.length + 1)); } };
      visit(join(base, "holo-data"));
      return found.join(", ") || "nulle part";
    };
    const holo = ["holo", "holo.exe"].map((name) => join(engine, "target", "release", name)).find(existsSync);
    const messages = () => spawnSync(holo, ["messages", base], { encoding: "utf8" }).stdout.trim().split("\n").filter(Boolean);
    let violations = [];
    try {
      await b.send("Network.enable");
      b.on("Network.requestWillBeSent", ({ request }) => sent.push({ url: request.url, method: request.method, body: request.postData ?? "" }));
      await b.send("Browser.grantPermissions", { permissions: ["clipboardReadWrite", "clipboardSanitizedWrite"] });
      await b.send("Page.bringToFront");
      // 1. Le champ, tel que le moteur le pose (avec le panneau des valeurs, ?values).
      await q.open(`/${lesson}?values`);
      if (!(await q.until("window.__holoStarted"))) return [false, "le moteur n'est pas arrivé"];
      const attributes = await q.value(`[${field}.type, ${field}.autocomplete, ${field}.getAttribute("aria-required"), ${field}.hasAttribute("maxlength"), ${field}.hasAttribute("name"), ${field}.hasAttribute("data-bind"), ${field}.disabled].join(" ")`);
      check("le champ", attributes === "password new-password true false false false false", attributes);
      check("le bouton « Montrer », montré par la page légère", await q.value(`!${reveal}.hidden && ${reveal}.offsetHeight > 0`), "caché");
      check("le lecteur d'écran, au départ", (await revealSaid()) === "Montrer le mot de passe, false", await revealSaid());
      // 2. Le collage : le presse-papiers du navigateur, puis Ctrl+V dans le champ. Rien ne l'empêche.
      await q.type('[data-bind="title"]', "Mon carnet");
      await q.value(`(() => { window.__pasteRefused = null; addEventListener("paste", (e) => { window.__pasteRefused = e.defaultPrevented; }); ${field}.focus(); return true; })()`);
      await b.send("Runtime.evaluate", { expression: `navigator.clipboard.writeText(${JSON.stringify(secret)})`, awaitPromise: true, userGesture: true });
      await b.send("Input.dispatchKeyEvent", { type: "keyDown", key: "v", code: "KeyV", windowsVirtualKeyCode: 86, modifiers: 2, commands: ["paste"] });
      await b.send("Input.dispatchKeyEvent", { type: "keyUp", key: "v", code: "KeyV", windowsVirtualKeyCode: 86, modifiers: 2 });
      await pause(200);
      const pasted = await q.value(`[${field}.value === ${JSON.stringify(secret)}, window.__pasteRefused].join(" ")`);
      check("coller", pasted === "true false", pasted);
      // 3. « Montrer », au clavier : Tab depuis le champ, puis Espace ; le lecteur d'écran dit l'état.
      await q.key("Tab", "Tab", 9);
      const onButton = await q.value(`document.activeElement === ${reveal}`);
      await q.key(" ", "Space", 32, " ");
      const shown = [await q.value(`${field}.type`), await revealSaid()].join(" ; ");
      await q.key(" ", "Space", 32, " ");
      const hidden = [await q.value(`${field}.type`), await revealSaid()].join(" ; ");
      check("« Montrer » au clavier", onButton && shown === "text ; Montrer le mot de passe, true" && hidden === "password ; Montrer le mot de passe, false", `focus ${onButton}, montré : ${shown}, caché : ${hidden}`);
      // Au doigt ou à la souris, puis on le laisse montré : il doit être caché de nouveau à l'envoi.
      await q.click(`[aria-controls="holo-password-Protect"]`);
      const clicked = await q.value(`${field}.type`);
      // 4. La page ne le lit jamais : ni l'état, ni le stockage, ni l'adresse, ni le texte, ni le HTML.
      const panel = await q.value(`document.getElementById("holo-values")?.textContent.includes('title = "Mon carnet"')`);
      const beforeSending = await leaks(secret);
      const sentBefore = sent.filter((r) => r.body.includes(secret)).length;
      check("la page ne le lit jamais", panel && beforeSending === "nulle part" && sentBefore === 0, `panneau des valeurs ${panel}, trouvé : ${beforeSending}, demandes qui le portent : ${sentBefore}`);
      // 5. Trop court : le moteur le dit sous le champ, le clavier y revient, rien ne part.
      await q.value(`(() => { window.__kept = ${field}.value; ${field}.value = "court"; return true; })()`);
      await q.click('[data-name="Send"]');
      await q.until(`document.querySelector("#holo-error-Protect-holo-password")`, 5000);
      const short = await q.value(`[document.querySelector("#holo-error-Protect-holo-password")?.textContent, ${field}.getAttribute("aria-invalid"), ${field}.getAttribute("aria-describedby"), document.activeElement === ${field}].join(" | ")`);
      await pause(200);
      const announced = await q.value(`document.getElementById("announcement").textContent`);
      check("trop court", short === "Au moins 12 caractères. | true | holo-password-Protect-hint holo-error-Protect-holo-password | true" && announced === "Au moins 12 caractères." && !sent.some((r) => r.body.includes("court")), `${short} ; annoncé : ${announced}`);
      // 6. L'envoi : il part une fois, vers holo serve, à côté des valeurs ; caché de nouveau, puis effacé.
      await q.value(`(() => { ${field}.value = window.__kept; delete window.__kept; ${field}.dispatchEvent(new Event("input", { bubbles: true })); return true; })()`);
      await q.click('[data-name="Send"]');
      const arrived = await q.until(`document.getElementById("page").innerText.includes("Le carnet « Mon carnet » est protégé.")`, 10000);
      const carried = sent.filter((r) => r.body.includes(JSON.stringify(secret).slice(1, -1)) || r.body.includes(secret));
      const shape = carried.map((r) => { try { const body = JSON.parse(r.body); return `${r.method} ${r.url === `${served.base}/${lesson}` ? "la page" : r.url} ${Object.keys(body).join(",")} ${JSON.stringify(body.values)}`; } catch { return `${r.method} ${r.url} (pas du JSON)`; } });
      const after = await q.value(`[${field}.type, ${reveal}.getAttribute("aria-pressed"), ${field}.value === ""].join(" ")`);
      check("l'envoi", arrived && clicked === "text" && shape.join(" ; ") === `POST la page form,values,password {"title":"Mon carnet"}` && after === "password false true", `arrivé ${arrived}, montré avant ${clicked} ; ${shape.join(" ; ") || "aucune demande"} ; après : ${after}`);
      check("après l'envoi, la page ne l'a toujours pas", (await leaks(secret)) === "nulle part", await leaks(secret));
      violations = (await (async () => {
        const axe = readFileSync(join(engine, "node_modules", "axe-core", "axe.min.js"), "utf8");
        await q.value(`${axe}\n;window.axe.version`);
        return q.value('window.axe.run(document, { resultTypes: ["violations"] }).then((r) => r.violations.map((v) => v.id))');
      })());
      // 7. holo serve n'en garde que l'empreinte Argon2id : ni la base, ni son journal d'écriture, ni
      // son journal à l'écran.
      const kept = messages();
      const print = kept.find((line) => line.includes("\"form\":\"Protect\"")) ?? "";
      check("holo serve garde l'empreinte", /"submission":\{"form":"Protect","values":\{"title":"Mon carnet"\},"password":"\$argon2id\$v=19\$m=19456,t=2,p=1\$[^"]+"\}/.test(print), print || "(aucun message)");
      check("rien en clair chez holo serve", written(secret) === "nulle part" && written("introuvable") === "nulle part" && !served.log().includes("introuvable") && !kept.join("\n").includes("introuvable"), `base : ${written("introuvable")} ; journal : ${served.log().includes("introuvable")}`);
      // 8. Le mot de passe du compte, sur la page réservée : le faux est dit sous le champ, le bon envoie.
      await q.open(`/account/signup?next=/${members}`, 300);
      await q.type("#name", "Ada");
      await q.type("#password", account);
      await q.type("#again", account);
      await q.click('main form button[type="submit"]');
      await q.until(`location.pathname === "/${members}" && window.__holoStarted`, 20000);
      const current = `document.getElementById("holo-password-Cancel")`;
      const currentAttributes = await q.value(`${current}?.autocomplete`);
      await q.type("#holo-password-Cancel", "pas le bon mot de passe");
      await q.click('[data-name="Confirm"]');
      await q.until(`document.querySelector("#holo-error-Cancel-holo-password")`, 10000);
      const wrong = await q.value(`[document.querySelector("#holo-error-Cancel-holo-password")?.textContent, ${current}.value === "", document.activeElement === ${current}, document.getElementById("page").innerText.includes("Ta réservation est annulée.")].join(" | ")`);
      await q.type("#holo-password-Cancel", account);
      await q.click('[data-name="Confirm"]');
      const cancelled = await q.until(`document.getElementById("page").innerText.includes("Ta réservation est annulée.")`, 10000);
      const cancel = messages().find((line) => line.includes("\"form\":\"Cancel\"")) ?? "";
      check("le mot de passe du compte", currentAttributes === "current-password" && wrong === "Ce n'est pas le mot de passe de ton compte. | true | true | false" && cancelled && cancel.includes(`"submission":{"form":"Cancel","values":{}}`), `${currentAttributes} ; faux : ${wrong} ; juste : ${cancelled} ; rangé : ${cancel}`);
      check("rien du compte en clair", written(account) === "nulle part" && written("pas le bon") === "nulle part" && !served.log().includes("pas le bon"), written("pas le bon"));
      // 9. Hors HTTPS : la page ouverte par l'adresse de ce PC sur le réseau local, comme un téléphone
      // du même Wi-Fi. Le navigateur dit qu'elle n'est pas sûre : le champ se ferme, la note dit pourquoi.
      const { networkInterfaces } = await import("node:os");
      const lan = Object.values(networkInterfaces()).flat().find((a) => a?.family === "IPv4" && !a.internal)?.address;
      let insecure = "pas d'adresse sur le réseau local : non essayé";
      if (lan) {
        const away = page(b, served.base.replace("localhost", lan));
        await away.open(`/${lesson}`);
        await away.until("window.__holoStarted", 20000);
        insecure = await away.value(`[window.isSecureContext, ${field}.disabled, ${field}.getAttribute("aria-describedby"), document.getElementById("holo-password-Protect-note").hidden, document.getElementById("holo-password-Protect-note").textContent, ${reveal}.hidden].join(" | ")`);
        check("hors HTTPS, fermé", insecure === "false | true | holo-password-Protect-note | false | Ce champ ne s'ouvre qu'en HTTPS : un mot de passe ne part jamais en clair. | true", insecure);
        // Un envoi forgé quand même : refusé par holo serve, rien n'est rangé.
        const before = messages().length;
        const forged = await fetch(`${served.base.replace("localhost", lan)}/${lesson}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ form: "Protect", values: { title: "x" }, password: secret }) });
        const refusal = `${forged.status} ${await forged.text()}`;
        // Le journal de holo serve arrive par un tuyau, un peu après la réponse : il dit la raison, rien d'autre.
        for (let i = 0; i < 20 && !served.log().includes("Mot de passe refusé"); i++) await pause(100);
        const told = served.log().split("\n").find((line) => line.startsWith("Mot de passe refusé")) ?? "(rien)";
        check("un envoi forgé hors HTTPS", refusal === "422 holo-password|Ce mot de passe ne part pas : la page n'est pas en HTTPS." && messages().length === before && told.includes("la demande n'est pas en HTTPS") && !told.includes("introuvable"), `${refusal} ; messages : ${messages().length - before} ; journal : ${told}`);
        // Sans JavaScript non plus : holo serve fabrique la page avec le champ fermé.
        const plain = await (await fetch(`${served.base.replace("localhost", lan)}/${lesson}`, { headers: { accept: "text/html" } })).text();
        check("hors HTTPS, sans JavaScript", plain.includes(`<input disabled aria-describedby="holo-password-Protect-note"`) && !plain.includes(`name="holo-password-Protect"`), plain.match(/<input[^>]*data-secret[^>]*>/)?.[0] ?? "(pas de champ)");
      }
      // 10. Sans JavaScript, sur ce PC : le champ part avec le formulaire des gestes ; holo serve n'en
      // garde que l'empreinte, et la page revient sans lui.
      let withoutScript = "";
      try {
        await b.send("Emulation.setScriptExecutionDisabled", { value: true });
        await q.open(`/${lesson}`, 300);
        const attached = await q.value(`[${field}.getAttribute("form"), ${field}.name, ${reveal}.hidden].join(" ")`);
        await q.value(`(() => { document.querySelector('[data-bind="title"]').value = "Sans script"; ${field}.value = ${JSON.stringify(secret)}; return true; })()`);
        const count = messages().length;
        await q.click('[data-name="Send"]');
        await q.until(`document.readyState === "complete" && document.getElementById("page").innerText.includes("Le carnet « Sans script » est protégé.")`, 10000);
        const last = messages().at(-1) ?? "";
        withoutScript = `${attached} ; ${messages().length - count} message, ${/"password":"\$argon2id\$/.test(last) ? "son empreinte" : last} ; ${await leaks(secret)}`;
        check("sans JavaScript", withoutScript === "holo-gestures holo-password-Protect true ; 1 message, son empreinte ; nulle part" && written(secret) === "nulle part", withoutScript);
      } finally {
        await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      }
      check("axe-core", !violations.length, violations.join(", "));
      if (b.errors.length) faults.push(`erreurs : ${b.errors.join(" | ")}`);
      return [faults.length === 0, faults.length ? faults.join("\n      ") : `le champ : ${attributes} ; « Montrer » au clavier : ${shown}, puis ${hidden} ; collé : ${pasted} ; trop court : ${short} ; envoyé une fois : ${shape.join(" ; ")}, puis ${after} ; la page ne l'a lu nulle part ; holo serve : l'empreinte seule, rien en clair (base, WAL, journal) ; le compte : faux « ${wrong} », juste ${cancelled} ; hors HTTPS : ${insecure} ; sans JavaScript : ${withoutScript} ; axe-core : zéro défaut`];
    } finally {
      b.on("Network.requestWillBeSent", null);
      await b.send("Network.disable");
      await b.send("Browser.resetPermissions");
      await b.send("Network.clearBrowserCookies");
      served.stop();
    }
  }],
  ["une grille : une case sur deux colonnes et deux lignes, des zones dans l'ordre de lecture ; rien ne déborde sur un téléphone (leçon 127)", async (p, b) => {
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    // Les boîtes (gauche, haut, largeur, hauteur) : la grille des tableaux, la grande case, les
    // autres cases, puis la grille du jardin et ses quatre zones, dans l'ordre de la page.
    const boxes = `(() => {
      const box = (e) => { const r = e.getBoundingClientRect(); return [Math.round(r.left), Math.round(r.top), Math.round(r.width), Math.round(r.height)]; };
      const [paintings, garden] = document.querySelectorAll("main .holo-Grid");
      const big = document.querySelector(".holo-s-vedette")?.closest(".holo-cell");
      if (!paintings || !garden || !big) return null;
      return { grid: box(paintings), big: box(big), cards: [...paintings.children].filter((c) => c !== big).map(box), garden: box(garden), zones: [...garden.children].map(box), wide: document.documentElement.scrollWidth };
    })()`;
    const near = (a, b2) => Math.abs(a - b2) <= 2;
    // Les zones l'une sous l'autre, dans l'ordre : chacune sur toute la largeur, plus bas que la précédente.
    const stacked = (m) => m.zones.every((z, i) => near(z[0], m.garden[0]) && near(z[2], m.garden[2]) && (i === 0 || z[1] > m.zones[i - 1][1]));
    try {
      // Sur un ordinateur : trois colonnes ; la grande case en prend deux, sur deux lignes ; les zones comme on les a dessinées.
      await b.send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 800, deviceScaleFactor: 1, mobile: false });
      await p.open("/exemples/lecons/127-une-grille-et-ses-zones.holo", 600);
      let m = await p.value(boxes);
      if (!m) return [false, `la leçon ne montre pas ses deux grilles : ${(await p.text()).slice(0, 200)}`];
      const [first, second, third, fourth] = m.cards;
      check("ordinateur : la grande case sur deux colonnes", near(m.big[2], 2 * third[2] + 12) && near(m.big[0], third[0]) && near(m.big[0] + m.big[2], fourth[0] + fourth[2]), `grande ${JSON.stringify(m.big)}, en dessous ${JSON.stringify(third)} et ${JSON.stringify(fourth)}`);
      check("ordinateur : la grande case sur deux lignes", near(m.big[1], first[1]) && near(m.big[1] + m.big[3], second[1] + second[3]), `grande ${JSON.stringify(m.big)}, à côté ${JSON.stringify(first)} et ${JSON.stringify(second)}`);
      const [top, menu, text, foot] = m.zones;
      const column = (m.garden[2] - 2 * 16) / 3;
      check("ordinateur : le haut et le pied sur toute la largeur", near(top[2], m.garden[2]) && near(foot[2], m.garden[2]), `${top[2]} et ${foot[2]} pour ${m.garden[2]}`);
      check("ordinateur : le menu à gauche du texte, sur une colonne de trois", near(menu[1], text[1]) && near(menu[2], column) && near(text[0], menu[0] + menu[2] + 16) && near(text[2], 2 * column + 16), `menu ${JSON.stringify(menu)}, texte ${JSON.stringify(text)}`);
      check("ordinateur : l'ordre de lecture est celui de la page", top[1] < menu[1] && menu[0] < text[0] && Math.max(menu[1] + menu[3], text[1] + text[3]) <= foot[1], JSON.stringify(m.zones));
      // Sur un téléphone (360 de large) : deux colonnes ; la grande case prend la ligne ; les zones s'empilent dans l'ordre.
      await b.send("Emulation.setDeviceMetricsOverride", { width: 360, height: 760, deviceScaleFactor: 2, mobile: true });
      await p.open("/exemples/lecons/127-une-grille-et-ses-zones.holo", 600);
      m = await p.value(boxes);
      check("téléphone : rien ne déborde", m.wide <= 360, `${m.wide} px pour 360`);
      check("téléphone : la grande case prend la ligne", near(m.big[2], m.grid[2]) && near(m.cards[0][2] * 2 + 12, m.grid[2]), `grande ${m.big[2]}, case ${m.cards[0][2]}, grille ${m.grid[2]}`);
      check("téléphone : les zones l'une sous l'autre, dans l'ordre", stacked(m), JSON.stringify(m.zones));
      // Un téléphone plié (280 de large, le Galaxy Z Fold fermé) : une seule colonne ; la grande case
      // n'en a pas deux, elle prend toute la ligne au lieu de créer une colonne qui déborde.
      await b.send("Emulation.setDeviceMetricsOverride", { width: 280, height: 700, deviceScaleFactor: 2, mobile: true });
      await p.open("/exemples/lecons/127-une-grille-et-ses-zones.holo", 600);
      m = await p.value(boxes);
      check("plié : rien ne déborde", m.wide <= 280, `${m.wide} px pour 280`);
      check("plié : une colonne, et la grande case sur toute la ligne", near(m.big[2], m.grid[2]) && near(m.cards[0][2], m.grid[2]), `grande ${m.big[2]}, case ${m.cards[0][2]}, grille ${m.grid[2]}`);
      check("plié : les zones l'une sous l'autre", stacked(m), JSON.stringify(m.zones));
      if (b.errors.length) faults.push(`erreurs : ${b.errors.join(" | ")}`);
    } finally {
      await b.send("Emulation.clearDeviceMetricsOverride");
    }
    return [faults.length === 0, faults.length ? faults.join("\n      ") : "ordinateur : deux colonnes et deux lignes, le menu à gauche du texte, dans l'ordre de lecture ; téléphone (360) et téléphone plié (280) : rien ne déborde, la grande case prend la ligne, les zones s'empilent dans l'ordre"];
  }],
  ["partager la page : la feuille du téléphone avec le titre et l'adresse, sinon l'adresse copiée (leçon 130)", async (p, b) => {
    const lesson = "/exemples/lecons/130-partager-la-page.holo";
    const status = `document.querySelector('[data-name="Partage"] [data-capability-status]')`;
    // Ce que la page montre : la zone d'état, le compte des partages (done), le texte de la panne (failed).
    const shown = () => p.value(`[${status}.textContent, document.getElementById("page").innerText.match(/Partagée (\\d+) fois/)?.[1] ?? "?", document.getElementById("page").innerText.includes("Le partage n'a pas marché")]`);
    // Un script posé avant la page, comme le navigateur d'un téléphone ou d'un ordinateur : le
    // partage du téléphone est remplacé pour l'essai, ou retiré (Chrome sous Windows en a un).
    const before = async (source) => (await b.send("Page.addScriptToEvaluateOnNewDocument", { source })).result.identifier;
    const forget = (identifier) => b.send("Page.removeScriptToEvaluateOnNewDocument", { identifier });
    // 1. Un ordinateur sans partage : l'adresse est copiée, et la page le dit, dans la zone que le
    // lecteur d'écran lit.
    let script = await before("delete Navigator.prototype.share; delete Navigator.prototype.canShare;");
    await b.send("Browser.grantPermissions", { permissions: ["clipboardReadWrite", "clipboardSanitizedWrite"] });
    let computer;
    try {
      await p.open(lesson);
      if (!(await p.until("window.__holoStarted"))) return [false, "le moteur n'est pas arrivé"];
      await p.click('[data-name="Envoyer"]');
      await p.until(`${status}.textContent.includes("copiée")`, 5000);
      const read = await b.send("Runtime.evaluate", { expression: `navigator.clipboard.readText().then((t) => [t, location.href, ${status}.getAttribute("aria-live")])`, awaitPromise: true, returnByValue: true, userGesture: true });
      const [clipboard, address, live] = read.result?.result?.value ?? [];
      computer = { said: await shown(), copied: clipboard === address, clipboard, live };
    } finally {
      await forget(script);
      await b.send("Browser.resetPermissions");
    }
    // 2. Un téléphone : navigator.share, remplacé avant la page, doit recevoir le titre et l'adresse,
    // pendant le toucher même (window.event : l'appel part dans le clic, avant toute attente).
    script = await before(`window.__shares = []; window.__answer = "ok"; Navigator.prototype.canShare = () => true;
      Navigator.prototype.share = function (data) {
        window.__shares.push({ ...data, during: window.event?.type ?? "", active: navigator.userActivation.isActive });
        return window.__answer === "ok" ? Promise.resolve() : Promise.reject(new DOMException("essai", window.__answer));
      };`);
    let phone, violations;
    try {
      await p.open(lesson);
      if (!(await p.until("window.__holoStarted"))) return [false, "le moteur n'est pas arrivé (téléphone)"];
      await p.click('[data-name="Envoyer"]');
      await p.until(`${status}.textContent === "Page partagée."`, 5000);
      const shared = await p.value(`(() => { const s = window.__shares[0] ?? {}; return { title: s.title, url: s.url, sameTitle: s.title === document.title, sameAddress: s.url === location.href, during: s.during, active: s.active, calls: window.__shares.length }; })()`);
      const done = await shown();
      // 3. Le visiteur ferme la feuille sans rien choisir : « Partage annulé. », ni done ni failed.
      await p.value(`window.__answer = "AbortError"`);
      await p.click('[data-name="Envoyer"]');
      await p.until(`${status}.textContent === "Partage annulé."`, 5000);
      const cancelled = await shown();
      // 4. Une vraie panne : l'adresse est écrite à l'écran, et la page le sait (failed).
      await p.value(`window.__answer = "NotAllowedError"`);
      await p.click('[data-name="Envoyer"]');
      await p.until(`${status}.textContent.includes("Partage impossible")`, 5000);
      const failed = await shown();
      phone = { shared, done, cancelled, failed, address: await p.value("location.href"), errors: [...b.errors] };
      // L'audit d'accessibilité de la page après la panne : le texte de la panne, l'adresse écrite.
      const { createRequire } = await import("node:module");
      let axe;
      try { axe = readFileSync(createRequire(join(engine, "x.js")).resolve("axe-core/axe.min.js"), "utf8"); }
      catch { return [false, "axe-core absent : « npm install --no-save axe-core@4.10.3 », dans moteur/"]; }
      await p.value(`${axe}\n;window.axe.version`);
      violations = await p.value(`window.axe.run(document, { resultTypes: ["violations"] }).then((r) => r.violations.map((v) => v.id + " : " + v.nodes.map((n) => n.html.slice(0, 100)).join(" | ")))`);
    } finally {
      await forget(script);
    }
    const ok = computer.copied && computer.live === "polite" && computer.said[0] === "Adresse de la page copiée : colle-la où tu veux." && computer.said[1] === "1" && !computer.said[2]
      && phone.shared.sameTitle && phone.shared.sameAddress && phone.shared.during === "click" && phone.shared.active === true && phone.shared.calls === 1
      && phone.done[0] === "Page partagée." && phone.done[1] === "1" && !phone.done[2]
      && phone.cancelled[0] === "Partage annulé." && phone.cancelled[1] === "1" && !phone.cancelled[2]
      && phone.failed[0] === `Partage impossible ici. L'adresse de la page, à copier : ${phone.address}` && phone.failed[1] === "1" && phone.failed[2]
      && phone.errors.length === 0 && violations.length === 0;
    return [ok, `ordinateur : « ${computer.said[0]} », presse-papiers « ${computer.clipboard} », zone ${computer.live}, ${computer.said[1]} partage ; téléphone : share(${JSON.stringify({ title: phone.shared.title, url: phone.shared.url })}) pendant « ${phone.shared.during} » (geste actif : ${phone.shared.active}), « ${phone.done[0]} », ${phone.done[1]} partage ; feuille fermée : « ${phone.cancelled[0]} », ${phone.cancelled[1]} partage, panne montrée : ${phone.cancelled[2]} ; panne : « ${phone.failed[0]} », panne montrée : ${phone.failed[2]} ; erreurs : ${phone.errors.length} ; axe-core : ${violations.length ? violations.join(" ; ") : "zéro défaut"}`];
  }],
  ["réordonner une liste : la poignée à la souris et au doigt, Monter et Descendre au clavier, annoncés ; sans JavaScript, holo serve (leçon 128, serve)", async (p, b) => {
    const lesson = "/exemples/lecons/128-reordonner-une-liste.holo";
    // L'ordre gardé par un essai précédent (keep) est oublié : la leçon part de son départ.
    await p.open(lesson);
    await p.value(`localStorage.removeItem("holo:${lesson}")`);
    await p.open(lesson);
    if (!(await p.until("window.__holoStarted"))) return [false, "le moteur n'est pas arrivé"];
    const order = () => p.value(`[...document.querySelectorAll('[data-list="tableaux"] > .holo-line')].map((l) => l.querySelector(".holo-line-content").textContent).join(" | ")`);
    const said = () => p.value(`document.getElementById("announcement").textContent`);
    // Le centre de la poignée de chaque ligne, et celui de la ligne, une fois la liste à l'écran.
    const places = () => p.value(`(() => {
      document.querySelector('[data-list="tableaux"]').closest(".holo-Column").scrollIntoView({ block: "center" });
      const centre = (e) => { const r = e.getBoundingClientRect(); return [r.left + r.width / 2, r.top + r.height / 2]; };
      return [...document.querySelectorAll('[data-list="tableaux"] > .holo-line')].map((l) => ({ grip: centre(l.querySelector(".holo-grip")), line: centre(l.querySelector(".holo-movable")) }));
    })()`);
    const start = await order();
    // 1. La souris tient la poignée de « La porte bleue » (rang 2) et la pose sur la première ligne.
    let spots = await places();
    const [[gx, gy], [, ty]] = [spots[2].grip, spots[0].line];
    await b.send("Input.dispatchMouseEvent", { type: "mousePressed", x: gx, y: gy, button: "left", buttons: 1, clickCount: 1 });
    for (let step = 1; step <= 10; step++) await b.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: gx, y: gy + ((ty - gy) * step) / 10, button: "left", buttons: 1 });
    await b.send("Input.dispatchMouseEvent", { type: "mouseReleased", x: gx, y: ty, button: "left", buttons: 0, clickCount: 1 });
    const byMouse = await p.until(`document.getElementById("page").innerText.includes("En tête : La porte bleue.")`, 5000);
    const afterMouse = await order();
    const heardMouse = (await p.until(`document.getElementById("announcement").textContent.startsWith("« La porte bleue »")`, 3000)) && (await said());
    // L'ordre est dans les valeurs de la page, et keep le garde : l'arbitre l'a changé.
    const kept = await p.value(`localStorage.getItem("holo:${lesson}") ?? ""`);
    // 2. Le clavier : « Descendre » de la ligne de tête, avec Entrée ; le clavier suit la ligne.
    await p.value(`document.querySelector('[data-list="tableaux"] > .holo-line[data-rank="0"] [data-move="down"]').focus()`);
    await p.key("Enter", "Enter", 13, "\r");
    await p.until(`document.getElementById("page").innerText.includes("En tête : Le lever du soleil.")`, 5000);
    const afterKey = await order();
    const focus = await p.value(`(() => { const e = document.activeElement; return \`\${e?.dataset.move ?? e?.tagName} de \${e?.closest(".holo-movable")?.dataset.label ?? "?"} (rang \${e?.closest(".holo-line")?.dataset.rank ?? "?"})\`; })()`);
    const heardKey = (await p.until(`document.getElementById("announcement").textContent.includes("position 2 sur 4")`, 3000)) && (await said());
    // « Monter » sur la ligne de tête, avec Espace : rien ne bouge, et c'est dit.
    await p.value(`document.querySelector('[data-list="tableaux"] > .holo-line[data-rank="0"] [data-move="up"]').focus()`);
    await p.key(" ", "Space", 32, " ");
    const heardTop = (await p.until(`document.getElementById("announcement").textContent.includes("déjà en haut")`, 3000)) && (await said());
    const unchanged = (await order()) === afterKey;
    // 3. Le lecteur d'écran : « Monter » et « Descendre » nommés avec la ligne ; la poignée, cachée.
    const { nodes } = (await b.send("Accessibility.getFullAXTree")).result;
    const named = nodes.filter((n) => !n.ignored && n.role?.value === "button" && /^(Monter|Descendre) « /.test(n.name?.value ?? "")).length;
    const gripHeard = nodes.some((n) => !n.ignored && n.name?.value === "⠿");
    // 4. Le doigt, sur un téléphone : la poignée du « Jour de marché » (rang 3), posée sur la deuxième ligne.
    let byFinger = false;
    let afterFinger = "";
    if (!phone) {
      await b.send("Emulation.setDeviceMetricsOverride", { width: 400, height: 800, deviceScaleFactor: 2, mobile: true });
      await b.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
    }
    try {
      await pause(400);
      spots = await places();
      const [[fx, fy], [, sy]] = [spots[3].grip, spots[1].line];
      await b.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: fx, y: fy, id: 1 }] });
      for (let step = 1; step <= 10; step++) await b.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [{ x: fx, y: fy + ((sy - fy) * step) / 10, id: 1 }] });
      await b.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
      byFinger = await p.until(`[...document.querySelectorAll('[data-list="tableaux"] > .holo-line')][1]?.querySelector(".holo-line-content").textContent === "Le jour de marché"`, 5000);
      afterFinger = await order();
    } finally {
      if (!phone) {
        await b.send("Emulation.setTouchEmulationEnabled", { enabled: false });
        await b.send("Emulation.clearDeviceMetricsOverride");
      }
    }
    const keptFinger = await p.value(`localStorage.getItem("holo:${lesson}") ?? ""`);
    // 5. Sans JavaScript, avec holo serve : pas de poignée ; « Descendre » de la première ligne part au serveur.
    const served = await startHoloServe(["128-reordonner-une-liste.holo"]);
    const q = page(b, served.base);
    let withoutScript = "";
    let gripHidden = false;
    try {
      await b.send("Emulation.setScriptExecutionDisabled", { value: true });
      await q.open("/128-reordonner-une-liste.holo", 300);
      gripHidden = await q.value(`getComputedStyle(document.querySelector(".holo-grip")).display === "none"`);
      await q.click('[data-list="tableaux"] > .holo-line[data-rank="0"] [data-move="down"]');
      await q.until(`document.readyState === "complete" && document.getElementById("page").innerText.includes("En tête : La rivière.")`, 5000);
      withoutScript = await q.value(`[...document.querySelectorAll('[data-list="tableaux"] > .holo-line')].map((l) => l.querySelector(".holo-line-content").textContent).join(" | ")`);
    } finally {
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      served.stop();
    }
    const ok = start === "Le lever du soleil | La rivière | La porte bleue | Le jour de marché"
      && byMouse && afterMouse === "La porte bleue | Le lever du soleil | La rivière | Le jour de marché" && heardMouse === "« La porte bleue » : position 1 sur 4."
      && kept.includes("tableaux=[La%20porte%20bleue,Le%20lever%20du%20soleil,La%20rivi%C3%A8re,Le%20jour%20de%20march%C3%A9]")
      && afterKey === "Le lever du soleil | La porte bleue | La rivière | Le jour de marché" && focus === "down de La porte bleue (rang 1)" && heardKey === "« La porte bleue » : position 2 sur 4."
      && heardTop === "« Le lever du soleil » est déjà en haut." && unchanged && named === 8 && !gripHeard
      && byFinger && afterFinger === "Le lever du soleil | Le jour de marché | La porte bleue | La rivière" && keptFinger.includes("tableaux=[Le%20lever%20du%20soleil,Le%20jour%20de%20march%C3%A9,")
      && gripHidden && withoutScript === "La rivière | Le lever du soleil | La porte bleue | Le jour de marché";
    return [ok, `départ : ${start} ; souris : ${afterMouse} (« ${heardMouse} ») ; clavier : ${afterKey}, le focus sur ${focus} (« ${heardKey} ») ; Monter en tête : « ${heardTop} », rien ne bouge : ${unchanged} ; lecteur d'écran : ${named} boutons nommés, poignée entendue : ${gripHeard} ; doigt : ${afterFinger} ; gardé : ${keptFinger.includes("Le%20jour%20de%20march%C3%A9,La%20porte")} ; sans JavaScript, poignée cachée : ${gripHidden}, après Descendre : ${withoutScript}`];
  }],
  ["une barre de lecture et un retour en haut : scroll au plus dix fois par seconde, sticky en haut et en bas, le focus jamais caché, un téléphone, le clavier de l'écran, sans JavaScript (leçon 129, défilement)", async (p, b) => {
    const lesson = "/exemples/lecons/129-une-barre-de-lecture.holo";
    const tall = "/exemples/.essais-navigateur/bloc-qui-reste-trop-haut.holo";
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${JSON.stringify(seen)}`); };
    const errors = (where) => { if (b.errors.length) faults.push(`${where}, erreurs : ${b.errors.join(" | ")}`); };
    // Ce que montre la page : la barre du haut, le bouton du bas, la valeur montrée et la place
    // réelle, la largeur de la page, la place du bouton rond du moteur, la marge laissée au focus.
    const seen = () => p.value(`(() => {
      const box = (e) => { if (!e) return null; const r = e.getBoundingClientRect(); return r.height > 0 ? { top: Math.round(r.top), bottom: Math.round(r.bottom), height: Math.round(r.height), position: getComputedStyle(e).position } : null; };
      const room = document.documentElement.scrollHeight - innerHeight;
      const html = getComputedStyle(document.documentElement);
      return { bar: box(document.querySelector('[data-sticky="top"]')), back: box(document.querySelector('[data-sticky="bottom"]')),
        value: document.querySelector('[data-progress="scroll"]')?.value, percent: document.querySelector('[data-state="scroll"]')?.textContent,
        real: room > 0 ? Math.round(scrollY / room * 100) : 0, scrollY: Math.round(scrollY), height: innerHeight, wide: document.documentElement.scrollWidth,
        menu: parseFloat(getComputedStyle(document.getElementById("menu")).bottom), padding: [parseFloat(html.scrollPaddingTop) || 0, parseFloat(html.scrollPaddingBottom) || 0] };
    })()`);
    // Toucher un bloc là où il est, sans faire défiler la page avant : un bloc qui reste à l'écran
    // n'est pas à sa place dans la page.
    const tapAt = async (selector) => {
      const at = await p.value(`(() => { const e = document.querySelector(${JSON.stringify(selector)}); if (!e) return null; const r = e.getBoundingClientRect(); return [r.left + r.width / 2, r.top + r.height / 2]; })()`);
      if (!at) return false;
      for (const type of ["mousePressed", "mouseReleased"]) await b.send("Input.dispatchMouseEvent", { type, x: at[0], y: at[1], button: "left", clickCount: 1 });
      return true;
    };
    // Tab, ou Maj + Tab ; puis le temps qu'un bloc apparaisse (If) et que la page le mesure.
    const tab = async (back) => {
      for (const type of ["keyDown", "keyUp"]) await b.send("Input.dispatchKeyEvent", { type, key: "Tab", code: "Tab", windowsVirtualKeyCode: 9, modifiers: back ? 8 : 0 });
      await pause(400);
    };
    // Ce qui a le focus, et ce qui le cache : un bloc qui reste à l'écran, ou le bord de l'écran.
    const focused = () => p.value(`(() => {
      const e = document.activeElement;
      if (!e || e === document.body) return { name: "(rien)", hidden: "" };
      const name = (e.textContent || e.getAttribute("aria-label") || e.tagName).trim().slice(0, 32);
      if (e.closest("[data-sticky]")) return { name, hidden: "" };
      const r = e.getBoundingClientRect();
      const stuck = [...document.querySelectorAll("[data-sticky]")].map((s) => s.getBoundingClientRect()).filter((s) => s.height > 0);
      const under = stuck.some((s) => r.top < s.bottom - 1 && r.bottom > s.top + 1);
      return { name, hidden: under ? "sous un bloc qui reste" : r.top < 0 || r.bottom > innerHeight ? "hors de l'écran" : "", at: [Math.round(r.top), Math.round(r.bottom)] };
    })()`);
    let summary = {};
    try {
      // 1. Sur un ordinateur : la barre en haut, vide, que le lecteur d'écran nomme ; pas de bouton.
      await b.send("Emulation.setDeviceMetricsOverride", { width: 1000, height: 700, deviceScaleFactor: 1, mobile: false });
      await p.open(lesson);
      if (!(await p.until("window.__holoStarted"))) return [false, "le moteur n'est pas arrivé"];
      let m = await seen();
      check("au départ : la barre vide, sans bouton", m.value === 0 && m.percent === "0" && m.back === null && m.bar?.position === "sticky", m);
      // Le bouton du bas est caché (If faux) : rien ne colle en bas, aucune marge du focus en bas.
      check("au départ : la marge du focus en haut seulement", m.padding[0] > 0 && m.padding[1] === 0, m);
      const { nodes } = (await b.send("Accessibility.getFullAXTree")).result;
      const named = nodes.find((n) => !n.ignored && n.role?.value === "progressbar")?.name?.value;
      check("le lecteur d'écran nomme la barre", named === "Lecture", named);
      // 2. À mi-page : la barre reste en haut et suit la place ; le bouton reste en bas ; le bouton
      // rond du moteur monte au-dessus de lui.
      await p.value("scrollTo(0, (document.documentElement.scrollHeight - innerHeight) / 2)");
      await p.until(`document.querySelector('[data-state="scroll"]')?.textContent === "50"`, 3000);
      await pause(400);
      m = await seen();
      check("à mi-page : la barre suit, collée en haut", m.percent === "50" && m.value === 50 && m.bar?.top === 0, m);
      check("à mi-page : le bouton, collé en bas", m.back !== null && m.back.bottom === m.height, m);
      check("le bouton rond du moteur au-dessus du bouton du bas", m.back !== null && Math.abs(m.menu - (12 + m.back.height)) <= 1, m);
      summary.half = `barre en haut à ${m.bar?.top}px, « ${m.percent} % », bouton collé en bas (${m.back?.height}px), menu à ${m.menu}px du bas`;
      // 3. Au plus dix fois par seconde : cinquante pas de défilement en une seconde ; la dernière
      // place est donnée.
      await p.value("window.__holoScrollsGiven = 0");
      // Le temps vraiment passé compte : une machine lente met plus d'une seconde à faire les cinquante pas.
      const elapsed = await p.value("new Promise((done) => { const start = performance.now(); let step = 0; const t = setInterval(() => { scrollBy(0, step < 25 ? 9 : -5); if (++step === 50) { clearInterval(t); done(performance.now() - start); } }, 20); })");
      await pause(500);
      const given = await p.value("window.__holoScrollsGiven");
      m = await seen();
      check("au plus dix fois par seconde, et la dernière place", given >= 4 && given <= Math.ceil(elapsed / 100) + 2 && Number(m.percent) === m.real, { given, elapsed: Math.round(elapsed), percent: m.percent, real: m.real });
      summary.rate = `${given} fois pour 50 pas en ${Math.round(elapsed)} ms, finie à ${m.percent} %`;
      // 4. « Retour en haut » : en haut, la barre vide, le bouton parti.
      await tapAt('[data-sticky="bottom"] a');
      await p.until(`scrollY === 0 && document.querySelector('[data-state="scroll"]')?.textContent === "0"`, 3000);
      await pause(400);
      m = await seen();
      check("retour en haut", m.scrollY === 0 && m.percent === "0" && m.back === null, m);
      // 5. Un lien vers un endroit de la page : le titre s'arrête juste sous la barre (scroll-padding).
      await tapAt('a[href="#Quand"]');
      await pause(700);
      const title = await p.value(`(() => { const t = document.getElementById("Quand").getBoundingClientRect(); const bar = document.querySelector('[data-sticky="top"]').getBoundingClientRect(); return [Math.round(t.top), Math.round(bar.bottom)]; })()`);
      check("le titre « Quand les voir » juste sous la barre", title[0] >= title[1] && title[0] - title[1] <= 32, title);
      summary.anchor = `titre à ${title[0]}px, barre jusqu'à ${title[1]}px`;
      // 6. Le clavier : Tab de lien en lien jusqu'au bas, puis Maj + Tab jusqu'en haut ; aucun lien
      // ne passe sous un bloc qui reste à l'écran (WCAG 2.4.11).
      await p.value(`scrollTo(0, 0); document.querySelector('a[href="#CeQueCest"]').focus()`);
      await pause(400);
      const walk = [];
      const step = async (back) => {
        await tab(back);
        const f = await focused();
        walk.push(f.name);
        if (f.hidden) faults.push(`${back ? "Maj + Tab" : "Tab"} : « ${f.name} » ${f.hidden} ${JSON.stringify(f.at)}`);
        return f.name;
      };
      for (let i = 0; i < 10 && (await step(false)) !== "Leçon 1 : une page →"; i++);
      for (let i = 0; i < 10 && (await step(true)) !== "Ce que c'est"; i++);
      check("le clavier passe par le bas de la page et revient en haut", walk.includes("Leçon 1 : une page →") && walk.some((n) => n.startsWith("Les Perséides")) && walk.at(-1) === "Ce que c'est", walk);
      summary.keyboard = walk.join(" → ");
      // Le piège connu d'un en-tête collé : un lien à l'écran, mais sous la barre (ou sous le bouton
      // du bas). Le navigateur le croit visible et ne défile pas ; la marge laissée au focus le fait
      // défiler. On pose le lien sous la barre, puis sous le bouton, et on y va avec Tab.
      const wiki = `document.querySelector('a[href^="https://fr.wikipedia"]')`;
      const under = [];
      for (const [edge, at] of [["top", "20"], ["bottom", "innerHeight - 20"]]) {
        await p.value(`scrollBy(0, ${wiki}.getBoundingClientRect().${edge} - (${at}))`);
        await pause(500);
        await p.value(`document.querySelector('a[href="#Ou"]').focus({ preventScroll: true })`);
        await tab(false);
        const f = await focused();
        under.push(`${edge === "top" ? "sous la barre" : "sous le bouton"} : ${f.hidden || "visible"}`);
        if (!f.name.startsWith("Les Perséides") || f.hidden) faults.push(`Tab vers un lien posé ${edge === "top" ? "sous la barre" : "sous le bouton du bas"} : « ${f.name} » ${f.hidden} ${JSON.stringify(f.at)}`);
      }
      // Un bloc qui apparaît (le bouton du bas, quand on passe 10 %) sur ce qui a le focus : ce
      // qu'on voyait revient au-dessus de lui.
      await p.value(`scrollTo(0, 0)`);
      await pause(500);
      await p.value(`${wiki}.focus({ preventScroll: true }); scrollBy(0, ${wiki}.getBoundingClientRect().bottom - (innerHeight - 20))`);
      await pause(700);
      const appeared = await focused();
      under.push(`un bouton qui apparaît : ${appeared.hidden || "visible"}`);
      if (appeared.hidden) faults.push(`un bloc qui apparaît sur le lien qui a le focus : « ${appeared.name} » ${appeared.hidden} ${JSON.stringify(appeared.at)}`);
      summary.under = under.join(", ");
      // L'audit d'accessibilité de la page, descendue : la barre collée en haut et le bouton collé
      // en bas, tous deux par-dessus le texte (leur fond, leur contraste, le nom de la barre).
      const { createRequire } = await import("node:module");
      let axe;
      try { axe = readFileSync(createRequire(join(engine, "x.js")).resolve("axe-core/axe.min.js"), "utf8"); }
      catch { return [false, "axe-core absent : « npm install --no-save axe-core@4.10.3 », dans moteur/"]; }
      await p.value(`${axe}\n;window.axe.version`);
      const stuckCount = await p.value(`[...document.querySelectorAll("[data-sticky]")].filter((s) => s.getBoundingClientRect().height > 0 && getComputedStyle(s).position === "sticky").length`);
      const violations = await p.value(`window.axe.run(document, { resultTypes: ["violations"] }).then((r) => r.violations.map((v) => v.id + " : " + v.nodes.map((n) => n.html.slice(0, 100)).join(" | ")))`);
      check("axe-core, la barre et le bouton collés à l'écran : aucun défaut", stuckCount === 2 && violations.length === 0, { stuckCount, violations });
      summary.axe = violations.length ? violations.join(" ; ") : `aucun défaut, ${stuckCount} blocs collés à l'écran`;
      errors("ordinateur");
      // 7. Sur un téléphone (360 × 780) : la barre prend moins du cinquième de l'écran ; rien ne déborde.
      await b.send("Emulation.setDeviceMetricsOverride", { width: 360, height: 780, deviceScaleFactor: 2, mobile: true });
      await p.open(lesson);
      await p.until("window.__holoStarted");
      await p.value("scrollTo(0, 900)");
      await pause(500);
      m = await seen();
      check("téléphone : la barre collée en haut, au plus le cinquième de l'écran", m.bar?.top === 0 && m.bar.height <= 156 && m.bar.position === "sticky", m);
      check("téléphone : rien ne déborde", m.wide <= 360, m);
      summary.phone = `barre de ${m.bar?.height}px sur 780`;
      errors("téléphone");
      // Un en-tête beaucoup trop haut : borné au cinquième de l'écran, et ce qui dépasse défile dedans.
      await p.open(tall);
      const header = await p.value(`(() => { const h = document.querySelector('[data-sticky="top"]'); return { height: Math.round(h.getBoundingClientRect().height), inside: h.scrollHeight > h.clientHeight, screen: innerHeight }; })()`);
      check("téléphone : l'en-tête trop haut, borné au cinquième de l'écran", header.height <= Math.ceil(header.screen / 5) && header.inside, header);
      summary.tall = `en-tête borné à ${header.height}px (contenu plus haut : ${header.inside})`;
      // 8. Le clavier de l'écran, ouvert sur le champ (sa hauteur imitée) : l'en-tête reprend sa
      // place dans la page, sans marge du focus ; refermé, il reste de nouveau en haut.
      await p.value(`document.querySelector('[data-bind="nom"]').focus()`);
      await pause(300);
      await p.value(`Object.defineProperty(visualViewport, "height", { configurable: true, get: () => 380 }); visualViewport.dispatchEvent(new Event("resize"))`);
      await pause(200);
      const underKeyboard = `[document.documentElement.classList.contains("holo-keyboard"), getComputedStyle(document.querySelector('[data-sticky="top"]')).position, getComputedStyle(document.documentElement).scrollPaddingTop]`;
      const typing = await p.value(underKeyboard);
      await p.value(`delete visualViewport.height; visualViewport.dispatchEvent(new Event("resize"))`);
      await pause(200);
      const closed = await p.value(underKeyboard);
      check("le clavier de l'écran : l'en-tête reprend sa place, puis reste de nouveau", typing[0] === true && typing[1] === "static" && typing[2] === "auto" && closed[0] === false && closed[1] === "sticky" && closed[2] !== "auto", { typing, closed });
      errors("en-tête trop haut");
      // 9. Un écran trop bas : un téléphone couché (780 × 360), une page grossie à 200 % (un
      // téléphone de 720 × 800 vu à 360 × 400). Rien ne reste : la barre reprend sa place dans la
      // page, part avec elle quand on descend, et aucune marge du focus ne reste posée.
      const low = [];
      for (const [name, metrics] of [["couché", { width: 780, height: 360, deviceScaleFactor: 2, mobile: true }], ["grossi à 200 %", { width: 360, height: 400, deviceScaleFactor: 2, mobile: true }]]) {
        await b.send("Emulation.setDeviceMetricsOverride", metrics);
        await p.open(lesson);
        await p.until("window.__holoStarted");
        await p.value("scrollTo(0, 300)");
        await pause(400);
        m = await seen();
        check(`${name} : rien ne reste, la barre part avec la page, pas de marge du focus`, m.bar?.position === "static" && m.bar.top < 0 && m.padding[0] === 0 && m.padding[1] === 0, m);
        low.push(`${name} : ${m.bar?.position}, barre à ${m.bar?.top}px, marge ${m.padding?.join("/")}`);
      }
      // Sur papier (le média « print », émulé sur l'écran d'ordinateur) : la barre garde sa place,
      // sans marge du focus ; de retour à l'écran, elle reste de nouveau.
      await b.send("Emulation.setDeviceMetricsOverride", { width: 1000, height: 700, deviceScaleFactor: 1, mobile: false });
      await p.open(lesson);
      await p.until("window.__holoStarted");
      const stuck = `[getComputedStyle(document.querySelector('[data-sticky="top"]')).position, getComputedStyle(document.documentElement).scrollPaddingTop]`;
      await b.send("Emulation.setEmulatedMedia", { media: "print" });
      await pause(200);
      const paper = await p.value(stuck);
      await b.send("Emulation.setEmulatedMedia", { media: "" });
      await pause(200);
      const screen = await p.value(stuck);
      check("sur papier : la barre garde sa place ; à l'écran, elle reste de nouveau", paper[0] === "static" && paper[1] === "auto" && screen[0] === "sticky" && screen[1] !== "auto", { paper, screen });
      summary.low = `${low.join(" ; ")} ; papier : ${paper[0]}, marge ${paper[1]}`;
      // 10. Sans JavaScript : la barre reste en haut (du CSS), vide ; pas de bouton ; la marge du
      // focus est le cinquième de l'écran.
      await b.send("Emulation.setDeviceMetricsOverride", { width: 1000, height: 700, deviceScaleFactor: 1, mobile: false });
      await b.send("Emulation.setScriptExecutionDisabled", { value: true });
      try {
        await p.open(lesson, 300);
        await p.value("scrollTo(0, 600)");
        await pause(300);
        m = await seen();
        check("sans JavaScript : la barre reste en haut, vide, sans bouton, et la marge du focus est prévue", m.bar?.position === "sticky" && m.bar.top === 0 && m.value === 0 && m.back === null && m.padding[0] >= 140, m);
        summary.withoutScript = `barre à ${m.bar?.top}px, valeur ${m.value}, marge du focus ${m.padding[0]}px`;
      } finally {
        await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      }
    } finally {
      await b.send("Emulation.setEmulatedMedia", { media: "" });
      await b.send("Emulation.clearDeviceMetricsOverride");
    }
    return [faults.length === 0, faults.length ? faults.join("\n      ") : `ordinateur : ${summary.half} ; ${summary.rate} ; retour en haut ; « Quand les voir » : ${summary.anchor} ; clavier : ${summary.keyboard}, rien de caché ; ${summary.under} ; axe-core : ${summary.axe} ; téléphone : ${summary.phone}, ${summary.tall}, clavier de l'écran : l'en-tête reprend sa place ; écran bas : ${summary.low} ; sans JavaScript : ${summary.withoutScript}`];
  }],
  ["faire vibrer le téléphone : un toucher, une rencontre, le mouvement réduit, un navigateur sans vibreur (leçon 133)", async (p, b) => {
    const lesson = "/exemples/lecons/133-faire-vibrer-le-telephone.holo";
    const status = (name) => `document.querySelector('[data-name="${name}"] [data-capability-status]').textContent`;
    const count = (label) => p.value(`document.getElementById("page").innerText.match(/${label} : (\\d+)/)?.[1] ?? "?"`);
    // Un script posé avant la page : navigator.vibrate est remplacé pour compter les vibrations et
    // garder leur motif, ou retiré, comme sur un iPhone.
    const before = async (source) => (await b.send("Page.addScriptToEvaluateOnNewDocument", { source })).result.identifier;
    const forget = (identifier) => b.send("Page.removeScriptToEvaluateOnNewDocument", { identifier });
    const catchIt = async () => {
      for (let i = 0; i < 24 && (await count("Prises")) === "0"; i++) await p.key("ArrowRight", "ArrowRight", 39);
      return count("Prises");
    };
    let script = await before(`window.__buzz = []; Navigator.prototype.vibrate = function (pattern) { window.__buzz.push(Array.isArray(pattern) ? pattern.join(",") : String(pattern)); return true; };`);
    let phone, reduced;
    try {
      await p.open(lesson);
      if (!(await p.until("window.__holoStarted"))) return [false, "le moteur n'est pas arrivé"];
      // 1. Un clic que la page reçoit sans que le visiteur ait rien touché : l'écran compte, rien ne vibre.
      await p.value(`document.querySelector('[data-name="Vibrer"]').click()`);
      await p.until(`document.getElementById("page").innerText.includes("Vibrations demandées : 1.")`, 5000);
      const untouched = await p.value("window.__buzz.length");
      // 2. Un vrai toucher : une vibration de 200 ms.
      await p.click('[data-name="Vibrer"]');
      await p.until("window.__buzz.length >= 1", 5000);
      // 3. Le jeu : le carré va sur le losange, à la flèche droite ; la rencontre vibre deux fois.
      const caught = await catchIt();
      await p.until("window.__buzz.length >= 2", 5000);
      phone = { untouched, buzz: await p.value("window.__buzz.slice()"), tries: await count("Vibrations demandées"), caught, errors: [...b.errors] };
      // 4. Le mouvement réduit, émulé : rien ne vibre, la zone d'état le dit, et l'écran compte encore.
      await b.send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
      try {
        await p.click('[data-name="Vibrer"]');
        await p.until(`${status("Petite")}.includes("moins de mouvement")`, 5000);
        reduced = { buzz: await p.value("window.__buzz.length"), said: await p.value(status("Petite")), tries: await count("Vibrations demandées") };
      } finally {
        await b.send("Emulation.setEmulatedMedia", { features: [] });
      }
    } finally {
      await forget(script);
    }
    // 5. Un navigateur sans vibreur (l'iPhone) : rien ne casse ; le toucher et la rencontre comptent à l'écran.
    script = await before("delete Navigator.prototype.vibrate;");
    let iphone, violations;
    try {
      await p.open(lesson);
      if (!(await p.until("window.__holoStarted"))) return [false, "le moteur n'est pas arrivé (sans vibreur)"];
      await p.click('[data-name="Vibrer"]');
      await p.until(`${status("Petite")}.includes("ne fait pas vibrer")`, 5000);
      const caught = await catchIt();
      iphone = { vibrate: await p.value("typeof navigator.vibrate"), said: await p.value(status("Petite")), saidGame: await p.value(status("Double")), tries: await count("Vibrations demandées"), caught, errors: [...b.errors] };
      // L'audit d'accessibilité de la page : les deux vibrations, leur zone d'état, le plateau.
      const { createRequire } = await import("node:module");
      let axe;
      try { axe = readFileSync(createRequire(join(engine, "x.js")).resolve("axe-core/axe.min.js"), "utf8"); }
      catch { return [false, "axe-core absent : « npm install --no-save axe-core@4.10.3 », dans moteur/"]; }
      await p.value(`${axe}\n;window.axe.version`);
      violations = await p.value(`window.axe.run(document, { resultTypes: ["violations"] }).then((r) => r.violations.map((v) => v.id + " : " + v.nodes.map((n) => n.html.slice(0, 100)).join(" | ")))`);
    } finally {
      await forget(script);
    }
    const ok = phone.untouched === 0 && phone.buzz[0] === "200" && phone.buzz[1] === "100,80,100" && phone.tries === "2" && phone.caught === "1" && phone.errors.length === 0
      && reduced.buzz === phone.buzz.length && reduced.said === "Pas de vibration : tu as demandé moins de mouvement." && reduced.tries === "3"
      && iphone.vibrate === "undefined" && iphone.said === "Ce navigateur ne fait pas vibrer." && iphone.saidGame === "Ce navigateur ne fait pas vibrer." && iphone.tries === "1" && iphone.caught === "1" && iphone.errors.length === 0
      && violations.length === 0;
    return [ok, `avant tout toucher : ${phone.untouched} vibration ; puis ${phone.buzz.map((m) => `[${m}]`).join(", ")} (le toucher, puis la rencontre), ${phone.tries} demandées, ${phone.caught} prise ; mouvement réduit : « ${reduced.said} », ${reduced.buzz - phone.buzz.length} vibration de plus, ${reduced.tries} demandées ; sans vibreur : navigator.vibrate ${iphone.vibrate}, « ${iphone.said} », ${iphone.tries} demandée, ${iphone.caught} prise, ${iphone.errors.length} erreur ; axe-core : ${violations.length ? violations.join(" ; ") : "zéro défaut"}`];
  }],
  ["mélanger des sons : deux à la fois, le fondu qui monte puis descend, le volume qui suit sa glissière (leçon 135)", async (p, b) => {
    await p.open("/exemples/lecons/135-melanger-des-sons.holo");
    const audio = (name) => `document.querySelector('audio[data-name="${name}"]')`;
    // Ce qu'on entend d'un son : son volume × son fondu, lus dans les gains du mélangeur (Web Audio).
    const heard = (name) => p.value(`window.__holoMixer?.(${JSON.stringify(name)}).heard ?? -1`);
    const shows = (text) => p.value(`document.getElementById("page").innerText.includes(${JSON.stringify(text)})`);
    // Le lecteur d'écran : chaque glissière a son nom. Et l'audit axe-core de la leçon.
    const { nodes } = (await b.send("Accessibility.getFullAXTree")).result;
    const sliders = nodes.filter((n) => !n.ignored && n.role?.value === "slider").map((n) => n.name?.value).join(", ");
    await p.value(readFileSync(join(engine, "node_modules", "axe-core", "axe.min.js"), "utf8") + "\n;0");
    const faults = await p.value(`axe.run(document, { resultTypes: ["violations"] }).then((r) => r.violations.map((v) => v.id).join(", "))`);
    // Un premier toucher fait venir le moteur : « Tout arrêter » n'a rien à arrêter, et ne
    // réveille pas le mélangeur.
    await p.click('[data-name="Silence"]');
    const arrived = await p.until(`${audio("Pluie")}.dataset.level === "0.6"`);
    const asleep = await p.value(`window.__holoMixer("Pluie").state`);
    // La pluie monte en 2 secondes jusqu'à son volume, 60 sur 100 : écoutée à plusieurs moments.
    await p.click('[data-name="LancerPluie"]');
    await p.until(`!${audio("Pluie")}.paused`, 5000);
    const rising = [];
    for (let i = 0; i < 4; i++) {
      rising.push(await heard("Pluie"));
      await pause(400);
    }
    const climbing = rising.filter((v) => v < 0.595);
    const rose = climbing.length >= 3 && climbing[0] < 0.3 && climbing.every((v, i) => i === 0 || v > climbing[i - 1]);
    const full = await p.until(`Math.abs(window.__holoMixer("Pluie").heard - 0.6) < 0.005`, 4000);
    // Le vent part à son tour : les deux jouent ensemble, et la pluie ne baisse pas.
    await p.click('[data-name="LancerVent"]');
    const together = await p.until(`!${audio("Pluie")}.paused && !${audio("Vent")}.paused && window.__holoMixer("Vent").heard > 0`, 5000);
    const rainKept = await heard("Pluie");
    const said = (await shows("La pluie joue.")) && (await shows("Le vent souffle."));
    // La glissière de la pluie, au clavier : Page suivante quatre fois, de 60 à 20. Le volume glisse
    // jusque-là, et la page montre la valeur.
    await p.value(`document.querySelector('input[data-bind="pluie"]').focus()`);
    for (let i = 0; i < 4; i++) await p.key("PageDown", "PageDown", 34);
    const followed = await p.until(`Math.abs(window.__holoMixer("Pluie").heard - 0.2) < 0.005`, 3000);
    const twenty = await shows("20 sur 100");
    // Arrêter la pluie : elle descend pendant 2 secondes en jouant encore, puis se met en pause et
    // revient au début. Le vent continue.
    await p.click('[data-name="ArreterPluie"]');
    const falling = [];
    for (let i = 0; i < 3; i++) {
      await pause(450);
      falling.push([await heard("Pluie"), await p.value(`${audio("Pluie")}.paused`)]);
    }
    const fading = falling.filter(([, paused]) => !paused).map(([v]) => v);
    const fell = fading.length >= 2 && fading.every((v, i) => v < 0.2 && v > 0 && (i === 0 || v < fading[i - 1]));
    const stopped = await p.until(`${audio("Pluie")}.paused && ${audio("Pluie")}.currentTime === 0`, 3000);
    const windOn = await p.value(`!${audio("Vent")}.paused`);
    // Tout arrêter : le vent s'éteint en 3 secondes, puis le mélangeur s'endort.
    await p.click('[data-name="Silence"]');
    const sleeping = await p.until(`${audio("Vent")}.paused && window.__holoMixer("Vent").state === "suspended"`, 6000);
    const round = (v) => Number(v).toFixed(2).replace(".", ",");
    const ok = sliders === "Volume de la pluie, Volume du vent" && faults === "" && arrived && asleep === "absent" && rose && full && together
      && Math.abs(rainKept - 0.6) < 0.005 && said && followed && twenty && fell && stopped && windOn && sleeping;
    return [ok, `glissières : ${sliders} ; axe-core : ${faults || "aucun défaut"} ; mélangeur avant le premier son : ${asleep} ; la pluie monte : ${rising.map(round).join(" → ")}, puis 0,60 : ${full} ; avec le vent, les deux jouent : ${together} (pluie à ${round(rainKept)}) ; écrit à l'écran : ${said} ; glissière à 20 au clavier : ${followed} (${twenty}) ; arrêtée, elle descend en jouant : ${falling.map(([v, paused]) => `${round(v)}${paused ? " (en pause)" : ""}`).join(" → ")}, puis en pause au début : ${stopped} ; le vent continue : ${windOn} ; tout arrêté, le mélangeur s'endort : ${sleeping}`];
  }],
  ["un son ne part jamais avant un geste du visiteur, même là où le navigateur le permettrait (ADR-112)", async (p) => {
    // Ce Chrome joue un son sans geste (--autoplay-policy=no-user-gesture-required) : seul le moteur
    // peut l'empêcher. La règle de temps de la page demande deux sons toutes les 400 ms.
    await p.open("/exemples/.essais-navigateur/son-avant-un-geste.holo", 600);
    const sounds = `[...document.querySelectorAll("audio")].map((a) => a.dataset.name + (a.played.length ? " entendu" : " muet")).join(", ")`;
    const asked = await p.until(`/demandé les sons ([3-9]|\\d\\d+) fois/.test(document.getElementById("page").innerText)`, 15000);
    const before = await p.value(sounds);
    const mixer = await p.value(`window.__holoMixer?.("Tac").state`);
    // Un toucher : les sons partent au tour suivant de la règle.
    await p.click('[data-name="Toucher"]');
    const heard = await p.until(`[...document.querySelectorAll("audio")].every((a) => a.played.length > 0)`, 5000);
    const after = await p.value(sounds);
    const ok = asked && before === "Tic muet, Tac muet" && mixer === "absent" && heard;
    return [ok, `demandés au moins 3 fois sans geste : ${asked} ; avant un geste : ${before}, mélangeur ${mixer} ; après un toucher : ${after}`];
  }],
  ["se souvenir le temps d'une visite : d'une page à l'autre du site, pas dans un autre onglet (leçon 136)", async (p, b) => {
    const first = "/exemples/lecons/136-se-souvenir-le-temps-d-une-visite.holo";
    const second = "/exemples/lecons/136-inscription/etape-2.holo";
    const shows = (text) => `document.getElementById("page").innerText.includes(${JSON.stringify(text)})`;
    const stored = (name) => `sessionStorage.getItem("holo-visit:${name}")`;
    const field = (name) => `document.querySelector('[data-bind="${name}"]').value`;
    const ticked = (option) => `document.querySelector('input[data-bind="atelier"][value="${option}"]').checked`;
    // L'accessibilité des deux pages, remplies : axe-core, la copie locale, comme pour les parcours.
    const axeSource = readFileSync(join(engine, "node_modules", "axe-core", "axe.min.js"), "utf8");
    const audit = async (where) => {
      await p.value(`${axeSource}\n;window.axe.version`);
      return (await p.value('window.axe.run(document, { resultTypes: ["violations"] }).then((r) => r.violations.map((v) => v.id))')).map((id) => `${where} : ${id}`);
    };
    // Ce que l'onglet envoie pendant l'essai : la mémoire de visite ne part jamais au serveur.
    const sent = [];
    await b.send("Network.enable");
    b.on("Network.requestWillBeSent", ({ request }) => sent.push(request));
    try {
      // Les essais d'avant ont ouvert d'autres pages du même site dans cet onglet : on part de rien.
      await p.open(first, 300);
      await p.value("sessionStorage.clear()");
      await p.open(first);
      // Page 1 : un prénom écrit (avant même l'arrivée du moteur), trois personnes, un atelier.
      await p.type('[data-bind="prenom"]', "Zoé");
      await p.until("window.__holoStarted === true", 40000);
      await p.value(`(() => { const i = document.querySelector('[data-bind="personnes"]'); i.value = "3"; i.dispatchEvent(new Event("input", { bubbles: true })); })()`);
      await p.click('input[data-bind="atelier"][value="Poterie"]');
      const written = await p.until(`${stored("prenom")} === '"Zoé"' && ${stored("personnes")} === "3" && ${stored("atelier")} === '"Poterie"'`, 5000);
      const faults = await audit("page 1");
      // Page 2, par le lien, dans le même onglet : la page du serveur arrive avec ses valeurs de
      // départ ; le moteur vient de lui-même, sans geste, et reprend celles de la visite.
      await p.click('a[href$="136-inscription/etape-2.holo"]');
      const followed = await p.until(`location.pathname.endsWith("/136-inscription/etape-2.holo") && window.__holoStarted === true && ${shows("Bonjour Zoé, vous serez 3.")} && ${shows("L'atelier : Poterie.")}`, 40000);
      faults.push(...(await audit("page 2")));
      // L'atelier change à la page 2 ; « Précédent » : la page 1 a suivi. Deux fois : rendue par le
      // cache du navigateur, telle qu'on l'avait quittée ; puis rechargée (un écouteur « unload »
      // interdit au navigateur de la garder), le navigateur remettant dans ses champs ce qu'on y
      // avait écrit avant : la mémoire de visite, plus récente, passe par-dessus.
      const backTo = async (option, before) => {
        await p.click(`input[data-bind="atelier"][value="${option}"]`);
        await p.until(`${stored("atelier")} === '"${option}"'`, 5000);
        await p.value("history.back()");
        const ok = await p.until(`location.pathname.endsWith("136-se-souvenir-le-temps-d-une-visite.holo") && window.__holoStarted === true && ${field("prenom")} === "Zoé" && ${field("personnes")} === "3" && ${ticked(option)} && !${ticked(before)}`, 40000);
        const how = (await p.value(`performance.getEntriesByType("navigation")[0]?.type`)) === "back_forward" ? "rechargée" : "rendue par le cache";
        return [ok && (await p.value(`${stored("atelier")} === '"${option}"'`)), how];
      };
      const [cached, cachedHow] = await backTo("Gravure", "Poterie");
      await p.value(`addEventListener("unload", () => {})`);
      await p.click('a[href$="136-inscription/etape-2.holo"]');
      await p.until(`location.pathname.endsWith("/136-inscription/etape-2.holo") && window.__holoStarted === true && ${shows("L'atelier : Gravure.")}`, 40000);
      const [reloaded, reloadedHow] = await backTo("Aquarelle", "Gravure");
      const back = cached && reloaded;
      const how = `${cachedHow}, puis ${reloadedHow}`;
      // Un autre onglet du même navigateur, à la même adresse : il ne les a pas, et la page n'y
      // fait même pas venir le moteur.
      const other = await b.tab();
      const q = page(other, server.base);
      let fresh = false;
      let engine = "";
      try {
        await q.open(second);
        fresh = await q.until(`${shows("Tu n'as pas encore écrit ton prénom")} && ${shows("Aucun atelier choisi.")} && sessionStorage.length === 0`, 10000);
        engine = String(await q.value("window.__holoStarted === true"));
      } finally {
        await other.close();
      }
      // « Recommencer », à la page 2 : la règle remet les valeurs au départ, et la visite le retient.
      await p.open(second);
      await p.until(`window.__holoStarted === true && ${shows("Bonjour Zoé, vous serez 3.")}`, 40000);
      await p.click('[data-name="Recommencer"]');
      const again = await p.until(`${stored("prenom")} === '""' && ${stored("personnes")} === "1" && ${stored("atelier")} === '""' && ${shows("Tu n'as pas encore écrit ton prénom")}`, 5000);
      // Ce qu'une autre page aurait écrit : un prénom juste, un nombre abîmé, un atelier qui n'est
      // pas une option de celle-ci, suivi d'une ligne qui voudrait glisser six personnes. Le juste
      // est repris ; les autres sont ignorés seuls, sans erreur, et restent tels quels.
      await p.value(`sessionStorage.setItem("holo-visit:prenom", '"Zoé"'); sessionStorage.setItem("holo-visit:personnes", "[abîmé"); sessionStorage.setItem("holo-visit:atelier", '"Pirate"\\npersonnes\\t6')`);
      await p.open(second);
      const damaged = await p.until(`window.__holoStarted === true && ${shows("Bonjour Zoé, vous serez 1.")} && ${shows("Aucun atelier choisi.")}`, 40000);
      const untouched = await p.value(`${stored("personnes")} === "[abîmé" && ${stored("atelier")} === '"Pirate"\\npersonnes\\t6'`);
      const quiet = b.errors.length === 0 && !(await p.value(`document.getElementById("error")?.textContent ?? ""`));
      // Une page du même site où « prenom » est un nombre : le texte retenu y est ignoré, sans
      // erreur, et reste pour la leçon.
      await p.open("/exemples/.essais-navigateur/visite-autre-sorte.holo");
      const otherSort = await p.until(`window.__holoStarted === true && ${shows("Prénom : 0 ; personnes : 1.")}`, 40000) && b.errors.length === 0;
      const kept = await p.value(`${stored("prenom")} === '"Zoé"'`);
      const cookie = await p.value("document.cookie");
      const leaks = sent.filter((r) => r.method !== "GET" || r.hasPostData || /Zo%C3%A9|Zoé|Poterie|Gravure|Aquarelle|Pirate/.test(r.url)).map((r) => `${r.method} ${r.url}`);
      const ok = written && followed && back && fresh && engine === "false" && again && damaged && untouched && quiet && otherSort && kept && !cookie && !leaks.length && !faults.length;
      return [ok, `retenues à la page 1 : ${written} ; la page 2 les a, dans le même onglet : ${followed} ; l'atelier changé à la page 2, la page 1 a suivi : ${back} (${how}) ; un nouvel onglet ne les a pas : ${fresh} (moteur venu : ${engine}) ; recommencer : ${again} ; valeurs abîmées ignorées : ${damaged}, laissées telles quelles : ${untouched}, sans erreur : ${quiet} ; une autre sorte ignorée : ${otherSort}, le prénom gardé pour la leçon : ${kept} ; cookie : ${cookie || "aucun"} ; envoyé au serveur : ${leaks.join(", ") || "rien"} ; axe-core : ${faults.join(", ") || "aucun défaut sur les deux pages"}`];
    } finally {
      b.on("Network.requestWillBeSent");
      await b.send("Network.disable");
    }
  }],
  ["les données d'un autre site, lues par holo serve et jamais par le navigateur, avec et sans JavaScript ; l'interrupteur des essais éteint par défaut (leçon 139, serve)", async (p, b) => {
    // ADR-116. Un faux « autre site » sur ce PC : un petit serveur HTTP de Node, qui note chaque
    // demande reçue. holo serve ne l'atteint que par l'interrupteur des essais, allumé ici
    // seulement dans son environnement (HOLO_TEST_ONLY_INSECURE_SITE=meteo.test:<port>).
    const binary = ["holo", "holo.exe"].map((name) => join(engine, "target", "release", name)).find(existsSync);
    if (!binary) return [false, "holo n'est pas construit : cargo build --release --bin holo"];
    const KEY = "cle-d-essai-7f3a9c41";
    const seen = [];
    const other = createServer((req, res) => {
      seen.push({ url: req.url, headers: req.headers });
      if (req.url.startsWith("/meteo.json")) {
        res.writeHead(200, { "content-type": "application/json" });
        res.end(JSON.stringify({ temperature: 31, sky: "Ensoleillé", account: "compte-42" }));
      } else if (req.url.startsWith("/redirige")) {
        res.writeHead(302, { location: "/meteo.json" });
        res.end();
      } else {
        res.writeHead(404);
        res.end();
      }
    });
    await new Promise((ok) => other.listen(0, "127.0.0.1", ok));
    const otherPort = other.address().port;
    const folder = mkdtempSync(join(tmpdir(), "holo-serve-"));
    const meteo = (from) => [
      'Page(title: "Météo d\'essai", state: State(loading: 1, broken: 0, temperature: 0, sky: ""),',
      `  data: Data(name: Meteo, from: "${from}", every: 600s),`,
      '  children: [',
      '    H1("La météo"),',
      '    If(loading, is: 1, children: [ P("Chargement…") ]),',
      '    If(broken, is: 1, children: [ P("La météo n\'est pas arrivée."), Button(name: Retry, text: "Réessayer") ]),',
      '    P("{temperature} degrés, {sky}"),',
      '    Button(name: Again, text: "Relire"),',
      '  ],',
      '  rules: [ On(Meteo.done, effect: [loading.set(0), broken.set(0)]), On(Meteo.failed, effect: [loading.set(0), broken.set(1)]),',
      '    On(Retry.tap, effect: [loading.set(1), broken.set(0), Meteo.refresh]), On(Again.tap, effect: Meteo.refresh) ],',
      ')',
    ].join("\n");
    writeFileSync(join(folder, "meteo.holo"), meteo("https://meteo.test/meteo.json?city=Kinshasa"));
    writeFileSync(join(folder, "redirige.holo"), meteo("https://meteo.test/redirige"));
    writeFileSync(join(folder, "139-les-donnees-d-un-autre-site.holo"), readFileSync(join(repo, "exemples", "lecons", "139-les-donnees-d-un-autre-site.holo")));
    mkdirSync(join(folder, "holo-data"));
    writeFileSync(join(folder, "holo-data", "sites.txt"), `# Les autres sites de l'essai\nmeteo.test X-Api-Key: ${KEY}\n`);
    // holo serve, avec ou sans l'interrupteur ; on attend qu'il ait dit ce qu'il permet.
    const serve = async (switched) => {
      const port = 24000 + Math.floor(Math.random() * 2000);
      const env = { ...process.env };
      delete env.HOLO_TEST_ONLY_INSECURE_SITE;
      if (switched) env.HOLO_TEST_ONLY_INSECURE_SITE = `meteo.test:${otherPort}`;
      const server = spawn(binary, ["serve", folder, String(port)], { cwd: engine, env, stdio: ["ignore", "pipe", "pipe"] });
      let output = "";
      server.stdout.on("data", (d) => { output += d; });
      server.stderr.on("data", (d) => { output += d; });
      for (let i = 0; i < 100 && !output.includes("Autres sites"); i++) await pause(100);
      return { base: `http://localhost:${port}`, output: () => output, stop: () => server.kill() };
    };
    const has = (words) => `document.getElementById("page").innerText.includes(${JSON.stringify(words)})`;
    const asked = (start) => seen.filter((r) => r.url.startsWith(start)).length;
    const browsed = [];
    b.on("Network.requestWillBeSent", ({ request }) => browsed.push(request.url));
    await b.send("Network.enable");
    let off, on;
    try {
      // 1. Éteint par défaut : sans la variable, holo serve ne dit rien de l'interrupteur, la page
      //    dit l'échec, et le faux site ne reçoit rien.
      off = await serve(false);
      const quietOff = !off.output().includes("ESSAIS SEULEMENT");
      const q0 = page(b, off.base);
      await q0.open("/meteo.holo", 300);
      const failedOff = await q0.until(`window.__holoStarted && ${has("La météo n'est pas arrivée.")}`, 40000);
      const nothingAsked = seen.length === 0;
      off.stop();
      // 2. Allumé : holo serve l'annonce, dit les sites permis (sans la clé) et la leçon qui lit un
      //    site non permis.
      on = await serve(true);
      const said = on.output();
      const announced = said.includes(`ESSAIS SEULEMENT : HOLO_TEST_ONLY_INSECURE_SITE=meteo.test:${otherPort}`)
        && said.includes("Autres sites    : 1 permis (holo-data/sites.txt) : meteo.test (une clé, en en-tête)")
        && said.includes("/139-les-donnees-d-un-autre-site.holo lit fr.wikipedia.org, qui n'est pas dans holo-data/sites.txt");
      const q = page(b, on.base);
      browsed.length = 0;
      await q.open("/meteo.holo", 300);
      const arrived = await q.until(`window.__holoStarted && ${has("31 degrés, Ensoleillé")} && !${has("Chargement")}`, 40000);
      // Relire trois fois, recharger trois fois : rien de plus vers l'autre site.
      for (let i = 0; i < 3; i++) {
        await pause(1100);
        await q.click('[data-name="Again"]');
      }
      await pause(800);
      const noFailure = await q.value(`!${has("La météo n'est pas arrivée.")} && ${has("31 degrés, Ensoleillé")}`);
      for (let i = 0; i < 3; i++) await q.open("/meteo.holo", 300);
      await q.until(`window.__holoStarted && ${has("31 degrés, Ensoleillé")}`, 40000);
      const reads = browsed.filter((url) => url.endsWith("?remote-data")).length;
      const once = asked("/meteo.json") === 1;
      // Ce que le faux site a reçu : notre User-Agent, la clé, et rien du visiteur.
      const first = seen.find((r) => r.url.startsWith("/meteo.json"));
      const h = first?.headers ?? {};
      const honest = first?.url === "/meteo.json?city=Kinshasa" && String(h["user-agent"]).startsWith("HoloCode/") && h["x-api-key"] === KEY && h.accept === "application/json"
        && !("cookie" in h) && !("referer" in h) && !("x-forwarded-for" in h) && !("origin" in h);
      // Le navigateur n'a parlé qu'à holo serve ; la page et ses données n'ont ni la clé, ni le reste de la réponse.
      const elsewhere = browsed.filter((url) => !url.startsWith(on.base) && !/^(data|blob|about):/.test(url));
      const html = await q.value(`fetch(location.pathname, { headers: { accept: "text/html" } }).then((r) => r.text())`);
      const json = await q.value(`fetch(location.pathname + "?remote-data", { headers: { accept: "application/json" } }).then((r) => r.text())`);
      const secret = [html, json, on.output()].some((text) => text.includes(KEY)) || json.includes("compte-42") || html.includes("compte-42");
      const exact = json === JSON.stringify({ temperature: 31, sky: "Ensoleillé" });
      // 3. Sans JavaScript : la page arrive avec ses données, et « Relire » part au serveur, qui
      //    répond depuis ce qu'il garde.
      let withoutScript = false;
      try {
        await b.send("Emulation.setScriptExecutionDisabled", { value: true });
        await q.open("/meteo.holo", 300);
        const served = await q.value(has("31 degrés, Ensoleillé"));
        await q.click('[data-name="Again"]');
        withoutScript = served && (await q.until(`document.readyState === "complete" && ${has("31 degrés, Ensoleillé")}`, 5000));
      } finally {
        await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      }
      const stillOnce = asked("/meteo.json") === 1;
      // 4. Une redirection n'est jamais suivie ; la leçon 139, dont le site n'est pas permis, dit l'échec.
      await q.open("/redirige.holo", 300);
      const redirected = (await q.until(`window.__holoStarted && ${has("La météo n'est pas arrivée.")}`, 40000)) && asked("/redirige") === 1 && asked("/meteo.json") === 1;
      await q.open("/139-les-donnees-d-un-autre-site.holo", 300);
      const lesson = (await q.until(`window.__holoStarted && ${has("Le résumé n'est pas arrivé.")}`, 40000)) && b.errors.length === 0;
      const journal = on.output();
      const told = journal.includes("Autre site      : /meteo.holo : meteo.test a répondu") && journal.includes("meteo.test répond par une redirection (302), qui n'est jamais suivie")
        && journal.includes("« fr.wikipedia.org » n'est pas dans holo-data/sites.txt");
      const ok = quietOff && failedOff && nothingAsked && announced && arrived && noFailure && reads >= 4 && once && honest && elsewhere.length === 0 && !secret && exact && withoutScript && stillOnce && redirected && lesson && told;
      return [ok, `éteint par défaut : rien d'annoncé ${quietOff}, échec ${failedOff}, rien demandé ${nothingAsked} ; allumé et annoncé : ${announced} ; arrivées : ${arrived}, relues sans échec : ${noFailure} ; ${reads} lectures par holo serve (?remote-data : ${json}), ${asked("/meteo.json")} demande(s) à l'autre site ; en-têtes honnêtes, clé en en-tête, rien du visiteur : ${honest} ; le navigateur ailleurs : ${elsewhere.length ? elsewhere.join(", ") : "jamais"} ; clé ou reste de la réponse dans la page, ses données ou le journal : ${secret} ; sans JavaScript : ${withoutScript} (toujours ${asked("/meteo.json")} demande) ; redirection non suivie : ${redirected} ; leçon 139 sans site permis : ${lesson}${b.errors.length ? ` (${b.errors.join(" | ")})` : ""} ; le journal le dit : ${told}`];
    } finally {
      b.on("Network.requestWillBeSent", null);
      await b.send("Network.disable");
      off?.stop();
      on?.stop();
      other.close();
      await pause(300);
      try { rmSync(folder, { recursive: true, force: true }); } catch { /* tant pis */ }
    }
  }],
  ["une page dans la page : rien vers l'autre site avant le toucher ; au toucher, au clavier comme au doigt, la page intégrée enfermée, avec son titre, et le clavier y entre ; frame-src ; sans JavaScript, un lien (leçon 140, serve)", async (p, b) => {
    // ADR-117. Ce conteneur n'atteint ni OpenStreetMap ni YouTube : Chrome arrête chaque demande vers
    // « l'autre site » (Fetch) et lui donne une fausse page, qui dit au parent ce qu'elle voit et ce
    // qu'on lui refuse. holo serve sert la leçon, pour éprouver aussi sa règle des cadres.
    const lesson = "140-une-page-dans-la-page.holo";
    const map = "https://www.openstreetmap.org/export/embed.html?bbox=-117.1570%2C32.7310%2C-117.1410%2C32.7400&layer=mapnik";
    const video = "https://www.youtube-nocookie.com/embed/jNQXAC9IVRw";
    const elsewhere = /^https:\/\/(www\.openstreetmap\.org|www\.youtube-nocookie\.com|pirate\.example\.org)\//;
    const fake = `<!doctype html><meta charset="utf-8"><title>Faux site</title><button>Dans la page intégrée</button><script>
      const tell = (what) => parent.postMessage({ fake: true, ...what }, "*");
      let page; try { page = parent.document.title; } catch { page = "fermée"; }
      const features = document.featurePolicy ? document.featurePolicy.allowedFeatures().filter((f) => ["fullscreen", "camera", "microphone", "geolocation", "autoplay", "payment"].includes(f)).sort().join(",") : "?";
      tell({ ready: true, referrer: document.referrer, page, features, focus: document.hasFocus() });
      addEventListener("focus", () => tell({ focus: true }));
      navigator.geolocation.getCurrentPosition(() => tell({ position: "donnée" }), (e) => tell({ position: "refusée (" + e.code + ")" }));
      // Une touche est un vrai geste du visiteur : sans l'enfermement, la page intégrée pourrait alors
      // ouvrir une fenêtre, et emmener la page de l'auteur ailleurs.
      addEventListener("keydown", (e) => {
        let popup; try { popup = String(window.open("https://pirate.example.org/fenetre")); } catch { popup = "refusée"; }
        tell({ key: e.key, popup });
        try { top.location.href = "https://pirate.example.org/dessus"; } catch { /* la page de l'auteur reste */ }
      });
    </script>`;
    const served = await startHoloServe([lesson, "140-carte.svg", "140-video.svg"]);
    const q = page(b, served.base);
    const asked = [];
    const sent = [];
    const axeSource = readFileSync(join(engine, "node_modules", "axe-core", "axe.min.js"), "utf8");
    const audit = async (where) => {
      await q.value(`${axeSource}\n;window.axe.version`);
      // Les fausses pages n'ont pas axe-core : l'audit ne les attend pas ; il vérifie le titre de chaque cadre.
      return (await q.value('window.axe.run(document, { resultTypes: ["violations"], iframes: false }).then((r) => r.violations.map((v) => v.id))')).map((id) => `${where} : ${id}`);
    };
    const listen = () => q.value(`(() => { window.__fake = []; addEventListener("message", (e) => { if (e.data?.fake) window.__fake.push({ from: e.origin, ...e.data }); }); window.__refused = []; addEventListener("securitypolicyviolation", (e) => window.__refused.push(e.violatedDirective + " " + e.blockedURI)); })()`);
    const said = (from) => q.value(`JSON.stringify(Object.assign({}, ...(window.__fake ?? []).filter((m) => m.from === ${JSON.stringify(from)})))`).then(JSON.parse);
    const centre = (selector) => q.value(`(() => { const e = document.querySelector(${JSON.stringify(selector)}); e.scrollIntoView({ block: "center" }); const r = e.getBoundingClientRect(); return [r.left + r.width / 2, r.top + r.height / 2]; })()`);
    try {
      await b.send("Network.enable");
      b.on("Network.requestWillBeSent", ({ request }) => sent.push(request.url));
      b.on("Fetch.requestPaused", ({ requestId, request, resourceType }) => {
        asked.push({ url: request.url, referer: request.headers.Referer ?? "(aucun)", resourceType });
        b.send("Fetch.fulfillRequest", { requestId, responseCode: 200, responseHeaders: [{ name: "Content-Type", value: "text/html; charset=utf-8" }], body: Buffer.from(fake).toString("base64") });
      });
      await b.send("Fetch.enable", { patterns: ["https://www.openstreetmap.org/*", "https://www.youtube-nocookie.com/*", "https://pirate.example.org/*"].map((urlPattern) => ({ urlPattern })) });
      // La règle des cadres que holo serve donne au navigateur.
      const policy = (await fetch(`${served.base}/${lesson}`, { headers: { accept: "text/html" } })).headers.get("content-security-policy");
      await q.open(`/${lesson}`);
      await listen();
      // 1. Avant le toucher : rien vers l'autre site. Ni demande, ni cadre, ni connexion préparée,
      // ni adresse de l'autre site dans un src ou un href ; les images de la façade sont de holo serve.
      const before = [asked.length, sent.filter((url) => elsewhere.test(url)).length];
      const untouched = await q.value(`document.querySelectorAll('iframe, link[rel~="preconnect"], link[rel~="dns-prefetch"], link[rel~="prefetch"], link[rel~="preload"], [src*="openstreetmap"], [src*="youtube"], [href*="openstreetmap"], [href*="youtube"], [srcset]').length === 0`);
      const images = await q.value(`[...document.querySelectorAll(".holo-embed-image")].map((i) => new URL(i.src).origin === location.origin).join(",")`);
      const shown = await q.value(`[...document.querySelectorAll(".holo-embed-load")].map((f) => !f.hidden && f.offsetHeight > 0).join(",")`);
      // 2. Le lecteur d'écran : deux vrais boutons, nommés avec le titre, puis le site qui se chargera.
      const { nodes } = (await b.send("Accessibility.getFullAXTree")).result;
      const buttons = nodes.filter((n) => !n.ignored && n.role?.value === "button" && /^(Carte|Vidéo) : /.test(n.name?.value ?? "")).map((n) => n.name.value);
      const faults = await audit("avant le toucher");
      // 3. Une taille qui suit l'écran : sur un téléphone de 360 px, la carte en 16/9, la vidéo en
      // 4/3 (son style), rien ne déborde.
      let phoneSize = "";
      try {
        await b.send("Emulation.setDeviceMetricsOverride", { width: 360, height: 760, deviceScaleFactor: 2, mobile: true });
        await pause(300);
        phoneSize = await q.value(`[...document.querySelectorAll(".holo-Embed")].map((e) => { const r = e.getBoundingClientRect(); return Math.round(r.width) + "x" + Math.round(r.height); }).join(" ") + " / " + document.documentElement.scrollWidth`);
      } finally {
        await b.send("Emulation.clearDeviceMetricsOverride");
      }
      // 4. Au clavier : Tab jusqu'à la façade de la carte, puis Entrée. (L'onglet au premier plan :
      // un essai d'avant en a ouvert un autre.)
      await b.send("Page.bringToFront");
      for (let i = 0; i < 20 && !(await q.value(`document.activeElement?.matches(".holo-embed-load")`)); i++) await q.key("Tab", "Tab", 9);
      const focusedFacade = await q.value(`document.activeElement?.matches(".holo-embed-load") ? document.activeElement.dataset.label : "(le clavier n'atteint pas la façade)"`);
      if (focusedFacade === "Carte : le zoo de San Diego") await q.key("Enter", "Enter", 13, "\r");
      const mapFrame = await q.until(`document.querySelector('iframe[src^="https://www.openstreetmap.org/"]')`, 5000);
      const mapAttributes = await q.value(`(() => { const f = document.querySelector('iframe[src^="https://www.openstreetmap.org/"]'); return f && [f.getAttribute("sandbox"), f.getAttribute("allow"), f.getAttribute("referrerpolicy"), f.title, f.src].join(" | "); })()`);
      const inside = await q.value(`document.activeElement?.tagName === "IFRAME" && document.activeElement.src.startsWith("https://www.openstreetmap.org/")`);
      const mapReady = await q.until(`(window.__fake ?? []).some((m) => m.ready && m.from === "https://www.openstreetmap.org")`, 10000);
      // Le clavier est dans la page intégrée : elle a le focus, et la touche « k » y arrive.
      const mapFocused = await q.until(`(window.__fake ?? []).some((m) => m.focus && m.from === "https://www.openstreetmap.org")`, 10000);
      await q.key("k", "KeyK", 75, "k");
      const typed = await q.until(`(window.__fake ?? []).some((m) => m.key === "k" && m.from === "https://www.openstreetmap.org")`, 5000);
      await q.until(`(window.__fake ?? []).some((m) => m.position && m.from === "https://www.openstreetmap.org")`, 5000);
      const mapSaid = await said("https://www.openstreetmap.org");
      // Pendant la touche, la page intégrée a voulu emmener la page de l'auteur ailleurs : elle reste.
      await pause(500);
      const afterKey = await q.value("location.href");
      const stayedAfterKey = afterKey === `${served.base}/${lesson}`;
      if (!stayedAfterKey) return [false, `avant le toucher, demandes vers l'autre site : ${before[0]} (Fetch), ${before[1]} (réseau), page sans cadre ni adresse de l'autre site : ${untouched} ; pendant la touche, la page intégrée a emmené la page de l'auteur vers ${afterKey} ; cadre : ${mapAttributes}`];
      // 5. Au doigt : toucher la façade de la vidéo.
      let fingerFrame = false;
      try {
        await b.send("Emulation.setDeviceMetricsOverride", { width: 400, height: 800, deviceScaleFactor: 2, mobile: true });
        await b.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
        await pause(300);
        const [x, y] = await centre('.holo-embed-load[data-embed^="https://www.youtube-nocookie.com/"]');
        await b.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x, y, id: 1 }] });
        await b.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
        fingerFrame = await q.until(`document.querySelector('iframe[src^="https://www.youtube-nocookie.com/"]')?.title === "Vidéo : Me at the zoo, la première vidéo publiée sur YouTube (2005)"`, 5000);
        await q.until(`(window.__fake ?? []).some((m) => m.ready && m.from === "https://www.youtube-nocookie.com")`, 10000);
      } finally {
        await b.send("Emulation.setTouchEmulationEnabled", { enabled: false });
        await b.send("Emulation.clearDeviceMetricsOverride");
      }
      const videoSaid = await said("https://www.youtube-nocookie.com");
      // 6. Ce qui est arrivé à l'autre site : une demande par toucher, la page seule, et de la page de
      // l'auteur, son site seulement (jamais son adresse). La page de l'auteur n'a pas bougé.
      const reached = asked.map((a) => `${a.resourceType} ${a.url} (Referer : ${a.referer})`);
      const stayed = await q.value(`location.pathname === "/${lesson}"`);
      // 7. La règle des cadres : un cadre vers un site non listé, posé par un script, est refusé par
      // le navigateur, et rien ne part vers lui.
      await q.value(`(() => { const f = document.createElement("iframe"); f.src = "https://pirate.example.org/piege"; document.body.append(f); })()`);
      await q.until(`window.__refused.length > 0`, 3000);
      const refused = await q.value(`window.__refused.join(", ")`);
      await q.value(`document.querySelector('iframe[src^="https://pirate.example.org/"]').remove()`);
      faults.push(...(await audit("après le toucher")));
      // 8. Sans JavaScript : un lien vers la page de l'autre site, avec le titre, dans un nouvel onglet ;
      // pas de bouton, pas de cadre, rien vers l'autre site.
      const beforeLinks = [asked.length, sent.filter((url) => elsewhere.test(url)).length];
      let links = "";
      let linkNames = "";
      let withoutScript = "";
      try {
        await b.send("Emulation.setScriptExecutionDisabled", { value: true });
        await q.open(`/${lesson}`, 300);
        links = await q.value(`[...document.querySelectorAll("a.holo-embed-link")].map((a) => [a.href, a.target, a.rel].join(" | ")).join(" || ")`);
        // Le lecteur d'écran : deux liens, nommés avec le titre ; ils disent qu'ils ouvrent un onglet.
        const { nodes: plain } = (await b.send("Accessibility.getFullAXTree")).result;
        linkNames = plain.filter((n) => !n.ignored && n.role?.value === "link" && /^(Carte|Vidéo) : /.test(n.name?.value ?? "")).map((n) => n.name.value).join(" | ");
        withoutScript = await q.value(`[...document.querySelectorAll(".holo-embed-load")].map((f) => getComputedStyle(f).display).join(",") + " / " + document.querySelectorAll("iframe").length`);
      } finally {
        await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      }
      const afterLinks = asked.length - beforeLinks[0] + sent.filter((url) => elsewhere.test(url)).length - beforeLinks[1];
      // 9. Une page intégrée de la même origine que la page de l'auteur (un auteur qui listerait son
      // propre site) perd le second droit : elle ne garderait pas son enfermement. La leçon est servie
      // à une adresse en HTTPS (Fetch prend chaque fichier à holo serve), et une façade vise une page
      // de cette même adresse.
      const home = "https://zoo.example.org";
      b.on("Fetch.requestPaused", ({ requestId, request }) => {
        if (!request.url.startsWith(`${home}/`)) return b.send("Fetch.fulfillRequest", { requestId, responseCode: 404, body: "" });
        (async () => {
          const reply = await fetch(served.base + request.url.slice(home.length), { headers: { accept: request.headers.Accept ?? "*/*" } });
          const responseHeaders = [...reply.headers].filter(([name]) => ["content-type", "content-security-policy"].includes(name)).map(([name, value]) => ({ name, value }));
          await b.send("Fetch.fulfillRequest", { requestId, responseCode: reply.status, responseHeaders, body: Buffer.from(await reply.arrayBuffer()).toString("base64") });
        })();
      });
      await b.send("Fetch.enable", { patterns: [{ urlPattern: `${home}/*` }] });
      const z = page(b, home);
      await z.open(`/${lesson}`);
      await z.value(`document.querySelector(".holo-embed-load").dataset.embed = location.origin + "/meme-origine"`);
      await z.click(".holo-embed-load");
      const sameOrigin = await z.value(`(document.querySelector('iframe[src="${home}/meme-origine"]')?.getAttribute("sandbox") ?? "(aucun cadre)") + " (page " + location.origin + ")"`);
      const origin = `${served.base}/`;
      const ok = policy === "frame-src https://www.openstreetmap.org https://www.youtube-nocookie.com"
        && before[0] === 0 && before[1] === 0 && untouched && images === "true,true" && shown === "true,true"
        && buttons.join(" | ") === "Carte : le zoo de San Diego, charger depuis www.openstreetmap.org | Vidéo : Me at the zoo, la première vidéo publiée sur YouTube (2005), charger depuis www.youtube-nocookie.com"
        && /^328x185 328x246 \/ 360$/.test(phoneSize)
        && focusedFacade === "Carte : le zoo de San Diego" && mapFrame && inside && mapReady && mapFocused && typed
        && mapAttributes === `allow-scripts allow-same-origin | fullscreen | strict-origin | Carte : le zoo de San Diego | ${map}`
        && mapSaid.referrer === origin && mapSaid.page === "fermée" && mapSaid.popup === "null" && mapSaid.features === "fullscreen" && mapSaid.position === "refusée (1)"
        && sameOrigin === "allow-scripts (page https://zoo.example.org)"
        && fingerFrame && videoSaid.ready === true && videoSaid.page === "fermée"
        && reached.join(" ; ") === `Document ${map} (Referer : ${origin}) ; Document ${video} (Referer : ${origin})`
        && stayed && refused === "frame-src https://pirate.example.org"
        && links === `${map} | _blank | noopener noreferrer || ${video} | _blank | noopener noreferrer`
        && linkNames === "Carte : le zoo de San Diego, ouvrir sur www.openstreetmap.org, dans un nouvel onglet | Vidéo : Me at the zoo, la première vidéo publiée sur YouTube (2005), ouvrir sur www.youtube-nocookie.com, dans un nouvel onglet"
        && withoutScript === "none,none / 0" && afterLinks === 0 && !faults.length;
      return [ok, `frame-src : ${policy} ; avant le toucher, demandes vers l'autre site : ${before[0]} (Fetch), ${before[1]} (réseau), page sans cadre ni adresse de l'autre site : ${untouched}, images de la façade chez holo serve : ${images}, façades montrées : ${shown} ; lecteur d'écran : ${buttons.join(" | ")} ; téléphone : ${phoneSize} ; au clavier, la façade « ${focusedFacade} », puis Entrée : cadre ${mapFrame} (${mapAttributes}), le clavier dedans : ${inside}, le focus reçu par la page intégrée : ${mapFocused}, la touche k reçue : ${typed}, la page de l'auteur reste : ${stayedAfterKey} ; la fausse carte voit : ${JSON.stringify(mapSaid)} ; au doigt, la vidéo : ${fingerFrame} (${JSON.stringify(videoSaid)}) ; arrivé à l'autre site : ${reached.join(" ; ") || "rien"} ; la page de l'auteur reste : ${stayed} ; un cadre non listé : ${refused || "pas refusé"} ; une adresse de la même origine : sandbox « ${sameOrigin} » ; sans JavaScript : ${links} (${linkNames}) ; boutons et cadres : ${withoutScript}, demandes : ${afterLinks} ; axe-core : ${faults.join(", ") || "aucun défaut, avant et après le toucher"}`];
    } finally {
      await b.send("Fetch.disable");
      b.on("Fetch.requestPaused", null);
      b.on("Network.requestWillBeSent", null);
      await b.send("Network.disable");
      served.stop();
    }
  }],
  ["travailler un texte : des majuscules dans la langue, les caractères comptés comme Intl.Segmenter, un aperçu coupé, découper ; sans JavaScript aussi (leçon 126, serve)", async (p, b) => {
    const lesson = "126-travailler-un-texte.holo";
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    const shown = (q, key) => q.value(`document.querySelector('[data-state="${key}"]')?.textContent ?? "(absent)"`);
    const tags = (q) => q.value(`[...document.querySelectorAll('[data-list="tags"] .holo-Text')].map((t) => t.textContent).join(" ")`);
    // Écrire dans un champ comme une main : le choisir, tout sélectionner, taper.
    const write = async (bind, text) => {
      await p.value(`(() => { const f = document.querySelector('[data-bind="${bind}"]'); f.focus(); f.select(); return true; })()`);
      await b.send("Input.insertText", { text });
      await pause(250);
    };
    // 1. La page fabriquée par le serveur montre déjà les textes travaillés, avant le moteur.
    await p.open(`/exemples/lecons/${lesson}`);
    const start = [await shown(p, "code:upper"), await shown(p, "message:length"), await shown(p, "message:max40"), await shown(p, "words"), await tags(p)];
    check("au départ", start.join(" | ") === "AB-12 | 71 | Bonjour à tous, voici mon premier… | 13 | #art #peinture #paris", start.join(" | "));
    // 2. Le visiteur écrit : la valeur montrée suit ; ce qu'il a écrit reste tel quel dans le champ.
    await write("code", "straße");
    const upper = await p.until(`document.querySelector('[data-state="code:upper"]').textContent === "STRASSE"`, 20000);
    check("majuscules", upper && (await p.value(`document.querySelector('[data-bind="code"]').value`)) === "straße", await shown(p, "code:upper"));
    // 3. Les caractères comptés comme Intl.Segmenter les compte, et non comme JavaScript (« .length »).
    const corpus = ["👍🏽🇫🇷", "été", "👨‍👩‍👧 et 🏴󠁧󠁢󠁳󠁣󠁴󠁿", "क्षि नमस्ते", "한국어 한", "مَرْحَبًا", "தமிழ் กำ", "deux\nlignes"];
    const counted = [];
    for (const text of corpus) {
      await write("message", text);
      const expected = await p.value(`[...new Intl.Segmenter("fr", { granularity: "grapheme" }).segment(document.querySelector('[data-bind="message"]').value)].length`);
      await p.until(`document.querySelector('[data-state="message:length"]').textContent === "${expected}"`, 3000);
      const engine = await shown(p, "message:length");
      const js = await p.value(`document.querySelector('[data-bind="message"]').value.length`);
      counted.push(`${JSON.stringify(text)} ${engine}/${expected} (JS ${js})`);
      check(`compté ${JSON.stringify(text)}`, engine === String(expected), `${engine} au lieu de ${expected}`);
    }
    // 4. L'aperçu, au plus 40 caractères « … » compris, ne coupe jamais une lettre en deux.
    await write("message", "é".repeat(50));
    await p.until(`document.querySelector('[data-state="message:max40"]').textContent.endsWith("…")`, 3000);
    const cut = await p.value(`(() => { const t = document.querySelector('[data-state="message:max40"]').textContent; const s = [...new Intl.Segmenter("fr", { granularity: "grapheme" }).segment(t)].map((x) => x.segment); return [s.length, s.slice(0, -1).every((x) => x === "é"), s.at(-1)]; })()`);
    check("aperçu coupé entre deux lettres", cut[0] === 40 && cut[1] && cut[2] === "…", JSON.stringify(cut));
    // 5. Publier : la ligne montre l'aperçu de l'élément, coupé à la fin d'un mot.
    await write("message", "Un message assez long pour être coupé à la fin d'un mot, voici la suite.");
    await p.click('[data-name="Publish"]');
    const posted = await p.until(`document.querySelector('[data-list="posts"] .holo-P')?.textContent === "Un message assez long pour être coupé à…"`, 5000);
    check("une ligne publiée, coupée", posted, await p.value(`document.querySelector('[data-list="posts"]')?.textContent`));
    // 6. Découper : les virgules du chinois aussi ; les morceaux vides oubliés ; le compte suit.
    await write("keywords", "北京，上海、 广州,,");
    const split = await p.until(`document.querySelector('[data-state="tags"]').textContent === "3"`, 3000);
    check("découpé", split && (await tags(p)) === "#北京 #上海 #广州", `${await shown(p, "tags")} : ${await tags(p)}`);
    // 7. Rien d'annoncé à chaque lettre : aucune zone vivante autour des valeurs montrées.
    const live = await p.value(`[...document.querySelectorAll('[data-state*=":"]')].filter((e) => e.closest("[aria-live], [role=status], [role=alert], [role=log]")).length`);
    check("pas de zone vivante", live === 0, `${live} valeur(s) dans une zone vivante`);
    check("aucune erreur", b.errors.length === 0, b.errors.join(" | "));
    // L'audit axe-core, quand sa copie locale est là (GitHub l'installe).
    let audit = "axe-core absent ici";
    const axeFile = join(engine, "node_modules", "axe-core", "axe.min.js");
    if (existsSync(axeFile)) {
      await p.value(readFileSync(axeFile, "utf8") + "\n;0");
      audit = await p.value(`axe.run(document, { resultTypes: ["violations"] }).then((r) => r.violations.map((v) => v.id).join(", "))`);
      check("axe-core", audit === "", audit);
      audit = audit || "zéro défaut";
    }
    // 8. Sans JavaScript, avec holo serve : le champ part au serveur avec « Publier », et la page
    // revient avec les textes travaillés par le même moteur.
    const served = await startHoloServe([lesson]);
    const q = page(b, served.base);
    let withoutScript = [];
    try {
      await b.send("Emulation.setScriptExecutionDisabled", { value: true });
      await q.open(`/${lesson}`, 300);
      await q.value(`(() => { document.querySelector('[data-bind="code"]').value = "istanbul ılık"; document.querySelector('[data-bind="keywords"]').value = "Un, DEUX ,trois"; return true; })()`);
      await q.click('[data-name="Publish"]');
      await q.until(`document.readyState === "complete" && document.querySelector('[data-state="code:upper"]')?.textContent === "ISTANBUL ILIK"`, 5000);
      withoutScript = [await shown(q, "code:upper"), await shown(q, "tags"), await tags(q), await q.value(`document.querySelector('[data-list="posts"] .holo-P')?.textContent ?? ""`)];
    } finally {
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      served.stop();
    }
    check("sans JavaScript", withoutScript.join(" | ") === "ISTANBUL ILIK | 3 | #un #deux #trois | Bonjour à tous, voici mon premier…", withoutScript.join(" | "));
    return [faults.length === 0, faults.length ? faults.join("\n      ") : `au départ : ${start.join(" | ")} ; « straße » → STRASSE ; compté comme Intl.Segmenter : ${counted.join(", ")} ; aperçu de « é » × 50 : ${cut[0]} lettres, jamais coupées ; publié, coupé à la fin d'un mot ; « 北京，上海、 广州,, » → 3 étiquettes ; aucune zone vivante ; axe-core : ${audit} ; sans JavaScript : ${withoutScript.join(" | ")}`];
  }],
  ["un module venu d'ailleurs : son empreinte vérifiée par le navigateur avant chaque lancement, même sur une adresse du réseau local ; holo serve le télécharge une fois, jamais le navigateur ; ce qui demande plus que sa mémoire est refusé ; sa licence dite aux visiteurs (leçon 141, serve)", async (p, b) => {
    // ADR-118. Un faux « autre site » sur ce PC, qui note chaque demande reçue : holo serve ne
    // l'atteint que par l'interrupteur des essais (HOLO_TEST_ONLY_INSECURE_SITE=modules.test:<port>).
    // La page est ouverte par l'adresse de ce PC sur le réseau local, comme un téléphone du même
    // Wi-Fi : un contexte non sûr, où crypto.subtle n'existe pas.
    const binary = ["holo", "holo.exe"].map((name) => join(engine, "target", "release", name)).find(existsSync);
    if (!binary) return [false, "holo n'est pas construit : cargo build --release --bin holo"];
    const lesson = "141-un-module-venu-d-ailleurs.holo";
    const lessons = join(repo, "exemples", "lecons");
    const primes = readFileSync(join(lessons, "141-premiers.wasm"));
    const sum = readFileSync(join(lessons, "69-compter.wasm"));
    const { createHash } = await import("node:crypto");
    const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
    // Un module qui demande aussi une fonction à la page, « env.fetch » : le réseau, s'il l'avait.
    const leb = (n) => { const out = []; do { let byte = n & 0x7f; n >>>= 7; if (n) byte |= 0x80; out.push(byte); } while (n); return out; };
    const named = (text) => [...leb(text.length), ...Buffer.from(text)];
    const items = (entries) => [...leb(entries.length), ...entries.flat()];
    const section = (id, content) => [id, ...leb(content.length), ...content];
    const pirate = Buffer.from([0, 0x61, 0x73, 0x6d, 1, 0, 0, 0,
      ...section(1, items([[0x60, 1, 0x7f, 1, 0x7f]])),
      ...section(2, items([[...named("env"), ...named("memory"), 0x02, 0x00, 0x01], [...named("env"), ...named("fetch"), 0x00, 0x00]])),
      ...section(3, items([[0x00]])),
      ...section(7, items([[...named("run"), 0x00, 0x01]])),
      ...section(10, items([[0x04, 0x00, 0x20, 0x00, 0x0b]]))]);
    const seen = [];
    const other = createServer((req, res) => {
      seen.push({ url: req.url, agent: String(req.headers["user-agent"]), cookie: "cookie" in req.headers, referer: "referer" in req.headers });
      const body = { "/premiers-1.0.wasm": primes, "/premiers-1.1.wasm": sum }[req.url];
      res.writeHead(body ? 200 : 404, { "content-type": "application/wasm" });
      res.end(body ?? "");
    });
    await new Promise((ok) => other.listen(0, "127.0.0.1", ok));
    const otherPort = other.address().port;
    const folder = mkdtempSync(join(tmpdir(), "holo-serve-"));
    writeFileSync(join(folder, lesson), readFileSync(join(lessons, lesson)));
    writeFileSync(join(folder, "141-premiers.wasm"), primes);
    const pageWith = (source, extra) => [
      `module "${source}"`,
      `Page(title: "Essai", state: State(n: 100, total: 0, done: 0, failed: 0),`,
      `  modules: [ Module(name: Calcul, source: "${source}", input: n, output: total${extra}) ],`,
      '  children: [ H1("Essai"), Button(name: Go, text: "Lancer"), P("Résultat : {total}"), If(done, is: 1, children: [ P("Arrivé.") ]), If(failed, is: 1, children: [ P("Pas lancé.") ]) ],',
      "  rules: [ On(Go.tap, effect: Calcul.run), On(Calcul.done, effect: done.set(1)), On(Calcul.failed, effect: failed.set(1)) ],",
      ")",
    ].join("\n");
    const pinned = (bytes) => `, sha256: "${sha(bytes)}", license: "MIT"`;
    // Sa copie manque : holo serve ira la chercher.
    writeFileSync(join(folder, "ailleurs.holo"), pageWith("premiers.wasm", `, from: "https://modules.test/premiers-1.0.wasm"${pinned(primes)}`));
    // L'autre site a changé son fichier en silence (une autre version, sous l'adresse écrite).
    writeFileSync(join(folder, "en-silence.holo"), pageWith("silence.wasm", `, from: "https://modules.test/premiers-1.1.wasm"${pinned(primes)}`));
    // La copie à côté de la page a été remplacée : la somme de la leçon 69, au lieu des nombres premiers.
    writeFileSync(join(folder, "change.holo"), pageWith("change.wasm", pinned(primes)));
    writeFileSync(join(folder, "change.wasm"), sum);
    // Un module qui demande le réseau, avec sa bonne empreinte.
    writeFileSync(join(folder, "pirate.holo"), pageWith("pirate.wasm", pinned(pirate)));
    writeFileSync(join(folder, "pirate.wasm"), pirate);
    const port = 26000 + Math.floor(Math.random() * 2000);
    const env = { ...process.env, HOLO_TEST_ONLY_INSECURE_SITE: `modules.test:${otherPort}` };
    const server = spawn(binary, ["serve", folder, String(port)], { cwd: engine, env, stdio: ["ignore", "pipe", "pipe"] });
    let journal = "";
    server.stdout.on("data", (d) => { journal += d; });
    server.stderr.on("data", (d) => { journal += d; });
    for (let i = 0; i < 100 && !journal.includes(`localhost:${port}`); i++) await pause(100);
    // L'adresse de ce PC sur le réseau local (holo serve écoute sur 0.0.0.0), sinon 127.0.0.1.
    const { networkInterfaces } = await import("node:os");
    const lan = Object.values(networkInterfaces()).flat().find((a) => a?.family === "IPv4" && !a.internal)?.address;
    const base = `http://${lan ?? "127.0.0.1"}:${port}`;
    const q = page(b, base);
    const sent = [];
    const caught = [];
    const shows = (words) => `document.getElementById("page").innerText.includes(${JSON.stringify(words)})`;
    const last = () => q.value("JSON.stringify((window.__holoModules ?? []).at(-1) ?? null)").then(JSON.parse);
    const notice = () => q.value(`[...document.querySelectorAll(".holo-modules p")].map((p) => p.innerText).join(" | ")`);
    const axeSource = readFileSync(join(engine, "node_modules", "axe-core", "axe.min.js"), "utf8");
    try {
      await b.send("Network.enable");
      b.on("Network.requestWillBeSent", ({ request }) => sent.push(request.url));
      // Si le navigateur allait chez l'autre site, il serait arrêté et compté ici.
      b.on("Fetch.requestPaused", ({ requestId, request }) => {
        caught.push(request.url);
        b.send("Fetch.failRequest", { requestId, errorReason: "BlockedByClient" });
      });
      await b.send("Fetch.enable", { patterns: ["*://modules.test/*", `*://127.0.0.1:${otherPort}/*`].map((urlPattern) => ({ urlPattern })) });
      // 1. La leçon, sur l'adresse du réseau local : un contexte non sûr, sans crypto.subtle.
      await q.open(`/${lesson}`);
      const context = await q.value("`${isSecureContext} ${typeof crypto.subtle}`");
      const told = await notice();
      const landmark = (await b.send("Accessibility.getFullAXTree")).result?.nodes?.some((n) => n.role?.value === "complementary" && n.name?.value === "Modules de cette page");
      await q.click('[data-name="Compter"]');
      const counted = await q.until(shows("Il y en a 168."), 10000);
      const lessonRun = await last();
      // Au clavier aussi : le bouton, puis Entrée.
      await q.value(`document.querySelector('[data-name="Compter"]').focus()`);
      await q.key("Enter", "Enter", 13, "\r");
      const byKey = await q.until(`(window.__holoModules ?? []).length === 2 && window.__holoModules[1].ok`, 10000);
      await q.value(axeSource + "\n;0");
      const faults = await q.value('window.axe.run(document, { resultTypes: ["violations"] }).then((r) => r.violations.map((v) => v.id))');
      // Sur un téléphone de 360 px : rien ne déborde, la mention se lit.
      await b.send("Emulation.setDeviceMetricsOverride", { width: 360, height: 740, deviceScaleFactor: 2, mobile: true });
      await q.open(`/${lesson}`, 600);
      const phoneFits = await q.value("document.documentElement.scrollWidth <= innerWidth && document.querySelector('.holo-modules').getBoundingClientRect().width <= innerWidth");
      await b.send("Emulation.clearDeviceMetricsOverride");
      // 2. Sa copie manque : holo serve la télécharge une fois, la range, la sert ; rien vers l'autre site depuis le navigateur.
      const before = existsSync(join(folder, "premiers.wasm"));
      await q.open("/ailleurs.holo");
      await q.click('[data-name="Go"]');
      const arrived = await q.until(`${shows("Résultat : 25")} && ${shows("Arrivé.")}`, 20000);
      const kept = existsSync(join(folder, "premiers.wasm")) && sha(readFileSync(join(folder, "premiers.wasm"))) === sha(primes);
      await q.click('[data-name="Go"]');
      await q.open("/ailleurs.holo");
      await q.click('[data-name="Go"]');
      const again = await q.until(shows("Arrivé."), 10000);
      const downloads = seen.filter((r) => r.url === "/premiers-1.0.wasm").length;
      // 3. L'autre site a changé son fichier : holo serve ne le range pas ; la page reçoit « failed ».
      await q.open("/en-silence.holo");
      await q.click('[data-name="Go"]');
      const silent = await q.until(shows("Pas lancé."), 20000);
      const silentKept = existsSync(join(folder, "silence.wasm"));
      // 4. La copie a changé : le navigateur refuse le fichier ; le module ne tourne pas, et le bas de la page dit pourquoi.
      await q.open("/change.holo");
      await q.click('[data-name="Go"]');
      const refusedChange = await q.until(shows("Pas lancé."), 10000);
      const changeRun = await last();
      const changeTold = await q.value(`(() => { const s = document.querySelector('.holo-modules [data-why="changed"]'); return s && !s.hidden && s.checkVisibility() ? s.closest("p").innerText : ""; })()`);
      const neverRan = !(await q.value(shows("5050")));
      // 5. Le module qui demande le réseau : refusé avant de tourner.
      await q.open("/pirate.holo");
      await q.click('[data-name="Go"]');
      const refusedPirate = await q.until(shows("Pas lancé."), 10000);
      const pirateRun = await last();
      const pirateTold = await q.value(`(() => { const s = document.querySelector('.holo-modules [data-why="box"]'); return s && !s.hidden ? s.closest("p").innerText : ""; })()`);
      // 6. Sans JavaScript : la licence et le site se lisent quand même.
      let withoutScript = "";
      try {
        await b.send("Emulation.setScriptExecutionDisabled", { value: true });
        await q.open(`/${lesson}`, 300);
        withoutScript = await notice();
      } finally {
        await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      }
      const elsewhere = sent.filter((url) => !url.startsWith(base) && !/^(data|blob|about|chrome-extension):/.test(url));
      const fromHoloServe = seen.every((r) => r.agent.startsWith("HoloCode/") && !r.cookie && !r.referer);
      const logged = journal.includes(`Module          : /premiers.wasm : téléchargé une fois depuis https://modules.test/premiers-1.0.wasm, empreinte vérifiée, 205 octets, licence : MIT`)
        && journal.includes("Module          : /silence.wasm : l'empreinte du fichier téléchargé ne correspond pas à celle que la page écrit");
      const french = "Module «\u202F141-premiers.wasm\u202F», venu de raw.githubusercontent.com — licence\u00A0: Tous droits réservés";
      const ok = (lan ? context === "false undefined" : true) && told === french && landmark && counted && lessonRun?.ok === true && byKey && !faults.length && phoneFits
        && !before && arrived && kept && again && downloads === 1
        && silent && !silentKept && seen.filter((r) => r.url === "/premiers-1.1.wasm").length === 1
        && refusedChange && changeRun?.why === "changed" && changeRun?.reason.startsWith("l'empreinte ne correspond pas") && changeTold.endsWith("refusé\u00A0: ce fichier n'est pas celui que l'auteur a vérifié (son empreinte diffère)") && neverRan
        && refusedPirate && pirateRun?.why === "box" && pirateRun?.reason.includes("« env.fetch » (une fonction)") && pirateTold.endsWith("refusé\u00A0: il demande plus que ce que la boîte lui donne")
        && withoutScript === french && elsewhere.length === 0 && caught.length === 0 && fromHoloServe && logged;
      return [ok, `page ouverte sur ${base} : isSecureContext, crypto.subtle : ${context}${lan ? "" : " (pas d'adresse du réseau local : 127.0.0.1)"} ; en bas de la page : « ${told} », repère pour le lecteur d'écran : ${landmark} ; le module compte : ${counted} (${JSON.stringify(lessonRun)}), au clavier : ${byKey} ; téléphone de 360 px sans débord : ${phoneFits} ; axe-core : ${faults.join(", ") || "aucun défaut"} ; copie absente au départ : ${!before}, téléchargée par holo serve et lancée : ${arrived}, rangée avec la bonne empreinte : ${kept}, encore après deux lancements et un rechargement : ${again}, demandes à l'autre site pour elle : ${downloads} ; changée en silence chez l'autre site : refusée ${silent}, rangée ${silentKept} ; copie changée : refusée ${refusedChange} (${changeRun?.reason}), dit au visiteur : « ${changeTold} », jamais lancée : ${neverRan} ; le module qui demande le réseau : refusé ${refusedPirate} (${pirateRun?.reason}), dit : « ${pirateTold} » ; sans JavaScript : « ${withoutScript} » ; le navigateur ailleurs : ${elsewhere.join(", ") || "jamais"} (arrêtées : ${caught.length}) ; l'autre site n'a vu que holo serve : ${fromHoloServe} (${seen.map((r) => r.url).join(", ")}) ; le journal le dit : ${logged}${b.errors.length ? ` (${b.errors.join(" | ")})` : ""}`];
    } finally {
      await b.send("Fetch.disable");
      b.on("Fetch.requestPaused", null);
      b.on("Network.requestWillBeSent", null);
      await b.send("Network.disable");
      server.kill();
      other.close();
      try { rmSync(folder, { recursive: true, force: true, maxRetries: 20, retryDelay: 100 }); } catch { /* le dossier temporaire restera */ }
    }
  }],
  ["les touches du clavier, et les lettres qu'on coupe", async (p) => {
    await p.open("/exemples/lecons/77-toutes-les-touches.holo");
    if (!(await p.until(`document.getElementById("shortcuts")`))) return [false, "le moteur n'est pas arrivé"];
    const count = () => p.value(`document.querySelector(".holo-s-grand").textContent.trim()`);
    await p.key("p", "KeyP", 80, "p");
    await p.key("%", "Digit5", 53, "%"); // un clavier français : la touche du 5 sans Maj
    await p.key("Enter", "Enter", 13, "\r");
    const before = await count();
    await p.value(`document.getElementById("shortcuts").click()`);
    await p.key("p", "KeyP", 80, "p");
    const cut = await count();
    await p.key("Escape", "Escape", 27);
    const reset = await count();
    await p.value(`document.getElementById("shortcuts").click()`);
    return [before === "16" && cut === "16" && reset === "0", `P, 5, Entrée → ${before} ; lettres coupées → ${cut} ; Échap → ${reset}`];
  }],
  ["Échap ferme une fenêtre, même quand la page écoute Échap", async (p) => {
    await p.open("/exemples/.essais-navigateur/fenetre-et-echap.holo");
    await p.click('[data-name="Ouvrir"]');
    if (!(await p.until(`document.querySelector("dialog").open`))) return [false, "la fenêtre ne s'ouvre pas"];
    await p.key("Escape", "Escape", 27);
    const closed = await p.until(`!document.querySelector("dialog").open`, 3000);
    await p.key("Escape", "Escape", 27);
    const counted = await p.until(`document.getElementById("page").innerText.includes("Échap hors de la fenêtre : 1")`, 5000);
    return [closed && counted, `fenêtre fermée : ${closed} ; règle d'Échap ailleurs : ${counted}`];
  }],
  ["une liste qui change : ajouter, retirer", async (p) => {
    await p.open("/exemples/lecons/68-liste-qui-change.holo");
    const lines = () => p.value(`document.querySelectorAll(".holo-line").length`);
    const start = await lines();
    await p.type("#page input", "Laver le pinceau");
    await p.click('[data-name="Ajouter"]');
    const added = await p.until(`document.querySelectorAll(".holo-line").length === ${start + 1}`);
    await p.click('.holo-line:last-child [data-name^="Fait"]');
    const removed = await p.until(`document.querySelectorAll(".holo-line").length === ${start}`);
    return [added && removed, `lignes : ${start} → ajout ${added ? "fait" : "raté"} → retrait ${removed ? "fait" : "raté"}`];
  }],
  ["une liste à champs reçue du serveur", async (p) => {
    await p.open("/exemples/lecons/71-liste-a-champs.holo");
    const ok = await p.until(`document.querySelectorAll(".holo-line").length > 1 && !document.getElementById("page").innerText.includes("Chargement…")`);
    return [ok, `${await p.value(`document.querySelectorAll(".holo-line").length`)} lignes`];
  }],
  ["des données de 64 Ko au plus : prises en dessous, refusées au-dessus", async (p) => {
    // Le fichier trop gros est fabriqué ici (plus de 64 Ko), puis effacé.
    const big = join(repo, "exemples", ".essais-navigateur", "donnees-trop-grosses.json");
    const items = Array.from({ length: 1400 }, (_, i) => ({ title: `Article numéro ${i} avec un titre assez long`, price: i }));
    writeFileSync(big, JSON.stringify({ articles: items }));
    try {
      await p.open("/exemples/.essais-navigateur/donnees.holo");
      const small = await p.until(`document.getElementById("page").innerText.includes("3 article(s)")`);
      await p.open("/exemples/.essais-navigateur/donnees-trop-grosses.holo", 3000);
      const refused = await p.until(`document.getElementById("page").innerText.includes("1 article(s)")`, 5000)
        && !(await p.value(`document.getElementById("page").innerText.includes("Article numéro")`));
      return [small && refused, `petites données prises : ${small} ; ${Math.round(JSON.stringify({ articles: items }).length / 1000)} Ko refusés : ${refused}`];
    } finally {
      rmSync(big, { force: true });
    }
  }],
  ["chercher, filtrer, trier et montrer plus (Filter)", async (p) => {
    await p.open("/exemples/lecons/82-chercher-filtrer-trier.holo");
    const titles = () => p.value(`[...document.querySelectorAll(".holo-line")].map((l) => l.innerText.split(String.fromCharCode(10))[0].trim()).join(" | ")`);
    const start = await titles();
    await p.type("#page input[type=text], #page input:not([type])", "RI");
    const searched = await p.until(`document.querySelectorAll(".holo-line").length === 2`);
    const afterSearch = await titles();
    await p.value(`(() => { const i = document.querySelector("#page input:not([type=radio])"); i.value = ""; i.dispatchEvent(new Event("input", { bubbles: true })); })()`);
    await p.click('input[type=radio][value="huile"]');
    const filtered = await p.until(`document.querySelectorAll(".holo-line").length === 3`);
    await p.click('[data-name="Toutes"]');
    const before = await p.value(`document.getElementById("page").innerText.includes("4 œuvre(s) sur 6")`);
    await p.click('[data-name="Plus"]');
    const more = await p.until(`document.querySelectorAll(".holo-line").length === 6 && document.getElementById("page").innerText.includes("6 œuvre(s) sur 6") && !document.querySelector('[data-name="Plus"]')?.offsetParent`);
    await p.type("#page input:not([type=radio])", "zzz");
    const empty = await p.until(`document.querySelector(".holo-empty")?.textContent === "Aucune œuvre ne correspond."`);
    const ok = start === "Le phare | Orage | Élan du matin | La rivière" && searched && afterSearch === "La rivière | Rizières" && filtered && before && more && empty;
    return [ok, `départ : ${start} ; « RI » : ${afterSearch} ; huile : ${filtered} ; « 4 sur 6 » : ${before} ; montrer plus, puis caché : ${more} ; « zzz » vide : ${empty}`];
  }],
  ["comparer des textes (If et When)", async (p) => {
    await p.open("/exemples/lecons/83-comparer-des-textes.holo");
    const has = (words) => `document.getElementById("page").innerText.includes(${JSON.stringify(words)})`;
    // Écrire dans un champ vide : on l'efface d'abord, comme le ferait le visiteur.
    const write = async (bind, text) => {
      await p.value(`(() => { const i = document.querySelector('input[data-bind="${bind}"]'); i.value = ""; i.dispatchEvent(new Event("input", { bubbles: true })); })()`);
      await p.type(`input[data-bind="${bind}"]`, text);
    };
    const start = (await p.value(has("Taille M."))) && !(await p.value(has("Le L est grand")));
    await p.click('input[type=radio][value="L"]');
    const large = await p.until(`${has("Le L est grand")} && !${has("Taille M.")}`);
    await write("answer", "paris");
    const lower = await p.until(`!${has("Bravo !")} && ${has("Bonnes réponses : 0")}`);
    await write("answer", "Paris");
    const bravo = await p.until(`${has("Bravo !")} && ${has("Bonnes réponses : 1")}`);
    await write("email", "ada@exemple.fr");
    await write("again", "ada@exemple");
    const differ = await p.until(has("Les deux e-mails sont différents."));
    await write("again", "ada@exemple.fr");
    const same = await p.until(`${has("Les deux sont pareils.")} && !${has("différents")}`);
    const ok = start && large && lower && bravo && differ && same;
    return [ok, `départ « Taille M. » : ${start} ; L : ${large} ; « paris » refusé : ${lower} ; « Paris » → Bravo et 1 : ${bravo} ; e-mails différents : ${differ} ; pareils : ${same}`];
  }],
  ["des données disent « arrivées » ou « échec », et se relisent (Data)", async (p, b) => {
    await p.open("/exemples/.essais-navigateur/donnees-nommees.holo");
    const has = (words) => `document.getElementById("page").innerText.includes(${JSON.stringify(words)})`;
    // La page arrive du serveur avec ses données (une lecture), puis le navigateur relit (une
    // de plus) ; sans `holo` compilé, seule la seconde a lieu. On part du compte, quel qu'il soit.
    const count = () => p.value(`Number((document.getElementById("page").innerText.match(/lectures (\\d+)/) ?? [])[1] ?? -1)`);
    const arrived = await p.until(`window.__holoStarted && ${has("échecs 0")} && document.querySelectorAll(".holo-line").length === 3 && !${has("Chargement")}`);
    await pause(1500);
    const reads = await count();
    // Les réponses du serveur, imitées par Chrome lui-même (son domaine Fetch) : un fichier
    // absent, illisible, trop gros, ou qui ne vient jamais.
    let mode = "";
    const json = [{ name: "content-type", value: "application/json" }];
    const body = (text) => Buffer.from(text).toString("base64");
    b.on("Fetch.requestPaused", ({ requestId }) => {
      if (mode === "absent") b.send("Fetch.fulfillRequest", { requestId, responseCode: 404, body: body("absent") });
      else if (mode === "illisible") b.send("Fetch.fulfillRequest", { requestId, responseCode: 200, responseHeaders: json, body: body("{ pas du json") });
      else if (mode === "trop gros") b.send("Fetch.fulfillRequest", { requestId, responseCode: 200, responseHeaders: json, body: body(JSON.stringify({ articles: [], note: "x".repeat(70000) })) });
      else if (mode === "trop lent") { /* aucune réponse : la page abandonne après 10 secondes */ }
      else b.send("Fetch.continueRequest", { requestId });
    });
    await b.send("Fetch.enable", { patterns: [{ urlPattern: "*donnees-petites.json*" }] });
    const seen = [];
    // « absent » deux fois de suite : la seconde demande, trop proche, attend son tour (une
    // lecture par seconde au plus) ; elle n'est pas perdue. « trop lent » : une seconde demande
    // pendant la lecture est sans objet, la lecture en cours répondra.
    for (const [m, fails, twice] of [["absent", 2, "tout de suite"], ["illisible", 3], ["trop gros", 4], ["trop lent", 5, "pendant la lecture"]]) {
      mode = m;
      await pause(1100);
      await p.click('[data-name="Again"]');
      const started = Date.now();
      if (twice) {
        await pause(twice === "tout de suite" ? 100 : 1200);
        await p.click('[data-name="Again"]');
      }
      const failed = await p.until(`${has(`échecs ${fails}`)} && !${has("Chargement")}`, m === "trop lent" ? 20000 : 6000);
      seen.push(`${m}${twice ? `, deux demandes (${twice})` : ""} → échecs ${fails} : ${failed}${m === "trop lent" ? ` (après ${((Date.now() - started) / 1000).toFixed(1)} s)` : ""}`);
    }
    await pause(1500);
    const noExtra = await p.value(`!${has("échecs 6")}`);
    seen.push(`pas de lecture en trop : ${noExtra}`);
    mode = "";
    await pause(1100);
    await p.click('[data-name="Again"]');
    const back = await p.until(has(`lectures ${reads + 1} ; échecs 5`));
    await b.send("Fetch.disable");
    b.on("Fetch.requestPaused", null);
    // La leçon 84 : les nouvelles arrivent, « Chargement… » disparaît.
    await p.open("/exemples/lecons/84-donnees-arrivees-ou-pas.holo");
    const lesson = await p.until(`${has("42 visiteurs aujourd'hui.")} && !${has("Chargement")}`);
    const ok = arrived && reads >= 1 && seen.every((s) => s.endsWith("true") || s.includes("true (après")) && back && lesson;
    return [ok, `arrivées : ${arrived} (lectures : ${reads}) ; ${seen.join(" ; ")} ; relues ensuite : ${back} ; leçon 84 : ${lesson}`];
  }],
  ["la page du serveur arrive avec ses données, et le navigateur part du même état", async (p, b) => {
    // Le fichier de données ne répond jamais au navigateur (Chrome le retient) : ce que la page
    // montre vient donc du serveur, puis de la réception rejouée par le navigateur (ADR-064).
    b.on("Fetch.requestPaused", () => { /* aucune réponse */ });
    await b.send("Fetch.enable", { patterns: [{ urlPattern: "*84-boutique.json*" }] });
    try {
      await p.open("/exemples/lecons/84-donnees-arrivees-ou-pas.holo");
      const has = (words) => `document.getElementById("page").innerText.includes(${JSON.stringify(words)})`;
      const fromServer = await p.value(`fetch(location.pathname, { headers: { accept: "text/html" } }).then((r) => r.text())`);
      const served = fromServer.includes(`<span data-state="visitors">42</span> visiteurs`) && fromServer.includes(`data-if="loading|is=1" hidden`) && fromServer.includes("data-received=");
      const started = await p.until("window.__holoStarted === true");
      await pause(500);
      const kept = await p.value(`${has("42 visiteurs aujourd'hui.")} && !${has("Chargement")}`);
      return [served && started && kept, `page du serveur avec les données : ${served} ; moteur arrivé : ${started} ; données gardées sans les relire : ${kept}`];
    } finally {
      await b.send("Fetch.disable");
      b.on("Fetch.requestPaused", null);
    }
  }],
  ["deux répétitions d'une liste gardent chacune leur modèle", async (p) => {
    await p.open("/exemples/.essais-navigateur/deux-repetitions.holo");
    const read = (rank) => p.value(`[...document.querySelectorAll('[data-list="tasks"]')[${rank}].querySelectorAll(".holo-line")].map((l) => l.innerText.trim()).join(" | ")`);
    await p.type('input[data-bind="draft"]', "Lait");
    await p.click('[data-name="Add"]');
    await p.until(`document.querySelectorAll('[data-list="tasks"]')[1]?.querySelectorAll(".holo-line").length === 2`);
    const [a, b] = [await read(0), await read(1)];
    const ok = a === "A: Pain | A: Lait" && b === "B: Pain | B: Lait";
    return [ok, `première : ${a} ; seconde : ${b}`];
  }],
  ["le clavier reste sur la ligne refaite (Repeat key)", async (p) => {
    await p.open("/exemples/lecons/85-une-cle-pour-chaque-element.holo");
    // Le clavier sur « Fait » de « Appeler Ada » (2e ligne), puis Entrée : la tâche descend en bas.
    await p.until(`document.querySelectorAll(".holo-line").length === 3`);
    await p.value(`document.querySelectorAll(".holo-line")[1].querySelector('[data-name="Done"]').focus()`);
    await p.key("Enter", "Enter", 13, "\r");
    const moved = await p.until(`document.querySelectorAll(".holo-line")[2]?.innerText.includes("Appeler Ada")`);
    await pause(300);
    const where = await p.value(`(() => { const a = document.activeElement; const line = a?.closest(".holo-line"); return line ? line.dataset.key + " " + a.dataset.name : (a?.tagName ?? "rien"); })()`);
    // « Retirer » sur cette ligne, au clavier : elle part, le clavier va à la ligne qui prend sa place (la dernière).
    await p.value(`document.activeElement.closest(".holo-line").querySelector('[data-name="Drop"]').focus()`);
    await p.key("Enter", "Enter", 13, "\r");
    const removed = await p.until(`document.querySelectorAll(".holo-line").length === 2`);
    await pause(300);
    const after = await p.value(`(() => { const a = document.activeElement; return a?.closest(".holo-line") ? a.closest(".holo-line").innerText.split(String.fromCharCode(10))[0] + " " + a.dataset.name : (a?.tagName ?? "rien"); })()`);
    const ok = moved && where === "k:t2-0 Undo" && removed && after.endsWith(" Drop");
    return [ok, `la tâche descend : ${moved} ; le clavier est sur : ${where} ; retirée : ${removed} ; puis sur : ${after}`];
  }],
  ["des nombres à virgule, exacts (12,50 × 4 = 50,00)", async (p) => {
    await p.open("/exemples/lecons/86-nombres-a-virgule.holo");
    const has = (words) => `document.getElementById("page").innerText.includes(${JSON.stringify(words)})`;
    // Comme un visiteur : le clavier sur le champ, on l'efface, on écrit (la page ne récrit jamais
    // le champ où l'on écrit).
    const write = async (bind, text) => {
      await p.value(`(() => { const i = document.querySelector('input[data-bind="${bind}"]'); i.focus(); i.value = ""; i.dispatchEvent(new Event("input", { bubbles: true })); })()`);
      await p.type(`input[data-bind="${bind}"]`, text);
    };
    const start = (await p.value(has("Prix : 12,50 €"))) && (await p.value(`document.querySelector('input[data-bind="price"]').value === "12.50"`));
    await write("qty", "4");
    await p.click('[data-name="Compute"]');
    const fifty = await p.until(`${has("Total : 50,00 €")} && ${has("Livraison offerte.")}`);
    await p.click('[data-name="Tip"]');
    const tip = await p.until(has("Total : 55,00 €"));
    await write("price", "9.99");
    const price = await p.until(has("Prix : 9,99 €"));
    const ok = start && fifty && tip && price;
    return [ok, `départ « 12,50 » : ${start} ; ×4 = 50,00 et livraison offerte : ${fifty} ; +10 % = 55,00 : ${tip} ; prix 9,99 : ${price}`];
  }],
  ["des nombres négatifs : sous zéro, le signe moins de la langue, un champ dont le clavier l'a (leçon 125)", async (p, b) => {
    await p.open("/exemples/lecons/125-des-nombres-negatifs.holo");
    const has = (words) => `document.getElementById("page").innerText.includes(${JSON.stringify(words)})`;
    // La page fabriquée d'avance, sans le moteur, montre déjà le départ sous zéro.
    const start = (await p.value(has("Au sommet : -2 °C"))) && (await p.value(has("Il gèle.")));
    // Au toucher, le moteur arrive et calcule sous zéro : −2 − 5 = −7 ; puis +5 +5 = 3.
    await p.click('[data-name="Colder"]');
    const colder = await p.until(has("Au sommet : -7 °C"));
    await p.click('[data-name="Warmer"]');
    await p.click('[data-name="Warmer"]');
    const warmer = await p.until(`${has("Au sommet : 3 °C")} && ${has("Il ne gèle pas.")}`);
    // Le champ : un nombre, sans inputmode (le clavier du téléphone garde le signe moins), de −50 à 50.
    const field = await p.value(`(() => { const i = document.querySelector('input[data-bind="temperature"]'); return [i.type, i.inputMode || "(aucun)", i.min, i.max, i.value].join(" "); })()`);
    // On écrit −40, comme au clavier : le grand froid. Plus bas que le min, la page garde −50.
    const write = async (text) => {
      await p.value(`(() => { const i = document.querySelector('input[data-bind="temperature"]'); i.focus(); i.value = ""; i.dispatchEvent(new Event("input", { bubbles: true })); })()`);
      await p.type('input[data-bind="temperature"]', text);
    };
    await write("-40");
    const typed = await p.until(`${has("Au sommet : -40 °C")} && ${has("Grand froid")}`);
    await write("-90");
    const floor = await p.until(has("Au sommet : -50 °C"));
    // Le lecteur d'écran : un champ de nombre (spinbutton), nommé par son étiquette.
    const { nodes } = (await b.send("Accessibility.getFullAXTree")).result;
    const spin = nodes.some((n) => !n.ignored && n.role?.value === "spinbutton" && n.name?.value === "Écrire la température");
    // Une page suédoise : le signe moins de sa langue, « − » (U+2212), au départ comme après un toucher.
    await p.open("/exemples/.essais-navigateur/nombres-negatifs-suedois.holo");
    const swedishStart = await p.value(has("Temperatur: −2 °C"));
    await p.click('[data-name="Kallare"]');
    const swedish = await p.until(has("Temperatur: −7 °C"));
    const seen = await p.value(`document.querySelector("main p, .holo-Page p").innerText`);
    const ok = start && colder && warmer && field === "number (aucun) -50 50 3" && typed && floor && spin && swedishStart && swedish;
    return [ok, `départ « -2 °C », il gèle : ${start} ; −5 = −7 : ${colder} ; +10 = 3, il ne gèle plus : ${warmer} ; champ (type, inputmode, min, max, valeur) : ${field} ; −40 écrit : ${typed} ; −90 gardé à −50 : ${floor} ; lecteur d'écran, spinbutton : ${spin} ; en suédois : départ ${swedishStart}, après un toucher « ${seen} »`];
  }],
  ["des dates : aujourd'hui, une semaine, des nuits (Days)", async (p) => {
    await p.open("/exemples/lecons/87-des-dates.holo");
    const has = (words) => `document.getElementById("page").innerText.includes(${JSON.stringify(words)})`;
    // Les dates de l'essai, d'après l'horloge de la machine : aujourd'hui, et dans trois jours.
    const iso = (shift) => { const d = new Date(); d.setDate(d.getDate() + shift); return [d.getFullYear(), String(d.getMonth() + 1).padStart(2, "0"), String(d.getDate()).padStart(2, "0")].join("-"); };
    const french = (shift) => { const d = new Date(); d.setDate(d.getDate() + shift); return new Intl.DateTimeFormat("fr-FR", { day: "numeric", month: "long", year: "numeric" }).format(d).replace(/^1 /, "1er "); };
    const weekday = (shift) => { const d = new Date(); d.setDate(d.getDate() + shift); return new Intl.DateTimeFormat("fr-FR", { weekday: "long" }).format(d); };
    const choose = (bind, day) => p.value(`(() => { const i = document.querySelector('input[data-bind="${bind}"]'); i.value = "${day}"; i.dispatchEvent(new Event("input", { bubbles: true })); })()`);
    const today = await p.until(has(`Nous sommes le ${french(0)}.`));
    await choose("arrival", iso(3));
    const arrival = await p.until(has(`Arrivée le ${weekday(3)} ${french(3)}.`));
    await p.click('[data-name="Week"]');
    const week = await p.until(`${has("7 nuit(s) à 80,00 € la nuit.")} && document.querySelector('input[data-bind="departure"]').value === "${iso(10)}"`);
    await p.click('[data-name="Compute"]');
    const price = await p.until(has("Prix du séjour : 560,00 €."));
    // Une arrivée hier : refusée (min: today), l'arrivée choisie reste.
    await choose("arrival", iso(-1));
    await pause(400);
    const refused = await p.value(`${has(`Arrivée le ${weekday(3)} ${french(3)}.`)} && document.querySelector('input[data-bind="arrival"]').value === "${iso(3)}"`);
    const ok = today && arrival && week && price && refused;
    return [ok, `aujourd'hui « ${french(0)} » : ${today} ; arrivée ${iso(3)} : ${arrival} ; une semaine → 7 nuits : ${week} ; 560,00 € : ${price} ; hier refusé : ${refused}`];
  }],
  ["un formulaire qui vérifie : messages, Entrée, un envoi, délai, serveur (leçon 88)", async (p, b) => {
    await p.open("/exemples/lecons/88-un-formulaire-qui-verifie.holo");
    await p.until("window.__holoStarted");
    const has = (words) => `document.getElementById("page").innerText.includes(${JSON.stringify(words)})`;
    const write = async (bind, text) => {
      await p.value(`(() => { const i = document.querySelector('[data-bind="${bind}"]'); i.focus(); i.value = ""; i.dispatchEvent(new Event("input", { bubbles: true })); })()`);
      await p.type(`[data-bind="${bind}"]`, text);
    };
    const errors = () => p.value(`[...document.querySelectorAll(".holo-error")].map((e) => e.textContent).join(" | ")`);
    // Le serveur, imité par Chrome : on compte les envois ; « lent » ne répond jamais.
    let posts = 0;
    let mode = "";
    b.on("Fetch.requestPaused", ({ requestId, request }) => {
      // Seulement les envois : la page relit aussi son fichier, et cette lecture doit passer.
      if (request.method !== "POST") return b.send("Fetch.continueRequest", { requestId });
      posts += 1;
      if (mode === "lent") return;
      setTimeout(() => b.send("Fetch.fulfillRequest", { requestId, responseCode: 204 }), 800);
    });
    await b.send("Fetch.enable", { patterns: [{ urlPattern: "*88-un-formulaire-qui-verifie.holo*", requestStage: "Request" }] });
    const seen = [];
    try {
      // Rien d'écrit : quatre messages, le clavier sur le premier champ, rien n'est parti.
      await p.click('[data-name="Send"]');
      await p.until(`document.querySelectorAll(".holo-error").length === 4`, 5000);
      const four = await errors();
      const focus = await p.value(`(() => { const a = document.activeElement; return [a.dataset.bind, a.getAttribute("aria-invalid"), document.getElementById(a.getAttribute("aria-describedby"))?.textContent].join(" / "); })()`);
      seen.push(`vide → ${four} ; clavier sur ${focus} ; envois : ${posts}`);
      const emptyOk = four.split(" | ").length === 4 && focus.startsWith("name / true / Ce champ est obligatoire.") && posts === 0;
      // Les messages suivent ce qu'on corrige.
      await write("name", "A");
      const short = await p.until(`${has("Au moins 2 caractères.")}`, 3000);
      await write("name", "Ada");
      await write("email", "ada@");
      const email = await p.until(`${has("Écris une adresse e-mail")} && !${has("Au moins 2 caractères.")}`, 3000);
      await write("email", "ada@exemple.fr");
      await write("message", "Bonjour, un essai du formulaire.");
      await p.click('input[data-bind="accept"]');
      const clean = await p.until(`document.querySelectorAll(".holo-error").length === 0`, 3000);
      seen.push(`« A » → trop court : ${short} ; « ada@ » → e-mail : ${email} ; tout corrigé : ${clean}`);
      // Entrée dans un champ envoie ; deux touches rapprochées, un seul envoi.
      await p.value(`document.querySelector('[data-bind="name"]').focus()`);
      await p.key("Enter", "Enter", 13, "\r");
      await p.click('[data-name="Send"]');
      const sent = await p.until(has("Merci, ton message est arrivé."), 5000);
      seen.push(`Entrée → envoyé : ${sent} ; envois : ${posts}`);
      const once = posts === 1;
      // Un serveur qui ne répond pas : un échec après 15 secondes.
      mode = "lent";
      await p.click('[data-name="Send"]');
      const started = Date.now();
      const late = await p.until(has("Le message n'est pas parti"), 20000);
      seen.push(`serveur muet → échec : ${late} (après ${((Date.now() - started) / 1000).toFixed(1)} s)`);
      await b.send("Fetch.disable");
      b.on("Fetch.requestPaused", null);
      // Le serveur vérifie à nouveau : un message forgé, sans passer par la page, est refusé.
      const forged = await p.value(`fetch(location.pathname, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ form: "Contact", values: { name: "", email: "pas-une-adresse", message: "court", accept: 0, admin: 1 } }) }).then(async (r) => r.status + " " + (await r.text()).split("\\n").length + " erreur(s)")`);
      seen.push(`message forgé → ${forged}`);
      const ok = emptyOk && short && email && clean && sent && once && late && forged.startsWith("422 ");
      return [ok, seen.join(" ; ")];
    } finally {
      await b.send("Fetch.disable").catch(() => {});
      b.on("Fetch.requestPaused", null);
    }
  }],
  ["la mise en page : téléphone, ordinateur, la place, ce qui dépasse, les proportions, le curseur, justifié (leçons 89 à 93)", async (p, b) => {
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    const style = (selector, property) => p.value(`(() => { const e = document.querySelector(${JSON.stringify(selector)}); return e ? getComputedStyle(e).getPropertyValue(${JSON.stringify(property)}) : "absent"; })()`);
    const shown = (selector) => p.value(`(() => { const e = document.querySelector(${JSON.stringify(selector)}); return e ? getComputedStyle(e).display !== "none" : "absent"; })()`);
    const width = (selector) => p.value(`document.querySelector(${JSON.stringify(selector)})?.getBoundingClientRect().width ?? -1`);
    try {
      // Sur un ordinateur (1280 de large) : la page s'élargit à 960, la phrase « téléphone » reste, l'autre se cache.
      await b.send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 800, deviceScaleFactor: 1, mobile: false });
      await p.open("/exemples/lecons/89-telephone-et-ordinateur.holo", 600);
      check("ordinateur, largeur de la page", Math.abs((await width("#page main")) - 960) < 2, await width("#page main"));
      check("ordinateur, bandeau à 22px", (await style(".holo-s-bandeau", "font-size")) === "22px", await style(".holo-s-bandeau", "font-size"));
      check("ordinateur, la phrase « téléphone » se voit", (await shown(".holo-s-grand")) === true, await shown(".holo-s-grand"));
      check("ordinateur, la phrase « ordinateur » se cache", (await shown(".holo-s-petit")) === false, await shown(".holo-s-petit"));
      // Plus bas, la place : la grande case garde sa phrase, les trois petites la cachent.
      const narrow = await p.until(`document.querySelectorAll(".holo-narrow").length === 3`, 5000);
      check("la place : trois cases étroites marquées", narrow, await p.value(`document.querySelectorAll(".holo-narrow").length`));
      check("la place : la grande case lit tout", (await p.value(`getComputedStyle(document.querySelectorAll(".holo-Grid")[0].querySelector(".holo-s-detail")).display !== "none"`)) === true, "phrase cachée");
      check("la place : une petite case cache sa phrase", (await shown(".holo-narrow .holo-s-detail")) === false, await shown(".holo-narrow .holo-s-detail"));
      check("la place : son titre rapetisse", (await style(".holo-narrow .holo-H3", "font-size")) === "16px", await style(".holo-narrow .holo-H3", "font-size"));
      // Sur un téléphone (400 de large) : l'inverse, et rien ne déborde.
      await b.send("Emulation.setDeviceMetricsOverride", { width: 400, height: 800, deviceScaleFactor: 2, mobile: true });
      await p.open("/exemples/lecons/89-telephone-et-ordinateur.holo", 600);
      check("téléphone, bandeau à 14px", (await style(".holo-s-bandeau", "font-size")) === "14px", await style(".holo-s-bandeau", "font-size"));
      check("téléphone, la phrase « téléphone » se cache", (await shown(".holo-s-grand")) === false, await shown(".holo-s-grand"));
      check("téléphone, la phrase « ordinateur » se voit", (await shown(".holo-s-petit")) === true, await shown(".holo-s-petit"));
      // Leçon 90, sur un téléphone : rien ne déborde, « … » après trois lignes, la boîte défile.
      await p.open("/exemples/lecons/90-ce-qui-depasse.holo", 600);
      const overflow = await p.value("document.documentElement.scrollWidth - document.documentElement.clientWidth");
      check("ce qui dépasse : le long mot ne fait pas déborder la page", overflow <= 0, `${overflow}px de trop`);
      check("ce qui dépasse : trois lignes puis « … »", (await style(".holo-s-resume", "-webkit-line-clamp")) === "3", await style(".holo-s-resume", "-webkit-line-clamp"));
      const resumeHeight = await p.value(`document.querySelector(".holo-s-resume").getBoundingClientRect().height`);
      check("ce qui dépasse : le résumé tient en trois lignes", resumeHeight > 40 && resumeHeight < 90, `${resumeHeight}px`);
      check("ce qui dépasse : la boîte défile", (await p.value(`(() => { const b = document.querySelector(".holo-s-boite"); return b.scrollHeight > b.clientHeight && getComputedStyle(b).overflowY === "auto"; })()`)) === true, "pas de défilement");
      check("ce qui dépasse : le prix ne se coupe pas", (await style(".holo-s-prix", "white-space")) === "nowrap", await style(".holo-s-prix", "white-space"));
      // Leçon 91 : des carrés, l'image coupée ou entière.
      await p.open("/exemples/lecons/91-garder-des-proportions.holo", 600);
      const square = await p.value(`(() => { const r = document.querySelector(".holo-s-carre").getBoundingClientRect(); return Math.abs(r.width - r.height) < 1 && r.width > 100; })()`);
      check("proportions : un carré", square === true, await p.value(`JSON.stringify(document.querySelector(".holo-s-carre").getBoundingClientRect())`));
      check("proportions : coupée au milieu", (await style(".holo-s-carre", "object-fit")) === "cover", await style(".holo-s-carre", "object-fit"));
      check("proportions : coupée à gauche", (await style(".holo-s-gauche", "object-position")) === "0% 50%", await style(".holo-s-gauche", "object-position"));
      check("proportions : en entier", (await style(".holo-s-entier", "object-fit")) === "contain", await style(".holo-s-entier", "object-fit"));
      // Leçon 92 : le curseur, dessiné avec sa forme de secours.
      await p.open("/exemples/lecons/92-le-curseur.holo", 600);
      check("curseur : une aide", (await style(".holo-s-aide", "cursor")) === "help", await style(".holo-s-aide", "cursor"));
      const drawn = await style(".holo-s-dessin", "cursor");
      check("curseur : dessiné, avec « auto » de secours", /viseur\.svg"?\),\s*auto$/.test(drawn), drawn);
      // Leçon 93 : justifié, avec les coupures de mots.
      await p.open("/exemples/lecons/93-texte-justifie.holo", 600);
      check("justifié", (await style(".holo-s-livre", "text-align")) === "justify", await style(".holo-s-livre", "text-align"));
      check("justifié : les mots se coupent", (await style(".holo-s-livre", "hyphens")) === "auto", await style(".holo-s-livre", "hyphens"));
      if (b.errors.length) faults.push(`erreurs : ${b.errors.join(" | ")}`);
    } finally {
      await b.send("Emulation.clearDeviceMetricsOverride");
    }
    return [faults.length === 0, faults.length ? faults.join("\n      ") : "24 vérifications, sur un ordinateur et sur un téléphone"];
  }],
  ["le zoom : accrochée par défaut, décrochée sur demande (leçon 94), et les touches à l'écran (leçon 77)", async (p, b) => {
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    await b.send("Emulation.setDeviceMetricsOverride", { width: 400, height: 800, deviceScaleFactor: 2, mobile: true });
    await b.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
    const fingers = (gap) => [{ x: 200 - gap, y: 300, id: 1 }, { x: 200 + gap, y: 300, id: 2 }];
    const pinch = async () => {
      await b.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: fingers(40) });
      for (let g = 50; g <= 160; g += 10) await b.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: fingers(g) });
      await b.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
      await pause(600);
    };
    const grown = () => p.value(`(() => { const m = document.querySelector("#page .holo-Page"); return !!m && getComputedStyle(m).transform !== "none"; })()`);
    try {
      // La leçon 9 (des points) : le pincement reste à la page, comme avant.
      await p.open("/exemples/lecons/09-zoom-et-points.holo", 600);
      check("avec des points : le zoom est au moteur", (await p.value(`document.documentElement.classList.contains("holo-zoom")`)) === true, "le pincement est resté au navigateur");
      // La leçon 94 : accrochée d'abord ; « Décrocher » dans le menu ; décrochée, le moteur approche la page ;
      // « Accrocher » la remet à sa place.
      await p.open("/exemples/lecons/94-decrocher-la-page.holo", 600);
      check("leçon 94 : accrochée au départ", (await p.value(`!document.documentElement.classList.contains("holo-zoom")`)) === true, "décrochée au départ");
      await p.click("#toggle");
      const offered = await p.until(`document.getElementById("detach") && !document.getElementById("detach").hidden`);
      check("leçon 94 : « Décrocher » est offert", offered, "pas de bouton");
      check("leçon 94 : « Décrocher » n'est pas sur une page ordinaire", (await p.value(`document.getElementById("detach").textContent`)) === "Décrocher", await p.value(`document.getElementById("detach").textContent`));
      await p.click("#detach");
      check("leçon 94 : décrochée", (await p.value(`document.body.classList.contains("detached") && document.getElementById("detach").textContent === "Accrocher"`)) === true, "pas décrochée");
      await pinch();
      check("leçon 94 : décrochée, le moteur approche la page", (await grown()) === true, "la page n'a pas grossi");
      await p.click("#detach");
      check("leçon 94 : accrochée à nouveau, à sa taille", (await p.value(`!document.body.classList.contains("detached")`)) === true && (await grown()) === false, "pas revenue");
      // La leçon 77 au doigt : les touches en bas de l'écran (l'écran tactile sans souris est simulé).
      await b.send("Emulation.setEmulatedMedia", { features: [{ name: "hover", value: "none" }, { name: "pointer", value: "coarse" }] });
      await p.open("/exemples/lecons/77-toutes-les-touches.holo", 600);
      const keys = await p.until(`document.querySelectorAll("#keys button").length === 5`, 15000);
      check("leçon 77 au doigt : cinq touches à l'écran", keys, await p.value(`document.querySelectorAll("#keys button").length`));
      check("leçon 77 au doigt : les étiquettes", (await p.value(`[...document.querySelectorAll("#keys button")].map((b) => b.textContent).join(" ")`)) === "P M 5 Entrée Échap", await p.value(`[...document.querySelectorAll("#keys button")].map((b) => b.textContent).join(" ")`));
      const tap = async (key) => {
        const box = await p.value(`(() => { const b = document.querySelector('#keys button[data-key="${key}"]').getBoundingClientRect(); return [b.left + b.width / 2, b.top + b.height / 2]; })()`);
        await b.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: box[0], y: box[1], id: 1 }] });
        await b.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
        await pause(300);
      };
      await tap("p");
      await tap("digit5");
      await tap("enter");
      const count = () => p.value(`document.querySelector('[data-state="compte"]')?.textContent`);
      check("leçon 77 au doigt : P, 5, Entrée → 16", (await count()) === "16", await count());
      await tap("escape");
      check("leçon 77 au doigt : Échap → 0", (await count()) === "0", await count());
      // Avec une souris, sur un ordinateur : pas de touches à l'écran, le clavier suffit.
      await b.send("Emulation.setEmulatedMedia", { features: [] });
      await b.send("Emulation.setTouchEmulationEnabled", { enabled: false });
      await b.send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 800, deviceScaleFactor: 1, mobile: false });
      await p.open("/exemples/lecons/77-toutes-les-touches.holo", 600);
      await p.click("#toggle");
      await p.until(`document.getElementById("tools") && !document.getElementById("tools").hidden`);
      check("leçon 77 à la souris : pas de touches à l'écran", (await p.value(`document.getElementById("keys").hidden`)) === true, "des touches sont affichées");
      // Une page ordinaire (leçon 1), au doigt : c'est le navigateur qui grossit la page, sur place.
      // Le moteur ne grossit rien, et il n'est même pas demandé. (En dernier : après un pincement
      // du navigateur, le Chrome d'essai ne transmet plus les doigts simulés aux pages suivantes.)
      await b.send("Emulation.setDeviceMetricsOverride", { width: 400, height: 800, deviceScaleFactor: 2, mobile: true });
      await b.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
      await p.open("/exemples/lecons/01-page.holo", 600);
      check("page ordinaire : le zoom est au navigateur", (await p.value(`!document.documentElement.classList.contains("holo-zoom")`)) === true, "la page garde le pincement");
      await pinch();
      const scale = await p.value("visualViewport.scale");
      check("page ordinaire : le navigateur a grossi la page", scale > 1, `grossissement ${scale}`);
      check("page ordinaire : le moteur ne grossit pas la page", (await grown()) === false, "la page a grossi par le moteur");
      check("page ordinaire : le moteur n'est pas demandé pour pincer", (await p.value(`!document.querySelector('script[src^="/page-engine.js"]')`)) === true, "le moteur a été demandé");
      if (b.errors.length) faults.push(`erreurs : ${b.errors.join(" | ")}`);
    } finally {
      await b.send("Emulation.setEmulatedMedia", { features: [] });
      await b.send("Emulation.setTouchEmulationEnabled", { enabled: false });
      await b.send("Emulation.clearDeviceMetricsOverride");
    }
    return [faults.length === 0, faults.length ? faults.join("\n      ") : "accrochée, décrochée, accrochée ; cinq touches au doigt"];
  }],
  ["depuis le Big Bang, les leçons et la pile se touchent (vu sur le téléphone)", async (p, b) => {
    // Trouvé par Yocthan sur son téléphone, le 2026-10-07 : depuis le Big Bang, rien ne menait aux leçons.
    await b.send("Emulation.setDeviceMetricsOverride", { width: 400, height: 800, deviceScaleFactor: 2, mobile: true });
    try {
      await p.open("/", 600);
      const links = await p.value(`[...document.querySelectorAll("nav a")].map((a) => a.textContent + " → " + a.getAttribute("href")).join(" ; ")`);
      const onTop = await p.value(`(() => { const a = document.querySelector('nav a[href$="01-page.holo"]'); const r = a.getBoundingClientRect(); return document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2) === a && r.height >= 44; })()`);
      await p.click('nav a[href$="01-page.holo"]');
      const moved = await p.until(`location.pathname.endsWith("01-page.holo")`, 8000);
      // La leçon 26, le Big Bang, n'a pas de liens en bas de page : la suivante est en haut.
      await p.open("/exemples/lecons/26-point-seul.holo", 600);
      const next = await p.until(`document.querySelector('nav a[href$="27-donnees.holo"]')?.textContent === "Leçon 27 →"`, 8000);
      return [onTop && moved && next, `${links} ; touchable : ${onTop} ; mène à la leçon 1 : ${moved} ; leçon 26 → 27 : ${next}`];
    } finally {
      await b.send("Emulation.clearDeviceMetricsOverride");
    }
  }],
  ["holo serve sert une adresse qui porte une valeur, et un formulaire envoyé de cette adresse (serve, ADR-078)", async (p, b) => {
    // Le vrai serveur (ADR-074) : un modèle `profils/{nom}.holo`, ouvert à `/profils/ada`. La page
    // fabriquée par le serveur, le moteur qui garde la valeur, le message rangé avec son modèle.
    const binary = ["holo", "holo.exe"].map((name) => join(engine, "target", "release", name)).find(existsSync);
    if (!binary) return [false, "holo n'est pas construit : cargo build --release --bin holo"];
    const folder = mkdtempSync(join(tmpdir(), "holo-serve-"));
    mkdirSync(join(folder, "profils"));
    writeFileSync(join(folder, "profils", "{nom}.holo"), [
      'Page(title: "Profil", state: State(email: "", sent: 0), children: [',
      '  H1("Bonjour, {nom}"),',
      '  If(nom, is: "ada", children: [ P.ada("Ada a écrit le premier programme.") ]),',
      '  Form(name: Contact, children: [ Input(value: email, type: email, label: "Ton e-mail", required: true), Button(name: Send, text: "Envoyer") ]),',
      '  If(sent, is: 1, children: [ P("Merci, ton message est arrivé.") ]),',
      '], rules: [ On(Send.tap, effect: Contact.send), On(Contact.sent, effect: sent.set(1)) ])',
      '.ada { color: #8fd3ff; }',
    ].join("\n"));
    const port = 22000 + Math.floor(Math.random() * 2000);
    const server = spawn(binary, ["serve", folder, String(port)], { cwd: engine, stdio: ["ignore", "pipe", "pipe"] });
    let output = "";
    server.stdout.on("data", (d) => { output += d; });
    server.stderr.on("data", (d) => { output += d; });
    try {
      for (let i = 0; i < 100 && !output.includes(`localhost:${port}`); i++) await pause(100);
      if (!output.includes(`localhost:${port}`)) return [false, `holo serve ne démarre pas : ${output.trim()}`];
      const q = page(b, `http://localhost:${port}`);
      // La page fabriquée par le serveur, avant le moteur.
      await q.open("/profils/ada", 300);
      const made = await q.value(`document.querySelector("#page h1")?.textContent`);
      const shown = await q.value(`(() => { const e = document.querySelector(".holo-s-ada"); return !!e && getComputedStyle(e).display !== "none"; })()`);
      const named = await q.value(`document.querySelector('meta[name="holo-file"]')?.content`);
      // Le moteur arrive (le menu le fait venir) : la même valeur, aucune erreur.
      await q.click("#toggle");
      const engineThere = await q.until(`document.getElementById("tools") && !document.getElementById("tools").hidden`, 40000);
      const kept = await q.value(`document.querySelector("#page h1")?.textContent`);
      const errors = b.errors.join(" | ");
      // Le formulaire, envoyé par le moteur à l'adresse : vérifié avec la valeur, rangé avec son modèle.
      await q.type('#page input[type=email]', "ada@example.org");
      await q.click('[data-name="Send"]');
      const thanked = await q.until(`document.getElementById("page").innerText.includes("Merci, ton message est arrivé.")`, 15000);
      let stored = "base non lue (node:sqlite absent)";
      try {
        const { DatabaseSync } = await import("node:sqlite");
        const base = new DatabaseSync(join(folder, "holo-data", "site.sqlite"));
        const row = base.prepare("SELECT page, model FROM messages").get();
        base.close();
        stored = row ? `${row.page} ${row.model}` : "aucun message";
      } catch (error) {
        if (!String(error).includes("node:sqlite")) stored = `base illisible : ${error.message}`;
      }
      const ok = made === "Bonjour, ada" && shown === true && named === "/profils/{nom}.holo" && engineThere && kept === "Bonjour, ada" && !errors && thanked
        && (stored === "/profils/ada /profils/{nom}.holo" || stored.startsWith("base non lue"));
      return [ok, `fabriquée : « ${made} » ; paragraphe d'Ada : ${shown} ; fichier nommé : ${named} ; moteur : ${engineThere}, « ${kept} »${errors ? ` ; erreurs : ${errors}` : ""} ; message : ${thanked ? "Merci" : "pas de Merci"} ; rangé : ${stored}`];
    } finally {
      server.kill();
      await pause(300);
      try { rmSync(folder, { recursive: true, force: true }); } catch { /* tant pis */ }
    }
  }],
  ["une fenêtre fermée ne couvre pas la page (leçon 63, vu sur le téléphone)", async (p) => {
    // Trouvé par Yocthan sur son téléphone, le 2026-10-07 : le lien vers la leçon 64 ne se
    // laissait pas toucher, la fenêtre fermée restait posée dessus.
    await p.open("/exemples/lecons/63-fenetre.holo");
    const onTop = () => p.value(`(() => { const a = document.querySelector('a[href$="64-formulaire.holo"]'); a.scrollIntoView({ block: "center" }); const b = a.getBoundingClientRect(); return document.elementFromPoint(b.left + b.width / 2, b.top + b.height / 2)?.closest("a") === a; })()`);
    const before = await onTop();
    await p.click('[data-name="Demander"]');
    const opened = await p.until(`!!document.querySelector("dialog[open]")`, 10000);
    await p.click('[data-name="Non"]');
    const closed = await p.until(`!document.querySelector("dialog[open]")`, 5000);
    const after = await onTop();
    await p.click('a[href$="64-formulaire.holo"]');
    const moved = await p.until(`location.pathname.endsWith("64-formulaire.holo")`, 8000);
    const ok = before && opened && closed && after && moved;
    return [ok, `lien touchable avant : ${before} ; fenêtre ouverte : ${opened}, puis fermée : ${closed} ; lien touchable après : ${after} ; mène à la leçon 64 : ${moved}`];
  }],
  ["un formulaire envoie son message", async (p) => {
    await p.open("/exemples/lecons/64-formulaire.holo");
    await p.type("#page input", "Ada");
    await p.type("#page textarea", "Un essai du navigateur.");
    await p.click('[data-name="Envoyer"]');
    const ok = await p.until(`document.getElementById("page").innerText.includes("Merci, ton message est arrivé.")`);
    return [ok, ok ? "« Merci, ton message est arrivé. »" : "pas de confirmation"];
  }],
  ["un bloc entre en arrivant à l'écran", async (p) => {
    await p.open("/exemples/lecons/78-apparaitre-en-descendant.holo");
    const waiting = await p.value(`[...document.querySelectorAll(".holo-in-view")].every((b) => !b.classList.contains("holo-seen"))`);
    await p.value(`document.querySelector(".holo-in-view").scrollIntoView({ block: "center" })`);
    const seen = await p.until(`document.querySelector(".holo-in-view").classList.contains("holo-seen")`, 5000);
    return [waiting && seen, `loin de l'écran, en attente : ${waiting} ; arrivé, entré : ${seen}`];
  }],
  ["un son : son volume, sa boucle, son arrêt", async (p) => {
    await p.open("/exemples/lecons/79-regler-un-son.holo");
    await p.click('[data-name="Lancer"]');
    const playing = await p.until(`(() => { const a = document.querySelector('audio[data-name="Pluie"]'); return !a.paused && a.loop && a.volume === 0.6; })()`);
    await p.click('[data-name="Arreter"]');
    const stopped = await p.until(`(() => { const a = document.querySelector('audio[data-name="Pluie"]'); return a.paused && a.currentTime === 0; })()`, 5000);
    return [playing && stopped, `joue en boucle au volume 0,6 : ${playing} ; arrêté au début : ${stopped}`];
  }],
  ["les tailles suivent le texte du visiteur", async (p) => {
    await p.open("/exemples/lecons/80-tailles-qui-suivent.holo");
    const normal = await p.value(`getComputedStyle(document.querySelector(".holo-s-carte")).padding`);
    await p.value(`document.documentElement.style.fontSize = "24px"`);
    const bigger = await p.value(`getComputedStyle(document.querySelector(".holo-s-carte")).padding`);
    return [normal === "16px 24px" && bigger === "24px 36px", `${normal} → ${bigger}`];
  }],
  ["la vue points se lit au lecteur d'écran", async (p, b) => {
    await p.open("/exemples/lecons/81-vue-points-et-lecteur-d-ecran.holo");
    let inPoints = false;
    for (let i = 0; i < 80 && !inPoints; i++) {
      await b.send("Input.dispatchMouseEvent", { type: "mouseWheel", x: 500, y: 200, deltaX: 0, deltaY: -500, modifiers: 2 });
      await pause(400);
      inPoints = await p.value(`document.body.classList.contains("in-points")`);
    }
    const { result } = await b.send("Accessibility.getFullAXTree");
    const heading = result.nodes.find((n) => n.role?.value === "heading" && n.name?.value === "La vue points se lit aussi");
    const announced = await p.until(`(document.getElementById("announcement")?.textContent ?? "").startsWith("Vue points")`, 5000);
    return [inPoints && heading && !heading.ignored && announced, `vue points : ${inPoints} ; titre lisible : ${Boolean(heading && !heading.ignored)} ; « Vue points » annoncé : ${announced}`];
  }],
  ["sans JavaScript, holo serve fait marcher les boutons (serve)", async (_, b) => {
    const served = await startHoloServe(["14-prix.holo", "68-liste-qui-change.holo"]);
    const q = page(b, served.base);
    try {
      await b.send("Emulation.setScriptExecutionDisabled", { value: true });
      // Une liste : écrire, ajouter (le champ part avec le bouton), puis « Fait » sur la première ligne.
      await q.open("/68-liste-qui-change.holo", 300);
      await q.type('[data-bind="tache"]', "Arroser");
      await q.click('[data-name="Ajouter"]');
      await q.until(`document.readyState === "complete" && document.getElementById("page").innerText.includes("Arroser")`, 5000);
      const added = await q.text();
      await q.click('.holo-line[data-rank="0"] [data-name="Fait"]');
      await q.until(`document.readyState === "complete" && !document.getElementById("page").innerText.includes("Encadrer")`, 5000);
      const done = await q.text();
      // La touche Entrée dans un champ envoie le champ, sans toucher « Ajouter ».
      await q.type('[data-bind="tache"]', "Semer");
      await q.key("Enter", "Enter", 13, "\r");
      await pause(800);
      const entered = await q.text();
      const kept = await q.value(`document.querySelector('[data-bind="tache"]').value`);
      // Des prix : deux pommes, le total calculé par le serveur.
      await q.open("/14-prix.holo", 300);
      await q.click('[data-name="Pomme"]');
      await pause(500);
      await q.click('[data-name="Pomme"]');
      await pause(500);
      const fruits = await q.text();
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      // Avec JavaScript, le moteur repart des valeurs du serveur, et le geste reste dans la page.
      await q.open("/14-prix.holo", 300);
      await q.value("window.__stayed = true");
      await q.click('[data-name="Pomme"]');
      const resumed = await q.until(`document.getElementById("page").innerText.includes("En tout : 3 fruits, 6 euros")`, 40000);
      const stayed = await q.value("window.__stayed === true");
      const ok = added.includes("Arroser") && added.includes("2 tâche(s)") && !done.includes("Encadrer") && done.includes("1 tâche(s)")
        && !entered.includes("Semer\nFait") && kept === "Semer" && fruits.includes("En tout : 2 fruits, 4 euros") && resumed && stayed;
      return [ok, `ajoutée : ${added.includes("Arroser")} ; faite : ${!done.includes("Encadrer")} ; Entrée n'ajoute pas : ${!entered.includes("Semer\nFait")} (champ gardé : ${kept}) ; 2 pommes, 4 euros : ${fruits.includes("2 fruits, 4 euros")} ; avec JavaScript, repris : ${resumed}, sans recharger : ${stayed}`];
    } finally {
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      served.stop();
    }
  }],
  ["un formulaire reçu par holo serve, avec ou sans JavaScript (serve)", async (_, b) => {
    const served = await startHoloServe(["88-un-formulaire-qui-verifie.holo"]);
    const q = page(b, served.base);
    const thanks = `document.getElementById("page").innerText.includes("Merci, ton message est arrivé.")`;
    try {
      await b.send("Emulation.setScriptExecutionDisabled", { value: true });
      await q.open("/88-un-formulaire-qui-verifie.holo", 300);
      // Envoyer vide : les messages sous les champs, écrits par le serveur.
      await q.click('[data-name="Send"]');
      await q.until(`document.readyState === "complete" && document.querySelectorAll(".holo-error").length > 0`, 5000);
      const errors = await q.value(`[...document.querySelectorAll(".holo-error")].map((e) => e.textContent).join(" | ")`);
      const linked = await q.value(`document.querySelector('[data-bind="name"]').getAttribute("aria-describedby") === document.querySelector(".holo-error").id`);
      // Tout remplir : envoyé, rangé, merci.
      await q.type('[data-bind="name"]', "Ada");
      await q.type('[data-bind="email"]', "ada@exemple.fr");
      await q.type('[data-bind="message"]', "Bonjour, une question.");
      await q.click('[data-bind="accept"]');
      await q.click('[data-name="Send"]');
      const sentWithout = await q.until(`document.readyState === "complete" && ${thanks}`, 5000);
      const noErrors = await q.value(`document.querySelectorAll(".holo-error").length === 0`);
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      // Avec JavaScript, un nouveau visiteur : le moteur envoie en JSON, holo serve range et répond 204.
      await b.send("Network.clearBrowserCookies");
      await q.open("/88-un-formulaire-qui-verifie.holo", 300);
      await q.value("window.__stayed = true");
      await q.type('[data-bind="name"]', "Bob");
      await q.until("window.__holoStarted", 40000);
      for (const [bind, text] of [["email", "bob@exemple.fr"], ["message", "Une autre question."]]) await q.type(`[data-bind="${bind}"]`, text);
      await q.click('[data-bind="accept"]');
      await q.click('[data-name="Send"]');
      const sentWith = await q.until(thanks, 15000);
      const stayed = await q.value("window.__stayed === true");
      const kept = spawnSync(["holo", "holo.exe"].map((name) => join(engine, "target", "release", name)).find(existsSync), ["messages", served.folder], { encoding: "utf8" }).stdout.trim().split("\n").filter(Boolean);
      const ok = errors.includes("Ce champ est obligatoire.") && linked && sentWithout && noErrors && sentWith && stayed && kept.length === 2 && kept[0].includes('"name":"Ada"') && kept[1].includes('"name":"Bob"');
      return [ok, `sans JavaScript, erreurs : ${errors.split(" | ").length} (reliées : ${linked}) ; envoyé : ${sentWithout} ; avec JavaScript, envoyé : ${sentWith}, sans recharger : ${stayed} ; messages rangés : ${kept.length}`];
    } finally {
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      served.stop();
    }
  }],
  ["une adresse porte une valeur (leçon 100), et un formulaire part de cette adresse", async (p, b) => {
    // Le fichier 100-profils/{nom}.holo sert …/100-profils/ada, …/yocthan, …/Adé (ADR-078).
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    const title = () => p.value(`document.querySelector("#page h1")?.textContent ?? "(pas de titre)"`);
    const engineAsked = () => p.value(`!!document.querySelector('script[src^="/page-engine.js"]')`);
    const refused = () => p.value(`document.getElementById("error")?.textContent ?? ""`);
    // Le bloc de l'If est invisible lui-même (display: contents) : on regarde son paragraphe.
    const yocthanSeen = () => p.value(`(() => { const b = document.querySelector('#page [data-if^="nom|"]'); const p = b?.querySelector("p"); return !!p && !b.hidden && p.getClientRects().length > 0; })()`);
    const startEngine = async () => {
      await p.click("#toggle");
      return p.until("window.__holoStarted === true", 20000);
    };
    // Le serveur fabrique la page avec la valeur de l'adresse : le titre est là avant le moteur.
    await p.open("/exemples/lecons/100-profils/ada", 600);
    const served = await title();
    check("ada, page du serveur", served === "Bonjour, ada" && !(await engineAsked()), `« ${served} », moteur déjà demandé : ${await engineAsked()}`);
    check("ada, le paragraphe de Yocthan reste caché", (await yocthanSeen()) === false, "il se voit");
    // Le menu fait venir le moteur : il lit le modèle (holo-file), avec la même valeur.
    const started = await startEngine();
    const taken = await title();
    check("ada, avec le moteur", started && taken === "Bonjour, ada" && !(await refused()) && b.errors.length === 0, `moteur arrivé : ${started} ; « ${taken} » ; ${(await refused()) || b.errors.join(" | ") || "aucune erreur"}`);
    // Yocthan : le paragraphe de l'If se voit.
    await p.open("/exemples/lecons/100-profils/yocthan", 600);
    check("yocthan, le paragraphe se voit", (await title()) === "Bonjour, yocthan" && (await yocthanSeen()) === true, `« ${await title()} », paragraphe : ${await yocthanSeen()}`);
    // Un nom accentué, encodé dans l'adresse : décodé par le moteur, au serveur puis dans la page.
    await p.open("/exemples/lecons/100-profils/Ad%C3%A9", 600);
    const accent = await title();
    const accentStarted = await startEngine();
    const accentTaken = await title();
    check("Adé, le nom accentué", accent === "Bonjour, Adé" && accentStarted && accentTaken === "Bonjour, Adé" && !(await refused()) && b.errors.length === 0, `serveur « ${accent} » ; moteur « ${accentTaken} » ; ${(await refused()) || b.errors.join(" | ") || "aucune erreur"}`);
    // Les liens : de la leçon vers le nom accentué, puis du modèle vers la leçon (« ../ »).
    await p.open("/exemples/lecons/100-une-adresse-qui-porte-une-valeur.holo", 600);
    await p.click('#page a[href$="100-profils/Adé"]');
    const followed = await p.until(`location.pathname.endsWith("/100-profils/Ad%C3%A9") && document.querySelector("#page h1")?.textContent === "Bonjour, Adé"`, 8000);
    check("le lien vers Adé", followed, await p.value("location.pathname"));
    await p.click('#page a[href$="100-une-adresse-qui-porte-une-valeur.holo"]');
    const back = await p.until(`location.pathname === "/exemples/lecons/100-une-adresse-qui-porte-une-valeur.holo"`, 8000);
    check("le lien de retour à la leçon", back, await p.value("location.pathname"));
    // Une adresse que rien ne sert : pas de page.
    const missing = await p.value(`fetch("/exemples/lecons/100-profils/a/b", { headers: { accept: "text/html" } }).then((r) => r.status)`);
    check("a/b, pas de page", missing === 404, missing);
    // L'éditeur vérifie le modèle comme holo check : un nom vide.
    await p.open(`/editor?file=${encodeURIComponent("/exemples/lecons/100-profils/{nom}.holo")}`, 600);
    const edited = await p.until(`document.getElementById("status").textContent.startsWith("✓")`, 15000);
    check("l'éditeur accepte le modèle", edited, `${await p.value(`document.getElementById("status").textContent`)} ; ${b.errors.join(" | ") || "aucune erreur"}`);
    // Un formulaire envoyé depuis une adresse : il part vers l'adresse, où le serveur retrouve le
    // modèle et la valeur ; le message garde son adresse. Un message forgé est vérifié de même.
    await p.open("/exemples/.essais-navigateur/adresse/ada", 600);
    await startEngine();
    const written = `Un essai depuis l'adresse, ${Date.now()}`;
    await p.type('[data-bind="message"]', written);
    await p.click('[data-name="Send"]');
    const sent = await p.until(`document.getElementById("page").innerText.includes("Message envoyé.")`, 10000);
    // Le dernier message rangé (un fichier par modèle, dans messages/ du dépôt) ; pas lu avec
    // --telephone, où le serveur 8080 peut servir un autre dossier.
    let recorded = true;
    let record = "le fichier des messages n'est pas lu avec --telephone";
    if (!phone) {
      const lines = readFileSync(join(repo, "messages", "exemples_essais-navigateur_adresse_nom_.jsonl"), "utf8").trim().split("\n");
      const message = JSON.parse(lines.at(-1));
      recorded = message.page === "/exemples/.essais-navigateur/adresse/ada" && message.values?.message === written;
      record = `rangé : ${lines.at(-1)}`;
    }
    check("le formulaire part de l'adresse", sent && recorded, `envoyé : ${sent} ; ${record}`);
    const forged = await p.value(`fetch(location.pathname, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ form: "Contact", values: { message: "" } }) }).then((r) => r.status)`);
    check("un message forgé, refusé", forged === 422, forged);
    return [faults.length === 0, faults.length ? faults.join("\n      ") : "ada, yocthan et Adé, servis puis repris par le moteur ; les liens ; a/b sans page ; l'éditeur ; un formulaire envoyé à l'adresse, un message forgé refusé"];
  }],
  // Les comptes (ADR-081) : seulement dans holo serve ; le serveur d'essai le dit.
  ["les comptes demandent holo serve : le serveur d'essai le dit (leçons 104 à 106)", async (p) => {
    await p.open("/account/signin", 300);
    const told = await p.value("document.body.innerText");
    const status = await p.value(`fetch("/account/signup").then((r) => r.status)`);
    await p.open("/exemples/lecons/105-une-page-reservee.holo", 300);
    const reserved = await p.value("document.body.innerText");
    await p.open("/exemples/lecons/104-se-connecter.holo", 600);
    const lesson = await p.text();
    const ok = told.includes("holo serve") && status === 501 && reserved.includes("réservée aux membres") && !reserved.includes("Seules les personnes connectées voient cette page") && lesson.includes("Tu n'es pas connecté");
    return [ok, `/account : ${told.includes("holo serve") ? "« holo serve » demandé" : told.slice(0, 80)} (${status}) ; la page réservée n'est pas montrée : ${!reserved.includes("Seules les personnes connectées")} ; leçon 104, pas connecté : ${lesson.includes("Tu n'es pas connecté")}`];
  }],
  ["un compte, son code à 6 chiffres, une page réservée, avec et sans JavaScript (serve)", async (_, b) => {
    const served = await startHoloServe(["104-se-connecter.holo", "105-une-page-reservee.holo", "106-le-panier-qui-suit-le-compte.holo"]);
    const q = page(b, served.base);
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    const where = () => q.value("location.pathname + location.search");
    const words = () => q.value("document.body.innerText");
    const after = (expression, timeout = 10000) => q.until(`document.readyState === "complete" && (${expression})`, timeout);
    const send = async (expression) => { await q.click('main form button[type="submit"]'); return after(expression); };
    const alert = () => q.value(`document.querySelector('[role="alert"]')?.textContent ?? ""`);
    const password = "une phrase que je connais";
    // Un parcours entier, pour un nom : créer le compte, activer le code, se déconnecter, se reconnecter.
    const journey = async (name, script) => {
      await b.send("Emulation.setScriptExecutionDisabled", { value: !script });
      await b.send("Network.clearBrowserCookies");
      const how = script ? "avec JavaScript" : "sans JavaScript";
      // La page réservée mène à « Se connecter », qui garde où revenir.
      await q.open("/105-une-page-reservee.holo", 300);
      check(`${how}, la page réservée mène à « Se connecter »`, (await where()) === "/account/signin?next=/105-une-page-reservee.holo&for=members" && (await words()).includes("Cette page est réservée aux membres"), await where());
      await q.click('a[href^="/account/signup"]');
      await after(`location.pathname === "/account/signup"`);
      // Un mot de passe trop court : refusé, le message sous le champ, relié à lui, lu tout de suite.
      await q.type("#name", name);
      await q.type("#password", "court");
      await q.type("#again", "court");
      await send(`document.getElementById("password-error")`);
      const short = await q.value(`[document.getElementById("password-error")?.textContent, document.getElementById("password").getAttribute("aria-describedby"), document.getElementById("password").getAttribute("aria-invalid"), document.getElementById("password").getAttribute("autocomplete")]`);
      check(`${how}, un mot de passe trop court`, short[0] === "Le mot de passe est trop court : 12 caractères au moins." && short[1].includes("password-error") && short[2] === "true" && short[3] === "new-password" && (await alert()).includes("trop court"), JSON.stringify(short));
      // Le bon : le compte est créé, et la page réservée s'ouvre, avec le nom.
      await q.type("#password", password);
      await q.type("#again", password);
      await send(`location.pathname === "/105-une-page-reservee.holo"`);
      check(`${how}, compte créé, la page réservée s'ouvre`, (await q.text()).includes(`Bonjour, ${name}.`), await q.text());
      // Activer le code : la clé lue sur la page, le code calculé ici, comme le fait l'application.
      await q.open("/account", 300);
      await q.click('form[action="/account/code/setup"] button');
      await after(`location.pathname === "/account/code/setup"`);
      const key = (await q.value(`document.getElementById("key")?.textContent ?? ""`)).replace(/\s+/g, "");
      const used = stepNow();
      await q.type("#code", totp(key, used));
      await send(`document.querySelectorAll("[data-recovery]").length === 10`);
      await q.click('a[href="/account?done=code"]');
      await after(`location.pathname + location.search === "/account?done=code"`);
      check(`${how}, le code s'active`, key.length === 32 && (await words()).includes("Le code à 6 chiffres est activé"), `clé de ${key.length} lettres ; ${(await words()).slice(0, 120)}`);
      // Se déconnecter : la page réservée ne s'ouvre plus.
      await q.click('form[action="/account/signout"] button');
      await after(`location.search === "?done=signedout"`);
      const out = await words();
      await q.open("/105-une-page-reservee.holo", 300);
      check(`${how}, déconnecté`, out.includes("Tu t'es déconnecté.") && (await where()).startsWith("/account/signin"), `${out.slice(0, 80)} ; ${await where()}`);
      // Un mauvais mot de passe, un nom inconnu : le même message, qui ne dit pas lequel est faux.
      await q.type("#name", name);
      await q.type("#password", "pas le bon mot de passe");
      await send(`document.querySelector('[role="alert"]')`);
      const wrong = await alert();
      await q.value(`document.getElementById("name").value = ""`);
      await q.type("#name", "Personne");
      await q.type("#password", "pas le bon mot de passe");
      await send(`document.querySelector('[role="alert"]') && document.getElementById("name").value === "Personne"`);
      const nobody = await alert();
      check(`${how}, le message ne dit pas lequel est faux`, wrong === "Ce nom et ce mot de passe ne vont pas ensemble." && nobody === wrong, `« ${wrong} » / « ${nobody} »`);
      // Le bon mot de passe, puis le code : un mauvais, refusé ; puis le suivant (celui de
      // l'activation a déjà servi), qui ramène à la page réservée.
      await q.value(`document.getElementById("name").value = ""`);
      await q.type("#name", name);
      await q.type("#password", password);
      await send(`location.pathname === "/account/code"`);
      await q.type("#code", wrongCode(key));
      await send(`document.querySelector('[role="alert"]')`);
      const refused = await alert();
      await q.type("#code", totp(key, used + 1));
      await send(`location.pathname === "/105-une-page-reservee.holo"`);
      check(`${how}, connecté avec le code`, refused.startsWith("Ce code ne va pas") && (await q.text()).includes(`Bonjour, ${name}.`), `${refused} ; ${(await q.text()).slice(0, 80)}`);
      // Le moteur de la page prend la main : il lit le même nom (l'en-tête de holo serve).
      if (script) {
        await q.click("#toggle");
        const started = await q.until("window.__holoStarted === true", 20000);
        const fault = await q.value(`document.getElementById("error")?.textContent ?? ""`);
        check(`${how}, le moteur garde le nom`, started && (await q.text()).includes(`Bonjour, ${name}.`) && !fault && b.errors.length === 0, `moteur : ${started} ; ${fault || b.errors.join(" | ") || "aucune erreur"}`);
      }
    };
    try {
      await journey("Ada", true);
      await journey("Bob", false);
      return [faults.length === 0, faults.length ? faults.join("\n      ") : "avec et sans JavaScript : la page réservée mène à « Se connecter » ; un mot de passe trop court refusé, le message relié au champ ; le compte créé ; le code activé (calculé par l'essai) ; déconnecté ; le même message pour un mauvais mot de passe et pour un nom inconnu ; un mauvais code refusé, puis connecté avec le code ; le moteur garde le nom"];
    } finally {
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      served.stop();
    }
  }],
  ["le panier suit le compte, sur deux appareils, avec et sans JavaScript (serve)", async (_, b) => {
    const served = await startHoloServe(["104-se-connecter.holo", "105-une-page-reservee.holo", "106-le-panier-qui-suit-le-compte.holo"]);
    const q = page(b, served.base);
    const lesson = "/106-le-panier-qui-suit-le-compte.holo";
    const cart = () => q.value(`(document.getElementById("page")?.innerText.match(/Dans le panier : (\\d+)/) ?? [])[1] ?? "?"`);
    const after = (expression, timeout = 10000) => q.until(`document.readyState === "complete" && (${expression})`, timeout);
    // Un autre appareil : ni cookie, ni rien de gardé par le navigateur.
    const newDevice = async () => {
      await b.send("Network.clearBrowserCookies");
      await b.send("Storage.clearDataForOrigin", { origin: served.base, storageTypes: "cookies,local_storage" });
    };
    const password = "une phrase que je connais";
    const signIn = async () => {
      await q.type("#name", "Cleo");
      await q.type("#password", password);
      await q.click('main form button[type="submit"]');
      await after(`location.pathname === "${lesson}"`);
    };
    try {
      // Sur le téléphone, sans JavaScript : un compte, et deux tableaux dans le panier.
      await b.send("Emulation.setScriptExecutionDisabled", { value: true });
      await newDevice();
      await q.open(`/account/signup?next=${lesson}`, 300);
      await q.type("#name", "Cleo");
      await q.type("#password", password);
      await q.type("#again", password);
      await q.click('main form button[type="submit"]');
      await after(`location.pathname === "${lesson}"`);
      for (const count of [1, 2]) {
        await q.click('[data-name="Ajouter"]');
        await after(`document.getElementById("page").innerText.includes("Dans le panier : ${count}")`);
      }
      const phone = await cart();
      const named = (await q.text()).includes("Tu es connecté sous le nom Cleo");
      // Sur l'ordinateur, un autre appareil : pas connecté, le panier est vide ; le lien « Se
      // connecter » de la leçon y ramène, et le panier du compte y est.
      await newDevice();
      await q.open(lesson, 300);
      const anonymous = await cart();
      await q.click('#page a[href="/account/signin"]');
      await after(`location.pathname === "/account/signin"`);
      await signIn();
      const computer = await cart();
      // Avec JavaScript : un troisième tableau, compté tout de suite par la page, renvoyé au serveur.
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      await q.open(lesson, 300);
      await q.click('[data-name="Ajouter"]');
      const counted = await q.until(`document.getElementById("page").innerText.includes("Dans le panier : 3")`, 40000);
      const mirrored = (await q.until("window.__holoMirrored", 5000)) && (await q.value("window.__holoMirrored.then(() => true)"));
      // Un troisième appareil, avec JavaScript : le panier du compte y est.
      await newDevice();
      await q.open(`/account/signin?next=${lesson}`, 300);
      await signIn();
      const third = await cart();
      // Se déconnecter : cet appareil repart d'un panier vide ; le compte garde le sien.
      await q.open("/account", 300);
      await q.click('form[action="/account/signout"] button');
      await after(`location.search === "?done=signedout"`);
      await q.open(lesson, 300);
      const out = await cart();
      const ok = phone === "2" && named && anonymous === "0" && computer === "2" && counted && mirrored && third === "3" && out === "0";
      return [ok, `téléphone, sans JavaScript : ${phone} (nommé : ${named}) ; ordinateur, pas connecté : ${anonymous}, connecté : ${computer} ; avec JavaScript, compté : ${counted}, renvoyé : ${mirrored} ; un troisième appareil : ${third} ; déconnecté : ${out}`];
    } finally {
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      served.stop();
    }
  }],
  ["un membre : le toucher renvoyé porte les champs et l'adresse d'avant le toucher (compte, serve)", async (_, b) => {
    // ADR-081 avec l'ADR-091 : le serveur rejoue le toucher d'un membre avec ce que la page avait
    // AVANT de le jouer. Une règle qui vide le champ (`text.set("")`) ne fait pas ranger une note
    // vide ; une valeur de l'adresse n'est pas comptée deux fois (`seen` vaut 3, pas 4 ni 2).
    const served = await startHoloServe([]);
    writeFileSync(join(served.folder, "notes.holo"), `Page(title: "Les notes de {account}", access: members, state: State(text: "", page: 1, seen: 0, tasks: []), address: [page], children: [
  Input(value: text, label: "Note"),
  Button(name: Add, text: "Ajouter"),
  Button(name: Next, text: "Page suivante"),
  P("Page {page}, vue {seen}"),
  Repeat(over: tasks, children: [ Text("Note : {item}") ]),
], rules: [ On(Add.tap, effect: [tasks.push(text), text.set("")]), On(Next.tap, effect: [page.add(1), seen.set(page)]) ])
`);
    const q = page(b, served.base);
    const after = (expression, timeout = 10000) => q.until(`document.readyState === "complete" && (${expression})`, timeout);
    const password = "une phrase que je connais";
    try {
      await b.send("Network.clearBrowserCookies");
      await q.open("/account/signup?next=/notes.holo", 300);
      await q.type("#name", "Dora");
      await q.type("#password", password);
      await q.type("#again", password);
      await q.click('main form button[type="submit"]');
      await after(`location.pathname === "/notes.holo"`);
      const titled = await q.value("document.title");
      // Le moteur d'abord, puis une note, puis deux pages.
      await q.click("#toggle");
      const started = await q.until("window.__holoStarted === true", 40000);
      await q.click("#toggle");
      await q.type('[data-bind="text"]', "Laver le pinceau");
      await q.click('[data-name="Add"]');
      await q.until(`document.getElementById("page").innerText.includes("Note : Laver le pinceau")`, 10000);
      await q.click('[data-name="Next"]');
      await q.until(`location.search === "?page=2"`, 10000);
      await q.click('[data-name="Next"]');
      const local = await q.until(`location.search === "?page=3" && document.getElementById("page").innerText.includes("vue 3")`, 10000);
      const sent = await q.until("window.__holoMirrored", 5000) && (await q.value("window.__holoMirrored.then(() => true)"));
      // Un autre appareil : ce que le compte garde.
      await b.send("Network.clearBrowserCookies");
      await q.open("/account/signin?next=/notes.holo", 300);
      await q.type("#name", "Dora");
      await q.type("#password", password);
      await q.click('main form button[type="submit"]');
      await after(`location.pathname === "/notes.holo"`);
      const elsewhere = await q.text();
      const ok = titled === "Les notes de Dora" && started && local && sent && elsewhere.includes("Note : Laver le pinceau") && (elsewhere.match(/Note :/g) ?? []).length === 1 && elsewhere.includes("vue 3");
      return [ok, `titre : « ${titled} » ; moteur : ${started} ; dans la page : page 3, vue 3 : ${local} ; renvoyé : ${sent} ; sur un autre appareil : ${elsewhere.replace(/\s+/g, " ").slice(0, 160)}`];
    } finally {
      await b.send("Network.clearBrowserCookies");
      served.stop();
    }
  }],
  ["une valeur partagée change en direct dans deux pages, avec le serveur d'essai (partage, leçon 101)", async (p, b) => {
    // ADR-079 : deux onglets ouverts à la même adresse ; un toucher dans l'un, l'autre change en
    // direct, sans recharger. Le serveur arbitre, chacun son tour ; une page fermée cesse d'écouter.
    // Avec --telephone, il faudrait ouvrir un autre onglet sur le téléphone de Yocthan : non.
    if (phone) return [true, "sauté avec --telephone : il ouvrirait un autre onglet sur le téléphone"];
    const path = "/exemples/lecons/101-une-valeur-partagee.holo";
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    const shown = (q, name) => q.value(`Number(document.querySelector('#page [data-state="${name}"]')?.textContent ?? NaN)`);
    const other = await b.tab();
    const q = page(other, server.base);
    try {
      await p.open(path, 300);
      await q.open(path, 300);
      const listening = await p.until("window.__holoLive?.()", 40000) && await q.until("window.__holoLive?.()", 40000);
      check("les deux pages écoutent", listening, `${await p.value("window.__holoLive?.()")} et ${await q.value("window.__holoLive?.()")}`);
      const [seats, likes] = [await shown(p, "seats"), await shown(p, "likes")];
      check("les mêmes valeurs des deux côtés", seats === (await shown(q, "seats")) && likes === (await shown(q, "likes")) && seats > 0, `${seats} et ${await shown(q, "seats")}`);
      await q.value("window.__stayed = true");
      // Réserver dans le premier onglet : le second voit la place partir.
      await p.click('[data-name="Book"]');
      const booked = await p.until(`document.getElementById("page").innerText.includes("Ta place est gardée") && Number(document.querySelector('#page [data-state="seats"]').textContent) === ${seats - 1}`, 10000);
      const followed = await q.until(`Number(document.querySelector('#page [data-state="seats"]').textContent) === ${seats - 1}`, 10000);
      check("réserver, vu en direct dans l'autre page", booked && followed && (await q.value("window.__stayed === true")), `réservé : ${booked} ; l'autre page : ${await shown(q, "seats")} (attendu ${seats - 1}), sans recharger : ${await q.value("window.__stayed === true")}`);
      // « J'aime » touché trois fois d'un côté et deux fois de l'autre, sans attendre : aucun perdu.
      for (const [r, times] of [[q, 3], [p, 2]]) for (let i = 0; i < times; i++) await r.click('[data-name="Like"]');
      const counted = await p.until(`Number(document.querySelector('#page [data-state="likes"]').textContent) === ${likes + 5}`, 15000) && await q.until(`Number(document.querySelector('#page [data-state="likes"]').textContent) === ${likes + 5}`, 15000);
      check("cinq « J'aime » des deux côtés, aucun perdu", counted, `${await shown(p, "likes")} et ${await shown(q, "likes")} (attendu ${likes + 5})`);
      // Le serveur arbitre : un geste qui n'est pas un toucher, ou un bouton caché pour l'état envoyé.
      const hover = await q.value(`fetch(location.pathname, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ signal: "Like.hover", state: "" }) }).then((r) => r.status)`);
      const hidden = await q.value(`fetch(location.pathname, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ signal: "Book.tap", state: "booked=1" }) }).then((r) => r.status)`);
      check("un survol, un bouton caché : refusés", hover === 400 && hidden === 409, `survol : ${hover} ; bouton caché : ${hidden}`);
      // Une page fermée cesse d'écouter ; l'autre suit encore.
      await other.close();
      const quiet = await listeners(server.log, "101-une-valeur-partagee.holo", 1, 8000);
      check("l'onglet fermé n'écoute plus", phone || quiet, lastListening(server.log(), "101-une-valeur-partagee.holo"));
      await p.click('[data-name="Cancel"]');
      const back = await p.until(`Number(document.querySelector('#page [data-state="seats"]').textContent) === ${seats}`, 10000);
      check("rendre la place", back, `${await shown(p, "seats")} (attendu ${seats})`);
      // Sans réponse du serveur, rien ne change, et la page le dit.
      await p.value(`window.__fetch = window.fetch; window.fetch = () => Promise.reject(new Error("hors ligne"))`);
      const likesNow = await shown(p, "likes");
      await p.click('[data-name="Like"]');
      const told = await p.until(`!document.getElementById("holo-note")?.hidden && document.getElementById("holo-note").textContent.includes("n'a pas répondu")`, 5000);
      await p.value(`window.fetch = window.__fetch`);
      check("sans réponse, rien ne change, et la page le dit", told && (await shown(p, "likes")) === likesNow, `message : ${await p.value(`document.getElementById("holo-note")?.textContent ?? "(aucun)"`)} ; J'aime : ${await shown(p, "likes")} (avant : ${likesNow})`);
      check("aucune erreur", b.errors.length === 0 && other.errors.length === 0, [...b.errors, ...other.errors].join(" | "));
    } finally {
      await other.close();
    }
    return [faults.length === 0, faults.length ? faults.join("\n      ") : "deux onglets à la même adresse : réserver vu en direct, sans recharger ; cinq « J'aime » des deux côtés, aucun perdu ; un survol et un bouton caché refusés ; l'onglet fermé n'écoute plus ; la place rendue ; sans réponse du serveur, rien ne change et la page le dit"];
  }],
  ["holo serve : une valeur partagée en direct, avec et sans JavaScript (partage, serve)", async (_, b) => {
    // ADR-079 : la même chose avec holo serve, sa base, et un troisième onglet sans JavaScript.
    if (phone) return [true, "sauté avec --telephone : il ouvrirait d'autres onglets sur le téléphone"];
    const served = await startHoloServe(["101-une-valeur-partagee.holo"]);
    writeFileSync(join(served.folder, "places.holo"), [
      'Page(title: "Places", state: State(booked: 0), shared: Shared(seats: 1, likes: 0), children: [',
      '  H1("Une place"),',
      '  P("Places : {seats}"),',
      '  If(seats, over: 0, children: [ Button(name: Book, text: "Réserver") ], else: [ P("Complet") ]),',
      '  Button(name: Like, text: "J\'aime ({likes})"),',
      '], rules: [ On(Book.tap, effect: [seats.sub(1), booked.set(1)]), On(Like.tap, effect: likes.add(1)) ])',
    ].join("\n"));
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    const tabs = [await b.tab(), await b.tab(), await b.tab()];
    const [a, c, without] = tabs.map((tab) => page(tab, served.base));
    const shown = (q, name) => q.value(`Number(document.querySelector('#page [data-state="${name}"]')?.textContent ?? NaN)`);
    try {
      await a.open("/places.holo", 300);
      await c.open("/places.holo", 300);
      const listening = await a.until("window.__holoLive?.()", 40000) && await c.until("window.__holoLive?.()", 40000);
      check("deux pages écoutent", listening, served.log().trim().split("\n").slice(-3).join(" | "));
      await a.value("window.__stayed = true");
      await c.value("window.__stayed = true");
      // Sans JavaScript, « J'aime » part par le formulaire des gestes : les deux autres le voient en direct.
      await tabs[2].send("Emulation.setScriptExecutionDisabled", { value: true });
      await without.open("/places.holo", 300);
      await without.click('[data-name="Like"]');
      const reloaded = await without.until(`document.readyState === "complete" && document.querySelector('#page [data-state="likes"]')?.textContent === "1"`, 8000);
      const live = await a.until(`document.querySelector('#page [data-state="likes"]').textContent === "1"`, 10000) && await c.until(`document.querySelector('#page [data-state="likes"]').textContent === "1"`, 10000);
      check("sans JavaScript, « J'aime » vu en direct ailleurs", reloaded && live && (await a.value("window.__stayed === true")), `page sans JavaScript : ${await shown(without, "likes")} ; les deux autres : ${await shown(a, "likes")}, ${await shown(c, "likes")}`);
      // La dernière place, réservée dans un onglet : « Complet » partout, en direct.
      await a.click('[data-name="Book"]');
      const full = await c.until(`document.querySelector('#page [data-state="seats"]').textContent === "0" && !document.querySelector('[data-name="Book"]')?.getClientRects().length`, 10000);
      check("la dernière place : complet en direct", full && (await shown(a, "seats")) === 0 && (await c.value("window.__stayed === true")), `ici : ${await shown(a, "seats")} ; l'autre : ${await shown(c, "seats")}`);
      // Un geste forgé pour une place qui n'existe plus : refusé, rien ne change.
      const forged = await c.value(`fetch("/places.holo", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ signal: "Book.tap", state: "" }) }).then(async (r) => r.status + " " + (await r.json()).shared)`);
      check("une place forgée, refusée", forged === "409 seats=0;likes=1", forged);
      // Sans JavaScript, la page revient à jour : complet.
      await without.open("/places.holo", 300);
      check("sans JavaScript, la page à jour", (await without.text()).includes("Complet") && (await shown(without, "seats")) === 0, await without.text());
      // Un onglet fermé est oublié par holo serve à son prochain envoi (ou au battement suivant).
      await tabs[1].close();
      let forgotten = false;
      for (let i = 0; i < 4 && !forgotten; i++) {
        await a.click('[data-name="Like"]');
        forgotten = await listeners(served.log, "/places.holo", 1, 2500);
      }
      check("l'onglet fermé n'écoute plus", forgotten, lastListening(served.log(), "/places.holo"));
      // La leçon 101, servie par holo serve, se lit sans erreur.
      await a.open("/101-une-valeur-partagee.holo", 300);
      const lesson = await a.until("window.__holoLive?.()", 40000) && (await shown(a, "seats")) === 20;
      check("la leçon 101 par holo serve", lesson, `${await shown(a, "seats")} places`);
      // Dans la base : l'adresse, le nom, la valeur.
      let stored = "base non lue (node:sqlite absent)";
      try {
        const { DatabaseSync } = await import("node:sqlite");
        const base = new DatabaseSync(join(served.folder, "holo-data", "site.sqlite"));
        stored = base.prepare("SELECT page, name, value FROM shared ORDER BY name").all().map((row) => `${row.page} ${row.name}=${row.value}`).join(", ");
        base.close();
      } catch (error) {
        if (!String(error).includes("node:sqlite")) stored = `base illisible : ${error.message}`;
      }
      check("rangé dans la base", stored.startsWith("base non lue") || /^\/places\.holo likes=\d+, \/places\.holo seats=0$/.test(stored), stored);
      check("aucune erreur", tabs.every((tab) => tab.errors.length === 0), tabs.flatMap((tab) => tab.errors).join(" | "));
    } finally {
      for (const tab of tabs) await tab.close();
      served.stop();
    }
    return [faults.length === 0, faults.length ? faults.join("\n      ") : "sans JavaScript, « J'aime » vu en direct dans deux autres onglets ; la dernière place : complet partout, en direct ; une place forgée refusée (409) ; la page sans JavaScript à jour ; l'onglet fermé oublié ; la leçon 101 ; la base"];
  }],
  ["holo serve : le compte garde son état contre une réservation forgée (compte, partage, serve)", async (_, b) => {
    if (phone) return [true, "sauté avec --telephone : demande un serveur isolé et un compte d'essai"];
    const served = await startHoloServe([]);
    writeFileSync(join(served.folder, "concert.holo"), readFileSync(join(repo, "exemples", ".essais-navigateur", "concert-des-membres.holo")));
    const q = page(b, served.base);
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    try {
      await b.send("Network.clearBrowserCookies");
      // Une réservation sans JavaScript précède l'attaque JSON du même compte.
      await b.send("Emulation.setScriptExecutionDisabled", { value: true });
      await q.open("/account/signup?next=/concert.holo", 300);
      await q.type("#name", "Ada");
      await q.type("#password", "une phrase assez longue");
      await q.type("#again", "une phrase assez longue");
      await q.click('main form button[type="submit"]');
      check("le compte créé", await q.until(`location.pathname === "/concert.holo" && document.readyState === "complete"`, 10000), await q.text());
      await q.type('[data-bind="note"]', "Ada");
      await q.click('[data-name="Book"]');
      check("la première réservation", await q.until(`document.querySelector('#page [data-state="seats"]')?.textContent === "2"`, 10000), await q.text());
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      await q.open("/concert.holo", 300);
      check("le moteur écoute", await q.until("window.__holoLive?.()", 40000), served.log());
      const refused = await q.value(`fetch(location.pathname, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ signal: "Book.tap", state: "booked=0;cart=777;note='Eve;seats=999" }) }).then(async (r) => ({status:r.status, ...await r.json()}))`);
      check("la seconde réservation forgée refusée", refused.status === 409 && refused.accepted === false && refused.shared === "seats=2;likes=0;last='Ada", JSON.stringify(refused));
      check("la réponse garde le compte", refused.state.split(";").includes("booked=1") && refused.state.split(";").includes("cart=0") && refused.state.split(";").includes("note='Ada"), refused.state);
      // Recharger vérifie la conservation en base, et pas seulement la réponse.
      await q.open("/concert.holo", 300);
      check("le refus n'a rien enregistré", (await q.value(`document.querySelector('[data-bind="note"]').value`)) === "Ada" && (await q.value(`document.querySelector('#page [data-state="cart"]').textContent`)) === "0", await q.text());
      await q.until("window.__holoLive?.()", 40000);
      await q.value(`document.querySelector('[data-bind="note"]').select()`);
      await q.type('[data-bind="note"]', "Grace");
      await q.click('[data-name="Like"]');
      check("un geste normal reste utilisable", await q.until(`document.querySelector('#page [data-state="likes"]')?.textContent === "1"`, 10000), await q.text());
      await q.open("/concert.holo", 300);
      const savedNote = await q.value(`document.querySelector('[data-bind="note"]').value`);
      check("la saisie acceptée est gardée", savedNote === "Grace", savedNote);
      await q.until("window.__holoLive?.()", 40000);
      // Le miroir est tenu volontairement ; le toucher partagé suivant doit attendre.
      await q.value(`window.__wire = []; window.__realFetch = window.fetch; window.fetch = async (url, options) => { const mirror = String(url).includes("?mirror"); if (mirror) await new Promise((resolve) => { window.__releaseMirror = resolve; }); window.__wire.push(mirror ? "mirror" : "shared"); return window.__realFetch(url, options); }`);
      await q.click('[data-name="Add"]');
      check("le miroir est en attente", await q.until('typeof window.__releaseMirror === "function"', 5000), await q.value("window.__wire"));
      await q.click('[data-name="Like"]');
      await pause(300);
      check("le geste partagé attend le miroir précédent", (await q.value("window.__wire.length")) === 0, await q.value("window.__wire"));
      await q.value("window.__releaseMirror?.()");
      check("les deux gestes finissent", await q.until(`document.querySelector('#page [data-state="likes"]')?.textContent === "2"`, 10000), await q.text());
      check("leur ordre et le panier sont gardés", (await q.value('window.__wire.join(",")')) === "mirror,shared" && (await q.value(`document.querySelector('#page [data-state="cart"]').textContent`)) === "1", await q.value("window.__wire"));
      await q.value("window.fetch = window.__realFetch");
      await q.open("/concert.holo", 300);
      check("le panier est enregistré", (await q.value(`document.querySelector('#page [data-state="cart"]').textContent`)) === "1", await q.text());
    } finally {
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      await b.send("Network.clearBrowserCookies");
      served.stop();
    }
    return [faults.length === 0, faults.length ? faults.join("\n      ") : "réservation sans JavaScript ; seconde forgée refusée (409) ; cart=777 et note=Eve ignorés ; état du compte intact après rechargement ; toucher et saisie normaux gardés ; miroir retardé : ordre et panier gardés"];
  }],
  ["holo serve : une page hors-ligne sans JavaScript ; le service worker laisse le direct et les écritures (leçon 119, hors-ligne, serve)", async (_, b) => {
    // ADR-096 : sans JavaScript, la page qui offre une copie hors-ligne reste une page comme les
    // autres. Une fois la copie prête, le service worker tient le site ; le direct des valeurs
    // partagées (text/event-stream, ADR-079) et les écritures (POST) passent à côté de lui.
    if (phone) return [true, "sauté avec --telephone : il installerait un service worker sur le téléphone"];
    const served = await startHoloServe(["119-une-page-hors-ligne.holo", "101-une-valeur-partagee.holo"]);
    const faults = [];
    const check = (name, ok, seen) => { if (!ok) faults.push(`${name} : ${seen}`); };
    const tab = await b.tab();
    const q = page(tab, served.base);
    const count = () => q.value(`document.querySelector('#page [data-state="count"]')?.textContent ?? ""`);
    const status = `(document.querySelector('[data-name="Copy"] [data-capability-status]')?.textContent ?? "")`;
    try {
      // Sans JavaScript : deux touchers, par le formulaire des gestes, gardés au rechargement.
      await tab.send("Emulation.setScriptExecutionDisabled", { value: true });
      await q.open("/119-une-page-hors-ligne.holo", 300);
      for (const expected of ["1", "2"]) {
        await q.click('[data-name="Add"]');
        await q.until(`document.readyState === "complete" && document.querySelector('#page [data-state="count"]')?.textContent === "${expected}"`, 8000);
      }
      await q.open("/119-une-page-hors-ligne.holo", 300);
      check("sans JavaScript, deux touchers gardés", (await count()) === "2", `compteur : ${await count()}`);
      // Avec JavaScript, le moteur repart des mêmes valeurs ; la copie se prépare.
      await tab.send("Emulation.setScriptExecutionDisabled", { value: false });
      await q.open("/119-une-page-hors-ligne.holo", 300);
      const started = await q.until("window.__holoStarted", 40000);
      check("avec JavaScript, les mêmes valeurs", started && (await count()) === "2", `compteur : ${await count()}`);
      await q.click('[data-name="Save"]');
      const ready = await q.until(`${status}.includes("prête")`, 20000);
      check("la copie prête", ready, await q.value(status));
      const copy = await q.value(`caches.keys().then(async (keys) => { for (const key of keys.filter((k) => k.startsWith("holo-offline-v1-"))) { const saved = await (await caches.open(key)).match(location.origin + "/119-une-page-hors-ligne.holo"); if (saved) return saved.text(); } return ""; })`);
      check("la copie, celle d'un premier visiteur", copy.includes('Compteur local : <span data-state="count">0</span>'), copy ? "d'autres valeurs" : "pas de copie");
      // Sous le service worker : la page passe par lui ; le direct et un geste partagé, non.
      const seen = new Map();
      tab.on("Network.requestWillBeSent", ({ requestId, request, type }) => seen.set(requestId, { method: request.method, type, url: request.url }));
      tab.on("Network.responseReceived", ({ requestId, response }) => { const s = seen.get(requestId); if (s) s.worker = response.fromServiceWorker; });
      await tab.send("Network.enable");
      await q.open("/101-une-valeur-partagee.holo", 300);
      const controlled = await q.until("navigator.serviceWorker.controller !== null", 10000);
      const listening = await q.until("window.__holoLive?.()", 40000);
      await q.click('[data-name="Like"]');
      const liked = await q.until(`document.querySelector('#page [data-state="likes"]')?.textContent === "1"`, 10000);
      await pause(300);
      const requests = [...seen.values()];
      const shown = requests.find((r) => r.type === "Document" && r.url.endsWith("/101-une-valeur-partagee.holo"));
      const live = requests.filter((r) => r.type === "EventSource");
      const writes = requests.filter((r) => r.method === "POST");
      check("la page servie par le service worker", controlled && shown?.worker === true, JSON.stringify(shown));
      check("le direct passe à côté", listening && live.length > 0 && live.every((r) => r.worker === false), JSON.stringify(live));
      check("le geste partagé passe à côté", liked && writes.length > 0 && writes.every((r) => r.worker === false), JSON.stringify(writes));
      check("aucune erreur", tab.errors.length === 0, tab.errors.join(" | "));
    } finally {
      tab.on("Network.requestWillBeSent", null);
      tab.on("Network.responseReceived", null);
      await q.value(`navigator.serviceWorker.getRegistrations().then((all) => Promise.all(all.map((r) => r.unregister()))).then(() => caches.keys()).then((keys) => Promise.all(keys.map((k) => caches.delete(k)))).then(() => true)`).catch(() => {});
      await tab.close();
      served.stop();
    }
    return [faults.length === 0, faults.length ? faults.join("\n      ") : "sans JavaScript, deux touchers gardés ; avec, les mêmes valeurs ; la copie prête, avec celles d'un premier visiteur ; sous le service worker, la page passe par lui, le direct et le geste partagé à côté"];
  }],
  ["des filtres d'image : gris, puis les couleurs sous la souris ; assombri, vif, flou ; sans filtre au focus du clavier ; la page floue derrière une fenêtre ; sans JavaScript aussi (leçon 131)", async (p, b) => {
    // ADR-108 : un réglage par effet, composés par le moteur en un seul `filter`, sur une image,
    // une forme ou un dessin ; au focus du clavier, le bloc se montre sans filtre ; `backdrop-blur`
    // sur une fenêtre floute la page derrière elle (son `::backdrop`).
    const lesson = "/exemples/lecons/131-des-filtres-d-image.holo";
    const filter = (q, selector) => q.value(`(() => { const e = document.querySelector(${JSON.stringify(selector)}); return e ? getComputedStyle(e).filter : "absent"; })()`);
    await p.open(lesson);
    const gray = await filter(p, ".holo-s-gris");
    const dim = await filter(p, ".holo-s-sombre");
    const vivid = await filter(p, ".holo-s-vive");
    const blurred = await filter(p, ".holo-s-floue");
    // Le passage d'une allure à l'autre couvre aussi le filtre.
    const eased = await p.value(`document.querySelector(".holo-s-gris") ? getComputedStyle(document.querySelector(".holo-s-gris")).transitionProperty : "absent"`);
    // La souris se pose sur l'image grise : elle reprend ses couleurs, en douceur.
    const box = await p.value(`(() => { const e = document.querySelector(".holo-s-gris"); if (!e) return null; const r = e.getBoundingClientRect(); return [r.left + r.width / 2, r.top + r.height / 2]; })()`);
    // Le survol n'existe qu'avec une souris (`@media (hover:hover)`, ADR-036). Chrome sans écran,
    // sous Linux (ce conteneur, les machines de GitHub), dit n'en avoir aucune, et le protocole ne
    // sait pas lui en donner une (`Emulation.setEmulatedMedia` n'y change rien) : sans souris, on
    // lit la règle du survol dans la feuille de style de la page.
    const mouse = await p.value(`matchMedia("(hover: hover)").matches`);
    let colored = false;
    if (box && mouse) {
      await b.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: box[0], y: box[1] });
      colored = await p.until(`getComputedStyle(document.querySelector(".holo-s-gris")).filter === "grayscale(0)"`, 5000);
      await b.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: 2, y: 2 });
      await p.until(`getComputedStyle(document.querySelector(".holo-s-gris")).filter === "grayscale(1)"`, 5000);
    } else if (box) {
      colored = await p.value(`[...document.styleSheets].some((sheet) => [...sheet.cssRules].some((rule) => rule.media?.mediaText === "(hover: hover)" && [...rule.cssRules].some((inner) => inner.selectorText === ".holo-s-gris:hover" && inner.style.filter === "grayscale(0)")))`);
    }
    // Au clavier : l'image grise, rendue atteignable comme une forme qu'on touche, reçoit le focus
    // par Tab ; elle se montre alors sans filtre, son cadre de focus net. La souris est partie.
    let focused = "pas atteinte";
    if (box) {
      await p.value(`(() => { const e = document.querySelector(".holo-s-gris"); e.tabIndex = 0; document.activeElement?.blur(); return true; })()`);
      for (let i = 0; i < 40 && !(await p.value(`document.activeElement === document.querySelector(".holo-s-gris")`)); i++) await p.key("Tab", "Tab", 9);
      if (await p.value(`document.activeElement === document.querySelector(".holo-s-gris")`)) {
        await p.until(`getComputedStyle(document.querySelector(".holo-s-gris")).filter === "none"`, 3000);
        focused = await p.value(`(() => { const e = document.querySelector(".holo-s-gris"); return (e.matches(":focus-visible") ? "focus-visible, " : "focus sans cadre, ") + getComputedStyle(e).filter; })()`);
      }
    }
    // La fenêtre s'ouvre : derrière elle, la page est floue.
    let opened = false;
    let behind = "absent";
    if (await p.value(`!!document.querySelector('[data-name="Ouvrir"]')`)) {
      await p.click('[data-name="Ouvrir"]');
      opened = await p.until(`!!document.querySelector("dialog[open]")`, 10000);
      behind = await p.value(`(() => { const d = document.querySelector("dialog.holo-Dialog"); return d ? getComputedStyle(d, "::backdrop").backdropFilter : "absent"; })()`);
    }
    // Sans JavaScript : la page fabriquée par le serveur porte les mêmes filtres.
    let withoutScript = "absent";
    try {
      await b.send("Emulation.setScriptExecutionDisabled", { value: true });
      await p.open(lesson, 300);
      withoutScript = `${await filter(p, ".holo-s-gris")} | ${await filter(p, ".holo-s-sombre")} | ${await filter(p, ".holo-s-floue")}`;
    } finally {
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
    }
    const ok = gray === "grayscale(1)" && colored && dim === "brightness(0.6) contrast(1.2)" && vivid === "saturate(1.8) hue-rotate(30deg)" && blurred === "blur(3px)"
      && eased.includes("filter") && focused === "focus-visible, none" && opened && behind === "blur(6px)"
      && withoutScript === "grayscale(1) | brightness(0.6) contrast(1.2) | blur(3px)";
    return [ok, `gris : ${gray}, sous la souris : ${mouse ? (colored ? "grayscale(0)" : "resté gris") : (colored ? "pas de souris dans ce Chrome, la règle du survol lue : grayscale(0)" : "pas de souris dans ce Chrome, et pas de règle du survol")} ; assombri : ${dim} ; vif : ${vivid} ; flou : ${blurred} ; transition : ${eased} ; au focus du clavier : ${focused} ; fenêtre ${opened ? "ouverte" : "fermée"}, derrière : ${behind} ; sans JavaScript : ${withoutScript}`];
  }],
];

tests.push(...sharingTests({engine,phone,page,startHoloServe,startChrome,pause}));
tests.push(...passkeyTests({engine,phone,page,startHoloServe}));
tests.push(...accountDebtTests({engine,phone,page,startHoloServe,pause,totp,stepNow}));
tests.push(...webTests({ repo, engine, phone, page, startHoloServe, startChrome, pause }));
tests.push(...capabilityTests({engine,phone,pause}));

// Les essais propres au téléphone : seulement avec --telephone.
const capturesFolder = join(repo, "proposals", "Claude", "telephone-2026-10-07", "captures");
if (phone) tests.push(
  ["téléphone : temps de chargement, sans cache", async (p, b) => {
    await b.send("Network.enable");
    await b.send("Network.setCacheDisabled", { cacheDisabled: true });
    const seen = [];
    try {
      for (const path of ["/exemples/lecons/01-page.holo", "/exemples/site-reference/accueil.holo", "/exemples/lecons/84-donnees-arrivees-ou-pas.holo", "/mondes/big-bang.holo"]) {
        await p.open(path, 2500);
        const [ready, loaded, bytes] = await p.value(`(() => { const n = performance.getEntriesByType("navigation")[0]; const all = [n, ...performance.getEntriesByType("resource")]; return [Math.round(n.domContentLoadedEventEnd), Math.round(n.loadEventEnd), all.reduce((s, e) => s + (e.transferSize || 0), 0)]; })()`);
        seen.push(`${path.split("/").pop()} : prête ${ready} ms, chargée ${loaded} ms, ${(bytes / 1000).toFixed(1)} Ko`);
      }
    } finally {
      await b.send("Network.setCacheDisabled", { cacheDisabled: false });
    }
    return [true, seen.join(" ; ")];
  }],
  ["téléphone : une lettre tapée, la liste refaite (leçon 82)", async (p) => {
    await p.open("/exemples/lecons/82-chercher-filtrer-trier.holo");
    await p.until(`window.__holoStarted && document.querySelectorAll(".holo-line").length > 0`);
    const times = [];
    for (const letters of ["r", "ri", "riv", "", "h", "hu"]) {
      times.push(await p.value(`(() => { const i = document.querySelector('input[data-bind="search"]'); const t0 = performance.now(); i.value = ${JSON.stringify(letters)}; i.dispatchEvent(new Event("input", { bubbles: true })); return Math.round((performance.now() - t0) * 10) / 10; })()`));
    }
    const worst = Math.max(...times);
    return [worst < 50, `${times.join(", ")} ms ; la plus lente : ${worst} ms (cible : moins de 50 ms)`];
  }],
  ["téléphone : le champ à virgule et le champ date (leçons 86, 87)", async (p, b) => {
    await p.open("/exemples/lecons/86-nombres-a-virgule.holo");
    await p.until("window.__holoStarted");
    const attributes = await p.value(`(() => { const i = document.querySelector('input[data-bind="price"]'); return [i.type, i.inputMode, i.step, i.value].join(" "); })()`);
    // La virgule, comme sur un clavier français : Chrome la prend-il dans un champ de nombre ?
    await p.value(`(() => { const i = document.querySelector('input[data-bind="price"]'); i.focus(); i.select(); })()`);
    await b.send("Input.insertText", { text: "9,99" });
    await pause(400);
    const comma = await p.value(`[document.querySelector('input[data-bind="price"]').value, (document.getElementById("page").innerText.match(/Prix : [^€]*€/) ?? ["?"])[0]]`);
    await p.open("/exemples/lecons/87-des-dates.holo");
    await p.until("window.__holoStarted");
    const date = await p.value(`(() => { const i = document.querySelector('input[data-bind="arrival"]'); return [i.type, i.min].join(" "); })()`);
    return [attributes.startsWith("number decimal 0.01") && date.startsWith("date "), `champ à virgule : ${attributes} ; « 9,99 » écrit → champ « ${comma[0]} », page « ${comma[1]} » ; champ date : ${date}`];
  }],
  ["téléphone : des captures", async (p, b) => {
    mkdirSync(capturesFolder, { recursive: true });
    const pages = ["01-page", "53-telephone", "80-tailles-qui-suivent", "82-chercher-filtrer-trier", "84-donnees-arrivees-ou-pas", "85-une-cle-pour-chaque-element", "86-nombres-a-virgule", "87-des-dates"];
    for (const name of pages) {
      await p.open(`/exemples/lecons/${name}.holo`, 2500);
      const shot = await b.send("Page.captureScreenshot", { format: "png" });
      writeFileSync(join(capturesFolder, `${name}.png`), Buffer.from(shot.result.data, "base64"));
    }
    return [true, `${pages.length} captures dans proposals/Claude/telephone-2026-10-07/captures/`];
  }],
);

const server = await startServer();
const browser = await startChrome();
const p = page(browser, server.base);
let failures = 0;
const started = Date.now();
try {
  for (const [name, run] of tests) {
    if (only && !name.includes(only)) continue;
    const t0 = Date.now();
    let ok = false;
    let detail = "";
    try {
      [ok, detail] = await run(p, browser);
    } catch (e) {
      detail = String(e?.message ?? e);
    }
    if (!ok) failures += 1;
    console.log(`${ok ? "OK  " : "RATÉ"} ${name} — ${detail} (${((Date.now() - t0) / 1000).toFixed(1)} s)`);
  }
} finally {
  browser.stop();
  server.stop();
}
console.log(`\n${failures ? `${failures} essai(s) raté(s)` : "tous les essais passent"} — ${((Date.now() - started) / 1000).toFixed(0)} s`);
process.exit(failures ? 1 : 0);
