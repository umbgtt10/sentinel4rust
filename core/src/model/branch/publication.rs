// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Everything written to `mutants` for one commit.

use crate::model::branch::run_record::RunRecord;
use crate::model::mutant_key::MutantKey;
use alloc::string::String;
use alloc::vec::Vec;

/// Everything written to `mutants` for one commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Publication {
    /// `summary.toml`.
    pub record: RunRecord,
    /// `missed.txt`, by key, sorted.
    pub missed: Vec<MutantKey>,
    /// `timeout.txt`, by key, sorted; not written when empty.
    pub timed_out: Vec<MutantKey>,
    /// `located/missed.txt`, as cargo-mutants wrote it.
    pub located_missed: Vec<String>,
    /// `located/timeout.txt`, as cargo-mutants wrote it.
    pub located_timed_out: Vec<String>,
}
