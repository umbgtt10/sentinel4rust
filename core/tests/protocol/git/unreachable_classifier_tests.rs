// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::command_results::failed;
use sentinel4rust::model::command_result::CommandResult;
use sentinel4rust::model::sync_error::SyncError;
use sentinel4rust::protocol::git::unreachable_classifier::UnreachableClassifier;

#[test]
fn classify_a_timeout_is_transient() {
    // Arrange
    let result = CommandResult::TimedOut {
        stdout: String::new(),
        stderr: String::from("Permission denied"),
    };

    // Act
    let error = UnreachableClassifier.classify(&result, "git fetch");

    // Assert
    assert_eq!(
        error,
        SyncError::Transient(String::from("git fetch: timed out"))
    );
}

#[test]
fn classify_an_unknown_failure_is_transient() {
    // Arrange
    let result = failed(128, "fatal: unable to access: Could not resolve host");

    // Act
    let error = UnreachableClassifier.classify(&result, "git ls-remote");

    // Assert
    assert_eq!(
        error,
        SyncError::Transient(String::from(
            "git ls-remote: exited with 128: fatal: unable to access: Could not resolve host"
        ))
    );
}

#[test]
fn classify_each_marker_in_any_case_is_unreachable() {
    // Arrange
    let stderrs = [
        "ERROR: Repository not found.",
        "git@github.com: Permission denied (publickey).",
        "fatal: '/x' does not appear to be a git repository",
        "remote: Access denied",
        "fatal: could not read Username for 'https://github.com'",
        "fatal: Authentication failed for 'https://x'",
    ];

    // Act
    let errors: Vec<SyncError> = stderrs
        .iter()
        .map(|stderr| UnreachableClassifier.classify(&failed(128, stderr), "git fetch"))
        .collect();

    // Assert
    for (error, stderr) in errors.iter().zip(stderrs) {
        assert_eq!(
            error,
            &SyncError::Unreachable(format!("git fetch: exited with 128: {stderr}"))
        );
    }
}
