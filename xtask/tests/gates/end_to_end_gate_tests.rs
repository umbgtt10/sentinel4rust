// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use serge4rust::gates::gate::Gate;
use xtask::gates::end_to_end_gate::EndToEndGate;

const INSTALL_HINT: &str = "cargo-mutants 27.1.0 is not installed -- run: cargo install cargo-mutants --version 27.1.0 --locked";

fn gate(runner: &FakeCommandRunner) -> EndToEndGate<'_> {
    EndToEndGate::new(runner, String::from("Cargo.toml"), String::from("27.1.0"))
}

fn pinned_runner() -> FakeCommandRunner {
    FakeCommandRunner::new().with_stdout("cargo-mutants 27.1.0\n")
}

#[test]
fn label_names_the_end_to_end_gate() {
    // Arrange
    let runner = pinned_runner();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "End to end (real cargo-mutants)");
}

#[test]
fn run_asks_the_version_then_runs_only_the_ignored_tests() {
    // Arrange
    let runner = pinned_runner();

    // Act
    let _ = gate(&runner).run();

    // Assert
    assert_eq!(
        runner.calls(),
        vec![
            vec![String::from("mutants"), String::from("--version")],
            vec![
                String::from("test"),
                String::from("--manifest-path"),
                String::from("Cargo.toml"),
                String::from("--workspace"),
                String::from("--"),
                String::from("--ignored"),
            ],
        ]
    );
}

#[test]
fn run_ended_by_a_signal_reports_the_exit() {
    // Arrange
    let runner = pinned_runner()
        .with_streaming_code(None)
        .with_capturing_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from("exit code None"))
    );
}

#[test]
fn run_with_a_failing_test_reports_it() {
    // Arrange
    let runner = pinned_runner()
        .with_streaming_code(Some(101))
        .with_capturing_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from(
            "an end-to-end test against the real cargo-mutants failed"
        ))
    );
}

#[test]
fn run_with_another_version_installed_names_the_pinned_one() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout("cargo-mutants 26.2.0\n");

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from(INSTALL_HINT))
    );
}

#[test]
fn run_with_the_pinned_version_and_passing_tests_returns_ok() {
    // Arrange
    let runner = pinned_runner();

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn run_with_the_tool_missing_names_the_pinned_version() {
    // Arrange
    let runner = pinned_runner().with_available(false);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from(INSTALL_HINT))
    );
    assert_eq!(runner.calls(), Vec::<Vec<String>>::new());
}
