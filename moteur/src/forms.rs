//! Découper une forme (ADR-111) : `form:` dans un style, avec les mots de `Shape(form:)`.
//! Huit formes nommées, chacune une découpe fixe, en pourcentages de la boîte du bloc : elle
//! suit sa taille. Jamais un tracé écrit à la main.
//!
//! Le rond n'est pas une découpe : il arrondit les coins (`border-radius: 50%`), et son bord, son
//! ombre et son cadre de focus suivent sa courbe. Les autres formes sont des polygones
//! (`clip-path`), qui coupent tout ce qui dépasse : le cadre de focus, l'ombre, le bord. Le moteur
//! garde donc le cadre de focus visible (WCAG 2.4.7) : une `Shape` n'est jamais découpée elle-même,
//! sa forme se dessine dans le bouton (`::before`) ; une image ou un dessin découpés se montrent
//! entiers quand ils ont le focus du clavier. Vu dans Chrome : sans cela, une forme qu'on touche,
//! en triangle ou en losange, n'a aucun cadre de focus.

use crate::holo::{Block, Error, Program, Setting, StyleRule, Target, Value};

/// Les formes, dans l'ordre où le moteur les nomme. Les quatre premières sont celles de
/// `ADR-032` ; `square` ne découpe rien (le bloc à angles droits).
pub const FORMS: &[&str] = &["circle", "square", "triangle", "diamond", "hexagon", "star", "heart", "wave"];

/// Les blocs qu'un style découpe : une image, un dessin. Ils ne portent ni texte ni commande ;
/// une forme (`Shape`) porte sa forme sur elle.
pub const CUT: &[&str] = &["Image", "Drawing"];

/// La découpe d'une forme, ou `None` pour le rond et le carré, qui ne découpent pas.
///
/// L'étoile a cinq branches et touche les quatre bords (ses mesures viennent du nombre d'or) ;
/// l'hexagone a deux pointes, à gauche et à droite. Le cœur suit la courbe
/// `x = 16 sin³ t, y = 13 cos t − 5 cos 2t − 2 cos 3t − cos 4t`, en 40 points, sa moitié gauche en
/// miroir de la droite. La vague : le dixième du bas ondule, deux vagues, une crête à chaque bord,
/// `y = 100 % − 10 % × (1 + cos(4π x)) / 2`, en 33 points. Points arrondis au dixième.
pub fn polygon(form: &str) -> Option<&'static str> {
    Some(match form {
        "triangle" => "polygon(50% 0,100% 100%,0 100%)",
        "diamond" => "polygon(50% 0,100% 50%,50% 100%,0 50%)",
        "hexagon" => "polygon(25% 0,75% 0,100% 50%,75% 100%,25% 100%,0 50%)",
        "star" => "polygon(50% 0,61.8% 38.2%,100% 38.2%,69.1% 61.8%,80.9% 100%,50% 76.4%,19.1% 100%,30.9% 61.8%,0 38.2%,38.2% 38.2%)",
        "heart" => "polygon(50% 23.9%,50.2% 22.2%,51.5% 17.6%,54.7% 11.4%,60.2% 5.3%,67.7% 1.1%,76.5% 0.1%,85.4% 2.8%,93% 8.8%,98.2% 17.4%,100% 27.4%,98.2% 37.8%,93% 47.8%,85.4% 57.2%,76.5% 66.1%,67.7% 74.4%,60.2% 82.3%,54.7% 89.3%,51.5% 95%,50.2% 98.7%,50% 100%,49.8% 98.7%,48.5% 95%,45.3% 89.3%,39.8% 82.3%,32.3% 74.4%,23.5% 66.1%,14.6% 57.2%,7% 47.8%,1.8% 37.8%,0 27.4%,1.8% 17.4%,7% 8.8%,14.6% 2.8%,23.5% 0.1%,32.3% 1.1%,39.8% 5.3%,45.3% 11.4%,48.5% 17.6%,49.8% 22.2%)",
        "wave" => "polygon(0 0,100% 0,100% 90%,96.9% 90.4%,93.8% 91.5%,90.6% 93.1%,87.5% 95%,84.4% 96.9%,81.3% 98.5%,78.1% 99.6%,75% 100%,71.9% 99.6%,68.8% 98.5%,65.6% 96.9%,62.5% 95%,59.4% 93.1%,56.3% 91.5%,53.1% 90.4%,50% 90%,46.9% 90.4%,43.8% 91.5%,40.6% 93.1%,37.5% 95%,34.4% 96.9%,31.3% 98.5%,28.1% 99.6%,25% 100%,21.9% 99.6%,18.8% 98.5%,15.6% 96.9%,12.5% 95%,9.4% 93.1%,6.3% 91.5%,3.1% 90.4%,0 90%)",
        _ => return None,
    })
}

/// La déclaration d'une forme dans un style, telle que le navigateur la reçoit. Elle dit toujours
/// la découpe et les coins : une forme écrite dans un style plus précis défait celle d'un style
/// plus général (un hexagone sur une image que `Image { form: circle; }` arrondit).
pub fn declaration(form: &str) -> String {
    match (form, polygon(form)) {
        ("circle", _) => "clip-path:none;border-radius:50%;".into(),
        (_, Some(cut)) => format!("clip-path:{cut};border-radius:0;"),
        _ => "clip-path:none;border-radius:0;".into(),
    }
}

/// Le style découpe-t-il son bloc en polygone, dans ses réglages ou dans l'un de ses états ? Le
/// moteur écrit alors, après les états, que le bloc se montre entier au focus du clavier.
pub fn cuts(rule: &StyleRule) -> bool {
    rule.settings.iter().chain(rule.states.iter().flat_map(|(_, settings, _)| settings.iter())).any(|s| s.name == "form" && polygon(&s.value).is_some())
}

/// Le style de base des formes de `Shape` découpées en polygone, seulement celles de la page :
/// le bouton n'est jamais découpé lui-même, la forme se dessine dedans, de sa couleur. Son cadre
/// de focus se voit donc autour d'elle, et il se touche sur tout son carré, au doigt comme à la
/// souris. `:where` laisse le dernier mot aux styles du fichier.
pub fn shapes_css(program: &Program) -> String {
    let mut used: Vec<&str> = Vec::new();
    let _ = crate::rules::for_each_block(&program.root, &mut |block| {
        if let (true, Some(Value::Name(form))) = (block.name == "Shape", block.argument("form").map(|a| &a.value)) {
            if let Some(known) = FORMS.iter().find(|f| **f == form.as_str() && polygon(f).is_some()) {
                if !used.contains(known) {
                    used.push(known);
                }
            }
        }
        Ok(())
    });
    let mut css = String::new();
    for form in FORMS.iter().filter(|f| used.contains(f)) {
        let cut = polygon(form).unwrap_or_default();
        css.push_str(&format!(
            ":where(.holo-forme-{form}){{background:transparent}}:where(.holo-forme-{form})::before{{content:\"\";display:block;width:100%;height:100%;background:var(--holo-color,currentColor);clip-path:{cut}}}"
        ));
    }
    css
}

/// Une forme découpe une image ou un dessin, ailleurs que sous la souris, le doigt ou le clavier ;
/// les coins sont les siens ; ce que la découpe couperait en silence (une ombre, un bord, le fond
/// d'une `Shape`) est refusé, avec la raison.
pub fn check(rule: &StyleRule, program: &Program) -> Result<(), Error> {
    let refusal = |message: String, pos| Err(Error { message, pos });
    let target = &rule.target;
    // Pas sous la souris ni sous le doigt : au bord, le pointeur tomberait hors de la forme
    // nouvelle, qui rendrait l'ancienne, et le bloc clignoterait. Pas au focus : le moteur montre
    // déjà le bloc entier.
    for (state, settings, _) in &rule.states {
        let Some(setting) = settings.iter().find(|s| s.name == "form") else { continue };
        match state.as_str() {
            "hover" | "active" => {
                return refusal(
                    format!("« form » dans l'état « {state} » de « {target} » : une forme ne change pas sous la souris ni sous le doigt ; au bord, le pointeur tomberait hors de la forme nouvelle, et le bloc clignoterait d'une forme à l'autre (ADR-111)"),
                    setting.pos,
                )
            }
            "focus" => {
                return refusal(
                    format!("« form » dans l'état « focus » de « {target} » : au clavier, le moteur montre déjà le bloc entier quand il a le focus, pour que son cadre de focus se voie (ADR-111)"),
                    setting.pos,
                )
            }
            _ => {}
        }
    }
    // Les coins : la forme les donne déjà (un rond, ou des angles droits).
    let lists = std::iter::once((None, &rule.settings)).chain(rule.states.iter().map(|(state, settings, _)| (Some(state.as_str()), settings)));
    for (state, settings) in lists {
        if let (Some(_), Some(corners)) = (settings.iter().find(|s| s.name == "form"), settings.iter().find(|s| s.name == "border-radius")) {
            let place = state.map_or(String::new(), |s| format!(", dans l'état « {s} »"));
            return refusal(format!("« border-radius » et « form » dans « {target} »{place} : la forme donne déjà les coins du bloc, un rond ou des angles droits ; garde l'un des deux (ADR-111)"), corners.pos);
        }
    }
    let all = || rule.settings.iter().chain(rule.states.iter().flat_map(|(_, settings, _)| settings.iter()));
    // Sur une image ou un dessin seulement.
    if let Some(form) = all().find(|s| s.name == "form") {
        if let Some(what) = outside(rule, program) {
            let message = match what.as_str() {
                "Shape" => format!("« form » dans « {target} » : la forme d'une Shape s'écrit sur elle, Shape(form: star) (ADR-111)"),
                "Video" => format!("« form » dans « {target} » : sur « Video », la découpe couperait aussi ses commandes et ses sous-titres ; une forme découpe une image ou un dessin (ADR-111)"),
                _ => format!(
                    "« form » dans « {target} » : une forme découpe une image ou un dessin (« Image.photo » et « .photo {{ form: hexagon; }} ») ; sur « {what} », elle couperait aussi les textes et les boutons qu'il porte (ADR-111)"
                ),
            };
            return refusal(message, form.pos);
        }
    }
    // Une découpe en polygone coupe l'ombre et le bord : il n'en resterait rien, ou des morceaux.
    // Le rond les garde : ils suivent sa courbe.
    if cuts(rule) {
        if let Some(lost) = all().find(|s| is_cut_away(&s.name, &s.value)) {
            return refusal(
                format!("« {} » et une forme découpée dans « {target} » : la découpe couperait {} ; garde le rond (form: circle), dont {} suit la courbe (ADR-111)", lost.name, lost_one(&lost.name), lost_kept(&lost.name)),
                lost.pos,
            );
        }
    }
    // Donnés au même bloc par un autre style : le bloc que ce style découpe (sa forme est celle de ce
    // style, le plus fort de ceux qui en donnent une) ne doit pas recevoir d'ombre ni de bord.
    for block in blocks_of(rule, program) {
        let rules = rules_for(block, program);
        let Some((owner, form)) = effective(&rules, "form") else { continue };
        if !std::ptr::eq(owner, rule) || polygon(&form.value).is_none() {
            continue;
        }
        for name in ["box-shadow", "border"] {
            let Some((giver, lost)) = effective(&rules, name).filter(|(giver, s)| s.value != "none" && !std::ptr::eq(*giver, rule)) else { continue };
            let stronger = rules.iter().position(|r| std::ptr::eq(*r, giver)) > rules.iter().position(|r| std::ptr::eq(*r, rule));
            let remedy = if stronger { format!("retire « {name} » de « {} »", giver.target) } else { format!("écris « {name}: none; » dans « {target} »") };
            return refusal(
                format!(
                    "« {target} » découpe « {} » en « {} », qui reçoit aussi « {name} » de « {} » : la découpe couperait {} ; {remedy}, ou garde le rond (form: circle), dont {} suit la courbe (ADR-111)",
                    block.name,
                    form.value,
                    giver.target,
                    lost_one(name),
                    lost_kept(name)
                ),
                lost.pos,
            );
        }
    }
    // Une Shape découpée se dessine dans son bouton : un fond, une ombre ou un bord rempliraient
    // ou entoureraient tout son carré, derrière la forme.
    for block in blocks_of(rule, program) {
        let polygonal = block.name == "Shape" && matches!(block.argument("form").map(|a| &a.value), Some(Value::Name(f)) if polygon(f).is_some());
        if !polygonal {
            continue;
        }
        if let Some(square) = all().find(|s| s.name == "background" || is_cut_away(&s.name, &s.value)) {
            let form = match block.argument("form").map(|a| &a.value) {
                Some(Value::Name(f)) => f.as_str(),
                _ => "",
            };
            let message = if square.name == "background" {
                format!("« background » dans « {target} » : la couleur d'une forme s'écrit sur elle, Shape(form: {form}, color: \"#E9B44C\") ; un fond remplirait tout son carré, derrière la forme (ADR-111)")
            } else {
                let what = lost_kept(&square.name);
                format!("« {} » dans « {target} » : une Shape en « {form} » se dessine dans son carré ; {what} entourerait le carré, pas la forme ; garde le rond (form: circle), dont {what} suit la courbe (ADR-111)", square.name)
            };
            return refusal(message, square.pos);
        }
    }
    Ok(())
}

/// Une ombre ou un bord, que la découpe couperait (`none` n'en est pas un).
fn is_cut_away(name: &str, value: &str) -> bool {
    (name == "box-shadow" || name == "border") && value != "none"
}

fn lost_one(name: &str) -> &'static str {
    if name == "box-shadow" { "l'ombre : il n'en resterait rien" } else { "le bord : il n'en resterait que des morceaux" }
}

fn lost_kept(name: &str) -> &'static str {
    if name == "box-shadow" { "l'ombre" } else { "le bord" }
}

/// Le style vaut-il pour ce bloc : par son type, ou par l'un de ses noms de style ?
fn applies(rule: &StyleRule, block: &Block) -> bool {
    match &rule.target {
        Target::Type(t) => &block.name == t,
        Target::Name(n) => block.styles.iter().any(|s| s == n),
    }
}

/// Les styles qui valent pour ce bloc, du plus faible au plus fort, dans l'ordre où la page les
/// écrit (`flat::css`) : le thème, le type, puis les noms, dans l'ordre du fichier.
fn rules_for<'a>(block: &Block, program: &'a Program) -> Vec<&'a StyleRule> {
    let rank = |target: &Target| match target {
        Target::Type(t) if t == "Page" || t == "World" => 0,
        Target::Type(_) => 1,
        Target::Name(_) => 2,
    };
    let mut rules: Vec<&StyleRule> = program.styles.iter().filter(|r| applies(r, block)).collect();
    rules.sort_by_key(|r| rank(&r.target));
    rules
}

/// Ce que le bloc reçoit pour ce réglage, hors états : le style le plus fort qui le donne, et sa valeur.
fn effective<'a>(rules: &[&'a StyleRule], name: &str) -> Option<(&'a StyleRule, &'a Setting)> {
    rules.iter().rev().find_map(|rule| rule.settings.iter().find(|s| s.name == name).map(|s| (*rule, s)))
}

/// Les blocs que le style touche, dans l'ordre du fichier.
fn blocks_of<'a>(rule: &StyleRule, program: &'a Program) -> Vec<&'a Block> {
    let mut found = Vec::new();
    let _ = crate::rules::for_each_block(&program.root, &mut |block| {
        if applies(rule, block) {
            found.push(block);
        }
        Ok(())
    });
    found
}

/// Le premier type de bloc visé par le style qu'une forme ne découpe pas : le type du style
/// lui-même, ou celui d'un bloc qui porte son nom. Un composant n'est jamais découpé.
fn outside(rule: &StyleRule, program: &Program) -> Option<String> {
    match &rule.target {
        Target::Type(t) => (!CUT.contains(&t.as_str())).then(|| t.clone()),
        Target::Name(_) => blocks_of(rule, program).into_iter().find(|b| !CUT.contains(&b.name.as_str())).map(|b| b.name.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les points d'une découpe, en pourcentages.
    fn points(cut: &str) -> Vec<(f64, f64)> {
        let inside = cut.strip_prefix("polygon(").and_then(|c| c.strip_suffix(')')).unwrap();
        let percent = |v: &str| if v == "0" { 0.0 } else { v.strip_suffix('%').unwrap().parse::<f64>().unwrap() };
        inside.split(',').map(|p| p.split_once(' ').map(|(x, y)| (percent(x), percent(y))).unwrap()).collect()
    }

    #[test]
    fn every_form_is_a_named_bounded_cut() {
        // Le rond et le carré ne découpent pas ; les six autres sont des polygones dans la boîte,
        // qui en touchent les bords (la forme suit la taille du bloc), sans tracé à la main.
        assert_eq!(FORMS.iter().filter(|f| polygon(f).is_none()).collect::<Vec<_>>(), [&"circle", &"square"]);
        for form in FORMS.iter().filter_map(|f| polygon(f).map(|cut| (*f, cut))) {
            let (name, cut) = form;
            let p = points(cut);
            assert!(p.len() >= 3 && p.iter().all(|(x, y)| (0.0..=100.0).contains(x) && (0.0..=100.0).contains(y)), "{name}");
            let (left, right) = (p.iter().map(|q| q.0).fold(f64::MAX, f64::min), p.iter().map(|q| q.0).fold(f64::MIN, f64::max));
            let (top, bottom) = (p.iter().map(|q| q.1).fold(f64::MAX, f64::min), p.iter().map(|q| q.1).fold(f64::MIN, f64::max));
            assert!(left == 0.0 && right == 100.0 && top <= 0.1 && bottom == 100.0, "{name} : {left} {right} {top} {bottom}");
        }
        // Le cœur est symétrique ; la vague ne touche que le dixième du bas.
        let heart = points(polygon("heart").unwrap());
        for (x, y) in &heart {
            assert!(heart.iter().any(|(x2, y2)| (x + x2 - 100.0).abs() < 1e-9 && (y - y2).abs() < 1e-9), "cœur : ({x}, {y}) sans son miroir");
        }
        assert!(points(polygon("wave").unwrap())[2..].iter().all(|(_, y)| (90.0..=100.0).contains(y)));
        // Dans un style : la découpe et les coins, toujours ensemble.
        assert_eq!(declaration("circle"), "clip-path:none;border-radius:50%;");
        assert_eq!(declaration("square"), "clip-path:none;border-radius:0;");
        assert_eq!(declaration("triangle"), "clip-path:polygon(50% 0,100% 100%,0 100%);border-radius:0;");
    }

    #[test]
    fn a_form_cuts_an_image_or_a_drawing_and_nothing_is_cut_silently() {
        let check = |source: &str| crate::styles::check_styles(&crate::holo::read(source).unwrap());
        let refused = |source: &str| check(source).unwrap_err().message;
        let photo = |styles: &str| format!("Page(children: [ Image.photo(source: \"a.png\", alt: \"\"), Image.autre(source: \"b.png\", alt: \"\"), Drawing(label: \"Un dessin\", width: 10, height: 10, children: [ Rect(x: 0, y: 0, width: 10, height: 10) ]) ])\n.autre {{ opacity: 1; }}\n{styles}");
        // Acceptés : une image, un dessin ; le rond garde son bord et son ombre ; les écrans et le
        // thème sombre ; une ombre retirée par le style qui découpe ; un rond plus fort qu'un polygone.
        let accepted = [
            ".photo { form: hexagon; opacity: 0.9; border: none; box-shadow: none; }\nImage { form: circle; border: 2px solid white; box-shadow: 0 4px 12px #00000066; }\nDrawing { form: star; }",
            ".photo { form: wave; phone: { form: square; } computer: { form: heart; } narrow: { form: diamond; } dark: { form: triangle; } print: { form: square; } }",
            "Image { box-shadow: 0 4px 12px #00000066; border: 1px solid gray; }\n.photo { form: hexagon; box-shadow: none; border: none; }",
            "Image { form: hexagon; }\n.photo { form: circle; box-shadow: 0 2px 4px black; }",
        ];
        for styles in accepted {
            let result = check(&photo(styles));
            assert!(result.is_ok(), "{styles}\n→ {result:?}");
        }
        assert!(check("Page(children: [ Shape.rond(form: circle), Shape(name: S, form: star) ])\n.rond { background: gold; box-shadow: 0 2px 4px black; }").is_ok());
        // Refusés, avec la raison : un texte, un bloc qui en porte, une vidéo, une Shape.
        assert!(refused("Page(children: [ P(\"x\") ])\nP { form: circle; }").contains("sur « P », elle couperait aussi les textes et les boutons qu'il porte"));
        assert!(refused("Page(children: [ Column.carte(children: [ Image(source: \"a.png\", alt: \"\") ]) ])\n.carte { form: hexagon; }").contains("sur « Column »"));
        assert!(refused("Page(children: [ Video(source: \"f.mp4\", label: \"x\") ])\nVideo { form: circle; }").contains("ses commandes et ses sous-titres"));
        assert!(refused("Page(children: [ Shape.s(form: star) ])\n.s { form: heart; }").contains("la forme d'une Shape s'écrit sur elle, Shape(form: star)"));
        // Ni sous la souris, ni sous le doigt, ni au focus.
        assert!(refused(&photo(".photo { hover: { form: star; } }")).contains("clignoterait"));
        assert!(refused(&photo(".photo { active: { form: square; } }")).contains("l'état « active »"));
        assert!(refused(&photo(".photo { focus: { form: square; } }")).contains("le moteur montre déjà le bloc entier"));
        // Les coins : la forme les donne.
        assert!(refused(&photo(".photo { form: circle; border-radius: 8px; }")).contains("« border-radius » et « form » dans « .photo » : la forme donne déjà les coins"));
        assert!(refused(&photo(".photo { phone: { form: square; border-radius: 4px; } }")).contains(", dans l'état « phone »"));
        // Ce que la découpe couperait en silence : une ombre, un bord, dans le même style ou un autre.
        assert!(refused(&photo(".photo { form: star; box-shadow: 0 4px 12px #00000066; }")).contains("l'ombre : il n'en resterait rien ; garde le rond (form: circle), dont l'ombre suit la courbe"));
        assert!(refused(&photo(".photo { form: hexagon; hover: { border: 2px solid white; } }")).contains("le bord : il n'en resterait que des morceaux"));
        assert!(refused(&photo(".photo { phone: { form: wave; } box-shadow: 0 2px 4px black; }")).contains("l'ombre"));
        assert!(refused(&photo("Image { box-shadow: 0 4px 12px #00000066; }\n.photo { form: hexagon; }")).contains("« .photo » découpe « Image » en « hexagon », qui reçoit aussi « box-shadow » de « Image » : la découpe couperait l'ombre : il n'en resterait rien ; écris « box-shadow: none; » dans « .photo »"));
        let stronger = "Page(children: [ Image.cadre(source: \"a.png\", alt: \"\") ])\nImage { form: star; }\n.cadre { border: 2px solid gold; }";
        assert!(refused(stronger).contains("retire « border » de « .cadre »"));
        // Une Shape découpée se dessine dans son carré : son fond, son ombre, son bord l'entoureraient.
        assert!(refused("Page(children: [ Shape.s(form: star) ])\n.s { background: gold; }").contains("la couleur d'une forme s'écrit sur elle, Shape(form: star, color: \"#E9B44C\")"));
        assert!(refused("Page(children: [ Shape(form: heart), Shape(form: circle) ])\nShape { box-shadow: 0 2px 4px black; }").contains("une Shape en « heart » se dessine dans son carré ; l'ombre entourerait le carré, pas la forme"));
        assert!(refused("Page(children: [ Shape.s(name: S, form: triangle) ])\n.s { hover: { border: 1px solid white; } }").contains("le bord entourerait le carré"));
        // Pas de tracé écrit à la main : le mot du CSS, une liste de points, un mot inconnu.
        assert!(refused(&photo(".photo { clip-path: circle(50%); }")).contains("« clip-path » s'écrit « form: hexagon » : une forme nommée, parmi circle, square, triangle, diamond, hexagon, star, heart, wave"));
        assert!(refused(&photo(".photo { clip: rect(0, 10px, 10px, 0); }")).contains("« clip » s'écrit « form: hexagon »"));
        assert!(refused(&photo(".photo { form: polygon(50% 0, 100% 100%, 0 100%); }")).contains("une forme nommée : circle, square"));
        assert!(refused(&photo(".photo { phone: { form: oval; } }")).contains("pas de tracé écrit à la main"));
    }

    #[test]
    fn only_the_shapes_of_the_page_are_drawn_inside_their_button() {
        let program = crate::holo::read("Page(children: [ Shape(form: circle), Shape(name: S, form: star), Shape(form: star), Shape(form: wave) ])").unwrap();
        let css = shapes_css(&program);
        assert!(css.starts_with(":where(.holo-forme-star){background:transparent}:where(.holo-forme-star)::before{content:\"\";display:block;width:100%;height:100%;background:var(--holo-color,currentColor);clip-path:polygon(50% 0,"), "{css}");
        assert_eq!(css.matches("::before").count(), 2, "{css}");
        assert!(css.contains(".holo-forme-wave") && !css.contains("circle") && !css.contains("heart"), "{css}");
        assert_eq!(shapes_css(&crate::holo::read("Page(children: [ P(\"x\") ])").unwrap()), "");
    }
}
