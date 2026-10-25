// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Whether a commit can be mutation-tested.

use std::path::Path;

use crate::model::commit_hash::CommitHash;
use crate::model::skip_reason::SkipReason;
use crate::system::git::git_constants::LOCAL_TIMEOUT;
use crate::system::git::git_runner::GitRunner;
use crate::system::git::outside_paths::OutsidePaths;

/// Whether a commit is a crate with cargo-mutants filters (`ADR-FiltersLiveInEachRepository`).
#[derive(Clone)]
pub struct CrateReadiness {
    git: GitRunner,
}

impl CrateReadiness {
    /// Reads commits through `git`.
    pub fn new(git: GitRunner) -> Self {
        Self { git }
    }

    /// `Ok` when `head` has a `Cargo.toml` and a `.cargo/mutants.toml`, and no manifest has a `path` dependency
    /// outside the repository; otherwise why it is skipped.
    pub fn check(&self, dir: &Path, head: &CommitHash) -> Result<(), SkipReason> {
        if !self.exists_at(dir, head, "Cargo.toml") {
            Err(SkipReason::NotACrate)
        } else if !self.exists_at(dir, head, ".cargo/mutants.toml") {
            Err(SkipReason::NoFilters)
        } else if let Some(manifest) = self.manifest_leaving(dir, head) {
            Err(SkipReason::PathOutsideRepo { manifest })
        } else {
            Ok(())
        }
    }

    fn exists_at(&self, dir: &Path, head: &CommitHash, path: &str) -> bool {
        let object = format!("{head}:{path}");
        self.git
            .git(dir, &["cat-file", "-e", &object], LOCAL_TIMEOUT)
            .is_ok()
    }

    fn manifest_leaving(&self, dir: &Path, head: &CommitHash) -> Option<String> {
        let listing = self
            .git
            .git(
                dir,
                &["ls-tree", "-r", "--name-only", head.as_str()],
                LOCAL_TIMEOUT,
            )
            .ok()?;
        listing
            .lines()
            .filter(|path| *path == "Cargo.toml" || path.ends_with("/Cargo.toml"))
            .find(|manifest| {
                self.git
                    .git(dir, &["show", &format!("{head}:{manifest}")], LOCAL_TIMEOUT)
                    .is_ok_and(|text| OutsidePaths.escapes(manifest, &text))
            })
            .map(ToString::to_string)
    }
}
