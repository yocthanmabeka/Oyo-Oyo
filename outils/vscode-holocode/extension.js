// Extension VS Code pour HoloCode.
// - La commande « ouvrir dans le navigateur ».
// - La faute que le moteur refuse, soulignée à sa place, et sa correction d'un clic (l'ampoule,
//   ou Ctrl + .) quand le moteur dit le bon mot (ADR-046). Le moteur reste strict (ADR-037).
// - Les mots du langage proposés pendant qu'on écrit.
// Les couleurs du langage sont décrites dans syntaxes/holo.tmLanguage.json. La correction est la
// même que dans l'éditeur du navigateur : fixes.mjs est une copie de moteur/web/fixes.js.

const vscode = require("vscode");
const path = require("path");
const fs = require("fs");
const { spawn, execFile } = require("child_process");
const { pathToFileURL } = require("url");

// Le serveur local (moteur/outils/server.mjs) sert deux dossiers du dépôt.
function addressOf(pathInRepo) {
  if (pathInRepo.startsWith("exemples/")) return "/" + pathInRepo;
  if (pathInRepo.startsWith("moteur/mondes/")) return "/mondes/" + pathInRepo.slice("moteur/mondes/".length);
  return null;
}

async function open(uri) {
  const file = uri ?? vscode.window.activeTextEditor?.document.uri;
  if (!file || !file.fsPath.endsWith(".holo")) {
    vscode.window.showWarningMessage("HoloCode : ouvre d'abord un fichier .holo.");
    return;
  }
  // Ce qui est affiché doit être ce qui est écrit : on enregistre avant d'ouvrir.
  const document = vscode.workspace.textDocuments.find((d) => d.uri.fsPath === file.fsPath);
  if (document?.isDirty) await document.save();

  const folder = vscode.workspace.getWorkspaceFolder(file);
  const relative = folder ? path.relative(folder.uri.fsPath, file.fsPath).replace(/\\/g, "/") : "";
  const address = addressOf(relative);
  if (!address) {
    vscode.window.showWarningMessage("HoloCode : le serveur local n'affiche que les fichiers rangés dans exemples/ ou dans moteur/mondes/.");
    return;
  }
  const server = vscode.workspace.getConfiguration("holocode").get("server").replace(/\/$/, "");
  try {
    await fetch(server + "/home.html", { signal: AbortSignal.timeout(1500) });
  } catch {
    vscode.window.showErrorMessage(`HoloCode : le serveur local ne répond pas (${server}). Dans un terminal : node moteur/outils/server.mjs`);
    return;
  }
  await vscode.env.openExternal(vscode.Uri.parse(server + encodeURI(address)));
}

// ---------------------------------------------------------------- la faute, et sa correction

// Le moteur en ligne de commande, construit dans le dépôt : moteur/target/release/holo.
function engineOf(document) {
  const folder = vscode.workspace.getWorkspaceFolder(document.uri);
  if (!folder) return null;
  for (const profile of ["release", "debug"]) {
    for (const name of ["holo.exe", "holo"]) {
      const enginePath = path.join(folder.uri.fsPath, "moteur", "target", profile, name);
      if (fs.existsSync(enginePath)) return enginePath;
    }
  }
  return null;
}

// Le texte tel qu'il est dans l'éditeur, même pas encore enregistré, passe par l'entrée standard.
function checkText(engine, document) {
  return new Promise((resolve) => {
    let output = "";
    const process = spawn(engine, ["check", "-", path.dirname(document.uri.fsPath)], { windowsHide: true });
    const stop = setTimeout(() => process.kill(), 5000);
    process.stdout.setEncoding("utf8").on("data", (chunk) => { output += chunk; });
    process.on("error", () => { clearTimeout(stop); resolve(null); });
    process.on("close", () => { clearTimeout(stop); resolve(output.trim()); });
    process.stdin.on("error", () => {});
    process.stdin.end(document.getText(), "utf8");
  });
}

let fixes = null; // le module partagé avec l'éditeur du navigateur
let vocab = null; // les mots du langage, donnés par le moteur
let noEngineReported = false;

async function prepare(context, engine) {
  fixes ??= await import(pathToFileURL(path.join(context.extensionPath, "fixes.mjs")).href);
  vocab ??= await new Promise((resolve) => {
    execFile(engine, ["vocabulary"], { windowsHide: true }, (error, output) => {
      try { resolve(error ? null : JSON.parse(output)); } catch { resolve(null); }
    });
  });
}

async function check(document, collection, context) {
  if (document.languageId !== "holo") return;
  const engine = engineOf(document);
  if (!engine) {
    collection.delete(document.uri);
    if (!noEngineReported) {
      noEngineReported = true;
      vscode.window.showInformationMessage("HoloCode : pour souligner les fautes, construis le moteur une fois : cargo build --release --bin holo (dans moteur/).");
    }
    return;
  }
  await prepare(context, engine);
  const version = document.version;
  const response = await checkText(engine, document);
  if (response === null || document.version !== version) return; // on a écrit entre-temps
  if (response.startsWith("ok")) {
    collection.set(document.uri, []);
    return;
  }
  const fault = fixes.readFault(response, document.getText());
  const range = new vscode.Range(document.positionAt(fault.start), document.positionAt(fault.end));
  const diagnostic = new vscode.Diagnostic(range, fault.message, vscode.DiagnosticSeverity.Error);
  diagnostic.source = "HoloCode";
  collection.set(document.uri, [diagnostic]);
}

// L'ampoule : « Remplacer « h1 » par « H1 » », quand le moteur a dit le bon mot.
const oneClickFixes = {
  provideCodeActions(document, _range, context) {
    if (!fixes) return [];
    const text = document.getText();
    return context.diagnostics.filter((d) => d.source === "HoloCode").flatMap((d) => {
      const fault = { line: d.range.start.line + 1, start: document.offsetAt(d.range.start), end: document.offsetAt(d.range.end), message: d.message };
      const fix = fixes.fixFor(fault, text, vocab);
      if (!fix) return [];
      const action = new vscode.CodeAction(fix.label, vscode.CodeActionKind.QuickFix);
      action.edit = new vscode.WorkspaceEdit();
      action.edit.replace(document.uri, new vscode.Range(document.positionAt(fix.start), document.positionAt(fix.end)), fix.by);
      action.diagnostics = [d];
      action.isPreferred = true;
      return [action];
    });
  },
};

// ---------------------------------------------------------------- les mots du langage

const languageWords = {
  provideCompletionItems(document, position) {
    if (!vocab) return [];
    const before = document.lineAt(position).text.slice(0, position.character);
    const items = [];
    const add = (word, kind, insertion, detail) => {
      const item = new vscode.CompletionItem(word, kind);
      if (insertion) item.insertText = insertion;
      item.detail = detail;
      items.push(item);
    };
    // Après un nom et un point : `Ajouter.` → tap ; `panier.` → add(…) ; `Key.` → left.
    const point = /([A-Za-z_][A-Za-z0-9_]*)\.[A-Za-z0-9_]*$/.exec(before);
    if (point) {
      const base = point[1];
      if (base === "Key") vocab.keypresses.forEach((m) => add(m, vscode.CompletionItemKind.EnumMember, null, "touche du clavier"));
      else if (/^[A-Z]/.test(base)) [...vocab.signals, ...vocab.capabilities].forEach((m) => add(m, vscode.CompletionItemKind.Event, null, "signal ou capacité"));
      else vocab.requests.forEach((m) => add(m, vscode.CompletionItemKind.Method, new vscode.SnippetString(`${m}($1)`), "demande"));
      return items;
    }
    vocab.blocks.forEach((b) => add(b, vscode.CompletionItemKind.Class, new vscode.SnippetString(`${b}($1)`), "bloc"));
    new Set(Object.values(vocab.params).flat()).forEach((p) => add(p, vscode.CompletionItemKind.Property, `${p}: `, "réglage d'un bloc"));
    vocab.settings.forEach((r) => add(r, vscode.CompletionItemKind.Property, `${r}: `, "réglage d'un style"));
    (fixes?.declaredValues(document.getText()) ?? []).forEach((v) => add(v, vscode.CompletionItemKind.Variable, null, "valeur de la page"));
    [...vocab.computed, ...vocab.words].forEach((m) => add(m, vscode.CompletionItemKind.Value, null, "mot du langage"));
    return items;
  },
};

// ---------------------------------------------------------------- départ

function activate(context) {
  const collection = vscode.languages.createDiagnosticCollection("holocode");
  const waitings = new Map();
  const soon = (document) => {
    clearTimeout(waitings.get(document.uri.toString()));
    waitings.set(document.uri.toString(), setTimeout(() => check(document, collection, context), 400));
  };
  context.subscriptions.push(
    collection,
    vscode.commands.registerCommand("holocode.open", open),
    vscode.workspace.onDidOpenTextDocument(soon),
    vscode.workspace.onDidChangeTextDocument((e) => soon(e.document)),
    vscode.workspace.onDidCloseTextDocument((d) => collection.delete(d.uri)),
    vscode.languages.registerCodeActionsProvider("holo", oneClickFixes, { providedCodeActionKinds: [vscode.CodeActionKind.QuickFix] }),
    vscode.languages.registerCompletionItemProvider("holo", languageWords, "."),
  );
  vscode.workspace.textDocuments.forEach(soon);
}

function deactivate() {}

module.exports = { activate, deactivate };
