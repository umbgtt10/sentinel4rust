// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Running mutation testing.

use std::path::Path;
use std::time::Duration;

use crate::model::mutation_run::MutationRun;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;

/// Runs mutation testing.
pub trait MutationRunner {
    /// Runs cargo-mutants in `checkout`, killed after `timeout`.
    fn run(
        &self,
        repo: &OnboardedRepo,
        checkout: &Path,
        jobs: u32,
        timeout: Duration,
    ) -> MutationRun;
}
