// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fmt;

// A file the store could not write, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFailure {
    pub path: String,
    pub reason: String,
}

impl fmt::Display for FileFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "failed to write {}: {}", self.path, self.reason)
    }
}
