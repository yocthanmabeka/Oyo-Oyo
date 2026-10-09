// Authentificateur virtuel de Chrome : le navigateur crée la clé et signe réellement.
import {join} from "node:path";
import {DatabaseSync} from "node:sqlite";
import {request} from "node:http";
export function passkeyTests({phone,page,startHoloServe}){
 if(phone)return [];
 const check=(ok,message)=>{if(!ok)throw Error(message);};
 // Une demande HTTP brute, pour éprouver ce que le serveur fait d'un Host et d'une Origin choisis.
 const post=(url,headers,body)=>new Promise((ok,ko)=>{const r=request(url,{method:"POST",headers},(res)=>{res.resume();res.on("end",()=>ok(res.statusCode));});r.on("error",ko);r.end(body);});
 return [["clés d'accès : créer, signer, refuser, révoquer",async(_,b)=>{
  const served=await startHoloServe(["104-se-connecter.holo","107-se-connecter-par-une-cle.holo"]);
  const q=page(b,served.base);
  let authenticator;
  try{
   await b.send("Network.clearBrowserCookies");await b.send("WebAuthn.enable");
   // Le protocole de Chrome rend { id, result } : la réponse est dans result.
   const added=await b.send("WebAuthn.addVirtualAuthenticator",{options:{protocol:"ctap2",transport:"internal",hasResidentKey:true,hasUserVerification:true,isUserVerified:true,automaticPresenceSimulation:true}});
   authenticator=added.result?.authenticatorId;check(authenticator,"authentificateur virtuel absent : "+JSON.stringify(added));
   await q.open("/account/signup",200);await q.type("#name","WithPasskey");await q.type("#password","une longue phrase pour ma clé locale");await q.type("#again","une longue phrase pour ma clé locale");await q.click('form button[type="submit"]');
   check(await q.until("location.pathname==='/account'"),"compte non créé");
   await q.open("/account/passkeys",200);await q.type("#label","Clé du PC");await q.type("#password","une longue phrase pour ma clé locale");await q.click("#passkey-button");
   check(await q.until("document.querySelectorAll('[data-passkey]').length===1",15000),"clé non enregistrée : "+await q.value("document.body.innerText"));
   const keys=(await b.send("WebAuthn.getCredentials",{authenticatorId:authenticator})).result?.credentials??[];check(keys.length===1&&keys[0].isResidentCredential,"clé découvrable absente : "+JSON.stringify(keys));
   const db=new DatabaseSync(join(served.folder,"holo-data","site.sqlite"));try{check(db.prepare("SELECT COUNT(*) n FROM passkeys").get().n===1,"clé non gardée chez l'auteur");}finally{db.close();}
   await q.open("/account",200);await q.click('form[action="/account/signout"] button');await q.until("location.pathname==='/account/signin'");
   for(const fault of ["signature","origin","userHandle"]){
    await q.open("/account/passkeys",200);
    await q.value("(()=>{const original=fetch;window.fetch=async(input,options)=>{if(String(input).endsWith('/login/finish')){const body=JSON.parse(options.body);window.__passkeyBody=JSON.stringify(body);const encode=b=>btoa(String.fromCharCode(...new Uint8Array(b))).replaceAll('+','-').replaceAll('/','_').replace(/=+$/,'');const decode=s=>Uint8Array.from(atob(s.replaceAll('-','+').replaceAll('_','/')),c=>c.charCodeAt(0));const fault="+JSON.stringify(fault)+";if(fault==='signature'){const b=decode(body.signature);b[b.length-1]^=1;body.signature=encode(b);}if(fault==='origin'){const data=JSON.parse(new TextDecoder().decode(decode(body.clientDataJSON)));data.origin='https://elsewhere.example';body.clientDataJSON=encode(new TextEncoder().encode(JSON.stringify(data)));}if(fault==='userHandle')body.userHandle=encode(new Uint8Array(8));options={...options,body:JSON.stringify(body)};}return original(input,options);};})()");
    await q.click("#passkey-button");
    check(await q.until("document.getElementById('passkey-status')?.getAttribute('role')==='alert'",15000),"réponse altérée acceptée : "+fault);
    check(await q.value("location.pathname==='/account/passkeys'"),"session ouverte malgré "+fault);
    const replay=await q.value("(async()=>{const r=await fetch('/account/passkeys/login/finish',{method:'POST',headers:{'content-type':'application/json'},body:window.__passkeyBody});return r.status;})()");
    check(replay===401,"défi réemployable après refus");
   }
   await q.open("/account/passkeys",200);await q.click("#passkey-button");check(await q.until("location.pathname==='/account'",15000),"signature correcte refusée : "+await q.value("document.body.innerText"));
   // Au clavier, confirmer le retrait ; l'identifiant ne suffit pas sans mot de passe.
   await q.open("/account/passkeys",200);await q.type('li input[name="password"]',"wrong");await q.click('li form button');check(await q.until("document.body.innerText.includes('confirmation du compte')"),"retrait sans mot de passe accepté");
   await q.open("/account/passkeys",200);await q.type('li input[name="password"]',"une longue phrase pour ma clé locale");await q.click('li form button');check(await q.until("location.pathname==='/account/passkeys'&&document.querySelectorAll('[data-passkey]').length===0"),"clé non retirée");
   await q.open("/account",200);await q.click('form[action="/account/signout"] button');await q.until("location.pathname==='/account/signin'");
   await q.open("/account/passkeys",200);await q.click("#passkey-button");check(await q.until("document.getElementById('passkey-status')?.getAttribute('role')==='alert'",15000),"clé retirée encore acceptée");
   // L'adresse IP en HTTP n'est pas une origine WebAuthn admise, même depuis ce PC : sans
   // HOLO_ORIGIN, seul localhost l'est. Le même serveur, appelé par 127.0.0.1, refuse.
   const byIp=served.base.replace("://localhost:","://127.0.0.1:");check(byIp!==served.base,"adresse IP introuvable : "+served.base);
   const notLocal=await post(byIp+"/account/passkeys/login/options",{"content-type":"application/json",origin:byIp},"{}");check(notLocal===403,"adresse IP HTTP admise comme origine WebAuthn : "+notLocal);
   // Et localhost, avec la même demande, est bien admis : le refus vient de l'adresse.
   const local=await post(served.base+"/account/passkeys/login/options",{"content-type":"application/json",origin:served.base},"{}");check(local===200,"localhost refusé : "+local);
   return [true,"Chrome/CTAP2 réel : clé résidente ES256 et UV ; signature correcte acceptée sans mot de passe ; signature, origine et compte altérés refusés ; nonce consommé après refus ; retrait confirmé et clé refusée ensuite ; 127.0.0.1 en HTTP refusé (403), localhost admis"];
  }finally{if(authenticator)await b.send("WebAuthn.removeVirtualAuthenticator",{authenticatorId:authenticator});await b.send("WebAuthn.disable");served.stop();}
 }]];
}
