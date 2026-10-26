// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Why a visit ran nothing although `main` had moved.

use alloc::string::String;

/// Why a visit ran nothing although `main` had moved (`ADR-FiltersLiveInEachRepository`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The configured branch does not exist on the remote.
    NoBranch,
    /// No `Cargo.toml` at the root of `head`.
    NotACrate,
    /// No `.cargo/mutants.toml` at `head`.
    NoFilters,
    /// A manifest at `head` has a `path` dependency outside the repository, which no checkout holds.
    PathOutsideRepo {
        /// The manifest, relative to the repository's root.
        manifest: String,
    },
    /// rustc does not run in the checkout: the toolchain its `rust-toolchain.toml` names is not installed.
    ToolchainMissing {
        /// How rustc failed: rustup's message.
        message: String,
    },
}

impl SkipReason {
    /// The text used in the journal.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoBranch => "no-branch",
            Self::NotACrate => "not-a-crate",
            Self::NoFilters => "no-filters",
            Self::PathOutsideRepo { .. } => "path-outside-repo",
            Self::ToolchainMissing { .. } => "toolchain-missing",
        }
    }
}
