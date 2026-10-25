// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::run_records::record;
use sentinel4rust::model::branch::record_error::RecordError;
use sentinel4rust::model::branch::summary_toml::SummaryToml;
use sentinel4rust::model::outcome_kind::OutcomeKind;

#[test]
fn parse_rendered_record_round_trips() {
    // Arrange
    let mut r = record(OutcomeKind::ToolFailed);
    r.reason = Some("exit 70".to_string());
    r.log_tail = Some("error: \"x\"\n\tat y\n".to_string());
    let plain = record(OutcomeKind::Completed);

    // Act
    let text = SummaryToml.render(&r);

    // Assert
    assert!(text.contains("duration_seconds = 3600"));
    assert_eq!(SummaryToml.parse(&text).unwrap(), r);
    assert_eq!(
        SummaryToml.parse(&SummaryToml.render(&plain)).unwrap(),
        plain
    );
}

#[test]
fn parse_unknown_schema_or_garbage_fails() {
    // Arrange
    let text = SummaryToml
        .render(&record(OutcomeKind::Completed))
        .replace("schema = 1", "schema = 2");

    // Act & Assert
    assert_eq!(SummaryToml.parse(&text), Err(RecordError::UnknownSchema(2)));
    assert!(matches!(
        SummaryToml.parse("schema = 1"),
        Err(RecordError::Invalid(_))
    ));
    assert!(matches!(
        SummaryToml.parse("[x"),
        Err(RecordError::Invalid(_))
    ));
}
