// La correction d'un clic (ADR-046), partagée par l'éditeur du navigateur et par l'extension
// VS Code. Le moteur reste strict : il refuse la faute et, quand il le sait, dit le bon mot
// (« écris « H1 » »). Ici, on lit sa réponse, on retrouve la faute dans le texte, et l'on
// propose le remplacement. Rien n'est jamais corrigé sans un clic.

// « ligne 7, colonne 5 : message ». La colonne du moteur compte des octets ; une lettre accentuée
// en vaut deux : on la ramène en lettres. Rend la faute avec sa place dans le texte.
export function lireFaute(reponse, texte) {
  const m = /^ligne (\d+), colonne (\d+) : ([\s\S]*)$/.exec(reponse);
  if (!m) return { ligne: 1, debut: 0, fin: Math.min(1, texte.length), message: reponse };
  const ligne = Number(m[1]);
  const lignes = texte.split("\n");
  const debutDeLigne = lignes.slice(0, ligne - 1).reduce((total, l) => total + l.length + 1, 0);
  const contenu = lignes[ligne - 1] ?? "";
  const encodeur = new TextEncoder();
  let octets = 0;
  let colonne = 0;
  for (const c of contenu) {
    if (octets >= Number(m[2]) - 1) break;
    octets += encodeur.encode(c).length;
    colonne += c.length;
  }
  const debut = Math.min(debutDeLigne + colonne, texte.length);
  let fin = debut;
  if (texte[debut] === '"') {
    const finDeLigne = texte.indexOf("\n", debut) < 0 ? texte.length : texte.indexOf("\n", debut);
    const fermeture = texte.indexOf('"', debut + 1);
    fin = fermeture > debut && fermeture < finDeLigne ? fermeture + 1 : debut + 1;
  } else {
    while (fin < texte.length && /[A-Za-z0-9_.\-]/.test(texte[fin])) fin++;
  }
  if (fin === debut) fin = Math.min(debut + 1, texte.length);
  return { ligne, debut, fin, message: m[3] };
}

// `apple_x` → `appleX`, comme le moteur.
export const enFlutter = (mot) => mot.replace(/_+([a-zA-Z0-9])/g, (_, c) => c.toUpperCase()).replace(/_/g, "");

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
export function lePlusProche(mot, candidats) {
  let meilleur = null;
  let ecart = Infinity;
  for (const c of new Set(candidats)) {
    const e = distance(mot.toLowerCase(), c.toLowerCase());
    if (e < ecart) [meilleur, ecart] = [c, e];
  }
  const permis = Math.min(3, Math.max(1, Math.floor(mot.length / 3)));
  return ecart <= permis && meilleur !== mot ? meilleur : null;
}

// Trouve le mot, entier, à partir d'une position.
function chercherLeMot(texte, mot, depuis) {
  const re = new RegExp(`(?<![A-Za-z0-9_])${mot.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}(?![A-Za-z0-9_])`, "g");
  re.lastIndex = Math.max(0, depuis);
  const m = re.exec(texte);
  return m && m.index < depuis + 6000 ? m.index : -1;
}

// Ce que le fichier déclare : ses valeurs (State), les noms de ses blocs.
export function valeursDeclarees(texte) {
  const etat = /State\s*\(([\s\S]*?)\)\s*,?\s*(?:\n|[a-z]+:)/.exec(texte);
  if (!etat) return [];
  return [...etat[1].matchAll(/([a-z][A-Za-z0-9]*)\s*:/g)].map((m) => m[1]);
}
export const nomsDeBlocs = (texte) => [...texte.matchAll(/name:\s*([A-Z][A-Za-z0-9]*)/g)].map((m) => m[1]);

// La correction d'une faute, quand il y en a une : { debut, fin, par, libelle }, ou null.
// `vocab` : les mots du langage donnés par le moteur (`vocabulaire()`).
export function correctionPour(f, texte, vocab) {
  const message = f.message;
  const dit = /écris « ([^»]+?) »/.exec(message) ?? /s'écrit « ([^»]+?) »/.exec(message) ?? /le premier titre est « (H1) »/.exec(message);
  if (dit) {
    let par = dit[1];
    // {apple_x} dans un texte → {appleX}
    if (/^\{[A-Za-z0-9]+\}$/.test(par)) {
      const re = /\{([A-Za-z0-9_]+)\}/g;
      re.lastIndex = f.debut;
      for (let m = re.exec(texte); m && m.index < f.debut + 6000; m = re.exec(texte)) {
        if (m[1] !== par.slice(1, -1) && enFlutter(m[1]) === par.slice(1, -1)) return { debut: m.index, fin: m.index + m[0].length, par, libelle: `Remplacer « ${m[0]} » par « ${par} »` };
      }
      return null;
    }
    // name: buy → name: Buy
    const reglage = /^([a-z]+): ([A-Za-z0-9_]+)$/.exec(par);
    if (reglage) {
      const re = new RegExp(`${reglage[1]}\\s*:\\s*([A-Za-z0-9_]+)`, "y");
      re.lastIndex = f.debut;
      const m = re.exec(texte);
      if (!m) return null;
      const debut = f.debut + m[0].length - m[1].length;
      return { debut, fin: debut + m[1].length, par: reglage[2], libelle: `Remplacer « ${m[1]} » par « ${reglage[2]} »` };
    }
    if (par.includes(" { … }")) par = par.split(" {")[0];
    if (!/^[A-Za-z0-9_.\-]+$/.test(par)) return null;
    const avant = texte.slice(f.debut, f.fin);
    return avant && avant !== par ? { debut: f.debut, fin: f.fin, par, libelle: `Remplacer « ${avant} » par « ${par} »` } : null;
  }
  // Un mot inconnu : le mot connu le plus proche, s'il y en a un.
  const inconnu = /(?:bloc inconnu|réglage inconnu|aucune valeur ne s'appelle|aucun bloc ne s'appelle|aucun nombre ne s'appelle|aucune liste ne s'appelle|n'a pas de paramètre|demande inconnue|signal inconnu|capacité inconnue|touche inconnue) « ([^»]+) »/.exec(message);
  if (!inconnu) return null;
  const mot = inconnu[1].replace(/^\{|\}$/g, "");
  let candidats = [];
  const liste = /(?:possibles :|émet|offre|peut demander|le clavier donne) ([^;(]+)/.exec(message.slice(inconnu.index + inconnu[0].length));
  if (liste) candidats = liste[1].split(",").map((m) => m.trim()).filter(Boolean);
  else if (/^bloc inconnu/.test(inconnu[0])) candidats = vocab?.blocs ?? [];
  else if (/aucun bloc/.test(inconnu[0])) candidats = nomsDeBlocs(texte);
  else candidats = [...valeursDeclarees(texte), ...(vocab?.calculees ?? [])];
  const proche = lePlusProche(mot, candidats);
  if (!proche) return null;
  const debut = texte.slice(f.debut, f.fin) === mot ? f.debut : chercherLeMot(texte, mot, f.debut);
  return debut < 0 ? null : { debut, fin: debut + mot.length, par: proche, libelle: `Remplacer « ${mot} » par « ${proche} »` };
}
