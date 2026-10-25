// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::toml::table::Table;
use sentinel4rust::model::toml::toml_error::TomlError;
use sentinel4rust::model::toml::toml_parser::TomlParser;
use sentinel4rust::model::toml::value::Value;

#[test]
fn insert_a_key_twice_keeps_the_first_and_says_so() {
    // Arrange
    let mut table = Table::new(3);

    // Act
    let first = table.insert(String::from("a"), Value::Integer(1));
    let second = table.insert(String::from("a"), Value::Integer(2));

    // Assert
    assert!(first);
    assert!(!second);
    assert_eq!(table.integer("a"), Ok(Some(1)));
}

#[test]
fn new_table_has_its_line_and_no_keys() {
    // Arrange & Act
    let table = Table::new(7);

    // Assert
    assert_eq!(table.line(), 7);
    assert_eq!(table.keys().count(), 0);
}

#[test]
fn required_integer_of_a_missing_key_names_it() {
    // Arrange
    let mut table = Table::new(4);
    table.insert(String::from("a"), Value::Integer(1));

    // Act
    let missing = table.required_integer("b");

    // Assert
    assert_eq!(table.required_integer("a"), Ok(1));
    assert_eq!(
        missing,
        Err(TomlError {
            line: 4,
            message: String::from("`b` is missing"),
        })
    );
}

#[test]
fn string_integer_and_only_reject_wrong_types_and_unknown_keys() {
    // Arrange
    let doc = TomlParser.parse("a = 1\nb = \"x\"").unwrap();

    // Act & Assert
    assert!(doc.root.string("a").is_err());
    assert!(doc.root.integer("b").is_err());
    assert!(doc.root.required_string("c").is_err());
    assert!(doc.root.only(&["a"]).is_err());
    assert!(doc.root.only(&["a", "b"]).is_ok());
}
