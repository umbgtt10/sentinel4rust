// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What one visit to a repository did.

use crate::model::commit_hash::CommitHash;
use crate::model::counts::MutantCounts;
use crate::model::outcome_kind::OutcomeKind;
use crate::model::skip_reason::SkipReason;
use alloc::string::String;

/// What one visit to a repository did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoundStep {
    /// The record in the results repository is current for `head`.
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
    /// The record in the results repository does not parse; the repository waits until it does.
    Unreadable {
        /// Why it does not parse.
        reason: String,
    },
}

impl RoundStep {
    /// Whether a `--once` pass counts this visit as failed: it backed off, or found the repository unreachable or
    /// its record unreadable.
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            Self::BackedOff { .. } | Self::Unreachable { .. } | Self::Unreadable { .. }
        )
    }

    /// The `step=` value in the journal.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unchanged { .. } => "unchanged",
            Self::Published { .. } => "published",
            Self::BackedOff { .. } => "backed-off",
            Self::Unreachable { .. } => "unreachable",
            Self::Skipped { .. } => "skipped",
            Self::Unreadable { .. } => "unreadable",
        }
    }
}
