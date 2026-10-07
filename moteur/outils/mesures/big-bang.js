(async () => {
  // Zoome dans le Big Bang pendant 6 secondes, comme une molette continue, et compte les images.
  const canvas = document.querySelector("canvas");
  const attendre = () => new Promise(requestAnimationFrame);
  const debut = performance.now();
  let images = 0;
  let pire = 0;
  let avant = debut;
  while (performance.now() - debut < 6000) {
    canvas.dispatchEvent(new WheelEvent("wheel", { deltaY: -25, clientX: innerWidth / 2, clientY: innerHeight / 2, cancelable: true }));
    const t = await attendre();
    pire = Math.max(pire, t - avant);
    avant = t;
    images++;
  }
  await new Promise((r) => setTimeout(r, 700));
  const m = window.__holo;
  return JSON.stringify({
    images_par_seconde_pendant_le_zoom: Math.round((images / 6) * 10) / 10,
    pire_image_ms: Math.round(pire * 10) / 10,
    profondeur_atteinte: m.depth,
    chemin: m.path,
    points_dessines: m.drawn_points,
    tas_js_Mo: performance.memory ? Math.round(performance.memory.usedJSHeapSize / 1e5) / 10 : null,
  }, null, 1);
})()
