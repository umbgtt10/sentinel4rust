// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Whether a record answers for a commit (design 2.1).

use crate::model::branch::run_record::RunRecord;
use crate::model::commit_hash::CommitHash;

/// Whether a record answers for a commit under the roster's `config_version`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordCurrency {
    config_version: i64,
}

impl RecordCurrency {
    /// Records are current under `config_version`.
    pub fn new(config_version: i64) -> Self {
        Self { config_version }
    }

    /// Whether `record` answers for `head`: the same commit, under the same `config_version`.
    pub fn is_current(&self, record: &RunRecord, head: &CommitHash) -> bool {
        &record.commit == head && record.config_version == self.config_version
    }
}
