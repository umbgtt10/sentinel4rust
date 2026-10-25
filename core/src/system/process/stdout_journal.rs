// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The journal on standard output.

use std::cell::RefCell;
use std::io::Write;

use crate::traits::journal::Journal;

/// Writes journal lines to standard output (systemd adds the timestamp), or to the writer a test hands it.
pub struct StdoutJournal {
    out: RefCell<Box<dyn Write>>,
}

impl StdoutJournal {
    /// A journal writing to `out`: `io::stdout()` in the service.
    pub fn new(out: Box<dyn Write>) -> Self {
        Self {
            out: RefCell::new(out),
        }
    }
}

impl Journal for StdoutJournal {
    fn line(&self, text: &str) {
        let _ = writeln!(self.out.borrow_mut(), "{text}");
    }
}
