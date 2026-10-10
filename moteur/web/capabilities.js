// Capacités explicites du navigateur ; chargé seulement si elles sont déclarées.
export function browserCapabilities({root,source,state,receive,exported,change,emit,pageKey}) {
  const busy=new Map(),streams=new Map(),timers=new Map();let epoch=0;
  const status=(e,t)=>{const n=e?.querySelector("[data-capability-status]");if(n)n.textContent=t;};
  const finish=(e,n,ok,t,k)=>{if(k!==epoch||!e.isConnected)return;status(e,t);emit(n+(ok?".done":".failed"));};
  function closeStream(name){const c=streams.get(name);if(c){c.stream.getTracks().forEach(t=>t.stop());c.video?.remove();clearTimeout(c.timer);streams.delete(name);}}
  async function worker(){
    if(!isSecureContext||!navigator.serviceWorker)throw Error("Cette capacité demande localhost ou HTTPS et les service workers.");
    const r=await navigator.serviceWorker.register("/holo-capabilities-worker.js",{scope:"/"});
    await Promise.race([navigator.serviceWorker.ready,new Promise((_,no)=>setTimeout(()=>no(Error("Le service worker ne répond pas.")),10000))]);return r;
  }
  async function messageWorker(message){
    const r=await worker(),c=new MessageChannel();
    const done=new Promise((ok,no)=>{const t=setTimeout(()=>{c.port1.close();no(Error("La sauvegarde ne répond pas."));},30000);c.port1.onmessage=({data})=>{clearTimeout(t);c.port1.close();data.ok?ok(data):no(Error(data.error));};});
    r.active.postMessage(message,[c.port2]);return done;
  }
  function download(json,file){const url=URL.createObjectURL(new Blob([json],{type:"application/json"})),a=document.createElement("a");a.href=url;a.download=file;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
  async function run(name,action){
    const e=[...root.querySelectorAll("[data-browser-capability]")].find(e=>e.dataset.name===name);if(!e)return false;
    const spec=JSON.parse(e.dataset.browserCapability);
    if(action==="stop"){closeStream(name);clearTimeout(timers.get(name));timers.delete(name);const pending=busy.get(name);if(pending){pending.cancelled=true;pending.cancel?.();}busy.delete(name);status(e,"Arrêté.");return true;}
    if(busy.has(name))return true;
    const stamp=epoch,token={cancel:null,cancelled:false};busy.set(name,token);
    try{
      if(!navigator.userActivation?.isActive)throw Error("Appuie à nouveau sur le bouton pour autoriser cette action.");
      status(e,"En attente…");
      if(spec.type==="Transfer"){
        if(action==="export"){const json=exported(source(),state(),name);if(new TextEncoder().encode(json).length>65536)throw Error("Le fichier dépasse 64 Ko.");download(json,spec.file);}
        else if(action==="import"){
          const picker=document.createElement("input");picker.type="file";picker.accept=".json,application/json";picker.hidden=true;picker.dataset.holoImport=name;document.body.append(picker);
          let json;
          try{
            const file=await new Promise((ok,no)=>{
              const timer=setTimeout(()=>no(Error("Import annulé après une minute.")),60000);
              const done=fn=>v=>{clearTimeout(timer);fn(v);};
              picker.onchange=done(()=>ok(picker.files[0]));picker.oncancel=done(()=>no(Error("Import annulé.")));token.cancel=done(()=>no(Error("Import arrêté.")));picker.click();
            });
            if(!file||file.size>65536)throw Error("Choisis un fichier JSON de 64 Ko au plus.");json=await file.text();
          }finally{picker.remove();}
          if(stamp!==epoch||token.cancelled)return true;change(receive(source(),state(),name,json));
        }else throw Error("Action de transfert inconnue.");
      }else if(spec.type==="Device"&&spec.kind==="share"){
        // Partager la page (ADR-107) : la feuille de partage du téléphone, avec le titre et l'adresse ;
        // sans elle (un ordinateur), l'adresse copiée. L'appel part dans le toucher même, avant toute
        // attente : un navigateur n'ouvre la feuille (et Safari ne copie) que pendant le geste du visiteur.
        if(action!=="request")throw Error("Action de partage inconnue.");
        const data={title:document.title,url:location.href},sheet=typeof navigator.share==="function"&&(!navigator.canShare||navigator.canShare(data));
        try{await(sheet?navigator.share(data):navigator.clipboard.writeText(data.url));}
        catch(error){
          // Fermer la feuille sans rien choisir n'est pas une panne : ni done, ni failed.
          if(sheet&&error?.name==="AbortError"){if(stamp===epoch&&!token.cancelled)status(e,"Partage annulé.");return true;}
          throw Error("Partage impossible ici. L'adresse de la page, à copier : "+data.url);
        }
        if(stamp!==epoch||token.cancelled)return true;
        finish(e,name,true,sheet?"Page partagée.":"Adresse de la page copiée : colle-la où tu veux.",stamp);return true;
      }else if(spec.type==="Device"){
        if(!isSecureContext)throw Error("L'appareil demande localhost ou HTTPS.");
        if(action==="write"&&spec.kind==="clipboard"){const json=JSON.parse(exported(source(),state(),name));await navigator.clipboard.writeText(json[spec.value]);}
        else if(action!=="request")throw Error("Action d'appareil inconnue.");
        else if(spec.kind==="clipboard"||spec.kind==="position"){
          const value=spec.kind==="clipboard"?await navigator.clipboard.readText():await new Promise((ok,no)=>navigator.geolocation.getCurrentPosition(p=>ok(JSON.stringify({latitude:p.coords.latitude,longitude:p.coords.longitude,accuracy:p.coords.accuracy})),no,{enableHighAccuracy:false,timeout:10000,maximumAge:0}));
          if(stamp!==epoch||token.cancelled)return true;change(receive(source(),state(),name,JSON.stringify({[spec.value]:value})));
        }else{
          closeStream(name);
          const stream=await navigator.mediaDevices.getUserMedia(spec.kind==="camera"?{video:{width:{ideal:640},height:{ideal:480}},audio:false}:{audio:true,video:false});
          if(stamp!==epoch||token.cancelled||!e.isConnected){stream.getTracks().forEach(t=>t.stop());return true;}
          let video;
          if(spec.kind==="camera"){video=document.createElement("video");video.muted=true;video.playsInline=true;video.controls=true;video.setAttribute("aria-label",spec.label);video.style.maxWidth="100%";video.srcObject=stream;e.querySelector("[data-capability-preview]").append(video);}
          streams.set(name,{stream,video,timer:setTimeout(()=>{closeStream(name);status(e,"Arrêté après une minute.");},60000)});
          if(video)await video.play();
          stream.getTracks().forEach(t=>t.addEventListener("ended",()=>{closeStream(name);status(e,"Capture arrêtée.");},{once:true}));
          finish(e,name,true,"Appareil actif localement ; arrêt automatique après une minute.",stamp);return true;
        }
      }else if(spec.type==="Notification"){
        if(!("Notification"in globalThis)||!isSecureContext)throw Error("Les notifications ne sont pas disponibles ici.");
        const permission=Notification.permission==="default"?await Notification.requestPermission():Notification.permission;
        if(permission!=="granted")throw Error("Permission refusée. La page reste utilisable.");
        const r=await worker();if(stamp!==epoch||token.cancelled)return true;
        const show=async()=>{if(stamp!==epoch)return;timers.delete(name);try{await r.showNotification(spec.title,{body:spec.body||"",tag:"holo-"+name});finish(e,name,true,"Notification affichée.",stamp);}catch{finish(e,name,false,"Notification indisponible.",stamp);}};
        if(spec.after>0){clearTimeout(timers.get(name));timers.set(name,setTimeout(show,spec.after*1000));status(e,"Rappel programmé tant que cette page reste ouverte.");}else await show();return true;
      }else if(spec.type==="Offline"){
        const url=new URL(pageKey(),location.origin);if(url.origin!==location.origin||url.search||url.hash)throw Error("Sauvegarde seulement à l'adresse publique, sans paramètres.");
        const urls=[url.href,...spec.files.map(f=>new URL(f,url).href),...["/page-engine.js","/capabilities.js","/pkg-light/holo_engine.js","/pkg-light/holo_engine_bg.wasm"].map(p=>new URL(p,url).href)];
        const saved=await messageWorker({type:action==="remove"?"remove":"save",page:url.href,urls});
        finish(e,name,true,action==="remove"?"Copie hors-ligne effacée.":"Copie hors-ligne prête ("+saved.bytes+" octets).",stamp);return true;
      }else return false;
      finish(e,name,true,"Terminé.",stamp);
    }catch(error){if(token.cancelled||stamp!==epoch)return true;closeStream(name);finish(e,name,false,(typeof error==="string"?error:error?.message)||"Permission refusée ou appareil indisponible.",stamp);}
    finally{if(busy.get(name)===token)busy.delete(name);}
    return true;
  }
  function stop(){epoch++;for(const name of streams.keys())closeStream(name);for(const t of timers.values())clearTimeout(t);timers.clear();for(const t of busy.values()){t.cancelled=true;t.cancel?.();}busy.clear();}
  addEventListener("pagehide",stop);document.addEventListener("visibilitychange",()=>{if(document.hidden){for(const n of streams.keys())closeStream(n);for(const [name,t] of busy){const e=[...root.querySelectorAll("[data-browser-capability]")].find(e=>e.dataset.name===name);if(e && ["camera","microphone"].includes(JSON.parse(e.dataset.browserCapability).kind)){t.cancelled=true;busy.delete(name);status(e,"Capture arrêtée : page cachée.");}}}});
  return {run,stop};
}
