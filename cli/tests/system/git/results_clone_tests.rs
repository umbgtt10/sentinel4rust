// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// `ResultsClone` against real git: the results repository is a bare
// repository in a temporary directory, reached through `file://`.

use crate::support::git_shell::git;
use crate::support::origin::Origin;
use crate::support::temp_dir::TempDir;
use sentinel4rust::model::branch::branch_constants::KIND_FULL;
use sentinel4rust::model::branch::publication::Publication;
use sentinel4rust::model::branch::record_error::RecordError;
use sentinel4rust::model::branch::record_state::RecordState;
use sentinel4rust::model::branch::run_record::RunRecord;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::mutant_key::MutantKey;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::model::sync_error::SyncError;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust_cli::system::git::git_runner::GitRunner;
use sentinel4rust_cli::system::git::results_clone::ResultsClone;
use sentinel4rust_cli::system::process::system_command_runner::SystemCommandRunner;
use sentinel4rust_cli::system::work_dir::WorkDir;
use std::fs;
use std::rc::Rc;

fn commit() -> CommitHash {
    CommitHash::parse(&"ab".repeat(20)).unwrap()
}

fn files(results: &Origin) -> Vec<String> {
    git(&results.bare(), &["ls-tree", "-r", "--name-only", "main"])
        .lines()
        .map(String::from)
        .collect()
}

fn publication(missed: &[&str], timed_out: &[&str]) -> Publication {
    let located = |names: &[&str]| -> Vec<String> {
        names
            .iter()
            .enumerate()
            .map(|(i, m)| format!("src/lib.rs:{}:1: {m}", i + 1))
            .collect()
    };
    let (located_missed, located_timed_out) = (located(missed), located(timed_out));
    let (missed, timed_out) = (
        MutantKey::from_located(&located_missed),
        MutantKey::from_located(&located_timed_out),
    );
    Publication {
        record: RunRecord {
            commit: commit(),
            config_version: 1,
            kind: KIND_FULL.to_string(),
            outcome: OutcomeKind::Completed,
            started: Timestamp::from_unix(1_792_800_000),
            finished: Timestamp::from_unix(1_792_800_060),
            counts: MutantCounts {
                caught: 4,
                missed: missed.len() as u64,
                timeout: timed_out.len() as u64,
                unviable: 0,
            },
            new: missed.len() as u64,
            gone: 0,
            cargo_mutants_version: "cargo-mutants 27.1.0".to_string(),
            rustc_version: "rustc 1.99.1".to_string(),
            arguments: "mutants".to_string(),
            reason: None,
            log_tail: None,
            test_timeout_seconds: None,
        },
        missed,
        timed_out,
        located_missed,
        located_timed_out,
    }
}

fn results_clone(work: &TempDir, results: &Origin) -> ResultsClone {
    ResultsClone::new(
        GitRunner::new(Rc::new(SystemCommandRunner)),
        WorkDir::new(work.path().to_path_buf()),
        results.url(),
    )
}

#[test]
fn fetch_of_an_unreachable_results_repository_is_unreachable() {
    // Arrange
    let results = Origin::new();
    let work = TempDir::new("work");
    let clone = ResultsClone::new(
        GitRunner::new(Rc::new(SystemCommandRunner)),
        WorkDir::new(work.path().to_path_buf()),
        format!("file://{}/nope.git", results.root.path().display()),
    );

    // Act
    let result = clone.fetch();

    // Assert
    assert!(
        matches!(result, Err(SyncError::Unreachable(_))),
        "{result:?}"
    );
}

#[test]
fn publish_after_results_moved_is_rejected() {
    // Arrange
    let results = Origin::new();
    let (work_a, work_b) = (TempDir::new("work-a"), TempDir::new("work-b"));
    let (a, b) = (
        results_clone(&work_a, &results),
        results_clone(&work_b, &results),
    );
    let x = results.repo("x");
    a.fetch().unwrap();
    b.fetch().unwrap();
    a.publish(&x, &publication(&["a"], &[])).unwrap();

    // Act
    let error = b.publish(&x, &publication(&["b"], &[])).unwrap_err();

    // Assert
    assert!(error.0.contains("git push"), "{error}");
    assert_eq!(
        git(&results.bare(), &["show", "main:x/missed.txt"]),
        "src/lib.rs: a"
    );
}

#[test]
fn publish_first_writes_the_readme_and_the_folder_on_main() {
    // Arrange
    let results = Origin::new();
    let work = TempDir::new("work");
    let clone = results_clone(&work, &results);
    clone.fetch().unwrap();

    // Act
    clone
        .publish(&results.repo("x"), &publication(&["a"], &[]))
        .unwrap();

    // Assert
    assert_eq!(
        files(&results),
        [
            "README.md",
            "x/located/missed.txt",
            "x/missed.txt",
            "x/summary.toml"
        ]
    );
    assert_eq!(
        git(&results.bare(), &["log", "-1", "--format=%s", "main"]),
        "x: Mutants of abababa: 1 missed (+1 -0), 4 caught"
    );
}

#[test]
fn publish_of_a_second_repository_keeps_the_first_folder() {
    // Arrange
    let results = Origin::new();
    let work = TempDir::new("work");
    let clone = results_clone(&work, &results);
    clone.fetch().unwrap();
    clone
        .publish(&results.repo("x"), &publication(&["a"], &[]))
        .unwrap();

    // Act
    clone
        .publish(&results.repo("y"), &publication(&["b"], &[]))
        .unwrap();

    // Assert
    assert_eq!(
        files(&results),
        [
            "README.md",
            "x/located/missed.txt",
            "x/missed.txt",
            "x/summary.toml",
            "y/located/missed.txt",
            "y/missed.txt",
            "y/summary.toml"
        ]
    );
    assert_eq!(git(&results.bare(), &["rev-list", "--count", "main"]), "2");
}

#[test]
fn publish_onto_a_hand_made_main_keeps_its_other_files() {
    // Arrange
    let results = Origin::new();
    results.commit(&[("README.md", "by hand"), ("NOTES.md", "kept")]);
    let work = TempDir::new("work");
    let clone = results_clone(&work, &results);
    clone.fetch().unwrap();

    // Act
    clone
        .publish(&results.repo("x"), &publication(&["a"], &[]))
        .unwrap();

    // Assert
    assert!(files(&results).contains(&"NOTES.md".to_string()));
    assert_eq!(git(&results.bare(), &["rev-list", "--count", "main"]), "2");
}

#[test]
fn publish_replaces_the_folder_dropping_what_is_no_longer_written() {
    // Arrange
    let results = Origin::new();
    let work = TempDir::new("work");
    let clone = results_clone(&work, &results);
    let x = results.repo("x");
    clone.fetch().unwrap();
    clone.publish(&x, &publication(&["a"], &["t"])).unwrap();

    // Act
    clone.publish(&x, &publication(&["b"], &[])).unwrap();

    // Assert
    assert_eq!(
        files(&results),
        [
            "README.md",
            "x/located/missed.txt",
            "x/missed.txt",
            "x/summary.toml"
        ]
    );
    let diff = git(
        &results.bare(),
        &["diff", "main~1", "main", "--", "x/missed.txt"],
    );
    assert!(
        diff.contains("-src/lib.rs: a") && diff.contains("+src/lib.rs: b"),
        "{diff}"
    );
}

#[test]
fn record_after_a_publish_reads_its_own_folder_only() {
    // Arrange
    let results = Origin::new();
    let work = TempDir::new("work");
    let clone = results_clone(&work, &results);
    clone.fetch().unwrap();
    clone
        .publish(&results.repo("x"), &publication(&["a"], &[]))
        .unwrap();
    clone.fetch().unwrap();

    // Act
    let (x, y) = (
        clone.record(&results.repo("x")),
        clone.record(&results.repo("y")),
    );

    // Assert
    let RecordState::Present(record) = x else {
        panic!("expected a record, found {x:?}");
    };
    assert_eq!(record.commit, commit());
    assert_eq!(
        clone.previous_missed(&results.repo("x")),
        [MutantKey::new("src/lib.rs: a")]
    );
    assert_eq!(y, RecordState::Absent);
    assert_eq!(
        clone.previous_missed(&results.repo("y")),
        Vec::<MutantKey>::new()
    );
}

#[test]
fn record_of_a_garbage_summary_is_unreadable() {
    // Arrange
    let results = Origin::new();
    results.commit(&[
        ("x/summary.toml", "schema = [\n"),
        ("x/missed.txt", "src/lib.rs: a\n"),
    ]);
    let work = TempDir::new("work");
    let clone = results_clone(&work, &results);
    clone.fetch().unwrap();

    // Act
    let record = clone.record(&results.repo("x"));

    // Assert
    assert!(matches!(
        record,
        RecordState::Unreadable(RecordError::Invalid(_))
    ));
    assert_eq!(
        clone.previous_missed(&results.repo("x")),
        [MutantKey::new("src/lib.rs: a")]
    );
}

#[test]
fn record_once_main_is_deleted_is_absent() {
    // Arrange
    let results = Origin::new();
    let work = TempDir::new("work");
    let clone = results_clone(&work, &results);
    clone.fetch().unwrap();
    clone
        .publish(&results.repo("x"), &publication(&["a"], &[]))
        .unwrap();
    git(&results.bare(), &["update-ref", "-d", "refs/heads/main"]);

    // Act
    clone.fetch().unwrap();

    // Assert
    assert_eq!(clone.record(&results.repo("x")), RecordState::Absent);
    assert!(fs::metadata(work.path().join("results.git")).is_ok());
}

#[test]
fn record_without_any_result_is_absent() {
    // Arrange
    let results = Origin::new();
    let work = TempDir::new("work");
    let clone = results_clone(&work, &results);

    // Act
    clone.fetch().unwrap();

    // Assert
    assert_eq!(clone.record(&results.repo("x")), RecordState::Absent);
}
