// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The loop (`ADR-PollingPerRepository`): visit one due repository at a time, run, publish, back off.

use std::time::Duration;

use crate::model::branch::record_currency::RecordCurrency;
use crate::model::branch::record_state::RecordState;
use crate::model::commit_hash::CommitHash;
use crate::model::mutant_key::MutantKey;
use crate::model::onboarding::onboarded_repo::OnboardedRepo;
use crate::model::onboarding::roster::Roster;
use crate::model::round_step::RoundStep;
use crate::model::run_outcome::RunOutcome;
use crate::model::skip_reason::SkipReason;
use crate::model::snapshot::Snapshot;
use crate::model::sync_error::SyncError;
use crate::patrol::baseline_retry::BaselineRetry;
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
    retry: BaselineRetry,
    currency: RecordCurrency,
}

impl Sentinel {
    /// The sentinel for `roster`, where a record made by another cargo-mutants than `cargo_mutants_version` is not
    /// current (`None`: the installed version could not be read, and decides nothing).
    pub fn new(roster: Roster, deps: Dependencies, cargo_mutants_version: Option<String>) -> Self {
        let schedule = Schedule::new(roster.pause);
        let retry = BaselineRetry::new(deps.source.clone(), deps.runner.clone());
        let currency = RecordCurrency::new(roster.config_version, cargo_mutants_version);
        Self {
            roster,
            deps,
            schedule,
            retry,
            currency,
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

    fn run_and_publish(
        &mut self,
        repo: &OnboardedRepo,
        head: CommitHash,
        previous_missed: &[MutantKey],
    ) -> RoundStep {
        let name = repo.name.as_str();
        let started = self.deps.clock.now();
        let run = match self.retry.run(repo, &head, repo.run_timeout) {
            Ok(run) => run,
            Err(error) => {
                self.schedule.failed(name, self.deps.clock.now());
                return RoundStep::BackedOff {
                    reason: format!("checkout: {error}"),
                };
            }
        };
        let finished = self.deps.clock.now();
        if let RunOutcome::ToolchainMissing { message } = &run.outcome {
            self.schedule.succeeded(name);
            return RoundStep::Skipped {
                reason: SkipReason::ToolchainMissing {
                    message: message.clone(),
                },
            };
        }
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
            previous_missed,
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

    fn visit(&mut self, repo: &OnboardedRepo) -> RoundStep {
        let name = repo.name.as_str();
        let now = self.deps.clock.now();
        match self.deps.source.sync(repo) {
            Ok(snapshot) => self.visit_snapshot(repo, snapshot),
            Err(SyncError::Unreachable(reason)) => {
                self.schedule.unreachable(name, now);
                RoundStep::Unreachable { reason }
            }
            Err(SyncError::Transient(reason)) => {
                self.schedule.failed(name, now);
                RoundStep::BackedOff { reason }
            }
            Err(SyncError::NoBranch) => {
                self.schedule.succeeded(name);
                RoundStep::Skipped {
                    reason: SkipReason::NoBranch,
                }
            }
        }
    }

    fn visit_and_log(&mut self, index: usize) -> RoundStep {
        let repo = self.roster.repos[index].clone();
        let started = self.deps.clock.now();
        let was_unreachable = self.schedule.is_unreachable(&repo.name);
        let was_unreadable = self.schedule.is_unreadable(&repo.name);
        let step = self.visit(&repo);
        let finished = self.deps.clock.now();
        self.schedule.visited(&repo.name, finished);
        let repeat = match step {
            RoundStep::Unreachable { .. } => was_unreachable,
            RoundStep::Unreadable { .. } => was_unreadable,
            _ => false,
        };
        if !repeat {
            self.deps
                .journal
                .line(&JournalLine.visit(&repo, &step, finished.since(started)));
        }
        step
    }

    fn visit_snapshot(&mut self, repo: &OnboardedRepo, snapshot: Snapshot) -> RoundStep {
        let name = repo.name.as_str();
        let Snapshot {
            head,
            record,
            previous_missed,
            readiness,
        } = snapshot;
        let current = match record {
            RecordState::Unreadable(error) => {
                self.schedule.unreadable(name, self.deps.clock.now());
                return RoundStep::Unreadable {
                    reason: error.to_string(),
                };
            }
            RecordState::Present(record) => self.currency.is_current(&record, &head),
            RecordState::Absent => false,
        };
        if current {
            self.schedule.succeeded(name);
            self.schedule.clear_tool_failures(name);
            return RoundStep::Unchanged { head };
        }
        if let Err(reason) = readiness {
            self.schedule.succeeded(name);
            return RoundStep::Skipped { reason };
        }
        self.run_and_publish(repo, head, &previous_missed)
    }
}
