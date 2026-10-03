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

const racine = fileURLToPath(new URL("..", import.meta.url));
// HOLO_DEPOT : le dossier du dépôt dont on affiche les fichiers .holo (exemples/ et moteur/mondes/),
// quand ce n'est pas celui où le moteur a été construit. Sert à afficher ce qu'on écrit dans VS Code.
const depot = process.env.HOLO_DEPOT ?? fileURLToPath(new URL("../..", import.meta.url));
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
};
const cache = new Map();

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

createServer(async (req, res) => {
  try {
    let url = decodeURIComponent(new URL(req.url, "http://x").pathname);
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
    const { brut, br } = await fichier(aServir);
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
  console.log(`Sur ce PC      : http://localhost:${port}`);
  for (const ip of ips) console.log(`Sur le téléphone (même Wi-Fi) : http://${ip}:${port}`);
  console.log("WebGPU exige une page sécurisée : sur le téléphone, voir « Tester sur le téléphone » dans moteur/README.md.");
});
