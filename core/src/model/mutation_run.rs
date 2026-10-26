// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! One run and the facts needed to record it.

use crate::model::run_outcome::RunOutcome;
use alloc::string::String;

/// One run and the facts needed to record it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationRun {
    /// What happened.
    pub outcome: RunOutcome,
    /// `cargo mutants --version`.
    pub cargo_mutants_version: String,
    /// `rustc --version` in the checkout (so the repository's toolchain).
    pub rustc_version: String,
    /// The arguments given to cargo, joined.
    pub arguments: String,
}
