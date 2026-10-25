// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The detached worktree a run works in.

use std::fs;
use std::path::PathBuf;

use crate::model::checkout_error::CheckoutError;
use crate::model::commit_hash::CommitHash;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::work_dir::WorkDir;
use crate::system::git::git_constants::NETWORK_TIMEOUT;
use crate::system::git::git_runner::GitRunner;

/// A worktree of the bare clone, detached at the commit under test, that nobody works in.
#[derive(Clone)]
pub struct DetachedWorktree {
    git: GitRunner,
    work_dir: WorkDir,
}

impl DetachedWorktree {
    /// Worktrees under `work_dir`, made through `git`.
    pub fn new(git: GitRunner, work_dir: WorkDir) -> Self {
        Self { git, work_dir }
    }

    /// Checks `head` out, detached, where `work_dir` puts `repo`'s checkout; returns that path.
    pub fn add(&self, repo: &OnboardedRepo, head: &CommitHash) -> Result<PathBuf, CheckoutError> {
        let dir = self.work_dir.bare_clone(repo);
        let run_dir = self.work_dir.checkout(repo);
        if let Some(parent) = run_dir.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CheckoutError(format!("create {}: {e}", parent.display())))?;
        }
        let run_text = run_dir.to_string_lossy().into_owned();
        self.git
            .git(
                &dir,
                &[
                    "worktree",
                    "add",
                    "--quiet",
                    "--detach",
                    "--force",
                    &run_text,
                    head.as_str(),
                ],
                NETWORK_TIMEOUT,
            )
            .map_err(|r| CheckoutError(format!("git worktree add: {}", r.describe())))?;
        Ok(run_dir)
    }

    /// Removes `repo`'s checkout, whatever state it is in.
    pub fn remove(&self, repo: &OnboardedRepo) {
        let dir = self.work_dir.bare_clone(repo);
        let run_dir = self.work_dir.checkout(repo);
        if run_dir.exists() && dir.join("HEAD").exists() {
            let run_text = run_dir.to_string_lossy().into_owned();
            let _ = self.git.local(
                &dir,
                &["worktree", "remove", "--force", "--force", &run_text],
            );
        }
        let _ = fs::remove_dir_all(&run_dir);
    }
}
