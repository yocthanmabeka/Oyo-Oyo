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
  // Prévenir quand la page est fermée (ADR-119). La permission et l'abonnement ne se demandent que
  // sur un toucher (News.request), jamais au chargement ; l'abonnement du navigateur (une adresse
  // chez son service de notification, et ses deux clés) part à holo serve, à l'adresse de la page,
  // qui seul envoie (News.send ne fait rien ici). Se désabonner (News.stop) marche toujours : si
  // holo serve ne répond pas, le navigateur oublie l'abonnement lui-même, et son service le refusera.
  const bytes=t=>Uint8Array.from(atob(String(t).replace(/-/g,"+").replace(/_/g,"/")),c=>c.charCodeAt(0));
  async function push(e,name,action){
    if(action==="send")return true;
    if(busy.has(name))return true;
    const stamp=epoch,token={cancel:null,cancelled:false};busy.set(name,token);
    const address=new URL(pageKey(),location.origin).pathname+"?push";
    const tell=body=>fetch(address,{method:"POST",headers:{"content-type":"application/json"},body:JSON.stringify(body)});
    try{
      if(action==="stop"){
        status(e,"En attente…");
        const registration=isSecureContext&&navigator.serviceWorker?await navigator.serviceWorker.getRegistration("/"):null;
        const subscription=await registration?.pushManager?.getSubscription();
        if(subscription){
          let left=0;
          try{const answer=await tell({unfollow:name,endpoint:subscription.endpoint});if(answer.ok)left=Number((await answer.json()).left)||0;}catch{/* holo serve injoignable : le navigateur oublie lui-même */}
          if(!left)await subscription.unsubscribe();
        }
        finish(e,name,true,"Tu ne seras plus prévenu sur cet appareil.",stamp);return true;
      }
      if(action!=="request")throw Error("Action de notification inconnue.");
      if(!navigator.userActivation?.isActive)throw Error("Appuie à nouveau sur le bouton pour être prévenu.");
      if(!isSecureContext||!navigator.serviceWorker)throw Error("Être prévenu page fermée demande localhost ou HTTPS.");
      // Sur iPhone et iPad, seule une page installée sur l'écran d'accueil reçoit des notifications.
      const apple=/iPhone|iPad|iPod/.test(navigator.userAgent)||(navigator.platform==="MacIntel"&&navigator.maxTouchPoints>1);
      const installed=matchMedia("(display-mode: standalone)").matches||navigator.standalone===true;
      if(!("PushManager"in globalThis)||!("Notification"in globalThis))throw Error(apple&&!installed?"Sur iPhone et iPad (iOS 16.4 ou plus), ajoute d'abord cette page à l'écran d'accueil : bouton Partager, puis « Sur l'écran d'accueil ». Ouvre-la depuis son icône, puis touche à nouveau ce bouton.":"Ce navigateur ne sait pas prévenir quand la page est fermée ; elle se met à jour tant qu'elle est ouverte.");
      status(e,"En attente…");
      // La permission d'abord, dans le toucher même : Safari ne la demande que pendant le geste.
      const permission=Notification.permission==="default"?await Notification.requestPermission():Notification.permission;
      if(permission!=="granted")throw Error("Permission refusée : la page se met à jour tant qu'elle est ouverte. Tu peux changer d'avis dans les réglages du navigateur.");
      const answer=await fetch(address,{headers:{accept:"application/json"}});
      if(answer.status===501)throw Error("Être prévenu page fermée demande holo serve.");
      if(!answer.ok)throw Error("Le serveur n'a pas donné sa clé : réessaie plus tard.");
      const key=bytes((await answer.json()).key);
      const registration=await worker();
      if(stamp!==epoch||token.cancelled)return true;
      let subscription=await registration.pushManager.getSubscription();
      // Un abonnement fait avec une autre clé (le site en a changé) ne sert plus : il est refait.
      const kept=subscription?.options?.applicationServerKey;
      if(subscription&&!(kept&&new Uint8Array(kept).join()===key.join())){await subscription.unsubscribe();subscription=null;}
      subscription??=await registration.pushManager.subscribe({userVisibleOnly:true,applicationServerKey:key});
      const json=subscription.toJSON();
      const sent=await tell({follow:name,endpoint:json.endpoint,p256dh:json.keys?.p256dh??"",auth:json.keys?.auth??""});
      if(!sent.ok)throw Error((await sent.text().catch(()=>""))||"Le serveur a refusé l'abonnement.");
      finish(e,name,true,"Tu seras prévenu, même page fermée. « Ne plus me prévenir » l'arrête.",stamp);
    }catch(error){if(token.cancelled||stamp!==epoch)return true;finish(e,name,false,error?.message||"Abonnement impossible.",stamp);}
    finally{if(busy.get(name)===token)busy.delete(name);}
    return true;
  }
  function download(json,file){const url=URL.createObjectURL(new Blob([json],{type:"application/json"})),a=document.createElement("a");a.href=url;a.download=file;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
  async function run(name,action){
    const e=[...root.querySelectorAll("[data-browser-capability]")].find(e=>e.dataset.name===name);if(!e)return false;
    const spec=JSON.parse(e.dataset.browserCapability);
    // Une notification push (ADR-119) : holo serve garde l'abonnement, et seul envoie.
    if(spec.type==="Notification"&&spec.push)return push(e,name,action);
    // Une vibration (ADR-110) se joue comme un son : d'un toucher, d'une touche ou d'une règle de jeu,
    // sans permission ni attente. Jamais avant que le visiteur ait touché la page, ni s'il demande
    // moins de mouvement ; là où le navigateur ne sait pas vibrer (iPhone, ordinateur), rien ne casse.
    if(spec.type==="Device"&&spec.kind==="vibration"){
      const touched=navigator.userActivation?.hasBeenActive!==false;
      if(action==="stop"){if(typeof navigator.vibrate==="function"&&touched)navigator.vibrate(0);return true;}
      const why=typeof navigator.vibrate!=="function"?"Ce navigateur ne fait pas vibrer.":matchMedia("(prefers-reduced-motion: reduce)").matches?"Pas de vibration : tu as demandé moins de mouvement.":"";
      if(why){if(e.querySelector("[data-capability-status]")?.textContent!==why)status(e,why);return true;}
      if(touched)navigator.vibrate(spec.pattern);return true;
    }
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
