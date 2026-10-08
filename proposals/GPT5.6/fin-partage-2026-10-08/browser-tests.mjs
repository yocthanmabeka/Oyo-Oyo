// Partage réel : deux profils Chrome, attente tenue puis relâchée, SQLite du serveur.
import {readFileSync,writeFileSync} from "node:fs";
import {join} from "node:path";
import {DatabaseSync} from "node:sqlite";
export function sharingTests({engine,phone,page,startHoloServe,startChrome,pause}){
 if(phone)return[];
 const check=(ok,t)=>{if(!ok)throw Error(t);};
 const field="input[data-bind=note]",title="input[data-bind=title]";
 const text=async(q,selector,value)=>{await q.value("(()=>{const e=document.querySelector("+JSON.stringify(selector)+");e.value="+JSON.stringify(value)+";e.dispatchEvent(new Event('input',{bubbles:true}));})()");};
 const state=(name,value)=>"document.querySelector('[data-state="+name+"]')?.textContent==="+JSON.stringify(value);
 return [
 ["partage : liste simultanée et édition pendant la réponse",async(_,b)=>{
  const served=await startHoloServe(["102-une-liste-partagee.holo"]),other=await startChrome(),a=page(b,served.base),c=page(other,served.base);
  try{
   await a.open("/102-une-liste-partagee.holo",200);await c.open("/102-une-liste-partagee.holo",200);
   check(await a.until("window.__holoLive?.()")&&await c.until("window.__holoLive?.()"),"écoute absente");
   await text(a,field,"Premier");await text(c,field,"Second");
   await Promise.all([a.click("[data-name=Add]"),c.click("[data-name=Add]")]);
   check(await a.until("document.querySelectorAll('[data-repeat] p').length===2")&&await c.until("document.querySelectorAll('[data-repeat] p').length===2"),"un ajout concurrent perdu");
   // Tenir la réponse après sa lecture ; le serveur a répondu et le visiteur tape encore.
   await a.value("(()=>{const original=fetch;window.fetch=async(...args)=>{const r=await original(...args);if(args[1]?.method==='POST'){await new Promise(resolve=>window.__releaseShare=resolve);}return r;};})()");
   await text(a,field,"Envoyé");await a.click("[data-name=Add]");
   check(await a.until("typeof window.__releaseShare==='function'",10000),"requête non tenue");
   await text(a,field,"Prochaine phrase");await a.value("window.__releaseShare()");
   check(await a.until(state("sent","2"),10000),"réponse non appliquée");
   check(await a.value("document.querySelector("+JSON.stringify(field)+").value==='Prochaine phrase'"),"la réponse a effacé la nouvelle frappe");
   check(await c.until("document.getElementById('page').innerText.includes('Envoyé')"),"message non transmis");
   return[true,"deux profils Chrome, deux ajouts simultanés sans perte ; nouvelle frappe conservée après la réponse qui vide l'ancien champ"];
  }finally{other.stop();served.stop();}
 }],
 ["partage : brouillon privé, confirmation et même geste sans JS",async(_,b)=>{
  const served=await startHoloServe(["103-un-texte-partage-confirme.holo"]),other=await startChrome(),a=page(b,served.base),c=page(other,served.base);
  try{
   await a.open("/103-un-texte-partage-confirme.holo",200);await c.open("/103-un-texte-partage-confirme.holo",200);check(await a.until("window.__holoLive?.()")&&await c.until("window.__holoLive?.()"),"écoute absente");
   await text(c,title,"Mon brouillon");await c.value("document.querySelector('h1').focus()");
   await text(a,title,"Publié");await a.click("[data-name=Save]");
   check(await c.until(state("title","Publié"),10000),"publication absente");
   check(await c.value("document.querySelector("+JSON.stringify(title)+").value==='Mon brouillon'"),"SSE a remplacé le brouillon");
   await c.click("[data-name=Save]");check(await a.until(state("title","Mon brouillon"),10000),"brouillon non confirmé");
   await b.send("Emulation.setScriptExecutionDisabled",{value:true});await a.open("/103-un-texte-partage-confirme.holo",200);await text(a,title,"Sans JS");await a.click("[data-name=Save]");
   check(await a.until(state("title","Sans JS"),10000)&&await c.until(state("title","Sans JS"),10000),"confirmation sans JS perdue");
   return[true,"le texte affiché reste celui du serveur ; le champ garde son brouillon pendant une publication étrangère ; bouton confirme avec et sans JavaScript"];
  }finally{await b.send("Emulation.setScriptExecutionDisabled",{value:false});other.stop();served.stop();}
 }],
 ["partage : soixante touchers, puis refus sans écriture",async(_,b)=>{
  const served=await startHoloServe([]),q=page(b,served.base);
  try{
   writeFileSync(join(served.folder,"brake.holo"),'Page(shared: Shared(n: 0), children: [P("{n}"),Button(name:Go,text:"+")], rules:[On(Go.tap,effect:n.add(1))])');
   await q.open("/brake.holo",200);check(await q.until("window.__holoLive?.()"),"page non ouverte");
   const replies=await q.value("(async()=>{const r=[];for(let i=0;i<61;i++){const response=await fetch('/brake.holo',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({signal:'Go.tap',state:''})});r.push([response.status,response.headers.get('retry-after')]);}return r;})()");
   check(replies.slice(0,60).every(([s])=>s===200)&&replies[60][0]===429&&replies[60][1]==='60',"frein incorrect : "+JSON.stringify(replies));
   const db=new DatabaseSync(join(served.folder,"holo-data","site.sqlite"));try{check(db.prepare("SELECT value FROM shared WHERE name='n'").get().value==='60',"geste refusé écrit");}finally{db.close();}
   return[true,"60 vrais POST depuis le même visiteur ; 61e HTTP 429 et Retry-After 60 ; SQLite reste à 60"];
  }finally{served.stop();}
 }],
 ];
}
