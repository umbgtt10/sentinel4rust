// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use xtask::files::file_failure::FileFailure;

#[test]
fn to_string_names_the_path_and_why() {
    // Arrange
    let failure = FileFailure {
        path: String::from("target/stern4rust/REV"),
        reason: String::from("Permission denied"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(
        text,
        "failed to write target/stern4rust/REV: Permission denied"
    );
}
