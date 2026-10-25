// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The outcome as published in `summary.toml`.

use std::fmt;

/// The outcome as published in `summary.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeKind {
    /// Lists published.
    Completed,
    /// Red `main`: no lists.
    BaselineFailed,
    /// Repeated tool failure on this commit: no lists.
    ToolFailed,
}

impl OutcomeKind {
    /// The inverse of [`OutcomeKind::as_str`].
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "completed" => Some(Self::Completed),
            "baseline-failed" => Some(Self::BaselineFailed),
            "tool-failed" => Some(Self::ToolFailed),
            _ => None,
        }
    }

    /// The text used in `summary.toml` and the journal.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::BaselineFailed => "baseline-failed",
            Self::ToolFailed => "tool-failed",
        }
    }
}

impl fmt::Display for OutcomeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
