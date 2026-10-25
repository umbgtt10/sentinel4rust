// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::run_records::record;
use sentinel4rust::model::branch::record_currency::RecordCurrency;
use sentinel4rust::model::commit_hash::CommitHash;
use sentinel4rust::model::outcome_kind::OutcomeKind;

const INSTALLED: &str = "cargo-mutants 27.1.0";

#[test]
fn is_current_needs_commit_and_config_version() {
    // Arrange
    let r = record(OutcomeKind::Completed);
    let other = CommitHash::parse(&"cd".repeat(20)).unwrap();
    let installed = Some(String::from(INSTALLED));

    // Act & Assert
    assert!(RecordCurrency::new(2, installed.clone()).is_current(&r, &r.commit));
    assert!(!RecordCurrency::new(3, installed.clone()).is_current(&r, &r.commit));
    assert!(!RecordCurrency::new(2, installed).is_current(&r, &other));
}

#[test]
fn is_current_with_another_cargo_mutants_installed_is_false() {
    // Arrange
    let r = record(OutcomeKind::Completed);

    // Act
    let current = RecordCurrency::new(2, Some(String::from("cargo-mutants 27.2.0")))
        .is_current(&r, &r.commit);

    // Assert
    assert!(!current);
}

// An installed version that could not be read must not rerun every
// repository on every visit: commit and config_version decide alone.
#[test]
fn is_current_with_the_installed_version_unknown_ignores_it() {
    // Arrange
    let mut r = record(OutcomeKind::Completed);
    r.cargo_mutants_version = String::from("cargo-mutants 26.0.0");

    // Act
    let current = RecordCurrency::new(2, None).is_current(&r, &r.commit);

    // Assert
    assert!(current);
}
