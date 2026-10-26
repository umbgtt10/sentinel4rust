// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::scripted_runner::failed;
use sentinel4rust_cli::system::git::git_failure::GitFailure;

#[test]
fn to_string_names_what_failed_and_how() {
    // Arrange
    let failure = GitFailure {
        what: String::from("git hash-object -w /s/blob-0"),
        result: failed(128, "\nfatal: unable to write\n"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(
        text,
        "git hash-object -w /s/blob-0: exited with 128: fatal: unable to write"
    );
}
