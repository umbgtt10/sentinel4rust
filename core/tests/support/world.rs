// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::branch::publication::Publication;
use sentinel4rust::model::checkout_error::CheckoutError;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::mutation_run::MutationRun;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::publish_error::PublishError;
use sentinel4rust::model::run_outcome::RunOutcome;
use sentinel4rust::model::snapshot::Snapshot;
use sentinel4rust::model::sync_error::SyncError;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::traits::clock::Clock;
use sentinel4rust::traits::journal::Journal;
use sentinel4rust::traits::mutation_runner::MutationRunner;
use sentinel4rust::traits::pause::Pause;
use sentinel4rust::traits::repo_source::RepoSource;
use sentinel4rust::traits::result_store::ResultStore;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Duration;

// Every run takes this long on the fake clock.
pub const RUN_SECONDS: i64 = 60;

// A fake for every contract the sentinel talks to: no git, no cargo, no
// sleeping. Syncs, runs and re-checks answer from queues -- a re-check with
// none queued confirms every mutant it was given --; everything else is
// recorded.
// With `pause_limit` set, the pause after that many panics, so a loop that
// never returns can be stopped.
#[derive(Default)]
pub struct World {
    pub now: Cell<i64>,
    pub syncs: RefCell<VecDeque<Result<Snapshot, SyncError>>>,
    pub checkout_error: RefCell<Option<CheckoutError>>,
    pub checkouts: Cell<usize>,
    pub committed: Cell<Option<i64>>,
    pub removals: Cell<usize>,
    pub runs: RefCell<VecDeque<RunOutcome>>,
    pub run_calls: RefCell<Vec<(String, u32, Duration)>>,
    pub rechecks: RefCell<VecDeque<RunOutcome>>,
    pub recheck_calls: RefCell<Vec<Vec<String>>>,
    pub publishes: RefCell<Vec<Publication>>,
    pub publish_errors: RefCell<VecDeque<PublishError>>,
    pub pauses: RefCell<Vec<Duration>>,
    pub pause_limit: Cell<Option<usize>>,
    pub lines: RefCell<Vec<String>>,
}

impl World {
    pub fn queue_recheck(&self, outcome: RunOutcome) {
        self.rechecks.borrow_mut().push_back(outcome);
    }

    pub fn queue_run(&self, outcome: RunOutcome) {
        self.runs.borrow_mut().push_back(outcome);
    }

    pub fn queue_sync(&self, result: Result<Snapshot, SyncError>) {
        self.syncs.borrow_mut().push_back(result);
    }
}

impl RepoSource for World {
    fn sync(&self, repo: &OnboardedRepo) -> Result<Snapshot, SyncError> {
        self.syncs
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| panic!("unexpected sync of {}", repo.name))
    }

    fn checkout(&self, _: &OnboardedRepo, _: &CommitHash) -> Result<String, CheckoutError> {
        self.checkouts.set(self.checkouts.get() + 1);
        match self.checkout_error.borrow().clone() {
            Some(error) => Err(error),
            None => Ok(String::from("/checkout")),
        }
    }

    fn remove_checkout(&self, _: &OnboardedRepo) {
        self.removals.set(self.removals.get() + 1);
    }

    fn committed_at(&self, _: &OnboardedRepo, _: &CommitHash) -> Option<Timestamp> {
        self.committed.get().map(Timestamp::from_unix)
    }
}

impl MutationRunner for World {
    fn run(&self, _: &OnboardedRepo, checkout: &str, jobs: u32, timeout: Duration) -> MutationRun {
        self.run_calls
            .borrow_mut()
            .push((checkout.to_string(), jobs, timeout));
        self.now.set(self.now.get() + RUN_SECONDS);
        MutationRun {
            outcome: self.runs.borrow_mut().pop_front().expect("unexpected run"),
            cargo_mutants_version: "cargo-mutants 27.1.0".to_string(),
            rustc_version: "rustc 1.97.0".to_string(),
            arguments: format!("mutants --jobs {jobs}"),
        }
    }

    fn recheck(
        &self,
        _: &OnboardedRepo,
        _: &str,
        _: u32,
        _: Duration,
        mutants: &[String],
    ) -> RunOutcome {
        self.recheck_calls.borrow_mut().push(mutants.to_vec());
        self.rechecks
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| RunOutcome::Completed {
                counts: MutantCounts::default(),
                missed: mutants.to_vec(),
                timed_out: vec![],
            })
    }
}

impl ResultStore for World {
    fn publish(&self, _: &OnboardedRepo, publication: &Publication) -> Result<(), PublishError> {
        if let Some(error) = self.publish_errors.borrow_mut().pop_front() {
            return Err(error);
        }
        self.publishes.borrow_mut().push(publication.clone());
        Ok(())
    }
}

impl Clock for World {
    fn now(&self) -> Timestamp {
        Timestamp::from_unix(self.now.get())
    }
}

impl Pause for World {
    fn pause(&self, duration: Duration) {
        self.pauses.borrow_mut().push(duration);
        if self
            .pause_limit
            .get()
            .is_some_and(|limit| self.pauses.borrow().len() >= limit)
        {
            panic!("stopped after {} pauses", self.pauses.borrow().len());
        }
        self.now
            .set(self.now.get() + i64::try_from(duration.as_secs()).unwrap());
    }
}

impl Journal for World {
    fn line(&self, text: &str) {
        self.lines.borrow_mut().push(text.to_string());
    }
}
