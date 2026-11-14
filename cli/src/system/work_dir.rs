// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Where the clones, checkouts, staging areas, cargo-mutants output and temporary files live (`ADR-ExactCommitOnlyMutantsPushed`).

use std::path::{Path, PathBuf};

use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;

/// The roster's `work_dir`, and every path under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkDir {
    root: PathBuf,
}

impl WorkDir {
    /// Paths under `root`.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// The bare clone of `repo`.
    pub fn bare_clone(&self, repo: &OnboardedRepo) -> PathBuf {
        self.root.join("repos").join(format!("{}.git", repo.name))
    }

    /// The detached checkout a run of `repo` works in.
    pub fn checkout(&self, repo: &OnboardedRepo) -> PathBuf {
        self.runs().join(&repo.name)
    }

    /// Where the jobs handed to the run unit are written.
    pub fn jobs(&self) -> PathBuf {
        self.root.join("jobs")
    }

    /// Where cargo-mutants writes `mutants.out` for `repo`.
    pub fn mutants_out(&self, repo: &OnboardedRepo) -> PathBuf {
        self.root.join("out").join(&repo.name)
    }

    /// The bare clone of the results repository.
    pub fn results_clone(&self) -> PathBuf {
        self.root.join("results.git")
    }

    /// The directory itself.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Every checkout.
    pub fn runs(&self) -> PathBuf {
        self.root.join("runs")
    }

    /// Where a publication of `repo` is staged.
    pub fn stage(&self, repo: &OnboardedRepo) -> PathBuf {
        self.stages().join(&repo.name)
    }

    /// Every staging area.
    pub fn stages(&self) -> PathBuf {
        self.root.join("stage")
    }

    /// Where a run of `repo` keeps its temporary files: cargo-mutants' copies of the tree, one per job.
    pub fn tmp(&self, repo: &OnboardedRepo) -> PathBuf {
        self.tmp_root().join(&repo.name)
    }

    /// Every run's temporary files.
    pub fn tmp_root(&self) -> PathBuf {
        self.root.join("tmp")
    }
}
