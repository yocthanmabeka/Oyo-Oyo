(() => {
  // Quinze minutes de Big Bang qui zoome puis dézoome sans arrêt (6 s chaque fois), pour la batterie et la chaleur.
  // Lancé par le câble, il continue seul quand on débranche le téléphone : l'écran reste allumé (Wake Lock).
  // En rebranchant : node outils/measure-phone.mjs big-bang.holo "JSON.stringify(window.__duree, null, 1)"
  if (window.__duree?.running) return "déjà en cours";
  const minutes = window.__dureeMinutes || 15; // une minute pour essayer le script sur le PC
  const canvas = document.querySelector("canvas");
  if (!canvas) return "pas de zone de dessin sur cette page";
  const d = (window.__duree = { running: true, start: new Date().toISOString(), wakeLock: null, hidden: 0, perMinute: [] });
  addEventListener("visibilitychange", () => { if (document.hidden) d.hidden++; });
  const nextFrame = () => new Promise(requestAnimationFrame);
  const battery = async () => {
    try {
      const b = await navigator.getBattery();
      return { level: Math.round(b.level * 100), charging: b.charging };
    } catch {
      return null;
    }
  };
  (async () => {
    let wake = null;
    try { wake = await navigator.wakeLock.request("screen"); } catch (e) { d.wakeLockError = String(e); }
    d.wakeLock = !!wake;
    d.batteryStart = await battery();
    const t0 = performance.now();
    for (let m = 1; m <= minutes; m++) {
      let frames = 0;
      let worst = 0;
      let before = performance.now();
      while (performance.now() < t0 + m * 60000) {
        const inward = Math.floor((performance.now() - t0) / 6000) % 2 === 0;
        canvas.dispatchEvent(new WheelEvent("wheel", { deltaY: inward ? -25 : 25, clientX: innerWidth / 2, clientY: innerHeight / 2, cancelable: true }));
        const t = await nextFrame();
        worst = Math.max(worst, t - before);
        before = t;
        frames++;
      }
      d.perMinute.push({
        minute: m,
        fps: Math.round((frames / 60) * 10) / 10,
        worst_ms: Math.round(worst * 10) / 10,
        depth: window.__holo?.depth ?? null,
        heap_MB: performance.memory ? Math.round(performance.memory.usedJSHeapSize / 1e5) / 10 : null,
        battery: await battery(),
      });
    }
    d.batteryEnd = await battery();
    d.end = new Date().toISOString();
    d.running = false;
    try { await wake?.release(); } catch {}
  })();
  return "lancé : " + minutes + " min";
})()
