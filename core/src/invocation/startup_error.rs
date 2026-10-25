// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Why the service could not start.

use std::fmt;
use std::path::PathBuf;

use crate::model::onboarding::roster_error::RosterError;
use crate::system::unit::job_error::JobError;

/// Why the service could not start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupError {
    /// The roster file could not be read.
    Unreadable {
        /// The roster file.
        path: PathBuf,
        /// What reading it said.
        reason: String,
    },
    /// The roster cannot be used.
    Invalid(RosterError),
    /// The work directory could not be created.
    NoWorkDir {
        /// The work directory.
        path: PathBuf,
        /// What creating it said.
        reason: String,
    },
    /// The lock file could not be opened.
    NoLockFile {
        /// The lock file.
        path: PathBuf,
        /// What opening it said.
        reason: String,
    },
    /// The run unit's job could not be read, or its result written.
    Job(JobError),
    /// Another instance holds the work directory.
    Locked {
        /// The work directory.
        work_dir: PathBuf,
        /// Its lock file.
        lock: PathBuf,
    },
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, reason } => write!(f, "read {}: {reason}", path.display()),
            Self::Invalid(error) => write!(f, "{error}"),
            Self::Job(error) => write!(f, "{error}"),
            Self::NoWorkDir { path, reason } => write!(f, "create {}: {reason}", path.display()),
            Self::NoLockFile { path, reason } => write!(f, "open {}: {reason}", path.display()),
            Self::Locked { work_dir, lock } => write!(
                f,
                "another sentinel4rust is running on {} (lock {})",
                work_dir.display(),
                lock.display()
            ),
        }
    }
}

impl From<JobError> for StartupError {
    fn from(error: JobError) -> Self {
        Self::Job(error)
    }
}

impl From<RosterError> for StartupError {
    fn from(error: RosterError) -> Self {
        Self::Invalid(error)
    }
}
