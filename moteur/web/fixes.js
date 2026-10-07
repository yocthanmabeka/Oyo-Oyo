// La correction d'un clic (ADR-046), partagée par l'éditeur du navigateur et par l'extension
// VS Code. Le moteur reste strict : il refuse la faute et, quand il le sait, dit le bon mot
// (« écris « H1 » »). Ici, on lit sa réponse, on retrouve la faute dans le texte, et l'on
// propose le remplacement. Rien n'est jamais corrigé sans un clic.

// « ligne 7, colonne 5 : message ». La colonne du moteur compte des octets ; une lettre accentuée
// en vaut deux : on la ramène en lettres. Rend la faute avec sa place dans le texte.
export function readFault(response, text) {
  const m = /^ligne (\d+), colonne (\d+) : ([\s\S]*)$/.exec(response);
  if (!m) return { line: 1, start: 0, end: Math.min(1, text.length), message: response };
  const line = Number(m[1]);
  const lines = text.split("\n");
  const lineStart = lines.slice(0, line - 1).reduce((total, l) => total + l.length + 1, 0);
  const content = lines[line - 1] ?? "";
  const encoder = new TextEncoder();
  let bytes = 0;
  let column = 0;
  for (const c of content) {
    if (bytes >= Number(m[2]) - 1) break;
    bytes += encoder.encode(c).length;
    column += c.length;
  }
  const start = Math.min(lineStart + column, text.length);
  let end = start;
  if (text[start] === '"') {
    const lineEnd = text.indexOf("\n", start) < 0 ? text.length : text.indexOf("\n", start);
    const closing = text.indexOf('"', start + 1);
    end = closing > start && closing < lineEnd ? closing + 1 : start + 1;
  } else {
    while (end < text.length && /[A-Za-z0-9_.\-]/.test(text[end])) end++;
  }
  if (end === start) end = Math.min(start + 1, text.length);
  return { line, start, end, message: m[3] };
}

// `apple_x` → `appleX`, comme le moteur.
export const inFlutter = (word) => word.replace(/_+([a-zA-Z0-9])/g, (_, c) => c.toUpperCase()).replace(/_/g, "");

function distance(a, b) {
  const d = Array.from({ length: a.length + 1 }, (_, i) => [i, ...Array(b.length).fill(0)]);
  for (let j = 1; j <= b.length; j++) d[0][j] = j;
  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
    }
  }
  return d[a.length][b.length];
}

// Le mot connu le plus proche d'un mot inconnu, s'il est assez proche pour être une faute de frappe.
export function theClosest(word, candidates) {
  let best = null;
  let gap = Infinity;
  for (const c of new Set(candidates)) {
    const e = distance(word.toLowerCase(), c.toLowerCase());
    if (e < gap) [best, gap] = [c, e];
  }
  const allowed = Math.min(3, Math.max(1, Math.floor(word.length / 3)));
  return gap <= allowed && best !== word ? best : null;
}

// Trouve le mot, entier, à partir d'une position.
function searchWord(text, word, since) {
  const re = new RegExp(`(?<![A-Za-z0-9_])${word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}(?![A-Za-z0-9_])`, "g");
  re.lastIndex = Math.max(0, since);
  const m = re.exec(text);
  return m && m.index < since + 6000 ? m.index : -1;
}

// Ce que le fichier déclare : ses valeurs (State), les noms de ses blocs.
export function declaredValues(text) {
  const state = /State\s*\(([\s\S]*?)\)\s*,?\s*(?:\n|[a-z]+:)/.exec(text);
  if (!state) return [];
  return [...state[1].matchAll(/([a-z][A-Za-z0-9]*)\s*:/g)].map((m) => m[1]);
}
export const blockNames = (text) => [...text.matchAll(/name:\s*([A-Z][A-Za-z0-9]*)/g)].map((m) => m[1]);

// La correction d'une faute, quand il y en a une : { debut, fin, par, libelle }, ou null.
// `vocab` : les mots du langage donnés par le moteur (`vocabulaire()`).
export function fixFor(f, text, vocab) {
  const message = f.message;
  const says = /écris « ([^»]+?) »/.exec(message) ?? /s'écrit « ([^»]+?) »/.exec(message) ?? /le premier titre est « (H1) »/.exec(message);
  if (says) {
    let by = says[1];
    // {apple_x} dans un texte → {appleX}
    if (/^\{[A-Za-z0-9]+\}$/.test(by)) {
      const re = /\{([A-Za-z0-9_]+)\}/g;
      re.lastIndex = f.start;
      for (let m = re.exec(text); m && m.index < f.start + 6000; m = re.exec(text)) {
        if (m[1] !== by.slice(1, -1) && inFlutter(m[1]) === by.slice(1, -1)) return { start: m.index, end: m.index + m[0].length, by, label: `Remplacer « ${m[0]} » par « ${by} »` };
      }
      return null;
    }
    // name: buy → name: Buy
    const setting = /^([a-z]+): ([A-Za-z0-9_]+)$/.exec(by);
    if (setting) {
      const re = new RegExp(`${setting[1]}\\s*:\\s*([A-Za-z0-9_]+)`, "y");
      re.lastIndex = f.start;
      const m = re.exec(text);
      if (!m) return null;
      const start = f.start + m[0].length - m[1].length;
      return { start, end: start + m[1].length, by: setting[2], label: `Remplacer « ${m[1]} » par « ${setting[2]} »` };
    }
    if (by.includes(" { … }")) by = by.split(" {")[0];
    if (!/^[A-Za-z0-9_.\-]+$/.test(by)) return null;
    const before = text.slice(f.start, f.end);
    return before && before !== by ? { start: f.start, end: f.end, by, label: `Remplacer « ${before} » par « ${by} »` } : null;
  }
  // Un mot inconnu : le mot connu le plus proche, s'il y en a un.
  const unknown = /(?:bloc inconnu|réglage inconnu|aucune valeur ne s'appelle|aucun bloc ne s'appelle|aucun nombre ne s'appelle|aucune liste ne s'appelle|n'a pas de paramètre|demande inconnue|signal inconnu|capacité inconnue|touche inconnue) « ([^»]+) »/.exec(message);
  if (!unknown) return null;
  const word = unknown[1].replace(/^\{|\}$/g, "");
  let candidates = [];
  const list = /(?:possibles :|émet|offre|peut demander|le clavier donne) ([^;(]+)/.exec(message.slice(unknown.index + unknown[0].length));
  if (list) candidates = list[1].split(",").map((m) => m.trim()).filter(Boolean);
  else if (/^bloc inconnu/.test(unknown[0])) candidates = vocab?.blocks ?? [];
  else if (/aucun bloc/.test(unknown[0])) candidates = blockNames(text);
  else candidates = [...declaredValues(text), ...(vocab?.computed ?? [])];
  const closest = theClosest(word, candidates);
  if (!closest) return null;
  const start = text.slice(f.start, f.end) === word ? f.start : searchWord(text, word, f.start);
  return start < 0 ? null : { start, end: start + word.length, by: closest, label: `Remplacer « ${word} » par « ${closest} »` };
}
