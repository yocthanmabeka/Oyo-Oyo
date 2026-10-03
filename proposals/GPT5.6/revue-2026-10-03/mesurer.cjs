// Mesure les corps Brotli, pas un transfert HTTP ni le téléchargement total du site.
const fs = require('fs');
const path = require('path');
const z = require('zlib');
for (const file of ['holo_moteur_bg.wasm', 'holo_moteur.js']) {
  const b = fs.readFileSync(path.join(__dirname, '../../../moteur/web/pkg', file));
  const br = z.brotliCompressSync(b, {params: {[z.constants.BROTLI_PARAM_QUALITY]: 11}});
  console.log(JSON.stringify({file, raw_bytes: b.length, brotli_bytes: br.length}));
}
