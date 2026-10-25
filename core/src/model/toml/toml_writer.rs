// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Writes values the parser reads back.

use std::fmt::Write as _;

/// Writes values the parser reads back.
#[derive(Debug, Clone, Copy, Default)]
pub struct TomlWriter;

impl TomlWriter {
    /// Quotes a string as a TOML basic string, on one line.
    pub fn quote(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len() + 2);
        out.push('"');
        for c in text.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                c if c.is_control() => {
                    let _ = write!(out, "\\u{:04X}", u32::from(c));
                }
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }
}
