// Relecture : chaque ligne d'un tableau Markdown a-t-elle autant de cases que son en-tête ?
// (une barre « | » non échappée, même dans un `code`, coupe une case en GFM)
import { readFileSync } from "node:fs";
import { join } from "node:path";

const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Z]:)/, "$1");
const cells = (line) => line.replace(/\\\|/g, "").split("|").length - 2;
for (const piste of ["02", "06", "07"]) {
  const lines = readFileSync(join(here, "..", `piste-${piste}.md`), "utf8").split("\n");
  let header = null;
  lines.forEach((line, i) => {
    if (!line.startsWith("|")) { header = null; return; }
    if (header === null) { header = cells(line); return; }
    if (cells(line) !== header) console.log(`piste ${piste}, ligne ${i + 1} : ${cells(line)} cases au lieu de ${header} : ${line.slice(0, 90)}…`);
  });
}
console.log("fin de la vérification des tableaux");
