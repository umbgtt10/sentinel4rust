// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::onboarding::roster_error::RosterError;
use sentinel4rust::model::toml::toml_error::TomlError;

#[test]
fn from_a_toml_error_keeps_its_line_and_message() {
    // Arrange
    let error = TomlError {
        line: 3,
        message: String::from("unterminated string"),
    };

    // Act
    let roster_error = RosterError::from(error);

    // Assert
    assert_eq!(
        roster_error,
        RosterError(String::from("line 3: unterminated string"))
    );
}

#[test]
fn to_string_names_the_roster_and_the_problem() {
    // Arrange & Act
    let text = RosterError(String::from("no [[repo]] entries")).to_string();

    // Assert
    assert_eq!(text, "roster: no [[repo]] entries");
}
