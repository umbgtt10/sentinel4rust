// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What the command line asked for.

use std::path::PathBuf;

use crate::invocation::usage_error::UsageError;

/// What the command line asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandLine {
    /// Run with this roster.
    Run {
        /// The roster file.
        roster: PathBuf,
        /// One pass and exit.
        once: bool,
    },
    /// One read-only pass reporting how far each repository's record is behind.
    Status {
        /// The roster file.
        roster: PathBuf,
    },
    /// Print usage.
    Help,
    /// Print the version.
    Version,
}

impl CommandLine {
    /// Parses the arguments after the program name.
    pub fn parse(args: &[String]) -> Result<Self, UsageError> {
        let mut roster = None;
        let mut once = false;
        let mut status = false;
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--help" | "-h" => return Ok(Self::Help),
                "--version" | "-V" => return Ok(Self::Version),
                "--once" => once = true,
                "--status" => status = true,
                "--roster" => {
                    let value = iter.next().ok_or(UsageError::MissingRosterFile)?;
                    roster = Some(PathBuf::from(value));
                }
                other => {
                    if let Some(value) = other.strip_prefix("--roster=") {
                        roster = Some(PathBuf::from(value));
                    } else {
                        return Err(UsageError::UnknownArgument(other.to_string()));
                    }
                }
            }
        }
        let roster = roster.ok_or(UsageError::MissingRoster)?;
        if status {
            Ok(Self::Status { roster })
        } else {
            Ok(Self::Run { roster, once })
        }
    }
}
