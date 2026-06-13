// Native CLI surface (abscissa, clap). Cargo's target-cfg dep partition
// keeps these crates out of the wasm32 resolution graph; we cfg-gate the
// modules themselves so the wasm library presents only `core` + `wasm`.
#[cfg(not(target_family = "wasm"))]
pub mod application;
#[cfg(not(target_family = "wasm"))]
pub mod commands;
#[cfg(not(target_family = "wasm"))]
pub mod config;

pub mod core;

#[cfg(not(target_family = "wasm"))]
pub mod error;

#[cfg(target_family = "wasm")]
pub mod wasm;
