// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The work directory's lock.

use std::fs::{self, File};
use std::path::Path;

use crate::invocation::startup_error::StartupError;

/// Holds the work directory's lock for the life of the process (design 5).
pub struct WorkDirLock {
    _file: File,
}

impl WorkDirLock {
    /// Creates `work_dir` if needed and takes its lock, or fails if another instance holds it.
    pub fn acquire(work_dir: &Path) -> Result<Self, StartupError> {
        fs::create_dir_all(work_dir).map_err(|e| StartupError::NoWorkDir {
            path: work_dir.to_path_buf(),
            reason: e.to_string(),
        })?;
        let path = work_dir.join("sentinel4rust.lock");
        let file = File::create(&path).map_err(|e| StartupError::NoLockFile {
            path: path.clone(),
            reason: e.to_string(),
        })?;
        file.try_lock().map_err(|_| StartupError::Locked {
            work_dir: work_dir.to_path_buf(),
            lock: path.clone(),
        })?;
        Ok(Self { _file: file })
    }
}
