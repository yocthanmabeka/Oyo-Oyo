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
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const engine = fileURLToPath(new URL("..", import.meta.url));
const repo = resolve(engine, "..");
const only = process.argv[2] ?? "";
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
  return { base: `http://localhost:${port}`, stop: () => server.kill() };
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
  for (let i = 0; i < 300 && !target && chrome.exitCode === null; i++) {
    await pause(200);
    let port;
    try { port = readFileSync(join(profile, "DevToolsActivePort"), "utf8").split("\n")[0].trim(); } catch { continue; }
    try { target = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((t) => t.type === "page"); } catch { /* pas encore */ }
  }
  if (!target) chrome.kill();
  return { chrome, profile, target, said: said.trim() || "(rien)" };
}

async function startChrome() {
  let launched = await launchChrome();
  // Sur une machine de GitHub qui vient de démarrer, Chrome tarde parfois : un second essai.
  if (!launched.target) {
    console.log(`Chrome ne répond pas ; second essai. Ce qu'il a dit : ${launched.said}`);
    launched = await launchChrome();
  }
  if (!launched.target) throw new Error(`Chrome ne démarre pas. Ce qu'il a dit : ${launched.said}`);
  const { chrome, profile, target } = launched;
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
    stop() {
      ws.close();
      chrome.kill();
      // Chrome garde son dossier un instant après s'être arrêté.
      setTimeout(() => { try { rmSync(profile, { recursive: true, force: true }); } catch { /* tant pis */ } }, 1500);
    },
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
    }
    return [faults.length === 0, faults.length ? faults.join("\n      ") : `${lessons().length} leçons`];
  }],
  ["le moteur arrive au premier geste", async (p) => {
    await p.open("/exemples/lecons/01-page.holo");
    await p.click("#toggle");
    const ok = await p.until(`document.getElementById("tools") && !document.getElementById("tools").hidden`);
    return [ok, ok ? "les outils s'ouvrent" : "les outils ne s'ouvrent pas"];
  }],
  ["pincer à deux doigts grossit la page (pinch)", async (p, b) => {
    await b.send("Emulation.setDeviceMetricsOverride", { width: 400, height: 800, deviceScaleFactor: 2, mobile: true });
    await b.send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
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
      await b.send("Emulation.setTouchEmulationEnabled", { enabled: false });
      await b.send("Emulation.clearDeviceMetricsOverride");
    }
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
];

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
