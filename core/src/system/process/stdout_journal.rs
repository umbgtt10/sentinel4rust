// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The journal on standard output.

use crate::traits::journal::Journal;

/// Writes journal lines to standard output (systemd adds the timestamp).
#[derive(Debug, Default)]
pub struct StdoutJournal;

impl Journal for StdoutJournal {
    fn line(&self, text: &str) {
        println!("{text}");
    }
}
