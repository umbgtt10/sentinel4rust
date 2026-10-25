// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::publish_error::PublishError;

#[test]
fn to_string_is_the_reason() {
    // Arrange
    let error = PublishError(String::from("git push: exited with 1: rejected"));

    // Act
    let text = error.to_string();

    // Assert
    assert_eq!(text, "git push: exited with 1: rejected");
}
