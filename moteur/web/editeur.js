// L'éditeur HoloCode (ADR-046). Le même moteur que celui des pages : il vérifie le texte à
// chaque pause de l'écriture, et fabrique l'aperçu. L'éditeur ne décide de rien sur le langage :
// il montre la faute que le moteur refuse, et, quand le moteur dit le bon mot (« écris « H1 » »),
// il propose de le mettre d'un clic. Le moteur reste strict (ADR-037).

import init, { verifier_texte, vocabulaire, vue_a_plat, imports, regler_maintenant } from "/pkg-leger/holo_moteur.js";
import { lireFaute, correctionPour, valeursDeclarees, nomsDeBlocs } from "/corrections.js";

const $ = (id) => document.getElementById(id);
const zone = $("code");
const dessin = $("dessin");
const numeros = $("numeros");
const apercu = $("apercu");
const etat = $("etat");
const corriger = $("corriger");
const mots = $("mots");
const ligneFautive = $("ligne-fautive");

const params = new URLSearchParams(location.search);
const cle = params.get("cle") ?? "";
let chemin = params.get("fichier") ?? "/exemples/lecons/01-page.holo";
let enregistre = ""; // le texte tel qu'il est sur le disque
let finsDeLigneWindows = false; // le fichier était écrit avec \r\n : on le garde ainsi
let faute = null; // { ligne, debut, fin, message }
let correction = null; // { debut, fin, par, libelle }
let vocab = { blocs: [], parametres: {}, reglages: [], etats: [], demandes: [], signaux: [], capacites: [], touches: [], mots: [], calculees: [], formats: [] };

const FICHIER_SUIVANT = "\u001e";
const SEPARE_LE_NOM = "\u001f";
const MODELE = 'Page(\n  title: "Ma page",\n  children: [\n    H1("Ma page"),\n    "Écris ici.",\n  ],\n)\n';
const dossierDe = (c) => c.slice(0, c.lastIndexOf("/") + 1);
const echapper = (t) => t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const hauteurDeLigne = () => parseFloat(getComputedStyle(zone).lineHeight) || 22.4;
const hautDuTexte = () => parseFloat(getComputedStyle(zone).paddingTop) || 12;

// ---------------------------------------------------------------- le fichier

async function ouvrirFichier(nouveau, { nouveauFichier = false } = {}) {
  if (zone.value !== enregistre && !confirm("Le fichier ouvert n'est pas enregistré. L'abandonner ?")) return;
  chemin = nouveau;
  history.replaceState(null, "", `?fichier=${encodeURIComponent(chemin)}${cle ? `&cle=${encodeURIComponent(cle)}` : ""}`);
  let texte = MODELE;
  enregistre = "";
  if (!nouveauFichier) {
    try {
      const reponse = await fetch(chemin, { headers: { accept: "text/plain" }, cache: "no-store" });
      if (reponse.ok) {
        texte = await reponse.text();
        enregistre = texte.replace(/\r\n/g, "\n");
      }
    } catch { /* pas de serveur : on part du modèle */ }
  }
  finsDeLigneWindows = texte.includes("\r\n");
  zone.value = texte.replace(/\r\n/g, "\n");
  zone.scrollTop = 0;
  importes.clear();
  $("chemin").textContent = chemin;
  document.title = `${chemin.split("/").pop()} — éditeur HoloCode`;
  $("ouvrir").href = chemin;
  majModifie();
  dessiner();
  verifierBientot(0);
}

function majModifie() {
  $("modifie").hidden = zone.value === enregistre;
}

async function enregistrer() {
  if (!cle) {
    signaler("Pour enregistrer, ouvre l'éditeur par l'adresse que le serveur affiche à son démarrage : elle contient la clé.");
    return;
  }
  const texte = finsDeLigneWindows ? zone.value.replace(/\n/g, "\r\n") : zone.value;
  try {
    const reponse = await fetch(chemin, { method: "PUT", headers: { "content-type": "text/plain; charset=utf-8", "x-holo-cle": cle }, body: texte });
    if (!reponse.ok) return signaler(`Pas enregistré : ${await reponse.text()}`);
    enregistre = zone.value;
    majModifie();
    signaler("Enregistré.");
  } catch {
    signaler("Pas enregistré : le serveur ne répond pas.");
  }
}

let messageTemporaire = null;
function signaler(texte) {
  messageTemporaire?.remove();
  messageTemporaire = Object.assign(document.createElement("div"), { className: "message-temporaire", role: "status", textContent: texte });
  document.body.append(messageTemporaire);
  setTimeout(() => messageTemporaire?.remove(), 3500);
}

// ---------------------------------------------------------------- vérifier, et montrer la page

// Les fichiers importés (import "commun.holo") sont lus à côté et joints au texte, comme le
// fait la page : le moteur ne lit rien tout seul.
const importes = new Map();
async function avecSesImports(texte) {
  let noms = [];
  try { noms = imports(texte).split(";").filter(Boolean); } catch { /* texte illisible : sans imports */ }
  let complet = texte;
  for (const nom of noms) {
    const adresse = dossierDe(chemin) + nom;
    if (!importes.has(adresse)) {
      importes.set(adresse, fetch(adresse, { headers: { accept: "text/plain" } }).then((r) => (r.ok ? r.text() : null)).catch(() => null));
    }
    const contenu = await importes.get(adresse);
    if (contenu !== null) complet += FICHIER_SUIVANT + nom + SEPARE_LE_NOM + contenu;
  }
  return complet;
}

let minuterie = 0;
let tour = 0;
function verifierBientot(ms = 250) {
  clearTimeout(minuterie);
  minuterie = setTimeout(verifier, ms);
}

async function verifier() {
  const monTour = ++tour;
  const texte = zone.value;
  const complet = await avecSesImports(texte);
  if (monTour !== tour) return; // on a écrit entre-temps
  const d = new Date();
  regler_maintenant(d.getFullYear(), d.getMonth() + 1, d.getDate(), ((d.getDay() + 6) % 7) + 1, d.getHours(), d.getMinutes());
  const reponse = verifier_texte(complet);
  faute = reponse.startsWith("ok") ? null : lireFaute(reponse, texte);
  correction = faute ? correctionPour(faute, texte, vocab) : null;
  dessiner();
  montrerEtat(reponse);
  if (!faute) montrerApercu(complet);
  $("apercu-ancien").hidden = !faute;
}

// ---------------------------------------------------------------- la correction d'un clic

function appliquerLaCorrection() {
  if (!correction) return;
  zone.focus();
  remplacer(correction.debut, correction.fin, correction.par);
  verifierBientot(0);
}

// Remplace un morceau du texte en gardant l'annulation (Ctrl + Z) du navigateur.
function remplacer(debut, fin, par) {
  zone.setSelectionRange(debut, fin);
  if (!document.execCommand("insertText", false, par)) zone.setRangeText(par, debut, fin, "end");
  apresLaFrappe();
}

// ---------------------------------------------------------------- ce que le fichier déclare

const nomsDeStyles = (texte) => [...texte.matchAll(/[A-Z][A-Za-z0-9]*\.([a-z][A-Za-z0-9]*)\s*\(/g)].map((m) => m[1]);
const variablesDeclarees = (texte) => [...texte.matchAll(/(--[a-z][a-z0-9-]*)\s*:/g)].map((m) => m[1]);

// ---------------------------------------------------------------- les couleurs du texte

// Où finit le bloc racine : après lui viennent les styles.
function finDeLaRacine(t) {
  let i = 0;
  let profondeur = 0;
  let commence = false;
  while (i < t.length) {
    const c = t[i];
    if (c === "/" && t[i + 1] === "/") {
      const fin = t.indexOf("\n", i);
      if (fin < 0) return t.length;
      i = fin;
      continue;
    }
    if (c === '"') {
      i = finDuTexte(t, i);
      continue;
    }
    if (c === "(" || c === "[") {
      profondeur++;
      commence = true;
    } else if ((c === ")" || c === "]") && --profondeur === 0 && commence) return i + 1;
    i++;
  }
  return t.length;
}

function finDuTexte(t, i) {
  if (t.startsWith('"""', i)) {
    const fin = t.indexOf('"""', i + 3);
    return fin < 0 ? t.length : fin + 3;
  }
  const guillemet = t.indexOf('"', i + 1);
  const ligne = t.indexOf("\n", i + 1);
  if (guillemet < 0) return ligne < 0 ? t.length : ligne;
  return ligne >= 0 && ligne < guillemet ? ligne : guillemet + 1;
}

function jetons(t) {
  const sortie = [];
  const ajouter = (debut, fin, sorte) => { if (fin > debut) sortie.push([debut, fin, sorte]); };
  const racine = finDeLaRacine(t);
  const debutDeLigne = (i) => /(^|\n)\s*$/.test(t.slice(Math.max(0, i - 40), i));
  let i = 0;
  while (i < racine) {
    const c = t[i];
    if (c === "/" && t[i + 1] === "/") {
      const fin = t.indexOf("\n", i) < 0 ? t.length : t.indexOf("\n", i);
      ajouter(i, fin, "commentaire");
      i = fin;
      continue;
    }
    if (c === '"') {
      const fin = finDuTexte(t, i);
      const re = /\{[a-zA-Z][a-zA-Z0-9_.]*(?::[a-zA-Z0-9]+)?\}/g;
      re.lastIndex = i;
      let j = i;
      for (let m = re.exec(t); m && m.index + m[0].length <= fin; m = re.exec(t)) {
        ajouter(j, m.index, "texte");
        ajouter(m.index, m.index + m[0].length, "dans-texte");
        j = m.index + m[0].length;
      }
      ajouter(j, fin, "texte");
      i = fin;
      continue;
    }
    if (/[0-9-]/.test(c) && !/[A-Za-z0-9_]/.test(t[i - 1] ?? "")) {
      const nombre = /^-?\d+(?:\.\d+)?(?:px|ms|min|deg|KB|MB|GB|B|mm|cm|km|s|h|m)?(?![A-Za-z0-9_])/.exec(t.slice(i, i + 40));
      if (nombre) {
        ajouter(i, i + nombre[0].length, "nombre");
        i += nombre[0].length;
        continue;
      }
    }
    const mot = /^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*/.exec(t.slice(i, i + 200));
    if (mot) {
      const fin = i + mot[0].length;
      let k = fin;
      while (t[k] === " " || t[k] === "\t") k++;
      let sorte;
      if (t[k] === "(") sorte = /^[A-Z]/.test(mot[0]) ? "bloc" : "demande";
      else if (t[k] === ":") sorte = "param";
      else if (mot[0].includes(".") && /^[A-Z]/.test(mot[0])) sorte = "signal";
      else if (["import", "module", "bridge"].includes(mot[0]) && debutDeLigne(i)) sorte = "mot-cle";
      else if (mot[0] === "true" || mot[0] === "false") sorte = "mot";
      else sorte = /^[A-Z]/.test(mot[0]) ? "nom" : "valeur";
      ajouter(i, fin, sorte);
      i = fin;
      continue;
    }
    if ("()[],:".includes(c)) ajouter(i, i + 1, "ponctuation");
    i++;
  }
  // Les styles, écrits comme en CSS.
  let profondeur = 0;
  while (i < t.length) {
    const c = t[i];
    if (c === "/" && t[i + 1] === "/") {
      const fin = t.indexOf("\n", i) < 0 ? t.length : t.indexOf("\n", i);
      ajouter(i, fin, "commentaire");
      i = fin;
      continue;
    }
    if (c === "{" || c === "}") {
      profondeur = Math.max(0, profondeur + (c === "{" ? 1 : -1));
      ajouter(i, i + 1, "ponctuation");
      i++;
      continue;
    }
    if (c === '"') {
      const fin = finDuTexte(t, i);
      ajouter(i, fin, "texte");
      i = fin;
      continue;
    }
    const couleur = /^#[0-9a-fA-F]{3,8}(?![0-9a-zA-Z])/.exec(t.slice(i, i + 10));
    const nombre = /^-?\d+(?:\.\d+)?(?:px|%|ms|s|deg)?(?![A-Za-z0-9_])/.exec(t.slice(i, i + 20));
    if (couleur || (nombre && !/[A-Za-z0-9_-]/.test(t[i - 1] ?? ""))) {
      const longueur = (couleur ?? nombre)[0].length;
      ajouter(i, i + longueur, "nombre");
      i += longueur;
      continue;
    }
    const mot = /^(?:--)?[A-Za-z_.][A-Za-z0-9_-]*/.exec(t.slice(i, i + 200));
    if (mot) {
      const fin = i + mot[0].length;
      let k = fin;
      while (t[k] === " ") k++;
      let sorte;
      if (profondeur === 0) sorte = mot[0].startsWith(".") ? "nom-de-style" : "bloc";
      else if (t[k] === ":") {
        let q = k + 1;
        while (t[q] === " ") q++;
        sorte = t[q] === "{" ? "etat" : mot[0].startsWith("--") ? "variable" : "param";
      } else sorte = mot[0].startsWith("--") ? "variable" : "valeur";
      ajouter(i, fin, sorte);
      i = fin;
      continue;
    }
    if (":;,()".includes(c)) ajouter(i, i + 1, "ponctuation");
    i++;
  }
  return sortie;
}

function colorer(t) {
  const marque = faute && faute.fin > faute.debut ? [faute.debut, faute.fin] : null;
  const morceau = (a, b, sorte) => {
    if (b <= a) return "";
    const x = echapper(t.slice(a, b));
    return sorte ? `<span class="j-${sorte}">${x}</span>` : x;
  };
  const avecLaFaute = (a, b, sorte) => {
    if (!marque || b <= marque[0] || a >= marque[1]) return morceau(a, b, sorte);
    const [x, y] = [Math.max(a, marque[0]), Math.min(b, marque[1])];
    return morceau(a, x, sorte) + `<mark class="faute">${morceau(x, y, sorte)}</mark>` + morceau(y, b, sorte);
  };
  let html = "";
  let position = 0;
  for (const [debut, fin, sorte] of jetons(t)) {
    html += avecLaFaute(position, debut, null) + avecLaFaute(debut, fin, sorte);
    position = fin;
  }
  return html + avecLaFaute(position, t.length, null) + "\n";
}

function dessiner() {
  dessin.innerHTML = colorer(zone.value);
  const lignes = zone.value.split("\n").length;
  if (numeros.dataset.lignes !== String(lignes)) {
    numeros.textContent = Array.from({ length: lignes }, (_, i) => i + 1).join("\n");
    numeros.dataset.lignes = String(lignes);
  }
  suivreLeDefilement();
}

function suivreLeDefilement() {
  dessin.scrollTop = zone.scrollTop;
  dessin.scrollLeft = zone.scrollLeft;
  numeros.scrollTop = zone.scrollTop;
  ligneFautive.hidden = !faute;
  if (faute) ligneFautive.style.top = `${hautDuTexte() + (faute.ligne - 1) * hauteurDeLigne() - zone.scrollTop}px`;
}

// ---------------------------------------------------------------- l'état, et l'aperçu

function montrerEtat(reponse) {
  corriger.hidden = !correction;
  if (correction) corriger.textContent = correction.libelle;
  if (!faute) {
    etat.className = "juste";
    etat.textContent = reponse === "ok" ? "✓ Le fichier est juste." : "✓ Ce morceau est juste ; il se vérifie aussi dans la page qui l'importe.";
    return;
  }
  etat.className = "fausse";
  etat.textContent = `Ligne ${faute.ligne} : ${faute.message}`;
  etat.title = "Aller à la faute";
}

function allerALaFaute() {
  if (!faute) return;
  document.body.classList.remove("voir-apercu");
  majOnglets();
  zone.focus();
  zone.setSelectionRange(faute.debut, faute.fin);
  zone.scrollTop = Math.max(0, (faute.ligne - 1) * hauteurDeLigne() - zone.clientHeight / 3);
  suivreLeDefilement();
}

let defilementDeLApercu = 0;
function montrerApercu(complet) {
  let html;
  try {
    html = vue_a_plat(complet, dossierDe(chemin), undefined);
  } catch {
    html = `<p style="font:16px system-ui;padding:24px">Ce fichier est un monde : il se regarde en profondeur. Touche « Ouvrir la page ».</p>`;
  }
  try { defilementDeLApercu = apercu.contentWindow?.scrollY ?? 0; } catch { /* pas encore de page */ }
  apercu.onload = () => { try { apercu.contentWindow.scrollTo(0, defilementDeLApercu); } catch { /* rien */ } };
  apercu.srcdoc = `<!doctype html><html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><style>html,body{margin:0}</style></head><body>${html}</body></html>`;
}

// ---------------------------------------------------------------- les mots à toucher

// Où l'on écrit : dans les réglages d'un bloc (le dernier `Nom(` encore ouvert), ou dans une
// liste (`children: [`), et laquelle.
function ouEstOn(t, pos) {
  const pile = [];
  let i = 0;
  while (i < pos) {
    const c = t[i];
    if (c === "/" && t[i + 1] === "/") {
      const fin = t.indexOf("\n", i);
      i = fin < 0 ? pos : fin;
      continue;
    }
    if (c === '"') {
      i = finDuTexte(t, i);
      continue;
    }
    if (c === "(") pile.push({ bloc: /([A-Za-z_][A-Za-z0-9_]*)(?:\.[A-Za-z0-9_]+)?\s*$/.exec(t.slice(Math.max(0, i - 60), i))?.[1] ?? "" });
    else if (c === "[") pile.push({ liste: /([a-z][A-Za-z0-9]*)\s*:\s*$/.exec(t.slice(Math.max(0, i - 40), i))?.[1] ?? "" });
    else if (c === ")" || c === "]") pile.pop();
    i++;
  }
  const dessus = pile.at(-1);
  return { bloc: [...pile].reverse().find((p) => p.bloc !== undefined)?.bloc ?? "", liste: dessus && dessus.liste !== undefined ? dessus.liste : null };
}

// Ce qu'on range dans chaque sorte de liste.
const CONTENU_DES_LISTES = { rules: ["On", "Every", "When", "After", "If"], fonts: ["Font"], modules: ["Module"], items: ["Item"], pixels: ["Point"] };

function dansUnTexte(t, pos) {
  let i = 0;
  while (i < pos) {
    if (t[i] === "/" && t[i + 1] === "/") {
      const fin = t.indexOf("\n", i);
      if (fin < 0 || fin >= pos) return false;
      i = fin;
      continue;
    }
    if (t[i] === '"') {
      const fin = finDuTexte(t, i);
      if (fin > pos || (fin === pos && t[fin - 1] !== '"')) return true;
      i = fin;
      continue;
    }
    i++;
  }
  return false;
}

let propositions = [];
let morceauEnCours = { debut: 0, fin: 0 };

function proposer() {
  const t = zone.value;
  const pos = zone.selectionStart;
  propositions = [];
  if (pos === zone.selectionEnd) {
    const avant = t.slice(0, pos);
    const racine = finDeLaRacine(t);
    if (pos > racine) propositions = dansLesStyles(t, pos, avant);
    else if (dansUnTexte(t, pos)) {
      const m = /\{([a-z][A-Za-z0-9]*)?$/.exec(avant);
      if (m) {
        morceauEnCours = { debut: pos - (m[1]?.length ?? 0), fin: pos };
        propositions = trier(m[1] ?? "", [...valeursDeclarees(t), ...vocab.calculees]).map((mot) => ({ mot, suite: "}" }));
      }
    } else propositions = dansLeCode(t, pos, avant);
  }
  mots.replaceChildren(...propositions.slice(0, 12).map(({ mot }, rang) => {
    const bouton = Object.assign(document.createElement("button"), { type: "button", textContent: mot });
    bouton.dataset.rang = rang;
    return bouton;
  }));
}

function dansLeCode(t, pos, avant) {
  const m = /([A-Za-z_][A-Za-z0-9_]*\.)?([A-Za-z_][A-Za-z0-9_]*)?$/.exec(avant);
  const [entier, prefixe = "", morceau = ""] = m;
  morceauEnCours = { debut: pos - morceau.length, fin: pos };
  // Après un nom et un point : `Ajouter.` → tap ; `panier.` → add.
  if (prefixe) {
    const base = prefixe.slice(0, -1);
    if (base === "Key") return trier(morceau, vocab.touches).map((mot) => ({ mot }));
    if (/^[A-Z]/.test(base)) return trier(morceau, [...vocab.signaux, ...vocab.capacites]).map((mot) => ({ mot }));
    return trier(morceau, vocab.demandes).map((mot) => ({ mot, suite: "(" }));
  }
  if (!entier && !/[(,[]\s*$/.test(avant)) return [];
  const precedent = /(\S)\s*$/.exec(avant.slice(0, pos - morceau.length))?.[1] ?? "";
  if (/^[A-Z]/.test(morceau)) {
    return trier(morceau, [...vocab.blocs, ...nomsDeBlocs(t), "Key"]).map((mot) => ({ mot, suite: vocab.blocs.includes(mot) ? "(" : "" }));
  }
  if (precedent === ":") {
    return trier(morceau, [...valeursDeclarees(t), ...vocab.calculees, ...vocab.mots, ...nomsDeBlocs(t)]).map((mot) => ({ mot }));
  }
  const { bloc, liste } = ouEstOn(t, pos);
  // Dans une liste : les blocs qu'on y range (keep: [ … ] prend des valeurs).
  if (liste !== null && liste !== "keep") {
    const blocs = CONTENU_DES_LISTES[liste] ?? vocab.blocs.filter((b) => !["Page", "State", "Prices", "Data", "Zoom", "Points", "Relief", "Portals", "Item", "Font", "Module", "On", "Every", "When", "After", "Scene"].includes(b));
    return trier(morceau, blocs).map((mot) => ({ mot, suite: "(" }));
  }
  if (liste === "keep") return trier(morceau, valeursDeclarees(t)).map((mot) => ({ mot }));
  const reglages = vocab.parametres[bloc] ?? [];
  return [
    ...trier(morceau, reglages).map((mot) => ({ mot, suite: ": " })),
    ...trier(morceau, [...valeursDeclarees(t), ...vocab.calculees]).filter((mot) => !reglages.includes(mot)).map((mot) => ({ mot })),
  ];
}

function dansLesStyles(t, pos, avant) {
  const styles = t.slice(finDeLaRacine(t), pos);
  const profondeur = [...styles].reduce((p, c) => (c === "{" ? p + 1 : c === "}" ? Math.max(0, p - 1) : p), 0);
  const m = /(--|\.)?([A-Za-z][A-Za-z0-9-]*)?$/.exec(avant);
  const [entier, signe = "", morceau = ""] = m;
  morceauEnCours = { debut: pos - entier.length, fin: pos };
  if (profondeur === 0) {
    return trier(entier, [...vocab.blocs, ...nomsDeStyles(t).map((n) => `.${n}`)]).map((mot) => ({ mot, suite: " { " }));
  }
  const precedent = /(\S)\s*$/.exec(avant.slice(0, pos - entier.length))?.[1] ?? "";
  if (precedent === ":") return trier(entier, [...variablesDeclarees(t), ...vocab.mots]).map((mot) => ({ mot, suite: ";" }));
  if (!entier && !/[{;]\s*$/.test(avant)) return [];
  return [
    ...trier(entier, vocab.reglages).map((mot) => ({ mot, suite: ": " })),
    ...trier(entier, vocab.etats).map((mot) => ({ mot, suite: ": { " })),
    ...(signe === "--" ? trier(entier, variablesDeclarees(t)).map((mot) => ({ mot, suite: ": " })) : []),
  ];
}

// Les mots qui commencent par ce qu'on a tapé d'abord, puis ceux qui le contiennent.
function trier(morceau, candidats) {
  const bas = morceau.toLowerCase();
  const uniques = [...new Set(candidats)].filter((c) => c && c !== morceau);
  const debut = uniques.filter((c) => c.toLowerCase().startsWith(bas));
  const dedans = bas ? uniques.filter((c) => !c.toLowerCase().startsWith(bas) && c.toLowerCase().includes(bas)) : [];
  return [...debut, ...dedans];
}

function prendre(rang) {
  const choix = propositions[rang];
  if (!choix) return;
  const t = zone.value;
  let suite = choix.suite ?? "";
  // Ne pas doubler ce qui suit déjà : « ( », « : », « } ».
  const apres = t.slice(morceauEnCours.fin).trimStart();
  if (suite && apres.startsWith(suite.trim())) suite = "";
  zone.focus();
  remplacer(morceauEnCours.debut, morceauEnCours.fin, choix.mot + suite);
}

// ---------------------------------------------------------------- la frappe

function apresLaFrappe() {
  majModifie();
  dessiner();
  proposer();
  verifierBientot();
}

zone.addEventListener("input", apresLaFrappe);
zone.addEventListener("scroll", suivreLeDefilement);
for (const nom of ["click", "keyup"]) zone.addEventListener(nom, (e) => { if (!(e.key && e.key.length === 1)) proposer(); });
zone.addEventListener("keydown", (e) => {
  if (e.isComposing) return;
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
    e.preventDefault();
    enregistrer();
    return;
  }
  // Tab prend le premier mot proposé ; sinon, deux espaces.
  if (e.key === "Tab" && !e.shiftKey) {
    e.preventDefault();
    if (propositions.length) prendre(0);
    else remplacer(zone.selectionStart, zone.selectionEnd, "  ");
    return;
  }
  if (e.key === "Escape") {
    propositions = [];
    mots.replaceChildren();
    return;
  }
  // À la ligne : le même retrait, deux espaces de plus après une parenthèse ou un crochet ouvert.
  if (e.key === "Enter" && !e.shiftKey && !e.ctrlKey && !e.altKey) {
    e.preventDefault();
    const pos = zone.selectionStart;
    const ligne = zone.value.slice(zone.value.lastIndexOf("\n", pos - 1) + 1, pos);
    const retrait = /^\s*/.exec(ligne)[0] + (/[([{]\s*$/.test(ligne) ? "  " : "");
    remplacer(pos, zone.selectionEnd, `\n${retrait}`);
  }
});
// Toucher un mot ne doit pas fermer le clavier du téléphone : on garde le focus dans le texte.
mots.addEventListener("pointerdown", (e) => { if (e.target.closest("button")) e.preventDefault(); });
mots.addEventListener("click", (e) => {
  const bouton = e.target.closest("button");
  if (bouton) prendre(Number(bouton.dataset.rang));
});
corriger.addEventListener("pointerdown", (e) => e.preventDefault());
corriger.addEventListener("click", appliquerLaCorrection);
etat.addEventListener("click", allerALaFaute);
$("enregistrer").addEventListener("click", enregistrer);
addEventListener("beforeunload", (e) => { if (zone.value !== enregistre) e.preventDefault(); });

// Sur un téléphone : Code ou Aperçu.
function majOnglets() {
  const apercuVisible = document.body.classList.contains("voir-apercu");
  $("voir-code").setAttribute("aria-pressed", String(!apercuVisible));
  $("voir-apercu").setAttribute("aria-pressed", String(apercuVisible));
}
$("voir-code").addEventListener("click", () => { document.body.classList.remove("voir-apercu"); majOnglets(); });
$("voir-apercu").addEventListener("click", () => { document.body.classList.add("voir-apercu"); majOnglets(); });

// ---------------------------------------------------------------- les fichiers

let tousLesFichiers = [];
async function montrerLesFichiers(ouvert) {
  $("fichiers").hidden = !ouvert;
  $("ouvrir-fichiers").setAttribute("aria-expanded", String(ouvert));
  if (!ouvert) return;
  try { tousLesFichiers = await (await fetch("/liste-holo", { cache: "no-store" })).json(); } catch { tousLesFichiers = []; }
  listerLesFichiers();
  $("filtre").focus();
}

function listerLesFichiers() {
  const filtre = $("filtre").value.toLowerCase();
  const elements = [];
  let dossier = "";
  for (const fichier of tousLesFichiers.filter((f) => f.toLowerCase().includes(filtre))) {
    const ici = dossierDe(fichier);
    if (ici !== dossier) {
      dossier = ici;
      elements.push(Object.assign(document.createElement("li"), { className: "dossier", textContent: ici.replace(/^\/exemples\//, "").replace(/\/$/, "") || "exemples" }));
    }
    const lien = Object.assign(document.createElement("a"), { href: `?fichier=${encodeURIComponent(fichier)}`, textContent: fichier.split("/").pop() });
    if (fichier === chemin) lien.setAttribute("aria-current", "page");
    lien.addEventListener("click", (e) => {
      e.preventDefault();
      montrerLesFichiers(false);
      ouvrirFichier(fichier);
    });
    const li = document.createElement("li");
    li.append(lien);
    elements.push(li);
  }
  $("liste").replaceChildren(...elements);
}

$("ouvrir-fichiers").addEventListener("click", () => montrerLesFichiers($("fichiers").hidden));
$("fermer-fichiers").addEventListener("click", () => montrerLesFichiers(false));
$("filtre").addEventListener("input", listerLesFichiers);
addEventListener("keydown", (e) => { if (e.key === "Escape" && !$("fichiers").hidden) montrerLesFichiers(false); });
$("nouveau").addEventListener("submit", (e) => {
  e.preventDefault();
  let nom = $("nouveau-nom").value.trim();
  if (!nom) return;
  if (!nom.startsWith("/")) nom = `/exemples/${nom.replace(/^exemples\//, "")}`;
  if (!nom.endsWith(".holo")) nom += ".holo";
  montrerLesFichiers(false);
  ouvrirFichier(nom, { nouveauFichier: !tousLesFichiers.includes(nom) });
});

// ---------------------------------------------------------------- départ

try {
  await init();
  vocab = JSON.parse(vocabulaire());
  if (!cle) signaler("Sans la clé du serveur, l'éditeur ne peut pas enregistrer : on lit et on essaie.");
  await ouvrirFichier(chemin);
  zone.focus();
} catch (e) {
  etat.className = "fausse";
  etat.textContent = `Le moteur n'a pas pu démarrer : ${e?.message ?? e}`;
}
