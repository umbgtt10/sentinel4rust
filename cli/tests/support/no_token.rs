// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::model::github::discovery_error::DiscoveryError;
use sentinel4rust_cli::system::github::token_source::TokenSource;

// A token source that cannot answer, as when GitHub is out of reach.
pub struct NoToken;

impl TokenSource for NoToken {
    fn token(&self) -> Result<String, DiscoveryError> {
        Err(DiscoveryError(String::from("curl: exited with 6")))
    }
}
