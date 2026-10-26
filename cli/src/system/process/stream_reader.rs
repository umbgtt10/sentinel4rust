// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Reading a child's output while it runs.

use std::io::Read;
use std::thread::{self, JoinHandle};

/// Drains a child's stream on a thread of its own, so a full pipe never blocks the child, and keeps all of it or
/// its tail.
#[derive(Debug, Clone, Copy, Default)]
pub struct StreamReader;

impl StreamReader {
    /// Reads `stream` to its end on a new thread, keeping only its last `limit` bytes when there is a limit; the
    /// text, lossily decoded, is what the thread returns.
    pub fn spawn(
        &self,
        stream: Option<impl Read + Send + 'static>,
        limit: Option<usize>,
    ) -> JoinHandle<String> {
        thread::spawn(move || {
            let mut kept = Vec::new();
            if let Some(mut stream) = stream {
                let mut chunk = [0_u8; 8192];
                while let Ok(read) = stream.read(&mut chunk) {
                    if read == 0 {
                        break;
                    }
                    kept.extend_from_slice(&chunk[..read]);
                    if let Some(limit) = limit
                        && kept.len() > limit.saturating_mul(2)
                    {
                        kept.drain(..kept.len() - limit);
                    }
                }
            }
            if let Some(limit) = limit
                && kept.len() > limit
            {
                kept.drain(..kept.len() - limit);
            }
            String::from_utf8_lossy(&kept).into_owned()
        })
    }
}
