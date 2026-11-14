// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::github::discovery_error::DiscoveryError;

#[test]
fn to_string_is_the_reason() {
    // Arrange
    let error = DiscoveryError(String::from("curl: exited with 6: could not resolve host"));

    // Act
    let text = error.to_string();

    // Assert
    assert_eq!(text, "curl: exited with 6: could not resolve host");
}
