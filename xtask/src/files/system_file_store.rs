// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::files::file_failure::FileFailure;
use crate::files::file_store::FileStore;
use std::fs::read_to_string;
use std::fs::write;
use std::path::Path;

pub struct SystemFileStore;

impl SystemFileStore {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemFileStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FileStore for SystemFileStore {
    fn is_file(&self, path: &str) -> bool {
        Path::new(path).is_file()
    }

    fn read(&self, path: &str) -> Option<String> {
        read_to_string(path).ok()
    }

    fn write(&self, path: &str, contents: &str) -> Result<(), FileFailure> {
        write(path, contents).map_err(|error| FileFailure {
            path: String::from(path),
            reason: error.to_string(),
        })
    }
}
