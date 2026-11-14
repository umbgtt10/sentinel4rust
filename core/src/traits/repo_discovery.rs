// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Finding the repositories to watch.

use alloc::vec::Vec;

use crate::model::github::discovered_repo::DiscoveredRepo;
use crate::model::github::discovery_error::DiscoveryError;

/// Finds the repositories to watch.
pub trait RepoDiscovery {
    /// Every repository that asks to be watched, by carrying a `.cargo/mutants.toml` on its default branch.
    fn discover(&self) -> Result<Vec<DiscoveredRepo>, DiscoveryError>;
}
