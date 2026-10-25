// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::timestamp::Timestamp;
use std::time::Duration;

#[test]
fn parse_displayed_instants_round_trips() {
    // Arrange
    let instants: Vec<Timestamp> = [0, 59, 951_782_400, 1_792_812_345, 4_102_444_799]
        .into_iter()
        .map(Timestamp::from_unix)
        .collect();

    // Act
    let parsed: Vec<Option<Timestamp>> = instants
        .iter()
        .map(|t| Timestamp::parse(&t.to_string()))
        .collect();

    // Assert
    for (t, back) in instants.iter().zip(parsed) {
        assert_eq!(back, Some(*t));
    }
}

#[test]
fn parse_malformed_text_returns_none() {
    // Arrange
    let texts = [
        "",
        "2026-10-24",
        "2026-13-01T00:00:00Z",
        "2026-10-24T24:00:00Z",
        "2026-10-24 00:00:00Z",
    ];

    // Act
    let parsed: Vec<Option<Timestamp>> = texts.iter().map(|text| Timestamp::parse(text)).collect();

    // Assert
    for (text, t) in texts.iter().zip(parsed) {
        assert_eq!(t, None, "{text}");
    }
}

#[test]
fn plus_and_since_saturate_and_clamp() {
    // Arrange
    let t = Timestamp::from_unix(100);

    // Act & Assert
    assert_eq!(t.plus(Duration::from_secs(5)).unix(), 105);
    assert_eq!(t.since(Timestamp::from_unix(40)), Duration::from_secs(60));
    assert_eq!(Timestamp::from_unix(40).since(t), Duration::ZERO);
}

#[test]
fn to_string_known_instants_reads_rfc3339() {
    // Arrange & Act & Assert
    assert_eq!(Timestamp::from_unix(0).to_string(), "1970-01-01T00:00:00Z");
    assert_eq!(
        Timestamp::from_unix(1_792_800_000).to_string(),
        "2026-10-24T00:00:00Z"
    );
    assert_eq!(
        Timestamp::from_unix(951_782_400).to_string(),
        "2000-02-29T00:00:00Z"
    );
}
