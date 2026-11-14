// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! One file written in the results repository.

use alloc::string::String;

/// One file written in the results repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchFile {
    /// The path from the results repository's root.
    pub path: String,
    /// What the file holds.
    pub contents: String,
}
