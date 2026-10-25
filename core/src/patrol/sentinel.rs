// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The loop (`ADR-PollingPerRepository`): visit one due repository at a time, run, publish, back off.

use std::time::Duration;

use crate::model::branch::record_currency::RecordCurrency;
use crate::model::commit_hash::CommitHash;
use crate::model::mutation_run::MutationRun;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::onboarding::roster::Roster;
use crate::model::round_step::RoundStep;
use crate::model::run_outcome::RunOutcome;
use crate::model::skip_reason::SkipReason;
use crate::model::sync_error::SyncError;
use crate::patrol::dependencies::Dependencies;
use crate::patrol::journal_line::JournalLine;
use crate::patrol::next::Next;
use crate::patrol::publication_builder::PublicationBuilder;
use crate::patrol::schedule::Schedule;

/// Tool failures on one commit before `tool-failed` is published.
const MAX_TOOL_FAILURES: u32 = 3;
/// Never sleep less than this, so a clock that does not move cannot spin the loop.
const MIN_SLEEP: Duration = Duration::from_secs(1);

/// The service.
pub struct Sentinel {
    roster: Roster,
    deps: Dependencies,
    schedule: Schedule,
}

impl Sentinel {
    /// The sentinel for `roster`.
    pub fn new(roster: Roster, deps: Dependencies) -> Self {
        let schedule = Schedule::new(roster.pause);
        Self {
            roster,
            deps,
            schedule,
        }
    }

    /// Runs forever.
    pub fn run_forever(&mut self) -> ! {
        loop {
            self.step();
        }
    }

    /// Visits every repository once, in roster order, ignoring the schedule.
    pub fn run_once(&mut self) -> Vec<RoundStep> {
        (0..self.roster.repos.len())
            .map(|index| self.visit_and_log(index))
            .collect()
    }

    /// Visits the next due repository, or sleeps until one is due.
    pub fn step(&mut self) -> Option<RoundStep> {
        let now = self.deps.clock.now();
        match self.schedule.next(&self.roster.repos, now) {
            Next::Visit(index) => Some(self.visit_and_log(index)),
            Next::SleepUntil(at) => {
                self.deps.pause.pause(at.since(now).max(MIN_SLEEP));
                None
            }
        }
    }

    /// Checks out `head`, runs, and on a baseline failure runs once more (flaky tests); always removes the
    /// checkout.
    fn run_with_baseline_retry(
        &self,
        repo: &OnboardedRepo,
        head: &CommitHash,
    ) -> Result<MutationRun, String> {
        let checkout = self.deps.source.checkout(repo, head);
        let result = checkout.map(|dir| {
            let run_once = || {
                self.deps
                    .runner
                    .run(repo, &dir, repo.jobs, self.roster.run_timeout)
            };
            let first = run_once();
            if matches!(first.outcome, RunOutcome::BaselineFailed { .. }) {
                run_once()
            } else {
                first
            }
        });
        self.deps.source.remove_checkout(repo);
        result.map_err(|e| format!("checkout: {e}"))
    }

    fn visit(&mut self, repo: &OnboardedRepo) -> RoundStep {
        let name = repo.name.as_str();
        let now = self.deps.clock.now();
        let snapshot = match self.deps.source.sync(repo) {
            Ok(snapshot) => snapshot,
            Err(SyncError::Unreachable(reason)) => {
                self.schedule.unreachable(name, now);
                return RoundStep::Unreachable { reason };
            }
            Err(SyncError::Transient(reason)) => {
                self.schedule.failed(name, now);
                return RoundStep::BackedOff { reason };
            }
            Err(SyncError::NoBranch) => {
                self.schedule.succeeded(name);
                return RoundStep::Skipped {
                    reason: SkipReason::NoBranch,
                };
            }
        };
        let head = snapshot.head.clone();
        if snapshot
            .record
            .as_ref()
            .is_some_and(|r| RecordCurrency::new(self.roster.config_version).is_current(r, &head))
        {
            self.schedule.succeeded(name);
            self.schedule.clear_tool_failures(name);
            return RoundStep::Unchanged { head };
        }
        if let Err(reason) = snapshot.readiness {
            self.schedule.succeeded(name);
            return RoundStep::Skipped { reason };
        }

        let started = self.deps.clock.now();
        let run = match self.run_with_baseline_retry(repo, &head) {
            Ok(run) => run,
            Err(reason) => {
                self.schedule.failed(name, self.deps.clock.now());
                return RoundStep::BackedOff { reason };
            }
        };
        let finished = self.deps.clock.now();

        if let RunOutcome::ToolFailed { reason, .. } = &run.outcome
            && self.schedule.tool_failure(name, &head) < MAX_TOOL_FAILURES
        {
            self.schedule.failed(name, finished);
            return RoundStep::BackedOff {
                reason: reason.clone(),
            };
        }
        let publication = PublicationBuilder::new(self.roster.config_version).build(
            &run,
            &head,
            started,
            finished,
            &snapshot.previous_missed,
        );
        match self.deps.store.publish(repo, &publication) {
            Ok(()) => {
                self.schedule.succeeded(name);
                self.schedule.clear_tool_failures(name);
                let r = &publication.record;
                RoundStep::Published {
                    head,
                    outcome: r.outcome,
                    counts: r.counts,
                    new: r.new,
                    gone: r.gone,
                }
            }
            Err(reason) => {
                self.schedule.failed(name, self.deps.clock.now());
                RoundStep::BackedOff {
                    reason: format!("publish: {reason}"),
                }
            }
        }
    }

    fn visit_and_log(&mut self, index: usize) -> RoundStep {
        let repo = self.roster.repos[index].clone();
        let started = self.deps.clock.now();
        let was_unreachable = self.schedule.is_unreachable(&repo.name);
        let step = self.visit(&repo);
        let finished = self.deps.clock.now();
        self.schedule.visited(&repo.name, finished);
        let repeat = was_unreachable && matches!(step, RoundStep::Unreachable { .. });
        if !repeat {
            self.deps
                .journal
                .line(&JournalLine.visit(&repo, &step, finished.since(started)));
        }
        step
    }
}
