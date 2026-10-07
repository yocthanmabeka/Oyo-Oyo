// Mesure des jumeaux HTML : octets bruts et compressés en Brotli (qualité 11, comme server.mjs:90),
// et vérification de la syntaxe de leur script. Lecture seule.
import { readFileSync, writeFileSync } from "node:fs";
import { brotliCompressSync, constants } from "node:zlib";
import { execFileSync } from "node:child_process";
for (const file of ["piste-02-jumeau.html", "piste-06-jumeau.html", "piste-07-jumeau.html"]) {
  const raw = readFileSync(file);
  const br = brotliCompressSync(raw, { params: { [constants.BROTLI_PARAM_QUALITY]: 11 } });
  const script = /<script>([\s\S]*?)<\/script>/.exec(raw.toString("utf8"))[1];
  writeFileSync(`${file}.js`, script);
  let syntax = "ok";
  try { execFileSync("node", ["--check", `${file}.js`], { stdio: "pipe" }); } catch (e) { syntax = String(e.stderr).slice(0, 200); }
  const lines = script.split("\n").filter((l) => l.trim() && !l.trim().startsWith("//")).length;
  console.log(`${file} : ${raw.length} octets bruts, ${br.length} octets en Brotli, ${lines} lignes de JavaScript, syntaxe du script : ${syntax}`);
}
