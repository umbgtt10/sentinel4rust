// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::world::World;
use sentinel4rust::model::checkout_error::CheckoutError;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
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

#[test]
fn run_of_a_flaky_baseline_runs_once_more() {
    // Arrange
    let world = Rc::new(World::default());
    world.queue_run(RunOutcome::BaselineFailed {
        log_tail: "test x ... FAILED".to_string(),
    });
    world.queue_run(completed());

    // Act
    let run = retry(&world).run(&repo(), &head(), Duration::from_secs(60));

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
    let run = retry(&world).run(&repo(), &head(), Duration::from_secs(60));

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
    let run = retry(&world).run(&repo(), &head(), Duration::from_secs(60));

    // Assert
    assert_eq!(
        run.map(|r| r.outcome),
        Err(CheckoutError("disk full".to_string()))
    );
    assert!(world.run_calls.borrow().is_empty());
    assert_eq!(world.removals.get(), 1);
}
