// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! How a command ended.

/// How a command ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    /// It exited; `code` is `None` when killed by a signal.
    Exited {
        /// The exit code.
        code: Option<i32>,
        /// Standard output.
        stdout: String,
        /// Standard error.
        stderr: String,
    },
    /// It ran past its deadline and was killed.
    TimedOut {
        /// Standard output up to the kill.
        stdout: String,
        /// Standard error up to the kill.
        stderr: String,
    },
    /// It could not be started.
    FailedToStart(String),
}

impl CommandResult {
    /// A one-line description of a failure.
    pub fn describe(&self) -> String {
        match self {
            Self::Exited { code, stderr, .. } => {
                let code = code.map_or_else(|| "a signal".to_string(), |c| c.to_string());
                let first = stderr.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
                format!("exited with {code}: {}", first.trim())
            }
            Self::TimedOut { .. } => "timed out".to_string(),
            Self::FailedToStart(message) => format!("could not start: {message}"),
        }
    }

    /// Standard output if the command exited with 0.
    pub fn success(&self) -> Option<&str> {
        match self {
            Self::Exited {
                code: Some(0),
                stdout,
                ..
            } => Some(stdout),
            _ => None,
        }
    }
}
