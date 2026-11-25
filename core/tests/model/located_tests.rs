// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::located::Located;

#[test]
fn base_of_a_line_without_a_position_is_its_trimmed_text() {
    // Arrange
    let located = Located::parse("  no position here ");

    // Act
    let base = located.base();

    // Assert
    assert_eq!(base, "no position here");
}

#[test]
fn base_of_a_located_line_drops_line_and_column() {
    // Arrange
    let located = Located::parse("src/x.rs:7:3: replace a with b");

    // Act
    let base = located.base();

    // Assert
    assert_eq!(base, "src/x.rs: replace a with b");
}

#[test]
fn base_of_a_signed_line_or_column_is_the_whole_line() {
    // Arrange
    let lines = [
        "src/x.rs:+7:3: replace a with b",
        "src/x.rs:7:+3: replace a with b",
    ];

    // Act
    let bases: Vec<String> = lines
        .iter()
        .map(|line| Located::parse(line).base())
        .collect();

    // Assert
    assert_eq!(bases, lines);
}

#[test]
fn parse_orders_by_file_then_line_then_column() {
    // Arrange
    let mut parsed = vec![
        Located::parse("src/b.rs:1:1: x"),
        Located::parse("src/a.rs:10:1: x"),
        Located::parse("src/a.rs:2:9: x"),
        Located::parse("src/a.rs:2:3: x"),
    ];

    // Act
    parsed.sort();

    // Assert
    assert_eq!(
        parsed,
        [
            Located::parse("src/a.rs:2:3: x"),
            Located::parse("src/a.rs:2:9: x"),
            Located::parse("src/a.rs:10:1: x"),
            Located::parse("src/b.rs:1:1: x"),
        ]
    );
}
