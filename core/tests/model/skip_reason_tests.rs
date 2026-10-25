// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::skip_reason::SkipReason;

#[test]
fn as_str_of_a_missing_toolchain_is_its_journal_text() {
    // Arrange
    let reason = SkipReason::ToolchainMissing {
        message: String::from(
            "exited with 1: error: toolchain '1.99.1-x86_64-unknown-linux-gnu' is not installed",
        ),
    };

    // Act
    let text = reason.as_str();

    // Assert
    assert_eq!(text, "toolchain-missing");
}

#[test]
fn as_str_of_a_path_outside_the_repository_is_its_journal_text() {
    // Arrange
    let reason = SkipReason::PathOutsideRepo {
        manifest: String::from("crates/a/Cargo.toml"),
    };

    // Act
    let text = reason.as_str();

    // Assert
    assert_eq!(text, "path-outside-repo");
}

#[test]
fn as_str_of_each_reason_is_its_journal_text() {
    // Arrange
    let reasons = [
        SkipReason::NoBranch,
        SkipReason::NotACrate,
        SkipReason::NoFilters,
    ];

    // Act
    let texts: Vec<&str> = reasons.iter().map(SkipReason::as_str).collect();

    // Assert
    assert_eq!(texts, ["no-branch", "not-a-crate", "no-filters"]);
}
