// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! sentinel4rust's technology-free core: mutation testing around the clock, as logic.
//!
//! The patrol -- which repository to visit, when to back off, what to publish --, the roster, the record kept on
//! each repository's `mutants` branch, and what git, cargo-mutants and the run unit say and what it means. Every
//! effect goes through a trait in `traits`; the real implementations, and the binary, are the `sentinel4rust`
//! package. See `docs/ARCHITECTURE.md`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod model;
pub mod patrol;
pub mod protocol;
pub mod traits;
