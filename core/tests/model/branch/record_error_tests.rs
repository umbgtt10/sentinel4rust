// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::branch::record_error::RecordError;
use sentinel4rust::model::toml::toml_error::TomlError;

#[test]
fn from_a_toml_error_is_invalid_with_its_line() {
    // Arrange
    let error = TomlError {
        line: 3,
        message: String::from("unterminated string"),
    };

    // Act
    let record_error = RecordError::from(error);

    // Assert
    assert_eq!(
        record_error,
        RecordError::Invalid(String::from("line 3: unterminated string"))
    );
}

#[test]
fn to_string_names_the_schema_or_the_problem() {
    // Arrange & Act & Assert
    assert_eq!(
        RecordError::UnknownSchema(2).to_string(),
        "unknown summary schema 2"
    );
    assert_eq!(
        RecordError::Invalid(String::from("bad commit")).to_string(),
        "invalid summary: bad commit"
    );
}
