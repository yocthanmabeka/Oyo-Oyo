// Relecture : chaque mot proposé, seul, doit être refusé ; sans eux, l'exemple doit passer.
// Part des blocs extraits des fichiers Markdown (extraire.mjs). Lecture seule du dépôt.
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { join } from "node:path";

const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Z]:)/, "$1");
const holo = "moteur/target/release/holo.exe";
const check = (code) => {
  try {
    return execFileSync(holo, ["check", "-"], { input: code, encoding: "utf8" }).trim();
  } catch (e) {
    return `${(e.stdout || "").trim()} ${(e.stderr || "").trim()}`.trim();
  }
};
// Retire les commentaires « // proposé … » pour comparer ligne à ligne.
const strip = (code) => code.replace(/[ \t]*\/\/ proposé[^\n]*/g, "");

// Chaque piste : les remplacements qui retirent un mot proposé. `all` les applique tous.
const pistes = {
  "02": [
    ["include", ", include: [lever, porte]", ""],
    ["required (nom)", 'label: "Votre nom", required: true, autofill: name', 'label: "Votre nom", autofill: name'],
    ["autofill (nom)", 'label: "Votre nom", autofill: name', 'label: "Votre nom"'],
    ["type email + required + autofill (courriel)", ', type: email, required: true, autofill: email', ""],
    ["required (Choice)", '"À l\'atelier"], required: true)', '"À l\'atelier"])'],
    ["If sur un texte", 'If(delivery, is: "À domicile"', 'If(delivery, not: ""'],
    ["required + autofill (adresse)", ", lines: 3, required: true, autofill: address", ", lines: 3"],
    ["announce (sent)", "If(sent, is: 1, announce: true", "If(sent, is: 1"],
    ["announce (failed)", "If(failed, is: 1, announce: true", "If(failed, is: 1"],
  ],
  "06": [
    ["toggles", ', toggles: SiteMenu', ""],
    ["Popover", "Popover(name: SiteMenu", "Column(name: SiteMenu"],
    ["Nav(label:)", 'Nav(label: "Menu principal", children', "Nav(children"],
    ["group (1)", '?", group: faq, children', '?", children'],
    ["group (2)", 'encadrés ?", group: faq, children', 'encadrés ?", children'],
    ["announce", "If(note, is: 1, announce: true", "If(note, is: 1"],
    ["When texte (Grand)", 'When(size, is: "Grand", effect: note.set(1)),', ""],
    ["When texte (Petit)", 'When(size, is: "Petit", effect: note.set(0)),', ""],
    ["closed", "On(News.closed, effect: asked.set(1)),", ""],
  ],
  "07": [
    ["image: Image(…)", 'image: Image(source: "lever-partage.png", alt: "Un soleil jaune se lève au-dessus d\'un fleuve bleu"),', 'image: "lever-partage.png",'],
    ["query", "query: [format],", ""],
    ["Nav(label:) en-tête", 'Nav(label: "Menu principal", children', "Nav(children"],
    ["Nav(label:) pied", 'Nav(label: "Informations", children', "Nav(children"],
    ["rowHeads, over, cells", '"Prix"], rowHeads: true,\n        over: formats, cells: ["{item.title}", "{item.size}", "{item.price:cents} €"]),', '"Prix"],\n        rows: [ ["Petit", "20 × 12 cm", "60,00 €"] ]),'],
  ],
};

for (const [piste, changes] of Object.entries(pistes)) {
  const original = strip(readFileSync(join(here, `piste-${piste}-bloc-1.holo`), "utf8"));
  let all = original;
  for (const [, before, after] of changes) {
    if (!all.includes(before)) console.log(`  (piste ${piste} : texte introuvable : ${before})`);
    all = all.replace(before, after);
  }
  console.log(`piste ${piste}, tous les mots proposés retirés → ${check(all)}`);
  // Chaque mot proposé, seul : les autres retirés.
  changes.forEach(([label], i) => {
    let one = original;
    changes.forEach(([, before, after], j) => { if (j !== i) one = one.replace(before, after); });
    // Les retraits qui se recouvrent (required puis autofill sur la même ligne) : on garde le mot visé.
    console.log(`piste ${piste}, seul « ${label} » → ${check(one)}`);
  });
}
