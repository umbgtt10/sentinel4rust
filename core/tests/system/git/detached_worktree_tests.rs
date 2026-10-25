// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::failed;
use crate::support::scripted_runner::ok;
use crate::support::temp_dir::TempDir;
use sentinel4rust::model::checkout_error::CheckoutError;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::work_dir::WorkDir;
use sentinel4rust::system::git::detached_worktree::DetachedWorktree;
use sentinel4rust::system::git::git_runner::GitRunner;
use std::fs;
use std::rc::Rc;
use std::time::Duration;

fn head() -> CommitHash {
    CommitHash::parse(&"cd".repeat(20)).unwrap()
}

fn repo() -> OnboardedRepo {
    OnboardedRepo {
        name: String::from("x"),
        url: String::from("u"),
        branch: String::from("main"),
        jobs: 1,
        run_timeout: Duration::from_secs(3600),
    }
}

fn work_dir(work: &TempDir) -> WorkDir {
    WorkDir::new(work.path().to_path_buf())
}

fn worktree(runner: &Rc<ScriptedRunner>, work: &TempDir) -> DetachedWorktree {
    DetachedWorktree::new(GitRunner::new(runner.clone()), work_dir(work))
}

#[test]
fn add_creates_a_detached_worktree_at_head() {
    // Arrange
    let work = TempDir::new("worktree-add");
    let runner = Rc::new(ScriptedRunner::new(vec![ok("")]));

    // Act
    let result = worktree(&runner, &work).add(&repo(), &head());

    // Assert
    let checkout = work_dir(&work).checkout(&repo());
    assert_eq!(result, Ok(checkout.clone()));
    let checkout_text = checkout.to_string_lossy().into_owned();
    let head_text = head().to_string();
    assert_eq!(
        runner.args(),
        [[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            "--force",
            checkout_text.as_str(),
            head_text.as_str()
        ]]
    );
    let command = &runner.commands()[0];
    assert_eq!(command.dir(), work_dir(&work).bare_clone(&repo()));
    assert_eq!(command.timeout(), Duration::from_secs(300));
    assert!(work_dir(&work).runs().is_dir());
}

#[test]
fn add_failing_is_a_checkout_error_naming_the_step() {
    // Arrange
    let work = TempDir::new("worktree-fail");
    let runner = Rc::new(ScriptedRunner::new(vec![failed(
        128,
        "fatal: invalid reference",
    )]));

    // Act
    let result = worktree(&runner, &work).add(&repo(), &head());

    // Assert
    assert_eq!(
        result,
        Err(CheckoutError(String::from(
            "git worktree add: exited with 128: fatal: invalid reference"
        )))
    );
}

#[test]
fn add_of_a_commit_with_submodules_initialises_them_in_the_checkout() {
    // Arrange
    let work = TempDir::new("worktree-submodules");
    let checkout = work_dir(&work).checkout(&repo());
    fs::create_dir_all(&checkout).unwrap();
    fs::write(checkout.join(".gitmodules"), "[submodule \"sub\"]\n").unwrap();
    let runner = Rc::new(ScriptedRunner::new(vec![ok(""), ok("")]));

    // Act
    let result = worktree(&runner, &work).add(&repo(), &head());

    // Assert
    assert_eq!(result, Ok(checkout.clone()));
    let command = &runner.commands()[1];
    assert_eq!(
        command.args(),
        ["submodule", "update", "--init", "--recursive"]
    );
    assert_eq!(command.dir(), checkout);
    assert_eq!(command.timeout(), Duration::from_secs(300));
}

#[test]
fn add_with_a_submodule_failing_is_a_checkout_error_naming_the_step() {
    // Arrange
    let work = TempDir::new("worktree-submodule-fails");
    let checkout = work_dir(&work).checkout(&repo());
    fs::create_dir_all(&checkout).unwrap();
    fs::write(checkout.join(".gitmodules"), "[submodule \"sub\"]\n").unwrap();
    let runner = Rc::new(ScriptedRunner::new(vec![
        ok(""),
        failed(
            128,
            "fatal: repository 'git@github.com:x/private.git' not found",
        ),
    ]));

    // Act
    let result = worktree(&runner, &work).add(&repo(), &head());

    // Assert
    assert_eq!(
        result,
        Err(CheckoutError(String::from(
            "git submodule update: exited with 128: fatal: repository 'git@github.com:x/private.git' not found"
        )))
    );
}

#[test]
fn remove_of_a_checkout_removes_the_worktree_and_its_directory() {
    // Arrange
    let work = TempDir::new("worktree-remove");
    let bare = work_dir(&work).bare_clone(&repo());
    fs::create_dir_all(&bare).unwrap();
    fs::write(bare.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    let checkout = work_dir(&work).checkout(&repo());
    fs::create_dir_all(checkout.join("src")).unwrap();
    let runner = Rc::new(ScriptedRunner::new(vec![ok("")]));

    // Act
    worktree(&runner, &work).remove(&repo());

    // Assert
    let checkout_text = checkout.to_string_lossy().into_owned();
    assert_eq!(
        runner.args(),
        [[
            "worktree",
            "remove",
            "--force",
            "--force",
            checkout_text.as_str()
        ]]
    );
    assert!(!checkout.exists());
}

#[test]
fn remove_without_a_checkout_runs_nothing() {
    // Arrange
    let work = TempDir::new("worktree-none");
    let runner = Rc::new(ScriptedRunner::new(vec![]));

    // Act
    worktree(&runner, &work).remove(&repo());

    // Assert
    assert_eq!(runner.commands().len(), 0);
}
