// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::files::file_store_tests::FakeFileStore;
use crate::process::command_runner_tests::FakeCommandRunner;
use std::collections::BTreeMap;
use xtask::gates::gate::Gate;
use xtask::gates::stern_gate::SternGate;

const INSTALL_ROOT: &str = "target/stern4rust";
const MARKER: &str = "target/stern4rust/REV";
const PINNED_BINARY: &str = "target/stern4rust/bin/cargo-stern4rust";

fn gate<'a>(runner: &'a FakeCommandRunner, files: &'a FakeFileStore) -> SternGate<'a> {
    SternGate::new(
        runner,
        files,
        String::from("Cargo.toml"),
        vec![String::from("sentinel4rust")],
        String::from(INSTALL_ROOT),
    )
}

fn install_args() -> Vec<String> {
    vec![
        String::from("install"),
        String::from("--git"),
        String::from("https://github.com/umbgtt10/stern4rust"),
        String::from("--rev"),
        String::from("8b0e8e5"),
        String::from("--locked"),
        String::from("--root"),
        String::from(INSTALL_ROOT),
        String::from("cargo-stern4rust"),
    ]
}

fn pinned_files() -> FakeFileStore {
    FakeFileStore::new()
        .with_file(PINNED_BINARY, "")
        .with_file(MARKER, "8b0e8e5\n")
}

fn run_args() -> Vec<String> {
    vec![
        String::from("stern4rust"),
        String::from("--manifest-path"),
        String::from("Cargo.toml"),
        String::from("--package"),
        String::from("sentinel4rust"),
    ]
}

#[test]
fn label_names_the_house_rules_gate() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let files = pinned_files();

    // Act
    let label = gate(&runner, &files).label();

    // Assert
    assert_eq!(label, "House rules");
}

#[test]
fn run_passes_the_manifest_path_and_package_to_the_tool() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let files = pinned_files();

    // Act
    let _ = gate(&runner, &files).run();

    // Assert
    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0],
        vec![
            String::from("stern4rust"),
            String::from("--manifest-path"),
            String::from("Cargo.toml"),
            String::from("--package"),
            String::from("sentinel4rust"),
        ]
    );
}

#[test]
fn run_with_a_marker_naming_another_rev_installs_the_pin_then_runs_it() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let files = FakeFileStore::new()
        .with_file(PINNED_BINARY, "")
        .with_file(MARKER, "1234567\n");

    // Act
    let result = gate(&runner, &files).run();

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(runner.calls(), vec![install_args(), run_args()]);
    assert_eq!(
        files.written(),
        BTreeMap::from([(String::from(MARKER), String::from("8b0e8e5\n"))])
    );
}

#[test]
fn run_with_a_zero_exit_code_returns_ok() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(0));
    let files = pinned_files();

    // Act
    let result = gate(&runner, &files).run();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn run_with_exit_code_one_reports_a_failure_to_run() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(1));
    let files = pinned_files();

    // Act
    let result = gate(&runner, &files).run();

    // Assert
    assert!(result.is_err_and(|error| error.to_string().contains("could not run")));
}

// 2 is a rule broken and 1 is the tool failing to start. Collapsing them would
// let a bad manifest read as a clean codebase.
#[test]
fn run_with_exit_code_two_reports_a_broken_rule() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(2));
    let files = pinned_files();

    // Act
    let result = gate(&runner, &files).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from("a house coding rule was broken"))
    );
}

#[test]
fn run_with_no_marker_installs_the_pin_then_runs_it() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let files = FakeFileStore::new().with_file(PINNED_BINARY, "");

    // Act
    let result = gate(&runner, &files).run();

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(runner.calls(), vec![install_args(), run_args()]);
    assert_eq!(
        files.written(),
        BTreeMap::from([(String::from(MARKER), String::from("8b0e8e5\n"))])
    );
}

#[test]
fn run_with_the_install_failing_returns_the_command_to_run_by_hand() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(101));
    let files = FakeFileStore::new();

    // Act
    let result = gate(&runner, &files).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from(
            "could not install, exit code Some(101) -- run: cargo install --git https://github.com/umbgtt10/stern4rust --rev 8b0e8e5 --locked --root target/stern4rust cargo-stern4rust"
        ))
    );
    assert_eq!(runner.calls(), vec![install_args()]);
    assert!(files.written().is_empty());
}

#[test]
fn run_with_the_marker_unwritable_returns_the_write_failure_without_running() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let files = FakeFileStore::new().with_unwritable(true);

    // Act
    let result = gate(&runner, &files).run();

    // Assert
    assert_eq!(
        result.map_err(|failure| failure.to_string()),
        Err(String::from(
            "failed to write target/stern4rust/REV: denied"
        ))
    );
    assert_eq!(runner.calls(), vec![install_args()]);
}

#[test]
fn run_with_the_pin_installed_runs_the_pinned_binary_without_installing() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let files = pinned_files();

    // Act
    let result = gate(&runner, &files).run();

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(runner.programs(), vec![String::from(PINNED_BINARY)]);
    assert!(files.written().is_empty());
}

#[test]
fn run_with_the_pinned_binary_missing_installs_it_with_cargo_then_runs_it() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let files = FakeFileStore::new().with_file(MARKER, "8b0e8e5\n");

    // Act
    let result = gate(&runner, &files).run();

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(runner.calls(), vec![install_args(), run_args()]);
    assert_eq!(
        runner.programs(),
        vec![String::from("cargo"), String::from(PINNED_BINARY)]
    );
    assert_eq!(
        files.written(),
        BTreeMap::from([(String::from(MARKER), String::from("8b0e8e5\n"))])
    );
}
