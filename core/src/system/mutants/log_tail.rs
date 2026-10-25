// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! The end of a failing run's output.

/// The last `lines` lines, and at most `bytes` bytes, of a run's output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogTail {
    lines: usize,
    bytes: usize,
}

impl LogTail {
    /// Keeps at most `lines` lines and `bytes` bytes.
    pub fn new(lines: usize, bytes: usize) -> Self {
        Self { lines, bytes }
    }

    /// The tail of stdout followed by stderr, cut on a character boundary.
    pub fn of(&self, stdout: &str, stderr: &str) -> String {
        let combined = match (stdout.trim_end().is_empty(), stderr.trim_end().is_empty()) {
            (true, _) => stderr.trim_end().to_string(),
            (false, true) => stdout.trim_end().to_string(),
            (false, false) => format!("{}\n{}", stdout.trim_end(), stderr.trim_end()),
        };
        let lines: Vec<&str> = combined.lines().collect();
        let tail = lines[lines.len().saturating_sub(self.lines)..].join("\n");
        if tail.len() <= self.bytes {
            return tail;
        }
        let mut start = tail.len() - self.bytes;
        while !tail.is_char_boundary(start) {
            start += 1;
        }
        tail[start..].to_string()
    }
}
