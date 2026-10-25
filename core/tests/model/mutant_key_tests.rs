// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::key_changes::KeyChanges;
use sentinel4rust::model::mutant_key::MutantKey;

fn keys(lines: &[&str]) -> Vec<String> {
    let lines: Vec<String> = lines.iter().map(ToString::to_string).collect();
    MutantKey::from_located(&lines)
        .into_iter()
        .map(|k| k.as_str().to_string())
        .collect()
}

#[test]
fn changes_between_lists_are_set_differences() {
    // Arrange
    let a = [MutantKey::new("x"), MutantKey::new("y")];
    let b = [
        MutantKey::new("y"),
        MutantKey::new("z"),
        MutantKey::new("w"),
    ];

    // Act & Assert
    assert_eq!(MutantKey::changes(&a, &b), KeyChanges { new: 2, gone: 1 });
    assert_eq!(MutantKey::changes(&[], &b), KeyChanges { new: 3, gone: 0 });
    assert_eq!(MutantKey::changes(&a, &a), KeyChanges { new: 0, gone: 0 });
}

#[test]
fn from_located_colons_and_unrecognised_lines_are_kept() {
    // Arrange & Act
    let keys = keys(&["odd:dir/x.rs:4:2: replace a with b", "  something else  "]);

    // Assert
    assert_eq!(keys, ["odd:dir/x.rs: replace a with b", "something else"]);
}

#[test]
fn from_located_drops_line_and_column() {
    // Arrange & Act
    let keys = keys(&["src/lib.rs:12:5: replace == with != in Parser::next"]);

    // Assert
    assert_eq!(keys, ["src/lib.rs: replace == with != in Parser::next"]);
}

#[test]
fn from_located_duplicates_get_ordinals_in_source_order() {
    // Arrange & Act
    let keys = keys(&[
        "src/a.rs:20:5: replace + with - in g",
        "src/a.rs:10:5: replace + with - in g",
        "src/b.rs:1:1: replace + with - in g",
    ]);

    // Assert
    assert_eq!(
        keys,
        [
            "src/a.rs: replace + with - in g",
            "src/a.rs: replace + with - in g #2",
            "src/b.rs: replace + with - in g",
        ]
    );
}

#[test]
fn from_located_lines_inserted_above_keep_their_keys() {
    // Arrange
    let before = keys(&[
        "src/a.rs:3:9: replace f -> bool with true",
        "src/a.rs:8:5: replace + with - in g",
    ]);

    // Act
    let after = keys(&[
        "src/a.rs:13:9: replace f -> bool with true",
        "src/a.rs:18:5: replace + with - in g",
    ]);

    // Assert
    assert_eq!(before, after);
}
