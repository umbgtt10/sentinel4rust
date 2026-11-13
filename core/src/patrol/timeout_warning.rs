// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A completed run whose timeouts outnumber its caught mutants.

use crate::model::counts::MutantCounts;

/// The warning's name, as the journal and the status pass write it: `warning=timeouts-outnumber-caught`.
const TIMEOUTS_OUTNUMBER_CAUGHT: &str = "timeouts-outnumber-caught";

/// Judges a run whose mutants timed out more often than they were caught: its lists say little, and the likeliest
/// cause is a test timeout below the tests' own run -- cargo-mutants timing its baseline without the packages
/// `test_package` names (`docs/OPEN_POINTS.md` point 10).
#[derive(Debug, Clone, Copy, Default)]
pub struct TimeoutWarning;

impl TimeoutWarning {
    /// The warning `counts` call for, or none: a failed run's counts are all zero, and call for none.
    pub fn of(&self, counts: &MutantCounts) -> Option<&'static str> {
        (counts.timeout > counts.caught).then_some(TIMEOUTS_OUTNUMBER_CAUGHT)
    }
}
