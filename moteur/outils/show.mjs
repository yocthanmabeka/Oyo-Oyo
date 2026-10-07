// Montre une page à Yocthan dans la pile, sans ouvrir de nouvel onglet ni de nouvelle fenêtre.
//
//     node outils/show.mjs /exemples/lecons/09-zoom-et-points.holo
//
// Si une pile est ouverte (http://localhost:8080/stack), elle affiche la page elle-même. Sinon,
// Chrome s'ouvre une fois, sur la pile, avec cette page déjà choisie. PORT choisit le serveur
// (8080 par défaut) ; HOLO_KEY, la clé de l'éditeur à garder dans l'adresse de la pile.

import { spawn } from "node:child_process";
import { existsSync } from "node:fs";

// Git Bash change « /exemples/… » en « C:/Program Files/Git/exemples/… » : on retrouve le chemin.
const path = (process.argv[2] ?? "").replace(/\\/g, "/").replace(/^.*?(?=\/(exemples|mondes)\/)/, "");
const port = process.env.PORT ?? 8080;
if (path && !/^\/(exemples|mondes)\/.+\.(holo|html)$/.test(path)) {
  console.error("usage : node outils/show.mjs /exemples/dossier/page.holo");
  process.exit(2);
}
if (path) {
  try {
    const response = await fetch(`http://localhost:${port}/stack/show`, { method: "POST", body: path });
    const text = await response.text();
    if (response.ok) {
      console.log(text);
      process.exit(0);
    }
    if (response.status !== 409) {
      console.error(text);
      process.exit(1);
    }
  } catch {
    console.error(`le serveur ne répond pas sur le port ${port} : node outils/server.mjs`);
    process.exit(1);
  }
}
// Aucune pile ouverte : Chrome l'ouvre, dans un onglet de la fenêtre déjà là s'il y en a une.
const key = process.env.HOLO_KEY ? `key=${encodeURIComponent(process.env.HOLO_KEY)}` : "";
const search = [path && `show=${encodeURIComponent(path)}`, key].filter(Boolean).join("&");
const address = `http://localhost:${port}/stack${search ? `?${search}` : ""}`;
const chrome = [process.env.CHROME, "C:/Program Files/Google/Chrome/Application/chrome.exe", "C:/Program Files (x86)/Google/Chrome/Application/chrome.exe"].find((c) => c && existsSync(c));
if (chrome) spawn(chrome, [address], { detached: true, stdio: "ignore" }).unref();
else spawn("cmd", ["/c", "start", "", address.replace(/&/g, "^&")], { detached: true, stdio: "ignore" }).unref();
console.log(`pile ouverte dans Chrome : ${address}`);
