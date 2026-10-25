// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::command::Command;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

fn command() -> Command {
    Command::new(
        "git",
        &["fetch", "--quiet"],
        Path::new("/w/repos/a.git"),
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
fn dir_of_a_new_command_is_the_one_given() {
    // Arrange & Act
    let command = command();

    // Assert
    assert_eq!(command.dir(), Path::new("/w/repos/a.git"));
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
