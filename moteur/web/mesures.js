// Affiche les mesures du sprint dans un coin de l'écran et permet de les copier.
// Tout ce qui est mesuré ici l'est sur l'appareil qui affiche la page : c'est le but.

const hud = document.createElement("pre");
hud.style.cssText =
  "position:fixed;left:8px;top:8px;margin:0;padding:6px 8px;color:#9fd;background:rgba(0,0,0,.55);" +
  "font:11px/1.35 ui-monospace,monospace;border-radius:6px;pointer-events:none;white-space:pre;z-index:9";
const bouton = document.createElement("button");
bouton.textContent = "Copier le rapport";
bouton.style.cssText =
  "position:fixed;right:8px;top:8px;padding:8px 12px;font:13px system-ui;border:0;border-radius:6px;" +
  "background:#246;color:#fff;z-index:9";
document.body.append(hud, bouton);

const debutPage = performance.timeOrigin;
let batterieDebut = null;
if (navigator.getBattery) {
  navigator.getBattery().then((b) => { batterieDebut = { niveau: b.level, t: performance.now(), b }; }).catch(() => {});
}

function poids() {
  const res = performance.getEntriesByType("resource").find((r) => r.name.endsWith(".wasm"));
  if (!res) return null;
  return { transfere: res.encodedBodySize, decompresse: res.decodedBodySize, duree_ms: Math.round(res.duration) };
}

function memoire() {
  const m = performance.memory; // Chrome seulement
  return m ? Math.round(m.usedJSHeapSize / 1048576) : null;
}

function rapport() {
  const h = window.__holo ?? {};
  const p = poids();
  const batterie = batterieDebut
    ? { depart: Math.round(batterieDebut.niveau * 100), maintenant: Math.round(batterieDebut.b.level * 100), minutes: Math.round((performance.now() - batterieDebut.t) / 60000) }
    : null;
  return {
    date: new Date().toISOString(),
    appareil: navigator.userAgent,
    ecran: `${innerWidth}x${innerHeight} @${devicePixelRatio}`,
    webgpu_disponible: !!navigator.gpu,
    contexte_securise: isSecureContext,
    backend: h.backend ?? "(pas encore démarré)",
    images_par_seconde: h.ips,
    pire_image_ms: h.pire_ms,
    premiere_image_ms: h.premiere_image_ms,
    moteur_wasm_octets: p,
    memoire_js_mo: memoire(),
    profondeur: h.profondeur,
    chemin: h.chemin,
    graine: h.graine,
    points_dessines: h.points_dessines,
    zoom: h.zoom,
    rendu: h.largeur ? `${h.largeur}x${h.hauteur}` : undefined,
    batterie,
  };
}

function afficher() {
  const r = rapport();
  const ko = (n) => (n == null ? "?" : (n / 1024).toFixed(0) + " Ko");
  hud.textContent = [
    `${r.backend}  ${r.images_par_seconde ?? "?"} i/s  pire ${r.pire_image_ms ?? "?"} ms`,
    `moteur ${r.moteur_wasm_octets ? ko(r.moteur_wasm_octets.transfere) + " transféré, " + ko(r.moteur_wasm_octets.decompresse) + " réel" : "…"}`,
    `première image ${r.premiere_image_ms ?? "?"} ms   mémoire JS ${r.memoire_js_mo ?? "n/d"} Mo`,
    `${r.chemin ?? ""}  profondeur ${r.profondeur ?? "?"}  zoom ${r.zoom ?? "?"}`,
    `${r.points_dessines ?? "?"} points  rendu ${r.rendu ?? "?"}` + (r.batterie ? `  batterie ${r.batterie.maintenant} % (${r.batterie.depart} % il y a ${r.batterie.minutes} min)` : ""),
    r.webgpu_disponible ? "" : `WebGPU indisponible${r.contexte_securise ? "" : " : page non sécurisée (http), voir le README"}`,
  ].filter(Boolean).join("\n");
}

bouton.addEventListener("click", async () => {
  const texte = JSON.stringify(rapport(), null, 2);
  try {
    await navigator.clipboard.writeText(texte);
    bouton.textContent = "Copié !";
  } catch {
    prompt("Copie ce rapport :", texte);
  }
  setTimeout(() => (bouton.textContent = "Copier le rapport"), 1500);
});

setInterval(afficher, 500);
afficher();
