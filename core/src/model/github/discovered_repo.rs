// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A repository the GitHub App can read, with a `.cargo/mutants.toml` on its default branch.

use alloc::string::String;

/// A repository discovery found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredRepo {
    /// Its name, without the owner.
    pub name: String,
    /// What git fetches it from.
    pub url: String,
    /// Its default branch.
    pub branch: String,
}
