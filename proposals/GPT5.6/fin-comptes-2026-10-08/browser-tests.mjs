// Vrais formulaires de compte et base locale ; un QR est relu par un décodeur indépendant.
import { DatabaseSync } from "node:sqlite";
import { readFileSync,existsSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
export function accountDebtTests({engine,phone,page,startHoloServe,pause,totp,stepNow}) {
 if(phone)return [];
 const password="une phrase locale pour mes comptes";
 const check=(ok,t)=>{if(!ok)throw Error(t);};
 async function button(q,selector){for(let i=0;i<80;i++){if(await q.value("document.activeElement?.matches("+JSON.stringify(selector)+")")){await q.key("Enter","Enter",13,String.fromCharCode(13));return;}await q.key("Tab","Tab",9);}throw Error("bouton inaccessible au clavier : "+selector);}
 async function create(q,name){await q.open("/account/signup",200);await q.type("#name",name);await q.type("#password",password);await q.type("#again",password);await button(q,'main form button[type="submit"]');check(await q.until("location.pathname==='/account'"),"compte non créé");}
 async function signin(q,name){await q.open("/account/signin",200);await q.type("#name",name);await q.type("#password",password);await button(q,'main form button[type="submit"]');check(await q.until("location.pathname==='/account/code'"),"code non demandé");}
 let decoder;
 async function decode(q){
  if(!decoder){const r=await fetch("https://unpkg.com/jsqr@1.4.0/dist/jsQR.js",{signal:AbortSignal.timeout(15000)});check(r.ok,"décodeur QR de test introuvable");decoder=await r.text();check(decoder.length<400000,"décodeur trop lourd");}
  await q.value(decoder+";typeof jsQR");
  // Reconstituer les pixels de l'image SVG réellement envoyée, marge blanche comprise.
  return q.value("(()=>{const s=document.getElementById('setup-qr'),size=s.viewBox.baseVal.width,scale=5,c=document.createElement('canvas');c.width=c.height=size*scale;const x=c.getContext('2d');x.fillStyle='white';x.fillRect(0,0,c.width,c.height);x.fillStyle='black';for(const m of s.querySelector('path').getAttribute('d').matchAll(/M(\\d+) (\\d+)h1v1h-1z/g))x.fillRect(Number(m[1])*scale,Number(m[2])*scale,scale,scale);const p=x.getImageData(0,0,c.width,c.height);return jsQR(p.data,p.width,p.height)?.data;})()");
 }
 return [
 ["comptes : QR local relu, secours à usage unique, avec et sans JS",async(_,b)=>{
  const served=await startHoloServe(["104-se-connecter.holo"]),q=page(b,served.base);
  try{
   for(const scripts of [false,true]){
    await b.send("Network.clearBrowserCookies");await b.send("Emulation.setScriptExecutionDisabled",{value:!scripts});
    const name=scripts?"RescueWith":"RescueWithout";await create(q,name);
    await button(q,'form[action="/account/code/setup"] button');check(await q.until("location.pathname==='/account/code/setup'"),"configuration absente");
    const link=await q.value("document.querySelector('a[href^=otpauth]')?.getAttribute('href')");
    const decoded=await decode(q);check(decoded===link,"QR illisible ou clé différente");
    const key=(await q.value("document.getElementById('key').textContent")).replaceAll(" ","");
    await q.type("#code",totp(key,stepNow()));await button(q,'main form button[type="submit"]');
    check(await q.until("document.querySelectorAll('[data-recovery]').length===10"),"dix codes absents");
    const clear=await q.value("[...document.querySelectorAll('[data-recovery]')].map(e=>e.textContent)");
    const db=new DatabaseSync(join(served.folder,"holo-data","site.sqlite"));
    try{const prints=db.prepare("SELECT fingerprint FROM recoveries").all();check(prints.length>=10 && prints.every(p=>!clear.includes(p.fingerprint)),"codes en clair dans la base");}finally{db.close();}
    await q.open("/account",200);check(!await q.value("document.querySelector('[data-recovery]')"),"secours montré à nouveau");
    await button(q,'form[action="/account/signout"] button');check(await q.until("location.pathname==='/account/signin'"),"déconnexion absente");
    await signin(q,name);await q.type("#code",clear[0]);await button(q,'main form button[type="submit"]');check(await q.until("location.pathname==='/account'"),"secours refusé la première fois");
    await button(q,'form[action="/account/signout"] button');await q.until("location.pathname==='/account/signin'");
    await signin(q,name);await q.type("#code",clear[0]);await button(q,'main form button[type="submit"]');
    check(await q.until("document.querySelector('[role=alert]')?.textContent.includes('ne va pas')"),"secours réutilisé accepté");
   }
   return [true,"Tab/Entrée ; QR SVG du serveur décodé par jsQR ; dix secours montrés une fois ; premier accepté, réemploi refusé ; JS activé et coupé"];
  }finally{await b.send("Emulation.setScriptExecutionDisabled",{value:false});served.stop();}
 }],
 ["comptes : effacement confirmé, session, panier, messages et sauvegarde",async(_,b)=>{
  const served=await startHoloServe(["104-se-connecter.holo","106-le-panier-qui-suit-le-compte.holo"]),q=page(b,served.base);
  try{
   await b.send("Network.clearBrowserCookies");await create(q,"EraseMe");
   await q.open("/106-le-panier-qui-suit-le-compte.holo",200);await q.click("[data-name=Add]");check(await q.until("document.getElementById('page').innerText.includes('Dans le panier : 1')"),"panier absent");
   const db=new DatabaseSync(join(served.folder,"holo-data","site.sqlite"));let id,backup;
   try{id=db.prepare("SELECT id FROM accounts WHERE name='EraseMe'").get().id;db.prepare("INSERT INTO messages(received,page,form,submission,account) VALUES(1,'/x','X','{}',?)").run(id);}finally{db.close();}
   await pause(1100);
   const r=spawnSync(join(engine,"target","release",process.platform==="win32"?"holo.exe":"holo"),["backup",served.folder],{encoding:"utf8",timeout:10000});check(r.status===0,"sauvegarde impossible");backup=r.stdout.trim();check(backup&&existsSync(backup),"chemin de sauvegarde absent : "+r.stdout);
   await q.open("/account/delete",200);await q.type("#password","wrong");await q.type("#confirm","EraseMe");await button(q,'main form button[type="submit"]');check(await q.until("document.querySelector('[role=alert]')"),"mauvais mot de passe non refusé");
   await q.type("#password",password);await q.type("#confirm","EraseMe");await button(q,'main form button[type="submit"]');check(await q.until("document.body.innerText.includes('Compte effacé')"),"compte non effacé");
   for(const path of [join(served.folder,"holo-data","site.sqlite"),backup]){const base=new DatabaseSync(path);try{for(const query of ["SELECT COUNT(*) n FROM accounts WHERE id=?","SELECT COUNT(*) n FROM sessions WHERE account=?","SELECT COUNT(*) n FROM messages WHERE account=?"])check(base.prepare(query).get(id).n===0,"donnée encore gardée : "+query);}finally{base.close();}}
   await q.open("/account",200);check(await q.value("location.pathname==='/account/signin'"),"ancienne session encore utilisable");
   return [true,"mot de passe incorrect refusé ; confirmation au clavier ; compte, session et messages retirés de la base et de sa sauvegarde ; tests Rust couvrent aussi le fichier privé et les autres comptes"];
  }finally{served.stop();}
 }],
 ["comptes : frein par IP malgré des noms différents",async(_,b)=>{
  const served=await startHoloServe(["104-se-connecter.holo"]),q=page(b,served.base);
  try{await q.open("/account/signup",200);const states=await q.value("(async()=>{const out=[];for(let i=0;i<31;i++){const r=await fetch('/account/signup',{method:'POST',headers:{'content-type':'application/x-www-form-urlencoded'},body:'name=Many'+i+'&password=x&again=x'});out.push([r.status,r.headers.get('retry-after')]);}return out;})()");
   check(states.slice(0,30).every(([s])=>s===422)&&states[30][0]===429&&states[30][1]==="60","limite IP incorrecte : "+JSON.stringify(states));return [true,"30 formulaires invalides sous 30 noms, puis HTTP 429 et Retry-After : 60 ; vraie IP du pair TCP"];
  }finally{served.stop();}
 }],
 ];
}
