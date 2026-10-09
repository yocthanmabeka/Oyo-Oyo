//! Ce qu'une page sait du visiteur connecté, et qui peut la voir (ADR-081, lot 7 du web).
//!
//! - `Page(access: members)` : la page n'est montrée qu'aux personnes connectées. Les autres sont
//!   menées à la page « Se connecter », puis ramenées ici.
//! - `signedIn` (1 quand le visiteur est connecté, 0 sinon) et `{account}` (son nom, vide sinon) :
//!   la page les lit comme ses autres valeurs, sans jamais pouvoir les changer. C'est le serveur qui
//!   les donne, comme il donne l'heure.
//!
//! Le nom du membre arrive au moteur comme les valeurs d'une adresse (ADR-078) : un petit fichier
//! joint après la page, nommé `@account`, qui contient `name=Ada`. `holo serve` le joint, le moteur
//! de la page le joint aussi (le serveur lui a donné le nom dans l'en-tête) ; partout ailleurs
//! (`holo check`, le serveur d'essai, l'éditeur), rien n'est joint : la page se lit comme pour un
//! visiteur qui n'est pas connecté. Les comptes eux-mêmes (mots de passe, code à 6 chiffres,
//! sessions) sont dans `accounts.rs`, sur le PC seulement.

use crate::holo::{Argument, Block, Error, Program, Value, NAME_SEPARATOR, NEXT_FILE};

/// Le nom du fichier joint qui porte le nom du membre connecté.
pub const ACCOUNT_FILE: &str = "@account";

/// Le nom du membre connecté, un texte ; vide pour un visiteur qui ne l'est pas.
pub const NAME: &str = "account";
/// 1 quand le visiteur est connecté, 0 sinon.
pub const SIGNED_IN: &str = "signedIn";
/// Les deux valeurs que le serveur donne : on les lit, on ne les change pas.
pub const GIVEN: &[&str] = &[NAME, SIGNED_IN];

/// Les pages du serveur où un lien peut mener : les seuls liens qui partent de la racine du site
/// (`A(to: "/account")`). Ce sont les pages de holo serve, pas celles du site : elles sont toujours
/// à la racine, où que soit rangée la page qui y mène.
pub const LINKS: &[&str] = &["/account", "/account/signin", "/account/signup"];

/// Un nom de compte : de 3 à 30 caractères, des lettres sans accent, des chiffres, « . », « - »
/// et « _ », une lettre ou un chiffre d'abord. Sans accent : deux écritures d'un « é » (une lettre,
/// ou un « e » suivi d'un accent) se ressemblent sans être le même nom.
pub fn valid_name(name: &str) -> bool {
    (3..=30).contains(&name.len())
        && name.starts_with(|c: char| c.is_ascii_alphanumeric())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// Le texte complet pour le moteur : la page (et ses imports, et son adresse), puis le nom du
/// membre connecté, vide pour un visiteur qui ne l'est pas.
pub fn joined(source: &str, name: &str) -> String {
    format!("{source}{NEXT_FILE}{ACCOUNT_FILE}{NAME_SEPARATOR}name={name}")
}

/// La page est-elle réservée aux membres (`access: members`) ?
pub fn members_only(program: &Program) -> bool {
    matches!(program.root.argument("access").map(|a| &a.value), Some(Value::Name(access)) if access == "members")
}

/// Pour un fichier que le moteur refuse (une faute de frappe ailleurs dans la page) : le texte
/// dit-il `access: members` ? Le serveur ne montre pas une page cassée qui se dit réservée.
pub fn members_in_text(source: &str) -> bool {
    let main = source.split(NEXT_FILE).next().unwrap_or("");
    main.lines().map(|line| line.split("//").next().unwrap_or("")).any(|line| {
        let mut rest = line;
        while let Some(at) = rest.find("access") {
            let after = rest[at + "access".len()..].trim_start();
            if after.strip_prefix(':').is_some_and(|value| value.trim_start().starts_with("members")) {
                return true;
            }
            rest = &rest[at + "access".len()..];
        }
        false
    })
}

/// Les valeurs du visiteur que la page lit : dans un texte (`{account}`), une condition
/// (`If(signedIn, is: 1)`), une comparaison, une demande (`best.set(signedIn)`).
fn read_ones(program: &Program) -> Vec<&'static str> {
    fn record(name: &str, found: &mut Vec<&'static str>) {
        if let Some(given) = GIVEN.iter().find(|g| **g == name) {
            if !found.contains(given) {
                found.push(given);
            }
        }
    }
    fn visit(value: &Value, found: &mut Vec<&'static str>) {
        match value {
            Value::Text(text) => crate::state::names_in(text).into_iter().for_each(|name| record(name, found)),
            Value::Name(name) => record(name, found),
            Value::List(elements) => elements.iter().for_each(|e| visit(e, found)),
            Value::Block(block) => block.arguments.iter().for_each(|a| visit(&a.value, found)),
            _ => {}
        }
    }
    let mut found = Vec::new();
    program.root.arguments.iter().for_each(|a| visit(&a.value, &mut found));
    found
}

/// Pose dans la page ce qu'elle sait du visiteur, si elle le lit : `account` (un texte) et
/// `signedIn` (un nombre), comme si elle les avait déclarés dans `State`. `given` : le texte du
/// fichier `@account` (`name=Ada`), ou rien pour un visiteur qui n'est pas connecté.
///
/// Refusé, avec la raison : `access:` autre que `everyone` ou `members` ; déclarer ces valeurs
/// dans `State` ; les changer (une demande, un champ, un glissement, un module) ; les garder
/// (`keep`).
pub fn inject(program: &mut Program, given: Option<&str>) -> Result<(), Error> {
    if let Some(argument) = program.root.argument("access") {
        let known = matches!(&argument.value, Value::Name(access) if access == "members" || access == "everyone");
        if !known {
            return Err(Error { message: "« access » dit qui voit la page : access: members (les personnes connectées), ou access: everyone (tout le monde, sans rien écrire)".into(), pos: argument.pos });
        }
    }
    // Une valeur du visiteur se lit ; elle ne se change pas, ne se garde pas (`keep`), et ne vient
    // pas de l'adresse de la page (`address:`, ADR-091).
    for (setting, why) in [("keep", "il ne se garde pas"), ("address", "l'adresse ne le porte pas")] {
        if let Some(argument) = program.root.argument(setting) {
            if let Value::List(names) = &argument.value {
                if let Some(Value::Name(given)) = names.iter().find(|n| matches!(n, Value::Name(n) if GIVEN.contains(&n.as_str()))) {
                    return Err(Error { message: format!("« {setting} » : « {given} » est donné par le serveur à chaque visite, {why}"), pos: argument.pos });
                }
            }
        }
    }
    crate::rules::for_each_block(&program.root, &mut |block| {
        let changed = match block.name.split_once('.') {
            Some((target, _)) if crate::state::is_requested(block) => Some(target.to_string()),
            _ if ["Input", "Checkbox", "Choice", "Slider"].contains(&block.name.as_str()) => match block.argument("value").map(|a| &a.value) {
                Some(Value::Name(bound)) => Some(bound.clone()),
                _ => None,
            },
            // Un module rend son nombre dans une valeur ; un bloc qu'on glisse change sa place.
            _ if block.name == "Module" => match block.argument("output").map(|a| &a.value) {
                Some(Value::Name(output)) => Some(output.clone()),
                _ => None,
            },
            _ if matches!(block.argument("drag").map(|a| &a.value), Some(Value::Bool(true))) => ["x", "y"].iter().find_map(|axis| match block.argument(axis).map(|a| &a.value) {
                Some(Value::Name(place)) if GIVEN.contains(&place.as_str()) => Some(place.clone()),
                _ => None,
            }),
            _ => None,
        };
        match changed {
            Some(name) if GIVEN.contains(&name.as_str()) => Err(Error { message: format!("« {name} » est donné par le serveur (le visiteur connecté) : on le lit, on ne le change pas"), pos: block.pos }),
            _ => Ok(()),
        }
    })?;
    let read = read_ones(program);
    if read.is_empty() {
        return Ok(());
    }
    if program.root.name != "Page" {
        return Err(Error { message: format!("« {} » est donné par le serveur à une page : il se lit dans Page(…)", read[0]), pos: program.root.pos });
    }
    // Le nom du membre : `name=Ada`, encodé comme un morceau d'adresse. Un nom qui n'en est pas un
    // (un en-tête abîmé) compte comme un visiteur qui n'est pas connecté.
    let name = given
        .and_then(|text| text.split('&').find_map(|pair| pair.strip_prefix("name=")))
        .and_then(crate::address::decode)
        .filter(|name| valid_name(name))
        .unwrap_or_default();
    let pos = program.root.pos;
    let given_value = |wanted: &str, pos| {
        let value = if wanted == NAME { Value::Text(name.clone()) } else { Value::Integer(u64::from(!name.is_empty())) };
        Argument { name: Some(wanted.to_string()), value, pos }
    };
    match program.root.arguments.iter_mut().find(|a| a.name.as_deref() == Some("state")) {
        Some(Argument { value: Value::Block(state), .. }) if state.name == "State" => {
            for wanted in &read {
                if state.arguments.iter().any(|a| a.name.as_deref() == Some(*wanted)) {
                    return Err(Error { message: format!("« {wanted} » est donné par le serveur (le visiteur connecté) : ne le déclare pas dans State"), pos: state.pos });
                }
                state.arguments.push(given_value(wanted, state.pos));
            }
        }
        Some(_) => return Err(Error { message: "« state » attend un bloc « State(...) », sur la page : state: State(cart: 0)".into(), pos }),
        None => {
            let arguments = read.iter().map(|wanted| given_value(wanted, pos)).collect();
            program.root.arguments.push(Argument { name: Some("state".into()), value: Value::Block(Block { name: "State".into(), styles: Vec::new(), arguments, pos }), pos });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GREETING: &str = "Page(children: [ If(signedIn, is: 1, children: [ H1(\"Bonjour, {account}\") ], else: [ P(\"Tu n'es pas connecté.\") ]) ])";

    #[test]
    fn the_page_knows_who_is_signed_in() {
        // Connecté : le nom dans le texte, la condition vraie.
        let html = crate::flat_view(&joined(GREETING, "Ada"), "").unwrap();
        assert!(html.contains("Bonjour, <span data-state=\"account\">Ada</span>") && html.contains("data-if=\"signedIn|is=1\">"), "{html}");
        // Rien de joint (holo check, le serveur d'essai) ou un nom vide : un visiteur qui n'est pas connecté.
        for source in [GREETING.to_string(), joined(GREETING, "")] {
            let html = crate::flat_view(&source, "").unwrap();
            assert!(html.contains("data-if=\"signedIn|is=1\" hidden") && html.contains("<span data-state=\"account\"></span>"), "{html}");
        }
        // Un nom qui n'en est pas un ne fait pas un membre.
        assert!(crate::flat_view(&joined(GREETING, "a<b"), "").unwrap().contains("data-if=\"signedIn|is=1\" hidden"));
        // L'état du visiteur suit l'arbitre, mais ces deux valeurs ne se relisent jamais d'un état
        // écrit : c'est toujours le serveur qui les donne.
        let state = crate::initial_state(&joined(GREETING, "Ada"));
        assert!(state.contains("signedIn=1") && state.contains("account='Ada"), "{state}");
        let forged = crate::arbitrate(GREETING, "signedIn=1;account='Admin", "Rien.tap");
        assert!(forged.contains("signedIn=0") && forged.contains("account='") && !forged.contains("Admin"), "{forged}");
        // Une page qui ne les lit pas ne les reçoit pas.
        assert_eq!(crate::initial_state(&joined("Page(state: State(cart: 0), children: [ P(\"{cart}\") ])", "Ada")), "cart=0");
    }

    #[test]
    fn the_visitor_values_are_read_never_changed() {
        let refused = |source: &str| crate::check_page(&joined(source, "Ada")).unwrap_err().message;
        assert!(refused("Page(children: [ Button(name: B, text: \"x\") ], rules: [ On(B.tap, effect: signedIn.set(1)) ])").contains("on le lit, on ne le change pas"));
        assert!(refused("Page(children: [ Input(value: account, label: \"Nom\") ])").contains("on le lit, on ne le change pas"));
        assert!(refused("Page(state: State(account: \"\"), children: [ P(\"{account}\") ])").contains("ne le déclare pas dans State"));
        assert!(refused("Page(keep: [signedIn], children: [ P(\"{signedIn}\") ])").contains("ne se garde pas"));
        assert!(refused("Page(address: [account], children: [ P(\"{account}\") ])").contains("l'adresse ne le porte pas"));
        assert!(refused("Page(access: friends, children: [ P(\"x\") ])").contains("access: members"));
        assert!(refused("Point(name: A, seed: 1, children: [ P(\"{account}\") ])").contains("se lit dans Page"));
        // Des données reçues ne le changent pas non plus.
        let page = "Page(state: State(cart: 0), data: Data(from: \"d.json\"), children: [ P(\"{account} {signedIn} {cart}\") ])";
        let received = crate::receive(page, &crate::initial_state(page), "{\"account\": \"Admin\", \"signedIn\": 1, \"cart\": 2}");
        assert!(received.contains("cart=2") && received.contains("signedIn=0") && !received.contains("Admin"), "{received}");
    }

    #[test]
    fn a_page_for_members() {
        let page = |source: &str| crate::holo::read(source).unwrap();
        assert!(members_only(&page("Page(access: members, children: [ P(\"x\") ])")));
        assert!(!members_only(&page("Page(access: everyone, children: [ P(\"x\") ])")));
        assert!(!members_only(&page("Page(children: [ P(\"x\") ])")));
        // Un fichier refusé qui se dit réservé reste réservé ; un commentaire ne compte pas.
        assert!(members_in_text("Page(\n  access:   members,\n  children: [ P(\"x\" ]\n"));
        assert!(!members_in_text("// access: members\nPage(children: [ P(\"access\") ])"));
        // Les pages du serveur sont les seuls liens qui partent de la racine ; les autres restent refusés.
        let html = crate::flat_view("Page(children: [ A(\"Mon compte\", to: \"/account\"), A(\"Se connecter\", to: \"/account/signin\"), A(\"Créer un compte\", to: \"/account/signup\") ])", "/exemples/lecons/").unwrap();
        assert!(html.contains("href=\"/account\"") && html.contains("href=\"/account/signin\"") && html.contains("href=\"/account/signup\""), "{html}");
        for refused in ["/account/code", "/accounts", "/secret.holo", "/account/../secret.holo"] {
            assert!(crate::check_page(&format!("Page(children: [ A(\"x\", to: \"{refused}\") ])")).is_err(), "{refused}");
        }
        assert!(valid_name("Ada") && valid_name("ada.lovelace_1815") && !valid_name("Ad") && !valid_name("Adé") && !valid_name("-ada") && !valid_name(&"a".repeat(31)));
    }
}
