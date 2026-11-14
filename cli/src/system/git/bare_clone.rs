// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The bare clones the watched repositories and the results repository are fetched into.

use std::fs;
use std::path::{Path, PathBuf};

use crate::system::git::git_runner::GitRunner;
use crate::system::work_dir::WorkDir;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::sync_error::SyncError;

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
        self.ensure_at(&self.work_dir.bare_clone(repo), &repo.url)
    }

    /// A clone of `url` at `dir`, made if missing and its origin reset if the URL changed.
    pub fn ensure_at(&self, dir: &Path, url: &str) -> Result<PathBuf, SyncError> {
        let dir = dir.to_path_buf();
        if Self::is_clone(&dir) {
            let current = self.local(&dir, &["remote", "get-url", "origin"])?;
            if current.trim() != url {
                self.local(&dir, &["remote", "set-url", "origin", url])?;
            }
            return Ok(dir);
        }
        let parent = dir.parent().unwrap_or(self.work_dir.root()).to_path_buf();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&parent)
            .map_err(|e| SyncError::Transient(format!("create {}: {e}", parent.display())))?;
        let dir_text = dir.to_string_lossy().into_owned();
        self.local(&parent, &["init", "--quiet", "--bare", &dir_text])?;
        self.local(&dir, &["remote", "add", "origin", url])?;
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
