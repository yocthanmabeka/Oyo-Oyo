// Ce que le Chrome de ce PC sait faire en CSS, pour les pistes 8 et 9 et le duel :
// chemins, mouvement lié au défilement, interpolation de formes, sizes="auto".
// Chrome de bureau seulement : pas un téléphone, pas Safari, pas Firefox.
//   node soutien.mjs
import { openChrome, pause } from "./cdp.mjs";

const c = await openChrome();
const out = {};
try {
  await c.ask("Page.enable");
  out.version = (await c.ask("Browser.getVersion")).result?.product;
  await c.ask("Page.navigate", { url: "data:text/html,<!doctype html><html><body></body></html>" });
  await pause(800);
  out.supports = await c.evaluate(`({
    offsetPath: CSS.supports("offset-path", "path('M0 0 L100 100')"),
    clipPathPath: CSS.supports("clip-path", "path('M0 0 L100 0 L50 100 Z')"),
    clipPathShape: CSS.supports("clip-path", "shape(from 0 0, line to 100% 0, line to 50% 100%, close)"),
    dPath: CSS.supports("d", "path('M0 0 L10 10')"),
    animationTimelineView: CSS.supports("animation-timeline", "view()"),
    animationTimelineScroll: CSS.supports("animation-timeline", "scroll()"),
    animationRange: CSS.supports("animation-range", "entry 0% cover 50%"),
    sizesAutoImg: "sizes" in HTMLImageElement.prototype,
    viewTransitions: "startViewTransition" in document,
    registerProperty: "registerProperty" in CSS,
  })`);
  // Deux polygones de même nombre de sommets : le navigateur passe-t-il de l'un à l'autre ?
  out.morphPolygone = await c.evaluate(`(async () => {
    const n = 32, ring = (f) => Array.from({ length: n }, (_, i) => { const a = i / n * 2 * Math.PI - Math.PI / 2; const [x, y] = f(a); return (50 + 50 * x).toFixed(2) + '% ' + (50 + 50 * y).toFixed(2) + '%'; }).join(',');
    const rond = ring((a) => [Math.cos(a), Math.sin(a)]);
    const losange = ring((a) => { const c = Math.cos(a), s = Math.sin(a), k = 1 / (Math.abs(c) + Math.abs(s)); return [c * k, s * k]; });
    const st = document.createElement('style');
    st.textContent = '@keyframes m{from{clip-path:polygon(' + losange + ')}to{clip-path:polygon(' + rond + ')}}#f{width:100px;height:100px;background:gold;animation:m 2s linear -1s paused}';
    document.head.append(st);
    const f = document.createElement('div'); f.id = 'f'; document.body.append(f);
    await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    const mid = getComputedStyle(f).clipPath;
    return { octetsDUnPolygone32: ('polygon(' + rond + ')').length, aMiChemin: mid.slice(0, 70) + '…', interpole: mid !== 'polygon(' + losange.replaceAll(',', ', ') + ')' && mid.startsWith('polygon(') };
  })()`);
  // sizes="auto" : le navigateur choisit la largeur d'après la mise en page (images lazy seulement).
  await c.ask("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 3, mobile: true });
  out.sizesAuto = await c.evaluate(`(async () => {
    const svg = (w) => 'data:image/svg+xml,' + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="' + w + '" height="' + (w * 9 / 16) + '"><rect width="100%" height="100%" fill="teal"/><text x="10" y="40" font-size="40">' + w + '</text></svg>');
    document.body.innerHTML = '<div style="width:155px"><img id="i" loading="lazy" sizes="auto" width="1280" height="720" style="max-width:100%;height:auto" srcset="' + svg(320) + ' 320w, ' + svg(640) + ' 640w, ' + svg(1280) + ' 1280w"></div>';
    const i = document.getElementById('i'); await i.decode().catch(() => {});
    return { affichee: Math.round(i.getBoundingClientRect().width), choisie: i.currentSrc.includes('640') ? '640w' : i.currentSrc.includes('1280') ? '1280w' : i.currentSrc.includes('320') ? '320w' : '?' };
  })()`);
} finally {
  await c.close();
}
console.log(JSON.stringify(out, null, 1));
