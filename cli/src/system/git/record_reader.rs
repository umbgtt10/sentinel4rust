// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What the fetched `mutants` branch holds.

use std::path::Path;

use crate::system::git::git_constants::{LOCAL_TIMEOUT, MUTANTS_REMOTE_REF};
use crate::system::git::git_runner::GitRunner;
use sentinel4rust::model::branch::record_error::RecordError;
use sentinel4rust::model::branch::record_state::RecordState;
use sentinel4rust::model::branch::summary_toml::SummaryToml;
use sentinel4rust::model::mutant_key::MutantKey;

/// Reads `summary.toml` and `missed.txt` off the fetched `mutants` branch.
#[derive(Clone)]
pub struct RecordReader {
    git: GitRunner,
}

impl RecordReader {
    /// Reads the branch through `git`.
    pub fn new(git: GitRunner) -> Self {
        Self { git }
    }

    /// The survivors `missed.txt` lists, one key per non-blank line; none when it cannot be read.
    pub fn previous_missed(&self, dir: &Path) -> Vec<MutantKey> {
        self.show(dir, "missed.txt")
            .map(|text| {
                text.lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(MutantKey::new)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// What `summary.toml` holds: absent when it is missing or in a newer schema, unreadable when it does not
    /// parse.
    pub fn record(&self, dir: &Path) -> RecordState {
        let Some(text) = self.show(dir, "summary.toml") else {
            return RecordState::Absent;
        };
        match SummaryToml.parse(&text) {
            Ok(record) => RecordState::Present(Box::new(record)),
            Err(RecordError::UnknownSchema(_)) => RecordState::Absent,
            Err(error) => RecordState::Unreadable(error),
        }
    }

    fn show(&self, dir: &Path, file: &str) -> Option<String> {
        let object = format!("{MUTANTS_REMOTE_REF}:{file}");
        self.git.git(dir, &["show", &object], LOCAL_TIMEOUT).ok()
    }
}
