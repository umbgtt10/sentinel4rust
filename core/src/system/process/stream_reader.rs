// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Reading a child's output while it runs.

use std::io::Read;
use std::thread::{self, JoinHandle};

/// Drains a child's stream on a thread of its own, so a full pipe never blocks the child.
#[derive(Debug, Clone, Copy, Default)]
pub struct StreamReader;

impl StreamReader {
    /// Reads `stream` to its end on a new thread; the text, lossily decoded, is what the thread returns.
    pub fn spawn(&self, stream: Option<impl Read + Send + 'static>) -> JoinHandle<String> {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut stream) = stream {
                let _ = stream.read_to_end(&mut bytes);
            }
            String::from_utf8_lossy(&bytes).into_owned()
        })
    }
}
