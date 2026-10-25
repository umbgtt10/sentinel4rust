// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A command line that cannot be used.

use std::fmt;

/// A command line that cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageError {
    /// `--roster` came last, with no file after it.
    MissingRosterFile,
    /// No `--roster` at all.
    MissingRoster,
    /// An argument nobody knows.
    UnknownArgument(String),
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRosterFile => write!(f, "--roster needs a file"),
            Self::MissingRoster => write!(f, "--roster is required"),
            Self::UnknownArgument(argument) => write!(f, "unknown argument `{argument}`"),
        }
    }
}
