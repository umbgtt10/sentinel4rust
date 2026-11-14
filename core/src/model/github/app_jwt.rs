// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The JSON Web Token a GitHub App signs to ask for an installation token, short of the signature itself, which
//! the private key makes elsewhere.

use alloc::format;
use alloc::string::String;

use crate::model::github::base64::Base64;
use crate::model::timestamp::Timestamp;

/// `{"alg":"RS256","typ":"JWT"}`, encoded.
const HEADER: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9";
/// Issued this long before now, against a clock a little ahead of GitHub's.
const BACKDATE_SECONDS: i64 = 60;
/// Valid this long after now; GitHub refuses more than ten minutes.
const LIFETIME_SECONDS: i64 = 540;

/// The App's token, for one App.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppJwt {
    app_id: u64,
}

impl AppJwt {
    /// For the App `app_id`.
    pub const fn new(app_id: u64) -> Self {
        Self { app_id }
    }

    /// The header and the claims, encoded and joined: what RS256 signs.
    pub fn signing_input(&self, now: Timestamp) -> String {
        let now = now.unix();
        let claims = format!(
            r#"{{"iat":{},"exp":{},"iss":"{}"}}"#,
            now - BACKDATE_SECONDS,
            now + LIFETIME_SECONDS,
            self.app_id
        );
        format!("{HEADER}.{}", Base64.url(claims.as_bytes()))
    }

    /// The token: the signing input and its signature.
    pub fn token(&self, signing_input: &str, signature: &[u8]) -> String {
        format!("{signing_input}.{}", Base64.url(signature))
    }
}
