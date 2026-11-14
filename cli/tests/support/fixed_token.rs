// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::github::discovery_error::DiscoveryError;
use sentinel4rust_cli::system::github::token_source::TokenSource;

// A token source that always answers the same token.
pub struct FixedToken(pub String);

impl TokenSource for FixedToken {
    fn token(&self) -> Result<String, DiscoveryError> {
        Ok(self.0.clone())
    }
}
