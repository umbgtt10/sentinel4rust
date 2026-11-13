// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! One repository's line in the status pass.

use alloc::string::String;

/// One repository's line in the status pass, and whether it says something needs a look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusLine {
    /// `repo=… state=… …`.
    pub text: String,
    /// The repository could not be judged, `main` has waited longer than the roster allows, or the record warns
    /// that its timeouts outnumber its caught mutants.
    pub needs_attention: bool,
}
