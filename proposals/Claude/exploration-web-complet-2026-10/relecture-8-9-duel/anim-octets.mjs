import { readFileSync } from "node:fs";
const html = readFileSync(process.argv[2], "utf8");
const bytes = (s) => Buffer.byteLength(s, "utf8");
const css = (html.match(/<style>([\s\S]*?)<\/style>/) || [, ""])[1];
// @keyframes hmN{...} : on compte les accolades
let kf = 0, kfCount = 0, rules = 0, scenesKf = 0;
for (let i = 0; i < css.length; ) {
  const k = css.indexOf("@keyframes ", i);
  if (k < 0) break;
  let depth = 0, j = css.indexOf("{", k);
  for (; j < css.length; j++) { if (css[j] === "{") depth++; else if (css[j] === "}") { depth--; if (depth === 0) break; } }
  const block = css.slice(k, j + 1);
  if (/^@keyframes hm\d/.test(block)) { kf += bytes(block); kfCount++; } else if (/^@keyframes hs\d/.test(block)) scenesKf += bytes(block);
  i = j + 1;
}
// règles .hmN{...} et .hmN .holo-letter{...} et .hmN>*>:nth-child(k){...}
const re = /\.hm\d+[^{]*\{[^}]*\}/g;
let m; while ((m = re.exec(css))) rules += bytes(m[0]);
console.log(JSON.stringify({ page: bytes(html), css: bytes(css), keyframesHm: kf, keyframesHmCount: kfCount, reglesHm: rules, total: kf + rules, keyframesScenes: scenesKf }));
