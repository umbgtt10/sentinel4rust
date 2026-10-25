// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A git command that failed.

use std::fmt;

use crate::model::command_result::CommandResult;

/// A git command that failed: what it was, and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitFailure {
    /// The command, as the journal names it.
    pub what: String,
    /// How it ended.
    pub result: CommandResult,
}

impl fmt::Display for GitFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.what, self.result.describe())
    }
}
