// Cache explicite de pages publiques ; aucune écriture n'est interceptée.
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
  if(url===page.href){const html=await item.response.clone().text();if(!html.includes("data-browser-capability=")||!html.includes("holo-Offline")||html.includes("holo-account")||html.includes("data-shared=")||html.includes("data-visit="))throw Error("Cette page ne déclare pas une copie publique hors-ligne.");}
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
  const page=new URL(d.page);if(page.origin!==self.location.origin)throw Error("Une page de ce site est attendue.");
  if(locks.has(page.href))throw Error("Une copie est déjà en cours.");locks.set(page.href,true);
  try{p.postMessage(d.type==="remove"?{ok:await caches.delete(keyFor(page.href))||true}:await save(d));}finally{locks.delete(page.href);}
 })().catch(error=>p.postMessage({ok:false,error:error.message})));
});
self.addEventListener("fetch",e=>{
 const r=e.request,u=new URL(r.url);if(r.method!=="GET"||u.origin!==self.location.origin||u.search||u.pathname.startsWith("/account")||r.headers.has("authorization"))return;
 e.respondWith((async()=>{try{return await fetch(r);}catch(error){
  const key=r.headers.get("accept")?.includes("text/plain")&&u.pathname.endsWith(".holo")?sourceKey(u.href):u.href;
  for(const n of(await caches.keys()).filter(k=>k.startsWith(PREFIX))){const saved=await(await caches.open(n)).match(key);if(saved)return saved;}throw error;
 }})());
});
