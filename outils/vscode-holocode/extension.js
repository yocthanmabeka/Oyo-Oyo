// Extension VS Code pour HoloCode : la commande « ouvrir dans le navigateur ».
// Les couleurs du langage sont décrites dans syntaxes/holo.tmLanguage.json.

const vscode = require("vscode");
const path = require("path");

// Le serveur local (moteur/outils/serveur.mjs) sert deux dossiers du dépôt.
function adresseDe(cheminDansLeDepot) {
  if (cheminDansLeDepot.startsWith("exemples/")) return "/" + cheminDansLeDepot;
  if (cheminDansLeDepot.startsWith("moteur/mondes/")) return "/mondes/" + cheminDansLeDepot.slice("moteur/mondes/".length);
  return null;
}

async function ouvrir(uri) {
  const fichier = uri ?? vscode.window.activeTextEditor?.document.uri;
  if (!fichier || !fichier.fsPath.endsWith(".holo")) {
    vscode.window.showWarningMessage("HoloCode : ouvre d'abord un fichier .holo.");
    return;
  }
  // Ce qui est affiché doit être ce qui est écrit : on enregistre avant d'ouvrir.
  const document = vscode.workspace.textDocuments.find((d) => d.uri.fsPath === fichier.fsPath);
  if (document?.isDirty) await document.save();

  const dossier = vscode.workspace.getWorkspaceFolder(fichier);
  const relatif = dossier ? path.relative(dossier.uri.fsPath, fichier.fsPath).replace(/\\/g, "/") : "";
  const adresse = adresseDe(relatif);
  if (!adresse) {
    vscode.window.showWarningMessage("HoloCode : le serveur local n'affiche que les fichiers rangés dans exemples/ ou dans moteur/mondes/.");
    return;
  }
  const serveur = vscode.workspace.getConfiguration("holocode").get("serveur").replace(/\/$/, "");
  try {
    await fetch(serveur + "/accueil.html", { signal: AbortSignal.timeout(1500) });
  } catch {
    vscode.window.showErrorMessage(`HoloCode : le serveur local ne répond pas (${serveur}). Dans un terminal : node moteur/outils/serveur.mjs`);
    return;
  }
  await vscode.env.openExternal(vscode.Uri.parse(serveur + encodeURI(adresse)));
}

function activate(context) {
  context.subscriptions.push(vscode.commands.registerCommand("holocode.ouvrir", ouvrir));
}

function deactivate() {}

module.exports = { activate, deactivate };
