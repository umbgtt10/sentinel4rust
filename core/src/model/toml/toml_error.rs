// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A parse or type error, with its line.

use std::error::Error;
use std::fmt;

/// A parse or type error, with its line (1-based; 0 when unknown).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomlError {
    /// The line the error is on.
    pub line: usize,
    /// What went wrong.
    pub message: String,
}

impl fmt::Display for TomlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl Error for TomlError {}
