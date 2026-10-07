# Les modules des leçons 69 et 97

Trois modules d'essai, une dizaine de lignes de Rust chacun, sans bibliothèque (`no_std`) :

- `compter.rs` : la somme des nombres de 1 à n ;
- `boucle.rs` : une boucle sans fin, exprès, pour prouver que le moteur l'arrête ;
- `memoire.rs` : une demande de 64 Mo, exprès, pour prouver que le plafond de mémoire tient.

Pour les fabriquer à nouveau (Rust et la cible `wasm32-unknown-unknown`, déjà là pour le moteur) :

```text
rustc --edition 2021 --target wasm32-unknown-unknown -O --crate-type cdylib -C panic=abort -C link-arg=--import-memory -C link-arg=--strip-all compter.rs -o ../69-compter.wasm
```

`--import-memory` : le module ne fabrique pas sa mémoire ; c'est le moteur qui la lui donne, plafonnée.

## Le second contrat (leçon 97, ADR-077)

- `bulletin.rs` : reçoit la liste des notes, en JSON, et rend la moyenne, la meilleure matière et le nombre de notes ;
- `menteur.rs` : rend une valeur qu'il n'annonce pas, exprès, pour prouver que le moteur refuse sa réponse.

Un module du second contrat offre deux fonctions :

- `alloc(taille) -> adresse` : où le moteur écrit ce que le module reçoit, un texte JSON (`{"notes":[…]}`) ; 0 s'il n'y a pas la place ;
- `run(adresse, taille) -> u64` : lit ce texte, et rend l'adresse de sa réponse (les 32 bits du haut) et sa taille (les 32 bits du bas). La réponse est un objet JSON de 64 Ko au plus, dont les clés sont les valeurs annoncées dans `output`.

```text
rustc --edition 2021 --target wasm32-unknown-unknown -O --crate-type cdylib -C panic=abort -C link-arg=--import-memory -C link-arg=--strip-all -C link-arg=-zstack-size=65536 bulletin.rs -o ../97-bulletin.wasm
```

`-zstack-size=65536` : une pile de 64 Ko au lieu d'1 Mo ; le module tient alors dans deux pages de mémoire, bien sous son plafond.
