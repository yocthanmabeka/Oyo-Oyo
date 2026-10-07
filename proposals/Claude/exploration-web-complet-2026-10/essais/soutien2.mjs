import { openChrome, pause } from "./cdp.mjs";
const c = await openChrome();
try {
  await c.ask("Page.enable");
  await c.ask("Page.navigate", { url: "data:text/html,<!doctype html><body></body>" });
  await pause(600);
  console.log(JSON.stringify(await c.evaluate(`(async () => {
    const r = {
      offsetPathCercle: CSS.supports("offset-path", "circle(50% at 50% 50%)"),
      offsetPathShape: CSS.supports("offset-path", "shape(from 0% 50%, arc to 100% 50% of 50%, arc to 0% 50% of 50%, close)"),
    };
    document.body.innerHTML = '<div style="display:inline-grid;width:200px;height:200px"><div style="grid-area:1/1;width:200px;height:200px"></div><div id="l" style="grid-area:1/1;width:16px;height:16px;offset-path:circle(50% at 50% 50%);offset-distance:25%;offset-rotate:0deg"></div></div>';
    await new Promise(k => requestAnimationFrame(() => requestAnimationFrame(k)));
    const b = document.getElementById('l').getBoundingClientRect();
    r.luneAuQuart = { centreX: Math.round(b.x + b.width / 2), centreY: Math.round(b.y + b.height / 2) };
    return r;
  })()`)));
} finally { await c.close(); }
