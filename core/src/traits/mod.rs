// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The contracts every part of the tool talks through (`docs/ARCHITECTURE.md`). Each has a real implementation in the
//! `sentinel4rust` package and, in tests, a fake.

pub mod clock;
pub mod command_runner;
pub mod journal;
pub mod mutants_out;
pub mod mutation_runner;
pub mod pause;
pub mod repo_source;
pub mod result_store;
