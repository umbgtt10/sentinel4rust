// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What a fetch found.

use crate::model::branch::record_state::RecordState;
use crate::model::commit_hash::CommitHash;
use crate::model::mutant_key::MutantKey;
use crate::model::skip_reason::SkipReason;
use alloc::vec::Vec;

/// What a fetch found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// The configured branch's commit.
    pub head: CommitHash,
    /// What the repository's `summary.toml` in the results repository holds.
    pub record: RecordState,
    /// The repository's `missed.txt` in the results repository, for new/gone.
    pub previous_missed: Vec<MutantKey>,
    /// Whether `head` can be tested.
    pub readiness: Result<(), SkipReason>,
}
