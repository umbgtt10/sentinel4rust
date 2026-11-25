// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Cuts a line's comment, leaving a `#` inside a string alone.

use crate::model::toml::quote_state::QuoteState;
use crate::model::toml::toml_error::TomlError;
use alloc::string::String;

/// Cuts a line's comment, leaving a `#` inside a string alone.
#[derive(Debug, Clone, Copy, Default)]
pub struct CommentStripper;

impl CommentStripper {
    /// The line up to its comment; `None` if a string is left open.
    pub fn code<'a>(&self, line: &'a str) -> Option<&'a str> {
        let mut state = QuoteState::Code;
        for (i, c) in line.char_indices() {
            if state.is_code() && c == '#' {
                return Some(&line[..i]);
            }
            state = state.after(c);
        }
        state.is_code().then_some(line)
    }

    /// The line up to its comment; an error naming `line_no` if a string is left open.
    pub fn strip<'a>(&self, line: &'a str, line_no: usize) -> Result<&'a str, TomlError> {
        self.code(line).ok_or_else(|| TomlError {
            line: line_no,
            message: String::from("unterminated string"),
        })
    }
}
