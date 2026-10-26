// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::failed;
use crate::support::scripted_runner::ok;
use crate::support::temp_dir::TempDir;
use sentinel4rust::model::onboarding::onboarded_repo::OnboardedRepo;
use sentinel4rust::model::sync_error::SyncError;
use sentinel4rust::model::work_dir::WorkDir;
use sentinel4rust::system::git::bare_clone::BareClone;
use sentinel4rust::system::git::git_runner::GitRunner;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

fn bare_clone(runner: &Rc<ScriptedRunner>, work: &TempDir) -> BareClone {
    BareClone::new(GitRunner::new(runner.clone()), work_dir(work))
}

fn existing(work: &TempDir) {
    let dir = work_dir(work).bare_clone(&repo());
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
}

fn repo() -> OnboardedRepo {
    OnboardedRepo {
        name: String::from("x"),
        url: String::from("file:///srv/x.git"),
        branch: String::from("main"),
        jobs: 1,
        run_timeout: Duration::from_secs(3600),
    }
}

fn work_dir(work: &TempDir) -> WorkDir {
    WorkDir::new(work.path().to_path_buf())
}

#[test]
fn ensure_failing_is_transient() {
    // Arrange
    let work = TempDir::new("bare-clone-fail");
    let runner = Rc::new(ScriptedRunner::new(vec![failed(1, "no space")]));

    // Act
    let result = bare_clone(&runner, &work).ensure(&repo());

    // Assert
    let dir = work_dir(&work).bare_clone(&repo());
    assert_eq!(
        result,
        Err(SyncError::Transient(format!(
            "git init --quiet --bare {}: exited with 1: no space",
            dir.display()
        )))
    );
}

#[test]
fn ensure_of_a_clone_with_another_url_resets_origin() {
    // Arrange
    let work = TempDir::new("bare-clone-moved");
    existing(&work);
    let runner = Rc::new(ScriptedRunner::new(vec![ok("file:///old.git\n"), ok("")]));

    // Act
    let result = bare_clone(&runner, &work).ensure(&repo());

    // Assert
    assert_eq!(result, Ok(work_dir(&work).bare_clone(&repo())));
    assert_eq!(
        runner.args(),
        [
            vec!["remote", "get-url", "origin"],
            vec!["remote", "set-url", "origin", "file:///srv/x.git"],
        ]
    );
}

#[test]
fn ensure_of_a_clone_with_the_same_url_only_reads_it() {
    // Arrange
    let work = TempDir::new("bare-clone-same");
    existing(&work);
    let runner = Rc::new(ScriptedRunner::new(vec![ok("file:///srv/x.git\n")]));

    // Act
    let result = bare_clone(&runner, &work).ensure(&repo());

    // Assert
    assert_eq!(result, Ok(work_dir(&work).bare_clone(&repo())));
    assert_eq!(runner.args(), [["remote", "get-url", "origin"]]);
}

#[test]
fn ensure_of_no_clone_inits_a_bare_repository_with_origin() {
    // Arrange
    let work = TempDir::new("bare-clone-new");
    let runner = Rc::new(ScriptedRunner::new(vec![ok(""), ok("")]));

    // Act
    let result = bare_clone(&runner, &work).ensure(&repo());

    // Assert
    let dir = work_dir(&work).bare_clone(&repo());
    assert_eq!(result, Ok(dir.clone()));
    let dir_text = dir.to_string_lossy().into_owned();
    assert_eq!(
        runner.args(),
        [
            vec!["init", "--quiet", "--bare", dir_text.as_str()],
            vec!["remote", "add", "origin", "file:///srv/x.git"],
        ]
    );
    assert_eq!(
        Path::new(runner.commands()[0].dir()),
        work.path().join("repos")
    );
    assert_eq!(Path::new(runner.commands()[1].dir()), dir);
}

#[test]
fn prune_worktrees_of_a_clone_prunes_them() {
    // Arrange
    let work = TempDir::new("bare-clone-prune");
    existing(&work);
    let runner = Rc::new(ScriptedRunner::new(vec![ok("")]));

    // Act
    bare_clone(&runner, &work).prune_worktrees(&repo());

    // Assert
    assert_eq!(runner.args(), [["worktree", "prune"]]);
}

#[test]
fn prune_worktrees_without_a_clone_runs_nothing() {
    // Arrange
    let work = TempDir::new("bare-clone-none");
    let runner = Rc::new(ScriptedRunner::new(vec![]));

    // Act
    bare_clone(&runner, &work).prune_worktrees(&repo());

    // Assert
    assert_eq!(runner.commands().len(), 0);
}
