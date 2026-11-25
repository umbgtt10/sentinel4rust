// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::toml::toml_parser::TomlParser;
use sentinel4rust::model::toml::toml_writer::TomlWriter;

#[test]
fn quote_awkward_text_round_trips_through_the_parser() {
    // Arrange
    let text = "a \"b\" \\ c\nd\te # f \u{1}";

    // Act
    let doc = TomlParser
        .parse(&format!("k = {}", TomlWriter.quote(text)))
        .unwrap();

    // Assert
    assert_eq!(doc.root.string("k").unwrap().unwrap(), text);
}

#[test]
fn quote_control_and_plain_characters_escapes_only_the_control_ones() {
    // Arrange & Act
    let quoted = TomlWriter.quote("a\u{1}\u{7f}");

    // Assert
    assert_eq!(quoted, "\"a\\u0001\\u007F\"");
}
