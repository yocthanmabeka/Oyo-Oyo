(async () => {
  // Sur la boutique : entre en vue points, mesure le temps d'entrée, puis zoome 6 secondes.
  const attendre = () => new Promise(requestAnimationFrame);
  const t0 = performance.now();
  document.getElementById("points").click();
  while (!document.body.classList.contains("en-points")) {
    await attendre();
    if (performance.now() - t0 > 20000) return "la vue points n'a pas démarré en 20 s";
  }
  const entree = performance.now() - t0;
  const canvas = document.querySelector("canvas");
  const image = document.getElementById("image");
  const debut = performance.now();
  let images = 0;
  let pire = 0;
  let avant = debut;
  let maxPoints = 0;
  while (performance.now() - debut < 6000) {
    canvas.dispatchEvent(new WheelEvent("wheel", { deltaY: -12, clientX: innerWidth / 2, clientY: 60, cancelable: true }));
    const t = await attendre();
    pire = Math.max(pire, t - avant);
    avant = t;
    images++;
    const texte = document.getElementById("etat").textContent;
    const n = Number((texte.match(/^([\d\s  ]+) points à l'écran/) || [])[1]?.replace(/[^\d]/g, "") || 0);
    maxPoints = Math.max(maxPoints, n);
  }
  return JSON.stringify({
    entree_en_vue_points_ms: Math.round(entree),
    image_de_la_page: [image.naturalWidth, image.naturalHeight],
    points_au_repos: image.naturalWidth * image.naturalHeight,
    images_par_seconde_pendant_le_zoom: Math.round((images / 6) * 10) / 10,
    pire_image_ms: Math.round(pire * 10) / 10,
    plus_grand_nombre_de_points_a_l_ecran: maxPoints,
    etat_final: document.getElementById("etat").textContent,
    backend: window.__holo?.backend ?? null,
    tas_js_Mo: performance.memory ? Math.round(performance.memory.usedJSHeapSize / 1e5) / 10 : null,
  }, null, 1);
})()
