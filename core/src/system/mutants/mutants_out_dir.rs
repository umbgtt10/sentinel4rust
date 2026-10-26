// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The `mutants.out` directory a run wrote, on disk.

use std::fs;
use std::path::PathBuf;

use crate::system::mutants::unreadable_list::UnreadableList;
use crate::traits::mutants_out::MutantsOut;

/// Reads the lists in one `mutants.out` directory.
pub struct MutantsOutDir {
    dir: PathBuf,
}

impl MutantsOutDir {
    /// The lists in `dir`.
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

impl MutantsOut for MutantsOutDir {
    fn list(&self, name: &str) -> Result<String, UnreadableList> {
        fs::read_to_string(self.dir.join(name)).map_err(|error| UnreadableList {
            name: name.to_string(),
            reason: error.to_string(),
        })
    }
}
