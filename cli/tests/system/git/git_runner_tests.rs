// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::failed;
use crate::support::scripted_runner::ok;
use sentinel4rust::model::command::Command;
use sentinel4rust_cli::system::git::git_failure::GitFailure;
use sentinel4rust_cli::system::git::git_runner::GitRunner;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

#[test]
fn command_runs_git_without_prompts_in_the_c_locale() {
    // Arrange
    let git = GitRunner::new(Rc::new(ScriptedRunner::new(vec![])));

    // Act
    let command = git.command(Path::new("/w"), &["fetch"], Duration::from_secs(5));

    // Assert
    assert_eq!(command.program(), "git");
    assert_eq!(command.args(), ["fetch"]);
    assert_eq!(Path::new(command.dir()), Path::new("/w"));
    assert_eq!(command.timeout(), Duration::from_secs(5));
    assert_eq!(
        command.env(),
        &BTreeMap::from([
            (String::from("GIT_TERMINAL_PROMPT"), String::from("0")),
            (String::from("LC_ALL"), String::from("C")),
        ])
    );
}

#[test]
fn git_exiting_non_zero_returns_the_result() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![failed(1, "boom")]));
    let git = GitRunner::new(runner.clone());

    // Act
    let result = git.git(Path::new("/w"), &["fetch"], Duration::from_secs(5));

    // Assert
    assert_eq!(result, Err(failed(1, "boom")));
}

#[test]
fn git_succeeding_returns_stdout() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![ok("abc\n")]));
    let git = GitRunner::new(runner.clone());

    // Act
    let result = git.git(
        Path::new("/w"),
        &["rev-parse", "HEAD"],
        Duration::from_secs(5),
    );

    // Assert
    assert_eq!(result, Ok(String::from("abc\n")));
    assert_eq!(runner.args(), [["rev-parse", "HEAD"]]);
}

#[test]
fn local_failing_names_the_command_line_within_a_minute() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![failed(128, "fatal: x")]));
    let git = GitRunner::new(runner.clone());

    // Act
    let result = git.local(Path::new("/w"), &["remote", "get-url", "origin"]);

    // Assert
    assert_eq!(
        result,
        Err(GitFailure {
            what: String::from("git remote get-url origin"),
            result: failed(128, "fatal: x"),
        })
    );
    assert_eq!(runner.commands()[0].timeout(), Duration::from_secs(60));
}

#[test]
fn run_passes_the_command_through_unchanged() {
    // Arrange
    let runner = Rc::new(ScriptedRunner::new(vec![ok("tree\n")]));
    let git = GitRunner::new(runner.clone());
    let command = Command::new("git", &["write-tree"], "/w", Duration::from_secs(1))
        .with_env("GIT_INDEX_FILE", "/s/index");

    // Act
    let result = git.run(&command);

    // Assert
    assert_eq!(result, Ok(String::from("tree\n")));
    assert_eq!(runner.commands(), [command]);
}
