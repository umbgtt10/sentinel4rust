// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What a mutation run produced, and what one visit to a repository did.

use std::fmt;

use crate::model::commit_hash::CommitHash;
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
    /// cargo-mutants could not do its job: any other exit, no start, or the run cap.
    ToolFailed {
        /// A one-line reason.
        reason: String,
        /// The end of the run's output.
        log_tail: String,
    },
}

/// One run and the facts needed to record it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationRun {
    /// What happened.
    pub outcome: RunOutcome,
    /// `cargo mutants --version`.
    pub cargo_mutants_version: String,
    /// `rustc --version` in the checkout (so the repository's toolchain).
    pub rustc_version: String,
    /// The arguments given to cargo, joined.
    pub arguments: String,
}

/// The outcome as published in `summary.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeKind {
    /// Lists published.
    Completed,
    /// Red `main`: no lists.
    BaselineFailed,
    /// Repeated tool failure on this commit: no lists.
    ToolFailed,
}

impl OutcomeKind {
    /// The text used in `summary.toml` and the journal.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::BaselineFailed => "baseline-failed",
            Self::ToolFailed => "tool-failed",
        }
    }

    /// The inverse of [`OutcomeKind::as_str`].
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "completed" => Some(Self::Completed),
            "baseline-failed" => Some(Self::BaselineFailed),
            "tool-failed" => Some(Self::ToolFailed),
            _ => None,
        }
    }
}

impl fmt::Display for OutcomeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a visit ran nothing although `main` had moved (design 2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// The configured branch does not exist on the remote.
    NoBranch,
    /// No `Cargo.toml` at the root of `head`.
    NotACrate,
    /// No `.cargo/mutants.toml` at `head`.
    NoFilters,
}

impl SkipReason {
    /// The text used in the journal.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoBranch => "no-branch",
            Self::NotACrate => "not-a-crate",
            Self::NoFilters => "no-filters",
        }
    }
}

/// What one visit to a repository did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoundStep {
    /// The record on `mutants` is current for `head`.
    Unchanged {
        /// `main`'s commit.
        head: CommitHash,
    },
    /// A record was published for `head`.
    Published {
        /// The commit tested.
        head: CommitHash,
        /// What was published.
        outcome: OutcomeKind,
        /// The counts published (zero unless completed).
        counts: MutantCounts,
        /// Survivors not in the previous record.
        new: u64,
        /// Survivors of the previous record no longer present.
        gone: u64,
    },
    /// Something failed; the repository waits before its next visit.
    BackedOff {
        /// A one-line reason.
        reason: String,
    },
    /// The repository is gone or refuses the key.
    Unreachable {
        /// git's message.
        reason: String,
    },
    /// Nothing to run yet.
    Skipped {
        /// Why.
        reason: SkipReason,
    },
}

impl RoundStep {
    /// The `step=` value in the journal.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unchanged { .. } => "unchanged",
            Self::Published { .. } => "published",
            Self::BackedOff { .. } => "backed-off",
            Self::Unreachable { .. } => "unreachable",
            Self::Skipped { .. } => "skipped",
        }
    }
}
