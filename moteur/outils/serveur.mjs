// Serveur local pour ouvrir la démo depuis le PC ou depuis un téléphone sur le même Wi-Fi.
//
//     node outils/serveur.mjs            → http://localhost:8080
//
// Sert web/ à la racine, mondes/ sous /mondes/ et les exemples du dépôt sous /exemples/. Compresse en Brotli ce que le navigateur
// accepte, pour que le poids transféré mesuré par la page soit celui d'un vrai hébergement.

import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";
import { brotliCompressSync, constants } from "node:zlib";
import { networkInterfaces } from "node:os";

const racine = fileURLToPath(new URL("..", import.meta.url));
const exemples = fileURLToPath(new URL("../../exemples/", import.meta.url));
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
    const chemin = dansExemples
      ? join(exemples, normalize(url.slice("/exemples/".length)))
      : url.startsWith("/mondes/") ? join(racine, normalize(url)) : join(racine, "web", normalize(url));
    if (!chemin.startsWith(dansExemples ? exemples : racine)) throw Object.assign(new Error("hors racine"), { code: "ENOENT" });
    const { brut, br } = await fichier(chemin);
    const type = types[extname(chemin)] ?? "application/octet-stream";
    const accepteBr = /\bbr\b/.test(req.headers["accept-encoding"] ?? "");
    res.writeHead(200, {
      "content-type": type,
      "cache-control": "no-cache",
      ...(accepteBr ? { "content-encoding": "br" } : {}),
    });
    res.end(accepteBr ? br : brut);
  } catch (e) {
    res.writeHead(e.code === "ENOENT" ? 404 : 500, { "content-type": "text/plain; charset=utf-8" });
    res.end(e.code === "ENOENT" ? "introuvable" : String(e));
  }
}).listen(port, "0.0.0.0", () => {
  const ips = Object.values(networkInterfaces()).flat().filter((i) => i.family === "IPv4" && !i.internal).map((i) => i.address);
  console.log(`Sur ce PC      : http://localhost:${port}`);
  for (const ip of ips) console.log(`Sur le téléphone (même Wi-Fi) : http://${ip}:${port}`);
  console.log("WebGPU exige une page sécurisée : sur le téléphone, voir « Tester sur le téléphone » dans moteur/README.md.");
});
