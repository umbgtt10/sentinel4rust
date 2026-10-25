// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::command_result::CommandResult;

fn exited(code: Option<i32>, stdout: &str, stderr: &str) -> CommandResult {
    CommandResult::Exited {
        code,
        stdout: String::from(stdout),
        stderr: String::from(stderr),
    }
}

#[test]
fn describe_a_failed_start_says_why() {
    // Arrange
    let result = CommandResult::FailedToStart(String::from("No such file or directory"));

    // Act
    let text = result.describe();

    // Assert
    assert_eq!(text, "could not start: No such file or directory");
}

#[test]
fn describe_a_kill_by_signal_says_a_signal() {
    // Arrange
    let result = exited(None, "", "");

    // Act
    let text = result.describe();

    // Assert
    assert_eq!(text, "exited with a signal: ");
}

#[test]
fn describe_a_timeout_says_timed_out() {
    // Arrange
    let result = CommandResult::TimedOut {
        stdout: String::from("partial"),
        stderr: String::new(),
    };

    // Act
    let text = result.describe();

    // Assert
    assert_eq!(text, "timed out");
}

#[test]
fn describe_an_exit_names_the_code_and_the_first_stderr_line() {
    // Arrange
    let result = exited(
        Some(128),
        "",
        "\n  fatal: repository not found  \nhint: x\n",
    );

    // Act
    let text = result.describe();

    // Assert
    assert_eq!(text, "exited with 128: fatal: repository not found");
}

#[test]
fn success_of_a_failure_returns_none() {
    // Arrange
    let result = exited(Some(1), "out", "err");

    // Act
    let stdout = result.success();

    // Assert
    assert_eq!(stdout, None);
}

#[test]
fn success_of_a_zero_exit_returns_its_stdout() {
    // Arrange
    let result = exited(Some(0), "abc\n", "");

    // Act
    let stdout = result.success();

    // Assert
    assert_eq!(stdout, Some("abc\n"));
}
