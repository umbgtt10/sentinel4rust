// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::protocol::mutants::log_tail::LogTail;
use std::fmt::Write;

const BYTES: usize = 16 * 1024;
const LINES: usize = 200;

#[test]
fn of_long_output_keeps_the_last_lines_and_bytes() {
    // Arrange
    let log_tail = LogTail::new(LINES, BYTES);
    let many: String = (0..500).fold(String::new(), |mut acc, i| {
        let _ = writeln!(acc, "line {i}");
        acc
    });
    let long = "é".repeat(BYTES);

    // Act
    let tail = log_tail.of(&many, "");
    let cut = log_tail.of(&long, "");

    // Assert
    assert_eq!(tail.lines().count(), LINES);
    assert!(tail.ends_with("line 499"));
    assert!(tail.starts_with("line 300"));
    assert!(cut.len() <= BYTES);
    assert_eq!(log_tail.of("", "err\n"), "err");
    assert_eq!(log_tail.of("out\n", "err\n"), "out\nerr");
}
