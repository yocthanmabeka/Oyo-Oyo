//! Sondes de revue GPT-5.6 — web HoloCode, 2026-10-07.
//!
//! Ce fichier est rangé dans la proposition et n'est pas compilé automatiquement.
//! Pour le lancer : le copier temporairement dans `moteur/tests/revue_web_gpt56.rs`
//! puis exécuter `cargo test --manifest-path moteur/Cargo.toml --test revue_web_gpt56`.
//!
//! Ces sondes utilisent seulement l'API publique du crate quand c'est possible.

use holo_engine::{arbitrate, check_page, flat_view};

#[test]
fn composant_parametre_et_signal_de_copie() {
    let source = r#"
Page(
  state: State(cart: 0, sunrise: 0),
  parts: [
    Part(
      name: ArticleCard,
      params: [title, price, qty],
      children: [
        Column(children: [
          H2("{title}"),
          Button(name: Add, text: "Add"),
        ]),
      ],
      rules: [
        On(Add.tap, effect: [qty.add(1), cart.add(price)]),
      ],
    ),
  ],
  children: [
    ArticleCard(name: Sunrise, title: "Sunrise", price: 120, qty: sunrise),
  ],
)
"#;

    check_page(source).unwrap();
    let html = flat_view(source, "").unwrap();

    assert!(html.contains("Sunrise"));
    assert!(html.contains("data-name="AddSunrise""));
    assert_eq!(arbitrate(source, "cart=0;sunrise=0", "AddSunrise.tap"), "cart=120;sunrise=1");
}

#[test]
fn deux_copies_ont_deux_signaux_distincts() {
    let source = r#"
Page(
  state: State(a: 0, b: 0),
  parts: [
    Part(
      name: Card,
      params: [qty],
      children: [ Button(name: Add, text: "Add") ],
      rules: [ On(Add.tap, effect: qty.add(1)) ],
    ),
  ],
  children: [
    Card(name: A, qty: a),
    Card(name: B, qty: b),
  ],
)
"#;

    check_page(source).unwrap();

    assert_eq!(arbitrate(source, "a=0;b=0", "AddA.tap"), "a=1;b=0");
    assert_eq!(arbitrate(source, "a=0;b=0", "AddB.tap"), "a=0;b=1");
}

#[test]
fn parametre_et_state_du_meme_nom_sont_refuses() {
    let source = r#"
Page(
  state: State(title: ""),
  parts: [
    Part(
      name: Card,
      params: [title],
      children: [ H2("{title}") ],
    ),
  ],
  children: [
    Card(title: "A"),
  ],
)
"#;

    let erreur = check_page(source).unwrap_err();
    assert!(erreur.message.contains("nom d'une valeur de la page"), "{erreur}");
}

#[test]
fn item_reste_distinct_des_parametres() {
    let source = r#"
Page(
  state: State(a: 0, b: 0),
  parts: [
    Part(
      name: Card,
      params: [title, qty],
      children: [
        Column(children: [
          H2("{title}"),
          Button(name: Add, text: "Add"),
        ]),
      ],
      rules: [ On(Add.tap, effect: qty.add(1)) ],
    ),
  ],
  children: [
    Repeat(
      items: [
        Item(key: a, title: "A"),
        Item(key: b, title: "B"),
      ],
      children: [
        Card(title: item.title, qty: item),
      ],
    ),
  ],
)
"#;

    check_page(source).unwrap();
    let html = flat_view(source, "").unwrap();
    assert!(html.contains(">A</h2>"));
    assert!(html.contains(">B</h2>"));
    assert!(html.contains("data-name="AddA""));
    assert!(html.contains("data-name="AddB""));
}

#[test]
fn composant_directement_recursif_est_refuse() {
    let source = r#"
Page(
  parts: [
    Part(
      name: Card,
      children: [ Card() ],
    ),
  ],
  children: [ Card() ],
)
"#;

    let erreur = check_page(source).unwrap_err();
    assert!(erreur.message.contains("se pose lui-même"), "{erreur}");
}

#[test]
fn deux_composants_de_meme_nom_sont_refuses() {
    let source = r#"
Page(
  parts: [
    Part(name: Card, children: [ P("A") ]),
    Part(name: Card, children: [ P("B") ]),
  ],
  children: [ Card() ],
)
"#;

    let erreur = check_page(source).unwrap_err();
    assert!(erreur.message.contains("deux composants"), "{erreur}");
}

#[test]
fn html_du_composant_reste_semantique() {
    let source = r#"
Page(
  parts: [
    Part(
      name: Product,
      params: [title, image],
      children: [
        Main(children: [
          H2("{title}"),
          Image(source: image, alt: title),
          Button(name: Buy, text: "Buy"),
        ]),
      ],
    ),
  ],
  children: [
    Product(name: Sunrise, title: "Sunrise", image: "sunrise.png"),
  ],
)
"#;

    check_page(source).unwrap();
    let html = flat_view(source, "").unwrap();

    assert!(html.contains("<main"));
    assert!(html.contains("<h2"));
    assert!(html.contains("<img"));
    assert!(html.contains("alt="Sunrise""));
    assert!(html.contains("<button"));
}

#[test]
fn variante_et_variable_css_sont_du_vrai_css() {
    let source = r#"
Page(
  parts: [
    Part(
      name: Card,
      children: [ Column(children: [ P("A") ]) ],
    ),
  ],
  children: [
    Card.promo(),
  ],
)
Card { --accent: #E9B44C; border: 1px solid --accent; }
.promo { --accent: crimson; }
"#;

    check_page(source).unwrap();
    let html = flat_view(source, "").unwrap();

    assert!(html.contains("holo-c-Card"));
    assert!(html.contains("holo-s-promo"));
    assert!(html.contains(".holo-c-Card{"));
    assert!(html.contains(".holo-s-promo{"));
    assert!(html.contains("var(--accent)"));
}

#[test]
fn sonde_fuite_actuelle_d_un_nom_de_style() {
    // Cette sonde documente le comportement actuel, elle ne dit pas qu'il est souhaitable.
    //
    // Le même nom `.card` est utilisé dans le composant et hors du composant.
    // Le CSS actuel produit une règle globale `.holo-s-card`.
    // Cela prouve qu'un nom de style interne n'est pas encore isolé par composant.
    let source = r#"
Page(
  parts: [
    Part(
      name: BoxCard,
      children: [ Column.card(children: [ P("Inside") ]) ],
    ),
  ],
  children: [
    BoxCard(),
    P.card("Outside"),
  ],
)
.card { color: red; }
"#;

    check_page(source).unwrap();
    let html = flat_view(source, "").unwrap();

    assert_eq!(html.matches("holo-s-card").count(), 3, "{html}");
    assert!(html.contains(".holo-s-card{color:red;}"), "{html}");
}

#[test]
fn un_state_interne_de_part_n_est_pas_un_deuxieme_arbitre() {
    let source = r#"
Page(
  parts: [
    Part(
      name: Card,
      state: State(count: 0),
      children: [ P("{count}") ],
    ),
  ],
  children: [ Card() ],
)
"#;

    let erreur = check_page(source).unwrap_err();
    assert!(erreur.message.contains("state") || erreur.message.contains("réglage"), "{erreur}");
}
