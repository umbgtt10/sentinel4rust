// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::skip_reason::SkipReason;

#[test]
fn as_str_of_each_reason_is_its_journal_text() {
    // Arrange
    let reasons = [
        SkipReason::NoBranch,
        SkipReason::NotACrate,
        SkipReason::NoFilters,
    ];

    // Act
    let texts: Vec<&str> = reasons.iter().map(|reason| reason.as_str()).collect();

    // Assert
    assert_eq!(texts, ["no-branch", "not-a-crate", "no-filters"]);
}
