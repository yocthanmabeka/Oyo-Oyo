/**
 * Sondes HTTP des comptes et de l'état partagé. Node.js 20 ou plus récent.
 * Le script démarre SON serveur dans un dossier temporaire ; aucune URL externe acceptée.
 * Les résultats constatés sont écrits en JSON sur la sortie standard.
 */
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { randomUUID } from "node:crypto";
import { copyFile, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const binary = process.argv[2];
if (!binary || Number(process.versions.node.split(".")[0]) < 20) {
  console.error("Usage : node probes.mjs <chemin du binaire holo> (Node.js 20+)");
  process.exit(2);
}
const binaryPath = path.resolve(binary);
const fixturePath = fileURLToPath(new URL("./reservation.holo", import.meta.url));
const folder = await mkdtemp(path.join(os.tmpdir(), "holo-codex-probes-"));
const marker = "Codex " + randomUUID();
const routes = ["protected", "forms", "duplicate", "rejected", "shared"];
let child;
let closed = false;
const rows = [];

async function freePort() {
  const listener = net.createServer();
  await new Promise((resolve, reject) => {
    listener.once("error", reject);
    listener.listen(0, "127.0.0.1", resolve);
  });
  const port = listener.address().port;
  await new Promise((resolve, reject) => listener.close(error => error ? reject(error) : resolve()));
  return port;
}

async function stopServer() {
  if (!child || closed) return;
  await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error("Le serveur ne s'est pas arrêté.")), 8000);
    child.once("close", () => { clearTimeout(timeout); resolve(); });
    child.kill();
  });
}

function numberIn(html, name) {
  assert.match(name, /^[a-zA-Z]+$/);
  const match = html.match(new RegExp('data-state="' + name + '"[^>]*>\\s*(-?\\d+)\\s*<'));
  assert.ok(match, "La page ne présente pas la valeur " + name + ".");
  return Number(match[1]);
}

try {
  for (const route of routes) await copyFile(fixturePath, path.join(folder, route + ".holo"));
  await writeFile(path.join(folder, "index.holo"),
    'Page(title: "Sondes", children: [ H1("' + marker + '") ])\n', "utf8");
  const checked = spawnSync(binaryPath, ["check", path.join(folder, "protected.holo")],
    { encoding: "utf8", windowsHide: true, timeout: 15000 });
  assert.ifError(checked.error);
  assert.equal(checked.status, 0, "Le binaire refuse la fixture : " + checked.stderr);
  const port = await freePort();
  const base = "http://127.0.0.1:" + port;
  child = spawn(binaryPath, ["serve", folder, String(port)], {
    windowsHide: true, stdio: ["ignore", "ignore", "pipe"],
  });
  let startupError;
  let stderr = "";
  child.on("error", error => { startupError = error; closed = true; });
  child.on("close", () => { closed = true; });
  child.stderr.on("data", chunk => { stderr = (stderr + chunk).slice(-3000); });

  async function request(route, cookie = "", { method = "GET", form, json, accept = "text/html" } = {}) {
    const headers = { Accept: accept };
    if (cookie) headers.Cookie = cookie;
    let body;
    if (form !== undefined) {
      headers["Content-Type"] = "application/x-www-form-urlencoded";
      body = new URLSearchParams(form).toString();
    }
    if (json !== undefined) {
      headers["Content-Type"] = "application/json";
      body = JSON.stringify(json);
    }
    if (method === "POST") headers.Origin = base;
    const response = await fetch(new URL(route, base), {
      method, headers, body, redirect: "manual", signal: AbortSignal.timeout(5000),
    });
    return { status: response.status, headers: response.headers, body: await response.text() };
  }

  let ready = false;
  for (let attempt = 0; attempt < 60 && !closed; attempt++) {
    if (startupError) throw startupError;
    try {
      const response = await request("/");
      if (response.status === 200 && response.body.includes(marker)) { ready = true; break; }
      if (response.status === 200) throw new Error("Le port répond depuis un autre site.");
    } catch (error) {
      if (error.message === "Le port répond depuis un autre site.") throw error;
    }
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  assert.ok(ready, "Le serveur de sondes ne démarre pas : " + stderr);

  async function account() {
    const name = "probe" + randomUUID().replaceAll("-", "").slice(0, 12);
    const password = randomUUID();
    const response = await request("/account/signup", "", {
      method: "POST", form: { name, password, again: password },
    });
    assert.equal(response.status, 303, "Le compte de test n'est pas créé.");
    const cookie = response.headers.get("set-cookie")?.match(/\bholo_session=[0-9a-f]{32}\b/)?.[0];
    assert.ok(cookie, "Le serveur ne donne pas de session de compte.");
    return cookie;
  }

  async function page(route, cookie) {
    const response = await request("/" + route + ".holo", cookie);
    assert.equal(response.status, 200, "La page authentifiée ne s'ouvre pas.");
    return response.body;
  }

  async function reserve(route, cookie) {
    const response = await request("/" + route + ".holo", cookie, {
      method: "POST", form: { signal: "Reserve.tap", note: "" },
    });
    assert.equal(response.status, 303, "Le geste ordinaire n'est pas reçu.");
  }

  async function test(id, title, run) {
    try { rows.push({ id, title, passed: true, observed: await run() }); }
    catch (error) { rows.push({ id, title, passed: false, error: error.message }); }
  }

  await test("P01", "Une page réservée ne se lit pas sans session", async () => {
    const html = await request("/protected.holo");
    const raw = await request("/protected.holo", "", { accept: "text/plain" });
    assert.equal(html.status, 303);
    assert.ok(html.headers.get("location")?.startsWith("/account/signin"));
    assert.equal(raw.status, 401);
    assert.ok(!raw.body.includes("Une place par compte"));
    return { html: html.status, source: raw.status };
  });

  await test("P02", "Deux formulaires du même compte ne réservent qu'une place", async () => {
    const cookie = await account();
    await reserve("forms", cookie);
    await reserve("forms", cookie);
    const html = await page("forms", cookie);
    const observed = { seats: numberIn(html, "seats"), booked: numberIn(html, "booked") };
    assert.deepEqual(observed, { seats: 2, booked: 1 });
    return observed;
  });

  await test("P03", "Un état JSON forgé ne permet pas une seconde réservation", async () => {
    const cookie = await account();
    await reserve("duplicate", cookie);
    const response = await request("/duplicate.holo", cookie, {
      method: "POST", json: { signal: "Reserve.tap", state: "booked=0;cart=0;seats=999" },
    });
    assert.equal(response.status, 409, "Le geste du bouton déjà caché doit être refusé.");
    assert.equal(JSON.parse(response.body).accepted, false);
    const html = await page("duplicate", cookie);
    const observed = { seats: numberIn(html, "seats"), booked: numberIn(html, "booked") };
    assert.deepEqual(observed, { seats: 2, booked: 1 });
    return observed;
  });

  await test("P04", "Un geste refusé ne remplace pas l'état gardé du compte", async () => {
    const cookie = await account();
    await reserve("rejected", cookie);
    const response = await request("/rejected.holo", cookie, {
      method: "POST", json: { signal: "Reserve.tap", state: "booked=1;cart=777;seats=999" },
    });
    assert.equal(response.status, 409, "La réservation déjà faite est refusée.");
    assert.equal(JSON.parse(response.body).accepted, false);
    const html = await page("rejected", cookie);
    const observed = { seats: numberIn(html, "seats"), booked: numberIn(html, "booked"), cart: numberIn(html, "cart") };
    assert.deepEqual(observed, { seats: 2, booked: 1, cart: 0 });
    return observed;
  });

  await test("P05", "La valeur partagée du serveur remplace celle du JSON", async () => {
    const cookie = await account();
    const response = await request("/shared.holo", cookie, {
      method: "POST", json: { signal: "Reserve.tap", state: "booked=0;cart=0;seats=999" },
    });
    assert.equal(response.status, 200);
    assert.equal(JSON.parse(response.body).accepted, true);
    const html = await page("shared", cookie);
    const observed = { seats: numberIn(html, "seats"), booked: numberIn(html, "booked") };
    assert.deepEqual(observed, { seats: 2, booked: 1 });
    return observed;
  });

  console.log(JSON.stringify({ node: process.version, results: rows }, null, 2));
  if (rows.some(row => !row.passed)) process.exitCode = 1;
} catch (error) {
  console.error("Préparation impossible : " + error.message);
  process.exitCode = 2;
} finally {
  try {
    await stopServer();
    // On ne supprime que le dossier créé par mkdtemp pour cette exécution.
    const resolvedFolder = path.resolve(folder);
    assert.equal(path.dirname(resolvedFolder), path.resolve(os.tmpdir()));
    assert.ok(path.basename(resolvedFolder).startsWith("holo-codex-probes-"));
    await rm(resolvedFolder, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
  } catch (error) {
    console.error("Nettoyage incomplet ; dossier conservé : " + folder + ". " + error.message);
    process.exitCode = 2;
  }
}
