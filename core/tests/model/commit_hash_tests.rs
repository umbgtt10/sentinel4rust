// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::commit_hash::CommitHash;

const HASH: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn parse_anything_else_returns_none() {
    // Arrange
    let texts = [
        String::new(),
        String::from("0123456"),
        HASH[..39].to_string(),
        format!("{HASH}0"),
        HASH.replace('a', "g"),
    ];

    // Act
    let parsed: Vec<Option<CommitHash>> =
        texts.iter().map(|text| CommitHash::parse(text)).collect();

    // Assert
    for (text, hash) in texts.iter().zip(parsed) {
        assert_eq!(hash, None, "{text}");
    }
}

#[test]
fn parse_forty_hex_digits_trims_and_lowercases() {
    // Arrange & Act
    let hash = CommitHash::parse(&format!(" {}\n", HASH.to_uppercase())).unwrap();

    // Assert
    assert_eq!(hash.as_str(), HASH);
    assert_eq!(hash.short(), "0123456");
    assert_eq!(hash.to_string(), HASH);
}
