// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Where the clones, checkouts, staging areas and cargo-mutants output live (design 2.4).

use std::path::{Path, PathBuf};

use crate::model::onboarding::onboarded_repo::OnboardedRepo;

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

    /// Where cargo-mutants writes `mutants.out` for `repo`.
    pub fn mutants_out(&self, repo: &OnboardedRepo) -> PathBuf {
        self.root.join("out").join(&repo.name)
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
}
