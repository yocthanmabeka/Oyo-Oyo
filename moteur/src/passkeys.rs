//! WebAuthn ES256 : les signatures sont vérifiées chez l'auteur, avec RustCrypto.
//! L'origine est fixée par le serveur. Sans configuration, seul localhost est admis.
//! Les données du navigateur ne créent jamais une session avant toutes les vérifications.
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use crate::{accounts::{self,Member}, flat::escape, server::{Ask,Reply,Site}};

const COOKIE:&str="holo_passkey";
const MAX_BODY:usize=16_384;
const MAX_KEYS:i64=8;
const MAX_CHALLENGES:i64=10_000;

pub fn prepare(base:&Connection,now:u64)->Result<(),String>{
 base.execute_batch("CREATE TABLE IF NOT EXISTS passkeys(id TEXT PRIMARY KEY,account INTEGER NOT NULL,key BLOB NOT NULL,counter INTEGER NOT NULL,backed INTEGER NOT NULL,label TEXT NOT NULL,created INTEGER NOT NULL);
 CREATE TABLE IF NOT EXISTS passkey_challenges(session TEXT PRIMARY KEY,account INTEGER NOT NULL,challenge TEXT NOT NULL,operation TEXT NOT NULL,origin TEXT NOT NULL,bound TEXT NOT NULL,created INTEGER NOT NULL);").map_err(|e|e.to_string())?;
 base.execute("DELETE FROM passkey_challenges WHERE created<?1",params![now.saturating_sub(300) as i64]).map_err(|e|e.to_string())?; Ok(())
}
pub fn configured_origin()->Result<Option<String>,String>{
 match std::env::var("HOLO_ORIGIN"){Ok(s)=>{let (host,_) = origin_parts(&s).ok_or("HOLO_ORIGIN : une origine HTTPS, sans chemin, attendue")?;if !s.starts_with("https://")||host=="localhost"{return Err("HOLO_ORIGIN : un nom de domaine HTTPS attendu".into());}Ok(Some(s))},Err(std::env::VarError::NotPresent)=>Ok(None),Err(_)=>Err("HOLO_ORIGIN illisible".into())}
}
fn origin_parts(origin:&str)->Option<(&str,&str)>{
 let authority=origin.strip_prefix("https://").or_else(||origin.strip_prefix("http://"))?;
 if authority.is_empty()||authority.len()>259||authority.contains(['/', '?','#','@','\\']) {return None;}
 let (host,port)=authority.split_once(':').map_or((authority,None),|(h,p)|(h,Some(p)));
 if !host.is_ascii()||host!=host.to_ascii_lowercase()||host.split('.').any(|s|s.is_empty()||s.len()>63||s.starts_with('-')||s.ends_with('-')||!s.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'-')){return None;}
 if port.is_some_and(|p|p.is_empty()||!p.bytes().all(|c|c.is_ascii_digit())||p.parse::<u16>().ok().is_none_or(|v|v==0)){return None;}
 Some((host,authority))
}
fn origin_for(site:&Site,ask:&Ask)->Option<String>{
 // Un serveur HTTP reste accessible sur le réseau : une fausse valeur Host ne suffit pas.
 if !ask.peer.parse::<std::net::IpAddr>().ok()?.is_loopback(){return None;}
 if let Some(fixed)=&site.passkeys_origin {let (_,authority)=origin_parts(fixed)?;return (ask.host==authority).then(||fixed.clone());}
 let origin=format!("http://{}",ask.host);let (host,_)=origin_parts(&origin)?;
 (host=="localhost").then_some(origin)
}
fn random(bytes:usize)->Result<Vec<u8>,String>{let mut b=vec![0;bytes];getrandom::getrandom(&mut b).map_err(|e|e.to_string())?;Ok(b)}
// Le base64url sans remplissage, strict : aussi celui des notifications push (ADR-119).
pub(crate) fn b64(b:&[u8])->String{
 const A:&[u8]=b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
 let mut s=String::new();for c in b.chunks(3){let v=(u32::from(c[0])<<16)|(u32::from(*c.get(1).unwrap_or(&0))<<8)|u32::from(*c.get(2).unwrap_or(&0));s.push(A[(v>>18)as usize]as char);s.push(A[((v>>12)&63)as usize]as char);if c.len()>1{s.push(A[((v>>6)&63)as usize]as char);}if c.len()>2{s.push(A[(v&63)as usize]as char);}}s
}
pub(crate) fn un64(s:&str,max:usize)->Option<Vec<u8>>{
 if s.len()>max.div_ceil(3)*4||s.len()%4==1{return None;}
 let mut out=Vec::new();let(mut v,mut bits)=(0u32,0usize);
 for c in s.bytes(){let n=match c{b'A'..=b'Z'=>c-b'A',b'a'..=b'z'=>c-b'a'+26,b'0'..=b'9'=>c-b'0'+52,b'-'=>62,b'_'=>63,_=>return None};v=(v<<6)|u32::from(n);bits+=6;if bits>=8{bits-=8;out.push((v>>bits)as u8);v&=(1<<bits)-1;}}
 (out.len()<=max&&b64(&out)==s).then_some(out)
}
fn quote(s:&str)->String {let mut t=String::from("\"");for c in s.chars(){match c{'"'=>t.push_str("\\\""),'\\'=>t.push_str("\\\\"),c if c<' '=>t.push_str(&format!("\\u{:04x}",c as u32)),_=>t.push(c)}}t.push('"');t}
fn hex(b:&[u8])->String{b.iter().map(|v|format!("{v:02x}")).collect()}
fn cookie<'a>(header:&'a str)->Option<&'a str>{header.split(';').filter_map(|p|p.trim().split_once('=')).find(|(n,_)|*n==COOKIE).map(|(_,v)|v).filter(|v|v.len()==32&&v.bytes().all(|c|c.is_ascii_hexdigit()))}
fn session(token:&str)->String{hex(&Sha256::digest(token.as_bytes()))}
fn json(status:u16,body:&str,cookie:Option<String>)->Reply{let mut h=accounts::private_headers();h.push(("Content-Type".into(),"application/json; charset=utf-8".into()));if let Some(c)=cookie{h.push(("Set-Cookie".into(),c));}Reply{status,headers:h,body:body.as_bytes().to_vec()}}
fn error(status:u16,message:&str)->Reply{json(status,&format!("{{\"error\":{}}}",quote(message)),None)}

/// Un lecteur JSON strict et borné, pour ces objets plats seulement. Aucun booléen ne vaut 0 ou 1.
#[derive(Debug,PartialEq)]
enum Atom{Text(String),Bool(bool),Null}
fn flat_json(s:&str)->Option<Vec<(String,Atom)>>{
 fn ws(b:&[u8],i:&mut usize){while b.get(*i).is_some_and(|v|matches!(v,b' '|b'\n'|b'\r'|b'\t')){*i+=1;}}
 fn text(b:&[u8],i:&mut usize)->Option<String>{
  if b.get(*i)!=Some(&b'"'){return None;}*i+=1;let mut out=String::new();let mut raw=*i;
  while let Some(&c)=b.get(*i){if c==b'"'{out.push_str(std::str::from_utf8(&b[raw..*i]).ok()?);*i+=1;return Some(out);}
   if c<32{return None;}if c==b'\\'{out.push_str(std::str::from_utf8(&b[raw..*i]).ok()?);*i+=1;let e=*b.get(*i)?;*i+=1;match e{b'"'=>out.push('"'),b'\\'=>out.push('\\'),b'/'=>out.push('/'),b'b'=>out.push('\u{8}'),b'f'=>out.push('\u{c}'),b'n'=>out.push('\n'),b'r'=>out.push('\r'),b't'=>out.push('\t'),b'u'=>{let h=std::str::from_utf8(b.get(*i..*i+4)?).ok()?;let n=u32::from_str_radix(h,16).ok()?;*i+=4;out.push(char::from_u32(n)?);},_=>return None};raw=*i;}else{*i+=1;}
  }None
 }
 let b=s.as_bytes();let mut i=0;ws(b,&mut i);if b.get(i)!=Some(&b'{'){return None;}i+=1;let mut out=Vec::new();ws(b,&mut i);
 if b.get(i)==Some(&b'}'){i+=1;ws(b,&mut i);return(i==b.len()).then_some(out);}
 loop{ws(b,&mut i);let k=text(b,&mut i)?;if k.len()>80||out.iter().any(|(known,_)|known==&k)||out.len()>=16{return None;}ws(b,&mut i);if b.get(i)!=Some(&b':'){return None;}i+=1;ws(b,&mut i);
 let v=if b.get(i)==Some(&b'"'){Atom::Text(text(b,&mut i)?)}else{let tail=b.get(i..)?;if tail.starts_with(b"false"){i+=5;Atom::Bool(false)}else if tail.starts_with(b"true"){i+=4;Atom::Bool(true)}else if tail.starts_with(b"null"){i+=4;Atom::Null}else{return None;}};
 out.push((k,v));ws(b,&mut i);match b.get(i){Some(b',')=>{i+=1;},Some(b'}')=>{i+=1;break;},_=>return None,}
 }ws(b,&mut i);(i==b.len()).then_some(out)
}
fn field<'a>(o:&'a[(String,Atom)],key:&str)->Option<&'a str>{o.iter().find(|(k,_)|k==key).and_then(|(_,v)|if let Atom::Text(s)=v{Some(s.as_str())}else{None})}
fn client_data(bytes:&[u8],challenge:&str,origin:&str,operation:&str)->bool{
 let Some(o)=std::str::from_utf8(bytes).ok().and_then(flat_json) else{return false;};
 field(&o,"type")==Some(operation)&&field(&o,"challenge")==Some(challenge)&&field(&o,"origin")==Some(origin)&&o.iter().all(|(k,v)|match k.as_str(){"crossOrigin"=>*v==Atom::Bool(false),"topOrigin"=>false,_=>true})
}

/// CBOR definite-length seulement : la taille totale, la profondeur et chaque collection sont bornées.
#[derive(Debug,Clone,PartialEq)]
enum Cbor{Int(i64),Bytes(Vec<u8>),Text(String),Map(Vec<(Cbor,Cbor)>),Array(Vec<Cbor>),Other}
fn cbor(b:&[u8],at:&mut usize,depth:usize)->Option<Cbor>{
 if depth>6{return None;}let h=*b.get(*at)?;*at+=1;
 let major=h>>5;let add=h&31;let n=match add{0..=23=>u64::from(add),24=>{let n=*b.get(*at)?;*at+=1;u64::from(n)},25=>{let x:u16=u16::from_be_bytes(b.get(*at..*at+2)?.try_into().ok()?);*at+=2;u64::from(x)},26=>{let x=u32::from_be_bytes(b.get(*at..*at+4)?.try_into().ok()?);*at+=4;u64::from(x)},27=>{let x=u64::from_be_bytes(b.get(*at..*at+8)?.try_into().ok()?);*at+=8;x},_=>return None};
 match major{
  0=>Some(Cbor::Int(i64::try_from(n).ok()?)),1=>Some(Cbor::Int(-1-i64::try_from(n).ok()?)),
  2|3=>{let n=usize::try_from(n).ok()?;if n>8192{return None;}let s=b.get(*at..at.checked_add(n)?)?;*at+=n;if major==2{Some(Cbor::Bytes(s.to_vec()))}else{Some(Cbor::Text(std::str::from_utf8(s).ok()?.into()))}},
  4=>{if n>128{return None;}let mut a=Vec::new();for _ in 0..n{a.push(cbor(b,at,depth+1)?);}Some(Cbor::Array(a))},
  5=>{if n>16{return None;}let mut m=Vec::new();for _ in 0..n{let k=cbor(b,at,depth+1)?;if m.iter().any(|(old,_)|old==&k){return None;}m.push((k,cbor(b,at,depth+1)?));}Some(Cbor::Map(m))},
  7 if add==20||add==21||add==22=>Some(Cbor::Other),_=>None,
 }
}
fn map_at<'a>(m:&'a[(Cbor,Cbor)],k:Cbor)->Option<&'a Cbor>{m.iter().find(|(key,_)|*key==k).map(|(_,v)|v)}
fn auth_head(auth:&[u8],rp:&str,register:bool)->Option<(u32,bool)>{
 if auth.len()<37||auth[..32]!=Sha256::digest(rp.as_bytes())[..]{return None;}
 let flags=auth[32];let mask=if register{0x5d}else{0x1d};
 if flags&!mask!=0||flags&5!=5||((flags&0x40!=0)!=register)||(flags&0x10!=0&&flags&8==0){return None;}
 if !register&&auth.len()!=37{return None;}
 Some((u32::from_be_bytes(auth[33..37].try_into().ok()?),flags&8!=0))
}
fn registration(att:&[u8],id:&[u8],rp:&str)->Option<(Vec<u8>,u32,bool)>{
 let mut at=0;let Cbor::Map(m)=cbor(att,&mut at,0)? else{return None;};
 if at!=att.len()||m.len()!=3||map_at(&m,Cbor::Text("fmt".into()))!=Some(&Cbor::Text("none".into()))||map_at(&m,Cbor::Text("attStmt".into()))!=Some(&Cbor::Map(Vec::new())) {return None;}
 let Cbor::Bytes(auth)=map_at(&m,Cbor::Text("authData".into()))? else{return None;};
 let(counter,backed)=auth_head(auth,rp,true)?;let len=usize::from(u16::from_be_bytes(auth.get(53..55)?.try_into().ok()?));
 if len==0||len>1024||auth.get(55..55+len)?!=id{return None;}
 let mut p=55+len;let Cbor::Map(key)=cbor(auth,&mut p,0)? else{return None;};
 if p!=auth.len()||key.len()!=5||map_at(&key,Cbor::Int(1))!=Some(&Cbor::Int(2))||map_at(&key,Cbor::Int(3))!=Some(&Cbor::Int(-7))||map_at(&key,Cbor::Int(-1))!=Some(&Cbor::Int(1)){return None;}
 let(Cbor::Bytes(x),Cbor::Bytes(y))=(map_at(&key,Cbor::Int(-2))?,map_at(&key,Cbor::Int(-3))?)else{return None;};
 if x.len()!=32||y.len()!=32{return None;}
 let mut sec=vec![4];sec.extend(x);sec.extend(y);VerifyingKey::from_sec1_bytes(&sec).ok()?;
 Some((sec,counter,backed))
}
fn assertion(key:&[u8],auth:&[u8],client:&[u8],sig:&[u8],rp:&str,old:u32,backed:bool)->Option<u32>{
 let(counter,now_backed)=auth_head(auth,rp,false)?;
 if now_backed!=backed||((old!=0||counter!=0)&&counter<=old){return None;}
 let key=VerifyingKey::from_sec1_bytes(key).ok()?;let sig=Signature::from_der(sig).ok()?;
 let mut signed=auth.to_vec();signed.extend(Sha256::digest(client));key.verify(&signed,&sig).ok()?;Some(counter)
}
fn handle(account:i64)->String{b64(&account.to_be_bytes())}
fn descriptors(base:&Connection,account:i64)->String{
 let Ok(mut q)=base.prepare("SELECT id FROM passkeys WHERE account=?1 ORDER BY created,id")else{return "[]".into()};
 let Ok(rows)=q.query_map(params![account],|r|r.get::<_,String>(0))else{return "[]".into()};
 format!("[{}]",rows.filter_map(Result::ok).map(|id|format!("{{\"type\":\"public-key\",\"id\":{}}}",quote(&id))).collect::<Vec<_>>().join(","))
}
fn challenge(base:&Connection,ask:&Ask,account:i64,op:&str,origin:&str,now:u64)->Result<(String,String),String>{
 base.execute("DELETE FROM passkey_challenges WHERE created<?1",params![now.saturating_sub(300)as i64]).map_err(|e|e.to_string())?;
 let total:i64=base.query_row("SELECT COUNT(*) FROM passkey_challenges",[],|r|r.get(0)).map_err(|e|e.to_string())?;
 let token=match cookie(ask.cookie){Some(t)=>t.into(),None=>hex(&random(16)?)};
 if total>=MAX_CHALLENGES{return Err("Trop de demandes en attente.".into());}
 let drawn=b64(&random(32)?);
 base.execute("INSERT INTO passkey_challenges(session,account,challenge,operation,origin,bound,created)VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(session)DO UPDATE SET account=excluded.account,challenge=excluded.challenge,operation=excluded.operation,origin=excluded.origin,bound=excluded.bound,created=excluded.created",params![session(&token),account,drawn,op,origin,if op=="create"{accounts::session_binding(ask.cookie).unwrap_or_default()}else{String::new()},now as i64]).map_err(|e|e.to_string())?;
 let secure=if origin.starts_with("https://"){"; Secure"}else{""};
 Ok((drawn,format!("{COOKIE}={token}; Path=/account/passkeys; HttpOnly; SameSite=Strict; Max-Age=300{secure}")))
}
fn consume(base:&Connection,cookie_header:&str,op:&str,origin:&str,now:u64)->Option<(i64,String)>{
 let token=cookie(cookie_header)?;let key=session(token);
 let row:Option<(i64,String,String,String,String,i64)>=base.query_row("SELECT account,challenge,operation,origin,bound,created FROM passkey_challenges WHERE session=?1",params![key],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().ok()?;
 // Même la réponse refusée consomme son défi : jamais deux essais de signature sur un nonce.
 base.execute("DELETE FROM passkey_challenges WHERE session=?1",params![key]).ok()?;
 let(account,challenge,known,issued,bound,created)=row?;(known==op&&issued==origin&&(op!="create"||accounts::session_binding(cookie_header).as_deref()==Some(bound.as_str()))&&now as i64>=created&&now.saturating_sub(created as u64)<=300).then_some((account,challenge))
}

pub fn answer(site:&Site,ask:&Ask,path:&str)->Option<Reply>{
 if path!="/account/passkeys"&&!path.starts_with("/account/passkeys/"){return None;}
 if path=="/account/passkeys/script.js"&&matches!(ask.method,"GET"|"HEAD"){
  let mut h=accounts::private_headers();h.push(("Content-Type".into(),"text/javascript; charset=utf-8".into()));
  return Some(Reply{status:200,headers:h,body:include_bytes!("../web/passkeys.js").to_vec()});
 }
 let origin=origin_for(site,ask);let member=accounts::member_of(site,ask.cookie);
 if matches!(ask.method,"GET"|"HEAD")&&path=="/account/passkeys"{return Some(match &member{Some(m)=>keys_page(site,m,origin.is_some()),None=>login_page(origin.is_some())});}
 if ask.method!="POST"{return Some(error(405,"Cette demande attend un bouton POST."));}
 let Some(origin)=origin else{return Some(error(403,"Clés d'accès : HTTPS chez l'auteur, ou localhost sur ce PC, requis."));};
 if ask.origin!=origin{return Some(error(403,"L'origine ne correspond pas au site."));}
 if ask.body.len()>MAX_BODY{return Some(error(413,"Réponse de clé trop lourde."));}
 let now=crate::server::now();let Ok(mut base)=site.base.lock()else{return Some(error(500,"Base indisponible."))};
 if !accounts::ip_allowed(&base,&crate::server::client_address(site,ask),now){return Some(error(429,"Trop de demandes : attends une minute."));}
 if path=="/account/passkeys/remove"{
  if !ask.content_type.starts_with("application/x-www-form-urlencoded"){return Some(error(415,"Formulaire attendu."));}
  let fields=crate::gestures::read_form(&String::from_utf8_lossy(ask.body));let get=|k:&str|fields.iter().find(|(n,_)|n==k).map_or("",|(_,v)|v.as_str());
  let Some(m)=member.as_ref()else{return Some(error(401,"Connecte-toi d'abord."));};
  if !accounts::reauthenticate(&base,m,get("password"),get("code"),now){return Some(error(401,"La confirmation du compte ne va pas."));}
  if base.execute("DELETE FROM passkeys WHERE id=?1 AND account=?2",params![get("id"),m.id]).is_err(){return Some(error(500,"Base indisponible."));}
  return Some(accounts::redirect("/account/passkeys",&[]));
 }
 if !ask.content_type.starts_with("application/json"){return Some(error(415,"Objet JSON attendu."));}
 let Some(fields)=std::str::from_utf8(ask.body).ok().and_then(flat_json)else{return Some(error(400,"Réponse illisible ou clés répétées."));};
 let get=|k:&str|field(&fields,k).unwrap_or("");
 let (rp,_)=origin_parts(&origin).expect("origine validée");
 match path{
  "/account/passkeys/register/options"=>{
   let Some(m)=member.as_ref()else{return Some(error(401,"Connecte-toi d'abord."));};
   if !accounts::reauthenticate(&base,m,get("password"),get("code"),now){return Some(error(401,"Le mot de passe ou le code de confirmation ne va pas."));}
   let count:i64=base.query_row("SELECT COUNT(*) FROM passkeys WHERE account=?1",params![m.id],|r|r.get(0)).unwrap_or(MAX_KEYS);
   if count>=MAX_KEYS{return Some(error(409,"Huit clés au plus : retire une ancienne clé."));}
   let Ok((nonce,c))=challenge(&base,ask,m.id,"create",&origin,now)else{return Some(error(503,"Défi indisponible."))};
   Some(json(200,&format!("{{\"challenge\":{},\"rp\":{{\"id\":{},\"name\":\"HoloCode chez l'auteur\"}},\"user\":{{\"id\":{},\"name\":{},\"displayName\":{}}},\"pubKeyCredParams\":[{{\"type\":\"public-key\",\"alg\":-7}}],\"timeout\":60000,\"attestation\":\"none\",\"authenticatorSelection\":{{\"residentKey\":\"required\",\"userVerification\":\"required\"}},\"excludeCredentials\":{}}}",quote(&nonce),quote(rp),quote(&handle(m.id)),quote(&m.name),quote(&m.name),descriptors(&base,m.id)),Some(c)))
  },
  "/account/passkeys/login/options"=>{
   // La liste réelle des identifiants n'est pas livrée à une personne non connectée.
   // Les clés découvrables du navigateur trouvent le compte, confirmé ensuite par userHandle.
   let Ok((nonce,c))=challenge(&base,ask,0,"get",&origin,now)else{return Some(error(503,"Défi indisponible."))};
   Some(json(200,&format!("{{\"challenge\":{},\"rpId\":{},\"timeout\":60000,\"userVerification\":\"required\"}}",quote(&nonce),quote(rp)),Some(c)))
  },
  "/account/passkeys/register/finish"|"/account/passkeys/login/finish"=>{
   let register=path.contains("/register/");let op=if register{"create"}else{"get"};
   let Some((account,nonce))=consume(&base,ask.cookie,op,&origin,now)else{return Some(error(401,"Défi oublié, déjà utilisé ou d'un autre geste."));};
   let Some(id)=un64(get("id"),1024).filter(|b|!b.is_empty())else{return Some(error(400,"Identifiant de clé illisible."));};
   let Some(client)=un64(get("clientDataJSON"),4096)else{return Some(error(400,"Réponse du navigateur illisible."));};
   let ty=if register{"webauthn.create"}else{"webauthn.get"};
   if !client_data(&client,&nonce,&origin,ty){return Some(error(401,"Défi, origine ou type de réponse incorrect."));}
   if register{
    let Some(m)=member.as_ref().filter(|m|m.id==account)else{return Some(error(401,"Ce défi n'appartient pas à ce compte."));};
    let Some(att)=un64(get("attestationObject"),8192)else{return Some(error(400,"Attestation illisible."));};
    let Some((key,counter,backed))=registration(&att,&id,rp)else{return Some(error(401,"Clé ES256, présence et vérification de l'utilisateur requises."));};
    let label=get("label").trim();if label.is_empty()||label.chars().count()>60||label.chars().any(char::is_control){return Some(error(400,"Nom de clé : 1 à 60 caractères."));}
    let count:i64=base.query_row("SELECT COUNT(*) FROM passkeys WHERE account=?1",params![m.id],|r|r.get(0)).unwrap_or(MAX_KEYS);
    if count>=MAX_KEYS{return Some(error(409,"Huit clés au plus."));}
    if base.execute("INSERT INTO passkeys(id,account,key,counter,backed,label,created)SELECT ?1,?2,?3,?4,?5,?6,?7 WHERE EXISTS(SELECT 1 FROM accounts WHERE id=?2)",params![b64(&id),m.id,key,i64::from(counter),i64::from(backed),label,now as i64]).ok()!=Some(1){return Some(error(409,"Clé déjà connue ou compte retiré."));}
    Some(json(200,"{\"next\":\"/account/passkeys\"}",None))
   }else{
    let row:Option<(i64,Vec<u8>,u32,bool)>=base.query_row("SELECT p.account,p.key,p.counter,p.backed FROM passkeys p JOIN accounts a ON a.id=p.account WHERE p.id=?1",params![b64(&id)],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().ok().flatten();
    let Some((account,key,old,backed))=row else{return Some(error(401,"Clé ou signature incorrecte."));};
    let Some(user)=un64(get("userHandle"),64)else{return Some(error(401,"Compte de la clé absent."));};
    if user!=account.to_be_bytes(){return Some(error(401,"Cette clé n'appartient pas au compte annoncé."));}
    let (Some(auth),Some(sig))=(un64(get("authenticatorData"),8192),un64(get("signature"),128))else{return Some(error(400,"Signature illisible."));};
    let Some(counter)=assertion(&key,&auth,&client,&sig,rp,old,backed)else{return Some(error(401,"Signature, présence, vérification ou compteur incorrect."));};
    let Ok(tx)=base.transaction()else{return Some(error(500,"Base indisponible."));};
    if tx.execute("UPDATE passkeys SET counter=?1 WHERE id=?2 AND counter=?3",params![counter,b64(&id),old]).ok()!=Some(1){return Some(error(409,"Cette clé a changé : recommence."));}
    let Some(mut c)=accounts::open_session(&tx,ask.cookie,account,false,now)else{return Some(error(500,"Session impossible."));};
    accounts::bring_visits(&tx,ask.cookie,account);if tx.commit().is_err(){return Some(error(500,"Session impossible."));}
    if origin.starts_with("https://"){c.push_str("; Secure");}
    Some(json(200,"{\"next\":\"/account\"}",Some(c)))
   }
  },
  _=>Some(error(404,"Aucune opération de clé à cette adresse.")),
 }
}
fn webpage(title:&str,main:&str)->Reply{
 let mut r=accounts::page(200,title,main,&[]);
 for(n,v)in &mut r.headers{if n=="Content-Security-Policy"{*v="default-src 'none'; script-src 'self'; connect-src 'self'; style-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'".into();}}
 r
}
fn warning()->&'static str{"<p role=\"alert\">Pour une clé d'accès sur téléphone, ouvre ce site en HTTPS. Sur ce PC, ouvre localhost. Le mot de passe reste disponible.</p>"}
fn controls(register:bool)->String{
 let action=if register{"register"}else{"login"};let label=if register{"Ajouter une clé d'accès"}else{"Se connecter par une clé d'accès"};
 let extra=if register{"<label for=\"label\">Nom de cette clé</label><input id=\"label\" name=\"label\" maxlength=\"60\" value=\"Cet appareil\" required><label for=\"password\">Mot de passe de confirmation</label><input id=\"password\" name=\"password\" type=\"password\" autocomplete=\"current-password\" required><label for=\"code\">Code de l'application ou de secours, si activé</label><input id=\"code\" name=\"code\" autocomplete=\"one-time-code\" maxlength=\"40\">"}else{""};
 format!("<form id=\"passkey-form\" data-operation=\"{action}\">{extra}<button id=\"passkey-button\" type=\"submit\">{label}</button></form><p id=\"passkey-status\" role=\"status\" aria-live=\"polite\"></p><noscript><p>Une clé d'accès demande JavaScript et la fonction sécurisée du navigateur. <a href=\"/account/signin\">Le mot de passe fonctionne sans JavaScript.</a></p></noscript><script src=\"/account/passkeys/script.js\" defer></script>")
}
fn login_page(available:bool)->Reply{webpage("Une clé d'accès",&format!("<h1>Se connecter par une clé d'accès</h1><p>Ton appareil vérifie ton identité. Ce site ne reçoit ni ton empreinte digitale ni ton code de déverrouillage.</p>{}{}<p><a href=\"/account/signin\">Se connecter par mot de passe</a></p>",if available{""}else{warning()},controls(false)))}
fn keys_page(site:&Site,m:&Member,available:bool)->Reply{
 let mut rows=String::new();if let Ok(base)=site.base.lock(){if let Ok(mut q)=base.prepare("SELECT id,label FROM passkeys WHERE account=?1 ORDER BY created,id"){if let Ok(keys)=q.query_map(params![m.id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))){for(id,label)in keys.flatten(){rows.push_str(&format!("<li data-passkey=\"{}\">{}<form method=\"post\" action=\"/account/passkeys/remove\"><input type=\"hidden\" name=\"id\" value=\"{}\"><label>Mot de passe pour retirer cette clé<input name=\"password\" type=\"password\" autocomplete=\"current-password\" required></label><label>Code, si activé<input name=\"code\" autocomplete=\"one-time-code\" maxlength=\"40\"></label><button type=\"submit\">Retirer cette clé</button></form></li>",escape(&id),escape(&label),escape(&id)));}}}}
 webpage("Mes clés d'accès",&format!("<h1>Mes clés d'accès</h1><p>Compte : {}. Huit clés au plus. Garde ton mot de passe et tes secours pour un appareil perdu.</p><ul>{rows}</ul>{}{}<p><a href=\"/account\">Mon compte</a></p>",escape(&m.name),if available{""}else{warning()},controls(true)))
}

#[cfg(test)]
mod tests{
 use super::*;
 use p256::ecdsa::{signature::Signer,SigningKey};
 fn auth(rp:&str,flags:u8,count:u32)->Vec<u8>{let mut b=Sha256::digest(rp.as_bytes()).to_vec();b.push(flags);b.extend(count.to_be_bytes());b}
 #[test]fn a_real_es256_signature_and_each_rejection(){
  let signing=SigningKey::from_bytes((&[7u8;32]).into()).unwrap();let key=signing.verifying_key().to_encoded_point(false);
  let client=b"{\"type\":\"webauthn.get\",\"challenge\":\"nonce\",\"origin\":\"http://localhost:8080\",\"crossOrigin\":false}";
  let a=auth("localhost",5,2);let mut data=a.clone();data.extend(Sha256::digest(client));
  let sig:Signature=signing.sign(&data);let der=sig.to_der();
  assert_eq!(assertion(key.as_bytes(),&a,client,der.as_bytes(),"localhost",1,false),Some(2));
  assert_eq!(assertion(key.as_bytes(),&a,client,der.as_bytes(),"elsewhere",1,false),None);
  assert_eq!(assertion(key.as_bytes(),&a,client,der.as_bytes(),"localhost",2,false),None);
  for flags in [1,4,7,5|0x40,5|0x80,5|0x10]{assert_eq!(assertion(key.as_bytes(),&auth("localhost",flags,2),client,der.as_bytes(),"localhost",1,false),None);}
  let mut corrupted=der.as_bytes().to_vec();let last=corrupted.len()-1;corrupted[last]^=1;
  assert_eq!(assertion(key.as_bytes(),&a,client,&corrupted,"localhost",1,false),None);
  assert_eq!(assertion(key.as_bytes(),&a,b"changed",der.as_bytes(),"localhost",1,false),None);
  let zero=auth("localhost",5|8,0);let mut input=zero.clone();input.extend(Sha256::digest(client));let signed:Signature=signing.sign(&input);
  assert_eq!(assertion(key.as_bytes(),&zero,client,signed.to_der().as_bytes(),"localhost",0,true),Some(0));
  assert_eq!(assertion(key.as_bytes(),&zero,client,signed.to_der().as_bytes(),"localhost",1,true),None);
 }
 #[test]fn client_data_is_exact_and_json_has_no_ambiguous_keys(){
  let good=br#"{"type":"webauthn.get","challenge":"n","origin":"http://localhost","crossOrigin":false}"#;
  assert!(client_data(good,"n","http://localhost","webauthn.get"));
  for value in [
   r#"{"type":"webauthn.get","challenge":"n","challenge":"n","origin":"http://localhost"}"#,
   r#"{"type":"webauthn.get","challenge":"n","origin":"http://localhost","crossOrigin":0}"#,
   r#"{"type":"webauthn.get","challenge":"n","origin":"http://localhost","crossOrigin":true}"#,
   r#"{"type":"webauthn.get","challenge":"n","origin":"http://localhost","topOrigin":"http://localhost"}"#,
   r#"{"type":"webauthn.get","challenge":"n","origin":"http://localhost",}"#] {assert!(!client_data(value.as_bytes(),"n","http://localhost","webauthn.get"));}
  assert!(!client_data(good,"other","http://localhost","webauthn.get"));
  assert!(!client_data(good,"n","https://localhost","webauthn.get"));
  assert!(!client_data(good,"n","http://localhost","webauthn.create"));
  for size in 0..100{let b=(0..size).map(|i|i as u8).collect::<Vec<_>>();assert_eq!(un64(&b64(&b),1024),Some(b));}
  assert_eq!(un64("Zh",10),None);assert_eq!(un64("Zg==",10),None);
 }
 fn bytes(b:&[u8])->Vec<u8>{let mut v=if b.len()<24{vec![0x40|b.len()as u8]}else if b.len()<256{vec![0x58,b.len()as u8]}else{let mut v=vec![0x59];v.extend((b.len()as u16).to_be_bytes());v};v.extend(b);v}
 fn attestation(a:&[u8])->Vec<u8>{let mut v=b"\xa3\x63fmt\x64none\x68authData".to_vec();v.extend(bytes(a));v.extend(b"\x67attStmt\xa0");v}
 #[test]fn registration_checks_cose_identity_curve_and_flags(){
  let signing=SigningKey::from_bytes((&[9u8;32]).into()).unwrap();let key=signing.verifying_key().to_encoded_point(false);
  let id=b"stable-credential";let mut a=auth("localhost",0x45,0);a.extend([0u8;16]);a.extend((id.len()as u16).to_be_bytes());a.extend(id);
  a.extend([0xa5,1,2,3,0x26,0x20,1,0x21]);a.extend(bytes(key.x().unwrap()));a.push(0x22);a.extend(bytes(key.y().unwrap()));
  assert_eq!(registration(&attestation(&a),id,"localhost"),Some((key.as_bytes().to_vec(),0,false)));
  assert_eq!(registration(&attestation(&a),b"wrong","localhost"),None);
  assert_eq!(registration(&attestation(&a),id,"elsewhere"),None);
  a[32]=0x41;assert_eq!(registration(&attestation(&a),id,"localhost"),None);
  a[32]=0x45;a.push(0);assert_eq!(registration(&attestation(&a),id,"localhost"),None);
  let mut at=0;assert_eq!(cbor(&[0xbf,0xff],&mut at,0),None);
  let mut at=0;assert_eq!(cbor(&[0xa2,1,2,1,3],&mut at,0),None);
 }
 #[test]fn a_challenge_is_single_use_bound_and_expiring(){
  let base=Connection::open_in_memory().unwrap();prepare(&base,1000).unwrap();
  let token="0123456789abcdef0123456789abcdef";let header=format!("{COOKIE}={token}");
  base.execute("INSERT INTO passkey_challenges VALUES(?1,2,'nonce','get','http://localhost','',1000)",params![session(token)]).unwrap();
  assert_eq!(consume(&base,&header,"get","http://localhost",1001),Some((2,"nonce".into())));
  assert_eq!(consume(&base,&header,"get","http://localhost",1001),None);
  for (op,origin,at)in[("create","http://localhost",1001),("get","https://localhost",1001),("get","http://localhost",1301)]{
   base.execute("INSERT INTO passkey_challenges VALUES(?1,2,'nonce','get','http://localhost','',1000)",params![session(token)]).unwrap();
   assert_eq!(consume(&base,&header,op,origin,at),None);assert_eq!(consume(&base,&header,"get","http://localhost",1001),None);
  }
  base.execute("INSERT INTO passkey_challenges VALUES(?1,2,'nonce','create','http://localhost','another-session',1000)",params![session(token)]).unwrap();
  assert_eq!(consume(&base,&header,"create","http://localhost",1001),None);
 }
}
