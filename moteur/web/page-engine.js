// Le moteur de la page d'entrée : il arrive après la page (voir page.html), quand un geste en
// a besoin ou tout de suite si la page est vivante (ADR-033). Il charge le moteur en Rust,
// compilé en WebAssembly, et prend la page en main.
  // Le moteur léger (ADR-053) : lire le fichier, fabriquer la page, arbitrer les valeurs. Le
  // dessin (les points, les mondes, la vue points) est un second moteur, chargé seulement quand
  // la page s'en sert : une page qui ne fait que bouger ne le télécharge jamais.
  import init, {
    flat_view, effects, initial_state, arbitrate, submission, form_errors, format_value, format_date, list_html, module_info, module_finished, module_input, module_received, delays, reads_time, set_now, advance_clock, conditions, clocks, touched_ones, keypresses, imports, data, receive, input, drag, to_keep, resume, neighbour_worlds, view_settings, needs_drawing,
  } from "/pkg-light/holo_engine.js";
  let drawing = null;
  let drawingLoading = null;
  const loadDrawing = () => (drawingLoading ??= import("/pkg/holo_engine.js").then(async (m) => { await m.default(); drawing = m; return m; }));
  const start_engine = async (...a) => (await loadDrawing()).start_engine(...a);
  const start_mosaic = async (...a) => (await loadDrawing()).start_mosaic(...a);
  // Ceux-là ne servent qu'une fois le dessin lancé : il est donc déjà là.
  const change_world = (...a) => drawing?.change_world(...a);
  const place_mosaic = (...a) => drawing?.place_mosaic(...a);
  const remove_mosaic = () => drawing?.remove_mosaic();
  const mosaic_camera = () => drawing?.mosaic_camera() ?? [];
  const mosaic_under = (...a) => drawing?.mosaic_under(...a) ?? [];
  const mosaic_turn = (a) => drawing?.mosaic_turn(a);
  const mosaic_pivot = (...a) => drawing?.mosaic_pivot(...a);
  const mosaic_front = () => drawing?.mosaic_front();
  const pause = (a) => drawing?.pause(a);
  const wake = () => drawing?.wake();
  const frames_drawn = () => drawing?.frames_drawn() ?? 0;
  window.__holoPause = pause;
  window.__holoImages = frames_drawn; // combien d'images le moteur a dessinées : pour vérifier la sobriété
  const params = new URLSearchParams(location.search);
  window.__holoWithoutWebGPU = params.has("webgl"); // ?webgl : mesurer le mode de secours, WebGL 2
  // L'adresse est celle du fichier .holo lui-même ; sinon ?world=… ou la boutique d'exemple.
  // Ce fichier peut changer en cours de route : on passe d'un fichier à l'autre par un point,
  // sans recharger la page.
  let path = location.pathname.endsWith(".holo") ? location.pathname : params.get("world") ?? "/exemples/boutique-comparee/boutique.holo";
  let base = path.slice(0, path.lastIndexOf("/") + 1);
  const folderOf = (file) => file.slice(0, file.lastIndexOf("/") + 1);
  // Les fichiers déjà lus, par adresse : leur texte, ou null s'ils sont introuvables ou refusés.
  const readFiles = new Map();
  // Garde-fou : on ne garde en mémoire que les derniers fichiers lus. On peut passer d'un
  // fichier à l'autre sans fin ; la mémoire, elle, ne grandit pas sans fin.
  const KEPT_FILES = 32;
  // Un fichier .holo est un texte court. On arrête de lire au-delà de cette taille, et on
  // n'attend pas un serveur qui ne répond pas (revue Codex du 2026-10-03, B-04). La mémoire
  // des fichiers lus est donc bornée : 32 fichiers de 256 Ko au plus.
  const BYTES_MAX = 262144;
  const DELAY_MAX = 8000;
  // La vue points redessine la page dans une image : jamais plus de huit millions de points,
  // quelle que soit la densité demandée, pour tenir dans la mémoire d'un téléphone (B-05).
  const POINTS_MAX = 8e6;
  let requestedDensity = 2;
  let zoomSpeed = 1;     // Zoom(speed:)
  let portalDuration = 450;  // Portals(duration:), en millisecondes
  let rotationAngle = 0;   // Relief(tilt:), en degrés ; 0 : la page ne tourne pas
  // Qui grossit la page quand on zoome (ADR-069). Par défaut, le navigateur : la page reste à sa
  // place. Le moteur, si le fichier le demande (points, Zoom(shrink:), Zoom(active: false)), ou
  // si le visiteur a décroché la page (Zoom(detach: true), puis « Décrocher »).
  let detachAllowed = false;
  let detached = false;
  let byEngine = false;
  let activePoints = false;  // l'auteur a-t-il demandé les points (points: ou pixels:) ? Sinon, un site ordinaire
  // Les valeurs de chaque fichier ouvert (State) : « cart=2 ». Elles suivent le visiteur tant
  // qu'il ne recharge pas la page : il peut entrer dans un monde, passer ailleurs, revenir.
  const states = new Map();
  const atHome = (host) => host === "localhost" || host === "127.0.0.1";
  // En http, on ne va que vers sa propre machine, et seulement si l'on y est déjà : une page
  // publique ne fait pas partir de requêtes vers le réseau privé du visiteur.
  function passageAllowed(file) {
    const address = new URL(file, location.href);
    return address.protocol === "https:" || address.origin === location.origin || (atHome(address.hostname) && atHome(location.hostname));
  }
  const fullAddress = (file) => new URL(file, location.href).href;
  const fromElsewhere = (file) => new URL(file, location.href).origin !== location.origin;
  let lastPassage = 0;  // pour qu'un seul coup de molette ne fasse pas remonter deux mondes
  // Le choix du visiteur, réglé dans son téléphone ou son ordinateur (« réduire les
  // animations »). Sans ce choix, tout est au niveau normal. Avec lui : pas de transition,
  // et la page ne devient pas des points toute seule quand on zoome ; le bouton « Vue points »
  // reste là pour qui veut y aller.
  const calm = matchMedia("(prefers-reduced-motion: reduce)");
  const root = document.getElementById("page");
  // La page arrive déjà fabriquée par le serveur : on peut toucher un bouton avant que le
  // moteur soit prêt. La page légère (page.html) note ces touchers ; ils sont rejoués dès que
  // le moteur l'est : aucun n'est perdu.
  const pendingTouches = window.__holoWaiting ?? [];
  const crossroads = document.getElementById("crossroads");
  const modeButton = document.getElementById("mode");
  const pointsButton = document.getElementById("points");
  const turnButton = document.getElementById("rotate");
  const detachButton = document.getElementById("detach");
  const image = document.getElementById("image");
  const state = document.getElementById("status");
  const number = new Intl.NumberFormat("fr-FR");
  const area = () => document.querySelector("canvas"); // le moteur peut l'avoir remplacée
  // Ce que le fichier dit de sa vue (Zoom, Points). Lu au démarrage.
  let density = 2;          // points par pixel d'écran, dans chaque sens
  let reduce = false;      // dézoomer réduit-il la page jusqu'à un point ?
  let zoomBeforePoints = 4;  // jusqu'à ce grossissement, la page reste un site ordinaire
  let zoomActive = true;     // le visiteur peut-il zoomer ? (Zoom(active:))
  // Le carrefour (Portals) : disposition, nombre de mondes, taille d'un portail, lumière du fond.
  let layout = "grid";
  let portalCount = 12;
  let portalSize = 170;
  let backgroundLight = 0.15;
  let inWorld = false;      // on est dans un monde calculé, ouvert en profondeur
  let source = "";
  let site = "";            // le site affiché : "" pour la page du fichier, sinon le chemin des points traversés
  let frame = null;         // l'élément .holo-Page
  let page = null;          // son contenu, <main>
  let engineStarted = false;
  let sharpZoom = 1;          // le grossissement de la page vivante
  let inPoints = false;     // la page est vue comme un ensemble de points
  let entryInProgress = null; // l'entrée en vue points, tant qu'elle se prépare
  let hasLeftRest = false; // a-t-on zoomé ou tourné depuis l'entrée en vue points ?
  let plantedPixels = [];   // les points plantés dans un pixel de la page, et où ils sont

  const parent = (sitePath) => sitePath.split("/").filter(Boolean).slice(0, -1).join("/");
  const last = (sitePath) => sitePath.split("/").filter(Boolean).at(-1) ?? "";

  // ---------------------------------------------------------------- les fichiers

  // Lit un fichier .holo et le fait vérifier par le moteur. Un fichier est petit : on peut
  // lire d'avance ceux où mènent les points de la page, pour que le passage soit immédiat.
  // Un fichier peut en importer d'autres, rangés à côté de lui (import "commun.holo"). La page
  // va les chercher et les joint à son texte : le moteur, lui, ne lit jamais rien tout seul.
  async function withImports(text, file) {
    let all = text;
    const discreet = fromElsewhere(file) ? { credentials: "omit", referrerPolicy: "no-referrer" } : {};
    for (const name of imports(text).split(";").filter(Boolean)) {
      try {
        const response = await fetch(folderOf(file) + name, { headers: { accept: "text/plain" }, ...discreet });
        if (response.ok) all += `\u001e${name}\u001f${(await response.text()).slice(0, BYTES_MAX)}`;
      } catch { /* introuvable : le moteur le dira, avec la ligne de l'import */ }
    }
    return all;
  }

  function read(file) {
    if (readFiles.has(file)) {
      // Relu à l'instant : il repasse en tête des fichiers gardés.
      const already = readFiles.get(file);
      readFiles.delete(file);
      readFiles.set(file, already);
    }
    if (!readFiles.has(file)) {
      for (const old of readFiles.keys()) {
        if (readFiles.size < KEPT_FILES) break;
        if (old !== path) readFiles.delete(old);
      }
      readFiles.set(file, (async () => {
        const stop = new AbortController();
        const delay = setTimeout(() => stop.abort(), DELAY_MAX);
        try {
          if (!passageAllowed(file)) return null;
          // Chez quelqu'un d'autre : sans cookies, et sans dire d'où l'on vient.
          const discreet = fromElsewhere(file) ? { credentials: "omit", referrerPolicy: "no-referrer" } : {};
          const response = await fetch(file, { headers: { accept: "text/plain" }, signal: stop.signal, ...discreet });
          if (!response.ok || Number(response.headers.get("content-length") ?? 0) > BYTES_MAX) return null;
          // Lecture par morceaux : on s'arrête dès que la limite est dépassée.
          const reader = response.body.getReader();
          const chunks = [];
          let received = 0;
          for (;;) {
            const { done, value } = await reader.read();
            if (done) break;
            received += value.length;
            if (received > BYTES_MAX) {
              stop.abort();
              return null;
            }
            chunks.push(value);
          }
          const text = await withImports(new TextDecoder().decode(await new Blob(chunks).arrayBuffer()), file);
          flat_view(text, folderOf(file), ""); // refusé par le moteur : le passage reste fermé
          return text;
        } catch {
          return null;
        } finally {
          clearTimeout(delay);
        }
      })().then((text) => {
        if (text === null) readFiles.delete(file); // un échec n'est pas gardé : on pourra réessayer
        return text;
      }));
    }
    return readFiles.get(file);
  }

  // Ce que le fichier affiché dit de sa vue (Zoom, Points, Portals).
  function readSettings() {
    let active, storage;
    [requestedDensity, reduce, zoomBeforePoints, active, storage, portalCount, portalSize, backgroundLight, zoomSpeed, portalDuration, rotationAngle, activePoints, detachAllowed] = view_settings(source);
    activePoints = activePoints === 1;
    document.body.classList.toggle("rotated", rotationAngle > 0 && active === 1);
    // L'état de départ, avec ce que la page a gardé d'une visite précédente (keep: […]).
    if (!states.has(path)) {
      let kept = "";
      try { kept = localStorage.getItem(`holo:${path}`) ?? ""; } catch { /* stockage refusé : on part du départ */ }
      states.set(path, resume(source, kept));
    }
    setClocks();
    document.documentElement.style.setProperty("--duration", `${portalDuration}ms`);
    density = requestedDensity;
    reduce = reduce === 1;
    zoomActive = active === 1;
    layout = ["grid", "line", "column", "diagonal"][storage];
    pointsButton.style.display = zoomActive && activePoints ? "" : "none";
    // Une autre page commence accrochée.
    detachAllowed = detachAllowed === 1;
    detached = false;
    document.body.classList.remove("detached");
    detachButton.textContent = "Décrocher";
    zoomMode();
  }

  // Qui grossit la page quand on zoome : le navigateur, ou le moteur (ADR-069).
  function zoomMode() {
    byEngine = activePoints || reduce || !zoomActive || detached;
    document.documentElement.classList.toggle("holo-zoom", byEngine);
    detachButton.hidden = !detachAllowed;
  }

  // Passe à un autre fichier sans recharger la page : son adresse devient celle de la barre
  // du navigateur, et le bouton « retour » ramène d'où l'on vient.
  async function openFile(file, sitePath = "", { inHistory = true } = {}) {
    const text = await read(file);
    if (text === null) return false;
    const since = location.pathname + location.hash;
    [path, base, source] = [file, folderOf(file), text];
    readSettings();
    // Un navigateur interdit à une page d'afficher l'adresse d'un autre serveur comme si elle
    // y était. Pour un fichier d'ailleurs, l'adresse garde donc le fichier de départ, suivi de
    // #@ et de l'adresse où l'on est vraiment.
    if (inHistory) {
      history.pushState({ since }, "", fromElsewhere(file) ? `#@${fullAddress(file)}` : file + (sitePath ? `#${sitePath}` : ""));
    }
    displaySite(sitePath, { inHistory: false });
    return true;
  }

  // Remonte d'un monde : au site qui contient celui-ci, ou au fichier d'où l'on est venu.
  function goUp() {
    if (performance.now() - lastPassage < 700) return;
    if (site) displaySite(parent(site));
    else if (history.state?.since) history.back();
    else return;
    lastPassage = performance.now();
  }

  // ---------------------------------------------------------------- le site affiché

  // Affiche un site du fichier, sans recharger la page : pas de noir entre deux sites.
  // Le temps : une horloge par rythme écrit dans le fichier (Every(1s, …)). À chaque battement,
  // le signal est donné à l'arbitre, comme un toucher. L'horloge se tait quand la fenêtre est
  // cachée, en vue points et devant le carrefour : rien ne tourne pour rien.
  const hoveredOnes = new Set(); // les blocs survolés en ce moment (ADR-039)
  let listenedKeys = []; // les touches que les règles du fichier écoutent : On(Key.left, …)
  // Les touches à une lettre ou à un chiffre peuvent gêner un logiciel de dictée : le visiteur
  // peut les couper dans le menu (WCAG 2.1.4, ADR-061). Son choix est gardé dans ce navigateur.
  let lettersAllowed = true;
  try { lettersAllowed = localStorage.getItem("holo:shortcuts") !== "off"; } catch { /* stockage refusé */ }
  const isSingleCharacter = (keypress) => /^([a-z]|digit\d)$/.test(keypress);
  function setShortcutsButton() {
    let button = document.getElementById("shortcuts");
    if (!listenedKeys.some(isSingleCharacter)) {
      button?.remove();
      return;
    }
    if (!button) {
      button = document.createElement("button");
      button.id = "shortcuts";
      button.type = "button";
      button.textContent = "Touches à une lettre";
      button.title = "Les lettres et les chiffres que cette page écoute ; à couper pour un logiciel de dictée";
      button.addEventListener("click", () => {
        lettersAllowed = !lettersAllowed;
        button.setAttribute("aria-pressed", String(lettersAllowed));
        try { localStorage.setItem("holo:shortcuts", lettersAllowed ? "on" : "off"); } catch { /* stockage refusé */ }
      });
      document.getElementById("views")?.append(button);
    }
    button.setAttribute("aria-pressed", String(lettersAllowed));
  }
  // Les touches à l'écran (ADR-069). Règle de Yocthan du 2026-10-07 : ce qui existe sur
  // l'ordinateur existe sur le téléphone, et l'inverse. Sur un appareil qu'on touche du doigt,
  // sans souris (donc, presque toujours, sans clavier), une page qui écoute des touches les
  // montre en bas de l'écran : le doigt fait ce que fait le clavier. Le clavier de l'écran,
  // lui, n'apparaît que dans un champ où l'on écrit.
  const touchOnly = matchMedia("(hover: none) and (pointer: coarse)");
  const keysBar = document.getElementById("keys");
  const keyLabels = { left: "←", up: "↑", down: "↓", right: "→", space: "Espace", enter: "Entrée", escape: "Échap" };
  const keySpoken = { left: "flèche gauche", up: "flèche haut", down: "flèche bas", right: "flèche droite" };
  function setKeysBar() {
    keysBar.replaceChildren();
    const shown = touchOnly.matches && listenedKeys.length > 0;
    keysBar.hidden = !shown;
    document.body.style.paddingBottom = "";
    if (!shown) return;
    // Les flèches d'abord, rangées comme sur un clavier ; puis les autres, dans l'ordre du fichier.
    const arrows = ["left", "up", "down", "right"].filter((key) => listenedKeys.includes(key));
    for (const key of [...arrows, ...listenedKeys.filter((key) => !arrows.includes(key))]) {
      const button = document.createElement("button");
      button.type = "button";
      button.dataset.key = key;
      button.textContent = keyLabels[key] ?? (key.startsWith("digit") ? key.slice(5) : key.toUpperCase());
      button.setAttribute("aria-label", `Touche ${keySpoken[key] ?? button.textContent}`);
      keysBar.append(button);
    }
    // Les touches ne cachent jamais le bas de la page : on peut toujours y descendre.
    document.body.style.paddingBottom = `${keysBar.offsetHeight + 24}px`;
  }
  touchOnly.addEventListener("change", setKeysBar);
  {
    let repeat = 0;
    let pressedByFinger = null;
    const stop = () => {
      clearTimeout(repeat);
      clearInterval(repeat);
      keysBar.querySelector(".pressed")?.classList.remove("pressed");
    };
    const press = (key) => {
      if (inPoints || inWorld || !crossroads.hidden || document.querySelector("dialog[open]")) return;
      emit(`Key.${key}`);
    };
    keysBar.addEventListener("pointerdown", (event) => {
      const button = event.target.closest("button[data-key]");
      if (!button) return;
      event.preventDefault(); // le doigt ne sélectionne rien et ne fait pas défiler
      stop();
      pressedByFinger = button;
      button.classList.add("pressed");
      press(button.dataset.key);
      // Gardée enfoncée, comme une touche du clavier, elle se répète.
      repeat = setTimeout(() => { repeat = setInterval(() => press(button.dataset.key), 80); }, 400);
    });
    for (const end of ["pointerup", "pointercancel", "pointerleave"]) keysBar.addEventListener(end, stop);
    // Un lecteur d'écran (TalkBack, VoiceOver) touche un bouton sans poser le doigt dessus : un
    // clic seul compte alors pour un appui. Après un appui du doigt, le clic qui suit est ignoré.
    keysBar.addEventListener("click", (event) => {
      const button = event.target.closest("button[data-key]");
      if (!button) return;
      if (pressedByFinger === button) pressedByFinger = null;
      else press(button.dataset.key);
    });
  }
  let beats = []; // une horloge par règle Every : { ms, valeur, minuterie }
  function launch(clock, rank) {
    clearInterval(clock.timer);
    clock.timer = setInterval(() => {
      if (document.hidden || inPoints || inWorld || !crossroads.hidden || entryInProgress) return;
      emit(`every:${rank}`);
    }, clock.ms);
  }
  // Les données venues du serveur (data: Data(from: "stock.json", every: 30s)). La page va
  // chercher le fichier, rangé à côté d'elle, et le donne à l'arbitre, qui range ce qu'il veut
  // bien prendre. Elle ne parle qu'au serveur d'où elle vient.
  let refresh = 0;
  // Lit une réponse sans jamais garder plus de `max` octets : au-delà, rien n'est pris, et la
  // lecture s'arrête aussitôt (une réponse trop grosse n'occupe pas la mémoire de la page).
  async function readCapped(response, max) {
    if (Number(response.headers.get("content-length") ?? 0) > max) return null;
    const reader = response.body?.getReader();
    if (!reader) return null;
    const chunks = [];
    let size = 0;
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.length;
      if (size > max) {
        reader.cancel().catch(() => {});
        return null;
      }
      chunks.push(value);
    }
    return new Blob(chunks).arrayBuffer();
  }
  // Les données de la page (ADR-030) : 64 Ko au plus, en 10 secondes au plus, une lecture à la
  // fois. Avec un nom, Data(name: Stock, …), la page dit ce qui s'est passé (ADR-064) :
  // Stock.done quand elles sont arrivées et rangées ; Stock.failed sans réseau, sur une erreur du
  // serveur, ou quand elles sont trop grosses, illisibles ou trop lentes. Sans nom, rien n'est dit.
  let dataReading = false;
  let dataLast = 0;
  async function loadData(file, for_, name) {
    if (dataReading) return;
    dataReading = true;
    dataLast = Date.now();
    let arrived = false;
    const stop = new AbortController();
    const late = setTimeout(() => stop.abort(), 10000);
    try {
      const discreet = fromElsewhere(for_) ? { credentials: "omit", referrerPolicy: "no-referrer" } : {};
      const response = await fetch(folderOf(for_) + file, { cache: "no-store", headers: { accept: "application/json" }, signal: stop.signal, ...discreet });
      // Des données de 64 Ko au plus (DATA_BYTES dans le moteur) : au-delà, elles sont refusées.
      const buffer = response.ok ? await readCapped(response, 65536) : null;
      if (buffer && for_ === path) { // on a pu changer de fichier entre-temps
        const json = new TextDecoder().decode(buffer);
        // Un objet JSON, `{ "stock": 4 }` : autre chose ne se lit pas.
        const read = JSON.parse(json);
        if (read && typeof read === "object" && !Array.isArray(read)) {
          const before = states.get(path) ?? "";
          const after = store(receive(source, before, json));
          if (after && after !== before) changeState(after);
          arrived = true;
        }
      }
    } catch { /* pas de réseau, trop lent, ou illisible : la page garde ses valeurs */ }
    clearTimeout(late);
    dataReading = false;
    if (name && for_ === path) emit(`${name}.${arrived ? "done" : "failed"}`);
  }
  function setData() {
    clearInterval(refresh);
    clearTimeout(dataLater);
    dataLater = 0;
    const [file, rhythm, name] = data(source).split("|");
    if (!file) return;
    const for_ = path;
    loadData(file, for_, name);
    if (Number(rhythm) > 0) {
      refresh = setInterval(() => {
        if (for_ !== path) clearInterval(refresh);
        else if (!document.hidden) loadData(file, for_, name);
      }, Number(rhythm));
    }
  }
  // Relire les données à la demande, On(Retry.tap, effect: Stock.refresh) (ADR-064). Pendant
  // une lecture, la demande est sans objet : la lecture en cours répondra (done ou failed). Une
  // seconde au moins entre deux lectures, même si une règle redemande à chaque échec : une
  // demande trop proche attend son tour, elle n'est pas perdue (sinon « Chargement… » resterait).
  let dataLater = 0;
  function refreshData(name) {
    const [file, , dataName] = data(source).split("|");
    if (!file || dataName !== name || dataReading || dataLater) return;
    const wait = dataLast + 1000 - Date.now();
    if (wait <= 0) return loadData(file, path, name);
    const for_ = path;
    dataLater = setTimeout(() => {
      dataLater = 0;
      if (for_ === path) loadData(file, for_, name);
    }, wait);
  }

  function setClocks() {
    setData();
    waitings.forEach((waiting) => clearTimeout(waiting.timer));
    waitings = [];
    setDelays();
    followTime();
    listenedKeys = keypresses(source).split(";").filter(Boolean);
    setShortcutsButton();
    setKeysBar();
    beats.forEach((clock) => clearInterval(clock.timer));
    beats = clocks(source).split(";").filter(Boolean).map((chunk) => {
      const [ms, values] = chunk.split(":");
      return { ms: Number(ms), values: values.split(","), timer: 0 };
    });
    beats.forEach(launch);
  }
  // Les attentes (After, ADR-039) : chacune sonne une fois, après sa durée, à partir du moment où
  // elle court. Le moteur dit lesquelles courent pour l'état du moment ; la page tient le compte
  // de celles qui ont déjà sonné, et l'oublie quand une attente cesse de courir.
  let waitings = [];
  function setDelays() {
    const list = delays(source, states.get(path) ?? "").split(";").filter(Boolean);
    list.forEach((chunk, rank) => {
      const [ms, short] = chunk.split(":");
      const waiting = (waitings[rank] ??= { timer: 0, rung: false });
      if (short === "1" && !waiting.timer && !waiting.rung) {
        const for_ = path;
        waiting.timer = setTimeout(() => {
          waiting.timer = 0;
          waiting.rung = true;
          if (for_ === path) emit(`after:${rank}`);
        }, Number(ms));
      } else if (short !== "1") {
        clearTimeout(waiting.timer);
        waiting.timer = 0;
        waiting.rung = false;
      }
    });
  }
  // L'heure du visiteur (ADR-039) : celle de son appareil, donnée au moteur à chaque minute.
  function giveTime() {
    const d = new Date();
    set_now(d.getFullYear(), d.getMonth() + 1, d.getDate(), ((d.getDay() + 6) % 7) + 1, d.getHours(), d.getMinutes());
  }
  let nextMinute = 0;
  function followTime() {
    clearTimeout(nextMinute);
    if (!reads_time(source)) return;
    const d = new Date();
    nextMinute = setTimeout(() => {
      giveTime();
      const before = states.get(path) ?? "";
      const after = store(advance_clock(source, before));
      if (after && after !== before) changeState(after);
      followTime();
    }, (60 - d.getSeconds()) * 1000 - d.getMilliseconds() + 50);
  }
  // Les listes qui changent pendant la visite (ADR-044) : quand une liste change, le moteur
  // fabrique ses lignes à nouveau, et la page les pose à la place des anciennes.
  function redrawLists() {
    const written = states.get(path) ?? "";
    for (const container of root.querySelectorAll("[data-list]")) {
      const name = container.dataset.list;
      const chunk = written.split(";").find((m) => m.startsWith(`${name}=[`)) ?? "";
      if (container.dataset.seen === chunk) continue;
      container.dataset.seen = chunk;
      // Chaque répétition dit où elle est écrite (`tasks@12:5`) : deux répétitions d'une même
      // liste se redessinent chacune avec son modèle.
      placeLines(container, list_html(source, base, written, container.dataset.repeat ? `${name}@${container.dataset.repeat}` : name));
      window.__holoWatchEntrances?.(container);
    }
  }
  // Pose les nouvelles lignes d'une liste en gardant, telles quelles, celles dont la clé et le
  // contenu n'ont pas changé (ADR-057) : le champ où l'on écrit, un pli ouvert, le focus restent.
  // On compare au HTML que le moteur avait fabriqué, pas à celui du moment : un pli ouvert
  // ajoute « open » à sa ligne sans qu'elle ait changé.
  const built = new WeakMap();
  function placeLines(container, html) {
    const model = document.createElement("template");
    model.innerHTML = html;
    // Le focus du clavier (lot 2 du web) : si la ligne où il est doit être refaite, il passe au
    // même bouton (même nom, même place) de la nouvelle ligne. Celle-ci est retrouvée par sa
    // clé quand l'auteur l'a choisie (`key: id`, une clé en « k: »), sinon par son rang.
    const focused = document.activeElement;
    const focusLine = focused && container.contains(focused) ? focused.closest(".holo-line") : null;
    const twin = (line) => {
      const same = (e) => e.tagName === focused.tagName && (e.dataset.name ?? "") === (focused.dataset.name ?? "");
      const index = [...focusLine.querySelectorAll(focused.tagName)].filter(same).indexOf(focused);
      return [...line.querySelectorAll(focused.tagName)].filter(same)[index];
    };
    const oldOnes = new Map([...container.children].filter((l) => l.dataset.key).map((l) => [l.dataset.key, l]));
    const lines = [...model.content.children].map((newOne) => {
      const oldOne = oldOnes.get(newOne.dataset.key);
      const before = oldOne && (built.get(oldOne) ?? oldOne.innerHTML);
      if (before === newOne.innerHTML) {
        oldOnes.delete(newOne.dataset.key);
        oldOne.dataset.rank = newOne.dataset.rank; // le rang sert aux gestes : Done.tap@2
        built.set(oldOne, before);
        return oldOne;
      }
      built.set(newOne, newOne.innerHTML);
      return newOne;
    });
    // On ne touche pas aux lignes déjà à leur place : déplacer un nœud lui ferait perdre le focus.
    lines.forEach((line, rank) => {
      if (container.children[rank] !== line) container.insertBefore(line, container.children[rank] ?? null);
    });
    while (container.children.length > lines.length) container.lastElementChild.remove();
    // Une ligne retirée : le focus passe à la ligne qui prend sa place (ou à la dernière).
    if (focusLine && !focusLine.isConnected && lines.length) {
      const key = focusLine.dataset.key ?? "";
      const again = (key.startsWith("k:") && lines.find((l) => l.dataset.key === key)) || lines[Math.min(Number(focusLine.dataset.rank), lines.length - 1)];
      // Le même bouton, ou, s'il n'y est plus (« Fait » devenu « Rouvrir »), le premier de la ligne.
      (twin(again) ?? again.querySelector("button, a[href], input, select, textarea, summary"))?.focus({ preventScroll: true });
    }
  }
  // Un nouvel état : la page le montre, le garde, et regarde quelles attentes courent.
  function changeState(after) {
    states.set(path, after);
    redrawLists();
    showValues();
    placePixels();
    keep();
    setDelays();
  }

  // Un geste vient de changer des valeurs : l'horloge de chacune repart de zéro. « Play » remet
  // le temps à trente : la première seconde dure une vraie seconde.
  function restartClocks(signal) {
    if (!beats.length || signal.startsWith("every:")) return;
    const changedOnes = touched_ones(source, signal).split(";");
    beats.forEach((clock, rank) => { if (clock.values.some((value) => changedOnes.includes(value))) launch(clock, rank); });
  }
  // Ce que la page garde d'une visite à l'autre reste dans le navigateur du visiteur.
  function keep() {
    try {
      const kept = to_keep(source, states.get(path) ?? "");
      if (kept) localStorage.setItem(`holo:${path}`, kept);
    } catch { /* stockage refusé ou plein : la page marche sans */ }
  }

  // Un résultat de l'arbitre peut porter, sous le nom « ! », des capacités demandées par une
  // règle de temps ou une règle qui guette (un son à faire entendre). On les applique, et on
  // rend l'état sans elles.
  function store(result) {
    const chunks = result.split(";");
    const requests = chunks.find((chunk) => chunk.startsWith("!="));
    if (!requests) return result;
    for (const effect of requests.slice(2).split(",")) apply(effect);
    return chunks.filter((chunk) => chunk !== requests).join(";");
  }

  // ?values dans l'adresse (ADR-054) : un petit panneau montre les valeurs de la page à chaque
  // changement, pour que l'auteur voie ce que ses règles font. Il ne sert qu'à l'essai.
  const valuesPanel = params.has("values") ? document.body.appendChild(Object.assign(document.createElement("aside"), { id: "holo-values", ariaLabel: "Les valeurs de la page" })) : null;
  if (valuesPanel) {
    Object.assign(valuesPanel.style, { position: "fixed", left: "8px", bottom: "8px", zIndex: 30, maxWidth: "min(92vw, 360px)", maxHeight: "40vh", overflow: "auto", font: "13px/1.5 ui-monospace, Consolas, monospace", background: "#000c", color: "#e8e8e8", border: "1px solid #888", borderRadius: "8px", padding: "8px 10px", whiteSpace: "pre-wrap" });
  }
  // Le dernier geste (ADR-056) : son signal, et ce qu'il a changé.
  let lastGesture = null;
  function theValues(written) {
    return new Map(written.split(";").filter(Boolean).filter((m) => !m.startsWith("!=")).map((m) => [m.slice(0, m.indexOf("=")), m.slice(m.indexOf("=") + 1)]));
  }
  function showPanel(written) {
    if (!valuesPanel) return;
    const lines = written.split(";").filter(Boolean).filter((m) => !m.startsWith("!=")).map((chunk) => {
      const cut = chunk.indexOf("=");
      const name = chunk.slice(0, cut);
      const raw = chunk.slice(cut + 1);
      if (raw.startsWith("'")) return `${name} = "${decodeURIComponent(raw.slice(1))}"`;
      if (raw.startsWith("[")) {
        const elements = raw.slice(1, -1).split(",").filter(Boolean).map((e) => decodeURIComponent(e).replace(/^\u001d/, "").replace(/&/g, ", ").replace(/=/g, ": ")).map((e) => decodeURIComponent(e));
        return `${name} = ${elements.length} élément(s)${elements.length ? "\n  · " + elements.join("\n  · ") : ""}`;
      }
      return `${name} = ${raw}`;
    });
    let gesture = "";
    if (lastGesture) {
      const before = theValues(lastGesture.before), after = theValues(lastGesture.after);
      const readable = (v) => (v === undefined ? "—" : v.startsWith("'") ? `"${decodeURIComponent(v.slice(1))}"` : v.startsWith("[") ? `${v.slice(1, -1).split(",").filter(Boolean).length} élément(s)` : v);
      const changes = [...after.keys()].filter((n) => before.get(n) !== after.get(n)).map((n) => `  ${n} : ${readable(before.get(n))} → ${readable(after.get(n))}`);
      gesture = `\n\nDernier geste : ${lastGesture.signal}\n${changes.join("\n") || "  (rien n'a changé)"}`;
    }
    valuesPanel.textContent = `Valeurs de la page\n${lines.join("\n") || "(aucune)"}${gesture}`;
  }

  // Écrit les valeurs de la page là où ses textes les montrent : « {cart} ».
  function showValues(or_ = root, written = states.get(path) ?? "", fileText = source) {
    if (or_ === root) showPanel(written);
    // Un texte voyage codé, précédé d'une apostrophe : buyer='Zo%C3%A9. Un nombre, tel quel.
    // Une liste voyage entre crochets : elle se montre par son nombre d'éléments (ADR-044).
    const readable = (value) => (value.startsWith("'") ? decodeURIComponent(value.slice(1)) : value.startsWith("[") ? String(value.slice(1, -1).split(",").filter(Boolean).length) : value);
    const values = new Map(written.split(";").filter(Boolean).map((chunk) => {
      const cut = chunk.indexOf("=");
      return [chunk.slice(0, cut), readable(chunk.slice(cut + 1))];
    }));
    // Une valeur à format (ADR-043) est écrite par le moteur, dans la langue de la page.
    const language = or_.querySelector?.("[data-lang]")?.dataset.lang || document.documentElement.lang || "fr";
    for (const place of or_.querySelectorAll("[data-state]")) {
      if (!values.has(place.dataset.state)) continue;
      const value = values.get(place.dataset.state);
      // Une date se montre dans la langue de la page (ADR-067) ; un nombre, avec son format.
      const format = place.dataset.format;
      place.textContent = !format ? value : format === "date" || format === "weekday" ? format_date(value, format, language) : format_value(place.dataset.state, Number(value), format, language);
    }
    // Un champ et une case montrent leur valeur ; on ne récrit pas le champ où l'on est en train d'écrire.
    for (const field of or_.querySelectorAll("[data-bind]")) {
      if (!values.has(field.dataset.bind)) continue;
      if (field.type === "checkbox") field.checked = values.get(field.dataset.bind) !== "0";
      // Un choix en boutons ronds : celui dont l'option est la valeur est coché (ADR-038).
      else if (field.type === "radio") field.checked = field.value === values.get(field.dataset.bind);
      // Un fichier choisi ne se récrit pas ; une règle peut seulement le vider : photo.set("").
      else if (field.type === "file") { if (values.get(field.dataset.bind) === "") field.value = ""; }
      // Un nombre à virgule voyage à son échelle, 12,50 → 1250 (ADR-066) : le champ montre 12.50.
      else if (field !== document.activeElement) {
        const raw = values.get(field.dataset.bind);
        const places = Number(field.dataset.places ?? 0);
        field.value = places ? (Number(raw) / 10 ** places).toFixed(places) : raw;
      }
    }
    // Une barre de progression suit sa valeur : Progress(value: lives) (ADR-042).
    for (const bar of or_.querySelectorAll("[data-progress]")) {
      if (values.has(bar.dataset.progress)) bar.value = Number(values.get(bar.dataset.progress));
    }
    // Sur un plateau, un bloc suit les valeurs qui disent sa place : Point(x: starX, y: starY).
    for (const axis of ["x", "y"]) {
      for (const placed of or_.querySelectorAll(`[data-${axis}]`)) {
        if (values.has(placed.dataset[axis])) placed.style.setProperty(`--${axis}`, Math.min(100, Number(values.get(placed.dataset[axis]))));
      }
    }
    // Un dessin suit les nombres qui disent ses mesures : Circle(y: sun) (ADR-086).
    for (const shape of or_.querySelectorAll("[data-svg]")) {
      for (const pair of shape.dataset.svg.split(" ")) {
        const [attribute, name] = pair.split(":");
        if (values.has(name)) shape.setAttribute(attribute, Math.min(4000, Number(values.get(name))));
      }
    }
    // Les conditions : If(count, is: 0). C'est le moteur qui répond ; la page ne compare rien
    // elle-même, elle cache ce que le moteur dit faux.
    const blocks = or_.querySelectorAll("[data-if],[data-else]");
    if (!blocks.length) return;
    const responses = new Map(conditions(fileText, written).split(";").filter(Boolean).map((chunk) => {
      const cut = chunk.lastIndexOf(":");
      return [chunk.slice(0, cut), chunk.slice(cut + 1) === "1"];
    }));
    for (const block of blocks) {
      if (responses.has(block.dataset.if)) block.hidden = !responses.get(block.dataset.if);
      // Le « sinon » (ADR-039) : montré quand la condition est fausse.
      else if (responses.has(block.dataset.else)) block.hidden = responses.get(block.dataset.else);
    }
  }

  // Un signal est émis (Add.tap) : l'arbitre du moteur dit ce que deviennent les valeurs, puis
  // les autres effets demandés par les règles sont appliqués.
  function emit(signal) {
    const before = states.get(path) ?? "";
    const after = store(arbitrate(source, before, signal));
    if (valuesPanel) lastGesture = { signal, before, after };
    // Ce qui apparaît ou disparaît déplace le reste de la page : changerLEtat replace les pixels.
    if (after !== before) changeState(after);
    restartClocks(signal);
    for (const effect of effects(source, signal).split(",").filter(Boolean)) {
      apply(effect, signal);
    }
  }

  // `reprendre` : la page est déjà là, fabriquée par le serveur (ADR-033). Le moteur la prend
  // en main sans la redessiner : ce qui a été écrit en l'attendant reste, le focus aussi, et
  // les mouvements (ADR-034) ne repartent pas de zéro.
  function displaySite(sitePath, { inHistory = true, resume = false } = {}) {
    exitPoints();
    closeCrossroads();
    if (sitePath.startsWith("~")) {
      openWorld(sitePath.slice(1), inHistory);
      return;
    }
    if (inWorld) {
      inWorld = false;
      document.body.classList.remove("in-world");
      pause(true);
      modeButton.textContent = "Carrefour";
    }
    site = sitePath;
    hoveredOnes.clear();
    if (resume) adoptInputs();
    else root.innerHTML = flat_view(source, base, site);
    // Une liste gardée d'une visite précédente n'est pas celle que le serveur a fabriquée (ADR-044).
    redrawLists();
    showValues();
    window.__holoWatchEntrances?.();
    frame = root.querySelector(".holo-Page");
    page = frame.querySelector("main");
    document.title = frame.dataset.title || "HoloCode";
    // La langue de la page (ADR-038) : un lecteur d'écran la prononce avec la bonne voix.
    if (frame.dataset.lang) document.documentElement.lang = frame.dataset.lang;
    // Le fond de la fenêtre est celui du site : jamais de noir autour d'un site clair.
    const background = getComputedStyle(frame).backgroundColor;
    document.documentElement.style.setProperty("--background", background);
    document.body.style.background = background;
    grow(1);
    placePixels();
    if (!resume) scrollTo(0, 0);
    if (inHistory && !fromElsewhere(path) && decodeURIComponent(location.hash.slice(1)) !== site) {
      history.pushState(history.state, "", site ? `#${site}` : location.pathname + location.search);
    }
    lastPassage = performance.now();
    // Chez quelqu'un d'autre, on le dit tant qu'on y est (B-08).
    const origin = document.getElementById("origin");
    origin.hidden = !fromElsewhere(path);
    origin.firstElementChild.textContent = `Vous êtes chez ${new URL(path, location.href).host}`;
    // Les fichiers où mènent les points de cette page sont lus d'avance, discrètement.
    // Seulement ceux du même serveur : on ne contacte pas le serveur de quelqu'un d'autre tant
    // que le visiteur n'a pas ouvert le carrefour.
    for (const content of containedSites()) {
      if (content.file && !fromElsewhere(content.file)) read(content.file);
    }
  }

  // Ce que le visiteur a écrit ou coché avant l'arrivée du moteur passe par l'arbitre, comme
  // une saisie ordinaire : rien n'est perdu, et la valeur est bornée comme d'habitude.
  function adoptInputs() {
    for (const field of root.querySelectorAll("[data-bind]")) {
      // Un bouton rond ne compte que s'il est coché.
      if (field.type === "radio" && !field.checked) continue;
      const changed = field.type === "checkbox" || field.type === "radio" ? field.checked !== field.defaultChecked : field.value !== field.defaultValue;
      if (!changed) continue;
      const written = field.type === "checkbox" ? (field.checked ? "1" : "0") : field.type === "file" ? allowedFile(field) : field.value;
      states.set(path, store(input(source, states.get(path) ?? "", field.dataset.bind, written)));
    }
  }

  // Un monde calculé à partir d'une graine s'ouvre en profondeur : c'est le Big Bang. On y
  // zoome, on entre dans ses points, sans fin. Son adresse : fichier.holo#~graine.
  async function openWorld(seed, inHistory) {
    if (!/^[0-9]+$/.test(seed)) return;
    const point = `Point(name: World, seed: ${seed}, fragments: 12)`;
    inWorld = true;
    document.body.classList.add("in-world");
    document.body.style.background = "#000";
    document.title = `Monde ${seed.slice(-4)}`;
    modeButton.textContent = "Retour";
    if (inHistory) history.pushState(null, "", `#~${seed}`);
    if (!engineStarted) {
      engineStarted = true;
      await start_engine(area(), point, 1);
    } else {
      remove_mosaic();
      change_world(point, 1);
      pause(false);
    }
  }

  // Les points plantés dans un pixel de la page : chacun se place juste au-dessus du bloc
  // que son « above » désigne, à l'extrémité droite de la page.
  function placePixels() {
    for (const pixel of page.querySelectorAll(".holo-pixel")) {
      const landmark = page.querySelector(`[data-name="${CSS.escape(pixel.dataset.above)}"]`);
      if (!landmark) continue;
      pixel.style.left = `${page.clientWidth - 1}px`;
      pixel.style.top = `${landmark.offsetTop - 6}px`;
    }
  }

  // Les sites contenus dans le site affiché : ses points qui ont un intérieur.
  function containedSites() {
    return [...page.querySelectorAll(".holo-Point, .holo-pixel")]
      .filter((point) => point.dataset.file || root.querySelector(`section[data-world="${CSS.escape(point.dataset.name)}"]`))
      .map((point) => ({
        file: point.dataset.file ?? null, // le monde de ce point est un autre fichier
        path: [site, point.dataset.name].filter(Boolean).join("/"),
        name: point.dataset.name,
        color: point.style.getPropertyValue("--holo-color") || "#E9B44C",
      }));
  }

  // ---------------------------------------------------------------- le zoom ordinaire

  // Grossit la page vivante, comme le zoom d'un navigateur : le texte reste du texte.
  function grow(value, x = innerWidth / 2, y = innerHeight / 2) {
    // La page peut être posée un peu en retrait (décrochée, ADR-069) : on compte depuis son coin.
    const [left, top] = [frame.offsetLeft, frame.offsetTop];
    const [beforeX, beforeY] = [(scrollX + x - left) / sharpZoom, (scrollY + y - top) / sharpZoom];
    sharpZoom = value;
    // Un vrai agrandissement, depuis le coin de la page : la mise en page ne bouge pas, tout
    // grossit ensemble. La zone à faire défiler grandit d'autant.
    const enlarged = sharpZoom !== 1;
    const width = frame.style.width || `${frame.offsetWidth}px`;
    Object.assign(frame.style, { transformOrigin: "0 0", transform: enlarged ? `scale(${sharpZoom})` : "", width: enlarged ? width : "" });
    Object.assign(root.style, { width: enlarged ? `${frame.offsetWidth * sharpZoom}px` : "", height: enlarged ? `${frame.offsetHeight * sharpZoom}px` : "", overflow: enlarged ? "hidden" : "", cursor: enlarged ? "grab" : "" });
    // Ce qui était sous le doigt y reste.
    scrollTo(beforeX * sharpZoom + left - x, beforeY * sharpZoom + top - y);
  }

  // ---------------------------------------------------------------- vue points

  const inAddress = (blob) => new Promise((ok, failed) => {
    const reader = new FileReader();
    reader.onload = () => ok(reader.result);
    reader.onerror = failed;
    reader.readAsDataURL(blob);
  });

  // La page telle qu'on la voit, redessinée dans une image aux dimensions de la fenêtre.
  // Chrome ne donne pas les pixels d'une page ; mais celle-ci, c'est le moteur qui l'a
  // fabriquée : on peut la remettre dans une image (un SVG qui la contient).
  async function pageImage() {
    const copy = root.cloneNode(true);
    // Une image ne peut pas aller chercher d'autres fichiers : celles de la page sont copiées dedans.
    const originals = root.querySelectorAll("img");
    for (const [i, img] of [...copy.querySelectorAll("img")].entries()) {
      try {
        img.setAttribute("src", await inAddress(await (await fetch(originals[i].src)).blob()));
      } catch {
        img.removeAttribute("src"); // une image qu'un autre serveur ne laisse pas copier
      }
    }
    const [l, h] = [innerWidth, innerHeight];
    const svg =
      `<svg xmlns="http://www.w3.org/2000/svg" width="${l * density}" height="${h * density}" viewBox="0 0 ${l} ${h}">` +
      `<foreignObject width="${l}" height="${h}"><div xmlns="http://www.w3.org/1999/xhtml" style="width:${l}px;height:${h}px;overflow:hidden">` +
      `<style>.holo-Page{min-height:${h / sharpZoom}px!important}</style>` +
      `<div style="margin:${-scrollY}px 0 0 ${-scrollX}px">${new XMLSerializer().serializeToString(copy)}</div></div></foreignObject></svg>`;
    image.src = "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svg);
    await image.decode();
    const canvas = new OffscreenCanvas(l * density, h * density);
    const drawing = canvas.getContext("2d");
    drawing.drawImage(image, 0, 0, canvas.width, canvas.height);
    image.style.width = `${canvas.width}px`;
    image.style.height = `${canvas.height}px`;
    return drawing.getImageData(0, 0, canvas.width, canvas.height);
  }

  // Le suivi ne tourne que pendant qu'on bouge : immobile, il s'arrête, comme le moteur.
  let trackingActive = false;
  let lastCamera = "";
  let stillFrames = 0;

  function wakeTracking() {
    if (!inPoints) return;
    wake();
    stillFrames = 0;
    if (!trackingActive) {
      trackingActive = true;
      requestAnimationFrame(trackPoints);
    }
  }

  function trackPoints() {
    trackingActive = false;
    if (!inPoints) return;
    const camera = mosaic_camera();
    if (camera.length) {
      const [cx, cy, scale, opacity, level, points, yaw, pitch, distance, scaleAtRest] = camera;
      // Revenu exactement à la vue de départ, de face : c'est le site normal, on y retourne.
      const atRest = Math.abs(scale / scaleAtRest - 1) < 1e-9 && yaw === 0 && pitch === 0;
      if (atRest && hasLeftRest) {
        // Si l'on s'est déplacé pendant la vue points, la page vivante reprend au même endroit.
        const [offsetX, offsetY] = [(cx - image.naturalWidth / 2) / density, (cy - image.naturalHeight / 2) / density];
        exitPoints();
        scrollBy(offsetX, offsetY);
        return;
      }
      hasLeftRest ||= !atRest;
      // L'image de la page suit la même vue, de face comme de biais, et s'efface quand les
      // points prennent le relais.
      image.style.transform =
        `translate(${innerWidth / 2}px,${innerHeight / 2}px) perspective(${distance}px) rotateX(${pitch}rad) rotateY(${yaw}rad) ` +
        `translate(${-cx * scale}px,${-cy * scale}px) scale(${scale})`;
      image.style.opacity = 1 - opacity;
      state.textContent = opacity === 0
        ? `${number.format(image.naturalWidth * image.naturalHeight)} points — zoomez encore`
        : `${number.format(points)} points à l'écran · morcelés ${level} fois`;
    }
    const key = camera.join();
    stillFrames = key === lastCamera ? stillFrames + 1 : 0;
    lastCamera = key;
    if (stillFrames > 6) return; // plus rien ne bouge : on s'arrête jusqu'au prochain geste
    trackingActive = true;
    requestAnimationFrame(trackPoints);
  }

  // On zoome vers un endroit de l'écran, comme si on s'en approchait.
  async function approach(x, y) {
    if (calm.matches) return; // animations réduites : on n'avance pas vers le point, on ouvre directement
    for (let step = 0; step < 45 && mosaic_camera()[2] * density * sharpZoom < 280; step++) {
      area().dispatchEvent(new WheelEvent("wheel", { deltaY: -70, clientX: x, clientY: y, cancelable: true }));
      wakeTracking();
      await new Promise(requestAnimationFrame);
    }
  }

  // En vue points, toucher un point planté par l'auteur l'active : on s'en approche, puis le
  // carrefour s'ouvre sur le site qu'il contient.
  async function touchPoint(x, y) {
    const under = mosaic_under(x, y);
    if (!under.length) return;
    const tolerance = Math.max(1.5 * density * sharpZoom, 14 / mosaic_camera()[2]);
    const planted = plantedPixels.find((p) => Math.abs(p.x - under[0]) < tolerance && Math.abs(p.y - under[1]) < tolerance);
    if (!planted) return;
    await approach(x, y);
    exitPoints();
    emit(`${planted.name}.tap`);
  }

  // La page devient un ensemble de points, un par pixel. Au départ rien ne change à l'écran :
  // un point fait exactement un pixel.
  function enterPoints() {
    if (inPoints || !crossroads.hidden || !activePoints) return Promise.resolve();
    entryInProgress ??= (async () => {
      density = Math.min(requestedDensity, Math.sqrt(POINTS_MAX / (innerWidth * innerHeight)));
      const pixels = await pageImage();
      const colors = new Uint8Array(pixels.data.buffer);
      plantedPixels = [...page.querySelectorAll(".holo-pixel")].map((pixel) => {
        const pixelFrame = pixel.getBoundingClientRect();
        return { name: pixel.dataset.name, x: (pixelFrame.left + pixelFrame.width / 2) * density, y: (pixelFrame.top + pixelFrame.height / 2) * density };
      });
      document.body.classList.add("in-points");
      if (!engineStarted) {
        engineStarted = true;
        await start_mosaic(area(), colors, pixels.width, pixels.height, source, sharpZoom);
      } else {
        place_mosaic(colors, pixels.width, pixels.height, source, sharpZoom);
        pause(false);
      }
      // La page reste sous les points, invisible mais lisible : un lecteur d'écran la lit
      // encore (ADR-061). Tab la fait revenir.
      page.classList.add("under-points");
      announce("Vue points : la page est devenue des points. Son texte reste lisible au lecteur d'écran ; Tab ramène la vue web.");
      pointsButton.textContent = "Vue web";
      inPoints = true;
      hasLeftRest = false;
      wakeTracking();
    })().finally(() => { entryInProgress = null; });
    return entryInProgress;
  }

  function exitPoints() {
    if (!inPoints) return;
    inPoints = false;
    mosaic_front();
    mosaic_turn(false);
    remove_mosaic();
    pause(true);
    turnButton.setAttribute("aria-pressed", "false");
    document.body.classList.remove("in-points");
    pointsButton.textContent = "Vue points";
    // L'image de la page et les repères ne servent plus : on rend la mémoire (B-10).
    image.removeAttribute("src");
    plantedPixels = [];
    page.classList.remove("under-points");
    announce("Vue web.");
  }

  // Ce que le lecteur d'écran annonce : le passage d'une vue à l'autre (ADR-061).
  function announce(text) {
    const announcement = document.getElementById("announcement");
    if (!announcement) return;
    announcement.textContent = "";
    setTimeout(() => { announcement.textContent = text; }, 50);
  }
  // Le clavier arrive sur la page pendant la vue points : la vue web revient, pour qu'on voie
  // où l'on est.
  root.addEventListener("focusin", () => { if (inPoints) exitPoints(); });

  // Un pincement arrive comme une molette avec Ctrl, par petits pas : on les grossit.
  const zoomStep = (event) => event.deltaY * (Math.abs(event.deltaY) < 50 ? 6 : 1);

  // Zoomer sur la page (Ctrl + molette, ou pincer). D'abord un zoom ordinaire, où la page reste
  // un site qu'on lit et qu'on copie ; au-delà de Points(after:), ses pixels deviennent des points.
  // Un seul chemin pour tout zoom, qu'il vienne de la molette ou de deux doigts : la page
  // vivante d'abord, les points ensuite.
  async function zoomIn(factor, clientX, clientY) {
    const deltaY = -Math.log2(factor) / 0.003;
    if (inPoints) {
      area().dispatchEvent(new WheelEvent("wheel", { deltaY, clientX, clientY, cancelable: true }));
      wakeTracking();
      return;
    }
    if (!crossroads.hidden || !frame || entryInProgress || inWorld || !zoomActive) return;
    const wanted = sharpZoom * factor;
    if (wanted <= zoomBeforePoints && (wanted >= 1 || sharpZoom > 1)) {
      grow(Math.max(1, wanted), clientX, clientY);
      return;
    }
    // Garde-fou : on ne dézoome pas en deçà de la page entière, sauf si le fichier le permet
    // (Zoom(shrink: true)).
    if (wanted < 1 && !reduce) {
      // Dézoomer alors que la page est déjà entière : on ressort du monde où l'on est.
      if (sharpZoom === 1) goUp();
      return;
    }
    if (wanted > zoomBeforePoints && sharpZoom < zoomBeforePoints) grow(zoomBeforePoints, clientX, clientY);
    // Sans points demandés par l'auteur, ou avec les animations réduites par le visiteur : la
    // page reste une page, on a grossi autant que permis.
    if (!activePoints || calm.matches) return;
    await enterPoints();
    area().dispatchEvent(new WheelEvent("wheel", { deltaY, clientX, clientY, cancelable: true }));
  }

  // À la souris : Ctrl + molette (un pavé tactile envoie la même chose quand on pince).
  addEventListener("wheel", (event) => {
    // Par défaut, c'est le navigateur qui grossit la page, comme pour tout site (ADR-069).
    if (!byEngine) return;
    // Sinon Chrome grossit ou réduit toute la fenêtre, et le site sort de son cadre.
    if (event.ctrlKey) event.preventDefault();
    if (inPoints || !event.ctrlKey) return; // en vue points, la zone de dessin reçoit la molette elle-même
    zoomIn(2 ** (-zoomStep(event) * 0.003 * zoomSpeed), event.clientX, event.clientY);
  }, { passive: false });

  // Au doigt : pincer la page. Le geste commencé sur la page continue sans lever les doigts
  // quand ses pixels deviennent des points, et dans l'autre sens quand on revient.
  let pinch = null; // l'écartement des deux doigts au dernier mouvement
  // `touches` : les doigts posés sur l'écran (le nom du navigateur ; ce ne sont pas des touches
  // du clavier : la traduction en anglais l'avait confondu, et le pincement ne marchait plus).
  const twoFingers = (touches) => ({
    gap: Math.hypot(touches[0].clientX - touches[1].clientX, touches[0].clientY - touches[1].clientY),
    x: (touches[0].clientX + touches[1].clientX) / 2,
    y: (touches[0].clientY + touches[1].clientY) / 2,
  });
  addEventListener("touchstart", (event) => {
    // Un pincement commencé sur la zone de dessin est suivi par le moteur lui-même.
    const onPage = byEngine && event.touches.length === 2 && !inPoints && !inWorld && crossroads.hidden;
    pinch = onPage ? twoFingers(event.touches).gap : null;
  }, { capture: true, passive: true });
  addEventListener("touchmove", (event) => {
    if (pinch === null || event.touches.length !== 2) return;
    event.preventDefault(); // ni le zoom ni le défilement du navigateur pendant qu'on pince
    const { gap, x, y } = twoFingers(event.touches);
    if (Math.abs(gap / pinch - 1) < 0.015) return; // un tremblement n'est pas un pincement
    zoomIn(gap / pinch, x, y);
    pinch = gap;
  }, { capture: true, passive: false });
  for (const end of ["touchend", "touchcancel"]) {
    addEventListener(end, (event) => { if (event.touches.length < 2) pinch = null; }, { capture: true, passive: true });
  }

  // ---------------------------------------------------------------- le carrefour

  // Un portail : un rond lumineux qui montre le site où il mène.
  function portal({ path: versSite, name, color, computed = false, file = null, text = null, back = false, toRead = false, inert = false }, size, vers) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = vers ? "portal to" : "portal";
    if (back) button.dataset.back = "yes";
    else if (file) button.dataset.file = file;
    else button.dataset.path = versSite;
    // Un fichier introuvable ou refusé par le moteur : le passage reste fermé.
    if (inert) button.disabled = true;
    if (file && toRead) {
      // Pas encore lu : pas d'aperçu, mais le nom du serveur où l'on irait.
      computed = true;
      name = `${name} · ${new URL(file, location.href).host}`;
      if (!passageAllowed(file)) {
        color = "#555";
        name = `${name} (fermé)`;
        button.disabled = true;
      }
    } else if (file && text === null) {
      computed = true;
      color = "#555";
      name = `${name} (fermé)`;
      button.disabled = true;
    } else if (file && fromElsewhere(file)) {
      // On dit où mène un passage vers le site de quelqu'un d'autre.
      name = `${name} · ${new URL(file, location.href).host}`;
    }
    button.style.setProperty("--size", `${size}px`);
    button.style.setProperty("--color", color);
    const preview = document.createElement("span");
    preview.className = computed ? "preview computed" : "preview";
    const labelAlone = document.createElement("span");
    labelAlone.className = "name";
    labelAlone.textContent = name;
    if (computed) {
      // Un monde calculé n'a pas de page à montrer : une boule de lumière de sa couleur.
      button.append(preview, labelAlone);
      return button;
    }
    const content = document.createElement("span");
    content.className = "content";
    content.inert = true;
    content.style.width = `${innerWidth}px`;
    content.style.height = `${innerWidth}px`; // un carré : l'aperçu remplit le rond
    content.style.transform = `scale(${size / innerWidth})`;
    // L'aperçu est enfermé : les styles d'un autre fichier ne débordent ni sur la page ni sur
    // les autres aperçus.
    const inner = content.attachShadow({ mode: "open" });
    inner.innerHTML =
      "<style>.holo-Page{min-height:100%!important}</style>" + (file ? flat_view(text, folderOf(file), "") : flat_view(source, base, versSite));
    // L'aperçu montre les valeurs où elles en sont : celles de ce fichier, ou de l'autre s'il a déjà été visité.
    showValues(inner, states.get(file || path) ?? (file ? initial_state(text) : ""), file ? text : source);
    preview.append(content);
    const label = document.createElement("span");
    label.className = "name";
    label.textContent = name;
    button.append(preview, label);
    return button;
  }

  // Ouvre le carrefour. « vers » : le site contenu vers lequel on se dirige, mis en avant ;
  // sans lui, c'est la carte de ce qui entoure le site où l'on est.
  async function openCrossroads(vers = null) {
    exitPoints();
    const contents = containedSites();
    // Les autres fichiers sont petits et déjà demandés : on les attend pour montrer leur aperçu.
    // Ceux d'un autre serveur ne sont pas lus ici : un clic, un serveur, un fichier (B-07).
    for (const content of contents) {
      if (!content.file) continue;
      if (fromElsewhere(content.file)) content.toRead = true;
      else content.text = await read(content.file);
    }
    const here = { path: site, name: site ? `Ici : ${last(site)}` : "Ici", color: "#ffffff" };
    const target = contents.find((s) => s.name === vers);
    const portals = [];
    if (site) portals.push(portal({ path: parent(site), name: `Retour : ${last(parent(site)) || "accueil"}`, color: "#9aa4b2" }, portalSize, false));
    // Venu d'un autre fichier : un portail pour y retourner.
    else if (history.state?.since) portals.push(portal({ name: "Retour", color: "#9aa4b2", computed: true, back: true }, portalSize, false));
    // Portals(count:) borne tout le carrefour, y compris les sites écrits par l'auteur (B-02).
    const others = contents.filter((content) => content !== target);
    const place = Math.max(0, portalCount - 1 - portals.length - (target ? 1 : 0));
    for (const content of others.slice(0, place)) portals.push(portal(content, portalSize, false));
    if (others.length > place) {
      portals.push(portal({ name: `+ ${others.length - place} autres`, color: "#555", computed: true, inert: true }, portalSize, false));
    }
    if (target) portals.push(portal(here, portalSize, false));
    // Le reste de la place est rempli par des mondes calculés à partir d'une graine : on peut
    // y entrer, comme dans le Big Bang.
    const missingOnes = Math.max(0, portalCount - portals.length - 1);
    for (const neighbour of neighbour_worlds(source, site, missingOnes).split(";").filter(Boolean)) {
      const [seed, color] = neighbour.split(":");
      portals.push(portal({ path: `~${seed}`, name: `Monde ${seed.slice(-4)}`, color: `rgb(${color})`, computed: true }, portalSize, false));
    }
    // Le portail mis en avant est plus grand ; en grille il se place au milieu des autres.
    const big = Math.round(Math.min(portalSize * 1.9, Math.min(innerWidth, innerHeight) * 0.42));
    portals.splice(layout === "grid" ? Math.ceil(portals.length / 2) : 0, 0, portal(target ?? here, big, true));
    portals.forEach((element, rank) => element.style.setProperty("--rank", rank));
    crossroads.className = layout;
    crossroads.style.setProperty("--glow", `${Math.round(backgroundLight * 100)}%`);
    crossroads.replaceChildren(...portals);
    crossroads.hidden = false;
    modeButton.textContent = "Fermer";
    pointsButton.hidden = true;
    crossroads.querySelector(".to").focus({ preventScroll: true });
  }

  // Une adresse désigne le fichier d'un autre serveur. On ne le contacte pas sans un geste du
  // visiteur : on lui montre où il irait, et c'est lui qui décide (B-07).
  function proposePassage(file) {
    const big = Math.round(Math.min(portalSize * 1.9, Math.min(innerWidth, innerHeight) * 0.42));
    const vers = portal({ file, name: "Aller", color: "#E9B44C", toRead: true }, big, true);
    const stay = portal({ path: site, name: "Rester ici", color: "#ffffff" }, portalSize, false);
    crossroads.className = "grid";
    crossroads.style.setProperty("--glow", `${Math.round(backgroundLight * 100)}%`);
    crossroads.replaceChildren(vers, stay);
    crossroads.hidden = false;
    modeButton.textContent = "Fermer";
    pointsButton.hidden = true;
  }

  function closeCrossroads() {
    crossroads.hidden = true;
    crossroads.replaceChildren();
    modeButton.textContent = "Carrefour";
    pointsButton.hidden = false;
  }

  // Le portail s'ouvre : son aperçu grandit jusqu'à remplir la fenêtre, et devient le site.
  async function cross(button) {
    const versSite = button.dataset.path;
    if (button.dataset.back) {
      history.back();
      return;
    }
    if (versSite === site) {
      closeCrossroads();
      return;
    }
    const preview = button.querySelector(".preview");
    if (button.dataset.file && preview.classList.contains("computed")) {
      // Le fichier d'un autre serveur, lu seulement maintenant que le visiteur l'a choisi.
      const alreadyThere = decodeURIComponent(location.hash.slice(1)) === `@${fullAddress(button.dataset.file)}`;
      if (!(await openFile(button.dataset.file, "", { inHistory: !alreadyThere }))) {
        button.disabled = true;
        button.querySelector(".name").textContent += " (fermé)";
        button.style.setProperty("--color", "#555");
      }
      return;
    }
    if (preview.classList.contains("computed")) {
      displaySite(versSite);
      return;
    }
    const startValue = preview.getBoundingClientRect();
    Object.assign(preview.style, { left: `${startValue.left}px`, top: `${startValue.top}px`, width: `${startValue.width}px`, height: `${startValue.height}px` });
    preview.classList.add("opening");
    preview.getBoundingClientRect(); // le départ est pris en compte avant de lancer le mouvement
    Object.assign(preview.style, { left: "0px", top: "0px", width: "100vw", height: "100vh", borderRadius: "0" });
    Object.assign(preview.querySelector(".content").style, { transform: "scale(1)", height: `${innerHeight}px` });
    const still = calm.matches;
    await new Promise((suite) => setTimeout(suite, still ? 0 : portalDuration + 20));
    if (button.dataset.file) await openFile(button.dataset.file);
    else displaySite(versSite);
  }

  // ---------------------------------------------------------------- les règles du fichier

  // Le passage ordinaire d'un site à l'autre, comme sur le web : la page s'efface, la suivante
  // apparaît. Bref et discret. Avec les animations réduites, le changement est immédiat.
  async function fade(change) {
    if (calm.matches) return change();
    root.style.transition = "opacity .16s ease";
    root.style.opacity = "0";
    await new Promise((suite) => setTimeout(suite, 170));
    await change();
    root.getBoundingClientRect(); // la page neuve est d'abord prise en compte invisible
    root.style.opacity = "1";
    setTimeout(() => { root.style.transition = ""; root.style.opacity = ""; }, 200);
  }

  // Entrer dans un point : on y va directement. Le carrefour ne s'ouvre que si on le demande
  // (son bouton, ou la capacité « portals »). Deux façons d'y aller, deux mouvements :
  // - par un bouton de la page (« Enter the workshop ») : c'est un lien, on passe d'un site à
  //   l'autre comme sur le web, par un simple fondu ;
  // - en touchant le point lui-même : là on est dans le métavers, le point s'ouvre où il est,
  //   grandit jusqu'à remplir la fenêtre, et devient le site.
  let directEntry = false;
  async function enterInto(name, byPoint = false) {
    const content = containedSites().find((s) => s.name === name);
    if (!content || directEntry) return;
    // Le fichier d'un autre serveur : on ne le lit pas sans que le visiteur ait vu où il va.
    // Là, le carrefour garde son rôle : il affiche le nom du serveur, et attend un clic.
    if (content.file && fromElsewhere(content.file)) return openCrossroads(name);
    if (content.file) {
      content.text = await read(content.file);
      if (content.text === null) return openCrossroads(name); // introuvable ou refusé : le carrefour le montre fermé
    }
    exitPoints();
    directEntry = true;
    if (!byPoint) {
      try {
        await fade(() => (content.file ? openFile(content.file) : displaySite(content.path)));
      } finally {
        directEntry = false;
      }
      return;
    }
    const point = page.querySelector(`[data-name="${CSS.escape(name)}"]`).getBoundingClientRect();
    const size = Math.max(point.width, 24);
    const button = portal(content, size, true);
    button.querySelector(".name").remove();
    const passage = document.createElement("div");
    passage.id = "passage";
    Object.assign(button.style, { position: "fixed", left: `${point.left + point.width / 2 - size / 2}px`, top: `${point.top + point.height / 2 - size / 2}px` });
    passage.append(button);
    document.body.append(passage);
    try {
      await cross(button);
    } finally {
      passage.remove();
      directEntry = false;
    }
  }

  // Un formulaire qu'on envoie (ADR-042) : le moteur dit ce qu'il contient, la page l'envoie
  // au serveur d'où elle vient, puis émet Contact.sent, ou Contact.failed. Un seul envoi à la fois.
  const submissionsInProgress = new Set();
  // Les messages d'un formulaire (ADR-068) : sous chaque champ qui ne va pas, relié au champ
  // (aria-describedby), le champ marqué (aria-invalid). Le moteur écrit les messages.
  function formElement(formName) {
    return root.querySelector(`form[data-name="${CSS.escape(formName)}"]`);
  }
  function showFormErrors(formName, focus) {
    const form = formElement(formName);
    if (!form) return 0;
    const errors = form_errors(source, states.get(path) ?? "", formName).split("\n").filter(Boolean).map((line) => line.split("|"));
    for (const old of form.querySelectorAll(".holo-error")) old.remove();
    for (const field of form.querySelectorAll("[aria-invalid]")) {
      field.removeAttribute("aria-invalid");
      field.removeAttribute("aria-describedby");
    }
    let first = null;
    for (const [bind, message] of errors) {
      const group = form.querySelector(`[data-group="${CSS.escape(bind)}"]`);
      const field = group ?? form.querySelector(`[data-bind="${CSS.escape(bind)}"]`);
      if (!field) continue;
      const id = `holo-error-${formName}-${bind}`;
      const note = Object.assign(document.createElement("p"), { className: "holo-error", id, textContent: message });
      (group ?? field.closest("label") ?? field).after(note);
      field.setAttribute("aria-invalid", "true");
      field.setAttribute("aria-describedby", id);
      first ??= group ? group.querySelector("input") : field;
    }
    if (focus && first) {
      first.focus();
      announce(errors.length === 1 ? errors[0][1] : `${errors.length} champs à corriger.`);
    }
    return errors.length;
  }

  async function submit(formName) {
    if (submissionsInProgress.has(formName)) return;
    // Vérifier avant d'envoyer (ADR-068) ; ensuite, les messages suivent ce qu'on corrige.
    const form = formElement(formName);
    if (form) form.dataset.tried = "1";
    if (showFormErrors(formName, true)) return;
    const body = submission(source, states.get(path) ?? "", formName);
    if (!body) return;
    submissionsInProgress.add(formName);
    form?.setAttribute("aria-busy", "true");
    const for_ = path;
    let arrived = false;
    // Avec des fichiers (ADR-059), l'envoi part en plusieurs morceaux : les valeurs, puis chaque fichier.
    const files = [...(root.querySelector(`form[data-name="${CSS.escape(formName)}"]`)?.querySelectorAll("input[type=file]") ?? [])].filter((field) => field.files?.[0]);
    let request = { method: "POST", headers: { "content-type": "application/json" }, body: body };
    if (files.length) {
      const chunks = new FormData();
      chunks.append("submission", body);
      for (const field of files) chunks.append(field.dataset.bind, field.files[0], field.files[0].name);
      request = { method: "POST", body: chunks };
    }
    // 15 secondes au plus (ADR-068) : un serveur qui ne répond pas est un échec.
    const stop = new AbortController();
    const late = setTimeout(() => stop.abort(), 15000);
    try {
      const response = await fetch(path, { ...request, signal: stop.signal });
      arrived = response.ok;
    } catch { /* pas de réseau, pas de serveur pour recevoir, ou trop lent */ }
    clearTimeout(late);
    submissionsInProgress.delete(formName);
    form?.removeAttribute("aria-busy");
    if (for_ === path) emit(`${formName}.${arrived ? "sent" : "failed"}`);
  }

  // Un fichier choisi (ADR-059) : la page vérifie sa sorte et sa taille tout de suite, et le dit
  // avec le message du navigateur ; un fichier refusé est retiré. Le serveur vérifie à nouveau.
  function allowedFile(field) {
    const file = field.files?.[0];
    if (!file) return "";
    const inFrench = (document.documentElement.lang || "fr").startsWith("fr");
    const max = Number(field.dataset.max);
    const kinds = field.accept.split(",");
    let refusal = "";
    if (!kinds.includes(file.type)) refusal = inFrench ? "Ce fichier n'est pas d'une sorte acceptée ici." : "This kind of file is not accepted here.";
    else if (file.size > max) {
      const size = max >= 1e6 ? `${max / 1e6} ${inFrench ? "Mo" : "MB"}` : `${max / 1e3} ${inFrench ? "Ko" : "KB"}`;
      refusal = inFrench ? `Ce fichier est trop lourd : ${size} au plus.` : `This file is too large: ${size} at most.`;
    }
    field.setCustomValidity(refusal);
    if (!refusal) return file.name;
    field.value = "";
    field.reportValidity();
    return "";
  }

  // Un module enfermé (ADR-011 partie C, ADR-045). Il tourne dans un fil à part : la page ne se
  // bloque jamais. Il ne reçoit que sa mémoire, plafonnée (elle ne peut pas grandir au-delà), et
  // un nombre ; il n'a ni réseau, ni page, ni heure : un module qui demande autre chose ne
  // démarre pas. Son temps court à partir du moment où il commence ; au-delà, le fil est arrêté.
  // Les noms des messages sont les mêmes des deux côtés (la traduction en anglais avait oublié
  // ce texte : la boîte attendait encore « octets », « entree », et aucun module ne marchait plus).
  // Deux contrats (ADR-077). Le premier : run(nombre) rend un nombre. Le second : le module offre
  // alloc(taille), qui dit où écrire ce qu'il reçoit (un texte JSON), et run(adresse, taille), qui
  // rend l'adresse et la taille de sa réponse, en un seul nombre de 64 bits. La réponse est lue
  // dans sa mémoire, 64 Ko au plus, puis relue par le moteur avec méfiance.
  const SANDBOX_CODE = `onmessage = async ({ data: { bytes, entry, json, simple, pages } }) => {
    try {
      const memory = new WebAssembly.Memory({ initial: pages, maximum: pages });
      const { instance } = await WebAssembly.instantiate(bytes, { env: { memory } });
      const { run, alloc } = instance.exports;
      if (typeof run !== "function") throw new Error("le module n'offre pas run");
      const second = typeof alloc === "function" && run.length === 2;
      if (!second && !simple) throw new Error("ce module ne sait recevoir et rendre qu'un nombre : il ne lit pas des textes ni des listes");
      postMessage({ start: true });
      if (!second) return postMessage({ ok: true, output: run(entry >>> 0) >>> 0 });
      const input = new TextEncoder().encode(json);
      const at = alloc(input.length) >>> 0;
      if (at === 0 || at + input.length > memory.buffer.byteLength) throw new Error("le module n'a pas la place de lire ce qu'il reçoit");
      new Uint8Array(memory.buffer, at, input.length).set(input);
      const answer = run(at, input.length);
      if (typeof answer !== "bigint") throw new Error("run doit rendre l'adresse et la taille de sa réponse, en un nombre de 64 bits");
      const where = Number(BigInt.asUintN(64, answer) >> 32n), size = Number(BigInt.asUintN(64, answer) & 0xffffffffn);
      if (size > 65536 || where + size > memory.buffer.byteLength) throw new Error("une réponse de plus de 64 Ko, ou hors de sa mémoire");
      postMessage({ ok: true, json: new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(memory.buffer, where, size)) });
    } catch (e) {
      postMessage({ ok: false, reason: String((e && e.message) || e) });
    }
  };`;
  const runningModules = new Set();
  async function execute(name) {
    if (runningModules.has(name)) return;
    const [file, entry, time, pages, simple] = module_info(source, states.get(path) ?? "", name).split("|");
    if (!file) return;
    runningModules.add(name);
    const for_ = path;
    let result = { ok: false, reason: "module introuvable" };
    try {
      const response = await fetch(folderOf(path) + file);
      // Un module de 4 Mo au plus : au-delà, il n'est pas téléchargé plus loin.
      const bytes = response.ok ? await readCapped(response, 4e6) : null;
      if (bytes) {
        const box = new Worker(URL.createObjectURL(new Blob([SANDBOX_CODE], { type: "text/javascript" })));
        result = await new Promise((end) => {
          let stop = 0;
          const finish = (r) => { clearTimeout(stop); clearTimeout(startup); box.terminate(); end(r); };
          // Le démarrage lui-même a une limite : un module trop lourd à préparer est arrêté aussi.
          const startup = setTimeout(() => finish({ ok: false, reason: "trop long à démarrer" }), 5000);
          box.onmessage = ({ data }) => {
            if (data.start) {
              clearTimeout(startup);
              stop = setTimeout(() => finish({ ok: false, reason: `arrêté après ${time} ms` }), Number(time));
            } else finish(data);
          };
          box.onerror = () => finish({ ok: false, reason: "erreur du module" });
          const json = module_input(source, states.get(path) ?? "", name);
          box.postMessage({ bytes, entry: Number(entry), json, simple: simple === "1", pages: Number(pages) }, [bytes]);
        });
      }
    } catch { /* pas de réseau */ }
    runningModules.delete(name);
    // La réponse du second contrat, relue par le moteur : refusée, le module a échoué.
    let after = "";
    if (result.ok && result.json !== undefined) {
      try {
        after = for_ === path ? store(module_received(source, states.get(path) ?? "", name, result.json)) : "";
      } catch (refusal) {
        result = { ok: false, reason: String(refusal), answer: result.json.slice(0, 200) };
      }
    }
    (window.__holoModules ??= []).push({ name, ...result }); // ce qui s'est passé, pour le vérifier
    if (for_ !== path) return;
    if (!result.ok) return emit(`${name}.failed`);
    if (result.json === undefined) after = store(module_finished(source, states.get(path) ?? "", name, result.output));
    if (after) changeState(after);
    for (const effect of effects(source, `${name}.done`).split(",").filter(Boolean)) apply(effect, `${name}.done`);
  }

  function apply(effect, signal = "") {
    const [name, capability] = effect.split(".");
    if (capability === "enter" && containedSites().some((s) => s.name === name)) {
      // Le signal vient-il du point lui-même, ou d'un bouton qui y mène ?
      enterInto(name, signal === `${name}.tap`);
    } else if (capability === "play" || capability === "stop") {
      // Faire entendre un son, ou l'arrêter (ADR-061). Un navigateur ne joue un son qu'après un
      // premier geste du visiteur : avant, il refuse, et la page continue sans lui.
      const sound = root.querySelector(`audio[data-name="${CSS.escape(name)}"]`);
      if (sound) {
        if (sound.dataset.volume) sound.volume = Number(sound.dataset.volume);
        sound.pause();
        sound.currentTime = 0;
        if (capability === "play") sound.play().catch(() => {});
      }
    } else if (capability === "open" || capability === "close") {
      // Une fenêtre par-dessus la page (ADR-042) : Confirm.open, Confirm.close.
      const window = root.querySelector(`dialog[data-name="${CSS.escape(name)}"]`);
      if (window && capability === "open" && !window.open) window.showModal();
      if (window && capability === "close") window.close();
    } else if (capability === "send") {
      submit(name);
    } else if (capability === "run") {
      execute(name);
    } else if (capability === "refresh") {
      refreshData(name);
    } else if (capability === "portals") {
      // La page demande son carrefour : On(Map.tap, effect: Shop.portals).
      openCrossroads();
    } else if (capability === "leave") {
      // Sortir d'un site : on remonte à celui qui le contient.
      if (!crossroads.hidden) closeCrossroads();
      else if (last(site) === name) fade(() => displaySite(parent(site)));
    }
  }

  try {
    try {
      await init();
      giveTime();
    } catch (e) {
      // Le moteur n'a pas pu arriver (réseau, fichier absent) : la page le dit, reste lisible,
      // et propose de réessayer. Aucun toucher n'est compté comme fait.
      window.__holoFailure?.(true);
      throw new Error("le moteur n'a pas pu démarrer");
    }
    source = await withImports(await (await fetch(path, { headers: { accept: "text/plain" } })).text(), path);
    readFiles.set(path, Promise.resolve(source));
    readSettings();
    // La page fabriquée par le serveur avec ses données (ADR-064) : le navigateur rejoue la même
    // réception, puis `Shop.done`, pour partir du même état. Sans cela, les valeurs de départ
    // remplaceraient un instant celles des données. Les effets (un son…) ne sont pas rejoués.
    // La page fabriquée par holo serve pour ce visiteur (ADR-074) : ses valeurs, gardées sur le
    // serveur après des touchers faits sans JavaScript. Le moteur repart de là.
    const visit = !location.hash.slice(1) && root.querySelector(".holo-Page")?.dataset.visit;
    if (visit) states.set(path, visit);
    const received = !visit && !location.hash.slice(1) && root.querySelector(".holo-Page")?.dataset.received;
    if (received) {
      const dataName = data(source).split("|")[2];
      let replayed = receive(source, states.get(path) ?? "", received);
      if (replayed && dataName) replayed = arbitrate(source, replayed, `${dataName}.done`);
      if (replayed) states.set(path, replayed.split(";").filter((chunk) => !chunk.startsWith("!=")).join(";"));
    }
    // Une page qui montrera des points ou des mondes fait venir le dessin tout de suite, sans
    // l'attendre : il sera prêt quand le visiteur zoomera.
    if (needs_drawing(source)) loadDrawing().catch(() => {});
    // Un endroit de la page (#Hours) n'est pas un site : on reste sur la page, à cet endroit.
    const pageSpot = (name) => name && !name.startsWith("@") && !name.startsWith("~") && !name.includes("/") && document.getElementById(name)?.closest("#page") && !containedSites().some((s) => s.name === name);
    const siteStart = pageSpot(decodeURIComponent(location.hash.slice(1))) ? "" : decodeURIComponent(location.hash.slice(1));
    // La page du fichier est déjà là, fabriquée par le serveur : on la reprend. Un monde
    // demandé par l'adresse (#Atelier), lui, se dessine.
    const alreadyThere = !siteStart && root.querySelector(".holo-Page") !== null;
    displaySite(siteStart.startsWith("@") ? "" : siteStart, { inHistory: false, resume: alreadyThere });
    window.__holoStarted = true; // le moteur a pris la page en main (pour les essais)
    // Une adresse en #@… désigne le fichier d'un autre serveur : on propose le passage.
    if (siteStart.startsWith("@")) proposePassage(siteStart.slice(1));
    // « Revenir » : par où l'on est venu, ou, si l'on est arrivé directement, au fichier de départ.
    document.querySelector("#origin button").addEventListener("click", () => (history.state?.since ? history.back() : openFile(location.pathname)));
    // Le bouton « retour » du navigateur, ou une adresse changée à la main.
    addEventListener("popstate", () => {
      const sitePath = decodeURIComponent(location.hash.slice(1));
      // L'adresse désigne un autre fichier : on y passe, toujours sans recharger.
      // Un fichier d'ailleurs déjà lu pendant cette visite : le visiteur l'avait choisi. Sinon, on propose.
      if (sitePath.startsWith("@") && readFiles.has(sitePath.slice(1))) openFile(sitePath.slice(1), "", { inHistory: false });
      else if (sitePath.startsWith("@")) proposePassage(sitePath.slice(1));
      else if (location.pathname !== path && location.pathname.endsWith(".holo")) openFile(location.pathname, sitePath, { inHistory: false });
      // Un lien vers un endroit de la page (ADR-042) : le navigateur y descend, rien d'autre.
      else if (pageSpot(sitePath)) return;
      else displaySite(sitePath, { inHistory: false });
    });
    // Dans un monde calculé : dézoomer alors qu'on est revenu tout en haut en fait ressortir.
    addEventListener("wheel", (event) => {
      const measures = window.__holo;
      if (inWorld && event.deltaY > 0 && measures && measures.depth === 0 && measures.zoom === 0 && performance.now() - lastPassage > 1500) {
        lastPassage = performance.now();
        history.back();
      }
    }, { passive: true });
    addEventListener("resize", () => { placePixels(); wakeTracking(); });

    // En vue points, un toucher sans glissement active le point planté qui se trouve dessous.
    let press = null;
    addEventListener("pointerdown", (event) => { press = [event.clientX, event.clientY]; }, true);
    addEventListener("pointerup", (event) => {
      const slide = press ? Math.hypot(event.clientX - press[0], event.clientY - press[1]) : Infinity;
      press = null;
      if (inPoints && slide < 6 && !event.target.closest("button")) touchPoint(event.clientX, event.clientY);
    }, true);
    // Page grossie : glisser la déplace, dans tous les sens, comme en vue points. Le geste est
    // donc le même du début à la fin du zoom. (Un double clic sélectionne toujours un mot.)
    let grab = null;
    root.addEventListener("mousedown", (event) => {
      if (sharpZoom === 1 || inPoints || event.button !== 0 || event.detail > 1) return;
      event.preventDefault(); // pas de sélection de texte en glissant
      grab = { x: event.clientX, y: event.clientY, dragging: false };
      root.style.cursor = "grabbing";
    });
    addEventListener("mousemove", (event) => {
      if (!grab) return;
      if (Math.hypot(event.clientX - grab.x, event.clientY - grab.y) > 4) grab.dragging = true;
      scrollBy(-event.movementX, -event.movementY);
    });
    addEventListener("mouseup", () => {
      if (!grab) return;
      // Après un glissement, le relâchement ne compte pas comme un clic sur un bouton.
      if (grab.dragging) addEventListener("click", (click) => click.stopPropagation(), { capture: true, once: true });
      grab = null;
      root.style.cursor = sharpZoom === 1 ? "" : "grab";
    });
    // Le survol (ADR-039) : la souris arrive sur un bloc qu'une règle écoute, le clavier s'y pose,
    // ou le doigt le touche sur un téléphone, où la souris n'existe pas. « hover » part à
    // l'arrivée, « hoverEnd » au départ ; un bloc dans un autre bloc survolé l'est aussi.
    const hoverOver = (name, inside) => {
      if (inside === hoveredOnes.has(name)) return;
      if (inside) hoveredOnes.add(name);
      else hoveredOnes.delete(name);
      emit(`${name}.${inside ? "hover" : "hoverEnd"}`);
    };
    const hoversOf = (target) => {
      const blocks = [];
      for (let block = target?.closest?.("[data-hover]"); block; block = block.parentElement?.closest("[data-hover]")) blocks.push(block);
      return blocks;
    };
    const passage = (inside) => (event) => {
      if (event.pointerType === "touch") return;
      for (const block of hoversOf(event.target)) {
        if (!block.contains(event.relatedTarget)) hoverOver(block.dataset.name, inside);
      }
    };
    root.addEventListener("pointerover", passage(true));
    root.addEventListener("pointerout", passage(false));
    root.addEventListener("focusin", passage(true));
    root.addEventListener("focusout", passage(false));
    let fingerTouch = false;
    root.addEventListener("pointerdown", (event) => { fingerTouch = event.pointerType === "touch"; }, true);
    // Sur la page : un toucher est envoyé au moteur, qui répond par les effets demandés.
    root.addEventListener("click", (event) => {
      if (fingerTouch) {
        // Au doigt : toucher un bloc le survole, toucher ailleurs le quitte.
        const touched = hoversOf(event.target).map((block) => block.dataset.name);
        for (const name of [...hoveredOnes]) if (!touched.includes(name)) hoverOver(name, false);
        for (const name of touched) hoverOver(name, true);
      }
      const block = event.target.closest("[data-name]");
      if (!block) return;
      // Un bouton dans la ligne d'une liste dit de quelle ligne il vient : Done.tap@2 (ADR-044).
      const line = block.closest("[data-rank]")?.dataset.rank;
      emit(line === undefined ? `${block.dataset.name}.tap` : `${block.dataset.name}.tap@${line}`);
    });
    // Faire glisser un bloc d'un plateau (drag: true), au doigt ou à la souris. La page dit à
    // l'arbitre où est le doigt, de 0 à 100 ; c'est lui qui change les valeurs.
    let slide = null;
    root.addEventListener("pointerdown", (event) => {
      const placed = event.target.closest(".holo-positioned[data-drag]");
      if (!placed) return;
      slide = { placed, board: placed.parentElement };
      placed.classList.add("holo-dragging");
      placed.setPointerCapture(event.pointerId);
      event.preventDefault();
    });
    root.addEventListener("pointermove", (event) => {
      if (!slide) return;
      const { placed, board } = slide;
      const [frame, size] = [board.getBoundingClientRect(), placed.getBoundingClientRect()];
      // Le centre du bloc suit le doigt ; 0 et 100 sont les deux bords où le bloc touche le plateau.
      const vers = (finger, start, length, width) => Math.round(Math.min(100, Math.max(0, ((finger - start - width / 2) / Math.max(1, length - width)) * 100)));
      const [x, y] = [vers(event.clientX, frame.left, frame.width, size.width), vers(event.clientY, frame.top, frame.height, size.height)];
      const before = states.get(path) ?? "";
      const after = store(drag(source, before, placed.dataset.drag, x, y));
      if (after !== before) changeState(after);
    });
    for (const end of ["pointerup", "pointercancel"]) {
      root.addEventListener(end, () => {
        slide?.placed.classList.remove("holo-dragging");
        slide = null;
      });
    }
    // Le clavier : une touche que le fichier écoute devient un signal, comme un toucher. Les
    // autres touches gardent leur rôle (défiler, écrire), et rien n'est pris à un champ où l'on écrit.
    // Les lettres : celles écrites sur la touche ; les chiffres : la rangée du haut ou le pavé
    // numérique, avec ou sans Maj (ADR-061). Sur un bouton, l'espace et Entrée le touchent.
    const keyName = (event) => {
      const named = { ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down", " ": "space", Enter: "enter", Escape: "escape" };
      if (named[event.key]) return named[event.key];
      const digit = /^(?:Digit|Numpad)(\d)$/.exec(event.code);
      if (digit) return `digit${digit[1]}`;
      return /^[a-z]$/i.test(event.key) ? event.key.toLowerCase() : null;
    };
    addEventListener("keydown", (event) => {
      const keypress = keyName(event);
      if (!keypress || !listenedKeys.includes(keypress) || event.ctrlKey || event.altKey || event.metaKey) return;
      if (isSingleCharacter(keypress) && !lettersAllowed) return;
      const activates = keypress === "space" || keypress === "enter";
      if (event.target.closest?.(activates ? "input, textarea, select, button" : "input, textarea, select") || inPoints || inWorld || !crossroads.hidden) return;
      // Une fenêtre ouverte garde le clavier pour elle : Échap la ferme, Tab reste dedans.
      if (document.querySelector("dialog[open]")) return;
      event.preventDefault();
      emit(`Key.${keypress}`);
    });
    // Un formulaire ne recharge jamais la page : c'est une règle qui l'envoie (ADR-042).
    root.addEventListener("submit", (event) => {
      if (event.target.closest(".holo-Form")) event.preventDefault();
    });
    // Écrire dans un champ, cocher une case : c'est l'arbitre du moteur qui change la valeur.
    root.addEventListener("input", (event) => {
      const field = event.target.closest("[data-bind]");
      if (!field) return;
      const written = field.type === "checkbox" ? (field.checked ? "1" : "0") : field.type === "file" ? allowedFile(field) : field.value;
      changeState(store(input(source, states.get(path) ?? "", field.dataset.bind, written)));
      // Après un premier essai d'envoi, les messages suivent ce qu'on corrige (ADR-068).
      const form = field.closest(".holo-Form");
      if (form?.dataset.tried) showFormErrors(form.dataset.name, false);
    });
    // Entrée dans un champ d'une ligne envoie le formulaire, comme sur le web : c'est le premier
    // bouton du formulaire qui est touché (ADR-068).
    root.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" || event.isComposing || event.shiftKey || event.ctrlKey || event.altKey || event.metaKey) return;
      const field = event.target;
      if (!field.matches?.(".holo-Form input:not([type=checkbox]):not([type=radio]):not([type=file]), .holo-Form select")) return;
      const button = field.closest(".holo-Form").querySelector("button[data-name]");
      if (!button) return;
      event.preventDefault();
      button.click();
    });
    // En quittant un champ, il montre la valeur que l'arbitre a retenue (bornée).
    root.addEventListener("change", () => showValues());
    // Le moteur est prêt : on rejoue ce qui a été touché en l'attendant.
    window.__holoStopRecording?.();
    for (const expected of pendingTouches.splice(0)) {
      const [name, gesture = "tap"] = expected.split(".");
      if (gesture === "tap") {
        const [single, line] = name.split("@");
        emit(line === undefined ? `${single}.tap` : `${single}.tap@${line}`);
        continue;
      }
      // Un survol à la souris ou au clavier est rejoué s'il dure encore ; au doigt, le toucher survole.
      const block = root.querySelector(`[data-name="${CSS.escape(name)}"]`);
      if (gesture === "touch" || (gesture === "hover" && block?.matches(":hover")) || (gesture === "focus" && block?.contains(document.activeElement))) hoverOver(name, true);
    }
    crossroads.addEventListener("click", (event) => {
      const button = event.target.closest(".portal");
      if (button) cross(button);
      else closeCrossroads(); // un clic à côté des portails : on reste où l'on est
    });
    addEventListener("keydown", (event) => { if (event.key === "Escape" && !crossroads.hidden) closeCrossroads(); });

    document.getElementById("menu").hidden = params.has("bare"); // ?bare : sans les boutons, pour en tirer une image
    // Le bouton unique : il ouvre les outils, puis devient la croix qui les referme.
    const toggle = document.getElementById("toggle");
    const tools = document.getElementById("tools");
    const openTools = (opened) => {
      tools.hidden = !opened;
      toggle.setAttribute("aria-expanded", String(opened));
      toggle.setAttribute("aria-label", opened ? "Fermer les outils" : "Ouvrir les outils");
      toggle.textContent = opened ? "✕" : "☰";
    };
    toggle.addEventListener("click", () => openTools(tools.hidden));
    // Le bouton a été touché avant que le moteur arrive : on ouvre les outils maintenant.
    if (window.__holoMenuRequested) openTools(true);
    addEventListener("keydown", (event) => { if (event.key === "Escape") openTools(false); });
    // Changer de vue ou ouvrir le carrefour referme le menu : l'écran revient au site.
    // « Tourner » le laisse ouvert, pour garder « De face » sous la main.
    for (const button of [pointsButton, modeButton]) button.addEventListener("click", () => openTools(false));
    modeButton.addEventListener("click", () => {
      if (inWorld) history.back(); // on quitte le monde calculé pour revenir au site
      else if (crossroads.hidden) openCrossroads();
      else closeCrossroads();
    });
    pointsButton.addEventListener("click", () => (inPoints ? exitPoints() : enterPoints()));
    turnButton.addEventListener("click", async () => {
      const active = turnButton.getAttribute("aria-pressed") !== "true";
      // De face, sur le site ordinaire : la page devient d'abord ses points, sans que rien ne
      // change à l'écran, puis elle tourne sous le doigt.
      if (!inPoints) await enterPoints();
      if (!inPoints) return;
      turnButton.setAttribute("aria-pressed", String(active));
      mosaic_turn(active);
    });
    document.getElementById("front").addEventListener("click", () => { mosaic_front(); wakeTracking(); });
    // Décrocher la page (Zoom(detach: true), ADR-069) : elle se détache de l'écran, et le zoom
    // l'approche comme une feuille. Accrocher la remet à sa place, à sa taille.
    detachButton.addEventListener("click", () => {
      detached = !detached;
      if (!detached) grow(1);
      document.body.classList.toggle("detached", detached);
      detachButton.textContent = detached ? "Accrocher" : "Décrocher";
      zoomMode();
      announce(detached ? "Page décrochée : le zoom l'approche comme une feuille." : "Page accrochée : elle reste à sa place.");
    });
    // En vue points, tout geste relance le suivi ; une souris qui passe sans bouton ne compte pas.
    for (const name of ["wheel", "pointerdown", "pointermove", "pointerup"]) {
      addEventListener(name, (event) => {
        if (name !== "pointermove" || event.buttons !== 0) wakeTracking();
      }, { capture: true, passive: true });
    }
    // En vue points, le bouton droit sert à tourner la page : pas de menu.
    addEventListener("contextmenu", (event) => { if (inPoints) event.preventDefault(); });

    // Pour les captures d'écran : ?view=crossroads, ?enter=Workshop,
    // ?view=points avec &zoom=…&x=…&y=… (comme la molette à cet endroit) et &yaw=…&pitch=… (en degrés).
    if (params.get("view") === "crossroads") openCrossroads();
    if (params.get("enter")) apply(`${params.get("enter")}.enter`);
    if (params.get("view") === "points") {
      await enterPoints();
      if (params.get("zoom")) {
        area().dispatchEvent(new WheelEvent("wheel", {
          deltaY: -Math.log2(Number(params.get("zoom"))) / 0.003,
          clientX: Number(params.get("x") ?? innerWidth / 2), clientY: Number(params.get("y") ?? innerHeight / 2), cancelable: true,
        }));
      }
      mosaic_pivot(Number(params.get("yaw") ?? 0) * Math.PI / 180, Number(params.get("pitch") ?? 0) * Math.PI / 180);
      wakeTracking();
    }
  } catch (e) {
    const pre = document.body.appendChild(document.createElement("pre"));
    pre.id = "error";
    pre.textContent = (e?.message === "le moteur n'a pas pu démarrer" ? "" : "Le moteur a refusé ce fichier : " + (e?.message ?? e));
    if (!pre.textContent) pre.remove();
  }
