// Serveur local pour ouvrir la démo depuis le PC ou depuis un téléphone sur le même Wi-Fi.
//
//     node outils/server.mjs            → http://localhost:8080
//
// Sert web/ à la racine, mondes/ sous /mondes/ et les exemples du dépôt sous /exemples/. Compresse en Brotli ce que le navigateur
// accepte, pour que le poids transféré mesuré par la page soit celui d'un vrai hébergement.
//
// Un fichier .holo s'ouvre directement : quand le navigateur demande son adresse pour l'afficher,
// le serveur répond par la porte d'entrée du moteur, qui va ensuite chercher le fichier lui-même.
// C'est le rôle que tiendra plus tard un navigateur qui sait lire le .holo.

import { createServer } from "node:http";
import { appendFile, mkdir, readFile, readdir, rename, stat, writeFile } from "node:fs/promises";
import { randomBytes } from "node:crypto";
import { extname, join, normalize, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { brotliCompressSync, constants } from "node:zlib";
import { networkInterfaces } from "node:os";
import { existsSync } from "node:fs";
import { execFileSync } from "node:child_process";

const root = fileURLToPath(new URL("..", import.meta.url));
// HOLO_REPO : le dossier du dépôt dont on affiche les fichiers .holo (exemples/ et moteur/mondes/),
// quand ce n'est pas celui où le moteur a été construit. Sert à afficher ce qu'on écrit dans VS Code.
const repo = process.env.HOLO_REPO || fileURLToPath(new URL("../..", import.meta.url));
const examples = join(repo, "exemples") + sep;
const worlds = join(repo, "moteur", "mondes") + sep;
const port = Number(process.env.PORT ?? 8080);
const types = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".wasm": "application/wasm",
  ".holo": "text/plain; charset=utf-8",
  ".json": "application/json",
  ".css": "text/css; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".webp": "image/webp",
  ".woff2": "font/woff2",
  ".woff": "font/woff",
  ".ttf": "font/ttf",
  ".otf": "font/otf",
  ".wav": "audio/wav",
  ".mp4": "video/mp4",
  ".webm": "video/webm",
  ".mp3": "audio/mpeg",
  ".ogg": "audio/ogg",
};
const cache = new Map();

// Le moteur en Rust, compilé pour ce PC (cargo build --release --bin holo). S'il est là, le
// serveur lui fait fabriquer la page avant de l'envoyer : un robot de recherche, ou un
// navigateur qui ne lance pas le moteur, lit quand même le site. S'il n'y est pas, la page
// est fabriquée dans le navigateur, comme avant.
const renderer = ["release", "debug"].flatMap((profile) => ["holo.exe", "holo"].map((name) => join(root, "target", profile, name))).find(existsSync);

function prerenderedPage(template, holoPath, folder) {
  if (!renderer) return template;
  try {
    // L'heure du lieu, pour une page qui la lit (ADR-039) : le moteur la corrige ensuite avec
    // celle de l'appareil du visiteur.
    const d = new Date();
    const HOLO_NOW = [d.getFullYear(), d.getMonth() + 1, d.getDate(), ((d.getDay() + 6) % 7) + 1, d.getHours(), d.getMinutes()].join(",");
    const html = execFileSync(renderer, ["html", holoPath, folder], { encoding: "utf8", timeout: 5000, maxBuffer: 4e6, env: { ...process.env, HOLO_NOW } }).trim();
    const title = /data-title="([^"]*)"/.exec(html)?.[1] || "HoloCode";
    // La langue, la description et l'image de partage de la page (ADR-038), dans l'en-tête :
    // pour les lecteurs d'écran, pour Google, et pour l'aperçu d'un lien partagé.
    const read = (attribute) => new RegExp(`${attribute}="([^"]*)"`).exec(html.slice(0, 20000))?.[1];
    const [language, description, image, icon] = [read("data-lang"), read("data-description"), read("data-image"), read("data-icon")];
    let header = `<title>${title}</title><meta property="og:title" content="${title}">`;
    // La petite image de l'onglet (ADR-042).
    if (icon) header += `<link rel="icon" href="${icon}">`;
    if (description) header += `<meta name="description" content="${description}"><meta property="og:description" content="${description}">`;
    if (image) header += `<meta property="og:image" content="${image}">`;
    let page = template.replace('<div id="page"></div>', () => `<div id="page">${html}</div>`).replace("<title>HoloCode</title>", () => header);
    if (language) page = page.replace('<html lang="fr">', () => `<html lang="${language}">`);
    return page;
  } catch {
    return template; // fichier refusé : la page d'entrée affichera l'erreur du moteur
  }
}

async function file(path) {
  const info = await stat(path);
  const entry = cache.get(path);
  if (entry && entry.mtime === info.mtimeMs) return entry;
  const raw = await readFile(path);
  const br = brotliCompressSync(raw, { params: { [constants.BROTLI_PARAM_QUALITY]: 11 } });
  const result = { raw, br, mtime: info.mtimeMs };
  cache.set(path, result);
  return result;
}

// Pour éprouver la page qui attend (recette de Codex, E02 et E03) : HOLO_ENGINE=lent:5000
// retarde le moteur de cinq secondes ; HOLO_ENGINE=outage le refuse. Sans cette variable, rien.
const [engineMode, engineDelay] = (process.env.HOLO_ENGINE ?? "").split(":");

// L'éditeur (ADR-046) : il peut écrire un fichier .holo sous exemples/, avec la clé que le
// serveur tire au hasard à son démarrage et affiche. Sans la clé, rien ne s'écrit : un autre
// appareil du Wi-Fi peut lire et essayer, pas modifier. Le fichier est d'abord écrit à côté,
// puis mis à sa place d'un coup : jamais de fichier à moitié écrit. Une ancienne version est
// gardée dans editor-backups/ (jamais versionné), au cas où.
const editorKey = process.env.HOLO_KEY || randomBytes(9).toString("base64url");
const backups = join(repo, "editor-backups");
const FILE_MAX = 262144;
async function saveFile(req, res, url) {
  if ((req.headers["x-holo-key"] ?? "") !== editorKey) return respond(res, 403, "la clé de l'éditeur manque ou n'est pas la bonne : ouvre l'éditeur par l'adresse affichée au démarrage du serveur");
  if (!url.startsWith("/exemples/") || !url.endsWith(".holo") || url.includes("..")) return respond(res, 400, "l'éditeur n'écrit que des fichiers .holo sous exemples/");
  const path = join(examples, normalize(url.slice("/exemples/".length)));
  if (!path.startsWith(examples)) return respond(res, 400, "hors du dossier des exemples");
  const chunks = [];
  let size = 0;
  for await (const chunk of req) {
    size += chunk.length;
    if (size > FILE_MAX) return respond(res, 413, `fichier trop long : plus de ${FILE_MAX} octets`);
    chunks.push(chunk);
  }
  const text = Buffer.concat(chunks);
  await mkdir(join(path, ".."), { recursive: true });
  const old = await readFile(path).catch(() => null);
  if (old && !old.equals(text)) {
    await mkdir(backups, { recursive: true });
    const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
    await writeFile(join(backups, `${url.slice(1).replace(/[\\/]/g, "_")}.${timestamp}`), old);
  }
  const temporary = `${path}.${process.pid}.tmp`;
  await writeFile(temporary, text);
  await rename(temporary, path);
  console.log(`Enregistré par l'éditeur : ${url} (${text.length} octets)`);
  announceToStack("list", {});
  return respond(res, 204, "");
}
// Tous les fichiers .holo servis, pour le panneau « Fichiers » de l'éditeur.
async function listHoloFiles() {
  const foundList = [];
  async function walk(folder, prefix) {
    let entries = [];
    try { entries = await readdir(folder, { withFileTypes: true }); } catch { return; }
    for (const e of entries.sort((a, b) => a.name.localeCompare(b.name, "fr"))) {
      if (e.name.startsWith(".") || e.name === "node_modules") continue;
      if (e.isDirectory()) await walk(join(folder, e.name), `${prefix}${e.name}/`);
      else if (e.name.endsWith(".holo")) foundList.push(`${prefix}${e.name}`);
    }
  }
  await walk(examples, "/exemples/");
  await walk(worlds, "/mondes/");
  return foundList;
}

// Les messages envoyés par un formulaire (ADR-042) : rangés dans messages/, à la racine du dépôt,
// un fichier par page, une ligne par message. Ce dossier n'est jamais versionné. C'est Yocthan
// qui les lit ; rien ne part ailleurs.
const messages = join(repo, "messages");
const MESSAGE_MAX = 16384;
const MESSAGES_PER_PAGE_MAX = 5_000_000;
// Les fichiers envoyés par un formulaire (ADR-059) : rangés dans messages/files/<page>/, sous
// un nom tiré au hasard ; le nom donné par le visiteur n'est gardé que dans le message. La sorte
// est lue dans les premiers octets du fichier, jamais dans son nom.
const SUBMISSION_WITH_FILES_MAX = 4 * 10_000_000 + 65536;
const FILES_PER_PAGE_MAX = 500_000_000;
const SIGNATURES = [
  ["image", "png", (o) => o.subarray(0, 8).equals(Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]))],
  ["image", "jpg", (o) => o[0] === 0xff && o[1] === 0xd8 && o[2] === 0xff],
  ["image", "gif", (o) => ["GIF87a", "GIF89a"].includes(o.subarray(0, 6).toString("latin1"))],
  ["image", "webp", (o) => o.subarray(0, 4).toString("latin1") === "RIFF" && o.subarray(8, 12).toString("latin1") === "WEBP"],
  ["pdf", "pdf", (o) => o.subarray(0, 5).toString("latin1") === "%PDF-"],
];
// Lit un envoi en plusieurs morceaux (multipart/form-data) : [{ nom, octets }], ou null s'il est mal formé.
function readMultipart(body, header) {
  const limit = /boundary=(?:"([^"]+)"|([^;]+))/i.exec(header);
  if (!limit) return null;
  const separator = Buffer.from(`--${limit[1] ?? limit[2]}`);
  const chunks = [];
  let start = body.indexOf(separator);
  if (start < 0) return null;
  while (true) {
    start += separator.length;
    if (body.subarray(start, start + 2).toString() === "--") return chunks;
    const end = body.indexOf(separator, start);
    if (end < 0) return null;
    const chunk = body.subarray(start + 2, end - 2); // sans le retour à la ligne de part et d'autre
    const cut = chunk.indexOf("\r\n\r\n");
    if (cut < 0) return null;
    const headers = chunk.subarray(0, cut).toString("utf8");
    const name = /name="([^"]*)"/i.exec(headers)?.[1];
    if (!name) return null;
    chunks.push({ name, bytes: chunk.subarray(cut + 4) });
    if (chunks.length > 64) return null;
    start = end;
  }
}
async function folderSize(folder) {
  let total = 0;
  for (const e of await readdir(folder, { withFileTypes: true }).catch(() => [])) if (e.isFile()) total += (await stat(join(folder, e.name))).size;
  return total;
}
async function receiveMessage(req, res, url, pageHolo) {
  const multipart = /^multipart\/form-data/i.test(req.headers["content-type"] ?? "");
  const chunks = [];
  let size = 0;
  for await (const chunk of req) {
    size += chunk.length;
    if (size > (multipart ? SUBMISSION_WITH_FILES_MAX : MESSAGE_MAX)) return respond(res, 413, "message trop long");
    chunks.push(chunk);
  }
  let body = Buffer.concat(chunks);
  let receivedFiles = [];
  if (multipart) {
    const parts = readMultipart(body, req.headers["content-type"]);
    const values = parts?.find((p) => p.name === "submission");
    if (!values || values.bytes.length > MESSAGE_MAX) return respond(res, 400, "message mal formé");
    body = values.bytes;
    receivedFiles = parts.filter((p) => p !== values && p.bytes.length);
  }
  let submission;
  try { submission = JSON.parse(body.toString("utf8")); } catch { return respond(res, 400, "message illisible"); }
  if (typeof submission?.form !== "string" || typeof submission.values !== "object" || submission.values === null || Array.isArray(submission.values)) return respond(res, 400, "message mal formé");
  const name = url.replace(/^\/+/, "").replace(/\.holo$/, "").replace(/[^A-Za-z0-9_-]+/g, "_");
  if (receivedFiles.length) {
    // Ce que la page permet, demandé au moteur : jamais à ce que dit le navigateur.
    if (!renderer) return respond(res, 501, "le moteur n'est pas construit : ce serveur ne reçoit pas de fichiers");
    let allowed;
    try {
      allowed = execFileSync(renderer, ["files", pageHolo], { encoding: "utf8", timeout: 5000 }).split("\n").filter(Boolean).map((l) => l.split("|"));
    } catch { return respond(res, 400, "page refusée par le moteur"); }
    const folder = join(messages, "files", name);
    let place = FILES_PER_PAGE_MAX - (await folderSize(folder));
    const seenOnes = new Set();
    const ranges = [];
    for (const { name: field, bytes } of receivedFiles) {
      const rule = allowed.find(([formName, value]) => formName === submission.form && value === field);
      if (!rule || seenOnes.has(field)) return respond(res, 400, `aucun champ de fichier « ${field} » dans ce formulaire`);
      seenOnes.add(field);
      const [, , kinds, max] = rule;
      if (bytes.length > Number(max)) return respond(res, 413, "fichier trop lourd");
      const kind = SIGNATURES.find(([s, , recognizes]) => kinds.split(",").includes(s) && recognizes(bytes));
      if (!kind) return respond(res, 415, "sorte de fichier refusée");
      place -= bytes.length;
      if (place < 0) return respond(res, 507, "trop de fichiers gardés pour cette page");
      ranges.push({ field, bytes, extension: kind[1] });
    }
    // Tout est vérifié : on range.
    await mkdir(folder, { recursive: true });
    for (const { field, bytes, extension } of ranges) {
      const file = `${Date.now()}-${randomBytes(6).toString("hex")}.${extension}`;
      await writeFile(join(folder, file), bytes);
      submission.values[field] = { name: String(submission.values[field] ?? "").slice(0, 120), file: `fichiers/${name}/${file}`, size: bytes.length };
    }
  }
  const messagesFile = join(messages, `${name}.jsonl`);
  await mkdir(messages, { recursive: true });
  const already = await stat(messagesFile).then((s) => s.size, () => 0);
  if (already > MESSAGES_PER_PAGE_MAX) return respond(res, 507, "trop de messages gardés pour cette page");
  await appendFile(messagesFile, JSON.stringify({ received: new Date().toISOString(), page: url, form: submission.form, values: submission.values }) + "\n");
  console.log(`Message reçu : ${url} (${submission.form}) → messages/${name}.jsonl${receivedFiles.length ? `, ${receivedFiles.length} fichier(s)` : ""}`);
  return respond(res, 204, "");
}
// La pile (demandée par Yocthan le 2026-10-06) : tout ce qui s'ouvre dans le navigateur, le plus
// récent en haut, pour tout voir dans un seul onglet (web/stack.html). La date est celle du dernier
// commit du fichier ; pour un fichier pas encore versionné, ou changé depuis, celle du disque.
let theGitDates = { when: 0, dates: new Map(), changes: new Set() };
function gitDates() {
  if (Date.now() - theGitDates.when < 30000) return theGitDates;
  const dates = new Map();
  const changes = new Set();
  try {
    const git = (...args) => execFileSync("git", ["-C", repo, "-c", "core.quotepath=false", ...args], { encoding: "utf8", maxBuffer: 64e6, timeout: 10000, stdio: ["ignore", "pipe", "ignore"] });
    let date = "";
    for (const line of git("log", "--format=@%cI", "--name-only", "--", "exemples", "moteur/mondes").split("\n")) {
      if (line.startsWith("@")) date = new Date(line.slice(1)).toISOString();
      else if (line && !dates.has(line)) dates.set(line, date);
    }
    for (const line of git("status", "--porcelain", "--untracked-files=all", "--", "exemples", "moteur/mondes").split("\n")) {
      if (line.length > 3) changes.add(line.slice(3).replace(/^.* -> /, "").replace(/^"|"$/g, ""));
    }
  } catch {
    // Pas de git : la date du disque pour tout.
  }
  return (theGitDates = { when: Date.now(), dates, changes });
}
const KINDS = { lessons: "leçon", game: "jeu", site: "site", "site-reference": "site", zoom: "zoom", motion: "mouvement", "boutique-comparee": "comparaison" };
async function theStack() {
  const { dates, changes } = gitDates();
  const things = [];
  async function walk(folder, prefix, gitPrefix) {
    let entries = [];
    try { entries = await readdir(folder, { withFileTypes: true }); } catch { return; }
    for (const e of entries) {
      if (e.name.startsWith(".") || e.name === "node_modules") continue;
      const full = join(folder, e.name);
      if (e.isDirectory()) {
        await walk(full, `${prefix}${e.name}/`, `${gitPrefix}${e.name}/`);
        continue;
      }
      const genre = extname(e.name);
      if (genre !== ".holo" && genre !== ".html") continue;
      const text = await readFile(full, "utf8").catch(() => "");
      const withoutComments = text.replace(/\/\/.*$/gm, "");
      // Un morceau (Component) ne s'ouvre pas seul : la pile le montre dans l'éditeur.
      const chunk = genre === ".holo" && /^\s*(import\s[^\n]*\n\s*)*Component\s*\(/.test(withoutComments);
      const title = genre === ".holo" ? /title:\s*"([^"]*)"/.exec(withoutComments)?.[1] : /<title>([^<]*)<\/title>/i.exec(text)?.[1];
      const gitPath = gitPrefix + e.name;
      const date = dates.has(gitPath) && !changes.has(gitPath) ? dates.get(gitPath) : (await stat(full)).mtime.toISOString();
      const kind = chunk ? "morceau" : genre === ".html" ? "jumeau en HTML" : prefix.startsWith("/mondes/") ? "monde" : KINDS[prefix.split("/")[2]] ?? "exemple";
      things.push({ path: prefix + e.name, title: (title || e.name).trim(), date, kind, chunk });
    }
  }
  await walk(examples, "/exemples/", "exemples/");
  await walk(worlds, "/mondes/", "moteur/mondes/");
  return things.sort((a, b) => b.date.localeCompare(a.date) || a.path.localeCompare(b.path, "fr"));
}
// Les piles ouvertes écoutent le serveur : quand Claude montre une page (outils/show.mjs),
// elles l'ouvrent elles-mêmes, sans nouvel onglet ni nouvelle fenêtre.
const stacks = new Set();
function listenToStack(req, res) {
  res.writeHead(200, { "content-type": "text/event-stream; charset=utf-8", "cache-control": "no-cache" });
  res.write("retry: 5000\n\n");
  stacks.add(res);
  req.on("close", () => stacks.delete(res));
}
function announceToStack(event, data) {
  for (const stack of stacks) stack.write(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`);
  return stacks.size;
}
async function showInStack(req, res) {
  // Seulement depuis ce PC : un autre appareil du Wi-Fi ne pilote pas la pile.
  if (!/^(::1|127\.0\.0\.1|::ffff:127\.0\.0\.1)$/.test(req.socket.remoteAddress ?? "")) return respond(res, 403, "seul ce PC peut montrer quelque chose dans la pile");
  let body = "";
  for await (const chunk of req) {
    body += chunk;
    if (body.length > 1024) return respond(res, 413, "chemin trop long");
  }
  const path = body.trim();
  if (!/^\/(exemples|mondes)\/.+\.(holo|html)$/.test(path) || path.includes("..")) return respond(res, 400, "un chemin /exemples/….holo, /mondes/….holo ou /exemples/….html");
  const views = announceToStack("show", { path });
  return views ? respond(res, 200, `montré dans ${views === 1 ? "la pile ouverte" : `${views} piles ouvertes`}`) : respond(res, 409, "aucune pile ouverte");
}

function respond(res, code, text) {
  res.writeHead(code, { "content-type": "text/plain; charset=utf-8" });
  res.end(text);
}

createServer(async (req, res) => {
  try {
    let url = decodeURIComponent(new URL(req.url, "http://x").pathname);
    if (req.method === "PUT") return await saveFile(req, res, url);
    if (url === "/liste-holo") {
      res.writeHead(200, { "content-type": "application/json; charset=utf-8", "cache-control": "no-cache" });
      return res.end(JSON.stringify(await listHoloFiles()));
    }
    if (url === "/editor") url = "/editor.html";
    if (url === "/stack") url = "/stack.html";
    if (url === "/stack.json") {
      res.writeHead(200, { "content-type": "application/json; charset=utf-8", "cache-control": "no-cache" });
      return res.end(JSON.stringify(await theStack()));
    }
    if (url === "/stack/listen") return listenToStack(req, res);
    if (url === "/stack/show" && req.method === "POST") return await showInStack(req, res);
    if (req.method === "POST") {
      if (!url.endsWith(".holo") || !/application\/json|multipart\/form-data/.test(req.headers["content-type"] ?? "")) return respond(res, 405, "seul un formulaire d'une page .holo envoie ici");
      // Seulement pour une page qui existe, parmi les exemples servis.
      const pageHolo = url.startsWith("/exemples/") ? join(examples, normalize(url.slice("/exemples/".length))) : "";
      if (!pageHolo.startsWith(examples) || !existsSync(pageHolo)) return respond(res, 404, "page introuvable");
      return await receiveMessage(req, res, url, pageHolo);
    }
    // Le retard ne compte qu'une fois, sur la première pièce du moteur.
    if (url === "/page-engine.js" && engineMode === "slow") await new Promise((r) => setTimeout(r, Number(engineDelay) || 5000));
    if ((url === "/page-engine.js" || url.startsWith("/pkg/") || url.startsWith("/pkg-light/")) && engineMode === "outage") {
      throw Object.assign(new Error("moteur refusé (HOLO_ENGINE=outage)"), { code: "PANNE" });
    }
    if (url === "/") url = "/index.html";
    const inExamples = url.startsWith("/exemples/");
    const inWorlds = url.startsWith("/mondes/");
    const path = inExamples
      ? join(examples, normalize(url.slice("/exemples/".length)))
      : inWorlds ? join(worlds, normalize(url.slice("/mondes/".length))) : join(root, "web", normalize(url));
    if (!path.startsWith(inExamples ? examples : inWorlds ? worlds : root)) throw Object.assign(new Error("hors racine"), { code: "ENOENT" });
    // Un .holo demandé pour être affiché (et non lu par le moteur) : on sert la porte d'entrée,
    // celle des pages ou celle des points selon le premier bloc du fichier.
    const forDisplay = extname(path) === ".holo" && /text\/html/.test(req.headers.accept ?? "");
    let toServe = path;
    if (forDisplay) {
      const source = (await readFile(path, "utf8")).replace(/\/\/.*$/gm, "");
      toServe = join(root, "web", /^\s*Point/.test(source) ? "index.html" : "page.html");
    }
    let { raw, br } = await file(toServe);
    if (forDisplay && toServe.endsWith("page.html")) {
      raw = Buffer.from(prerenderedPage(raw.toString("utf8"), path, url.slice(0, url.lastIndexOf("/") + 1)));
      br = brotliCompressSync(raw, { params: { [constants.BROTLI_PARAM_QUALITY]: 5 } });
    }
    const type = types[extname(toServe)] ?? "application/octet-stream";
    const acceptsBrotli = /\bbr\b/.test(req.headers["accept-encoding"] ?? "");
    res.writeHead(200, {
      "content-type": type,
      "cache-control": "no-cache",
      // Un fichier .holo et ses images sont publics : un site rangé ailleurs peut y mener.
      "access-control-allow-origin": "*",
      ...(acceptsBrotli ? { "content-encoding": "br" } : {}),
    });
    res.end(acceptsBrotli ? br : raw);
  } catch (e) {
    res.writeHead(e.code === "ENOENT" ? 404 : 500, { "content-type": "text/plain; charset=utf-8" });
    res.end(e.code === "ENOENT" ? "introuvable" : String(e));
  }
}).listen(port, "0.0.0.0", () => {
  const fps = Object.values(networkInterfaces()).flat().filter((i) => i.family === "IPv4" && !i.internal).map((i) => i.address);
  console.log(`Fichiers .holo : ${repo}`);
  console.log(renderer ? `Pages fabriquées d'avance par : ${renderer}` : "Pages fabriquées dans le navigateur (pour les fabriquer d'avance : cargo build --release --bin holo)");
  console.log(`Sur ce PC      : http://localhost:${port}`);
  console.log(`L'éditeur      : http://localhost:${port}/editor?key=${editorKey}   (la clé permet d'enregistrer ; sans elle, on lit et on essaie)`);
  console.log(`La pile        : http://localhost:${port}/pile?key=${editorKey}   (tout ce qui a été créé, dans un seul onglet)`);
  for (const ip of fps) console.log(`Sur le téléphone (même Wi-Fi) : http://${ip}:${port}`);
  console.log("WebGPU exige une page sécurisée : sur le téléphone, voir « Tester sur le téléphone » dans moteur/README.md.");
});
