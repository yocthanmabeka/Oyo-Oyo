# Les modules de la leçon 69

Trois modules d'essai, une dizaine de lignes de Rust chacun, sans bibliothèque (`no_std`) :

- `compter.rs` : la somme des nombres de 1 à n ;
- `boucle.rs` : une boucle sans fin, exprès, pour prouver que le moteur l'arrête ;
- `memoire.rs` : une demande de 64 Mo, exprès, pour prouver que le plafond de mémoire tient.

Pour les fabriquer à nouveau (Rust et la cible `wasm32-unknown-unknown`, déjà là pour le moteur) :

```text
rustc --edition 2021 --target wasm32-unknown-unknown -O --crate-type cdylib -C panic=abort -C link-arg=--import-memory -C link-arg=--strip-all compter.rs -o ../69-compter.wasm
```

`--import-memory` : le module ne fabrique pas sa mémoire ; c'est le moteur qui la lui donne, plafonnée.
