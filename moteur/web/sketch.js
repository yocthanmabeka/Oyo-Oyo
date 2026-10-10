// La zone de dessin du visiteur (ADR-115) ; chargé seulement par une page qui en a une.
//
// La page ne décide de rien : elle recueille les points d'un trait, au doigt, à la souris, au
// stylet ou au clavier, et les donne au moteur, qui les range dans la feuille, les lisse, les
// simplifie, les borne et les garde dans la valeur de la zone. Elle montre ensuite ce que le moteur
// rend : les traits, et ce qui a été dessiné, dit avec des mots. Un trait refusé n'est pas gardé,
// et la page dit pourquoi.
//
// Le défilement d'un téléphone n'est pris que sur la feuille (`touch-action: none`, dans le style
// du moteur) ; à deux doigts, la feuille fait défiler la page au lieu de dessiner.

export function sketches({ root, source, state, change, input, stroke, view, image }) {
  // Les points bruts d'un trait, au plus : comme le moteur (sketch::RAW_MAX). Au-delà, le trait
  // continue à l'écran, mais aucun point n'est plus pris.
  const RAW_MAX = 4000;
  // Les dessins d'avant, par valeur : « Annuler » les rend un à un, même après « Effacer ».
  const UNDO_MAX = 100;
  const undone = new Map();
  // Le dessin que chaque zone montre ; ce que la page dira avant la description, une fois.
  const shown = new WeakMap();
  const before = new Map();
  let forSource = null;
  // Le trait en cours, au doigt, à la souris ou au stylet ; les doigts posés sur la feuille.
  let drawing = null;
  const fingers = new Map();
  let scrolling = null;
  // La pointe du clavier, par zone : sa place, et le trait en cours si le crayon est posé.
  const tips = new WeakMap();

  const french = () => (root.querySelector("[data-lang]")?.dataset.lang || document.documentElement.lang || "fr").startsWith("fr");
  const say = (fr, en) => (french() ? fr : en);
  const zones = () => [...root.querySelectorAll(".holo-Sketch[data-sketch]")];
  const isShared = (zone) => zone.hasAttribute("data-sketch-shared");
  // Le dessin d'une valeur, tel qu'il voyage dans l'état (codé) : `drawing='%23c62828%205%20…`.
  const codeOf = (written, name) => written.split(";").find((chunk) => chunk.startsWith(`${name}='`))?.slice(name.length + 2) ?? "";
  const decoded = (code) => { try { return decodeURIComponent(code); } catch { return ""; } };
  const sheetSize = (sheet) => sheet.viewBox.baseVal;
  const SVG = "http://www.w3.org/2000/svg";

  // Ce que la zone dit au visiteur, et au lecteur d'écran (role="status").
  function tell(zone, text) {
    const status = zone.querySelector(".holo-sketch-said");
    if (status && status.textContent !== text) status.textContent = text;
  }

  // Les choix du visiteur : la couleur et l'épaisseur cochées, sinon celles du départ.
  function chosen(zone) {
    return {
      color: zone.querySelector("[data-sketch-color]:checked")?.value ?? zone.dataset.startColor,
      size: Number(zone.querySelector("[data-sketch-size]:checked")?.value ?? zone.dataset.startSize),
    };
  }

  // Une zone fabriquée par le moteur devient une zone où l'on dessine : la feuille reçoit le clavier
  // (Tab), se présente comme une zone de dessin, et ses outils se montrent.
  function prepare() {
    if (forSource !== source()) {
      undone.clear();
      before.clear();
      forSource = source();
    }
    for (const zone of zones()) {
      if (zone.dataset.sketchReady) continue;
      zone.dataset.sketchReady = "1";
      const sheet = zone.querySelector("[data-sketch-sheet]");
      const tools = zone.querySelector(".holo-sketch-tools");
      const said = zone.querySelector(".holo-sketch-said");
      const how = zone.querySelector(".holo-sketch-how");
      if (tools) tools.hidden = false;
      sheet.setAttribute("aria-describedby", `${said.id} ${how.id}`);
      if (isShared(zone)) continue;
      // Une zone qu'on parcourt au clavier, les flèches à elle : `application`, et le nom de son rôle.
      sheet.setAttribute("role", "application");
      sheet.setAttribute("aria-roledescription", say("zone de dessin", "drawing area"));
      sheet.tabIndex = 0;
      // La pointe du clavier : une croix, au centre de la feuille, qui ne se voit qu'au clavier.
      const { width, height } = sheetSize(sheet);
      const r = Math.max(width, height) / 60;
      const tip = document.createElementNS(SVG, "g");
      tip.setAttribute("class", "holo-sketch-pen");
      tip.innerHTML = `<circle r="${r}" fill="none" stroke="#1565c0" stroke-width="${r / 3}"/><path d="M${-2 * r} 0H${-r}M${r} 0H${2 * r}M0 ${-2 * r}V${-r}M0 ${r}V${2 * r}" stroke="#1565c0" stroke-width="${r / 3}"/>`;
      sheet.append(tip);
      tips.set(zone, { x: width / 2, y: height / 2, points: null, live: null, mark: tip });
      placeTip(zone);
    }
    redraw();
  }

  // Les traits et la description de chaque zone, quand son dessin a changé.
  function redraw() {
    const written = state();
    for (const zone of zones()) {
      const name = zone.dataset.sketch;
      const code = codeOf(written, name);
      if (shown.get(zone) === code && !before.has(name)) continue;
      const [strokes, said = ""] = view(source(), written, name).split("\n");
      const group = zone.querySelector(".holo-sketch-strokes");
      if (shown.get(zone) !== code) group.innerHTML = strokes; // le SVG du moteur : des formes, des nombres, des couleurs de la feuille
      shown.set(zone, code);
      tell(zone, (before.get(name) ?? "") + said);
      before.delete(name);
    }
  }

  // Un dessin d'avant, pour « Annuler ».
  function remember(name, code) {
    const stack = undone.get(name) ?? [];
    stack.push(code);
    if (stack.length > UNDO_MAX) stack.shift();
    undone.set(name, stack);
  }

  // Le trait fini part au moteur : gardé, la page montre le nouveau dessin ; refusé, elle dit pourquoi.
  function keep(zone, color, size, points, said = "") {
    const name = zone.dataset.sketch;
    const written = state();
    let after;
    try {
      after = stroke(source(), written, name, color, size, points.map((n) => Math.round(n * 100) / 100).join(" "));
    } catch (reason) {
      tell(zone, String(reason?.message ?? reason));
      return;
    }
    remember(name, codeOf(written, name));
    if (said) before.set(name, said);
    change(after);
  }

  function undo(zone) {
    const name = zone.dataset.sketch;
    const stack = undone.get(name) ?? [];
    if (!stack.length) return tell(zone, say("Rien à annuler.", "Nothing to undo."));
    before.set(name, say("Annulé. ", "Undone. "));
    change(input(source(), state(), name, decoded(stack.pop())));
    redraw();
  }

  function clear(zone) {
    const name = zone.dataset.sketch;
    const code = codeOf(state(), name);
    if (!code) return tell(zone, say("Rien à effacer.", "Nothing to clear."));
    remember(name, code);
    before.set(name, say("Effacé : « Annuler » le rend. ", "Cleared: Undo brings it back. "));
    change(input(source(), state(), name, ""));
    redraw();
  }

  // Le nom du fichier enregistré : le nom de la zone, sans accent ni signe.
  const fileName = (zone) => (zone.querySelector("legend")?.textContent ?? "").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 60) || say("dessin", "drawing");

  function download(zone, blob, file) {
    const address = URL.createObjectURL(blob);
    const link = Object.assign(document.createElement("a"), { href: address, download: file });
    link.click();
    setTimeout(() => URL.revokeObjectURL(address), 2000);
    tell(zone, say(`Image enregistrée : ${file}.`, `Image saved: ${file}.`));
  }

  // L'image enregistrée : le SVG fabriqué par le moteur, tel quel ; le PNG, ce même SVG dessiné
  // par le navigateur, assez grand pour être net (1 600 pixels de côté, au plus quatre fois la feuille).
  async function save(zone, format) {
    const svg = image(source(), state(), zone.dataset.sketch);
    if (!svg) return;
    const file = `${fileName(zone)}.${format}`;
    const picture = new Blob([svg], { type: "image/svg+xml" });
    if (format === "svg") return download(zone, picture, file);
    const address = URL.createObjectURL(picture);
    try {
      const drawn = new Image();
      drawn.src = address;
      await drawn.decode();
      const { width, height } = sheetSize(zone.querySelector("[data-sketch-sheet]"));
      const scale = Math.min(4, Math.max(1, 1600 / Math.max(width, height)));
      const canvas = Object.assign(document.createElement("canvas"), { width: Math.round(width * scale), height: Math.round(height * scale) });
      canvas.getContext("2d").drawImage(drawn, 0, 0, canvas.width, canvas.height);
      const png = await new Promise((done) => canvas.toBlob(done, "image/png"));
      if (png) download(zone, png, file);
    } catch {
      tell(zone, say("L'image n'a pas pu être enregistrée.", "The image could not be saved."));
    } finally {
      URL.revokeObjectURL(address);
    }
  }

  // Un point de l'écran, dans les unités de la feuille (la feuille peut être plus petite que sa
  // place, ou grossie : la matrice de l'écran le dit).
  function toSheet(sheet, event) {
    const matrix = sheet.getScreenCTM();
    if (!matrix) return null;
    const point = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse());
    return [point.x, point.y];
  }

  // Le trait en cours, montré tel que la main le trace ; le moteur le remplacera par le sien.
  function liveLine(sheet, color, size) {
    const line = document.createElementNS(SVG, "polyline");
    for (const [name, value] of [["fill", "none"], ["stroke", color], ["stroke-width", size], ["stroke-linecap", "round"], ["stroke-linejoin", "round"], ["class", "holo-sketch-live"]]) line.setAttribute(name, value);
    sheet.querySelector(".holo-sketch-strokes").append(line);
    return line;
  }
  const follow = (line, points) => line.setAttribute("points", points.join(" "));

  function add(event) {
    if (drawing.points.length >= 2 * RAW_MAX) return;
    const point = toSheet(drawing.sheet, event);
    if (!point) return;
    drawing.points.push(...point);
    follow(drawing.live, drawing.points);
  }

  function forget() {
    drawing?.live.remove();
    drawing = null;
  }

  // Au doigt, à la souris, au stylet : le même code (les événements de pointeur).
  root.addEventListener("pointerdown", (event) => {
    const sheet = event.target.closest?.("[data-sketch-sheet]");
    const zone = sheet?.closest(".holo-Sketch[data-sketch-ready]");
    if (!zone || isShared(zone) || (event.pointerType === "mouse" && event.button !== 0)) return;
    if (event.pointerType === "touch") fingers.set(event.pointerId, [event.clientX, event.clientY]);
    // Un second doigt : la feuille fait défiler la page, et le trait commencé est oublié.
    if (fingers.size > 1) {
      forget();
      scrolling = true;
      return;
    }
    if (drawing) return;
    event.preventDefault();
    sheet.setPointerCapture?.(event.pointerId);
    // Le clavier suit la main : Ctrl+Z annule aussitôt ce trait.
    sheet.focus({ preventScroll: true });
    const { color, size } = chosen(zone);
    drawing = { zone, sheet, pointer: event.pointerId, color, size, points: [], live: liveLine(sheet, color, size) };
    add(event);
  });
  root.addEventListener("pointermove", (event) => {
    if (scrolling && fingers.has(event.pointerId)) {
      const [x, y] = fingers.get(event.pointerId);
      fingers.set(event.pointerId, [event.clientX, event.clientY]);
      scrollBy(-(event.clientX - x) / fingers.size, -(event.clientY - y) / fingers.size);
      return;
    }
    if (!drawing || event.pointerId !== drawing.pointer) return;
    for (const each of event.getCoalescedEvents?.() ?? [event]) add(each);
  });
  const lift = (event, cancelled) => {
    fingers.delete(event.pointerId);
    if (!fingers.size) scrolling = null;
    if (!drawing || event.pointerId !== drawing.pointer) return;
    const done = drawing;
    forget();
    if (!cancelled && done.points.length) keep(done.zone, done.color, done.size, done.points);
  };
  root.addEventListener("pointerup", (event) => lift(event, false));
  root.addEventListener("pointercancel", (event) => lift(event, true));

  // Les outils : de vrais boutons, au doigt, à la souris ou au clavier.
  root.addEventListener("click", (event) => {
    const button = event.target.closest?.("[data-sketch-do]");
    const zone = button?.closest(".holo-Sketch[data-sketch-ready]");
    if (!zone) return;
    // Ce toucher est celui d'un outil : il ne part pas au moteur comme celui d'un bloc.
    event.stopPropagation();
    const action = button.dataset.sketchDo;
    if (action === "undo" && !isShared(zone)) undo(zone);
    else if (action === "clear" && !isShared(zone)) clear(zone);
    else if (action === "svg" || action === "png") save(zone, action);
  });

  // La pointe du clavier, à sa place sur la feuille.
  function placeTip(zone) {
    const tip = tips.get(zone);
    if (!tip) return;
    tip.mark.setAttribute("transform", `translate(${tip.x} ${tip.y})`);
    tip.mark.querySelector("circle").setAttribute("fill", tip.points ? "#1565c0" : "none");
  }

  // Au clavier : Ctrl+Z annule, où que soit le clavier dans la zone. Sur la feuille, les flèches
  // déplacent la pointe (Maj : par petits pas), Entrée ou Espace pose le crayon puis le lève, Échap
  // oublie le trait en cours. Ces touches sont à la zone : elles ne vont pas aux règles de la page.
  root.addEventListener("keydown", (event) => {
    const zone = event.target.closest?.(".holo-Sketch[data-sketch-ready]");
    if (!zone || isShared(zone) || event.altKey) return;
    if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.key.toLowerCase() === "z") {
      event.preventDefault();
      event.stopPropagation();
      return undo(zone);
    }
    const sheet = event.target.closest("[data-sketch-sheet]");
    const tip = tips.get(zone);
    if (!sheet || !tip || event.ctrlKey || event.metaKey) return;
    const { width, height } = sheetSize(sheet);
    const step = (event.shiftKey ? 0.1 : 1) * Math.max(width, height) / 40;
    const moves = { ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step] };
    if (moves[event.key]) {
      tip.x = Math.min(width, Math.max(0, tip.x + moves[event.key][0]));
      tip.y = Math.min(height, Math.max(0, tip.y + moves[event.key][1]));
      if (tip.points) {
        tip.points.push(tip.x, tip.y);
        follow(tip.live, tip.points);
      }
    } else if (event.key === "Enter" || event.key === " ") {
      if (!tip.points) {
        const { color, size } = chosen(zone);
        tip.points = [tip.x, tip.y];
        tip.live = liveLine(sheet, color, size);
        follow(tip.live, tip.points);
        Object.assign(tip, { color, size });
        tell(zone, say(`Crayon posé en ${Math.round(tip.x)}, ${Math.round(tip.y)} : les flèches tracent, Entrée lève le crayon.`, `Pen down at ${Math.round(tip.x)}, ${Math.round(tip.y)}: the arrows draw, Enter lifts the pen.`));
      } else {
        const { points, color, size } = tip;
        tip.live.remove();
        Object.assign(tip, { points: null, live: null });
        keep(zone, color, size, points, say("Trait gardé. ", "Stroke kept. "));
      }
    } else if (event.key === "Escape" && tip.points) {
      tip.live.remove();
      Object.assign(tip, { points: null, live: null });
      tell(zone, say("Trait oublié.", "Stroke forgotten."));
    } else {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    placeTip(zone);
  });

  return { prepare, redraw };
}
