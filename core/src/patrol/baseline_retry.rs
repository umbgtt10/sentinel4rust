// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! One run of a commit, retried once on a failing baseline (`ADR-ResultsFailuresAndBackOff`).

use alloc::rc::Rc;
use core::time::Duration;

use crate::model::checkout_error::CheckoutError;
use crate::model::commit_hash::CommitHash;
use crate::model::mutant_key::MutantKey;
use crate::model::mutation_run::MutationRun;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::run_outcome::RunOutcome;
use crate::patrol::survivor_recheck::SurvivorRecheck;
use crate::traits::mutation_runner::MutationRunner;
use crate::traits::repo_source::RepoSource;

/// Checks a commit out, runs cargo-mutants in it, on a baseline failure runs once more and re-checks new survivors
/// once (flaky tests); the checkout is removed whatever happens.
pub struct BaselineRetry {
    source: Rc<dyn RepoSource>,
    runner: Rc<dyn MutationRunner>,
}

impl BaselineRetry {
    /// Checks out through `source` and runs through `runner`.
    pub fn new(source: Rc<dyn RepoSource>, runner: Rc<dyn MutationRunner>) -> Self {
        Self { source, runner }
    }

    /// The run of `head`, killed after `timeout`, with the repository's `--jobs`, its survivors not in
    /// `previous_missed` checked once more in the same checkout; or why there was no checkout.
    pub fn run(
        &self,
        repo: &OnboardedRepo,
        head: &CommitHash,
        timeout: Duration,
        previous_missed: &[MutantKey],
    ) -> Result<MutationRun, CheckoutError> {
        let result = self.source.checkout(repo, head).map(|dir| {
            let run_once = || self.runner.run(repo, &dir, repo.jobs, timeout);
            let first = run_once();
            let run = if matches!(first.outcome, RunOutcome::BaselineFailed { .. }) {
                run_once()
            } else {
                first
            };
            self.rechecked(repo, &dir, timeout, run, previous_missed)
        });
        self.source.remove_checkout(repo);
        result
    }

    fn rechecked(
        &self,
        repo: &OnboardedRepo,
        dir: &str,
        timeout: Duration,
        run: MutationRun,
        previous_missed: &[MutantKey],
    ) -> MutationRun {
        let names = SurvivorRecheck.new_survivors(&run.outcome, previous_missed);
        if names.is_empty() {
            return run;
        }
        let recheck = self.runner.recheck(repo, dir, repo.jobs, timeout, &names);
        MutationRun {
            outcome: SurvivorRecheck.apply(run.outcome, &names, &recheck),
            ..run
        }
    }
}
