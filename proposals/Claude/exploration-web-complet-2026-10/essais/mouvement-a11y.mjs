// Le film du duel (servi par le serveur de Yocthan, en lecture seule) : ce qu'entend un lecteur
// d'écran pour un titre coupé en lettres (letters:), et ce que voit le visiteur qui demande
// moins de mouvement. Chrome de PC, sans fenêtre : pas un téléphone, pas un vrai lecteur d'écran.
//   node mouvement-a11y.mjs
import { openChrome, pause } from "./cdp.mjs";

const url = "http://localhost:8080/exemples/motion/holocode/showreel.holo";
const twin = "http://localhost:8080/exemples/motion/web/showreel.html";
const c = await openChrome();
const out = {};
try {
  await c.ask("Page.enable");
  await c.ask("Accessibility.enable");
  for (const [name, address] of [["holocode", url], ["jumeau web", twin]]) {
    await c.ask("Page.navigate", { url: address });
    await pause(3000);
    const { result } = await c.ask("Accessibility.getFullAXTree");
    const headings = (result?.nodes ?? []).filter((n) => n.role?.value === "heading").map((n) => n.name?.value);
    out[name] = { titresEntendus: headings };
  }
  await c.ask("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  await c.ask("Page.navigate", { url });
  await pause(3000);
  out.moinsDeMouvement = await c.evaluate(`({
    animationsEnCours: document.getAnimations().filter(a => a.playState === 'running').length,
    scenesVisibles: [...document.querySelectorAll('.holo-Scene')].filter(s => getComputedStyle(s).visibility === 'visible' && getComputedStyle(s).opacity !== '0').length,
    derniereVisible: (() => { const s = [...document.querySelectorAll('.holo-Scene')].pop(); return s ? getComputedStyle(s).visibility + '/' + getComputedStyle(s).opacity : 'aucune'; })(),
  })`);
} finally {
  await c.close();
}
console.log(JSON.stringify(out, null, 1));
