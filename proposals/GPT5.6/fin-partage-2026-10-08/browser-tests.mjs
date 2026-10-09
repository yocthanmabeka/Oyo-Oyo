// Partage réel : deux profils Chrome, attente tenue puis relâchée, SQLite du serveur.
import {readFileSync,writeFileSync} from "node:fs";
import {join} from "node:path";
import {DatabaseSync} from "node:sqlite";
export function sharingTests({engine,phone,page,startHoloServe,startChrome,pause}){
 if(phone)return[];
 const check=(ok,t)=>{if(!ok)throw Error(t);};
 const field="input[data-bind=what]",title="input[data-bind=title]";
 const text=async(q,selector,value)=>{await q.value("(()=>{const e=document.querySelector("+JSON.stringify(selector)+");e.value="+JSON.stringify(value)+";e.dispatchEvent(new Event('input',{bubbles:true}));})()");};
 const state=(name,value)=>"document.querySelector('[data-state="+name+"]')?.textContent==="+JSON.stringify(value);
 // Les lignes de la liste partagée de la leçon 102, telles qu'on les lit.
 const lines="[...document.querySelectorAll('[data-list=groceries] .holo-line')].map(l=>l.innerText.split('\\n')[0].trim())";
 // Toucher le bouton « name » de la ligne qui montre « text », comme une main.
 const inLine=async(q,text,name)=>{await q.value(`(()=>{document.getElementById("holo-target")?.removeAttribute("id");const l=[...document.querySelectorAll("[data-list=groceries] .holo-line")].find(l=>l.innerText.includes(${JSON.stringify(text)}));l.querySelector("[data-name=${name}]").id="holo-target";})()`);await q.click("#holo-target");};
 return [
 ["partage : liste simultanée et édition pendant la réponse",async(_,b)=>{
  const served=await startHoloServe(["102-une-liste-partagee.holo"]),other=await startChrome(),a=page(b,served.base),c=page(other,served.base);
  try{
   await a.open("/102-une-liste-partagee.holo",200);await c.open("/102-une-liste-partagee.holo",200);
   check(await a.until("window.__holoLive?.()")&&await c.until("window.__holoLive?.()"),"écoute absente");
   await text(a,field,"Premier");await text(c,field,"Second");
   await Promise.all([a.click("[data-name=Add]"),c.click("[data-name=Add]")]);
   check(await a.until(lines+".length===3")&&await c.until(lines+".length===3"),"un ajout concurrent perdu : "+JSON.stringify(await a.value(lines)));
   // Tenir la réponse après sa lecture ; le serveur a répondu et le visiteur tape encore.
   await a.value("(()=>{const original=fetch;window.fetch=async(...args)=>{const r=await original(...args);if(args[1]?.method==='POST'){await new Promise(resolve=>window.__releaseShare=resolve);}return r;};})()");
   await text(a,field,"Envoyé");await a.click("[data-name=Add]");
   check(await a.until("typeof window.__releaseShare==='function'",10000),"requête non tenue");
   await text(a,field,"Prochaine phrase");await a.value("window.__releaseShare()");
   check(await a.until(state("added","2"),10000),"réponse non appliquée");
   check(await a.value("document.querySelector("+JSON.stringify(field)+").value==='Prochaine phrase'"),"la réponse a effacé la nouvelle frappe");
   check(await c.until("document.getElementById('page').innerText.includes('Envoyé')"),"message non transmis");
   return[true,"deux profils Chrome, deux ajouts simultanés sans perte ; nouvelle frappe conservée après la réponse qui vide l'ancien champ"];
  }finally{other.stop();served.stop();}
 }],
 // ADR-080 : une ligne d'une liste partagée se désigne par sa clé. Une page sans JavaScript ne
 // reçoit rien en direct : elle reste en retard sur les autres, et ses rangs ne disent plus
 // quelle ligne elle touche ; sa clé, si.
 ["partage : retirer et cocher une ligne par sa clé, une page sans JavaScript en retard",async(_,b)=>{
  const served=await startHoloServe(["102-une-liste-partagee.holo"]),other=await startChrome(),a=page(b,served.base),c=page(other,served.base);
  try{
   await a.open("/102-une-liste-partagee.holo",200);check(await a.until("window.__holoLive?.()"),"écoute absente");
   for(const [rank,what] of ["Lait","Oeufs"].entries()){await text(a,field,what);await a.click("[data-name=Add]");await a.until(lines+".length==="+(rank+2),10000);}
   check(await a.until(lines+".join('|')==='Du pain|Lait|Oeufs'"),"ajouts absents : "+JSON.stringify(await a.value(lines)));
   // La seconde page, sans JavaScript, voit la liste telle qu'elle est maintenant ; puis la
   // première retire le pain.
   await other.send("Emulation.setScriptExecutionDisabled",{value:true});await c.open("/102-une-liste-partagee.holo",200);
   check((await c.value(lines)).join("|")==="Du pain|Lait|Oeufs","page sans JavaScript : "+JSON.stringify(await c.value(lines)));
   await inLine(a,"Du pain","Remove");check(await a.until(lines+".join('|')==='Lait|Oeufs'"),"pain non retiré : "+JSON.stringify(await a.value(lines)));
   // La page en retard touche « Retirer » sur le lait, au rang 1 qu'elle voit encore : le
   // serveur retire le lait (au rang 0 chez lui), et non les oeufs.
   check(await c.value("document.querySelector('[data-list=groceries] .holo-line:nth-child(2) [data-name=Remove]').value.startsWith('Remove.tap@1#')"),"le bouton sans JavaScript ne porte pas sa clé : "+await c.value("document.querySelector('[data-list=groceries] .holo-line:nth-child(2) [data-name=Remove]')?.value"));
   await inLine(c,"Lait","Remove");
   check(await c.until(lines+".join('|')==='Oeufs'"),"la page sans JavaScript a retiré la mauvaise ligne : "+JSON.stringify(await c.value(lines)));
   check(await a.until(lines+".join('|')==='Oeufs'"),"le retrait n'est pas arrivé en direct : "+JSON.stringify(await a.value(lines)));
   // La première coche les oeufs : la ligne change, et sa clé avec elle. La page en retard
   // touche « Retirer » sur l'ancienne ligne : refusé, et elle montre la liste du moment.
   await inLine(a,"Oeufs","Done");check(await a.until(lines+".join('|')==='✓ Oeufs'"),"oeufs non cochés : "+JSON.stringify(await a.value(lines)));
   await inLine(c,"Oeufs","Remove");
   check(await c.until(lines+".join('|')==='✓ Oeufs'"),"un geste sur une ligne changée a été accepté : "+JSON.stringify(await c.value(lines)));
   const db=new DatabaseSync(join(served.folder,"holo-data","site.sqlite"));
   try{const kept=db.prepare("SELECT value FROM shared WHERE name='groceries'").get()?.value??"";check(kept.includes("Oeufs")&&!kept.includes("Lait")&&!kept.includes("pain"),"base : "+kept);}finally{db.close();}
   return[true,"une page sans JavaScript en retard retire le lait qu'elle touche (rang 1 chez elle, 0 au serveur), pas les oeufs ; un geste sur une ligne cochée entre-temps est refusé ; direct et base d'accord"];
  }finally{await other.send("Emulation.setScriptExecutionDisabled",{value:false}).catch(()=>{});other.stop();served.stop();}
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
