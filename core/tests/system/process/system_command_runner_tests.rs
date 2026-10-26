// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::command::Command;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::system::process::system_command_runner::SystemCommandRunner;
use sentinel4rust::traits::command_runner::CommandRunner;
use std::env::temp_dir;
use std::time::Duration;

fn sh(script: &str, timeout: Duration) -> CommandResult {
    let command = Command::new(
        "sh",
        &["-c", script],
        &temp_dir().to_string_lossy(),
        timeout,
    )
    .with_env("SENTINEL_TEST", "yes");
    SystemCommandRunner.run(&command)
}

#[test]
fn run_captures_exit_code_output_and_environment() {
    // Arrange
    let script = "echo out $SENTINEL_TEST; echo err >&2; exit 3";

    // Act
    let result = sh(script, Duration::from_secs(10));

    // Assert
    assert_eq!(
        result,
        CommandResult::Exited {
            code: Some(3),
            stdout: "out yes\n".to_string(),
            stderr: "err\n".to_string(),
        }
    );
    assert_eq!(result.success(), None);
    assert_eq!(result.describe(), "exited with 3: err");
}

#[test]
fn run_exiting_zero_succeeds_with_stdout() {
    // Arrange & Act
    let result = sh("printf hi", Duration::from_secs(10));

    // Assert
    assert_eq!(result.success(), Some("hi"));
}

#[test]
fn run_missing_program_fails_to_start() {
    // Arrange
    let command = Command::new(
        "/nonexistent/program",
        &[],
        &temp_dir().to_string_lossy(),
        Duration::from_secs(1),
    );

    // Act
    let result = SystemCommandRunner.run(&command);

    // Assert
    assert!(matches!(result, CommandResult::FailedToStart(_)));
}

// The background child would print `survived` a second in. Captured stdout
// ends only when every process holding it is gone, so a child that outlived
// the kill, or a kill that came late, shows in the output -- no clock needed.
#[test]
fn run_past_its_deadline_kills_the_whole_group() {
    // Arrange
    let script = "(sleep 1; echo survived) & echo started; sleep 30";

    // Act
    let result = sh(script, Duration::from_millis(300));

    // Assert
    assert!(matches!(&result, CommandResult::TimedOut { stdout, .. } if stdout == "started\n"));
}

#[test]
fn run_with_a_capture_limit_keeps_the_tail_of_the_output() {
    // Arrange
    let command = Command::new(
        "sh",
        &["-c", "seq 1 20000; seq 1 20000 >&2"],
        &temp_dir().to_string_lossy(),
        Duration::from_secs(10),
    )
    .with_capture_limit(12);

    // Act
    let result = SystemCommandRunner.run(&command);

    // Assert
    assert_eq!(
        result,
        CommandResult::Exited {
            code: Some(0),
            stdout: "19999\n20000\n".to_string(),
            stderr: "19999\n20000\n".to_string(),
        }
    );
}
