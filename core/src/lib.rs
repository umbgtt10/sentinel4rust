//! sentinel4rust: mutation testing around the clock.
//!
//! Walks a roster of repositories; whenever a repository's `main` moves, runs cargo-mutants against that exact
//! commit and publishes the surviving mutants on the repository's own `mutants` branch. See `docs/DESIGN.md`.

pub mod cli;
pub mod git;
pub mod model;
pub mod mutants;
pub mod schedule;
pub mod sentinel;
pub mod system;
pub mod toml;
pub mod traits;
