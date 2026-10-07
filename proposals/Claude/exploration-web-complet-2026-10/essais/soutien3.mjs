import { openChrome, pause } from "./cdp.mjs";
const c = await openChrome();
try {
  await c.ask("Page.enable");
  await c.ask("Page.navigate", { url: "data:text/html,<!doctype html><body style='margin:0'></body>" });
  await pause(600);
  const code = String.raw`(async () => {
    const essai = async (chemin, positionne) => {
      document.body.innerHTML = '<div id="b" style="display:inline-grid;width:200px;height:200px"><div style="grid-area:1/1;width:200px;height:200px"></div><div id="l" style="grid-area:1/1;width:16px;height:16px"></div></div>';
      if (positionne) document.getElementById("b").style.position = "relative";
      const l = document.getElementById("l");
      l.style.offsetPath = chemin; l.style.offsetDistance = "25%"; l.style.offsetRotate = "0deg";
      await new Promise(k => requestAnimationFrame(() => requestAnimationFrame(k)));
      const r = l.getBoundingClientRect();
      return (l.style.offsetPath ? "" : "(refusé) ") + Math.round(r.x + 8) + "," + Math.round(r.y + 8);
    };
    return {
      cercle: await essai("circle(50% at 50% 50%)", false),
      cerclePositionne: await essai("circle(50% at 50% 50%)", true),
      chemin: await essai('path("M 100 0 A 100 100 0 1 1 100 200 A 100 100 0 1 1 100 0")', false),
      cheminPositionne: await essai('path("M 100 0 A 100 100 0 1 1 100 200 A 100 100 0 1 1 100 0")', true),
    };
  })()`;
  console.log(JSON.stringify(await c.evaluate(code)));
} finally { await c.close(); }
