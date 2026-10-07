// Ce qu'une page télécharge en sons à l'ouverture, sans aucun geste (serveur de Yocthan, lecture seule).
import { openChrome, pause } from "./cdp.mjs";
const c = await openChrome();
const out = {};
try {
  await c.ask("Network.enable"); await c.ask("Page.enable");
  await c.ask("Network.setCacheDisabled", { cacheDisabled: true });
  let req = new Map();
  c.on("Network.requestWillBeSent", (p) => req.set(p.requestId, { url: p.request.url, type: p.type, bytes: 0 }));
  c.on("Network.loadingFinished", (p) => { const r = req.get(p.requestId); if (r) r.bytes = p.encodedDataLength; });
  await c.ask("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 3, mobile: true });
  for (const page of ["/exemples/site-reference/jeu.holo", "/exemples/lecons/79-regler-un-son.holo", "/exemples/lecons/58-lecteur-de-son.holo"]) {
    req = new Map();
    await c.ask("Page.navigate", { url: "http://localhost:8080" + page });
    await pause(5000);
    const sons = [...req.values()].filter((r) => /\.(wav|mp3|ogg)$/.test(r.url));
    out[page] = { sonsDemandes: sons.map((s) => `${s.url.split("/").pop()} : ${s.bytes} octets`), totalSons: sons.reduce((a, s) => a + s.bytes, 0) };
  }
} finally { await c.close(); }
console.log(JSON.stringify(out, null, 1));
