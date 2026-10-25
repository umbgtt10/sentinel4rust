// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What the fetched `mutants` branch holds.

use std::path::Path;

use crate::model::branch::run_record::RunRecord;
use crate::model::branch::summary_toml::SummaryToml;
use crate::model::mutant_key::MutantKey;
use crate::system::git::git_constants::{LOCAL_TIMEOUT, MUTANTS_REMOTE_REF};
use crate::system::git::git_runner::GitRunner;

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

    /// The record, when `summary.toml` is there and parses.
    pub fn record(&self, dir: &Path) -> Option<RunRecord> {
        self.show(dir, "summary.toml")
            .and_then(|text| SummaryToml.parse(&text).ok())
    }

    fn show(&self, dir: &Path, file: &str) -> Option<String> {
        let object = format!("{MUTANTS_REMOTE_REF}:{file}");
        self.git.git(dir, &["show", &object], LOCAL_TIMEOUT).ok()
    }
}
