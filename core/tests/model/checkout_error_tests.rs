// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::checkout_error::CheckoutError;

#[test]
fn to_string_is_the_reason() {
    // Arrange
    let error = CheckoutError(String::from("git worktree add: timed out"));

    // Act
    let text = error.to_string();

    // Assert
    assert_eq!(text, "git worktree add: timed out");
}
