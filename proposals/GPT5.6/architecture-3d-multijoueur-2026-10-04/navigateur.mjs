// Sondes de la démo locale dans Chrome sans fenêtre, profil temporaire vide.
// Pas de mesure de fluidité ou de téléphone. Node 22+, serveur sur localhost:8080.
import { spawn } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const sortie = process.argv[2] ?? join(tmpdir(), 'holo-revue-navigateur.json');
const dossier = fileURLToPath(new URL('.', import.meta.url));
const base = process.env.HOLO_URL ?? 'http://localhost:8080';
const port = 9800 + Math.floor(Math.random() * 500);
const pause = ms => new Promise(r => setTimeout(r, ms));
const chrome = spawn(process.env.CHROME ?? 'C:/Program Files/Google/Chrome/Application/chrome.exe', [
  '--headless=new', '--enable-unsafe-swiftshader', '--no-first-run', '--disable-lcd-text',
  `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(tmpdir(), 'holo-revue-'))}`, 'about:blank',
], { stdio: 'ignore', windowsHide: true });
let socket;
try {
  let tab;
  for (let n = 0; n < 50 && !tab; n++) {
    await pause(200);
    try { tab = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find(t => t.type === 'page'); } catch {}
  }
  assert(tab, 'Chrome non démarré');
  socket = new WebSocket(tab.webSocketDebuggerUrl);
  await new Promise((ok, fail) => { socket.onopen = ok; socket.onerror = fail; });
  let id = 0;
  const pending = new Map();
  socket.onmessage = e => { const r = JSON.parse(e.data); pending.get(r.id)?.(r); pending.delete(r.id); };
  const send = (method, params = {}) => new Promise((ok, fail) => {
    const n = ++id;
    const deadline = setTimeout(() => { pending.delete(n); fail(new Error(`Délai : ${method}`)); }, 20000);
    pending.set(n, r => { clearTimeout(deadline); r.error ? fail(new Error(JSON.stringify(r.error))) : ok(r.result); });
    socket.send(JSON.stringify({ id: n, method, params }));
  });
  const evaluer = async expression => {
    const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
    assert(!r.exceptionDetails, JSON.stringify(r.exceptionDetails));
    return r.result.value;
  };
  const erreurs = [];
  // On écoute les erreurs JavaScript, sans confondre une ressource facultative avec le jeu.
  const ancien = socket.onmessage;
  socket.onmessage = e => { const r = JSON.parse(e.data); if (r.method === 'Runtime.exceptionThrown') erreurs.push(r.params.exceptionDetails.text); ancien(e); };
  await send('Runtime.enable');
  await send('Emulation.setDeviceMetricsOverride', { width: 360, height: 777, deviceScaleFactor: 1, mobile: false });
  const wasm = Buffer.from(await (await fetch(`${base}/pkg/holo_moteur_bg.wasm`)).arrayBuffer());
  const local = readFileSync(new URL('../../../moteur/web/pkg/holo_moteur_bg.wasm', import.meta.url));
  assert(wasm.equals(local), 'WASM servi différent du build local');
  const rapport = { base, chrome: await send('Browser.getVersion'), viewport: '360x777', wasm: { octets: wasm.length, sha256: createHash('sha256').update(wasm).digest('hex') }, erreurs, resultats: [] };
  await send('Page.navigate', { url: `${base}/exemples/jeu/panier.holo` });
  for (let n = 0; n < 60; n++) {
    if (await evaluer(`!!document.querySelector('[data-name="Play"]') && !document.querySelector('[data-name="Play"]').closest('[hidden]')`)) break;
    await pause(200);
  }
  // Attendre aussi le chargement du moteur : le bouton est déjà fabriqué côté serveur.
  await pause(1500);
  const lire = `(() => ({score: [...document.querySelectorAll('[data-state="score"]')].map(x=>x.textContent), apple:document.querySelector('[data-name="Apple"]')?.closest('.holo-place').style.getPropertyValue('--y'), basket:document.querySelector('[data-name="Basket"]')?.closest('.holo-place').style.getPropertyValue('--x'), board:document.querySelector('.holo-Board')?.clientWidth, playVisible: !document.querySelector('[data-name="Play"]')?.closest('[hidden]')}))()`;
  const avant = await evaluer(lire);
  await pause(1100);
  const repos = await evaluer(lire);
  assert.equal(repos.apple, avant.apple, 'La pomme bouge avant Play');
  assert(repos.score.every(x => x === '0'), JSON.stringify(repos));
  await evaluer(`document.querySelector('[data-name="Play"]').click()`);
  await pause(350);
  const lance = await evaluer(lire);
  assert(!lance.playVisible && Number(lance.apple) > 0, JSON.stringify(lance));
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'ArrowLeft', code: 'ArrowLeft', windowsVirtualKeyCode: 37 });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'ArrowLeft', code: 'ArrowLeft', windowsVirtualKeyCode: 37 });
  const gauche = await evaluer(lire);
  assert.equal(Number(gauche.basket), Number(lance.basket) - 8, JSON.stringify(gauche));
  await pause(3150);
  const prise = await evaluer(lire);
  assert(prise.score.some(x => Number(x) >= 1), JSON.stringify(prise));
  rapport.resultats.push({ page: 'panier.holo', avant, repos, lance, gauche, prise });
  await send('Page.captureScreenshot', { format: 'png' }).then(r => writeFileSync(join(dossier, 'target', 'panier.png'), Buffer.from(r.data, 'base64')));
  await send('Page.navigate', { url: `${base}/exemples/lecons/01-page.holo` });
  await pause(1500);
  const lecon = await evaluer(`({title:document.title,h1:document.querySelector('h1')?.textContent,paragraphes:[...document.querySelectorAll('.holo-P')].map(x=>x.textContent),hr:!!document.querySelector('hr')})`);
  assert.equal(lecon.h1, 'Ma première page');
  assert(lecon.hr && lecon.paragraphes.includes('En voici un second.'), JSON.stringify(lecon));
  rapport.resultats.push({ page: '01-page.holo', ...lecon });
  assert.deepEqual(erreurs, []);
  writeFileSync(sortie, JSON.stringify(rapport, null, 2) + '\n');
  console.log(JSON.stringify(rapport, null, 2));
} finally {
  socket?.close();
  chrome.kill();
}
