// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::toml::document_builder::DocumentBuilder;
use sentinel4rust::model::toml::value::Value;

#[test]
fn finish_without_arrays_returns_the_root_on_line_one() {
    // Arrange
    let mut builder = DocumentBuilder::new();
    builder.insert("a", Value::Integer(1), 3).unwrap();

    // Act
    let document = builder.finish();

    // Assert
    assert_eq!(document.root.line(), 1);
    assert_eq!(document.root.keys().collect::<Vec<_>>(), ["a"]);
    assert!(document.arrays.is_empty());
}

#[test]
fn insert_a_duplicate_key_fails_naming_its_line() {
    // Arrange
    let mut builder = DocumentBuilder::new();
    builder.insert("a", Value::Integer(1), 1).unwrap();

    // Act
    let error = builder.insert("a", Value::Integer(2), 2).unwrap_err();

    // Assert
    assert_eq!(error.line, 2);
    assert_eq!(error.message, "duplicate key `a`");
}

#[test]
fn start_array_collects_its_tables_in_order() {
    // Arrange
    let mut builder = DocumentBuilder::new();
    builder.start_array(String::from("repo"), 4);
    builder
        .insert("name", Value::String(String::from("a")), 5)
        .unwrap();
    builder.start_array(String::from("repo"), 6);

    // Act
    let document = builder.finish();

    // Assert
    let repos = &document.arrays["repo"];
    assert_eq!(repos.len(), 2);
    assert_eq!(repos[0].line(), 4);
    assert_eq!(repos[0].string("name"), Ok(Some(String::from("a"))));
    assert_eq!(repos[1].line(), 6);
}
