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

import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

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
  for (const file of files) writeFileSync(join(folder, file), readFileSync(join(repo, "exemples", "lecons", file)));
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
  ["un module enfermé rend son nombre", async (p) => {
    await p.open("/exemples/lecons/69-module-enferme.holo");
    await p.click('[data-name="Calculer"]');
    const ok = await p.until(`(window.__holoModules ?? []).some((m) => m.ok && m.output === 5050)`);
    return [ok, ok ? "5 050" : JSON.stringify(await p.value("window.__holoModules ?? null"))];
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
];

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
