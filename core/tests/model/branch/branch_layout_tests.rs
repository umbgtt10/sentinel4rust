// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::run_records::record;
use sentinel4rust::model::branch::branch_layout::BranchLayout;
use sentinel4rust::model::branch::publication::Publication;
use sentinel4rust::model::mutant_key::MutantKey;
use sentinel4rust::model::outcome_kind::OutcomeKind;

fn paths(p: &Publication) -> Vec<String> {
    BranchLayout
        .files(p)
        .into_iter()
        .map(|file| file.path)
        .collect()
}

fn publication(outcome: OutcomeKind, timed_out: bool) -> Publication {
    Publication {
        record: record(outcome),
        missed: vec![MutantKey::new("a"), MutantKey::new("b")],
        timed_out: if timed_out {
            vec![MutantKey::new("t")]
        } else {
            vec![]
        },
        located_missed: vec!["x:1:1: a".to_string(), "x:2:1: b".to_string()],
        located_timed_out: if timed_out {
            vec!["x:3:1: t".to_string()]
        } else {
            vec![]
        },
    }
}

#[test]
fn files_of_completed_run_list_timeouts_only_when_present() {
    // Arrange
    let with = publication(OutcomeKind::Completed, true);

    // Act & Assert
    assert_eq!(
        paths(&publication(OutcomeKind::Completed, false)),
        [
            "README.md",
            "summary.toml",
            "missed.txt",
            "located/missed.txt"
        ]
    );
    assert_eq!(paths(&with).len(), 6);

    // Act
    let missed = BranchLayout
        .files(&with)
        .into_iter()
        .find(|file| file.path == "missed.txt")
        .unwrap();

    // Assert
    assert_eq!(missed.contents, "a\nb\n");
}

#[test]
fn files_of_failed_run_write_no_lists() {
    // Arrange
    let outcomes = [OutcomeKind::BaselineFailed, OutcomeKind::ToolFailed];

    // Act
    let written: Vec<Vec<String>> = outcomes
        .iter()
        .map(|outcome| paths(&publication(*outcome, true)))
        .collect();

    // Assert
    for written_paths in written {
        assert_eq!(written_paths, ["README.md", "summary.toml"]);
    }
}

#[test]
fn message_of_each_outcome_follows_the_design() {
    // Arrange & Act & Assert
    assert_eq!(
        BranchLayout.message(&publication(OutcomeKind::Completed, false)),
        "Mutants of ababab: 2 missed (+1 -0), 10 caught".replace("ababab", "abababa")
    );
    assert_eq!(
        BranchLayout.message(&publication(OutcomeKind::BaselineFailed, false)),
        "Mutants of abababa: baseline failed"
    );
    assert_eq!(
        BranchLayout.message(&publication(OutcomeKind::ToolFailed, false)),
        "Mutants of abababa: tool failed"
    );
}
