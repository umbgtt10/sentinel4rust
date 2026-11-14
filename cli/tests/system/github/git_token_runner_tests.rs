// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::fixed_token::FixedToken;
use crate::support::no_token::NoToken;
use crate::support::scripted_runner::ScriptedRunner;
use crate::support::scripted_runner::ok;
use sentinel4rust::model::command::Command;
use sentinel4rust::traits::command_runner::CommandRunner;
use sentinel4rust_cli::system::github::git_token_runner::GitTokenRunner;
use std::rc::Rc;
use std::time::Duration;

fn git(args: &[&str]) -> Command {
    Command::new("git", args, "/w/repos/x.git", Duration::from_secs(60))
}

#[test]
fn run_of_a_git_network_command_carries_the_token_in_its_environment() {
    // Arrange
    let inner = Rc::new(ScriptedRunner::new(vec![ok(""), ok(""), ok(""), ok("")]));
    let runner = GitTokenRunner::new(inner.clone(), Rc::new(FixedToken(String::from("ghs_t"))));
    let commands = [
        git(&["fetch", "--quiet", "origin"]),
        git(&["ls-remote", "--heads", "origin"]),
        git(&["clone", "--bare", "https://github.com/o/x.git"]),
        git(&["push", "--quiet", "origin", "c:refs/heads/main"]),
    ];

    // Act
    for command in &commands {
        runner.run(command);
    }

    // Assert
    for (sent, original) in inner.commands().iter().zip(&commands) {
        assert_eq!(sent.args(), original.args());
        assert_eq!(sent.env()["GIT_CONFIG_COUNT"], "1");
        assert_eq!(
            sent.env()["GIT_CONFIG_KEY_0"],
            "http.https://github.com/.extraheader"
        );
        assert_eq!(
            sent.env()["GIT_CONFIG_VALUE_0"],
            "AUTHORIZATION: basic eC1hY2Nlc3MtdG9rZW46Z2hzX3Q="
        );
    }
}

#[test]
fn run_of_a_local_git_command_or_another_program_is_left_alone() {
    // Arrange
    let inner = Rc::new(ScriptedRunner::new(vec![ok(""), ok("")]));
    let runner = GitTokenRunner::new(inner.clone(), Rc::new(FixedToken(String::from("ghs_t"))));
    let other = Command::new("cargo", &["mutants"], "/w", Duration::from_secs(60));

    // Act
    runner.run(&git(&["rev-parse", "HEAD"]));
    runner.run(&other);

    // Assert
    assert!(
        inner
            .commands()
            .iter()
            .all(|command| command.env().is_empty())
    );
}

#[test]
fn run_without_a_token_runs_the_command_as_it_is() {
    // Arrange
    let inner = Rc::new(ScriptedRunner::new(vec![ok("listed")]));
    let runner = GitTokenRunner::new(inner.clone(), Rc::new(NoToken));

    // Act
    let result = runner.run(&git(&["ls-remote", "origin"]));

    // Assert
    assert_eq!(result.success(), Some("listed"));
    assert!(inner.commands()[0].env().is_empty());
}
