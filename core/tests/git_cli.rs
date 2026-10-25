// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! `GitCli` against real git: bare repositories in temporary directories, `file://` remotes, no network.

mod support;

use std::fs;
use std::rc::Rc;

use sentinel4rust::git::GitCli;
use sentinel4rust::model::{
    KIND_FULL, MutantCounts, MutantKey, OutcomeKind, Publication, RunRecord, SkipReason, Timestamp,
};
use sentinel4rust::system::SystemCommandRunner;
use sentinel4rust::traits::{RepoSource, ResultStore, SyncError};
use support::{Origin, TempDir, git};

fn cli(work: &TempDir) -> GitCli {
    GitCli::new(Rc::new(SystemCommandRunner), work.path().to_path_buf())
}

fn publication(commit: &sentinel4rust::model::CommitHash, missed: &[&str]) -> Publication {
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

#[test]
fn fetching_without_a_mutants_branch_reports_no_record() {
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work = TempDir::new("work");
    let snapshot = cli(&work).sync(&origin.repo("x")).unwrap();
    assert_eq!(snapshot.head, head);
    assert_eq!(snapshot.record, None);
    assert_eq!(snapshot.previous_missed, Vec::<MutantKey>::new());
    assert_eq!(snapshot.readiness, Ok(()));
}

#[test]
fn readiness_needs_cargo_toml_and_mutants_config() {
    let origin = Origin::new();
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    origin.commit(&[("README.md", "hi")]);
    assert_eq!(
        git_cli.sync(&origin.repo("x")).unwrap().readiness,
        Err(SkipReason::NotACrate)
    );
    origin.commit(&[("Cargo.toml", "[package]\nname = \"x\"\n")]);
    assert_eq!(
        git_cli.sync(&origin.repo("x")).unwrap().readiness,
        Err(SkipReason::NoFilters)
    );
}

#[test]
fn a_missing_branch_and_a_missing_repository_are_told_apart() {
    let origin = Origin::new();
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    assert_eq!(git_cli.sync(&origin.repo("x")), Err(SyncError::NoBranch));
    let mut gone = origin.repo("gone");
    gone.url = format!("file://{}/nope.git", origin.root.path().display());
    assert!(matches!(
        git_cli.sync(&gone),
        Err(SyncError::Unreachable(_))
    ));
}

#[test]
fn publishing_creates_an_orphan_branch_and_leaves_main_alone() {
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();
    let main_before = origin.remote_ref("refs/heads/main");

    git_cli
        .publish(
            &repo,
            &publication(&head, &["replace f -> bool with false"]),
        )
        .unwrap();

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
    // Orphan: no parent, no common history with main.
    assert_eq!(git(&bare, &["rev-list", "--count", "mutants"]), "1");
    assert_eq!(
        std::process::Command::new("git")
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

    // The record and the previous list read back.
    let snapshot = git_cli.sync(&repo).unwrap();
    let record = snapshot.record.unwrap();
    assert!(record.is_current(&head, 1));
    assert_eq!(
        snapshot.previous_missed,
        [MutantKey::new("src/lib.rs: replace f -> bool with false")]
    );
}

#[test]
fn a_second_publish_appends_to_mutants() {
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
    let snapshot = git_cli.sync(&repo).unwrap();
    assert_eq!(snapshot.head, second);
    assert!(!snapshot.record.unwrap().is_current(&second, 1));
    git_cli
        .publish(&repo, &publication(&second, &["b"]))
        .unwrap();
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
fn a_push_rejected_because_mutants_moved_is_an_error() {
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work_a = TempDir::new("work-a");
    let work_b = TempDir::new("work-b");
    let (a, b) = (cli(&work_a), cli(&work_b));
    let repo = origin.repo("x");
    a.sync(&repo).unwrap();
    b.sync(&repo).unwrap();
    a.publish(&repo, &publication(&head, &["a"])).unwrap();
    let error = b.publish(&repo, &publication(&head, &["b"])).unwrap_err();
    assert!(error.contains("git push"), "{error}");
    assert_eq!(
        git(&origin.bare(), &["show", "mutants:missed.txt"]),
        "src/lib.rs: a"
    );
}

#[test]
fn checkouts_are_detached_at_head_and_removed_completely() {
    let origin = Origin::new();
    let head = origin.commit_crate();
    origin.commit(&[("later.txt", "after head")]);
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    let repo = origin.repo("x");
    git_cli.sync(&repo).unwrap();
    let checkout = git_cli.checkout(&repo, &head).unwrap();
    assert!(checkout.join("src/lib.rs").exists());
    assert!(!checkout.join("later.txt").exists());
    assert_eq!(git(&checkout, &["rev-parse", "HEAD"]), head.as_str());
    fs::write(checkout.join("dirty.txt"), "x").unwrap();
    git_cli.remove_checkout(&repo);
    assert!(!checkout.exists());
    assert_eq!(
        git(&git_cli.repo_dir(&repo), &["worktree", "list"])
            .lines()
            .count(),
        1
    );
}

#[test]
fn a_changed_url_resets_origin_and_a_deleted_mutants_branch_drops_the_record() {
    let origin = Origin::new();
    let head = origin.commit_crate();
    let work = TempDir::new("work");
    let git_cli = cli(&work);
    let mut repo = origin.repo("x");
    repo.url = format!("file://{}/nope.git", origin.root.path().display());
    assert!(git_cli.sync(&repo).is_err());
    repo.url = origin.url();
    git_cli.sync(&repo).unwrap();
    git_cli.publish(&repo, &publication(&head, &["a"])).unwrap();
    assert_eq!(
        git(&git_cli.repo_dir(&repo), &["remote", "get-url", "origin"]),
        origin.url()
    );
    git(&origin.bare(), &["update-ref", "-d", "refs/heads/mutants"]);
    assert_eq!(git_cli.sync(&repo).unwrap().record, None);
}
