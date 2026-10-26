// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Running mutation testing.

use std::time::Duration;

use crate::model::mutation_run::MutationRun;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::run_outcome::RunOutcome;

/// Runs mutation testing.
pub trait MutationRunner {
    /// Runs cargo-mutants in `checkout`, killed after `timeout`.
    fn run(
        &self,
        repo: &OnboardedRepo,
        checkout: &str,
        jobs: u32,
        timeout: Duration,
    ) -> MutationRun;
    /// Runs cargo-mutants in `checkout` on exactly `mutants`, named as cargo-mutants lists them, killed after
    /// `timeout`.
    fn recheck(
        &self,
        repo: &OnboardedRepo,
        checkout: &str,
        jobs: u32,
        timeout: Duration,
        mutants: &[String],
    ) -> RunOutcome;
}
