// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::counts::MutantCounts;
use sentinel4rust::patrol::timeout_warning::TimeoutWarning;

fn counts(caught: u64, timeout: u64) -> MutantCounts {
    MutantCounts {
        caught,
        missed: 2,
        timeout,
        unviable: 9,
    }
}

#[test]
fn of_counts_with_as_many_timeouts_as_caught_is_none() {
    // Arrange & Act
    let warning = TimeoutWarning.of(&counts(5, 5));

    // Assert
    assert_eq!(warning, None);
}

// beggar on the development laptop, 2026-11-13: a 20 s test timeout under a
// 45 s ladder.
#[test]
fn of_counts_with_more_timeouts_than_caught_is_the_warning() {
    // Arrange & Act
    let warning = TimeoutWarning.of(&counts(74, 121));

    // Assert
    assert_eq!(warning, Some("timeouts-outnumber-caught"));
}

// A run that failed publishes no counts, and so no warning.
#[test]
fn of_counts_with_none_of_either_is_none() {
    // Arrange & Act
    let warning = TimeoutWarning.of(&MutantCounts::default());

    // Assert
    assert_eq!(warning, None);
}
