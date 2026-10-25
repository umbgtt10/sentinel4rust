// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The bare clone each repository is fetched into.

use std::fs;
use std::path::{Path, PathBuf};

use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::sync_error::SyncError;
use crate::model::work_dir::WorkDir;
use crate::system::git::git_runner::GitRunner;

/// The bare clone of a repository under the work directory, made once and kept pointing at its URL.
#[derive(Clone)]
pub struct BareClone {
    git: GitRunner,
    work_dir: WorkDir,
}

impl BareClone {
    /// Clones under `work_dir` through `git`.
    pub fn new(git: GitRunner, work_dir: WorkDir) -> Self {
        Self { git, work_dir }
    }

    /// The clone of `repo`, made if missing and its origin reset if the roster's URL changed.
    pub fn ensure(&self, repo: &OnboardedRepo) -> Result<PathBuf, SyncError> {
        let dir = self.work_dir.bare_clone(repo);
        if Self::is_clone(&dir) {
            let current = self.local(&dir, &["remote", "get-url", "origin"])?;
            if current.trim() != repo.url {
                self.local(&dir, &["remote", "set-url", "origin", &repo.url])?;
            }
            return Ok(dir);
        }
        let parent = dir.parent().unwrap_or(self.work_dir.root()).to_path_buf();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&parent)
            .map_err(|e| SyncError::Transient(format!("create {}: {e}", parent.display())))?;
        let dir_text = dir.to_string_lossy().into_owned();
        self.local(&parent, &["init", "--quiet", "--bare", &dir_text])?;
        self.local(&dir, &["remote", "add", "origin", &repo.url])?;
        Ok(dir)
    }

    /// Drops the clone's entries for worktrees that are gone; nothing when there is no clone.
    pub fn prune_worktrees(&self, repo: &OnboardedRepo) {
        let dir = self.work_dir.bare_clone(repo);
        if Self::is_clone(&dir) {
            let _ = self.git.local(&dir, &["worktree", "prune"]);
        }
    }

    fn is_clone(dir: &Path) -> bool {
        dir.join("HEAD").exists()
    }

    fn local(&self, dir: &Path, args: &[&str]) -> Result<String, SyncError> {
        self.git
            .local(dir, args)
            .map_err(|failure| SyncError::Transient(failure.to_string()))
    }
}
