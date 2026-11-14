// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A token for GitHub's API and for git over https.

use sentinel4rust::model::github::discovery_error::DiscoveryError;

/// Answers a token GitHub accepts now.
pub trait TokenSource {
    /// A token valid for at least a few more minutes.
    fn token(&self) -> Result<String, DiscoveryError>;
}
