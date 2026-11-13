// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The test timeout a cargo-mutants run set, as its debug log says it.

/// What cargo-mutants 27.1.0 writes, at `INFO`, once it has timed the baseline: `Auto-set test timeout to 240s`.
const MARKER: &str = "Auto-set test timeout to ";

/// Reads the test timeout out of `mutants.out/debug.log`, where cargo-mutants states the one it set from the baseline
/// -- no other file of `mutants.out` holds it.
#[derive(Debug, Clone, Copy, Default)]
pub struct TestTimeoutLine;

impl TestTimeoutLine {
    /// The timeout in whole seconds, as cargo-mutants writes it; `None` when the log does not say it in that form.
    pub fn seconds(&self, debug_log: &str) -> Option<u64> {
        debug_log
            .lines()
            .find_map(|line| line.split_once(MARKER))
            .and_then(|(_, rest)| rest.trim_end().strip_suffix('s'))
            .and_then(|seconds| seconds.parse().ok())
    }
}
