// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What the fetched results repository holds for one watched repository.

use std::path::Path;

use crate::system::git::git_constants::{LOCAL_TIMEOUT, RESULTS_REMOTE_REF};
use crate::system::git::git_runner::GitRunner;
use sentinel4rust::model::branch::record_error::RecordError;
use sentinel4rust::model::branch::record_state::RecordState;
use sentinel4rust::model::branch::summary_toml::SummaryToml;
use sentinel4rust::model::mutant_key::MutantKey;

/// Reads a repository's `summary.toml` and `missed.txt` off the fetched results branch, in its folder.
#[derive(Clone)]
pub struct RecordReader {
    git: GitRunner,
}

impl RecordReader {
    /// Reads the branch through `git`.
    pub fn new(git: GitRunner) -> Self {
        Self { git }
    }

    /// The survivors `folder`'s `missed.txt` lists, one key per non-blank line; none when it cannot be read.
    pub fn previous_missed(&self, dir: &Path, folder: &str) -> Vec<MutantKey> {
        self.show(dir, &format!("{folder}/missed.txt"))
            .map(|text| {
                text.lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(MutantKey::new)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// What `folder`'s `summary.toml` holds: absent when it is missing or in a newer schema, unreadable when it does not
    /// parse.
    pub fn record(&self, dir: &Path, folder: &str) -> RecordState {
        let Some(text) = self.show(dir, &format!("{folder}/summary.toml")) else {
            return RecordState::Absent;
        };
        match SummaryToml.parse(&text) {
            Ok(record) => RecordState::Present(Box::new(record)),
            Err(RecordError::UnknownSchema(_)) => RecordState::Absent,
            Err(error) => RecordState::Unreadable(error),
        }
    }

    fn show(&self, dir: &Path, path: &str) -> Option<String> {
        let object = format!("{RESULTS_REMOTE_REF}:{path}");
        self.git.git(dir, &["show", &object], LOCAL_TIMEOUT).ok()
    }
}
