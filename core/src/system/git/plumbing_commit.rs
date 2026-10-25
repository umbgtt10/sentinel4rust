// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A commit on `mutants` written without a checkout.

use std::fs;
use std::path::Path;

use crate::model::branch::branch_file::BranchFile;
use crate::model::branch::branch_layout::BranchLayout;
use crate::model::branch::publication::Publication;
use crate::model::publish_error::PublishError;
use crate::system::git::git_constants::{LOCAL_TIMEOUT, MUTANTS_REMOTE_REF};
use crate::system::git::git_runner::GitRunner;

const IDENTITY_NAME: &str = "sentinel4rust";
const IDENTITY_EMAIL: &str = "sentinel4rust@localhost";

/// Writes a publication as a commit with plumbing only: blobs, a tree in a temporary index, `commit-tree`.
#[derive(Clone)]
pub struct PlumbingCommit {
    git: GitRunner,
}

impl PlumbingCommit {
    /// Writes through `git`.
    pub fn new(git: GitRunner) -> Self {
        Self { git }
    }

    /// Writes `publication`'s files from `stage` into a commit whose parent is the fetched `mutants`, if
    /// any; returns the commit.
    pub fn write(
        &self,
        dir: &Path,
        stage: &Path,
        publication: &Publication,
    ) -> Result<String, PublishError> {
        let index = stage.join("index").to_string_lossy().into_owned();
        for (number, file) in BranchLayout.files(publication).iter().enumerate() {
            self.stage_file(dir, stage, &index, number, file)?;
        }
        let tree = self
            .git
            .run(
                &self
                    .git
                    .command(dir, &["write-tree"], LOCAL_TIMEOUT)
                    .with_env("GIT_INDEX_FILE", &index),
            )
            .map_err(|r| PublishError(format!("git write-tree: {}", r.describe())))?;
        self.commit_tree(dir, tree.trim(), &BranchLayout.message(publication))
    }

    fn commit_tree(&self, dir: &Path, tree: &str, message: &str) -> Result<String, PublishError> {
        let parent = self
            .git
            .git(
                dir,
                &[
                    "rev-parse",
                    "--verify",
                    "--quiet",
                    &format!("{MUTANTS_REMOTE_REF}^{{commit}}"),
                ],
                LOCAL_TIMEOUT,
            )
            .ok();
        let mut args = vec!["commit-tree", tree];
        if let Some(parent) = &parent {
            args.extend(["-p", parent.trim()]);
        }
        args.extend(["-m", message]);
        let command = self
            .git
            .command(dir, &args, LOCAL_TIMEOUT)
            .with_env("GIT_AUTHOR_NAME", IDENTITY_NAME)
            .with_env("GIT_AUTHOR_EMAIL", IDENTITY_EMAIL)
            .with_env("GIT_COMMITTER_NAME", IDENTITY_NAME)
            .with_env("GIT_COMMITTER_EMAIL", IDENTITY_EMAIL);
        self.git
            .run(&command)
            .map(|commit| commit.trim().to_string())
            .map_err(|r| PublishError(format!("git commit-tree: {}", r.describe())))
    }

    fn stage_file(
        &self,
        dir: &Path,
        stage: &Path,
        index: &str,
        number: usize,
        file: &BranchFile,
    ) -> Result<(), PublishError> {
        let blob_file = stage.join(format!("blob-{number}"));
        fs::write(&blob_file, &file.contents)
            .map_err(|e| PublishError(format!("write {}: {e}", blob_file.display())))?;
        let blob = self
            .git
            .local(dir, &["hash-object", "-w", &blob_file.to_string_lossy()])
            .map_err(|failure| PublishError(failure.to_string()))?;
        let info = format!("100644,{},{}", blob.trim(), file.path);
        let command = self
            .git
            .command(
                dir,
                &["update-index", "--add", "--cacheinfo", &info],
                LOCAL_TIMEOUT,
            )
            .with_env("GIT_INDEX_FILE", index);
        self.git
            .run(&command)
            .map(|_| ())
            .map_err(|r| PublishError(format!("git update-index: {}", r.describe())))
    }
}
