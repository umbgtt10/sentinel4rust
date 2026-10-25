// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::model::round_step::RoundStep;
use sentinel4rust::model::skip_reason::SkipReason;

fn every_step() -> [RoundStep; 5] {
    let head = CommitHash::parse(&"a".repeat(40)).unwrap();
    [
        RoundStep::Unchanged { head: head.clone() },
        RoundStep::Published {
            head,
            outcome: OutcomeKind::Completed,
            counts: MutantCounts::default(),
            new: 0,
            gone: 0,
        },
        RoundStep::BackedOff {
            reason: String::from("network"),
        },
        RoundStep::Unreachable {
            reason: String::from("gone"),
        },
        RoundStep::Skipped {
            reason: SkipReason::NoFilters,
        },
    ]
}

#[test]
fn is_failure_of_a_back_off_or_unreachable_alone_is_true() {
    // Arrange
    let steps = every_step();

    // Act
    let mut failures = Vec::new();
    for step in &steps {
        failures.push(step.is_failure());
    }

    // Assert
    assert_eq!(failures, [false, false, true, true, false]);
}

#[test]
fn name_of_each_step_is_its_journal_text() {
    // Arrange
    let steps = every_step();

    let names = [
        "unchanged",
        "published",
        "backed-off",
        "unreachable",
        "skipped",
    ];

    // Act & Assert
    for (step, name) in steps.iter().zip(names) {
        assert_eq!(step.name(), name);
    }
}
