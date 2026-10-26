// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! sentinel4rust: mutation testing around the clock.
//!
//! The `sentinel4rust` binary: its command line, the composition root, and the real implementations of the core's
//! contracts -- processes, git, cargo-mutants, the run unit, the clock, sleeping and stdout. The logic is
//! `sentinel4rust-core`. See `docs/ARCHITECTURE.md`.

#![warn(missing_docs)]

pub mod invocation;
