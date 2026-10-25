// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The contents of `summary.toml`.

use crate::model::commit_hash::CommitHash;
use crate::model::counts::MutantCounts;
use crate::model::outcome_kind::OutcomeKind;
use crate::model::timestamp::Timestamp;

/// The contents of `summary.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    /// The commit tested.
    pub commit: CommitHash,
    /// The roster's `config_version` at the time.
    pub config_version: i64,
    /// `full`.
    pub kind: String,
    /// What was published.
    pub outcome: OutcomeKind,
    /// When the run started.
    pub started: Timestamp,
    /// When the run finished.
    pub finished: Timestamp,
    /// The counts (zero unless completed).
    pub counts: MutantCounts,
    /// Survivors not in the previous record.
    pub new: u64,
    /// Survivors of the previous record no longer present.
    pub gone: u64,
    /// `cargo mutants --version`.
    pub cargo_mutants_version: String,
    /// `rustc --version`.
    pub rustc_version: String,
    /// The arguments given to cargo.
    pub arguments: String,
    /// For `tool-failed`: why.
    pub reason: Option<String>,
    /// For failures: the end of the log.
    pub log_tail: Option<String>,
}
