// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! One file written on the `mutants` branch.

/// One file written on the `mutants` branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchFile {
    /// The path inside the branch.
    pub path: String,
    /// What the file holds.
    pub contents: String,
}
