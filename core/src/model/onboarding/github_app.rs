// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The GitHub App the sentinel discovers repositories through (`ADR-GithubAppDiscovery`).

use alloc::string::String;

/// The GitHub App the sentinel discovers and reads repositories through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubApp {
    /// The App's id.
    pub app_id: u64,
    /// Its installation on the owner's account.
    pub installation_id: u64,
    /// The path of its private key, where the run unit cannot read it.
    pub key: String,
}
