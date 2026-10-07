// Banc d'essai de l'exploration (pistes 8 et 9, duel du motion design).
//
//   node banc.mjs
//
// Lance un second serveur HoloCode (celui du dépôt, sans le modifier) sur le port 8093, dont les
// exemples sont ceux du dossier site/ d'ici (HOLO_REPO). Puis un Chrome sans fenêtre, qui ouvre
// chaque page dans une taille de téléphone (390 × 844, densité 3) ou de PC (1280 × 800), avec le
// réseau du cahier de Codex : 10 Mbit/s, 100 ms d'aller-retour, cache vide. Ce n'est PAS un
// téléphone : seulement le poids, la mise en page et les erreurs, mesurés sur un PC.
import { spawn } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { openChrome, pause } from "./cdp.mjs";

const repo = "C:/Users/mokea/Documents/IA_creation/Metaverse";
const here = new URL(".", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const site = join(here, "site");
const PORT = 8096;
const base = `http://localhost:${PORT}`;

const server = spawn(process.execPath, [join(repo, "moteur/outils/server.mjs")], { env: { ...process.env, PORT: String(PORT), HOLO_REPO: site }, stdio: "ignore" });
const results = {};
try {
  for (let i = 0; i < 50; i++) {
    await pause(200);
    try { if ((await fetch(`${base}/page.html`)).ok) break; } catch { /* pas encore */ }
  }

  // La page « proposée » : le HTML que le moteur fabriquerait avec l'option recommandée
  // (largeurs, sizes, width/height, loading lazy sauf la première ligne, decoding async).
  // Fabriquée à partir de la page réelle de catalogue-phone.holo, seules les images changent.
  const dims = { "01": [1280, 720], "02": [900, 700], "03": [1280, 720], "04": [1280, 720], "05": [540, 960], "06": [540, 960], "07": [1280, 760], "08": [1280, 1456], "09": [540, 960], "10": [1000, 900], "11": [540, 960], "12": [540, 960] };
  const real = await (await fetch(`${base}/exemples/media/catalogue-phone.holo`, { headers: { accept: "text/html" } })).text();
  let rank = 0;
  const proposed = real.replace(/<picture>.*?<img class="holo-Image" src="([^"]+)-1280\.webp" alt="([^"]*)"><\/picture>/g, (_, stem, alt) => {
    const nn = stem.slice(-2);
    const [w, h] = dims[nn];
    rank += 1;
    const lazy = rank > 2 ? ' loading="lazy"' : "";
    const big = w >= 1280 ? `, ${stem}-1280.webp 1280w` : "";
    const mid = Math.min(640, w);
    return `<img class="holo-Image" src="${stem}-640.webp" srcset="${stem}-320.webp 320w, ${stem}-640.webp ${mid}w${big}" sizes="(max-width:640px) 50vw, 210px" width="${w}" height="${h}"${lazy} decoding="async" alt="${alt}">`;
  });
  const CSS = "<style>:where(.holo-Image){max-width:100%;height:auto}</style>";
  writeFileSync(join(site, "exemples/media/catalogue-propose.html"), proposed.replace("</head>", CSS + "</head>"));
  const grande = await (await fetch(`${base}/exemples/media/grande-image.holo`, { headers: { accept: "text/html" } })).text();
  writeFileSync(join(site, "exemples/media/grande-image-corrigee.html"), grande.replace("</head>", CSS + "</head>").replace(/(<img class="holo-Image" src="[^"]+")/, "$1 width=\"1280\" height=\"720\""));
  results.proposedImages = rank;

  // Une vidéo avec des sous-titres, écrite à la main (ce que le web fait).
  writeFileSync(join(site, "exemples/media/video-sous-titres.html"), `<!doctype html><html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Vidéo et sous-titres</title></head><body><main><h1>Une vidéo</h1><video controls preload="metadata" playsinline src="film.mp4" aria-label="Une mire de couleurs qui défile"><track kind="captions" src="film.fr.vtt" srclang="fr" label="Français" default></video></main></body></html>`);
  writeFileSync(join(site, "exemples/media/video.holo"), `Page(\n  title: "Une vidéo",\n  children: [\n    H1("Une vidéo"),\n    Video(source: "film.mp4", label: "Une mire de couleurs qui défile, avec un son continu"),\n  ],\n)\n`);

  const c = await openChrome();
  try {
    await c.ask("Network.enable");
    await c.ask("Runtime.enable");
    await c.ask("Page.enable");
    await c.ask("Network.setCacheDisabled", { cacheDisabled: true });
    await c.ask("Network.emulateNetworkConditions", { offline: false, latency: 100, downloadThroughput: 1250000, uploadThroughput: 1250000 });
    await c.ask("Page.addScriptToEvaluateOnNewDocument", { source: "window.__cls=0;try{new PerformanceObserver(l=>{for(const e of l.getEntries())if(!e.hadRecentInput)window.__cls+=e.value}).observe({type:'layout-shift',buffered:true})}catch(e){}" });
    let requests = new Map();
    let exceptions = [];
    c.on("Network.requestWillBeSent", (p) => requests.set(p.requestId, { url: p.request.url, type: p.type, bytes: 0, done: false }));
    c.on("Network.responseReceived", (p) => { const r = requests.get(p.requestId); if (r) { r.mime = p.response.mimeType; r.status = p.response.status; } });
    c.on("Network.loadingFinished", (p) => { const r = requests.get(p.requestId); if (r) { r.bytes = p.encodedDataLength; r.done = true; } });
    c.on("Runtime.exceptionThrown", (p) => exceptions.push((p.exceptionDetails.exception?.description ?? p.exceptionDetails.text ?? "").split("\n").slice(0, 2).join(" | ")));

    const phone = { width: 390, height: 844, deviceScaleFactor: 3, mobile: true };
    const desk = { width: 1280, height: 800, deviceScaleFactor: 1, mobile: false };
    const tally = () => {
      const all = [...requests.values()];
      const images = all.filter((r) => r.type === "Image");
      const media = all.filter((r) => r.type === "Media");
      return {
        imagesDemandees: images.length,
        octetsImages: images.reduce((s, r) => s + r.bytes, 0),
        octetsMedia: media.reduce((s, r) => s + r.bytes, 0),
        octetsTotal: all.reduce((s, r) => s + r.bytes, 0),
        moteurDemande: all.some((r) => r.url.includes("page-engine.js")),
      };
    };
    async function open(name, metrics, url, { scroll = true, wait = 4000 } = {}) {
      requests = new Map();
      exceptions = [];
      await c.ask("Emulation.setDeviceMetricsOverride", metrics);
      await c.ask("Emulation.setTouchEmulationEnabled", { enabled: metrics.mobile, maxTouchPoints: metrics.mobile ? 5 : 1 });
      await c.ask("Page.navigate", { url });
      await pause(wait);
      const atOpen = tally();
      const layout = await c.evaluate(`({ largeurEcran: innerWidth, largeurPage: document.documentElement.scrollWidth, cls: Math.round(window.__cls*1000)/1000,
        images: [...document.querySelectorAll('img.holo-Image')].slice(0,3).map(i => ({ affichee: Math.round(i.getBoundingClientRect().width) + 'x' + Math.round(i.getBoundingClientRect().height), choisie: (i.currentSrc||'').split('/').pop() })) })`);
      let afterScroll = null;
      if (scroll) {
        for (let y = 0; y < 8; y++) { await c.evaluate("scrollBy(0, innerHeight)"); await pause(500); }
        await pause(2500);
        afterScroll = tally();
      }
      results[name] = { atOpen, layout, afterScroll, exceptions: [...exceptions] };
    }

    await open("propose-telephone", phone, `${base}/exemples/media/catalogue-propose.html`);
    await open("propose-pc", desk, `${base}/exemples/media/catalogue-propose.html`);
    await open("grande-image-telephone", phone, `${base}/exemples/media/grande-image.holo`, { scroll: false });
    await open("grande-image-corrigee-telephone", phone, `${base}/exemples/media/grande-image-corrigee.html`, { scroll: false });
  } finally {
    await c.close();
  }
} finally {
  server.kill();
}
console.log(JSON.stringify(results, null, 1));
