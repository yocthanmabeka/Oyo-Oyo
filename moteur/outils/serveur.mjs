// Serveur local pour ouvrir la démo depuis le PC ou depuis un téléphone sur le même Wi-Fi.
//
//     node outils/serveur.mjs            → http://localhost:8080
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

const racine = fileURLToPath(new URL("..", import.meta.url));
// HOLO_DEPOT : le dossier du dépôt dont on affiche les fichiers .holo (exemples/ et moteur/mondes/),
// quand ce n'est pas celui où le moteur a été construit. Sert à afficher ce qu'on écrit dans VS Code.
const depot = process.env.HOLO_DEPOT || fileURLToPath(new URL("../..", import.meta.url));
const exemples = join(depot, "exemples") + sep;
const mondes = join(depot, "moteur", "mondes") + sep;
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
const rendeur = ["release", "debug"].flatMap((profil) => ["holo.exe", "holo"].map((nom) => join(racine, "target", profil, nom))).find(existsSync);

function pageToutePrete(gabarit, cheminHolo, dossier) {
  if (!rendeur) return gabarit;
  try {
    // L'heure du lieu, pour une page qui la lit (ADR-039) : le moteur la corrige ensuite avec
    // celle de l'appareil du visiteur.
    const d = new Date();
    const HOLO_MAINTENANT = [d.getFullYear(), d.getMonth() + 1, d.getDate(), ((d.getDay() + 6) % 7) + 1, d.getHours(), d.getMinutes()].join(",");
    const html = execFileSync(rendeur, ["html", cheminHolo, dossier], { encoding: "utf8", timeout: 5000, maxBuffer: 4e6, env: { ...process.env, HOLO_MAINTENANT } }).trim();
    const titre = /data-title="([^"]*)"/.exec(html)?.[1] || "HoloCode";
    // La langue, la description et l'image de partage de la page (ADR-038), dans l'en-tête :
    // pour les lecteurs d'écran, pour Google, et pour l'aperçu d'un lien partagé.
    const lire = (attribut) => new RegExp(`${attribut}="([^"]*)"`).exec(html.slice(0, 20000))?.[1];
    const [langue, description, image, icone] = [lire("data-lang"), lire("data-description"), lire("data-image"), lire("data-icon")];
    let entete = `<title>${titre}</title><meta property="og:title" content="${titre}">`;
    // La petite image de l'onglet (ADR-042).
    if (icone) entete += `<link rel="icon" href="${icone}">`;
    if (description) entete += `<meta name="description" content="${description}"><meta property="og:description" content="${description}">`;
    if (image) entete += `<meta property="og:image" content="${image}">`;
    let page = gabarit.replace('<div id="page"></div>', () => `<div id="page">${html}</div>`).replace("<title>HoloCode</title>", () => entete);
    if (langue) page = page.replace('<html lang="fr">', () => `<html lang="${langue}">`);
    return page;
  } catch {
    return gabarit; // fichier refusé : la page d'entrée affichera l'erreur du moteur
  }
}

async function fichier(chemin) {
  const info = await stat(chemin);
  const entree = cache.get(chemin);
  if (entree && entree.mtime === info.mtimeMs) return entree;
  const brut = await readFile(chemin);
  const br = brotliCompressSync(brut, { params: { [constants.BROTLI_PARAM_QUALITY]: 11 } });
  const resultat = { brut, br, mtime: info.mtimeMs };
  cache.set(chemin, resultat);
  return resultat;
}

// Pour éprouver la page qui attend (recette de Codex, E02 et E03) : HOLO_MOTEUR=lent:5000
// retarde le moteur de cinq secondes ; HOLO_MOTEUR=panne le refuse. Sans cette variable, rien.
const [modeMoteur, retardMoteur] = (process.env.HOLO_MOTEUR ?? "").split(":");

// L'éditeur (ADR-046) : il peut écrire un fichier .holo sous exemples/, avec la clé que le
// serveur tire au hasard à son démarrage et affiche. Sans la clé, rien ne s'écrit : un autre
// appareil du Wi-Fi peut lire et essayer, pas modifier. Le fichier est d'abord écrit à côté,
// puis mis à sa place d'un coup : jamais de fichier à moitié écrit. Une ancienne version est
// gardée dans editeur-sauvegardes/ (jamais versionné), au cas où.
const cleDeLEditeur = process.env.HOLO_CLE || randomBytes(9).toString("base64url");
const sauvegardes = join(depot, "editeur-sauvegardes");
const FICHIER_MAX = 262144;
async function enregistrerUnFichier(req, res, url) {
  if ((req.headers["x-holo-cle"] ?? "") !== cleDeLEditeur) return repondre(res, 403, "la clé de l'éditeur manque ou n'est pas la bonne : ouvre l'éditeur par l'adresse affichée au démarrage du serveur");
  if (!url.startsWith("/exemples/") || !url.endsWith(".holo") || url.includes("..")) return repondre(res, 400, "l'éditeur n'écrit que des fichiers .holo sous exemples/");
  const chemin = join(exemples, normalize(url.slice("/exemples/".length)));
  if (!chemin.startsWith(exemples)) return repondre(res, 400, "hors du dossier des exemples");
  const morceaux = [];
  let taille = 0;
  for await (const morceau of req) {
    taille += morceau.length;
    if (taille > FICHIER_MAX) return repondre(res, 413, `fichier trop long : plus de ${FICHIER_MAX} octets`);
    morceaux.push(morceau);
  }
  const texte = Buffer.concat(morceaux);
  await mkdir(join(chemin, ".."), { recursive: true });
  const ancien = await readFile(chemin).catch(() => null);
  if (ancien && !ancien.equals(texte)) {
    await mkdir(sauvegardes, { recursive: true });
    const horodatage = new Date().toISOString().replace(/[:.]/g, "-");
    await writeFile(join(sauvegardes, `${url.slice(1).replace(/[\\/]/g, "_")}.${horodatage}`), ancien);
  }
  const provisoire = `${chemin}.${process.pid}.tmp`;
  await writeFile(provisoire, texte);
  await rename(provisoire, chemin);
  console.log(`Enregistré par l'éditeur : ${url} (${texte.length} octets)`);
  return repondre(res, 204, "");
}
// Tous les fichiers .holo servis, pour le panneau « Fichiers » de l'éditeur.
async function listerLesHolo() {
  const trouves = [];
  async function parcourir(dossier, prefixe) {
    let entrees = [];
    try { entrees = await readdir(dossier, { withFileTypes: true }); } catch { return; }
    for (const e of entrees.sort((a, b) => a.name.localeCompare(b.name, "fr"))) {
      if (e.name.startsWith(".") || e.name === "node_modules") continue;
      if (e.isDirectory()) await parcourir(join(dossier, e.name), `${prefixe}${e.name}/`);
      else if (e.name.endsWith(".holo")) trouves.push(`${prefixe}${e.name}`);
    }
  }
  await parcourir(exemples, "/exemples/");
  await parcourir(mondes, "/mondes/");
  return trouves;
}

// Les messages envoyés par un formulaire (ADR-042) : rangés dans messages/, à la racine du dépôt,
// un fichier par page, une ligne par message. Ce dossier n'est jamais versionné. C'est Yocthan
// qui les lit ; rien ne part ailleurs.
const messages = join(depot, "messages");
const MESSAGE_MAX = 16384;
const MESSAGES_PAR_PAGE_MAX = 5_000_000;
async function recevoirUnMessage(req, res, url) {
  const morceaux = [];
  let taille = 0;
  for await (const morceau of req) {
    taille += morceau.length;
    if (taille > MESSAGE_MAX) return repondre(res, 413, "message trop long");
    morceaux.push(morceau);
  }
  let envoi;
  try { envoi = JSON.parse(Buffer.concat(morceaux).toString("utf8")); } catch { return repondre(res, 400, "message illisible"); }
  if (typeof envoi?.form !== "string" || typeof envoi.values !== "object" || envoi.values === null || Array.isArray(envoi.values)) return repondre(res, 400, "message mal formé");
  const nom = url.replace(/^\/+/, "").replace(/\.holo$/, "").replace(/[^A-Za-z0-9_-]+/g, "_");
  const fichierDesMessages = join(messages, `${nom}.jsonl`);
  await mkdir(messages, { recursive: true });
  const deja = await stat(fichierDesMessages).then((s) => s.size, () => 0);
  if (deja > MESSAGES_PAR_PAGE_MAX) return repondre(res, 507, "trop de messages gardés pour cette page");
  await appendFile(fichierDesMessages, JSON.stringify({ recu: new Date().toISOString(), page: url, form: envoi.form, values: envoi.values }) + "\n");
  console.log(`Message reçu : ${url} (${envoi.form}) → messages/${nom}.jsonl`);
  return repondre(res, 204, "");
}
function repondre(res, code, texte) {
  res.writeHead(code, { "content-type": "text/plain; charset=utf-8" });
  res.end(texte);
}

createServer(async (req, res) => {
  try {
    let url = decodeURIComponent(new URL(req.url, "http://x").pathname);
    if (req.method === "PUT") return await enregistrerUnFichier(req, res, url);
    if (url === "/liste-holo") {
      res.writeHead(200, { "content-type": "application/json; charset=utf-8", "cache-control": "no-cache" });
      return res.end(JSON.stringify(await listerLesHolo()));
    }
    if (url === "/editeur") url = "/editeur.html";
    if (req.method === "POST") {
      if (!url.endsWith(".holo") || !/application\/json/.test(req.headers["content-type"] ?? "")) return repondre(res, 405, "seul un formulaire d'une page .holo envoie ici");
      // Seulement pour une page qui existe, parmi les exemples servis.
      const pageHolo = url.startsWith("/exemples/") ? join(exemples, normalize(url.slice("/exemples/".length))) : "";
      if (!pageHolo.startsWith(exemples) || !existsSync(pageHolo)) return repondre(res, 404, "page introuvable");
      return await recevoirUnMessage(req, res, url);
    }
    // Le retard ne compte qu'une fois, sur la première pièce du moteur.
    if (url === "/page-moteur.js" && modeMoteur === "lent") await new Promise((r) => setTimeout(r, Number(retardMoteur) || 5000));
    if ((url === "/page-moteur.js" || url.startsWith("/pkg/")) && modeMoteur === "panne") {
      throw Object.assign(new Error("moteur refusé (HOLO_MOTEUR=panne)"), { code: "PANNE" });
    }
    if (url === "/") url = "/index.html";
    const dansExemples = url.startsWith("/exemples/");
    const dansMondes = url.startsWith("/mondes/");
    const chemin = dansExemples
      ? join(exemples, normalize(url.slice("/exemples/".length)))
      : dansMondes ? join(mondes, normalize(url.slice("/mondes/".length))) : join(racine, "web", normalize(url));
    if (!chemin.startsWith(dansExemples ? exemples : dansMondes ? mondes : racine)) throw Object.assign(new Error("hors racine"), { code: "ENOENT" });
    // Un .holo demandé pour être affiché (et non lu par le moteur) : on sert la porte d'entrée,
    // celle des pages ou celle des points selon le premier bloc du fichier.
    const pourAffichage = extname(chemin) === ".holo" && /text\/html/.test(req.headers.accept ?? "");
    let aServir = chemin;
    if (pourAffichage) {
      const source = (await readFile(chemin, "utf8")).replace(/\/\/.*$/gm, "");
      aServir = join(racine, "web", /^\s*Point/.test(source) ? "index.html" : "page.html");
    }
    let { brut, br } = await fichier(aServir);
    if (pourAffichage && aServir.endsWith("page.html")) {
      brut = Buffer.from(pageToutePrete(brut.toString("utf8"), chemin, url.slice(0, url.lastIndexOf("/") + 1)));
      br = brotliCompressSync(brut, { params: { [constants.BROTLI_PARAM_QUALITY]: 5 } });
    }
    const type = types[extname(aServir)] ?? "application/octet-stream";
    const accepteBr = /\bbr\b/.test(req.headers["accept-encoding"] ?? "");
    res.writeHead(200, {
      "content-type": type,
      "cache-control": "no-cache",
      // Un fichier .holo et ses images sont publics : un site rangé ailleurs peut y mener.
      "access-control-allow-origin": "*",
      ...(accepteBr ? { "content-encoding": "br" } : {}),
    });
    res.end(accepteBr ? br : brut);
  } catch (e) {
    res.writeHead(e.code === "ENOENT" ? 404 : 500, { "content-type": "text/plain; charset=utf-8" });
    res.end(e.code === "ENOENT" ? "introuvable" : String(e));
  }
}).listen(port, "0.0.0.0", () => {
  const ips = Object.values(networkInterfaces()).flat().filter((i) => i.family === "IPv4" && !i.internal).map((i) => i.address);
  console.log(`Fichiers .holo : ${depot}`);
  console.log(rendeur ? `Pages fabriquées d'avance par : ${rendeur}` : "Pages fabriquées dans le navigateur (pour les fabriquer d'avance : cargo build --release --bin holo)");
  console.log(`Sur ce PC      : http://localhost:${port}`);
  console.log(`L'éditeur      : http://localhost:${port}/editeur?cle=${cleDeLEditeur}   (la clé permet d'enregistrer ; sans elle, on lit et on essaie)`);
  for (const ip of ips) console.log(`Sur le téléphone (même Wi-Fi) : http://${ip}:${port}`);
  console.log("WebGPU exige une page sécurisée : sur le téléphone, voir « Tester sur le téléphone » dans moteur/README.md.");
});
