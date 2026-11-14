// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::run_records::record;
use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::failed;
use crate::support::scripted_runner::ok;
use crate::support::temp_dir::TempDir;
use sentinel4rust::model::branch::branch_layout::BranchLayout;
use sentinel4rust::model::branch::publication::Publication;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::model::outcome_kind::OutcomeKind;
use sentinel4rust::model::publish_error::PublishError;
use sentinel4rust_cli::system::git::git_runner::GitRunner;
use sentinel4rust_cli::system::git::plumbing_commit::PlumbingCommit;
use std::path::Path;
use std::rc::Rc;

fn onto_parent(listed: &str) -> Vec<CommandResult> {
    vec![ok("parent\n"), ok(""), ok(listed)]
}

// A failed run writes two files, README.md and x/summary.toml.
fn publication() -> Publication {
    Publication {
        record: record(OutcomeKind::ToolFailed),
        missed: vec![],
        timed_out: vec![],
        located_missed: vec![],
        located_timed_out: vec![],
    }
}

fn staged_replies() -> Vec<CommandResult> {
    vec![ok("blob-a\n"), ok(""), ok("blob-b\n"), ok(""), ok("tree\n")]
}

#[test]
fn write_failing_names_the_step() {
    // Arrange
    let stage = TempDir::new("plumbing-fail");
    let mut replies = vec![failed(1, "")];
    replies.extend(staged_replies());
    replies.pop();
    replies.push(failed(128, "fatal: bad index"));
    let runner = Rc::new(ScriptedRunner::new(replies));
    let commit = PlumbingCommit::new(GitRunner::new(runner));

    // Act
    let result = commit.write(
        Path::new("/w/results.git"),
        stage.path(),
        "x",
        &publication(),
    );

    // Assert
    assert_eq!(
        result,
        Err(PublishError(String::from(
            "git write-tree: exited with 128: fatal: bad index"
        )))
    );
}

#[test]
fn write_first_commits_a_root_as_the_tool() {
    // Arrange
    let stage = TempDir::new("plumbing-first");
    let mut replies = vec![failed(1, "")];
    replies.extend(staged_replies());
    replies.push(ok("commit\n"));
    let runner = Rc::new(ScriptedRunner::new(replies));
    let commit = PlumbingCommit::new(GitRunner::new(runner.clone()));

    // Act
    let result = commit.write(
        Path::new("/w/results.git"),
        stage.path(),
        "x",
        &publication(),
    );

    // Assert
    assert_eq!(result, Ok(String::from("commit")));
    let index = stage.path().join("index").to_string_lossy().into_owned();
    let commands = runner.commands();
    assert_eq!(
        commands[0].args(),
        [
            "rev-parse",
            "--verify",
            "--quiet",
            "refs/remotes/origin/main^{commit}"
        ]
    );
    let blob = stage.path().join("blob-0").to_string_lossy().into_owned();
    assert_eq!(commands[1].args(), ["hash-object", "-w", blob.as_str()]);
    assert_eq!(
        commands[2].args(),
        [
            "update-index",
            "--add",
            "--cacheinfo",
            "100644,blob-a,README.md"
        ]
    );
    assert_eq!(commands[2].env()["GIT_INDEX_FILE"], index);
    assert_eq!(
        commands[4].args(),
        [
            "update-index",
            "--add",
            "--cacheinfo",
            "100644,blob-b,x/summary.toml"
        ]
    );
    assert_eq!(commands[5].env()["GIT_INDEX_FILE"], index);
    let message = BranchLayout.message("x", &publication());
    assert_eq!(
        commands[6].args(),
        ["commit-tree", "tree", "-m", message.as_str()]
    );
    assert_eq!(commands[6].env()["GIT_AUTHOR_NAME"], "sentinel4rust");
    assert_eq!(
        commands[6].env()["GIT_AUTHOR_EMAIL"],
        "sentinel4rust@localhost"
    );
    assert_eq!(commands[6].env()["GIT_COMMITTER_NAME"], "sentinel4rust");
    assert_eq!(
        commands[6].env()["GIT_COMMITTER_EMAIL"],
        "sentinel4rust@localhost"
    );
}

#[test]
fn write_onto_results_drops_the_folder_and_names_the_parent() {
    // Arrange
    let stage = TempDir::new("plumbing-parent");
    let mut replies = onto_parent("x/summary.toml\nx/timeout.txt\n");
    replies.push(ok(""));
    replies.extend(staged_replies());
    replies.push(ok("commit\n"));
    let runner = Rc::new(ScriptedRunner::new(replies));
    let commit = PlumbingCommit::new(GitRunner::new(runner.clone()));

    // Act
    let result = commit.write(
        Path::new("/w/results.git"),
        stage.path(),
        "x",
        &publication(),
    );

    // Assert
    assert_eq!(result, Ok(String::from("commit")));
    let index = stage.path().join("index").to_string_lossy().into_owned();
    let stage_text = stage.path().to_string_lossy().into_owned();
    let commands = runner.commands();
    assert_eq!(commands[1].args(), ["read-tree", "parent"]);
    assert_eq!(commands[1].env()["GIT_INDEX_FILE"], index);
    assert_eq!(
        commands[2].args(),
        ["ls-tree", "-r", "--name-only", "parent", "--", "x"]
    );
    assert_eq!(
        commands[3].args(),
        [
            "--work-tree",
            stage_text.as_str(),
            "update-index",
            "--force-remove",
            "--",
            "x/summary.toml",
            "x/timeout.txt"
        ]
    );
    assert_eq!(commands[3].env()["GIT_INDEX_FILE"], index);
    let message = BranchLayout.message("x", &publication());
    assert_eq!(
        commands[9].args(),
        [
            "commit-tree",
            "tree",
            "-p",
            "parent",
            "-m",
            message.as_str()
        ]
    );
}

#[test]
fn write_onto_results_without_the_folder_removes_nothing() {
    // Arrange
    let stage = TempDir::new("plumbing-new-folder");
    let mut replies = onto_parent("");
    replies.extend(staged_replies());
    replies.push(ok("commit\n"));
    let runner = Rc::new(ScriptedRunner::new(replies));
    let commit = PlumbingCommit::new(GitRunner::new(runner.clone()));

    // Act
    let result = commit.write(
        Path::new("/w/results.git"),
        stage.path(),
        "x",
        &publication(),
    );

    // Assert
    assert_eq!(result, Ok(String::from("commit")));
    let commands = runner.commands();
    assert_eq!(commands[3].args()[0], "hash-object");
    assert!(
        commands
            .iter()
            .all(|command| !command.args().iter().any(|arg| arg == "--force-remove"))
    );
}
