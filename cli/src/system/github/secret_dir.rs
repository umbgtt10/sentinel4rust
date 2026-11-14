// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A directory of private files, for what must not reach a command line.

use std::fs::{self, OpenOptions, Permissions};
use std::io::Write as _;
use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};
use std::path::PathBuf;

use sentinel4rust::model::github::discovery_error::DiscoveryError;
use std::io;

/// A directory only its owner can enter, of files only its owner can read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretDir {
    dir: PathBuf,
}

impl SecretDir {
    /// The directory `dir`, made when first written to.
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Where the file `name` is.
    pub fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// Writes `contents` to the file `name`, mode 600, in the directory, mode 700; returns its path.
    pub fn write(&self, name: &str, contents: &[u8]) -> Result<PathBuf, DiscoveryError> {
        let failed = |what: &str, e: io::Error| DiscoveryError(format!("{what}: {e}"));
        fs::create_dir_all(&self.dir)
            .map_err(|e| failed(&format!("create {}", self.dir.display()), e))?;
        fs::set_permissions(&self.dir, Permissions::from_mode(0o700))
            .map_err(|e| failed(&format!("chmod {}", self.dir.display()), e))?;
        let path = self.path(name);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .map_err(|e| failed(&format!("open {}", path.display()), e))?;
        fs::set_permissions(&path, Permissions::from_mode(0o600))
            .map_err(|e| failed(&format!("chmod {}", path.display()), e))?;
        file.write_all(contents)
            .map_err(|e| failed(&format!("write {}", path.display()), e))?;
        Ok(path)
    }
}
