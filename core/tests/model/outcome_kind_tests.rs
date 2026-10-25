// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::outcome_kind::OutcomeKind;

const KINDS: [(OutcomeKind, &str); 3] = [
    (OutcomeKind::Completed, "completed"),
    (OutcomeKind::BaselineFailed, "baseline-failed"),
    (OutcomeKind::ToolFailed, "tool-failed"),
];

#[test]
fn as_str_of_each_kind_is_its_summary_text() {
    // Arrange & Act
    let texts: Vec<&str> = KINDS.iter().map(|(kind, _)| kind.as_str()).collect();

    // Assert
    assert_eq!(texts, ["completed", "baseline-failed", "tool-failed"]);
}

#[test]
fn parse_each_summary_text_reads_its_kind() {
    // Arrange & Act
    let kinds: Vec<Option<OutcomeKind>> = KINDS
        .iter()
        .map(|(_, text)| OutcomeKind::parse(text))
        .collect();

    // Assert
    assert_eq!(
        kinds,
        KINDS
            .iter()
            .map(|(kind, _)| Some(*kind))
            .collect::<Vec<_>>()
    );
}

#[test]
fn parse_unknown_text_returns_none() {
    // Arrange & Act
    let kind = OutcomeKind::parse("finished");

    // Assert
    assert_eq!(kind, None);
}

#[test]
fn to_string_is_the_summary_text() {
    // Arrange & Act
    let text = OutcomeKind::BaselineFailed.to_string();

    // Assert
    assert_eq!(text, "baseline-failed");
}
