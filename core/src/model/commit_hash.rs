// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A validated full commit hash.

use std::fmt;

/// A 40-character lowercase hexadecimal git commit hash.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommitHash(String);

impl CommitHash {
    /// Parses a hash, trimming whitespace and lowercasing; `None` unless 40 hex digits.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim().to_ascii_lowercase();
        (text.len() == 40 && text.bytes().all(|b| b.is_ascii_hexdigit())).then_some(Self(text))
    }

    /// The full hash.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The first seven characters.
    pub fn short(&self) -> &str {
        &self.0[..7]
    }
}

impl fmt::Display for CommitHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
