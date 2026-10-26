// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::command::Command;
use std::collections::BTreeMap;
use std::time::Duration;

fn command() -> Command {
    Command::new(
        "git",
        &["fetch", "--quiet"],
        "/w/repos/a.git",
        Duration::from_secs(60),
    )
}

#[test]
fn args_of_a_new_command_are_the_ones_given_in_order() {
    // Arrange & Act
    let command = command();

    // Assert
    assert_eq!(command.args(), ["fetch", "--quiet"]);
}

#[test]
fn capture_limit_of_a_new_command_is_none() {
    // Arrange & Act
    let command = command();

    // Assert
    assert_eq!(command.capture_limit(), None);
}

#[test]
fn dir_of_a_new_command_is_the_one_given() {
    // Arrange & Act
    let command = command();

    // Assert
    assert_eq!(command.dir(), "/w/repos/a.git");
}

#[test]
fn display_joins_the_program_and_its_arguments() {
    // Arrange
    let command = command();

    // Act
    let text = command.display();

    // Assert
    assert_eq!(text, "git fetch --quiet");
}

#[test]
fn env_of_a_new_command_is_empty() {
    // Arrange & Act
    let command = command();

    // Assert
    assert!(command.env().is_empty());
}

#[test]
fn program_of_a_new_command_is_the_one_given() {
    // Arrange & Act
    let command = command();

    // Assert
    assert_eq!(command.program(), "git");
}

#[test]
fn timeout_of_a_new_command_is_the_one_given() {
    // Arrange & Act
    let command = command();

    // Assert
    assert_eq!(command.timeout(), Duration::from_secs(60));
}

#[test]
fn with_capture_limit_keeps_that_many_last_bytes_of_output() {
    // Arrange
    let command = command();

    // Act
    let limited = command.with_capture_limit(64 * 1024);

    // Assert
    assert_eq!(limited.capture_limit(), Some(64 * 1024));
    assert_eq!(limited.display(), "git fetch --quiet");
}

#[test]
fn with_env_adds_each_variable_and_keeps_the_rest() {
    // Arrange
    let command = command();

    // Act
    let command = command
        .with_env("LC_ALL", "C")
        .with_env("GIT_TERMINAL_PROMPT", "0");

    // Assert
    assert_eq!(
        command.env(),
        &BTreeMap::from([
            (String::from("GIT_TERMINAL_PROMPT"), String::from("0")),
            (String::from("LC_ALL"), String::from("C")),
        ])
    );
    assert_eq!(command.display(), "git fetch --quiet");
}
