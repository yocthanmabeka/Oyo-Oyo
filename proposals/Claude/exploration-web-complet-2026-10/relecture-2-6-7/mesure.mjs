// Relecture : remesure les jumeaux HTML tels qu'ils sont écrits dans les fichiers Markdown
// (blocs extraits par extraire.mjs), et compare avec les fichiers d'essai de l'autre agent.
import { readFileSync } from "node:fs";
import { brotliCompressSync, constants } from "node:zlib";
import { join } from "node:path";

const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Z]:)/, "$1");
const essais = join(here, "..", "essais-2-6-7");
const br = (b, q) => brotliCompressSync(b, { params: { [constants.BROTLI_PARAM_QUALITY]: q } }).length;
for (const piste of ["02", "06", "07"]) {
  const md = readFileSync(join(here, `piste-${piste}-bloc-2.html`));
  const file = readFileSync(join(essais, `piste-${piste}-jumeau.html`));
  const script = /<script>([\s\S]*?)<\/script>/.exec(md.toString("utf8"))[1];
  const lines = script.split("\n").filter((l) => l.trim() && !l.trim().startsWith("//")).length;
  const allLines = script.split("\n").filter((l) => l.trim()).length;
  console.log(`piste ${piste} : bloc du Markdown ${md.length} octets (fichier d'essai ${file.length}, identiques : ${md.equals(file)}), Brotli 11 : ${br(md, 11)}, Brotli 5 : ${br(md, 5)}, lignes de JS sans commentaire seul : ${lines} (toutes les lignes non vides : ${allLines})`);
}
// La page HoloCode de la leçon 64, telle que le serveur l'envoie, est compressée en qualité 5 (server.mjs:392).
