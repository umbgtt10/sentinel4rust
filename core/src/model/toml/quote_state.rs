// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Where a scan of one line stands: in code, or inside a string.

/// Where a scan of one line stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteState {
    /// Outside any string.
    Code,
    /// Inside a basic string, `"..."`.
    Basic,
    /// Just after a backslash inside a basic string.
    BasicEscape,
    /// Inside a literal string, `'...'`.
    Literal,
}

impl QuoteState {
    /// The state after reading `c`.
    #[must_use]
    pub fn after(self, c: char) -> Self {
        match (self, c) {
            (Self::Code, '"') | (Self::BasicEscape, _) => Self::Basic,
            (Self::Code, '\'') => Self::Literal,
            (Self::Basic, '\\') => Self::BasicEscape,
            (Self::Basic, '"') | (Self::Literal, '\'') => Self::Code,
            (state, _) => state,
        }
    }

    /// Whether the scan is outside every string.
    pub fn is_code(self) -> bool {
        self == Self::Code
    }
}
