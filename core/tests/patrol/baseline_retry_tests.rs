// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::world::World;
use sentinel4rust::model::checkout_error::CheckoutError;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::mutant_key::MutantKey;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::run_outcome::RunOutcome;
use sentinel4rust::patrol::baseline_retry::BaselineRetry;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

fn completed() -> RunOutcome {
    RunOutcome::Completed {
        counts: MutantCounts::default(),
        missed: vec![],
        timed_out: vec![],
    }
}

fn head() -> CommitHash {
    CommitHash::parse(&"a".repeat(40)).unwrap()
}

fn repo() -> OnboardedRepo {
    OnboardedRepo {
        name: "a".to_string(),
        url: "u".to_string(),
        branch: "main".to_string(),
        jobs: 3,
        run_timeout: Duration::from_secs(3600),
    }
}

fn retry(world: &Rc<World>) -> BaselineRetry {
    BaselineRetry::new(world.clone(), world.clone())
}

fn survived(missed: &[&str]) -> RunOutcome {
    RunOutcome::Completed {
        counts: MutantCounts {
            caught: 3,
            missed: missed.len() as u64,
            timeout: 0,
            unviable: 0,
        },
        missed: missed.iter().map(ToString::to_string).collect(),
        timed_out: vec![],
    }
}

#[test]
fn run_of_a_flaky_baseline_runs_once_more() {
    // Arrange
    let world = Rc::new(World::default());
    world.queue_run(RunOutcome::BaselineFailed {
        log_tail: "test x ... FAILED".to_string(),
    });
    world.queue_run(completed());

    // Act
    let run = retry(&world).run(&repo(), &head(), Duration::from_secs(60), &[]);

    // Assert
    assert_eq!(run.map(|r| r.outcome), Ok(completed()));
    assert_eq!(world.run_calls.borrow().len(), 2);
    assert_eq!(world.removals.get(), 1);
}

#[test]
fn run_of_a_passing_baseline_runs_once_in_the_checkout_and_removes_it() {
    // Arrange
    let world = Rc::new(World::default());
    world.queue_run(completed());

    // Act
    let run = retry(&world).run(&repo(), &head(), Duration::from_secs(60), &[]);

    // Assert
    assert!(run.is_ok());
    assert_eq!(
        world.run_calls.borrow().as_slice(),
        [(PathBuf::from("/checkout"), 3, Duration::from_secs(60))]
    );
    assert_eq!((world.checkouts.get(), world.removals.get()), (1, 1));
}

#[test]
fn run_with_a_failed_checkout_runs_nothing_and_still_removes_it() {
    // Arrange
    let world = Rc::new(World::default());
    *world.checkout_error.borrow_mut() = Some(CheckoutError("disk full".to_string()));

    // Act
    let run = retry(&world).run(&repo(), &head(), Duration::from_secs(60), &[]);

    // Assert
    assert_eq!(
        run.map(|r| r.outcome),
        Err(CheckoutError("disk full".to_string()))
    );
    assert!(world.run_calls.borrow().is_empty());
    assert_eq!(world.removals.get(), 1);
}

#[test]
fn run_with_a_survivor_caught_on_the_recheck_counts_it_caught() {
    // Arrange
    let world = Rc::new(World::default());
    world.queue_run(survived(&["src/a.rs:9:1: new one", "src/a.rs:2:1: kept"]));
    world.queue_recheck(survived(&[]));

    // Act
    let run = retry(&world).run(
        &repo(),
        &head(),
        Duration::from_secs(60),
        &[MutantKey::new("src/a.rs: kept")],
    );

    // Assert
    assert_eq!(
        run.map(|r| r.outcome),
        Ok(RunOutcome::Completed {
            counts: MutantCounts {
                caught: 4,
                missed: 1,
                timeout: 0,
                unviable: 0,
            },
            missed: vec!["src/a.rs:2:1: kept".to_string()],
            timed_out: vec![],
        })
    );
}

#[test]
fn run_with_new_survivors_rechecks_exactly_them_once_in_the_checkout() {
    // Arrange
    let world = Rc::new(World::default());
    world.queue_run(survived(&["src/a.rs:9:1: new one", "src/a.rs:2:1: kept"]));

    // Act
    let run = retry(&world).run(
        &repo(),
        &head(),
        Duration::from_secs(60),
        &[MutantKey::new("src/a.rs: kept")],
    );

    // Assert
    assert_eq!(
        world.recheck_calls.borrow().as_slice(),
        [vec!["src/a.rs:9:1: new one".to_string()]]
    );
    assert_eq!(
        run.map(|r| r.outcome),
        Ok(survived(&["src/a.rs:9:1: new one", "src/a.rs:2:1: kept"]))
    );
    assert_eq!(world.removals.get(), 1);
}

#[test]
fn run_without_new_survivors_rechecks_nothing() {
    // Arrange
    let world = Rc::new(World::default());
    world.queue_run(survived(&["src/a.rs:2:1: kept"]));

    // Act
    let _ = retry(&world).run(
        &repo(),
        &head(),
        Duration::from_secs(60),
        &[MutantKey::new("src/a.rs: kept")],
    );

    // Assert
    assert!(world.recheck_calls.borrow().is_empty());
}
