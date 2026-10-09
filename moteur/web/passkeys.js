// Seul le navigateur dialogue avec sa clé ; le serveur vérifie ensuite sa signature.
const form=document.getElementById("passkey-form"),status=document.getElementById("passkey-status"),button=document.getElementById("passkey-button");
const decode=s=>Uint8Array.from(atob(s.replaceAll("-","+").replaceAll("_","/")),c=>c.charCodeAt(0));
const encode=b=>btoa(String.fromCharCode(...new Uint8Array(b))).replaceAll("+","-").replaceAll("/","_").replace(/=+$/,"");
async function post(path,body){
 const r=await fetch("/account/passkeys/"+path,{method:"POST",credentials:"same-origin",headers:{"content-type":"application/json"},body:JSON.stringify(body),signal:AbortSignal.timeout(15000)});
 const result=await r.json();if(!r.ok)throw Error(result.error||"La réponse est refusée.");return result;
}
if(form)form.addEventListener("submit",async e=>{
 e.preventDefault();if(button.disabled)return;button.disabled=true;status.setAttribute("role","status");status.textContent="Confirme sur ton appareil…";
 const cancel=new AbortController(),timer=setTimeout(()=>cancel.abort(),65000);
 try{
  if(!isSecureContext||!globalThis.PublicKeyCredential)throw Error("Une clé d'accès demande HTTPS, ou localhost sur ce PC.");
  const register=form.dataset.operation==="register",data=Object.fromEntries(new FormData(form)),publicKey=await post(register?"register/options":"login/options",data);
  publicKey.challenge=decode(publicKey.challenge);
  if(register){publicKey.user.id=decode(publicKey.user.id);for(const c of publicKey.excludeCredentials||[])c.id=decode(c.id);}
  const key=await navigator.credentials[register?"create":"get"]({publicKey,signal:cancel.signal});
  if(!key||key.type!=="public-key")throw Error("Aucune clé n'a été donnée.");
  const body={id:encode(key.rawId),clientDataJSON:encode(key.response.clientDataJSON)};
  if(register){body.attestationObject=encode(key.response.attestationObject);body.label=data.label;}
  else{body.authenticatorData=encode(key.response.authenticatorData);body.signature=encode(key.response.signature);body.userHandle=key.response.userHandle?encode(key.response.userHandle):"";}
  const result=await post(register?"register/finish":"login/finish",body);
  if(result.next!=="/account"&&result.next!=="/account/passkeys")throw Error("Adresse de retour refusée.");
  location.assign(result.next);
 }catch(error){status.setAttribute("role","alert");status.textContent=error?.name==="NotAllowedError"||error?.name==="AbortError"?"Demande annulée ou expirée. Tu peux réessayer ou utiliser ton mot de passe.":error?.message||"La clé n'a pas répondu.";}
 finally{clearTimeout(timer);button.disabled=false;}
});
