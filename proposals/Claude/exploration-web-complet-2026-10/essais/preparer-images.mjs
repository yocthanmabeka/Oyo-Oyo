// Prépare douze « photos » d'essai à partir de captures du journal (lues, jamais modifiées) :
// la copie PNG d'origine, puis des versions WebP de 1280, 640 et 320 pixels de large,
// encodées par Chrome lui-même (canvas.toBlob). Écrit dans site/exemples/media/.
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { openChrome } from "./cdp.mjs";

const repo = "C:/Users/mokea/Documents/IA_creation/Metaverse";
const out = join(new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1"), "site", "exemples", "media");
mkdirSync(out, { recursive: true });
const sources = [
  "2026-10-04-motion-web-big-bang.png", "2026-10-03-pixel-2-dedans.png", "2026-10-04-motion-web-cube.png",
  "2026-10-03-mosaique-5-lettres.png", "3-plongee-apercu-du-monde-interieur.png", "1-point-entier.png",
  "2026-10-06-pile.png", "2026-10-06-motion-jumeaux-1.png", "4-entre-profondeur-2.png",
  "2026-10-04-tourner-depuis-la-page.png", "2-morcellement.png", "5-boule-visee-mesures-cachees.png",
];
const c = await openChrome();
const table = [];
try {
  for (const [i, name] of sources.entries()) {
    const nn = String(i + 1).padStart(2, "0");
    const png = readFileSync(join(repo, "docs/06-journal/images", name));
    writeFileSync(join(out, `photo-${nn}.png`), png);
    const data = `data:image/png;base64,${png.toString("base64")}`;
    const result = await c.evaluate(`(async () => {
      const img = new Image(); img.src = ${JSON.stringify(data)}; await img.decode();
      const out = { w: img.naturalWidth, h: img.naturalHeight, v: {} };
      for (const width of [1280, 640, 320]) {
        const w = Math.min(width, img.naturalWidth), h = Math.round(img.naturalHeight * w / img.naturalWidth);
        const cv = new OffscreenCanvas(w, h); cv.getContext("2d").drawImage(img, 0, 0, w, h);
        const blob = await cv.convertToBlob({ type: "image/webp", quality: 0.8 });
        const bytes = new Uint8Array(await blob.arrayBuffer());
        let s = ""; for (let k = 0; k < bytes.length; k += 32768) s += String.fromCharCode(...bytes.subarray(k, k + 32768));
        out.v[width] = { w, h, b64: btoa(s), type: blob.type };
      }
      return out;
    })()`);
    const line = { photo: nn, source: name, w: result.w, h: result.h, png: png.length };
    for (const [width, v] of Object.entries(result.v)) {
      const buf = Buffer.from(v.b64, "base64");
      writeFileSync(join(out, `photo-${nn}-${width}.webp`), buf);
      line[`webp${width}`] = `${buf.length} (${v.w}x${v.h}, ${v.type})`;
    }
    table.push(line);
  }
} finally {
  await c.close();
}
console.table(table);
const sum = (k) => table.reduce((s, l) => s + (typeof l[k] === "number" ? l[k] : Number(String(l[k]).split(" ")[0])), 0);
console.log(`Total PNG : ${sum("png")} octets ; WebP 1280 : ${sum("webp1280")} ; WebP 640 : ${sum("webp640")} ; WebP 320 : ${sum("webp320")}`);
