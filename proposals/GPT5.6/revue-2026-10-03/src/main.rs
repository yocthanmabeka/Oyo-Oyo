use holo_moteur::{holo, verifier};

fn main() {
    for seed in ["9007199254740992", "9007199254740993"] {
        let source = format!("Point(nom: A, graine: {seed})");
        println!("seed {seed} => {:?}", verifier(&source).map(|p| p.graine));
    }
    println!("ignored import => {:?}", verifier("import \"absent.holo\" Point(nom: A, graine: 1)").map(|p| p.graine));
    println!("unit with whitespace => {:?}", holo::lire("Point(nom: A, graine: 1, budget: 500 Ko)").map(|p| p.racine.nom));
    let source = include_str!("../boutique.holo");
    println!("boutique syntax => {:?}", holo::lire(source).map(|p| p.racine.nom));
    println!("boutique semantics => {:?}", verifier(source).map(|p| p.nom));
}
