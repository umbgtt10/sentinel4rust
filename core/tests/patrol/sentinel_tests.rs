// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The loop with a fake for every contract: no git, no cargo, no sleeping.

use crate::support::world::World;
use sentinel4rust::model::branch::branch_constants::KIND_FULL;
use sentinel4rust::model::branch::branch_layout::BranchLayout;
use sentinel4rust::model::branch::record_error::RecordError;
use sentinel4rust::model::branch::record_state::RecordState;
use sentinel4rust::model::branch::run_record::RunRecord;
use sentinel4rust::model::checkout_error::CheckoutError;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::mutant_key::MutantKey;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::onboarding::roster::Roster;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::model::publish_error::PublishError;
use sentinel4rust::model::round_step::RoundStep;
use sentinel4rust::model::run_outcome::RunOutcome;
use sentinel4rust::model::skip_reason::SkipReason;
use sentinel4rust::model::snapshot::Snapshot;
use sentinel4rust::model::sync_error::SyncError;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::patrol::dependencies::Dependencies;
use sentinel4rust::patrol::patrol_constants::MAX_BACKOFF;
use sentinel4rust::patrol::sentinel::Sentinel;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::rc::Rc;
use std::time::Duration;

const INSTALLED: &str = "cargo-mutants 27.1.0";
const PAUSE: Duration = Duration::from_secs(300);

fn baseline_failed() -> RunOutcome {
    RunOutcome::BaselineFailed {
        log_tail: "test x ... FAILED".to_string(),
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
        test_timeout_seconds: None,
    }
}

fn hash(c: char) -> CommitHash {
    CommitHash::parse(&c.to_string().repeat(40)).unwrap()
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
        cargo_mutants_version: INSTALLED.to_string(),
        rustc_version: String::new(),
        arguments: String::new(),
        reason: None,
        log_tail: None,
        test_timeout_seconds: None,
    }
}

fn repo(name: &str) -> OnboardedRepo {
    OnboardedRepo {
        name: name.to_string(),
        url: format!("git@x:{name}.git"),
        branch: "main".to_string(),
        jobs: 4,
        run_timeout: Duration::from_secs(3600),
    }
}

fn sentinel(names: &[&str]) -> (Sentinel, Rc<World>) {
    sentinel_with(names, Some(INSTALLED))
}

fn sentinel_for(repos: Vec<OnboardedRepo>, installed: Option<&str>) -> (Sentinel, Rc<World>) {
    let world = Rc::new(World::default());
    world.now.set(1_792_800_000);
    let roster = Roster {
        work_dir: String::from("/w"),
        pause: PAUSE,
        jobs: 1,
        run_timeout: Duration::from_secs(3600),
        config_version: 1,
        stale_after: Duration::from_secs(24 * 3600),
        results_url: "r".to_string(),
        github_app: None,
        discovery_interval: Duration::from_secs(3600),
        run_unit: None,
        repos,
    };
    let deps = Dependencies {
        source: world.clone(),
        runner: world.clone(),
        store: world.clone(),
        clock: world.clone(),
        pause: world.clone(),
        journal: world.clone(),
    };
    (
        Sentinel::new(roster, deps, installed.map(ToString::to_string)),
        world,
    )
}

fn sentinel_with(names: &[&str], installed: Option<&str>) -> (Sentinel, Rc<World>) {
    sentinel_for(names.iter().map(|n| repo(n)).collect(), installed)
}

fn snapshot(head: &CommitHash, record: Option<RunRecord>, previous: &[&str]) -> Snapshot {
    Snapshot {
        head: head.clone(),
        record: record.map_or(RecordState::Absent, |r| RecordState::Present(Box::new(r))),
        previous_missed: previous.iter().map(|k| MutantKey::new(*k)).collect(),
        readiness: Ok(()),
    }
}

fn tool_failed() -> RunOutcome {
    RunOutcome::ToolFailed {
        reason: "cargo-mutants exited with 70".to_string(),
        log_tail: "boom".to_string(),
    }
}

// The pause panics after its second call, which is the only way out.
#[test]
fn run_forever_steps_until_something_stops_it() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('a');
    for _ in 0..2 {
        w.queue_sync(Ok(snapshot(&head, Some(record_for(&head, 1)), &[])));
    }
    w.pause_limit.set(Some(2));

    // Act
    let stopped = catch_unwind(AssertUnwindSafe(|| s.run_forever()));

    // Assert
    assert!(stopped.is_err());
    assert_eq!(w.pauses.borrow().as_slice(), [PAUSE, PAUSE]);
    assert_eq!(w.lines.borrow().len(), 2);
}

#[test]
fn run_once_caps_the_run_at_the_repositorys_own_timeout() {
    // Arrange
    let mut large = repo("a");
    large.run_timeout = Duration::from_secs(30 * 3600);
    let (mut s, w) = sentinel_for(vec![large], Some(INSTALLED));
    w.queue_sync(Ok(snapshot(&hash('a'), None, &[])));
    w.queue_run(completed(&[]));

    // Act
    s.run_once();

    // Assert
    assert_eq!(
        w.run_calls.borrow().as_slice(),
        [(String::from("/checkout"), 4, Duration::from_secs(30 * 3600))]
    );
}

#[test]
fn run_once_config_version_bump_reruns_the_same_commit() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('a');
    w.queue_sync(Ok(snapshot(&head, Some(record_for(&head, 0)), &[])));
    w.queue_run(completed(&[]));

    // Act
    let steps = s.run_once();

    // Assert
    assert!(matches!(steps[0], RoundStep::Published { .. }));
    assert_eq!(w.publishes.borrow()[0].record.config_version, 1);
}

#[test]
fn run_once_first_run_publishes_everything_as_new() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Ok(snapshot(&hash('a'), None, &[])));
    w.queue_run(completed(&["src/a.rs:1:1: x"]));

    // Act
    let steps = s.run_once();

    // Assert
    assert!(matches!(
        steps[0],
        RoundStep::Published {
            new: 1,
            gone: 0,
            ..
        }
    ));
}

#[test]
fn run_once_flaky_baseline_is_retried_once() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Ok(snapshot(&hash('a'), None, &[])));
    w.queue_run(baseline_failed());
    w.queue_run(completed(&[]));

    // Act
    let steps = s.run_once();

    // Assert
    assert!(matches!(
        steps[0],
        RoundStep::Published {
            outcome: OutcomeKind::Completed,
            ..
        }
    ));
    assert_eq!(w.run_calls.borrow().len(), 2);
    assert_eq!(w.removals.get(), 1);
}

#[test]
fn run_once_moved_main_publishes_keys_new_and_gone() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('b');
    w.queue_sync(Ok(snapshot(
        &head,
        Some(record_for(&hash('a'), 1)),
        &["src/a.rs: kept", "src/a.rs: fixed"],
    )));
    w.queue_run(completed(&["src/a.rs:9:1: new one", "src/a.rs:2:1: kept"]));

    // Act
    let steps = s.run_once();

    // Assert
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
        [(String::from("/checkout"), 4, Duration::from_secs(3600))]
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
fn run_once_new_cargo_mutants_reruns_the_same_commit() {
    // Arrange
    let (mut s, w) = sentinel_with(&["a"], Some("cargo-mutants 27.2.0"));
    let head = hash('a');
    w.queue_sync(Ok(snapshot(&head, Some(record_for(&head, 1)), &[])));
    w.queue_run(completed(&[]));

    // Act
    let steps = s.run_once();

    // Assert
    assert!(matches!(steps[0], RoundStep::Published { .. }));
    assert_eq!(w.run_calls.borrow().len(), 1);
}

#[test]
fn run_once_new_commit_restarts_the_tool_failure_count() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    for c in ['a', 'a', 'b', 'b'] {
        w.queue_sync(Ok(snapshot(&hash(c), None, &[])));
        w.queue_run(tool_failed());
    }

    // Act & Assert
    for _ in 0..4 {
        assert!(matches!(s.run_once()[0], RoundStep::BackedOff { .. }));
    }
    assert!(w.publishes.borrow().is_empty());
}

#[test]
fn run_once_not_ready_skips_without_running() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);

    // Act & Assert
    for reason in [SkipReason::NotACrate, SkipReason::NoFilters] {
        let mut snap = snapshot(&hash('a'), None, &[]);
        snap.readiness = Err(reason.clone());
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
fn run_once_second_baseline_failure_publishes_baseline_failed() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Ok(snapshot(&hash('a'), None, &["src/a.rs: old"])));
    w.queue_run(baseline_failed());
    w.queue_run(baseline_failed());

    // Act
    let steps = s.run_once();

    // Assert
    assert!(matches!(
        steps[0],
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
    assert_eq!(
        BranchLayout.message("a", p),
        "a: Mutants of aaaaaaa: baseline failed"
    );
}

#[test]
fn run_once_third_tool_failure_on_a_commit_publishes_tool_failed() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('a');
    for _ in 0..3 {
        w.queue_sync(Ok(snapshot(&head, None, &[])));
        w.queue_run(tool_failed());
    }
    let reason = "cargo-mutants exited with 70".to_string();

    // Act & Assert
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
fn run_once_transient_failures_back_off() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Err(SyncError::Transient("timed out".to_string())));

    // Act & Assert
    assert_eq!(
        s.run_once(),
        [RoundStep::BackedOff {
            reason: "timed out".to_string()
        }]
    );

    *w.checkout_error.borrow_mut() = Some(CheckoutError("disk full".to_string()));
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
        .push_back(PublishError("rejected".to_string()));
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
fn run_once_unchanged_main_publishes_nothing() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('a');
    w.queue_sync(Ok(snapshot(&head, Some(record_for(&head, 1)), &[])));

    // Act
    let steps = s.run_once();

    // Assert
    assert_eq!(steps, [RoundStep::Unchanged { head }]);
    assert!(w.run_calls.borrow().is_empty());
    assert!(w.publishes.borrow().is_empty());
    assert_eq!(w.checkouts.get(), 0);
    assert_eq!(
        w.lines.borrow().as_slice(),
        ["repo=a step=unchanged hash=aaaaaaa duration_s=0"]
    );
}

// An installed version that could not be read is no reason to rerun.
#[test]
fn run_once_unchanged_main_with_cargo_mutants_unknown_publishes_nothing() {
    // Arrange
    let (mut s, w) = sentinel_with(&["a"], None);
    let head = hash('a');
    let mut record = record_for(&head, 1);
    record.cargo_mutants_version = "cargo-mutants 26.0.0".to_string();
    w.queue_sync(Ok(snapshot(&head, Some(record), &[])));

    // Act
    let steps = s.run_once();

    // Assert
    assert_eq!(steps, [RoundStep::Unchanged { head }]);
    assert!(w.run_calls.borrow().is_empty());
}

#[test]
fn run_once_unreachable_is_logged_once_until_it_recovers() {
    // Arrange
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

    // Act
    s.run_once();
    s.run_once();
    s.run_once();
    s.run_once();

    // Assert
    let lines = w.lines.borrow();
    let steps: Vec<&str> = lines.iter().map(|l| l.split(' ').nth(1).unwrap()).collect();
    assert_eq!(
        steps,
        ["step=unreachable", "step=unchanged", "step=unreachable"]
    );
}

// Never rerun over: the record may be one a newer schema or a hand edit
// broke, and overwriting it would lose it. Logged once, it recovers by itself.
#[test]
fn run_once_unreadable_is_logged_once_never_run_and_recovers() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    let head = hash('a');
    let unreadable = || Snapshot {
        record: RecordState::Unreadable(RecordError::Invalid("line 1: expected `=`".to_string())),
        ..snapshot(&head, None, &[])
    };
    w.queue_sync(Ok(unreadable()));
    w.queue_sync(Ok(unreadable()));
    w.queue_sync(Ok(snapshot(&head, Some(record_for(&head, 1)), &[])));
    w.queue_sync(Ok(unreadable()));

    // Act
    let steps: Vec<RoundStep> = (0..4).flat_map(|_| s.run_once()).collect();

    // Assert
    assert_eq!(
        steps[0],
        RoundStep::Unreadable {
            reason: "invalid summary: line 1: expected `=`".to_string()
        }
    );
    assert!(w.run_calls.borrow().is_empty());
    assert!(w.publishes.borrow().is_empty());
    let lines = w.lines.borrow();
    let names: Vec<&str> = lines.iter().map(|l| l.split(' ').nth(1).unwrap()).collect();
    assert_eq!(
        names,
        ["step=unreadable", "step=unchanged", "step=unreadable"]
    );
}

#[test]
fn run_once_without_the_toolchain_skips_with_rustups_message() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Ok(snapshot(&hash('a'), None, &[])));
    w.queue_run(RunOutcome::ToolchainMissing {
        message:
            "exited with 1: error: toolchain '1.99.1-x86_64-unknown-linux-gnu' is not installed"
                .to_string(),
    });

    // Act
    let steps = s.run_once();

    // Assert
    assert_eq!(
        steps,
        [RoundStep::Skipped {
            reason: SkipReason::ToolchainMissing {
                message: "exited with 1: error: toolchain '1.99.1-x86_64-unknown-linux-gnu' is not installed".to_string()
            }
        }]
    );
    assert_eq!(w.run_calls.borrow().len(), 1);
    assert!(w.publishes.borrow().is_empty());
}

#[test]
fn step_follows_the_polling_interval() {
    // Arrange
    let (mut s, w) = sentinel(&["a", "b"]);
    let head = hash('a');
    for _ in 0..3 {
        w.queue_sync(Ok(snapshot(&head, Some(record_for(&head, 1)), &[])));
    }

    // Act & Assert
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
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    for _ in 0..3 {
        w.queue_sync(Err(SyncError::Transient("network".to_string())));
    }

    // Act
    s.step();
    s.step(); // sleeps the first back-off: one pause
    s.step();
    s.step(); // sleeps the second: two pauses
    s.step();

    // Assert
    assert_eq!(w.pauses.borrow().as_slice(), [PAUSE, PAUSE * 2]);
    assert!(w.syncs.borrow().is_empty());
}

#[test]
fn step_waits_the_cap_after_unreachable() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Err(SyncError::Unreachable("gone".to_string())));

    // Act
    s.step();
    s.step();

    // Assert
    assert_eq!(w.pauses.borrow().as_slice(), [MAX_BACKOFF]);
}

#[test]
fn step_waits_the_cap_after_unreadable() {
    // Arrange
    let (mut s, w) = sentinel(&["a"]);
    w.queue_sync(Ok(Snapshot {
        record: RecordState::Unreadable(RecordError::Invalid("line 1".to_string())),
        ..snapshot(&hash('a'), None, &[])
    }));

    // Act
    s.step();
    s.step();

    // Assert
    assert_eq!(w.pauses.borrow().as_slice(), [MAX_BACKOFF]);
}
