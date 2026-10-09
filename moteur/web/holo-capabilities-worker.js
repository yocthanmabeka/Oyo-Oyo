// Cache explicite de pages publiques (ADR-096) ; aucune écriture n'est interceptée, ni le direct.
const PREFIX="holo-offline-v1-",MAX_BYTES=16*1024*1024,MAX_PAGES=8,MAX_URLS=24;
self.addEventListener("install",e=>e.waitUntil(self.skipWaiting()));
self.addEventListener("activate",e=>e.waitUntil(self.clients.claim()));
const locks=new Map(),keyFor=page=>PREFIX+encodeURIComponent(page);
const sourceKey=url=>{const k=new URL(url);k.searchParams.set("__holo_source","1");return k.href;};
async function bounded(r,left){
 if(!r.ok||r.type==="opaque"||r.redirected||/\b(no-store|private)\b/i.test(r.headers.get("cache-control")||"")||r.headers.has("set-cookie"))throw Error("Une réponse privée, refusée ou redirigée ne se sauvegarde pas.");
 const reader=r.body.getReader(),chunks=[];let size=0;
 try{for(;;){const {done,value}=await reader.read();if(done)break;size+=value.byteLength;if(size>left)throw Error("La copie dépasse 16 Mio.");chunks.push(value);}}finally{await reader.cancel().catch(()=>{});}
 return {response:new Response(new Blob(chunks),{status:200,headers:r.headers}),size};
}
async function save(data){
 const page=new URL(data.page);
 if(page.origin!==self.location.origin||page.search||page.hash||!page.pathname.endsWith(".holo")||page.pathname.startsWith("/account"))throw Error("Une page publique .holo de ce site est attendue.");
 const urls=[...new Set(data.urls)],required=[page.href,...["/page-engine.js","/capabilities.js","/pkg-light/holo_engine.js","/pkg-light/holo_engine_bg.wasm"].map(p=>new URL(p,page).href)];
 if(urls.length>MAX_URLS||required.some(u=>!urls.includes(u)))throw Error("La liste des ressources est incomplète ou trop longue.");
 if(urls.some(u=>{const p=new URL(u);return p.origin!==page.origin||p.search||p.hash||p.pathname.startsWith("/account")||(!required.includes(u)&&(p.pathname.endsWith(".holo")||p.pathname.endsWith(".html")||p.pathname.endsWith(".js")||p.pathname.endsWith(".wasm")));}))throw Error("Ressource hors du contrat.");
 const name=keyFor(page.href),existing=(await caches.keys()).filter(k=>k.startsWith(PREFIX));
 if(!existing.includes(name)&&existing.length>=MAX_PAGES)throw Error("Huit pages hors-ligne au plus ; efface une copie.");
 const entries=[];let bytes=0;
 for(const url of urls){
  const item=await bounded(await fetch(url,{credentials:"omit",cache:"no-store",headers:{accept:url===page.href?"text/html":"*/*"}}),MAX_BYTES-bytes);bytes+=item.size;
  if(url===page.href){
   const html=await item.response.clone().text();
   // Demandée sans cookie, la page est celle d'un premier visiteur : holo serve la fabrique avec
   // ses valeurs de départ (data-visit n'en porte pas d'autres). Ni compte, ni valeurs partagées.
   if(!html.includes("holo-Offline")||html.includes("holo-account")||html.includes("data-shared="))throw Error("Cette page ne déclare pas une copie publique hors-ligne.");
   const decode=s=>s.replace(/&quot;/g,'"').replace(/&#39;|&#x27;/g,"'").replace(/&lt;/g,"<").replace(/&gt;/g,">").replace(/&amp;/g,"&");
   const declared=[...html.matchAll(/data-browser-capability="([^"]*)"/g)].map(m=>JSON.parse(decode(m[1]))).filter(s=>s.type==="Offline");
   if(declared.length!==1||!Array.isArray(declared[0].files))throw Error("Une seule déclaration hors-ligne est attendue.");
   const expected=[...new Set([...required,...declared[0].files.map(f=>new URL(f,page).href)])];
   if(urls.length!==expected.length||urls.some(u=>!expected.includes(u)))throw Error("Seules les ressources déclarées par cette page se sauvegardent.");
  }
  entries.push([url,item.response]);
 }
 const raw=await bounded(await fetch(page.href,{credentials:"omit",cache:"no-store",headers:{accept:"text/plain"}}),MAX_BYTES-bytes);bytes+=raw.size;entries.push([sourceKey(page.href),raw.response]);
 // Le remplacement reste borné ; en cas de panne d'écriture, une copie incomplète est retirée.
 const cache=await caches.open(name);
 try{for(const r of await cache.keys())await cache.delete(r);for(const [u,r]of entries)await cache.put(u,r);}catch(e){await caches.delete(name);throw e;}
 return {ok:true,bytes};
}
self.addEventListener("message",e=>{
 const d=e.data,p=e.ports[0];if(!p||!["save","remove"].includes(d?.type))return;
 e.waitUntil((async()=>{
  const client=await self.clients.get(e.source.id);if(!client||new URL(client.url).origin!==self.location.origin)throw Error("Client inconnu.");
  const page=new URL(d.page);if(page.origin!==self.location.origin||new URL(client.url).pathname!==page.pathname)throw Error("Seule la page affichée se sauvegarde ou s'efface.");
  if(locks.size)throw Error("Une copie est déjà en cours.");locks.set(page.href,true);
  try{p.postMessage(d.type==="remove"?{ok:await caches.delete(keyFor(page.href))||true}:await save(d));}finally{locks.delete(page.href);}
 })().catch(error=>p.postMessage({ok:false,error:error.message})));
});
// Le réseau d'abord ; la copie seulement quand il manque. Une écriture (un geste, un formulaire,
// un compte : tout ce qui n'est pas GET) et le direct (text/event-stream, ADR-079) vont au réseau
// sans passer par ici : jamais retenus, jamais rejoués, jamais servis depuis une copie.
self.addEventListener("fetch",e=>{
 const r=e.request,u=new URL(r.url);
 if(r.method!=="GET"||(r.headers.get("accept")||"").includes("text/event-stream")||u.origin!==self.location.origin||u.search||u.pathname.startsWith("/account")||r.headers.has("authorization"))return;
 e.respondWith((async()=>{try{return await fetch(r);}catch(error){
  const key=r.headers.get("accept")?.includes("text/plain")&&u.pathname.endsWith(".holo")?sourceKey(u.href):u.href;
  for(const n of(await caches.keys()).filter(k=>k.startsWith(PREFIX))){const saved=await(await caches.open(n)).match(key);if(saved)return saved;}throw error;
 }})());
});
