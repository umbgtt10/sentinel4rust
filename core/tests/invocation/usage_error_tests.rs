// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::invocation::usage_error::UsageError;

#[test]
fn to_string_of_each_names_the_problem() {
    // Arrange & Act & Assert
    assert_eq!(
        UsageError::MissingRosterFile.to_string(),
        "--roster needs a file"
    );
    assert_eq!(
        UsageError::MissingRoster.to_string(),
        "--roster is required"
    );
    assert_eq!(
        UsageError::MissingJobDir.to_string(),
        "--run-job needs a directory"
    );
    assert_eq!(
        UsageError::UnknownArgument(String::from("--frobnicate")).to_string(),
        "unknown argument `--frobnicate`"
    );
}
