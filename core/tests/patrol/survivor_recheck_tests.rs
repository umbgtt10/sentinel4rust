// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::mutant_key::MutantKey;
use sentinel4rust::model::run_outcome::RunOutcome;
use sentinel4rust::patrol::survivor_recheck::SurvivorRecheck;

const KEPT: &str = "src/a.rs:2:1: replace f -> bool with true";
const NEW: &str = "src/a.rs:9:1: replace g -> u8 with 0";
const OTHER: &str = "src/b.rs:4:5: delete ! in h";

fn completed(caught: u64, missed: &[&str]) -> RunOutcome {
    RunOutcome::Completed {
        counts: MutantCounts {
            caught,
            missed: missed.len() as u64,
            timeout: 0,
            unviable: 1,
        },
        missed: missed.iter().map(ToString::to_string).collect(),
        timed_out: vec![],
    }
}

#[test]
fn apply_with_a_failed_recheck_keeps_the_first_lists() {
    // Arrange
    let first = completed(7, &[KEPT, NEW]);
    let recheck = RunOutcome::ToolFailed {
        reason: "run cap hit".to_string(),
        log_tail: String::new(),
    };

    // Act
    let outcome = SurvivorRecheck.apply(first.clone(), &[NEW.to_string()], &recheck);

    // Assert
    assert_eq!(outcome, first);
}

#[test]
fn apply_with_a_timeout_on_the_recheck_keeps_it_missed() {
    // Arrange
    let first = completed(7, &[KEPT, NEW]);
    let recheck = RunOutcome::Completed {
        counts: MutantCounts::default(),
        missed: vec![],
        timed_out: vec![NEW.to_string()],
    };

    // Act
    let outcome = SurvivorRecheck.apply(first.clone(), &[NEW.to_string()], &recheck);

    // Assert
    assert_eq!(outcome, first);
}

#[test]
fn apply_with_one_caught_on_the_recheck_counts_it_caught() {
    // Arrange
    let first = completed(7, &[KEPT, NEW, OTHER]);
    let recheck = completed(1, &[OTHER]);

    // Act
    let outcome = SurvivorRecheck.apply(first, &[NEW.to_string(), OTHER.to_string()], &recheck);

    // Assert
    assert_eq!(
        outcome,
        RunOutcome::Completed {
            counts: MutantCounts {
                caught: 8,
                missed: 2,
                timeout: 0,
                unviable: 1,
            },
            missed: vec![KEPT.to_string(), OTHER.to_string()],
            timed_out: vec![],
        }
    );
}

#[test]
fn new_survivors_of_a_completed_run_are_the_lines_whose_keys_are_new() {
    // Arrange
    let outcome = completed(7, &[NEW, KEPT, OTHER]);
    let previous = [
        MutantKey::new("src/a.rs: replace f -> bool with true"),
        MutantKey::new("src/z.rs: gone since"),
    ];

    // Act
    let names = SurvivorRecheck.new_survivors(&outcome, &previous);

    // Assert
    assert_eq!(names, [NEW, OTHER]);
}

#[test]
fn new_survivors_of_a_failed_run_are_none() {
    // Arrange
    let outcome = RunOutcome::BaselineFailed {
        log_tail: "x".to_string(),
    };

    // Act
    let names = SurvivorRecheck.new_survivors(&outcome, &[]);

    // Assert
    assert_eq!(names, Vec::<String>::new());
}
