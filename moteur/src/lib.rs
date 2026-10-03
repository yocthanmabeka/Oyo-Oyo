//! Moteur HoloCode, sprint Big Bang.
//!
//! Le cœur est en Rust pur et se teste sur le PC (`cargo test`) :
//! - `holo` lit un fichier `.holo` ;
//! - `blocs` vérifie que chaque bloc existe et que les titres ne sautent pas de niveau ;
//! - `univers` en fait un monde, entièrement calculé à partir d'une graine ;
//! - `navigation` gère le morcellement, le zoom, l'entrée et la sortie.
//!
//! La partie qui parle au navigateur et à la carte graphique (`web`, `rendu`) n'est
//! compilée que pour WebAssembly.

pub mod blocs;
pub mod graine;
pub mod holo;
pub mod navigation;
pub mod univers;

#[cfg(target_arch = "wasm32")]
mod rendu;
#[cfg(target_arch = "wasm32")]
mod web;

use holo::Erreur;
use univers::PointDecl;

/// Lit et vérifie un fichier `.holo`, sans rien exécuter. Toute erreur est rendue avec sa
/// ligne et sa colonne.
pub fn verifier(source: &str) -> Result<PointDecl, Erreur> {
    let programme = holo::lire(source)?;
    blocs::verifier_blocs(&programme)?;
    univers::point_depuis(&programme)
}
