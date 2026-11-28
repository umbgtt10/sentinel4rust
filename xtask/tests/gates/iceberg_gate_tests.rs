// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::gates::support::fake_command_runner::FakeCommandRunner;
use serge4rust::gates::gate::Gate;
use xtask::gates::iceberg_gate::IcebergGate;

fn gate(runner: &FakeCommandRunner) -> IcebergGate<'_> {
    IcebergGate::new(
        runner,
        String::from("Cargo.toml"),
        vec![String::from("sentinel4rust")],
        String::from("20"),
    )
}

#[test]
fn label_names_the_file_risk_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let label = gate(&runner).label();

    // Assert
    assert_eq!(label, "File risk");
}

#[test]
fn run_passes_the_manifest_path_packages_and_threshold_to_the_installed_tool() {
    // Arrange
    let runner = FakeCommandRunner::new();

    // Act
    let _ = gate(&runner).run();

    // Assert
    assert_eq!(
        runner.calls()[0],
        vec![
            String::from("iceberg4rust"),
            String::from("--manifest-path"),
            String::from("Cargo.toml"),
            String::from("--package"),
            String::from("sentinel4rust"),
            String::from("--threshold"),
            String::from("20"),
        ]
    );
}

#[test]
fn run_with_a_zero_exit_code_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(0));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn run_with_exit_code_one_reports_the_exit_code() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(1));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from("exit code Some(1)"))
    );
}

// 2 is iceberg4rust's own "offenders found", which the gate must report as a
// breached ceiling rather than as the tool failing.
#[test]
fn run_with_exit_code_two_names_the_ceiling_it_breached() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(2));

    // Act
    let result = gate(&runner).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from("a file is at or above the ceiling of 20"))
    );
}

#[test]
fn run_with_the_tool_missing_returns_an_install_hint() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);

    // Act
    let result = gate(&runner).run();

    // Assert
    assert!(result.is_err_and(|error| {
        error
            .to_string()
            .contains("cargo install cargo-iceberg4rust")
    }));
}
