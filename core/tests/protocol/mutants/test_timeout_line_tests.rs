// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::protocol::mutants::test_timeout_line::TestTimeoutLine;

// The end of the baseline in the debug log cargo-mutants 27.1.0 wrote on
// 2026-11-13 for the `spin` fixture of main_tests.rs, whose .cargo/mutants.toml
// sets the timeout's floor to 2 s; unedited but for the home directory in its
// paths.
const SPIN_DEBUG_LOG: &str = include_str!("../../fixtures/mutants_out/timeouts/debug.log");

#[test]
fn seconds_of_a_log_without_the_line_is_none() {
    // Arrange & Act
    let seconds = TestTimeoutLine
        .seconds("   0.04s DEBUG cargo_mutants::lab: lab.rs:56: starting jobserver n_tasks=8\n");

    // Assert
    assert_eq!(seconds, None);
}

#[test]
fn seconds_of_a_real_debug_log_is_the_timeout_cargo_mutants_set() {
    // Arrange & Act
    let seconds = TestTimeoutLine.seconds(SPIN_DEBUG_LOG);

    // Assert
    assert_eq!(seconds, Some(2));
}

// cargo-mutants writes whole seconds; anything else is not read as some
// other number.
#[test]
fn seconds_of_a_value_that_is_not_whole_seconds_is_none() {
    // Arrange & Act
    let seconds = TestTimeoutLine.seconds(
        "   0.94s  INFO cargo_mutants::timeouts: timeouts.rs:68: Auto-set test timeout to 2.5s\n",
    );

    // Assert
    assert_eq!(seconds, None);
}
