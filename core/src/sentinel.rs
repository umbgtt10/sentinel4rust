// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The loop (design 1): visit one due repository at a time, run, publish, back off.

use std::fmt::Write as _;
use std::time::Duration;

use crate::model::branch::branch_constants::KIND_FULL;
use crate::model::branch::publication::Publication;
use crate::model::branch::record_currency::RecordCurrency;
use crate::model::branch::run_record::RunRecord;
use crate::model::commit_hash::CommitHash;
use crate::model::counts::MutantCounts;
use crate::model::mutant_key::MutantKey;
use crate::model::mutation_run::MutationRun;
use crate::model::outcome_kind::OutcomeKind;
use crate::model::roster::OnboardedRepo;
use crate::model::roster::Roster;
use crate::model::round_step::RoundStep;
use crate::model::run_outcome::RunOutcome;
use crate::model::skip_reason::SkipReason;
use crate::model::timestamp::Timestamp;
use crate::schedule::{MAX_TOOL_FAILURES, Next, Schedule};
use crate::traits::{Clock, Journal, MutationRunner, Pause, RepoSource, ResultStore, SyncError};

/// Everything the sentinel talks to, injected once.
pub struct Dependencies {
    /// Fetches and checks out repositories.
    pub source: Box<dyn RepoSource>,
    /// Runs cargo-mutants.
    pub runner: Box<dyn MutationRunner>,
    /// Publishes on `mutants`.
    pub store: Box<dyn ResultStore>,
    /// The time.
    pub clock: Box<dyn Clock>,
    /// Sleeping.
    pub pause: Box<dyn Pause>,
    /// One line per visit.
    pub journal: Box<dyn Journal>,
}

/// The service.
pub struct Sentinel {
    roster: Roster,
    deps: Dependencies,
    schedule: Schedule,
}

/// Never sleep less than this, so a clock that does not move cannot spin the loop.
const MIN_SLEEP: Duration = Duration::from_secs(1);

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
                .line(&journal_line(&repo, &step, finished.since(started)));
        }
        step
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
        let times = (started, finished);
        let publication = publication_for(
            &run,
            &head,
            self.roster.config_version,
            times,
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
}

/// What to publish for a run that is to be published.
fn publication_for(
    run: &MutationRun,
    head: &CommitHash,
    config_version: i64,
    (started, finished): (Timestamp, Timestamp),
    previous_missed: &[MutantKey],
) -> Publication {
    let record = |outcome, counts, (new, gone), reason, log_tail| RunRecord {
        commit: head.clone(),
        config_version,
        kind: KIND_FULL.to_string(),
        outcome,
        started,
        finished,
        counts,
        new,
        gone,
        cargo_mutants_version: run.cargo_mutants_version.clone(),
        rustc_version: run.rustc_version.clone(),
        arguments: run.arguments.clone(),
        reason,
        log_tail,
    };
    match &run.outcome {
        RunOutcome::Completed {
            counts,
            missed,
            timed_out,
        } => {
            let missed_keys = MutantKey::from_located(missed);
            let changes = MutantKey::changes(previous_missed, &missed_keys);
            let new_gone = (changes.new, changes.gone);
            Publication {
                record: record(OutcomeKind::Completed, *counts, new_gone, None, None),
                missed: missed_keys,
                timed_out: MutantKey::from_located(timed_out),
                located_missed: sorted(missed),
                located_timed_out: sorted(timed_out),
            }
        }
        RunOutcome::BaselineFailed { log_tail } => empty_publication(record(
            OutcomeKind::BaselineFailed,
            MutantCounts::default(),
            (0, 0),
            None,
            Some(log_tail.clone()),
        )),
        RunOutcome::ToolFailed { reason, log_tail } => empty_publication(record(
            OutcomeKind::ToolFailed,
            MutantCounts::default(),
            (0, 0),
            Some(reason.clone()),
            Some(log_tail.clone()),
        )),
    }
}

fn sorted(lines: &[String]) -> Vec<String> {
    let mut lines = lines.to_vec();
    lines.sort();
    lines
}

fn empty_publication(record: RunRecord) -> Publication {
    Publication {
        record,
        missed: Vec::new(),
        timed_out: Vec::new(),
        located_missed: Vec::new(),
        located_timed_out: Vec::new(),
    }
}

/// `repo=… step=… hash=… …` (design 4).
pub fn journal_line(repo: &OnboardedRepo, step: &RoundStep, duration: Duration) -> String {
    let mut line = format!("repo={} step={}", repo.name, step.name());
    match step {
        RoundStep::Unchanged { head } => {
            let _ = write!(line, " hash={}", head.short());
        }
        RoundStep::Published {
            head,
            outcome,
            counts,
            new,
            gone,
        } => {
            let _ = write!(
                line,
                " hash={} outcome={outcome} missed={} caught={} timeout={} unviable={} new={new} gone={gone}",
                head.short(),
                counts.missed,
                counts.caught,
                counts.timeout,
                counts.unviable,
            );
        }
        RoundStep::BackedOff { reason } | RoundStep::Unreachable { reason } => {
            let _ = write!(line, " reason={reason:?}");
        }
        RoundStep::Skipped { reason } => {
            let _ = write!(line, " reason={}", reason.as_str());
        }
    }
    let _ = write!(line, " duration_s={}", duration.as_secs());
    line
}

/// The start of the journal's first line, for logs.
pub fn startup_line(roster: &Roster, now: Timestamp) -> String {
    format!(
        "sentinel4rust {} started at {now}: {} repositories, pause {}s, config_version {}",
        env!("CARGO_PKG_VERSION"),
        roster.repos.len(),
        roster.pause.as_secs(),
        roster.config_version
    )
}

#[cfg(test)]
mod tests;
