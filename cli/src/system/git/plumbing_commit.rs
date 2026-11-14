// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A commit in the results repository written without a checkout.

use std::fs;
use std::path::Path;

use crate::system::git::git_constants::{LOCAL_TIMEOUT, RESULTS_REMOTE_REF};
use crate::system::git::git_runner::GitRunner;
use sentinel4rust::model::branch::branch_file::BranchFile;
use sentinel4rust::model::branch::branch_layout::BranchLayout;
use sentinel4rust::model::branch::publication::Publication;
use sentinel4rust::model::publish_error::PublishError;

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

    /// Writes `publication`'s files from `stage` into a commit whose tree is the fetched results branch's, with
    /// `folder` replaced, and whose parent is that branch, if any; returns the commit.
    pub fn write(
        &self,
        dir: &Path,
        stage: &Path,
        folder: &str,
        publication: &Publication,
    ) -> Result<String, PublishError> {
        let index = stage.join("index").to_string_lossy().into_owned();
        let parent = self.parent(dir);
        if let Some(parent) = &parent {
            self.seed_index(dir, stage, &index, parent, folder)?;
        }
        for (number, file) in BranchLayout.files(folder, publication).iter().enumerate() {
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
        self.commit_tree(
            dir,
            tree.trim(),
            parent.as_deref(),
            &BranchLayout.message(folder, publication),
        )
    }

    fn commit_tree(
        &self,
        dir: &Path,
        tree: &str,
        parent: Option<&str>,
        message: &str,
    ) -> Result<String, PublishError> {
        let mut args = vec!["commit-tree", tree];
        if let Some(parent) = parent {
            args.extend(["-p", parent]);
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

    fn parent(&self, dir: &Path) -> Option<String> {
        self.git
            .git(
                dir,
                &[
                    "rev-parse",
                    "--verify",
                    "--quiet",
                    &format!("{RESULTS_REMOTE_REF}^{{commit}}"),
                ],
                LOCAL_TIMEOUT,
            )
            .ok()
            .map(|parent| parent.trim().to_string())
    }

    // The parent's tree in the index, less `folder`: the bare clone has no work tree, and `--force-remove` touches
    // none, so the staging area stands in for one.
    fn seed_index(
        &self,
        dir: &Path,
        stage: &Path,
        index: &str,
        parent: &str,
        folder: &str,
    ) -> Result<(), PublishError> {
        let read = self
            .git
            .command(dir, &["read-tree", parent], LOCAL_TIMEOUT)
            .with_env("GIT_INDEX_FILE", index);
        self.git
            .run(&read)
            .map_err(|r| PublishError(format!("git read-tree: {}", r.describe())))?;
        let listed = self
            .git
            .local(dir, &["ls-tree", "-r", "--name-only", parent, "--", folder])
            .map_err(|failure| PublishError(failure.to_string()))?;
        let paths: Vec<&str> = listed.lines().filter(|l| !l.is_empty()).collect();
        if paths.is_empty() {
            return Ok(());
        }
        let stage_text = stage.to_string_lossy();
        let mut args = vec![
            "--work-tree",
            stage_text.as_ref(),
            "update-index",
            "--force-remove",
            "--",
        ];
        args.extend(paths);
        let remove = self
            .git
            .command(dir, &args, LOCAL_TIMEOUT)
            .with_env("GIT_INDEX_FILE", index);
        self.git
            .run(&remove)
            .map(|_| ())
            .map_err(|r| PublishError(format!("git update-index --force-remove: {}", r.describe())))
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
