// L'éditeur HoloCode (ADR-046). Le même moteur que celui des pages : il vérifie le texte à
// chaque pause de l'écriture, et fabrique l'aperçu. L'éditeur ne décide de rien sur le langage :
// il montre la faute que le moteur refuse, et, quand le moteur dit le bon mot (« écris « H1 » »),
// il propose de le mettre d'un clic. Le moteur reste strict (ADR-037).

import init, { check_text, vocabulary, flat_view, imports, set_now } from "/pkg-light/holo_engine.js";
import { readFault, fixFor, declaredValues, blockNames } from "/fixes.js";

const $ = (id) => document.getElementById(id);
const area = $("code");
const drawing = $("drawing");
const numbers_ = $("numbers");
const preview = $("preview");
const state = $("status");
const fixButton = $("fix");
const words = $("words");
const faultyLine = $("faulty-line");

const params = new URLSearchParams(location.search);
const key = params.get("key") ?? "";
let path = params.get("file") ?? "/exemples/lecons/01-page.holo";
let saved = ""; // le texte tel qu'il est sur le disque
let windowsLineEndings = false; // le fichier était écrit avec \r\n : on le garde ainsi
let fault = null; // { ligne, debut, fin, message }
let fix = null; // { debut, fin, par, libelle }
let vocab = { blocks: [], params: {}, settings: [], states: [], requests: [], signals: [], capabilities: [], keypresses: [], words: [], computed: [], formats: [] };

const NEXT_FILE = "\u001e";
const NAME_SEPARATOR = "\u001f";
const ADDRESS_FILE = "@adresse"; // les valeurs d'une adresse, pour le moteur (ADR-078)
const MODEL = 'Page(\n  title: "Ma page",\n  children: [\n    H1("Ma page"),\n    "Écris ici.",\n  ],\n)\n';
const folderOf = (c) => c.slice(0, c.lastIndexOf("/") + 1);
const escape = (t) => t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const lineHeight = () => parseFloat(getComputedStyle(area).lineHeight) || 22.4;
const textTop = () => parseFloat(getComputedStyle(area).paddingTop) || 12;

// ---------------------------------------------------------------- le fichier

async function openFile(newOne, { newFile = false } = {}) {
  if (area.value !== saved && !confirm("Le fichier ouvert n'est pas enregistré. L'abandonner ?")) return;
  path = newOne;
  history.replaceState(null, "", `?file=${encodeURIComponent(path)}${key ? `&key=${encodeURIComponent(key)}` : ""}`);
  let text = MODEL;
  saved = "";
  if (!newFile) {
    try {
      const response = await fetch(path, { headers: { accept: "text/plain" }, cache: "no-store" });
      if (response.ok) {
        text = await response.text();
        saved = text.replace(/\r\n/g, "\n");
      }
    } catch { /* pas de serveur : on part du modèle */ }
  }
  windowsLineEndings = text.includes("\r\n");
  area.value = text.replace(/\r\n/g, "\n");
  area.scrollTop = 0;
  importedOnes.clear();
  $("path").textContent = path;
  document.title = `${path.split("/").pop()} — éditeur HoloCode`;
  $("open").href = path;
  updateModified();
  draw();
  checkSoon(0);
}

function updateModified() {
  $("modified").hidden = area.value === saved;
}

async function save() {
  if (!key) {
    flag("Pour enregistrer, ouvre l'éditeur par l'adresse que le serveur affiche à son démarrage : elle contient la clé.");
    return;
  }
  const text = windowsLineEndings ? area.value.replace(/\n/g, "\r\n") : area.value;
  try {
    const response = await fetch(path, { method: "PUT", headers: { "content-type": "text/plain; charset=utf-8", "x-holo-key": key }, body: text });
    if (!response.ok) return flag(`Pas enregistré : ${await response.text()}`);
    saved = area.value;
    updateModified();
    flag("Enregistré.");
  } catch {
    flag("Pas enregistré : le serveur ne répond pas.");
  }
}

let temporaryMessage = null;
function flag(text) {
  temporaryMessage?.remove();
  temporaryMessage = Object.assign(document.createElement("div"), { className: "temporary-message", role: "status", textContent: text });
  document.body.append(temporaryMessage);
  setTimeout(() => temporaryMessage?.remove(), 3500);
}

// ---------------------------------------------------------------- vérifier, et montrer la page

// Les fichiers importés (import "commun.holo") sont lus à côté et joints au texte, comme le
// fait la page : le moteur ne lit rien tout seul.
const importedOnes = new Map();
async function withImports(text) {
  let names = [];
  try { names = imports(text).split(";").filter(Boolean); } catch { /* texte illisible : sans imports */ }
  let complete = text;
  for (const name of names) {
    const address = folderOf(path) + name;
    if (!importedOnes.has(address)) {
      importedOnes.set(address, fetch(address, { headers: { accept: "text/plain" } }).then((r) => (r.ok ? r.text() : null)).catch(() => null));
    }
    const content = await importedOnes.get(address);
    if (content !== null) complete += NEXT_FILE + name + NAME_SEPARATOR + content;
  }
  // Un modèle d'adresse (profil/{id}.holo, ADR-078) n'a pas d'adresse ici : chaque nom vaut un
  // texte vide, comme pour holo check. Joint après la page et ses imports, comme le fait la page.
  const addressNames = path.replace(/%7B/gi, "{").replace(/%7D/gi, "}").replace(/\.holo$/, "").split("/").map((piece) => /^\{(.*)\}$/.exec(piece)?.[1]).filter((name) => name !== undefined);
  if (addressNames.length) complete += NEXT_FILE + ADDRESS_FILE + NAME_SEPARATOR + addressNames.map((name) => `${name}=`).join("&");
  return complete;
}

let timer = 0;
let turn = 0;
function checkSoon(ms = 250) {
  clearTimeout(timer);
  timer = setTimeout(check, ms);
}

async function check() {
  const myTurn = ++turn;
  const text = area.value;
  const complete = await withImports(text);
  if (myTurn !== turn) return; // on a écrit entre-temps
  const d = new Date();
  set_now(d.getFullYear(), d.getMonth() + 1, d.getDate(), ((d.getDay() + 6) % 7) + 1, d.getHours(), d.getMinutes());
  const response = check_text(complete);
  fault = response.startsWith("ok") ? null : readFault(response, text);
  fix = fault ? fixFor(fault, text, vocab) : null;
  draw();
  showStatus(response);
  if (!fault) showPreview(complete);
  $("preview-old").hidden = !fault;
}

// ---------------------------------------------------------------- la correction d'un clic

function applyFix() {
  if (!fix) return;
  area.focus();
  replace(fix.start, fix.end, fix.by);
  checkSoon(0);
}

// Remplace un morceau du texte en gardant l'annulation (Ctrl + Z) du navigateur.
function replace(start, end, by) {
  area.setSelectionRange(start, end);
  if (!document.execCommand("insertText", false, by)) area.setRangeText(by, start, end, "end");
  afterKeystroke();
}

// ---------------------------------------------------------------- ce que le fichier déclare

const styleNames = (text) => [...text.matchAll(/[A-Z][A-Za-z0-9]*\.([a-z][A-Za-z0-9]*)\s*\(/g)].map((m) => m[1]);
const declaredVariables = (text) => [...text.matchAll(/(--[a-z][a-z0-9-]*)\s*:/g)].map((m) => m[1]);

// ---------------------------------------------------------------- les couleurs du texte

// Où finit le bloc racine : après lui viennent les styles.
function rootEnd(t) {
  let i = 0;
  let depth = 0;
  let begins = false;
  while (i < t.length) {
    const c = t[i];
    if (c === "/" && t[i + 1] === "/") {
      const end = t.indexOf("\n", i);
      if (end < 0) return t.length;
      i = end;
      continue;
    }
    if (c === '"') {
      i = textEnd(t, i);
      continue;
    }
    if (c === "(" || c === "[") {
      depth++;
      begins = true;
    } else if ((c === ")" || c === "]") && --depth === 0 && begins) return i + 1;
    i++;
  }
  return t.length;
}

function textEnd(t, i) {
  if (t.startsWith('"""', i)) {
    const end = t.indexOf('"""', i + 3);
    return end < 0 ? t.length : end + 3;
  }
  const quoteMark = t.indexOf('"', i + 1);
  const line = t.indexOf("\n", i + 1);
  if (quoteMark < 0) return line < 0 ? t.length : line;
  return line >= 0 && line < quoteMark ? line : quoteMark + 1;
}

function tokens(t) {
  const output = [];
  const add = (start, end, kind) => { if (end > start) output.push([start, end, kind]); };
  const root = rootEnd(t);
  const lineStart = (i) => /(^|\n)\s*$/.test(t.slice(Math.max(0, i - 40), i));
  let i = 0;
  while (i < root) {
    const c = t[i];
    if (c === "/" && t[i + 1] === "/") {
      const end = t.indexOf("\n", i) < 0 ? t.length : t.indexOf("\n", i);
      add(i, end, "comment");
      i = end;
      continue;
    }
    if (c === '"') {
      const end = textEnd(t, i);
      const re = /\{[a-zA-Z][a-zA-Z0-9_.]*(?::[a-zA-Z0-9]+)?\}/g;
      re.lastIndex = i;
      let j = i;
      for (let m = re.exec(t); m && m.index + m[0].length <= end; m = re.exec(t)) {
        add(j, m.index, "text");
        add(m.index, m.index + m[0].length, "in-text");
        j = m.index + m[0].length;
      }
      add(j, end, "text");
      i = end;
      continue;
    }
    if (/[0-9-]/.test(c) && !/[A-Za-z0-9_]/.test(t[i - 1] ?? "")) {
      const number = /^-?\d+(?:\.\d+)?(?:px|ms|min|deg|KB|MB|GB|B|mm|cm|km|s|h|m)?(?![A-Za-z0-9_])/.exec(t.slice(i, i + 40));
      if (number) {
        add(i, i + number[0].length, "number");
        i += number[0].length;
        continue;
      }
    }
    const word = /^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*/.exec(t.slice(i, i + 200));
    if (word) {
      const end = i + word[0].length;
      let k = end;
      while (t[k] === " " || t[k] === "\t") k++;
      let kind;
      if (t[k] === "(") kind = /^[A-Z]/.test(word[0]) ? "block" : "request";
      else if (t[k] === ":") kind = "param";
      else if (word[0].includes(".") && /^[A-Z]/.test(word[0])) kind = "signal";
      else if (["import", "module", "bridge"].includes(word[0]) && lineStart(i)) kind = "directive";
      else if (word[0] === "true" || word[0] === "false") kind = "keyword";
      else kind = /^[A-Z]/.test(word[0]) ? "name" : "value";
      add(i, end, kind);
      i = end;
      continue;
    }
    if ("()[],:".includes(c)) add(i, i + 1, "punctuation");
    i++;
  }
  // Les styles, écrits comme en CSS.
  let depth = 0;
  while (i < t.length) {
    const c = t[i];
    if (c === "/" && t[i + 1] === "/") {
      const end = t.indexOf("\n", i) < 0 ? t.length : t.indexOf("\n", i);
      add(i, end, "comment");
      i = end;
      continue;
    }
    if (c === "{" || c === "}") {
      depth = Math.max(0, depth + (c === "{" ? 1 : -1));
      add(i, i + 1, "punctuation");
      i++;
      continue;
    }
    if (c === '"') {
      const end = textEnd(t, i);
      add(i, end, "text");
      i = end;
      continue;
    }
    const color = /^#[0-9a-fA-F]{3,8}(?![0-9a-zA-Z])/.exec(t.slice(i, i + 10));
    const number = /^-?\d+(?:\.\d+)?(?:px|%|ms|s|deg)?(?![A-Za-z0-9_])/.exec(t.slice(i, i + 20));
    if (color || (number && !/[A-Za-z0-9_-]/.test(t[i - 1] ?? ""))) {
      const length = (color ?? number)[0].length;
      add(i, i + length, "number");
      i += length;
      continue;
    }
    const word = /^(?:--)?[A-Za-z_.][A-Za-z0-9_-]*/.exec(t.slice(i, i + 200));
    if (word) {
      const end = i + word[0].length;
      let k = end;
      while (t[k] === " ") k++;
      let kind;
      if (depth === 0) kind = word[0].startsWith(".") ? "style-name" : "block";
      else if (t[k] === ":") {
        let q = k + 1;
        while (t[q] === " ") q++;
        kind = t[q] === "{" ? "state" : word[0].startsWith("--") ? "variable" : "param";
      } else kind = word[0].startsWith("--") ? "variable" : "value";
      add(i, end, kind);
      i = end;
      continue;
    }
    if (":;,()".includes(c)) add(i, i + 1, "punctuation");
    i++;
  }
  return output;
}

function colorize(t) {
  const mark = fault && fault.end > fault.start ? [fault.start, fault.end] : null;
  const chunk = (a, b, kind) => {
    if (b <= a) return "";
    const x = escape(t.slice(a, b));
    return kind ? `<span class="j-${kind}">${x}</span>` : x;
  };
  const withFault = (a, b, kind) => {
    if (!mark || b <= mark[0] || a >= mark[1]) return chunk(a, b, kind);
    const [x, y] = [Math.max(a, mark[0]), Math.min(b, mark[1])];
    return chunk(a, x, kind) + `<mark class="fault">${chunk(x, y, kind)}</mark>` + chunk(y, b, kind);
  };
  let html = "";
  let position = 0;
  for (const [start, end, kind] of tokens(t)) {
    html += withFault(position, start, null) + withFault(start, end, kind);
    position = end;
  }
  return html + withFault(position, t.length, null) + "\n";
}

function draw() {
  drawing.innerHTML = colorize(area.value);
  const lines = area.value.split("\n").length;
  if (numbers_.dataset.lines !== String(lines)) {
    numbers_.textContent = Array.from({ length: lines }, (_, i) => i + 1).join("\n");
    numbers_.dataset.lines = String(lines);
  }
  followScroll();
}

function followScroll() {
  drawing.scrollTop = area.scrollTop;
  drawing.scrollLeft = area.scrollLeft;
  numbers_.scrollTop = area.scrollTop;
  faultyLine.hidden = !fault;
  if (fault) faultyLine.style.top = `${textTop() + (fault.line - 1) * lineHeight() - area.scrollTop}px`;
}

// ---------------------------------------------------------------- l'état, et l'aperçu

function showStatus(response) {
  fixButton.hidden = !fix;
  if (fix) fixButton.textContent = fix.label;
  if (!fault) {
    state.className = "valid";
    state.textContent = response === "ok" ? "✓ Le fichier est juste." : "✓ Ce morceau est juste ; il se vérifie aussi dans la page qui l'importe.";
    return;
  }
  state.className = "invalid";
  state.textContent = `Ligne ${fault.line} : ${fault.message}`;
  state.title = "Aller à la faute";
}

function goToFault() {
  if (!fault) return;
  document.body.classList.remove("see-preview");
  updateTabs();
  area.focus();
  area.setSelectionRange(fault.start, fault.end);
  area.scrollTop = Math.max(0, (fault.line - 1) * lineHeight() - area.clientHeight / 3);
  followScroll();
}

let previewScroll = 0;
function showPreview(complete) {
  let html;
  try {
    html = flat_view(complete, folderOf(path), undefined);
  } catch {
    html = `<p style="font:16px system-ui;padding:24px">Ce fichier est un monde : il se regarde en profondeur. Touche « Ouvrir la page ».</p>`;
  }
  try { previewScroll = preview.contentWindow?.scrollY ?? 0; } catch { /* pas encore de page */ }
  preview.onload = () => { try { preview.contentWindow.scrollTo(0, previewScroll); } catch { /* rien */ } };
  preview.srcdoc = `<!doctype html><html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><style>html,body{margin:0}</style></head><body>${html}</body></html>`;
}

// ---------------------------------------------------------------- les mots à toucher

// Où l'on écrit : dans les réglages d'un bloc (le dernier `Nom(` encore ouvert), ou dans une
// liste (`children: [`), et laquelle.
function whereAreWe(t, pos) {
  const stack = [];
  let i = 0;
  while (i < pos) {
    const c = t[i];
    if (c === "/" && t[i + 1] === "/") {
      const end = t.indexOf("\n", i);
      i = end < 0 ? pos : end;
      continue;
    }
    if (c === '"') {
      i = textEnd(t, i);
      continue;
    }
    if (c === "(") stack.push({ block: /([A-Za-z_][A-Za-z0-9_]*)(?:\.[A-Za-z0-9_]+)?\s*$/.exec(t.slice(Math.max(0, i - 60), i))?.[1] ?? "" });
    else if (c === "[") stack.push({ list: /([a-z][A-Za-z0-9]*)\s*:\s*$/.exec(t.slice(Math.max(0, i - 40), i))?.[1] ?? "" });
    else if (c === ")" || c === "]") stack.pop();
    i++;
  }
  const above = stack.at(-1);
  return { block: [...stack].reverse().find((p) => p.block !== undefined)?.block ?? "", list: above && above.list !== undefined ? above.list : null };
}

// Ce qu'on range dans chaque sorte de liste.
const LISTS_CONTENT = { rules: ["On", "Every", "When", "After", "If"], fonts: ["Font"], modules: ["Module"], items: ["Item"], pixels: ["Point"] };

function inText(t, pos) {
  let i = 0;
  while (i < pos) {
    if (t[i] === "/" && t[i + 1] === "/") {
      const end = t.indexOf("\n", i);
      if (end < 0 || end >= pos) return false;
      i = end;
      continue;
    }
    if (t[i] === '"') {
      const end = textEnd(t, i);
      if (end > pos || (end === pos && t[end - 1] !== '"')) return true;
      i = end;
      continue;
    }
    i++;
  }
  return false;
}

let proposals = [];
let currentChunk = { start: 0, end: 0 };

function propose() {
  const t = area.value;
  const pos = area.selectionStart;
  proposals = [];
  if (pos === area.selectionEnd) {
    const before = t.slice(0, pos);
    const root = rootEnd(t);
    if (pos > root) proposals = inStyles(t, pos, before);
    else if (inText(t, pos)) {
      const m = /\{([a-z][A-Za-z0-9]*)?$/.exec(before);
      if (m) {
        currentChunk = { start: pos - (m[1]?.length ?? 0), end: pos };
        proposals = sort(m[1] ?? "", [...declaredValues(t), ...vocab.computed]).map((word) => ({ word, suite: "}" }));
      }
    } else proposals = inCode(t, pos, before);
  }
  words.replaceChildren(...proposals.slice(0, 12).map(({ word }, rank) => {
    const button = Object.assign(document.createElement("button"), { type: "button", textContent: word });
    button.dataset.rank = rank;
    return button;
  }));
}

function inCode(t, pos, before) {
  const m = /([A-Za-z_][A-Za-z0-9_]*\.)?([A-Za-z_][A-Za-z0-9_]*)?$/.exec(before);
  const [integer, prefix = "", chunk = ""] = m;
  currentChunk = { start: pos - chunk.length, end: pos };
  // Après un nom et un point : `Ajouter.` → tap ; `panier.` → add.
  if (prefix) {
    const base = prefix.slice(0, -1);
    if (base === "Key") return sort(chunk, vocab.keypresses).map((word) => ({ word }));
    if (/^[A-Z]/.test(base)) return sort(chunk, [...vocab.signals, ...vocab.capabilities]).map((word) => ({ word }));
    return sort(chunk, vocab.requests).map((word) => ({ word, suite: "(" }));
  }
  if (!integer && !/[(,[]\s*$/.test(before)) return [];
  const previous = /(\S)\s*$/.exec(before.slice(0, pos - chunk.length))?.[1] ?? "";
  if (/^[A-Z]/.test(chunk)) {
    return sort(chunk, [...vocab.blocks, ...blockNames(t), "Key"]).map((word) => ({ word, suite: vocab.blocks.includes(word) ? "(" : "" }));
  }
  if (previous === ":") {
    return sort(chunk, [...declaredValues(t), ...vocab.computed, ...vocab.words, ...blockNames(t)]).map((word) => ({ word }));
  }
  const { block, list } = whereAreWe(t, pos);
  // Dans une liste : les blocs qu'on y range (keep: [ … ] prend des valeurs).
  if (list !== null && list !== "keep") {
    const blocks = LISTS_CONTENT[list] ?? vocab.blocks.filter((b) => !["Page", "State", "Shared", "Prices", "Data", "Zoom", "Points", "Relief", "Portals", "Item", "Font", "Module", "On", "Every", "When", "After", "Scene"].includes(b));
    return sort(chunk, blocks).map((word) => ({ word, suite: "(" }));
  }
  if (list === "keep") return sort(chunk, declaredValues(t)).map((word) => ({ word }));
  const settings = vocab.params[block] ?? [];
  return [
    ...sort(chunk, settings).map((word) => ({ word, suite: ": " })),
    ...sort(chunk, [...declaredValues(t), ...vocab.computed]).filter((word) => !settings.includes(word)).map((word) => ({ word })),
  ];
}

function inStyles(t, pos, before) {
  const styles = t.slice(rootEnd(t), pos);
  const depth = [...styles].reduce((p, c) => (c === "{" ? p + 1 : c === "}" ? Math.max(0, p - 1) : p), 0);
  const m = /(--|\.)?([A-Za-z][A-Za-z0-9-]*)?$/.exec(before);
  const [integer, sign = "", chunk = ""] = m;
  currentChunk = { start: pos - integer.length, end: pos };
  if (depth === 0) {
    return sort(integer, [...vocab.blocks, ...styleNames(t).map((n) => `.${n}`)]).map((word) => ({ word, suite: " { " }));
  }
  const previous = /(\S)\s*$/.exec(before.slice(0, pos - integer.length))?.[1] ?? "";
  if (previous === ":") return sort(integer, [...declaredVariables(t), ...vocab.words]).map((word) => ({ word, suite: ";" }));
  if (!integer && !/[{;]\s*$/.test(before)) return [];
  return [
    ...sort(integer, vocab.settings).map((word) => ({ word, suite: ": " })),
    ...sort(integer, vocab.states).map((word) => ({ word, suite: ": { " })),
    ...(sign === "--" ? sort(integer, declaredVariables(t)).map((word) => ({ word, suite: ": " })) : []),
  ];
}

// Les mots qui commencent par ce qu'on a tapé d'abord, puis ceux qui le contiennent.
function sort(chunk, candidates) {
  const bottom = chunk.toLowerCase();
  const unique = [...new Set(candidates)].filter((c) => c && c !== chunk);
  const start = unique.filter((c) => c.toLowerCase().startsWith(bottom));
  const inside = bottom ? unique.filter((c) => !c.toLowerCase().startsWith(bottom) && c.toLowerCase().includes(bottom)) : [];
  return [...start, ...inside];
}

function take(rank) {
  const choice = proposals[rank];
  if (!choice) return;
  const t = area.value;
  let suite = choice.suite ?? "";
  // Ne pas doubler ce qui suit déjà : « ( », « : », « } ».
  const after = t.slice(currentChunk.end).trimStart();
  if (suite && after.startsWith(suite.trim())) suite = "";
  area.focus();
  replace(currentChunk.start, currentChunk.end, choice.word + suite);
}

// ---------------------------------------------------------------- la frappe

function afterKeystroke() {
  updateModified();
  draw();
  propose();
  checkSoon();
}

area.addEventListener("input", afterKeystroke);
area.addEventListener("scroll", followScroll);
for (const name of ["click", "keyup"]) area.addEventListener(name, (e) => { if (!(e.key && e.key.length === 1)) propose(); });
area.addEventListener("keydown", (e) => {
  if (e.isComposing) return;
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
    e.preventDefault();
    save();
    return;
  }
  // Tab prend le premier mot proposé ; sinon, deux espaces.
  if (e.key === "Tab" && !e.shiftKey) {
    e.preventDefault();
    if (proposals.length) take(0);
    else replace(area.selectionStart, area.selectionEnd, "  ");
    return;
  }
  if (e.key === "Escape") {
    proposals = [];
    words.replaceChildren();
    return;
  }
  // À la ligne : le même retrait, deux espaces de plus après une parenthèse ou un crochet ouvert.
  if (e.key === "Enter" && !e.shiftKey && !e.ctrlKey && !e.altKey) {
    e.preventDefault();
    const pos = area.selectionStart;
    const line = area.value.slice(area.value.lastIndexOf("\n", pos - 1) + 1, pos);
    const indent = /^\s*/.exec(line)[0] + (/[([{]\s*$/.test(line) ? "  " : "");
    replace(pos, area.selectionEnd, `\n${indent}`);
  }
});
// Toucher un mot ne doit pas fermer le clavier du téléphone : on garde le focus dans le texte.
words.addEventListener("pointerdown", (e) => { if (e.target.closest("button")) e.preventDefault(); });
words.addEventListener("click", (e) => {
  const button = e.target.closest("button");
  if (button) take(Number(button.dataset.rank));
});
fixButton.addEventListener("pointerdown", (e) => e.preventDefault());
fixButton.addEventListener("click", applyFix);
state.addEventListener("click", goToFault);
$("save").addEventListener("click", save);
addEventListener("beforeunload", (e) => { if (area.value !== saved) e.preventDefault(); });

// Sur un téléphone : Code ou Aperçu.
function updateTabs() {
  const previewVisible = document.body.classList.contains("see-preview");
  $("see-code").setAttribute("aria-pressed", String(!previewVisible));
  $("see-preview").setAttribute("aria-pressed", String(previewVisible));
}
$("see-code").addEventListener("click", () => { document.body.classList.remove("see-preview"); updateTabs(); });
$("see-preview").addEventListener("click", () => { document.body.classList.add("see-preview"); updateTabs(); });

// ---------------------------------------------------------------- les fichiers

let allFiles = [];
async function showFiles(opened) {
  $("files").hidden = !opened;
  $("open-files").setAttribute("aria-expanded", String(opened));
  if (!opened) return;
  try { allFiles = await (await fetch("/liste-holo", { cache: "no-store" })).json(); } catch { allFiles = []; }
  listFiles();
  $("filter").focus();
}

function listFiles() {
  const filter = $("filter").value.toLowerCase();
  const elements = [];
  let folder = "";
  for (const file of allFiles.filter((f) => f.toLowerCase().includes(filter))) {
    const here = folderOf(file);
    if (here !== folder) {
      folder = here;
      elements.push(Object.assign(document.createElement("li"), { className: "folder", textContent: here.replace(/^\/exemples\//, "").replace(/\/$/, "") || "exemples" }));
    }
    const link = Object.assign(document.createElement("a"), { href: `?file=${encodeURIComponent(file)}`, textContent: file.split("/").pop() });
    if (file === path) link.setAttribute("aria-current", "page");
    link.addEventListener("click", (e) => {
      e.preventDefault();
      showFiles(false);
      openFile(file);
    });
    const li = document.createElement("li");
    li.append(link);
    elements.push(li);
  }
  $("list").replaceChildren(...elements);
}

$("open-files").addEventListener("click", () => showFiles($("files").hidden));
$("close-files").addEventListener("click", () => showFiles(false));
$("filter").addEventListener("input", listFiles);
addEventListener("keydown", (e) => { if (e.key === "Escape" && !$("files").hidden) showFiles(false); });
$("new").addEventListener("submit", (e) => {
  e.preventDefault();
  let name = $("new-name").value.trim();
  if (!name) return;
  if (!name.startsWith("/")) name = `/exemples/${name.replace(/^exemples\//, "")}`;
  if (!name.endsWith(".holo")) name += ".holo";
  showFiles(false);
  openFile(name, { newFile: !allFiles.includes(name) });
});

// ---------------------------------------------------------------- départ

try {
  await init();
  vocab = JSON.parse(vocabulary());
  if (!key) flag("Sans la clé du serveur, l'éditeur ne peut pas enregistrer : on lit et on essaie.");
  await openFile(path);
  area.focus();
} catch (e) {
  state.className = "invalid";
  state.textContent = `Le moteur n'a pas pu démarrer : ${e?.message ?? e}`;
}
