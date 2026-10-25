// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Whether a record answers for a commit (`ADR-StateLivesOnMutants`).

use crate::model::branch::run_record::RunRecord;
use crate::model::commit_hash::CommitHash;

/// Whether a record answers for a commit under the roster's `config_version` and the installed cargo-mutants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordCurrency {
    config_version: i64,
    cargo_mutants_version: Option<String>,
}

impl RecordCurrency {
    /// Records are current under `config_version` and `cargo_mutants_version`, as `cargo mutants --version`
    /// prints it; `None` when it could not be read, so it decides nothing.
    pub fn new(config_version: i64, cargo_mutants_version: Option<String>) -> Self {
        Self {
            config_version,
            cargo_mutants_version,
        }
    }

    /// Whether `record` answers for `head`: the same commit, under the same `config_version`, by the same
    /// cargo-mutants when its version is known.
    pub fn is_current(&self, record: &RunRecord, head: &CommitHash) -> bool {
        &record.commit == head
            && record.config_version == self.config_version
            && self
                .cargo_mutants_version
                .as_ref()
                .is_none_or(|installed| installed == &record.cargo_mutants_version)
    }
}
