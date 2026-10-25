// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Telling a repository that is gone from a network that is down.

use crate::model::command_result::CommandResult;
use crate::model::sync_error::SyncError;

/// git messages meaning the repository is gone or refuses the key.
const UNREACHABLE_MARKERS: &[&str] = &[
    "repository not found",
    "permission denied",
    "does not appear to be a git repository",
    "access denied",
    "could not read username",
    "authentication failed",
];

/// Reads a failed fetch's stderr for the messages that mean the repository is unreachable.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnreachableClassifier;

impl UnreachableClassifier {
    /// `Unreachable` when git said the repository is gone or refuses the key, `Transient` otherwise.
    pub fn classify(&self, result: &CommandResult, what: &str) -> SyncError {
        let message = format!("{what}: {}", result.describe());
        let text = match result {
            CommandResult::Exited { stderr, .. } => stderr.to_lowercase(),
            _ => String::new(),
        };
        if UNREACHABLE_MARKERS.iter().any(|m| text.contains(m)) {
            SyncError::Unreachable(message)
        } else {
            SyncError::Transient(message)
        }
    }
}
