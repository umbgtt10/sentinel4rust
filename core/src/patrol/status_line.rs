// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! One repository's line in the status pass.

/// One repository's line in the status pass, and whether it says something needs a look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusLine {
    /// `repo=… state=… …`.
    pub text: String,
    /// The repository could not be judged, or `main` has waited longer than the roster allows.
    pub needs_attention: bool,
}
