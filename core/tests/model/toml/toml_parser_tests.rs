// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::toml::toml_parser::TomlParser;

#[test]
fn parse_bare_keys_with_dashes_and_underscores_reads_them() {
    // Arrange & Act
    let doc = TomlParser.parse("a-b_c = 1\n[[x-y_z]]\nk = 2").unwrap();

    // Assert
    assert_eq!(doc.root.integer("a-b_c").unwrap(), Some(1));
    assert_eq!(doc.arrays["x-y_z"].len(), 1);
}

#[test]
fn parse_integers_with_underscores_and_signs_reads_them() {
    // Arrange & Act
    let doc = TomlParser.parse("a = 21_600\nb = -3").unwrap();

    // Assert
    assert_eq!(doc.root.integer("a").unwrap(), Some(21600));
    assert_eq!(doc.root.integer("b").unwrap(), Some(-3));
}

#[test]
fn parse_keys_outside_the_bare_set_names_the_line() {
    // Arrange
    let cases = [
        ("a.b = 1", 1, "invalid key `a.b`"),
        ("= 1", 1, "invalid key ``"),
        ("a = 1\n[[a b]]", 2, "invalid table name `a b`"),
        ("[[]]", 1, "invalid table name ``"),
    ];

    // Act
    let errors: Vec<_> = cases
        .iter()
        .map(|(text, _, _)| TomlParser.parse(text).unwrap_err())
        .collect();

    // Assert
    for ((text, line, message), err) in cases.iter().zip(&errors) {
        assert_eq!(err.line, *line, "{text}");
        assert_eq!(err.message, *message, "{text}");
    }
}

#[test]
fn parse_malformed_lines_names_the_line() {
    // Arrange
    let cases = [
        ("a = 1\na = 2", 2, "duplicate"),
        ("a = \"x", 1, "unterminated"),
        ("[table]", 1, "only"),
        ("a = 1.5", 1, "unsupported"),
        ("a 1", 1, "expected `=`"),
        ("a = \"x\" y", 1, "unexpected"),
    ];

    // Act
    let errors: Vec<_> = cases
        .iter()
        .map(|(text, _, _)| TomlParser.parse(text).unwrap_err())
        .collect();

    // Assert
    for ((text, line, fragment), err) in cases.iter().zip(&errors) {
        assert_eq!(err.line, *line, "{text}");
        assert!(err.message.contains(fragment), "{text}: {}", err.message);
    }
}

#[test]
fn parse_root_keys_and_arrays_of_tables_reads_both() {
    // Arrange
    let text = r#"
            # roster
            work_dir = "/tmp/w"   # trailing comment
            jobs = 6
            [[repo]]
            name = "a"
            url = 'git@x:a.git'
            [[repo]]
            name = "b"
            "quoted key" = true
            "#;

    // Act
    let doc = TomlParser.parse(text).unwrap();

    // Assert
    assert_eq!(doc.root.string("work_dir").unwrap().unwrap(), "/tmp/w");
    assert_eq!(doc.root.integer("jobs").unwrap(), Some(6));
    let repos = &doc.arrays["repo"];
    assert_eq!(repos.len(), 2);
    assert_eq!(repos[0].string("url").unwrap().unwrap(), "git@x:a.git");
    assert_eq!(repos[1].keys().collect::<Vec<_>>(), ["name", "quoted key"]);
}

#[test]
fn parse_true_and_false_reads_both() {
    // Arrange & Act
    let doc = TomlParser.parse("a = true\nb = false").unwrap();

    // Assert
    assert_eq!(doc.root.keys().collect::<Vec<_>>(), ["a", "b"]);
}
