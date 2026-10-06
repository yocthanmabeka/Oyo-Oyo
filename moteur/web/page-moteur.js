// Le moteur de la page d'entrée : il arrive après la page (voir page.html), quand un geste en
// a besoin ou tout de suite si la page est vivante (ADR-033). Il charge le moteur en Rust,
// compilé en WebAssembly, et prend la page en main.
  import init, {
    pause, vue_a_plat, effets, etat_initial, arbitrer, conditions, horloges, touchees, touches, imports, donnees, recevoir, saisir, glisser, a_garder, reprendre, demarrer, changer_de_monde, mondes_voisins, demarrer_mosaique, poser_mosaique, retirer_mosaique, mosaique_camera, mosaique_tourner,
    mosaique_pivoter, mosaique_de_face, mosaique_sous, reglages_de_vue, reveiller, images_dessinees,
  } from "/pkg/holo_moteur.js";
  window.__holoPause = pause;
  window.__holoImages = images_dessinees; // combien d'images le moteur a dessinées : pour vérifier la sobriété
  const params = new URLSearchParams(location.search);
  window.__holoSansWebGPU = params.has("webgl"); // ?webgl : mesurer le mode de secours, WebGL 2
  // L'adresse est celle du fichier .holo lui-même ; sinon ?monde=… ou la boutique d'exemple.
  // Ce fichier peut changer en cours de route : on passe d'un fichier à l'autre par un point,
  // sans recharger la page.
  let chemin = location.pathname.endsWith(".holo") ? location.pathname : params.get("monde") ?? "/exemples/boutique-comparee/boutique.holo";
  let base = chemin.slice(0, chemin.lastIndexOf("/") + 1);
  const dossierDe = (fichier) => fichier.slice(0, fichier.lastIndexOf("/") + 1);
  // Les fichiers déjà lus, par adresse : leur texte, ou null s'ils sont introuvables ou refusés.
  const fichiersLus = new Map();
  // Garde-fou : on ne garde en mémoire que les derniers fichiers lus. On peut passer d'un
  // fichier à l'autre sans fin ; la mémoire, elle, ne grandit pas sans fin.
  const FICHIERS_GARDES = 32;
  // Un fichier .holo est un texte court. On arrête de lire au-delà de cette taille, et on
  // n'attend pas un serveur qui ne répond pas (revue Codex du 2026-10-03, B-04). La mémoire
  // des fichiers lus est donc bornée : 32 fichiers de 256 Ko au plus.
  const OCTETS_MAX = 262144;
  const DELAI_MAX = 8000;
  // La vue points redessine la page dans une image : jamais plus de huit millions de points,
  // quelle que soit la densité demandée, pour tenir dans la mémoire d'un téléphone (B-05).
  const POINTS_MAX = 8e6;
  let densiteDemandee = 2;
  let vitesseDuZoom = 1;     // Zoom(speed:)
  let dureeDuPortail = 450;  // Portals(duration:), en millisecondes
  let angleDeRotation = 0;   // Relief(tilt:), en degrés ; 0 : la page ne tourne pas
  let pointsActifs = false;  // l'auteur a-t-il demandé les points (points: ou pixels:) ? Sinon, un site ordinaire
  // Les valeurs de chaque fichier ouvert (State) : « cart=2 ». Elles suivent le visiteur tant
  // qu'il ne recharge pas la page : il peut entrer dans un monde, passer ailleurs, revenir.
  const etats = new Map();
  const chezSoi = (hote) => hote === "localhost" || hote === "127.0.0.1";
  // En http, on ne va que vers sa propre machine, et seulement si l'on y est déjà : une page
  // publique ne fait pas partir de requêtes vers le réseau privé du visiteur.
  function passagePermis(fichier) {
    const adresse = new URL(fichier, location.href);
    return adresse.protocol === "https:" || adresse.origin === location.origin || (chezSoi(adresse.hostname) && chezSoi(location.hostname));
  }
  const adresseEntiere = (fichier) => new URL(fichier, location.href).href;
  const dAilleurs = (fichier) => new URL(fichier, location.href).origin !== location.origin;
  let dernierePassage = 0;  // pour qu'un seul coup de molette ne fasse pas remonter deux mondes
  // Le choix du visiteur, réglé dans son téléphone ou son ordinateur (« réduire les
  // animations »). Sans ce choix, tout est au niveau normal. Avec lui : pas de transition,
  // et la page ne devient pas des points toute seule quand on zoome ; le bouton « Vue points »
  // reste là pour qui veut y aller.
  const calme = matchMedia("(prefers-reduced-motion: reduce)");
  const racine = document.getElementById("page");
  // La page arrive déjà fabriquée par le serveur : on peut toucher un bouton avant que le
  // moteur soit prêt. La page légère (page.html) note ces touchers ; ils sont rejoués dès que
  // le moteur l'est : aucun n'est perdu.
  const touchersEnAttente = window.__holoAttente ?? [];
  const carrefour = document.getElementById("carrefour");
  const boutonMode = document.getElementById("mode");
  const boutonPoints = document.getElementById("points");
  const boutonTourner = document.getElementById("tourner");
  const image = document.getElementById("image");
  const etat = document.getElementById("etat");
  const nombre = new Intl.NumberFormat("fr-FR");
  const zone = () => document.querySelector("canvas"); // le moteur peut l'avoir remplacée
  // Ce que le fichier dit de sa vue (Zoom, Points). Lu au démarrage.
  let densite = 2;          // points par pixel d'écran, dans chaque sens
  let reduire = false;      // dézoomer réduit-il la page jusqu'à un point ?
  let zoomAvantPoints = 4;  // jusqu'à ce grossissement, la page reste un site ordinaire
  let zoomActif = true;     // le visiteur peut-il zoomer ? (Zoom(active:))
  // Le carrefour (Portals) : disposition, nombre de mondes, taille d'un portail, lumière du fond.
  let disposition = "grille";
  let nombreDePortails = 12;
  let tailleDePortail = 170;
  let lumiereDuFond = 0.15;
  let enMonde = false;      // on est dans un monde calculé, ouvert en profondeur
  let source = "";
  let site = "";            // le site affiché : "" pour la page du fichier, sinon le chemin des points traversés
  let cadre = null;         // l'élément .holo-Page
  let page = null;          // son contenu, <main>
  let moteurLance = false;
  let zoomVif = 1;          // le grossissement de la page vivante
  let enPoints = false;     // la page est vue comme un ensemble de points
  let entreeEnCours = null; // l'entrée en vue points, tant qu'elle se prépare
  let aQuitteLeRepos = false; // a-t-on zoomé ou tourné depuis l'entrée en vue points ?
  let pixelsPlantes = [];   // les points plantés dans un pixel de la page, et où ils sont

  const parent = (cheminDeSite) => cheminDeSite.split("/").filter(Boolean).slice(0, -1).join("/");
  const dernier = (cheminDeSite) => cheminDeSite.split("/").filter(Boolean).at(-1) ?? "";

  // ---------------------------------------------------------------- les fichiers

  // Lit un fichier .holo et le fait vérifier par le moteur. Un fichier est petit : on peut
  // lire d'avance ceux où mènent les points de la page, pour que le passage soit immédiat.
  // Un fichier peut en importer d'autres, rangés à côté de lui (import "commun.holo"). La page
  // va les chercher et les joint à son texte : le moteur, lui, ne lit jamais rien tout seul.
  async function avecSesImports(texte, fichier) {
    let tout = texte;
    const discret = dAilleurs(fichier) ? { credentials: "omit", referrerPolicy: "no-referrer" } : {};
    for (const nom of imports(texte).split(";").filter(Boolean)) {
      try {
        const reponse = await fetch(dossierDe(fichier) + nom, { headers: { accept: "text/plain" }, ...discret });
        if (reponse.ok) tout += `\u001e${nom}\u001f${(await reponse.text()).slice(0, OCTETS_MAX)}`;
      } catch { /* introuvable : le moteur le dira, avec la ligne de l'import */ }
    }
    return tout;
  }

  function lire(fichier) {
    if (fichiersLus.has(fichier)) {
      // Relu à l'instant : il repasse en tête des fichiers gardés.
      const deja = fichiersLus.get(fichier);
      fichiersLus.delete(fichier);
      fichiersLus.set(fichier, deja);
    }
    if (!fichiersLus.has(fichier)) {
      for (const ancien of fichiersLus.keys()) {
        if (fichiersLus.size < FICHIERS_GARDES) break;
        if (ancien !== chemin) fichiersLus.delete(ancien);
      }
      fichiersLus.set(fichier, (async () => {
        const arret = new AbortController();
        const delai = setTimeout(() => arret.abort(), DELAI_MAX);
        try {
          if (!passagePermis(fichier)) return null;
          // Chez quelqu'un d'autre : sans cookies, et sans dire d'où l'on vient.
          const discret = dAilleurs(fichier) ? { credentials: "omit", referrerPolicy: "no-referrer" } : {};
          const reponse = await fetch(fichier, { headers: { accept: "text/plain" }, signal: arret.signal, ...discret });
          if (!reponse.ok || Number(reponse.headers.get("content-length") ?? 0) > OCTETS_MAX) return null;
          // Lecture par morceaux : on s'arrête dès que la limite est dépassée.
          const lecteur = reponse.body.getReader();
          const morceaux = [];
          let recu = 0;
          for (;;) {
            const { done, value } = await lecteur.read();
            if (done) break;
            recu += value.length;
            if (recu > OCTETS_MAX) {
              arret.abort();
              return null;
            }
            morceaux.push(value);
          }
          const texte = await avecSesImports(new TextDecoder().decode(await new Blob(morceaux).arrayBuffer()), fichier);
          vue_a_plat(texte, dossierDe(fichier), ""); // refusé par le moteur : le passage reste fermé
          return texte;
        } catch {
          return null;
        } finally {
          clearTimeout(delai);
        }
      })().then((texte) => {
        if (texte === null) fichiersLus.delete(fichier); // un échec n'est pas gardé : on pourra réessayer
        return texte;
      }));
    }
    return fichiersLus.get(fichier);
  }

  // Ce que le fichier affiché dit de sa vue (Zoom, Points, Portals).
  function lireLesReglages() {
    let actif, rangement;
    [densiteDemandee, reduire, zoomAvantPoints, actif, rangement, nombreDePortails, tailleDePortail, lumiereDuFond, vitesseDuZoom, dureeDuPortail, angleDeRotation, pointsActifs] = reglages_de_vue(source);
    pointsActifs = pointsActifs === 1;
    document.body.classList.toggle("tourne", angleDeRotation > 0 && actif === 1);
    // L'état de départ, avec ce que la page a gardé d'une visite précédente (keep: […]).
    if (!etats.has(chemin)) {
      let garde = "";
      try { garde = localStorage.getItem(`holo:${chemin}`) ?? ""; } catch { /* stockage refusé : on part du départ */ }
      etats.set(chemin, reprendre(source, garde));
    }
    reglerLesHorloges();
    document.documentElement.style.setProperty("--duree", `${dureeDuPortail}ms`);
    densite = densiteDemandee;
    reduire = reduire === 1;
    zoomActif = actif === 1;
    disposition = ["grille", "ligne", "colonne", "diagonale"][rangement];
    boutonPoints.style.display = zoomActif && pointsActifs ? "" : "none";
  }

  // Passe à un autre fichier sans recharger la page : son adresse devient celle de la barre
  // du navigateur, et le bouton « retour » ramène d'où l'on vient.
  async function ouvrirFichier(fichier, cheminDeSite = "", { dansLHistorique = true } = {}) {
    const texte = await lire(fichier);
    if (texte === null) return false;
    const depuis = location.pathname + location.hash;
    [chemin, base, source] = [fichier, dossierDe(fichier), texte];
    lireLesReglages();
    // Un navigateur interdit à une page d'afficher l'adresse d'un autre serveur comme si elle
    // y était. Pour un fichier d'ailleurs, l'adresse garde donc le fichier de départ, suivi de
    // #@ et de l'adresse où l'on est vraiment.
    if (dansLHistorique) {
      history.pushState({ depuis }, "", dAilleurs(fichier) ? `#@${adresseEntiere(fichier)}` : fichier + (cheminDeSite ? `#${cheminDeSite}` : ""));
    }
    afficherSite(cheminDeSite, { dansLHistorique: false });
    return true;
  }

  // Remonte d'un monde : au site qui contient celui-ci, ou au fichier d'où l'on est venu.
  function remonter() {
    if (performance.now() - dernierePassage < 700) return;
    if (site) afficherSite(parent(site));
    else if (history.state?.depuis) history.back();
    else return;
    dernierePassage = performance.now();
  }

  // ---------------------------------------------------------------- le site affiché

  // Affiche un site du fichier, sans recharger la page : pas de noir entre deux sites.
  // Le temps : une horloge par rythme écrit dans le fichier (Every(1s, …)). À chaque battement,
  // le signal est donné à l'arbitre, comme un toucher. L'horloge se tait quand la fenêtre est
  // cachée, en vue points et devant le carrefour : rien ne tourne pour rien.
  let touchesEcoutees = []; // les touches que les règles du fichier écoutent : On(Key.left, …)
  let battements = []; // une horloge par règle Every : { ms, valeur, minuterie }
  function lancer(horloge, rang) {
    clearInterval(horloge.minuterie);
    horloge.minuterie = setInterval(() => {
      if (document.hidden || enPoints || enMonde || !carrefour.hidden || entreeEnCours) return;
      emettre(`every:${rang}`);
    }, horloge.ms);
  }
  // Les données venues du serveur (data: Data(from: "stock.json", every: 30s)). La page va
  // chercher le fichier, rangé à côté d'elle, et le donne à l'arbitre, qui range ce qu'il veut
  // bien prendre. Elle ne parle qu'au serveur d'où elle vient.
  let rafraichissement = 0;
  async function chargerLesDonnees(fichier, pour) {
    try {
      const discret = dAilleurs(pour) ? { credentials: "omit", referrerPolicy: "no-referrer" } : {};
      const reponse = await fetch(dossierDe(pour) + fichier, { cache: "no-store", headers: { accept: "application/json" }, ...discret });
      if (!reponse.ok || pour !== chemin) return; // on a changé de fichier entre-temps
      const json = (await reponse.text()).slice(0, 65536);
      const avant = etats.get(chemin) ?? "";
      const apres = ranger(recevoir(source, avant, json));
      if (apres && apres !== avant && pour === chemin) {
        etats.set(chemin, apres);
        montrerLesValeurs();
        placerLesPixels();
        garder();
      }
    } catch { /* serveur muet : la page garde ses valeurs */ }
  }
  function reglerLesDonnees() {
    clearInterval(rafraichissement);
    const [fichier, rythme] = donnees(source).split("|");
    if (!fichier) return;
    const pour = chemin;
    chargerLesDonnees(fichier, pour);
    if (Number(rythme) > 0) {
      rafraichissement = setInterval(() => {
        if (pour !== chemin) clearInterval(rafraichissement);
        else if (!document.hidden) chargerLesDonnees(fichier, pour);
      }, Number(rythme));
    }
  }

  function reglerLesHorloges() {
    reglerLesDonnees();
    touchesEcoutees = touches(source).split(";").filter(Boolean);
    battements.forEach((horloge) => clearInterval(horloge.minuterie));
    battements = horloges(source).split(";").filter(Boolean).map((morceau) => {
      const [ms, valeurs] = morceau.split(":");
      return { ms: Number(ms), valeurs: valeurs.split(","), minuterie: 0 };
    });
    battements.forEach(lancer);
  }
  // Un geste vient de changer des valeurs : l'horloge de chacune repart de zéro. « Play » remet
  // le temps à trente : la première seconde dure une vraie seconde.
  function relancerLesHorloges(signal) {
    if (!battements.length || signal.startsWith("every:")) return;
    const changees = touchees(source, signal).split(";");
    battements.forEach((horloge, rang) => { if (horloge.valeurs.some((valeur) => changees.includes(valeur))) lancer(horloge, rang); });
  }
  // Ce que la page garde d'une visite à l'autre reste dans le navigateur du visiteur.
  function garder() {
    try {
      const garde = a_garder(source, etats.get(chemin) ?? "");
      if (garde) localStorage.setItem(`holo:${chemin}`, garde);
    } catch { /* stockage refusé ou plein : la page marche sans */ }
  }

  // Un résultat de l'arbitre peut porter, sous le nom « ! », des capacités demandées par une
  // règle de temps ou une règle qui guette (un son à faire entendre). On les applique, et on
  // rend l'état sans elles.
  function ranger(resultat) {
    const morceaux = resultat.split(";");
    const demandes = morceaux.find((morceau) => morceau.startsWith("!="));
    if (!demandes) return resultat;
    for (const effet of demandes.slice(2).split(",")) appliquer(effet);
    return morceaux.filter((morceau) => morceau !== demandes).join(";");
  }

  // Écrit les valeurs de la page là où ses textes les montrent : « {cart} ».
  function montrerLesValeurs(ou = racine, ecrit = etats.get(chemin) ?? "", texteDuFichier = source) {
    // Un texte voyage codé, précédé d'une apostrophe : buyer='Zo%C3%A9. Un nombre, tel quel.
    const lisible = (valeur) => (valeur.startsWith("'") ? decodeURIComponent(valeur.slice(1)) : valeur);
    const valeurs = new Map(ecrit.split(";").filter(Boolean).map((morceau) => {
      const coupe = morceau.indexOf("=");
      return [morceau.slice(0, coupe), lisible(morceau.slice(coupe + 1))];
    }));
    for (const place of ou.querySelectorAll("[data-state]")) {
      if (valeurs.has(place.dataset.state)) place.textContent = valeurs.get(place.dataset.state);
    }
    // Un champ et une case montrent leur valeur ; on ne récrit pas le champ où l'on est en train d'écrire.
    for (const champ of ou.querySelectorAll("[data-bind]")) {
      if (!valeurs.has(champ.dataset.bind)) continue;
      if (champ.type === "checkbox") champ.checked = valeurs.get(champ.dataset.bind) !== "0";
      else if (champ !== document.activeElement) champ.value = valeurs.get(champ.dataset.bind);
    }
    // Sur un plateau, un bloc suit les valeurs qui disent sa place : Point(x: star_x, y: star_y).
    for (const axe of ["x", "y"]) {
      for (const pose of ou.querySelectorAll(`[data-${axe}]`)) {
        if (valeurs.has(pose.dataset[axe])) pose.style.setProperty(`--${axe}`, Math.min(100, Number(valeurs.get(pose.dataset[axe]))));
      }
    }
    // Les conditions : If(count, is: 0). C'est le moteur qui répond ; la page ne compare rien
    // elle-même, elle cache ce que le moteur dit faux.
    const blocs = ou.querySelectorAll("[data-if]");
    if (!blocs.length) return;
    const reponses = new Map(conditions(texteDuFichier, ecrit).split(";").filter(Boolean).map((morceau) => {
      const coupe = morceau.lastIndexOf(":");
      return [morceau.slice(0, coupe), morceau.slice(coupe + 1) === "1"];
    }));
    for (const bloc of blocs) {
      if (reponses.has(bloc.dataset.if)) bloc.hidden = !reponses.get(bloc.dataset.if);
    }
  }

  // Un signal est émis (Add.tap) : l'arbitre du moteur dit ce que deviennent les valeurs, puis
  // les autres effets demandés par les règles sont appliqués.
  function emettre(signal) {
    const avant = etats.get(chemin) ?? "";
    const apres = ranger(arbitrer(source, avant, signal));
    if (apres !== avant) {
      etats.set(chemin, apres);
      montrerLesValeurs();
      placerLesPixels(); // ce qui apparaît ou disparaît déplace le reste de la page
      garder();
    }
    relancerLesHorloges(signal);
    for (const effet of effets(source, signal).split(",").filter(Boolean)) {
      appliquer(effet, signal);
    }
  }

  // `reprendre` : la page est déjà là, fabriquée par le serveur (ADR-033). Le moteur la prend
  // en main sans la redessiner : ce qui a été écrit en l'attendant reste, le focus aussi, et
  // les mouvements (ADR-034) ne repartent pas de zéro.
  function afficherSite(cheminDeSite, { dansLHistorique = true, reprendre = false } = {}) {
    sortirDesPoints();
    fermerCarrefour();
    if (cheminDeSite.startsWith("~")) {
      ouvrirMonde(cheminDeSite.slice(1), dansLHistorique);
      return;
    }
    if (enMonde) {
      enMonde = false;
      document.body.classList.remove("en-monde");
      pause(true);
      boutonMode.textContent = "Carrefour";
    }
    site = cheminDeSite;
    if (reprendre) adopterLesSaisies();
    else racine.innerHTML = vue_a_plat(source, base, site);
    montrerLesValeurs();
    cadre = racine.querySelector(".holo-Page");
    page = cadre.querySelector("main");
    document.title = cadre.dataset.title || "HoloCode";
    // Le fond de la fenêtre est celui du site : jamais de noir autour d'un site clair.
    const fond = getComputedStyle(cadre).backgroundColor;
    document.documentElement.style.setProperty("--fond", fond);
    document.body.style.background = fond;
    grossir(1);
    placerLesPixels();
    if (!reprendre) scrollTo(0, 0);
    if (dansLHistorique && !dAilleurs(chemin) && decodeURIComponent(location.hash.slice(1)) !== site) {
      history.pushState(history.state, "", site ? `#${site}` : location.pathname + location.search);
    }
    dernierePassage = performance.now();
    // Chez quelqu'un d'autre, on le dit tant qu'on y est (B-08).
    const origine = document.getElementById("origine");
    origine.hidden = !dAilleurs(chemin);
    origine.firstElementChild.textContent = `Vous êtes chez ${new URL(chemin, location.href).host}`;
    // Les fichiers où mènent les points de cette page sont lus d'avance, discrètement.
    // Seulement ceux du même serveur : on ne contacte pas le serveur de quelqu'un d'autre tant
    // que le visiteur n'a pas ouvert le carrefour.
    for (const contenu of sitesContenus()) {
      if (contenu.fichier && !dAilleurs(contenu.fichier)) lire(contenu.fichier);
    }
  }

  // Ce que le visiteur a écrit ou coché avant l'arrivée du moteur passe par l'arbitre, comme
  // une saisie ordinaire : rien n'est perdu, et la valeur est bornée comme d'habitude.
  function adopterLesSaisies() {
    for (const champ of racine.querySelectorAll("[data-bind]")) {
      const change = champ.type === "checkbox" ? champ.checked !== champ.defaultChecked : champ.value !== champ.defaultValue;
      if (!change) continue;
      const ecrit = champ.type === "checkbox" ? (champ.checked ? "1" : "0") : champ.value;
      etats.set(chemin, ranger(saisir(source, etats.get(chemin) ?? "", champ.dataset.bind, ecrit)));
    }
  }

  // Un monde calculé à partir d'une graine s'ouvre en profondeur : c'est le Big Bang. On y
  // zoome, on entre dans ses points, sans fin. Son adresse : fichier.holo#~graine.
  async function ouvrirMonde(graine, dansLHistorique) {
    if (!/^[0-9]+$/.test(graine)) return;
    const point = `Point(name: World, seed: ${graine}, fragments: 12)`;
    enMonde = true;
    document.body.classList.add("en-monde");
    document.body.style.background = "#000";
    document.title = `Monde ${graine.slice(-4)}`;
    boutonMode.textContent = "Retour";
    if (dansLHistorique) history.pushState(null, "", `#~${graine}`);
    if (!moteurLance) {
      moteurLance = true;
      await demarrer(zone(), point, 1);
    } else {
      retirer_mosaique();
      changer_de_monde(point, 1);
      pause(false);
    }
  }

  // Les points plantés dans un pixel de la page : chacun se place juste au-dessus du bloc
  // que son « above » désigne, à l'extrémité droite de la page.
  function placerLesPixels() {
    for (const pixel of page.querySelectorAll(".holo-pixel")) {
      const repere = page.querySelector(`[data-name="${CSS.escape(pixel.dataset.above)}"]`);
      if (!repere) continue;
      pixel.style.left = `${page.clientWidth - 1}px`;
      pixel.style.top = `${repere.offsetTop - 6}px`;
    }
  }

  // Les sites contenus dans le site affiché : ses points qui ont un intérieur.
  function sitesContenus() {
    return [...page.querySelectorAll(".holo-Point, .holo-pixel")]
      .filter((point) => point.dataset.file || racine.querySelector(`section[data-world="${CSS.escape(point.dataset.name)}"]`))
      .map((point) => ({
        fichier: point.dataset.file ?? null, // le monde de ce point est un autre fichier
        chemin: [site, point.dataset.name].filter(Boolean).join("/"),
        nom: point.dataset.name,
        couleur: point.style.getPropertyValue("--holo-color") || "#E9B44C",
      }));
  }

  // ---------------------------------------------------------------- le zoom ordinaire

  // Grossit la page vivante, comme le zoom d'un navigateur : le texte reste du texte.
  function grossir(valeur, x = innerWidth / 2, y = innerHeight / 2) {
    const [avantX, avantY] = [(scrollX + x) / zoomVif, (scrollY + y) / zoomVif];
    zoomVif = valeur;
    // Un vrai agrandissement, depuis le coin de la page : la mise en page ne bouge pas, tout
    // grossit ensemble. La zone à faire défiler grandit d'autant.
    const grossi = zoomVif !== 1;
    Object.assign(cadre.style, { transformOrigin: "0 0", transform: grossi ? `scale(${zoomVif})` : "", width: grossi ? `${document.documentElement.clientWidth}px` : "" });
    Object.assign(racine.style, { width: grossi ? `${cadre.offsetWidth * zoomVif}px` : "", height: grossi ? `${cadre.offsetHeight * zoomVif}px` : "", overflow: grossi ? "hidden" : "", cursor: grossi ? "grab" : "" });
    // Ce qui était sous le doigt y reste.
    scrollTo(avantX * zoomVif - x, avantY * zoomVif - y);
  }

  // ---------------------------------------------------------------- vue points

  const enAdresse = (blob) => new Promise((ok, raté) => {
    const lecteur = new FileReader();
    lecteur.onload = () => ok(lecteur.result);
    lecteur.onerror = raté;
    lecteur.readAsDataURL(blob);
  });

  // La page telle qu'on la voit, redessinée dans une image aux dimensions de la fenêtre.
  // Chrome ne donne pas les pixels d'une page ; mais celle-ci, c'est le moteur qui l'a
  // fabriquée : on peut la remettre dans une image (un SVG qui la contient).
  async function imageDeLaPage() {
    const copie = racine.cloneNode(true);
    // Une image ne peut pas aller chercher d'autres fichiers : celles de la page sont copiées dedans.
    const originales = racine.querySelectorAll("img");
    for (const [i, img] of [...copie.querySelectorAll("img")].entries()) {
      try {
        img.setAttribute("src", await enAdresse(await (await fetch(originales[i].src)).blob()));
      } catch {
        img.removeAttribute("src"); // une image qu'un autre serveur ne laisse pas copier
      }
    }
    const [l, h] = [innerWidth, innerHeight];
    const svg =
      `<svg xmlns="http://www.w3.org/2000/svg" width="${l * densite}" height="${h * densite}" viewBox="0 0 ${l} ${h}">` +
      `<foreignObject width="${l}" height="${h}"><div xmlns="http://www.w3.org/1999/xhtml" style="width:${l}px;height:${h}px;overflow:hidden">` +
      `<style>.holo-Page{min-height:${h / zoomVif}px!important}</style>` +
      `<div style="margin:${-scrollY}px 0 0 ${-scrollX}px">${new XMLSerializer().serializeToString(copie)}</div></div></foreignObject></svg>`;
    image.src = "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svg);
    await image.decode();
    const toile = new OffscreenCanvas(l * densite, h * densite);
    const dessin = toile.getContext("2d");
    dessin.drawImage(image, 0, 0, toile.width, toile.height);
    image.style.width = `${toile.width}px`;
    image.style.height = `${toile.height}px`;
    return dessin.getImageData(0, 0, toile.width, toile.height);
  }

  // Le suivi ne tourne que pendant qu'on bouge : immobile, il s'arrête, comme le moteur.
  let suiviActif = false;
  let derniereCamera = "";
  let imagesImmobiles = 0;

  function reveillerLeSuivi() {
    if (!enPoints) return;
    reveiller();
    imagesImmobiles = 0;
    if (!suiviActif) {
      suiviActif = true;
      requestAnimationFrame(suivreLesPoints);
    }
  }

  function suivreLesPoints() {
    suiviActif = false;
    if (!enPoints) return;
    const camera = mosaique_camera();
    if (camera.length) {
      const [cx, cy, echelle, opacite, niveau, points, lacet, tangage, distance, echelleAuRepos] = camera;
      // Revenu exactement à la vue de départ, de face : c'est le site normal, on y retourne.
      const auRepos = Math.abs(echelle / echelleAuRepos - 1) < 1e-9 && lacet === 0 && tangage === 0;
      if (auRepos && aQuitteLeRepos) {
        // Si l'on s'est déplacé pendant la vue points, la page vivante reprend au même endroit.
        const [decalageX, decalageY] = [(cx - image.naturalWidth / 2) / densite, (cy - image.naturalHeight / 2) / densite];
        sortirDesPoints();
        scrollBy(decalageX, decalageY);
        return;
      }
      aQuitteLeRepos ||= !auRepos;
      // L'image de la page suit la même vue, de face comme de biais, et s'efface quand les
      // points prennent le relais.
      image.style.transform =
        `translate(${innerWidth / 2}px,${innerHeight / 2}px) perspective(${distance}px) rotateX(${tangage}rad) rotateY(${lacet}rad) ` +
        `translate(${-cx * echelle}px,${-cy * echelle}px) scale(${echelle})`;
      image.style.opacity = 1 - opacite;
      etat.textContent = opacite === 0
        ? `${nombre.format(image.naturalWidth * image.naturalHeight)} points — zoomez encore`
        : `${nombre.format(points)} points à l'écran · morcelés ${niveau} fois`;
    }
    const cle = camera.join();
    imagesImmobiles = cle === derniereCamera ? imagesImmobiles + 1 : 0;
    derniereCamera = cle;
    if (imagesImmobiles > 6) return; // plus rien ne bouge : on s'arrête jusqu'au prochain geste
    suiviActif = true;
    requestAnimationFrame(suivreLesPoints);
  }

  // On zoome vers un endroit de l'écran, comme si on s'en approchait.
  async function approcher(x, y) {
    if (calme.matches) return; // animations réduites : on n'avance pas vers le point, on ouvre directement
    for (let pas = 0; pas < 45 && mosaique_camera()[2] * densite * zoomVif < 280; pas++) {
      zone().dispatchEvent(new WheelEvent("wheel", { deltaY: -70, clientX: x, clientY: y, cancelable: true }));
      reveillerLeSuivi();
      await new Promise(requestAnimationFrame);
    }
  }

  // En vue points, toucher un point planté par l'auteur l'active : on s'en approche, puis le
  // carrefour s'ouvre sur le site qu'il contient.
  async function toucherUnPoint(x, y) {
    const sous = mosaique_sous(x, y);
    if (!sous.length) return;
    const tolerance = Math.max(1.5 * densite * zoomVif, 14 / mosaique_camera()[2]);
    const plante = pixelsPlantes.find((p) => Math.abs(p.x - sous[0]) < tolerance && Math.abs(p.y - sous[1]) < tolerance);
    if (!plante) return;
    await approcher(x, y);
    sortirDesPoints();
    emettre(`${plante.nom}.tap`);
  }

  // La page devient un ensemble de points, un par pixel. Au départ rien ne change à l'écran :
  // un point fait exactement un pixel.
  function entrerEnPoints() {
    if (enPoints || !carrefour.hidden || !pointsActifs) return Promise.resolve();
    entreeEnCours ??= (async () => {
      densite = Math.min(densiteDemandee, Math.sqrt(POINTS_MAX / (innerWidth * innerHeight)));
      const pixels = await imageDeLaPage();
      const couleurs = new Uint8Array(pixels.data.buffer);
      pixelsPlantes = [...page.querySelectorAll(".holo-pixel")].map((pixel) => {
        const cadreDuPixel = pixel.getBoundingClientRect();
        return { nom: pixel.dataset.name, x: (cadreDuPixel.left + cadreDuPixel.width / 2) * densite, y: (cadreDuPixel.top + cadreDuPixel.height / 2) * densite };
      });
      document.body.classList.add("en-points");
      if (!moteurLance) {
        moteurLance = true;
        await demarrer_mosaique(zone(), couleurs, pixels.width, pixels.height, source, zoomVif);
      } else {
        poser_mosaique(couleurs, pixels.width, pixels.height, source, zoomVif);
        pause(false);
      }
      page.style.visibility = "hidden";
      boutonPoints.textContent = "Vue web";
      enPoints = true;
      aQuitteLeRepos = false;
      reveillerLeSuivi();
    })().finally(() => { entreeEnCours = null; });
    return entreeEnCours;
  }

  function sortirDesPoints() {
    if (!enPoints) return;
    enPoints = false;
    mosaique_de_face();
    mosaique_tourner(false);
    retirer_mosaique();
    pause(true);
    boutonTourner.setAttribute("aria-pressed", "false");
    document.body.classList.remove("en-points");
    boutonPoints.textContent = "Vue points";
    // L'image de la page et les repères ne servent plus : on rend la mémoire (B-10).
    image.removeAttribute("src");
    pixelsPlantes = [];
    page.style.visibility = "";
  }

  // Un pincement arrive comme une molette avec Ctrl, par petits pas : on les grossit.
  const pasDeZoom = (evenement) => evenement.deltaY * (Math.abs(evenement.deltaY) < 50 ? 6 : 1);

  // Zoomer sur la page (Ctrl + molette, ou pincer). D'abord un zoom ordinaire, où la page reste
  // un site qu'on lit et qu'on copie ; au-delà de Points(after:), ses pixels deviennent des points.
  // Un seul chemin pour tout zoom, qu'il vienne de la molette ou de deux doigts : la page
  // vivante d'abord, les points ensuite.
  async function zoomer(facteur, clientX, clientY) {
    const deltaY = -Math.log2(facteur) / 0.003;
    if (enPoints) {
      zone().dispatchEvent(new WheelEvent("wheel", { deltaY, clientX, clientY, cancelable: true }));
      reveillerLeSuivi();
      return;
    }
    if (!carrefour.hidden || !cadre || entreeEnCours || enMonde || !zoomActif) return;
    const voulu = zoomVif * facteur;
    if (voulu <= zoomAvantPoints && (voulu >= 1 || zoomVif > 1)) {
      grossir(Math.max(1, voulu), clientX, clientY);
      return;
    }
    // Garde-fou : on ne dézoome pas en deçà de la page entière, sauf si le fichier le permet
    // (Zoom(shrink: true)).
    if (voulu < 1 && !reduire) {
      // Dézoomer alors que la page est déjà entière : on ressort du monde où l'on est.
      if (zoomVif === 1) remonter();
      return;
    }
    if (voulu > zoomAvantPoints && zoomVif < zoomAvantPoints) grossir(zoomAvantPoints, clientX, clientY);
    // Sans points demandés par l'auteur, ou avec les animations réduites par le visiteur : la
    // page reste une page, on a grossi autant que permis.
    if (!pointsActifs || calme.matches) return;
    await entrerEnPoints();
    zone().dispatchEvent(new WheelEvent("wheel", { deltaY, clientX, clientY, cancelable: true }));
  }

  // À la souris : Ctrl + molette (un pavé tactile envoie la même chose quand on pince).
  addEventListener("wheel", (evenement) => {
    // Sinon Chrome grossit ou réduit toute la fenêtre, et le site sort de son cadre.
    if (evenement.ctrlKey) evenement.preventDefault();
    if (enPoints || !evenement.ctrlKey) return; // en vue points, la zone de dessin reçoit la molette elle-même
    zoomer(2 ** (-pasDeZoom(evenement) * 0.003 * vitesseDuZoom), evenement.clientX, evenement.clientY);
  }, { passive: false });

  // Au doigt : pincer la page. Le geste commencé sur la page continue sans lever les doigts
  // quand ses pixels deviennent des points, et dans l'autre sens quand on revient.
  let pincement = null; // l'écartement des deux doigts au dernier mouvement
  const deuxDoigts = (touches) => ({
    ecart: Math.hypot(touches[0].clientX - touches[1].clientX, touches[0].clientY - touches[1].clientY),
    x: (touches[0].clientX + touches[1].clientX) / 2,
    y: (touches[0].clientY + touches[1].clientY) / 2,
  });
  addEventListener("touchstart", (evenement) => {
    // Un pincement commencé sur la zone de dessin est suivi par le moteur lui-même.
    const surLaPage = evenement.touches.length === 2 && !enPoints && !enMonde && carrefour.hidden;
    pincement = surLaPage ? deuxDoigts(evenement.touches).ecart : null;
  }, { capture: true, passive: true });
  addEventListener("touchmove", (evenement) => {
    if (pincement === null || evenement.touches.length !== 2) return;
    evenement.preventDefault(); // ni le zoom ni le défilement du navigateur pendant qu'on pince
    const { ecart, x, y } = deuxDoigts(evenement.touches);
    if (Math.abs(ecart / pincement - 1) < 0.015) return; // un tremblement n'est pas un pincement
    zoomer(ecart / pincement, x, y);
    pincement = ecart;
  }, { capture: true, passive: false });
  for (const fin of ["touchend", "touchcancel"]) {
    addEventListener(fin, (evenement) => { if (evenement.touches.length < 2) pincement = null; }, { capture: true, passive: true });
  }

  // ---------------------------------------------------------------- le carrefour

  // Un portail : un rond lumineux qui montre le site où il mène.
  function portail({ chemin: versSite, nom, couleur, calcule = false, fichier = null, texte = null, retour = false, aLire = false, inerte = false }, taille, vers) {
    const bouton = document.createElement("button");
    bouton.type = "button";
    bouton.className = vers ? "portail vers" : "portail";
    if (retour) bouton.dataset.retour = "oui";
    else if (fichier) bouton.dataset.fichier = fichier;
    else bouton.dataset.chemin = versSite;
    // Un fichier introuvable ou refusé par le moteur : le passage reste fermé.
    if (inerte) bouton.disabled = true;
    if (fichier && aLire) {
      // Pas encore lu : pas d'aperçu, mais le nom du serveur où l'on irait.
      calcule = true;
      nom = `${nom} · ${new URL(fichier, location.href).host}`;
      if (!passagePermis(fichier)) {
        couleur = "#555";
        nom = `${nom} (fermé)`;
        bouton.disabled = true;
      }
    } else if (fichier && texte === null) {
      calcule = true;
      couleur = "#555";
      nom = `${nom} (fermé)`;
      bouton.disabled = true;
    } else if (fichier && dAilleurs(fichier)) {
      // On dit où mène un passage vers le site de quelqu'un d'autre.
      nom = `${nom} · ${new URL(fichier, location.href).host}`;
    }
    bouton.style.setProperty("--taille", `${taille}px`);
    bouton.style.setProperty("--couleur", couleur);
    const apercu = document.createElement("span");
    apercu.className = calcule ? "apercu calcule" : "apercu";
    const etiquetteSeule = document.createElement("span");
    etiquetteSeule.className = "nom";
    etiquetteSeule.textContent = nom;
    if (calcule) {
      // Un monde calculé n'a pas de page à montrer : une boule de lumière de sa couleur.
      bouton.append(apercu, etiquetteSeule);
      return bouton;
    }
    const contenu = document.createElement("span");
    contenu.className = "contenu";
    contenu.inert = true;
    contenu.style.width = `${innerWidth}px`;
    contenu.style.height = `${innerWidth}px`; // un carré : l'aperçu remplit le rond
    contenu.style.transform = `scale(${taille / innerWidth})`;
    // L'aperçu est enfermé : les styles d'un autre fichier ne débordent ni sur la page ni sur
    // les autres aperçus.
    const interieur = contenu.attachShadow({ mode: "open" });
    interieur.innerHTML =
      "<style>.holo-Page{min-height:100%!important}</style>" + (fichier ? vue_a_plat(texte, dossierDe(fichier), "") : vue_a_plat(source, base, versSite));
    // L'aperçu montre les valeurs où elles en sont : celles de ce fichier, ou de l'autre s'il a déjà été visité.
    montrerLesValeurs(interieur, etats.get(fichier || chemin) ?? (fichier ? etat_initial(texte) : ""), fichier ? texte : source);
    apercu.append(contenu);
    const etiquette = document.createElement("span");
    etiquette.className = "nom";
    etiquette.textContent = nom;
    bouton.append(apercu, etiquette);
    return bouton;
  }

  // Ouvre le carrefour. « vers » : le site contenu vers lequel on se dirige, mis en avant ;
  // sans lui, c'est la carte de ce qui entoure le site où l'on est.
  async function ouvrirCarrefour(vers = null) {
    sortirDesPoints();
    const contenus = sitesContenus();
    // Les autres fichiers sont petits et déjà demandés : on les attend pour montrer leur aperçu.
    // Ceux d'un autre serveur ne sont pas lus ici : un clic, un serveur, un fichier (B-07).
    for (const contenu of contenus) {
      if (!contenu.fichier) continue;
      if (dAilleurs(contenu.fichier)) contenu.aLire = true;
      else contenu.texte = await lire(contenu.fichier);
    }
    const ici = { chemin: site, nom: site ? `Ici : ${dernier(site)}` : "Ici", couleur: "#ffffff" };
    const cible = contenus.find((s) => s.nom === vers);
    const portails = [];
    if (site) portails.push(portail({ chemin: parent(site), nom: `Retour : ${dernier(parent(site)) || "accueil"}`, couleur: "#9aa4b2" }, tailleDePortail, false));
    // Venu d'un autre fichier : un portail pour y retourner.
    else if (history.state?.depuis) portails.push(portail({ nom: "Retour", couleur: "#9aa4b2", calcule: true, retour: true }, tailleDePortail, false));
    // Portals(count:) borne tout le carrefour, y compris les sites écrits par l'auteur (B-02).
    const autres = contenus.filter((contenu) => contenu !== cible);
    const place = Math.max(0, nombreDePortails - 1 - portails.length - (cible ? 1 : 0));
    for (const contenu of autres.slice(0, place)) portails.push(portail(contenu, tailleDePortail, false));
    if (autres.length > place) {
      portails.push(portail({ nom: `+ ${autres.length - place} autres`, couleur: "#555", calcule: true, inerte: true }, tailleDePortail, false));
    }
    if (cible) portails.push(portail(ici, tailleDePortail, false));
    // Le reste de la place est rempli par des mondes calculés à partir d'une graine : on peut
    // y entrer, comme dans le Big Bang.
    const manquants = Math.max(0, nombreDePortails - portails.length - 1);
    for (const voisin of mondes_voisins(source, site, manquants).split(";").filter(Boolean)) {
      const [graine, couleur] = voisin.split(":");
      portails.push(portail({ chemin: `~${graine}`, nom: `Monde ${graine.slice(-4)}`, couleur: `rgb(${couleur})`, calcule: true }, tailleDePortail, false));
    }
    // Le portail mis en avant est plus grand ; en grille il se place au milieu des autres.
    const grand = Math.round(Math.min(tailleDePortail * 1.9, Math.min(innerWidth, innerHeight) * 0.42));
    portails.splice(disposition === "grille" ? Math.ceil(portails.length / 2) : 0, 0, portail(cible ?? ici, grand, true));
    portails.forEach((element, rang) => element.style.setProperty("--rang", rang));
    carrefour.className = disposition;
    carrefour.style.setProperty("--lueur", `${Math.round(lumiereDuFond * 100)}%`);
    carrefour.replaceChildren(...portails);
    carrefour.hidden = false;
    boutonMode.textContent = "Fermer";
    boutonPoints.hidden = true;
    carrefour.querySelector(".vers").focus({ preventScroll: true });
  }

  // Une adresse désigne le fichier d'un autre serveur. On ne le contacte pas sans un geste du
  // visiteur : on lui montre où il irait, et c'est lui qui décide (B-07).
  function proposerPassage(fichier) {
    const grand = Math.round(Math.min(tailleDePortail * 1.9, Math.min(innerWidth, innerHeight) * 0.42));
    const vers = portail({ fichier, nom: "Aller", couleur: "#E9B44C", aLire: true }, grand, true);
    const rester = portail({ chemin: site, nom: "Rester ici", couleur: "#ffffff" }, tailleDePortail, false);
    carrefour.className = "grille";
    carrefour.style.setProperty("--lueur", `${Math.round(lumiereDuFond * 100)}%`);
    carrefour.replaceChildren(vers, rester);
    carrefour.hidden = false;
    boutonMode.textContent = "Fermer";
    boutonPoints.hidden = true;
  }

  function fermerCarrefour() {
    carrefour.hidden = true;
    carrefour.replaceChildren();
    boutonMode.textContent = "Carrefour";
    boutonPoints.hidden = false;
  }

  // Le portail s'ouvre : son aperçu grandit jusqu'à remplir la fenêtre, et devient le site.
  async function franchir(bouton) {
    const versSite = bouton.dataset.chemin;
    if (bouton.dataset.retour) {
      history.back();
      return;
    }
    if (versSite === site) {
      fermerCarrefour();
      return;
    }
    const apercu = bouton.querySelector(".apercu");
    if (bouton.dataset.fichier && apercu.classList.contains("calcule")) {
      // Le fichier d'un autre serveur, lu seulement maintenant que le visiteur l'a choisi.
      const dejaLa = decodeURIComponent(location.hash.slice(1)) === `@${adresseEntiere(bouton.dataset.fichier)}`;
      if (!(await ouvrirFichier(bouton.dataset.fichier, "", { dansLHistorique: !dejaLa }))) {
        bouton.disabled = true;
        bouton.querySelector(".nom").textContent += " (fermé)";
        bouton.style.setProperty("--couleur", "#555");
      }
      return;
    }
    if (apercu.classList.contains("calcule")) {
      afficherSite(versSite);
      return;
    }
    const depart = apercu.getBoundingClientRect();
    Object.assign(apercu.style, { left: `${depart.left}px`, top: `${depart.top}px`, width: `${depart.width}px`, height: `${depart.height}px` });
    apercu.classList.add("ouverture");
    apercu.getBoundingClientRect(); // le départ est pris en compte avant de lancer le mouvement
    Object.assign(apercu.style, { left: "0px", top: "0px", width: "100vw", height: "100vh", borderRadius: "0" });
    Object.assign(apercu.querySelector(".contenu").style, { transform: "scale(1)", height: `${innerHeight}px` });
    const immobile = calme.matches;
    await new Promise((suite) => setTimeout(suite, immobile ? 0 : dureeDuPortail + 20));
    if (bouton.dataset.fichier) await ouvrirFichier(bouton.dataset.fichier);
    else afficherSite(versSite);
  }

  // ---------------------------------------------------------------- les règles du fichier

  // Le passage ordinaire d'un site à l'autre, comme sur le web : la page s'efface, la suivante
  // apparaît. Bref et discret. Avec les animations réduites, le changement est immédiat.
  async function fondu(changer) {
    if (calme.matches) return changer();
    racine.style.transition = "opacity .16s ease";
    racine.style.opacity = "0";
    await new Promise((suite) => setTimeout(suite, 170));
    await changer();
    racine.getBoundingClientRect(); // la page neuve est d'abord prise en compte invisible
    racine.style.opacity = "1";
    setTimeout(() => { racine.style.transition = ""; racine.style.opacity = ""; }, 200);
  }

  // Entrer dans un point : on y va directement. Le carrefour ne s'ouvre que si on le demande
  // (son bouton, ou la capacité « portals »). Deux façons d'y aller, deux mouvements :
  // - par un bouton de la page (« Enter the workshop ») : c'est un lien, on passe d'un site à
  //   l'autre comme sur le web, par un simple fondu ;
  // - en touchant le point lui-même : là on est dans le métavers, le point s'ouvre où il est,
  //   grandit jusqu'à remplir la fenêtre, et devient le site.
  let entreeDirecte = false;
  async function entrerDans(nom, parLePoint = false) {
    const contenu = sitesContenus().find((s) => s.nom === nom);
    if (!contenu || entreeDirecte) return;
    // Le fichier d'un autre serveur : on ne le lit pas sans que le visiteur ait vu où il va.
    // Là, le carrefour garde son rôle : il affiche le nom du serveur, et attend un clic.
    if (contenu.fichier && dAilleurs(contenu.fichier)) return ouvrirCarrefour(nom);
    if (contenu.fichier) {
      contenu.texte = await lire(contenu.fichier);
      if (contenu.texte === null) return ouvrirCarrefour(nom); // introuvable ou refusé : le carrefour le montre fermé
    }
    sortirDesPoints();
    entreeDirecte = true;
    if (!parLePoint) {
      try {
        await fondu(() => (contenu.fichier ? ouvrirFichier(contenu.fichier) : afficherSite(contenu.chemin)));
      } finally {
        entreeDirecte = false;
      }
      return;
    }
    const point = page.querySelector(`[data-name="${CSS.escape(nom)}"]`).getBoundingClientRect();
    const taille = Math.max(point.width, 24);
    const bouton = portail(contenu, taille, true);
    bouton.querySelector(".nom").remove();
    const passage = document.createElement("div");
    passage.id = "passage";
    Object.assign(bouton.style, { position: "fixed", left: `${point.left + point.width / 2 - taille / 2}px`, top: `${point.top + point.height / 2 - taille / 2}px` });
    passage.append(bouton);
    document.body.append(passage);
    try {
      await franchir(bouton);
    } finally {
      passage.remove();
      entreeDirecte = false;
    }
  }

  function appliquer(effet, signal = "") {
    const [nom, capacite] = effet.split(".");
    if (capacite === "enter" && sitesContenus().some((s) => s.nom === nom)) {
      // Le signal vient-il du point lui-même, ou d'un bouton qui y mène ?
      entrerDans(nom, signal === `${nom}.tap`);
    } else if (capacite === "play") {
      // Faire entendre un son. Un navigateur ne joue un son qu'après un premier geste du
      // visiteur : avant, il refuse, et la page continue sans lui.
      const son = racine.querySelector(`audio[data-name="${CSS.escape(nom)}"]`);
      if (son) {
        son.currentTime = 0;
        son.play().catch(() => {});
      }
    } else if (capacite === "portals") {
      // La page demande son carrefour : On(Map.tap, effect: Shop.portals).
      ouvrirCarrefour();
    } else if (capacite === "leave") {
      // Sortir d'un site : on remonte à celui qui le contient.
      if (!carrefour.hidden) fermerCarrefour();
      else if (dernier(site) === nom) fondu(() => afficherSite(parent(site)));
    }
  }

  try {
    try {
      await init();
    } catch (e) {
      // Le moteur n'a pas pu arriver (réseau, fichier absent) : la page le dit, reste lisible,
      // et propose de réessayer. Aucun toucher n'est compté comme fait.
      window.__holoEchec?.(true);
      throw new Error("le moteur n'a pas pu démarrer");
    }
    source = await avecSesImports(await (await fetch(chemin, { headers: { accept: "text/plain" } })).text(), chemin);
    fichiersLus.set(chemin, Promise.resolve(source));
    lireLesReglages();
    const departDuSite = decodeURIComponent(location.hash.slice(1));
    // La page du fichier est déjà là, fabriquée par le serveur : on la reprend. Un monde
    // demandé par l'adresse (#Atelier), lui, se dessine.
    const dejaLa = !departDuSite && racine.querySelector(".holo-Page") !== null;
    afficherSite(departDuSite.startsWith("@") ? "" : departDuSite, { dansLHistorique: false, reprendre: dejaLa });
    // Une adresse en #@… désigne le fichier d'un autre serveur : on propose le passage.
    if (departDuSite.startsWith("@")) proposerPassage(departDuSite.slice(1));
    // « Revenir » : par où l'on est venu, ou, si l'on est arrivé directement, au fichier de départ.
    document.querySelector("#origine button").addEventListener("click", () => (history.state?.depuis ? history.back() : ouvrirFichier(location.pathname)));
    // Le bouton « retour » du navigateur, ou une adresse changée à la main.
    addEventListener("popstate", () => {
      const cheminDeSite = decodeURIComponent(location.hash.slice(1));
      // L'adresse désigne un autre fichier : on y passe, toujours sans recharger.
      // Un fichier d'ailleurs déjà lu pendant cette visite : le visiteur l'avait choisi. Sinon, on propose.
      if (cheminDeSite.startsWith("@") && fichiersLus.has(cheminDeSite.slice(1))) ouvrirFichier(cheminDeSite.slice(1), "", { dansLHistorique: false });
      else if (cheminDeSite.startsWith("@")) proposerPassage(cheminDeSite.slice(1));
      else if (location.pathname !== chemin && location.pathname.endsWith(".holo")) ouvrirFichier(location.pathname, cheminDeSite, { dansLHistorique: false });
      else afficherSite(cheminDeSite, { dansLHistorique: false });
    });
    // Dans un monde calculé : dézoomer alors qu'on est revenu tout en haut en fait ressortir.
    addEventListener("wheel", (evenement) => {
      const mesures = window.__holo;
      if (enMonde && evenement.deltaY > 0 && mesures && mesures.profondeur === 0 && mesures.zoom === 0 && performance.now() - dernierePassage > 1500) {
        dernierePassage = performance.now();
        history.back();
      }
    }, { passive: true });
    addEventListener("resize", () => { placerLesPixels(); reveillerLeSuivi(); });

    // En vue points, un toucher sans glissement active le point planté qui se trouve dessous.
    let appui = null;
    addEventListener("pointerdown", (evenement) => { appui = [evenement.clientX, evenement.clientY]; }, true);
    addEventListener("pointerup", (evenement) => {
      const glissement = appui ? Math.hypot(evenement.clientX - appui[0], evenement.clientY - appui[1]) : Infinity;
      appui = null;
      if (enPoints && glissement < 6 && !evenement.target.closest("button")) toucherUnPoint(evenement.clientX, evenement.clientY);
    }, true);
    // Page grossie : glisser la déplace, dans tous les sens, comme en vue points. Le geste est
    // donc le même du début à la fin du zoom. (Un double clic sélectionne toujours un mot.)
    let prise = null;
    racine.addEventListener("mousedown", (evenement) => {
      if (zoomVif === 1 || enPoints || evenement.button !== 0 || evenement.detail > 1) return;
      evenement.preventDefault(); // pas de sélection de texte en glissant
      prise = { x: evenement.clientX, y: evenement.clientY, glisse: false };
      racine.style.cursor = "grabbing";
    });
    addEventListener("mousemove", (evenement) => {
      if (!prise) return;
      if (Math.hypot(evenement.clientX - prise.x, evenement.clientY - prise.y) > 4) prise.glisse = true;
      scrollBy(-evenement.movementX, -evenement.movementY);
    });
    addEventListener("mouseup", () => {
      if (!prise) return;
      // Après un glissement, le relâchement ne compte pas comme un clic sur un bouton.
      if (prise.glisse) addEventListener("click", (clic) => clic.stopPropagation(), { capture: true, once: true });
      prise = null;
      racine.style.cursor = zoomVif === 1 ? "" : "grab";
    });
    // Sur la page : un toucher est envoyé au moteur, qui répond par les effets demandés.
    racine.addEventListener("click", (evenement) => {
      const bloc = evenement.target.closest("[data-name]");
      if (!bloc) return;
      emettre(`${bloc.dataset.name}.tap`);
    });
    // Faire glisser un bloc d'un plateau (drag: true), au doigt ou à la souris. La page dit à
    // l'arbitre où est le doigt, de 0 à 100 ; c'est lui qui change les valeurs.
    let glissement = null;
    racine.addEventListener("pointerdown", (evenement) => {
      const pose = evenement.target.closest(".holo-place[data-drag]");
      if (!pose) return;
      glissement = { pose, plateau: pose.parentElement };
      pose.classList.add("holo-glisse");
      pose.setPointerCapture(evenement.pointerId);
      evenement.preventDefault();
    });
    racine.addEventListener("pointermove", (evenement) => {
      if (!glissement) return;
      const { pose, plateau } = glissement;
      const [cadre, taille] = [plateau.getBoundingClientRect(), pose.getBoundingClientRect()];
      // Le centre du bloc suit le doigt ; 0 et 100 sont les deux bords où le bloc touche le plateau.
      const vers = (doigt, debut, longueur, largeur) => Math.round(Math.min(100, Math.max(0, ((doigt - debut - largeur / 2) / Math.max(1, longueur - largeur)) * 100)));
      const [x, y] = [vers(evenement.clientX, cadre.left, cadre.width, taille.width), vers(evenement.clientY, cadre.top, cadre.height, taille.height)];
      const avant = etats.get(chemin) ?? "";
      const apres = ranger(glisser(source, avant, pose.dataset.drag, x, y));
      if (apres !== avant) {
        etats.set(chemin, apres);
        montrerLesValeurs();
        garder();
      }
    });
    for (const fin of ["pointerup", "pointercancel"]) {
      racine.addEventListener(fin, () => {
        glissement?.pose.classList.remove("holo-glisse");
        glissement = null;
      });
    }
    // Le clavier : une touche que le fichier écoute devient un signal, comme un toucher. Les
    // autres touches gardent leur rôle (défiler, écrire), et rien n'est pris à un champ où l'on écrit.
    const nomDeTouche = { ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down", " ": "space" };
    addEventListener("keydown", (evenement) => {
      const touche = nomDeTouche[evenement.key];
      if (!touche || !touchesEcoutees.includes(touche) || evenement.ctrlKey || evenement.altKey || evenement.metaKey) return;
      if (evenement.target.closest?.("input, textarea, select, button") || enPoints || enMonde || !carrefour.hidden) return;
      evenement.preventDefault();
      emettre(`Key.${touche}`);
    });
    // Écrire dans un champ, cocher une case : c'est l'arbitre du moteur qui change la valeur.
    racine.addEventListener("input", (evenement) => {
      const champ = evenement.target.closest("[data-bind]");
      if (!champ) return;
      const ecrit = champ.type === "checkbox" ? (champ.checked ? "1" : "0") : champ.value;
      etats.set(chemin, ranger(saisir(source, etats.get(chemin) ?? "", champ.dataset.bind, ecrit)));
      montrerLesValeurs();
      placerLesPixels();
      garder();
    });
    // En quittant un champ, il montre la valeur que l'arbitre a retenue (bornée).
    racine.addEventListener("change", () => montrerLesValeurs());
    // Le moteur est prêt : on rejoue ce qui a été touché en l'attendant.
    window.__holoArreterDeNoter?.();
    for (const nom of touchersEnAttente.splice(0)) emettre(`${nom}.tap`);
    carrefour.addEventListener("click", (evenement) => {
      const bouton = evenement.target.closest(".portail");
      if (bouton) franchir(bouton);
      else fermerCarrefour(); // un clic à côté des portails : on reste où l'on est
    });
    addEventListener("keydown", (evenement) => { if (evenement.key === "Escape" && !carrefour.hidden) fermerCarrefour(); });

    document.getElementById("menu").hidden = params.has("nu"); // ?nu : sans les boutons, pour en tirer une image
    // Le bouton unique : il ouvre les outils, puis devient la croix qui les referme.
    const bascule = document.getElementById("bascule");
    const outils = document.getElementById("outils");
    const ouvrirLesOutils = (ouvert) => {
      outils.hidden = !ouvert;
      bascule.setAttribute("aria-expanded", String(ouvert));
      bascule.setAttribute("aria-label", ouvert ? "Fermer les outils" : "Ouvrir les outils");
      bascule.textContent = ouvert ? "✕" : "☰";
    };
    bascule.addEventListener("click", () => ouvrirLesOutils(outils.hidden));
    // Le bouton a été touché avant que le moteur arrive : on ouvre les outils maintenant.
    if (window.__holoMenuDemande) ouvrirLesOutils(true);
    addEventListener("keydown", (evenement) => { if (evenement.key === "Escape") ouvrirLesOutils(false); });
    // Changer de vue ou ouvrir le carrefour referme le menu : l'écran revient au site.
    // « Tourner » le laisse ouvert, pour garder « De face » sous la main.
    for (const bouton of [boutonPoints, boutonMode]) bouton.addEventListener("click", () => ouvrirLesOutils(false));
    boutonMode.addEventListener("click", () => {
      if (enMonde) history.back(); // on quitte le monde calculé pour revenir au site
      else if (carrefour.hidden) ouvrirCarrefour();
      else fermerCarrefour();
    });
    boutonPoints.addEventListener("click", () => (enPoints ? sortirDesPoints() : entrerEnPoints()));
    boutonTourner.addEventListener("click", async () => {
      const actif = boutonTourner.getAttribute("aria-pressed") !== "true";
      // De face, sur le site ordinaire : la page devient d'abord ses points, sans que rien ne
      // change à l'écran, puis elle tourne sous le doigt.
      if (!enPoints) await entrerEnPoints();
      if (!enPoints) return;
      boutonTourner.setAttribute("aria-pressed", String(actif));
      mosaique_tourner(actif);
    });
    document.getElementById("face").addEventListener("click", () => { mosaique_de_face(); reveillerLeSuivi(); });
    // En vue points, tout geste relance le suivi ; une souris qui passe sans bouton ne compte pas.
    for (const nom of ["wheel", "pointerdown", "pointermove", "pointerup"]) {
      addEventListener(nom, (evenement) => {
        if (nom !== "pointermove" || evenement.buttons !== 0) reveillerLeSuivi();
      }, { capture: true, passive: true });
    }
    // En vue points, le bouton droit sert à tourner la page : pas de menu.
    addEventListener("contextmenu", (evenement) => { if (enPoints) evenement.preventDefault(); });

    // Pour les captures d'écran : ?vue=carrefour, ?entrer=Workshop,
    // ?vue=points avec &zoom=…&x=…&y=… (comme la molette à cet endroit) et &lacet=…&tangage=… (en degrés).
    if (params.get("vue") === "carrefour") ouvrirCarrefour();
    if (params.get("entrer")) appliquer(`${params.get("entrer")}.enter`);
    if (params.get("vue") === "points") {
      await entrerEnPoints();
      if (params.get("zoom")) {
        zone().dispatchEvent(new WheelEvent("wheel", {
          deltaY: -Math.log2(Number(params.get("zoom"))) / 0.003,
          clientX: Number(params.get("x") ?? innerWidth / 2), clientY: Number(params.get("y") ?? innerHeight / 2), cancelable: true,
        }));
      }
      mosaique_pivoter(Number(params.get("lacet") ?? 0) * Math.PI / 180, Number(params.get("tangage") ?? 0) * Math.PI / 180);
      reveillerLeSuivi();
    }
  } catch (e) {
    const pre = document.body.appendChild(document.createElement("pre"));
    pre.id = "erreur";
    pre.textContent = (e?.message === "le moteur n'a pas pu démarrer" ? "" : "Le moteur a refusé ce fichier : " + (e?.message ?? e));
    if (!pre.textContent) pre.remove();
  }
