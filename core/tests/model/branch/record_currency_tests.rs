// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::run_records::record;
use sentinel4rust::model::branch::record_currency::RecordCurrency;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::outcome_kind::OutcomeKind;

#[test]
fn is_current_needs_commit_and_config_version() {
    // Arrange
    let r = record(OutcomeKind::Completed);
    let other = CommitHash::parse(&"cd".repeat(20)).unwrap();

    // Act & Assert
    assert!(RecordCurrency::new(2).is_current(&r, &r.commit));
    assert!(!RecordCurrency::new(3).is_current(&r, &r.commit));
    assert!(!RecordCurrency::new(2).is_current(&r, &other));
}
