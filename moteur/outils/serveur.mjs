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
import { readFile, stat } from "node:fs/promises";
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
    const html = execFileSync(rendeur, ["html", cheminHolo, dossier], { encoding: "utf8", timeout: 5000, maxBuffer: 4e6 }).trim();
    const titre = /data-title="([^"]*)"/.exec(html)?.[1] || "HoloCode";
    // La langue, la description et l'image de partage de la page (ADR-038), dans l'en-tête :
    // pour les lecteurs d'écran, pour Google, et pour l'aperçu d'un lien partagé.
    const lire = (attribut) => new RegExp(`${attribut}="([^"]*)"`).exec(html.slice(0, 20000))?.[1];
    const [langue, description, image] = [lire("data-lang"), lire("data-description"), lire("data-image")];
    let entete = `<title>${titre}</title><meta property="og:title" content="${titre}">`;
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

createServer(async (req, res) => {
  try {
    let url = decodeURIComponent(new URL(req.url, "http://x").pathname);
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
  for (const ip of ips) console.log(`Sur le téléphone (même Wi-Fi) : http://${ip}:${port}`);
  console.log("WebGPU exige une page sécurisée : sur le téléphone, voir « Tester sur le téléphone » dans moteur/README.md.");
});
