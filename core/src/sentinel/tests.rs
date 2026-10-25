// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The loop with a fake for every contract: no git, no cargo, no sleeping.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use super::*;
use crate::model::commit_hash::CommitHash;
use crate::model::skip_reason::SkipReason;
use crate::schedule::MAX_BACKOFF;
use crate::traits::Snapshot;

const PAUSE: Duration = Duration::from_secs(300);
const RUN_SECONDS: i64 = 60;

#[derive(Default)]
struct World {
    now: Cell<i64>,
    syncs: RefCell<VecDeque<Result<Snapshot, SyncError>>>,
    checkout_error: RefCell<Option<String>>,
    checkouts: Cell<usize>,
    removals: Cell<usize>,
    runs: RefCell<VecDeque<RunOutcome>>,
    run_calls: RefCell<Vec<(PathBuf, u32, Duration)>>,
    publishes: RefCell<Vec<Publication>>,
    publish_errors: RefCell<VecDeque<String>>,
    pauses: RefCell<Vec<Duration>>,
    lines: RefCell<Vec<String>>,
}

impl RepoSource for World {
    fn sync(&self, repo: &OnboardedRepo) -> Result<Snapshot, SyncError> {
        self.syncs
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| panic!("unexpected sync of {}", repo.name))
    }

    fn checkout(&self, _: &OnboardedRepo, _: &CommitHash) -> Result<PathBuf, String> {
        self.checkouts.set(self.checkouts.get() + 1);
        match self.checkout_error.borrow().clone() {
            Some(error) => Err(error),
            None => Ok(PathBuf::from("/checkout")),
        }
    }

    fn remove_checkout(&self, _: &OnboardedRepo) {
        self.removals.set(self.removals.get() + 1);
    }
}

impl MutationRunner for World {
    fn run(&self, _: &OnboardedRepo, checkout: &Path, jobs: u32, timeout: Duration) -> MutationRun {
        self.run_calls
            .borrow_mut()
            .push((checkout.to_path_buf(), jobs, timeout));
        self.now.set(self.now.get() + RUN_SECONDS);
        MutationRun {
            outcome: self.runs.borrow_mut().pop_front().expect("unexpected run"),
            cargo_mutants_version: "cargo-mutants 27.1.0".to_string(),
            rustc_version: "rustc 1.97.0".to_string(),
            arguments: format!("mutants --jobs {jobs}"),
        }
    }
}

impl ResultStore for World {
    fn publish(&self, _: &OnboardedRepo, publication: &Publication) -> Result<(), String> {
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
        self.now
            .set(self.now.get() + i64::try_from(duration.as_secs()).unwrap());
    }
}

impl Journal for World {
    fn line(&self, text: &str) {
        self.lines.borrow_mut().push(text.to_string());
    }
}

fn hash(c: char) -> CommitHash {
    CommitHash::parse(&c.to_string().repeat(40)).unwrap()
}

fn repo(name: &str) -> OnboardedRepo {
    OnboardedRepo {
        name: name.to_string(),
        url: format!("git@x:{name}.git"),
        branch: "main".to_string(),
        jobs: 4,
    }
}

fn sentinel(names: &[&str]) -> (Sentinel, Rc<World>) {
    let world = Rc::new(World::default());
    world.now.set(1_792_800_000);
    let roster = Roster {
        work_dir: PathBuf::from("/w"),
        pause: PAUSE,
        jobs: 1,
        run_timeout: Duration::from_secs(3600),
        config_version: 1,
        repos: names.iter().map(|n| repo(n)).collect(),
    };
    let deps = Dependencies {
        source: Box::new(world.clone()),
        runner: Box::new(world.clone()),
        store: Box::new(world.clone()),
        clock: Box::new(world.clone()),
        pause: Box::new(world.clone()),
        journal: Box::new(world.clone()),
    };
    (Sentinel::new(roster, deps), world)
}

fn record_for(commit: &CommitHash, config_version: i64) -> RunRecord {
    RunRecord {
        commit: commit.clone(),
        config_version,
        kind: KIND_FULL.to_string(),
        outcome: OutcomeKind::Completed,
        started: Timestamp::from_unix(0),
        finished: Timestamp::from_unix(0),
        counts: MutantCounts::default(),
        new: 0,
        gone: 0,
        cargo_mutants_version: String::new(),
        rustc_version: String::new(),
        arguments: String::new(),
        reason: None,
        log_tail: None,
    }
}

fn snapshot(head: &CommitHash, record: Option<RunRecord>, previous: &[&str]) -> Snapshot {
    Snapshot {
        head: head.clone(),
        record,
        previous_missed: previous.iter().map(|k| MutantKey::new(*k)).collect(),
        readiness: Ok(()),
    }
}

fn completed(missed: &[&str]) -> RunOutcome {
    RunOutcome::Completed {
        counts: MutantCounts {
            caught: 7,
            missed: missed.len() as u64,
            timeout: 0,
            unviable: 1,
        },
        missed: missed.iter().map(ToString::to_string).collect(),
        timed_out: vec![],
    }
}

fn tool_failed() -> RunOutcome {
    RunOutcome::ToolFailed {
        reason: "cargo-mutants exited with 70".to_string(),
        log_tail: "boom".to_string(),
    }
}

fn baseline_failed() -> RunOutcome {
    RunOutcome::BaselineFailed {
        log_tail: "test x ... FAILED".to_string(),
    }
}

impl World {
    fn queue_sync(&self, result: Result<Snapshot, SyncError>) {
        self.syncs.borrow_mut().push_back(result);
    }

    fn queue_run(&self, outcome: RunOutcome) {
        self.runs.borrow_mut().push_back(outcome);
    }
}

#[test]
fn an_unchanged_main_publishes_nothing() {
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('a');
    w.queue_sync(Ok(snapshot(&head, Some(record_for(&head, 1)), &[])));
    assert_eq!(s.run_once(), [RoundStep::Unchanged { head }]);
    assert!(w.run_calls.borrow().is_empty());
    assert!(w.publishes.borrow().is_empty());
    assert_eq!(w.checkouts.get(), 0);
    assert_eq!(
        w.lines.borrow().as_slice(),
        ["repo=a step=unchanged hash=aaaaaaa duration_s=0"]
    );
}

#[test]
fn a_moved_main_runs_once_and_publishes_keys_and_new_and_gone() {
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('b');
    w.queue_sync(Ok(snapshot(
        &head,
        Some(record_for(&hash('a'), 1)),
        &["src/a.rs: kept", "src/a.rs: fixed"],
    )));
    w.queue_run(completed(&["src/a.rs:9:1: new one", "src/a.rs:2:1: kept"]));
    let steps = s.run_once();
    assert_eq!(
        steps,
        [RoundStep::Published {
            head: head.clone(),
            outcome: OutcomeKind::Completed,
            counts: MutantCounts {
                caught: 7,
                missed: 2,
                timeout: 0,
                unviable: 1
            },
            new: 1,
            gone: 1,
        }]
    );
    assert_eq!(
        w.run_calls.borrow().as_slice(),
        [(PathBuf::from("/checkout"), 4, Duration::from_secs(3600))]
    );
    assert_eq!((w.checkouts.get(), w.removals.get()), (1, 1));
    let p = &w.publishes.borrow()[0];
    assert_eq!(
        p.missed,
        [
            MutantKey::new("src/a.rs: kept"),
            MutantKey::new("src/a.rs: new one")
        ]
    );
    assert_eq!(
        p.located_missed,
        ["src/a.rs:2:1: kept", "src/a.rs:9:1: new one"]
    );
    assert_eq!(p.record.commit, head);
    assert_eq!(p.record.finished.since(p.record.started).as_secs(), 60);
    assert_eq!(p.record.arguments, "mutants --jobs 4");
    assert!(w.lines.borrow()[0].starts_with(
        "repo=a step=published hash=bbbbbbb outcome=completed missed=2 caught=7 timeout=0 unviable=1 new=1 gone=1"
    ));
}

#[test]
fn the_first_run_publishes_and_everything_is_new() {
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Ok(snapshot(&hash('a'), None, &[])));
    w.queue_run(completed(&["src/a.rs:1:1: x"]));
    assert!(matches!(
        s.run_once()[0],
        RoundStep::Published {
            new: 1,
            gone: 0,
            ..
        }
    ));
}

#[test]
fn a_config_version_bump_reruns_the_same_commit() {
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('a');
    w.queue_sync(Ok(snapshot(&head, Some(record_for(&head, 0)), &[])));
    w.queue_run(completed(&[]));
    assert!(matches!(s.run_once()[0], RoundStep::Published { .. }));
    assert_eq!(w.publishes.borrow()[0].record.config_version, 1);
}

#[test]
fn a_flaky_baseline_is_retried_once() {
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Ok(snapshot(&hash('a'), None, &[])));
    w.queue_run(baseline_failed());
    w.queue_run(completed(&[]));
    assert!(matches!(
        s.run_once()[0],
        RoundStep::Published {
            outcome: OutcomeKind::Completed,
            ..
        }
    ));
    assert_eq!(w.run_calls.borrow().len(), 2);
    assert_eq!(w.removals.get(), 1);
}

#[test]
fn a_second_baseline_failure_publishes_baseline_failed_without_lists() {
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Ok(snapshot(&hash('a'), None, &["src/a.rs: old"])));
    w.queue_run(baseline_failed());
    w.queue_run(baseline_failed());
    assert!(matches!(
        s.run_once()[0],
        RoundStep::Published {
            outcome: OutcomeKind::BaselineFailed,
            new: 0,
            gone: 0,
            ..
        }
    ));
    let p = &w.publishes.borrow()[0];
    assert_eq!(p.missed, Vec::<MutantKey>::new());
    assert_eq!(p.record.log_tail.as_deref(), Some("test x ... FAILED"));
    assert_eq!(p.message(), "Mutants of aaaaaaa: baseline failed");
}

#[test]
fn tool_failures_back_off_and_the_third_on_one_commit_publishes_tool_failed() {
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('a');
    for _ in 0..3 {
        w.queue_sync(Ok(snapshot(&head, None, &[])));
        w.queue_run(tool_failed());
    }
    let reason = "cargo-mutants exited with 70".to_string();
    assert_eq!(
        s.run_once(),
        [RoundStep::BackedOff {
            reason: reason.clone()
        }]
    );
    assert_eq!(
        s.run_once(),
        [RoundStep::BackedOff {
            reason: reason.clone()
        }]
    );
    assert!(w.publishes.borrow().is_empty());
    assert!(matches!(
        s.run_once()[0],
        RoundStep::Published {
            outcome: OutcomeKind::ToolFailed,
            ..
        }
    ));
    let p = &w.publishes.borrow()[0];
    assert_eq!(p.record.reason.as_deref(), Some(reason.as_str()));
    assert_eq!(p.record.log_tail.as_deref(), Some("boom"));
}

#[test]
fn a_new_commit_restarts_the_tool_failure_count() {
    let (mut s, w) = sentinel(&["a"]);
    for c in ['a', 'a', 'b', 'b'] {
        w.queue_sync(Ok(snapshot(&hash(c), None, &[])));
        w.queue_run(tool_failed());
    }
    for _ in 0..4 {
        assert!(matches!(s.run_once()[0], RoundStep::BackedOff { .. }));
    }
    assert!(w.publishes.borrow().is_empty());
}

#[test]
fn repositories_not_ready_are_skipped_without_running() {
    let (mut s, w) = sentinel(&["a"]);
    for reason in [SkipReason::NotACrate, SkipReason::NoFilters] {
        let mut snap = snapshot(&hash('a'), None, &[]);
        snap.readiness = Err(reason);
        w.queue_sync(Ok(snap));
        assert_eq!(s.run_once(), [RoundStep::Skipped { reason }]);
    }
    w.queue_sync(Err(SyncError::NoBranch));
    assert_eq!(
        s.run_once(),
        [RoundStep::Skipped {
            reason: SkipReason::NoBranch
        }]
    );
    assert!(w.run_calls.borrow().is_empty());
    assert_eq!(
        w.lines.borrow()[1],
        "repo=a step=skipped reason=no-filters duration_s=0"
    );
}

#[test]
fn unreachable_is_logged_once_until_it_recovers() {
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('a');
    w.queue_sync(Err(SyncError::Unreachable(
        "Repository not found".to_string(),
    )));
    w.queue_sync(Err(SyncError::Unreachable(
        "Repository not found".to_string(),
    )));
    w.queue_sync(Ok(snapshot(&head, Some(record_for(&head, 1)), &[])));
    w.queue_sync(Err(SyncError::Unreachable(
        "Repository not found".to_string(),
    )));
    s.run_once();
    s.run_once();
    s.run_once();
    s.run_once();
    let lines = w.lines.borrow();
    let steps: Vec<&str> = lines.iter().map(|l| l.split(' ').nth(1).unwrap()).collect();
    assert_eq!(
        steps,
        ["step=unreachable", "step=unchanged", "step=unreachable"]
    );
}

#[test]
fn transient_fetch_failures_checkout_failures_and_rejected_pushes_back_off() {
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Err(SyncError::Transient("timed out".to_string())));
    assert_eq!(
        s.run_once(),
        [RoundStep::BackedOff {
            reason: "timed out".to_string()
        }]
    );

    *w.checkout_error.borrow_mut() = Some("disk full".to_string());
    w.queue_sync(Ok(snapshot(&hash('a'), None, &[])));
    assert_eq!(
        s.run_once(),
        [RoundStep::BackedOff {
            reason: "checkout: disk full".to_string()
        }]
    );
    assert_eq!(w.removals.get(), 1, "a failed checkout is still cleaned up");
    *w.checkout_error.borrow_mut() = None;

    w.publish_errors
        .borrow_mut()
        .push_back("rejected".to_string());
    w.queue_sync(Ok(snapshot(&hash('a'), None, &[])));
    w.queue_run(completed(&[]));
    assert_eq!(
        s.run_once(),
        [RoundStep::BackedOff {
            reason: "publish: rejected".to_string()
        }]
    );
    // The next visit simply reruns and publishes.
    w.queue_sync(Ok(snapshot(&hash('a'), None, &[])));
    w.queue_run(completed(&[]));
    assert!(matches!(s.run_once()[0], RoundStep::Published { .. }));
}

#[test]
fn step_follows_the_polling_interval() {
    let (mut s, w) = sentinel(&["a", "b"]);
    let head = hash('a');
    for _ in 0..3 {
        w.queue_sync(Ok(snapshot(&head, Some(record_for(&head, 1)), &[])));
    }
    assert!(s.step().is_some()); // a
    assert!(s.step().is_some()); // b
    assert_eq!(s.step(), None); // nothing due: sleep the full pause
    assert_eq!(w.pauses.borrow().as_slice(), [PAUSE]);
    assert!(s.step().is_some()); // a again
    let lines = w.lines.borrow();
    let repos: Vec<&str> = lines.iter().map(|l| l.split(' ').next().unwrap()).collect();
    assert_eq!(repos, ["repo=a", "repo=b", "repo=a"]);
}

#[test]
fn step_waits_out_the_back_off() {
    let (mut s, w) = sentinel(&["a"]);
    for _ in 0..3 {
        w.queue_sync(Err(SyncError::Transient("network".to_string())));
    }
    s.step();
    s.step(); // sleeps the first back-off: one pause
    s.step();
    s.step(); // sleeps the second: two pauses
    s.step();
    assert_eq!(w.pauses.borrow().as_slice(), [PAUSE, PAUSE * 2]);
    assert!(w.syncs.borrow().is_empty());
}

#[test]
fn step_waits_the_cap_after_unreachable() {
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Err(SyncError::Unreachable("gone".to_string())));
    s.step();
    s.step();
    assert_eq!(w.pauses.borrow().as_slice(), [MAX_BACKOFF]);
}

#[test]
fn journal_lines_quote_reasons() {
    let line = journal_line(
        &repo("a"),
        &RoundStep::BackedOff {
            reason: "git fetch: exited with 128: fatal \"x\"".to_string(),
        },
        Duration::from_secs(3),
    );
    assert_eq!(
        line,
        r#"repo=a step=backed-off reason="git fetch: exited with 128: fatal \"x\"" duration_s=3"#
    );
}
