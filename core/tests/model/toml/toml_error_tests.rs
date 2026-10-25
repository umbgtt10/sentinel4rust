// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::toml::toml_error::TomlError;

#[test]
fn to_string_names_the_line_and_the_message() {
    // Arrange
    let error = TomlError {
        line: 12,
        message: String::from("unknown key `x`"),
    };

    // Act
    let text = error.to_string();

    // Assert
    assert_eq!(text, "line 12: unknown key `x`");
}
