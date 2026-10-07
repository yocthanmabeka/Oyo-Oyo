use holo_engine::{holo, check};

fn main() {
    for seed in ["9007199254740992", "9007199254740993"] {
        let source = format!("Point(nom: A, graine: {seed})");
        println!("seed {seed} => {:?}", check(&source).map(|p| p.seed));
    }
    println!("ignored import => {:?}", check("import \"absent.holo\" Point(nom: A, graine: 1)").map(|p| p.seed));
    println!("unit with whitespace => {:?}", holo::read("Point(nom: A, graine: 1, budget: 500 Ko)").map(|p| p.root.name));
    let source = include_str!("../boutique.holo");
    println!("boutique syntax => {:?}", holo::read(source).map(|p| p.root.name));
    println!("boutique semantics => {:?}", check(source).map(|p| p.name));
}
