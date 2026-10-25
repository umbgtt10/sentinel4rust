// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What a fetch found.

use crate::model::branch::run_record::RunRecord;
use crate::model::commit_hash::CommitHash;
use crate::model::mutant_key::MutantKey;
use crate::model::skip_reason::SkipReason;

/// What a fetch found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// The configured branch's commit.
    pub head: CommitHash,
    /// The record on `mutants`, if there is a usable one.
    pub record: Option<RunRecord>,
    /// `missed.txt` on `mutants`, for new/gone.
    pub previous_missed: Vec<MutantKey>,
    /// Whether `head` can be tested.
    pub readiness: Result<(), SkipReason>,
}
