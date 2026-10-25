// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What the fetched `mutants` branch says about its record.

use crate::model::branch::record_error::RecordError;
use crate::model::branch::run_record::RunRecord;

/// What `summary.toml` on the fetched `mutants` branch holds (`ADR-StateLivesOnMutants`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordState {
    /// No `mutants`, no `summary.toml` on it, or one in a schema this version does not know: the commit is run.
    Absent,
    /// A record that parsed.
    Present(Box<RunRecord>),
    /// A `summary.toml` that does not parse: never run over, logged once, backed off until it reads again.
    Unreadable(RecordError),
}
