// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A `mutants.out` list that could not be read.

use alloc::string::String;
use core::fmt;

/// A `mutants.out` list that could not be read, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnreadableList {
    /// The file, e.g. `caught.txt`.
    pub name: String,
    /// What reading it said.
    pub reason: String,
}

impl fmt::Display for UnreadableList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mutants.out/{} unreadable: {}", self.name, self.reason)
    }
}
