// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Publishing results.

use crate::model::branch::publication::Publication;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::publish_error::PublishError;

/// Publishes results.
pub trait ResultStore {
    /// Writes `publication` as a new commit on `mutants` and pushes only that branch.
    fn publish(&self, repo: &OnboardedRepo, publication: &Publication) -> Result<(), PublishError>;
}
