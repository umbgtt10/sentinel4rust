// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// `GitCli` against real git: bare repositories in temporary directories,
// `file://` remotes, no network.

use crate::support::file_protocol_runner::FileProtocolRunner;
use crate::support::git_shell::git;
use crate::support::origin::Origin;
use crate::support::temp_dir::TempDir;
use sentinel4rust::model::branch::branch_constants::KIND_FULL;
use sentinel4rust::model::branch::publication::Publication;
use sentinel4rust::model::branch::record_currency::RecordCurrency;
use sentinel4rust::model::branch::record_state::RecordState;
use sentinel4rust::model::branch::run_record::RunRecord;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::model::mutant_key::MutantKey;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::model::skip_reason::SkipReason;
use sentinel4rust::model::sync_error::SyncError;
use sentinel4rust::model::timestamp::Timestamp;
use sentinel4rust::traits::repo_source::RepoSource;
use sentinel4rust::traits::result_store::ResultStore;
use sentinel4rust_cli::system::git::git_cli::GitCli;
use sentinel4rust_cli::system::process::system_command_runner::SystemCommandRunner;
use sentinel4rust_cli::system::work_dir::WorkDir;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::slice;

fn cli(work: &TempDir, results: &Origin) -> GitCli {
    GitCli::new(Rc::new(SystemCommandRunner), work_dir(work), results.url())
}

fn present(state: RecordState) -> RunRecord {
    let RecordState::Present(record) = state else {
        panic!("expected a record, found {state:?}");
    };
    *record
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
            test_timeout_seconds: None,
        },
        missed: keys,
        timed_out: vec![],
        located_missed: located,
        located_timed_out: vec![],
    }
}

fn refs(origin: &Origin) -> String {
    git(
        &origin.bare(),
        &["for-each-ref", "--format=%(refname) %(objectname)"],
    )
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
    let results = Origin::new();
    let git_cli = cli(&work, &results);
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();

    // Act
    let checkout = PathBuf::from(git_cli.checkout(&repo, &head).unwrap());

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
fn checkout_of_a_commit_with_a_submodule_initialises_it() {
    // Arrange
    let sub = Origin::new();
    sub.commit(&[("lib.txt", "from the submodule")]);
    let origin = Origin::new();
    origin.commit_crate();
    let dev = origin.dev();
    git(
        &dev,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "--quiet",
            &sub.url(),
            "vendor/sub",
        ],
    );
    git(&dev, &["commit", "--quiet", "-m", "add a submodule"]);
    git(&dev, &["push", "--quiet", "origin", "main"]);
    let head = CommitHash::parse(&git(&dev, &["rev-parse", "HEAD"])).unwrap();
    let work = TempDir::new("work");
    let results = Origin::new();
    let git_cli = GitCli::new(Rc::new(FileProtocolRunner), work_dir(&work), results.url());
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();

    // Act
    let checkout = PathBuf::from(git_cli.checkout(&repo, &head).unwrap());

    // Assert
    assert_eq!(
        fs::read_to_string(checkout.join("vendor/sub/lib.txt")).unwrap(),
        "from the submodule"
    );
}

#[test]
fn committed_at_reads_the_commit_time_git_recorded() {
    // Arrange
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work = TempDir::new("work");
    let results = Origin::new();
    let git_cli = cli(&work, &results);
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();
    let seconds: i64 = git(&origin.bare(), &["log", "-1", "--format=%ct", "main"])
        .parse()
        .unwrap();

    // Act
    let committed = git_cli.committed_at(&repo, &head);

    // Assert
    assert_eq!(committed, Some(Timestamp::from_unix(seconds)));
}

#[test]
fn prune_removes_leftover_checkouts_staging_areas_and_worktree_entries() {
    // Arrange
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work = TempDir::new("work");
    let results = Origin::new();
    let git_cli = cli(&work, &results);
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();
    git_cli.checkout(&repo, &head).unwrap();
    fs::create_dir_all(work_dir(&work).stage(&repo)).unwrap();

    // Act
    git_cli.prune(slice::from_ref(&repo));

    // Assert
    assert!(!work_dir(&work).runs().exists());
    assert!(!work_dir(&work).stages().exists());
    assert_eq!(
        git(&work_dir(&work).bare_clone(&repo), &["worktree", "list"])
            .lines()
            .count(),
        1
    );
}

#[test]
fn publish_writes_the_results_and_leaves_the_source_alone() {
    // Arrange
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work = TempDir::new("work");
    let results = Origin::new();
    let git_cli = cli(&work, &results);
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();
    let source_before = refs(&origin);

    // Act
    git_cli
        .publish(
            &repo,
            &publication(&head, &["replace f -> bool with false"]),
        )
        .unwrap();

    // Assert
    assert_eq!(refs(&origin), source_before);
    assert_eq!(
        git(&results.bare(), &["show", "main:x/missed.txt"]),
        "src/lib.rs: replace f -> bool with false"
    );

    // Act
    let snapshot = git_cli.sync(&repo).unwrap();

    // Assert
    let record = present(snapshot.record);
    assert!(RecordCurrency::new(1, None).is_current(&record, &head));
    assert_eq!(
        snapshot.previous_missed,
        [MutantKey::new("src/lib.rs: replace f -> bool with false")]
    );
}

#[test]
fn sync_after_main_moved_finds_the_record_behind() {
    // Arrange
    let origin = Origin::new();
    let first = origin.commit_crate();
    let work = TempDir::new("work");
    let results = Origin::new();
    let git_cli = cli(&work, &results);
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();
    git_cli
        .publish(&repo, &publication(&first, &["a"]))
        .unwrap();
    let second = origin.commit(&[("src/lib.rs", "pub fn f() -> bool { false }\n")]);

    // Act
    let snapshot = git_cli.sync(&repo).unwrap();

    // Assert
    assert_eq!(snapshot.head, second);
    assert!(!RecordCurrency::new(1, None).is_current(&present(snapshot.record), &second));
    assert_eq!(snapshot.previous_missed, [MutantKey::new("src/lib.rs: a")]);
}

#[test]
fn sync_changed_url_resets_origin() {
    // Arrange
    let origin = Origin::new();
    origin.commit_crate();
    let work = TempDir::new("work");
    let results = Origin::new();
    let git_cli = cli(&work, &results);
    let mut repo = origin.repo("x");
    repo.url = format!("file://{}/nope.git", origin.root.path().display());

    // Act & Assert
    assert!(git_cli.sync(&repo).is_err());

    // Act
    repo.url = origin.url();
    git_cli.sync(&repo).unwrap();

    // Assert
    assert_eq!(
        git(
            &work_dir(&work).bare_clone(&repo),
            &["remote", "get-url", "origin"]
        ),
        origin.url()
    );
}

#[test]
fn sync_missing_branch_and_missing_repository_are_told_apart() {
    // Arrange
    let origin = Origin::new();
    let work = TempDir::new("work");
    let results = Origin::new();
    let git_cli = cli(&work, &results);
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
fn sync_with_a_path_dependency_outside_the_repository_is_not_ready() {
    // Arrange
    let origin = Origin::new();
    origin.commit_crate();
    origin.commit(&[(
        "crates/a/Cargo.toml",
        "[package]\nname = \"a\"\n\n[dependencies]\nsibling = { path = \"../../../sibling\" }\n",
    )]);
    let work = TempDir::new("work");
    let results = Origin::new();
    let git_cli = cli(&work, &results);

    // Act
    let snapshot = git_cli.sync(&origin.repo("x")).unwrap();

    // Assert
    assert_eq!(
        snapshot.readiness,
        Err(SkipReason::PathOutsideRepo {
            manifest: String::from("crates/a/Cargo.toml")
        })
    );
}

#[test]
fn sync_with_an_unreachable_results_repository_is_unreachable() {
    // Arrange
    let origin = Origin::new();
    origin.commit_crate();
    let work = TempDir::new("work");
    let git_cli = GitCli::new(
        Rc::new(SystemCommandRunner),
        work_dir(&work),
        format!("file://{}/nope.git", origin.root.path().display()),
    );

    // Act
    let result = git_cli.sync(&origin.repo("x"));

    // Assert
    assert!(
        matches!(result, Err(SyncError::Unreachable(_))),
        "{result:?}"
    );
}

#[test]
fn sync_without_a_result_reports_no_record() {
    // Arrange
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work = TempDir::new("work");
    let results = Origin::new();

    // Act
    let snapshot = cli(&work, &results).sync(&origin.repo("x")).unwrap();

    // Assert
    assert_eq!(snapshot.head, head);
    assert_eq!(snapshot.record, RecordState::Absent);
    assert_eq!(snapshot.previous_missed, Vec::<MutantKey>::new());
    assert_eq!(snapshot.readiness, Ok(()));
}

#[test]
fn sync_without_cargo_toml_or_mutants_config_is_not_ready() {
    // Arrange
    let origin = Origin::new();
    let work = TempDir::new("work");
    let results = Origin::new();
    let git_cli = cli(&work, &results);
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
