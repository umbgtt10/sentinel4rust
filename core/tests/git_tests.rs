// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// `GitCli` against real git: bare repositories in temporary directories,
// `file://` remotes, no network.

use crate::support::git_shell::git;
use crate::support::origin::Origin;
use crate::support::temp_dir::TempDir;
use sentinel4rust::git::GitCli;
use sentinel4rust::model::branch::branch_constants::KIND_FULL;
use sentinel4rust::model::branch::publication::Publication;
use sentinel4rust::model::branch::record_currency::RecordCurrency;
use sentinel4rust::model::branch::run_record::RunRecord;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::mutant_key::MutantKey;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::model::skip_reason::SkipReason;
use sentinel4rust::model::sync_error::SyncError;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::model::work_dir::WorkDir;
use sentinel4rust::system::process::system_command_runner::SystemCommandRunner;
use sentinel4rust::traits::repo_source::RepoSource;
use sentinel4rust::traits::result_store::ResultStore;
use std::fs;
use std::process::Command;
use std::rc::Rc;

fn cli(work: &TempDir) -> GitCli {
    GitCli::new(Rc::new(SystemCommandRunner), work_dir(work))
}

fn publication(commit: &CommitHash, missed: &[&str]) -> Publication {
    let located: Vec<String> = missed
        .iter()
        .enumerate()
        .map(|(i, m)| format!("src/lib.rs:{}:1: {m}", i + 1))
        .collect();
    let keys = MutantKey::from_located(&located);
    Publication {
        record: RunRecord {
            commit: commit.clone(),
            config_version: 1,
            kind: KIND_FULL.to_string(),
            outcome: OutcomeKind::Completed,
            started: Timestamp::from_unix(1_792_800_000),
            finished: Timestamp::from_unix(1_792_800_060),
            counts: MutantCounts {
                caught: 4,
                missed: keys.len() as u64,
                timeout: 0,
                unviable: 0,
            },
            new: keys.len() as u64,
            gone: 0,
            cargo_mutants_version: "cargo-mutants 27.1.0".to_string(),
            rustc_version: "rustc 1.97.0".to_string(),
            arguments: "mutants".to_string(),
            reason: None,
            log_tail: None,
        },
        missed: keys,
        timed_out: vec![],
        located_missed: located,
        located_timed_out: vec![],
    }
}

fn work_dir(work: &TempDir) -> WorkDir {
    WorkDir::new(work.path().to_path_buf())
}

#[test]
fn checkout_is_detached_at_head_and_removed_completely() {
    // Arrange
    let origin = Origin::new();
    let head = origin.commit_crate();
    origin.commit(&[("later.txt", "after head")]);
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();

    // Act
    let checkout = git_cli.checkout(&repo, &head).unwrap();

    // Assert
    assert!(checkout.join("src/lib.rs").exists());
    assert!(!checkout.join("later.txt").exists());
    assert_eq!(git(&checkout, &["rev-parse", "HEAD"]), head.as_str());

    // Act
    fs::write(checkout.join("dirty.txt"), "x").unwrap();
    git_cli.remove_checkout(&repo);

    // Assert
    assert!(!checkout.exists());
    assert_eq!(
        git(&work_dir(&work).bare_clone(&repo), &["worktree", "list"])
            .lines()
            .count(),
        1
    );
}

#[test]
fn publish_after_mutants_moved_is_rejected() {
    // Arrange
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work_a = TempDir::new("work-a");
    let work_b = TempDir::new("work-b");
    let (a, b) = (cli(&work_a), cli(&work_b));
    let repo = origin.repo("x");
    a.sync(&repo).unwrap();
    b.sync(&repo).unwrap();
    a.publish(&repo, &publication(&head, &["a"])).unwrap();

    // Act
    let error = b.publish(&repo, &publication(&head, &["b"])).unwrap_err();

    // Assert
    assert!(error.0.contains("git push"), "{error}");
    assert_eq!(
        git(&origin.bare(), &["show", "mutants:missed.txt"]),
        "src/lib.rs: a"
    );
}

#[test]
fn publish_first_creates_an_orphan_branch_and_leaves_main_alone() {
    // Arrange
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();
    let main_before = origin.remote_ref("refs/heads/main");

    // Act
    git_cli
        .publish(
            &repo,
            &publication(&head, &["replace f -> bool with false"]),
        )
        .unwrap();

    // Assert
    assert_eq!(origin.remote_ref("refs/heads/main"), main_before);
    let bare = origin.bare();
    let files = git(&bare, &["ls-tree", "-r", "--name-only", "mutants"]);
    assert_eq!(
        files.lines().collect::<Vec<_>>(),
        [
            "README.md",
            "located/missed.txt",
            "missed.txt",
            "summary.toml"
        ]
    );
    assert_eq!(git(&bare, &["rev-list", "--count", "mutants"]), "1");
    assert_eq!(
        Command::new("git")
            .args(["merge-base", "main", "mutants"])
            .current_dir(&bare)
            .output()
            .unwrap()
            .stdout,
        Vec::<u8>::new()
    );
    assert_eq!(
        git(&bare, &["log", "-1", "--format=%s", "mutants"]),
        format!("Mutants of {}: 1 missed (+1 -0), 4 caught", head.short())
    );
    assert_eq!(
        git(&bare, &["show", "mutants:missed.txt"]),
        "src/lib.rs: replace f -> bool with false"
    );

    // Act
    let snapshot = git_cli.sync(&repo).unwrap();

    // Assert
    let record = snapshot.record.unwrap();
    assert!(RecordCurrency::new(1).is_current(&record, &head));
    assert_eq!(
        snapshot.previous_missed,
        [MutantKey::new("src/lib.rs: replace f -> bool with false")]
    );
}

#[test]
fn publish_second_appends_to_mutants() {
    // Arrange
    let origin = Origin::new();
    let first = origin.commit_crate();
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();
    git_cli
        .publish(&repo, &publication(&first, &["a"]))
        .unwrap();
    let second = origin.commit(&[("src/lib.rs", "pub fn f() -> bool { false }\n")]);
    let main_before = origin.remote_ref("refs/heads/main");

    // Act
    let snapshot = git_cli.sync(&repo).unwrap();

    // Assert
    assert_eq!(snapshot.head, second);
    assert!(!RecordCurrency::new(1).is_current(&snapshot.record.unwrap(), &second));

    // Act
    git_cli
        .publish(&repo, &publication(&second, &["b"]))
        .unwrap();

    // Assert
    assert_eq!(origin.remote_ref("refs/heads/main"), main_before);
    assert_eq!(
        git(&origin.bare(), &["rev-list", "--count", "mutants"]),
        "2"
    );
    let diff = git(
        &origin.bare(),
        &["diff", "mutants~1", "mutants", "--", "missed.txt"],
    );
    assert!(
        diff.contains("-src/lib.rs: a") && diff.contains("+src/lib.rs: b"),
        "{diff}"
    );
}

#[test]
fn sync_changed_url_resets_origin_and_deleted_mutants_drops_the_record() {
    // Arrange
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    let mut repo = origin.repo("x");
    repo.url = format!("file://{}/nope.git", origin.root.path().display());

    // Act & Assert
    assert!(git_cli.sync(&repo).is_err());

    // Act
    repo.url = origin.url();
    git_cli.sync(&repo).unwrap();
    git_cli.publish(&repo, &publication(&head, &["a"])).unwrap();

    // Assert
    assert_eq!(
        git(
            &work_dir(&work).bare_clone(&repo),
            &["remote", "get-url", "origin"]
        ),
        origin.url()
    );

    // Act
    git(&origin.bare(), &["update-ref", "-d", "refs/heads/mutants"]);

    // Assert
    assert_eq!(git_cli.sync(&repo).unwrap().record, None);
}

#[test]
fn sync_missing_branch_and_missing_repository_are_told_apart() {
    // Arrange
    let origin = Origin::new();
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    let mut gone = origin.repo("gone");
    gone.url = format!("file://{}/nope.git", origin.root.path().display());

    // Act & Assert
    assert_eq!(git_cli.sync(&origin.repo("x")), Err(SyncError::NoBranch));
    assert!(matches!(
        git_cli.sync(&gone),
        Err(SyncError::Unreachable(_))
    ));
}

#[test]
fn sync_without_a_mutants_branch_reports_no_record() {
    // Arrange
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work = TempDir::new("work");

    // Act
    let snapshot = cli(&work).sync(&origin.repo("x")).unwrap();

    // Assert
    assert_eq!(snapshot.head, head);
    assert_eq!(snapshot.record, None);
    assert_eq!(snapshot.previous_missed, Vec::<MutantKey>::new());
    assert_eq!(snapshot.readiness, Ok(()));
}

#[test]
fn sync_without_cargo_toml_or_mutants_config_is_not_ready() {
    // Arrange
    let origin = Origin::new();
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    origin.commit(&[("README.md", "hi")]);

    // Act & Assert
    assert_eq!(
        git_cli.sync(&origin.repo("x")).unwrap().readiness,
        Err(SkipReason::NotACrate)
    );

    // Act
    origin.commit(&[("Cargo.toml", "[package]\nname = \"x\"\n")]);

    // Assert
    assert_eq!(
        git_cli.sync(&origin.repo("x")).unwrap().readiness,
        Err(SkipReason::NoFilters)
    );
}
