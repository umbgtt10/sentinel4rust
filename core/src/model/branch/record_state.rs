// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What the fetched results repository says about a repository's record.

use crate::model::branch::record_error::RecordError;
use crate::model::branch::run_record::RunRecord;
use alloc::boxed::Box;

/// What a repository's `summary.toml` on the fetched results branch holds (`ADR-StateLivesOnMutants`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordState {
    /// No results branch, no `summary.toml` in the repository's folder, or one in a schema this version does not know: the commit is run.
    Absent,
    /// A record that parsed.
    Present(Box<RunRecord>),
    /// A `summary.toml` that does not parse: never run over, logged once, backed off until it reads again.
    Unreadable(RecordError),
}
