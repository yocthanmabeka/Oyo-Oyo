// Affiche les mesures du sprint dans un coin de l'écran et permet de les copier.
// Tout ce qui est mesuré ici l'est sur l'appareil qui affiche la page : c'est le but.

const hud = document.createElement("pre");
hud.style.cssText =
  "position:fixed;left:8px;top:8px;margin:0;padding:6px 8px;color:#9fd;background:rgba(0,0,0,.55);" +
  "font:11px/1.35 ui-monospace,monospace;border-radius:6px;pointer-events:none;white-space:pre;z-index:9";
const button = document.createElement("button");
button.textContent = "Copier le rapport";
button.style.cssText =
  "position:fixed;right:8px;top:8px;padding:8px 12px;font:13px system-ui;border:0;border-radius:6px;" +
  "background:#246;color:#fff;z-index:9";
// Les mesures servent au sprint, pas à l'utilisateur : elles restent cachées. Un petit
// bouton en bas à droite les affiche, et l'adresse ?measures=1 les affiche d'emblée.
const toggle = document.createElement("button");
toggle.textContent = "measures";
toggle.title = "Afficher ou cacher les mesures";
toggle.style.cssText =
  "position:fixed;right:8px;bottom:8px;padding:6px 10px;font:11px system-ui;border:0;border-radius:6px;" +
  "background:rgba(255,255,255,.08);color:rgba(255,255,255,.55);z-index:9";
let visible = new URLSearchParams(location.search).get("measures") === "1";
function show() {
  hud.style.display = visible ? "block" : "none";
  button.style.display = visible ? "block" : "none";
}
toggle.addEventListener("click", () => { visible = !visible; show(); });

// Pause : le moteur ne calcule ni ne dessine plus rien tant qu'on ne reprend pas.
const pauseButton = document.createElement("button");
pauseButton.textContent = "pause";
pauseButton.title = "Mettre le monde en pause, ou le reprendre";
pauseButton.style.cssText = toggle.style.cssText;
pauseButton.style.right = "84px";
let paused = false;
pauseButton.addEventListener("click", () => {
  if (!window.__holoPause) return;
  paused = !paused;
  window.__holoPause(paused);
  pauseButton.textContent = paused ? "reprendre" : "pause";
  pauseButton.style.background = paused ? "rgba(255,180,60,.35)" : "rgba(255,255,255,.08)";
  display();
});
document.addEventListener("visibilitychange", () => {
  // Onglet caché : on se met en pause de nous-mêmes ; on reprend quand il revient, sauf pause manuelle.
  if (!window.__holoPause || paused) return;
  window.__holoPause(document.hidden);
});
document.body.append(hud, button, toggle, pauseButton);
show();

const pageStart = performance.timeOrigin;
let batteryStart = null;
if (navigator.getBattery) {
  navigator.getBattery().then((b) => { batteryStart = { level: b.level, t: performance.now(), b }; }).catch(() => {});
}

function weight() {
  const res = performance.getEntriesByType("resource").find((r) => r.name.endsWith(".wasm"));
  if (!res) return null;
  return { transferred: res.encodedBodySize, decompressed: res.decodedBodySize, duration_ms: Math.round(res.duration) };
}

function memory() {
  const m = performance.memory; // Chrome seulement
  return m ? Math.round(m.usedJSHeapSize / 1048576) : null;
}

function report() {
  const h = window.__holo ?? {};
  const p = weight();
  const battery = batteryStart
    ? { startValue: Math.round(batteryStart.level * 100), now: Math.round(batteryStart.b.level * 100), minutes: Math.round((performance.now() - batteryStart.t) / 60000) }
    : null;
  return {
    date: new Date().toISOString(),
    device: navigator.userAgent,
    screen: `${innerWidth}x${innerHeight} @${devicePixelRatio}`,
    webgpu_available: !!navigator.gpu,
    secure_context: isSecureContext,
    backend: h.backend ?? "(pas encore démarré)",
    frames_per_second: h.fps,
    worst_frame_ms: h.worst_ms,
    first_frame_ms: h.first_frame_ms,
    engine_wasm_bytes: p,
    js_memory_mb: memory(),
    depth: h.depth,
    path: h.path,
    seed: h.seed,
    drawn_points: h.drawn_points,
    zoom: h.zoom,
    renderer: h.width ? `${h.width}x${h.height}` : undefined,
    battery,
  };
}

function display() {
  const r = report();
  const ko = (n) => (n == null ? "?" : (n / 1024).toFixed(0) + " Ko");
  hud.textContent = [
    paused ? "EN PAUSE : rien n'est calculé ni dessiné" : `${r.backend}  ${r.frames_per_second ?? "?"} i/s  pire ${r.worst_frame_ms ?? "?"} ms`,
    `moteur ${r.engine_wasm_bytes ? ko(r.engine_wasm_bytes.transferred) + " transféré, " + ko(r.engine_wasm_bytes.decompressed) + " réel" : "…"}`,
    `première image ${r.first_frame_ms ?? "?"} ms   mémoire JS ${r.js_memory_mb ?? "n/d"} Mo`,
    `${r.path ?? ""}  profondeur ${r.depth ?? "?"}  zoom ${r.zoom ?? "?"}`,
    `${r.drawn_points ?? "?"} points  rendu ${r.renderer ?? "?"}` + (r.battery ? `  batterie ${r.battery.now} % (${r.battery.startValue} % il y a ${r.battery.minutes} min)` : ""),
    r.webgpu_available ? "" : `WebGPU indisponible${r.secure_context ? "" : " : page non sécurisée (http), voir le README"}`,
  ].filter(Boolean).join("\n");
}

button.addEventListener("click", async () => {
  const text = JSON.stringify(report(), null, 2);
  try {
    await navigator.clipboard.writeText(text);
    button.textContent = "Copié !";
  } catch {
    prompt("Copie ce rapport :", text);
  }
  setTimeout(() => (button.textContent = "Copier le rapport"), 1500);
});

setInterval(display, 500);
display();
