// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::toml::basic_string::BasicString;

#[test]
fn read_a_raw_control_character_fails_naming_it() {
    // Arrange
    let cases = [
        ("\u{0}\"", "unescaped control character `U+0000`"),
        ("a\u{7}b\"", "unescaped control character `U+0007`"),
        ("a\rb\"", "unescaped control character `U+000D`"),
        ("a\u{1f}\"", "unescaped control character `U+001F`"),
        ("a\u{7f}\"", "unescaped control character `U+007F`"),
    ];

    // Act
    let errors: Vec<_> = cases
        .iter()
        .map(|(text, _)| BasicString.read(text, 4).unwrap_err())
        .collect();

    // Assert
    for ((text, message), error) in cases.iter().zip(&errors) {
        assert_eq!(error.line, 4, "{text:?}");
        assert_eq!(error.message, *message, "{text:?}");
    }
}

#[test]
fn read_a_raw_tab_or_other_character_keeps_it() {
    // Arrange
    let text = "a\tb\u{80}\u{a0}\"";

    // Act
    let read = BasicString.read(text, 1);

    // Assert
    assert_eq!(read, Ok((String::from("a\tb\u{80}\u{a0}"), 8)));
}

#[test]
fn read_an_escape_cut_short_fails_naming_it() {
    // Arrange
    let cases = [
        (r"a\", "unterminated escape"),
        (r#"\uZZZZ""#, r"invalid escape `\uZZZZ`"),
    ];

    // Act
    let errors: Vec<_> = cases
        .iter()
        .map(|(text, _)| BasicString.read(text, 3).unwrap_err())
        .collect();

    // Assert
    for ((text, message), error) in cases.iter().zip(&errors) {
        assert_eq!(error.line, 3, "{text}");
        assert_eq!(error.message, *message, "{text}");
    }
}

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
fn read_multibyte_text_counts_its_bytes_to_the_closing_quote() {
    // Arrange
    let text = r#"é\u00e9" rest"#;

    // Act
    let read = BasicString.read(text, 1);

    // Assert
    assert_eq!(read, Ok((String::from("éé"), 9)));
    assert_eq!(&text[9..], " rest");
}

#[test]
fn read_unterminated_text_fails() {
    // Arrange & Act
    let error = BasicString.read("no end", 2).unwrap_err();

    // Assert
    assert_eq!(error.message, "unterminated string");
}
