// Relecture des pistes 2, 6, 7 : extrait les blocs ```holo et ```html des fichiers Markdown,
// les écrit à côté, et fait vérifier les blocs holo par `holo check -`. Lecture seule du dépôt.
import { readFileSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { join } from "node:path";

const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Z]:)/, "$1");
const exploration = join(here, "..");
const holo = "moteur/target/release/holo.exe";
for (const piste of ["02", "06", "07"]) {
  const md = readFileSync(join(exploration, `piste-${piste}.md`), "utf8");
  const blocks = [...md.matchAll(/```(holo|html)\n([\s\S]*?)```/g)];
  blocks.forEach(([, kind, code], i) => {
    const file = join(here, `piste-${piste}-bloc-${i + 1}.${kind}`);
    writeFileSync(file, code);
    if (kind !== "holo") return console.log(`${file} : ${code.length} caractères (html)`);
    let out;
    try {
      out = execFileSync(holo, ["check", "-"], { input: code, encoding: "utf8" }).trim();
    } catch (e) {
      out = `${(e.stdout || "").trim()} ${(e.stderr || "").trim()}`.trim();
    }
    console.log(`piste ${piste}, bloc holo ${i + 1} → ${out}`);
  });
}
