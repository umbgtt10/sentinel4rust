// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Reading repositories.

use std::path::PathBuf;

use crate::model::checkout_error::CheckoutError;
use crate::model::commit_hash::CommitHash;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::snapshot::Snapshot;
use crate::model::sync_error::SyncError;

/// Reads repositories.
pub trait RepoSource {
    /// Clones if needed, fetches, and reads the branch's commit and the recorded result.
    fn sync(&self, repo: &OnboardedRepo) -> Result<Snapshot, SyncError>;
    /// Creates a detached checkout of `head` that nobody works in; returns its path.
    fn checkout(&self, repo: &OnboardedRepo, head: &CommitHash) -> Result<PathBuf, CheckoutError>;
    /// Removes the checkout, whatever state it is in.
    fn remove_checkout(&self, repo: &OnboardedRepo);
}
