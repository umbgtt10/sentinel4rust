// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::toml::comment_stripper::CommentStripper;

#[test]
fn code_hash_outside_strings_cuts_the_comment() {
    // Arrange & Act
    let code = CommentStripper.code("jobs = 6   # six");

    // Assert
    assert_eq!(code, Some("jobs = 6   "));
}

#[test]
fn code_unterminated_string_is_none() {
    // Arrange & Act
    let code = CommentStripper.code("a = 'x # y");

    // Assert
    assert_eq!(code, None);
}

#[test]
fn strip_escaped_quote_keeps_the_string_open() {
    // Arrange & Act
    let stripped = CommentStripper.strip(r#"a = "x \" # y" # z"#, 1);

    // Assert
    assert_eq!(stripped, Ok(r#"a = "x \" # y" "#));
}

#[test]
fn strip_hash_inside_strings_is_kept() {
    // Arrange & Act
    let stripped = CommentStripper.strip(r##"a = "#1" b = '#2'"##, 1);

    // Assert
    assert_eq!(stripped, Ok(r##"a = "#1" b = '#2'"##));
}

#[test]
fn strip_hash_outside_strings_cuts_the_comment() {
    // Arrange & Act
    let stripped = CommentStripper.strip("jobs = 6   # six", 1);

    // Assert
    assert_eq!(stripped, Ok("jobs = 6   "));
}

#[test]
fn strip_unterminated_string_fails_naming_the_line() {
    // Arrange & Act
    let error = CommentStripper.strip("a = 'x", 9).unwrap_err();

    // Assert
    assert_eq!(error.line, 9);
    assert_eq!(error.message, "unterminated string");
}
