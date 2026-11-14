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
        .files("twin4rust", p)
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
fn files_of_any_run_lead_with_the_shared_readme_at_the_root() {
    // Arrange
    let completed = publication(OutcomeKind::Completed, false);

    // Act
    let readme = BranchLayout
        .files("twin4rust", &completed)
        .into_iter()
        .next()
        .unwrap();

    // Assert
    assert_eq!(readme.path, "README.md");
    assert!(
        readme
            .contents
            .contains("One folder per watched repository"),
        "{}",
        readme.contents
    );
    assert!(
        readme.contents.contains("git log -p -- <repo>/"),
        "{}",
        readme.contents
    );
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
            "twin4rust/summary.toml",
            "twin4rust/missed.txt",
            "twin4rust/located/missed.txt"
        ]
    );
    assert_eq!(paths(&with).len(), 6);

    // Act
    let missed = BranchLayout
        .files("twin4rust", &with)
        .into_iter()
        .find(|file| file.path == "twin4rust/missed.txt")
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
        assert_eq!(written_paths, ["README.md", "twin4rust/summary.toml"]);
    }
}

// A run with no timeout says nothing of them: the clause is there to keep a run
// whose mutants mostly timed out from reading as a clean one.
#[test]
fn message_of_a_completed_run_with_no_timeout_leaves_them_out() {
    // Arrange
    let mut publication = publication(OutcomeKind::Completed, false);
    publication.record.counts.timeout = 0;

    // Act
    let message = BranchLayout.message("twin4rust", &publication);

    // Assert
    assert_eq!(
        message,
        "twin4rust: Mutants of abababa: 2 missed (+1 -0), 10 caught"
    );
}

#[test]
fn message_of_each_outcome_follows_the_design() {
    // Arrange & Act & Assert
    assert_eq!(
        BranchLayout.message("twin4rust", &publication(OutcomeKind::Completed, false)),
        "twin4rust: Mutants of abababa: 2 missed (+1 -0), 10 caught, 1 timed out"
    );
    assert_eq!(
        BranchLayout.message(
            "twin4rust",
            &publication(OutcomeKind::BaselineFailed, false)
        ),
        "twin4rust: Mutants of abababa: baseline failed"
    );
    assert_eq!(
        BranchLayout.message("twin4rust", &publication(OutcomeKind::ToolFailed, false)),
        "twin4rust: Mutants of abababa: tool failed"
    );
}
