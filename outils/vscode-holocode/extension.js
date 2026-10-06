// Extension VS Code pour HoloCode.
// - La commande « ouvrir dans le navigateur ».
// - La faute que le moteur refuse, soulignée à sa place, et sa correction d'un clic (l'ampoule,
//   ou Ctrl + .) quand le moteur dit le bon mot (ADR-046). Le moteur reste strict (ADR-037).
// - Les mots du langage proposés pendant qu'on écrit.
// Les couleurs du langage sont décrites dans syntaxes/holo.tmLanguage.json. La correction est la
// même que dans l'éditeur du navigateur : corrections.mjs est une copie de moteur/web/corrections.js.

const vscode = require("vscode");
const path = require("path");
const fs = require("fs");
const { spawn, execFile } = require("child_process");
const { pathToFileURL } = require("url");

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

// ---------------------------------------------------------------- la faute, et sa correction

// Le moteur en ligne de commande, construit dans le dépôt : moteur/target/release/holo.
function moteurDe(document) {
  const dossier = vscode.workspace.getWorkspaceFolder(document.uri);
  if (!dossier) return null;
  for (const profil of ["release", "debug"]) {
    for (const nom of ["holo.exe", "holo"]) {
      const chemin = path.join(dossier.uri.fsPath, "moteur", "target", profil, nom);
      if (fs.existsSync(chemin)) return chemin;
    }
  }
  return null;
}

// Le texte tel qu'il est dans l'éditeur, même pas encore enregistré, passe par l'entrée standard.
function verifierTexte(moteur, document) {
  return new Promise((resoudre) => {
    let sortie = "";
    const processus = spawn(moteur, ["check", "-", path.dirname(document.uri.fsPath)], { windowsHide: true });
    const arret = setTimeout(() => processus.kill(), 5000);
    processus.stdout.setEncoding("utf8").on("data", (morceau) => { sortie += morceau; });
    processus.on("error", () => { clearTimeout(arret); resoudre(null); });
    processus.on("close", () => { clearTimeout(arret); resoudre(sortie.trim()); });
    processus.stdin.on("error", () => {});
    processus.stdin.end(document.getText(), "utf8");
  });
}

let corrections = null; // le module partagé avec l'éditeur du navigateur
let vocab = null; // les mots du langage, donnés par le moteur
let sansMoteurSignale = false;

async function preparer(contexte, moteur) {
  corrections ??= await import(pathToFileURL(path.join(contexte.extensionPath, "corrections.mjs")).href);
  vocab ??= await new Promise((resoudre) => {
    execFile(moteur, ["vocabulaire"], { windowsHide: true }, (erreur, sortie) => {
      try { resoudre(erreur ? null : JSON.parse(sortie)); } catch { resoudre(null); }
    });
  });
}

async function verifier(document, collection, contexte) {
  if (document.languageId !== "holo") return;
  const moteur = moteurDe(document);
  if (!moteur) {
    collection.delete(document.uri);
    if (!sansMoteurSignale) {
      sansMoteurSignale = true;
      vscode.window.showInformationMessage("HoloCode : pour souligner les fautes, construis le moteur une fois : cargo build --release --bin holo (dans moteur/).");
    }
    return;
  }
  await preparer(contexte, moteur);
  const version = document.version;
  const reponse = await verifierTexte(moteur, document);
  if (reponse === null || document.version !== version) return; // on a écrit entre-temps
  if (reponse.startsWith("ok")) {
    collection.set(document.uri, []);
    return;
  }
  const faute = corrections.lireFaute(reponse, document.getText());
  const plage = new vscode.Range(document.positionAt(faute.debut), document.positionAt(faute.fin));
  const diagnostic = new vscode.Diagnostic(plage, faute.message, vscode.DiagnosticSeverity.Error);
  diagnostic.source = "HoloCode";
  collection.set(document.uri, [diagnostic]);
}

// L'ampoule : « Remplacer « h1 » par « H1 » », quand le moteur a dit le bon mot.
const correctionsDUnClic = {
  provideCodeActions(document, _plage, contexte) {
    if (!corrections) return [];
    const texte = document.getText();
    return contexte.diagnostics.filter((d) => d.source === "HoloCode").flatMap((d) => {
      const faute = { ligne: d.range.start.line + 1, debut: document.offsetAt(d.range.start), fin: document.offsetAt(d.range.end), message: d.message };
      const correction = corrections.correctionPour(faute, texte, vocab);
      if (!correction) return [];
      const action = new vscode.CodeAction(correction.libelle, vscode.CodeActionKind.QuickFix);
      action.edit = new vscode.WorkspaceEdit();
      action.edit.replace(document.uri, new vscode.Range(document.positionAt(correction.debut), document.positionAt(correction.fin)), correction.par);
      action.diagnostics = [d];
      action.isPreferred = true;
      return [action];
    });
  },
};

// ---------------------------------------------------------------- les mots du langage

const motsDuLangage = {
  provideCompletionItems(document, position) {
    if (!vocab) return [];
    const avant = document.lineAt(position).text.slice(0, position.character);
    const items = [];
    const ajouter = (mot, sorte, insertion, detail) => {
      const item = new vscode.CompletionItem(mot, sorte);
      if (insertion) item.insertText = insertion;
      item.detail = detail;
      items.push(item);
    };
    // Après un nom et un point : `Ajouter.` → tap ; `panier.` → add(…) ; `Key.` → left.
    const point = /([A-Za-z_][A-Za-z0-9_]*)\.[A-Za-z0-9_]*$/.exec(avant);
    if (point) {
      const base = point[1];
      if (base === "Key") vocab.touches.forEach((m) => ajouter(m, vscode.CompletionItemKind.EnumMember, null, "touche du clavier"));
      else if (/^[A-Z]/.test(base)) [...vocab.signaux, ...vocab.capacites].forEach((m) => ajouter(m, vscode.CompletionItemKind.Event, null, "signal ou capacité"));
      else vocab.demandes.forEach((m) => ajouter(m, vscode.CompletionItemKind.Method, new vscode.SnippetString(`${m}($1)`), "demande"));
      return items;
    }
    vocab.blocs.forEach((b) => ajouter(b, vscode.CompletionItemKind.Class, new vscode.SnippetString(`${b}($1)`), "bloc"));
    new Set(Object.values(vocab.parametres).flat()).forEach((p) => ajouter(p, vscode.CompletionItemKind.Property, `${p}: `, "réglage d'un bloc"));
    vocab.reglages.forEach((r) => ajouter(r, vscode.CompletionItemKind.Property, `${r}: `, "réglage d'un style"));
    (corrections?.valeursDeclarees(document.getText()) ?? []).forEach((v) => ajouter(v, vscode.CompletionItemKind.Variable, null, "valeur de la page"));
    [...vocab.calculees, ...vocab.mots].forEach((m) => ajouter(m, vscode.CompletionItemKind.Value, null, "mot du langage"));
    return items;
  },
};

// ---------------------------------------------------------------- départ

function activate(contexte) {
  const collection = vscode.languages.createDiagnosticCollection("holocode");
  const attentes = new Map();
  const bientot = (document) => {
    clearTimeout(attentes.get(document.uri.toString()));
    attentes.set(document.uri.toString(), setTimeout(() => verifier(document, collection, contexte), 400));
  };
  contexte.subscriptions.push(
    collection,
    vscode.commands.registerCommand("holocode.ouvrir", ouvrir),
    vscode.workspace.onDidOpenTextDocument(bientot),
    vscode.workspace.onDidChangeTextDocument((e) => bientot(e.document)),
    vscode.workspace.onDidCloseTextDocument((d) => collection.delete(d.uri)),
    vscode.languages.registerCodeActionsProvider("holo", correctionsDUnClic, { providedCodeActionKinds: [vscode.CodeActionKind.QuickFix] }),
    vscode.languages.registerCompletionItemProvider("holo", motsDuLangage, "."),
  );
  vscode.workspace.textDocuments.forEach(bientot);
}

function deactivate() {}

module.exports = { activate, deactivate };
