// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use sentinel4rust::system::process::stdout_journal::StdoutJournal;
use sentinel4rust::traits::journal::Journal;
use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;

// A writer the test can read back: the journal gets one handle, the test
// keeps the other.
#[derive(Clone, Default)]
struct SharedBuffer(Rc<RefCell<Vec<u8>>>);

impl Write for SharedBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn line_writes_each_line_with_its_newline() {
    // Arrange
    let buffer = SharedBuffer::default();
    let journal = StdoutJournal::new(Box::new(buffer.clone()));

    // Act
    journal.line("repo=a step=unchanged hash=aaaaaaa duration_s=0");
    journal.line("repo=b step=skipped reason=no-filters duration_s=0");

    // Assert
    assert_eq!(
        String::from_utf8(buffer.0.borrow().clone()).unwrap(),
        "repo=a step=unchanged hash=aaaaaaa duration_s=0\nrepo=b step=skipped reason=no-filters duration_s=0\n"
    );
}
