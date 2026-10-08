// Parcours complets du web, avec le vrai serveur Rust et le Chrome de la suite.
// Les largeurs émulées et axe-core ne remplacent pas un téléphone ni une personne au TalkBack.
import { cpSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

export function webTests({ repo, engine, phone, page, startHoloServe, startChrome, pause }) {
  if (phone) return []; // Le serveur temporaire de CI n'est pas celui du téléphone.
  const binary = join(engine, "target", "release", process.platform === "win32" ? "holo.exe" : "holo");
  const has = (text) => 'document.getElementById("page").innerText.includes(' + JSON.stringify(text) + ')';
  const state = (name, value) => 'document.querySelector(\'#page [data-state="' + name + '"]\')?.textContent === ' + JSON.stringify(value);
  const check = (ok, description) => { if (!ok) throw new Error(description); };
  // Tab est le vrai geste du navigateur ; aucun focus DOM n'est utilisé pour trouver le bouton.
  async function tabTo(q, selector) {
    for (let i = 0; i < 120; i++) {
      if (await q.value('document.activeElement?.matches(' + JSON.stringify(selector) + ')')) return;
      await q.key("Tab", "Tab", 9);
    }
    throw new Error("le clavier n'atteint pas : " + selector);
  }
  async function activate(q, selector) {
    await tabTo(q, selector);
    await q.key("Enter", "Enter", 13, "\r");
  }
  async function text(q, bind, value) {
    await q.value('document.querySelector(\'[data-bind="' + bind + '"]\').select()');
    await q.type('[data-bind="' + bind + '"]', value);
  }
  async function site(browser, run) {
    const served = await startHoloServe([]);
    cpSync(join(repo, "exemples", "parcours"), served.folder, { recursive: true });
    for (const name of ["116-importer-et-exporter","117-appareil-sur-permission","118-notifications-locales","119-une-page-hors-ligne"]) cpSync(join(repo,"exemples","lecons",name+".holo"),join(served.folder,name+".holo"));
    const q = page(browser, served.base);
    try {
      await browser.send("Network.clearBrowserCookies");
      return await run(q, served);
    } finally {
      await browser.send("Fetch.disable").catch(() => {});
      browser.on("Fetch.requestPaused", null);
      await browser.send("Emulation.setScriptExecutionDisabled", { value: false });
      await browser.send("Emulation.clearDeviceMetricsOverride");
      await browser.send("Emulation.setEmulatedMedia", { media: "", features: [] });
      served.stop();
    }
  }
  const messages = (folder) => {
    const result = spawnSync(binary, ["messages", folder], { encoding: "utf8", timeout: 10000 });
    check(result.status === 0, "holo messages : " + result.stderr);
    return result.stdout.trim().split("\n").filter(Boolean);
  };
  let axeSource;
  async function axe(q) {
    if (!axeSource) {
      const response = await fetch("https://unpkg.com/axe-core@4.10.3/axe.min.js", { signal: AbortSignal.timeout(15000) });
      check(response.ok, "axe-core introuvable : " + response.status);
      const bytes = await response.arrayBuffer();
      check(bytes.byteLength < 1000000, "bibliothèque de test trop lourde");
      axeSource = new TextDecoder().decode(bytes);
    }
    await q.value(axeSource + "\n;window.axe.version");
    check(await q.value('window.axe.version === "4.10.3"'), "version axe-core inattendue");
    return q.value('window.axe.run(document, {resultTypes:["violations"]}).then(r => r.violations.map(v => ({id:v.id, nodes:v.nodes.map(n=>n.html.slice(0,160))})))');
  }
  return [
    ["parcours : tous les tests Rust en mode release", async () => {
      // Le serveur distribué est optimisé : éprouver aussi les invariants sous ce profil.
      // Le même dossier et le même Cargo.lock sont utilisés, sans dépendance ajoutée.
      const result = spawnSync("cargo", ["test", "--release", "--locked"], {
        cwd: engine, encoding: "utf8", timeout: 300000, maxBuffer: 4 * 1024 * 1024,
      });
      check(result.status === 0, "cargo test --release --locked : " + (result.error?.message ?? "") + "\n" + result.stdout + result.stderr);
      const summary = result.stdout.match(/test result: ok\. (\d+) passed; 0 failed/);
      check(summary && Number(summary[1]) >= 180, "résumé des tests Rust absent ou incomplet");
      return [true, summary[1] + " tests Rust optimisés réussis ; cargo test --release --locked"];
    }],
    ["parcours : les pages et le catalogue se vérifient avec holo", async () => {
      const folder = join(repo, "exemples", "parcours");
      const paths = [];
      const visit = (current) => { for (const entry of readdirSync(current, { withFileTypes: true })) {
        if (entry.isDirectory()) visit(join(current, entry.name));
        else if (entry.name.endsWith(".holo")) paths.push(join(current, entry.name));
      }};
      visit(folder);
      paths.push(join(repo, "exemples", "lecons", "109-un-catalogue-page-par-page.holo"));
      for (const path of paths) {
        const result = spawnSync(binary, ["check", path], { encoding: "utf8", timeout: 10000 });
        check(result.status === 0, path + " : " + result.stdout + result.stderr);
      }
      const products = JSON.parse(readFileSync(join(folder, "catalogue.json"), "utf8")).products;
      check(products.length === 200 && new Set(products.map(p=>p.id)).size === 200, "le catalogue n'a pas 200 clés distinctes");
      return [true, paths.length + " fichiers .holo acceptés ; 200 produits, 200 clés"];
    }],
    ["parcours 1 : 200 produits, recherche, catégories, pages et HTML sans JavaScript", async (_, b) => site(b, async (q, served) => {
      // Accept text/html : le serveur doit fabriquer les premières cartes sans exécuter de script.
      const response = await fetch(served.base + "/catalogue.holo", { headers: { accept: "text/html" } });
      const html = await response.text();
      check(response.status === 200 && html.includes("Produit 200") && html.includes('data-state="matching">200'), "catalogue non rempli dans le HTML : " + html.slice(-1500));
      await b.send("Emulation.setScriptExecutionDisabled", { value: true });
      await q.open("/catalogue.holo", 200);
      check(await q.value(state("matching", "200")), "le serveur a perdu des produits");
      await activate(q, '[data-name="Next"]');
      check(await q.until(has("Produit 180") + ' && !' + has("Produit 200"), 10000), "la deuxième page ne suit pas le tri");
      await activate(q, '[data-name="Previous"]');
      check(await q.until(has("Produit 200"), 10000), "le retour sans JavaScript a échoué");
      await b.send("Emulation.setScriptExecutionDisabled", { value: false });
      await q.open("/catalogue.holo", 200);
      await text(q, "search", "Produit 200");
      check(await q.until(state("matching", "1") + ' && ' + has("Produit 200")), "la recherche ne trouve pas le dernier produit");
      await activate(q, '[data-name="All"]');
      check(await q.until(state("matching", "200")), "le filtre vide ne rend pas les 200 produits");
      await tabTo(q, 'select[data-bind="chosen"]');
      await q.key("ArrowDown", "ArrowDown", 40);
      check(await q.until(state("matching", "100") + ' && ' + has("Produit 199")), "le filtre de catégorie ne trouve pas 100 livres");
      await activate(q, '[data-name="All"]');
      for (let i = 0; i < 9; i++) {
        await activate(q, '[data-name="Next"]');
        check(await q.until(has("Produit " + String(200 - (i + 1) * 20).padStart(3, "0")), 5000), "page " + (i + 2) + " incorrecte");
      }
      check(await q.value('document.querySelectorAll(".holo-line").length === 20 && !document.querySelector(\'[data-name="Next"]\')?.getClientRects().length'), "la dernière page déborde ou propose une page vide");
      check(b.errors.length === 0, b.errors.join(" | "));
      return [true, "200 produits dans le HTML ; page suivante/précédente sans JS ; produit 200 retrouvé ; 100 livres ; dix pages triées au clavier"];
    })],
    ["parcours 2 : fiche, panier et TVA exacte avec et sans JavaScript", async (_, b) => site(b, async (q) => {
      for (const scripts of [false, true]) {
        await b.send("Network.clearBrowserCookies");
        await b.send("Emulation.setScriptExecutionDisabled", { value: !scripts });
        await q.open("/fiche.holo", 200);
        for (let i = 1; i <= 4; i++) {
          await activate(q, '[data-name="Add"]');
          check(await q.until(state("quantity", String(i)) + ' && ' + state("gross", (i * 15).toFixed(2).replace(".", ",")), 10000), "total incorrect, quantité " + i + ", JS=" + scripts + " : " + await q.text());
        }
        check(await q.value(state("net", "50,00") + ' && ' + state("vat", "10,00")), "TVA ou total HT incorrect à quatre unités");
      await activate(q, '[data-name="Remove"]');
        check(await q.until(state("quantity", "3") + ' && ' + state("gross", "45,00"), 10000), "le retrait ne garde pas les centimes");
      }
      return [true, "12,50 HT + 20 % = 15,00 TTC ; quatre unités : 60,00 ; retrait : 45,00 ; Tab/Entrée, JS activé et coupé"];
    })],
    ["parcours 3 : inscription, erreurs reliées et un seul message enregistré", async (_, b) => site(b, async (q, served) => {
      await q.open("/inscription.holo", 200);
      await activate(q, '[data-name="Send"]');
      check(await q.until('document.querySelectorAll(".holo-error").length === 3', 10000), "les trois champs vides ne sont pas signalés");
      const linked = await q.value('Array.from(document.querySelectorAll("[aria-invalid=true]")).every(e=>e.getAttribute("aria-describedby")?.split(" ").some(id=>document.getElementById(id)?.classList.contains("holo-error")))');
      check(linked, "erreur non reliée au champ");
      const tree = await b.send("Accessibility.getFullAXTree");
      check(tree.result.nodes.some(n => /obligatoire/i.test(n.name?.value ?? "")), "erreurs absentes de l'arbre d'accessibilité");
      await text(q, "name", "Ada");
      await text(q, "email", "ada@example.test");
      await tabTo(q, '[data-bind="consent"]');
      await q.key(" ", "Space", 32, " ");
      // Le vrai POST est retenu brièvement : la seconde Entrée arrive pendant l'envoi,
      // et non après sa confirmation. La réponse reste celle du serveur Rust.
      b.on("Fetch.requestPaused", ({requestId, request}) => {
        if (request.method === "POST") setTimeout(() => b.send("Fetch.continueRequest", {requestId}), 800);
        else b.send("Fetch.continueRequest", {requestId});
      });
      await b.send("Fetch.enable", {patterns:[{urlPattern:"*inscription.holo*",requestStage:"Request"}]});
      await activate(q, '[data-name="Send"]');
      await q.key("Enter", "Enter", 13, "\r");
      check(await q.until(has("Ta demande est arrivée."), 15000), "pas de confirmation");
      check(messages(served.folder).length === 1, "l'envoi rapproché a créé plusieurs messages");
      const forged = await fetch(served.base + "/inscription.holo", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({form:"Registration", values:{name:"",email:"faux",consent:0}}) });
      check(forged.status === 422 && messages(served.folder).length === 1, "le serveur a accepté la demande invalide");
      return [true, "trois erreurs reliées et présentes dans l'arbre AX ; Entrée rapprochée : un message ; données invalides refusées par HTTP 422"];
    })],
    ["parcours 4 : contact, vraie image envoyée et octets conservés", async (_, b) => site(b, async (q, served) => {
      const png = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=", "base64");
      const upload = join(served.folder, "upload.png");
      writeFileSync(upload, png);
      await q.open("/contact.holo", 200);
      await text(q, "name", "Ada");
      await text(q, "message", "Bonjour, voici une image de recette.");
      const document = await b.send("DOM.getDocument");
      const field = await b.send("DOM.querySelector", {nodeId: document.result.root.nodeId, selector:'input[type="file"]'});
      await b.send("DOM.setFileInputFiles", {nodeId: field.result.nodeId, files:[upload]});
      await activate(q, '[data-name="Send"]');
      check(await q.until(has("Ton message et ton image sont arrivés."), 15000), "pas de confirmation après la photo");
      const kept = messages(served.folder);
      check(kept.length === 1 && kept[0].includes("upload.png"), "message et nom du fichier non conservés");
      const paths = [];
      const visit = (current) => { for (const entry of readdirSync(current, {withFileTypes:true})) {
        const next = join(current, entry.name); if (entry.isDirectory()) visit(next); else paths.push(next);
      }};
      visit(join(served.folder, "holo-data", "files"));
      check(paths.length === 1 && readFileSync(paths[0]).equals(png), "les octets de la photo ont changé");
      return [true, "PNG choisi par Chrome ; message, confirmation et fichier conservés ; mêmes octets dans le dossier privé"];
    })],
    ["parcours 5 : menu au clavier et mêmes actions à 360 et 1280 pixels", async (_, b) => site(b, async (q) => {
      let links;
      for (const width of [360, 1280]) {
        await b.send("Emulation.setDeviceMetricsOverride", {width,height:800,deviceScaleFactor:1,mobile:false});
        await q.open("/disposition.holo", 200);
        await activate(q, "summary");
        check(await q.value('document.querySelector("details").open'), "menu fermé à " + width);
        const now = await q.value('Array.from(document.querySelectorAll("#page nav a")).filter(e=>e.getClientRects().length).map(e=>e.textContent).join("|")');
        check(now === "Catalogue|Contact|Article" && (!links || links === now), "actions différentes à " + width + " : " + now);
        links = now;
        check(await q.value("document.documentElement.scrollWidth <= innerWidth + 1"), "débordement à " + width);
        await activate(q, '#page nav a[href$="catalogue.holo"]');
        check(await q.until('location.pathname === "/catalogue.holo" && document.readyState === "complete"', 10000), "lien du menu inaccessible au clavier");
      }
      return [true, "mêmes trois liens ; menu ouvert et catalogue rejoint par Tab/Entrée ; aucun débordement à 360 et 1280 pixels (émulation)"];
    })],
    ["parcours 6 : réservation vue en direct dans deux navigateurs isolés", async (_, b) => site(b, async (a, served) => {
      const other = await startChrome();
      const bob = page(other, served.base);
      try {
        await a.open("/reservation.holo", 200);
        await bob.open("/reservation.holo", 200);
        check(await a.until("window.__holoLive?.()") && await bob.until("window.__holoLive?.()"), "les deux pages n'écoutent pas le serveur");
        await bob.value("window.__stayed = true");
        await activate(a, '[data-name="Book"]');
        check(await a.until(has("Ta place est gardée."), 10000) && await bob.until(state("seats","1"), 10000), "la réservation n'arrive pas ailleurs");
        check(await bob.value("window.__stayed === true"), "la seconde page a été rechargée");
        await activate(bob, '[data-name="Book"]');
        check(await a.until(state("seats","0"), 10000) && await bob.until(has("Ta place est gardée."), 10000), "la seconde place est perdue");
        return [true, "deux profils Chrome distincts ; la réservation arrive en SSE sans rechargement ; seconde réservation au clavier ; zéro place"];
      } finally { other.stop(); }
    })],
    ["parcours 7 : le compte retrouve le panier après reconnexion", async (_, b) => site(b, async (q, served) => {
      const next = "/fiche.holo", password = "une phrase locale de recette";
      await b.send("Emulation.setScriptExecutionDisabled", {value:true});
      await q.open("/account/signup?next=" + next, 200);
      await q.type("#name", "Cleo"); await q.type("#password", password); await q.type("#again", password);
      await activate(q, 'main form button[type="submit"]');
      check(await q.until('location.pathname === "/fiche.holo" && document.readyState === "complete"', 10000), "compte non créé");
      for (const amount of [1,2]) {
        await activate(q, '[data-name="Add"]');
        check(await q.until(state("quantity",String(amount)),10000), "panier non enregistré");
      }
      await b.send("Network.clearBrowserCookies");
      await b.send("Storage.clearDataForOrigin", {origin:served.base,storageTypes:"cookies,local_storage"});
      await q.open("/fiche.holo",200);
      check(await q.value(state("quantity","0")), "le visiteur anonyme voit le panier d'un compte");
      await q.open("/account/signin?next=" + next,200);
      await q.type("#name","Cleo"); await q.type("#password",password);
      await activate(q, 'main form button[type="submit"]');
      check(await q.until(state("quantity","2") + ' && ' + state("gross","30,00"),10000), "le compte ne retrouve pas le total exact");
      return [true, "compte local ; deux livres et 30,00 TTC ; navigateur vidé : panier anonyme vide ; reconnexion : panier retrouvé, sans JavaScript"];
    })],
    ["parcours 8 et 9 : profil partageable, vidéo sous-titrée et impression", async (_, b) => site(b, async (q, served) => {
      await q.open("/profil/123",200);
      check(await q.value('document.querySelector("#page h1").textContent === "Profil 123"'), "le profil ne lit pas son adresse");
      check(await q.value('document.querySelector(\'meta[name="description"]\')?.content === "Un profil public de l\'atelier." && !!document.querySelector(\'meta[property="og:image"]\')?.content'), "description ou image de partage absente");
      await activate(q, '#page a[href$="index.holo"]');
      check(await q.until('location.pathname === "/index.holo"',5000), "le lien du profil ne remonte pas");
      await q.open("/article.holo",200);
      check(await q.value('document.querySelectorAll("#page h2").length === 13 && !!document.querySelector("#page aside")'), "sections et encadré non lisibles");
      const captions = await fetch(served.base + "/film.vtt").then(r=>r.text());
      check(captions.startsWith("WEBVTT") && captions.includes("mire de couleurs"), "sous-titres introuvables");
      await q.value('document.querySelector("video").play()');
      check(await q.until('document.querySelector("video").currentTime > 0 && document.querySelector("video track")?.kind === "captions" && document.querySelector("video").textTracks[0]?.cues?.length > 0',5000), "la vidéo ou ses sous-titres ne sont pas chargés");
      await b.send("Emulation.setEmulatedMedia", {media:"print"});
      check(await q.value('getComputedStyle(document.querySelector("#page nav")).display === "none"'), "le menu se voit à l'impression");
      const pdf = await b.send("Page.printToPDF", {printBackground:true});
      check(pdf.result?.data?.startsWith("JVBER"), "Chrome n'a pas fabriqué le document d'impression");
      return [true, "profil /123, description et image ; retour par lien relatif ; treize sections ; vidéo locale jouée et sous-titres chargés ; menu absent sur papier ; PDF généré"];
    })],
    ["parcours 10 : graphique au clavier et chiffres accessibles, avec et sans JS", async (_, b) => site(b, async(q)=>{
      for (const scripts of [false,true]) {
        await b.send("Network.clearBrowserCookies");
        await b.send("Emulation.setScriptExecutionDisabled",{value:!scripts});
        await q.open("/dashboard.holo",200);
        check(await q.until('document.querySelectorAll(".holo-Chart:first-of-type svg rect").length===3'),"trois ventes absentes");
        await text(q,"day","Jeudi");await text(q,"amount","180");await activate(q,'[data-name="AddSale"]');
        check(await q.until('document.querySelectorAll(".holo-Chart:first-of-type svg rect").length===4 && document.querySelectorAll(".holo-Chart:nth-of-type(2) svg path").length===4',10000),"vente absente du dessin, JS="+scripts);
        check(await q.value('document.querySelector(".holo-Chart table").textContent.includes("Jeudi180")'),"chiffres absents du tableau accessible");
        const ax=await b.send("Accessibility.getFullAXTree");
        check(ax.result.nodes.some(n=>n.role?.value==="table") && ax.result.nodes.some(n=>n.name?.value==="180"),"tableau absent de l'arbre AX");
        check(await q.value("document.documentElement.scrollWidth<=innerWidth+1"),"débordement du graphique");
      }
      return [true,"trois ventes puis quatre, barres et parts ; Tab/Entrée ; 180 dans le tableau et l'arbre AX ; serveur Rust, JS activé et coupé"];
    })],
    ["parcours : axe-core, clair/sombre, 360/1280 pixels", async (_, b) => site(b, async (q) => {
      const paths = ["/index.holo","/catalogue.holo","/fiche.holo","/inscription.holo","/contact.holo","/disposition.holo","/reservation.holo","/profil/123","/article.holo","/dashboard.holo","/116-importer-et-exporter.holo","/117-appareil-sur-permission.holo","/118-notifications-locales.holo","/119-une-page-hors-ligne.holo"];
      const faults = [];
      for (const width of [360,1280]) for (const dark of [false,true]) {
        await b.send("Emulation.setDeviceMetricsOverride",{width,height:800,deviceScaleFactor:1,mobile:false});
        await b.send("Emulation.setEmulatedMedia",{media:"",features:[{name:"prefers-color-scheme",value:dark?"dark":"light"}]});
        for (const path of paths) {
          await q.open(path,100);
          const violations = await axe(q);
          if (violations.length) faults.push(JSON.stringify({path,width,dark,violations}));
          if (!(await q.value("document.documentElement.scrollWidth <= innerWidth + 1"))) faults.push(path + " : débordement à " + width);
        }
      }
      return [faults.length === 0, faults.length ? faults.join("\n") : "axe-core 4.10.3 : 14 pages × 4 modes = 56 audits, zéro défaut ; aucun débordement ; ce n'est pas un essai humain TalkBack"];
    })],
  ];
}
