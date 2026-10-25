// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::files::file_failure::FileFailure;

pub trait FileStore {
    fn is_file(&self, path: &str) -> bool;

    fn read(&self, path: &str) -> Option<String>;

    fn write(&self, path: &str, contents: &str) -> Result<(), FileFailure>;
}
