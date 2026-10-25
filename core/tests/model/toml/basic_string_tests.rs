// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::toml::basic_string::BasicString;

#[test]
fn read_escapes_decodes_them_and_counts_the_closing_quote() {
    // Arrange
    let text = r#"a\"b\\c\n\t\r\u0041" rest"#;

    // Act
    let read = BasicString.read(text, 1);

    // Assert
    assert_eq!(read, Ok((String::from("a\"b\\c\n\t\rA"), 20)));
    assert_eq!(&text[20..], " rest");
}

#[test]
fn read_invalid_escape_fails_naming_it() {
    // Arrange & Act
    let error = BasicString.read(r#"a\q""#, 5).unwrap_err();

    // Assert
    assert_eq!(error.line, 5);
    assert_eq!(error.message, "invalid escape `\\q`");
}

#[test]
fn read_unterminated_text_fails() {
    // Arrange & Act
    let error = BasicString.read("no end", 2).unwrap_err();

    // Assert
    assert_eq!(error.message, "unterminated string");
}
