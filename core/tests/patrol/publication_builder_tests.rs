// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::branch::branch_constants::KIND_FULL;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::mutant_key::MutantKey;
use sentinel4rust::model::mutation_run::MutationRun;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::model::run_outcome::RunOutcome;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::patrol::publication_builder::PublicationBuilder;

const FINISHED: i64 = 1_792_803_600;
const STARTED: i64 = 1_792_800_000;

fn head() -> CommitHash {
    CommitHash::parse(&"ab".repeat(20)).unwrap()
}

fn run(outcome: RunOutcome) -> MutationRun {
    MutationRun {
        outcome,
        cargo_mutants_version: "cargo-mutants 27.1.0".to_string(),
        rustc_version: "rustc 1.99.1".to_string(),
        arguments: "mutants --jobs 2".to_string(),
    }
}

#[test]
fn build_of_a_baseline_failure_publishes_its_log_tail_without_lists() {
    // Arrange
    let failed = run(RunOutcome::BaselineFailed {
        log_tail: "test x ... FAILED".to_string(),
    });

    // Act
    let publication = PublicationBuilder::new(3).build(
        &failed,
        &head(),
        Timestamp::from_unix(STARTED),
        Timestamp::from_unix(FINISHED),
        &[MutantKey::new("src/a.rs: old")],
    );

    // Assert
    let record = &publication.record;
    assert_eq!(record.outcome, OutcomeKind::BaselineFailed);
    assert_eq!(record.counts, MutantCounts::default());
    assert_eq!((record.new, record.gone), (0, 0));
    assert_eq!(record.reason, None);
    assert_eq!(record.log_tail.as_deref(), Some("test x ... FAILED"));
    assert_eq!(publication.missed, Vec::<MutantKey>::new());
    assert_eq!(publication.located_missed, Vec::<String>::new());
}

#[test]
fn build_of_a_completed_run_keys_and_sorts_the_lists_and_counts_new_and_gone() {
    // Arrange
    let counts = MutantCounts {
        caught: 7,
        missed: 2,
        timeout: 1,
        unviable: 0,
    };
    let completed = run(RunOutcome::Completed {
        counts,
        missed: vec![
            "src/a.rs:9:1: new one".to_string(),
            "src/a.rs:2:1: kept".to_string(),
        ],
        timed_out: vec!["src/b.rs:4:1: slow".to_string()],
    });

    // Act
    let publication = PublicationBuilder::new(3).build(
        &completed,
        &head(),
        Timestamp::from_unix(STARTED),
        Timestamp::from_unix(FINISHED),
        &[
            MutantKey::new("src/a.rs: kept"),
            MutantKey::new("src/a.rs: fixed"),
        ],
    );

    // Assert
    let record = &publication.record;
    assert_eq!(record.commit, head());
    assert_eq!(record.config_version, 3);
    assert_eq!(record.kind, KIND_FULL);
    assert_eq!(record.outcome, OutcomeKind::Completed);
    assert_eq!(record.started, Timestamp::from_unix(STARTED));
    assert_eq!(record.finished, Timestamp::from_unix(FINISHED));
    assert_eq!(record.counts, counts);
    assert_eq!((record.new, record.gone), (1, 1));
    assert_eq!(record.cargo_mutants_version, "cargo-mutants 27.1.0");
    assert_eq!(record.rustc_version, "rustc 1.99.1");
    assert_eq!(record.arguments, "mutants --jobs 2");
    assert_eq!(
        (record.reason.clone(), record.log_tail.clone()),
        (None, None)
    );
    assert_eq!(
        publication.missed,
        [
            MutantKey::new("src/a.rs: kept"),
            MutantKey::new("src/a.rs: new one")
        ]
    );
    assert_eq!(publication.timed_out, [MutantKey::new("src/b.rs: slow")]);
    assert_eq!(
        publication.located_missed,
        ["src/a.rs:2:1: kept", "src/a.rs:9:1: new one"]
    );
    assert_eq!(publication.located_timed_out, ["src/b.rs:4:1: slow"]);
}

// The sentinel skips such a run rather than publish it; were one published,
// it would read as the tool failing, with rustup's message as the reason.
#[test]
fn build_of_a_missing_toolchain_is_a_tool_failure_with_its_message() {
    // Arrange
    let missing = run(RunOutcome::ToolchainMissing {
        message:
            "exited with 1: error: toolchain '1.99.1-x86_64-unknown-linux-gnu' is not installed"
                .to_string(),
    });

    // Act
    let publication = PublicationBuilder::new(3).build(
        &missing,
        &head(),
        Timestamp::from_unix(STARTED),
        Timestamp::from_unix(FINISHED),
        &[],
    );

    // Assert
    assert_eq!(publication.record.outcome, OutcomeKind::ToolFailed);
    assert_eq!(
        publication.record.reason.as_deref(),
        Some("exited with 1: error: toolchain '1.99.1-x86_64-unknown-linux-gnu' is not installed")
    );
    assert_eq!(publication.record.log_tail, None);
}

#[test]
fn build_of_a_tool_failure_publishes_its_reason_and_log_tail() {
    // Arrange
    let failed = run(RunOutcome::ToolFailed {
        reason: "run cap hit".to_string(),
        log_tail: "boom".to_string(),
    });

    // Act
    let publication = PublicationBuilder::new(3).build(
        &failed,
        &head(),
        Timestamp::from_unix(STARTED),
        Timestamp::from_unix(FINISHED),
        &[],
    );

    // Assert
    let record = &publication.record;
    assert_eq!(record.outcome, OutcomeKind::ToolFailed);
    assert_eq!(record.reason.as_deref(), Some("run cap hit"));
    assert_eq!(record.log_tail.as_deref(), Some("boom"));
    assert_eq!(publication.timed_out, Vec::<MutantKey>::new());
}
