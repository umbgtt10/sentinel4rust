// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Why a visit ran nothing although `main` had moved.

/// Why a visit ran nothing although `main` had moved (`ADR-FiltersLiveInEachRepository`).
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
