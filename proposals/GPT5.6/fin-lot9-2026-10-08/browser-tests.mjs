import { mkdtempSync,writeFileSync,readFileSync,readdirSync,rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
export function capabilityTests({engine,phone,pause}) {
 if(phone)return [];
 const lesson=n=>"/exemples/lecons/"+({116:"116-importer-et-exporter",117:"117-appareil-sur-permission",118:"118-notifications-locales",119:"119-une-page-hors-ligne"}[n])+".holo";
 const ready=async(p,n)=>{await p.open(lesson(n));if(!await p.until("window.__holoStarted"))throw Error("moteur absent");};
 const status=(name)=>"document.querySelector('[data-name="+name+"] [data-capability-status]').textContent";
 const choose=async(p,b,file)=>{
  const found=await b.send("DOM.getDocument",{depth:-1,pierce:true});
  const match=await b.send("DOM.querySelector",{nodeId:found.result.root.nodeId,selector:"input[data-holo-import]"});
  if(!match.result.nodeId)throw Error("sélecteur de fichier absent");
  await b.send("DOM.setFileInputFiles",{nodeId:match.result.nodeId,files:[file]});
 };
 return [
 ["lot9 : Rust release",async()=>{
  const r=spawnSync("cargo",["test","--release","--locked"],{cwd:engine,encoding:"utf8",timeout:300000,maxBuffer:4*1024*1024});
  const summary=(r.stdout||"").split("\n").filter(l=>l.startsWith("test result:")).join(" ; ");
  return [r.status===0,summary||r.stderr?.slice(-1800)||"aucun résultat"];
 }],
 ["lot9 : importer et exporter, limites, import atomique",async(p,b)=>{
  const folder=mkdtempSync(join(tmpdir(),"holo-transfer-"));
  try{
   await ready(p,116);
   await b.send("Page.setInterceptFileChooserDialog",{enabled:true});
   await b.send("Browser.setDownloadBehavior",{behavior:"allow",downloadPath:folder});
   await p.click("[data-name=Export]");
   if(!await p.until(status("File")+".includes('Terminé')"))throw Error("export non terminé");
   for(let i=0;i<50&&!readdirSync(folder).includes("notes.json");i++)await pause(100);
   const exported=JSON.parse(readFileSync(join(folder,"notes.json"),"utf8"));
   const file=join(folder,"incoming.json");writeFileSync(file,JSON.stringify({note:"Importé",notes:["Alpha","Beta"]}));
   await p.click("[data-name=Import]");await p.until("document.querySelector('input[data-holo-import]')");await choose(p,b,file);
   const imported=await p.until("document.querySelector('input[data-bind=note]').value==='Importé' && document.querySelectorAll('.holo-line').length===2");
   writeFileSync(file,JSON.stringify({note:"Piège",notes:[],admin:1}));
   await p.click("[data-name=Import]");await p.until("document.querySelector('input[data-holo-import]')");await choose(p,b,file);
   const refused=await p.until(status("File")+".includes('valeur') || "+status("File")+".includes('annoncées')");
   const intact=await p.value("document.querySelector('input[data-bind=note]').value==='Importé' && document.querySelectorAll('.holo-line').length===2");
   writeFileSync(file," ".repeat(65537));
   await p.click("[data-name=Import]");await p.until("document.querySelector('input[data-holo-import]')");await choose(p,b,file);
   const limited=await p.until(status("File")+".includes('64 Ko')");
   return [exported.note==="Bonjour"&&exported.notes.length===1&&imported&&refused&&intact&&limited&&await p.value("document.getElementById('page').innerText.includes('Transferts terminés : 2')"),JSON.stringify({exported,imported,refused,intact,limited,dom:await p.value("({note:document.querySelector('input[data-bind=note]').value,lines:document.querySelectorAll('.holo-line').length,text:document.getElementById('page').innerText})")})];
  }finally{await b.send("Page.setInterceptFileChooserDialog",{enabled:false});await b.send("Browser.setDownloadBehavior",{behavior:"default"});rmSync(folder,{recursive:true,force:true});}
 }],
 ["lot9 : presse-papiers Chrome, écriture puis lecture",async(p,b)=>{
  await b.send("Browser.grantPermissions",{permissions:["clipboardReadWrite","clipboardSanitizedWrite"]});
  try{
   await ready(p,117);await p.click("[data-name=Write]");
   const written=await p.until(status("Clipboard")+".includes('Terminé')");
   await p.value("document.querySelector('input[data-bind=copied]').value=''");
   await p.value("document.querySelector('input[data-bind=copied]').dispatchEvent(new Event('input',{bubbles:true}))");
   await p.click("[data-name=Read]");
   const read=await p.until("document.querySelector('input[data-bind=copied]').value==='Bonjour'");
   return [written&&read,"API Chrome réelle, presse-papiers de la machine de CI ; aucun téléphone"];
  }finally{await b.send("Browser.resetPermissions");}
 }],
 ["lot9 : position via API Chrome et refus de permission",async(p,b)=>{
  await b.send("Browser.grantPermissions",{permissions:["geolocation"]});
  await b.send("Emulation.setGeolocationOverride",{latitude:-4.3,longitude:15.3,accuracy:20});
  try{
   await ready(p,117);await p.click("[data-name=Locate]");
   const located=await p.until("document.getElementById('page').innerText.includes('-4.3')");
   await p.value("navigator.geolocation.getCurrentPosition=(yes,no)=>no({message:'Permission simulée refusée'})");
   await p.click("[data-name=Locate]");const denied=await p.until(status("Position")+".includes('refusée')");
   return [located&&denied&&await p.value("document.getElementById('page').innerText.includes('-4.3')"),"position négative fournie par DevTools ; refus simulé, dernière valeur conservée ; aucune mesure GPS réelle"];
  }finally{await b.send("Emulation.clearGeolocationOverride");await b.send("Browser.resetPermissions");}
 }],
 ["lot9 : caméra et microphone, arrêt des pistes (appareils simulés)",async(p)=>{
  await ready(p,117);
  await p.value("(()=>{window.__tracks=[];navigator.mediaDevices.getUserMedia=async c=>{const canvas=document.createElement('canvas');canvas.width=16;canvas.height=16;canvas.getContext('2d').fillRect(0,0,16,16);const stream=canvas.captureStream(1);window.__tracks.push(...stream.getTracks());return stream;};})()");
  await p.click("[data-name=StartCamera]");
  const active=await p.until("document.querySelector('[data-name=Camera] video')?.srcObject?.active");
  await p.click("[data-name=StopCamera]");
  const stopped=await p.value("window.__tracks.every(t=>t.readyState==='ended')");
  await p.click("[data-name=StartMic]");
  const mic=await p.until(status("Microphone")+".includes('actif')");
  await p.click("[data-name=StopMic]");
  const all=await p.value("window.__tracks.every(t=>t.readyState==='ended')");
  await p.value("navigator.mediaDevices.getUserMedia=()=>new Promise(ok=>{window.__grant=ok})");
  await p.click("[data-name=StartCamera]");await p.click("[data-name=StopCamera]");
  await p.value("(()=>{const c=document.createElement('canvas');const s=c.captureStream(1);window.__tracks.push(...s.getTracks());window.__grant(s);})()");
  await pause(200);const late=await p.value("window.__tracks.every(t=>t.readyState==='ended') && !document.querySelector('[data-name=Camera] video')");
  return [active&&stopped&&mic&&all&&late,"flux de canvas simulant l'appareil ; toutes les pistes arrêtées par le bouton, aussi après une permission tardive ; aucun matériel réel"];
 }],
 ["lot9 : notification locale autorisée puis refusée (API simulée)",async(p,b)=>{
  await ready(p,118);
  await p.value("(()=>{window.__notifications=[];Object.defineProperty(Notification,'permission',{configurable:true,get:()=> 'granted'});ServiceWorkerRegistration.prototype.showNotification=async function(title,options){window.__notifications.push({title,options});};})()");
  await p.click("[data-name=Notify]");await p.click("[data-name=Cancel]");await pause(3200);
  const cancelled=await p.value("window.__notifications.length===0");
  await p.click("[data-name=Notify]");
  const sent=await p.until("window.__notifications.length===1");
  const title=await p.value("window.__notifications[0]?.title");
  await p.value("Object.defineProperty(Notification,'permission',{configurable:true,get:()=> 'denied'})");
  await p.click("[data-name=Notify]");
  const denied=await p.until(status("Reminder")+".includes('refusée')");
  return [cancelled&&sent&&title==="Pause"&&denied,"service worker réel ; affichage système simulé ; permission refusée annoncée sans planter la page"];
 }],
 ["lot9 : copie hors-ligne, rechargement réel sans réseau et effacement",async(p,b)=>{
  await b.send("Network.enable");
  try{
   await ready(p,119);await p.click("[data-name=Save]");
   if(!await p.until(status("Copy")+".includes('prête')"))throw Error("copie non prête : "+await p.value(status("Copy")));
   await b.send("Network.emulateNetworkConditions",{offline:true,latency:0,downloadThroughput:0,uploadThroughput:0});
   await p.open(lesson(119));const readable=await p.until("window.__holoStarted && document.getElementById('page').innerText.includes('Mon carnet')");
   await p.click("[data-name=Add]");const changed=await p.until("document.getElementById('page').innerText.includes('Compteur local : 1')");
   await p.click("[data-name=Remove]");const removed=await p.until(status("Copy")+".includes('effacée')");
   const empty=await p.value("caches.keys().then(keys=>keys.filter(k=>k.startsWith('holo-offline-v1-')).length===0)");
   return [readable&&changed&&removed&&empty,"page, source, JS et WASM rechargés sans réseau ; compteur local ; cache effacé"];
  }finally{await b.send("Network.emulateNetworkConditions",{offline:false,latency:0,downloadThroughput:-1,uploadThroughput:-1});}
 }],
 ];
}
