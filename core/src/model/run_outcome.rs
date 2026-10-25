// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The result of one cargo-mutants run.

use crate::model::counts::MutantCounts;

/// The result of one cargo-mutants run, as the runner reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    /// The run finished: exit 0, 2 or 3. Lists are as cargo-mutants wrote them, with positions.
    Completed {
        /// Caught, missed, timed out, unviable.
        counts: MutantCounts,
        /// `mutants.out/missed.txt`.
        missed: Vec<String>,
        /// `mutants.out/timeout.txt`.
        timed_out: Vec<String>,
    },
    /// The unmutated tree failed to build or pass its tests: exit 4.
    BaselineFailed {
        /// The end of the run's output.
        log_tail: String,
    },
    /// rustc does not run in the checkout -- its `rust-toolchain.toml` names a toolchain the machine lacks --, so
    /// cargo-mutants was not started.
    ToolchainMissing {
        /// How rustc failed: rustup's message.
        message: String,
    },
    /// cargo-mutants could not do its job: any other exit, no start, or the run cap.
    ToolFailed {
        /// A one-line reason.
        reason: String,
        /// The end of the run's output.
        log_tail: String,
    },
}
