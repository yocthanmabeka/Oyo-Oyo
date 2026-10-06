// Montre une page à Yocthan dans la pile, sans ouvrir de nouvel onglet ni de nouvelle fenêtre.
//
//     node outils/montrer.mjs /exemples/lecons/09-zoom-et-points.holo
//
// Si une pile est ouverte (http://localhost:8080/pile), elle affiche la page elle-même. Sinon,
// Chrome s'ouvre une fois, sur la pile, avec cette page déjà choisie. PORT choisit le serveur
// (8080 par défaut) ; HOLO_CLE, la clé de l'éditeur à garder dans l'adresse de la pile.

import { spawn } from "node:child_process";
import { existsSync } from "node:fs";

// Git Bash change « /exemples/… » en « C:/Program Files/Git/exemples/… » : on retrouve le chemin.
const chemin = (process.argv[2] ?? "").replace(/\\/g, "/").replace(/^.*?(?=\/(exemples|mondes)\/)/, "");
const port = process.env.PORT ?? 8080;
if (chemin && !/^\/(exemples|mondes)\/.+\.(holo|html)$/.test(chemin)) {
  console.error("usage : node outils/montrer.mjs /exemples/dossier/page.holo");
  process.exit(2);
}
if (chemin) {
  try {
    const reponse = await fetch(`http://localhost:${port}/pile/montrer`, { method: "POST", body: chemin });
    const texte = await reponse.text();
    if (reponse.ok) {
      console.log(texte);
      process.exit(0);
    }
    if (reponse.status !== 409) {
      console.error(texte);
      process.exit(1);
    }
  } catch {
    console.error(`le serveur ne répond pas sur le port ${port} : node outils/serveur.mjs`);
    process.exit(1);
  }
}
// Aucune pile ouverte : Chrome l'ouvre, dans un onglet de la fenêtre déjà là s'il y en a une.
const cle = process.env.HOLO_CLE ? `cle=${encodeURIComponent(process.env.HOLO_CLE)}` : "";
const recherche = [chemin && `voir=${encodeURIComponent(chemin)}`, cle].filter(Boolean).join("&");
const adresse = `http://localhost:${port}/pile${recherche ? `?${recherche}` : ""}`;
const chrome = [process.env.CHROME, "C:/Program Files/Google/Chrome/Application/chrome.exe", "C:/Program Files (x86)/Google/Chrome/Application/chrome.exe"].find((c) => c && existsSync(c));
if (chrome) spawn(chrome, [adresse], { detached: true, stdio: "ignore" }).unref();
else spawn("cmd", ["/c", "start", "", adresse.replace(/&/g, "^&")], { detached: true, stdio: "ignore" }).unref();
console.log(`pile ouverte dans Chrome : ${adresse}`);
